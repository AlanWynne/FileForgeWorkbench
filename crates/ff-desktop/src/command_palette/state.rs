//! Command Palette state data model.
//!
//! Validates: Requirement 1.1, 4.3 (command-palette)

/// A single entry displayed in the Command Palette list.
///
/// Validates: Requirement 3.1 (command-palette)
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteEntry {
    /// The command ID string (e.g. "file.open").
    pub command_id: String,
    /// Human-readable display name (e.g. "Open File").
    pub display_name: String,
    /// Category label (e.g. "File", "Edit").
    pub category: String,
    /// Full description shown in the detail area.
    pub description: String,
    /// Bound keyboard shortcut label, if any (e.g. "Ctrl+S").
    pub shortcut: Option<String>,
    /// Whether this command is currently enabled.
    pub enabled: bool,
    /// Match score from the fuzzy engine (higher = better).
    pub score: i32,
}

/// Runtime state for the Command Palette overlay.
///
/// Validates: Requirement 1.1, 4.3 (command-palette)
#[derive(Debug, Default)]
pub struct CommandPaletteState {
    /// Current text in the search input field.
    pub query: String,
    /// Filtered and sorted list of matching entries.
    pub filtered: Vec<PaletteEntry>,
    /// Index of the currently highlighted entry (0-based).
    pub selected_index: usize,
    /// Whether the palette is open.
    pub open: bool,
    /// Whether the search field should receive focus on the next frame.
    pub focus_search: bool,
}

impl CommandPaletteState {
    /// Open the palette and reset search state.
    ///
    /// Validates: Requirement 1.1
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected_index = 0;
        self.focus_search = true;
    }

    /// Close the palette.
    ///
    /// Validates: Requirement 1.2, 1.3
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.filtered.clear();
        self.selected_index = 0;
    }

    /// Move selection down by one, wrapping at the bottom.
    ///
    /// Validates: Requirement 4.3
    pub fn select_next(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.filtered.len();
    }

    /// Move selection up by one, wrapping at the top.
    ///
    /// Validates: Requirement 4.3
    pub fn select_prev(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.filtered.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Return the currently highlighted entry, if any.
    pub fn selected_entry(&self) -> Option<&PaletteEntry> {
        self.filtered.get(self.selected_index)
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, name: &str, enabled: bool) -> PaletteEntry {
        PaletteEntry {
            command_id: id.to_string(),
            display_name: name.to_string(),
            category: "Test".to_string(),
            description: String::new(),
            shortcut: None,
            enabled,
            score: 0,
        }
    }

    /// Validates: Requirement 1.1 -- open() resets query and sets open=true.
    #[test]
    fn open_resets_state_and_sets_open_true() {
        // Validates: command-palette Requirement 1.1
        let mut state = CommandPaletteState::default();
        state.query = "old query".to_string();
        state.selected_index = 3;
        state.open();
        assert!(state.open);
        assert!(state.query.is_empty());
        assert_eq!(state.selected_index, 0);
        assert!(state.focus_search);
    }

    /// Validates: Requirement 1.2 -- close() clears state and sets open=false.
    #[test]
    fn close_clears_state_and_sets_open_false() {
        // Validates: command-palette Requirement 1.2
        let mut state = CommandPaletteState::default();
        state.open();
        state.filtered.push(entry("a", "A", true));
        state.close();
        assert!(!state.open);
        assert!(state.query.is_empty());
        assert!(state.filtered.is_empty());
        assert_eq!(state.selected_index, 0);
    }

    /// Validates: Requirement 4.3 -- select_next wraps at bottom.
    #[test]
    fn select_next_wraps_at_bottom() {
        // Validates: command-palette Requirement 4.3
        let mut state = CommandPaletteState::default();
        state.filtered = vec![entry("a", "A", true), entry("b", "B", true)];
        state.selected_index = 1;
        state.select_next();
        assert_eq!(state.selected_index, 0);
    }

    /// Validates: Requirement 4.3 -- select_prev wraps at top.
    #[test]
    fn select_prev_wraps_at_top() {
        // Validates: command-palette Requirement 4.3
        let mut state = CommandPaletteState::default();
        state.filtered = vec![entry("a", "A", true), entry("b", "B", true)];
        state.selected_index = 0;
        state.select_prev();
        assert_eq!(state.selected_index, 1);
    }

    /// Validates: Requirement 4.3 -- selected_entry returns correct entry.
    #[test]
    fn selected_entry_returns_highlighted_entry() {
        // Validates: command-palette Requirement 4.3
        let mut state = CommandPaletteState::default();
        state.filtered = vec![entry("a", "Alpha", true), entry("b", "Beta", true)];
        state.selected_index = 1;
        assert_eq!(state.selected_entry().unwrap().display_name, "Beta");
    }
}
