//! Configuration types, TOML deserialization, defaults, and validation.
//!
//! Defines `LogConfig` with support for platform-appropriate defaults,
//! value clamping, and TOML parsing.

use std::path::PathBuf;

use crate::level::LogLevel;

/// Configuration for the logging subsystem.
///
/// Sourced from the `[logging]` section of the workbench TOML configuration.
/// Provides defaults for all fields and clamps out-of-range values.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Minimum log level (default: `Info`).
    pub level: LogLevel,
    /// Log directory path (absolute or relative to working directory).
    pub directory: PathBuf,
    /// Maximum single file size in MB before rotation (default: 10, range: 1–1024).
    pub max_file_size_mb: u32,
    /// Maximum number of retained log files (default: 5, range: 1–100).
    pub max_retained_files: u32,
}

impl Default for LogConfig {
    /// Creates a `LogConfig` with sensible defaults:
    ///
    /// - level: `LogLevel::Info`
    /// - directory: platform-appropriate default (see [`default_log_directory`])
    /// - max_file_size_mb: 10
    /// - max_retained_files: 5
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            directory: default_log_directory(),
            max_file_size_mb: 10,
            max_retained_files: 5,
        }
    }
}

impl LogConfig {
    /// Validates and clamps configuration values to their allowed ranges.
    ///
    /// - `max_file_size_mb` is clamped to [1, 1024]
    /// - `max_retained_files` is clamped to [1, 100]
    ///
    /// Returns a `Vec<String>` of warning messages for any values that were
    /// clamped. An empty vector indicates no adjustments were needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_logging::LogConfig;
    ///
    /// let mut config = LogConfig::default();
    /// config.max_file_size_mb = 0;
    /// let warnings = config.validate();
    /// assert_eq!(config.max_file_size_mb, 1);
    /// assert_eq!(warnings.len(), 1);
    /// ```
    pub fn validate(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.max_file_size_mb < 1 {
            warnings.push(format!(
                "max_file_size_mb value {} is below minimum; clamped to 1",
                self.max_file_size_mb
            ));
            self.max_file_size_mb = 1;
        } else if self.max_file_size_mb > 1024 {
            warnings.push(format!(
                "max_file_size_mb value {} exceeds maximum; clamped to 1024",
                self.max_file_size_mb
            ));
            self.max_file_size_mb = 1024;
        }

        if self.max_retained_files < 1 {
            warnings.push(format!(
                "max_retained_files value {} is below minimum; clamped to 1",
                self.max_retained_files
            ));
            self.max_retained_files = 1;
        } else if self.max_retained_files > 100 {
            warnings.push(format!(
                "max_retained_files value {} exceeds maximum; clamped to 100",
                self.max_retained_files
            ));
            self.max_retained_files = 100;
        }

        warnings
    }

    /// Parses a log level from a user-provided string, applying the fallback
    /// behavior specified in Requirement 3, AC 3.4.
    ///
    /// If the string is recognized (case-insensitive, whitespace-trimmed),
    /// sets `self.level` to the parsed level and returns `None`.
    ///
    /// If the string is unrecognized, defaults `self.level` to `LogLevel::Info`
    /// and returns a warning message describing the invalid value and fallback.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_logging::{LogConfig, LogLevel};
    ///
    /// let mut config = LogConfig::default();
    /// let warning = config.set_level_from_str("debug");
    /// assert_eq!(config.level, LogLevel::Debug);
    /// assert!(warning.is_none());
    ///
    /// let warning = config.set_level_from_str("banana");
    /// assert_eq!(config.level, LogLevel::Info);
    /// assert!(warning.is_some());
    /// ```
    pub fn set_level_from_str(&mut self, level_str: &str) -> Option<String> {
        match LogLevel::from_str_lenient(level_str) {
            Some(level) => {
                self.level = level;
                None
            }
            None => {
                let warning = format!(
                    "logging.level contains unrecognized value '{}'; defaulting to INFO",
                    level_str.trim()
                );
                self.level = LogLevel::Info;
                Some(warning)
            }
        }
    }
}

