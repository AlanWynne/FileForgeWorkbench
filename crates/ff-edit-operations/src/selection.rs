//! Selection container — holds all active SelectionRanges.
//!
//! The `SelectionContainer` maintains ranges sorted by document position
//! with a designated main range. It supports Add, Drop, Trim, and
//! MovePositions operations as required by the selection model.

use crate::error::EditError;
use crate::position::SelectionPosition;
use crate::range::SelectionRange;

/// Descriptor for a document change, used by `SelectionContainer::move_positions`.
///
/// Describes the location and extent of a document modification so that
/// all selection positions can be adjusted to remain valid.
#[derive(Debug, Clone, Copy)]
pub struct DocumentModification {
    /// Line number where the modification occurred (0-based).
    pub line: u64,
    /// Column where the modification occurred (0-based).
    pub column: u64,
    /// Number of characters (columns) inserted at the position.
    pub inserted_length: u64,
    /// Number of characters (columns) deleted at the position.
    pub deleted_length: u64,
    /// Number of lines inserted (for line splits).
    pub lines_inserted: u64,
    /// Number of lines deleted (for line joins).
    pub lines_deleted: u64,
}

/// The top-level structure holding all active SelectionRanges.
///
/// Maintains ranges sorted by document position with a designated main range.
/// Invariant: always contains at least one range.
#[derive(Debug, Clone)]
pub struct SelectionContainer {
    ranges: Vec<SelectionRange>,
    main_index: usize,
}

impl SelectionContainer {
    /// Creates a new container with a single collapsed range at document start.
    pub fn new() -> Self {
        Self {
            ranges: vec![SelectionRange::collapsed(SelectionPosition::default())],
            main_index: 0,
        }
    }

    /// Creates a new container with the given initial range.
    pub fn with_range(range: SelectionRange) -> Self {
        Self {
            ranges: vec![range],
            main_index: 0,
        }
    }

    /// Add a new range, maintaining sorted order by document position.
    ///
    /// The new range is inserted at the correct position to maintain
    /// document order. Does not merge overlapping ranges — call `trim()`
    /// after if needed.
    pub fn add(&mut self, range: SelectionRange) {
        let sort_key = range.sort_key();
        let insert_pos = self
            .ranges
            .iter()
            .position(|r| r.sort_key() > sort_key)
            .unwrap_or(self.ranges.len());

        // Adjust main_index if insertion is before or at it
        if insert_pos <= self.main_index {
            self.main_index += 1;
        }

        self.ranges.insert(insert_pos, range);
    }

    /// Remove range at the given index.
    ///
    /// # Errors
    ///
    /// Returns `EditError::LastCaretRemoval` if this would leave zero ranges.
    pub fn drop_range(&mut self, index: usize) -> Result<(), EditError> {
        if self.ranges.len() <= 1 {
            return Err(EditError::LastCaretRemoval);
        }
        if index >= self.ranges.len() {
            return Err(EditError::LastCaretRemoval);
        }

        self.ranges.remove(index);

        // Adjust main_index
        if self.main_index >= self.ranges.len() {
            self.main_index = self.ranges.len() - 1;
        } else if index < self.main_index {
            self.main_index -= 1;
        }

        Ok(())
    }

    /// Merge overlapping or identical ranges into their union.
    ///
    /// After trim, no two ranges overlap or are identical. This is
    /// idempotent: `trim(trim(container)) == trim(container)`.
    pub fn trim(&mut self) {
        if self.ranges.len() <= 1 {
            return;
        }

        // Sort by start position
        self.ranges.sort_by_key(|a| a.sort_key());

        let mut merged: Vec<SelectionRange> = Vec::with_capacity(self.ranges.len());
        let mut current = self.ranges[0].clone();

        for range in self.ranges.iter().skip(1) {
            if current.overlaps(range) {
                current = current.merge(range);
            } else {
                merged.push(current);
                current = range.clone();
            }
        }
        merged.push(current);

        // Adjust main_index to be valid
        if self.main_index >= merged.len() {
            self.main_index = merged.len() - 1;
        }

        self.ranges = merged;
    }

