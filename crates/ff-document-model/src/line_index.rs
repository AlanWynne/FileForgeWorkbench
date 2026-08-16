//! Line index structure mapping line numbers to byte positions.
//!
//! Uses a sorted Vec of line-start byte positions for O(log n) lookups
//! in both directions.

use crate::gap_buffer::GapBuffer;
use crate::line_end::{self, LineEndMode};
use crate::types::{BytePosition, LineNumber};

/// Balanced structure mapping line numbers to byte positions.
/// Provides O(log n) lookups in both directions.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Line start positions. Entry i is the byte position of line i.
    /// Always has at least one entry (line 0 starts at position 0).
    line_starts: Vec<u64>,
}

impl LineIndex {
    /// Create a new line index with a single line at position 0.
    pub fn new() -> Self {
        Self {
            line_starts: vec![0],
        }
    }

    /// Total number of lines.
    pub fn line_count(&self) -> u64 {
        self.line_starts.len() as u64
    }

    /// Byte position of the first byte on a line.
    /// If line is past the end, returns the document length (which callers must provide).
    pub fn line_start(&self, line: LineNumber) -> BytePosition {
        let idx = line.0 as usize;
        if idx < self.line_starts.len() {
            BytePosition(self.line_starts[idx])
        } else {
            // Return the position after the last known start as a fallback.
            // The caller should provide doc length for the "past end" case.
            BytePosition(*self.line_starts.last().unwrap_or(&0))
        }
    }

    /// Get the byte position of line start, with doc_length as fallback for out-of-range.
    pub fn line_start_clamped(&self, line: LineNumber, doc_length: u64) -> BytePosition {
        let idx = line.0 as usize;
        if idx < self.line_starts.len() {
            BytePosition(self.line_starts[idx])
        } else {
            BytePosition(doc_length)
        }
    }

    /// Find which line contains a byte position via O(log n) binary search.
    pub fn line_from_position(&self, position: BytePosition) -> LineNumber {
        let pos = position.0;
        // Binary search for the last line_start <= position
        match self.line_starts.binary_search(&pos) {
            Ok(idx) => LineNumber(idx as u64),
            Err(idx) => {
                // idx is where pos would be inserted; the line is idx - 1
                LineNumber(idx.saturating_sub(1) as u64)
            }
        }
    }

    /// Insert a new line record after `after_line` at the given byte position.
    pub fn insert_line_after(&mut self, after_line: LineNumber, position: BytePosition) {
        let insert_idx = (after_line.0 as usize + 1).min(self.line_starts.len());
        self.line_starts.insert(insert_idx, position.0);
    }

    /// Insert a line at a specific index.
    pub fn insert_line_at_index(&mut self, index: usize, position: u64) {
        if index <= self.line_starts.len() {
            self.line_starts.insert(index, position);
        }
    }

    /// Remove a line record at the given index.
    pub fn remove_line(&mut self, line: LineNumber) {
        let idx = line.0 as usize;
        if idx < self.line_starts.len() && self.line_starts.len() > 1 {
            self.line_starts.remove(idx);
        }
    }

    /// Remove `count` lines starting at `start_line`.
    pub fn remove_lines(&mut self, start_line: LineNumber, count: u64) {
        let start = start_line.0 as usize;
        let end = (start + count as usize).min(self.line_starts.len());
        if start < self.line_starts.len() && self.line_starts.len() > 1 {
            // Don't remove line 0
            let actual_start = start.max(1);
            let actual_end = end.max(actual_start);
            if actual_start < actual_end {
                self.line_starts.drain(actual_start..actual_end);
            }
        }
    }

    /// Adjust positions of all lines from `from_line` onwards by `delta`.
    pub fn adjust_positions(&mut self, from_line: LineNumber, delta: i64) {
        let start = from_line.0 as usize;
        for i in start..self.line_starts.len() {
            if delta >= 0 {
                self.line_starts[i] += delta as u64;
            } else {
                self.line_starts[i] = self.line_starts[i].saturating_sub((-delta) as u64);
            }
        }
    }

    /// Rebuild the index from buffer content by scanning for line endings.
    pub fn rebuild(&mut self, content: &[u8], mode: LineEndMode) {
        self.line_starts.clear();
        self.line_starts.push(0); // Line 0 always starts at 0

        let mut i = 0;
        while i < content.len() {
            let le_len = line_end::line_ending_length_at(content, i, mode);
            if le_len > 0 {
                // Next line starts after this line ending
                self.line_starts.push((i + le_len) as u64);
                i += le_len;
            } else {
                i += 1;
            }
        }
    }

    /// Rebuild from a gap buffer (compacts the gap temporarily).
    pub fn rebuild_from_buffer(&mut self, buffer: &mut GapBuffer, mode: LineEndMode) {
        let content = buffer.contiguous_view();
        self.line_starts.clear();
        self.line_starts.push(0);

        let mut i = 0;
        while i < content.len() {
            let le_len = line_end::line_ending_length_at(content, i, mode);
            if le_len > 0 {
                self.line_starts.push((i + le_len) as u64);
                i += le_len;
            } else {
                i += 1;
            }
        }
    }

