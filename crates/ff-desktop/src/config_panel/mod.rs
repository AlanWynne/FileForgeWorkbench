//! Config Panel -- the interactive flat configuration-key browser/editor opened
//! by the `CONFIG` command. This is NOT the Settings MENU (a data-driven
//! Menu_Workspace backed by `menus/settings.toml`); "Settings" refers to that
//! menu, "Config" refers to this key browser (CR-CH-025).
//!
//! Displays all schema-registered configuration keys grouped by namespace,
//! with type-appropriate widgets, provenance badges, inline validation,
//! and a Reset to Default button.
//!
//! Validates: Requirement 15.2-15.8

use std::collections::HashMap;

use eframe::egui;
use ff_config::value::ConfigValue;

mod tree;
pub use tree::{
    reconcile_cursor, reduce_config_key, visible_rows, ConfigNodeId, ConfigRow, ConfigTreeEffect,
    ConfigTreeKey, TreeEntry,
};

mod commit;
mod keyboard;
mod render;
mod widgets;
pub use render::render;

/// A single pending edit held in the Config panel's WORKING-COPY BUFFER
/// (project-master Requirement 11, B081). The panel edits this buffer only; the
/// backing `ConfigHandle` store is untouched until an explicit SAVE/END flushes
/// it (Req 11.2 -- no autosave-on-edit). A `Set` stages a new user-layer value;
/// a `Reset` stages removal of the user override (restoring the lower-layer
/// default), deferred until commit.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkingEdit {
    /// Stage a new value for the key (flushed via `set_user_value`).
    Set(ConfigValue),
    /// Stage a Reset-to-default for the key (flushed via `remove_user_value`).
    Reset,
}

/// Persistent state for the Config Panel tab (the flat config-key browser).
///
/// Validates: Requirement 15.2, 15.7
pub struct ConfigPanelState {
    /// Current filter text (case-insensitive substring match).
    pub filter: String,
    /// Active namespace filter when this Config view is namespace-scoped
    /// (e.g. `Some("editor".to_string())`, opened by `CONFIG editor`). `None`
    /// for the unfiltered all-keys view. Drives the tab title and F3/END return.
    ///
    /// Validates: cw-requirements.md Requirement 10.1, 10.5, 10.6
    pub namespace_filter: Option<String>,
    /// Collapsed state per namespace group (true = collapsed).
    pub collapsed: HashMap<String, bool>,
    /// Pending text held WHILE a text/number field has focus (before the value
    /// is parsed into the working buffer). Transient per-widget edit state, NOT
    /// the working copy itself.
    pub pending: HashMap<String, String>,
    /// Inline validation error messages keyed by schema key.
    pub errors: HashMap<String, String>,
    /// The WORKING-COPY BUFFER of unsaved edits (project-master Requirement 11,
    /// B081). Keyed by schema key; a `Set` stages a value, a `Reset` stages
    /// removal of the user override. Empty == no unsaved edits (the displayed
    /// values equal the persisted store). Editing a widget mutates THIS buffer
    /// only; SAVE/END flush it to the `ConfigHandle`, RETURN drops it. This
    /// replaces the former autosave-on-edit model (Req 11.2).
    pub working: HashMap<String, WorkingEdit>,
    /// The error from the LAST persist attempt (SAVE/END), if any, so END can
    /// stay-open-and-report on a persist failure (Req 11.4). Cleared on a
    /// successful flush and on discard/reload.
    pub persist_error: Option<String>,
    /// The keyboard Tree_Cursor: the node the arrow keys act on (CR-CH-039,
    /// Requirement 21.2). `None` until navigation first enters the tree.
    pub cursor: Option<ConfigNodeId>,
    /// Transient per-frame map from a key's full path to the `egui::Id` of its
    /// value-editing widget, captured while rendering so Enter on a key node
    /// (Requirement 21.6) can request focus on the exact widget regardless of
    /// its type. Rebuilt every frame; not persisted.
    widget_ids: HashMap<String, egui::Id>,
}

impl ConfigPanelState {
    /// Create a new, empty config panel state.
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            namespace_filter: None,
            collapsed: HashMap::new(),
            pending: HashMap::new(),
            errors: HashMap::new(),
            working: HashMap::new(),
            persist_error: None,
            cursor: None,
            widget_ids: HashMap::new(),
        }
    }

    /// Stage a value edit into the WORKING-COPY BUFFER (Req 11.2). The backing
    /// store is NOT written -- the edit is held until an explicit SAVE/END.
    pub fn stage_set(&mut self, key: &str, value: ConfigValue) {
        self.errors.remove(key);
        self.working
            .insert(key.to_string(), WorkingEdit::Set(value));
    }

    /// Stage a Reset-to-default into the WORKING-COPY BUFFER (Req 11.2): the
    /// user override is removed on the next SAVE/END, not immediately.
    pub fn stage_reset(&mut self, key: &str) {
        self.pending.remove(key);
        self.errors.remove(key);
        self.working.insert(key.to_string(), WorkingEdit::Reset);
    }

    /// The value the panel should DISPLAY/EDIT for `key`: the staged buffer
    /// value if the key has a pending `Set`, otherwise the persisted effective
    /// value passed in. A staged `Reset` falls through to the persisted value
    /// shown by the caller (the reset only takes effect on flush).
    ///
    /// Validates: project-master Requirement 11.1
    pub fn working_value(&self, key: &str, persisted: &ConfigValue) -> ConfigValue {
        match self.working.get(key) {
            Some(WorkingEdit::Set(v)) => v.clone(),
            _ => persisted.clone(),
        }
    }

    /// Whether the working buffer holds any unsaved edits (dirty state, Req 11.9).
    pub fn is_dirty(&self) -> bool {
        !self.working.is_empty()
    }

    /// Drop the working buffer WITHOUT writing the backing store (the RETURN /
    /// discard path, Req 11.5). Leaves the persisted store exactly as it was.
    pub fn clear_working(&mut self) {
        self.working.clear();
        self.pending.clear();
        self.errors.clear();
        self.persist_error = None;
    }

    /// Take the staged edits out of the buffer, leaving it empty, so the caller
    /// (`config_panel_persist`) can flush them to the store. On a flush FAILURE
    /// the caller is responsible for restoring the unflushed edits.
    pub fn take_working(&mut self) -> HashMap<String, WorkingEdit> {
        std::mem::take(&mut self.working)
    }
}

impl Default for ConfigPanelState {
    fn default() -> Self {
        Self::new()
    }
}

/// Stable egui id of the Filter field -- the FIRST interior control of the
/// Settings/CONFIG panel. The shell reports this as `first_interior_id` so the
/// CR-CH-023 Boundary_Policy can latch the command-field -> first-interior Tab
/// jump to a real, non-phantom widget (B058).
pub fn filter_field_id() -> egui::Id {
    egui::Id::new("config_panel_filter")
}

/// `WorkspaceContext` impl (CR-NR-078): the Config panel renders the flat
/// config-key browser and reports the Filter field as its single interior focus
/// stop. Edits stage into the panel's WORKING-COPY BUFFER (project-master
/// Req 11, B081); the in-panel Save affordance enqueues a `SAVE` `ShellRequest`
/// so persistence is gated through the shared SAVE/END seam, not an in-place
/// autosave.
///
/// Validates: workspace-framework Requirement 1.4, 1.5, 6.1; project-master Req 11.
impl crate::shell::workspace_context::WorkspaceContext for ConfigPanelState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        render(ui, self, services.config, services.requests);
        crate::shell::workspace_context::InteriorFocus::single(filter_field_id())
    }
}
