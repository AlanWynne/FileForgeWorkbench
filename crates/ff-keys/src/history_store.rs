//! History Store — TOML persistence for Command History.
//!
//! Handles loading and saving the command history to/from a TOML file.
//! Implements graceful degradation: missing or corrupt files result in
//! an empty history, never a startup failure.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::command_history::CommandHistory;
use crate::error::KeysError;
use crate::key_map::KeyMapWarning;

/// The TOML schema for the history file.
#[derive(Debug, Serialize, Deserialize)]
struct HistoryFile {
    /// Schema version for forward compatibility.
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    /// The history entries in most-recent-first order.
    #[serde(default)]
    entries: Vec<HistoryFileEntry>,
}

/// A single entry in the history file.
#[derive(Debug, Serialize, Deserialize)]
struct HistoryFileEntry {
    /// The full command string.
    command: String,
}

fn default_schema_version() -> u32 {
    1
}

/// Persists CommandHistory to/from a TOML file.
#[derive(Debug, Clone)]
pub struct HistoryStore {
    /// Path to the history TOML file.
    path: PathBuf,
}

impl HistoryStore {
    /// Create a store handle for the given file path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// The path to the history file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether the history file exists on disk.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Load history from disk.
    ///
    /// Returns an empty history on missing or corrupt file (graceful degradation).
    /// Warnings are returned for corrupt file cases.
    pub fn load(&self, max_entries: usize) -> (CommandHistory, Vec<KeyMapWarning>) {
        let mut warnings = Vec::new();

        if !self.path.exists() {
            return (CommandHistory::new(max_entries), warnings);
        }

        let content = match fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(e) => {
                warnings.push(KeyMapWarning {
                    field: self.path.display().to_string(),
                    message: format!("failed to read history file: {}", e),
                });
                return (CommandHistory::new(max_entries), warnings);
            }
        };

        let history_file: HistoryFile = match toml::from_str(&content) {
            Ok(h) => h,
            Err(e) => {
                warnings.push(KeyMapWarning {
                    field: self.path.display().to_string(),
                    message: format!("invalid TOML in history file: {}", e),
                });
                return (CommandHistory::new(max_entries), warnings);
            }
        };

        let commands: Vec<String> = history_file
            .entries
            .into_iter()
            .map(|e| e.command)
            .collect();

        (
            CommandHistory::from_command_strings(commands, max_entries),
            warnings,
        )
    }

    /// Persist the current history to disk.
    ///
    /// Uses atomic write (write to temp file, then rename) to prevent
    /// data corruption on crash during save.
    pub fn save(&self, history: &CommandHistory) -> Result<(), KeysError> {
        let history_file = HistoryFile {
            schema_version: 1,
            entries: history
                .to_command_strings()
                .into_iter()
                .map(|cmd| HistoryFileEntry { command: cmd })
                .collect(),
        };

        let content = toml::to_string_pretty(&history_file).map_err(|e| {
            KeysError::HistoryStoreWriteFailed {
                reason: format!("serialization error: {}", e),
            }
        })?;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| KeysError::Io {
                operation: "history-save".to_string(),
                source: e,
            })?;
        }

        // Atomic write: write to temp file, then rename
        let temp_path = self.path.with_extension("toml.tmp");
        fs::write(&temp_path, &content).map_err(|e| KeysError::Io {
            operation: "history-save".to_string(),
            source: e,
        })?;

        fs::rename(&temp_path, &self.path).map_err(|e| KeysError::Io {
            operation: "history-save".to_string(),
            source: e,
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_store() -> (HistoryStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("command_history.toml");
        (HistoryStore::new(path), dir)
    }

    #[test]
    fn load_missing_file_returns_empty_history() {
        // Validates: Requirement 6.5
        let (store, _dir) = temp_store();
        let (history, warnings) = store.load(200);
        assert!(history.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn save_and_load_round_trip() {
        // Validates: Requirement 6.1, 6.7
        let (store, _dir) = temp_store();

        let mut history = CommandHistory::new(200);
        history.add("SAVE");
        history.add("FIND 'ERROR' ALL");
        history.add("CHANGE 'foo' 'bar' ALL");

        store.save(&history).unwrap();

        let (loaded, warnings) = store.load(200);
        assert!(warnings.is_empty());
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded.get(0).unwrap().command(), "CHANGE 'foo' 'bar' ALL");
        assert_eq!(loaded.get(1).unwrap().command(), "FIND 'ERROR' ALL");
        assert_eq!(loaded.get(2).unwrap().command(), "SAVE");
    }

    #[test]
    fn load_corrupt_file_returns_empty_with_warning() {
        // Validates: Requirement 6.6
        let (store, _dir) = temp_store();
        fs::write(store.path(), "this is not valid toml {{{{").unwrap();

        let (history, warnings) = store.load(200);
        assert!(history.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("invalid TOML"));
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sub").join("dir").join("history.toml");
        let store = HistoryStore::new(path);

        let mut history = CommandHistory::new(200);
        history.add("CMD1");
        store.save(&history).unwrap();

        assert!(store.exists());
    }

    #[test]
    fn load_empty_entries_array() {
        let (store, _dir) = temp_store();
        let content = r#"
schema_version = 1
entries = []
"#;
        fs::write(store.path(), content).unwrap();

        let (history, warnings) = store.load(200);
        assert!(history.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn load_respects_max_entries() {
        let (store, _dir) = temp_store();

        let mut history = CommandHistory::new(200);
        for i in 0..50 {
            history.add(format!("CMD{}", i));
        }
        store.save(&history).unwrap();

        let (loaded, _) = store.load(10);
        assert_eq!(loaded.len(), 10);
    }

    #[test]
    fn exists_returns_false_for_missing_file() {
        let (store, _dir) = temp_store();
        assert!(!store.exists());
    }

    #[test]
    fn exists_returns_true_after_save() {
        let (store, _dir) = temp_store();
        let history = CommandHistory::new(200);
        store.save(&history).unwrap();
        assert!(store.exists());
    }
}
