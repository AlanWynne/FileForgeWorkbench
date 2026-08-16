//! Document capture mode handler.
//!
//! Captures stdout from command execution and inserts it into the document
//! at the specified A/B target position.

use crate::error::ShellError;
use crate::process::ExitStatus;

/// Specifies where captured output should be inserted in the document.
#[derive(Debug, Clone)]
pub struct CaptureTarget {
    /// The target line number (0-indexed).
    pub line: usize,
    /// Whether to insert after (A) or before (B) the target line.
    pub position: CapturePosition,
}

/// Insert position relative to the target line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapturePosition {
    /// Insert after the target line (A line command).
    After,
    /// Insert before the target line (B line command).
    Before,
}

/// Result of a document capture operation.
#[derive(Debug)]
pub struct CaptureResult {
    /// Number of lines inserted into the document.
    pub lines_inserted: usize,
    /// Exit status of the command.
    pub exit_status: ExitStatus,
    /// Stderr output (displayed separately, not inserted).
    pub stderr: Vec<String>,
    /// The lines that were inserted (for undo support).
    pub inserted_lines: Vec<String>,
    /// The insertion position (0-indexed line in document).
    pub insertion_position: usize,
}

/// Handles document capture mode: splitting output into lines and computing
/// insertion position.
#[derive(Debug)]
pub struct CaptureHandler;

