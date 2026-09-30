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

    // The framed screen text as a fenced code block (Requirement 4.4). Use the
    // ASCII frame inside code fences for maximum portability.
    let body = text::render_plain(model, RenderOptions { unicode_box: false });
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
