//! Property-based tests for selection model invariants.
//!
//! Feature: ff-edit-operations, Selection Model Properties
//! These tests validate invariants of the SelectionContainer, position adjustment,
//! and trim operations using randomly generated inputs.

use ff_edit_operations::{
    DocumentModification, SelectionContainer, SelectionPosition, SelectionRange,
};
use proptest::prelude::*;

/// Strategy to generate a valid SelectionPosition.
fn arb_position() -> impl Strategy<Value = SelectionPosition> {
    (0u64..100, 0u64..200, 0u64..20)
        .prop_map(|(line, col, vs)| SelectionPosition::with_virtual_space(line, col, vs))
}

/// Strategy to generate a SelectionRange (may or may not be collapsed).
fn arb_range() -> impl Strategy<Value = SelectionRange> {
    (arb_position(), arb_position()).prop_map(|(anchor, caret)| SelectionRange::new(anchor, caret))
}

/// Strategy to generate a collapsed SelectionRange (single caret).
fn arb_collapsed_range() -> impl Strategy<Value = SelectionRange> {
    arb_position().prop_map(SelectionRange::collapsed)
}

/// Strategy to generate a DocumentModification.
fn arb_modification() -> impl Strategy<Value = DocumentModification> {
    (0u64..50, 0u64..100, 0u64..50, 0u64..50, 0u64..5, 0u64..5).prop_map(
        |(line, column, inserted, deleted, lines_ins, lines_del)| DocumentModification {
            line,
            column,
            inserted_length: inserted,
            deleted_length: deleted,
            lines_inserted: lines_ins,
            lines_deleted: lines_del,
        },
    )
}

proptest! {
    /// Property 24.1: SelectionContainer always maintains ranges in sorted
    /// document order after any Add/Drop/Trim sequence.
    ///
    /// **Validates: Requirement 14.1, 14.3**
    #[test]
    fn selection_container_maintains_sorted_order_after_operations(
        ranges in proptest::collection::vec(arb_collapsed_range(), 1..20),
        drop_indices in proptest::collection::vec(0usize..10, 0..5),
    ) {
        // Feature: ff-edit-operations, Property 24.1: sorted order invariant
        let mut container = SelectionContainer::new();

        // Add all ranges
        for range in &ranges {
            container.add(range.clone());
        }

        // Try dropping some (may fail gracefully if index out of range or last)
        for &idx in &drop_indices {
            let _ = container.drop_range(idx % container.count().max(1));
        }

        // Trim to merge overlaps
        container.trim();

        // Verify sorted order
        let result_ranges = container.ranges();
        for window in result_ranges.windows(2) {
            prop_assert!(
                window[0].sort_key() <= window[1].sort_key(),
                "Ranges not in sorted order: {:?} > {:?}",
                window[0].sort_key(),
                window[1].sort_key()
            );
        }
    }

    /// Property 24.2: MovePositions never produces negative positions
    /// (all positions remain >= 0) for arbitrary DocumentModification inputs.
    ///
    /// **Validates: Requirement 7.1–7.4, 14.4**
    #[test]
    fn move_positions_never_produces_negative_positions(
        initial_ranges in proptest::collection::vec(arb_collapsed_range(), 1..10),
        modification in arb_modification(),
    ) {
        // Feature: ff-edit-operations, Property 24.2: no negative positions
        let mut container = SelectionContainer::new();
        for range in &initial_ranges {
            container.add(range.clone());
        }

        container.move_positions(&modification);

        for range in container.ranges() {
            // All position fields are u64, so they can't be negative by type,
            // but we verify they're reasonable (not overflow-wrapped)
            prop_assert!(range.anchor.line < u64::MAX / 2,
                "Anchor line looks like overflow: {}", range.anchor.line);
            prop_assert!(range.anchor.column < u64::MAX / 2,
                "Anchor column looks like overflow: {}", range.anchor.column);
            prop_assert!(range.caret.line < u64::MAX / 2,
                "Caret line looks like overflow: {}", range.caret.line);
            prop_assert!(range.caret.column < u64::MAX / 2,
                "Caret column looks like overflow: {}", range.caret.column);
        }
    }

    /// Property 24.3: Trim operation is idempotent (trim(trim(container)) == trim(container))
    /// and eliminates all overlaps.
    ///
    /// **Validates: Requirement 14.3, 7.7**
    #[test]
    fn trim_is_idempotent_and_eliminates_overlaps(
        ranges in proptest::collection::vec(arb_range(), 1..15),
    ) {
        // Feature: ff-edit-operations, Property 24.3: trim idempotency
        let mut container = SelectionContainer::new();
        for range in &ranges {
            container.add(range.clone());
        }

        // First trim
        container.trim();
        let after_first_trim: Vec<SelectionRange> = container.ranges().to_vec();

        // Second trim (should be identical)
        container.trim();
        let after_second_trim: Vec<SelectionRange> = container.ranges().to_vec();

        prop_assert_eq!(
            &after_first_trim, &after_second_trim,
            "Trim is not idempotent"
        );

        // Verify no overlaps exist after trim
        let trimmed = container.ranges();
        for i in 0..trimmed.len() {
            for j in (i + 1)..trimmed.len() {
                prop_assert!(
                    !trimmed[i].overlaps(&trimmed[j]),
                    "Overlapping ranges after trim: {:?} and {:?}",
                    trimmed[i],
                    trimmed[j]
                );
            }
        }
    }

    /// Property 24.4: SelectionContainer always has count() >= 1 after any
    /// sequence of Add/Drop operations.
    ///
    /// **Validates: Requirement 14.2, 14.8**
    #[test]
    fn selection_container_count_always_at_least_one(
        adds in proptest::collection::vec(arb_collapsed_range(), 0..20),
        drops in proptest::collection::vec(0usize..20, 0..30),
    ) {
        // Feature: ff-edit-operations, Property 24.4: count >= 1 invariant
        let mut container = SelectionContainer::new();

        for range in &adds {
            container.add(range.clone());
        }

        for &idx in &drops {
            let _ = container.drop_range(idx % container.count().max(1));
        }

        prop_assert!(
            container.count() >= 1,
            "Container has zero ranges after operations"
        );
    }
}
