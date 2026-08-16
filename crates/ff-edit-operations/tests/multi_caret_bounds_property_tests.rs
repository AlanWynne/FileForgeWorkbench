//! Property-based tests for multi-caret and bounds invariants.
//!
//! Feature: ff-edit-operations, Multi-Caret and Bounds Properties
//! These tests validate invariants of multi-caret operations, bounds enforcement,
//! and rectangular selection conversions.

use ff_edit_operations::{
    BoundsEnforcer, DocumentModification, MultiCaretCoordinator, RectangularSelection,
    SelectionContainer, SelectionPosition, SelectionRange,
};
use proptest::prelude::*;

/// Strategy to generate a valid SelectionPosition within reasonable bounds.
fn arb_position() -> impl Strategy<Value = SelectionPosition> {
    (0u64..50, 0u64..100).prop_map(|(line, col)| SelectionPosition::new(line, col))
}

/// Strategy to generate valid bounds (left >= 1, right > left).
fn arb_valid_bounds() -> impl Strategy<Value = (u64, u64)> {
    (1u64..100, 2u64..100).prop_map(|(left, width)| (left, left + width))
}

proptest! {
    /// Property 26.1: multi-caret insert in reverse order produces the same result
    /// regardless of the number of carets (no position drift).
    ///
    /// **Validates: Requirement 8.4, 8.5**
    ///
    /// We validate that reverse_order_indices produces a correct reverse ordering
    /// such that processing from last to first avoids drift.
    #[test]
    fn multi_caret_reverse_order_produces_correct_ordering(
        positions in proptest::collection::vec(arb_position(), 2..10),
    ) {
        // Feature: ff-edit-operations, Property 26.1: reverse order no-drift
        let mut container = SelectionContainer::new();
        for pos in &positions {
            MultiCaretCoordinator::add_caret(&mut container, *pos);
        }
        container.trim(); // Merge any coincident positions

        let indices = MultiCaretCoordinator::reverse_order_indices(&container);
        let ranges = container.ranges();

        // Verify indices are in reverse document order
        for window in indices.windows(2) {
            let pos_a = ranges[window[0]].sort_key();
            let pos_b = ranges[window[1]].sort_key();
            prop_assert!(
                pos_a >= pos_b,
                "Reverse order violated: {:?} should be >= {:?}",
                pos_a, pos_b
            );
        }
    }

    /// Property 26.2: after multi-caret operation, Trim merges any coincident carets,
    /// and count never exceeds the pre-operation count.
    ///
    /// **Validates: Requirement 8.8, 8.13**
    #[test]
    fn trim_after_multi_caret_never_exceeds_pre_count(
        positions in proptest::collection::vec(arb_position(), 1..15),
        modification in (0u64..30, 0u64..50, 0u64..20, 0u64..20).prop_map(
            |(line, col, ins, del)| DocumentModification {
                line,
                column: col,
                inserted_length: ins,
                deleted_length: del,
                lines_inserted: 0,
                lines_deleted: 0,
            }
        ),
    ) {
        // Feature: ff-edit-operations, Property 26.2: trim count invariant
        let mut container = SelectionContainer::new();
        for pos in &positions {
            container.add(SelectionRange::collapsed(*pos));
        }
        let pre_trim_count = container.count();

        // Apply modification (may cause positions to converge)
        container.move_positions(&modification);
        // move_positions calls trim internally

        prop_assert!(
            container.count() <= pre_trim_count,
            "Count after trim ({}) should not exceed pre-operation count ({})",
            container.count(),
            pre_trim_count
        );
    }

    /// Property 26.3: BOUNDS enforcement — any character insertion with BOUNDS active
    /// never modifies columns outside [left, right] range for any line content and caret position.
    ///
    /// **Validates: Requirement 13.2, 13.3, 13.5**
    #[test]
    fn bounds_enforcement_rejects_edits_outside_range(
        (left, right) in arb_valid_bounds(),
        test_column in 0u64..200,
    ) {
        // Feature: ff-edit-operations, Property 26.3: bounds protection
        let mut enforcer = BoundsEnforcer::new();
        enforcer.set_bounds(left, right).unwrap();

        let allowed = enforcer.allows_edit_at(test_column);

        if test_column >= left && test_column <= right {
            prop_assert!(
                allowed,
                "Column {} should be allowed within bounds [{}, {}]",
                test_column, left, right
            );
        } else {
            prop_assert!(
                !allowed,
                "Column {} should be rejected outside bounds [{}, {}]",
                test_column, left, right
            );
        }
    }

    /// Property 26.4: rectangular selection to_selection_ranges() produces exactly
    /// (bottom_line - top_line + 1) ranges, each spanning [left_column, right_column].
    ///
    /// **Validates: Requirement 9.1, 9.2**
    #[test]
    fn rectangular_selection_produces_correct_range_count(
        top_line in 0u64..50,
        height in 1u64..20,
        left_column in 0u64..50,
        width in 1u64..50,
    ) {
        // Feature: ff-edit-operations, Property 26.4: rectangular range count
        let bottom_line = top_line + height - 1;
        let right_column = left_column + width - 1;

        let rect = RectangularSelection {
            top_line,
            bottom_line,
            left_column,
            right_column,
        };

        let ranges = rect.to_selection_ranges();
        let expected_count = (bottom_line - top_line + 1) as usize;

        prop_assert_eq!(
            ranges.len(),
            expected_count,
            "Should produce exactly {} ranges, got {}",
            expected_count,
            ranges.len()
        );

        // Each range should span the correct columns on its line
        for (i, range) in ranges.iter().enumerate() {
            let expected_line = top_line + i as u64;
            prop_assert_eq!(range.anchor.line, expected_line);
            prop_assert_eq!(range.anchor.column, left_column);
            prop_assert_eq!(range.caret.line, expected_line);
            prop_assert_eq!(range.caret.column, right_column);
        }
    }
}
