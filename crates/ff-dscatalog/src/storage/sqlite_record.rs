//! SQLite-backed record storage for keyed datasets.
//!
//! A record database is deliberately separate from `catalog.db`: the catalogue
//! stores the dataset and key metadata, while this provider stores only keyed
//! record payloads.  The database path is derived from the physical dataset UUID
//! and never from a dataset name.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::CatalogError;

use super::{ObjectId, ObjectStat, ProviderCapability, StorageProvider};

const INDEXED_DIR: &str = "indexed";
const RECORD_TABLE: &str = "KSDS_RECORDS";
const METADATA_TABLE: &str = "KSDS_METADATA";
const ALT_INDEX_REGISTRY: &str = "KSDS_ALT_INDEXES";

/// The key representation supported by the initial KSDS provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Keys are stored as SQLite `TEXT` values.
    Text,
}

/// Comparison rules for a KSDS primary key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCollation {
    /// Case-sensitive byte-wise SQLite ordering.
    Binary,
    /// SQLite's ASCII case-insensitive ordering.
    NoCase,
}

impl KeyCollation {
    fn sqlite_name(self) -> &'static str {
        match self {
            Self::Binary => "BINARY",
            Self::NoCase => "NOCASE",
        }
    }
}

/// Primary-key metadata for a KSDS.
///
/// `offset` and `length` describe the key field in a record payload.  The
/// provider accepts an explicit key for normal CRUD operations and also offers
/// [`SqliteRecordProvider::insert_record`] to derive it from these fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsdsKeyDefinition {
    pub offset: u32,
    pub length: u32,
    pub key_type: KeyType,
    pub collation: KeyCollation,
    pub unique: bool,
}

/// Descriptive alias used by catalogue callers.
pub type PrimaryKeyDefinition = KsdsKeyDefinition;
/// Short alias for callers that do not need the KSDS-specific name.
pub type KeyDefinition = KsdsKeyDefinition;

impl KsdsKeyDefinition {
    /// Construct a text primary-key definition with binary collation.
    pub fn new(offset: u32, length: u32) -> Self {
        Self {
            offset,
            length,
            key_type: KeyType::Text,
            collation: KeyCollation::Binary,
            unique: true,
        }
    }

    /// Set the key collation.
    pub fn with_collation(mut self, collation: KeyCollation) -> Self {
        self.collation = collation;
        self
    }

    /// Set the uniqueness metadata flag.
    ///
    /// SQLite KSDS tables always use the primary key as a unique constraint.
    /// The flag is retained as metadata so catalogue policy can be represented
    /// without weakening that invariant.
    pub fn with_unique(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }
}

/// A key and its record payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsdsRecord {
    pub key: String,
    pub data: Vec<u8>,
}

/// Definition of an alternate index on a KSDS dataset.
///
/// An alternate index maps a secondary key field (offset + length within the
/// record payload) to the primary key, enabling lookups by fields other than
/// the primary key.  Alternate indexes are stored as additional tables and
/// SQLite indexes within the same per-dataset SQLite database.
///
/// Validates: Requirement 21.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlternateIndex {
    /// Logical name for this alternate index (e.g. `"BY_SURNAME"`).
    pub name: String,
    /// Byte offset of the secondary key field within the record payload.
    pub offset: u32,
    /// Byte length of the secondary key field.
    pub length: u32,
    /// Whether the alternate key must be unique across all records.
    pub unique: bool,
    /// Collation rule applied to the alternate key.
    pub collation: KeyCollation,
}

/// A dedicated SQLite database containing records for one KSDS dataset.
pub struct SqliteRecordProvider {
    database_path: PathBuf,
    dataset_id: Uuid,
    key_definition: KsdsKeyDefinition,
    connection: Mutex<Connection>,
}

impl std::fmt::Debug for SqliteRecordProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SqliteRecordProvider")
            .field("database_path", &self.database_path)
            .field("dataset_id", &self.dataset_id)
            .field("key_definition", &self.key_definition)
            .finish_non_exhaustive()
    }
}

