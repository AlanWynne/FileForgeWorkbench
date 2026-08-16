//! WRAP primary command handler.
//!
//! Parses command arguments and routes to the appropriate wrap operation.
//! The WRAP command is non-undoable and not recorded in command history.

use crate::error::WrapError;
use crate::mode::WrapMode;
use crate::state::WrapState;

/// All possible wrap operations that can be dispatched via the WRAP command.
///
/// Addresses: Requirement 3 (WRAP Primary Command)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrapOperation {
    /// Enable wrap using Word mode (the default enabled mode).
    /// `WRAP ON`
    On,

    /// Disable wrap (set mode to None).
    /// `WRAP OFF`
    Off,

    /// Toggle: if None → Word; if Word/Character → None.
    /// `WRAP TOGGLE` or `WRAP` with no arguments.
    Toggle,

    /// Set mode to Word explicitly.
    /// `WRAP WORD`
    SetWord,

    /// Set mode to Character explicitly.
    /// `WRAP CHAR`
    SetCharacter,

    /// Set wrap column boundary.
    /// `WRAP COL n` (n=0 means viewport, n>0 means fixed column).
    SetColumn(u16),
}

/// Command ID for the WRAP primary command.
pub const WRAP_COMMAND_ID: &str = "view.wrap";

/// Parse WRAP command arguments into a `WrapOperation`.
///
/// Supported forms:
/// - `""` (empty) → Toggle
/// - `"ON"` → On
/// - `"OFF"` → Off
/// - `"TOGGLE"` → Toggle
/// - `"WORD"` → SetWord
/// - `"CHAR"` → SetCharacter
/// - `"COL n"` → SetColumn(n)
///
/// Addresses: Requirement 3 AC 14
pub fn parse_wrap_args(args: &str) -> Result<WrapOperation, WrapError> {
    let trimmed = args.trim();

    if trimmed.is_empty() {
        return Ok(WrapOperation::Toggle);
    }

    let upper = trimmed.to_uppercase();
    let parts: Vec<&str> = upper.split_whitespace().collect();

    match parts[0] {
        "ON" => Ok(WrapOperation::On),
        "OFF" => Ok(WrapOperation::Off),
        "TOGGLE" => Ok(WrapOperation::Toggle),
        "WORD" => Ok(WrapOperation::SetWord),
        "CHAR" => Ok(WrapOperation::SetCharacter),
        "COL" => {
            if parts.len() < 2 {
                return Err(WrapError::InvalidColumn {
                    value: String::new(),
                });
            }
            match parts[1].parse::<u16>() {
                Ok(n) if n <= 10_000 => Ok(WrapOperation::SetColumn(n)),
                Ok(n) => Err(WrapError::InvalidColumn {
                    value: n.to_string(),
                }),
                Err(_) => Err(WrapError::InvalidColumn {
                    value: parts[1].to_string(),
                }),
            }
        }
        _ => Err(WrapError::InvalidSubCommand {
            arg: trimmed.to_string(),
        }),
    }
}

/// Result of executing a wrap command operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapCommandResult {
    /// Status message to display to the user.
    pub message: String,
    /// Whether the state actually changed.
    pub state_changed: bool,
    /// The new wrap mode after the operation.
    pub new_mode: WrapMode,
}

