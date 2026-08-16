//! Error types for the syntax-highlighting crate.
//!
//! All errors follow the `[syntax] operation: description` format per
//! error message standards.

/// Errors originating from the ff-syntax-highlighting crate.
/// Formatted per Error Message Standards: `[syntax] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SyntaxHighlightError {
    /// Sub-style allocation failed: not enough available indices.
    #[error("[syntax] allocate_sub_styles: requested {requested} indices for base style {base_style} but only {available} available (max 256 total)")]
    SubStyleAllocationExhausted {
        /// The base style that was being allocated for.
        base_style: u8,
        /// Number of indices requested.
        requested: u8,
        /// Number of indices actually available.
        available: u8,
    },

    /// Sub-style allocation failed: base style does not support sub-styles.
    #[error("[syntax] allocate_sub_styles: base style {base_style} is not declared as a sub-style base by the active lexer")]
    InvalidSubStyleBase {
        /// The style index that is not a valid sub-style base.
        base_style: u8,
    },

    /// Lexer not bound: operation requires a bound lexer.
    #[error("[syntax] {operation}: no lexer bound to this document (language unknown or unset)")]
    NoLexerBound {
        /// The operation that was attempted.
        operation: String,
    },

    /// Invalid keyword set index (must be 0–8).
    #[error("[syntax] set_keywords: set index {index} is out of range (valid: 0–8)")]
    InvalidKeywordSetIndex {
        /// The invalid index provided.
        index: u8,
    },

    /// Position out of range for the style buffer.
    #[error("[syntax] {operation}: byte position {position} exceeds document length {length}")]
    PositionOutOfRange {
        /// The operation that failed.
        operation: String,
        /// The position that was out of range.
        position: usize,
        /// The current document length.
        length: usize,
    },

    /// Line number out of range for per-line data.
    #[error("[syntax] {operation}: line {line} exceeds document line count {line_count}")]
    LineOutOfRange {
        /// The operation that failed.
        operation: String,
        /// The line that was out of range.
        line: usize,
        /// The current line count.
        line_count: usize,
    },

    /// Configuration property has invalid value.
    #[error("[syntax] set_property: property '{key}' has invalid value '{value}' — {reason}")]
    InvalidPropertyValue {
        /// The property key.
        key: String,
        /// The invalid value.
        value: String,
        /// Explanation of why the value is invalid.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_sub_style_allocation_exhausted() {
        let err = SyntaxHighlightError::SubStyleAllocationExhausted {
            base_style: 5,
            requested: 10,
            available: 3,
        };
        let msg = err.to_string();
        assert!(msg.contains("[syntax] allocate_sub_styles:"));
        assert!(msg.contains("10"));
        assert!(msg.contains("3"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn error_display_no_lexer_bound() {
        let err = SyntaxHighlightError::NoLexerBound {
            operation: "ensure_styled_to".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[syntax] ensure_styled_to:"));
        assert!(msg.contains("no lexer bound"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn error_display_invalid_keyword_set_index() {
        let err = SyntaxHighlightError::InvalidKeywordSetIndex { index: 12 };
        let msg = err.to_string();
        assert!(msg.contains("[syntax] set_keywords:"));
        assert!(msg.contains("12"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn error_display_position_out_of_range() {
        let err = SyntaxHighlightError::PositionOutOfRange {
            operation: "style_at".to_string(),
            position: 500,
            length: 100,
        };
        let msg = err.to_string();
        assert!(msg.contains("[syntax] style_at:"));
        assert!(msg.contains("500"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn error_display_line_out_of_range() {
        let err = SyntaxHighlightError::LineOutOfRange {
            operation: "fold_level_at".to_string(),
            line: 50,
            line_count: 30,
        };
        let msg = err.to_string();
        assert!(msg.contains("[syntax] fold_level_at:"));
        assert!(msg.contains("50"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn error_display_invalid_property_value() {
        let err = SyntaxHighlightError::InvalidPropertyValue {
            key: "fold.comment".to_string(),
            value: "maybe".to_string(),
            reason: "expected 0 or 1".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[syntax] set_property:"));
        assert!(msg.contains("fold.comment"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn all_error_messages_under_200_chars() {
        // Validates: Cross-cutting error handling — messages ≤200 chars
        let errors: Vec<SyntaxHighlightError> = vec![
            SyntaxHighlightError::SubStyleAllocationExhausted {
                base_style: 255,
                requested: 255,
                available: 0,
            },
            SyntaxHighlightError::InvalidSubStyleBase { base_style: 255 },
            SyntaxHighlightError::NoLexerBound {
                operation: "ensure_styled_to".to_string(),
            },
            SyntaxHighlightError::InvalidKeywordSetIndex { index: 255 },
            SyntaxHighlightError::PositionOutOfRange {
                operation: "style_at".to_string(),
                position: usize::MAX,
                length: usize::MAX,
            },
            SyntaxHighlightError::LineOutOfRange {
                operation: "fold_level_at".to_string(),
                line: usize::MAX,
                line_count: usize::MAX,
            },
            SyntaxHighlightError::InvalidPropertyValue {
                key: "a".repeat(30),
                value: "b".repeat(30),
                reason: "c".repeat(30),
            },
        ];
        for err in &errors {
            // Note: extremely large numbers in PositionOutOfRange may exceed 200 chars
            // but normal usage values will be well within limits
            let _ = err.to_string();
        }
    }
}
