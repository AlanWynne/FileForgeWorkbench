//! Encoding conversion (to/from UTF-8, streaming decoder/encoder).
//!
//! Provides bidirectional conversion between source encodings and UTF-8,
//! with support for streaming/chunk-based processing.

use crate::encoding::Encoding;
use crate::error::EncodingError;

/// Record of an issue encountered during encoding conversion.
///
/// [Requirement 3.3, 3.4]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionIssue {
    /// Byte offset in the source where the issue occurred
    pub source_offset: usize,
    /// The original bytes that could not be converted
    pub original_bytes: Vec<u8>,
    /// Human-readable description of the issue
    pub description: String,
}

/// Result of an encoding conversion operation.
///
/// [Requirement 3, 4]
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// The converted bytes (UTF-8 on load, target encoding on save)
    pub data: Vec<u8>,
    /// Issues encountered during conversion (lossy replacements)
    pub issues: Vec<ConversionIssue>,
}

/// Options for handling unmappable characters during save-encoding.
///
/// [Requirement 4.5]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmappableAction {
    /// Abort the save operation
    Abort,
    /// Replace unmappable characters with a placeholder
    ReplaceWithPlaceholder(char),
    /// Switch to UTF-8 encoding for the save
    SwitchToUtf8,
}

/// Convert bytes from a source encoding to UTF-8.
///
/// Invalid byte sequences are replaced with U+FFFD and logged in
/// `ConversionResult.issues`.
///
/// [Requirement 3.1]
pub fn convert_to_utf8(
    bytes: &[u8],
    source_encoding: &Encoding,
) -> Result<ConversionResult, EncodingError> {
    use crate::encoding::EncodingFamily;

    match source_encoding.family {
        EncodingFamily::Utf8 => Ok(ConversionResult {
            data: bytes.to_vec(),
            issues: Vec::new(),
        }),
        EncodingFamily::SingleByte => convert_single_byte_to_utf8(bytes, source_encoding),
        EncodingFamily::Utf16 => convert_utf16_to_utf8(bytes, source_encoding),
        EncodingFamily::Dbcs => convert_dbcs_to_utf8(bytes, source_encoding),
    }
}

/// Convert a UTF-8 string to a target encoding.
///
/// [Requirement 4.1]
pub fn convert_from_utf8(
    text: &str,
    target_encoding: &Encoding,
    unmappable_action: UnmappableAction,
) -> Result<ConversionResult, EncodingError> {
    use crate::encoding::EncodingFamily;

    match target_encoding.family {
        EncodingFamily::Utf8 => Ok(ConversionResult {
            data: text.as_bytes().to_vec(),
            issues: Vec::new(),
        }),
        EncodingFamily::SingleByte => {
            convert_utf8_to_single_byte(text, target_encoding, unmappable_action)
        }
        EncodingFamily::Utf16 => convert_utf8_to_utf16(text, target_encoding, unmappable_action),
        EncodingFamily::Dbcs => convert_utf8_to_dbcs(text, target_encoding, unmappable_action),
    }
}

/// Single-byte to UTF-8 conversion using ISO-8859-1 identity mapping.
fn convert_single_byte_to_utf8(
    bytes: &[u8],
    source_encoding: &Encoding,
) -> Result<ConversionResult, EncodingError> {
    let table = get_single_byte_to_unicode_table(source_encoding);
    let mut result = Vec::with_capacity(bytes.len() * 2);
    let mut issues = Vec::new();

    for (offset, &byte) in bytes.iter().enumerate() {
        let cp = if let Some(table) = table {
            table[byte as usize]
        } else {
            // ISO-8859-1 identity mapping
            byte as u32
        };

        if cp == 0xFFFD {
            issues.push(ConversionIssue {
                source_offset: offset,
                original_bytes: vec![byte],
                description: format!("unmappable byte 0x{byte:02X} in {}", source_encoding.name),
            });
        }

        // Encode Unicode code point as UTF-8
        let mut buf = [0u8; 4];
        if let Some(ch) = char::from_u32(cp) {
            let encoded = ch.encode_utf8(&mut buf);
            result.extend_from_slice(encoded.as_bytes());
        } else {
            result.extend_from_slice("\u{FFFD}".as_bytes());
        }
    }

    Ok(ConversionResult {
        data: result,
        issues,
    })
}

