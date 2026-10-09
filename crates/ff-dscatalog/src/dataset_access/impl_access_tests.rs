//! Tests for the `DatasetAccess` contract and the concrete
//! `CatalogDatasetAccess` (CR-CH-059, Requirement 34; volume-model Req 12).
//!
//! The physical seam under test is a real `ff_vfs::PosixNativeProvider` over a
//! `tempfile::TempDir`, proving end-to-end I/O through the single
//! `StorageProvider` seam -- never a raw `storage_path`.

use std::sync::Arc;

use ff_vfs::{PosixNativeProvider, StorageProvider};
use ff_volume::{
    AccessMode, AllocationUnit, GeometryProfile, UnitKind, Volser, Volume, VolumeId, VolumeRegistry,
};
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::error::DatasetError;
use super::impl_access::CatalogDatasetAccess;
use super::trait_def::DatasetAccess;
use super::types::{AccessIntent, DdRequest};
use crate::dataset::{Dsorg, Recfm};
use crate::Record;

// === Fixtures ===================================================

fn provider(dir: &TempDir) -> Arc<dyn StorageProvider> {
    Arc::new(PosixNativeProvider::new(dir.path(), false))
}

fn registry(total_tracks: u64, mode: AccessMode, online: bool) -> VolumeRegistry {
    let mut reg = VolumeRegistry::new();
    let mut vol = Volume::new(
        VolumeId(1),
        Volser::try_new("VOL001").expect("volser"),
        "ignored-storage-uri",
        total_tracks,
    );
    vol.set_access_mode(mode);
    if !online {
        vol.set_offline();
    }
    reg.define(vol).expect("define volume");
    reg
}

fn access(dir: &TempDir, total_tracks: u64) -> CatalogDatasetAccess {
    CatalogDatasetAccess::new(
        provider(dir),
        registry(total_tracks, AccessMode::ReadWrite, true),
    )
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

fn rec(data: &[u8]) -> Record {
    Record {
        key: Vec::new(),
        data: data.to_vec(),
    }
}

// Drain all records from an open dataset for assertions.
fn drain(access: &CatalogDatasetAccess, open: &mut super::types::OpenDataset) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    while let Some(r) = access.get(open).expect("get") {
        out.push(r.data);
    }
    out
}

// === RECFM round-trips (Req 34.1, 34.2) ===================================================

#[test]
fn allocate_open_put_get_close_round_trips_fixed_fb() {
    // Validates: Requirement 34.1, 34.2
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.FB", Recfm::FB, 5, Dsorg::PS))
        .expect("allocate");

    let mut w = a.open(&handle, AccessIntent::Write).expect("open write");
    a.put(&mut w, &rec(b"AB")).expect("put1");
    a.put(&mut w, &rec(b"CDE")).expect("put2");
    a.close(w).expect("close");

    let mut r = a.open(&handle, AccessIntent::Read).expect("open read");
    let records = drain(&a, &mut r);
    // Fixed codec pads each record to LRECL=5 with EBCDIC spaces (0x40).
    assert_eq!(records.len(), 2);
    assert_eq!(records[0], vec![b'A', b'B', 0x40, 0x40, 0x40]);
    assert_eq!(records[1], vec![b'C', b'D', b'E', 0x40, 0x40]);
}

#[test]
fn round_trips_variable_vb_with_rdw() {
    // Validates: Requirement 34.2 -- variable records carry the 4-byte RDW
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.VB", Recfm::VB, 0, Dsorg::PS))
        .expect("allocate");

    let mut w = a.open(&handle, AccessIntent::Write).expect("open write");
    a.put(&mut w, &rec(b"HELLO")).expect("put1");
    a.put(&mut w, &rec(b"HI")).expect("put2");
    a.close(w).expect("close");

    let mut r = a.open(&handle, AccessIntent::Read).expect("open read");
    let records = drain(&a, &mut r);
    assert_eq!(records, vec![b"HELLO".to_vec(), b"HI".to_vec()]);
}

#[test]
fn round_trips_recfm_u_passthrough() {
    // Validates: Requirement 34.2 -- RECFM=U passes bytes through
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.U", Recfm::U, 0, Dsorg::PS))
        .expect("allocate");

    let mut w = a.open(&handle, AccessIntent::Write).expect("open write");
    a.put(&mut w, &rec(b"\x00\x01\x02rawbytes")).expect("put");
    a.close(w).expect("close");

    let mut r = a.open(&handle, AccessIntent::Read).expect("open read");
    let records = drain(&a, &mut r);
    // BinaryCodec returns the whole object as one record (pass-through).
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], b"\x00\x01\x02rawbytes".to_vec());
}

