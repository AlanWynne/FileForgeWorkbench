//! UndoManager — per-document orchestrator for the undo/redo system.
//!
//! [`DocumentUndoManager`] integrates all components (stacks, coalescing, save point,
//! selection history, tentative actions, scrap stack) into a single per-document API.

use crate::coalesce::{CoalesceOpType, CoalesceState};
use crate::config::UndoConfig;
use crate::edit_op::EditOperation;
use crate::error::UndoError;
use crate::notify::{ListenerId, UndoNotifier};
use crate::save_point::SavePointState;
use crate::scrap::ScrapStack;
use crate::selection::SelectionState;
use crate::stack::{RedoStack, UndoStack};
use crate::tentative::TentativeState;
use crate::transaction::{Transaction, TransactionBuilder};

/// The primary public type — one instance per open document.
///
/// Encapsulates all undo/redo state for a single document session.
pub struct DocumentUndoManager {
    config: UndoConfig,
    undo_stack: UndoStack,
    redo_stack: RedoStack,
    scrap: ScrapStack,
    save_point: SavePointState,
    coalesce: CoalesceState,
    tentative: TentativeState,
    builder: TransactionBuilder,
    current_selection: Option<SelectionState>,
    listeners: Vec<(ListenerId, Box<dyn UndoNotifier>)>,
    next_listener_id: u64,
}

impl DocumentUndoManager {
    /// Creates a new undo manager with the given configuration.
    pub fn new(config: UndoConfig) -> Self {
        let coalesce_timeout = config.coalesce_timeout_ms;
        let max_levels = config.max_levels;
        Self {
            config,
            undo_stack: UndoStack::new(max_levels),
            redo_stack: RedoStack::new(),
            scrap: ScrapStack::new(),
            save_point: SavePointState::new(),
            coalesce: CoalesceState::new(coalesce_timeout),
            tentative: TentativeState::new(),
            builder: TransactionBuilder::new(),
            current_selection: None,
            listeners: Vec::new(),
            next_listener_id: 1,
        }
    }

    // --- Transaction API ---

    /// Begin an explicit transaction group. Nested calls increment depth.
    pub fn begin_transaction(&mut self, name: &str) {
        if self.config.is_undo_disabled() {
            return;
        }
        self.coalesce.break_coalesce();
        if self.builder.depth() == 0 {
            if let Some(sel) = self.current_selection.clone() {
                self.builder.set_selection_before(sel);
            }
        }
        self.builder.begin(name);
        self.coalesce.enter_explicit_group();
    }

    /// End an explicit transaction group. Commits when outermost closes.
    pub fn end_transaction(&mut self) {
        if self.config.is_undo_disabled() {
            return;
        }
        if self.builder.depth() == 1 {
            self.coalesce.exit_explicit_group();
        }
        if let Some(txn) = self.builder.end(self.current_selection.clone()) {
            self.commit_transaction(txn);
        }
    }

    /// Abort the current transaction, rolling back all operations.
    pub fn abort_transaction(&mut self) {
        let _ops = self.builder.abort();
        self.coalesce.break_coalesce();
        // Caller is responsible for reversing ops in the document
    }

    /// Returns the current transaction nesting depth.
    pub fn transaction_depth(&self) -> usize {
        self.builder.depth()
    }

    // --- Edit Recording ---

    /// Record an insert operation.
    pub fn record_insert(&mut self, position: u64, text: &[u8]) {
        if self.config.is_undo_disabled() {
            return;
        }
        let (scrap_offset, length) = self.scrap.push(text);
        let op = EditOperation::Insert {
            position,
            length,
            scrap_offset,
        };
        self.record_operation(op, position, length, CoalesceOpType::CharInsert);
    }

    /// Record a delete operation.
    pub fn record_delete(&mut self, position: u64, text: &[u8]) {
        if self.config.is_undo_disabled() {
            return;
        }
        let (scrap_offset, length) = self.scrap.push(text);
        let op = EditOperation::Delete {
            position,
            length,
            scrap_offset,
        };
        // Determine if backspace or delete-key pattern
        let op_type = CoalesceOpType::CharDelete;
        self.record_operation(op, position, length, op_type);
    }

