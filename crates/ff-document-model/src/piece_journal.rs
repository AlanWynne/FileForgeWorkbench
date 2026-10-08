//! Piece-splice undo journal with `inverse()`.
//!
//! F1 ships this self-contained undo mechanism INSIDE `ff-document-model`
//! (decision 4 of the plan). The journal records each Piece_List splice as a
//! `SpliceOp` carrying the before/after piece arrangements; `inverse()` swaps
//! them, so undo restores the exact prior arrangement and redo re-applies.
//!
//! DEFERRAL (flagged): migrating `ff-undo-redo`'s byte-position `EditOperation`
//! to record/piece addressing is explicitly post-F1. The byte-identical native
//! round-trip (AC 12.12) is driven by this document-model journal, not by
//! `ff-undo-redo`, which is left untouched so its tests stay green.

use crate::piece_list::{Piece, PieceList};

/// A single reversible Piece_List splice.
///
/// It captures the piece arrangement before and after the splice. `inverse()`
/// swaps them, which is the exact undo of any splice (insert/delete/move/copy/
/// overtype) over the small piece list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpliceOp {
    before: Vec<Piece>,
    after: Vec<Piece>,
}

impl SpliceOp {
    /// Build a splice op from the piece vectors before and after the edit.
    pub fn new(before: Vec<Piece>, after: Vec<Piece>) -> Self {
        Self { before, after }
    }

    /// The inverse splice (restores `before` from `after`).
    pub fn inverse(&self) -> SpliceOp {
        SpliceOp {
            before: self.after.clone(),
            after: self.before.clone(),
        }
    }

    /// Apply this op's `after` arrangement to the piece list.
    fn apply_forward(&self, list: &mut PieceList) {
        list.set_pieces(self.after.clone());
    }

    /// Apply this op's `before` arrangement to the piece list (undo).
    fn apply_backward(&self, list: &mut PieceList) {
        list.set_pieces(self.before.clone());
    }
}

/// The undo/redo journal of piece-list splices, with a save-point marker.
#[derive(Debug, Clone, Default)]
pub struct PieceJournal {
    undo_stack: Vec<SpliceOp>,
    redo_stack: Vec<SpliceOp>,
    /// Depth of the undo stack at the last save point (re-baseline point).
    save_point_depth: Option<usize>,
}

impl PieceJournal {
    /// Create an empty journal with the save point at depth 0 (fresh document).
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            save_point_depth: Some(0),
        }
    }

    /// Record a splice op. Clears the redo stack (new edit invalidates redo).
    pub fn record(&mut self, op: SpliceOp) {
        self.undo_stack.push(op);
        self.redo_stack.clear();
        // If the save point was ahead of the current depth (only reachable via
        // redo), a fresh edit invalidates it.
        if let Some(d) = self.save_point_depth {
            if d > self.undo_stack.len() {
                self.save_point_depth = None;
            }
        }
    }

    /// Whether an undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether a redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Undo the most recent splice, applying its `before` arrangement.
    pub fn undo(&mut self, list: &mut PieceList) -> bool {
        if let Some(op) = self.undo_stack.pop() {
            op.apply_backward(list);
            self.redo_stack.push(op);
            true
        } else {
            false
        }
    }

    /// Redo the most recently undone splice, applying its `after` arrangement.
    pub fn redo(&mut self, list: &mut PieceList) -> bool {
        if let Some(op) = self.redo_stack.pop() {
            op.apply_forward(list);
            self.undo_stack.push(op);
            true
        } else {
            false
        }
    }

    /// Mark the current state as the save point.
    pub fn set_save_point(&mut self) {
        self.save_point_depth = Some(self.undo_stack.len());
    }

    /// Whether the current undo depth matches the save point.
    pub fn is_at_save_point(&self) -> bool {
        self.save_point_depth == Some(self.undo_stack.len())
    }

    /// Drop all undo/redo history to the current point (re-baseline on SAVE).
    /// After this, `can_undo` is false and the current state is the save point.
    pub fn drop_to_current(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.save_point_depth = Some(0);
    }

    /// Current undo depth (number of recorded splices not yet undone).
    pub fn depth(&self) -> usize {
        self.undo_stack.len()
    }
}

#[cfg(test)]
#[path = "piece_journal_tests.rs"]
mod tests;
