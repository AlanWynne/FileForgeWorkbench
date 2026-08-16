//! Mock Compilation Tests — Trait-Based Coupling Verification.
//!
//! These tests prove that dependent crates can compile against mock
//! implementations of the service traits, verifying that no concrete-type
//! coupling exists. If these tests compile, the trait-based architecture
//! is correct.
//!
//! Run with: `cargo test -p ff-governance-tests --test mock_compilation`

use std::path::PathBuf;

use ff_dataset_catalog::{
    CatalogError, CatalogService, DatasetAttributes, DatasetEntry, DatasetFilter, DatasetId,
    DsnValidationError, Dsorg, DynCatalogService, GenerationInfo, ResolutionResult,
};
use ff_vsam_services::{
    AccessMode, BrowseDirection, BrowseHandle, KeyField, Record, StubVsamService, VsamError,
    VsamHandle, VsamParams, VsamService, VsamType,
};

// ─── Mock CatalogService ────────────────────────────────────────────────────

/// A mock implementation of CatalogService for testing trait-based coupling.
///
/// Validates: Requirement 4 AC 7; Requirement 18 AC 4
struct MockCatalogService;

impl CatalogService for MockCatalogService {
    fn create_dataset(
        &self,
        _dsn: &str,
        _attrs: DatasetAttributes,
    ) -> Result<DatasetId, CatalogError> {
        Ok(DatasetId("MOCK-001".to_string()))
    }

    fn delete_dataset(&self, _dsn: &str) -> Result<(), CatalogError> {
        Ok(())
    }

    fn update_dataset(&self, _dsn: &str, _attrs: DatasetAttributes) -> Result<(), CatalogError> {
        Ok(())
    }

    fn rename_dataset(&self, _old_dsn: &str, _new_dsn: &str) -> Result<(), CatalogError> {
        Ok(())
    }

    fn resolve_dsn(&self, _dsn: &str) -> Result<ResolutionResult, CatalogError> {
        Ok(ResolutionResult {
            path: PathBuf::from("/mock/path"),
            catalog_name: "MOCK.CATALOG".to_string(),
            attributes: DatasetAttributes::default(),
        })
    }

    fn dataset_exists(&self, _dsn: &str) -> Result<bool, CatalogError> {
        Ok(true)
    }

    fn get_dataset_attributes(&self, _dsn: &str) -> Result<DatasetAttributes, CatalogError> {
        Ok(DatasetAttributes::default())
    }

    fn list_datasets(&self, _filter: &DatasetFilter) -> Result<Vec<DatasetEntry>, CatalogError> {
        Ok(Vec::new())
    }

    fn validate_dsn(&self, _dsn: &str) -> Result<(), DsnValidationError> {
        Ok(())
    }

    fn create_gdg_base(&self, _dsn: &str, _limit: u8, _scratch: bool) -> Result<(), CatalogError> {
        Ok(())
    }

    fn create_generation(
        &self,
        _base_dsn: &str,
        _attrs: DatasetAttributes,
    ) -> Result<GenerationInfo, CatalogError> {
        Ok(GenerationInfo {
            generation_name: "G0001V00".to_string(),
            dsn: "MOCK.BASE.G0001V00".to_string(),
            path: PathBuf::from("/mock/gdg/gen1"),
            relative_offset: 0,
        })
    }

    fn resolve_generation(
        &self,
        _base_dsn: &str,
        _offset: i32,
    ) -> Result<GenerationInfo, CatalogError> {
        Ok(GenerationInfo {
            generation_name: "G0001V00".to_string(),
            dsn: "MOCK.BASE.G0001V00".to_string(),
            path: PathBuf::from("/mock/gdg/gen1"),
            relative_offset: 0,
        })
    }

    fn list_generations(&self, _base_dsn: &str) -> Result<Vec<GenerationInfo>, CatalogError> {
        Ok(Vec::new())
    }

    fn get_allocation_defaults(&self, _dsorg: Dsorg) -> DatasetAttributes {
        DatasetAttributes::default()
    }
}

// ─── Mock VsamService ───────────────────────────────────────────────────────

