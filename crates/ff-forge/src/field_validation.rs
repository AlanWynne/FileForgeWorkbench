//! Field validation engine.
//!
//! Validates user input against field data types and length constraints
//! before applying edits to the document buffer.

use crate::error::FileForgeError;
use crate::field_def::{DataType, FieldDefinition};

/// Validates field input values against their declared data type.
pub struct FieldValidator;

impl FieldValidator {
    /// Validates a value against a field's data type and length.
    ///
    /// # Errors
    ///
    /// Returns `FileForgeError::FieldValidation` if the value fails type checks.
    /// Returns `FileForgeError::FieldOverflow` if the encoded value exceeds field length.
    pub fn validate(field: &FieldDefinition, value: &str) -> Result<(), FileForgeError> {
        match field.data_type {
            DataType::Str => Self::validate_str(field, value),
            DataType::Int => Self::validate_int(field, value),
            DataType::Float => Self::validate_float(field, value),
            DataType::Bool => Self::validate_bool(field, value),
            DataType::Comp3 => Self::validate_comp3(field, value),
        }
    }

    /// Validates a string field — accepts any input that fits the byte length.
    fn validate_str(field: &FieldDefinition, value: &str) -> Result<(), FileForgeError> {
        if value.len() > field.length {
            return Err(FileForgeError::FieldOverflow {
                field_name: field.field_name.clone(),
                max_length: field.length,
                actual_length: value.len(),
            });
        }
        Ok(())
    }

