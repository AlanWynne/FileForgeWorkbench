//! Rasterise the plain-text screen render into a bitmap and encode it as PNG.
//!
//! The picture SNAPSHOT (screen-snapshot-scrm Requirement 23.2) is produced by
//! RASTERISING the monospaced plain-TEXT render of a [`ScreenModel`] with a
//! fixed-width font (one glyph per character cell), NOT by grabbing egui pixels.
//! This keeps the logical-model-first constraint (Requirement 1, 2.2): the exact
//! same text the clipboard/text flavour carries is what the picture draws.
//!
//! The output is deterministic for a fixed model (fixed cell size, fixed font,
//! fixed colours), so it is unit-testable without a GUI.
//!
//! Validates: screen-snapshot-scrm Requirement 23.2, 1.1, 2.2.

mod font;

use crate::model::ScreenModel;
use crate::render::{render, RenderOptions};
use crate::SnapshotFormat;

/// Pixels per character cell, derived from the 5x7 glyph plus a one-pixel gap
/// so adjacent glyphs never touch. Fixed so the raster is deterministic.
const CELL_W: usize = font::GLYPH_W + 1; // 6 px
const CELL_H: usize = font::GLYPH_H + 2; // 9 px
/// Pixel margin around the text block.
const MARGIN: usize = 4;

/// Foreground (ink) and background colours. Black ink on white, like a printed
/// panel; opaque alpha so a raster paste is solid.
const FG: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];
const BG: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

/// A simple RGBA8 raster: `width * height` pixels, 4 bytes each, row-major from
/// the top-left. Kept free of any image crate so the clipboard layer can build a
/// platform DIB directly from `rgba` without re-decoding the PNG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row-major RGBA8 pixels (`width * height * 4` bytes).
    pub rgba: Vec<u8>,
}

/// An integer pixel rectangle used to crop a [`RasterImage`] to a sub-region
/// (screen-snapshot-scrm Requirement 23.2a). `x`/`y` are the top-left origin and
/// `width`/`height` the extent, all in physical pixels from the top-left of the
/// source image. Kept egui-free so the crop stays in this GUI-independent crate
/// (Req 2.4); the caller converts an on-screen points rect to physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    /// Left edge, pixels from the image's left.
    pub x: u32,
    /// Top edge, pixels from the image's top.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Crop `image` to the sub-rectangle `rect`, returning a new [`RasterImage`]
/// whose pixels are the copied sub-region (screen-snapshot-scrm Requirement
/// 23.2a). The rectangle is CLAMPED to the source bounds so an off-image or
/// oversized rect can never panic: the returned image is the intersection of
/// `rect` and the source. A rect with zero area after clamping (e.g. an origin
/// at or beyond the image edge) yields a 1x1 image so the PNG encoder and the
/// DIB layer always receive a non-empty raster.
///
/// This is implemented as a direct row/column copy on the RGBA buffer (no egui,
/// no image crate), so it is deterministic and unit-testable without a GUI.
///
/// Validates: screen-snapshot-scrm Requirement 23.2a, 2.4.
pub fn crop(image: &RasterImage, rect: PixelRect) -> RasterImage {
    // Clamp the origin to the image, then the extent to what remains, so the
    // crop is always a valid sub-rectangle of the source.
    let x0 = rect.x.min(image.width);
    let y0 = rect.y.min(image.height);
    let w = rect.width.min(image.width - x0);
    let h = rect.height.min(image.height - y0);

    // A degenerate (zero-area) crop would produce an empty buffer the PNG/DIB
    // layers cannot use; fall back to a single opaque-black pixel instead.
    if w == 0 || h == 0 {
        return RasterImage {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 0xFF],
        };
    }

    let src_w = image.width as usize;
    let (x0, y0, w, h) = (x0 as usize, y0 as usize, w as usize, h as usize);
    let mut rgba = Vec::with_capacity(w * h * 4);
    for row in 0..h {
        let src_row = y0 + row;
        let start = (src_row * src_w + x0) * 4;
        let end = start + w * 4;
        rgba.extend_from_slice(&image.rgba[start..end]);
    }
    RasterImage {
        width: w as u32,
        height: h as u32,
        rgba,
    }
}

