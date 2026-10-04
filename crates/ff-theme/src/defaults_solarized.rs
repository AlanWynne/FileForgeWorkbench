//! Solarized built-in palette builders (CR-CH-056, Phase 2 / Task 30).
//!
//! The `Default Dark` and `Default Light` built-ins are Solarized instances of
//! the egui-native theme model. The NAMES (`Default Dark` / `Default Light`) and
//! `VisualMode` (Dark / Light) are retained; only the colour data changes from
//! the former Catppuccin instances. Split out of `defaults.rs` to keep that file
//! from growing (rust-standards 400-line rule).
//!
//! Both instances satisfy the egui-Visuals chrome contract (Requirement 23.7 -
//! 23.10): a three-level background hierarchy (window/base -> raised surface ->
//! inset input), an accent focus ring, an accent-tinted active tab distinct from
//! inactive, and an accent-tinted primary-menu / title band distinct from base.
//! The chrome accent is Solarized blue (`#268BD2`).

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

// === Shared Solarized accent palette ========================================
// The eight Solarized accent hues are shared between the Dark and Light modes
// (Ethan Schoonover's Solarized specification).

const SOL_YELLOW: ColourRGBA = ColourRGBA::rgb(0xB5, 0x89, 0x00); // #B58900
const SOL_ORANGE: ColourRGBA = ColourRGBA::rgb(0xCB, 0x4B, 0x16); // #CB4B16
const SOL_RED: ColourRGBA = ColourRGBA::rgb(0xDC, 0x32, 0x2F); // #DC322F
const SOL_MAGENTA: ColourRGBA = ColourRGBA::rgb(0xD3, 0x36, 0x82); // #D33682
const SOL_VIOLET: ColourRGBA = ColourRGBA::rgb(0x6C, 0x71, 0xC4); // #6C71C4
const SOL_BLUE: ColourRGBA = ColourRGBA::rgb(0x26, 0x8B, 0xD2); // #268BD2
const SOL_CYAN: ColourRGBA = ColourRGBA::rgb(0x2A, 0xA1, 0x98); // #2AA198
const SOL_GREEN: ColourRGBA = ColourRGBA::rgb(0x85, 0x99, 0x00); // #859900

// Solarized monotone base tones.
const SOL_BASE03: ColourRGBA = ColourRGBA::rgb(0x00, 0x2B, 0x36); // #002B36
const SOL_BASE02: ColourRGBA = ColourRGBA::rgb(0x07, 0x36, 0x42); // #073642
const SOL_BASE01: ColourRGBA = ColourRGBA::rgb(0x58, 0x6E, 0x75); // #586E75
const SOL_BASE00: ColourRGBA = ColourRGBA::rgb(0x65, 0x7B, 0x83); // #657B83
const SOL_BASE0: ColourRGBA = ColourRGBA::rgb(0x83, 0x94, 0x96); // #839496
const SOL_BASE1: ColourRGBA = ColourRGBA::rgb(0x93, 0xA1, 0xA1); // #93A1A1
const SOL_BASE2: ColourRGBA = ColourRGBA::rgb(0xEE, 0xE8, 0xD5); // #EEE8D5
const SOL_BASE3: ColourRGBA = ColourRGBA::rgb(0xFD, 0xF6, 0xE3); // #FDF6E3

// Dark-mode-only chrome tones derived to give three perceptibly distinct levels
// (base03 is the content panel; the window sits one step darker; the inset text
// background sits darker still).
const SOL_D_WINDOW: ColourRGBA = ColourRGBA::rgb(0x00, 0x20, 0x29); // #002029 window/base
const SOL_D_INPUT: ColourRGBA = ColourRGBA::rgb(0x00, 0x18, 0x20); // #001820 inset input
const SOL_D_RAISED_HOVER: ColourRGBA = ColourRGBA::rgb(0x0B, 0x45, 0x53); // #0B4553
                                                                          // Accent-tinted tab/band tones, darkened just enough that base1 text clears
                                                                          // WCAG AA (Req 23.11): active tab 5.23:1, title band 5.61:1.
