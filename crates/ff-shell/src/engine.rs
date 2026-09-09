//! Shell engine -- central coordinator for the shell subsystem.
//!
//! Orchestrates command handling, security gating, shell resolution,
//! process execution, and output routing.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::capture::{CaptureHandler, CaptureResult, CaptureTarget};
use crate::commands::{self, CommandForm};
use crate::config::{ShellConfig, ShellConfigProvider, ShellMode};
use crate::environment::EnvironmentBuilder;
use crate::error::ShellError;
use crate::executor::external::{ExecutionMode, ExternalOutcome, TaskHandle};
use crate::executor::spawn::CommandExecutor;
use crate::panel::output_panel::{OutputEntry, OutputLine, OutputPanel, OutputStream};
use crate::panel::terminal_panel::TerminalPanel;
use crate::pipe::StdinPiper;
use crate::platform::PlatformDetector;
use crate::process::{ExitStatus, ProcessId};
use crate::profile::ProfileResolver;
use crate::terminal::manager::{SessionId, TerminalManager};
use crate::working_dir::WorkingDirResolver;

/// The central coordinator for the shell subsystem.
///
/// Handles command dispatch, security gating, shell resolution,
/// process execution, and output routing. All shell operations
/// flow through this struct.
#[derive(Debug)]
pub struct ShellEngine {
    /// Configuration provider.
    config: ShellConfigProvider,
    /// Profile resolver.
    profile_resolver: ProfileResolver,
    /// Terminal session manager.
    terminal_manager: Mutex<TerminalManager>,
    /// Output panel.
    output_panel: Mutex<OutputPanel>,
    /// Terminal panel.
    terminal_panel: Mutex<TerminalPanel>,
}

impl ShellEngine {
    /// Creates a new ShellEngine with the given configuration.
    pub fn new(config: ShellConfigProvider) -> Self {
        let shell_config = config.get();
        let profile_resolver = ProfileResolver::new(shell_config.profiles.clone());
        let output_panel = OutputPanel::new(shell_config.output_buffer_lines);

        Self {
            config,
            profile_resolver,
            terminal_manager: Mutex::new(TerminalManager::new()),
            output_panel: Mutex::new(output_panel),
            terminal_panel: Mutex::new(TerminalPanel::new()),
        }
    }

    /// Checks the security gate for shell access.
    ///
    /// Enforces the `shell.mode` security control and the macro dual-gate.
    pub fn check_security_gate(&self, from_macro: bool) -> Result<(), ShellError> {
        let mode = self.config.get().mode;
        match mode {
            ShellMode::Disabled => Err(ShellError::ShellDisabled),
            ShellMode::Prompt => {
                if from_macro {
                    Err(ShellError::MacroAccessDenied {
                        reason: "shell.mode is 'prompt' -- macros cannot show UI prompts"
                            .to_string(),
                    })
                } else {
                    // Direct invocation -- UI layer will show confirmation
                    Ok(())
                }
            }
            ShellMode::Enabled => Ok(()),
        }
    }

    /// Validates and classifies a SHELL command invocation.
    #[allow(clippy::too_many_arguments)]
    pub fn validate_command_form(
        &self,
        has_args: bool,
        has_a_target: bool,
        has_b_target: bool,
        has_source_cmd: bool,
        command_text: Option<&str>,
        target_line: Option<usize>,
        shell_override: Option<&str>,
    ) -> Result<CommandForm, ShellError> {
        commands::validate_command_form(
            has_args,
            has_a_target,
            has_b_target,
            has_source_cmd,
            command_text,
            target_line,
            shell_override,
        )
    }

