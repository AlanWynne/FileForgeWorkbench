//! Error types for the ff-forge crate.
//!
//! All error messages follow the format: `[fileforge] operation: description`

/// Errors produced by the ff-forge crate.
///
/// Each variant carries enough context to diagnose the problem without
/// needing to inspect logs or source code.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FileForgeError {
    /// Structure file contains invalid JSON or is missing required fields.
    #[error("[fileforge] parse structure: {description}")]
    StructureParse {
        /// Human-readable description of the parse failure.
        description: String,
    },

    /// A field value does not conform to its declared data type or constraints.
    #[error("[fileforge] validate field: '{field_name}' — {reason}")]
    FieldValidation {
        /// Name of the field that failed validation.
        field_name: String,
        /// Explanation of what was wrong with the value.
        reason: String,
    },

    /// A field edit would produce a byte sequence exceeding the declared field length.
    #[error("[fileforge] encode field: '{field_name}' value exceeds {max_length} bytes (got {actual_length})")]
    FieldOverflow {
        /// Name of the field that overflowed.
        field_name: String,
        /// Maximum byte length allowed for this field.
        max_length: usize,
        /// Actual byte length of the encoded value.
        actual_length: usize,
    },

    /// VB binary file has an invalid Record Descriptor Word.
    #[error("[fileforge] read VB record: invalid RDW at byte offset {byte_offset} — {reason}")]
    InvalidRdw {
        /// Byte offset in the file where the invalid RDW was found.
        byte_offset: u64,
        /// Explanation of the RDW error.
        reason: String,
    },

    /// A character has no mapping in the target EBCDIC code page.
    #[error("[fileforge] encode EBCDIC: character '{character}' has no mapping in {code_page}")]
    EncodingError {
        /// The character that could not be mapped.
        character: char,
        /// The target code page name.
        code_page: String,
    },

    /// I/O error during file read, write, or seek.
    #[error("[fileforge] {operation}: I/O error on '{uri}' — {source}")]
    IoError {
        /// The operation that was being performed.
        operation: String,
        /// The resource URI that was being accessed.
        uri: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Source file is empty (zero bytes).
    #[error("[fileforge] open file: file is empty: '{uri}'")]
    EmptyFile {
        /// URI of the empty file.
        uri: String,
    },

    /// Source file or structure file not found via VFS.
    #[error("[fileforge] open file: resource not found: '{uri}'")]
    ResourceNotFound {
        /// URI of the missing resource.
        uri: String,
    },

    /// Requested output format is not supported for conversion.
    #[error("[fileforge] convert: unsupported output format '{format}'")]
    UnsupportedOutputFormat {
        /// The format string that was not recognised.
        format: String,
    },

    /// LRECL auto-detection could not determine a uniform record length.
    #[error("[fileforge] detect LRECL: non-uniform line lengths in first {sample_size} lines")]
    LreclDetectionFailed {
        /// Number of lines sampled before giving up.
        sample_size: usize,
    },

    /// COMP-3 field contains invalid nibbles.
    #[error("[fileforge] decode COMP-3: invalid packed decimal in field '{field_name}' at offset {offset}")]
    InvalidComp3 {
        /// Name of the field with invalid COMP-3 data.
        field_name: String,
        /// Byte offset within the record where the error was detected.
        offset: usize,
    },

    /// Unexpected end of file while reading a record.
    #[error("[fileforge] read record: unexpected EOF at byte offset {byte_offset}, expected {expected} bytes")]
    UnexpectedEof {
        /// Byte offset in the file where EOF was encountered.
        byte_offset: u64,
        /// Number of bytes expected at this position.
        expected: usize,
    },

    /// Record index is out of range.
    #[error("[fileforge] navigate: record {requested} is out of range (file has {total} records)")]
    RecordOutOfRange {
        /// The record number that was requested.
        requested: usize,
        /// Total number of records in the file.
        total: usize,
    },

    /// No active FileForge session for the requested operation.
    #[error(
        "[fileforge] {operation}: no FileForge session is active — load a structure file first"
    )]
    NoActiveSession {
        /// The operation that required an active session.
        operation: String,
    },
}

