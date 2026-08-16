//! Rectangular (column) selection support.
//!
//! Provides the `RectangularSelection` type that represents a column-oriented
//! selection defined by corner positions, and converts to per-line SelectionRanges.

use crate::position::SelectionPosition;
use crate::range::SelectionRange;

/// Distinguishes the kind of selection active in the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionKind {
    /// Normal stream selection flowing across line boundaries.
    #[default]
    Stream,
    /// Rectangular (column) selection defined by corner positions.
    Rectangular,
}

/// A rectangular (column) selection defined by four edges.
///
/// All values are 0-based. The selection covers lines from `top_line` to
/// `bottom_line` (inclusive) and columns from `left_column` to `right_column`
/// (inclusive) on each line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RectangularSelection {
    /// Top line of the rectangle (0-based, inclusive).
    pub top_line: u64,
    /// Bottom line of the rectangle (0-based, inclusive).
    pub bottom_line: u64,
    /// Left column of the rectangle (0-based, inclusive).
    pub left_column: u64,
    /// Right column of the rectangle (0-based, inclusive).
    pub right_column: u64,
}

impl RectangularSelection {
    /// Creates a rectangular selection from an Alt+drag operation.
    ///
    /// The start and current positions define the diagonal corners;
    /// the rectangle is normalised so that top <= bottom and left <= right.
    pub fn from_alt_drag(start: SelectionPosition, current: SelectionPosition) -> Self {
        let top_line = start.line.min(current.line);
        let bottom_line = start.line.max(current.line);
        let left_column = start.effective_column().min(current.effective_column());
        let right_column = start.effective_column().max(current.effective_column());

        Self {
            top_line,
            bottom_line,
            left_column,
            right_column,
        }
    }

    /// Extends the rectangular selection in a direction.
    ///
    /// - Up: decreases top_line (min 0)
    /// - Down: increases bottom_line
    /// - Left: decreases left_column (min 0)
    /// - Right: increases right_column
    pub fn extend(&mut self, direction: RectDirection) {
        match direction {
            RectDirection::Up => {
                self.top_line = self.top_line.saturating_sub(1);
            }
            RectDirection::Down => {
                self.bottom_line += 1;
            }
            RectDirection::Left => {
                self.left_column = self.left_column.saturating_sub(1);
            }
            RectDirection::Right => {
                self.right_column += 1;
            }
        }
    }

    /// Converts the rectangular selection to one `SelectionRange` per line.
    ///
    /// Each range spans from `left_column` to `right_column` on its respective line.
    /// Lines shorter than `right_column` will have virtual space in their range.
    pub fn to_selection_ranges(&self) -> Vec<SelectionRange> {
        (self.top_line..=self.bottom_line)
            .map(|line| {
                let anchor = SelectionPosition::new(line, self.left_column);
                let caret = SelectionPosition::new(line, self.right_column);
                SelectionRange::new(anchor, caret)
            })
            .collect()
    }

    /// Returns the number of lines this selection spans.
    pub fn line_count(&self) -> u64 {
        self.bottom_line - self.top_line + 1
    }

    /// Returns the width of the selected column range.
    pub fn column_width(&self) -> u64 {
        self.right_column - self.left_column + 1
    }

    /// Collapses the rectangular selection to a single caret at the top-left corner.
    pub fn collapse_to_caret(&self) -> SelectionPosition {
        SelectionPosition::new(self.top_line, self.left_column)
    }
}

/// Direction for extending a rectangular selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectDirection {
    /// Extend upward (decrease top_line).
    Up,
    /// Extend downward (increase bottom_line).
    Down,
    /// Extend leftward (decrease left_column).
    Left,
    /// Extend rightward (increase right_column).
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_alt_drag_normalises_to_top_left_bottom_right() {
        // Drag from bottom-right to top-left
        let start = SelectionPosition::new(5, 20);
        let current = SelectionPosition::new(2, 5);
        let rect = RectangularSelection::from_alt_drag(start, current);

        assert_eq!(rect.top_line, 2);
        assert_eq!(rect.bottom_line, 5);
        assert_eq!(rect.left_column, 5);
        assert_eq!(rect.right_column, 20);
    }

    #[test]
    fn from_alt_drag_top_left_to_bottom_right() {
        let start = SelectionPosition::new(1, 3);
        let current = SelectionPosition::new(4, 10);
        let rect = RectangularSelection::from_alt_drag(start, current);

        assert_eq!(rect.top_line, 1);
        assert_eq!(rect.bottom_line, 4);
        assert_eq!(rect.left_column, 3);
        assert_eq!(rect.right_column, 10);
    }

    #[test]
    fn extend_up_decreases_top_line() {
        let mut rect = RectangularSelection {
            top_line: 3,
            bottom_line: 5,
            left_column: 2,
            right_column: 10,
        };
        rect.extend(RectDirection::Up);
        assert_eq!(rect.top_line, 2);
    }

    #[test]
    fn extend_up_clamps_at_zero() {
        let mut rect = RectangularSelection {
            top_line: 0,
            bottom_line: 5,
            left_column: 2,
            right_column: 10,
        };
        rect.extend(RectDirection::Up);
        assert_eq!(rect.top_line, 0);
    }

    #[test]
    fn extend_down_increases_bottom_line() {
        let mut rect = RectangularSelection {
            top_line: 3,
            bottom_line: 5,
            left_column: 2,
            right_column: 10,
        };
        rect.extend(RectDirection::Down);
        assert_eq!(rect.bottom_line, 6);
    }

    #[test]
    fn extend_left_decreases_left_column() {
        let mut rect = RectangularSelection {
            top_line: 3,
            bottom_line: 5,
            left_column: 2,
            right_column: 10,
        };
        rect.extend(RectDirection::Left);
        assert_eq!(rect.left_column, 1);
    }

    #[test]
    fn extend_right_increases_right_column() {
        let mut rect = RectangularSelection {
            top_line: 3,
            bottom_line: 5,
            left_column: 2,
            right_column: 10,
        };
        rect.extend(RectDirection::Right);
        assert_eq!(rect.right_column, 11);
    }

    #[test]
    fn to_selection_ranges_produces_one_range_per_line() {
        let rect = RectangularSelection {
            top_line: 2,
            bottom_line: 5,
            left_column: 3,
            right_column: 10,
        };
        let ranges = rect.to_selection_ranges();
        assert_eq!(ranges.len(), 4);

        for (i, range) in ranges.iter().enumerate() {
            assert_eq!(range.anchor.line, 2 + i as u64);
            assert_eq!(range.anchor.column, 3);
            assert_eq!(range.caret.line, 2 + i as u64);
            assert_eq!(range.caret.column, 10);
        }
    }

    #[test]
    fn line_count_is_inclusive() {
        let rect = RectangularSelection {
            top_line: 2,
            bottom_line: 5,
            left_column: 0,
            right_column: 10,
        };
        assert_eq!(rect.line_count(), 4);
    }

    #[test]
    fn column_width_is_inclusive() {
        let rect = RectangularSelection {
            top_line: 0,
            bottom_line: 0,
            left_column: 3,
            right_column: 10,
        };
        assert_eq!(rect.column_width(), 8);
    }

    #[test]
    fn collapse_to_caret_returns_top_left() {
        let rect = RectangularSelection {
            top_line: 3,
            bottom_line: 7,
            left_column: 5,
            right_column: 20,
        };
        let pos = rect.collapse_to_caret();
        assert_eq!(pos.line, 3);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn selection_kind_default_is_stream() {
        assert_eq!(SelectionKind::default(), SelectionKind::Stream);
    }
}
