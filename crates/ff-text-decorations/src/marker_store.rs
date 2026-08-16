//! Per-document storage of line marker assignments.
//!
//! Manages bitmask-based marker tracking for document lines,
//! with support for line insertion/deletion position adjustments.

use std::collections::BTreeMap;

use crate::{MarkerMask, MarkerNumber};

/// Per-document storage of line marker assignments.
///
/// Addresses: Requirement 9 AC 7–10
pub struct MarkerStore {
    /// Map from document line number to marker bitmask.
    /// Lines without markers are not present in the map.
    markers: BTreeMap<u64, MarkerMask>,
    /// Total line count (for bounds checking).
    line_count: u64,
}

impl MarkerStore {
    /// Create a new MarkerStore for a document with the given line count.
    pub fn new(line_count: u64) -> Self {
        Self {
            markers: BTreeMap::new(),
            line_count,
        }
    }

    /// Add a marker to a line.
    ///
    /// Addresses: Requirement 9 AC 7
    pub fn marker_add(&mut self, line: u64, marker: MarkerNumber) {
        let mask = self.markers.entry(line).or_default();
        mask.set(marker);
    }

    /// Remove a marker from a line.
    ///
    /// Addresses: Requirement 9 AC 7
    pub fn marker_delete(&mut self, line: u64, marker: MarkerNumber) {
        if let Some(mask) = self.markers.get_mut(&line) {
            mask.clear(marker);
            if mask.is_empty() {
                self.markers.remove(&line);
            }
        }
    }

    /// Delete all markers with the given number from all lines.
    pub fn marker_delete_all(&mut self, marker: MarkerNumber) {
        let mut empty_lines = Vec::new();
        for (&line, mask) in self.markers.iter_mut() {
            mask.clear(marker);
            if mask.is_empty() {
                empty_lines.push(line);
            }
        }
        for line in empty_lines {
            self.markers.remove(&line);
        }
    }

    /// Get the marker bitmask for a line.
    ///
    /// Addresses: Requirement 9 AC 7
    pub fn marker_get(&self, line: u64) -> MarkerMask {
        self.markers.get(&line).copied().unwrap_or_default()
    }

    /// Find the next line at or after `from_line` with any marker in `mask`.
    ///
    /// Addresses: Requirement 9 AC 8
    pub fn marker_next(&self, from_line: u64, mask: MarkerMask) -> Option<u64> {
        // Search from from_line to end
        for (&line, line_mask) in self.markers.range(from_line..) {
            if line_mask.0 & mask.0 != 0 {
                return Some(line);
            }
        }
        // Wrap around: search from beginning
        for (&line, line_mask) in self.markers.range(..from_line) {
            if line_mask.0 & mask.0 != 0 {
                return Some(line);
            }
        }
        None
    }

    /// Find the previous line at or before `from_line` with any marker in `mask`.
    ///
    /// Addresses: Requirement 9 AC 9
    pub fn marker_previous(&self, from_line: u64, mask: MarkerMask) -> Option<u64> {
        // Search from from_line backwards to beginning
        for (&line, line_mask) in self.markers.range(..=from_line).rev() {
            if line_mask.0 & mask.0 != 0 {
                return Some(line);
            }
        }
        // Wrap around: search from end backwards
        if from_line < self.line_count {
            for (&line, line_mask) in self.markers.range((from_line + 1)..).rev() {
                if line_mask.0 & mask.0 != 0 {
                    return Some(line);
                }
            }
        }
        None
    }

    /// Shift all markers on lines >= `from_line` by `count` lines (for line insertion).
    ///
    /// Addresses: Requirement 9 AC 10
    pub fn lines_inserted(&mut self, from_line: u64, count: u64) {
        // Collect markers that need to move
        let to_move: Vec<(u64, MarkerMask)> = self
            .markers
            .range(from_line..)
            .map(|(&line, &mask)| (line, mask))
            .collect();

        // Remove old positions
        for &(line, _) in &to_move {
            self.markers.remove(&line);
        }

        // Re-insert at shifted positions
        for (line, mask) in to_move {
            self.markers.insert(line + count, mask);
        }

        self.line_count += count;
    }

    /// Remove markers on deleted lines and shift subsequent lines.
    ///
    /// Addresses: Requirement 9 AC 10
    pub fn lines_deleted(&mut self, from_line: u64, count: u64) {
        let delete_end = from_line + count;

        // Remove markers on deleted lines
        let deleted: Vec<u64> = self
            .markers
            .range(from_line..delete_end)
            .map(|(&line, _)| line)
            .collect();
        for line in deleted {
            self.markers.remove(&line);
        }

        // Shift markers after the deleted range
        let to_move: Vec<(u64, MarkerMask)> = self
            .markers
            .range(delete_end..)
            .map(|(&line, &mask)| (line, mask))
            .collect();

        for &(line, _) in &to_move {
            self.markers.remove(&line);
        }

        for (line, mask) in to_move {
            self.markers.insert(line - count, mask);
        }

        self.line_count = self.line_count.saturating_sub(count);
    }

