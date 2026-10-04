//! Legacy Soft built-in palette builder (CR-CH-056, Phase 2 / Task 30).
//!
//! `Legacy Soft` is a softer phosphor variant of the ISPF 3270 Legacy look. It
//! keeps the ISPF semantic ROLES in every domain group (syntax / gutter /
//! file_tree / decorations / indicators) but softens the harshest pure-saturated
//! values -- body green `#00FF00` becomes a gentler `#33FF66`, and the most
//! electric blues are muted -- so long sessions are easier on the eyes without
//! altering the authentic `Default Legacy`. It keeps the toned `#000060` primary
//! option-menu band (shared with the amended Legacy, Requirement 13.2). It is a
//! `VisualMode::Legacy` theme. Split out of `defaults.rs` to keep that file from
//! growing (rust-standards 400-line rule).

use crate::chrome_style::ChromeStyle;
use crate::colour::ColourRGBA;
use crate::design_tokens::DesignTokens;
use crate::element::ElementColourMap;
use crate::font::FontConfig;
use crate::mode::VisualMode;
use crate::palette::{
    DecorationColours, EditorColours, FileTreeColours, GutterColours, IndicatorColours,
    SyntaxColours, TabBarColours, ThemePalette, UiColours,
};
use crate::style_slot::{CaseTransform, StyleSlot, StyleSlotTable};

// === Softened ISPF phosphor palette =========================================
// Each colour keeps its ISPF semantic role but trades the pure-saturated 3270
// value for a gentler phosphor tone.

const SOFT_BG: ColourRGBA = ColourRGBA::rgb(0x0A, 0x0A, 0x0E); // #0A0A0E near-black
const SOFT_BG_ALT: ColourRGBA = ColourRGBA::rgb(0x10, 0x12, 0x20); // #101220 wash
const SOFT_GREEN: ColourRGBA = ColourRGBA::rgb(0x33, 0xFF, 0x66); // #33FF66 body (was #00FF00)
const SOFT_GREEN_DIM: ColourRGBA = ColourRGBA::rgb(0x2C, 0xC4, 0x5A); // #2CC45A
const SOFT_BLUE: ColourRGBA = ColourRGBA::rgb(0x6E, 0x9B, 0xFF); // #6E9BFF (muted electric blue)
const SOFT_BLUE_DIM: ColourRGBA = ColourRGBA::rgb(0x3C, 0x55, 0x9E); // #3C559E structure
const SOFT_TURQUOISE: ColourRGBA = ColourRGBA::rgb(0x4F, 0xD6, 0xD6); // #4FD6D6 (was #00FFFF)
const SOFT_YELLOW: ColourRGBA = ColourRGBA::rgb(0xE6, 0xD2, 0x5A); // #E6D25A (was #FFFF00)
const SOFT_RED: ColourRGBA = ColourRGBA::rgb(0xF2, 0x5A, 0x5A); // #F25A5A (was #FF0000)
const SOFT_PINK: ColourRGBA = ColourRGBA::rgb(0xE0, 0x6C, 0xC8); // #E06CC8 (was #FF00FF)
const SOFT_WHITE: ColourRGBA = ColourRGBA::rgb(0xE8, 0xE8, 0xE8); // #E8E8E8 (was #FFFFFF)
const SOFT_MENU_BAND: ColourRGBA = ColourRGBA::rgb(0x00, 0x00, 0x60); // #000060 toned band

fn chrome_style_for(
    ui: &UiColours,
    tab_bar: &TabBarColours,
    editor: &EditorColours,
    design: &DesignTokens,
) -> ChromeStyle {
    ChromeStyle::from_palette_parts(ui, tab_bar, editor, design, VisualMode::Legacy)
}

/// Build the `Legacy Soft` built-in palette.
///
/// Validates: theme-and-appearance Requirement 18.3
pub fn legacy_soft_palette() -> ThemePalette {
    let editor = soft_editor();
    let tab_bar = soft_tab_bar();
    let ui = soft_ui();
    let design = DesignTokens::default();
    let chrome_style = chrome_style_for(&ui, &tab_bar, &editor, &design);
    ThemePalette {
        name: "Legacy Soft".to_string(),
        mode: VisualMode::Legacy,
        editor,
        syntax: soft_syntax(),
        file_tree: soft_file_tree(),
        tab_bar,
        gutter: soft_gutter(),
        decorations: soft_decorations(),
        indicators: soft_indicators(),
        ui,
        style_slots: soft_style_slot_table(),
        fonts: FontConfig::default(),
        design,
        elements: ElementColourMap::new(),
        chrome_style,
    }
}

fn soft_editor() -> EditorColours {
    EditorColours {
        background: SOFT_BG,
        foreground: SOFT_GREEN,
        accent: SOFT_YELLOW,
        muted: SOFT_BLUE_DIM,
        modified_indicator: SOFT_YELLOW,
        current_line_background: SOFT_BG_ALT,
        selection_secondary_background: ColourRGBA::rgba(0x4F, 0xD6, 0xD6, 55),
    }
}

