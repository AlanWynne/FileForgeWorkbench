//! Pure render for the Alias Editor ISPF layout (command-configurator Req 5.13
//! -5.24, CR-CH-068).
//!
//! Top to bottom: a `Command ===>` primary command field; the column headings
//! (Act / CE / Root / L18N / ALIAS / Parameters); a FILTER row (filter CE + Root
//! only); then the filtered data rows inside a vertical scroll area with a
//! right-hand scrollbar. The render is PURE: it returns an [`AliasEditorAction`]
//! and the shell applies structural side effects (apply line commands, run the
//! primary command). Per-field edits bind directly to the mutable working rows
//! (B054 pattern) so edits persist across frames.

use super::filter::displayed_indices;
use super::rows::{render_data_row, render_root_combo};
use super::state::{AliasEditorAction, AliasEditorState, AliasRowEdit};
use crate::shell::environment_registry::EnvironmentRegistry;
use crate::shell::user_aliases::AliasEnvironment;

/// Stable egui id of the `Command ===>` primary command field (design 9.10.3).
fn command_line_id() -> egui::Id {
    egui::Id::new("alias_editor_command_line")
}

/// `WorkspaceContext` impl (CR-NR-078): render the Alias Editor, stash the
/// produced [`AliasEditorAction`] on `pending_action`, and report interior focus
/// (FIRST = the FILTER row's filter CE dropdown captured fresh each frame; LAST =
/// the last interior control).
///
/// Validates: command-configurator Requirement 5.13, 5.23.
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
/// visual order (`Command ===>` is the entry control; then filter CE -> filter
/// Root -> each data row Act -> CE -> Root -> L18N -> ALIAS -> Parameters), so
/// egui's built-in Tab traversal walks them in that exact order with no phantom
/// stop (Req 5.21, 5.23). The FILTER row's filter CE dropdown is reported as the
/// first interior stop.
///
/// Validates: command-configurator Requirement 5.13-5.23
pub fn render(ui: &mut egui::Ui, state: &mut AliasEditorState) -> AliasEditorAction {
    let mut action = AliasEditorAction::None;

    // Reset the reported focus anchors each frame; set below.
    state.first_interior_id = None;
    state.last_interior_id = None;

    // --- Title (ISPF) ----------------------------------------------------
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("FileForge Workbench Alias Editor").strong());
    });

    // --- Command ===> primary command line -------------------------------
    ui.horizontal(|ui| {
        ui.label("Command ===>");
        let resp = ui.add(
            egui::TextEdit::singleline(&mut state.command_line)
                .id(command_line_id())
                .desired_width(ui.available_width().max(200.0)),
        );
        // On Enter (lost focus + Enter key), emit the command-line action. A bare
        // Enter (no verb typed) applies pending line commands only (design 9.10.4
        // / 9.10.8 part 4); a typed verb carries through as CommandLine.
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let text = std::mem::take(&mut state.command_line);
            action = if text.trim().is_empty() {
                AliasEditorAction::ApplyLineCommands
            } else {
                AliasEditorAction::CommandLine(text)
            };
        }
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::from_rgb(0xD2, 0x0F, 0x39), err);
    }

    // --- Column headings -------------------------------------------------
    ui.horizontal(|ui| {
        heading(ui, "Act", 28.0);
        heading(ui, "CE", 84.0);
        heading(ui, "Root", 96.0);
        heading(ui, "L18N", 36.0);
        heading(ui, "ALIAS", 120.0);
        heading(ui, "Parameters", 260.0);
    });

    // --- FILTER row (filter CE + filter Root ONLY) -----------------------
    render_filter_row(ui, state);
    ui.separator();

    // --- Data rows (filtered) inside a scroll area with a right scrollbar --
    let displayed = displayed_indices(&state.rows, state.filter_ce, &state.filter_root);
    let rows_height = ui.available_height().max(64.0);
    egui::ScrollArea::vertical()
        .id_salt("alias_editor_rows")
        .auto_shrink([false, false])
        .max_height(rows_height)
        .show(ui, |ui| {
            if state.rows.is_empty() {
                // EMPTY-STORE affordance (Req 5.17): always render one empty
                // insertable scratch row so the first alias can be created. The
                // scratch row is promoted into `rows` the moment it is edited or
                // an `I` is typed, keeping "empty store = zero persisted rows"
                // true until then.
                render_scratch_row(ui, state);
            } else {
                for backing in &displayed {
                    let idx = *backing;
                    if let Some(row) = state.rows.get_mut(idx) {
                        render_data_row(ui, idx, row);
                    }
                }
            }
        });

    action
}

/// Render a strong, fixed-width column heading.
fn heading(ui: &mut egui::Ui, text: &str, width: f32) {
    ui.add_sized(
        [width, ui.spacing().interact_size.y],
        egui::Label::new(egui::RichText::new(text).strong()),
    );
}

/// Render the FILTER row: a filter CE dropdown (the first interior control) and a
/// filter Root dropdown constrained to the chosen CE's roots. The filter is set
/// by direct selection (applied immediately to the DISPLAY), consistent with the
/// `FILTER` / `RESET` command path (Req 5.15). Filtering never mutates `rows`.
fn render_filter_row(ui: &mut egui::Ui, state: &mut AliasEditorState) {
    ui.horizontal(|ui| {
        // Spacer under the Act heading (the FILTER row has no Act field).
        ui.add_space(28.0 + ui.spacing().item_spacing.x);

        // Filter CE dropdown -- the FIRST interior control (Req 5.23). Its id is
        // captured FRESH each frame from the combo response (design 9.10.8 pt 3).
        let shown = match state.filter_ce {
            Some(env) => env.label(),
            None => "--",
        };
        let mut picked: Option<Option<AliasEnvironment>> = None;
        let combo = egui::ComboBox::from_id_salt("alias_filter_ce")
            .selected_text(shown)
            .width(84.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(state.filter_ce.is_none(), "--")
                    .clicked()
                {
                    picked = Some(None);
                }
                for choice in AliasEnvironment::ALL {
                    if ui
                        .selectable_label(state.filter_ce == Some(choice), choice.label())
                        .clicked()
                    {
                        picked = Some(Some(choice));
                    }
                }
            });
        state.first_interior_id = Some(combo.response.id);
        state.last_interior_id = Some(combo.response.id);
        if let Some(sel) = picked {
            state.filter_ce = sel;
            // A new/cleared CE invalidates a root filter not in that environment.
            let env_name = sel.unwrap_or_default().env_name();
            if !state.filter_root.is_empty()
                && !EnvironmentRegistry::contains_root(env_name, &state.filter_root)
            {
                state.filter_root.clear();
            }
        }

        // Filter Root dropdown constrained to the chosen CE's roots (Req 5.15).
        let root_env = state.filter_ce.unwrap_or_default();
        render_root_combo(ui, "alias_filter_root", root_env, &mut state.filter_root);
    });
}

/// Render the empty-store scratch row and promote it into `rows` as soon as the
/// user edits a field or types an Act letter (Req 5.17, design 9.10.6).
fn render_scratch_row(ui: &mut egui::Ui, state: &mut AliasEditorState) {
    let mut scratch = AliasRowEdit::default();
    render_data_row(ui, 0, &mut scratch);
    if scratch != AliasRowEdit::default() {
        state.rows.push(scratch);
    }
}
