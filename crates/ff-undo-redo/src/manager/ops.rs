//! Undo/redo execution + the private state-mutation helpers for
//! `DocumentUndoManager`. Split from the model for file-size; operates on
//! the same struct via pub(super) fields/helpers.

use crate::coalesce::CoalesceOpType;
use crate::edit_op::EditOperation;
use crate::error::UndoError;
use crate::transaction::Transaction;

use super::{format_op_name, DocumentUndoManager};

impl DocumentUndoManager {
    // --- Undo/Redo Execution ---

    /// Execute a single undo operation.
    pub fn undo(&mut self) -> Result<Option<&Transaction>, UndoError> {
        if self.config.is_undo_disabled() {
            return Err(UndoError::UndoDisabled);
        }
        // Force-close any open transaction
        if let Some(txn) = self.builder.force_close(self.current_selection.clone()) {
            self.commit_transaction(txn);
        }
        let txn = self.undo_stack.pop().ok_or(UndoError::NothingToUndo)?;
        let was_dirty = self.save_point.is_dirty();
        self.save_point.on_undo();
        // Restore before-state selection
        if self.config.selection_history_enabled {
            if let Some(ref sel) = txn.selection_before {
                self.current_selection = Some(sel.clone());
            }
        }
        self.redo_stack.push(txn);
        self.coalesce.break_coalesce();
        if self.tentative.is_active() {
            self.tentative.record_step();
        }
        let is_dirty = self.save_point.is_dirty();
        if was_dirty != is_dirty {
            self.notify_dirty_changed(is_dirty);
        }
        self.notify_undo_redo_availability();
        let name = self
            .redo_stack
            .peek()
            .map(|t| t.name.as_str())
            .unwrap_or("");
        self.notify_transaction_undone(name);
        Ok(self.redo_stack.peek())
    }

    /// Execute N successive undo operations. Returns count actually undone.
    pub fn undo_n(&mut self, count: usize) -> Result<usize, UndoError> {
        if self.config.is_undo_disabled() {
            return Err(UndoError::UndoDisabled);
        }
        let mut undone = 0;
        for _ in 0..count {
            match self.undo() {
                Ok(_) => undone += 1,
                Err(UndoError::NothingToUndo) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(undone)
    }

    /// Execute a single redo operation.
    pub fn redo(&mut self) -> Result<Option<&Transaction>, UndoError> {
        if self.config.is_undo_disabled() {
            return Err(UndoError::UndoDisabled);
        }
        let txn = self.redo_stack.pop().ok_or(UndoError::NothingToRedo)?;
        let was_dirty = self.save_point.is_dirty();
        self.save_point.on_redo();
        // Restore after-state selection
        if self.config.selection_history_enabled {
            if let Some(ref sel) = txn.selection_after {
                self.current_selection = Some(sel.clone());
            }
        }
        self.undo_stack.push(txn);
        self.coalesce.break_coalesce();
        let is_dirty = self.save_point.is_dirty();
        if was_dirty != is_dirty {
            self.notify_dirty_changed(is_dirty);
        }
        self.notify_undo_redo_availability();
        let name = self
            .undo_stack
            .peek()
            .map(|t| t.name.as_str())
            .unwrap_or("");
        self.notify_transaction_redone(name);
        Ok(self.undo_stack.peek())
    }

    /// Execute N successive redo operations. Returns count actually redone.
    pub fn redo_n(&mut self, count: usize) -> Result<usize, UndoError> {
        if self.config.is_undo_disabled() {
            return Err(UndoError::UndoDisabled);
        }
        let mut redone = 0;
        for _ in 0..count {
            match self.redo() {
                Ok(_) => redone += 1,
                Err(UndoError::NothingToRedo) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(redone)
    }

    /// Check whether undo is available.
    pub fn can_undo(&self) -> bool {
        !self.config.is_undo_disabled() && !self.undo_stack.is_empty()
    }

    /// Check whether redo is available.
    pub fn can_redo(&self) -> bool {
        !self.config.is_undo_disabled() && !self.redo_stack.is_empty()
    }

    /// Get the description of the next undo transaction.
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.peek().map(|t| t.name.as_str())
    }

    /// Get the description of the next redo transaction.
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.peek().map(|t| t.name.as_str())
    }

    // --- Private helpers ---

    pub(super) fn record_operation(
        &mut self,
        op: EditOperation,
        position: u64,
        length: u32,
        op_type: CoalesceOpType,
    ) {
        // If in explicit transaction, just add to builder
        if self.builder.is_active() {
            self.builder.add_operation(op);
            self.coalesce
                .record_operation(op_type, position, length, true);
            return;
        }

        // Check coalescing
        if self
            .coalesce
            .should_coalesce(op_type, position, length, true)
        {
            // Extend existing transaction on top of undo stack
            if let Some(txn) = self.undo_stack.peek_mut() {
                txn.operations.push(op);
                txn.selection_after = self.current_selection.clone();
                self.coalesce
                    .record_operation(op_type, position, length, true);
                return;
            }
        }

        // Start new transaction
        self.coalesce.break_coalesce();
        let txn = Transaction {
            name: format_op_name(op_type),
            timestamp: chrono::Utc::now(),
            operations: vec![op],
            selection_before: self.current_selection.clone(),
            selection_after: self.current_selection.clone(),
            may_coalesce: true,
        };
        self.commit_transaction(txn);
        self.coalesce
            .record_operation(op_type, position, length, true);
    }

    pub(super) fn add_to_transaction(&mut self, op: EditOperation) {
        if self.builder.is_active() {
            self.builder.add_operation(op);
        } else {
            let txn = Transaction {
                name: "replace".to_string(),
                timestamp: chrono::Utc::now(),
                operations: vec![op],
                selection_before: self.current_selection.clone(),
                selection_after: self.current_selection.clone(),
                may_coalesce: false,
            };
            self.commit_transaction(txn);
        }
    }

    pub(super) fn commit_transaction(&mut self, txn: Transaction) {
        let redo_was_non_empty = !self.redo_stack.is_empty();
        self.redo_stack.clear();
        self.undo_stack.push(txn);
        self.save_point.on_commit(redo_was_non_empty);

        if self.tentative.is_active() {
            self.tentative.record_step();
        }

        self.notify_dirty_changed(self.save_point.is_dirty());
        self.notify_undo_redo_availability();
        if let Some(name) = self.undo_stack.peek().map(|t| t.name.clone()) {
            self.notify_transaction_committed(&name);
        }
    }

    pub(super) fn notify_dirty_changed(&self, is_dirty: bool) {
        for (_, listener) in &self.listeners {
            listener.dirty_flag_changed(is_dirty);
        }
    }

    pub(super) fn notify_undo_redo_availability(&self) {
        let can_undo = self.can_undo();
        let can_redo = self.can_redo();
        for (_, listener) in &self.listeners {
            listener.undo_available_changed(can_undo);
            listener.redo_available_changed(can_redo);
        }
    }

    pub(super) fn notify_transaction_committed(&self, name: &str) {
        for (_, listener) in &self.listeners {
            listener.transaction_committed(name);
        }
    }

    pub(super) fn notify_transaction_undone(&self, name: &str) {
        for (_, listener) in &self.listeners {
            listener.transaction_undone(name);
        }
    }

    pub(super) fn notify_transaction_redone(&self, name: &str) {
        for (_, listener) in &self.listeners {
            listener.transaction_redone(name);
        }
    }
}
