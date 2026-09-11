//! Streaming chunk-based encoder/decoder built on the conversion API.

use crate::encoding::Encoding;
use crate::error::EncodingError;

use super::{convert_from_utf8, convert_to_utf8, ConversionResult, UnmappableAction};

/// A streaming decoder that converts chunks from source encoding to UTF-8.
///
/// [Requirement 3.8]
#[derive(Debug)]
pub struct StreamDecoder {
    source_encoding: Encoding,
    /// Buffer for incomplete multi-byte sequences at chunk boundaries
    pending: Vec<u8>,
}

impl StreamDecoder {
    /// Create a new streaming decoder for the given source encoding.
    pub fn new(source_encoding: &Encoding) -> Self {
        Self {
            source_encoding: source_encoding.clone(),
            pending: Vec::new(),
        }
    }

    /// Decode a chunk of bytes, returning the converted UTF-8 result.
    ///
    /// Incomplete multi-byte sequences at the end of a chunk are buffered
    /// for the next call.
    pub fn decode_chunk(&mut self, chunk: &[u8]) -> Result<ConversionResult, EncodingError> {
        let mut input = Vec::with_capacity(self.pending.len() + chunk.len());
        input.extend_from_slice(&self.pending);
        input.extend_from_slice(chunk);
        self.pending.clear();

        // Check for incomplete UTF-8/multi-byte sequence at end
        if self.source_encoding.family == crate::encoding::EncodingFamily::Utf8 {
            let trailing = count_incomplete_utf8_trailing(&input);
            if trailing > 0 {
                let split = input.len() - trailing;
                self.pending = input[split..].to_vec();
                return convert_to_utf8(&input[..split], &self.source_encoding);
            }
        }

        convert_to_utf8(&input, &self.source_encoding)
    }

    /// Finish decoding, flushing any remaining buffered bytes.
    pub fn finish(self) -> Result<ConversionResult, EncodingError> {
        if self.pending.is_empty() {
            return Ok(ConversionResult {
                data: Vec::new(),
                issues: Vec::new(),
            });
        }
        convert_to_utf8(&self.pending, &self.source_encoding)
    }
}

/// A streaming encoder that converts UTF-8 chunks to target encoding.
///
/// [Requirement 4.8]
#[derive(Debug)]
pub struct StreamEncoder {
    target_encoding: Encoding,
    unmappable_action: UnmappableAction,
    /// Buffer for incomplete UTF-8 sequences at chunk boundaries
    pending: String,
}

impl StreamEncoder {
    /// Create a new streaming encoder for the given target encoding.
    pub fn new(target_encoding: &Encoding, unmappable_action: UnmappableAction) -> Self {
        Self {
            target_encoding: target_encoding.clone(),
            unmappable_action,
            pending: String::new(),
        }
    }

    /// Encode a chunk of UTF-8 text to the target encoding.
    pub fn encode_chunk(&mut self, text: &str) -> Result<ConversionResult, EncodingError> {
        let full_text = if self.pending.is_empty() {
            text.to_string()
        } else {
            let mut s = std::mem::take(&mut self.pending);
            s.push_str(text);
            s
        };

        convert_from_utf8(&full_text, &self.target_encoding, self.unmappable_action)
    }

    /// Finish encoding, flushing any remaining buffered text.
    pub fn finish(self) -> Result<ConversionResult, EncodingError> {
        if self.pending.is_empty() {
            return Ok(ConversionResult {
                data: Vec::new(),
                issues: Vec::new(),
            });
        }
        convert_from_utf8(&self.pending, &self.target_encoding, self.unmappable_action)
    }
}

/// Count incomplete UTF-8 trailing bytes at the end of a buffer.
fn count_incomplete_utf8_trailing(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }

    // Look backwards for an incomplete sequence
    let len = bytes.len();
    for i in 1..=4.min(len) {
        let pos = len - i;
        let byte = bytes[pos];
        if byte < 0x80 {
            return 0; // ASCII — complete
        }
        if byte >= 0xC2 {
            // This is a lead byte — check if the sequence is complete
            let expected = crate::utf8::utf8_byte_length_from_lead(byte);
            if i < expected {
                return i; // Incomplete
            }
            return 0; // Complete
        }
        // Continue byte (0x80-0xBF) — keep looking back
    }
    0
}
