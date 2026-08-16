//! LineLayoutCache — collection of LineLayout entries with LRU eviction.
//!
//! Supports configurable scope (Viewport/Page/Document), LRU eviction,
//! and memory budget enforcement.
//!
//! Adapted from Scintilla's `LineLayoutCache`.

use std::collections::{HashMap, VecDeque};

use crate::line_layout::LineLayout;
use crate::types::{CacheLevel, ValidLevel};

/// Collection of LineLayout entries with level-based scoping, LRU eviction,
/// and memory budget enforcement.
pub struct LineLayoutCache {
    /// Cached layouts indexed by document line number.
    entries: HashMap<u64, LineLayout>,
    /// LRU ordering (most-recent at back).
    lru_order: VecDeque<u64>,
    /// Current cache level.
    level: CacheLevel,
    /// Maximum entry count.
    max_entries: usize,
    /// Memory budget in bytes.
    memory_budget: usize,
    /// Current memory usage in bytes.
    memory_used: usize,
    /// LRU access counter.
    access_counter: u64,
    /// Per-entry last-access time.
    last_access: HashMap<u64, u64>,
    /// Line number of the caret line (prioritised for retention).
    caret_line: Option<u64>,
}

impl LineLayoutCache {
    /// Create a new LineLayoutCache with the given level and viewport size.
    pub fn new(level: CacheLevel, visible_count: usize, overscan: usize) -> Self {
        let max_entries = Self::compute_max_entries(level, visible_count, overscan);
        Self {
            entries: HashMap::new(),
            lru_order: VecDeque::new(),
            level,
            max_entries,
            memory_budget: 64 * 1024 * 1024, // 64 MB default
            memory_used: 0,
            access_counter: 0,
            last_access: HashMap::new(),
            caret_line: None,
        }
    }

    fn compute_max_entries(level: CacheLevel, visible_count: usize, overscan: usize) -> usize {
        match level {
            CacheLevel::Viewport => visible_count.max(1),
            CacheLevel::Page => (visible_count + 2 * overscan).max(1),
            CacheLevel::Document => usize::MAX,
        }
    }

    /// Set the memory budget in bytes.
    pub fn set_memory_budget(&mut self, budget_bytes: usize) {
        self.memory_budget = budget_bytes;
        self.enforce_memory_budget();
    }

    /// Set the caret line (prioritised for retention during eviction).
    pub fn set_caret_line(&mut self, line: Option<u64>) {
        self.caret_line = line;
    }

    /// Look up a cached LineLayout for the given line number.
    ///
    /// Returns `None` on cache miss; updates LRU on hit.
    pub fn get(&mut self, line_number: u64) -> Option<&LineLayout> {
        if self.entries.contains_key(&line_number) {
            self.access_counter += 1;
            self.last_access.insert(line_number, self.access_counter);
            // Move to back of LRU
            self.lru_order.retain(|&l| l != line_number);
            self.lru_order.push_back(line_number);
            self.entries.get(&line_number)
        } else {
            None
        }
    }

    /// Store a LineLayout. May evict LRU entries if at capacity or memory budget.
    pub fn insert(&mut self, layout: LineLayout) {
        let line_number = layout.line_number;
        let mem = layout.memory_bytes();

        // Remove existing entry if present
        if let Some(old) = self.entries.remove(&line_number) {
            self.memory_used = self.memory_used.saturating_sub(old.memory_bytes());
            self.lru_order.retain(|&l| l != line_number);
            self.last_access.remove(&line_number);
        }

        // Evict if at capacity
        while self.entries.len() >= self.max_entries && !self.entries.is_empty() {
            self.evict_one();
        }

        // Evict if memory budget exceeded
        while self.memory_used + mem > self.memory_budget && !self.entries.is_empty() {
            self.evict_one();
        }

        self.memory_used += mem;
        self.access_counter += 1;
        self.last_access.insert(line_number, self.access_counter);
        self.lru_order.push_back(line_number);
        self.entries.insert(line_number, layout);
    }

    /// Evict the least-recently-used non-caret entry.
    fn evict_one(&mut self) {
        // Find the LRU entry that is not the caret line
        let evict_line = self
            .lru_order
            .iter()
            .find(|&&l| Some(l) != self.caret_line)
            .copied()
            .or_else(|| self.lru_order.front().copied());

        if let Some(line) = evict_line {
            self.lru_order.retain(|&l| l != line);
            if let Some(entry) = self.entries.remove(&line) {
                self.memory_used = self.memory_used.saturating_sub(entry.memory_bytes());
            }
            self.last_access.remove(&line);
        }
    }

    fn enforce_memory_budget(&mut self) {
        let target = (self.memory_budget as f64 * 0.9) as usize;
        while self.memory_used > target && !self.entries.is_empty() {
            self.evict_one();
        }
    }

    /// Invalidate a single line's entry.
    pub fn invalidate_line(&mut self, line_number: u64) {
        if let Some(entry) = self.entries.get_mut(&line_number) {
            entry.validity = ValidLevel::Invalid;
        }
    }

    /// Invalidate all entries at or after the given line (for line-count changes).
    pub fn invalidate_from(&mut self, line_number: u64) {
        for entry in self.entries.values_mut() {
            if entry.line_number >= line_number {
                entry.validity = ValidLevel::Invalid;
            }
        }
    }

    /// Set validity level for all entries (e.g., Positions after resize).
    pub fn downgrade_all_to(&mut self, level: ValidLevel) {
        for entry in self.entries.values_mut() {
            entry.downgrade_to(level);
        }
    }

