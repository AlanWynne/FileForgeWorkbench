//! Immutable_Original_Index: one lean ~16-byte entry per original record.
//!
//! Built on open from the fully-resident bytes (F1), never mutated after the
//! open scan. Provides O(1) "where is original record K" (AC 12.1). The index
//! sits behind the `OriginalIndex` trait so a later above-budget sparse/mmap
//! mode (F5) can replace the resident `Vec` implementation without touching
//! callers (AC 12.11).

use crate::line_end::LineEndMode;
use crate::record_format::{self, RecordFormat};
use crate::types::RecordNumber;

/// A single lean index entry: `file_offset` + `byte_length` + `flags`.
///
/// Laid out as `u64 + u32 + u32` = 16 bytes so a 100M-record index is ~1.6 GB
/// (AC 12.3 memory-budget target). `byte_length` INCLUDES the record's
/// terminator (Delimited) so re-emit is byte-identical (AC 12.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct OriginalIndexEntry {
    /// Byte offset of the record's first byte in the original file image.
    pub file_offset: u64,
    /// Total byte length of the record including its terminator/slot.
    pub byte_length: u32,
    /// Reserved flags (unused in F1; kept for the 16-byte budget layout).
    pub flags: u32,
}

/// Read-only access to the immutable original record index.
///
/// F5: an above-budget sparse/mmap implementation replaces
/// `ResidentOriginalIndex` behind this trait; callers depend only on the trait.
pub trait OriginalIndex: std::fmt::Debug {
    /// Total number of original records.
    fn total_records(&self) -> u64;

    /// The entry for original record `k`, or `None` if out of range.
    fn entry(&self, k: RecordNumber) -> Option<OriginalIndexEntry>;

    /// Byte offset of original record `k`'s first byte. Returns the image
    /// length (one past the last byte) for `k` at or beyond the end.
    fn record_start(&self, k: RecordNumber) -> u64;

    /// Byte length (including terminator) of original record `k`, or 0 if out
    /// of range.
    fn record_byte_length(&self, k: RecordNumber) -> u32;

    /// The original record containing byte offset `pos`, or `None` if `pos` is
    /// beyond the image.
    fn record_from_offset(&self, pos: u64) -> Option<RecordNumber>;

    /// Total byte length of the original image (sum of all record lengths).
    fn image_length(&self) -> u64;
}

/// Lean fully-resident `Vec`-backed `OriginalIndex` (the only F1 implementation).
#[derive(Debug, Clone, Default)]
pub struct ResidentOriginalIndex {
    entries: Vec<OriginalIndexEntry>,
    image_length: u64,
}

impl ResidentOriginalIndex {
    /// Build the index from the fully-resident original bytes per `format`.
    pub fn from_bytes(bytes: &[u8], format: RecordFormat, mode: LineEndMode) -> Self {
        let boundaries = record_format::frame_records(bytes, format, mode);
        let entries: Vec<OriginalIndexEntry> = boundaries
            .iter()
            .map(|b| OriginalIndexEntry {
                file_offset: b.start,
                byte_length: b.byte_length,
                flags: 0,
            })
            .collect();
        Self {
            entries,
            image_length: bytes.len() as u64,
        }
    }

    /// An empty-document index: a single zero-length record (AC 4.8).
    pub fn empty() -> Self {
        Self {
            entries: vec![OriginalIndexEntry {
                file_offset: 0,
                byte_length: 0,
                flags: 0,
            }],
            image_length: 0,
        }
    }
}

impl OriginalIndex for ResidentOriginalIndex {
    fn total_records(&self) -> u64 {
        self.entries.len() as u64
    }

    fn entry(&self, k: RecordNumber) -> Option<OriginalIndexEntry> {
        self.entries.get(k.0 as usize).copied()
    }

    fn record_start(&self, k: RecordNumber) -> u64 {
        match self.entries.get(k.0 as usize) {
            Some(e) => e.file_offset,
            None => self.image_length,
        }
    }

    fn record_byte_length(&self, k: RecordNumber) -> u32 {
        self.entries
            .get(k.0 as usize)
            .map(|e| e.byte_length)
            .unwrap_or(0)
    }

    fn record_from_offset(&self, pos: u64) -> Option<RecordNumber> {
        if pos > self.image_length {
            return None;
        }
        // Binary search over the sorted file_offsets for the last start <= pos.
        // An empty trailing record has start == image_length; a position at the
        // image end resolves to the last record.
        let mut lo = 0usize;
        let mut hi = self.entries.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.entries[mid].file_offset <= pos {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 {
            self.entries.first().map(|_| RecordNumber(0))
        } else {
            Some(RecordNumber((lo - 1) as u64))
        }
    }

    fn image_length(&self) -> u64 {
        self.image_length
    }
}

#[cfg(test)]
#[path = "original_index_tests.rs"]
mod tests;
