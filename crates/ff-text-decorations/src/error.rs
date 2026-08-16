//! Error types for the text-decorations crate.
//!
//! All errors follow the `[decorations] operation: description` format
//! per cross-cutting Requirement 8.

/// Errors produced by the text-decorations crate.
///
/// Addresses: Cross-cutting Req 8 (error format)
#[derive(Debug, thiserror::Error)]
pub enum DecorationError {
    /// Position is beyond document length.
    #[error(
        "[decorations] value_at: position {position} exceeds document length {document_length}"
    )]
    PositionOutOfRange { position: u64, document_length: u64 },

    /// Indicator number is out of the valid range (0–43).
    #[error("[decorations] indicator: number {number} exceeds maximum 43")]
    InvalidIndicatorNumber { number: u8 },

    /// Marker number is out of the valid range (0–31).
    #[error("[decorations] marker: number {number} exceeds maximum 31")]
    InvalidMarkerNumber { number: u8 },

    /// Attempted to write to the lexer range (0–7) from non-lexer code.
    #[error("[decorations] fill_range: indicator {number} is in the lexer range (0–7), reserved for syntax-highlighting")]
    LexerRangeViolation { number: u8 },

    /// No available indicator slots in the container range.
    #[error("[decorations] allocate: all container-range indicator numbers (8–31) are allocated")]
    NoAvailableIndicators,

    /// Attempted to release an indicator that was not allocated.
    #[error("[decorations] release: indicator {number} was not allocated")]
    NotAllocated { number: u8 },

    /// Line number out of range.
    #[error("[decorations] marker: line {line} exceeds document line count {line_count}")]
    LineOutOfRange { line: u64, line_count: u64 },

    /// Theme value validation failure.
    #[error("[decorations] theme: invalid value for {field}: {reason}")]
    InvalidThemeValue { field: String, reason: String },
}
