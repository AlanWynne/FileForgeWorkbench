//! Integration tests demonstrating viewport-style usage patterns.
//!
//! Validates: Requirement 7 AC 1-9, Requirement 11

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ff_display_line_mapping::{ContractionState, DisplayLine, DisplayLineMapping, DocLine};

#[test]
fn viewport_scroll_and_render_pattern() {
    // Validates: Requirement 7.1, 7.2, 7.3
    // Simulates: viewport with 20 display lines, user scrolls to display line 50
    let mut state = ContractionState::new(200);
    // Simulate some hidden lines
    state.set_visible(DocLine(10), DocLine(19), false);
    state.set_visible(DocLine(50), DocLine(59), false);
    // Simulate some wrapped lines
    state.set_height(DocLine(25), 3);
    state.set_height(DocLine(70), 2);

    let viewport_height = 20;
    let scroll_position = 50; // Display line offset

    // Translate scroll position to document lines
    let first_doc_pos = state.doc_from_display(DisplayLine(scroll_position));
    let last_doc_pos = state.doc_from_display(DisplayLine(scroll_position + viewport_height - 1));

    // The viewport should render lines from first_doc_pos to last_doc_pos
    assert!(first_doc_pos.doc_line.0 <= last_doc_pos.doc_line.0);

    // Scrollbar range = total display lines
    let scrollbar_max = state.lines_displayed();
    assert!(scrollbar_max < 200); // Fewer display lines due to hidden lines
}

#[test]
fn change_notification_fires_on_visibility_change() {
    // Validates: Requirement 7.9
    let mut state = ContractionState::new(100);
    let notification_count = Arc::new(AtomicUsize::new(0));
    let counter = notification_count.clone();

    state.on_display_count_change(Box::new(move |change| {
        counter.fetch_add(1, Ordering::SeqCst);
        assert!(change.old_count != change.new_count);
    }));

    state.set_visible(DocLine(10), DocLine(19), false);
    assert_eq!(notification_count.load(Ordering::SeqCst), 1);
}

#[test]
fn change_notification_fires_on_height_change() {
    // Validates: Requirement 7.9
    let mut state = ContractionState::new(100);
    let last_new_count = Arc::new(AtomicUsize::new(0));
    let counter = last_new_count.clone();

    state.on_display_count_change(Box::new(move |change| {
        counter.store(change.new_count, Ordering::SeqCst);
    }));

    state.set_height(DocLine(5), 4);
    assert_eq!(last_new_count.load(Ordering::SeqCst), 103);
}

#[test]
fn remove_listener_stops_notifications() {
    // Validates: Requirement 7.9
    let mut state = ContractionState::new(100);
    let notification_count = Arc::new(AtomicUsize::new(0));
    let counter = notification_count.clone();

    let handle = state.on_display_count_change(Box::new(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    }));

    state.set_visible(DocLine(0), DocLine(0), false);
    assert_eq!(notification_count.load(Ordering::SeqCst), 1);

    state.remove_listener(handle);
    state.set_visible(DocLine(1), DocLine(1), false);
    // Should not have received another notification
    assert_eq!(notification_count.load(Ordering::SeqCst), 1);
}

#[test]
fn fold_simulation_with_nested_folds() {
    // Validates: Requirement 3.9 (nested folds), Requirement 10
    let mut state = ContractionState::new(30);

    // Simulate outer fold: lines 5-15 (header=5, body=6-15)
    state.set_expanded(DocLine(5), false);
    state.set_fold_display_text(DocLine(5), Some("fn outer() { ... }"));
    state.set_visible(DocLine(6), DocLine(15), false);

    // Simulate inner fold: lines 8-12 (header=8, body=9-12)
    // Already hidden by outer fold
    state.set_expanded(DocLine(8), false);

    assert_eq!(state.lines_displayed(), 20); // 30 - 10 hidden

    // Expand outer fold
    state.set_expanded(DocLine(5), true);
    // Body lines made visible, but inner fold body stays hidden
    state.set_visible(DocLine(6), DocLine(8), true); // Lines 6-8 shown
    state.set_visible(DocLine(13), DocLine(15), true); // Lines 13-15 shown
                                                       // Lines 9-12 remain hidden (inner fold still collapsed)

    assert_eq!(state.lines_displayed(), 26); // 30 - 4 hidden (inner fold body)
    assert!(!state.get_expanded(DocLine(8))); // Inner fold still collapsed
}

#[test]
fn ispf_exclusion_and_fold_coexistence() {
    // Validates: Requirement 10.1, 10.2, 10.3
    let mut state = ContractionState::new(50);

    // ISPF EXCLUDE: hide lines 10-15
    state.set_visible(DocLine(10), DocLine(15), false);

    // Code fold: collapse at line 20 (body 21-25)
    state.set_expanded(DocLine(20), false);
    state.set_visible(DocLine(21), DocLine(25), false);

    assert_eq!(state.lines_displayed(), 39); // 50 - 6 - 5

    // ISPF SHOW: show lines 10-15
    state.set_visible(DocLine(10), DocLine(15), true);
    assert_eq!(state.lines_displayed(), 45); // 50 - 5 (fold still collapsed)

    // Fold state is still collapsed even after SHOW on other lines
    assert!(!state.get_expanded(DocLine(20)));
}

#[test]
fn display_line_mapping_trait_is_send_sync() {
    // Validates: Requirement 7.10 — trait is Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ContractionState>();
}

#[test]
fn complete_workflow_edit_fold_wrap_scroll() {
    // Validates: End-to-end integration
    let mut state = ContractionState::new(1000);

    // 1. User opens file — starts in one-to-one mode
    assert!(state.is_one_to_one());
    assert_eq!(state.lines_displayed(), 1000);

    // 2. Word wrap enabled — some lines get height > 1
    state.set_height(DocLine(100), 3);
    state.set_height(DocLine(200), 2);
    state.set_height(DocLine(500), 4);
    assert_eq!(state.lines_displayed(), 1006); // 1000 + 2 + 1 + 3

    // 3. User folds a function
    state.set_expanded(DocLine(150), false);
    state.set_visible(DocLine(151), DocLine(170), false);
    assert_eq!(state.lines_displayed(), 986); // 1006 - 20

    // 4. User EXCLUDES some lines
    state.set_visible(DocLine(300), DocLine(310), false);
    assert_eq!(state.lines_displayed(), 975); // 986 - 11

    // 5. User edits — insert 5 lines at position 400
    state.insert_lines(DocLine(400), 5);
    assert_eq!(state.lines_in_doc(), 1005);
    assert_eq!(state.lines_displayed(), 980); // 975 + 5

    // 6. Scrollbar queries total display lines
    let scrollbar_max = state.lines_displayed();
    assert_eq!(scrollbar_max, 980);

    // 7. User scrolls to display line 500
    let doc_pos = state.doc_from_display(DisplayLine(500));
    assert!(state.get_visible(doc_pos.doc_line));

    // 8. User does SHOW ALL
    state.show_all();
    assert!(state.is_one_to_one());
    assert_eq!(state.lines_displayed(), 1005);
}
