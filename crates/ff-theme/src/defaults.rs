//! Built-in default palettes for all three visual modes.
//!
//! These defaults are compiled into the binary so the workbench can
//! always start even if no theme file is available on disk.

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
use crate::style_slot::{StyleSlot, StyleSlotTable};

/// Derive the egui-native chrome layer for a palette from its retained chrome
/// groups (Phase 1: `ui`/`tab_bar`/`editor` + design + mode). Keeps the chrome
/// layer consistent with the authoring groups so Phase 1 has no visible change.
fn chrome_style_for(
    ui: &UiColours,
    tab_bar: &TabBarColours,
    editor: &EditorColours,
    design: &DesignTokens,
    mode: VisualMode,
) -> ChromeStyle {
    ChromeStyle::from_palette_parts(ui, tab_bar, editor, design, mode)
}

/// Build the default dark mode palette.
///
/// CR-CH-056 Phase 2: the `Default Dark` built-in is now a Solarized Dark
/// instance (name and `VisualMode::Dark` retained). The colour data lives in the
/// sibling `defaults_solarized` module.
pub fn dark_palette() -> ThemePalette {
    crate::defaults_solarized::solarized_dark_palette()
}

/// Build the default light mode palette.
///
/// CR-CH-056 Phase 2: the `Default Light` built-in is now a Solarized Light
/// instance (name and `VisualMode::Light` retained). The colour data lives in
/// the sibling `defaults_solarized` module.
pub fn light_palette() -> ThemePalette {
    crate::defaults_solarized::solarized_light_palette()
}

/// Build the default high-contrast mode palette.
///
/// All foreground/background pairs meet WCAG AAA (7:1) contrast ratio.
pub fn high_contrast_palette() -> ThemePalette {
    let editor = high_contrast_editor_colours();
    let tab_bar = high_contrast_tab_bar_colours();
    let ui = high_contrast_ui_colours();
    let design = DesignTokens::default();
    let chrome_style = chrome_style_for(&ui, &tab_bar, &editor, &design, VisualMode::HighContrast);
    ThemePalette {
        name: "Default High Contrast".to_string(),
        mode: VisualMode::HighContrast,
        editor,
        syntax: high_contrast_syntax_colours(),
        file_tree: high_contrast_file_tree_colours(),
        tab_bar,
        gutter: high_contrast_gutter_colours(),
        decorations: high_contrast_decoration_colours(),
        indicators: high_contrast_indicator_colours(),
        ui,
        style_slots: default_style_slot_table(VisualMode::HighContrast),
        fonts: FontConfig::default(),
        design,
        elements: ElementColourMap::new(),
        chrome_style,
    }
}

/// Build the Legacy IBM 3270 / ISPF palette.
///
/// Follows the ISPF semantic attribute system on a black background:
/// - Blue   — normal text / informational labels
/// - Turquoise — input fields (user-typed content)
/// - Yellow — commands and interactive actions
/// - Red    — errors and alarms
/// - Pink   — warnings and attention items
/// - Green  — success and positive status
/// - White  — intense headings and titles
pub fn legacy_palette() -> ThemePalette {
    let editor = legacy_editor_colours();
    let tab_bar = legacy_tab_bar_colours();
    let ui = legacy_ui_colours();
    let design = DesignTokens::default();
    let chrome_style = chrome_style_for(&ui, &tab_bar, &editor, &design, VisualMode::Legacy);
    ThemePalette {
        name: "Legacy (ISPF 3270)".to_string(),
        mode: VisualMode::Legacy,
        editor,
        syntax: legacy_syntax_colours(),
        file_tree: legacy_file_tree_colours(),
        tab_bar,
        gutter: legacy_gutter_colours(),
        decorations: legacy_decoration_colours(),
        indicators: legacy_indicator_colours(),
        ui,
        style_slots: default_style_slot_table(VisualMode::Legacy),
        fonts: FontConfig::default(),
        design,
        elements: ElementColourMap::new(),
        chrome_style,
    }
}

/// Get the default palette for a given visual mode.
pub fn default_palette_for_mode(mode: VisualMode) -> ThemePalette {
    match mode {
        VisualMode::Dark => dark_palette(),
        VisualMode::Light => light_palette(),
        VisualMode::HighContrast => high_contrast_palette(),
        VisualMode::Legacy => legacy_palette(),
    }
}

/// Build the `Default Legacy` palette: a canonical, always-available theme based
/// on the Legacy ISPF 3270 look. Its colours are identical to `legacy_palette()`;
/// only the name differs, so users can freely edit the `Legacy (ISPF 3270)` theme
/// or their own themes and still fall back to a known-good baseline.
///
/// Validates: theme-and-appearance Requirement 18.1
pub fn default_legacy_palette() -> ThemePalette {
    ThemePalette {
        name: "Default Legacy".to_string(),
        ..legacy_palette()
    }
}

