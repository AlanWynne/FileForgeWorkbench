//! Property-based tests for transaction nesting.
//! Feature: undo-redo-transactions

use proptest::prelude::*;

use ff_undo_redo::{DocumentUndoManager, UndoConfig};

// --- Property 8: Transaction Nesting Depth Tracking ---
// **Validates: Requirements 3.3, 3.7**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 8: depth == begins - ends; commits only at depth 0.
    #[test]
    fn transaction_nesting_depth_tracking(
        nesting_depth in 1usize..10,
    ) {
        // Feature: undo-redo-transactions, Property 8: nesting depth tracking
        let mut mgr = DocumentUndoManager::new(UndoConfig::default());

        // Nest begins
        for i in 0..nesting_depth {
            mgr.begin_transaction(&format!("level{}", i));
            prop_assert_eq!(mgr.transaction_depth(), i + 1,
                "depth should be {} after {} begins", i + 1, i + 1);
        }

        // Add an operation at the deepest level
        mgr.record_insert(0, b"x");

        // End all but the last
        for i in 0..(nesting_depth - 1) {
            mgr.end_transaction();
            prop_assert_eq!(mgr.transaction_depth(), nesting_depth - 1 - i,
                "depth should decrease on end");
            // Should NOT have committed yet
            prop_assert_eq!(mgr.undo_depth(), 0,
                "should not commit until outermost end");
        }

        // End the outermost — should commit
        mgr.end_transaction();
        prop_assert_eq!(mgr.transaction_depth(), 0);
        prop_assert_eq!(mgr.undo_depth(), 1,
            "should commit exactly one transaction at depth 0");
    }
}
