//! Theme palette serialisation to TOML format.
//!
//! The serialiser writes a `ThemePalette` to valid TOML with section
//! grouping, descriptive comments, and consistent colour formatting.

use crate::colour::ColourRGBA;
use crate::palette::ThemePalette;
use crate::style_slot::CaseTransform;

/// Serialise a `ThemePalette` to a TOML string with section comments.
///
/// Produces valid TOML with the semantic grouping structure preserved.
/// Colours are formatted as `#RRGGBB` for opaque and `#RRGGBBAA` for translucent.
pub fn serialise(palette: &ThemePalette) -> String {
    let mut out = String::with_capacity(4096);

    // Header
    out.push_str("# Theme: ");
    out.push_str(&palette.name);
    out.push_str("\n# Mode: ");
    out.push_str(palette.mode.section_name());
    out.push_str("\n\n");

    out.push_str("name = \"");
    out.push_str(&palette.name);
    out.push_str("\"\n\n");

    // Editor colours
    out.push_str("# Editor content area colours\n");
    out.push_str("[editor]\n");
    write_colour(&mut out, "background", palette.editor.background);
    write_colour(&mut out, "foreground", palette.editor.foreground);
    write_colour(&mut out, "accent", palette.editor.accent);
    write_colour(&mut out, "muted", palette.editor.muted);
    write_colour(
        &mut out,
        "modified_indicator",
        palette.editor.modified_indicator,
    );
    write_colour(
        &mut out,
        "current_line_background",
        palette.editor.current_line_background,
    );
    write_colour(
        &mut out,
        "selection_secondary_background",
        palette.editor.selection_secondary_background,
    );
    out.push('\n');

    // Syntax colours
    out.push_str("# Syntax highlighting colours\n");
    out.push_str("[syntax]\n");
    write_colour(&mut out, "keyword", palette.syntax.keyword);
    write_colour(&mut out, "comment", palette.syntax.comment);
    write_colour(&mut out, "string", palette.syntax.string);
    write_colour(&mut out, "number", palette.syntax.number);
    write_colour(&mut out, "operator", palette.syntax.operator);
    write_colour(&mut out, "type", palette.syntax.type_name);
    write_colour(&mut out, "function", palette.syntax.function);
    write_colour(&mut out, "macro", palette.syntax.macro_name);
    write_colour(&mut out, "preprocessor", palette.syntax.preprocessor);
    write_colour(&mut out, "default", palette.syntax.default_text);
    out.push('\n');

    // File tree colours
    out.push_str("# File tree panel colours\n");
    out.push_str("[file_tree]\n");
    write_colour(&mut out, "binary", palette.file_tree.binary);
    write_colour(&mut out, "structured", palette.file_tree.structured);
    write_colour(&mut out, "text", palette.file_tree.text);
    write_colour(&mut out, "unknown", palette.file_tree.unknown);
    write_colour(&mut out, "directory", palette.file_tree.directory);
    write_colour(&mut out, "symlink", palette.file_tree.symlink);
    out.push('\n');

    // Tab bar colours
    out.push_str("# Tab bar colours\n");
    out.push_str("[tab_bar]\n");
    write_colour(&mut out, "active_background", palette.tab_bar.active_bg);
    write_colour(&mut out, "inactive_background", palette.tab_bar.inactive_bg);
    write_colour(&mut out, "active_text", palette.tab_bar.active_text);
    write_colour(&mut out, "inactive_text", palette.tab_bar.inactive_text);
    write_colour(
        &mut out,
        "modified_indicator",
        palette.tab_bar.modified_indicator,
    );
    write_colour(&mut out, "close_button", palette.tab_bar.close_button);
    write_colour(&mut out, "drop_target", palette.tab_bar.drop_target);
    out.push('\n');

    // Chrome colours
    out.push_str("# Editor chrome colours (line numbers, margins)\n");
    out.push_str("[chrome]\n");
    write_colour(
        &mut out,
        "cursor_row_border",
        palette.chrome.cursor_row_border,
    );
    write_colour(
        &mut out,
        "cursor_column_indicator",
        palette.chrome.cursor_column_indicator,
    );
    write_colour(
        &mut out,
        "line_number_foreground",
        palette.chrome.line_number_fg,
    );
    write_colour(
        &mut out,
        "line_number_background",
        palette.chrome.line_number_bg,
    );
    write_colour(
        &mut out,
        "fold_margin_background",
        palette.chrome.fold_margin_bg,
    );
    write_colour(
        &mut out,
        "fold_margin_foreground",
        palette.chrome.fold_margin_fg,
    );
    write_colour(
        &mut out,
        "margin_separator",
        palette.chrome.margin_separator,
    );
    out.push('\n');

    // Decoration colours
    out.push_str("# Text decoration and marker colours\n");
    out.push_str("[decorations]\n");
    write_colour(
        &mut out,
        "search_highlight",
        palette.decorations.search_highlight,
    );
    write_colour(
        &mut out,
        "error_underline",
        palette.decorations.error_underline,
    );
    write_colour(
        &mut out,
        "warning_underline",
        palette.decorations.warning_underline,
    );
    write_colour(
        &mut out,
        "info_underline",
        palette.decorations.info_underline,
    );
    write_colour(&mut out, "change_added", palette.decorations.change_added);
    write_colour(
        &mut out,
        "change_modified",
        palette.decorations.change_modified,
    );
    write_colour(
        &mut out,
        "change_deleted",
        palette.decorations.change_deleted,
    );
    write_colour(&mut out, "bookmark", palette.decorations.bookmark);
    out.push('\n');

    // Indicator colours
    out.push_str("# Indicator and match highlight colours\n");
    out.push_str("[indicators]\n");
    write_colour(&mut out, "find_match", palette.indicators.find_match);
    write_colour(&mut out, "brace_match", palette.indicators.brace_match);
    write_colour(
        &mut out,
        "brace_mismatch",
        palette.indicators.brace_mismatch,
    );
    write_colour(
        &mut out,
        "hotspot_underline",
        palette.indicators.hotspot_underline,
    );
    out.push_str("user_defined = [");
    for (i, c) in palette.indicators.user_defined.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(&c.to_hex());
        out.push('"');
    }
    out.push_str("]\n\n");

    // UI colours
    out.push_str("# General UI component colours\n");
    out.push_str("[ui]\n");
    write_colour(&mut out, "panel_background", palette.ui.panel_bg);
    write_colour(&mut out, "panel_foreground", palette.ui.panel_fg);
    write_colour(&mut out, "panel_border", palette.ui.panel_border);
    write_colour(&mut out, "button_background", palette.ui.button_bg);
    write_colour(&mut out, "button_foreground", palette.ui.button_fg);
    write_colour(&mut out, "button_hover", palette.ui.button_hover);
    write_colour(&mut out, "input_background", palette.ui.input_bg);
    write_colour(&mut out, "input_border", palette.ui.input_border);
    write_colour(&mut out, "input_foreground", palette.ui.input_fg);
    write_colour(&mut out, "scrollbar_track", palette.ui.scrollbar_track);
    write_colour(&mut out, "scrollbar_thumb", palette.ui.scrollbar_thumb);
    write_colour(&mut out, "tooltip_background", palette.ui.tooltip_bg);
    write_colour(&mut out, "tooltip_foreground", palette.ui.tooltip_fg);
    write_colour(&mut out, "menu_bar_foreground", palette.ui.menu_bar_fg);
    write_colour(
        &mut out,
        "primary_menu_background",
        palette.ui.primary_menu_bg,
    );
    out.push('\n');

    // Font config
    out.push_str("# Font configuration\n");
    out.push_str("[font.monospace]\n");
    out.push_str("families = [");
    for (i, f) in palette.fonts.monospace.families.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(f);
        out.push('"');
    }
    out.push_str("]\n");
    out.push_str(&format!(
        "size = {:.1}\n\n",
        palette.fonts.monospace.base_size_pt
    ));

    out.push_str("[font.proportional]\n");
    out.push_str("families = [");
    for (i, f) in palette.fonts.proportional.families.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(f);
        out.push('"');
    }
    out.push_str("]\n");
    out.push_str(&format!(
        "size = {:.1}\n\n",
        palette.fonts.proportional.base_size_pt
    ));

    // Design tokens
    out.push_str("# Design system tokens\n");
    out.push_str("[design.spacing]\n");
    out.push_str(&format!("xs = {:.1}\n", palette.design.spacing.xs));
    out.push_str(&format!("sm = {:.1}\n", palette.design.spacing.sm));
    out.push_str(&format!("md = {:.1}\n", palette.design.spacing.md));
    out.push_str(&format!("lg = {:.1}\n", palette.design.spacing.lg));
    out.push_str(&format!("xl = {:.1}\n\n", palette.design.spacing.xl));

    out.push_str("[design.border_radius]\n");
    out.push_str(&format!(
        "none = {:.1}\n",
        palette.design.border_radius.none
    ));
    out.push_str(&format!("sm = {:.1}\n", palette.design.border_radius.sm));
    out.push_str(&format!("md = {:.1}\n", palette.design.border_radius.md));
    out.push_str(&format!("lg = {:.1}\n", palette.design.border_radius.lg));
    out.push_str(&format!(
        "full = {:.1}\n\n",
        palette.design.border_radius.full
    ));

    // Style slots (only defined ones beyond default)
    let has_custom_slots = (0..=255u8)
        .any(|i| i != crate::style_slot::DEFAULT_STYLE_INDEX && palette.style_slots.is_defined(i));
    if has_custom_slots {
        out.push_str("# Style slots (indexed 0-255)\n");
        for i in 0..=255u8 {
            if i != crate::style_slot::DEFAULT_STYLE_INDEX && palette.style_slots.is_defined(i) {
                let slot = palette.style_slots.get(i);
                out.push_str(&format!("[style_slots.{}]\n", i));
                write_colour(&mut out, "foreground", slot.foreground);
                write_colour(&mut out, "background", slot.background);
                if let Some(ref family) = slot.font_family {
                    out.push_str(&format!("font_family = \"{}\"\n", family));
                }
                out.push_str(&format!("bold = {}\n", slot.bold));
                out.push_str(&format!("italic = {}\n", slot.italic));
                out.push_str(&format!("underline = {}\n", slot.underline));
                let case_str = match slot.case_transform {
                    CaseTransform::None => "none",
                    CaseTransform::Upper => "upper",
                    CaseTransform::Lower => "lower",
                    CaseTransform::Camel => "camel",
                };
                out.push_str(&format!("case_transform = \"{}\"\n\n", case_str));
            }
        }
    }

    out
}

