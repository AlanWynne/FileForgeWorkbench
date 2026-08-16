//! Core types shared across the ff-hex crate.
//!
//! Defines newtypes, enums, and small value types used throughout
//! the hex display subsystem.

use std::fmt;

// ─── ByteOffset ─────────────────────────────────────────────────────────────

/// A 0-based byte offset within the document buffer.
///
/// Wraps `u64` to prevent accidental confusion with line numbers,
/// column indices, or other positional values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ByteOffset(pub u64);

impl ByteOffset {
    /// Create a new byte offset.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get the raw u64 value.
    pub fn value(self) -> u64 {
        self.0
    }
}

impl From<u64> for ByteOffset {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<ByteOffset> for u64 {
    fn from(offset: ByteOffset) -> Self {
        offset.0
    }
}

impl fmt::Display for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08X}", self.0)
    }
}

impl fmt::LowerHex for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl std::ops::Add<u64> for ByteOffset {
    type Output = Self;

    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl std::ops::Sub<u64> for ByteOffset {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self {
        Self(self.0.saturating_sub(rhs))
    }
}

// ─── NibblePosition ─────────────────────────────────────────────────────────

/// Position within a byte when editing in the Hex_Pane.
///
/// Each byte is represented by two hex digits: the high nibble (bits 7–4)
/// and the low nibble (bits 3–0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NibblePosition {
    /// High nibble (first hex digit, bits 7–4).
    #[default]
    High,
    /// Low nibble (second hex digit, bits 3–0).
    Low,
}

impl NibblePosition {
    /// Advance to the next nibble position.
    ///
    /// Returns `(next_nibble, should_advance_byte)`:
    /// - High → Low (stay on same byte)
    /// - Low → High (advance to next byte)
    pub fn advance(self) -> (Self, bool) {
        match self {
            Self::High => (Self::Low, false),
            Self::Low => (Self::High, true),
        }
    }

    /// Retreat to the previous nibble position.
    ///
    /// Returns `(prev_nibble, should_retreat_byte)`:
    /// - Low → High (stay on same byte)
    /// - High → Low (retreat to previous byte)
    pub fn retreat(self) -> (Self, bool) {
        match self {
            Self::Low => (Self::High, false),
            Self::High => (Self::Low, true),
        }
    }
}

// ─── HexPane ────────────────────────────────────────────────────────────────

/// Which pane currently has editing focus in hex mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HexPane {
    /// Focus is in the hex digit pane (editing nibbles).
    #[default]
    Hex,
    /// Focus is in the ASCII character pane.
    Ascii,
}

// ─── HexMode ────────────────────────────────────────────────────────────────

/// The current hex display mode state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HexMode {
    /// Normal text display (hex mode inactive).
    #[default]
    Off,
    /// Hex display mode is active.
    On,
}

impl HexMode {
    /// Toggle the mode.
    pub fn toggle(self) -> Self {
        match self {
            Self::Off => Self::On,
            Self::On => Self::Off,
        }
    }

    /// Whether hex mode is currently active.
    pub fn is_active(self) -> bool {
        matches!(self, Self::On)
    }
}

// ─── HexDigitCase ───────────────────────────────────────────────────────────

/// Hex digit display case preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HexDigitCase {
    /// Display hex digits A–F in uppercase (default).
    #[default]
    Uppercase,
    /// Display hex digits a–f in lowercase.
    Lowercase,
}

impl HexDigitCase {
    /// Format a byte as two hex digits using the configured case.
    pub fn format_byte(self, byte: u8) -> String {
        match self {
            Self::Uppercase => format!("{byte:02X}"),
            Self::Lowercase => format!("{byte:02x}"),
        }
    }

    /// Format an offset value with the configured case.
    pub fn format_offset(self, offset: u64, width: usize) -> String {
        match self {
            Self::Uppercase => format!("{offset:0>width$X}"),
            Self::Lowercase => format!("{offset:0>width$x}"),
        }
    }
}

// ─── BytesPerRow ────────────────────────────────────────────────────────────

/// Valid values for bytes displayed per hex row.
///
/// Only powers of 2 in {8, 16, 32, 64} are accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum BytesPerRow {
    /// 8 bytes per row.
    Eight = 8,
    /// 16 bytes per row (default).
    #[default]
    Sixteen = 16,
    /// 32 bytes per row.
    ThirtyTwo = 32,
    /// 64 bytes per row.
    SixtyFour = 64,
}

impl BytesPerRow {
    /// Attempt to create from a raw u32 value.
    ///
    /// Returns `None` for invalid values (not 8, 16, 32, or 64).
    pub fn from_value(value: u32) -> Option<Self> {
        match value {
            8 => Some(Self::Eight),
            16 => Some(Self::Sixteen),
            32 => Some(Self::ThirtyTwo),
            64 => Some(Self::SixtyFour),
            _ => None,
        }
    }

    /// Get the value as usize for arithmetic.
    pub fn as_usize(self) -> usize {
        self as usize
    }

    /// Get the value as u64 for offset arithmetic.
    pub fn as_u64(self) -> u64 {
        self as u64
    }
}

impl TryFrom<u32> for BytesPerRow {
    type Error = crate::error::HexError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_value(value).ok_or(crate::error::HexError::InvalidBytesPerRow(value))
    }
}

// ─── AutoActivateBinary ─────────────────────────────────────────────────────