const SOL_D_TAB_ACTIVE: ColourRGBA = ColourRGBA::rgb(0x06, 0x30, 0x3F); // #06303F accent-tinted
const SOL_D_TITLE_BAND: ColourRGBA = ColourRGBA::rgb(0x06, 0x2A, 0x3A); // #062A3A accent-tinted

// Light-mode-only chrome tones (base3 content; base2 raised; a step lighter inset).
const SOL_L_WINDOW: ColourRGBA = ColourRGBA::rgb(0xE7, 0xE1, 0xCF); // #E7E1CF window/base
const SOL_L_INPUT: ColourRGBA = ColourRGBA::rgb(0xFB, 0xF3, 0xDC); // #FBF3DC inset input
const SOL_L_RAISED_HOVER: ColourRGBA = ColourRGBA::rgb(0xDE, 0xD7, 0xC1); // #DED7C1
const SOL_L_TAB_ACTIVE: ColourRGBA = ColourRGBA::rgb(0xD9, 0xE6, 0xF2); // #D9E6F2 accent-tinted
const SOL_L_TITLE_BAND: ColourRGBA = ColourRGBA::rgb(0xD2, 0xE2, 0xF1); // #D2E2F1 accent-tinted
const SOL_L_TITLE_FG: ColourRGBA = ColourRGBA::rgb(0x1A, 0x3A, 0x52); // #1A3A52 band text
                                                                      // Body/chrome foreground: Solarized base01 (#586E75) deepened to #4E5F64 so all
                                                                      // checked text pairs clear WCAG AA (Req 23.11) on the lighter raised surfaces,
                                                                      // while remaining visually the Solarized base01 role tone. base00 (#657B83)
                                                                      // stays for muted / gutter / inactive-tab (the >= 3:1 UI-element pairs).
const SOL_L_FG: ColourRGBA = ColourRGBA::rgb(0x4E, 0x5F, 0x64); // #4E5F64

fn chrome_style_for(
    ui: &UiColours,
    tab_bar: &TabBarColours,
    editor: &EditorColours,
    design: &DesignTokens,
    mode: VisualMode,
) -> ChromeStyle {
    ChromeStyle::from_palette_parts(ui, tab_bar, editor, design, mode)
}

fn solarized_style_slot_table(fg: ColourRGBA, bg: ColourRGBA) -> StyleSlotTable {
    let default_slot = StyleSlot {
        foreground: fg,
        background: bg,
        font_family: None,
        bold: false,
        italic: false,
        underline: false,
        case_transform: CaseTransform::None,
    };
    StyleSlotTable::new(default_slot)
}

// === Solarized Dark =========================================================

/// Build the `Default Dark` built-in as a Solarized Dark instance.
///
/// Validates: theme-and-appearance Requirement 18.3, 23.7, 23.8, 23.9, 23.10
pub fn solarized_dark_palette() -> ThemePalette {
    let editor = solarized_dark_editor();
    let tab_bar = solarized_dark_tab_bar();
    let ui = solarized_dark_ui();
    let design = DesignTokens::default();
    let chrome_style = chrome_style_for(&ui, &tab_bar, &editor, &design, VisualMode::Dark);
    ThemePalette {
        name: "Default Dark".to_string(),
        mode: VisualMode::Dark,
        editor,
        syntax: solarized_dark_syntax(),
        file_tree: solarized_dark_file_tree(),
        tab_bar,
        gutter: solarized_dark_gutter(),
        decorations: solarized_decorations(),
        indicators: solarized_indicators(SOL_BASE01),
        ui,
        style_slots: solarized_style_slot_table(SOL_BASE0, SOL_BASE03),
        fonts: FontConfig::default(),
        design,
        elements: ElementColourMap::new(),
        chrome_style,
    }
}

