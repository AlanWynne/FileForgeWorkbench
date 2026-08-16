//! Error types for the idle-processing subsystem.

/// Errors originating from the ff-idle-processing crate.
///
/// All `Display` output follows `[idle-processing] operation: description`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IdleProcessingError {
    /// Attempted to register a work source with a name that already exists.
    #[error("[idle-processing] register: work source '{name}' already registered")]
    DuplicateWorkSource { name: String },

    /// Attempted to unregister a work source that does not exist.
    #[error("[idle-processing] unregister: work source '{name}' not found")]
    WorkSourceNotFound { name: String },

    /// Invalid configuration value.
    #[error("[idle-processing] config: {field} value {value} is invalid — {reason}")]
    InvalidConfig {
        field: String,
        value: String,
        reason: String,
    },

    /// Subscription not found for removal.
    #[error(
        "[idle-processing] unsubscribe: subscription {id} not found for source '{source_name}'"
    )]
    SubscriptionNotFound { source_name: String, id: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_follow_format() {
        let err = IdleProcessingError::DuplicateWorkSource {
            name: "test".into(),
        };
        assert!(err.to_string().starts_with("[idle-processing]"));
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn work_source_not_found_message() {
        let err = IdleProcessingError::WorkSourceNotFound {
            name: "missing".into(),
        };
        assert!(err.to_string().contains("missing"));
    }
}