    /// Invalidate a specific line to CheckTextAndStyle.
    pub fn mark_check_style(&mut self, line_number: u64) {
        if let Some(entry) = self.entries.get_mut(&line_number) {
            entry.downgrade_to(ValidLevel::CheckTextAndStyle);
        }
    }

    /// Clear all entries (full invalidation).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
        self.last_access.clear();
        self.memory_used = 0;
    }

    /// Update the cache level.
    pub fn set_level(&mut self, level: CacheLevel, visible_count: usize, overscan: usize) {
        self.level = level;
        self.max_entries = Self::compute_max_entries(level, visible_count, overscan);
        // Evict excess entries
        while self.entries.len() > self.max_entries {
            self.evict_one();
        }
    }

    /// Current memory usage in bytes.
    pub fn memory_used(&self) -> usize {
        self.memory_used
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::XPosition;

    fn make_layout(line: u64, text_len: u64, validity: ValidLevel) -> LineLayout {
        let mut l = LineLayout::new(line, text_len);
        l.validity = validity;
        l
    }

    #[test]
    fn insert_and_get() {
        // Validates: Requirement 3 AC 1
        let mut cache = LineLayoutCache::new(CacheLevel::Page, 20, 5);
        cache.insert(make_layout(5, 100, ValidLevel::Lines));
        assert!(cache.get(5).is_some());
        assert!(cache.get(6).is_none());
    }

    #[test]
    fn lru_eviction_removes_oldest() {
        // Validates: Requirement 3 AC 7 — Property 4: LRU Ordering
        let mut cache = LineLayoutCache::new(CacheLevel::Viewport, 2, 0);
        cache.insert(make_layout(1, 10, ValidLevel::Lines));
        cache.insert(make_layout(2, 10, ValidLevel::Lines));
        // Access line 1 to make it more recent
        cache.get(1);
        // Insert line 3 — should evict line 2 (LRU)
        cache.insert(make_layout(3, 10, ValidLevel::Lines));
        assert!(cache.get(1).is_some());
        assert!(cache.get(3).is_some());
        assert!(cache.get(2).is_none());
    }

    #[test]
    fn caret_line_not_evicted_first() {
        // Validates: Requirement 3 AC 7
        let mut cache = LineLayoutCache::new(CacheLevel::Viewport, 2, 0);
        cache.set_caret_line(Some(1));
        cache.insert(make_layout(1, 10, ValidLevel::Lines));
        cache.insert(make_layout(2, 10, ValidLevel::Lines));
        // Insert line 3 — should evict line 2 (not caret line 1)
        cache.insert(make_layout(3, 10, ValidLevel::Lines));
        assert!(cache.get(1).is_some(), "caret line should be retained");
    }

    #[test]
    fn invalidate_line_sets_invalid() {
        // Validates: Requirement 9 AC 1
        let mut cache = LineLayoutCache::new(CacheLevel::Page, 20, 5);
        cache.insert(make_layout(5, 100, ValidLevel::Lines));
        cache.invalidate_line(5);
        let entry = cache.get(5).unwrap();
        assert_eq!(entry.validity, ValidLevel::Invalid);
    }

    #[test]
    fn invalidate_from_affects_later_lines() {
        // Validates: Requirement 9 AC 2
        let mut cache = LineLayoutCache::new(CacheLevel::Page, 20, 5);
        cache.insert(make_layout(3, 10, ValidLevel::Lines));
        cache.insert(make_layout(5, 10, ValidLevel::Lines));
        cache.insert(make_layout(7, 10, ValidLevel::Lines));
        cache.invalidate_from(5);
        assert_eq!(cache.get(3).unwrap().validity, ValidLevel::Lines);
        assert_eq!(cache.get(5).unwrap().validity, ValidLevel::Invalid);
        assert_eq!(cache.get(7).unwrap().validity, ValidLevel::Invalid);
    }

    #[test]
    fn downgrade_all_to_positions() {
        // Validates: Requirement 9 AC 5
        let mut cache = LineLayoutCache::new(CacheLevel::Page, 20, 5);
        cache.insert(make_layout(1, 10, ValidLevel::Lines));
        cache.insert(make_layout(2, 10, ValidLevel::Lines));
        cache.downgrade_all_to(ValidLevel::Positions);
        assert_eq!(cache.get(1).unwrap().validity, ValidLevel::Positions);
        assert_eq!(cache.get(2).unwrap().validity, ValidLevel::Positions);
    }

    #[test]
    fn clear_removes_all() {
        // Validates: Requirement 9 AC 3, AC 4
        let mut cache = LineLayoutCache::new(CacheLevel::Page, 20, 5);
        cache.insert(make_layout(1, 10, ValidLevel::Lines));
        cache.insert(make_layout(2, 10, ValidLevel::Lines));
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.memory_used(), 0);
    }

    #[test]
    fn memory_budget_enforced() {
        // Validates: Requirement 7 AC 4, AC 5 — Property 11: Memory Budget Enforcement
        let mut cache = LineLayoutCache::new(CacheLevel::Document, 100, 10);
        // Set a very small budget
        cache.set_memory_budget(1000);
        // Insert many entries
        for i in 0..100u64 {
            cache.insert(make_layout(i, 10, ValidLevel::Lines));
        }
        assert!(cache.memory_used() <= 1000);
    }

    #[test]
    fn page_level_capacity() {
        // Validates: Requirement 3 AC 8
        let cache = LineLayoutCache::new(CacheLevel::Page, 20, 5);
        assert_eq!(cache.max_entries, 30); // 20 + 2*5
    }

    #[test]
    fn viewport_level_capacity() {
        // Validates: Requirement 3 AC 8
        let cache = LineLayoutCache::new(CacheLevel::Viewport, 20, 5);
        assert_eq!(cache.max_entries, 20);
    }
}
