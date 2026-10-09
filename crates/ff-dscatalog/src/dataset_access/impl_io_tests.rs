//! Tests for `DatasetAccess` error mapping, object-safety, the deferred VSAM
//! point() paths, and dispose (CR-CH-059, Requirement 34.1/34.7/34.8).
//!
//! Split from `impl_access_tests.rs` to keep each test file under the ~200-line
//! testing.md guidance.

use std::sync::Arc;

use ff_vfs::{PosixNativeProvider, StorageProvider, VfsError};
use ff_volume::{
    AccessMode, AllocationUnit, GeometryProfile, UnitKind, Volser, Volume, VolumeId, VolumeRegistry,
};
use tempfile::TempDir;

use super::error::DatasetError;
use super::impl_access::CatalogDatasetAccess;
use super::trait_def::DatasetAccess;
use super::types::{AccessIntent, DdRequest, Positioner, StepOutcome};
use crate::dataset::{Dsorg, Recfm};
use crate::error::CatalogError;

// === Fixtures ===================================================

fn provider(dir: &TempDir) -> Arc<dyn StorageProvider> {
    Arc::new(PosixNativeProvider::new(dir.path(), false))
}

fn registry(total_tracks: u64) -> VolumeRegistry {
    let mut reg = VolumeRegistry::new();
    let mut vol = Volume::new(
        VolumeId(1),
        Volser::try_new("VOL001").expect("volser"),
        "ignored-storage-uri",
        total_tracks,
    );
    vol.set_access_mode(AccessMode::ReadWrite);
    reg.define(vol).expect("define volume");
    reg
}

fn access(dir: &TempDir, total_tracks: u64) -> CatalogDatasetAccess {
    CatalogDatasetAccess::new(provider(dir), registry(total_tracks))
}

fn dd(dsn: &str, recfm: Recfm, lrecl: u32, dsorg: Dsorg) -> DdRequest {
    DdRequest {
        dsn: dsn.to_string(),
        dsorg,
        recfm,
        lrecl,
        blksize: lrecl,
        space: AllocationUnit::new(UnitKind::Trk, 1, 1),
        geometry: GeometryProfile::default(),
        volume_id: VolumeId(1),
        max_extents: 16,
    }
}

// === Error mapping (Req 34.7) ===================================================

#[test]
fn dataset_error_maps_to_vfs_error() {
    // Validates: Requirement 34.7 -- DatasetError crosses the seam onto VfsError
    let not_found = DatasetError::NotFound {
        dsn: "A.B".to_string(),
    };
    let vfs: VfsError = not_found.into();
    assert!(matches!(vfs, VfsError::NotFound { .. }));

    let readonly = DatasetError::ReadOnlyVolume {
        volser: "VOL001".to_string(),
    };
    let vfs: VfsError = readonly.into();
    assert!(matches!(vfs, VfsError::PermissionDenied { .. }));
}

#[test]
fn dataset_error_maps_from_catalog_volume_codec_vfs() {
    // Validates: Requirement 34.7 -- maps from every upstream taxonomy
    let from_cat: DatasetError = CatalogError::DatasetNotFound {
        dsn: "A.B".to_string(),
        operation: "resolve".to_string(),
    }
    .into();
    assert!(matches!(from_cat, DatasetError::NotFound { .. }));

    let from_vol: DatasetError = ff_volume::VolumeError::VolumeFull {
        volser: "VOL001".to_string(),
    }
    .into();
    assert!(matches!(from_vol, DatasetError::VolumeFull { .. }));

    let from_vfs: DatasetError = VfsError::AlreadyExists {
        uri: "A.B".to_string(),
        operation: "create".to_string(),
    }
    .into();
    assert!(matches!(from_vfs, DatasetError::AlreadyExists { .. }));
}

// === Object safety (Req 34.8) ===================================================

#[test]
fn dataset_access_is_object_safe_as_dyn() {
    // Validates: Requirement 34.8 -- dyn DatasetAccess + mock substitution
    let dir = TempDir::new().unwrap();
    let a: Box<dyn DatasetAccess> = Box::new(access(&dir, 1000));
    let handle = a
        .allocate(&dd("USR.DYN", Recfm::FB, 3, Dsorg::PS))
        .expect("allocate via dyn");
    assert_eq!(handle.dsn(), "USR.DYN");
}

// === VSAM paths DEFINED, concrete op deferred (Req 34.1; RC.B.7 deferred) ============

#[test]
fn point_on_ksds_returns_not_yet_wired() {
    // Validates: Requirement 34.1 (defined); RC.B.7 deferral
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.KSDS", Recfm::F, 10, Dsorg::PS))
        .expect("allocate");
    let mut open = a.open(&handle, AccessIntent::Read).expect("open");
    let err = a
        .point(&mut open, &Positioner::Key(b"KEY1".to_vec()))
        .expect_err("not yet wired");
    assert!(
        matches!(err, DatasetError::NotYetWired { .. }),
        "got {err:?}"
    );
}

#[test]
fn point_relative_rrds_returns_not_yet_wired() {
    // Validates: Requirement 34.1 (defined); RC.B.7 deferral
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.RRDS", Recfm::F, 10, Dsorg::PS))
        .expect("allocate");
    let mut open = a.open(&handle, AccessIntent::Read).expect("open");
    let err = a
        .point(&mut open, &Positioner::Rrn(5))
        .expect_err("not yet wired");
    assert!(
        matches!(err, DatasetError::NotYetWired { .. }),
        "got {err:?}"
    );
}

// === dispose (Req 34.1) ===================================================

#[test]
fn dispose_delete_removes_physical_object() {
    // Validates: Requirement 34.1 -- dispose DELETE removes bytes + extents
    let dir = TempDir::new().unwrap();
    let prov = provider(&dir);
    let a = CatalogDatasetAccess::new(Arc::clone(&prov), registry(1000));
    let handle = a
        .allocate(&dd("USR.DEL", Recfm::U, 0, Dsorg::PS))
        .expect("allocate");
    assert!(prov.list().unwrap().iter().any(|(_, n)| n == "USR.DEL"));
    a.dispose(handle, StepOutcome::Delete).expect("delete");
    assert!(
        !prov.list().unwrap().iter().any(|(_, n)| n == "USR.DEL"),
        "physical object should be gone after DELETE"
    );
}

#[test]
fn dispose_keep_catlg_uncatlg_pass_retain_bytes() {
    // Validates: Requirement 34.1 -- KEEP/CATLG/UNCATLG/PASS keep the bytes
    let dir = TempDir::new().unwrap();
    let prov = provider(&dir);
    for (i, outcome) in [
        StepOutcome::Keep,
        StepOutcome::Catlg,
        StepOutcome::Uncatlg,
        StepOutcome::Pass,
    ]
    .into_iter()
    .enumerate()
    {
        let a = CatalogDatasetAccess::new(Arc::clone(&prov), registry(1000));
        let dsn = format!("USR.K{i}");
        let handle = a
            .allocate(&dd(&dsn, Recfm::U, 0, Dsorg::PS))
            .expect("allocate");
        a.dispose(handle, outcome).expect("dispose");
        assert!(
            prov.list().unwrap().iter().any(|(_, n)| n == &dsn),
            "{outcome:?} must retain the physical object"
        );
    }
}
