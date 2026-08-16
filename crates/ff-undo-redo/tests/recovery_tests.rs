//! Property-based tests for recovery file round-trip.
//! Feature: undo-redo-transactions

use proptest::prelude::*;

use ff_undo_redo::recovery::{deserialize_recovery, serialize_for_recovery};
use ff_undo_redo::scrap::ScrapStack;

// --- Property 12: Recovery Round-Trip Integrity ---
// **Validates: Requirements 8.5, 8.7, 16.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 12: serialize then deserialize produces equivalent state.
    #[test]
    fn recovery_round_trip_integrity(
        scrap_data in prop::collection::vec(any::<u8>(), 0..500),
        save_point in 0usize..1000,
        current_action in 0usize..1000,
        has_record_data in any::<bool>(),
        record_data in prop::collection::vec(any::<u8>(), 0..200),
    ) {
        // Feature: undo-redo-transactions, Property 12: recovery round-trip
        let mut scrap = ScrapStack::new();
        if !scrap_data.is_empty() {
            scrap.push(&scrap_data);
        }

        let record_ref = if has_record_data {
            Some(record_data.as_slice())
        } else {
            None
        };

        let serialized = serialize_for_recovery(
            &scrap, save_point, current_action, record_ref,
        ).unwrap();

        let payload = deserialize_recovery(&serialized).unwrap();

        prop_assert_eq!(payload.save_point, save_point);
        prop_assert_eq!(payload.current_action, current_action);
        prop_assert_eq!(payload.scrap_data, scrap.as_bytes());

        if has_record_data {
            prop_assert_eq!(payload.record_id_data.as_deref(), Some(record_data.as_slice()));
        } else {
            prop_assert!(payload.record_id_data.is_none());
        }
    }
}