/// Execute a wrap operation on the given state.
///
/// Returns a result with the status message and whether state changed.
///
/// Addresses: Requirement 3 AC 2–10
pub fn execute_wrap_operation(
    operation: &WrapOperation,
    state: &mut WrapState,
) -> WrapCommandResult {
    match operation {
        WrapOperation::On => {
            if state.is_active() {
                // Already active — return confirmation
                WrapCommandResult {
                    message: format!("Wrap: {} (already active)", state.mode().display_label()),
                    state_changed: false,
                    new_mode: state.mode(),
                }
            } else {
                state.set_mode(WrapMode::DEFAULT_ENABLED);
                WrapCommandResult {
                    message: format_status_message(state.mode()),
                    state_changed: true,
                    new_mode: state.mode(),
                }
            }
        }
        WrapOperation::Off => {
            if !state.is_active() {
                WrapCommandResult {
                    message: "Wrap is already off".to_string(),
                    state_changed: false,
                    new_mode: WrapMode::None,
                }
            } else {
                state.set_mode(WrapMode::None);
                WrapCommandResult {
                    message: format_status_message(state.mode()),
                    state_changed: true,
                    new_mode: WrapMode::None,
                }
            }
        }
        WrapOperation::Toggle => {
            if state.is_active() {
                state.set_mode(WrapMode::None);
            } else {
                state.set_mode(WrapMode::DEFAULT_ENABLED);
            }
            WrapCommandResult {
                message: format_status_message(state.mode()),
                state_changed: true,
                new_mode: state.mode(),
            }
        }
        WrapOperation::SetWord => {
            let changed = state.mode() != WrapMode::Word;
            state.set_mode(WrapMode::Word);
            WrapCommandResult {
                message: format_status_message(state.mode()),
                state_changed: changed,
                new_mode: WrapMode::Word,
            }
        }
        WrapOperation::SetCharacter => {
            let changed = state.mode() != WrapMode::Character;
            state.set_mode(WrapMode::Character);
            WrapCommandResult {
                message: format_status_message(state.mode()),
                state_changed: changed,
                new_mode: WrapMode::Character,
            }
        }
        WrapOperation::SetColumn(n) => {
            use crate::boundary::{WrapBoundary, WrapColumn};
            let boundary = if *n == 0 {
                WrapBoundary::Viewport
            } else {
                match WrapColumn::new(*n) {
                    Some(col) => WrapBoundary::Column(col),
                    Option::None => WrapBoundary::Viewport,
                }
            };
            state.set_boundary(boundary);
            let msg = if *n == 0 {
                "Wrap column: viewport width".to_string()
            } else {
                format!("Wrap column: {}", n)
            };
            WrapCommandResult {
                message: msg,
                state_changed: true,
                new_mode: state.mode(),
            }
        }
    }
}

