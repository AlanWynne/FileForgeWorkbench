//! State and Action types for the Menus Editor (menu-workspace Req 13).

use crate::menu_workspace::{GroupSeparator, MenuFile};

/// Which text field of an option row an [`MenusEditorAction::EditOption`]
/// targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionField {
    /// The Option_Key (1-4 chars, uppercased on save).
    Key,
    /// The Option_Command.
    Command,
    /// The Option_Description.
    Description,
    /// The optional group label.
    Group,
}

/// An action produced by the Menus Editor render, applied by the shell.
///
/// Validates: menu-workspace Requirement 13.4-13.9, 13.12.
#[derive(Debug, Clone, PartialEq)]
pub enum MenusEditorAction {
    /// No action this frame.
    None,
    /// Select a different menu to edit (by name: "POM", "Settings", or user).
    Select(String),
    /// Set the menu title.
    EditTitle(String),
    /// Toggle the per-menu calendar.
    SetShowCalendar(bool),
    /// Set the group boundary style.
    SetGroupSeparator(GroupSeparator),
    /// Toggle group header labels.
    SetGroupHeaders(bool),
    /// Edit a text field of the option at `index`.
    EditOption {
        /// Zero-based option index.
        index: usize,
        /// Which field is edited.
        field: OptionField,
        /// The new value.
        value: String,
    },
    /// Set the `enabled` flag of the option at `index`.
    SetOptionEnabled {
        /// Zero-based option index.
        index: usize,
        /// The new enabled state.
        enabled: bool,
    },
    /// Append a new blank option.
    AddOption,
    /// Delete the option at `index`.
    DeleteOption(usize),
    /// Move the option at `index` one position earlier.
    MoveOptionUp(usize),
    /// Move the option at `index` one position later.
    MoveOptionDown(usize),
    /// Save the working menu to the selected menu's file.
    Save,
    /// Save the working menu under a new name.
    SaveAs(String),
}

/// Per-Context UI state for the Menus Editor. Lives on the shell (like
/// `ThemeEditorState`), not on the `TabState`.
///
/// Validates: menu-workspace Requirement 13.1, 13.2.
#[derive(Debug, Clone, Default)]
pub struct MenusEditorState {
    /// Editable menu names: "POM", "Settings", plus every user menu file.
    pub available: Vec<String>,
    /// The menu currently selected for editing.
    pub selected: Option<String>,
    /// The working-copy menu being edited (unsaved edits live here).
    pub working: Option<MenuFile>,
    /// New-name buffer for Save As.
    pub name_buffer: String,
    /// The most recent validation/save error, shown inline.
    pub error: Option<String>,
    /// True when the editor was opened from the Settings menu (so END returns
    /// to the Settings menu, not straight to the POM). Mirrors the
    /// Settings_Namespace_View one-level-back behaviour (cw-requirements
    /// Req 10.4; menu-workspace Req 13.1, B053).
    pub opened_from_settings: bool,
}

impl MenusEditorState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a menu as the working copy: sets `selected` and `working`, clears
    /// any error.
    pub fn load_working(&mut self, name: &str, menu: MenuFile) {
        self.selected = Some(name.to_string());
        self.working = Some(menu);
        self.error = None;
    }
}
