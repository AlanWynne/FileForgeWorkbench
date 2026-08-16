//! Error types for the edit-operations crate.
//!
//! All errors follow the `[edit] operation: description` format as required
//! by the cross-cutting error message standards.

/// Errors produced by the edit-operations crate.
///
/// Each variant carries enough context to diagnose the problem without
/// consulting additional state.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EditError {
    /// Attempted to edit in Browse mode (read-only).
    #[error("[edit] {operation}: document is in Browse mode (read-only)")]
    ReadOnly {
        /// The operation that was attempted.
        operation: String,
    },

    /// Edit position is outside the active BOUNDS range.
    #[error("[edit] {operation}: column {column} is outside BOUNDS ({left}–{right})")]
    OutsideBounds {
        /// The operation that was attempted.
        operation: String,
        /// The column where the edit was attempted.
        column: u64,
        /// Left boundary (inclusive).
        left: u64,
        /// Right boundary (inclusive).
        right: u64,
    },

    /// Invalid BOUNDS values supplied.
    #[error("[edit] bounds: invalid range ({left}, {right}) — left must be >= 1 and right > left")]
    InvalidBounds {
        /// Supplied left boundary.
        left: u64,
        /// Supplied right boundary.
        right: u64,
    },

    /// Cannot drop the last remaining selection range.
    #[error("[edit] selection: cannot remove last remaining caret")]
    LastCaretRemoval,

    /// The document buffer reported an error during mutation.
    #[error("[edit] {operation}: document error — {description}")]
    DocumentError {
        /// The operation that triggered the error.
        operation: String,
        /// Description of the underlying document error.
        description: String,
    },

    /// Clipboard operation failed (system clipboard unavailable).
    #[error("[edit] clipboard: {description}")]
    ClipboardError {
        /// Description of the clipboard failure.
        description: String,
    },

    /// A no-op boundary condition (e.g., line transpose on first line).
    /// Used internally to signal no action taken.
    #[error("[edit] {operation}: no action taken at boundary")]
    NoOpAtBoundary {
        /// The operation that was a no-op.
        operation: String,
    },

    /// Line number is out of range for the document.
    #[error("[edit] {operation}: line {line} is out of range (document has {total} lines)")]
    LineOutOfRange {
        /// The operation that was attempted.
        operation: String,
        /// The requested line number.
        line: u64,
        /// Total lines in the document.
        total: u64,
    },

    /// Column is out of range for the given line.
    #[error("[edit] {operation}: column {column} is out of range for line {line}")]
    ColumnOutOfRange {
        /// The operation that was attempted.
        operation: String,
        /// The requested column.
        column: u64,
        /// The line on which the column was requested.
        line: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_error_formats_correctly() {
        let err = EditError::ReadOnly {
            operation: "insert_char".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[edit] insert_char: document is in Browse mode (read-only)"
        );
    }

    #[test]
    fn outside_bounds_error_formats_correctly() {
        let err = EditError::OutsideBounds {
            operation: "insert_char".to_string(),
            column: 80,
            left: 1,
            right: 72,
        };
        assert_eq!(
            err.to_string(),
            "[edit] insert_char: column 80 is outside BOUNDS (1–72)"
        );
    }

    #[test]
    fn invalid_bounds_error_formats_correctly() {
        let err = EditError::InvalidBounds { left: 0, right: 5 };
        assert_eq!(
            err.to_string(),
            "[edit] bounds: invalid range (0, 5) — left must be >= 1 and right > left"
        );
    }

    #[test]
    fn last_caret_removal_error_formats_correctly() {
        let err = EditError::LastCaretRemoval;
        assert_eq!(
            err.to_string(),
            "[edit] selection: cannot remove last remaining caret"
        );
    }

    #[test]
    fn document_error_formats_correctly() {
        let err = EditError::DocumentError {
            operation: "delete_back".to_string(),
            description: "buffer underflow".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[edit] delete_back: document error — buffer underflow"
        );
    }

    #[test]
    fn clipboard_error_formats_correctly() {
        let err = EditError::ClipboardError {
            description: "system clipboard unavailable".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[edit] clipboard: system clipboard unavailable"
        );
    }

    #[test]
    fn no_op_at_boundary_formats_correctly() {
        let err = EditError::NoOpAtBoundary {
            operation: "line_transpose".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[edit] line_transpose: no action taken at boundary"
        );
    }

    #[test]
    fn line_out_of_range_formats_correctly() {
        let err = EditError::LineOutOfRange {
            operation: "delete_line".to_string(),
            line: 100,
            total: 50,
        };
        assert_eq!(
            err.to_string(),
            "[edit] delete_line: line 100 is out of range (document has 50 lines)"
        );
    }

    #[test]
    fn column_out_of_range_formats_correctly() {
        let err = EditError::ColumnOutOfRange {
            operation: "insert_char".to_string(),
            column: 200,
            line: 5,
        };
        assert_eq!(
            err.to_string(),
            "[edit] insert_char: column 200 is out of range for line 5"
        );
    }
}
