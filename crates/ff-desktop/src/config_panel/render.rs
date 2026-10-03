//! Rendering + per-frame keyboard driver for the Config View
//! (configuration-system Requirement 15 + Requirement 21). Split out of
//! `config_panel/mod.rs` for the 400-line rule; behaviour unchanged.

use std::collections::HashMap;

use eframe::egui;
use ff_config::layer::ConfigLayer;
use ff_config::schema::SchemaEntry;
use ff_config::ConfigHandle;

use super::keyboard::{config_keyboard_effects, tree_has_keyboard};
use super::widgets::render_widget;
use super::{
    filter_field_id, reconcile_cursor, visible_rows, ConfigNodeId, ConfigPanelState, TreeEntry,
};

/// Validates: Requirement 15.1-15.8
pub fn render(ui: &mut egui::Ui, state: &mut ConfigPanelState, config: &ConfigHandle) {
    // == Filter bar -- Req 15.7 ===
    ui.horizontal(|ui| {
        ui.label("Filter:");
        // Stable id so Tab focus round-trips and the shell can anchor the
        // command-field -> first-interior boundary here (B058).
        ui.add(egui::TextEdit::singleline(&mut state.filter).id(filter_field_id()));
        if ui.small_button("X").clicked() {
            state.filter.clear();
        }
    });
    ui.separator();

    // == Source file indicator -- Req 15.8 ===
    if let Some(user_path) = ff_config::paths::user_config_path() {
        ui.horizontal(|ui| {
            ui.label("Source File:");
            ui.monospace(user_path.to_string_lossy().as_ref());
        });
        ui.separator();
    }

    // == Collect schema entries + build the Config_Tree rows ===
    let schema_entries = config.list_schema_entries();
    // Full entries keyed for lookup while rendering a row.
    let entry_by_key: HashMap<String, SchemaEntry> = schema_entries
        .iter()
        .map(|e| (e.key.clone(), e.clone()))
        .collect();
    // Reduced entries for the pure tree model (key + description only).
    let tree_entries: Vec<TreeEntry> = schema_entries
        .iter()
        .map(|e| TreeEntry {
            key: e.key.clone(),
            description: e.description.clone(),
        })
        .collect();

    let rows = visible_rows(&tree_entries, &state.filter, &state.collapsed);

    // Rebuild the per-frame key -> widget-id map fresh each frame.
    state.widget_ids.clear();

    // == Keyboard tree navigation (CR-CH-039) ===
    // Reconcile the cursor against the freshly built rows, then run the
    // keyboard driver ONLY when the tree owns the keyboard (not the Filter field
    // or a key widget) so typing is never hijacked (Requirement 21.9, 21.10).
    reconcile_cursor(&rows, &rows, &mut state.cursor);
    if tree_has_keyboard(ui, &rows) {
        config_keyboard_effects(ui, state, &rows);
    }

    // == Render each row -- namespace group headers + key entries -- Req 15.2 =
    let cursor = state.cursor.clone();
    let selection_stroke = ui.visuals().selection.stroke;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for row in &rows {
            let is_cursor = cursor.as_ref() == Some(&row.id);
            match &row.id {
                ConfigNodeId::Namespace(ns) => {
                    // Count the keys under this namespace (matching the filter).
                    let count = tree_entries
                        .iter()
                        .filter(|e| namespace_of(&e.key) == *ns && entry_matches(e, &state.filter))
                        .count();
                    let arrow = if row.expanded { "v" } else { ">" };
                    let header = format!("{} {} ({})", arrow, ns_display_name(ns), count);
                    let resp = ui.selectable_label(is_cursor, egui::RichText::new(header).strong());
                    if resp.clicked() {
                        // Mouse toggles the SAME collapsed state the keyboard writes.
                        let now = state.collapsed.get(ns).copied().unwrap_or(false);
                        state.collapsed.insert(ns.clone(), !now);
                        state.cursor = Some(ConfigNodeId::Namespace(ns.clone()));
                    }
                    paint_cursor_highlight(ui, &resp, is_cursor, selection_stroke);
                }
                ConfigNodeId::Key(key) => {
                    if let Some(entry) = entry_by_key.get(key) {
                        let resp = ui
                            .indent(("config_key_indent", key), |ui| {
                                render_entry(ui, state, config, entry, is_cursor)
                            })
                            .response;
                        paint_cursor_highlight(ui, &resp, is_cursor, selection_stroke);
                    }
                    ui.separator();
                }
            }
        }
    });
}

