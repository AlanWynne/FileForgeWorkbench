//! # eframe::App implementation
//!
//! The `update()` frame loop and `on_exit()` callback for WorkbenchShell.

use eframe::egui;

use crate::catalog_manager_dialog::{self, DeleteChoice, DialogOutcome};
use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};
use crate::command_palette::render::{render_command_palette, PaletteOutcome};
use crate::command_palette::state::PaletteEntry;
use crate::dataset_alloc_dialog::{self, validate_for_catalog, AllocOutcome, Dsorg, Recfm};
use crate::files_panel;
use crate::session_manager::SessionManager;
use ff_dscatalog::{
    dataset::{
        AllocParams as DsAllocParams, Dsorg as DsDsorg, PartitionedSubtype, Recfm as DsRecfm,
    },
    dsn::Dsn,
    hierarchy::CatalogScope,
};

use super::helpers::*;
use super::FocusStop;
use super::WorkbenchShell;
use crate::primary_option_menu;
use crate::tab_state::TabKind;
use ff_fftest::AutomationRegistry as _;
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

/// Ensure a `PrimaryOptionMenu` tab is present at index 0.
///
/// If no tab of kind `PrimaryOptionMenu` exists, inserts one at index 0.
/// Called after session restore so the POM is always reachable on startup.
///
/// Validates: Requirement 14.1b
pub(super) fn ensure_pom_tab_present(
    tabs: &mut crate::tab_manager::TabManager,
    runtime: &tokio::runtime::Runtime,
) {
    let has_pom = tabs
        .tabs()
        .iter()
        .any(|t| t.kind == crate::tab_state::TabKind::PrimaryOptionMenu);
    if !has_pom {
        tabs.insert_pom_tab(runtime);
    }
}

/// Auto-open a catalog or directory node when Tab lands on it.
///
/// For `cat:NAME` nodes: inserts into `open_catalogs` and opens the egui
/// `CollapsingState` so files become visible immediately.
/// For directory paths: inserts into `open_directories` and opens the egui
/// `CollapsingState` keyed by `fep_dir_<path>`.
///
/// Validates: Requirement 20.2
fn auto_open_node(
    ctx: &egui::Context,
    node: &str,
    panel: &mut crate::file_explorer_panel::FileExplorerPanelState,
) {
    if let Some(name) = node.strip_prefix("cat:") {
        panel.open_catalogs.insert(name.to_string());
        let id = egui::Id::new(format!("fep_cat_{name}"));
        let mut cs =
            egui::collapsing_header::CollapsingState::load_with_default_open(ctx, id, false);
        cs.set_open(true);
        cs.store(ctx);
    } else if std::path::Path::new(node).is_dir() {
        panel.open_directories.insert(node.to_string());
        let id = egui::Id::new(format!("fep_dir_{node}"));
        let mut cs =
            egui::collapsing_header::CollapsingState::load_with_default_open(ctx, id, false);
        cs.set_open(true);
        cs.store(ctx);
    }
}

