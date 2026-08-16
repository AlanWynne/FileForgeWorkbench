//! File-insert handler — reads a file via VFS and inserts content at a target line.
//!
//! Handles path resolution (relative/absolute), quoted-path parsing, binary file
//! detection, and maps VFS errors to [`ClipboardError`] variants.

use crate::error::ClipboardError;
use crate::router::TargetPosition;
use crate::splitter::LineSplitter;

/// Result of a file-insert operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInsertResult {
    /// Number of logical lines inserted.
    pub lines_inserted: usize,
    /// The resolved absolute path of the file that was read.
    pub resolved_path: String,
    /// The lines that were split from the file content.
    pub lines: Vec<String>,
    /// Target position (before or after).
    pub target_position: TargetPosition,
}

/// Reads a file and prepares its content for insertion at a target line.
///
/// Handles path resolution, quoted-path stripping, and binary-file detection.
/// The actual document insertion is performed by the caller.
pub struct FileInsertHandler;

impl FileInsertHandler {
    /// Parse a file path argument, stripping surrounding double quotes.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_clipboard::file_insert::FileInsertHandler;
    ///
    /// assert_eq!(FileInsertHandler::parse_path("file.txt"), "file.txt");
    /// assert_eq!(FileInsertHandler::parse_path("\"path with spaces.txt\""), "path with spaces.txt");
    /// ```
    pub fn parse_path(path: &str) -> &str {
        let trimmed = path.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        }
    }

    /// Check if content appears to be binary (contains null bytes).
    ///
    /// Returns `true` if the content contains null bytes, indicating it is
    /// likely binary/non-text content.
    pub fn is_binary(content: &[u8]) -> bool {
        content.contains(&0)
    }

    /// Prepare file content for insertion by splitting into logical lines.
    ///
    /// Applies the same line-splitting rules as clipboard paste:
    /// - Splits on LF, CRLF, or CR
    /// - Trailing line terminator does not produce empty final line
    /// - Whitespace is preserved exactly
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::FileBinary`] if the content contains null bytes.
    pub fn prepare_content(
        content: &str,
        resolved_path: &str,
        target_position: TargetPosition,
    ) -> Result<FileInsertResult, ClipboardError> {
        // Check for binary content (null bytes in the string representation
        // shouldn't normally occur in valid UTF-8 text files, but check anyway)
        if content.as_bytes().contains(&0) {
            return Err(ClipboardError::FileBinary {
                path: resolved_path.to_string(),
            });
        }

        let split = LineSplitter::split(content);

        Ok(FileInsertResult {
            lines_inserted: split.lines.len(),
            resolved_path: resolved_path.to_string(),
            lines: split.lines,
            target_position,
        })
    }

    /// Resolve a relative path against a base directory.
    ///
    /// If the path is absolute, returns it as-is.
    /// If relative, joins it with the base directory.
    pub fn resolve_path(path: &str, base_dir: Option<&str>) -> String {
        let path = std::path::Path::new(path);
        if path.is_absolute() {
            return path.to_string_lossy().to_string();
        }

        match base_dir {
            Some(base) => {
                let base_path = std::path::Path::new(base);
                base_path.join(path).to_string_lossy().to_string()
            }
            None => path.to_string_lossy().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_path_strips_quotes() {
        // Validates: Requirement 9.6
        assert_eq!(
            FileInsertHandler::parse_path("\"path with spaces.txt\""),
            "path with spaces.txt"
        );
    }

    #[test]
    fn parse_path_leaves_unquoted_unchanged() {
        assert_eq!(FileInsertHandler::parse_path("simple.txt"), "simple.txt");
    }

    #[test]
    fn parse_path_trims_whitespace() {
        assert_eq!(FileInsertHandler::parse_path("  file.txt  "), "file.txt");
    }

    #[test]
    fn is_binary_detects_null_bytes() {
        // Validates: Requirement 10.3
        assert!(FileInsertHandler::is_binary(b"hello\x00world"));
        assert!(!FileInsertHandler::is_binary(b"hello world"));
    }

    #[test]
    fn prepare_content_splits_lines_correctly() {
        // Validates: Requirement 9.9, 9.10
        let result = FileInsertHandler::prepare_content(
            "line1\nline2\nline3\n",
            "/path",
            TargetPosition::After,
        )
        .unwrap();
        assert_eq!(result.lines, vec!["line1", "line2", "line3"]);
        assert_eq!(result.lines_inserted, 3);
        assert_eq!(result.resolved_path, "/path");
    }

    #[test]
    fn prepare_content_preserves_whitespace() {
        // Validates: Requirement 9.10
        let result = FileInsertHandler::prepare_content(
            "  indented\n\ttabbed",
            "/f",
            TargetPosition::Before,
        )
        .unwrap();
        assert_eq!(result.lines, vec!["  indented", "\ttabbed"]);
    }

    #[test]
    fn prepare_content_rejects_binary() {
        // Validates: Requirement 10.3
        let result = FileInsertHandler::prepare_content(
            "hello\x00world",
            "/binary.dat",
            TargetPosition::After,
        );
        assert!(matches!(result, Err(ClipboardError::FileBinary { .. })));
    }

    #[test]
    fn resolve_path_absolute_used_as_is() {
        // Validates: Requirement 9.5
        #[cfg(windows)]
        let result = FileInsertHandler::resolve_path("C:\\absolute\\path.txt", Some("C:\\base"));
        #[cfg(not(windows))]
        let result = FileInsertHandler::resolve_path("/absolute/path.txt", Some("/base"));

        #[cfg(windows)]
        assert_eq!(result, "C:\\absolute\\path.txt");
        #[cfg(not(windows))]
        assert_eq!(result, "/absolute/path.txt");
    }

    #[test]
    fn resolve_path_relative_joins_with_base() {
        // Validates: Requirement 9.4
        #[cfg(windows)]
        {
            let result = FileInsertHandler::resolve_path("sub\\file.txt", Some("C:\\project"));
            assert_eq!(result, "C:\\project\\sub\\file.txt");
        }
        #[cfg(not(windows))]
        {
            let result = FileInsertHandler::resolve_path("sub/file.txt", Some("/project"));
            assert_eq!(result, "/project/sub/file.txt");
        }
    }

    #[test]
    fn resolve_path_no_base_returns_relative_as_is() {
        let result = FileInsertHandler::resolve_path("file.txt", None);
        assert_eq!(result, "file.txt");
    }
}
