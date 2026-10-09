//! Schema-v4 Volume-indirected resolution helpers (Requirement 32.3, 32.6,
//! 32.7).
//!
//! These are free functions operating on a `&Connection` plus the Repository
//! root, so the resolution concern lives in its own module (keeping
//! `catalog.rs` within the 400-non-test-line budget). `Catalog` exposes thin
//! methods that delegate here. The Volume TYPE and its behaviour are owned by
//! `ff-volume` (Requirement 32.8); this module only maps rows to/from
//! `ff_volume` types.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::dataset::DatasetRecord;
use crate::error::CatalogError;

/// Resolve a dataset's physical path through the DatasetVolume indirection
/// (Requirement 32.3): read its `dataset_volumes` rows ordered by
/// `sequence_number`, join to the `volumes` row, verify each required Volume is
/// Online, and build the path from the primary Volume's `storage_uri` plus the
/// per-volume `locator`.
///
/// When the dataset has NO `dataset_volumes` row (an unmigrated row), this
/// FALLS BACK to the legacy `storage_path` joined to `repo_root` (dual-read).
///
/// # Errors
/// `CatalogError::VolumeUnavailable` identifying the first Offline or missing
/// required Volume, or `CatalogError::SqliteError` on a database error.
pub fn resolve_locator(
    conn: &Connection,
    repo_root: &Path,
    record: &DatasetRecord,
) -> Result<PathBuf, CatalogError> {
    let rows = dataset_volume_rows(conn, record.id)?;

    // Dual-read fallback: no DatasetVolume rows -> legacy storage_path.
    if rows.is_empty() {
        return Ok(repo_root.join(&record.storage_path));
    }

    // Verify every required Volume is Online, reporting the first that is not
    // (Requirement 32.3). The primary (lowest sequence) locator yields the
    // resolved path.
    let mut primary: Option<(String, String)> = None;
    for (volume_id, locator, sequence) in &rows {
        let (volser, status, storage_uri) = volume_row(conn, *volume_id)?;
        if status != "Online" {
            return Err(CatalogError::VolumeUnavailable {
                volser,
                reason: format!("status {status}"),
                operation: "resolve_locator".to_string(),
            });
        }
        if *sequence == 1 || primary.is_none() {
            primary = Some((storage_uri, locator.clone()));
        }
    }

    let (storage_uri, locator) = primary.expect("non-empty rows yield a primary");
    Ok(Path::new(&storage_uri).join(locator))
}

/// Read the `dataset_volumes` rows for a dataset id, ordered by sequence, as
/// `(volume_id, locator, sequence_number)` tuples.
fn dataset_volume_rows(
    conn: &Connection,
    dataset_id: i64,
) -> Result<Vec<(i64, String, u32)>, CatalogError> {
    let mut stmt = conn
        .prepare(
            "SELECT volume_id, locator, sequence_number FROM dataset_volumes \
             WHERE dataset_id = ?1 ORDER BY sequence_number",
        )
        .map_err(|source| CatalogError::SqliteError {
            operation: "dataset_volume_rows".to_string(),
            source,
        })?;
    let mapped = stmt
        .query_map(rusqlite::params![dataset_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
            ))
        })
        .map_err(|source| CatalogError::SqliteError {
            operation: "dataset_volume_rows".to_string(),
            source,
        })?;
    let mut out = Vec::new();
    for row in mapped {
        out.push(row.map_err(|source| CatalogError::SqliteError {
            operation: "dataset_volume_rows".to_string(),
            source,
        })?);
    }
    Ok(out)
}

/// Read a `volumes` row as `(volser, status, storage_uri)`.
fn volume_row(conn: &Connection, volume_id: i64) -> Result<(String, String, String), CatalogError> {
    conn.query_row(
        "SELECT volser, status, storage_uri FROM volumes WHERE volume_id = ?1",
        rusqlite::params![volume_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    )
    .map_err(|source| match source {
        rusqlite::Error::QueryReturnedNoRows => CatalogError::VolumeUnavailable {
            volser: format!("volume_id {volume_id}"),
            reason: "volume row not found".to_string(),
            operation: "resolve_locator".to_string(),
        },
        other => CatalogError::SqliteError {
            operation: "volume_row".to_string(),
            source: other,
        },
    })
}

/// Resolve an uncataloged dataset by explicit VOL=SER plus UNIT, with no
/// `datasets` row (Requirement 32.7, 32.6). Delegates to the `ff-volume`
/// resolver over a registry built from the `volumes` rows.
///
/// # Errors
/// `CatalogError::VolumeUnavailable` when no Volume matches `volser` or the
/// matching Volume is Offline.
pub fn resolve_uncataloged(
    conn: &Connection,
    volser: &str,
    unit: &str,
) -> Result<PathBuf, CatalogError> {
    let registry = volume_registry(conn)?;
    let serial =
        ff_volume::Volser::try_new(volser).map_err(|e| CatalogError::VolumeUnavailable {
            volser: volser.to_string(),
            reason: e.to_string(),
            operation: "resolve_uncataloged".to_string(),
        })?;
    let resolved = ff_volume::dataset_volume::resolve_uncataloged(&serial, unit, &registry)
        .map_err(|e| CatalogError::VolumeUnavailable {
            volser: volser.to_string(),
            reason: e.to_string(),
            operation: "resolve_uncataloged".to_string(),
        })?;
    Ok(Path::new(resolved.volume.storage_uri()).join(resolved.locator))
}

/// Build an in-memory `ff_volume::VolumeRegistry` from the `volumes` rows
/// (Requirement 32.6, 32.7).
fn volume_registry(conn: &Connection) -> Result<ff_volume::VolumeRegistry, CatalogError> {
    let mut stmt = conn
        .prepare("SELECT volume_id, volser, storage_uri, status, total_units FROM volumes")
        .map_err(|source| CatalogError::SqliteError {
            operation: "volume_registry".to_string(),
            source,
        })?;
    let mapped = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i64>>(4)?,
            ))
        })
        .map_err(|source| CatalogError::SqliteError {
            operation: "volume_registry".to_string(),
            source,
        })?;
    let mut registry = ff_volume::VolumeRegistry::new();
    for row in mapped {
        let (id, volser, storage_uri, status, total) =
            row.map_err(|source| CatalogError::SqliteError {
                operation: "volume_registry".to_string(),
                source,
            })?;
        let serial =
            ff_volume::Volser::try_new(volser).map_err(|e| CatalogError::RepositoryCorrupt {
                path: storage_uri.clone(),
                reason: format!("invalid VOLSER in volumes table: {e}"),
                operation: "volume_registry".to_string(),
            })?;
        let mut volume = ff_volume::Volume::new(
            ff_volume::VolumeId(id as u64),
            serial,
            storage_uri,
            total.unwrap_or(0) as u64,
        );
        if status != "Online" {
            volume.set_offline();
        }
        // Duplicate VOLSER in a single catalog's table should not happen;
        // ignore a duplicate define rather than fail resolution.
        let _ = registry.define(volume);
    }
    Ok(registry)
}
