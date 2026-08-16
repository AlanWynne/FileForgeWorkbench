//! Clipboard history ring — bounded ring buffer of recent clipboard entries.
//!
//! Provides clipboard history navigation (paste-from-history) with configurable
//! capacity and FIFO eviction of oldest entries when capacity is exceeded.

use std::collections::VecDeque;

use crate::entry::ClipboardEntry;

/// Bounded ring buffer of recent clipboard entries.
///
/// The most recently pushed entry is at the front. When capacity is reached,
/// the oldest entry (at the back) is evicted to make room for new entries.
#[derive(Debug)]
pub struct ClipboardHistoryRing {
    entries: VecDeque<ClipboardEntry>,
    capacity: usize,
    cursor: usize,
}

impl ClipboardHistoryRing {
    /// Create a new history ring with the given maximum capacity.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "history ring capacity must be at least 1");
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
            cursor: 0,
        }
    }

    /// Push a new entry to the front of the ring.
    ///
    /// If the ring is at capacity, the oldest entry (at the back) is evicted.
    /// The cursor is reset to 0 (most recent entry).
    pub fn push(&mut self, entry: ClipboardEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_back();
        }
        self.entries.push_front(entry);
        self.cursor = 0;
    }

    /// Get the most recent (latest) entry, if any.
    pub fn latest(&self) -> Option<&ClipboardEntry> {
        self.entries.front()
    }

    /// Get the entry at the current cursor position.
    pub fn current(&self) -> Option<&ClipboardEntry> {
        self.entries.get(self.cursor)
    }

    /// Move cursor to the previous (older) entry and return it.
    ///
    /// Wraps around to the newest entry when reaching the end.
    pub fn cycle_back(&mut self) -> Option<&ClipboardEntry> {
        if self.entries.is_empty() {
            return None;
        }
        self.cursor = (self.cursor + 1) % self.entries.len();
        self.entries.get(self.cursor)
    }

    /// Move cursor to the next (newer) entry and return it.
    ///
    /// Wraps around to the oldest entry when reaching the beginning.
    pub fn cycle_forward(&mut self) -> Option<&ClipboardEntry> {
        if self.entries.is_empty() {
            return None;
        }
        if self.cursor == 0 {
            self.cursor = self.entries.len() - 1;
        } else {
            self.cursor -= 1;
        }
        self.entries.get(self.cursor)
    }

    /// Iterate over all entries, newest first.
    pub fn iter(&self) -> impl Iterator<Item = &ClipboardEntry> {
        self.entries.iter()
    }

    /// The number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the ring is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The maximum capacity of the ring.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all entries from the ring.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::ClipboardEntry;

    #[test]
    fn new_ring_is_empty() {
        let ring = ClipboardHistoryRing::new(10);
        assert!(ring.is_empty());
        assert_eq!(ring.len(), 0);
        assert!(ring.latest().is_none());
    }

    #[test]
    fn push_adds_to_front() {
        let mut ring = ClipboardHistoryRing::new(5);
        ring.push(ClipboardEntry::stream("first".to_string()));
        ring.push(ClipboardEntry::stream("second".to_string()));

        assert_eq!(ring.latest().unwrap().text(), "second");
        assert_eq!(ring.len(), 2);
    }

    #[test]
    fn push_evicts_oldest_at_capacity() {
        let mut ring = ClipboardHistoryRing::new(3);
        ring.push(ClipboardEntry::stream("a".to_string()));
        ring.push(ClipboardEntry::stream("b".to_string()));
        ring.push(ClipboardEntry::stream("c".to_string()));
        assert_eq!(ring.len(), 3);

        ring.push(ClipboardEntry::stream("d".to_string()));
        assert_eq!(ring.len(), 3);
        assert_eq!(ring.latest().unwrap().text(), "d");

        // "a" should have been evicted
        let texts: Vec<&str> = ring.iter().map(|e| e.text()).collect();
        assert_eq!(texts, vec!["d", "c", "b"]);
    }

    #[test]
    fn cycle_back_moves_to_older_entries() {
        let mut ring = ClipboardHistoryRing::new(5);
        ring.push(ClipboardEntry::stream("oldest".to_string()));
        ring.push(ClipboardEntry::stream("middle".to_string()));
        ring.push(ClipboardEntry::stream("newest".to_string()));

        assert_eq!(ring.current().unwrap().text(), "newest");
        assert_eq!(ring.cycle_back().unwrap().text(), "middle");
        assert_eq!(ring.cycle_back().unwrap().text(), "oldest");
        // Wraps around
        assert_eq!(ring.cycle_back().unwrap().text(), "newest");
    }

    #[test]
    fn cycle_forward_moves_to_newer_entries() {
        let mut ring = ClipboardHistoryRing::new(5);
        ring.push(ClipboardEntry::stream("oldest".to_string()));
        ring.push(ClipboardEntry::stream("middle".to_string()));
        ring.push(ClipboardEntry::stream("newest".to_string()));

        // Move back first
        ring.cycle_back(); // -> middle
        ring.cycle_back(); // -> oldest

        // Now forward
        assert_eq!(ring.cycle_forward().unwrap().text(), "middle");
        assert_eq!(ring.cycle_forward().unwrap().text(), "newest");
    }

    #[test]
    fn cycle_back_on_empty_returns_none() {
        let mut ring = ClipboardHistoryRing::new(5);
        assert!(ring.cycle_back().is_none());
    }

    #[test]
    fn cycle_forward_on_empty_returns_none() {
        let mut ring = ClipboardHistoryRing::new(5);
        assert!(ring.cycle_forward().is_none());
    }

    #[test]
    fn clear_empties_the_ring() {
        let mut ring = ClipboardHistoryRing::new(5);
        ring.push(ClipboardEntry::stream("a".to_string()));
        ring.push(ClipboardEntry::stream("b".to_string()));
        assert_eq!(ring.len(), 2);

        ring.clear();
        assert!(ring.is_empty());
        assert_eq!(ring.len(), 0);
        assert!(ring.latest().is_none());
    }

    #[test]
    fn push_resets_cursor_to_zero() {
        let mut ring = ClipboardHistoryRing::new(5);
        ring.push(ClipboardEntry::stream("a".to_string()));
        ring.push(ClipboardEntry::stream("b".to_string()));
        ring.cycle_back(); // cursor = 1

        ring.push(ClipboardEntry::stream("c".to_string()));
        assert_eq!(ring.current().unwrap().text(), "c"); // cursor reset to 0
    }

    #[test]
    #[should_panic(expected = "capacity must be at least 1")]
    fn zero_capacity_panics() {
        ClipboardHistoryRing::new(0);
    }
}