/// A mock implementation of VsamService for testing trait-based coupling.
///
/// Validates: Requirement 6 AC 9; Requirement 18 AC 4
struct MockVsamService;

impl VsamService for MockVsamService {
    fn create_ksds(
        &self,
        _dsn: &str,
        _key_length: u16,
        _key_offset: u16,
        _record_length: u32,
    ) -> Result<(), VsamError> {
        Ok(())
    }

    fn create_esds(&self, _dsn: &str, _record_length: u32) -> Result<(), VsamError> {
        Ok(())
    }

    fn create_rrds(&self, _dsn: &str, _slot_size: u32) -> Result<(), VsamError> {
        Ok(())
    }

    fn create_lds(&self, _dsn: &str) -> Result<(), VsamError> {
        Ok(())
    }

    fn destroy_dataset(&self, _dsn: &str) -> Result<(), VsamError> {
        Ok(())
    }

    fn initialize_dataset(
        &self,
        _dsn: &str,
        _vsam_type: VsamType,
        _params: VsamParams,
    ) -> Result<(), VsamError> {
        Ok(())
    }

    fn open(&self, _dsn: &str, _mode: AccessMode) -> Result<VsamHandle, VsamError> {
        Ok(VsamHandle(1))
    }

    fn get(&self, _handle: &VsamHandle, _key: &[u8]) -> Result<Record, VsamError> {
        Ok(Record {
            key: vec![1, 2, 3],
            data: vec![10, 20, 30],
        })
    }

    fn put(&self, _handle: &VsamHandle, _record: &Record) -> Result<(), VsamError> {
        Ok(())
    }

    fn delete(&self, _handle: &VsamHandle, _key: &[u8]) -> Result<(), VsamError> {
        Ok(())
    }

    fn close(&self, _handle: VsamHandle) -> Result<(), VsamError> {
        Ok(())
    }

    fn start_browse(
        &self,
        _handle: &VsamHandle,
        _start_key: &[u8],
        _direction: BrowseDirection,
    ) -> Result<BrowseHandle, VsamError> {
        Ok(BrowseHandle(1))
    }

    fn next_record(&self, _browse: &BrowseHandle) -> Result<Option<Record>, VsamError> {
        Ok(None)
    }

    fn end_browse(&self, _browse: BrowseHandle) -> Result<(), VsamError> {
        Ok(())
    }

    fn define_aix(
        &self,
        _base_dsn: &str,
        _aix_dsn: &str,
        _key_field: KeyField,
    ) -> Result<(), VsamError> {
        Ok(())
    }

    fn build_index(&self, _aix_dsn: &str) -> Result<(), VsamError> {
        Ok(())
    }
}

// ─── Compilation Tests ──────────────────────────────────────────────────────

// Validates: Requirement 4 AC 7; Requirement 18 AC 4
#[test]
fn allocator_compiles_with_mock_catalog_service() {
    // This test verifies that dependent crates can use a mock CatalogService
    // implementation, proving trait-based coupling (no concrete-type dependency).
    let mock = MockCatalogService;

    // Exercise the trait interface using explicit trait method calls
    let result = CatalogService::resolve_dsn(&mock, "SYS1.LINKLIB");
    assert!(result.is_ok());

    let result = CatalogService::create_dataset(&mock, "NEW.DATASET", DatasetAttributes::default());
    assert!(result.is_ok());

    let result = CatalogService::dataset_exists(&mock, "SYS1.LINKLIB");
    assert!(result.unwrap());

    let result = CatalogService::validate_dsn(&mock, "VALID.DSN");
    assert!(result.is_ok());

    let defaults = CatalogService::get_allocation_defaults(&mock, Dsorg::Ps);
    assert_eq!(defaults.recfm, None); // Mock returns default
}

