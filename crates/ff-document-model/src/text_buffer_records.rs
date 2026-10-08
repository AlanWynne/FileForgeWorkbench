//! Continued `impl TextBuffer`: the CR-CH-058 record-query API (Req 11.4) and
//! the piece-table-spine SAVE entry points (Req 12.1/12.12). Split from
//! `text_buffer.rs` for the 400-line rule. These use only the public/crate
//! methods on `TextBuffer` (`line_*`, `length`, `record_format`,
//! `build_original_index`, `image_bytes`, `rebuild_line_index`), so no private
//! field access is needed.
//!
//! For a Delimited document a record IS a line: the record API delegates to the
//! line index so record and line agree byte-for-byte (AC 11.3). F1 keeps the
//! whole file resident; these entry points build the piece-table layers from the
//! resident image to PROVE the spine and the byte-identical SAVE round-trip
//! without yet routing interactive edits through pieces (that, and windowing,
//! are F2).

use crate::original_index::OriginalIndex;
use crate::piece_list::PieceList;
use crate::text_buffer::TextBuffer;
use crate::types::{BytePosition, LineNumber, RecordNumber};

impl TextBuffer {
    /// Total number of records. For Delimited this equals `line_count()`
    /// (a record IS a line); otherwise it is derived from the record framing.
    pub fn total_records(&self) -> u64 {
        if self.record_format().is_delimited() {
            self.line_count()
        } else {
            self.build_original_index().total_records()
        }
    }

    /// Byte offset of record `k`'s first byte. For Delimited this equals
    /// `line_start(k)` (record IS line).
    pub fn record_start(&self, k: RecordNumber) -> BytePosition {
        if self.record_format().is_delimited() {
            self.line_start(LineNumber(k.0))
        } else {
            BytePosition(self.build_original_index().record_start(k))
        }
    }

    /// Byte length (including terminator/slot) of record `k`.
    pub fn record_byte_length(&self, k: RecordNumber) -> u64 {
        if self.record_format().is_delimited() {
            // A Delimited record spans from its start to the next record's
            // start (or the document end for the last record), terminator
            // included -- byte-identical to the index-built length.
            let start = self.line_start(LineNumber(k.0)).0;
            let total = self.line_count();
            let end = if k.0 + 1 < total {
                self.line_start(LineNumber(k.0 + 1)).0
            } else {
                self.length()
            };
            end.saturating_sub(start)
        } else {
            self.build_original_index().record_byte_length(k) as u64
        }
    }

    /// The record containing byte position `pos`. For Delimited this equals
    /// `line_from_position(pos)` (record IS line).
    pub fn record_from_position(&self, pos: BytePosition) -> RecordNumber {
        if self.record_format().is_delimited() {
            RecordNumber(self.line_from_position(pos).0)
        } else {
            RecordNumber(
                self.build_original_index()
                    .record_from_offset(pos.0)
                    .map(|r| r.0)
                    .unwrap_or(0),
            )
        }
    }

    /// A single Original piece spanning the whole document (the clean,
    /// post-open / post-rebaseline arrangement, AC 12.1).
    //
    // allow(dead_code): F1 PROVES the baseline piece list (exercised by tests);
    // the F2 windowed edit path is its non-test consumer. Remove the allow when
    // F2 routes interactive edits through the piece list.
    #[allow(dead_code)]
    pub(crate) fn baseline_piece_list(&self) -> PieceList {
        PieceList::from_original(self.total_records())
    }

    /// Produce the full byte image of the document AS CURRENTLY STORED. This is
    /// the re-baseline SAVE image: for a resident Delimited document it is the
    /// exact resident bytes, so a save round-trip is byte-identical (AC 12.12).
    pub fn save_image(&self) -> Vec<u8> {
        self.image_bytes()
    }

    /// Re-baseline after a save: the saved bytes become the new original image.
    /// F1-resident, this re-frames the resident buffer and resets the line
    /// index; the caller (Document/CE) owns the atomic write and journal reset.
    /// (CR-CH-058 SAVE = re-baseline.)
    pub fn rebaseline(&mut self) {
        self.rebuild_line_index();
    }
}
