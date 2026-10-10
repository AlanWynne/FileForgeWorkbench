//! Tests for the mainframe backend Command Environment (BRC.4).
//!
//! The fixture mirrors `ff-dscatalog`'s `impl_access_tests.rs`: a real
//! `ff_vfs::PosixNativeProvider` over a `tempfile::TempDir` plus a
//! `ff_volume::VolumeRegistry`, allocating a PS dataset so the CE stores records
//! through a genuine `CatalogDatasetAccess` -- no mock, no host-text framing.

use std::sync::Arc;

use ff_dscatalog::dataset::{Dsorg, Recfm};
use ff_dscatalog::{AccessIntent, CatalogDatasetAccess, DatasetAccess, DdRequest, Record};
use ff_vfs::{
    BackendEnvironment, PosixNativeProvider, RecordAttrs, RecordFormatKind, RecordSource,
    RecordStoreOutcome, StorageProvider, StoreTarget,
};
use ff_volume::{
    AccessMode, AllocationUnit, GeometryProfile, UnitKind, Volser, Volume, VolumeId, VolumeRegistry,
};

use super::MainframeEnvironment;

// === Fixtures ===================================================

fn provider(dir: &tempfile::TempDir) -> Arc<dyn StorageProvider> {
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

fn access(dir: &tempfile::TempDir, total_tracks: u64) -> Arc<CatalogDatasetAccess> {
    Arc::new(CatalogDatasetAccess::new(
        provider(dir),
        registry(total_tracks),
    ))
}

fn dd(dsn: &str, recfm: Recfm, lrecl: u32, max_extents: u32) -> DdRequest {
    DdRequest {
        dsn: dsn.to_string(),
        dsorg: Dsorg::PS,
        recfm,
        lrecl,
        blksize: lrecl,
        space: AllocationUnit::new(UnitKind::Trk, 1, 1),
        geometry: GeometryProfile::default(),
        volume_id: VolumeId(1),
        max_extents,
    }
}

fn store_target(dsn: &str) -> StoreTarget {
    StoreTarget {
        dsn: dsn.to_string(),
        owning_env: "MAINFRAME".to_string(),
        catalog_id: None,
    }
}

/// A `RecordSource` over owned record byte vectors.
struct VecRecordSource {
    records: Vec<Vec<u8>>,
    index: usize,
}

impl VecRecordSource {
    fn new(records: Vec<Vec<u8>>) -> Self {
        Self { records, index: 0 }
    }
}

impl RecordSource for VecRecordSource {
    fn next_record(&mut self) -> Option<&[u8]> {
        let rec = self.records.get(self.index)?;
        self.index += 1;
        Some(rec.as_slice())
    }
}

// Drain all records from an open dataset for assertions.
fn drain(a: &CatalogDatasetAccess, open: &mut ff_dscatalog::OpenDataset) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    while let Some(r) = a.get(open).expect("get") {
        out.push(r.data);
    }
    out
}

// === T1a: record round-trip through DatasetAccess ===================================

#[test]
fn mainframe_env_stores_records_via_dataset_access_round_trip() {
    // Validates: command-environments Requirement 18.7
    let dir = tempfile::TempDir::new().expect("tempdir");
    let a = access(&dir, 1000);
    // The open flow allocates/resolves the dataset; here we pre-allocate the PS
    // FB dataset the CE will store over.
    let handle = a
        .allocate(&dd("USR.CE.FB", Recfm::FB, 5, 16))
        .expect("allocate");

    let env = MainframeEnvironment::new(Arc::clone(&a) as Arc<dyn DatasetAccess>);
    let attrs = RecordAttrs {
        recfm: RecordFormatKind::Fixed,
        lrecl: 5,
        encoding: "cp037".to_string(),
    };
    let mut source = VecRecordSource::new(vec![b"AB".to_vec(), b"CDE".to_vec()]);
    let outcome = env.save_records(&store_target("USR.CE.FB"), &mut source, &attrs);
    assert_eq!(outcome, RecordStoreOutcome::Stored { rc: 0 });

    // Re-open through the SAME access and assert the records round-trip framed by
    // the RECFM codec (Fixed pads each to LRECL=5 with EBCDIC spaces 0x40) --
    // boundaries are the codec's, not CRLF.
    let mut r = a.open(&handle, AccessIntent::Read).expect("open read");
    let records = drain(&a, &mut r);
    assert_eq!(records.len(), 2);
    assert_eq!(records[0], vec![b'A', b'B', 0x40, 0x40, 0x40]);
    assert_eq!(records[1], vec![b'C', b'D', b'E', 0x40, 0x40]);
}

// === T1b: x37 / space-full -> non-zero rc ===================================

#[test]
fn mainframe_env_maps_space_abend_to_nonzero_rc() {
    // Validates: command-environments Requirement 18.8
    let dir = tempfile::TempDir::new().expect("tempdir");
    let a = access(&dir, 1000);
    // max_extents=1 means no secondary extent may be acquired; a write larger
    // than one track triggers the x37-style abend on close (mirrors
    // put_growth_surfaces_x37_space_abend).
    let handle = a
        .allocate(&dd("USR.CE.X37", Recfm::U, 0, 1))
        .expect("allocate");
    let _ = handle;

    let env = MainframeEnvironment::new(Arc::clone(&a) as Arc<dyn DatasetAccess>);
    let attrs = RecordAttrs {
        recfm: RecordFormatKind::Undefined,
        lrecl: 0,
        encoding: "cp037".to_string(),
    };
    // One track is 56664 bytes; write just over to force growth past max_extents.
    let mut source = VecRecordSource::new(vec![vec![b'Z'; 56_665]]);
    let outcome = env.save_records(&store_target("USR.CE.X37"), &mut source, &attrs);
    match outcome {
        RecordStoreOutcome::Stored { rc } => {
            assert_ne!(rc, 0, "x37 space abend must surface as a non-zero rc");
        }
        other => panic!("expected Stored with non-zero rc, got {other:?}"),
    }
}

// === T1c: record-capable + object-safe ===================================

#[test]
fn mainframe_env_is_record_capable_and_object_safe() {
    // Validates: command-environments Requirement 18.3, 18.4
    let dir = tempfile::TempDir::new().expect("tempdir");
    let a = access(&dir, 1000);
    let env: Box<dyn BackendEnvironment> = Box::new(MainframeEnvironment::new(
        Arc::clone(&a) as Arc<dyn DatasetAccess>
    ));
    assert_eq!(env.name(), "MAINFRAME");
    assert!(env.record_capable());
    assert!(!env.is_case_sensitive());
    // The byte save entry is an error: a mainframe resource has no host-path byte
    // save.
    let err = env
        .save(std::path::Path::new("ignored"), b"bytes")
        .expect_err("byte save must error for a record-only resource");
    let _ = err;
    let _ = Record {
        key: Vec::new(),
        data: Vec::new(),
    };
}
