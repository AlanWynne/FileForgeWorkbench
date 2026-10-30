//! # Per-Context ScreenProvider dispatch seam (master-screen-directive Phase 1A)
//!
//! This is the SINGLE place SNAPSHOT and the CAPTURE family obtain the logical
//! screen of the ACTIVE Workspace Context. It replaces the former hard-coded
//! menu-only check in [`super::commands_scrm`] with a dispatch keyed on the
//! Context's [`KindTag`], asking the active Context for its
//! [`ff_screen_model::ScreenModel`] via the existing `ScreenProvider` seam
//! (screen-snapshot-scrm Requirement 2.1, 2.2).
//!
//! The dispatch returns a [`ScreenCapture`], which makes the two legitimate
//! outcomes explicit and symmetric with the focus contract's
//! `InteriorFocus::none()`:
//!
//! * [`ScreenCapture::Model`] -- the Context has a `ScreenProvider`; the
//!   capture engine renders this model.
//! * [`ScreenCapture::OptOut`] -- the Context carries an EXPLICIT, DOCUMENTED
//!   opt-out: it has no logical screen and is deliberately not capturable
//!   (master-screen-directive Requirement 4.3). The opt-out carries a reason so
//!   the "not capturable" response is a declared exception, never a silent gap.
//!
//! PHASE 1A SCOPE (master-screen-directive Requirement 4, screen-snapshot-scrm
//! Requirement 2): this phase wires the SEAM plus the existing Menu_Workspace
//! provider plus the opt-out mechanism. The remaining non-menu Contexts do NOT
//! yet have a provider; until Stream SCRM task C.2 adds them, they flow through
//! this seam's [`ScreenCapture::OptOut`] default with a reason that points at
//! the retrofit matrix (master-screen-directive Requirement 5). Crucially, the
//! not-capturable path now flows through THIS seam/opt-out rather than the old
//! hard-coded `menu_workspace()`-only check, so Stream SCRM only has to flip
//! each Context from the default opt-out to a `Model`.
//!
//! Validates: master-screen-directive Requirement 4.1, 4.2, 4.3, 4.4, 4.5;
//! screen-snapshot-scrm Requirement 2.1, 2.2, 2.3.

use ff_screen_model::ScreenModel;

use super::WorkbenchShell;
use crate::screen_snapshot::MenuScreenChrome;
use crate::tab_state::KindTag;