impl SqliteRecordProvider {
    /// Open or create the indexed database for `dataset_id`.
    ///
    /// The resulting path is `<repository_root>/indexed/<uuid>.sqlite`.
    /// Existing databases must be opened with the same key metadata that was
    /// recorded when they were created.
    pub fn open(
        repository_root: impl AsRef<Path>,
        dataset_id: Uuid,
        key_definition: KsdsKeyDefinition,
    ) -> Result<Self, CatalogError> {
        validate_key_definition(&key_definition)?;

        let indexed_dir = repository_root.as_ref().join(INDEXED_DIR);
        fs::create_dir_all(&indexed_dir).map_err(|source| CatalogError::IoError {
            operation: "open_ksds".to_string(),
            source,
        })?;
        let database_path = indexed_dir.join(format!("{dataset_id}.sqlite"));
        let connection =
            Connection::open(&database_path).map_err(|source| CatalogError::SqliteError {
                operation: "open_ksds".to_string(),
                source,
            })?;

        initialise_database(&connection, &key_definition)?;

        Ok(Self {
            database_path,
            dataset_id,
            key_definition,
            connection: Mutex::new(connection),
        })
    }

    /// Open an existing indexed database without creating its parent directory.
    pub fn open_existing(
        repository_root: impl AsRef<Path>,
        dataset_id: Uuid,
        key_definition: KsdsKeyDefinition,
    ) -> Result<Self, CatalogError> {
        let path = repository_root
            .as_ref()
            .join(INDEXED_DIR)
            .join(format!("{dataset_id}.sqlite"));
        if !path.is_file() {
            return Err(CatalogError::DatasetNotFound {
                dsn: dataset_id.to_string(),
                operation: "open_ksds".to_string(),
            });
        }
        Self::open(repository_root, dataset_id, key_definition)
    }

    /// The UUID identifying this physical record database.
    pub fn dataset_id(&self) -> Uuid {
        self.dataset_id
    }

    /// The physical SQLite path.
    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    /// The key metadata recorded for this KSDS.
    pub fn key_definition(&self) -> &KsdsKeyDefinition {
        &self.key_definition
    }

    /// Insert a keyed record, returning an error when the key already exists.
    pub fn insert(&self, key: &str, data: &[u8]) -> Result<(), CatalogError> {
        let mut connection = self.connection("insert")?;
        let transaction = connection
            .transaction()
            .map_err(|source| sqlite_error("insert", source))?;
        transaction
            .execute(
                "INSERT INTO KSDS_RECORDS (VSAM_KEY, RECORD_DATA) VALUES (?1, ?2)",
                params![key, data],
            )
            .map_err(|source| sqlite_error("insert", source))?;
        transaction
            .commit()
            .map_err(|source| sqlite_error("insert", source))
    }

    /// Derive the key from the configured record field and insert the record.
    pub fn insert_record(&self, data: &[u8]) -> Result<String, CatalogError> {
        let key = self.key_from_record(data)?;
        self.insert(&key, data)?;
        Ok(key)
    }

