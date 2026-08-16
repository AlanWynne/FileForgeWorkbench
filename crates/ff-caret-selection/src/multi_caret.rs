//! Multi-caret display coordination.
//!
//! Produces ordered lists of caret and selection render info,
//! assigning primary vs additional colours based on the main range designation.

use crate::blink::BlinkState;
use crate::caret_colour::CaretColours;
use crate::caret_style::{CaretShape, CaretStyle};
use crate::colour::ColourRGBA;
use crate::selection_colours::SelectionContext;
use ff_edit_operations::{EditMode, SelectionContainer, SelectionPosition};

/// Rendering information for a single caret.
///
/// Addresses: Requirements 1, 2, 9
#[derive(Debug, Clone, PartialEq)]
pub struct CaretRenderInfo {
    /// The caret position (line, column, virtual_space).
    pub position: SelectionPosition,
    /// Whether this is the primary caret.
    pub is_primary: bool,
    /// Colour to use for this caret.
    pub colour: ColourRGBA,
    /// The caret style to render.
    pub style: CaretStyle,
    /// The pixel width for Line style.
    pub width: u8,
}

/// Rendering information for a selection range.
///
/// Addresses: Requirements 5, 6, 8, 9
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionRenderInfo {
    /// Start position of the selection.
    pub start: SelectionPosition,
    /// End position of the selection.
    pub end: SelectionPosition,
    /// The colour context for this selection.
    pub colour_context: SelectionContext,
    /// Whether this is a rectangular selection.
    pub is_rectangular: bool,
}

/// Coordinates multi-caret display, assigning appropriate colours
/// and producing ordered render lists.
///
/// Addresses: Requirement 9, criteria 9.1–9.6
#[derive(Debug)]
pub struct MultiCaretDisplay<'a> {
    /// The selection container holding all ranges.
    selection: &'a SelectionContainer,
    /// Caret shape configuration.
    shape: &'a CaretShape,
    /// Caret colour configuration.
    colours: &'a CaretColours,
}

impl<'a> MultiCaretDisplay<'a> {
    /// Creates a new multi-caret display from the given state.
    pub fn new(
        selection: &'a SelectionContainer,
        shape: &'a CaretShape,
        colours: &'a CaretColours,
    ) -> Self {
        Self {
            selection,
            shape,
            colours,
        }
    }

    /// Produces an ordered list of all caret positions with colour assignment.
    ///
    /// The primary caret (from main_range) uses the primary colour.
    /// All other carets use the additional colour.
    ///
    /// Addresses: Requirement 9, criteria 9.1–9.4
    pub fn caret_render_list(&self, edit_mode: EditMode) -> Vec<CaretRenderInfo> {
        let main_index = self.selection.main_index();
        let effective_style = self.shape.effective_style(edit_mode);
        let width = self.shape.effective_width();

        self.selection
            .ranges()
            .iter()
            .enumerate()
            .map(|(i, range)| {
                let is_primary = i == main_index;
                CaretRenderInfo {
                    position: range.caret,
                    is_primary,
                    colour: self.colours.colour_for(is_primary),
                    style: effective_style,
                    width,
                }
            })
            .collect()
    }

    /// Produces an ordered list of selection ranges with colour context.
    ///
    /// The primary selection uses `SelectionContext::Primary`.
    /// Additional selections use `SelectionContext::Additional`.
    ///
    /// Only non-collapsed ranges (anchor ≠ caret) are included.
    ///
    /// Addresses: Requirement 9, criteria 9.5
    pub fn selection_render_list(&self) -> Vec<SelectionRenderInfo> {
        let main_index = self.selection.main_index();

        self.selection
            .ranges()
            .iter()
            .enumerate()
            .filter(|(_, range)| !range.is_collapsed())
            .map(|(i, range)| {
                let colour_context = if i == main_index {
                    SelectionContext::Primary
                } else {
                    SelectionContext::Additional
                };
                SelectionRenderInfo {
                    start: range.start(),
                    end: range.end(),
                    colour_context,
                    is_rectangular: false,
                }
            })
            .collect()
    }

