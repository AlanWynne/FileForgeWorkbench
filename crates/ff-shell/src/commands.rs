//! Command registration with the `ff-command` framework.
//!
//! Registers shell command IDs: `shell.execute`, `shell.terminal`,
//! `shell.capture`, and `shell.output.clear`.

use ff_command::CommandMetadata;

use crate::error::ShellError;

/// Shell command identifiers.
pub mod ids {
    /// Command ID for shell command execution mode.
    pub const SHELL_EXECUTE: &str = "shell.execute";
    /// Command ID for interactive terminal mode.
    pub const SHELL_TERMINAL: &str = "shell.terminal";
    /// Command ID for document capture mode.
    pub const SHELL_CAPTURE: &str = "shell.capture";
    /// Command ID for clearing the output panel.
    pub const SHELL_OUTPUT_CLEAR: &str = "shell.output.clear";
}

/// The input form classification for a SHELL command invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandForm {
    /// Interactive terminal mode: `SHELL` with no arguments, no target.
    Terminal,
    /// Command execution mode: `SHELL <command>` with no target.
    Execute {
        /// The command text to execute.
        command_text: String,
        /// Optional shell override (first argument if recognized as shell name).
        shell_override: Option<String>,
    },
    /// Document capture mode: `SHELL <command>` with A/B target.
    Capture {
        /// The command text to execute.
        command_text: String,
        /// Optional shell override.
        shell_override: Option<String>,
        /// The target line and position.
        target_line: usize,
        /// Whether the target is After (A) or Before (B).
        is_after: bool,
    },
}

/// Validates and classifies a SHELL command invocation.
///
/// Returns the appropriate `CommandForm` or an error for invalid forms.
///
/// # Invalid Forms (Requirement 9.4–9.6)
///
/// - SHELL with source line commands (C, CC, M, MM) → error
/// - SHELL with no args + A/B target → error (command required for capture)
/// - SHELL with multiple A/B targets → error (only one target permitted)
pub fn validate_command_form(
    has_args: bool,
    has_a_target: bool,
    has_b_target: bool,
    has_source_cmd: bool,
    command_text: Option<&str>,
    target_line: Option<usize>,
    shell_override: Option<&str>,
) -> Result<CommandForm, ShellError> {
    // Rule: source line commands are incompatible with SHELL (Req 9.4)
    if has_source_cmd {
        return Err(ShellError::InvalidCommandForm {
            reason: "source line commands (C, CC, M, MM) are incompatible with SHELL".to_string(),
        });
    }

    // Rule: multiple targets not allowed (Req 9.6)
    if has_a_target && has_b_target {
        return Err(ShellError::InvalidCommandForm {
            reason: "only one A or B target is permitted with SHELL".to_string(),
        });
    }

    // Rule: no args + target is invalid (Req 9.5)
    if !has_args && (has_a_target || has_b_target) {
        return Err(ShellError::InvalidCommandForm {
            reason: "a command argument is required for document capture mode".to_string(),
        });
    }

    let has_target = has_a_target || has_b_target;

    match (has_args, has_target) {
        (false, false) => {
            // Interactive terminal mode (Req 9.1)
            Ok(CommandForm::Terminal)
        }
        (true, false) => {
            // Command execution mode (Req 9.2)
            Ok(CommandForm::Execute {
                command_text: command_text.unwrap_or("").to_string(),
                shell_override: shell_override.map(String::from),
            })
        }
        (true, true) => {
            // Document capture mode (Req 9.3)
            Ok(CommandForm::Capture {
                command_text: command_text.unwrap_or("").to_string(),
                shell_override: shell_override.map(String::from),
                target_line: target_line.unwrap_or(0),
                is_after: has_a_target,
            })
        }
        (false, true) => {
            // Already handled above (no args + target → error)
            unreachable!()
        }
    }
}

/// Creates command metadata for the shell execute command.
pub fn shell_execute_metadata() -> CommandMetadata {
    CommandMetadata::builder(
        "Execute Shell Command",
        "Run an OS command or open a terminal session",
    )
    .category("shell")
    .build()
}

