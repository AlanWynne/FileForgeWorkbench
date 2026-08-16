//! Tests for one-to-one mode behaviour.
//!
//! Validates: Requirement 1 AC 9, Requirement 9 AC 1, AC 4, AC 5, AC 6

use ff_display_line_mapping::{
    ContractionState, DisplayLine, DisplayLineMapping, DocLine, SubLine,
};

#[test]
fn new_state_starts_in_one_to_one_mode() {
    // Validates: Requirement 9.1
    let state = ContractionState::new(100);
    assert!(state.is_one_to_one());
}

#[test]
fn one_to_one_display_from_doc_returns_identity() {
    // Validates: Requirement 1.9, Requirement 9.5
    let state = ContractionState::new(50);
    for i in 0..50 {
        assert_eq!(state.display_from_doc(DocLine(i)), DisplayLine(i));
    }
}

#[test]
fn one_to_one_doc_from_display_returns_identity() {
    // Validates: Requirement 1.9, Requirement 9.5
    let state = ContractionState::new(50);
    for i in 0..50 {
        let pos = state.doc_from_display(DisplayLine(i));
        assert_eq!(pos.doc_line, DocLine(i));
        assert_eq!(pos.sub_line, SubLine(0));
    }
}

#[test]
fn one_to_one_lines_in_doc_returns_line_count() {
    // Validates: Requirement 1.7
    let state = ContractionState::new(42);
    assert_eq!(state.lines_in_doc(), 42);
}

#[test]
fn one_to_one_lines_displayed_equals_lines_in_doc() {
    // Validates: Requirement 1.8
    let state = ContractionState::new(42);
    assert_eq!(state.lines_displayed(), 42);
}

#[test]
fn one_to_one_get_visible_always_true() {
    // Validates: Requirement 9.5
    let state = ContractionState::new(10);
    for i in 0..10 {
        assert!(state.get_visible(DocLine(i)));
    }
}

#[test]
fn one_to_one_get_expanded_always_true() {
    // Validates: Requirement 9.5
    let state = ContractionState::new(10);
    for i in 0..10 {
        assert!(state.get_expanded(DocLine(i)));
    }
}

#[test]
fn one_to_one_get_height_always_one() {
    // Validates: Requirement 9.5
    let state = ContractionState::new(10);
    for i in 0..10 {
        assert_eq!(state.get_height(DocLine(i)), 1);
    }
}

#[test]
fn one_to_one_insert_lines_updates_count_without_allocation() {
    // Validates: Requirement 9.6
    let mut state = ContractionState::new(10);
    state.insert_lines(DocLine(5), 3);
    assert!(state.is_one_to_one());
    assert_eq!(state.lines_in_doc(), 13);
    assert_eq!(state.lines_displayed(), 13);
}

#[test]
fn one_to_one_delete_lines_updates_count_without_allocation() {
    // Validates: Requirement 9.6
    let mut state = ContractionState::new(10);
    state.delete_lines(DocLine(3), 2);
    assert!(state.is_one_to_one());
    assert_eq!(state.lines_in_doc(), 8);
    assert_eq!(state.lines_displayed(), 8);
}

#[test]
fn one_to_one_hidden_lines_returns_false() {
    // Validates: Requirement 2.5
    let state = ContractionState::new(10);
    assert!(!state.hidden_lines());
}

#[test]
fn set_visible_false_transitions_away_from_one_to_one() {
    // Validates: Requirement 9.2
    let mut state = ContractionState::new(10);
    assert!(state.is_one_to_one());
    state.set_visible(DocLine(3), DocLine(5), false);
    assert!(!state.is_one_to_one());
}

#[test]
fn set_height_greater_than_one_transitions_away_from_one_to_one() {
    // Validates: Requirement 9.2
    let mut state = ContractionState::new(10);
    assert!(state.is_one_to_one());
    state.set_height(DocLine(2), 3);
    assert!(!state.is_one_to_one());
}

#[test]
fn set_expanded_false_transitions_away_from_one_to_one() {
    // Validates: Requirement 9.2
    let mut state = ContractionState::new(10);
    assert!(state.is_one_to_one());
    state.set_expanded(DocLine(2), false);
    assert!(!state.is_one_to_one());
}

#[test]
fn display_from_doc_sub_in_one_to_one_returns_identity() {
    // Validates: Requirement 1.2
    let state = ContractionState::new(10);
    assert_eq!(
        state.display_from_doc_sub(DocLine(5), SubLine(0)),
        DisplayLine(5)
    );
}

#[test]
fn display_last_from_doc_in_one_to_one_returns_identity() {
    // Validates: Requirement 1.3
    let state = ContractionState::new(10);
    assert_eq!(state.display_last_from_doc(DocLine(5)), DisplayLine(5));
}

#[test]
fn doc_from_display_clamps_out_of_range_high() {
    // Validates: Requirement 1.6
    let state = ContractionState::new(10);
    let pos = state.doc_from_display(DisplayLine(100));
    assert_eq!(pos.doc_line, DocLine(9));
}

#[test]
fn zero_line_document_handles_safely() {
    let state = ContractionState::new(0);
    assert_eq!(state.lines_in_doc(), 0);
    assert_eq!(state.lines_displayed(), 0);
    let pos = state.doc_from_display(DisplayLine(0));
    assert_eq!(pos.doc_line, DocLine(0));
}
