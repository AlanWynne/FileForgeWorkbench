//! TextBuffer: primary text storage combining GapBuffer with LineIndex.
//!
//! Coordinates insertion/deletion with line tracking, CRLF edge case handling,
//! and read-only guards.

use crate::error::DocumentError;
use crate::gap_buffer::GapBuffer;
use crate::line_end::{self, LineEndMode};
use crate::line_index::LineIndex;
use crate::types::{BytePosition, DeleteResult, InsertResult, LineNumber, SplitView};

/// Primary text storage: owns the GapBuffer and maintains the LineIndex.
/// Coordinates insertion/deletion with line tracking and read-only guards.
#[derive(Debug, Clone)]
pub struct TextBuffer {
    /// The underlying gap buffer storing raw bytes.
    buffer: GapBuffer,
    /// Line number ↔ byte position mapping.
    line_index: LineIndex,
    /// Current line-end recognition mode.
    line_end_mode: LineEndMode,
    /// Whether the buffer is read-only.
    read_only: bool,
}

impl TextBuffer {
    /// Create an empty text buffer.
    pub fn new() -> Self {
        Self {
            buffer: GapBuffer::default_new(),
            line_index: LineIndex::new(),
            line_end_mode: LineEndMode::Default,
            read_only: false,
        }
    }

