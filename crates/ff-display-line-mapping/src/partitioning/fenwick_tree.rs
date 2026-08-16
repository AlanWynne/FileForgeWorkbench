//! Fenwick tree (Binary Indexed Tree) for O(log n) prefix-sum queries
//! and point updates.
//!
//! This data structure stores per-line effective display heights and supports:
//! - `prefix_sum(idx)`: cumulative height of lines [0, idx) in O(log n)
//! - `find_prefix(target)`: inverse lookup — smallest idx where prefix_sum > target
//! - `update(idx, delta)`: point update at a specific index in O(log n)
//! - `insert/remove`: structural changes with partial rebuild
//!
//! Addresses: Requirement 5 AC 1, AC 2, AC 3

/// A Fenwick tree (Binary Indexed Tree) storing per-line effective display heights.
///
/// Internally uses 1-based indexing. The tree array has length `line_count + 1`.
/// Each element stores partial sums enabling O(log n) prefix queries.
#[derive(Debug, Clone)]
pub struct FenwickTree {
    /// Internal tree storage (1-indexed). tree[0] is unused.
    tree: Vec<i64>,
    /// Original values for each position (0-indexed).
    values: Vec<i64>,
}

impl FenwickTree {
    /// Create a new Fenwick tree with `n` elements, all initialized to `initial_value`.
    pub fn new(n: usize, initial_value: i64) -> Self {
        let values = vec![initial_value; n];
        let tree = Self::build_tree(&values);
        Self { tree, values }
    }

    /// Build the internal tree array from a slice of values.
    fn build_tree(values: &[i64]) -> Vec<i64> {
        let n = values.len();
        let mut tree = vec![0i64; n + 1];
        // Copy values into 1-indexed positions
        tree[1..(n + 1)].copy_from_slice(&values[..n]);
        // Build prefix sums in-place
        for i in 1..=n {
            let parent = i + (i & i.wrapping_neg());
            if parent <= n {
                tree[parent] += tree[i];
            }
        }
        tree
    }

    /// Query the prefix sum from index 0 to `idx` (exclusive).
    /// Returns the cumulative sum of values at positions [0, idx).
    ///
    /// O(log n) time.
    pub fn prefix_sum(&self, idx: usize) -> i64 {
        let mut sum = 0i64;
        let mut i = idx; // 1-indexed: sum of [1..idx] = sum of 0-indexed [0..idx)
        while i > 0 {
            sum += self.tree[i];
            i -= i & i.wrapping_neg();
        }
        sum
    }

    /// Get the value at a specific 0-based index.
    ///
    /// O(1) time (uses the values array).
    pub fn get(&self, idx: usize) -> i64 {
        self.values[idx]
    }

    /// Update the value at 0-based `idx` by adding `delta`.
    ///
    /// O(log n) time.
    pub fn update(&mut self, idx: usize, delta: i64) {
        self.values[idx] += delta;
        let n = self.values.len();
        let mut i = idx + 1; // Convert to 1-indexed
        while i <= n {
            self.tree[i] += delta;
            i += i & i.wrapping_neg();
        }
    }

    /// Set the value at 0-based `idx` to `new_value`.
    ///
    /// O(log n) time.
    pub fn set(&mut self, idx: usize, new_value: i64) {
        let delta = new_value - self.values[idx];
        if delta != 0 {
            self.update(idx, delta);
        }
    }

    /// Find the smallest 0-based index where the prefix sum of [0..=idx] > target.
    /// This is equivalent to finding which document line contains display line `target`.
    ///
    /// Returns the 0-based document line index. If target >= total(), returns len()-1.
    ///
    /// O(log n) time via binary lifting.
    pub fn find_prefix(&self, target: i64) -> usize {
        let n = self.values.len();
        if n == 0 {
            return 0;
        }
        let mut pos = 0usize;
        let mut remaining = target;
        // Find highest bit
        let mut bit_mask = 1;
        while bit_mask <= n {
            bit_mask <<= 1;
        }
        bit_mask >>= 1;

        while bit_mask > 0 {
            let next = pos + bit_mask;
            if next <= n && self.tree[next] <= remaining {
                pos = next;
                remaining -= self.tree[next];
            }
            bit_mask >>= 1;
        }
        // pos is now the 1-indexed position of the line containing the target
        // The 0-indexed document line is pos (since prefix_sum(pos) <= target < prefix_sum(pos+1))
        pos.min(n.saturating_sub(1))
    }

