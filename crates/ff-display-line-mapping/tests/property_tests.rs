//! Property-based tests for display-line mapping invariants.
//!
//! Uses the `proptest` crate with a minimum of 100 cases per property.
//! These tests validate universal invariants that must hold across all
//! valid inputs.

use proptest::prelude::*;

use ff_display_line_mapping::{
    ContractionState, DisplayLine, DisplayLineMapping, DocLine, SubLine,
};

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Generate a line count between 1 and 200 for reasonable test sizes.
fn line_count_strategy() -> impl Strategy<Value = usize> {
    1usize..200
}

/// Generate a ContractionState with random visibility and heights.
fn contraction_state_strategy() -> impl Strategy<Value = ContractionState> {
    line_count_strategy()
        .prop_flat_map(|n| {
            let visibility = proptest::collection::vec(prop::bool::ANY, n);
            let heights = proptest::collection::vec(1u32..5, n);
            (Just(n), visibility, heights)
        })
        .prop_map(|(n, visibility, heights)| {
            let mut state = ContractionState::new(n);
            // Apply visibility and heights
            for i in 0..n {
                if heights[i] > 1 {
                    state.set_height(DocLine(i), heights[i]);
                }
                if !visibility[i] {
                    state.set_visible(DocLine(i), DocLine(i), false);
                }
            }
            state
        })
}

/// Generate a ContractionState with at least one visible line.
fn state_with_visible_lines() -> impl Strategy<Value = ContractionState> {
    contraction_state_strategy().prop_filter("need at least one visible line", |s| {
        s.lines_displayed() > 0
    })
}

