//! # Shell Panel Body Arms -- Catalog Explorer and Menu_Workspace
//!
//! The two larger `match tab.kind` arm helpers extracted from
//! `render_active_tab_body`: `render_body_files_panel` (Catalog Explorer) and
//! `render_body_menu_workspace` (Home Context / POM and every named menu). Moved
//! out of `render_body.rs` verbatim as part of the Phase 2 task 2.2 file-size
//! split; behaviour, method names, signatures, and visibility are unchanged.

use eframe::egui;

use crate::primary_option_menu;

use super::WorkbenchShell;

// core-001 strip: `render_body_files_panel` (the Catalog Explorer / Files Panel
// body, backed by the parked ff-files-panel / catalog crates) is removed with
// the catalog domain. The `TabKind::FilesPanel` arm now renders an "unavailable
// in core" notice directly in `render_body.rs`.

impl WorkbenchShell {
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
        // CR-NR-109 (localization Req 11.7): compute the per-option DISPLAYED
        // command-column surfaces SHELL-side (where `&self` and the alias store
        // are available), so the menu renderer stays free of shell coupling. Each
        // surface is the active-locale alias surface for the option's ROOT command
        // in the active environment's table, falling back to the raw root when no
        // localised alias exists (Req 11.8). Selection still carries the ROOT
        // (Req 11.9) -- these surfaces are display-only. At `en`/no overlay every
        // surface equals its root, so the column is byte-identical (Req 10.2).
        let command_surfaces = self.menu_option_command_surfaces();
        // CR-NR-111 (localization Req 12.1, 12.3): compute the per-option
        // DISPLAYED key-column mnemonic surfaces SHELL-side alongside the command
        // surfaces, each the active-locale mnemonic for the option's ROOT command,
        // falling back per option to the stored option.key (Req 12.8). Selection
        // still matches on option.key (Req 12.5) -- these are display-only. At
        // `en`/no catalogue key every surface equals option.key (Req 10.2).
        let mnemonic_surfaces = self.menu_option_mnemonic_surfaces();
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
                &command_surfaces,
                &mnemonic_surfaces,
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