/// Convert the UI-layer `AllocParams` to the ff-dscatalog `AllocParams`.
///
/// Returns `Err` if the dataset name is not a valid DSN.
fn ui_params_to_ds_params(
    params: crate::dataset_alloc_dialog::AllocParams,
    form: &crate::dataset_alloc_dialog::AllocDatasetForm,
) -> Result<DsAllocParams, String> {
    let dsn = Dsn::parse(&params.dataset_name)
        .map_err(|_| format!("'{}': invalid dataset name", params.dataset_name))?;
    let dsorg = match params.dsorg {
        Dsorg::Ps => DsDsorg::PS,
        Dsorg::Po | Dsorg::Pdse => DsDsorg::PO,
        Dsorg::Gdg => DsDsorg::GDG,
    };
    let subtype = match params.dsorg {
        Dsorg::Pdse => Some(PartitionedSubtype::PDSE),
        Dsorg::Po => Some(PartitionedSubtype::PDS),
        _ => None,
    };
    let recfm = match params.recfm {
        Recfm::Fb => Some(DsRecfm::FB),
        Recfm::F => Some(DsRecfm::F),
        Recfm::Vb => Some(DsRecfm::VB),
        Recfm::V => Some(DsRecfm::V),
        Recfm::U => Some(DsRecfm::U),
    };
    let gdg_limit = params.gdg_limit.map(|n| n as u8);
    let gdg_scratch = if params.dsorg == Dsorg::Gdg {
        Some(form.scratch)
    } else {
        None
    };
    Ok(DsAllocParams {
        dsn,
        dsorg,
        recfm,
        lrecl: Some(params.lrecl),
        blksize: if params.blksize == 0 {
            None
        } else {
            Some(params.blksize)
        },
        dir_blocks: params.dir_blocks,
        gdg_limit,
        gdg_scratch,
        subtype,
        description: params.description,
        scope: CatalogScope::User,
    })
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
                // No CLI args -- restore previous session tabs (Req 5 AC 1, 2).
                let state = session.load();
                let restored_any = !SessionManager::tab_uris(&state).is_empty();
                // Extract workspace path before any mutable borrows.
                let ws_path_to_restore = state.active_workspace_path.clone();
                // Validates: Requirement 6.2 (view-zoom) -- restore global zoom offset.
                if state.global_zoom_offset != 0 {
                    self.zoom = ff_zoom::ZoomState::from_persisted(
                        state.global_zoom_offset,
                        &ff_zoom::ZoomConfig::default(),
                    );
                }
                // Validates: Requirement 12.4 (function-keys-and-history) -- restore PFSHOW state.
                self.key_bar_visible = state.key_bar_visible;
                // Validates: Requirement 23.9 (file-tree-panel) -- restore sidebar width.
                if state.file_explorer_sidebar_width >= 120.0 {
                    self.file_explorer_panel.sidebar_width = state.file_explorer_sidebar_width;
                }
                // Validates: command-palette Requirement 5.2 -- restore recent palette commands.
                self.recent_palette_commands = state.recent_palette_commands.clone();
                // Validates: global-search Requirement 6.2 -- restore search history.
                self.search_results_panel
                    .restore_history(state.search_history.clone());
                // Validates: Requirement 2.1, 2.2 (virtual-catalog-manager) -- restore catalog registry.
                self.files_panel.registry = session.load_catalog_registry();
                // Validates: Requirement 14.1-14.5 -- create default Home catalog when none exist.
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
                // Validates: Requirement 14.1 / 14.1b -- ensure POM tab is always present.
                if !restored_any {
                    self.tabs.close_welcome_tab();
                    self.tabs.insert_pom_tab(&self.runtime);
                } else {
                    ensure_pom_tab_present(&mut self.tabs, &self.runtime);
                }
                // Validates: workspace-model Requirement 5.2, 5.3 -- restore active workspace.
                // Done after session borrow ends to allow &mut self in open_workspace.
                if let Some(ws_path_str) = ws_path_to_restore {
                    let ws_path = std::path::PathBuf::from(&ws_path_str);
                    if ws_path.exists() {
                        self.open_workspace(&ws_path);
                    } else {
                        self.open_error = Some(format!(
                            "Workspace '{}' not found -- starting without workspace",
                            ws_path.display()
                        ));
                    }
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

        // Ctrl+Shift+P -- toggle Command Palette -- Validates: command-palette Req 1.1, 1.5
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl && i.modifiers.shift) {
            if self.palette_state.open {
                self.palette_state.close();
            } else {
                self.palette_state.open();
            }
        }

        // Ctrl+Shift+F -- open Global Search panel -- Validates: global-search Req 1.1, 1.3
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift) {
            self.open_or_focus_search_panel();
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
            || self.palette_state.open
            || self.show_history_list.is_some()
            || self.show_unsaved_workspace_dialog
            || !matches!(self.files_panel.dialog, files_panel::FilesDialogState::None);
        {
            let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
            let tab_count = self.tabs.len();
            let pom_active = self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu;
            let is_file_explorer = self.tabs.active_tab().kind == TabKind::FileExplorerPanel;
            let cmd_id = egui::Id::new("command_field_input");
            let cmd_has_focus = ctx.memory(|m| m.focused() == Some(cmd_id));

            let (tab_pressed, shift_tab_pressed) = ctx.input_mut(|i| {
                if self.modal_open {
                    return (false, false);
                }
                let shift = i.modifiers.shift;
                let tab = i.key_pressed(egui::Key::Tab);
                if tab {
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

            // Validates: Requirement 20.1 — Tab from shell command field while File Explorer
            // is active transfers focus into the explorer tree instead of cycling focus stops.
            // Tab past the last tree node exits the tree and returns focus to CommandField.
            // Escape while explorer has focus also exits the tree back to CommandField.
            if !self.modal_open
                && is_file_explorer
                && self.file_explorer_panel.explorer_focused
                && ctx.input(|i| i.key_pressed(egui::Key::Escape))
            {
                self.file_explorer_panel.explorer_focused = false;
                self.file_explorer_panel.cursor_node = None;
                self.focus_stop = FocusStop::CommandField;
                self.command_field_focus_requested = true;
            } else if tab_pressed
                && is_file_explorer
                && (cmd_has_focus || self.file_explorer_panel.explorer_focused)
            {
                let entering = !self.file_explorer_panel.explorer_focused;

                if entering {
                    // Entering from command field — open first catalog and land on
                    // its first child.
                    self.file_explorer_panel.explorer_focused = true;
                    // Open the first catalog node immediately.
                    let first_cat =
                        crate::file_explorer_panel::collect_visible_node_paths_with_dirs(
                            &self.files_panel.registry,
                            &self.files_panel,
                            &self.file_explorer_panel.open_catalogs,
                            &self.file_explorer_panel.open_directories,
                        )
                        .into_iter()
                        .next();
                    if let Some(ref node) = first_cat {
                        auto_open_node(ctx, node, &mut self.file_explorer_panel);
                    }
                    // Recompute with the catalog now open.
                    let visible2 = crate::file_explorer_panel::collect_visible_node_paths_with_dirs(
                        &self.files_panel.registry,
                        &self.files_panel,
                        &self.file_explorer_panel.open_catalogs,
                        &self.file_explorer_panel.open_directories,
                    );
                    // Land on first child (index 1), or the catalog itself if empty.
                    self.file_explorer_panel.cursor_node =
                        visible2.into_iter().nth(1).or(first_cat);
                } else {
                    // Already inside the tree — open current container (if any) then advance.
                    if let Some(ref cur) = self.file_explorer_panel.cursor_node.clone() {
                        if cur.starts_with("cat:") || std::path::Path::new(cur.as_str()).is_dir() {
                            auto_open_node(ctx, cur, &mut self.file_explorer_panel);
                        }
                    }
                    // Recompute visible after any container open.
                    let visible = crate::file_explorer_panel::collect_visible_node_paths_with_dirs(
                        &self.files_panel.registry,
                        &self.files_panel,
                        &self.file_explorer_panel.open_catalogs,
                        &self.file_explorer_panel.open_directories,
                    );
                    let current_idx = self
                        .file_explorer_panel
                        .cursor_node
                        .as_ref()
                        .and_then(|c| visible.iter().position(|p| p == c));
                    let next = current_idx.map(|i| i + 1).unwrap_or(0);
                    if next >= visible.len() {
                        // Past the last node — exit tree, return to CommandField.
                        self.file_explorer_panel.explorer_focused = false;
                        self.file_explorer_panel.cursor_node = None;
                        self.focus_stop = FocusStop::CommandField;
                        self.command_field_focus_requested = true;
                    } else {
                        self.file_explorer_panel.cursor_node = visible.into_iter().nth(next);
                    }
                }
            } else if tab_pressed {
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
        // Validates: Requirement 2.5 -- clear stale automation entries at frame start.
        self.automation.begin_frame();
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

        // Catalog Manager Dialogs - Req 3.1-3.8, 4.1-4.5
        match &mut self.files_panel.dialog {
            files_panel::FilesDialogState::NewCatalog(ref mut form) => {
                let outcome =
                    catalog_manager_dialog::render(ctx, form, &mut self.files_panel.registry);
                if outcome == DialogOutcome::Confirmed {
                    // Persist immediately so a force-close does not lose the new catalog (B020).
                    if let Some(session) = &self.session {
                        session.save_catalog_registry(&self.files_panel.registry);
                    }
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                } else if outcome == DialogOutcome::Cancelled {
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
                        // Persist immediately so a force-close does not lose the deletion (B020).
                        if let Some(session) = &self.session {
                            session.save_catalog_registry(&self.files_panel.registry);
                        }
                    }
                }
                self.files_panel.dialog = files_panel::FilesDialogState::None;
            }
            files_panel::FilesDialogState::AllocateDataset(ref mut form) => {
                let outcome = dataset_alloc_dialog::render(ctx, form);
                if outcome == AllocOutcome::Confirmed {
                    // Req 13.1 — validate form (duplicate check deferred to SQLite uniqueness)
                    match validate_for_catalog(form, &[]) {
                        Ok(params) => {
                            if let Some(cat) = self.files_panel.pending_alloc_catalog.take() {
                                // Convert UI AllocParams -> ff-dscatalog AllocParams
                                let ds_params = ui_params_to_ds_params(params, form);
                                match ds_params {
                                    Ok(p) => {
                                        if let Err(e) = self.files_panel.registry.allocate(&cat, p)
                                        {
                                            form.error = Some(format!("Allocation failed: {e}"));
                                            self.files_panel.pending_alloc_catalog = Some(cat);
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        form.error = Some(e);
                                        self.files_panel.pending_alloc_catalog = Some(cat);
                                        return;
                                    }
                                }
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
            // Validates: workspace-model Requirement 5.1 -- persist active workspace path.
            let ws_path = self
                .active_workspace
                .as_ref()
                .and_then(|ws| ws.file_path.as_ref())
                .map(|p| p.to_string_lossy().into_owned());
            session.save_with_workspace(
                &self.tabs,
                self.zoom.offset().value(),
                self.key_bar_visible,
                self.file_explorer_panel.sidebar_width,
                ws_path,
                self.recent_palette_commands.clone(),
                self.search_results_panel.history.clone(),
            );
            session.save_catalog_registry(&self.files_panel.registry);
        }
        self.runtime.block_on(self.app.shutdown());
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

#[cfg(test)]
mod startup_tests {
    use super::{ensure_default_home_catalog, ensure_pom_tab_present};
    use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use std::path::PathBuf;
    use tokio::runtime::Runtime;

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

    // === Phase CL: POM guaranteed on startup (Req 14.1, 14.1a, 14.1b) ===

    /// Validates: Requirement 14.1 -- empty session opens a single POM tab.
    #[test]
    fn empty_session_opens_single_pom_tab() {
        // Validates: Requirement 14.1
        let runtime = Runtime::new().expect("runtime");
        let mut tabs = TabManager::new(&runtime, "");
        tabs.close_welcome_tab();
        tabs.insert_pom_tab(&runtime);
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs.tabs()[0].kind, TabKind::PrimaryOptionMenu);
    }

    /// Validates: Requirement 14.1a -- session with POM tab: ensure_pom_tab_present is a no-op.
    #[test]
    fn session_with_pom_tab_restores_exactly() {
        // Validates: Requirement 14.1a
        let runtime = Runtime::new().expect("runtime");
        let mut tabs = TabManager::new(&runtime, "");
        tabs.close_welcome_tab();
        tabs.insert_pom_tab(&runtime);
        tabs.new_untitled_tab(&runtime);
        let count_before = tabs.len();
        ensure_pom_tab_present(&mut tabs, &runtime);
        assert_eq!(
            tabs.len(),
            count_before,
            "must not add a second POM when one already exists"
        );
        assert_eq!(tabs.tabs()[0].kind, TabKind::PrimaryOptionMenu);
    }

    /// Validates: Requirement 14.1b -- session without POM tab gets POM prepended at index 0.
    #[test]
    fn session_without_pom_tab_prepends_pom() {
        // Validates: Requirement 14.1b
        let runtime = Runtime::new().expect("runtime");
        let mut tabs = TabManager::new(&runtime, "");
        tabs.close_welcome_tab();
        tabs.new_untitled_tab(&runtime);
        tabs.new_untitled_tab(&runtime);
        let count_before = tabs.len();
        ensure_pom_tab_present(&mut tabs, &runtime);
        assert_eq!(
            tabs.len(),
            count_before + 1,
            "must prepend a POM tab when none exists"
        );
        assert_eq!(
            tabs.tabs()[0].kind,
            TabKind::PrimaryOptionMenu,
            "POM must be at index 0"
        );
    }

    /// Validates: Requirement 14.3 -- returned true signals caller to persist registry.
    /// Validates: Requirement 14.5 -- fallback path is used when home_path is provided.
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