    /// Read a record by primary key.
    pub fn read(&self, key: &str) -> Result<Option<KsdsRecord>, CatalogError> {
        let connection = self.connection("read")?;
        connection
            .query_row(
                "SELECT VSAM_KEY, RECORD_DATA FROM KSDS_RECORDS WHERE VSAM_KEY = ?1",
                params![key],
                |row| {
                    Ok(KsdsRecord {
                        key: row.get(0)?,
                        data: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(|source| sqlite_error("read", source))
    }

    /// Alias for [`Self::read`] for callers using keyed-access terminology.
    pub fn read_key(&self, key: &str) -> Result<Option<KsdsRecord>, CatalogError> {
        self.read(key)
    }

    /// Update the payload for an existing key. Returns whether a row changed.
    pub fn update(&self, key: &str, data: &[u8]) -> Result<bool, CatalogError> {
        let mut connection = self.connection("update")?;
        let transaction = connection
            .transaction()
            .map_err(|source| sqlite_error("update", source))?;
        let changed = transaction
            .execute(
                "UPDATE KSDS_RECORDS SET RECORD_DATA = ?2 WHERE VSAM_KEY = ?1",
                params![key, data],
            )
            .map_err(|source| sqlite_error("update", source))?;
        transaction
            .commit()
            .map_err(|source| sqlite_error("update", source))?;
        Ok(changed != 0)
    }

    /// Delete a record by primary key. Returns whether a row was deleted.
    pub fn delete(&self, key: &str) -> Result<bool, CatalogError> {
        let mut connection = self.connection("delete")?;
        let transaction = connection
            .transaction()
            .map_err(|source| sqlite_error("delete", source))?;
        let deleted = transaction
            .execute("DELETE FROM KSDS_RECORDS WHERE VSAM_KEY = ?1", params![key])
            .map_err(|source| sqlite_error("delete", source))?;
        transaction
            .commit()
            .map_err(|source| sqlite_error("delete", source))?;
        Ok(deleted != 0)
    }

    /// Read all records in primary-key order.
    pub fn sequential_read(&self) -> Result<Vec<KsdsRecord>, CatalogError> {
        self.query_records(
            "SELECT VSAM_KEY, RECORD_DATA FROM KSDS_RECORDS ORDER BY VSAM_KEY",
            &[],
            "sequential_read",
        )
    }

    /// Alias for [`Self::sequential_read`].
    pub fn records(&self) -> Result<Vec<KsdsRecord>, CatalogError> {
        self.sequential_read()
    }

    /// Read records in the inclusive `[start, end]` primary-key range.
    pub fn range(&self, start: &str, end: &str) -> Result<Vec<KsdsRecord>, CatalogError> {
        self.query_records(
            "SELECT VSAM_KEY, RECORD_DATA FROM KSDS_RECORDS \
             WHERE VSAM_KEY >= ?1 AND VSAM_KEY <= ?2 ORDER BY VSAM_KEY",
            &[&start, &end],
            "range_read",
        )
    }

    /// Read a range with independently optional inclusive bounds.
    pub fn range_opt(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<KsdsRecord>, CatalogError> {
        match (start, end) {
            (Some(start), Some(end)) => self.range(start, end),
            (Some(start), None) => self.query_records(
                "SELECT VSAM_KEY, RECORD_DATA FROM KSDS_RECORDS \
                 WHERE VSAM_KEY >= ?1 ORDER BY VSAM_KEY",
                &[&start],
                "range_read",
            ),
            (None, Some(end)) => self.query_records(
                "SELECT VSAM_KEY, RECORD_DATA FROM KSDS_RECORDS \
                 WHERE VSAM_KEY <= ?1 ORDER BY VSAM_KEY",
                &[&end],
                "range_read",
            ),
            (None, None) => self.sequential_read(),
        }
    }

    // === Alternate index operations =====================================

    /// Register an alternate index on this KSDS.
    ///
    /// Creates a mapping table `AIX_<NAME>` (columns `ALT_KEY`, `PRIMARY_KEY`)
    /// and a SQLite index on `ALT_KEY`.  The registry table records the index
    /// definition so it can be reconstructed on reopen.
    ///
    /// Validates: Requirement 21.5
    pub fn add_alternate_index(&self, definition: &AlternateIndex) -> Result<(), CatalogError> {
        validate_alternate_index(definition)?;
        let table = aix_table_name(&definition.name);
        let mut connection = self.connection("add_alternate_index")?;
        let transaction = connection
            .transaction()
            .map_err(|source| sqlite_error("add_alternate_index", source))?;
        transaction
            .execute_batch(&format!(
                "CREATE TABLE IF NOT EXISTS {table} (
                     ALT_KEY  TEXT COLLATE {collation} NOT NULL,
                     PRIMARY_KEY TEXT NOT NULL,
                     CONSTRAINT {table}_pk UNIQUE (ALT_KEY, PRIMARY_KEY)
                 );
                 CREATE INDEX IF NOT EXISTS {table}_idx ON {table} (ALT_KEY);",
                collation = definition.collation.sqlite_name(),
            ))
            .map_err(|source| sqlite_error("add_alternate_index", source))?;
        transaction
            .execute(
                &format!(
                    "INSERT OR IGNORE INTO {ALT_INDEX_REGISTRY} \
                     (INDEX_NAME, KEY_OFFSET, KEY_LENGTH, UNIQUE_KEYS, COLLATION) \
                     VALUES (?1, ?2, ?3, ?4, ?5)"
                ),
                params![
                    definition.name,
                    definition.offset,
                    definition.length,
                    definition.unique as i32,
                    definition.collation.sqlite_name(),
                ],
            )
            .map_err(|source| sqlite_error("add_alternate_index", source))?;
        transaction
            .commit()
            .map_err(|source| sqlite_error("add_alternate_index", source))
    }

    /// Populate an alternate index from the current record set.
    ///
    /// Extracts the secondary key from each record and inserts a mapping row
    /// into the alternate index table.  Call this after `add_alternate_index`
    /// to back-fill an index on an existing dataset.
    ///
    /// Validates: Requirement 21.5
    pub fn rebuild_alternate_index(&self, name: &str) -> Result<(), CatalogError> {
        let definition = self
            .list_alternate_indexes()?
            .into_iter()
            .find(|aix| aix.name == name)
            .ok_or_else(|| CatalogError::InvalidAllocationParams {
                reason: format!("alternate index '{name}' not found"),
                operation: "rebuild_alternate_index".to_string(),
            })?;
        let records = self.sequential_read()?;
        let table = aix_table_name(name);
        let mut connection = self.connection("rebuild_alternate_index")?;
        let transaction = connection
            .transaction()
            .map_err(|source| sqlite_error("rebuild_alternate_index", source))?;
        transaction
            .execute(&format!("DELETE FROM {table}"), [])
            .map_err(|source| sqlite_error("rebuild_alternate_index", source))?;
        for record in &records {
            let alt_key = extract_key_field(&record.data, definition.offset, definition.length)?;
            transaction
                .execute(
                    &format!(
                        "INSERT OR IGNORE INTO {table} (ALT_KEY, PRIMARY_KEY) VALUES (?1, ?2)"
                    ),
                    params![alt_key, record.key],
                )
                .map_err(|source| sqlite_error("rebuild_alternate_index", source))?;
        }
        transaction
            .commit()
            .map_err(|source| sqlite_error("rebuild_alternate_index", source))
    }

    /// Look up primary keys via an alternate index.
    ///
    /// Returns all primary keys whose alternate key field equals `alt_key`.
    /// For unique alternate indexes this returns at most one entry.
    ///
    /// Validates: Requirement 21.5
    pub fn lookup_by_alternate_key(
        &self,
        index_name: &str,
        alt_key: &str,
    ) -> Result<Vec<String>, CatalogError> {
        let table = aix_table_name(index_name);
        let connection = self.connection("lookup_by_alternate_key")?;
        let mut statement = connection
            .prepare(&format!(
                "SELECT PRIMARY_KEY FROM {table} WHERE ALT_KEY = ?1 ORDER BY PRIMARY_KEY"
            ))
            .map_err(|source| sqlite_error("lookup_by_alternate_key", source))?;
        let rows = statement
            .query_map(params![alt_key], |row| row.get(0))
            .map_err(|source| sqlite_error("lookup_by_alternate_key", source))?;
        rows.map(|row| row.map_err(|source| sqlite_error("lookup_by_alternate_key", source)))
            .collect()
    }

    /// List all alternate indexes registered on this KSDS.
    ///
    /// Validates: Requirement 21.5
    pub fn list_alternate_indexes(&self) -> Result<Vec<AlternateIndex>, CatalogError> {
        let connection = self.connection("list_alternate_indexes")?;
        let mut statement = connection
            .prepare(&format!(
                "SELECT INDEX_NAME, KEY_OFFSET, KEY_LENGTH, UNIQUE_KEYS, COLLATION \
                     FROM {ALT_INDEX_REGISTRY} ORDER BY INDEX_NAME"
            ))
            .map_err(|source| sqlite_error("list_alternate_indexes", source))?;
        let rows = statement
            .query_map([], |row| {
                let collation_str: String = row.get(4)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, u32>(2)?,
                    row.get::<_, i32>(3)?,
                    collation_str,
                ))
            })
            .map_err(|source| sqlite_error("list_alternate_indexes", source))?;
        rows.map(|row| {
            let (name, offset, length, unique_flag, collation_str) =
                row.map_err(|source| sqlite_error("list_alternate_indexes", source))?;
            let collation = if collation_str == "NOCASE" {
                KeyCollation::NoCase
            } else {
                KeyCollation::Binary
            };
            Ok(AlternateIndex {
                name,
                offset,
                length,
                unique: unique_flag != 0,
                collation,
            })
        })
        .collect()
    }

    // === Record count helpers =============================================

    /// Number of records currently stored.
    pub fn len(&self) -> Result<usize, CatalogError> {
        let connection = self.connection("count")?;
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM KSDS_RECORDS", [], |row| row.get(0))
            .map_err(|source| sqlite_error("count", source))?;
        Ok(count as usize)
    }

    /// Whether the record database contains no records.
    pub fn is_empty(&self) -> Result<bool, CatalogError> {
        Ok(self.len()? == 0)
    }

    fn key_from_record(&self, data: &[u8]) -> Result<String, CatalogError> {
        extract_key_field(data, self.key_definition.offset, self.key_definition.length)
    }

    fn connection(
        &self,
        operation: &'static str,
    ) -> Result<MutexGuard<'_, Connection>, CatalogError> {
        self.connection
            .lock()
            .map_err(|_| CatalogError::RepositoryCorrupt {
                path: self.database_path.display().to_string(),
                reason: "record database lock was poisoned".to_string(),
                operation: operation.to_string(),
            })
    }

    fn query_records(
        &self,
        sql: &str,
        values: &[&dyn rusqlite::ToSql],
        operation: &'static str,
    ) -> Result<Vec<KsdsRecord>, CatalogError> {
        let connection = self.connection(operation)?;
        let mut statement = connection
            .prepare(sql)
            .map_err(|source| sqlite_error(operation, source))?;
        let rows = statement
            .query_map(values, |row| {
                Ok(KsdsRecord {
                    key: row.get(0)?,
                    data: row.get(1)?,
                })
            })
            .map_err(|source| sqlite_error(operation, source))?;
        rows.map(|row| row.map_err(|source| sqlite_error(operation, source)))
            .collect()
    }
}

fn extract_key_field(data: &[u8], offset: u32, length: u32) -> Result<String, CatalogError> {
    let start = offset as usize;
    let end = start
        .checked_add(length as usize)
        .ok_or_else(|| invalid_key("key field overflows record bounds"))?;
    let field = data
        .get(start..end)
        .ok_or_else(|| invalid_key("record is shorter than the configured key field"))?;
    String::from_utf8(field.to_vec())
        .map_err(|_| invalid_key("configured text key field is not valid UTF-8"))
}

fn validate_key_definition(definition: &KsdsKeyDefinition) -> Result<(), CatalogError> {
    if definition.length == 0 {
        return Err(invalid_key("key length must be greater than zero"));
    }
    if definition.key_type != KeyType::Text {
        return Err(invalid_key("only text primary keys are supported"));
    }
    Ok(())
}

fn validate_alternate_index(definition: &AlternateIndex) -> Result<(), CatalogError> {
    if definition.name.is_empty() || definition.name.len() > 30 {
        return Err(CatalogError::InvalidAllocationParams {
            reason: "alternate index name must be 1-30 characters".to_string(),
            operation: "add_alternate_index".to_string(),
        });
    }
    if !definition
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(CatalogError::InvalidAllocationParams {
            reason: "alternate index name must contain only ASCII alphanumeric characters or underscores".to_string(),
            operation: "add_alternate_index".to_string(),
        });
    }
    if definition.length == 0 {
        return Err(CatalogError::InvalidAllocationParams {
            reason: "alternate index key length must be greater than zero".to_string(),
            operation: "add_alternate_index".to_string(),
        });
    }
    Ok(())
}

