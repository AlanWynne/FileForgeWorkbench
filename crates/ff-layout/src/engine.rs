//! The `LayoutEngine` — central coordinator for the layout system.
//!
//! Owns the layout tree, manages all transitions between layout states,
//! and serves as the primary API surface for the shell and command framework.

use std::path::Path;

use crate::dock::zone::DockZone;
use crate::drag::coordinator::{DragDropCoordinator, DragItem, DragResult};
use crate::drag::indicator::DropIndicator;
use crate::error::LayoutError;
use crate::floating::manager::FloatingWindowManager;
use crate::floating::monitor::MonitorInfo;
use crate::floating::window::FloatingWindowId;
use crate::panel::display_state::PanelDisplayState;
use crate::panel::registry::PanelRegistry;
use crate::persona::definition::Persona;
use crate::persona::manager::PersonaManager;
use crate::resize::manager::SplitterManager;
use crate::resize::splitter::SplitterId;
use crate::state::layout_state::{DockedPanelState, LayoutState};
use crate::state::serializer;
use crate::tabs::group::TabGroupId;
use crate::tabs::manager::TabGroupManager;
use crate::{Position, Size, MAX_FLOATING_WINDOWS};

/// Result of handling a floating window close.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseAction {
    /// Panel was redocked successfully.
    Redocked,
    /// Unsaved changes — show save confirmation dialog.
    NeedsSaveConfirmation {
        /// The tab_id with unsaved changes.
        tab_id: String,
    },
}

/// The central coordinator for the layout system.
///
/// Owns the layout tree, manages all transitions between layout states,
/// and serves as the primary API surface for the shell and command framework.
pub struct LayoutEngine {
    /// The current in-memory layout state.
    state: LayoutState,
    /// Registry of all known panel types.
    panel_registry: PanelRegistry,
    /// Tab group management for center area splits.
    tab_groups: TabGroupManager,
    /// Floating window tracking.
    floating_windows: FloatingWindowManager,
    /// Persona management (presets).
    personas: PersonaManager,
    /// Drag-and-drop coordination.
    drag_drop: DragDropCoordinator,
    /// Splitter/resize management.
    splitters: SplitterManager,
}

impl LayoutEngine {
    /// Maximum number of simultaneous floating windows.
    pub const MAX_FLOATING_WINDOWS: usize = MAX_FLOATING_WINDOWS;

    /// Creates a new LayoutEngine with default layout (dock zones: left,
    /// right, bottom, center).
    pub fn new() -> Self {
        Self {
            state: LayoutState::default(),
            panel_registry: PanelRegistry::new(),
            tab_groups: TabGroupManager::new(),
            floating_windows: FloatingWindowManager::new(),
            personas: PersonaManager::new(),
            drag_drop: DragDropCoordinator::new(),
            splitters: SplitterManager::new(),
        }
    }

    /// Initialize from a persisted LayoutState (startup restoration).
    ///
    /// Applies graceful degradation for missing panels: panel_ids not
    /// in the registry are skipped with an INFO log.
    pub fn from_state(state: LayoutState, registry: PanelRegistry) -> Self {
        let tab_groups = TabGroupManager::from_tree(
            state.tab_groups.clone(),
            state
                .tab_groups
                .all_group_ids()
                .into_iter()
                .next()
                .unwrap_or(TabGroupId::new(1)),
        );
        let floating_windows = FloatingWindowManager::from_windows(state.floating_windows.clone());
        Self {
            state,
            panel_registry: registry,
            tab_groups,
            floating_windows,
            personas: PersonaManager::new(),
            drag_drop: DragDropCoordinator::new(),
            splitters: SplitterManager::new(),
        }
    }

    /// Returns the current LayoutState as a serializable snapshot.
    pub fn current_state(&self) -> &LayoutState {
        &self.state
    }

    /// Returns a mutable reference to the current LayoutState.
    ///
    /// Used for direct state manipulation during initialization or testing.
    pub fn current_state_mut(&mut self) -> &mut LayoutState {
        &mut self.state
    }

    /// Returns whether the layout has been modified from the active persona.
    pub fn is_persona_modified(&self) -> bool {
        self.personas.is_modified()
    }

    /// Returns the active persona name, if any.
    pub fn active_persona_name(&self) -> Option<&str> {
        self.personas.active_persona_name()
    }

    /// Returns a reference to the panel registry.
    pub fn panel_registry(&self) -> &PanelRegistry {
        &self.panel_registry
    }

