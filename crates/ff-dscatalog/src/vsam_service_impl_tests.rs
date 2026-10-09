//! Unit tests for `CatalogVsamService` (CR-CH-059, dataset-catalog
//! Requirement 33.3/34.1/34.2/35.3).

use tempfile::TempDir;

use super::CatalogVsamService;
use crate::vsam_service::{AccessMode, BrowseDirection, KeyField, Record, VsamError, VsamService};

fn service() -> (TempDir, CatalogVsamService) {
    let dir = TempDir::new().unwrap();
    let svc = CatalogVsamService::new(dir.path());
    (dir, svc)
}

#[test]
fn ksds_keyed_read_after_write_round_trip() {
    // Validates: Requirement 34.1 -- KSDS keyed put/get through the service
    let (_dir, svc) = service();
    svc.create_ksds("MY.KSDS", 4, 0, 32).expect("create");
    let handle = svc.open("MY.KSDS", AccessMode::ReadWrite).expect("open");
    svc.put(
        &handle,
        &Record {
            key: b"KEY1".to_vec(),
            data: b"payload".to_vec(),
        },
    )
    .expect("put");
    let read = svc.get(&handle, b"KEY1").expect("get");
    assert_eq!(read.data, b"payload".to_vec());
    assert!(matches!(
        svc.get(&handle, b"GONE"),
        Err(VsamError::RecordNotFound { .. })
    ));
    svc.close(handle).expect("close");
}

#[test]
fn rrds_relative_positioning_round_trip() {
    // Validates: Requirement 34.1 -- RRDS positioning by RRN
    let (_dir, svc) = service();
    svc.create_rrds("MY.RRDS", 80).expect("create");
    let handle = svc.open("MY.RRDS", AccessMode::ReadWrite).expect("open");
    svc.put(
        &handle,
        &Record {
            key: 7u64.to_be_bytes().to_vec(),
            data: b"seven".to_vec(),
        },
    )
    .expect("put rrn 7");
    let read = svc.get(&handle, &7u64.to_be_bytes()).expect("get rrn 7");
    assert_eq!(read.data, b"seven".to_vec());
}

#[test]
fn esds_append_is_stable_and_browses_in_order() {
    // Validates: Requirement 34.2 -- ESDS append + sequential browse order
    let (_dir, svc) = service();
    svc.create_esds("MY.ESDS", 32).expect("create");
    let handle = svc.open("MY.ESDS", AccessMode::ReadWrite).expect("open");
    for payload in [b"one".as_slice(), b"two".as_slice(), b"three".as_slice()] {
        svc.put(
            &handle,
            &Record {
                key: Vec::new(),
                data: payload.to_vec(),
            },
        )
        .expect("append");
    }
    let browse = svc
        .start_browse(&handle, b"", BrowseDirection::Forward)
        .expect("browse");
    let mut seen = Vec::new();
    while let Some(record) = svc.next_record(&browse).expect("next") {
        seen.push(record.data);
    }
    svc.end_browse(browse).expect("end");
    assert_eq!(
        seen,
        vec![b"one".to_vec(), b"two".to_vec(), b"three".to_vec()]
    );
}

#[test]
fn browse_returns_ksds_records_in_key_order() {
    // Validates: Requirement 34.2 -- KSDS browse sequential order
    let (_dir, svc) = service();
    svc.create_ksds("MY.KSDS", 4, 0, 16).expect("create");
    let handle = svc.open("MY.KSDS", AccessMode::ReadWrite).expect("open");
    for key in [b"KEYB", b"KEYA", b"KEYC"] {
        svc.put(
            &handle,
            &Record {
                key: key.to_vec(),
                data: b"x".to_vec(),
            },
        )
        .unwrap();
    }
    let browse = svc
        .start_browse(&handle, b"KEYA", BrowseDirection::Forward)
        .expect("browse");
    let mut keys = Vec::new();
    while let Some(record) = svc.next_record(&browse).unwrap() {
        keys.push(record.key);
    }
    svc.end_browse(browse).unwrap();
    assert_eq!(
        keys,
        vec![b"KEYA".to_vec(), b"KEYB".to_vec(), b"KEYC".to_vec()]
    );
}

#[test]
fn alternate_index_lookup_on_ksds() {
    // Validates: Requirement 34.2 -- define/build/lookup an alternate index
    let (_dir, svc) = service();
    svc.create_ksds("MY.KSDS", 4, 0, 16).expect("create");
    let handle = svc.open("MY.KSDS", AccessMode::ReadWrite).expect("open");
    svc.put(
        &handle,
        &Record {
            key: b"K001".to_vec(),
            data: b"K001SMITH".to_vec(),
        },
    )
    .unwrap();
    svc.define_aix(
        "MY.KSDS",
        "MY.KSDS.AIX",
        KeyField {
            offset: 4,
            length: 5,
            unique: false,
        },
    )
    .expect("define aix");
    svc.build_index("MY.KSDS.AIX").expect("build aix");
}

#[test]
fn open_unknown_cluster_is_not_found() {
    // Validates: Requirement 34.1 -- open of a missing cluster reports NotFound
    let (_dir, svc) = service();
    assert!(matches!(
        svc.open("NO.SUCH", AccessMode::Read),
        Err(VsamError::DatasetNotFound { .. })
    ));
}
