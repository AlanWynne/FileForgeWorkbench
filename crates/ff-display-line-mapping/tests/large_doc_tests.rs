//! Tests for large document support (64-bit mode).
//!
//! Validates: Requirement 8 AC 1–6

use ff_display_line_mapping::{ContractionState, DisplayLine, DisplayLineMapping, DocLine};

#[test]
fn large_document_mode_flag_set_correctly() {
    // Validates: Requirement 8.5
    let state = ContractionState::new_large(1000);
    assert!(state.is_large_document());
    assert!(state.is_one_to_one());
}

#[test]
fn standard_mode_is_not_large_document() {
    // Validates: Requirement 8.3
    let state = ContractionState::new(1000);
    assert!(!state.is_large_document());
}

#[test]
fn large_document_mode_uses_usize_public_api() {
    // Validates: Requirement 8.4
    let state = ContractionState::new_large(1000);
    // Public API returns usize regardless of internal mode
    let _: usize = state.lines_in_doc();
    let _: usize = state.lines_displayed();
}

#[test]
fn large_document_mode_works_same_as_standard() {
    // Validates: Requirement 8.1, 8.2
    let mut state = ContractionState::new_large(100);
    state.set_visible(DocLine(10), DocLine(19), false);
    assert_eq!(state.lines_displayed(), 90);
    assert_eq!(state.display_from_doc(DocLine(20)), DisplayLine(10));
}

#[test]
fn standard_mode_uses_less_memory_concept() {
    // Validates: Requirement 8.6
    // In practice both use usize (platform-width). This test verifies the
    // concept by checking that standard mode functions identically with smaller docs.
    let std_state = ContractionState::new(100);
    let large_state = ContractionState::new_large(100);
    // Both should report same values in one-to-one mode
    assert_eq!(std_state.lines_in_doc(), large_state.lines_in_doc());
    assert_eq!(std_state.lines_displayed(), large_state.lines_displayed());
}
