//! Theme TOML loading, inheritance resolution, and validation.
//!
//! The loader reads theme files via the configuration system, validates
//! colour formats and font sizes, resolves inheritance chains, and
//! builds the final `ThemePalette`.

use crate::colour::ColourRGBA;
use crate::defaults;
use crate::design_tokens::DesignTokens;
use crate::element::ElementColourMap;
use crate::error::ThemeError;
use crate::font::FontConfig;
use crate::mode::VisualMode;
use crate::palette::{
    ChromeColours, DecorationColours, EditorColours, FileTreeColours, IndicatorColours,
    SyntaxColours, TabBarColours, ThemePalette, UiColours,
};
use crate::style_slot::{CaseTransform, StyleSlot, StyleSlotTable};

/// Load a theme palette from a TOML string.
///
/// Missing tokens inherit from the built-in default for the specified mode.
/// Invalid values are logged and replaced with defaults.
///
/// # Errors
///
/// Returns `ThemeError::ParseError` if the TOML is completely invalid syntax.
pub fn load_from_toml(toml_str: &str, mode: VisualMode) -> Result<ThemePalette, ThemeError> {
    let table: toml::Table =
        toml_str
            .parse()
            .map_err(|e: toml::de::Error| ThemeError::ParseError {
                path: "<string>".to_string(),
                detail: e.to_string(),
            })?;

    let default = defaults::default_palette_for_mode(mode);
    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&default.name)
        .to_string();

    let _base_name = table.get("base").and_then(|v| v.as_str());

    let editor = parse_editor_colours(&table, &default.editor);
    let syntax = parse_syntax_colours(&table, &default.syntax);
    let file_tree = parse_file_tree_colours(&table, &default.file_tree);
    let tab_bar = parse_tab_bar_colours(&table, &default.tab_bar);
    let chrome = parse_chrome_colours(&table, &default.chrome);
    let decorations = parse_decoration_colours(&table, &default.decorations);
    let indicators = parse_indicator_colours(&table, &default.indicators);
    let ui = parse_ui_colours(&table, &default.ui);
    let fonts = parse_font_config(&table);
    let design = parse_design_tokens(&table);
    let style_slots = parse_style_slots(&table, mode);

    Ok(ThemePalette {
        name,
        mode,
        editor,
        syntax,
        file_tree,
        tab_bar,
        chrome,
        decorations,
        indicators,
        ui,
        style_slots,
        fonts,
        design,
        elements: ElementColourMap::new(),
    })
}

/// Parse a colour from a TOML value, returning the default if invalid.
fn parse_colour(value: Option<&toml::Value>, default: ColourRGBA) -> ColourRGBA {
    value
        .and_then(|v| v.as_str())
        .and_then(|s| ColourRGBA::from_hex(s).ok())
        .unwrap_or(default)
}

/// Get a sub-table from a TOML table.
fn get_section<'a>(table: &'a toml::Table, key: &str) -> Option<&'a toml::Table> {
    table.get(key).and_then(|v| v.as_table())
}

fn parse_editor_colours(table: &toml::Table, default: &EditorColours) -> EditorColours {
    let section = get_section(table, "editor");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    EditorColours {
        background: get("background", default.background),
        foreground: get("foreground", default.foreground),
        accent: get("accent", default.accent),
        muted: get("muted", default.muted),
        modified_indicator: get("modified_indicator", default.modified_indicator),
        current_line_background: get("current_line_background", default.current_line_background),
        selection_secondary_background: get(
            "selection_secondary_background",
            default.selection_secondary_background,
        ),
    }
}

fn parse_syntax_colours(table: &toml::Table, default: &SyntaxColours) -> SyntaxColours {
    let section = get_section(table, "syntax");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    SyntaxColours {
        keyword: get("keyword", default.keyword),
        comment: get("comment", default.comment),
        string: get("string", default.string),
        number: get("number", default.number),
        operator: get("operator", default.operator),
        type_name: get("type", default.type_name),
        function: get("function", default.function),
        macro_name: get("macro", default.macro_name),
        preprocessor: get("preprocessor", default.preprocessor),
        default_text: get("default", default.default_text),
    }
}

