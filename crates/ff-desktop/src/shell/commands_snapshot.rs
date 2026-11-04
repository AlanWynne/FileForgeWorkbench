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
use super::message_routing::SnapshotPlaceOutcome;
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
        self.stash_composite_request(&model);
        self.open_error = None;
    }

    /// Render the two composite TEXT flavors from `model` and stash them as the
    /// pending async request, arming the live window screenshot. SHARED by bare
    /// `SNAPSHOT` and the on-dialog Copy affordance (Req 27.3, owner decision B)
    /// so both place the IDENTICAL composite (picture + Markdown text + CF_HTML)
    /// through the ONE assembly path in `shell/update.rs` -- there is no second
    /// clipboard path. The caller supplies the already-resolved (and, for a
    /// dialog, already-REDACTED) logical model, so redaction stays a single
    /// point.
    ///
    /// Validates: screen-snapshot-scrm Requirement 4.1, 23.4, 23.5c, 23.5d,
    /// 27.3.
    pub(super) fn stash_composite_request(&mut self, model: &ff_screen_model::ScreenModel) {
        // Req 23.4: the composite TEXT flavor is the Markdown-fenced rendering
        // (NOT plain text, NOT ANSI) so box-drawing stays monospaced/aligned in
        // a Markdown-aware target and readable in a plain-text target.
        let text = crate::screen_snapshot::render_snapshot(
            model,
            ff_screen_model::SnapshotFormat::Markdown,
        );
        // Req 23.5c: the CF_HTML monospace body is the plain Unicode box-drawn
        // screen (the default options frame with the U+2500 family), rendered
        // here so the frame loop never touches the model.
        let pre_text = ff_screen_model::render(
            model,
            ff_screen_model::SnapshotFormat::PlainText,
            ff_screen_model::RenderOptions::default(),
        );
        // Req 28.2: the property/value HTML tables appended after the box in the
        // CF_HTML flavor, derived from the SAME tabular content the Markdown
        // flavor uses (`markdown_sections`, else `fields`/`tables`). Empty for a
        // box-only model (Req 28.7), leaving the CF_HTML composite unchanged.
        let tables_html = ff_screen_model::render_tables_html(model);
        // Stash the pre-rendered flavors and ask the frame loop for a live window
        // screenshot. The payload + clipboard placement happen on the
        // Event::Screenshot frame (see `shell/update.rs`). The picture naturally
        // includes any open dialog on top, which is correct for the dialog Copy.
        self.snapshot.pending = Some(super::state_groups::PendingSnapshot {
            text,
            pre_text,
            tables_html,
        });
        self.snapshot.screenshot_requested = true;
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
    /// Unicode box-drawn monospace body AND, after it, the property/value HTML
    /// tables (Req 23.5c, 28.2, 28.4). ANSI is never a flavor here (Req 23.5e).
    ///
    /// Validates: screen-snapshot-scrm Requirement 4.1, 23.4, 23.5c, 23.5d,
    /// 23.5e, 28.2.
    pub(super) fn place_snapshot_composite(
        &mut self,
        image: &egui::ColorImage,
        pending: super::state_groups::PendingSnapshot,
        pixels_per_point: f32,
    ) {
        use crate::clipboard_backend::SnapshotClipboard;
        let raster = crate::screenshot::color_image_to_raster(image);
        // Req 23.2a: when a dialog is the frontmost captured surface, CROP the
        // full-window screenshot to the dialog's on-screen rectangle so only the
        // dialog -- not the window behind it -- is in the picture. A full-window
        // SNAPSHOT with no dialog open is left uncropped.
        let raster = self.crop_capture_to_dialog(raster, pixels_per_point);
        let mut payload =
            crate::clipboard_backend::SnapshotPayload::from_window_capture(raster, pending.text);
        payload.html = Some(crate::clipboard_html::build_cf_html(
            &payload.png,
            &pending.pre_text,
            &pending.tables_html,
        ));
        let fmt = snapshot_format_label(ff_screen_model::SnapshotFormat::Markdown);
        let mut backend = crate::clipboard_backend::OsClipboard;
        let outcome = match backend.place(&payload) {
            Ok(offered) if offered.both() => SnapshotPlaceOutcome::Both,
            // Only one flavor made it (e.g. a backend that could not offer the
            // image): still report success for the flavor that landed.
            Ok(_) => SnapshotPlaceOutcome::Partial,
            Err(_) => SnapshotPlaceOutcome::Failed,
        };
        // Req 7.28: the confirmation goes to the Message_Line band (live, and
        // captured by the NEXT snapshot); only a clipboard failure is an error.
        // This runs on the async frame-loop path AFTER the action's clear, so it
        // survives until the next action.
        self.post_snapshot_place_outcome(outcome, fmt);
    }

    /// Crop `raster` (the full-window screenshot) to the frontmost dialog's
    /// rectangle when a dialog is the frontmost captured surface (Req 23.2a),
    /// otherwise return it unchanged. The decision is: a dialog is open
    /// (`any_dialog_open`) AND its on-screen rect was captured this frame
    /// (`snapshot.dialog_rect`). The points rect is converted to a physical-pixel
    /// `PixelRect` via `pixels_per_point` and clamped by `ff_screen_model::crop`.
    ///
    /// Validates: screen-snapshot-scrm Requirement 23.2a.
    fn crop_capture_to_dialog(
        &self,
        raster: ff_screen_model::RasterImage,
        pixels_per_point: f32,
    ) -> ff_screen_model::RasterImage {
        if !self.any_dialog_open() {
            return raster;
        }
        let rect = match self.snapshot.dialog_rect {
            Some(r) => r,
            None => return raster,
        };
        let pixel_rect = points_rect_to_pixels(rect, pixels_per_point);
        ff_screen_model::crop(&raster, pixel_rect)
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
        // Req 7.28: success is informational (Message_Line), failure an error.
        match std::fs::write(&path, bytes) {
            Ok(()) => self.post_info(ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-written",
                &ff_i18n::MessageArgs::new()
                    .set("format", fmt)
                    .set("path", path.display().to_string()),
            )),
            Err(e) => {
                self.open_error = Some(ff_i18n::t_args(
                    "ff-desktop-shell-msg-scrm-snapshot-pdf-failed",
                    &ff_i18n::MessageArgs::new().set("error", e.to_string()),
                ));
            }
        }
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
        // Req 7.28: success is informational (Message_Line), failure an error.
        match std::fs::write(&path, bytes) {
            Ok(()) => self.post_info(ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-pdf-written",
                &ff_i18n::MessageArgs::new().set("path", path.display().to_string()),
            )),
            Err(e) => {
                self.open_error = Some(ff_i18n::t_args(
                    "ff-desktop-shell-msg-scrm-snapshot-pdf-failed",
                    &ff_i18n::MessageArgs::new().set("error", e.to_string()),
                ));
            }
        }
    }
}

