//! Integration tests for ff-viewport-scrolling.
//!
//! End-to-end scenarios testing the full scroll lifecycle.

use ff_viewport_scrolling::commands::execute_scroll_command;
use ff_viewport_scrolling::{
    CaretPolicyEngine, CursorModel, ScrollCommand, ScrollFraction, ScrollMode, ViewportModel,
};

/// Integration test: full scroll scenario.
///
/// Open document → page down → cursor move → scroll to bottom → restore.
#[test]
fn full_scroll_scenario() {
    // Validates: Requirements 1, 2, 3, 12
    let mut viewport = ViewportModel::with_line_count(1000);
    viewport.set_visible_count(40);
    let mut cursor = CursorModel::new();
    let policy = CaretPolicyEngine::default_policy();

    // Initial state
    assert_eq!(viewport.top_line(), 1);
    assert_eq!(viewport.max_top_line(), 961);

    // Page down
    viewport.scroll_page_down(&mut cursor);
    assert_eq!(viewport.top_line(), 41);
    assert_eq!(cursor.cursor_line(), 41);

    // Move cursor down several times
    for _i in 0..10 {
        viewport.move_cursor_down(&mut cursor, 80, 1000, &policy);
    }
    assert_eq!(cursor.cursor_line(), 51);

    // Scroll to bottom
    viewport.scroll_to_bottom(&cursor);
    assert_eq!(viewport.top_line(), 961);

    // Take snapshot
    let snapshot = viewport.snapshot(&cursor);
    assert_eq!(snapshot.top_line, 961);
    assert_eq!(snapshot.cursor_line, 51);

    // Restore into a new viewport with same document
    let mut viewport2 = ViewportModel::with_line_count(1000);
    viewport2.set_visible_count(40);
    let mut cursor2 = CursorModel::new();
    viewport2.restore(&snapshot, &mut cursor2);

    assert_eq!(viewport2.top_line(), 961);
    assert_eq!(cursor2.cursor_line(), 51);
}

/// Integration test: viewport with display line mapping (wrapping + scrollbar).
#[test]
fn viewport_with_display_line_mapping() {
    // Validates: Requirement 11
    use ff_viewport_scrolling::DisplayLineMapper;

    struct WrappingMapper;

    impl DisplayLineMapper for WrappingMapper {
        fn total_display_lines(&self) -> u64 {
            2000
        }
        fn doc_to_display(&self, doc_line: u64) -> u64 {
            doc_line * 2 - 1
        }
        fn display_to_doc(&self, display_line: u64) -> u64 {
            (display_line + 1) / 2
        }
        fn is_visible(&self, _doc_line: u64) -> bool {
            true
        }
        fn display_lines_for_doc_line(&self, _doc_line: u64) -> u64 {
            2
        }
    }

    let mut viewport = ViewportModel::with_line_count(1000);
    viewport.set_visible_count(40);

    // Before mapper: based on raw line count
    assert_eq!(viewport.total_display_lines(), 1000);
    assert_eq!(viewport.max_top_line(), 961);

    // Attach mapper — now 2000 display lines
    viewport.set_display_mapper(Some(Box::new(WrappingMapper)));
    assert_eq!(viewport.total_display_lines(), 2000);
    assert_eq!(viewport.max_top_line(), 1961);

    // Scrollbar thumb should reflect new total
    let thumb = viewport.vertical_scrollbar_thumb_ratio();
    assert!((thumb - 0.02).abs() < 0.001); // 40/2000 = 0.02
}

