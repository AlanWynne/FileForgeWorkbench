//! Piece_List: the small ordered list of pieces presenting the CURRENT document.
//!
//! Each piece is either `Original { first_record, count }` (bytes resolve via
//! the Immutable_Original_Index) or `Edited { buf_range, count }` (bytes in the
//! Append_Buffer), and carries a running record-count. Every edit is a SPLICE
//! over this list (never an O(file) byte shift, AC 12.2); unedited spans stay a
//! single Original piece. "Current record N -> piece" is a LINEAR scan over
//! pieces (AC 12.9).

use crate::types::{BufRange, RecordNumber};

/// A single piece in the Piece_List.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    /// A run of original records resolved via the Immutable_Original_Index.
    Original {
        /// Original record number of the first record in this run.
        first_record: u64,
        /// Number of consecutive original records in this run.
        count: u64,
        /// Dirty flag: a splice touched this piece (tracked for F2 pinning).
        dirty: bool,
    },
    /// A run of edited/inserted records whose bytes live in the Append_Buffer.
    Edited {
        /// Byte range of this piece's record bytes within the Append_Buffer.
        buf_range: BufRange,
        /// Number of records represented by this piece.
        count: u64,
        /// Dirty flag: always true for edited pieces (tracked for F2 pinning).
        dirty: bool,
    },
}

impl Piece {
    /// Number of records this piece contributes to the running count.
    pub fn count(&self) -> u64 {
        match self {
            Piece::Original { count, .. } | Piece::Edited { count, .. } => *count,
        }
    }

    /// Whether this piece is marked dirty (touched by a splice / edited).
    pub fn is_dirty(&self) -> bool {
        match self {
            Piece::Original { dirty, .. } | Piece::Edited { dirty, .. } => *dirty,
        }
    }
}

/// The ordered list of pieces presenting the current document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PieceList {
    pieces: Vec<Piece>,
}

impl PieceList {
    /// Build a piece list of a single Original piece spanning `total_records`.
    pub fn from_original(total_records: u64) -> Self {
        let pieces = if total_records == 0 {
            Vec::new()
        } else {
            vec![Piece::Original {
                first_record: 0,
                count: total_records,
                dirty: false,
            }]
        };
        Self { pieces }
    }

    /// The pieces, in document order.
    pub fn pieces(&self) -> &[Piece] {
        &self.pieces
    }

    /// Total records = sum of piece counts (the running-record-count invariant).
    pub fn total_records(&self) -> u64 {
        self.pieces.iter().map(Piece::count).sum()
    }

    /// Whether any piece is dirty (an edit happened since the last re-baseline).
    pub fn has_dirty(&self) -> bool {
        self.pieces.iter().any(Piece::is_dirty)
    }

    /// Resolve current record `n` to `(piece_index, local_record)` by linear
    /// scan over pieces using running counts (AC 12.9). Returns `None` if `n`
    /// is at or beyond `total_records`.
    pub fn piece_at_record(&self, n: RecordNumber) -> Option<(usize, u64)> {
        let mut acc = 0u64;
        for (i, p) in self.pieces.iter().enumerate() {
            let c = p.count();
            if n.0 < acc + c {
                return Some((i, n.0 - acc));
            }
            acc += c;
        }
        None
    }

    /// Replace the whole list with a single Original piece (re-baseline).
    pub fn collapse_to_original(&mut self, total_records: u64) {
        *self = PieceList::from_original(total_records);
    }

    // === Splices (AC 12.2) ===================================================

    /// Split the list at current record boundary `at_record`, so that
    /// `at_record` begins a new piece. Returns the index of the piece that now
    /// starts at `at_record` (== pieces.len() when splitting at the very end).
    ///
    /// This never shifts bytes; it only subdivides pieces structurally.
    fn split_at(&mut self, at_record: u64) -> usize {
        if at_record == 0 {
            return 0;
        }
        let mut acc = 0u64;
        for i in 0..self.pieces.len() {
            let c = self.pieces[i].count();
            if at_record == acc + c {
                return i + 1;
            }
            if at_record < acc + c {
                let local = at_record - acc;
                let (left, right) = split_piece(self.pieces[i], local);
                self.pieces[i] = left;
                self.pieces.insert(i + 1, right);
                return i + 1;
            }
            acc += c;
        }
        self.pieces.len()
    }

    /// Insert a single-record Edited piece (bytes at `buf_range`) so that the
    /// inserted record begins at current record `at_record`.
    ///
    /// Edited pieces are always single-record units in F1 so that `split_at`
    /// never needs to subdivide one internally (its boundaries always align
    /// with piece boundaries), which keeps per-record byte resolution exact.
    pub fn insert_edited_record(&mut self, at_record: RecordNumber, buf_range: BufRange) {
        let idx = self.split_at(at_record.0);
        self.pieces.insert(
            idx,
            Piece::Edited {
                buf_range,
                count: 1,
                dirty: true,
            },
        );
    }

    /// Delete `count` current records starting at `at_record`. Splits boundary
    /// pieces as needed and removes the fully-covered pieces; neighbours of a
    /// deletion are marked dirty.
    pub fn delete_records(&mut self, at_record: RecordNumber, count: u64) {
        if count == 0 {
            return;
        }
        let start_idx = self.split_at(at_record.0);
        let end_idx = self.split_at(at_record.0 + count);
        if start_idx < end_idx {
            self.pieces.drain(start_idx..end_idx);
        }
    }

