//! Paragraph navigation implementation.
//!
//! Moves the caret to the previous or next paragraph boundary.
//! A paragraph boundary is defined as a blank or whitespace-only line.

use ff_viewport_scrolling::{CursorModel, ViewportModel};

use crate::types::SelectionModifier;

/// Paragraph navigation executor.
pub struct ParagraphNav;

impl ParagraphNav {
    /// Move caret to the previous paragraph boundary.
    ///
    /// Searches backwards from the current caret position for a blank line
    /// (paragraph boundary), then positions the caret on the first non-blank
    /// line after that boundary.
    ///
    /// If no boundary is found, moves to line 1.
    pub fn paragraph_up(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        lines: &[&str],
        excluded_lines: &[bool],
        _selection: SelectionModifier,
    ) {
        let current_line = cursor.cursor_line() as usize;
        let total = lines.len();

        if total == 0 || current_line <= 1 {
            cursor.set_position(1, 1);
            viewport.scroll_to_line(1, cursor);
            return;
        }

        // Search backwards from current_line - 1
        let mut target = 1;
        let mut in_blank_group = false;
        let start_idx = (current_line - 1).min(total); // 0-based index for line above current

        // Skip any initial blank lines at current position
        let mut i = start_idx;
        while i > 0 {
            i -= 1;
            if Self::is_excluded(i, excluded_lines) {
                continue;
            }
            if Self::is_paragraph_boundary_str(lines.get(i).unwrap_or(&"")) {
                in_blank_group = true;
            } else if in_blank_group {
                // Found first content line after a blank group going backwards
                target = (i + 1) as u64;
                break;
            } else {
                // Still in content, keep scanning
            }

            if i == 0 && !in_blank_group {
                target = 1;
                break;
            }
            if i == 0 && in_blank_group {
                target = 1;
                break;
            }
        }

        cursor.set_position(target, 1);
        // Ensure viewport shows the caret
        if target < viewport.top_line() {
            viewport.scroll_to_line(target, cursor);
        }
    }

    /// Move caret to the next paragraph boundary.
    ///
    /// Searches forwards from the current caret position for a blank line,
    /// then positions the caret on the first non-blank line after that boundary.
    ///
    /// If no boundary is found, moves to the last line.
    pub fn paragraph_down(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        lines: &[&str],
        excluded_lines: &[bool],
        _selection: SelectionModifier,
    ) {
        let current_line = cursor.cursor_line() as usize;
        let total = lines.len();

        if total == 0 {
            return;
        }

        let last_line = total as u64;
        let mut target = last_line;
        let mut in_blank_group = false;

        // Search forwards from current_line (0-based: current_line index)
        for i in current_line..total {
            if Self::is_excluded(i, excluded_lines) {
                continue;
            }
            if Self::is_paragraph_boundary_str(lines.get(i).unwrap_or(&"")) {
                in_blank_group = true;
            } else if in_blank_group {
                // Found first content line after a blank group
                target = (i + 1) as u64; // 1-based
                break;
            }
        }

        cursor.set_position(target, 1);
        // Ensure viewport shows the caret
        let bottom_visible = viewport.top_line() + viewport.visible_count().saturating_sub(1);
        if target > bottom_visible {
            let new_top = target.saturating_sub(viewport.visible_count().saturating_sub(1));
            viewport.scroll_to_line(new_top.max(1), cursor);
        }
    }

    /// Check if a line is a paragraph boundary (empty or whitespace-only).
    pub fn is_paragraph_boundary(line_content: &[u8]) -> bool {
        line_content.iter().all(|b| b.is_ascii_whitespace())
    }

    /// Check if a string line is a paragraph boundary.
    pub fn is_paragraph_boundary_str(line: &str) -> bool {
        line.chars().all(|c| c.is_whitespace())
    }

    /// Check if a line at a given index is excluded.
    fn is_excluded(idx: usize, excluded: &[bool]) -> bool {
        excluded.get(idx).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph_boundary_detection() {
        // Validates: Requirement 6.3
        assert!(ParagraphNav::is_paragraph_boundary(b""));
        assert!(ParagraphNav::is_paragraph_boundary(b"   "));
        assert!(ParagraphNav::is_paragraph_boundary(b"\t  "));
        assert!(!ParagraphNav::is_paragraph_boundary(b"hello"));
        assert!(!ParagraphNav::is_paragraph_boundary(b" x "));
    }

    #[test]
    fn paragraph_down_basic() {
        // Validates: Requirement 6.2
        let lines = vec!["line1", "line2", "", "line4", "line5"];
        let excluded = vec![false; 5];
        let mut viewport = ViewportModel::with_line_count(5);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();

        ParagraphNav::paragraph_down(
            &mut cursor,
            &mut viewport,
            &lines,
            &excluded,
            SelectionModifier::Move,
        );
        // Should land on line4 (index 3, 1-based = 4)
        assert_eq!(cursor.cursor_line(), 4);
    }

    #[test]
    fn paragraph_down_no_boundary_goes_to_end() {
        // Validates: Requirement 6.5
        let lines = vec!["line1", "line2", "line3"];
        let excluded = vec![false; 3];
        let mut viewport = ViewportModel::with_line_count(3);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();

        ParagraphNav::paragraph_down(
            &mut cursor,
            &mut viewport,
            &lines,
            &excluded,
            SelectionModifier::Move,
        );
        assert_eq!(cursor.cursor_line(), 3);
    }

    #[test]
    fn paragraph_up_basic() {
        // Validates: Requirement 6.1
        let lines = vec!["line1", "line2", "", "line4", "line5"];
        let excluded = vec![false; 5];
        let mut viewport = ViewportModel::with_line_count(5);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        cursor.set_position(5, 1);

        ParagraphNav::paragraph_up(
            &mut cursor,
            &mut viewport,
            &lines,
            &excluded,
            SelectionModifier::Move,
        );
        // Moving up from line 5, should find blank at index 2, land on line before it
        // Going backwards from line 4 (idx 3): idx 2 is blank → in_blank_group=true
        // Then idx 1 is content → target = idx 1 + 1 = 2 (1-based)
        assert_eq!(cursor.cursor_line(), 2);
    }

    #[test]
    fn paragraph_up_at_start_goes_to_line_1() {
        // Validates: Requirement 6.4
        let lines = vec!["line1", "line2", "line3"];
        let excluded = vec![false; 3];
        let mut viewport = ViewportModel::with_line_count(3);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        cursor.set_position(2, 5);

        ParagraphNav::paragraph_up(
            &mut cursor,
            &mut viewport,
            &lines,
            &excluded,
            SelectionModifier::Move,
        );
        assert_eq!(cursor.cursor_line(), 1);
    }

    #[test]
    fn paragraph_skips_excluded_lines() {
        // Validates: Requirement 6.9
        let lines = vec!["line1", "line2", "", "excluded", "line5"];
        let excluded = vec![false, false, false, true, false];
        let mut viewport = ViewportModel::with_line_count(5);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();

        ParagraphNav::paragraph_down(
            &mut cursor,
            &mut viewport,
            &lines,
            &excluded,
            SelectionModifier::Move,
        );
        // Should skip excluded line at index 3, land on line5 (index 4, 1-based = 5)
        assert_eq!(cursor.cursor_line(), 5);
    }
}
