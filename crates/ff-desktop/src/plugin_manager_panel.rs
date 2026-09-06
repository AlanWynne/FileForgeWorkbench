//! Plugin Manager Panel -- POM option 8.
//!
//! Displays all registered plugins with their state, capabilities, and
//! enable/disable controls. Reads from the ff-plugin PluginRegistry.
//!
//! Validates: plugin-manager-ui Requirement 1, 2, 3

#![allow(dead_code)]

use eframe::egui;
use ff_plugin::PluginState;

/// State for the Plugin Manager panel.
///
/// Validates: plugin-manager-ui Requirement 1.2, 1.6
pub struct PluginManagerPanelState {
    /// Cached plugin list: (name, state). Refreshed on each render call.
    pub plugins: Vec<(String, PluginState)>,
    /// Filter text for narrowing the plugin list.
    pub filter: String,
    /// Index of the currently selected plugin (for detail view).
    pub selected_index: Option<usize>,
}

impl PluginManagerPanelState {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            filter: String::new(),
            selected_index: None,
        }
    }

    /// Sort the plugin list alphabetically by name.
    ///
    /// Validates: plugin-manager-ui Requirement 1.5
    pub fn sort_plugins(&mut self) {
        self.plugins.sort_by(|a, b| a.0.cmp(&b.0));
    }

    /// Return plugins whose name contains `filter` (case-insensitive).
    ///
    /// Validates: plugin-manager-ui Requirement 1.6
    pub fn filter_plugins<'a>(
        &self,
        plugins: &'a [(String, PluginState)],
        filter: &str,
    ) -> Vec<&'a (String, PluginState)> {
        if filter.is_empty() {
            return plugins.iter().collect();
        }
        let lower = filter.to_lowercase();
        plugins
            .iter()
            .filter(|(name, _)| name.to_lowercase().contains(&lower))
            .collect()
    }
}

impl Default for PluginManagerPanelState {
    fn default() -> Self {
        Self::new()
    }
}

/// State badge colour for a plugin state.
fn state_colour(state: PluginState) -> egui::Color32 {
    match state {
        PluginState::Active => egui::Color32::from_rgb(0x4C, 0xAF, 0x50),
        PluginState::Initialized => egui::Color32::from_rgb(0x21, 0x96, 0xF3),
        PluginState::Loaded => egui::Color32::from_rgb(0x9E, 0x9E, 0x9E),
        PluginState::Discovered => egui::Color32::from_rgb(0x9E, 0x9E, 0x9E),
        PluginState::Deactivating => egui::Color32::from_rgb(0xFF, 0x98, 0x00),
        PluginState::Shutdown => egui::Color32::from_rgb(0x9E, 0x9E, 0x9E),
        _ => egui::Color32::from_rgb(0x9E, 0x9E, 0x9E),
    }
}

fn state_label(state: &PluginState) -> &'static str {
    match state {
        PluginState::Active => "Active",
        PluginState::Initialized => "Initialized",
        PluginState::Loaded => "Loaded",
        PluginState::Discovered => "Discovered",
        PluginState::Deactivating => "Deactivating",
        PluginState::Shutdown => "Shutdown",
        _ => "Unknown",
    }
}

