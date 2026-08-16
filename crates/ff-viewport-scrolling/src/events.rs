//! Viewport state-change events and observer trait.
//!
//! After any mutation to the viewport or cursor state, a `ViewportChanged` event
//! is emitted to all registered observers. This enables status bar updates, GUI
//! re-renders, and other reactive systems to respond without polling.

/// Event emitted after any viewport state mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportChanged {
    /// New top_line value.
    pub top_line: u64,
    /// New cursor_line value.
    pub cursor_line: u64,
    /// New cursor_column value.
    pub cursor_column: u64,
    /// New horizontal_offset value.
    pub horizontal_offset: u64,
    /// Whether this change was triggered by a cursor move (vs. explicit scroll).
    pub cursor_triggered: bool,
}

/// Observer trait for viewport state changes.
pub trait ViewportObserver: Send + Sync {
    /// Called after any viewport state mutation.
    fn on_viewport_changed(&self, event: &ViewportChanged);
}