    /// Validates an integer field — optional leading sign + digits only.
    fn validate_int(field: &FieldDefinition, value: &str) -> Result<(), FileForgeError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(FileForgeError::FieldValidation {
                field_name: field.field_name.clone(),
                reason: "empty value for integer field".to_string(),
            });
        }

        let check = if trimmed.starts_with('+') || trimmed.starts_with('-') {
            &trimmed[1..]
        } else {
            trimmed
        };

        if check.is_empty() || !check.chars().all(|c| c.is_ascii_digit()) {
            return Err(FileForgeError::FieldValidation {
                field_name: field.field_name.clone(),
                reason: format!("expected integer, got '{value}'"),
            });
        }

        // Check byte length
        if trimmed.len() > field.length {
            return Err(FileForgeError::FieldOverflow {
                field_name: field.field_name.clone(),
                max_length: field.length,
                actual_length: trimmed.len(),
            });
        }

        Ok(())
    }

    /// Validates a float field — optional sign + digits + optional decimal point.
    fn validate_float(field: &FieldDefinition, value: &str) -> Result<(), FileForgeError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(FileForgeError::FieldValidation {
                field_name: field.field_name.clone(),
                reason: "empty value for float field".to_string(),
            });
        }

        let check = if trimmed.starts_with('+') || trimmed.starts_with('-') {
            &trimmed[1..]
        } else {
            trimmed
        };

        if check.is_empty() {
            return Err(FileForgeError::FieldValidation {
                field_name: field.field_name.clone(),
                reason: format!("expected numeric value, got '{value}'"),
            });
        }

        let mut dot_seen = false;
        for ch in check.chars() {
            if ch == '.' {
                if dot_seen {
                    return Err(FileForgeError::FieldValidation {
                        field_name: field.field_name.clone(),
                        reason: format!("expected numeric value, got '{value}'"),
                    });
                }
                dot_seen = true;
            } else if !ch.is_ascii_digit() {
                return Err(FileForgeError::FieldValidation {
                    field_name: field.field_name.clone(),
                    reason: format!("expected numeric value, got '{value}'"),
                });
            }
        }

        if trimmed.len() > field.length {
            return Err(FileForgeError::FieldOverflow {
                field_name: field.field_name.clone(),
                max_length: field.length,
                actual_length: trimmed.len(),
            });
        }

        Ok(())
    }

    /// Validates a bool field — accepts true/false/T/F/Y/N/1/0 (case-insensitive).
    fn validate_bool(field: &FieldDefinition, value: &str) -> Result<(), FileForgeError> {
        let trimmed = value.trim().to_lowercase();
        let valid = matches!(
            trimmed.as_str(),
            "true" | "false" | "t" | "f" | "y" | "n" | "1" | "0"
        );
        if !valid {
            return Err(FileForgeError::FieldValidation {
                field_name: field.field_name.clone(),
                reason: format!("expected boolean (true/false/T/F/Y/N/1/0), got '{value}'"),
            });
        }
        Ok(())
    }

    /// Validates a COMP-3 field — accepts decimal numeric input that fits field length.
    fn validate_comp3(field: &FieldDefinition, value: &str) -> Result<(), FileForgeError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(FileForgeError::FieldValidation {
                field_name: field.field_name.clone(),
                reason: "empty value for COMP-3 field".to_string(),
            });
        }

        // Validate it's a valid decimal number
        let check = if trimmed.starts_with('+') || trimmed.starts_with('-') {
            &trimmed[1..]
        } else {
            trimmed
        };

        if check.is_empty() {
            return Err(FileForgeError::FieldValidation {
                field_name: field.field_name.clone(),
                reason: format!("expected decimal number, got '{value}'"),
            });
        }

        let mut dot_seen = false;
        for ch in check.chars() {
            if ch == '.' {
                if dot_seen {
                    return Err(FileForgeError::FieldValidation {
                        field_name: field.field_name.clone(),
                        reason: format!("expected decimal number, got '{value}'"),
                    });
                }
                dot_seen = true;
            } else if !ch.is_ascii_digit() {
                return Err(FileForgeError::FieldValidation {
                    field_name: field.field_name.clone(),
                    reason: format!("expected decimal number, got '{value}'"),
                });
            }
        }

        // Check that the encoded value fits in field length
        // Max digits = (field.length * 2) - 1
        let max_digits = field.length * 2 - 1;
        let digit_count: usize = check.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count > max_digits {
            return Err(FileForgeError::FieldOverflow {
                field_name: field.field_name.clone(),
                max_length: field.length,
                actual_length: (digit_count + 2) / 2,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field(name: &str, data_type: DataType, length: usize) -> FieldDefinition {
        FieldDefinition {
            field_name: name.to_string(),
            offset: 0,
            length,
            data_type,
            decimals: 0,
            identifiers: vec![],
            filters: vec![],
        }
    }

    // Validates: Requirement 9.1
    #[test]
    fn validate_int_accepts_valid_integers() {
        let field = make_field("count", DataType::Int, 10);
        assert!(FieldValidator::validate(&field, "123").is_ok());
        assert!(FieldValidator::validate(&field, "-456").is_ok());
        assert!(FieldValidator::validate(&field, "+789").is_ok());
        assert!(FieldValidator::validate(&field, "0").is_ok());
    }

    #[test]
    fn validate_int_rejects_non_numeric() {
        let field = make_field("count", DataType::Int, 10);
        assert!(FieldValidator::validate(&field, "abc").is_err());
        assert!(FieldValidator::validate(&field, "12.34").is_err());
        assert!(FieldValidator::validate(&field, "").is_err());
    }

    // Validates: Requirement 9.2
    #[test]
    fn validate_float_accepts_valid_decimals() {
        let field = make_field("rate", DataType::Float, 10);
        assert!(FieldValidator::validate(&field, "3.14").is_ok());
        assert!(FieldValidator::validate(&field, "-2.5").is_ok());
        assert!(FieldValidator::validate(&field, "100").is_ok());
        assert!(FieldValidator::validate(&field, ".5").is_ok());
    }

    #[test]
    fn validate_float_rejects_invalid() {
        let field = make_field("rate", DataType::Float, 10);
        assert!(FieldValidator::validate(&field, "abc").is_err());
        assert!(FieldValidator::validate(&field, "1.2.3").is_err());
        assert!(FieldValidator::validate(&field, "").is_err());
    }

    // Validates: Requirement 9.3
    #[test]
    fn validate_bool_accepts_recognised_values() {
        let field = make_field("flag", DataType::Bool, 5);
        for val in &[
            "true", "false", "T", "F", "Y", "N", "1", "0", "TRUE", "False",
        ] {
            assert!(
                FieldValidator::validate(&field, val).is_ok(),
                "Should accept: {val}"
            );
        }
    }

    #[test]
    fn validate_bool_rejects_unrecognised() {
        let field = make_field("flag", DataType::Bool, 5);
        assert!(FieldValidator::validate(&field, "yes").is_err());
        assert!(FieldValidator::validate(&field, "no").is_err());
        assert!(FieldValidator::validate(&field, "maybe").is_err());
    }

    // Validates: Requirement 9.4
    #[test]
    fn validate_str_accepts_within_length() {
        let field = make_field("name", DataType::Str, 10);
        assert!(FieldValidator::validate(&field, "Hello").is_ok());
        assert!(FieldValidator::validate(&field, "1234567890").is_ok());
    }

    // Validates: Requirement 9.6
    #[test]
    fn validate_str_rejects_over_length() {
        let field = make_field("name", DataType::Str, 5);
        let result = FieldValidator::validate(&field, "Too Long!");
        assert!(matches!(result, Err(FileForgeError::FieldOverflow { .. })));
    }

    // Validates: Requirement 9.5
    #[test]
    fn validate_comp3_accepts_valid_decimal() {
        let field = make_field("amount", DataType::Comp3, 4);
        assert!(FieldValidator::validate(&field, "1234567").is_ok());
        assert!(FieldValidator::validate(&field, "-100").is_ok());
        assert!(FieldValidator::validate(&field, "12.34").is_ok());
    }

    #[test]
    fn validate_comp3_rejects_non_numeric() {
        let field = make_field("amount", DataType::Comp3, 4);
        assert!(FieldValidator::validate(&field, "abc").is_err());
    }

    #[test]
    fn validate_comp3_rejects_overflow() {
        // 2-byte field: max 3 digits (2*2-1=3)
        let field = make_field("small", DataType::Comp3, 2);
        let result = FieldValidator::validate(&field, "12345");
        assert!(matches!(result, Err(FileForgeError::FieldOverflow { .. })));
    }

    // Validates: Requirement 9.6
    #[test]
    fn validate_int_overflow_detection() {
        let field = make_field("tiny", DataType::Int, 3);
        let result = FieldValidator::validate(&field, "12345");
        assert!(matches!(result, Err(FileForgeError::FieldOverflow { .. })));
    }
}
