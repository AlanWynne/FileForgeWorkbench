//! Hex editing engine.
//!
//! Manages byte-level editing in hex mode. Validates input, computes
//! new byte values, and produces edit actions for the undo/redo system.

use crate::cursor::HexCursor;
use crate::error::HexError;
use crate::types::{HexPane, NibblePosition};

/// The result of a validated hex edit input.
///
/// Contains the byte offset, old value, and new value needed to
/// apply (or undo) the modification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexEditAction {
    /// The byte offset being modified.
    pub byte_offset: u64,
    /// The new byte value to write.
    pub new_value: u8,
    /// The previous byte value (for undo).
    pub old_value: u8,
}

/// Manages byte-level editing in hex mode.
///
/// Validates input characters, computes new byte values from nibble
/// edits, and enforces read-only mode guards.
#[derive(Debug, Clone)]
pub struct HexEditState {
    /// Whether editing is permitted (false in Browse/View mode).
    editing_enabled: bool,
    /// EBCDIC warning shown flag (per-session, per-file).
    ebcdic_warning_shown: bool,
}

impl Default for HexEditState {
    fn default() -> Self {
        Self::new()
    }
}

impl HexEditState {
    /// Create a new edit state with editing enabled.
    pub fn new() -> Self {
        Self {
            editing_enabled: true,
            ebcdic_warning_shown: false,
        }
    }

    /// Process a hex digit keystroke in the Hex_Pane.
    ///
    /// Returns the byte modification to apply, or an error if the digit
    /// is invalid or editing is not permitted.
    pub fn input_hex_digit(
        &self,
        digit: char,
        cursor: &HexCursor,
        current_byte: u8,
    ) -> Result<HexEditAction, HexError> {
        if !self.editing_enabled {
            return Err(HexError::EditNotAllowed("Browse/View".to_string()));
        }

        let nibble_value = char_to_nibble(digit)?;

        let new_value = match cursor.nibble() {
            NibblePosition::High => (nibble_value << 4) | (current_byte & 0x0F),
            NibblePosition::Low => (current_byte & 0xF0) | nibble_value,
        };

        Ok(HexEditAction {
            byte_offset: cursor.byte_offset(),
            new_value,
            old_value: current_byte,
        })
    }

    /// Process a character keystroke in the ASCII_Pane.
    ///
    /// Returns the byte modification to apply, or an error if the character
    /// is not printable ASCII or editing is not permitted.
    pub fn input_ascii_char(
        &self,
        ch: char,
        cursor: &HexCursor,
        current_byte: u8,
    ) -> Result<HexEditAction, HexError> {
        if !self.editing_enabled {
            return Err(HexError::EditNotAllowed("Browse/View".to_string()));
        }

        let byte_value = ch as u32;
        if !(0x20..=0x7E).contains(&byte_value) {
            return Err(HexError::NonPrintableAscii(byte_value as u8));
        }

        Ok(HexEditAction {
            byte_offset: cursor.byte_offset(),
            new_value: byte_value as u8,
            old_value: current_byte,
        })
    }

    /// Set editing enabled/disabled based on editor mode.
    pub fn set_editing_enabled(&mut self, enabled: bool) {
        self.editing_enabled = enabled;
    }

    /// Whether editing is currently enabled.
    pub fn is_editing_enabled(&self) -> bool {
        self.editing_enabled
    }

    /// Check if EBCDIC warning should be shown.
    ///
    /// Returns the warning message on first call when EBCDIC is active,
    /// `None` on subsequent calls or when not in EBCDIC mode.
    pub fn check_ebcdic_warning(&mut self, is_ebcdic: bool) -> Option<&str> {
        if is_ebcdic && !self.ebcdic_warning_shown {
            self.ebcdic_warning_shown = true;
            Some("Hex editing on EBCDIC files modifies raw bytes directly — ensure edited values are valid EBCDIC characters")
        } else {
            None
        }
    }

    /// Reset the EBCDIC warning flag (e.g., when switching files).
    pub fn reset_ebcdic_warning(&mut self) {
        self.ebcdic_warning_shown = false;
    }

    /// Determine which pane should handle the input character.
    pub fn classify_input(_ch: char, active_pane: HexPane) -> HexPane {
        match active_pane {
            HexPane::Hex => HexPane::Hex,
            HexPane::Ascii => HexPane::Ascii,
        }
    }
}

/// Convert a hex digit character to its nibble value (0–15).
fn char_to_nibble(ch: char) -> Result<u8, HexError> {
    match ch {
        '0'..='9' => Ok(ch as u8 - b'0'),
        'A'..='F' => Ok(ch as u8 - b'A' + 10),
        'a'..='f' => Ok(ch as u8 - b'a' + 10),
        _ => Err(HexError::InvalidHexDigit(ch)),
    }
}

/// Check if a character is a valid hex digit.
pub fn is_hex_digit(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}