    /// Returns a mutable reference to the panel registry.
    pub fn panel_registry_mut(&mut self) -> &mut PanelRegistry {
        &mut self.panel_registry
    }

    /// Returns a reference to the tab group manager.
    pub fn tab_groups(&self) -> &TabGroupManager {
        &self.tab_groups
    }

    /// Returns the floating window count.
    pub fn floating_window_count(&self) -> usize {
        self.floating_windows.count()
    }

    // ─── Panel Operations ───────────────────────────────────────────────

    /// Show a hidden panel in its last known dock zone.
    pub fn show_panel(&mut self, panel_id: &str) -> Result<(), LayoutError> {
        if !self.panel_registry.is_registered(panel_id) {
            return Err(LayoutError::PanelNotFound {
                panel_id: panel_id.to_string(),
            });
        }
        self.state
            .panel_visibility
            .insert(panel_id.to_string(), true);
        self.personas.mark_modified();
        Ok(())
    }

    /// Hide a panel while preserving its position in the LayoutState.
    pub fn hide_panel(&mut self, panel_id: &str) -> Result<(), LayoutError> {
        if !self.panel_registry.is_registered(panel_id) {
            return Err(LayoutError::PanelNotFound {
                panel_id: panel_id.to_string(),
            });
        }
        self.state
            .panel_visibility
            .insert(panel_id.to_string(), false);
        self.personas.mark_modified();
        Ok(())
    }

    /// Toggle panel visibility (show if hidden, hide if visible).
    pub fn toggle_panel(&mut self, panel_id: &str) -> Result<(), LayoutError> {
        if !self.panel_registry.is_registered(panel_id) {
            return Err(LayoutError::PanelNotFound {
                panel_id: panel_id.to_string(),
            });
        }
        let visible = self.state.is_panel_visible(panel_id);
        self.state
            .panel_visibility
            .insert(panel_id.to_string(), !visible);
        self.personas.mark_modified();
        Ok(())
    }

    /// Minimize a panel (collapse to tab/icon in zone header).
    pub fn minimize_panel(&mut self, panel_id: &str) -> Result<(), LayoutError> {
        if !self.panel_registry.is_registered(panel_id) {
            return Err(LayoutError::PanelNotFound {
                panel_id: panel_id.to_string(),
            });
        }
        self.state
            .panel_display_states
            .insert(panel_id.to_string(), PanelDisplayState::Minimized);
        self.personas.mark_modified();
        Ok(())
    }

    /// Maximize a panel (expand to fill primary window content area).
    pub fn maximize_panel(&mut self, panel_id: &str) -> Result<(), LayoutError> {
        if !self.panel_registry.is_registered(panel_id) {
            return Err(LayoutError::PanelNotFound {
                panel_id: panel_id.to_string(),
            });
        }
        self.state
            .panel_display_states
            .insert(panel_id.to_string(), PanelDisplayState::Maximized);
        self.personas.mark_modified();
        Ok(())
    }

    /// Restore a panel to normal display state.
    pub fn restore_panel(&mut self, panel_id: &str) -> Result<(), LayoutError> {
        if !self.panel_registry.is_registered(panel_id) {
            return Err(LayoutError::PanelNotFound {
                panel_id: panel_id.to_string(),
            });
        }
        self.state
            .panel_display_states
            .insert(panel_id.to_string(), PanelDisplayState::Normal);
        self.personas.mark_modified();
        Ok(())
    }

    // ─── Floating Window Operations ─────────────────────────────────────

    /// Undock a panel from its dock zone into a new floating window.
    pub fn undock_panel(&mut self, panel_id: &str) -> Result<FloatingWindowId, LayoutError> {
        if !self.panel_registry.is_registered(panel_id) {
            return Err(LayoutError::PanelNotFound {
                panel_id: panel_id.to_string(),
            });
        }
        let zone = self
            .state
            .find_docked_panel(panel_id)
            .map(|p| p.zone)
            .or_else(|| self.panel_registry.get(panel_id).map(|r| r.default_zone))
            .unwrap_or(DockZone::Center);

        let dim = self
            .state
            .find_docked_panel(panel_id)
            .map(|p| p.zone_dimension)
            .unwrap_or(300.0);

        let window_id = self.floating_windows.create_window(
            panel_id,
            Size::new(dim.max(200.0), dim.max(150.0)),
            zone,
        )?;

        // Remove from docked panels
        self.state.docked_panels.retain(|p| p.panel_id != panel_id);

        // Update state
        self.sync_floating_state();
        self.personas.mark_modified();
        Ok(window_id)
    }

