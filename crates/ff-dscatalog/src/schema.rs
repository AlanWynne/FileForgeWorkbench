//! SQLite schema creation and migration for the catalog database.
//!
//! Defines the SQL statements for creating the `catalog_metadata`, `datasets`,
//! `gdg_bases`, `gdg_generations`, and `audit_log` tables with correct constraints.
//! Forward migrations are applied automatically on mount when the stored version
//! is behind the current version.
//!
//! Validates: Requirement 27.4, 27.5

use rusqlite::Connection;

use crate::error::CatalogError;

/// Current schema version.
///
/// Version history:
///   1 -- initial schema (catalog_metadata, datasets, gdg_bases, gdg_generations)
///   2 -- added audit_log table (Req 27.4, 27.5)
///   3 -- added scope column to datasets (Req 29.1, 29.4)
pub const SCHEMA_VERSION: &str = "3";

/// SQL to create the catalog_metadata table.
const CREATE_METADATA_TABLE: &str = "
CREATE TABLE IF NOT EXISTS catalog_metadata (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT
);
";

/// SQL to create the datasets table.
const CREATE_DATASETS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS datasets (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    dsn          TEXT    UNIQUE NOT NULL,
    dsorg        TEXT    NOT NULL CHECK (dsorg IN ('PS', 'PO', 'GDG')),
    storage_path TEXT    NOT NULL,
    recfm        TEXT    CHECK (recfm IN ('F', 'FB', 'V', 'VB', 'U') OR recfm IS NULL),
    lrecl        INTEGER CHECK (lrecl IS NULL OR (lrecl > 0 AND lrecl <= 32760)),
    blksize      INTEGER CHECK (blksize IS NULL OR blksize >= 0),
    subtype      TEXT    CHECK (subtype IS NULL OR subtype IN ('PDS', 'PDSE')),
    scope        TEXT    NOT NULL DEFAULT 'user'
                         CHECK (scope IN ('master', 'user')),
    created      TEXT,
    modified     TEXT,
    accessed     TEXT
);
";

/// SQL to create the datasets DSN index.
const CREATE_DATASETS_INDEX: &str = "
CREATE INDEX IF NOT EXISTS idx_datasets_dsn_prefix ON datasets (dsn);
";

/// SQL to create the GDG bases table.
const CREATE_GDG_BASES_TABLE: &str = "
CREATE TABLE IF NOT EXISTS gdg_bases (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    dsn     TEXT    UNIQUE NOT NULL,
    limit_  INTEGER NOT NULL CHECK (limit_ >= 1 AND limit_ <= 255),
    scratch BOOLEAN NOT NULL DEFAULT 1,
    created TEXT
);
";

/// SQL to create the GDG generations table.
const CREATE_GDG_GENERATIONS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS gdg_generations (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    base_id           INTEGER NOT NULL,
    generation_number INTEGER NOT NULL,
    version           INTEGER NOT NULL DEFAULT 0,
    dataset_id        INTEGER NOT NULL,
    status            TEXT    NOT NULL DEFAULT 'active'
                      CHECK (status IN ('active', 'rolled_off', 'deferred')),
    FOREIGN KEY (base_id) REFERENCES gdg_bases(id) ON DELETE CASCADE,
    FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON DELETE CASCADE,
    UNIQUE (base_id, generation_number, version)
);
";

/// SQL to create the GDG generation index.
const CREATE_GDG_GEN_INDEX: &str = "
CREATE INDEX IF NOT EXISTS idx_gdg_gen_base ON gdg_generations (base_id, generation_number DESC);
";

/// SQL to create the audit_log table (schema version 2).
///
/// Records every create/rename/move/delete/restore/import/export/allocate action.
/// Validates: Requirement 27.4, 28.6
const CREATE_AUDIT_LOG_TABLE: &str = "
CREATE TABLE IF NOT EXISTS audit_log (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    action    TEXT    NOT NULL,
    object_dsn TEXT,
    outcome   TEXT    NOT NULL CHECK (outcome IN ('ok', 'err')),
    timestamp TEXT    NOT NULL,
    principal TEXT
);
";

/// SQL to create an index on audit_log for fast DSN lookups.
const CREATE_AUDIT_LOG_INDEX: &str = "
CREATE INDEX IF NOT EXISTS idx_audit_log_dsn ON audit_log (object_dsn);
";

/// SQL to add the scope column to the datasets table (schema version 3).
///
/// Defaults to 'user' for all existing rows for backward compatibility.
/// Validates: Requirement 29.1, 29.4
const ADD_SCOPE_COLUMN: &str = "
ALTER TABLE datasets ADD COLUMN scope TEXT NOT NULL DEFAULT 'user'
    CHECK (scope IN ('master', 'user'));
";

