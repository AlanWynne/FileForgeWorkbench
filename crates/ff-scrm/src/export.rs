//! Collection exporters: plain text, Markdown, and HTML. Each renders every
//! capture's logical screen (via ff-screen-model) so exported screen content
//! remains selectable text (never a raster).
//!
//! Validates: screen-snapshot-scrm Requirement 12.1, 12.2, 12.3, 1.4.

use ff_screen_model::{render, RenderOptions, ScreenModel, SnapshotFormat};

use crate::model::ScreenCollection;
use crate::rules::MaskingRules;

/// Whether an export applies masking.
///
/// Validates: Requirement 13.3 (with or without masking).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Masking {
    /// Mask sensitive/masked fields.
    On,
    /// Do not mask.
    Off,
}

/// Apply masking to a capture's screen if requested.
fn screen_for(model: &ScreenModel, masking: Masking, rules: &MaskingRules) -> ScreenModel {
    match masking {
        Masking::On => rules.apply(model),
        Masking::Off => model.clone(),
    }
}

/// Export a collection as plain text: each capture framed, separated by a rule.
///
/// Validates: Requirement 12.1.
pub fn export_text(
    collection: &ScreenCollection,
    masking: Masking,
    rules: &MaskingRules,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("Collection: {}\n", collection.name));
    out.push_str(&format!("Captures: {}\n\n", collection.len()));
    for cap in &collection.screens {
        out.push_str(&format!(
            "--- Capture {} ({}) ---\n",
            cap.sequence_number,
            cap.timestamp.to_rfc3339()
        ));
        let screen = screen_for(&cap.screen, masking, rules);
        out.push_str(&render(
            &screen,
            SnapshotFormat::PlainText,
            RenderOptions::default(),
        ));
        out.push('\n');
    }
    out
}

/// Export a collection as Markdown: a heading per capture, each screen a fenced
/// block (via the Markdown renderer).
///
/// Validates: Requirement 12.2.
pub fn export_markdown(
    collection: &ScreenCollection,
    masking: Masking,
    rules: &MaskingRules,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", collection.name));
    for cap in &collection.screens {
        out.push_str(&format!(
            "## Capture {} -- {}\n\n",
            cap.sequence_number,
            cap.timestamp.to_rfc3339()
        ));
        let screen = screen_for(&cap.screen, masking, rules);
        out.push_str(&render(
            &screen,
            SnapshotFormat::Markdown,
            RenderOptions::default(),
        ));
        out.push('\n');
    }
    out
}

/// Export a collection as an HTML document with each capture as a section.
///
/// Validates: Requirement 12.3.
pub fn export_html(
    collection: &ScreenCollection,
    masking: Masking,
    rules: &MaskingRules,
) -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\">\n");
    out.push_str(&format!(
        "<title>{}</title></head><body>\n",
        html_escape(&collection.name)
    ));
    out.push_str(&format!("<h1>{}</h1>\n", html_escape(&collection.name)));
    for cap in &collection.screens {
        out.push_str(&format!(
            "<section><h2>Capture {} -- {}</h2>\n",
            cap.sequence_number,
            html_escape(&cap.timestamp.to_rfc3339())
        ));
        let screen = screen_for(&cap.screen, masking, rules);
        out.push_str(&render(
            &screen,
            SnapshotFormat::Html,
            RenderOptions::default(),
        ));
        out.push_str("</section>\n");
    }
    out.push_str("</body></html>\n");
    out
}

/// Minimal HTML escape for the document chrome (the screen bodies are escaped by
/// the HTML renderer itself).
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use ff_screen_model::{Field, FieldAttributes};

    fn ts() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).unwrap()
    }

    fn sample() -> ScreenCollection {
        let mut c = ScreenCollection::new("c", "Repro", "alan", ts());
        c.append_capture(
            "a",
            ts(),
            ScreenModel::new("Login").with_field(
                Field::new("Password", "hunter2").with_attrs(FieldAttributes::plain().sensitive()),
            ),
        );
        c
    }

    // Validates: Requirement 12.1 -- text export includes framed screen text.
    #[test]
    fn text_export_includes_capture_and_value() {
        let out = export_text(&sample(), Masking::Off, &MaskingRules::default());
        assert!(out.contains("Collection: Repro"));
        assert!(out.contains("Capture 1"));
        assert!(out.contains("hunter2"));
    }

    // Validates: Requirement 13.3 -- masking on hides the sensitive value.
    #[test]
    fn text_export_masks_when_on() {
        let out = export_text(&sample(), Masking::On, &MaskingRules::default());
        assert!(!out.contains("hunter2"));
        assert!(out.contains("********"));
    }

    // Validates: Requirement 12.2 -- markdown export has a per-capture heading.
    #[test]
    fn markdown_export_has_heading() {
        let out = export_markdown(&sample(), Masking::Off, &MaskingRules::default());
        assert!(out.contains("# Repro"));
        assert!(out.contains("## Capture 1"));
    }

    // Validates: Requirement 12.3 -- html export is a full document.
    #[test]
    fn html_export_is_a_document() {
        let out = export_html(&sample(), Masking::Off, &MaskingRules::default());
        assert!(out.contains("<!DOCTYPE html>"));
        assert!(out.contains("<h1>Repro</h1>"));
        assert!(out.contains("Capture 1"));
    }
}
