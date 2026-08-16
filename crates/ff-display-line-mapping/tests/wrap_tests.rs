//! Tests for word wrap height mapping.
//!
//! Validates: Requirement 4 AC 1–8

use ff_display_line_mapping::{
    ContractionState, DisplayLine, DisplayLineMapping, DocLine, SubLine,
};
use pretty_assertions::assert_eq;

#[test]
fn set_height_stores_wrap_height() {
    // Validates: Requirement 4.1, 4.2
    let mut state = ContractionState::new(10);
    assert!(state.set_height(DocLine(3), 4));
    assert_eq!(state.get_height(DocLine(3)), 4);
}

#[test]
fn set_height_adjusts_display_count() {
    // Validates: Requirement 4.5
    let mut state = ContractionState::new(10);
    assert_eq!(state.lines_displayed(), 10);
    state.set_height(DocLine(3), 4);
    // Added 3 extra display lines
    assert_eq!(state.lines_displayed(), 13);
}

#[test]
fn set_height_on_hidden_line_does_not_change_display_count() {
    // Validates: Requirement 4.6
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(3), DocLine(3), false);
    assert_eq!(state.lines_displayed(), 9);
    state.set_height(DocLine(3), 5);
    // Still 9, hidden line doesn't contribute
    assert_eq!(state.lines_displayed(), 9);
    assert_eq!(state.get_height(DocLine(3)), 5);
}

#[test]
fn set_height_invalid_line_returns_false() {
    // Validates: Requirement 4.7
    let mut state = ContractionState::new(10);
    assert!(!state.set_height(DocLine(10), 2));
    assert!(!state.set_height(DocLine(100), 2));
}

#[test]
fn set_height_zero_returns_false() {
    // Validates: height must be >= 1
    let mut state = ContractionState::new(10);
    assert!(!state.set_height(DocLine(3), 0));
}

#[test]
fn set_height_returns_false_when_no_change() {
    // Validates: Requirement 4.1
    let mut state = ContractionState::new(10);
    state.set_height(DocLine(3), 4);
    assert!(!state.set_height(DocLine(3), 4)); // Same value
}

#[test]
fn display_from_doc_sub_returns_correct_sub_lines() {
    // Validates: Requirement 4.8
    let mut state = ContractionState::new(5);
    state.set_height(DocLine(2), 3);
    // Line 2 starts at display line 2 (lines 0, 1 before it each have height 1)
    assert_eq!(
        state.display_from_doc_sub(DocLine(2), SubLine(0)),
        DisplayLine(2)
    );
    assert_eq!(
        state.display_from_doc_sub(DocLine(2), SubLine(1)),
        DisplayLine(3)
    );
    assert_eq!(
        state.display_from_doc_sub(DocLine(2), SubLine(2)),
        DisplayLine(4)
    );
}

#[test]
fn display_from_doc_sub_clamps_sub_line() {
    // Validates: Requirement 1.2 (clamping)
    let mut state = ContractionState::new(5);
    state.set_height(DocLine(2), 3);
    // Sub-line 10 is clamped to height-1 = 2
    assert_eq!(
        state.display_from_doc_sub(DocLine(2), SubLine(10)),
        DisplayLine(4)
    );
}

#[test]
fn display_last_from_doc_returns_last_sub_line() {
    // Validates: Requirement 1.3
    let mut state = ContractionState::new(5);
    state.set_height(DocLine(2), 3);
    // First display line of line 2 is 2, last is 4 (2+3-1)
    assert_eq!(state.display_last_from_doc(DocLine(2)), DisplayLine(4));
}

#[test]
fn doc_from_display_returns_correct_sub_line_for_wrapped_lines() {
    // Validates: Requirement 1.4
    let mut state = ContractionState::new(5);
    state.set_height(DocLine(1), 3);
    // Display lines: [doc0, doc1-sub0, doc1-sub1, doc1-sub2, doc2, doc3, doc4]
    let pos = state.doc_from_display(DisplayLine(1));
    assert_eq!(pos.doc_line, DocLine(1));
    assert_eq!(pos.sub_line, SubLine(0));

    let pos = state.doc_from_display(DisplayLine(2));
    assert_eq!(pos.doc_line, DocLine(1));
    assert_eq!(pos.sub_line, SubLine(1));

    let pos = state.doc_from_display(DisplayLine(3));
    assert_eq!(pos.doc_line, DocLine(1));
    assert_eq!(pos.sub_line, SubLine(2));

    let pos = state.doc_from_display(DisplayLine(4));
    assert_eq!(pos.doc_line, DocLine(2));
    assert_eq!(pos.sub_line, SubLine(0));
}

#[test]
fn sub_line_contiguity_for_wrapped_lines() {
    // Validates: Requirement 4.8
    let mut state = ContractionState::new(5);
    state.set_height(DocLine(2), 4);
    let base = state.display_from_doc_sub(DocLine(2), SubLine(0));
    for s in 1..4 {
        let next = state.display_from_doc_sub(DocLine(2), SubLine(s));
        assert_eq!(next.0, base.0 + s, "Sub-line {s} should be contiguous");
    }
}

#[test]
fn multiple_wrapped_lines_display_correctly() {
    // Validates: Requirement 4.5
    let mut state = ContractionState::new(5);
    state.set_height(DocLine(1), 2);
    state.set_height(DocLine(3), 3);
    // Display: [doc0(1), doc1(2), doc1, doc2(1), doc3(3), doc3, doc3, doc4(1)]
    assert_eq!(state.lines_displayed(), 8);
    assert_eq!(state.display_from_doc(DocLine(0)), DisplayLine(0));
    assert_eq!(state.display_from_doc(DocLine(1)), DisplayLine(1));
    assert_eq!(state.display_from_doc(DocLine(2)), DisplayLine(3));
    assert_eq!(state.display_from_doc(DocLine(3)), DisplayLine(4));
    assert_eq!(state.display_from_doc(DocLine(4)), DisplayLine(7));
}
