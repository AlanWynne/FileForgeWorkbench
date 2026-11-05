//! # Shell dialog / overlay cluster
//!
//! The Command Palette, About, Command History, SWAP picker, External-exec
//! confirmation, RESET BARE confirmation, Unsaved-workspace, and Catalog Manager
//! dialogs/overlays rendered after the central panel in the eframe `update()`
//! loop. Extracted verbatim from `update.rs` as part of the file-size split;
//! behaviour and order are unchanged -- `update` calls this in the same place
//! (as its last statement) where the cluster ran inline.

use eframe::egui;

use crate::command_palette::render::{render_command_palette, PaletteOutcome};
use crate::command_palette::state::PaletteEntry;

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render the dialog / overlay cluster that follows the central panel.
    /// Extracted verbatim from `update()`; it is the last statement of `update`,
    /// so the `return` early-exits inside the AllocateDataset arm behave exactly
    /// as before (they simply end the frame's update).
    pub(super) fn render_overlays(&mut self, ctx: &egui::Context) {
        // Req 23.2a: clear last frame's frontmost-dialog rect before the overlay
        // pass, so a dialog closed since then leaves no stale crop rect behind.
        // Each overlay below records its rect with its frontmost priority (lower
        // = more frontmost, matching `frontmost_dialog_model`).
        self.snapshot.reset_dialog_rect();

        // Command Palette overlay -- Validates: command-palette Requirement 1.1-1.5, 4.1-4.5
        if self.palette_state.open {
            let all_entries = build_palette_entries(&self.cmd_registry, &self.help.registry);
            let recent = self.recent_palette_commands.clone();
            let palette_outcome =
                render_command_palette(ctx, &mut self.palette_state, &all_entries, &recent);
            match palette_outcome {
                PaletteOutcome::Execute(cmd_id) => {
                    // Add to recent list (most recent first, capped at 10).
                    // Validates: command-palette Requirement 4.4, 5.4
                    self.recent_palette_commands.retain(|c| c != &cmd_id);
                    self.recent_palette_commands.insert(0, cmd_id.clone());
                    self.recent_palette_commands.truncate(10);
                    self.handle_command(&cmd_id);
                }
                PaletteOutcome::Dismissed | PaletteOutcome::None => {}
            }
        }

        // About dialog - Req 13.1, 13.8. The on-dialog Copy affordance
        // (screen-snapshot-scrm Req 27.3) copies the dialog's redacted text
        // through the single copy path and leaves the dialog open.
        if self.show_about {
            let (copy_clicked, rect) = crate::about_dialog::render(ctx, &mut self.show_about);
            // Req 23.2a: About is priority 0 (most frontmost in the chain).
            if let Some(rect) = rect {
                self.snapshot.note_dialog_rect(0, rect);
            }
            if copy_clicked {
                self.copy_frontmost_surface();
            }
        }

        // SCRM overlays (Stream SCRM Phase 2C FEAT-002/003): capture-STOP Save-As
        // and the bare CAPTURE REPLAY picker. Both dispatch through the single
        // front door; render bodies live in `update_dialogs_scrm.rs`.
        self.render_scrm_save_as_overlay(ctx);
        self.render_scrm_replay_picker_overlay(ctx);

        // (The modal Key Configuration Dialog was retired in CR-CH-029; key
        // assignments are now edited in the Keys Workspace, TabKind::KeysEditor.)

        // History list overlay -- Validates: Requirement 19.3, 19.4
        if let Some(entries) = self.show_history_list.clone() {
            let mut keep_open = true;
            let mut selected: Option<String> = None;
            let mut copy_clicked = false;
            let hist_resp = egui::Window::new(ff_i18n::t("ff-desktop-shell-dialogs-history-title"))
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    if entries.is_empty() {
                        ui.label(ff_i18n::t("ff-desktop-shell-dialogs-history-empty"));
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for entry in &entries {
                                    if ui
                                        .selectable_label(
                                            false,
                                            egui::RichText::new(entry).monospace(),
                                        )
                                        .clicked()
                                    {
                                        selected = Some(entry.clone());
                                    }
                                }
                            });
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
                            .clicked()
                        {
                            keep_open = false;
                        }
                        // Direct COPY affordance (Req 27.3): copies this
                        // dialog's redacted text and leaves it open.
                        if ui
                            .button(ff_i18n::t("ff-desktop-shell-dialogs-copy"))
                            .clicked()
                        {
                            copy_clicked = true;
                        }
                    });
                });
            // Req 23.2a: history picker is priority 1.
            if let Some(resp) = hist_resp {
                self.snapshot.note_dialog_rect(1, resp.response.rect);
            }
            if let Some(cmd) = selected {
                self.command_text = cmd;
                self.show_history_list = None;
            } else if !keep_open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.command_text.clear();
                self.show_history_list = None;
            } else if copy_clicked {
                self.copy_frontmost_surface();
            }
        }

        // SWAP tab picker overlay -- Validates: multi-tab-editor Req 18.3-18.5.
        // Lists open tabs as "{n}: {title}" (1-based). Selection by mouse click,
        // or by typing a number + Enter. Escape / Cancel closes without change.
        if self.show_swap_list.is_some() {
            // Build the display rows from the live tab list (1-based).
            let rows: Vec<(usize, String)> = self
                .tabs
                .tabs()
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let label = t.workspace_name.clone().unwrap_or_else(|| t.title.clone());
                    (i + 1, label)
                })
                .collect();

            let mut chosen: Option<usize> = None; // 1-based selection
            let mut cancel = false;
            let mut copy_clicked = false;
            let num_id = egui::Id::new("swap_list_number_input");
            let swap_resp = egui::Window::new(ff_i18n::t("ff-desktop-shell-dialogs-swap-title"))
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (n, label) in &rows {
                                if ui
                                    .selectable_label(
                                        false,
                                        egui::RichText::new(format!("{n}: {label}")).monospace(),
                                    )
                                    .clicked()
                                {
                                    chosen = Some(*n);
                                }
                            }
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(ff_i18n::t("ff-desktop-shell-dialogs-swap-number"));
                        let mut num_text: String =
                            ui.data_mut(|d| d.get_temp::<String>(num_id).unwrap_or_default());
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut num_text)
                                .desired_width(60.0)
                                .id(num_id),
                        );
                        let enter =
                            resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if enter {
                            if let Ok(n) = num_text.trim().parse::<usize>() {
                                if n >= 1 && n <= rows.len() {
                                    chosen = Some(n);
                                }
                            }
                        }
                        ui.data_mut(|d| d.insert_temp(num_id, num_text));
                        if ui
                            .button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
                            .clicked()
                        {
                            cancel = true;
                        }
                        // Direct COPY affordance (Req 27.3): copies this
                        // dialog's redacted text and leaves it open.
                        if ui
                            .button(ff_i18n::t("ff-desktop-shell-dialogs-copy"))
                            .clicked()
                        {
                            copy_clicked = true;
                        }
                    });
                });
            // Req 23.2a: SWAP picker is priority 2.
            if let Some(resp) = swap_resp {
                self.snapshot.note_dialog_rect(2, resp.response.rect);
            }

            if let Some(n) = chosen {
                self.tabs.set_active(n - 1);
                self.show_swap_list = None;
                self.open_error = None;
                ctx.data_mut(|d| d.remove::<String>(num_id));
            } else if cancel || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_swap_list = None;
                ctx.data_mut(|d| d.remove::<String>(num_id));
            } else if copy_clicked {
                self.copy_frontmost_surface();
            }
        }

        // External execution confirmation (shell.mode = prompt).
        // Validates: command-configurator Requirement 3.8
        if let Some(pending) = self.pending_external.clone() {
            self.modal_open = true;
            let mut run_clicked = false;
            let mut cancel_clicked = false;
            let mut copy_clicked = false;
            let cmd_display = if pending.args.is_empty() {
                pending.program.clone()
            } else {
                format!("{} {}", pending.program, pending.args.join(" "))
            };
            let exec_resp = egui::Window::new(ff_i18n::t("ff-desktop-shell-dialogs-exec-title"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(ff_i18n::t("ff-desktop-shell-dialogs-exec-body"));
                    ui.monospace(&cmd_display);
                    ui.horizontal(|ui| {
                        if ui
                            .button(ff_i18n::t("ff-desktop-shell-dialogs-exec-run"))
                            .clicked()
                        {
                            run_clicked = true;
                        }
                        if ui
                            .button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
                            .clicked()
                        {
                            cancel_clicked = true;
                        }
                        // Direct COPY affordance (Req 27.3): copies this
                        // dialog's redacted text and leaves it open.
                        if ui
                            .button(ff_i18n::t("ff-desktop-shell-dialogs-copy"))
                            .clicked()
                        {
                            copy_clicked = true;
                        }
                    });
                });
            // Req 23.2a: external-exec confirmation is priority 5.
            if let Some(resp) = exec_resp {
                self.snapshot.note_dialog_rect(5, resp.response.rect);
            }
            if run_clicked {
                self.pending_external = None;
                self.modal_open = false;
                self.execute_external_now(&pending);
            } else if cancel_clicked {
                // Req 3.8: declining must NOT spawn the process.
                self.pending_external = None;
                self.modal_open = false;
                self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-external-cancelled"));
            } else if copy_clicked {
                self.copy_frontmost_surface();
            }
        }

        // RESET BARE confirmation dialog (CR-CH-021, CR-NR-083) and the
        // Unsaved-workspace-changes dialog. Both carry the on-dialog Copy
        // affordance (Req 27.3); their render bodies live in
        // `update_dialogs_confirm.rs` (file-size split, rust-standards.md).
        self.render_reset_bare_overlay(ctx);
        self.render_unsaved_workspace_overlay(ctx);

        // The SNAPSHOT format picker (Req 4.7) was RETIRED under CR-NR-114: bare
        // SNAPSHOT now places the composite clipboard payload directly, so there
        // is no format to choose and no overlay to render here.

        // core-001 strip: the Catalog Manager dialogs (update_dialogs_catalog.rs)
        // are parked with the catalog domain.
    }
}

