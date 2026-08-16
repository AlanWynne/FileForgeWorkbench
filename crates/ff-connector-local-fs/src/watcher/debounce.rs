//! Debounce logic for file watching events.
//!
//! Coalesces rapid successive events on the same path within a configurable
//! time window into a single event.
//!
//! Addresses: Requirement 3, criteria 5–6

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Debounce state tracker for a set of watched paths.
///
/// Records the last event time per path and determines whether a new event
/// should be emitted or suppressed.
///
/// Validates: Requirement 3 AC 5
pub struct DebounceTracker {
    /// Last event time per path.
    last_event_times: HashMap<PathBuf, Instant>,
    /// The debounce window duration.
    window: Duration,
}

impl DebounceTracker {
    /// Create a new debounce tracker with the given window.
    pub fn new(window: Duration) -> Self {
        Self {
            last_event_times: HashMap::new(),
            window,
        }
    }

    /// Check if an event on the given path should be emitted.
    ///
    /// Returns `true` if enough time has passed since the last event on this path,
    /// or if this is the first event for the path. Updates the tracker state.
    ///
    /// Validates: Requirement 3 AC 5
    pub fn should_emit(&mut self, path: &PathBuf) -> bool {
        let now = Instant::now();

        if let Some(last_time) = self.last_event_times.get(path) {
            if now.duration_since(*last_time) < self.window {
                return false;
            }
        }

        self.last_event_times.insert(path.clone(), now);
        true
    }

    /// Remove tracking state for a specific path.
    pub fn remove_path(&mut self, path: &PathBuf) {
        self.last_event_times.remove(path);
    }

    /// Clear all tracking state.
    pub fn clear(&mut self) {
        self.last_event_times.clear();
    }

    /// Returns the configured debounce window.
    pub fn window(&self) -> Duration {
        self.window
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn first_event_always_emits() {
        let mut tracker = DebounceTracker::new(Duration::from_millis(100));
        let path = PathBuf::from("/test/file.txt");
        assert!(tracker.should_emit(&path));
    }

    #[test]
    fn rapid_events_are_suppressed() {
        let mut tracker = DebounceTracker::new(Duration::from_millis(100));
        let path = PathBuf::from("/test/file.txt");

        assert!(tracker.should_emit(&path));
        assert!(!tracker.should_emit(&path)); // too fast
    }

    #[test]
    fn events_after_window_are_emitted() {
        let mut tracker = DebounceTracker::new(Duration::from_millis(10));
        let path = PathBuf::from("/test/file.txt");

        assert!(tracker.should_emit(&path));
        sleep(Duration::from_millis(15));
        assert!(tracker.should_emit(&path)); // enough time passed
    }

    #[test]
    fn different_paths_are_independent() {
        let mut tracker = DebounceTracker::new(Duration::from_millis(100));
        let path_a = PathBuf::from("/test/a.txt");
        let path_b = PathBuf::from("/test/b.txt");

        assert!(tracker.should_emit(&path_a));
        assert!(tracker.should_emit(&path_b)); // different path, should emit
        assert!(!tracker.should_emit(&path_a)); // same path, too fast
    }

    #[test]
    fn remove_path_allows_immediate_emit() {
        let mut tracker = DebounceTracker::new(Duration::from_millis(100));
        let path = PathBuf::from("/test/file.txt");

        assert!(tracker.should_emit(&path));
        assert!(!tracker.should_emit(&path));

        tracker.remove_path(&path);
        assert!(tracker.should_emit(&path)); // tracking was cleared
    }
}
