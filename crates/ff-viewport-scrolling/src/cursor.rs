//! Cursor position and column affinity tracking.
//!
//! The `CursorModel` maintains the editing cursor position within the document
//! and implements column affinity (Scintilla's `lastXChosen`) for natural
//! vertical navigation through lines of varying length.

/// Whether column affinity is tracked in pixel or column units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AffinityMode {
    /// Column-based affinity (monospace/character-grid editors).
    #[default]
    Columns,
    /// Pixel-based affinity (proportional-font editors).
    Pixels,
}

/// Cursor position and column affinity tracking.
///
/// Maintains the editing caret position and implements column affinity for
/// natural vertical cursor movement through lines of varying length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorModel {
    /// Current cursor line (1-based document line).
    cursor_line: u64,
    /// Current cursor column (1-based).
    cursor_column: u64,
    /// Remembered column for vertical movement (column affinity / lastXChosen).
    column_affinity: u64,
    /// Whether column_affinity is measured in pixels or columns.
    affinity_mode: AffinityMode,
}

impl CursorModel {
    /// Create a new cursor at line 1, column 1.
    pub fn new() -> Self {
        Self {
            cursor_line: 1,
            cursor_column: 1,
            column_affinity: 1,
            affinity_mode: AffinityMode::default(),
        }
    }

    /// Current cursor line (1-based).
    pub fn cursor_line(&self) -> u64 {
        self.cursor_line
    }

    /// Current cursor column (1-based).
    pub fn cursor_column(&self) -> u64 {
        self.cursor_column
    }

    /// Current column affinity value.
    pub fn column_affinity(&self) -> u64 {
        self.column_affinity
    }

    /// Current affinity mode.
    pub fn affinity_mode(&self) -> AffinityMode {
        self.affinity_mode
    }

    /// Set the affinity mode (columns or pixels).
    pub fn set_affinity_mode(&mut self, mode: AffinityMode) {
        self.affinity_mode = mode;
    }

    /// Move cursor down one line. Returns the new cursor_line.
    ///
    /// Applies column affinity to determine target column on the new line.
    /// If the cursor is already on the last line, this is a no-op.
    pub fn move_down(&mut self, target_line_length: u64, total_lines: u64) -> u64 {
        if self.cursor_line >= total_lines {
            return self.cursor_line;
        }
        self.cursor_line += 1;
        // Apply column affinity: place at affinity column or end of line
        self.cursor_column = if target_line_length >= self.column_affinity {
            self.column_affinity
        } else {
            // Line is shorter than affinity — place at end, preserve affinity
            target_line_length.max(1)
        };
        self.cursor_line
    }

    /// Move cursor up one line. Returns the new cursor_line.
    ///
    /// Applies column affinity to determine target column on the new line.
    /// If the cursor is already on line 1, this is a no-op.
    pub fn move_up(&mut self, target_line_length: u64) -> u64 {
        if self.cursor_line <= 1 {
            return self.cursor_line;
        }
        self.cursor_line -= 1;
        // Apply column affinity: place at affinity column or end of line
        self.cursor_column = if target_line_length >= self.column_affinity {
            self.column_affinity
        } else {
            target_line_length.max(1)
        };
        self.cursor_line
    }

    /// Move cursor left one column.
    ///
    /// Updates column_affinity to the new position. Clamps to column 1.
    pub fn move_left(&mut self) {
        if self.cursor_column > 1 {
            self.cursor_column -= 1;
        }
        self.column_affinity = self.cursor_column;
    }

    /// Move cursor right one column.
    ///
    /// Updates column_affinity to the new position. Clamps to line_length + 1
    /// (the end-of-line position).
    pub fn move_right(&mut self, current_line_length: u64) {
        let max_column = current_line_length + 1;
        if self.cursor_column < max_column {
            self.cursor_column += 1;
        }
        self.column_affinity = self.cursor_column;
    }

    /// Set cursor to a specific position (e.g., click).
    ///
    /// Resets column_affinity to the new column.
    pub fn set_position(&mut self, line: u64, column: u64) {
        self.cursor_line = line.max(1);
        self.cursor_column = column.max(1);
        self.column_affinity = self.cursor_column;
    }

