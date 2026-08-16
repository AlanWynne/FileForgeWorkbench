//! Hex Viewer — renders binary file content in hex dump format.
//!
//! This is a stub implementation. The full rendering logic is defined in the
//! separate `hex-display` spec. This registration provides discoverability via
//! `PREVIEW LIST`. The canonical activation commands remain `HEX ON` / `HEX OFF`;
//! `PREVIEW hex` is accepted as an alias.

use crate::trait_def::FileViewer;

/// Built-in viewer for binary file content in hex dump format.
///
/// Displays content as offset + hex bytes + ASCII decode, similar to
/// traditional hex dump utilities.
pub struct HexViewer {
    /// Cached display name.
    display_name: String,
    /// Number of bytes to display per row.
    bytes_per_row: usize,
}

impl HexViewer {
    /// Create a new Hex Viewer instance with default settings.
    pub fn new() -> Self {
        Self {
            display_name: "Hex Display".to_string(),
            bytes_per_row: 16,
        }
    }
}

impl Default for HexViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileViewer for HexViewer {
    fn viewer_key(&self) -> &str {
        "hex"
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        "Renders binary content as hex dump (offset + hex bytes + ASCII decode)"
    }

    fn supported_extensions(&self) -> &[&str] {
        // Hex viewer is activated explicitly, not by extension
        &[]
    }

    fn supported_mime_types(&self) -> &[&str] {
        &["application/octet-stream"]
    }

    fn can_render(&self, _uri: &str, content_sample: &[u8]) -> bool {
        // Consider binary if content contains null bytes
        content_sample.contains(&0)
    }

    fn render(&self, content: &[u8]) -> String {
        let mut output = String::new();

        for (chunk_idx, chunk) in content.chunks(self.bytes_per_row).enumerate() {
            let offset = chunk_idx * self.bytes_per_row;

            // Offset column
            output.push_str(&format!("{offset:08X}  "));

            // Hex bytes column
            for (i, byte) in chunk.iter().enumerate() {
                output.push_str(&format!("{byte:02X} "));
                if i == 7 {
                    output.push(' ');
                }
            }

            // Pad remaining space if short row
            let padding = self.bytes_per_row - chunk.len();
            for _ in 0..padding {
                output.push_str("   ");
            }
            if chunk.len() <= 8 {
                output.push(' ');
            }

            output.push_str(" |");

            // ASCII decode column
            for byte in chunk {
                let ch = if byte.is_ascii_graphic() || *byte == b' ' {
                    *byte as char
                } else {
                    '.'
                };
                output.push(ch);
            }

            output.push_str("|\n");
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
    fn viewer_key_is_hex() {
        // Validates: Requirement 4 AC 2
        let viewer = HexViewer::new();
        assert_eq!(viewer.viewer_key(), "hex");
    }

    #[test]
    fn supported_extensions_is_empty() {
        // Validates: Requirement 4 AC 2 — activated explicitly
        let viewer = HexViewer::new();
        assert!(viewer.supported_extensions().is_empty());
    }

    #[test]
    fn can_render_detects_binary_content() {
        let viewer = HexViewer::new();
        assert!(viewer.can_render("file:///data.bin", &[0x00, 0x48, 0x65]));
        assert!(!viewer.can_render("file:///text.txt", b"hello world"));
    }

    #[test]
    fn render_produces_hex_dump_format() {
        let viewer = HexViewer::new();
        let content = b"Hello, World!";
        let output = viewer.render(content);
        assert!(output.contains("00000000"));
        assert!(output.contains("48 65 6C 6C"));
        assert!(output.contains("|Hello, World!|"));
    }

    #[test]
    fn render_handles_empty_content() {
        let viewer = HexViewer::new();
        let output = viewer.render(b"");
        assert!(output.is_empty());
    }
}