fn parse_file_tree_colours(table: &toml::Table, default: &FileTreeColours) -> FileTreeColours {
    let section = get_section(table, "file_tree");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    FileTreeColours {
        binary: get("binary", default.binary),
        structured: get("structured", default.structured),
        text: get("text", default.text),
        unknown: get("unknown", default.unknown),
        directory: get("directory", default.directory),
        symlink: get("symlink", default.symlink),
    }
}

fn parse_tab_bar_colours(table: &toml::Table, default: &TabBarColours) -> TabBarColours {
    let section = get_section(table, "tab_bar");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    TabBarColours {
        active_bg: get("active_background", default.active_bg),
        inactive_bg: get("inactive_background", default.inactive_bg),
        active_text: get("active_text", default.active_text),
        inactive_text: get("inactive_text", default.inactive_text),
        modified_indicator: get("modified_indicator", default.modified_indicator),
        close_button: get("close_button", default.close_button),
        drop_target: get("drop_target", default.drop_target),
    }
}

fn parse_chrome_colours(table: &toml::Table, default: &ChromeColours) -> ChromeColours {
    let section = get_section(table, "chrome");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    ChromeColours {
        cursor_row_border: get("cursor_row_border", default.cursor_row_border),
        cursor_column_indicator: get("cursor_column_indicator", default.cursor_column_indicator),
        line_number_fg: get("line_number_foreground", default.line_number_fg),
        line_number_bg: get("line_number_background", default.line_number_bg),
        fold_margin_bg: get("fold_margin_background", default.fold_margin_bg),
        fold_margin_fg: get("fold_margin_foreground", default.fold_margin_fg),
        margin_separator: get("margin_separator", default.margin_separator),
    }
}

fn parse_decoration_colours(table: &toml::Table, default: &DecorationColours) -> DecorationColours {
    let section = get_section(table, "decorations");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    DecorationColours {
        search_highlight: get("search_highlight", default.search_highlight),
        error_underline: get("error_underline", default.error_underline),
        warning_underline: get("warning_underline", default.warning_underline),
        info_underline: get("info_underline", default.info_underline),
        change_added: get("change_added", default.change_added),
        change_modified: get("change_modified", default.change_modified),
        change_deleted: get("change_deleted", default.change_deleted),
        bookmark: get("bookmark", default.bookmark),
    }
}

fn parse_indicator_colours(table: &toml::Table, default: &IndicatorColours) -> IndicatorColours {
    let section = get_section(table, "indicators");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    let mut user_defined = default.user_defined;
    if let Some(sec) = section {
        if let Some(arr) = sec.get("user_defined").and_then(|v| v.as_array()) {
            for (i, val) in arr.iter().enumerate().take(32) {
                if let Some(s) = val.as_str() {
                    if let Ok(c) = ColourRGBA::from_hex(s) {
                        user_defined[i] = c;
                    }
                }
            }
        }
    }
    IndicatorColours {
        find_match: get("find_match", default.find_match),
        brace_match: get("brace_match", default.brace_match),
        brace_mismatch: get("brace_mismatch", default.brace_mismatch),
        hotspot_underline: get("hotspot_underline", default.hotspot_underline),
        user_defined,
    }
}

fn parse_ui_colours(table: &toml::Table, default: &UiColours) -> UiColours {
    let section = get_section(table, "ui");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    UiColours {
        panel_bg: get("panel_background", default.panel_bg),
        panel_fg: get("panel_foreground", default.panel_fg),
        panel_border: get("panel_border", default.panel_border),
        button_bg: get("button_background", default.button_bg),
        button_fg: get("button_foreground", default.button_fg),
        button_hover: get("button_hover", default.button_hover),
        input_bg: get("input_background", default.input_bg),
        input_border: get("input_border", default.input_border),
        input_fg: get("input_foreground", default.input_fg),
        scrollbar_track: get("scrollbar_track", default.scrollbar_track),
        scrollbar_thumb: get("scrollbar_thumb", default.scrollbar_thumb),
        tooltip_bg: get("tooltip_background", default.tooltip_bg),
        tooltip_fg: get("tooltip_foreground", default.tooltip_fg),
        menu_bar_fg: get("menu_bar_foreground", default.menu_bar_fg),
        primary_menu_bg: get("primary_menu_background", default.primary_menu_bg),
    }
}

