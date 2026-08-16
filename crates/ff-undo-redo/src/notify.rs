//! Notification traits for undo state changes.
//!
//! The [`UndoNotifier`] trait allows external consumers (GUI shell, status bar)
//! to react to undo system state changes without coupling to the implementation.

/// Unique identifier for a registered listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(pub(crate) u64);

/// Notification trait for undo state changes.
///
/// Implemented by the GUI shell or other consumers that need to react
/// to undo system events (dirty flag changes, availability changes, etc.).
///
/// All methods have default no-op implementations so consumers only need
/// to override the events they care about.
pub trait UndoNotifier: Send + Sync {
    /// Called when the dirty flag changes.
    fn dirty_flag_changed(&self, _is_dirty: bool) {}

    /// Called when undo availability changes.
    fn undo_available_changed(&self, _available: bool) {}

    /// Called when redo availability changes.
    fn redo_available_changed(&self, _available: bool) {}

    /// Called when a transaction is committed.
    fn transaction_committed(&self, _name: &str) {}

    /// Called when a transaction is undone.
    fn transaction_undone(&self, _name: &str) {}

    /// Called when a transaction is redone.
    fn transaction_redone(&self, _name: &str) {}
}
