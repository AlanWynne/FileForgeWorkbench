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
    /// Ordered egui widget ids of every focusable control in the editor, in
    /// visual (top-to-bottom, left-to-right) order, captured fresh each render.
    /// The shell's Tab handler walks this list so EVERY interactive control --
    /// including the menu selector, checkboxes and the separator selectables,
    /// not just text fields -- is reachable by keyboard (accessibility). The
    /// command line is prepended by the shell, so this holds only the editor's
    /// own controls.
    ///
    /// Validates: accessibility (keyboard reachability); menu-workspace Req 13.4.
    pub focus_ids: Vec<egui::Id>,
    /// The ring id the shell last requested focus for (Tab/Shift+Tab). Advancing
    /// from THIS -- rather than from egui's reported focus -- avoids skipping a
    /// stop when a `request_focus` needs a frame to take effect (some widgets do
    /// not report as focused the same frame the request is made).
    pub last_focus_target: Option<egui::Id>,
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
