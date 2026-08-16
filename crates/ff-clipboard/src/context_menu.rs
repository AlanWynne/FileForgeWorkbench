//! Context menu clipboard state — determines enabled/disabled state for
//! Cut, Copy, and Paste menu items.

/// Represents the enabled/disabled state of clipboard context menu items.
///
/// The UI layer uses this to render menu items as enabled or greyed out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardContextMenuState {
    /// Whether the Cut menu item should be enabled.
    pub can_cut: bool,
    /// Whether the Copy menu item should be enabled.
    pub can_copy: bool,
    /// Whether the Paste menu item should be enabled.
    pub can_paste: bool,
}

impl ClipboardContextMenuState {
    /// Compute the menu state from the current selection and clipboard state.
    ///
    /// - Cut and Copy are enabled only when a non-empty selection is active.
    /// - Paste is enabled only when the clipboard contains text content.
    pub fn compute(has_selection: bool, clipboard_has_text: bool) -> Self {
        Self {
            can_cut: has_selection,
            can_copy: has_selection,
            can_paste: clipboard_has_text,
        }
    }

    /// Create a state where all items are disabled.
    pub fn all_disabled() -> Self {
        Self {
            can_cut: false,
            can_copy: false,
            can_paste: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_selection_disables_cut_and_copy() {
        // Validates: Requirement 5.6
        let state = ClipboardContextMenuState::compute(false, true);
        assert!(!state.can_cut);
        assert!(!state.can_copy);
        assert!(state.can_paste);
    }

    #[test]
    fn no_clipboard_content_disables_paste() {
        // Validates: Requirement 5.7
        let state = ClipboardContextMenuState::compute(true, false);
        assert!(state.can_cut);
        assert!(state.can_copy);
        assert!(!state.can_paste);
    }

    #[test]
    fn selection_and_clipboard_enables_all() {
        // Validates: Requirement 5.2, 5.3, 5.4
        let state = ClipboardContextMenuState::compute(true, true);
        assert!(state.can_cut);
        assert!(state.can_copy);
        assert!(state.can_paste);
    }

    #[test]
    fn no_selection_no_clipboard_disables_all() {
        let state = ClipboardContextMenuState::compute(false, false);
        assert!(!state.can_cut);
        assert!(!state.can_copy);
        assert!(!state.can_paste);
    }

    #[test]
    fn all_disabled_helper() {
        let state = ClipboardContextMenuState::all_disabled();
        assert!(!state.can_cut);
        assert!(!state.can_copy);
        assert!(!state.can_paste);
    }
}
