//! Error types for the find-and-replace crate.
//!
//! All errors follow the `[find-replace] operation: description` format.
//!
//! Addresses: Requirement 20, cross-cutting error standards

/// Errors originating from the ff-find-and-replace crate.
///
/// Formatted per Error Message Standards: `[find-replace] operation: description`
///
/// Addresses: Requirements 3, 4, 5, 9, 12, 20
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FindReplaceError {
    /// No search term specified and no previous search to reuse.
    /// Addresses: Requirement 20 AC 1
    #[error("[find-replace] find: no search term specified")]
    NoSearchTerm,

    /// No previous FIND to repeat (RFIND with no history).
    /// Addresses: Requirement 5 AC 2
    #[error("[find-replace] rfind: no previous FIND to repeat")]
    NoPreviousFind,

    /// No previous CHANGE to repeat (RCHANGE with no history).
    /// Addresses: Requirement 9 AC 2
    #[error("[find-replace] rchange: no previous CHANGE to repeat")]
    NoPreviousChange,

    /// Document is read-only; CHANGE not permitted.
    /// Addresses: Requirement 20 AC 4
    #[error("[find-replace] change: document is read-only")]
    DocumentReadOnly,

    /// Invalid hex pattern: odd number of digits.
    /// Addresses: Requirement 3 AC 2
    #[error("[find-replace] find: invalid hex pattern: odd number of digits")]
    HexOddDigits,

    /// Invalid hex pattern: non-hex character encountered.
    /// Addresses: Requirement 3 AC 3
    #[error("[find-replace] find: invalid hex pattern: non-hex character '{0}'")]
    HexInvalidChar(char),

    /// Regex compilation error.
    /// Addresses: Requirement 12 AC 2–9
    #[error("[find-replace] regex: {message}")]
    RegexCompile { message: String },

    /// Regex pattern too long (NFA exceeds max size).
    /// Addresses: Requirement 12 AC 2
    #[error("[find-replace] regex: pattern too long")]
    RegexPatternTooLong,

    /// No previous regular expression to reuse.
    /// Addresses: Requirement 12 AC 9
    #[error("[find-replace] regex: no previous regular expression")]
    NoPreviousRegex,

    /// Invalid substitution template escape.
    /// Addresses: Requirement 20 AC 5
    #[error("[find-replace] replace: invalid escape in replacement template: {detail}")]
    InvalidSubstitution { detail: String },

    /// Search was cancelled via cancellation token.
    /// Addresses: Requirement 19 AC 1
    #[error("[find-replace] find: operation cancelled")]
    Cancelled,

    /// Internal error from document access.
    #[error("[find-replace] {operation}: document error: {detail}")]
    DocumentAccess { operation: String, detail: String },

    /// Serialisation/deserialisation error for FindState.
    #[error("[find-replace] state: serialisation error: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_follow_standard_format() {
        let err = FindReplaceError::NoSearchTerm;
        assert!(err.to_string().starts_with("[find-replace]"));

        let err = FindReplaceError::NoPreviousFind;
        assert!(err.to_string().contains("rfind"));

        let err = FindReplaceError::HexInvalidChar('G');
        assert!(err.to_string().contains("'G'"));

        let err = FindReplaceError::RegexCompile {
            message: "Unmatched (".to_string(),
        };
        assert!(err.to_string().contains("Unmatched ("));
    }

    #[test]
    fn all_error_variants_produce_non_empty_messages() {
        let errors: Vec<FindReplaceError> = vec![
            FindReplaceError::NoSearchTerm,
            FindReplaceError::NoPreviousFind,
            FindReplaceError::NoPreviousChange,
            FindReplaceError::DocumentReadOnly,
            FindReplaceError::HexOddDigits,
            FindReplaceError::HexInvalidChar('X'),
            FindReplaceError::RegexCompile {
                message: "test".into(),
            },
            FindReplaceError::RegexPatternTooLong,
            FindReplaceError::NoPreviousRegex,
            FindReplaceError::InvalidSubstitution {
                detail: "test".into(),
            },
            FindReplaceError::Cancelled,
            FindReplaceError::DocumentAccess {
                operation: "find".into(),
                detail: "test".into(),
            },
            FindReplaceError::Serialization("test".into()),
        ];

        for err in &errors {
            assert!(
                !err.to_string().is_empty(),
                "Empty error message: {:?}",
                err
            );
        }
    }
}