    /// Direct access to internal line starts (for testing and sparse index finalization).
    #[allow(dead_code)]
    pub(crate) fn line_starts(&self) -> &[u64] {
        &self.line_starts
    }

    /// Set line starts directly (used by sparse index finalization).
    #[allow(dead_code)]
    pub(crate) fn set_line_starts(&mut self, starts: Vec<u64>) {
        self.line_starts = starts;
    }
}

impl Default for LineIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_index_has_one_line() {
        let idx = LineIndex::new();
        assert_eq!(idx.line_count(), 1);
        assert_eq!(idx.line_start(LineNumber(0)), BytePosition(0));
    }

    #[test]
    fn rebuild_counts_lines_correctly() {
        let mut idx = LineIndex::new();
        idx.rebuild(b"hello\nworld\n", LineEndMode::Default);
        assert_eq!(idx.line_count(), 3); // "hello\n", "world\n", ""
        assert_eq!(idx.line_start(LineNumber(0)), BytePosition(0));
        assert_eq!(idx.line_start(LineNumber(1)), BytePosition(6));
        assert_eq!(idx.line_start(LineNumber(2)), BytePosition(12));
    }

    #[test]
    fn rebuild_crlf_counts_as_one() {
        let mut idx = LineIndex::new();
        idx.rebuild(b"line1\r\nline2\r\n", LineEndMode::Default);
        assert_eq!(idx.line_count(), 3);
        assert_eq!(idx.line_start(LineNumber(1)), BytePosition(7));
        assert_eq!(idx.line_start(LineNumber(2)), BytePosition(14));
    }

    #[test]
    fn line_from_position_binary_search() {
        let mut idx = LineIndex::new();
        idx.rebuild(b"abc\ndef\nghi", LineEndMode::Default);
        // Lines: 0=[0,3], 1=[4,7], 2=[8,10]
        assert_eq!(idx.line_from_position(BytePosition(0)), LineNumber(0));
        assert_eq!(idx.line_from_position(BytePosition(3)), LineNumber(0));
        assert_eq!(idx.line_from_position(BytePosition(4)), LineNumber(1));
        assert_eq!(idx.line_from_position(BytePosition(8)), LineNumber(2));
        assert_eq!(idx.line_from_position(BytePosition(10)), LineNumber(2));
    }

    #[test]
    fn insert_line_after() {
        let mut idx = LineIndex::new();
        // Initial: [0]
        idx.insert_line_after(LineNumber(0), BytePosition(10));
        assert_eq!(idx.line_count(), 2);
        assert_eq!(idx.line_start(LineNumber(1)), BytePosition(10));
    }

    #[test]
    fn remove_line_preserves_line_zero() {
        let mut idx = LineIndex::new();
        idx.rebuild(b"a\nb\nc", LineEndMode::Default);
        assert_eq!(idx.line_count(), 3);
        idx.remove_line(LineNumber(1));
        assert_eq!(idx.line_count(), 2);
    }

    #[test]
    fn adjust_positions_positive_delta() {
        let mut idx = LineIndex::new();
        idx.rebuild(b"ab\ncd\nef", LineEndMode::Default);
        // line_starts = [0, 3, 6]
        idx.adjust_positions(LineNumber(1), 5);
        assert_eq!(idx.line_start(LineNumber(0)), BytePosition(0));
        assert_eq!(idx.line_start(LineNumber(1)), BytePosition(8));
        assert_eq!(idx.line_start(LineNumber(2)), BytePosition(11));
    }

    #[test]
    fn empty_document_has_one_line() {
        let mut idx = LineIndex::new();
        idx.rebuild(b"", LineEndMode::Default);
        assert_eq!(idx.line_count(), 1);
    }

    #[test]
    fn line_start_past_last_line_returns_last() {
        let mut idx = LineIndex::new();
        idx.rebuild(b"abc", LineEndMode::Default);
        // Only 1 line, asking for line 5 should return the last known start
        assert_eq!(idx.line_start(LineNumber(5)), BytePosition(0));
        // With clamped version:
        assert_eq!(idx.line_start_clamped(LineNumber(5), 3), BytePosition(3));
    }

    #[test]
    fn unicode_mode_rebuild() {
        let mut idx = LineIndex::new();
        // NEL = 0xC2 0x85
        let content: Vec<u8> = [b"hello".as_slice(), &[0xC2, 0x85], b"world"].concat();
        idx.rebuild(&content, LineEndMode::Unicode);
        assert_eq!(idx.line_count(), 2);
        assert_eq!(idx.line_start(LineNumber(1)), BytePosition(7)); // 5 + 2
    }
}