/// Derive the SQLite table name for an alternate index.
fn aix_table_name(name: &str) -> String {
    format!("AIX_{}", name.to_ascii_uppercase())
}

fn initialise_database(
    connection: &Connection,
    definition: &KsdsKeyDefinition,
) -> Result<(), CatalogError> {
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|source| sqlite_error("initialize_ksds", source))?;
    connection
        .execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS {METADATA_TABLE} (
                 METADATA_KEY TEXT PRIMARY KEY NOT NULL,
                 METADATA_VALUE TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS {RECORD_TABLE} (
                 VSAM_KEY TEXT PRIMARY KEY COLLATE {} NOT NULL,
                 RECORD_DATA BLOB NOT NULL
             );
             CREATE TABLE IF NOT EXISTS {ALT_INDEX_REGISTRY} (
                 INDEX_NAME  TEXT PRIMARY KEY NOT NULL,
                 KEY_OFFSET  INTEGER NOT NULL,
                 KEY_LENGTH  INTEGER NOT NULL,
                 UNIQUE_KEYS INTEGER NOT NULL DEFAULT 0,
                 COLLATION   TEXT NOT NULL DEFAULT 'BINARY'
             );",
            definition.collation.sqlite_name()
        ))
        .map_err(|source| sqlite_error("initialize_ksds", source))?;

    let metadata = [
        ("key_offset", definition.offset.to_string()),
        ("key_length", definition.length.to_string()),
        (
            "key_type",
            match definition.key_type {
                KeyType::Text => "TEXT".to_string(),
            },
        ),
        (
            "key_collation",
            definition.collation.sqlite_name().to_string(),
        ),
        (
            "key_unique",
            if definition.unique { "1" } else { "0" }.to_string(),
        ),
    ];
    for (key, value) in metadata {
        connection
            .execute(
                "INSERT OR IGNORE INTO KSDS_METADATA (METADATA_KEY, METADATA_VALUE)
                 VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|source| sqlite_error("initialize_ksds", source))?;
    }

    let stored = |key: &str| -> Result<String, CatalogError> {
        connection
            .query_row(
                "SELECT METADATA_VALUE FROM KSDS_METADATA WHERE METADATA_KEY = ?1",
                params![key],
                |row| row.get(0),
            )
            .map_err(|source| sqlite_error("read_ksds_metadata", source))
    };
    let expected = [
        ("key_offset", definition.offset.to_string()),
        ("key_length", definition.length.to_string()),
        ("key_type", "TEXT".to_string()),
        (
            "key_collation",
            definition.collation.sqlite_name().to_string(),
        ),
        (
            "key_unique",
            if definition.unique { "1" } else { "0" }.to_string(),
        ),
    ];
    for (key, expected_value) in expected {
        if stored(key)? != expected_value {
            return Err(invalid_key("key metadata does not match the existing KSDS"));
        }
    }
    Ok(())
}

