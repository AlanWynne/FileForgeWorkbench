//! Recent files list management.
//!
//! Maintains a bounded, ordered collection of recently opened/saved
//! resource URIs with persistence support.

use ff_vfs::ResourceUri;

/// A bounded, ordered list of recently accessed file URIs.
///
/// Most-recently-used ordering with configurable max capacity.
/// Addresses: Requirement 6, criteria 1–10
#[derive(Debug, Clone)]
pub struct RecentFilesList {
    /// Ordered entries (index 0 = most recent).
    entries: Vec<ResourceUri>,
    /// Maximum number of entries (from configuration).
    max_count: usize,
}

impl RecentFilesList {
    /// Create a new empty list with the given maximum capacity.
    pub fn new(max_count: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_count: max_count.max(1),
        }
    }

    /// Add or promote a URI to the top of the list.
    ///
    /// Removes any existing duplicate entry. Evicts the oldest
    /// entry when at capacity.
    ///
    /// Addresses: Requirement 6 AC 6.3, 6.4
    pub fn add(&mut self, uri: ResourceUri) {
        // Remove existing duplicate
        self.entries.retain(|existing| existing != &uri);
        // Insert at front
        self.entries.insert(0, uri);
        // Evict oldest if over capacity
        if self.entries.len() > self.max_count {
            self.entries.truncate(self.max_count);
        }
    }

    /// Remove a specific URI from the list.
    ///
    /// Returns `true` if the URI was found and removed.
    /// Addresses: Requirement 6 AC 6.6
    pub fn remove(&mut self, uri: &ResourceUri) -> bool {
        let before = self.entries.len();
        self.entries.retain(|existing| existing != uri);
        self.entries.len() < before
    }

    /// Get all entries in most-recent-first order.
    pub fn list(&self) -> &[ResourceUri] {
        &self.entries
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the configured maximum count.
    pub fn max_count(&self) -> usize {
        self.max_count
    }

    /// Update the maximum capacity. Truncates if the list currently
    /// exceeds the new maximum.
    pub fn set_max_count(&mut self, max_count: usize) {
        self.max_count = max_count.max(1);
        if self.entries.len() > self.max_count {
            self.entries.truncate(self.max_count);
        }
    }

    /// Serialize the list to a vector of URI strings for persistence.
    ///
    /// Addresses: Requirement 6 AC 6.7
    pub fn serialize(&self) -> Vec<String> {
        self.entries.iter().map(|uri| uri.as_str()).collect()
    }

    /// Deserialize from persisted URI strings.
    ///
    /// Invalid URIs are silently skipped (graceful degradation).
    /// Addresses: Requirement 6 AC 6.8, 6.9
    pub fn deserialize(data: &[String], max_count: usize) -> Self {
        let max_count = max_count.max(1);
        let entries: Vec<ResourceUri> = data
            .iter()
            .filter_map(|s| ResourceUri::parse(s).ok())
            .take(max_count)
            .collect();

        Self { entries, max_count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_list_is_empty() {
        let list = RecentFilesList::new(10);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.max_count(), 10);
    }

    #[test]
    fn add_places_uri_at_front() {
        let mut list = RecentFilesList::new(10);
        let uri = ResourceUri::new("local", "/test.txt");
        list.add(uri.clone());
        assert_eq!(list.list()[0], uri);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn add_deduplicates_and_moves_to_front() {
        let mut list = RecentFilesList::new(10);
        let uri1 = ResourceUri::new("local", "/first.txt");
        let uri2 = ResourceUri::new("local", "/second.txt");

        list.add(uri1.clone());
        list.add(uri2.clone());
        list.add(uri1.clone());

        assert_eq!(list.len(), 2);
        assert_eq!(list.list()[0], uri1);
        assert_eq!(list.list()[1], uri2);
    }

    #[test]
    fn add_evicts_oldest_when_at_capacity() {
        let mut list = RecentFilesList::new(2);
        let uri1 = ResourceUri::new("local", "/one.txt");
        let uri2 = ResourceUri::new("local", "/two.txt");
        let uri3 = ResourceUri::new("local", "/three.txt");

        list.add(uri1.clone());
        list.add(uri2.clone());
        list.add(uri3.clone());

        assert_eq!(list.len(), 2);
        assert_eq!(list.list()[0], uri3);
        assert_eq!(list.list()[1], uri2);
        assert!(!list.list().contains(&uri1));
    }

    #[test]
    fn remove_returns_true_when_found() {
        let mut list = RecentFilesList::new(10);
        let uri = ResourceUri::new("local", "/test.txt");
        list.add(uri.clone());
        assert!(list.remove(&uri));
        assert!(list.is_empty());
    }

    #[test]
    fn remove_returns_false_when_not_found() {
        let mut list = RecentFilesList::new(10);
        let uri = ResourceUri::new("local", "/missing.txt");
        assert!(!list.remove(&uri));
    }

    #[test]
    fn set_max_count_truncates_if_over() {
        let mut list = RecentFilesList::new(10);
        for i in 0..5 {
            list.add(ResourceUri::new("local", &format!("/file{i}.txt")));
        }
        assert_eq!(list.len(), 5);
        list.set_max_count(3);
        assert_eq!(list.len(), 3);
        assert_eq!(list.max_count(), 3);
    }

    #[test]
    fn serialize_produces_uri_strings() {
        let mut list = RecentFilesList::new(10);
        list.add(ResourceUri::new("local", "/a.txt"));
        list.add(ResourceUri::new("local", "/b.txt"));
        let serialized = list.serialize();
        assert_eq!(serialized.len(), 2);
        assert!(serialized[0].contains("/b.txt"));
        assert!(serialized[1].contains("/a.txt"));
    }

    #[test]
    fn deserialize_gracefully_skips_invalid_uris() {
        let data = vec![
            "vfs://local/valid.txt".to_string(),
            "not-a-valid-uri".to_string(),
            "vfs://local/also-valid.txt".to_string(),
        ];
        let list = RecentFilesList::deserialize(&data, 10);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn deserialize_respects_max_count() {
        let data: Vec<String> = (0..20)
            .map(|i| format!("vfs://local/file{i}.txt"))
            .collect();
        let list = RecentFilesList::deserialize(&data, 5);
        assert_eq!(list.len(), 5);
    }

    #[test]
    fn min_max_count_is_one() {
        let list = RecentFilesList::new(0);
        assert_eq!(list.max_count(), 1);
    }
}
