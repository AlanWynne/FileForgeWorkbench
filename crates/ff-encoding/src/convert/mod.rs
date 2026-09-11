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

mod codecs;
mod stream;

use codecs::{
    convert_dbcs_to_utf8, convert_single_byte_to_utf8, convert_utf16_to_utf8, convert_utf8_to_dbcs,
    convert_utf8_to_single_byte, convert_utf8_to_utf16,
};

pub use stream::{StreamDecoder, StreamEncoder};

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
