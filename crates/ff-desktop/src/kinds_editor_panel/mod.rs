//! Kinds Editor Context (workspace-kinds Requirement 6, CR-NR-090 B.4).
//!
//! Configures a Workspace Kind's title / menu bar / key list / profile, and
//! creates a new Kind "modelled on" a built-in base. Modelled on the Keys
//! Workspace (`keys_editor_panel`): a pure render that stashes a
//! [`KindsEditorAction`] the shell applies (write `workspace-kinds/<name>.toml`
//! + reload the registry).

use crate::workspace_kind::{BuiltinKind, KindConfig};

/// The action produced by one Kinds Editor render frame; the shell applies it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum KindsEditorAction {
    /// No action this frame.
    #[default]
    None,
    /// Select a different Kind by name (load its config into the working copy).
    SelectKind(String),
    /// Create a NEW user Kind `name` modelled on the built-in `base`.
    NewKind { name: String, base: BuiltinKind },
    /// Save the working Kind config (write file + reload registry).
    Save,
}

/// The Kinds Editor working state (workspace-kinds Req 6.1).
#[derive(Debug, Clone)]
pub struct KindsEditorState {
    /// The name of the currently selected Kind, or `None` before selection.
    pub selected: Option<String>,
    /// The editable working copy of the selected Kind's config.
    pub working: Option<KindConfig>,
    /// The list of Kind names offered in the selector (built-in + user).
    pub kind_names: Vec<String>,
    /// The "new Kind" sub-form: the entered name.
    pub new_name: String,
    /// The "new Kind" sub-form: the selected built-in base.
    pub new_base: BuiltinKind,
    /// A non-blocking status / error line.
    pub error: Option<String>,
    /// Interior focus ids for the shell Boundary_Policy (CR-NR-078).
    pub first_interior_id: Option<egui::Id>,
    pub last_interior_id: Option<egui::Id>,
    /// The action stashed by the last render for the shell to drain.
    pub pending_action: KindsEditorAction,
}

impl Default for KindsEditorState {
    fn default() -> Self {
        Self {
            selected: None,
            working: None,
            kind_names: Vec::new(),
            new_name: String::new(),
            new_base: BuiltinKind::Editor,
            error: None,
            first_interior_id: None,
            last_interior_id: None,
            pending_action: KindsEditorAction::None,
        }
    }
}

impl KindsEditorState {
    /// Load a Kind's config into the working copy (selection changed).
    pub fn load_kind(&mut self, name: &str, config: KindConfig) {
        self.selected = Some(name.to_string());
        self.working = Some(config);
        self.error = None;
    }
}

mod render;

#[cfg(test)]
mod tests;
