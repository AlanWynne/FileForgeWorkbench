//! # Frontmost transient-overlay capture + direct COPY (screen-snapshot-scrm Req 27)
//!
//! Two shell methods that bind the transient message/dialog overlays into the
//! existing capture stack:
//!
//! * [`WorkbenchShell::active_dialog_capture`] -- when a transient overlay is
//!   open, build the FRONTMOST overlay's logical [`ScreenModel`] from its OWN
//!   state (never egui) and return it REDACTED (Req 27.2, 27.4, 27.7). The
//!   [`super::screen_provider::WorkbenchShell::active_screen_capture`] seam calls
//!   this FIRST, so SNAPSHOT captures the dialog, not the Context behind it. When
//!   no overlay is open it returns `None` and the Context model flows as before.
//!
//! * [`WorkbenchShell::copy_frontmost_surface`] -- the single direct-COPY path:
//!   place the frontmost surface's redacted PlainText on the clipboard WITHOUT
//!   the SNAPSHOT screenshot machinery (Req 27.3). It reuses the frontmost
//!   dialog model, else falls back to the active Context model, so COPY works on
//!   every surface. The `COPY` front-door verb, the on-dialog Copy buttons, and
//!   the Ctrl+C-while-a-dialog-is-open shortcut ALL route through this one
//!   helper, so there is a single clipboard path and a single redaction point.
//!
//! Redaction is applied ONCE, here, so every caller (SNAPSHOT, COPY, the
//! composite flavors) receives the masked model (Req 27.4). The frontmost
//! precedence mirrors the `modal_open` OR-chain in `shell/update_input.rs`.
//!
//! Validates: screen-snapshot-scrm Requirement 27.2, 27.3, 27.4, 27.5, 27.7.

use ff_screen_model::{Field, ScreenModel};

