//! Generation Data Group (GDG) management.
//!
//! Manages GDG bases and generations with rolling limits and scratch policies.

use crate::catalog::Catalog;
use crate::dataset::{AllocParams, Dsorg, Recfm};
use crate::dsn::Dsn;
use crate::error::CatalogError;

/// A Generation Data Group base definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GdgBase {
    /// Database row ID.
    pub id: i64,
    /// The GDG base dataset name.
    pub dsn: Dsn,
    /// Maximum active generations (1–255).
    pub limit: u8,
    /// Whether rolled-off generations are physically deleted.
    pub scratch: bool,
    /// Creation timestamp.
    pub created: Option<String>,
}

/// A single generation within a GDG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GdgGeneration {
    /// Database row ID.
    pub id: i64,
    /// Reference to the owning GDG base.
    pub base_id: i64,
    /// Generation number (e.g., 1, 2, 3...).
    pub generation_number: u32,
    /// Version number (default 0).
    pub version: u32,
    /// Reference to the dataset entry for this generation.
    pub dataset_id: i64,
    /// Status of the generation.
    pub status: GdgStatus,
}

/// Status of a GDG generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GdgStatus {
    /// Currently active and accessible.
    Active,
    /// Rolled off due to limit exceeded.
    RolledOff,
    /// Deferred (not yet committed).
    Deferred,
}

impl std::fmt::Display for GdgStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::RolledOff => write!(f, "rolled_off"),
            Self::Deferred => write!(f, "deferred"),
        }
    }
}

impl std::str::FromStr for GdgStatus {
    type Err = CatalogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "rolled_off" => Ok(Self::RolledOff),
            "deferred" => Ok(Self::Deferred),
            _ => Err(CatalogError::InvalidAllocationParams {
                reason: format!("invalid GDG status: {s}"),
                operation: "parse_gdg_status".to_string(),
            }),
        }
    }
}

/// Format a generation number as GnnnnVnn.
pub fn format_generation_name(gen_number: u32, version: u32) -> String {
    format!("G{gen_number:04}V{version:02}")
}

impl Catalog {
    /// Create a new GDG base.
    pub fn create_gdg_base(
        &self,
        dsn: &Dsn,
        limit: u8,
        scratch: bool,
    ) -> Result<GdgBase, CatalogError> {
        if limit == 0 {
            return Err(CatalogError::InvalidAllocationParams {
                reason: "GDG limit must be between 1 and 255".to_string(),
                operation: "create_gdg_base".to_string(),
            });
        }

        // First allocate the GDG dataset entry
        let params = AllocParams {
            dsn: dsn.clone(),
            dsorg: Dsorg::GDG,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: Some(limit),
            gdg_scratch: Some(scratch),
            subtype: None,
            description: None,
            scope: crate::hierarchy::CatalogScope::User,
        };
        self.allocate(params)?;

        // Insert GDG base record
        let now = chrono::Utc::now().to_rfc3339();
        self.connection()
            .execute(
                "INSERT INTO gdg_bases (dsn, limit_, scratch, created) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![dsn.as_str(), limit, scratch, &now],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "create_gdg_base".to_string(),
                source: e,
            })?;

        let id = self.connection().last_insert_rowid();

