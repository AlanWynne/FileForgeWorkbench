//! Tests for the Piece_List construction, resolution, and splices.

use super::*;
use crate::types::{BufRange, RecordNumber};

fn r(n: u64) -> RecordNumber {
    RecordNumber(n)
}

#[test]
fn from_original_single_piece() {
    // Validates: Requirement 12.1
    let pl = PieceList::from_original(10);
    assert_eq!(pl.pieces().len(), 1);
    assert_eq!(pl.total_records(), 10);
    assert_eq!(
        pl.pieces()[0],
        Piece::Original {
            first_record: 0,
            count: 10,
            dirty: false
        }
    );
    assert!(!pl.has_dirty());
}

#[test]
fn empty_document_has_no_pieces() {
    // Validates: Requirement 4.8
    let pl = PieceList::from_original(0);
    assert_eq!(pl.pieces().len(), 0);
    assert_eq!(pl.total_records(), 0);
}

#[test]
fn piece_at_record_linear_scan() {
    // Validates: Requirement 12.9
    let pl = PieceList::from_original(5);
    assert_eq!(pl.piece_at_record(r(0)), Some((0, 0)));
    assert_eq!(pl.piece_at_record(r(4)), Some((0, 4)));
    assert_eq!(pl.piece_at_record(r(5)), None);
}

#[test]
fn insert_splits_original_into_three() {
    // Validates: Requirement 12.2
    let mut pl = PieceList::from_original(5);
    // Insert one edited record at record 2.
    pl.insert_edited_record(r(2), BufRange::new(0, 4));
    // Expect: Original{0,2}, Edited{..,1}, Original{2,3}
    assert_eq!(pl.total_records(), 6);
    assert_eq!(pl.pieces().len(), 3);
    assert_eq!(
        pl.pieces()[0],
        Piece::Original {
            first_record: 0,
            count: 2,
            dirty: false
        }
    );
    assert!(matches!(pl.pieces()[1], Piece::Edited { count: 1, .. }));
    assert_eq!(
        pl.pieces()[2],
        Piece::Original {
            first_record: 2,
            count: 3,
            dirty: false
        }
    );
    assert!(pl.has_dirty());
}

#[test]
fn insert_at_start_and_end() {
    // Validates: Requirement 12.2
    let mut pl = PieceList::from_original(3);
    pl.insert_edited_record(r(0), BufRange::new(0, 2));
    pl.insert_edited_record(r(4), BufRange::new(2, 2));
    assert_eq!(pl.total_records(), 5);
    assert!(matches!(pl.pieces().first(), Some(Piece::Edited { .. })));
    assert!(matches!(pl.pieces().last(), Some(Piece::Edited { .. })));
}

#[test]
fn delete_trims_and_splits_pieces() {
    // Validates: Requirement 12.2
    let mut pl = PieceList::from_original(5);
    pl.delete_records(r(1), 2); // remove records 1,2
    assert_eq!(pl.total_records(), 3);
    // Original{0,1}, Original{3,2}
    assert_eq!(
        pl.pieces()[0],
        Piece::Original {
            first_record: 0,
            count: 1,
            dirty: false
        }
    );
    assert_eq!(
        pl.pieces()[1],
        Piece::Original {
            first_record: 3,
            count: 2,
            dirty: false
        }
    );
}

#[test]
fn delete_whole_edited_piece() {
    // Validates: Requirement 12.2
    let mut pl = PieceList::from_original(3);
    pl.insert_edited_record(r(1), BufRange::new(0, 2));
    assert_eq!(pl.total_records(), 4);
    pl.delete_records(r(1), 1); // remove the edited record
    assert_eq!(pl.total_records(), 3);
    assert!(pl.pieces().iter().all(|p| matches!(p, Piece::Original { .. })));
}

#[test]
fn overtype_replaces_single_record() {
    // Validates: Requirement 12.2
    let mut pl = PieceList::from_original(4);
    pl.overtype_record(r(2), BufRange::new(0, 5));
    assert_eq!(pl.total_records(), 4);
    let (idx, local) = pl.piece_at_record(r(2)).unwrap();
    assert_eq!(local, 0);
    assert!(matches!(pl.pieces()[idx], Piece::Edited { count: 1, .. }));
}

#[test]
fn move_reorders_records() {
    // Validates: Requirement 12.2
    let mut pl = PieceList::from_original(5);
    // Move record 0 to position 2 (after removal, insert at record 2).
    pl.move_records(r(0), 1, r(2));
    assert_eq!(pl.total_records(), 5);
    // The moved original record is now dirty.
    assert!(pl.has_dirty());
}

#[test]
fn copy_adds_piece_referencing_existing_bytes() {
    // Validates: Requirement 12.2
    let mut pl = PieceList::from_original(3);
    pl.copy_records(r(0), 1, r(3)); // copy record 0 to the end
    assert_eq!(pl.total_records(), 4);
    assert_eq!(pl.pieces().len(), 2);
    // Last piece references original record 0.
    assert_eq!(
        pl.pieces()[1],
        Piece::Original {
            first_record: 0,
            count: 1,
            dirty: true
        }
    );
}

#[test]
fn running_count_invariant_after_sequence() {
    // Validates: Requirement 12.2
    let mut pl = PieceList::from_original(10);
    pl.insert_edited_record(r(3), BufRange::new(0, 2));
    assert_eq!(pl.total_records(), pl.pieces().iter().map(Piece::count).sum());
    pl.delete_records(r(0), 2);
    assert_eq!(pl.total_records(), pl.pieces().iter().map(Piece::count).sum());
    pl.overtype_record(r(1), BufRange::new(2, 3));
    assert_eq!(pl.total_records(), pl.pieces().iter().map(Piece::count).sum());
    pl.move_records(r(0), 1, r(3));
    assert_eq!(pl.total_records(), pl.pieces().iter().map(Piece::count).sum());
    pl.copy_records(r(0), 2, r(4));
    assert_eq!(pl.total_records(), pl.pieces().iter().map(Piece::count).sum());
}

#[test]
fn collapse_to_original_resets_list() {
    // Validates: Requirement 12.12 (re-baseline)
    let mut pl = PieceList::from_original(3);
    pl.insert_edited_record(r(1), BufRange::new(0, 2));
    pl.collapse_to_original(4);
    assert_eq!(pl.pieces().len(), 1);
    assert!(!pl.has_dirty());
    assert_eq!(pl.total_records(), 4);
}
