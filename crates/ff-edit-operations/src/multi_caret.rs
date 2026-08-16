//! Multi-caret coordination — simultaneous editing at multiple positions.
//!
//! The `MultiCaretCoordinator` dispatches edits across multiple carets
//! in reverse document order to avoid position drift.

use crate::position::SelectionPosition;
use crate::range::SelectionRange;
use crate::selection::{DocumentModification, SelectionContainer};

/// Result of a single edit at one caret position.
///
/// Used by `MultiCaretCoordinator` to track what happened at each caret
/// so that subsequent positions can be adjusted.
#[derive(Debug, Clone)]
pub struct SingleEditResult {
    /// The document modification descriptor for position adjustment.
    pub modification: DocumentModification,
    /// New caret position after the edit.
    pub new_caret: SelectionPosition,
}

/// Coordinates simultaneous edits across multiple carets.
///
/// Processes carets in reverse document order (last-to-first) so that
/// earlier insertions do not shift positions for later ones. This is
/// critical for correctness in multi-caret editing.
pub struct MultiCaretCoordinator;

impl MultiCaretCoordinator {
    /// Adds a new caret at the given position.
    ///
    /// The caret is added as a collapsed selection range.
    pub fn add_caret(container: &mut SelectionContainer, position: SelectionPosition) {
        container.add(SelectionRange::collapsed(position));
    }

    /// Removes a caret at the given position, if one exists.
    ///
    /// Returns true if a caret was found and removed, false otherwise.
    /// Will not remove the last remaining caret.
    pub fn remove_caret_at(
        container: &mut SelectionContainer,
        position: SelectionPosition,
    ) -> bool {
        if container.count() <= 1 {
            return false;
        }

        let index = container
            .ranges()
            .iter()
            .position(|r| r.is_collapsed() && r.caret == position);

        if let Some(idx) = index {
            container.drop_range(idx).is_ok()
        } else {
            false
        }
    }

    /// Adds a caret one line above the main caret at the same column.
    ///
    /// Does nothing if the main caret is already on line 0.
    pub fn add_caret_above(container: &mut SelectionContainer) {
        let main = container.main_range().clone();
        if main.caret.line == 0 {
            return;
        }
        let new_pos = SelectionPosition::new(main.caret.line - 1, main.caret.column);
        container.add(SelectionRange::collapsed(new_pos));
    }

    /// Adds a caret one line below the main caret at the same column.
    pub fn add_caret_below(container: &mut SelectionContainer) {
        let main = container.main_range().clone();
        let new_pos = SelectionPosition::new(main.caret.line + 1, main.caret.column);
        container.add(SelectionRange::collapsed(new_pos));
    }

    /// Reduces to a single caret (the main range), removing all others.
    pub fn escape_to_single_caret(container: &mut SelectionContainer) {
        container.clear_to_main();
    }

    /// Returns the indices of all ranges in reverse document order.
    ///
    /// This is the correct processing order for multi-caret edits.
    pub fn reverse_order_indices(container: &SelectionContainer) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..container.count()).collect();
        indices.sort_by(|&a, &b| {
            container.ranges()[b]
                .sort_key()
                .cmp(&container.ranges()[a].sort_key())
        });
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_caret_increases_count() {
        let mut container = SelectionContainer::new();
        MultiCaretCoordinator::add_caret(&mut container, SelectionPosition::new(5, 3));
        assert_eq!(container.count(), 2);
    }

    #[test]
    fn remove_caret_at_existing_position() {
        let mut container = SelectionContainer::new();
        MultiCaretCoordinator::add_caret(&mut container, SelectionPosition::new(5, 3));
        assert_eq!(container.count(), 2);

        let removed =
            MultiCaretCoordinator::remove_caret_at(&mut container, SelectionPosition::new(5, 3));
        assert!(removed);
        assert_eq!(container.count(), 1);
    }

    #[test]
    fn remove_caret_at_nonexistent_position_returns_false() {
        let mut container = SelectionContainer::new();
        MultiCaretCoordinator::add_caret(&mut container, SelectionPosition::new(5, 3));

        let removed =
            MultiCaretCoordinator::remove_caret_at(&mut container, SelectionPosition::new(10, 0));
        assert!(!removed);
        assert_eq!(container.count(), 2);
    }

    #[test]
    fn remove_caret_does_not_remove_last_caret() {
        let mut container = SelectionContainer::new();
        let removed =
            MultiCaretCoordinator::remove_caret_at(&mut container, SelectionPosition::new(0, 0));
        assert!(!removed);
        assert_eq!(container.count(), 1);
    }

    #[test]
    fn add_caret_above_creates_caret_on_previous_line() {
        let mut container = SelectionContainer::with_range(SelectionRange::collapsed(
            SelectionPosition::new(5, 10),
        ));

        MultiCaretCoordinator::add_caret_above(&mut container);
        assert_eq!(container.count(), 2);

        let ranges = container.ranges();
        // Should have carets at line 4 and line 5
        assert!(ranges
            .iter()
            .any(|r| r.caret.line == 4 && r.caret.column == 10));
        assert!(ranges
            .iter()
            .any(|r| r.caret.line == 5 && r.caret.column == 10));
    }

    #[test]
    fn add_caret_above_noop_at_line_zero() {
        let mut container =
            SelectionContainer::with_range(SelectionRange::collapsed(SelectionPosition::new(0, 5)));

        MultiCaretCoordinator::add_caret_above(&mut container);
        assert_eq!(container.count(), 1);
    }

    #[test]
    fn add_caret_below_creates_caret_on_next_line() {
        let mut container =
            SelectionContainer::with_range(SelectionRange::collapsed(SelectionPosition::new(3, 7)));

        MultiCaretCoordinator::add_caret_below(&mut container);
        assert_eq!(container.count(), 2);

        let ranges = container.ranges();
        assert!(ranges
            .iter()
            .any(|r| r.caret.line == 4 && r.caret.column == 7));
    }

    #[test]
    fn escape_to_single_caret_reduces_to_main() {
        let mut container = SelectionContainer::new();
        MultiCaretCoordinator::add_caret(&mut container, SelectionPosition::new(5, 0));
        MultiCaretCoordinator::add_caret(&mut container, SelectionPosition::new(10, 0));
        assert_eq!(container.count(), 3);

        MultiCaretCoordinator::escape_to_single_caret(&mut container);
        assert_eq!(container.count(), 1);
    }

    #[test]
    fn reverse_order_indices_returns_last_to_first() {
        let mut container = SelectionContainer::new();
        MultiCaretCoordinator::add_caret(&mut container, SelectionPosition::new(3, 0));
        MultiCaretCoordinator::add_caret(&mut container, SelectionPosition::new(7, 0));

        let indices = MultiCaretCoordinator::reverse_order_indices(&container);
        // Should be [2, 1, 0] (line 7, line 3, line 0)
        assert_eq!(indices.len(), 3);
        assert_eq!(container.ranges()[indices[0]].caret.line, 7);
        assert_eq!(container.ranges()[indices[1]].caret.line, 3);
        assert_eq!(container.ranges()[indices[2]].caret.line, 0);
    }
}