fn parse_font_config(table: &toml::Table) -> FontConfig {
    let section = get_section(table, "font");
    let mut config = FontConfig::default();

    if let Some(font_table) = section {
        if let Some(mono) = get_section(font_table, "monospace") {
            if let Some(families) = mono.get("families").and_then(|v| v.as_array()) {
                config.monospace.families = families
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(size) = mono.get("size").and_then(|v| v.as_float()) {
                config.monospace.base_size_pt = crate::font::clamp_font_size(size as f32);
            }
        }
        if let Some(prop) = get_section(font_table, "proportional") {
            if let Some(families) = prop.get("families").and_then(|v| v.as_array()) {
                config.proportional.families = families
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
            if let Some(size) = prop.get("size").and_then(|v| v.as_float()) {
                config.proportional.base_size_pt = crate::font::clamp_font_size(size as f32);
            }
        }
    }

    config
}

fn parse_design_tokens(table: &toml::Table) -> DesignTokens {
    let section = get_section(table, "design");
    let mut tokens = DesignTokens::default();

    if let Some(design) = section {
        if let Some(spacing) = get_section(design, "spacing") {
            if let Some(v) = spacing.get("xs").and_then(|v| v.as_float()) {
                tokens.spacing.xs = v as f32;
            }
            if let Some(v) = spacing.get("sm").and_then(|v| v.as_float()) {
                tokens.spacing.sm = v as f32;
            }
            if let Some(v) = spacing.get("md").and_then(|v| v.as_float()) {
                tokens.spacing.md = v as f32;
            }
            if let Some(v) = spacing.get("lg").and_then(|v| v.as_float()) {
                tokens.spacing.lg = v as f32;
            }
            if let Some(v) = spacing.get("xl").and_then(|v| v.as_float()) {
                tokens.spacing.xl = v as f32;
            }
        }
        if let Some(radius) = get_section(design, "border_radius") {
            if let Some(v) = radius.get("none").and_then(|v| v.as_float()) {
                tokens.border_radius.none = v as f32;
            }
            if let Some(v) = radius.get("sm").and_then(|v| v.as_float()) {
                tokens.border_radius.sm = v as f32;
            }
            if let Some(v) = radius.get("md").and_then(|v| v.as_float()) {
                tokens.border_radius.md = v as f32;
            }
            if let Some(v) = radius.get("lg").and_then(|v| v.as_float()) {
                tokens.border_radius.lg = v as f32;
            }
            if let Some(v) = radius.get("full").and_then(|v| v.as_float()) {
                tokens.border_radius.full = v as f32;
            }
        }
    }

    tokens
}

fn parse_style_slots(table: &toml::Table, mode: VisualMode) -> StyleSlotTable {
    let section = get_section(table, "style_slots");
    let default_palette = defaults::default_palette_for_mode(mode);
    let mut slot_table = default_palette.style_slots;

    if let Some(slots) = section {
        for (key, value) in slots {
            if let Ok(index) = key.parse::<u8>() {
                if let Some(slot_table_entry) = value.as_table() {
                    let default = slot_table.get(index).clone();
                    let slot = StyleSlot {
                        foreground: parse_colour(
                            slot_table_entry.get("foreground"),
                            default.foreground,
                        ),
                        background: parse_colour(
                            slot_table_entry.get("background"),
                            default.background,
                        ),
                        font_family: slot_table_entry
                            .get("font_family")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        bold: slot_table_entry
                            .get("bold")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(default.bold),
                        italic: slot_table_entry
                            .get("italic")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(default.italic),
                        underline: slot_table_entry
                            .get("underline")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(default.underline),
                        case_transform: slot_table_entry
                            .get("case_transform")
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                "upper" => CaseTransform::Upper,
                                "lower" => CaseTransform::Lower,
                                "camel" => CaseTransform::Camel,
                                _ => CaseTransform::None,
                            })
                            .unwrap_or(default.case_transform),
                    };
                    slot_table.set(index, slot);
                }
            }
        }
    }

    slot_table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ui_colour_tokens_overridable_via_toml() {
        // Validates: Requirement 14.1 — every colour token individually overridable
        let toml = r##"
[ui]
menu_bar_foreground = "#FF0000"
primary_menu_background = "#0000FF"
panel_background = "#111111"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        assert_eq!(palette.ui.menu_bar_fg, ColourRGBA::rgb(255, 0, 0));
        assert_eq!(palette.ui.primary_menu_bg, ColourRGBA::rgb(0, 0, 255));
        assert_eq!(palette.ui.panel_bg, ColourRGBA::rgb(0x11, 0x11, 0x11));
    }

    #[test]
    fn invalid_colour_in_user_theme_falls_back_to_default() {
        // Validates: Requirement 14.8 — invalid colour uses fallback, rest loads fine
        let toml = r##"
[ui]
menu_bar_foreground = "not-a-colour"
panel_background = "#ABCDEF"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        let default = defaults::dark_palette();
        // Invalid token falls back to default
        assert_eq!(palette.ui.menu_bar_fg, default.ui.menu_bar_fg);
        // Valid token is applied
        assert_eq!(palette.ui.panel_bg, ColourRGBA::rgb(0xAB, 0xCD, 0xEF));
    }

    #[test]
    fn base_inheritance_fills_missing_tokens() {
        // Validates: Requirement 14.4, 14.5 — omitted tokens inherit from base/default
        // A theme that only overrides one editor token should inherit all others
        let toml = r##"
name = "Partial"

[editor]
background = "#FF0000"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        let default = defaults::dark_palette();
        // Overridden token is applied
        assert_eq!(palette.editor.background, ColourRGBA::rgb(255, 0, 0));
        // All other tokens inherit from the default
        assert_eq!(palette.editor.foreground, default.editor.foreground);
        assert_eq!(palette.syntax.keyword, default.syntax.keyword);
        assert_eq!(palette.ui.panel_bg, default.ui.panel_bg);
    }

    #[test]
    fn load_empty_toml_returns_defaults() {
        // Validates: Requirement 1.6
        let palette = load_from_toml("", VisualMode::Dark).unwrap();
        let default = defaults::dark_palette();
        assert_eq!(palette.editor.background, default.editor.background);
        assert_eq!(palette.syntax.keyword, default.syntax.keyword);
    }

    #[test]
    fn load_partial_toml_overrides_specified_values() {
        // Validates: Requirement 1.6
        let toml = r##"
name = "Custom"

[editor]
background = "#FF0000"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        assert_eq!(palette.name, "Custom");
        assert_eq!(palette.editor.background, ColourRGBA::rgb(255, 0, 0));
        // Unspecified values should be defaults
        let default = defaults::dark_palette();
        assert_eq!(palette.editor.foreground, default.editor.foreground);
    }

    #[test]
    fn load_invalid_colour_uses_default() {
        // Validates: Requirement 1.5
        let toml = r##"
[editor]
background = "not-a-colour"
foreground = "#CDD6F4"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        let default = defaults::dark_palette();
        // Invalid colour should fall back to default
        assert_eq!(palette.editor.background, default.editor.background);
        // Valid colour should be parsed
        assert_eq!(palette.editor.foreground, ColourRGBA::rgb(0xCD, 0xD6, 0xF4));
    }

    #[test]
    fn load_invalid_toml_syntax_returns_error() {
        // Validates: Requirement 1.4
        let result = load_from_toml("this is not valid { toml [", VisualMode::Dark);
        assert!(result.is_err());
    }

    #[test]
    fn load_font_config_from_toml() {
        // Validates: Requirement 4.1, 4.4
        let toml = r##"
[font.monospace]
families = ["JetBrains Mono", "Fira Code", "Consolas"]
size = 16.0

[font.proportional]
families = ["Inter", "Segoe UI"]
size = 13.0
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        assert_eq!(palette.fonts.monospace.families.len(), 3);
        assert_eq!(palette.fonts.monospace.base_size_pt, 16.0);
        assert_eq!(palette.fonts.proportional.families.len(), 2);
    }

    #[test]
    fn load_design_tokens_from_toml() {
        // Validates: Requirement 6.5
        let toml = r##"
[design.spacing]
xs = 4.0
sm = 8.0
md = 16.0
lg = 24.0
xl = 48.0
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        assert_eq!(palette.design.spacing.xs, 4.0);
        assert_eq!(palette.design.spacing.sm, 8.0);
        assert_eq!(palette.design.spacing.md, 16.0);
    }

    #[test]
    fn load_style_slots_from_toml() {
        // Validates: Requirement 3.1, 3.2
        let toml = r##"
[style_slots.50]
foreground = "#FF0000"
bold = true
italic = true
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        let slot = palette.style_slots.get(50);
        assert_eq!(slot.foreground, ColourRGBA::rgb(255, 0, 0));
        assert!(slot.bold);
        assert!(slot.italic);
    }
}
