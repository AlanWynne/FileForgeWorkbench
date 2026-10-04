//! # Shell Panel Rendering -- command line, title line, key bar, status bar
//!
//! Title_Line, the primary and detached command fields, the shared command
//! field body, the F-key label bar, the status bar, and the open-file dialog.
//! Split out of `render.rs` (TASK 2.2, pure code movement, no behaviour change).

use eframe::egui;

use super::helpers::*;
use super::render::{CommandFieldSignal, HistoryStep};
use super::WorkbenchShell;

impl WorkbenchShell {
    pub(super) fn render_title_line(&self, ctx: &egui::Context) {
        // CR-CH-041 IRP-b: the app-level Title_Line is the unsplit single
        // instance's chrome; it delegates to the shared Ui-level painter so the
        // split-region path (which draws the SAME Title_Line for the instance
        // placed in each region) is byte-identical. The active tab is the sole
        // docked instance when unsplit.
        let tab_index = self.tabs.active_index();
        egui::TopBottomPanel::top("title_line").show(ctx, |ui| {
            self.render_title_line_into_ui(ui, tab_index);
        });
    }

    /// Paint the Title_Line for the tab at `tab_index` INTO an existing `Ui`
    /// (CR-CH-041 IRP-b: the ctx-panel-free core of [`render_title_line`]).
    ///
    /// The app-level unsplit path wraps this in a `TopBottomPanel` (byte-identical
    /// to before the extraction, passing the active tab); the split-region path
    /// calls it directly for the instance placed in each region so every region
    /// shows its own instance's Title_Line. Home banner / editor path / menu
    /// label styling is preserved (delegated through `kind_title`).
    pub(super) fn render_title_line_into_ui(&self, ui: &mut egui::Ui, tab_index: usize) {
        let Some(tab) = self.tabs.tabs().get(tab_index) else {
            return;
        };
        // CR-NR-090 B.1: the Title_Line label comes from the Kind registry
        // (kind_title) so a reconfigured/user Kind shows its configured title;
        // Home banner / menu label / editor path are delegated inside kind_title.
        // CR-CH-042: the Menu Workspace Title_Line uses ONE THEME-DRIVEN, centered
        // "menu heading" look standardised on the POM aesthetic for EVERY menu --
        // the POM, Settings, and user menus alike (owner: "standardise on the POM
        // look and feel"). The colours come from the theme tokens
        // `primary_menu_bg` / `menu_bar_fg` (the ISPF "primary menu / screen
        // heading" pair), so the Theme Workspace already caters for it -- there
        // are NO hardcoded POM colours and NO `is_home` branch.
        //
        // CR-CH-045 (Req 17.12-17.13): the editor/config + read-only panel
        // Contexts ALSO use this centered themed heading, sourced from a
        // descriptive Title-Case display name (`title_line_display`). Only the
        // file-editor path keeps the left-aligned Title_Line (full path /
        // [Untitled]).
        let is_menu_workspace = tab.kind == crate::tab_state::TabKind::MenuWorkspace;
        let display = self.title_line_display(tab);
        let centered = is_menu_workspace || display.is_some();
        let text = display.unwrap_or_else(|| self.kind_title(tab));
        let bg = to_egui_color(self.palette.ui.primary_menu_bg);
        let fg = to_egui_color(self.palette.ui.menu_bar_fg);
        if centered {
            // Themed fill + centered, strong monospace title. One path for the
            // POM, every other menu, and (CR-CH-045) the custom editor/config +
            // panel Contexts (menu-workspace Req 20.2, menu-and-statusbar Req
            // 17.11/17.12).
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, bg);
            ui.centered_and_justified(|ui| {
                ui.colored_label(fg, egui::RichText::new(&text).monospace().strong());
            });
        } else {
            // File-editor Context (path / [Untitled]): themed fill, left-aligned
            // (Req 17.8 Legacy / Req 21.5 non-Legacy -- same tokens).
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect_filled(rect, 0.0, bg);
            ui.colored_label(fg, egui::RichText::new(text).monospace());
        }
    }

    // ── Command field ────────────────────────────────────────────────────

    pub(super) fn render_command_field(&mut self, ctx: &egui::Context) {
        // CR-NR-095 (Req 8.3): place the primary command field at the top or the
        // bottom per the active instance's Kind Command_Line_Position. The panel
        // BODY is identical either way; only the panel constructor differs.
        use crate::workspace_kind::CommandLinePosition;
        let position = self.command_line_position_for(self.tabs.active_index());
        let panel = match position {
            CommandLinePosition::Top => egui::TopBottomPanel::top("command_field"),
            CommandLinePosition::Bottom => egui::TopBottomPanel::bottom("command_field"),
        };
        panel.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Command ===>");
                let cmd_id = egui::Id::new("command_field_input");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.command_text)
                        .id(cmd_id)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                // Validates: Requirement 16.1, 16.2 — request focus once after Tab cycle
                // lands on CommandField, or on startup. NOT every frame — that would steal
                // focus from POM buttons and other interactive elements.
                // Suppressed when a modal dialog is open so the dialog retains focus.
                if self.focus.command_field_focus_requested && !self.modal_open {
                    self.focus.command_field_focus_requested = false;
                    ctx.memory_mut(|m| m.request_focus(cmd_id));
                }
                // Validates: Requirement 8.1 — Enter while field has focus submits the command.
                // Use lost_focus() to catch the frame egui clears focus on Enter for
                // single-line TextEdit (egui 0.29 surrenders focus on Enter).
                // Also check has_focus() as a fallback for frames where focus is retained.
                let field_has_focus = response.has_focus() || response.lost_focus();
                // Validates: Requirement 2.5 -- register command field state for automation.
                self.automation.register_str(
                    crate::automation::ids::COMMAND_FIELD,
                    crate::automation::ControlState::with_value(&self.command_text),
                );
                if field_has_focus
                    && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                    && !self.command_text.is_empty()
                {
                    let cmd = self.command_text.trim().to_string();
                    // CR-CH-033 (Req 13.1): run through the single decision point
                    // that applies the Command_Line_Outcome to the field (clear on
                    // success, restore on error, or the command's explicit outcome
                    // such as RETRIEVE's recall). The field is left intact during
                    // dispatch so field-reading commands (RETRIEVE) still work.
                    self.run_command_line(&cmd);
                    // Return focus to the command field after every command execution.
                    self.focus.command_field_focus_requested = true;
                }
                // CR-NR-096 (Req 23.1, 23.2, 23.7): while the command field has
                // focus, Up/Down step the shared Command_History. egui's
                // single-line TextEdit does not consume the arrow keys, so we
                // intercept them here. A submit this frame takes precedence
                // (never both). Only the FOCUSED field acts, so a background
                // window's field never hijacks arrows meant for the body.
                else if field_has_focus {
                    let (up, down) = ctx.input(|i| {
                        (
                            i.key_pressed(egui::Key::ArrowUp),
                            i.key_pressed(egui::Key::ArrowDown),
                        )
                    });
                    if up {
                        self.step_command_history(HistoryStep::Older);
                    } else if down {
                        self.step_command_history(HistoryStep::Newer);
                    }
                }

                // ── SCROLL ===> field — Validates: Requirement 19.1, 19.2, 19.3 ──
                ui.separator();
                ui.label("SCROLL ===>");
                let scroll_id = egui::Id::new("scroll_field_input");
                let scroll_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.scroll_field_text)
                        .id(scroll_id)
                        .desired_width(60.0)
                        .font(egui::TextStyle::Monospace),
                );
                // On Enter in the SCROLL field, update the active scroll amount.
                // Validates: Requirement 19.2
                let scroll_has_focus = scroll_resp.has_focus() || scroll_resp.lost_focus();
                if scroll_has_focus && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let text = self.scroll_field_text.trim().to_string();
                    if let Some(amount) = crate::scroll_amount::ScrollAmount::parse(&text) {
                        self.scroll_amount = amount;
                        self.scroll_field_text = self.scroll_amount.display_string();
                        self.open_error = None;
                    } else {
                        self.open_error = Some(format!(
                            "SCROLL: '{}' is not a valid scroll amount (PAGE/HALF/CSR/MAX/DATA/n)",
                            text
                        ));
                    }
                    // Return focus to command field.
                    self.focus.command_field_focus_requested = true;
                }
            });
        });
    }

    /// Render a Detached_Workspace's own `Command ===>` field into its child
    /// viewport (CR-CH-036, menu-and-statusbar Req 18.10). Called from WITHIN
    /// `with_workspace_context`, so `self.command_text` currently holds THIS
    /// window's buffer and dispatch targets THIS window's tab. The panel and
    /// widget ids are salted by `tab_id` so they never collide with the
    /// Primary_Window's command field or another detached window's.
    ///
    /// Validates: menu-and-statusbar Requirement 18.10
    pub(super) fn render_detached_command_field(
        &mut self,
        ctx: &egui::Context,
        tab_id: crate::tab_state::TabId,
    ) {
        let panel_id = egui::Id::new(("detached_command_field", tab_id.0));
        let cmd_id = egui::Id::new(("detached_command_field_input", tab_id.0));
        let accent = to_egui_color(self.palette.editor.accent);
        let modal_open = self.modal_open;
        // CR-NR-095 (Req 8.5): place the detached window's command field per the
        // detached instance's Kind position. We are inside `with_workspace_context`,
        // so the active tab IS this detached instance.
        use crate::workspace_kind::CommandLinePosition;
        let position = self.command_line_position_for(self.tabs.active_index());
        // The detached field's buffers are the shell's own fields right now
        // (installed by `with_workspace_context`). Render the shared body against
        // them via short-lived local bindings, then reflect focus/submit back.
        let mut command_text = std::mem::take(&mut self.command_text);
        let mut focus_requested = self.focus.command_field_focus_requested;
        let open_error = self.open_error.clone();
        let panel = match position {
            CommandLinePosition::Top => egui::TopBottomPanel::top(panel_id),
            CommandLinePosition::Bottom => egui::TopBottomPanel::bottom(panel_id),
        };
        let signal = panel
            .show(ctx, |ui| {
                Self::render_command_field_body(
                    ctx,
                    ui,
                    cmd_id,
                    &mut command_text,
                    &mut focus_requested,
                    modal_open,
                    accent,
                    open_error.as_deref(),
                )
            })
            .inner;
        self.command_text = command_text;
        self.focus.command_field_focus_requested = focus_requested;
        if let Some(cmd) = signal.submitted {
            // Dispatches through the SAME pipeline; because we are inside
            // `with_workspace_context`, it acts on this window's tab and the
            // Command_Line_Outcome applies to this window's buffer.
            self.run_command_line(&cmd);
            self.focus.command_field_focus_requested = true;
        } else if let Some(step) = signal.history_step {
            // CR-NR-096 (Req 23.9): the SAME arrow-history behaviour. We are
            // inside `with_workspace_context`, so `self.command_text` is this
            // window's buffer; the history/In_Progress_Line are shared.
            self.step_command_history(step);
        }
    }

    /// Shared `Command ===>` field body used by BOTH the Detached_Workspace
    /// command line (Req 18.10) and each in-window split region's command line
    /// (CR-NR-094, Req 15.1-15.9). It borrows only the field's own state -- never
    /// `&mut self` -- so a caller can bind it to whichever command context owns
    /// the region (the shell's fields for a detached window, or an entry of
    /// `region_cmd_ctx` for a split leaf). Returns `Some(command)` when the user
    /// pressed Enter on a non-empty line this frame; the caller performs the
    /// dispatch against the correct tab. The `cmd_id` MUST be a stable, per-region
    /// salted id so focus round-trips and Tab-order stay deterministic (B056).
    ///
    /// Validates: layout-and-docking Requirement 15.1, 15.2, 15.3, 15.9;
    /// function-keys-and-history Requirement 23.1, 23.2, 23.7, 23.9
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_command_field_body(
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        cmd_id: egui::Id,
        command_text: &mut String,
        focus_requested: &mut bool,
        modal_open: bool,
        accent: egui::Color32,
        open_error: Option<&str>,
    ) -> CommandFieldSignal {
        let mut signal = CommandFieldSignal::default();
        ui.horizontal(|ui| {
            ui.label("Command ===>");
            let response = ui.add(
                egui::TextEdit::singleline(command_text)
                    .id(cmd_id)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace),
            );
            if *focus_requested && !modal_open {
                *focus_requested = false;
                ctx.memory_mut(|m| m.request_focus(cmd_id));
            }
            let field_has_focus = response.has_focus() || response.lost_focus();
            if field_has_focus
                && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                && !command_text.is_empty()
            {
                signal.submitted = Some(command_text.trim().to_string());
            }
            // CR-NR-096 (Req 23.1, 23.2, 23.7): Up/Down step the Command_History
            // ONLY while this field has keyboard focus. egui's single-line
            // `TextEdit` does not consume the arrow keys, so we intercept them
            // here and report the gesture to the caller (which owns the shared
            // command processor history + In_Progress_Line). Enter takes
            // precedence -- a submit this frame is never also a history step.
            if field_has_focus && signal.submitted.is_none() {
                let (up, down) = ctx.input(|i| {
                    (
                        i.key_pressed(egui::Key::ArrowUp),
                        i.key_pressed(egui::Key::ArrowDown),
                    )
                });
                if up {
                    signal.history_step = Some(HistoryStep::Older);
                } else if down {
                    signal.history_step = Some(HistoryStep::Newer);
                }
            }
            // Status/error line for THIS region (its own open_error).
            if let Some(err) = open_error {
                ui.separator();
                ui.colored_label(accent, egui::RichText::new(err).monospace().small());
            }
        });
        signal
    }
}
