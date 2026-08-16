//! Property-based tests for ff-viewport-scrolling.
//!
//! Each property test validates a core invariant of the viewport model
//! across randomly generated inputs.

use proptest::prelude::*;

use ff_viewport_scrolling::commands::execute_scroll_command;
use ff_viewport_scrolling::{
    CaretPolicyEngine, CursorModel, ScrollCommand, ScrollFraction, ScrollMode, VerticalScrollbar,
    ViewportModel,
};

// ─── Strategies ─────────────────────────────────────────────────────────────

// ─── P1: top_line always in [1, max_top_line] after any operation ────────────

proptest! {
    /// **Validates: Requirement 1.10**
    ///
    /// P1: top_line is always in [1, max_top_line] after any resize operation.
    #[test]
    fn p1_top_line_always_in_bounds_after_resize(
        total in 1u64..100_000,
        visible in 1u64..1000,
        initial_top in 1u64..100_000,
    ) {
        let mut viewport = ViewportModel::with_line_count(total);
        viewport.set_visible_count(visible.max(1));
        // Manually set top_line then resize to trigger clamping
        let cursor = CursorModel::new();
        viewport.scroll_to_line(initial_top, &cursor);

        // Now resize — should clamp
        viewport.set_visible_count(visible);

        let max = viewport.max_top_line();
        prop_assert!(viewport.top_line() >= 1);
        prop_assert!(viewport.top_line() <= max);
    }
}

// ─── P2: Scroll commands never produce out-of-bounds top_line ────────────────

proptest! {
    /// **Validates: Requirement 2**
    ///
    /// P2: After any scroll command, top_line is in [1, max_top_line].
    #[test]
    fn p2_scroll_commands_never_exceed_bounds(
        total in 1u64..50_000,
        visible in 1u64..500,
        command_idx in 0u8..7,
        target_line in 0u64..60_000,
    ) {
        let mut viewport = ViewportModel::with_line_count(total);
        viewport.set_visible_count(visible);
        let mut cursor = CursorModel::new();
        let policy = CaretPolicyEngine::default_policy();

        let command = match command_idx {
            0 => ScrollCommand::ScrollLineUp,
            1 => ScrollCommand::ScrollLineDown,
            2 => ScrollCommand::ScrollPageUp,
            3 => ScrollCommand::ScrollPageDown,
            4 => ScrollCommand::ScrollToLine(target_line),
            5 => ScrollCommand::ScrollToTop,
            _ => ScrollCommand::ScrollToBottom,
        };

        execute_scroll_command(&command, &mut viewport, &mut cursor, &policy);

        let max = viewport.max_top_line();
        prop_assert!(viewport.top_line() >= 1);
        prop_assert!(viewport.top_line() <= max);
    }
}

// ─── P3: Cursor always within visible range after move + viewport adjust ─────

proptest! {
    /// **Validates: Requirement 3**
    ///
    /// P3: After cursor move with viewport adjustment, cursor is visible.
    #[test]
    fn p3_cursor_visible_after_move_down(
        total in 2u64..10_000,
        visible in 1u64..500,
        start_line in 1u64..10_000,
        line_length in 1u64..200,
    ) {
        let total = total.max(2);
        let start_line = start_line.min(total);
        let mut viewport = ViewportModel::with_line_count(total);
        viewport.set_visible_count(visible);
        let mut cursor = CursorModel::new();
        cursor.set_position(start_line, 1);
        let policy = CaretPolicyEngine::default_policy();

        // Sync viewport to cursor first
        viewport.set_cursor_position(&mut cursor, start_line, 1, &policy);

        // Move cursor down
        viewport.move_cursor_down(&mut cursor, line_length, total, &policy);

        let top = viewport.top_line();
        let bottom = top + viewport.visible_count() - 1;
        prop_assert!(
            cursor.cursor_line() >= top && cursor.cursor_line() <= bottom,
            "cursor {} not in [{}, {}]", cursor.cursor_line(), top, bottom
        );
    }
}

