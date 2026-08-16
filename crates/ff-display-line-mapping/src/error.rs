//! Error types for the display-line-mapping crate.
//!
//! Errors follow the project format: `[display-mapping] operation: description`

/// Errors originating from the ff-display-line-mapping crate.
///
/// Formatted per Error Message Standards: `[display-mapping] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DisplayMappingError {
    /// Document line index is out of valid range.
    #[error("[display-mapping] {operation}: doc_line {line} out of range (total: {total})")]
    LineOutOfRange {
        /// The operation that was attempted.
        operation: String,
        /// The invalid line index.
        line: usize,
        /// The total number of document lines.
        total: usize,
    },

    /// Display line index is out of valid range.
    #[error("[display-mapping] {operation}: display_line {line} out of range (total: {total})")]
    DisplayLineOutOfRange {
        /// The operation that was attempted.
        operation: String,
        /// The invalid display line index.
        line: usize,
        /// The total number of display lines.
        total: usize,
    },

    /// Height value is invalid (must be >= 1).
    #[error("[display-mapping] set_height: height 0 is not valid for doc_line {line}")]
    InvalidHeight {
        /// The line for which an invalid height was specified.
        line: usize,
    },

    /// Listener handle not found for removal.
    #[error("[display-mapping] remove_listener: handle {handle_id} not found")]
    ListenerNotFound {
        /// The handle ID that was not found.
        handle_id: u64,
    },
}
