//! Primary command field controller and history.
//!
//! The command field ("Command ===>") provides ISPF-style command entry.
//! It supports history recall with Up/Down arrow navigation.

/// The state of the primary command field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFieldState {
    /// Current text content of the input field.
    pub text: String,
    /// Whether the field currently has keyboard focus.
    pub has_focus: bool,
    /// Current position in the history ring (-1 = live input, 0 = most recent).
    pub history_position: i32,
    /// Saved live input when browsing history.
    pub saved_input: String,
}

/// Result of a command field submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitResult {
    /// Command was dispatched successfully.
    Dispatched,
    /// Command was not recognized — error message provided.
    Unrecognized {
        /// The error message to display.
        error_message: String,
    },
}

/// Controller for the primary command field ("Command ===>").
///
/// Manages text input, command submission, and history navigation.
/// The controller does not directly dispatch commands — it prepares
/// the command text and delegates dispatch to the caller.
#[derive(Debug, Clone)]
pub struct CommandFieldController {
    /// Current field state.
    state: CommandFieldState,
    /// Command history (most recent at the end).
    history: Vec<String>,
    /// Maximum history entries to retain.
    max_history: usize,
}

impl CommandFieldController {
    /// Creates a new controller with empty history and default max of 100 entries.
    pub fn new() -> Self {
        Self {
            state: CommandFieldState {
                text: String::new(),
                has_focus: false,
                history_position: -1,
                saved_input: String::new(),
            },
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Returns the current field state for rendering.
    pub fn state(&self) -> &CommandFieldState {
        &self.state
    }

    /// Returns the current text content.
    pub fn text(&self) -> &str {
        &self.state.text
    }

    /// Sets the text content (e.g., when the user types).
    pub fn set_text(&mut self, text: String) {
        self.state.text = text;
    }

    /// Submits the current field content.
    ///
    /// If the field is empty, returns `None`.
    /// Otherwise, adds the text to history, clears the field, and returns the command text.
    pub fn submit(&mut self) -> Option<String> {
        let text = self.state.text.trim().to_string();
        if text.is_empty() {
            return None;
        }

        // Add to history (deduplicate: remove previous occurrence if any)
        self.history.retain(|h| h != &text);
        self.history.push(text.clone());

        // Trim history to max
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Clear field and reset history position
        self.state.text.clear();
        self.state.history_position = -1;
        self.state.saved_input.clear();

        Some(text)
    }

    /// Navigates command history.
    ///
    /// `direction`: -1 for older (Up arrow), +1 for newer (Down arrow).
    pub fn history_navigate(&mut self, direction: i32) {
        if self.history.is_empty() {
            return;
        }

        let history_len = self.history.len() as i32;

        if direction < 0 {
            // Navigate to older entries (Up arrow)
            if self.state.history_position == -1 {
                // Save current input before entering history
                self.state.saved_input = self.state.text.clone();
                self.state.history_position = 0;
            } else if self.state.history_position < history_len - 1 {
                self.state.history_position += 1;
            }
            // Clamp at oldest
        } else if direction > 0 {
            // Navigate to newer entries (Down arrow)
            if self.state.history_position > 0 {
                self.state.history_position -= 1;
            } else if self.state.history_position == 0 {
                // Return to live input
                self.state.history_position = -1;
                self.state.text = self.state.saved_input.clone();
                return;
            }
        }

        // Update text from history (history_position 0 = most recent = last element)
        if self.state.history_position >= 0 {
            let index = self.history.len() - 1 - self.state.history_position as usize;
            self.state.text = self.history[index].clone();
        }
    }

    /// Returns true if focus should transfer to the editor (Down arrow on empty field).
    pub fn should_transfer_focus_down(&self) -> bool {
        self.state.text.is_empty() && self.state.history_position == -1
    }

    /// Sets the focus state of the command field.
    pub fn set_focus(&mut self, focused: bool) {
        self.state.has_focus = focused;
    }

    /// Loads command history from persisted state.
    pub fn load_history(&mut self, entries: Vec<String>) {
        self.history = entries;
        if self.history.len() > self.max_history {
            let excess = self.history.len() - self.max_history;
            self.history.drain(0..excess);
        }
    }

    /// Returns the current history entries for persistence.
    pub fn history_entries(&self) -> &[String] {
        &self.history
    }

    /// Returns the number of history entries.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for CommandFieldController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_controller_has_empty_state() {
        let ctrl = CommandFieldController::new();
        assert_eq!(ctrl.text(), "");
        assert!(!ctrl.state().has_focus);
        assert_eq!(ctrl.state().history_position, -1);
        assert_eq!(ctrl.history_len(), 0);
    }

    #[test]
    fn submit_returns_trimmed_text_and_clears_field() {
        let mut ctrl = CommandFieldController::new();
        ctrl.set_text("  SAVE  ".to_string());
        let result = ctrl.submit();
        assert_eq!(result, Some("SAVE".to_string()));
        assert_eq!(ctrl.text(), "");
    }

    #[test]
    fn submit_empty_field_returns_none() {
        let mut ctrl = CommandFieldController::new();
        ctrl.set_text("   ".to_string());
        assert_eq!(ctrl.submit(), None);
    }

    #[test]
    fn submit_adds_to_history() {
        let mut ctrl = CommandFieldController::new();
        ctrl.set_text("FIND foo".to_string());
        ctrl.submit();
        ctrl.set_text("SAVE".to_string());
        ctrl.submit();
        assert_eq!(ctrl.history_entries(), &["FIND foo", "SAVE"]);
    }

    #[test]
    fn submit_deduplicates_history() {
        let mut ctrl = CommandFieldController::new();
        ctrl.set_text("SAVE".to_string());
        ctrl.submit();
        ctrl.set_text("FIND".to_string());
        ctrl.submit();
        ctrl.set_text("SAVE".to_string());
        ctrl.submit();
        // "SAVE" should appear only once (at the end)
        assert_eq!(ctrl.history_entries(), &["FIND", "SAVE"]);
    }

    #[test]
    fn history_navigate_up_cycles_through_history() {
        let mut ctrl = CommandFieldController::new();
        ctrl.set_text("cmd1".to_string());
        ctrl.submit();
        ctrl.set_text("cmd2".to_string());
        ctrl.submit();
        ctrl.set_text("cmd3".to_string());
        ctrl.submit();

        // Navigate up through history
        ctrl.history_navigate(-1); // most recent = cmd3
        assert_eq!(ctrl.text(), "cmd3");
        ctrl.history_navigate(-1); // cmd2
        assert_eq!(ctrl.text(), "cmd2");
        ctrl.history_navigate(-1); // cmd1 (oldest)
        assert_eq!(ctrl.text(), "cmd1");
        ctrl.history_navigate(-1); // stays at cmd1 (clamped)
        assert_eq!(ctrl.text(), "cmd1");
    }

    #[test]
    fn history_navigate_down_returns_to_live_input() {
        let mut ctrl = CommandFieldController::new();
        ctrl.set_text("cmd1".to_string());
        ctrl.submit();
        ctrl.set_text("cmd2".to_string());
        ctrl.submit();

        // Type something, then navigate
        ctrl.set_text("partial".to_string());
        ctrl.history_navigate(-1); // cmd2
        ctrl.history_navigate(-1); // cmd1
        ctrl.history_navigate(1); // cmd2
        assert_eq!(ctrl.text(), "cmd2");
        ctrl.history_navigate(1); // back to live input
        assert_eq!(ctrl.text(), "partial");
    }

    #[test]
    fn should_transfer_focus_down_when_empty_and_no_history_browsing() {
        let ctrl = CommandFieldController::new();
        assert!(ctrl.should_transfer_focus_down());
    }

    #[test]
    fn should_not_transfer_focus_when_text_present() {
        let mut ctrl = CommandFieldController::new();
        ctrl.set_text("something".to_string());
        assert!(!ctrl.should_transfer_focus_down());
    }

    #[test]
    fn history_navigate_with_empty_history_does_nothing() {
        let mut ctrl = CommandFieldController::new();
        ctrl.history_navigate(-1);
        assert_eq!(ctrl.text(), "");
        assert_eq!(ctrl.state().history_position, -1);
    }
}
