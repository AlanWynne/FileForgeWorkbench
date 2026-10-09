//! # eframe::App implementation
//!
//! The `update()` frame loop and `on_exit()` callback for WorkbenchShell.

use eframe::egui;

use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};

use super::WorkbenchShell;
use ff_fftest::AutomationRegistry as _;
use std::path::PathBuf;

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

impl eframe::App for WorkbenchShell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // One-shot startup (CLI files / session restore / default catalog / POM /
        // menus dir). Extracted to `update_startup.rs` (Phase 2 task 2.2).
        self.run_startup();

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
        if active_now != self.focus.last_active_tab {
            self.focus.last_active_tab = active_now;
            self.focus.command_field_focus_requested = true;
        }

        let pending = self.pending_open.lock().expect("pending lock").take();
        if let Some((p, owning_env)) = pending {
            if !p.is_empty() {
                // CR-CH-053 Task 19: bind the opened tab's Owning_Environment from
                // the captured origin (Req 15.2); `None` -> host FS default
                // (Req 15.3).
                if let Err(e) = self.shell_open_file_with_env(&p, owning_env.as_deref()) {
                    self.open_error = Some(e);
                } else {
                    self.open_error = None;
                }
            }
        }

        // Ctrl+S -- dispatch the SAVE command (CR-CH-054 point-fix: route through
        // the single front door -> the active environment, NOT a direct
        // `save_active_tab` call). On an editor Context FFEDIT's dirty-aware SAVE
        // runs (clean = no-op, dirty = write + stay); elsewhere SAVE is unresolved.
        // This keeps the key and the typed `SAVE` verb one code path (every user
        // action is a command). Suppressed when a modal dialog is open.
        if !self.modal_open && ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            self.handle_command("SAVE");
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
        if self.pending_tab_actions.new_pom {
            self.pending_tab_actions.new_pom = false;
            self.tabs.insert_pom_tab(&self.runtime);
        }
        if self.pending_tab_actions.new_file {
            self.pending_tab_actions.new_file = false;
            self.shell_new_untitled();
        }
        // F3/END from the Files Panel (or another panel deferring an END) pops
        // one level of the tab's Navigation_Stack (menu-workspace Req 14.4,
        // CR-CH-022) -- the same uniform END path as the command.
        if self.pending_tab_actions.return_to_pom {
            self.pending_tab_actions.return_to_pom = false;
            self.nav_end();
        }

        // Process deferred Menu_Workspace option click.
        // CR-CH-043 (menu-workspace Req 19.1/19.3, command-framework Req 14.1):
        // a CLICK activates the option through the SAME single Option-Selection
        // path as a typed Option_Key -- `activate_menu_option`, which honours an
        // inline [options.target] (Req 10.6) inside the command pipeline and
        // otherwise resolves-and-dispatches the option's command (falling through
        // to `handle_command`). There is no click-only dispatch pre-branch.
        // Validates: menu-workspace Requirement 3.2, 10.1, 10.3, 10.6, 19.1, 19.3
        if let Some(option) = self.pending_menu_option.take() {
            self.activate_menu_option(option.target.as_ref(), &option.command);
        }
        if let Some(idx) = self.detach_split.detach_pending.take() {
            if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                tab.is_floating = true;
                let tab_id = tab.id;
                let vid = egui::ViewportId::from_hash_of(format!("floating_tab_{}", tab_id.0));
                ff_logging::log_info!(
                    "[shell] detach: tab {} (index {}) -> Detached_Workspace {:?}",
                    tab_id.0,
                    idx,
                    vid
                );
                self.detach_split.floating_tabs.push(super::FloatingTab {
                    viewport_id: vid,
                    tab_id,
                    origin_index: idx,
                    cmd_ctx: super::WorkspaceCommandContext::default(),
                });
            }
        }

        // === Drop stale Detached_Workspaces -- Validates: Requirement 18.3 ===
        // CR-CH-037: the window Close button now runs RETURN (not redock). When
        // RETURN closes a detached POM workspace, its tab is removed from the
        // TabManager, so any FloatingTab whose stable id no longer resolves is
        // stale -- drop it so its OS window is not re-created next frame. (Re-dock
        // is now the explicit DOCK command, CR-NR-088, which drops the FloatingTab
        // itself.)
        self.detach_split
            .floating_tabs
            .retain(|ft| self.tabs.index_of_id(ft.tab_id).is_some());
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
        // Ctrl+Scroll global zoom + window-drag DPI handling. Extracted to
        // `update_input.rs` (Phase 2 task 2.2).
        self.handle_zoom_and_dpi(ctx);
        // The unified Tab-order focus cycle / Boundary_Policy. Extracted to
        // `update_input.rs` (Phase 2 task 2.2).
        self.handle_tab_order_focus(ctx);
        // CR-CH-041 (Req 16.2): the menu bar is chrome OWNED BY THE INSTANCE and
        // rendered at its placement. While split, each region draws its own
        // instance's menu bar (see `render_split_region`), so the single
        // app-level menu bar is suppressed -- exactly as the top-level command
        // field is (below). Unsplit behaviour is byte-identical to before.
        if !self.tabs.is_split() {
            self.render_menu_bar(ctx);
        }
        // Validates: Requirement 2.5 -- clear stale automation entries at frame start.
        self.automation.begin_frame();
        self.render_tab_bar(ctx);
        // CR-CH-041 (Req 16.2): the Title_Line is likewise per-instance chrome;
        // suppressed at the app level while split (each region draws its own).
        if !self.tabs.is_split() {
            self.render_title_line(ctx);
        }
        // CR-NR-094 Slice 2d (Req 15.1, 15.2): while the Workbench is split, each
        // region carries its OWN `Command ===>` line (rendered inside every split
        // region), so the single top-level command field is suppressed to avoid
        // an ambiguous "which region does this line target?" shared field. Unsplit
        // behaviour is byte-identical to before.
        if !self.tabs.is_split() {
            self.render_command_field(ctx);
        }
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
        // CR-CH-037/B068: shared F-key resolution (also used by detached windows
        // via `dispatch_detached_function_key`). Suppressed while a modal is open.
        let fkey_cmd = self.resolve_function_key_command(ctx);
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

        // Floating tab viewports (Detached_Workspaces). Extracted verbatim to
        // `update_floating.rs` as part of the file-size split; runs in the same
        // place it ran inline. Validates: Requirement 18.1, 18.2, 18.5, 18.8
        self.render_floating_tabs(ctx);

        // Dialog / overlay cluster (Command Palette, About, Command History,
        // SWAP picker, External-exec confirmation, RESET BARE confirmation,
        // Unsaved-workspace, Catalog Manager). Extracted verbatim to
        // `update_dialogs.rs` as part of the file-size split; it is the LAST
        // statement of `update`, so the early `return`s inside its AllocateDataset
        // arm end the frame exactly as before.
        self.render_overlays(ctx);
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
                self.key_bar_scope.persist_name(),
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

#[cfg(test)]
mod startup_tests {
    use super::{ensure_default_home_catalog, ensure_pom_tab_present};
    use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};
    use crate::tab_manager::TabManager;
    use crate::tab_state::KindTag;
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
        assert_eq!(tabs.tabs()[0].kind.tag(), KindTag::MenuWorkspace);
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
