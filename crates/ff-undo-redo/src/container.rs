//! Container actions — plugin/extension state that participates in undo.
//!
//! Container actions allow plugins to interleave their own state changes with
//! document edits, so that undo/redo correctly reverses/re-applies plugin state.

use std::fmt::Debug;

/// Trait for plugin/extension state that participates in undo.
///
/// Implementors record enough state to reverse and re-apply a state change.
/// Container actions are interleaved with edit operations in a transaction
/// and are invoked in the correct order during undo/redo.
///
/// # Rules
///
/// - Container actions do NOT affect the dirty flag or modified line markers.
/// - Container actions participate in coalescing via `may_coalesce()`.
/// - On undo, container actions are invoked in reverse order.
/// - On redo, container actions are invoked in original order.
pub trait UndoableState: Send + Sync + Debug {
    /// Reverse this state change.
    fn undo(&self);

    /// Re-apply this state change.
    fn redo(&self);

    /// Human-readable description for diagnostics.
    fn description(&self) -> &str;

    /// Whether this container action may coalesce with adjacent actions.
    fn may_coalesce(&self) -> bool {
        false
    }
}

/// A stored container action with its position index within the transaction.
#[derive(Debug)]
pub struct ContainerActionEntry {
    /// Index within the transaction's operation list where this action sits.
    pub operation_index: usize,
    /// The undoable state object.
    pub action: Box<dyn UndoableState>,
}

impl Clone for ContainerActionEntry {
    fn clone(&self) -> Self {
        // Container actions are not clonable in general — this is a limitation.
        // In practice, transactions with container actions are not cloned.
        panic!("ContainerActionEntry cannot be cloned; transactions with container actions must not be cloned");
    }
}