fn sqlite_error(operation: &str, source: rusqlite::Error) -> CatalogError {
    CatalogError::SqliteError {
        operation: operation.to_string(),
        source,
    }
}

fn invalid_key(reason: &str) -> CatalogError {
    CatalogError::InvalidAllocationParams {
        reason: reason.to_string(),
        operation: "configure_ksds_key".to_string(),
    }
}

fn indexed_path(root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
    let path = Path::new(locator);
    let mut components = path.components();
    if components.next() != Some(std::path::Component::Normal("indexed".as_ref()))
        || components.next().is_none()
        || components.next().is_some()
    {
        return Err(CatalogError::RepositoryCorrupt {
            path: locator.to_string(),
            reason: "invalid indexed record locator".to_string(),
            operation: "resolve_ksds_path".to_string(),
        });
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let uuid_text = file_name.strip_suffix(".sqlite").unwrap_or_default();
    if Uuid::parse_str(uuid_text).is_err() {
        return Err(CatalogError::RepositoryCorrupt {
            path: locator.to_string(),
            reason: "indexed record locator must contain a UUID".to_string(),
            operation: "resolve_ksds_path".to_string(),
        });
    }
    Ok(root.join(path))
}

impl StorageProvider for SqliteRecordProvider {
    fn capabilities(&self) -> &[ProviderCapability] {
        static CAPABILITIES: [ProviderCapability; 3] = [
            ProviderCapability::RecordRead,
            ProviderCapability::RecordWrite,
            ProviderCapability::KeyedAccess,
        ];
        &CAPABILITIES
    }

    fn allocate(
        &self,
        workspace_root: &Path,
        _is_container: bool,
    ) -> Result<(ObjectId, String), CatalogError> {
        let id = Uuid::new_v4();
        let definition = KsdsKeyDefinition::new(0, 1);
        Self::open(workspace_root, id, definition)?;
        Ok((id, format!("{INDEXED_DIR}/{id}.sqlite")))
    }

    fn open(&self, workspace_root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
        let path = indexed_path(workspace_root, locator)?;
        if !path.is_file() {
            return Err(CatalogError::DatasetNotFound {
                dsn: locator.to_string(),
                operation: "open_ksds".to_string(),
            });
        }
        Ok(path)
    }

    fn stat(&self, workspace_root: &Path, locator: &str) -> Result<ObjectStat, CatalogError> {
        let path = indexed_path(workspace_root, locator)?;
        let metadata = fs::metadata(&path).map_err(|source| CatalogError::IoError {
            operation: "stat_ksds".to_string(),
            source,
        })?;
        Ok(ObjectStat {
            size: metadata.len(),
            is_container: false,
            locator: locator.to_string(),
        })
    }

    fn rename(
        &self,
        _workspace_root: &Path,
        _locator: &str,
        _new_locator: &str,
    ) -> Result<(), CatalogError> {
        Ok(())
    }

    fn delete(&self, workspace_root: &Path, locator: &str) -> Result<(), CatalogError> {
        let path = indexed_path(workspace_root, locator)?;
        if path.exists() {
            fs::remove_file(path).map_err(|source| CatalogError::IoError {
                operation: "delete_ksds".to_string(),
                source,
            })?;
        }
        Ok(())
    }

    fn list(&self, workspace_root: &Path, _locator: &str) -> Result<Vec<String>, CatalogError> {
        let indexed_dir = workspace_root.join(INDEXED_DIR);
        if !indexed_dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut locators = Vec::new();
        for entry in fs::read_dir(indexed_dir).map_err(|source| CatalogError::IoError {
            operation: "list_ksds".to_string(),
            source,
        })? {
            let entry = entry.map_err(|source| CatalogError::IoError {
                operation: "list_ksds".to_string(),
                source,
            })?;
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("sqlite") {
                locators.push(format!(
                    "{INDEXED_DIR}/{}",
                    entry.file_name().to_string_lossy()
                ));
            }
        }
        locators.sort();
        Ok(locators)
    }

    fn reconcile(
        &self,
        workspace_root: &Path,
        known_locators: &[String],
    ) -> Result<Vec<String>, CatalogError> {
        let mut discrepancies = Vec::new();
        for locator in known_locators {
            match indexed_path(workspace_root, locator) {
                Ok(path) if !path.is_file() => {
                    discrepancies.push(format!("missing physical object for locator '{locator}'"));
                }
                Err(error) => discrepancies.push(format!("invalid locator '{locator}': {error}")),
                Ok(_) => {}
            }
        }
        Ok(discrepancies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn provider() -> (TempDir, SqliteRecordProvider) {
        let directory = tempfile::tempdir().expect("tempdir");
        let id = Uuid::new_v4();
        let provider =
            SqliteRecordProvider::open(directory.path(), id, KsdsKeyDefinition::new(0, 1))
                .expect("open provider");
        (directory, provider)
    }

    #[test]
    fn creates_indexed_database_with_wal_and_schema() {
        let (_directory, provider) = provider();
        assert!(provider.database_path().is_file());
        let connection = Connection::open(provider.database_path()).unwrap();
        let journal_mode: String = connection
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
        let columns: Vec<String> = connection
            .prepare("PRAGMA table_info(KSDS_RECORDS)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(columns, ["VSAM_KEY", "RECORD_DATA"]);
    }

    #[test]
    fn supports_keyed_crud_and_ordered_ranges() {
        let (_directory, provider) = provider();
        provider.insert("C", b"third").unwrap();
        provider.insert("A", b"first").unwrap();
        provider.insert("B", b"second").unwrap();
        assert_eq!(provider.read("B").unwrap().unwrap().data, b"second");
        assert_eq!(
            provider
                .sequential_read()
                .unwrap()
                .into_iter()
                .map(|record| record.key)
                .collect::<Vec<_>>(),
            ["A", "B", "C"]
        );
        assert_eq!(
            provider
                .range("A", "B")
                .unwrap()
                .into_iter()
                .map(|record| record.key)
                .collect::<Vec<_>>(),
            ["A", "B"]
        );
        assert!(provider.update("B", b"changed").unwrap());
        assert_eq!(provider.read("B").unwrap().unwrap().data, b"changed");
        assert!(provider.delete("B").unwrap());
        assert!(provider.read("B").unwrap().is_none());
    }

    #[test]
    fn primary_key_uniqueness_is_transactional() {
        let (_directory, provider) = provider();
        provider.insert("A", b"one").unwrap();
        let error = provider.insert("A", b"two").unwrap_err();
        assert!(matches!(error, CatalogError::SqliteError { .. }));
        assert_eq!(provider.read("A").unwrap().unwrap().data, b"one");
    }

    #[test]
    fn metadata_survives_reopen_and_mismatches_are_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let definition = KsdsKeyDefinition::new(2, 3).with_collation(KeyCollation::NoCase);
        let provider =
            SqliteRecordProvider::open(directory.path(), id, definition.clone()).unwrap();
        drop(provider);
        let reopened =
            SqliteRecordProvider::open_existing(directory.path(), id, definition).unwrap();
        assert_eq!(reopened.key_definition().collation, KeyCollation::NoCase);
        let mismatch = SqliteRecordProvider::open_existing(
            directory.path(),
            id,
            KsdsKeyDefinition::new(0, 3).with_collation(KeyCollation::NoCase),
        );
        assert!(matches!(
            mismatch,
            Err(CatalogError::InvalidAllocationParams { .. })
        ));
    }

    #[test]
    fn insert_record_uses_configured_key_field() {
        let directory = tempfile::tempdir().unwrap();
        let provider = SqliteRecordProvider::open(
            directory.path(),
            Uuid::new_v4(),
            KsdsKeyDefinition::new(2, 3),
        )
        .unwrap();
        let key = provider.insert_record(b"XXKEYpayload").unwrap();
        assert_eq!(key, "KEY");
        assert!(provider.read("KEY").unwrap().is_some());
    }

    // === Alternate index tests (Validates: Requirement 21.5) =============

    #[test]
    fn alternate_index_is_registered_and_listed() {
        // Validates: Requirement 21.5
        let (_dir, provider) = provider();
        let aix = AlternateIndex {
            name: "BY_DEPT".to_string(),
            offset: 4,
            length: 3,
            unique: false,
            collation: KeyCollation::Binary,
        };
        provider.add_alternate_index(&aix).unwrap();
        let indexes = provider.list_alternate_indexes().unwrap();
        assert_eq!(indexes.len(), 1);
        assert_eq!(indexes[0].name, "BY_DEPT");
        assert_eq!(indexes[0].offset, 4);
        assert_eq!(indexes[0].length, 3);
        assert!(!indexes[0].unique);
    }

    #[test]
    fn alternate_index_lookup_returns_matching_primary_keys() {
        // Validates: Requirement 21.5
        let (_dir, provider) = provider();
        // Record layout: [primary_key(1)] [padding(3)] [dept(3)] [rest]
        // primary key is at offset 0, length 1 (from provider() fixture)
        provider.insert("A", b"AXXX001rest").unwrap();
        provider.insert("B", b"BXXX001rest").unwrap();
        provider.insert("C", b"CXXX002rest").unwrap();
        let aix = AlternateIndex {
            name: "BY_DEPT".to_string(),
            offset: 4,
            length: 3,
            unique: false,
            collation: KeyCollation::Binary,
        };
        provider.add_alternate_index(&aix).unwrap();
        provider.rebuild_alternate_index("BY_DEPT").unwrap();
        let mut keys = provider.lookup_by_alternate_key("BY_DEPT", "001").unwrap();
        keys.sort();
        assert_eq!(keys, ["A", "B"]);
        let keys_002 = provider.lookup_by_alternate_key("BY_DEPT", "002").unwrap();
        assert_eq!(keys_002, ["C"]);
    }

    #[test]
    fn alternate_index_rejects_invalid_name() {
        // Validates: Requirement 21.5
        let (_dir, provider) = provider();
        let bad = AlternateIndex {
            name: "".to_string(),
            offset: 0,
            length: 1,
            unique: false,
            collation: KeyCollation::Binary,
        };
        assert!(matches!(
            provider.add_alternate_index(&bad),
            Err(CatalogError::InvalidAllocationParams { .. })
        ));
    }

    #[test]
    fn alternate_index_survives_reopen() {
        // Validates: Requirement 21.5
        let directory = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let definition = KsdsKeyDefinition::new(0, 1);
        {
            let provider =
                SqliteRecordProvider::open(directory.path(), id, definition.clone()).unwrap();
            provider
                .add_alternate_index(&AlternateIndex {
                    name: "BY_CODE".to_string(),
                    offset: 1,
                    length: 2,
                    unique: true,
                    collation: KeyCollation::NoCase,
                })
                .unwrap();
        }
        let reopened =
            SqliteRecordProvider::open_existing(directory.path(), id, definition).unwrap();
        let indexes = reopened.list_alternate_indexes().unwrap();
        assert_eq!(indexes.len(), 1);
        assert_eq!(indexes[0].name, "BY_CODE");
        assert!(indexes[0].unique);
        assert_eq!(indexes[0].collation, KeyCollation::NoCase);
    }
}
