//! Binary pass-through codec (RECFM=U).
//!
//! Treats content as an opaque binary stream and preserves bytes exactly
//! without interpretation. Encode concatenates all records; decode returns
//! the entire byte slice as a single record.
//!
//! Validates: Requirement 16.4, 17.4

use super::{CodecError, RecordCodec};

/// Pass-through codec for RECFM=U (undefined record format) datasets.
///
/// Encode: concatenates all records into a single byte stream.
/// Decode: returns the entire byte slice as one record.
#[derive(Debug, Clone, Default)]
pub struct BinaryCodec;

impl RecordCodec for BinaryCodec {
    /// Concatenate all records into a single byte stream.
    fn encode(&self, records: &[Vec<u8>]) -> Result<Vec<u8>, CodecError> {
        // Validates: Requirement 16.4, 17.4
        let total: usize = records.iter().map(|r| r.len()).sum();
        let mut out = Vec::with_capacity(total);
        for rec in records {
            out.extend_from_slice(rec);
        }
        Ok(out)
    }

    /// Return the entire byte slice as a single record.
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, CodecError> {
        // Validates: Requirement 16.4, 17.4
        if bytes.is_empty() {
            return Ok(vec![]);
        }
        Ok(vec![bytes.to_vec()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_concatenates_records() {
        // Validates: Requirement 16.4, 17.4
        let codec = BinaryCodec;
        let records = vec![vec![1u8, 2], vec![3u8, 4, 5]];
        let bytes = codec.encode(&records).unwrap();
        assert_eq!(bytes, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn decode_returns_single_record() {
        // Validates: Requirement 16.4, 17.4
        let codec = BinaryCodec;
        let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let records = codec.decode(&bytes).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], bytes);
    }

    #[test]
    fn decode_empty_returns_empty() {
        // Validates: Requirement 17.4
        let codec = BinaryCodec;
        let records = codec.decode(&[]).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn encode_preserves_bytes_exactly() {
        // Validates: Requirement 16.4 -- bytes preserved without interpretation
        let codec = BinaryCodec;
        let data = vec![0x00u8, 0x0A, 0x0D, 0xFF, 0x1B];
        let records = vec![data.clone()];
        let bytes = codec.encode(&records).unwrap();
        assert_eq!(bytes, data);
    }

    #[test]
    fn encode_decode_round_trip() {
        // Validates: Requirement 17.6
        let codec = BinaryCodec;
        let original = vec![vec![0xCAu8, 0xFE, 0xBA, 0xBE]];
        let bytes = codec.encode(&original).unwrap();
        let decoded = codec.decode(&bytes).unwrap();
        // Round-trip: single record containing all bytes
        let rejoined: Vec<u8> = decoded.into_iter().flatten().collect();
        let expected: Vec<u8> = original.into_iter().flatten().collect();
        assert_eq!(rejoined, expected);
    }
}
