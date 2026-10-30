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

/// The reason attached to the Phase 1A default opt-out for a non-menu Context
/// whose `ScreenProvider` has not been added yet. Tracked by the
/// master-screen-directive Requirement 5 retrofit matrix; Stream SCRM task C.2
/// replaces this default with a real provider per Context.
pub(super) const PENDING_PROVIDER_REASON: &str =
    "no ScreenProvider yet (tracked by the master-screen-directive retrofit matrix, Req 5)";

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
                Some(mw) => ScreenCapture::Model(Box::new(
                    crate::screen_snapshot::menu_workspace_screen_model(mw, &self.command_text),
                )),
                None => ScreenCapture::OptOut("Menu_Workspace not yet seeded (no loaded menu)"),
            },
            // Every other Context: no provider yet in Phase 1A. The opt-out is
            // the explicit, documented path (master-screen-directive Req 4.3);
            // Stream SCRM C.2 flips each of these to a real provider.
            _ => ScreenCapture::OptOut(PENDING_PROVIDER_REASON),
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

    // Validates: master-screen-directive Req 4.3, 4.4 -- a non-menu Context with
    // no provider yet flows through the DOCUMENTED opt-out seam (not the old
    // menu-only hard check). Stream SCRM C.2 will flip this to a model.
    #[test]
    fn non_menu_context_reports_documented_opt_out() {
        use ff_session::session_state::WorkspaceKind;
        let mut shell = seeded_pom_shell();
        // Transform the POM to a non-menu Context (Event Log is read-only, no
        // provider in Phase 1A) through the shared `navigate_to` choke point.
        shell.nav_to_kind(WorkspaceKind::EventLog);
        assert_eq!(
            shell.tabs.active_tab().kind.tag(),
            KindTag::EventLog,
            "test precondition: active Context is the Event Log"
        );
        match shell.active_screen_capture() {
            ScreenCapture::OptOut(reason) => {
                assert_eq!(reason, PENDING_PROVIDER_REASON);
            }
            ScreenCapture::Model(_) => {
                panic!("Event Log has no provider in Phase 1A; expected the opt-out")
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