/// Check if a character is printable ASCII (0x20–0x7E).
pub fn is_printable_ascii(ch: char) -> bool {
    let val = ch as u32;
    (0x20..=0x7E).contains(&val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn edit_state() -> HexEditState {
        HexEditState::new()
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn input_hex_digit_high_nibble_overwrites_correctly() {
        let state = edit_state();
        let cursor = HexCursor::at_offset(0); // High nibble
        let action = state.input_hex_digit('A', &cursor, 0x34).unwrap();
        assert_eq!(action.new_value, 0xA4); // A in high nibble, 4 preserved in low
        assert_eq!(action.old_value, 0x34);
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn input_hex_digit_low_nibble_overwrites_correctly() {
        let state = edit_state();
        let mut cursor = HexCursor::at_offset(0);
        cursor.set_nibble(NibblePosition::Low);
        let action = state.input_hex_digit('B', &cursor, 0xA0).unwrap();
        assert_eq!(action.new_value, 0xAB); // A preserved in high, B in low
        assert_eq!(action.old_value, 0xA0);
    }

    // Validates: Requirement 4 AC 4
    #[test]
    fn input_hex_digit_rejects_invalid_character() {
        let state = edit_state();
        let cursor = HexCursor::at_offset(0);
        let result = state.input_hex_digit('G', &cursor, 0x00);
        assert_eq!(result.unwrap_err(), HexError::InvalidHexDigit('G'));
    }

    // Validates: Requirement 4 AC 4
    #[test]
    fn input_hex_digit_rejects_non_hex_symbols() {
        let state = edit_state();
        let cursor = HexCursor::at_offset(0);
        assert!(state.input_hex_digit('X', &cursor, 0x00).is_err());
        assert!(state.input_hex_digit(' ', &cursor, 0x00).is_err());
        assert!(state.input_hex_digit('z', &cursor, 0x00).is_err());
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn input_hex_digit_accepts_lowercase() {
        let state = edit_state();
        let cursor = HexCursor::at_offset(0);
        let action = state.input_hex_digit('f', &cursor, 0x00).unwrap();
        assert_eq!(action.new_value, 0xF0);
    }

    // Validates: Requirement 4 AC 3
    #[test]
    fn input_ascii_char_overwrites_byte() {
        let state = edit_state();
        let cursor = HexCursor::at_offset(5);
        let action = state.input_ascii_char('X', &cursor, 0x41).unwrap();
        assert_eq!(action.new_value, 0x58); // 'X' = 0x58
        assert_eq!(action.old_value, 0x41); // 'A' = 0x41
        assert_eq!(action.byte_offset, 5);
    }

    // Validates: Requirement 4 AC 3
    #[test]
    fn input_ascii_char_accepts_space_and_tilde() {
        let state = edit_state();
        let cursor = HexCursor::at_offset(0);
        // Space (0x20) — minimum printable
        let action = state.input_ascii_char(' ', &cursor, 0x00).unwrap();
        assert_eq!(action.new_value, 0x20);
        // Tilde (0x7E) — maximum printable
        let action = state.input_ascii_char('~', &cursor, 0x00).unwrap();
        assert_eq!(action.new_value, 0x7E);
    }

    // Validates: Requirement 4 AC 3
    #[test]
    fn input_ascii_char_rejects_non_printable() {
        let state = edit_state();
        let cursor = HexCursor::at_offset(0);
        // DEL (0x7F) — not printable
        let result = state.input_ascii_char('\x7F', &cursor, 0x00);
        assert!(result.is_err());
    }

    // Validates: Requirement 4 AC 6
    #[test]
    fn editing_rejected_when_disabled() {
        let mut state = edit_state();
        state.set_editing_enabled(false);
        let cursor = HexCursor::at_offset(0);

        let result = state.input_hex_digit('A', &cursor, 0x00);
        assert_eq!(
            result.unwrap_err(),
            HexError::EditNotAllowed("Browse/View".to_string())
        );

        let result = state.input_ascii_char('X', &cursor, 0x00);
        assert_eq!(
            result.unwrap_err(),
            HexError::EditNotAllowed("Browse/View".to_string())
        );
    }

    // Validates: Requirement 4 AC 9
    #[test]
    fn ebcdic_warning_shown_once() {
        let mut state = edit_state();
        let warning = state.check_ebcdic_warning(true);
        assert!(warning.is_some());

        // Second call returns None
        let warning = state.check_ebcdic_warning(true);
        assert!(warning.is_none());
    }

    // Validates: Requirement 4 AC 9
    #[test]
    fn ebcdic_warning_not_shown_when_not_ebcdic() {
        let mut state = edit_state();
        let warning = state.check_ebcdic_warning(false);
        assert!(warning.is_none());
    }

    #[test]
    fn is_hex_digit_identifies_valid_digits() {
        for ch in "0123456789ABCDEFabcdef".chars() {
            assert!(is_hex_digit(ch), "expected {ch} to be hex digit");
        }
        for ch in "GHIJghij !@#$%".chars() {
            assert!(!is_hex_digit(ch), "expected {ch} to NOT be hex digit");
        }
    }
}