#[test]
fn get_put_use_codec_boundaries_not_crlf() {
    // Validates: Requirement 34.2 -- record boundaries are the codec's, not CRLF
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    // A record that itself contains CR and LF bytes must survive intact under
    // the fixed codec: boundaries come from LRECL, not the newline bytes.
    let handle = a
        .allocate(&dd("USR.NL", Recfm::FB, 4, Dsorg::PS))
        .expect("allocate");
    let mut w = a.open(&handle, AccessIntent::Write).expect("open write");
    a.put(&mut w, &rec(b"A\r\nB")).expect("put");
    a.close(w).expect("close");

    let mut r = a.open(&handle, AccessIntent::Read).expect("open read");
    let records = drain(&a, &mut r);
    assert_eq!(records.len(), 1, "one 4-byte record, not split on the LF");
    assert_eq!(records[0], vec![b'A', b'\r', b'\n', b'B']);
}

// === Resolution + physical seam (Req 34.3; volume-model 12.1, 12.2) ===================

#[test]
fn resolves_via_volume_datasetvolume_locator() {
    // Validates: Requirement 34.3; volume-model Requirement 12.1, 12.2
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.RES", Recfm::FB, 3, Dsorg::PS))
        .expect("allocate");
    // Open succeeds only because resolution followed Dataset -> DatasetVolume ->
    // Volume -> locator; a handle exposes no raw storage_path.
    let open = a.open(&handle, AccessIntent::Read).expect("open resolves");
    assert_eq!(open.handle().dsn(), "USR.RES");
}

#[test]
fn physical_io_through_storage_provider_seam() {
    // Validates: Requirement 34.3 -- bytes land in the StorageProvider object
    let dir = TempDir::new().unwrap();
    let prov = provider(&dir);
    let a = CatalogDatasetAccess::new(
        Arc::clone(&prov),
        registry(1000, AccessMode::ReadWrite, true),
    );
    let handle = a
        .allocate(&dd("USR.SEAM", Recfm::U, 0, Dsorg::PS))
        .expect("allocate");
    let mut w = a.open(&handle, AccessIntent::Write).expect("open");
    a.put(&mut w, &rec(b"seam-bytes")).expect("put");
    a.close(w).expect("close");
    // The native provider lists the physical object the seam wrote.
    let listed = prov.list().expect("list");
    assert!(
        listed.iter().any(|(_, name)| name == "USR.SEAM"),
        "physical object not created through the StorageProvider seam"
    );
}

// === allocate honours Volume status (Req 34.4; volume-model 2.1/2.2) =================

#[test]
fn allocate_rejects_offline_volume() {
    // Validates: Requirement 34.4; volume-model Requirement 2.1
    let dir = TempDir::new().unwrap();
    let a = CatalogDatasetAccess::new(provider(&dir), registry(1000, AccessMode::ReadWrite, false));
    let err = a
        .allocate(&dd("USR.OFF", Recfm::FB, 3, Dsorg::PS))
        .expect_err("offline");
    assert!(
        matches!(err, DatasetError::VolumeUnavailable { .. }),
        "got {err:?}"
    );
}

#[test]
fn allocate_rejects_readonly_volume() {
    // Validates: Requirement 34.4; volume-model Requirement 2.2
    let dir = TempDir::new().unwrap();
    let a = CatalogDatasetAccess::new(provider(&dir), registry(1000, AccessMode::ReadOnly, true));
    let err = a
        .allocate(&dd("USR.RO", Recfm::FB, 3, Dsorg::PS))
        .expect_err("read-only");
    assert!(
        matches!(err, DatasetError::ReadOnlyVolume { .. }),
        "got {err:?}"
    );
}

// === SPACE charging and the two failures (Req 34.4; volume-model 12.4) ===============

#[test]
fn allocate_charges_space_against_volume() {
    // Validates: Requirement 34.4; volume-model Requirement 12.4
    let dir = TempDir::new().unwrap();
    // Volume with exactly one track: a 1-track primary allocation succeeds and
    // consumes all free space; a second allocation then reports VolumeFull.
    let a = access(&dir, 1);
    a.allocate(&dd("USR.A", Recfm::FB, 3, Dsorg::PS))
        .expect("first allocate");
    let err = a
        .allocate(&dd("USR.B", Recfm::FB, 3, Dsorg::PS))
        .expect_err("full");
    assert!(
        matches!(err, DatasetError::VolumeFull { .. }),
        "got {err:?}"
    );
}

