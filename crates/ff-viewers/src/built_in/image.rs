//! Image Viewer — renders image files as a preview placeholder.
//!
//! This is a stub implementation that displays image metadata and a placeholder.
//! Full image decoding and rendering would require an image processing library;
//! this viewer provides the framework registration and basic header parsing.

use crate::trait_def::FileViewer;

/// Built-in viewer for image files (PNG, JPEG, GIF, BMP, WEBP).
///
/// Renders a scaled preview placeholder within the Viewer_Panel. If the image
/// cannot be decoded, displays the filename, dimensions (if available from
/// headers), and an error description.
pub struct ImageViewer {
    /// Cached display name.
    display_name: String,
}

impl ImageViewer {
    /// Create a new Image Viewer instance.
    pub fn new() -> Self {
        Self {
            display_name: "Image Preview".to_string(),
        }
    }

    /// Attempt to determine image dimensions from file headers.
    fn detect_dimensions(&self, content: &[u8]) -> Option<(u32, u32)> {
        if content.len() < 24 {
            return None;
        }

        // PNG: bytes 16-23 contain width (4 bytes) and height (4 bytes) in IHDR
        if content.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            let width = u32::from_be_bytes([content[16], content[17], content[18], content[19]]);
            let height = u32::from_be_bytes([content[20], content[21], content[22], content[23]]);
            return Some((width, height));
        }

        // BMP: bytes 18-25 contain width and height as little-endian u32
        if content.starts_with(&[0x42, 0x4D]) && content.len() >= 26 {
            let width = u32::from_le_bytes([content[18], content[19], content[20], content[21]]);
            let height = u32::from_le_bytes([content[22], content[23], content[24], content[25]]);
            return Some((width, height));
        }

        None
    }

    /// Detect the image format from magic bytes.
    fn detect_format(&self, content: &[u8]) -> &'static str {
        if content.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            "PNG"
        } else if content.starts_with(&[0xFF, 0xD8, 0xFF]) {
            "JPEG"
        } else if content.starts_with(b"GIF87a") || content.starts_with(b"GIF89a") {
            "GIF"
        } else if content.starts_with(&[0x42, 0x4D]) {
            "BMP"
        } else if content.starts_with(b"RIFF") && content.len() >= 12 && &content[8..12] == b"WEBP"
        {
            "WEBP"
        } else {
            "Unknown"
        }
    }
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileViewer for ImageViewer {
    fn viewer_key(&self) -> &str {
        "image"
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        "Renders image files (PNG, JPEG, GIF, BMP, WEBP) as a scaled preview"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "gif", "bmp", "webp"]
    }

    fn supported_mime_types(&self) -> &[&str] {
        &[
            "image/png",
            "image/jpeg",
            "image/gif",
            "image/bmp",
            "image/webp",
        ]
    }

    fn can_render(&self, uri: &str, content_sample: &[u8]) -> bool {
        // Check by extension first
        let uri_lower = uri.to_lowercase();
        if self
            .supported_extensions()
            .iter()
            .any(|ext| uri_lower.ends_with(&format!(".{ext}")))
        {
            return true;
        }

        // Check by magic bytes
        let format = self.detect_format(content_sample);
        format != "Unknown"
    }

    fn render(&self, content: &[u8]) -> String {
        let format = self.detect_format(content);
        let dimensions = self.detect_dimensions(content);

        let mut output = String::new();
        output.push_str(&format!("[Image Preview — {format} format]\n"));

        if let Some((w, h)) = dimensions {
            output.push_str(&format!("Dimensions: {w} × {h} pixels\n"));
        }

        output.push_str(&format!("Size: {} bytes\n", content.len()));

        if format == "Unknown" {
            output.push_str("Error: Unable to decode image — unrecognized format\n");
        } else {
            output
                .push_str("(Image rendering placeholder — full rendering requires GUI context)\n");
        }

        output
    }

    fn on_content_changed(&mut self, _new_content: &[u8]) {
        // Stub: no internal state to update for now.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_key_is_image() {
        // Validates: Requirement 4 AC 3
        let viewer = ImageViewer::new();
        assert_eq!(viewer.viewer_key(), "image");
    }

    #[test]
    fn supported_extensions_include_all_image_formats() {
        // Validates: Requirement 4 AC 3
        let viewer = ImageViewer::new();
        let exts = viewer.supported_extensions();
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"jpeg"));
        assert!(exts.contains(&"gif"));
        assert!(exts.contains(&"bmp"));
        assert!(exts.contains(&"webp"));
    }

    #[test]
    fn can_render_matches_by_extension() {
        let viewer = ImageViewer::new();
        assert!(viewer.can_render("file:///photo.png", b""));
        assert!(viewer.can_render("file:///photo.jpg", b""));
        assert!(!viewer.can_render("file:///data.csv", b""));
    }

    #[test]
    fn can_render_matches_by_magic_bytes() {
        let viewer = ImageViewer::new();
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(viewer.can_render("file:///unknown", &png_header));
    }

    #[test]
    fn render_shows_format_and_size() {
        let viewer = ImageViewer::new();
        let output = viewer.render(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]);
        assert!(output.contains("JPEG"));
        assert!(output.contains("6 bytes"));
    }

    #[test]
    fn render_shows_error_for_unknown_format() {
        let viewer = ImageViewer::new();
        let output = viewer.render(b"not an image");
        assert!(output.contains("Unknown"));
        assert!(output.contains("unrecognized format"));
    }
}
