//! WCAG contrast checking for theme palettes.
//!
//! Provides `check_theme_contrast` which iterates all text/background token
//! pairs in a `ThemePalette` and returns a `ContrastWarning` for every pair
//! that falls below the WCAG AA threshold of 4.5:1.

use crate::colour::ColourRGBA;
use crate::palette::ThemePalette;

/// A contrast ratio failure for a named foreground/background pair.
#[derive(Debug, Clone, PartialEq)]
pub struct ContrastWarning {
    /// Human-readable name of the colour pair (e.g. `"editor.foreground / editor.background"`).
    pub pair_name: &'static str,
    /// The foreground colour that was tested.
    pub foreground: ColourRGBA,
    /// The background colour that was tested.
    pub background: ColourRGBA,
    /// The actual contrast ratio (always < threshold when returned).
    pub ratio: f64,
    /// The required minimum ratio (4.5 for normal text, 3.0 for large/UI).
    pub required: f64,
}

/// Check all text and interactive-element colour pairs in `palette` against
/// WCAG AA thresholds and return one `ContrastWarning` per failing pair.
///
/// - Normal text pairs require >= 4.5:1 (WCAG AA for text below 18pt).
/// - UI element pairs (buttons, borders) require >= 3.0:1 (WCAG AA for
///   large text and UI components).
///
/// The palette is never rejected -- callers should emit warnings via the
/// logging subsystem and continue loading.
///
/// # Validates
/// Requirement 1.3, 1.4 (accessibility)
pub fn check_theme_contrast(palette: &ThemePalette) -> Vec<ContrastWarning> {
    let mut warnings = Vec::new();

    // Text pairs -- threshold 4.5:1
    let text_pairs: &[(&'static str, ColourRGBA, ColourRGBA)] = &[
        (
            "editor.foreground / editor.background",
            palette.editor.foreground,
            palette.editor.background,
        ),
        (
            "ui.panel_fg / ui.panel_bg",
            palette.ui.panel_fg,
            palette.ui.panel_bg,
        ),
        (
            "ui.button_fg / ui.button_bg",
            palette.ui.button_fg,
            palette.ui.button_bg,
        ),
        (
            "ui.input_fg / ui.input_bg",
            palette.ui.input_fg,
            palette.ui.input_bg,
        ),
        (
            "ui.tooltip_fg / ui.tooltip_bg",
            palette.ui.tooltip_fg,
            palette.ui.tooltip_bg,
        ),
        (
            "tab_bar.active_text / tab_bar.active_bg",
            palette.tab_bar.active_text,
            palette.tab_bar.active_bg,
        ),
    ];

    for (name, fg, bg) in text_pairs {
        let ratio = fg.contrast_ratio(bg);
        if ratio < 4.5 {
            warnings.push(ContrastWarning {
                pair_name: name,
                foreground: *fg,
                background: *bg,
                ratio,
                required: 4.5,
            });
        }
    }

    // UI chrome pairs -- threshold 3.0:1 (WCAG AA for large text / UI components)
    let ui_pairs: &[(&'static str, ColourRGBA, ColourRGBA)] = &[
        (
            "chrome.line_number_fg / chrome.line_number_bg",
            palette.chrome.line_number_fg,
            palette.chrome.line_number_bg,
        ),
        (
            "tab_bar.inactive_text / tab_bar.inactive_bg",
            palette.tab_bar.inactive_text,
            palette.tab_bar.inactive_bg,
        ),
    ];

    for (name, fg, bg) in ui_pairs {
        let ratio = fg.contrast_ratio(bg);
        if ratio < 3.0 {
            warnings.push(ContrastWarning {
                pair_name: name,
                foreground: *fg,
                background: *bg,
                ratio,
                required: 3.0,
            });
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::ColourRGBA;
    use crate::defaults;

    #[test]
    fn check_theme_contrast_dark_palette_passes() {
        // Validates: Requirement 1.3 (accessibility) -- dark theme meets 4.5:1 for all text pairs
        let palette = defaults::dark_palette();
        let warnings = check_theme_contrast(&palette);
        assert!(
            warnings.is_empty(),
            "Dark palette has contrast failures: {warnings:?}"
        );
    }

    #[test]
    fn check_theme_contrast_light_palette_passes() {
        // Validates: Requirement 1.3 (accessibility) -- light theme meets 4.5:1 for all text pairs
        let palette = defaults::light_palette();
        let warnings = check_theme_contrast(&palette);
        assert!(
            warnings.is_empty(),
            "Light palette has contrast failures: {warnings:?}"
        );
    }

    #[test]
    fn check_theme_contrast_high_contrast_palette_passes() {
        // Validates: Requirement 1.3, 1.5 (accessibility) -- high-contrast theme meets 4.5:1
        let palette = defaults::high_contrast_palette();
        let warnings = check_theme_contrast(&palette);
        assert!(
            warnings.is_empty(),
            "High-contrast palette has contrast failures: {warnings:?}"
        );
    }

    #[test]
    fn check_theme_contrast_warns_on_low_contrast_pair() {
        // Validates: Requirement 1.4 (accessibility) -- low-contrast custom theme emits warning
        let mut palette = defaults::dark_palette();
        // Force editor fg to nearly match bg (ratio ~1.0)
        palette.editor.foreground = ColourRGBA::rgb(31, 31, 47); // almost same as bg (30,30,46)
        let warnings = check_theme_contrast(&palette);
        assert!(
            !warnings.is_empty(),
            "Expected a contrast warning for near-identical fg/bg"
        );
        let w = &warnings[0];
        assert_eq!(w.pair_name, "editor.foreground / editor.background");
        assert!(w.ratio < 4.5);
        assert_eq!(w.required, 4.5);
    }

    #[test]
    fn contrast_warning_carries_correct_colours() {
        // Validates: Requirement 1.4 (accessibility) -- warning struct has correct fg/bg fields
        let mut palette = defaults::dark_palette();
        let bad_fg = ColourRGBA::rgb(35, 35, 50);
        palette.editor.foreground = bad_fg;
        let warnings = check_theme_contrast(&palette);
        let w = warnings
            .iter()
            .find(|w| w.pair_name == "editor.foreground / editor.background")
            .expect("expected editor fg/bg warning");
        assert_eq!(w.foreground, bad_fg);
        assert_eq!(w.background, palette.editor.background);
    }

    #[test]
    fn check_theme_contrast_button_pair_warns_when_low() {
        // Validates: Requirement 1.2 (accessibility) -- button fg/bg pair is checked
        let mut palette = defaults::dark_palette();
        // Make button fg same as bg
        palette.ui.button_fg = palette.ui.button_bg;
        let warnings = check_theme_contrast(&palette);
        assert!(
            warnings
                .iter()
                .any(|w| w.pair_name == "ui.button_fg / ui.button_bg"),
            "Expected button contrast warning"
        );
    }
}
