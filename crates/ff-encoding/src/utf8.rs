//! UTF-8 validation, classification, and repair utilities.
//!
//! Provides RFC 3629 conformant validation, byte-length classification from
//! lead bytes, and repair of invalid sequences by replacement with U+FFFD.

/// Returns the expected UTF-8 sequence length from a lead byte.
///
/// Returns 1 for ASCII (0x00–0x7F), 2–4 for valid multi-byte leads,
/// and 1 for invalid lead bytes (treating them as single-byte replacement targets).
///
/// [Requirement 5.6]
pub fn utf8_byte_length_from_lead(byte: u8) -> usize {
    match byte {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => 1, // Invalid lead bytes: 0x80-0xBF (trail), 0xC0-0xC1 (overlong), 0xF5-0xFF
    }
}

/// Classify the first UTF-8 character in a byte slice.
///
/// Returns `(byte_length, is_valid)`. For invalid sequences the byte_length
/// indicates how many bytes were consumed (always at least 1).
///
/// [Requirement 5.2]
pub fn utf8_classify(bytes: &[u8]) -> (usize, bool) {
    if bytes.is_empty() {
        return (0, false);
    }

    let lead = bytes[0];

    // ASCII
    if lead <= 0x7F {
        return (1, true);
    }

    // Trail byte or invalid lead
    if !(0xC2..=0xF4).contains(&lead) {
        return (1, false);
    }

    let expected_len = utf8_byte_length_from_lead(lead);

    if bytes.len() < expected_len {
        return (1, false);
    }

    // Check trail bytes are in range 0x80..=0xBF
    for &b in bytes.iter().take(expected_len).skip(1) {
        if b & 0xC0 != 0x80 {
            return (1, false);
        }
    }

    // Validate ranges per RFC 3629
    match expected_len {
        2 => (2, true), // 0xC2..=0xDF already excludes overlongs
        3 => {
            let cp = ((lead as u32 & 0x0F) << 12)
                | ((bytes[1] as u32 & 0x3F) << 6)
                | (bytes[2] as u32 & 0x3F);
            // Reject surrogates U+D800..U+DFFF
            if (0xD800..=0xDFFF).contains(&cp) {
                return (1, false);
            }
            // Reject overlong: code point must be >= 0x800
            if cp < 0x0800 {
                return (1, false);
            }
            (3, true)
        }
        4 => {
            let cp = ((lead as u32 & 0x07) << 18)
                | ((bytes[1] as u32 & 0x3F) << 12)
                | ((bytes[2] as u32 & 0x3F) << 6)
                | (bytes[3] as u32 & 0x3F);
            // Reject > U+10FFFF
            if cp > 0x10_FFFF {
                return (1, false);
            }
            // Reject overlong: code point must be >= 0x10000
            if cp < 0x1_0000 {
                return (1, false);
            }
            (4, true)
        }
        _ => (1, false),
    }
}

/// Validate that a byte slice is valid UTF-8 per RFC 3629.
///
/// Rejects overlong encodings, surrogates (U+D800–U+DFFF), and code points above U+10FFFF.
///
/// [Requirement 5.1]
pub fn utf8_validate(bytes: &[u8]) -> bool {
    let mut pos = 0;
    while pos < bytes.len() {
        let (len, valid) = utf8_classify(&bytes[pos..]);
        if !valid {
            return false;
        }
        pos += len;
    }
    true
}

