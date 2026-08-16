//! Error types for the ff-hex crate.
//!
//! All error messages follow the `[hex] operation: description` format.

/// Errors produced by the ff-hex crate.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HexError {
    /// HEX ON issued when hex mode is already active.
    #[error("[hex] activate: hex mode is already active")]
    AlreadyActive,

    /// HEX OFF issued when hex mode is already inactive.
    #[error("[hex] deactivate: hex mode is already off")]
    AlreadyInactive,

    /// Invalid hex digit typed in Hex_Pane.
    #[error("[hex] input: invalid hex digit '{0}'")]
    InvalidHexDigit(char),

    /// Non-printable character typed in ASCII_Pane.
    #[error("[hex] input: character 0x{0:02X} is not printable ASCII")]
    NonPrintableAscii(u8),

    /// Editing attempted in Browse or View mode.
    #[error("[hex] edit: cannot edit in {0} mode")]
    EditNotAllowed(String),

    /// Invalid bytes_per_row value.
    #[error("[hex] config: invalid bytes_per_row value {0} (must be 8, 16, 32, or 64)")]
    InvalidBytesPerRow(u32),

    /// GOTO offset exceeds document size.
    #[error("[hex] goto: offset 0x{offset:X} exceeds document size (0x{size:X} bytes)")]
    OffsetOutOfRange {
        /// The requested offset.
        offset: u64,
        /// The document size in bytes.
        size: u64,
    },

    /// Invalid offset format in GOTO command.
    #[error("[hex] goto: invalid offset format '{0}'")]
    InvalidOffsetFormat(String),

    /// Hex pattern has odd number of digits.
    #[error("[hex] search: hex pattern must contain an even number of digits")]
    OddHexPatternLength,

    /// Hex pattern contains invalid characters.
    #[error("[hex] search: invalid character '{0}' in hex pattern")]
    InvalidHexPatternChar(char),

    /// Hex dump export failed.
    #[error("[hex] dump: export failed: {0}")]
    DumpExportFailed(String),

    /// Session state restore failed.
    #[error("[hex] session: failed to restore hex state: {0}")]
    SessionRestoreFailed(String),
}
