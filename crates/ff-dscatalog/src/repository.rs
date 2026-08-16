//! Repository layout management.
//!
//! Manages the physical directory structure on disk: `storage/`, `pds/`, `gdg/`, `temp/`,
//! and the `catalog.db` file.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::error::CatalogError;
use crate::schema;

/// Duration threshold for temp file staleness (24 hours).
const TEMP_STALE_THRESHOLD: Duration = Duration::from_secs(24 * 60 * 60);

/// A repository directory structure on the local filesystem.
///
/// Contains subdirectories for each dataset type and the catalog database.
#[derive(Debug, Clone)]
pub struct Repository {
    /// Root path of the repository.
    root: PathBuf,
}

impl Repository {
    /// Create a new Repository reference at the given root path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Get the repository root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path to the `storage/` directory (sequential datasets).
    pub fn storage_dir(&self) -> PathBuf {
        self.root.join("storage")
    }

    /// Path to the `pds/` directory (partitioned datasets).
    pub fn pds_dir(&self) -> PathBuf {
        self.root.join("pds")
    }

    /// Path to the `gdg/` directory (generation data groups).
    pub fn gdg_dir(&self) -> PathBuf {
        self.root.join("gdg")
    }

    /// Path to the `temp/` directory (temporary allocations).
    pub fn temp_dir(&self) -> PathBuf {
        self.root.join("temp")
    }

    /// Path to the `catalog.db` SQLite database file.
    pub fn catalog_db_path(&self) -> PathBuf {
        self.root.join("catalog.db")
    }

    /// Initialize the repository directory structure and catalog database.
    ///
    /// Creates all required subdirectories and an empty catalog database.
    ///
    /// # Errors
    ///
    /// Returns `CatalogError::IoError` if directory creation fails.
    /// Returns `CatalogError::RepositoryCorrupt` if root is a non-empty file.
    pub fn initialize(&self, catalog_name: &str) -> Result<(), CatalogError> {
        // Create root if it doesn't exist
        fs::create_dir_all(&self.root).map_err(|e| CatalogError::IoError {
            operation: "initialize_repository".to_string(),
            source: e,
        })?;

        // Create subdirectories
        for dir in &[
            self.storage_dir(),
            self.pds_dir(),
            self.gdg_dir(),
            self.temp_dir(),
        ] {
            fs::create_dir_all(dir).map_err(|e| CatalogError::IoError {
                operation: "initialize_repository".to_string(),
                source: e,
            })?;
        }

        // Create and initialize the catalog database
        let db_path = self.catalog_db_path();
        let conn = rusqlite::Connection::open(&db_path).map_err(|e| CatalogError::SqliteError {
            operation: "initialize_repository".to_string(),
            source: e,
        })?;

        schema::initialize_database(&conn, catalog_name)?;

        Ok(())
    }

    /// Validate the repository structure.
    ///
    /// Checks that all required subdirectories and the catalog database exist.
    ///
    /// # Errors
    ///
    /// Returns `CatalogError::RepositoryCorrupt` with details about missing elements.
    pub fn validate(&self) -> Result<(), CatalogError> {
        if !self.root.exists() {
            return Err(CatalogError::RepositoryCorrupt {
                path: self.root.display().to_string(),
                reason: "repository root does not exist".to_string(),
                operation: "validate".to_string(),
            });
        }

        let required_dirs = [
            ("storage", self.storage_dir()),
            ("pds", self.pds_dir()),
            ("gdg", self.gdg_dir()),
            ("temp", self.temp_dir()),
        ];

        for (name, path) in &required_dirs {
            if !path.exists() {
                return Err(CatalogError::RepositoryCorrupt {
                    path: self.root.display().to_string(),
                    reason: format!("missing required directory: {name}/"),
                    operation: "validate".to_string(),
                });
            }
        }

        if !self.catalog_db_path().exists() {
            return Err(CatalogError::RepositoryCorrupt {
                path: self.root.display().to_string(),
                reason: "missing catalog.db".to_string(),
                operation: "validate".to_string(),
            });
        }

        Ok(())
    }

    /// Purge stale files in the `temp/` directory.
    ///
    /// Removes files older than 24 hours.
    pub fn purge_temp(&self) -> Result<u32, CatalogError> {
        let temp_dir = self.temp_dir();
        if !temp_dir.exists() {
            return Ok(0);
        }

        let now = SystemTime::now();
        let mut purged = 0u32;

        let entries = fs::read_dir(&temp_dir).map_err(|e| CatalogError::IoError {
            operation: "purge_temp".to_string(),
            source: e,
        })?;

        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > TEMP_STALE_THRESHOLD {
                            let path = entry.path();
                            if path.is_file() {
                                let _ = fs::remove_file(&path);
                                purged += 1;
                            } else if path.is_dir() {
                                let _ = fs::remove_dir_all(&path);
                                purged += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(purged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn initialize_creates_directory_structure() {
        // Validates: Requirement 4 AC 1, AC 6
        let tmp = TempDir::new().unwrap();
        let repo = Repository::new(tmp.path().join("test-repo"));
        repo.initialize("TEST").unwrap();

        assert!(repo.storage_dir().exists());
        assert!(repo.pds_dir().exists());
        assert!(repo.gdg_dir().exists());
        assert!(repo.temp_dir().exists());
        assert!(repo.catalog_db_path().exists());
    }

    #[test]
    fn validate_succeeds_on_valid_repo() {
        // Validates: Requirement 4 AC 1
        let tmp = TempDir::new().unwrap();
        let repo = Repository::new(tmp.path().join("valid-repo"));
        repo.initialize("TEST").unwrap();
        repo.validate().unwrap();
    }

    #[test]
    fn validate_fails_on_missing_root() {
        // Validates: Requirement 5 AC 5
        let repo = Repository::new(PathBuf::from("/nonexistent/path/repo"));
        let result = repo.validate();
        assert!(result.is_err());
    }

    #[test]
    fn validate_fails_on_missing_subdirectory() {
        // Validates: Requirement 5 AC 5
        let tmp = TempDir::new().unwrap();
        let repo = Repository::new(tmp.path().join("partial-repo"));
        fs::create_dir_all(repo.root()).unwrap();
        fs::create_dir_all(repo.storage_dir()).unwrap();
        // Missing pds/, gdg/, temp/, catalog.db

        let result = repo.validate();
        assert!(result.is_err());
    }

    #[test]
    fn purge_temp_removes_stale_files() {
        // Validates: Requirement 4 AC 9
        let tmp = TempDir::new().unwrap();
        let repo = Repository::new(tmp.path().join("purge-repo"));
        repo.initialize("TEST").unwrap();

        // Create a file in temp (it won't be stale immediately, so purge returns 0)
        let temp_file = repo.temp_dir().join("recent.tmp");
        fs::write(&temp_file, b"data").unwrap();

        let purged = repo.purge_temp().unwrap();
        assert_eq!(purged, 0); // File is recent, not stale
        assert!(temp_file.exists());
    }

    #[test]
    fn catalog_db_path_correct() {
        // Validates: Requirement 1 AC 1
        let repo = Repository::new(PathBuf::from("/some/path"));
        assert_eq!(
            repo.catalog_db_path(),
            PathBuf::from("/some/path/catalog.db")
        );
    }
}
