//! Schema version 4: the `volumes` and `dataset_volumes` tables plus the
//! forward v3->v4 dual-read seed (Requirement 32.1, 32.2, 32.5).
//!
//! The Volume ROW is persisted here, but the Volume TYPE and its behaviour are
//! owned by `ff-volume` (Requirement 32.8); this module only maps rows to/from
//! `ff_volume` types. The seed moves NO dataset bytes -- it derives one
//! `volumes` row per Repository and one `dataset_volumes` row per existing
//! `datasets` row, with the locator taken from the legacy `storage_path`
//! (ADR-003). `storage_path` is retained for dual-read.

use rusqlite::Connection;

use crate::error::CatalogError;

/// SQL to create the `volumes` table (schema version 4).
///
/// Mirrors dataset-catalog design.md: `volume_id` PK, `volser` UNIQUE NOT NULL,
/// `storage_uri` (the promoted Repository root), `status`, `access_mode`, and
/// capacity counters in tracks (`total_units` / `used_units`).
/// Validates: Requirement 32.1
pub const CREATE_VOLUMES_TABLE: &str = "
CREATE TABLE IF NOT EXISTS volumes (
    volume_id   INTEGER PRIMARY KEY,
    volser      TEXT    UNIQUE NOT NULL,
    storage_uri TEXT    NOT NULL,
    status      TEXT    NOT NULL CHECK (status IN ('Online', 'Offline')),
    access_mode TEXT    NOT NULL CHECK (access_mode IN ('ReadWrite', 'ReadOnly')),
    total_units INTEGER,
    used_units  INTEGER
);
";

/// SQL to create the `dataset_volumes` table (schema version 4).
///
/// Mirrors dataset-catalog design.md: the ordered association from a dataset to
/// one or more volumes, with an opaque per-volume `locator` replacing the
/// dataset `storage_path` as the authoritative location.
/// Validates: Requirement 32.2
pub const CREATE_DATASET_VOLUMES_TABLE: &str = "
CREATE TABLE IF NOT EXISTS dataset_volumes (
    dataset_id      INTEGER NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    volume_id       INTEGER NOT NULL REFERENCES volumes(volume_id),
    sequence_number INTEGER NOT NULL,
    is_primary      INTEGER NOT NULL CHECK (is_primary IN (0, 1)),
    locator         TEXT    NOT NULL,
    PRIMARY KEY (dataset_id, sequence_number)
);
";

/// Combined DDL batch for the v3->v4 forward migration. Creating the tables is
/// idempotent (`IF NOT EXISTS`) so a connection-only `apply_migrations` run
/// never corrupts the DB even before the row seed executes.
/// Validates: Requirement 32.1, 32.2
pub const CREATE_V4_TABLES: &str = "
CREATE TABLE IF NOT EXISTS volumes (
    volume_id   INTEGER PRIMARY KEY,
    volser      TEXT    UNIQUE NOT NULL,
    storage_uri TEXT    NOT NULL,
    status      TEXT    NOT NULL CHECK (status IN ('Online', 'Offline')),
    access_mode TEXT    NOT NULL CHECK (access_mode IN ('ReadWrite', 'ReadOnly')),
    total_units INTEGER,
    used_units  INTEGER
);
CREATE TABLE IF NOT EXISTS dataset_volumes (
    dataset_id      INTEGER NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    volume_id       INTEGER NOT NULL REFERENCES volumes(volume_id),
    sequence_number INTEGER NOT NULL,
    is_primary      INTEGER NOT NULL CHECK (is_primary IN (0, 1)),
    locator         TEXT    NOT NULL,
    PRIMARY KEY (dataset_id, sequence_number)
);
";

/// Default capacity in tracks for a migrated Volume. The migration does not
/// know the host filesystem's true size, so it seeds a generous nominal
/// capacity; the real free-space check happens at the OS layer. Chosen as a
/// large round number so dual-read allocation is never blocked by a seeded cap.
pub const MIGRATION_DEFAULT_CAPACITY_TRACKS: u64 = 1_000_000;