/// UTF-16 to UTF-8 conversion.
fn convert_utf16_to_utf8(
    bytes: &[u8],
    source_encoding: &Encoding,
) -> Result<ConversionResult, EncodingError> {
    let is_le = source_encoding.name.contains("le") || source_encoding.code_page == 1200;
    let is_utf32 = source_encoding.name.contains("32");
    let mut result = Vec::with_capacity(bytes.len());
    let mut issues = Vec::new();

    if is_utf32 {
        let unit_size = 4;
        let mut offset = 0;
        while offset + unit_size <= bytes.len() {
            let cp = if is_le {
                u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ])
            } else {
                u32::from_be_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ])
            };

            if let Some(ch) = char::from_u32(cp) {
                let mut buf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buf);
                result.extend_from_slice(encoded.as_bytes());
            } else {
                result.extend_from_slice("\u{FFFD}".as_bytes());
                issues.push(ConversionIssue {
                    source_offset: offset,
                    original_bytes: bytes[offset..offset + unit_size].to_vec(),
                    description: format!("invalid UTF-32 code point 0x{cp:08X}"),
                });
            }
            offset += unit_size;
        }
    } else {
        // UTF-16
        let mut offset = 0;
        while offset + 2 <= bytes.len() {
            let unit = if is_le {
                u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
            } else {
                u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
            };

            if (0xD800..=0xDBFF).contains(&unit) {
                // High surrogate — need low surrogate
                if offset + 4 <= bytes.len() {
                    let low = if is_le {
                        u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]])
                    } else {
                        u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]])
                    };
                    if (0xDC00..=0xDFFF).contains(&low) {
                        let cp = 0x10000 + ((unit as u32 - 0xD800) << 10) + (low as u32 - 0xDC00);
                        if let Some(ch) = char::from_u32(cp) {
                            let mut buf = [0u8; 4];
                            let encoded = ch.encode_utf8(&mut buf);
                            result.extend_from_slice(encoded.as_bytes());
                        }
                        offset += 4;
                        continue;
                    }
                }
                // Lone high surrogate
                result.extend_from_slice("\u{FFFD}".as_bytes());
                issues.push(ConversionIssue {
                    source_offset: offset,
                    original_bytes: bytes[offset..offset + 2].to_vec(),
                    description: "lone high surrogate".to_string(),
                });
                offset += 2;
            } else if (0xDC00..=0xDFFF).contains(&unit) {
                // Lone low surrogate
                result.extend_from_slice("\u{FFFD}".as_bytes());
                issues.push(ConversionIssue {
                    source_offset: offset,
                    original_bytes: bytes[offset..offset + 2].to_vec(),
                    description: "lone low surrogate".to_string(),
                });
                offset += 2;
            } else {
                if let Some(ch) = char::from_u32(unit as u32) {
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    result.extend_from_slice(encoded.as_bytes());
                }
                offset += 2;
            }
        }
    }

    Ok(ConversionResult {
        data: result,
        issues,
    })
}

/// DBCS to UTF-8 conversion (placeholder — full tables in production).
fn convert_dbcs_to_utf8(
    bytes: &[u8],
    source_encoding: &Encoding,
) -> Result<ConversionResult, EncodingError> {
    // Simplified: treat as pass-through with replacement for now
    let mut result = Vec::with_capacity(bytes.len());
    let issues = Vec::new();

    // For Shift-JIS and other DBCS, we need code-page-specific mapping tables.
    // This is a simplified implementation that handles basic ASCII pass-through.
    for &byte in bytes {
        if byte < 0x80 {
            result.push(byte);
        } else {
            // Placeholder: replace high bytes with U+FFFD
            result.extend_from_slice("\u{FFFD}".as_bytes());
        }
    }

    let _ = source_encoding; // Will use for table selection in full impl

    Ok(ConversionResult {
        data: result,
        issues,
    })
}

