//! SQLite-backed VSAM relative-record dataset storage.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::CatalogError;

use super::{ObjectId, ObjectStat, ProviderCapability, StorageProvider};

const RELATIVE_DIR: &str = "relative";

/// The state returned when reading an RRDS slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RrdsSlot {
    /// No record has been allocated at this relative record number.
    Unallocated,
    /// A record exists, including when its payload is empty.
    Allocated(Vec<u8>),
}

/// An allocated record returned by sequential iteration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RrdsRecord {
    pub record_number: u64,
    pub data: Vec<u8>,
}

/// A dedicated SQLite database containing one RRDS dataset.
pub struct SqliteRrdsProvider {
    database_path: PathBuf,
    dataset_id: Uuid,
    connection: Mutex<Connection>,
}

impl std::fmt::Debug for SqliteRrdsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteRrdsProvider")
            .field("database_path", &self.database_path)
            .field("dataset_id", &self.dataset_id)
            .finish_non_exhaustive()
    }
}

impl SqliteRrdsProvider {
    /// Open or create `<repository_root>/relative/<uuid>.sqlite`.
    pub fn open(repository_root: impl AsRef<Path>, dataset_id: Uuid) -> Result<Self, CatalogError> {
        let directory = repository_root.as_ref().join(RELATIVE_DIR);
        fs::create_dir_all(&directory).map_err(|source| CatalogError::IoError {
            operation: "open_rrds".into(),
            source,
        })?;
        let database_path = directory.join(format!("{dataset_id}.sqlite"));
        let connection = Connection::open(&database_path)
            .map_err(|source| sqlite_error("open_rrds", source))?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|source| sqlite_error("open_rrds", source))?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS RRDS_RECORDS (
                    RECNO INTEGER PRIMARY KEY,
                    RECORD_DATA BLOB NOT NULL,
                    ALLOCATED INTEGER NOT NULL DEFAULT 1 CHECK (ALLOCATED = 1)
                )",
            )
            .map_err(|source| sqlite_error("create_rrds_schema", source))?;
        Ok(Self {
            database_path,
            dataset_id,
            connection: Mutex::new(connection),
        })
    }

    pub fn open_existing(
        repository_root: impl AsRef<Path>,
        dataset_id: Uuid,
    ) -> Result<Self, CatalogError> {
        let path = repository_root
            .as_ref()
            .join(RELATIVE_DIR)
            .join(format!("{dataset_id}.sqlite"));
        if !path.is_file() {
            return Err(CatalogError::DatasetNotFound {
                dsn: dataset_id.to_string(),
                operation: "open_rrds".into(),
            });
        }
        Self::open(repository_root, dataset_id)
    }

    pub fn dataset_id(&self) -> Uuid {
        self.dataset_id
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    /// Read a slot without conflating an absent record with an allocated blank.
    pub fn read(&self, record_number: u64) -> Result<RrdsSlot, CatalogError> {
        validate_record_number(record_number)?;
        let connection = self.connection("read_rrds")?;
        let data = connection
            .query_row(
                "SELECT RECORD_DATA FROM RRDS_RECORDS WHERE RECNO = ?1 AND ALLOCATED = 1",
                params![record_number],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()
            .map_err(|source| sqlite_error("read_rrds", source))?;
        Ok(data.map_or(RrdsSlot::Unallocated, RrdsSlot::Allocated))
    }

    /// Allocate or replace a slot. Empty data is a valid allocated blank record.
    pub fn write(&self, record_number: u64, data: &[u8]) -> Result<(), CatalogError> {
        validate_record_number(record_number)?;
        let mut connection = self.connection("write_rrds")?;
        let transaction = connection
            .transaction()
            .map_err(|source| sqlite_error("write_rrds", source))?;
        transaction
            .execute(
                "INSERT INTO RRDS_RECORDS (RECNO, RECORD_DATA, ALLOCATED)
                 VALUES (?1, ?2, 1)
                 ON CONFLICT(RECNO) DO UPDATE SET RECORD_DATA = excluded.RECORD_DATA,
                                                   ALLOCATED = 1",
                params![record_number, data],
            )
            .map_err(|source| sqlite_error("write_rrds", source))?;
        transaction
            .commit()
            .map_err(|source| sqlite_error("write_rrds", source))
    }

    /// Delete a slot, returning it to the unallocated state.
    pub fn delete_record(&self, record_number: u64) -> Result<bool, CatalogError> {
        validate_record_number(record_number)?;
        let connection = self.connection("delete_rrds")?;
        connection
            .execute(
                "DELETE FROM RRDS_RECORDS WHERE RECNO = ?1",
                params![record_number],
            )
            .map(|count| count != 0)
            .map_err(|source| sqlite_error("delete_rrds", source))
    }

    /// Iterate allocated records in relative-record order.
    pub fn sequential_read(&self) -> Result<Vec<RrdsRecord>, CatalogError> {
        let connection = self.connection("sequential_rrds")?;
        let mut statement = connection
            .prepare(
                "SELECT RECNO, RECORD_DATA FROM RRDS_RECORDS
                 WHERE ALLOCATED = 1 ORDER BY RECNO",
            )
            .map_err(|source| sqlite_error("sequential_rrds", source))?;
        let rows = statement
            .query_map([], |row| {
                Ok(RrdsRecord {
                    record_number: row.get(0)?,
                    data: row.get(1)?,
                })
            })
            .map_err(|source| sqlite_error("sequential_rrds", source))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|source| sqlite_error("sequential_rrds", source))
    }

    fn connection(&self, operation: &str) -> Result<MutexGuard<'_, Connection>, CatalogError> {
        self.connection.lock().map_err(|_| CatalogError::RepositoryCorrupt {
            path: self.database_path.display().to_string(),
            reason: "RRDS connection mutex is poisoned".into(),
            operation: operation.into(),
        })
    }
}

