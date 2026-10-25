//! State and Action types for the Alias Editor (command-configurator Req 5,
//! CR-NR-113). Modelled on `MenusEditorState` / `MenusEditorAction`.

use crate::shell::user_aliases::AliasEnvironment;

/// One editable alias row in the Alias Editor working copy (Req 5.1, 5.3).
/// `localisation_key` is intentionally omitted: a user shorthand is
/// language-independent (left blank on save).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AliasRowEdit {
    /// The alias name (typeable surface, e.g. `ECOB`).
    pub surface: String,
    /// The root command (canonical verb, e.g. `EDIT`).
    pub canonical: String,
    /// The target Command Environment (ffcmd | ffedit, default ffcmd).
    pub environment: AliasEnvironment,
    /// CR-NR-112 parameter template (`""` = pure verb alias).
    pub command_template: String,
}

/// A structural action produced by the Alias Editor render, applied by the
/// shell. Per-field edits (surface / root / environment / template) mutate the
/// working copy directly in the render (mirroring the Menus Editor B054) and do
/// NOT produce actions.
///
/// Validates: command-configurator Requirement 5.3, 5.4, 5.5.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AliasEditorAction {
    /// No action this frame.
    #[default]
    None,
    /// Append a new blank alias row.
    AddRow,
    /// Delete the row at `index`.
    DeleteRow(usize),
    /// Validate + serialise + write the working copy to the User_Alias_Store.
    Save,
}

/// Per-Context UI state for the Alias Editor. Lives on the shell (like
/// `MenusEditorState`), not on the `TabState`.
///
/// Validates: command-configurator Requirement 5.1, 5.3.
#[derive(Debug, Clone, Default)]
pub struct AliasEditorState {
    /// The working copy of the user's alias rows (unsaved edits live here).
    pub rows: Vec<AliasRowEdit>,
    /// The most recent validation/save error, shown inline.
    pub error: Option<String>,
    /// Confirmation gate for a per-row delete (the row index awaiting confirm).
    pub pending_delete: Option<usize>,
    /// Egui id of the FIRST focusable interior control (the first row's surface
    /// field, or the Add button when empty), captured fresh each frame by
    /// `render` so the shell Boundary_Policy can focus it on Tab from the command
    /// field (CR-CH-023). Transient render output; not serialised.
    pub first_interior_id: Option<egui::Id>,
    /// Egui id of the LAST focusable interior control (the Save button), captured
    /// fresh each frame; anchors the Shift+Tab reverse boundary. Transient.
    pub last_interior_id: Option<egui::Id>,
    /// Action produced by the most recent `WorkspaceContext::render`, stashed for
    /// the shell to apply via `apply_alias_editor_action` (CR-NR-078). Transient.
    pub pending_action: AliasEditorAction,
}

impl AliasEditorState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load `rows` as the working copy, clearing any error and pending delete.
    pub fn load_rows(&mut self, rows: Vec<AliasRowEdit>) {
        self.rows = rows;
        self.error = None;
        self.pending_delete = None;
    }
}
