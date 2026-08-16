//! Hex byte pattern parsing and matching.
//!
//! Addresses: Requirement 3

use crate::error::FindReplaceError;

/// Parse a hex digit string into raw bytes.
///
/// Validates that the string has an even number of hex digits and contains
/// only valid hex characters (0-9, A-F, a-f).
///
/// Addresses: Requirement 3 AC 1–3, AC 6
pub fn parse_hex_pattern(hex_str: &str) -> Result<Vec<u8>, FindReplaceError> {
    if hex_str.is_empty() {
        return Err(FindReplaceError::NoSearchTerm);
    }

    if !hex_str.len().is_multiple_of(2) {
        return Err(FindReplaceError::HexOddDigits);
    }

    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    let chars: Vec<char> = hex_str.chars().collect();

    for pair in chars.chunks(2) {
        let high = hex_digit_value(pair[0])?;
        let low = hex_digit_value(pair[1])?;
        bytes.push((high << 4) | low);
    }

    Ok(bytes)
}

/// Convert a hex character to its numeric value (0–15).
fn hex_digit_value(ch: char) -> Result<u8, FindReplaceError> {
    match ch {
        '0'..='9' => Ok(ch as u8 - b'0'),
        'A'..='F' => Ok(ch as u8 - b'A' + 10),
        'a'..='f' => Ok(ch as u8 - b'a' + 10),
        _ => Err(FindReplaceError::HexInvalidChar(ch)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_pattern_converts_valid_hex_to_bytes() {
        assert_eq!(parse_hex_pattern("4A5B").unwrap(), vec![0x4A, 0x5B]);
        assert_eq!(parse_hex_pattern("FF00").unwrap(), vec![0xFF, 0x00]);
        assert_eq!(parse_hex_pattern("0a1b2c").unwrap(), vec![0x0A, 0x1B, 0x2C]);
    }

    #[test]
    fn parse_hex_pattern_is_case_insensitive_for_hex_digits() {
        assert_eq!(
            parse_hex_pattern("4a5b").unwrap(),
            parse_hex_pattern("4A5B").unwrap()
        );
    }

    #[test]
    fn parse_hex_pattern_rejects_odd_length_string() {
        let err = parse_hex_pattern("4A5").unwrap_err();
        assert!(matches!(err, FindReplaceError::HexOddDigits));
    }

    #[test]
    fn parse_hex_pattern_rejects_non_hex_characters() {
        let err = parse_hex_pattern("4G5B").unwrap_err();
        assert!(matches!(err, FindReplaceError::HexInvalidChar('G')));
    }

    #[test]
    fn parse_hex_pattern_rejects_empty_string() {
        let err = parse_hex_pattern("").unwrap_err();
        assert!(matches!(err, FindReplaceError::NoSearchTerm));
    }

    #[test]
    fn parse_hex_pattern_handles_null_bytes() {
        assert_eq!(parse_hex_pattern("0000").unwrap(), vec![0x00, 0x00]);
    }
}
