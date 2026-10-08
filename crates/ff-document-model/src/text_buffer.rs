//! TextBuffer: primary text storage combining GapBuffer with LineIndex.
//!
//! Coordinates insertion/deletion with line tracking, CRLF edge case handling,
//! and read-only guards.

use crate::error::DocumentError;
use crate::gap_buffer::GapBuffer;
use crate::line_end::{self, LineEndMode};
use crate::line_index::LineIndex;
use crate::original_index::ResidentOriginalIndex;
use crate::record_format::RecordFormat;
use crate::types::{BytePosition, DeleteResult, InsertResult, LineNumber, SplitView};

/// Primary text storage: owns the GapBuffer and maintains the LineIndex.
/// Coordinates insertion/deletion with line tracking and read-only guards.
///
/// CR-CH-058 F1: the buffer also carries a `RecordFormat` (the native CE
/// supplies `Delimited`, generalising `LineEndMode`) and exposes a record-API
/// (`total_records` / `record_start` / `record_byte_length` /
/// `record_from_position`). For a Delimited document a record IS a line, so the
/// record-API and the line-API agree byte-for-byte (AC 11.3, 11.4). F1 keeps
/// the whole file resident in the gap buffer; the piece-table spine is proven
/// via `save_image` / `rebaseline` (AC 12.1, 12.2, 12.12). Windowed residency
/// and piece-table-backed interactive editing are F2.
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
    /// The record framing (CR-CH-058). Native = Delimited from `line_end_mode`.
    record_format: RecordFormat,
}

impl TextBuffer {
    /// Create an empty text buffer.
    pub fn new() -> Self {
        Self {
            buffer: GapBuffer::default_new(),
            line_index: LineIndex::new(),
            line_end_mode: LineEndMode::Default,
            read_only: false,
            record_format: RecordFormat::native(LineEndMode::Default),
        }
    }

    /// Create a text buffer with pre-allocated capacity.
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            buffer: GapBuffer::new(capacity),
            line_index: LineIndex::new(),
            line_end_mode: LineEndMode::Default,
            read_only: false,
            record_format: RecordFormat::native(LineEndMode::Default),
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

        // Insert into the gap buffer, then rebuild the line index over the full
        // buffer (CRLF split/merge across the insertion boundary is handled by
        // the rebuild, so `line_count()` is always exact). `lines_added` is the
        // count of line endings within the inserted text.
        self.buffer.insert(position.0, text);
        let lines_added = line_end::count_line_endings(text, self.line_end_mode);
        self.rebuild_line_index();

        Ok(InsertResult {
            lines_added,
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
            // Keep the native (Delimited) record format in step with the mode
            // (the native CE generalises LineEndMode into RecordFormat, Req
            // 11.2). A non-Delimited format (mainframe) is left untouched.
            if self.record_format.is_delimited() {
                self.record_format = RecordFormat::native(mode);
            }
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

    // === Record API (CR-CH-058 Req 11.4) ====================================
    //
    // For a Delimited document a record IS a line: these delegate to the line
    // index so the record-API and line-API agree byte-for-byte (AC 11.3). The
    // native record format generalises `LineEndMode` (AC 11.2); Fixed/Variable
    // framing is built from the resident bytes and is NOT flattened to
    // delimiters (AC 11.5), though only Delimited is exercised end-to-end in F1.

    // The record-query API and the piece-table-spine SAVE methods are a
    // continued `impl TextBuffer` in `text_buffer_records.rs` (kept separate for
    // the 400-line rule). The private-field-touching helpers they call stay here.

    /// The document's record format (CR-CH-058).
    pub fn record_format(&self) -> RecordFormat {
        self.record_format
    }

    /// Set the record format (the owning CE supplies it on open, Req 11.2).
    pub fn set_record_format(&mut self, format: RecordFormat) {
        self.record_format = format;
    }

    /// Build the Immutable_Original_Index over the current resident bytes per
    /// the active record format (CR-CH-058 AC 12.1). F1-resident: rebuilt from
    /// the whole buffer; F2 builds it incrementally/windowed.
    pub(crate) fn build_original_index(&self) -> ResidentOriginalIndex {
        let bytes = self.image_bytes();
        if bytes.is_empty() {
            ResidentOriginalIndex::empty()
        } else {
            ResidentOriginalIndex::from_bytes(&bytes, self.record_format, self.line_end_mode)
        }
    }

    /// The current content as a contiguous byte image (resident in F1). This is
    /// the re-baseline SAVE image source (byte-identical for Delimited).
    pub(crate) fn image_bytes(&self) -> Vec<u8> {
        self.buffer
            .get_range(0, self.buffer.length())
            .unwrap_or_default()
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
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "text_buffer_tests.rs"]
mod tests;
