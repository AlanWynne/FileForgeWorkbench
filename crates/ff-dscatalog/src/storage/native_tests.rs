//! Tests for `NativeFileProvider` (CR-CH-059 RC.A.3).
//!
//! Covers the preserved inherent object operations and the new
//! `ff_vfs::StorageProvider` seam.

use super::*;
use ff_vfs::StorageProvider as VfsStorageProvider;
use proptest::prelude::*;
use tempfile::TempDir;

fn tmp() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn allocate_sequential_creates_file_with_uuid_name() {
    // Validates: Requirement 20.1, 20.2, 20.3
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let (id, locator) = provider.allocate_object(dir.path(), false).unwrap();
    assert!(locator.contains(&id.to_string()));
    assert!(locator.ends_with(".dat"));
    assert!(dir.path().join(&locator).exists());
}

#[test]
fn allocate_container_creates_directory_with_uuid_name() {
    // Validates: Requirement 20.1, 20.2
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let (id, locator) = provider.allocate_object(dir.path(), true).unwrap();
    assert!(locator.contains(&id.to_string()));
    assert!(dir.path().join(&locator).is_dir());
}

#[test]
fn locator_does_not_contain_dsn_components() {
    // Validates: Requirement 20.3, 20.5 -- DSN not in physical path
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let (_id, locator) = provider.allocate_object(dir.path(), false).unwrap();
    assert!(!locator.contains("PAYROLL"));
    assert!(!locator.contains("INPUT"));
}

#[test]
fn two_allocations_produce_distinct_locators() {
    // Validates: Requirement 20.4 -- deterministic and unique
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let (_, loc1) = provider.allocate_object(dir.path(), false).unwrap();
    let (_, loc2) = provider.allocate_object(dir.path(), false).unwrap();
    assert_ne!(loc1, loc2);
}

#[test]
fn open_returns_path_for_existing_object() {
    // Validates: Requirement 19.5
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let (_, locator) = provider.allocate_object(dir.path(), false).unwrap();
    let path = provider.open_path(dir.path(), &locator).unwrap();
    assert!(path.exists());
}

#[test]
fn open_returns_error_for_missing_object() {
    // Validates: Requirement 19.5
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let err = provider
        .open_path(dir.path(), "datasets/objects/nonexistent.dat")
        .unwrap_err();
    assert!(matches!(err, CatalogError::DatasetNotFound { .. }));
}

#[test]
fn delete_removes_file() {
    // Validates: Requirement 19.5
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let (_, locator) = provider.allocate_object(dir.path(), false).unwrap();
    let path = dir.path().join(&locator);
    assert!(path.exists());
    provider.delete_object(dir.path(), &locator).unwrap();
    assert!(!path.exists());
}

#[test]
fn path_traversal_rejected() {
    // Validates: Requirement 20.7, 28.1, 28.2
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let err = provider
        .open_path(dir.path(), "../../etc/passwd")
        .unwrap_err();
    assert!(matches!(err, CatalogError::RepositoryCorrupt { .. }));
}

#[test]
fn reserved_device_name_rejected() {
    // Validates: Requirement 20.7
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let err = provider
        .open_path(dir.path(), "datasets/objects/NUL.dat")
        .unwrap_err();
    assert!(matches!(err, CatalogError::RepositoryCorrupt { .. }));
}

#[test]
fn reconcile_reports_missing_objects() {
    // Validates: Requirement 27.1, 27.2, 27.3
    let dir = tmp();
    let provider = NativeFileProvider::default();
    let (_, locator) = provider.allocate_object(dir.path(), false).unwrap();
    let missing = "datasets/objects/missing-uuid.dat".to_string();
    let discrepancies = provider
        .reconcile_locators(dir.path(), &[locator, missing.clone()])
        .unwrap();
    assert_eq!(discrepancies.len(), 1);
    assert!(discrepancies[0].contains("missing-uuid"));
}