    /// Undock a panel to a specific position (drag-to-float).
    pub fn undock_panel_at(
        &mut self,
        panel_id: &str,
        position: Position,
    ) -> Result<FloatingWindowId, LayoutError> {
        if !self.panel_registry.is_registered(panel_id) {
            return Err(LayoutError::PanelNotFound {
                panel_id: panel_id.to_string(),
            });
        }
        let zone = self
            .state
            .find_docked_panel(panel_id)
            .map(|p| p.zone)
            .or_else(|| self.panel_registry.get(panel_id).map(|r| r.default_zone))
            .unwrap_or(DockZone::Center);

        let dim = self
            .state
            .find_docked_panel(panel_id)
            .map(|p| p.zone_dimension)
            .unwrap_or(300.0);

        let window_id = self.floating_windows.create_window_at(
            panel_id,
            position,
            Size::new(dim.max(200.0), dim.max(150.0)),
            zone,
        )?;

        self.state.docked_panels.retain(|p| p.panel_id != panel_id);

        self.sync_floating_state();
        self.personas.mark_modified();
        Ok(window_id)
    }

    /// Redock a floating panel back to its most recent dock zone.
    pub fn redock_panel(&mut self, window_id: FloatingWindowId) -> Result<(), LayoutError> {
        let window = self
            .floating_windows
            .remove_window(window_id)
            .ok_or(LayoutError::FloatingWindowNotFound { window_id })?;

        for panel_id in &window.panels {
            self.state.docked_panels.push(DockedPanelState {
                panel_id: panel_id.clone(),
                zone: window.origin_zone,
                zone_dimension: window.size.width.max(window.size.height),
            });
        }

        self.sync_floating_state();
        self.personas.mark_modified();
        Ok(())
    }

    /// Undock a tab from a TabGroup into a new floating window.
    pub fn undock_tab(
        &mut self,
        group_id: TabGroupId,
        tab_index: usize,
    ) -> Result<FloatingWindowId, LayoutError> {
        let group = self
            .tab_groups
            .tree()
            .find_group(group_id)
            .ok_or(LayoutError::TabGroupNotFound { group_id })?;

        if group.tab_count() <= 1 && self.tab_groups.tree().all_group_ids().len() <= 1 {
            return Err(LayoutError::CannotEmptyEditor);
        }
        if tab_index >= group.tab_count() {
            return Err(LayoutError::TabIndexOutOfBounds {
                group_id,
                index: tab_index,
                count: group.tab_count(),
            });
        }

        let tab_id = group.tabs[tab_index].clone();
        // Remove from tab group
        let group_mut = self.tab_groups.tree_mut().find_group_mut(group_id).unwrap();
        group_mut.tabs.remove(tab_index);
        if group_mut.active_tab >= group_mut.tabs.len() && !group_mut.tabs.is_empty() {
            group_mut.active_tab = group_mut.tabs.len() - 1;
        }

        let window_id = self.floating_windows.create_window(
            &tab_id,
            Size::new(600.0, 400.0),
            DockZone::Center,
        )?;

        // Store original tab index for redock
        if let Some(window) = self.floating_windows.get_mut(window_id) {
            window.origin_tab_index = Some(tab_index);
        }

        self.sync_floating_state();
        self.personas.mark_modified();
        Ok(window_id)
    }

    /// Undock a tab to a specific position.
    pub fn undock_tab_at(
        &mut self,
        group_id: TabGroupId,
        tab_index: usize,
        position: Position,
    ) -> Result<FloatingWindowId, LayoutError> {
        let group = self
            .tab_groups
            .tree()
            .find_group(group_id)
            .ok_or(LayoutError::TabGroupNotFound { group_id })?;

        if group.tab_count() <= 1 && self.tab_groups.tree().all_group_ids().len() <= 1 {
            return Err(LayoutError::CannotEmptyEditor);
        }
        if tab_index >= group.tab_count() {
            return Err(LayoutError::TabIndexOutOfBounds {
                group_id,
                index: tab_index,
                count: group.tab_count(),
            });
        }

        let tab_id = group.tabs[tab_index].clone();
        let group_mut = self.tab_groups.tree_mut().find_group_mut(group_id).unwrap();
        group_mut.tabs.remove(tab_index);
        if group_mut.active_tab >= group_mut.tabs.len() && !group_mut.tabs.is_empty() {
            group_mut.active_tab = group_mut.tabs.len() - 1;
        }

        let window_id = self.floating_windows.create_window_at(
            &tab_id,
            position,
            Size::new(600.0, 400.0),
            DockZone::Center,
        )?;

        if let Some(window) = self.floating_windows.get_mut(window_id) {
            window.origin_tab_index = Some(tab_index);
        }

        self.sync_floating_state();
        self.personas.mark_modified();
        Ok(window_id)
    }

