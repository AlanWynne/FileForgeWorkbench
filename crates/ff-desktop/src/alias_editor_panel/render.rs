//! Pure render for the Alias Editor (command-configurator Req 5, CR-NR-113).
//!
//! Returns an [`AliasEditorAction`]; the shell applies structural side effects
//! (add / delete / save). Per-field edits (surface / root / environment /
//! template) are bound DIRECTLY to the mutable working rows so edits persist
//! across frames (mirroring the Menus Editor B054); they do not produce actions.

use super::state::{AliasEditorAction, AliasEditorState};
use crate::shell::user_aliases::AliasEnvironment;

/// Stable egui id of the first interior control when the editor is EMPTY (the
/// "Add alias" button), so the focus contract has a guaranteed-present first
/// stop even with no rows (Req 5.10).
fn add_button_id() -> egui::Id {
    egui::Id::new("alias_editor_add_row")
}

/// Stable egui id of row `i`'s surface field (the first per-row control).
fn surface_id(i: usize) -> egui::Id {
    egui::Id::new(("alias_surface", i))
}

/// `WorkspaceContext` impl (CR-NR-078): render the Alias Editor, stash the
/// produced [`AliasEditorAction`] on `pending_action` for the shell to apply, and
/// report interior focus: FIRST = the first row's surface field (or the Add
/// button when empty), LAST = the stable Save button.
///
/// Validates: command-configurator Requirement 5.3, 5.10.
impl crate::shell::workspace_context::WorkspaceContext for AliasEditorState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        _services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        self.pending_action = render(ui, self);
        crate::shell::workspace_context::InteriorFocus {
            first: self.first_interior_id,
            last: self.last_interior_id,
        }
    }
}

/// Render the Alias Editor Context, returning the action to apply.
///
/// Tab focus order is egui-native: every interactive control is created in
/// visual order (each row's surface -> root -> environment -> template ->
/// delete, then the Add and Save footer), so egui's built-in Tab traversal walks
/// them in that exact order. The first row's surface field (or the Add button
/// when there are no rows) is reported as the first interior stop.
///
/// Validates: command-configurator Requirement 5.3, 5.5, 5.10.
pub fn render(ui: &mut egui::Ui, state: &mut AliasEditorState) -> AliasEditorAction {
    let mut action = AliasEditorAction::None;

    // Reset the reported first-interior id each frame; set below.
    state.first_interior_id = None;

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::from_rgb(0xD2, 0x0F, 0x39), err);
    }

    // Column header row.
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Alias").strong());
        ui.label(egui::RichText::new("Root").strong());
        ui.label(egui::RichText::new("Env").strong());
        ui.label(egui::RichText::new("Template").strong());
    });
    ui.separator();

    let row_count = state.rows.len();
    let footer_reserve = 72.0;
    let rows_height = (ui.available_height() - footer_reserve).max(64.0);
    egui::ScrollArea::both()
        .id_salt("alias_editor_rows")
        .auto_shrink([false, false])
        .max_height(rows_height)
        .show(ui, |ui| {
            for (i, row) in state.rows.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let surface_resp = ui.add(
                        egui::TextEdit::singleline(&mut row.surface)
                            .id(surface_id(i))
                            .desired_width(120.0)
                            .hint_text("ECOB"),
                    );
                    // The first row's surface field is the FIRST interior control.
                    if i == 0 {
                        state.first_interior_id = Some(surface_resp.id);
                    }
                    if surface_resp.lost_focus() {
                        let upper = row.surface.trim().to_uppercase();
                        if upper != row.surface {
                            row.surface = upper;
                        }
                    }
                    ui.add(
                        egui::TextEdit::singleline(&mut row.canonical)
                            .id(egui::Id::new(("alias_root", i)))
                            .desired_width(120.0)
                            .hint_text("EDIT"),
                    );
                    render_env_selector(ui, i, &mut row.environment);
                    ui.add(
                        egui::TextEdit::singleline(&mut row.command_template)
                            .id(egui::Id::new(("alias_template", i)))
                            .desired_width(240.0)
                            .hint_text("FFWB.COBOL.MASTER(&1)"),
                    );
                    if ui.button("Delete").clicked() {
                        state.pending_delete = Some(i);
                    }
                });
            }
        });

    // Footer: Add + Save, created AFTER the rows so egui-native Tab reaches them
    // last.
    ui.separator();
    ui.horizontal(|ui| {
        // The Add button carries a STABLE id so it can anchor the first-interior
        // focus stop when the editor is empty (Req 5.10). `push_id` makes the
        // wrapped widget's id deterministic and equal to `add_button_id()`.
        let add_resp = ui
            .push_id(add_button_id(), |ui| {
                ui.add(egui::Button::new("Add alias"))
                    .on_hover_text("Append a new blank alias row")
            })
            .inner;
        if row_count == 0 {
            state.first_interior_id = Some(add_resp.id);
        }
        if add_resp.clicked() {
            action = AliasEditorAction::AddRow;
        }
        let save_resp = ui
            .add(egui::Button::new("Save"))
            .on_hover_text("Validate and save the alias store");
        state.last_interior_id = Some(save_resp.id);
        if save_resp.clicked() {
            action = AliasEditorAction::Save;
        }
    });

    // Per-row delete confirmation (Req 5.3 "with confirmation").
    if let Some(index) = state.pending_delete {
        egui::Window::new("Delete alias?")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                let label = state
                    .rows
                    .get(index)
                    .map(|r| r.surface.clone())
                    .unwrap_or_default();
                ui.label(format!("Delete alias '{label}'?"));
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        action = AliasEditorAction::DeleteRow(index);
                        state.pending_delete = None;
                    }
                    if ui.button("Cancel").clicked() {
                        state.pending_delete = None;
                    }
                });
            });
    }

    action
}

/// Render the per-row environment selector combo (ffcmd | ffedit).
fn render_env_selector(ui: &mut egui::Ui, i: usize, env: &mut AliasEnvironment) {
    egui::ComboBox::from_id_salt(("alias_env", i))
        .selected_text(env.label())
        .show_ui(ui, |ui| {
            for choice in AliasEnvironment::ALL {
                ui.selectable_value(env, choice, choice.label());
            }
        });
}
