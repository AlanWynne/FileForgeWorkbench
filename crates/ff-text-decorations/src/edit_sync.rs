//! Edit synchronization.
//!
//! Receives edit events (insert/delete) and propagates position adjustments
//! to all active decorations and line markers.

use crate::decoration_list::DecorationList;
use crate::marker_store::MarkerStore;

/// Edit synchronization module.
///
/// Coordinates decoration position adjustments in response to document edits.
///
/// Addresses: Requirement 4 AC 1–8
pub struct EditSync;

impl EditSync {
    /// Handle a text insertion at position P with length L.
    ///
    /// Propagates insert_space to decorations and lines_inserted to markers.
    pub fn handle_insert(
        decorations: &mut DecorationList,
        markers: &mut MarkerStore,
        position: u64,
        length: u64,
        lines_added: u64,
        line_of_insert: u64,
    ) {
        decorations.insert_space(position, length);
        if lines_added > 0 {
            markers.lines_inserted(line_of_insert + 1, lines_added);
        }
    }

    /// Handle a text deletion at position P with length L.
    ///
    /// Propagates delete_range to decorations and lines_deleted to markers.
    pub fn handle_delete(
        decorations: &mut DecorationList,
        markers: &mut MarkerStore,
        position: u64,
        length: u64,
        lines_removed: u64,
        line_of_delete: u64,
    ) {
        decorations.delete_range(position, length);
        if lines_removed > 0 {
            markers.lines_deleted(line_of_delete + 1, lines_removed);
        }
    }

    /// Handle undo of an insertion (equivalent to delete_range).
    ///
    /// Addresses: Requirement 4 AC 5
    pub fn handle_undo_insert(
        decorations: &mut DecorationList,
        markers: &mut MarkerStore,
        position: u64,
        length: u64,
        lines_removed: u64,
        line_of_delete: u64,
    ) {
        Self::handle_delete(
            decorations,
            markers,
            position,
            length,
            lines_removed,
            line_of_delete,
        );
    }

    /// Handle undo of a deletion (equivalent to insert_space).
    ///
    /// Addresses: Requirement 4 AC 6
    pub fn handle_undo_delete(
        decorations: &mut DecorationList,
        markers: &mut MarkerStore,
        position: u64,
        length: u64,
        lines_added: u64,
        line_of_insert: u64,
    ) {
        Self::handle_insert(
            decorations,
            markers,
            position,
            length,
            lines_added,
            line_of_insert,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IndicatorNumber, MarkerNumber};

    #[test]
    fn handle_insert_propagates_to_decorations() {
        // Validates: Requirement 4 AC 1
        let mut dl = DecorationList::new(100);
        let mut ms = MarkerStore::new(10);
        dl.fill_range(IndicatorNumber(5), 10, 1, 10);
        EditSync::handle_insert(&mut dl, &mut ms, 15, 5, 0, 1);
        assert_eq!(dl.document_length(), 105);
        assert_eq!(dl.value_at(IndicatorNumber(5), 15), 0);
    }

    #[test]
    fn handle_delete_propagates_to_decorations() {
        // Validates: Requirement 4 AC 2
        let mut dl = DecorationList::new(100);
        let mut ms = MarkerStore::new(10);
        dl.fill_range(IndicatorNumber(5), 10, 1, 20);
        EditSync::handle_delete(&mut dl, &mut ms, 15, 5, 0, 1);
        assert_eq!(dl.document_length(), 95);
    }

    #[test]
    fn handle_insert_shifts_markers_on_new_lines() {
        let mut dl = DecorationList::new(100);
        let mut ms = MarkerStore::new(10);
        let marker = MarkerNumber::new(0).unwrap();
        ms.marker_add(5, marker);
        EditSync::handle_insert(&mut dl, &mut ms, 50, 10, 2, 3);
        // Marker on line 5 should shift to line 7 (was >= line 4 = line_of_insert+1)
        assert!(ms.marker_get(7).has(marker));
    }

    #[test]
    fn undo_insert_is_equivalent_to_delete() {
        // Validates: Requirement 4 AC 5
        let mut dl = DecorationList::new(100);
        let mut ms = MarkerStore::new(10);
        dl.fill_range(IndicatorNumber(5), 10, 1, 10);
        let before = dl.value_at(IndicatorNumber(5), 10);
        dl.insert_space(15, 5);
        EditSync::handle_undo_insert(&mut dl, &mut ms, 15, 5, 0, 1);
        assert_eq!(dl.value_at(IndicatorNumber(5), 10), before);
        assert_eq!(dl.document_length(), 100);
    }

    #[test]
    fn undo_delete_is_equivalent_to_insert() {
        // Validates: Requirement 4 AC 6
        let mut dl = DecorationList::new(100);
        let mut ms = MarkerStore::new(10);
        dl.fill_range(IndicatorNumber(5), 10, 1, 20);
        dl.delete_range(15, 5);
        EditSync::handle_undo_delete(&mut dl, &mut ms, 15, 5, 0, 1);
        assert_eq!(dl.document_length(), 100);
    }
}
