//! Connector error types and error mapping.
//!
//! Defines `ConnectorError` — a structured error type that maps provider-specific
//! errors to common VFS error categories with additional diagnostic context.
//! Formatted per Error Message Standards (Req 8):
//! `[connector:{scheme}] {operation}: {description}`

use std::fmt;

/// Errors originating from the connector extensibility framework.
///
/// Each variant carries enough context to produce a diagnostic message following
/// the workbench error format: `[connector:{scheme}] {operation}: {description}`.
///
/// Addresses: Requirement 7, all acceptance criteria
#[derive(Debug)]
#[non_exhaustive]
pub enum ConnectorError {
    /// Connector is not in a connected state.
    NotConnected {
        /// The URI scheme of the connector.
        scheme: String,
        /// The VFS operation that was attempted.
        operation: String,
    },

    /// Authentication failed (credentials invalid or expired).
    AuthenticationFailed {
        /// The URI scheme of the connector.
        scheme: String,
        /// Human-readable description of the authentication failure.
        message: String,
    },

    /// Permission denied by the remote service.
    PermissionDenied {
        /// The URI scheme of the connector.
        scheme: String,
        /// The VFS operation that was attempted.
        operation: String,
        /// The resource URI that was denied.
        uri: String,
    },

    /// Resource does not exist on the remote service.
    ResourceNotFound {
        /// The URI scheme of the connector.
        scheme: String,
        /// The VFS operation that was attempted.
        operation: String,
        /// The resource URI that was not found.
        uri: String,
    },

    /// Resource already exists (e.g., create on existing).
    ResourceAlreadyExists {
        /// The URI scheme of the connector.
        scheme: String,
        /// The VFS operation that was attempted.
        operation: String,
        /// The resource URI that already exists.
        uri: String,
    },

    /// Operation timed out.
    Timeout {
        /// The URI scheme of the connector.
        scheme: String,
        /// The VFS operation that was attempted.
        operation: String,
        /// Elapsed time in milliseconds before timeout.
        elapsed_ms: u64,
    },

    /// Network-level error (connection refused, DNS failure, etc.).
    NetworkError {
        /// The URI scheme of the connector.
        scheme: String,
        /// The VFS operation that was attempted.
        operation: String,
        /// Human-readable description of the network error.
        message: String,
    },

    /// Operation not supported by this connector.
    UnsupportedOperation {
        /// The URI scheme of the connector.
        scheme: String,
        /// The VFS operation that was attempted.
        operation: String,
        /// Human-readable message explaining the limitation.
        message: String,
    },

    /// Registration validation failed.
    RegistrationFailed {
        /// Human-readable description of why registration failed.
        message: String,
    },

    /// Provider-specific error that doesn't fit common categories.
    ProviderSpecific {
        /// The URI scheme of the connector.
        scheme: String,
        /// The VFS operation that was attempted.
        operation: String,
        /// Human-readable description of the error.
        message: String,
        /// The underlying provider-specific error, if available.
        #[allow(dead_code)]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Internal error (bug in connector implementation).
    Internal {
        /// The URI scheme of the connector.
        scheme: String,
        /// Human-readable description of the internal error.
        message: String,
    },
}

impl fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected { scheme, operation } => {
                write!(f, "[connector:{scheme}] {operation}: not connected")
            }
            Self::AuthenticationFailed { scheme, message } => {
                write!(f, "[connector:{scheme}] authenticate: {message}")
            }
            Self::PermissionDenied {
                scheme,
                operation,
                uri,
            } => {
                write!(
                    f,
                    "[connector:{scheme}] {operation}: permission denied on {uri}"
                )
            }
            Self::ResourceNotFound {
                scheme,
                operation,
                uri,
            } => {
                write!(f, "[connector:{scheme}] {operation}: not found: {uri}")
            }
            Self::ResourceAlreadyExists {
                scheme,
                operation,
                uri,
            } => {
                write!(f, "[connector:{scheme}] {operation}: already exists: {uri}")
            }
            Self::Timeout {
                scheme,
                operation,
                elapsed_ms,
            } => {
                write!(
                    f,
                    "[connector:{scheme}] {operation}: timeout after {elapsed_ms}ms"
                )
            }
            Self::NetworkError {
                scheme,
                operation,
                message,
            } => {
                write!(
                    f,
                    "[connector:{scheme}] {operation}: network error: {message}"
                )
            }
            Self::UnsupportedOperation {
                scheme,
                operation,
                message,
            } => {
                write!(f, "[connector:{scheme}] {operation}: {message}")
            }
            Self::RegistrationFailed { message } => {
                write!(f, "[connector-registry] register: {message}")
            }
            Self::ProviderSpecific {
                scheme,
                operation,
                message,
                ..
            } => {
                write!(f, "[connector:{scheme}] {operation}: {message}")
            }
            Self::Internal { scheme, message } => {
                write!(f, "[connector:{scheme}] internal: {message}")
            }
        }
    }
}