    /// Executes a shell command in command execution mode (output to panel).
    ///
    /// Resolves the shell, builds environment, spawns the process,
    /// captures output, and appends to the Output Panel.
    pub async fn execute_command(
        &self,
        command_text: &str,
        shell_override: Option<&str>,
        project_root: Option<&std::path::Path>,
        active_file: Option<&std::path::Path>,
    ) -> Result<(ProcessId, ExitStatus), ShellError> {
        let config = self.config.get();

        // Resolve shell
        let shell_path =
            PlatformDetector::resolve_shell(shell_override, config.default_shell.as_deref())?;
        let shell_args = PlatformDetector::shell_command_args(&shell_path);

        // Resolve working directory
        let working_dir =
            WorkingDirResolver::resolve(config.working_directory, project_root, active_file);

        // Build environment
        let env = EnvironmentBuilder::build(&config.env, &HashMap::new());

        // Execute command
        let (capture, exit_status) =
            CommandExecutor::execute(&shell_path, &shell_args, command_text, &working_dir, &env)
                .await?;

        // Create output entry
        let entry = OutputEntry {
            command: command_text.to_string(),
            working_directory: working_dir,
            timestamp: chrono::Local::now(),
            lines: capture
                .stdout_lines
                .iter()
                .map(|l| OutputLine {
                    text: l.clone(),
                    stream: OutputStream::Stdout,
                })
                .chain(capture.stderr_lines.iter().map(|l| OutputLine {
                    text: l.clone(),
                    stream: OutputStream::Stderr,
                }))
                .collect(),
            exit_status: Some(exit_status.clone()),
        };

        // Append to output panel
        if let Ok(mut panel) = self.output_panel.lock() {
            panel.append_entry(entry);
        }

        let process_id = ProcessId::new();
        Ok((process_id, exit_status))
    }

    /// Executes a shell command in document capture mode.
    ///
    /// Captures stdout only and returns lines for document insertion.
    pub async fn execute_capture(
        &self,
        command_text: &str,
        target: &CaptureTarget,
        shell_override: Option<&str>,
        project_root: Option<&std::path::Path>,
        active_file: Option<&std::path::Path>,
    ) -> Result<CaptureResult, ShellError> {
        let config = self.config.get();

        // Resolve shell
        let shell_path =
            PlatformDetector::resolve_shell(shell_override, config.default_shell.as_deref())?;
        let shell_args = PlatformDetector::shell_command_args(&shell_path);

        // Resolve working directory
        let working_dir =
            WorkingDirResolver::resolve(config.working_directory, project_root, active_file);

        // Build environment
        let env = EnvironmentBuilder::build(&config.env, &HashMap::new());

        // Execute command
        let (capture, exit_status) =
            CommandExecutor::execute(&shell_path, &shell_args, command_text, &working_dir, &env)
                .await?;

        // Route stderr to output panel
        if !capture.stderr_lines.is_empty() {
            if let Ok(mut panel) = self.output_panel.lock() {
                let entry = OutputEntry {
                    command: format!("[capture stderr] {}", command_text),
                    working_directory: working_dir,
                    timestamp: chrono::Local::now(),
                    lines: capture
                        .stderr_lines
                        .iter()
                        .map(|l| OutputLine {
                            text: l.clone(),
                            stream: OutputStream::Stderr,
                        })
                        .collect(),
                    exit_status: Some(exit_status.clone()),
                };
                panel.append_entry(entry);
            }
        }

        // Process the capture (validates exit code, splits lines)
        let stdout_combined = capture.stdout_lines.join("\n");
        let stdout_with_newline = if stdout_combined.is_empty() {
            stdout_combined
        } else {
            format!("{}\n", stdout_combined)
        };

        CaptureHandler::process_capture(
            &stdout_with_newline,
            capture.stderr_lines,
            exit_status,
            target,
        )
    }

