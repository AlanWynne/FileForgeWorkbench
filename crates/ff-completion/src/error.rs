//! Error types for the completion subsystem.

/// Errors produced by the completion subsystem.
///
/// All error messages follow the format `[completion] operation: description`
/// for consistency with the workspace error message standards.
#[derive(Debug, thiserror::Error)]
pub enum CompletionError {
    /// A provider failed to produce candidates.
    #[error("[completion] provider '{provider_id}': {reason}")]
    ProviderFailed {
        /// The identifier of the provider that failed.
        provider_id: String,
        /// Description of what went wrong.
        reason: String,
    },

    /// VFS directory listing failed during file path completion.
    #[error("[completion] vfs_listing '{path}': {reason}")]
    VfsListingFailed {
        /// The path being listed.
        path: String,
        /// Description of the failure.
        reason: String,
    },

    /// Configuration value is invalid (wrong type, out of range).
    #[error("[completion] config '{key}': invalid value '{value}', using default '{default}'")]
    InvalidConfig {
        /// The configuration key.
        key: String,
        /// The invalid value encountered.
        value: String,
        /// The default value being used instead.
        default: String,
    },

    /// Provider registration failed (duplicate ID).
    #[error("[completion] register_provider: provider '{provider_id}' already registered")]
    DuplicateProvider {
        /// The provider ID that was duplicated.
        provider_id: String,
    },

    /// Internal error — should not occur in normal operation.
    #[error("[completion] internal: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 22.2 — error display output
    #[test]
    fn provider_failed_displays_correctly() {
        let err = CompletionError::ProviderFailed {
            provider_id: "command_name".to_string(),
            reason: "registry unavailable".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[completion] provider 'command_name': registry unavailable"
        );
    }

    #[test]
    fn vfs_listing_failed_displays_correctly() {
        let err = CompletionError::VfsListingFailed {
            path: "/home/user".to_string(),
            reason: "permission denied".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[completion] vfs_listing '/home/user': permission denied"
        );
    }

    #[test]
    fn invalid_config_displays_correctly() {
        let err = CompletionError::InvalidConfig {
            key: "completion.popup_max_items".to_string(),
            value: "999".to_string(),
            default: "10".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[completion] config 'completion.popup_max_items': invalid value '999', using default '10'"
        );
    }

    #[test]
    fn duplicate_provider_displays_correctly() {
        let err = CompletionError::DuplicateProvider {
            provider_id: "my_plugin".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[completion] register_provider: provider 'my_plugin' already registered"
        );
    }

    #[test]
    fn internal_error_displays_correctly() {
        let err = CompletionError::Internal("unexpected state".to_string());
        assert_eq!(err.to_string(), "[completion] internal: unexpected state");
    }
}