/// The canonical Fallback_Theme used when the configured active theme cannot be
/// resolved (missing file, invalid TOML, unresolved `base`). Returns
/// `default_legacy_palette()`.
///
/// Validates: theme-and-appearance Requirement 18.2, 18.6
pub fn fallback_palette() -> ThemePalette {
    default_legacy_palette()
}

/// Build the `Legacy Soft` built-in: a softer phosphor variant of the Legacy
/// ISPF look (CR-CH-056 Phase 2). The colour data lives in the sibling
/// `defaults_legacy_soft` module.
///
/// Validates: theme-and-appearance Requirement 18.3
pub fn legacy_soft_palette() -> ThemePalette {
    crate::defaults_legacy_soft::legacy_soft_palette()
}

// ─── Dark / Light Mode Colours ──────────────────────────────────────────────
// CR-CH-056 Phase 2: the Default Dark / Default Light built-ins are Solarized
// instances; their colour builders live in `defaults_solarized.rs`. The former
// Catppuccin builders here were removed when `dark_palette()` / `light_palette()`
// began delegating to the sibling module.

// ─── High-Contrast Mode Colours ─────────────────────────────────────────────
// All fg/bg pairs achieve WCAG AAA (7:1) contrast ratio minimum.

fn high_contrast_editor_colours() -> EditorColours {
    EditorColours {
        background: ColourRGBA::rgb(0, 0, 0),
        foreground: ColourRGBA::rgb(255, 255, 255),
        accent: ColourRGBA::rgb(0, 255, 255),
        muted: ColourRGBA::rgb(170, 170, 170),
        modified_indicator: ColourRGBA::rgb(255, 255, 0),
        current_line_background: ColourRGBA::rgb(20, 20, 20),
        selection_secondary_background: ColourRGBA::rgba(0, 255, 255, 60),
    }
}

fn high_contrast_syntax_colours() -> SyntaxColours {
    SyntaxColours {
        keyword: ColourRGBA::rgb(255, 128, 255),
        comment: ColourRGBA::rgb(128, 255, 128),
        string: ColourRGBA::rgb(255, 200, 100),
        number: ColourRGBA::rgb(180, 255, 180),
        operator: ColourRGBA::rgb(0, 255, 255),
        type_name: ColourRGBA::rgb(255, 255, 100),
        function: ColourRGBA::rgb(100, 200, 255),
        macro_name: ColourRGBA::rgb(255, 180, 255),
        preprocessor: ColourRGBA::rgb(255, 128, 128),
        default_text: ColourRGBA::rgb(255, 255, 255),
    }
}

fn high_contrast_file_tree_colours() -> FileTreeColours {
    FileTreeColours {
        binary: ColourRGBA::rgb(255, 100, 100),
        structured: ColourRGBA::rgb(100, 200, 255),
        text: ColourRGBA::rgb(255, 255, 255),
        unknown: ColourRGBA::rgb(170, 170, 170),
        directory: ColourRGBA::rgb(255, 255, 100),
        symlink: ColourRGBA::rgb(0, 255, 255),
    }
}

fn high_contrast_tab_bar_colours() -> TabBarColours {
    TabBarColours {
        active_bg: ColourRGBA::rgb(0, 0, 0),
        inactive_bg: ColourRGBA::rgb(20, 20, 20),
        active_text: ColourRGBA::rgb(255, 255, 255),
        inactive_text: ColourRGBA::rgb(170, 170, 170),
        modified_indicator: ColourRGBA::rgb(255, 255, 0),
        close_button: ColourRGBA::rgb(255, 255, 255),
        drop_target: ColourRGBA::rgba(0, 255, 255, 100),
    }
}

fn high_contrast_gutter_colours() -> GutterColours {
    GutterColours {
        cursor_row_border: ColourRGBA::rgb(255, 255, 255),
        cursor_column_indicator: ColourRGBA::rgb(255, 255, 255),
        line_number_fg: ColourRGBA::rgb(170, 170, 170),
        line_number_bg: ColourRGBA::rgb(0, 0, 0),
        fold_margin_bg: ColourRGBA::rgb(0, 0, 0),
        fold_margin_fg: ColourRGBA::rgb(255, 255, 255),
        margin_separator: ColourRGBA::rgb(255, 255, 255),
    }
}

fn high_contrast_decoration_colours() -> DecorationColours {
    DecorationColours {
        search_highlight: ColourRGBA::rgba(255, 255, 0, 100),
        error_underline: ColourRGBA::rgb(255, 0, 0),
        warning_underline: ColourRGBA::rgb(255, 200, 0),
        info_underline: ColourRGBA::rgb(0, 200, 255),
        change_added: ColourRGBA::rgb(0, 255, 0),
        change_modified: ColourRGBA::rgb(255, 255, 0),
        change_deleted: ColourRGBA::rgb(255, 0, 0),
        bookmark: ColourRGBA::rgb(0, 255, 255),
    }
}

