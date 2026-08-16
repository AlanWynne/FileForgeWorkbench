//! Plugin error types.
//!
//! All errors within the plugin architecture are represented by the
//! `PluginError` enum. Error messages follow the format:
//! `[plugin:{name}] operation: description`

use crate::lifecycle::PluginState;
use crate::version::Version;

/// Errors within the plugin architecture.
///
/// # Variants
///
/// Each variant carries enough context to diagnose the issue: the plugin
/// name, the operation that failed, and a human-readable description.
///
/// # Error Format
///
/// All errors follow the pattern `[plugin:{name}] operation: description`
/// for consistency with the workbench error message standards.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PluginError {
    /// Plugin failed to initialize.
    #[error("[plugin:{plugin}] initialization failed: {description}")]
    InitializationFailed {
        /// Name of the plugin that failed.
        plugin: String,
        /// Human-readable description of what went wrong.
        description: String,
    },

    /// Plugin failed to activate.
    #[error("[plugin:{plugin}] activation failed: {description}")]
    ActivationFailed {
        /// Name of the plugin that failed.
        plugin: String,
        /// Human-readable description of what went wrong.
        description: String,
    },

    /// Plugin failed to deactivate.
    #[error("[plugin:{plugin}] deactivation failed: {description}")]
    DeactivationFailed {
        /// Name of the plugin that failed.
        plugin: String,
        /// Human-readable description of what went wrong.
        description: String,
    },

    /// Plugin failed to shut down.
    #[error("[plugin:{plugin}] shutdown failed: {description}")]
    ShutdownFailed {
        /// Name of the plugin that failed.
        plugin: String,
        /// Human-readable description of what went wrong.
        description: String,
    },

    /// A required dependency could not be satisfied.
    #[error("[plugin:{plugin}] dependency not satisfied: {dependency} ({reason})")]
    DependencyNotSatisfied {
        /// Name of the plugin with the unmet dependency.
        plugin: String,
        /// Name of the missing or incompatible dependency.
        dependency: String,
        /// Reason the dependency cannot be satisfied.
        reason: String,
    },

    /// Plugin requires an incompatible API version.
    #[error("[plugin:{plugin}] incompatible API version: requires {required}, host provides {available}")]
    IncompatibleApiVersion {
        /// Name of the plugin with the version mismatch.
        plugin: String,
        /// Version the plugin requires.
        required: Version,
        /// Version the host provides.
        available: Version,
    },

    /// Plugin not found in registry.
    #[error("[plugin-registry] plugin not found: {name}")]
    PluginNotFound {
        /// Name that was looked up.
        name: String,
    },

    /// Invalid state transition attempted.
    #[error("[plugin:{plugin}] invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition {
        /// Name of the plugin.
        plugin: String,
        /// State the plugin is currently in.
        from: PluginState,
        /// State that was attempted.
        to: PluginState,
    },

    /// Circular dependency detected among plugins.
    #[error("[plugin-registry] circular dependency detected: {cycle:?}")]
    CircularDependency {
        /// Names of plugins forming the cycle.
        cycle: Vec<String>,
    },

    /// Configuration access violation (attempted access outside namespace).
    #[error("[plugin:{plugin}] configuration access denied: {key}")]
    ConfigAccessDenied {
        /// Name of the plugin that violated access rules.
        plugin: String,
        /// Key that was denied.
        key: String,
    },

    /// VFS access error.
    #[error("[plugin:{plugin}] VFS operation failed: {operation} on {uri}: {description}")]
    VfsError {
        /// Name of the plugin.
        plugin: String,
        /// VFS operation that failed (read, write, exists, list_directory).
        operation: String,
        /// URI that was targeted.
        uri: String,
        /// Description of the failure.
        description: String,
    },

    /// Plugin panicked during a lifecycle method.
    #[error("[plugin:{plugin}] panicked during {phase}: {message}")]
    Panicked {
        /// Name of the plugin that panicked.
        plugin: String,
        /// Lifecycle phase during which the panic occurred.
        phase: String,
        /// Panic message extracted from the catch_unwind payload.
        message: String,
    },

    /// Capability registration conflict.
    #[error("[plugin:{plugin}] capability conflict: {description}")]
    CapabilityConflict {
        /// Name of the plugin.
        plugin: String,
        /// Description of the conflict.
        description: String,
    },

    /// Network access denied (plugin did not declare NetworkAccess).
    #[error("[plugin:{plugin}] network access denied: capability not declared")]
    NetworkAccessDenied {
        /// Name of the plugin that attempted unauthorized network access.
        plugin: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_failed_error_format_matches_spec() {
        // Validates: Requirement 1.5
        let err = PluginError::InitializationFailed {
            plugin: "my-plugin".to_string(),
            description: "missing config".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[plugin:my-plugin] initialization failed: missing config"
        );
    }

    #[test]
    fn activation_failed_error_format_matches_spec() {
        // Validates: Requirement 1.5
        let err = PluginError::ActivationFailed {
            plugin: "viewer".to_string(),
            description: "port in use".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[plugin:viewer] activation failed: port in use"
        );
    }

    #[test]
    fn deactivation_failed_error_format_matches_spec() {
        // Validates: Requirement 1.5
        let err = PluginError::DeactivationFailed {
            plugin: "sql-connector".to_string(),
            description: "connection pool busy".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[plugin:sql-connector] deactivation failed: connection pool busy"
        );
    }

    #[test]
    fn shutdown_failed_error_format_matches_spec() {
        // Validates: Requirement 1.5
        let err = PluginError::ShutdownFailed {
            plugin: "theme".to_string(),
            description: "timeout".to_string(),
        };
        assert_eq!(err.to_string(), "[plugin:theme] shutdown failed: timeout");
    }

    #[test]
    fn dependency_not_satisfied_error_includes_details() {
        // Validates: Requirement 1.5
        let err = PluginError::DependencyNotSatisfied {
            plugin: "editor-ext".to_string(),
            dependency: "language-service".to_string(),
            reason: "not found".to_string(),
        };
        assert!(err.to_string().contains("[plugin:editor-ext]"));
        assert!(err.to_string().contains("language-service"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn incompatible_api_version_error_shows_versions() {
        // Validates: Requirement 1.5
        let err = PluginError::IncompatibleApiVersion {
            plugin: "old-plugin".to_string(),
            required: Version::new(2, 0, 0),
            available: Version::new(1, 0, 0),
        };
        let msg = err.to_string();
        assert!(msg.contains("2.0.0"));
        assert!(msg.contains("1.0.0"));
    }

    #[test]
    fn plugin_not_found_error_format() {
        // Validates: Requirement 1.5
        let err = PluginError::PluginNotFound {
            name: "nonexistent".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[plugin-registry] plugin not found: nonexistent"
        );
    }

    #[test]
    fn invalid_state_transition_error_shows_states() {
        // Validates: Requirement 1.5
        let err = PluginError::InvalidStateTransition {
            plugin: "my-plugin".to_string(),
            from: PluginState::Active,
            to: PluginState::Discovered,
        };
        let msg = err.to_string();
        assert!(msg.contains("Active"));
        assert!(msg.contains("Discovered"));
    }

    #[test]
    fn circular_dependency_error_lists_cycle() {
        // Validates: Requirement 1.5
        let err = PluginError::CircularDependency {
            cycle: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let msg = err.to_string();
        assert!(msg.contains("circular dependency"));
        assert!(msg.contains("a"));
        assert!(msg.contains("b"));
        assert!(msg.contains("c"));
    }

    #[test]
    fn config_access_denied_error_format() {
        // Validates: Requirement 7.7
        let err = PluginError::ConfigAccessDenied {
            plugin: "bad-plugin".to_string(),
            key: "plugins.other.secret".to_string(),
        };
        assert!(err.to_string().contains("[plugin:bad-plugin]"));
        assert!(err.to_string().contains("plugins.other.secret"));
    }

    #[test]
    fn network_access_denied_error_format() {
        // Validates: Requirement 7.3
        let err = PluginError::NetworkAccessDenied {
            plugin: "sketchy".to_string(),
        };
        assert!(err.to_string().contains("[plugin:sketchy]"));
        assert!(err.to_string().contains("network access denied"));
    }
}
