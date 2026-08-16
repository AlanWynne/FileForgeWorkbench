//! RETRIEVE command and Retrieve Pointer logic.
//!
//! Manages the single-step backward recall through Command History.
//! The pointer advances on each successive RETRIEVE call and resets
//! when any non-RETRIEVE command is submitted.

use crate::command_history::CommandHistory;

/// Result of a RETRIEVE command invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrieveResult {
    /// Successfully recalled a command. Place it in the command field.
    Recalled {
        /// The recalled command string.
        command: String,
    },
    /// History is empty; nothing to recall.
    HistoryEmpty,
    /// Already at the oldest entry; no older history exists.
    NoOlderHistory,
    /// The command field contained "LIST" — show the full history as a selectable list.
    ///
    /// Validates: Requirement 19.1, 19.2
    ShowList {
        /// All history entries in most-recent-first order.
        entries: Vec<String>,
    },
}

/// The state of the RETRIEVE pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PointerState {
    /// No retrieval cycle active. Next RETRIEVE starts from most recent.
    Initial,
    /// Currently pointing at a specific index in CommandHistory.
    AtIndex(usize),
}

/// Manages the Retrieve Pointer, cycling backward through history on successive calls.
#[derive(Debug)]
pub struct RetrieveState {
    /// Current pointer state.
    state: PointerState,
}

impl RetrieveState {
    /// Create a new retrieve state at the initial position.
    pub fn new() -> Self {
        Self {
            state: PointerState::Initial,
        }
    }

    /// Execute one RETRIEVE step.
    ///
    /// `command_field_text` is the current text in the Primary_Command_Field.
    /// If it equals "LIST" (case-insensitive, trimmed), returns `ShowList` with
    /// all history entries instead of performing single-step recall.
    ///
    /// Validates: Requirement 5.1–5.4, 5.7, 19.1–19.2
    pub fn retrieve(
        &mut self,
        history: &CommandHistory,
        command_field_text: &str,
    ) -> RetrieveResult {
        // LIST trigger — Validates: Requirement 19.1
        if command_field_text.trim().eq_ignore_ascii_case("LIST") {
            let entries = history.iter().map(|e| e.command().to_string()).collect();
            return RetrieveResult::ShowList { entries };
        }

        if history.is_empty() {
            return RetrieveResult::HistoryEmpty;
        }

        match &self.state {
            PointerState::Initial => {
                self.state = PointerState::AtIndex(0);
                RetrieveResult::Recalled {
                    command: history.get(0).unwrap().command().to_string(),
                }
            }
            PointerState::AtIndex(current) => {
                let next = current + 1;
                if next >= history.len() {
                    RetrieveResult::NoOlderHistory
                } else {
                    self.state = PointerState::AtIndex(next);
                    RetrieveResult::Recalled {
                        command: history.get(next).unwrap().command().to_string(),
                    }
                }
            }
        }
    }

    /// Reset the pointer to initial position.
    ///
    /// Called when any non-RETRIEVE command is submitted.
    pub fn reset(&mut self) {
        self.state = PointerState::Initial;
    }

    /// Set the pointer to a specific index (used by History_Dropdown selection).
    pub fn set_position(&mut self, index: usize) {
        self.state = PointerState::AtIndex(index);
    }

    /// Whether the pointer is at the initial (no retrieval) position.
    pub fn is_at_initial(&self) -> bool {
        matches!(self.state, PointerState::Initial)
    }

    /// The current pointer index, if active.
    pub fn current_index(&self) -> Option<usize> {
        match &self.state {
            PointerState::Initial => None,
            PointerState::AtIndex(i) => Some(*i),
        }
    }
}