    /// Redock a tab from a floating window back to its originating TabGroup.
    pub fn redock_tab(&mut self, window_id: FloatingWindowId) -> Result<(), LayoutError> {
        let window = self
            .floating_windows
            .remove_window(window_id)
            .ok_or(LayoutError::FloatingWindowNotFound { window_id })?;

        let tab_id = window.panels.first().cloned().unwrap_or_default();
        let insert_index = window.origin_tab_index.unwrap_or(0);
        let active_group = self.tab_groups.active_group();

        // Add tab back to the active group at the original index
        if let Some(group) = self.tab_groups.tree_mut().find_group_mut(active_group) {
            let idx = insert_index.min(group.tabs.len());
            group.tabs.insert(idx, tab_id);
            group.active_tab = idx;
        }

        self.sync_floating_state();
        self.personas.mark_modified();
        Ok(())
    }

    /// Update a floating window's position and size after a move/resize.
    pub fn update_floating_window(
        &mut self,
        window_id: FloatingWindowId,
        position: Position,
        size: Size,
    ) -> Result<(), LayoutError> {
        self.floating_windows
            .update_window(window_id, position, size)?;
        self.sync_floating_state();
        Ok(())
    }

    /// Handle OS window close button — redock rather than destroy.
    pub fn on_floating_window_close(
        &mut self,
        window_id: FloatingWindowId,
    ) -> Result<CloseAction, LayoutError> {
        // In a full implementation, check for unsaved changes here.
        // For now, always redock.
        self.redock_panel(window_id)?;
        Ok(CloseAction::Redocked)
    }

    // ─── Tab Group Operations ───────────────────────────────────────────

    /// Split the active tab group horizontally (side-by-side).
    pub fn split_horizontal(&mut self) -> Result<TabGroupId, LayoutError> {
        let id = self.tab_groups.split_horizontal()?;
        self.sync_tab_state();
        self.personas.mark_modified();
        Ok(id)
    }

    /// Split the active tab group vertically (stacked).
    pub fn split_vertical(&mut self) -> Result<TabGroupId, LayoutError> {
        let id = self.tab_groups.split_vertical()?;
        self.sync_tab_state();
        self.personas.mark_modified();
        Ok(id)
    }

    /// Move a tab from one group to another at the specified index.
    pub fn move_tab(
        &mut self,
        source_group: TabGroupId,
        tab_index: usize,
        target_group: TabGroupId,
        insert_index: usize,
    ) -> Result<(), LayoutError> {
        self.tab_groups
            .move_tab(source_group, tab_index, target_group, insert_index)?;
        self.sync_tab_state();
        self.personas.mark_modified();
        Ok(())
    }

    /// Add a new tab to the active tab group (or specified group).
    pub fn add_tab(
        &mut self,
        tab_id: &str,
        target_group: Option<TabGroupId>,
    ) -> Result<(), LayoutError> {
        self.tab_groups.add_tab(tab_id, target_group)?;
        self.sync_tab_state();
        Ok(())
    }

    /// Returns the currently active tab group ID.
    pub fn active_tab_group(&self) -> TabGroupId {
        self.tab_groups.active_group()
    }

    /// Set the active tab group.
    pub fn set_active_tab_group(&mut self, group_id: TabGroupId) -> Result<(), LayoutError> {
        self.tab_groups.set_active_group(group_id)
    }

    // ─── Persona Operations ─────────────────────────────────────────────

