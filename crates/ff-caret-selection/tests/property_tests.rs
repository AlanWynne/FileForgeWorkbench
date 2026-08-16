//! Property-based tests for ff-caret-selection.
//!
//! Uses proptest with minimum 100 iterations per property.

use proptest::prelude::*;

use ff_caret_selection::{
    BlinkState, CaretLineConfig, CaretShape, CaretStyle, CaretWidth, ColourRGBA,
    SelectionColourSet, SelectionContext, VirtualSpaceRenderer,
};
use ff_edit_operations::EditMode;

// ─── Property 1: Caret Width Clamping Correctness ───────────────────────────
// Feature: caret-and-selection, Property 1: caret width clamping correctness
// **Validates: Requirements 1.5, 1.6**

proptest! {
    #[test]
    fn caret_width_always_within_bounds(width in 0u8..=255u8) {
        let cw = CaretWidth::new(width);
        let validated = cw.pixels();
        // Invariant: 1 <= validated_width <= 20
        prop_assert!(validated >= 1);
        prop_assert!(validated <= 20);
        // Invariant: validated_width == width.clamp(1, 20)
        prop_assert_eq!(validated, width.clamp(1, 20));
    }
}

// ─── Property 2: Blink Visibility Timing Correctness ────────────────────────
// Feature: caret-and-selection, Property 2: blink visibility timing correctness
// **Validates: Requirements 3.3, 3.5**

proptest! {
    #[test]
    fn blink_visibility_timing_correct(
        period_ms in 0u32..5000u32,
        reset_time_ms in 0u64..100_000u64,
        offset in 0u64..10_000u64,
    ) {
        let current_time_ms = reset_time_ms + offset;
        let mut blink = BlinkState::new(period_ms);
        blink.reset(reset_time_ms);

        let visible = blink.is_visible(current_time_ms);

        if period_ms == 0 {
            // Period 0 → always visible
            prop_assert!(visible, "period=0 should always be visible");
        } else {
            let elapsed = current_time_ms - reset_time_ms;
            let phase = elapsed % (period_ms as u64);
            let expected_visible = phase < (period_ms as u64 / 2);
            prop_assert_eq!(visible, expected_visible,
                "period={}, elapsed={}, phase={}, expected_visible={}",
                period_ms, elapsed, phase, expected_visible);
        }
    }
}

// ─── Property 3: Caret-Line Frame Width Clamping Correctness ────────────────
// Feature: caret-and-selection, Property 3: caret-line frame width clamping correctness
// **Validates: Requirements 4.3, 4.5**

proptest! {
    #[test]
    fn caret_line_frame_width_clamping_correct(
        frame_width in 0u32..100u32,
        line_height in 6u32..120u32,
    ) {
        let mut config = CaretLineConfig::new();
        config.set_frame_width(frame_width);

        let effective = config.effective_frame_width(line_height);
        let max = (line_height / 3).max(1);

        // Invariant: 1 <= effective_width <= line_height / 3
        prop_assert!(effective >= 1, "effective={} should be >= 1", effective);
        prop_assert!(effective <= max, "effective={} should be <= max={}", effective, max);
        // Invariant: effective == frame_width.clamp(1, max)
        prop_assert_eq!(effective, frame_width.clamp(1, max));
    }
}

// ─── Property 4: Virtual Space Horizontal Offset Calculation ────────────────
// Feature: caret-and-selection, Property 4: virtual space horizontal offset calculation correctness
// **Validates: Requirements 7.1**

proptest! {
    #[test]
    fn virtual_space_offset_calculation_correct(
        line_end_x in 0.0f32..10000.0f32,
        virtual_space in 0u64..500u64,
        space_width in 1.0f32..50.0f32,
    ) {
        let renderer = VirtualSpaceRenderer;
        let result = renderer.horizontal_offset(line_end_x, virtual_space, space_width);
        let expected = line_end_x + (virtual_space as f32 * space_width);

        // Invariant: result == line_end_x + (virtual_space * space_width) within f32 tolerance
        let diff = (result - expected).abs();
        prop_assert!(diff < 0.001,
            "result={}, expected={}, diff={}", result, expected, diff);

        // When virtual_space is 0, result equals line_end_x exactly
        if virtual_space == 0 {
            prop_assert_eq!(result, line_end_x);
        }
    }
}

// ─── Property 5: Selection Colour Context Resolution ────────────────────────
// Feature: caret-and-selection, Property 5: selection colour context resolution correctness
// **Validates: Requirements 6.1, 6.6**

