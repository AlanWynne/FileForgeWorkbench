//! HTML renderer: preserves colours and layout. Emitted as a self-contained
//! fragment (no <script>), with field colours as inline styles.
//!
//! Validates: screen-snapshot-scrm Requirement 4.5, 5.3.

use crate::model::{Colour, Field, ScreenModel};

/// Map a [`Colour`] to a CSS colour name, or `None` for the default.
fn css_colour(colour: Colour) -> Option<&'static str> {
    match colour {
        Colour::Default => None,
        Colour::Black => Some("black"),
        Colour::Red => Some("red"),
        Colour::Green => Some("green"),
        Colour::Yellow => Some("olive"),
        Colour::Blue => Some("blue"),
        Colour::Magenta => Some("magenta"),
        Colour::Cyan => Some("teal"),
        Colour::White => Some("white"),
    }
}

/// Escape text for safe inclusion in HTML.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Wrap a field value in a span carrying its colour/highlight, if any.
fn field_span(field: &Field) -> String {
    let mut styles: Vec<String> = Vec::new();
    if let Some(c) = css_colour(field.attrs.colour) {
        styles.push(format!("color:{c}"));
    }
    if field.attrs.highlight {
        styles.push("font-weight:bold".to_string());
    }
    let value = escape(&field.value);
    if styles.is_empty() {
        value
    } else {
        format!("<span style=\"{}\">{}</span>", styles.join(";"), value)
    }
}

/// Render `model` as an HTML fragment.
pub fn render_html(model: &ScreenModel) -> String {
    let mut out = String::new();
    out.push_str("<div class=\"ffwb-screen\">\n");
    // Header chrome (title bar, menu bar, tab header) -- Requirement 3.9.
    for h in &model.header_lines {
        out.push_str(&format!("  <p class=\"ffwb-header\">{}</p>\n", escape(h)));
    }
    out.push_str(&format!("  <h2>{}</h2>\n", escape(&model.title)));

    // Pre-composed verbatim body lines (the layout-owning menu/POM path) --
    // emitted as a monospaced block so the two-column layout is preserved
    // (screen-snapshot-scrm Req 3.8-3.13).
    if !model.body_lines.is_empty() {
        out.push_str("  <pre class=\"ffwb-body\">");
        for b in &model.body_lines {
            out.push_str(&escape(b));
            out.push('\n');
        }
        out.push_str("</pre>\n");
    }

    if !model.fields.is_empty() {
        out.push_str("  <table class=\"ffwb-fields\">\n");
        for f in &model.fields {
            let label = if f.label.is_empty() {
                "(field)"
            } else {
                &f.label
            };
            out.push_str(&format!(
                "    <tr><th>{}</th><td>{}</td></tr>\n",
                escape(label),
                field_span(f)
            ));
        }
        out.push_str("  </table>\n");
    }

    for table in &model.tables {
        out.push_str("  <table class=\"ffwb-table\">\n");
        if let Some(cap) = &table.caption {
            out.push_str(&format!("    <caption>{}</caption>\n", escape(cap)));
        }
        if !table.headers.is_empty() {
            out.push_str("    <tr>");
            for h in &table.headers {
                out.push_str(&format!("<th>{}</th>", escape(h)));
            }
            out.push_str("</tr>\n");
        }
        for row in &table.rows {
            out.push_str("    <tr>");
            for c in row {
                out.push_str(&format!("<td>{}</td>", escape(c)));
            }
            out.push_str("</tr>\n");
        }
        out.push_str("  </table>\n");
    }

    for m in &model.messages {
        out.push_str(&format!("  <p class=\"ffwb-message\">{}</p>\n", escape(m)));
    }

    if !model.buttons.is_empty() {
        out.push_str("  <p class=\"ffwb-buttons\">");
        for b in &model.buttons {
            out.push_str(&format!("<button>{}</button> ", escape(b)));
        }
        out.push_str("</p>\n");
    }

    // Footer chrome (calendar, PF-key footer) -- Requirement 3.9, 3.10.
    for f in &model.footer_lines {
        out.push_str(&format!("  <p class=\"ffwb-footer\">{}</p>\n", escape(f)));
    }

    if !model.command_line.is_empty() {
        out.push_str(&format!(
            "  <p class=\"ffwb-command\">Command ===&gt; {}</p>\n",
            escape(&model.command_line)
        ));
    }
    if !model.status_bar.text.is_empty() {
        out.push_str(&format!(
            "  <p class=\"ffwb-status\">{}</p>\n",
            escape(&model.status_bar.text)
        ));
    }
    out.push_str("</div>\n");
    out
}

