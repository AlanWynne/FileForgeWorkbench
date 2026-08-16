//! Property-based tests for save point and dirty flag semantics.
//! Feature: undo-redo-transactions

use proptest::prelude::*;

use ff_undo_redo::{DocumentUndoManager, UndoConfig};

// --- Property 4: Save Point Dirty Flag Derivation ---
// **Validates: Requirements 5.1, 5.3, 5.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 4: is_dirty() == (position != save_point || detach_point.is_some())
    #[test]
    fn save_point_dirty_flag_derivation(
        num_commits in 1usize..20,
        save_at in 0usize..20,
        undo_count in 0usize..20,
    ) {
        // Feature: undo-redo-transactions, Property 4: dirty flag derivation
        let mut mgr = DocumentUndoManager::new(UndoConfig::default());

        for i in 0..num_commits {
            mgr.begin_transaction(&format!("op{}", i));
            mgr.record_insert(i as u64, b"x");
            mgr.end_transaction();
        }

        // Save at some point
        if save_at < num_commits {
            // Undo to save_at position
            let undos_needed = num_commits - save_at;
            for _ in 0..undos_needed.min(num_commits) {
                if mgr.can_undo() {
                    mgr.undo().unwrap();
                }
            }
            mgr.set_save_point();
            prop_assert!(!mgr.is_dirty(), "should be clean at save point");
        }

        // Do some undos
        for _ in 0..undo_count {
            if mgr.can_undo() {
                mgr.undo().unwrap();
            }
        }

        // Verify: if at save point then not dirty, if not at save point then dirty
        if mgr.is_at_save_point() {
            prop_assert!(!mgr.is_dirty(),
                "should not be dirty when at save point");
        } else {
            prop_assert!(mgr.is_dirty(),
                "should be dirty when not at save point");
        }
    }
}

// --- Property 5: Detach Point Semantics ---
// **Validates: Requirement 5.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 5: Once detached, is_dirty() always true regardless of undo/redo.
    #[test]
    fn detach_point_semantics(
        num_commits in 3usize..15,
        subsequent_ops in 0usize..10,
    ) {
        // Feature: undo-redo-transactions, Property 5: detach makes permanently dirty
        let mut mgr = DocumentUndoManager::new(UndoConfig::default());

        // Build some history
        for i in 0..num_commits {
            mgr.begin_transaction(&format!("op{}", i));
            mgr.record_insert(i as u64, b"x");
            mgr.end_transaction();
        }

        // Save, then undo, then commit (creates detach)
        mgr.set_save_point();
        mgr.undo().unwrap();
        mgr.begin_transaction("diverge");
        mgr.record_insert(0, b"y");
        mgr.end_transaction();

        // Verify detach
        if mgr.after_detach_point() {
            // Once detached, dirty should always be true
            prop_assert!(mgr.is_dirty(), "should be dirty after detach");

            // Do subsequent undo/redo — should still be dirty
            for _ in 0..subsequent_ops {
                if mgr.can_undo() {
                    mgr.undo().unwrap();
                    prop_assert!(mgr.is_dirty(),
                        "should remain dirty after undo when detached");
                }
            }
            for _ in 0..subsequent_ops {
                if mgr.can_redo() {
                    mgr.redo().unwrap();
                    prop_assert!(mgr.is_dirty(),
                        "should remain dirty after redo when detached");
                }
            }
        }
    }
}