fn solarized_dark_editor() -> EditorColours {
    EditorColours {
        background: SOL_BASE03,
        foreground: SOL_BASE0,
        accent: SOL_BLUE,
        muted: SOL_BASE01,
        modified_indicator: SOL_YELLOW,
        current_line_background: SOL_BASE02,
        selection_secondary_background: ColourRGBA::rgba(0x26, 0x8B, 0xD2, 50),
    }
}

fn solarized_dark_syntax() -> SyntaxColours {
    SyntaxColours {
        keyword: SOL_GREEN,
        comment: SOL_BASE01,
        string: SOL_CYAN,
        number: SOL_MAGENTA,
        operator: SOL_BASE0,
        type_name: SOL_YELLOW,
        function: SOL_BLUE,
        macro_name: SOL_VIOLET,
        preprocessor: SOL_ORANGE,
        default_text: SOL_BASE0,
    }
}

fn solarized_dark_file_tree() -> FileTreeColours {
    FileTreeColours {
        binary: SOL_RED,
        structured: SOL_BLUE,
        text: SOL_BASE0,
        unknown: SOL_BASE01,
        directory: SOL_YELLOW,
        symlink: SOL_CYAN,
    }
}

fn solarized_dark_tab_bar() -> TabBarColours {
    TabBarColours {
        // Req 23.9: accent-tinted active tab, distinct from inactive.
        active_bg: SOL_D_TAB_ACTIVE,
        inactive_bg: SOL_D_WINDOW,
        active_text: SOL_BASE1,
        inactive_text: SOL_BASE0,
        modified_indicator: SOL_YELLOW,
        close_button: SOL_BASE01,
        drop_target: ColourRGBA::rgba(0x26, 0x8B, 0xD2, 80),
    }
}

fn solarized_dark_gutter() -> GutterColours {
    GutterColours {
        cursor_row_border: SOL_BASE02,
        cursor_column_indicator: SOL_BASE02,
        line_number_fg: SOL_BASE0,
        line_number_bg: SOL_BASE03,
        fold_margin_bg: SOL_BASE03,
        fold_margin_fg: SOL_BASE0,
        margin_separator: SOL_BASE02,
    }
}

fn solarized_dark_ui() -> UiColours {
    UiColours {
        panel_bg: SOL_D_WINDOW,
        panel_fg: SOL_BASE1,
        panel_border: SOL_BASE02,
        // Req 23.7: raised surface = base02; distinct from window + input.
        button_bg: SOL_BASE02,
        button_fg: SOL_BASE1,
        button_hover: SOL_D_RAISED_HOVER,
        input_bg: SOL_D_INPUT,
        input_border: SOL_BASE01,
        input_fg: SOL_BASE0,
        scrollbar_track: SOL_D_WINDOW,
        scrollbar_thumb: SOL_BASE01,
        tooltip_bg: SOL_BASE02,
        tooltip_fg: SOL_BASE1,
        menu_bar_fg: SOL_BASE1,
        // Req 23.10: accent-tinted title band distinct from the base.
        primary_menu_bg: SOL_D_TITLE_BAND,
        // Req 23.8: focus ring is the chrome accent (Solarized blue).
        focus_ring: SOL_BLUE,
    }
}

// === Solarized Light ========================================================

/// Build the `Default Light` built-in as a Solarized Light instance.
///
/// Validates: theme-and-appearance Requirement 18.3, 23.7, 23.8, 23.9, 23.10
pub fn solarized_light_palette() -> ThemePalette {
    let editor = solarized_light_editor();
    let tab_bar = solarized_light_tab_bar();
    let ui = solarized_light_ui();
    let design = DesignTokens::default();
    let chrome_style = chrome_style_for(&ui, &tab_bar, &editor, &design, VisualMode::Light);
    ThemePalette {
        name: "Default Light".to_string(),
        mode: VisualMode::Light,
        editor,
        syntax: solarized_light_syntax(),
        file_tree: solarized_light_file_tree(),
        tab_bar,
        gutter: solarized_light_gutter(),
        decorations: solarized_decorations(),
        indicators: solarized_indicators(SOL_BASE00),
        ui,
        style_slots: solarized_style_slot_table(SOL_BASE00, SOL_BASE3),
        fonts: FontConfig::default(),
        design,
        elements: ElementColourMap::new(),
        chrome_style,
    }
}

