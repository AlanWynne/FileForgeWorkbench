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
/// Tab focus order is egui-native (CR-NR-076, Option X): every interactive
/// control is created in visual order (menu selector -> title -> display
/// toggles -> separator -> each option's fields -> footer), so egui's built-in
/// Tab traversal walks them in that exact order with no widget skipped. The
/// footer widgets are created AFTER the option rows so Tab reaches them last;
/// the options sit in a height-bounded scroll area so the footer stays on
/// screen. There is no shell-driven focus ring any more -- the previous
/// request_focus ring fought egui's own Tab pass and skipped widgets (B054).
///
/// `WorkspaceContext` impl (CR-NR-078): render the Menus Editor, stash the
/// produced `MenusEditorAction` on `pending_action` for the shell to apply, and
/// report interior focus: FIRST = the "Menu:" selector combo (captured on
/// `first_interior_id`), LAST = the stable Save-As name field.
///
/// Validates: workspace-framework Requirement 1.4, 1.5, 6.1.
impl crate::shell::workspace_context::WorkspaceContext for MenusEditorState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        _services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        self.pending_action = render(ui, self);
        crate::shell::workspace_context::InteriorFocus {
            first: self.first_interior_id,
            last: Some(egui::Id::new("menus_editor_save_as_name")),
        }
    }
}

