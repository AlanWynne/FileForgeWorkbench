//! Edit mode management — Insert, Overstrike, Browse.
//!
//! The `EditMode` enum and `EditModeManager` struct handle per-editor-instance
//! mode state. The default mode is Insert.

/// The current editing mode for an editor instance.
///
/// - `Insert`: Characters are inserted at the caret, pushing text rightward.
/// - `Overstrike`: Characters replace the character at the caret position.
/// - `Browse`: Document is read-only; no edits permitted. Navigation only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditMode {
    /// Characters are inserted at the caret, pushing text rightward.
    #[default]
    Insert,
    /// Characters replace the character at the caret position.
    Overstrike,
    /// Document is read-only; no edits permitted. Navigation only.
    Browse,
}

impl std::fmt::Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditMode::Insert => write!(f, "INSERT"),
            EditMode::Overstrike => write!(f, "OVERSTRIKE"),
            EditMode::Browse => write!(f, "BROWSE"),
        }
    }
}

/// Manages per-editor-instance edit mode state.
///
/// Each editor instance has its own `EditModeManager` that persists
/// for the lifetime of the editor session.
#[derive(Debug, Clone)]
pub struct EditModeManager {
    mode: EditMode,
}

impl EditModeManager {
    /// Creates a new mode manager in the default Insert mode.
    pub fn new() -> Self {
        Self {
            mode: EditMode::Insert,
        }
    }

    /// Returns the current editing mode.
    pub fn mode(&self) -> EditMode {
        self.mode
    }

    /// Sets the editing mode directly.
    pub fn set_mode(&mut self, mode: EditMode) {
        self.mode = mode;
    }

    /// Toggles between Insert and Overstrike mode.
    ///
    /// If currently in Browse mode, this is a no-op (Browse mode must
    /// be exited explicitly).
    pub fn toggle(&mut self) {
        self.mode = match self.mode {
            EditMode::Insert => EditMode::Overstrike,
            EditMode::Overstrike => EditMode::Insert,
            EditMode::Browse => EditMode::Browse,
        };
    }

    /// Returns true if the current mode is Insert.
    pub fn is_insert(&self) -> bool {
        self.mode == EditMode::Insert
    }

    /// Returns true if the current mode is Overstrike.
    pub fn is_overstrike(&self) -> bool {
        self.mode == EditMode::Overstrike
    }

    /// Returns true if the editor is in an editable mode (Insert or Overstrike).
    pub fn is_editable(&self) -> bool {
        matches!(self.mode, EditMode::Insert | EditMode::Overstrike)
    }
}

impl Default for EditModeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_insert() {
        let manager = EditModeManager::new();
        assert_eq!(manager.mode(), EditMode::Insert);
        assert!(manager.is_insert());
        assert!(!manager.is_overstrike());
    }

    #[test]
    fn toggle_switches_insert_to_overstrike() {
        let mut manager = EditModeManager::new();
        manager.toggle();
        assert_eq!(manager.mode(), EditMode::Overstrike);
        assert!(manager.is_overstrike());
        assert!(!manager.is_insert());
    }

    #[test]
    fn toggle_switches_overstrike_to_insert() {
        let mut manager = EditModeManager::new();
        manager.toggle();
        manager.toggle();
        assert_eq!(manager.mode(), EditMode::Insert);
    }

    #[test]
    fn toggle_is_noop_in_browse_mode() {
        let mut manager = EditModeManager::new();
        manager.set_mode(EditMode::Browse);
        manager.toggle();
        assert_eq!(manager.mode(), EditMode::Browse);
    }

    #[test]
    fn is_editable_true_for_insert_and_overstrike() {
        let mut manager = EditModeManager::new();
        assert!(manager.is_editable());

        manager.set_mode(EditMode::Overstrike);
        assert!(manager.is_editable());
    }

    #[test]
    fn is_editable_false_for_browse() {
        let mut manager = EditModeManager::new();
        manager.set_mode(EditMode::Browse);
        assert!(!manager.is_editable());
    }

    #[test]
    fn set_mode_changes_mode_directly() {
        let mut manager = EditModeManager::new();
        manager.set_mode(EditMode::Browse);
        assert_eq!(manager.mode(), EditMode::Browse);
    }

    #[test]
    fn display_formats_mode_names_correctly() {
        assert_eq!(EditMode::Insert.to_string(), "INSERT");
        assert_eq!(EditMode::Overstrike.to_string(), "OVERSTRIKE");
        assert_eq!(EditMode::Browse.to_string(), "BROWSE");
    }

    #[test]
    fn mode_persists_across_multiple_operations() {
        let mut manager = EditModeManager::new();
        manager.toggle(); // Now Overstrike
        assert_eq!(manager.mode(), EditMode::Overstrike);
        // Simulate multiple queries — mode doesn't change on its own
        assert_eq!(manager.mode(), EditMode::Overstrike);
        assert_eq!(manager.mode(), EditMode::Overstrike);
    }
}