/// Render ONLY the tabular property/value content of `model` as HTML `<table>`
/// markup, for the composite CF_HTML clipboard flavor (screen-snapshot-scrm Req
/// 28.2, 28.6, 28.7). This is the HTML structural equivalent of the Markdown
/// flavor's property/value tables: it derives from the SAME logical source the
/// Markdown renderer uses -- `markdown_sections` when present, else `fields` and
/// `tables` -- so both flavors show identical data and group structure.
///
/// It deliberately renders ONLY the tables (no title, body, chrome, or command
/// line) because the composite already carries the box-drawn layout as the
/// monospace `<span>` paragraphs; this appends the tables AFTER that box
/// (box-then-tables order, Req 28.4). Returns an EMPTY string when the model has
/// no tabular content, so a box-only Context's composite is unchanged (Req 28.7,
/// 28.3).
///
/// Each `markdown_sections` group emits a `<table>` whose optional heading is a
/// `<caption>`; a 2-column `Property`/`Value` header row; and one `<tr>` per
/// data row. When `markdown_sections` is empty the `fields` property table and
/// each `tables` element are emitted instead, mirroring the Markdown path.
///
/// Validates: screen-snapshot-scrm Requirement 28.2, 28.6, 28.7.
pub fn render_tables_html(model: &ScreenModel) -> String {
    let mut out = String::new();
    if !model.markdown_sections.is_empty() {
        for section in &model.markdown_sections {
            push_data_table(&mut out, section.heading.as_deref(), &section.table);
        }
        return out;
    }

    // No markdown_sections: mirror the Markdown generic path -- a fields property
    // table (if any) then each table element.
    if !model.fields.is_empty() {
        out.push_str("  <table class=\"ffwb-table\">\n");
        out.push_str("    <tr><th>Property</th><th>Value</th></tr>\n");
        for f in &model.fields {
            let label = if f.label.is_empty() {
                "(field)"
            } else {
                &f.label
            };
            out.push_str(&format!(
                "    <tr><td>{}</td><td>{}</td></tr>\n",
                escape(label),
                field_span(f)
            ));
        }
        out.push_str("  </table>\n");
    }
    for table in &model.tables {
        push_data_table(&mut out, None, table);
    }
    out
}

