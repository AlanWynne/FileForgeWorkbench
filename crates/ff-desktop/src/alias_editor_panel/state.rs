//! State and Action types for the Alias Editor (command-configurator Req 5,
//! CR-NR-113 + the CR-CH-068 ISPF redesign). Modelled on `MenusEditorState` /
//! `MenusEditorAction`.

use crate::shell::user_aliases::AliasEnvironment;

/// One editable alias row in the Alias Editor working copy (Req 5.1, 5.3, 5.13).
///
/// The persisted fields (`surface`, `canonical`, `environment`,
/// `command_template`, `localisation_key`) map 1:1 to `UserAliasRowToml`
/// (design 9.10.2). `act` is a TRANSIENT per-row line-command letter (I / R / D)
/// that is never persisted (Req 5.16).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AliasRowEdit {
    /// Transient ISPF line-command letter for this row (I / R / D). Never
    /// persisted; collected as a PENDING code and applied as a batch on the
    /// `Command ===>` Enter (Req 5.16, 5.22).
    pub act: String,
    /// The alias name (typeable surface, e.g. `ECOB`). Column: ALIAS.
    pub surface: String,
    /// The root command (canonical verb, e.g. `EDIT`). Column: Root.
    pub canonical: String,
    /// The target Command Environment (ffcmd | ffedit, default ffcmd). Column: CE.
    pub environment: AliasEnvironment,
    /// CR-NR-112 parameter template (`""` = pure verb alias). Column: Parameters.
    pub command_template: String,
    /// The locale key (L18N column). Blank for a language-independent user
    /// shorthand; PERSISTS via `UserAliasRowToml.localisation_key` (Req 5.13).
    pub localisation_key: String,
}

/// A structural action produced by the Alias Editor render, applied by the
/// shell. Per-field edits (surface / root / environment / template / L18N / act)
/// mutate the working copy directly in the render (mirroring the Menus Editor
/// B054) and do NOT produce actions.
///
/// Validates: command-configurator Requirement 5.3, 5.4, 5.5, 5.14, 5.16, 5.22.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AliasEditorAction {
    /// No action this frame.
    #[default]
    None,
    /// Apply the pending Act line commands (I / R / D) as a deterministic batch
    /// (Req 5.16). Emitted on a bare `Command ===>` Enter with no verb.
    ApplyLineCommands,
    /// A `Command ===>` primary command was entered (carries the typed text).
    /// The shell applies pending line commands FIRST, then interprets the verb
    /// (Req 5.14, 5.22).
    CommandLine(String),
}

/// Per-Context UI state for the Alias Editor. Lives on the shell (like
/// `MenusEditorState`), not on the `TabState`.
///
/// Validates: command-configurator Requirement 5.1, 5.3, 5.13, 5.14, 5.15.
#[derive(Debug, Clone, Default)]
pub struct AliasEditorState {
    /// The working copy of the user's alias rows (unsaved edits live here).
    pub rows: Vec<AliasRowEdit>,
    /// The most recent validation/save error, shown inline.
    pub error: Option<String>,
    /// The `Command ===>` primary command field text (transient, not persisted).
    pub command_line: String,
    /// The FILTER row's CE filter (None = any environment); transient (Req 5.15).
    pub filter_ce: Option<AliasEnvironment>,
    /// The FILTER row's Root filter (empty = any root); transient (Req 5.15).
    pub filter_root: String,
    /// Egui id of the FIRST focusable interior control -- the FILTER row's filter
    /// CE dropdown, captured fresh each frame from its `response.id` (design
    /// 9.10.8 part 3), so the shell Boundary_Policy can focus it on Tab from the
    /// command field (CR-CH-023, Req 5.23). Transient render output.
    pub first_interior_id: Option<egui::Id>,
    /// Egui id of the LAST focusable interior control, captured fresh each frame;
    /// anchors the Shift+Tab reverse boundary. Transient.
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

    /// Load `rows` as the working copy, clearing any error, command line, and
    /// filter.
    pub fn load_rows(&mut self, rows: Vec<AliasRowEdit>) {
        self.rows = rows;
        self.error = None;
        self.command_line.clear();
        self.filter_ce = None;
        self.filter_root.clear();
    }
}
