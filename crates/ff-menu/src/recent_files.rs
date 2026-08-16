//! Recent files list management and persistence.
//!
//! Manages a bounded MRU (Most Recently Used) list of file paths.
//! The list is persisted as JSON in the workbench data directory.

use crate::error::MenuError;
use std::path::Path;

/// A single entry in the recent files list.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecentFileEntry {
    /// Absolute path to the file.
    pub path: String,
    /// Whether the file still exists on disk (checked lazily).
    #[serde(default)]
    pub verified_exists: Option<bool>,
}

/// Manages the most recently used files list.
///
/// The list is bounded by a configurable maximum (default 10, max 50).
/// Adding a file that already exists promotes it to the top.
#[derive(Debug, Clone)]
pub struct RecentFilesManager {
    /// Ordered entries (most recent first).
    entries: Vec<RecentFileEntry>,
    /// Maximum number of entries to retain.
    max_entries: usize,
}

impl RecentFilesManager {
    /// Creates a new manager with the given maximum capacity.
    ///
    /// The capacity is clamped to [1, 50].
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries: max_entries.clamp(1, 50),
        }
    }

    /// Returns the current maximum capacity.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Adds or promotes a file path to the top of the list.
    ///
    /// If the path already exists in the list, it is moved to the top.
    /// If the list exceeds max_entries after addition, the oldest entry is removed.
    pub fn add_or_promote(&mut self, path: &str) {
        // Remove existing entry with same path (case-sensitive)
        self.entries.retain(|e| e.path != path);

        // Insert at the front (most recent)
        self.entries.insert(
            0,
            RecentFileEntry {
                path: path.to_string(),
                verified_exists: None,
            },
        );

        // Trim to max
        self.entries.truncate(self.max_entries);
    }

    /// Returns the current list of recent files (most recent first).
    pub fn entries(&self) -> &[RecentFileEntry] {
        &self.entries
    }

    /// Returns the number of entries in the list.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes a specific entry by path.
    pub fn remove(&mut self, path: &str) {
        self.entries.retain(|e| e.path != path);
    }

    /// Clears the entire list.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Marks a path as non-existent (for greyed display).
    pub fn mark_missing(&mut self, path: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.path == path) {
            entry.verified_exists = Some(false);
        }
    }

    /// Removes entries marked as missing.
    pub fn purge_missing(&mut self) {
        self.entries.retain(|e| e.verified_exists != Some(false));
    }

    /// Loads recent files from a JSON file in the given data directory.
    ///
    /// # Errors
    ///
    /// Returns `MenuError::RecentFilesIoError` if the file cannot be read,
    /// or `MenuError::RecentFilesParseError` if the JSON is invalid.
    pub fn load(data_dir: &Path) -> Result<Self, MenuError> {
        let file_path = data_dir.join("recent_files.json");
        if !file_path.exists() {
            return Ok(Self::new(10));
        }

        let content =
            std::fs::read_to_string(&file_path).map_err(|e| MenuError::RecentFilesIoError {
                operation: "load".to_string(),
                path: file_path.clone(),
                source: e,
            })?;

        let data: RecentFilesData =
            serde_json::from_str(&content).map_err(|e| MenuError::RecentFilesParseError {
                path: file_path,
                detail: e.to_string(),
            })?;

        let max_entries = data.max_entries.unwrap_or(10).clamp(1, 50);
        let mut manager = Self::new(max_entries);
        manager.entries = data.entries;
        manager.entries.truncate(manager.max_entries);
        Ok(manager)
    }

    /// Persists the current list to a JSON file in the given data directory.
    ///
    /// # Errors
    ///
    /// Returns `MenuError::RecentFilesIoError` if the file cannot be written.
    pub fn save(&self, data_dir: &Path) -> Result<(), MenuError> {
        let file_path = data_dir.join("recent_files.json");

        let data = RecentFilesData {
            max_entries: Some(self.max_entries),
            entries: self.entries.clone(),
        };

        let json =
            serde_json::to_string_pretty(&data).map_err(|e| MenuError::RecentFilesIoError {
                operation: "serialize".to_string(),
                path: file_path.clone(),
                source: std::io::Error::other(e.to_string()),
            })?;

        std::fs::write(&file_path, json).map_err(|e| MenuError::RecentFilesIoError {
            operation: "save".to_string(),
            path: file_path,
            source: e,
        })?;

        Ok(())
    }
}

