//! BOM (Byte Order Mark) detection and writing.
//!
//! Provides detection of BOM sequences at the start of byte streams and
//! writing of BOM bytes for encoding output.

use std::io::Write;

use crate::error::EncodingError;

/// Information about a detected BOM.
///
/// [Requirement 2]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BomInfo {
    /// The encoding indicated by the BOM
    pub encoding: BomEncoding,
    /// Length of the BOM in bytes (2, 3, or 4)
    pub length: usize,
}

/// Encodings that can be identified via BOM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomEncoding {
    /// UTF-8 BOM: EF BB BF
    Utf8,
    /// UTF-16 Little Endian BOM: FF FE
    Utf16Le,
    /// UTF-16 Big Endian BOM: FE FF
    Utf16Be,
    /// UTF-32 Little Endian BOM: FF FE 00 00
    Utf32Le,
    /// UTF-32 Big Endian BOM: 00 00 FE FF
    Utf32Be,
}

/// UTF-8 BOM bytes.
const BOM_UTF8: &[u8] = &[0xEF, 0xBB, 0xBF];
/// UTF-16LE BOM bytes.
const BOM_UTF16LE: &[u8] = &[0xFF, 0xFE];
/// UTF-16BE BOM bytes.
const BOM_UTF16BE: &[u8] = &[0xFE, 0xFF];
/// UTF-32LE BOM bytes.
const BOM_UTF32LE: &[u8] = &[0xFF, 0xFE, 0x00, 0x00];
/// UTF-32BE BOM bytes.
const BOM_UTF32BE: &[u8] = &[0x00, 0x00, 0xFE, 0xFF];

/// Detect a BOM at the start of a byte slice.
///
/// Checks UTF-32 (4-byte) BOMs before UTF-16 (2-byte) to correctly
/// disambiguate UTF-32LE from UTF-16LE + NUL.
///
/// Returns `None` if no BOM is present.
pub fn detect_bom(bytes: &[u8]) -> Option<BomInfo> {
    // Check 4-byte BOMs first to disambiguate UTF-32LE from UTF-16LE
    if bytes.len() >= 4 {
        if bytes[..4] == *BOM_UTF32LE {
            return Some(BomInfo {
                encoding: BomEncoding::Utf32Le,
                length: 4,
            });
        }
        if bytes[..4] == *BOM_UTF32BE {
            return Some(BomInfo {
                encoding: BomEncoding::Utf32Be,
                length: 4,
            });
        }
    }
    // Check 3-byte UTF-8 BOM
    if bytes.len() >= 3 && bytes[..3] == *BOM_UTF8 {
        return Some(BomInfo {
            encoding: BomEncoding::Utf8,
            length: 3,
        });
    }
    // Check 2-byte UTF-16 BOMs
    if bytes.len() >= 2 {
        if bytes[..2] == *BOM_UTF16BE {
            return Some(BomInfo {
                encoding: BomEncoding::Utf16Be,
                length: 2,
            });
        }
        if bytes[..2] == *BOM_UTF16LE {
            return Some(BomInfo {
                encoding: BomEncoding::Utf16Le,
                length: 2,
            });
        }
    }
    None
}

/// Return the BOM bytes for a given BOM encoding.
pub fn bom_bytes(encoding: BomEncoding) -> &'static [u8] {
    match encoding {
        BomEncoding::Utf8 => BOM_UTF8,
        BomEncoding::Utf16Le => BOM_UTF16LE,
        BomEncoding::Utf16Be => BOM_UTF16BE,
        BomEncoding::Utf32Le => BOM_UTF32LE,
        BomEncoding::Utf32Be => BOM_UTF32BE,
    }
}