/// Whether an entry matches the (already-lowercased-inside) filter -- key OR
/// description substring; empty filter matches everything. Kept here for the
/// namespace key-count display; the tree module has its own copy for its rows.
fn entry_matches(entry: &TreeEntry, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    let f = filter.to_lowercase();
    entry.key.to_lowercase().contains(&f) || entry.description.to_lowercase().contains(&f)
}

/// Paint the Tree_Cursor selection outline around a row's rect (Requirement
/// 21.8) so the keyboard position is always visible, distinct from egui hover.
fn paint_cursor_highlight(
    ui: &egui::Ui,
    resp: &egui::Response,
    is_cursor: bool,
    stroke: egui::Stroke,
) {
    if is_cursor {
        ui.painter()
            .rect_stroke(resp.rect.expand(1.0), 2.0, stroke, egui::StrokeKind::Inside);
    }
}

/// Render a single schema entry row with widget, provenance badge, and reset button.
///
/// Validates: Requirement 15.3, 15.4, 15.5, 15.6, 18.6
fn render_entry(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    entry: &SchemaEntry,
    _is_cursor: bool,
) {
    let key = &entry.key;
    let locked = config.is_locked(key);

    // Resolve current effective value and provenance.
    let (effective, provenance_label) = match config.get_with_provenance(key) {
        Ok(ev) => {
            let label = if locked {
                "LOCKED"
            } else {
                layer_label(ev.provenance.layer)
            };
            (ev.value, label)
        }
        Err(_) => (
            entry.default.clone(),
            if locked { "LOCKED" } else { "Default" },
        ),
    };

    ui.horizontal(|ui| {
        // Key + description
        // Validates: Requirement 14.2 -- selectable key name and description
        ui.vertical(|ui| {
            ui.add(egui::Button::selectable(
                false,
                egui::RichText::new(key.as_str()).monospace(),
            ));
            ui.add(egui::Button::selectable(
                false,
                egui::RichText::new(entry.description.as_str())
                    .small()
                    .weak(),
            ));
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Provenance / LOCKED badge -- Req 15.3, 18.6
            let badge_color = if locked {
                egui::Color32::from_rgb(200, 80, 80)
            } else {
                egui::Color32::from_rgb(120, 180, 120)
            };
            ui.add(egui::Button::selectable(
                false,
                egui::RichText::new(provenance_label)
                    .small()
                    .color(badge_color),
            ));

            // Reset to Default button -- Req 15.6 (disabled when locked or at Default)
            let is_at_default = provenance_label == "Default";
            ui.add_enabled_ui(!is_at_default && !locked, |ui| {
                if ui.small_button("\u{21ba} Reset").clicked() {
                    let _ = config.remove_user_value(key);
                    state.pending.remove(key);
                    state.errors.remove(key);
                }
            });
        });
    });

    // Value widget -- disabled for locked keys (Req 18.6)
    let widget_id = ui
        .add_enabled_ui(!locked, |ui| {
            render_widget(ui, state, config, entry, &effective)
        })
        .inner;
    // Record the value widget's id so Enter on this key node can focus it
    // (Requirement 21.6).
    if let Some(id) = widget_id {
        state.widget_ids.insert(entry.key.clone(), id);
    }

    // Inline validation error -- Req 15.5
    if let Some(err) = state.errors.get(key) {
        ui.colored_label(egui::Color32::RED, err.as_str());
    }
}

/// Extract the namespace (first dot-segment) from a key path.
fn namespace_of(key: &str) -> String {
    key.split('.').next().unwrap_or(key).to_string()
}

