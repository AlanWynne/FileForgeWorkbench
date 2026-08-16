//! Character indexer trait: abstract byte-level access to document buffers.
//!
//! The `CharacterIndexer` trait decouples the search engine from the concrete
//! buffer implementation in `ff-document-model`.
//!
//! Addresses: Requirement 18

use crate::error::FindReplaceError;
use crate::types::{BytePosition, Direction, LineNumber};

/// Abstract byte-level access to the document buffer.
///
/// Implemented by `ff-document-model` over its GapBuffer/SplitView.
/// Enables the FindEngine to search without depending on a specific buffer type.
///
/// Addresses: Requirement 18
pub trait CharacterIndexer: Send + Sync {
    /// Read a single byte at the given position.
    /// Returns None if position is beyond document length.
    ///
    /// Addresses: Requirement 18 AC 1
    fn char_at(&self, position: BytePosition) -> Option<u8>;

    /// Read a contiguous slice of bytes as a Vec.
    /// Returns None if range is invalid (start > end or end > length).
    ///
    /// Addresses: Requirement 18 AC 2
    fn slice(&self, start: BytePosition, end: BytePosition) -> Option<Vec<u8>>;

    /// Align a byte position to the nearest UTF-8 character boundary.
    ///
    /// Addresses: Requirement 18 AC 3
    fn move_position_outside_char(
        &self,
        position: BytePosition,
        direction: Direction,
    ) -> BytePosition;

    /// Get the byte range [start, end) of a given line.
    /// Returns None if line number is beyond document line count.
    ///
    /// Addresses: Requirement 18 AC 5
    fn line_range(&self, line: LineNumber) -> Option<(BytePosition, BytePosition)>;

    /// Total byte length of the document.
    fn length(&self) -> u64;

    /// Total line count.
    fn line_count(&self) -> u64;

    /// Determine which line a byte position belongs to.
    fn line_from_position(&self, position: BytePosition) -> LineNumber;
}

/// Mutable extension of CharacterIndexer for CHANGE operations.
///
/// Provides document mutation primitives used by the replacement engine.
///
/// Addresses: Requirement 6 (replacement requires mutation)
pub trait CharacterIndexerMut: CharacterIndexer {
    /// Replace bytes in range [start, end) with new_bytes.
    /// Returns the length delta (new_len - old_len).
    fn replace_range(
        &mut self,
        start: BytePosition,
        end: BytePosition,
        new_bytes: &[u8],
    ) -> Result<i64, FindReplaceError>;

    /// Check if the document is read-only.
    fn is_read_only(&self) -> bool;
}

/// A simple adapter implementing `CharacterIndexer` over a byte slice.
///
/// Used for testing purposes.
#[derive(Debug, Clone)]
pub struct SliceIndexer {
    data: Vec<u8>,
    line_starts: Vec<u64>,
}

impl SliceIndexer {
    /// Create a new SliceIndexer from a byte slice.
    pub fn new(data: &[u8]) -> Self {
        let mut line_starts = vec![0u64];
        for (i, &byte) in data.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push((i + 1) as u64);
            }
        }
        Self {
            data: data.to_vec(),
            line_starts,
        }
    }

    /// Create from a string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self::new(s.as_bytes())
    }
}

impl CharacterIndexer for SliceIndexer {
    fn char_at(&self, position: BytePosition) -> Option<u8> {
        self.data.get(position.0 as usize).copied()
    }

    fn slice(&self, start: BytePosition, end: BytePosition) -> Option<Vec<u8>> {
        let s = start.0 as usize;
        let e = end.0 as usize;
        if s > e || e > self.data.len() {
            None
        } else {
            Some(self.data[s..e].to_vec())
        }
    }

