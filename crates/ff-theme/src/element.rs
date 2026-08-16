//! Element-based colour system with optional alpha/transparency.
//!
//! Named UI elements (selection, caret, whitespace, fold markers) can be
//! queried for their colour with optional transparency support. Elements
//! not in the translucent set have their alpha forced to 255.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::colour::ColourRGBA;

/// Named UI elements that support element-colour queries.
///
/// Each element represents a specific visual component that rendering code
/// can query for its colour. Some elements support translucent alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Element {
    /// Selection background.
    SelectionBg,
    /// Selection foreground.
    SelectionFg,
    /// Additional selection background.
    AdditionalSelectionBg,
    /// Additional selection foreground.
    AdditionalSelectionFg,
    /// Caret (cursor) foreground colour.
    CaretFg,
    /// Additional caret foreground.
    AdditionalCaretFg,
    /// Caret line background highlight.
    CaretLineBg,
    /// Whitespace dot foreground.
    WhitespaceFg,
    /// Whitespace background.
    WhitespaceBg,
    /// Fold line colour.
    FoldLineColour,
    /// Fold line highlight colour.
    FoldLineHighlight,
    /// Hidden line indicator.
    HiddenLineIndicator,
}

impl Element {
    /// Returns all element variants.
    pub fn all() -> &'static [Element] {
        &[
            Element::SelectionBg,
            Element::SelectionFg,
            Element::AdditionalSelectionBg,
            Element::AdditionalSelectionFg,
            Element::CaretFg,
            Element::AdditionalCaretFg,
            Element::CaretLineBg,
            Element::WhitespaceFg,
            Element::WhitespaceBg,
            Element::FoldLineColour,
            Element::FoldLineHighlight,
            Element::HiddenLineIndicator,
        ]
    }

    /// Check if this element supports translucent alpha.
    ///
    /// Elements that allow translucency: selection backgrounds, caret line
    /// background, and indicator overlays. All other elements have alpha
    /// forced to 255.
    pub fn allows_translucent(&self) -> bool {
        matches!(
            self,
            Element::SelectionBg
                | Element::AdditionalSelectionBg
                | Element::CaretLineBg
                | Element::FoldLineColour
                | Element::FoldLineHighlight
        )
    }
}

/// Runtime map of element colours with translucency tracking.
///
/// Supports both user-set colours (from theme file or runtime override)
/// and base colours (derived from palette groups). User-set colours take
/// priority over base colours.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementColourMap {
    /// User-set element colours (from theme file or runtime override).
    user_colours: HashMap<Element, ColourRGBA>,
    /// Base element colours (derived from palette groups).
    base_colours: HashMap<Element, ColourRGBA>,
}

impl ElementColourMap {
    /// Create an empty element colour map.
    pub fn new() -> Self {
        Self {
            user_colours: HashMap::new(),
            base_colours: HashMap::new(),
        }
    }

    /// Create an element colour map with base colours pre-populated.
    pub fn with_base_colours(base: HashMap<Element, ColourRGBA>) -> Self {
        Self {
            user_colours: HashMap::new(),
            base_colours: base,
        }
    }

    /// Get the colour for an element, applying alpha enforcement.
    ///
    /// Returns `None` if no colour is set for the element (neither user
    /// nor base). When the element does not allow translucent rendering,
    /// alpha is forced to 255.
    pub fn get(&self, element: Element) -> Option<ColourRGBA> {
        let colour = self
            .user_colours
            .get(&element)
            .or_else(|| self.base_colours.get(&element))
            .copied()?;

        Some(enforce_alpha(element, colour))
    }

    /// Check if an element allows translucent alpha.
    pub fn allows_translucent(&self, element: Element) -> bool {
        element.allows_translucent()
    }

    /// Set a user-override colour for an element.
    pub fn set(&mut self, element: Element, colour: ColourRGBA) {
        self.user_colours.insert(element, colour);
    }

    /// Reset an element colour by removing the user override.
    ///
    /// After reset, the element falls back to its base colour (if any).
    pub fn reset(&mut self, element: Element) {
        self.user_colours.remove(&element);
    }

    /// Set a base colour for an element (derived from palette).
    pub fn set_base(&mut self, element: Element, colour: ColourRGBA) {
        self.base_colours.insert(element, colour);
    }

    /// Check if a user override exists for the given element.
    pub fn has_user_override(&self, element: Element) -> bool {
        self.user_colours.contains_key(&element)
    }

    /// Get all elements that currently allow translucent rendering.
    pub fn translucent_elements(&self) -> HashSet<Element> {
        Element::all()
            .iter()
            .filter(|e| e.allows_translucent())
            .copied()
            .collect()
    }
}

impl Default for ElementColourMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Enforce alpha for elements that don't allow translucent rendering.
fn enforce_alpha(element: Element, colour: ColourRGBA) -> ColourRGBA {
    if element.allows_translucent() {
        colour
    } else {
        colour.as_opaque()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_get_returns_none_for_unset() {
        // Validates: Requirement 10.1
        let map = ElementColourMap::new();
        assert_eq!(map.get(Element::SelectionBg), None);
    }

    #[test]
    fn element_user_override_takes_priority() {
        // Validates: Requirement 10.5
        let mut map = ElementColourMap::new();
        map.set_base(Element::CaretFg, ColourRGBA::rgb(100, 100, 100));
        map.set(Element::CaretFg, ColourRGBA::rgb(255, 255, 0));
        assert_eq!(
            map.get(Element::CaretFg),
            Some(ColourRGBA::rgb(255, 255, 0))
        );
    }

    #[test]
    fn element_reset_reverts_to_base() {
        // Validates: Requirement 10.6
        let mut map = ElementColourMap::new();
        map.set_base(Element::CaretFg, ColourRGBA::rgb(100, 100, 100));
        map.set(Element::CaretFg, ColourRGBA::rgb(255, 255, 0));
        map.reset(Element::CaretFg);
        assert_eq!(
            map.get(Element::CaretFg),
            Some(ColourRGBA::rgb(100, 100, 100))
        );
    }

    #[test]
    fn alpha_forced_for_non_translucent_elements() {
        // Validates: Requirement 10.4
        let mut map = ElementColourMap::new();
        // CaretFg does NOT allow translucent
        map.set(Element::CaretFg, ColourRGBA::rgba(255, 0, 0, 128));
        let result = map.get(Element::CaretFg).unwrap();
        assert_eq!(result.a, 255);
    }

    #[test]
    fn alpha_preserved_for_translucent_elements() {
        // Validates: Requirement 10.4
        let mut map = ElementColourMap::new();
        // SelectionBg DOES allow translucent
        map.set(Element::SelectionBg, ColourRGBA::rgba(0, 0, 255, 128));
        let result = map.get(Element::SelectionBg).unwrap();
        assert_eq!(result.a, 128);
    }

    #[test]
    fn allows_translucent_returns_correct_values() {
        // Validates: Requirement 10.4
        assert!(Element::SelectionBg.allows_translucent());
        assert!(Element::AdditionalSelectionBg.allows_translucent());
        assert!(Element::CaretLineBg.allows_translucent());
        assert!(!Element::CaretFg.allows_translucent());
        assert!(!Element::WhitespaceFg.allows_translucent());
    }
}