    /// Checks that all carets share the same blink phase.
    ///
    /// Returns true if the blink state indicates visible for the given time.
    /// This ensures uniform blink — all carets visible or hidden simultaneously.
    ///
    /// Addresses: Requirement 9, criterion 9.6
    pub fn are_carets_visible(blink: &BlinkState, current_time_ms: u64) -> bool {
        blink.is_visible(current_time_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_edit_operations::{SelectionContainer, SelectionPosition, SelectionRange};

    #[test]
    fn single_caret_is_primary() {
        // Validates: Requirement 9.2
        let selection = SelectionContainer::new();
        let shape = CaretShape::default();
        let colours = CaretColours::default();
        let display = MultiCaretDisplay::new(&selection, &shape, &colours);

        let carets = display.caret_render_list(EditMode::Insert);
        assert_eq!(carets.len(), 1);
        assert!(carets[0].is_primary);
        assert_eq!(carets[0].colour, colours.primary());
    }

    #[test]
    fn multi_caret_assigns_primary_and_additional_colours() {
        // Validates: Requirement 9.2, 9.3
        let mut selection = SelectionContainer::new();
        selection.add(SelectionRange::collapsed(SelectionPosition::new(5, 0)));
        selection.add(SelectionRange::collapsed(SelectionPosition::new(10, 0)));

        let shape = CaretShape::default();
        let colours = CaretColours::default();
        let display = MultiCaretDisplay::new(&selection, &shape, &colours);

        let carets = display.caret_render_list(EditMode::Insert);
        assert_eq!(carets.len(), 3);

        // Main index is 0 by default
        assert!(carets[0].is_primary);
        assert_eq!(carets[0].colour, colours.primary());
        assert!(!carets[1].is_primary);
        assert_eq!(carets[1].colour, colours.additional());
        assert!(!carets[2].is_primary);
        assert_eq!(carets[2].colour, colours.additional());
    }

    #[test]
    fn all_carets_use_same_style() {
        // Validates: Requirement 9.4
        let mut selection = SelectionContainer::new();
        selection.add(SelectionRange::collapsed(SelectionPosition::new(5, 0)));

        let shape = CaretShape::default();
        let colours = CaretColours::default();
        let display = MultiCaretDisplay::new(&selection, &shape, &colours);

        let carets = display.caret_render_list(EditMode::Insert);
        let styles: Vec<_> = carets.iter().map(|c| c.style).collect();
        assert!(styles.iter().all(|s| *s == CaretStyle::Line));
    }

    #[test]
    fn selection_render_list_excludes_collapsed_ranges() {
        let selection = SelectionContainer::new(); // single collapsed range
        let shape = CaretShape::default();
        let colours = CaretColours::default();
        let display = MultiCaretDisplay::new(&selection, &shape, &colours);

        let selections = display.selection_render_list();
        assert!(selections.is_empty());
    }

    #[test]
    fn selection_render_list_assigns_correct_context() {
        // Validates: Requirement 9.5
        let mut selection = SelectionContainer::with_range(SelectionRange::new(
            SelectionPosition::new(1, 0),
            SelectionPosition::new(1, 10),
        ));
        selection.add(SelectionRange::new(
            SelectionPosition::new(3, 0),
            SelectionPosition::new(3, 5),
        ));

        let shape = CaretShape::default();
        let colours = CaretColours::default();
        let display = MultiCaretDisplay::new(&selection, &shape, &colours);

        let selections = display.selection_render_list();
        assert_eq!(selections.len(), 2);
        assert_eq!(selections[0].colour_context, SelectionContext::Primary);
        assert_eq!(selections[1].colour_context, SelectionContext::Additional);
    }

    #[test]
    fn uniform_blink_all_carets_share_phase() {
        // Validates: Requirement 9.6
        let blink = BlinkState::new(500);
        // At time 0 with reset at 0, should be visible
        assert!(MultiCaretDisplay::are_carets_visible(&blink, 0));
    }
}
