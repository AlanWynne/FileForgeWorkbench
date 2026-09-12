use std::fmt::Debug;

use super::{Run, RunStyles};

impl<T: Clone + Eq + Default + Debug> RunStyles<T> {
    // === Private Helpers ====================================================

    /// Find the index of the run containing `position` using binary search.
    pub(super) fn find_run_index(&self, position: u64) -> usize {
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
    pub(super) fn find_run_index_for_insert(&self, position: u64) -> usize {
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
    pub(super) fn split_at(&mut self, position: u64) {
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
    pub(super) fn merge_adjacent(&mut self, idx: usize) {
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
    pub(super) fn rebuild_cumulative(&mut self) {
        self.cumulative.clear();
        let mut sum = 0u64;
        for run in &self.runs {
            sum += run.length;
            self.cumulative.push(sum);
        }
    }
}
