//! Recent Files list management — bounded ordered storage with deduplication,
//! eviction, and availability checking.
//!
//! Addresses: Requirement 4 (AC 4.4, 4.5)

use crate::session_state::RecentFileEntry;

/// A bounded, ordered list of recently opened files.
///
/// Entries are ordered by most-recent-first. The list enforces:
/// - A configurable maximum count (evicts oldest on overflow)
/// - URI-based deduplication (re-adding an existing URI moves it to the top)
/// - Availability marking (files that no longer exist are flagged but retained)
///
/// Addresses: Requirement 4 AC 4.4, 4.5
#[derive(Debug, Clone)]
pub struct RecentFilesList {
    /// The entries, ordered by most-recent-first.
    entries: Vec<RecentFileEntry>,
    /// Maximum number of entries allowed.
    max_count: u32,
}

impl RecentFilesList {
    /// Create a new empty recent files list with the given maximum count.
    pub fn new(max_count: u32) -> Self {
        Self {
            entries: Vec::new(),
            max_count: max_count.max(1), // minimum 1
        }
    }

    /// Create a recent files list from existing entries (e.g., loaded from session).
    ///
    /// If entries exceed `max_count`, the oldest entries are evicted.
    pub fn from_entries(entries: Vec<RecentFileEntry>, max_count: u32) -> Self {
        let max_count = max_count.max(1);
        let mut list = Self { entries, max_count };
        list.enforce_max_count();
        list
    }

    /// Add an entry to the recent files list.
    ///
    /// If the URI already exists, moves it to the top (position 0) and
    /// updates its metadata. If the list is at capacity, evicts the oldest
    /// entry.
    ///
    /// Addresses: Requirement 4 AC 4.4
    pub fn add(&mut self, entry: RecentFileEntry) {
        // Remove existing entry with same URI (deduplication)
        self.entries.retain(|e| e.uri != entry.uri);

        // Insert at position 0 (most recent)
        self.entries.insert(0, entry);

        // Evict oldest if over max
        self.enforce_max_count();
    }

    /// Return entries ordered by most-recent-first.
    pub fn list(&self) -> &[RecentFileEntry] {
        &self.entries
    }

    /// Return the number of entries currently in the list.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return the configured maximum count.
    pub fn max_count(&self) -> u32 {
        self.max_count
    }

    /// Update the maximum count.
    ///
    /// If the new max is smaller than the current list length, oldest
    /// entries are evicted immediately.
    pub fn set_max_count(&mut self, new_max: u32) {
        self.max_count = new_max.max(1);
        self.enforce_max_count();
    }

    /// Mark entries as available or unavailable based on a predicate.
    ///
    /// The predicate receives the URI and returns whether the file exists.
    /// Entries that no longer exist are marked as unavailable but NOT removed.
    ///
    /// Addresses: Requirement 4 AC 4.5
    pub fn check_availability<F>(&mut self, file_exists: F)
    where
        F: Fn(&str) -> bool,
    {
        for entry in &mut self.entries {
            entry.available = file_exists(&entry.uri);
        }
    }

    /// Convert the list to a Vec of entries for serialisation.
    pub fn into_entries(self) -> Vec<RecentFileEntry> {
        self.entries
    }

