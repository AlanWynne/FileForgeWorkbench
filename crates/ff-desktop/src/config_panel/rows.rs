//! Per-key row rendering for the Config View tree (configuration-system
//! Requirement 22). Split out of `config_panel/render.rs` for the 400-line rule;
//! behaviour unchanged. Holds the single key-row renderer (`render_key_row`),
//! the Tree_Cursor highlight painter, and the provenance-layer label mapper.

use eframe::egui;
use ff_config::layer::ConfigLayer;
use ff_config::schema::SchemaEntry;
use ff_config::ConfigHandle;

use super::layout::{aligned_left_segment, child_left_label};
use super::widgets::render_widget;
use super::ConfigPanelState;

/// Paint the Tree_Cursor selection outline around a row's rect (Requirement
/// 21.8) so the keyboard position is always visible, distinct from egui hover.
pub(super) fn paint_cursor_highlight(
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

/// Render a single schema entry key row under a group: line 1 is the
/// globally-aligned indented key + ":" + value widget (the key a frameless
/// selectable monospace control), line 2 is the selectable description plus the
/// right-to-left provenance/LOCKED badge, staged `*` marker, and Reset block.
///
/// Validates: Requirement 22.4, 22.5, 22.9; Requirement 14.2, 15.3-15.6, 18.6
pub(super) fn render_key_row(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    entry: &SchemaEntry,
    column: usize,
    is_cursor: bool,
) -> egui::Response {
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

    ui.vertical(|ui| {
        // -- Line 1: globally-aligned indented key -> `:` -> value -----------
        ui.horizontal(|ui| {
            // The key stays a SELECTABLE control (Req 14.2 / AC 22.9). Frameless
            // + monospace so char-count padding maps to pixels and the `:` lands
            // at the GLOBAL Value_Column on every row. The one-level indent is
            // baked into the left-label (AC 22.4).
            let segment = aligned_left_segment(&child_left_label(key), column);
            let _ = ui.selectable_label(is_cursor, egui::RichText::new(segment).monospace());

            // Value widget starts at the Value_Column -- disabled for locked
            // keys (Req 18.6). Its id is recorded so Enter can focus it (Req 21.6).
            let widget_id = ui
                .add_enabled_ui(!locked, |ui| {
                    render_widget(ui, state, config, entry, &effective)
                })
                .inner;
            if let Some(id) = widget_id {
                state.widget_ids.insert(entry.key.clone(), id);
            }
        });

        // -- Line 2: selectable description + right-to-left provenance block --
        ui.horizontal(|ui| {
            // Visible, SELECTABLE description (Req 14.2) -- left-aligned.
            ui.add(egui::Button::selectable(
                false,
                egui::RichText::new(entry.description.as_str())
                    .small()
                    .weak(),
            ));

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

                // A staged (unsaved) edit on this key shows an asterisk after
                // the provenance badge (Req 11.9).
                if state.working.contains_key(key) {
                    ui.add(egui::Button::selectable(
                        false,
                        egui::RichText::new("*")
                            .small()
                            .color(egui::Color32::from_rgb(220, 180, 100)),
                    ));
                }

                // Reset to Default button -- Req 15.6 (disabled when locked or at
                // Default WITH no staged edit).
                let is_at_default = provenance_label == "Default";
                let has_staged_edit = state.working.contains_key(key);
                let reset_enabled = !locked && (!is_at_default || has_staged_edit);
                ui.add_enabled_ui(reset_enabled, |ui| {
                    if ui.small_button("\u{21ba} Reset").clicked() {
                        // Stage a Reset into the working buffer (Req 11.2).
                        state.stage_reset(key);
                    }
                });
            });
        });

        // Inline validation error -- Req 15.5
        if let Some(err) = state.errors.get(key) {
            ui.colored_label(egui::Color32::RED, err.as_str());
        }
    })
    .response
}

/// Map a `ConfigLayer` to a short provenance label string.
pub(super) fn layer_label(layer: ConfigLayer) -> &'static str {
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

    // Validates: Requirement 15.6 -- reset button hidden when at default (provenance check)
    #[test]
    fn reset_button_hidden_when_at_default() {
        let at_default = layer_label(ConfigLayer::Defaults) == "Default";
        assert!(at_default, "Default layer should produce 'Default' label");
    }
}
