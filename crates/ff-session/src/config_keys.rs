//! Session configuration key definitions — typed accessors for all keys
//! under the `[session]` TOML namespace.
//!
//! Addresses: Requirement 2 (Configuration Loading Orchestration),
//!            Requirement 12 (Startup Configuration Keys)

use std::path::PathBuf;

/// All session-related configuration fields registered under `[session]`.
///
/// These keys are registered with the `configuration-system` schema API
/// so that validation, layered merging, and hot-reload apply to them.
///
/// # Configuration Keys
///
/// | Key | Type | Default | Range |
/// |-----|------|---------|-------|
/// | `session.user_data_dir` | String (optional) | Platform default | — |
/// | `session.max_recent_files` | Integer | 50 | 1–500 |
/// | `session.restore_on_startup` | Boolean | true | — |
/// | `session.restore_tabs_on_startup` | Boolean | true | — |
/// | `session.startup_file` | String (optional) | None | — |
/// | `session.save_window_geometry` | Boolean | true | — |
/// | `session.crash_recovery_enabled` | Boolean | true | — |
/// | `session.auto_save_interval_seconds` | Integer | 300 | 30–3600 |
#[derive(Debug, Clone, PartialEq)]
pub struct SessionConfig {
    /// Custom User Data Directory path override. When `None`, the platform
    /// default is used.
    pub user_data_dir: Option<PathBuf>,

    /// Maximum number of entries in the Recent Files list.
    /// Validated range: 1–500.
    pub max_recent_files: u32,

    /// Whether to restore the previous session on startup.
    pub restore_on_startup: bool,

    /// Whether to reopen previously open tabs during session restore.
    /// Only meaningful when `restore_on_startup` is true.
    pub restore_tabs_on_startup: bool,

    /// Path to a file that should be opened on every launch (overrides session
    /// tab restore when set and no CLI args are provided).
    pub startup_file: Option<String>,

    /// Whether to persist and restore window geometry across restarts.
    pub save_window_geometry: bool,

    /// Whether crash recovery scanning is enabled.
    pub crash_recovery_enabled: bool,

    /// Interval in seconds between automatic session state saves.
    /// Validated range: 30–3600.
    pub auto_save_interval_seconds: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            user_data_dir: None,
            max_recent_files: 50,
            restore_on_startup: true,
            restore_tabs_on_startup: true,
            startup_file: None,
            save_window_geometry: true,
            crash_recovery_enabled: true,
            auto_save_interval_seconds: 300,
        }
    }
}

/// The minimum allowed value for `max_recent_files`.
pub const MAX_RECENT_FILES_MIN: u32 = 1;

/// The maximum allowed value for `max_recent_files`.
pub const MAX_RECENT_FILES_MAX: u32 = 500;

/// The minimum allowed value for `auto_save_interval_seconds`.
pub const AUTO_SAVE_INTERVAL_MIN: u32 = 30;

/// The maximum allowed value for `auto_save_interval_seconds`.
pub const AUTO_SAVE_INTERVAL_MAX: u32 = 3600;

/// Validation error for session configuration values.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValidationError {
    /// `max_recent_files` is outside the valid range.
    MaxRecentFilesOutOfRange {
        /// The invalid value that was provided.
        value: u32,
    },
    /// `auto_save_interval_seconds` is outside the valid range.
    AutoSaveIntervalOutOfRange {
        /// The invalid value that was provided.
        value: u32,
    },
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxRecentFilesOutOfRange { value } => {
                write!(
                    f,
                    "session.max_recent_files value {} is out of range ({}-{})",
                    value, MAX_RECENT_FILES_MIN, MAX_RECENT_FILES_MAX
                )
            }
            Self::AutoSaveIntervalOutOfRange { value } => {
                write!(
                    f,
                    "session.auto_save_interval_seconds value {} is out of range ({}-{})",
                    value, AUTO_SAVE_INTERVAL_MIN, AUTO_SAVE_INTERVAL_MAX
                )
            }
        }
    }
}

impl SessionConfig {
    /// Validate that all bounded fields are within their allowed ranges.
    ///
    /// Returns a list of validation errors (empty if valid).
    pub fn validate(&self) -> Vec<ConfigValidationError> {
        let mut errors = Vec::new();

        if self.max_recent_files < MAX_RECENT_FILES_MIN
            || self.max_recent_files > MAX_RECENT_FILES_MAX
        {
            errors.push(ConfigValidationError::MaxRecentFilesOutOfRange {
                value: self.max_recent_files,
            });
        }

        if self.auto_save_interval_seconds < AUTO_SAVE_INTERVAL_MIN
            || self.auto_save_interval_seconds > AUTO_SAVE_INTERVAL_MAX
        {
            errors.push(ConfigValidationError::AutoSaveIntervalOutOfRange {
                value: self.auto_save_interval_seconds,
            });
        }

        errors
    }