    /// Record a replace operation.
    pub fn record_replace(&mut self, position: u64, old_text: &[u8], new_text: &[u8]) {
        if self.config.is_undo_disabled() {
            return;
        }
        let (old_scrap_offset, old_length) = self.scrap.push(old_text);
        let (new_scrap_offset, new_length) = self.scrap.push(new_text);
        let op = EditOperation::Replace {
            position,
            old_length,
            new_length,
            old_scrap_offset,
            new_scrap_offset,
        };
        self.coalesce.break_coalesce();
        self.add_to_transaction(op);
    }

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

    // --- Save Point ---

    /// Mark current position as save point.
    pub fn set_save_point(&mut self) {
        self.save_point.set_save_point();
        self.coalesce.break_coalesce();
        self.notify_dirty_changed(false);
    }

    /// Returns true if at the save point.
    pub fn is_at_save_point(&self) -> bool {
        self.save_point.is_at_save_point()
    }

    /// Returns true if before the save point.
    pub fn before_save_point(&self) -> bool {
        self.save_point.before_save_point()
    }

    /// Returns true if after the save point.
    pub fn after_save_point(&self) -> bool {
        self.save_point.after_save_point()
    }

    /// Returns true if save point is unreachable.
    pub fn after_detach_point(&self) -> bool {
        self.save_point.after_detach_point()
    }

    /// Returns the dirty flag state.
    pub fn is_dirty(&self) -> bool {
        self.save_point.is_dirty()
    }

    // --- Coalescing ---

    /// Notify that the coalesce timeout has elapsed.
    pub fn coalesce_timeout_expired(&mut self) {
        self.coalesce.timeout_expired();
    }

    /// Force a coalescing boundary.
    pub fn break_coalesce(&mut self) {
        self.coalesce.break_coalesce();
    }

    // --- Tentative Actions ---

    /// Enter tentative mode for IME composition.
    pub fn tentative_start(&mut self) {
        self.coalesce.break_coalesce();
        let idx = self.save_point.current_action();
        let _ = self.tentative.start(idx);
    }

    /// Commit tentative actions.
    pub fn tentative_commit(&mut self) {
        let _ = self.tentative.commit();
        self.coalesce.break_coalesce();
    }

    /// Roll back tentative actions.
    pub fn tentative_rollback(&mut self) -> usize {
        match self.tentative.rollback() {
            Ok((_point, steps)) => {
                // Undo `steps` transactions without recording to redo
                for _ in 0..steps {
                    if self.undo_stack.pop().is_some() {
                        self.save_point.on_undo();
                    }
                }
                self.coalesce.break_coalesce();
                steps
            }
            Err(()) => 0,
        }
    }

    /// Query whether tentative mode is active.
    pub fn tentative_active(&self) -> bool {
        self.tentative.is_active()
    }

    /// Number of actions since the tentative point.
    pub fn tentative_steps(&self) -> Option<usize> {
        self.tentative.steps()
    }

    // --- Selection History ---

    /// Set the current selection state.
    pub fn set_selection_state(&mut self, state: SelectionState) {
        self.current_selection = Some(state);
    }

    /// Get the current selection state.
    pub fn current_selection(&self) -> Option<&SelectionState> {
        self.current_selection.as_ref()
    }

    // --- Recovery ---

    /// Serialize current undo state for recovery file.
    pub fn serialize_for_recovery(&self) -> Result<Vec<u8>, UndoError> {
        crate::recovery::serialize_for_recovery(
            &self.scrap,
            self.save_point.save_point(),
            self.save_point.current_action(),
            None,
        )
    }

    /// Restore undo state from recovery data.
    pub fn restore_from_recovery(data: &[u8], config: UndoConfig) -> Result<Self, UndoError> {
        let payload = crate::recovery::deserialize_recovery(data)?;
        let scrap = ScrapStack::from_bytes(payload.scrap_data);
        let mut manager = Self::new(config);
        manager.scrap = scrap;
        Ok(manager)
    }

    // --- History Management ---

