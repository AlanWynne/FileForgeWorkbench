//! Selection element colours.
//!
//! Defines `SelectionColourSet` and `SelectionContext` for fine-grained
//! colour control across different selection states.

use crate::colour::ColourRGBA;

/// Distinguishes the context in which a selection is rendered.
///
/// Each context maps to a specific pair of text/background colour elements.
///
/// Addresses: Requirement 6, criteria 6.1, 6.9, 6.10
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectionContext {
    /// The primary (main) selection range.
    Primary,
    /// Additional (non-primary) multi-selection ranges.
    Additional,
    /// Secondary selections (e.g., find-all match highlights).
    Secondary,
    /// Selections in a pane that has lost keyboard focus.
    Inactive,
}

/// Holds all selection colour pairs for different contexts.
///
/// Each context has a background colour (always set) and an optional
/// text colour override. When the text colour is None, syntax highlighting
/// colours are preserved.
///
/// Addresses: Requirement 6, criteria 6.1–6.10
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionColourSet {
    /// Primary selection background.
    pub primary_back: ColourRGBA,
    /// Primary selection text override (None = retain syntax colours).
    pub primary_text: Option<ColourRGBA>,
    /// Additional selection background.
    pub additional_back: ColourRGBA,
    /// Additional selection text override.
    pub additional_text: Option<ColourRGBA>,
    /// Secondary selection background.
    pub secondary_back: ColourRGBA,
    /// Secondary selection text override.
    pub secondary_text: Option<ColourRGBA>,
    /// Inactive selection background.
    pub inactive_back: ColourRGBA,
    /// Inactive selection text override.
    pub inactive_text: Option<ColourRGBA>,
}

impl SelectionColourSet {
    /// Returns the (text_override, background) colour pair for a given context.
    ///
    /// When `text_override` is None, the caller should preserve syntax colours.
    ///
    /// Addresses: Requirement 6, criteria 6.1, 6.6
    pub fn colours_for_context(
        &self,
        context: SelectionContext,
    ) -> (Option<ColourRGBA>, ColourRGBA) {
        match context {
            SelectionContext::Primary => (self.primary_text, self.primary_back),
            SelectionContext::Additional => (self.additional_text, self.additional_back),
            SelectionContext::Secondary => (self.secondary_text, self.secondary_back),
            SelectionContext::Inactive => (self.inactive_text, self.inactive_back),
        }
    }
}

impl Default for SelectionColourSet {
    fn default() -> Self {
        Self {
            primary_back: ColourRGBA::rgb(192, 192, 192), // #C0C0C0
            primary_text: None,
            additional_back: ColourRGBA::rgb(215, 215, 215), // #D7D7D7
            additional_text: None,
            secondary_back: ColourRGBA::rgb(176, 176, 176), // #B0B0B0
            secondary_text: None,
            inactive_back: ColourRGBA::rgba(128, 128, 128, 0x3F), // #808080 alpha 0x3F
            inactive_text: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_primary_back_is_c0c0c0() {
        // Validates: Requirement 6.2
        let colours = SelectionColourSet::default();
        assert_eq!(colours.primary_back, ColourRGBA::rgb(192, 192, 192));
    }

    #[test]
    fn default_additional_back_is_d7d7d7() {
        // Validates: Requirement 6.3
        let colours = SelectionColourSet::default();
        assert_eq!(colours.additional_back, ColourRGBA::rgb(215, 215, 215));
    }

    #[test]
    fn default_secondary_back_is_b0b0b0() {
        // Validates: Requirement 6.4
        let colours = SelectionColourSet::default();
        assert_eq!(colours.secondary_back, ColourRGBA::rgb(176, 176, 176));
    }

    #[test]
    fn default_inactive_back_is_808080_with_alpha() {
        // Validates: Requirement 6.5
        let colours = SelectionColourSet::default();
        assert_eq!(colours.inactive_back, ColourRGBA::rgba(128, 128, 128, 0x3F));
    }

    #[test]
    fn default_text_overrides_are_none() {
        // Validates: Requirement 6.6
        let colours = SelectionColourSet::default();
        assert_eq!(colours.primary_text, None);
        assert_eq!(colours.additional_text, None);
        assert_eq!(colours.secondary_text, None);
        assert_eq!(colours.inactive_text, None);
    }

    #[test]
    fn colours_for_context_primary_returns_correct_pair() {
        // Validates: Requirement 6.1
        let colours = SelectionColourSet::default();
        let (text, back) = colours.colours_for_context(SelectionContext::Primary);
        assert_eq!(text, None);
        assert_eq!(back, ColourRGBA::rgb(192, 192, 192));
    }

    #[test]
    fn colours_for_context_additional_returns_correct_pair() {
        let colours = SelectionColourSet::default();
        let (text, back) = colours.colours_for_context(SelectionContext::Additional);
        assert_eq!(text, None);
        assert_eq!(back, ColourRGBA::rgb(215, 215, 215));
    }

    #[test]
    fn colours_for_context_secondary_returns_correct_pair() {
        let colours = SelectionColourSet::default();
        let (text, back) = colours.colours_for_context(SelectionContext::Secondary);
        assert_eq!(text, None);
        assert_eq!(back, ColourRGBA::rgb(176, 176, 176));
    }

    #[test]
    fn colours_for_context_inactive_returns_correct_pair() {
        let colours = SelectionColourSet::default();
        let (text, back) = colours.colours_for_context(SelectionContext::Inactive);
        assert_eq!(text, None);
        assert_eq!(back, ColourRGBA::rgba(128, 128, 128, 0x3F));
    }

    #[test]
    fn text_override_returned_when_set() {
        let colours = SelectionColourSet {
            primary_text: Some(ColourRGBA::rgb(255, 255, 255)),
            ..SelectionColourSet::default()
        };
        let (text, _back) = colours.colours_for_context(SelectionContext::Primary);
        assert_eq!(text, Some(ColourRGBA::rgb(255, 255, 255)));
    }

    #[test]
    fn alpha_is_preserved_in_colours() {
        // Validates: Requirement 6.7
        let colours = SelectionColourSet {
            primary_back: ColourRGBA::rgba(100, 200, 50, 0x80),
            ..SelectionColourSet::default()
        };
        let (_, back) = colours.colours_for_context(SelectionContext::Primary);
        assert_eq!(back.a, 0x80);
    }
}