impl CaptureHandler {
    /// Splits raw stdout output into logical lines for document insertion.
    ///
    /// Handles all line ending types (LF, CRLF, CR). A trailing line ending
    /// does NOT produce an extra empty line (Requirement 5.5).
    /// Content is preserved exactly without trimming (Requirement 5.6).
    pub fn split_output_lines(stdout: &str) -> Vec<String> {
        if stdout.is_empty() {
            return Vec::new();
        }

        // Normalize and split on any line ending: CRLF, CR, LF
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut chars = stdout.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\r' => {
                    // CR or CRLF
                    lines.push(std::mem::take(&mut current_line));
                    if chars.peek() == Some(&'\n') {
                        chars.next(); // consume the LF in CRLF
                    }
                }
                '\n' => {
                    // LF
                    lines.push(std::mem::take(&mut current_line));
                }
                _ => {
                    current_line.push(ch);
                }
            }
        }

        // If there's remaining content (no trailing newline), include it
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // If the output ended with a newline, we already pushed the last line
        // but don't add an extra empty line for the trailing terminator

        lines
    }

    /// Computes the actual insertion position in the document.
    ///
    /// For `After` (A target): inserts at `target.line + 1`.
    /// For `Before` (B target): inserts at `target.line`.
    pub fn compute_insertion_position(target: &CaptureTarget) -> usize {
        match target.position {
            CapturePosition::After => target.line + 1,
            CapturePosition::Before => target.line,
        }
    }

    /// Performs the capture operation: validates exit code, splits output,
    /// computes position, and returns the capture result.
    ///
    /// # Errors
    ///
    /// Returns `ShellError::CaptureExitError` if the command exited with
    /// a non-zero exit code (Requirement 8.4).
    pub fn process_capture(
        stdout: &str,
        stderr_lines: Vec<String>,
        exit_status: ExitStatus,
        target: &CaptureTarget,
    ) -> Result<CaptureResult, ShellError> {
        // Reject non-zero exit code (Requirement 8.4)
        if let Some(code) = exit_status.code {
            if code != 0 {
                return Err(ShellError::CaptureExitError {
                    code,
                    stderr: stderr_lines,
                });
            }
        } else if exit_status.force_killed || exit_status.signal.is_some() {
            return Err(ShellError::CaptureExitError {
                code: -1,
                stderr: stderr_lines,
            });
        }

        let lines = Self::split_output_lines(stdout);
        let insertion_position = Self::compute_insertion_position(target);

        Ok(CaptureResult {
            lines_inserted: lines.len(),
            exit_status,
            stderr: stderr_lines,
            inserted_lines: lines,
            insertion_position,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 5.4
    #[test]
    fn split_output_lines_handles_lf() {
        let lines = CaptureHandler::split_output_lines("line1\nline2\nline3");
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    // Validates: Requirement 5.4
    #[test]
    fn split_output_lines_handles_crlf() {
        let lines = CaptureHandler::split_output_lines("line1\r\nline2\r\nline3");
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    // Validates: Requirement 5.4
    #[test]
    fn split_output_lines_handles_cr() {
        let lines = CaptureHandler::split_output_lines("line1\rline2\rline3");
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    // Validates: Requirement 5.5
    #[test]
    fn split_output_lines_trailing_lf_no_extra_empty_line() {
        let lines = CaptureHandler::split_output_lines("line1\nline2\n");
        assert_eq!(lines, vec!["line1", "line2"]);
    }

    // Validates: Requirement 5.5
    #[test]
    fn split_output_lines_trailing_crlf_no_extra_empty_line() {
        let lines = CaptureHandler::split_output_lines("line1\r\nline2\r\n");
        assert_eq!(lines, vec!["line1", "line2"]);
    }

    // Validates: Requirement 5.6
    #[test]
    fn split_output_lines_preserves_whitespace() {
        let lines = CaptureHandler::split_output_lines("  indented\n\ttabbed\n  spaces  ");
        assert_eq!(lines, vec!["  indented", "\ttabbed", "  spaces  "]);
    }

    // Validates: Requirement 5.9
    #[test]
    fn split_output_lines_empty_input_returns_empty_vec() {
        let lines = CaptureHandler::split_output_lines("");
        assert!(lines.is_empty());
    }

    // Validates: Requirement 5.4
    #[test]
    fn split_output_lines_mixed_endings() {
        let lines = CaptureHandler::split_output_lines("a\nb\r\nc\rd");
        assert_eq!(lines, vec!["a", "b", "c", "d"]);
    }

    // Validates: Requirement 5.2
    #[test]
    fn compute_insertion_position_after() {
        let target = CaptureTarget {
            line: 5,
            position: CapturePosition::After,
        };
        assert_eq!(CaptureHandler::compute_insertion_position(&target), 6);
    }

    // Validates: Requirement 5.3
    #[test]
    fn compute_insertion_position_before() {
        let target = CaptureTarget {
            line: 5,
            position: CapturePosition::Before,
        };
        assert_eq!(CaptureHandler::compute_insertion_position(&target), 5);
    }

    // Validates: Requirement 8.4
    #[test]
    fn process_capture_rejects_non_zero_exit() {
        let target = CaptureTarget {
            line: 0,
            position: CapturePosition::After,
        };
        let exit_status = ExitStatus::from_code(1);
        let result = CaptureHandler::process_capture(
            "output",
            vec!["error".to_string()],
            exit_status,
            &target,
        );
        assert!(matches!(
            result,
            Err(ShellError::CaptureExitError { code: 1, .. })
        ));
    }

    // Validates: Requirement 5.1
    #[test]
    fn process_capture_returns_stdout_lines_only() {
        let target = CaptureTarget {
            line: 0,
            position: CapturePosition::After,
        };
        let exit_status = ExitStatus::from_code(0);
        let result = CaptureHandler::process_capture(
            "line1\nline2\n",
            vec!["stderr_ignored".to_string()],
            exit_status,
            &target,
        );
        let capture = result.unwrap();
        assert_eq!(capture.lines_inserted, 2);
        assert_eq!(capture.inserted_lines, vec!["line1", "line2"]);
        assert_eq!(capture.stderr, vec!["stderr_ignored"]);
    }

    // Validates: Requirement 8.5
    #[test]
    fn process_capture_rejects_force_killed() {
        let target = CaptureTarget {
            line: 0,
            position: CapturePosition::After,
        };
        let exit_status = ExitStatus::force_killed();
        let result = CaptureHandler::process_capture("partial", Vec::new(), exit_status, &target);
        assert!(result.is_err());
    }
}
