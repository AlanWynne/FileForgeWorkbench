//! Error types for the external modification detection system.
//!
//! All errors follow the `[external-mod] operation: description` format with
//! resource URI context where applicable.

use ff_vfs::{ResourceUri, VfsError};

use crate::types::DocumentId;

/// Error type for all external modification detection failures.
///
/// Display format: `[external-mod] operation: description`
///
/// # Variants
///
/// Each variant carries enough context to produce a meaningful diagnostic
/// message that identifies the failing operation and the affected resource.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[allow(clippy::result_large_err)]
pub enum ExternalModError {
    /// VFS stat operation failed for a resource.
    #[error("[external-mod] stat: VFS error for {uri} — {source}")]
    VfsStatFailed {
        /// The URI of the resource that stat failed on.
        uri: ResourceUri,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// VFS watch registration failed for a resource.
    #[error("[external-mod] watch: VFS error for {uri} — {source}")]
    VfsWatchFailed {
        /// The URI of the resource that watch registration failed for.
        uri: ResourceUri,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// VFS watch cancellation failed.
    #[error("[external-mod] cancel-watch: failed to cancel watch for {uri} — {reason}")]
    WatchCancellationFailed {
        /// The URI of the resource whose watch could not be cancelled.
        uri: ResourceUri,
        /// Description of why cancellation failed.
        reason: String,
    },

    /// Reload operation failed (VFS read or document update error).
    #[error("[external-mod] reload: failed to reload {uri} — {reason}")]
    ReloadFailed {
        /// The URI of the resource that failed to reload.
        uri: ResourceUri,
        /// Description of why the reload failed.
        reason: String,
    },

    /// Document is not registered for external modification tracking.
    #[error("[external-mod] {operation}: document {doc_id:?} is not registered")]
    DocumentNotFound {
        /// The operation that was attempted.
        operation: String,
        /// The document identifier that was not found.
        doc_id: DocumentId,
    },

    /// The VFS provider does not support the required operation and fallback also failed.
    #[error("[external-mod] watch: provider does not support watching for {uri}")]
    ProviderUnsupported {
        /// The URI of the resource that cannot be monitored.
        uri: ResourceUri,
    },

    /// Configuration value is invalid (out of range or wrong type).
    #[error("[external-mod] config: invalid value for '{key}' — {reason}")]
    ConfigInvalid {
        /// The configuration key that is invalid.
        key: String,
        /// Description of why the value is invalid.
        reason: String,
    },

    /// Polling operation timed out without producing a result.
    #[error("[external-mod] poll: timeout after {duration_ms}ms for {uri}")]
    PollingTimeout {
        /// The URI of the resource that timed out.
        uri: ResourceUri,
        /// The timeout duration in milliseconds.
        duration_ms: u64,
    },

    /// The batch coalescer's event buffer exceeded its maximum capacity.
    #[error("[external-mod] batch: overflow — {count} events exceed maximum buffer capacity")]
    BatchOverflow {
        /// The number of events that caused the overflow.
        count: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vfs_stat_failed_display_includes_uri_and_prefix() {
        let err = ExternalModError::VfsStatFailed {
            uri: ResourceUri::new("local", "/test/file.rs"),
            source: VfsError::NotFound {
                uri: "vfs://local/test/file.rs".to_string(),
                operation: "stat".to_string(),
            },
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("stat"));
        assert!(msg.contains("/test/file.rs"));
    }

    #[test]
    fn vfs_watch_failed_display_includes_uri_and_prefix() {
        let err = ExternalModError::VfsWatchFailed {
            uri: ResourceUri::new("local", "/test/file.rs"),
            source: VfsError::UnsupportedOperation {
                operation: "watch".to_string(),
                provider: "local".to_string(),
            },
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("watch"));
    }

    #[test]
    fn watch_cancellation_failed_display_includes_reason() {
        let err = ExternalModError::WatchCancellationFailed {
            uri: ResourceUri::new("local", "/test/file.rs"),
            reason: "handle already dropped".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("cancel-watch"));
        assert!(msg.contains("handle already dropped"));
    }

    #[test]
    fn reload_failed_display_includes_uri_and_reason() {
        let err = ExternalModError::ReloadFailed {
            uri: ResourceUri::new("local", "/test/file.rs"),
            reason: "permission denied".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("reload"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn document_not_found_display_includes_doc_id() {
        let err = ExternalModError::DocumentNotFound {
            operation: "unregister".to_string(),
            doc_id: DocumentId(42),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("unregister"));
        assert!(msg.contains("42"));
    }

    #[test]
    fn provider_unsupported_display_includes_uri() {
        let err = ExternalModError::ProviderUnsupported {
            uri: ResourceUri::new("remote", "/shared/file.txt"),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("provider does not support"));
    }

    #[test]
    fn config_invalid_display_includes_key_and_reason() {
        let err = ExternalModError::ConfigInvalid {
            key: "batch_debounce_ms".to_string(),
            reason: "value must be between 100 and 5000".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("config"));
        assert!(msg.contains("batch_debounce_ms"));
    }

    #[test]
    fn polling_timeout_display_includes_duration() {
        let err = ExternalModError::PollingTimeout {
            uri: ResourceUri::new("local", "/test/file.rs"),
            duration_ms: 5000,
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("timeout"));
        assert!(msg.contains("5000"));
    }

    #[test]
    fn batch_overflow_display_includes_count() {
        let err = ExternalModError::BatchOverflow { count: 1024 };
        let msg = err.to_string();
        assert!(msg.starts_with("[external-mod]"));
        assert!(msg.contains("overflow"));
        assert!(msg.contains("1024"));
    }
}
