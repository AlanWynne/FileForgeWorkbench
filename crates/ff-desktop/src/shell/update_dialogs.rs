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
        // Command Palette overlay -- Validates: command-palette Requirement 1.1-1.5, 4.1-4.5
        if self.palette_state.open {
            let all_entries = build_palette_entries(&self.cmd_registry);
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

        // About dialog - Req 13.1, 13.8
        if self.show_about {
            crate::about_dialog::render(ctx, &mut self.show_about);
        }

        // (The modal Key Configuration Dialog was retired in CR-CH-029; key
        // assignments are now edited in the Keys Workspace, TabKind::KeysEditor.)

        // History list overlay -- Validates: Requirement 19.3, 19.4
        if let Some(entries) = self.show_history_list.clone() {
            let mut keep_open = true;
            let mut selected: Option<String> = None;
            egui::Window::new("Command History")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    if entries.is_empty() {
                        ui.label("No command history.");
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
                    if ui.button("Cancel").clicked() {
                        keep_open = false;
                    }
                });
            if let Some(cmd) = selected {
                self.command_text = cmd;
                self.show_history_list = None;
            } else if !keep_open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.command_text.clear();
                self.show_history_list = None;
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
            let num_id = egui::Id::new("swap_list_number_input");
            egui::Window::new("Swap to Tab")
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
                        ui.label("Number:");
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
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });
                });

            if let Some(n) = chosen {
                self.tabs.set_active(n - 1);
                self.show_swap_list = None;
                self.open_error = None;
                ctx.data_mut(|d| d.remove::<String>(num_id));
            } else if cancel || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_swap_list = None;
                ctx.data_mut(|d| d.remove::<String>(num_id));
            }
        }

        // External execution confirmation (shell.mode = prompt).
        // Validates: command-configurator Requirement 3.8
        if let Some(pending) = self.pending_external.clone() {
            self.modal_open = true;
            let mut run_clicked = false;
            let mut cancel_clicked = false;
            let cmd_display = if pending.args.is_empty() {
                pending.program.clone()
            } else {
                format!("{} {}", pending.program, pending.args.join(" "))
            };
            egui::Window::new("Run external program?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("An external program is about to run:");
                    ui.monospace(&cmd_display);
                    ui.horizontal(|ui| {
                        if ui.button("Run").clicked() {
                            run_clicked = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });
                });
            if run_clicked {
                self.pending_external = None;
                self.modal_open = false;
                self.execute_external_now(&pending);
            } else if cancel_clicked {
                // Req 3.8: declining must NOT spawn the process.
                self.pending_external = None;
                self.modal_open = false;
                self.open_error = Some("External execution cancelled.".to_string());
            }
        }

        // RESET BARE confirmation dialog (CR-CH-021, CR-NR-083). The SAME single
        // dialog is reused for every target list; the only difference is that it
        // now names the profile(s) that will be reset -- one name, or the
        // enumerated list for a subset / ALL.
        // Validates: configuration-system Requirement 19.2, 19.3, 19.6, 19.15
        if self.reset_bare_confirm.is_some() {
            self.modal_open = true;
            let mut confirm_clicked = false;
            let mut cancel_clicked = false;
            // Snapshot the display names for the body without holding a borrow
            // across the closure.
            let profile_names: Vec<String> = self
                .reset_bare_confirm
                .as_ref()
                .map(|t| t.profiles.iter().map(|(name, _)| name.clone()).collect())
                .unwrap_or_default();
            // B077: render as an `egui::Modal` so the background is dimmed and
            // input is captured by the dialog (accessibility Req 2.3 -- modal
            // focus trap). Focus lands on Cancel on the first frame; Tab cycles
            // between Cancel and Confirm; Escape cancels.
            let focus_cancel = self.reset_bare_focus_requested;
            self.reset_bare_focus_requested = false;
            let modal = egui::Modal::new(egui::Id::new("reset_bare_modal")).show(ctx, |ui| {
                ui.set_max_width(460.0);
                ui.heading("Reset to barebones?");
                ui.label(
                    "This archives the current configuration and reopens the \
                     workbench in a minimal barebones state.",
                );
                if profile_names.len() == 1 {
                    ui.label(format!("Profile to be reset: {}", profile_names[0]));
                } else {
                    ui.label(format!("{} profiles will be reset:", profile_names.len()));
                    for name in &profile_names {
                        ui.label(format!("    - {name}"));
                    }
                }
                ui.label(
                    "Each profile's menus, themes, session, config and catalogs \
                     are MOVED (not deleted) to a timestamped folder under that \
                     profile's config-archive/ so you can recover them later.",
                );
                ui.horizontal(|ui| {
                    // Cancel is first and the safe default -- it receives initial
                    // focus (B077). Its stable id anchors the modal focus trap.
                    let cancel = ui
                        .add(egui::Button::new("Cancel").min_size(egui::vec2(96.0, 0.0)))
                        .on_hover_text("Close without changing any configuration");
                    let confirm =
                        ui.add(egui::Button::new("Confirm reset").min_size(egui::vec2(120.0, 0.0)));
                    if focus_cancel {
                        cancel.request_focus();
                    }
                    if cancel.clicked() {
                        cancel_clicked = true;
                    }
                    if confirm.clicked() {
                        confirm_clicked = true;
                    }
                });
            });
            // Escape (or clicking the dimmed background) cancels -- the safe
            // default that leaves configuration untouched (Req 19.3).
            if modal.should_close() {
                cancel_clicked = true;
            }
            if confirm_clicked {
                self.modal_open = false;
                if let Some(target) = self.reset_bare_confirm.take() {
                    self.execute_reset_bare(&target);
                }
            } else if cancel_clicked {
                // Req 19.3: cancelling leaves all configuration untouched.
                self.reset_bare_confirm = None;
                self.modal_open = false;
                self.open_error = None;
            }
        }

        // Unsaved workspace changes dialog -- Validates: workspace-model Requirement 2.5
        if self.show_unsaved_workspace_dialog {
            let mut save_clicked = false;
            let mut discard_clicked = false;
            let mut cancel_clicked = false;
            let ws_name = self
                .active_workspace
                .as_ref()
                .map(|ws| ws.name.clone())
                .unwrap_or_default();
            egui::Window::new("Unsaved Workspace Changes")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Workspace '{}' has unsaved changes.", ws_name));
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            save_clicked = true;
                        }
                        if ui.button("Discard").clicked() {
                            discard_clicked = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });
                });
            if save_clicked {
                self.save_workspace_to(None);
                self.show_unsaved_workspace_dialog = false;
                if let Some(path) = self.pending_workspace_open.take() {
                    self.open_workspace_force(&path);
                }
            } else if discard_clicked {
                self.show_unsaved_workspace_dialog = false;
                if let Some(path) = self.pending_workspace_open.take() {
                    if let Some(ws) = self.active_workspace.as_mut() {
                        ws.is_modified = false;
                    }
                    self.open_workspace_force(&path);
                }
            } else if cancel_clicked {
                self.show_unsaved_workspace_dialog = false;
                self.pending_workspace_open = None;
            }
        }

        // Catalog Manager Dialogs. Extracted verbatim to
        // `update_dialogs_catalog.rs`; called as the LAST statement of
        // `render_overlays` so the early `return`s inside its AllocateDataset arm
        // end the frame exactly as before. Req 3.1-3.8, 4.1-4.5
        self.render_catalog_dialogs(ctx);
    }
}

/// Build the full list of palette entries from the command registry.
///
/// Validates: command-palette Requirement 2.1
fn build_palette_entries(registry: &ff_command::CommandRegistry) -> Vec<PaletteEntry> {
    registry
        .list_all()
        .into_iter()
        .filter_map(|id| {
            registry.metadata(&id).map(|meta| PaletteEntry {
                command_id: id.as_str().to_string(),
                display_name: meta.display_name.clone(),
                category: meta.category.clone(),
                description: meta.description.clone(),
                shortcut: None,
                enabled: true,
                score: 0,
            })
        })
        .collect()
}