/// Rasterise `model`'s plain-text render into an [`RasterImage`] using the
/// fixed-width bitmap font. The ASCII fallback render is used (no Unicode box
/// glyphs) so every character maps to a font glyph; the logical content is
/// identical to the Unicode text, only the framing characters differ.
///
/// Validates: Requirement 23.2.
pub fn rasterise_screen(model: &ScreenModel) -> RasterImage {
    let text = render(
        model,
        SnapshotFormat::PlainText,
        RenderOptions { unicode_box: false },
    );
    rasterise_text(&text)
}

/// Rasterise an already-rendered monospaced `text` block. Exposed for tests and
/// so the picture is provably derived from the SAME text the clipboard carries.
pub fn rasterise_text(text: &str) -> RasterImage {
    let rows: Vec<&str> = text.lines().collect();
    let cols = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let width = MARGIN * 2 + cols * CELL_W;
    let height = MARGIN * 2 + rows.len().max(1) * CELL_H;

    // Fill the whole buffer with the background colour (repeating the 4-byte
    // RGBA pattern).
    let mut rgba: Vec<u8> = BG
        .iter()
        .copied()
        .cycle()
        .take(width * height * 4)
        .collect();

    for (row_idx, line) in rows.iter().enumerate() {
        let y0 = MARGIN + row_idx * CELL_H;
        for (col_idx, ch) in line.chars().enumerate() {
            let x0 = MARGIN + col_idx * CELL_W;
            draw_glyph(&mut rgba, width, x0, y0, ch);
        }
    }

    RasterImage {
        width: width as u32,
        height: height as u32,
        rgba,
    }
}

/// Draw one glyph's ink pixels at cell origin `(x0, y0)`.
fn draw_glyph(rgba: &mut [u8], width: usize, x0: usize, y0: usize, ch: char) {
    let cols = font::glyph(ch);
    for (cx, col_bits) in cols.iter().enumerate() {
        for ry in 0..font::GLYPH_H {
            if col_bits & (1 << ry) != 0 {
                let x = x0 + cx;
                let y = y0 + ry;
                let off = (y * width + x) * 4;
                rgba[off..off + 4].copy_from_slice(&FG);
            }
        }
    }
}

/// Encode a [`RasterImage`] as PNG bytes using the pinned `png` encoder.
///
/// Validates: Requirement 23.2, 23.7.
pub fn encode_png(image: &RasterImage) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .expect("PNG header write to an in-memory buffer cannot fail");
        writer
            .write_image_data(&image.rgba)
            .expect("PNG image write to an in-memory buffer cannot fail");
    }
    out
}

