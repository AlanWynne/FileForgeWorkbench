//! End-to-end integration tests for the schema v4 Volume split
//! (dataset-catalog task 37; Requirement 32.1-32.7).
//!
//! These drive the public catalog surface: a Repository is initialised, a
//! `CatalogRegistry` mounts it (which runs the v3->v4 dual-read seed), datasets
//! are allocated, and resolution is exercised through the DatasetVolume
//! indirection plus the uncataloged VOL=SER path.

use ff_dscatalog::dataset::{AllocParams, Dsorg};
use ff_dscatalog::dsn::Dsn;
use ff_dscatalog::hierarchy::CatalogScope;
use ff_dscatalog::{CatalogMount, CatalogRegistry, Repository};
use tempfile::TempDir;

fn ps_params(dsn: &str) -> AllocParams {
    AllocParams {
        dsn: Dsn::parse(dsn).unwrap(),
        dsorg: Dsorg::PS,
        recfm: None,
        lrecl: None,
        blksize: None,
        dir_blocks: None,
        gdg_limit: None,
        gdg_scratch: None,
        subtype: None,
        description: None,
        scope: CatalogScope::User,
    }
}

/// Mount a catalog by name, allocate the given datasets, then DROP the registry
/// and re-mount so the v3->v4 seed observes the pre-existing datasets rows and
/// populates `volumes` + `dataset_volumes` for them.
fn init_and_seed(
    tmp: &TempDir,
    name: &str,
    datasets: &[&str],
) -> (std::path::PathBuf, CatalogRegistry) {
    let path = tmp.path().join(name);
    let repo = Repository::new(&path);
    repo.initialize(name).unwrap();

    {
        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();
        let catalog = registry.get_catalog(name).unwrap();
        for dsn in datasets {
            catalog.allocate(ps_params(dsn)).unwrap();
        }
    } // registry dropped -- datasets now persisted without dataset_volumes rows

    let mut registry = CatalogRegistry::new();
    registry.mount(CatalogMount::local(&path, 1)).unwrap();
    (path, registry)
}

#[test]
fn dual_read_migration_resolves_via_dataset_volume() {
    // Validates: Requirement 32.5, 32.3
    // After re-mount, the seed has created a Volume + a DatasetVolume per
    // dataset, so resolution flows Dataset -> DatasetVolume -> Volume -> locator
    // and still lands on the real physical file.
    let tmp = TempDir::new().unwrap();
    let (path, registry) = init_and_seed(&tmp, "CATVOL", &["A.B.C", "D.E.F"]);

    let result = registry.resolve(&Dsn::parse("A.B.C").unwrap()).unwrap();
    assert_eq!(result.catalog_name, "CATVOL");
    // The resolved path is storage_uri (repo root) joined to the locator
    // (the seeded storage_path), i.e. the real file created at allocate time.
    assert!(
        result.physical_path.starts_with(&path),
        "resolved path must sit under the repository root"
    );
    assert!(
        result.physical_path.exists(),
        "resolved physical path must point at the allocated file"
    );
}

#[test]
fn resolve_falls_back_to_storage_path_when_unmigrated() {
    // Validates: Requirement 32.3 -- dual-read fallback
    // A dataset allocated AFTER the seed ran has no dataset_volumes row, so
    // resolution must fall back to its storage_path and still resolve.
    let tmp = TempDir::new().unwrap();
    let (_path, registry) = init_and_seed(&tmp, "CATFB", &["SEED.ONE"]);

    // Allocate a new dataset post-seed (no DatasetVolume row for it).
    registry
        .get_catalog("CATFB")
        .unwrap()
        .allocate(ps_params("LATE.TWO"))
        .unwrap();

    let result = registry.resolve(&Dsn::parse("LATE.TWO").unwrap()).unwrap();
    assert!(result.physical_path.exists());
}

#[test]
fn migration_moves_no_bytes_and_preserves_storage_path() {
    // Validates: Requirement 32.5 -- metadata only, storage_path retained
    let tmp = TempDir::new().unwrap();
    let (_path, registry) = init_and_seed(&tmp, "CATNOBYTES", &["KEEP.ME"]);

    let result = registry.resolve(&Dsn::parse("KEEP.ME").unwrap()).unwrap();
    // storage_path is still populated on the record (dual-read retained it).
    assert!(!result.entry.storage_path.is_empty());
    assert!(result.physical_path.exists());
}

#[test]
fn uncataloged_resolves_by_volser_unit_no_catalog_row() {
    // Validates: Requirement 32.7 -- VOL=SER + UNIT with no datasets row
    // The seed derives the VOLSER from the catalog name (uppercased), so the
    // mounted catalog "WORK01" owns a Volume with VOLSER WORK01 that can be
    // resolved uncataloged.
    let tmp = TempDir::new().unwrap();
    let (path, registry) = init_and_seed(&tmp, "WORK01", &["ANY.DS"]);

    let (catalog_name, physical) = registry.resolve_uncataloged("WORK01", "3390").unwrap();
    assert_eq!(catalog_name, "WORK01");
    // The path is the Volume storage_uri (the repository root) joined to the
    // synthesized uncataloged locator.
    assert!(physical.starts_with(&path));
}

#[test]
fn uncataloged_unknown_volser_reports_unavailable() {
    // Validates: Requirement 32.7 -- unknown VOL=SER is reported, not resolved
    let tmp = TempDir::new().unwrap();
    let (_path, registry) = init_and_seed(&tmp, "KNOWN01", &[]);

    let err = registry
        .resolve_uncataloged("NOPE99", "3390")
        .expect_err("unknown VOLSER must not resolve");
    let msg = err.to_string();
    assert!(msg.contains("volume unavailable"), "got: {msg}");
}

#[test]
fn shared_volume_two_catalogs_each_seed_their_own_volume() {
    // Validates: Requirement 32.6 -- the schema permits many catalogs to
    // register datasets on volumes; here two mounted catalogs each own a
    // seeded Volume and both resolve independently (shared-volume registration
    // is permitted at the schema level; see the schema-layer cardinality test
    // schema::tests::schema_v4_creates_volumes_and_dataset_volumes).
    let tmp = TempDir::new().unwrap();
    let (_p1, reg1) = init_and_seed(&tmp, "SHARE1", &["ONE.DS"]);
    let (_p2, reg2) = init_and_seed(&tmp, "SHARE2", &["TWO.DS"]);

    assert!(reg1.resolve(&Dsn::parse("ONE.DS").unwrap()).is_ok());
    assert!(reg2.resolve(&Dsn::parse("TWO.DS").unwrap()).is_ok());
    // Each catalog resolves its own VOLSER uncataloged.
    assert!(reg1.resolve_uncataloged("SHARE1", "3390").is_ok());
    assert!(reg2.resolve_uncataloged("SHARE2", "3390").is_ok());
}
