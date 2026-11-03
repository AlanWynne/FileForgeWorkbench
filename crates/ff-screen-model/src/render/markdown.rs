//! Markdown renderer: screen text in a fenced code block, plus each table as a
//! Markdown table.
//!
//! Validates: screen-snapshot-scrm Requirement 4.4.

use crate::model::{ScreenModel, Table};
use crate::render::{text, RenderOptions};

/// Render `model` as Markdown.
pub fn render_markdown(model: &ScreenModel) -> String {
    let mut out = String::new();
    // Heading from the title.
    if !model.title.is_empty() {
        out.push_str(&format!("# {}\n\n", model.title));
    }

    // FLAVOR-SPECIFIC per-group sections (master-screen-directive Req 4.4c,
    // screen-snapshot-scrm Req 28.1/28.4/28.6): when a Context supplies
    // `markdown_sections` (the Config panel does), the Markdown capture emits the
    // box-drawn body FIRST (the fenced Unicode-box block) and THEN those sections
    // -- each an optional `### heading` followed by a GFM table. So the plain-text
    // (`CF_UNICODETEXT`) flavor carries BOTH the box layout AND the property/value
    // tables (box-before-tables ordering, Req 28.4), symmetric with the CF_HTML
    // flavor. A Context that leaves the field empty falls through to the generic
    // Markdown path below, unchanged (Req 28.7).
    if !model.markdown_sections.is_empty() {
        push_fenced_body(model, &mut out);
        render_markdown_sections(model, &mut out);
        return out;
    }

    // The framed screen text as a fenced code block (Requirement 4.4, 5.1).
    push_fenced_body(model, &mut out);

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
        push_gfm_table(&mut out, table);
    }

    out
}

/// Push the box-drawn screen body as a fenced code block. Uses the UNICODE
/// box-drawing frame inside the code fence: a Markdown-aware target renders the
/// fenced block monospaced, so the U+2500-family glyphs line up into a proper
/// box rather than the ASCII +/-/| fallback. Shared by the generic Markdown path
/// and the `markdown_sections` path so the plain-text flavor carries the box
/// layout in both (screen-snapshot-scrm Req 28.1, 28.4).
fn push_fenced_body(model: &ScreenModel, out: &mut String) {
    let body = text::render_plain(model, RenderOptions { unicode_box: true });
    out.push_str("```\n");
    out.push_str(&body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n");
}

/// Render the FLAVOR-SPECIFIC `markdown_sections` (master-screen-directive Req
/// 4.4c): each section is an optional `### heading` followed by a GFM table.
/// Used by the Config panel to emit one heading + 2-column Property/Value table
/// per namespace group (and a heading-less intro table for the header fields).
fn render_markdown_sections(model: &ScreenModel, out: &mut String) {
    for (idx, section) in model.markdown_sections.iter().enumerate() {
        if let Some(heading) = &section.heading {
            // A blank line before every heading except an opening one that
            // immediately follows the title's own blank line.
            if idx > 0 || !out.ends_with("\n\n") {
                out.push('\n');
            }
            out.push_str(&format!("### {}\n\n", heading));
        } else {
            // Separate a heading-less table from whatever precedes it (the title
            // blank line, the fenced box body, or a prior section) with one blank
            // line, unless the preceding content already ended with one.
            if !out.ends_with("\n\n") {
                out.push('\n');
            }
        }
        push_gfm_table(out, &section.table);
    }
}

/// Emit a [`Table`] as a GFM table (header row, separator, data rows) with every
/// cell escaped. A table with no headers emits only its data rows.
fn push_gfm_table(out: &mut String, table: &Table) {
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

/// Escape a Markdown table cell (pipes would break the table).
fn escape_cell(s: &str) -> String {
    s.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Field;

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

    use crate::model::{MarkdownSection, Table};

    /// A Config-like model: a box-drawn body (`body_lines`) PLUS per-group
    /// `markdown_sections` (a heading-less intro table + one `[Group]`
    /// heading + Property/Value table).
    fn config_like_model() -> ScreenModel {
        let mut m = ScreenModel::new("Configuration");
        m.body_lines = vec![
            "Filter:                              :".to_string(),
            "Unsaved edits                        : no".to_string(),
            "[Editor]                             : expanded".to_string(),
            "    editor.tab_size                  : 4".to_string(),
        ];
        m.markdown_sections = vec![
            MarkdownSection {
                heading: None,
                table: Table {
                    caption: None,
                    headers: vec!["Property".to_string(), "Value".to_string()],
                    rows: vec![
                        vec!["Filter:".to_string(), String::new()],
                        vec!["Unsaved edits".to_string(), "no".to_string()],
                    ],
                },
            },
            MarkdownSection {
                heading: Some("[Editor]  (expanded)".to_string()),
                table: Table {
                    caption: None,
                    headers: vec!["Property".to_string(), "Value".to_string()],
                    rows: vec![vec!["editor.tab_size".to_string(), "4".to_string()]],
                },
            },
        ];
        m
    }

    // Validates: screen-snapshot-scrm Requirement 28.1, 28.4 -- a
    // `markdown_sections` model's Markdown render carries BOTH a Unicode box
    // glyph (U+250C family) AND a `| Property | Value |` table header, with the
    // box block BEFORE the first table (asserted by byte index).
    #[test]
    fn markdown_sections_render_box_then_tables() {
        let out = render_markdown(&config_like_model());

        let box_glyph = '\u{250C}'; // top-left corner of the fenced box body
        let box_idx = out
            .find(box_glyph)
            .expect("the box-drawn body is present in the plain-text flavor");
        let table_idx = out
            .find("| Property | Value |")
            .expect("the property/value Markdown table header is present");
        assert!(
            box_idx < table_idx,
            "the box-drawn layout comes BEFORE the property/value tables (Req 28.4)"
        );

        // The per-group heading and a data row are present (Req 28.6).
        assert!(out.contains("### [Editor]  (expanded)"));
        assert!(out.contains("| editor.tab_size | 4 |"));
    }

    // Validates: screen-snapshot-scrm Requirement 28.7, 28.8 -- a model with NO
    // `markdown_sections` and no tabular content still renders box-only (the
    // generic fenced path, unchanged).
    #[test]
    fn no_sections_no_tables_renders_box_only() {
        let mut m = ScreenModel::new("Plain");
        m.body_lines = vec!["just a line".to_string()];
        let out = render_markdown(&m);
        assert!(out.contains('\u{250C}'), "box frame present");
        assert!(
            !out.contains("| Property | Value |"),
            "no property/value table when there is no tabular content"
        );
    }

    // Validates: screen-snapshot-scrm Requirement 28.7 -- a model with only
    // `fields` (no `markdown_sections`) still renders box + the fields property
    // table (the generic path already carries both representations).
    #[test]
    fn fields_only_model_renders_box_and_property_table() {
        let m = ScreenModel::new("Props").with_field(Field::new("RECFM", "FB"));
        let out = render_markdown(&m);
        let box_idx = out.find('\u{250C}').expect("box frame present");
        let table_idx = out
            .find("| Property | Value |")
            .expect("fields property table present");
        assert!(box_idx < table_idx, "box before the fields table");
        assert!(out.contains("| RECFM | FB |"));
    }
}
