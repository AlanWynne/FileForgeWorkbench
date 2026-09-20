//! Pure render for the Kinds Editor Context (workspace-kinds Req 6, CR-NR-090
//! B.4), modelled on `keys_editor_panel::render`.
//!
//! Returns a [`KindsEditorAction`]; the shell applies side effects (select /
//! new / save). Editable fields bind DIRECTLY to the mutable working
//! `KindConfig` so edits persist across frames.

use super::{KindsEditorAction, KindsEditorState};
use crate::workspace_kind::{BaseKind, BuiltinKind};
use ff_edit_operations::{CapsMode, HiliteMode, NullsMode, ProfileLock, StatsMode};

/// `WorkspaceContext` impl (CR-NR-078): render, stash the action, report focus.
///
/// Validates: workspace-kinds Requirement 6.5; workspace-framework Req 1.4, 1.5.
impl crate::shell::workspace_context::WorkspaceContext for KindsEditorState {
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

/// Render the Kinds Editor, returning the action to apply.
///
/// Validates: workspace-kinds Requirement 6.1, 6.2.
pub fn render(ui: &mut egui::Ui, state: &mut KindsEditorState) -> KindsEditorAction {
    let mut action = KindsEditorAction::None;

    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Workspace Kinds Editor").strong().size(14.0));
    });
    ui.add_space(4.0);

    // --- Kind selector (FIRST interior control) --------------------------
    state.first_interior_id = None;
    ui.horizontal(|ui| {
        ui.label("Kind:");
        let current = state
            .selected
            .clone()
            .unwrap_or_else(|| "(select)".to_string());
        let combo = egui::ComboBox::from_id_salt("kinds_editor_select")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for name in &state.kind_names {
                    if ui
                        .selectable_label(state.selected.as_deref() == Some(name.as_str()), name)
                        .clicked()
                    {
                        action = KindsEditorAction::SelectKind(name.clone());
                    }
                }
            });
        state.first_interior_id = Some(combo.response.id);
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::from_rgb(0xC8, 0x8A, 0x00), err);
    }

    // --- "New Kind modelled on <base>" -----------------------------------
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("New Kind:");
        ui.add(
            egui::TextEdit::singleline(&mut state.new_name)
                .id(egui::Id::new("kinds_editor_new_name"))
                .desired_width(160.0)
                .hint_text("name"),
        );
        ui.label("modelled on");
        egui::ComboBox::from_id_salt("kinds_editor_new_base")
            .selected_text(state.new_base.stable_name())
            .show_ui(ui, |ui| {
                for base in BuiltinKind::ALL {
                    ui.selectable_value(&mut state.new_base, base, base.stable_name());
                }
            });
        if ui.button("Create").clicked() {
            let name = state.new_name.trim().to_string();
            if !name.is_empty() {
                action = KindsEditorAction::NewKind {
                    name,
                    base: state.new_base,
                };
            }
        }
    });

    // --- Editable form for the selected Kind -----------------------------
    let Some(cfg) = state.working.as_mut() else {
        ui.separator();
        ui.label("Select a Kind to edit, or create a new one.");
        return action;
    };

    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Modelled on:");
        let base_label = match &cfg.modelled_on {
            BaseKind::Builtin(k) => k.stable_name().to_string(),
            BaseKind::External(n) => format!("ext:{n}"),
        };
        ui.label(egui::RichText::new(base_label).monospace());
        ui.label(egui::RichText::new("(create a new Kind to change the base)").weak().small());
    });
    ui.horizontal(|ui| {
        ui.label("Title:");
        ui.add(
            egui::TextEdit::singleline(&mut cfg.title)
                .id(egui::Id::new("kinds_editor_title"))
                .desired_width(220.0),
        );
    });
    // menu_bar / key_list are Option<String>; edit via a scratch String bound to
    // the option (empty => None).
    edit_optional(ui, "Menu bar:", "kinds_editor_menu_bar", &mut cfg.menu_bar);
    edit_optional(ui, "Key list:", "kinds_editor_key_list", &mut cfg.key_list);

    ui.separator();
    ui.label(egui::RichText::new("Profile").strong());
    let p = &mut cfg.profile;
    let mut caps = p.edit_profile.caps.is_on();
    if ui.checkbox(&mut caps, "CAPS").changed() {
        p.edit_profile.caps = if caps { CapsMode::On } else { CapsMode::Off };
    }
    let mut nulls = p.edit_profile.nulls.is_on();
    if ui.checkbox(&mut nulls, "NULLS").changed() {
        p.edit_profile.nulls = if nulls { NullsMode::On } else { NullsMode::Off };
    }
    let mut stats = p.edit_profile.stats == StatsMode::On;
    if ui.checkbox(&mut stats, "STATS").changed() {
        p.edit_profile.stats = if stats { StatsMode::On } else { StatsMode::Off };
    }
    let mut lock = p.edit_profile.lock == ProfileLock::On;
    if ui.checkbox(&mut lock, "LOCK").changed() {
        p.edit_profile.lock = if lock { ProfileLock::On } else { ProfileLock::Off };
    }
    let mut hilite = p.edit_profile.hilite != HiliteMode::Off;
    if ui.checkbox(&mut hilite, "HILITE").changed() {
        p.edit_profile.hilite = if hilite { HiliteMode::On } else { HiliteMode::Off };
    }
    ui.horizontal(|ui| {
        ui.label("Tab size:");
        ui.add(egui::DragValue::new(&mut p.tab_size).range(1..=16));
        ui.label("Line endings:");
        let mut unicode = p.line_end_mode.eq_ignore_ascii_case("unicode");
        if ui.checkbox(&mut unicode, "Unicode").changed() {
            p.line_end_mode = if unicode { "unicode" } else { "default" }.to_string();
        }
    });

    ui.separator();
    ui.horizontal(|ui| {
        let save = ui.add(egui::Button::new("Save").min_size(egui::vec2(60.0, 0.0)));
        state.last_interior_id = Some(save.id);
        if save.clicked() {
            action = KindsEditorAction::Save;
        }
        ui.label(
            egui::RichText::new("Saves to workspace-kinds/<name>.toml")
                .weak()
                .small(),
        );
    });
    action
}

/// Edit an `Option<String>` field via a text box (empty => None).
fn edit_optional(ui: &mut egui::Ui, label: &str, id: &str, field: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut text = field.clone().unwrap_or_default();
        let resp = ui.add(
            egui::TextEdit::singleline(&mut text)
                .id(egui::Id::new(id))
                .desired_width(220.0)
                .hint_text("(use base default)"),
        );
        if resp.changed() {
            let t = text.trim().to_string();
            *field = if t.is_empty() { None } else { Some(t) };
        }
    });
}