    /// Adjust all positions given a document modification.
    ///
    /// Positions before the modification offset are unchanged.
    /// Positions within a deleted range collapse to the modification offset.
    /// Positions after the modification shift by (inserted - deleted).
    /// After adjustment, trim is invoked to merge newly-overlapping ranges.
    pub fn move_positions(&mut self, modification: &DocumentModification) {
        for range in &mut self.ranges {
            range.anchor = adjust_position(&range.anchor, modification);
            range.caret = adjust_position(&range.caret, modification);
        }
        self.trim();
    }

    /// Get the main (primary) selection range.
    pub fn main_range(&self) -> &SelectionRange {
        &self.ranges[self.main_index]
    }

    /// Set which range index is the main range.
    ///
    /// Clamps to valid range if index is out of bounds.
    pub fn set_main_range(&mut self, index: usize) {
        if index < self.ranges.len() {
            self.main_index = index;
        }
    }

    /// Returns all ranges in document order as a slice.
    pub fn ranges(&self) -> &[SelectionRange] {
        &self.ranges
    }

    /// Returns an iterator over all ranges in reverse document order.
    ///
    /// This is the correct order for multi-caret edit operations to avoid
    /// position drift (later positions are edited first).
    pub fn ranges_reverse(&self) -> impl Iterator<Item = &SelectionRange> {
        self.ranges.iter().rev()
    }

    /// Number of active selections/carets.
    pub fn count(&self) -> usize {
        self.ranges.len()
    }

    /// Returns true if multiple carets are active.
    pub fn is_multi_caret(&self) -> bool {
        self.ranges.len() > 1
    }

    /// Collapse to a single caret (the main range only).
    ///
    /// Removes all ranges except the main range.
    pub fn clear_to_main(&mut self) {
        let main = self.ranges[self.main_index].clone();
        self.ranges = vec![main];
        self.main_index = 0;
    }

    /// Returns the main range index.
    pub fn main_index(&self) -> usize {
        self.main_index
    }
}