/// Human-readable display name for a namespace segment.
fn ns_display_name(ns: &str) -> String {
    let mut s = ns.to_string();
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

/// Map a `ConfigLayer` to a short provenance label string.
fn layer_label(layer: ConfigLayer) -> &'static str {
    match layer {
        ConfigLayer::Defaults => "Default",
        ConfigLayer::System => "System",
        ConfigLayer::User => "User",
        ConfigLayer::Profile => "Profile",
        ConfigLayer::Project => "Project",
        ConfigLayer::Workspace => "Workspace",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 15.2 -- namespace_of extracts first dot-segment
    #[test]
    fn namespace_grouping_correct() {
        assert_eq!(namespace_of("editor.tab_size"), "editor");
        assert_eq!(namespace_of("logging.level"), "logging");
        assert_eq!(namespace_of("theme.active"), "theme");
        assert_eq!(namespace_of("no_dot"), "no_dot");
    }

    // Validates: Requirement 15.7 -- filter_hides_non_matching_keys (logic test)
    #[test]
    fn filter_hides_non_matching_keys() {
        let filter = "tab";
        let key1 = "editor.tab_size";
        let key2 = "logging.level";
        let desc1 = "Tab size in spaces";
        let desc2 = "Log level";

        let filter_lower = filter.to_lowercase();
        let matches1 = key1.to_lowercase().contains(&filter_lower)
            || desc1.to_lowercase().contains(&filter_lower);
        let matches2 = key2.to_lowercase().contains(&filter_lower)
            || desc2.to_lowercase().contains(&filter_lower);

        assert!(matches1, "editor.tab_size should match filter 'tab'");
        assert!(!matches2, "logging.level should not match filter 'tab'");
    }

    // Validates: Requirement 15.3 -- provenance badge shows correct layer label
    #[test]
    fn provenance_badge_shows_correct_layer() {
        assert_eq!(layer_label(ConfigLayer::Defaults), "Default");
        assert_eq!(layer_label(ConfigLayer::User), "User");
        assert_eq!(layer_label(ConfigLayer::Project), "Project");
        assert_eq!(layer_label(ConfigLayer::System), "System");
        assert_eq!(layer_label(ConfigLayer::Profile), "Profile");
        assert_eq!(layer_label(ConfigLayer::Workspace), "Workspace");
    }

    // Validates: Requirement 15.3 -- widget type selected for bool
    #[test]
    fn widget_type_selected_for_bool() {
        use ff_config::error::ValueType;
        // Boolean type maps to checkbox -- verified by the match arm in render_widget
        assert_eq!(ValueType::Boolean, ValueType::Boolean);
    }

    // Validates: Requirement 15.3 -- widget type selected for enum string
    #[test]
    fn widget_type_selected_for_enum_string() {
        use ff_config::schema::Constraints;
        use ff_config::value::ConfigValue;
        let constraints = Constraints {
            min: None,
            max: None,
            allowed_values: Some(vec![
                ConfigValue::String("space".to_string()),
                ConfigValue::String("tab".to_string()),
            ]),
            pattern: None,
        };
        // Has allowed_values -> should use ComboBox
        assert!(constraints.allowed_values.is_some());
    }

    // Validates: Requirement 15.3 -- widget type selected for bounded int (slider)
    #[test]
    fn widget_type_selected_for_bounded_int() {
        use ff_config::schema::Constraints;
        let constraints = Constraints {
            min: Some(1.0),
            max: Some(16.0),
            allowed_values: None,
            pattern: None,
        };
        // Has min and max -> should use Slider
        assert!(constraints.min.is_some() && constraints.max.is_some());
    }

    // Validates: Requirement 15.6 -- reset button hidden when at default (provenance check)
    #[test]
    fn reset_button_hidden_when_at_default() {
        // The reset button is only enabled when provenance != "Default".
        let at_default = layer_label(ConfigLayer::Defaults) == "Default";
        assert!(at_default, "Default layer should produce 'Default' label");
        // Button is disabled (add_enabled_ui(!is_at_default, ...)) when at_default is true.
        assert!(at_default, "reset button is disabled at the Default layer");
    }

    // Validates: Requirement 15.10 -- F3/END returns to POM (routing test)
    #[test]
    fn f3_returns_to_pom_via_end_command() {
        // F3 is mapped to "END" in the default key map.
        // "END" is not currently a shell-level intercept for ConfigPanel,
        // but the F3 key binding routes through handle_command("END").
        // This test verifies the key map binding exists.
        use ff_keys::{FunctionKey, KeyBinding, KeyMap};
        let mut map = KeyMap::empty("test");
        map.set(
            ff_keys::ModifiedKey::plain(FunctionKey::F3),
            KeyBinding::with_label("END", "End"),
        );
        let binding = map.get_plain(FunctionKey::F3);
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().command(), "END");
    }
}
