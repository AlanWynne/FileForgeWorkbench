//! Append_Buffer: append-only byte store for edited/inserted record bytes.
//!
//! This is the surviving role of the `GapBuffer` under the windowed piece-table
//! model (CR-CH-058). The Append_Buffer grows with EDITS, not with file size:
//! original record bytes stay in the (fully-resident in F1) original image and
//! are addressed through the Immutable_Original_Index, while inserted/edited
//! record bytes are appended here and addressed by a `BufRange`.

use crate::gap_buffer::GapBuffer;
use crate::types::BufRange;

/// Append-only store of edited/inserted record bytes.
///
/// Wraps a `GapBuffer` (reused unchanged) and only ever appends at the end, so
/// an earlier `BufRange` remains stable as later edits append more bytes.
#[derive(Debug, Clone)]
pub struct AppendBuffer {
    buffer: GapBuffer,
    len: u64,
}

impl Default for AppendBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl AppendBuffer {
    /// Create an empty append buffer.
    pub fn new() -> Self {
        Self {
            buffer: GapBuffer::default_new(),
            len: 0,
        }
    }

    /// Total number of bytes appended so far.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Whether no bytes have been appended.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Append `data` and return the stable range of the appended bytes.
    pub fn append(&mut self, data: &[u8]) -> BufRange {
        let start = self.len;
        if !data.is_empty() {
            self.buffer.insert(self.len, data);
            self.len += data.len() as u64;
        }
        BufRange::new(start, data.len() as u64)
    }

    /// Read back the bytes of a previously appended range.
    ///
    /// Returns `None` if the range falls outside the appended content.
    pub fn bytes(&self, range: BufRange) -> Option<Vec<u8>> {
        if range.end() > self.len {
            return None;
        }
        self.buffer.get_range(range.start, range.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        // Validates: Requirement 12.1
        let ab = AppendBuffer::new();
        assert!(ab.is_empty());
        assert_eq!(ab.len(), 0);
    }

    #[test]
    fn append_returns_range_and_reads_back() {
        // Validates: Requirement 12.1
        let mut ab = AppendBuffer::new();
        let r1 = ab.append(b"hello");
        assert_eq!(r1, BufRange::new(0, 5));
        assert_eq!(ab.bytes(r1), Some(b"hello".to_vec()));
        assert_eq!(ab.len(), 5);
    }

    #[test]
    fn earlier_range_stable_after_later_appends() {
        // Validates: Requirement 12.1
        let mut ab = AppendBuffer::new();
        let r1 = ab.append(b"first");
        let r2 = ab.append(b"second");
        let r3 = ab.append(b"third");
        assert_eq!(ab.bytes(r1), Some(b"first".to_vec()));
        assert_eq!(ab.bytes(r2), Some(b"second".to_vec()));
        assert_eq!(ab.bytes(r3), Some(b"third".to_vec()));
        assert_eq!(r2, BufRange::new(5, 6));
        assert_eq!(r3, BufRange::new(11, 5));
    }

    #[test]
    fn empty_append_is_zero_length_range() {
        // Validates: Requirement 12.1
        let mut ab = AppendBuffer::new();
        let r = ab.append(b"");
        assert_eq!(r, BufRange::new(0, 0));
        assert_eq!(ab.bytes(r), Some(Vec::new()));
        assert_eq!(ab.len(), 0);
    }

    #[test]
    fn out_of_range_read_is_none() {
        // Validates: Requirement 12.1
        let mut ab = AppendBuffer::new();
        ab.append(b"abc");
        assert_eq!(ab.bytes(BufRange::new(0, 10)), None);
    }
}