fn high_contrast_indicator_colours() -> IndicatorColours {
    IndicatorColours {
        find_match: ColourRGBA::rgba(255, 255, 0, 80),
        brace_match: ColourRGBA::rgb(0, 255, 0),
        brace_mismatch: ColourRGBA::rgb(255, 0, 0),
        hotspot_underline: ColourRGBA::rgb(0, 255, 255),
        user_defined: [ColourRGBA::rgb(170, 170, 170); 32],
    }
}

fn high_contrast_ui_colours() -> UiColours {
    UiColours {
        panel_bg: ColourRGBA::rgb(0, 0, 0),
        panel_fg: ColourRGBA::rgb(255, 255, 255),
        panel_border: ColourRGBA::rgb(255, 255, 255),
        button_bg: ColourRGBA::rgb(40, 40, 40),
        button_fg: ColourRGBA::rgb(255, 255, 255),
        button_hover: ColourRGBA::rgb(60, 60, 60),
        input_bg: ColourRGBA::rgb(20, 20, 20),
        input_border: ColourRGBA::rgb(255, 255, 255),
        input_fg: ColourRGBA::rgb(255, 255, 255),
        scrollbar_track: ColourRGBA::rgb(0, 0, 0),
        scrollbar_thumb: ColourRGBA::rgb(170, 170, 170),
        tooltip_bg: ColourRGBA::rgb(20, 20, 20),
        tooltip_fg: ColourRGBA::rgb(255, 255, 255),
        menu_bar_fg: ColourRGBA::rgb(255, 255, 255),
        primary_menu_bg: ColourRGBA::rgb(0, 0, 0),
        focus_ring: ColourRGBA::rgb(255, 255, 0), // #FFFF00 -- yellow, maximum contrast on black
    }
}

// ─── Legacy IBM 3270 / ISPF Colours ─────────────────────────────────────────
//
// ISPF assigns colours by semantic attribute type, not arbitrarily.
// Each attribute type maps to one of the 3270 logical colours:
//
//   ISPF Attribute Type          Logical Colour   RGB (normal / bright)
//   ───────────────────────────────────────────────────────────────────────────
//   Normal text / informational  Blue             #0000AA / #5555FF
//   Input fields (user types)    Turquoise        #00AAAA / #00FFFF
//   Commands / actions           Yellow           #AAAA00 / #FFFF00
//   Errors                       Red              #AA0000 / #FF0000
//   Warnings / attention         Pink/Magenta     #AA00AA / #FF00FF
//   Success / positive status    Green            #00AA00 / #00FF00
//   Intense headings / titles    White            #AAAAAA / #FFFFFF

// Normal-intensity 3270 colours (standard attribute)
const ISPF_BG: ColourRGBA = ColourRGBA::rgb(0, 0, 0);
const ISPF_BG_ALT: ColourRGBA = ColourRGBA::rgb(0, 0, 28);
const ISPF_BLUE: ColourRGBA = ColourRGBA::rgb(0, 0, 170); // normal text / labels
                                                          // Toned-down primary option-menu / title band (CR-CH-056 Req 13.2, amended from
                                                          // the full ISPF structural blue #0000AA to a muted deep navy #000060).
const ISPF_MENU_BAND: ColourRGBA = ColourRGBA::rgb(0, 0, 96); // #000060
const ISPF_TURQUOISE: ColourRGBA = ColourRGBA::rgb(0, 170, 170); // input fields
const ISPF_YELLOW: ColourRGBA = ColourRGBA::rgb(170, 170, 0); // commands / actions
const ISPF_RED: ColourRGBA = ColourRGBA::rgb(170, 0, 0); // errors
const ISPF_PINK: ColourRGBA = ColourRGBA::rgb(170, 0, 170); // warnings / attention
const ISPF_GREEN: ColourRGBA = ColourRGBA::rgb(0, 170, 0); // success / positive
const ISPF_WHITE: ColourRGBA = ColourRGBA::rgb(170, 170, 170); // headings / titles

// High-intensity (bright) variants — used for emphasis within each category
const ISPF_BLUE_HI: ColourRGBA = ColourRGBA::rgb(120, 120, 255);
const ISPF_TURQUOISE_HI: ColourRGBA = ColourRGBA::rgb(0, 255, 255);
const ISPF_YELLOW_HI: ColourRGBA = ColourRGBA::rgb(255, 255, 0);
const ISPF_RED_HI: ColourRGBA = ColourRGBA::rgb(255, 0, 0);
const ISPF_PINK_HI: ColourRGBA = ColourRGBA::rgb(255, 0, 255);
const ISPF_GREEN_HI: ColourRGBA = ColourRGBA::rgb(0, 255, 0);
const ISPF_WHITE_HI: ColourRGBA = ColourRGBA::rgb(255, 255, 255);

