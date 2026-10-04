//! # Shell Tab-Bar Rendering
//!
//! `render_tab_bar` -- the flat top tab bar (headers, close buttons, per-tab and
//! empty-space context menus, drag-to-detach) and its deferred-action
//! application. Moved out of `render_chrome.rs` verbatim as part of the Phase 2
//! task 2.2 file-size split; behaviour, method name, signature, and visibility
//! are unchanged.

use eframe::egui;

use crate::tab_state::TabKind;

use super::helpers::{open_containing_folder, to_egui_color, FolderOpenMode};
use super::WorkbenchShell;

impl WorkbenchShell {
    pub(super) fn render_tab_bar(&mut self, ctx: &egui::Context) {
        // CR-NR-092 (Slice 2b): when the Workspace area is split, each region
        // draws its OWN tab bar inside the split central panel
        // (`render_split_central`), so the shared flat top tab bar is suppressed.
        // The unsplit path below is byte-identical to Slice 2a.
        if self.tabs.is_split() {
            return;
        }
        // Validates: Requirement 21.6 -- render_tab_bar reads the tab_bar.* palette
        // group (previously dead: bg reused ui.input_bg/panel_bg and text reused
        // editor.foreground). Active/inactive tabs now get distinct bg + text.
        let active_bg = to_egui_color(self.palette.tab_bar.active_bg);
        let inactive_bg = to_egui_color(self.palette.tab_bar.inactive_bg);
        let active_text = to_egui_color(self.palette.tab_bar.active_text);
        let inactive_text = to_egui_color(self.palette.tab_bar.inactive_text);
        let modified_color = to_egui_color(self.palette.editor.accent);

        // Collect context-menu actions outside the borrow of self.tabs.
        let mut activate_idx: Option<usize> = None;
        let mut close_idx: Option<usize> = None;
        let mut close_all_but: Option<usize> = None;
        let mut close_left_of: Option<usize> = None;
        let mut close_right_of: Option<usize> = None;
        let mut close_unchanged = false;
        // CR-CH-035 (Req 18.6): a tab whose header was dragged out of the bar.
        let mut detach_drag_idx: Option<usize> = None;

        egui::TopBottomPanel::top("tab_bar")
            .min_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let tab_count = self.tabs.len();
                    let active_idx_cur = self.tabs.active_index();

                    for i in 0..tab_count {
                        let tab = &self.tabs.tabs()[i];
                        // Validates: Requirement 18.4 — skip tabs that are floating.
                        if tab.is_floating {
                            continue;
                        }
                        let is_active = i == active_idx_cur;
                        let tab_kind = tab.kind;

                        let bg = if is_active { active_bg } else { inactive_bg };
                        let tab_text = if is_active {
                            active_text
                        } else {
                            inactive_text
                        };
                        // The Tab_Header short label, centralised in
                        // `tab_header_label` (CR-CH-042): workspace_name > POM
                        // short `POM` > non-Home menu `[TITLE]` > Kind title.
                        let base_title = self.tab_header_label(tab);
                        let label = if tab.is_modified {
                            format!("● {}", base_title)
                        } else {
                            base_title
                        };
                        let color = if tab.is_modified {
                            modified_color
                        } else {
                            tab_text
                        };

                        // CR-CH-023 Req 16.9: tab headers are clickable but not
                        // keyboard Tab stops (tab switching is via the SWAP
                        // command / mouse). CR-CH-035 (B045 drag-out, Req 18.6):
                        // also sense DRAG so the header can be dragged out of the
                        // bar to detach. `click_and_drag` keeps the click and
                        // drag but egui skips it for keyboard Tab (not FOCUSABLE).
                        let btn =
                            egui::Button::new(egui::RichText::new(&label).color(color).monospace())
                                .fill(bg)
                                .stroke(if is_active {
                                    egui::Stroke::new(1.0_f32, color)
                                } else {
                                    egui::Stroke::NONE
                                })
                                .min_size(egui::vec2(0.0, 24.0))
                                .sense(egui::Sense::click_and_drag());

                        let resp = ui.add(btn);
                        if resp.clicked() {
                            activate_idx = Some(i);
                        }
                        // CR-CH-035 (Req 18.6): dragging a Tab_Header more than
                        // 20px beyond the tab-bar boundary detaches it into a
                        // Detached_Workspace. We detect the drag on release: if
                        // the pointer moved >20px vertically below the bar (or the
                        // total drag exceeded the threshold and ended outside the
                        // bar rect), request a detach for this tab. The real
                        // cross-window release position is applied by the OS; the
                        // headless-testable part is "drag beyond threshold on a
                        // tab header sets detach_pending".
                        if resp.drag_stopped() {
                            let bar_bottom = ui.max_rect().bottom();
                            let released = ui
                                .ctx()
                                .input(|inp| inp.pointer.interact_pos())
                                .unwrap_or(resp.rect.center());
                            let moved = resp.drag_delta().length()
                                + ui.ctx().input(|inp| {
                                    inp.pointer
                                        .press_origin()
                                        .map(|o| (released - o).length())
                                        .unwrap_or(0.0)
                                });
                            let outside_bar = released.y > bar_bottom + 20.0;
                            if (outside_bar || moved > 20.0) && !tab.is_floating {
                                detach_drag_idx = Some(i);
                            }
                        }
                        // CR-CH-023: tab headers are no longer keyboard focus
                        // stops (Req 16.9), so there is no tab-header focus ring
                        // indicator here anymore.

                        // Validates: Requirement 3.8 multi-tab-editor — close button on tab header (B002/B015)
                        // CR-CH-023 Req 16.9: click-only sense (not a Tab stop).
                        let close_resp = ui.add(
                            egui::Button::new(
                                egui::RichText::new("\u{00d7}")
                                    .color(tab_text)
                                    .monospace()
                                    .small(),
                            )
                            .fill(bg)
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(16.0, 24.0))
                            .sense(egui::Sense::CLICK),
                        );
                        if close_resp.clicked() {
                            close_idx = Some(i);
                        }
                        close_resp.on_hover_text("Close tab");
                        // Validates: Requirement 14.15, 14.15a, 14.15b, 14.15c
                        resp.context_menu(|ui| {
                            let tab_count_inner = self.tabs.len();
                            // ── Universal items (all tab kinds) — Req 14.15a ──
                            if ui.button("Close").clicked() {
                                close_idx = Some(i);
                                ui.close();
                            }
                            ui.add_enabled_ui(tab_count_inner > 1, |ui| {
                                if ui.button("Close All BUT This").clicked() {
                                    close_all_but = Some(i);
                                    ui.close();
                                }
                            });
                            ui.add_enabled_ui(i > 0, |ui| {
                                if ui.button("Close All to the Left").clicked() {
                                    close_left_of = Some(i);
                                    ui.close();
                                }
                            });
                            ui.add_enabled_ui(i < tab_count_inner - 1, |ui| {
                                if ui.button("Close All to the Right").clicked() {
                                    close_right_of = Some(i);
                                    ui.close();
                                }
                            });
                            if ui.button("Close All Unchanged").clicked() {
                                close_unchanged = true;
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("Clone to Other Tab").clicked() {
                                // stub — deferred
                                ui.close();
                            }
                            if ui.button("Move to Other View").clicked() {
                                // Validates: Requirement 18.1, 18.7
                                if self.detach_split.floating_tabs.len() < 16 {
                                    self.detach_split.detach_pending = Some(i);
                                } else {
                                    self.open_error = Some(
                                        "Maximum 16 floating windows already open.".to_string(),
                                    );
                                }
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("Pin Tab").clicked() {
                                // stub — deferred
                                ui.close();
                            }

                            // ── Exit — Req 14.15a, 14.38 (all tab kinds) ─────
                            ui.separator();
                            if ui.button("Exit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                ui.close();
                            }

                            // ── File-editor-only items — Req 14.15b ──────────
                            // Only shown when the tab is a FileEditor.
                            if tab_kind == TabKind::FileEditor {
                                ui.separator();
                                if ui.button("Open Containing Folder in Explorer").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Explorer);
                                    }
                                    ui.close();
                                }
                                if ui.button("Open Containing Folder in CMD").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Cmd);
                                    }
                                    ui.close();
                                }
                                if ui.button("Open Containing Folder in PowerShell").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::PowerShell);
                                    }
                                    ui.close();
                                }
                                if ui.button("Open Containing Folder in Terminal").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Terminal);
                                    }
                                    ui.close();
                                }
                                ui.separator();
                                if ui.button("Copy Name to Clipboard").clicked() {
                                    if let Some(title) =
                                        self.tabs.tabs().get(i).map(|t| t.title.clone())
                                    {
                                        ui.ctx().copy_text(title);
                                    }
                                    ui.close();
                                }
                                if ui.button("Copy Path to Clipboard").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.clone() {
                                        ui.ctx().copy_text(path);
                                    }
                                    ui.close();
                                }
                                ui.separator();
                                if ui.button("Save").clicked() {
                                    // handled after menu closes via pending action
                                    ui.close();
                                }
                                if ui.button("Save As").clicked() {
                                    ui.close();
                                }
                                if ui.button("Reload").clicked() {
                                    ui.close();
                                }
                            }
                        });
                    }

                    // ── Empty tab-bar space right-click — Req 14.9 ──────
                    let bar_resp = ui.interact(
                        ui.available_rect_before_wrap(),
                        ui.id().with("tab_bar_empty"),
                        egui::Sense::click(),
                    );
                    bar_resp.context_menu(|ui| {
                        if ui.button("New").clicked() {
                            self.pending_new_pom = true;
                            ui.close();
                        }
                        if ui.button("New File").clicked() {
                            self.pending_new_file = true;
                            ui.close();
                        }
                    });
                });
            });

        // Apply deferred tab-bar actions.
        if let Some(i) = activate_idx {
            // Track previous tab for END navigation -- Validates: Requirement 17.1
            self.tab_history.push(self.tabs.active_index());
            self.tabs.set_active(i);
            // Update key map context for the new active tab -- Validates:
            // Requirement 14.4; CR-NR-090 B.2 (workspace-kinds Req 4.3): the
            // context is the Kind's configured key_list if set, else the base
            // kind context.
            let ctx_name = self.key_list_context_for_tab(self.tabs.active_tab());
            self.key_map_resolver.set_context(ctx_name.as_deref());
            self.key_label_bar
                .update(self.key_map_resolver.active_key_map());
        }
        if let Some(i) = close_idx {
            self.tabs.close_tab(i);
        }
        if let Some(pivot) = close_all_but {
            let count = self.tabs.len();
            // Close right-of-pivot first (indices stable), then left.
            for i in (pivot + 1..count).rev() {
                self.tabs.close_tab(i);
            }
            for i in (0..pivot).rev() {
                self.tabs.close_tab(i);
            }
        }
        if let Some(pivot) = close_left_of {
            for i in (0..pivot).rev() {
                self.tabs.close_tab(i);
            }
        }
        if let Some(pivot) = close_right_of {
            let count = self.tabs.len();
            for i in (pivot + 1..count).rev() {
                self.tabs.close_tab(i);
            }
        }
        if close_unchanged {
            let count = self.tabs.len();
            for i in (0..count).rev() {
                if !self.tabs.tabs()[i].is_modified {
                    self.tabs.close_tab(i);
                }
            }
        }
        // CR-CH-035 (Req 18.6): a tab header dragged out of the bar detaches,
        // subject to the shared 16-window limit. Sets detach_pending; the frame
        // loop consumes it (same path as "Move to Other View" / SPLIT DETACH).
        if let Some(i) = detach_drag_idx {
            if self.detach_split.floating_tabs.len() < 16 {
                self.detach_split.detach_pending = Some(i);
                self.open_error = None;
            } else {
                self.open_error =
                    Some("Maximum number of detached Workspaces (16) reached.".to_string());
            }
        }
    }
}
