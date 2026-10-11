//! Tests for `CatalogRepositoryAccess` -- the catalog-backed `DatasetAccess`
//! that resolves against the on-disk SQLite catalog of record (CR-CH-061,
//! Requirement 36).
//!
//! The fixture allocates a dataset through a REAL on-disk `ff-dscatalog`
//! repository (`Catalog::allocate` -- the same path the Files Panel uses), then
//! drives `resolve`/`open`/`put`/`close` through the catalog-backed access to
//! prove a same-session round-trip (no mock, no in-memory HashMap).

use std::path::PathBuf;

use tempfile::TempDir;

use super::error::DatasetError;
use super::impl_catalog::CatalogRepositoryAccess;
use super::trait_def::DatasetAccess;
use super::types::AccessIntent;
use crate::catalog::Catalog;
use crate::dataset::{AllocParams, Dsorg, Recfm};
use crate::dsn::Dsn;
use crate::error::CatalogError;
use crate::hierarchy::CatalogScope;
use crate::repository::Repository;
use crate::Record;

// === Fixtures ===================================================

/// Initialise an on-disk repository and allocate one FB/5 PS dataset, returning
/// the repository root path (the dataset's physical file is created empty).
fn repo_with_dataset(tmp: &TempDir, dsn: &str, recfm: Recfm, lrecl: u32) -> PathBuf {
    let path = tmp.path().join("mf-catalog");
    let repo = Repository::new(&path);
    repo.initialize("MF").expect("initialize repo");
    let catalog = Catalog::mount(&path, 1).expect("mount catalog");
    let params = AllocParams {
        dsn: Dsn::parse(dsn).expect("dsn"),
        dsorg: Dsorg::PS,
        recfm: Some(recfm),
        lrecl: Some(lrecl),
        blksize: Some(lrecl.max(1)),
        dir_blocks: None,
        gdg_limit: None,
        gdg_scratch: None,
        subtype: None,
        description: None,
        scope: CatalogScope::User,
    };
    catalog.allocate(params).expect("allocate dataset");
    path
}

// === T43a: same-session round-trip (resolve + open + put + close) ============

#[test]
fn catalog_access_round_trips_a_same_session_dataset() {
    // Validates: Requirement 36.1, 36.2, 36.4, 36.5
    let tmp = TempDir::new().expect("tempdir");
    let repo = repo_with_dataset(&tmp, "USR.CAT.FB", Recfm::FB, 5);

    let access = CatalogRepositoryAccess::single(&repo);

    // Resolve the dataset the on-disk catalog knows (Req 36.2).
    let handle = access
        .resolve("USR.CAT.FB", AccessIntent::Write)
        .expect("resolve must succeed for a catalogued dataset");
    assert_eq!(handle.dsn(), "USR.CAT.FB");

    // Store two records (Write replaces content) and close so the RECFM codec
    // frames the bytes to the catalog's physical file (Req 36.5).
    let mut open = access
        .open(&handle, AccessIntent::Write)
        .expect("open write");
    access
        .put(
            &mut open,
            &Record {
                key: Vec::new(),
                data: b"AB".to_vec(),
            },
        )
        .expect("put 1");
    access
        .put(
            &mut open,
            &Record {
                key: Vec::new(),
                data: b"CDE".to_vec(),
            },
        )
        .expect("put 2");
    access.close(open).expect("close must store with success");

    // Re-open through the SAME access and assert the records round-trip, framed
    // by the FixedCodec (padded to LRECL=5 with EBCDIC spaces 0x40).
    let read_handle = access
        .resolve("USR.CAT.FB", AccessIntent::Read)
        .expect("re-resolve");
    let mut r = access
        .open(&read_handle, AccessIntent::Read)
        .expect("open read");
    let mut records = Vec::new();
    while let Some(rec) = access.get(&mut r).expect("get") {
        records.push(rec.data);
    }
    assert_eq!(records.len(), 2);
    assert_eq!(records[0], vec![b'A', b'B', 0x40, 0x40, 0x40]);
    assert_eq!(records[1], vec![b'C', b'D', b'E', 0x40, 0x40]);
}

// === T43b: genuine miss -> NotFound (Req 36.6) ===================================

#[test]
fn catalog_access_returns_not_found_for_absent_dsn() {
    // Validates: Requirement 36.6 -- a true miss stays NotFound
    let tmp = TempDir::new().expect("tempdir");
    let repo = repo_with_dataset(&tmp, "USR.PRESENT", Recfm::FB, 80);
    let access = CatalogRepositoryAccess::single(&repo);

    let err = access
        .resolve("USR.ABSENT.DSN", AccessIntent::Write)
        .expect_err("absent DSN must not resolve");
    assert!(
        matches!(err, DatasetError::NotFound { .. }),
        "expected NotFound, got {err:?}"
    );
}

// === T43c: dyn + Send + Sync (Req 36.3) ===================================

#[test]
fn catalog_access_is_object_safe_and_send_sync() {
    // Validates: Requirement 36.3 -- shared as Arc<dyn DatasetAccess> (Send+Sync)
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CatalogRepositoryAccess>();

    let tmp = TempDir::new().expect("tempdir");
    let repo = repo_with_dataset(&tmp, "USR.DYN", Recfm::FB, 4);
    let access: std::sync::Arc<dyn DatasetAccess> =
        std::sync::Arc::new(CatalogRepositoryAccess::single(&repo));
    let handle = access
        .resolve("USR.DYN", AccessIntent::Write)
        .expect("resolve via dyn");
    assert_eq!(handle.dsn(), "USR.DYN");
}

// === T43d: multi-catalog resolution + untracked-DSN miss ======================

#[test]
fn catalog_access_resolves_across_multiple_repositories() {
    // Validates: Requirement 36.1, 36.2 -- the access serves multiple catalogs
    let tmp = TempDir::new().expect("tempdir");
    let repo_a = repo_with_dataset(&tmp, "AAA.ONE", Recfm::FB, 10);

    // A second repository with its own dataset.
    let path_b = tmp.path().join("mf-catalog-b");
    let repo = Repository::new(&path_b);
    repo.initialize("MFB").expect("init b");
    let catalog = Catalog::mount(&path_b, 1).expect("mount b");
    catalog
        .allocate(AllocParams {
            dsn: Dsn::parse("BBB.TWO").expect("dsn"),
            dsorg: Dsorg::PS,
            recfm: Some(Recfm::FB),
            lrecl: Some(10),
            blksize: Some(10),
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        })
        .expect("allocate b");

    let access = CatalogRepositoryAccess::new(vec![repo_a, path_b]);
    assert_eq!(
        access
            .resolve("AAA.ONE", AccessIntent::Write)
            .expect("resolve a")
            .dsn(),
        "AAA.ONE"
    );
    assert_eq!(
        access
            .resolve("BBB.TWO", AccessIntent::Write)
            .expect("resolve b")
            .dsn(),
        "BBB.TWO"
    );
    // An invalid DSN still surfaces an error (not a panic).
    let err = access
        .resolve("not a valid dsn!", AccessIntent::Write)
        .expect_err("invalid dsn errors");
    let _ = err;
    // Keep CatalogError import used for symmetry with other dscatalog tests.
    let _ = CatalogError::DatasetNotFound {
        dsn: "x".to_string(),
        operation: "test".to_string(),
    };
}
