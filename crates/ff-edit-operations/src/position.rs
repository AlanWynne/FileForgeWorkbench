//! Selection position type with virtual space support.
//!
//! `SelectionPosition` represents a document position that includes both
//! a real position (line + column) and a virtual space offset for positions
//! beyond line ends.

use std::cmp::Ordering;

/// A document position including real coordinates and virtual space offset.
///
/// Virtual space allows the caret to be placed beyond the end of a line's
/// actual content. When an edit occurs in virtual space, the space is
/// "realised" by padding with actual space characters.
///
/// # Ordering
///
/// Positions are ordered by line first, then by effective column
/// (column + virtual_space). This gives document order for all positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectionPosition {
    /// 0-based line number in the document.
    pub line: u64,
    /// 0-based column offset within the line's content.
    pub column: u64,
    /// Virtual space columns beyond the end of the line's actual content.
    /// When > 0, the caret is in virtual space.
    pub virtual_space: u64,
}

impl SelectionPosition {
    /// Creates a new position at the given line and column with no virtual space.
    pub fn new(line: u64, column: u64) -> Self {
        Self {
            line,
            column,
            virtual_space: 0,
        }
    }

    /// Creates a new position with explicit virtual space.
    pub fn with_virtual_space(line: u64, column: u64, virtual_space: u64) -> Self {
        Self {
            line,
            column,
            virtual_space,
        }
    }

    /// Creates a position representing the end of a line.
    ///
    /// This is a convenience constructor; the actual line end column
    /// must be supplied by the caller (from document state).
    pub fn at_line_end(line: u64, line_length: u64) -> Self {
        Self {
            line,
            column: line_length,
            virtual_space: 0,
        }
    }

    /// Returns the effective column: real column + virtual space.
    ///
    /// This is the visual column position as the user perceives it.
    pub fn effective_column(&self) -> u64 {
        self.column.saturating_add(self.virtual_space)
    }

    /// Returns true if this position is in virtual space.
    pub fn is_in_virtual_space(&self) -> bool {
        self.virtual_space > 0
    }

    /// Returns the number of space characters needed to materialise virtual space.
    ///
    /// After realisation, the position should be updated to have
    /// `column = column + virtual_space` and `virtual_space = 0`.
    pub fn realise_virtual_space(&self) -> u64 {
        self.virtual_space
    }

    /// Shift this position forward by `amount` columns.
    ///
    /// Only the real column is shifted; virtual space is unchanged.
    pub fn shift_forward(&self, amount: u64) -> Self {
        Self {
            line: self.line,
            column: self.column.saturating_add(amount),
            virtual_space: self.virtual_space,
        }
    }

    /// Shift this position backward by `amount` columns.
    ///
    /// The column is clamped to 0 (never goes negative).
    /// Virtual space is unchanged.
    pub fn shift_backward(&self, amount: u64) -> Self {
        Self {
            line: self.line,
            column: self.column.saturating_sub(amount),
            virtual_space: self.virtual_space,
        }
    }

    /// Returns a new position with virtual space cleared (realised into column).
    pub fn with_realised_virtual_space(&self) -> Self {
        Self {
            line: self.line,
            column: self.column.saturating_add(self.virtual_space),
            virtual_space: 0,
        }
    }
}

impl PartialOrd for SelectionPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SelectionPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.line
            .cmp(&other.line)
            .then_with(|| self.effective_column().cmp(&other.effective_column()))
    }
}

impl Default for SelectionPosition {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_position_without_virtual_space() {
        let pos = SelectionPosition::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
        assert_eq!(pos.virtual_space, 0);
        assert!(!pos.is_in_virtual_space());
    }

    #[test]
    fn with_virtual_space_creates_position_in_virtual_space() {
        let pos = SelectionPosition::with_virtual_space(3, 8, 4);
        assert_eq!(pos.line, 3);
        assert_eq!(pos.column, 8);
        assert_eq!(pos.virtual_space, 4);
        assert!(pos.is_in_virtual_space());
    }

    #[test]
    fn effective_column_adds_virtual_space() {
        let pos = SelectionPosition::with_virtual_space(0, 10, 5);
        assert_eq!(pos.effective_column(), 15);
    }

    #[test]
    fn effective_column_without_virtual_space() {
        let pos = SelectionPosition::new(0, 10);
        assert_eq!(pos.effective_column(), 10);
    }

    #[test]
    fn at_line_end_creates_position_at_line_length() {
        let pos = SelectionPosition::at_line_end(2, 42);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 42);
        assert_eq!(pos.virtual_space, 0);
    }

    #[test]
    fn realise_virtual_space_returns_padding_amount() {
        let pos = SelectionPosition::with_virtual_space(0, 10, 7);
        assert_eq!(pos.realise_virtual_space(), 7);
    }

    #[test]
    fn realise_virtual_space_returns_zero_when_no_virtual_space() {
        let pos = SelectionPosition::new(0, 10);
        assert_eq!(pos.realise_virtual_space(), 0);
    }

    #[test]
    fn shift_forward_increases_column() {
        let pos = SelectionPosition::new(1, 5);
        let shifted = pos.shift_forward(3);
        assert_eq!(shifted.column, 8);
        assert_eq!(shifted.line, 1);
    }

    #[test]
    fn shift_backward_decreases_column() {
        let pos = SelectionPosition::new(1, 5);
        let shifted = pos.shift_backward(3);
        assert_eq!(shifted.column, 2);
        assert_eq!(shifted.line, 1);
    }

    #[test]
    fn shift_backward_clamps_to_zero() {
        let pos = SelectionPosition::new(1, 2);
        let shifted = pos.shift_backward(10);
        assert_eq!(shifted.column, 0);
    }

    #[test]
    fn ordering_compares_line_first() {
        let a = SelectionPosition::new(1, 100);
        let b = SelectionPosition::new(2, 0);
        assert!(a < b);
    }

    #[test]
    fn ordering_compares_effective_column_on_same_line() {
        let a = SelectionPosition::new(5, 10);
        let b = SelectionPosition::new(5, 20);
        assert!(a < b);
    }

    #[test]
    fn ordering_considers_virtual_space_in_effective_column() {
        let a = SelectionPosition::with_virtual_space(5, 10, 5); // effective 15
        let b = SelectionPosition::new(5, 20); // effective 20
        assert!(a < b);

        let c = SelectionPosition::with_virtual_space(5, 10, 15); // effective 25
        assert!(c > b);
    }

    #[test]
    fn equal_positions_are_equal() {
        let a = SelectionPosition::new(3, 7);
        let b = SelectionPosition::new(3, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn with_realised_virtual_space_moves_virtual_into_column() {
        let pos = SelectionPosition::with_virtual_space(2, 10, 5);
        let realised = pos.with_realised_virtual_space();
        assert_eq!(realised.line, 2);
        assert_eq!(realised.column, 15);
        assert_eq!(realised.virtual_space, 0);
    }

    #[test]
    fn default_position_is_at_document_start() {
        let pos = SelectionPosition::default();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
        assert_eq!(pos.virtual_space, 0);
    }
}
