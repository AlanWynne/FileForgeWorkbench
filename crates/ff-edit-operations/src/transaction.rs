//! Transaction recording — EditorTransaction, LineSnapshot, and modified line tracking.
//!
//! This module defines what constitutes a transaction unit for the undo system.
//! The actual TransactionStack mechanics are in `ff-undo-redo-transactions`.

use std::collections::HashSet;

/// A snapshot of a single line's state at a point in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineSnapshot {
    /// 0-based line number.
    pub line_number: u64,
    /// The content of the line (without line ending).
    pub content: String,
}

impl LineSnapshot {
    /// Creates a new line snapshot.
    pub fn new(line_number: u64, content: String) -> Self {
        Self {
            line_number,
            content,
        }
    }
}

/// A single transaction unit for the undo system.
///
/// Contains before/after snapshots of affected lines. Each edit operation
/// produces one `EditorTransaction` that records the full state change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorTransaction {
    /// Lines affected by this transaction (0-based line numbers).
    pub affected_lines: Vec<u64>,
    /// Snapshot of line content before the edit.
    pub before_snapshot: Vec<LineSnapshot>,
    /// Snapshot of line content after the edit.
    pub after_snapshot: Vec<LineSnapshot>,
    /// Description for undo history display.
    pub description: String,
}

impl EditorTransaction {
    /// Creates a new transaction with the given snapshots.
    pub fn new(
        affected_lines: Vec<u64>,
        before_snapshot: Vec<LineSnapshot>,
        after_snapshot: Vec<LineSnapshot>,
        description: String,
    ) -> Self {
        Self {
            affected_lines,
            before_snapshot,
            after_snapshot,
            description,
        }
    }

    /// Returns true if this transaction has valid (non-empty) snapshots.
    pub fn is_valid(&self) -> bool {
        !self.before_snapshot.is_empty() || !self.after_snapshot.is_empty()
    }

    /// Returns the lines that were modified (present in either snapshot).
    pub fn modified_lines(&self) -> Vec<u64> {
        self.affected_lines.clone()
    }
}

/// Tracks which lines have been modified since the last save.
///
/// Provides set/clear operations for modified line markers and
/// supports save-point-relative recalculation.
#[derive(Debug, Clone)]
pub struct ModifiedLineTracker {
    modified_lines: HashSet<u64>,
}

impl ModifiedLineTracker {
    /// Creates a new tracker with no modified lines.
    pub fn new() -> Self {
        Self {
            modified_lines: HashSet::new(),
        }
    }

    /// Marks a line as modified.
    pub fn mark_modified(&mut self, line: u64) {
        self.modified_lines.insert(line);
    }

    /// Returns true if the given line is marked as modified.
    pub fn is_modified(&self, line: u64) -> bool {
        self.modified_lines.contains(&line)
    }

    /// Clears all modified markers (called on save).
    pub fn clear_all(&mut self) {
        self.modified_lines.clear();
    }

    /// Clears the modified marker for a single line.
    pub fn clear_line(&mut self, line: u64) {
        self.modified_lines.remove(&line);
    }

    /// Returns an iterator over all modified line numbers.
    pub fn modified_lines(&self) -> impl Iterator<Item = u64> + '_ {
        self.modified_lines.iter().copied()
    }

    /// Returns the count of modified lines.
    pub fn count(&self) -> usize {
        self.modified_lines.len()
    }

    /// Returns true if any lines are modified.
    pub fn has_modifications(&self) -> bool {
        !self.modified_lines.is_empty()
    }
}

impl Default for ModifiedLineTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A group of transactions that form a single undoable unit.
///
/// Used for multi-caret operations where multiple sub-edits should
/// be undone/redone atomically.
#[derive(Debug, Clone)]
pub struct UndoGroup {
    /// The sub-transactions in this group.
    pub transactions: Vec<EditorTransaction>,
    /// Description for the entire group.
    pub description: String,
}

impl UndoGroup {
    /// Creates a new empty undo group.
    pub fn new(description: String) -> Self {
        Self {
            transactions: Vec::new(),
            description,
        }
    }