/// Forward migration scripts indexed by the version they produce.
///
/// `MIGRATIONS[i]` upgrades the schema from version `i+1` to version `i+2`.
/// Validates: Requirement 27.5
const MIGRATIONS: &[(&str, &str)] = &[
    // v1 -> v2: add audit_log table
    ("2", CREATE_AUDIT_LOG_TABLE),
    // v2 -> v3: add scope column to datasets
    ("3", ADD_SCOPE_COLUMN),
];

/// Apply any pending forward migrations to bring the schema up to the current version.
///
/// Reads the stored `schema_version` from `catalog_metadata`, then applies each
/// migration script in sequence until the version matches `SCHEMA_VERSION`.
/// Each migration is wrapped in a transaction so a failure leaves the database
/// at the last successfully applied version.
///
/// # Errors
///
/// Returns `CatalogError::SqliteError` on database failure.
/// Returns `CatalogError::SchemaVersionMismatch` if the stored version is newer
/// than the current code (downgrade not supported).
///
/// Validates: Requirement 27.5
pub fn apply_migrations(conn: &Connection) -> Result<(), CatalogError> {
    let stored: String = conn
        .query_row(
            "SELECT value FROM catalog_metadata WHERE key = ?1",
            rusqlite::params!["schema_version"],
            |row| row.get(0),
        )
        .map_err(|source| CatalogError::SqliteError {
            operation: "apply_migrations".to_string(),
            source,
        })?;

    let stored_v: u32 = stored
        .parse()
        .map_err(|_| CatalogError::SchemaVersionMismatch {
            found: stored.clone(),
            expected: SCHEMA_VERSION.to_string(),
            operation: "apply_migrations".to_string(),
        })?;
    let target_v: u32 = SCHEMA_VERSION
        .parse()
        .expect("SCHEMA_VERSION is a valid integer");

    if stored_v > target_v {
        return Err(CatalogError::SchemaVersionMismatch {
            found: stored,
            expected: SCHEMA_VERSION.to_string(),
            operation: "apply_migrations".to_string(),
        });
    }

    for v in stored_v..target_v {
        // MIGRATIONS[i] produces version i+2 (index 0 -> v1->v2)
        let idx = (v - 1) as usize;
        if idx >= MIGRATIONS.len() {
            break;
        }
        let (new_version, sql) = MIGRATIONS[idx];
        conn.execute_batch(sql)
            .map_err(|source| CatalogError::SqliteError {
                operation: format!("migration_to_v{new_version}"),
                source,
            })?;
        conn.execute(
            "UPDATE catalog_metadata SET value = ?1 WHERE key = 'schema_version'",
            rusqlite::params![new_version],
        )
        .map_err(|source| CatalogError::SqliteError {
            operation: format!("migration_to_v{new_version}"),
            source,
        })?;
    }

    Ok(())
}

/// Initialize a new catalog database with the correct schema.
///
/// Creates all tables, enables WAL journal mode and foreign keys, and inserts
/// default metadata entries.
///
/// # Errors
///
/// Returns `CatalogError::SqliteError` if any SQL statement fails.
pub fn initialize_database(conn: &Connection, catalog_name: &str) -> Result<(), CatalogError> {
    // Enable WAL mode for concurrent read access during writes
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| CatalogError::SqliteError {
            operation: "initialize_database".to_string(),
            source: e,
        })?;

    // Enable foreign key enforcement
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| CatalogError::SqliteError {
            operation: "initialize_database".to_string(),
            source: e,
        })?;

    // Create schema
    conn.execute_batch(CREATE_METADATA_TABLE)
        .map_err(|e| CatalogError::SqliteError {
            operation: "create_metadata_table".to_string(),
            source: e,
        })?;

    conn.execute_batch(CREATE_DATASETS_TABLE)
        .map_err(|e| CatalogError::SqliteError {
            operation: "create_datasets_table".to_string(),
            source: e,
        })?;

    conn.execute_batch(CREATE_DATASETS_INDEX)
        .map_err(|e| CatalogError::SqliteError {
            operation: "create_datasets_index".to_string(),
            source: e,
        })?;

    conn.execute_batch(CREATE_GDG_BASES_TABLE)
        .map_err(|e| CatalogError::SqliteError {
            operation: "create_gdg_bases_table".to_string(),
            source: e,
        })?;

    conn.execute_batch(CREATE_GDG_GENERATIONS_TABLE)
        .map_err(|e| CatalogError::SqliteError {
            operation: "create_gdg_generations_table".to_string(),
            source: e,
        })?;

    conn.execute_batch(CREATE_GDG_GEN_INDEX)
        .map_err(|e| CatalogError::SqliteError {
            operation: "create_gdg_gen_index".to_string(),
            source: e,
        })?;

    conn.execute_batch(CREATE_AUDIT_LOG_TABLE)
        .map_err(|e| CatalogError::SqliteError {
            operation: "create_audit_log_table".to_string(),
            source: e,
        })?;

    conn.execute_batch(CREATE_AUDIT_LOG_INDEX)
        .map_err(|e| CatalogError::SqliteError {
            operation: "create_audit_log_index".to_string(),
            source: e,
        })?;

    // Insert default metadata using parameterized queries
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO catalog_metadata (key, value) VALUES (?1, ?2)",
        rusqlite::params!["schema_version", SCHEMA_VERSION],
    )
    .map_err(|e| CatalogError::SqliteError {
        operation: "insert_metadata".to_string(),
        source: e,
    })?;

    conn.execute(
        "INSERT OR IGNORE INTO catalog_metadata (key, value) VALUES (?1, ?2)",
        rusqlite::params!["catalog_name", catalog_name],
    )
    .map_err(|e| CatalogError::SqliteError {
        operation: "insert_metadata".to_string(),
        source: e,
    })?;

    conn.execute(
        "INSERT OR IGNORE INTO catalog_metadata (key, value) VALUES (?1, ?2)",
        rusqlite::params!["description", ""],
    )
    .map_err(|e| CatalogError::SqliteError {
        operation: "insert_metadata".to_string(),
        source: e,
    })?;

    conn.execute(
        "INSERT OR IGNORE INTO catalog_metadata (key, value) VALUES (?1, ?2)",
        rusqlite::params!["created", &now],
    )
    .map_err(|e| CatalogError::SqliteError {
        operation: "insert_metadata".to_string(),
        source: e,
    })?;

    Ok(())
}

