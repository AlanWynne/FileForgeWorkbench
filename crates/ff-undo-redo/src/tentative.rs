//! Tentative actions — IME composition support.
//!
//! During IME composition, edits are tentative: they can be rolled back without
//! leaving a trace in the undo history if the user cancels composition, or
//! committed to become permanent undo history if composition completes.

/// Manages IME composition tentative actions.
///
/// # Lifecycle
///
/// 1. `start()` — enters tentative mode, records the tentative point
/// 2. Edit operations are recorded as tentative steps
/// 3. Either:
///    - `commit()` — tentative actions become permanent history
///    - `rollback()` — tentative actions are reversed, no history trace
///
/// # Coalescing Interaction
///
/// The tentative point acts as a coalescing barrier — coalescing does not
/// cross the tentative boundary.
#[derive(Debug, Clone)]
pub struct TentativeState {
    /// Whether tentative mode is active.
    active: bool,
    /// The action index where tentative mode began.
    tentative_point: Option<usize>,
    /// Number of tentative steps since the tentative point.
    step_count: usize,
}

impl TentativeState {
    /// Creates a new inactive tentative state.
    pub fn new() -> Self {
        Self {
            active: false,
            tentative_point: None,
            step_count: 0,
        }
    }

    /// Enters tentative mode at the given action index.
    ///
    /// Returns `Err` if tentative mode is already active.
    #[allow(clippy::result_unit_err)]
    pub fn start(&mut self, current_action_index: usize) -> Result<(), ()> {
        if self.active {
            return Err(());
        }
        self.active = true;
        self.tentative_point = Some(current_action_index);
        self.step_count = 0;
        Ok(())
    }

    /// Commits tentative actions — they become permanent history.
    ///
    /// Clears the tentative point. Returns the number of steps committed.
    #[allow(clippy::result_unit_err)]
    pub fn commit(&mut self) -> Result<usize, ()> {
        if !self.active {
            return Err(());
        }
        let steps = self.step_count;
        self.active = false;
        self.tentative_point = None;
        self.step_count = 0;
        Ok(steps)
    }

    /// Prepares for rollback. Returns the tentative point and step count.
    ///
    /// The caller is responsible for actually reversing the operations.
    /// After calling this, the tentative state is reset.
    #[allow(clippy::result_unit_err)]
    pub fn rollback(&mut self) -> Result<(usize, usize), ()> {
        if !self.active {
            return Err(());
        }
        let point = self.tentative_point.unwrap_or(0);
        let steps = self.step_count;
        self.active = false;
        self.tentative_point = None;
        self.step_count = 0;
        Ok((point, steps))
    }

    /// Records a tentative step (an operation was added during tentative mode).
    pub fn record_step(&mut self) {
        if self.active {
            self.step_count += 1;
        }
    }

    /// Returns whether tentative mode is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns the tentative point (action index where tentative mode began).
    pub fn tentative_point(&self) -> Option<usize> {
        self.tentative_point
    }

    /// Returns the number of steps since the tentative point.
    pub fn steps(&self) -> Option<usize> {
        if self.active {
            Some(self.step_count)
        } else {
            None
        }
    }

    /// Resets all state (for delete_history).
    pub fn reset(&mut self) {
        self.active = false;
        self.tentative_point = None;
        self.step_count = 0;
    }
}

impl Default for TentativeState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_inactive() {
        let state = TentativeState::new();
        assert!(!state.is_active());
        assert_eq!(state.steps(), None);
        assert_eq!(state.tentative_point(), None);
    }

    #[test]
    fn start_activates_tentative_mode() {
        let mut state = TentativeState::new();
        assert!(state.start(5).is_ok());
        assert!(state.is_active());
        assert_eq!(state.tentative_point(), Some(5));
        assert_eq!(state.steps(), Some(0));
    }

    #[test]
    fn start_when_already_active_returns_error() {
        let mut state = TentativeState::new();
        state.start(0).unwrap();
        assert!(state.start(1).is_err());
    }

    #[test]
    fn record_step_increments_count() {
        let mut state = TentativeState::new();
        state.start(0).unwrap();
        state.record_step();
        state.record_step();
        assert_eq!(state.steps(), Some(2));
    }

    #[test]
    fn commit_returns_step_count_and_deactivates() {
        let mut state = TentativeState::new();
        state.start(3).unwrap();
        state.record_step();
        state.record_step();
        state.record_step();
        let steps = state.commit().unwrap();
        assert_eq!(steps, 3);
        assert!(!state.is_active());
    }

    #[test]
    fn commit_when_not_active_returns_error() {
        let mut state = TentativeState::new();
        assert!(state.commit().is_err());
    }

    #[test]
    fn rollback_returns_point_and_steps() {
        let mut state = TentativeState::new();
        state.start(10).unwrap();
        state.record_step();
        state.record_step();
        let (point, steps) = state.rollback().unwrap();
        assert_eq!(point, 10);
        assert_eq!(steps, 2);
        assert!(!state.is_active());
    }

    #[test]
    fn rollback_when_not_active_returns_error() {
        let mut state = TentativeState::new();
        assert!(state.rollback().is_err());
    }

    #[test]
    fn reset_clears_all_state() {
        let mut state = TentativeState::new();
        state.start(5).unwrap();
        state.record_step();
        state.reset();
        assert!(!state.is_active());
        assert_eq!(state.tentative_point(), None);
        assert_eq!(state.steps(), None);
    }
}
