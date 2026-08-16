//! Field display and value conversion.
//!
//! Implements the rendering pipeline from raw bytes to display-ready values
//! across three modes: Raw, Structured, and Transformed.

use crate::comp3;
use crate::field_def::{DataType, FieldDefinition};

/// Display mode for grid cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Show raw byte content (hex for binary, decoded text for string).
    Raw,
    /// Show parsed field values (strings decoded, numbers formatted).
    Structured,
    /// Show values after decimal/COMP-3 conversion with implied decimals applied.
    Transformed,
}

/// A rendered field value ready for display in a grid cell.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Successfully rendered text.
    Text(String),
    /// Raw hex representation (for errors or raw mode).
    Hex(String),
    /// Validation error — show raw hex with error indicator.
    Error {
        /// Hex representation of the raw bytes.
        hex: String,
        /// Error description.
        message: String,
    },
}

impl FieldValue {
    /// Returns the display string for this value.
    pub fn display_text(&self) -> &str {
        match self {
            Self::Text(s) => s,
            Self::Hex(s) => s,
            Self::Error { hex, .. } => hex,
        }
    }

    /// Returns true if this value represents an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

/// Renders a field value from raw record bytes according to the display mode.
pub fn render_field(record_bytes: &[u8], field: &FieldDefinition, mode: DisplayMode) -> FieldValue {
    // Extract field bytes
    if record_bytes.len() < field.offset + field.length {
        return FieldValue::Error {
            hex: String::new(),
            message: "record too short for field".to_string(),
        };
    }
    let field_bytes = &record_bytes[field.offset..field.offset + field.length];

    match mode {
        DisplayMode::Raw => render_raw(field_bytes),
        DisplayMode::Structured => render_structured(field_bytes, field),
        DisplayMode::Transformed => render_transformed(field_bytes, field),
    }
}

/// Renders field bytes in Raw mode (hex for non-printable, text for printable).
fn render_raw(bytes: &[u8]) -> FieldValue {
    if bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
        FieldValue::Text(String::from_utf8_lossy(bytes).into_owned())
    } else {
        let hex: String = bytes
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");
        FieldValue::Hex(hex)
    }
}

/// Renders field bytes in Structured mode (parsed per data type).
fn render_structured(bytes: &[u8], field: &FieldDefinition) -> FieldValue {
    match field.data_type {
        DataType::Str | DataType::Bool => {
            let text = String::from_utf8_lossy(bytes).trim_end().to_string();
            FieldValue::Text(text)
        }
        DataType::Int => {
            let text = String::from_utf8_lossy(bytes).trim().to_string();
            if text.is_empty() || text.chars().all(|c| c == ' ') {
                FieldValue::Text("0".to_string())
            } else {
                FieldValue::Text(text)
            }
        }
        DataType::Float => {
            let text = String::from_utf8_lossy(bytes).trim().to_string();
            FieldValue::Text(text)
        }
        DataType::Comp3 => match comp3::decode_comp3(bytes) {
            Ok(value) => FieldValue::Text(comp3::format_comp3(&value)),
            Err(_) => {
                let hex: String = bytes
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join("");
                FieldValue::Error {
                    hex: format!("X'{hex}'"),
                    message: "invalid COMP-3 data".to_string(),
                }
            }
        },
    }
}