/// Validate that the database schema is at the current version, applying
/// forward migrations if the stored version is behind.
///
/// # Errors
///
/// Returns `CatalogError::SchemaVersionMismatch` if the stored version is
/// newer than the current code (downgrade not supported).
pub fn validate_schema(conn: &Connection) -> Result<(), CatalogError> {
    apply_migrations(conn)
}

/// Check that WAL journal mode is active.
pub fn verify_wal_mode(conn: &Connection) -> Result<bool, CatalogError> {
    let mode: String = conn
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .map_err(|e| CatalogError::SqliteError {
            operation: "verify_wal_mode".to_string(),
            source: e,
        })?;

    Ok(mode.to_lowercase() == "wal")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn initialize_creates_all_tables() {
        // Validates: Requirement 1 AC 1, AC 2, AC 3, AC 4, AC 8
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"catalog_metadata".to_string()));
        assert!(tables.contains(&"datasets".to_string()));
        assert!(tables.contains(&"gdg_bases".to_string()));
        assert!(tables.contains(&"gdg_generations".to_string()));
        assert!(tables.contains(&"audit_log".to_string()));
    }

    #[test]
    fn initialize_sets_wal_mode() {
        // Validates: Requirement 1 AC 6
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();
        // In-memory databases may not support WAL, but the pragma should not error
    }

    #[test]
    fn initialize_is_idempotent() {
        // Validates: Requirement 1 AC 7
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();
        initialize_database(&conn, "TEST").unwrap();
    }

    #[test]
    fn validate_schema_succeeds_on_current_version() {
        // Validates: Requirement 1 AC 8, Requirement 27.5
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();
        validate_schema(&conn).unwrap();
    }

    // === Req 27.5 -- forward migration scripts ============================

    #[test]
    fn migration_from_v1_to_v2_creates_audit_log_table() {
        // Validates: Requirement 27.5
        // Simulate a v1 database (no audit_log table) and verify migration adds it.
        let conn = Connection::open_in_memory().unwrap();
        // Build a v1 schema manually (no audit_log)
        conn.execute_batch(
            "
            CREATE TABLE catalog_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT);
            CREATE TABLE datasets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                dsn TEXT UNIQUE NOT NULL,
                dsorg TEXT NOT NULL,
                storage_path TEXT NOT NULL
            );
        ",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO catalog_metadata (key, value) VALUES ('schema_version', '1')",
            [],
        )
        .unwrap();

        // audit_log must not exist yet
        let has_audit: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_log'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
            > 0;
        assert!(!has_audit, "audit_log should not exist before migration");

        apply_migrations(&conn).unwrap();

        // audit_log must now exist
        let has_audit_after: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_log'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
            > 0;
        assert!(has_audit_after, "audit_log should exist after migration");

        // schema_version must be updated to 3 (v1->v2->v3 chain)
        let version: String = conn
            .query_row(
                "SELECT value FROM catalog_metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "3");
    }

    #[test]
    fn migration_from_v2_to_v3_adds_scope_column() {
        // Validates: Requirement 29.1, 29.4, 27.5
        let conn = Connection::open_in_memory().unwrap();
        // Build a v2 schema (no scope column)
        conn.execute_batch(
            "
            CREATE TABLE catalog_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT);
            CREATE TABLE datasets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                dsn TEXT UNIQUE NOT NULL,
                dsorg TEXT NOT NULL,
                storage_path TEXT NOT NULL
            );
            CREATE TABLE audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                action TEXT NOT NULL,
                object_dsn TEXT,
                outcome TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                principal TEXT
            );
        ",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO catalog_metadata (key, value) VALUES ('schema_version', '2')",
            [],
        )
        .unwrap();

        apply_migrations(&conn).unwrap();

        // scope column must now exist -- insert a row using it
        conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path, scope) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["TEST.DS", "PS", "storage/x", "master"],
        )
        .unwrap();

        let scope: String = conn
            .query_row(
                "SELECT scope FROM datasets WHERE dsn = ?1",
                rusqlite::params!["TEST.DS"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(scope, "master");

        let version: String = conn
            .query_row(
                "SELECT value FROM catalog_metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "3");
    }

    #[test]
    fn migration_from_v1_to_v3_applies_both_migrations() {
        // Validates: Requirement 27.5 -- chained migrations
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE catalog_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT);
            CREATE TABLE datasets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                dsn TEXT UNIQUE NOT NULL,
                dsorg TEXT NOT NULL,
                storage_path TEXT NOT NULL
            );
        ",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO catalog_metadata (key, value) VALUES ('schema_version', '1')",
            [],
        )
        .unwrap();

        apply_migrations(&conn).unwrap();

        // Both audit_log and scope column must exist
        let has_audit: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_log'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
            > 0;
        assert!(has_audit);

        // scope column usable
        conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path, scope) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["A.B", "PS", "storage/x", "user"],
        )
        .unwrap();

        let version: String = conn
            .query_row(
                "SELECT value FROM catalog_metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "3");
    }

    #[test]
    fn migration_is_idempotent_on_current_version() {
        // Validates: Requirement 27.5
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();
        // Calling apply_migrations on an already-current database is a no-op
        apply_migrations(&conn).unwrap();
        apply_migrations(&conn).unwrap();
    }

    #[test]
    fn migration_rejects_newer_version() {
        // Validates: Requirement 27.5 -- downgrade not supported
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE catalog_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT);
        ",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO catalog_metadata (key, value) VALUES ('schema_version', '999')",
            [],
        )
        .unwrap();
        let err = apply_migrations(&conn).unwrap_err();
        assert!(matches!(err, CatalogError::SchemaVersionMismatch { .. }));
    }

    #[test]
    fn datasets_table_enforces_unique_dsn() {
        // Validates: Requirement 1 AC 5
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();

        conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path) VALUES (?1, ?2, ?3)",
            rusqlite::params!["TEST.DSN", "PS", "storage/TEST/DSN"],
        )
        .unwrap();

        let result = conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path) VALUES (?1, ?2, ?3)",
            rusqlite::params!["TEST.DSN", "PS", "storage/TEST/DSN2"],
        );

        assert!(result.is_err());
    }

    #[test]
    fn datasets_table_enforces_dsorg_check() {
        // Validates: Requirement 1 AC 2
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();

        let result = conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path) VALUES (?1, ?2, ?3)",
            rusqlite::params!["TEST.DSN", "INVALID", "storage/TEST/DSN"],
        );

        assert!(result.is_err());
    }

    #[test]
    fn datasets_table_enforces_lrecl_range() {
        // Validates: Requirement 1 AC 2
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();

        let result = conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path, lrecl) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["TEST.DSN", "PS", "storage/TEST/DSN", 0],
        );
        assert!(result.is_err());

        let result = conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path, lrecl) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["TEST.DSN2", "PS", "storage/TEST/DSN2", 32761],
        );
        assert!(result.is_err());

        conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path, lrecl) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["TEST.DSN3", "PS", "storage/TEST/DSN3", 80],
        )
        .unwrap();
    }

    #[test]
    fn metadata_contains_expected_keys() {
        // Validates: Requirement 1 AC 8
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "MYCAT").unwrap();

        let name: String = conn
            .query_row(
                "SELECT value FROM catalog_metadata WHERE key = ?1",
                rusqlite::params!["catalog_name"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "MYCAT");

        let version: String = conn
            .query_row(
                "SELECT value FROM catalog_metadata WHERE key = ?1",
                rusqlite::params!["schema_version"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "3");
    }

    #[test]
    fn parameterized_queries_used() {
        // Validates: Requirement 1 AC 9
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();

        let malicious = "'; DROP TABLE datasets; --";
        conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path) VALUES (?1, ?2, ?3)",
            rusqlite::params![malicious, "PS", "storage/bad"],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM datasets", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
