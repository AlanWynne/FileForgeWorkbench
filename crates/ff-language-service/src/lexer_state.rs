//! Multi-line lexer state persistence: per-line state vectors.

/// The lexer's internal state at the end of a document line.
pub type LexerState = i32;

/// Sentinel value indicating an invalid/uninitialized line state.
pub const LEXER_STATE_INVALID: LexerState = i32::MIN;

/// Initial state for the beginning of a document (line 0 start state).
pub const LEXER_STATE_INITIAL: LexerState = 0;

/// Per-document vector of end-of-line lexer states.
///
/// Supports documents with millions of lines using a compact `Vec<i32>`.
/// Each entry stores the lexer state at the end of the corresponding line.
/// `LEXER_STATE_INVALID` indicates the state needs recomputation.
#[derive(Debug, Clone)]
pub struct LineStateVector {
    /// End-of-line state for each line.
    states: Vec<LexerState>,
}

impl LineStateVector {
    /// Create a new state vector for a document with `line_count` lines.
    ///
    /// All states are initialized to `LEXER_STATE_INVALID`.
    pub fn new(line_count: usize) -> Self {
        Self {
            states: vec![LEXER_STATE_INVALID; line_count],
        }
    }

    /// Store the end-of-line lexer state after highlighting a line.
    ///
    /// # Panics
    ///
    /// Panics if `line_index` is out of bounds.
    pub fn set_end_state(&mut self, line_index: usize, state: LexerState) {
        self.states[line_index] = state;
    }

    /// Get the starting state for highlighting line `line_index`.
    ///
    /// Returns `LEXER_STATE_INITIAL` (0) for line 0.
    /// For other lines, returns the stored state of the previous line,
    /// or `LEXER_STATE_INVALID` if the previous line's state is invalid.
    pub fn get_start_state(&self, line_index: usize) -> LexerState {
        if line_index == 0 {
            LEXER_STATE_INITIAL
        } else {
            self.states[line_index - 1]
        }
    }

    /// Mark the specified line's state as invalid, signalling re-highlighting.
    pub fn invalidate_from(&mut self, line_index: usize) {
        if line_index < self.states.len() {
            self.states[line_index] = LEXER_STATE_INVALID;
        }
    }

    /// Check whether re-highlighting should continue past a line.
    ///
    /// Returns `false` when `new_state` equals the previously stored state
    /// (incremental highlighting termination condition).
    /// Returns `true` if propagation should continue.
    pub fn should_continue(&self, line_index: usize, new_state: LexerState) -> bool {
        if line_index >= self.states.len() {
            return false;
        }
        self.states[line_index] != new_state
    }

    /// Insert `count` lines at position `at` with `LEXER_STATE_INVALID` entries.
    ///
    /// Also invalidates the line following the insertion point (if it exists).
    pub fn insert_lines(&mut self, at: usize, count: usize) {
        let at = at.min(self.states.len());
        self.states
            .splice(at..at, std::iter::repeat_n(LEXER_STATE_INVALID, count));
        // Invalidate the line following the insertion
        let following = at + count;
        if following < self.states.len() {
            self.states[following] = LEXER_STATE_INVALID;
        }
    }

    /// Remove `count` entries starting at `at`.
    ///
    /// Invalidates the line at `at` after deletion (the new occupant of that position).
    pub fn delete_lines(&mut self, at: usize, count: usize) {
        let at = at.min(self.states.len());
        let end = (at + count).min(self.states.len());
        self.states.drain(at..end);
        // Invalidate the line at the deletion point (new occupant)
        if at < self.states.len() {
            self.states[at] = LEXER_STATE_INVALID;
        }
    }

