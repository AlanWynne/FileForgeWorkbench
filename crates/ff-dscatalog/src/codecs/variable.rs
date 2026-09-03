//! Variable-length record codec (RECFM=V, VB).
//!
//! Each record is preceded by a 4-byte Record Descriptor Word (RDW).
//! The RDW encodes the total record length including the 4-byte RDW itself.
//! No CRLF or LF delimiter follows the data bytes.
//!
//! RDW layout (big-endian):
//!   bytes 0-1: total length of this record including RDW (u16 big-endian)
//!   bytes 2-3: reserved, must be 0x00 0x00
//!
//! Validates: Requirement 16.3, 17.3

use super::{CodecError, RecordCodec};

const RDW_LEN: usize = 4;
const RDW_MIN: usize = RDW_LEN; // minimum valid RDW value (empty record)

/// Encodes and decodes variable-length records using 4-byte RDW headers.
///
/// On encode, each record is prefixed with a 4-byte RDW.
/// On decode, malformed RDWs produce `CodecError::MalformedRdw` with position info.
#[derive(Debug, Clone)]
pub struct VariableCodec {
    dataset: String,
}

impl VariableCodec {
    /// Create a new `VariableCodec`.
    ///
    /// `dataset` is used only in error messages.
    pub fn new(dataset: impl Into<String>) -> Self {
        Self {
            dataset: dataset.into(),
        }
    }

    fn make_rdw(total_len: usize) -> [u8; RDW_LEN] {
        let len = total_len as u16;
        [
            (len >> 8) as u8,
            (len & 0xFF) as u8,
            0x00,
            0x00,
        ]
    }
}

impl RecordCodec for VariableCodec {
    /// Encode records into RDW-prefixed bytes.
    ///
    /// Each record is prefixed with a 4-byte RDW encoding `record.len() + 4`.
    fn encode(&self, records: &[Vec<u8>]) -> Result<Vec<u8>, CodecError> {
        // Validates: Requirement 16.3
        let total = records.iter().map(|r| RDW_LEN + r.len()).sum();
        let mut out = Vec::with_capacity(total);
        for rec in records {
            let rdw = Self::make_rdw(RDW_LEN + rec.len());
            out.extend_from_slice(&rdw);
            out.extend_from_slice(rec);
        }
        Ok(out)
    }

    /// Decode RDW-prefixed bytes into records.
    ///
    /// Returns `CodecError::MalformedRdw` with record index and byte offset
    /// if any RDW is invalid.
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, CodecError> {
        // Validates: Requirement 16.3, 16.7, 17.3
        let mut records = Vec::new();
        let mut offset = 0usize;
        let mut record_index = 0usize;

        while offset < bytes.len() {
            if offset + RDW_LEN > bytes.len() {
                return Err(CodecError::MalformedRdw {
                    record_index,
                    byte_offset: offset,
                    dataset: self.dataset.clone(),
                    reason: format!(
                        "truncated RDW: need {} bytes but only {} remain",
                        RDW_LEN,
                        bytes.len() - offset
                    ),
                });
            }

            let rdw_total = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;

            if rdw_total < RDW_MIN {
                return Err(CodecError::MalformedRdw {
                    record_index,
                    byte_offset: offset,
                    dataset: self.dataset.clone(),
                    reason: format!(
                        "RDW total length {rdw_total} is less than minimum {RDW_MIN}"
                    ),
                });
            }

            if offset + rdw_total > bytes.len() {
                return Err(CodecError::MalformedRdw {
                    record_index,
                    byte_offset: offset,
                    dataset: self.dataset.clone(),
                    reason: format!(
                        "RDW claims {rdw_total} bytes but only {} remain",
                        bytes.len() - offset
                    ),
                });
            }

            let data = bytes[offset + RDW_LEN..offset + rdw_total].to_vec();
            records.push(data);
            offset += rdw_total;
            record_index += 1;
        }

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_prefixes_each_record_with_rdw() {
        // Validates: Requirement 16.3, 17.3
        let codec = VariableCodec::new("TEST.DS");
        let records = vec![vec![b'A', b'B', b'C']];
        let bytes = codec.encode(&records).unwrap();
        // RDW = 4 + 3 = 7, big-endian: 0x00 0x07 0x00 0x00
        assert_eq!(bytes[0..4], [0x00, 0x07, 0x00, 0x00]);
        assert_eq!(bytes[4..], [b'A', b'B', b'C']);
    }

    #[test]
    fn encode_no_crlf_between_records() {
        // Validates: Requirement 16.3 -- no CRLF after data bytes
        let codec = VariableCodec::new("TEST.DS");
        let records = vec![vec![1u8, 2], vec![3u8, 4, 5]];
        let bytes = codec.encode(&records).unwrap();
        // No 0x0D or 0x0A anywhere
        assert!(!bytes.contains(&0x0D));
        assert!(!bytes.contains(&0x0A));
    }

    #[test]
    fn decode_recovers_records_from_rdw_stream() {
        // Validates: Requirement 16.3, 17.3
        let codec = VariableCodec::new("TEST.DS");
        let records = vec![vec![1u8, 2, 3], vec![4u8, 5]];
        let bytes = codec.encode(&records).unwrap();
        let decoded = codec.decode(&bytes).unwrap();
        assert_eq!(decoded, records);
    }

    #[test]
    fn decode_empty_bytes_returns_empty_records() {
        // Validates: Requirement 17.3
        let codec = VariableCodec::new("TEST.DS");
        let records = codec.decode(&[]).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn decode_truncated_rdw_returns_error_with_position() {
        // Validates: Requirement 16.7, 17.3
        let codec = VariableCodec::new("TEST.DS");
        // Only 3 bytes -- not enough for a full RDW
        let err = codec.decode(&[0x00, 0x07, 0x00]).unwrap_err();
        assert!(matches!(
            err,
            CodecError::MalformedRdw { record_index: 0, byte_offset: 0, .. }
        ));
    }

    #[test]
    fn decode_rdw_below_minimum_returns_error() {
        // Validates: Requirement 16.7
        let codec = VariableCodec::new("TEST.DS");
        // RDW total = 3, below minimum of 4
        let bytes = [0x00, 0x03, 0x00, 0x00];
        let err = codec.decode(&bytes).unwrap_err();
        assert!(matches!(err, CodecError::MalformedRdw { .. }));
    }

    #[test]
    fn decode_rdw_claims_more_bytes_than_available_returns_error() {
        // Validates: Requirement 16.7
        let codec = VariableCodec::new("TEST.DS");
        // RDW claims 100 bytes but only 4 present
        let bytes = [0x00, 0x64, 0x00, 0x00];
        let err = codec.decode(&bytes).unwrap_err();
        assert!(matches!(err, CodecError::MalformedRdw { .. }));
    }

    #[test]
    fn encode_decode_round_trip_multiple_records() {
        // Validates: Requirement 17.6
        let codec = VariableCodec::new("TEST.DS");
        let original = vec![
            vec![1u8, 2, 3],
            vec![],
            vec![4u8, 5, 6, 7, 8],
        ];
        let bytes = codec.encode(&original).unwrap();
        let decoded = codec.decode(&bytes).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn error_contains_dataset_identity_and_position() {
        // Validates: Requirement 16.7
        let codec = VariableCodec::new("PAYROLL.INPUT");
        let bytes = [0x00, 0x03, 0x00, 0x00]; // RDW below minimum
        let err = codec.decode(&bytes).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("PAYROLL.INPUT"));
        assert!(msg.contains("record 0"));
    }
}
