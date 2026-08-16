//! Output Panel — dockable panel for command output with scrollback history.
//!
//! Displays command execution results (stdout/stderr) with timestamps,
//! exit codes, and navigable file references.

use std::path::PathBuf;

use ff_layout::{DockState, DockZone, DockablePanel};

use crate::process::ExitStatus;

/// Classification of an output line's source stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
    /// Internal system messages (timeout, cancellation, etc.).
    System,
}

/// A single line of output with stream classification.
#[derive(Debug, Clone)]
pub struct OutputLine {
    /// The text content of this line.
    pub text: String,
    /// Which stream this line came from.
    pub stream: OutputStream,
}

/// A single command execution entry in the Output Panel scrollback.
#[derive(Debug, Clone)]
pub struct OutputEntry {
    /// The command text that was executed.
    pub command: String,
    /// Working directory at execution time.
    pub working_directory: PathBuf,
    /// Timestamp when execution started.
    pub timestamp: chrono::DateTime<chrono::Local>,
    /// Combined output lines (stdout + stderr interleaved).
    pub lines: Vec<OutputLine>,
    /// Exit status of the command.
    pub exit_status: Option<ExitStatus>,
}

impl OutputEntry {
    /// Returns the total number of output lines in this entry.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

/// A navigable file reference found in output text.
#[derive(Debug, Clone, PartialEq)]
pub struct FileReference {
    /// The file path.
    pub path: String,
    /// The line number (1-indexed).
    pub line: usize,
}

/// Output Panel — displays command output history with scrollback.
///
/// Registered as a `DockablePanel` with ID `"shell.output"` in the
/// `ff-layout` system, defaulting to the Bottom dock zone.
#[derive(Debug)]
pub struct OutputPanel {
    /// All output entries (command history).
    entries: Vec<OutputEntry>,
    /// Maximum total lines in the scrollback buffer.
    max_buffer_lines: usize,
    /// Current total line count across all entries.
    total_lines: usize,
}

impl OutputPanel {
    /// Creates a new output panel with the given buffer limit.
    pub fn new(max_buffer_lines: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_buffer_lines,
            total_lines: 0,
        }
    }

    /// Appends a new command entry to the output history.
    pub fn append_entry(&mut self, entry: OutputEntry) {
        self.total_lines += entry.line_count() + 1; // +1 for header
        self.entries.push(entry);
        self.enforce_limit();
    }

    /// Appends a line to the most recent entry (streaming).
    pub fn append_line(&mut self, line: OutputLine) {
        if let Some(entry) = self.entries.last_mut() {
            entry.lines.push(line);
            self.total_lines += 1;
            self.enforce_limit();
        }
    }

    /// Clears the entire scrollback buffer.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_lines = 0;
    }

    /// Returns the total number of lines in the scrollback.
    pub fn line_count(&self) -> usize {
        self.total_lines
    }

    /// Returns all entries in the scrollback.
    pub fn entries(&self) -> &[OutputEntry] {
        &self.entries
    }

    /// Returns the number of command entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Extracts navigable file references from all output text.
    ///
    /// Matches patterns like `path:line` and `path(line)`.
    pub fn file_references(&self) -> Vec<FileReference> {
        let mut refs = Vec::new();
        for entry in &self.entries {
            for line in &entry.lines {
                refs.extend(Self::parse_file_references(&line.text));
            }
        }
        refs
    }

    /// Parses file references from a single line of text.
    fn parse_file_references(text: &str) -> Vec<FileReference> {
        let mut refs = Vec::new();

        // Pattern: path:line (e.g., "src/main.rs:42")
        for (i, _) in text.match_indices(':') {
            let path_part = &text[..i];
            let rest = &text[i + 1..];

            // Extract line number
            let line_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(line) = line_str.parse::<usize>() {
                if line > 0 && !path_part.is_empty() && Self::looks_like_path(path_part) {
                    refs.push(FileReference {
                        path: path_part.to_string(),
                        line,
                    });
                }
            }
        }

        // Pattern: path(line) (e.g., "src/main.rs(42)")
        for (i, _) in text.match_indices('(') {
            let path_part = &text[..i];
            let rest = &text[i + 1..];

            let line_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if rest.get(line_str.len()..line_str.len() + 1) == Some(")") {
                if let Ok(line) = line_str.parse::<usize>() {
                    if line > 0 && !path_part.is_empty() && Self::looks_like_path(path_part) {
                        refs.push(FileReference {
                            path: path_part.to_string(),
                            line,
                        });
                    }
                }
            }
        }

        refs
    }

    /// Heuristic: does this string look like a file path?
    fn looks_like_path(s: &str) -> bool {
        let trimmed = s.trim();
        // Must contain a dot (extension) or path separator
        trimmed.contains('.') || trimmed.contains('/') || trimmed.contains('\\')
    }

    /// Enforces the maximum buffer line limit by removing oldest entries.
    fn enforce_limit(&mut self) {
        while self.total_lines > self.max_buffer_lines && !self.entries.is_empty() {
            let removed = self.entries.remove(0);
            self.total_lines = self.total_lines.saturating_sub(removed.line_count() + 1);
        }
    }
}

