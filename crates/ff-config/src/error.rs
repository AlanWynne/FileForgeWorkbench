//! Configuration error types.
//!
//! Defines `ConfigError` — the unified error enum for all configuration
//! operations — following the `[config] operation: description` format.

use std::path::PathBuf;

/// The type of a configuration value, used in type mismatch errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    /// A UTF-8 string.
    String,
    /// A signed 64-bit integer.
    Integer,
    /// A 64-bit floating-point number.
    Float,
    /// A boolean.
    Boolean,
    /// An ordered array.
    Array,
    /// A nested table.
    Table,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Integer => write!(f, "integer"),
            Self::Float => write!(f, "float"),
            Self::Boolean => write!(f, "boolean"),
            Self::Array => write!(f, "array"),
            Self::Table => write!(f, "table"),
        }
    }
}

/// Errors produced by the configuration system.
///
/// All variants follow the `[config] operation: description` message format.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A TOML file failed to parse.
    #[error("[config] parse: {path}: {details}")]
    ParseError {
        /// Path to the file that failed to parse.
        path: PathBuf,
        /// Human-readable parse error details.
        details: String,
    },

    /// A requested configuration key has no schema entry and is not defined
    /// in any layer.
    #[error("[config] lookup: undefined key \"{key}\"")]
    UndefinedKey {
        /// The key that was not found.
        key: String,
    },

    /// A typed getter was called but the stored value has a different type.
    #[error("[config] type mismatch: key \"{key}\" expected {expected}, found {found}")]
    TypeMismatch {
        /// The configuration key.
        key: String,
        /// The type requested by the caller.
        expected: ValueType,
        /// The actual type stored.
        found: ValueType,
    },

    /// A value failed schema validation constraints.
    #[error("[config] validation: key \"{key}\": {reason}")]
    ValidationFailed {
        /// The configuration key.
        key: String,
        /// Human-readable description of the constraint violation.
        reason: String,
    },

    /// A plugin attempted to access a key outside its namespace.
    #[error("[config] namespace violation: plugin \"{plugin}\" cannot access \"{key}\"")]
    NamespaceViolation {
        /// The plugin that attempted the access.
        plugin: String,
        /// The key the plugin tried to access.
        key: String,
    },

    /// A plugin name does not conform to naming rules.
    #[error("[config] invalid plugin name: \"{name}\"")]
    InvalidPluginName {
        /// The invalid plugin name.
        name: String,
    },

    /// A plugin attempted to register keys in a reserved core namespace.
    #[error(
        "[config] reserved namespace: plugin \"{plugin}\" cannot use namespace \"{namespace}\""
    )]
    ReservedNamespace {
        /// The plugin that attempted the registration.
        plugin: String,
        /// The reserved namespace.
        namespace: String,
    },

    /// A requested profile was not found.
    #[error("[config] profile: profile \"{name}\" not found")]
    ProfileNotFound {
        /// The profile name that was not found.
        name: String,
    },

    /// The file watcher encountered an error.
    #[error("[config] watcher: {details}")]
    WatcherError {
        /// Human-readable watcher error details.
        details: String,
    },

    /// An I/O error occurred while reading or writing configuration.
    #[error("[config] io: {0}")]
    Io(#[from] std::io::Error),

    /// A schema registration conflict occurred.
    #[error("[config] schema conflict: key \"{key}\": {details}")]
    SchemaConflict {
        /// The conflicting key.
        key: String,
        /// Description of the conflict.
        details: String,
    },

    /// An `.editorconfig` file failed to parse.
    #[error("[config] editorconfig parse: {path}: {details}")]
    EditorConfigParseError {
        /// Path to the `.editorconfig` file.
        path: PathBuf,
        /// Human-readable parse error details.
        details: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Helper: asserts that an error message starts with "[config] ".
    fn assert_config_prefix(error: &ConfigError) {
        let msg = error.to_string();
        assert!(
            msg.starts_with("[config] "),
            "Error message must start with '[config] ', got: {msg}"
        );
    }

    // Validates: Requirement 8 (Error Message Standards)
    #[test]
    fn parse_error_display_follows_config_prefix_pattern() {
        let err = ConfigError::ParseError {
            path: PathBuf::from("/home/user/config.toml"),
            details: "expected '=' after key".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] parse: /home/user/config.toml: expected '=' after key"
        );
    }

    // Validates: Requirement 2.6
    #[test]
    fn undefined_key_display_follows_config_prefix_pattern() {
        let err = ConfigError::UndefinedKey {
            key: "editor.unknown_setting".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] lookup: undefined key \"editor.unknown_setting\""
        );
    }

    // Validates: Requirement 7.5
    #[test]
    fn type_mismatch_display_follows_config_prefix_pattern() {
        let err = ConfigError::TypeMismatch {
            key: "editor.tab_size".to_string(),
            expected: ValueType::Integer,
            found: ValueType::String,
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] type mismatch: key \"editor.tab_size\" expected integer, found string"
        );
    }

    // Validates: Requirement 7.6, Requirement 9.4
    #[test]
    fn validation_failed_display_follows_config_prefix_pattern() {
        let err = ConfigError::ValidationFailed {
            key: "editor.tab_size".to_string(),
            reason: "value 0 is below minimum 1".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] validation: key \"editor.tab_size\": value 0 is below minimum 1"
        );
    }

    // Validates: Requirement 8.3
    #[test]
    fn namespace_violation_display_follows_config_prefix_pattern() {
        let err = ConfigError::NamespaceViolation {
            plugin: "sql-viewer".to_string(),
            key: "editor.tab_size".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] namespace violation: plugin \"sql-viewer\" cannot access \"editor.tab_size\""
        );
    }

    // Validates: Requirement 8.1
    #[test]
    fn invalid_plugin_name_display_follows_config_prefix_pattern() {
        let err = ConfigError::InvalidPluginName {
            name: "My Plugin!!".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] invalid plugin name: \"My Plugin!!\""
        );
    }

    // Validates: Requirement 8.7
    #[test]
    fn reserved_namespace_display_follows_config_prefix_pattern() {
        let err = ConfigError::ReservedNamespace {
            plugin: "evil-plugin".to_string(),
            namespace: "logging".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] reserved namespace: plugin \"evil-plugin\" cannot use namespace \"logging\""
        );
    }

    // Validates: Requirement 4.6
    #[test]
    fn profile_not_found_display_follows_config_prefix_pattern() {
        let err = ConfigError::ProfileNotFound {
            name: "mainframe".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] profile: profile \"mainframe\" not found"
        );
    }

    #[test]
    fn watcher_error_display_follows_config_prefix_pattern() {
        let err = ConfigError::WatcherError {
            details: "inotify limit reached".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(err.to_string(), "[config] watcher: inotify limit reached");
    }

    // Validates: Requirement 5.7
    #[test]
    fn io_error_display_follows_config_prefix_pattern() {
        let err = ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "access denied",
        ));
        assert_config_prefix(&err);
        assert_eq!(err.to_string(), "[config] io: access denied");
    }

    #[test]
    fn schema_conflict_display_follows_config_prefix_pattern() {
        let err = ConfigError::SchemaConflict {
            key: "editor.tab_size".to_string(),
            details: "already registered as integer, cannot re-register as string".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] schema conflict: key \"editor.tab_size\": already registered as integer, cannot re-register as string"
        );
    }

    // Validates: Requirement 6.6
    #[test]
    fn editorconfig_parse_error_display_follows_config_prefix_pattern() {
        let err = ConfigError::EditorConfigParseError {
            path: PathBuf::from("/project/.editorconfig"),
            details: "invalid glob pattern".to_string(),
        };
        assert_config_prefix(&err);
        assert_eq!(
            err.to_string(),
            "[config] editorconfig parse: /project/.editorconfig: invalid glob pattern"
        );
    }

    // Validates: Requirement 1.4 — ValueType Display
    #[test]
    fn value_type_display_produces_lowercase_names() {
        assert_eq!(ValueType::String.to_string(), "string");
        assert_eq!(ValueType::Integer.to_string(), "integer");
        assert_eq!(ValueType::Float.to_string(), "float");
        assert_eq!(ValueType::Boolean.to_string(), "boolean");
        assert_eq!(ValueType::Array.to_string(), "array");
        assert_eq!(ValueType::Table.to_string(), "table");
    }

    #[test]
    fn value_type_equality_and_copy() {
        let a = ValueType::Integer;
        let b = a; // Copy
        assert_eq!(a, b);
        assert_ne!(ValueType::String, ValueType::Integer);
    }

    #[test]
    fn all_error_variants_start_with_config_prefix() {
        let errors: Vec<ConfigError> = vec![
            ConfigError::ParseError {
                path: PathBuf::from("test.toml"),
                details: "error".to_string(),
            },
            ConfigError::UndefinedKey {
                key: "k".to_string(),
            },
            ConfigError::TypeMismatch {
                key: "k".to_string(),
                expected: ValueType::String,
                found: ValueType::Integer,
            },
            ConfigError::ValidationFailed {
                key: "k".to_string(),
                reason: "bad".to_string(),
            },
            ConfigError::NamespaceViolation {
                plugin: "p".to_string(),
                key: "k".to_string(),
            },
            ConfigError::InvalidPluginName {
                name: "x".to_string(),
            },
            ConfigError::ReservedNamespace {
                plugin: "p".to_string(),
                namespace: "n".to_string(),
            },
            ConfigError::ProfileNotFound {
                name: "prof".to_string(),
            },
            ConfigError::WatcherError {
                details: "err".to_string(),
            },
            ConfigError::Io(std::io::Error::new(std::io::ErrorKind::Other, "io")),
            ConfigError::SchemaConflict {
                key: "k".to_string(),
                details: "conflict".to_string(),
            },
            ConfigError::EditorConfigParseError {
                path: PathBuf::from("f"),
                details: "parse".to_string(),
            },
        ];

        for error in &errors {
            assert_config_prefix(error);
        }
    }
}