fn legacy_editor_colours() -> EditorColours {
    EditorColours {
        // Black background — the defining characteristic of a 3270 terminal
        background: ISPF_BG,
        // Normal text is Green — positive / informational body text in ISPF
        foreground: ISPF_GREEN_HI,
        // Accent (cursor, active element) is Yellow — command / action colour
        accent: ISPF_YELLOW_HI,
        // Muted (de-emphasised) stays in the blue family
        muted: ISPF_BLUE,
        // Modified indicator is Yellow — signals a changed / dirty field
        modified_indicator: ISPF_YELLOW_HI,
        // Current line tint — barely-visible dark blue wash
        current_line_background: ISPF_BG_ALT,
        // Selection uses turquoise at low opacity — input-field colour
        selection_secondary_background: ColourRGBA::rgba(0, 170, 170, 55),
    }
}

fn legacy_syntax_colours() -> SyntaxColours {
    SyntaxColours {
        // Keywords are Yellow — commands / interactive actions in ISPF
        keyword: ISPF_YELLOW_HI,
        // Comments are Blue -- informational / non-interactive labels.
        // Bright blue for legibility on black (CR-CH-020 Req 22: #0000AA is
        // ~1.58:1 on black; #7878FF is ~5.93:1).
        comment: ISPF_BLUE_HI,
        // String literals are Green — positive / data content
        string: ISPF_GREEN,
        // Numbers are White — heading-intensity data values
        number: ISPF_WHITE_HI,
        // Operators are Turquoise — interactive / input-adjacent elements
        operator: ISPF_TURQUOISE,
        // Type names are Pink — warning / attention category
        type_name: ISPF_PINK_HI,
        // Functions are Yellow (bright) — action / command category
        function: ISPF_YELLOW,
        // Macros are Pink — attention / special processing
        macro_name: ISPF_PINK,
        // Preprocessor directives are Red — error / alarm category
        preprocessor: ISPF_RED_HI,
        // Default unclassified text is Blue — normal informational text
        default_text: ISPF_BLUE_HI,
    }
}

fn legacy_file_tree_colours() -> FileTreeColours {
    FileTreeColours {
        // Binary files are Red — error / non-editable alarm
        binary: ISPF_RED,
        // Structured files are Turquoise — input / interactive
        structured: ISPF_TURQUOISE_HI,
        // Plain text files are Blue — normal informational
        text: ISPF_BLUE_HI,
        // Unknown files are Blue -- bright for legibility on black (Req 22).
        unknown: ISPF_BLUE_HI,
        // Directories are White — heading / title emphasis
        directory: ISPF_WHITE_HI,
        // Symlinks are Turquoise — indirect / input-adjacent
        symlink: ISPF_TURQUOISE,
    }
}

fn legacy_tab_bar_colours() -> TabBarColours {
    TabBarColours {
        active_bg: ISPF_BG_ALT,
        inactive_bg: ISPF_BG,
        // Active tab title is White — heading / title emphasis
        active_text: ISPF_WHITE_HI,
        // Inactive tab title is Blue — normal informational
        inactive_text: ISPF_BLUE_HI,
        // Modified indicator is Yellow — changed / action pending
        modified_indicator: ISPF_YELLOW_HI,
        // Close button is White — neutral heading colour
        close_button: ISPF_WHITE,
        drop_target: ColourRGBA::rgba(0, 170, 170, 80),
    }
}

fn legacy_gutter_colours() -> GutterColours {
    GutterColours {
        // Cursor row border is Turquoise — marks the active input position
        cursor_row_border: ISPF_TURQUOISE,
        cursor_column_indicator: ISPF_TURQUOISE,
        // Line numbers are bright Blue — legible on black (Req 22)
        line_number_fg: ISPF_BLUE_HI,
        line_number_bg: ISPF_BG,
        fold_margin_bg: ISPF_BG,
        // Fold margin foreground: bright blue for legibility on black (Req 22)
        fold_margin_fg: ISPF_BLUE_HI,
        // Margin separator is a bright Blue line — legible on black (Req 22)
        margin_separator: ISPF_BLUE_HI,
    }
}

fn legacy_decoration_colours() -> DecorationColours {
    DecorationColours {
        // Search highlight is Yellow — command / action attention
        search_highlight: ColourRGBA::rgba(170, 170, 0, 85),
        // Error underline is Red — error / alarm
        error_underline: ISPF_RED_HI,
        // Warning underline is Pink — warning / attention
        warning_underline: ISPF_PINK_HI,
        // Info underline is Turquoise — input / interactive hint
        info_underline: ISPF_TURQUOISE_HI,
        // Change added is Green — success / positive
        change_added: ISPF_GREEN_HI,
        // Change modified is Yellow — action / changed
        change_modified: ISPF_YELLOW_HI,
        // Change deleted is Red — error / removal
        change_deleted: ISPF_RED_HI,
        // Bookmark is Turquoise — interactive marker
        bookmark: ISPF_TURQUOISE_HI,
    }
}