/// The outcome of asking the active Context for its logical screen.
///
/// Symmetric with the focus contract's `InteriorFocus`: a Context either
/// provides a model or declares a documented opt-out. There is no silent
/// "nothing" -- a Context that renders a logical screen but omits a provider is
/// a defect (master-screen-directive Requirement 4.3), surfaced as the opt-out
/// default only until its provider lands (Stream SCRM C.2).
#[derive(Debug)]
pub(super) enum ScreenCapture {
    /// The active Context provided a logical screen model.
    Model(Box<ScreenModel>),
    /// The active Context is not capturable, with a documented reason.
    OptOut(&'static str),
}

impl ScreenCapture {
    /// The model when the Context is capturable, else `None`. Thin bridge for
    /// the existing `Option`-shaped call sites (SNAPSHOT / CAPTURE SCREEN /
    /// auto-capture) so they keep their current not-capturable handling.
    pub(super) fn into_model(self) -> Option<ScreenModel> {
        match self {
            ScreenCapture::Model(model) => Some(*model),
            ScreenCapture::OptOut(_) => None,
        }
    }
}

/// The documented opt-out reason for the editor Context (FileEditor / Untitled).
/// After Stream SCRM C.2/C.3 every other core Context has a provider, so the
/// editor -- PARKED by the core-001 strip -- is the sole remaining documented
/// opt-out (master-screen-directive Req 4.3). The constant is the single source
/// of the reason text so the dispatch and the conformance harness agree.
pub(super) const PARKED_EDITOR_REASON: &str =
    "editor Context is parked (core-001 strip); provider returns when the editor re-enters";

impl WorkbenchShell {
    /// Ask the ACTIVE Context for its logical screen via the per-Context
    /// `ScreenProvider` dispatch. This is the ONE seam SNAPSHOT and the CAPTURE
    /// engine use to obtain screen content (screen-snapshot-scrm Requirement
    /// 2.2 -- via a provider, never egui).
    ///
    /// Phase 1A: `MenuWorkspace` Contexts dispatch to the existing
    /// `menu_workspace_screen_model` provider (byte-for-byte unchanged); every
    /// other Context flows through the documented opt-out default until its
    /// provider is added (Stream SCRM C.2).
    ///
    /// Validates: master-screen-directive Requirement 4.1, 4.2, 4.3;
    /// screen-snapshot-scrm Requirement 2.1, 2.2, 2.3.
    pub(super) fn active_screen_capture(&self) -> ScreenCapture {
        let tab = self.tabs.active_tab();
        match tab.kind.tag() {
            // The Menu_Workspace provider (POM, Settings, every named menu).
            // Kept EXACTLY as the former behaviour so menu snapshots are
            // unchanged. A MenuWorkspace tab that is not yet seeded has no
            // loaded menu, so it declares the documented opt-out below.
            KindTag::MenuWorkspace => match tab.kind.menu_workspace() {
                Some(mw) => {
                    // Stream SCRM C.1: the POM/menu snapshot is now the FULL
                    // screen (chrome + calendar + PF-keys), built from the
                    // shell's logical state (screen-snapshot-scrm Req 3.9-3.13).
                    let chrome = self.menu_screen_chrome(mw);
                    ScreenCapture::Model(Box::new(
                        crate::screen_snapshot::menu_workspace_screen_model_full(
                            mw,
                            &self.command_text,
                            &chrome,
                        ),
                    ))
                }
                None => ScreenCapture::OptOut("Menu_Workspace not yet seeded (no loaded menu)"),
            },
            // Stream SCRM C.2/C.3: each retrofitted Context provides a model from
            // its logical state (master-screen-directive Req 4.1, 4.2).
            KindTag::ConfigPanel => {
                ScreenCapture::Model(Box::new(self.config_panel_screen_model()))
            }
            KindTag::ThemeEditor => {
                ScreenCapture::Model(Box::new(self.theme_editor_screen_model()))
            }
            KindTag::MenusEditor => {
                ScreenCapture::Model(Box::new(self.menus_editor_screen_model()))
            }
            KindTag::KeysEditor => ScreenCapture::Model(Box::new(self.keys_editor_screen_model())),
            KindTag::KindsEditor => {
                ScreenCapture::Model(Box::new(self.kinds_editor_screen_model()))
            }
            KindTag::AliasEditor => {
                ScreenCapture::Model(Box::new(self.alias_editor_screen_model()))
            }
            KindTag::CommandConfigurator => {
                ScreenCapture::Model(Box::new(self.command_configurator_screen_model()))
            }
            KindTag::FilesPanel => ScreenCapture::Model(Box::new(self.files_panel_screen_model())),
            KindTag::FileExplorerPanel => {
                ScreenCapture::Model(Box::new(self.file_explorer_screen_model()))
            }
            KindTag::SearchResults => {
                ScreenCapture::Model(Box::new(self.search_results_screen_model()))
            }
            KindTag::PluginManager => {
                ScreenCapture::Model(Box::new(self.plugin_manager_screen_model()))
            }
            KindTag::EventLog => ScreenCapture::Model(Box::new(self.event_log_screen_model())),
            KindTag::MacroLibrary => {
                ScreenCapture::Model(Box::new(self.macro_library_screen_model()))
            }
            KindTag::HelpContext => {
                ScreenCapture::Model(Box::new(self.help_context_screen_model()))
            }
            KindTag::ScrmViewer => ScreenCapture::Model(Box::new(self.scrm_viewer_screen_model())),
            // The editor Context (FileEditor / Untitled) is PARKED (core-001
            // strip): no logical screen is rendered while the editor plugin is
            // out of the core build. This is the one DOCUMENTED opt-out
            // (master-screen-directive Req 4.3); it flips to a provider when the
            // editor re-enters.
            KindTag::FileEditor | KindTag::Untitled => ScreenCapture::OptOut(PARKED_EDITOR_REASON),
        }
    }