impl std::error::Error for ConnectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ProviderSpecific {
                source: Some(src), ..
            } => Some(src.as_ref()),
            _ => None,
        }
    }
}

impl ConnectorError {
    /// Classifies whether the error is retryable.
    ///
    /// Timeout and NetworkError are retryable.
    /// All other variants are not retryable.
    ///
    /// Note: NotConnected retryability depends on the connector's RetryPolicy,
    /// which is checked separately via `should_reconnect()`.
    ///
    /// Addresses: Requirement 7 AC 2
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout { .. } | Self::NetworkError { .. })
    }

    /// Returns whether this error indicates the connector should attempt reconnection.
    ///
    /// Returns true for NotConnected, Timeout, and NetworkError — errors that
    /// suggest a connection issue that might resolve with a reconnection attempt.
    ///
    /// Addresses: Requirement 7 AC 2
    pub fn should_reconnect(&self) -> bool {
        matches!(
            self,
            Self::NotConnected { .. } | Self::Timeout { .. } | Self::NetworkError { .. }
        )
    }

    /// Create a connector error from an I/O error with full context.
    ///
    /// Maps standard I/O error kinds to appropriate connector error categories:
    /// - `PermissionDenied` → `ConnectorError::PermissionDenied`
    /// - `NotFound` → `ConnectorError::ResourceNotFound`
    /// - `TimedOut` → `ConnectorError::Timeout`
    /// - `AlreadyExists` → `ConnectorError::ResourceAlreadyExists`
    /// - Other → `ConnectorError::ProviderSpecific`
    ///
    /// Addresses: Requirement 7 AC 3
    pub fn from_io_error(scheme: &str, operation: &str, uri: &str, source: std::io::Error) -> Self {
        match source.kind() {
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied {
                scheme: scheme.to_string(),
                operation: operation.to_string(),
                uri: uri.to_string(),
            },
            std::io::ErrorKind::NotFound => Self::ResourceNotFound {
                scheme: scheme.to_string(),
                operation: operation.to_string(),
                uri: uri.to_string(),
            },
            std::io::ErrorKind::TimedOut => Self::Timeout {
                scheme: scheme.to_string(),
                operation: operation.to_string(),
                elapsed_ms: 0,
            },
            std::io::ErrorKind::AlreadyExists => Self::ResourceAlreadyExists {
                scheme: scheme.to_string(),
                operation: operation.to_string(),
                uri: uri.to_string(),
            },
            _ => Self::ProviderSpecific {
                scheme: scheme.to_string(),
                operation: operation.to_string(),
                message: source.to_string(),
                source: Some(Box::new(source)),
            },
        }
    }
}

