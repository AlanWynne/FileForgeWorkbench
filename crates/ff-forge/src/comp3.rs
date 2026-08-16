//! COMP-3 packed decimal codec.
//!
//! Implements IBM packed decimal (COMP-3) decode, format, and encode operations.
//! Each byte holds two BCD digit nibbles (high nibble first). The low nibble
//! of the final byte is the sign (C=positive, D=negative, F=unsigned).

use crate::error::FileForgeError;

/// The sign nibble of a COMP-3 field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comp3Sign {
    /// Positive (sign nibble 0xC).
    Positive,
    /// Negative (sign nibble 0xD).
    Negative,
    /// Unsigned (sign nibble 0xF).
    Unsigned,
}

/// A decoded COMP-3 packed decimal field value.
///
/// Stores the integer mantissa and implied decimal places separately.
/// The display value is `mantissa / 10^decimals`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comp3Value {
    /// The absolute integer mantissa value.
    pub mantissa: i64,
    /// Number of implied decimal places.
    pub decimals: u8,
    /// Sign from the source bytes.
    pub sign: Comp3Sign,
}

/// Decodes packed decimal bytes into a `Comp3Value`.
///
/// # Errors
///
/// Returns `FileForgeError::InvalidComp3` if any digit nibble is
/// outside 0x0–0x9 or the sign nibble is invalid.
pub fn decode_comp3(bytes: &[u8]) -> Result<Comp3Value, FileForgeError> {
    if bytes.is_empty() {
        return Err(FileForgeError::InvalidComp3 {
            field_name: String::new(),
            offset: 0,
        });
    }

    let mut mantissa: i64 = 0;

    // Process all bytes except the last one: both nibbles are digits
    for (i, &byte) in bytes.iter().enumerate() {
        let high = (byte >> 4) & 0x0F;
        let low = byte & 0x0F;

        if i < bytes.len() - 1 {
            // Both nibbles are digits
            if high > 9 {
                return Err(FileForgeError::InvalidComp3 {
                    field_name: String::new(),
                    offset: i,
                });
            }
            if low > 9 {
                return Err(FileForgeError::InvalidComp3 {
                    field_name: String::new(),
                    offset: i,
                });
            }
            mantissa = mantissa * 100 + i64::from(high) * 10 + i64::from(low);
        } else {
            // Last byte: high nibble is digit, low nibble is sign
            if high > 9 {
                return Err(FileForgeError::InvalidComp3 {
                    field_name: String::new(),
                    offset: i,
                });
            }
            mantissa = mantissa * 10 + i64::from(high);

            let sign = match low {
                0x0C => Comp3Sign::Positive,
                0x0D => Comp3Sign::Negative,
                0x0F => Comp3Sign::Unsigned,
                // Also accept A, E as positive (common variants)
                0x0A | 0x0E => Comp3Sign::Positive,
                0x0B => Comp3Sign::Negative,
                _ => {
                    return Err(FileForgeError::InvalidComp3 {
                        field_name: String::new(),
                        offset: i,
                    });
                }
            };

            if sign == Comp3Sign::Negative {
                mantissa = -mantissa;
            }

            return Ok(Comp3Value {
                mantissa,
                decimals: 0,
                sign,
            });
        }
    }

    // Should not reach here given the loop structure
    Err(FileForgeError::InvalidComp3 {
        field_name: String::new(),
        offset: 0,
    })
}

/// Formats a `Comp3Value` as a human-readable decimal string.
///
/// Applies the implied decimal point: mantissa / 10^decimals.
/// The decimal separator is always '.' regardless of locale.
pub fn format_comp3(value: &Comp3Value) -> String {
    let abs_mantissa = value.mantissa.unsigned_abs();
    let is_negative = value.mantissa < 0;

    if value.decimals == 0 {
        if is_negative {
            format!("-{abs_mantissa}")
        } else {
            format!("{abs_mantissa}")
        }
    } else {
        let divisor = 10u64.pow(u32::from(value.decimals));
        let integer_part = abs_mantissa / divisor;
        let fractional_part = abs_mantissa % divisor;

        if is_negative {
            format!(
                "-{integer_part}.{fractional_part:0>width$}",
                width = value.decimals as usize
            )
        } else {
            format!(
                "{integer_part}.{fractional_part:0>width$}",
                width = value.decimals as usize
            )
        }
    }
}

