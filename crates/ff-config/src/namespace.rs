//! Namespace management.
//!
//! Enforces plugin namespace isolation and prevents plugins from registering
//! keys that collide with reserved core namespaces (`logging`, `editor`,
//! `theme`, `vfs`, `commands`, `layout`, `core`, `_session`).

use crate::error::ConfigError;

/// Reserved core namespaces that plugins cannot register under.
///
/// These namespaces are exclusively owned by the platform core and
/// attempting to register a plugin with any of these names will be rejected.
pub const RESERVED_NAMESPACES: &[&str] = &[
    "logging", "editor", "theme", "vfs", "commands", "layout", "core", "_session",
];

/// Maximum allowed length for a plugin name.
const MAX_PLUGIN_NAME_LENGTH: usize = 64;

/// Validate a plugin name against naming rules.
///
/// A valid plugin name must:
/// - Be 1–64 characters long
/// - Contain only lowercase ASCII letters (`a-z`), digits (`0-9`), and hyphens (`-`)
/// - Not start or end with a hyphen
///
/// # Errors
///
/// Returns `ConfigError::InvalidPluginName` if the name does not conform.
pub fn validate_plugin_name(name: &str) -> Result<(), ConfigError> {
    if name.is_empty() || name.len() > MAX_PLUGIN_NAME_LENGTH {
        return Err(ConfigError::InvalidPluginName {
            name: name.to_string(),
        });
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(ConfigError::InvalidPluginName {
            name: name.to_string(),
        });
    }

    let all_valid = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !all_valid {
        return Err(ConfigError::InvalidPluginName {
            name: name.to_string(),
        });
    }

    Ok(())
}

/// Check whether a namespace is reserved for core platform use.
///
/// Returns `true` if the given namespace matches one of the reserved
/// core namespaces.
pub fn is_reserved_namespace(namespace: &str) -> bool {
    RESERVED_NAMESPACES.contains(&namespace)
}

/// Create the full namespace prefix for a plugin.
///
/// For a plugin named `"sql-viewer"`, this returns `"plugins.sql-viewer."`.
pub fn plugin_namespace_prefix(plugin_name: &str) -> String {
    format!("plugins.{plugin_name}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────
    // validate_plugin_name
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.1
    #[test]
    fn valid_plugin_names_accepted() {
        assert!(validate_plugin_name("sql-viewer").is_ok());
        assert!(validate_plugin_name("my-plugin").is_ok());
        assert!(validate_plugin_name("plugin123").is_ok());
        assert!(validate_plugin_name("a").is_ok());
        assert!(validate_plugin_name("abc-def-ghi").is_ok());
        assert!(validate_plugin_name("a1b2c3").is_ok());
        assert!(validate_plugin_name("x").is_ok());
    }

    // Validates: Requirement 8.1
    #[test]
    fn empty_plugin_name_rejected() {
        assert!(validate_plugin_name("").is_err());
    }

    // Validates: Requirement 8.1
    #[test]
    fn too_long_plugin_name_rejected() {
        let long_name = "a".repeat(65);
        assert!(validate_plugin_name(&long_name).is_err());
    }

    // Validates: Requirement 8.1
    #[test]
    fn max_length_plugin_name_accepted() {
        let max_name = "a".repeat(64);
        assert!(validate_plugin_name(&max_name).is_ok());
    }

    // Validates: Requirement 8.1
    #[test]
    fn uppercase_plugin_name_rejected() {
        assert!(validate_plugin_name("MyPlugin").is_err());
        assert!(validate_plugin_name("SQL-viewer").is_err());
        assert!(validate_plugin_name("pluginA").is_err());
    }

    // Validates: Requirement 8.1
    #[test]
    fn spaces_in_plugin_name_rejected() {
        assert!(validate_plugin_name("my plugin").is_err());
        assert!(validate_plugin_name(" leading").is_err());
        assert!(validate_plugin_name("trailing ").is_err());
    }

    // Validates: Requirement 8.1
    #[test]
    fn special_chars_in_plugin_name_rejected() {
        assert!(validate_plugin_name("my_plugin").is_err());
        assert!(validate_plugin_name("my.plugin").is_err());
        assert!(validate_plugin_name("my@plugin").is_err());
        assert!(validate_plugin_name("my/plugin").is_err());
        assert!(validate_plugin_name("my!plugin").is_err());
    }

    // Validates: Requirement 8.1
    #[test]
    fn hyphen_at_start_rejected() {
        assert!(validate_plugin_name("-leading").is_err());
    }

    // Validates: Requirement 8.1
    #[test]
    fn hyphen_at_end_rejected() {
        assert!(validate_plugin_name("trailing-").is_err());
    }

    // ──────────────────────────────────────────────────────────────────
    // is_reserved_namespace
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.7
    #[test]
    fn reserved_namespaces_detected() {
        assert!(is_reserved_namespace("logging"));
        assert!(is_reserved_namespace("editor"));
        assert!(is_reserved_namespace("theme"));
        assert!(is_reserved_namespace("vfs"));
        assert!(is_reserved_namespace("commands"));
        assert!(is_reserved_namespace("layout"));
        assert!(is_reserved_namespace("core"));
        assert!(is_reserved_namespace("_session"));
    }

    // Validates: Requirement 8.7
    #[test]
    fn non_reserved_namespaces_not_detected() {
        assert!(!is_reserved_namespace("sql-viewer"));
        assert!(!is_reserved_namespace("my-plugin"));
        assert!(!is_reserved_namespace("custom"));
        assert!(!is_reserved_namespace("plugins"));
        assert!(!is_reserved_namespace("editor-helper")); // not exact match
    }

    // ──────────────────────────────────────────────────────────────────
    // plugin_namespace_prefix
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.1
    #[test]
    fn plugin_namespace_prefix_format() {
        assert_eq!(plugin_namespace_prefix("sql-viewer"), "plugins.sql-viewer.");
        assert_eq!(plugin_namespace_prefix("my-plugin"), "plugins.my-plugin.");
    }
}
