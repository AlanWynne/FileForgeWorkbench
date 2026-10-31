//! # Shell Command Dispatch -- SCRM / Snapshot handlers
//!
//! Screen Collection/Replay Manager command handlers (CAPTURE ...). The SNAPSHOT
//! command family lives in the sibling `commands_snapshot.rs` and the
//! key-label-bar PFSHOW command in `commands_pfshow.rs` (both split out to keep
//! this file under the 400-line limit; pure code movement, no behaviour change).
//! All methods are on `WorkbenchShell` and keep their `pub(super)` visibility.

use super::helpers::*;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Build the logical [`ScreenModel`] of the ACTIVE Context via its
    /// `ScreenProvider`, or `None` when the active Context declares the
    /// documented opt-out (master-screen-directive Requirement 4.3).
    ///
    /// This delegates to the per-Context [`super::screen_provider`] dispatch
    /// seam -- the SINGLE place SNAPSHOT and the CAPTURE engine obtain screen
    /// content (screen-snapshot-scrm Requirement 2.2 -- via a provider, never
    /// egui). Phase 1A: `MenuWorkspace` Contexts yield a model exactly as
    /// before; every other Context flows through the opt-out default until its
    /// provider lands (Stream SCRM C.2). The former hard-coded menu-only check
    /// has been replaced by that seam.
    ///
    /// Validates: master-screen-directive Requirement 4.1, 4.2, 4.3;
    /// screen-snapshot-scrm Requirement 2.1, 2.2, 2.3.
    pub(super) fn active_screen_model(&self) -> Option<ff_screen_model::ScreenModel> {
        self.active_screen_capture().into_model()
    }

    /// A human label for the active Context, used as a capture's screen name.
    pub(super) fn active_screen_name(&self) -> Option<String> {
        let tab = self.tabs.active_tab();
        tab.kind.menu_workspace().and_then(|mw| mw.menu_title())
    }

    /// Handle the CAPTURE command family (CR-NR-098, Wave 2). `rest` is the
    /// case-preserving argument after `CAPTURE` (e.g. `START Repro`, `SCREEN`,
    /// `STATUS`). Routes to the SCRM session; every outcome is reported in the
    /// command area.
    ///
    /// Validates: screen-snapshot-scrm Requirement 7, 8, 9.1, 11.1.
    pub(super) fn handle_capture(&mut self, rest: &str) {
        let rest = rest.trim();
        let (verb, arg) = rest
            .split_once(char::is_whitespace)
            .map(|(v, a)| (v, a.trim()))
            .unwrap_or((rest, ""));
        let msg = match verb.to_ascii_uppercase().as_str() {
            "START" => {
                let m = self.scrm.start(arg, "user");
                // START also enables automatic capture for the session.
                self.scrm.set_auto_capture(true);
                m
            }
            "STOP" => return self.handle_capture_stop(),
            "SCREEN" => match self.active_screen_model() {
                Some(model) => {
                    let name = self.active_screen_name();
                    let seq = self.scrm.capture(model, name);
                    ff_i18n::t_args(
                        "ff-desktop-shell-msg-scrm-captured-screen",
                        &ff_i18n::MessageArgs::new().set("seq", seq.to_string()),
                    )
                }
                None => ff_i18n::t("ff-desktop-shell-msg-scrm-screen-not-capturable"),
            },
            "STATUS" => self.scrm.status(),
            "LIST" => self.scrm.list(),
            "PURGE" => self.scrm.purge(),
            "REPLAY" => return self.handle_capture_replay(arg),
            "EXPORT" => self.handle_capture_export(arg),
            "EVIDENCE" => self.handle_capture_evidence(arg),
            "SAVE" => self.handle_capture_save(arg),
            "LOAD" | "OPEN" => self.handle_capture_load(arg),
            "" => self.scrm.status(),
            other => ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-unknown-subcommand",
                &ff_i18n::MessageArgs::new().set("subcommand", other.to_string()),
            ),
        };
        self.open_error = Some(msg);
    }

    /// Handle `CAPTURE STOP` (Stream SCRM Phase 2C FEAT-002): stop the session
    /// (Req 7.6) and, when a non-empty active Collection exists, open the Save-As
    /// modal defaulting the save location to `scrm_dir()` (Req 24.1). With no
    /// collection (or an empty one) no modal opens and the status message stands.
    ///
    /// The modal only CAPTURES the user's choice; the actual persistence runs
    /// through the EXISTING `CAPTURE SAVE` / `CAPTURE EXPORT` front door when the
    /// overlay resolves to SAVE (Req 24.4). CANCEL writes nothing (Req 24.5).
    ///
    /// Validates: screen-snapshot-scrm Requirement 7.6, 24.1, 24.5.
    fn handle_capture_stop(&mut self) {
        let msg = self.scrm.stop();
        let default_name = self
            .scrm
            .active_collection()
            .filter(|c| !c.is_empty())
            .map(|c| c.name.clone());
        if let Some(name) = default_name {
            let dir = self.scrm_dir();
            self.scrm_ui.save_as = Some(crate::scrm_save_as_dialog::SaveAsState::new(name, dir));
        }
        self.open_error = Some(msg);
    }

    /// Handle `CAPTURE EXPORT <FORMAT> [file]`: render the active Collection to
    /// text/markdown/html and write it to a file under `scrm_dir()` (or an
    /// absolute path). Returns a status string.
    ///
    /// Validates: screen-snapshot-scrm Requirement 12.1, 12.2, 12.3, 11.1.
    fn handle_capture_export(&self, arg: &str) -> String {
        use ff_scrm::{export_html, export_markdown, export_text, Masking, MaskingRules};
        let Some(collection) = self.scrm.active_collection() else {
            return "No active screen collection to export.".to_string();
        };
        let (fmt, file_arg) = arg
            .split_once(char::is_whitespace)
            .map(|(f, r)| (f, r.trim()))
            .unwrap_or((arg, ""));
        // `EXPORT PDF PROTECTED [owner-pw] [file]` is a distinct sub-command.
        if fmt.eq_ignore_ascii_case("PDF") {
            let rest = file_arg.trim();
            let (second, tail) = rest
                .split_once(char::is_whitespace)
                .map(|(a, b)| (a, b.trim()))
                .unwrap_or((rest, ""));
            if second.eq_ignore_ascii_case("PROTECTED") {
                return self.handle_capture_export_pdf_protected(tail);
            }
        }
        let rules = MaskingRules::default();
        // PDF renders to bytes; the text formats render to a String. Resolve the
        // bytes-to-write and extension uniformly.
        let stem = sanitise_stem(&collection.name);
        let (bytes, ext): (Vec<u8>, &str) = match fmt.to_ascii_uppercase().as_str() {
            "TEXT" | "" => (
                export_text(collection, Masking::Off, &rules).into_bytes(),
                "txt",
            ),
            "MD" | "MARKDOWN" => (
                export_markdown(collection, Masking::Off, &rules).into_bytes(),
                "md",
            ),
            "HTML" => (
                export_html(collection, Masking::Off, &rules).into_bytes(),
                "html",
            ),
            "PDF" => (ff_scrm::export_pdf(collection, Masking::Off, &rules), "pdf"),
            other => return format!("CAPTURE EXPORT: unknown format '{other}'."),
        };
        let path = self.resolve_scrm_path(file_arg, &stem, ext);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&path, bytes) {
            Ok(()) => format!("Collection exported to {}.", path.display()),
            Err(e) => format!("CAPTURE EXPORT failed: {e}"),
        }
    }

    /// Handle `CAPTURE EXPORT PDF PROTECTED [owner-password] [file]`: write a
    /// copy-enabled, edit-locked, tamper-evident PDF (owner-password encryption +
    /// permission flags + embedded content hash). When no owner password is
    /// supplied, a default is used and reported so the user can change
    /// permissions later.
    ///
    /// Validates: screen-snapshot-scrm Requirement 20.1, 20.2, 20.4, 20.5.
    fn handle_capture_export_pdf_protected(&self, arg: &str) -> String {
        use ff_scrm::{export_pdf_protected, Masking, MaskingRules, ProtectionOptions};
        let Some(collection) = self.scrm.active_collection() else {
            return "No active screen collection to export.".to_string();
        };
        let (owner_pw, file_arg) = arg
            .split_once(char::is_whitespace)
            .map(|(a, b)| (a.trim(), b.trim()))
            .unwrap_or((arg.trim(), ""));
        let owner_pw = if owner_pw.is_empty() {
            "ffwb-owner".to_string()
        } else {
            owner_pw.to_string()
        };
        let options = ProtectionOptions {
            owner_password: owner_pw,
            user_password: None,
        };
        let rules = MaskingRules::default();
        let bytes = match export_pdf_protected(collection, Masking::Off, &rules, &options) {
            Ok(b) => b,
            Err(e) => return format!("CAPTURE EXPORT PDF PROTECTED failed: {e}"),
        };
        let stem = format!("{}-protected", sanitise_stem(&collection.name));
        let path = self.resolve_scrm_path(file_arg, &stem, "pdf");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&path, bytes) {
            Ok(()) => format!(
                "Protected PDF written to {} (copy-enabled, edit-locked).",
                path.display()
            ),
            Err(e) => format!("CAPTURE EXPORT PDF PROTECTED failed: {e}"),
        }
    }

    /// Handle `CAPTURE EVIDENCE [test-case-id] [PASS|FAIL] [file]`: build an
    /// evidence package around the active Collection (user/date/session/count +
    /// optional test-case id + pass/fail + content hash) and write it as JSON.
    ///
    /// Validates: screen-snapshot-scrm Requirement 14.1-14.4, 20.4, 20.5.
    fn handle_capture_evidence(&self, arg: &str) -> String {
        use ff_scrm::{EvidencePackage, EvidenceStatus};
        let Some(collection) = self.scrm.active_collection() else {
            return "No active screen collection for an evidence package.".to_string();
        };
        // Parse optional tokens: <test-case-id> <PASS|FAIL> <file>. Any may be
        // omitted; a bare EVIDENCE records NotAssessed with a default file name.
        let mut test_case_id: Option<String> = None;
        let mut status = EvidenceStatus::NotAssessed;
        let mut file_arg = "";
        for token in arg.split_whitespace() {
            match token.to_ascii_uppercase().as_str() {
                "PASS" => status = EvidenceStatus::Pass,
                "FAIL" => status = EvidenceStatus::Fail,
                _ if test_case_id.is_none() => test_case_id = Some(token.to_string()),
                _ => file_arg = token,
            }
        }
        let package = EvidencePackage::build(collection, "user", test_case_id, status);
        let json = match package.to_json() {
            Ok(j) => j,
            Err(e) => return format!("CAPTURE EVIDENCE failed to serialise: {e}"),
        };
        let stem = format!("{}-evidence", sanitise_stem(&collection.name));
        let path = self.resolve_scrm_path(file_arg, &stem, "json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&path, json) {
            Ok(()) => format!("Evidence package written to {}.", path.display()),
            Err(e) => format!("CAPTURE EVIDENCE failed: {e}"),
        }
    }

    /// Handle `CAPTURE SAVE [file]`: write the active Collection to the native
    /// zip archive under `scrm_dir()` (or an absolute path).
    ///
    /// Validates: screen-snapshot-scrm Requirement 11.1, 12.5.
    fn handle_capture_save(&self, arg: &str) -> String {
        let Some(collection) = self.scrm.active_collection() else {
            return "No active screen collection to save.".to_string();
        };
        let stem = sanitise_stem(&collection.name);
        let path = self.resolve_scrm_path(arg, &stem, "ffscrm");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match ff_scrm::save_archive(collection, &path) {
            Ok(()) => format!("Collection saved to {}.", path.display()),
            Err(e) => format!("CAPTURE SAVE failed: {e}"),
        }
    }

    /// Handle `CAPTURE LOAD|OPEN <file>`: read a native zip archive into the
    /// active session, replacing any active Collection.
    ///
    /// Validates: screen-snapshot-scrm Requirement 11.1, 12.5.
    fn handle_capture_load(&mut self, arg: &str) -> String {
        if arg.trim().is_empty() {
            return "CAPTURE LOAD requires a file name.".to_string();
        }
        let path = self.resolve_scrm_path(arg, "collection", "ffscrm");
        match ff_scrm::load_archive(&path) {
            Ok(collection) => {
                let n = collection.len();
                self.scrm.load(collection);
                format!("Loaded collection from {} ({n} captures).", path.display())
            }
            Err(e) => format!("CAPTURE LOAD failed: {e}"),
        }
    }

    /// Handle `CAPTURE REPLAY [file]` (Stream SCRM Phase 2C FEAT-003):
    ///
    /// - With a NON-EMPTY `arg`: LOAD the saved `.ffscrm` archive into the active
    ///   session (reusing the single `handle_capture_load` path -> `ff_scrm::load_archive`)
    ///   and, on success, open the ScrmViewer Context on the loaded Collection
    ///   (Req 10.11). A load failure reports the error and does NOT open the
    ///   viewer.
    /// - With an EMPTY `arg`: open the saved-session PICKER modal (Req 10.12).
    ///   The overlay (`shell/update_dialogs_scrm.rs`) enumerates `scrm_dir()` for
    ///   `*.ffscrm` and, on selection, dispatches `CAPTURE REPLAY <path>` through
    ///   the single front door -- which re-enters this method's NON-EMPTY branch.
    ///   When a Collection is already active, the picker still opens so the user
    ///   can choose a saved session rather than only the in-memory one.
    ///
    /// All paths share the one load seam and the one dispatch seam -- no second
    /// dispatcher, no bespoke intercept (framework-conformance, Req 11).
    ///
    /// Validates: screen-snapshot-scrm Requirement 10.11, 10.12, 16.1.
    pub(super) fn handle_capture_replay(&mut self, arg: &str) {
        let arg = arg.trim();
        if arg.is_empty() {
            self.scrm_ui.replay_picker = Some(crate::scrm_session_picker::ReplayPickerState::new());
            self.open_error = None;
            return;
        }
        let msg = self.handle_capture_load(arg);
        if msg.starts_with("Loaded collection") {
            // Load succeeded: open the viewer on the freshly loaded Collection.
            self.open_scrm_viewer();
        } else {
            // Load failed: surface the error and do NOT open the viewer (which
            // could otherwise show a stale previously-active Collection).
            self.open_error = Some(msg);
        }
    }

    /// Open (navigate the current tab to) the SCRM Replay viewer Context
    /// (CAPTURE REPLAY). Reports when there is nothing to replay.
    ///
    /// Validates: screen-snapshot-scrm Requirement 10.1, 16.1.
    pub(super) fn open_scrm_viewer(&mut self) {
        if !self.scrm.is_active() {
            self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-scrm-no-collection-replay"));
            return;
        }
        self.nav_to_kind(ff_session::session_state::WorkspaceKind::ScrmViewer);
        self.open_error = None;
    }

    /// Perform one automatic capture of the active Context when auto-capture is
    /// enabled and the Context is capturable. Called from the Context-transition
    /// hook (`reconstruct_context`). A no-op when auto-capture is off or the
    /// Context has no provider, so it never interrupts navigation.
    ///
    /// CR-NR-098 Wave 2R (Req 9.10/9.11/9.12): when the transition was CAUSED by
    /// a command, the triggering command (recorded at the single dispatch seam)
    /// is read-and-cleared here and attached to the capture. A timed-interval /
    /// rule capture finds `last_dispatched` already `None`, so the triggering
    /// command is recorded as explicitly absent (Req 9.12). The command is never
    /// gathered via a bespoke intercept -- only from the seam-recorded value.
    ///
    /// Validates: screen-snapshot-scrm Requirement 9.1, 9.5, 9.10, 9.11, 9.12.
    pub(super) fn auto_capture_active_context(&mut self) {
        if !self.scrm.auto_capture_enabled() {
            // Always clear so a stale command never attaches to a LATER capture.
            self.scrm_ui.last_dispatched = None;
            return;
        }
        let triggering = self
            .scrm_ui
            .last_dispatched
            .take()
            .map(|(command, target_kind)| ff_scrm::TriggeringCommand {
                command,
                target_kind,
            });
        if let Some(model) = self.active_screen_model() {
            let name = self.active_screen_name();
            self.scrm.capture_with_command(model, name, triggering);
        }
    }
}
