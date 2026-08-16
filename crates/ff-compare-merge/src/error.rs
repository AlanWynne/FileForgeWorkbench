//! Error types for the compare-and-merge crate.

/// Unified error type for all compare-and-merge operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CompareError {
    /// Resource not found at the specified URI.
    #[error("[compare] {operation}: resource not found: {uri}")]
    ResourceNotFound { uri: String, operation: String },

    /// No active document available for the requested operation.
    #[error("[compare] {operation}: {message}")]
    NoActiveDocument { operation: String, message: String },

    /// No active compare session for session-dependent operations.
    #[error("[compare] {operation}: no active compare session")]
    NoActiveSession { operation: String },

    /// Hunk index is out of range.
    #[error("[compare] {operation}: hunk index {index} out of range (total: {total})")]
    HunkIndexOutOfRange {
        operation: String,
        index: usize,
        total: usize,
    },

    /// Attempted merge on a read-only compare session.
    #[error("[compare] {operation}: merge operations not available in this session")]
    MergeNotAvailable { operation: String },

    /// Unresolved conflicts prevent building the merge result.
    #[error("[compare] build_result: {count} unresolved conflicts remain")]
    UnresolvedConflicts { count: usize },

    /// Clipboard does not contain text content.
    #[error("[compare] with_clipboard: clipboard does not contain text content")]
    ClipboardEmpty,

    /// No selection marked for comparison.
    #[error("[compare] selections: no selection marked for comparison")]
    NoMarkedSelection,

    /// Current selection is empty.
    #[error("[compare] selections: no text selected")]
    EmptySelection,

    /// Document has not been saved.
    #[error(
        "[compare] with_saved: document has not been saved — no saved version to compare against"
    )]
    DocumentNotSaved,

    /// Encoding error during content normalisation.
    #[error("[compare] {operation}: encoding error for {uri}: {reason}")]
    EncodingError {
        uri: String,
        operation: String,
        reason: String,
    },

    /// Binary/text mismatch.
    #[error("[compare] {operation}: mixed comparison — one resource is binary, the other is text")]
    MixedBinaryText { operation: String },
}
