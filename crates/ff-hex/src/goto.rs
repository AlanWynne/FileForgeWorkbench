//! Goto offset command handling.
//!
//! Parses offset arguments in various formats (hex, decimal) and
//! validates bounds before positioning the cursor.

use crate::error::HexError;

/// Parsed offset value from a GOTO command argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedOffset {
    /// The byte offset value.
    pub value: u64,
}

/// Handles GOTO offset command parsing and navigation.
#[derive(Debug)]
pub struct HexGotoHandler;

impl HexGotoHandler {
    /// Parse an offset string in supported formats:
    ///
    /// - `X'1A4F'` — ISPF hex literal
    /// - `0x1A4F` — C-style hex prefix
    /// - `6735` — decimal (no prefix)
    pub fn parse_offset(input: &str) -> Result<ParsedOffset, HexError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(HexError::InvalidOffsetFormat(input.to_string()));
        }

        // X'...' format (ISPF style)
        if let Some(hex_str) = trimmed
            .strip_prefix("X'")
            .or_else(|| trimmed.strip_prefix("x'"))
        {
            let hex_str = hex_str
                .strip_suffix('\'')
                .ok_or_else(|| HexError::InvalidOffsetFormat(input.to_string()))?;

            let value = u64::from_str_radix(hex_str, 16)
                .map_err(|_| HexError::InvalidOffsetFormat(input.to_string()))?;

            return Ok(ParsedOffset { value });
        }

        // 0x... format (C style)
        if let Some(hex_str) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            let value = u64::from_str_radix(hex_str, 16)
                .map_err(|_| HexError::InvalidOffsetFormat(input.to_string()))?;

            return Ok(ParsedOffset { value });
        }

        // Decimal format
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| HexError::InvalidOffsetFormat(input.to_string()))?;

        Ok(ParsedOffset { value })
    }

    /// Validate that an offset is within document bounds.
    pub fn validate_bounds(offset: u64, document_length: u64) -> Result<(), HexError> {
        if offset >= document_length {
            Err(HexError::OffsetOutOfRange {
                offset,
                size: document_length,
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 12 AC 5
    #[test]
    fn parse_offset_ispf_hex_format() {
        let parsed = HexGotoHandler::parse_offset("X'1A4F'").unwrap();
        assert_eq!(parsed.value, 0x1A4F);
    }

    // Validates: Requirement 12 AC 5
    #[test]
    fn parse_offset_ispf_hex_lowercase() {
        let parsed = HexGotoHandler::parse_offset("x'ff00'").unwrap();
        assert_eq!(parsed.value, 0xFF00);
    }

    // Validates: Requirement 12 AC 5
    #[test]
    fn parse_offset_c_style_hex() {
        let parsed = HexGotoHandler::parse_offset("0x1A4F").unwrap();
        assert_eq!(parsed.value, 0x1A4F);
    }

    // Validates: Requirement 12 AC 5
    #[test]
    fn parse_offset_c_style_hex_uppercase_prefix() {
        let parsed = HexGotoHandler::parse_offset("0XDEAD").unwrap();
        assert_eq!(parsed.value, 0xDEAD);
    }

    // Validates: Requirement 12 AC 5
    #[test]
    fn parse_offset_decimal_format() {
        let parsed = HexGotoHandler::parse_offset("6735").unwrap();
        assert_eq!(parsed.value, 6735);
    }

    // Validates: Requirement 12 AC 5
    #[test]
    fn parse_offset_decimal_zero() {
        let parsed = HexGotoHandler::parse_offset("0").unwrap();
        assert_eq!(parsed.value, 0);
    }

    // Validates: Requirement 12 AC 5
    #[test]
    fn parse_offset_rejects_invalid_format() {
        assert!(HexGotoHandler::parse_offset("").is_err());
        assert!(HexGotoHandler::parse_offset("abc").is_err());
        assert!(HexGotoHandler::parse_offset("X'GG'").is_err());
        assert!(HexGotoHandler::parse_offset("0xZZZZ").is_err());
        assert!(HexGotoHandler::parse_offset("X'123").is_err()); // missing closing quote
    }

    // Validates: Requirement 12 AC 5
    #[test]
    fn parse_offset_handles_whitespace() {
        let parsed = HexGotoHandler::parse_offset("  0x10  ").unwrap();
        assert_eq!(parsed.value, 0x10);
    }

    // Validates: Requirement 12 AC 4
    #[test]
    fn validate_bounds_accepts_valid_offset() {
        assert!(HexGotoHandler::validate_bounds(0, 100).is_ok());
        assert!(HexGotoHandler::validate_bounds(99, 100).is_ok());
    }

    // Validates: Requirement 12 AC 4
    #[test]
    fn validate_bounds_rejects_out_of_range() {
        let err = HexGotoHandler::validate_bounds(100, 100).unwrap_err();
        assert_eq!(
            err,
            HexError::OffsetOutOfRange {
                offset: 100,
                size: 100
            }
        );
    }
}
