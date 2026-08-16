//! Status bar wrap indicator model.
//!
//! Provides formatted wrap data for the status bar display.

use crate::mode::WrapMode;
use crate::state::WrapState;

/// Format the wrap mode for status bar display.
///
/// Returns `None` when mode is `None` (indicator hidden per Req 8 AC 3).
///
/// Returns:
/// - `Some("Wrap: Word")` when mode is Word
/// - `Some("Wrap: Char")` when mode is Character
/// - `None` when mode is None
///
/// Addresses: Requirement 8 AC 1, AC 2, AC 3
pub fn format_indicator(state: &WrapState) -> Option<String> {
    match state.mode() {
        WrapMode::None => Option::None,
        WrapMode::Word => Some("Wrap: Word".to_string()),
        WrapMode::Character => Some("Wrap: Char".to_string()),
    }
}

/// Compute the next wrap mode in the cycle for status bar click.
///
/// Cycle: None → Word → Character → None
///
/// Addresses: Requirement 8 AC 5
pub fn next_mode_in_cycle(current: WrapMode) -> WrapMode {
    match current {
        WrapMode::None => WrapMode::Word,
        WrapMode::Word => WrapMode::Character,
        WrapMode::Character => WrapMode::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WrapConfig;

    #[test]
    fn indicator_hidden_when_none() {
        // Validates: Requirement 8.3
        let state = WrapState::from_config(&WrapConfig::default());
        assert_eq!(format_indicator(&state), Option::None);
    }

    #[test]
    fn indicator_shows_wrap_word() {
        // Validates: Requirement 8.1
        let config = WrapConfig {
            default_mode: WrapMode::Word,
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        assert_eq!(format_indicator(&state), Some("Wrap: Word".to_string()));
    }

    #[test]
    fn indicator_shows_wrap_char() {
        // Validates: Requirement 8.2
        let config = WrapConfig {
            default_mode: WrapMode::Character,
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        assert_eq!(format_indicator(&state), Some("Wrap: Char".to_string()));
    }

    #[test]
    fn cycle_none_to_word() {
        // Validates: Requirement 8.5
        assert_eq!(next_mode_in_cycle(WrapMode::None), WrapMode::Word);
    }

    #[test]
    fn cycle_word_to_character() {
        // Validates: Requirement 8.5
        assert_eq!(next_mode_in_cycle(WrapMode::Word), WrapMode::Character);
    }

    #[test]
    fn cycle_character_to_none() {
        // Validates: Requirement 8.5
        assert_eq!(next_mode_in_cycle(WrapMode::Character), WrapMode::None);
    }
}
