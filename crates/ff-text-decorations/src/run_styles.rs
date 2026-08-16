//! Run-length encoded storage for indicator values.
//!
//! `RunStyles<T>` stores a sequence of values using (value, length) pairs,
//! providing O(log n) position lookup and efficient insert/delete operations.

use std::fmt::Debug;

/// A single run in the RLE storage: a contiguous range of positions with the same value.
///
/// Addresses: Requirement 3 AC 1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run<T: Clone + Eq> {
    /// The value for all positions in this run.
    pub value: T,
    /// The number of consecutive positions with this value.
    pub length: u64,
}

/// Generic run-length-encoded sequence supporting efficient position queries and edits.
///
/// Addresses: Requirement 3 AC 1, 10
pub struct RunStyles<T: Clone + Eq + Default + Debug> {
    /// Ordered sequence of runs; total of all lengths == document length.
    runs: Vec<Run<T>>,
    /// Cached cumulative lengths for O(log n) binary search.
    cumulative: Vec<u64>,
    /// Total length (sum of all run lengths).
    total_length: u64,
}

impl<T: Clone + Eq + Default + Debug> RunStyles<T> {
    /// Create storage for a document of the given initial length (all values = T::default()).
    pub fn new(initial_length: u64) -> Self {
        let runs = vec![Run {
            value: T::default(),
            length: initial_length,
        }];
        let cumulative = vec![initial_length];
        Self {
            runs,
            cumulative,
            total_length: initial_length,
        }
    }

    /// Get the value at the given position.
    /// O(log n) via binary search on cumulative lengths.
    pub fn value_at(&self, position: u64) -> T {
        if position >= self.total_length {
            return T::default();
        }
        let idx = self.find_run_index(position);
        self.runs[idx].value.clone()
    }

    /// Get the start position of the run containing `position`.
    pub fn run_start(&self, position: u64) -> u64 {
        if position >= self.total_length {
            return self.total_length;
        }
        let idx = self.find_run_index(position);
        if idx == 0 {
            0
        } else {
            self.cumulative[idx - 1]
        }
    }

    /// Get the end position (exclusive) of the run containing `position`.
    pub fn run_end(&self, position: u64) -> u64 {
        if position >= self.total_length {
            return self.total_length;
        }
        let idx = self.find_run_index(position);
        self.cumulative[idx]
    }

    /// Set all positions in [position, position+length) to `value`.
    /// Returns true if any values actually changed.
    /// Merges adjacent runs with the same value.
    pub fn fill_range(&mut self, position: u64, value: T, length: u64) -> bool {
        if length == 0 || position >= self.total_length {
            return false;
        }

        let end = (position + length).min(self.total_length);
        let actual_length = end - position;

        // Check if all positions already have this value
        let start_idx = self.find_run_index(position);
        let end_idx = self.find_run_index(end.saturating_sub(1));

        // Fast path: single run covering the whole range with same value
        if start_idx == end_idx && self.runs[start_idx].value == value {
            return false;
        }

        // Check if all runs in range already have the target value
        let all_same = (start_idx..=end_idx).all(|i| self.runs[i].value == value);
        if all_same {
            return false;
        }

        // Perform the fill: split runs at boundaries, replace middle runs
        self.split_at(end);
        self.split_at(position);

        // Find the range of runs to replace
        let fill_start = self.find_run_index(position);
        let fill_end = if end >= self.total_length {
            self.runs.len() - 1
        } else {
            self.find_run_index(end.saturating_sub(1))
        };

        // Replace the range with a single run
        self.runs.drain(fill_start..=fill_end);
        self.runs.insert(
            fill_start,
            Run {
                value,
                length: actual_length,
            },
        );

        // Merge adjacent runs with same value
        self.merge_adjacent(fill_start);

        self.rebuild_cumulative();
        true
    }