/// Renders field bytes in Transformed mode (with implied decimals applied).
fn render_transformed(bytes: &[u8], field: &FieldDefinition) -> FieldValue {
    match field.data_type {
        DataType::Comp3 => match comp3::decode_comp3(bytes) {
            Ok(mut value) => {
                value.decimals = field.decimals;
                FieldValue::Text(comp3::format_comp3(&value))
            }
            Err(_) => {
                let hex: String = bytes
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join("");
                FieldValue::Error {
                    hex: format!("X'{hex}'"),
                    message: "invalid COMP-3 data".to_string(),
                }
            }
        },
        DataType::Int | DataType::Float => {
            let text = String::from_utf8_lossy(bytes).trim().to_string();
            if field.decimals > 0 {
                // Apply implied decimal
                if let Ok(raw_value) = text.parse::<i64>() {
                    let divisor = 10i64.pow(u32::from(field.decimals));
                    let integer_part = raw_value / divisor;
                    let frac_part = (raw_value % divisor).unsigned_abs();
                    if raw_value < 0 {
                        FieldValue::Text(format!(
                            "-{}.{:0>width$}",
                            integer_part.unsigned_abs(),
                            frac_part,
                            width = field.decimals as usize
                        ))
                    } else {
                        FieldValue::Text(format!(
                            "{integer_part}.{frac_part:0>width$}",
                            width = field.decimals as usize
                        ))
                    }
                } else {
                    FieldValue::Text(text)
                }
            } else {
                FieldValue::Text(text)
            }
        }
        _ => render_structured(bytes, field),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field(
        data_type: DataType,
        offset: usize,
        length: usize,
        decimals: u8,
    ) -> FieldDefinition {
        FieldDefinition {
            field_name: "test".to_string(),
            offset,
            length,
            data_type,
            decimals,
            identifiers: vec![],
            filters: vec![],
        }
    }

    // Validates: Requirement 3.2
    #[test]
    fn render_raw_mode_printable_text() {
        let record = b"Hello World";
        let field = make_field(DataType::Str, 0, 5, 0);
        let result = render_field(record, &field, DisplayMode::Raw);
        assert_eq!(result, FieldValue::Text("Hello".to_string()));
    }

    #[test]
    fn render_raw_mode_binary_as_hex() {
        let record = &[0x00, 0x01, 0xFF, 0xFE];
        let field = make_field(DataType::Str, 0, 4, 0);
        let result = render_field(record, &field, DisplayMode::Raw);
        assert!(matches!(result, FieldValue::Hex(_)));
    }

    #[test]
    fn render_structured_string_trims_trailing_spaces() {
        let record = b"Hello   ";
        let field = make_field(DataType::Str, 0, 8, 0);
        let result = render_field(record, &field, DisplayMode::Structured);
        assert_eq!(result, FieldValue::Text("Hello".to_string()));
    }

    #[test]
    fn render_structured_int() {
        let record = b"  1234  ";
        let field = make_field(DataType::Int, 0, 8, 0);
        let result = render_field(record, &field, DisplayMode::Structured);
        assert_eq!(result, FieldValue::Text("1234".to_string()));
    }

    // Validates: Requirement 5.3
    #[test]
    fn render_structured_comp3() {
        let record = &[0x12, 0x34, 0x5C]; // +12345
        let field = make_field(DataType::Comp3, 0, 3, 0);
        let result = render_field(record, &field, DisplayMode::Structured);
        assert_eq!(result, FieldValue::Text("12345".to_string()));
    }

    // Validates: Requirement 5.7
    #[test]
    fn render_structured_comp3_invalid_shows_hex_error() {
        let record = &[0xAB, 0xCD, 0xE7]; // invalid nibbles
        let field = make_field(DataType::Comp3, 0, 3, 0);
        let result = render_field(record, &field, DisplayMode::Structured);
        assert!(result.is_error());
    }

    #[test]
    fn render_transformed_comp3_with_decimals() {
        let record = &[0x12, 0x34, 0x56, 0x7C]; // +1234567
        let field = make_field(DataType::Comp3, 0, 4, 2);
        let result = render_field(record, &field, DisplayMode::Transformed);
        assert_eq!(result, FieldValue::Text("12345.67".to_string()));
    }

    #[test]
    fn render_field_record_too_short_returns_error() {
        let record = b"AB";
        let field = make_field(DataType::Str, 0, 10, 0);
        let result = render_field(record, &field, DisplayMode::Structured);
        assert!(result.is_error());
    }
}