// Ensure ConnectorError is Send + Sync for use across thread boundaries.
fn _assert_send_sync() {
    fn _assert<T: Send + Sync>() {}
    _assert::<ConnectorError>();
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7 AC 6
    #[test]
    fn display_format_not_connected() {
        let err = ConnectorError::NotConnected {
            scheme: "ftp".to_string(),
            operation: "read".to_string(),
        };
        assert_eq!(err.to_string(), "[connector:ftp] read: not connected");
    }

    // Validates: Requirement 7 AC 6
    #[test]
    fn display_format_authentication_failed() {
        let err = ConnectorError::AuthenticationFailed {
            scheme: "sftp".to_string(),
            message: "invalid key".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[connector:sftp] authenticate: invalid key"
        );
    }

    // Validates: Requirement 7 AC 6
    #[test]
    fn display_format_permission_denied() {
        let err = ConnectorError::PermissionDenied {
            scheme: "ftp".to_string(),
            operation: "write".to_string(),
            uri: "/protected/file.txt".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[connector:ftp] write: permission denied on /protected/file.txt"
        );
    }

    // Validates: Requirement 7 AC 6
    #[test]
    fn display_format_registration_failed() {
        let err = ConnectorError::RegistrationFailed {
            message: "duplicate scheme".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[connector-registry] register: duplicate scheme"
        );
    }

    // Validates: Requirement 7 AC 2
    #[test]
    fn timeout_is_retryable() {
        let err = ConnectorError::Timeout {
            scheme: "ftp".to_string(),
            operation: "connect".to_string(),
            elapsed_ms: 5000,
        };
        assert!(err.is_retryable());
    }

    // Validates: Requirement 7 AC 2
    #[test]
    fn network_error_is_retryable() {
        let err = ConnectorError::NetworkError {
            scheme: "ftp".to_string(),
            operation: "connect".to_string(),
            message: "connection refused".to_string(),
        };
        assert!(err.is_retryable());
    }

    // Validates: Requirement 7 AC 2
    #[test]
    fn authentication_failed_is_not_retryable() {
        let err = ConnectorError::AuthenticationFailed {
            scheme: "sftp".to_string(),
            message: "bad password".to_string(),
        };
        assert!(!err.is_retryable());
    }

    // Validates: Requirement 7 AC 2
    #[test]
    fn permission_denied_is_not_retryable() {
        let err = ConnectorError::PermissionDenied {
            scheme: "ftp".to_string(),
            operation: "write".to_string(),
            uri: "/file.txt".to_string(),
        };
        assert!(!err.is_retryable());
    }

    // Validates: Requirement 7 AC 2
    #[test]
    fn not_connected_should_reconnect() {
        let err = ConnectorError::NotConnected {
            scheme: "ftp".to_string(),
            operation: "read".to_string(),
        };
        assert!(err.should_reconnect());
        assert!(!err.is_retryable());
    }

    // Validates: Requirement 7 AC 3
    #[test]
    fn from_io_error_permission_denied_maps_correctly() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = ConnectorError::from_io_error("ftp", "write", "/file.txt", io_err);
        assert!(matches!(err, ConnectorError::PermissionDenied { .. }));
    }

    // Validates: Requirement 7 AC 3
    #[test]
    fn from_io_error_not_found_maps_correctly() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let err = ConnectorError::from_io_error("sftp", "read", "/missing.txt", io_err);
        assert!(matches!(err, ConnectorError::ResourceNotFound { .. }));
    }

    // Validates: Requirement 7 AC 3
    #[test]
    fn from_io_error_timed_out_maps_correctly() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out");
        let err = ConnectorError::from_io_error("ftp", "connect", "", io_err);
        assert!(matches!(err, ConnectorError::Timeout { .. }));
    }

    // Validates: Requirement 7 AC 5
    #[test]
    fn provider_specific_error_has_source_chain() {
        let inner = std::io::Error::other("underlying cause");
        let err = ConnectorError::ProviderSpecific {
            scheme: "zos".to_string(),
            operation: "submit_job".to_string(),
            message: "JES error".to_string(),
            source: Some(Box::new(inner)),
        };
        assert!(std::error::Error::source(&err).is_some());
    }

    // Validates: Requirement 7 AC 5
    #[test]
    fn non_provider_specific_errors_have_no_source() {
        let err = ConnectorError::NotConnected {
            scheme: "ftp".to_string(),
            operation: "read".to_string(),
        };
        assert!(std::error::Error::source(&err).is_none());
    }

    // Validates: Requirement 7 AC 6
    #[test]
    fn display_messages_are_within_200_chars() {
        let errors = vec![
            ConnectorError::NotConnected {
                scheme: "ftp".to_string(),
                operation: "read".to_string(),
            },
            ConnectorError::Timeout {
                scheme: "sftp".to_string(),
                operation: "connect".to_string(),
                elapsed_ms: 30000,
            },
            ConnectorError::NetworkError {
                scheme: "cloud".to_string(),
                operation: "list".to_string(),
                message: "DNS resolution failed".to_string(),
            },
            ConnectorError::RegistrationFailed {
                message: "missing required capabilities: Read, List".to_string(),
            },
        ];

        for err in &errors {
            let display = err.to_string();
            assert!(
                display.len() <= 200,
                "error display too long ({} chars): {}",
                display.len(),
                display
            );
        }
    }
}
