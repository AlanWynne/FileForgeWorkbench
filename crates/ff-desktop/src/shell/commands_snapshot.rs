//! # Shell Command Dispatch -- SNAPSHOT command family (Phase 2B, CR-NR-114)
//!
//! The SNAPSHOT command:
//!
//! - Bare `SNAPSHOT` (no argument) places the COMPOSITE clipboard payload
//!   directly (Req 4.1): the PNG picture AND the MARKDOWN-FENCED text, as
//!   co-equal flavors (Req 23.4). There is NO format picker (Req 4.7 RETIRED).
//! - `SNAPSHOT <FORMAT>` with an EXPLICIT format produces that ONE format
//!   directly (Req 4.8): text formats (TEXT/ANSI/MARKDOWN/MD/HTML/YAML/AI) and
//!   PNG are written to a single-format file; PDF reuses the Collection PDF
//!   path. ANSI is available ONLY here, never as a clipboard flavor (Req 23.5e).
//!
//! The multi-flavor clipboard backend lives in [`crate::clipboard_backend`];
//! the PNG rasteriser and the PDF reuse live in `ff_screen_model` and
//! `ff_scrm` respectively, so this module is only the shell wiring.
//!
//! Validates: screen-snapshot-scrm Requirement 4.1-4.8, 6.1-6.3, 2.3, 23.3,
//! 23.4, 23.5c, 23.5d, 23.5e.

use eframe::egui;

