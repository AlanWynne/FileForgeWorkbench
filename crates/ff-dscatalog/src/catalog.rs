//! Single catalog instance management.
//!
//! A `Catalog` represents one mounted catalog with its SQLite database
//! and repository. Provides dataset CRUD, PDS member ops, and GDG management.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::dataset::{AllocParams, DatasetRecord, Dsorg, PartitionedSubtype, Recfm};
use crate::dsn::Dsn;
use crate::encoding::dsn_to_storage_path;
use crate::error::CatalogError;
use crate::repository::Repository;
use crate::schema;

/// A mounted catalog instance.
///
/// Holds the SQLite connection and repository reference for one catalog.
#[derive(Debug)]
pub struct Catalog {
    /// Human-readable catalog name.
    name: String,
    /// The repository managing physical storage.
    repository: Repository,
    /// SQLite database connection.
    conn: Connection,
    /// Priority for resolution ordering.
    priority: u32,
}

impl Catalog {
    /// Mount a catalog from a repository path.
    ///
    /// Opens the repository, validates structure and schema, opens SQLite
    /// connection with WAL mode.
    ///
    /// # Errors
    ///
    /// Returns error if validation fails or database cannot be opened.
    pub fn mount(path: &Path, priority: u32) -> Result<Self, CatalogError> {
        let repository = Repository::new(path);
        repository.validate()?;

        let db_path = repository.catalog_db_path();
        let conn = Connection::open(&db_path).map_err(|e| CatalogError::SqliteError {
            operation: "mount".to_string(),
            source: e,
        })?;

        // Enable WAL mode and foreign keys
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| CatalogError::SqliteError {
                operation: "mount".to_string(),
                source: e,
            })?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| CatalogError::SqliteError {
                operation: "mount".to_string(),
                source: e,
            })?;

        schema::validate_schema(&conn)?;

        // Read catalog name from metadata
        let name: String = conn
            .query_row(
                "SELECT value FROM catalog_metadata WHERE key = ?1",
                rusqlite::params!["catalog_name"],
                |row| row.get(0),
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "mount".to_string(),
                source: e,
            })?;

        // Purge stale temp files
        repository.purge_temp()?;

        Ok(Self {
            name,
            repository,
            conn,
            priority,
        })
    }

    /// Get the catalog name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the catalog priority.
    pub fn priority(&self) -> u32 {
        self.priority
    }

    /// Get the repository root path.
    pub fn repository_path(&self) -> &Path {
        self.repository.root()
    }

    /// Allocate (create) a new dataset in this catalog.
    ///
    /// Validates parameters, creates physical storage, inserts catalog entry.
    pub fn allocate(&self, params: AllocParams) -> Result<DatasetRecord, CatalogError> {
        // Apply defaults and validate
        let params = params.with_defaults();
        params.validate()?;

        // Check DSN uniqueness within this catalog
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM datasets WHERE dsn = ?1",
                rusqlite::params![params.dsn.as_str()],
                |row| row.get(0),
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "allocate".to_string(),
                source: e,
            })?;

        if exists {
            return Err(CatalogError::DuplicateDataset {
                dsn: params.dsn.as_str().to_string(),
                catalog: self.name.clone(),
                operation: "allocate".to_string(),
            });
        }

        // Determine storage path and create physical storage
        let storage_path = self.create_physical_storage(&params)?;
        let now = chrono::Utc::now().to_rfc3339();

        // Insert catalog entry
        self.conn
            .execute(
                "INSERT INTO datasets (dsn, dsorg, storage_path, recfm, lrecl, blksize, subtype, created, modified) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    params.dsn.as_str(),
                    params.dsorg.to_string(),
                    &storage_path,
                    params.recfm.map(|r| r.to_string()),
                    params.lrecl,
                    params.blksize,
                    params.subtype.map(|s| s.to_string()),
                    &now,
                    &now,
                ],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "allocate".to_string(),
                source: e,
            })?;

        let id = self.conn.last_insert_rowid();

        Ok(DatasetRecord {
            id,
            dsn: params.dsn,
            dsorg: params.dsorg,
            storage_path,
            recfm: params.recfm,
            lrecl: params.lrecl,
            blksize: params.blksize,
            subtype: params.subtype,
            created: Some(now.clone()),
            modified: Some(now),
            accessed: None,
        })
    }

    /// Create physical storage for a dataset based on its organization type.
    fn create_physical_storage(&self, params: &AllocParams) -> Result<String, CatalogError> {
        let encoded_path = dsn_to_storage_path(&params.dsn);

        match params.dsorg {
            Dsorg::PS => {
                let full_path = self.repository.storage_dir().join(&encoded_path);
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| CatalogError::IoError {
                        operation: "allocate".to_string(),
                        source: e,
                    })?;
                }
                // Create empty file
                std::fs::File::create(&full_path).map_err(|e| CatalogError::IoError {
                    operation: "allocate".to_string(),
                    source: e,
                })?;
                Ok(format!("storage/{encoded_path}"))
            }
            Dsorg::PO => {
                let full_path = self.repository.pds_dir().join(&encoded_path);
                std::fs::create_dir_all(&full_path).map_err(|e| CatalogError::IoError {
                    operation: "allocate".to_string(),
                    source: e,
                })?;
                Ok(format!("pds/{encoded_path}"))
            }
            Dsorg::GDG => {
                // GDG base: create directory in gdg/
                let full_path = self.repository.gdg_dir().join(&encoded_path);
                std::fs::create_dir_all(&full_path).map_err(|e| CatalogError::IoError {
                    operation: "allocate".to_string(),
                    source: e,
                })?;
                Ok(format!("gdg/{encoded_path}"))
            }
        }
    }

    /// Delete a dataset by DSN.
    ///
    /// Removes catalog entry and physical storage.
    pub fn delete(&self, dsn: &Dsn) -> Result<(), CatalogError> {
        let record = self.lookup(dsn)?;

        // Delete physical storage
        let physical_path = self.repository.root().join(&record.storage_path);
        if physical_path.is_file() {
            std::fs::remove_file(&physical_path).map_err(|e| CatalogError::IoError {
                operation: "delete".to_string(),
                source: e,
            })?;
        } else if physical_path.is_dir() {
            std::fs::remove_dir_all(&physical_path).map_err(|e| CatalogError::IoError {
                operation: "delete".to_string(),
                source: e,
            })?;
        }

        // Remove catalog entry
        self.conn
            .execute(
                "DELETE FROM datasets WHERE dsn = ?1",
                rusqlite::params![dsn.as_str()],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "delete".to_string(),
                source: e,
            })?;

        Ok(())
    }

    /// Rename a dataset.
    ///
    /// Updates catalog entry and renames physical storage path.
    pub fn rename(&self, old_dsn: &Dsn, new_dsn: &Dsn) -> Result<(), CatalogError> {
        // Check new DSN doesn't already exist
        let new_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM datasets WHERE dsn = ?1",
                rusqlite::params![new_dsn.as_str()],
                |row| row.get(0),
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "rename".to_string(),
                source: e,
            })?;

        if new_exists {
            return Err(CatalogError::DuplicateDataset {
                dsn: new_dsn.as_str().to_string(),
                catalog: self.name.clone(),
                operation: "rename".to_string(),
            });
        }

        let record = self.lookup(old_dsn)?;
        let new_encoded = dsn_to_storage_path(new_dsn);

        // Determine new storage path based on dsorg prefix
        let prefix = if record.storage_path.starts_with("storage/") {
            "storage"
        } else if record.storage_path.starts_with("pds/") {
            "pds"
        } else {
            "gdg"
        };
        let new_storage_path = format!("{prefix}/{new_encoded}");

        // Rename physical path
        let old_physical = self.repository.root().join(&record.storage_path);
        let new_physical = self.repository.root().join(&new_storage_path);

        if let Some(parent) = new_physical.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CatalogError::IoError {
                operation: "rename".to_string(),
                source: e,
            })?;
        }

        if old_physical.exists() {
            std::fs::rename(&old_physical, &new_physical).map_err(|e| CatalogError::IoError {
                operation: "rename".to_string(),
                source: e,
            })?;
        }

        // Update catalog entry
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE datasets SET dsn = ?1, storage_path = ?2, modified = ?3 WHERE dsn = ?4",
                rusqlite::params![new_dsn.as_str(), &new_storage_path, &now, old_dsn.as_str()],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "rename".to_string(),
                source: e,
            })?;

        Ok(())
    }

    /// Look up a dataset by DSN in this catalog.
    pub fn lookup(&self, dsn: &Dsn) -> Result<DatasetRecord, CatalogError> {
        let row = self.conn.query_row(
            "SELECT id, dsn, dsorg, storage_path, recfm, lrecl, blksize, subtype, created, modified, accessed \
             FROM datasets WHERE dsn = ?1",
            rusqlite::params![dsn.as_str()],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<u32>>(5)?,
                    row.get::<_, Option<u32>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                ))
            },
        );

        match row {
            Ok((
                id,
                dsn_str,
                dsorg_str,
                storage_path,
                recfm_str,
                lrecl,
                blksize,
                subtype_str,
                created,
                modified,
                accessed,
            )) => {
                let dsn = Dsn::parse(&dsn_str).map_err(|_| CatalogError::RepositoryCorrupt {
                    path: self.repository.root().display().to_string(),
                    reason: format!("invalid DSN in database: {dsn_str}"),
                    operation: "lookup".to_string(),
                })?;
                let dsorg: Dsorg =
                    dsorg_str
                        .parse()
                        .map_err(|_| CatalogError::RepositoryCorrupt {
                            path: self.repository.root().display().to_string(),
                            reason: format!("invalid DSORG in database: {dsorg_str}"),
                            operation: "lookup".to_string(),
                        })?;
                let recfm = recfm_str.and_then(|s| s.parse::<Recfm>().ok());
                let subtype = subtype_str.and_then(|s| s.parse::<PartitionedSubtype>().ok());

                Ok(DatasetRecord {
                    id,
                    dsn,
                    dsorg,
                    storage_path,
                    recfm,
                    lrecl,
                    blksize,
                    subtype,
                    created,
                    modified,
                    accessed,
                })
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(CatalogError::DatasetNotFound {
                dsn: dsn.as_str().to_string(),
                operation: "lookup".to_string(),
            }),
            Err(e) => Err(CatalogError::SqliteError {
                operation: "lookup".to_string(),
                source: e,
            }),
        }
    }

    /// Check if a DSN exists in this catalog.
    pub fn exists(&self, dsn: &Dsn) -> Result<bool, CatalogError> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM datasets WHERE dsn = ?1",
                rusqlite::params![dsn.as_str()],
                |row| row.get(0),
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "exists".to_string(),
                source: e,
            })?;
        Ok(count > 0)
    }

    /// List all datasets in this catalog.
    pub fn list_datasets(&self) -> Result<Vec<DatasetRecord>, CatalogError> {
        let mut stmt = self.conn
            .prepare(
                "SELECT id, dsn, dsorg, storage_path, recfm, lrecl, blksize, subtype, created, modified, accessed \
                 FROM datasets ORDER BY dsn"
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "list_datasets".to_string(),
                source: e,
            })?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<u32>>(5)?,
                    row.get::<_, Option<u32>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                ))
            })
            .map_err(|e| CatalogError::SqliteError {
                operation: "list_datasets".to_string(),
                source: e,
            })?;

        let mut results = Vec::new();
        for row in rows {
            let (
                id,
                dsn_str,
                dsorg_str,
                storage_path,
                recfm_str,
                lrecl,
                blksize,
                subtype_str,
                created,
                modified,
                accessed,
            ) = row.map_err(|e| CatalogError::SqliteError {
                operation: "list_datasets".to_string(),
                source: e,
            })?;
            if let Ok(dsn) = Dsn::parse(&dsn_str) {
                if let Ok(dsorg) = dsorg_str.parse::<Dsorg>() {
                    let recfm = recfm_str.and_then(|s| s.parse::<Recfm>().ok());
                    let subtype = subtype_str.and_then(|s| s.parse::<PartitionedSubtype>().ok());
                    results.push(DatasetRecord {
                        id,
                        dsn,
                        dsorg,
                        storage_path,
                        recfm,
                        lrecl,
                        blksize,
                        subtype,
                        created,
                        modified,
                        accessed,
                    });
                }
            }
        }
        Ok(results)
    }

    /// Update access date for a dataset.
    pub fn update_access_date(&self, dsn: &Dsn) -> Result<(), CatalogError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE datasets SET accessed = ?1 WHERE dsn = ?2",
                rusqlite::params![&now, dsn.as_str()],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "update_access_date".to_string(),
                source: e,
            })?;
        Ok(())
    }

    /// Get the physical path for a dataset.
    pub fn physical_path(&self, dsn: &Dsn) -> Result<PathBuf, CatalogError> {
        let record = self.lookup(dsn)?;
        Ok(self.repository.root().join(&record.storage_path))
    }

    /// Get the repository reference.
    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    /// Get a reference to the database connection (for GDG/PDS ops).
    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_catalog() -> (TempDir, Catalog) {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("test-catalog");
        let repo = Repository::new(&repo_path);
        repo.initialize("TEST").unwrap();
        let catalog = Catalog::mount(&repo_path, 1).unwrap();
        (tmp, catalog)
    }

    #[test]
    fn mount_succeeds_on_valid_repo() {
        // Validates: Requirement 5 AC 1, AC 5
        let (_tmp, catalog) = setup_catalog();
        assert_eq!(catalog.name(), "TEST");
        assert_eq!(catalog.priority(), 1);
    }

    #[test]
    fn mount_fails_on_nonexistent_path() {
        // Validates: Requirement 5 AC 5
        let result = Catalog::mount(Path::new("/nonexistent"), 1);
        assert!(result.is_err());
    }

    #[test]
    fn allocate_ps_creates_file() {
        // Validates: Requirement 3 AC 2, Requirement 7 AC 2
        let (_tmp, catalog) = setup_catalog();
        let params = AllocParams {
            dsn: Dsn::parse("TEST.DATA.FILE").unwrap(),
            dsorg: Dsorg::PS,
            recfm: Some(Recfm::FB),
            lrecl: Some(80),
            blksize: Some(27920),
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
        };
        let record = catalog.allocate(params).unwrap();
        assert_eq!(record.dsorg, Dsorg::PS);
        let phys = catalog.repository().root().join(&record.storage_path);
        assert!(phys.exists());
        assert!(phys.is_file());
    }

    #[test]
    fn allocate_po_creates_directory() {
        // Validates: Requirement 3 AC 3, Requirement 7 AC 2
        let (_tmp, catalog) = setup_catalog();
        let params = AllocParams {
            dsn: Dsn::parse("SYS1.MACLIB").unwrap(),
            dsorg: Dsorg::PO,
            recfm: Some(Recfm::FB),
            lrecl: Some(80),
            blksize: Some(27920),
            dir_blocks: Some(10),
            gdg_limit: None,
            gdg_scratch: None,
            subtype: Some(PartitionedSubtype::PDS),
            description: None,
        };
        let record = catalog.allocate(params).unwrap();
        assert_eq!(record.dsorg, Dsorg::PO);
        let phys = catalog.repository().root().join(&record.storage_path);
        assert!(phys.exists());
        assert!(phys.is_dir());
    }

    #[test]
    fn allocate_duplicate_dsn_fails() {
        // Validates: Requirement 7 AC 3
        let (_tmp, catalog) = setup_catalog();
        let params = AllocParams {
            dsn: Dsn::parse("TEST.DUP").unwrap(),
            dsorg: Dsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
        };
        catalog.allocate(params.clone()).unwrap();
        let result = catalog.allocate(params);
        assert!(matches!(result, Err(CatalogError::DuplicateDataset { .. })));
    }

    #[test]
    fn delete_removes_entry_and_file() {
        // Validates: Requirement 7 AC 4
        let (_tmp, catalog) = setup_catalog();
        let dsn = Dsn::parse("DEL.ME").unwrap();
        let params = AllocParams {
            dsn: dsn.clone(),
            dsorg: Dsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
        };
        let record = catalog.allocate(params).unwrap();
        let phys = catalog.repository().root().join(&record.storage_path);
        assert!(phys.exists());

        catalog.delete(&dsn).unwrap();
        assert!(!phys.exists());
        assert!(catalog.lookup(&dsn).is_err());
    }

    #[test]
    fn rename_updates_dsn_and_path() {
        // Validates: Requirement 7 AC 6, AC 7
        let (_tmp, catalog) = setup_catalog();
        let old = Dsn::parse("OLD.NAME").unwrap();
        let new = Dsn::parse("NEW.NAME").unwrap();
        let params = AllocParams {
            dsn: old.clone(),
            dsorg: Dsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
        };
        catalog.allocate(params).unwrap();
        catalog.rename(&old, &new).unwrap();

        assert!(catalog.lookup(&old).is_err());
        let record = catalog.lookup(&new).unwrap();
        assert!(record.storage_path.contains("NEW"));
    }

    #[test]
    fn lookup_not_found_returns_error() {
        // Validates: Requirement 7 AC 9
        let (_tmp, catalog) = setup_catalog();
        let dsn = Dsn::parse("NONEXIST.DSN").unwrap();
        let result = catalog.lookup(&dsn);
        assert!(matches!(result, Err(CatalogError::DatasetNotFound { .. })));
    }
}
