//! Gap buffer data structure for efficient text storage.
//!
//! The gap buffer stores text in a contiguous allocation with a movable gap.
//! Insertions at the gap position are O(1) amortized; insertions at other
//! positions require moving the gap first (O(n) in the gap distance).

use crate::types::SplitView;

/// Default growth factor when the gap is exhausted.
const DEFAULT_GROWTH_FACTOR: f64 = 2.0;
/// Minimum gap size after growth.
const MIN_GAP_SIZE: u64 = 64;

/// Low-level text storage with a movable gap for O(1) amortized editing.
///
/// The gap sits at the current edit position; insertions fill the gap,
/// deletions expand it.
#[derive(Debug, Clone)]
pub struct GapBuffer {
    /// Raw byte storage (includes gap region).
    storage: Vec<u8>,
    /// Start of the gap (byte offset in storage).
    gap_start: u64,
    /// End of the gap (byte offset in storage, exclusive).
    gap_end: u64,
    /// Growth factor when gap is exhausted (default: 2.0).
    growth_factor: f64,
}

impl GapBuffer {
    /// Create an empty gap buffer with the specified initial capacity.
    pub fn new(initial_capacity: u64) -> Self {
        let cap = initial_capacity.max(MIN_GAP_SIZE) as usize;
        Self {
            storage: vec![0u8; cap],
            gap_start: 0,
            gap_end: cap as u64,
            growth_factor: DEFAULT_GROWTH_FACTOR,
        }
    }

    /// Create an empty gap buffer with default capacity.
    pub fn default_new() -> Self {
        Self::new(256)
    }

    /// Pre-allocate storage for at least `capacity` bytes of content.
    pub fn allocate(&mut self, capacity: u64) {
        let current_content = self.length();
        let needed = capacity.saturating_sub(current_content);
        if needed > self.gap_size() {
            self.grow_gap(needed);
        }
    }

    /// Set the growth factor (must be >= 1.5).
    pub fn set_growth_factor(&mut self, factor: f64) {
        self.growth_factor = factor.max(1.5);
    }

    /// Total content length (excluding the gap).
    pub fn length(&self) -> u64 {
        self.storage.len() as u64 - self.gap_size()
    }

    /// Current gap size.
    fn gap_size(&self) -> u64 {
        self.gap_end - self.gap_start
    }

    /// Insert bytes at the given content position.
    pub fn insert(&mut self, position: u64, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        let position = position.min(self.length());

        // Move gap to the insertion point
        self.move_gap_to(position);

        // Ensure gap is large enough
        let data_len = data.len() as u64;
        if data_len > self.gap_size() {
            self.grow_gap(data_len);
        }

        // Copy data into the gap
        let start = self.gap_start as usize;
        self.storage[start..start + data.len()].copy_from_slice(data);
        self.gap_start += data_len;
    }

    /// Delete `length` bytes starting at `position`.
    pub fn delete(&mut self, position: u64, length: u64) {
        if length == 0 {
            return;
        }
        let content_len = self.length();
        let position = position.min(content_len);
        let length = length.min(content_len - position);

        // Move gap to the deletion start
        self.move_gap_to(position);

        // Expand gap over deleted bytes (they're after the gap)
        self.gap_end += length;
    }

    /// Get a single byte at a content position.
    pub fn byte_at(&self, position: u64) -> Option<u8> {
        if position >= self.length() {
            return None;
        }
        let storage_pos = self.content_to_storage(position);
        Some(self.storage[storage_pos as usize])
    }

    /// Copy a range of content bytes into a Vec.
    pub fn get_range(&self, position: u64, length: u64) -> Option<Vec<u8>> {
        if position + length > self.length() {
            return None;
        }
        if length == 0 {
            return Some(Vec::new());
        }

        let mut result = Vec::with_capacity(length as usize);
        let end = position + length;

        if end <= self.gap_start {
            // Entirely before the gap
            let start = position as usize;
            let end = end as usize;
            result.extend_from_slice(&self.storage[start..end]);
        } else if position >= self.gap_start {
            // Entirely after the gap
            let offset = self.gap_end - self.gap_start;
            let start = (position + offset) as usize;
            let end = (end + offset) as usize;
            result.extend_from_slice(&self.storage[start..end]);
        } else {
            // Spans the gap
            let before_end = self.gap_start as usize;
            result.extend_from_slice(&self.storage[position as usize..before_end]);
            let after_start = self.gap_end as usize;
            let remaining = (end - self.gap_start) as usize;
            result.extend_from_slice(&self.storage[after_start..after_start + remaining]);
        }

        Some(result)
    }

    /// Compact the gap to end of buffer and return a contiguous view of all content.
    pub fn contiguous_view(&mut self) -> &[u8] {
        // Move gap to the end
        self.move_gap_to(self.length());
        &self.storage[..self.gap_start as usize]
    }

    /// Return a two-segment view without moving the gap.
    pub fn split_view(&self) -> SplitView {
        let before = self.storage[..self.gap_start as usize].to_vec();
        let after = self.storage[self.gap_end as usize..].to_vec();
        SplitView {
            before_gap: before,
            after_gap: after,
        }
    }

    /// Move the gap to the specified content position.
    fn move_gap_to(&mut self, position: u64) {
        if position == self.gap_start {
            return;
        }

        let gap_size = self.gap_size();

        if position < self.gap_start {
            // Move gap left: shift bytes from [position..gap_start] to end of new gap
            let move_count = (self.gap_start - position) as usize;
            let src_start = position as usize;
            let dst_start = (self.gap_end - (self.gap_start - position)) as usize;
            self.storage
                .copy_within(src_start..src_start + move_count, dst_start);
            self.gap_start = position;
            self.gap_end = position + gap_size;
        } else {
            // Move gap right: shift bytes from [gap_end..gap_end + (position - gap_start)] before gap
            let move_count = (position - self.gap_start) as usize;
            let src_start = self.gap_end as usize;
            let dst_start = self.gap_start as usize;
            self.storage
                .copy_within(src_start..src_start + move_count, dst_start);
            self.gap_start = position;
            self.gap_end = position + gap_size;
        }
    }

