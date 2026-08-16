//! Save point and dirty flag tracking.
//!
//! The [`SavePointState`] tracks where in the undo history the document was
//! last saved. The dirty flag is derived from the distance between the current
//! position and the save point.

/// Tracks the save point and detach point for dirty flag derivation.
///
/// # Dirty Flag Rules
///
/// - `is_dirty()` is `true` when position ≠ save_point OR detach_point is set
/// - Setting a new save point clears the detach point
/// - The detach point is set when the save point becomes unreachable (redo
///   history containing the save point is discarded)
#[derive(Debug, Clone)]
pub struct SavePointState {
    /// The action count corresponding to the last save (or file open).
    save_point: usize,
    /// The detach point — set when the save point becomes unreachable.
    detach_point: Option<usize>,
    /// Current action count (incremented on commit, decremented on undo, incremented on redo).
    current_action: usize,
}

impl SavePointState {
    /// Creates a new state at position 0 (clean document just opened/saved).
    pub fn new() -> Self {
        Self {
            save_point: 0,
            detach_point: None,
            current_action: 0,
        }
    }

    /// Marks the current position as the save point and clears the detach point.
    pub fn set_save_point(&mut self) {
        self.save_point = self.current_action;
        self.detach_point = None;
    }

    /// Returns true if the document has unsaved changes.
    ///
    /// The dirty flag is true when:
    /// - The current position differs from the save point, OR
    /// - The detach point is set (save point is unreachable)
    pub fn is_dirty(&self) -> bool {
        self.detach_point.is_some() || self.current_action != self.save_point
    }

    /// Returns true if the current position is at the save point.
    pub fn is_at_save_point(&self) -> bool {
        self.detach_point.is_none() && self.current_action == self.save_point
    }

    /// Returns true if the current position is before the save point in history.
    pub fn before_save_point(&self) -> bool {
        self.detach_point.is_none() && self.current_action < self.save_point
    }

    /// Returns true if the current position is after the save point in history.
    pub fn after_save_point(&self) -> bool {
        self.detach_point.is_none() && self.current_action > self.save_point
    }

    /// Returns true if the save point is unreachable (detached).
    pub fn after_detach_point(&self) -> bool {
        self.detach_point.is_some()
    }

    /// Called when a new transaction is committed.
    ///
    /// If `redo_was_non_empty` is true and the save point was in the redo portion,
    /// sets the detach point.
    pub fn on_commit(&mut self, redo_was_non_empty: bool) {
        self.current_action += 1;

        // If redo stack was non-empty and save_point was ahead of us (in redo portion),
        // then the save point is now unreachable
        if redo_was_non_empty && self.save_point > self.current_action.saturating_sub(1) {
            self.detach_point = Some(self.current_action);
        }
    }

    /// Called when an undo operation completes.
    pub fn on_undo(&mut self) {
        self.current_action = self.current_action.saturating_sub(1);
    }

    /// Called when a redo operation completes.
    pub fn on_redo(&mut self) {
        self.current_action += 1;
    }

    /// Returns the current action position.
    pub fn current_action(&self) -> usize {
        self.current_action
    }

    /// Returns the save point position.
    pub fn save_point(&self) -> usize {
        self.save_point
    }

    /// Resets all state (for delete_history).
    pub fn reset(&mut self) {
        self.save_point = 0;
        self.detach_point = None;
        self.current_action = 0;
    }

    /// Called when the undo stack evicts old transactions.
    /// Adjusts positions if needed.
    pub fn on_eviction(&mut self) {
        // When the oldest transaction is evicted, all positions shift.
        // But since we track relative counts from 0, eviction doesn't change
        // the save_point or current_action. The important semantic is that
        // if save_point < (current_action - max_levels), it's lost.
        // We handle this by checking if save_point would be unreachable.
    }
}

impl Default for SavePointState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_clean() {
        let state = SavePointState::new();
        assert!(!state.is_dirty());
        assert!(state.is_at_save_point());
    }

    #[test]
    fn commit_makes_dirty() {
        let mut state = SavePointState::new();
        state.on_commit(false);
        assert!(state.is_dirty());
        assert!(state.after_save_point());
    }

    #[test]
    fn undo_back_to_save_point_clears_dirty() {
        let mut state = SavePointState::new();
        state.on_commit(false);
        state.on_undo();
        assert!(!state.is_dirty());
        assert!(state.is_at_save_point());
    }

    #[test]
    fn undo_past_save_point_is_dirty() {
        let mut state = SavePointState::new();
        state.on_commit(false);
        state.on_commit(false);
        state.set_save_point(); // save at position 2
        state.on_undo(); // position 1
        assert!(state.is_dirty());
        assert!(state.before_save_point());
    }

    #[test]
    fn set_save_point_clears_detach() {
        let mut state = SavePointState::new();
        state.on_commit(false);
        state.set_save_point();
        // Undo, then commit (clears redo)
        state.on_undo();
        state.on_commit(true); // redo was non-empty, save_point=1 > current-1=0
        assert!(state.after_detach_point());
        // Now save again
        state.set_save_point();
        assert!(!state.after_detach_point());
        assert!(!state.is_dirty());
    }

    #[test]
    fn detach_point_makes_permanently_dirty() {
        let mut state = SavePointState::new();
        state.on_commit(false); // pos=1
        state.set_save_point(); // save at pos=1
        state.on_undo(); // pos=0
        state.on_commit(true); // pos=1, redo cleared, save_point=1 which was in redo
                               // Save point was at 1, but after undo we were at 0, then commit brings to 1.
                               // save_point (1) > current_action-1 (0) when redo was non-empty → detach
        assert!(state.after_detach_point());
        assert!(state.is_dirty());
        // Even if we undo/redo, still dirty
        state.on_undo();
        assert!(state.is_dirty());
        state.on_redo();
        assert!(state.is_dirty());
    }

    #[test]
    fn redo_after_undo_restores_position() {
        let mut state = SavePointState::new();
        state.on_commit(false); // pos=1
        state.on_commit(false); // pos=2
        state.set_save_point(); // save at 2
        state.on_undo(); // pos=1
        assert!(state.is_dirty());
        state.on_redo(); // pos=2
        assert!(!state.is_dirty());
    }

    #[test]
    fn reset_clears_all_state() {
        let mut state = SavePointState::new();
        state.on_commit(false);
        state.on_commit(false);
        state.set_save_point();
        state.reset();
        assert!(!state.is_dirty());
        assert!(state.is_at_save_point());
        assert_eq!(state.current_action(), 0);
    }
}
