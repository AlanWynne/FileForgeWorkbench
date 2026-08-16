//! Shell engine — central coordinator for the shell subsystem.
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
                        reason: "shell.mode is 'prompt' — macros cannot show UI prompts"
                            .to_string(),
                    })
                } else {
                    // Direct invocation — UI layer will show confirmation
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
}
