//! ANSI renderer: the plain-text framing plus SGR colour escape sequences for
//! coloured/highlighted fields. Colours remain visible when pasted into a
//! terminal and readable when pasted into a plain editor.
//!
//! Validates: screen-snapshot-scrm Requirement 4.3, 5.3.

use crate::model::{Colour, Field, ScreenModel};
use crate::render::{text, RenderOptions};

const RESET: &str = "\x1b[0m";

/// Map a [`Colour`] to its SGR foreground code, or `None` for the default.
fn sgr_code(colour: Colour) -> Option<&'static str> {
    match colour {
        Colour::Default => None,
        Colour::Black => Some("30"),
        Colour::Red => Some("31"),
        Colour::Green => Some("32"),
        Colour::Yellow => Some("33"),
        Colour::Blue => Some("34"),
        Colour::Magenta => Some("35"),
        Colour::Cyan => Some("36"),
        Colour::White => Some("37"),
    }
}

/// Wrap `text` in an SGR sequence for the field's colour/highlight, if any.
fn colourise(field: &Field, text: &str) -> String {
    let mut codes: Vec<&str> = Vec::new();
    if field.attrs.highlight {
        codes.push("1"); // bold / intensified
    }
    if let Some(c) = sgr_code(field.attrs.colour) {
        codes.push(c);
    }
    if codes.is_empty() {
        text.to_string()
    } else {
        format!("\x1b[{}m{}{}", codes.join(";"), text, RESET)
    }
}

/// Render `model` as framed text with an ANSI colour legend for coloured fields.
///
/// The frame is the same plain-text layout (so it stays readable without ANSI
/// support); coloured field values are additionally emitted with SGR codes in a
/// short legend below the frame, keeping the framed body copy/paste-clean while
/// still carrying colour (Requirement 5.3).
pub fn render_ansi(model: &ScreenModel, options: RenderOptions) -> String {
    let mut out = text::render_plain(model, options);
    let coloured: Vec<&Field> = model
        .fields
        .iter()
        .filter(|f| f.attrs.highlight || sgr_code(f.attrs.colour).is_some())
        .collect();
    if !coloured.is_empty() {
        out.push('\n');
        for f in coloured {
            let label = if f.label.is_empty() {
                "(field)"
            } else {
                &f.label
            };
            out.push_str(&format!("{}: {}\n", label, colourise(f, &f.value)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FieldAttributes;

    // Validates: Requirement 5.3 -- coloured field emits an SGR escape, and the
    // plain value text is still present (readable without ANSI support).
    #[test]
    fn coloured_field_emits_sgr_and_keeps_plain_text() {
        let attrs = FieldAttributes {
            colour: Colour::Red,
            ..Default::default()
        };
        let m = ScreenModel::new("T").with_field(Field::new("ERR", "boom").with_attrs(attrs));
        let out = render_ansi(&m, RenderOptions::default());
        assert!(out.contains("\x1b[31m")); // red SGR
        assert!(out.contains(RESET));
        assert!(out.contains("boom")); // plain text still present
    }

    // Validates: Requirement 4.3 -- an uncoloured model produces no escapes.
    #[test]
    fn uncoloured_model_has_no_escape_sequences() {
        let m = ScreenModel::new("T").with_field(Field::new("A", "b"));
        let out = render_ansi(&m, RenderOptions::default());
        assert!(!out.contains('\x1b'));
    }
}