#[test]
fn put_growth_surfaces_x37_space_abend() {
    // Validates: Requirement 34.4; volume-model Requirement 12.4
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    // max_extents=1 means no secondary extent may be acquired; a write larger
    // than one track triggers the x37-style abend on close.
    let mut req = dd("USR.X37", Recfm::U, 0, Dsorg::PS);
    req.max_extents = 1;
    let handle = a.allocate(&req).expect("allocate");
    let mut w = a.open(&handle, AccessIntent::Write).expect("open");
    // One track is 56664 bytes; write just over to force growth.
    a.put(&mut w, &rec(&vec![b'Z'; 56_665])).expect("put");
    let err = a.close(w).expect_err("x37");
    assert!(
        matches!(err, DatasetError::SpaceAbend { .. }),
        "got {err:?}"
    );
}

// === Opaque handle (Req 34.5) ===================================================

#[test]
fn allocate_returns_opaque_handle() {
    // Validates: Requirement 34.5 -- handle exposes DSN only, no raw path
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let handle = a
        .allocate(&dd("USR.OPAQUE", Recfm::FB, 3, Dsorg::PS))
        .expect("allocate");
    assert_eq!(handle.dsn(), "USR.OPAQUE");
    assert_eq!(handle.dsorg(), Dsorg::PS);
    // The only public accessors are dsn()/dsorg(); there is no storage_path
    // getter -- the physical locator stays inside the opaque inner payload.
}

#[test]
fn allocate_rejects_duplicate_dsn() {
    // Validates: Requirement 34.1 -- duplicate allocate reports AlreadyExists
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    a.allocate(&dd("USR.DUP", Recfm::FB, 3, Dsorg::PS))
        .expect("first");
    let err = a
        .allocate(&dd("USR.DUP", Recfm::FB, 3, Dsorg::PS))
        .expect_err("dup");
    assert!(
        matches!(err, DatasetError::AlreadyExists { .. }),
        "got {err:?}"
    );
}

// === resolve-by-DSN (Req 34.5; dataset-allocator Req 19.3) =====================

#[test]
fn resolve_by_dsn_returns_handle_matching_the_dsn() {
    // Validates: Requirement 34.5; dataset-allocator Requirement 19.3
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    a.allocate(&dd("USR.RESOLVE", Recfm::FB, 80, Dsorg::PS))
        .expect("allocate");
    let handle = a
        .resolve("USR.RESOLVE", AccessIntent::Read)
        .expect("resolve existing dataset");
    assert_eq!(handle.dsn(), "USR.RESOLVE");
    assert_eq!(handle.dsorg(), Dsorg::PS);
    // The resolved handle is usable with open, proving it carries a real
    // locator (no raw path exposed on the surface).
    let open = a.open(&handle, AccessIntent::Read).expect("open resolved");
    assert_eq!(open.handle().dsn(), "USR.RESOLVE");
}

#[test]
fn resolve_unknown_dsn_reports_not_found() {
    // Validates: Requirement 34.5 -- resolving an uncatalogued DSN is NotFound
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    let err = a
        .resolve("USR.NOPE", AccessIntent::Read)
        .expect_err("unknown dsn");
    assert!(matches!(err, DatasetError::NotFound { .. }), "got {err:?}");
}

#[test]
fn resolve_normalises_dsn_case() {
    // Validates: Requirement 34.5 -- resolution uses the catalog's uppercase key
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    a.allocate(&dd("USR.CASED", Recfm::FB, 80, Dsorg::PS))
        .expect("allocate");
    let handle = a
        .resolve("usr.cased", AccessIntent::Read)
        .expect("resolve lower-case dsn");
    assert_eq!(handle.dsn(), "USR.CASED");
}

#[test]
fn dyn_dataset_access_can_resolve_by_dsn() {
    // Validates: Requirement 34.8 -- resolve is object-safe (callable via dyn)
    let dir = TempDir::new().unwrap();
    let a = access(&dir, 1000);
    a.allocate(&dd("USR.DYN", Recfm::FB, 80, Dsorg::PS))
        .expect("allocate");
    let dyn_access: &dyn DatasetAccess = &a;
    let handle = dyn_access
        .resolve("USR.DYN", AccessIntent::Read)
        .expect("resolve via dyn");
    assert_eq!(handle.dsn(), "USR.DYN");
}

// Error-mapping, object-safety, VSAM point(), and dispose tests live in the
// sibling `impl_io_tests.rs` (split for the ~200-line testing.md guidance).