/// Validates: menu-workspace Requirement 13.1-13.12; automated-dialog-testing
/// Requirement 14.7 (Tab reaches every control in visual order).
pub fn render(ui: &mut egui::Ui, state: &mut MenusEditorState) -> MenusEditorAction {
    // B054: text fields mutate the working menu directly (no field-commit
    // action), so a single `action` slot suffices -- only the structural
    // buttons (select / add / delete / move / save / save as) produce actions.
    let mut action = MenusEditorAction::None;

    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Menus Editor").strong().size(14.0));
    });
    ui.add_space(4.0);

    // --- Menu selector --------------------------------------------------
    // Reset the reported first-interior id each frame; the combo below sets it.
    state.first_interior_id = None;
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
        // The "Menu:" selector combo is the FIRST focusable interior control;
        // report its FRESH id so the shell Boundary_Policy focuses it on Tab
        // from the command field (B056: it was skipped when the shell latched to
        // the Title field instead).
        state.first_interior_id = Some(combo.response.id);
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
        ui.add(
            egui::TextEdit::singleline(&mut menu.title)
                .id(egui::Id::new("menus_editor_title"))
                .desired_width(260.0),
        );
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
    ui.label(egui::RichText::new("Options (key | command | description | group)").strong());

    let option_count = menu.options.len();

    // Options first (creation order == Tab order), in a scroll area bounded to
    // leave room for the footer below so the Add/Save/Save As row is never
    // pushed off screen behind the status bar (Req 13.5). Reserve ~64px for the
    // footer; the scroll area takes the rest of the available height.
    let footer_reserve = 64.0;
    let options_height = (ui.available_height() - footer_reserve).max(64.0);
    egui::ScrollArea::both()
        .id_salt("menus_editor_options")
        .auto_shrink([false, false])
        .max_height(options_height)
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
                    ui.add(
                        egui::TextEdit::singleline(&mut option.command)
                            .id(egui::Id::new(("menus_opt_command", i)))
                            .desired_width(120.0)
                            .hint_text("command"),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut option.description)
                            .id(egui::Id::new(("menus_opt_description", i)))
                            .desired_width(200.0)
                            .hint_text("description"),
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
                    ui.checkbox(&mut option.enabled, "on");
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

    // Footer LAST (created after the options so egui-native Tab reaches it after
    // every option control). Rendered as ordinary rows at the bottom of the
    // central area rather than a separate bottom panel, so creation order
    // matches Tab order. Validates Req 13.5.
    ui.separator();
    if ui.button("Add option").clicked() {
        action = MenusEditorAction::AddOption;
    }
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            action = MenusEditorAction::Save;
        }
        ui.label("Save As:");
        ui.add(
            egui::TextEdit::singleline(&mut state.name_buffer)
                .id(egui::Id::new("menus_editor_save_as_name"))
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

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_workspace::{GroupSeparator, MenuFile, MenuOption};
    use crate::menus_editor_panel::state::MenusEditorState;

    /// Build a MenusEditorState with a loaded menu of `option_count` options,
    /// exercising every interactive control the panel renders.
    fn fixture_state(option_count: usize) -> MenusEditorState {
        let options = (0..option_count)
            .map(|i| MenuOption {
                key: format!("K{i}"),
                command: format!("CMD{i}"),
                description: format!("Option {i}"),
                enabled: true,
                group: None,
                target: None,
            })
            .collect();
        let menu = MenuFile {
            title: "Test Menu".to_string(),
            options,
            show_calendar: true,
            group_separator: GroupSeparator::Space,
            group_headers: false,
        };
        let mut state = MenusEditorState::new();
        state.available = vec!["POM".to_string()];
        state.load_working("POM", menu);
        state
    }

    /// Render the panel headlessly, then press Tab `presses` times, recording
    /// the ORDERED sequence of focused widget ids (one per press) driven purely
    /// by egui-native Tab traversal (Option X -- no shell ring walk). Returns
    /// the ordered list of focused ids.
    fn native_tab_order(option_count: usize, presses: usize) -> Vec<egui::Id> {
        use egui_kittest::Harness;
        let state = fixture_state(option_count);
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(1200.0, 900.0))
            .build_ui_state(
                |ui, state| {
                    let _ = render(ui, state);
                },
                state,
            );
        let mut order: Vec<egui::Id> = Vec::new();
        for _ in 0..presses {
            harness.press_key(egui::Key::Tab);
            harness.run();
            if let Some(id) = harness.ctx.memory(|m| m.focused()) {
                order.push(id);
            }
        }
        order
    }

    /// The stable, explicitly-assigned widget ids in the exact VISUAL order the
    /// panel renders them, for a menu with `option_count` options. These are the
    /// controls the user reported being skipped (Title, per-option key, etc.).
    /// The combo, checkboxes, separator selectables and buttons use egui
    /// auto-ids that cannot be reconstructed here, so this list covers the
    /// stable-id text fields -- which is exactly the reported-bug surface.
    fn stable_ids_in_visual_order(option_count: usize) -> Vec<egui::Id> {
        let mut ids = vec![egui::Id::new("menus_editor_title")];
        for i in 0..option_count {
            ids.push(egui::Id::new(("menus_opt_key", i)));
            ids.push(egui::Id::new(("menus_opt_command", i)));
            ids.push(egui::Id::new(("menus_opt_description", i)));
            ids.push(egui::Id::new(("menus_opt_group", i)));
        }
        ids.push(egui::Id::new("menus_editor_save_as_name"));
        ids
    }

    // Validates: automated-dialog-testing Requirement 14.6, 14.7 -- with the
    // Option X design (egui-native Tab, widgets created in visual order), every
    // stable-id control the panel renders is reached by Tab. This is the
    // regression guard for the B054 report that Title and per-option key were
    // skipped: those are stable-id fields and MUST all be visited.
    #[test]
    fn menus_editor_every_stable_control_is_tab_reachable() {
        let option_count = 3;
        let expected = stable_ids_in_visual_order(option_count);
        // Press enough times to cycle the whole panel a couple of times.
        let order = native_tab_order(option_count, expected.len() * 3 + 8);
        let visited: std::collections::HashSet<egui::Id> = order.iter().copied().collect();
        for id in &expected {
            assert!(
                visited.contains(id),
                "control {id:?} was never focused by Tab -- keyboard-unreachable.\n\
                 visited order: {order:?}"
            );
        }
    }

    // Validates: automated-dialog-testing Requirement 14.7 -- the stable-id
    // controls are visited by Tab in the correct VISUAL order (Title before the
    // first option's key, each option's key before its command/description/group,
    // options before the Save-As field). Guards against the B054 out-of-order /
    // skip behaviour where the footer interleaved before the options.
    #[test]
    fn menus_editor_tab_visits_stable_controls_in_visual_order() {
        let option_count = 3;
        let expected = stable_ids_in_visual_order(option_count);
        let order = native_tab_order(option_count, expected.len() * 3 + 8);

        // Walk the observed focus order; each expected id must appear, and their
        // first-appearances must be in the same relative order as `expected`.
        let mut search_from = 0usize;
        for id in &expected {
            let pos = order[search_from..].iter().position(|o| o == id);
            assert!(
                pos.is_some(),
                "stable control {id:?} not focused in visual order.\n\
                 expected order: {expected:?}\nobserved: {order:?}"
            );
            search_from += pos.unwrap() + 1;
        }
    }
}
