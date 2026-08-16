//! Configuration key constants and typed access helpers for file operations.
//!
//! Centralises all configuration keys used by this crate.

/// Configuration keys for file operation settings.
pub mod keys {
    /// Save strategy: "atomic", "delete_first", or "direct".
    pub const FILE_SAVE_STRATEGY: &str = "file.save_strategy";
    /// Whether backup copies are enabled.
    pub const FILE_BACKUP_ENABLED: &str = "file.backup.enabled";
    /// Backup location: "alongside" or "directory".
    pub const FILE_BACKUP_LOCATION: &str = "file.backup.location";
    /// Suffix for alongside backups (default ".bak").
    pub const FILE_BACKUP_SUFFIX: &str = "file.backup.suffix";
    /// Directory path for directory-mode backups.
    pub const FILE_BACKUP_DIRECTORY: &str = "file.backup.directory";
    /// Maximum number of recent files entries.
    pub const FILE_RECENT_FILES_MAX_COUNT: &str = "file.recent_files.max_count";
    /// Size threshold (bytes) for async I/O.
    pub const FILE_ASYNC_THRESHOLD_BYTES: &str = "file.async_threshold_bytes";
    /// Whether to check modification time before save.
    pub const SAVE_CHECK_MODIFIED_TIME: &str = "save.check_modified_time";
    /// Whether to show unsaved-changes dialog.
    pub const FILE_UNSAVED_PROMPT: &str = "file.unsaved_prompt";
    /// Glob pattern for force read-only.
    pub const READ_ONLY: &str = "read.only";
}

/// Default values for file operation configuration.
pub mod defaults {
    /// Default save strategy.
    pub const SAVE_STRATEGY: &str = "atomic";
    /// Default backup enabled state.
    pub const BACKUP_ENABLED: bool = false;
    /// Default backup location.
    pub const BACKUP_LOCATION: &str = "alongside";
    /// Default backup suffix.
    pub const BACKUP_SUFFIX: &str = ".bak";
    /// Default maximum recent files count.
    pub const RECENT_FILES_MAX_COUNT: usize = 10;
    /// Default async threshold (1 MB).
    pub const ASYNC_THRESHOLD_BYTES: u64 = 1_048_576;
    /// Default modification time check.
    pub const CHECK_MODIFIED_TIME: bool = true;
    /// Default unsaved prompt.
    pub const UNSAVED_PROMPT: bool = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_keys_are_non_empty() {
        let all_keys = [
            keys::FILE_SAVE_STRATEGY,
            keys::FILE_BACKUP_ENABLED,
            keys::FILE_BACKUP_LOCATION,
            keys::FILE_BACKUP_SUFFIX,
            keys::FILE_BACKUP_DIRECTORY,
            keys::FILE_RECENT_FILES_MAX_COUNT,
            keys::FILE_ASYNC_THRESHOLD_BYTES,
            keys::SAVE_CHECK_MODIFIED_TIME,
            keys::FILE_UNSAVED_PROMPT,
            keys::READ_ONLY,
        ];
        for key in &all_keys {
            assert!(!key.is_empty(), "Config key should not be empty");
        }
    }

    #[test]
    fn default_values_are_sane() {
        assert_eq!(defaults::SAVE_STRATEGY, "atomic");
        assert!(!defaults::BACKUP_ENABLED);
        assert_eq!(defaults::RECENT_FILES_MAX_COUNT, 10);
        assert_eq!(defaults::ASYNC_THRESHOLD_BYTES, 1_048_576);
        assert!(defaults::CHECK_MODIFIED_TIME);
        assert!(defaults::UNSAVED_PROMPT);
    }
}
