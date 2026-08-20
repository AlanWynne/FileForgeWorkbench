//! # eframe::App implementation
//!
//! The `update()` frame loop and `on_exit()` callback for WorkbenchShell.

use eframe::egui;

use crate::catalog_manager_dialog::{self, DeleteChoice, DialogOutcome};
use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};
use crate::dataset_alloc_dialog::{self, validate_for_catalog, AllocOutcome};
use crate::files_panel;
use crate::session_manager::SessionManager;

use super::helpers::*;
use super::FocusStop;
use super::WorkbenchShell;
use crate::primary_option_menu;
use crate::tab_state::TabKind;
use ff_keys::FunctionKey;
use ff_keys::{KeyModifier, ModifiedKey};
use std::path::PathBuf;
use std::sync::Arc;

/// Create a default `"Home"` Native catalog pointing at `home_path` and register
/// it in `registry`, but only when no Native catalogs exist yet.
///
/// Returns `true` when a catalog was added (caller should persist the registry).
///
/// Validates: Requirement 14.1, 14.2, 14.4, 14.5
pub(super) fn ensure_default_home_catalog(
    registry: &mut CatalogRegistry,
    home_path: PathBuf,
) -> bool {
    if !registry.list_by_type(CatalogType::Native).is_empty() {
        return false;
    }
    let catalog = VirtualCatalog {
        name: "Home".to_string(),
        catalog_type: CatalogType::Native,
        path: home_path.to_string_lossy().into_owned(),
        description: Some("Default home directory catalog".to_string()),
        auto_mount: true,
        default_hlq: None,
        mount_point: None,
        read_only: false,
    };
    // register() only fails on duplicate name or invalid name — neither applies here.
    // If a non-Native catalog named "Home" already exists the register silently fails,
    // which is acceptable (the user has a catalog named Home of a different type).
    let _ = registry.register(catalog);
    true
}

