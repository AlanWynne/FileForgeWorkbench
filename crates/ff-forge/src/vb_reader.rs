//! Variable-length binary (VB) record reader with RDW support.
//!
//! Handles IBM mainframe VB binary files where each record is prefixed
//! with a 4-byte Record Descriptor Word (RDW):
//! - Bytes 0–1: big-endian u16 record length (includes the 4-byte RDW)
//! - Bytes 2–3: reserved (must be 0x0000)

use crate::error::FileForgeError;

/// The 4-byte Record Descriptor Word (RDW) prefix on a VB binary record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RdwHeader {
    /// Total record length including the 4-byte RDW (minimum value: 4).
    pub record_length: u16,
    /// Reserved bytes (expected to be 0x0000).
    pub reserved: u16,
}

impl RdwHeader {
    /// Returns the content length (record_length minus the 4-byte RDW).
    pub fn content_length(&self) -> u16 {
        self.record_length.saturating_sub(4)
    }

    /// Serializes this RDW to a 4-byte array.
    pub fn to_bytes(&self) -> [u8; 4] {
        let len_bytes = self.record_length.to_be_bytes();
        let res_bytes = self.reserved.to_be_bytes();
        [len_bytes[0], len_bytes[1], res_bytes[0], res_bytes[1]]
    }

    /// Creates an RDW for a record with the given content length.
    pub fn for_content_length(content_length: u16) -> Self {
        Self {
            record_length: content_length + 4,
            reserved: 0,
        }
    }
}

/// Parses an RDW from a 4-byte slice at the given offset in the data.
///
/// # Errors
///
/// Returns `FileForgeError::UnexpectedEof` if fewer than 4 bytes remain.
/// Returns `FileForgeError::InvalidRdw` if L < 4 or reserved bytes are non-zero.
pub fn parse_rdw(data: &[u8], offset: usize) -> Result<RdwHeader, FileForgeError> {
    if offset + 4 > data.len() {
        return Err(FileForgeError::UnexpectedEof {
            byte_offset: offset as u64,
            expected: 4,
        });
    }

    let record_length = u16::from_be_bytes([data[offset], data[offset + 1]]);
    let reserved = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);

    if record_length < 4 {
        return Err(FileForgeError::InvalidRdw {
            byte_offset: offset as u64,
            reason: format!("record length {record_length} is less than minimum 4"),
        });
    }

    if reserved != 0 {
        return Err(FileForgeError::InvalidRdw {
            byte_offset: offset as u64,
            reason: format!("reserved bytes are 0x{reserved:04X}, expected 0x0000"),
        });
    }

    Ok(RdwHeader {
        record_length,
        reserved,
    })
}

/// A record read from a VB binary file.
#[derive(Debug, Clone, PartialEq)]
pub struct VbRecord {
    /// Byte offset of the content start (after RDW) in the source file.
    pub content_offset: u64,
    /// The record content bytes (excluding RDW).
    pub content: Vec<u8>,
}

/// Iterates over VB records in a byte buffer, yielding records and building an index.
///
/// Stops on the first invalid RDW or EOF within a record.
pub struct VbRecordIterator<'a> {
    data: &'a [u8],
    position: usize,
    records_read: usize,
}

impl<'a> VbRecordIterator<'a> {
    /// Creates a new iterator over the given VB binary data.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: 0,
            records_read: 0,
        }
    }

    /// Returns the number of records successfully read so far.
    pub fn records_read(&self) -> usize {
        self.records_read
    }

    /// Returns the current byte position in the source data.
    pub fn position(&self) -> usize {
        self.position
    }
}

impl<'a> Iterator for VbRecordIterator<'a> {
    type Item = Result<VbRecord, FileForgeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.data.len() {
            return None;
        }

        // Parse RDW
        let rdw = match parse_rdw(self.data, self.position) {
            Ok(rdw) => rdw,
            Err(e) => {
                // Advance past end to stop iteration
                self.position = self.data.len();
                return Some(Err(e));
            }
        };

        let content_length = rdw.content_length() as usize;
        let content_start = self.position + 4;
        let record_end = self.position + rdw.record_length as usize;

        // Check if content extends past EOF
        if record_end > self.data.len() {
            // Advance past end to stop iteration
            self.position = self.data.len();
            return Some(Err(FileForgeError::UnexpectedEof {
                byte_offset: self.position as u64,
                expected: rdw.record_length as usize,
            }));
        }

        let content = self.data[content_start..content_start + content_length].to_vec();
        let record = VbRecord {
            content_offset: content_start as u64,
            content,
        };

        self.position = record_end;
        self.records_read += 1;

        Some(Ok(record))
    }
}

/// Builds a byte-offset index from VB binary data.
///
/// Returns a vector of content start offsets (after each RDW) for O(1) random access.
///
/// # Errors
///
/// Returns an error tuple containing the error and the number of records
/// successfully indexed before the error.
pub fn build_vb_index(data: &[u8]) -> Result<Vec<u64>, (FileForgeError, usize)> {
    let mut offsets = Vec::new();
    let iter = VbRecordIterator::new(data);

    for result in iter {
        match result {
            Ok(record) => {
                offsets.push(record.content_offset);
            }
            Err(e) => {
                return Err((e, offsets.len()));
            }
        }
    }

    Ok(offsets)
}