impl Default for RetrieveState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_history(commands: &[&str]) -> CommandHistory {
        let mut history = CommandHistory::new(200);
        // Add in reverse order so first element is "most recent"
        for &cmd in commands.iter().rev() {
            history.add(cmd);
        }
        history
    }

    #[test]
    fn retrieve_from_empty_history() {
        // Validates: Requirement 5.7
        let history = CommandHistory::new(200);
        let mut state = RetrieveState::new();
        assert_eq!(state.retrieve(&history, ""), RetrieveResult::HistoryEmpty);
    }

    #[test]
    fn initial_retrieve_returns_most_recent() {
        // Validates: Requirement 5.2
        let history = make_history(&["CMD1", "CMD2", "CMD3"]);
        let mut state = RetrieveState::new();

        let result = state.retrieve(&history, "");
        assert_eq!(
            result,
            RetrieveResult::Recalled {
                command: "CMD1".to_string()
            }
        );
    }

    #[test]
    fn successive_retrieves_cycle_backward() {
        // Validates: Requirement 5.3
        let history = make_history(&["CMD1", "CMD2", "CMD3"]);
        let mut state = RetrieveState::new();

        assert_eq!(
            state.retrieve(&history, ""),
            RetrieveResult::Recalled {
                command: "CMD1".to_string()
            }
        );
        assert_eq!(
            state.retrieve(&history, ""),
            RetrieveResult::Recalled {
                command: "CMD2".to_string()
            }
        );
        assert_eq!(
            state.retrieve(&history, ""),
            RetrieveResult::Recalled {
                command: "CMD3".to_string()
            }
        );
    }

    #[test]
    fn retrieve_past_end_returns_no_older_history() {
        // Validates: Requirement 5.4
        let history = make_history(&["CMD1", "CMD2"]);
        let mut state = RetrieveState::new();

        state.retrieve(&history, ""); // CMD1
        state.retrieve(&history, ""); // CMD2
        let result = state.retrieve(&history, ""); // past end
        assert_eq!(result, RetrieveResult::NoOlderHistory);
    }

    #[test]
    fn reset_on_non_retrieve_command() {
        // Validates: Requirement 5.5
        let history = make_history(&["CMD1", "CMD2", "CMD3"]);
        let mut state = RetrieveState::new();

        state.retrieve(&history, ""); // CMD1
        state.retrieve(&history, ""); // CMD2

        state.reset();

        assert_eq!(
            state.retrieve(&history, ""),
            RetrieveResult::Recalled {
                command: "CMD1".to_string()
            }
        );
    }

    #[test]
    fn set_position_from_dropdown() {
        // Validates: Requirement 10.4
        let history = make_history(&["CMD1", "CMD2", "CMD3"]);
        let mut state = RetrieveState::new();

        state.set_position(1);

        assert_eq!(
            state.retrieve(&history, ""),
            RetrieveResult::Recalled {
                command: "CMD3".to_string()
            }
        );
    }

    #[test]
    fn list_trigger_returns_show_list_with_all_entries() {
        // Validates: Requirement 19.1, 19.2
        let history = make_history(&["CMD1", "CMD2", "CMD3"]);
        let mut state = RetrieveState::new();

        let result = state.retrieve(&history, "LIST");
        assert_eq!(
            result,
            RetrieveResult::ShowList {
                entries: vec!["CMD1".to_string(), "CMD2".to_string(), "CMD3".to_string()]
            }
        );
    }

    #[test]
    fn list_trigger_case_insensitive() {
        // Validates: Requirement 19.1
        let history = make_history(&["CMD1"]);
        let mut state = RetrieveState::new();
        assert!(matches!(
            state.retrieve(&history, "list"),
            RetrieveResult::ShowList { .. }
        ));
        assert!(matches!(
            state.retrieve(&history, "  List  "),
            RetrieveResult::ShowList { .. }
        ));
    }

    #[test]
    fn list_trigger_on_empty_history_returns_show_list_empty() {
        // Validates: Requirement 19.5
        let history = CommandHistory::new(200);
        let mut state = RetrieveState::new();
        assert_eq!(
            state.retrieve(&history, "LIST"),
            RetrieveResult::ShowList { entries: vec![] }
        );
    }

    #[test]
    fn is_at_initial_after_creation() {
        let state = RetrieveState::new();
        assert!(state.is_at_initial());
    }

    #[test]
    fn is_not_at_initial_after_retrieve() {
        let history = make_history(&["CMD1"]);
        let mut state = RetrieveState::new();
        state.retrieve(&history, "");
        assert!(!state.is_at_initial());
    }
}
