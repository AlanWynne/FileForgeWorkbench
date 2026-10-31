//! Markdown renderer: screen text in a fenced code block, plus each table as a
//! Markdown table.
//!
//! Validates: screen-snapshot-scrm Requirement 4.4.

use crate::model::ScreenModel;
use crate::render::{text, RenderOptions};

/// Render `model` as Markdown.
pub fn render_markdown(model: &ScreenModel) -> String {
    let mut out = String::new();
    // Heading from the title.
    if !model.title.is_empty() {
        out.push_str(&format!("# {}\n\n", model.title));
    }

    // The framed screen text as a fenced code block (Requirement 4.4, 5.1). Use
    // the UNICODE box-drawing frame inside the code fence: a Markdown-aware
    // target renders the fenced block monospaced, so the U+2500-family glyphs
    // line up into a proper box rather than the ASCII +/-/| fallback.
    let body = text::render_plain(model, RenderOptions { unicode_box: true });
    out.push_str("```\n");
    out.push_str(&body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n");

    // Fields as a property table (in addition to the code block) so the values
    // are also copy-pasteable as structured data.
    if !model.fields.is_empty() {
        out.push_str("\n| Property | Value |\n|----------|-------|\n");
        for f in &model.fields {
            let label = if f.label.is_empty() {
                "(field)"
            } else {
                &f.label
            };
            out.push_str(&format!(
                "| {} | {} |\n",
                escape_cell(label),
                escape_cell(&f.value)
            ));
        }
    }

    // Each table element as its own Markdown table.
    for table in &model.tables {
        out.push('\n');
        if let Some(cap) = &table.caption {
            out.push_str(&format!("**{}**\n\n", cap));
        }
        if !table.headers.is_empty() {
            out.push_str(&format!(
                "| {} |\n",
                table
                    .headers
                    .iter()
                    .map(|h| escape_cell(h))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
            out.push_str(&format!(
                "|{}|\n",
                table
                    .headers
                    .iter()
                    .map(|_| "------")
                    .collect::<Vec<_>>()
                    .join("|")
            ));
        }
        for row in &table.rows {
            out.push_str(&format!(
                "| {} |\n",
                row.iter()
                    .map(|c| escape_cell(c))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
    }

    out
}

/// Escape a Markdown table cell (pipes would break the table).
fn escape_cell(s: &str) -> String {
    s.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Field, Table};

    // Validates: Requirement 4.4 -- fenced code block + property table + title.
    #[test]
    fn markdown_has_heading_code_block_and_table() {
        let m = ScreenModel::new("Dataset Properties").with_field(Field::new("RECFM", "FB"));
        let out = render_markdown(&m);
        assert!(out.contains("# Dataset Properties"));
        assert!(out.contains("```"));
        assert!(out.contains("| Property | Value |"));
        assert!(out.contains("| RECFM | FB |"));
    }

    // Validates: screen-snapshot-scrm Req 4.4, 5.1 -- the fenced code block
    // frames the screen with UNICODE box-drawing glyphs (U+2500 family), not the
    // ASCII +/-/| fallback, so a Markdown-aware target renders a proper box. The
    // expected glyphs are built from char constants (no raw box glyph in source).
    #[test]
    fn fenced_block_uses_unicode_box_drawing_not_ascii() {
        let m = ScreenModel::new("Dataset Properties").with_field(Field::new("RECFM", "FB"));
        let out = render_markdown(&m);

        // Isolate the fenced code block body (between the first and second ```).
        let mut parts = out.split("```");
        let _before = parts.next();
        let body = parts.next().expect("a fenced code block is present");

        // The frame uses Unicode box-drawing glyphs.
        let top_left = '\u{250C}';
        let horizontal = '\u{2500}';
        let vertical = '\u{2502}';
        assert!(
            body.contains(top_left),
            "fenced body frames with the Unicode top-left corner U+250C"
        );
        assert!(
            body.contains(horizontal),
            "fenced body frames with the Unicode horizontal rule U+2500"
        );
        assert!(
            body.contains(vertical),
            "fenced body frames with the Unicode vertical rule U+2502"
        );

        // The ASCII frame glyphs do NOT appear in the fenced body. The '-' check
        // is scoped to the body (the property-table separator outside the fence
        // legitimately uses '-').
        assert!(
            !body.contains('+'),
            "no ASCII '+' corner glyph in the Unicode-framed fence"
        );
        assert!(
            !body.contains('|'),
            "no ASCII '|' vertical glyph in the Unicode-framed fence"
        );
    }

    // Validates: Requirement 4.4 -- a table element becomes a Markdown table.
    #[test]
    fn table_element_renders_as_markdown_table() {
        let mut m = ScreenModel::new("Browser");
        m.tables.push(Table {
            caption: Some("Datasets".to_string()),
            headers: vec!["NAME".to_string(), "RECFM".to_string()],
            rows: vec![vec!["CUST.MASTER".to_string(), "FB".to_string()]],
        });
        let out = render_markdown(&m);
        assert!(out.contains("**Datasets**"));
        assert!(out.contains("| NAME | RECFM |"));
        assert!(out.contains("| CUST.MASTER | FB |"));
    }

    // Validates: Requirement 4.4 -- a pipe in a value is escaped.
    #[test]
    fn pipe_in_value_is_escaped() {
        let m = ScreenModel::new("T").with_field(Field::new("K", "a|b"));
        let out = render_markdown(&m);
        assert!(out.contains("a\\|b"));
    }
}
