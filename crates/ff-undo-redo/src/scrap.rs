//! Scrap stack — contiguous byte buffer for undo text storage.
//!
//! The [`ScrapStack`] minimises allocation overhead and cache misses by storing
//! all text data for undo history operations in a single contiguous buffer.
//! Each edit operation references its text data by offset and length into this buffer.

use serde::{Deserialize, Serialize};

/// Contiguous byte buffer storing all text data for undo history.
///
/// Text from insert and delete operations is pushed here. Operations reference
/// their text by `(offset, length)` pairs rather than owning separate `Vec<u8>`
/// allocations.
///
/// # Design
///
/// - Push appends data to the end of the buffer and returns the (offset, length).
/// - Get retrieves a slice by offset and length.
/// - Clear releases all storage and resets the position.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScrapStack {
    /// The contiguous byte buffer.
    buffer: Vec<u8>,
}

impl ScrapStack {
    /// Creates a new empty scrap stack.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Creates a scrap stack with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Appends data to the scrap stack.
    ///
    /// Returns the `(offset, length)` pair that can be used to retrieve the data later.
    pub fn push(&mut self, data: &[u8]) -> (u64, u32) {
        let offset = self.buffer.len() as u64;
        let length = data.len() as u32;
        self.buffer.extend_from_slice(data);
        (offset, length)
    }

    /// Retrieves text by offset and length.
    ///
    /// # Panics
    ///
    /// Panics if the range is out of bounds. Callers must ensure offsets are valid
    /// (they come from prior `push` calls that are still within the buffer).
    pub fn get(&self, offset: u64, length: u32) -> &[u8] {
        let start = offset as usize;
        let end = start + length as usize;
        &self.buffer[start..end]
    }

    /// Retrieves text by offset and length, returning `None` if out of bounds.
    pub fn try_get(&self, offset: u64, length: u32) -> Option<&[u8]> {
        let start = offset as usize;
        let end = start.checked_add(length as usize)?;
        if end <= self.buffer.len() {
            Some(&self.buffer[start..end])
        } else {
            None
        }
    }

    /// Releases all storage and resets the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the total number of bytes stored.
    pub fn len(&self) -> u64 {
        self.buffer.len() as u64
    }

    /// Returns true if the scrap stack is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the raw buffer for serialization purposes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Constructs a scrap stack from raw bytes (for deserialization).
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { buffer: data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scrap_stack_is_empty() {
        let scrap = ScrapStack::new();
        assert!(scrap.is_empty());
        assert_eq!(scrap.len(), 0);
    }

    #[test]
    fn push_returns_correct_offset_and_length() {
        let mut scrap = ScrapStack::new();
        let (offset, length) = scrap.push(b"hello");
        assert_eq!(offset, 0);
        assert_eq!(length, 5);
    }

    #[test]
    fn push_multiple_items_returns_sequential_offsets() {
        let mut scrap = ScrapStack::new();
        let (o1, l1) = scrap.push(b"hello");
        let (o2, l2) = scrap.push(b" world");

        assert_eq!(o1, 0);
        assert_eq!(l1, 5);
        assert_eq!(o2, 5);
        assert_eq!(l2, 6);
    }

    #[test]
    fn get_retrieves_correct_data() {
        let mut scrap = ScrapStack::new();
        let (offset, length) = scrap.push(b"hello");
        assert_eq!(scrap.get(offset, length), b"hello");
    }

    #[test]
    fn get_retrieves_multiple_items_correctly() {
        let mut scrap = ScrapStack::new();
        let (o1, l1) = scrap.push(b"aaa");
        let (o2, l2) = scrap.push(b"bbb");
        let (o3, l3) = scrap.push(b"ccc");

        assert_eq!(scrap.get(o1, l1), b"aaa");
        assert_eq!(scrap.get(o2, l2), b"bbb");
        assert_eq!(scrap.get(o3, l3), b"ccc");
    }

    #[test]
    fn clear_resets_all_state() {
        let mut scrap = ScrapStack::new();
        scrap.push(b"some data");
        scrap.clear();

        assert!(scrap.is_empty());
        assert_eq!(scrap.len(), 0);
    }

    #[test]
    fn len_tracks_total_bytes() {
        let mut scrap = ScrapStack::new();
        scrap.push(b"abc");
        assert_eq!(scrap.len(), 3);
        scrap.push(b"defgh");
        assert_eq!(scrap.len(), 8);
    }

    #[test]
    fn try_get_returns_none_for_invalid_range() {
        let scrap = ScrapStack::new();
        assert_eq!(scrap.try_get(0, 1), None);
    }

    #[test]
    fn try_get_returns_data_for_valid_range() {
        let mut scrap = ScrapStack::new();
        let (o, l) = scrap.push(b"test");
        assert_eq!(scrap.try_get(o, l), Some(b"test".as_slice()));
    }

    #[test]
    fn push_empty_data_returns_zero_length() {
        let mut scrap = ScrapStack::new();
        let (offset, length) = scrap.push(b"");
        assert_eq!(offset, 0);
        assert_eq!(length, 0);
    }
}
