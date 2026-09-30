//! Minimal, dependency-free PDF writer for Collection export (CR-NR-098, Wave 3).
//!
//! We emit a hand-built PDF 1.7 document using the standard Base-14 `Courier`
//! font and real text content streams (`BT ... Tj ... ET`). The text is
//! therefore GENUINELY SELECTABLE and copy/paste-able in any compliant viewer
//! (screen-snapshot-scrm Requirement 12.6), never a rasterised image.
//!
//! Rationale for hand-writing rather than pulling a PDF crate: the target is a
//! monospaced text dump (one page per capture plus a title page, table of
//! contents and screen index). That is a small, well-specified slice of PDF
//! that we can emit deterministically and keep under our own control (mirroring
//! the hand-written YAML renderer), rather than tracking a fast-moving external
//! PDF crate's API through the `-D warnings` gate. The bytes we produce are a
//! valid PDF `Document` that the protected-PDF slice loads into `lopdf` to apply
//! owner-password encryption + permission flags.
//!
//! Layout: a title page, a table of contents / screen index page, then one page
//! per capture. Courier at 9pt on US Letter (612 x 792 pt) with a 54pt margin.
//!
//! Validates: screen-snapshot-scrm Requirement 12.4, 12.6.

use ff_screen_model::{render, RenderOptions, SnapshotFormat};

use crate::export::Masking;
use crate::model::ScreenCollection;
use crate::rules::MaskingRules;

const PAGE_W: f32 = 612.0;
const PAGE_H: f32 = 792.0;
const MARGIN: f32 = 54.0;
const FONT_SIZE: f32 = 9.0;
const LINE_H: f32 = 11.0;

/// Escape a string for a PDF literal string `( ... )`.
fn pdf_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            // Non-ASCII: replace with '?' -- Courier is a Latin base font; the
            // logical text is ASCII box-drawing/fallback in this exporter.
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => out.push('?'),
            c => out.push(c),
        }
    }
    out
}

/// Build a single page's content stream from text `lines`, top-down.
fn page_content(lines: &[String]) -> String {
    let mut s = String::new();
    s.push_str("BT\n");
    s.push_str(&format!("/F0 {FONT_SIZE} Tf\n"));
    s.push_str(&format!("{LINE_H} TL\n"));
    let start_y = PAGE_H - MARGIN;
    s.push_str(&format!("{MARGIN} {start_y} Td\n"));
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            s.push_str(&format!("({}) Tj\n", pdf_escape(line)));
        } else {
            // T* moves to the next line (uses the leading TL set above).
            s.push_str(&format!("T*\n({}) Tj\n", pdf_escape(line)));
        }
    }
    s.push_str("ET\n");
    s
}

/// Render a Collection to a self-contained PDF byte vector with selectable text.
///
/// Validates: screen-snapshot-scrm Requirement 12.4, 12.6.
pub fn export_pdf(
    collection: &ScreenCollection,
    masking: Masking,
    rules: &MaskingRules,
) -> Vec<u8> {
    // Assemble the logical text of every page first.
    let mut pages: Vec<Vec<String>> = Vec::new();

    // Title page.
    pages.push(vec![
        "FileForge Workbench -- Screen Collection".to_string(),
        String::new(),
        format!("Collection : {}", collection.name),
        format!("Created by : {}", collection.created_by),
        format!("Created    : {}", collection.created_timestamp.to_rfc3339()),
        format!("Captures   : {}", collection.len()),
    ]);

    // Table of contents / screen index.
    let mut toc = vec!["Screen Index".to_string(), String::new()];
    for cap in &collection.screens {
        let name = cap.screen_name.as_deref().unwrap_or(&cap.screen.title);
        toc.push(format!(
            "{:>4}. {}",
            cap.sequence_number,
            truncate(name, 60)
        ));
    }
    pages.push(toc);

    // One page per capture (masked per request).
    for cap in &collection.screens {
        let screen = match masking {
            Masking::On => rules.apply(&cap.screen),
            Masking::Off => cap.screen.clone(),
        };
        let body = render(
            &screen,
            SnapshotFormat::PlainText,
            RenderOptions { unicode_box: false },
        );
        let mut lines = vec![
            format!(
                "Capture {} -- {}",
                cap.sequence_number,
                cap.timestamp.to_rfc3339()
            ),
            String::new(),
        ];
        lines.extend(body.lines().map(|l| l.to_string()));
        pages.push(lines);
    }

    build_pdf(&pages)
}

/// Truncate a label to `max` characters for the index line.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(3)).collect();
        t.push_str("...");
        t
    }
}