/// Configuration for auto-activating hex mode on binary files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AutoActivateBinary {
    /// Prompt the user when binary content is detected (default).
    #[default]
    Prompt,
    /// Always activate hex mode for binary files without prompting.
    Always,
    /// Never auto-activate; user must manually invoke HEX ON.
    Never,
}

// ─── HexInput ───────────────────────────────────────────────────────────────

/// Input event types handled by the hex mode controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexInput {
    /// A hex digit typed in the Hex_Pane (0-9, A-F, a-f).
    HexDigit(char),
    /// A printable ASCII character typed in the ASCII_Pane.
    AsciiChar(char),
    /// Arrow key navigation.
    Arrow(ArrowDirection),
    /// Tab key: switch panes.
    SwitchPane,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
}

/// Arrow key direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    /// Move up one row.
    Up,
    /// Move down one row.
    Down,
    /// Move left (one nibble in hex pane, one byte in ASCII pane).
    Left,
    /// Move right (one nibble in hex pane, one byte in ASCII pane).
    Right,
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.3, 13.1-13.4
    #[test]
    fn byte_offset_display_formats_as_8_digit_uppercase_hex() {
        let offset = ByteOffset::new(0x0000_001A);
        assert_eq!(offset.to_string(), "0000001A");

        let zero = ByteOffset::new(0);
        assert_eq!(zero.to_string(), "00000000");

        let large = ByteOffset::new(0xDEAD_BEEF);
        assert_eq!(large.to_string(), "DEADBEEF");
    }

    // Validates: Requirement 2.3
    #[test]
    fn byte_offset_from_u64_conversion() {
        let offset: ByteOffset = 42u64.into();
        assert_eq!(offset.value(), 42);

        let back: u64 = offset.into();
        assert_eq!(back, 42);
    }

    // Validates: Requirement 2.3
    #[test]
    fn byte_offset_arithmetic() {
        let offset = ByteOffset::new(100);
        assert_eq!((offset + 10).value(), 110);
        assert_eq!((offset - 10).value(), 90);
        // Saturating subtraction
        let small = ByteOffset::new(5);
        assert_eq!((small - 10).value(), 0);
    }

    // Validates: Requirement 3.1-3.4
    #[test]
    fn bytes_per_row_valid_values_accepted() {
        assert_eq!(BytesPerRow::from_value(8), Some(BytesPerRow::Eight));
        assert_eq!(BytesPerRow::from_value(16), Some(BytesPerRow::Sixteen));
        assert_eq!(BytesPerRow::from_value(32), Some(BytesPerRow::ThirtyTwo));
        assert_eq!(BytesPerRow::from_value(64), Some(BytesPerRow::SixtyFour));
    }

    // Validates: Requirement 3.4
    #[test]
    fn bytes_per_row_invalid_values_rejected() {
        assert_eq!(BytesPerRow::from_value(0), None);
        assert_eq!(BytesPerRow::from_value(1), None);
        assert_eq!(BytesPerRow::from_value(10), None);
        assert_eq!(BytesPerRow::from_value(128), None);
    }

    // Validates: Requirement 3.2
    #[test]
    fn bytes_per_row_default_is_sixteen() {
        assert_eq!(BytesPerRow::default(), BytesPerRow::Sixteen);
    }

    // Validates: Requirement 3.1
    #[test]
    fn bytes_per_row_try_from_u32() {
        assert_eq!(BytesPerRow::try_from(16u32).unwrap(), BytesPerRow::Sixteen);
        assert!(BytesPerRow::try_from(12u32).is_err());
    }

    // Validates: Requirement 4 AC 1-2
    #[test]
    fn nibble_position_advance_transitions() {
        let (next, advance_byte) = NibblePosition::High.advance();
        assert_eq!(next, NibblePosition::Low);
        assert!(!advance_byte);

        let (next, advance_byte) = NibblePosition::Low.advance();
        assert_eq!(next, NibblePosition::High);
        assert!(advance_byte);
    }

    // Validates: Requirement 6 AC 6
    #[test]
    fn nibble_position_retreat_transitions() {
        let (prev, retreat_byte) = NibblePosition::Low.retreat();
        assert_eq!(prev, NibblePosition::High);
        assert!(!retreat_byte);

        let (prev, retreat_byte) = NibblePosition::High.retreat();
        assert_eq!(prev, NibblePosition::Low);
        assert!(retreat_byte);
    }

    // Validates: Requirement 1 AC 3
    #[test]
    fn hex_mode_toggle() {
        assert_eq!(HexMode::Off.toggle(), HexMode::On);
        assert_eq!(HexMode::On.toggle(), HexMode::Off);
    }

    // Validates: Requirement 1 AC 1
    #[test]
    fn hex_mode_is_active() {
        assert!(!HexMode::Off.is_active());
        assert!(HexMode::On.is_active());
    }

    // Validates: Requirement 13 AC 2-3
    #[test]
    fn hex_digit_case_format_byte() {
        assert_eq!(HexDigitCase::Uppercase.format_byte(0xAB), "AB");
        assert_eq!(HexDigitCase::Lowercase.format_byte(0xAB), "ab");
        assert_eq!(HexDigitCase::Uppercase.format_byte(0x0F), "0F");
        assert_eq!(HexDigitCase::Lowercase.format_byte(0x0F), "0f");
    }

    // Validates: Requirement 13 AC 4
    #[test]
    fn hex_digit_case_format_offset() {
        assert_eq!(HexDigitCase::Uppercase.format_offset(0x1A4F, 8), "00001A4F");
        assert_eq!(HexDigitCase::Lowercase.format_offset(0x1A4F, 8), "00001a4f");
    }
}