    /// Build the full-screen chrome for a Menu_Workspace snapshot from the
    /// shell's LOGICAL state (never egui): window title bar, menu bar, tab
    /// header, title line, the calendar (when the menu shows one), and the
    /// PF-key footer (only when the Key_Label_Bar is shown, pfshow on).
    ///
    /// Validates: screen-snapshot-scrm Req 3.9, 3.10, 3.13.
    fn menu_screen_chrome(
        &self,
        mw: &crate::menu_workspace::MenuWorkspaceState,
    ) -> MenuScreenChrome {
        let tab = self.tabs.active_tab();

        // Menu bar: the bar-visible option descriptions, in order.
        let menu_bar: Vec<String> = self
            .resolve_menu_bar_menu()
            .options
            .iter()
            .filter(|o| o.show_in_menu_bar)
            .map(|o| o.description.clone())
            .collect();

        // Tab header tag (e.g. "[POM]") -- the command-derived Menu_Name, same
        // as the on-screen tab label -- and the descriptive title line.
        let tab_header = mw
            .menu_name_label()
            .map(|n| format!("[{n}]"))
            .unwrap_or_else(|| mw.tab_title());
        let title_line = super::title_line_text(tab);

        // Calendar: only when the loaded menu shows it (POM yes, Settings no).
        let calendar_lines = match &mw.menu {
            Some(menu) if menu.show_calendar => {
                use chrono::Datelike;
                let today = chrono::Local::now().date_naive();
                crate::screen_snapshot::calendar_lines(today.year(), today.month(), today.day())
            }
            _ => Vec::new(),
        };

        // PF-key footer: present ONLY when the Key_Label_Bar is shown (Req 3.10).
        let key_bar_lines = if self.key_bar_visible {
            vec![crate::screen_snapshot::key_bar_line(
                self.key_map_resolver.active_key_map(),
                self.key_bar_scope.segment_label(),
                self.key_bar_scope.to_modifier(),
            )]
        } else {
            Vec::new()
        };

        MenuScreenChrome {
            window_title: "FileForge Workbench".to_string(),
            menu_bar,
            tab_header,
            title_line,
            calendar_lines,
            key_bar_lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab_state::KindTag;

    fn seeded_pom_shell() -> WorkbenchShell {
        let mut shell = crate::shell::tests_common::make_shell();
        shell.tabs.insert_pom_tab(&shell.runtime);
        shell.ensure_pom_menu_loaded();
        shell
    }

    // Validates: master-screen-directive Req 4.1, 4.2; screen-snapshot-scrm
    // Req 2.1, 2.2 -- a seeded Menu_Workspace (POM) dispatches to its provider
    // and yields a model through the new seam (not the old hard-coded check).
    #[test]
    fn menu_workspace_dispatches_to_provider() {
        let shell = seeded_pom_shell();
        match shell.active_screen_capture() {
            ScreenCapture::Model(model) => {
                assert!(
                    !model.fields.is_empty() || !model.messages.is_empty(),
                    "POM provider must yield a populated screen model"
                );
            }
            ScreenCapture::OptOut(reason) => {
                panic!("seeded POM must be capturable; got opt-out: {reason}")
            }
        }
    }

    // Validates: master-screen-directive Req 4.1, 4.2 -- after Stream SCRM
    // C.2/C.3 a non-menu Context (Event Log) now has a provider and yields a
    // model through the seam (flipped from the Phase 1A opt-out).
    #[test]
    fn non_menu_context_now_provides_a_model() {
        use ff_session::session_state::WorkspaceKind;
        let mut shell = seeded_pom_shell();
        // Transform the POM to a non-menu Context through the shared
        // `navigate_to` choke point.
        shell.nav_to_kind(WorkspaceKind::EventLog);
        assert_eq!(
            shell.tabs.active_tab().kind.tag(),
            KindTag::EventLog,
            "test precondition: active Context is the Event Log"
        );
        match shell.active_screen_capture() {
            ScreenCapture::Model(model) => {
                assert!(
                    !model.fields.is_empty(),
                    "Event Log provider must yield a populated model"
                );
            }
            ScreenCapture::OptOut(reason) => {
                panic!("Event Log now has a provider; got opt-out: {reason}")
            }
        }
    }

    // Validates: master-screen-directive Req 4.3 -- an unseeded Menu_Workspace
    // declares its own documented opt-out rather than panicking or silently
    // returning nothing.
    #[test]
    fn unseeded_menu_workspace_opts_out_with_reason() {
        let mut shell = crate::shell::tests_common::make_shell();
        // Insert a POM tab WITHOUT seeding its menu.
        shell.tabs.insert_pom_tab(&shell.runtime);
        assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::MenuWorkspace);
        match shell.active_screen_capture() {
            ScreenCapture::OptOut(reason) => {
                assert!(reason.contains("not yet seeded"));
            }
            ScreenCapture::Model(_) => panic!("unseeded Menu_Workspace must opt out"),
        }
    }
}