/// Integration test: command dispatch and event emission.
#[test]
fn command_dispatch_emits_events() {
    // Validates: Requirement 10
    use ff_viewport_scrolling::{ViewportChanged, ViewportObserver};
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

    struct Counter(Arc<AtomicU64>);
    impl ViewportObserver for Counter {
        fn on_viewport_changed(&self, _: &ViewportChanged) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    let mut viewport = ViewportModel::with_line_count(500);
    viewport.set_visible_count(20);
    let count = Arc::new(AtomicU64::new(0));
    viewport.add_observer(Box::new(Counter(count.clone())));

    let mut cursor = CursorModel::new();
    let policy = CaretPolicyEngine::default_policy();

    // Execute several commands
    let commands = [
        ScrollCommand::ScrollLineDown,
        ScrollCommand::ScrollLineDown,
        ScrollCommand::ScrollPageDown,
        ScrollCommand::ScrollToTop,
        ScrollCommand::ScrollToBottom,
    ];

    for cmd in &commands {
        execute_scroll_command(cmd, &mut viewport, &mut cursor, &policy);
    }

    assert_eq!(count.load(Ordering::Relaxed), 5);
}

/// Integration test: scrollbar precision with large files.
#[test]
fn scrollbar_precision_large_file() {
    // Validates: Requirement 13
    use ff_viewport_scrolling::VerticalScrollbar;

    let total = 5_000_000u64;
    let visible = 50u64;
    let max_top = total - visible + 1;

    // Verify monotonicity across 1000 steps
    let mut prev = 0u64;
    for i in 0..=1000 {
        let f = i as f64 / 1000.0;
        let top = VerticalScrollbar::fraction_to_top_line(ScrollFraction::new(f), max_top);
        assert!(
            top >= prev,
            "non-monotonic at step {}: {} < {}",
            i,
            top,
            prev
        );
        prev = top;
    }

    // Boundary checks
    let at_zero = VerticalScrollbar::fraction_to_top_line(ScrollFraction::new(0.0), max_top);
    assert_eq!(at_zero, 1);

    let at_one = VerticalScrollbar::fraction_to_top_line(ScrollFraction::new(1.0), max_top);
    assert_eq!(at_one, max_top);
}

/// Integration test: smooth scrolling pixel offset management.
#[test]
fn smooth_scrolling_pixel_management() {
    // Validates: Requirement 9
    let mut viewport = ViewportModel::with_line_count(1000);
    viewport.set_visible_count(40);
    viewport.set_line_height(20);

    // Default mode is Line — pixel offset should stay 0
    viewport.set_pixel_offset(10);
    assert_eq!(viewport.pixel_offset().0, 0); // not in smooth mode

    // Switch to smooth mode
    viewport.set_scroll_mode(ScrollMode::Smooth);
    viewport.set_pixel_offset(15);
    assert_eq!(viewport.pixel_offset().0, 15);

    // Overflow wraps around line_height
    viewport.set_pixel_offset(25);
    assert_eq!(viewport.pixel_offset().0, 5); // 25 % 20 = 5

    // Switching back to Line resets pixel_offset
    viewport.set_scroll_mode(ScrollMode::Line);
    assert_eq!(viewport.pixel_offset().0, 0);
}

/// Integration test: column affinity across varying line lengths.
#[test]
fn column_affinity_integration() {
    // Validates: Requirement 6
    let mut cursor = CursorModel::new();

    // Position cursor at column 50
    cursor.set_position(1, 50);
    assert_eq!(cursor.column_affinity(), 50);

    // Move down through lines of varying length
    // Line 2: length 30 (shorter than affinity)
    cursor.move_down(30, 10);
    assert_eq!(cursor.cursor_line(), 2);
    assert_eq!(cursor.cursor_column(), 30); // clamped
    assert_eq!(cursor.column_affinity(), 50); // preserved

    // Line 3: length 10 (even shorter)
    cursor.move_down(10, 10);
    assert_eq!(cursor.cursor_line(), 3);
    assert_eq!(cursor.cursor_column(), 10);
    assert_eq!(cursor.column_affinity(), 50);

    // Line 4: length 80 (longer than affinity)
    cursor.move_down(80, 10);
    assert_eq!(cursor.cursor_line(), 4);
    assert_eq!(cursor.cursor_column(), 50); // restored to affinity
    assert_eq!(cursor.column_affinity(), 50);

    // Horizontal move resets affinity
    cursor.move_right(80);
    assert_eq!(cursor.cursor_column(), 51);
    assert_eq!(cursor.column_affinity(), 51);
}

/// Integration test: word-wrap disables horizontal scrollbar.
#[test]
fn word_wrap_disables_horizontal_scrollbar() {
    // Validates: Requirement 7.7
    let mut viewport = ViewportModel::with_line_count(100);
    viewport.set_visible_count(40);
    viewport.set_max_horizontal_extent(500);

    assert!(!viewport.is_horizontal_scrollbar_disabled());

    viewport.set_word_wrap_enabled(true);
    assert!(viewport.is_horizontal_scrollbar_disabled());
    assert_eq!(viewport.horizontal_offset(), 0); // reset to 0
}

/// Integration test: snapshot restore with shortened document.
#[test]
fn snapshot_restore_with_shortened_document() {
    // Validates: Requirement 12.3, 12.4
    let mut viewport = ViewportModel::with_line_count(1000);
    viewport.set_visible_count(40);
    let mut cursor = CursorModel::new();
    cursor.set_position(900, 5);
    let policy = CaretPolicyEngine::default_policy();
    viewport.set_cursor_position(&mut cursor, 900, 5, &policy);

    let snapshot = viewport.snapshot(&cursor);

    // Restore into shorter document (only 200 lines)
    let mut viewport2 = ViewportModel::with_line_count(200);
    viewport2.set_visible_count(40);
    let mut cursor2 = CursorModel::new();
    viewport2.restore(&snapshot, &mut cursor2);

    // Should be clamped
    assert!(viewport2.top_line() <= viewport2.max_top_line());
    assert!(cursor2.cursor_line() <= 200);
}
