//! Pure render for the Keys Editor Context (function-keys Requirement 22,
//! CR-CH-029), modelled on `menus_editor_panel::render`.
//!
//! Returns a [`KeysEditorAction`]; the shell applies side effects (select kind /
//! save / reset). Command text fields bind DIRECTLY to the mutable working rows
//! so edits persist across frames (mirroring the Menus editor B054 fix).

use super::state::{KeysEditorAction, KeysEditorState, KIND_NAMES};
use ff_keys::{KeyModifier, ModifiedKey};

/// `WorkspaceContext` impl (CR-NR-078): render the Keys Editor, stash the
/// produced [`KeysEditorAction`] on `pending_action` for the shell to apply, and
/// report interior focus: FIRST = the workspace-kind selector combo (captured on
/// `first_interior_id`), LAST = the Save button (a stable, always-present id).
///
/// Validates: function-keys-and-history Requirement 22.7; workspace-framework
/// Requirement 1.4, 1.5, 6.1.
impl crate::shell::workspace_context::WorkspaceContext for KeysEditorState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        _services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        self.pending_action = render(ui, self);
        crate::shell::workspace_context::InteriorFocus {
            first: self.first_interior_id,
            last: self.last_interior_id.or(self.first_interior_id),
        }
    }
}

/// Render the Keys Editor Context, returning the action to apply.
///
/// Tab focus order is egui-native: the kind dropdown is created first (its fresh
/// combo id is reported as the first interior), then the editable grid, then the
/// Save/Reset footer buttons in visual order.
///
/// Validates: function-keys-and-history Requirement 22.1, 22.2, 22.3.
pub fn render(ui: &mut egui::Ui, state: &mut KeysEditorState) -> KeysEditorAction {
    let mut action = KeysEditorAction::None;

    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Keys Editor").strong().size(14.0));
    });
    ui.add_space(4.0);

    // --- Workspace-kind selector (the "name" that links the key list) -----
    // Reset the reported first-interior id each frame; the combo below sets it.
    state.first_interior_id = None;
    ui.horizontal(|ui| {
        ui.label("Workspace kind:");
        let current = state
            .selected_kind
            .clone()
            .unwrap_or_else(|| "(select)".to_string());
        let combo = egui::ComboBox::from_id_salt("keys_editor_kind")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for kind in KIND_NAMES {
                    if ui
                        .selectable_label(state.selected_kind.as_deref() == Some(*kind), *kind)
                        .clicked()
                    {
                        action = KeysEditorAction::SelectKind((*kind).to_string());
                    }
                }
            });
        // The kind selector is the FIRST focusable interior control; report its
        // FRESH id so the shell Boundary_Policy focuses it on Tab from the
        // command field (Requirement 22.7).
        state.first_interior_id = Some(combo.response.id);
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::from_rgb(0xC8, 0x8A, 0x00), err);
    }

    if state.selected_kind.is_none() {
        ui.label("Select a workspace kind to edit its key list.");
        return action;
    }

    ui.separator();
    // Fixed widths for the key label and each command field. A `TextEdit`'s
    // `desired_width` is only honoured reliably inside a plain horizontal
    // layout; inside an `egui::Grid` nested in a horizontal `ScrollArea` the
    // cells collapse to a few characters (and the grid stretches its last
    // column), so we lay each row out manually instead. FIELD_WIDTH comfortably
    // shows at least 16 characters at the default font.
    const KEY_LABEL_WIDTH: f32 = 44.0;
    const FIELD_WIDTH: f32 = 220.0;

    ui.horizontal(|ui| {
        ui.add_sized([KEY_LABEL_WIDTH, 0.0], egui::Label::new(""));
        for header in ["Base command", "Shift", "Ctrl", "Alt"] {
            ui.add_sized(
                [FIELD_WIDTH, 0.0],
                egui::Label::new(egui::RichText::new(header).strong()),
            );
        }
    });

    // --- Editable grid: one row per F1-F12, four modifier-layer commands ---
    egui::ScrollArea::both()
        .id_salt("keys_editor_grid")
        .max_height(360.0)
        .show(ui, |ui| {
            for row in state.rows.iter_mut() {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [KEY_LABEL_WIDTH, 0.0],
                        egui::Label::new(row.key.display_name()),
                    );
                    for (i, layer) in [
                        KeyModifier::None,
                        KeyModifier::Shift,
                        KeyModifier::Ctrl,
                        KeyModifier::Alt,
                    ]
                    .iter()
                    .enumerate()
                    {
                        let mk = ModifiedKey {
                            key: row.key,
                            modifier: *layer,
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut row.commands[i])
                                .id(egui::Id::new(("keys_editor_cmd", mk.toml_name())))
                                .desired_width(FIELD_WIDTH)
                                .hint_text("unassigned"),
                        );
                    }
                });
            }
        });

    ui.separator();
    ui.horizontal(|ui| {
        // Save is the LAST interior control; capture its FRESH response id so
        // the WorkspaceContext reports it as `InteriorFocus.last`, anchoring the
        // Shift+Tab reverse boundary on a known widget.
        let save = ui.add(egui::Button::new("Save").min_size(egui::vec2(60.0, 0.0)));
        state.last_interior_id = Some(save.id);
        if save.clicked() {
            action = KeysEditorAction::Save;
        }
        if ui.button("Reset to default").clicked() {
            action = KeysEditorAction::Reset;
        }
        ui.label(
            egui::RichText::new("Saves to keymaps/<kind>.toml")
                .weak()
                .small(),
        );
    });
    action
}
