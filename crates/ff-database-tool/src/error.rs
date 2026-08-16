//! Error types for the database tool.

/// Errors originating from the ff-database-tool crate.
///
/// All `Display` output follows `[db] operation: description`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DatabaseToolError {
    /// Connection to the database failed.
    #[error("[db] connect: connection failed for '{connection_name}': {reason}")]
    ConnectionFailed {
        connection_name: String,
        reason: String,
    },

    /// Query execution failed.
    #[error("[db] execute: query execution failed: {reason}")]
    QueryExecutionError { reason: String },

    /// Query timed out.
    #[error("[db] execute: query timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Authentication failed.
    #[error("[db] connect: authentication failed for '{connection_name}'")]
    AuthenticationFailed { connection_name: String },

    /// Driver not found in registry.
    #[error("[db] driver: driver '{driver_name}' not found in registry")]
    DriverNotFound { driver_name: String },

    /// Metadata retrieval failed.
    #[error("[db] metadata: failed to retrieve {object_type}: {reason}")]
    MetadataError { object_type: String, reason: String },

    /// Data transfer operation failed.
    #[error("[db] transfer: data transfer failed: {reason}")]
    DataTransferError { reason: String },

    /// Operation was cancelled.
    #[error("[db] {operation}: operation cancelled")]
    Cancelled { operation: String },

    /// Connection pool exhausted.
    #[error("[db] pool: connection pool exhausted for '{connection_name}'")]
    PoolExhausted { connection_name: String },

    /// SSH tunnel establishment failed.
    #[error("[db] ssh: tunnel failed for '{host}': {reason}")]
    SshTunnelFailed { host: String, reason: String },

    /// Credential storage error.
    #[error("[db] credentials: {reason}")]
    CredentialError { reason: String },

    /// Invalid state transition.
    #[error("[db] state: invalid transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// I/O error.
    #[error("[db] io: {reason}")]
    Io { reason: String },

    /// Serialization error.
    #[error("[db] serialize: {reason}")]
    Serialization { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_follow_format() {
        let err = DatabaseToolError::ConnectionFailed {
            connection_name: "prod-db".into(),
            reason: "refused".into(),
        };
        assert!(err.to_string().starts_with("[db]"));
        assert!(err.to_string().contains("prod-db"));
    }

    #[test]
    fn driver_not_found_message() {
        let err = DatabaseToolError::DriverNotFound {
            driver_name: "oracle".into(),
        };
        assert!(err.to_string().contains("oracle"));
    }

    #[test]
    fn timeout_message_includes_duration() {
        let err = DatabaseToolError::Timeout { timeout_ms: 5000 };
        assert!(err.to_string().contains("5000ms"));
    }
}
