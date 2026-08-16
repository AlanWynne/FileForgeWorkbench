//! Viewport navigation commands (UP, DOWN, LEFT, RIGHT, TOP, BOTTOM).
//!
//! These commands scroll the viewport without modifying document content.
//! All operations delegate to `ViewportModel` for state mutation and clamping.

use ff_viewport_scrolling::{CursorModel, ViewportModel};

use crate::types::NavigationConfig;

/// Viewport scroll command executors.
pub struct ScrollCommands;

impl ScrollCommands {
    /// Scroll viewport up by page (visible_count - overlap).
    ///
    /// The overlap retains context lines at the boundary.
    pub fn up_page(
        viewport: &mut ViewportModel,
        cursor: &mut CursorModel,
        config: &NavigationConfig,
    ) {
        let page_amount = viewport
            .visible_count()
            .saturating_sub(config.page_overlap_lines);
        let page_amount = page_amount.max(1);
        let new_top = viewport.top_line().saturating_sub(page_amount).max(1);
        viewport.scroll_to_line(new_top, cursor);
    }

    /// Scroll viewport up by n lines.
    pub fn up_lines(viewport: &mut ViewportModel, cursor: &mut CursorModel, n: u64) {
        let new_top = viewport.top_line().saturating_sub(n).max(1);
        viewport.scroll_to_line(new_top, cursor);
    }

    /// Scroll viewport down by page (visible_count - overlap).
    pub fn down_page(
        viewport: &mut ViewportModel,
        cursor: &mut CursorModel,
        config: &NavigationConfig,
    ) {
        let page_amount = viewport
            .visible_count()
            .saturating_sub(config.page_overlap_lines);
        let page_amount = page_amount.max(1);
        let new_top = viewport.top_line().saturating_add(page_amount);
        let max = viewport.max_top_line();
        let clamped = new_top.min(max);
        viewport.scroll_to_line(clamped, cursor);
    }

    /// Scroll viewport down by n lines.
    pub fn down_lines(viewport: &mut ViewportModel, cursor: &mut CursorModel, n: u64) {
        let new_top = viewport.top_line().saturating_add(n);
        let max = viewport.max_top_line();
        let clamped = new_top.min(max);
        viewport.scroll_to_line(clamped, cursor);
    }

    /// Scroll viewport left by configured default amount.
    pub fn left_default(
        viewport: &mut ViewportModel,
        cursor: &CursorModel,
        config: &NavigationConfig,
    ) {
        let current = viewport.horizontal_offset();
        let new_offset = current.saturating_sub(config.horizontal_scroll_columns);
        viewport.set_horizontal_offset(new_offset, cursor);
    }

    /// Scroll viewport left by n columns.
    pub fn left_columns(viewport: &mut ViewportModel, cursor: &CursorModel, n: u64) {
        let current = viewport.horizontal_offset();
        let new_offset = current.saturating_sub(n);
        viewport.set_horizontal_offset(new_offset, cursor);
    }

    /// Scroll viewport right by configured default amount.
    pub fn right_default(
        viewport: &mut ViewportModel,
        cursor: &CursorModel,
        config: &NavigationConfig,
    ) {
        let current = viewport.horizontal_offset();
        let new_offset = current.saturating_add(config.horizontal_scroll_columns);
        viewport.set_horizontal_offset(new_offset, cursor);
    }

    /// Scroll viewport right by n columns.
    pub fn right_columns(viewport: &mut ViewportModel, cursor: &CursorModel, n: u64) {
        let current = viewport.horizontal_offset();
        let new_offset = current.saturating_add(n);
        viewport.set_horizontal_offset(new_offset, cursor);
    }

    /// Scroll to first line and update cursor.
    pub fn top(viewport: &mut ViewportModel, cursor: &mut CursorModel) {
        viewport.scroll_to_line(1, cursor);
        cursor.set_position(1, 1);
    }