    /// Insert `length` positions with T::default() at `position`.
    /// Splits the run containing position; shifts subsequent runs rightward.
    ///
    /// Addresses: Requirement 4 AC 1, 3, 4
    pub fn insert_space(&mut self, position: u64, length: u64) {
        if length == 0 {
            return;
        }

        if position >= self.total_length {
            // Append at end
            let last = self.runs.len() - 1;
            if self.runs[last].value == T::default() {
                self.runs[last].length += length;
            } else {
                self.runs.push(Run {
                    value: T::default(),
                    length,
                });
            }
        } else {
            // Split at position and insert default run
            self.split_at(position);
            let idx = self.find_run_index_for_insert(position);

            // If the run at idx already has default value, just extend it
            if self.runs[idx].value == T::default() {
                self.runs[idx].length += length;
            } else {
                self.runs.insert(
                    idx,
                    Run {
                        value: T::default(),
                        length,
                    },
                );
            }
        }

        self.total_length += length;
        self.rebuild_cumulative();
    }

    /// Remove `length` positions starting at `position`.
    /// Merges the runs on either side of the deleted range.
    ///
    /// Addresses: Requirement 4 AC 2
    pub fn delete_range(&mut self, position: u64, length: u64) {
        if length == 0 || position >= self.total_length {
            return;
        }

        let delete_end = (position + length).min(self.total_length);
        let actual_length = delete_end - position;

        // Split at boundaries
        self.split_at(delete_end);
        self.split_at(position);

        // Find runs to remove
        let start_idx = self.find_run_index(position);
        let _end_idx = if delete_end >= self.total_length {
            self.runs.len() - 1
        } else {
            let idx = self.find_run_index(delete_end.saturating_sub(1));
            // After split_at(delete_end), the run at delete_end starts a new run
            // We need to find the last run that ends at or before delete_end
            idx
        };

        // Remove runs in the deleted range
        // After splitting, the runs from start_idx should cover exactly [position, delete_end)
        let mut removed = 0u64;
        let mut remove_end = start_idx;
        while remove_end < self.runs.len() && removed < actual_length {
            let run_start_pos = if remove_end == 0 {
                0
            } else {
                self.cumulative[remove_end - 1]
            };
            if run_start_pos >= delete_end {
                break;
            }
            removed += self.runs[remove_end].length;
            remove_end += 1;
        }

        self.runs.drain(start_idx..remove_end);

        // If all runs were removed, add an empty run
        if self.runs.is_empty() {
            self.runs.push(Run {
                value: T::default(),
                length: 0,
            });
        }

        self.total_length -= actual_length;

        // Merge adjacent at the deletion point
        if start_idx > 0
            && start_idx < self.runs.len()
            && self.runs[start_idx - 1].value == self.runs[start_idx].value
        {
            self.runs[start_idx - 1].length += self.runs[start_idx].length;
            self.runs.remove(start_idx);
        }

        // Handle zero-length total: ensure at least one run
        if self.total_length == 0 && self.runs.is_empty() {
            self.runs.push(Run {
                value: T::default(),
                length: 0,
            });
        }

        self.rebuild_cumulative();
    }

    /// Returns true if the entire sequence has T::default() values (effectively empty).
    pub fn is_empty(&self) -> bool {
        self.runs.iter().all(|r| r.value == T::default())
    }

    /// Total length of the sequence.
    pub fn total_length(&self) -> u64 {
        self.total_length
    }

    /// Iterator over runs intersecting [start, end).
    /// Returns (position, &Run<T>) pairs.
    pub fn runs_in_range(&self, start: u64, end: u64) -> Vec<(u64, &Run<T>)> {
        if start >= end || start >= self.total_length {
            return Vec::new();
        }

        let end = end.min(self.total_length);
        let mut result = Vec::new();
        let start_idx = self.find_run_index(start);

        let mut pos = if start_idx == 0 {
            0
        } else {
            self.cumulative[start_idx - 1]
        };

        for i in start_idx..self.runs.len() {
            if pos >= end {
                break;
            }
            result.push((pos, &self.runs[i]));
            pos += self.runs[i].length;
        }

        result
    }

    /// Get a reference to the internal runs (for testing and verification).
    pub fn runs(&self) -> &[Run<T>] {
        &self.runs
    }

    // ─── Private Helpers ────────────────────────────────────────────────────

