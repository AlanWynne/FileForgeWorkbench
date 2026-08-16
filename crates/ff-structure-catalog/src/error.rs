//! Error types for the ff-structure-catalog crate.
//!
//! All errors follow the message standard: `[structure-catalog] operation: description`

use std::fmt;

/// All errors produced by the ff-structure-catalog crate.
///
/// Follows the error message standard: `[structure-catalog] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CatalogError {
    /// Structure definition not found in the active catalog location.
    #[error("[structure-catalog] read: structure '{name}' not found in active location")]
    NotFound {
        /// The name that was searched for.
        name: String,
    },

    /// A structure with the same name already exists.
    #[error("[structure-catalog] create: structure '{name}' already exists in active location")]
    DuplicateName {
        /// The duplicated name.
        name: String,
    },

    /// Validation of a structure definition failed.
    #[error("[structure-catalog] validate: {detail}")]
    ValidationFailed {
        /// Description of the validation failure.
        detail: String,
    },

    /// An I/O error occurred during a catalog operation.
    #[error("[structure-catalog] io: {operation} failed for '{path}': {source}")]
    Io {
        /// What operation was being performed.
        operation: String,
        /// The path involved.
        path: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// TOML parsing failed for an .ffs file.
    #[error("[structure-catalog] parse: invalid TOML in '{path}': {detail}")]
    ParseError {
        /// Path of the file that failed parsing.
        path: String,
        /// Details of the parse error.
        detail: String,
    },

    /// Schema validation failed for a parsed .ffs file.
    #[error("[structure-catalog] validate: schema violation in '{path}': {detail}")]
    SchemaError {
        /// Path of the file that failed validation.
        path: String,
        /// Details of the schema violation.
        detail: String,
    },

    /// A catalog location is inaccessible.
    #[error("[structure-catalog] location: path '{path}' is inaccessible: {reason}")]
    PermissionDenied {
        /// The inaccessible path.
        path: String,
        /// Reason for inaccessibility.
        reason: String,
    },

    /// A catalog location does not exist.
    #[error("[structure-catalog] location: path '{path}' does not exist")]
    LocationNotFound {
        /// The missing path.
        path: String,
    },

    /// Configuration error.
    #[error("[structure-catalog] config: {detail}")]
    ConfigError {
        /// Description of the configuration problem.
        detail: String,
    },

    /// Import operation failed.
    #[error("[structure-catalog] import: failed to parse {format} file '{path}': {detail}")]
    ImportError {
        /// The format being imported.
        format: String,
        /// The source file path.
        path: String,
        /// Details of the import failure.
        detail: String,
    },

    /// Export operation failed.
    #[error("[structure-catalog] export: failed to write {format} to '{path}': {detail}")]
    ExportError {
        /// The target format.
        format: String,
        /// The destination path.
        path: String,
        /// Details of the export failure.
        detail: String,
    },

    /// External modification conflict detected.
    #[error("[structure-catalog] conflict: external modification detected for '{name}'")]
    ConflictDetected {
        /// The structure that was externally modified.
        name: String,
    },

    /// Deletion was not confirmed.
    #[error("[structure-catalog] delete: deletion of '{name}' not confirmed")]
    DeleteNotConfirmed {
        /// The structure targeted for deletion.
        name: String,
    },

    /// COBOL copybook parse error.
    #[error("[structure-catalog] copybook: parse error at line {line}: {detail}")]
    CopybookParseError {
        /// The line where the error occurred.
        line: u32,
        /// Details of the parse error.
        detail: String,
    },
}

impl From<std::io::Error> for CatalogError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            operation: "unknown".to_string(),
            path: String::new(),
            source: err,
        }
    }
}

impl From<toml::de::Error> for CatalogError {
    fn from(err: toml::de::Error) -> Self {
        Self::ParseError {
            path: String::new(),
            detail: err.to_string(),
        }
    }
}

