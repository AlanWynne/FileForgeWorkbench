//! External Command_Target execution adapter.
//!
//! Routes a [`ff_command::CommandTarget::External`] through the `ff-shell`
//! engine: Detached spawns fire-and-forget; Captured runs asynchronously with
//! output routed to the shell's Output_Panel. Placeholders `${workspace_root}`
//! and `${file_dir}` are expanded before execution, and the `shell.mode` gate
//! (disabled / prompt / enabled) is applied -- `prompt` stages the run behind a
//! confirmation dialog.
//!
//! Validates: command-configurator Requirement 3.2-3.9; shell-command
//! Requirement 19.

use ff_command::ExternalMode;
use ff_shell::{ExecutionMode, ExternalOutcome, ShellError, ShellMode};

use super::WorkbenchShell;

/// An External target that has been placeholder-expanded and is awaiting a
/// `shell.mode = prompt` confirmation before it is spawned.
///
/// Validates: command-configurator Requirement 3.8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingExternal {
    /// The program to launch (placeholders already expanded).
    pub program: String,
    /// Argument list (placeholders already expanded).
    pub args: Vec<String>,
    /// Optional working directory (placeholders already expanded).
    pub working_dir: Option<String>,
    /// Detached (fire-and-forget) or Captured (output to Output_Panel).
    pub mode: ExternalMode,
}

impl WorkbenchShell {
    /// Expand `${workspace_root}` and `${file_dir}` in a single string.
    ///
    /// An unresolved placeholder expands to an empty string and logs a DEBUG
    /// record (Requirement 3.6).
    pub(super) fn expand_external_placeholders(&self, input: &str) -> String {
        let workspace_root = self
            .active_workspace
            .as_ref()
            .and_then(|ws| ws.roots.first())
            .map(|p| p.to_string_lossy().into_owned());
        let file_dir = self
            .tabs
            .active_tab()
            .path
            .as_deref()
            .and_then(|p| std::path::Path::new(p).parent())
            .map(|p| p.to_string_lossy().into_owned());

        let mut out = input.to_string();
        for (token, value) in [
            ("${workspace_root}", workspace_root),
            ("${file_dir}", file_dir),
        ] {
            if out.contains(token) {
                let replacement = value.unwrap_or_else(|| {
                    ff_logging::log(
                        ff_logging::LogLevel::Debug,
                        "ff_desktop::external",
                        &format!("placeholder {token} is unresolved; expanding to empty string"),
                    );
                    String::new()
                });
                out = out.replace(token, &replacement);
            }
        }
        out
    }

    /// Run an External Command_Target, applying placeholder expansion and the
    /// `shell.mode` gate.
    ///
    /// - `disabled`: refuse with the standard message (Requirement 3.7).
    /// - `prompt`: stage a [`PendingExternal`] for the confirmation dialog
    ///   (Requirement 3.8); the caller renders the dialog.
    /// - `enabled`: execute immediately (Requirement 3.2, 3.4).
    ///
    /// Validates: command-configurator Requirement 3.2-3.7.
    pub(super) fn run_external_target(
        &mut self,
        program: &str,
        args: &[String],
        working_dir: Option<&str>,
        mode: ExternalMode,
    ) {
        // Expand placeholders in program, args, and working_dir (Requirement 3.6).
        let program = self.expand_external_placeholders(program);
        let args: Vec<String> = args
            .iter()
            .map(|a| self.expand_external_placeholders(a))
            .collect();
        let working_dir = working_dir.map(|w| self.expand_external_placeholders(w));

        let pending = PendingExternal {
            program,
            args,
            working_dir,
            mode,
        };

        match self.shell_engine.config().mode {
            ShellMode::Disabled => {
                // Req 3.7: refuse with the standard shell-disabled message.
                self.open_error = Some(ShellError::ShellDisabled.to_string());
            }
            ShellMode::Prompt => {
                // Req 3.8: confirm before spawning; the dialog is rendered in update().
                self.pending_external = Some(pending);
            }
            ShellMode::Enabled => {
                self.execute_external_now(&pending);
            }
        }
    }

    /// Execute an already-confirmed / already-expanded External target.
    ///
    /// Detached spawns fire-and-forget; Captured runs on the shell runtime and
    /// its output lands in the Output_Panel. A launch failure is surfaced as a
    /// status message (Requirement 3.9).
    ///
    /// Validates: command-configurator Requirement 3.2, 3.4, 3.5, 3.9.
    pub(super) fn execute_external_now(&mut self, pending: &PendingExternal) {
        let working_dir = pending.working_dir.as_deref().map(std::path::Path::new);
        let project_root = self
            .active_workspace
            .as_ref()
            .and_then(|ws| ws.roots.first())
            .map(|p| p.as_path());
        let active_file = self
            .tabs
            .active_tab()
            .path
            .as_deref()
            .map(std::path::Path::new);

        match pending.mode {
            ExternalMode::Detached => {
                match self.shell_engine.spawn_detached(
                    &pending.program,
                    &pending.args,
                    working_dir,
                    project_root,
                    active_file,
                ) {
                    Ok(_handle) => {
                        self.open_error =
                            Some(format!("Started '{}' (detached).", pending.program));
                    }
                    Err(e) => {
                        // Req 3.9: launch failure reported; no Workspace opened.
                        self.open_error = Some(e.to_string());
                    }
                }
            }
            ExternalMode::Captured => {
                // Captured runs async; block on the shell runtime (never the
                // GUI's per-frame path except for this explicit user action).
                let result = self.runtime.block_on(self.shell_engine.execute_external(
                    &pending.program,
                    &pending.args,
                    working_dir,
                    ExecutionMode::Captured,
                    project_root,
                    active_file,
                ));
                match result {
                    Ok(ExternalOutcome::Captured { exit_status, .. }) => {
                        let code = exit_status
                            .code
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "signal".to_string());
                        self.open_error = Some(format!(
                            "'{}' finished (exit {}). See the Output panel.",
                            pending.program, code
                        ));
                    }
                    Ok(ExternalOutcome::Detached(_)) => {
                        // Not reachable for Captured mode, but handle exhaustively.
                    }
                    Err(e) => {
                        self.open_error = Some(e.to_string());
                    }
                }
            }
        }
    }
}
