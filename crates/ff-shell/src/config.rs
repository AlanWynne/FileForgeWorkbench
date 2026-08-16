//! Shell configuration provider with hot-reload support.
//!
//! Reads all `shell.*` keys from `ff-config` and provides a typed snapshot
//! of the current effective configuration.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::ShellError;
use crate::profile::ShellProfile;

/// Security mode controlling shell access availability.
///
/// Determines whether shell commands can execute, require confirmation, or
/// are completely disabled.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellMode {
    /// Shell access is completely disabled.
    Disabled,
    /// User is prompted for confirmation before each execution.
    #[default]
    Prompt,
    /// Shell commands execute without prompting.
    Enabled,
}

/// Controls how the working directory is resolved for child processes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkingDirectoryMode {
    /// Use the project root directory (fallback: home directory).
    #[default]
    ProjectRoot,
    /// Use the parent directory of the active file (fallback: project root → home).
    FileDirectory,
}

/// Aggregate configuration for the shell subsystem.
///
/// All values sourced from `ff-config` under the `shell.*` namespace.
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// Security mode: disabled | prompt | enabled.
    pub mode: ShellMode,
    /// Override for default shell executable (None = auto-detect).
    pub default_shell: Option<String>,
    /// Command timeout in seconds (0 = disabled). Default: 30.
    pub timeout_seconds: u64,
    /// Working directory mode: project_root | file_directory.
    pub working_directory: WorkingDirectoryMode,
    /// Additional environment variables injected into child processes.
    pub env: HashMap<String, String>,
    /// Maximum scrollback lines for the Output Panel. Default: 10000.
    pub output_buffer_lines: usize,
    /// Named shell profiles.
    pub profiles: HashMap<String, ShellProfile>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            mode: ShellMode::default(),
            default_shell: None,
            timeout_seconds: 30,
            working_directory: WorkingDirectoryMode::default(),
            env: HashMap::new(),
            output_buffer_lines: 10_000,
            profiles: HashMap::new(),
        }
    }
}

/// Provides typed access to shell configuration with hot-reload support.
///
/// Wraps the `ff-config` system and maintains an in-memory snapshot of the
/// current effective shell configuration. Supports reload callbacks for
/// configuration changes.
#[derive(Debug, Clone)]
pub struct ShellConfigProvider {
    config: Arc<RwLock<ShellConfig>>,
}

impl ShellConfigProvider {
    /// Creates a new config provider with default configuration.
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ShellConfig::default())),
        }
    }

    /// Creates a new config provider with the given initial configuration.
    pub fn with_config(config: ShellConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Returns the current effective shell configuration snapshot.
    pub fn get(&self) -> ShellConfig {
        self.config.read().expect("config lock poisoned").clone()
    }

    /// Updates the configuration with new values.
    ///
    /// Used by the hot-reload callback to apply configuration changes.
    pub fn update(&self, config: ShellConfig) {
        let mut guard = self.config.write().expect("config lock poisoned");
        *guard = config;
    }

    /// Validates a shell mode string value, returning the mode or defaulting to `Prompt`.
    ///
    /// Logs a warning via `ff-logging` when an invalid value is encountered.
    pub fn validate_mode(value: &str) -> ShellMode {
        match value.to_lowercase().as_str() {
            "disabled" => ShellMode::Disabled,
            "prompt" => ShellMode::Prompt,
            "enabled" => ShellMode::Enabled,
            _ => {
                ff_logging::log(
                    ff_logging::LogLevel::Warn,
                    "ff_shell::config",
                    &format!(
                        "invalid shell.mode value '{}', falling back to 'prompt'",
                        value
                    ),
                );
                ShellMode::Prompt
            }
        }
    }

    /// Validates a timeout value, returning the value or defaulting to 30.
    ///
    /// Non-positive or non-numeric values are treated as invalid.
    pub fn validate_timeout(value: i64) -> u64 {
        if value > 0 {
            value as u64
        } else {
            ff_logging::log(
                ff_logging::LogLevel::Warn,
                "ff_shell::config",
                &format!(
                    "invalid shell.timeout_seconds value '{}', falling back to 30",
                    value
                ),
            );
            30
        }
    }

    /// Validates a working directory mode string.
    pub fn validate_working_directory(value: &str) -> Result<WorkingDirectoryMode, ShellError> {
        match value.to_lowercase().as_str() {
            "project_root" => Ok(WorkingDirectoryMode::ProjectRoot),
            "file_directory" => Ok(WorkingDirectoryMode::FileDirectory),
            _ => Err(ShellError::ConfigError {
                reason: format!("invalid shell.working_directory value: '{}'", value),
            }),
        }
    }
}