impl eframe::App for WorkbenchShell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // One-shot startup
        if !self.started {
            self.started = true;
            let _ = self.runtime.block_on(self.app.startup());

            let cli_files = std::mem::take(&mut self.cli_files);
            if !cli_files.is_empty() {
                // CLI args take precedence over session restore (Req 5 AC 6).
                for path in cli_files {
                    if let Err(e) = self.tabs.open_file(&path, &self.runtime) {
                        self.open_error = Some(e);
                    } else {
                        self.open_error = None;
                    }
                }
            } else if let Some(session) = &self.session {
                // No CLI args — restore previous session tabs (Req 5 AC 1, 2).
                let state = session.load();
                let restored_any = !SessionManager::tab_uris(&state).is_empty();
                // Validates: Requirement 6.2 (view-zoom) — restore global zoom offset.
                if state.global_zoom_offset != 0 {
                    self.zoom = ff_zoom::ZoomState::from_persisted(
                        state.global_zoom_offset,
                        &ff_zoom::ZoomConfig::default(),
                    );
                }
                // Validates: Requirement 12.4 (function-keys-and-history) — restore PFSHOW state.
                self.key_bar_visible = state.key_bar_visible;
                // Validates: Requirement 2.1, 2.2 (virtual-catalog-manager) — restore catalog registry.
                self.files_panel.registry = session.load_catalog_registry();
                // Validates: Requirement 14.1–14.5 — create default Home catalog when none exist.
                let home_path = dirs::home_dir()
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_else(|| PathBuf::from("."));
                if ensure_default_home_catalog(&mut self.files_panel.registry, home_path) {
                    session.save_catalog_registry(&self.files_panel.registry);
                }
                for session_tab in &state.tabs {
                    if let Some(uri) = &session_tab.uri {
                        if let Err(e) = self.tabs.open_file(uri, &self.runtime) {
                            self.open_error = Some(format!("Could not restore: {e}"));
                        }
                    }
                }
                // Validates: Requirement 14.1 — if no files restored, open POM tab
                if !restored_any {
                    // Close the welcome placeholder before inserting POM so POM is
                    // the sole tab at index 0 on a clean first launch.
                    // Validates: Requirement 14.1 — POM is always in first position.
                    self.tabs.close_welcome_tab();
                    self.tabs.insert_pom_tab(&self.runtime);
                }
            } else {
                // No session manager — first launch: open POM tab
                // Validates: Requirement 14.1–14.5 — create default Home catalog when none exist.
                let home_path = dirs::home_dir()
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_else(|| PathBuf::from("."));
                let _ = ensure_default_home_catalog(&mut self.files_panel.registry, home_path);
                // Validates: Requirement 14.1
                self.tabs.close_welcome_tab();
                self.tabs.insert_pom_tab(&self.runtime);
            }

            // Startup focus is handled by command_field_focus_requested = true (set in new()).
        }

        // Check if file.exit handler fired
        if *self.should_close.lock().expect("close lock") {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Process any pending file open (set by file.open handler or menu)
        let path = self.pending_open.lock().expect("pending lock").take();
        if let Some(p) = path {
            if !p.is_empty() {
                if let Err(e) = self.tabs.open_file(&p, &self.runtime) {
                    self.open_error = Some(e);
                } else {
                    self.open_error = None;
                }
            }
        }

        // Ctrl+S — save active tab (suppressed when a modal dialog is open)
        if !self.modal_open && ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            if let Err(e) = self.tabs.save_active_tab(&self.runtime) {
                self.open_error = Some(e);
            } else {
                self.open_error = None;
            }
        }

        // Process deferred tab-bar context menu actions (set previous frame).
        if self.pending_new_pom {
            self.pending_new_pom = false;
            self.tabs.insert_pom_tab(&self.runtime);
        }
        if self.pending_new_file {
            self.pending_new_file = false;
            self.tabs.new_untitled_tab(&self.runtime);
        }
        // Validates: Requirement 1.7 — F3/END in Files Panel returns tab to POM view.
        if self.pending_return_to_pom {
            self.pending_return_to_pom = false;
            let idx = self.tabs.active_index();
            if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                tab.kind = crate::tab_state::TabKind::PrimaryOptionMenu;
                tab.title = "[POM]".to_string();
            }
        }

        // ── Detach pending — Validates: Requirement 18.1, 18.4 ───────────────
        if let Some(idx) = self.detach_pending.take() {
            if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                tab.is_floating = true;
                let vid =
                    egui::ViewportId::from_hash_of(format!("floating_tab_{idx}_{}", tab.title));
                self.floating_tabs.push(super::FloatingTab {
                    viewport_id: vid,
                    tab_index: idx,
                    origin_index: idx,
                });
            }
        }

        // ── Redock pending — Validates: Requirement 18.3 ─────────────────────
        let redock_indices: Vec<usize> = {
            let mut guard = self.redock_pending.lock().expect("redock lock");
            std::mem::take(&mut *guard)
        };
        for origin in redock_indices {
            // Find the super::FloatingTab with this origin_index.
            if let Some(ft_pos) = self
                .floating_tabs
                .iter()
                .position(|ft| ft.origin_index == origin)
            {
                let ft = self.floating_tabs.remove(ft_pos);
                let tab_idx = ft.tab_index;
                if let Some(tab) = self.tabs.tabs_mut().get_mut(tab_idx) {
                    tab.is_floating = false;
                }
                // Restore to origin_index (clamped to current tab count).
                let target = origin.min(self.tabs.len().saturating_sub(1));
                if tab_idx != target && tab_idx < self.tabs.len() {
                    self.tabs.tabs_mut().swap(tab_idx, target);
                }
            }
        }
        if let Ok(active) = self
            .config_handle
            .get_string(ff_config::keys::theme::ACTIVE)
        {
            let desired_mode = ff_theme::mode::VisualMode::from_str_loose(&active);
            if desired_mode
                .map(|m| m != self.palette.mode)
                .unwrap_or(false)
            {
                if let Some(mode) = desired_mode {
                    self.palette = ff_theme::defaults::default_palette_for_mode(mode);
                }
            }
        }

        self.apply_theme(ctx);
        // Validates: Requirement 3.1/3.2 (view-zoom) — Ctrl+Scroll updates global zoom.
        // Single zoom level shared across all tab kinds and contexts.
        {
            let (scroll_delta, ctrl_held) = ctx.input_mut(|i| {
                let raw = i.raw_scroll_delta.y;
                let smooth = i.smooth_scroll_delta.y;
                let ctrl = i.modifiers.ctrl;
                if ctrl {
                    i.raw_scroll_delta = egui::Vec2::ZERO;
                    i.smooth_scroll_delta = egui::Vec2::ZERO;
                }
                let delta = if raw != 0.0 { raw } else { smooth };
                (delta, ctrl)
            });
            if ctrl_held && scroll_delta != 0.0 {
                if scroll_delta > 0.0 {
                    self.zoom.zoom_in();
                } else {
                    self.zoom.zoom_out();
                }
            }
        }
        // Track whether the primary mouse button is held — window drag detection.
        // While the mouse is down we suppress any pixels_per_point change so that
        // WM_DPICHANGED messages fired as the window crosses a monitor boundary do
        // not trigger mid-move resize stuttering.  The change is applied on release.
        let mouse_down = ctx.input(|i| i.pointer.primary_down());
        if mouse_down {
            self.is_dragging = true;
        } else if self.is_dragging {
            // Mouse just released — apply any deferred ppp now.
            self.is_dragging = false;
            if let Some(ppp) = self.pending_ppp.take() {
                self.last_ppp = ppp;
                ctx.set_pixels_per_point(ppp);
            }
        }

        // Apply global zoom only when it has changed — do NOT call set_pixels_per_point
        // every frame, as that fights the OS DPI adjustment during cross-monitor moves
        // and causes the window to flash and stick at monitor boundaries.
        {
            let ppp = (1.0_f32 + self.zoom.offset().value() as f32 * 0.07).clamp(0.3, 4.0);
            if (ppp - self.last_ppp).abs() > f32::EPSILON {
                if self.is_dragging {
                    // Defer until mouse release.
                    self.pending_ppp = Some(ppp);
                } else {
                    self.last_ppp = ppp;
                    ctx.set_pixels_per_point(ppp);
                }
            }
        }
        // ── Tab-order focus cycle — Validates: Requirement 16.2–16.22 ───────────
        // Consume Tab / Shift+Tab before egui processes them so we control focus.
        // Suppressed when a modal dialog is open so Tab navigates inside the dialog.
        self.modal_open = self.key_config_dialog.open
            || self.show_about
            || self.show_history_list.is_some()
            || !matches!(self.files_panel.dialog, files_panel::FilesDialogState::None);
        {
            let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
            let tab_count = self.tabs.len();
            let pom_active = self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu;
            let (tab_pressed, shift_tab_pressed) = ctx.input_mut(|i| {
                if self.modal_open {
                    return (false, false);
                }
                let shift = i.modifiers.shift;
                let tab = i.key_pressed(egui::Key::Tab);
                if tab {
                    // consume so egui doesn't also move focus
                    i.events.retain(|e| {
                        !matches!(
                            e,
                            egui::Event::Key {
                                key: egui::Key::Tab,
                                ..
                            }
                        )
                    });
                }
                (tab && !shift, tab && shift)
            });
            if tab_pressed {
                self.focus_stop = self.focus_stop.next(menu_count, tab_count, pom_active);
                if self.focus_stop == FocusStop::CommandField {
                    self.command_field_focus_requested = true;
                }
            } else if shift_tab_pressed {
                self.focus_stop = self.focus_stop.prev(menu_count, tab_count, pom_active);
                if self.focus_stop == FocusStop::CommandField {
                    self.command_field_focus_requested = true;
                }
            }
            // Validates: Requirement 16.20 — request egui focus on the tab header button
            // when a TabHeader stop is active (one-shot on Tab press).
            if tab_pressed || shift_tab_pressed {
                if let FocusStop::TabHeader { index } = self.focus_stop {
                    let tab_btn_id = egui::Id::new("tab_header_btn").with(index);
                    ctx.memory_mut(|m| m.request_focus(tab_btn_id));
                }
            }
            // Validates: Requirement 16.13–16.16 — Enter/Space activates focused POM stop.
            if pom_active {
                let enter_or_space = ctx
                    .input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
                if enter_or_space {
                    match &self.focus_stop.clone() {
                        FocusStop::PomOption { index } => {
                            let key = primary_option_menu::BUILT_IN_OPTIONS
                                .get(*index)
                                .map(|o| o.key)
                                .unwrap_or("0");
                            self.handle_command(key);
                        }
                        FocusStop::PomExit => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        FocusStop::CalendarPrev => {
                            self.pom_calendar_offset -= 1;
                        }
                        FocusStop::CalendarNext => {
                            self.pom_calendar_offset += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        self.render_menu_bar(ctx);
        self.render_tab_bar(ctx);
        self.render_title_line(ctx);
        self.render_command_field(ctx);
        self.render_key_label_bar(ctx);
        self.render_status_bar(ctx);

        // ── Function key dispatch (Req 3.1, 3.2) ────────────────────────
        // Suppressed when a modal dialog is open so Ctrl/Shift/Alt combos inside
        // dialog text fields are not intercepted by the shell key map.
        let fkey_cmd = if self.modal_open {
            None
        } else {
            ctx.input(|i| {
                let modifier = if i.modifiers.shift {
                    KeyModifier::Shift
                } else if i.modifiers.ctrl {
                    KeyModifier::Ctrl
                } else if i.modifiers.alt {
                    KeyModifier::Alt
                } else {
                    KeyModifier::None
                };
                FunctionKey::ALL.iter().find_map(|&fk| {
                    egui_fkey(fk).and_then(|ek| {
                        if i.key_pressed(ek) {
                            let mk = ModifiedKey { key: fk, modifier };
                            self.key_map_resolver
                                .active_key_map()
                                .get(mk)
                                .or_else(|| {
                                    if modifier != KeyModifier::None {
                                        self.key_map_resolver.active_key_map().get_plain(fk)
                                    } else {
                                        None
                                    }
                                })
                                .map(|b| b.command().to_string())
                        } else {
                            None
                        }
                    })
                })
            })
        };
        if let Some(cmd) = fkey_cmd {
            self.handle_command(&cmd);
        }

        self.render_central_panel(ctx);

        // ── Floating tab viewports — Validates: Requirement 18.1, 18.2, 18.5 ──
        for ft_idx in 0..self.floating_tabs.len() {
            let vid = self.floating_tabs[ft_idx].viewport_id;
            let tab_index = self.floating_tabs[ft_idx].tab_index;
            let origin_index = self.floating_tabs[ft_idx].origin_index;
            let title = self
                .tabs
                .tabs()
                .get(tab_index)
                .map(|t| format!("{} — FileForge Workbench", super::title_line_text(t)))
                .unwrap_or_else(|| "FileForge Workbench".to_string());
            let redock_tx = Arc::clone(&self.redock_pending);
            ctx.show_viewport_deferred(
                vid,
                egui::ViewportBuilder::default().with_title(&title),
                move |ctx, class| {
                    if class == egui::ViewportClass::Deferred {
                        // Detect close — push origin_index into redock_pending.
                        if ctx.input(|i| i.viewport().close_requested()) {
                            redock_tx.lock().expect("redock lock").push(origin_index);
                            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                        }
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.label(format!("Tab {tab_index} — floating"));
                        });
                    }
                },
            );
        }

        // ── Catalog Manager Dialog — Req 3.1–3.8 ──────────────────────────────
        // About dialog - Req 13.1, 13.8
        if self.show_about {
            crate::about_dialog::render(ctx, &mut self.show_about);
        }

        // Key Configuration Dialog -- Validates: Requirement 20.1
        crate::key_config_dialog::render_if_open(
            ctx,
            &mut self.key_config_dialog,
            &self.key_map_resolver,
            &self.config_handle,
        );

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

        // Catalog Manager Dialogs - Req 3.1-3.8, 4.1-4.5
        match &mut self.files_panel.dialog {
            files_panel::FilesDialogState::NewCatalog(ref mut form) => {
                let outcome =
                    catalog_manager_dialog::render(ctx, form, &mut self.files_panel.registry);
                if outcome == DialogOutcome::Confirmed || outcome == DialogOutcome::Cancelled {
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::EditCatalog(ref mut form) => {
                let outcome =
                    catalog_manager_dialog::render_edit(ctx, form, &mut self.files_panel.registry);
                if outcome == DialogOutcome::Confirmed || outcome == DialogOutcome::Cancelled {
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::DeleteCatalog(ref confirm) => {
                let choice = catalog_manager_dialog::render_delete(ctx, confirm);
                if choice != DeleteChoice::Cancel {
                    let confirm_clone = confirm.clone();
                    if let Err(e) = catalog_manager_dialog::execute_delete(
                        &choice,
                        &confirm_clone,
                        &mut self.files_panel.registry,
                    ) {
                        self.open_error = Some(e);
                    } else {
                        // Req 13.5 — remove datasets for the deleted catalog
                        self.files_panel
                            .remove_catalog_datasets(&confirm_clone.name);
                    }
                }
                self.files_panel.dialog = files_panel::FilesDialogState::None;
            }
            files_panel::FilesDialogState::AllocateDataset(ref mut form) => {
                let outcome = dataset_alloc_dialog::render(ctx, form);
                if outcome == AllocOutcome::Confirmed {
                    // Req 13.2 — persist the allocated dataset into the UI-layer store
                    // Req 13.2, 5.9 — validate with duplicate check then persist
                    let existing: Vec<String> = self
                        .files_panel
                        .pending_alloc_catalog
                        .as_deref()
                        .and_then(|cat| self.files_panel.datasets.get(cat))
                        .map(|ds| ds.iter().map(|d| d.name.clone()).collect())
                        .unwrap_or_default();
                    match validate_for_catalog(form, &existing) {
                        Ok(params) => {
                            if let Some(cat) = self.files_panel.pending_alloc_catalog.take() {
                                self.files_panel.add_dataset(&cat, params);
                            }
                        }
                        Err(e) => {
                            form.error = Some(e);
                            return;
                        }
                    }
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                } else if outcome == AllocOutcome::Cancelled {
                    self.files_panel.pending_alloc_catalog = None;
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::None => {}
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(session) = &self.session {
            session.save(&self.tabs, self.zoom.offset().value(), self.key_bar_visible);
            session.save_catalog_registry(&self.files_panel.registry);
        }
        self.runtime.block_on(self.app.shutdown());
    }
}

#[cfg(test)]
mod startup_tests {
    use super::ensure_default_home_catalog;
    use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};
    use std::path::PathBuf;

    fn home() -> PathBuf {
        PathBuf::from("C:/Users/testuser")
    }

    fn native_catalog(name: &str) -> VirtualCatalog {
        VirtualCatalog {
            name: name.to_string(),
            catalog_type: CatalogType::Native,
            path: "C:/some/path".to_string(),
            description: None,
            auto_mount: true,
            default_hlq: None,
            mount_point: None,
            read_only: false,
        }
    }

    /// Validates: Requirement 14.1, 14.2 — empty registry gets a "Home" Native catalog.
    #[test]
    fn no_native_catalogs_triggers_home_catalog_creation() {
        // Validates: Requirement 14.1, 14.2
        let mut registry = CatalogRegistry::new();
        let added = ensure_default_home_catalog(&mut registry, home());
        assert!(added, "must return true when catalog was added");
        let cat = registry
            .get_by_name("Home")
            .expect("Home catalog must exist");
        assert_eq!(cat.catalog_type, CatalogType::Native);
        assert_eq!(cat.path, "C:/Users/testuser");
        assert!(cat.auto_mount);
    }

    /// Validates: Requirement 14.4 — existing Native catalog suppresses Home creation.
    #[test]
    fn existing_native_catalog_suppresses_home_creation() {
        // Validates: Requirement 14.4
        let mut registry = CatalogRegistry::new();
        registry.register(native_catalog("Projects")).unwrap();
        let added = ensure_default_home_catalog(&mut registry, home());
        assert!(
            !added,
            "must return false when Native catalog already exists"
        );
        assert!(
            registry.get_by_name("Home").is_none(),
            "Home must not be created when a Native catalog already exists"
        );
    }

    /// Validates: Requirement 14.3 — returned true signals caller to persist registry.
    /// Validates: Requirement 14.5 — fallback path is used when home_path is provided.
    #[test]
    fn home_catalog_uses_provided_path() {
        // Validates: Requirement 14.3, 14.5
        let fallback = PathBuf::from("C:/fallback");
        let mut registry = CatalogRegistry::new();
        let added = ensure_default_home_catalog(&mut registry, fallback.clone());
        assert!(added);
        let cat = registry.get_by_name("Home").unwrap();
        assert_eq!(cat.path, "C:/fallback");
    }
}
