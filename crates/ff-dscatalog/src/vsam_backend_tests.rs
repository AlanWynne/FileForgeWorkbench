//! Unit tests for `VsamBackend` -- the in-crate bridge to the concrete VSAM
//! backends (CR-CH-059, dataset-catalog Requirement 34.1/34.2/35.3).

use tempfile::TempDir;
use uuid::Uuid;

use super::VsamBackend;
use crate::storage::{AlternateIndex, KeyCollation};
use crate::vsam_service::{BrowseDirection, Record, VsamError, VsamParams, VsamType};

fn ksds_params() -> VsamParams {
    VsamParams {
        key_length: Some(4),
        key_offset: Some(0),
        record_length: Some(32),
        slot_size: None,
    }
}

#[test]
fn ksds_keyed_read_after_write_round_trip() {
    // Validates: Requirement 34.1 -- KSDS keyed put/get round-trip
    let dir = TempDir::new().unwrap();
    let backend =
        VsamBackend::open_for(dir.path(), Uuid::new_v4(), VsamType::Ksds, &ksds_params()).unwrap();
    backend
        .put_keyed(&Record {
            key: b"KEY1".to_vec(),
            data: b"first".to_vec(),
        })
        .expect("put");
    let read = backend.get_keyed(b"KEY1").expect("get").expect("present");
    assert_eq!(read.key, b"KEY1".to_vec());
    assert_eq!(read.data, b"first".to_vec());
    assert!(backend.get_keyed(b"NONE").expect("get").is_none());
}

#[test]
fn ksds_put_updates_existing_key() {
    // Validates: Requirement 34.2 -- a second put on the same key updates it
    let dir = TempDir::new().unwrap();
    let backend =
        VsamBackend::open_for(dir.path(), Uuid::new_v4(), VsamType::Ksds, &ksds_params()).unwrap();
    backend
        .put_keyed(&Record {
            key: b"KEYA".to_vec(),
            data: b"one".to_vec(),
        })
        .unwrap();
    backend
        .put_keyed(&Record {
            key: b"KEYA".to_vec(),
            data: b"two".to_vec(),
        })
        .unwrap();
    assert_eq!(
        backend.get_keyed(b"KEYA").unwrap().unwrap().data,
        b"two".to_vec()
    );
}

#[test]
fn ksds_browse_returns_records_in_key_order() {
    // Validates: Requirement 34.2 -- browse yields sequential key order
    let dir = TempDir::new().unwrap();
    let backend =
        VsamBackend::open_for(dir.path(), Uuid::new_v4(), VsamType::Ksds, &ksds_params()).unwrap();
    for key in [b"KEYC", b"KEYA", b"KEYB"] {
        backend
            .put_keyed(&Record {
                key: key.to_vec(),
                data: b"x".to_vec(),
            })
            .unwrap();
    }
    let forward: Vec<Vec<u8>> = backend
        .browse_from(b"KEYA", BrowseDirection::Forward)
        .unwrap()
        .into_iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(
        forward,
        vec![b"KEYA".to_vec(), b"KEYB".to_vec(), b"KEYC".to_vec()]
    );
    let from_b: Vec<Vec<u8>> = backend
        .browse_from(b"KEYB", BrowseDirection::Forward)
        .unwrap()
        .into_iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(from_b, vec![b"KEYB".to_vec(), b"KEYC".to_vec()]);
}

#[test]
fn ksds_alternate_index_lookup_resolves_primary_keys() {
    // Validates: Requirement 34.2 -- KSDS alternate-index lookup
    let dir = TempDir::new().unwrap();
    let params = VsamParams {
        key_length: Some(4),
        key_offset: Some(0),
        record_length: Some(16),
        slot_size: None,
    };
    let backend =
        VsamBackend::open_for(dir.path(), Uuid::new_v4(), VsamType::Ksds, &params).unwrap();
    // Record layout: primary key (0..4), alt key (4..8).
    backend
        .put_keyed(&Record {
            key: b"K001".to_vec(),
            data: b"K001SMITH".to_vec(),
        })
        .unwrap();
    backend
        .put_keyed(&Record {
            key: b"K002".to_vec(),
            data: b"K002SMITH".to_vec(),
        })
        .unwrap();
    backend
        .add_alternate_index(&AlternateIndex {
            name: "BY_SURNAME".to_string(),
            offset: 4,
            length: 5,
            unique: false,
            collation: KeyCollation::Binary,
        })
        .expect("define aix");
    backend
        .rebuild_alternate_index("BY_SURNAME")
        .expect("rebuild aix");
    let mut primaries = backend.lookup_alternate("BY_SURNAME", b"SMITH").unwrap();
    primaries.sort();
    assert_eq!(primaries, vec![b"K001".to_vec(), b"K002".to_vec()]);
}

#[test]
fn rrds_relative_round_trip_and_rejects_rrn_zero() {
    // Validates: Requirement 34.1 -- RRDS RRN positioning; RRN 0 rejected
    let dir = TempDir::new().unwrap();
    let backend = VsamBackend::open_for(
        dir.path(),
        Uuid::new_v4(),
        VsamType::Rrds,
        &VsamParams::default(),
    )
    .unwrap();
    backend.put_relative(5, b"five").expect("write rrn 5");
    assert_eq!(backend.get_relative(5).unwrap().unwrap().data, b"five");
    assert!(backend.get_relative(9).unwrap().is_none());
    assert!(matches!(
        backend.put_relative(0, b"zero"),
        Err(VsamError::StorageError(_))
    ));
    assert!(matches!(
        backend.get_relative(0),
        Err(VsamError::StorageError(_))
    ));
}

#[test]
fn esds_append_preserves_insertion_order_and_stable_address() {
    // Validates: Requirement 34.1 -- ESDS append with stable address
    let dir = TempDir::new().unwrap();
    let backend = VsamBackend::open_for(
        dir.path(),
        Uuid::new_v4(),
        VsamType::Esds,
        &VsamParams::default(),
    )
    .unwrap();
    let first = backend.append_entry(b"one").expect("append one");
    let second = backend.append_entry(b"two").expect("append two");
    assert!(second > first);
    assert_eq!(backend.get_address(first).unwrap().unwrap().data, b"one");
    let order: Vec<Vec<u8>> = backend
        .sequential()
        .unwrap()
        .into_iter()
        .map(|r| r.data)
        .collect();
    assert_eq!(order, vec![b"one".to_vec(), b"two".to_vec()]);
}

#[test]
fn lds_has_no_concrete_backend() {
    // Validates: Requirement 35.3 -- LDS documented as unsupported for now
    let dir = TempDir::new().unwrap();
    let result = VsamBackend::open_for(
        dir.path(),
        Uuid::new_v4(),
        VsamType::Lds,
        &VsamParams::default(),
    );
    assert!(matches!(
        result,
        Err(VsamError::UnsupportedOperation { .. })
    ));
}

#[test]
fn keyed_op_on_rrds_backend_is_unsupported() {
    // Validates: Requirement 34.1 -- a keyed op on a non-KSDS backend is rejected
    let dir = TempDir::new().unwrap();
    let backend = VsamBackend::open_for(
        dir.path(),
        Uuid::new_v4(),
        VsamType::Rrds,
        &VsamParams::default(),
    )
    .unwrap();
    assert!(matches!(
        backend.get_keyed(b"KEY1"),
        Err(VsamError::UnsupportedOperation { .. })
    ));
}