impl Default for ShellConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 10.1
    #[test]
    fn validate_mode_accepts_valid_values() {
        assert_eq!(
            ShellConfigProvider::validate_mode("disabled"),
            ShellMode::Disabled
        );
        assert_eq!(
            ShellConfigProvider::validate_mode("prompt"),
            ShellMode::Prompt
        );
        assert_eq!(
            ShellConfigProvider::validate_mode("enabled"),
            ShellMode::Enabled
        );
    }

    // Validates: Requirement 10.1
    #[test]
    fn validate_mode_is_case_insensitive() {
        assert_eq!(
            ShellConfigProvider::validate_mode("DISABLED"),
            ShellMode::Disabled
        );
        assert_eq!(
            ShellConfigProvider::validate_mode("Prompt"),
            ShellMode::Prompt
        );
        assert_eq!(
            ShellConfigProvider::validate_mode("ENABLED"),
            ShellMode::Enabled
        );
    }

    // Validates: Requirement 10.1
    #[test]
    fn validate_mode_falls_back_to_prompt_for_invalid() {
        assert_eq!(
            ShellConfigProvider::validate_mode("invalid"),
            ShellMode::Prompt
        );
        assert_eq!(ShellConfigProvider::validate_mode(""), ShellMode::Prompt);
        assert_eq!(ShellConfigProvider::validate_mode("yes"), ShellMode::Prompt);
    }

    // Validates: Requirement 10.3
    #[test]
    fn validate_timeout_accepts_positive_values() {
        assert_eq!(ShellConfigProvider::validate_timeout(30), 30);
        assert_eq!(ShellConfigProvider::validate_timeout(1), 1);
        assert_eq!(ShellConfigProvider::validate_timeout(120), 120);
    }

    // Validates: Requirement 10.3
    #[test]
    fn validate_timeout_falls_back_for_non_positive() {
        assert_eq!(ShellConfigProvider::validate_timeout(0), 30);
        assert_eq!(ShellConfigProvider::validate_timeout(-1), 30);
        assert_eq!(ShellConfigProvider::validate_timeout(-100), 30);
    }

    // Validates: Requirement 2.5
    #[test]
    fn default_config_has_prompt_mode() {
        let config = ShellConfig::default();
        assert_eq!(config.mode, ShellMode::Prompt);
    }

    // Validates: Requirement 10.3
    #[test]
    fn default_config_has_30_second_timeout() {
        let config = ShellConfig::default();
        assert_eq!(config.timeout_seconds, 30);
    }

    // Validates: Requirement 10.4
    #[test]
    fn default_config_has_project_root_working_directory() {
        let config = ShellConfig::default();
        assert_eq!(config.working_directory, WorkingDirectoryMode::ProjectRoot);
    }

    // Validates: Requirement 10.5
    #[test]
    fn config_provider_update_reflects_in_get() {
        let provider = ShellConfigProvider::new();
        let mut new_config = ShellConfig::default();
        new_config.mode = ShellMode::Enabled;
        new_config.timeout_seconds = 60;

        provider.update(new_config);

        let retrieved = provider.get();
        assert_eq!(retrieved.mode, ShellMode::Enabled);
        assert_eq!(retrieved.timeout_seconds, 60);
    }

    // Validates: Requirement 11.4
    #[test]
    fn validate_working_directory_accepts_valid_values() {
        assert_eq!(
            ShellConfigProvider::validate_working_directory("project_root").unwrap(),
            WorkingDirectoryMode::ProjectRoot
        );
        assert_eq!(
            ShellConfigProvider::validate_working_directory("file_directory").unwrap(),
            WorkingDirectoryMode::FileDirectory
        );
    }

    // Validates: Requirement 11.4
    #[test]
    fn validate_working_directory_rejects_invalid() {
        assert!(ShellConfigProvider::validate_working_directory("invalid").is_err());
    }
}
