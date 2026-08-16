//! Error types for the whitespace-guides subsystem.

/// Errors originating from the `ff-whitespace-guides` crate.
///
/// Formatted per Error Message Standards (Cross-cutting Requirement 8):
/// `[whitespace-guides] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WhitespaceGuidesError {
    /// Configuration key has an invalid value for whitespace mode.
    #[error(
        "[whitespace-guides] config: invalid whitespace mode '{value}' — using default 'invisible'"
    )]
    InvalidWhitespaceMode {
        /// The invalid value that was encountered.
        value: String,
    },

    /// Configuration key has an invalid value for tab draw mode.
    #[error(
        "[whitespace-guides] config: invalid tab draw mode '{value}' — using default 'long_arrow'"
    )]
    InvalidTabDrawMode {
        /// The invalid value that was encountered.
        value: String,
    },

    /// Configuration key has an invalid value for indent guide mode.
    #[error(
        "[whitespace-guides] config: invalid indent guide mode '{value}' — using default 'none'"
    )]
    InvalidIndentGuideMode {
        /// The invalid value that was encountered.
        value: String,
    },

    /// Configuration key has an invalid value for edge mode.
    #[error("[whitespace-guides] config: invalid edge mode '{value}' — using default 'none'")]
    InvalidEdgeMode {
        /// The invalid value that was encountered.
        value: String,
    },

    /// Configuration key has an invalid edge column value.
    #[error("[whitespace-guides] config: invalid edge column '{value}' — using default 80")]
    InvalidEdgeColumn {
        /// The invalid value that was encountered.
        value: String,
    },

    /// Configuration key has an invalid wrap flags value.
    #[error("[whitespace-guides] config: invalid wrap flags '{value}' — using default 0 (none)")]
    InvalidWrapFlags {
        /// The invalid value that was encountered.
        value: String,
    },

    /// Configuration system read error.
    #[error("[whitespace-guides] config: failed to read key '{key}' — {reason}")]
    ConfigReadError {
        /// The key that failed to read.
        key: String,
        /// Description of the failure.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_follow_format_standard() {
        // Validates: Cross-cutting Requirement 8
        let err = WhitespaceGuidesError::InvalidWhitespaceMode {
            value: "bad_mode".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[whitespace-guides]"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn all_error_variants_have_prefix() {
        // Validates: Cross-cutting Requirement 8
        let errors: Vec<WhitespaceGuidesError> = vec![
            WhitespaceGuidesError::InvalidWhitespaceMode {
                value: "x".to_string(),
            },
            WhitespaceGuidesError::InvalidTabDrawMode {
                value: "x".to_string(),
            },
            WhitespaceGuidesError::InvalidIndentGuideMode {
                value: "x".to_string(),
            },
            WhitespaceGuidesError::InvalidEdgeMode {
                value: "x".to_string(),
            },
            WhitespaceGuidesError::InvalidEdgeColumn {
                value: "x".to_string(),
            },
            WhitespaceGuidesError::InvalidWrapFlags {
                value: "x".to_string(),
            },
            WhitespaceGuidesError::ConfigReadError {
                key: "editor.x".to_string(),
                reason: "not found".to_string(),
            },
        ];

        for err in &errors {
            let msg = err.to_string();
            assert!(
                msg.starts_with("[whitespace-guides]"),
                "Error message does not start with prefix: {}",
                msg
            );
            assert!(msg.len() <= 200, "Error message exceeds 200 chars: {}", msg);
        }
    }

    #[test]
    fn config_read_error_includes_key_and_reason() {
        // Validates: Cross-cutting Requirement 8
        let err = WhitespaceGuidesError::ConfigReadError {
            key: "editor.whitespace_mode".to_string(),
            reason: "key not found in store".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("editor.whitespace_mode"));
        assert!(msg.contains("key not found in store"));
    }
}