/// Write a colour key-value pair to the output buffer.
fn write_colour(out: &mut String, key: &str, colour: ColourRGBA) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&colour.to_hex());
    out.push_str("\"\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults;
    use crate::loader::load_from_toml;
    use crate::mode::VisualMode;

    #[test]
    fn serialise_produces_valid_toml() {
        // Validates: Requirement 9.1
        let palette = defaults::dark_palette();
        let toml_str = serialise(&palette);
        // Should parse without error
        let parsed: Result<toml::Table, _> = toml_str.parse();
        assert!(parsed.is_ok(), "Serialised output is not valid TOML");
    }

    #[test]
    fn serialise_includes_section_comments() {
        // Validates: Requirement 9.4
        let palette = defaults::dark_palette();
        let toml_str = serialise(&palette);
        assert!(toml_str.contains("# Editor content area colours"));
        assert!(toml_str.contains("# Syntax highlighting colours"));
        assert!(toml_str.contains("# File tree panel colours"));
    }

    #[test]
    fn serialise_opaque_colour_uses_rrggbb() {
        // Validates: Requirement 9.5
        let palette = defaults::dark_palette();
        let toml_str = serialise(&palette);
        // Editor background should be opaque #RRGGBB format
        assert!(toml_str.contains("\"#1E1E2E\""));
    }

    #[test]
    fn serialise_round_trip_preserves_colours() {
        // Validates: Requirement 9.2
        let original = defaults::dark_palette();
        let toml_str = serialise(&original);
        let round_tripped = load_from_toml(&toml_str, VisualMode::Dark).unwrap();

        assert_eq!(original.editor, round_tripped.editor);
        assert_eq!(original.syntax, round_tripped.syntax);
        assert_eq!(original.file_tree, round_tripped.file_tree);
        assert_eq!(original.tab_bar, round_tripped.tab_bar);
        assert_eq!(original.chrome, round_tripped.chrome);
        assert_eq!(original.decorations, round_tripped.decorations);
        assert_eq!(original.ui, round_tripped.ui);
    }

    #[test]
    fn serialise_round_trip_preserves_fonts() {
        // Validates: Requirement 9.2
        let mut palette = defaults::dark_palette();
        palette.fonts.monospace.families = vec!["JetBrains Mono".to_string()];
        palette.fonts.monospace.base_size_pt = 16.0;
        let toml_str = serialise(&palette);
        let round_tripped = load_from_toml(&toml_str, VisualMode::Dark).unwrap();
        assert_eq!(
            palette.fonts.monospace.families,
            round_tripped.fonts.monospace.families
        );
        assert_eq!(
            palette.fonts.monospace.base_size_pt,
            round_tripped.fonts.monospace.base_size_pt
        );
    }
}