    /// Grow the gap to accommodate at least `needed` additional bytes.
    fn grow_gap(&mut self, needed: u64) {
        let current_cap = self.storage.len() as u64;
        let content_len = self.length();
        let new_cap = ((current_cap as f64 * self.growth_factor) as u64)
            .max(content_len + needed + MIN_GAP_SIZE);

        let additional_gap = new_cap - current_cap;

        // Insert space after gap_end by extending the vector
        let gap_end_usize = self.gap_end as usize;
        let _after_gap_bytes = self.storage.len() - gap_end_usize;

        let mut new_storage = Vec::with_capacity(new_cap as usize);
        // Copy before gap
        new_storage.extend_from_slice(&self.storage[..self.gap_start as usize]);
        // Extend gap with zeros
        new_storage.resize(
            self.gap_start as usize + (self.gap_size() + additional_gap) as usize,
            0,
        );
        // Copy after gap
        new_storage.extend_from_slice(&self.storage[gap_end_usize..]);

        self.gap_end += additional_gap;
        self.storage = new_storage;
    }

    /// Convert a content position to a storage position.
    fn content_to_storage(&self, position: u64) -> u64 {
        if position < self.gap_start {
            position
        } else {
            position + (self.gap_end - self.gap_start)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        let buf = GapBuffer::new(64);
        assert_eq!(buf.length(), 0);
    }

    #[test]
    fn insert_at_beginning() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"hello");
        assert_eq!(buf.length(), 5);
        assert_eq!(buf.get_range(0, 5), Some(b"hello".to_vec()));
    }

    #[test]
    fn insert_at_end() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"hello");
        buf.insert(5, b" world");
        assert_eq!(buf.length(), 11);
        assert_eq!(buf.get_range(0, 11), Some(b"hello world".to_vec()));
    }

    #[test]
    fn insert_in_middle() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"helo");
        buf.insert(2, b"ll");
        // "helo" = [h,e,l,o]. Insert "ll" at pos 2:
        // content[0..2] + "ll" + content[2..] = "he" + "ll" + "lo" = "hellllo"
        // Bytes: [h(104), e(101), l(108), l(108), l(108), o(111)]
        assert_eq!(buf.length(), 6);
        assert_eq!(
            buf.get_range(0, 6),
            Some(vec![104, 101, 108, 108, 108, 111])
        );
    }

    #[test]
    fn insert_in_middle_correct() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"helo");
        buf.insert(2, b"l");
        // "helo"[0..2] = "he", insert "l", "helo"[2..] = "lo" -> "hello"
        assert_eq!(buf.length(), 5);
        assert_eq!(buf.get_range(0, 5), Some(b"hello".to_vec()));
    }

    #[test]
    fn delete_from_beginning() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"hello world");
        buf.delete(0, 6);
        assert_eq!(buf.length(), 5);
        assert_eq!(buf.get_range(0, 5), Some(b"world".to_vec()));
    }

    #[test]
    fn delete_from_middle() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"hello world");
        buf.delete(5, 1);
        assert_eq!(buf.length(), 10);
        assert_eq!(buf.get_range(0, 10), Some(b"helloworld".to_vec()));
    }

    #[test]
    fn byte_at_valid_positions() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"abc");
        assert_eq!(buf.byte_at(0), Some(b'a'));
        assert_eq!(buf.byte_at(1), Some(b'b'));
        assert_eq!(buf.byte_at(2), Some(b'c'));
        assert_eq!(buf.byte_at(3), None);
    }

    #[test]
    fn get_range_out_of_bounds_returns_none() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"abc");
        assert_eq!(buf.get_range(0, 4), None);
        assert_eq!(buf.get_range(2, 2), None);
    }

    #[test]
    fn contiguous_view_returns_all_content() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"hello");
        buf.insert(5, b" world");
        let view = buf.contiguous_view();
        assert_eq!(view, b"hello world");
    }

    #[test]
    fn split_view_matches_contiguous() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"hello world");
        buf.insert(5, b" beautiful");

        let split = buf.split_view();
        let mut combined = split.before_gap.clone();
        combined.extend_from_slice(&split.after_gap);

        let contiguous = buf.contiguous_view().to_vec();
        assert_eq!(combined, contiguous);
    }

    #[test]
    fn gap_growth_handles_large_insert() {
        let mut buf = GapBuffer::new(4); // tiny initial capacity
        let data = vec![b'x'; 1000];
        buf.insert(0, &data);
        assert_eq!(buf.length(), 1000);
        assert_eq!(buf.get_range(0, 1000), Some(data));
    }

    #[test]
    fn allocate_preallocates_space() {
        let mut buf = GapBuffer::new(64);
        buf.allocate(10000);
        // Should not panic and gap should be large enough
        let data = vec![b'a'; 10000];
        buf.insert(0, &data);
        assert_eq!(buf.length(), 10000);
    }

    #[test]
    fn multiple_inserts_and_deletes() {
        let mut buf = GapBuffer::new(64);
        buf.insert(0, b"abcdef");
        buf.delete(2, 2); // "abef"
        buf.insert(2, b"XY"); // "abXYef"
        assert_eq!(buf.get_range(0, 6), Some(b"abXYef".to_vec()));
    }
}
