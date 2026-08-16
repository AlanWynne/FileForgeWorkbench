//! Modified byte tracking.
//!
//! Tracks which byte offsets have been modified since the last save.
//! The indicator reflects whether the current value differs from the
//! last-saved value, correctly handling edit/undo/redo cycles.

use std::collections::BTreeSet;

/// Tracks which byte offsets have been modified since the last save.
///
/// The tracker maintains a set of byte offsets where the current
/// buffer value differs from the last-saved state.
#[derive(Debug, Clone)]
pub struct ModifiedByteTracker {
    /// Set of byte offsets that differ from the last-saved state.
    modified_offsets: BTreeSet<u64>,
}

impl Default for ModifiedByteTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ModifiedByteTracker {
    /// Create a new empty tracker (no bytes modified).
    pub fn new() -> Self {
        Self {
            modified_offsets: BTreeSet::new(),
        }
    }

    /// Mark a byte offset as modified.
    pub fn mark_modified(&mut self, offset: u64) {
        self.modified_offsets.insert(offset);
    }

    /// Check if a byte is currently marked as modified.
    pub fn is_modified(&self, offset: u64) -> bool {
        self.modified_offsets.contains(&offset)
    }

    /// Remove the modified indicator for a byte offset.
    ///
    /// Called when undo restores a byte to its saved value.
    pub fn mark_restored(&mut self, offset: u64) {
        self.modified_offsets.remove(&offset);
    }

    /// Clear all modified indicators.
    ///
    /// Called on document save — all indicators removed since the
    /// saved state now matches the buffer.
    pub fn on_save(&mut self) {
        self.modified_offsets.clear();
    }

    /// Get all modified offsets within a byte range (for rendering).
    ///
    /// Returns offsets in the range `[start, end)`.
    pub fn modified_in_range(&self, start: u64, end: u64) -> Vec<u64> {
        self.modified_offsets.range(start..end).copied().collect()
    }

    /// Recalculate modification state for a byte after undo/redo.
    ///
    /// Compares current value against saved value: if they match,
    /// the modified indicator is removed; if they differ, it is added.
    pub fn recalculate(&mut self, offset: u64, current_value: u8, saved_value: u8) {
        if current_value == saved_value {
            self.modified_offsets.remove(&offset);
        } else {
            self.modified_offsets.insert(offset);
        }
    }

    /// Get the count of modified bytes.
    pub fn modified_count(&self) -> usize {
        self.modified_offsets.len()
    }

    /// Check if any bytes are modified.
    pub fn has_modifications(&self) -> bool {
        !self.modified_offsets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 8 AC 1-2
    #[test]
    fn mark_modified_adds_offset_to_set() {
        let mut tracker = ModifiedByteTracker::new();
        assert!(!tracker.is_modified(10));

        tracker.mark_modified(10);
        assert!(tracker.is_modified(10));
        assert!(!tracker.is_modified(11));
    }

    // Validates: Requirement 8 AC 3
    #[test]
    fn on_save_clears_all_modified_indicators() {
        let mut tracker = ModifiedByteTracker::new();
        tracker.mark_modified(5);
        tracker.mark_modified(10);
        tracker.mark_modified(15);
        assert_eq!(tracker.modified_count(), 3);

        tracker.on_save();
        assert_eq!(tracker.modified_count(), 0);
        assert!(!tracker.is_modified(5));
        assert!(!tracker.is_modified(10));
        assert!(!tracker.is_modified(15));
    }

    // Validates: Requirement 8 AC 4
    #[test]
    fn mark_restored_removes_single_offset() {
        let mut tracker = ModifiedByteTracker::new();
        tracker.mark_modified(5);
        tracker.mark_modified(10);

        tracker.mark_restored(5);
        assert!(!tracker.is_modified(5));
        assert!(tracker.is_modified(10));
    }

    // Validates: Requirement 8 AC 5
    #[test]
    fn recalculate_adds_or_removes_based_on_value_comparison() {
        let mut tracker = ModifiedByteTracker::new();

        // Byte differs from saved → mark as modified
        tracker.recalculate(10, 0xAA, 0xBB);
        assert!(tracker.is_modified(10));

        // Byte matches saved → remove indicator
        tracker.recalculate(10, 0xBB, 0xBB);
        assert!(!tracker.is_modified(10));
    }

    // Validates: Requirement 8 AC 5
    #[test]
    fn multi_modify_undo_cycle_tracks_correctly() {
        let mut tracker = ModifiedByteTracker::new();
        let saved_value = 0x41;

        // Edit: change from saved value
        tracker.recalculate(0, 0x42, saved_value);
        assert!(tracker.is_modified(0));

        // Undo: restore to saved value
        tracker.recalculate(0, saved_value, saved_value);
        assert!(!tracker.is_modified(0));

        // Re-edit: change again
        tracker.recalculate(0, 0x43, saved_value);
        assert!(tracker.is_modified(0));

        // Undo again: restore
        tracker.recalculate(0, saved_value, saved_value);
        assert!(!tracker.is_modified(0));
    }

    // Validates: Requirement 8 AC 1-2
    #[test]
    fn modified_in_range_returns_offsets_within_bounds() {
        let mut tracker = ModifiedByteTracker::new();
        tracker.mark_modified(5);
        tracker.mark_modified(10);
        tracker.mark_modified(15);
        tracker.mark_modified(20);

        let range = tracker.modified_in_range(8, 18);
        assert_eq!(range, vec![10, 15]);
    }

    #[test]
    fn has_modifications_reflects_state() {
        let mut tracker = ModifiedByteTracker::new();
        assert!(!tracker.has_modifications());

        tracker.mark_modified(0);
        assert!(tracker.has_modifications());

        tracker.on_save();
        assert!(!tracker.has_modifications());
    }
}