/// Returns the platform-appropriate default log directory.
///
/// - **Windows:** `%LOCALAPPDATA%/FileForgeWorkbench/logs`
/// - **Linux/macOS:** `$XDG_DATA_HOME/file-forge-workbench/logs`
///   (falls back to `~/.local/share/file-forge-workbench/logs` if `XDG_DATA_HOME` is unset)
///
/// Uses the `dirs` crate's `data_local_dir()` function which handles
/// environment variable resolution and platform differences.
///
/// # Panics
///
/// Falls back to a relative path `logs/` if the platform directory cannot
/// be determined (should not happen on supported platforms).
pub fn default_log_directory() -> PathBuf {
    if let Some(data_dir) = dirs::data_local_dir() {
        if cfg!(windows) {
            data_dir.join("FileForgeWorkbench").join("logs")
        } else {
            data_dir.join("file-forge-workbench").join("logs")
        }
    } else {
        // Fallback if platform directory cannot be determined
        PathBuf::from("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Default Tests ──────────────────────────────────────────────────────

    #[test]
    fn default_config_has_info_level() {
        // Validates: Requirement 3.3
        let config = LogConfig::default();
        assert_eq!(config.level, LogLevel::Info);
    }

    #[test]
    fn default_config_has_10mb_max_file_size() {
        // Validates: Requirement 5.2
        let config = LogConfig::default();
        assert_eq!(config.max_file_size_mb, 10);
    }

    #[test]
    fn default_config_has_5_max_retained_files() {
        // Validates: Requirement 5.7
        let config = LogConfig::default();
        assert_eq!(config.max_retained_files, 5);
    }

    #[test]
    fn default_config_directory_is_platform_default() {
        // Validates: Requirement 4.3
        let config = LogConfig::default();
        assert_eq!(config.directory, default_log_directory());
    }

    // ─── Platform Directory Tests ───────────────────────────────────────────

    #[test]
    fn default_log_directory_is_absolute_path() {
        // Validates: Requirement 4.3
        let dir = default_log_directory();
        // On supported platforms, dirs::data_local_dir() should return an absolute path
        if dirs::data_local_dir().is_some() {
            assert!(dir.is_absolute());
        }
    }

    #[test]
    fn default_log_directory_ends_with_logs() {
        // Validates: Requirement 4.3
        let dir = default_log_directory();
        assert_eq!(dir.file_name().and_then(|n| n.to_str()), Some("logs"));
    }

    #[cfg(windows)]
    #[test]
    fn default_log_directory_contains_fileforge_workbench_on_windows() {
        // Validates: Requirement 4.3
        let dir = default_log_directory();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains("FileForgeWorkbench"));
    }

    #[cfg(not(windows))]
    #[test]
    fn default_log_directory_contains_file_forge_workbench_on_unix() {
        // Validates: Requirement 4.3
        let dir = default_log_directory();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains("file-forge-workbench"));
    }

    // ─── Validation / Clamping Tests ────────────────────────────────────────

    #[test]
    fn validate_clamps_max_file_size_below_minimum() {
        // Validates: Requirement 5.3
        let mut config = LogConfig::default();
        config.max_file_size_mb = 0;
        let warnings = config.validate();
        assert_eq!(config.max_file_size_mb, 1);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("max_file_size_mb"));
    }

    #[test]
    fn validate_clamps_max_file_size_above_maximum() {
        // Validates: Requirement 5.3
        let mut config = LogConfig::default();
        config.max_file_size_mb = 2000;
        let warnings = config.validate();
        assert_eq!(config.max_file_size_mb, 1024);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("max_file_size_mb"));
    }

    #[test]
    fn validate_clamps_max_retained_files_below_minimum() {
        // Validates: Requirement 5.8
        let mut config = LogConfig::default();
        config.max_retained_files = 0;
        let warnings = config.validate();
        assert_eq!(config.max_retained_files, 1);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("max_retained_files"));
    }

    #[test]
    fn validate_clamps_max_retained_files_above_maximum() {
        // Validates: Requirement 5.8
        let mut config = LogConfig::default();
        config.max_retained_files = 200;
        let warnings = config.validate();
        assert_eq!(config.max_retained_files, 100);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("max_retained_files"));
    }

    #[test]
    fn validate_returns_no_warnings_for_valid_config() {
        // Validates: Requirement 5.3, 5.8
        let mut config = LogConfig::default();
        let warnings = config.validate();
        assert!(warnings.is_empty());
    }

    #[test]
    fn validate_clamps_both_values_when_both_out_of_range() {
        // Validates: Requirement 5.3, 5.8
        let mut config = LogConfig::default();
        config.max_file_size_mb = 0;
        config.max_retained_files = 999;
        let warnings = config.validate();
        assert_eq!(config.max_file_size_mb, 1);
        assert_eq!(config.max_retained_files, 100);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn validate_preserves_boundary_values() {
        // Validates: Requirement 5.3, 5.8
        let mut config = LogConfig::default();
        config.max_file_size_mb = 1;
        config.max_retained_files = 1;
        let warnings = config.validate();
        assert_eq!(config.max_file_size_mb, 1);
        assert_eq!(config.max_retained_files, 1);
        assert!(warnings.is_empty());

        config.max_file_size_mb = 1024;
        config.max_retained_files = 100;
        let warnings = config.validate();
        assert_eq!(config.max_file_size_mb, 1024);
        assert_eq!(config.max_retained_files, 100);
        assert!(warnings.is_empty());
    }

    // ─── set_level_from_str Tests ───────────────────────────────────────────

    #[test]
    fn set_level_from_str_sets_valid_level_and_returns_none() {
        // Validates: Requirement 3.1
        let mut config = LogConfig::default();

        let warning = config.set_level_from_str("trace");
        assert_eq!(config.level, LogLevel::Trace);
        assert!(warning.is_none());

        let warning = config.set_level_from_str("debug");
        assert_eq!(config.level, LogLevel::Debug);
        assert!(warning.is_none());

        let warning = config.set_level_from_str("warn");
        assert_eq!(config.level, LogLevel::Warn);
        assert!(warning.is_none());

        let warning = config.set_level_from_str("error");
        assert_eq!(config.level, LogLevel::Error);
        assert!(warning.is_none());
    }

    #[test]
    fn set_level_from_str_is_case_insensitive() {
        // Validates: Requirement 3.1
        let mut config = LogConfig::default();

        let warning = config.set_level_from_str("DEBUG");
        assert_eq!(config.level, LogLevel::Debug);
        assert!(warning.is_none());

        let warning = config.set_level_from_str("  Warn  ");
        assert_eq!(config.level, LogLevel::Warn);
        assert!(warning.is_none());
    }

    #[test]
    fn set_level_from_str_defaults_to_info_on_invalid_value() {
        // Validates: Requirement 3.4
        let mut config = LogConfig::default();
        config.level = LogLevel::Error; // set to something other than Info

        let warning = config.set_level_from_str("banana");
        assert_eq!(config.level, LogLevel::Info);
        assert!(warning.is_some());
    }

    #[test]
    fn set_level_from_str_warning_contains_invalid_value_and_fallback() {
        // Validates: Requirement 3.4
        let mut config = LogConfig::default();

        let warning = config.set_level_from_str("foobar").unwrap();
        assert!(
            warning.contains("foobar"),
            "warning should contain the invalid value"
        );
        assert!(
            warning.contains("INFO"),
            "warning should mention the fallback level"
        );
    }

    #[test]
    fn set_level_from_str_warning_trims_whitespace_in_message() {
        // Validates: Requirement 3.4
        let mut config = LogConfig::default();

        let warning = config.set_level_from_str("  invalid  ").unwrap();
        assert!(
            warning.contains("invalid"),
            "warning should contain the trimmed invalid value"
        );
    }

    #[test]
    fn set_level_from_str_handles_empty_string() {
        // Validates: Requirement 3.4
        let mut config = LogConfig::default();
        config.level = LogLevel::Error;

        let warning = config.set_level_from_str("");
        assert_eq!(config.level, LogLevel::Info);
        assert!(warning.is_some());
    }
}
