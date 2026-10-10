//! # Shell Panel Rendering -- active-tab Context body
//!
//! `render_active_tab_body` (the `match tab.kind` Context dispatcher) plus the
//! search-root collection and Mainframe-DSN resolution free helpers it uses.
//! Split out of `render.rs` (TASK 2.2, pure code movement, no behaviour change).

use eframe::egui;

use crate::editor_panel;
use crate::tab_state::TabKind;

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render the ACTIVE tab's Context body (the `match tab.kind` dispatch) into
    /// `ui`. Extracted from `render_central_panel` (CR-CH-035, B045) so the SAME
    /// code renders a docked tab in the primary CentralPanel AND a detached tab
    /// inside its floating viewport: the floating loop temporarily sets the
    /// active index to the detached tab, calls this, and restores it. Behaviour
    /// for the docked path is unchanged.
    ///
    /// Validates: menu-and-statusbar Requirement 18.8 (real detached content)
    pub(super) fn render_active_tab_body(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        {
            // CR-CH-023: reset the Boundary_Policy interior anchors each
            // frame; the active Workspace arm re-populates them if it has
            // interior focus stops. Workspaces that leave them None cause
            // Tab from the command field to go straight to the menu bar.
            self.focus.first_interior_id = None;
            self.focus.last_interior_id = None;
            match self.tabs.active_tab().kind {
                TabKind::FilesPanel => {
                    self.render_body_files_panel(ctx, ui);
                }
                TabKind::FileEditor | TabKind::Untitled => {
                    // CR-NR-078 WF.6 special case (task 8.2): the Editor Context
                    // renders the ACTIVE `TabState` (not a shell-owned panel) and
                    // needs shell-entangled inputs (cmd_engine, exclude_manager,
                    // runtime, the mutable tab) that do not fit the
                    // `ShellServices`-only trait, so it is NOT a `WorkspaceContext`
                    // implementor. Its body is a native egui multiline surface
                    // with its OWN internal focus/caret model and its own
                    // "Command ===>" line; it has NO shell-latched interior Tab
                    // stop. It therefore reports `InteriorFocus::none()`
                    // EXPLICITLY through the single latch path, documenting the
                    // deliberate no-interior case (workspace-conformance rule
                    // exception 2) rather than silently leaving the anchors unset.
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
                    self.apply_interior_focus(
                        ctx,
                        crate::shell::workspace_context::InteriorFocus::none(),
                    );
                }
                TabKind::HelpContext => {
                    // Help Context (CR-NR-097, context-help Req 18.2/18.5):
                    // owned-panel swap through the WorkspaceContext trait, which
                    // reports the Help_Search field as the first interior and
                    // honours the latch on the single path.
                    let mut panel = std::mem::take(&mut self.help.context_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    self.help.context_panel = panel;
                }
                TabKind::ConfigPanel => {
                    // Validates: Requirement 15.1-15.3; CR-NR-078 (framework).
                    // Migrated to the WorkspaceContext trait: owned-panel swap
                    // (Option A) so the single dispatch path reports the focus
                    // contract and honours the latch -- no inline ritual.
                    let mut panel = std::mem::take(&mut self.config_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    self.config_panel = panel;
                }
                TabKind::PluginManager => {
                    // Validates: plugin-manager-ui Requirement 1.1-1.6;
                    // CR-NR-078 WF.6 (framework). Owned-panel swap: render
                    // through the trait, which reports InteriorFocus (Filter
                    // field) and honours the latch on the single path.
                    let mut panel = std::mem::take(&mut self.plugin_manager_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    self.plugin_manager_panel = panel;
                }
                TabKind::EventLog => {
                    // Validates: notification-system Requirement 2.1-2.6;
                    // CR-NR-078 WF.6 (framework). Owned-panel swap: the trait
                    // render applies any Clear-Log request against the shared
                    // queue and reports the level-filter combo as the interior.
                    let mut panel = std::mem::take(&mut self.event_log_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    self.event_log_panel = panel;
                }
                TabKind::ScrmViewer => {
                    // Validates: screen-snapshot-scrm Req 10, 16 (CR-NR-098).
                    // Stage the active Collection onto the viewer BEFORE dispatch
                    // (the trait render only receives ShellServices), then the
                    // owned-panel swap through the framework applies the focus
                    // latch (reports the First button as the interior).
                    let collection = self.scrm.active_collection().cloned();
                    let mut panel = std::mem::take(&mut self.scrm_viewer);
                    panel.set_collection(collection);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    self.scrm_viewer = panel;
                }
                TabKind::SearchResults => {
                    // Validates: global-search Requirement 1.1, 4.1;
                    // CR-NR-078 WF.6 (framework). Compute the search roots and
                    // stage them on the panel BEFORE dispatch (the trait render
                    // only receives ShellServices), owned-panel swap through the
                    // framework (reports the query-field interior + honours the
                    // latch), then apply the stashed outcome shell-side.
                    let roots = collect_search_roots(
                        &self.files_panel.registry,
                        self.active_workspace.as_ref(),
                    );
                    let mut panel = std::mem::take(&mut self.search_results_panel);
                    panel.search_roots = roots.clone();
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let outcome = std::mem::take(&mut panel.pending_outcome);
                    self.search_results_panel = panel;
                    self.apply_search_outcome(roots, outcome);
                }
                TabKind::FileExplorerPanel => {
                    // Unsplit: rendered as a full-window ctx-level panel by
                    // render_central_panel (the is_file_explorer branch), so this
                    // arm is skipped. Split (CR-NR-093 B071): the ctx-level branch
                    // is suppressed, so render the File Explorer HERE, inside this
                    // region's `ui`, keeping the split visible (Req 13.4/14.4).
                    if self.tabs.is_split() {
                        self.nav_explorer_seed();
                        let effects = self.render_nav_explorer_body(ui);
                        self.apply_nav_explorer_effects(ctx, effects);
                    }
                }
                TabKind::MacroLibrary => {
                    // Validates: lua-macro-engine Requirement 12.1-12.8;
                    // CR-NR-078 WF.6 (framework). Owned-panel swap: render
                    // through the trait (reports the Filter field interior +
                    // honours the latch), then apply the stashed action.
                    let mut panel = std::mem::take(&mut self.macro_library_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.macro_library_panel = panel;
                    self.apply_macro_library_action(action);
                }
                TabKind::MenuWorkspace(_) => {
                    self.render_body_menu_workspace(ctx, ui);
                }
                TabKind::ThemeEditor => {
                    // Validates: theme-and-appearance Req 20.1, 20.3-20.9;
                    // CR-NR-078 (framework). Owned-panel swap: render through
                    // the trait (reports first/last interior + honours the
                    // latch), then apply the stashed action.
                    let mut panel = std::mem::take(&mut self.theme_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.theme_editor_panel = panel;
                    self.apply_theme_editor_action(action);
                }
                TabKind::MenusEditor => {
                    // Validates: menu-workspace Req 13.1-13.12 (CR-NR-075);
                    // CR-NR-078 (framework). Owned-panel swap: render through
                    // the trait, then apply the stashed action.
                    let mut panel = std::mem::take(&mut self.menus_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.menus_editor_panel = panel;
                    self.apply_menus_editor_action(action);
                }
                TabKind::KeysEditor => {
                    // Validates: function-keys Req 22 (CR-CH-029); CR-NR-078
                    // (framework). Owned-panel swap: render through the trait,
                    // then apply the stashed action.
                    let mut panel = std::mem::take(&mut self.keys_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.keys_editor_panel = panel;
                    self.apply_keys_editor_action(action);
                }
                TabKind::KindsEditor => {
                    // Validates: workspace-kinds Req 6 (CR-NR-090 B.4); CR-NR-078
                    // (framework). Owned-panel swap: render through the trait,
                    // then apply the stashed action (save/select/new).
                    let mut panel = std::mem::take(&mut self.kinds_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.kinds_editor_panel = panel;
                    self.apply_kinds_editor_action(action);
                }
                TabKind::CommandConfigurator => {
                    // Validates: command-configurator Requirement 2.2-2.6;
                    // CR-NR-078 WF.6 (framework). poll_reload the store first
                    // (mutable), then owned-panel swap: the trait render reads
                    // the settled store from ShellServices, reports the "Add"
                    // button interior + honours the latch, and stashes the
                    // action the shell applies after put-back.
                    self.command_store.poll_reload();
                    let mut panel = std::mem::take(&mut self.command_configurator_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.command_configurator_panel = panel;
                    self.apply_configurator_action(action);
                }
            }
        }
    }

    // The two larger arm helpers (render_body_files_panel for the Catalog
    // Explorer, render_body_menu_workspace for the Home Context / POM and every
    // named menu) now live in `render_body_arms.rs` (Phase 2 task 2.2 file-size
    // split).
}

/// Collect search root paths from the active workspace or mounted Native catalogs.
///
/// Validates: global-search Requirement 2.4
pub(super) fn collect_search_roots(
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
pub(super) fn open_mainframe_dsn(
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

/// Look up a Mainframe dataset's record format (RC.B.8 (c)) in `catalog_name`
/// and map it to the `file.open` `recfm` + `lrecl` params (so the opened
/// Document's RecordFormat can be set and the BRC.3 record-save selection
/// fires). Returns `None` when the dataset has no RECFM/LRECL catalogued or the
/// catalog cannot be read -- the open then falls back to the host/Delimited
/// default (a conservative, byte-identical save).
///
/// The `recfm` string is the ff-vfs record-format kind name
/// (`Fixed`/`Variable`/`Undefined`) that `FileOpenHandler` understands: catalog
/// `F`/`FB` -> `Fixed`, `V`/`VB` -> `Variable`, `U` -> `Undefined`.
///
/// Validates: command-environments Requirement 18.2; document-model Req 11.2
pub(super) fn mainframe_dataset_recfm(
    registry: &crate::catalog_registry::CatalogRegistry,
    catalog_name: &str,
    dsn: &str,
) -> Option<(String, i64)> {
    use ff_dscatalog::dataset::Recfm;
    let records = registry.list_datasets(catalog_name).ok()?;
    let want = dsn.trim().to_uppercase();
    let record = records.into_iter().find(|r| r.dsn.as_str() == want)?;
    let recfm = record.recfm?;
    let lrecl = record.lrecl? as i64;
    let kind = match recfm {
        Recfm::F | Recfm::FB => "Fixed",
        Recfm::V | Recfm::VB => "Variable",
        Recfm::U => "Undefined",
        // `Recfm` is #[non_exhaustive]; any future variant falls back to the
        // host/Delimited default (no record params emitted) by returning None.
        _ => return None,
    };
    Some((kind.to_string(), lrecl))
}
