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
                    self.command_text.clear();
                    self.handle_command(&cmd);
                    // Return focus to the command field after every command execution.
                    self.focus_stop = FocusStop::CommandField;
                    self.command_field_focus_requested = true;
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
                // A clicked key-label bar slot is a shortcut binding; route it
                // through Target_Resolution so a user-defined command id runs
                // its target (command-configurator Requirement 4.4), falling
                // through to the pipeline otherwise (Requirement 10.2).
                self.dispatch_bound_command(&cmd);
            }
        }
    }

    // ── Status bar ───────────────────────────────────────────────────────

    pub(super) fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let phase_label = match self.app.phase() {
                    LifecyclePhase::Running => "RUNNING",
                    LifecyclePhase::Initializing => "STARTING",
                    LifecyclePhase::ShuttingDown => "SHUTTING DOWN",
                    LifecyclePhase::Terminated => "TERMINATED",
                };
                // Validates: Requirement 14.3 -- selectable status bar text
                ui.add(egui::SelectableLabel::new(false, phase_label));
                ui.separator();

                // Validates: Requirement 20.1 -- session start timestamp
                ui.add(egui::SelectableLabel::new(
                    false,
                    self.format_session_start(),
                ));
                ui.separator();

                let tab = self.tabs.active_tab();
                let line = tab.cursor.cursor_line();
                let col = tab.cursor.cursor_column();
                // Requirement 7.1: format "Ln {line}, Col {col}" (1-based)
                ui.add(egui::SelectableLabel::new(
                    false,
                    format!("Ln {line}, Col {col}"),
                ));
                ui.separator();
                // Requirement 7.3: real encoding from document
                ui.add(egui::SelectableLabel::new(false, tab.encoding_label()));
                ui.separator();
                // Requirement 7.1 (view-zoom) -- zoom indicator when non-zero
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
                ui.add(egui::SelectableLabel::new(
                    false,
                    format!("{} lines", tab.line_count),
                ));
                ui.separator();
                // Requirement 6.5: modified indicator
                if tab.is_modified {
                    ui.colored_label(to_egui_color(self.palette.editor.accent), "\u{25cf}");
                    ui.separator();
                }
                // Req 16.3: CAPS mode indicator
                if tab.edit_profile.caps.is_on() {
                    ui.colored_label(to_egui_color(self.palette.editor.accent), "CAPS");
                    ui.separator();
                }

                if let Some(err) = &self.open_error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.separator();
                    // Validates: Requirement 2.5 -- register error message for automation.
                    self.automation.register_str(
                        crate::automation::ids::STATUSBAR_MESSAGE,
                        crate::automation::ControlState::with_value(err.as_str()),
                    );
                } else {
                    self.automation.register_str(
                        crate::automation::ids::STATUSBAR_MESSAGE,
                        crate::automation::ControlState::with_value(""),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Validates: Requirement 14.3 -- selectable version label
                    ui.add(egui::SelectableLabel::new(
                        false,
                        "FileForge Workbench v0.1.0",
                    ));
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
        // ── File Explorer side panel (ctx-level, resizable) ────────────
        // Must be shown before CentralPanel so egui allocates space correctly.
        // Validates: Requirement 1.3 file-tree-panel, fix B019
        // Validates: Requirement 20.1 -- Tab from CommandField in FilesPanel transfers focus
        // to the first catalog node in the catalog tree.
        // ── Tab from FilesPanel command field → first catalog node ──────
        // Validates: Requirement 20.1
        // The Files Panel has its own internal "Command ===>" TextEdit.
        // When that field has egui focus and the user presses plain Tab,
        // we consume the Tab event and request focus on the first catalog node.
        // We cannot use focus_stop here because focus_stop tracks the shell's
        // top-level command field, not the panel-internal one.
        let is_files_panel = self.tabs.active_tab().kind == TabKind::FilesPanel;
        if is_files_panel && !self.modal_open {
            let files_cmd_id = egui::Id::new("files_panel_cmd");
            let files_cmd_focused = ctx.memory(|m| m.focused() == Some(files_cmd_id));
            if files_cmd_focused {
                let tab_pressed = ctx.input_mut(|i| {
                    if i.key_pressed(egui::Key::Tab) && !i.modifiers.shift {
                        i.events.retain(|e| {
                            !matches!(
                                e,
                                egui::Event::Key {
                                    key: egui::Key::Tab,
                                    ..
                                }
                            )
                        });
                        return true;
                    }
                    false
                });
                if tab_pressed {
                    self.files_panel.tree_focus_requested = true;
                }
            }
        }

        let is_file_explorer = self.tabs.active_tab().kind == TabKind::FileExplorerPanel;
        // CR-NR-060 Slice A (parallel-build): render the NavModel-backed explorer
        // preview beneath the legacy panel when the preview gate is on. Default
        // off, so the shipping explorer is unaffected until the swap.
        if is_file_explorer && self.nav_explorer_preview {
            self.render_nav_explorer_preview(ctx);
        }
        if is_file_explorer {
            let open_path = egui::CentralPanel::default().show(ctx, |ui| {
                // Req 20.2–20.12 — keyboard handling when explorer has focus.
                if self.file_explorer_panel.explorer_focused {
                    let visible = file_explorer_panel::collect_visible_node_paths_with_dirs(
                        &self.files_panel.registry,
                        &self.files_panel,
                        &self.file_explorer_panel.open_catalogs,
                        &self.file_explorer_panel.open_directories,
                    );
                    file_explorer_panel::handle_explorer_keyboard(
                        ui,
                        &mut self.file_explorer_panel,
                        &visible,
                    );
                    if ui.input(|i| i.pointer.any_click())
                        && !ui.rect_contains_pointer(ui.max_rect())
                    {
                        self.file_explorer_panel.explorer_focused = false;
                    }
                }

                file_explorer_panel::render(
                    ui,
                    &mut self.file_explorer_panel,
                    &self.files_panel.registry,
                    &self.files_panel,
                    self.active_workspace
                        .as_ref()
                        .map(|ws| {
                            let root_names: Vec<String> = ws
                                .roots
                                .iter()
                                .filter_map(|r| r.file_name())
                                .map(|n| n.to_string_lossy().into_owned())
                                .collect();
                            (ws.name.as_str(), root_names)
                        })
                        .as_ref()
                        .map(|(name, names)| (*name, names.as_slice())),
                )
            });
            // Persist sidebar width from the state (updated inside render())
            self.file_explorer_panel_width =
                self.file_explorer_panel.sidebar_width.clamp(120.0, 600.0);
            if let Some(dsn) = open_path.inner {
                // Req 16 — for Mainframe datasets, resolve physical path via SQLite.
                // Search all Mainframe catalogs for the DSN.
                let mainframe_catalog = self
                    .files_panel
                    .registry
                    .list_by_type(crate::catalog_registry::CatalogType::Mainframe)
                    .into_iter()
                    .find(|c| {
                        if let Ok(parsed) = ff_dscatalog::dsn::Dsn::parse(&dsn) {
                            self.files_panel
                                .registry
                                .resolve_dsn(&c.name, &parsed)
                                .is_ok()
                        } else {
                            false
                        }
                    })
                    .map(|c| c.name.clone());
                match mainframe_catalog {
                    Some(catalog_name) => {
                        match open_mainframe_dsn(&self.files_panel.registry, &catalog_name, &dsn) {
                            Err(e) => self.open_error = Some(e),
                            Ok(path_str) => {
                                let mut p = ff_command::CommandParams::new();
                                p.insert("path", path_str.as_str());
                                let _ = self.dispatch.execute_command("file.open", p);
                            }
                        }
                    }
                    None => {
                        // Native path or unknown — dispatch directly
                        let mut p = ff_command::CommandParams::new();
                        p.insert("path", dsn.as_str());
                        let _ = self.dispatch.execute_command("file.open", p);
                    }
                }
            }
            if let Some(err) = self.file_explorer_panel.last_error.take() {
                self.open_error = Some(err);
            }

            // Req 21.6 — paste-into-editor prompt when Ctrl+V pressed while
            // file_copy_clipboard is non-empty. Write the file list to the OS
            // clipboard as plain text so the user can paste it into any editor tab.
            // Req 21.7 — one path per line.
            if self.file_explorer_panel.paste_prompt_open {
                self.file_explorer_panel.paste_prompt_open = false;
                if let Some(ref cb) = self.file_explorer_panel.file_copy_clipboard.clone() {
                    let text = cb.paths.join("\n");
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&text);
                    }
                    self.open_error = Some(format!(
                        "{} file path(s) copied to clipboard — press Ctrl+V in editor to paste",
                        cb.paths.len()
                    ));
                }
            }
        }

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
        if !is_file_explorer {
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
                                primary_option_menu::CalendarNav::Prev => {
                                    self.pom_calendar_offset -= 1
                                }
                                primary_option_menu::CalendarNav::Next => {
                                    self.pom_calendar_offset += 1
                                }
                            }
                        }
                        if let Some(pom_action) = pom_result.action {
                            match pom_action {
                                primary_option_menu::PomAction::Navigate(key) => {
                                    self.handle_command(&key.to_string());
                                }
                                // Validates: Requirement 6.2, 6.3, 6.4 (cv-requirements.md)
                                // Non-numeric option keys (S, B) route through the
                                // same command handler as typed command-field input.
                                primary_option_menu::PomAction::NavigateKey(key) => {
                                    self.handle_command(&key);
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
                                        .get_string(
                                            ff_config::keys::catalogs::DEFAULT_MAINFRAME_ROOT,
                                        )
                                        .unwrap_or_default();
                                    let posix_root = self
                                        .config_handle
                                        .get_string(ff_config::keys::catalogs::DEFAULT_POSIX_ROOT)
                                        .unwrap_or_default();
                                    self.files_panel.dialog =
                                        files_panel::FilesDialogState::NewCatalog(
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
                                    if let Some(cat) = self.files_panel.registry.get_by_name(&name)
                                    {
                                        let form =
                                            catalog_manager_dialog::EditCatalogForm::from_catalog(
                                                cat,
                                            );
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
                                    if let Some(cat) = self.files_panel.registry.get_by_name(&name)
                                    {
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
                                    self.files_panel.pending_alloc_catalog =
                                        Some(catalog_name.clone());
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
                            files_panel::FilesPanelAction::OpenFile(dsn) => {
                                // Req 16 — resolve physical path from catalog repository + DSN
                                let is_mainframe = self
                                    .files_panel
                                    .content
                                    .selected_catalog
                                    .as_deref()
                                    .and_then(|n| self.files_panel.registry.get_by_name(n))
                                    .map(|c| {
                                        c.catalog_type
                                            == crate::catalog_registry::CatalogType::Mainframe
                                    })
                                    .unwrap_or(false);
                                if is_mainframe {
                                    let catalog_name = self
                                        .files_panel
                                        .content
                                        .selected_catalog
                                        .clone()
                                        .unwrap_or_default();
                                    match open_mainframe_dsn(
                                        &self.files_panel.registry,
                                        &catalog_name,
                                        &dsn,
                                    ) {
                                        Err(e) => self.open_error = Some(e),
                                        Ok(path_str) => {
                                            let mut p = ff_command::CommandParams::new();
                                            p.insert("path", path_str.as_str());
                                            let _ = self.dispatch.execute_command("file.open", p);
                                        }
                                    }
                                } else {
                                    let mut p = ff_command::CommandParams::new();
                                    p.insert("path", dsn.as_str());
                                    let _ = self.dispatch.execute_command("file.open", p);
                                }
                            }
                            files_panel::FilesPanelAction::NavigateInto(_) => {}
                            files_panel::FilesPanelAction::None => {}
                        }
                    }
                    TabKind::FileEditor | TabKind::Untitled => {
                        let tab_id = self.tabs.active_tab().id;
                        let scroll_amount = self.scroll_amount.clone();
                        let tab = self.tabs.active_tab_mut();
                        if let Some(err) = editor_panel::render(
                            ui,
                            tab,
                            &self.runtime,
                            &mut self.cmd_engine,
                            &mut self.exclude_manager,
                            tab_id,
                            &scroll_amount,
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
                    TabKind::PluginManager => {
                        // Validates: plugin-manager-ui Requirement 1.1-1.6
                        crate::plugin_manager_panel::render(ui, &mut self.plugin_manager_panel);
                    }
                    TabKind::EventLog => {
                        // Validates: notification-system Requirement 2.1-2.6
                        crate::event_log_panel::render(
                            ui,
                            &mut self.event_log_panel,
                            &self.notification_queue,
                        );
                        if self.event_log_panel.clear_requested {
                            self.event_log_panel.clear_requested = false;
                            self.notification_queue.lock().expect("queue").clear();
                        }
                    }
                    TabKind::SearchResults => {
                        // Validates: global-search Requirement 1.1, 4.1
                        let roots = collect_search_roots(
                            &self.files_panel.registry,
                            self.active_workspace.as_ref(),
                        );
                        let outcome = crate::search_results_panel::render(
                            ui,
                            &mut self.search_results_panel,
                            &roots,
                            &self.runtime,
                        );
                        match outcome {
                            crate::search_results_panel::SearchPanelOutcome::OpenMatch {
                                path,
                                line,
                            } => {
                                if let Err(e) = self.tabs.open_file(&path, &self.runtime) {
                                    self.open_error = Some(e);
                                } else {
                                    // Scroll to the matching line.
                                    let idx = self.tabs.active_index();
                                    if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                                        tab.viewport.scroll_to_line(
                                            line.saturating_sub(1).max(1),
                                            &tab.cursor.clone(),
                                        );
                                    }
                                }
                            }
                            crate::search_results_panel::SearchPanelOutcome::ReplaceAll => {
                                let unsaved: Vec<String> = self
                                    .tabs
                                    .tabs()
                                    .iter()
                                    .filter(|t| t.is_modified)
                                    .filter_map(|t| t.path.clone())
                                    .collect();
                                let req = self.search_results_panel.build_request(roots).ok();
                                if let Some(r) = req {
                                    let results = self.search_results_panel.results.clone();
                                    match ff_global_search::GlobalReplaceEngine::replace_all(
                                        &results,
                                        &r,
                                        &self.search_results_panel.replace_text.clone(),
                                        &unsaved,
                                    ) {
                                        Ok((summary, _conflicts)) => {
                                            self.open_error = Some(format!(
                                                "Replaced {} occurrence(s) in {} file(s)",
                                                summary.replacements, summary.files_modified
                                            ));
                                        }
                                        Err(e) => {
                                            self.open_error = Some(format!("Replace failed: {e}"));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    TabKind::FileExplorerPanel => {
                        // Rendered above in the is_file_explorer block -- unreachable here
                    }
                    TabKind::MacroLibrary => {
                        // Validates: lua-macro-engine Requirement 12.1-12.8
                        let action =
                            crate::macro_library_panel::render(ui, &mut self.macro_library_panel);
                        match action {
                            crate::macro_library_panel::MacroLibraryAction::Edit(path) => {
                                let mut p = ff_command::CommandParams::new();
                                p.insert("path", path.as_str());
                                let _ = self.dispatch.execute_command("file.open", p);
                            }
                            crate::macro_library_panel::MacroLibraryAction::Run(_path) => {
                                self.open_error =
                                    Some("Lua execution not yet available".to_string());
                            }
                            crate::macro_library_panel::MacroLibraryAction::Delete(path) => {
                                if let Err(e) = std::fs::remove_file(&path) {
                                    self.open_error = Some(format!("Delete failed: {e}"));
                                } else {
                                    let dirs = self.macro_dirs();
                                    self.macro_library_panel.refresh(&dirs);
                                    self.open_error = None;
                                }
                            }
                            crate::macro_library_panel::MacroLibraryAction::None => {}
                        }
                    }
                    TabKind::MenuWorkspace => {
                        // Validates: menu-workspace Requirement 2.1-2.6
                        let active_idx = self.tabs.active_index();
                        if let Some(mw) = self
                            .tabs
                            .tabs_mut()
                            .get_mut(active_idx)
                            .and_then(|t| t.menu_workspace.as_mut())
                        {
                            mw.poll_reload();
                            if let Some(option) =
                                crate::menu_workspace::render::render_menu_workspace(mw, ui)
                            {
                                self.pending_menu_option = Some(option);
                            }
                        }
                    }
                    TabKind::CommandConfigurator => {
                        // Validates: command-configurator Requirement 2.2-2.6
                        self.command_store.poll_reload();
                        let action = crate::command_config::render::render(
                            ui,
                            &mut self.command_configurator_panel,
                            &self.command_store,
                        );
                        self.apply_configurator_action(action);
                    }
                }
            });
        } // end !is_file_explorer
    }

    /// Render the NavModel-backed File Explorer preview (CR-NR-060 Slice A,
    /// parallel-build). Seeds the Local Files root from the local/POSIX provider
    /// on first display, renders the modern tree, and applies the returned
    /// effects (expand -> async VFS list; collapse; open -> file.open dispatch or
    /// OS default app). Gated behind `nav_explorer_preview`; removed at the swap.
    ///
    /// Validates: Requirement 24.1, 24.3, 24.5, 24.9
    fn render_nav_explorer_preview(&mut self, ctx: &egui::Context) {
        use crate::explorer_view::{
            keyboard_effects, render_tree, resolve_open, ExplorerEffect, OpenTarget,
        };
        use crate::nav_model::list_via_provider;

        // Seed the Local Files root on first display: associate its URI and load
        // its immediate children via the provider (never std::fs).
        let local_root = self.nav_model.tree.root_categories[0];
        let needs_seed = self
            .nav_model
            .tree
            .get_node(local_root)
            .map(|n| !n.children_loaded)
            .unwrap_or(false);
        if needs_seed {
            let root_dir = dirs::home_dir()
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            // Enter the Tokio runtime context so the provider's file-watcher
            // (which calls `tokio::spawn`) can start; constructing the provider
            // outside a runtime context panics ("no reactor running") -- B040.
            let provider = {
                let _rt_guard = self.runtime.enter();
                crate::posix_provider::PosixProvider::new(root_dir, true)
            };
            if let Ok(provider) = provider {
                let root_uri = ff_vfs::ResourceUri::new("posix", "/");
                self.nav_model.set_uri(local_root, root_uri.clone());
                match list_via_provider(&self.runtime, &provider, root_uri.path()) {
                    Ok(entries) => self.nav_model.apply_listing(local_root, "posix", &entries),
                    Err(e) => self.nav_model.apply_load_error(local_root, e),
                }
            }

            // Seed the Catalogs root generically: one CatalogRoot node per
            // registered catalog (Slice A -- no mainframe qualifier/dataset
            // duality; that is Slice B). Requirement 24.8.
            let catalogs_root = self.nav_model.tree.root_categories[1];
            for cat in self.files_panel.registry.list() {
                let uri = ff_vfs::ResourceUri::new("catalog", format!("/{}", cat.name));
                self.nav_model.add_child(
                    catalogs_root,
                    cat.name.clone(),
                    ff_file_tree::NodeType::CatalogRoot,
                    uri,
                );
            }
            if let Some(n) = self.nav_model.tree.get_node_mut(catalogs_root) {
                n.children_loaded = true;
            }
        }

        let mut effects = Vec::new();
        egui::TopBottomPanel::bottom("nav_explorer_preview")
            .resizable(true)
            .default_height(240.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("File Explorer (preview -- ff-file-tree)")
                        .monospace()
                        .strong(),
                );
                // Keyboard navigation (Req 8/20) when the preview panel is
                // hovered/focused, then mouse interactions from the tree.
                if ui.rect_contains_pointer(ui.max_rect()) {
                    effects.extend(keyboard_effects(
                        ui,
                        &self.nav_model,
                        &mut self.nav_selection,
                    ));
                }
                effects.extend(render_tree(
                    ui,
                    &self.nav_model,
                    &mut self.nav_selection,
                    &self.palette,
                ));
            });

        // Apply interaction effects outside the render borrow.
        for eff in effects {
            match eff {
                ExplorerEffect::Expand(id) => {
                    if let Some(uri) = self.nav_model.uri_of(id).cloned() {
                        match uri.scheme() {
                            // Catalogs root child: resolve the catalog by name and
                            // list its backing directory generically (Req 24.8 --
                            // no mainframe qualifier/dataset duality; that is
                            // Slice B). URI is vfs://catalog/{name}.
                            "catalog" => self.expand_catalog_node(id, &uri),
                            // Local Files (posix/local): root-jailed provider at the
                            // local root; URI path is provider-relative (Req 24.3).
                            _ => self.expand_local_node(id, &uri),
                        }
                    } else {
                        self.nav_model.tree.toggle_expand(id);
                    }
                }
                ExplorerEffect::Collapse(id) => self.nav_model.tree.toggle_expand(id),
                ExplorerEffect::Open(id) => match resolve_open(&self.nav_model, id) {
                    OpenTarget::Editor(uri) => {
                        let mut p = ff_command::CommandParams::new();
                        p.insert("path", uri.path());
                        let _ = self.dispatch.execute_command("file.open", p);
                    }
                    OpenTarget::External(uri) => {
                        crate::context_menu::launch_default_app(uri.path());
                    }
                    OpenTarget::Dataset { catalog, dsn } => {
                        // Resolve the DSN to its physical file (create if
                        // missing) and open it in the editor -- same path the
                        // legacy panel uses. Req 16.1/16.3.
                        match open_mainframe_dsn(&self.files_panel.registry, &catalog, &dsn) {
                            Ok(path_str) => {
                                let mut p = ff_command::CommandParams::new();
                                p.insert("path", path_str.as_str());
                                let _ = self.dispatch.execute_command("file.open", p);
                            }
                            Err(e) => self.open_error = Some(e),
                        }
                    }
                    OpenTarget::None => {}
                },
                ExplorerEffect::CopyPath(id) => {
                    // Req 16 Copy Full Path: copy the node's resource path.
                    if let Some(uri) = self.nav_model.uri_of(id) {
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_text(uri.path());
                        }
                    }
                }
                ExplorerEffect::Reveal(id) => {
                    // Req 16 Reveal in Explorer: open the OS file manager at the
                    // node. Only local/POSIX nodes map to a real host path.
                    if let Some(uri) = self.nav_model.uri_of(id).cloned() {
                        if uri.scheme() == "posix" || uri.scheme() == "local" {
                            let root_dir = dirs::home_dir()
                                .or_else(|| std::env::current_dir().ok())
                                .unwrap_or_else(|| std::path::PathBuf::from("."));
                            let rel = uri.path().trim_start_matches('/');
                            let full = root_dir.join(rel);
                            crate::file_explorer_panel::reveal_in_explorer(&full.to_string_lossy());
                        }
                    }
                }
                ExplorerEffect::None => {}
            }
        }
    }

    /// Expand a Local Files (posix/local) node: list its provider-relative path
    /// through a root-jailed provider rooted at the local root.
    ///
    /// Validates: Requirement 24.3, 24.5
    fn expand_local_node(&mut self, id: ff_file_tree::NodeId, uri: &ff_vfs::ResourceUri) {
        use crate::nav_model::list_via_provider;
        let root_dir = dirs::home_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        // Enter the runtime context so the provider's watcher can spawn
        // (constructing outside a runtime panics -- B040).
        let provider = {
            let _rt_guard = self.runtime.enter();
            crate::posix_provider::PosixProvider::new(root_dir, true)
        };
        if let Ok(provider) = provider {
            match list_via_provider(&self.runtime, &provider, uri.path()) {
                Ok(entries) => self.nav_model.apply_listing(id, "posix", &entries),
                Err(e) => self.nav_model.apply_load_error(id, e),
            }
        }
    }

    /// Expand a Catalogs root child generically (Req 24.8): resolve the catalog
    /// by name (URI `vfs://catalog/{name}`) and list its backing directory
    /// through a provider rooted at the catalog `path`. No mainframe qualifier/
    /// dataset duality is applied here -- that is Slice B. If the catalog is
    /// unknown or its directory cannot be listed, an error node is shown.
    ///
    /// Validates: Requirement 24.8, 24.5
    fn expand_catalog_node(&mut self, id: ff_file_tree::NodeId, uri: &ff_vfs::ResourceUri) {
        use crate::catalog_registry::CatalogType;
        use crate::nav_model::split_catalog_uri_path;
        // URI path is "/{name}[/{subpath...}]": the first segment is the catalog
        // name, the remainder is a directory path relative to the catalog root.
        let (name, sub_path) = split_catalog_uri_path(uri.path());
        let resolved = self.files_panel.registry.get_by_name(name).map(|c| {
            (
                c.catalog_type,
                std::path::PathBuf::from(&c.path),
                c.read_only,
            )
        });
        let (catalog_type, root_dir, read_only) = match resolved {
            Some(v) => v,
            None => {
                self.nav_model
                    .apply_load_error(id, format!("Catalog '{name}' not found"));
                return;
            }
        };
        match catalog_type {
            // Mainframe catalog root: list the datasets registered in SQLite,
            // not the repository's internal directory layout. Members/qualifier
            // duality is Slice B; a dataset node is a leaf (or shows a Slice B
            // placeholder if expanded).
            CatalogType::Mainframe if sub_path == "/" => self.list_catalog_datasets(id, name),
            CatalogType::Mainframe => {
                // A dataset node under a Mainframe catalog was expanded: member
                // navigation is deferred to Slice B.
                self.nav_model.apply_load_error(
                    id,
                    "Dataset member browsing is available in a later update.".to_string(),
                );
            }
            // POSIX / Native catalog: a real host directory -- list it generically.
            CatalogType::Posix | CatalogType::Native => {
                self.list_catalog_directory(id, root_dir, read_only, &sub_path)
            }
        }
    }

    /// List the datasets of a Mainframe catalog (from SQLite) as tree nodes.
    /// PS -> sequential (leaf), PO -> partitioned, GDG -> GDG base. Dataset node
    /// URIs use the `dataset` scheme (`vfs://dataset/{catalog}/{DSN}`) so open/
    /// expand routing can distinguish them from directories.
    ///
    /// Validates: Requirement 24.8
    fn list_catalog_datasets(&mut self, id: ff_file_tree::NodeId, catalog: &str) {
        use crate::catalog_registry::dataset_node;
        match self.files_panel.registry.list_datasets(catalog) {
            Ok(records) => {
                let children = records
                    .iter()
                    .map(|r| dataset_node(catalog, r))
                    .collect::<Vec<_>>();
                self.nav_model.apply_child_data(id, children);
            }
            Err(e) => self.nav_model.apply_load_error(id, e.to_string()),
        }
    }

    /// List a POSIX/Native catalog's backing directory generically (Req 24.8).
    ///
    /// Validates: Requirement 24.8, 24.5
    fn list_catalog_directory(
        &mut self,
        id: ff_file_tree::NodeId,
        root_dir: std::path::PathBuf,
        read_only: bool,
        sub_path: &str,
    ) {
        use crate::nav_model::list_via_provider;
        // Enter the runtime context so the provider's watcher can spawn (B040).
        let provider = {
            let _rt_guard = self.runtime.enter();
            crate::posix_provider::PosixProvider::new(root_dir, read_only)
        };
        match provider {
            Ok(provider) => match list_via_provider(&self.runtime, &provider, sub_path) {
                Ok(entries) => self.nav_model.apply_listing(id, "posix", &entries),
                Err(e) => self.nav_model.apply_load_error(id, e),
            },
            Err(e) => self.nav_model.apply_load_error(id, e.to_string()),
        }
    }
}

// === Helpers ================================================================

/// Draw a 2 px focus ring around `rect` using the theme's `focus_ring` colour.
///
/// Call this once per frame for the currently focused `FocusStop` element.
/// Validates: accessibility Requirement 3.1, 3.3, 3.4
pub(crate) fn render_focus_indicator(
    ui: &egui::Ui,
    rect: egui::Rect,
    palette: &ff_theme::ThemePalette,
) {
    use ff_theme::ColourToken;
    let colour = to_egui_color(palette.colour(ColourToken::UiFocusRing));
    ui.painter()
        .rect_stroke(rect.expand(2.0), 2.0, egui::Stroke::new(2.0_f32, colour));
}

/// Collect search root paths from the active workspace or mounted Native catalogs.
///
/// Validates: global-search Requirement 2.4
fn collect_search_roots(
    registry: &crate::catalog_registry::CatalogRegistry,
    workspace: Option<&ff_session::WorkspaceState>,
) -> Vec<String> {
    if let Some(ws) = workspace {
        if !ws.roots.is_empty() {
            return ws
                .roots
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
        }
    }
    registry
        .list_by_type(crate::catalog_registry::CatalogType::Native)
        .iter()
        .map(|c| c.path.clone())
        .collect()
}

/// Resolve a Mainframe DSN to a physical path via the ff-desktop CatalogRegistry,
/// creating the file on disk if it does not yet exist.
///
/// Returns the path as a `String` on success, or a human-readable error.
///
/// Validates: Requirement 16.1, 16.3, 16.4
fn open_mainframe_dsn(
    registry: &crate::catalog_registry::CatalogRegistry,
    catalog_name: &str,
    dsn: &str,
) -> Result<String, String> {
    let parsed = ff_dscatalog::dsn::Dsn::parse(dsn)
        .map_err(|_| format!("'{}': invalid dataset name", dsn))?;
    let path = registry
        .resolve_dsn(catalog_name, &parsed)
        .map_err(|_| format!("'{}': dataset not found in catalog '{}'", dsn, catalog_name))?;
    if !path.exists() {
        crate::files_panel::FilesPanelState::create_dataset_file(&path).map_err(|e| {
            format!(
                "'{}': cannot create dataset file at {}: {}",
                dsn,
                path.display(),
                e
            )
        })?;
    }
    Ok(path.to_string_lossy().into_owned())
}