    /// Move cursor to the beginning of the current line.
    ///
    /// Resets column_affinity.
    pub fn move_home(&mut self) {
        self.cursor_column = 1;
        self.column_affinity = 1;
    }

    /// Move cursor to the end of the current line.
    ///
    /// Resets column_affinity.
    pub fn move_end(&mut self, current_line_length: u64) {
        self.cursor_column = current_line_length + 1;
        self.column_affinity = self.cursor_column;
    }
}

impl Default for CursorModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cursor_starts_at_line_1_column_1() {
        let cursor = CursorModel::new();
        assert_eq!(cursor.cursor_line(), 1);
        assert_eq!(cursor.cursor_column(), 1);
        assert_eq!(cursor.column_affinity(), 1);
    }

    #[test]
    fn move_down_advances_cursor_line() {
        let mut cursor = CursorModel::new();
        cursor.move_down(80, 100);
        assert_eq!(cursor.cursor_line(), 2);
    }

    #[test]
    fn move_down_at_last_line_is_noop() {
        let mut cursor = CursorModel::new();
        cursor.set_position(100, 5);
        cursor.move_down(80, 100);
        assert_eq!(cursor.cursor_line(), 100);
    }

    #[test]
    fn move_up_retreats_cursor_line() {
        let mut cursor = CursorModel::new();
        cursor.set_position(5, 3);
        cursor.move_up(80);
        assert_eq!(cursor.cursor_line(), 4);
    }

    #[test]
    fn move_up_at_first_line_is_noop() {
        let mut cursor = CursorModel::new();
        cursor.move_up(80);
        assert_eq!(cursor.cursor_line(), 1);
    }

    #[test]
    fn move_left_retreats_cursor_column() {
        let mut cursor = CursorModel::new();
        cursor.set_position(1, 5);
        cursor.move_left();
        assert_eq!(cursor.cursor_column(), 4);
        assert_eq!(cursor.column_affinity(), 4);
    }

    #[test]
    fn move_left_at_column_1_is_noop() {
        let mut cursor = CursorModel::new();
        cursor.move_left();
        assert_eq!(cursor.cursor_column(), 1);
    }

    #[test]
    fn move_right_advances_cursor_column() {
        let mut cursor = CursorModel::new();
        cursor.move_right(80);
        assert_eq!(cursor.cursor_column(), 2);
        assert_eq!(cursor.column_affinity(), 2);
    }

    #[test]
    fn move_right_at_end_of_line_is_noop() {
        let mut cursor = CursorModel::new();
        cursor.set_position(1, 81);
        cursor.move_right(80);
        assert_eq!(cursor.cursor_column(), 81);
    }

    #[test]
    fn column_affinity_preserved_through_short_lines() {
        let mut cursor = CursorModel::new();
        // Start at column 10
        cursor.set_position(1, 10);
        assert_eq!(cursor.column_affinity(), 10);

        // Move down to a short line (length 5)
        cursor.move_down(5, 100);
        assert_eq!(cursor.cursor_line(), 2);
        assert_eq!(cursor.cursor_column(), 5); // clamped to line end
        assert_eq!(cursor.column_affinity(), 10); // preserved

        // Move down to a long line (length 20)
        cursor.move_down(20, 100);
        assert_eq!(cursor.cursor_line(), 3);
        assert_eq!(cursor.cursor_column(), 10); // restored to affinity
    }

    #[test]
    fn set_position_resets_column_affinity() {
        let mut cursor = CursorModel::new();
        cursor.set_position(5, 15);
        assert_eq!(cursor.column_affinity(), 15);
    }

    #[test]
    fn move_home_sets_column_to_1() {
        let mut cursor = CursorModel::new();
        cursor.set_position(3, 10);
        cursor.move_home();
        assert_eq!(cursor.cursor_column(), 1);
        assert_eq!(cursor.column_affinity(), 1);
    }

    #[test]
    fn move_end_sets_column_past_line_end() {
        let mut cursor = CursorModel::new();
        cursor.move_end(40);
        assert_eq!(cursor.cursor_column(), 41);
        assert_eq!(cursor.column_affinity(), 41);
    }
}
