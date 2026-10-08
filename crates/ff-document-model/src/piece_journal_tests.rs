//! Tests for the piece-splice undo journal.

use super::*;
use crate::piece_list::PieceList;
use crate::types::{BufRange, RecordNumber};

fn r(n: u64) -> RecordNumber {
    RecordNumber(n)
}

/// Apply a splice via a closure, recording the before/after on the journal.
fn record_splice<F: FnOnce(&mut PieceList)>(
    journal: &mut PieceJournal,
    list: &mut PieceList,
    f: F,
) {
    let before = list.snapshot();
    f(list);
    let after = list.snapshot();
    journal.record(SpliceOp::new(before, after));
}

#[test]
fn inverse_swaps_before_and_after() {
    // Validates: Requirement 12.2
    let a = PieceList::from_original(2).snapshot();
    let b = PieceList::from_original(3).snapshot();
    let op = SpliceOp::new(a.clone(), b.clone());
    let inv = op.inverse();
    assert_eq!(inv, SpliceOp::new(b, a));
}

#[test]
fn undo_restores_prior_arrangement() {
    // Validates: Requirement 12.2
    let mut list = PieceList::from_original(5);
    let original = list.snapshot();
    let mut journal = PieceJournal::new();

    record_splice(&mut journal, &mut list, |l| {
        l.insert_edited_record(r(2), BufRange::new(0, 3));
    });
    assert_eq!(list.total_records(), 6);
    assert!(journal.can_undo());

    assert!(journal.undo(&mut list));
    assert_eq!(list.snapshot(), original);
    assert_eq!(list.total_records(), 5);
}

#[test]
fn redo_reapplies() {
    // Validates: Requirement 12.2
    let mut list = PieceList::from_original(4);
    let mut journal = PieceJournal::new();
    record_splice(&mut journal, &mut list, |l| l.delete_records(r(1), 1));
    let after_delete = list.snapshot();

    journal.undo(&mut list);
    assert_eq!(list.total_records(), 4);
    assert!(journal.can_redo());

    journal.redo(&mut list);
    assert_eq!(list.snapshot(), after_delete);
    assert_eq!(list.total_records(), 3);
}

#[test]
fn arbitrary_sequence_round_trips_to_original() {
    // Validates: Requirement 12.12
    let mut list = PieceList::from_original(8);
    let original = list.snapshot();
    let mut journal = PieceJournal::new();

    record_splice(&mut journal, &mut list, |l| {
        l.insert_edited_record(r(2), BufRange::new(0, 2))
    });
    record_splice(&mut journal, &mut list, |l| l.delete_records(r(0), 1));
    record_splice(&mut journal, &mut list, |l| {
        l.overtype_record(r(3), BufRange::new(2, 4))
    });
    record_splice(&mut journal, &mut list, |l| l.copy_records(r(1), 2, r(5)));

    // Undo everything.
    while journal.can_undo() {
        journal.undo(&mut list);
    }
    assert_eq!(list.snapshot(), original);
}

#[test]
fn save_point_and_drop_to_current() {
    // Validates: Requirement 12.12
    let mut list = PieceList::from_original(3);
    let mut journal = PieceJournal::new();
    assert!(journal.is_at_save_point());

    record_splice(&mut journal, &mut list, |l| {
        l.insert_edited_record(r(1), BufRange::new(0, 2))
    });
    assert!(!journal.is_at_save_point());

    // Simulate SAVE = re-baseline: drop undo to this point.
    journal.drop_to_current();
    assert!(journal.is_at_save_point());
    assert!(!journal.can_undo());
    assert!(!journal.can_redo());
}

#[test]
fn new_edit_clears_redo() {
    // Validates: Requirement 12.2
    let mut list = PieceList::from_original(4);
    let mut journal = PieceJournal::new();
    record_splice(&mut journal, &mut list, |l| l.delete_records(r(0), 1));
    journal.undo(&mut list);
    assert!(journal.can_redo());
    record_splice(&mut journal, &mut list, |l| {
        l.insert_edited_record(r(0), BufRange::new(0, 1))
    });
    assert!(!journal.can_redo());
}
