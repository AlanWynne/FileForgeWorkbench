//! Save point tracking for modification state management.
//!
//! Tracks whether the document has unsaved modifications by comparing
//! the current undo position to a saved marker.

/// Tracks save-point state for a document.
#[derive(Debug, Clone)]
pub struct SavePointTracker {
    /// The undo position at which the document was last saved.
    save_point: Option<u64>,
    /// Current undo position counter (increments on each mutation).
    undo_position: u64,
}

impl SavePointTracker {
    /// Create a new tracker at initial state (no save point set).
    pub fn new() -> Self {
        Self {
            save_point: None,
            undo_position: 0,
        }
    }

    /// Record the current undo position as the save point.
    pub fn set_save_point(&mut self) {
        self.save_point = Some(self.undo_position);
    }

    /// Check if at save point (no unsaved modifications).
    pub fn is_at_save_point(&self) -> bool {
        self.save_point == Some(self.undo_position)
    }

    /// Record that a mutation occurred (increments undo position).
    /// Returns true if we just transitioned away from the save point.
    pub fn record_mutation(&mut self) -> bool {
        let was_at_save_point = self.is_at_save_point();
        self.undo_position += 1;
        was_at_save_point && !self.is_at_save_point()
    }

    /// Get the current undo position.
    pub fn undo_position(&self) -> u64 {
        self.undo_position
    }

    /// Set the undo position directly (for undo/redo integration).
    /// Returns true if we transitioned to/from the save point.
    pub fn set_undo_position(&mut self, position: u64) -> Option<bool> {
        let was_at = self.is_at_save_point();
        self.undo_position = position;
        let is_at = self.is_at_save_point();
        if was_at != is_at {
            Some(is_at)
        } else {
            None
        }
    }
}

impl Default for SavePointTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tracker_not_at_save_point() {
        let tracker = SavePointTracker::new();
        assert!(!tracker.is_at_save_point());
    }

    #[test]
    fn set_save_point_marks_current_position() {
        let mut tracker = SavePointTracker::new();
        tracker.set_save_point();
        assert!(tracker.is_at_save_point());
    }

    #[test]
    fn mutation_moves_away_from_save_point() {
        let mut tracker = SavePointTracker::new();
        tracker.set_save_point();
        assert!(tracker.is_at_save_point());

        let transitioned = tracker.record_mutation();
        assert!(transitioned);
        assert!(!tracker.is_at_save_point());
    }

    #[test]
    fn multiple_mutations_stay_away() {
        let mut tracker = SavePointTracker::new();
        tracker.set_save_point();
        tracker.record_mutation();
        let transitioned = tracker.record_mutation();
        assert!(!transitioned); // already away
        assert!(!tracker.is_at_save_point());
    }

    #[test]
    fn set_undo_position_can_return_to_save_point() {
        let mut tracker = SavePointTracker::new();
        tracker.set_save_point(); // save point at 0
        tracker.record_mutation(); // undo_position = 1
        assert!(!tracker.is_at_save_point());

        let transition = tracker.set_undo_position(0);
        assert_eq!(transition, Some(true)); // returned to save point
        assert!(tracker.is_at_save_point());
    }

    #[test]
    fn set_save_point_after_mutations() {
        let mut tracker = SavePointTracker::new();
        tracker.record_mutation();
        tracker.record_mutation();
        assert!(!tracker.is_at_save_point());

        tracker.set_save_point();
        assert!(tracker.is_at_save_point());
    }
}
