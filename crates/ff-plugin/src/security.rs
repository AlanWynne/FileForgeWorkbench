//! Security and sandboxing enforcement.
//!
//! Provides validation functions for enforcing plugin security boundaries:
//! - Configuration namespace scoping
//! - Network access control
//! - Capability ownership verification

use crate::error::PluginError;

/// Validates that a configuration key is within a plugin's allowed namespace.
///
/// This is the public API exposed for property testing and direct use.
///
/// Rules:
/// - Simple keys (no dots prefix) are always allowed
/// - Keys starting with `plugins.{plugin_name}` are allowed
/// - Keys referencing other plugin namespaces are denied
/// - Path traversal attempts (`..`, leading `/` or `\`) are denied
///
/// # Errors
///
/// Returns `PluginError::ConfigAccessDenied` if the key violates namespace rules.
pub fn validate_config_key(plugin_name: &str, key: &str) -> Result<(), PluginError> {
    validate_config_namespace(plugin_name, key)
}

/// Internal implementation of namespace validation.
pub fn validate_config_namespace(plugin_name: &str, key: &str) -> Result<(), PluginError> {
    // Reject path traversal
    if key.contains("..") || key.starts_with('/') || key.starts_with('\\') {
        return Err(PluginError::ConfigAccessDenied {
            plugin: plugin_name.to_string(),
            key: key.to_string(),
        });
    }

    // If key references a plugin namespace, verify it's our own
    if let Some(after_prefix) = key.strip_prefix("plugins.") {
        if !after_prefix.starts_with(plugin_name) {
            return Err(PluginError::ConfigAccessDenied {
                plugin: plugin_name.to_string(),
                key: key.to_string(),
            });
        }
        // Verify it's exactly our namespace (not a prefix attack)
        let remainder = &after_prefix[plugin_name.len()..];
        if !remainder.is_empty() && !remainder.starts_with('.') {
            return Err(PluginError::ConfigAccessDenied {
                plugin: plugin_name.to_string(),
                key: key.to_string(),
            });
        }
    }

    Ok(())
}

/// Checks if a plugin has declared the NetworkAccess capability.
///
/// # Errors
///
/// Returns `PluginError::NetworkAccessDenied` if the plugin has not
/// declared network access.
pub fn check_network_permission(
    plugin_name: &str,
    has_network_capability: bool,
) -> Result<(), PluginError> {
    if has_network_capability {
        Ok(())
    } else {
        ff_logging::log(
            ff_logging::LogLevel::Warn,
            "security",
            &format!("[plugin:{plugin_name}] network access denied: capability not declared"),
        );
        Err(PluginError::NetworkAccessDenied {
            plugin: plugin_name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_key_is_allowed() {
        // Validates: Requirement 7.5
        assert!(validate_config_namespace("my-plugin", "timeout_ms").is_ok());
    }

    #[test]
    fn own_namespace_key_is_allowed() {
        // Validates: Requirement 7.5
        assert!(validate_config_namespace("my-plugin", "plugins.my-plugin.timeout").is_ok());
    }

    #[test]
    fn other_plugin_namespace_is_denied() {
        // Validates: Requirement 7.5
        let result = validate_config_namespace("my-plugin", "plugins.other-plugin.secret");
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginError::ConfigAccessDenied { plugin, key } => {
                assert_eq!(plugin, "my-plugin");
                assert_eq!(key, "plugins.other-plugin.secret");
            }
            _ => panic!("expected ConfigAccessDenied"),
        }
    }

    #[test]
    fn path_traversal_is_denied() {
        // Validates: Requirement 7.5
        assert!(validate_config_namespace("my-plugin", "../secret").is_err());
        assert!(validate_config_namespace("my-plugin", "/etc/passwd").is_err());
        assert!(validate_config_namespace("my-plugin", "\\windows\\system").is_err());
    }

    #[test]
    fn prefix_attack_is_denied() {
        // Validates: Requirement 7.5
        // "my-plugin-evil" should not match "my-plugin" namespace
        assert!(validate_config_namespace("my-plugin", "plugins.my-plugin-evil.setting").is_err());
    }

    #[test]
    fn network_denied_without_capability() {
        // Validates: Requirement 7.3
        let result = check_network_permission("test-plugin", false);
        assert!(result.is_err());
    }

    #[test]
    fn network_allowed_with_capability() {
        // Validates: Requirement 7.3
        let result = check_network_permission("test-plugin", true);
        assert!(result.is_ok());
    }
}
