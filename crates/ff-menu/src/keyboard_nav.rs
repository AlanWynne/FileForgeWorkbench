//! Keyboard navigation state machine for menus.
//!
//! Implements the full keyboard navigation flow for the menu bar:
//! - Alt+letter to open a top-level menu
//! - Arrow keys for item/menu navigation
//! - Escape to close menus
//! - F10 to activate the menu bar

/// Keyboard navigation state machine for the menu bar.
///
/// Tracks whether the menu is inactive, focused (no dropdown open),
/// or open with a specific dropdown and highlighted item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuNavState {
    /// Menu bar is inactive; no menu is open.
    Inactive,
    /// Menu bar is focused (e.g., F10 pressed) but no dropdown open yet.
    Focused {
        /// Index of the highlighted top-level menu heading.
        highlighted_index: usize,
    },
    /// A dropdown menu is open with a highlighted item.
    Open {
        /// Index of the open top-level menu.
        menu_index: usize,
        /// Index of the highlighted item within the menu (None = no item highlighted).
        item_index: Option<usize>,
        /// Stack of submenu indices for nested navigation.
        submenu_stack: Vec<usize>,
    },
}

impl MenuNavState {
    /// Returns true if any menu is currently open.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open { .. })
    }

    /// Returns true if the menu bar has focus (but not necessarily an open dropdown).
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Inactive)
    }

    /// Returns the index of the currently open or focused menu, if any.
    pub fn active_menu_index(&self) -> Option<usize> {
        match self {
            Self::Inactive => None,
            Self::Focused { highlighted_index } => Some(*highlighted_index),
            Self::Open { menu_index, .. } => Some(*menu_index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_state_properties() {
        let state = MenuNavState::Inactive;
        assert!(!state.is_open());
        assert!(!state.is_active());
        assert_eq!(state.active_menu_index(), None);
    }

    #[test]
    fn focused_state_properties() {
        let state = MenuNavState::Focused {
            highlighted_index: 2,
        };
        assert!(!state.is_open());
        assert!(state.is_active());
        assert_eq!(state.active_menu_index(), Some(2));
    }

    #[test]
    fn open_state_properties() {
        let state = MenuNavState::Open {
            menu_index: 1,
            item_index: Some(3),
            submenu_stack: vec![],
        };
        assert!(state.is_open());
        assert!(state.is_active());
        assert_eq!(state.active_menu_index(), Some(1));
    }
}
