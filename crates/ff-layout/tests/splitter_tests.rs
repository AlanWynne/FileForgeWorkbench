//! Property-based tests for Splitter operations.
//! Feature: layout-and-docking, Property 6 and Property 9

use ff_layout::resize::manager::SplitterManager;
use ff_layout::resize::splitter::SplitterOrientation;
use ff_layout::Size;
use proptest::prelude::*;

proptest! {
    /// **Validates: Requirements 8.3, 8.4, 8.5**
    ///
    /// Property 6: Splitter Proportion Invariant
    /// For any splitter drag, the result proportion respects both adjacent
    /// minimum sizes and stays in [0.0, 1.0].
    #[test]
    fn splitter_proportion_invariant(
        min_first in 24.0f32..200.0,
        min_second in 24.0f32..200.0,
        total_size in 100.0f32..2000.0,
        target_proportion in 0.0f32..1.0,
    ) {
        let mut mgr = SplitterManager::new();
        let id = mgr.add_splitter(0.5, SplitterOrientation::Vertical, min_first, min_second);

        mgr.update_splitter(id, target_proportion, total_size).unwrap();

        let splitter = mgr.get(id).unwrap();
        let proportion = splitter.proportion;

        // Proportion must be in [0.0, 1.0]
        prop_assert!(proportion >= 0.0 && proportion <= 1.0,
            "Proportion {} is outside [0.0, 1.0]", proportion);

        // If total_size can satisfy both minimums, enforce them
        if total_size >= min_first + min_second {
            let first_size = proportion * total_size;
            let second_size = (1.0 - proportion) * total_size;

            prop_assert!(first_size >= min_first - 0.01,
                "First size {} < min_first {}", first_size, min_first);
            prop_assert!(second_size >= min_second - 0.01,
                "Second size {} < min_second {}", second_size, min_second);
        }
    }

    /// **Validates: Requirements 8.5**
    ///
    /// Property 9: Proportional Resize Maintains Ratios
    /// When the primary window is resized, proportions are preserved since
    /// they are stored as relative values.
    #[test]
    fn proportional_resize_maintains_ratios(
        initial_proportion in 0.1f32..0.9,
        min_first in 24.0f32..100.0,
        min_second in 24.0f32..100.0,
        old_size in 500.0f32..2000.0,
        new_width in 400.0f32..3000.0,
        new_height in 300.0f32..2000.0,
    ) {
        let mut mgr = SplitterManager::new();
        let id = mgr.add_splitter(initial_proportion, SplitterOrientation::Vertical, min_first, min_second);

        // Set a specific proportion
        let total_size = old_size;
        mgr.update_splitter(id, initial_proportion, total_size).unwrap();
        let pre_resize_proportion = mgr.get(id).unwrap().proportion;

        // Window resize — proportions should be unchanged
        mgr.on_window_resize(Size::new(new_width, new_height));

        let post_resize_proportion = mgr.get(id).unwrap().proportion;

        // Proportions are relative — they should not change on window resize
        prop_assert!(
            (pre_resize_proportion - post_resize_proportion).abs() < f32::EPSILON,
            "Proportion changed after resize: {} -> {}",
            pre_resize_proportion, post_resize_proportion
        );
    }
}