proptest! {
    /// **Validates: Requirement 3**
    ///
    /// P3b: After cursor move up with viewport adjustment, cursor is visible.
    #[test]
    fn p3_cursor_visible_after_move_up(
        total in 2u64..10_000,
        visible in 1u64..500,
        start_line in 1u64..10_000,
        line_length in 1u64..200,
    ) {
        let total = total.max(2);
        let start_line = start_line.min(total).max(1);
        let mut viewport = ViewportModel::with_line_count(total);
        viewport.set_visible_count(visible);
        let mut cursor = CursorModel::new();
        cursor.set_position(start_line, 1);
        let policy = CaretPolicyEngine::default_policy();

        viewport.set_cursor_position(&mut cursor, start_line, 1, &policy);
        viewport.move_cursor_up(&mut cursor, line_length, &policy);

        let top = viewport.top_line();
        let bottom = top + viewport.visible_count() - 1;
        prop_assert!(
            cursor.cursor_line() >= top && cursor.cursor_line() <= bottom,
            "cursor {} not in [{}, {}]", cursor.cursor_line(), top, bottom
        );
    }
}

// ─── P4: Scrollbar round-trip (top_line → fraction → top_line) ───────────────

proptest! {
    /// **Validates: Requirement 4.8**
    ///
    /// P4: Scrollbar fraction round-trip produces original value within ±1.
    #[test]
    fn p4_scrollbar_round_trip(
        total in 2u64..1_000_000,
        visible in 1u64..10_000,
    ) {
        let max_top = if total <= visible { 1 } else { total - visible + 1 };
        if max_top <= 1 {
            // Scrollbar disabled case — trivial
            return Ok(());
        }

        // Pick a random top_line in valid range
        let top_line = 1u64.max(max_top / 3);

        let fraction = VerticalScrollbar::position_fraction(top_line, max_top);
        let restored = VerticalScrollbar::fraction_to_top_line(fraction, max_top);

        // Round-trip should be within ±1 due to floating point
        let diff = if restored > top_line {
            restored - top_line
        } else {
            top_line - restored
        };
        prop_assert!(diff <= 1, "round-trip diff {} > 1 (top={}, restored={})", diff, top_line, restored);
    }
}

// ─── P5: Caret policy ensures cursor visible ─────────────────────────────────

proptest! {
    /// **Validates: Requirement 5**
    ///
    /// P5: After caret policy adjustment, cursor is within visible bounds.
    #[test]
    fn p5_caret_policy_ensures_cursor_visible(
        total in 10u64..10_000,
        visible in 5u64..500,
        cursor_line in 1u64..10_000,
        _slop_value in 0u32..10,
    ) {
        let total = total.max(10);
        let visible = visible.min(total);
        let cursor_line = cursor_line.min(total);
        let max_top = total - visible + 1;

        let engine = CaretPolicyEngine::default_policy();
        let new_top = engine.compute_vertical_scroll(cursor_line, 1, visible, max_top);

        let bottom = new_top + visible - 1;
        prop_assert!(
            cursor_line >= new_top && cursor_line <= bottom,
            "cursor {} not in [{}, {}] (max_top={})",
            cursor_line, new_top, bottom, max_top
        );
    }
}

// ─── P6: Column affinity preserved through short-line sequences ──────────────

proptest! {
    /// **Validates: Requirement 6**
    ///
    /// P6: Column affinity is preserved across vertical moves through short lines.
    #[test]
    fn p6_column_affinity_preserved(
        initial_column in 1u64..200,
        line_lengths in prop::collection::vec(1u64..200, 3..10),
    ) {
        let mut cursor = CursorModel::new();
        let total_lines = line_lengths.len() as u64 + 1;

        // Set initial position at column with known affinity
        cursor.set_position(1, initial_column);
        let saved_affinity = cursor.column_affinity();
        prop_assert_eq!(saved_affinity, initial_column);

        // Move through lines — affinity should be preserved
        for (i, &length) in line_lengths.iter().enumerate() {
            cursor.move_down(length, total_lines);
            // Affinity is always the original value
            prop_assert_eq!(
                cursor.column_affinity(), saved_affinity,
                "affinity changed at line {}", i + 2
            );
        }
    }
}