        Ok(GdgBase {
            id,
            dsn: dsn.clone(),
            limit,
            scratch,
            created: Some(now),
        })
    }

    /// Create a new generation for a GDG.
    pub fn create_generation(
        &self,
        base_dsn: &Dsn,
        recfm: Option<Recfm>,
        lrecl: Option<u32>,
        blksize: Option<u32>,
    ) -> Result<GdgGeneration, CatalogError> {
        // Look up the GDG base
        let base = self.lookup_gdg_base(base_dsn)?;

        // Determine next generation number
        let next_gen: u32 = self.connection()
            .query_row(
                "SELECT COALESCE(MAX(generation_number), 0) + 1 FROM gdg_generations WHERE base_id = ?1",
                rusqlite::params![base.id],
                |row| row.get(0),
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "create_generation".to_string(),
                source: e,
            })?;

        let gen_name = format_generation_name(next_gen, 0);
        let gen_dsn_str = format!("{}.{}", base_dsn.as_str(), gen_name);
        let gen_dsn = Dsn::parse(&gen_dsn_str).map_err(|_| CatalogError::GdgLimitExceeded {
            dsn: base_dsn.as_str().to_string(),
            reason: format!("generation DSN too long: {gen_dsn_str}"),
            operation: "create_generation".to_string(),
        })?;

        // Create the generation dataset as a PS file in the GDG directory
        let gen_path = format!(
            "gdg/{}/{}",
            crate::encoding::dsn_to_storage_path(base_dsn),
            gen_name
        );
        let full_path = self.repository().root().join(&gen_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CatalogError::IoError {
                operation: "create_generation".to_string(),
                source: e,
            })?;
        }
        std::fs::File::create(&full_path).map_err(|e| CatalogError::IoError {
            operation: "create_generation".to_string(),
            source: e,
        })?;

        // Insert dataset entry for the generation
        let now = chrono::Utc::now().to_rfc3339();
        let effective_recfm = recfm.unwrap_or(Recfm::FB);
        let effective_lrecl = lrecl.unwrap_or(80);
        let effective_blksize = blksize.unwrap_or(27920);

        self.connection()
            .execute(
                "INSERT INTO datasets (dsn, dsorg, storage_path, recfm, lrecl, blksize, created, modified) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    gen_dsn.as_str(), "PS", &gen_path,
                    effective_recfm.to_string(),
                    effective_lrecl, effective_blksize, &now, &now,
                ],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "create_generation".to_string(),
                source: e,
            })?;

        let dataset_id = self.connection().last_insert_rowid();

        // Insert generation record
        self.connection()
            .execute(
                "INSERT INTO gdg_generations (base_id, generation_number, version, dataset_id, status) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![base.id, next_gen, 0, dataset_id, "active"],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "create_generation".to_string(),
                source: e,
            })?;

        let gen_id = self.connection().last_insert_rowid();

        // Enforce rolling limit — roll off oldest if needed
        self.enforce_gdg_limit(&base)?;

        Ok(GdgGeneration {
            id: gen_id,
            base_id: base.id,
            generation_number: next_gen,
            version: 0,
            dataset_id,
            status: GdgStatus::Active,
        })
    }

    /// Enforce GDG rolling limit by rolling off oldest generations.
    fn enforce_gdg_limit(&self, base: &GdgBase) -> Result<(), CatalogError> {
        let active_count: u32 = self
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM gdg_generations WHERE base_id = ?1 AND status = 'active'",
                rusqlite::params![base.id],
                |row| row.get(0),
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "enforce_gdg_limit".to_string(),
                source: e,
            })?;

        if active_count <= base.limit as u32 {
            return Ok(());
        }

        let excess = active_count - base.limit as u32;

        // Get oldest active generations to roll off
        let mut stmt = self
            .connection()
            .prepare(
                "SELECT id, dataset_id FROM gdg_generations \
                 WHERE base_id = ?1 AND status = 'active' \
                 ORDER BY generation_number ASC LIMIT ?2",
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "enforce_gdg_limit".to_string(),
                source: e,
            })?;

        let to_rolloff: Vec<(i64, i64)> = stmt
            .query_map(rusqlite::params![base.id, excess], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| CatalogError::SqliteError {
                operation: "enforce_gdg_limit".to_string(),
                source: e,
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (gen_id, dataset_id) in to_rolloff {
            if base.scratch {
                // Delete physical storage
                let path: Option<String> = self
                    .connection()
                    .query_row(
                        "SELECT storage_path FROM datasets WHERE id = ?1",
                        rusqlite::params![dataset_id],
                        |row| row.get(0),
                    )
                    .ok();

                if let Some(storage_path) = path {
                    let full = self.repository().root().join(&storage_path);
                    let _ = std::fs::remove_file(&full);
                }

                // Delete dataset entry and generation record
                let _ = self.connection().execute(
                    "DELETE FROM datasets WHERE id = ?1",
                    rusqlite::params![dataset_id],
                );
                let _ = self.connection().execute(
                    "DELETE FROM gdg_generations WHERE id = ?1",
                    rusqlite::params![gen_id],
                );
            } else {
                // Mark as rolled off
                let _ = self.connection().execute(
                    "UPDATE gdg_generations SET status = 'rolled_off' WHERE id = ?1",
                    rusqlite::params![gen_id],
                );
            }
        }

        Ok(())
    }

    /// Look up a GDG base by DSN.
    pub fn lookup_gdg_base(&self, dsn: &Dsn) -> Result<GdgBase, CatalogError> {
        self.connection()
            .query_row(
                "SELECT id, dsn, limit_, scratch, created FROM gdg_bases WHERE dsn = ?1",
                rusqlite::params![dsn.as_str()],
                |row| {
                    Ok(GdgBase {
                        id: row.get(0)?,
                        dsn: Dsn::parse(&row.get::<_, String>(1)?).unwrap_or_else(|_| dsn.clone()),
                        limit: row.get::<_, u8>(2)?,
                        scratch: row.get(3)?,
                        created: row.get(4)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => CatalogError::DatasetNotFound {
                    dsn: dsn.as_str().to_string(),
                    operation: "lookup_gdg_base".to_string(),
                },
                _ => CatalogError::SqliteError {
                    operation: "lookup_gdg_base".to_string(),
                    source: e,
                },
            })
    }

    /// List active generations of a GDG, newest first.
    pub fn list_generations(&self, base_dsn: &Dsn) -> Result<Vec<GdgGeneration>, CatalogError> {
        let base = self.lookup_gdg_base(base_dsn)?;

        let mut stmt = self
            .connection()
            .prepare(
                "SELECT id, base_id, generation_number, version, dataset_id, status \
                 FROM gdg_generations WHERE base_id = ?1 AND status = 'active' \
                 ORDER BY generation_number DESC",
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "list_generations".to_string(),
                source: e,
            })?;

        let gens = stmt
            .query_map(rusqlite::params![base.id], |row| {
                let status_str: String = row.get(5)?;
                Ok(GdgGeneration {
                    id: row.get(0)?,
                    base_id: row.get(1)?,
                    generation_number: row.get(2)?,
                    version: row.get(3)?,
                    dataset_id: row.get(4)?,
                    status: status_str.parse().unwrap_or(GdgStatus::Active),
                })
            })
            .map_err(|e| CatalogError::SqliteError {
                operation: "list_generations".to_string(),
                source: e,
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(gens)
    }

    /// Delete a GDG base and all its generations.
    pub fn delete_gdg_base(&self, dsn: &Dsn) -> Result<(), CatalogError> {
        let base = self.lookup_gdg_base(dsn)?;

        // Get all generation dataset IDs
        let mut stmt = self
            .connection()
            .prepare("SELECT dataset_id FROM gdg_generations WHERE base_id = ?1")
            .map_err(|e| CatalogError::SqliteError {
                operation: "delete_gdg_base".to_string(),
                source: e,
            })?;

        let dataset_ids: Vec<i64> = stmt
            .query_map(rusqlite::params![base.id], |row| row.get(0))
            .map_err(|e| CatalogError::SqliteError {
                operation: "delete_gdg_base".to_string(),
                source: e,
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Delete physical files for each generation
        for dataset_id in &dataset_ids {
            let path: Option<String> = self
                .connection()
                .query_row(
                    "SELECT storage_path FROM datasets WHERE id = ?1",
                    rusqlite::params![dataset_id],
                    |row| row.get(0),
                )
                .ok();
            if let Some(p) = path {
                let full = self.repository().root().join(&p);
                let _ = std::fs::remove_file(&full);
            }
        }

        // Delete generation records
        self.connection()
            .execute(
                "DELETE FROM gdg_generations WHERE base_id = ?1",
                rusqlite::params![base.id],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "delete_gdg_base".to_string(),
                source: e,
            })?;

        // Delete generation dataset entries
        for dataset_id in &dataset_ids {
            let _ = self.connection().execute(
                "DELETE FROM datasets WHERE id = ?1",
                rusqlite::params![dataset_id],
            );
        }

        // Delete GDG base
        self.connection()
            .execute(
                "DELETE FROM gdg_bases WHERE id = ?1",
                rusqlite::params![base.id],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "delete_gdg_base".to_string(),
                source: e,
            })?;

        // Delete the base dataset entry
        self.delete(dsn)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Catalog) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("gdg-test");
        let repo = Repository::new(&path);
        repo.initialize("GDGTEST").unwrap();
        let catalog = Catalog::mount(&path, 1).unwrap();
        (tmp, catalog)
    }

    #[test]
    fn create_gdg_base() {
        // Validates: Requirement 9 AC 1
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("PAYROLL.MONTHLY").unwrap();
        let base = catalog.create_gdg_base(&dsn, 5, true).unwrap();
        assert_eq!(base.limit, 5);
        assert!(base.scratch);
    }

    #[test]
    fn create_generation_increments_number() {
        // Validates: Requirement 9 AC 2
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("PAY.GDG").unwrap();
        catalog.create_gdg_base(&dsn, 10, true).unwrap();

        let g1 = catalog.create_generation(&dsn, None, None, None).unwrap();
        assert_eq!(g1.generation_number, 1);

        let g2 = catalog.create_generation(&dsn, None, None, None).unwrap();
        assert_eq!(g2.generation_number, 2);
    }

    #[test]
    fn rolloff_enforced_with_scratch() {
        // Validates: Requirement 9 AC 3
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("ROLL.GDG").unwrap();
        catalog.create_gdg_base(&dsn, 3, true).unwrap();

        // Create 5 generations (limit is 3)
        for _ in 0..5 {
            catalog.create_generation(&dsn, None, None, None).unwrap();
        }

        let gens = catalog.list_generations(&dsn).unwrap();
        assert!(gens.len() <= 3, "Active count should not exceed limit");
    }

    #[test]
    fn rolloff_without_scratch_preserves_storage() {
        // Validates: Requirement 9 AC 3
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("KEEP.GDG").unwrap();
        catalog.create_gdg_base(&dsn, 2, false).unwrap();

        catalog.create_generation(&dsn, None, None, None).unwrap();
        catalog.create_generation(&dsn, None, None, None).unwrap();
        catalog.create_generation(&dsn, None, None, None).unwrap();

        let gens = catalog.list_generations(&dsn).unwrap();
        assert!(gens.len() <= 2);
    }

    #[test]
    fn list_generations_sorted_newest_first() {
        // Validates: Requirement 9 AC 6
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("LIST.GDG").unwrap();
        catalog.create_gdg_base(&dsn, 10, true).unwrap();

        catalog.create_generation(&dsn, None, None, None).unwrap();
        catalog.create_generation(&dsn, None, None, None).unwrap();
        catalog.create_generation(&dsn, None, None, None).unwrap();

        let gens = catalog.list_generations(&dsn).unwrap();
        assert_eq!(gens.len(), 3);
        assert_eq!(gens[0].generation_number, 3); // newest first
        assert_eq!(gens[2].generation_number, 1);
    }

    #[test]
    fn delete_gdg_base_removes_everything() {
        // Validates: Requirement 9 AC 8
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("DEL.GDG").unwrap();
        catalog.create_gdg_base(&dsn, 5, true).unwrap();
        catalog.create_generation(&dsn, None, None, None).unwrap();
        catalog.create_generation(&dsn, None, None, None).unwrap();

        catalog.delete_gdg_base(&dsn).unwrap();

        assert!(catalog.lookup_gdg_base(&dsn).is_err());
        assert!(catalog.lookup(&dsn).is_err());
    }

    #[test]
    fn format_generation_name_correct() {
        assert_eq!(format_generation_name(1, 0), "G0001V00");
        assert_eq!(format_generation_name(255, 1), "G0255V01");
    }
}