/// Convert an egui on-screen rectangle (in POINTS) to a physical-pixel
/// [`ff_screen_model::PixelRect`] using `pixels_per_point` (Req 23.2a). The
/// origin and extent are scaled and rounded to whole pixels; a negative origin
/// (a dialog partly off the top/left) is clamped to zero so the rect is a valid
/// image coordinate (the crop itself further clamps to the image bounds).
fn points_rect_to_pixels(rect: egui::Rect, pixels_per_point: f32) -> ff_screen_model::PixelRect {
    let ppp = if pixels_per_point > 0.0 {
        pixels_per_point
    } else {
        1.0
    };
    let x = (rect.min.x * ppp).round().max(0.0) as u32;
    let y = (rect.min.y * ppp).round().max(0.0) as u32;
    let width = (rect.width() * ppp).round().max(0.0) as u32;
    let height = (rect.height() * ppp).round().max(0.0) as u32;
    ff_screen_model::PixelRect {
        x,
        y,
        width,
        height,
    }
}

#[cfg(test)]
mod dialog_crop_tests {
    use super::*;
    use crate::shell::tests_common::make_shell;
    use ff_screen_model::RasterImage;

    /// A solid WxH image whose pixels encode (col, row) in R/G, so a crop can be
    /// checked against known coordinates.
    fn coded_image(w: u32, h: u32) -> RasterImage {
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                rgba.extend_from_slice(&[x as u8, y as u8, 0, 0xFF]);
            }
        }
        RasterImage {
            width: w,
            height: h,
            rgba,
        }
    }

    // Validates: screen-snapshot-scrm Req 23.2a -- a points rect is scaled to
    // physical pixels by pixels-per-point and rounded to whole pixels.
    #[test]
    fn points_rect_scales_by_pixels_per_point() {
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(30.0, 40.0));
        let px = points_rect_to_pixels(rect, 2.0);
        assert_eq!(px.x, 20);
        assert_eq!(px.y, 40);
        assert_eq!(px.width, 60);
        assert_eq!(px.height, 80);
    }

    // Validates: screen-snapshot-scrm Req 23.2a -- a negative (partly off-screen)
    // origin is clamped to zero so the pixel rect is a valid image coordinate.
    #[test]
    fn points_rect_clamps_negative_origin() {
        let rect = egui::Rect::from_min_size(egui::pos2(-5.0, -3.0), egui::vec2(10.0, 10.0));
        let px = points_rect_to_pixels(rect, 1.0);
        assert_eq!(px.x, 0);
        assert_eq!(px.y, 0);
    }

    // Validates: screen-snapshot-scrm Req 23.2a -- when a dialog is frontmost and
    // its rect is captured, the full-window capture is CROPPED to the dialog
    // rect (the result has the dialog-rect pixel dimensions, NOT the full
    // framebuffer size). Uses `show_about` as the open-dialog seam and a stubbed
    // rect, so no real screenshot is needed.
    #[test]
    fn crop_uses_dialog_rect_when_dialog_frontmost() {
        let mut shell = make_shell();
        let full = coded_image(100, 80);
        // A dialog is frontmost (seam) and its on-screen rect was captured.
        shell.show_about = true;
        shell.snapshot.dialog_rect = Some(egui::Rect::from_min_size(
            egui::pos2(10.0, 20.0),
            egui::vec2(30.0, 25.0),
        ));
        let cropped = shell.crop_capture_to_dialog(full, 1.0);
        assert_eq!(cropped.width, 30, "cropped to the dialog rect width");
        assert_eq!(cropped.height, 25, "cropped to the dialog rect height");
        assert_ne!(
            (cropped.width, cropped.height),
            (100, 80),
            "the capture is NOT the full framebuffer size"
        );
        // Top-left pixel of the crop is source (10, 20): R=10, G=20.
        assert_eq!(&cropped.rgba[0..4], &[10, 20, 0, 0xFF]);
    }

    // Validates: screen-snapshot-scrm Req 23.2a -- with NO dialog open the
    // capture is left UNCROPPED (whole window), even if a stale rect lingers.
    #[test]
    fn no_dialog_open_leaves_capture_uncropped() {
        let mut shell = make_shell();
        let full = coded_image(100, 80);
        // No dialog open; a stale rect must not trigger a crop.
        shell.snapshot.dialog_rect = Some(egui::Rect::from_min_size(
            egui::pos2(5.0, 5.0),
            egui::vec2(10.0, 10.0),
        ));
        assert!(!shell.any_dialog_open(), "precondition: no dialog open");
        let out = shell.crop_capture_to_dialog(full.clone(), 1.0);
        assert_eq!(
            (out.width, out.height),
            (100, 80),
            "a full-window SNAPSHOT with no dialog is uncropped"
        );
        assert_eq!(out, full, "the capture is returned unchanged");
    }
}