// ─── P7: horizontal_offset always in [0, max_horizontal_extent] ──────────────

proptest! {
    /// **Validates: Requirement 7**
    ///
    /// P7: horizontal_offset never exceeds max_horizontal_extent.
    #[test]
    fn p7_horizontal_offset_in_bounds(
        max_extent in 0u64..10_000,
        ticks in -20i32..20,
        pixels_per_tick in 1u32..50,
    ) {
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(40);
        viewport.set_max_horizontal_extent(max_extent);
        let cursor = CursorModel::new();

        viewport.scroll_wheel_horizontal(ticks, pixels_per_tick, &cursor);

        prop_assert!(viewport.horizontal_offset() <= max_extent);
    }
}

// ─── P8: Mouse wheel never exceeds document bounds ───────────────────────────

proptest! {
    /// **Validates: Requirement 8**
    ///
    /// P8: Mouse wheel scrolling never produces out-of-bounds top_line.
    #[test]
    fn p8_wheel_scroll_in_bounds(
        total in 1u64..100_000,
        visible in 1u64..1000,
        ticks in -100i32..100,
    ) {
        let mut viewport = ViewportModel::with_line_count(total);
        viewport.set_visible_count(visible);
        let cursor = CursorModel::new();

        viewport.scroll_wheel_vertical(ticks, &cursor);

        let max = viewport.max_top_line();
        prop_assert!(viewport.top_line() >= 1);
        prop_assert!(viewport.top_line() <= max);
    }
}

// ─── P9: pixel_offset always in [0, line_height) in smooth mode ──────────────

proptest! {
    /// **Validates: Requirement 9**
    ///
    /// P9: pixel_offset is always in [0, line_height) when smooth mode active.
    #[test]
    fn p9_pixel_offset_in_range(
        line_height in 1u32..100,
        offset in 0u32..10_000,
    ) {
        let mut viewport = ViewportModel::with_line_count(1000);
        viewport.set_visible_count(40);
        viewport.set_line_height(line_height);
        viewport.set_scroll_mode(ScrollMode::Smooth);
        viewport.set_pixel_offset(offset);

        prop_assert!(viewport.pixel_offset().0 < line_height,
            "pixel_offset {} >= line_height {}",
            viewport.pixel_offset().0, line_height
        );
    }
}

// ─── P10: ViewportChanged event emitted after every mutation ─────────────────

use ff_viewport_scrolling::{ViewportChanged, ViewportObserver};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

struct CountingObserver {
    count: Arc<AtomicU64>,
}

