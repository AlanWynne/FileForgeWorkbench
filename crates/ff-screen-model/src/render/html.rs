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
    out.push_str(&format!("  <h2>{}</h2>\n", escape(&model.title)));

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
}
