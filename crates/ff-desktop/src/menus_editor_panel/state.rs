//! State and Action types for the Menus Editor (menu-workspace Req 13).

use crate::menu_workspace::MenuFile;

/// A structural action produced by the Menus Editor render, applied by the
/// shell. Text and toggle fields mutate the working `MenuFile` directly in the
/// render (B054) and do NOT produce actions.
///
/// Validates: menu-workspace Requirement 13.4-13.9.
#[derive(Debug, Clone, PartialEq)]
pub enum MenusEditorAction {
    /// No action this frame.
    None,
    /// Select a different menu to edit (by name: "POM", "Settings", or user).
    Select(String),
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
