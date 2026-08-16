//! Selection range type representing a contiguous selected region.
//!
//! A `SelectionRange` is defined by an anchor (fixed end) and a caret (moving end).
//! The selected text spans between these two positions regardless of document order.

use crate::position::SelectionPosition;

/// An ordered pair (anchor, caret) defining a contiguous selected region.
///
/// The anchor is the fixed end of the selection (where selection started).
/// The caret is the moving end (where the cursor currently is).
/// The selected text is all text between anchor and caret regardless of
/// their document order (anchor may be after caret for backward selections).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionRange {
    /// The fixed end of the selection (start point).
    pub anchor: SelectionPosition,
    /// The moving end of the selection (cursor position).
    pub caret: SelectionPosition,
}

impl SelectionRange {
    /// Creates a new selection range from anchor and caret positions.
    pub fn new(anchor: SelectionPosition, caret: SelectionPosition) -> Self {
        Self { anchor, caret }
    }

    /// Creates a collapsed selection (no text selected, just a caret position).
    ///
    /// A collapsed range has anchor == caret.
    pub fn collapsed(position: SelectionPosition) -> Self {
        Self {
            anchor: position,
            caret: position,
        }
    }

    /// Returns true when anchor == caret (no text is selected).
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.caret
    }

    /// Returns the position that comes first in document order.
    ///
    /// This is the lesser of anchor and caret.
    pub fn start(&self) -> SelectionPosition {
        std::cmp::min(self.anchor, self.caret)
    }

    /// Returns the position that comes last in document order.
    ///
    /// This is the greater of anchor and caret.
    pub fn end(&self) -> SelectionPosition {
        std::cmp::max(self.anchor, self.caret)
    }

    /// Returns true if the given position is within this range (inclusive of start, exclusive of end).
    ///
    /// A collapsed range contains no positions.
    pub fn contains(&self, pos: &SelectionPosition) -> bool {
        if self.is_collapsed() {
            return false;
        }
        let start = self.start();
        let end = self.end();
        *pos >= start && *pos < end
    }

    /// Returns true if this range overlaps with another range.
    ///
    /// Two ranges overlap if one starts before the other ends and vice versa.
    /// Adjacent ranges (one ends exactly where the other starts) do NOT overlap.
    pub fn overlaps(&self, other: &SelectionRange) -> bool {
        let self_start = self.start();
        let self_end = self.end();
        let other_start = other.start();
        let other_end = other.end();

        // Two collapsed ranges at same position overlap
        if self.is_collapsed() && other.is_collapsed() {
            return self_start == other_start;
        }

        // A collapsed range overlaps a non-collapsed if it's inside
        if self.is_collapsed() {
            return self_start >= other_start && self_start < other_end;
        }
        if other.is_collapsed() {
            return other_start >= self_start && other_start < self_end;
        }

        // Two non-collapsed ranges overlap if they intersect
        self_start < other_end && other_start < self_end
    }

    /// Produces the union (merge) of two overlapping ranges.
    ///
    /// The resulting range spans from the minimum start to the maximum end.
    /// The anchor is the min position, the caret is the max position.
    pub fn merge(&self, other: &SelectionRange) -> SelectionRange {
        let min_pos = std::cmp::min(self.start(), other.start());
        let max_pos = std::cmp::max(self.end(), other.end());
        SelectionRange {
            anchor: min_pos,
            caret: max_pos,
        }
    }

    /// Returns the start position for sorting purposes (document order).
    ///
    /// Used by `SelectionContainer` to maintain sorted order.
    pub fn sort_key(&self) -> SelectionPosition {
        self.start()
    }
}