impl Default for SelectionContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Adjust a single position based on a document modification.
///
/// Rules:
/// - Position on an earlier line: unchanged
/// - Position on a later line: adjust line number by (lines_inserted - lines_deleted)
/// - Position on the same line, before the modification column: unchanged
/// - Position on the same line, within the deleted range: collapse to modification point
/// - Position on the same line, after the deleted range: shift by (inserted - deleted)
fn adjust_position(
    pos: &SelectionPosition,
    modification: &DocumentModification,
) -> SelectionPosition {
    // Position is on an earlier line — completely unaffected
    if pos.line < modification.line {
        return *pos;
    }

    // Position is on a later line — adjust line number only
    if pos.line > modification.line {
        let new_line = if modification.lines_deleted > 0 {
            // Lines were deleted — check if our line was one of them
            let deletion_end_line = modification.line + modification.lines_deleted;
            if pos.line < deletion_end_line {
                // Our line was deleted — collapse to modification line
                return SelectionPosition::new(modification.line, modification.column);
            }
            pos.line - modification.lines_deleted + modification.lines_inserted
        } else {
            pos.line + modification.lines_inserted
        };
        return SelectionPosition::with_virtual_space(new_line, pos.column, pos.virtual_space);
    }

    // Same line — adjust column
    // Position is before the modification column — unchanged
    if pos.column < modification.column {
        return *pos;
    }

    // Position is within the deleted range — collapse to modification point
    if modification.deleted_length > 0
        && pos.column < modification.column + modification.deleted_length
    {
        return SelectionPosition::new(modification.line, modification.column);
    }

    // Position is at or after the deleted range — shift by net change
    let net_shift = modification.inserted_length as i64 - modification.deleted_length as i64;
    let new_column = if net_shift >= 0 {
        pos.column.saturating_add(net_shift as u64)
    } else {
        pos.column.saturating_sub((-net_shift) as u64)
    };

    SelectionPosition::with_virtual_space(pos.line, new_column, pos.virtual_space)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_container_has_single_collapsed_range_at_origin() {
        let container = SelectionContainer::new();
        assert_eq!(container.count(), 1);
        assert!(container.main_range().is_collapsed());
        assert_eq!(container.main_range().caret, SelectionPosition::default());
    }

    #[test]
    fn add_inserts_in_sorted_order() {
        let mut container = SelectionContainer::new();
        container.add(SelectionRange::collapsed(SelectionPosition::new(5, 0)));
        container.add(SelectionRange::collapsed(SelectionPosition::new(2, 0)));

        let ranges = container.ranges();
        assert_eq!(ranges.len(), 3);
        assert_eq!(ranges[0].caret.line, 0);
        assert_eq!(ranges[1].caret.line, 2);
        assert_eq!(ranges[2].caret.line, 5);
    }

    #[test]
    fn drop_range_removes_specified_range() {
        let mut container = SelectionContainer::new();
        container.add(SelectionRange::collapsed(SelectionPosition::new(5, 0)));
        assert_eq!(container.count(), 2);

        container.drop_range(0).unwrap();
        assert_eq!(container.count(), 1);
        assert_eq!(container.main_range().caret.line, 5);
    }

    #[test]
    fn drop_range_fails_on_last_range() {
        let mut container = SelectionContainer::new();
        let result = container.drop_range(0);
        assert!(result.is_err());
    }

    #[test]
    fn trim_merges_overlapping_ranges() {
        let mut container = SelectionContainer::with_range(SelectionRange::new(
            SelectionPosition::new(1, 0),
            SelectionPosition::new(1, 10),
        ));
        container.add(SelectionRange::new(
            SelectionPosition::new(1, 5),
            SelectionPosition::new(1, 20),
        ));

        container.trim();
        assert_eq!(container.count(), 1);
        assert_eq!(container.main_range().start(), SelectionPosition::new(1, 0));
        assert_eq!(container.main_range().end(), SelectionPosition::new(1, 20));
    }

    #[test]
    fn trim_is_idempotent() {
        let mut container = SelectionContainer::with_range(SelectionRange::new(
            SelectionPosition::new(1, 0),
            SelectionPosition::new(1, 10),
        ));
        container.add(SelectionRange::new(
            SelectionPosition::new(1, 5),
            SelectionPosition::new(1, 20),
        ));

        container.trim();
        let after_first_trim = container.ranges().to_vec();
        container.trim();
        assert_eq!(container.ranges().to_vec(), after_first_trim);
    }

    #[test]
    fn trim_leaves_non_overlapping_ranges_intact() {
        let mut container = SelectionContainer::with_range(SelectionRange::new(
            SelectionPosition::new(1, 0),
            SelectionPosition::new(1, 5),
        ));
        container.add(SelectionRange::new(
            SelectionPosition::new(2, 0),
            SelectionPosition::new(2, 5),
        ));

        container.trim();
        assert_eq!(container.count(), 2);
    }

    #[test]
    fn main_range_returns_designated_range() {
        let mut container = SelectionContainer::new();
        container.add(SelectionRange::collapsed(SelectionPosition::new(5, 0)));
        container.set_main_range(1);
        assert_eq!(container.main_range().caret.line, 5);
    }

    #[test]
    fn clear_to_main_reduces_to_single_range() {
        let mut container = SelectionContainer::new();
        container.add(SelectionRange::collapsed(SelectionPosition::new(5, 0)));
        container.add(SelectionRange::collapsed(SelectionPosition::new(10, 0)));
        container.set_main_range(1);

        container.clear_to_main();
        assert_eq!(container.count(), 1);
        assert_eq!(container.main_range().caret.line, 5);
    }

    #[test]
    fn is_multi_caret_reflects_count() {
        let container = SelectionContainer::new();
        assert!(!container.is_multi_caret());

        let mut multi = SelectionContainer::new();
        multi.add(SelectionRange::collapsed(SelectionPosition::new(5, 0)));
        assert!(multi.is_multi_caret());
    }

    #[test]
    fn move_positions_shifts_positions_forward_on_insertion_before() {
        let mut container = SelectionContainer::with_range(SelectionRange::collapsed(
            SelectionPosition::new(1, 10),
        ));

        let modification = DocumentModification {
            line: 1,
            column: 5,
            inserted_length: 3,
            deleted_length: 0,
            lines_inserted: 0,
            lines_deleted: 0,
        };

        container.move_positions(&modification);
        assert_eq!(container.main_range().caret.column, 13);
    }

    #[test]
    fn move_positions_collapses_position_within_deleted_range() {
        let mut container =
            SelectionContainer::with_range(SelectionRange::collapsed(SelectionPosition::new(1, 7)));

        let modification = DocumentModification {
            line: 1,
            column: 5,
            inserted_length: 0,
            deleted_length: 5,
            lines_inserted: 0,
            lines_deleted: 0,
        };

        container.move_positions(&modification);
        assert_eq!(container.main_range().caret.column, 5);
    }

    #[test]
    fn move_positions_shifts_backward_on_deletion_before() {
        let mut container = SelectionContainer::with_range(SelectionRange::collapsed(
            SelectionPosition::new(1, 15),
        ));

        let modification = DocumentModification {
            line: 1,
            column: 5,
            inserted_length: 0,
            deleted_length: 3,
            lines_inserted: 0,
            lines_deleted: 0,
        };

        container.move_positions(&modification);
        assert_eq!(container.main_range().caret.column, 12);
    }

    #[test]
    fn move_positions_does_not_affect_earlier_lines() {
        let mut container =
            SelectionContainer::with_range(SelectionRange::collapsed(SelectionPosition::new(0, 5)));

        let modification = DocumentModification {
            line: 1,
            column: 0,
            inserted_length: 10,
            deleted_length: 0,
            lines_inserted: 0,
            lines_deleted: 0,
        };

        container.move_positions(&modification);
        assert_eq!(container.main_range().caret, SelectionPosition::new(0, 5));
    }

    #[test]
    fn move_positions_adjusts_line_numbers_on_line_insertion() {
        let mut container =
            SelectionContainer::with_range(SelectionRange::collapsed(SelectionPosition::new(5, 3)));

        let modification = DocumentModification {
            line: 2,
            column: 0,
            inserted_length: 0,
            deleted_length: 0,
            lines_inserted: 2,
            lines_deleted: 0,
        };

        container.move_positions(&modification);
        assert_eq!(container.main_range().caret.line, 7);
        assert_eq!(container.main_range().caret.column, 3);
    }

    #[test]
    fn move_positions_never_produces_negative_positions() {
        let mut container =
            SelectionContainer::with_range(SelectionRange::collapsed(SelectionPosition::new(0, 0)));

        let modification = DocumentModification {
            line: 0,
            column: 0,
            inserted_length: 0,
            deleted_length: 100,
            lines_inserted: 0,
            lines_deleted: 0,
        };

        container.move_positions(&modification);
        // Should collapse to (0, 0) without underflow
        assert_eq!(container.main_range().caret, SelectionPosition::new(0, 0));
    }

    #[test]
    fn count_always_at_least_one() {
        let container = SelectionContainer::new();
        assert!(container.count() >= 1);
    }

    #[test]
    fn ranges_reverse_yields_in_reverse_document_order() {
        let mut container = SelectionContainer::new();
        container.add(SelectionRange::collapsed(SelectionPosition::new(3, 0)));
        container.add(SelectionRange::collapsed(SelectionPosition::new(7, 0)));

        let reversed: Vec<_> = container.ranges_reverse().collect();
        assert_eq!(reversed[0].caret.line, 7);
        assert_eq!(reversed[1].caret.line, 3);
        assert_eq!(reversed[2].caret.line, 0);
    }
}
