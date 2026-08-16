//! Error types for the ff-document-model crate.
//!
//! All errors follow the `[document] operation: description` format per
//! project error message standards.

use ff_vfs::VfsError;

/// Errors originating from the ff-document-model crate.
///
/// Formatted per Error Message Standards: `[document] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DocumentError {
    /// Attempted mutation on a read-only document.
    #[error("[document] {operation}: document is read-only")]
    ReadOnly {
        /// The operation that was attempted.
        operation: String,
    },

    /// Byte position is out of valid range.
    #[error("[document] {operation}: position {position} out of range (length: {length})")]
    PositionOutOfRange {
        /// The operation that was attempted.
        operation: String,
        /// The invalid position.
        position: u64,
        /// The document length.
        length: u64,
    },

    /// Line number is out of valid range.
    #[error("[document] {operation}: line {line} out of range (total: {total})")]
    LineOutOfRange {
        /// The operation that was attempted.
        operation: String,
        /// The invalid line number.
        line: u64,
        /// Total line count.
        total: u64,
    },

    /// VFS I/O error during streaming load or save.
    #[error("[document] {operation}: VFS error for {uri}: {source}")]
    VfsIo {
        /// The operation that was attempted.
        operation: String,
        /// The URI that caused the error.
        uri: String,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// Streaming load was cancelled.
    #[error("[document] load: cancelled after {bytes_loaded} bytes")]
    LoadCancelled {
        /// Bytes loaded before cancellation.
        bytes_loaded: u64,
    },

    /// Watcher already registered (duplicate).
    #[error("[document] add_watcher: watcher is already registered")]
    DuplicateWatcher,

    /// Watcher handle not found for removal.
    #[error("[document] remove_watcher: handle {handle_id} not found")]
    WatcherNotFound {
        /// The handle ID that was not found.
        handle_id: u64,
    },

    /// Document is still loading; operation not available.
    #[error("[document] {operation}: document is still loading ({bytes_loaded} bytes loaded)")]
    StillLoading {
        /// The operation that was attempted.
        operation: String,
        /// Bytes loaded so far.
        bytes_loaded: u64,
    },
}

impl From<VfsError> for DocumentError {
    fn from(err: VfsError) -> Self {
        DocumentError::VfsIo {
            operation: "vfs".to_string(),
            uri: String::new(),
            source: err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_error_format_includes_operation() {
        let err = DocumentError::ReadOnly {
            operation: "insert".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[document]"));
        assert!(msg.contains("insert"));
        assert!(msg.contains("read-only"));
    }

    #[test]
    fn position_out_of_range_error_includes_context() {
        let err = DocumentError::PositionOutOfRange {
            operation: "delete".to_string(),
            position: 100,
            length: 50,
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[document]"));
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
    }

    #[test]
    fn line_out_of_range_error_includes_context() {
        let err = DocumentError::LineOutOfRange {
            operation: "line_start".to_string(),
            line: 42,
            total: 10,
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[document]"));
        assert!(msg.contains("42"));
        assert!(msg.contains("10"));
    }

    #[test]
    fn load_cancelled_includes_bytes() {
        let err = DocumentError::LoadCancelled { bytes_loaded: 1024 };
        let msg = err.to_string();
        assert!(msg.contains("1024"));
    }

    #[test]
    fn duplicate_watcher_format() {
        let err = DocumentError::DuplicateWatcher;
        let msg = err.to_string();
        assert!(msg.starts_with("[document]"));
        assert!(msg.contains("already registered"));
    }

    #[test]
    fn watcher_not_found_includes_handle_id() {
        let err = DocumentError::WatcherNotFound { handle_id: 7 };
        let msg = err.to_string();
        assert!(msg.contains("7"));
    }
}
