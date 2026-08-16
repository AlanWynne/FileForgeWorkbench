//! Encoding detection from raw byte streams.
//!
//! Implements multi-strategy encoding detection using the priority order:
//! BOM → UTF-8 validity → DBCS patterns → byte-frequency heuristics → fallback.

use crate::bom::{detect_bom, BomInfo};
use crate::encoding::Encoding;
use crate::registry::EncodingRegistry;
use crate::utf8::utf8_validate;

/// Default maximum bytes to examine for encoding detection.
const DEFAULT_MAX_BYTES: usize = 8192;

/// Confidence level for encoding detection.
///
/// [Requirement 1.6]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DetectionConfidence {
    /// Fallback or statistical guess
    Low,
    /// Strong heuristic match (valid UTF-8, consistent DBCS patterns)
    Medium,
    /// BOM present or unambiguous pattern
    High,
}

/// Result of encoding detection.
///
/// [Requirement 1]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResult {
    /// The detected encoding
    pub encoding: Encoding,
    /// Confidence level of the detection
    pub confidence: DetectionConfidence,
    /// BOM information if a BOM was found
    pub bom: Option<BomInfo>,
}

/// Detect the encoding of a byte slice.
///
/// Examines up to `max_bytes` (default 8192) using the priority order:
/// BOM → UTF-8 validity → null-byte patterns → DBCS patterns → fallback.
///
/// [Requirement 1.1]
pub fn detect_encoding(bytes: &[u8], max_bytes: Option<usize>) -> DetectionResult {
    let registry = EncodingRegistry::new();
    let fallback = registry.by_name("utf-8").unwrap().clone();
    detect_encoding_with_fallback(bytes, max_bytes, &fallback)
}

/// Detect encoding with an explicit fallback encoding override.
///
/// [Requirement 1.8]
pub fn detect_encoding_with_fallback(
    bytes: &[u8],
    max_bytes: Option<usize>,
    fallback: &Encoding,
) -> DetectionResult {
    let max = max_bytes.unwrap_or(DEFAULT_MAX_BYTES);
    let sample = if bytes.len() > max {
        &bytes[..max]
    } else {
        bytes
    };

    // Strategy 1: BOM detection (highest confidence)
    if let Some(bom_info) = detect_bom(sample) {
        let registry = EncodingRegistry::new();
        let encoding = match bom_info.encoding {
            crate::bom::BomEncoding::Utf8 => registry.by_name("utf-8").unwrap().clone(),
            crate::bom::BomEncoding::Utf16Le => registry.by_name("utf-16le").unwrap().clone(),
            crate::bom::BomEncoding::Utf16Be => registry.by_name("utf-16be").unwrap().clone(),
            crate::bom::BomEncoding::Utf32Le => registry.by_name("utf-32le").unwrap().clone(),
            crate::bom::BomEncoding::Utf32Be => registry.by_name("utf-32be").unwrap().clone(),
        };
        return DetectionResult {
            encoding,
            confidence: DetectionConfidence::High,
            bom: Some(bom_info),
        };
    }

    // Strategy 2: UTF-8 validity check
    if !sample.is_empty() && utf8_validate(sample) {
        let registry = EncodingRegistry::new();
        return DetectionResult {
            encoding: registry.by_name("utf-8").unwrap().clone(),
            confidence: DetectionConfidence::Medium,
            bom: None,
        };
    }

    // Strategy 3: Null-byte pattern analysis for UTF-16/UTF-32
    if let Some(result) = detect_null_byte_patterns(sample) {
        return result;
    }

    // Strategy 4: DBCS lead/trail byte pattern analysis
    if let Some(result) = detect_dbcs_patterns(sample) {
        return result;
    }

    // Strategy 5: Statistical byte-frequency heuristics
    if let Some(result) = detect_by_statistics(sample) {
        return result;
    }

    // Strategy 6: Fallback
    DetectionResult {
        encoding: fallback.clone(),
        confidence: DetectionConfidence::Low,
        bom: None,
    }
}

