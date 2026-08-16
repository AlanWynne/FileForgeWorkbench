//! Keyboard focus integration.
//!
//! Tracks pane focus and manages blink reset on focus gain and caret movement.

use crate::blink::BlinkState;

/// Tracks pane focus and caret visibility state.
///
/// Addresses: Requirement 12, criteria 12.1–12.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusState {
    /// Whether the containing pane has keyboard focus.
    pane_focused: bool,
}

impl FocusState {
    /// Creates a new focus state (initially unfocused).
    pub fn new() -> Self {
        Self {
            pane_focused: false,
        }
    }

    /// Called when the pane gains keyboard focus.
    ///
    /// Resets the blink cycle to the visible phase so the caret
    /// is immediately visible.
    ///
    /// Addresses: Requirement 12, criterion 12.3
    pub fn on_focus_gained(&mut self, blink: &mut BlinkState, current_time_ms: u64) {
        self.pane_focused = true;
        blink.reset(current_time_ms);
    }

    /// Called when the pane loses keyboard focus.
    pub fn on_focus_lost(&mut self) {
        self.pane_focused = false;
    }

    /// Called when the caret moves to a new position.
    ///
    /// Resets the blink cycle to the visible phase to ensure
    /// the caret is immediately visible after movement.
    ///
    /// Addresses: Requirement 12, criterion 12.1
    pub fn on_caret_moved(&mut self, blink: &mut BlinkState, current_time_ms: u64) {
        blink.reset(current_time_ms);
    }

    /// Returns whether the caret should currently be visible.
    ///
    /// The caret is visible when the pane is focused.
    ///
    /// Addresses: Requirement 12, criterion 12.2
    pub fn is_caret_visible(&self) -> bool {
        self.pane_focused
    }

    /// Returns whether the pane is focused.
    pub fn is_focused(&self) -> bool {
        self.pane_focused
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_unfocused() {
        let focus = FocusState::new();
        assert!(!focus.is_focused());
        assert!(!focus.is_caret_visible());
    }

    #[test]
    fn focus_gain_resets_blink_to_visible() {
        // Validates: Requirement 12.3
        let mut focus = FocusState::new();
        let mut blink = BlinkState::new(500);
        // Simulate time passing into hidden phase
        blink.reset(0);
        assert!(!blink.is_visible(300)); // hidden at 300ms

        // Focus gained at time 300
        focus.on_focus_gained(&mut blink, 300);
        assert!(focus.is_focused());
        assert!(blink.is_visible(300)); // visible again
    }

    #[test]
    fn caret_move_resets_blink_to_visible() {
        // Validates: Requirement 12.1
        let mut focus = FocusState::new();
        focus.pane_focused = true;
        let mut blink = BlinkState::new(500);
        blink.reset(0);
        assert!(!blink.is_visible(300)); // hidden at 300ms

        // Caret moved at time 300
        focus.on_caret_moved(&mut blink, 300);
        assert!(blink.is_visible(300)); // visible again
    }

    #[test]
    fn is_caret_visible_returns_false_when_unfocused() {
        // Validates: Requirement 12.2
        let focus = FocusState::new();
        assert!(!focus.is_caret_visible());
    }

    #[test]
    fn is_caret_visible_returns_true_when_focused() {
        let mut focus = FocusState::new();
        let mut blink = BlinkState::new(500);
        focus.on_focus_gained(&mut blink, 0);
        assert!(focus.is_caret_visible());
    }

    #[test]
    fn focus_lost_makes_caret_invisible() {
        let mut focus = FocusState::new();
        let mut blink = BlinkState::new(500);
        focus.on_focus_gained(&mut blink, 0);
        assert!(focus.is_caret_visible());

        focus.on_focus_lost();
        assert!(!focus.is_caret_visible());
    }
}
