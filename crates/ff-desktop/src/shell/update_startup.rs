//! # Shell One-Shot Startup
//!
//! `run_startup` -- the first-frame startup phase of the eframe `update()` loop
//! (CLI-file open, session restore, default-catalog seeding, POM presence, and
//! the menus-dir ensure). Extracted verbatim from `update.rs` as part of the
//! Phase 2 task 2.2 file-size split; behaviour and order are unchanged. `update`
//! now calls `self.run_startup()` as its first statement.

use std::path::PathBuf;

use super::update::{ensure_default_home_catalog, ensure_pom_tab_present};
use super::{KeyBarScope, WorkbenchShell};

impl WorkbenchShell {
    /// First-frame startup: runs ONCE (guarded by `self.started`). Opens CLI
    /// files or restores the previous session, seeds the default Home catalog,
    /// guarantees a POM tab, and ensures the menus directory exists. No-op after
    /// the first frame.
    pub(super) fn run_startup(&mut self) {
        if !self.started {
            self.started = true;
            let _ = self.runtime.block_on(self.app.startup());

            let cli_files = std::mem::take(&mut self.cli_files);
            if !cli_files.is_empty() {
                // CLI args take precedence over session restore (Req 5 AC 6).
                for path in cli_files {
                    if let Err(e) = self.shell_open_file(&path) {
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
                // CR-NR-093 Slice 2c.3: capture the persisted split layout (owned)
                // before the session borrow ends; applied AFTER tabs are rebuilt.
                let layout_to_restore = state.layout.clone();
                // Validates: Requirement 6.2 (view-zoom) -- restore global zoom offset.
                if state.global_zoom_offset != 0 {
                    self.zoom = ff_zoom::ZoomState::from_persisted(
                        state.global_zoom_offset,
                        &ff_zoom::ZoomConfig::default(),
                    );
                }
                // Validates: Requirement 12.4 (function-keys-and-history) -- restore PFSHOW state.
                self.key_bar_visible = state.key_bar_visible;
                // CR-CH-046: restore the persisted Key_Label_Bar scope (unknown /
                // absent falls back to Base).
                self.key_bar_scope =
                    KeyBarScope::parse(&state.key_bar_scope).unwrap_or(KeyBarScope::Base);
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
                // CR-NR-093 Slice 2c.3 (Req 14.11): restore the split arrangement
                // AFTER the tabs are reconstructed. The descriptor is identity-
                // free (structure + per-leaf counts), so `restore_layout`
                // distributes the restored store tabs across the saved tree shape
                // and reconciles (Req 14.13). Absent layout -> stays unsplit
                // (Req 14.12), byte-identical to Slice 2a/2b.
                if let Some(layout) = &layout_to_restore {
                    if let Ok(desc) = layout
                        .data
                        .clone()
                        .try_into::<crate::tab_manager::LayoutDescriptor>()
                    {
                        self.tabs.restore_layout(&desc);
                    }
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
                // No session manager -- first launch: open POM tab
                // Validates: Requirement 14.1-14.5 -- create default Home catalog when none exist.
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
    }
}