    /// Overtype (replace) a single current record at `at_record` with a new
    /// single-record Edited piece (bytes at `buf_range`).
    pub fn overtype_record(&mut self, at_record: RecordNumber, buf_range: BufRange) {
        self.delete_records(at_record, 1);
        self.insert_edited_record(at_record, buf_range);
    }

    /// Move `count` current records from `from_record` to `to_record` (the
    /// destination is interpreted in the list AFTER the records are removed).
    pub fn move_records(&mut self, from_record: RecordNumber, count: u64, to_record: RecordNumber) {
        if count == 0 {
            return;
        }
        let from_start = self.split_at(from_record.0);
        let from_end = self.split_at(from_record.0 + count);
        let moved: Vec<Piece> = self.pieces.drain(from_start..from_end).collect();
        let dest_idx = self.split_at(to_record.0);
        for (offset, mut p) in moved.into_iter().enumerate() {
            mark_dirty(&mut p);
            self.pieces.insert(dest_idx + offset, p);
        }
    }

    /// Copy `count` current records from `from_record`, inserting the SAME piece
    /// references at `to_record` (copy references existing bytes, AC 12.2).
    pub fn copy_records(&mut self, from_record: RecordNumber, count: u64, to_record: RecordNumber) {
        if count == 0 {
            return;
        }
        // Copy references existing bytes WITHOUT fragmenting the source: extract
        // the sub-pieces covering [from, from+count) as standalone values and
        // leave the source pieces untouched (unlike move, which removes them).
        let copied = self.extract_subpieces(from_record.0, count);
        let dest_idx = self.split_at(to_record.0);
        for (offset, mut p) in copied.into_iter().enumerate() {
            mark_dirty(&mut p);
            self.pieces.insert(dest_idx + offset, p);
        }
    }

    /// Collect the pieces covering current records `[start, start+count)` as
    /// standalone `Piece` values, sub-slicing the pieces at the range edges,
    /// WITHOUT mutating `self`. Used by copy so the source stays whole.
    fn extract_subpieces(&self, start: u64, count: u64) -> Vec<Piece> {
        let end = start + count;
        let mut out: Vec<Piece> = Vec::new();
        let mut acc = 0u64;
        for p in &self.pieces {
            let c = p.count();
            let p_start = acc;
            let p_end = acc + c;
            acc = p_end;
            // Overlap of [p_start, p_end) with [start, end).
            let lo = p_start.max(start);
            let hi = p_end.min(end);
            if lo < hi {
                let local_off = lo - p_start;
                let local_len = hi - lo;
                out.push(subpiece(*p, local_off, local_len));
            }
            if p_end >= end {
                break;
            }
        }
        out
    }

    /// Replace the entire piece vector (used by the undo journal to restore an
    /// exact prior arrangement).
    pub(crate) fn set_pieces(&mut self, pieces: Vec<Piece>) {
        self.pieces = pieces;
    }

    /// Snapshot the current piece vector (used by the undo journal to build a
    /// `SpliceOp`'s before/after arrangements).
    //
    // allow(dead_code): the piece-splice journal is driven from tests in F1
    // (the undo mechanism is proven); the F2 windowed edit path is its non-test
    // consumer. Remove the allow when F2 records real edits through the journal.
    #[allow(dead_code)]
    pub(crate) fn snapshot(&self) -> Vec<Piece> {
        self.pieces.clone()
    }
}

/// Split a piece into `(left, right)` at `local` records from its start.
fn split_piece(piece: Piece, local: u64) -> (Piece, Piece) {
    match piece {
        Piece::Original {
            first_record,
            count,
            dirty,
        } => (
            Piece::Original {
                first_record,
                count: local,
                dirty,
            },
            Piece::Original {
                first_record: first_record + local,
                count: count - local,
                dirty,
            },
        ),
        Piece::Edited { .. } => {
            // Edited pieces are always single-record in F1, so split_at never
            // requests an internal split of one (local would be 0, handled by
            // the early-return boundary case). Keeping the piece whole is the
            // only correct behaviour since an Edited piece's buf_range cannot be
            // subdivided without per-record byte lengths.
            debug_assert!(
                local == 0 || local == piece.count(),
                "Edited pieces must not be split internally"
            );
            (piece, piece)
        }
    }
}

fn mark_dirty(piece: &mut Piece) {
    match piece {
        Piece::Original { dirty, .. } | Piece::Edited { dirty, .. } => *dirty = true,
    }
}

/// Return the sub-slice of `piece` starting `local_off` records in, `local_len`
/// records long (used by copy's non-mutating extraction). For an Original piece
/// this shifts `first_record` and narrows `count`; an Edited piece is a single
/// record in F1 so the only valid sub-slice is the whole piece.
fn subpiece(piece: Piece, local_off: u64, local_len: u64) -> Piece {
    match piece {
        Piece::Original {
            first_record,
            dirty,
            ..
        } => Piece::Original {
            first_record: first_record + local_off,
            count: local_len,
            dirty,
        },
        Piece::Edited { .. } => {
            debug_assert!(
                local_off == 0 && local_len == piece.count(),
                "Edited pieces are single-record in F1; cannot sub-slice"
            );
            piece
        }
    }
}

#[cfg(test)]
#[path = "piece_list_tests.rs"]
mod tests;