fn legacy_indicator_colours() -> IndicatorColours {
    IndicatorColours {
        // Find match is Yellow — command result / action
        find_match: ColourRGBA::rgba(170, 170, 0, 75),
        // Brace match is Green — success / valid pair
        brace_match: ISPF_GREEN_HI,
        // Brace mismatch is Red — error
        brace_mismatch: ISPF_RED_HI,
        // Hotspot underline is Yellow — actionable / command
        hotspot_underline: ISPF_YELLOW_HI,
        user_defined: [ISPF_BLUE_HI; 32],
    }
}

fn legacy_ui_colours() -> UiColours {
    UiColours {
        // Panel background is black
        panel_bg: ISPF_BG,
        // Panel foreground is Turquoise — field labels adjacent to input fields
        panel_fg: ISPF_TURQUOISE_HI,
        // Panel border is Blue — structural / informational
        panel_border: ISPF_BLUE,
        // Button background is a very dark blue wash
        button_bg: ISPF_BG_ALT,
        // Button text is Yellow — command / action
        button_fg: ISPF_YELLOW,
        button_hover: ColourRGBA::rgb(0, 0, 60),
        // Input field background is black
        input_bg: ISPF_BG,
        // Input field border is Turquoise — marks editable area
        input_border: ISPF_TURQUOISE,
        // Input field text is Turquoise — user-typed content
        input_fg: ISPF_TURQUOISE_HI,
        scrollbar_track: ISPF_BG,
        scrollbar_thumb: ISPF_BLUE_HI,
        // Tooltip background is a dark blue wash
        tooltip_bg: ISPF_BG_ALT,
        // Tooltip text is Turquoise — informational label adjacent to input
        tooltip_fg: ISPF_TURQUOISE_HI,
        // Menu bar top-level items are White — heading / title emphasis
        menu_bar_fg: ISPF_WHITE_HI,
        // Primary menu / screen heading background: TONED DOWN from the full
        // ISPF structural blue (#0000AA) to a muted deep navy (#000060) by
        // CR-CH-056 Req 13.2 (amended), so the title band is less harsh while
        // keeping the ISPF structural-blue role. All OTHER Legacy colours are
        // byte-identical.
        primary_menu_bg: ISPF_MENU_BAND,
        // Focus ring is Yellow -- command / action colour, visible on black
        focus_ring: ISPF_YELLOW_HI,
    }
}

// ─── Style Slot Table Defaults ──────────────────────────────────────────────