    /// Scroll to last page and update cursor.
    pub fn bottom(viewport: &mut ViewportModel, cursor: &mut CursorModel, doc_line_count: u64) {
        let max = viewport.max_top_line();
        viewport.scroll_to_line(max, cursor);
        cursor.set_position(doc_line_count.max(1), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_viewport(total_lines: u64, visible: u64) -> (ViewportModel, CursorModel) {
        let mut vp = ViewportModel::with_line_count(total_lines);
        vp.set_visible_count(visible);
        vp.set_max_horizontal_extent(10000); // Allow horizontal scrolling in tests
        let cursor = CursorModel::new();
        (vp, cursor)
    }

    #[test]
    fn up_page_scrolls_by_visible_minus_overlap() {
        // Validates: Requirement 3.1
        let (mut vp, mut cursor) = setup_viewport(100, 20);
        let config = NavigationConfig::default(); // overlap = 2
                                                  // Start at line 50
        vp.scroll_to_line(50, &cursor);
        ScrollCommands::up_page(&mut vp, &mut cursor, &config);
        assert_eq!(vp.top_line(), 32); // 50 - (20 - 2) = 32
    }

    #[test]
    fn up_lines_scrolls_by_n() {
        // Validates: Requirement 3.2
        let (mut vp, mut cursor) = setup_viewport(100, 20);
        vp.scroll_to_line(20, &cursor);
        ScrollCommands::up_lines(&mut vp, &mut cursor, 5);
        assert_eq!(vp.top_line(), 15);
    }

    #[test]
    fn down_page_scrolls_by_visible_minus_overlap() {
        // Validates: Requirement 3.3
        let (mut vp, mut cursor) = setup_viewport(100, 20);
        let config = NavigationConfig::default();
        ScrollCommands::down_page(&mut vp, &mut cursor, &config);
        assert_eq!(vp.top_line(), 19); // 1 + (20 - 2) = 19
    }

    #[test]
    fn down_lines_scrolls_by_n() {
        // Validates: Requirement 3.4
        let (mut vp, mut cursor) = setup_viewport(100, 20);
        ScrollCommands::down_lines(&mut vp, &mut cursor, 10);
        assert_eq!(vp.top_line(), 11);
    }

    #[test]
    fn up_clamps_at_line_1() {
        // Validates: Requirement 3.11
        let (mut vp, mut cursor) = setup_viewport(100, 20);
        ScrollCommands::up_lines(&mut vp, &mut cursor, 100);
        assert_eq!(vp.top_line(), 1);
    }

    #[test]
    fn down_clamps_at_max_top_line() {
        // Validates: Requirement 3.12
        let (mut vp, mut cursor) = setup_viewport(100, 20);
        ScrollCommands::down_lines(&mut vp, &mut cursor, 200);
        assert_eq!(vp.top_line(), vp.max_top_line());
    }

    #[test]
    fn left_columns_clamps_at_zero() {
        // Validates: Requirement 3.13
        let (mut vp, cursor) = setup_viewport(100, 20);
        ScrollCommands::left_columns(&mut vp, &cursor, 100);
        assert_eq!(vp.horizontal_offset(), 0);
    }

    #[test]
    fn right_columns_increases_offset() {
        // Validates: Requirement 3.8
        let (mut vp, cursor) = setup_viewport(100, 20);
        ScrollCommands::right_columns(&mut vp, &cursor, 15);
        assert_eq!(vp.horizontal_offset(), 15);
    }

    #[test]
    fn top_sets_line_1_and_cursor() {
        // Validates: Requirement 3.9, 3.16
        let (mut vp, mut cursor) = setup_viewport(100, 20);
        vp.scroll_to_line(50, &cursor);
        cursor.set_position(50, 10);
        ScrollCommands::top(&mut vp, &mut cursor);
        assert_eq!(vp.top_line(), 1);
        assert_eq!(cursor.cursor_line(), 1);
        assert_eq!(cursor.cursor_column(), 1);
    }

    #[test]
    fn bottom_sets_last_page_and_cursor() {
        // Validates: Requirement 3.10, 3.16
        let (mut vp, mut cursor) = setup_viewport(100, 20);
        ScrollCommands::bottom(&mut vp, &mut cursor, 100);
        assert_eq!(vp.top_line(), vp.max_top_line());
        assert_eq!(cursor.cursor_line(), 100);
        assert_eq!(cursor.cursor_column(), 1);
    }

    #[test]
    fn left_default_uses_config() {
        // Validates: Requirement 3.5
        let (mut vp, cursor) = setup_viewport(100, 20);
        ScrollCommands::right_columns(&mut vp, &cursor, 20);
        let config = NavigationConfig {
            horizontal_scroll_columns: 8,
            ..Default::default()
        };
        ScrollCommands::left_default(&mut vp, &cursor, &config);
        assert_eq!(vp.horizontal_offset(), 12); // 20 - 8
    }

    #[test]
    fn right_default_uses_config() {
        // Validates: Requirement 3.7
        let (mut vp, cursor) = setup_viewport(100, 20);
        let config = NavigationConfig {
            horizontal_scroll_columns: 8,
            ..Default::default()
        };
        ScrollCommands::right_default(&mut vp, &cursor, &config);
        assert_eq!(vp.horizontal_offset(), 8);
    }
}