/// Emit a single [`crate::model::Table`] as an HTML `<table class="ffwb-table">`
/// with an optional `<caption>` (the group heading), a header `<tr>` of `<th>`,
/// and one data `<tr>` of `<td>` per row. All cells are HTML-escaped. Shared by
/// [`render_tables_html`] so every table has identical markup (Req 28.6).
fn push_data_table(out: &mut String, heading: Option<&str>, table: &crate::model::Table) {
    out.push_str("  <table class=\"ffwb-table\">\n");
    if let Some(h) = heading {
        out.push_str(&format!("    <caption>{}</caption>\n", escape(h)));
    } else if let Some(cap) = &table.caption {
        out.push_str(&format!("    <caption>{}</caption>\n", escape(cap)));
    }
    if !table.headers.is_empty() {
        out.push_str("    <tr>");
        for th in &table.headers {
            out.push_str(&format!("<th>{}</th>", escape(th)));
        }
        out.push_str("</tr>\n");
    }
    for row in &table.rows {
        out.push_str("    <tr>");
        for c in row {
            out.push_str(&format!("<td>{}</td>", escape(c)));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("  </table>\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FieldAttributes;

    // Validates: Requirement 4.5 -- title, field value, and structure present.
    #[test]
    fn html_contains_title_and_field() {
        let m = ScreenModel::new("Props").with_field(Field::new("RECFM", "FB"));
        let out = render_html(&m);
        assert!(out.contains("<h2>Props</h2>"));
        assert!(out.contains("<th>RECFM</th>"));
        assert!(out.contains("FB"));
    }

    // Validates: Requirement 5.3 -- colour rendered as an inline style.
    #[test]
    fn coloured_field_has_inline_style() {
        let attrs = FieldAttributes {
            colour: Colour::Red,
            ..Default::default()
        };
        let m = ScreenModel::new("T").with_field(Field::new("ERR", "boom").with_attrs(attrs));
        let out = render_html(&m);
        assert!(out.contains("color:red"));
        assert!(out.contains("boom"));
    }

    // Validates: Requirement 4.5 -- HTML special characters are escaped.
    #[test]
    fn special_characters_are_escaped() {
        let m = ScreenModel::new("T").with_field(Field::new("K", "a<b>&c"));
        let out = render_html(&m);
        assert!(out.contains("a&lt;b&gt;&amp;c"));
    }

    use crate::model::{MarkdownSection, Table};

    // Validates: screen-snapshot-scrm Requirement 30.12 -- the HTML renderer
    // emits each table cell as a `<td>`, carrying the provider cell conventions
    // `[value v]` and `____` verbatim (escaped as HTML text).
    #[test]
    fn html_table_cells_carry_cell_conventions() {
        let mut m = ScreenModel::new("Alias Editor");
        m.tables.push(Table {
            caption: None,
            headers: vec!["CE".to_string(), "Command".to_string()],
            rows: vec![
                vec!["[FFCMD v]".to_string(), "SNAPSHOT".to_string()],
                vec!["[ v]".to_string(), "____".to_string()],
            ],
        });
        let out = render_html(&m);
        assert!(
            out.contains("<td>[FFCMD v]</td>"),
            "populated dropdown cell:\n{out}"
        );
        assert!(out.contains("<td>[ v]</td>"), "empty dropdown cell:\n{out}");
        assert!(
            out.contains("<td>____</td>"),
            "empty editable field cell:\n{out}"
        );
        assert!(out.contains("<td>SNAPSHOT</td>"), "a value cell:\n{out}");
    }

    // Validates: screen-snapshot-scrm Requirement 28.2, 28.6 -- the HTML-table
    // emitter renders `markdown_sections` as `<table>`/`<tr>`/`<td>` with the
    // per-group heading as a `<caption>` and a `Property`/`Value` header row.
    #[test]
    fn render_tables_html_emits_sections_as_tables() {
        let mut m = ScreenModel::new("Configuration");
        m.markdown_sections = vec![
            MarkdownSection {
                heading: None,
                table: Table {
                    caption: None,
                    headers: vec!["Property".to_string(), "Value".to_string()],
                    rows: vec![vec!["Unsaved edits".to_string(), "no".to_string()]],
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
        let out = render_tables_html(&m);
        assert!(out.contains("<table class=\"ffwb-table\">"));
        assert!(out.contains("<caption>[Editor]  (expanded)</caption>"));
        assert!(out.contains("<th>Property</th><th>Value</th>"));
        assert!(out.contains("<td>editor.tab_size</td><td>4</td>"));
        assert!(out.contains("<td>Unsaved edits</td><td>no</td>"));
    }

    // Validates: screen-snapshot-scrm Requirement 28.7 -- a model with no
    // tabular content yields an EMPTY tables fragment (box-only composite is
    // unchanged).
    #[test]
    fn render_tables_html_empty_when_no_tables() {
        let mut m = ScreenModel::new("Plain");
        m.body_lines = vec!["just a line".to_string()];
        assert_eq!(render_tables_html(&m), "");
    }

    // Validates: screen-snapshot-scrm Requirement 28.7 -- with no
    // `markdown_sections`, the emitter falls back to the `fields` property table
    // and `tables` elements (mirroring the Markdown generic path).
    #[test]
    fn render_tables_html_falls_back_to_fields_and_tables() {
        let mut m = ScreenModel::new("Props").with_field(Field::new("RECFM", "FB"));
        m.tables.push(Table {
            caption: Some("Datasets".to_string()),
            headers: vec!["NAME".to_string()],
            rows: vec![vec!["CUST.MASTER".to_string()]],
        });
        let out = render_tables_html(&m);
        assert!(out.contains("<tr><th>Property</th><th>Value</th></tr>"));
        assert!(out.contains("<td>RECFM</td><td>FB</td>"));
        assert!(out.contains("<caption>Datasets</caption>"));
        assert!(out.contains("<th>NAME</th>"));
        assert!(out.contains("<td>CUST.MASTER</td>"));
    }
}
