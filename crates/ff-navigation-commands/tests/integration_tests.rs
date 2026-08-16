//! Integration tests for ff-navigation-commands.
//!
//! End-to-end scenarios testing interactions between multiple navigation
//! commands, viewport state, cursor position, and bounds.

use ff_navigation_commands::locate::LabelRegistry;
use ff_navigation_commands::types::{
    ActiveBounds, NavigationConfig, SelectionModifier, SortDirection, SortParams, SortScope,
};
use ff_navigation_commands::{
    BoundsManager, CharClassifier, ColsManager, DocStartEndNav, LocateCommand, ParagraphNav,
    ScrollCommands, SortCommand, VerticalCaretNav, WordNav, WordPartNav,
};
use ff_viewport_scrolling::{CursorModel, ViewportModel};

// ─── Test Helpers ──────────────────────────────────────────────────────────

struct TestLabels {
    labels: Vec<(&'static str, u64)>,
}

impl LabelRegistry for TestLabels {
    fn resolve_label(&self, name: &str) -> Option<u64> {
        self.labels
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, l)| *l)
    }
}

// ─── Integration Test 19.1: LOCATE → viewport scroll → cursor update ───────

#[test]
fn locate_viewport_scroll_cursor_update_lifecycle() {
    // Validates: Requirement 1.1, 1.3, 1.6
    let mut viewport = ViewportModel::with_line_count(500);
    viewport.set_visible_count(25);
    let mut cursor = CursorModel::new();
    cursor.set_position(1, 15); // Start somewhere

    // LOCATE to line 200
    let result = LocateCommand::locate_line(&mut viewport, &mut cursor, 200, 500);
    assert!(result.is_ok());
    assert_eq!(cursor.cursor_line(), 200);
    assert_eq!(cursor.cursor_column(), 1); // Reset to column 1

    // LOCATE to a label
    let labels = TestLabels {
        labels: vec![("DATA_SECTION", 350)],
    };
    let result =
        LocateCommand::locate_label(&mut viewport, &mut cursor, "DATA_SECTION", &labels, 500);
    assert!(result.is_ok());
    assert_eq!(cursor.cursor_line(), 350);
    assert_eq!(cursor.cursor_column(), 1);
}

// ─── Integration Test 19.2: SORT with bounds and undo record ───────────────

#[test]
fn sort_with_bounds_interaction_and_undo_record() {
    // Validates: Requirement 2.9, 2.10, 2.11
    let mut lines = vec![
        "XXCC hello".to_string(),
        "XXAA world".to_string(),
        "XXBB first".to_string(),
    ];

    // Set up bounds that restrict sort key to columns 3-4
    let mut bounds_mgr = BoundsManager::new();
    bounds_mgr.set_bounds(3, 4).unwrap();

    let params = SortParams {
        column_range: None,
        direction: SortDirection::Ascending,
        scope: SortScope::AllVisible,
    };

    let bounds = bounds_mgr.active_bounds();
    let record = SortCommand::execute(&mut lines, &params, bounds).unwrap();

    // Lines sorted by columns 3-4: "CC", "AA", "BB" → AA, BB, CC
    assert_eq!(lines[0], "XXAA world");
    assert_eq!(lines[1], "XXBB first");
    assert_eq!(lines[2], "XXCC hello");

    // Undo record captures original order
    assert_eq!(record.original_order, vec![1, 2, 0]);
}

// ─── Integration Test 19.3: Word and word-part navigation ──────────────────

#[test]
fn word_and_word_part_navigation_across_multi_line() {
    // Validates: Requirement 7.2, 7.3, 7.5, 8.1, 8.2
    let classifier = CharClassifier::new();
    let lines = vec!["getValue here", "nextWord there"];

    // Word right from start of first line
    let (line, col) = WordNav::word_right(&lines, 1, 1, &classifier, SelectionModifier::Move);
    assert_eq!(line, 1);
    // "getValue" is all Word-class chars (8 chars), space at 9, "here" starts at 10
    assert_eq!(col, 10); // Lands on 'h' of "here"

    // Word left from position 10 goes back to start of "getValue"
    let (line, col) = WordNav::word_left(&lines, 1, 10, &classifier, SelectionModifier::Move);
    assert_eq!(line, 1);
    assert_eq!(col, 1);

    // Word-part navigation within "getValue"
    let pos = WordPartNav::word_part_right("getValue", 0, SelectionModifier::Move);
    assert_eq!(pos, 3); // "get|Value"

    let pos = WordPartNav::word_part_right("getValue", 3, SelectionModifier::Move);
    assert_eq!(pos, 8); // end of "Value"
}

// ─── Integration Test 19.4: Paragraph navigation with excluded lines ───────

#[test]
fn paragraph_navigation_with_excluded_lines() {
    // Validates: Requirement 6.2, 6.9
    let lines = vec![
        "first paragraph line 1",
        "first paragraph line 2",
        "",
        "excluded line",
        "second paragraph line 1",
        "second paragraph line 2",
    ];
    // Line at index 3 is excluded
    let excluded = vec![false, false, false, true, false, false];

    let mut viewport = ViewportModel::with_line_count(6);
    viewport.set_visible_count(20);
    let mut cursor = CursorModel::new();

    // Paragraph down from line 1 should skip the blank line and excluded line
    ParagraphNav::paragraph_down(
        &mut cursor,
        &mut viewport,
        &lines,
        &excluded,
        SelectionModifier::Move,
    );
    // Should land on "second paragraph line 1" (line 5, 1-based)
    assert_eq!(cursor.cursor_line(), 5);
}

// ─── Integration Test 19.5: COLS/BOUNDS display artifact lifecycle ──────────