    /// Apply a hot-reload change to the configuration.
    ///
    /// Accepts a new configuration, validates it, and returns the updated
    /// config if valid. Returns the validation errors if the new config
    /// is invalid.
    ///
    /// Addresses: Requirement 2 AC 2.4 — hot-reload takes effect on next
    /// session operation without requiring restart.
    pub fn apply_reload(
        &mut self,
        new_config: SessionConfig,
    ) -> Result<(), Vec<ConfigValidationError>> {
        let errors = new_config.validate();
        if errors.is_empty() {
            *self = new_config;
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_values() {
        // Validates: Requirement 12 AC 12.1-12.9
        let config = SessionConfig::default();

        assert_eq!(config.user_data_dir, None);
        assert_eq!(config.max_recent_files, 50);
        assert!(config.restore_on_startup);
        assert!(config.restore_tabs_on_startup);
        assert_eq!(config.startup_file, None);
        assert!(config.save_window_geometry);
        assert!(config.crash_recovery_enabled);
        assert_eq!(config.auto_save_interval_seconds, 300);
    }

    #[test]
    fn default_config_passes_validation() {
        // Validates: Requirement 12 AC 12.3, 12.9
        let config = SessionConfig::default();
        let errors = config.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn max_recent_files_below_minimum_fails_validation() {
        // Validates: Requirement 12 AC 12.3
        let config = SessionConfig {
            max_recent_files: 0,
            ..Default::default()
        };
        let errors = config.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ConfigValidationError::MaxRecentFilesOutOfRange { value: 0 }
        );
    }

    #[test]
    fn max_recent_files_above_maximum_fails_validation() {
        // Validates: Requirement 12 AC 12.3
        let config = SessionConfig {
            max_recent_files: 501,
            ..Default::default()
        };
        let errors = config.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ConfigValidationError::MaxRecentFilesOutOfRange { value: 501 }
        );
    }

    #[test]
    fn max_recent_files_at_boundaries_passes_validation() {
        // Validates: Requirement 12 AC 12.3
        let config_min = SessionConfig {
            max_recent_files: 1,
            ..Default::default()
        };
        assert!(config_min.validate().is_empty());

        let config_max = SessionConfig {
            max_recent_files: 500,
            ..Default::default()
        };
        assert!(config_max.validate().is_empty());
    }

    #[test]
    fn auto_save_interval_below_minimum_fails_validation() {
        // Validates: Requirement 12 AC 12.9
        let config = SessionConfig {
            auto_save_interval_seconds: 29,
            ..Default::default()
        };
        let errors = config.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ConfigValidationError::AutoSaveIntervalOutOfRange { value: 29 }
        );
    }

    #[test]
    fn auto_save_interval_above_maximum_fails_validation() {
        // Validates: Requirement 12 AC 12.9
        let config = SessionConfig {
            auto_save_interval_seconds: 3601,
            ..Default::default()
        };
        let errors = config.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ConfigValidationError::AutoSaveIntervalOutOfRange { value: 3601 }
        );
    }

    #[test]
    fn auto_save_interval_at_boundaries_passes_validation() {
        // Validates: Requirement 12 AC 12.9
        let config_min = SessionConfig {
            auto_save_interval_seconds: 30,
            ..Default::default()
        };
        assert!(config_min.validate().is_empty());

        let config_max = SessionConfig {
            auto_save_interval_seconds: 3600,
            ..Default::default()
        };
        assert!(config_max.validate().is_empty());
    }

    #[test]
    fn multiple_validation_errors_reported_together() {
        // Validates: Requirement 12 AC 12.3, 12.9
        let config = SessionConfig {
            max_recent_files: 0,
            auto_save_interval_seconds: 10,
            ..Default::default()
        };
        let errors = config.validate();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn apply_reload_with_valid_config_succeeds() {
        // Validates: Requirement 2 AC 2.4
        let mut config = SessionConfig::default();
        let new_config = SessionConfig {
            max_recent_files: 100,
            auto_save_interval_seconds: 60,
            restore_on_startup: false,
            ..Default::default()
        };

        let result = config.apply_reload(new_config);
        assert!(result.is_ok());
        assert_eq!(config.max_recent_files, 100);
        assert_eq!(config.auto_save_interval_seconds, 60);
        assert!(!config.restore_on_startup);
    }

    #[test]
    fn apply_reload_with_invalid_config_rejects_and_preserves_original() {
        // Validates: Requirement 2 AC 2.4
        let mut config = SessionConfig::default();
        let original_max = config.max_recent_files;

        let invalid_config = SessionConfig {
            max_recent_files: 0,
            ..Default::default()
        };

        let result = config.apply_reload(invalid_config);
        assert!(result.is_err());
        // Original config is preserved
        assert_eq!(config.max_recent_files, original_max);
    }

    #[test]
    fn validation_error_display_is_descriptive() {
        let err = ConfigValidationError::MaxRecentFilesOutOfRange { value: 0 };
        let msg = err.to_string();
        assert!(msg.contains("max_recent_files"));
        assert!(msg.contains("0"));
        assert!(msg.contains("1-500"));

        let err = ConfigValidationError::AutoSaveIntervalOutOfRange { value: 10 };
        let msg = err.to_string();
        assert!(msg.contains("auto_save_interval_seconds"));
        assert!(msg.contains("10"));
        assert!(msg.contains("30-3600"));
    }
}