    /// Activate a persona by name, transitioning the layout.
    ///
    /// Open documents are preserved (excess tabs placed in last group).
    pub fn activate_persona(&mut self, name: &str) -> Result<(), LayoutError> {
        let current_tabs: Vec<String> = self
            .tab_groups
            .tree()
            .all_tabs()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let target_layout = self.personas.activate(name)?.clone();

        // Apply the persona's layout structure
        self.state.docked_panels = target_layout.docked_panels;
        self.state.splitter_positions = target_layout.splitter_positions;
        self.state.panel_visibility = target_layout.panel_visibility;
        self.state.panel_display_states = target_layout.panel_display_states;

        // Rebuild tab groups from persona, preserving open tabs
        let mut new_tree = target_layout.tab_groups;
        let group_ids = new_tree.all_group_ids();

        if !current_tabs.is_empty() && !group_ids.is_empty() {
            // Place all current tabs into the last available group
            let last_group_id = *group_ids.last().unwrap();
            if let Some(group) = new_tree.find_group_mut(last_group_id) {
                group.tabs = current_tabs;
                group.active_tab = 0;
            }
        }

        let active_id = new_tree
            .all_group_ids()
            .into_iter()
            .next()
            .unwrap_or(TabGroupId::new(1));
        self.tab_groups = TabGroupManager::from_tree(new_tree.clone(), active_id);
        self.state.tab_groups = new_tree;
        self.sync_floating_state();
        Ok(())
    }

    /// Save the current layout as a custom persona.
    pub fn save_persona(&mut self, name: &str) -> Result<(), LayoutError> {
        self.sync_state();
        self.personas.save(name, self.state.clone());
        Ok(())
    }

    /// Delete a custom persona. Returns error for built-in personas.
    pub fn delete_persona(&mut self, name: &str) -> Result<(), LayoutError> {
        self.personas.delete(name)
    }

    /// Update the active persona to match the current layout.
    pub fn update_active_persona(&mut self) -> Result<(), LayoutError> {
        self.sync_state();
        self.personas.update_active(self.state.clone())
    }

    /// Revert the layout to the active persona's saved state.
    pub fn revert_to_persona(&mut self) -> Result<(), LayoutError> {
        let name = self
            .personas
            .active_persona_name()
            .ok_or(LayoutError::PersonaNotFound {
                name: "<none>".to_string(),
            })?
            .to_string();
        self.activate_persona(&name)
    }

    /// List all available personas (built-in and custom).
    pub fn list_personas(&self) -> &[Persona] {
        self.personas.list()
    }

    // ─── Serialization Operations ───────────────────────────────────────

    /// Serialize the current layout state to the session file.
    pub fn save_session(&self, path: &Path) -> Result<(), LayoutError> {
        serializer::save_to_file(&self.state, path)
    }

    /// Export the current layout state to a user-specified path.
    pub fn export_layout(&self, path: &Path) -> Result<(), LayoutError> {
        serializer::save_to_file(&self.state, path)
    }

    /// Import and apply a layout from a file.
    ///
    /// Missing panels are skipped gracefully.
    pub fn import_layout(&mut self, path: &Path) -> Result<(), LayoutError> {
        let imported = serializer::load_from_file(path)?;
        // Filter docked panels to only those registered
        let valid_panels: Vec<DockedPanelState> = imported
            .docked_panels
            .into_iter()
            .filter(|p| self.panel_registry.is_registered(&p.panel_id))
            .collect();

        self.state = LayoutState {
            docked_panels: valid_panels,
            ..imported
        };

        let active_id = self
            .state
            .tab_groups
            .all_group_ids()
            .into_iter()
            .next()
            .unwrap_or(TabGroupId::new(1));
        self.tab_groups = TabGroupManager::from_tree(self.state.tab_groups.clone(), active_id);
        self.floating_windows =
            FloatingWindowManager::from_windows(self.state.floating_windows.clone());
        self.personas.mark_modified();
        Ok(())
    }

    /// Reset to the built-in default layout.
    pub fn reset_to_default(&mut self) {
        self.state = LayoutState::default();
        self.tab_groups = TabGroupManager::new();
        self.floating_windows = FloatingWindowManager::new();
        self.personas.mark_modified();
    }

    // ─── Drag-and-Drop Operations ───────────────────────────────────────

    /// Begin a drag operation from a panel header or tab.
    pub fn begin_drag(&mut self, item: DragItem, origin: Position) {
        self.drag_drop.begin_drag(item, origin);
    }

    /// Update the drag position — triggers hit testing and indicator display.
    pub fn update_drag(&mut self, cursor: Position) {
        self.drag_drop.update_position(cursor);
    }

    /// End a drag operation — executes the drop or cancels.
    pub fn end_drag(&mut self, _cursor: Position) -> Result<DragResult, LayoutError> {
        self.drag_drop.complete();
        Ok(DragResult::Cancelled)
    }

    /// Cancel an in-progress drag operation.
    pub fn cancel_drag(&mut self) {
        self.drag_drop.cancel();
    }

    /// Returns the current drop indicator (for rendering by the shell).
    pub fn current_drop_indicator(&self) -> Option<&DropIndicator> {
        self.drag_drop.current_indicator()
    }

