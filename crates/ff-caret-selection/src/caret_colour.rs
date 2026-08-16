//! Caret colour model.
//!
//! Defines `CaretColours` — the primary and additional caret colours,
//! plus block-caret text inversion logic.

use crate::colour::ColourRGBA;

/// Holds caret colours for primary and additional carets.
///
/// - Primary colour: used for the main caret (from the main SelectionRange).
/// - Additional colour: used for non-main carets in multi-caret scenarios.
///
/// Addresses: Requirement 2, criteria 2.1–2.7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaretColours {
    /// Colour for the primary caret (element: Caret). Default: black (#000000).
    primary: ColourRGBA,
    /// Colour for additional carets (element: CaretAdditional). Default: grey (#7F7F7F).
    additional: ColourRGBA,
}

impl CaretColours {
    /// Creates caret colours with custom primary and additional colours.
    pub fn new(primary: ColourRGBA, additional: ColourRGBA) -> Self {
        Self {
            primary,
            additional,
        }
    }

    /// Returns the appropriate colour based on whether this is the primary caret.
    ///
    /// Addresses: Requirement 2, criteria 2.1, 2.3
    pub fn colour_for(&self, is_primary: bool) -> ColourRGBA {
        if is_primary {
            self.primary
        } else {
            self.additional
        }
    }

    /// Returns the primary caret colour.
    pub fn primary(&self) -> ColourRGBA {
        self.primary
    }

    /// Sets the primary caret colour.
    pub fn set_primary(&mut self, colour: ColourRGBA) {
        self.primary = colour;
    }

    /// Returns the additional caret colour.
    pub fn additional(&self) -> ColourRGBA {
        self.additional
    }

    /// Sets the additional caret colour.
    pub fn set_additional(&mut self, colour: ColourRGBA) {
        self.additional = colour;
    }

    /// Computes the inverse text colour for block-caret rendering.
    ///
    /// When a Block-style caret is drawn, the character underneath must be
    /// rendered in a contrasting colour for legibility. This method returns
    /// the colour-space inverse of the caret colour.
    ///
    /// Addresses: Requirement 2, criterion 2.7
    pub fn inverse_text_colour(caret_colour: ColourRGBA) -> ColourRGBA {
        caret_colour.inverse()
    }
}

impl Default for CaretColours {
    fn default() -> Self {
        Self {
            primary: ColourRGBA::rgb(0, 0, 0),          // black
            additional: ColourRGBA::rgb(127, 127, 127), // grey #7F7F7F
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_primary_colour_is_black() {
        // Validates: Requirement 2.2
        let colours = CaretColours::default();
        assert_eq!(colours.primary(), ColourRGBA::rgb(0, 0, 0));
    }

    #[test]
    fn default_additional_colour_is_grey() {
        // Validates: Requirement 2.4
        let colours = CaretColours::default();
        assert_eq!(colours.additional(), ColourRGBA::rgb(127, 127, 127));
    }

    #[test]
    fn colour_for_primary_returns_primary_colour() {
        // Validates: Requirement 2.1
        let colours = CaretColours::new(ColourRGBA::rgb(255, 0, 0), ColourRGBA::rgb(0, 255, 0));
        assert_eq!(colours.colour_for(true), ColourRGBA::rgb(255, 0, 0));
    }

    #[test]
    fn colour_for_additional_returns_additional_colour() {
        // Validates: Requirement 2.3
        let colours = CaretColours::new(ColourRGBA::rgb(255, 0, 0), ColourRGBA::rgb(0, 255, 0));
        assert_eq!(colours.colour_for(false), ColourRGBA::rgb(0, 255, 0));
    }

    #[test]
    fn inverse_text_colour_of_black_is_white() {
        // Validates: Requirement 2.7
        let inv = CaretColours::inverse_text_colour(ColourRGBA::rgb(0, 0, 0));
        assert_eq!(inv, ColourRGBA::rgb(255, 255, 255));
    }

    #[test]
    fn inverse_text_colour_of_white_is_black() {
        // Validates: Requirement 2.7
        let inv = CaretColours::inverse_text_colour(ColourRGBA::rgb(255, 255, 255));
        assert_eq!(inv, ColourRGBA::rgb(0, 0, 0));
    }

    #[test]
    fn inverse_text_colour_preserves_alpha() {
        // Validates: Requirement 2.7
        let inv = CaretColours::inverse_text_colour(ColourRGBA::rgba(100, 150, 200, 0x80));
        assert_eq!(inv.a, 0x80);
    }
}
