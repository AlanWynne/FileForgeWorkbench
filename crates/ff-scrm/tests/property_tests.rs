//! Property tests for ff-scrm.
//!
//! Feature: screen-snapshot-scrm, Property 2: a collection round-trips through
//! the native zip archive unchanged, at scale (Requirement 18.1 says the
//! subsystem supports at least 10,000 captures).
//!
//! Validates: screen-snapshot-scrm Requirement 12.5, 17.3, 18.1.

use chrono::{DateTime, Utc};
use ff_screen_model::{Field, ScreenModel};
use ff_scrm::{load_archive, save_archive, ScreenCollection};
use proptest::prelude::*;

fn ts() -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).unwrap()
}

fn build(n: usize) -> ScreenCollection {
    let mut c = ScreenCollection::new("cid", "Scale", "alan", ts());
    for i in 0..n {
        c.append_capture(
            format!("cap{i}"),
            ts(),
            ScreenModel::new(format!("Screen {i}")).with_field(Field::new("Seq", i.to_string())),
        );
    }
    c
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // Feature: screen-snapshot-scrm, Property 2 -- archive round-trip for a
    // range of collection sizes.
    #[test]
    fn collection_round_trips_through_archive(n in 0usize..50) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.ffscrm");
        let original = build(n);
        save_archive(&original, &path).unwrap();
        let loaded = load_archive(&path).unwrap();
        prop_assert_eq!(loaded, original);
    }
}

// Validates: Requirement 18.1 -- at least 10,000 captures are supported and
// round-trip through the archive. Kept as a single (non-proptest) test to avoid
// running a 10k-capture build 100 times.
#[test]
fn supports_ten_thousand_captures() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("big.ffscrm");
    let original = build(10_000);
    save_archive(&original, &path).unwrap();
    let loaded = load_archive(&path).unwrap();
    assert_eq!(loaded.len(), 10_000);
    assert_eq!(loaded, original);
}