/// Build the full list of palette entries from the command registry.
///
/// CR-NR-119 (command-configurator Req 9.3): when a command's own metadata
/// description is EMPTY, a home-grown FFCMD ROOT command inherits its one-line
/// description from the SINGLE `short_description(cmd:<ROOT>)` lookup rather than
/// a duplicated literal, so there is one description per command. A command that
/// is not a root, or whose root has no `cmd:<ROOT>` topic, keeps the empty
/// description (Req 9.4 -- graceful fallback). The derived value is DISPLAY-time
/// only and is never written back into the command registry.
///
/// Validates: command-palette Requirement 2.1; command-configurator Requirement 9.3, 9.4
fn build_palette_entries(
    registry: &ff_command::CommandRegistry,
    help_registry: &ff_help::HelpTopicRegistry,
) -> Vec<PaletteEntry> {
    registry
        .list_all()
        .into_iter()
        .filter_map(|id| {
            registry.metadata(&id).map(|meta| {
                let mut description = meta.description.clone();
                if description.trim().is_empty() {
                    // Inherit the FFCMD root one-liner (empty when the id is not a
                    // root or has no cmd:<ROOT> topic).
                    description = crate::shell::alias_description::inherited_root_description(
                        id.as_str(),
                        help_registry,
                    );
                }
                PaletteEntry {
                    command_id: id.as_str().to_string(),
                    display_name: meta.display_name.clone(),
                    category: meta.category.clone(),
                    description,
                    shortcut: None,
                    enabled: true,
                    score: 0,
                }
            })
        })
        .collect()
}