    /// Find the index of the run containing `position` using binary search.
    fn find_run_index(&self, position: u64) -> usize {
        match self.cumulative.binary_search(&(position + 1)) {
            Ok(idx) => {
                // position+1 is exactly a cumulative boundary
                // This means position is the last element of run at idx
                // But we need to be careful: if position+1 equals cumulative[idx],
                // then position is in run idx
                idx
            }
            Err(idx) => idx,
        }
    }

    /// Find the run index where a position would be inserted (for split operations).
    fn find_run_index_for_insert(&self, position: u64) -> usize {
        if position == 0 {
            return 0;
        }
        // Find the run that starts at `position`
        for (i, &cum) in self.cumulative.iter().enumerate() {
            if cum == position {
                return i + 1;
            }
        }
        self.find_run_index(position)
    }

    /// Split the run at `position` so that a run boundary exists at that position.
    /// No-op if there's already a boundary there.
    fn split_at(&mut self, position: u64) {
        if position == 0 || position >= self.total_length {
            return;
        }

        let idx = self.find_run_index(position);
        let run_start = if idx == 0 {
            0
        } else {
            self.cumulative[idx - 1]
        };

        if run_start == position {
            // Already a boundary here
            return;
        }

        // Split run at idx into two parts
        let offset = position - run_start;
        let original_length = self.runs[idx].length;
        let value = self.runs[idx].value.clone();

        self.runs[idx].length = offset;
        self.runs.insert(
            idx + 1,
            Run {
                value,
                length: original_length - offset,
            },
        );

        self.rebuild_cumulative();
    }

    /// Merge adjacent runs with the same value around index `idx`.
    fn merge_adjacent(&mut self, idx: usize) {
        // Merge with next
        if idx + 1 < self.runs.len() && self.runs[idx].value == self.runs[idx + 1].value {
            self.runs[idx].length += self.runs[idx + 1].length;
            self.runs.remove(idx + 1);
        }
        // Merge with previous
        if idx > 0 && self.runs[idx - 1].value == self.runs[idx].value {
            self.runs[idx - 1].length += self.runs[idx].length;
            self.runs.remove(idx);
        }
    }

    /// Rebuild the cumulative length cache.
    fn rebuild_cumulative(&mut self) {
        self.cumulative.clear();
        let mut sum = 0u64;
        for run in &self.runs {
            sum += run.length;
            self.cumulative.push(sum);
        }
    }
}

impl<T: Clone + Eq + Default + Debug> Clone for RunStyles<T> {
    fn clone(&self) -> Self {
        Self {
            runs: self.runs.clone(),
            cumulative: self.cumulative.clone(),
            total_length: self.total_length,
        }
    }
}

impl<T: Clone + Eq + Default + Debug> PartialEq for RunStyles<T> {
    fn eq(&self, other: &Self) -> bool {
        self.runs == other.runs && self.total_length == other.total_length
    }
}

