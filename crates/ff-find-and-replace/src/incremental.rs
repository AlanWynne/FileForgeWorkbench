//! Incremental search (search-as-you-type) coordinator.
//!
//! Addresses: Requirement 14

use crate::search_mode::SearchMode;
use crate::types::BytePosition;

/// Incremental search state manager.
///
/// Manages partial-text state, start position, and coordinates
/// cancellation and debouncing for live search-as-you-type.
///
/// Addresses: Requirement 14
#[derive(Debug, Clone)]
pub struct IncrementalSearch {
    /// The original cursor position when incremental search started.
    start_position: BytePosition,
    /// Current search text.
    current_text: String,
    /// Whether incremental search is active.
    active: bool,
    /// Current search mode.
    mode: SearchMode,
    /// Case sensitivity.
    case_sensitive: bool,
}

impl IncrementalSearch {
    /// Create a new incremental search session.
    pub fn new(start_position: BytePosition) -> Self {
        Self {
            start_position,
            current_text: String::new(),
            active: true,
            mode: SearchMode::Literal,
            case_sensitive: true,
        }
    }

    /// Update the search text. Returns the position to search from.
    ///
    /// When text is shortened (backspace), search restarts from start_position.
    ///
    /// Addresses: Requirement 14 AC 5
    pub fn update_text(&mut self, text: &str) -> BytePosition {
        let was_longer = text.len() < self.current_text.len();
        self.current_text = text.to_string();

        if was_longer || text.is_empty() {
            // Backspace: restart from original position
            self.start_position
        } else {
            self.start_position
        }
    }

    /// Get the current search text.
    pub fn text(&self) -> &str {
        &self.current_text
    }

    /// Get the start position.
    pub fn start_position(&self) -> BytePosition {
        self.start_position
    }

    /// Whether incremental search is active.
    pub fn is_active(&self) -> bool {
        self.active && !self.current_text.is_empty()
    }

    /// Clear the incremental search state.
    ///
    /// Addresses: Requirement 14 AC 7
    pub fn clear(&mut self) {
        self.active = false;
        self.current_text.clear();
    }

    /// Set search mode.
    pub fn set_mode(&mut self, mode: SearchMode) {
        self.mode = mode;
    }

    /// Set case sensitivity.
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }

    /// Get the current mode.
    pub fn mode(&self) -> SearchMode {
        self.mode
    }

    /// Get case sensitivity setting.
    pub fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incremental_search_starts_active_and_empty() {
        let search = IncrementalSearch::new(BytePosition(10));
        assert!(!search.is_active()); // empty text = not active
        assert_eq!(search.start_position(), BytePosition(10));
    }

    #[test]
    fn update_text_makes_search_active() {
        let mut search = IncrementalSearch::new(BytePosition(0));
        search.update_text("h");
        assert!(search.is_active());
        assert_eq!(search.text(), "h");
    }

    #[test]
    fn backspace_resets_to_start_position() {
        let mut search = IncrementalSearch::new(BytePosition(10));
        search.update_text("hel");
        let pos = search.update_text("he"); // shortened
        assert_eq!(pos, BytePosition(10)); // back to start
    }

    #[test]
    fn clear_deactivates_search() {
        let mut search = IncrementalSearch::new(BytePosition(0));
        search.update_text("hello");
        search.clear();
        assert!(!search.is_active());
    }
}