    /// Returns whether a drag is currently in progress.
    pub fn is_dragging(&self) -> bool {
        self.drag_drop.is_dragging()
    }

    // ─── Splitter Operations ────────────────────────────────────────────

    /// Begin dragging a splitter.
    pub fn begin_splitter_drag(&mut self, splitter_id: SplitterId) {
        let _ = self.splitters.begin_drag(splitter_id);
    }

    /// Update splitter position during drag (real-time resize).
    pub fn update_splitter(
        &mut self,
        splitter_id: SplitterId,
        new_proportion: f32,
    ) -> Result<(), LayoutError> {
        // Use a reasonable default total size; the shell passes the real value
        self.splitters
            .update_splitter(splitter_id, new_proportion, 1000.0)?;
        self.personas.mark_modified();
        Ok(())
    }

    /// End splitter drag — finalizes the position.
    pub fn end_splitter_drag(&mut self, splitter_id: SplitterId) {
        self.splitters.end_drag(splitter_id);
    }

    /// Reset a splitter to its default position (double-click).
    pub fn reset_splitter(&mut self, splitter_id: SplitterId) -> Result<(), LayoutError> {
        self.splitters.reset_splitter(splitter_id)?;
        self.personas.mark_modified();
        Ok(())
    }

    /// Handle primary window resize — proportional redistribution.
    pub fn on_window_resize(&mut self, new_size: Size) {
        self.splitters.on_window_resize(new_size);
    }

    // ─── Multi-Monitor Support ──────────────────────────────────────────

    /// Handle monitor disconnection — relocate affected windows.
    pub fn on_monitor_disconnected(&mut self, monitor_id: &str) {
        let window_ids: Vec<FloatingWindowId> = self
            .floating_windows
            .windows()
            .iter()
            .filter(|w| w.monitor_id.as_deref() == Some(monitor_id))
            .map(|w| w.id)
            .collect();

        // For now, just clear the monitor assignment.
        // The shell layer will handle actual repositioning.
        for id in window_ids {
            if let Some(window) = self.floating_windows.get_mut(id) {
                window.monitor_id = None;
            }
        }
        self.sync_floating_state();
    }

    /// Update a floating window's monitor assignment after a move.
    pub fn update_window_monitor(
        &mut self,
        window_id: FloatingWindowId,
        monitor_id: &str,
    ) -> Result<(), LayoutError> {
        let window = self
            .floating_windows
            .get_mut(window_id)
            .ok_or(LayoutError::FloatingWindowNotFound { window_id })?;
        window.monitor_id = Some(monitor_id.to_string());
        self.sync_floating_state();
        Ok(())
    }

    /// Validate window positions during startup restoration.
    ///
    /// Repositions windows with less than 50% visibility.
    pub fn validate_window_positions(&mut self, available_monitors: &[MonitorInfo]) {
        use crate::floating::monitor::{center_on_primary, is_window_sufficiently_visible};

        let window_count = self.floating_windows.count();
        for i in 0..window_count {
            let windows = self.floating_windows.windows();
            if i >= windows.len() {
                break;
            }
            let window = &windows[i];
            if !is_window_sufficiently_visible(window.position, window.size, available_monitors) {
                let window_size = window.size;
                let window_id = window.id;
                if let Some(new_pos) = center_on_primary(window_size, available_monitors) {
                    if let Some(w) = self.floating_windows.get_mut(window_id) {
                        w.position = new_pos;
                    }
                }
            }
        }
        self.sync_floating_state();
    }

    // ─── Internal Helpers ───────────────────────────────────────────────

    /// Synchronizes floating window state into the LayoutState.
    fn sync_floating_state(&mut self) {
        self.state.floating_windows = self.floating_windows.windows().to_vec();
    }

    /// Synchronizes tab group tree into the LayoutState.
    fn sync_tab_state(&mut self) {
        self.state.tab_groups = self.tab_groups.tree().clone();
    }

