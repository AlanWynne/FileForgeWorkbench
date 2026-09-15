//! Pure render for the Menus Editor (menu-workspace Req 13).
//!
//! Returns a [`MenusEditorAction`]; the shell applies all side effects. Uses the
//! two-slot pattern (button/selector `action` wins over a text-field
//! `field_action` produced on commit) so a button click is never lost to a
//! same-frame `lost_focus` edit (the B052 rule from the Theme editor).

use super::state::{MenusEditorAction, MenusEditorState, OptionField};
use crate::menu_workspace::GroupSeparator;

/// Render the Menus Editor Context, returning the action to apply.
///
/// Validates: menu-workspace Requirement 13.1-13.12.
pub fn render(ui: &mut egui::Ui, state: &mut MenusEditorState) -> MenusEditorAction {
    let mut action = MenusEditorAction::None;
    let mut field_action = MenusEditorAction::None;

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
    ui.horizontal(|ui| {
        ui.label("Title:");
        let mut title = menu.title.clone();
        if ui.text_edit_singleline(&mut title).lost_focus() && title != menu.title {
            field_action = MenusEditorAction::EditTitle(title);
        }
    });

    ui.horizontal(|ui| {
        let mut show_cal = menu.show_calendar;
        if ui.checkbox(&mut show_cal, "Show calendar").changed() {
            action = MenusEditorAction::SetShowCalendar(show_cal);
        }
        let mut headers = menu.group_headers;
        if ui.checkbox(&mut headers, "Group headers").changed() {
            action = MenusEditorAction::SetGroupHeaders(headers);
        }
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
                action = MenusEditorAction::SetGroupSeparator(sep);
            }
        }
    });

    ui.separator();
    ui.label(egui::RichText::new("Options (key | command | description)").strong());

    // --- Option rows ----------------------------------------------------
    let option_count = menu.options.len();
    egui::ScrollArea::vertical()
        .id_salt("menus_editor_options")
        .show(ui, |ui| {
            for (i, option) in menu.options.iter().enumerate() {
                ui.horizontal(|ui| {
                    // Key
                    let mut key = option.key.clone();
                    let key_resp = ui.add(
                        egui::TextEdit::singleline(&mut key)
                            .desired_width(48.0)
                            .hint_text("key"),
                    );
                    if key_resp.lost_focus() && key != option.key {
                        field_action = MenusEditorAction::EditOption {
                            index: i,
                            field: OptionField::Key,
                            value: key,
                        };
                    }
                    // Command
                    let mut command = option.command.clone();
                    let cmd_resp = ui.add(
                        egui::TextEdit::singleline(&mut command)
                            .desired_width(120.0)
                            .hint_text("command"),
                    );
                    if cmd_resp.lost_focus() && command != option.command {
                        field_action = MenusEditorAction::EditOption {
                            index: i,
                            field: OptionField::Command,
                            value: command,
                        };
                    }
                    // Description
                    let mut desc = option.description.clone();
                    let desc_resp = ui.add(
                        egui::TextEdit::singleline(&mut desc)
                            .desired_width(200.0)
                            .hint_text("description"),
                    );
                    if desc_resp.lost_focus() && desc != option.description {
                        field_action = MenusEditorAction::EditOption {
                            index: i,
                            field: OptionField::Description,
                            value: desc,
                        };
                    }
                    // Group
                    let mut group = option.group.clone().unwrap_or_default();
                    let grp_resp = ui.add(
                        egui::TextEdit::singleline(&mut group)
                            .desired_width(80.0)
                            .hint_text("group"),
                    );
                    if grp_resp.lost_focus() && group != option.group.clone().unwrap_or_default() {
                        field_action = MenusEditorAction::EditOption {
                            index: i,
                            field: OptionField::Group,
                            value: group,
                        };
                    }
                    // Enabled
                    let mut enabled = option.enabled;
                    if ui.checkbox(&mut enabled, "on").changed() {
                        action = MenusEditorAction::SetOptionEnabled { index: i, enabled };
                    }
                    // Reorder / delete
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

    // Button/selector actions win over a same-frame field commit (B052).
    if action != MenusEditorAction::None {
        action
    } else {
        field_action
    }
}