/// Replace invalid UTF-8 sequences with U+FFFD, preserving valid content.
///
/// [Requirement 5.3]
pub fn utf8_fix_invalid(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let (len, valid) = utf8_classify(&bytes[pos..]);
        if valid && len > 0 {
            // Safe because we validated this is valid UTF-8
            let s = std::str::from_utf8(&bytes[pos..pos + len]).unwrap_or("\u{FFFD}");
            result.push_str(s);
            pos += len;
        } else {
            result.push('\u{FFFD}');
            pos += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_byte_length_from_lead_ascii() {
        // Validates: Requirement 5.6
        assert_eq!(utf8_byte_length_from_lead(0x00), 1);
        assert_eq!(utf8_byte_length_from_lead(b'A'), 1);
        assert_eq!(utf8_byte_length_from_lead(0x7F), 1);
    }

    #[test]
    fn utf8_byte_length_from_lead_two_byte() {
        // Validates: Requirement 5.6
        assert_eq!(utf8_byte_length_from_lead(0xC2), 2);
        assert_eq!(utf8_byte_length_from_lead(0xDF), 2);
    }

    #[test]
    fn utf8_byte_length_from_lead_three_byte() {
        // Validates: Requirement 5.6
        assert_eq!(utf8_byte_length_from_lead(0xE0), 3);
        assert_eq!(utf8_byte_length_from_lead(0xEF), 3);
    }

    #[test]
    fn utf8_byte_length_from_lead_four_byte() {
        // Validates: Requirement 5.6
        assert_eq!(utf8_byte_length_from_lead(0xF0), 4);
        assert_eq!(utf8_byte_length_from_lead(0xF4), 4);
    }

    #[test]
    fn utf8_byte_length_from_lead_invalid_returns_one() {
        // Validates: Requirement 5.6
        // Trail bytes
        assert_eq!(utf8_byte_length_from_lead(0x80), 1);
        assert_eq!(utf8_byte_length_from_lead(0xBF), 1);
        // Invalid leads (overlong)
        assert_eq!(utf8_byte_length_from_lead(0xC0), 1);
        assert_eq!(utf8_byte_length_from_lead(0xC1), 1);
        // Above F4
        assert_eq!(utf8_byte_length_from_lead(0xF5), 1);
        assert_eq!(utf8_byte_length_from_lead(0xFF), 1);
    }

    #[test]
    fn utf8_classify_ascii() {
        // Validates: Requirement 5.2
        assert_eq!(utf8_classify(b"A"), (1, true));
        assert_eq!(utf8_classify(b"\0"), (1, true));
        assert_eq!(utf8_classify(&[0x7F]), (1, true));
    }

    #[test]
    fn utf8_classify_two_byte_valid() {
        // Validates: Requirement 5.2
        // U+00E9 'é' = C3 A9
        assert_eq!(utf8_classify(&[0xC3, 0xA9]), (2, true));
    }

    #[test]
    fn utf8_classify_three_byte_valid() {
        // Validates: Requirement 5.2
        // U+4E2D '中' = E4 B8 AD
        assert_eq!(utf8_classify(&[0xE4, 0xB8, 0xAD]), (3, true));
    }

    #[test]
    fn utf8_classify_four_byte_valid() {
        // Validates: Requirement 5.2
        // U+1F600 '😀' = F0 9F 98 80
        assert_eq!(utf8_classify(&[0xF0, 0x9F, 0x98, 0x80]), (4, true));
    }

    #[test]
    fn utf8_classify_overlong_two_byte() {
        // Validates: Requirement 5.5
        // C0 80 is overlong encoding of NUL
        assert_eq!(utf8_classify(&[0xC0, 0x80]), (1, false));
        assert_eq!(utf8_classify(&[0xC1, 0xBF]), (1, false));
    }

    #[test]
    fn utf8_classify_surrogate_three_byte() {
        // Validates: Requirement 5.4, 5.5
        // ED A0 80 encodes U+D800 (surrogate)
        assert_eq!(utf8_classify(&[0xED, 0xA0, 0x80]), (1, false));
    }

    #[test]
    fn utf8_classify_above_10ffff() {
        // Validates: Requirement 5.5
        // F4 90 80 80 encodes U+110000 (above max)
        assert_eq!(utf8_classify(&[0xF4, 0x90, 0x80, 0x80]), (1, false));
    }

    #[test]
    fn utf8_validate_valid_ascii() {
        // Validates: Requirement 5.1
        assert!(utf8_validate(b"Hello, world!"));
    }

    #[test]
    fn utf8_validate_valid_multibyte() {
        // Validates: Requirement 5.1
        assert!(utf8_validate("日本語テスト".as_bytes()));
        assert!(utf8_validate("café".as_bytes()));
    }

    #[test]
    fn utf8_validate_valid_four_byte() {
        // Validates: Requirement 5.1
        assert!(utf8_validate("😀🎉".as_bytes()));
    }

    #[test]
    fn utf8_validate_invalid_trail_byte_alone() {
        // Validates: Requirement 5.4
        assert!(!utf8_validate(&[0x80]));
        assert!(!utf8_validate(&[0xBF]));
    }

    #[test]
    fn utf8_validate_truncated_sequence() {
        // Validates: Requirement 5.1
        assert!(!utf8_validate(&[0xC3])); // Missing trail byte
        assert!(!utf8_validate(&[0xE4, 0xB8])); // Missing one trail byte
    }

    #[test]
    fn utf8_validate_overlong_rejected() {
        // Validates: Requirement 5.5
        assert!(!utf8_validate(&[0xC0, 0x80]));
    }

    #[test]
    fn utf8_validate_line_separators_accepted() {
        // Validates: Requirement 5.7
        // U+2028 Line Separator = E2 80 A8
        assert!(utf8_validate(&[0xE2, 0x80, 0xA8]));
        // U+2029 Paragraph Separator = E2 80 A9
        assert!(utf8_validate(&[0xE2, 0x80, 0xA9]));
        // U+0085 NEL = C2 85
        assert!(utf8_validate(&[0xC2, 0x85]));
    }

    #[test]
    fn utf8_fix_invalid_preserves_valid_content() {
        // Validates: Requirement 5.3
        let result = utf8_fix_invalid(b"Hello");
        assert_eq!(result, "Hello");
    }

    #[test]
    fn utf8_fix_invalid_replaces_invalid_with_replacement_char() {
        // Validates: Requirement 5.3
        let result = utf8_fix_invalid(&[0x48, 0xFF, 0x65]); // H, invalid, e
        assert_eq!(result, "H\u{FFFD}e");
    }

    #[test]
    fn utf8_fix_invalid_handles_mixed_valid_and_invalid() {
        // Validates: Requirement 5.3
        let input = [0xE4, 0xB8, 0xAD, 0xFF, 0xC3, 0xA9]; // 中, invalid, é
        let result = utf8_fix_invalid(&input);
        assert_eq!(result, "中\u{FFFD}é");
    }
}
