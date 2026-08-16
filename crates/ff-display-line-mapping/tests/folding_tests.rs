//! Tests for code folding state management.
//!
//! Validates: Requirement 3 AC 1–8, Requirement 10 AC 1–7

use ff_display_line_mapping::{ContractionState, DisplayLineMapping, DocLine};

#[test]
fn set_expanded_stores_fold_state() {
    // Validates: Requirement 3.1
    let mut state = ContractionState::new(10);
    assert!(state.set_expanded(DocLine(3), false));
    assert!(!state.get_expanded(DocLine(3)));
}

#[test]
fn get_expanded_returns_true_by_default() {
    // Validates: Requirement 3.2
    let state = ContractionState::new(10);
    for i in 0..10 {
        assert!(state.get_expanded(DocLine(i)));
    }
}

#[test]
fn set_expanded_returns_false_when_no_change() {
    // Validates: Requirement 3.1
    let mut state = ContractionState::new(10);
    state.set_expanded(DocLine(3), false);
    assert!(!state.set_expanded(DocLine(3), false)); // Already collapsed
}

#[test]
fn expand_all_sets_all_to_expanded() {
    // Validates: Requirement 3.3
    let mut state = ContractionState::new(10);
    state.set_expanded(DocLine(2), false);
    state.set_expanded(DocLine(5), false);
    state.set_expanded(DocLine(8), false);
    assert!(state.expand_all());
    assert!(state.get_expanded(DocLine(2)));
    assert!(state.get_expanded(DocLine(5)));
    assert!(state.get_expanded(DocLine(8)));
}

#[test]
fn expand_all_returns_false_when_all_already_expanded() {
    // Validates: Requirement 3.3
    let mut state = ContractionState::new(10);
    // Force away from one-to-one so expand_all can check
    state.set_visible(DocLine(0), DocLine(0), false);
    state.set_visible(DocLine(0), DocLine(0), true);
    assert!(!state.expand_all());
}

#[test]
fn contracted_next_finds_next_collapsed_fold() {
    // Validates: Requirement 3.4
    let mut state = ContractionState::new(20);
    state.set_expanded(DocLine(5), false);
    state.set_expanded(DocLine(12), false);
    assert_eq!(state.contracted_next(DocLine(0)), Some(DocLine(5)));
    assert_eq!(state.contracted_next(DocLine(5)), Some(DocLine(5)));
    assert_eq!(state.contracted_next(DocLine(6)), Some(DocLine(12)));
    assert_eq!(state.contracted_next(DocLine(13)), None);
}

#[test]
fn contracted_next_returns_none_when_no_folds() {
    // Validates: Requirement 3.4
    let state = ContractionState::new(10);
    assert_eq!(state.contracted_next(DocLine(0)), None);
}

#[test]
fn set_fold_display_text_stores_text() {
    // Validates: Requirement 3.7, 3.8
    let mut state = ContractionState::new(10);
    assert!(state.set_fold_display_text(DocLine(3), Some("{ ... }")));
    assert_eq!(state.get_fold_display_text(DocLine(3)), Some("{ ... }"));
}

#[test]
fn set_fold_display_text_none_clears_text() {
    // Validates: Requirement 3.7
    let mut state = ContractionState::new(10);
    state.set_fold_display_text(DocLine(3), Some("collapsed"));
    assert!(state.set_fold_display_text(DocLine(3), None));
    assert_eq!(state.get_fold_display_text(DocLine(3)), None);
}

#[test]
fn get_fold_display_text_returns_none_by_default() {
    // Validates: Requirement 3.8
    let state = ContractionState::new(10);
    assert_eq!(state.get_fold_display_text(DocLine(5)), None);
}

#[test]
fn set_fold_display_text_returns_false_when_no_change() {
    // Validates: Requirement 3.7
    let mut state = ContractionState::new(10);
    state.set_fold_display_text(DocLine(3), Some("text"));
    assert!(!state.set_fold_display_text(DocLine(3), Some("text")));
}

#[test]
fn fold_state_is_orthogonal_to_visibility() {
    // Validates: Requirement 10.1, 10.3
    let mut state = ContractionState::new(10);
    // Collapse a fold header
    state.set_expanded(DocLine(3), false);
    // The line is still visible (fold state doesn't affect visibility directly)
    assert!(state.get_visible(DocLine(3)));
    // Hide the line via exclusion
    state.set_visible(DocLine(3), DocLine(3), false);
    // Fold state is still collapsed
    assert!(!state.get_expanded(DocLine(3)));
    // Line is hidden
    assert!(!state.get_visible(DocLine(3)));
}

#[test]
fn show_all_resets_both_visibility_and_fold_state() {
    // Validates: Requirement 10.6
    let mut state = ContractionState::new(10);
    state.set_visible(DocLine(2), DocLine(4), false);
    state.set_expanded(DocLine(5), false);
    state.set_fold_display_text(DocLine(5), Some("collapsed"));
    state.show_all();
    // All visible
    for i in 0..10 {
        assert!(state.get_visible(DocLine(i)));
    }
    // All expanded (back in one-to-one mode)
    for i in 0..10 {
        assert!(state.get_expanded(DocLine(i)));
    }
    // Fold text cleared
    assert_eq!(state.get_fold_display_text(DocLine(5)), None);
}

#[test]
fn mapping_layer_does_not_store_fold_levels() {
    // Validates: Requirement 10.7
    // ContractionState only stores a boolean expanded/collapsed per line.
    // No fold level, depth, or region extent data is stored.
    let mut state = ContractionState::new(20);
    state.set_expanded(DocLine(5), false);
    // Only boolean state available
    assert!(!state.get_expanded(DocLine(5)));
    assert!(state.get_expanded(DocLine(6)));
}
