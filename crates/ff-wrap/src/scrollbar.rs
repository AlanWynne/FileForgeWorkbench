//! Horizontal scrollbar interaction logic.
//!
//! Determines scrollbar visibility based on wrap state and boundary mode.

use crate::boundary::WrapBoundary;
use crate::mode::WrapMode;
use crate::state::WrapState;

/// Scrollbar visibility state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    /// Scrollbar should be visible and functional.
    Visible,
    /// Scrollbar should be hidden.
    Hidden,
}

/// Determine if the horizontal scrollbar should be visible given the current wrap state.
///
/// Rules:
/// - `None` mode → always Visible (horizontal scrolling needed for long lines)
/// - `Word`/`Character` + `Viewport` boundary → Hidden (all content fits)
/// - `Word`/`Character` + `Column(n)` + viewport < n → Visible (content may overflow viewport)
/// - `Word`/`Character` + `Column(n)` + viewport >= n → Hidden (wrapped content fits)
///
/// Addresses: Requirement 7 AC 1–5
pub fn scrollbar_visibility(state: &WrapState, viewport_width: u16) -> ScrollbarVisibility {
    match state.mode() {
        WrapMode::None => ScrollbarVisibility::Visible,
        WrapMode::Word | WrapMode::Character => match state.boundary() {
            WrapBoundary::Viewport => ScrollbarVisibility::Hidden,
            WrapBoundary::Column(col) => {
                if viewport_width < col.value() {
                    ScrollbarVisibility::Visible
                } else {
                    ScrollbarVisibility::Hidden
                }
            }
        },
    }
}

/// Determine if horizontal_offset should be reset when wrap state changes.
///
/// Returns `true` when transitioning from None → active mode with Viewport boundary.
///
/// Addresses: Requirement 7 AC 1
pub fn should_reset_horizontal_offset(state: &WrapState) -> bool {
    state.is_active() && state.boundary() == WrapBoundary::Viewport
}

/// Check if the scrollbar should be shown, as a simple bool interface.
///
/// Equivalent to `scrollbar_visibility(state, viewport_width) == Visible`.
///
/// Addresses: Requirement 7 AC 1–5
pub fn should_show_horizontal_scrollbar(state: &WrapState, viewport_width: u16) -> bool {
    scrollbar_visibility(state, viewport_width) == ScrollbarVisibility::Visible
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::{WrapBoundary, WrapColumn};
    use crate::config::WrapConfig;

    fn state_with_mode_and_boundary(mode: WrapMode, boundary: WrapBoundary) -> WrapState {
        let config = WrapConfig {
            default_mode: mode,
            wrap_column: boundary,
            ..WrapConfig::default()
        };
        WrapState::from_config(&config)
    }

    #[test]
    fn none_mode_scrollbar_visible() {
        // Validates: Requirement 7.3
        let state = state_with_mode_and_boundary(WrapMode::None, WrapBoundary::Viewport);
        assert_eq!(
            scrollbar_visibility(&state, 80),
            ScrollbarVisibility::Visible
        );
    }

    #[test]
    fn word_mode_viewport_boundary_scrollbar_hidden() {
        // Validates: Requirement 7.4
        let state = state_with_mode_and_boundary(WrapMode::Word, WrapBoundary::Viewport);
        assert_eq!(
            scrollbar_visibility(&state, 80),
            ScrollbarVisibility::Hidden
        );
    }

    #[test]
    fn character_mode_viewport_boundary_scrollbar_hidden() {
        // Validates: Requirement 7.4
        let state = state_with_mode_and_boundary(WrapMode::Character, WrapBoundary::Viewport);
        assert_eq!(
            scrollbar_visibility(&state, 80),
            ScrollbarVisibility::Hidden
        );
    }

    #[test]
    fn word_mode_column_boundary_narrow_viewport_scrollbar_visible() {
        // Validates: Requirement 7.5
        let col = WrapColumn::new(80).unwrap();
        let state = state_with_mode_and_boundary(WrapMode::Word, WrapBoundary::Column(col));
        assert_eq!(
            scrollbar_visibility(&state, 60),
            ScrollbarVisibility::Visible
        );
    }

    #[test]
    fn word_mode_column_boundary_wide_viewport_scrollbar_hidden() {
        let col = WrapColumn::new(80).unwrap();
        let state = state_with_mode_and_boundary(WrapMode::Word, WrapBoundary::Column(col));
        assert_eq!(
            scrollbar_visibility(&state, 120),
            ScrollbarVisibility::Hidden
        );
    }

    #[test]
    fn should_reset_offset_when_active_viewport() {
        // Validates: Requirement 7.1
        let state = state_with_mode_and_boundary(WrapMode::Word, WrapBoundary::Viewport);
        assert!(should_reset_horizontal_offset(&state));
    }

    #[test]
    fn should_not_reset_offset_when_none() {
        let state = state_with_mode_and_boundary(WrapMode::None, WrapBoundary::Viewport);
        assert!(!should_reset_horizontal_offset(&state));
    }

    #[test]
    fn should_not_reset_offset_when_column_boundary() {
        let col = WrapColumn::new(80).unwrap();
        let state = state_with_mode_and_boundary(WrapMode::Word, WrapBoundary::Column(col));
        assert!(!should_reset_horizontal_offset(&state));
    }
}
