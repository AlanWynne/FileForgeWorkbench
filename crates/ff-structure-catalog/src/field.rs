//! Field definition and field type models.
//!
//! Provides [`FieldDefinition`] and [`FieldType`] which describe a single field
//! within a record structure — its byte position, length, data type, and interpretation.

use crate::error::{FieldValidationError, ValidationErrors};

/// Supported field data types.
///
/// Determines how raw bytes in a record are decoded and displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum FieldType {
    /// Character data (UTF-8 or EBCDIC code page).
    #[default]
    Alphanumeric,
    /// Unsigned integer stored as displayable digit characters (zoned decimal).
    Numeric,
    /// IBM COMP-3 packed BCD encoding.
    PackedDecimal,
    /// Raw binary bytes displayed as hex string.
    Binary,
    /// Hex dump with optional ASCII sidebar.
    Hex,
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alphanumeric => write!(f, "alphanumeric"),
            Self::Numeric => write!(f, "numeric"),
            Self::PackedDecimal => write!(f, "packed-decimal"),
            Self::Binary => write!(f, "binary"),
            Self::Hex => write!(f, "hex"),
        }
    }
}

impl std::str::FromStr for FieldType {
    type Err = FieldValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alphanumeric" => Ok(Self::Alphanumeric),
            "numeric" => Ok(Self::Numeric),
            "packed-decimal" => Ok(Self::PackedDecimal),
            "binary" => Ok(Self::Binary),
            "hex" => Ok(Self::Hex),
            other => Err(FieldValidationError::InvalidFieldType {
                value: other.to_string(),
            }),
        }
    }
}

/// A single field within a record structure.
///
/// Specifies the byte-level location, length, and interpretation
/// of one logical field in a flat-file record.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldDefinition {
    /// Field name (must be non-empty).
    pub name: String,
    /// Byte offset from start of record (0-based).
    pub offset: u32,
    /// Field length in bytes (must be >= 1).
    pub length: u32,
    /// Data type determining how bytes are interpreted.
    #[serde(default)]
    pub field_type: FieldType,
    /// Number of implied decimal positions (0 = integer).
    #[serde(default)]
    pub decimals: u8,
    /// Identifier values for record-type matching.
    #[serde(default)]
    pub identifiers: Vec<String>,
    /// Filter expressions for this field.
    #[serde(default)]
    pub filters: Vec<String>,
}

impl FieldDefinition {
    /// Create a new field definition with required parameters and defaults.
    pub fn new(name: impl Into<String>, offset: u32, length: u32, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            offset,
            length,
            field_type,
            decimals: 0,
            identifiers: Vec::new(),
            filters: Vec::new(),
        }
    }

    /// Validate this field definition against all constraints.
    ///
    /// Returns `Ok(())` if all constraints hold:
    /// - name is non-empty
    /// - length >= 1
    /// - field_type is a valid enum variant (always true for typed enum)
    /// - decimals >= 0 (always true for u8)
    ///
    /// Returns `Err(ValidationErrors)` with all violations found.
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push(FieldValidationError::EmptyName);
        }

        if self.length == 0 {
            errors.push(FieldValidationError::ZeroLength);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors { errors })
        }
    }
}

/// Decode a packed-decimal (COMP-3) byte slice into a signed decimal string.
///
/// In COMP-3 encoding, each byte holds two BCD nibbles, except the low nibble
/// of the final byte which holds the sign: C = positive, D = negative, F = unsigned.
///
/// # Errors
///
/// Returns `Err` with the byte index if any digit nibble is not 0–9,
/// or if the sign nibble is not C, D, or F.
pub fn decode_packed_decimal(bytes: &[u8], decimals: u8) -> Result<String, u32> {
    if bytes.is_empty() {
        return Ok("0".to_string());
    }

    let mut digits = String::new();
    let last_idx = bytes.len() - 1;

    // Process all bytes except the last one (full digit pairs)
    for (i, &byte) in bytes.iter().enumerate() {
        let high = (byte >> 4) & 0x0F;
        let low = byte & 0x0F;

        if i < last_idx {
            // Both nibbles are digits
            if high > 9 {
                return Err(i as u32);
            }
            if low > 9 {
                return Err(i as u32);
            }
            digits.push(char::from(b'0' + high));
            digits.push(char::from(b'0' + low));
        } else {
            // Last byte: high nibble is digit, low nibble is sign
            if high > 9 {
                return Err(i as u32);
            }
            digits.push(char::from(b'0' + high));

            // Validate sign nibble
            if !matches!(low, 0x0C | 0x0D | 0x0F) {
                return Err(i as u32);
            }
        }
    }

    // Determine sign
    let sign_nibble = bytes[last_idx] & 0x0F;
    let is_negative = sign_nibble == 0x0D;

    // Remove leading zeros but keep at least one digit
    let trimmed = digits.trim_start_matches('0');
    let trimmed = if trimmed.is_empty() { "0" } else { trimmed };

    // Insert decimal point if decimals > 0
    let result = if decimals > 0 {
        let dec = decimals as usize;
        if trimmed.len() <= dec {
            // Need leading zeros after decimal point
            let zeros_needed = dec - trimmed.len() + 1;
            let padded = format!("{}{}", "0".repeat(zeros_needed), trimmed);
            let insert_pos = padded.len() - dec;
            format!("{}.{}", &padded[..insert_pos], &padded[insert_pos..])
        } else {
            let insert_pos = trimmed.len() - dec;
            format!("{}.{}", &trimmed[..insert_pos], &trimmed[insert_pos..])
        }
    } else {
        trimmed.to_string()
    };

    if is_negative && result != "0" && result != "0.00" {
        Ok(format!("-{result}"))
    } else {
        Ok(result)
    }
}

