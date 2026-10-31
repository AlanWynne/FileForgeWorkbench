//! Adapter from an egui window screenshot to the egui-free
//! [`ff_screen_model::RasterImage`] (screen-snapshot-scrm Requirement 23.4, 2.4).
//!
//! This is the ONLY new egui-touching code in the picture-clipboard path; it
//! lives in `ff-desktop` so `ff-screen-model` stays egui-free (Req 2.4). The
//! live window grab arrives as an [`egui::ColorImage`] (RGBA8, row-major from the
//! top-left) in a later frame after a screenshot viewport command; this copies
//! its bytes and dimensions into the plain [`RasterImage`] the clipboard backend
//! and PNG encoder consume.
//!
//! Validates: screen-snapshot-scrm Requirement 23.4, 2.4.

use ff_screen_model::RasterImage;

/// Copy an egui window-screenshot [`egui::ColorImage`] into an egui-free
/// [`RasterImage`] (RGBA8, row-major, top-left origin). `as_raw()` yields the
/// RGBA bytes in the same layout `RasterImage` expects, so this is a direct copy.
///
/// Validates: Req 23.4, 2.4.
//
// Wired into the live SNAPSHOT frame loop in FEAT-002
// (`shell/commands_snapshot::place_snapshot_composite`).
pub fn color_image_to_raster(image: &egui::ColorImage) -> RasterImage {
    let [width, height] = image.size;
    RasterImage {
        width: width as u32,
        height: height as u32,
        rgba: image.as_raw().to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Req 23.4, 2.4 -- the adapter preserves width, height, and the
    // RGBA bytes of a known egui ColorImage.
    #[test]
    fn color_image_to_raster_preserves_dimensions_and_bytes() {
        // 2x1 image: opaque red then opaque green. Opaque alpha keeps the
        // un-multiplied RGBA bytes equal to the input exactly.
        let rgba_in: [u8; 8] = [0xFF, 0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF];
        let image = egui::ColorImage::from_rgba_unmultiplied([2, 1], &rgba_in);
        let raster = color_image_to_raster(&image);
        assert_eq!(raster.width, 2);
        assert_eq!(raster.height, 1);
        assert_eq!(
            raster.rgba,
            rgba_in.to_vec(),
            "RGBA bytes copied in row-major top-left order"
        );
    }
}
