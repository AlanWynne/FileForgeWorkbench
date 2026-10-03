//! Per-frame keyboard driver for the Config View (configuration-system
//! Requirement 21). Split out of `config_panel/render.rs` for the 400-line rule;
//! behaviour unchanged.

use eframe::egui;

use super::{
    filter_field_id, reduce_config_key, ConfigNodeId, ConfigPanelState, ConfigRow,
    ConfigTreeEffect, ConfigTreeKey,
};

/// Stable egui id of a configuration key's value-editing widget, so Enter on a
/// key node (Requirement 21.6) can move focus straight to it.
fn key_widget_id(key: &str) -> egui::Id {
    egui::Id::new(("config_panel_key_widget", key))
}

/// Whether the Config_Tree currently owns keyboard focus: true unless the Filter
/// field or one of the per-key value widgets is focused. Arrow-key tree
/// navigation is active ONLY in this state (Requirement 21.9), so typing in the
/// Filter field or a text value is never hijacked.
pub(super) fn tree_has_keyboard(ui: &egui::Ui, rows: &[ConfigRow]) -> bool {
    let focused = ui.memory(|m| m.focused());
    let Some(focused) = focused else {
        // Nothing focused -> the tree may drive navigation (matches the File
        // Navigator, whose arrows act whenever a node is focused / none else is).
        return true;
    };
    if focused == filter_field_id() {
        return false;
    }
    // A key's value widget holding focus means the user is editing a value.
    !rows.iter().any(|r| match &r.id {
        ConfigNodeId::Key(k) => focused == key_widget_id(k),
        ConfigNodeId::Namespace(_) => false,
    })
}

/// Translate this frame's keyboard input into a Config_Tree gesture, run it
/// through the pure [`reduce_config_key`] reducer against the built rows +
/// cursor, and apply the resulting [`ConfigTreeEffect`] (expand/collapse the
/// shared `collapsed` map, or request focus on a key widget). Call once per
/// frame while the tree owns the keyboard (see [`tree_has_keyboard`]).
///
/// Validates: Requirement 21.3-21.7, 21.9.
pub(super) fn config_keyboard_effects(
    ui: &egui::Ui,
    state: &mut ConfigPanelState,
    rows: &[ConfigRow],
) {
    let gesture = ui.input(|i| {
        if i.key_pressed(egui::Key::ArrowDown) {
            Some(ConfigTreeKey::Down)
        } else if i.key_pressed(egui::Key::ArrowUp) {
            Some(ConfigTreeKey::Up)
        } else if i.key_pressed(egui::Key::ArrowRight) {
            Some(ConfigTreeKey::Right)
        } else if i.key_pressed(egui::Key::ArrowLeft) {
            Some(ConfigTreeKey::Left)
        } else if i.key_pressed(egui::Key::Enter) {
            Some(ConfigTreeKey::Enter)
        } else if i.key_pressed(egui::Key::Home) {
            Some(ConfigTreeKey::Home)
        } else if i.key_pressed(egui::Key::End) {
            Some(ConfigTreeKey::End)
        } else {
            None
        }
    });
    let Some(gesture) = gesture else {
        return;
    };
    match reduce_config_key(rows, &mut state.cursor, gesture) {
        ConfigTreeEffect::None => {}
        ConfigTreeEffect::Expand(ns) => {
            state.collapsed.insert(ns, false);
        }
        ConfigTreeEffect::Collapse(ns) => {
            state.collapsed.insert(ns, true);
        }
        ConfigTreeEffect::FocusKeyWidget(key) => {
            // Prefer the exact widget id captured while rendering the key's value
            // widget last frame (works for any widget type); fall back to the
            // derived id. Widget ids are stable frame-to-frame.
            let id = state
                .widget_ids
                .get(&key)
                .copied()
                .unwrap_or_else(|| key_widget_id(&key));
            ui.memory_mut(|m| m.request_focus(id));
        }
    }
}