use super::helpers::*;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Produce the snapshot text for the ACTIVE Context in the requested format,
    /// or an error message when the format is unknown or the Context is not
    /// capturable (has no `ScreenProvider`).
    ///
    /// This is the pure, side-effect-free core of the SNAPSHOT TEXT path (no
    /// clipboard, no status mutation), so it is unit-testable. `arg` is the raw
    /// (case-preserving) argument after `SNAPSHOT` (empty selects PlainText).
    ///
    /// Phase 2B: `handle_snapshot` no longer calls this directly (it branches to
    /// the multi-flavor clipboard / PDF paths), but it remains the pure,
    /// test-only core that the SNAPSHOT/capture tests assert against, so it is
    /// retained as test-supporting. The `allow` keeps a non-test build clean
    /// since production reaches the same renderers via `render_snapshot`.
    ///
    /// Validates: screen-snapshot-scrm Requirement 2.3, 4.1-4.6.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn snapshot_text_for_active(
        &self,
        arg: &str,
    ) -> Result<(String, ff_screen_model::SnapshotFormat), String> {
        let format = ff_screen_model::SnapshotFormat::parse_arg(arg)
            .ok_or_else(|| format!("SNAPSHOT: unknown format '{}'", arg.trim()))?;
        match self.active_screen_capture() {
            super::screen_provider::ScreenCapture::Model(model) => Ok((
                crate::screen_snapshot::render_snapshot(&model, format),
                format,
            )),
            // Req 2.3 / master-screen-directive Req 4.3: a Context that declares
            // the documented opt-out is not capturable; surface the opt-out
            // reason so the "cannot be captured yet" response names WHY.
            super::screen_provider::ScreenCapture::OptOut(reason) => Err(format!(
                "SNAPSHOT: the active workspace cannot be captured yet ({reason})."
            )),
        }
    }

    /// Handle the SNAPSHOT command.
    ///
    /// - Bare `SNAPSHOT` (no format) places the COMPOSITE clipboard payload
    ///   directly: the PNG picture AND the Markdown-fenced text (Req 4.1, 4.7
    ///   RETIRED). There is no format picker.
    /// - `SNAPSHOT <FORMAT>` with an explicit format produces that ONE format
    ///   directly (Req 4.8): a text format or PNG is written to a single-format
    ///   file; PDF reuses the Collection PDF path (Req 23.3). ANSI is produced
    ///   here as a file only -- never a clipboard flavor (Req 23.5e).
    ///
    /// Validates: screen-snapshot-scrm Requirement 4.1-4.8, 6.1-6.3, 2.3, 23.3,
    /// 23.4, 23.5e.
    pub(super) fn handle_snapshot(&mut self, arg: &str) {
        // Req 4.1 / 4.7-RETIRED: bare SNAPSHOT places the composite clipboard
        // payload directly (no picker).
        if arg.trim().is_empty() {
            self.snapshot_composite_to_clipboard();
            return;
        }
        let format = match ff_screen_model::SnapshotFormat::parse_arg(arg) {
            Some(f) => f,
            None => {
                // Req 6.3: unknown format is a clear non-fatal error.
                self.open_error = Some(format!("SNAPSHOT: unknown format '{}'", arg.trim()));
                return;
            }
        };
        // Req 4.8: an explicit format produces that single format directly.
        if format == ff_screen_model::SnapshotFormat::Pdf {
            self.snapshot_to_pdf();
            return;
        }
        self.snapshot_single_format_to_file(format);
    }

    /// REQUEST phase of the two-phase async bare `SNAPSHOT` (CR-NR-114,
    /// FEAT-002). Bare SNAPSHOT no longer rasterises the text into the clipboard
    /// picture synchronously; instead it resolves the active Context's logical
    /// screen, renders the two TEXT flavors from it, stashes them in
    /// [`super::state_groups::PendingSnapshot`], and asks the egui frame loop for
    /// a LIVE WINDOW screenshot (egui 0.33's window screenshot is ASYNC). The
    /// frame loop (`update.rs`) issues the `ViewportCommand::Screenshot` and, on
    /// the later `Event::Screenshot` frame, assembles the composite picture
    /// (the REAL window RGBA, not a synthetic text render) + text + CF_HTML and
    /// places all three flavors on the clipboard in one transaction. This method
    /// therefore does NOT build a payload or touch the clipboard; it only sets up
    /// the pending request (non-blocking -- no spin-wait on the screenshot).
    ///
    /// Both text flavors are rendered HERE (in the command phase, where the
    /// `ScreenProvider` seam lives) so the frame loop stays egui-only and never
    /// touches the screen model:
    /// - `text` (the `CF_UNICODETEXT` flavor) is the MARKDOWN-FENCED rendering so
    ///   a Markdown-aware target renders the box-drawing monospaced and a
    ///   plain-text target still pastes readable text; ANSI is never on the
    ///   clipboard (Req 23.5e).
    /// - `pre_text` (the CF_HTML `<pre>` body, built in the frame loop) is the
    ///   plain Unicode box-drawn screen body (`RenderOptions::default()` frames
    ///   with the U+2500 family), so the HTML composite shows the real box-drawn
    ///   screen text, not the fenced Markdown source.
    ///
    /// Validates: screen-snapshot-scrm Requirement 4.1, 23.4, 23.5c, 23.5d,
    /// 23.5e, 6.1-6.3.
    fn snapshot_composite_to_clipboard(&mut self) {
        let model = match self.active_screen_capture() {
            super::screen_provider::ScreenCapture::Model(m) => m,
            super::screen_provider::ScreenCapture::OptOut(reason) => {
                self.open_error = Some(format!(
                    "SNAPSHOT: the active workspace cannot be captured yet ({reason})."
                ));
                return;
            }
        };
        // Req 23.4: the composite TEXT flavor is the Markdown-fenced rendering
        // (NOT plain text, NOT ANSI) so box-drawing stays monospaced/aligned in
        // a Markdown-aware target and readable in a plain-text target.
        let text = crate::screen_snapshot::render_snapshot(
            &model,
            ff_screen_model::SnapshotFormat::Markdown,
        );
        // Req 23.5c: the CF_HTML `<pre>` body is the plain Unicode box-drawn
        // screen (the default options frame with the U+2500 family), rendered
        // here so the frame loop never touches the model.
        let pre_text = ff_screen_model::render(
            &model,
            ff_screen_model::SnapshotFormat::PlainText,
            ff_screen_model::RenderOptions::default(),
        );
        // Stash the pre-rendered flavors and ask the frame loop for a live window
        // screenshot. The payload + clipboard placement happen on the
        // Event::Screenshot frame (see `shell/update.rs`).
        self.snapshot.pending = Some(super::state_groups::PendingSnapshot { text, pre_text });
        self.snapshot.screenshot_requested = true;
        self.open_error = None;
    }

    /// ASSEMBLY phase of the two-phase async bare `SNAPSHOT` (CR-NR-114,
    /// FEAT-002). Called by the frame loop (`shell/update.rs`) once the async
    /// `egui::Event::Screenshot` has delivered the live window image and a
    /// [`super::state_groups::PendingSnapshot`] is waiting. Adapts the real
    /// window RGBA to a [`ff_screen_model::RasterImage`] via the sole
    /// egui-touching adapter [`crate::screenshot::color_image_to_raster`], builds
    /// the composite payload (the captured picture, the Markdown-fenced text, and
    /// the CF_HTML image+`<pre>` fragment), and places all three flavors on the
    /// clipboard in one transaction. Sets the success / partial / failure status
    /// via the existing SNAPSHOT i18n keys.
    ///
    /// The picture is the REAL window grab (Req 23.4, 23.5d), not a synthetic
    /// text render; the CF_HTML composite carries the same PNG plus the plain
    /// Unicode box-drawn `<pre>` body (Req 23.5c). ANSI is never a flavor here
    /// (Req 23.5e).
    ///
    /// Validates: screen-snapshot-scrm Requirement 4.1, 23.4, 23.5c, 23.5d,
    /// 23.5e.
    pub(super) fn place_snapshot_composite(
        &mut self,
        image: &egui::ColorImage,
        pending: super::state_groups::PendingSnapshot,
    ) {
        use crate::clipboard_backend::SnapshotClipboard;
        let raster = crate::screenshot::color_image_to_raster(image);
        let mut payload =
            crate::clipboard_backend::SnapshotPayload::from_window_capture(raster, pending.text);
        payload.html = Some(crate::clipboard_html::build_cf_html(
            &payload.png,
            &pending.pre_text,
        ));
        let fmt = snapshot_format_label(ff_screen_model::SnapshotFormat::Markdown);
        let mut backend = crate::clipboard_backend::OsClipboard;
        self.open_error = Some(match backend.place(&payload) {
            Ok(offered) if offered.both() => ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-both-copied",
                &ff_i18n::MessageArgs::new().set("format", fmt),
            ),
            // Only one flavor made it (e.g. a backend that could not offer the
            // image): still report success for the flavor that landed.
            Ok(_) => ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-copied",
                &ff_i18n::MessageArgs::new().set("format", fmt),
            ),
            Err(_) => ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-no-clipboard",
                &ff_i18n::MessageArgs::new().set("format", fmt),
            ),
        });
    }

    /// Produce a single EXPLICIT format directly to a file (Req 4.8): a text
    /// format (TEXT/ANSI/MARKDOWN/MD/HTML/YAML/AI) writes its rendered text; PNG
    /// writes the rasterised picture. The file lands under `scrm_dir()` with the
    /// format's natural extension, matching the `SNAPSHOT PDF` / CAPTURE EXPORT
    /// conventions. ANSI is reachable only here, never on the clipboard (Req
    /// 23.5e).
    ///
    /// Validates: screen-snapshot-scrm Requirement 4.2-4.6, 4.8, 23.2, 23.5e.
    fn snapshot_single_format_to_file(&mut self, format: ff_screen_model::SnapshotFormat) {
        let model = match self.active_screen_capture() {
            super::screen_provider::ScreenCapture::Model(m) => m,
            super::screen_provider::ScreenCapture::OptOut(reason) => {
                self.open_error = Some(format!(
                    "SNAPSHOT: the active workspace cannot be captured yet ({reason})."
                ));
                return;
            }
        };
        // Bytes + extension per format: PNG rasterises; the text formats render
        // to a String (ANSI included -- a file target, not the clipboard).
        let (bytes, ext): (Vec<u8>, &str) = match format {
            ff_screen_model::SnapshotFormat::Png => {
                let image = ff_screen_model::rasterise_screen(&model);
                (ff_screen_model::encode_png(&image), "png")
            }
            other => {
                let text = crate::screen_snapshot::render_snapshot(&model, other);
                (text.into_bytes(), snapshot_file_extension(other))
            }
        };
        let name = self
            .active_screen_name()
            .unwrap_or_else(|| "snapshot".to_string());
        let stem = sanitise_stem(&name);
        let path = self.resolve_scrm_path("", &stem, ext);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let fmt = snapshot_format_label(format);
        self.open_error = Some(match std::fs::write(&path, bytes) {
            Ok(()) => ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-written",
                &ff_i18n::MessageArgs::new()
                    .set("format", fmt)
                    .set("path", path.display().to_string()),
            ),
            Err(e) => ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-pdf-failed",
                &ff_i18n::MessageArgs::new().set("error", e.to_string()),
            ),
        });
    }

    /// Write a single-screen SNAPSHOT PDF of the active Context, reusing the
    /// `ff-scrm` Collection PDF path (selectable text, Req 23.3, 12.6).
    ///
    /// Validates: screen-snapshot-scrm Requirement 23.3, 12.6.
    fn snapshot_to_pdf(&mut self) {
        let model = match self.active_screen_capture() {
            super::screen_provider::ScreenCapture::Model(m) => m,
            super::screen_provider::ScreenCapture::OptOut(reason) => {
                self.open_error = Some(format!(
                    "SNAPSHOT: the active workspace cannot be captured yet ({reason})."
                ));
                return;
            }
        };
        let name = self
            .active_screen_name()
            .unwrap_or_else(|| "snapshot".to_string());
        let stem = sanitise_stem(&name);
        let path = self.resolve_scrm_path("", &stem, "pdf");
        let bytes = ff_scrm::export_screen_pdf(*model, &name, chrono::Utc::now());
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        self.open_error = Some(match std::fs::write(&path, bytes) {
            Ok(()) => ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-pdf-written",
                &ff_i18n::MessageArgs::new().set("path", path.display().to_string()),
            ),
            Err(e) => ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-pdf-failed",
                &ff_i18n::MessageArgs::new().set("error", e.to_string()),
            ),
        });
    }
}