/// Serialization format for the recent files data file.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RecentFilesData {
    /// Configured maximum entries.
    max_entries: Option<usize>,
    /// The list of recent file entries.
    entries: Vec<RecentFileEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn new_manager_is_empty_with_clamped_max() {
        let mgr = RecentFilesManager::new(10);
        assert!(mgr.is_empty());
        assert_eq!(mgr.max_entries(), 10);

        // Clamp to minimum
        let mgr = RecentFilesManager::new(0);
        assert_eq!(mgr.max_entries(), 1);

        // Clamp to maximum
        let mgr = RecentFilesManager::new(100);
        assert_eq!(mgr.max_entries(), 50);
    }

    #[test]
    fn add_or_promote_adds_to_front() {
        let mut mgr = RecentFilesManager::new(10);
        mgr.add_or_promote("/path/a.txt");
        mgr.add_or_promote("/path/b.txt");

        assert_eq!(mgr.entries()[0].path, "/path/b.txt");
        assert_eq!(mgr.entries()[1].path, "/path/a.txt");
    }

    #[test]
    fn add_or_promote_promotes_existing_to_front() {
        let mut mgr = RecentFilesManager::new(10);
        mgr.add_or_promote("/path/a.txt");
        mgr.add_or_promote("/path/b.txt");
        mgr.add_or_promote("/path/c.txt");
        mgr.add_or_promote("/path/a.txt"); // promote

        assert_eq!(mgr.len(), 3);
        assert_eq!(mgr.entries()[0].path, "/path/a.txt");
        assert_eq!(mgr.entries()[1].path, "/path/c.txt");
        assert_eq!(mgr.entries()[2].path, "/path/b.txt");
    }

    #[test]
    fn add_or_promote_trims_to_max() {
        let mut mgr = RecentFilesManager::new(3);
        mgr.add_or_promote("/path/1.txt");
        mgr.add_or_promote("/path/2.txt");
        mgr.add_or_promote("/path/3.txt");
        mgr.add_or_promote("/path/4.txt");

        assert_eq!(mgr.len(), 3);
        assert_eq!(mgr.entries()[0].path, "/path/4.txt");
        assert_eq!(mgr.entries()[2].path, "/path/2.txt");
    }

    #[test]
    fn clear_removes_all_entries() {
        let mut mgr = RecentFilesManager::new(10);
        mgr.add_or_promote("/path/a.txt");
        mgr.add_or_promote("/path/b.txt");
        mgr.clear();
        assert!(mgr.is_empty());
    }

    #[test]
    fn mark_missing_and_purge() {
        let mut mgr = RecentFilesManager::new(10);
        mgr.add_or_promote("/path/a.txt");
        mgr.add_or_promote("/path/b.txt");
        mgr.mark_missing("/path/a.txt");
        mgr.purge_missing();
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.entries()[0].path, "/path/b.txt");
    }

    #[test]
    fn persistence_round_trip() {
        let dir = TempDir::new().unwrap();
        let mut mgr = RecentFilesManager::new(10);
        mgr.add_or_promote("/path/first.txt");
        mgr.add_or_promote("/path/second.txt");
        mgr.save(dir.path()).unwrap();

        let loaded = RecentFilesManager::load(dir.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.entries()[0].path, "/path/second.txt");
        assert_eq!(loaded.entries()[1].path, "/path/first.txt");
    }

    #[test]
    fn load_nonexistent_returns_empty_default() {
        let dir = TempDir::new().unwrap();
        let loaded = RecentFilesManager::load(dir.path()).unwrap();
        assert!(loaded.is_empty());
        assert_eq!(loaded.max_entries(), 10);
    }
}