    /// Clear all undo/redo history and reset state.
    pub fn delete_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.scrap.clear();
        self.save_point.reset();
        self.coalesce.reset();
        self.tentative.reset();
        self.builder = TransactionBuilder::new();
        self.current_selection = None;
        self.notify_dirty_changed(false);
        self.notify_undo_redo_availability();
    }

    /// Validate internal consistency against current document size.
    pub fn validate(&self, document_length: u64) -> bool {
        let txns: Vec<&Transaction> = self.undo_stack.iter().collect();
        crate::validate::validate_history(&txns, 0, document_length).is_ok()
    }

    /// Get the current undo stack depth.
    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the current redo stack depth.
    pub fn redo_depth(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get the configured max levels.
    pub fn max_levels(&self) -> u32 {
        self.config.max_levels
    }

    /// Check if undo is disabled.
    pub fn is_undo_disabled(&self) -> bool {
        self.config.is_undo_disabled()
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &UndoConfig {
        &self.config
    }

    // --- Notifications ---

    /// Register a notification listener.
    pub fn add_listener(&mut self, listener: Box<dyn UndoNotifier>) -> ListenerId {
        let id = ListenerId(self.next_listener_id);
        self.next_listener_id += 1;
        self.listeners.push((id, listener));
        id
    }

    /// Remove a notification listener.
    pub fn remove_listener(&mut self, id: ListenerId) {
        self.listeners.retain(|(lid, _)| *lid != id);
    }

    // --- Private helpers ---

    fn record_operation(
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

    fn add_to_transaction(&mut self, op: EditOperation) {
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

    fn commit_transaction(&mut self, txn: Transaction) {
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

    fn notify_dirty_changed(&self, is_dirty: bool) {
        for (_, listener) in &self.listeners {
            listener.dirty_flag_changed(is_dirty);
        }
    }

    fn notify_undo_redo_availability(&self) {
        let can_undo = self.can_undo();
        let can_redo = self.can_redo();
        for (_, listener) in &self.listeners {
            listener.undo_available_changed(can_undo);
            listener.redo_available_changed(can_redo);
        }
    }

    fn notify_transaction_committed(&self, name: &str) {
        for (_, listener) in &self.listeners {
            listener.transaction_committed(name);
        }
    }

    fn notify_transaction_undone(&self, name: &str) {
        for (_, listener) in &self.listeners {
            listener.transaction_undone(name);
        }
    }

    fn notify_transaction_redone(&self, name: &str) {
        for (_, listener) in &self.listeners {
            listener.transaction_redone(name);
        }
    }
}

fn format_op_name(op_type: CoalesceOpType) -> String {
    match op_type {
        CoalesceOpType::CharInsert => "typing".to_string(),
        CoalesceOpType::CharBackspace => "backspace".to_string(),
        CoalesceOpType::CharDelete => "delete".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_manager() -> DocumentUndoManager {
        DocumentUndoManager::new(UndoConfig::default())
    }

    #[test]
    fn new_manager_has_empty_stacks() {
        let mgr = default_manager();
        assert_eq!(mgr.undo_depth(), 0);
        assert_eq!(mgr.redo_depth(), 0);
        assert!(!mgr.can_undo());
        assert!(!mgr.can_redo());
    }

    #[test]
    fn record_insert_adds_to_undo_stack() {
        let mut mgr = default_manager();
        mgr.record_insert(0, b"a");
        assert_eq!(mgr.undo_depth(), 1);
        assert!(mgr.can_undo());
    }

    #[test]
    fn undo_moves_to_redo_stack() {
        let mut mgr = default_manager();
        mgr.record_insert(0, b"a");
        mgr.break_coalesce();
        mgr.record_insert(1, b"b");
        assert_eq!(mgr.undo_depth(), 2);
        mgr.undo().unwrap();
        assert_eq!(mgr.undo_depth(), 1);
        assert_eq!(mgr.redo_depth(), 1);
    }

    #[test]
    fn redo_moves_back_to_undo_stack() {
        let mut mgr = default_manager();
        mgr.record_insert(0, b"a");
        mgr.break_coalesce();
        mgr.record_insert(1, b"b");
        mgr.undo().unwrap();
        mgr.redo().unwrap();
        assert_eq!(mgr.undo_depth(), 2);
        assert_eq!(mgr.redo_depth(), 0);
    }

    #[test]
    fn new_commit_clears_redo() {
        let mut mgr = default_manager();
        mgr.record_insert(0, b"a");
        mgr.undo().unwrap();
        assert!(mgr.can_redo());
        mgr.record_insert(0, b"x");
        assert!(!mgr.can_redo());
    }

    #[test]
    fn undo_on_empty_returns_error() {
        let mut mgr = default_manager();
        assert!(matches!(mgr.undo(), Err(UndoError::NothingToUndo)));
    }

    #[test]
    fn redo_on_empty_returns_error() {
        let mut mgr = default_manager();
        assert!(matches!(mgr.redo(), Err(UndoError::NothingToRedo)));
    }

    #[test]
    fn disabled_undo_returns_error() {
        let config = UndoConfig {
            max_levels: 0,
            ..UndoConfig::default()
        };
        let mut mgr = DocumentUndoManager::new(config);
        mgr.record_insert(0, b"a");
        assert_eq!(mgr.undo_depth(), 0);
        assert!(matches!(mgr.undo(), Err(UndoError::UndoDisabled)));
    }

    #[test]
    fn save_point_and_dirty_flag() {
        let mut mgr = default_manager();
        assert!(!mgr.is_dirty());
        mgr.record_insert(0, b"a");
        assert!(mgr.is_dirty());
        mgr.set_save_point();
        assert!(!mgr.is_dirty());
        mgr.record_insert(1, b"b");
        assert!(mgr.is_dirty());
    }

    #[test]
    fn undo_back_to_save_clears_dirty() {
        let mut mgr = default_manager();
        mgr.set_save_point();
        mgr.record_insert(0, b"a");
        assert!(mgr.is_dirty());
        mgr.undo().unwrap();
        assert!(!mgr.is_dirty());
    }

    #[test]
    fn transaction_grouping() {
        let mut mgr = default_manager();
        mgr.begin_transaction("group");
        mgr.record_insert(0, b"abc");
        mgr.record_insert(3, b"def");
        mgr.end_transaction();
        // Should be a single transaction
        assert_eq!(mgr.undo_depth(), 1);
        let desc = mgr.undo_description().unwrap();
        assert_eq!(desc, "group");
    }

    #[test]
    fn undo_n_undoes_multiple() {
        let mut mgr = default_manager();
        mgr.record_insert(0, b"a");
        mgr.break_coalesce();
        mgr.record_insert(1, b"b");
        mgr.break_coalesce();
        mgr.record_insert(2, b"c");
        let count = mgr.undo_n(2).unwrap();
        assert_eq!(count, 2);
        assert_eq!(mgr.undo_depth(), 1);
        assert_eq!(mgr.redo_depth(), 2);
    }

    #[test]
    fn delete_history_clears_everything() {
        let mut mgr = default_manager();
        mgr.record_insert(0, b"abc");
        mgr.undo().unwrap();
        mgr.delete_history();
        assert_eq!(mgr.undo_depth(), 0);
        assert_eq!(mgr.redo_depth(), 0);
        assert!(!mgr.is_dirty());
    }

    #[test]
    fn tentative_commit_makes_permanent() {
        let mut mgr = default_manager();
        mgr.tentative_start();
        mgr.record_insert(0, b"x");
        mgr.tentative_commit();
        assert_eq!(mgr.undo_depth(), 1);
        assert!(!mgr.tentative_active());
    }

    #[test]
    fn tentative_rollback_removes_transactions() {
        let mut mgr = default_manager();
        mgr.record_insert(0, b"base");
        mgr.break_coalesce();
        assert_eq!(mgr.undo_depth(), 1);
        mgr.tentative_start();
        mgr.record_insert(4, b"x");
        mgr.break_coalesce();
        mgr.record_insert(5, b"y");
        // tentative added 2 transactions
        assert_eq!(mgr.undo_depth(), 3);
        let rolled = mgr.tentative_rollback();
        assert_eq!(rolled, 2);
        assert_eq!(mgr.undo_depth(), 1);
    }

    #[test]
    fn selection_state_captured_and_restored() {
        let mut mgr = default_manager();
        mgr.set_selection_state(SelectionState::single_caret(0));
        mgr.record_insert(0, b"hello");
        mgr.set_selection_state(SelectionState::single_caret(5));
        mgr.break_coalesce();
        mgr.record_insert(5, b" world");
        // Undo should restore selection before second op
        mgr.undo().unwrap();
        // After undo, current selection should be the before-state of undone txn
        assert_eq!(mgr.current_selection().unwrap().carets[0].position, 5);
    }
}
