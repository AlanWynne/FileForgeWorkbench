//! Undo/Redo integration — stack management for undoable commands.

use std::sync::Mutex;

use crate::context::ExecutionContext;
use crate::error::CommandError;
use crate::result::UndoRecord;

/// Trait for managing undo/redo stacks.
///
/// Implemented by the `undo-redo-transactions` crate. Provided to
/// `CommandDispatch` during platform initialization.
pub trait UndoManager: Send + Sync {
    /// Push an undo record onto the undo stack.
    fn push_undo(&self, record: Box<dyn UndoRecord>);

    /// Pop the most recent undo record from the undo stack.
    fn pop_undo(&self) -> Option<Box<dyn UndoRecord>>;

    /// Push a record onto the redo stack.
    fn push_redo(&self, record: Box<dyn UndoRecord>);

    /// Pop the most recent record from the redo stack.
    fn pop_redo(&self) -> Option<Box<dyn UndoRecord>>;

    /// Clear the redo stack (called when a new undoable command executes after undo).
    fn clear_redo(&self);
}

/// Default in-process undo manager using simple stacks.
///
/// Used when no external undo manager is provided. Stores undo/redo records
/// in LIFO stacks with thread-safe access.
pub struct DefaultUndoManager {
    undo_stack: Mutex<Vec<Box<dyn UndoRecord>>>,
    redo_stack: Mutex<Vec<Box<dyn UndoRecord>>>,
}

impl DefaultUndoManager {
    /// Creates a new empty undo manager.
    pub fn new() -> Self {
        Self {
            undo_stack: Mutex::new(Vec::new()),
            redo_stack: Mutex::new(Vec::new()),
        }
    }

    /// Returns the current undo stack depth.
    pub fn undo_depth(&self) -> usize {
        self.undo_stack.lock().expect("undo lock poisoned").len()
    }

    /// Returns the current redo stack depth.
    pub fn redo_depth(&self) -> usize {
        self.redo_stack.lock().expect("redo lock poisoned").len()
    }

    /// Performs an undo operation: pops from undo, applies reversal, pushes to redo.
    pub fn perform_undo(&self, ctx: &ExecutionContext) -> Result<(), CommandError> {
        let record = {
            let mut stack = self.undo_stack.lock().expect("undo lock poisoned");
            stack.pop()
        };

        match record {
            Some(record) => {
                record.undo(ctx)?;
                let mut redo = self.redo_stack.lock().expect("redo lock poisoned");
                redo.push(record);
                Ok(())
            }
            None => Err(CommandError::UndoFailed {
                id: "edit.undo".to_string(),
                description: "nothing to undo".to_string(),
            }),
        }
    }

    /// Performs a redo operation: pops from redo, applies, pushes to undo.
    pub fn perform_redo(&self, ctx: &ExecutionContext) -> Result<(), CommandError> {
        let record = {
            let mut stack = self.redo_stack.lock().expect("redo lock poisoned");
            stack.pop()
        };

        match record {
            Some(record) => {
                record.redo(ctx)?;
                let mut undo = self.undo_stack.lock().expect("undo lock poisoned");
                undo.push(record);
                Ok(())
            }
            None => Err(CommandError::RedoFailed {
                id: "edit.redo".to_string(),
                description: "nothing to redo".to_string(),
            }),
        }
    }
}

impl Default for DefaultUndoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoManager for DefaultUndoManager {
    fn push_undo(&self, record: Box<dyn UndoRecord>) {
        let mut stack = self.undo_stack.lock().expect("undo lock poisoned");
        stack.push(record);
    }

    fn pop_undo(&self) -> Option<Box<dyn UndoRecord>> {
        let mut stack = self.undo_stack.lock().expect("undo lock poisoned");
        stack.pop()
    }

    fn push_redo(&self, record: Box<dyn UndoRecord>) {
        let mut stack = self.redo_stack.lock().expect("redo lock poisoned");
        stack.push(record);
    }

    fn pop_redo(&self) -> Option<Box<dyn UndoRecord>> {
        let mut stack = self.redo_stack.lock().expect("redo lock poisoned");
        stack.pop()
    }