fn solarized_light_editor() -> EditorColours {
    EditorColours {
        background: SOL_BASE3,
        foreground: SOL_L_FG,
        accent: SOL_BLUE,
        muted: SOL_BASE00,
        modified_indicator: SOL_ORANGE,
        current_line_background: SOL_BASE2,
        selection_secondary_background: ColourRGBA::rgba(0x26, 0x8B, 0xD2, 40),
    }
}

fn solarized_light_syntax() -> SyntaxColours {
    SyntaxColours {
        keyword: SOL_GREEN,
        comment: SOL_BASE1,
        string: SOL_CYAN,
        number: SOL_MAGENTA,
        operator: SOL_BASE00,
        type_name: SOL_YELLOW,
        function: SOL_BLUE,
        macro_name: SOL_VIOLET,
        preprocessor: SOL_ORANGE,
        default_text: SOL_L_FG,
    }
}

fn solarized_light_file_tree() -> FileTreeColours {
    FileTreeColours {
        binary: SOL_RED,
        structured: SOL_BLUE,
        text: SOL_L_FG,
        unknown: SOL_BASE00,
        directory: SOL_YELLOW,
        symlink: SOL_CYAN,
    }
}

fn solarized_light_tab_bar() -> TabBarColours {
    TabBarColours {
        active_bg: SOL_L_TAB_ACTIVE,
        inactive_bg: SOL_BASE2,
        active_text: SOL_L_FG,
        inactive_text: SOL_BASE00,
        modified_indicator: SOL_ORANGE,
        close_button: SOL_BASE1,
        drop_target: ColourRGBA::rgba(0x26, 0x8B, 0xD2, 60),
    }
}

fn solarized_light_gutter() -> GutterColours {
    GutterColours {
        cursor_row_border: SOL_BASE2,
        cursor_column_indicator: SOL_BASE2,
        line_number_fg: SOL_BASE00,
        line_number_bg: SOL_BASE3,
        fold_margin_bg: SOL_BASE3,
        fold_margin_fg: SOL_BASE00,
        margin_separator: SOL_BASE2,
    }
}

fn solarized_light_ui() -> UiColours {
    UiColours {
        panel_bg: SOL_L_WINDOW,
        panel_fg: SOL_L_FG,
        panel_border: SOL_BASE2,
        // Req 23.7: raised surface = base2; distinct from window + input.
        button_bg: SOL_BASE2,
        button_fg: SOL_L_FG,
        button_hover: SOL_L_RAISED_HOVER,
        input_bg: SOL_L_INPUT,
        input_border: SOL_BASE1,
        input_fg: SOL_L_FG,
        scrollbar_track: SOL_L_WINDOW,
        scrollbar_thumb: SOL_BASE1,
        tooltip_bg: SOL_BASE2,
        tooltip_fg: SOL_L_FG,
        menu_bar_fg: SOL_L_TITLE_FG,
        // Req 23.10: accent-tinted title band distinct from the base.
        primary_menu_bg: SOL_L_TITLE_BAND,
        // Req 23.8: focus ring is the chrome accent (Solarized blue).
        focus_ring: SOL_BLUE,
    }
}

// === Shared domain groups ===================================================

fn solarized_decorations() -> DecorationColours {
    DecorationColours {
        search_highlight: ColourRGBA::rgba(0xB5, 0x89, 0x00, 70),
        error_underline: SOL_RED,
        warning_underline: SOL_ORANGE,
        info_underline: SOL_BLUE,
        change_added: SOL_GREEN,
        change_modified: SOL_YELLOW,
        change_deleted: SOL_RED,
        bookmark: SOL_BLUE,
    }
}