impl ViewportObserver for CountingObserver {
    fn on_viewport_changed(&self, _event: &ViewportChanged) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

proptest! {
    /// **Validates: Requirement 10.5**
    ///
    /// P10: ViewportChanged event is emitted after every state mutation.
    #[test]
    fn p10_event_emitted_after_mutations(
        command_indices in prop::collection::vec(0u8..7, 1..20),
        total in 10u64..10_000,
        visible in 5u64..500,
    ) {
        let mut viewport = ViewportModel::with_line_count(total);
        viewport.set_visible_count(visible);

        let count = Arc::new(AtomicU64::new(0));
        let observer = CountingObserver { count: count.clone() };
        viewport.add_observer(Box::new(observer));

        let mut cursor = CursorModel::new();
        let policy = CaretPolicyEngine::default_policy();

        let mut expected_count = 0u64;
        for &idx in &command_indices {
            let command = match idx {
                0 => ScrollCommand::ScrollLineUp,
                1 => ScrollCommand::ScrollLineDown,
                2 => ScrollCommand::ScrollPageUp,
                3 => ScrollCommand::ScrollPageDown,
                4 => ScrollCommand::ScrollToTop,
                5 => ScrollCommand::ScrollToBottom,
                _ => ScrollCommand::ScrollToLine(5),
            };
            execute_scroll_command(&command, &mut viewport, &mut cursor, &policy);
            expected_count += 1;
        }

        prop_assert_eq!(count.load(Ordering::Relaxed), expected_count);
    }
}

// ─── P11: Scrollbar total equals DisplayLineMapper total ─────────────────────

use ff_viewport_scrolling::DisplayLineMapper;

struct MockMapper {
    total: u64,
}

impl DisplayLineMapper for MockMapper {
    fn total_display_lines(&self) -> u64 {
        self.total
    }
    fn doc_to_display(&self, doc_line: u64) -> u64 {
        doc_line
    }
    fn display_to_doc(&self, display_line: u64) -> u64 {
        display_line
    }
    fn is_visible(&self, _doc_line: u64) -> bool {
        true
    }
    fn display_lines_for_doc_line(&self, _doc_line: u64) -> u64 {
        1
    }
}

proptest! {
    /// **Validates: Requirement 11.5**
    ///
    /// P11: When a DisplayLineMapper is set, total_display_lines matches mapper.
    #[test]
    fn p11_scrollbar_uses_mapper_total(
        mapper_total in 1u64..100_000,
        raw_total in 1u64..100_000,
    ) {
        let mut viewport = ViewportModel::with_line_count(raw_total);
        let mapper = MockMapper { total: mapper_total };
        viewport.set_display_mapper(Some(Box::new(mapper)));

        prop_assert_eq!(viewport.total_display_lines(), mapper_total);
    }
}

// ─── P12: restore(snapshot(state)) produces equivalent state ─────────────────

proptest! {
    /// **Validates: Requirement 12**
    ///
    /// P12: Snapshot round-trip produces equivalent state when document unchanged.
    #[test]
    fn p12_snapshot_round_trip(
        total in 1u64..10_000,
        visible in 1u64..500,
        cursor_line in 1u64..10_000,
        cursor_column in 1u64..200,
    ) {
        let total = total.max(1);
        let visible = visible.min(total);
        let cursor_line = cursor_line.min(total);

        let mut viewport = ViewportModel::with_line_count(total);
        viewport.set_visible_count(visible);
        let mut cursor = CursorModel::new();
        cursor.set_position(cursor_line, cursor_column);

        // Sync viewport
        let policy = CaretPolicyEngine::default_policy();
        viewport.set_cursor_position(&mut cursor, cursor_line, cursor_column, &policy);

        let snapshot = viewport.snapshot(&cursor);

        // Restore into same-sized document
        let mut viewport2 = ViewportModel::with_line_count(total);
        viewport2.set_visible_count(visible);
        let mut cursor2 = CursorModel::new();
        viewport2.restore(&snapshot, &mut cursor2);

        prop_assert_eq!(viewport2.top_line(), viewport.top_line());
        prop_assert_eq!(cursor2.cursor_line(), cursor.cursor_line());
        prop_assert_eq!(cursor2.cursor_column(), cursor.cursor_column());
    }
}

// ─── P13: Scrollbar mapping is monotonically non-decreasing ──────────────────

proptest! {
    /// **Validates: Requirement 13.2**
    ///
    /// P13: Scrollbar mapping is monotonically non-decreasing across fraction range.
    #[test]
    fn p13_scrollbar_monotonic(
        total in 100u64..10_000_000,
        visible in 1u64..1000,
        steps in 10u32..200,
    ) {
        let max_top = if total <= visible { 1 } else { total - visible + 1 };
        if max_top <= 1 {
            return Ok(());
        }

        let mut prev_top_line = 0u64;
        for i in 0..=steps {
            let f = i as f64 / steps as f64;
            let fraction = ScrollFraction::new(f);
            let top_line = VerticalScrollbar::fraction_to_top_line(fraction, max_top);

            prop_assert!(
                top_line >= prev_top_line,
                "monotonicity violated: step {}/{}, prev={}, cur={}, f={}",
                i, steps, prev_top_line, top_line, f
            );
            prev_top_line = top_line;
        }
    }
}