    /// Remove an entry by URI.
    pub fn remove(&mut self, uri: &str) {
        self.entries.retain(|e| e.uri != uri);
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Enforce the maximum count by evicting oldest entries.
    fn enforce_max_count(&mut self) {
        let max = self.max_count as usize;
        if self.entries.len() > max {
            self.entries.truncate(max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(uri: &str) -> RecentFileEntry {
        RecentFileEntry {
            uri: uri.to_string(),
            display_name: uri.rsplit('/').next().unwrap_or(uri).to_string(),
            last_accessed: "2024-01-15T10:00:00Z".to_string(),
            last_viewport_top_line: None,
            available: true,
        }
    }

    #[test]
    fn new_list_is_empty() {
        // Validates: Requirement 4 AC 4.4
        let list = RecentFilesList::new(50);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.max_count(), 50);
    }

    #[test]
    fn add_entry_places_at_front() {
        // Validates: Requirement 4 AC 4.4
        let mut list = RecentFilesList::new(10);
        list.add(make_entry("file1.txt"));
        list.add(make_entry("file2.txt"));

        assert_eq!(list.list()[0].uri, "file2.txt");
        assert_eq!(list.list()[1].uri, "file1.txt");
    }

    #[test]
    fn add_duplicate_uri_moves_to_front_without_creating_duplicate() {
        // Validates: Requirement 4 AC 4.4 (deduplication)
        let mut list = RecentFilesList::new(10);
        list.add(make_entry("a.txt"));
        list.add(make_entry("b.txt"));
        list.add(make_entry("c.txt"));

        // Re-add "a.txt" — should move to front
        list.add(make_entry("a.txt"));

        assert_eq!(list.len(), 3);
        assert_eq!(list.list()[0].uri, "a.txt");
        assert_eq!(list.list()[1].uri, "c.txt");
        assert_eq!(list.list()[2].uri, "b.txt");
    }

    #[test]
    fn add_evicts_oldest_when_over_max() {
        // Validates: Requirement 4 AC 4.4
        let mut list = RecentFilesList::new(3);
        list.add(make_entry("a.txt"));
        list.add(make_entry("b.txt"));
        list.add(make_entry("c.txt"));
        list.add(make_entry("d.txt"));

        assert_eq!(list.len(), 3);
        assert_eq!(list.list()[0].uri, "d.txt");
        assert_eq!(list.list()[1].uri, "c.txt");
        assert_eq!(list.list()[2].uri, "b.txt");
        // "a.txt" was evicted
    }

    #[test]
    fn list_never_exceeds_max_count() {
        // Validates: Requirement 4 AC 4.4
        let mut list = RecentFilesList::new(5);
        for i in 0..100 {
            list.add(make_entry(&format!("file{i}.txt")));
            assert!(list.len() <= 5);
        }
    }

    #[test]
    fn set_max_count_evicts_excess_entries() {
        let mut list = RecentFilesList::new(10);
        for i in 0..10 {
            list.add(make_entry(&format!("file{i}.txt")));
        }
        assert_eq!(list.len(), 10);

        list.set_max_count(3);
        assert_eq!(list.len(), 3);
        // Oldest entries evicted, most recent kept
        assert_eq!(list.list()[0].uri, "file9.txt");
    }

    #[test]
    fn check_availability_marks_missing_files() {
        // Validates: Requirement 4 AC 4.5
        let mut list = RecentFilesList::new(10);
        list.add(make_entry("exists.txt"));
        list.add(make_entry("missing.txt"));

        list.check_availability(|uri| uri == "exists.txt");

        // "missing.txt" was added last so it's at index 0
        let missing_entry = list.list().iter().find(|e| e.uri == "missing.txt").unwrap();
        let exists_entry = list.list().iter().find(|e| e.uri == "exists.txt").unwrap();
        assert!(exists_entry.available);
        assert!(!missing_entry.available);
    }

    #[test]
    fn check_availability_retains_unavailable_entries() {
        // Validates: Requirement 4 AC 4.5
        let mut list = RecentFilesList::new(10);
        list.add(make_entry("gone.txt"));

        list.check_availability(|_| false);

        // Entry is retained but marked unavailable
        assert_eq!(list.len(), 1);
        assert!(!list.list()[0].available);
    }

    #[test]
    fn from_entries_respects_max_count() {
        let entries: Vec<_> = (0..20)
            .map(|i| make_entry(&format!("file{i}.txt")))
            .collect();
        let list = RecentFilesList::from_entries(entries, 5);
        assert_eq!(list.len(), 5);
    }

    #[test]
    fn remove_deletes_entry_by_uri() {
        let mut list = RecentFilesList::new(10);
        list.add(make_entry("a.txt"));
        list.add(make_entry("b.txt"));

        list.remove("a.txt");
        assert_eq!(list.len(), 1);
        assert_eq!(list.list()[0].uri, "b.txt");
    }

    #[test]
    fn clear_removes_all_entries() {
        let mut list = RecentFilesList::new(10);
        list.add(make_entry("a.txt"));
        list.add(make_entry("b.txt"));

        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn minimum_max_count_is_one() {
        let list = RecentFilesList::new(0);
        assert_eq!(list.max_count(), 1);
    }

    #[test]
    fn into_entries_returns_entries_in_order() {
        let mut list = RecentFilesList::new(10);
        list.add(make_entry("first.txt"));
        list.add(make_entry("second.txt"));

        let entries = list.into_entries();
        assert_eq!(entries[0].uri, "second.txt");
        assert_eq!(entries[1].uri, "first.txt");
    }
}
