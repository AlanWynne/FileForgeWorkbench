//! Transaction types and builder for the undo/redo system.
//!
//! A [`Transaction`] is a named, atomic unit of work containing one or more edit
//! operations. The [`TransactionBuilder`] accumulates operations and supports
//! nesting, abort/rollback, and orphan detection.

use chrono::{DateTime, Utc};

use crate::edit_op::EditOperation;
use crate::selection::SelectionState;

/// A named, atomic unit of work in the undo history.
///
/// Contains one or more [`EditOperation`]s that are applied/reversed together.
/// Each transaction also records selection state for cursor restoration and
/// metadata for display in undo history UI.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Human-readable description (e.g., "Delete line 42").
    pub name: String,
    /// UTC timestamp when the transaction was committed.
    pub timestamp: DateTime<Utc>,
    /// Ordered list of edit operations in this transaction.
    pub operations: Vec<EditOperation>,
    /// Selection state before the transaction.
    pub selection_before: Option<SelectionState>,
    /// Selection state after the transaction.
    pub selection_after: Option<SelectionState>,
    /// Whether this transaction may be coalesced with the next.
    pub may_coalesce: bool,
}

impl Transaction {
    /// Returns the net document size change of this transaction.
    pub fn size_delta(&self) -> i64 {
        self.operations.iter().map(EditOperation::size_delta).sum()
    }

    /// Returns the number of operations in this transaction.
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Returns true if this transaction contains no operations.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

/// Builder for constructing transactions with nesting support.
///
/// Manages open transactions, tracks nesting depth, accumulates edit operations,
/// and handles abort/rollback.
///
/// # Nesting
///
/// Nested `begin`/`end` calls are counted. Only the outermost pair creates a
/// transaction boundary. Inner pairs increment/decrement the depth counter.
#[derive(Debug)]
pub struct TransactionBuilder {
    /// Current nesting depth (0 = no active transaction).
    depth: usize,
    /// Name of the outermost transaction.
    name: Option<String>,
    /// Operations accumulated in the current transaction.
    operations: Vec<EditOperation>,
    /// Selection state captured at transaction start.
    selection_before: Option<SelectionState>,
    /// Whether this transaction should allow coalescing.
    may_coalesce: bool,
}

impl TransactionBuilder {
    /// Creates a new transaction builder with no active transaction.
    pub fn new() -> Self {
        Self {
            depth: 0,
            name: None,
            operations: Vec::new(),
            selection_before: None,
            may_coalesce: true,
        }
    }

    /// Begins a transaction. Nested calls increment depth without creating a new boundary.
    ///
    /// Only the outermost `begin` sets the transaction name.
    pub fn begin(&mut self, name: &str) {
        if self.depth == 0 {
            self.name = Some(name.to_string());
            self.operations.clear();
            self.may_coalesce = true;
        }
        self.depth += 1;
    }

    /// Ends a transaction. Decrements depth; returns a committed [`Transaction`] only
    /// when depth reaches 0.
    ///
    /// Returns `None` if there are still open nested transactions, or if no
    /// transaction is in progress.
    pub fn end(&mut self, selection_after: Option<SelectionState>) -> Option<Transaction> {
        if self.depth == 0 {
            return None;
        }
        self.depth -= 1;
        if self.depth == 0 {
            let txn = Transaction {
                name: self.name.take().unwrap_or_default(),
                timestamp: Utc::now(),
                operations: std::mem::take(&mut self.operations),
                selection_before: self.selection_before.take(),
                selection_after,
                may_coalesce: self.may_coalesce,
            };
            Some(txn)
        } else {
            None
        }
    }

    /// Aborts the current transaction, discarding all accumulated operations.
    ///
    /// Returns the operations that were rolled back (for reversal by the caller).
    /// Resets depth to 0.
    pub fn abort(&mut self) -> Vec<EditOperation> {
        self.depth = 0;
        self.name = None;
        self.selection_before = None;
        self.may_coalesce = true;
        std::mem::take(&mut self.operations)
    }

    /// Force-closes an orphaned transaction (called at end of command dispatch cycle).
    ///
    /// Returns a committed transaction if operations were accumulated, or None if empty.
    pub fn force_close(&mut self, selection_after: Option<SelectionState>) -> Option<Transaction> {
        if self.depth == 0 {
            return None;
        }
        self.depth = 0;
        let ops = std::mem::take(&mut self.operations);
        if ops.is_empty() {
            self.name = None;
            self.selection_before = None;
            return None;
        }
        let txn = Transaction {
            name: self.name.take().unwrap_or_else(|| "orphaned".to_string()),
            timestamp: Utc::now(),
            operations: ops,
            selection_before: self.selection_before.take(),
            selection_after,
            may_coalesce: false,
        };
        Some(txn)
    }

