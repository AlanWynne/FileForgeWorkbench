//! # Shell Command Dispatch -- SCRM / Snapshot / PFSHOW handlers
//!
//! Screen Snapshot Service and Screen Collection/Replay Manager command
//! handlers (SNAPSHOT, CAPTURE ...), plus the key-label-bar PFSHOW command.
//! Split out of `commands.rs` (TASK 2.2, pure code movement, no behaviour
//! change). All methods are on `WorkbenchShell` and keep their exact
//! signatures and `pub(super)` visibility.

use super::helpers::*;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Produce the snapshot text for the ACTIVE Context in the requested format,
    /// or an error message when the format is unknown or the Context is not
    /// capturable (has no `ScreenProvider`).
    ///
    /// This is the pure, side-effect-free core of the SNAPSHOT command (no
    /// clipboard, no status mutation), so it is unit-testable. `arg` is the raw
    /// (case-preserving) argument after `SNAPSHOT` (empty selects the default
    /// PlainText format).
    ///
    /// Validates: screen-snapshot-scrm Requirement 2.3, 4.1-4.6.
    pub(super) fn snapshot_text_for_active(
        &self,
        arg: &str,
    ) -> Result<(String, ff_screen_model::SnapshotFormat), String> {
        let format = ff_screen_model::SnapshotFormat::parse_arg(arg)
            .ok_or_else(|| format!("SNAPSHOT: unknown format '{}'", arg.trim()))?;
        match self.active_screen_model() {
            Some(model) => Ok((
                crate::screen_snapshot::render_snapshot(&model, format),
                format,
            )),
            None => Err("SNAPSHOT: the active workspace cannot be captured yet.".to_string()),
        }
    }

    /// Build the logical [`ScreenModel`] of the ACTIVE Context via its
    /// `ScreenProvider`, or `None` when the active Context is not capturable.
    ///
    /// Wave 1/2: the capturable Contexts are Menu_Workspaces (POM and other
    /// menus). Later waves add providers for more Contexts. This is the single
    /// place the SNAPSHOT command and the CAPTURE engine obtain screen content
    /// (Requirement 2.2 -- via a provider, never egui).
    ///
    /// Validates: screen-snapshot-scrm Requirement 2.1, 2.2, 2.3.
    pub(super) fn active_screen_model(&self) -> Option<ff_screen_model::ScreenModel> {
        let tab = self.tabs.active_tab();
        tab.kind
            .menu_workspace()
            .map(|mw| crate::screen_snapshot::menu_workspace_screen_model(mw, &self.command_text))
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
            "STOP" => self.scrm.stop(),
            "SCREEN" => match self.active_screen_model() {
                Some(model) => {
                    let name = self.active_screen_name();
                    let seq = self.scrm.capture(model, name);
                    format!("Captured screen {seq}.")
                }
                None => "CAPTURE SCREEN: the active workspace cannot be captured yet.".to_string(),
            },
            "STATUS" => self.scrm.status(),
            "LIST" => self.scrm.list(),
            "PURGE" => self.scrm.purge(),
            "REPLAY" => {
                self.open_scrm_viewer();
                return;
            }
            "EXPORT" => self.handle_capture_export(arg),
            "EVIDENCE" => self.handle_capture_evidence(arg),
            "SAVE" => self.handle_capture_save(arg),
            "LOAD" | "OPEN" => self.handle_capture_load(arg),
            "" => self.scrm.status(),
            other => format!("CAPTURE: unknown subcommand '{other}'."),
        };
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

    /// Open (navigate the current tab to) the SCRM Replay viewer Context
    /// (CAPTURE REPLAY). Reports when there is nothing to replay.
    ///
    /// Validates: screen-snapshot-scrm Requirement 10.1, 16.1.
    pub(super) fn open_scrm_viewer(&mut self) {
        if !self.scrm.is_active() {
            self.open_error = Some("No screen collection to replay.".to_string());
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
    /// Validates: screen-snapshot-scrm Requirement 9.1, 9.5.
    pub(super) fn auto_capture_active_context(&mut self) {
        if !self.scrm.auto_capture_enabled() {
            return;
        }
        if let Some(model) = self.active_screen_model() {
            let name = self.active_screen_name();
            self.scrm.capture(model, name);
        }
    }

    /// Handle the SNAPSHOT command: render the active Context to selectable text
    /// and copy it to the OS clipboard, reporting the outcome in the command
    /// area.
    ///
    /// Validates: screen-snapshot-scrm Requirement 4.1-4.6, 6.1, 6.2, 6.3, 2.3.
    pub(super) fn handle_snapshot(&mut self, arg: &str) {
        match self.snapshot_text_for_active(arg) {
            Ok((text, format)) => {
                // Req 6.1: copy the rendered text to the OS clipboard.
                let copied = match arboard::Clipboard::new() {
                    Ok(mut cb) => cb.set_text(&text).is_ok(),
                    Err(_) => false,
                };
                // Req 6.2: confirm to the user, including the format.
                let fmt = snapshot_format_label(format);
                self.open_error = Some(if copied {
                    format!("Snapshot copied to clipboard ({fmt}).")
                } else {
                    format!("Snapshot rendered ({fmt}) but the clipboard was unavailable.")
                });
            }
            // Req 6.3 / 2.3: unknown format or not-capturable Context.
            Err(message) => {
                self.open_error = Some(message);
            }
        }
    }

    /// Handle the PFSHOW command (CR-CH-046).
    ///
    /// `arg` is the uppercased, trimmed argument after `PFSHOW` (empty for the
    /// bare form). Bare PFSHOW cycles the five-state mode
    /// `Off -> Base -> Shift -> Ctrl -> Alt -> Off`; the scope keywords jump
    /// directly to a scope and show the bar; `ON`/`OFF` toggle visibility; an
    /// unrecognised argument leaves the mode unchanged and sets a non-fatal error.
    ///
    /// Validates: Requirement 12.1, 12.2, 12.3, 12.6, 12.7, 12.8, 12.9, 12.13
    pub(super) fn handle_pfshow(&mut self, arg: &str) {
        use super::KeyBarScope;
        match arg {
            // Bare PFSHOW: advance the five-state cycle.
            "" => {
                if !self.key_bar_visible {
                    // Off -> Base
                    self.key_bar_visible = true;
                    self.key_bar_scope = KeyBarScope::Base;
                } else {
                    match self.key_bar_scope {
                        KeyBarScope::Base => self.key_bar_scope = KeyBarScope::Shift,
                        KeyBarScope::Shift => self.key_bar_scope = KeyBarScope::Ctrl,
                        KeyBarScope::Ctrl => self.key_bar_scope = KeyBarScope::Alt,
                        // Alt -> Off (bar hidden; scope retained for PFSHOW ON).
                        KeyBarScope::Alt => self.key_bar_visible = false,
                    }
                }
                self.open_error = None;
            }
            // PFSHOW ON: show at the current-or-retained scope (12.1).
            "ON" => {
                self.key_bar_visible = true;
                self.open_error = None;
            }
            // PFSHOW OFF: hide (12.2). Scope retained.
            "OFF" => {
                self.key_bar_visible = false;
                self.open_error = None;
            }
            // Explicit scope jump (12.9).
            _ => {
                if let Some(scope) = KeyBarScope::parse(arg) {
                    self.key_bar_scope = scope;
                    self.key_bar_visible = true;
                    self.open_error = None;
                } else {
                    // Unknown argument (12.13): leave the mode unchanged, surface a
                    // clear non-fatal error.
                    self.open_error = Some(format!(
                        "Unknown PFSHOW argument '{arg}' -- expected ON, OFF, BASE, SHIFT, CTRL, or ALT."
                    ));
                }
            }
        }
    }
}