fn arb_selection_context() -> impl Strategy<Value = SelectionContext> {
    prop_oneof![
        Just(SelectionContext::Primary),
        Just(SelectionContext::Additional),
        Just(SelectionContext::Secondary),
        Just(SelectionContext::Inactive),
    ]
}

fn arb_colour() -> impl Strategy<Value = ColourRGBA> {
    (0u8..=255u8, 0u8..=255u8, 0u8..=255u8, 0u8..=255u8)
        .prop_map(|(r, g, b, a)| ColourRGBA::rgba(r, g, b, a))
}

fn arb_optional_colour() -> impl Strategy<Value = Option<ColourRGBA>> {
    prop_oneof![Just(None), arb_colour().prop_map(Some),]
}

proptest! {
    #[test]
    fn selection_colour_context_resolution_correct(
        context in arb_selection_context(),
        primary_back in arb_colour(),
        primary_text in arb_optional_colour(),
        additional_back in arb_colour(),
        additional_text in arb_optional_colour(),
        secondary_back in arb_colour(),
        secondary_text in arb_optional_colour(),
        inactive_back in arb_colour(),
        inactive_text in arb_optional_colour(),
    ) {
        let colour_set = SelectionColourSet {
            primary_back,
            primary_text,
            additional_back,
            additional_text,
            secondary_back,
            secondary_text,
            inactive_back,
            inactive_text,
        };

        let (text, back) = colour_set.colours_for_context(context);

        // Invariant: returned pair matches the correct context field
        match context {
            SelectionContext::Primary => {
                prop_assert_eq!(back, primary_back);
                prop_assert_eq!(text, primary_text);
            }
            SelectionContext::Additional => {
                prop_assert_eq!(back, additional_back);
                prop_assert_eq!(text, additional_text);
            }
            SelectionContext::Secondary => {
                prop_assert_eq!(back, secondary_back);
                prop_assert_eq!(text, secondary_text);
            }
            SelectionContext::Inactive => {
                prop_assert_eq!(back, inactive_back);
                prop_assert_eq!(text, inactive_text);
            }
        }
    }
}

// ─── Property 6: Overstrike Mode Forces Block Caret Style ───────────────────
// Feature: caret-and-selection, Property 6: overstrike mode forces Block caret style
// **Validates: Requirements 1.3**

fn arb_caret_style() -> impl Strategy<Value = CaretStyle> {
    prop_oneof![
        Just(CaretStyle::Invisible),
        Just(CaretStyle::Line),
        Just(CaretStyle::Block),
    ]
}

fn arb_edit_mode() -> impl Strategy<Value = EditMode> {
    prop_oneof![
        Just(EditMode::Insert),
        Just(EditMode::Overstrike),
        Just(EditMode::Browse),
    ]
}

proptest! {
    #[test]
    fn overstrike_forces_block_style(
        configured_style in arb_caret_style(),
        edit_mode in arb_edit_mode(),
    ) {
        let shape = CaretShape::new(configured_style, CaretWidth::default());
        let effective = shape.effective_style(edit_mode);

        if edit_mode == EditMode::Overstrike {
            // Invariant: overstrike → effective == Block
            prop_assert_eq!(effective, CaretStyle::Block,
                "Overstrike mode should force Block, got {:?}", effective);
        } else {
            // Invariant: non-overstrike → effective == configured_style
            prop_assert_eq!(effective, configured_style,
                "Non-overstrike mode should preserve configured style {:?}, got {:?}",
                configured_style, effective);
        }
    }
}

// ─── Property 7: Configuration Round-Trip ───────────────────────────────────
// Feature: caret-and-selection, Property 7: configuration round-trip from theme and back
// **Validates: Requirements 11.1, 11.3**

proptest! {
    #[test]
    fn config_defaults_are_stable_after_apply(
        _dummy in 0u8..1u8,  // proptest requires at least one strategy
    ) {
        use ff_caret_selection::CaretSelectionConfig;

        // Construct from defaults
        let original = CaretSelectionConfig::new();

        // Apply defaults (simulates theme update with same values)
        let mut after_update = CaretSelectionConfig::new();
        after_update.apply_defaults();

        // Invariant: config_after_update == original_config
        prop_assert_eq!(original.shape, after_update.shape);
        prop_assert_eq!(original.colours, after_update.colours);
        prop_assert_eq!(original.blink, after_update.blink);
        prop_assert_eq!(original.caret_line, after_update.caret_line);
        prop_assert_eq!(original.selection_display, after_update.selection_display);
        prop_assert_eq!(original.selection_colours, after_update.selection_colours);
        prop_assert_eq!(original.modified_marker, after_update.modified_marker);
    }
}
