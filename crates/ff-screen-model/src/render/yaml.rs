//! YAML ("AI") renderer: a structured, machine-readable view of the screen an
//! AI agent can consume more easily than a screenshot. Hand-emitted so the crate
//! stays dependency-light; all scalars are quoted to keep them unambiguous.
//!
//! Validates: screen-snapshot-scrm Requirement 4.6.

use crate::model::ScreenModel;

/// Quote a scalar for YAML, escaping backslashes and double quotes.
fn q(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Render `model` as a structured YAML document.
pub fn render_yaml(model: &ScreenModel) -> String {
    let mut out = String::new();
    out.push_str("screen:\n");
    out.push_str(&format!("  title: {}\n", q(&model.title)));

    out.push_str("  fields:\n");
    if model.fields.is_empty() {
        out.push_str("    []\n");
    } else {
        for f in &model.fields {
            out.push_str(&format!("    - name: {}\n", q(&f.label)));
            out.push_str(&format!("      value: {}\n", q(&f.value)));
            if f.attrs.sensitive {
                out.push_str("      sensitive: true\n");
            }
        }
    }

    if !model.tables.is_empty() {
        out.push_str("  tables:\n");
        for t in &model.tables {
            out.push_str(&format!(
                "    - headers: [{}]\n",
                t.headers
                    .iter()
                    .map(|h| q(h))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            out.push_str("      rows:\n");
            for row in &t.rows {
                out.push_str(&format!(
                    "        - [{}]\n",
                    row.iter().map(|c| q(c)).collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }

    if !model.buttons.is_empty() {
        out.push_str("  buttons:\n");
        for b in &model.buttons {
            out.push_str(&format!("    - {}\n", q(b)));
        }
    }

    if !model.messages.is_empty() {
        out.push_str("  messages:\n");
        for m in &model.messages {
            out.push_str(&format!("    - {}\n", q(m)));
        }
    }

    out.push_str("  cursor:\n");
    out.push_str(&format!("    row: {}\n", model.cursor.row));
    out.push_str(&format!("    column: {}\n", model.cursor.column));
    if let Some(field) = &model.cursor.field_label {
        out.push_str(&format!("    field: {}\n", q(field)));
    }

    if !model.command_line.is_empty() {
        out.push_str(&format!("  command_line: {}\n", q(&model.command_line)));
    }
    if !model.status_bar.text.is_empty() {
        out.push_str(&format!("  status: {}\n", q(&model.status_bar.text)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Field, FieldAttributes};

    // Validates: Requirement 4.6 -- structured title/fields/cursor.
    #[test]
    fn yaml_has_title_fields_and_cursor() {
        let m = ScreenModel::new("Dataset Properties")
            .with_field(Field::new("RECFM", "FB"))
            .with_button("Save");
        let out = render_yaml(&m);
        assert!(out.contains("screen:"));
        assert!(out.contains("title: \"Dataset Properties\""));
        assert!(out.contains("- name: \"RECFM\""));
        assert!(out.contains("value: \"FB\""));
        assert!(out.contains("buttons:"));
        assert!(out.contains("- \"Save\""));
        assert!(out.contains("cursor:"));
    }

    // Validates: Requirement 4.6 + 13.5 -- sensitive flag emitted.
    #[test]
    fn sensitive_field_flag_emitted() {
        let m = ScreenModel::new("T").with_field(
            Field::new("Password", "hunter2").with_attrs(FieldAttributes::plain().sensitive()),
        );
        let out = render_yaml(&m);
        assert!(out.contains("sensitive: true"));
    }
}
