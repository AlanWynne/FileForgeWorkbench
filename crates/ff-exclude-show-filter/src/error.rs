//! Error types for the ff-exclude-show-filter crate.
//!
//! All errors follow the `[exclude-filter] operation: description` format
//! per cross-cutting Requirement 8.

/// Errors originating from the ff-exclude-show-filter crate.
///
/// Addresses: Cross-cutting Requirement 8, Requirement 9 AC 8
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExcludeFilterError {
    /// Invalid regex pattern in EXCLUDE REGEX or SHOW REGEX.
    #[error("[exclude-filter] {command}: invalid regex pattern: {detail}")]
    InvalidRegex { command: String, detail: String },

    /// Invalid line range arguments (start > end, non-numeric, out of bounds).
    #[error(
        "[exclude-filter] exclude: invalid line range {start}–{end} (document has {total} lines)"
    )]
    InvalidRange {
        start: usize,
        end: usize,
        total: usize,
    },

    /// Unterminated quote in text argument.
    #[error("[exclude-filter] {command}: unterminated quote in argument")]
    UnterminatedQuote { command: String },

    /// Invalid argument syntax.
    #[error("[exclude-filter] {command}: invalid arguments: {detail}")]
    InvalidArgument { command: String, detail: String },

    /// Line out of range for the document.
    #[error(
        "[exclude-filter] {operation}: line {line} is out of range (document has {total} lines)"
    )]
    LineOutOfRange {
        operation: String,
        line: usize,
        total: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_regex_error_formats_correctly() {
        let err = ExcludeFilterError::InvalidRegex {
            command: "exclude".to_string(),
            detail: "unclosed group".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[exclude-filter] exclude: invalid regex pattern: unclosed group"
        );
    }

    #[test]
    fn invalid_range_error_formats_correctly() {
        let err = ExcludeFilterError::InvalidRange {
            start: 50,
            end: 10,
            total: 100,
        };
        assert!(err.to_string().contains("invalid line range 50–10"));
        assert!(err.to_string().contains("document has 100 lines"));
    }

    #[test]
    fn unterminated_quote_error_formats_correctly() {
        let err = ExcludeFilterError::UnterminatedQuote {
            command: "show".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[exclude-filter] show: unterminated quote in argument"
        );
    }

    #[test]
    fn invalid_argument_error_formats_correctly() {
        let err = ExcludeFilterError::InvalidArgument {
            command: "exclude".to_string(),
            detail: "expected text argument".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[exclude-filter] exclude: invalid arguments: expected text argument"
        );
    }

    #[test]
    fn line_out_of_range_error_formats_correctly() {
        let err = ExcludeFilterError::LineOutOfRange {
            operation: "exclude_range".to_string(),
            line: 200,
            total: 100,
        };
        assert!(err.to_string().contains("line 200 is out of range"));
    }
}
