//! Multi-flavor clipboard backend for the picture SNAPSHOT (screen-snapshot-scrm
//! Requirement 23.4, 23.5c, 23.5d, 23.5e).
//!
//! ## C.4a finding -- what the app had, and what this adds
//!
//! Before this module, the shell copied a SNAPSHOT to the clipboard with
//! `arboard::Clipboard::set_text` (see the former `handle_snapshot`). `arboard`
//! is a fine TEXT clipboard, but its `set()` builder is single-flavor per
//! transaction: each terminal method (`.text()`, `.image()`) opens, clears,
//! writes ONE format, and closes. There is no single `arboard` call that places
//! BOTH an image flavor AND a Unicode-text flavor on the OS clipboard at once,
//! so a second `set()` would clear the first. The app's clipboard backend was
//! therefore effectively TEXT-ONLY for a single transaction.
//!
//! Req 23.5d requires the image AND text flavors to be offered SIMULTANEOUSLY
//! (Windows `CF_DIB` / `CF_DIBV5` + `CF_UNICODETEXT`) so a rich target (Word /
//! PowerPoint) can take the picture while a plain-text target (Notepad) takes
//! the text. On Windows this module uses `clipboard-win`'s RAW API: it opens the
//! clipboard ONCE, empties it ONCE, then publishes several formats with
//! `raw::set_without_clear` in the same open -- the true multi-flavor offer. On
//! other platforms it falls back to `arboard` (image then text, best-effort)
//! behind the SAME trait, so the capture code never depends on the backend.
//!
//! ## Windows picture-paste fix (CF_DIB / CF_DIBV5)
//!
//! Phase 2B first published the picture as `CF_BITMAP` (via the higher-level
//! `clipboard_win::formats::Bitmap`, from a full BMP file). Word / PowerPoint
//! did NOT render the pasted picture: they reliably accept a DEVICE-INDEPENDENT
//! BITMAP (`CF_DIB` = 8, or `CF_DIBV5` = 17) whose bytes are the
//! `BITMAPINFOHEADER` (or `BITMAPV5HEADER`) FOLLOWED BY pixel data, with NO
//! 14-byte `BITMAPFILEHEADER`. This module now publishes BOTH `CF_DIB` and
//! `CF_DIBV5` in the same clipboard open (maximising target compatibility) via
//! the raw API, plus a registered `PNG` format, alongside the `CF_UNICODETEXT`
//! Markdown-fenced text. The DIB bytes are derived from [`raster_to_bmp`] by
//! stripping its 14-byte file header; the V5 variant rebuilds the header as a
//! `BITMAPV5HEADER`. See [`raster_to_dib`] / [`raster_to_dib_v5`].
//!
//! The whole thing sits behind the [`SnapshotClipboard`] trait and a plain
//! [`SnapshotPayload`] so the "offers both flavors" behaviour is unit-testable
//! WITHOUT touching a real OS clipboard (we assert the built payload, the DIB
//! byte layout, and the reported [`OfferedFlavors`], not the Windows clipboard
//! itself). The real proof the picture pastes is the owner's manual Word paste;
//! a non-Windows CI host cannot verify the Win32 clipboard contract (Req 23.5d).
//!
//! Validates: screen-snapshot-scrm Requirement 23.4, 23.5c, 23.5d, 23.5e.

use ff_screen_model::{RasterImage, ScreenModel};

/// The two co-equal clipboard flavors a picture SNAPSHOT offers: a PNG/raster
/// picture AND the box-drawn ANSI/TEXT rendering, both derived from the SAME
/// logical [`ScreenModel`] (Req 23.4). Neither is subordinate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotPayload {
    /// The rasterised picture (RGBA pixels) for the image flavor.
    pub image: RasterImage,
    /// PNG-encoded bytes of the same picture (for a PNG file target / image flavor).
    pub png: Vec<u8>,
    /// The selectable text rendering for the text flavor.
    pub text: String,
    /// The CF_HTML composite fragment (image + box-drawn `<pre>` text) for a
    /// rich HTML-aware target such as Word, or `None` when no HTML flavor is
    /// offered. Built by [`crate::clipboard_html::build_cf_html`] (Req 23.5c).
    pub html: Option<String>,
}