    /// Adds an edit operation to the current transaction.
    pub fn add_operation(&mut self, op: EditOperation) {
        self.operations.push(op);
    }

    /// Sets the selection state captured before the transaction began.
    pub fn set_selection_before(&mut self, state: SelectionState) {
        if self.selection_before.is_none() {
            self.selection_before = Some(state);
        }
    }

    /// Sets whether this transaction allows coalescing.
    pub fn set_may_coalesce(&mut self, may_coalesce: bool) {
        self.may_coalesce = may_coalesce;
    }

    /// Returns the current nesting depth (0 = no active transaction).
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns true if a transaction is currently in progress.
    pub fn is_active(&self) -> bool {
        self.depth > 0
    }

    /// Returns the number of operations accumulated so far.
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Returns a reference to the accumulated operations.
    pub fn operations(&self) -> &[EditOperation] {
        &self.operations
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builder_has_zero_depth() {
        let builder = TransactionBuilder::new();
        assert_eq!(builder.depth(), 0);
        assert!(!builder.is_active());
    }

    #[test]
    fn begin_increments_depth() {
        let mut builder = TransactionBuilder::new();
        builder.begin("test");
        assert_eq!(builder.depth(), 1);
        assert!(builder.is_active());
    }

    #[test]
    fn nested_begin_increments_depth_without_new_boundary() {
        let mut builder = TransactionBuilder::new();
        builder.begin("outer");
        builder.begin("inner");
        assert_eq!(builder.depth(), 2);
    }

    #[test]
    fn end_at_depth_one_commits_transaction() {
        let mut builder = TransactionBuilder::new();
        builder.begin("test");
        builder.add_operation(EditOperation::Insert {
            position: 0,
            length: 1,
            scrap_offset: 0,
        });
        let txn = builder.end(None);
        assert!(txn.is_some());
        let txn = txn.unwrap();
        assert_eq!(txn.name, "test");
        assert_eq!(txn.operations.len(), 1);
        assert_eq!(builder.depth(), 0);
    }

    #[test]
    fn end_at_depth_two_does_not_commit() {
        let mut builder = TransactionBuilder::new();
        builder.begin("outer");
        builder.begin("inner");
        let txn = builder.end(None);
        assert!(txn.is_none());
        assert_eq!(builder.depth(), 1);
    }

    #[test]
    fn end_with_no_active_transaction_returns_none() {
        let mut builder = TransactionBuilder::new();
        let txn = builder.end(None);
        assert!(txn.is_none());
    }

    #[test]
    fn abort_resets_depth_and_returns_operations() {
        let mut builder = TransactionBuilder::new();
        builder.begin("test");
        builder.add_operation(EditOperation::Insert {
            position: 0,
            length: 1,
            scrap_offset: 0,
        });
        let ops = builder.abort();
        assert_eq!(ops.len(), 1);
        assert_eq!(builder.depth(), 0);
        assert!(!builder.is_active());
    }

    #[test]
    fn force_close_commits_orphaned_transaction() {
        let mut builder = TransactionBuilder::new();
        builder.begin("orphaned");
        builder.add_operation(EditOperation::Delete {
            position: 5,
            length: 3,
            scrap_offset: 0,
        });
        let txn = builder.force_close(None);
        assert!(txn.is_some());
        let txn = txn.unwrap();
        assert_eq!(txn.name, "orphaned");
        assert!(!txn.may_coalesce);
        assert_eq!(builder.depth(), 0);
    }

    #[test]
    fn force_close_with_no_ops_returns_none() {
        let mut builder = TransactionBuilder::new();
        builder.begin("empty");
        let txn = builder.force_close(None);
        assert!(txn.is_none());
    }

    #[test]
    fn transaction_size_delta_sums_operations() {
        let txn = Transaction {
            name: "test".to_string(),
            timestamp: Utc::now(),
            operations: vec![
                EditOperation::Insert {
                    position: 0,
                    length: 5,
                    scrap_offset: 0,
                },
                EditOperation::Delete {
                    position: 10,
                    length: 2,
                    scrap_offset: 5,
                },
            ],
            selection_before: None,
            selection_after: None,
            may_coalesce: true,
        };
        assert_eq!(txn.size_delta(), 3); // +5 - 2
    }

    #[test]
    fn selection_before_set_only_once() {
        let mut builder = TransactionBuilder::new();
        builder.begin("test");
        builder.set_selection_before(SelectionState::single_caret(0));
        builder.set_selection_before(SelectionState::single_caret(99));
        let txn = builder.end(None).unwrap();
        assert_eq!(txn.selection_before.unwrap().carets[0].position, 0);
    }
}
