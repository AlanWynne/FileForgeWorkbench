//! Continued `impl Document`: the CR-CH-058 record model (Req 11) and the
//! piece-table-spine SAVE entry points (Req 12.12). Split from `document.rs`
//! for the 400-line rule; these delegate to the `TextBuffer` via the existing
//! `text_buffer` / `text_buffer_mut` crate-internal accessors.

use crate::document::Document;
use crate::record_format::RecordFormat;
use crate::types::{BytePosition, RecordNumber};

impl Document {
    /// The document's record format. Defaults to native `Delimited` (generalises
    /// `LineEndMode`); the owning Command Environment supplies it on open
    /// (Req 11.2). A Delimited record IS a line (Req 11.3).
    pub fn record_format(&self) -> RecordFormat {
        self.text_buffer().record_format()
    }

    /// Set the record format (the owning CE supplies it on open, Req 11.2).
    /// Fixed/Variable records are NOT flattened to delimiters (Req 11.5).
    pub fn set_record_format(&mut self, format: RecordFormat) {
        self.text_buffer_mut().set_record_format(format);
    }

    /// Total number of records (== `line_count()` for a Delimited document).
    pub fn total_records(&self) -> u64 {
        self.text_buffer().total_records()
    }

    /// Byte offset of record `k`'s first byte (== `line_start(k)` for Delimited).
    pub fn record_start(&self, k: RecordNumber) -> BytePosition {
        self.text_buffer().record_start(k)
    }

    /// Byte length (including terminator/slot) of record `k`.
    pub fn record_byte_length(&self, k: RecordNumber) -> u64 {
        self.text_buffer().record_byte_length(k)
    }

    /// The record containing byte position `pos` (== `line_from_position(pos)`
    /// for a Delimited document).
    pub fn record_from_position(&self, position: BytePosition) -> RecordNumber {
        self.text_buffer().record_from_position(position)
    }

    /// The re-baseline SAVE byte image (CR-CH-058 Req 12.12). For a native
    /// (Delimited) document this is byte-identical to the current content, so a
    /// save round-trip is byte-identical. The owning CE / shell performs the
    /// atomic write (CR-CH-053 Task 20/21 seam) and then calls `rebaseline`.
    pub fn save_image(&self) -> Vec<u8> {
        self.text_buffer().save_image()
    }

    /// Re-baseline after a save: the saved bytes become the new original image
    /// (CR-CH-058 SAVE = re-baseline). F1-resident.
    pub fn rebaseline(&mut self) {
        self.text_buffer_mut().rebaseline();
    }
}

#[cfg(test)]
#[path = "document_records_tests.rs"]
mod tests;
