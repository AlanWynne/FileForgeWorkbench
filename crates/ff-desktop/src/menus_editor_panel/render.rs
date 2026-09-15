//! Pure render for the Menus Editor (menu-workspace Req 13).
//!
//! Returns a [`MenusEditorAction`]; the shell applies structural side effects
//! (select / add / delete / move / save). Text and toggle fields are bound
//! DIRECTLY to the mutable working `MenuFile` so edits persist across frames
//! (B054); they do not produce actions.

use super::state::{MenusEditorAction, MenusEditorState};
use crate::menu_workspace::GroupSeparator;

/// Render the Menus Editor Context, returning the action to apply.
///
/// Validates: menu-workspace Requirement 13.1-13.12.
pub fn render(ui: &mut egui::Ui, state: &mut MenusEditorState) -> MenusEditorAction {
    // B054: text fields now mutate the working menu directly (no field-commit
    // action), so a single `action` slot suffices -- only the structural
    // buttons (select / add / delete / move / save / save as) produce actions.
    let mut action = MenusEditorAction::None;

    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Menus Editor").strong().size(14.0));
    });
    ui.add_space(4.0);

    // --- Menu selector --------------------------------------------------
    ui.horizontal(|ui| {
        ui.label("Menu:");
        let current = state
            .selected
            .clone()
            .unwrap_or_else(|| "(none)".to_string());
        egui::ComboBox::from_id_salt("menus_editor_select")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for name in state.available.clone() {
                    if ui
                        .selectable_label(state.selected.as_deref() == Some(name.as_str()), &name)
                        .clicked()
                    {
                        action = MenusEditorAction::Select(name.clone());
                    }
                }
            });
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::from_rgb(0xD2, 0x0F, 0x39), err);
    }

    let Some(menu) = state.working.as_mut() else {
        ui.label("Select a menu to edit.");
        return action;
    };

    ui.separator();

    // --- Menu-level fields ----------------------------------------------
    // B054 fix: bind the TextEdit DIRECTLY to the mutable working menu so
    // keystrokes persist across frames (a per-frame local buffer is discarded
    // before the next frame and the field appears frozen).
    ui.horizontal(|ui| {
        ui.label("Title:");
        ui.text_edit_singleline(&mut menu.title);
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut menu.show_calendar, "Show calendar");
        ui.checkbox(&mut menu.group_headers, "Group headers");
    });

    ui.horizontal(|ui| {
        ui.label("Group separator:");
        for (label, sep) in [
            ("Space", GroupSeparator::Space),
            ("Line", GroupSeparator::Line),
            ("None", GroupSeparator::None),
        ] {
            if ui
                .selectable_label(menu.group_separator == sep, label)
                .clicked()
            {
                menu.group_separator = sep;
            }
        }
    });

    ui.separator();
    ui.label(egui::RichText::new("Options (key | command | description)").strong());

    // --- Option rows ----------------------------------------------------
    // B054 fix: iterate the options MUTABLY and bind each TextEdit directly to
    // the working field, so typed input persists across frames. Structural
    // actions (add/delete/move) still fire as actions since they change the
    // vector length and cannot run during this mutable borrow.
    let option_count = menu.options.len();
    let mut group_buf = String::new();
    egui::ScrollArea::vertical()
        .id_salt("menus_editor_options")
        .show(ui, |ui| {
            for (i, option) in menu.options.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    // Key -- edited directly; uppercased on commit (lost_focus).
                    let key_resp = ui.add(
                        egui::TextEdit::singleline(&mut option.key)
                            .desired_width(48.0)
                            .hint_text("key"),
                    );
                    if key_resp.lost_focus() {
                        let upper = option.key.trim().to_uppercase();
                        if upper != option.key {
                            option.key = upper;
                        }
                    }
                    // Command / Description -- edited directly.
                    ui.add(
                        egui::TextEdit::singleline(&mut option.command)
                            .desired_width(120.0)
                            .hint_text("command"),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut option.description)
                            .desired_width(200.0)
                            .hint_text("description"),
                    );
                    // Group -- Option<String> edited via a scratch buffer.
                    group_buf.clear();
                    if let Some(g) = &option.group {
                        group_buf.push_str(g);
                    }
                    let grp_resp = ui.add(
                        egui::TextEdit::singleline(&mut group_buf)
                            .desired_width(80.0)
                            .hint_text("group"),
                    );
                    if grp_resp.changed() {
                        let trimmed = group_buf.trim();
                        option.group = if trimmed.is_empty() {
                            None
                        } else {
                            Some(group_buf.clone())
                        };
                    }
                    // Enabled -- edited directly.
                    ui.checkbox(&mut option.enabled, "on");
                    // Reorder / delete (structural -> action; applied after loop).
                    if ui.add_enabled(i > 0, egui::Button::new("^")).clicked() {
                        action = MenusEditorAction::MoveOptionUp(i);
                    }
                    if ui
                        .add_enabled(i + 1 < option_count, egui::Button::new("v"))
                        .clicked()
                    {
                        action = MenusEditorAction::MoveOptionDown(i);
                    }
                    if ui.button("Delete").clicked() {
                        action = MenusEditorAction::DeleteOption(i);
                    }
                });
            }
        });

    if ui.button("Add option").clicked() {
        action = MenusEditorAction::AddOption;
    }

    ui.separator();

    // --- Save / Save As -------------------------------------------------
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            action = MenusEditorAction::Save;
        }
        ui.label("Save As:");
        ui.add(
            egui::TextEdit::singleline(&mut state.name_buffer)
                .desired_width(120.0)
                .hint_text("new name"),
        );
        if ui.button("Save As").clicked() {
            let name = state.name_buffer.trim().to_string();
            if !name.is_empty() {
                action = MenusEditorAction::SaveAs(name);
            }
        }
    });

    action
}
