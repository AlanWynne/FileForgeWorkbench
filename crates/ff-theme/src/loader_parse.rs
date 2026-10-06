//! Per-group TOML parsing helpers for the theme loader.
//!
//! Split out of `loader.rs` (CR-CH-056 Phase 4) to keep each source file under
//! the 400-line limit (`rust-standards.md`). These functions each parse one
//! colour / config group from the theme TOML table, falling back per-key to the
//! supplied default so an omitted token inherits the resolved base (or the mode
//! default). No behaviour changed in the split -- the functions are identical to
//! their former in-`loader.rs` definitions.

use crate::colour::ColourRGBA;
use crate::defaults;
use crate::design_tokens::DesignTokens;
use crate::font::FontConfig;
use crate::mode::VisualMode;
use crate::palette::{
    DecorationColours, EditorColours, FileTreeColours, GutterColours, IndicatorColours,
    SyntaxColours, TabBarColours, UiColours,
};
use crate::style_slot::{CaseTransform, StyleSlot, StyleSlotTable};

/// Parse a colour from a TOML value, returning the default if invalid.
pub(crate) fn parse_colour(value: Option<&toml::Value>, default: ColourRGBA) -> ColourRGBA {
    value
        .and_then(|v| v.as_str())
        .and_then(|s| ColourRGBA::from_hex(s).ok())
        .unwrap_or(default)
}

/// Get a sub-table from a TOML table.
pub(crate) fn get_section<'a>(table: &'a toml::Table, key: &str) -> Option<&'a toml::Table> {
    table.get(key).and_then(|v| v.as_table())
}

pub(crate) fn parse_editor_colours(table: &toml::Table, default: &EditorColours) -> EditorColours {
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

pub(crate) fn parse_syntax_colours(table: &toml::Table, default: &SyntaxColours) -> SyntaxColours {
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

pub(crate) fn parse_file_tree_colours(
    table: &toml::Table,
    default: &FileTreeColours,
) -> FileTreeColours {
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

pub(crate) fn parse_tab_bar_colours(table: &toml::Table, default: &TabBarColours) -> TabBarColours {
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

pub(crate) fn parse_gutter_colours(table: &toml::Table, default: &GutterColours) -> GutterColours {
    // The gutter group is persisted under the `[chrome]` TOML section (unchanged
    // file format in Phase 1); only the in-memory type was renamed.
    let section = get_section(table, "chrome");
    let get = |key: &str, def: ColourRGBA| -> ColourRGBA {
        parse_colour(section.and_then(|s| s.get(key)), def)
    };
    GutterColours {
        cursor_row_border: get("cursor_row_border", default.cursor_row_border),
        cursor_column_indicator: get("cursor_column_indicator", default.cursor_column_indicator),
        line_number_fg: get("line_number_foreground", default.line_number_fg),
        line_number_bg: get("line_number_background", default.line_number_bg),
        fold_margin_bg: get("fold_margin_background", default.fold_margin_bg),
        fold_margin_fg: get("fold_margin_foreground", default.fold_margin_fg),
        margin_separator: get("margin_separator", default.margin_separator),
    }
}

pub(crate) fn parse_decoration_colours(
    table: &toml::Table,
    default: &DecorationColours,
) -> DecorationColours {
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

pub(crate) fn parse_indicator_colours(
    table: &toml::Table,
    default: &IndicatorColours,
) -> IndicatorColours {
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

pub(crate) fn parse_ui_colours(table: &toml::Table, default: &UiColours) -> UiColours {
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
        focus_ring: get("focus_ring", default.focus_ring),
    }
}

pub(crate) fn parse_font_config(table: &toml::Table) -> FontConfig {
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

pub(crate) fn parse_design_tokens(table: &toml::Table) -> DesignTokens {
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

pub(crate) fn parse_style_slots(table: &toml::Table, mode: VisualMode) -> StyleSlotTable {
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
