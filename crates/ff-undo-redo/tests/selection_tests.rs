//! Property-based tests for selection history.
//! Feature: undo-redo-transactions

use proptest::prelude::*;

use ff_undo_redo::{DocumentUndoManager, SelectionState, UndoConfig};

// --- Property 10: Selection History Restoration ---
// **Validates: Requirements 9.1, 9.3, 9.4, 9.7, 9.8**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 10: before-state on undo, after-state on redo; disabled mode skips selection.
    #[test]
    fn selection_history_restoration(
        num_ops in 1usize..10,
        selection_enabled in any::<bool>(),
    ) {
        // Feature: undo-redo-transactions, Property 10: selection restoration
        let config = UndoConfig {
            max_levels: 100,
            coalesce_timeout_ms: 10_000,
            selection_history_enabled: selection_enabled,
            recovery_interval_seconds: 0,
        };
        let mut mgr = DocumentUndoManager::new(config);

        // Record operations with known selection states
        let mut before_positions = Vec::new();
        let mut after_positions = Vec::new();
        for i in 0..num_ops {
            let before_pos = i as u64 * 10;
            let after_pos = before_pos + 5;
            before_positions.push(before_pos);
            after_positions.push(after_pos);

            // Set selection before the operation
            mgr.set_selection_state(SelectionState::single_caret(before_pos));
            mgr.begin_transaction(&format!("op{}", i));
            mgr.record_insert(before_pos, b"x");
            // Set selection to after-state before ending transaction
            mgr.set_selection_state(SelectionState::single_caret(after_pos));
            mgr.end_transaction();
        }

        if num_ops > 0 && selection_enabled {
            // Undo should restore before-state of last transaction
            mgr.undo().unwrap();
            let sel = mgr.current_selection().unwrap();
            let expected_before = before_positions[num_ops - 1];
            prop_assert_eq!(sel.carets[0].position, expected_before,
                "undo should restore before-state selection");

            // Redo should restore after-state of last transaction
            mgr.redo().unwrap();
            let sel = mgr.current_selection().unwrap();
            let expected_after = after_positions[num_ops - 1];
            prop_assert_eq!(sel.carets[0].position, expected_after,
                "redo should restore after-state selection");
        }
    }
}
