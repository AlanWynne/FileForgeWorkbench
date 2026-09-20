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

mod tree;
pub use tree::{
    reconcile_cursor, reduce_config_key, visible_rows, ConfigNodeId, ConfigRow, ConfigTreeEffect,
    ConfigTreeKey, TreeEntry,
};

mod render;
pub use render::render;

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
    /// Pending edit values keyed by schema key (before commit).
    pub pending: HashMap<String, String>,
    /// Inline validation error messages keyed by schema key.
    pub errors: HashMap<String, String>,
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
            cursor: None,
            widget_ids: HashMap::new(),
        }
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
/// stop. Config changes are committed in-place through the `ConfigHandle`, so
/// there are no `ShellRequest`s to enqueue.
///
/// Validates: workspace-framework Requirement 1.4, 1.5, 6.1.
impl crate::shell::workspace_context::WorkspaceContext for ConfigPanelState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        render(ui, self, services.config);
        crate::shell::workspace_context::InteriorFocus::single(filter_field_id())
    }
}