impl Default for SelectionRange {
    fn default() -> Self {
        Self::collapsed(SelectionPosition::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_range_has_equal_anchor_and_caret() {
        let pos = SelectionPosition::new(3, 5);
        let range = SelectionRange::collapsed(pos);
        assert_eq!(range.anchor, range.caret);
        assert!(range.is_collapsed());
    }

    #[test]
    fn non_collapsed_range_is_not_collapsed() {
        let range =
            SelectionRange::new(SelectionPosition::new(1, 0), SelectionPosition::new(1, 10));
        assert!(!range.is_collapsed());
    }

    #[test]
    fn start_returns_lesser_position_for_forward_selection() {
        let anchor = SelectionPosition::new(1, 5);
        let caret = SelectionPosition::new(1, 15);
        let range = SelectionRange::new(anchor, caret);
        assert_eq!(range.start(), anchor);
        assert_eq!(range.end(), caret);
    }

    #[test]
    fn start_returns_lesser_position_for_backward_selection() {
        let anchor = SelectionPosition::new(2, 10);
        let caret = SelectionPosition::new(1, 5);
        let range = SelectionRange::new(anchor, caret);
        assert_eq!(range.start(), caret);
        assert_eq!(range.end(), anchor);
    }

    #[test]
    fn contains_returns_true_for_position_within_range() {
        let range =
            SelectionRange::new(SelectionPosition::new(1, 5), SelectionPosition::new(1, 15));
        let inside = SelectionPosition::new(1, 10);
        assert!(range.contains(&inside));
    }

    #[test]
    fn contains_returns_false_for_position_at_end() {
        let range =
            SelectionRange::new(SelectionPosition::new(1, 5), SelectionPosition::new(1, 15));
        let at_end = SelectionPosition::new(1, 15);
        assert!(!range.contains(&at_end));
    }

    #[test]
    fn contains_returns_true_for_position_at_start() {
        let range =
            SelectionRange::new(SelectionPosition::new(1, 5), SelectionPosition::new(1, 15));
        let at_start = SelectionPosition::new(1, 5);
        assert!(range.contains(&at_start));
    }

    #[test]
    fn contains_returns_false_for_position_outside_range() {
        let range =
            SelectionRange::new(SelectionPosition::new(1, 5), SelectionPosition::new(1, 15));
        let outside = SelectionPosition::new(1, 20);
        assert!(!range.contains(&outside));
    }

    #[test]
    fn collapsed_range_contains_nothing() {
        let range = SelectionRange::collapsed(SelectionPosition::new(1, 5));
        let same_pos = SelectionPosition::new(1, 5);
        assert!(!range.contains(&same_pos));
    }

    #[test]
    fn overlapping_ranges_detected() {
        let a = SelectionRange::new(SelectionPosition::new(1, 0), SelectionPosition::new(1, 10));
        let b = SelectionRange::new(SelectionPosition::new(1, 5), SelectionPosition::new(1, 15));
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn non_overlapping_ranges_not_detected() {
        let a = SelectionRange::new(SelectionPosition::new(1, 0), SelectionPosition::new(1, 5));
        let b = SelectionRange::new(SelectionPosition::new(1, 10), SelectionPosition::new(1, 15));
        assert!(!a.overlaps(&b));
        assert!(!b.overlaps(&a));
    }

    #[test]
    fn adjacent_ranges_do_not_overlap() {
        let a = SelectionRange::new(SelectionPosition::new(1, 0), SelectionPosition::new(1, 5));
        let b = SelectionRange::new(SelectionPosition::new(1, 5), SelectionPosition::new(1, 10));
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn collapsed_ranges_at_same_position_overlap() {
        let a = SelectionRange::collapsed(SelectionPosition::new(1, 5));
        let b = SelectionRange::collapsed(SelectionPosition::new(1, 5));
        assert!(a.overlaps(&b));
    }

    #[test]
    fn collapsed_ranges_at_different_positions_do_not_overlap() {
        let a = SelectionRange::collapsed(SelectionPosition::new(1, 5));
        let b = SelectionRange::collapsed(SelectionPosition::new(1, 10));
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn merge_produces_union_of_overlapping_ranges() {
        let a = SelectionRange::new(SelectionPosition::new(1, 0), SelectionPosition::new(1, 10));
        let b = SelectionRange::new(SelectionPosition::new(1, 5), SelectionPosition::new(1, 20));
        let merged = a.merge(&b);
        assert_eq!(merged.anchor, SelectionPosition::new(1, 0));
        assert_eq!(merged.caret, SelectionPosition::new(1, 20));
    }

    #[test]
    fn merge_works_for_non_overlapping_ranges() {
        let a = SelectionRange::new(SelectionPosition::new(1, 0), SelectionPosition::new(1, 5));
        let b = SelectionRange::new(SelectionPosition::new(1, 10), SelectionPosition::new(1, 15));
        let merged = a.merge(&b);
        assert_eq!(merged.anchor, SelectionPosition::new(1, 0));
        assert_eq!(merged.caret, SelectionPosition::new(1, 15));
    }

    #[test]
    fn default_range_is_collapsed_at_document_start() {
        let range = SelectionRange::default();
        assert!(range.is_collapsed());
        assert_eq!(range.caret, SelectionPosition::default());
    }
}
