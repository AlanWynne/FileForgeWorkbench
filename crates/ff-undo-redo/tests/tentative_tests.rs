//! Property-based tests for tentative actions (IME).
//! Feature: undo-redo-transactions

use proptest::prelude::*;

use ff_undo_redo::{DocumentUndoManager, UndoConfig};

// --- Property 11: Tentative Action Isolation ---
// **Validates: Requirements 12.1, 12.3, 12.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 11: rollback leaves no trace; commit makes permanent.
    #[test]
    fn tentative_action_isolation(
        base_ops in 0usize..5,
        tentative_ops in 1usize..5,
        do_commit in any::<bool>(),
    ) {
        // Feature: undo-redo-transactions, Property 11: tentative isolation
        let config = UndoConfig {
            max_levels: 100,
            coalesce_timeout_ms: 10_000,
            selection_history_enabled: true,
            recovery_interval_seconds: 0,
        };
        let mut mgr = DocumentUndoManager::new(config);

        // Record base operations
        for i in 0..base_ops {
            mgr.begin_transaction(&format!("base{}", i));
            mgr.record_insert(i as u64, b"b");
            mgr.end_transaction();
        }
        let depth_before_tentative = mgr.undo_depth();

        // Start tentative mode
        mgr.tentative_start();
        prop_assert!(mgr.tentative_active());

        // Record tentative operations
        for i in 0..tentative_ops {
            mgr.begin_transaction(&format!("tent{}", i));
            mgr.record_insert((base_ops + i) as u64, b"t");
            mgr.end_transaction();
        }

        if do_commit {
            // Commit: tentative actions become permanent
            mgr.tentative_commit();
            prop_assert!(!mgr.tentative_active());
            prop_assert_eq!(mgr.undo_depth(), depth_before_tentative + tentative_ops,
                "commit should make tentative ops permanent");
        } else {
            // Rollback: tentative actions removed without trace
            let rolled_back = mgr.tentative_rollback();
            prop_assert!(!mgr.tentative_active());
            prop_assert_eq!(rolled_back, tentative_ops);
            prop_assert_eq!(mgr.undo_depth(), depth_before_tentative,
                "rollback should restore stack to pre-tentative state");
        }
    }
}
