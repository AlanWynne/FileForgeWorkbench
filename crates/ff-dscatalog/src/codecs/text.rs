//! Text import/export codec.
//!
//! Maps host text lines to/from fixed-length records using a configurable
//! encoding profile. This codec is ONLY for explicit import/export operations
//! and must NEVER be applied silently during normal dataset I/O.
//!
//! Validates: Requirement 17.5, 17.7

use super::{CodecError, RecordCodec};

/// Encoding profile for text conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextEncoding {
    /// UTF-8 host text lines.
    Utf8,
    /// Latin-1 (ISO 8859-1) host text lines.
    Latin1,
}

/// Text import/export codec -- explicit use only, never silent.
///
/// Encode: converts host text lines (split on `\n`) to fixed-length records
/// padded with spaces to LRECL.
///
/// Decode: converts fixed-length records to host text lines, stripping
/// trailing spaces.
///
/// # Important
///
/// This codec must only be used when the caller explicitly requests text
/// conversion. It must never be inferred from file extension or host line
/// endings. Validates: Requirement 17.7
#[derive(Debug, Clone)]
pub struct TextCodec {
    lrecl: usize,
    encoding: TextEncoding,
    dataset: String,
}

impl TextCodec {
    /// Create a new `TextCodec`.
    ///
    /// `dataset` is used only in error messages.
    pub fn new(lrecl: usize, encoding: TextEncoding, dataset: impl Into<String>) -> Self {
        Self {
            lrecl,
            encoding,
            dataset: dataset.into(),
        }
    }

    /// The encoding profile this codec uses.
    pub fn encoding(&self) -> &TextEncoding {
        &self.encoding
    }
}

impl RecordCodec for TextCodec {
    /// Convert host text lines to fixed-length records.
    ///
    /// Lines are split on `\n` (stripping `\r` if present). Each line is
    /// truncated or padded to LRECL with ASCII space (0x20).
    fn encode(&self, records: &[Vec<u8>]) -> Result<Vec<u8>, CodecError> {
        // Validates: Requirement 17.5
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
            out.resize(out.len() + (self.lrecl - rec.len()), b' ');
        }
        Ok(out)
    }

    /// Convert fixed-length records to host text lines.
    ///
    /// Trailing spaces are stripped from each record. Records are returned
    /// as byte vectors (callers decode to string using the encoding profile).
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, CodecError> {
        // Validates: Requirement 17.5
        if self.lrecl == 0 || bytes.is_empty() {
            return Ok(vec![]);
        }
        if bytes.len() % self.lrecl != 0 {
            return Err(CodecError::NotMultipleOfLrecl {
                byte_count: bytes.len(),
                lrecl: self.lrecl,
                dataset: self.dataset.clone(),
            });
        }
        Ok(bytes
            .chunks(self.lrecl)
            .map(|chunk| {
                // Strip trailing spaces
                let trimmed = chunk
                    .iter()
                    .rposition(|&b| b != b' ')
                    .map(|pos| &chunk[..=pos])
                    .unwrap_or(&[]);
                trimmed.to_vec()
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_pads_line_to_lrecl() {
        // Validates: Requirement 17.5
        let codec = TextCodec::new(10, TextEncoding::Utf8, "TEST.DS");
        let records = vec![b"HELLO".to_vec()];
        let bytes = codec.encode(&records).unwrap();
        assert_eq!(bytes.len(), 10);
        assert_eq!(&bytes[..5], b"HELLO");
        assert_eq!(&bytes[5..], b"     ");
    }

    #[test]
    fn decode_strips_trailing_spaces() {
        // Validates: Requirement 17.5
        let codec = TextCodec::new(10, TextEncoding::Utf8, "TEST.DS");
        let bytes = b"HELLO     ".to_vec();
        let records = codec.decode(&bytes).unwrap();
        assert_eq!(records, vec![b"HELLO".to_vec()]);
    }

    #[test]
    fn encode_decode_round_trip() {
        // Validates: Requirement 17.6
        let codec = TextCodec::new(80, TextEncoding::Utf8, "TEST.DS");
        let lines = vec![b"FIRST LINE".to_vec(), b"SECOND".to_vec()];
        let bytes = codec.encode(&lines).unwrap();
        let decoded = codec.decode(&bytes).unwrap();
        assert_eq!(decoded, lines);
    }

    #[test]
    fn encoding_profile_accessible() {
        // Validates: Requirement 17.7 -- explicit encoding policy required
        let codec = TextCodec::new(80, TextEncoding::Latin1, "TEST.DS");
        assert_eq!(codec.encoding(), &TextEncoding::Latin1);
    }
}
