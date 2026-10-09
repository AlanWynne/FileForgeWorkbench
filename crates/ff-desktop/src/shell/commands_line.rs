//! # Command-Line Field Lifecycle
//!
//! The `run_command_line` / `begin_command_line` / `finish_command_line` method
//! trio extracted VERBATIM from `commands.rs` as part of the
//! behaviour-preserving file-size split (pure code movement -- no logic edits).
//! Together they own the disposition of the `Command ===>` field across a
//! command-line invocation.

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Run a command submitted from the `Command ===>` field (Enter or a
    /// key-forwarded F-key), then apply its Command_Line_Outcome to the field.
    ///
    /// This is the SINGLE decision point for what the command line holds after
    /// an invocation, shared by both input paths (command-framework Req 13.1,
    /// 9.9 revised, 9.10). The field is left INTACT during dispatch (so commands
    /// that read it -- notably RETRIEVE, Req 19.1 -- still work), then set to the
    /// effective outcome:
    /// - an explicit outcome a command stashed in `pending_command_line_outcome`
    ///   (RETRIEVE -> `Set(recalled)` / `Leave`); else
    /// - the DEFAULT: `Clear` on success, `Restore` (the executed text) when the
    ///   command left an `open_error` -- which covers BOTH an unresolved command
    ///   (typo kept for correction, Req 13.2) and a resolved-but-failed command
    ///   (e.g. FIND-not-found comes back, Req 13.3).
    ///
    /// Validates: command-framework Requirement 13.1, 13.2, 13.3, 9.9, 9.10
    pub(super) fn run_command_line(&mut self, cmd: &str) {
        let original = cmd.trim().to_string();
        self.begin_command_line();
        self.dispatch_command_string(&original);
        self.finish_command_line(&original);
    }

    /// Reset the per-dispatch Command_Line_Outcome state before a command-line
    /// invocation: no stashed outcome, no stale error. Shared by the Enter path
    /// (`run_command_line`) and the key-forward path (`dispatch_key_command`) so
    /// the outcome is computed from THIS invocation only.
    ///
    /// Validates: command-framework Requirement 13.1
    pub(super) fn begin_command_line(&mut self) {
        self.pending_command_line_outcome = None;
        self.open_error = None;
    }

    /// Apply the effective Command_Line_Outcome to the `Command ===>` field after
    /// a command-line invocation dispatched with the field left intact. An
    /// explicit stash wins; else the default is `Clear` on success and `Restore`
    /// (the executed `original`) when the command left an `open_error` -- which
    /// covers BOTH an unresolved typo (kept for correction, Req 13.2) and a
    /// resolved-but-failed command (e.g. FIND-not-found comes back, Req 13.3).
    ///
    /// Validates: command-framework Requirement 13.1, 13.2, 13.3
    pub(super) fn finish_command_line(&mut self, original: &str) {
        use crate::shell::command_line_outcome::CommandLineOutcome;
        let effective = self.pending_command_line_outcome.take().unwrap_or_else(|| {
            if self.open_error.is_some() {
                CommandLineOutcome::Restore
            } else {
                CommandLineOutcome::Clear
            }
        });
        match effective {
            CommandLineOutcome::Clear => self.command_text.clear(),
            CommandLineOutcome::Restore => self.command_text = original.to_string(),
            CommandLineOutcome::Set(text) => self.command_text = text,
            // Keep whatever the command itself left in the field.
            CommandLineOutcome::Leave => {}
        }
    }
}