/// Format the status message for the current wrap state.
///
/// Addresses: Requirement 3 AC 8
pub fn format_status_message(mode: WrapMode) -> String {
    match mode {
        WrapMode::None => "Wrap: Off".to_string(),
        WrapMode::Word => "Wrap: Word".to_string(),
        WrapMode::Character => "Wrap: Character".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WrapConfig;

    fn default_state() -> WrapState {
        WrapState::from_config(&WrapConfig::default())
    }

    // --- Argument parsing ---

    #[test]
    fn parse_empty_is_toggle() {
        // Validates: Requirement 3.5
        assert_eq!(parse_wrap_args("").unwrap(), WrapOperation::Toggle);
    }

    #[test]
    fn parse_on() {
        assert_eq!(parse_wrap_args("ON").unwrap(), WrapOperation::On);
    }

    #[test]
    fn parse_off() {
        assert_eq!(parse_wrap_args("OFF").unwrap(), WrapOperation::Off);
    }

    #[test]
    fn parse_toggle() {
        assert_eq!(parse_wrap_args("TOGGLE").unwrap(), WrapOperation::Toggle);
    }

    #[test]
    fn parse_word() {
        assert_eq!(parse_wrap_args("WORD").unwrap(), WrapOperation::SetWord);
    }

    #[test]
    fn parse_char() {
        assert_eq!(
            parse_wrap_args("CHAR").unwrap(),
            WrapOperation::SetCharacter
        );
    }

    #[test]
    fn parse_col_valid() {
        // Validates: Requirement 4.6
        assert_eq!(
            parse_wrap_args("COL 80").unwrap(),
            WrapOperation::SetColumn(80)
        );
    }

    #[test]
    fn parse_col_zero() {
        assert_eq!(
            parse_wrap_args("COL 0").unwrap(),
            WrapOperation::SetColumn(0)
        );
    }

    #[test]
    fn parse_col_invalid_not_a_number() {
        // Validates: Requirement 3.14
        assert!(parse_wrap_args("COL abc").is_err());
    }

    #[test]
    fn parse_invalid_sub_command() {
        // Validates: Requirement 3.14
        let err = parse_wrap_args("BANANA").unwrap_err();
        assert!(matches!(err, WrapError::InvalidSubCommand { .. }));
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(parse_wrap_args("on").unwrap(), WrapOperation::On);
        assert_eq!(parse_wrap_args("Off").unwrap(), WrapOperation::Off);
        assert_eq!(parse_wrap_args("word").unwrap(), WrapOperation::SetWord);
    }

    // --- Execution ---

    #[test]
    fn execute_on_from_none_sets_word() {
        // Validates: Requirement 3.2
        let mut state = default_state();
        let result = execute_wrap_operation(&WrapOperation::On, &mut state);
        assert_eq!(state.mode(), WrapMode::Word);
        assert!(result.state_changed);
        assert_eq!(result.message, "Wrap: Word");
    }

    #[test]
    fn execute_on_when_already_active_is_idempotent() {
        // Validates: Requirement 3.9
        let mut state = default_state();
        state.set_mode(WrapMode::Word);
        let result = execute_wrap_operation(&WrapOperation::On, &mut state);
        assert!(!result.state_changed);
        assert!(result.message.contains("already active"));
    }

    #[test]
    fn execute_off_from_active_sets_none() {
        // Validates: Requirement 3.3
        let mut state = default_state();
        state.set_mode(WrapMode::Word);
        let result = execute_wrap_operation(&WrapOperation::Off, &mut state);
        assert_eq!(state.mode(), WrapMode::None);
        assert!(result.state_changed);
    }

    #[test]
    fn execute_off_when_already_none_is_idempotent() {
        // Validates: Requirement 3.10
        let mut state = default_state();
        let result = execute_wrap_operation(&WrapOperation::Off, &mut state);
        assert!(!result.state_changed);
        assert_eq!(result.message, "Wrap is already off");
    }

    #[test]
    fn execute_toggle_from_none_enables_word() {
        // Validates: Requirement 3.4
        let mut state = default_state();
        let result = execute_wrap_operation(&WrapOperation::Toggle, &mut state);
        assert_eq!(state.mode(), WrapMode::Word);
        assert!(result.state_changed);
    }

    #[test]
    fn execute_toggle_from_word_disables() {
        // Validates: Requirement 3.4
        let mut state = default_state();
        state.set_mode(WrapMode::Word);
        let result = execute_wrap_operation(&WrapOperation::Toggle, &mut state);
        assert_eq!(state.mode(), WrapMode::None);
        assert!(result.state_changed);
    }

    #[test]
    fn execute_toggle_from_character_disables() {
        // Validates: Requirement 3.4
        let mut state = default_state();
        state.set_mode(WrapMode::Character);
        let result = execute_wrap_operation(&WrapOperation::Toggle, &mut state);
        assert_eq!(state.mode(), WrapMode::None);
        assert!(result.state_changed);
    }

    #[test]
    fn execute_set_word() {
        // Validates: Requirement 3.6
        let mut state = default_state();
        let result = execute_wrap_operation(&WrapOperation::SetWord, &mut state);
        assert_eq!(state.mode(), WrapMode::Word);
        assert!(result.state_changed);
    }

    #[test]
    fn execute_set_character() {
        // Validates: Requirement 3.7
        let mut state = default_state();
        let result = execute_wrap_operation(&WrapOperation::SetCharacter, &mut state);
        assert_eq!(state.mode(), WrapMode::Character);
        assert!(result.state_changed);
    }

    #[test]
    fn execute_set_column() {
        // Validates: Requirement 4.6
        let mut state = default_state();
        let result = execute_wrap_operation(&WrapOperation::SetColumn(80), &mut state);
        assert!(result.state_changed);
        assert_eq!(state.effective_wrap_width(120), 80);
    }

    #[test]
    fn status_message_format() {
        // Validates: Requirement 3.8
        assert_eq!(format_status_message(WrapMode::None), "Wrap: Off");
        assert_eq!(format_status_message(WrapMode::Word), "Wrap: Word");
        assert_eq!(
            format_status_message(WrapMode::Character),
            "Wrap: Character"
        );
    }
}
