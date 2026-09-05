//! Undo and redo stack implementations.
//!
//! [`UndoStack`] is a bounded FIFO stack (oldest evicted when full).
//! [`RedoStack`] is an unbounded LIFO stack (cleared on new commit).

use std::collections::VecDeque;

use crate::transaction::Transaction;

/// Bounded undo stack — discards oldest transactions when exceeding `max_levels`.
///
/// Implemented as a `VecDeque` with FIFO eviction at the front and LIFO access
/// from the back (most recent transaction).
#[derive(Debug, Clone)]
pub struct UndoStack {
    /// Bounded transaction storage.
    stack: VecDeque<Transaction>,
    /// Maximum depth (0 = undo disabled).
    max_levels: usize,
}

impl UndoStack {
    /// Creates a new undo stack with the given maximum depth.
    pub fn new(max_levels: u32) -> Self {
        let max = max_levels as usize;
        Self {
            stack: VecDeque::with_capacity(max.min(128)),
            max_levels: max,
        }
    }

    /// Pushes a transaction onto the stack.
    ///
    /// If the stack exceeds `max_levels`, the oldest transaction is evicted.
    /// Returns `true` if an eviction occurred.
    pub fn push(&mut self, transaction: Transaction) -> bool {
        if self.max_levels == 0 {
            return false;
        }
        let evicted = if self.stack.len() >= self.max_levels {
            self.stack.pop_front();
            true
        } else {
            false
        };
        self.stack.push_back(transaction);
        evicted
    }

    /// Pops the most recent transaction from the stack.
    pub fn pop(&mut self) -> Option<Transaction> {
        self.stack.pop_back()
    }

    /// Returns a reference to the most recent transaction without removing it.
    pub fn peek(&self) -> Option<&Transaction> {
        self.stack.back()
    }

    /// Returns a mutable reference to the most recent transaction.
    pub fn peek_mut(&mut self) -> Option<&mut Transaction> {
        self.stack.back_mut()
    }

    /// Clears all transactions from the stack.
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Returns the current number of transactions in the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns true if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns the configured maximum depth.
    pub fn max_levels(&self) -> usize {
        self.max_levels
    }

    /// Returns an iterator over transactions from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = &Transaction> {
        self.stack.iter()
    }

    /// Sets a new maximum depth, taking immediate effect.
    ///
    /// If the new limit is smaller than the current stack depth, the oldest
    /// transactions are evicted until the stack fits within the new limit.
    pub fn set_max_levels(&mut self, new_max: u32) {
        self.max_levels = new_max as usize;
        while self.max_levels > 0 && self.stack.len() > self.max_levels {
            self.stack.pop_front();
        }
        if self.max_levels == 0 {
            self.stack.clear();
        }
    }
}

/// Redo stack — unbounded LIFO storage for undone transactions.
///
/// Cleared when a new transaction is committed (standard branching semantics).
#[derive(Debug, Clone, Default)]
pub struct RedoStack {
    /// Transaction storage.
    stack: Vec<Transaction>,
}

impl RedoStack {
    /// Creates a new empty redo stack.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Pushes an undone transaction onto the redo stack.
    pub fn push(&mut self, transaction: Transaction) {
        self.stack.push(transaction);
    }

    /// Pops the most recent undone transaction for re-application.
    pub fn pop(&mut self) -> Option<Transaction> {
        self.stack.pop()
    }

    /// Returns a reference to the most recent redo transaction.
    pub fn peek(&self) -> Option<&Transaction> {
        self.stack.last()
    }

    /// Clears all transactions from the redo stack.
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Returns the current number of transactions in the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns true if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_transaction(name: &str) -> Transaction {
        Transaction {
            name: name.to_string(),
            timestamp: Utc::now(),
            operations: vec![],
            selection_before: None,
            selection_after: None,
            may_coalesce: true,
        }
    }

    // --- UndoStack tests ---

    #[test]
    fn new_undo_stack_is_empty() {
        let stack = UndoStack::new(100);
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn push_increases_length() {
        let mut stack = UndoStack::new(100);
        stack.push(make_transaction("t1"));
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn pop_returns_most_recent() {
        let mut stack = UndoStack::new(100);
        stack.push(make_transaction("first"));
        stack.push(make_transaction("second"));
        let txn = stack.pop().unwrap();
        assert_eq!(txn.name, "second");
    }

    #[test]
    fn push_evicts_oldest_when_at_max_levels() {
        let mut stack = UndoStack::new(3);
        stack.push(make_transaction("t1"));
        stack.push(make_transaction("t2"));
        stack.push(make_transaction("t3"));
        let evicted = stack.push(make_transaction("t4"));
        assert!(evicted);
        assert_eq!(stack.len(), 3);
        // t1 should be evicted; oldest remaining is t2
        let names: Vec<_> = stack.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["t2", "t3", "t4"]);
    }

    #[test]
    fn push_with_zero_max_levels_is_noop() {
        let mut stack = UndoStack::new(0);
        let evicted = stack.push(make_transaction("t1"));
        assert!(!evicted);
        assert!(stack.is_empty());
    }

    #[test]
    fn clear_removes_all_transactions() {
        let mut stack = UndoStack::new(100);
        stack.push(make_transaction("t1"));
        stack.push(make_transaction("t2"));
        stack.clear();
        assert!(stack.is_empty());
    }

    #[test]
    fn peek_returns_most_recent_without_removing() {
        let mut stack = UndoStack::new(100);
        stack.push(make_transaction("t1"));
        assert_eq!(stack.peek().unwrap().name, "t1");
        assert_eq!(stack.len(), 1);
    }

    // --- RedoStack tests ---

    #[test]
    fn new_redo_stack_is_empty() {
        let stack = RedoStack::new();
        assert!(stack.is_empty());
    }

    #[test]
    fn redo_push_and_pop_is_lifo() {
        let mut stack = RedoStack::new();
        stack.push(make_transaction("r1"));
        stack.push(make_transaction("r2"));
        assert_eq!(stack.pop().unwrap().name, "r2");
        assert_eq!(stack.pop().unwrap().name, "r1");
    }

    #[test]
    fn redo_clear_empties_stack() {
        let mut stack = RedoStack::new();
        stack.push(make_transaction("r1"));
        stack.clear();
        assert!(stack.is_empty());
    }

    #[test]
    fn pop_on_empty_undo_stack_returns_none() {
        let mut stack = UndoStack::new(100);
        assert!(stack.pop().is_none());
    }

    #[test]
    fn pop_on_empty_redo_stack_returns_none() {
        let mut stack = RedoStack::new();
        assert!(stack.pop().is_none());
    }
}