impl<T: Clone + Eq + Default + Debug> Debug for RunStyles<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunStyles")
            .field("runs", &self.runs)
            .field("total_length", &self.total_length)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_single_default_run() {
        // Validates: Requirement 3.1
        let rs: RunStyles<u32> = RunStyles::new(100);
        assert_eq!(rs.total_length(), 100);
        assert_eq!(rs.runs().len(), 1);
        assert_eq!(rs.runs()[0].value, 0);
        assert_eq!(rs.runs()[0].length, 100);
    }

    #[test]
    fn value_at_returns_default_for_new_storage() {
        // Validates: Requirement 3.5
        let rs: RunStyles<u32> = RunStyles::new(100);
        assert_eq!(rs.value_at(0), 0);
        assert_eq!(rs.value_at(50), 0);
        assert_eq!(rs.value_at(99), 0);
    }

    #[test]
    fn value_at_out_of_range_returns_default() {
        let rs: RunStyles<u32> = RunStyles::new(100);
        assert_eq!(rs.value_at(100), 0);
        assert_eq!(rs.value_at(1000), 0);
    }

    #[test]
    fn fill_range_sets_values_and_returns_true() {
        // Validates: Requirement 3.8
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        let changed = rs.fill_range(10, 5, 20);
        assert!(changed);
        assert_eq!(rs.value_at(9), 0);
        assert_eq!(rs.value_at(10), 5);
        assert_eq!(rs.value_at(29), 5);
        assert_eq!(rs.value_at(30), 0);
    }

    #[test]
    fn fill_range_idempotent_returns_false() {
        // Validates: Property 2 — fill_range idempotency
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(10, 5, 20);
        let changed = rs.fill_range(10, 5, 20);
        assert!(!changed);
    }

    #[test]
    fn fill_range_merges_adjacent_same_value_runs() {
        // Validates: Property 9 — run merge optimality
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(10, 5, 10); // [10..20) = 5
        rs.fill_range(20, 5, 10); // [20..30) = 5 → should merge
                                  // No adjacent runs with same value
        for i in 0..rs.runs().len() - 1 {
            assert_ne!(rs.runs()[i].value, rs.runs()[i + 1].value);
        }
        assert_eq!(rs.value_at(10), 5);
        assert_eq!(rs.value_at(29), 5);
    }

    #[test]
    fn insert_space_shifts_values_rightward() {
        // Validates: Requirement 4 AC 1
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(10, 5, 10); // [10..20) = 5
        rs.insert_space(15, 5); // insert 5 at position 15
        assert_eq!(rs.total_length(), 105);
        // Before insertion point: unchanged
        assert_eq!(rs.value_at(10), 5);
        assert_eq!(rs.value_at(14), 5);
        // Inserted space: default value
        assert_eq!(rs.value_at(15), 0);
        assert_eq!(rs.value_at(19), 0);
        // After insertion: shifted
        assert_eq!(rs.value_at(20), 5);
        assert_eq!(rs.value_at(24), 5);
        assert_eq!(rs.value_at(25), 0);
    }

    #[test]
    fn delete_range_removes_positions_and_shifts() {
        // Validates: Requirement 4 AC 2
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(10, 5, 20); // [10..30) = 5
        rs.delete_range(15, 5); // delete [15..20)
        assert_eq!(rs.total_length(), 95);
        assert_eq!(rs.value_at(10), 5);
        assert_eq!(rs.value_at(14), 5);
        assert_eq!(rs.value_at(15), 5); // was at 20, shifted left
        assert_eq!(rs.value_at(24), 5); // was at 29
        assert_eq!(rs.value_at(25), 0); // was at 30
    }

    #[test]
    fn insert_delete_round_trip_restores_state() {
        // Validates: Property 3 — insert-delete round trip
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(10, 3, 20);
        rs.fill_range(50, 7, 10);
        let before = rs.clone();
        rs.insert_space(30, 15);
        rs.delete_range(30, 15);
        assert_eq!(rs, before);
    }

    #[test]
    fn is_empty_true_when_all_default() {
        let rs: RunStyles<u32> = RunStyles::new(100);
        assert!(rs.is_empty());
    }

    #[test]
    fn is_empty_false_with_non_default_values() {
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(5, 1, 1);
        assert!(!rs.is_empty());
    }

    #[test]
    fn runs_in_range_returns_intersecting_runs() {
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(10, 5, 10); // [10..20) = 5
        rs.fill_range(30, 8, 10); // [30..40) = 8

        let runs = rs.runs_in_range(5, 35);
        // Should include runs covering positions 5..35
        assert!(!runs.is_empty());
    }

    #[test]
    fn run_start_and_run_end_return_boundaries() {
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(10, 5, 20); // [10..30) = 5
        assert_eq!(rs.run_start(15), 10);
        assert_eq!(rs.run_end(15), 30);
        assert_eq!(rs.run_start(5), 0);
        assert_eq!(rs.run_end(5), 10);
    }

    #[test]
    fn total_length_preserved_after_operations() {
        // Validates: Property 1 — RLE total length preservation
        let mut rs: RunStyles<u32> = RunStyles::new(100);
        rs.fill_range(10, 5, 20);
        assert_eq!(rs.total_length(), 100);
        rs.insert_space(50, 30);
        assert_eq!(rs.total_length(), 130);
        rs.delete_range(20, 10);
        assert_eq!(rs.total_length(), 120);
    }
}