    fn move_position_outside_char(
        &self,
        position: BytePosition,
        direction: Direction,
    ) -> BytePosition {
        let pos = position.0 as usize;
        if pos >= self.data.len() {
            return BytePosition(self.data.len() as u64);
        }

        // Check if we're inside a multi-byte UTF-8 character (continuation byte)
        let byte = self.data[pos];
        if (byte & 0xC0) != 0x80 {
            // Not a continuation byte, already at a boundary
            return position;
        }

        match direction {
            Direction::Forward => {
                // Move forward to next non-continuation byte
                let mut p = pos;
                while p < self.data.len() && (self.data[p] & 0xC0) == 0x80 {
                    p += 1;
                }
                BytePosition(p as u64)
            }
            Direction::Backward => {
                // Move backward to the start of this character
                let mut p = pos;
                while p > 0 && (self.data[p] & 0xC0) == 0x80 {
                    p -= 1;
                }
                BytePosition(p as u64)
            }
        }
    }

    fn line_range(&self, line: LineNumber) -> Option<(BytePosition, BytePosition)> {
        let line_idx = line.0 as usize;
        if line_idx >= self.line_starts.len() {
            return None;
        }

        let start = self.line_starts[line_idx];
        let end = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1]
        } else {
            self.data.len() as u64
        };

        Some((BytePosition(start), BytePosition(end)))
    }

    fn length(&self) -> u64 {
        self.data.len() as u64
    }

    fn line_count(&self) -> u64 {
        self.line_starts.len() as u64
    }

    fn line_from_position(&self, position: BytePosition) -> LineNumber {
        let pos = position.0;
        // Binary search for the line containing this position
        let idx = match self.line_starts.binary_search(&pos) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        LineNumber(idx as u64)
    }
}

/// A mutable slice indexer for testing CHANGE operations.
#[derive(Debug, Clone)]
pub struct MutableSliceIndexer {
    inner: SliceIndexer,
    read_only: bool,
}

impl MutableSliceIndexer {
    /// Create a new mutable indexer from a string.
    pub fn new(data: &str) -> Self {
        Self {
            inner: SliceIndexer::from_str(data),
            read_only: false,
        }
    }

    /// Create a read-only mutable indexer.
    pub fn read_only(data: &str) -> Self {
        Self {
            inner: SliceIndexer::from_str(data),
            read_only: true,
        }
    }

    /// Get the current content as a string.
    pub fn content(&self) -> &[u8] {
        &self.inner.data
    }

    /// Get the current content as a UTF-8 string (if valid).
    pub fn content_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.inner.data).ok()
    }
}

impl CharacterIndexer for MutableSliceIndexer {
    fn char_at(&self, position: BytePosition) -> Option<u8> {
        self.inner.char_at(position)
    }

    fn slice(&self, start: BytePosition, end: BytePosition) -> Option<Vec<u8>> {
        self.inner.slice(start, end)
    }

    fn move_position_outside_char(
        &self,
        position: BytePosition,
        direction: Direction,
    ) -> BytePosition {
        self.inner.move_position_outside_char(position, direction)
    }

    fn line_range(&self, line: LineNumber) -> Option<(BytePosition, BytePosition)> {
        self.inner.line_range(line)
    }

    fn length(&self) -> u64 {
        self.inner.length()
    }

    fn line_count(&self) -> u64 {
        self.inner.line_count()
    }

    fn line_from_position(&self, position: BytePosition) -> LineNumber {
        self.inner.line_from_position(position)
    }
}