impl DockablePanel for OutputPanel {
    fn panel_id(&self) -> &str {
        "shell.output"
    }

    fn default_dock_zone(&self) -> DockZone {
        DockZone::Bottom
    }

    fn title(&self) -> &str {
        "Output"
    }

    fn on_dock_state_changed(&mut self, _state: DockState) {
        // No special handling needed for dock state transitions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn test_entry(command: &str, lines: Vec<&str>) -> OutputEntry {
        OutputEntry {
            command: command.to_string(),
            working_directory: PathBuf::from("/tmp"),
            timestamp: Local::now(),
            lines: lines
                .into_iter()
                .map(|l| OutputLine {
                    text: l.to_string(),
                    stream: OutputStream::Stdout,
                })
                .collect(),
            exit_status: Some(ExitStatus::from_code(0)),
        }
    }

    // Validates: Requirement 15.1
    #[test]
    fn output_panel_has_correct_panel_id() {
        let panel = OutputPanel::new(10000);
        assert_eq!(panel.panel_id(), "shell.output");
        assert_eq!(panel.default_dock_zone(), DockZone::Bottom);
    }

    // Validates: Requirement 15.2
    #[test]
    fn append_entry_adds_to_history() {
        let mut panel = OutputPanel::new(10000);
        panel.append_entry(test_entry("ls", vec!["file1", "file2"]));
        assert_eq!(panel.entry_count(), 1);
        assert_eq!(panel.line_count(), 3); // 2 lines + 1 header
    }

    // Validates: Requirement 15.3
    #[test]
    fn overflow_trims_oldest_entries() {
        let mut panel = OutputPanel::new(5); // very small buffer
        panel.append_entry(test_entry("cmd1", vec!["a", "b", "c"])); // 4 lines (3 + header)
        panel.append_entry(test_entry("cmd2", vec!["d", "e", "f"])); // 4 lines

        // Should have trimmed to stay under 5
        assert!(panel.line_count() <= 5);
    }

    // Validates: Requirement 15.6
    #[test]
    fn clear_empties_buffer() {
        let mut panel = OutputPanel::new(10000);
        panel.append_entry(test_entry("ls", vec!["file1"]));
        panel.clear();
        assert_eq!(panel.entry_count(), 0);
        assert_eq!(panel.line_count(), 0);
    }

    // Validates: Requirement 15.7
    #[test]
    fn file_reference_parsing_colon_format() {
        let refs = OutputPanel::parse_file_references("src/main.rs:42: error");
        assert!(refs.iter().any(|r| r.path == "src/main.rs" && r.line == 42));
    }

    // Validates: Requirement 15.7
    #[test]
    fn file_reference_parsing_paren_format() {
        let refs = OutputPanel::parse_file_references("src/main.rs(42) error");
        assert!(refs.iter().any(|r| r.path == "src/main.rs" && r.line == 42));
    }

    // Validates: Requirement 15.7
    #[test]
    fn file_reference_parsing_no_match() {
        let refs = OutputPanel::parse_file_references("just plain text");
        assert!(refs.is_empty());
    }

    // Validates: Requirement 13.6
    #[test]
    fn append_line_to_current_entry() {
        let mut panel = OutputPanel::new(10000);
        panel.append_entry(test_entry("ls", vec![]));
        panel.append_line(OutputLine {
            text: "streaming line".to_string(),
            stream: OutputStream::Stdout,
        });
        assert_eq!(panel.entries().last().unwrap().lines.len(), 1);
    }
}
