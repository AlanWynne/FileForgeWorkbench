//! Session state management — tracks pending line commands, last command,
//! cursor position, tags, and status messages per-document.

use std::collections::HashSet;

use crate::line_parser::LineCommandDescriptor;
use crate::scope::ResolvedScope;
use crate::status::StatusMessage;

/// A pending line command associated with a specific line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingLineCommand {
    /// The line number where this command was entered (0-based).
    pub line: u64,
    /// The parsed line command descriptor.
    pub descriptor: LineCommandDescriptor,
}

/// Per-document mutable state maintained by the command engine.
///
/// Tracks pending line commands, last command, tags, cursor, and status.
pub struct SessionState {
    /// Line commands awaiting execution.
    pending_line_commands: Vec<PendingLineCommand>,
    /// The last successfully executed command name (for repeat).
    last_command: Option<String>,
    /// The scope from the last execution (for RFIND/RCHANGE).
    last_scope: Option<ResolvedScope>,
    /// Per-line tag state (line numbers that are tagged).
    tagged_lines: HashSet<u64>,
    /// Current cursor line (0-based).
    cursor_line: u64,
    /// Current cursor column (0-based).
    cursor_column: u64,
    /// The last status message produced.
    last_status: Option<StatusMessage>,
}

impl SessionState {
    /// Create a new empty session state.
    pub fn new() -> Self {
        Self {
            pending_line_commands: Vec::new(),
            last_command: None,
            last_scope: None,
            tagged_lines: HashSet::new(),
            cursor_line: 0,
            cursor_column: 0,
            last_status: None,
        }
    }

    /// Add a pending line command.
    pub fn add_pending(&mut self, line: u64, descriptor: LineCommandDescriptor) {
        self.pending_line_commands
            .push(PendingLineCommand { line, descriptor });
    }

    /// Check if there are any pending line commands.
    pub fn has_pending(&self) -> bool {
        !self.pending_line_commands.is_empty()
    }

    /// Get a reference to pending line commands.
    pub fn pending(&self) -> &[PendingLineCommand] {
        &self.pending_line_commands
    }

    /// Drain and return all pending line commands, clearing them.
    pub fn take_pending(&mut self) -> Vec<PendingLineCommand> {
        std::mem::take(&mut self.pending_line_commands)
    }

    /// Clear consumed line commands (those at specific line numbers).
    pub fn clear_consumed(&mut self, consumed_lines: &[u64]) {
        self.pending_line_commands
            .retain(|cmd| !consumed_lines.contains(&cmd.line));
    }

    /// Retain all pending line commands (on failure — don't clear anything).
    pub fn retain_pending(&self) {
        // No-op: the commands remain in place.
        // This method documents the semantic intent.
    }

    /// Record a successful command execution.
    pub fn record_success(&mut self, command_name: String, scope: ResolvedScope) {
        self.last_command = Some(command_name);
        self.last_scope = Some(scope);
    }

    /// Get the last successfully executed command name.
    pub fn last_command(&self) -> Option<&str> {
        self.last_command.as_deref()
    }

    /// Get the last scope used.
    pub fn last_scope(&self) -> Option<&ResolvedScope> {
        self.last_scope.as_ref()
    }

    /// Tag a set of lines.
    pub fn tag_lines(&mut self, lines: impl IntoIterator<Item = u64>) {
        self.tagged_lines.extend(lines);
    }

    /// Clear all tags.
    pub fn clear_tags(&mut self) {
        self.tagged_lines.clear();
    }

    /// Check if a line is tagged.
    pub fn is_tagged(&self, line: u64) -> bool {
        self.tagged_lines.contains(&line)
    }

    /// Get all tagged lines.
    pub fn tagged_lines(&self) -> &HashSet<u64> {
        &self.tagged_lines
    }

    /// Update cursor position.
    pub fn set_cursor(&mut self, line: u64, column: u64) {
        self.cursor_line = line;
        self.cursor_column = column;
    }

    /// Get the current cursor line.
    pub fn cursor_line(&self) -> u64 {
        self.cursor_line
    }

    /// Get the current cursor column.
    pub fn cursor_column(&self) -> u64 {
        self.cursor_column
    }

    /// Set the last status message.
    pub fn set_status(&mut self, status: StatusMessage) {
        self.last_status = Some(status);
    }

    /// Get the last status message.
    pub fn last_status(&self) -> Option<&StatusMessage> {
        self.last_status.as_ref()
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line_parser::LineCommandKind;

    // Validates: Requirement 1.2
    #[test]
    fn new_session_has_no_pending_commands() {
        let session = SessionState::new();
        assert!(!session.has_pending());
        assert!(session.pending().is_empty());
    }

    // Validates: Requirement 1.2
    #[test]
    fn add_pending_increases_pending_count() {
        let mut session = SessionState::new();
        session.add_pending(
            0,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Copy,
                count: 1,
            },
        );
        assert!(session.has_pending());
        assert_eq!(session.pending().len(), 1);
    }

    // Validates: Requirement 1.5
    #[test]
    fn take_pending_clears_all_pending_commands() {
        let mut session = SessionState::new();
        session.add_pending(
            0,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Copy,
                count: 1,
            },
        );
        session.add_pending(
            5,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Delete,
                count: 3,
            },
        );

        let taken = session.take_pending();
        assert_eq!(taken.len(), 2);
        assert!(!session.has_pending());
    }

    // Validates: Requirement 1.5
    #[test]
    fn clear_consumed_removes_specific_lines() {
        let mut session = SessionState::new();
        session.add_pending(
            0,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Copy,
                count: 1,
            },
        );
        session.add_pending(
            5,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Delete,
                count: 3,
            },
        );
        session.add_pending(
            10,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Move,
                count: 1,
            },
        );

        session.clear_consumed(&[0, 10]);
        assert_eq!(session.pending().len(), 1);
        assert_eq!(session.pending()[0].line, 5);
    }

    // Validates: Requirement 1.6
    #[test]
    fn retain_pending_preserves_all_commands() {
        let mut session = SessionState::new();
        session.add_pending(
            0,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Copy,
                count: 1,
            },
        );
        session.retain_pending();
        assert!(session.has_pending());
        assert_eq!(session.pending().len(), 1);
    }

    #[test]
    fn record_success_stores_command_and_scope() {
        use crate::scope::{ScopeLines, ScopeSource};
        let mut session = SessionState::new();
        let scope = ResolvedScope {
            lines: ScopeLines::CursorLine(5),
            column_bounds: None,
            source: ScopeSource::CursorLine,
        };
        session.record_success("FIND".to_string(), scope);
        assert_eq!(session.last_command(), Some("FIND"));
        assert!(session.last_scope().is_some());
    }

    #[test]
    fn tag_and_query_lines() {
        let mut session = SessionState::new();
        assert!(!session.is_tagged(5));
        session.tag_lines(vec![5, 10, 15]);
        assert!(session.is_tagged(5));
        assert!(session.is_tagged(10));
        assert!(!session.is_tagged(7));
    }

    #[test]
    fn clear_tags_removes_all() {
        let mut session = SessionState::new();
        session.tag_lines(vec![1, 2, 3]);
        session.clear_tags();
        assert!(!session.is_tagged(1));
        assert!(!session.is_tagged(2));
    }

    #[test]
    fn cursor_position_default_and_update() {
        let mut session = SessionState::new();
        assert_eq!(session.cursor_line(), 0);
        assert_eq!(session.cursor_column(), 0);
        session.set_cursor(42, 15);
        assert_eq!(session.cursor_line(), 42);
        assert_eq!(session.cursor_column(), 15);
    }
}
