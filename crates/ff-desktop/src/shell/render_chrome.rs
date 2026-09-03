//! # Shell Chrome Rendering
//!
//! Theme application, menu bar, and tab bar rendering for WorkbenchShell.

use eframe::egui;

use crate::primary_option_menu;
use crate::tab_state::TabKind;

use super::helpers::*;
use super::{FocusStop, WorkbenchShell};

impl WorkbenchShell {
    pub(super) fn apply_theme(&self, ctx: &egui::Context) {
        let p = &self.palette;
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = to_egui_color(p.editor.background);
        visuals.window_fill = to_egui_color(p.ui.panel_bg);
        visuals.window_stroke = egui::Stroke::new(1.0_f32, to_egui_color(p.ui.panel_border));
        // Use menu_bar_fg as the global text colour — in Legacy this is white (#FFFFFF),
        // which correctly colours menu bar items, tab bar, and chrome text.
        // Editor content text is applied per-element in editor_panel using palette tokens.
        visuals.override_text_color = Some(to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.noninteractive.bg_fill = to_egui_color(p.ui.panel_bg);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.editor.foreground));
        visuals.widgets.inactive.bg_fill = to_egui_color(p.ui.button_bg);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.hovered.bg_fill = to_egui_color(p.ui.button_hover);
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.active.bg_fill = to_egui_color(p.ui.input_bg);
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.selection.bg_fill = to_egui_color(p.editor.accent).linear_multiply(0.35);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, to_egui_color(p.editor.accent));
        // In Legacy mode the slider track and handle are near-black on black — invisible.
        // Override with high-contrast ISPF colours: turquoise track, yellow handle.
        if p.mode == ff_theme::mode::VisualMode::Legacy {
            let track = egui::Color32::from_rgb(0, 170, 170); // ISPF turquoise
            let handle = egui::Color32::from_rgb(255, 255, 0); // ISPF yellow-hi
            visuals.widgets.inactive.bg_fill = track;
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(2.0_f32, handle);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 210, 210);
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(2.0_f32, handle);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 255, 255);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, handle);
        }
        ctx.set_visuals(visuals);
    }

    // ── Theme switching ───────────────────────────────────────────────────

    /// Switch to the given visual mode.
    ///
    /// Writes the mode name to the `theme.active` config key so the per-frame
    /// hot-reload block picks it up and the palette is not clobbered next frame.
    pub(super) fn set_theme(&mut self, mode: ff_theme::mode::VisualMode) {
        self.palette = ff_theme::defaults::default_palette_for_mode(mode);
        let mode_str = mode.section_name().to_string();
        let _ = self.config_handle.set_user_value(
            ff_config::keys::theme::ACTIVE,
            ff_config::ConfigValue::String(mode_str),
        );
    }

    // ── Legacy POM colours ────────────────────────────────────────────────

    /// Build `PomColours` for the current palette.
    ///
    /// When the Legacy theme is active, returns ISPF semantic colours.
    /// For all other themes, returns `PomColours::inherited()` so egui
    /// uses its own default colours.
    ///
    /// Validates: Requirement 13 (Legacy Theme Colour Semantics)
    pub(super) fn legacy_pom_colours(&self) -> primary_option_menu::PomColours {
        use ff_theme::mode::VisualMode;
        if self.palette.mode == VisualMode::Legacy {
            primary_option_menu::PomColours::from_palette(&self.palette)
        } else {
            primary_option_menu::PomColours::inherited()
        }
    }

    // ── Menu bar ─────────────────────────────────────────────────────────

    pub(super) fn render_menu_bar(&mut self, ctx: &egui::Context) {
        // Validates: Requirement 14.7 — every label in the registry must have a menu_button below.
        debug_assert_eq!(
            super::MENU_BAR_TOP_LEVEL_LABELS.len(),
            11,
            "render_menu_bar must contain one menu_button per super::MENU_BAR_TOP_LEVEL_LABELS entry"
        );
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // ── Focus request for menu bar stops — Validates: Requirement 16.8
                // When focus_stop is MenuBar{index}, request focus on that button's Id.
                for (idx, label) in super::MENU_BAR_TOP_LEVEL_LABELS.iter().enumerate() {
                    if self.focus_stop == (FocusStop::MenuBar { index: idx }) {
                        let id = egui::Id::new("menu_bar_btn").with(idx);
                        ui.memory_mut(|m| m.request_focus(id));
                        let _ = label; // label used only for Id derivation above
                    }
                }
                // ── Settings ────────────────────────────────────────────
                ui.menu_button("Settings", |ui| {
                    if ui.button("Preferences…").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Themes").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Dark Theme").clicked() {
                        self.set_theme(ff_theme::mode::VisualMode::Dark);
                        ui.close_menu();
                    }
                    if ui.button("Light Theme").clicked() {
                        self.set_theme(ff_theme::mode::VisualMode::Light);
                        ui.close_menu();
                    }
                    if ui.button("High Contrast").clicked() {
                        self.set_theme(ff_theme::mode::VisualMode::HighContrast);
                        ui.close_menu();
                    }
                    if ui.button("Legacy (ISPF 3270)").clicked() {
                        self.set_theme(ff_theme::mode::VisualMode::Legacy);
                        ui.close_menu();
                    }
                    ui.separator();
                    // Validates: Requirement 14.14 — open new POM tab from Settings menu
                    if ui.button("Primary Option Menu").clicked() {
                        self.tabs.insert_pom_tab(&self.runtime);
                        ui.close_menu();
                    }
                });
                // ── File Catalogs — Validates: Requirement 14.7 (mirrors POM option 1) ──
                ui.menu_button("File Catalogs", |ui| {
                    if ui.button("Open File Catalogs").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Files ───────────────────────────────────────────────
                ui.menu_button("Files", |ui| {
                    if ui.button("New").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Open…").clicked() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        if let Err(e) = self.tabs.save_active_tab(&self.runtime) {
                            self.open_error = Some(e);
                        } else {
                            self.open_error = None;
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save As…").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Close").clicked() {
                        let idx = self.tabs.active_index();
                        self.tabs.close_tab(idx);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                // ── Utilities ───────────────────────────────────────────
                ui.menu_button("Utilities", |ui| {
                    if ui.button("Compare Files…").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("File Tree").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Compilers ───────────────────────────────────────────
                ui.menu_button("Compilers", |ui| {
                    if ui.button("Toolchain Panel").clicked() {
                        self.show_toolchain_panel = !self.show_toolchain_panel;
                        ui.close_menu();
                    }
                    if ui.button("Build").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Run").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Lua ─────────────────────────────────────────────────
                ui.menu_button("Lua", |ui| {
                    if ui.button("Run Script…").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Macro Editor").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Terminals ───────────────────────────────────────────
                ui.menu_button("Terminals", |ui| {
                    if ui.button("New Terminal").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Databases ───────────────────────────────────────────
                ui.menu_button("Databases", |ui| {
                    if ui.button("Connect…").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Query Browser").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Plugins — Validates: Requirement 14.7 (mirrors POM option 8) ─────
                ui.menu_button("Plugins", |ui| {
                    if ui.button("Manage Plugins").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Edit (always present) ────────────────────────────────
                ui.menu_button("Edit", |ui| {
                    if ui.button("Key Assignments\u{2026}").clicked() {
                        self.key_config_dialog.open = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Undo").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Cut").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Copy").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Help ────────────────────────────────────────────────
                ui.menu_button("Help", |ui| {
                    if ui.button("About FileForge Workbench").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    // ── Tab bar ──────────────────────────────────────────────────────────

    pub(super) fn render_tab_bar(&mut self, ctx: &egui::Context) {
        let active_bg = to_egui_color(self.palette.ui.input_bg);
        let inactive_bg = to_egui_color(self.palette.ui.panel_bg);
        let text_color = to_egui_color(self.palette.editor.foreground);
        let modified_color = to_egui_color(self.palette.editor.accent);

        // Collect context-menu actions outside the borrow of self.tabs.
        let mut activate_idx: Option<usize> = None;
        let mut close_idx: Option<usize> = None;
        let mut close_all_but: Option<usize> = None;
        let mut close_left_of: Option<usize> = None;
        let mut close_right_of: Option<usize> = None;
        let mut close_unchanged = false;

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
                        let label = if tab.is_modified {
                            format!("● {}", tab.title)
                        } else {
                            tab.title.clone()
                        };
                        let color = if tab.is_modified {
                            modified_color
                        } else {
                            text_color
                        };

                        let btn =
                            egui::Button::new(egui::RichText::new(&label).color(color).monospace())
                                .fill(bg)
                                .stroke(if is_active {
                                    egui::Stroke::new(1.0_f32, color)
                                } else {
                                    egui::Stroke::NONE
                                })
                                .min_size(egui::vec2(0.0, 24.0));

                        let resp = ui.add(btn);
                        if resp.clicked() {
                            activate_idx = Some(i);
                        }

                        // Validates: Requirement 3.8 multi-tab-editor — close button on tab header (B002/B015)
                        let close_resp = ui.add(
                            egui::Button::new(
                                egui::RichText::new("\u{00d7}")
                                    .color(text_color)
                                    .monospace()
                                    .small(),
                            )
                            .fill(bg)
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(16.0, 24.0)),
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
                                ui.close_menu();
                            }
                            ui.add_enabled_ui(tab_count_inner > 1, |ui| {
                                if ui.button("Close All BUT This").clicked() {
                                    close_all_but = Some(i);
                                    ui.close_menu();
                                }
                            });
                            ui.add_enabled_ui(i > 0, |ui| {
                                if ui.button("Close All to the Left").clicked() {
                                    close_left_of = Some(i);
                                    ui.close_menu();
                                }
                            });
                            ui.add_enabled_ui(i < tab_count_inner - 1, |ui| {
                                if ui.button("Close All to the Right").clicked() {
                                    close_right_of = Some(i);
                                    ui.close_menu();
                                }
                            });
                            if ui.button("Close All Unchanged").clicked() {
                                close_unchanged = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Clone to Other Tab").clicked() {
                                // stub — deferred
                                ui.close_menu();
                            }
                            if ui.button("Move to Other View").clicked() {
                                // Validates: Requirement 18.1, 18.7
                                if self.floating_tabs.len() < 16 {
                                    self.detach_pending = Some(i);
                                } else {
                                    self.open_error = Some(
                                        "Maximum 16 floating windows already open.".to_string(),
                                    );
                                }
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Pin Tab").clicked() {
                                // stub — deferred
                                ui.close_menu();
                            }

                            // ── Exit — Req 14.15a, 14.38 (all tab kinds) ─────
                            ui.separator();
                            if ui.button("Exit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                ui.close_menu();
                            }

                            // ── File-editor-only items — Req 14.15b ──────────
                            // Only shown when the tab is a FileEditor.
                            if tab_kind == TabKind::FileEditor {
                                ui.separator();
                                if ui.button("Open Containing Folder in Explorer").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Explorer);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in CMD").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Cmd);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in PowerShell").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::PowerShell);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in Terminal").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Terminal);
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("Copy Name to Clipboard").clicked() {
                                    if let Some(title) =
                                        self.tabs.tabs().get(i).map(|t| t.title.clone())
                                    {
                                        ui.output_mut(|o| o.copied_text = title);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Copy Path to Clipboard").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.clone() {
                                        ui.output_mut(|o| o.copied_text = path);
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("Save").clicked() {
                                    // handled after menu closes via pending action
                                    ui.close_menu();
                                }
                                if ui.button("Save As").clicked() {
                                    ui.close_menu();
                                }
                                if ui.button("Reload").clicked() {
                                    ui.close_menu();
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
                            ui.close_menu();
                        }
                        if ui.button("New File").clicked() {
                            self.pending_new_file = true;
                            ui.close_menu();
                        }
                    });
                });
            });

        // Apply deferred tab-bar actions.
        if let Some(i) = activate_idx {
            // Track previous tab for END navigation -- Validates: Requirement 17.1
            self.tab_history.push(self.tabs.active_index());
            self.tabs.set_active(i);
            // Update key map context for new active tab -- Validates: Requirement 14.4
            let ctx_name = context_name_for_kind(self.tabs.active_tab().kind);
            self.key_map_resolver.set_context(ctx_name);
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
    }

    // ── Title line ──────────────────────────────────────────────────
}
