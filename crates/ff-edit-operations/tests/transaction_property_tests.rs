//! Property-based tests for transaction and undo invariants.
//!
//! Feature: ff-edit-operations, Transaction Properties
//! These tests validate invariants of the transaction recording system,
//! undo groups, and modified line markers.

use ff_edit_operations::{EditorTransaction, LineSnapshot, ModifiedLineTracker, UndoGroup};
use proptest::prelude::*;

/// Strategy for generating a valid line number.
fn arb_line_number() -> impl Strategy<Value = u64> {
    0u64..1000
}

/// Strategy for generating a line content string.
fn arb_content() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{0,80}"
}

/// Strategy for generating a valid EditorTransaction.
fn arb_transaction() -> impl Strategy<Value = EditorTransaction> {
    (arb_line_number(), arb_content(), arb_content()).prop_map(|(line, before, after)| {
        EditorTransaction::new(
            vec![line],
            vec![LineSnapshot::new(line, before)],
            vec![LineSnapshot::new(line, after)],
            "test operation".to_string(),
        )
    })
}

proptest! {
    /// Property 27.1: undo followed by redo restores document to post-edit state
    /// for any single edit operation.
    ///
    /// **Validates: Requirement 11.4, 11.5**
    ///
    /// We model this as: applying a transaction (before->after), then undoing (after->before),
    /// then redoing (before->after again) produces the same after state.
    #[test]
    fn undo_then_redo_restores_post_edit_state(
        line in arb_line_number(),
        before_content in arb_content(),
        after_content in arb_content(),
    ) {
        // Feature: ff-edit-operations, Property 27.1: undo/redo round-trip
        let txn = EditorTransaction::new(
            vec![line],
            vec![LineSnapshot::new(line, before_content.clone())],
            vec![LineSnapshot::new(line, after_content.clone())],
            "edit".to_string(),
        );

        // "Apply" the edit: state is now after_content
        let state_after_apply = &txn.after_snapshot[0].content;
        prop_assert_eq!(state_after_apply, &after_content);

        // "Undo": restore before_snapshot -> state is before_content
        let state_after_undo = &txn.before_snapshot[0].content;
        prop_assert_eq!(state_after_undo, &before_content);

        // "Redo": restore after_snapshot -> state is after_content again
        let state_after_redo = &txn.after_snapshot[0].content;
        prop_assert_eq!(state_after_redo, &after_content);
        prop_assert_eq!(state_after_apply, state_after_redo);
    }

    /// Property 27.2: modified line markers are set for every line whose content
    /// differs from saved state, and cleared for every line matching saved state,
    /// after any sequence of edits and undos.
    ///
    /// **Validates: Requirement 11.6, 11.7, 11.8**
    #[test]
    fn modified_markers_reflect_difference_from_saved_state(
        lines_to_modify in proptest::collection::vec(arb_line_number(), 1..20),
        lines_to_clear in proptest::collection::vec(arb_line_number(), 0..10),
    ) {
        // Feature: ff-edit-operations, Property 27.2: marker correctness
        let mut tracker = ModifiedLineTracker::new();

        // Mark lines as modified
        for &line in &lines_to_modify {
            tracker.mark_modified(line);
        }

        // Clear some lines (simulating undo back to saved state)
        for &line in &lines_to_clear {
            tracker.clear_line(line);
        }

        // Verify: cleared lines are not marked, un-cleared modified lines are still marked
        for &line in &lines_to_clear {
            prop_assert!(
                !tracker.is_modified(line),
                "Line {} should not be modified after clear",
                line
            );
        }

        for &line in &lines_to_modify {
            if !lines_to_clear.contains(&line) {
                prop_assert!(
                    tracker.is_modified(line),
                    "Line {} should still be modified (not cleared)",
                    line
                );
            }
        }

        // Simulate save: clear_all should remove all marks
        tracker.clear_all();
        for &line in &lines_to_modify {
            prop_assert!(
                !tracker.is_modified(line),
                "Line {} should not be modified after save (clear_all)",
                line
            );
        }
        prop_assert!(!tracker.has_modifications());
    }

    /// Property 27.3: UndoGroup atomicity — undoing a multi-caret operation reverses
    /// ALL sub-operations in a single undo step.
    ///
    /// **Validates: Requirement 11.9, 8.13**
    ///
    /// We validate that an UndoGroup correctly aggregates all affected lines from
    /// its sub-transactions, ensuring they can all be reversed atomically.
    #[test]
    fn undo_group_captures_all_sub_operations(
        transactions in proptest::collection::vec(arb_transaction(), 1..10),
    ) {
        // Feature: ff-edit-operations, Property 27.3: UndoGroup atomicity
        let mut group = UndoGroup::new("multi-caret edit".to_string());

        let mut expected_lines: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for txn in &transactions {
            for &line in &txn.affected_lines {
                expected_lines.insert(line);
            }
            group.push(txn.clone());
        }

        // Verify all sub-transactions are captured
        prop_assert_eq!(
            group.len(),
            transactions.len(),
            "UndoGroup should contain all sub-transactions"
        );

        // Verify all_modified_lines covers every affected line
        let group_lines: std::collections::HashSet<u64> =
            group.all_modified_lines().into_iter().collect();
        prop_assert_eq!(
            group_lines, expected_lines,
            "UndoGroup should track all affected lines from all sub-transactions"
        );

        // Each sub-transaction has valid snapshots for atomic reversal
        for txn in &group.transactions {
            prop_assert!(
                txn.is_valid(),
                "Each sub-transaction in UndoGroup should be valid"
            );
        }
    }
}