    /// Query all lines with a specific marker.
    ///
    /// Addresses: Requirement 8 AC 5
    pub fn all_lines_with_marker(&self, marker: MarkerNumber) -> Vec<u64> {
        self.markers
            .iter()
            .filter(|(_, mask)| mask.has(marker))
            .map(|(&line, _)| line)
            .collect()
    }

    /// Clear all markers on all lines.
    pub fn clear_all(&mut self) {
        self.markers.clear();
    }

    /// Get the current line count.
    pub fn line_count(&self) -> u64 {
        self.line_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_store_has_no_markers() {
        let store = MarkerStore::new(100);
        assert!(store.marker_get(0).is_empty());
        assert!(store.marker_get(50).is_empty());
    }

    #[test]
    fn marker_add_and_get() {
        // Validates: Requirement 9 AC 7
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(5).unwrap();
        store.marker_add(10, marker);
        assert!(store.marker_get(10).has(marker));
        assert!(!store.marker_get(11).has(marker));
    }

    #[test]
    fn marker_delete_removes_marker() {
        // Validates: Requirement 9 AC 7
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(5).unwrap();
        store.marker_add(10, marker);
        store.marker_delete(10, marker);
        assert!(!store.marker_get(10).has(marker));
    }

    #[test]
    fn marker_delete_all_removes_from_all_lines() {
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(3).unwrap();
        store.marker_add(5, marker);
        store.marker_add(15, marker);
        store.marker_add(25, marker);
        store.marker_delete_all(marker);
        assert!(store.all_lines_with_marker(marker).is_empty());
    }

    #[test]
    fn marker_next_finds_forward() {
        // Validates: Requirement 9 AC 8
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(2).unwrap();
        store.marker_add(20, marker);
        store.marker_add(40, marker);
        let mask = MarkerMask(1 << 2);
        assert_eq!(store.marker_next(10, mask), Some(20));
        assert_eq!(store.marker_next(20, mask), Some(20));
        assert_eq!(store.marker_next(21, mask), Some(40));
    }

    #[test]
    fn marker_next_wraps_around() {
        // Validates: Property 8 — bookmark wrapping
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(0).unwrap();
        store.marker_add(10, marker);
        store.marker_add(50, marker);
        let mask = MarkerMask(1 << 0);
        // From line 60, should wrap to line 10
        assert_eq!(store.marker_next(60, mask), Some(10));
    }

    #[test]
    fn marker_previous_finds_backward() {
        // Validates: Requirement 9 AC 9
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(2).unwrap();
        store.marker_add(20, marker);
        store.marker_add(40, marker);
        let mask = MarkerMask(1 << 2);
        assert_eq!(store.marker_previous(50, mask), Some(40));
        assert_eq!(store.marker_previous(40, mask), Some(40));
        assert_eq!(store.marker_previous(39, mask), Some(20));
    }

    #[test]
    fn marker_previous_wraps_around() {
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(0).unwrap();
        store.marker_add(50, marker);
        store.marker_add(80, marker);
        let mask = MarkerMask(1 << 0);
        // From line 10, should wrap to line 80
        assert_eq!(store.marker_previous(10, mask), Some(80));
    }

    #[test]
    fn lines_inserted_shifts_markers() {
        // Validates: Requirement 9 AC 10, Property 6
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(1).unwrap();
        store.marker_add(10, marker);
        store.marker_add(20, marker);
        store.marker_add(30, marker);
        store.lines_inserted(15, 5);
        // Line 10 unchanged (before insertion)
        assert!(store.marker_get(10).has(marker));
        // Lines 20 and 30 shifted by 5
        assert!(store.marker_get(25).has(marker));
        assert!(store.marker_get(35).has(marker));
        // Old positions cleared
        assert!(!store.marker_get(20).has(marker));
        assert!(!store.marker_get(30).has(marker));
    }

    #[test]
    fn lines_deleted_removes_and_shifts() {
        // Validates: Requirement 9 AC 10
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(1).unwrap();
        store.marker_add(10, marker);
        store.marker_add(15, marker); // will be deleted
        store.marker_add(30, marker);
        store.lines_deleted(12, 5); // delete lines 12..17
                                    // Line 10 unchanged
        assert!(store.marker_get(10).has(marker));
        // Line 15 was in deleted range — gone
        assert!(!store.marker_get(15).has(marker));
        // Line 30 shifted down by 5
        assert!(store.marker_get(25).has(marker));
    }

    #[test]
    fn clear_all_removes_everything() {
        let mut store = MarkerStore::new(100);
        let m1 = MarkerNumber::new(0).unwrap();
        let m2 = MarkerNumber::new(5).unwrap();
        store.marker_add(10, m1);
        store.marker_add(20, m2);
        store.clear_all();
        assert!(store.marker_get(10).is_empty());
        assert!(store.marker_get(20).is_empty());
    }

    #[test]
    fn all_lines_with_marker_returns_sorted_lines() {
        // Validates: Requirement 8 AC 5
        let mut store = MarkerStore::new(100);
        let marker = MarkerNumber::new(0).unwrap();
        store.marker_add(30, marker);
        store.marker_add(10, marker);
        store.marker_add(50, marker);
        let lines = store.all_lines_with_marker(marker);
        assert_eq!(lines, vec![10, 30, 50]);
    }
}
