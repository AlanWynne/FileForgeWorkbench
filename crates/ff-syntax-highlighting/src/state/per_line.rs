//! Per-line state storage for incremental re-highlighting.
//!
//! Stores the lexer state at the end of each line, enabling the engine to resume
//! lexing from any line without re-processing from the beginning.

use crate::types::{LexerState, LineNumber};

/// Per-line state storage synchronized with document line count.
/// Addresses: Requirement 3, criterion 3.1
pub struct PerLineState {
    /// Lexer state at end of each line. Index i holds the state at end of line i.
    states: Vec<LexerState>,
}

impl PerLineState {
    /// Create per-line state for the given number of lines, initialized to INITIAL.
    pub fn new(line_count: usize) -> Self {
        Self {
            states: vec![LexerState::INITIAL; line_count],
        }
    }

    /// Get the state at the start of a line.
    /// For line 0, returns INITIAL. For line N, returns the state at end of line N-1.
    /// Addresses: Requirement 3, criterion 3.3
    pub fn state_at_line_start(&self, line: LineNumber) -> LexerState {
        if line.0 == 0 {
            LexerState::INITIAL
        } else {
            self.states
                .get(line.0 - 1)
                .copied()
                .unwrap_or(LexerState::INITIAL)
        }
    }

    /// Set the state at end of a line.
    /// Addresses: Requirement 3, criterion 3.7
    pub fn set_state(&mut self, line: LineNumber, state: LexerState) {
        if line.0 < self.states.len() {
            self.states[line.0] = state;
        }
    }

    /// Get the state stored at end of a line (for convergence checking).
    pub fn state_at_line_end(&self, line: LineNumber) -> LexerState {
        self.states
            .get(line.0)
            .copied()
            .unwrap_or(LexerState::INITIAL)
    }

    /// Insert initial-state entries for new lines.
    /// Addresses: Requirement 3, criterion 3.8
    pub fn insert_lines(&mut self, at: LineNumber, count: usize) {
        let pos = at.0.min(self.states.len());
        self.states
            .splice(pos..pos, std::iter::repeat_n(LexerState::INITIAL, count));
    }

    /// Remove entries for deleted lines.
    /// Addresses: Requirement 3, criterion 3.9
    pub fn delete_lines(&mut self, at: LineNumber, count: usize) {
        let pos = at.0.min(self.states.len());
        let end = (pos + count).min(self.states.len());
        self.states.drain(pos..end);
    }

    /// Get the total number of lines tracked.
    pub fn line_count(&self) -> usize {
        self.states.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_all_to_initial() {
        // Validates: Requirement 3, criterion 3.1
        let pls = PerLineState::new(5);
        assert_eq!(pls.line_count(), 5);
        for i in 0..5 {
            assert_eq!(pls.state_at_line_end(LineNumber(i)), LexerState::INITIAL);
        }
    }

    #[test]
    fn state_at_line_start_returns_initial_for_line_0() {
        // Validates: Requirement 3, criterion 3.3
        let pls = PerLineState::new(3);
        assert_eq!(pls.state_at_line_start(LineNumber(0)), LexerState::INITIAL);
    }

    #[test]
    fn state_at_line_start_returns_previous_line_end() {
        let mut pls = PerLineState::new(3);
        pls.set_state(LineNumber(0), LexerState(5));
        assert_eq!(pls.state_at_line_start(LineNumber(1)), LexerState(5));
    }

    #[test]
    fn set_and_get_state() {
        // Validates: Requirement 3, criterion 3.7
        let mut pls = PerLineState::new(5);
        pls.set_state(LineNumber(2), LexerState(42));
        assert_eq!(pls.state_at_line_end(LineNumber(2)), LexerState(42));
    }

    #[test]
    fn insert_lines_adds_initial_state_entries() {
        // Validates: Requirement 3, criterion 3.8
        let mut pls = PerLineState::new(3);
        pls.set_state(LineNumber(0), LexerState(1));
        pls.set_state(LineNumber(1), LexerState(2));
        pls.set_state(LineNumber(2), LexerState(3));

        pls.insert_lines(LineNumber(1), 2);
        assert_eq!(pls.line_count(), 5);
        assert_eq!(pls.state_at_line_end(LineNumber(0)), LexerState(1));
        assert_eq!(pls.state_at_line_end(LineNumber(1)), LexerState::INITIAL);
        assert_eq!(pls.state_at_line_end(LineNumber(2)), LexerState::INITIAL);
        assert_eq!(pls.state_at_line_end(LineNumber(3)), LexerState(2));
        assert_eq!(pls.state_at_line_end(LineNumber(4)), LexerState(3));
    }

    #[test]
    fn delete_lines_removes_state_entries() {
        // Validates: Requirement 3, criterion 3.9
        let mut pls = PerLineState::new(5);
        pls.set_state(LineNumber(0), LexerState(1));
        pls.set_state(LineNumber(1), LexerState(2));
        pls.set_state(LineNumber(2), LexerState(3));
        pls.set_state(LineNumber(3), LexerState(4));
        pls.set_state(LineNumber(4), LexerState(5));

        pls.delete_lines(LineNumber(1), 2);
        assert_eq!(pls.line_count(), 3);
        assert_eq!(pls.state_at_line_end(LineNumber(0)), LexerState(1));
        assert_eq!(pls.state_at_line_end(LineNumber(1)), LexerState(4));
        assert_eq!(pls.state_at_line_end(LineNumber(2)), LexerState(5));
    }

    #[test]
    fn insert_lines_at_end() {
        let mut pls = PerLineState::new(2);
        pls.insert_lines(LineNumber(2), 3);
        assert_eq!(pls.line_count(), 5);
    }

    #[test]
    fn delete_lines_clamps_to_bounds() {
        let mut pls = PerLineState::new(3);
        pls.delete_lines(LineNumber(1), 100);
        assert_eq!(pls.line_count(), 1);
    }
}