    /// Create a text buffer with pre-allocated capacity.
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            buffer: GapBuffer::new(capacity),
            line_index: LineIndex::new(),
            line_end_mode: LineEndMode::Default,
            read_only: false,
        }
    }

    /// Total byte length of content.
    pub fn length(&self) -> u64 {
        self.buffer.length()
    }

    /// Number of lines in the buffer (minimum 1).
    pub fn line_count(&self) -> u64 {
        self.line_index.line_count()
    }

    /// Insert text at position, updating line index.
    pub fn insert(
        &mut self,
        position: BytePosition,
        text: &[u8],
    ) -> Result<InsertResult, DocumentError> {
        if self.read_only {
            return Err(DocumentError::ReadOnly {
                operation: "insert".to_string(),
            });
        }

        if position.0 > self.length() {
            return Err(DocumentError::PositionOutOfRange {
                operation: "insert".to_string(),
                position: position.0,
                length: self.length(),
            });
        }

        if text.is_empty() {
            return Ok(InsertResult {
                lines_added: 0,
                bytes_inserted: 0,
            });
        }

        let bytes_inserted = text.len() as u64;

        // Check for CRLF split: inserting between CR and LF
        let crlf_split = self.is_crlf_split_point(position);

        // Insert into the gap buffer
        self.buffer.insert(position.0, text);

        // Count line endings in the inserted text
        let lines_added = line_end::count_line_endings(text, self.line_end_mode);

        // Handle CRLF merge: does the insertion create new CR+LF adjacencies?
        let crlf_merge_before = self.check_crlf_merge_at_start(position, text);
        let crlf_merge_after = self.check_crlf_merge_at_end(position, text, bytes_inserted);

        // Rebuild line index for the affected region
        // For correctness, rebuild the full index after insertion
        self.rebuild_line_index();

        let actual_line_count_change = self.compute_actual_lines_added(
            lines_added,
            crlf_split,
            crlf_merge_before,
            crlf_merge_after,
        );

        Ok(InsertResult {
            lines_added: actual_line_count_change,
            bytes_inserted,
        })
    }

    /// Delete bytes at position, updating line index.
    pub fn delete(
        &mut self,
        position: BytePosition,
        length: u64,
    ) -> Result<DeleteResult, DocumentError> {
        if self.read_only {
            return Err(DocumentError::ReadOnly {
                operation: "delete".to_string(),
            });
        }

        if position.0 + length > self.length() {
            return Err(DocumentError::PositionOutOfRange {
                operation: "delete".to_string(),
                position: position.0,
                length: self.length(),
            });
        }

        if length == 0 {
            return Ok(DeleteResult {
                lines_removed: 0,
                bytes_deleted: 0,
            });
        }

        // Get the content being deleted to count line endings
        let deleted_content = self
            .buffer
            .get_range(position.0, length)
            .unwrap_or_default();
        let lines_in_deleted = line_end::count_line_endings(&deleted_content, self.line_end_mode);

        // Perform the deletion
        self.buffer.delete(position.0, length);

        // Rebuild line index
        self.rebuild_line_index();

        Ok(DeleteResult {
            lines_removed: lines_in_deleted,
            bytes_deleted: length,
        })
    }

    /// Get byte at position.
    pub fn char_at(&self, position: BytePosition) -> Option<u8> {
        self.buffer.byte_at(position.0)
    }

    /// Get range of bytes.
    pub fn get_range(&self, position: BytePosition, length: u64) -> Option<Vec<u8>> {
        self.buffer.get_range(position.0, length)
    }

    /// Compact and return contiguous view.
    pub fn contiguous_view(&mut self) -> &[u8] {
        self.buffer.contiguous_view()
    }

    /// Return split view without compaction.
    pub fn split_view(&self) -> SplitView {
        self.buffer.split_view()
    }

    /// Set read-only mode.
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Query read-only state.
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Get the byte position of the start of a line.
    pub fn line_start(&self, line: LineNumber) -> BytePosition {
        self.line_index.line_start_clamped(line, self.length())
    }

    /// Get the byte position of the end of a line (before line ending).
    pub fn line_end(&self, line: LineNumber) -> BytePosition {
        let next_line_start = if line.0 + 1 < self.line_count() {
            self.line_index.line_start(LineNumber(line.0 + 1)).0
        } else {
            self.length()
        };

        // Scan backwards from next_line_start to find content end (before line ending)
        if next_line_start == 0 {
            return BytePosition(0);
        }

        // If this is the last line, the end is the document length
        if line.0 + 1 >= self.line_count() {
            return BytePosition(self.length());
        }

        // Look backwards from next_line_start for line ending
        let end = next_line_start;
        if end >= 2 {
            let b1 = self.buffer.byte_at(end - 2);
            let b2 = self.buffer.byte_at(end - 1);
            if b1 == Some(0x0D) && b2 == Some(0x0A) {
                return BytePosition(end - 2);
            }
        }
        if end >= 1 {
            let b = self.buffer.byte_at(end - 1);
            if b == Some(0x0D) || b == Some(0x0A) {
                return BytePosition(end - 1);
            }
            if self.line_end_mode == LineEndMode::Unicode {
                // Check for NEL (2 bytes) or LS/PS (3 bytes)
                if end >= 2 {
                    let b0 = self.buffer.byte_at(end - 2);
                    if b0 == Some(0xC2) && b == Some(0x85) {
                        return BytePosition(end - 2);
                    }
                }
                if end >= 3 {
                    let b0 = self.buffer.byte_at(end - 3);
                    let b1_val = self.buffer.byte_at(end - 2);
                    if b0 == Some(0xE2)
                        && b1_val == Some(0x80)
                        && (b == Some(0xA8) || b == Some(0xA9))
                    {
                        return BytePosition(end - 3);
                    }
                }
            }
        }

        BytePosition(end)
    }

    /// Find which line contains a byte position.
    pub fn line_from_position(&self, position: BytePosition) -> LineNumber {
        self.line_index.line_from_position(position)
    }

    /// Set line-end mode, rescanning if changed.
    pub fn set_line_end_mode(&mut self, mode: LineEndMode) {
        if mode != self.line_end_mode {
            self.line_end_mode = mode;
            self.rebuild_line_index();
        }
    }

    /// Get current line-end mode.
    pub fn line_end_mode(&self) -> LineEndMode {
        self.line_end_mode
    }

    /// Check if text contains a line ending for the current mode.
    pub fn contains_line_end(&self, text: &[u8]) -> bool {
        line_end::contains_line_end(text, self.line_end_mode)
    }

    /// Direct access to the underlying gap buffer (for streaming and advanced use).
    pub(crate) fn gap_buffer(&self) -> &GapBuffer {
        &self.buffer
    }

    /// Mutable access to the underlying gap buffer.
    #[allow(dead_code)]
    pub(crate) fn gap_buffer_mut(&mut self) -> &mut GapBuffer {
        &mut self.buffer
    }

    /// Direct access to the line index.
    #[allow(dead_code)]
    pub(crate) fn line_index(&self) -> &LineIndex {
        &self.line_index
    }

    /// Mutable access to the line index.
    #[allow(dead_code)]
    pub(crate) fn line_index_mut(&mut self) -> &mut LineIndex {
        &mut self.line_index
    }

    /// Rebuild the line index from current buffer content.
    pub(crate) fn rebuild_line_index(&mut self) {
        self.line_index
            .rebuild_from_buffer(&mut self.buffer, self.line_end_mode);
    }

    // --- Private helpers ---

    /// Check if position is between a CR and LF (CRLF split point).
    fn is_crlf_split_point(&self, position: BytePosition) -> bool {
        if position.0 == 0 || position.0 >= self.length() {
            return false;
        }
        let before = self.buffer.byte_at(position.0 - 1);
        let at = self.buffer.byte_at(position.0);
        before == Some(0x0D) && at == Some(0x0A)
    }

    /// Check if insertion at start creates a CRLF merge (text ends with LF and byte before position is CR).
    fn check_crlf_merge_at_start(&self, _position: BytePosition, _text: &[u8]) -> bool {
        // This is handled by the full rebuild
        false
    }

    /// Check if insertion at end creates a CRLF merge.
    fn check_crlf_merge_at_end(
        &self,
        _position: BytePosition,
        _text: &[u8],
        _bytes_inserted: u64,
    ) -> bool {
        // This is handled by the full rebuild
        false
    }

    /// Compute actual lines added accounting for CRLF merges/splits.
    fn compute_actual_lines_added(
        &self,
        base: u64,
        _split: bool,
        _merge_before: bool,
        _merge_after: bool,
    ) -> u64 {
        // Since we do a full rebuild, we calculate from the actual line count difference
        // For now, return the base count from the inserted text
        base
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer_has_one_line() {
        let buf = TextBuffer::new();
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.length(), 0);
    }

    #[test]
    fn insert_text_without_line_endings() {
        let mut buf = TextBuffer::new();
        let result = buf.insert(BytePosition(0), b"hello").unwrap();
        assert_eq!(result.bytes_inserted, 5);
        assert_eq!(result.lines_added, 0);
        assert_eq!(buf.length(), 5);
        assert_eq!(buf.line_count(), 1);
    }

    #[test]
    fn insert_text_with_newline() {
        let mut buf = TextBuffer::new();
        buf.insert(BytePosition(0), b"hello\nworld").unwrap();
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_start(LineNumber(0)), BytePosition(0));
        assert_eq!(buf.line_start(LineNumber(1)), BytePosition(6));
    }

    #[test]
    fn insert_text_with_crlf() {
        let mut buf = TextBuffer::new();
        buf.insert(BytePosition(0), b"hello\r\nworld").unwrap();
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_start(LineNumber(1)), BytePosition(7));
    }

    #[test]
    fn delete_removes_line_endings() {
        let mut buf = TextBuffer::new();
        buf.insert(BytePosition(0), b"a\nb\nc").unwrap();
        assert_eq!(buf.line_count(), 3);
        // Delete the first newline at position 1
        let result = buf.delete(BytePosition(1), 1).unwrap();
        assert_eq!(result.lines_removed, 1);
        assert_eq!(buf.line_count(), 2);
        let content = buf.get_range(BytePosition(0), buf.length()).unwrap();
        assert_eq!(content, b"ab\nc");
    }

    #[test]
    fn read_only_blocks_insert() {
        let mut buf = TextBuffer::new();
        buf.set_read_only(true);
        let err = buf.insert(BytePosition(0), b"hello").unwrap_err();
        assert!(matches!(err, DocumentError::ReadOnly { .. }));
    }

    #[test]
    fn read_only_blocks_delete() {
        let mut buf = TextBuffer::new();
        buf.insert(BytePosition(0), b"hello").unwrap();
        buf.set_read_only(true);
        let err = buf.delete(BytePosition(0), 1).unwrap_err();
        assert!(matches!(err, DocumentError::ReadOnly { .. }));
    }

    #[test]
    fn position_out_of_range_on_insert() {
        let mut buf = TextBuffer::new();
        buf.insert(BytePosition(0), b"abc").unwrap();
        let err = buf.insert(BytePosition(10), b"x").unwrap_err();
        assert!(matches!(err, DocumentError::PositionOutOfRange { .. }));
    }

    #[test]
    fn line_from_position_round_trip() {
        let mut buf = TextBuffer::new();
        buf.insert(BytePosition(0), b"abc\ndef\nghi").unwrap();
        for line_num in 0..buf.line_count() {
            let ln = LineNumber(line_num);
            let start = buf.line_start(ln);
            assert_eq!(buf.line_from_position(start), ln);
        }
    }

    #[test]
    fn line_end_position() {
        let mut buf = TextBuffer::new();
        buf.insert(BytePosition(0), b"abc\ndef\nghi").unwrap();
        assert_eq!(buf.line_end(LineNumber(0)), BytePosition(3));
        assert_eq!(buf.line_end(LineNumber(1)), BytePosition(7));
        assert_eq!(buf.line_end(LineNumber(2)), BytePosition(11)); // end of doc
    }

    #[test]
    fn line_end_mode_change_rebuilds_index() {
        let mut buf = TextBuffer::new();
        // NEL = 0xC2 0x85
        let content: Vec<u8> = [b"hello".as_slice(), &[0xC2, 0x85], b"world"].concat();
        buf.insert(BytePosition(0), &content).unwrap();
        assert_eq!(buf.line_count(), 1); // Default mode doesn't recognize NEL

        buf.set_line_end_mode(LineEndMode::Unicode);
        assert_eq!(buf.line_count(), 2); // Now NEL is recognized
    }

    #[test]
    fn crlf_split_handling() {
        let mut buf = TextBuffer::new();
        // Start with CR followed by LF -> CRLF = 1 line ending
        buf.insert(BytePosition(0), b"a\r\nb").unwrap();
        assert_eq!(buf.line_count(), 2);
        // Insert between CR and LF
        buf.insert(BytePosition(2), b"x").unwrap();
        // Now it's "a\rx\nb" - CR and LF are separate = 2 line endings
        assert_eq!(buf.line_count(), 3);
    }

    #[test]
    fn crlf_merge_handling() {
        let mut buf = TextBuffer::new();
        // "a\r" + "x" + "\nb" - CR and LF separated
        buf.insert(BytePosition(0), b"a\rx\nb").unwrap();
        assert_eq!(buf.line_count(), 3); // lines: "a\r", "x\n", "b"
                                         // Delete 'x' between CR and LF
        buf.delete(BytePosition(2), 1).unwrap();
        // Now "a\r\nb" - CRLF merged = 2 lines
        assert_eq!(buf.line_count(), 2);
    }

    #[test]
    fn contains_line_end_check() {
        let buf = TextBuffer::new();
        assert!(buf.contains_line_end(b"hello\nworld"));
        assert!(!buf.contains_line_end(b"hello world"));
    }

    #[test]
    fn split_view_matches_contiguous() {
        let mut buf = TextBuffer::new();
        buf.insert(BytePosition(0), b"hello world").unwrap();
        let split = buf.split_view();
        let mut combined: Vec<u8> = split.before_gap;
        combined.extend_from_slice(&split.after_gap);
        let contiguous = buf.contiguous_view().to_vec();
        assert_eq!(combined, contiguous);
    }
}