fn solarized_indicators(user_default: ColourRGBA) -> IndicatorColours {
    IndicatorColours {
        find_match: ColourRGBA::rgba(0xB5, 0x89, 0x00, 60),
        brace_match: SOL_GREEN,
        brace_mismatch: SOL_RED,
        hotspot_underline: SOL_BLUE,
        user_defined: [user_default; 32],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solarized_dark_has_expected_identity() {
        // Validates: Requirement 18.3 -- name/mode retained, Solarized colours.
        let p = solarized_dark_palette();
        assert_eq!(p.name, "Default Dark");
        assert_eq!(p.mode, VisualMode::Dark);
        assert_eq!(p.editor.background, SOL_BASE03);
        assert_eq!(p.editor.foreground, SOL_BASE0);
        assert_eq!(p.editor.accent, SOL_BLUE);
    }

    #[test]
    fn solarized_light_has_expected_identity() {
        // Validates: Requirement 18.3 -- name/mode retained, Solarized colours.
        let p = solarized_light_palette();
        assert_eq!(p.name, "Default Light");
        assert_eq!(p.mode, VisualMode::Light);
        assert_eq!(p.editor.background, SOL_BASE3);
        // Body text is the AA-deepened Solarized base01 role tone (Req 23.11).
        assert_eq!(p.editor.foreground, SOL_L_FG);
        assert_eq!(p.editor.accent, SOL_BLUE);
    }

    #[test]
    fn solarized_dark_three_level_background_hierarchy_is_distinct() {
        // Validates: Requirement 23.7 -- window/base, raised, inset distinct.
        let p = solarized_dark_palette();
        let window = p.ui.panel_bg;
        let base = p.editor.background;
        let raised = p.ui.button_bg;
        let inset = p.ui.input_bg;
        assert_ne!(window, raised);
        assert_ne!(raised, inset);
        assert_ne!(window, inset);
        assert_ne!(base, raised);
    }

    #[test]
    fn solarized_light_three_level_background_hierarchy_is_distinct() {
        // Validates: Requirement 23.7 -- window/base, raised, inset distinct.
        let p = solarized_light_palette();
        assert_ne!(p.ui.panel_bg, p.ui.button_bg);
        assert_ne!(p.ui.button_bg, p.ui.input_bg);
        assert_ne!(p.ui.panel_bg, p.ui.input_bg);
    }

    #[test]
    fn solarized_focus_ring_is_the_accent() {
        // Validates: Requirement 23.8 -- focus ring == accent (Solarized blue).
        for p in [solarized_dark_palette(), solarized_light_palette()] {
            assert_eq!(p.ui.focus_ring, p.editor.accent);
            assert_eq!(p.ui.focus_ring, SOL_BLUE);
        }
    }

    #[test]
    fn solarized_active_tab_distinct_from_inactive() {
        // Validates: Requirement 23.9 -- accent-tinted active tab distinct.
        for p in [solarized_dark_palette(), solarized_light_palette()] {
            assert_ne!(p.tab_bar.active_bg, p.tab_bar.inactive_bg);
        }
    }

    #[test]
    fn solarized_title_band_distinct_from_base_panel() {
        // Validates: Requirement 23.10 -- accent-tinted title band distinct.
        for p in [solarized_dark_palette(), solarized_light_palette()] {
            assert_ne!(p.ui.primary_menu_bg, p.ui.panel_bg);
            assert_ne!(p.ui.primary_menu_bg, p.editor.background);
        }
    }

    #[test]
    fn solarized_palettes_have_no_contrast_warnings() {
        // Validates: Requirement 23.11 -- no new below-AA text pair.
        for p in [solarized_dark_palette(), solarized_light_palette()] {
            let warnings = crate::check_theme_contrast(&p);
            assert!(warnings.is_empty(), "contrast warnings: {warnings:?}");
        }
    }
}
