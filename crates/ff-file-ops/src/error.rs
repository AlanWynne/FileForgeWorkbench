//! Error types for all file operation failures.
//!
//! All variants include sufficient context for diagnostics.
//! Display format: `[file-ops] operation: description`

use ff_vfs::{ResourceUri, VfsError};

use crate::read_only::ReadOnlyStatus;

/// Error type for all file operation failures.
///
/// All variants include sufficient context for diagnostics.
/// Display format: `[file-ops] operation: description`
///
/// # Errors
///
/// Each variant represents a specific failure mode in file operations,
/// carrying enough context for logging and user notification.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FileOpsError {
    /// VFS read operation failed.
    #[error("[file-ops] {operation}: VFS read error for {uri} — {source}")]
    VfsReadError {
        /// The operation that was being performed.
        operation: String,
        /// The resource URI involved.
        uri: ResourceUri,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// VFS write operation failed.
    #[error("[file-ops] {operation}: VFS write error for {uri} — {source}")]
    VfsWriteError {
        /// The operation that was being performed.
        operation: String,
        /// The resource URI involved.
        uri: ResourceUri,
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// Atomic rename failed during save.
    #[error("[file-ops] save: atomic rename failed for {uri} — {reason}")]
    AtomicRenameFailed {
        /// The resource URI involved.
        uri: ResourceUri,
        /// Description of why rename failed.
        reason: String,
    },

    /// Backup copy creation failed (non-fatal, logged as WARN).
    #[error("[file-ops] backup: failed to create backup for {uri} — {reason}")]
    BackupFailed {
        /// The resource URI involved.
        uri: ResourceUri,
        /// Description of why backup failed.
        reason: String,
    },

    /// The resource was not found on the VFS.
    #[error("[file-ops] {operation}: resource not found — {uri}")]
    ResourceNotFound {
        /// The operation that was being performed.
        operation: String,
        /// The resource URI that was not found.
        uri: ResourceUri,
    },

    /// Permission was denied for the operation.
    #[error("[file-ops] {operation}: permission denied for {uri}")]
    PermissionDenied {
        /// The operation that was being performed.
        operation: String,
        /// The resource URI involved.
        uri: ResourceUri,
    },

    /// No VFS provider is available for the requested scheme.
    #[error("[file-ops] {operation}: provider unavailable for scheme '{scheme}'")]
    ProviderUnavailable {
        /// The operation that was being performed.
        operation: String,
        /// The scheme with no registered provider.
        scheme: String,
    },

    /// Document is read-only; mutation was rejected.
    #[error("[file-ops] {operation}: document is read-only ({status:?})")]
    ReadOnlyResource {
        /// The operation that was being performed.
        operation: String,
        /// The read-only status that caused rejection.
        status: ReadOnlyStatus,
    },

    /// A save is already in progress for the target document.
    #[error("[file-ops] save: operation already in progress for {uri}")]
    SaveInProgress {
        /// The resource URI being saved.
        uri: ResourceUri,
    },

    /// The user cancelled the operation.
    #[error("[file-ops] {operation}: cancelled by user")]
    UserCancelled {
        /// The operation that was cancelled.
        operation: String,
    },

    /// The URI string is invalid.
    #[error("[file-ops] {operation}: invalid URI '{uri}' — {reason}")]
    InvalidUri {
        /// The operation that was being performed.
        operation: String,
        /// The invalid URI string.
        uri: String,
        /// Description of why the URI is invalid.
        reason: String,
    },

    /// Document has no associated URI (e.g., revert on untitled document).
    #[error("[file-ops] {operation}: document has no associated resource URI")]
    NoUri {
        /// The operation that required a URI.
        operation: String,
    },

    /// External modification detected; user declined to proceed.
    #[error("[file-ops] save: external modification detected for {uri} — user declined")]
    ExternalModificationDeclined {
        /// The resource URI that was modified externally.
        uri: ResourceUri,
    },

    /// Configuration error (invalid setting value).
    #[error("[file-ops] config: invalid value for '{key}' — {reason}")]
    ConfigError {
        /// The configuration key with the invalid value.
        key: String,
        /// Description of the validation failure.
        reason: String,
    },

    /// The recent files list could not be persisted.
    #[error("[file-ops] recent: failed to persist recent files list — {reason}")]
    RecentPersistFailed {
        /// Description of the persistence failure.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.5 — FileOpsError variants exist
    #[test]
    fn error_variants_are_constructible() {
        let uri = ResourceUri::new("local", "/test.txt");

        let err = FileOpsError::VfsReadError {
            operation: "open".to_string(),
            uri: uri.clone(),
            source: VfsError::NotFound {
                uri: "/test.txt".to_string(),
                operation: "read".to_string(),
            },
        };
        assert!(err.to_string().starts_with("[file-ops]"));

        let err = FileOpsError::VfsWriteError {
            operation: "save".to_string(),
            uri: uri.clone(),
            source: VfsError::PermissionDenied {
                uri: "/test.txt".to_string(),
                operation: "write".to_string(),
            },
        };
        assert!(err.to_string().contains("VFS write error"));

        let err = FileOpsError::AtomicRenameFailed {
            uri: uri.clone(),
            reason: "cross-device".to_string(),
        };
        assert!(err.to_string().contains("atomic rename failed"));

        let err = FileOpsError::BackupFailed {
            uri: uri.clone(),
            reason: "disk full".to_string(),
        };
        assert!(err.to_string().contains("backup"));

        let err = FileOpsError::ResourceNotFound {
            operation: "open".to_string(),
            uri: uri.clone(),
        };
        assert!(err.to_string().contains("resource not found"));

        let err = FileOpsError::PermissionDenied {
            operation: "save".to_string(),
            uri: uri.clone(),
        };
        assert!(err.to_string().contains("permission denied"));

        let err = FileOpsError::ProviderUnavailable {
            operation: "save".to_string(),
            scheme: "remote".to_string(),
        };
        assert!(err.to_string().contains("provider unavailable"));

        let err = FileOpsError::ReadOnlyResource {
            operation: "edit".to_string(),
            status: ReadOnlyStatus::VfsRestricted,
        };
        assert!(err.to_string().contains("read-only"));

        let err = FileOpsError::SaveInProgress { uri: uri.clone() };
        assert!(err.to_string().contains("already in progress"));

        let err = FileOpsError::UserCancelled {
            operation: "save_as".to_string(),
        };
        assert!(err.to_string().contains("cancelled by user"));

        let err = FileOpsError::InvalidUri {
            operation: "open".to_string(),
            uri: "bad://uri".to_string(),
            reason: "malformed".to_string(),
        };
        assert!(err.to_string().contains("invalid URI"));
    }

    // Validates: Requirement 1.6 — Display messages are descriptive
    #[test]
    fn error_display_format_includes_file_ops_prefix() {
        let uri = ResourceUri::new("local", "/file.txt");

        let errors: Vec<FileOpsError> = vec![
            FileOpsError::VfsReadError {
                operation: "open".to_string(),
                uri: uri.clone(),
                source: VfsError::NotFound {
                    uri: "/file.txt".to_string(),
                    operation: "read".to_string(),
                },
            },
            FileOpsError::AtomicRenameFailed {
                uri: uri.clone(),
                reason: "test".to_string(),
            },
            FileOpsError::SaveInProgress { uri: uri.clone() },
            FileOpsError::UserCancelled {
                operation: "new".to_string(),
            },
            FileOpsError::NoUri {
                operation: "revert".to_string(),
            },
            FileOpsError::ExternalModificationDeclined { uri: uri.clone() },
            FileOpsError::ConfigError {
                key: "file.save_strategy".to_string(),
                reason: "unknown value".to_string(),
            },
            FileOpsError::RecentPersistFailed {
                reason: "disk full".to_string(),
            },
        ];

        for err in &errors {
            let msg = err.to_string();
            assert!(
                msg.starts_with("[file-ops]"),
                "Error message should start with [file-ops]: {msg}"
            );
        }
    }
}
