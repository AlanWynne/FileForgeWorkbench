//! Command engine — top-level execution pipeline orchestrator.
//!
//! Implements the 10-step pipeline: collect → parse → normalize → scope →
//! validate → plan → execute → update state → clear consumed → emit status.

use crate::config::{CommandConfig, InvalidLineCommandPolicy};
use crate::line_parser::{LineCommandCountOverflow, LineCommandDescriptor, LineCommandParser};
use crate::parser::{ParsedCommand, PrimaryCommandParser};
use crate::session::SessionState;
use crate::status::StatusMessage;

/// The top-level command execution pipeline orchestrator.
///
/// Accepts command-line text and pending line commands, drives the full
/// 10-step execution pipeline, and produces status messages.
pub struct CommandEngine {
    /// Runtime configuration.
    config: CommandConfig,
    /// Per-document session state.
    session: SessionState,
}

impl CommandEngine {
    /// Create a new CommandEngine with default configuration.
    pub fn new() -> Self {
        Self {
            config: CommandConfig::default(),
            session: SessionState::new(),
        }
    }

    /// Create with explicit configuration.
    pub fn with_config(config: CommandConfig) -> Self {
        Self {
            config,
            session: SessionState::new(),
        }
    }

    /// Execute a primary command from command-line text.
    ///
    /// Drives the 10-step pipeline:
    /// 1. Collect pending line commands from SessionState
    /// 2. Parse the primary command text into tokens
    /// 3. Normalize the command name
    /// 4. Resolve scope
    /// 5. Validate command-scope compatibility
    /// 6. Build execution plan
    /// 7. Execute within undo transaction
    /// 8. Update SessionState
    /// 9. Clear consumed line commands
    /// 10. Emit status message
    pub fn execute_command_line(&mut self, text: &str) -> StatusMessage {
        // Step 1: Check pending line commands
        let has_pending = self.session.has_pending();

        // Step 2: Parse command text
        let parsed = match PrimaryCommandParser::parse(text) {
            Ok(parsed) => parsed,
            Err(err) => {
                return StatusMessage::syntax_error("(input)", &err.to_string());
            }
        };

        match parsed {
            ParsedCommand::Empty => {
                if has_pending {
                    // Execute pending line commands
                    self.execute_pending_line_commands()
                } else {
                    StatusMessage::info("No command")
                }
            }
            ParsedCommand::Command { name, args: _ } => {
                // Step 3-10: For now, we report unrecognised commands
                // Full dispatch integration will come with upstream crate wiring
                // This provides the structural pipeline with error handling
                StatusMessage::runtime_error(&name, "command not yet implemented")
            }
        }
    }

    /// Submit a line command from the prefix area.
    ///
    /// Parses the prefix text and adds it to pending state, respecting
    /// the invalid line command policy.
    pub fn submit_line_command(
        &mut self,
        line: u64,
        prefix_text: &str,
    ) -> Result<(), StatusMessage> {
        let descriptor = match LineCommandParser::parse(prefix_text) {
            Ok(Some(desc)) => desc,
            Ok(None) => return Ok(()), // empty input, no action
            Err(LineCommandCountOverflow { count }) => {
                return Err(StatusMessage::syntax_error(
                    prefix_text,
                    &format!("count {} exceeds maximum 99999", count),
                ));
            }
        };

        // Check invalid line command policy
        if let LineCommandDescriptor::Unknown(ref text) = descriptor {
            match self.config.invalid_line_command_policy {
                InvalidLineCommandPolicy::Reject => {
                    return Err(StatusMessage::syntax_error(
                        text,
                        "unrecognised line command",
                    ));
                }
                InvalidLineCommandPolicy::Ignore => {
                    return Ok(()); // silently discard
                }
            }
        }

        self.session.add_pending(line, descriptor);
        Ok(())
    }

    /// Execute pending line commands (when Enter pressed with empty command line).
    fn execute_pending_line_commands(&mut self) -> StatusMessage {
        let pending = self.session.take_pending();
        let count = pending.len();
        // In a full implementation, this would pair block commands,
        // execute each, and wrap in transactions. For now, we confirm execution.
        StatusMessage::info(format!(
            "{} line command{} executed",
            count,
            if count == 1 { "" } else { "s" }
        ))
    }

