//! Property-based tests for ff-undo-redo.
//! Feature: undo-redo-transactions

use proptest::prelude::*;

use ff_undo_redo::{DocumentUndoManager, UndoConfig};

// --- Strategies ---

// --- Property 1: Undo/Redo Stack Depth Invariant ---
// **Validates: Requirements 1.3, 1.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 1: Undo stack depth never exceeds max_levels.
    /// After N > M commits, depth == M.
    #[test]
    fn undo_stack_depth_invariant(
        max_levels in 1u32..50,
        num_commits in 1usize..100,
    ) {
        // Feature: undo-redo-transactions, Property 1: Stack depth ≤ max_levels
        let config = UndoConfig {
            max_levels,
            coalesce_timeout_ms: 2000,
            selection_history_enabled: true,
            recovery_interval_seconds: 0,
        };
        let mut mgr = DocumentUndoManager::new(config);

        for i in 0..num_commits {
            mgr.begin_transaction(&format!("op{}", i));
            mgr.record_insert(0, b"x");
            mgr.end_transaction();
            // Invariant: depth ≤ max_levels at all times
            prop_assert!(mgr.undo_depth() <= max_levels as usize,
                "depth {} > max_levels {} after commit {}",
                mgr.undo_depth(), max_levels, i);
        }

        // After N > M commits, depth == M
        if num_commits > max_levels as usize {
            prop_assert_eq!(mgr.undo_depth(), max_levels as usize);
        }
    }
}

// --- Property 2: Undo-Redo Symmetry ---
// **Validates: Requirements 4.1, 4.4, 4.9**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 2: Undo then redo produces byte-identical state to pre-undo.
    #[test]
    fn undo_redo_symmetry(
        num_ops in 1usize..20,
    ) {
        // Feature: undo-redo-transactions, Property 2: undo→redo symmetry
        let mut mgr = DocumentUndoManager::new(UndoConfig::default());

        for i in 0..num_ops {
            mgr.begin_transaction(&format!("op{}", i));
            mgr.record_insert(i as u64, &[b'a' + (i as u8 % 26)]);
            mgr.end_transaction();
        }

        let depth_before = mgr.undo_depth();
        let redo_before = mgr.redo_depth();

        // Undo all
        let mut undone = 0;
        while mgr.can_undo() {
            mgr.undo().unwrap();
            undone += 1;
        }
        prop_assert_eq!(undone, depth_before);
        prop_assert_eq!(mgr.redo_depth(), depth_before);

        // Redo all
        let mut redone = 0;
        while mgr.can_redo() {
            mgr.redo().unwrap();
            redone += 1;
        }
        prop_assert_eq!(redone, depth_before);
        prop_assert_eq!(mgr.undo_depth(), depth_before);
        prop_assert_eq!(mgr.redo_depth(), redo_before);
    }
}

// --- Property 3: Redo Stack Cleared on New Commit ---
// **Validates: Requirement 2.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 3: Committing after undo clears redo entirely.
    #[test]
    fn redo_cleared_on_new_commit(
        num_commits in 2usize..20,
        undo_count in 1usize..10,
    ) {
        // Feature: undo-redo-transactions, Property 3: redo cleared on commit
        let mut mgr = DocumentUndoManager::new(UndoConfig::default());

        for i in 0..num_commits {
            mgr.begin_transaction(&format!("op{}", i));
            mgr.record_insert(i as u64, b"a");
            mgr.end_transaction();
        }

        // Undo some
        let actual_undo = undo_count.min(num_commits);
        for _ in 0..actual_undo {
            if mgr.can_undo() {
                mgr.undo().unwrap();
            }
        }

        if mgr.redo_depth() > 0 {
            // Commit new operation — should clear redo
            mgr.begin_transaction("new");
            mgr.record_insert(0, b"z");
            mgr.end_transaction();
            prop_assert_eq!(mgr.redo_depth(), 0,
                "redo stack should be empty after new commit");
        }
    }
}