    /// Adds a transaction to this group.
    pub fn push(&mut self, transaction: EditorTransaction) {
        self.transactions.push(transaction);
    }

    /// Returns the number of sub-transactions in this group.
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Returns true if the group has no transactions.
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Returns all lines modified by any transaction in this group.
    pub fn all_modified_lines(&self) -> Vec<u64> {
        let mut lines: HashSet<u64> = HashSet::new();
        for txn in &self.transactions {
            for line in &txn.affected_lines {
                lines.insert(*line);
            }
        }
        let mut result: Vec<u64> = lines.into_iter().collect();
        result.sort_unstable();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_snapshot_stores_content() {
        let snap = LineSnapshot::new(5, "hello world".to_string());
        assert_eq!(snap.line_number, 5);
        assert_eq!(snap.content, "hello world");
    }

    #[test]
    fn editor_transaction_is_valid_with_snapshots() {
        let txn = EditorTransaction::new(
            vec![0],
            vec![LineSnapshot::new(0, "before".to_string())],
            vec![LineSnapshot::new(0, "after".to_string())],
            "insert char".to_string(),
        );
        assert!(txn.is_valid());
    }

    #[test]
    fn editor_transaction_is_invalid_when_empty() {
        let txn = EditorTransaction::new(vec![], vec![], vec![], "no-op".to_string());
        assert!(!txn.is_valid());
    }

    #[test]
    fn modified_line_tracker_marks_and_queries() {
        let mut tracker = ModifiedLineTracker::new();
        assert!(!tracker.is_modified(5));

        tracker.mark_modified(5);
        assert!(tracker.is_modified(5));
        assert!(!tracker.is_modified(6));
    }

    #[test]
    fn modified_line_tracker_clear_all_removes_all_marks() {
        let mut tracker = ModifiedLineTracker::new();
        tracker.mark_modified(1);
        tracker.mark_modified(5);
        tracker.mark_modified(10);

        tracker.clear_all();
        assert!(!tracker.is_modified(1));
        assert!(!tracker.is_modified(5));
        assert!(!tracker.is_modified(10));
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn modified_line_tracker_clear_line_removes_single_mark() {
        let mut tracker = ModifiedLineTracker::new();
        tracker.mark_modified(1);
        tracker.mark_modified(5);

        tracker.clear_line(1);
        assert!(!tracker.is_modified(1));
        assert!(tracker.is_modified(5));
    }

    #[test]
    fn modified_line_tracker_count_reflects_state() {
        let mut tracker = ModifiedLineTracker::new();
        assert_eq!(tracker.count(), 0);
        assert!(!tracker.has_modifications());

        tracker.mark_modified(1);
        tracker.mark_modified(3);
        assert_eq!(tracker.count(), 2);
        assert!(tracker.has_modifications());
    }

    #[test]
    fn undo_group_collects_transactions() {
        let mut group = UndoGroup::new("multi-insert".to_string());
        assert!(group.is_empty());

        group.push(EditorTransaction::new(
            vec![0],
            vec![LineSnapshot::new(0, "a".to_string())],
            vec![LineSnapshot::new(0, "ab".to_string())],
            "insert b".to_string(),
        ));
        group.push(EditorTransaction::new(
            vec![2],
            vec![LineSnapshot::new(2, "c".to_string())],
            vec![LineSnapshot::new(2, "cb".to_string())],
            "insert b".to_string(),
        ));

        assert_eq!(group.len(), 2);
        assert!(!group.is_empty());
    }

    #[test]
    fn undo_group_all_modified_lines_aggregates_and_sorts() {
        let mut group = UndoGroup::new("test".to_string());
        group.push(EditorTransaction::new(
            vec![5, 3],
            vec![],
            vec![],
            "a".to_string(),
        ));
        group.push(EditorTransaction::new(
            vec![1, 5],
            vec![],
            vec![],
            "b".to_string(),
        ));

        let lines = group.all_modified_lines();
        assert_eq!(lines, vec![1, 3, 5]);
    }
}