impl CharacterIndexerMut for MutableSliceIndexer {
    fn replace_range(
        &mut self,
        start: BytePosition,
        end: BytePosition,
        new_bytes: &[u8],
    ) -> Result<i64, FindReplaceError> {
        if self.read_only {
            return Err(FindReplaceError::DocumentReadOnly);
        }

        let s = start.0 as usize;
        let e = end.0 as usize;
        let old_len = e - s;
        let new_len = new_bytes.len();
        let delta = new_len as i64 - old_len as i64;

        self.inner.data.splice(s..e, new_bytes.iter().copied());

        // Rebuild line starts
        self.inner.line_starts = vec![0u64];
        for (i, &byte) in self.inner.data.iter().enumerate() {
            if byte == b'\n' {
                self.inner.line_starts.push((i + 1) as u64);
            }
        }

        Ok(delta)
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_indexer_char_at_returns_byte_at_position() {
        let indexer = SliceIndexer::from_str("hello");
        assert_eq!(indexer.char_at(BytePosition(0)), Some(b'h'));
        assert_eq!(indexer.char_at(BytePosition(4)), Some(b'o'));
        assert_eq!(indexer.char_at(BytePosition(5)), None);
    }

    #[test]
    fn slice_indexer_slice_returns_byte_range() {
        let indexer = SliceIndexer::from_str("hello world");
        let result = indexer.slice(BytePosition(0), BytePosition(5)).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn slice_indexer_slice_returns_none_for_invalid_range() {
        let indexer = SliceIndexer::from_str("hello");
        assert_eq!(indexer.slice(BytePosition(3), BytePosition(2)), None);
        assert_eq!(indexer.slice(BytePosition(0), BytePosition(10)), None);
    }

    #[test]
    fn slice_indexer_length_returns_byte_count() {
        let indexer = SliceIndexer::from_str("hello");
        assert_eq!(indexer.length(), 5);
    }

    #[test]
    fn slice_indexer_line_count_counts_newlines_plus_one() {
        let indexer = SliceIndexer::from_str("line1\nline2\nline3");
        assert_eq!(indexer.line_count(), 3);
    }

    #[test]
    fn slice_indexer_line_range_returns_correct_byte_spans() {
        let indexer = SliceIndexer::from_str("line1\nline2\nline3");
        let (start, end) = indexer.line_range(LineNumber(0)).unwrap();
        assert_eq!(start, BytePosition(0));
        assert_eq!(end, BytePosition(6)); // includes \n

        let (start, end) = indexer.line_range(LineNumber(1)).unwrap();
        assert_eq!(start, BytePosition(6));
        assert_eq!(end, BytePosition(12));

        let (start, end) = indexer.line_range(LineNumber(2)).unwrap();
        assert_eq!(start, BytePosition(12));
        assert_eq!(end, BytePosition(17));
    }

    #[test]
    fn slice_indexer_line_from_position_maps_byte_to_line() {
        let indexer = SliceIndexer::from_str("line1\nline2\nline3");
        assert_eq!(indexer.line_from_position(BytePosition(0)), LineNumber(0));
        assert_eq!(indexer.line_from_position(BytePosition(5)), LineNumber(0));
        assert_eq!(indexer.line_from_position(BytePosition(6)), LineNumber(1));
        assert_eq!(indexer.line_from_position(BytePosition(12)), LineNumber(2));
    }

    #[test]
    fn slice_indexer_move_position_outside_char_handles_multibyte() {
        // "é" is 2 bytes: 0xC3 0xA9
        let indexer = SliceIndexer::from_str("aé");
        // Position 2 is the continuation byte
        let result = indexer.move_position_outside_char(BytePosition(2), Direction::Backward);
        assert_eq!(result, BytePosition(1));
    }

    #[test]
    fn mutable_indexer_replace_range_substitutes_bytes() {
        let mut indexer = MutableSliceIndexer::new("hello world");
        let delta = indexer
            .replace_range(BytePosition(0), BytePosition(5), b"goodbye")
            .unwrap();
        assert_eq!(delta, 2); // "goodbye" is 7 bytes, "hello" is 5
        assert_eq!(indexer.content_str(), Some("goodbye world"));
    }

    #[test]
    fn mutable_indexer_read_only_rejects_replace() {
        let mut indexer = MutableSliceIndexer::read_only("hello");
        let result = indexer.replace_range(BytePosition(0), BytePosition(5), b"bye");
        assert!(matches!(result, Err(FindReplaceError::DocumentReadOnly)));
    }
}
