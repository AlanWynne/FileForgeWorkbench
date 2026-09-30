//! Plain-text renderer with Unicode box-drawing framing and an ASCII fallback.
//!
//! Validates: screen-snapshot-scrm Requirement 4.2, 5.1, 5.2, 5.4.

use crate::model::ScreenModel;
use crate::render::RenderOptions;

/// Box-drawing glyphs, either Unicode or ASCII fallback.
struct Glyphs {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    horizontal: char,
    vertical: char,
    tee_left: char,
    tee_right: char,
}

impl Glyphs {
    fn for_options(options: RenderOptions) -> Self {
        if options.unicode_box {
            Self {
                top_left: '\u{250C}',
                top_right: '\u{2510}',
                bottom_left: '\u{2514}',
                bottom_right: '\u{2518}',
                horizontal: '\u{2500}',
                vertical: '\u{2502}',
                tee_left: '\u{251C}',
                tee_right: '\u{2524}',
            }
        } else {
            Self {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '-',
                vertical: '|',
                tee_left: '+',
                tee_right: '+',
            }
        }
    }
}

/// Render `model` as plain framed text.
pub fn render_plain(model: &ScreenModel, options: RenderOptions) -> String {
    let g = Glyphs::for_options(options);
    // Inner width: fit the widest content line, with sane bounds.
    let mut lines: Vec<String> = Vec::new();
    lines.push(model.title.clone());

    if !model.fields.is_empty() {
        lines.push(String::new());
        for f in &model.fields {
            if f.label.is_empty() {
                lines.push(f.value.clone());
            } else {
                lines.push(format!("{} . . . : {}", f.label, f.value));
            }
        }
    }

    for table in &model.tables {
        lines.push(String::new());
        if let Some(cap) = &table.caption {
            lines.push(cap.clone());
        }
        if !table.headers.is_empty() {
            lines.push(table.headers.join("  "));
        }
        for row in &table.rows {
            lines.push(row.join("  "));
        }
    }

    if !model.messages.is_empty() {
        lines.push(String::new());
        for m in &model.messages {
            lines.push(m.clone());
        }
    }

    if !model.buttons.is_empty() {
        lines.push(String::new());
        lines.push(
            model
                .buttons
                .iter()
                .map(|b| format!("[{b}]"))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }

    let inner_width = lines
        .iter()
        .map(|l| l.chars().count())
        .chain(std::iter::once(model.title.chars().count()))
        .max()
        .unwrap_or(0)
        .max(20);

    let mut out = String::new();
    // Top border.
    push_border(&mut out, g.top_left, g.horizontal, g.top_right, inner_width);
    // Title row, then a separator.
    push_content(&mut out, g.vertical, &model.title, inner_width);
    push_border(&mut out, g.tee_left, g.horizontal, g.tee_right, inner_width);
    // Body rows (skip the title we already emitted at index 0).
    for line in lines.iter().skip(1) {
        push_content(&mut out, g.vertical, line, inner_width);
    }
    // Bottom border.
    push_border(
        &mut out,
        g.bottom_left,
        g.horizontal,
        g.bottom_right,
        inner_width,
    );

    // Command line and status bar sit OUTSIDE the frame, like ISPF.
    if !model.command_line.is_empty() {
        out.push('\n');
        out.push_str(&format!("Command ===> {}\n", model.command_line));
    }
    if !model.status_bar.text.is_empty() {
        out.push_str(&model.status_bar.text);
        out.push('\n');
    }
    out
}

fn push_border(out: &mut String, left: char, fill: char, right: char, inner_width: usize) {
    out.push(left);
    for _ in 0..inner_width + 2 {
        out.push(fill);
    }
    out.push(right);
    out.push('\n');
}

fn push_content(out: &mut String, vertical: char, text: &str, inner_width: usize) {
    let len = text.chars().count();
    let pad = inner_width.saturating_sub(len);
    out.push(vertical);
    out.push(' ');
    out.push_str(text);
    for _ in 0..pad {
        out.push(' ');
    }
    out.push(' ');
    out.push(vertical);
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Field;

    // Validates: Requirement 4.2 -- values and command line appear.
    #[test]
    fn plain_includes_command_line_outside_frame() {
        let mut m = ScreenModel::new("T").with_field(Field::new("A", "b"));
        m.command_line = "SNAPSHOT".to_string();
        let out = render_plain(&m, RenderOptions::default());
        assert!(out.contains("Command ===> SNAPSHOT"));
        assert!(out.contains('b'));
    }
}