/// Writes a VB record with its RDW prefix.
///
/// Returns the complete record bytes (RDW + content).
pub fn write_vb_record(content: &[u8]) -> Vec<u8> {
    let rdw = RdwHeader::for_content_length(content.len() as u16);
    let mut result = Vec::with_capacity(content.len() + 4);
    result.extend_from_slice(&rdw.to_bytes());
    result.extend_from_slice(content);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 6.2
    #[test]
    fn parse_rdw_valid_record() {
        // Length = 84 (0x0054), reserved = 0x0000
        let data = [0x00, 0x54, 0x00, 0x00];
        let rdw = parse_rdw(&data, 0).unwrap();
        assert_eq!(rdw.record_length, 84);
        assert_eq!(rdw.reserved, 0);
        assert_eq!(rdw.content_length(), 80);
    }

    // Validates: Requirement 6.2
    #[test]
    fn parse_rdw_minimum_length_4() {
        let data = [0x00, 0x04, 0x00, 0x00];
        let rdw = parse_rdw(&data, 0).unwrap();
        assert_eq!(rdw.record_length, 4);
        assert_eq!(rdw.content_length(), 0);
    }

    // Validates: Requirement 6.3
    #[test]
    fn parse_rdw_length_less_than_4_returns_error() {
        let data = [0x00, 0x03, 0x00, 0x00];
        let result = parse_rdw(&data, 0);
        assert!(matches!(result, Err(FileForgeError::InvalidRdw { .. })));
    }

    #[test]
    fn parse_rdw_reserved_nonzero_returns_error() {
        let data = [0x00, 0x10, 0x00, 0x01];
        let result = parse_rdw(&data, 0);
        assert!(matches!(result, Err(FileForgeError::InvalidRdw { .. })));
    }

    #[test]
    fn parse_rdw_insufficient_data_returns_eof() {
        let data = [0x00, 0x10];
        let result = parse_rdw(&data, 0);
        assert!(matches!(result, Err(FileForgeError::UnexpectedEof { .. })));
    }

    // Validates: Requirement 6.4
    #[test]
    fn vb_iterator_reads_multiple_records() {
        // Two records: first is 8 bytes (4 RDW + 4 content), second is 6 bytes (4 + 2)
        let mut data = Vec::new();
        data.extend_from_slice(&[0x00, 0x08, 0x00, 0x00]); // RDW: len=8
        data.extend_from_slice(&[0x41, 0x42, 0x43, 0x44]); // content: "ABCD" (or bytes)
        data.extend_from_slice(&[0x00, 0x06, 0x00, 0x00]); // RDW: len=6
        data.extend_from_slice(&[0x45, 0x46]); // content: "EF"

        let records: Vec<_> = VbRecordIterator::new(&data)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].content, vec![0x41, 0x42, 0x43, 0x44]);
        assert_eq!(records[0].content_offset, 4);
        assert_eq!(records[1].content, vec![0x45, 0x46]);
        assert_eq!(records[1].content_offset, 12);
    }

    // Validates: Requirement 6.3
    #[test]
    fn vb_iterator_stops_on_read_past_eof() {
        // RDW says 100 bytes but only 10 available
        let mut data = Vec::new();
        data.extend_from_slice(&[0x00, 0x64, 0x00, 0x00]); // RDW: len=100
        data.extend_from_slice(&[0x00; 6]); // Only 6 content bytes

        let results: Vec<_> = VbRecordIterator::new(&data).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }

    // Validates: Requirement 6.5
    #[test]
    fn vb_records_show_content_only_not_rdw() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x00, 0x0A, 0x00, 0x00]); // RDW: len=10
        data.extend_from_slice(&[0x48, 0x45, 0x4C, 0x4C, 0x4F, 0x00]); // "HELLO\0"

        let records: Vec<_> = VbRecordIterator::new(&data)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(records[0].content, vec![0x48, 0x45, 0x4C, 0x4C, 0x4F, 0x00]);
        // RDW bytes should NOT be in content
        assert_ne!(records[0].content[0], 0x00);
    }

    // Validates: Requirement 6.6
    #[test]
    fn write_vb_record_includes_correct_rdw() {
        let content = vec![0x41, 0x42, 0x43]; // 3 bytes
        let written = write_vb_record(&content);
        assert_eq!(written.len(), 7); // 4 RDW + 3 content
                                      // RDW should be 0x0007 (7 = 3 + 4)
        assert_eq!(written[0..2], [0x00, 0x07]);
        assert_eq!(written[2..4], [0x00, 0x00]); // reserved
        assert_eq!(written[4..], content);
    }

    #[test]
    fn build_vb_index_returns_content_offsets() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x00, 0x08, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]);
        data.extend_from_slice(&[0x00, 0x06, 0x00, 0x00, 0x05, 0x06]);

        let offsets = build_vb_index(&data).unwrap();
        assert_eq!(offsets, vec![4, 12]);
    }

    #[test]
    fn build_vb_index_partial_on_error() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x00, 0x08, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]); // valid
        data.extend_from_slice(&[0x00, 0x02, 0x00, 0x00]); // invalid: len < 4

        let result = build_vb_index(&data);
        assert!(result.is_err());
        let (_, records_read) = result.unwrap_err();
        assert_eq!(records_read, 1);
    }

    #[test]
    fn rdw_to_bytes_roundtrip() {
        let rdw = RdwHeader {
            record_length: 100,
            reserved: 0,
        };
        let bytes = rdw.to_bytes();
        let parsed = parse_rdw(&bytes, 0).unwrap();
        assert_eq!(parsed, rdw);
    }
}
