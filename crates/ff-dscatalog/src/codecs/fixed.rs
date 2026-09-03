//! Fixed-length record codec (RECFM=F, FB).
//!
//! Records are packed contiguously as N x LRECL bytes with no inter-record
//! delimiters. Record n is located at byte offset n x LRECL.
//!
//! Validates: Requirement 16.2, 17.2

use super::{CodecError, RecordCodec};

/// Encodes and decodes fixed-length records given an LRECL value.
///
/// On encode, records shorter than LRECL are padded with spaces (0x40).
/// Records longer than LRECL are rejected with `CodecError::RecordTooLong`.
///
/// On decode, the byte slice must be an exact multiple of LRECL.
#[derive(Debug, Clone)]
pub struct FixedCodec {
    lrecl: usize,
    dataset: String,
}

impl FixedCodec {
    /// Create a new `FixedCodec` for the given LRECL.
    ///
    /// `dataset` is used only in error messages.
    pub fn new(lrecl: usize, dataset: impl Into<String>) -> Self {
        Self {
            lrecl,
            dataset: dataset.into(),
        }
    }
}

impl RecordCodec for FixedCodec {
    /// Encode records into contiguous fixed-length bytes.
    ///
    /// Each record is padded to LRECL with 0x40 (EBCDIC space) if shorter.
    /// Returns `CodecError::RecordTooLong` if any record exceeds LRECL.
    fn encode(&self, records: &[Vec<u8>]) -> Result<Vec<u8>, CodecError> {
        // Validates: Requirement 16.2
        let mut out = Vec::with_capacity(records.len() * self.lrecl);
        for (i, rec) in records.iter().enumerate() {
            if rec.len() > self.lrecl {
                return Err(CodecError::RecordTooLong {
                    record_index: i,
                    record_len: rec.len(),
                    lrecl: self.lrecl,
                    dataset: self.dataset.clone(),
                });
            }
            out.extend_from_slice(rec);
            // Pad to LRECL with EBCDIC space (0x40)
            out.resize(out.len() + (self.lrecl - rec.len()), 0x40);
        }
        Ok(out)
    }

    /// Decode contiguous bytes into fixed-length records.
    ///
    /// Returns `CodecError::NotMultipleOfLrecl` if `bytes.len()` is not a
    /// multiple of LRECL.
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, CodecError> {
        // Validates: Requirement 16.2, 17.2
        if self.lrecl == 0 {
            return Ok(vec![]);
        }
        if bytes.len() % self.lrecl != 0 {
            return Err(CodecError::NotMultipleOfLrecl {
                byte_count: bytes.len(),
                lrecl: self.lrecl,
                dataset: self.dataset.clone(),
            });
        }
        Ok(bytes.chunks(self.lrecl).map(|c| c.to_vec()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_pads_short_record_to_lrecl() {
        // Validates: Requirement 16.2, 17.2
        let codec = FixedCodec::new(5, "TEST.DS");
        let records = vec![vec![b'A', b'B']];
        let bytes = codec.encode(&records).unwrap();
        assert_eq!(bytes, vec![b'A', b'B', 0x40, 0x40, 0x40]);
    }

    #[test]
    fn encode_exact_length_record_no_padding() {
        // Validates: Requirement 16.2, 17.2
        let codec = FixedCodec::new(3, "TEST.DS");
        let records = vec![vec![1u8, 2, 3]];
        let bytes = codec.encode(&records).unwrap();
        assert_eq!(bytes, vec![1, 2, 3]);
    }

    #[test]
    fn encode_multiple_records_contiguous() {
        // Validates: Requirement 16.2 -- no inter-record delimiters
        let codec = FixedCodec::new(4, "TEST.DS");
        let records = vec![vec![1u8, 2, 3, 4], vec![5u8, 6, 7, 8]];
        let bytes = codec.encode(&records).unwrap();
        assert_eq!(bytes, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn encode_rejects_record_longer_than_lrecl() {
        // Validates: Requirement 17.2
        let codec = FixedCodec::new(3, "TEST.DS");
        let records = vec![vec![1u8, 2, 3, 4]];
        let err = codec.encode(&records).unwrap_err();
        assert!(matches!(err, CodecError::RecordTooLong { record_index: 0, .. }));
    }

    #[test]
    fn decode_splits_bytes_into_records() {
        // Validates: Requirement 16.2, 17.2
        let codec = FixedCodec::new(3, "TEST.DS");
        let bytes = vec![1u8, 2, 3, 4, 5, 6];
        let records = codec.decode(&bytes).unwrap();
        assert_eq!(records, vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }

    #[test]
    fn decode_rejects_non_multiple_of_lrecl() {
        // Validates: Requirement 16.7, 17.2
        let codec = FixedCodec::new(3, "TEST.DS");
        let bytes = vec![1u8, 2, 3, 4];
        let err = codec.decode(&bytes).unwrap_err();
        assert!(matches!(
            err,
            CodecError::NotMultipleOfLrecl { byte_count: 4, lrecl: 3, .. }
        ));
    }

    #[test]
    fn decode_empty_bytes_returns_empty_records() {
        // Validates: Requirement 17.2
        let codec = FixedCodec::new(80, "TEST.DS");
        let records = codec.decode(&[]).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn encode_decode_round_trip() {
        // Validates: Requirement 17.6
        let codec = FixedCodec::new(10, "TEST.DS");
        let original = vec![
            b"HELLO     ".to_vec(),
            b"WORLD     ".to_vec(),
        ];
        let bytes = codec.encode(&original).unwrap();
        let decoded = codec.decode(&bytes).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn record_n_at_offset_n_times_lrecl() {
        // Validates: Requirement 16.2 -- record n at offset n x LRECL
        let lrecl = 5;
        let codec = FixedCodec::new(lrecl, "TEST.DS");
        let records = vec![
            vec![1u8, 2, 3, 4, 5],
            vec![6u8, 7, 8, 9, 10],
            vec![11u8, 12, 13, 14, 15],
        ];
        let bytes = codec.encode(&records).unwrap();
        for (n, rec) in records.iter().enumerate() {
            let offset = n * lrecl;
            assert_eq!(&bytes[offset..offset + lrecl], rec.as_slice());
        }
    }
}