/// Assemble the final PDF bytes from per-page text lines. Object layout:
/// 1 = Catalog, 2 = Pages, 3 = Font (Courier), then per page a Page object and a
/// Contents stream object.
fn build_pdf(pages: &[Vec<String>]) -> Vec<u8> {
    let mut objects: Vec<String> = Vec::new(); // object bodies, 1-indexed by position+1

    // Reserve ids: 1 catalog, 2 pages, 3 font. Pages/contents start at 4.
    let font_id = 3;
    let first_page_obj = 4;
    let page_count = pages.len();
    // Page object ids and content ids interleave: page i -> obj (4 + 2*i),
    // content i -> obj (5 + 2*i).
    let kids: Vec<String> = (0..page_count)
        .map(|i| format!("{} 0 R", first_page_obj + 2 * i))
        .collect();

    // 1: Catalog
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    // 2: Pages
    objects.push(format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids.join(" "),
        page_count
    ));
    // 3: Font (Base-14 Courier -- standard, no embedding needed)
    objects.push(
        "<< /Type /Font /Subtype /Type1 /BaseFont /Courier /Encoding /WinAnsiEncoding >>"
            .to_string(),
    );

    // Per page: Page object then Contents stream object.
    for (i, lines) in pages.iter().enumerate() {
        let content_id = 5 + 2 * i;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] \
             /Resources << /Font << /F0 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
        ));
        let stream = page_content(lines);
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            stream.len(),
            stream
        ));
    }

    // Serialise with a cross-reference table.
    let mut out = String::from("%PDF-1.7\n");
    let mut offsets: Vec<usize> = Vec::with_capacity(objects.len());
    for (idx, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.push_str(&format!("{} 0 obj\n{}\nendobj\n", idx + 1, body));
    }
    let xref_pos = out.len();
    let n = objects.len() + 1; // +1 for the free object 0
    out.push_str(&format!("xref\n0 {n}\n"));
    out.push_str("0000000000 65535 f \n");
    for off in &offsets {
        out.push_str(&format!("{:010} 00000 n \n", off));
    }
    out.push_str(&format!(
        "trailer\n<< /Size {n} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n"
    ));

    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ff_screen_model::{Field, ScreenModel};

    fn sample() -> ScreenCollection {
        let mut c = ScreenCollection::new("cid", "Repro", "alan", Utc::now());
        c.append_capture(
            "a",
            Utc::now(),
            ScreenModel::new("Login").with_field(Field::new("User", "alan")),
        );
        c
    }

    // Validates: Req 12.4 -- export produces a valid PDF header/trailer.
    #[test]
    fn pdf_has_header_and_trailer() {
        let bytes = export_pdf(&sample(), Masking::Off, &MaskingRules::default());
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.starts_with("%PDF-1.7"), "PDF header present");
        assert!(text.contains("%%EOF"), "PDF trailer present");
        assert!(text.contains("/Type /Catalog"));
        assert!(text.contains("startxref"));
    }

    // Validates: Req 12.6 -- screen content is REAL selectable text (a Tj text
    // operator with the value), not a rasterised image.
    #[test]
    fn pdf_contains_selectable_text_operators() {
        let bytes = export_pdf(&sample(), Masking::Off, &MaskingRules::default());
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("BT\n"), "text block present");
        assert!(text.contains(" Tj"), "text-show operator present");
        assert!(text.contains("Repro"), "collection name is selectable text");
        assert!(text.contains("alan"), "field value is selectable text");
        assert!(
            text.contains("/BaseFont /Courier"),
            "standard font, not an image"
        );
    }

    // Validates: Req 12.4 -- title page + index + one page per capture.
    #[test]
    fn pdf_page_count_is_title_index_plus_captures() {
        let bytes = export_pdf(&sample(), Masking::Off, &MaskingRules::default());
        let text = String::from_utf8_lossy(&bytes);
        // 1 capture -> 3 pages (title + index + 1). Count is in the Pages object.
        assert!(
            text.contains("/Count 3"),
            "title + index + one capture page"
        );
    }

    // Validates: Req 13.3 -- masking hides a sensitive value in the PDF text.
    #[test]
    fn pdf_masks_sensitive_when_on() {
        use ff_screen_model::FieldAttributes;
        let mut c = ScreenCollection::new("cid", "Sec", "alan", Utc::now());
        c.append_capture(
            "a",
            Utc::now(),
            ScreenModel::new("Login").with_field(
                Field::new("Password", "hunter2").with_attrs(FieldAttributes::plain().sensitive()),
            ),
        );
        let bytes = export_pdf(&c, Masking::On, &MaskingRules::default());
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("hunter2"), "sensitive value masked in PDF");
    }
}