    fn clear_redo(&self) {
        let mut stack = self.redo_stack.lock().expect("redo lock poisoned");
        stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::CommandId;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[derive(Debug)]
    struct CountingUndoRecord {
        undo_count: Arc<AtomicUsize>,
        redo_count: Arc<AtomicUsize>,
        cmd_id: CommandId,
    }

    impl UndoRecord for CountingUndoRecord {
        fn undo(&self, _ctx: &ExecutionContext) -> Result<(), CommandError> {
            self.undo_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn redo(&self, _ctx: &ExecutionContext) -> Result<(), CommandError> {
            self.redo_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn description(&self) -> &str {
            "test record"
        }

        fn command_id(&self) -> &CommandId {
            &self.cmd_id
        }
    }

    fn make_record() -> (Box<dyn UndoRecord>, Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let undo_count = Arc::new(AtomicUsize::new(0));
        let redo_count = Arc::new(AtomicUsize::new(0));
        let record = CountingUndoRecord {
            undo_count: undo_count.clone(),
            redo_count: redo_count.clone(),
            cmd_id: CommandId::new("edit.insert").unwrap(),
        };
        (Box::new(record), undo_count, redo_count)
    }

    // Validates: Requirement 4.2
    #[test]
    fn push_undo_increases_stack_depth() {
        let manager = DefaultUndoManager::new();
        let (record, _, _) = make_record();
        manager.push_undo(record);
        assert_eq!(manager.undo_depth(), 1);
    }

    // Validates: Requirement 4.5
    #[test]
    fn perform_undo_pops_and_applies_record() {
        let manager = DefaultUndoManager::new();
        let (record, undo_count, _) = make_record();
        manager.push_undo(record);

        let ctx = ExecutionContext::empty();
        manager.perform_undo(&ctx).unwrap();

        assert_eq!(undo_count.load(Ordering::SeqCst), 1);
        assert_eq!(manager.undo_depth(), 0);
        assert_eq!(manager.redo_depth(), 1);
    }

    // Validates: Requirement 4.6
    #[test]
    fn perform_redo_pops_and_applies_record() {
        let manager = DefaultUndoManager::new();
        let (record, _, redo_count) = make_record();
        manager.push_undo(record);

        let ctx = ExecutionContext::empty();
        manager.perform_undo(&ctx).unwrap();
        manager.perform_redo(&ctx).unwrap();

        assert_eq!(redo_count.load(Ordering::SeqCst), 1);
        assert_eq!(manager.undo_depth(), 1);
        assert_eq!(manager.redo_depth(), 0);
    }

    // Validates: Requirement 4.7
    #[test]
    fn clear_redo_empties_redo_stack() {
        let manager = DefaultUndoManager::new();
        let (record, _, _) = make_record();
        manager.push_undo(record);

        let ctx = ExecutionContext::empty();
        manager.perform_undo(&ctx).unwrap();
        assert_eq!(manager.redo_depth(), 1);

        manager.clear_redo();
        assert_eq!(manager.redo_depth(), 0);
    }

    // Validates: Requirement 4.5
    #[test]
    fn undo_on_empty_stack_returns_error() {
        let manager = DefaultUndoManager::new();
        let ctx = ExecutionContext::empty();
        let result = manager.perform_undo(&ctx);
        assert!(result.is_err());
    }

    // Validates: Requirement 4.6
    #[test]
    fn redo_on_empty_stack_returns_error() {
        let manager = DefaultUndoManager::new();
        let ctx = ExecutionContext::empty();
        let result = manager.perform_redo(&ctx);
        assert!(result.is_err());
    }

    // Validates: Requirement 4.2, 4.5 — LIFO semantics
    #[test]
    fn undo_stack_has_lifo_semantics() {
        let manager = DefaultUndoManager::new();
        let undo1 = Arc::new(AtomicUsize::new(0));
        let undo2 = Arc::new(AtomicUsize::new(0));

        let record1: Box<dyn UndoRecord> = Box::new(CountingUndoRecord {
            undo_count: undo1.clone(),
            redo_count: Arc::new(AtomicUsize::new(0)),
            cmd_id: CommandId::new("edit.insert").unwrap(),
        });
        let record2: Box<dyn UndoRecord> = Box::new(CountingUndoRecord {
            undo_count: undo2.clone(),
            redo_count: Arc::new(AtomicUsize::new(0)),
            cmd_id: CommandId::new("edit.delete").unwrap(),
        });

        manager.push_undo(record1);
        manager.push_undo(record2);

        let ctx = ExecutionContext::empty();
        manager.perform_undo(&ctx).unwrap();
        // Record2 should have been undone (most recent)
        assert_eq!(undo2.load(Ordering::SeqCst), 1);
        assert_eq!(undo1.load(Ordering::SeqCst), 0);
    }
}
