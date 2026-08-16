//! The `LayoutState` — a serializable snapshot of the complete layout.

use std::collections::HashMap;

use crate::dock::zone::DockZone;
use crate::floating::window::FloatingWindow;
use crate::panel::display_state::PanelDisplayState;
use crate::tabs::group::TabGroupTree;
use crate::SCHEMA_VERSION;

/// A serializable snapshot of the complete layout.
///
/// Persisted at exit, restored at startup, and used for persona definitions.
/// Contains all information needed to reconstruct the workspace layout.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayoutState {
    /// Schema version for forward-compatible migration.
    pub schema_version: u32,
    /// Dock zone contents: panel assignments with dimensions.
    pub docked_panels: Vec<DockedPanelState>,
    /// Tab group arrangement in the center area.
    pub tab_groups: TabGroupTree,
    /// Floating window positions and contents.
    pub floating_windows: Vec<FloatingWindow>,
    /// Splitter positions as proportional values [0.0, 1.0].
    pub splitter_positions: HashMap<String, f32>,
    /// Panel visibility map (hidden panels tracked here).
    pub panel_visibility: HashMap<String, bool>,
    /// Panel display states (minimized/normal/maximized).
    pub panel_display_states: HashMap<String, PanelDisplayState>,
}

impl Default for LayoutState {
    fn default() -> Self {
        use crate::tabs::group::{TabGroup, TabGroupId};
        Self {
            schema_version: SCHEMA_VERSION,
            docked_panels: Vec::new(),
            tab_groups: TabGroupTree::Leaf(TabGroup::new(TabGroupId::new(1), vec![])),
            floating_windows: Vec::new(),
            splitter_positions: HashMap::new(),
            panel_visibility: HashMap::new(),
            panel_display_states: HashMap::new(),
        }
    }
}

impl LayoutState {
    /// Creates a new default layout state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all tab identifiers currently open in the layout.
    pub fn all_open_tabs(&self) -> Vec<&str> {
        self.tab_groups.all_tabs()
    }

    /// Returns true if the given panel is visible (not hidden).
    pub fn is_panel_visible(&self, panel_id: &str) -> bool {
        self.panel_visibility.get(panel_id).copied().unwrap_or(true)
    }

    /// Returns the display state of a panel.
    pub fn panel_display_state(&self, panel_id: &str) -> PanelDisplayState {
        self.panel_display_states
            .get(panel_id)
            .copied()
            .unwrap_or(PanelDisplayState::Normal)
    }

    /// Returns the docked panel state for a given panel_id.
    pub fn find_docked_panel(&self, panel_id: &str) -> Option<&DockedPanelState> {
        self.docked_panels.iter().find(|p| p.panel_id == panel_id)
    }

    /// Returns the floating window containing the given panel_id.
    pub fn find_floating_panel(&self, panel_id: &str) -> Option<&FloatingWindow> {
        self.floating_windows
            .iter()
            .find(|w| w.panels.contains(&panel_id.to_string()))
    }
}

/// State for a single docked panel within the layout.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DockedPanelState {
    /// The unique panel identifier.
    pub panel_id: String,
    /// The dock zone this panel is assigned to.
    pub zone: DockZone,
    /// Zone width or height in logical pixels.
    pub zone_dimension: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_state_has_schema_version() {
        let state = LayoutState::default();
        assert_eq!(state.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn default_layout_state_has_empty_tab_group() {
        let state = LayoutState::default();
        assert_eq!(state.tab_groups.total_tab_count(), 0);
    }

    #[test]
    fn is_panel_visible_defaults_to_true() {
        let state = LayoutState::default();
        assert!(state.is_panel_visible("any_panel"));
    }

    #[test]
    fn is_panel_visible_respects_hidden_state() {
        let mut state = LayoutState::default();
        state
            .panel_visibility
            .insert("hidden_panel".to_string(), false);
        assert!(!state.is_panel_visible("hidden_panel"));
    }

    #[test]
    fn panel_display_state_defaults_to_normal() {
        let state = LayoutState::default();
        assert_eq!(
            state.panel_display_state("any_panel"),
            PanelDisplayState::Normal
        );
    }

    #[test]
    fn find_docked_panel_returns_match() {
        let mut state = LayoutState::default();
        state.docked_panels.push(DockedPanelState {
            panel_id: "file_tree".to_string(),
            zone: DockZone::Left,
            zone_dimension: 250.0,
        });
        let found = state.find_docked_panel("file_tree").unwrap();
        assert_eq!(found.zone, DockZone::Left);
        assert_eq!(found.zone_dimension, 250.0);
    }

    #[test]
    fn find_docked_panel_returns_none_for_missing() {
        let state = LayoutState::default();
        assert!(state.find_docked_panel("nonexistent").is_none());
    }
}
