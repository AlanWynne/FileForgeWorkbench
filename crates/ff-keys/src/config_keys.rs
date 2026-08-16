//! Configuration accessors for the `[keys]` namespace.
//!
//! Provides typed access to all key-related configuration values
//! with validation and defaults.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Default maximum history entries.
pub const DEFAULT_MAX_HISTORY_ENTRIES: usize = 200;

/// Default history file name.
pub const DEFAULT_HISTORY_FILE: &str = "command_history.toml";

/// Default excluded commands that are always excluded from history.
pub const DEFAULT_EXCLUDED_COMMANDS: &[&str] = &["RETRIEVE", "UNDO", "REDO"];

/// Typed configuration for the `ff-keys` subsystem.
///
/// Holds all configuration values with validation applied.
#[derive(Debug, Clone)]
pub struct KeysConfig {
    /// Maximum history entries (1–10000, default 200).
    max_history_entries: usize,
    /// History file path (relative to User_Data_Dir).
    history_file: String,
    /// Additional excluded commands beyond the defaults.
    additional_excluded_commands: Vec<String>,
}

impl KeysConfig {
    /// Create a new configuration with the given values, applying validation.
    ///
    /// Invalid `max_history_entries` (0 or greater than 10000) is clamped
    /// to the default of 200.
    pub fn new(
        max_history_entries: usize,
        history_file: Option<String>,
        additional_excluded_commands: Vec<String>,
    ) -> Self {
        let max = if max_history_entries == 0 || max_history_entries > 10000 {
            DEFAULT_MAX_HISTORY_ENTRIES
        } else {
            max_history_entries
        };

        Self {
            max_history_entries: max,
            history_file: history_file.unwrap_or_else(|| DEFAULT_HISTORY_FILE.to_string()),
            additional_excluded_commands,
        }
    }

    /// Maximum history entries. Default: 200.
    pub fn max_history_entries(&self) -> usize {
        self.max_history_entries
    }

    /// Resolve the history file path relative to the given User_Data_Dir.
    pub fn history_file_path(&self, user_data_dir: &Path) -> PathBuf {
        let file_path = Path::new(&self.history_file);
        if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            user_data_dir.join(file_path)
        }
    }

    /// The complete set of excluded commands (defaults + user-configured).
    pub fn excluded_commands(&self) -> HashSet<String> {
        let mut set: HashSet<String> = DEFAULT_EXCLUDED_COMMANDS
            .iter()
            .map(|s| s.to_string())
            .collect();
        for cmd in &self.additional_excluded_commands {
            set.insert(cmd.to_ascii_uppercase());
        }
        set
    }

    /// The additional excluded commands beyond defaults.
    pub fn additional_excluded_commands(&self) -> &[String] {
        &self.additional_excluded_commands
    }

    /// The raw history file string (before path resolution).
    pub fn history_file(&self) -> &str {
        &self.history_file
    }
}

impl Default for KeysConfig {
    fn default() -> Self {
        Self {
            max_history_entries: DEFAULT_MAX_HISTORY_ENTRIES,
            history_file: DEFAULT_HISTORY_FILE.to_string(),
            additional_excluded_commands: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        // Validates: Requirement 9.2
        let config = KeysConfig::default();
        assert_eq!(config.max_history_entries(), 200);
        assert_eq!(config.history_file(), "command_history.toml");
        assert!(config.additional_excluded_commands().is_empty());
    }

    #[test]
    fn zero_max_entries_uses_default() {
        // Validates: Requirement 9.4
        let config = KeysConfig::new(0, None, vec![]);
        assert_eq!(config.max_history_entries(), 200);
    }

    #[test]
    fn over_10000_max_entries_uses_default() {
        let config = KeysConfig::new(99999, None, vec![]);
        assert_eq!(config.max_history_entries(), 200);
    }

    #[test]
    fn valid_max_entries_preserved() {
        let config = KeysConfig::new(500, None, vec![]);
        assert_eq!(config.max_history_entries(), 500);
    }

    #[test]
    fn history_file_path_relative_to_user_data_dir() {
        // Validates: Requirement 6.4
        let config = KeysConfig::default();
        let path = config.history_file_path(Path::new("/home/user/.fileforge"));
        assert_eq!(
            path,
            PathBuf::from("/home/user/.fileforge/command_history.toml")
        );
    }

    #[test]
    fn history_file_path_absolute_not_joined() {
        let config = KeysConfig::new(200, Some("/absolute/path/history.toml".to_string()), vec![]);
        let path = config.history_file_path(Path::new("/home/user/.fileforge"));
        assert_eq!(path, PathBuf::from("/absolute/path/history.toml"));
    }

    #[test]
    fn excluded_commands_includes_defaults() {
        // Validates: Requirement 8.2
        let config = KeysConfig::default();
        let excluded = config.excluded_commands();
        assert!(excluded.contains("RETRIEVE"));
        assert!(excluded.contains("UNDO"));
        assert!(excluded.contains("REDO"));
    }

    #[test]
    fn excluded_commands_merges_additional() {
        // Validates: Requirement 8.3
        let config = KeysConfig::new(200, None, vec!["CUSTOM".to_string()]);
        let excluded = config.excluded_commands();
        assert!(excluded.contains("RETRIEVE"));
        assert!(excluded.contains("CUSTOM"));
    }
}