// ─── Property Tests ─────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property 1: Display line count invariant.
    /// lines_displayed() always equals the sum of effective heights of visible lines.
    ///
    /// **Validates: Requirement 6.7**
    #[test]
    fn display_count_invariant(state in contraction_state_strategy()) {
        // Feature: display-line-mapping, Property 1: Display Line Count Invariant
        let expected: usize = (0..state.lines_in_doc())
            .map(|i| {
                if state.get_visible(DocLine(i)) {
                    state.get_height(DocLine(i)) as usize
                } else {
                    0
                }
            })
            .sum();
        prop_assert_eq!(state.lines_displayed(), expected);
    }

    /// Property 2: Doc-to-display round-trip.
    /// For any visible document line d, doc_from_display(display_from_doc(d)) == d.
    ///
    /// **Validates: Requirement 1.10**
    #[test]
    fn roundtrip_invariant(state in state_with_visible_lines()) {
        // Feature: display-line-mapping, Property 2: Doc-to-Display Round-Trip
        for d in 0..state.lines_in_doc() {
            if state.get_visible(DocLine(d)) {
                let display = state.display_from_doc(DocLine(d));
                let back = state.doc_from_display(display);
                prop_assert_eq!(back.doc_line, DocLine(d));
            }
        }
    }

    /// Property 3: Hidden lines contribute zero display lines.
    /// Hiding a visible line decreases display count by exactly its height.
    ///
    /// **Validates: Requirement 2.8**
    #[test]
    fn hidden_lines_contribute_zero(
        n in 5usize..50,
        hide_idx in 0usize..5
    ) {
        // Feature: display-line-mapping, Property 3: Hidden Lines Contribute Zero
        let mut state = ContractionState::new(n);
        let idx = hide_idx % n;
        let h = (idx as u32 % 3) + 1;
        if h > 1 {
            state.set_height(DocLine(idx), h);
        }
        let old_count = state.lines_displayed();
        let height = state.get_height(DocLine(idx));
        state.set_visible(DocLine(idx), DocLine(idx), false);
        prop_assert_eq!(state.lines_displayed(), old_count - height as usize);
    }

    /// Property 4: Insert/delete line count consistency.
    /// After insert_lines(pos, n), lines_in_doc() increases by n.
    /// After delete_lines(pos, n), lines_in_doc() decreases by n.
    ///
    /// **Validates: Requirements 6.1, 6.2**
    #[test]
    fn insert_delete_count_consistency(
        n in 5usize..50,
        pos in 0usize..50,
        count in 1usize..10
    ) {
        // Feature: display-line-mapping, Property 4: Insert/Delete Count Consistency
        let pos = pos % (n + 1); // Valid insertion point
        let mut state = ContractionState::new(n);

        // Test insert
        state.insert_lines(DocLine(pos), count);
        prop_assert_eq!(state.lines_in_doc(), n + count);

        // Test delete (delete what we just inserted)
        state.delete_lines(DocLine(pos), count);
        prop_assert_eq!(state.lines_in_doc(), n);
    }

    /// Property 5: set_height on visible line adjusts display count by (new - old).
    ///
    /// **Validates: Requirement 4.5**
    #[test]
    fn height_change_adjusts_display_count(
        n in 5usize..50,
        line_idx in 0usize..50,
        new_height in 1u32..6
    ) {
        // Feature: display-line-mapping, Property 5: Height Change Adjusts Display Count
        let idx = line_idx % n;
        let mut state = ContractionState::new(n);
        // Ensure line is visible
        let old_count = state.lines_displayed();
        let old_h = state.get_height(DocLine(idx));
        state.set_height(DocLine(idx), new_height);
        let new_count = state.lines_displayed();
        let expected_diff = new_height as i64 - old_h as i64;
        prop_assert_eq!(new_count as i64 - old_count as i64, expected_diff);
    }

    /// Property 6: One-to-one mode identity.
    /// When no lines are hidden and all heights are 1, display_from_doc(n) == n
    /// and doc_from_display(n) == n.
    ///
    /// **Validates: Requirement 1.9**
    #[test]
    fn one_to_one_identity(n in 1usize..200) {
        // Feature: display-line-mapping, Property 6: One-to-One Mode Identity
        let state = ContractionState::new(n);
        prop_assert!(state.is_one_to_one());
        for i in 0..n {
            prop_assert_eq!(state.display_from_doc(DocLine(i)), DisplayLine(i));
            let pos = state.doc_from_display(DisplayLine(i));
            prop_assert_eq!(pos.doc_line, DocLine(i));
            prop_assert_eq!(pos.sub_line, SubLine(0));
        }
    }

    /// Property 7: Sub-line contiguity.
    /// For a visible line with height h > 1, display_from_doc_sub(d, 0..h-1)
    /// returns h contiguous values.
    ///
    /// **Validates: Requirement 4.8**
    #[test]
    fn sub_line_contiguity(
        n in 5usize..50,
        line_idx in 0usize..50,
        height in 2u32..6
    ) {
        // Feature: display-line-mapping, Property 7: Sub-Line Contiguity
        let idx = line_idx % n;
        let mut state = ContractionState::new(n);
        state.set_height(DocLine(idx), height);
        let base = state.display_from_doc_sub(DocLine(idx), SubLine(0));
        for s in 1..height as usize {
            let next = state.display_from_doc_sub(DocLine(idx), SubLine(s));
            prop_assert_eq!(next.0, base.0 + s);
        }
    }

    /// Property 8: show_all restores one-to-one mode.
    /// After arbitrary operations, show_all() returns to identity mapping.
    ///
    /// **Validates: Requirement 2.6**
    #[test]
    fn show_all_restores_one_to_one(mut state in contraction_state_strategy()) {
        // Feature: display-line-mapping, Property 8: Show All Restores One-to-One
        state.show_all();
        prop_assert!(state.is_one_to_one());
        prop_assert_eq!(state.lines_displayed(), state.lines_in_doc());
        prop_assert!(!state.hidden_lines());
        for i in 0..state.lines_in_doc() {
            prop_assert!(state.get_visible(DocLine(i)));
            prop_assert_eq!(state.display_from_doc(DocLine(i)), DisplayLine(i));
        }
    }
}
