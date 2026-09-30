//! Hand-written PDF 1.7 writer. Zero dependencies.
//! Produces selectable, copy-pasteable text using Base-14 Courier font.

const PAGE_W: f32 = 612.0;
const PAGE_H: f32 = 792.0;
const MARGIN: f32 = 54.0;
const FONT_SIZE: f32 = 9.0;
const LINE_H: f32 = 11.0;

/// A single page of text lines.
pub struct PdfPage {
    pub lines: Vec<String>,
}

/// A document composed of pages.
pub struct PdfDocument {
    pub pages: Vec<PdfPage>,
}

impl PdfDocument {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn add_page(&mut self, lines: Vec<String>) {
        self.pages.push(PdfPage { lines });
    }

    pub fn build(&self) -> Vec<u8> {
        let page_lines: Vec<Vec<String>> = self.pages.iter().map(|p| p.lines.clone()).collect();
        build_pdf(&page_lines)
    }
}

impl Default for PdfDocument {
    fn default() -> Self {
        Self::new()
    }
}

fn pdf_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => out.push('?'),
            c => out.push(c),
        }
    }
    out
}

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
            s.push_str(&format!("T*\n({}) Tj\n", pdf_escape(line)));
        }
    }
    s.push_str("ET\n");
    s
}

pub fn build_pdf(pages: &[Vec<String>]) -> Vec<u8> {
    let mut objects: Vec<String> = Vec::new();

    let font_id = 3;
    let first_page_obj = 4;
    let page_count = pages.len();
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
    // 3: Font
    objects.push(
        "<< /Type /Font /Subtype /Type1 /BaseFont /Courier /Encoding /WinAnsiEncoding >>"
            .to_string(),
    );

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

    let mut out = String::from("%PDF-1.7\n");
    let mut offsets: Vec<usize> = Vec::with_capacity(objects.len());
    for (idx, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.push_str(&format!("{} 0 obj\n{}\nendobj\n", idx + 1, body));
    }
    let xref_pos = out.len();
    let n = objects.len() + 1;
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

    #[test]
    fn pdf_has_valid_header_and_trailer() {
        let pages = vec![vec!["Hello, World!".to_string()]];
        let bytes = build_pdf(&pages);
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.starts_with("%PDF-1.7"));
        assert!(text.contains("%%EOF"));
        assert!(text.contains("/Type /Catalog"));
        assert!(text.contains("startxref"));
    }

    #[test]
    fn pdf_contains_selectable_text() {
        let pages = vec![vec!["Selectable text".to_string()]];
        let bytes = build_pdf(&pages);
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("BT\n"));
        assert!(text.contains(" Tj"));
        assert!(text.contains("Selectable text"));
        assert!(text.contains("/BaseFont /Courier"));
    }

    #[test]
    fn pdf_document_builder() {
        let mut doc = PdfDocument::new();
        doc.add_page(vec!["Page one".to_string()]);
        doc.add_page(vec!["Page two".to_string()]);
        let bytes = doc.build();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("/Count 2"));
    }
}