#[test]
fn cols_bounds_display_artifact_lifecycle() {
    // Validates: Requirement 4.1, 4.4, 4.5, 5.1, 5.4
    let mut cols_mgr = ColsManager::new();
    let mut bounds_mgr = BoundsManager::new();

    // Insert COLS at line 5
    cols_mgr.toggle_at(5);
    assert_eq!(cols_mgr.active_cols_lines().len(), 1);

    // Insert COLS at line 10
    cols_mgr.toggle_at(10);
    assert_eq!(cols_mgr.active_cols_lines().len(), 2);

    // Toggle off at line 5
    cols_mgr.toggle_at(5);
    assert_eq!(cols_mgr.active_cols_lines().len(), 1);
    assert_eq!(cols_mgr.active_cols_lines()[0].anchor_line, 10);

    // RESET clears all
    cols_mgr.reset_all();
    assert!(cols_mgr.active_cols_lines().is_empty());

    // Set bounds
    bounds_mgr.set_bounds(5, 72).unwrap();
    assert_eq!(
        bounds_mgr.active_bounds(),
        Some(ActiveBounds { left: 5, right: 72 })
    );

    // Clear bounds
    bounds_mgr.clear_bounds();
    assert_eq!(bounds_mgr.active_bounds(), None);
}

// ─── Integration Test 19.6: Full navigation sequence with clamping ─────────

#[test]
fn full_navigation_sequence_top_down_locate_bottom() {
    // Validates: Requirement 3.9, 3.4, 1.1, 3.10, 3.11, 3.12
    let mut viewport = ViewportModel::with_line_count(200);
    viewport.set_visible_count(20);
    let mut cursor = CursorModel::new();

    // TOP
    ScrollCommands::top(&mut viewport, &mut cursor);
    assert_eq!(viewport.top_line(), 1);
    assert_eq!(cursor.cursor_line(), 1);

    // DOWN 50
    ScrollCommands::down_lines(&mut viewport, &mut cursor, 50);
    assert_eq!(viewport.top_line(), 51);

    // LOCATE 100
    LocateCommand::locate_line(&mut viewport, &mut cursor, 100, 200).unwrap();
    assert_eq!(cursor.cursor_line(), 100);

    // BOTTOM
    ScrollCommands::bottom(&mut viewport, &mut cursor, 200);
    assert_eq!(cursor.cursor_line(), 200);
    assert_eq!(viewport.top_line(), viewport.max_top_line());

    // UP past beginning → clamped
    ScrollCommands::up_lines(&mut viewport, &mut cursor, 500);
    assert_eq!(viewport.top_line(), 1);

    // DOWN past end → clamped
    ScrollCommands::down_lines(&mut viewport, &mut cursor, 500);
    assert_eq!(viewport.top_line(), viewport.max_top_line());
}

// ─── Integration Test 19.7: Vertical caret with affinity ───────────────────

#[test]
fn vertical_caret_movement_with_affinity_across_varying_lines() {
    // Validates: Requirement 9.1, 9.3, 9.4
    let mut cursor = CursorModel::new();
    cursor.set_position(1, 20); // column 20, affinity = 20
    let mut viewport = ViewportModel::with_line_count(5);
    viewport.set_visible_count(5);

    // Line 2 has length 10 (shorter than affinity)
    VerticalCaretNav::line_down(&mut cursor, &mut viewport, 10, 5, SelectionModifier::Move);
    assert_eq!(cursor.cursor_line(), 2);
    assert_eq!(cursor.cursor_column(), 10); // clamped to line end
    assert_eq!(cursor.column_affinity(), 20); // affinity preserved

    // Line 3 has length 30 (longer than affinity)
    VerticalCaretNav::line_down(&mut cursor, &mut viewport, 30, 5, SelectionModifier::Move);
    assert_eq!(cursor.cursor_line(), 3);
    assert_eq!(cursor.cursor_column(), 20); // restored to affinity

    // Line 4 has length 5 (very short)
    VerticalCaretNav::line_down(&mut cursor, &mut viewport, 5, 5, SelectionModifier::Move);
    assert_eq!(cursor.cursor_line(), 4);
    assert_eq!(cursor.cursor_column(), 5); // clamped
    assert_eq!(cursor.column_affinity(), 20); // still preserved

    // Line 5 has length 50 (long)
    VerticalCaretNav::line_down(&mut cursor, &mut viewport, 50, 5, SelectionModifier::Move);
    assert_eq!(cursor.cursor_line(), 5);
    assert_eq!(cursor.cursor_column(), 20); // restored again
}

// ─── Integration Test: Document start/end lifecycle ────────────────────────

#[test]
fn doc_start_end_full_lifecycle() {
    // Validates: Requirement 10.1, 10.2, 10.5, 10.6
    let mut cursor = CursorModel::new();
    cursor.set_position(50, 25);
    let mut viewport = ViewportModel::with_line_count(100);
    viewport.set_visible_count(20);
    viewport.scroll_to_line(40, &cursor);

    // DOC_START
    DocStartEndNav::document_start(&mut cursor, &mut viewport, SelectionModifier::Move);
    assert_eq!(cursor.cursor_line(), 1);
    assert_eq!(cursor.cursor_column(), 1);
    assert_eq!(cursor.column_affinity(), 1);
    assert_eq!(viewport.top_line(), 1);

    // DOC_END
    DocStartEndNav::document_end(&mut cursor, &mut viewport, 100, 80, SelectionModifier::Move);
    assert_eq!(cursor.cursor_line(), 100);
    assert_eq!(cursor.cursor_column(), 81); // past last char
    assert_eq!(cursor.column_affinity(), 81);
    assert_eq!(viewport.top_line(), viewport.max_top_line());
}