impl From<std::io::Error> for FileForgeError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            operation: "io".to_string(),
            uri: String::new(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for FileForgeError {
    fn from(err: serde_json::Error) -> Self {
        Self::StructureParse {
            description: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 16
    #[test]
    fn error_display_structure_parse() {
        let err = FileForgeError::StructureParse {
            description: "missing 'structures' array".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] parse structure: missing 'structures' array"
        );
    }

    #[test]
    fn error_display_field_validation() {
        let err = FileForgeError::FieldValidation {
            field_name: "amount".to_string(),
            reason: "expected integer, got 'abc'".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] validate field: 'amount' — expected integer, got 'abc'"
        );
    }

    #[test]
    fn error_display_field_overflow() {
        let err = FileForgeError::FieldOverflow {
            field_name: "name".to_string(),
            max_length: 10,
            actual_length: 15,
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] encode field: 'name' value exceeds 10 bytes (got 15)"
        );
    }

    #[test]
    fn error_display_invalid_rdw() {
        let err = FileForgeError::InvalidRdw {
            byte_offset: 1024,
            reason: "record length < 4".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] read VB record: invalid RDW at byte offset 1024 — record length < 4"
        );
    }

    #[test]
    fn error_display_encoding_error() {
        let err = FileForgeError::EncodingError {
            character: '€',
            code_page: "EBCDIC-037".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] encode EBCDIC: character '€' has no mapping in EBCDIC-037"
        );
    }

    #[test]
    fn error_display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = FileForgeError::IoError {
            operation: "read index".to_string(),
            uri: "/data/file.dat".to_string(),
            source: io_err,
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] read index: I/O error on '/data/file.dat' — file not found"
        );
    }

    #[test]
    fn error_display_empty_file() {
        let err = FileForgeError::EmptyFile {
            uri: "/data/empty.dat".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] open file: file is empty: '/data/empty.dat'"
        );
    }

    #[test]
    fn error_display_resource_not_found() {
        let err = FileForgeError::ResourceNotFound {
            uri: "/data/missing.ffs".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] open file: resource not found: '/data/missing.ffs'"
        );
    }

    #[test]
    fn error_display_unsupported_output_format() {
        let err = FileForgeError::UnsupportedOutputFormat {
            format: "xlsx".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] convert: unsupported output format 'xlsx'"
        );
    }

    #[test]
    fn error_display_lrecl_detection_failed() {
        let err = FileForgeError::LreclDetectionFailed { sample_size: 100 };
        assert_eq!(
            err.to_string(),
            "[fileforge] detect LRECL: non-uniform line lengths in first 100 lines"
        );
    }

    #[test]
    fn error_display_invalid_comp3() {
        let err = FileForgeError::InvalidComp3 {
            field_name: "balance".to_string(),
            offset: 20,
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] decode COMP-3: invalid packed decimal in field 'balance' at offset 20"
        );
    }

    #[test]
    fn error_display_unexpected_eof() {
        let err = FileForgeError::UnexpectedEof {
            byte_offset: 5000,
            expected: 80,
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] read record: unexpected EOF at byte offset 5000, expected 80 bytes"
        );
    }

    #[test]
    fn error_display_record_out_of_range() {
        let err = FileForgeError::RecordOutOfRange {
            requested: 1001,
            total: 1000,
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] navigate: record 1001 is out of range (file has 1000 records)"
        );
    }

    #[test]
    fn error_display_no_active_session() {
        let err = FileForgeError::NoActiveSession {
            operation: "convert".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[fileforge] convert: no FileForge session is active — load a structure file first"
        );
    }

    #[test]
    fn from_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: FileForgeError = io_err.into();
        assert!(matches!(err, FileForgeError::IoError { .. }));
    }

    #[test]
    fn from_serde_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("{{invalid").unwrap_err();
        let err: FileForgeError = json_err.into();
        assert!(matches!(err, FileForgeError::StructureParse { .. }));
    }
}