/// Rasterise `model` and encode it as PNG in one step (the `SNAPSHOT PNG` path).
///
/// Validates: Requirement 23.2.
pub fn screen_png(model: &ScreenModel) -> Vec<u8> {
    encode_png(&rasterise_screen(model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Field;

    fn sample() -> ScreenModel {
        ScreenModel::new("Dataset Properties")
            .with_field(Field::new("Dataset Name", "USER.TEST.PDS"))
            .with_field(Field::new("RECFM", "FB"))
    }

    // Validates: Req 23.2 -- SNAPSHOT PNG produces non-empty bytes with the PNG
    // 8-byte signature.
    #[test]
    fn screen_png_has_valid_signature() {
        let bytes = screen_png(&sample());
        assert!(!bytes.is_empty(), "PNG must not be empty");
        const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(&bytes[..8], &SIG, "valid PNG signature");
    }

    // Validates: Req 23.2, Req 1 -- the picture is derived from the TEXT render,
    // not an egui grab: the raster height tracks the text render's row count.
    #[test]
    fn raster_height_tracks_text_row_count() {
        let model = sample();
        let text = render(
            &model,
            SnapshotFormat::PlainText,
            RenderOptions { unicode_box: false },
        );
        let rows = text.lines().count();
        let img = rasterise_screen(&model);
        let expected = (MARGIN * 2 + rows * CELL_H) as u32;
        assert_eq!(img.height, expected, "height is a function of text rows");
    }

    // Validates: Req 23.2 -- rasterisation is deterministic for a fixed model.
    #[test]
    fn raster_is_deterministic() {
        let a = screen_png(&sample());
        let b = screen_png(&sample());
        assert_eq!(a, b, "same model must produce identical PNG bytes");
    }

    // Validates: Req 23.2 -- the raster actually draws ink (not a blank page).
    #[test]
    fn raster_has_ink_pixels() {
        let img = rasterise_screen(&sample());
        let has_ink = img.rgba.windows(4).any(|px| px == FG);
        assert!(has_ink, "rendered text must leave ink pixels");
    }

    /// Build a 4x3 image where each pixel's R channel encodes its column and G
    /// its row, so a crop can be checked pixel-exactly against known values.
    fn coded_image() -> RasterImage {
        let (w, h) = (4u32, 3u32);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                rgba.extend_from_slice(&[x as u8, y as u8, 0x00, 0xFF]);
            }
        }
        RasterImage {
            width: w,
            height: h,
            rgba,
        }
    }

    // Validates: Req 23.2a -- cropping a known image to an interior sub-rect
    // yields the sub-rect dimensions and the exact pixels of that region.
    #[test]
    fn crop_to_subrect_has_subrect_dimensions_and_pixels() {
        let img = coded_image();
        // Crop the 2x2 block whose top-left is column 1, row 1.
        let cropped = crop(
            &img,
            PixelRect {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            },
        );
        assert_eq!(cropped.width, 2, "cropped width is the sub-rect width");
        assert_eq!(cropped.height, 2, "cropped height is the sub-rect height");
        // Pixel (0,0) of the crop is source (1,1): R=1, G=1.
        assert_eq!(&cropped.rgba[0..4], &[1, 1, 0x00, 0xFF]);
        // Pixel (1,0) of the crop is source (2,1): R=2, G=1.
        assert_eq!(&cropped.rgba[4..8], &[2, 1, 0x00, 0xFF]);
        // Pixel (0,1) of the crop is source (1,2): R=1, G=2.
        assert_eq!(&cropped.rgba[8..12], &[1, 2, 0x00, 0xFF]);
        // Pixel (1,1) of the crop is source (2,2): R=2, G=2.
        assert_eq!(&cropped.rgba[12..16], &[2, 2, 0x00, 0xFF]);
    }

    // Validates: Req 23.2a -- a rect that overruns the source is clamped to the
    // image bounds rather than panicking, and the crop never exceeds the source.
    #[test]
    fn crop_rect_is_clamped_to_source_bounds() {
        let img = coded_image(); // 4x3
        let cropped = crop(
            &img,
            PixelRect {
                x: 2,
                y: 1,
                width: 100,
                height: 100,
            },
        );
        // Only columns 2..4 (2 wide) and rows 1..3 (2 tall) exist.
        assert_eq!(cropped.width, 2);
        assert_eq!(cropped.height, 2);
        // First crop pixel is source (2,1): R=2, G=1.
        assert_eq!(&cropped.rgba[0..4], &[2, 1, 0x00, 0xFF]);
    }

    // Validates: Req 23.2a -- a degenerate (zero-area / off-image) rect yields a
    // 1x1 fallback image, never an empty buffer the PNG/DIB layer cannot use.
    #[test]
    fn crop_zero_area_rect_yields_one_by_one() {
        let img = coded_image();
        let cropped = crop(
            &img,
            PixelRect {
                x: 10,
                y: 10,
                width: 5,
                height: 5,
            },
        );
        assert_eq!(cropped.width, 1);
        assert_eq!(cropped.height, 1);
        assert_eq!(cropped.rgba.len(), 4, "a single RGBA pixel");
    }
}