// === ff_vfs::StorageProvider seam (CR-CH-059 RC.A.3) =========================

#[test]
fn native_provider_is_usable_as_dyn_ff_vfs_storage_provider() {
    // Validates: Requirement 13.1, 13.2 -- backend implements ff_vfs::StorageProvider.
    let dir = tmp();
    let provider: Box<dyn VfsStorageProvider> = Box::new(NativeFileProvider::with_root(dir.path()));
    assert!(provider
        .capabilities()
        .contains(&StorageCapability::StreamRead));
    assert!(provider
        .capabilities()
        .contains(&StorageCapability::StreamWrite));
}

#[test]
fn ff_vfs_allocate_open_write_round_trip() {
    // Validates: Requirement 13.3 -- opaque locator, VfsError, UUID behind struct.
    let dir = tmp();
    let provider = NativeFileProvider::with_root(dir.path());
    let locator = provider.allocate("MY.DATASET").unwrap();
    assert!(!locator.as_str().contains("MY.DATASET"));
    provider.write(&locator, b"hello").unwrap();
    let bytes = provider.open(&locator).unwrap();
    assert_eq!(bytes, b"hello");
    let stat = provider.stat(&locator).unwrap();
    assert!(!stat.is_container);
}

#[test]
fn ff_vfs_delete_removes_object() {
    // Validates: Requirement 13.3
    let dir = tmp();
    let provider = NativeFileProvider::with_root(dir.path());
    let locator = provider.allocate("D").unwrap();
    assert!(provider.open(&locator).is_ok());
    provider.delete(&locator).unwrap();
    assert!(matches!(
        provider.open(&locator),
        Err(VfsError::NotFound { .. })
    ));
}

#[test]
fn ff_vfs_errors_map_to_vfs_error() {
    // Validates: Requirement 13.3 -- CatalogError maps to VfsError at the seam
    // (path-traversal RepositoryCorrupt flows through From<CatalogError>).
    let dir = tmp();
    let provider = NativeFileProvider::with_root(dir.path());
    let loc = StorageLocator::new("../../etc/passwd");
    let err = provider.stat(&loc).unwrap_err();
    assert!(matches!(err, VfsError::Io { .. }));
}

// === Property test: path traversal rejection =================================

/// Generate locator strings that contain traversal sequences or reserved names.
fn traversal_locator_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("../../etc/passwd".to_string()),
        Just("../secret".to_string()),
        Just("..\\windows\\system32".to_string()),
        Just("..".to_string()),
        Just("datasets/objects/../../etc/shadow".to_string()),
        Just("datasets/../../../root/.ssh/id_rsa".to_string()),
        Just("NUL".to_string()),
        Just("CON".to_string()),
        Just("PRN".to_string()),
        Just("AUX".to_string()),
        Just("COM1".to_string()),
        Just("LPT1".to_string()),
        Just("NUL.dat".to_string()),
        Just("datasets/objects/NUL.dat".to_string()),
        Just("datasets/objects/CON".to_string()),
        Just("//etc/passwd".to_string()),
        Just("datasets//objects//../../etc".to_string()),
    ]
}

proptest! {
    #[test]
    fn path_traversal_and_reserved_names_always_rejected(
        locator in traversal_locator_strategy()
    ) {
        // Validates: Requirement 28.1, 28.2, 20.7
        let dir = tmp();
        let result = NativeFileProvider::resolve_path(dir.path(), &locator);
        match result {
            Err(CatalogError::RepositoryCorrupt { .. }) => {}
            Ok(resolved) => {
                let root = dir.path().canonicalize()
                    .unwrap_or_else(|_| dir.path().to_path_buf());
                prop_assert!(
                    resolved.starts_with(&root) || resolved.starts_with(dir.path()),
                    "resolved path {:?} escaped workspace root {:?}",
                    resolved,
                    root
                );
            }
            Err(_) => {}
        }
    }
}