/// End-to-end flavor assertions over a Config-like `ScreenModel` fixture
/// (screen-snapshot-scrm Req 28, CR-CH-079, task 2F.4): each clipboard TEXT
/// flavor carries BOTH the box-drawn layout AND the property/value tables, and
/// the PNG picture flavor is unchanged.
#[cfg(test)]
mod dual_representation_flavor_tests {
    use ff_screen_model::{MarkdownSection, ScreenModel, SnapshotFormat, Table};

    /// A Config-like model: a box-drawn body (`body_lines`) PLUS per-group
    /// `markdown_sections` (a heading-less intro table + one `[Group]` heading +
    /// Property/Value table), mirroring what the Config `ScreenProvider`
    /// produces.
    fn config_fixture() -> ScreenModel {
        let mut m = ScreenModel::new("Configuration");
        m.body_lines = vec![
            "Filter:                              :".to_string(),
            "Unsaved edits                        : no".to_string(),
            "[Editor]                             : expanded".to_string(),
            "    editor.tab_size                  : 4".to_string(),
        ];
        m.markdown_sections = vec![
            MarkdownSection {
                heading: None,
                table: Table {
                    caption: None,
                    headers: vec!["Property".to_string(), "Value".to_string()],
                    rows: vec![vec!["Unsaved edits".to_string(), "no".to_string()]],
                },
            },
            MarkdownSection {
                heading: Some("[Editor]  (expanded)".to_string()),
                table: Table {
                    caption: None,
                    headers: vec!["Property".to_string(), "Value".to_string()],
                    rows: vec![vec!["editor.tab_size".to_string(), "4".to_string()]],
                },
            },
        ];
        m
    }