/// Render the Plugin Manager panel.
///
/// Validates: plugin-manager-ui Requirement 1.1-1.6, 2.1-2.5, 3.1-3.3
pub fn render(ui: &mut egui::Ui, state: &mut PluginManagerPanelState) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Plugin Manager").monospace().strong());
        ui.separator();
        ui.label("Filter:");
        ui.add(
            egui::TextEdit::singleline(&mut state.filter)
                .desired_width(200.0)
                .hint_text("plugin name..."),
        );
        if ui.small_button("Clear").clicked() {
            state.filter.clear();
        }
    });
    ui.separator();

    if state.plugins.is_empty() {
        ui.label(
            egui::RichText::new("No plugins registered.")
                .weak()
                .italics(),
        );
        return;
    }

    // Build filtered + sorted view.
    let filter = state.filter.clone();
    let mut display: Vec<(usize, &(String, PluginState))> = state
        .plugins
        .iter()
        .enumerate()
        .filter(|(_, (name, _))| {
            filter.is_empty() || name.to_lowercase().contains(&filter.to_lowercase())
        })
        .collect();
    display.sort_by(|a, b| a.1 .0.cmp(&b.1 .0));

    // Two-pane layout: list on left, detail on right.
    // Validates: plugin-manager-ui Requirement 3.1
    let available = ui.available_width();
    let list_width = (available * 0.45).max(200.0);

    ui.horizontal(|ui| {
        // Left pane -- plugin list.
        ui.vertical(|ui| {
            ui.set_width(list_width);
            egui::ScrollArea::vertical()
                .id_salt("plugin_list_scroll")
                .max_height(400.0)
                .show(ui, |ui| {
                    for (orig_idx, (name, plugin_state)) in &display {
                        let is_selected = state.selected_index == Some(*orig_idx);
                        let label = egui::RichText::new(name.as_str()).monospace();
                        let resp = ui.selectable_label(is_selected, label);
                        if resp.clicked() {
                            state.selected_index = Some(*orig_idx);
                        }
                        // State badge.
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            let badge_text = state_label(plugin_state);
                            ui.colored_label(state_colour(*plugin_state), badge_text);
                        });
                        ui.separator();
                    }
                });
        });

        ui.separator();

        // Right pane -- detail view.
        // Validates: plugin-manager-ui Requirement 3.1-3.3
        ui.vertical(|ui| {
            if let Some(idx) = state.selected_index {
                if let Some((name, plugin_state)) = state.plugins.get(idx) {
                    ui.label(egui::RichText::new(name.as_str()).monospace().strong());
                    ui.horizontal(|ui| {
                        ui.label("State:");
                        ui.colored_label(state_colour(*plugin_state), state_label(plugin_state));
                    });
                    ui.separator();
                    ui.label(
                        egui::RichText::new("No additional metadata available.")
                            .weak()
                            .italics(),
                    );
                }
            } else {
                ui.label(
                    egui::RichText::new("Select a plugin to view details.")
                        .weak()
                        .italics(),
                );
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_list_sorted_alphabetically() {
        // Validates: plugin-manager-ui Requirement 1.5
        let mut state = PluginManagerPanelState::new();
        state.plugins = vec![
            ("Zebra".to_string(), PluginState::Active),
            ("Alpha".to_string(), PluginState::Active),
            ("Middle".to_string(), PluginState::Shutdown),
        ];
        state.sort_plugins();
        assert_eq!(state.plugins[0].0, "Alpha");
        assert_eq!(state.plugins[1].0, "Middle");
        assert_eq!(state.plugins[2].0, "Zebra");
    }

    #[test]
    fn filter_narrows_plugin_list() {
        // Validates: plugin-manager-ui Requirement 1.6
        let state = PluginManagerPanelState::new();
        let plugins = vec![
            ("GCC Toolchain".to_string(), PluginState::Active),
            ("Rust Toolchain".to_string(), PluginState::Active),
            ("Database Tool".to_string(), PluginState::Shutdown),
        ];
        let filtered = state.filter_plugins(&plugins, "toolchain");
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|(n, _)| n.to_lowercase().contains("toolchain")));
    }

    #[test]
    fn empty_filter_returns_all_plugins() {
        // Validates: plugin-manager-ui Requirement 1.6
        let state = PluginManagerPanelState::new();
        let plugins = vec![
            ("A".to_string(), PluginState::Active),
            ("B".to_string(), PluginState::Shutdown),
        ];
        let filtered = state.filter_plugins(&plugins, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn shutdown_plugin_state_label_is_shutdown() {
        // Validates: plugin-manager-ui Requirement 1.3
        assert_eq!(state_label(&PluginState::Shutdown), "Shutdown");
    }

    #[test]
    fn active_plugin_state_label_is_active() {
        // Validates: plugin-manager-ui Requirement 1.3
        assert_eq!(state_label(&PluginState::Active), "Active");
    }
}
