//! Error handling and OS error → VfsError mapping.
//!
//! This module maps `std::io::Error` to the appropriate `VfsError` variant,
//! following the mapping table defined in Requirement 7. All errors are logged
//! at WARN level before being returned to the caller.

use std::io::ErrorKind;

use ff_vfs::VfsError;

/// Internal error types specific to the connector that don't map to VfsError directly.
///
/// These represent initialization failures within the connector itself.
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    /// The user's home directory could not be determined.
    #[error("[connector-local-fs] init: home directory not found")]
    HomeDirNotFound,

    /// The current working directory could not be determined.
    #[error("[connector-local-fs] init: failed to determine working directory: {reason}")]
    WorkingDirFailed {
        /// Description of why the working directory lookup failed.
        reason: String,
    },

    /// The file watcher could not be initialized.
    #[error("[connector-local-fs] init: file watcher initialization failed: {reason}")]
    WatcherInitFailed {
        /// Description of why initialization failed.
        reason: String,
    },
}

/// Maps an OS I/O error to a `VfsError` with full context.
///
/// Follows the error mapping table from Requirement 7:
/// - EACCES / ERROR_ACCESS_DENIED → PermissionDenied
/// - ENOENT / ERROR_FILE_NOT_FOUND → NotFound
/// - ENOSPC / ERROR_DISK_FULL → Io (with StorageFull semantics in message)
/// - ENAMETOOLONG → Io (with InvalidPath semantics in message)
/// - EBUSY / ETXTBSY → Io (with ResourceBusy semantics in message)
/// - ENOTEMPTY → Io (with DirectoryNotEmpty semantics in message)
/// - EROFS → PermissionDenied (read-only filesystem)
/// - All others → Io (generic)
///
/// All mapped errors are logged at WARN level via ff-logging.
///
/// # Arguments
///
/// * `error` - The OS I/O error to map.
/// * `operation` - The VFS operation that was attempted (e.g., "read", "write").
/// * `uri` - The resource URI that was being accessed.
///
/// Validates: Requirement 7 AC 1–10
pub fn map_io_error(error: std::io::Error, operation: &str, uri: &str) -> VfsError {
    let vfs_error = match error.kind() {
        ErrorKind::NotFound => VfsError::NotFound {
            uri: uri.to_string(),
            operation: operation.to_string(),
        },
        ErrorKind::PermissionDenied => VfsError::PermissionDenied {
            uri: uri.to_string(),
            operation: operation.to_string(),
        },
        ErrorKind::AlreadyExists => VfsError::AlreadyExists {
            uri: uri.to_string(),
            operation: operation.to_string(),
        },
        // DirectoryNotEmpty maps to a descriptive Io error since VfsError
        // doesn't have a DirectoryNotEmpty variant yet
        ErrorKind::DirectoryNotEmpty => VfsError::Io {
            uri: uri.to_string(),
            operation: operation.to_string(),
            source: error,
        },
        // StorageFull, InvalidPath, ResourceBusy — map via Io with the source error
        // which carries the OS-specific semantics
        ErrorKind::StorageFull => VfsError::Io {
            uri: uri.to_string(),
            operation: operation.to_string(),
            source: error,
        },
        ErrorKind::InvalidInput => VfsError::Io {
            uri: uri.to_string(),
            operation: operation.to_string(),
            source: error,
        },
        // All other errors map to the generic Io variant
        _ => {
            // Check for specific OS error codes that need special handling
            #[cfg(unix)]
            {
                if let Some(os_code) = error.raw_os_error() {
                    match os_code {
                        30 => {
                            // EROFS - Read-only file system
                            return VfsError::PermissionDenied {
                                uri: uri.to_string(),
                                operation: operation.to_string(),
                            };
                        }
                        16 => {
                            // EBUSY
                            return VfsError::Io {
                                uri: uri.to_string(),
                                operation: operation.to_string(),
                                source: error,
                            };
                        }
                        _ => {}
                    }
                }
            }
            VfsError::Io {
                uri: uri.to_string(),
                operation: operation.to_string(),
                source: error,
            }
        }
    };

    // Log at WARN level before returning (Requirement 7 AC 10)
    ff_logging::log_warn!(
        "[connector-local-fs] {}: {} (uri: {})",
        operation,
        format_error_brief(&vfs_error),
        uri
    );

    vfs_error
}