fn soft_syntax() -> SyntaxColours {
    SyntaxColours {
        keyword: SOFT_YELLOW,
        comment: SOFT_BLUE,
        string: SOFT_GREEN_DIM,
        number: SOFT_WHITE,
        operator: SOFT_TURQUOISE,
        type_name: SOFT_PINK,
        function: SOFT_YELLOW,
        macro_name: SOFT_PINK,
        preprocessor: SOFT_RED,
        default_text: SOFT_BLUE,
    }
}

fn soft_file_tree() -> FileTreeColours {
    FileTreeColours {
        binary: SOFT_RED,
        structured: SOFT_TURQUOISE,
        text: SOFT_BLUE,
        unknown: SOFT_BLUE,
        directory: SOFT_WHITE,
        symlink: SOFT_TURQUOISE,
    }
}

fn soft_tab_bar() -> TabBarColours {
    TabBarColours {
        active_bg: SOFT_BG_ALT,
        inactive_bg: SOFT_BG,
        active_text: SOFT_WHITE,
        inactive_text: SOFT_BLUE,
        modified_indicator: SOFT_YELLOW,
        close_button: SOFT_WHITE,
        drop_target: ColourRGBA::rgba(0x4F, 0xD6, 0xD6, 80),
    }
}

fn soft_gutter() -> GutterColours {
    GutterColours {
        cursor_row_border: SOFT_TURQUOISE,
        cursor_column_indicator: SOFT_TURQUOISE,
        line_number_fg: SOFT_BLUE,
        line_number_bg: SOFT_BG,
        fold_margin_bg: SOFT_BG,
        fold_margin_fg: SOFT_BLUE,
        margin_separator: SOFT_BLUE,
    }
}

fn soft_decorations() -> DecorationColours {
    DecorationColours {
        search_highlight: ColourRGBA::rgba(0xE6, 0xD2, 0x5A, 85),
        error_underline: SOFT_RED,
        warning_underline: SOFT_PINK,
        info_underline: SOFT_TURQUOISE,
        change_added: SOFT_GREEN,
        change_modified: SOFT_YELLOW,
        change_deleted: SOFT_RED,
        bookmark: SOFT_TURQUOISE,
    }
}

fn soft_indicators() -> IndicatorColours {
    IndicatorColours {
        find_match: ColourRGBA::rgba(0xE6, 0xD2, 0x5A, 75),
        brace_match: SOFT_GREEN,
        brace_mismatch: SOFT_RED,
        hotspot_underline: SOFT_YELLOW,
        user_defined: [SOFT_BLUE; 32],
    }
}

fn soft_ui() -> UiColours {
    UiColours {
        panel_bg: SOFT_BG,
        panel_fg: SOFT_TURQUOISE,
        panel_border: SOFT_BLUE_DIM,
        button_bg: SOFT_BG_ALT,
        button_fg: SOFT_YELLOW,
        button_hover: ColourRGBA::rgb(0x18, 0x1C, 0x30),
        input_bg: SOFT_BG,
        input_border: SOFT_TURQUOISE,
        input_fg: SOFT_TURQUOISE,
        scrollbar_track: SOFT_BG,
        scrollbar_thumb: SOFT_BLUE,
        tooltip_bg: SOFT_BG_ALT,
        tooltip_fg: SOFT_TURQUOISE,
        menu_bar_fg: SOFT_WHITE,
        // Toned primary option-menu band, shared with the amended Legacy.
        primary_menu_bg: SOFT_MENU_BAND,
        focus_ring: SOFT_YELLOW,
    }
}

fn soft_style_slot_table() -> StyleSlotTable {
    let default_slot = StyleSlot {
        foreground: SOFT_GREEN,
        background: SOFT_BG,
        font_family: None,
        bold: false,
        italic: false,
        underline: false,
        case_transform: CaseTransform::None,
    };
    StyleSlotTable::new(default_slot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_soft_has_expected_identity() {
        // Validates: Requirement 18.3 -- Legacy Soft is a distinct Legacy-mode
        // built-in with softened body green #33FF66.
        let p = legacy_soft_palette();
        assert_eq!(p.name, "Legacy Soft");
        assert_eq!(p.mode, VisualMode::Legacy);
        assert_eq!(p.editor.foreground, ColourRGBA::rgb(0x33, 0xFF, 0x66));
    }

    #[test]
    fn legacy_soft_keeps_toned_menu_band() {
        // Validates: Requirement 13.2 -- toned #000060 primary-menu band.
        let p = legacy_soft_palette();
        assert_eq!(p.ui.primary_menu_bg, ColourRGBA::rgb(0x00, 0x00, 0x60));
    }

    #[test]
    fn legacy_soft_softens_pure_saturated_values() {
        // Validates: Requirement 18.3 -- harshest pure values are softened.
        let p = legacy_soft_palette();
        // Not the electric 3270 primaries.
        assert_ne!(p.editor.foreground, ColourRGBA::rgb(0, 255, 0));
        assert_ne!(p.ui.input_fg, ColourRGBA::rgb(0, 255, 255));
        assert_ne!(p.editor.accent, ColourRGBA::rgb(255, 255, 0));
    }
}