/// Encodes a decimal string value into COMP-3 packed bytes.
///
/// # Errors
///
/// Returns `FileForgeError::FieldOverflow` if the value requires more
/// digit pairs than `max_length` bytes can hold.
/// Returns `FileForgeError::FieldValidation` if the input is not valid numeric.
pub fn encode_comp3(
    value: &str,
    decimals: u8,
    max_length: usize,
) -> Result<Vec<u8>, FileForgeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(FileForgeError::FieldValidation {
            field_name: String::new(),
            reason: "empty value for COMP-3 field".to_string(),
        });
    }

    // Determine sign
    let (is_negative, numeric_str) = if let Some(stripped) = trimmed.strip_prefix('-') {
        (true, stripped)
    } else if let Some(stripped) = trimmed.strip_prefix('+') {
        (false, stripped)
    } else {
        (false, trimmed)
    };

    // Remove decimal point and compute mantissa
    let mantissa_str = if decimals > 0 {
        // If there's a decimal point, handle it
        if let Some(dot_pos) = numeric_str.find('.') {
            let integer_part = &numeric_str[..dot_pos];
            let frac_part = &numeric_str[dot_pos + 1..];
            // Pad or truncate fractional part to `decimals` digits
            let padded_frac = if frac_part.len() >= decimals as usize {
                frac_part[..decimals as usize].to_string()
            } else {
                format!("{frac_part:0<width$}", width = decimals as usize)
            };
            format!("{integer_part}{padded_frac}")
        } else {
            // No decimal point — multiply by 10^decimals
            format!(
                "{numeric_str}{zeros}",
                zeros = "0".repeat(decimals as usize)
            )
        }
    } else {
        // No decimal places — value should be integer
        numeric_str.replace('.', "")
    };

    // Validate all characters are digits
    if !mantissa_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(FileForgeError::FieldValidation {
            field_name: String::new(),
            reason: format!("invalid numeric value for COMP-3: '{value}'"),
        });
    }

    // Remove leading zeros but keep at least one digit
    let digits: Vec<u8> = mantissa_str.chars().map(|c| c as u8 - b'0').collect();

    let digits = if digits.iter().all(|&d| d == 0) {
        vec![0]
    } else {
        let first_nonzero = digits.iter().position(|&d| d != 0).unwrap_or(0);
        digits[first_nonzero..].to_vec()
    };

    // COMP-3: each byte holds 2 digits, except the last byte which holds
    // 1 digit + sign nibble. Total digits that fit = (max_length * 2) - 1
    let max_digits = max_length * 2 - 1;
    if digits.len() > max_digits {
        return Err(FileForgeError::FieldOverflow {
            field_name: String::new(),
            max_length,
            actual_length: (digits.len() + 2) / 2, // approximate
        });
    }

    // Pad digits with leading zeros to fill max_digits positions
    let mut padded_digits = vec![0u8; max_digits - digits.len()];
    padded_digits.extend_from_slice(&digits);

    // Pack into bytes: pairs of digits, last byte is digit + sign
    let sign_nibble: u8 = if is_negative { 0x0D } else { 0x0C };

    let mut result = Vec::with_capacity(max_length);
    let mut i = 0;
    while i < padded_digits.len() - 1 {
        let byte = (padded_digits[i] << 4) | padded_digits[i + 1];
        result.push(byte);
        i += 2;
    }
    // Last byte: final digit in high nibble, sign in low nibble
    let last_byte = (padded_digits[padded_digits.len() - 1] << 4) | sign_nibble;
    result.push(last_byte);

    debug_assert_eq!(result.len(), max_length);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 5.2
    #[test]
    fn decode_comp3_positive() {
        // X'1234567C' → +1234567
        let bytes = [0x12, 0x34, 0x56, 0x7C];
        let value = decode_comp3(&bytes).unwrap();
        assert_eq!(value.mantissa, 1234567);
        assert_eq!(value.sign, Comp3Sign::Positive);
    }

    #[test]
    fn decode_comp3_negative() {
        // X'1234567D' → -1234567
        let bytes = [0x12, 0x34, 0x56, 0x7D];
        let value = decode_comp3(&bytes).unwrap();
        assert_eq!(value.mantissa, -1234567);
        assert_eq!(value.sign, Comp3Sign::Negative);
    }

    #[test]
    fn decode_comp3_unsigned() {
        // X'1234567F' → 1234567 (unsigned)
        let bytes = [0x12, 0x34, 0x56, 0x7F];
        let value = decode_comp3(&bytes).unwrap();
        assert_eq!(value.mantissa, 1234567);
        assert_eq!(value.sign, Comp3Sign::Unsigned);
    }

    // Validates: Requirement 5.3
    #[test]
    fn format_comp3_no_decimals() {
        let value = Comp3Value {
            mantissa: 1234567,
            decimals: 0,
            sign: Comp3Sign::Positive,
        };
        assert_eq!(format_comp3(&value), "1234567");
    }

    // Validates: Requirement 5.4
    #[test]
    fn format_comp3_with_decimals() {
        let value = Comp3Value {
            mantissa: 123456,
            decimals: 2,
            sign: Comp3Sign::Positive,
        };
        assert_eq!(format_comp3(&value), "1234.56");
    }

    #[test]
    fn format_comp3_negative_with_decimals() {
        let value = Comp3Value {
            mantissa: -123456,
            decimals: 2,
            sign: Comp3Sign::Negative,
        };
        assert_eq!(format_comp3(&value), "-1234.56");
    }

    #[test]
    fn format_comp3_zero() {
        let value = Comp3Value {
            mantissa: 0,
            decimals: 0,
            sign: Comp3Sign::Positive,
        };
        assert_eq!(format_comp3(&value), "0");
    }

    // Validates: Requirement 5.5
    #[test]
    fn encode_comp3_positive() {
        let encoded = encode_comp3("1234567", 0, 4).unwrap();
        assert_eq!(encoded, vec![0x12, 0x34, 0x56, 0x7C]);
        // Verify roundtrip
        let decoded = decode_comp3(&encoded).unwrap();
        assert_eq!(decoded.mantissa, 1234567);
        assert_eq!(decoded.sign, Comp3Sign::Positive);
    }

    #[test]
    fn encode_comp3_negative() {
        let encoded = encode_comp3("-1234567", 0, 4).unwrap();
        assert_eq!(encoded, vec![0x12, 0x34, 0x56, 0x7D]);
        let decoded = decode_comp3(&encoded).unwrap();
        assert_eq!(decoded.mantissa, -1234567);
    }

    #[test]
    fn encode_comp3_with_decimals() {
        // "1234.56" with decimals=2 → mantissa 123456
        let encoded = encode_comp3("1234.56", 2, 4).unwrap();
        let decoded = decode_comp3(&encoded).unwrap();
        assert_eq!(decoded.mantissa, 123456);
    }

    // Validates: Requirement 5.6
    #[test]
    fn encode_comp3_overflow_rejected() {
        // 1 byte can hold 1 digit + sign = max value 9
        // 10 digits won't fit in 2 bytes (max 3 digits in 2 bytes)
        let result = encode_comp3("12345678901234567890", 0, 2);
        assert!(result.is_err());
        assert!(matches!(result, Err(FileForgeError::FieldOverflow { .. })));
    }

    // Validates: Requirement 5.7
    #[test]
    fn decode_comp3_invalid_nibble_returns_error() {
        // 0xAB has A in high nibble (>9) for a non-last byte
        let bytes = [0xAB, 0x12, 0x3C];
        let result = decode_comp3(&bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(FileForgeError::InvalidComp3 { .. })));
    }

    #[test]
    fn decode_comp3_invalid_sign_nibble() {
        // Sign nibble 0x7 is invalid
        let bytes = [0x12, 0x37];
        let result = decode_comp3(&bytes);
        assert!(result.is_err());
    }

    // Validates: Requirement 5.10
    #[test]
    fn format_comp3_uses_period_as_decimal_separator() {
        let value = Comp3Value {
            mantissa: 12345,
            decimals: 3,
            sign: Comp3Sign::Positive,
        };
        let formatted = format_comp3(&value);
        assert!(formatted.contains('.'));
        assert!(!formatted.contains(','));
        assert_eq!(formatted, "12.345");
    }

    #[test]
    fn decode_comp3_empty_bytes_returns_error() {
        let result = decode_comp3(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn decode_comp3_single_byte() {
        // X'5C' → digit 5, sign positive
        let bytes = [0x5C];
        let value = decode_comp3(&bytes).unwrap();
        assert_eq!(value.mantissa, 5);
        assert_eq!(value.sign, Comp3Sign::Positive);
    }

    #[test]
    fn encode_decode_roundtrip_zero() {
        let encoded = encode_comp3("0", 0, 2).unwrap();
        let decoded = decode_comp3(&encoded).unwrap();
        assert_eq!(decoded.mantissa, 0);
    }
}