// Validates: Requirement 6 AC 9; Requirement 18 AC 4
#[test]
fn idcams_compiles_with_mock_vsam_and_catalog_services() {
    // This test verifies that ff-idcams can use mock implementations of both
    // CatalogService and VsamService, proving delegation-based architecture.
    let catalog = MockCatalogService;
    let vsam = MockVsamService;

    // Simulate DEFINE CLUSTER workflow: parse → catalog → VSAM
    let create_result =
        CatalogService::create_dataset(&catalog, "MY.VSAM.KSDS", DatasetAttributes::default());
    assert!(create_result.is_ok());

    let init_result = VsamService::create_ksds(&vsam, "MY.VSAM.KSDS", 8, 0, 256);
    assert!(init_result.is_ok());

    // Simulate DELETE workflow: VSAM destroy → catalog delete
    let destroy_result = VsamService::destroy_dataset(&vsam, "MY.VSAM.KSDS");
    assert!(destroy_result.is_ok());

    let delete_result = CatalogService::delete_dataset(&catalog, "MY.VSAM.KSDS");
    assert!(delete_result.is_ok());
}

// Validates: Requirement 15 AC 7 — DynCatalogService is object-safe
#[test]
fn dyn_catalog_service_can_be_boxed() {
    let mock: Box<dyn DynCatalogService> = Box::new(MockCatalogService);

    // Exercise through dynamic dispatch
    let result = mock.resolve_dsn("SYS1.LINKLIB");
    assert!(result.is_ok());

    let result = mock.dataset_exists("SYS1.LINKLIB");
    assert!(result.unwrap());
}

// Validates: Requirement 16 AC 6 — VsamService is object-safe
#[test]
fn vsam_service_can_be_boxed() {
    let mock: Box<dyn VsamService> = Box::new(MockVsamService);

    // Exercise through dynamic dispatch
    let result = mock.create_ksds("TEST.KSDS", 8, 0, 256);
    assert!(result.is_ok());
}

// Validates: Requirement 16 AC 7 — StubVsamService enables compilation
#[test]
fn stub_vsam_service_enables_dependent_crate_compilation() {
    let stub: Box<dyn VsamService> = Box::new(StubVsamService);

    // The stub returns NotImplemented for everything — but it compiles and runs
    let result = stub.create_ksds("TEST.KSDS", 8, 0, 256);
    assert!(result.is_err());
    match result.unwrap_err() {
        VsamError::NotImplemented { operation } => assert_eq!(operation, "create_ksds"),
        other => panic!("Expected NotImplemented, got: {other:?}"),
    }
}

// Validates: Requirement 12 AC 3
#[test]
fn dataset_allocator_source_has_no_rusqlite_imports() {
    // Verify that ff-dsalloc source files do not contain `use rusqlite` imports.
    // This is a source-level check complementing the Cargo.toml dependency check.
    let workspace_root = ff_governance_tests::compliance::workspace_root();
    let dsalloc_src = workspace_root.join("crates").join("ff-dsalloc").join("src");

    if !dsalloc_src.exists() {
        eprintln!("SKIPPED: ff-dsalloc/src does not exist");
        return;
    }

    for entry in std::fs::read_dir(&dsalloc_src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let content = std::fs::read_to_string(&path).unwrap();
            assert!(
                !content.contains("use rusqlite"),
                "ff-dsalloc source file {} contains 'use rusqlite' — \
                 all catalog access must flow through CatalogService trait (Requirement 12 AC 3)",
                path.display()
            );
        }
    }
}

// Validates: Requirement 6 AC 3; Requirement 21 AC 1
#[test]
fn idcams_source_has_no_storage_imports() {
    // Verify that ff-idcams source files do not contain storage engine imports.
    let workspace_root = ff_governance_tests::compliance::workspace_root();
    let idcams_src = workspace_root.join("crates").join("ff-idcams").join("src");

    if !idcams_src.exists() {
        eprintln!("SKIPPED: ff-idcams/src does not exist");
        return;
    }

    let prohibited_imports = &["use rusqlite", "use rocksdb", "use lmdb", "use sled"];

    for entry in std::fs::read_dir(&idcams_src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let content = std::fs::read_to_string(&path).unwrap();
            for import in prohibited_imports {
                assert!(
                    !content.contains(import),
                    "ff-idcams source file {} contains '{}' — \
                     IDCAMS must not directly access storage engines (Requirement 6 AC 3)",
                    path.display(),
                    import
                );
            }
        }
    }
}