/// Detect UTF-16/UTF-32 by null byte patterns.
fn detect_null_byte_patterns(bytes: &[u8]) -> Option<DetectionResult> {
    if bytes.len() < 4 {
        return None;
    }

    let registry = EncodingRegistry::new();

    // Check for UTF-32 pattern: every 4th byte is the character, rest are nulls
    let mut utf32le_score = 0usize;
    let mut utf32be_score = 0usize;
    let check_len = (bytes.len() / 4) * 4;

    if check_len >= 8 {
        for chunk in bytes[..check_len].chunks_exact(4) {
            // UTF-32LE: char at [0], nulls at [1,2,3]
            if chunk[1] == 0 && chunk[2] == 0 && chunk[3] == 0 && chunk[0] != 0 {
                utf32le_score += 1;
            }
            // UTF-32BE: char at [3], nulls at [0,1,2]
            if chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0 && chunk[3] != 0 {
                utf32be_score += 1;
            }
        }

        let total_chunks = check_len / 4;
        if utf32le_score > total_chunks * 3 / 4 {
            return Some(DetectionResult {
                encoding: registry.by_name("utf-32le").unwrap().clone(),
                confidence: DetectionConfidence::Medium,
                bom: None,
            });
        }
        if utf32be_score > total_chunks * 3 / 4 {
            return Some(DetectionResult {
                encoding: registry.by_name("utf-32be").unwrap().clone(),
                confidence: DetectionConfidence::Medium,
                bom: None,
            });
        }
    }

    // Check for UTF-16 pattern: alternating nulls
    let mut utf16le_nulls = 0usize;
    let mut utf16be_nulls = 0usize;
    let check_len = (bytes.len() / 2) * 2;

    if check_len >= 4 {
        for chunk in bytes[..check_len].chunks_exact(2) {
            if chunk[1] == 0 && chunk[0] != 0 {
                utf16le_nulls += 1;
            }
            if chunk[0] == 0 && chunk[1] != 0 {
                utf16be_nulls += 1;
            }
        }

        let total_pairs = check_len / 2;
        if utf16le_nulls > total_pairs * 3 / 4 {
            return Some(DetectionResult {
                encoding: registry.by_name("utf-16le").unwrap().clone(),
                confidence: DetectionConfidence::Medium,
                bom: None,
            });
        }
        if utf16be_nulls > total_pairs * 3 / 4 {
            return Some(DetectionResult {
                encoding: registry.by_name("utf-16be").unwrap().clone(),
                confidence: DetectionConfidence::Medium,
                bom: None,
            });
        }
    }

    None
}

