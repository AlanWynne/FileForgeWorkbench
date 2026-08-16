//! Error types for the background I/O subsystem.
//!
//! Defines [`IoError`] — the unified error type wrapping `VfsError` with additional
//! context about the operation phase, resource URI, and bytes transferred before failure.
//! Every variant produces a diagnostic message conforming to the format:
//! `[background-io] phase: description (uri: resource_uri, transferred: N bytes)`

use ff_vfs::VfsError;

/// Error type for all background I/O operations.
///
/// Wraps `VfsError` with operation-phase context, resource URI, and transfer state.
/// Every variant produces a message conforming to:
/// `[background-io] phase: description (uri: resource_uri, transferred: N bytes)`
///
/// # Errors
///
/// Each variant represents a specific failure phase in the I/O pipeline:
/// - Open failures occur before any data is transferred
/// - Read/Write chunk failures occur during streaming I/O
/// - Flush/Rename failures occur during the save finalization sequence
/// - Cleanup failures occur when error recovery itself fails
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IoError {
    /// Failed to open or initiate the VFS stream.
    #[error("[background-io] open: {description} (uri: {uri}, transferred: 0 bytes)")]
    OpenFailed {
        /// The resource URI that failed to open.
        uri: String,
        /// Human-readable description of the failure.
        description: String,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// Failed during chunk read.
    #[error("[background-io] read-chunk: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    ReadChunkFailed {
        /// The resource URI being read.
        uri: String,
        /// Human-readable description of the failure.
        description: String,
        /// Bytes successfully transferred before the failure.
        bytes_transferred: u64,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// Failed during chunk write.
    #[error("[background-io] write-chunk: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    WriteChunkFailed {
        /// The resource URI being written.
        uri: String,
        /// Human-readable description of the failure.
        description: String,
        /// Bytes successfully transferred before the failure.
        bytes_transferred: u64,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// Failed during flush/fsync.
    #[error(
        "[background-io] flush: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)"
    )]
    FlushFailed {
        /// The resource URI being flushed.
        uri: String,
        /// Human-readable description of the failure.
        description: String,
        /// Bytes successfully transferred before the failure.
        bytes_transferred: u64,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// Failed during atomic rename.
    #[error("[background-io] rename: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    RenameFailed {
        /// The resource URI being renamed.
        uri: String,
        /// Human-readable description of the failure.
        description: String,
        /// Bytes successfully transferred before the failure.
        bytes_transferred: u64,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// Failed during cleanup (temp file deletion).
    #[error("[background-io] cleanup: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    CleanupFailed {
        /// The resource URI being cleaned up.
        uri: String,
        /// Human-readable description of the failure.
        description: String,
        /// Bytes successfully transferred before the failure.
        bytes_transferred: u64,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// Insufficient disk space detected before write.
    #[error("[background-io] space-check: insufficient space for save (uri: {uri}, transferred: 0 bytes)")]
    InsufficientSpace {
        /// The resource URI for the save operation.
        uri: String,
        /// Required bytes for the save.
        required_bytes: u64,
        /// Available bytes on the target.
        available_bytes: u64,
    },

    /// Task was cancelled.
    #[error("[background-io] cancelled: operation cancelled by user (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    Cancelled {
        /// The resource URI of the cancelled operation.
        uri: String,
        /// Bytes successfully transferred before cancellation.
        bytes_transferred: u64,
    },

    /// All retries exhausted for a transient error.
    #[error("[background-io] retries-exhausted: {description} after {attempts} attempts (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    RetriesExhausted {
        /// The resource URI of the failed operation.
        uri: String,
        /// Human-readable description of the failure.
        description: String,
        /// Bytes successfully transferred before the failure.
        bytes_transferred: u64,
        /// Number of retry attempts made.
        attempts: u8,
        /// The underlying VFS error from the last attempt.
        #[source]
        source: VfsError,
    },

    /// Provider does not support required capability.
    #[error("[background-io] capability: provider '{provider}' does not support {capability} (uri: {uri}, transferred: 0 bytes)")]
    UnsupportedCapability {
        /// The resource URI.
        uri: String,
        /// The provider that lacks the capability.
        provider: String,
        /// The required capability name.
        capability: String,
    },

    /// Task not found (invalid TaskId).
    #[error(
        "[background-io] lookup: task {task_id} not found (uri: unknown, transferred: 0 bytes)"
    )]
    TaskNotFound {
        /// The task ID that was not found.
        task_id: u64,
    },

    /// Operation timed out.
    #[error("[background-io] timeout: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    Timeout {
        /// The resource URI.
        uri: String,
        /// Human-readable description of the timeout.
        description: String,
        /// Bytes successfully transferred before the timeout.
        bytes_transferred: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vfs_error() -> VfsError {
        VfsError::Io {
            uri: "vfs://local/test.txt".to_string(),
            operation: "read".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "connection lost"),
        }
    }

    #[test]
    fn open_failed_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::OpenFailed {
            uri: "vfs://local/test.txt".to_string(),
            description: "connection refused".to_string(),
            source: make_vfs_error(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("open:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://local/test.txt"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 0 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn read_chunk_failed_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::ReadChunkFailed {
            uri: "vfs://local/data.bin".to_string(),
            description: "stream interrupted".to_string(),
            bytes_transferred: 65536,
            source: make_vfs_error(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("read-chunk:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://local/data.bin"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 65536 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn write_chunk_failed_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::WriteChunkFailed {
            uri: "vfs://local/output.txt".to_string(),
            description: "disk full".to_string(),
            bytes_transferred: 1024,
            source: make_vfs_error(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("write-chunk:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://local/output.txt"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 1024 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn flush_failed_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::FlushFailed {
            uri: "vfs://local/file.dat".to_string(),
            description: "fsync failed".to_string(),
            bytes_transferred: 99999,
            source: make_vfs_error(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("flush:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://local/file.dat"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 99999 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn rename_failed_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::RenameFailed {
            uri: "vfs://local/target.txt".to_string(),
            description: "cross-device link".to_string(),
            bytes_transferred: 5000,
            source: make_vfs_error(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("rename:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://local/target.txt"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 5000 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn cleanup_failed_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::CleanupFailed {
            uri: "vfs://local/tmp.ffwtmp.abc123".to_string(),
            description: "permission denied on temp file".to_string(),
            bytes_transferred: 2048,
            source: make_vfs_error(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("cleanup:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://local/tmp.ffwtmp.abc123"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 2048 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn insufficient_space_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::InsufficientSpace {
            uri: "vfs://local/big.dat".to_string(),
            required_bytes: 1_000_000,
            available_bytes: 500_000,
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("space-check:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://local/big.dat"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 0 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn cancelled_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::Cancelled {
            uri: "vfs://local/large.log".to_string(),
            bytes_transferred: 10_000_000,
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("cancelled:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://local/large.log"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 10000000 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn retries_exhausted_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::RetriesExhausted {
            uri: "vfs://remote/data.csv".to_string(),
            description: "network timeout".to_string(),
            bytes_transferred: 32768,
            attempts: 3,
            source: make_vfs_error(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(
            msg.contains("retries-exhausted:"),
            "must contain phase: {msg}"
        );
        assert!(
            msg.contains("uri: vfs://remote/data.csv"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 32768 bytes"),
            "must contain bytes: {msg}"
        );
        assert!(
            msg.contains("3 attempts"),
            "must contain attempt count: {msg}"
        );
    }

    #[test]
    fn unsupported_capability_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::UnsupportedCapability {
            uri: "vfs://catalog/doc.txt".to_string(),
            provider: "catalog".to_string(),
            capability: "write".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("capability:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://catalog/doc.txt"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 0 bytes"),
            "must contain bytes: {msg}"
        );
    }

    #[test]
    fn task_not_found_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::TaskNotFound { task_id: 42 };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("lookup:"), "must contain phase: {msg}");
        assert!(msg.contains("task 42"), "must contain task id: {msg}");
    }

    #[test]
    fn timeout_display_conforms_to_format() {
        // Validates: Requirement 6 AC 1, AC 2
        let err = IoError::Timeout {
            uri: "vfs://remote/slow.dat".to_string(),
            description: "read timed out after 30s".to_string(),
            bytes_transferred: 4096,
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[background-io]"),
            "must start with prefix: {msg}"
        );
        assert!(msg.contains("timeout:"), "must contain phase: {msg}");
        assert!(
            msg.contains("uri: vfs://remote/slow.dat"),
            "must contain uri: {msg}"
        );
        assert!(
            msg.contains("transferred: 4096 bytes"),
            "must contain bytes: {msg}"
        );
    }
}
