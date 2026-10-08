//! Property-based tests for the piece-table spine (CR-CH-058 F1, plan item 14).
//!
//! These exercise random splice sequences against the `PieceList` + `PieceJournal`
//! and assert the F1 invariants: splice/undo round-trips return to the exact
//! original arrangement, the running-record-count invariant holds after every
//! op, and record addressing is stable across intervening edits. Minimum 100
//! iterations per property (proptest default). The estimated->exact
//! Total_Records monotonicity property is intentionally OMITTED -- windowing /
//! background scan is F2, not built in F1.

use proptest::prelude::*;

use crate::piece_journal::{PieceJournal, SpliceOp};
use crate::piece_list::PieceList;
use crate::types::{BufRange, RecordNumber};

/// A scripted splice operation over a PieceList, parameterised so proptest can
/// generate arbitrary sequences. Positions are taken modulo the current record
/// count so every op is in range for whatever the list currently holds.
#[derive(Debug, Clone)]
enum Op {
    Insert { at: u64 },
    Delete { at: u64, count: u64 },
    Overtype { at: u64 },
    Move { from: u64, count: u64, to: u64 },
    Copy { from: u64, count: u64, to: u64 },
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0u64..1000).prop_map(|at| Op::Insert { at }),
        (0u64..1000, 1u64..5).prop_map(|(at, count)| Op::Delete { at, count }),
        (0u64..1000).prop_map(|at| Op::Overtype { at }),
        (0u64..1000, 1u64..4, 0u64..1000).prop_map(|(from, count, to)| Op::Move {
            from,
            count,
            to
        }),
        (0u64..1000, 1u64..4, 0u64..1000).prop_map(|(from, count, to)| Op::Copy {
            from,
            count,
            to
        }),
    ]
}

/// Apply `op` to `list`, clamping generated positions/counts into range for the
/// list's CURRENT record count. Returns the snapshot taken BEFORE the op and the
/// one AFTER, so the caller can record a journal `SpliceOp`. Ops that would be
/// no-ops in range (e.g. delete on an empty list) are applied as clamped.
fn apply(
    list: &mut PieceList,
    op: &Op,
    append_cursor: &mut u64,
) -> (Vec<crate::piece_list::Piece>, Vec<crate::piece_list::Piece>) {
    let before = list.snapshot();
    let total = list.total_records();
    match *op {
        Op::Insert { at } => {
            let at = if total == 0 { 0 } else { at % (total + 1) };
            let r = BufRange::new(*append_cursor, 1);
            *append_cursor += 1;
            list.insert_edited_record(RecordNumber(at), r);
        }
        Op::Delete { at, count } => {
            if total > 0 {
                let at = at % total;
                let count = count.min(total - at);
                list.delete_records(RecordNumber(at), count);
            }
        }
        Op::Overtype { at } => {
            if total > 0 {
                let at = at % total;
                let r = BufRange::new(*append_cursor, 1);
                *append_cursor += 1;
                list.overtype_record(RecordNumber(at), r);
            }
        }
        Op::Move { from, count, to } => {
            if total > 0 {
                let from = from % total;
                let count = count.min(total - from);
                // After removal there are (total - count) records; dest in range.
                let remaining = total - count;
                let to = if remaining == 0 {
                    0
                } else {
                    to % (remaining + 1)
                };
                list.move_records(RecordNumber(from), count, RecordNumber(to));
            }
        }
        Op::Copy { from, count, to } => {
            if total > 0 {
                let from = from % total;
                let count = count.min(total - from);
                let to = to % (total + 1);
                list.copy_records(RecordNumber(from), count, RecordNumber(to));
            }
        }
    }
    let after = list.snapshot();
    (before, after)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Feature: document-model, Property 1: splice/undo round-trip.
    /// For any sequence of splices recorded in the journal, undoing them all
    /// restores the EXACT original piece arrangement and total record count.
    /// Validates: Requirement 12.2, 12.12
    #[test]
    fn splice_undo_round_trip_restores_original(
        start in 1u64..40,
        ops in prop::collection::vec(op_strategy(), 0..30),
    ) {
        let mut list = PieceList::from_original(start);
        let original = list.snapshot();
        let original_total = list.total_records();
        let mut journal = PieceJournal::new();
        let mut cursor = 0u64;

        for op in &ops {
            let (before, after) = apply(&mut list, op, &mut cursor);
            journal.record(SpliceOp::new(before, after));
        }

        // Undo everything.
        while journal.can_undo() {
            journal.undo(&mut list);
        }

        prop_assert_eq!(list.snapshot(), original);
        prop_assert_eq!(list.total_records(), original_total);
    }

    /// Feature: document-model, Property 2: running-count invariant.
    /// After every splice the sum of piece counts equals total_records(), and
    /// total_records() never underflows (stays a valid count).
    /// Validates: Requirement 12.1, 12.2
    #[test]
    fn running_count_invariant_holds_after_every_op(
        start in 0u64..40,
        ops in prop::collection::vec(op_strategy(), 0..40),
    ) {
        let mut list = PieceList::from_original(start);
        let mut cursor = 0u64;
        for op in &ops {
            let _ = apply(&mut list, op, &mut cursor);
            let sum: u64 = list.pieces().iter().map(crate::piece_list::Piece::count).sum();
            prop_assert_eq!(sum, list.total_records());
        }
    }

    /// Feature: document-model, Property 3: redo re-applies.
    /// Undoing then redoing a recorded sequence returns to the post-sequence
    /// arrangement (symmetry of the journal).
    /// Validates: Requirement 12.2
    #[test]
    fn undo_then_redo_restores_edited_state(
        start in 1u64..40,
        ops in prop::collection::vec(op_strategy(), 1..25),
    ) {
        let mut list = PieceList::from_original(start);
        let mut journal = PieceJournal::new();
        let mut cursor = 0u64;
        for op in &ops {
            let (before, after) = apply(&mut list, op, &mut cursor);
            journal.record(SpliceOp::new(before, after));
        }
        let edited = list.snapshot();

        while journal.can_undo() {
            journal.undo(&mut list);
        }
        while journal.can_redo() {
            journal.redo(&mut list);
        }

        prop_assert_eq!(list.snapshot(), edited);
    }

    /// Feature: document-model, Property 4: record addressing resolves in range.
    /// For a list after an arbitrary splice sequence, every record index in
    /// [0, total) resolves to some piece, and total resolves to None (the
    /// one-past-the-end boundary) -- i.e. piece_at_record is total-consistent.
    /// Validates: Requirement 12.1, 12.9
    #[test]
    fn record_addressing_is_total_consistent(
        start in 0u64..40,
        ops in prop::collection::vec(op_strategy(), 0..25),
    ) {
        let mut list = PieceList::from_original(start);
        let mut cursor = 0u64;
        for op in &ops {
            let _ = apply(&mut list, op, &mut cursor);
        }
        let total = list.total_records();
        for n in 0..total {
            prop_assert!(list.piece_at_record(RecordNumber(n)).is_some(),
                "record {} must resolve to a piece (total {})", n, total);
        }
        prop_assert!(list.piece_at_record(RecordNumber(total)).is_none(),
            "one-past-the-end must not resolve");
    }
}
