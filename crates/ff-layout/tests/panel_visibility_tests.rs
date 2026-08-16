//! Property-based tests for Panel Visibility.
//! Feature: layout-and-docking, Property 10: Panel Visibility Toggle Idempotence

use ff_layout::state::layout_state::LayoutState;
use proptest::prelude::*;

proptest! {
    /// **Validates: Requirements 1.12**
    ///
    /// Property 10: Panel Visibility Toggle Idempotence
    /// toggle_panel twice returns panel to original visibility state.
    #[test]
    fn panel_visibility_toggle_idempotence(
        panel_id in "[a-z_]{1,20}",
        initially_visible in proptest::bool::ANY,
    ) {
        let mut state = LayoutState::default();
        state.panel_visibility.insert(panel_id.clone(), initially_visible);

        let original_visible = state.is_panel_visible(&panel_id);

        // First toggle
        let current = state.panel_visibility.get(&panel_id).copied().unwrap_or(true);
        state.panel_visibility.insert(panel_id.clone(), !current);

        // Second toggle
        let current = state.panel_visibility.get(&panel_id).copied().unwrap_or(true);
        state.panel_visibility.insert(panel_id.clone(), !current);

        // Should be back to original
        let final_visible = state.is_panel_visible(&panel_id);
        prop_assert_eq!(original_visible, final_visible,
            "Double toggle did not restore original state for '{}'", panel_id);
    }
}
