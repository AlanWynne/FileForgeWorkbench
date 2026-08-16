//! Document start and end navigation.
//!
//! Implements DOC_START (jump to position 0) and DOC_END (jump to end of last line).

use ff_viewport_scrolling::{CursorModel, ViewportModel};

use crate::types::SelectionModifier;

/// Document start/end navigation executor.
pub struct DocStartEndNav;

impl DocStartEndNav {
    /// Move caret to position 0 (first char of first line), scroll viewport to top.
    ///
    /// Resets column affinity to 1.
    pub fn document_start(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        _selection: SelectionModifier,
    ) {
        cursor.set_position(1, 1);
        viewport.scroll_to_line(1, cursor);
    }

    /// Move caret to end of last line, scroll viewport to last page.
    ///
    /// Updates column affinity to the caret's position on the last line.
    pub fn document_end(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        doc_line_count: u64,
        last_line_length: u64,
        _selection: SelectionModifier,
    ) {
        let end_col = last_line_length + 1; // Past last character
        cursor.set_position(doc_line_count.max(1), end_col);
        let max = viewport.max_top_line();
        viewport.scroll_to_line(max, cursor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_start_moves_to_line_1_column_1() {
        // Validates: Requirement 10.1
        let mut cursor = CursorModel::new();
        cursor.set_position(50, 20);
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        viewport.scroll_to_line(40, &cursor);

        DocStartEndNav::document_start(&mut cursor, &mut viewport, SelectionModifier::Move);
        assert_eq!(cursor.cursor_line(), 1);
        assert_eq!(cursor.cursor_column(), 1);
        assert_eq!(viewport.top_line(), 1);
    }

    #[test]
    fn document_start_resets_affinity() {
        // Validates: Requirement 10.5
        let mut cursor = CursorModel::new();
        cursor.set_position(50, 20);
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);

        DocStartEndNav::document_start(&mut cursor, &mut viewport, SelectionModifier::Move);
        assert_eq!(cursor.column_affinity(), 1);
    }

    #[test]
    fn document_end_moves_to_last_line_end() {
        // Validates: Requirement 10.2
        let mut cursor = CursorModel::new();
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);

        DocStartEndNav::document_end(&mut cursor, &mut viewport, 100, 45, SelectionModifier::Move);
        assert_eq!(cursor.cursor_line(), 100);
        assert_eq!(cursor.cursor_column(), 46); // past last char
    }

    #[test]
    fn document_end_updates_affinity() {
        // Validates: Requirement 10.6
        let mut cursor = CursorModel::new();
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);

        DocStartEndNav::document_end(&mut cursor, &mut viewport, 100, 45, SelectionModifier::Move);
        assert_eq!(cursor.column_affinity(), 46);
    }

    #[test]
    fn document_end_scrolls_to_last_page() {
        // Validates: Requirement 10.2
        let mut cursor = CursorModel::new();
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);

        DocStartEndNav::document_end(&mut cursor, &mut viewport, 100, 45, SelectionModifier::Move);
        assert_eq!(viewport.top_line(), viewport.max_top_line());
    }

    #[test]
    fn document_start_on_empty_doc() {
        let mut cursor = CursorModel::new();
        cursor.set_position(1, 5);
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_visible_count(20);

        DocStartEndNav::document_start(&mut cursor, &mut viewport, SelectionModifier::Move);
        assert_eq!(cursor.cursor_line(), 1);
        assert_eq!(cursor.cursor_column(), 1);
    }
}
