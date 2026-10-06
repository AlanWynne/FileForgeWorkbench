//! # Shell Panel Body Arms -- Catalog Explorer and Menu_Workspace
//!
//! The two larger `match tab.kind` arm helpers extracted from
//! `render_active_tab_body`: `render_body_files_panel` (Catalog Explorer) and
//! `render_body_menu_workspace` (Home Context / POM and every named menu). Moved
//! out of `render_body.rs` verbatim as part of the Phase 2 task 2.2 file-size
//! split; behaviour, method names, signatures, and visibility are unchanged.

use eframe::egui;

use crate::catalog_manager_dialog::{self, NewCatalogForm};
use crate::dataset_alloc_dialog::{self};
use crate::files_panel;
use crate::primary_option_menu;

use super::render_body::open_mainframe_dsn;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render the Catalog Explorer (Files Panel) Context body and apply its
    /// panel actions. Extracted verbatim from the `TabKind::FilesPanel` arm of
    /// `render_active_tab_body` (TASK 2.2, pure code movement, no behaviour
    /// change) to keep that dispatcher under the 400-line rule.
    pub(super) fn render_body_files_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Validates: Requirement 1.1, 1.7
        let action = files_panel::render(ui, &mut self.files_panel);
        match action {
            files_panel::FilesPanelAction::ReturnToPom => {
                self.pending_return_to_pom = true;
            }
            files_panel::FilesPanelAction::NewCatalog => {
                if matches!(self.files_panel.dialog, files_panel::FilesDialogState::None) {
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
                if matches!(self.files_panel.dialog, files_panel::FilesDialogState::None) {
                    if let Some(cat) = self.files_panel.registry.get_by_name(&name) {
                        let form = catalog_manager_dialog::EditCatalogForm::from_catalog(cat);
                        self.files_panel.dialog = files_panel::FilesDialogState::EditCatalog(form);
                    }
                }
            }
            files_panel::FilesPanelAction::DeleteCatalog(name) => {
                // Req 4.3 - open Delete Catalog confirmation dialog
                if matches!(self.files_panel.dialog, files_panel::FilesDialogState::None) {
                    if let Some(cat) = self.files_panel.registry.get_by_name(&name) {
                        let confirm =
                            catalog_manager_dialog::DeleteCatalogConfirm::from_catalog(cat);
                        self.files_panel.dialog =
                            files_panel::FilesDialogState::DeleteCatalog(confirm);
                    }
                }
            }

            files_panel::FilesPanelAction::AllocateDataset(catalog_name) => {
                // Req 5.1 - open Allocate Dataset dialog
                // Req 13.2 - record which catalog opened the dialog
                if matches!(self.files_panel.dialog, files_panel::FilesDialogState::None) {
                    self.files_panel.pending_alloc_catalog = Some(catalog_name.clone());
                    // Req 5.7 — pre-populate Dataset Name with catalog HLQ if set
                    let form = self
                        .files_panel
                        .registry
                        .get_by_name(&catalog_name)
                        .and_then(|c| c.default_hlq.as_deref())
                        .map(dataset_alloc_dialog::AllocDatasetForm::with_hlq)
                        .unwrap_or_default();
                    self.files_panel.dialog = files_panel::FilesDialogState::AllocateDataset(form);
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
                    .map(|c| c.catalog_type == crate::catalog_registry::CatalogType::Mainframe)
                    .unwrap_or(false);
                if is_mainframe {
                    let catalog_name = self
                        .files_panel
                        .content
                        .selected_catalog
                        .clone()
                        .unwrap_or_default();
                    match open_mainframe_dsn(&self.files_panel.registry, &catalog_name, &dsn) {
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
        // CR-NR-078 WF.6 special case (task 8.2): the Files Panel
        // (Catalog Explorer Context) has its OWN internal
        // "Command ===>" field and a bespoke Tab redirect
        // (files_panel_cmd -> first catalog node via
        // `tree_focus_requested`, B024/Req 20.1) handled in
        // `render_central_panel`, NOT the shell command-field ->
        // first-interior latch. It has no shell-latched interior Tab
        // stop, so it reports `InteriorFocus::none()` EXPLICITLY
        // through the single latch path (workspace-conformance rule
        // exception 2) rather than silently leaving the anchors unset.
        self.apply_interior_focus(ctx, crate::shell::workspace_context::InteriorFocus::none());
    }

    /// Render the Menu_Workspace (Home Context / POM and every named menu)
    /// Context body. Extracted verbatim from the `TabKind::MenuWorkspace` arm of
    /// `render_active_tab_body` (TASK 2.2, pure code movement, no behaviour
    /// change) to keep that dispatcher under the 400-line rule.
    ///
    /// Validates: menu-workspace Requirement 2.1-2.6, 2.1a-2.1c, 18.2, 18.4
    pub(super) fn render_body_menu_workspace(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Validates: menu-workspace Requirement 2.1-2.6, 2.1a-2.1c, 18.2, 18.4.
        // After CR-NR-082 Slice 1 the Home Context (POM) is a MenuWorkspace tab,
        // so this single arm renders both the POM and every named menu via the
        // SHARED menu renderer. ensure_pom_menu_loaded is a no-op unless the tab
        // is Home (it seeds pom.toml / the barebones Recovery_Baseline).
        self.ensure_pom_menu_loaded();
        // Resolve calendar colours and month offset before borrowing
        // the active tab mutably (menu_calendar_colours borrows &self).
        let menu_cal = self.menu_colours();
        let calendar_offset = self.pom_calendar_offset;
        let active_idx = self.tabs.active_index();
        let mut calendar_nav = None;
        let mut first_interior = None;
        let mut last_interior = None;
        if let Some(mw) = self
            .tabs
            .tabs_mut()
            .get_mut(active_idx)
            .and_then(|t| t.kind.menu_workspace_mut())
        {
            mw.poll_reload();
            let result = crate::menu_workspace::render::render_menu_workspace(
                mw,
                ui,
                calendar_offset,
                menu_cal,
            );
            if let Some(option) = result.selected {
                self.pending_menu_option = Some(option);
            }
            calendar_nav = result.calendar_nav;
            first_interior = result.first_interior_id;
            last_interior = result.last_interior_id;
            // CR-CH-028: record the focused option for the
            // Cursor_Context (Req 12.2).
            self.focused_menu_option = result.focused_option;
        }
        // CR-NR-078: MenuWorkspace routes its focus contract
        // through the shared framework helper (single latch path).
        self.apply_interior_focus(
            ctx,
            crate::shell::workspace_context::InteriorFocus {
                first: first_interior,
                last: last_interior,
            },
        );
        if let Some(nav) = calendar_nav {
            match nav {
                primary_option_menu::CalendarNav::Prev => self.pom_calendar_offset -= 1,
                primary_option_menu::CalendarNav::Next => self.pom_calendar_offset += 1,
            }
        }
    }
}
