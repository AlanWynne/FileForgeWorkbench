//! Tests for incremental updates (insert/delete lines).
//!
//! Validates: Requirement 6 AC 1–7

use ff_display_line_mapping::{ContractionState, DisplayLineMapping, DocLine};
use pretty_assertions::assert_eq;

#[test]
fn insert_lines_increases_doc_count() {
    // Validates: Requirement 6.1
    let mut state = ContractionState::new(10);
    state.insert_lines(DocLine(5), 3);
    assert_eq!(state.lines_in_doc(), 13);
}

#[test]
fn insert_lines_increases_display_count() {
    // Validates: Requirement 6.1
    let mut state = ContractionState::new(10);
    state.insert_lines(DocLine(5), 3);
    assert_eq!(state.lines_displayed(), 13);
}

#[test]
fn delete_lines_decreases_doc_count() {
    // Validates: Requirement 6.2
    let mut state = ContractionState::new(10);
    state.delete_lines(DocLine(3), 2);
    assert_eq!(state.lines_in_doc(), 8);
}

#[test]
fn delete_lines_decreases_display_count() {
    // Validates: Requirement 6.2
    let mut state = ContractionState::new(10);
    state.delete_lines(DocLine(3), 2);
    assert_eq!(state.lines_displayed(), 8);
}

#[test]
fn insert_lines_in_full_mode_preserves_prior_mapping() {
    // Validates: Requirement 6.1
    let mut state = ContractionState::new(10);
    state.set_height(DocLine(2), 3); // Transition to full mode

    let before_2 = state.display_from_doc(DocLine(2));

    state.insert_lines(DocLine(5), 3);

    // Line 2's display position should not change
    assert_eq!(state.display_from_doc(DocLine(2)), before_2);
    assert_eq!(state.lines_in_doc(), 13);
}

#[test]
fn delete_visible_lines_reduces_display_count() {
    // Validates: Requirement 6.2
    let mut state = ContractionState::new(10);
    state.set_height(DocLine(3), 4); // line 3 has height 4
                                     // Display count: 9 + 4 = 13
    assert_eq!(state.lines_displayed(), 13);
    // Delete line 3 (height 4)
    state.delete_lines(DocLine(3), 1);
    assert_eq!(state.lines_in_doc(), 9);
    assert_eq!(state.lines_displayed(), 9);
}

#[test]
fn delete_hidden_lines_does_not_change_display_count() {
    // Validates: Requirement 6.2
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(3), DocLine(5), false);
    assert_eq!(state.lines_displayed(), 7);
    state.delete_lines(DocLine(3), 3);
    assert_eq!(state.lines_in_doc(), 7);
    assert_eq!(state.lines_displayed(), 7); // same since hidden lines were removed
}

#[test]
fn insert_at_beginning_shifts_all_display_lines() {
    // Validates: Requirement 6.1
    let mut state = ContractionState::new(5);
    state.set_height(DocLine(0), 2); // Transition to full mode
    state.insert_lines(DocLine(0), 2);
    assert_eq!(state.lines_in_doc(), 7);
    // Original line 0 is now at index 2
    assert_eq!(state.get_height(DocLine(2)), 2);
}

#[test]
fn insert_at_end_appends_new_lines() {
    // Validates: Requirement 6.1
    let mut state = ContractionState::new(5);
    state.set_height(DocLine(4), 2); // Transition to full mode
    state.insert_lines(DocLine(5), 3);
    assert_eq!(state.lines_in_doc(), 8);
    // Original line 4 still has its height
    assert_eq!(state.get_height(DocLine(4)), 2);
    // New lines have height 1
    assert_eq!(state.get_height(DocLine(5)), 1);
    assert_eq!(state.get_height(DocLine(6)), 1);
    assert_eq!(state.get_height(DocLine(7)), 1);
}

#[test]
fn delete_beyond_end_clamps_to_available_lines() {
    // Validates: Requirement 6.2
    let mut state = ContractionState::new(5);
    state.delete_lines(DocLine(3), 10); // Only 2 lines available to delete
    assert_eq!(state.lines_in_doc(), 3);
}

#[test]
fn insert_zero_count_is_no_op() {
    let mut state = ContractionState::new(10);
    state.insert_lines(DocLine(5), 0);
    assert_eq!(state.lines_in_doc(), 10);
}

#[test]
fn delete_zero_count_is_no_op() {
    let mut state = ContractionState::new(10);
    state.delete_lines(DocLine(5), 0);
    assert_eq!(state.lines_in_doc(), 10);
}

#[test]
fn insert_preserves_fold_text_after_insertion_point() {
    let mut state = ContractionState::new(10);
    state.set_fold_display_text(DocLine(5), Some("folded"));
    state.insert_lines(DocLine(3), 2);
    // Line 5 became line 7
    assert_eq!(state.get_fold_display_text(DocLine(7)), Some("folded"));
    assert_eq!(state.get_fold_display_text(DocLine(5)), None);
}

#[test]
fn delete_removes_fold_text_in_deleted_range() {
    let mut state = ContractionState::new(10);
    state.set_fold_display_text(DocLine(5), Some("folded"));
    state.delete_lines(DocLine(4), 3); // Deletes lines 4, 5, 6
    assert_eq!(state.get_fold_display_text(DocLine(4)), None);
}

#[test]
fn display_count_invariant_after_insert_and_hide() {
    // Validates: Requirement 6.7
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(2), DocLine(4), false);
    state.set_height(DocLine(7), 3);
    state.insert_lines(DocLine(5), 2);
    // Verify invariant: lines_displayed == sum of effective heights
    let expected: usize = (0..state.lines_in_doc())
        .map(|i| {
            if state.get_visible(DocLine(i)) {
                state.get_height(DocLine(i)) as usize
            } else {
                0
            }
        })
        .sum();
    assert_eq!(state.lines_displayed(), expected);
}