/// Write the BOM bytes for a given encoding to a writer.
///
/// # Errors
///
/// Returns `EncodingError::Io` if writing fails.
pub fn write_bom(encoding: BomEncoding, writer: &mut dyn Write) -> Result<(), EncodingError> {
    writer.write_all(bom_bytes(encoding))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_bom_utf8() {
        // Validates: Requirement 2.1, 2.2
        let data = [0xEF, 0xBB, 0xBF, b'H', b'e', b'l', b'l', b'o'];
        let result = detect_bom(&data).expect("should detect UTF-8 BOM");
        assert_eq!(result.encoding, BomEncoding::Utf8);
        assert_eq!(result.length, 3);
    }

    #[test]
    fn detect_bom_utf16le() {
        // Validates: Requirement 2.1, 2.2
        let data = [0xFF, 0xFE, b'H', 0x00];
        let result = detect_bom(&data).expect("should detect UTF-16LE BOM");
        assert_eq!(result.encoding, BomEncoding::Utf16Le);
        assert_eq!(result.length, 2);
    }

    #[test]
    fn detect_bom_utf16be() {
        // Validates: Requirement 2.1, 2.2
        let data = [0xFE, 0xFF, 0x00, b'H'];
        let result = detect_bom(&data).expect("should detect UTF-16BE BOM");
        assert_eq!(result.encoding, BomEncoding::Utf16Be);
        assert_eq!(result.length, 2);
    }

    #[test]
    fn detect_bom_utf32le() {
        // Validates: Requirement 2.1, 2.2, 2.3
        let data = [0xFF, 0xFE, 0x00, 0x00, b'H', 0x00, 0x00, 0x00];
        let result = detect_bom(&data).expect("should detect UTF-32LE BOM");
        assert_eq!(result.encoding, BomEncoding::Utf32Le);
        assert_eq!(result.length, 4);
    }

    #[test]
    fn detect_bom_utf32be() {
        // Validates: Requirement 2.1, 2.2
        let data = [0x00, 0x00, 0xFE, 0xFF, 0x00, 0x00, 0x00, b'H'];
        let result = detect_bom(&data).expect("should detect UTF-32BE BOM");
        assert_eq!(result.encoding, BomEncoding::Utf32Be);
        assert_eq!(result.length, 4);
    }

    #[test]
    fn detect_bom_disambiguates_utf32le_from_utf16le_with_nul() {
        // Validates: Requirement 2.3
        // FF FE 00 00 should be UTF-32LE, not UTF-16LE + NUL char
        let data = [0xFF, 0xFE, 0x00, 0x00];
        let result = detect_bom(&data).expect("should detect UTF-32LE");
        assert_eq!(result.encoding, BomEncoding::Utf32Le);
        assert_eq!(result.length, 4);
    }

    #[test]
    fn detect_bom_no_bom_present() {
        // Validates: Requirement 2.8
        let data = b"Hello, world!";
        assert!(detect_bom(data).is_none());
    }

    #[test]
    fn detect_bom_empty_input() {
        assert!(detect_bom(&[]).is_none());
    }

    #[test]
    fn detect_bom_too_short_for_utf8() {
        let data = [0xEF, 0xBB]; // Only 2 bytes, need 3
        assert!(detect_bom(&data).is_none());
    }

    #[test]
    fn bom_bytes_returns_correct_sequences() {
        // Validates: Requirement 2.5, 2.6
        assert_eq!(bom_bytes(BomEncoding::Utf8), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(bom_bytes(BomEncoding::Utf16Le), &[0xFF, 0xFE]);
        assert_eq!(bom_bytes(BomEncoding::Utf16Be), &[0xFE, 0xFF]);
        assert_eq!(bom_bytes(BomEncoding::Utf32Le), &[0xFF, 0xFE, 0x00, 0x00]);
        assert_eq!(bom_bytes(BomEncoding::Utf32Be), &[0x00, 0x00, 0xFE, 0xFF]);
    }

    #[test]
    fn write_bom_writes_correct_bytes() {
        // Validates: Requirement 2.5, 2.7
        let mut buf = Vec::new();
        write_bom(BomEncoding::Utf8, &mut buf).unwrap();
        assert_eq!(buf, &[0xEF, 0xBB, 0xBF]);

        let mut buf = Vec::new();
        write_bom(BomEncoding::Utf16Le, &mut buf).unwrap();
        assert_eq!(buf, &[0xFF, 0xFE]);
    }
}