impl SnapshotPayload {
    /// Build both flavors from a single logical screen model (Req 23.4): the
    /// raster/PNG picture and the box-drawn text, so the two always agree.
    ///
    /// `text` is the SAME rendering the text-only SNAPSHOT path produces, passed
    /// in so the payload carries exactly what the user asked for (e.g. ANSI or
    /// plain text); the picture is rasterised from the model.
    ///
    /// Validates: Req 23.4.
    //
    // Test-supporting since FEAT-002: the live bare-SNAPSHOT path now captures
    // the REAL window via `from_window_capture` (the picture must be the window
    // grab, not a synthetic model rasterisation), so production no longer calls
    // `from_model`. It is retained as the pure, model-only constructor the
    // clipboard_backend unit tests assert against; the `allow` keeps a non-test
    // build clean.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_model(model: &ScreenModel, text: String) -> Self {
        let image = ff_screen_model::rasterise_screen(model);
        let png = ff_screen_model::encode_png(&image);
        Self {
            image,
            png,
            text,
            html: None,
        }
    }

    /// Build a payload from an ALREADY-CAPTURED window raster (the live egui
    /// screenshot adapted by [`crate::screenshot::color_image_to_raster`]) plus
    /// the box-drawn text rendering, PNG-encoding the given image directly. The
    /// model is NOT re-rasterised here: the picture is the real window grab, not
    /// a synthetic text render (Req 23.4, 23.5d). The HTML composite flavor is
    /// filled in later by the shell once the CF_HTML fragment is built.
    ///
    /// Validates: Req 23.4, 23.5d.
    //
    // Wired into the live SNAPSHOT frame loop in FEAT-002
    // (`shell/commands_snapshot::place_snapshot_composite`).
    pub fn from_window_capture(image: RasterImage, text: String) -> Self {
        let png = ff_screen_model::encode_png(&image);
        Self {
            image,
            png,
            text,
            html: None,
        }
    }
}

/// Which flavors a [`SnapshotClipboard::place`] call actually offered. Used by
/// tests to assert that BOTH an image flavor and a text flavor were placed
/// (Req 23.4, 23.5d) without reading a real OS clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OfferedFlavors {
    /// Whether the image (picture) flavor was offered.
    pub image: bool,
    /// Whether the Unicode-text flavor was offered.
    pub text: bool,
    /// Whether the CF_HTML composite flavor was offered (Req 23.5c, 23.5e).
    pub html: bool,
}

impl OfferedFlavors {
    /// True when BOTH co-equal flavors were offered (the Req 23.4 happy path).
    pub fn both(self) -> bool {
        self.image && self.text
    }
}

/// A clipboard that can place a multi-flavor SNAPSHOT payload. The production
/// implementation talks to the OS clipboard; tests use a capturing fake.
///
/// Validates: Req 23.4, 23.5d.
pub trait SnapshotClipboard {
    /// Place BOTH the picture and the text flavor on the clipboard in a single
    /// transaction, returning which flavors were offered. An `Err` means the
    /// clipboard could not be opened at all.
    fn place(&mut self, payload: &SnapshotPayload) -> Result<OfferedFlavors, String>;
}

/// Offset of the pixel bits in a BMP file / a DIB: the 14-byte
/// `BITMAPFILEHEADER` followed by the 40-byte `BITMAPINFOHEADER`.
const FILE_HEADER: usize = 14;
const INFO_HEADER: usize = 40;
/// Size in bytes of a `BITMAPV5HEADER` (`bV5Size`).
const V5_HEADER: usize = 124;

