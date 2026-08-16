//! RefreshController — debounced change notification for viewer refresh.
//!
//! Manages debounced refresh notifications from document changes and VFS watch
//! events. Ensures viewers are not overwhelmed by rapid edits.

use std::time::{Duration, Instant};

/// Default debounce interval in milliseconds.
pub const DEFAULT_DEBOUNCE_MS: u64 = 300;

/// Manages debounced refresh notifications from document changes and VFS watch events.
///
/// After a document change, the controller waits for a configurable quiet period
/// before signaling that a refresh should fire. If additional changes arrive within
/// the quiet period, the timer resets.
pub struct RefreshController {
    /// Debounce interval.
    debounce: Duration,
    /// When the last change was recorded.
    last_change: Option<Instant>,
    /// Whether a refresh has already been dispatched for the current change batch.
    refresh_dispatched: bool,
    /// Whether a refresh is currently in-flight on a background task.
    refresh_in_flight: bool,
    /// Count of refresh calls that have been triggered.
    refresh_count: u64,
}

impl RefreshController {
    /// Create a new refresh controller with the given debounce interval.
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debounce: Duration::from_millis(debounce_ms),
            last_change: None,
            refresh_dispatched: false,
            refresh_in_flight: false,
            refresh_count: 0,
        }
    }

    /// Notify that the document has changed. Resets the debounce timer.
    ///
    /// After the quiet period elapses, `should_refresh()` will return `true`.
    pub fn notify_document_changed(&mut self) {
        self.last_change = Some(Instant::now());
        self.refresh_dispatched = false;
    }

    /// Notify that the file was modified externally (VFS watch event).
    ///
    /// Treated identically to a document change for debouncing purposes.
    pub fn notify_external_change(&mut self) {
        self.notify_document_changed();
    }

    /// Update the debounce interval (e.g., from config hot-reload).
    pub fn set_debounce_ms(&mut self, debounce_ms: u64) {
        self.debounce = Duration::from_millis(debounce_ms);
    }

    /// Returns the current debounce interval in milliseconds.
    pub fn debounce_ms(&self) -> u64 {
        self.debounce.as_millis() as u64
    }

    /// Check whether a refresh should fire (called each frame/tick).
    ///
    /// Returns `true` if the debounce period has elapsed since the last change
    /// and no refresh has been dispatched yet for this change batch.
    pub fn should_refresh(&mut self) -> bool {
        if self.refresh_dispatched {
            return false;
        }

        if let Some(last) = self.last_change {
            if last.elapsed() >= self.debounce {
                self.refresh_dispatched = true;
                self.refresh_count += 1;
                return true;
            }
        }

        false
    }

    /// Mark that a refresh is currently in-flight.
    pub fn set_refresh_in_flight(&mut self, in_flight: bool) {
        self.refresh_in_flight = in_flight;
    }

    /// Returns whether a refresh is currently in-flight.
    pub fn is_refresh_in_flight(&self) -> bool {
        self.refresh_in_flight
    }

    /// Returns the total number of refreshes triggered.
    pub fn refresh_count(&self) -> u64 {
        self.refresh_count
    }

    /// Reset the controller state (e.g., when viewer is deactivated).
    pub fn reset(&mut self) {
        self.last_change = None;
        self.refresh_dispatched = false;
        self.refresh_in_flight = false;
    }
}

impl Default for RefreshController {
    fn default() -> Self {
        Self::new(DEFAULT_DEBOUNCE_MS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn new_controller_does_not_fire_immediately() {
        // Validates: Requirement 9 AC 2
        let mut ctrl = RefreshController::new(300);
        assert!(!ctrl.should_refresh());
    }

    #[test]
    fn fires_after_debounce_period() {
        // Validates: Requirement 9 AC 1, AC 2
        let mut ctrl = RefreshController::new(10); // 10ms for fast test
        ctrl.notify_document_changed();

        // Should not fire immediately
        assert!(!ctrl.should_refresh());

        // Wait for debounce period
        thread::sleep(Duration::from_millis(15));
        assert!(ctrl.should_refresh());
    }

    #[test]
    fn does_not_fire_twice_for_same_change() {
        // Validates: Requirement 9 AC 2
        let mut ctrl = RefreshController::new(10);
        ctrl.notify_document_changed();

        thread::sleep(Duration::from_millis(15));
        assert!(ctrl.should_refresh());
        // Second check should not fire again
        assert!(!ctrl.should_refresh());
    }

    #[test]
    fn rapid_changes_reset_timer() {
        // Validates: Requirement 9 AC 2
        let mut ctrl = RefreshController::new(50);
        ctrl.notify_document_changed();
        thread::sleep(Duration::from_millis(20));

        // Another change arrives before debounce expires
        ctrl.notify_document_changed();
        thread::sleep(Duration::from_millis(20));

        // Still shouldn't fire — timer was reset
        assert!(!ctrl.should_refresh());

        // Wait for full debounce from last change
        thread::sleep(Duration::from_millis(35));
        assert!(ctrl.should_refresh());
    }

    #[test]
    fn external_change_triggers_same_as_document_change() {
        // Validates: Requirement 9 AC 4
        let mut ctrl = RefreshController::new(10);
        ctrl.notify_external_change();

        thread::sleep(Duration::from_millis(15));
        assert!(ctrl.should_refresh());
    }

    #[test]
    fn set_debounce_ms_updates_interval() {
        // Validates: Requirement 9 AC 3
        let mut ctrl = RefreshController::new(1000);
        ctrl.set_debounce_ms(10);
        assert_eq!(ctrl.debounce_ms(), 10);

        ctrl.notify_document_changed();
        thread::sleep(Duration::from_millis(15));
        assert!(ctrl.should_refresh());
    }

    #[test]
    fn reset_clears_pending_refresh() {
        let mut ctrl = RefreshController::new(10);
        ctrl.notify_document_changed();
        ctrl.reset();

        thread::sleep(Duration::from_millis(15));
        assert!(!ctrl.should_refresh());
    }

    #[test]
    fn refresh_count_increments_on_each_fire() {
        let mut ctrl = RefreshController::new(10);
        assert_eq!(ctrl.refresh_count(), 0);

        ctrl.notify_document_changed();
        thread::sleep(Duration::from_millis(15));
        ctrl.should_refresh();
        assert_eq!(ctrl.refresh_count(), 1);

        ctrl.notify_document_changed();
        thread::sleep(Duration::from_millis(15));
        ctrl.should_refresh();
        assert_eq!(ctrl.refresh_count(), 2);
    }
}
