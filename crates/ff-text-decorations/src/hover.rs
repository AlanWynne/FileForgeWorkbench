//! Hover state tracking for dynamic indicators.
//!
//! Tracks mouse position over text and determines when indicators
//! with hover states need to be redrawn.

use crate::catalogue::IndicatorCatalogue;
use crate::decoration_list::DecorationList;
use crate::IndicatorNumber;

/// Tracks mouse hover position and dynamic indicator interaction.
///
/// Addresses: Requirement 11 AC 1–7
pub struct HoverState {
    /// Current character position under the mouse cursor, or None if outside text.
    current_position: Option<u64>,
    /// Previous position (for detecting transitions).
    previous_position: Option<u64>,
    /// Whether a click has been notified for the current hover position.
    click_notified: bool,
}

impl HoverState {
    /// Create a new hover state with no position.
    pub fn new() -> Self {
        Self {
            current_position: None,
            previous_position: None,
            click_notified: false,
        }
    }

    /// Update the hover position. Returns true if a redraw is needed
    /// (dynamic indicators changed at the new vs old position).
    ///
    /// Addresses: Requirement 11 AC 1, 2
    pub fn update_position(
        &mut self,
        position: Option<u64>,
        decoration_list: &DecorationList,
        catalogue: &IndicatorCatalogue,
    ) -> bool {
        if position == self.current_position {
            return false;
        }

        self.previous_position = self.current_position;
        self.current_position = position;
        self.click_notified = false;

        // Check if any dynamic indicators are active at either position
        let old_has_dynamic = self
            .previous_position
            .map(|pos| self.has_dynamic_at(pos, decoration_list, catalogue))
            .unwrap_or(false);

        let new_has_dynamic = self
            .current_position
            .map(|pos| self.has_dynamic_at(pos, decoration_list, catalogue))
            .unwrap_or(false);

        old_has_dynamic || new_has_dynamic
    }

    /// Mark a click as dispatched at the current position.
    ///
    /// Addresses: Requirement 11 AC 4
    pub fn notify_click(&mut self) {
        self.click_notified = true;
    }

    /// Get the current hover position.
    pub fn position(&self) -> Option<u64> {
        self.current_position
    }

    /// Get the previous hover position.
    pub fn previous_position(&self) -> Option<u64> {
        self.previous_position
    }

    /// Check if click has been notified for current position.
    pub fn is_click_notified(&self) -> bool {
        self.click_notified
    }

    /// Reset click notification state.
    pub fn reset_click(&mut self) {
        self.click_notified = false;
    }

    /// Check if any dynamic indicator is active at a position.
    fn has_dynamic_at(
        &self,
        position: u64,
        decoration_list: &DecorationList,
        catalogue: &IndicatorCatalogue,
    ) -> bool {
        let mask = decoration_list.all_on_for(position);
        for i in 0..=IndicatorNumber::MAX {
            if mask & (1u64 << i) != 0 {
                let indicator = IndicatorNumber(i);
                if catalogue.is_dynamic(indicator) {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for HoverState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicator::{IndicatorConfig, IndicatorFlags, StyleAppearance};
    use crate::indicator_style::IndicatorStyle;
    use crate::ColourRGBA;

    fn make_dynamic_catalogue() -> IndicatorCatalogue {
        let mut catalogue = IndicatorCatalogue::new();
        let normal = StyleAppearance {
            style: IndicatorStyle::Plain,
            fore: ColourRGBA::new(255, 0, 0),
        };
        let hover = StyleAppearance {
            style: IndicatorStyle::Plain,
            fore: ColourRGBA::new(0, 255, 0), // different from normal
        };
        catalogue.set(
            IndicatorNumber(5),
            IndicatorConfig {
                normal,
                hover,
                under: false,
                fill_alpha: 30,
                outline_alpha: 50,
                stroke_width: 1.0,
                flags: IndicatorFlags::default(),
            },
        );
        catalogue
    }

    #[test]
    fn new_hover_state_has_no_position() {
        let hs = HoverState::new();
        assert_eq!(hs.position(), None);
        assert!(!hs.is_click_notified());
    }

    #[test]
    fn update_position_returns_false_when_no_dynamic_indicators() {
        // Validates: Requirement 11 AC 2
        let mut hs = HoverState::new();
        let dl = DecorationList::new(100);
        let catalogue = IndicatorCatalogue::new();
        let needs_redraw = hs.update_position(Some(50), &dl, &catalogue);
        assert!(!needs_redraw);
        assert_eq!(hs.position(), Some(50));
    }

    #[test]
    fn update_position_returns_true_when_dynamic_indicator_present() {
        // Validates: Requirement 11 AC 1
        let mut hs = HoverState::new();
        let mut dl = DecorationList::new(100);
        dl.fill_range(IndicatorNumber(5), 10, 1, 20);
        let catalogue = make_dynamic_catalogue();
        let needs_redraw = hs.update_position(Some(15), &dl, &catalogue);
        assert!(needs_redraw);
    }

    #[test]
    fn update_same_position_returns_false() {
        let mut hs = HoverState::new();
        let dl = DecorationList::new(100);
        let catalogue = IndicatorCatalogue::new();
        hs.update_position(Some(50), &dl, &catalogue);
        let needs_redraw = hs.update_position(Some(50), &dl, &catalogue);
        assert!(!needs_redraw);
    }

    #[test]
    fn notify_click_sets_flag() {
        // Validates: Requirement 11 AC 4
        let mut hs = HoverState::new();
        assert!(!hs.is_click_notified());
        hs.notify_click();
        assert!(hs.is_click_notified());
    }

    #[test]
    fn reset_click_clears_flag() {
        let mut hs = HoverState::new();
        hs.notify_click();
        hs.reset_click();
        assert!(!hs.is_click_notified());
    }

    #[test]
    fn update_position_resets_click_flag() {
        let mut hs = HoverState::new();
        let dl = DecorationList::new(100);
        let catalogue = IndicatorCatalogue::new();
        hs.update_position(Some(10), &dl, &catalogue);
        hs.notify_click();
        hs.update_position(Some(20), &dl, &catalogue);
        assert!(!hs.is_click_notified());
    }
}