    // Validates: screen-snapshot-scrm Requirement 28.1 -- the `CF_UNICODETEXT`
    // flavor (the Markdown-fenced `render_snapshot(model, Markdown)` text) carries
    // BOTH the box block AND the Markdown `| Property | Value |` tables, box
    // before tables.
    #[test]
    fn cf_unicode_text_flavor_has_box_and_markdown_tables() {
        let text =
            crate::screen_snapshot::render_snapshot(&config_fixture(), SnapshotFormat::Markdown);
        let box_idx = text.find('\u{250C}').expect("box frame present");
        let table_idx = text
            .find("| Property | Value |")
            .expect("markdown property/value table present");
        assert!(box_idx < table_idx, "box before tables (Req 28.4)");
        assert!(text.contains("### [Editor]  (expanded)"));
        assert!(text.contains("| editor.tab_size | 4 |"));
    }

    // Validates: screen-snapshot-scrm Requirement 28.2 -- the `CF_HTML` flavor
    // (built by `build_cf_html` from the PNG, the plain box body, and the
    // HTML-table emitter output) carries the base64 image, the monospace box
    // `<span>` block, AND HTML `<table>` tables.
    #[test]
    fn cf_html_flavor_has_image_box_and_html_tables() {
        let model = config_fixture();
        let png = [0x89u8, b'P', b'N', b'G'];
        let pre_text = ff_screen_model::render(
            &model,
            SnapshotFormat::PlainText,
            ff_screen_model::RenderOptions::default(),
        );
        let tables_html = ff_screen_model::render_tables_html(&model);
        let html = crate::clipboard_html::build_cf_html(&png, &pre_text, &tables_html);

        assert!(
            html.contains("data:image/png;base64,"),
            "the PNG image is inlined (Req 28.2)"
        );
        assert!(
            html.contains("<span style=\"font-family:'Consolas','Courier New',monospace\">"),
            "the monospace box span block is present (Req 28.2, 28.5)"
        );
        assert!(
            html.contains("<table class=\"ffwb-table\">"),
            "HTML property/value tables are present (Req 28.2)"
        );
        assert!(html.contains("<td>editor.tab_size</td><td>4</td>"));

        // Order: image, then box span, then table (Req 28.4).
        let img_idx = html.find("data:image/png;base64,").unwrap();
        let span_idx = html
            .find("<span style=\"font-family:'Consolas','Courier New',monospace\">")
            .unwrap();
        let table_idx = html.find("<table class=\"ffwb-table\">").unwrap();
        assert!(img_idx < span_idx && span_idx < table_idx);
    }

    // Validates: screen-snapshot-scrm Requirement 28.3 -- the PNG picture flavor
    // is unchanged by this requirement: the picture bytes are produced by the
    // unchanged `ff_screen_model` rasteriser from the model, independent of the
    // text-flavor table work, and depend only on the model (not on
    // `markdown_sections`).
    #[test]
    fn png_picture_flavor_unchanged_by_tables() {
        let model = config_fixture();
        let png_with_tables =
            ff_screen_model::encode_png(&ff_screen_model::rasterise_screen(&model));
        // A clone with the markdown_sections (a TEXT-flavor-only field) removed
        // must rasterise to the IDENTICAL PNG: the picture path ignores tables.
        let mut no_sections = model.clone();
        no_sections.markdown_sections.clear();
        let png_without_tables =
            ff_screen_model::encode_png(&ff_screen_model::rasterise_screen(&no_sections));
        assert_eq!(
            png_with_tables, png_without_tables,
            "the PNG picture flavor is independent of the property/value tables (Req 28.3)"
        );
    }
}