/// Formats an error message conforming to the `[connector-local-fs] operation: description`
/// format with a maximum of 200 characters.
///
/// Validates: Requirement 7 AC 9
pub fn format_error_message(operation: &str, description: &str) -> String {
    let prefix = format!("[connector-local-fs] {}: ", operation);
    let max_desc_len = 200 - prefix.len();
    if description.len() > max_desc_len {
        format!("{}{}", prefix, &description[..max_desc_len])
    } else {
        format!("{}{}", prefix, description)
    }
}

/// Produces a brief error description for logging purposes.
fn format_error_brief(error: &VfsError) -> String {
    match error {
        VfsError::NotFound { .. } => "resource not found".to_string(),
        VfsError::PermissionDenied { .. } => "permission denied".to_string(),
        VfsError::AlreadyExists { .. } => "resource already exists".to_string(),
        VfsError::Io { source, .. } => source.to_string(),
        _ => "operation failed".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // Validates: Requirement 7 AC 2
    #[test]
    fn map_io_error_not_found_produces_vfs_not_found() {
        let io_err = io::Error::new(ErrorKind::NotFound, "file not found");
        let vfs_err = map_io_error(io_err, "read", "vfs://local/missing.txt");
        match vfs_err {
            VfsError::NotFound { uri, operation } => {
                assert_eq!(uri, "vfs://local/missing.txt");
                assert_eq!(operation, "read");
            }
            other => panic!("expected NotFound, got: {other:?}"),
        }
    }

    // Validates: Requirement 7 AC 1
    #[test]
    fn map_io_error_permission_denied_produces_vfs_permission_denied() {
        let io_err = io::Error::new(ErrorKind::PermissionDenied, "access denied");
        let vfs_err = map_io_error(io_err, "write", "vfs://local/secret.txt");
        match vfs_err {
            VfsError::PermissionDenied { uri, operation } => {
                assert_eq!(uri, "vfs://local/secret.txt");
                assert_eq!(operation, "write");
            }
            other => panic!("expected PermissionDenied, got: {other:?}"),
        }
    }

    // Validates: Requirement 7 AC 6
    #[test]
    fn map_io_error_directory_not_empty_produces_vfs_io() {
        let io_err = io::Error::new(ErrorKind::DirectoryNotEmpty, "directory not empty");
        let vfs_err = map_io_error(io_err, "delete", "vfs://local/mydir");
        match vfs_err {
            VfsError::Io { uri, operation, .. } => {
                assert_eq!(uri, "vfs://local/mydir");
                assert_eq!(operation, "delete");
            }
            other => panic!("expected Io, got: {other:?}"),
        }
    }

    // Validates: Requirement 7 AC 3
    #[test]
    fn map_io_error_storage_full_produces_vfs_io() {
        let io_err = io::Error::new(ErrorKind::StorageFull, "no space left");
        let vfs_err = map_io_error(io_err, "write", "vfs://local/bigfile.bin");
        match vfs_err {
            VfsError::Io { uri, operation, .. } => {
                assert_eq!(uri, "vfs://local/bigfile.bin");
                assert_eq!(operation, "write");
            }
            other => panic!("expected Io, got: {other:?}"),
        }
    }

    // Validates: Requirement 7 AC 8
    #[test]
    fn map_io_error_unknown_kind_produces_vfs_io() {
        let io_err = io::Error::new(ErrorKind::Other, "some unknown error");
        let vfs_err = map_io_error(io_err, "stat", "vfs://local/file.txt");
        match vfs_err {
            VfsError::Io { uri, operation, .. } => {
                assert_eq!(uri, "vfs://local/file.txt");
                assert_eq!(operation, "stat");
            }
            other => panic!("expected Io, got: {other:?}"),
        }
    }

    // Validates: Requirement 7 AC 9
    #[test]
    fn format_error_message_respects_200_char_limit() {
        let long_desc = "x".repeat(300);
        let msg = format_error_message("read", &long_desc);
        assert!(msg.len() <= 200, "message is {} chars", msg.len());
        assert!(msg.starts_with("[connector-local-fs] read: "));
    }

    // Validates: Requirement 7 AC 9
    #[test]
    fn format_error_message_includes_operation_and_description() {
        let msg = format_error_message("write", "permission denied");
        assert_eq!(msg, "[connector-local-fs] write: permission denied");
    }
}
