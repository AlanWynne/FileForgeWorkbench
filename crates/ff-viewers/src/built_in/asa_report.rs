//! ASA Report Viewer — renders ASA carriage control report files.
//!
//! This is a stub implementation. The full rendering logic is defined in the
//! separate `asa-report-preview` spec. This registration makes the viewer
//! discoverable via the ViewerRegistry and activatable via `PREVIEW asa-report`.

use crate::trait_def::FileViewer;

/// Built-in viewer for ASA carriage control report files.
///
/// Interprets ASA carriage control characters (space, 0, -, 1, +) in column 1
/// and renders the report with appropriate line spacing.
pub struct AsaReportViewer {
    /// Cached display name.
    display_name: String,
}

impl AsaReportViewer {
    /// Create a new ASA Report Viewer instance.
    pub fn new() -> Self {
        Self {
            display_name: "ASA Report".to_string(),
        }
    }
}

impl Default for AsaReportViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileViewer for AsaReportViewer {
    fn viewer_key(&self) -> &str {
        "asa-report"
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        "Renders ASA carriage control report files with proper line spacing and page breaks"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["lst", "rpt", "spool"]
    }

    fn supported_mime_types(&self) -> &[&str] {
        &[]
    }

    fn can_render(&self, uri: &str, _content_sample: &[u8]) -> bool {
        let uri_lower = uri.to_lowercase();
        self.supported_extensions()
            .iter()
            .any(|ext| uri_lower.ends_with(&format!(".{ext}")))
    }

    fn render(&self, content: &[u8]) -> String {
        // Stub rendering: interpret ASA control characters
        let text = String::from_utf8_lossy(content);
        let mut output = String::new();

        for line in text.lines() {
            if line.is_empty() {
                output.push('\n');
                continue;
            }

            let (control, rest) = line.split_at(1.min(line.len()));
            match control {
                " " => output.push('\n'),
                "0" => {
                    output.push('\n');
                    output.push('\n');
                }
                "-" => {
                    output.push('\n');
                    output.push('\n');
                    output.push('\n');
                }
                "1" => output.push_str("\n--- PAGE BREAK ---\n"),
                "+" => {} // Overprint: no advance
                _ => output.push('\n'),
            }
            output.push_str(rest);
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
    fn viewer_key_is_asa_report() {
        // Validates: Requirement 4 AC 1
        let viewer = AsaReportViewer::new();
        assert_eq!(viewer.viewer_key(), "asa-report");
    }

    #[test]
    fn supported_extensions_include_lst_rpt_spool() {
        // Validates: Requirement 4 AC 1
        let viewer = AsaReportViewer::new();
        let exts = viewer.supported_extensions();
        assert!(exts.contains(&"lst"));
        assert!(exts.contains(&"rpt"));
        assert!(exts.contains(&"spool"));
    }

    #[test]
    fn can_render_matches_by_extension() {
        let viewer = AsaReportViewer::new();
        assert!(viewer.can_render("file:///report.lst", b""));
        assert!(viewer.can_render("file:///output.rpt", b""));
        assert!(!viewer.can_render("file:///data.csv", b""));
    }

    #[test]
    fn render_produces_output() {
        let viewer = AsaReportViewer::new();
        let content = b" Hello World\n0Double Spaced\n1New Page";
        let output = viewer.render(content);
        assert!(output.contains("Hello World"));
        assert!(output.contains("Double Spaced"));
        assert!(output.contains("PAGE BREAK"));
    }
}