fn default_style_slot_table(mode: VisualMode) -> StyleSlotTable {
    let (fg, bg) = match mode {
        VisualMode::Dark => (ColourRGBA::rgb(205, 214, 244), ColourRGBA::rgb(30, 30, 46)),
        VisualMode::Light => (ColourRGBA::rgb(76, 79, 105), ColourRGBA::rgb(239, 241, 245)),
        VisualMode::HighContrast => (ColourRGBA::rgb(255, 255, 255), ColourRGBA::rgb(0, 0, 0)),
        VisualMode::Legacy => (ISPF_BLUE_HI, ISPF_BG),
    };

    let default_slot = StyleSlot {
        foreground: fg,
        background: bg,
        font_family: None,
        bold: false,
        italic: false,
        underline: false,
        case_transform: crate::style_slot::CaseTransform::None,
    };

    StyleSlotTable::new(default_slot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_palette_is_complete() {
        // Validates: Requirement 5.5
        let palette = dark_palette();
        assert_eq!(palette.mode, VisualMode::Dark);
        assert_eq!(palette.name, "Default Dark");
    }

    #[test]
    fn light_palette_is_complete() {
        // Validates: Requirement 5.5
        let palette = light_palette();
        assert_eq!(palette.mode, VisualMode::Light);
        assert_eq!(palette.name, "Default Light");
    }

    #[test]
    fn high_contrast_palette_is_complete() {
        // Validates: Requirement 5.5
        let palette = high_contrast_palette();
        assert_eq!(palette.mode, VisualMode::HighContrast);
        assert_eq!(palette.name, "Default High Contrast");
    }

    #[test]
    fn high_contrast_fg_bg_pairs_meet_wcag_aaa() {
        // Validates: Requirement 5.6
        let palette = high_contrast_palette();
        // Check main editor fg/bg pair
        let ratio = palette
            .editor
            .foreground
            .contrast_ratio(&palette.editor.background);
        assert!(ratio >= 7.0, "Editor fg/bg ratio {ratio:.2} is below 7:1");
        // Check syntax colours against editor background
        let bg = &palette.editor.background;
        let ratio = palette.syntax.keyword.contrast_ratio(bg);
        assert!(ratio >= 7.0, "Syntax keyword ratio {ratio:.2} is below 7:1");
        let ratio = palette.syntax.comment.contrast_ratio(bg);
        assert!(ratio >= 7.0, "Syntax comment ratio {ratio:.2} is below 7:1");
        let ratio = palette.syntax.string.contrast_ratio(bg);
        assert!(ratio >= 7.0, "Syntax string ratio {ratio:.2} is below 7:1");
    }

    #[test]
    fn default_palette_for_mode_selects_correctly() {
        // Validates: Requirement 5.1
        assert_eq!(
            default_palette_for_mode(VisualMode::Dark).mode,
            VisualMode::Dark
        );
        assert_eq!(
            default_palette_for_mode(VisualMode::Light).mode,
            VisualMode::Light
        );
        assert_eq!(
            default_palette_for_mode(VisualMode::HighContrast).mode,
            VisualMode::HighContrast
        );
        assert_eq!(
            default_palette_for_mode(VisualMode::Legacy).mode,
            VisualMode::Legacy
        );
    }

    #[test]
    fn legacy_palette_is_complete() {
        // Validates: Requirement 5.5
        let palette = legacy_palette();
        assert_eq!(palette.mode, VisualMode::Legacy);
        assert_eq!(palette.name, "Legacy (ISPF 3270)");
        // Editor background must be black
        assert_eq!(palette.editor.background, ColourRGBA::rgb(0, 0, 0));
        // Default foreground is Green (ISPF normal body text colour)
        assert_eq!(palette.editor.foreground, ColourRGBA::rgb(0, 255, 0));
        // Accent is Yellow (ISPF command / action colour)
        assert_eq!(palette.editor.accent, ColourRGBA::rgb(255, 255, 0));
        // Error underline is Red
        assert_eq!(
            palette.decorations.error_underline,
            ColourRGBA::rgb(255, 0, 0)
        );
        // Warning underline is Pink
        assert_eq!(
            palette.decorations.warning_underline,
            ColourRGBA::rgb(255, 0, 255)
        );
        // Input field foreground is Turquoise
        assert_eq!(palette.ui.input_fg, ColourRGBA::rgb(0, 255, 255));
        // Menu bar text is White
        assert_eq!(palette.ui.menu_bar_fg, ColourRGBA::rgb(255, 255, 255));
        // Primary menu background: toned deep navy #000060 (CR-CH-056 Req 13.2).
        assert_eq!(palette.ui.primary_menu_bg, ColourRGBA::rgb(0, 0, 96));
    }

    // Validates: theme-and-appearance Requirement 18.1 -- Default Legacy has the
    // same colours as the Legacy palette; only the name differs.
    #[test]
    fn default_legacy_matches_legacy_colours() {
        let legacy = legacy_palette();
        let default_legacy = default_legacy_palette();
        assert_eq!(default_legacy.name, "Default Legacy");
        assert_eq!(default_legacy.mode, VisualMode::Legacy);
        // Same colour groups (compare everything except the name).
        assert_eq!(default_legacy.editor, legacy.editor);
        assert_eq!(default_legacy.syntax, legacy.syntax);
        assert_eq!(default_legacy.file_tree, legacy.file_tree);
        assert_eq!(default_legacy.tab_bar, legacy.tab_bar);
        assert_eq!(default_legacy.gutter, legacy.gutter);
        assert_eq!(default_legacy.decorations, legacy.decorations);
        assert_eq!(default_legacy.indicators, legacy.indicators);
        assert_eq!(default_legacy.ui, legacy.ui);
    }

    // Validates: theme-and-appearance Requirement 18.2/18.6 -- the canonical
    // fallback is Default Legacy.
    #[test]
    fn fallback_is_default_legacy() {
        let fb = fallback_palette();
        assert_eq!(fb.name, "Default Legacy");
        assert_eq!(fb.mode, VisualMode::Legacy);
        assert_eq!(fb.editor, default_legacy_palette().editor);
    }

    // Validates: theme-and-appearance Requirement 18.5 -- serialising the
    // Default Legacy palette and parsing it back yields an equal palette
    // (round-trip; consistent with Requirement 9.2).
    #[test]
    fn default_legacy_serialise_round_trips() {
        let original = default_legacy_palette();
        let toml = crate::serialiser::serialise(&original);
        let parsed = crate::loader::load_from_toml(&toml, VisualMode::Legacy)
            .expect("Default Legacy TOML must parse");
        assert_eq!(parsed.editor, original.editor);
        assert_eq!(parsed.ui, original.ui);
        assert_eq!(parsed.syntax, original.syntax);
        assert_eq!(parsed.decorations, original.decorations);
    }

    // === CR-CH-020: non-monochrome Dark/Light chrome + Legacy legibility =====

    // Validates: Requirement 22.1/22.2 -- Legacy blue-on-black foreground uses
    // the bright blue #7878FF (ISPF_BLUE_HI, ~5.93:1 on black) instead of the
    // barely-legible #0000AA (~1.58:1). Applies to comment, unknown file tree,
    // fold margin foreground and margin separator (line_number_fg already used
    // the bright variant).
    #[test]
    fn legacy_blue_on_black_uses_bright_blue_for_legibility() {
        let p = legacy_palette();
        assert_eq!(p.syntax.comment, ISPF_BLUE_HI);
        assert_eq!(p.file_tree.unknown, ISPF_BLUE_HI);
        assert_eq!(p.gutter.fold_margin_fg, ISPF_BLUE_HI);
        assert_eq!(p.gutter.margin_separator, ISPF_BLUE_HI);
        assert_eq!(p.gutter.line_number_fg, ISPF_BLUE_HI);
        // The bright blue clears the >= 3:1 UI-component bar on black; the dim
        // #0000AA it replaced does not.
        let black = ColourRGBA::rgb(0, 0, 0);
        assert!(
            ISPF_BLUE_HI.contrast_ratio(&black) >= 3.0,
            "bright legacy blue must be legible on black"
        );
        assert!(
            ColourRGBA::rgb(0, 0, 170).contrast_ratio(&black) < 3.0,
            "the dim blue it replaced was not legible (guards the regression)"
        );
    }

    // Validates: Requirement 22.3 -- everything else in Legacy is byte-identical
    // to the pre-CR-CH-020 look and feel (spot-check the invariants that were
    // NOT touched).
    #[test]
    fn legacy_retains_look_and_feel() {
        let p = legacy_palette();
        assert_eq!(p.editor.background, ColourRGBA::rgb(0, 0, 0)); // black
        assert_eq!(p.editor.foreground, ColourRGBA::rgb(0, 255, 0)); // green body
        assert_eq!(p.editor.accent, ColourRGBA::rgb(255, 255, 0)); // yellow command
        assert_eq!(p.ui.input_fg, ColourRGBA::rgb(0, 255, 255)); // turquoise inputs
        assert_eq!(p.ui.menu_bar_fg, ColourRGBA::rgb(255, 255, 255)); // white
        assert_eq!(p.ui.primary_menu_bg, ColourRGBA::rgb(0, 0, 96)); // toned navy title band
    }

    // Validates: Requirement 23.7 -- the Solarized Dark built-in presents a
    // three-level surface hierarchy (window base, raised, inset) with distinct
    // values. The concrete Solarized values are asserted in defaults_solarized.
    #[test]
    fn dark_chrome_has_three_level_surface_hierarchy() {
        let p = dark_palette();
        assert_ne!(p.ui.panel_bg, p.ui.button_bg);
        assert_ne!(p.ui.button_bg, p.ui.input_bg);
        assert_ne!(p.ui.panel_bg, p.ui.input_bg);
    }

    // Validates: Requirement 23.7 -- Solarized Light 3-level surface hierarchy.
    #[test]
    fn light_chrome_has_three_level_surface_hierarchy() {
        let p = light_palette();
        assert_ne!(p.ui.panel_bg, p.ui.button_bg);
        assert_ne!(p.ui.button_bg, p.ui.input_bg);
        assert_ne!(p.ui.panel_bg, p.ui.input_bg);
    }

    // Validates: Requirement 23.8 -- the focus ring is the theme accent colour
    // (Solarized blue) for Dark and Light.
    #[test]
    fn dark_and_light_focus_ring_is_accent() {
        let dark = dark_palette();
        assert_eq!(dark.ui.focus_ring, dark.editor.accent);
        assert_eq!(dark.ui.focus_ring, ColourRGBA::rgb(0x26, 0x8B, 0xD2));
        let light = light_palette();
        assert_eq!(light.ui.focus_ring, light.editor.accent);
        assert_eq!(light.ui.focus_ring, ColourRGBA::rgb(0x26, 0x8B, 0xD2));
    }

    // Validates: Requirement 23.9 -- active tab is accent-tinted and distinct
    // from the inactive tab, for Dark and Light.
    #[test]
    fn dark_and_light_active_tab_is_distinct_and_accented() {
        let dark = dark_palette();
        assert_ne!(dark.tab_bar.active_bg, dark.tab_bar.inactive_bg);
        let light = light_palette();
        assert_ne!(light.tab_bar.active_bg, light.tab_bar.inactive_bg);
    }

    // Validates: Requirement 21.4 -- the title bar background is accent-tinted
    // (primary_menu_bg differs from the plain panel background) for Dark/Light,
    // and the fg/bg pair clears WCAG AA (4.5:1).
    #[test]
    fn dark_and_light_title_bar_is_accented_and_legible() {
        for p in [dark_palette(), light_palette()] {
            assert_ne!(
                p.ui.primary_menu_bg, p.ui.panel_bg,
                "title bar must be tinted, not a flat panel"
            );
            let ratio = p.ui.menu_bar_fg.contrast_ratio(&p.ui.primary_menu_bg);
            assert!(
                ratio >= 4.5,
                "title text ratio {ratio:.2} must clear AA on the tinted bar"
            );
        }
    }

    // Validates: Requirement 21.7 -- Dark and Light palettes emit no new
    // below-threshold contrast pairs after the chrome enhancement.
    #[test]
    fn dark_and_light_palettes_have_no_contrast_warnings() {
        for p in [dark_palette(), light_palette()] {
            let warnings = crate::check_theme_contrast(&p);
            assert!(
                warnings.is_empty(),
                "unexpected contrast warnings: {warnings:?}"
            );
        }
    }

    // Validates: Requirement 21.8 / 23.11 -- High Contrast is unchanged; it still
    // meets its stronger bar and produces no warnings.
    #[test]
    fn high_contrast_unchanged_no_warnings() {
        let warnings = crate::check_theme_contrast(&high_contrast_palette());
        assert!(warnings.is_empty(), "HC warnings: {warnings:?}");
    }

    // === CR-CH-056 Phase 2 (Task 30) =========================================

    // Validates: Requirement 13.2 -- the Legacy primary option-menu / title band
    // is toned from the full ISPF blue #0000AA to the muted navy #000060.
    #[test]
    fn legacy_primary_menu_band_is_toned_navy() {
        assert_eq!(
            legacy_palette().ui.primary_menu_bg,
            ColourRGBA::rgb(0, 0, 96)
        );
        // The toned value is NOT the former full-intensity ISPF structural blue.
        assert_ne!(
            legacy_palette().ui.primary_menu_bg,
            ColourRGBA::rgb(0, 0, 170)
        );
    }

    // Validates: Requirement 18.1 / 13.2 -- EVERY Legacy colour OTHER than the
    // toned primary-menu band is byte-identical to the historical ISPF palette.
    #[test]
    fn legacy_non_band_colours_are_byte_identical() {
        let p = legacy_palette();
        // Editor group (unchanged).
        assert_eq!(p.editor.background, ISPF_BG);
        assert_eq!(p.editor.foreground, ISPF_GREEN_HI);
        assert_eq!(p.editor.accent, ISPF_YELLOW_HI);
        assert_eq!(p.editor.muted, ISPF_BLUE);
        // ui group: everything except primary_menu_bg is unchanged.
        assert_eq!(p.ui.panel_bg, ISPF_BG);
        assert_eq!(p.ui.panel_fg, ISPF_TURQUOISE_HI);
        assert_eq!(p.ui.panel_border, ISPF_BLUE);
        assert_eq!(p.ui.button_bg, ISPF_BG_ALT);
        assert_eq!(p.ui.button_fg, ISPF_YELLOW);
        assert_eq!(p.ui.input_border, ISPF_TURQUOISE);
        assert_eq!(p.ui.input_fg, ISPF_TURQUOISE_HI);
        assert_eq!(p.ui.menu_bar_fg, ISPF_WHITE_HI);
        assert_eq!(p.ui.focus_ring, ISPF_YELLOW_HI);
        // The ONLY changed ui field is the toned band.
        assert_eq!(p.ui.primary_menu_bg, ColourRGBA::rgb(0, 0, 96));
    }

    // Validates: Requirement 23.5 (Task 30.3) -- the Legacy ChromeStyle carries
    // the ISPF slider colours (turquoise track #00AAAA, yellow handle #FFFF00) so
    // the Phase 1 slider regression (near-black on black) is resolved WITHOUT a
    // seam hack. Track = ui.input_border (turquoise); handle stroke = editor
    // accent (yellow).
    #[test]
    fn legacy_chrome_style_carries_ispf_slider_colours() {
        let chrome = legacy_palette().chrome_style;
        let v = &chrome.style.visuals;
        // Slider track (inactive widget fill) is ISPF turquoise, not near-black.
        assert_eq!(
            v.widgets.inactive.bg_fill,
            egui::Color32::from_rgb(0, 170, 170)
        );
        assert_ne!(v.widgets.inactive.bg_fill, egui::Color32::BLACK);
        // Handle stroke is ISPF yellow.
        assert_eq!(
            v.widgets.inactive.fg_stroke.color,
            egui::Color32::from_rgb(255, 255, 0)
        );
    }
}
