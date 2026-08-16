//! Tests for line exclusion and hiding.
//!
//! Validates: Requirement 2 AC 1–8

use ff_display_line_mapping::{ContractionState, DisplayLine, DisplayLineMapping, DocLine};
use pretty_assertions::assert_eq;

#[test]
fn hide_single_line_reduces_display_count_by_one() {
    // Validates: Requirement 2.3
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(5), DocLine(5), false);
    assert_eq!(state.lines_displayed(), 9);
}

#[test]
fn hide_range_reduces_display_count_by_range_size() {
    // Validates: Requirement 2.3
    let mut state = ContractionState::new(20);
    state.set_visible(DocLine(5), DocLine(9), false);
    assert_eq!(state.lines_displayed(), 15);
}

#[test]
fn get_visible_returns_false_for_hidden_lines() {
    // Validates: Requirement 2.2
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(3), DocLine(5), false);
    assert!(!state.get_visible(DocLine(3)));
    assert!(!state.get_visible(DocLine(4)));
    assert!(!state.get_visible(DocLine(5)));
    assert!(state.get_visible(DocLine(2)));
    assert!(state.get_visible(DocLine(6)));
}

#[test]
fn hidden_lines_returns_true_when_lines_are_hidden() {
    // Validates: Requirement 2.5
    let mut state = ContractionState::new(10);
    assert!(!state.hidden_lines());
    state.set_visible(DocLine(3), DocLine(3), false);
    assert!(state.hidden_lines());
}

#[test]
fn show_hidden_line_increases_display_count() {
    // Validates: Requirement 2.4
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(3), DocLine(5), false);
    assert_eq!(state.lines_displayed(), 7);
    state.set_visible(DocLine(3), DocLine(5), true);
    assert_eq!(state.lines_displayed(), 10);
}

#[test]
fn show_all_makes_all_lines_visible_and_returns_to_one_to_one() {
    // Validates: Requirement 2.6
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(2), DocLine(7), false);
    assert_eq!(state.lines_displayed(), 4);
    state.show_all();
    assert!(state.is_one_to_one());
    assert_eq!(state.lines_displayed(), 10);
    assert!(!state.hidden_lines());
}

#[test]
fn set_visible_invalid_range_returns_false() {
    // Validates: Requirement 2.7
    let mut state = ContractionState::new(10);
    // start > end
    assert!(!state.set_visible(DocLine(5), DocLine(3), false));
    // end out of range
    assert!(!state.set_visible(DocLine(0), DocLine(10), false));
    // Both out of range
    assert!(!state.set_visible(DocLine(11), DocLine(12), false));
}

#[test]
fn set_visible_returns_false_when_no_change() {
    // Validates: Requirement 2.1 (returns true only if changed)
    let mut state = ContractionState::new(10);
    // Showing already visible lines
    assert!(!state.set_visible(DocLine(0), DocLine(5), true));
}

#[test]
fn display_from_doc_skips_hidden_lines() {
    // Validates: Requirement 1.1, Requirement 2.3
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(2), DocLine(4), false);
    // Lines 0, 1 are at display 0, 1
    assert_eq!(state.display_from_doc(DocLine(0)), DisplayLine(0));
    assert_eq!(state.display_from_doc(DocLine(1)), DisplayLine(1));
    // Lines 2-4 hidden, display offset is still 2 (they contribute 0)
    // Line 5 should be at display 2
    assert_eq!(state.display_from_doc(DocLine(5)), DisplayLine(2));
    assert_eq!(state.display_from_doc(DocLine(6)), DisplayLine(3));
}

#[test]
fn doc_from_display_skips_hidden_lines() {
    // Validates: Requirement 1.4
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(2), DocLine(4), false);
    // Display 0 = doc 0
    let pos = state.doc_from_display(DisplayLine(0));
    assert_eq!(pos.doc_line, DocLine(0));
    // Display 1 = doc 1
    let pos = state.doc_from_display(DisplayLine(1));
    assert_eq!(pos.doc_line, DocLine(1));
    // Display 2 = doc 5 (lines 2-4 are hidden)
    let pos = state.doc_from_display(DisplayLine(2));
    assert_eq!(pos.doc_line, DocLine(5));
}

#[test]
fn hiding_line_with_wrap_height_reduces_display_by_height() {
    // Validates: Requirement 2.3, Requirement 4.6
    let mut state = ContractionState::new(10);
    state.set_height(DocLine(3), 4);
    assert_eq!(state.lines_displayed(), 13); // 9 + 4
    state.set_visible(DocLine(3), DocLine(3), false);
    assert_eq!(state.lines_displayed(), 9); // 9 lines visible, each height 1
}

#[test]
fn showing_line_with_stored_height_restores_full_height() {
    // Validates: Requirement 2.4
    let mut state = ContractionState::new(10);
    state.set_height(DocLine(3), 4);
    state.set_visible(DocLine(3), DocLine(3), false);
    assert_eq!(state.lines_displayed(), 9);
    state.set_visible(DocLine(3), DocLine(3), true);
    assert_eq!(state.lines_displayed(), 13);
}

#[test]
fn display_line_count_invariant_after_mixed_operations() {
    // Validates: Requirement 2.8
    let mut state = ContractionState::new(10);
    state.set_height(DocLine(0), 2);
    state.set_height(DocLine(5), 3);
    state.set_visible(DocLine(3), DocLine(4), false);
    // Heights: [2, 1, 1, 1, 1, 3, 1, 1, 1, 1]
    // Visible: [T, T, T, F, F, T, T, T, T, T]
    // Expected: 2 + 1 + 1 + 0 + 0 + 3 + 1 + 1 + 1 + 1 = 11
    assert_eq!(state.lines_displayed(), 11);
}