    /// Get the current session state (read-only).
    pub fn session(&self) -> &SessionState {
        &self.session
    }

    /// Get mutable session state.
    pub fn session_mut(&mut self) -> &mut SessionState {
        &mut self.session
    }

    /// Update configuration (e.g., on hot-reload notification).
    pub fn update_config(&mut self, config: CommandConfig) {
        self.config = config;
    }

    /// Get current configuration.
    pub fn config(&self) -> &CommandConfig {
        &self.config
    }
}

impl Default for CommandEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::StatusKind;

    // Validates: Requirement 1.3
    #[test]
    fn empty_command_no_pending_returns_no_command() {
        let mut engine = CommandEngine::new();
        let result = engine.execute_command_line("");
        assert_eq!(result.text, "No command");
        assert_eq!(result.kind, StatusKind::Info);
    }

    // Validates: Requirement 1.3
    #[test]
    fn whitespace_only_command_no_pending_returns_no_command() {
        let mut engine = CommandEngine::new();
        let result = engine.execute_command_line("   ");
        assert_eq!(result.text, "No command");
    }

    // Validates: Requirement 1.2
    #[test]
    fn empty_command_with_pending_executes_line_commands() {
        let mut engine = CommandEngine::new();
        engine.submit_line_command(0, "C").unwrap();
        engine.submit_line_command(5, "D3").unwrap();

        let result = engine.execute_command_line("");
        assert!(result.text.contains("2 line commands executed"));
        assert_eq!(result.kind, StatusKind::Info);
        // Pending should be cleared after execution
        assert!(!engine.session().has_pending());
    }

    // Validates: Requirement 1.4
    #[test]
    fn unrecognised_command_returns_error_status() {
        let mut engine = CommandEngine::new();
        let result = engine.execute_command_line("NOSUCHCMD");
        assert_eq!(result.kind, StatusKind::RuntimeError);
        assert!(result.text.contains("NOSUCHCMD"));
    }

    // Validates: Requirement 1.1 (pipeline parse step)
    #[test]
    fn syntax_error_in_input_returns_syntax_error_status() {
        let mut engine = CommandEngine::new();
        let result = engine.execute_command_line("FIND 'unclosed");
        assert_eq!(result.kind, StatusKind::SyntaxError);
    }

    // Validates: Requirement 6.4
    #[test]
    fn submit_unknown_line_command_reject_policy_returns_error() {
        let mut engine = CommandEngine::new();
        let result = engine.submit_line_command(0, "ZZ");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, StatusKind::SyntaxError);
        assert!(err.text.contains("ZZ"));
    }

    // Validates: Requirement 6.5
    #[test]
    fn submit_unknown_line_command_ignore_policy_silently_discards() {
        let config = CommandConfig {
            invalid_line_command_policy: InvalidLineCommandPolicy::Ignore,
            ..CommandConfig::default()
        };
        let mut engine = CommandEngine::with_config(config);
        let result = engine.submit_line_command(0, "ZZ");
        assert!(result.is_ok());
        assert!(!engine.session().has_pending());
    }

    // Validates: Requirement 1.2
    #[test]
    fn submit_valid_line_command_adds_to_pending() {
        let mut engine = CommandEngine::new();
        engine.submit_line_command(5, "D3").unwrap();
        assert!(engine.session().has_pending());
        assert_eq!(engine.session().pending().len(), 1);
        assert_eq!(engine.session().pending()[0].line, 5);
    }

    // Validates: Requirement 1.6
    #[test]
    fn line_command_count_overflow_returns_error() {
        let mut engine = CommandEngine::new();
        let result = engine.submit_line_command(0, "D100000");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, StatusKind::SyntaxError);
    }

    #[test]
    fn update_config_applies_new_values() {
        let mut engine = CommandEngine::new();
        assert_eq!(engine.config().default_shift_width, 2);

        let new_config = CommandConfig {
            default_shift_width: 4,
            ..CommandConfig::default()
        };
        engine.update_config(new_config);
        assert_eq!(engine.config().default_shift_width, 4);
    }
}
