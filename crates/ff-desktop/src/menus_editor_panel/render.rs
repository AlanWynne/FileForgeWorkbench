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

    // Accessibility (keyboard reachability): capture EVERY interactive control's
    // real egui id, in visual order, so the shell's Tab handler can walk them
    // all -- combo, checkboxes and separator selectables included, not just text
    // fields. We collect into a local and store it on the state at the end
    // (the state is borrowed mutably as `menu` in between).
    let mut focus_ids: Vec<egui::Id> = Vec::new();

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
        let combo = egui::ComboBox::from_id_salt("menus_editor_select")
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
        focus_ids.push(combo.response.id);
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::from_rgb(0xD2, 0x0F, 0x39), err);
    }

    let Some(menu) = state.working.as_mut() else {
        ui.label("Select a menu to edit.");
        state.focus_ids = focus_ids;
        return action;
    };

    ui.separator();

    // --- Menu-level fields ----------------------------------------------
    // B054 fix: bind the TextEdit DIRECTLY to the mutable working menu so
    // keystrokes persist across frames (a per-frame local buffer is discarded
    // before the next frame and the field appears frozen).
    ui.horizontal(|ui| {
        ui.label("Title:");
        let r = ui.add(
            egui::TextEdit::singleline(&mut menu.title)
                .id(egui::Id::new("menus_editor_title"))
                .desired_width(260.0),
        );
        focus_ids.push(r.id);
    });

    ui.horizontal(|ui| {
        focus_ids.push(ui.checkbox(&mut menu.show_calendar, "Show calendar").id);
        focus_ids.push(ui.checkbox(&mut menu.group_headers, "Group headers").id);
    });

    ui.horizontal(|ui| {
        ui.label("Group separator:");
        for (label, sep) in [
            ("Space", GroupSeparator::Space),
            ("Line", GroupSeparator::Line),
            ("None", GroupSeparator::None),
        ] {
            let r = ui.selectable_label(menu.group_separator == sep, label);
            if r.clicked() {
                menu.group_separator = sep;
            }
            focus_ids.push(r.id);
        }
    });

    ui.separator();
    ui.label(egui::RichText::new("Options (key | command | description | group)").strong());

    let option_count = menu.options.len();

    // Reserve the fixed footer (Add / Save / Save As) at the BOTTOM first, then
    // let the option ScrollArea fill only the remaining height. Rendering the
    // footer bottom-up before the scroll body is what keeps the buttons on
    // screen (previously the unbounded ScrollArea grew past the visible central
    // panel and pushed the footer behind the status bar). Validates Req 13.5.
    // Footer ids are collected separately and APPENDED to the ring after the
    // option ids, so the Tab order is menu-level -> options -> footer even though
    // the footer panel is drawn first (bottom-up) in code.
    let mut footer_ids: Vec<egui::Id> = Vec::new();
    egui::TopBottomPanel::bottom("menus_editor_footer")
        .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(0, 4)))
        .show_inside(ui, |ui| {
            let add = ui.button("Add option");
            if add.clicked() {
                action = MenusEditorAction::AddOption;
            }
            footer_ids.push(add.id);
            ui.separator();
            ui.horizontal(|ui| {
                let save = ui.button("Save");
                if save.clicked() {
                    action = MenusEditorAction::Save;
                }
                footer_ids.push(save.id);
                ui.label("Save As:");
                footer_ids.push(
                    ui.add(
                        egui::TextEdit::singleline(&mut state.name_buffer)
                            .id(egui::Id::new("menus_editor_save_as_name"))
                            .desired_width(120.0)
                            .hint_text("new name"),
                    )
                    .id,
                );
                let save_as = ui.button("Save As");
                if save_as.clicked() {
                    let name = state.name_buffer.trim().to_string();
                    if !name.is_empty() {
                        action = MenusEditorAction::SaveAs(name);
                    }
                }
                footer_ids.push(save_as.id);
            });
        });

    // --- Option rows (original compact single-row-per-option layout) --------
    // Each option is ONE row: key | command | description | group | on |
    // Up | Down | Delete. Wrapped in a CentralPanel so the ScrollArea fills the
    // space left above the footer (never overlapping the status bar), and in a
    // horizontal ScrollArea so wide rows scroll rather than clip. Fields bind
    // directly to the working model with explicit unique ids (B054).
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt("menus_editor_options")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (i, option) in menu.options.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            let key_resp = ui.add(
                                egui::TextEdit::singleline(&mut option.key)
                                    .id(egui::Id::new(("menus_opt_key", i)))
                                    .desired_width(44.0)
                                    .hint_text("key"),
                            );
                            if key_resp.lost_focus() {
                                let upper = option.key.trim().to_uppercase();
                                if upper != option.key {
                                    option.key = upper;
                                }
                            }
                            focus_ids.push(key_resp.id);
                            focus_ids.push(
                                ui.add(
                                    egui::TextEdit::singleline(&mut option.command)
                                        .id(egui::Id::new(("menus_opt_command", i)))
                                        .desired_width(120.0)
                                        .hint_text("command"),
                                )
                                .id,
                            );
                            focus_ids.push(
                                ui.add(
                                    egui::TextEdit::singleline(&mut option.description)
                                        .id(egui::Id::new(("menus_opt_description", i)))
                                        .desired_width(200.0)
                                        .hint_text("description"),
                                )
                                .id,
                            );
                            let mut group_val = option.group.clone().unwrap_or_default();
                            let grp_resp = ui.add(
                                egui::TextEdit::singleline(&mut group_val)
                                    .id(egui::Id::new(("menus_opt_group", i)))
                                    .desired_width(80.0)
                                    .hint_text("group"),
                            );
                            if grp_resp.changed() {
                                let trimmed = group_val.trim();
                                option.group = if trimmed.is_empty() {
                                    None
                                } else {
                                    Some(group_val.clone())
                                };
                            }
                            focus_ids.push(grp_resp.id);
                            focus_ids.push(ui.checkbox(&mut option.enabled, "on").id);
                            let up = ui.add_enabled(i > 0, egui::Button::new("^"));
                            if up.clicked() {
                                action = MenusEditorAction::MoveOptionUp(i);
                            }
                            focus_ids.push(up.id);
                            let down = ui.add_enabled(i + 1 < option_count, egui::Button::new("v"));
                            if down.clicked() {
                                action = MenusEditorAction::MoveOptionDown(i);
                            }
                            focus_ids.push(down.id);
                            let del = ui.button("Delete");
                            if del.clicked() {
                                action = MenusEditorAction::DeleteOption(i);
                            }
                            focus_ids.push(del.id);
                        });
                    }
                });
        });

    // Append the footer controls after the option controls, then publish the
    // full ordered ring for the shell's Tab handler.
    focus_ids.extend(footer_ids);

    // TEMP DIAGNOSTIC (B054/Tab): report the full captured ring once, and flag
    // any duplicate ids (a duplicate would make the Tab handler's position()
    // land on the first occurrence and skip the second widget). Logged every
    // frame at trace; grep "[menus-ring]".
    {
        let mut seen = std::collections::HashSet::new();
        let mut dups: Vec<egui::Id> = Vec::new();
        for id in &focus_ids {
            if !seen.insert(*id) {
                dups.push(*id);
            }
        }
        ff_logging::log_debug!(
            "[menus-ring] captured {} ids dups={:?} ids={:?}",
            focus_ids.len(),
            dups,
            focus_ids
        );
    }

    state.focus_ids = focus_ids;

    action
}