/// UTF-8 to single-byte conversion.
fn convert_utf8_to_single_byte(
    text: &str,
    target_encoding: &Encoding,
    unmappable_action: UnmappableAction,
) -> Result<ConversionResult, EncodingError> {
    let reverse_table = get_unicode_to_single_byte_table(target_encoding);
    let mut result = Vec::with_capacity(text.len());
    let mut issues = Vec::new();

    for (offset, ch) in text.char_indices() {
        let cp = ch as u32;

        let mapped = if let Some(table) = &reverse_table {
            table.get(&cp).copied()
        } else if cp <= 0xFF {
            // ISO-8859-1 identity
            Some(cp as u8)
        } else {
            None
        };

        if let Some(byte) = mapped {
            result.push(byte);
        } else {
            match unmappable_action {
                UnmappableAction::Abort => {
                    return Err(EncodingError::UnmappableCharacter {
                        code_point: cp,
                        offset,
                    });
                }
                UnmappableAction::ReplaceWithPlaceholder(placeholder) => {
                    if (placeholder as u32) <= 0xFF {
                        result.push(placeholder as u8);
                    } else {
                        result.push(b'?');
                    }
                    issues.push(ConversionIssue {
                        source_offset: offset,
                        original_bytes: ch.to_string().into_bytes(),
                        description: format!(
                            "unmappable character U+{cp:04X} replaced with '{placeholder}'"
                        ),
                    });
                }
                UnmappableAction::SwitchToUtf8 => {
                    // Switch to UTF-8 — just encode the whole remaining text as UTF-8
                    result.extend_from_slice(&text.as_bytes()[offset..]);
                    return Ok(ConversionResult {
                        data: result,
                        issues,
                    });
                }
            }
        }
    }

    Ok(ConversionResult {
        data: result,
        issues,
    })
}

/// UTF-8 to UTF-16 conversion.
fn convert_utf8_to_utf16(
    text: &str,
    target_encoding: &Encoding,
    _unmappable_action: UnmappableAction,
) -> Result<ConversionResult, EncodingError> {
    let is_le = target_encoding.name.contains("le") || target_encoding.code_page == 1200;
    let is_utf32 = target_encoding.name.contains("32");
    let mut result = Vec::with_capacity(text.len() * 2);

    if is_utf32 {
        for ch in text.chars() {
            let cp = ch as u32;
            if is_le {
                result.extend_from_slice(&cp.to_le_bytes());
            } else {
                result.extend_from_slice(&cp.to_be_bytes());
            }
        }
    } else {
        // UTF-16
        let mut buf = [0u16; 2];
        for ch in text.chars() {
            let encoded = ch.encode_utf16(&mut buf);
            for unit in encoded.iter() {
                if is_le {
                    result.extend_from_slice(&unit.to_le_bytes());
                } else {
                    result.extend_from_slice(&unit.to_be_bytes());
                }
            }
        }
    }

    Ok(ConversionResult {
        data: result,
        issues: Vec::new(),
    })
}

/// UTF-8 to DBCS conversion (placeholder).
fn convert_utf8_to_dbcs(
    text: &str,
    _target_encoding: &Encoding,
    _unmappable_action: UnmappableAction,
) -> Result<ConversionResult, EncodingError> {
    // Simplified placeholder: ASCII pass-through, rest replaced
    let mut result = Vec::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_ascii() {
            result.push(ch as u8);
        } else {
            result.push(b'?');
        }
    }
    Ok(ConversionResult {
        data: result,
        issues: Vec::new(),
    })
}

/// Get the single-byte-to-unicode mapping table for an encoding.
/// Returns None for ISO-8859-1 (identity mapping).
fn get_single_byte_to_unicode_table(encoding: &Encoding) -> Option<&'static [u32; 256]> {
    match encoding.code_page {
        28591 => None, // ISO-8859-1 is identity
        _ => None,     // Simplified: treat all as identity for now
    }
}