/// Format a numeric (zoned decimal) value with implied decimal places.
///
/// Takes a string of digit characters and inserts a decimal point
/// N positions from the right.
///
/// # Examples
///
/// ```
/// use ff_structure_catalog::field::format_numeric_decimal;
/// assert_eq!(format_numeric_decimal("12345", 2), "123.45");
/// assert_eq!(format_numeric_decimal("5", 2), "0.05");
/// ```
pub fn format_numeric_decimal(digits: &str, decimals: u8) -> String {
    if decimals == 0 {
        return digits.to_string();
    }

    let dec = decimals as usize;
    let clean: String = digits.chars().filter(|c| c.is_ascii_digit()).collect();

    if clean.is_empty() {
        let zeros = "0".repeat(dec);
        return format!("0.{zeros}");
    }

    if clean.len() <= dec {
        let zeros_needed = dec - clean.len();
        format!("0.{}{}", "0".repeat(zeros_needed), clean)
    } else {
        let insert_pos = clean.len() - dec;
        format!("{}.{}", &clean[..insert_pos], &clean[insert_pos..])
    }
}

/// Validate packed-decimal bytes for invalid nibble values.
///
/// Returns `Ok(())` if all digit nibbles are 0–9 and the sign nibble is C, D, or F.
/// Returns `Err(byte_index)` for the first invalid byte found.
pub fn validate_packed_decimal(bytes: &[u8]) -> Result<(), u32> {
    if bytes.is_empty() {
        return Ok(());
    }

    let last_idx = bytes.len() - 1;

    for (i, &byte) in bytes.iter().enumerate() {
        let high = (byte >> 4) & 0x0F;
        let low = byte & 0x0F;

        if i < last_idx {
            if high > 9 || low > 9 {
                return Err(i as u32);
            }
        } else {
            // Last byte: high is digit, low is sign
            if high > 9 {
                return Err(i as u32);
            }
            if !matches!(low, 0x0C | 0x0D | 0x0F) {
                return Err(i as u32);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.2 — FieldDefinition structure
    #[test]
    fn field_definition_new_sets_defaults() {
        let field = FieldDefinition::new("CUST_NAME", 0, 30, FieldType::Alphanumeric);
        assert_eq!(field.name, "CUST_NAME");
        assert_eq!(field.offset, 0);
        assert_eq!(field.length, 30);
        assert_eq!(field.field_type, FieldType::Alphanumeric);
        assert_eq!(field.decimals, 0);
        assert!(field.identifiers.is_empty());
        assert!(field.filters.is_empty());
    }

    // Validates: Requirement 5.9 — field validation: name non-empty
    #[test]
    fn validate_rejects_empty_name() {
        let field = FieldDefinition::new("", 0, 10, FieldType::Alphanumeric);
        let err = field.validate().unwrap_err();
        assert!(err.errors.contains(&FieldValidationError::EmptyName));
    }

    // Validates: Requirement 5.9 — field validation: length >= 1
    #[test]
    fn validate_rejects_zero_length() {
        let field = FieldDefinition::new("FIELD", 0, 0, FieldType::Numeric);
        let err = field.validate().unwrap_err();
        assert!(err.errors.contains(&FieldValidationError::ZeroLength));
    }

    // Validates: Requirement 5.9 — field validation: all valid
    #[test]
    fn validate_accepts_valid_field() {
        let field = FieldDefinition::new("AMOUNT", 10, 8, FieldType::PackedDecimal);
        assert!(field.validate().is_ok());
    }

    // Validates: Requirement 5.9 — multiple validation errors reported
    #[test]
    fn validate_reports_multiple_errors() {
        let field = FieldDefinition::new("", 0, 0, FieldType::Alphanumeric);
        let err = field.validate().unwrap_err();
        assert_eq!(err.errors.len(), 2);
    }

    // Validates: Requirement 6.3 — packed-decimal decode positive
    #[test]
    fn decode_packed_decimal_positive_no_decimals() {
        // 0x12345C = +12345
        let bytes = [0x12, 0x34, 0x5C];
        let result = decode_packed_decimal(&bytes, 0).unwrap();
        assert_eq!(result, "12345");
    }

    // Validates: Requirement 6.6 — packed-decimal with decimal places
    #[test]
    fn decode_packed_decimal_with_decimals() {
        // 0x12345C with decimals=2 → "123.45"
        let bytes = [0x12, 0x34, 0x5C];
        let result = decode_packed_decimal(&bytes, 2).unwrap();
        assert_eq!(result, "123.45");
    }

    // Validates: Requirement 6.3 — packed-decimal decode negative
    #[test]
    fn decode_packed_decimal_negative() {
        // 0x12345D = -12345
        let bytes = [0x12, 0x34, 0x5D];
        let result = decode_packed_decimal(&bytes, 0).unwrap();
        assert_eq!(result, "-12345");
    }

    // Validates: Requirement 6.3 — packed-decimal unsigned (F sign)
    #[test]
    fn decode_packed_decimal_unsigned() {
        // 2 bytes: high nibble of byte 0 = 9, low nibble = 8, high nibble of byte 1 = 7, sign = F
        let bytes = [0x98, 0x7F];
        let result = decode_packed_decimal(&bytes, 0).unwrap();
        assert_eq!(result, "987");
    }

    // Validates: Requirement 6.8 — invalid nibble detection
    #[test]
    fn decode_packed_decimal_invalid_digit_nibble() {
        // 0xA0 has high nibble = 10 (invalid digit)
        let bytes = [0xA0, 0x1C];
        let result = decode_packed_decimal(&bytes, 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), 0);
    }

    // Validates: Requirement 6.8 — invalid sign nibble
    #[test]
    fn decode_packed_decimal_invalid_sign_nibble() {
        // Sign nibble = 0xA (not C, D, or F)
        let bytes = [0x12, 0x3A];
        let result = decode_packed_decimal(&bytes, 0);
        assert!(result.is_err());
    }

    // Validates: Requirement 6.7 — numeric implied decimal
    #[test]
    fn format_numeric_decimal_basic() {
        assert_eq!(format_numeric_decimal("12345", 2), "123.45");
    }

    // Validates: Requirement 6.7 — numeric implied decimal with short value
    #[test]
    fn format_numeric_decimal_short_value() {
        assert_eq!(format_numeric_decimal("5", 2), "0.05");
    }

    // Validates: Requirement 6.7 — numeric no decimals
    #[test]
    fn format_numeric_decimal_zero_decimals() {
        assert_eq!(format_numeric_decimal("12345", 0), "12345");
    }

    // Validates: Requirement 6.8 — packed-decimal validation
    #[test]
    fn validate_packed_decimal_valid_bytes() {
        let bytes = [0x12, 0x34, 0x5C];
        assert!(validate_packed_decimal(&bytes).is_ok());
    }

    // Validates: Requirement 6.8 — packed-decimal validation invalid
    #[test]
    fn validate_packed_decimal_invalid_digit() {
        let bytes = [0xAB, 0x1C];
        let result = validate_packed_decimal(&bytes);
        assert!(result.is_err());
    }

    // Validates: Requirement 6 — FieldType display
    #[test]
    fn field_type_display() {
        assert_eq!(FieldType::Alphanumeric.to_string(), "alphanumeric");
        assert_eq!(FieldType::PackedDecimal.to_string(), "packed-decimal");
    }

    // Validates: Requirement 6 — FieldType from_str
    #[test]
    fn field_type_from_str_valid() {
        assert_eq!(
            "alphanumeric".parse::<FieldType>().unwrap(),
            FieldType::Alphanumeric
        );
        assert_eq!(
            "packed-decimal".parse::<FieldType>().unwrap(),
            FieldType::PackedDecimal
        );
        assert_eq!("numeric".parse::<FieldType>().unwrap(), FieldType::Numeric);
        assert_eq!("binary".parse::<FieldType>().unwrap(), FieldType::Binary);
        assert_eq!("hex".parse::<FieldType>().unwrap(), FieldType::Hex);
    }

    // Validates: Requirement 6 — FieldType from_str invalid
    #[test]
    fn field_type_from_str_invalid() {
        let result = "invalid_type".parse::<FieldType>();
        assert!(result.is_err());
    }

    // Validates: Requirement 6 — FieldType default
    #[test]
    fn field_type_default_is_alphanumeric() {
        assert_eq!(FieldType::default(), FieldType::Alphanumeric);
    }
}