    /// Total sum of all elements (= total display line count when storing effective heights).
    pub fn total(&self) -> i64 {
        self.prefix_sum(self.values.len())
    }

    /// Number of elements in the tree.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Insert `count` new elements at 0-based position `idx`, each with value `val`.
    ///
    /// Rebuilds the tree after insertion.
    pub fn insert(&mut self, idx: usize, count: usize, val: i64) {
        let insert_at = idx.min(self.values.len());
        for i in 0..count {
            self.values.insert(insert_at + i, val);
        }
        self.tree = Self::build_tree(&self.values);
    }

    /// Remove `count` elements starting at 0-based position `idx`.
    ///
    /// Rebuilds the tree after removal.
    pub fn remove(&mut self, idx: usize, count: usize) {
        let end = (idx + count).min(self.values.len());
        self.values.drain(idx..end);
        self.tree = Self::build_tree(&self.values);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree_all_ones_has_correct_total() {
        let tree = FenwickTree::new(10, 1);
        assert_eq!(tree.total(), 10);
    }

    #[test]
    fn prefix_sum_returns_cumulative_values() {
        let tree = FenwickTree::new(5, 1);
        assert_eq!(tree.prefix_sum(0), 0);
        assert_eq!(tree.prefix_sum(1), 1);
        assert_eq!(tree.prefix_sum(2), 2);
        assert_eq!(tree.prefix_sum(3), 3);
        assert_eq!(tree.prefix_sum(4), 4);
        assert_eq!(tree.prefix_sum(5), 5);
    }

    #[test]
    fn update_changes_value_and_prefix_sums() {
        let mut tree = FenwickTree::new(5, 1);
        // Set index 2 to height 3 (delta = +2)
        tree.update(2, 2);
        assert_eq!(tree.get(2), 3);
        assert_eq!(tree.prefix_sum(3), 5); // 1 + 1 + 3
        assert_eq!(tree.total(), 7); // 1+1+3+1+1
    }

    #[test]
    fn set_value_works_correctly() {
        let mut tree = FenwickTree::new(5, 1);
        tree.set(2, 4);
        assert_eq!(tree.get(2), 4);
        assert_eq!(tree.total(), 8); // 1+1+4+1+1
    }

    #[test]
    fn find_prefix_returns_correct_line() {
        // Heights: [1, 1, 3, 1, 1] -> prefix sums: [0, 1, 2, 5, 6, 7]
        let mut tree = FenwickTree::new(5, 1);
        tree.set(2, 3);

        // Display line 0 belongs to doc line 0
        assert_eq!(tree.find_prefix(0), 0);
        // Display line 1 belongs to doc line 1
        assert_eq!(tree.find_prefix(1), 1);
        // Display lines 2, 3, 4 belong to doc line 2 (height 3)
        assert_eq!(tree.find_prefix(2), 2);
        assert_eq!(tree.find_prefix(3), 2);
        assert_eq!(tree.find_prefix(4), 2);
        // Display line 5 belongs to doc line 3
        assert_eq!(tree.find_prefix(5), 3);
        // Display line 6 belongs to doc line 4
        assert_eq!(tree.find_prefix(6), 4);
    }

    #[test]
    fn insert_adds_elements_and_updates_sums() {
        let mut tree = FenwickTree::new(3, 1);
        assert_eq!(tree.total(), 3);
        tree.insert(1, 2, 1);
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.total(), 5);
    }

    #[test]
    fn remove_deletes_elements_and_updates_sums() {
        let mut tree = FenwickTree::new(5, 1);
        tree.set(2, 3);
        assert_eq!(tree.total(), 7);
        tree.remove(2, 1);
        assert_eq!(tree.len(), 4);
        assert_eq!(tree.total(), 4);
    }

    #[test]
    fn empty_tree_operations_are_safe() {
        let tree = FenwickTree::new(0, 1);
        assert_eq!(tree.total(), 0);
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
        assert_eq!(tree.find_prefix(0), 0);
    }

    #[test]
    fn update_with_negative_delta_decreases_values() {
        let mut tree = FenwickTree::new(5, 2);
        assert_eq!(tree.total(), 10);
        tree.update(3, -2); // Set index 3 to 0
        assert_eq!(tree.get(3), 0);
        assert_eq!(tree.total(), 8);
    }
}