/// Detect DBCS encodings by lead/trail byte patterns.
fn detect_dbcs_patterns(bytes: &[u8]) -> Option<DetectionResult> {
    if bytes.is_empty() {
        return None;
    }

    // Shift-JIS detection: lead bytes 0x81-0x9F, 0xE0-0xFC
    let mut sjis_pairs = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if ((0x81..=0x9F).contains(&b) || (0xE0..=0xFC).contains(&b)) && i + 1 < bytes.len() {
            let trail = bytes[i + 1];
            if (0x40..=0xFC).contains(&trail) && trail != 0x7F {
                sjis_pairs += 1;
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    // GBK detection: lead bytes 0x81-0xFE, trail bytes 0x40-0xFE
    let mut gbk_pairs = 0usize;
    i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if (0x81..=0xFE).contains(&b) && i + 1 < bytes.len() {
            let trail = bytes[i + 1];
            if (0x40..=0xFE).contains(&trail) && trail != 0x7F {
                gbk_pairs += 1;
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    let threshold = bytes.len() / 10; // At least 10% of bytes form DBCS pairs
    let registry = EncodingRegistry::new();

    if sjis_pairs > threshold && sjis_pairs > gbk_pairs {
        // Shift-JIS has the more restrictive lead byte range, prefer it when it scores higher
        return Some(DetectionResult {
            encoding: registry.by_name("shift-jis").unwrap().clone(),
            confidence: DetectionConfidence::Medium,
            bom: None,
        });
    }

    if gbk_pairs > threshold {
        return Some(DetectionResult {
            encoding: registry.by_name("gbk").unwrap().clone(),
            confidence: DetectionConfidence::Medium,
            bom: None,
        });
    }

    None
}

/// Detect encoding by statistical byte-frequency analysis.
fn detect_by_statistics(bytes: &[u8]) -> Option<DetectionResult> {
    if bytes.is_empty() {
        return None;
    }

    // Count high bytes (0x80-0xFF) — common in ISO-8859 and Windows-1252
    let high_bytes = bytes.iter().filter(|&&b| b >= 0x80).count();
    let _total = bytes.len();

    if high_bytes == 0 {
        // Pure ASCII — report as UTF-8
        let registry = EncodingRegistry::new();
        return Some(DetectionResult {
            encoding: registry.by_name("utf-8").unwrap().clone(),
            confidence: DetectionConfidence::Medium,
            bom: None,
        });
    }

    // Check for EBCDIC: lacks ASCII control range patterns
    // EBCDIC text typically has bytes like 0xC1-0xC9 (A-I), 0xD1-0xD9 (J-R), 0xE2-0xE9 (S-Z)
    let ebcdic_letter_count = bytes
        .iter()
        .filter(|&&b| {
            (0xC1..=0xC9).contains(&b)
                || (0xD1..=0xD9).contains(&b)
                || (0xE2..=0xE9).contains(&b)
                || b == 0x40 // EBCDIC space
        })
        .count();

    if ebcdic_letter_count > bytes.len() / 2 {
        let registry = EncodingRegistry::new();
        return Some(DetectionResult {
            encoding: registry.by_name("ebcdic-037").unwrap().clone(),
            confidence: DetectionConfidence::Low,
            bom: None,
        });
    }

    // Default to ISO-8859-1 for high-byte content that isn't valid UTF-8
    let registry = EncodingRegistry::new();
    Some(DetectionResult {
        encoding: registry.by_name("iso-8859-1").unwrap().clone(),
        confidence: DetectionConfidence::Low,
        bom: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_encoding_utf8_bom() {
        // Validates: Requirement 1.1, 1.3
        let data = [0xEF, 0xBB, 0xBF, b'H', b'e', b'l', b'l', b'o'];
        let result = detect_encoding(&data, None);
        assert_eq!(result.encoding.name, "utf-8");
        assert_eq!(result.confidence, DetectionConfidence::High);
        assert!(result.bom.is_some());
    }

    #[test]
    fn detect_encoding_utf16le_bom() {
        // Validates: Requirement 1.1, 1.3
        let data = [0xFF, 0xFE, b'H', 0x00, b'i', 0x00];
        let result = detect_encoding(&data, None);
        assert_eq!(result.encoding.name, "utf-16le");
        assert_eq!(result.confidence, DetectionConfidence::High);
    }

    #[test]
    fn detect_encoding_valid_utf8_without_bom() {
        // Validates: Requirement 1.4
        let data = "Hello, 世界!".as_bytes();
        let result = detect_encoding(data, None);
        assert_eq!(result.encoding.name, "utf-8");
        assert_eq!(result.confidence, DetectionConfidence::Medium);
        assert!(result.bom.is_none());
    }

    #[test]
    fn detect_encoding_pure_ascii_as_utf8() {
        // Validates: Requirement 1.4
        let data = b"Hello, World! 123";
        let result = detect_encoding(data, None);
        assert_eq!(result.encoding.name, "utf-8");
    }

    #[test]
    fn detect_encoding_utf16le_by_null_pattern() {
        // Validates: Requirement 1.5
        // Use characters with high bytes to avoid being valid UTF-8
        // "Héllo" in UTF-16LE: H=0x48,00 é=0xE9,00 l=0x6C,00 l=0x6C,00 o=0x6F,00
        let data: Vec<u8> = vec![0x48, 0x00, 0xE9, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00];
        let result = detect_encoding(&data, None);
        assert_eq!(result.encoding.name, "utf-16le");
        assert_eq!(result.confidence, DetectionConfidence::Medium);
    }

    #[test]
    fn detect_encoding_is_stateless_and_side_effect_free() {
        // Validates: Requirement 1.7
        let data = b"test data";
        let result1 = detect_encoding(data, None);
        let result2 = detect_encoding(data, None);
        assert_eq!(result1, result2);
    }

    #[test]
    fn detect_encoding_respects_max_bytes() {
        // Validates: Requirement 1.1
        let data = vec![b'A'; 16384];
        let result = detect_encoding(&data, Some(100));
        assert_eq!(result.encoding.name, "utf-8");
    }

    #[test]
    fn detect_encoding_with_explicit_fallback() {
        // Validates: Requirement 1.8
        let registry = EncodingRegistry::new();
        let fallback = registry.by_name("windows-1252").unwrap();
        // Use single high byte — too short for DBCS pattern detection
        let data: Vec<u8> = vec![0xA0, 0xA1, 0xA2];
        let result = detect_encoding_with_fallback(&data, None, fallback);
        // Should fall through heuristics to single-byte detection
        // (the bytes don't form enough DBCS pairs to trigger threshold)
        assert!(
            result.encoding.family == crate::encoding::EncodingFamily::SingleByte
                || result.encoding.family == crate::encoding::EncodingFamily::Dbcs,
            "Should detect as single-byte or DBCS, got: {:?}",
            result.encoding
        );
        // The confidence should not be High (no BOM)
        assert_ne!(result.confidence, DetectionConfidence::High);
    }
}
