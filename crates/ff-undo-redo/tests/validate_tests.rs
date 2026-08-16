//! Property-based tests for history validation.
//! Feature: undo-redo-transactions

use proptest::prelude::*;

use ff_undo_redo::edit_op::EditOperation;
use ff_undo_redo::transaction::Transaction;
use ff_undo_redo::validate::validate_history;

// --- Property 13: Validation Detects Inconsistency ---
// **Validates: Requirements 16.1, 16.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 13: valid histories pass, corrupted histories fail.
    #[test]
    fn validation_detects_inconsistency(
        num_inserts in 1usize..20,
        insert_sizes in prop::collection::vec(1u32..100, 1..20),
    ) {
        // Feature: undo-redo-transactions, Property 13: validation correctness
        let sizes: Vec<u32> = insert_sizes.into_iter().take(num_inserts).collect();

        // Build a valid history
        let mut offset = 0u64;
        let mut txns = Vec::new();
        let mut total_inserted: u64 = 0;

        for (i, &size) in sizes.iter().enumerate() {
            let txn = Transaction {
                name: format!("op{}", i),
                timestamp: chrono::Utc::now(),
                operations: vec![EditOperation::Insert {
                    position: offset,
                    length: size,
                    scrap_offset: total_inserted,
                }],
                selection_before: None,
                selection_after: None,
                may_coalesce: true,
            };
            total_inserted += size as u64;
            offset += size as u64;
            txns.push(txn);
        }

        let txn_refs: Vec<&Transaction> = txns.iter().collect();
        let expected_length = total_inserted;

        // Valid history should pass
        prop_assert!(validate_history(&txn_refs, 0, expected_length).is_ok(),
            "valid history should pass validation");

        // Corrupted history (wrong expected length) should fail
        if expected_length > 0 {
            prop_assert!(validate_history(&txn_refs, 0, expected_length + 1).is_err(),
                "corrupted length should fail validation");
        }
    }
}
