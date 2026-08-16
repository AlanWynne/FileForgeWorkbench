//! Error types for the ff-encoding crate.
//!
//! All error messages follow the format: `[encoding] operation: description`

/// Errors produced by the ff-encoding crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EncodingError {
    /// The encoding is not supported for the requested operation.
    #[error("[encoding] conversion: unsupported encoding '{name}'")]
    UnsupportedEncoding {
        /// The name of the unsupported encoding.
        name: String,
    },

    /// Encoding detection could not determine the file encoding.
    #[error("[encoding] detection: failed to detect encoding (examined {bytes_examined} bytes)")]
    DetectionFailed {
        /// Number of bytes that were examined before giving up.
        bytes_examined: usize,
    },

    /// A character cannot be represented in the target encoding.
    #[error(
        "[encoding] conversion: unmappable character U+{code_point:04X} at byte offset {offset}"
    )]
    UnmappableCharacter {
        /// The Unicode code point that cannot be mapped.
        code_point: u32,
        /// Byte offset in the source where the unmappable character was found.
        offset: usize,
    },

    /// The BOM encoding requested does not have a BOM sequence.
    #[error("[encoding] bom: no BOM defined for encoding '{encoding}'")]
    NoBomForEncoding {
        /// The encoding name that was requested.
        encoding: String,
    },

    /// Invalid UTF-8 encountered where valid UTF-8 was required.
    #[error("[encoding] utf8: invalid UTF-8 at byte offset {offset}")]
    InvalidUtf8 {
        /// Byte offset where the invalid sequence starts.
        offset: usize,
    },

    /// Invalid byte offset (not on a character boundary).
    #[error("[encoding] navigation: byte offset {offset} is not on a character boundary")]
    InvalidBoundary {
        /// The invalid byte offset.
        offset: usize,
    },

    /// The code page is not a valid DBCS code page.
    #[error("[encoding] dbcs: code page {code_page} is not a supported DBCS code page")]
    InvalidDbcsCodePage {
        /// The invalid code page number.
        code_page: u32,
    },

    /// I/O error during streaming conversion.
    #[error("[encoding] io: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_unsupported_encoding() {
        let err = EncodingError::UnsupportedEncoding {
            name: "unknown-enc".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[encoding] conversion: unsupported encoding 'unknown-enc'"
        );
    }

    #[test]
    fn error_display_detection_failed() {
        let err = EncodingError::DetectionFailed {
            bytes_examined: 8192,
        };
        assert_eq!(
            err.to_string(),
            "[encoding] detection: failed to detect encoding (examined 8192 bytes)"
        );
    }

    #[test]
    fn error_display_unmappable_character() {
        let err = EncodingError::UnmappableCharacter {
            code_point: 0x1F600,
            offset: 42,
        };
        assert_eq!(
            err.to_string(),
            "[encoding] conversion: unmappable character U+1F600 at byte offset 42"
        );
    }

    #[test]
    fn error_display_no_bom_for_encoding() {
        let err = EncodingError::NoBomForEncoding {
            encoding: "iso-8859-1".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[encoding] bom: no BOM defined for encoding 'iso-8859-1'"
        );
    }

    #[test]
    fn error_display_invalid_utf8() {
        let err = EncodingError::InvalidUtf8 { offset: 10 };
        assert_eq!(
            err.to_string(),
            "[encoding] utf8: invalid UTF-8 at byte offset 10"
        );
    }

    #[test]
    fn error_display_invalid_boundary() {
        let err = EncodingError::InvalidBoundary { offset: 5 };
        assert_eq!(
            err.to_string(),
            "[encoding] navigation: byte offset 5 is not on a character boundary"
        );
    }

    #[test]
    fn error_display_invalid_dbcs_code_page() {
        let err = EncodingError::InvalidDbcsCodePage { code_page: 999 };
        assert_eq!(
            err.to_string(),
            "[encoding] dbcs: code page 999 is not a supported DBCS code page"
        );
    }
}
