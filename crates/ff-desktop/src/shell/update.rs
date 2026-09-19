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
use ff_dscatalog::{
    dataset::{
        AllocParams as DsAllocParams, Dsorg as DsDsorg, PartitionedSubtype, Recfm as DsRecfm,
    },
    dsn::Dsn,
    hierarchy::CatalogScope,
};

use super::helpers::*;
use super::WorkbenchShell;
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

/// Ensure a Home Context (POM) tab is present at index 0.
///
/// If no Home Context (POM) tab exists, inserts one at index 0.
/// Called after session restore so the POM is always reachable on startup.
///
/// Validates: Requirement 14.1b
pub(super) fn ensure_pom_tab_present(
    tabs: &mut crate::tab_manager::TabManager,
    runtime: &tokio::runtime::Runtime,
) {
    let has_pom = tabs.tabs().iter().any(|t| t.is_home);
    if !has_pom {
        tabs.insert_pom_tab(runtime);
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
                // Validates: Requirement 21.1, 21.5 -- collect a Workspace_Descriptor
                // for every persisted tab (owned, so the session borrow can end
                // before we reconstruct with &mut self). Legacy sessions map via
                // effective_descriptor (Requirement 21.10).
                let restore_descriptors: Vec<ff_session::WorkspaceDescriptor> = state
                    .tabs
                    .iter()
                    .filter_map(|t| t.effective_descriptor())
                    .collect();
                let restored_any = !restore_descriptors.is_empty();
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
                    self.file_explorer_panel_width = state.file_explorer_sidebar_width;
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
                // The `session` immutable borrow ends here; reconstruction below
                // needs `&mut self` (open_settings_view etc.), so it runs after.
                if restored_any {
                    self.tabs.close_welcome_tab();
                }
                self.restore_workspace_descriptors(&restore_descriptors);
                // Validates: Requirement 14.1 / 14.1b / 21.8 -- POM always present.
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

            // Validates: menu-workspace Requirement 4.1/4.2 (revised, CR-CH-021),
            // 4.6 -- built-in menus are code-only; only ensure the (possibly
            // empty) menus/ directory exists. The compiled Recovery_Baseline is
            // used at render time when no valid user file exists.
            if let Some(_session) = &self.session {
                if let Ok(mut udd) = ff_session::UserDataDir::resolve(None) {
                    let _ = udd.initialise();
                    crate::menu_workspace::defaults::ensure_menus_dir(udd.path());
                }
            }

            // Startup focus is handled by command_field_focus_requested = true (set in new()).
        }

        // Check if file.exit handler fired
        if *self.should_close.lock().expect("close lock") {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Process any pending file open (set by file.open handler or menu)
        // Drain notification channel into queue -- Validates: notification-system Requirement 1.1
        {
            let mut queue = self.notification_queue.lock().expect("notification queue");
            while let Ok(n) = self.notification_rx.try_recv() {
                queue.push(n);
            }
        }

        // CR-CH-023 Req 16.1a: when the active tab changes (tab click, SWAP,
        // END navigation, close, etc.), re-arm command-field focus so entering
        // a Workspace always places focus on the command line.
        let active_now = self.tabs.active_index();
        if active_now != self.last_active_tab {
            self.last_active_tab = active_now;
            self.command_field_focus_requested = true;
        }

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
        // F3/END from the Files Panel (or another panel deferring an END) pops
        // one level of the tab's Navigation_Stack (menu-workspace Req 14.4,
        // CR-CH-022) -- the same uniform END path as the command.
        if self.pending_return_to_pom {
            self.pending_return_to_pom = false;
            self.nav_end();
        }

        // Process deferred Menu_Workspace option click.
        // Validates: menu-workspace Requirement 3.2, 10.1, 10.3, 10.6
        if let Some(option) = self.pending_menu_option.take() {
            if let Some(target) = option.target.clone() {
                // Req 10.6: inline [options.target] wins over `command`.
                self.dispatch_command_target(&target);
            } else {
                // Req 10.1/10.3: resolve to a user-owned target, else fall
                // through to the existing pipeline (Req 10.2).
                match self.resolve_and_dispatch_command(&option.command) {
                    super::target_dispatch::ResolveOutcome::Dispatched => {}
                    super::target_dispatch::ResolveOutcome::FallThrough => {
                        self.handle_command(&option.command);
                    }
                }
            }
        }
        if let Some(idx) = self.detach_pending.take() {
            if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                tab.is_floating = true;
                let tab_id = tab.id;
                let vid = egui::ViewportId::from_hash_of(format!("floating_tab_{}", tab_id.0));
                self.floating_tabs.push(super::FloatingTab {
                    viewport_id: vid,
                    tab_id,
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
                // Resolve the tab's CURRENT index from its stable id (it may have
                // shifted while other tabs detached/redocked).
                if let Some(tab_idx) = self.tabs.index_of_id(ft.tab_id) {
                    if let Some(tab) = self.tabs.tabs_mut().get_mut(tab_idx) {
                        tab.is_floating = false;
                    }
                    // CR-CH-035 (Req 18.3/18.9): faithful redock -- move the tab
                    // back to its origin index preserving the order of the other
                    // tabs (remove+reinsert, not a positional swap). `move_tab`
                    // clamps an origin beyond the current count to the end
                    // (append, per 18.3).
                    self.tabs.move_tab(tab_idx, ft.origin_index);
                }
            }
        }
        // File-backed active theme + hot-reload (CR-NR-074 Req 19.6). Resolve the
        // active theme file, and reload the palette when the file changes on disk
        // or when the configured active theme changes. This keeps the palette
        // driven by the theme file (not just the compiled mode default).
        {
            let themes_dir = self.themes_dir();
            // Determine the active theme's file path: prefer theme.active_name,
            // else the built-in for the current theme.active mode.
            let active_name = self
                .config_handle
                .get_string(ff_config::keys::theme::ACTIVE_NAME)
                .unwrap_or_default();
            let active_name = active_name.trim();
            let theme_path = if !active_name.is_empty() {
                Some(themes_dir.join(format!(
                    "{}.toml",
                    crate::theme_defaults::theme_slug(active_name)
                )))
            } else {
                self.config_handle
                    .get_string(ff_config::keys::theme::ACTIVE)
                    .ok()
                    .and_then(|m| ff_theme::mode::VisualMode::from_str_loose(&m))
                    .map(|mode| {
                        let name = ff_theme::defaults::default_palette_for_mode(mode).name;
                        themes_dir
                            .join(format!("{}.toml", crate::theme_defaults::theme_slug(&name)))
                    })
            };

            if let Some(path) = theme_path {
                let disk_mtime = std::fs::metadata(&path)
                    .ok()
                    .and_then(|m| m.modified().ok());
                let tracked = self.active_theme_file.as_ref();
                let path_changed = tracked.map(|(p, _)| p != &path).unwrap_or(true);
                let mtime_changed = match (tracked, disk_mtime) {
                    (Some((p, t)), Some(dm)) => p == &path && *t != dm,
                    _ => false,
                };
                if path_changed || mtime_changed {
                    // Reload from the resolver so a missing/invalid file falls
                    // back to Default Legacy (Req 19.5) without crashing.
                    self.palette = crate::theme_defaults::resolve_startup_palette(
                        &self.config_handle,
                        &themes_dir,
                    );
                    self.active_theme_file = disk_mtime.map(|dm| (path, dm));
                }
            }
        }

        // OS dark/light mode follow -- Validates: theme-and-appearance Requirement 16.1-16.4
        // When theme.follow_os is true, read the OS dark/light preference from the egui
        // context each frame and apply the matching palette without writing to theme.active.
        let follow_os = self
            .config_handle
            .get_bool(ff_config::keys::theme::FOLLOW_OS)
            .unwrap_or(false);
        if follow_os {
            let os_dark = ctx.style().visuals.dark_mode;
            let target_mode = if os_dark {
                ff_theme::mode::VisualMode::Dark
            } else {
                ff_theme::mode::VisualMode::Light
            };
            if target_mode != self.palette.mode {
                self.palette = ff_theme::defaults::default_palette_for_mode(target_mode);
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
        self.modal_open = self.show_about
            || self.palette_state.open
            || self.show_history_list.is_some()
            || self.show_swap_list.is_some()
            || self.show_unsaved_workspace_dialog
            || !matches!(self.files_panel.dialog, files_panel::FilesDialogState::None);
        {
            let is_file_explorer = self.tabs.active_tab().kind == TabKind::FileExplorerPanel;
            let cmd_id = egui::Id::new("command_field_input");
            let cmd_has_focus = ctx.memory(|m| m.focused() == Some(cmd_id));

            // CR-CH-023 unified tab-order: the shell owns only the Boundary_Policy
            // (command-line entry, menu-bar-last, wrap). Interior order is
            // egui-native, so between boundaries the Tab event is LEFT in the
            // queue for egui to process. At a boundary the shell consumes Tab and
            // redirects focus. The File Explorer keeps its own tree-transfer.
            //
            // Snapshot the boundary anchors reported by last frame's render and
            // the currently-focused widget so we can decide, BEFORE egui gets the
            // Tab event, whether this press is a boundary (shell handles it) or an
            // interior/menu-internal move (egui-native handles it).
            let first_interior = self.first_interior_id;
            let last_interior = self.last_interior_id;
            let menu_first_id = self.menu_first_id;
            let menu_last_id = self.menu_last_id;
            let focused = ctx.memory(|m| m.focused());
            let on_menu_last = menu_last_id.is_some() && focused == menu_last_id;
            let on_menu_first = menu_first_id.is_some() && focused == menu_first_id;
            let on_last_interior = last_interior.is_some() && focused == last_interior;
            let on_first_interior = first_interior.is_some() && focused == first_interior;

            // Classify the boundary this frame (None = not a shell boundary; let
            // egui-native traversal handle it). Computed from focus position only.
            #[derive(Clone, Copy)]
            enum Boundary {
                None,
                /// Focus the given (reliable) id -- used for menu-bar buttons.
                Focus(egui::Id),
                /// Latch: interior render focuses its first control (fresh id).
                FirstInterior,
                /// Latch: interior render focuses its last control (fresh id).
                LastInterior,
                /// Re-arm command-field focus.
                CommandField,
            }
            let is_explorer_transfer = is_file_explorer && (cmd_has_focus || self.nav_focused);

            // Detect Tab/Shift+Tab AND, in the SAME input pass, decide the
            // boundary and consume the event ONLY when the shell will handle it.
            // Consuming during detection (not afterwards) is essential: egui
            // latches the Tab into its `give_to_next` focus machinery while
            // rendering, which would otherwise override our request_focus (B056).
            let boundary = ctx.input_mut(|i| {
                if self.modal_open {
                    return Boundary::None;
                }
                let shift = i.modifiers.shift;
                if !i.key_pressed(egui::Key::Tab) {
                    return Boundary::None;
                }
                // The File Explorer tree-transfer consumes Tab in its own branch.
                if is_explorer_transfer {
                    return Boundary::None;
                }
                // Boundary_Policy (B056). egui-native traversal handles the ORDER
                // WITHIN the interior and WITHIN the menu bar; the shell handles
                // the boundary JUMPS. The command -> first-interior jump uses a
                // one-shot latch (`FirstInterior`) honoured by the interior render
                // with the FRESH same-frame id, because interior option auto-ids
                // do NOT round-trip through egui focus when requested from a
                // previous frame. Menu-bar jumps use the reliable captured
                // menu-button ids directly.
                // egui-native traversal already produces the correct ORDER
                // through the interior and the menu bar and wraps back to the
                // command field on its own. The shell only needs to intercept the
                // command-field -> interior entry so egui does not first stop on
                // the SCROLL field / other command-panel widgets: use the
                // one-shot latch that focuses the FRESH first-interior id (B056).
                // Reverse Shift+Tab from the command field similarly latches to
                // the LAST interior. Everything else is egui-native.
                // egui-native traversal handles the interior order, interior ->
                // menu bar, and the menu-bar order. The shell handles the moves
                // egui gets wrong on its own (B056):
                //  - FORWARD from the command field: egui would stop on the
                //    SCROLL field (chrome) next, so latch to the FRESH
                //    first-interior id instead.
                //  - FORWARD from the last menu button: egui wraps to the first
                //    focusable of the frame (the first menu button, since the
                //    menu bar renders first), NOT the command field -- so wrap to
                //    the command field explicitly.
                //  - REVERSE from the command field: jump to the last menu button.
                //  - REVERSE from the first menu button: jump to the last interior
                //    (latch) or the command field when there is no interior.
                // Menu-bar button ids are reliable (they round-trip through
                // egui focus); the interior uses the fresh-id latch.
                let decision = if !shift {
                    if cmd_has_focus && first_interior.is_some() {
                        Boundary::FirstInterior
                    } else if on_menu_last {
                        Boundary::CommandField
                    } else {
                        Boundary::None
                    }
                } else if cmd_has_focus {
                    menu_last_id.map(Boundary::Focus).unwrap_or(Boundary::None)
                } else if on_menu_first {
                    if last_interior.is_some() {
                        Boundary::LastInterior
                    } else {
                        Boundary::CommandField
                    }
                } else {
                    Boundary::None
                };
                let _ = (menu_first_id, on_last_interior, on_first_interior);
                // Consume Tab only when the shell handles this boundary; otherwise
                // leave it for egui-native interior/menu-bar traversal
                // (Req 16.4, 16.6, 16.14).
                if !matches!(decision, Boundary::None) {
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
                decision
            });

            // Validates: Requirement 20.1 (file-tree-panel) -- Escape exits the
            // explorer tree back to the command field.
            if !self.modal_open
                && is_file_explorer
                && self.nav_focused
                && ctx.input(|i| i.key_pressed(egui::Key::Escape))
            {
                self.nav_focused = false;
                self.nav_selection.cursor = None;
                self.command_field_focus_requested = true;
            } else if is_explorer_transfer
                && ctx.input_mut(|i| {
                    // Modern explorer Tab focus-transfer (Req 20.1 / 24.9):
                    // consume Tab in its own branch and move the tree cursor.
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
                        true
                    } else {
                        false
                    }
                })
            {
                use crate::explorer_view::{first_row_id, next_row_id};
                let next = if !self.nav_focused {
                    self.nav_focused = true;
                    first_row_id(&self.nav_model)
                } else {
                    self.nav_selection
                        .cursor
                        .and_then(|c| next_row_id(&self.nav_model, c))
                };
                match next {
                    Some(id) => self.nav_selection.move_cursor(id),
                    None => {
                        self.nav_focused = false;
                        self.nav_selection.cursor = None;
                        self.command_field_focus_requested = true;
                    }
                }
            } else {
                match boundary {
                    Boundary::Focus(id) => ctx.memory_mut(|m| m.request_focus(id)),
                    Boundary::FirstInterior => {
                        self.focus_first_interior_requested = true;
                        self.focus_last_interior_requested = false;
                    }
                    Boundary::LastInterior => {
                        self.focus_last_interior_requested = true;
                        self.focus_first_interior_requested = false;
                    }
                    Boundary::CommandField => self.command_field_focus_requested = true,
                    Boundary::None => {}
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

        // CR-CH-028 (Requirement 12.4/12.5): refresh the Cursor_Context snapshot
        // from live focus/selection BEFORE any dispatch this frame, so the
        // function-key path, the command line, and registry dispatch all carry
        // the same per-invocation package.
        self.refresh_cursor_context_snapshot(ctx);

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
            // A shortcut binding may target any Command_Target, including a
            // user-defined command id (command-framework Requirement 8.5,
            // command-configurator Requirement 4.4). Built-in commands fall
            // through to the existing pipeline unchanged (Requirement 10.2).
            // The current Command ===> field content is merged as the argument
            // (Req 9.8, B066): type `1` + F9=SWAP -> `SWAP 1`.
            self.dispatch_key_command(&cmd);
        }

        self.render_central_panel(ctx);

        // ── Floating tab viewports — Validates: Requirement 18.1, 18.2, 18.5, 18.8 ──
        // CR-CH-035 (B045): render each Detached_Workspace with an IMMEDIATE
        // viewport (synchronous, so the closure can borrow `&mut self`), drawing
        // the tab's REAL Context via `render_active_tab_body` -- not a placeholder.
        // The detached tab is temporarily made the active tab for the duration of
        // its render, then the previous active index is restored, so the shared
        // render path (which operates on the active tab) draws the correct tab
        // without duplicating the whole `match tab.kind`. Detached tabs are
        // `is_floating`, so the primary tab bar never shows them as active; there
        // is no double-render of the same tab in one frame.
        for ft_idx in 0..self.floating_tabs.len() {
            let vid = self.floating_tabs[ft_idx].viewport_id;
            let tab_id = self.floating_tabs[ft_idx].tab_id;
            let origin_index = self.floating_tabs[ft_idx].origin_index;
            // Resolve the live index from the stable id each frame.
            let Some(tab_index) = self.tabs.index_of_id(tab_id) else {
                continue;
            };
            let title = self
                .tabs
                .tabs()
                .get(tab_index)
                .map(|t| {
                    super::truncate_title(
                        &format!("{} -- FileForge Workbench", super::title_line_text(t)),
                        80,
                    )
                })
                .unwrap_or_else(|| "FileForge Workbench".to_string());
            let redock_tx = Arc::clone(&self.redock_pending);
            ctx.show_viewport_immediate(
                vid,
                egui::ViewportBuilder::default().with_title(&title),
                |vctx, _class| {
                    // Detect OS-window close -> queue a redock at the origin index.
                    if vctx.input(|i| i.viewport().close_requested()) {
                        redock_tx.lock().expect("redock lock").push(origin_index);
                        vctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    }
                    if tab_index >= self.tabs.len() {
                        return;
                    }
                    // Title_Line for the detached tab (read-only chrome, Req 18.1).
                    egui::TopBottomPanel::top(egui::Id::new(("floating_title", tab_index))).show(
                        vctx,
                        |ui| {
                            ui.label(
                                egui::RichText::new(super::title_line_text(
                                    &self.tabs.tabs()[tab_index],
                                ))
                                .monospace()
                                .strong(),
                            );
                        },
                    );
                    // Render the tab's REAL Context body by temporarily making it
                    // the active tab (Req 18.2/18.8), then restoring.
                    let saved_active = self.tabs.active_index();
                    self.tabs.set_active(tab_index);
                    egui::CentralPanel::default().show(vctx, |ui| {
                        self.render_active_tab_body(vctx, ui);
                    });
                    self.tabs.set_active(saved_active);
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
            egui::Window::new("Reset to barebones?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
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
                        if ui.button("Confirm reset").clicked() {
                            confirm_clicked = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });
                });
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
                self.file_explorer_panel_width,
                ws_path,
                self.recent_palette_commands.clone(),
                self.search_results_panel.history.clone(),
                self.config_panel.namespace_filter.as_deref(),
            );
            session.save_catalog_registry(&self.files_panel.registry);
        }

        // Persist the command-line history (function-keys-and-history Req 6.3).
        self.persist_command_history();

        self.runtime.block_on(self.app.shutdown());
    }
}

impl super::WorkbenchShell {
    /// Write the current command-line history to the History_Store
    /// (function-keys-and-history Requirement 6.3). Best-effort: a write failure
    /// is logged, never fatal. Called from `on_exit`; extracted so it is
    /// unit-testable without driving a full eframe shutdown.
    ///
    /// Validates: function-keys-and-history Requirement 6.3, 6.7
    pub(super) fn persist_command_history(&self) {
        if let Some(store) = &self.history_store {
            let ring = ff_command::CommandLineRing::from_command_strings(
                self.command_line_history.list(),
                self.command_line_history.max_entries(),
            );
            if let Err(e) = store.save(&ring) {
                ff_logging::log_warn!("[keys] command history save failed: {}", e);
            }
        }
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
        assert!(tabs.tabs()[0].is_home);
        assert_eq!(tabs.tabs()[0].kind, TabKind::MenuWorkspace);
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
        assert!(tabs.tabs()[0].is_home);
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
        assert!(tabs.tabs()[0].is_home, "POM must be at index 0");
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
