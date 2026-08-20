//! # Shell Panel Rendering
//!
//! Title line, command field, key label bar, status bar, and central panel rendering.

use eframe::egui;

use crate::primary_option_menu;
use ff_core::LifecyclePhase;
use ff_keys::FunctionKey;

use crate::catalog_manager_dialog::{self, NewCatalogForm};
use crate::dataset_alloc_dialog::{self};
use crate::editor_panel;
use crate::file_explorer_panel;
use crate::files_panel;
use crate::tab_state::TabKind;
use crate::toolchain_panel;

use super::helpers::*;
use super::{FocusStop, WorkbenchShell};

impl WorkbenchShell {
    pub(super) fn render_title_line(&self, ctx: &egui::Context) {
        use ff_theme::mode::VisualMode;
        let text = super::title_line_text(self.tabs.active_tab());
        let is_legacy = self.palette.mode == VisualMode::Legacy;
        let is_pom = self.tabs.active_tab().kind == crate::tab_state::TabKind::PrimaryOptionMenu;
        egui::TopBottomPanel::top("title_line").show(ctx, |ui| {
            if is_pom {
                // POM title: black background, blue text, centered
                let bg = egui::Color32::BLACK;
                let fg = egui::Color32::from_rgb(0x00, 0x55, 0xFF);
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, bg);
                ui.centered_and_justified(|ui| {
                    ui.colored_label(fg, egui::RichText::new(&text).monospace().strong());
                });
            } else if is_legacy {
                // Validates: Requirement 17.8 — Legacy: blue bg, white text
                let bg = to_egui_color(self.palette.ui.primary_menu_bg);
                let fg = to_egui_color(self.palette.ui.menu_bar_fg);
                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(rect, 0.0, bg);
                ui.colored_label(fg, egui::RichText::new(text).monospace());
            } else {
                ui.label(egui::RichText::new(text).monospace());
            }
        });
    }

    // ── Command field ────────────────────────────────────────────────────

    pub(super) fn render_command_field(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("command_field").show(ctx, |ui| {
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
                if self.command_field_focus_requested && !self.modal_open {
                    self.command_field_focus_requested = false;
                    ctx.memory_mut(|m| m.request_focus(cmd_id));
                }
                // Validates: Requirement 8.1 — Enter while field has focus submits the command.
                // Use lost_focus() to catch the frame egui clears focus on Enter for
                // single-line TextEdit (egui 0.29 surrenders focus on Enter).
                // Also check has_focus() as a fallback for frames where focus is retained.
                let field_has_focus = response.has_focus() || response.lost_focus();
                if field_has_focus
                    && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                    && !self.command_text.is_empty()
                {
                    let cmd = self.command_text.trim().to_string();
                    self.command_text.clear();
                    self.handle_command(&cmd);
                    // Return focus to the command field after every command execution.
                    self.focus_stop = FocusStop::CommandField;
                    self.command_field_focus_requested = true;
                }
            });
        });
    }

    // ── Key label bar ─────────────────────────────────────────────────────

    /// Render the ISPF-style function key label bar in the footer.
    ///
    /// Shows only assigned slots as `Fn label` pairs.
    /// Validates: Requirement 4.1, 4.2, 4.3
    pub(super) fn render_key_label_bar(&mut self, ctx: &egui::Context) {
        if !self.key_bar_visible {
            return;
        }
        let key_color = to_egui_color(self.palette.editor.accent);
        let label_color = to_egui_color(self.palette.editor.foreground);
        let mut clicked_key: Option<FunctionKey> = None;
        egui::TopBottomPanel::bottom("key_label_bar").show(ctx, |ui| {
            for row in [self.key_label_bar.row0(), self.key_label_bar.row1()] {
                ui.horizontal(|ui| {
                    for slot in row {
                        let key = slot.key;
                        let btn_text = if let Some(lbl) = &slot.label {
                            format!("{} {}", key.display_name(), lbl)
                        } else {
                            key.display_name().to_string()
                        };
                        let enabled = slot.label.is_some();
                        let tooltip = self
                            .key_map_resolver
                            .active_key_map()
                            .get_plain(key)
                            .map(|b| b.command().to_string())
                            .unwrap_or_default();
                        let resp = ui.add_enabled(
                            enabled,
                            egui::Button::new(
                                egui::RichText::new(&btn_text)
                                    .color(if enabled { label_color } else { key_color })
                                    .monospace()
                                    .small(),
                            )
                            .frame(false),
                        );
                        if enabled && !tooltip.is_empty() {
                            resp.clone().on_hover_text(&tooltip);
                        }
                        if resp.clicked() && enabled {
                            clicked_key = Some(key);
                        }
                    }
                });
            }
        });
        if let Some(key) = clicked_key {
            if let Some(cmd) = self
                .key_map_resolver
                .active_key_map()
                .get_plain(key)
                .map(|b| b.command().to_string())
            {
                self.handle_command(&cmd);
            }
        }
    }

    // ── Status bar ───────────────────────────────────────────────────────

    pub(super) fn render_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let phase_label = match self.app.phase() {
                    LifecyclePhase::Running => "RUNNING",
                    LifecyclePhase::Initializing => "STARTING",
                    LifecyclePhase::ShuttingDown => "SHUTTING DOWN",
                    LifecyclePhase::Terminated => "TERMINATED",
                };
                ui.label(phase_label);
                ui.separator();

                let tab = self.tabs.active_tab();
                let line = tab.cursor.cursor_line();
                let col = tab.cursor.cursor_column();
                // Requirement 7.1: format "Ln {line}, Col {col}" (1-based)
                ui.label(format!("Ln {line}, Col {col}"));
                ui.separator();
                // Requirement 7.3: real encoding from document
                ui.label(tab.encoding_label());
                ui.separator();
                // Requirement 7.1 (view-zoom) — zoom indicator when non-zero
                {
                    use ff_zoom::ZoomIndicatorState;
                    if let ZoomIndicatorState::Visible { text, .. } =
                        ZoomIndicatorState::from_offset(self.zoom.offset())
                    {
                        ui.colored_label(to_egui_color(self.palette.editor.accent), text);
                        ui.separator();
                    }
                }
                // Requirement 7.4: real line count
                ui.label(format!("{} lines", tab.line_count));
                ui.separator();
                // Requirement 6.5: modified indicator
                if tab.is_modified {
                    ui.colored_label(to_egui_color(self.palette.editor.accent), "●");
                    ui.separator();
                }

                if let Some(err) = &self.open_error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.separator();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("FileForge Workbench v0.1.0");
                });
            });
        });
    }

    // ── File-open dialog ──────────────────────────────────────────────────

    /// Spawn a native file-open dialog on a blocking thread.
    ///
    /// When the user picks a file the path is written into `pending_open`;
    /// the next egui frame will pick it up and open the tab.
    pub(super) fn open_file_dialog(&self) {
        let pending = self.pending_open.clone();
        self.runtime.spawn_blocking(move || {
            if let Some(handle) = rfd::FileDialog::new().pick_file() {
                let path = handle.to_string_lossy().into_owned();
                *pending.lock().expect("pending lock") = Some(path);
            }
        });
    }

    // ── Central panel ────────────────────────────────────────────────────

    pub(super) fn render_central_panel(&mut self, ctx: &egui::Context) {
        // ── Toolchain Panel (bottom dock) ────────────────────────────────
        if self.show_toolchain_panel {
            egui::TopBottomPanel::bottom("toolchain_panel")
                .resizable(true)
                .min_height(160.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Toolchain Panel").monospace().strong());
                        if ui.small_button("✕").clicked() {
                            self.show_toolchain_panel = false;
                        }
                    });
                    ui.separator();
                    if let Some((file, line, col)) =
                        toolchain_panel::render(ui, &mut self.toolchain_panel)
                    {
                        // Navigate editor to the clicked diagnostic location.
                        // Req 16.7, 18.6 — open the file if not already open,
                        // then scroll to the target line.
                        let _ = self.tabs.open_file(&file, &self.runtime);
                        self.nav_manager.locate(&line.to_string(), &mut self.tabs);
                        let _ = col; // column navigation deferred to Phase W follow-up
                    }
                });
        }

        // Validates: Requirement 14.8 — central panel dispatches on tab kind
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tabs.active_tab().kind {
                TabKind::PrimaryOptionMenu => {
                    // Validates: Requirement 14.1, 14.2-14.5, 14.39, 14.40, 14.41, 14.42
                    // Validates: Requirement 13 (Legacy theme semantic colours)
                    let pom_colours = self.legacy_pom_colours();
                    let focused_pom_option = match self.focus_stop {
                        FocusStop::PomOption { index } => Some(index),
                        _ => None,
                    };
                    let pom_result = primary_option_menu::render(
                        ui,
                        self.pom_calendar_offset,
                        pom_colours,
                        focused_pom_option,
                    );
                    if let Some(nav) = pom_result.calendar_nav {
                        match nav {
                            primary_option_menu::CalendarNav::Prev => self.pom_calendar_offset -= 1,
                            primary_option_menu::CalendarNav::Next => self.pom_calendar_offset += 1,
                        }
                    }
                    if let Some(pom_action) = pom_result.action {
                        match pom_action {
                            primary_option_menu::PomAction::Navigate(key) => {
                                self.handle_command(&key.to_string());
                            }
                            primary_option_menu::PomAction::Exit => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    }
                }
                TabKind::FilesPanel => {
                    // Validates: Requirement 1.1, 1.7
                    let action = files_panel::render(ui, &mut self.files_panel);
                    match action {
                        files_panel::FilesPanelAction::ReturnToPom => {
                            self.pending_return_to_pom = true;
                        }
                        files_panel::FilesPanelAction::NewCatalog => {
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                // Req 12.1, 12.2 — pre-populate with configured defaults
                                let mf_root = self
                                    .config_handle
                                    .get_string(ff_config::keys::catalogs::DEFAULT_MAINFRAME_ROOT)
                                    .unwrap_or_default();
                                let posix_root = self
                                    .config_handle
                                    .get_string(ff_config::keys::catalogs::DEFAULT_POSIX_ROOT)
                                    .unwrap_or_default();
                                self.files_panel.dialog = files_panel::FilesDialogState::NewCatalog(
                                    NewCatalogForm::with_defaults(mf_root, posix_root),
                                );
                            }
                        }
                        files_panel::FilesPanelAction::EditCatalog(name) => {
                            // Req 4.1 - open Edit Catalog dialog pre-populated
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                if let Some(cat) = self.files_panel.registry.get_by_name(&name) {
                                    let form =
                                        catalog_manager_dialog::EditCatalogForm::from_catalog(cat);
                                    self.files_panel.dialog =
                                        files_panel::FilesDialogState::EditCatalog(form);
                                }
                            }
                        }
                        files_panel::FilesPanelAction::DeleteCatalog(name) => {
                            // Req 4.3 - open Delete Catalog confirmation dialog
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                if let Some(cat) = self.files_panel.registry.get_by_name(&name) {
                                    let confirm =
                                        catalog_manager_dialog::DeleteCatalogConfirm::from_catalog(
                                            cat,
                                        );
                                    self.files_panel.dialog =
                                        files_panel::FilesDialogState::DeleteCatalog(confirm);
                                }
                            }
                        }

                        files_panel::FilesPanelAction::AllocateDataset(catalog_name) => {
                            // Req 5.1 - open Allocate Dataset dialog
                            // Req 13.2 - record which catalog opened the dialog
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                self.files_panel.pending_alloc_catalog = Some(catalog_name.clone());
                                // Req 5.7 — pre-populate Dataset Name with catalog HLQ if set
                                let form = self
                                    .files_panel
                                    .registry
                                    .get_by_name(&catalog_name)
                                    .and_then(|c| c.default_hlq.as_deref())
                                    .map(dataset_alloc_dialog::AllocDatasetForm::with_hlq)
                                    .unwrap_or_default();
                                self.files_panel.dialog =
                                    files_panel::FilesDialogState::AllocateDataset(form);
                            }
                        }
                        files_panel::FilesPanelAction::OpenFile(_) => {}
                        files_panel::FilesPanelAction::NavigateInto(_) => {}
                        files_panel::FilesPanelAction::None => {}
                    }
                }
                TabKind::FileEditor | TabKind::Untitled => {
                    let tab_id = self.tabs.active_tab().id;
                    let tab = self.tabs.active_tab_mut();
                    if let Some(err) = editor_panel::render(
                        ui,
                        tab,
                        &self.runtime,
                        &mut self.cmd_engine,
                        &mut self.exclude_manager,
                        tab_id,
                    ) {
                        self.open_error = Some(err);
                    }
                }
                TabKind::SettingsPanel => {
                    // Validates: Requirement 15.1, 15.2, 15.3
                    crate::settings_panel::render(
                        ui,
                        &mut self.settings_panel,
                        &self.config_handle,
                    );
                }
                TabKind::FileExplorerPanel => {
                    // Validates: Requirement 19.5, 19.6, 19.7, 19.8, 19.9
                    if let Some(path) = file_explorer_panel::render(
                        ui,
                        &mut self.file_explorer_panel,
                        &self.files_panel.registry,
                        &self.files_panel,
                    ) {
                        let mut p = ff_command::CommandParams::new();
                        p.insert("path", path.as_str());
                        let _ = self.dispatch.execute_command("file.open", p);
                    }
                }
            }
        });
    }
}
