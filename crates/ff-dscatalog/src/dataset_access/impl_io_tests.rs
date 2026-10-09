//! Tests for `DatasetAccess` error mapping, object-safety, the wired VSAM
//! keyed/relative point() paths, and dispose (CR-CH-059, Requirement
//! 34.1/34.2/34.7/34.8/35.3).
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
use crate::vsam_service::{VsamParams, VsamType};
use crate::Record;

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

// === VSAM keyed/relative record ops wired (Req 34.1, 34.2, 35.3) ============

#[test]
fn point_on_ksds_round_trips_keyed_record() {
    // Validates: Requirement 34.1 -- KSDS keyed read-after-write through
    // point + put + get; boundaries come from the backend, not CRLF.
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.KSDS", Recfm::F, 10, Dsorg::PS))
        .expect("allocate");
    let params = VsamParams {
        key_length: Some(4),
        key_offset: Some(0),
        record_length: Some(10),
        slot_size: None,
    };
    let mut open = a
        .open_vsam(&handle, VsamType::Ksds, &params, AccessIntent::Update)
        .expect("open vsam");
    a.point(&mut open, &Positioner::Key(b"KEY1".to_vec()))
        .expect("point");
    a.put(
        &mut open,
        &Record {
            key: b"KEY1".to_vec(),
            data: b"A\r\nB".to_vec(),
        },
    )
    .expect("put");
    a.point(&mut open, &Positioner::Key(b"KEY1".to_vec()))
        .expect("re-point");
    let read = a.get(&mut open).expect("get").expect("present");
    assert_eq!(read.key, b"KEY1".to_vec());
    assert_eq!(read.data, b"A\r\nB".to_vec(), "record survives CR/LF bytes");
}

#[test]
fn point_relative_rrds_round_trips_and_rejects_rrn_zero() {
    // Validates: Requirement 34.1 -- RRDS point(Rrn) put/get; RRN 0 rejected
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.RRDS", Recfm::F, 10, Dsorg::PS))
        .expect("allocate");
    let mut open = a
        .open_vsam(
            &handle,
            VsamType::Rrds,
            &VsamParams::default(),
            AccessIntent::Update,
        )
        .expect("open vsam");
    a.point(&mut open, &Positioner::Rrn(5)).expect("point");
    a.put(
        &mut open,
        &Record {
            key: Vec::new(),
            data: b"five".to_vec(),
        },
    )
    .expect("put");
    a.point(&mut open, &Positioner::Rrn(5)).expect("re-point");
    assert_eq!(
        a.get(&mut open).expect("get").expect("present").data,
        b"five"
    );

    let err = a
        .point(&mut open, &Positioner::Rrn(0))
        .expect_err("rrn 0 rejected");
    assert!(
        matches!(err, DatasetError::BadPositioner { .. }),
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
