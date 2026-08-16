//! COPY command router — disambiguation logic for the COPY primary command.
//!
//! Determines which of four modes the COPY command operates in:
//! in-document, clipboard-paste, file-insert, or shell-capture.

use crate::error::ClipboardError;

/// The resolved mode of the COPY primary command after disambiguation.
///
/// The router examines pending line commands (C/CC sources, A/B targets)
/// and command arguments to determine the correct execution mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyCommandMode {
    /// In-document copy: C/CC source + A/B target (delegates to ff-line-commands).
    InDocument,
    /// Clipboard paste: no args, no source, A/B target present.
    ClipboardPaste,
    /// File insert: path argument + A/B target, no source.
    FileInsert {
        /// The file path argument provided with the COPY command.
        path: String,
    },
    /// Shell capture: SHELL command + A/B target (delegates to ff-shell-command).
    ShellCapture {
        /// The shell command to execute.
        command: String,
    },
}

/// Whether insertion is before or after the target line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPosition {
    /// Insert after the target line.
    After,
    /// Insert before the target line.
    Before,
}

/// Disambiguates the COPY primary command into its four modes.
///
/// This is pure logic with no I/O — actual execution is delegated to the
/// appropriate handler based on the resolved mode.
pub struct CopyCommandRouter;

impl CopyCommandRouter {
    /// Given the command arguments and current pending line-command state,
    /// determine which COPY mode should execute.
    ///
    /// # Arguments
    ///
    /// * `args` — The argument string following the COPY command (may be empty or a file path)
    /// * `has_pending_source` — Whether C/CC source line commands are pending
    /// * `has_target` — Whether an A/B target line command is present
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::NoTarget`] if no target is present and no source is pending.
    /// Returns [`ClipboardError::ConflictingSourceAndPath`] if C/CC source + path arg.
    /// Returns [`ClipboardError::IncompleteSourceTarget`] if C/CC source but no target.
    pub fn resolve(
        args: &str,
        has_pending_source: bool,
        has_target: bool,
    ) -> Result<CopyCommandMode, ClipboardError> {
        let trimmed_args = args.trim();
        let has_path_arg = !trimmed_args.is_empty();

        match (has_pending_source, has_target, has_path_arg) {
            // Rule 8.1/8.3: pending C/CC + A/B → InDocument (route to line-commands)
            (true, true, false) => Ok(CopyCommandMode::InDocument),

            // Rule 8.7: pending C/CC + path arg → error (conflicting commands)
            (true, _, true) => Err(ClipboardError::ConflictingSourceAndPath),

            // Rule 8.8: pending C/CC + no target + no args → incomplete
            (true, false, false) => Err(ClipboardError::IncompleteSourceTarget),

            // Rule 8.5/8.6: no pending C/CC + path arg + A/B → FileInsert
            (false, true, true) => {
                let path = Self::parse_path(trimmed_args);
                Ok(CopyCommandMode::FileInsert { path })
            }

            // Rule 8.4: no pending C/CC + no args + A/B → ClipboardPaste
            (false, true, false) => Ok(CopyCommandMode::ClipboardPaste),

            // Rule 8.2: no pending C/CC + no target + no args → error (target required)
            (false, false, false) => Err(ClipboardError::NoTarget),

            // No pending C/CC + path arg but no target → error (target required)
            (false, false, true) => Err(ClipboardError::NoTarget),
        }
    }

    /// Parse a path argument, stripping surrounding double quotes if present.
    fn parse_path(args: &str) -> String {
        let trimmed = args.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            trimmed.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_source_with_target_routes_to_in_document() {
        // Validates: Requirement 8.1, 8.3
        let result = CopyCommandRouter::resolve("", true, true);
        assert_eq!(result.unwrap(), CopyCommandMode::InDocument);
    }

    #[test]
    fn pending_source_with_path_arg_returns_conflicting_error() {
        // Validates: Requirement 8.7
        let result = CopyCommandRouter::resolve("file.txt", true, true);
        assert!(matches!(
            result,
            Err(ClipboardError::ConflictingSourceAndPath)
        ));

        let result = CopyCommandRouter::resolve("file.txt", true, false);
        assert!(matches!(
            result,
            Err(ClipboardError::ConflictingSourceAndPath)
        ));
    }

    #[test]
    fn pending_source_no_target_no_args_returns_incomplete() {
        // Validates: Requirement 8.8
        let result = CopyCommandRouter::resolve("", true, false);
        assert!(matches!(
            result,
            Err(ClipboardError::IncompleteSourceTarget)
        ));
    }

    #[test]
    fn no_source_with_path_and_target_routes_to_file_insert() {
        // Validates: Requirement 8.5, 8.6
        let result = CopyCommandRouter::resolve("path/to/file.txt", false, true);
        assert_eq!(
            result.unwrap(),
            CopyCommandMode::FileInsert {
                path: "path/to/file.txt".to_string()
            }
        );
    }

    #[test]
    fn no_source_no_args_with_target_routes_to_clipboard_paste() {
        // Validates: Requirement 8.4
        let result = CopyCommandRouter::resolve("", false, true);
        assert_eq!(result.unwrap(), CopyCommandMode::ClipboardPaste);
    }

    #[test]
    fn no_source_no_target_no_args_returns_no_target_error() {
        // Validates: Requirement 8.2
        let result = CopyCommandRouter::resolve("", false, false);
        assert!(matches!(result, Err(ClipboardError::NoTarget)));
    }

    #[test]
    fn no_source_with_path_but_no_target_returns_no_target_error() {
        let result = CopyCommandRouter::resolve("file.txt", false, false);
        assert!(matches!(result, Err(ClipboardError::NoTarget)));
    }

    #[test]
    fn quoted_path_is_parsed_without_quotes() {
        // Validates: Requirement 9.6
        let result = CopyCommandRouter::resolve("\"path with spaces/file.txt\"", false, true);
        assert_eq!(
            result.unwrap(),
            CopyCommandMode::FileInsert {
                path: "path with spaces/file.txt".to_string()
            }
        );
    }

    #[test]
    fn whitespace_only_args_treated_as_no_args() {
        let result = CopyCommandRouter::resolve("   ", false, true);
        assert_eq!(result.unwrap(), CopyCommandMode::ClipboardPaste);
    }

    #[test]
    fn file_insert_takes_precedence_over_clipboard_paste_when_path_present() {
        // Validates: Requirement 8.6
        let result = CopyCommandRouter::resolve("some_file", false, true);
        assert!(matches!(result, Ok(CopyCommandMode::FileInsert { .. })));
    }
}
