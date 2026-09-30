//! Renderers converting a [`ScreenModel`] into text of a chosen
//! [`SnapshotFormat`]. Every renderer is a pure `ScreenModel -> String`
//! function and never touches egui.
//!
//! Validates: screen-snapshot-scrm Requirement 4.2-4.6, 5.1-5.4.

mod ansi;
mod html;
mod markdown;
mod text;
mod yaml;

use crate::model::ScreenModel;
use crate::SnapshotFormat;

/// Options controlling text rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    /// When true, use Unicode box-drawing characters for framing; when false,
    /// use an ASCII fallback (`+`, `-`, `|`).
    ///
    /// Validates: Requirement 5.1, 5.2.
    pub unicode_box: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        // Unicode box-drawing by default (Requirement 5.1).
        Self { unicode_box: true }
    }
}

/// Render `model` to `format` using `options`.
///
/// The result is always selectable text (screen-snapshot-scrm Requirement 1.1).
pub fn render(model: &ScreenModel, format: SnapshotFormat, options: RenderOptions) -> String {
    match format {
        SnapshotFormat::PlainText => text::render_plain(model, options),
        SnapshotFormat::Ansi => ansi::render_ansi(model, options),
        SnapshotFormat::Markdown => markdown::render_markdown(model),
        SnapshotFormat::Html => html::render_html(model),
        SnapshotFormat::Yaml => yaml::render_yaml(model),
    }
}

/// The set of visible text fragments a renderer MUST preserve as selectable
/// characters: the title, every field label and value, table cells, messages,
/// buttons, the command line, and the status bar. Used by the property test
/// that guards the "capture is copy/paste-able text" constraint.
///
/// Validates: Requirement 1.1, 1.2.
#[cfg(test)]
pub(crate) fn visible_text_fragments(model: &ScreenModel) -> Vec<String> {
    let mut out = Vec::new();
    if !model.title.is_empty() {
        out.push(model.title.clone());
    }
    for f in &model.fields {
        if !f.label.is_empty() {
            out.push(f.label.clone());
        }
        if !f.value.is_empty() {
            out.push(f.value.clone());
        }
    }
    for t in &model.tables {
        out.extend(t.headers.iter().filter(|h| !h.is_empty()).cloned());
        for row in &t.rows {
            out.extend(row.iter().filter(|c| !c.is_empty()).cloned());
        }
    }
    out.extend(model.messages.iter().filter(|m| !m.is_empty()).cloned());
    out.extend(model.buttons.iter().filter(|b| !b.is_empty()).cloned());
    if !model.command_line.is_empty() {
        out.push(model.command_line.clone());
    }
    if !model.status_bar.text.is_empty() {
        out.push(model.status_bar.text.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Field;

    fn sample() -> ScreenModel {
        ScreenModel::new("Dataset Properties")
            .with_field(Field::new("Dataset Name", "USER.TEST.PDS"))
            .with_field(Field::new("RECFM", "FB"))
            .with_button("Save")
    }

    // Validates: Requirement 5.1 -- default options use Unicode box drawing.
    #[test]
    fn default_render_options_use_unicode() {
        assert!(RenderOptions::default().unicode_box);
    }

    // Validates: Requirement 4.2 -- plain text render contains title + values.
    #[test]
    fn plain_render_contains_title_and_values() {
        let out = render(
            &sample(),
            SnapshotFormat::PlainText,
            RenderOptions::default(),
        );
        assert!(out.contains("Dataset Properties"));
        assert!(out.contains("USER.TEST.PDS"));
        assert!(out.contains("RECFM"));
    }

    // Validates: Requirement 5.2 -- ASCII fallback uses +/-/| not box chars.
    #[test]
    fn ascii_fallback_uses_ascii_frame() {
        let opts = RenderOptions { unicode_box: false };
        let out = render(&sample(), SnapshotFormat::PlainText, opts);
        assert!(out.contains('+'));
        assert!(!out.contains('\u{2500}')); // no horizontal box char
        assert!(!out.contains('\u{250C}')); // no top-left corner
    }

    // Validates: Requirement 5.1 -- Unicode framing uses box chars.
    #[test]
    fn unicode_frame_uses_box_chars() {
        let out = render(
            &sample(),
            SnapshotFormat::PlainText,
            RenderOptions::default(),
        );
        assert!(out.contains('\u{2500}') || out.contains('\u{250C}'));
    }

    // Validates: Requirement 1.1/1.2 -- all visible text survives every text
    // format as selectable characters (spot-check across formats).
    #[test]
    fn all_formats_preserve_visible_field_text() {
        let m = sample();
        for fmt in [
            SnapshotFormat::PlainText,
            SnapshotFormat::Ansi,
            SnapshotFormat::Markdown,
            SnapshotFormat::Html,
            SnapshotFormat::Yaml,
        ] {
            let out = render(&m, fmt, RenderOptions::default());
            for frag in visible_text_fragments(&m) {
                assert!(
                    out.contains(&frag),
                    "format {:?} dropped visible text {:?}",
                    fmt,
                    frag
                );
            }
        }
    }
}