use super::dialog_screen::DialogScreen;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Build the FRONTMOST open transient overlay's logical screen, REDACTED,
    /// or `None` when no overlay is open. The frontmost precedence mirrors the
    /// `modal_open` OR-chain in `shell/update_input.rs`; the first open overlay
    /// in that order is the one the user sees on top.
    ///
    /// Redaction is applied HERE (not in a renderer) so every caller of this
    /// seam -- SNAPSHOT, the direct COPY, and the composite clipboard flavors --
    /// receives the masked model from the one source (Req 27.4, 27.5).
    ///
    /// Validates: screen-snapshot-scrm Requirement 27.2, 27.4, 27.5, 27.7.
    pub(super) fn active_dialog_capture(&self) -> Option<ScreenModel> {
        let model = self.frontmost_dialog_model()?;
        Some(model.redacted())
    }

    /// Whether ANY transient message/dialog overlay bound by Req 27 is open.
    /// Mirrors the precedence set enumerated in [`Self::frontmost_dialog_model`]
    /// so the Ctrl+C copy shortcut fires for exactly the overlays that have a
    /// capturable model (Req 27.3). The Command Palette is deliberately excluded
    /// (it is not a message/dialog box bound by Req 27).
    pub(super) fn any_dialog_open(&self) -> bool {
        self.show_about
            || self.show_history_list.is_some()
            || self.show_swap_list.is_some()
            || self.show_unsaved_workspace_dialog
            || self.reset_bare_confirm.is_some()
            || self.pending_external.is_some()
            || self.scrm_ui.save_as.is_some()
            || self.scrm_ui.replay_picker.is_some()
    }

    /// The frontmost open overlay's RAW (un-redacted) model, or `None`. Split
    /// from [`Self::active_dialog_capture`] so the single redaction point stays
    /// obvious. Precedence follows the `modal_open` OR-chain order.
    fn frontmost_dialog_model(&self) -> Option<ScreenModel> {
        // Test-only overlay hook: no LIVE core overlay carries a credential
        // secret today, so the redaction contract (Req 27.4/27.5) is proven by
        // injecting a synthetic frontmost dialog model in tests. Compiled out of
        // production entirely.
        #[cfg(test)]
        if let Some(model) = tests::take_test_frontmost() {
            return Some(model);
        }
        if self.show_about {
            return Some(self.about_dialog_model());
        }
        if let Some(entries) = &self.show_history_list {
            return Some(self.history_dialog_model(entries));
        }
        if self.show_swap_list.is_some() {
            return Some(self.swap_dialog_model());
        }
        if self.show_unsaved_workspace_dialog {
            return Some(self.unsaved_dialog_model());
        }
        if let Some(target) = &self.reset_bare_confirm {
            return Some(target.screen_model());
        }
        if let Some(pending) = &self.pending_external {
            return Some(pending.screen_model());
        }
        if let Some(save_as) = &self.scrm_ui.save_as {
            return Some(save_as.screen_model());
        }
        if self.scrm_ui.replay_picker.is_some() {
            return Some(self.replay_picker_dialog_model());
        }
        None
    }

    /// The About dialog's logical screen: the heading, version, description,
    /// credits, copyright, and the Close button -- the SAME text the overlay
    /// renders (Req 27.1).
    fn about_dialog_model(&self) -> ScreenModel {
        let mut model = ScreenModel::new(ff_i18n::t("ff-desktop-about-window-title"));
        model.messages.push(ff_i18n::t("ff-desktop-about-heading"));
        model.messages.push(ff_i18n::t_args(
            "ff-desktop-about-version",
            &ff_i18n::MessageArgs::new().set("version", crate::about_dialog::VERSION),
        ));
        model
            .messages
            .push(crate::about_dialog::DESCRIPTION.to_string());
        model
            .messages
            .push(crate::about_dialog::CREATOR.to_string());
        model
            .messages
            .push(crate::about_dialog::AI_CREDIT.to_string());
        model
            .messages
            .push(crate::about_dialog::COPYRIGHT.to_string());
        model.with_button(ff_i18n::t("ff-desktop-about-close"))
    }

    /// The command-history picker's logical screen: one row per history entry,
    /// plus the Cancel button (Req 27.1).
    fn history_dialog_model(&self, entries: &[String]) -> ScreenModel {
        let mut model = ScreenModel::new(ff_i18n::t("ff-desktop-shell-dialogs-history-title"));
        if entries.is_empty() {
            model
                .messages
                .push(ff_i18n::t("ff-desktop-shell-dialogs-history-empty"));
        } else {
            for entry in entries {
                model = model.with_field(Field::new("History", entry.clone()));
            }
        }
        model.with_button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
    }

    /// The SWAP tab-picker's logical screen: one row per open tab as
    /// "{n}: {title}" (1-based), plus the Cancel button (Req 27.1). Built from
    /// the live tab list exactly as the overlay renders it.
    fn swap_dialog_model(&self) -> ScreenModel {
        let mut model = ScreenModel::new(ff_i18n::t("ff-desktop-shell-dialogs-swap-title"));
        for (i, tab) in self.tabs.tabs().iter().enumerate() {
            let label = tab
                .workspace_name
                .clone()
                .unwrap_or_else(|| tab.title.clone());
            model = model.with_field(Field::new(format!("{}", i + 1), label));
        }
        model.with_button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
    }

    /// The unsaved-workspace-changes dialog's logical screen: the body naming
    /// the active workspace, plus the Save / Discard / Cancel buttons (Req 27.1).
    fn unsaved_dialog_model(&self) -> ScreenModel {
        let ws_name = self
            .active_workspace
            .as_ref()
            .map(|ws| ws.name.clone())
            .unwrap_or_default();
        let mut model = ScreenModel::new(ff_i18n::t("ff-desktop-shell-dialogs-unsaved-title"));
        model.messages.push(ff_i18n::t_args(
            "ff-desktop-shell-dialogs-unsaved-body",
            &ff_i18n::MessageArgs::new().set("name", ws_name),
        ));
        model
            .with_button(ff_i18n::t("ff-desktop-shell-dialogs-unsaved-save"))
            .with_button(ff_i18n::t("ff-desktop-shell-dialogs-unsaved-discard"))
            .with_button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
    }

    /// The SCRM saved-session replay picker's logical screen: one row per saved
    /// `*.ffscrm` session discovered in `scrm_dir()`, plus the Cancel button
    /// (Req 27.1). Built from the SAME enumeration the overlay renders.
    fn replay_picker_dialog_model(&self) -> ScreenModel {
        let entries = self.enumerate_saved_scrm_sessions();
        let mut model = ScreenModel::new("Replay a saved screen collection".to_string());
        if entries.is_empty() {
            model
                .messages
                .push("No saved sessions found. Use CAPTURE STOP to save one first.".to_string());
        } else {
            for entry in &entries {
                model = model.with_field(Field::new("Session", entry.stem.clone()));
            }
        }
        model.with_button("Cancel")
    }

    /// The single direct-COPY path (Req 27.3, REVISED -- owner decision B):
    /// place the FRONTMOST surface's REDACTED content on the clipboard as the
    /// SAME MULTI-FLAVOR COMPOSITE as bare SNAPSHOT -- the live window screenshot
    /// PICTURE + the Unicode-box Markdown TEXT + the CF_HTML composite -- so
    /// pasting the copied dialog into Word brings in BOTH the picture and the
    /// text, exactly like SNAPSHOT (the earlier text-only Copy was insufficient).
    /// The frontmost surface is the open dialog (if any), else the active
    /// Context; a not-capturable Context (documented opt-out) yields an error.
    ///
    /// Because the window screenshot is ASYNC (an egui frame round-trip, Phase
    /// 2B), this drives the SAME async screenshot-then-place flow SNAPSHOT uses:
    /// it renders the two TEXT flavors from the redacted model NOW, stashes them,
    /// and arms `snapshot.screenshot_requested`; the frame loop (`update.rs`)
    /// takes the live window shot -- which naturally includes the dialog on top
    /// -- and assembles+places the composite. The dialog stays OPEN throughout
    /// (arming a screenshot never touches an overlay flag).
    ///
    /// The `COPY` front-door verb, the on-dialog Copy buttons, and the Ctrl+C
    /// shortcut while a dialog is open all call THIS method, which reuses the one
    /// composite builder (`stash_composite_request`) -- there is exactly one
    /// clipboard path and one redaction point (Req 27.3, 27.4).
    ///
    /// Validates: screen-snapshot-scrm Requirement 27.3, 27.4.
    pub(super) fn copy_frontmost_surface(&mut self) {
        let model = match self.active_dialog_capture() {
            Some(m) => m,
            None => match self.active_screen_capture().into_model() {
                Some(m) => m,
                None => {
                    self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-copy-no-clipboard"));
                    return;
                }
            },
        };
        // Reuse the SNAPSHOT composite path: render the flavors from the already
        // -redacted model and arm the async window screenshot. The composite is
        // placed on the Event::Screenshot frame (shell/update.rs), identical to
        // bare SNAPSHOT.
        self.stash_composite_request(&model);
        self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-copy-copied"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::tests_common::make_shell;
    use ff_screen_model::FieldAttributes;
    use std::cell::RefCell;

    thread_local! {
        /// Per-thread injected frontmost dialog model for the redaction tests.
        /// `frontmost_dialog_model` takes-and-clears it so a single test turn
        /// sees exactly one synthetic frontmost overlay.
        static TEST_FRONTMOST: RefCell<Option<ScreenModel>> = const { RefCell::new(None) };
    }

    /// Install a synthetic frontmost overlay model for the next capture call.
    pub(super) fn set_test_frontmost(model: ScreenModel) {
        TEST_FRONTMOST.with(|c| *c.borrow_mut() = Some(model));
    }

    /// Take-and-clear the injected frontmost overlay model (consumed by
    /// `frontmost_dialog_model`).
    pub(super) fn take_test_frontmost() -> Option<ScreenModel> {
        TEST_FRONTMOST.with(|c| c.borrow_mut().take())
    }

    /// A synthetic dialog model with a declared-secret password field.
    fn secret_dialog_model() -> ScreenModel {
        let mut model = ScreenModel::new("Connect to host".to_string());
        model.messages.push("Enter your credentials.".to_string());
        let model = model.with_field(Field::new("Host", "h1")).with_field(
            Field::new("Password", "s3cr3t").with_attrs(FieldAttributes::plain().sensitive()),
        );
        model.with_button("[OK]")
    }

    fn seeded_pom_shell() -> WorkbenchShell {
        let mut shell = make_shell();
        shell.tabs.insert_pom_tab(&shell.runtime);
        shell.ensure_pom_menu_loaded();
        shell
    }

    // Validates: screen-snapshot-scrm Req 27.2 -- with a transient dialog open,
    // the capture is the DIALOG surface (its title/body), not the POM Context
    // behind it.
    #[test]
    fn dialog_open_captures_dialog_not_context_behind() {
        let mut shell = seeded_pom_shell();
        // Open the external-run confirmation through the real state path.
        shell.pending_external = Some(crate::shell::external_adapter::PendingExternal {
            program: "echo".to_string(),
            args: vec!["hi".to_string()],
            working_dir: None,
            mode: ff_command::ExternalMode::Detached,
        });
        let dialog = shell
            .active_dialog_capture()
            .expect("a dialog is open, so capture is Some");
        assert_eq!(
            dialog.title,
            ff_i18n::t("ff-desktop-shell-dialogs-exec-title")
        );
        // The captured dialog carries the command string, which the POM Context
        // model does not.
        assert!(
            dialog.fields.iter().any(|f| f.value == "echo hi"),
            "the dialog model carries the resolved command string"
        );
        // The dialog title differs from the POM Context title behind it, so the
        // frontmost surface -- not the Context -- is captured.
        assert_ne!(
            dialog.title, "Primary Option Menu",
            "the dialog surface is captured, not the POM Context behind it"
        );
        assert!(
            !dialog.title.is_empty(),
            "the dialog has its own title distinct from the Context"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.2 -- with no overlay open,
    // active_dialog_capture returns None (the Context model flows as before).
    #[test]
    fn no_dialog_open_returns_none() {
        let shell = seeded_pom_shell();
        assert!(
            shell.active_dialog_capture().is_none(),
            "no overlay open -> no dialog capture"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.4 -- the redaction point is
    // active_dialog_capture: a synthetic secret dialog would be masked. Here we
    // confirm a non-secret dialog's values survive (no over-masking) and the
    // redaction is applied (the model is cloned through redacted()).
    #[test]
    fn non_secret_dialog_values_are_copyable() {
        let mut shell = seeded_pom_shell();
        shell.pending_external = Some(crate::shell::external_adapter::PendingExternal {
            program: "git".to_string(),
            args: vec!["status".to_string()],
            working_dir: None,
            mode: ff_command::ExternalMode::Detached,
        });
        let dialog = shell.active_dialog_capture().expect("dialog open");
        assert!(
            dialog.fields.iter().any(|f| f.value == "git status"),
            "non-secret command value is copyable (27.5)"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.2, 27.4, 27.7 -- with a secret
    // dialog wired as frontmost, the pure snapshot_text_for_active captures the
    // dialog title + the redaction placeholder and NOT the secret value, from
    // the logical model (no egui grab).
    #[test]
    fn snapshot_text_for_active_captures_frontmost_redacted_dialog() {
        let shell = seeded_pom_shell();
        set_test_frontmost(secret_dialog_model());
        let (text, _fmt) = shell
            .snapshot_text_for_active("")
            .expect("frontmost dialog is capturable");
        assert!(text.contains("Connect to host"), "dialog title captured");
        assert!(
            text.contains(ff_screen_model::REDACTED_PLACEHOLDER),
            "the secret field is redacted in the captured text"
        );
        assert!(
            !text.contains("s3cr3t"),
            "the secret value is never emitted in clear (27.4)"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.3 (REVISED, owner decision B),
    // 27.4 -- COPY places the SAME multi-flavor COMPOSITE as SNAPSHOT: it arms
    // the async window screenshot and stashes the pending composite flavors
    // (picture is captured by the frame loop), rather than a text-only copy.
    // The stashed TEXT flavor is the frontmost surface's REDACTED rendering.
    #[test]
    fn copy_arms_composite_screenshot_with_redacted_text() {
        let mut shell = seeded_pom_shell();

        // COPY must arm the SNAPSHOT screenshot request and stash the pending
        // composite (the picture arrives via the frame loop, exactly like bare
        // SNAPSHOT) -- NOT a text-only copy.
        set_test_frontmost(secret_dialog_model());
        shell.copy_frontmost_surface();
        assert!(
            shell.snapshot.screenshot_requested,
            "COPY arms the composite screenshot path, like SNAPSHOT (27.3)"
        );
        let pending = shell
            .snapshot
            .pending
            .as_ref()
            .expect("COPY stashes the pending composite flavors (27.3)");
        // The stashed text flavor is redacted: the secret is masked and never
        // emitted in clear (27.4).
        assert!(
            pending.text.contains(ff_screen_model::REDACTED_PLACEHOLDER),
            "the composite text flavor masks the secret field (27.4)"
        );
        assert!(
            !pending.text.contains("s3cr3t"),
            "the composite text flavor never emits the secret in clear (27.4)"
        );
        assert!(
            !pending.pre_text.contains("s3cr3t"),
            "the CF_HTML <pre> body never emits the secret in clear (27.4)"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.3 -- the direct COPY path (the one
    // the on-dialog Copy buttons and Ctrl+C both call) leaves the dialog OPEN:
    // copying a message/dialog box must not dismiss it. The copy here is the
    // external-run confirmation; after copy the overlay state is unchanged, and
    // the composite screenshot is armed (the picture includes the dialog on top).
    #[test]
    fn copy_frontmost_surface_leaves_dialog_open() {
        let mut shell = seeded_pom_shell();
        shell.pending_external = Some(crate::shell::external_adapter::PendingExternal {
            program: "echo".to_string(),
            args: vec!["hi".to_string()],
            working_dir: None,
            mode: ff_command::ExternalMode::Detached,
        });
        shell.copy_frontmost_surface();
        assert!(
            shell.pending_external.is_some(),
            "the Copy action must not close the dialog (27.3)"
        );
        assert!(
            shell.snapshot.screenshot_requested,
            "the Copy action arms the composite screenshot path (27.3, owner B)"
        );
        assert!(
            shell.snapshot.pending.is_some(),
            "the Copy action stashes the pending composite flavors (27.3)"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.3 -- the Ctrl+C-while-a-dialog-is-
    // open shortcut is scoped by `any_dialog_open`: true for exactly the
    // transient overlays bound by Req 27 (so the shortcut fires), false when no
    // overlay is up (so normal Ctrl+C elsewhere is never intercepted).
    #[test]
    fn any_dialog_open_tracks_the_overlay_set() {
        let mut shell = seeded_pom_shell();
        assert!(
            !shell.any_dialog_open(),
            "no overlay open -> Ctrl+C copy shortcut is NOT armed"
        );
        shell.show_about = true;
        assert!(shell.any_dialog_open(), "About open -> shortcut armed");
        shell.show_about = false;
        shell.pending_external = Some(crate::shell::external_adapter::PendingExternal {
            program: "echo".to_string(),
            args: vec![],
            working_dir: None,
            mode: ff_command::ExternalMode::Detached,
        });
        assert!(
            shell.any_dialog_open(),
            "external-exec confirmation open -> shortcut armed"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.3, 27.4 -- the Copy path computes
    // the frontmost surface's REDACTED text (the same text a Copy button or
    // Ctrl+C places): a secret field is masked and the placeholder is present.
    #[test]
    fn copy_path_text_is_redacted_for_secret_dialog() {
        let shell = seeded_pom_shell();
        set_test_frontmost(secret_dialog_model());
        let model = shell
            .active_dialog_capture()
            .expect("frontmost secret dialog");
        let text = crate::screen_snapshot::render_snapshot(
            &model,
            ff_screen_model::SnapshotFormat::PlainText,
        );
        assert!(
            text.contains(ff_screen_model::REDACTED_PLACEHOLDER),
            "the Copy path masks the secret field (27.4)"
        );
        assert!(
            !text.contains("s3cr3t"),
            "the Copy path never emits the secret in clear (27.4)"
        );
    }

    // Validates: screen-snapshot-scrm Req 27.4 -- the composite Markdown flavor
    // built from the frontmost secret dialog omits the secret and contains the
    // redaction placeholder.
    #[test]
    fn composite_text_flavor_for_dialog_is_redacted() {
        let shell = seeded_pom_shell();
        set_test_frontmost(secret_dialog_model());
        let captured = shell.active_dialog_capture().expect("frontmost dialog");
        let markdown = crate::screen_snapshot::render_snapshot(
            &captured,
            ff_screen_model::SnapshotFormat::Markdown,
        );
        assert!(
            markdown.contains(ff_screen_model::REDACTED_PLACEHOLDER),
            "Markdown composite flavor carries the redaction placeholder"
        );
        assert!(
            !markdown.contains("s3cr3t"),
            "Markdown composite flavor omits the secret (27.4)"
        );
    }
}