/// Creates command metadata for the shell terminal command.
pub fn shell_terminal_metadata() -> CommandMetadata {
    CommandMetadata::builder(
        "Open Terminal",
        "Open an interactive terminal session in a new tab",
    )
    .category("shell")
    .build()
}

/// Creates command metadata for the shell capture command.
pub fn shell_capture_metadata() -> CommandMetadata {
    CommandMetadata::builder(
        "Capture Shell Output",
        "Run a command and insert stdout into the document",
    )
    .category("shell")
    .build()
}

/// Creates command metadata for the output clear command.
pub fn shell_output_clear_metadata() -> CommandMetadata {
    CommandMetadata::builder(
        "Clear Output",
        "Clear the shell output panel scrollback buffer",
    )
    .category("shell")
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 9.1
    #[test]
    fn no_args_no_target_is_terminal_mode() {
        let form = validate_command_form(false, false, false, false, None, None, None);
        assert_eq!(form.unwrap(), CommandForm::Terminal);
    }

    // Validates: Requirement 9.2
    #[test]
    fn args_no_target_is_execute_mode() {
        let form = validate_command_form(true, false, false, false, Some("ls -la"), None, None);
        assert!(matches!(form.unwrap(), CommandForm::Execute { .. }));
    }

    // Validates: Requirement 9.3
    #[test]
    fn args_with_a_target_is_capture_mode() {
        let form = validate_command_form(true, true, false, false, Some("date"), Some(5), None);
        let form = form.unwrap();
        assert!(matches!(form, CommandForm::Capture { is_after: true, .. }));
    }

    // Validates: Requirement 9.3
    #[test]
    fn args_with_b_target_is_capture_mode() {
        let form = validate_command_form(true, false, true, false, Some("date"), Some(5), None);
        let form = form.unwrap();
        assert!(matches!(
            form,
            CommandForm::Capture {
                is_after: false,
                ..
            }
        ));
    }

    // Validates: Requirement 9.4
    #[test]
    fn source_commands_are_rejected() {
        let form = validate_command_form(true, false, false, true, Some("ls"), None, None);
        assert!(matches!(form, Err(ShellError::InvalidCommandForm { .. })));
    }

    // Validates: Requirement 9.5
    #[test]
    fn no_args_with_target_is_rejected() {
        let form = validate_command_form(false, true, false, false, None, Some(3), None);
        assert!(matches!(form, Err(ShellError::InvalidCommandForm { .. })));
    }

    // Validates: Requirement 9.6
    #[test]
    fn multiple_targets_are_rejected() {
        let form = validate_command_form(true, true, true, false, Some("ls"), Some(3), None);
        assert!(matches!(form, Err(ShellError::InvalidCommandForm { .. })));
    }

    // Validates: Requirement 1.5
    #[test]
    fn shell_execute_metadata_has_correct_fields() {
        let meta = shell_execute_metadata();
        assert_eq!(meta.display_name, "Execute Shell Command");
        assert_eq!(meta.category, "shell");
    }

    // Validates: Requirement 9 — completeness: all valid boolean combos classified
    #[test]
    fn all_valid_input_combinations_are_classified() {
        // No args, no target, no source → Terminal
        assert!(validate_command_form(false, false, false, false, None, None, None).is_ok());
        // Args, no target, no source → Execute
        assert!(validate_command_form(true, false, false, false, Some("x"), None, None).is_ok());
        // Args, A target, no source → Capture
        assert!(validate_command_form(true, true, false, false, Some("x"), Some(0), None).is_ok());
        // Args, B target, no source → Capture
        assert!(validate_command_form(true, false, true, false, Some("x"), Some(0), None).is_ok());
    }

    // Validates: Requirement 9 — completeness: all invalid combos rejected
    #[test]
    fn all_invalid_input_combinations_are_rejected() {
        // Source cmd always rejected
        assert!(validate_command_form(true, false, false, true, Some("x"), None, None).is_err());
        assert!(validate_command_form(false, false, false, true, None, None, None).is_err());
        // No args + target
        assert!(validate_command_form(false, true, false, false, None, Some(0), None).is_err());
        assert!(validate_command_form(false, false, true, false, None, Some(0), None).is_err());
        // Both targets
        assert!(validate_command_form(true, true, true, false, Some("x"), Some(0), None).is_err());
    }
}