/// Build a COMPLETE 24-bit, bottom-up, BGR `BMP` FILE (the 2-byte "BM"
/// signature + 14-byte `BITMAPFILEHEADER` + `BITMAPINFOHEADER` + pixel payload)
/// from an RGBA raster.
///
/// This is the single source of the pixel layout: the `CF_DIB` and `CF_DIBV5`
/// clipboard flavors are both derived from it ([`raster_to_dib`],
/// [`raster_to_dib_v5`]) by stripping / rebuilding the leading header, so the
/// pixel bytes always agree. It is also directly useful anywhere a `.bmp` FILE
/// is wanted (the 14-byte file header + `bfOffBits` make it a loadable file).
///
/// Kept dependency-free and deterministic so it is unit-tested without a real
/// clipboard.
///
/// Validates: Req 23.5d.
pub fn raster_to_bmp(image: &RasterImage) -> Vec<u8> {
    let w = image.width as usize;
    let h = image.height as usize;
    // Each row is padded to a 4-byte boundary (BMP requirement).
    let row_bytes = w * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let padded_row = row_bytes + padding;
    let pixel_size = padded_row * h;
    let file_size = FILE_HEADER + INFO_HEADER + pixel_size;

    let mut out = Vec::with_capacity(file_size);
    // BITMAPFILEHEADER.
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(file_size as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // reserved
    out.extend_from_slice(&((FILE_HEADER + INFO_HEADER) as u32).to_le_bytes()); // pixel offset
                                                                                // BITMAPINFOHEADER.
    out.extend_from_slice(&(INFO_HEADER as u32).to_le_bytes());
    out.extend_from_slice(&(image.width as i32).to_le_bytes());
    out.extend_from_slice(&(image.height as i32).to_le_bytes()); // positive = bottom-up
    out.extend_from_slice(&1u16.to_le_bytes()); // planes
    out.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel
    out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB (no compression)
    out.extend_from_slice(&(pixel_size as u32).to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes()); // x ppm (~72 dpi)
    out.extend_from_slice(&2835i32.to_le_bytes()); // y ppm
    out.extend_from_slice(&0u32.to_le_bytes()); // colours used
    out.extend_from_slice(&0u32.to_le_bytes()); // important colours

    // Pixel data, bottom-up, BGR, each row padded.
    for row in (0..h).rev() {
        for col in 0..w {
            let off = (row * w + col) * 4;
            let r = image.rgba[off];
            let g = image.rgba[off + 1];
            let b = image.rgba[off + 2];
            out.push(b);
            out.push(g);
            out.push(r);
        }
        out.extend(std::iter::repeat_n(0u8, padding));
    }
    out
}

/// Build the `CF_DIB` (format 8) clipboard payload: a bare DEVICE-INDEPENDENT
/// BITMAP, i.e. the 40-byte `BITMAPINFOHEADER` immediately followed by the
/// bottom-up BGR pixel data, with NO 14-byte `BITMAPFILEHEADER`.
///
/// Derived from [`raster_to_bmp`] by stripping the leading 14-byte file header
/// (exactly as the owner brief requires), so the DIB and the `.bmp` file carry
/// identical pixels. This is the flavor Word / PowerPoint read when pasting a
/// picture; the earlier `CF_BITMAP` publish did not render there.
///
/// Validates: Req 23.5d.
pub fn raster_to_dib(image: &RasterImage) -> Vec<u8> {
    // Skip the BITMAPFILEHEADER -- a CF_DIB begins at the BITMAPINFOHEADER.
    raster_to_bmp(image)[FILE_HEADER..].to_vec()
}

/// Build the `CF_DIBV5` (format 17) clipboard payload: a `BITMAPV5HEADER`
/// (124 bytes) immediately followed by the SAME bottom-up BGR pixel data as the
/// `CF_DIB`. `CF_DIBV5` is the richest DIB flavor and the most reliably accepted
/// by modern Office; publishing it alongside `CF_DIB` maximises compatibility.
///
/// The pixel bytes are taken from the `CF_DIB` payload (after its 40-byte
/// `BITMAPINFOHEADER`), so all three representations share one pixel layout.
/// The V5 header re-states the width/height/planes/bit-count and sets BI_RGB
/// (no compression) with the sRGB colour-space tag; the colour masks are zero
/// for a BI_RGB bitmap.
///
/// Validates: Req 23.5d.
pub fn raster_to_dib_v5(image: &RasterImage) -> Vec<u8> {
    let dib = raster_to_dib(image);
    // Pixel data begins right after the 40-byte BITMAPINFOHEADER in `dib`.
    let pixels = &dib[INFO_HEADER..];
    let mut out = Vec::with_capacity(V5_HEADER + pixels.len());
    // --- BITMAPV5HEADER (124 bytes) ---
    out.extend_from_slice(&(V5_HEADER as u32).to_le_bytes()); // bV5Size = 124
    out.extend_from_slice(&(image.width as i32).to_le_bytes()); // bV5Width
    out.extend_from_slice(&(image.height as i32).to_le_bytes()); // bV5Height (+ = bottom-up)
    out.extend_from_slice(&1u16.to_le_bytes()); // bV5Planes
    out.extend_from_slice(&24u16.to_le_bytes()); // bV5BitCount
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5Compression = BI_RGB
    out.extend_from_slice(&(pixels.len() as u32).to_le_bytes()); // bV5SizeImage
    out.extend_from_slice(&2835i32.to_le_bytes()); // bV5XPelsPerMeter (~72 dpi)
    out.extend_from_slice(&2835i32.to_le_bytes()); // bV5YPelsPerMeter
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5ClrUsed
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5ClrImportant
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5RedMask (0 for BI_RGB)
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5GreenMask
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5BlueMask
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5AlphaMask
                                                // bV5CSType = LCS_sRGB ("sRGB" little-endian: 'B','G','R','s' = 0x73524742).
    out.extend_from_slice(&0x7352_4742u32.to_le_bytes());
    // bV5Endpoints: CIEXYZTRIPLE = 3 * 3 * u32 = 36 bytes, all zero for sRGB.
    out.extend_from_slice(&[0u8; 36]);
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5GammaRed
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5GammaGreen
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5GammaBlue
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5Intent
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5ProfileData
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5ProfileSize
    out.extend_from_slice(&0u32.to_le_bytes()); // bV5Reserved
    debug_assert_eq!(out.len(), V5_HEADER, "BITMAPV5HEADER is 124 bytes");
    out.extend_from_slice(pixels);
    out
}

/// The production, OS-backed multi-flavor clipboard.
#[derive(Debug, Default)]
pub struct OsClipboard;

/// Standard Windows clipboard format ids used by the DIB picture flavors.
#[cfg(windows)]
const CF_DIB: u32 = 8;
#[cfg(windows)]
const CF_DIBV5: u32 = 17;

/// Encode a string as the `CF_UNICODETEXT` payload: UTF-16LE code units plus a
/// terminating NUL, as raw little-endian bytes (what `raw::set_without_clear`
/// expects for format 13).
#[cfg(windows)]
fn unicode_text_bytes(text: &str) -> Vec<u8> {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0); // NUL terminator
    let mut bytes = Vec::with_capacity(units.len() * 2);
    for u in units {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    bytes
}

#[cfg(windows)]
impl SnapshotClipboard for OsClipboard {
    /// Open the Windows clipboard ONCE, empty it, then publish every flavor in
    /// the same open with the RAW API so they are offered SIMULTANEOUSLY
    /// (Req 23.5d): the Markdown-fenced Unicode text (`CF_UNICODETEXT`) AND the
    /// picture as `CF_DIB` + `CF_DIBV5` (the DIB flavors Word / PowerPoint
    /// accept), plus a registered `PNG` format some apps prefer.
    ///
    /// `Clipboard::new_attempts` empties the clipboard on open, so every format
    /// is written with `raw::set_without_clear` to ADD to the same transaction
    /// (a plain `raw::set` would clear the earlier formats). ANSI is never put
    /// on the clipboard (Req 23.5e); the text flavor is the caller-supplied
    /// Markdown-fenced rendering.
    fn place(&mut self, payload: &SnapshotPayload) -> Result<OfferedFlavors, String> {
        use clipboard_win::{raw, Clipboard};
        // `new_attempts` opens and empties the clipboard; the guard closes it on
        // drop. Every write below ADDS a format to this one open transaction.
        let _clip =
            Clipboard::new_attempts(10).map_err(|e| format!("cannot open clipboard: {e}"))?;
        let mut flavors = OfferedFlavors::default();

        // Text flavor: CF_UNICODETEXT (format 13), UTF-16LE + NUL.
        let text_bytes = unicode_text_bytes(&payload.text);
        const CF_UNICODETEXT: u32 = 13;
        if raw::set_without_clear(CF_UNICODETEXT, &text_bytes).is_ok() {
            flavors.text = true;
        }

        // Picture flavors: publish BOTH CF_DIB and CF_DIBV5 (header + BGR pixel
        // data, NO 14-byte BITMAPFILEHEADER). Either one accepted = image offered.
        let dib = raster_to_dib(&payload.image);
        if raw::set_without_clear(CF_DIB, &dib).is_ok() {
            flavors.image = true;
        }
        let dib_v5 = raster_to_dib_v5(&payload.image);
        if raw::set_without_clear(CF_DIBV5, &dib_v5).is_ok() {
            flavors.image = true;
        }

        // Also offer a registered "PNG" format (best-effort; some apps prefer
        // it). Not the must-have -- CF_DIB/CF_DIBV5 is what Word needs -- so a
        // failure to register or write it does not affect `flavors`.
        if let Some(png_fmt) = raw::register_format("PNG") {
            let _ = raw::set_without_clear(png_fmt.get(), &payload.png);
        }

        // CF_HTML composite flavor (Req 23.5c): the registered "HTML Format"
        // carrying the image + box-drawn <pre> text, so a rich HTML-aware target
        // (Word) pastes the whole composite. ANSI is never a clipboard flavor
        // (Req 23.5e); only this HTML fragment and the Markdown-fenced text.
        if let Some(html) = &payload.html {
            if let Some(html_fmt) = raw::register_format("HTML Format") {
                if raw::set_without_clear(html_fmt.get(), html.as_bytes()).is_ok() {
                    flavors.html = true;
                }
            }
        }

        Ok(flavors)
    }
}

#[cfg(not(windows))]
impl SnapshotClipboard for OsClipboard {
    /// Non-Windows fallback via `arboard`: write the image, then the text.
    /// `arboard` is single-flavor per `set()`, so this is best-effort (the text
    /// is the authoritative flavor a plain-text target reads). Windows is the
    /// primary target for the simultaneous multi-flavor offer (Req 23.5d).
    fn place(&mut self, payload: &SnapshotPayload) -> Result<OfferedFlavors, String> {
        let mut cb =
            arboard::Clipboard::new().map_err(|e| format!("cannot open clipboard: {e}"))?;
        let mut flavors = OfferedFlavors::default();
        let img = arboard::ImageData {
            width: payload.image.width as usize,
            height: payload.image.height as usize,
            bytes: std::borrow::Cow::Borrowed(&payload.image.rgba),
        };
        if cb.set_image(img).is_ok() {
            flavors.image = true;
        }
        if cb.set_text(&payload.text).is_ok() {
            flavors.text = true;
        }
        Ok(flavors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_screen_model::Field;

    fn sample_model() -> ScreenModel {
        ScreenModel::new("Props").with_field(Field::new("Name", "USER.TEST"))
    }

    /// A capturing fake clipboard: records the payload it was asked to place and
    /// reports both flavors offered, so the multi-flavor layer is testable with
    /// no real OS clipboard (Req 23.4).
    #[derive(Default)]
    struct FakeClipboard {
        placed: Option<SnapshotPayload>,
    }
    impl SnapshotClipboard for FakeClipboard {
        fn place(&mut self, payload: &SnapshotPayload) -> Result<OfferedFlavors, String> {
            self.placed = Some(payload.clone());
            Ok(OfferedFlavors {
                image: !payload.png.is_empty(),
                text: !payload.text.is_empty(),
                html: payload.html.as_ref().is_some_and(|h| !h.is_empty()),
            })
        }
    }

    // Validates: Req 23.4 -- the payload carries BOTH a picture (non-empty PNG
    // with a valid signature) and the text, both from the one model.
    #[test]
    fn payload_from_model_has_both_flavors() {
        let model = sample_model();
        let text = ff_screen_model::render(
            &model,
            ff_screen_model::SnapshotFormat::Ansi,
            ff_screen_model::RenderOptions::default(),
        );
        let payload = SnapshotPayload::from_model(&model, text);
        assert!(!payload.png.is_empty(), "picture flavor present");
        const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(&payload.png[..8], &SIG);
        assert!(payload.text.contains("USER.TEST"), "text flavor present");
    }

    // Validates: Req 23.4, 23.5d -- from_window_capture PNG-encodes the given
    // (already-captured) window raster and round-trips the text, without
    // re-rasterising a model. A 2x1 stub raster yields a valid PNG signature.
    #[test]
    fn from_window_capture_encodes_png_and_keeps_text() {
        let image = RasterImage {
            width: 2,
            height: 1,
            rgba: vec![
                0xFF, 0x00, 0x00, 0xFF, // red
                0x00, 0xFF, 0x00, 0xFF, // green
            ],
        };
        let payload = SnapshotPayload::from_window_capture(image.clone(), "BOX TEXT".to_string());
        const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(
            !payload.png.is_empty(),
            "PNG encoded from the window raster"
        );
        assert_eq!(&payload.png[..8], &SIG, "valid PNG signature");
        assert_eq!(payload.text, "BOX TEXT", "the text flavor round-trips");
        assert_eq!(payload.image, image, "the captured raster is stored as-is");
        assert!(
            payload.html.is_none(),
            "HTML is filled in later by the shell"
        );
    }

    // Validates: Req 23.4, 23.5d -- placing a SNAPSHOT offers BOTH an image
    // flavor and a text flavor (asserted via the backend abstraction, no real
    // OS clipboard).
    #[test]
    fn place_offers_both_image_and_text_flavors() {
        let model = sample_model();
        let payload = SnapshotPayload::from_model(&model, "USER.TEST".to_string());
        let mut cb = FakeClipboard::default();
        let offered = cb.place(&payload).expect("place");
        assert!(offered.image, "image flavor offered");
        assert!(offered.text, "text flavor offered");
        assert!(offered.both(), "both co-equal flavors offered");
        assert!(!offered.html, "no HTML flavor when payload.html is None");
        // The exact payload reached the backend.
        let placed = cb.placed.expect("payload captured");
        assert_eq!(placed.text, "USER.TEST");
        assert!(!placed.png.is_empty());
    }

    // Validates: Req 23.5c, 23.5d, 23.5e -- when the payload carries an HTML
    // composite, the clipboard offers ALL THREE flavors (image, text, html),
    // asserted via the backend abstraction with no real OS clipboard.
    #[test]
    fn place_offers_all_three_flavors_when_html_present() {
        let model = sample_model();
        let mut payload = SnapshotPayload::from_model(&model, "USER.TEST".to_string());
        payload.html = Some("<pre>USER.TEST</pre>".to_string());
        let mut cb = FakeClipboard::default();
        let offered = cb.place(&payload).expect("place");
        assert!(offered.image, "image flavor offered");
        assert!(offered.text, "text flavor offered");
        assert!(offered.html, "HTML composite flavor offered");
    }

    // Validates: Req 23.4 -- the composite TEXT flavor placed on the clipboard
    // is the MARKDOWN-FENCED rendering: it carries the triple-backtick fence and
    // the box-drawn screen body inside it, so a Markdown-aware target renders the
    // box-drawing monospaced while a plain-text target still pastes readable
    // text. (The shell builds the payload from SnapshotFormat::Markdown for bare
    // SNAPSHOT; here we assert the renderer's output has the fence + body.)
    #[test]
    fn composite_text_flavor_is_markdown_fenced() {
        let model = sample_model();
        let text = ff_screen_model::render(
            &model,
            ff_screen_model::SnapshotFormat::Markdown,
            ff_screen_model::RenderOptions::default(),
        );
        let payload = SnapshotPayload::from_model(&model, text);
        assert!(
            payload.text.contains("```"),
            "text flavor is Markdown-fenced (triple-backtick code block)"
        );
        assert!(
            payload.text.contains("USER.TEST"),
            "the box-drawn screen body is inside the fence"
        );
        // ANSI escape sequences must NOT appear in the clipboard text (Req
        // 23.5e: ANSI is not a clipboard flavor).
        assert!(
            !payload.text.contains('\u{1b}'),
            "no raw ANSI escapes in the clipboard text flavor"
        );
    }

    // Validates: Req 23.5d -- the CF_DIB (format 8) buffer handed to the Windows
    // clipboard is a BARE device-independent bitmap: it STARTS with a 40-byte
    // BITMAPINFOHEADER (biSize = 0x28,0,0,0) and does NOT begin with the "BM"
    // BITMAPFILEHEADER signature (the 14-byte file header is stripped). This is
    // exactly the layout Word / PowerPoint accept for a pasted picture; the old
    // CF_BITMAP-from-full-BMP publish did not render there.
    #[test]
    fn cf_dib_buffer_starts_with_bitmapinfoheader_not_file_header() {
        let img = ff_screen_model::rasterise_screen(&sample_model());
        let dib = raster_to_dib(&img);

        // biSize = 40 (0x28) at offset 0 -- a DIB begins here.
        assert_eq!(
            &dib[0..4],
            &[0x28, 0x00, 0x00, 0x00],
            "CF_DIB begins with a 40-byte BITMAPINFOHEADER (biSize = 40)"
        );
        // The 14-byte BITMAPFILEHEADER is gone: no "BM" signature at the front.
        assert_ne!(&dib[0..2], b"BM", "no BITMAPFILEHEADER (not a BMP file)");
        // Width/height/bpp survive the strip and match the raster.
        let width = i32::from_le_bytes([dib[4], dib[5], dib[6], dib[7]]);
        let height = i32::from_le_bytes([dib[8], dib[9], dib[10], dib[11]]);
        let bpp = u16::from_le_bytes([dib[14], dib[15]]);
        assert_eq!(width, img.width as i32);
        assert_eq!(height, img.height as i32, "positive = bottom-up DIB");
        assert_eq!(bpp, 24, "24-bit BGR DIB");
        // The DIB is exactly the full BMP minus its 14-byte file header.
        assert_eq!(dib.len(), raster_to_bmp(&img).len() - 14);
    }

    // Validates: Req 23.5d -- the CF_DIBV5 (format 17) buffer begins with a
    // 124-byte BITMAPV5HEADER (bV5Size = 0x7C,0,0,0) and carries the same pixel
    // data as the CF_DIB. CF_DIBV5 is the richest DIB flavor and the most
    // reliably accepted by modern Office.
    #[test]
    fn cf_dibv5_buffer_starts_with_bitmapv5header() {
        let img = ff_screen_model::rasterise_screen(&sample_model());
        let v5 = raster_to_dib_v5(&img);

        // bV5Size = 124 (0x7C) at offset 0.
        assert_eq!(
            &v5[0..4],
            &[0x7C, 0x00, 0x00, 0x00],
            "CF_DIBV5 begins with a 124-byte BITMAPV5HEADER (bV5Size = 124)"
        );
        assert_ne!(&v5[0..2], b"BM", "not a BMP file");
        // Width/height/bpp.
        let width = i32::from_le_bytes([v5[4], v5[5], v5[6], v5[7]]);
        let height = i32::from_le_bytes([v5[8], v5[9], v5[10], v5[11]]);
        let bpp = u16::from_le_bytes([v5[14], v5[15]]);
        assert_eq!(width, img.width as i32);
        assert_eq!(height, img.height as i32);
        assert_eq!(bpp, 24);
        // Pixel bytes after the 124-byte header match the CF_DIB pixel bytes
        // (after its 40-byte header): one shared pixel layout.
        let dib = raster_to_dib(&img);
        assert_eq!(&v5[124..], &dib[40..], "V5 and DIB share the pixel layout");
    }
}