    /// Fully synchronizes all sub-manager states into the LayoutState.
    fn sync_state(&mut self) {
        self.sync_floating_state();
        self.sync_tab_state();
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_engine_with_panels() -> LayoutEngine {
        let mut engine = LayoutEngine::new();
        engine
            .panel_registry_mut()
            .register("file_tree", "File Tree", DockZone::Left)
            .unwrap();
        engine
            .panel_registry_mut()
            .register("output", "Output", DockZone::Bottom)
            .unwrap();
        engine
            .panel_registry_mut()
            .register("properties", "Properties", DockZone::Right)
            .unwrap();
        // Add to docked panels
        engine.state.docked_panels.push(DockedPanelState {
            panel_id: "file_tree".to_string(),
            zone: DockZone::Left,
            zone_dimension: 250.0,
        });
        engine.state.docked_panels.push(DockedPanelState {
            panel_id: "output".to_string(),
            zone: DockZone::Bottom,
            zone_dimension: 200.0,
        });
        engine
    }

    #[test]
    fn new_engine_has_default_state() {
        // Validates: Requirement 1 criterion 1
        let engine = LayoutEngine::new();
        assert_eq!(engine.current_state().schema_version, crate::SCHEMA_VERSION);
        assert_eq!(engine.floating_window_count(), 0);
        assert!(!engine.is_persona_modified());
    }

    #[test]
    fn show_panel_makes_hidden_panel_visible() {
        // Validates: Requirement 1 criterion 11
        let mut engine = setup_engine_with_panels();
        engine.hide_panel("file_tree").unwrap();
        assert!(!engine.current_state().is_panel_visible("file_tree"));
        engine.show_panel("file_tree").unwrap();
        assert!(engine.current_state().is_panel_visible("file_tree"));
    }

    #[test]
    fn hide_panel_makes_visible_panel_hidden() {
        // Validates: Requirement 1 criterion 11
        let mut engine = setup_engine_with_panels();
        engine.hide_panel("file_tree").unwrap();
        assert!(!engine.current_state().is_panel_visible("file_tree"));
    }

    #[test]
    fn toggle_panel_flips_visibility() {
        // Validates: Requirement 1 criterion 12
        let mut engine = setup_engine_with_panels();
        assert!(engine.current_state().is_panel_visible("file_tree"));
        engine.toggle_panel("file_tree").unwrap();
        assert!(!engine.current_state().is_panel_visible("file_tree"));
        engine.toggle_panel("file_tree").unwrap();
        assert!(engine.current_state().is_panel_visible("file_tree"));
    }

    #[test]
    fn panel_operations_reject_unregistered_panel() {
        let mut engine = LayoutEngine::new();
        assert!(matches!(
            engine.show_panel("nonexistent"),
            Err(LayoutError::PanelNotFound { .. })
        ));
        assert!(matches!(
            engine.hide_panel("nonexistent"),
            Err(LayoutError::PanelNotFound { .. })
        ));
        assert!(matches!(
            engine.toggle_panel("nonexistent"),
            Err(LayoutError::PanelNotFound { .. })
        ));
    }

    #[test]
    fn minimize_maximize_restore_panel() {
        // Validates: Requirement 1 criterion 13
        let mut engine = setup_engine_with_panels();
        engine.minimize_panel("file_tree").unwrap();
        assert_eq!(
            engine.current_state().panel_display_state("file_tree"),
            PanelDisplayState::Minimized
        );
        engine.maximize_panel("file_tree").unwrap();
        assert_eq!(
            engine.current_state().panel_display_state("file_tree"),
            PanelDisplayState::Maximized
        );
        engine.restore_panel("file_tree").unwrap();
        assert_eq!(
            engine.current_state().panel_display_state("file_tree"),
            PanelDisplayState::Normal
        );
    }

    #[test]
    fn undock_panel_creates_floating_window() {
        // Validates: Requirement 3 criterion 1
        let mut engine = setup_engine_with_panels();
        let window_id = engine.undock_panel("file_tree").unwrap();
        assert_eq!(engine.floating_window_count(), 1);
        assert!(engine
            .current_state()
            .floating_windows
            .iter()
            .any(|w| w.id == window_id));
        // Panel removed from docked
        assert!(engine
            .current_state()
            .find_docked_panel("file_tree")
            .is_none());
    }

    #[test]
    fn redock_panel_removes_floating_window() {
        // Validates: Requirement 3 criterion 5
        let mut engine = setup_engine_with_panels();
        let window_id = engine.undock_panel("file_tree").unwrap();
        engine.redock_panel(window_id).unwrap();
        assert_eq!(engine.floating_window_count(), 0);
        // Panel back in docked state
        assert!(engine
            .current_state()
            .find_docked_panel("file_tree")
            .is_some());
    }

    #[test]
    fn undock_panel_at_uses_specified_position() {
        // Validates: Requirement 3 criterion 9
        let mut engine = setup_engine_with_panels();
        let pos = Position::new(500.0, 300.0);
        let window_id = engine.undock_panel_at("file_tree", pos).unwrap();
        let window = engine
            .current_state()
            .floating_windows
            .iter()
            .find(|w| w.id == window_id)
            .unwrap();
        assert_eq!(window.position, pos);
    }

    #[test]
    fn undock_panel_enforces_max_windows() {
        // Validates: Requirement 3 criterion 14
        let mut engine = LayoutEngine::new();
        for i in 0..17 {
            let id = format!("panel_{i}");
            engine
                .panel_registry_mut()
                .register(&id, &format!("Panel {i}"), DockZone::Left)
                .unwrap();
            engine.state.docked_panels.push(DockedPanelState {
                panel_id: id.clone(),
                zone: DockZone::Left,
                zone_dimension: 200.0,
            });
        }
        for i in 0..16 {
            engine.undock_panel(&format!("panel_{i}")).unwrap();
        }
        let result = engine.undock_panel("panel_16");
        assert!(matches!(
            result,
            Err(LayoutError::MaxFloatingWindows { .. })
        ));
    }

    #[test]
    fn on_floating_window_close_redocks() {
        // Validates: Requirement 3 criterion 8
        let mut engine = setup_engine_with_panels();
        let window_id = engine.undock_panel("file_tree").unwrap();
        let result = engine.on_floating_window_close(window_id).unwrap();
        assert_eq!(result, CloseAction::Redocked);
        assert_eq!(engine.floating_window_count(), 0);
    }

    #[test]
    fn split_horizontal_preserves_tabs() {
        // Validates: Requirement 2 criterion 2
        let mut engine = LayoutEngine::new();
        engine.add_tab("main.rs", None).unwrap();
        engine.add_tab("lib.rs", None).unwrap();
        let total = engine.tab_groups().total_tab_count();
        engine.split_horizontal().unwrap();
        assert_eq!(engine.tab_groups().total_tab_count(), total);
    }

    #[test]
    fn activate_persona_changes_active_name() {
        // Validates: Requirement 5 criterion 9
        let mut engine = LayoutEngine::new();
        engine.activate_persona("Debug").unwrap();
        assert_eq!(engine.active_persona_name(), Some("Debug"));
    }

    #[test]
    fn save_and_activate_custom_persona() {
        // Validates: Requirement 5 criterion 3
        let mut engine = setup_engine_with_panels();
        engine.save_persona("My Layout").unwrap();
        engine.activate_persona("My Layout").unwrap();
        assert_eq!(engine.active_persona_name(), Some("My Layout"));
    }

    #[test]
    fn reset_to_default_clears_state() {
        // Validates: Requirement 6 criterion 8
        let mut engine = setup_engine_with_panels();
        engine.undock_panel("file_tree").unwrap();
        engine.reset_to_default();
        assert_eq!(engine.floating_window_count(), 0);
        assert!(engine.current_state().docked_panels.is_empty());
    }

    #[test]
    fn save_session_writes_toml_file() {
        // Validates: Requirement 6 criterion 1
        let engine = setup_engine_with_panels();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("layout.toml");
        engine.save_session(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn import_layout_applies_state() {
        // Validates: Requirement 6 criterion 7
        let engine = setup_engine_with_panels();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("layout.toml");
        engine.export_layout(&path).unwrap();

        let mut engine2 = LayoutEngine::new();
        engine2
            .panel_registry_mut()
            .register("file_tree", "File Tree", DockZone::Left)
            .unwrap();
        engine2
            .panel_registry_mut()
            .register("output", "Output", DockZone::Bottom)
            .unwrap();
        engine2.import_layout(&path).unwrap();
        assert_eq!(engine2.current_state().docked_panels.len(), 2);
    }

    #[test]
    fn undock_tab_from_group() {
        // Validates: Requirement 3 criterion 9
        let mut engine = LayoutEngine::new();
        engine.add_tab("main.rs", None).unwrap();
        engine.add_tab("lib.rs", None).unwrap();
        let group_id = engine.active_tab_group();
        let window_id = engine.undock_tab(group_id, 0).unwrap();
        assert_eq!(engine.floating_window_count(), 1);
    }

    #[test]
    fn undock_only_tab_in_only_group_returns_error() {
        // Validates: Requirement 9 criterion 4
        let mut engine = LayoutEngine::new();
        engine.add_tab("main.rs", None).unwrap();
        let group_id = engine.active_tab_group();
        let result = engine.undock_tab(group_id, 0);
        assert!(matches!(result, Err(LayoutError::CannotEmptyEditor)));
    }
}
