//! Line splitter — splits text into logical lines handling all standard
//! line-ending conventions (LF, CRLF, CR).
//!
//! The [`LineSplitter`] is used by clipboard paste, file-insert, and shell-capture
//! operations to convert text into individual logical lines for document insertion.

/// Result of splitting text into logical lines.
///
/// Contains the individual lines (content preserved without trimming) and
/// whether the source text ended with a trailing line terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineSplitResult {
    /// Individual logical lines after splitting. Content is preserved exactly.
    pub lines: Vec<String>,
    /// Whether the source text ended with a trailing line terminator
    /// (used to suppress empty trailing line creation).
    pub had_trailing_terminator: bool,
}

/// Line ending style for normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Unix-style line ending: `\n`
    Lf,
    /// Windows-style line ending: `\r\n`
    CrLf,
    /// Classic Mac-style line ending: `\r`
    Cr,
}

impl LineEnding {
    /// The string representation of this line ending.
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }
}

/// Splits text into logical lines handling all standard line-ending conventions.
///
/// The splitter handles LF (`\n`), CRLF (`\r\n`), and CR (`\r`) line endings,
/// including mixed line endings within a single text. A trailing line terminator
/// does NOT produce an empty final line.
pub struct LineSplitter;

impl LineSplitter {
    /// Split text on LF, CRLF, or CR boundaries.
    ///
    /// A trailing line terminator does NOT produce an empty final line.
    /// Content of each line is preserved without trimming.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_clipboard::splitter::{LineSplitter, LineSplitResult};
    ///
    /// let result = LineSplitter::split("hello\nworld\n");
    /// assert_eq!(result.lines, vec!["hello", "world"]);
    /// assert!(result.had_trailing_terminator);
    ///
    /// let result = LineSplitter::split("hello\nworld");
    /// assert_eq!(result.lines, vec!["hello", "world"]);
    /// assert!(!result.had_trailing_terminator);
    /// ```
    pub fn split(text: &str) -> LineSplitResult {
        if text.is_empty() {
            return LineSplitResult {
                lines: vec![String::new()],
                had_trailing_terminator: false,
            };
        }

        let mut lines = Vec::new();
        let mut current_start = 0;
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            if bytes[i] == b'\r' {
                // Check for CRLF
                let line_end = i;
                if i + 1 < len && bytes[i + 1] == b'\n' {
                    i += 2; // skip \r\n
                } else {
                    i += 1; // skip \r only
                }
                lines.push(text[current_start..line_end].to_string());
                current_start = i;
            } else if bytes[i] == b'\n' {
                lines.push(text[current_start..i].to_string());
                i += 1;
                current_start = i;
            } else {
                i += 1;
            }
        }

        // Handle remaining text after last line ending
        let had_trailing_terminator = current_start == len && !lines.is_empty();

        if current_start < len {
            // There's text after the last line ending (no trailing terminator)
            lines.push(text[current_start..].to_string());
        }

        LineSplitResult {
            lines,
            had_trailing_terminator,
        }
    }

    /// Split and normalize line endings to the given target style.
    ///
    /// Each line's content is preserved; only the line-ending convention is
    /// normalized. The returned lines do NOT include the line endings themselves.
    pub fn split_and_normalize(text: &str, _target_ending: LineEnding) -> LineSplitResult {
        // Splitting already strips line endings, so normalization is implicit.
        // The caller can join with the target ending when inserting.
        Self::split(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_empty_string_returns_single_empty_line() {
        let result = LineSplitter::split("");
        assert_eq!(result.lines, vec![""]);
        assert!(!result.had_trailing_terminator);
    }

    #[test]
    fn split_single_line_no_terminator() {
        // Validates: Requirement 4.8 (whitespace preservation)
        let result = LineSplitter::split("hello world");
        assert_eq!(result.lines, vec!["hello world"]);
        assert!(!result.had_trailing_terminator);
    }

    #[test]
    fn split_lf_terminated_lines() {
        // Validates: Requirement 4.6, 16.1
        let result = LineSplitter::split("line1\nline2\nline3");
        assert_eq!(result.lines, vec!["line1", "line2", "line3"]);
        assert!(!result.had_trailing_terminator);
    }

    #[test]
    fn split_crlf_terminated_lines() {
        // Validates: Requirement 4.6, 16.1
        let result = LineSplitter::split("line1\r\nline2\r\nline3");
        assert_eq!(result.lines, vec!["line1", "line2", "line3"]);
        assert!(!result.had_trailing_terminator);
    }

    #[test]
    fn split_cr_terminated_lines() {
        // Validates: Requirement 4.6, 16.1
        let result = LineSplitter::split("line1\rline2\rline3");
        assert_eq!(result.lines, vec!["line1", "line2", "line3"]);
        assert!(!result.had_trailing_terminator);
    }

    #[test]
    fn split_trailing_lf_does_not_produce_empty_line() {
        // Validates: Requirement 4.7, 16.3
        let result = LineSplitter::split("line1\nline2\n");
        assert_eq!(result.lines, vec!["line1", "line2"]);
        assert!(result.had_trailing_terminator);
    }

    #[test]
    fn split_trailing_crlf_does_not_produce_empty_line() {
        // Validates: Requirement 4.7, 16.3
        let result = LineSplitter::split("line1\r\nline2\r\n");
        assert_eq!(result.lines, vec!["line1", "line2"]);
        assert!(result.had_trailing_terminator);
    }

    #[test]
    fn split_trailing_cr_does_not_produce_empty_line() {
        // Validates: Requirement 4.7, 16.3
        let result = LineSplitter::split("line1\rline2\r");
        assert_eq!(result.lines, vec!["line1", "line2"]);
        assert!(result.had_trailing_terminator);
    }

    #[test]
    fn split_preserves_whitespace_exactly() {
        // Validates: Requirement 4.8, 9.10
        let result = LineSplitter::split("  indented\n\ttabbed\n  spaces  ");
        assert_eq!(result.lines, vec!["  indented", "\ttabbed", "  spaces  "]);
    }

    #[test]
    fn split_mixed_line_endings() {
        // Validates: Requirement 16.2
        let result = LineSplitter::split("lf\ncrlf\r\ncr\r");
        assert_eq!(result.lines, vec!["lf", "crlf", "cr"]);
        assert!(result.had_trailing_terminator);
    }

    #[test]
    fn split_empty_lines_in_middle() {
        let result = LineSplitter::split("a\n\nb\n\nc");
        assert_eq!(result.lines, vec!["a", "", "b", "", "c"]);
        assert!(!result.had_trailing_terminator);
    }

    #[test]
    fn split_only_newline() {
        // Single \n should produce one empty line with trailing terminator
        let result = LineSplitter::split("\n");
        assert_eq!(result.lines, vec![""]);
        assert!(result.had_trailing_terminator);
    }

    #[test]
    fn split_only_crlf() {
        let result = LineSplitter::split("\r\n");
        assert_eq!(result.lines, vec![""]);
        assert!(result.had_trailing_terminator);
    }

    #[test]
    fn split_and_normalize_delegates_to_split() {
        // Validates: Requirement 16.5
        let result = LineSplitter::split_and_normalize("a\r\nb\nc\r", LineEnding::Lf);
        assert_eq!(result.lines, vec!["a", "b", "c"]);
        assert!(result.had_trailing_terminator);
    }

    #[test]
    fn line_ending_as_str() {
        assert_eq!(LineEnding::Lf.as_str(), "\n");
        assert_eq!(LineEnding::CrLf.as_str(), "\r\n");
        assert_eq!(LineEnding::Cr.as_str(), "\r");
    }
}
