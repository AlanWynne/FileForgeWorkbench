//! Resolved colour cache for visual elements.

use crate::types::ColourRGBA;

/// Colours resolved from the active theme for all visual elements.
///
/// Refreshed on theme change events.
///
/// Addresses: Requirement 2 AC 2.7–2.9, Requirement 3 AC 3.6,
///            Requirement 4 AC 4.4, Requirement 5, Requirement 6 AC 6.8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedColours {
    /// Foreground colour for whitespace glyphs (dot, arrow, strikeout).
    pub whitespace_foreground: ColourRGBA,
    /// Background colour for whitespace glyphs (optional highlight).
    pub whitespace_background: Option<ColourRGBA>,
    /// Colour for inactive indent guide lines.
    pub indent_guide: ColourRGBA,
    /// Colour for the active (highlighted) indent guide.
    pub indent_guide_highlight: ColourRGBA,
    /// Colour for single-edge line or background shading.
    pub edge_colour: ColourRGBA,
    /// Colour for wrap marker glyphs.
    pub wrap_marker: ColourRGBA,
}

impl Default for ResolvedColours {
    fn default() -> Self {
        // Default fallback colours (grey tones) used when no theme is available.
        let muted = ColourRGBA {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        };
        Self {
            whitespace_foreground: muted,
            whitespace_background: None,
            indent_guide: muted,
            indent_guide_highlight: ColourRGBA {
                r: 100,
                g: 149,
                b: 237,
                a: 255,
            },
            edge_colour: muted,
            wrap_marker: muted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_colours_have_full_alpha_for_foreground() {
        // Validates: Requirement 2.9
        let colours = ResolvedColours::default();
        assert_eq!(colours.whitespace_foreground.a, 255);
    }

    #[test]
    fn default_whitespace_background_is_none() {
        // Validates: Requirement 2.8
        let colours = ResolvedColours::default();
        assert_eq!(colours.whitespace_background, None);
    }
}
