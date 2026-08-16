//! Shell-capture mode — document-insertion contract for captured shell output.
//!
//! This module defines the insertion mechanics for shell command output.
//! Actual shell execution is owned by the `shell-command` sub-project.

use crate::error::ClipboardError;
use crate::router::TargetPosition;
use crate::splitter::LineSplitter;

/// Result of shell command execution (provided by the shell-command subsystem).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCaptureResult {
    /// Lines of stdout captured from the shell command.
    pub stdout_lines: Vec<String>,
    /// Total number of lines captured.
    pub line_count: usize,
}

/// Result of preparing shell capture output for document insertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellInsertResult {
    /// Lines to insert into the document.
    pub lines: Vec<String>,
    /// Number of lines to insert.
    pub lines_inserted: usize,
    /// Target position for insertion.
    pub target_position: TargetPosition,
}

/// Prepares shell capture output for insertion into the document.
///
/// Follows the same line-splitting and insertion rules as COPY clipboard-paste mode.
pub struct ShellCaptureHandler;

impl ShellCaptureHandler {
    /// Prepare captured shell output for document insertion.
    ///
    /// If the capture result has pre-split lines, uses them directly.
    /// Otherwise, splits raw output text using the standard line splitter.
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::Empty`] if the captured output is empty
    /// (command produced no output).
    pub fn prepare_insert(
        capture: &ShellCaptureResult,
        target_position: TargetPosition,
    ) -> Result<ShellInsertResult, ClipboardError> {
        if capture.stdout_lines.is_empty() {
            return Err(ClipboardError::Empty);
        }

        Ok(ShellInsertResult {
            lines_inserted: capture.stdout_lines.len(),
            lines: capture.stdout_lines.clone(),
            target_position,
        })
    }

    /// Split raw stdout text into lines for insertion.
    ///
    /// Used when the shell-command subsystem provides raw text instead of
    /// pre-split lines.
    pub fn split_output(raw_output: &str) -> ShellCaptureResult {
        let split = LineSplitter::split(raw_output);
        ShellCaptureResult {
            line_count: split.lines.len(),
            stdout_lines: split.lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_insert_with_lines() {
        // Validates: Requirement 11.1, 11.2
        let capture = ShellCaptureResult {
            stdout_lines: vec!["output1".to_string(), "output2".to_string()],
            line_count: 2,
        };
        let result = ShellCaptureHandler::prepare_insert(&capture, TargetPosition::After).unwrap();
        assert_eq!(result.lines, vec!["output1", "output2"]);
        assert_eq!(result.lines_inserted, 2);
        assert_eq!(result.target_position, TargetPosition::After);
    }

    #[test]
    fn prepare_insert_empty_output_returns_error() {
        // Validates: Requirement 11.5
        let capture = ShellCaptureResult {
            stdout_lines: vec![],
            line_count: 0,
        };
        let result = ShellCaptureHandler::prepare_insert(&capture, TargetPosition::Before);
        assert!(matches!(result, Err(ClipboardError::Empty)));
    }

    #[test]
    fn split_output_handles_mixed_line_endings() {
        let result = ShellCaptureHandler::split_output("line1\nline2\r\nline3\r");
        assert_eq!(result.stdout_lines, vec!["line1", "line2", "line3"]);
        assert_eq!(result.line_count, 3);
    }

    #[test]
    fn prepare_insert_before_target() {
        // Validates: Requirement 11.3
        let capture = ShellCaptureResult {
            stdout_lines: vec!["data".to_string()],
            line_count: 1,
        };
        let result = ShellCaptureHandler::prepare_insert(&capture, TargetPosition::Before).unwrap();
        assert_eq!(result.target_position, TargetPosition::Before);
    }
}
