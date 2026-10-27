//! # Shell Panel Rendering -- active-tab Context body
//!
//! `render_active_tab_body` (the `match tab.kind` Context dispatcher) plus the
//! search-root collection and Mainframe-DSN resolution free helpers it uses.
//! Split out of `render.rs` (TASK 2.2, pure code movement, no behaviour change).

use eframe::egui;

use crate::tab_state::TabKind;

use super::WorkbenchShell;

/// Render a centred "unavailable in the core build" notice for a parked Context
/// (core-001 strip). The editor, file navigator, and catalog explorer Contexts
/// are unwired from the minimal shell core (see docs/reentry-register.md); the
/// POM options / commands that opened them land here. This is a plain,
/// non-interactive display with no interior Tab stop -- its arm reports
/// `InteriorFocus::none()`.
pub(super) fn render_core_unavailable_notice(ui: &mut egui::Ui, message: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);
        ui.heading(message);
        ui.add_space(8.0);
        ui.label("This capability is parked in the core build and returns later as a plugin.");
    });
}

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
                    // core-001 strip: the Catalog Explorer Context (Virtual
                    // Catalog Manager, POM option 1) is PARKED (see
                    // docs/reentry-register.md). Render an "unavailable in core"
                    // notice; no interior Tab stop -> InteriorFocus::none().
                    render_core_unavailable_notice(
                        ui,
                        "Catalog Explorer is not available in the core build.",
                    );
                    self.apply_interior_focus(
                        ctx,
                        crate::shell::workspace_context::InteriorFocus::none(),
                    );
                }
                TabKind::FileEditor | TabKind::Untitled => {
                    // core-001 strip: the Editor Context (text/file editor) is
                    // PARKED (see docs/reentry-register.md). The editor crates are
                    // no longer wired into `ffwb`, so this arm renders an
                    // "unavailable in core" notice. It reports
                    // `InteriorFocus::none()` EXPLICITLY through the single latch
                    // path (workspace-conformance rule exception 2).
                    render_core_unavailable_notice(
                        ui,
                        "Editor is not available in the core build.",
                    );
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
                    let roots = collect_search_roots(self.active_workspace.as_ref());
                    let mut panel = std::mem::take(&mut self.search_results_panel);
                    panel.search_roots = roots.clone();
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let outcome = std::mem::take(&mut panel.pending_outcome);
                    self.search_results_panel = panel;
                    self.apply_search_outcome(roots, outcome);
                }
                TabKind::FileExplorerPanel => {
                    // core-001 strip: the File Explorer Context (VFS tree browser,
                    // POM option 2) is PARKED (see docs/reentry-register.md).
                    // Render an "unavailable in core" notice; no interior Tab stop
                    // -> InteriorFocus::none().
                    render_core_unavailable_notice(
                        ui,
                        "File Explorer is not available in the core build.",
                    );
                    self.apply_interior_focus(
                        ctx,
                        crate::shell::workspace_context::InteriorFocus::none(),
                    );
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
                TabKind::AliasEditor => {
                    // Validates: command-configurator Req 5 (CR-NR-113 + the
                    // CR-CH-068 ISPF redesign); CR-NR-078 (framework). Owned-panel
                    // swap: render through the trait (reports the FILTER CE
                    // dropdown as the first interior + honours the latch), then
                    // apply the stashed action (ApplyLineCommands / CommandLine).
                    let mut panel = std::mem::take(&mut self.alias_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.alias_editor_panel = panel;
                    self.apply_alias_editor_action(action);
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

/// Collect search root paths from the active workspace.
///
/// core-001 strip: the catalog registry (Native-catalog roots) is parked with
/// the Catalog domain, so the search roots are reduced to the active
/// workspace's roots only. Search Results stays functional over the active
/// workspace without the catalog registry.
///
/// Validates: global-search Requirement 2.4
pub(super) fn collect_search_roots(workspace: Option<&ff_session::WorkspaceState>) -> Vec<String> {
    if let Some(ws) = workspace {
        if !ws.roots.is_empty() {
            return ws
                .roots
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
        }
    }
    Vec::new()
}

// core-001 strip: `open_mainframe_dsn` and `mainframe_dataset_recfm` (catalog
// DSN -> physical path / record format, via ff-dscatalog + ff-files-panel) are
// parked with the catalog domain.
