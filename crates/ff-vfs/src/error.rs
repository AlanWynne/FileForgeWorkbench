//! Unified error type for the VFS abstraction layer.
//!
//! All errors produced by VFS operations are represented as [`VfsError`] variants.
//! Each variant carries sufficient context to produce a diagnostic message conforming
//! to the project error message standard: `[vfs] operation: description` with the
//! resource URI included where applicable.

/// Unified error type for VFS operations.
///
/// Every variant follows the `[vfs] operation: description` format and includes
/// enough context (URI, scheme, operation name) for diagnostics.
///
/// # Error Format
///
/// All `Display` output starts with `[vfs]` followed by the operation name and a
/// descriptive message including the relevant resource URI or scheme.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VfsError {
    /// The requested resource was not found.
    #[error("[vfs] {operation}: resource not found: {uri}")]
    NotFound {
        /// The URI of the resource that was not found.
        uri: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Permission was denied for the requested operation.
    #[error("[vfs] {operation}: permission denied: {uri}")]
    PermissionDenied {
        /// The URI of the resource.
        uri: String,
        /// The operation that was denied.
        operation: String,
    },

    /// The resource already exists and cannot be created again.
    #[error("[vfs] {operation}: resource already exists: {uri}")]
    AlreadyExists {
        /// The URI of the existing resource.
        uri: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// The target URI does not refer to a directory or container.
    #[error("[vfs] {operation}: not a directory: {uri}")]
    NotADirectory {
        /// The URI that is not a directory.
        uri: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// The provider does not support the requested operation.
    #[error("[vfs] {operation}: unsupported by provider '{provider}'")]
    UnsupportedOperation {
        /// The operation that was attempted.
        operation: String,
        /// The provider that does not support the operation.
        provider: String,
    },

    /// The URI string failed validation during parsing.
    #[error("[vfs] parse: invalid URI '{uri}': {reason}")]
    InvalidUri {
        /// The URI string that failed validation.
        uri: String,
        /// A description of why validation failed.
        reason: String,
    },

    /// No provider is registered for the requested scheme.
    #[error("[vfs] route: provider unavailable: '{scheme}'")]
    ProviderUnavailable {
        /// The scheme that has no registered provider.
        scheme: String,
    },

    /// The operation exceeded the configured timeout.
    #[error("[vfs] {operation}: timeout after {duration_ms}ms: {uri}")]
    Timeout {
        /// The URI of the resource.
        uri: String,
        /// The operation that timed out.
        operation: String,
        /// The timeout duration in milliseconds.
        duration_ms: u64,
    },

    /// An I/O error occurred during the operation.
    #[error("[vfs] {operation}: I/O error: {uri}: {source}")]
    Io {
        /// The URI of the resource.
        uri: String,
        /// The operation that encountered an I/O error.
        operation: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// A provider with the given scheme is already registered.
    #[error("[vfs] register: provider scheme '{scheme}' is already registered")]
    DuplicateScheme {
        /// The scheme that is already registered.
        scheme: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn not_found_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::NotFound {
            uri: "vfs://local/home/user/file.txt".to_string(),
            operation: "read".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(msg.contains("read"), "expected operation in message: {msg}");
        assert!(
            msg.contains("vfs://local/home/user/file.txt"),
            "expected URI in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn permission_denied_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::PermissionDenied {
            uri: "vfs://local/etc/shadow".to_string(),
            operation: "write".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(
            msg.contains("write"),
            "expected operation in message: {msg}"
        );
        assert!(
            msg.contains("vfs://local/etc/shadow"),
            "expected URI in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn already_exists_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::AlreadyExists {
            uri: "vfs://local/tmp/existing.txt".to_string(),
            operation: "create".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(
            msg.contains("create"),
            "expected operation in message: {msg}"
        );
        assert!(
            msg.contains("vfs://local/tmp/existing.txt"),
            "expected URI in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn not_a_directory_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::NotADirectory {
            uri: "vfs://local/home/user/file.txt".to_string(),
            operation: "list".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(msg.contains("list"), "expected operation in message: {msg}");
        assert!(
            msg.contains("vfs://local/home/user/file.txt"),
            "expected URI in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn unsupported_operation_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::UnsupportedOperation {
            operation: "watch".to_string(),
            provider: "catalog".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(
            msg.contains("watch"),
            "expected operation in message: {msg}"
        );
        assert!(
            msg.contains("catalog"),
            "expected provider in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn invalid_uri_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::InvalidUri {
            uri: "not-a-valid-uri".to_string(),
            reason: "missing vfs:// scheme prefix".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(
            msg.contains("not-a-valid-uri"),
            "expected URI in message: {msg}"
        );
        assert!(
            msg.contains("missing vfs:// scheme prefix"),
            "expected reason in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn provider_unavailable_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::ProviderUnavailable {
            scheme: "remote-ftp".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(
            msg.contains("remote-ftp"),
            "expected scheme in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn timeout_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::Timeout {
            uri: "vfs://remote/slow-resource".to_string(),
            operation: "read".to_string(),
            duration_ms: 5000,
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(msg.contains("read"), "expected operation in message: {msg}");
        assert!(msg.contains("5000"), "expected duration in message: {msg}");
        assert!(
            msg.contains("vfs://remote/slow-resource"),
            "expected URI in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn io_error_display_starts_with_vfs_prefix_and_includes_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "connection reset");
        let err = VfsError::Io {
            uri: "vfs://local/data/file.bin".to_string(),
            operation: "read_stream".to_string(),
            source: io_err,
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(
            msg.contains("read_stream"),
            "expected operation in message: {msg}"
        );
        assert!(
            msg.contains("vfs://local/data/file.bin"),
            "expected URI in message: {msg}"
        );
        assert!(
            msg.contains("connection reset"),
            "expected source in message: {msg}"
        );
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }

    // Validates: Requirement 1 AC 4, AC 5
    #[test]
    fn duplicate_scheme_display_starts_with_vfs_prefix_and_includes_context() {
        let err = VfsError::DuplicateScheme {
            scheme: "local".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[vfs]"),
            "expected [vfs] prefix, got: {msg}"
        );
        assert!(msg.contains("local"), "expected scheme in message: {msg}");
        assert!(msg.len() <= 200, "message exceeds 200 chars: {}", msg.len());
    }
}