/// Errors specific to field value interpretation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FieldInterpretError {
    /// Not enough bytes in the record to extract this field.
    #[error("field '{name}' at offset {offset}: insufficient bytes (need {need}, have {have})")]
    InsufficientBytes {
        /// Field name.
        name: String,
        /// Field offset in bytes.
        offset: u32,
        /// Bytes needed.
        need: u32,
        /// Bytes available.
        have: u32,
    },

    /// Invalid packed-decimal nibble values.
    #[error("field '{name}': invalid packed-decimal nibbles at byte {byte_index}")]
    InvalidPackedDecimal {
        /// Field name.
        name: String,
        /// Byte index with the invalid nibble.
        byte_index: u32,
    },

    /// Character encoding error.
    #[error("field '{name}': encoding error: {detail}")]
    EncodingError {
        /// Field name.
        name: String,
        /// Encoding error details.
        detail: String,
    },
}

/// Errors specific to field value validation during editing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FieldValidationError {
    /// Value exceeds the declared field length.
    #[error("value exceeds field length {max_length}")]
    TooLong {
        /// Maximum allowed length.
        max_length: u32,
    },

    /// Non-numeric characters in a numeric field.
    #[error("non-numeric characters in numeric field")]
    NonNumeric,

    /// Invalid decimal format.
    #[error("invalid decimal format")]
    InvalidDecimal,

    /// Field name must not be empty.
    #[error("field name must be non-empty")]
    EmptyName,

    /// Offset must be non-negative (always true for u32, but validated for semantic clarity).
    #[error("offset must be >= 0")]
    NegativeOffset,

    /// Length must be at least 1.
    #[error("length must be >= 1")]
    ZeroLength,

    /// Invalid field type string value.
    #[error("invalid field type value: {value}")]
    InvalidFieldType {
        /// The invalid value.
        value: String,
    },
}

/// A collection of validation errors for a single field definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors {
    /// All validation errors found.
    pub errors: Vec<FieldValidationError>,
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{err}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 28 — Error display output
    #[test]
    fn catalog_error_not_found_displays_correctly() {
        let err = CatalogError::NotFound {
            name: "CUSTOMER".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[structure-catalog] read: structure 'CUSTOMER' not found in active location"
        );
    }

    // Validates: Requirement 28 — Error display output
    #[test]
    fn catalog_error_duplicate_name_displays_correctly() {
        let err = CatalogError::DuplicateName {
            name: "INVOICE".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[structure-catalog] create: structure 'INVOICE' already exists in active location"
        );
    }

    // Validates: Requirement 28 — Error display output
    #[test]
    fn catalog_error_delete_not_confirmed_displays_correctly() {
        let err = CatalogError::DeleteNotConfirmed {
            name: "OLD_STRUCT".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[structure-catalog] delete: deletion of 'OLD_STRUCT' not confirmed"
        );
    }

    // Validates: Requirement 28 — From<io::Error> conversion
    #[test]
    fn from_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: CatalogError = io_err.into();
        assert!(matches!(err, CatalogError::Io { .. }));
    }

    // Validates: Requirement 28 — From<toml::de::Error> conversion
    #[test]
    fn from_toml_error_conversion() {
        let toml_result: Result<toml::Value, _> = toml::from_str("invalid [[[");
        let toml_err = toml_result.unwrap_err();
        let err: CatalogError = toml_err.into();
        assert!(matches!(err, CatalogError::ParseError { .. }));
    }

    // Validates: Requirement 28 — Field validation errors
    #[test]
    fn field_validation_error_empty_name_displays_correctly() {
        let err = FieldValidationError::EmptyName;
        assert_eq!(err.to_string(), "field name must be non-empty");
    }

    // Validates: Requirement 28 — Field validation errors
    #[test]
    fn field_validation_error_zero_length_displays_correctly() {
        let err = FieldValidationError::ZeroLength;
        assert_eq!(err.to_string(), "length must be >= 1");
    }

    // Validates: Requirement 28 — ValidationErrors collection display
    #[test]
    fn validation_errors_collection_displays_all() {
        let errs = ValidationErrors {
            errors: vec![
                FieldValidationError::EmptyName,
                FieldValidationError::ZeroLength,
            ],
        };
        assert_eq!(
            errs.to_string(),
            "field name must be non-empty; length must be >= 1"
        );
    }
}