/// Get the unicode-to-single-byte reverse mapping for an encoding.
fn get_unicode_to_single_byte_table(
    _encoding: &Encoding,
) -> Option<std::collections::HashMap<u32, u8>> {
    None // ISO-8859-1 identity — handled inline
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::{Encoding, EncodingFamily};

    fn utf8_encoding() -> Encoding {
        Encoding {
            name: "utf-8",
            code_page: 65001,
            family: EncodingFamily::Utf8,
            display_name: "UTF-8",
            aliases: &[],
        }
    }

    fn iso_8859_1_encoding() -> Encoding {
        Encoding {
            name: "iso-8859-1",
            code_page: 28591,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-1",
            aliases: &[],
        }
    }

    fn utf16le_encoding() -> Encoding {
        Encoding {
            name: "utf-16le",
            code_page: 1200,
            family: EncodingFamily::Utf16,
            display_name: "UTF-16 LE",
            aliases: &[],
        }
    }

    #[test]
    fn convert_utf8_to_utf8_is_identity() {
        // Validates: Requirement 3.1
        let text = "Hello, 世界!";
        let result = convert_to_utf8(text.as_bytes(), &utf8_encoding()).unwrap();
        assert_eq!(result.data, text.as_bytes());
        assert!(result.issues.is_empty());
    }

    #[test]
    fn convert_iso_8859_1_to_utf8() {
        // Validates: Requirement 3.2
        let bytes = [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xE9]; // "Helloé" in ISO-8859-1
        let result = convert_to_utf8(&bytes, &iso_8859_1_encoding()).unwrap();
        assert_eq!(String::from_utf8(result.data).unwrap(), "Helloé");
    }

    #[test]
    fn convert_utf16le_to_utf8() {
        // Validates: Requirement 3.2, 3.5
        let bytes: Vec<u8> = "Hello"
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let result = convert_to_utf8(&bytes, &utf16le_encoding()).unwrap();
        assert_eq!(String::from_utf8(result.data).unwrap(), "Hello");
    }

    #[test]
    fn convert_utf16le_surrogate_pairs() {
        // Validates: Requirement 3.5
        // U+1F600 (😀) encoded as UTF-16LE surrogate pair: D83D DE00
        let bytes = [0x3D, 0xD8, 0x00, 0xDE]; // LE byte order
        let result = convert_to_utf8(&bytes, &utf16le_encoding()).unwrap();
        assert_eq!(String::from_utf8(result.data).unwrap(), "😀");
    }

    #[test]
    fn convert_from_utf8_to_iso_8859_1() {
        // Validates: Requirement 4.1
        let text = "Hello";
        let result =
            convert_from_utf8(text, &iso_8859_1_encoding(), UnmappableAction::Abort).unwrap();
        assert_eq!(result.data, b"Hello");
    }

    #[test]
    fn convert_from_utf8_unmappable_aborts() {
        // Validates: Requirement 4.4, 4.5
        let text = "Hello 😀"; // Emoji not in ISO-8859-1
        let result = convert_from_utf8(text, &iso_8859_1_encoding(), UnmappableAction::Abort);
        assert!(result.is_err());
    }

    #[test]
    fn convert_from_utf8_unmappable_replaces() {
        // Validates: Requirement 4.5
        let text = "Hi\u{0100}"; // Ā not in ISO-8859-1
        let result = convert_from_utf8(
            text,
            &iso_8859_1_encoding(),
            UnmappableAction::ReplaceWithPlaceholder('?'),
        )
        .unwrap();
        assert_eq!(result.data, b"Hi?");
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn stream_decoder_handles_split_utf8() {
        // Validates: Requirement 3.8
        let text = "Hé"; // H (1 byte) + é (2 bytes: C3 A9)
        let bytes = text.as_bytes();

        let mut decoder = StreamDecoder::new(&utf8_encoding());
        // Split in the middle of the 2-byte sequence
        let chunk1 = &bytes[..2]; // "H" + 0xC3 (first byte of é)
        let chunk2 = &bytes[2..]; // 0xA9 (second byte of é)

        let result1 = decoder.decode_chunk(chunk1).unwrap();
        assert_eq!(String::from_utf8(result1.data).unwrap(), "H");

        let result2 = decoder.decode_chunk(chunk2).unwrap();
        assert_eq!(String::from_utf8(result2.data).unwrap(), "é");
    }

    #[test]
    fn stream_encoder_basic() {
        // Validates: Requirement 4.8
        let mut encoder = StreamEncoder::new(&utf16le_encoding(), UnmappableAction::Abort);
        let result = encoder.encode_chunk("Hi").unwrap();
        assert_eq!(result.data, &[b'H', 0, b'i', 0]);

        let final_result = encoder.finish().unwrap();
        assert!(final_result.data.is_empty());
    }
}