impl StorageProvider for SqliteRrdsProvider {
    fn capabilities(&self) -> &[ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] =
            &[ProviderCapability::RecordRead, ProviderCapability::RecordWrite,
              ProviderCapability::RelativeAccess];
        CAPABILITIES
    }

    fn allocate(
        &self,
        _workspace_root: &Path,
        _is_container: bool,
    ) -> Result<(ObjectId, String), CatalogError> {
        Err(CatalogError::RepositoryCorrupt {
            path: self.database_path.display().to_string(),
            reason: "an RRDS provider must be opened with its dataset UUID".into(),
            operation: "allocate_rrds".into(),
        })
    }

    fn open(&self, workspace_root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
        let id = parse_locator(locator)?;
        let path = workspace_root
            .join(RELATIVE_DIR)
            .join(format!("{id}.sqlite"));
        if path.is_file() {
            Ok(path)
        } else {
            Err(CatalogError::DatasetNotFound {
                dsn: locator.into(),
                operation: "open_rrds".into(),
            })
        }
    }

    fn stat(&self, workspace_root: &Path, locator: &str) -> Result<ObjectStat, CatalogError> {
        let path = self.open(workspace_root, locator)?;
        let size = fs::metadata(&path)
            .map_err(|source| CatalogError::IoError { operation: "stat_rrds".into(), source })?
            .len();
        Ok(ObjectStat { size, is_container: false, locator: locator.into() })
    }

    fn rename(&self, _workspace_root: &Path, _locator: &str, _new_locator: &str) -> Result<(), CatalogError> {
        Ok(())
    }

    fn delete(&self, workspace_root: &Path, locator: &str) -> Result<(), CatalogError> {
        let path = self.open(workspace_root, locator)?;
        fs::remove_file(path).map_err(|source| CatalogError::IoError {
            operation: "delete_rrds".into(),
            source,
        })
    }

    fn list(&self, _workspace_root: &Path, _locator: &str) -> Result<Vec<String>, CatalogError> {
        Ok(Vec::new())
    }

    fn reconcile(&self, _workspace_root: &Path, _known_locators: &[String]) -> Result<Vec<String>, CatalogError> {
        Ok(Vec::new())
    }
}

fn validate_record_number(record_number: u64) -> Result<(), CatalogError> {
    if record_number == 0 {
        return Err(CatalogError::InvalidAllocationParams {
            reason: "RRDS relative record numbers start at 1".into(),
            operation: "rrds_record".into(),
        });
    }
    Ok(())
}

fn parse_locator(locator: &str) -> Result<Uuid, CatalogError> {
    Uuid::parse_str(locator).map_err(|_| CatalogError::RepositoryCorrupt {
        path: locator.into(),
        reason: "RRDS locator must be a UUID".into(),
        operation: "parse_rrds_locator".into(),
    })
}

fn sqlite_error(operation: &str, source: rusqlite::Error) -> CatalogError {
    CatalogError::SqliteError {
        operation: operation.into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn provider() -> (TempDir, SqliteRrdsProvider) {
        let directory = tempfile::tempdir().unwrap();
        let provider = SqliteRrdsProvider::open(directory.path(), Uuid::new_v4()).unwrap();
        (directory, provider)
    }

    #[test]
    fn distinguishes_unallocated_and_allocated_blank() {
        let (_directory, provider) = provider();
        assert_eq!(provider.read(4).unwrap(), RrdsSlot::Unallocated);
        provider.write(4, b"").unwrap();
        assert_eq!(provider.read(4).unwrap(), RrdsSlot::Allocated(Vec::new()));
    }

    #[test]
    fn writes_replaces_deletes_and_reads_in_order() {
        let (_directory, provider) = provider();
        provider.write(8, b"eight").unwrap();
        provider.write(2, b"two").unwrap();
        provider.write(8, b"updated").unwrap();
        assert_eq!(provider.read(8).unwrap(), RrdsSlot::Allocated(b"updated".to_vec()));
        assert!(provider.delete_record(2).unwrap());
        assert!(!provider.delete_record(2).unwrap());
        assert_eq!(
            provider.sequential_read().unwrap(),
            vec![RrdsRecord { record_number: 8, data: b"updated".to_vec() }]
        );
    }

    #[test]
    fn rejects_zero_record_number() {
        let (_directory, provider) = provider();
        assert!(provider.read(0).is_err());
        assert!(provider.write(0, b"x").is_err());
    }

    #[test]
    fn reopens_existing_database() {
        let (directory, provider) = provider();
        let id = provider.dataset_id();
        provider.write(1, b"value").unwrap();
        drop(provider);
        let reopened = SqliteRrdsProvider::open_existing(directory.path(), id).unwrap();
        assert_eq!(reopened.read(1).unwrap(), RrdsSlot::Allocated(b"value".to_vec()));
    }
}
