//! Property-based tests for coalescing rules.
//! Feature: undo-redo-transactions

use proptest::prelude::*;

use ff_undo_redo::{DocumentUndoManager, UndoConfig};

// --- Property 6: Coalescing Contiguity Rule ---
// **Validates: Requirements 6.1, 6.7**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 6: Contiguous single-char inserts merge into one transaction.
    #[test]
    fn coalescing_contiguity_rule(
        num_chars in 2usize..50,
    ) {
        // Feature: undo-redo-transactions, Property 6: contiguous inserts coalesce
        let config = UndoConfig {
            max_levels: 100,
            coalesce_timeout_ms: 10_000, // long timeout to avoid timing issues
            selection_history_enabled: true,
            recovery_interval_seconds: 0,
        };
        let mut mgr = DocumentUndoManager::new(config);

        // Type N contiguous characters one at a time
        for i in 0..num_chars {
            mgr.record_insert(i as u64, b"a");
        }

        // All should be coalesced into a single transaction
        prop_assert_eq!(mgr.undo_depth(), 1,
            "contiguous inserts should coalesce into 1 transaction, got {}",
            mgr.undo_depth());
    }
}

// --- Property 7: Coalescing Boundary Events ---
// **Validates: Requirement 6.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 7: Boundary events break coalescing into separate transactions.
    #[test]
    fn coalescing_boundary_events(
        num_groups in 2usize..10,
        chars_per_group in 1usize..10,
    ) {
        // Feature: undo-redo-transactions, Property 7: boundaries break coalescing
        let config = UndoConfig {
            max_levels: 100,
            coalesce_timeout_ms: 10_000,
            selection_history_enabled: true,
            recovery_interval_seconds: 0,
        };
        let mut mgr = DocumentUndoManager::new(config);

        let mut pos = 0u64;
        for _group in 0..num_groups {
            for _ in 0..chars_per_group {
                mgr.record_insert(pos, b"x");
                pos += 1;
            }
            // Break coalescing between groups (simulates cursor move)
            mgr.break_coalesce();
        }

        // Should have exactly num_groups transactions
        prop_assert_eq!(mgr.undo_depth(), num_groups,
            "should have {} transactions (one per group), got {}",
            num_groups, mgr.undo_depth());
    }
}