/// Seed the v3->v4 rows for one mounted catalog: one `volumes` row for the
/// Repository and one `dataset_volumes` row per existing `datasets` row
/// (locator = `storage_path`, sequence 1, is_primary 1). Moves NO bytes and
/// retains `storage_path` (Requirement 32.5).
///
/// Idempotent: if a `volumes` row already exists for this `repo_root` the seed
/// is skipped, so re-mounting a migrated catalog does not duplicate rows.
///
/// `volume_id` is assigned as `MAX(volume_id) + 1` so multiple catalogs sharing
/// one process-level numbering do not collide. The VOLSER is derived from
/// `catalog_name` uppercased.
///
/// # Errors
/// Returns `CatalogError::SqliteError` on any database failure, or propagates a
/// `ff_volume` validation error (e.g. empty VOLSER) as a `RepositoryCorrupt`.
pub fn migrate_v3_to_v4(
    conn: &Connection,
    repo_root: &str,
    catalog_name: &str,
) -> Result<(), CatalogError> {
    // Ensure the tables exist (idempotent) even if apply_migrations has not run.
    conn.execute_batch(CREATE_V4_TABLES)
        .map_err(|source| CatalogError::SqliteError {
            operation: "migrate_v3_to_v4".to_string(),
            source,
        })?;

    // Skip if this Repository already has a Volume row (idempotent re-mount).
    let already: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM volumes WHERE storage_uri = ?1",
            rusqlite::params![repo_root],
            |row| row.get(0),
        )
        .map_err(|source| CatalogError::SqliteError {
            operation: "migrate_v3_to_v4".to_string(),
            source,
        })?;
    if already > 0 {
        return Ok(());
    }

    // Build the Volume via ff-volume (metadata only, no byte movement).
    let next_id: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(volume_id), 0) + 1 FROM volumes",
            [],
            |row| row.get(0),
        )
        .map_err(|source| CatalogError::SqliteError {
            operation: "migrate_v3_to_v4".to_string(),
            source,
        })?;
    let volume_id = ff_volume::VolumeId(next_id as u64);
    let volume = ff_volume::volume_over_repository(
        volume_id,
        repo_root,
        catalog_name,
        MIGRATION_DEFAULT_CAPACITY_TRACKS,
    )
    .map_err(|e| CatalogError::RepositoryCorrupt {
        path: repo_root.to_string(),
        reason: format!("volume migration failed: {e}"),
        operation: "migrate_v3_to_v4".to_string(),
    })?;

    conn.execute(
        "INSERT INTO volumes (volume_id, volser, storage_uri, status, access_mode, total_units, used_units) \
         VALUES (?1, ?2, ?3, 'Online', 'ReadWrite', ?4, 0)",
        rusqlite::params![
            next_id,
            volume.volser().as_str(),
            volume.storage_uri(),
            volume.total_tracks() as i64,
        ],
    )
    .map_err(|source| CatalogError::SqliteError {
        operation: "migrate_v3_to_v4".to_string(),
        source,
    })?;

    // Collect the existing datasets (id, storage_path) for the seed.
    let datasets: Vec<(ff_volume::DatasetId, String)> = {
        let mut stmt = conn
            .prepare("SELECT id, storage_path FROM datasets ORDER BY id")
            .map_err(|source| CatalogError::SqliteError {
                operation: "migrate_v3_to_v4".to_string(),
                source,
            })?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|source| CatalogError::SqliteError {
                operation: "migrate_v3_to_v4".to_string(),
                source,
            })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, path) = row.map_err(|source| CatalogError::SqliteError {
                operation: "migrate_v3_to_v4".to_string(),
                source,
            })?;
            out.push((ff_volume::DatasetId(id as u64), path));
        }
        out
    };

    // Build the DatasetVolume rows via ff-volume and insert them.
    let seeded = ff_volume::seed_dataset_volumes(&datasets, volume_id);
    for row in &seeded {
        conn.execute(
            "INSERT INTO dataset_volumes (dataset_id, volume_id, sequence_number, is_primary, locator) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                row.dataset_id.0 as i64,
                row.volume_id.0 as i64,
                row.sequence_number,
                if row.is_primary { 1 } else { 0 },
                &row.locator,
            ],
        )
        .map_err(|source| CatalogError::SqliteError {
            operation: "migrate_v3_to_v4".to_string(),
            source,
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;
    use rusqlite::Connection;

    /// Build a schema-v3 DB with the given datasets (dsn, storage_path) and set
    /// the stored version to '3' so apply_migrations advances it to '4'.
    fn v3_db_with_datasets(datasets: &[(&str, &str)]) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::initialize_database(&conn, "TESTCAT").unwrap();
        // initialize_database sets version to the current SCHEMA_VERSION ("4").
        // Roll the stored version back to "3" and drop the v4 tables so this
        // simulates a pre-migration v3 catalog.
        conn.execute_batch("DROP TABLE IF EXISTS dataset_volumes; DROP TABLE IF EXISTS volumes;")
            .unwrap();
        conn.execute(
            "UPDATE catalog_metadata SET value = '3' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
        for (dsn, path) in datasets {
            conn.execute(
                "INSERT INTO datasets (dsn, dsorg, storage_path) VALUES (?1, 'PS', ?2)",
                rusqlite::params![dsn, path],
            )
            .unwrap();
        }
        conn
    }

    #[test]
    fn migration_v3_to_v4_seeds_volume_per_repository() {
        // Validates: Requirement 32.5
        let conn = v3_db_with_datasets(&[]);
        schema::apply_migrations(&conn).unwrap();
        migrate_v3_to_v4(&conn, "/repo/root", "TESTCAT").unwrap();

        let (volser, uri, status): (String, String, String) = conn
            .query_row("SELECT volser, storage_uri, status FROM volumes", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .unwrap();
        assert_eq!(volser, "TESTCAT");
        assert_eq!(uri, "/repo/root");
        assert_eq!(status, "Online");
    }

    #[test]
    fn migration_v3_to_v4_populates_dataset_volumes_from_storage_path() {
        // Validates: Requirement 32.5
        let conn = v3_db_with_datasets(&[("A.B", "storage/A/B"), ("C.D", "storage/C/D")]);
        schema::apply_migrations(&conn).unwrap();
        migrate_v3_to_v4(&conn, "/repo/root", "TESTCAT").unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM dataset_volumes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);

        let locator: String = conn
            .query_row(
                "SELECT dv.locator FROM dataset_volumes dv \
                 JOIN datasets d ON d.id = dv.dataset_id WHERE d.dsn = 'A.B'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(locator, "storage/A/B");
        // Every seeded row is primary at sequence 1.
        let (seq, primary): (i64, i64) = conn
            .query_row(
                "SELECT sequence_number, is_primary FROM dataset_volumes \
                 JOIN datasets d ON d.id = dataset_volumes.dataset_id WHERE d.dsn = 'C.D'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(seq, 1);
        assert_eq!(primary, 1);
    }

    #[test]
    fn migration_v3_to_v4_preserves_storage_path_column() {
        // Validates: Requirement 32.5 -- dual-read, storage_path retained
        let conn = v3_db_with_datasets(&[("A.B", "storage/A/B")]);
        schema::apply_migrations(&conn).unwrap();
        migrate_v3_to_v4(&conn, "/repo/root", "TESTCAT").unwrap();

        let path: String = conn
            .query_row(
                "SELECT storage_path FROM datasets WHERE dsn = 'A.B'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(path, "storage/A/B");
    }

    #[test]
    fn migration_v3_to_v4_is_idempotent() {
        // Validates: Requirement 32.5 -- re-mount does not duplicate rows
        let conn = v3_db_with_datasets(&[("A.B", "storage/A/B")]);
        schema::apply_migrations(&conn).unwrap();
        migrate_v3_to_v4(&conn, "/repo/root", "TESTCAT").unwrap();
        migrate_v3_to_v4(&conn, "/repo/root", "TESTCAT").unwrap();

        let volumes: i64 = conn
            .query_row("SELECT COUNT(*) FROM volumes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(volumes, 1);
        let dv: i64 = conn
            .query_row("SELECT COUNT(*) FROM dataset_volumes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(dv, 1);
    }
}