    /// Total number of lines tracked.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether the vector is empty (no lines).
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Get the raw state at a specific line index.
    pub fn get_state(&self, line_index: usize) -> Option<LexerState> {
        self.states.get(line_index).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_all_entries_as_invalid() {
        // Validates: Requirement 4.1
        let sv = LineStateVector::new(5);
        assert_eq!(sv.len(), 5);
        for i in 0..5 {
            assert_eq!(sv.get_state(i), Some(LEXER_STATE_INVALID));
        }
    }

    #[test]
    fn set_and_get_end_state() {
        // Validates: Requirement 4.2
        let mut sv = LineStateVector::new(3);
        sv.set_end_state(0, 42);
        sv.set_end_state(1, 7);
        assert_eq!(sv.get_state(0), Some(42));
        assert_eq!(sv.get_state(1), Some(7));
    }

    #[test]
    fn get_start_state_line_zero_returns_initial() {
        // Validates: Requirement 4.4
        let sv = LineStateVector::new(3);
        assert_eq!(sv.get_start_state(0), LEXER_STATE_INITIAL);
    }

    #[test]
    fn get_start_state_returns_previous_line_state() {
        // Validates: Requirement 4.4
        let mut sv = LineStateVector::new(3);
        sv.set_end_state(0, 10);
        sv.set_end_state(1, 20);
        assert_eq!(sv.get_start_state(1), 10);
        assert_eq!(sv.get_start_state(2), 20);
    }

    #[test]
    fn invalidate_from_marks_line_invalid() {
        // Validates: Requirement 4.3
        let mut sv = LineStateVector::new(3);
        sv.set_end_state(1, 5);
        sv.invalidate_from(1);
        assert_eq!(sv.get_state(1), Some(LEXER_STATE_INVALID));
    }

    #[test]
    fn should_continue_returns_false_when_state_matches() {
        // Validates: Requirement 4.5
        let mut sv = LineStateVector::new(3);
        sv.set_end_state(1, 42);
        assert!(!sv.should_continue(1, 42));
    }

    #[test]
    fn should_continue_returns_true_when_state_differs() {
        // Validates: Requirement 4.5
        let mut sv = LineStateVector::new(3);
        sv.set_end_state(1, 42);
        assert!(sv.should_continue(1, 99));
    }

    #[test]
    fn insert_lines_adds_invalid_entries() {
        // Validates: Requirement 4.6
        let mut sv = LineStateVector::new(3);
        sv.set_end_state(0, 1);
        sv.set_end_state(1, 2);
        sv.set_end_state(2, 3);

        sv.insert_lines(1, 2);
        assert_eq!(sv.len(), 5);
        assert_eq!(sv.get_state(0), Some(1));
        assert_eq!(sv.get_state(1), Some(LEXER_STATE_INVALID));
        assert_eq!(sv.get_state(2), Some(LEXER_STATE_INVALID));
        // Line following insertion is invalidated
        assert_eq!(sv.get_state(3), Some(LEXER_STATE_INVALID));
        assert_eq!(sv.get_state(4), Some(3));
    }

    #[test]
    fn delete_lines_removes_entries_and_invalidates_next() {
        // Validates: Requirement 4.7
        let mut sv = LineStateVector::new(5);
        sv.set_end_state(0, 10);
        sv.set_end_state(1, 20);
        sv.set_end_state(2, 30);
        sv.set_end_state(3, 40);
        sv.set_end_state(4, 50);

        sv.delete_lines(1, 2);
        assert_eq!(sv.len(), 3);
        assert_eq!(sv.get_state(0), Some(10));
        // Line at deletion point is invalidated
        assert_eq!(sv.get_state(1), Some(LEXER_STATE_INVALID));
        assert_eq!(sv.get_state(2), Some(50));
    }

    #[test]
    fn large_document_allocation() {
        // Validates: Requirement 4.8
        let sv = LineStateVector::new(1_000_000);
        assert_eq!(sv.len(), 1_000_000);
        assert_eq!(sv.get_start_state(0), LEXER_STATE_INITIAL);
        assert_eq!(sv.get_state(999_999), Some(LEXER_STATE_INVALID));
    }
}