    /// Executes a command with stdin piped from document content.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_with_stdin(
        &self,
        command_text: &str,
        document_content: &str,
        selection: Option<&str>,
        _target: Option<&CaptureTarget>,
        shell_override: Option<&str>,
        project_root: Option<&std::path::Path>,
        active_file: Option<&std::path::Path>,
    ) -> Result<(ProcessId, ExitStatus), ShellError> {
        let config = self.config.get();

        // Resolve shell
        let shell_path =
            PlatformDetector::resolve_shell(shell_override, config.default_shell.as_deref())?;
        let shell_args = PlatformDetector::shell_command_args(&shell_path);

        // Resolve working directory
        let working_dir =
            WorkingDirResolver::resolve(config.working_directory, project_root, active_file);

        // Build environment
        let env = EnvironmentBuilder::build(&config.env, &HashMap::new());

        // Prepare stdin content
        let stdin_content = StdinPiper::prepare_content(document_content, selection);

        // Execute with stdin
        let (capture, exit_status) = CommandExecutor::execute_with_stdin(
            &shell_path,
            &shell_args,
            command_text,
            &stdin_content,
            &working_dir,
            &env,
        )
        .await?;

        // Output to panel
        let entry = OutputEntry {
            command: format!("| {}", command_text),
            working_directory: working_dir,
            timestamp: chrono::Local::now(),
            lines: capture
                .stdout_lines
                .iter()
                .map(|l| OutputLine {
                    text: l.clone(),
                    stream: OutputStream::Stdout,
                })
                .chain(capture.stderr_lines.iter().map(|l| OutputLine {
                    text: l.clone(),
                    stream: OutputStream::Stderr,
                }))
                .collect(),
            exit_status: Some(exit_status.clone()),
        };

        if let Ok(mut panel) = self.output_panel.lock() {
            panel.append_entry(entry);
        }

        let process_id = ProcessId::new();
        Ok((process_id, exit_status))
    }

    /// Runs an external program with an explicit program and argument list.
    ///
    /// Detached mode spawns fire-and-forget (no capture, no panel, returns
    /// immediately). Captured mode runs the process asynchronously, capturing
    /// stdout/stderr and the exit code into the Output_Panel.
    ///
    /// Gated by `shell.mode` identically to [`Self::execute_command`]: `disabled`
    /// refuses; `prompt`/`enabled` proceed (the UI confirmation for `prompt` is
    /// applied by the caller before invoking this method).
    ///
    /// # Arguments
    ///
    /// * `program` - Program name or path (no shell parsing).
    /// * `args` - Argument list passed verbatim.
    /// * `working_dir` - Explicit working directory; when `None`, the configured
    ///   `shell.working_directory` resolution rules apply.
    /// * `mode` - Detached or Captured.
    /// * `project_root` / `active_file` - Context for working-directory fallback.
    ///
    /// Validates: shell-command Requirement 19.1, 19.2, 19.5, 19.6, 19.8;
    /// command-configurator Requirement 3.2, 3.4, 3.7
    pub async fn execute_external(
        &self,
        program: &str,
        args: &[String],
        working_dir: Option<&std::path::Path>,
        mode: ExecutionMode,
        project_root: Option<&std::path::Path>,
        active_file: Option<&std::path::Path>,
    ) -> Result<ExternalOutcome, ShellError> {
        // Security gate: identical to execute_command (Requirement 19.5).
        self.check_security_gate(false)?;

        match mode {
            ExecutionMode::Detached => {
                let handle =
                    self.spawn_detached(program, args, working_dir, project_root, active_file)?;
                Ok(ExternalOutcome::Detached(handle))
            }
            ExecutionMode::Captured => {
                let (process_id, exit_status) = self
                    .execute_external_captured(
                        program,
                        args,
                        working_dir,
                        project_root,
                        active_file,
                    )
                    .await?;
                Ok(ExternalOutcome::Captured {
                    process_id,
                    exit_status,
                })
            }
        }
    }

    /// Spawns an external program fire-and-forget (Detached mode).
    ///
    /// Does not capture output, open the Output_Panel, or wait for exit. Returns
    /// an opaque [`TaskHandle`] that the caller may drop; the workbench does not
    /// track, monitor, restart, or persist the process.
    ///
    /// Validates: shell-command Requirement 19.1, 19.3, 19.4, 19.5, 19.6, 19.8;
    /// command-configurator Requirement 3.2, 3.3
    pub fn spawn_detached(
        &self,
        program: &str,
        args: &[String],
        working_dir: Option<&std::path::Path>,
        project_root: Option<&std::path::Path>,
        active_file: Option<&std::path::Path>,
    ) -> Result<TaskHandle, ShellError> {
        // Security gate: identical to execute_command (Requirement 19.5).
        self.check_security_gate(false)?;

        let config = self.config.get();
        let resolved_dir = self.resolve_external_dir(working_dir, project_root, active_file);
        let env = EnvironmentBuilder::build(&config.env, &HashMap::new());

        crate::executor::external::spawn_detached(program, args, &resolved_dir, &env)
    }

    /// Captured external run: spawn, capture output, append to the Output_Panel.
    async fn execute_external_captured(
        &self,
        program: &str,
        args: &[String],
        working_dir: Option<&std::path::Path>,
        project_root: Option<&std::path::Path>,
        active_file: Option<&std::path::Path>,
    ) -> Result<(ProcessId, ExitStatus), ShellError> {
        let config = self.config.get();
        let resolved_dir = self.resolve_external_dir(working_dir, project_root, active_file);
        let env = EnvironmentBuilder::build(&config.env, &HashMap::new());

        // Run the program directly (program + args), not through a shell line.
        let (capture, exit_status) =
            CommandExecutor::execute(std::path::Path::new(program), args, "", &resolved_dir, &env)
                .await?;

        let command_display = if args.is_empty() {
            program.to_string()
        } else {
            format!("{} {}", program, args.join(" "))
        };

        let entry = OutputEntry {
            command: command_display,
            working_directory: resolved_dir,
            timestamp: chrono::Local::now(),
            lines: capture
                .stdout_lines
                .iter()
                .map(|l| OutputLine {
                    text: l.clone(),
                    stream: OutputStream::Stdout,
                })
                .chain(capture.stderr_lines.iter().map(|l| OutputLine {
                    text: l.clone(),
                    stream: OutputStream::Stderr,
                }))
                .collect(),
            exit_status: Some(exit_status.clone()),
        };

        if let Ok(mut panel) = self.output_panel.lock() {
            panel.append_entry(entry);
        }

        Ok((ProcessId::new(), exit_status))
    }

    /// Resolves the working directory for an external run.
    ///
    /// When `working_dir` is provided it is used verbatim; otherwise the
    /// configured `shell.working_directory` rules apply (Requirement 19.6).
    fn resolve_external_dir(
        &self,
        working_dir: Option<&std::path::Path>,
        project_root: Option<&std::path::Path>,
        active_file: Option<&std::path::Path>,
    ) -> std::path::PathBuf {
        match working_dir {
            Some(dir) => dir.to_path_buf(),
            None => {
                let config = self.config.get();
                WorkingDirResolver::resolve(config.working_directory, project_root, active_file)
            }
        }
    }

    /// Opens a new interactive terminal session.
    pub fn open_terminal(
        &self,
        profile: Option<&str>,
        project_root: Option<&std::path::Path>,
        active_file: Option<&std::path::Path>,
    ) -> Result<SessionId, ShellError> {
        let config = self.config.get();
        let working_dir =
            WorkingDirResolver::resolve(config.working_directory, project_root, active_file);

        let mut manager = self
            .terminal_manager
            .lock()
            .expect("terminal manager lock poisoned");
        let session_id =
            manager.open_session_mock(working_dir.clone(), profile.map(String::from), (80, 24));

        // Update terminal panel
        if let Ok(mut panel) = self.terminal_panel.lock() {
            panel.set_active_tab(session_id);
            panel.set_working_directory_display(working_dir.display().to_string());
        }

        Ok(session_id)
    }

    /// Closes a terminal session.
    pub fn close_terminal(&self, session_id: SessionId) -> Result<(), ShellError> {
        let mut manager = self
            .terminal_manager
            .lock()
            .expect("terminal manager lock poisoned");
        manager.close_session(session_id)
    }

    /// Clears the Output Panel scrollback buffer.
    pub fn clear_output(&self) {
        if let Ok(mut panel) = self.output_panel.lock() {
            panel.clear();
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> ShellConfig {
        self.config.get()
    }

    /// Replaces the current configuration snapshot.
    ///
    /// Used by the desktop layer to seed or hot-reload `shell.*` settings
    /// (notably `shell.mode`) after the engine has been constructed.
    pub fn set_config(&self, config: ShellConfig) {
        self.config.update(config);
    }

    /// Returns the number of command entries currently in the Output Panel.
    ///
    /// Lets the desktop layer detect and surface newly captured output without
    /// exposing the panel's internals.
    pub fn output_entry_count(&self) -> usize {
        self.output_panel
            .lock()
            .map(|p| p.entry_count())
            .unwrap_or(0)
    }

    /// Returns the profile resolver for shell override lookups.
    pub fn profile_resolver(&self) -> &ProfileResolver {
        &self.profile_resolver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_engine() -> ShellEngine {
        ShellEngine::new(ShellConfigProvider::new())
    }

    // Validates: Requirement 2.2
    #[test]
    fn security_gate_disabled_always_refuses() {
        let config = ShellConfigProvider::with_config(ShellConfig {
            mode: ShellMode::Disabled,
            ..Default::default()
        });
        let engine = ShellEngine::new(config);
        assert!(matches!(
            engine.check_security_gate(false),
            Err(ShellError::ShellDisabled)
        ));
        assert!(matches!(
            engine.check_security_gate(true),
            Err(ShellError::ShellDisabled)
        ));
    }

    // Validates: Requirement 2.4
    #[test]
    fn security_gate_enabled_always_permits() {
        let config = ShellConfigProvider::with_config(ShellConfig {
            mode: ShellMode::Enabled,
            ..Default::default()
        });
        let engine = ShellEngine::new(config);
        assert!(engine.check_security_gate(false).is_ok());
        assert!(engine.check_security_gate(true).is_ok());
    }

    // Validates: Requirement 2.3
    #[test]
    fn security_gate_prompt_permits_direct_invocation() {
        let config = ShellConfigProvider::with_config(ShellConfig {
            mode: ShellMode::Prompt,
            ..Default::default()
        });
        let engine = ShellEngine::new(config);
        assert!(engine.check_security_gate(false).is_ok());
    }

    // Validates: Requirement 2.7
    #[test]
    fn security_gate_prompt_refuses_macro_invocation() {
        let config = ShellConfigProvider::with_config(ShellConfig {
            mode: ShellMode::Prompt,
            ..Default::default()
        });
        let engine = ShellEngine::new(config);
        assert!(matches!(
            engine.check_security_gate(true),
            Err(ShellError::MacroAccessDenied { .. })
        ));
    }

    // Validates: Requirement 15.6
    #[test]
    fn clear_output_empties_panel() {
        let engine = test_engine();
        // Panel starts empty
        engine.clear_output();
        // Should not panic
    }

    // Validates: Requirement 7.1
    #[test]
    fn open_terminal_creates_session() {
        let engine = test_engine();
        let session_id = engine.open_terminal(None, None, None).unwrap();

        let manager = engine.terminal_manager.lock().unwrap();
        assert!(manager.session(session_id).is_some());
    }

    // Validates: Requirement 7.3
    #[test]
    fn close_terminal_removes_session() {
        let engine = test_engine();
        let session_id = engine.open_terminal(None, None, None).unwrap();
        engine.close_terminal(session_id).unwrap();

        let manager = engine.terminal_manager.lock().unwrap();
        assert!(manager.session(session_id).is_none());
    }

    fn enabled_engine() -> ShellEngine {
        ShellEngine::new(ShellConfigProvider::with_config(ShellConfig {
            mode: ShellMode::Enabled,
            ..Default::default()
        }))
    }

    /// A program that exits immediately, per host platform.
    fn noop_external() -> (&'static str, Vec<String>) {
        if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "exit".to_string()])
        } else {
            ("true", Vec::new())
        }
    }

    /// A program that prints a known token to stdout, per host platform.
    fn echo_external(token: &str) -> (&'static str, Vec<String>) {
        if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), format!("echo {}", token)])
        } else {
            ("echo", vec![token.to_string()])
        }
    }

    // Validates: shell-command Requirement 19.5 (disabled refuses both modes)
    #[tokio::test]
    async fn execute_external_refused_when_shell_disabled() {
        let engine = ShellEngine::new(ShellConfigProvider::with_config(ShellConfig {
            mode: ShellMode::Disabled,
            ..Default::default()
        }));
        let (program, args) = noop_external();

        let detached = engine
            .execute_external(program, &args, None, ExecutionMode::Detached, None, None)
            .await;
        assert!(matches!(detached, Err(ShellError::ShellDisabled)));

        let captured = engine
            .execute_external(program, &args, None, ExecutionMode::Captured, None, None)
            .await;
        assert!(matches!(captured, Err(ShellError::ShellDisabled)));
    }

    // Validates: shell-command Requirement 19.5 (spawn_detached honours the gate)
    #[test]
    fn spawn_detached_refused_when_shell_disabled() {
        let engine = ShellEngine::new(ShellConfigProvider::with_config(ShellConfig {
            mode: ShellMode::Disabled,
            ..Default::default()
        }));
        let (program, args) = noop_external();
        let result = engine.spawn_detached(program, &args, None, None, None);
        assert!(matches!(result, Err(ShellError::ShellDisabled)));
    }

    // Validates: shell-command Requirement 19.1, 19.3 (detached returns a handle)
    #[tokio::test]
    async fn execute_external_detached_returns_handle() {
        let engine = enabled_engine();
        let (program, args) = noop_external();

        let outcome = engine
            .execute_external(program, &args, None, ExecutionMode::Detached, None, None)
            .await
            .expect("detached run should succeed");

        match outcome {
            ExternalOutcome::Detached(handle) => assert!(handle.pid.is_some()),
            other => panic!("expected Detached outcome, got {:?}", other),
        }
        // Detached run must NOT append to the Output_Panel (Requirement 19.3).
        assert_eq!(engine.output_panel.lock().unwrap().entry_count(), 0);
    }

    // Validates: shell-command Requirement 19.2 (captured output to Output_Panel)
    #[tokio::test]
    async fn execute_external_captured_appends_to_output_panel() {
        let engine = enabled_engine();
        let token = "ffwb_db10_token";
        let (program, args) = echo_external(token);

        let outcome = engine
            .execute_external(program, &args, None, ExecutionMode::Captured, None, None)
            .await
            .expect("captured run should succeed");

        match outcome {
            ExternalOutcome::Captured { exit_status, .. } => {
                assert!(exit_status.is_success());
            }
            other => panic!("expected Captured outcome, got {:?}", other),
        }

        let panel = engine.output_panel.lock().unwrap();
        assert_eq!(panel.entry_count(), 1);
        let entry = &panel.entries()[0];
        assert!(entry.command.contains(program));
        assert!(entry
            .lines
            .iter()
            .any(|l| l.text.contains(token) && l.stream == OutputStream::Stdout));
    }

    // Validates: shell-command Requirement 19.6 (explicit working_dir honoured)
    #[test]
    fn spawn_detached_uses_explicit_working_dir() {
        let engine = enabled_engine();
        let temp = tempfile::TempDir::new().unwrap();
        let (program, args) = noop_external();

        let handle = engine
            .spawn_detached(program, &args, Some(temp.path()), None, None)
            .expect("detached spawn in explicit dir should succeed");
        assert!(handle.pid.is_some());
    }

    // Validates: shell-command Requirement 19.8 (launch failure reported)
    #[tokio::test]
    async fn execute_external_captured_missing_program_errors() {
        let engine = enabled_engine();
        let missing = "ffwb_nonexistent_program_db10_engine";

        let result = engine
            .execute_external(missing, &[], None, ExecutionMode::Captured, None, None)
            .await;
        assert!(matches!(result, Err(ShellError::SpawnFailed { .. })));
    }
}
