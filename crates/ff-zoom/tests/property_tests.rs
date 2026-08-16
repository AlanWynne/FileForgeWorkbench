//! Property-based tests for ff-zoom.
//!
//! These tests verify universal invariants across all valid inputs using
//! the proptest framework. Minimum 256 iterations per property.

use proptest::prelude::*;

use ff_zoom::config::ZoomConfig;
use ff_zoom::indicator::ZoomIndicatorState;
use ff_zoom::persistence::ZoomSessionEntry;
use ff_zoom::state::ZoomState;
use ff_zoom::types::ZoomOffset;

// ─── Helpers: Config Strategy ───────────────────────────────────────────────

/// Generate a valid ZoomConfig with min < max.
fn valid_config_strategy() -> impl Strategy<Value = ZoomConfig> {
    (
        (-20i32..=0i32), // min_offset
        (1i32..=100i32), // max_offset (before adjustment)
        (1u32..=10u32),  // step
    )
        .prop_filter("min must be less than max", |(min, max, _)| min < max)
        .prop_flat_map(|(min, max, step)| {
            let default_range = min..=max;
            default_range.prop_map(move |default| ZoomConfig {
                default_offset: default,
                step,
                min_offset: min,
                max_offset: max,
            })
        })
}

// ─── Task 16: ZoomOffset invariants ─────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 1.5, 4.1**
    ///
    /// For any i32 value and any valid [min, max] range, `ZoomOffset::new`
    /// always produces an offset within [min, max] inclusive.
    // Feature: view-zoom, Property 1: Offset Clamping Invariant
    #[test]
    fn zoom_offset_always_within_range(
        value in prop::num::i32::ANY,
        min in -20i32..=0i32,
        max in 1i32..=100i32,
    ) {
        prop_assume!(min < max);
        let offset = ZoomOffset::new(value, min, max);
        prop_assert!(offset.value() >= min);
        prop_assert!(offset.value() <= max);
    }

    /// **Validates: Requirement 1.2**
    ///
    /// `effective_font_size` is always >= 1 for any base_size >= 1 and any
    /// ZoomOffset value (including extreme negatives).
    // Feature: view-zoom, Property 2: Effective Font Size Minimum
    #[test]
    fn effective_font_size_always_at_least_one(
        base_size in 1u32..=72u32,
        offset_value in -100i32..=100i32,
    ) {
        // Use wide range for offset to test extremes
        let offset = ZoomOffset::new(offset_value, -100, 100);
        let effective = offset.effective_font_size(base_size);
        prop_assert!(effective >= 1);
    }

    /// **Validates: Requirement 1.4**
    ///
    /// `ZoomOffset::zero().is_zero()` is always true and
    /// `ZoomOffset::new(n, min, max).is_zero()` is true iff clamped value is 0.
    // Feature: view-zoom, Property 3: Zero Predicate Correctness
    #[test]
    fn is_zero_iff_value_is_zero(
        value in prop::num::i32::ANY,
        min in -20i32..=0i32,
        max in 1i32..=100i32,
    ) {
        prop_assume!(min < max);
        let offset = ZoomOffset::new(value, min, max);
        prop_assert_eq!(offset.is_zero(), offset.value() == 0);
    }

    /// **Validates: Requirements 1.2, 1.8**
    ///
    /// For any base_size and two offsets a < b, effective_font_size(base, a) <= effective_font_size(base, b).
    /// Effective size is monotonically non-decreasing with offset.
    // Feature: view-zoom, Property 4: Effective Size Monotonicity
    #[test]
    fn effective_size_monotonically_non_decreasing(
        base_size in 1u32..=72u32,
        a in -20i32..=59i32,
        b in -20i32..=60i32,
    ) {
        prop_assume!(a < b);
        let offset_a = ZoomOffset::new(a, -20, 60);
        let offset_b = ZoomOffset::new(b, -20, 60);
        let size_a = offset_a.effective_font_size(base_size);
        let size_b = offset_b.effective_font_size(base_size);
        prop_assert!(size_a <= size_b);
    }
}

// ─── Task 17: Zoom operation invariants ─────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 1.5, 2.1, 2.6**
    ///
    /// After any sequence of zoom_in calls, offset never exceeds max_offset.
    // Feature: view-zoom, Property 5: Zoom In Upper Bound
    #[test]
    fn zoom_in_never_exceeds_max(
        config in valid_config_strategy(),
        num_zooms in 1usize..=100,
    ) {
        let mut state = ZoomState::new(&config);
        for _ in 0..num_zooms {
            state.zoom_in();
        }
        prop_assert!(state.offset().value() <= config.max_offset);
    }

    /// **Validates: Requirements 1.5, 2.2, 2.7**
    ///
    /// After any sequence of zoom_out calls, offset never goes below min_offset.
    // Feature: view-zoom, Property 6: Zoom Out Lower Bound
    #[test]
    fn zoom_out_never_below_min(
        config in valid_config_strategy(),
        num_zooms in 1usize..=100,
    ) {
        let mut state = ZoomState::new(&config);
        for _ in 0..num_zooms {
            state.zoom_out();
        }
        prop_assert!(state.offset().value() >= config.min_offset);
    }

    /// **Validates: Requirements 2.3, 8.5**
    ///
    /// zoom_reset always produces offset 0 regardless of prior state.
    // Feature: view-zoom, Property 7: Reset Always Zero
    #[test]
    fn zoom_reset_always_produces_zero(
        config in valid_config_strategy(),
        initial_offset in -20i32..=100i32,
    ) {
        let mut state = ZoomState::from_persisted(initial_offset, &config);
        state.zoom_reset();
        prop_assert_eq!(state.offset().value(), 0);
    }

    /// **Validates: Requirements 1.5, 8.2**
    ///
    /// set_offset(n) followed by offset() returns clamped(n, min, max).
    // Feature: view-zoom, Property 8: Set Offset Clamping
    #[test]
    fn set_offset_returns_clamped_value(
        config in valid_config_strategy(),
        target in prop::num::i32::ANY,
    ) {
        let mut state = ZoomState::new(&config);
        state.set_offset(target);
        let expected = target.clamp(config.min_offset, config.max_offset);
        prop_assert_eq!(state.offset().value(), expected);
    }

    /// **Validates: Requirements 2.1, 2.2**
    ///
    /// zoom_in followed by zoom_out (with step=1) returns to original offset
    /// when not at either limit.
    // Feature: view-zoom, Property 9: In/Out Symmetry
    #[test]
    fn zoom_in_then_out_returns_to_original_when_not_at_limit(
        min in -20i32..=-1i32,
        max in 2i32..=100i32,
        start in -19i32..=99i32,
    ) {
        // Ensure start is strictly within (min, max) so neither in nor out hits limit
        prop_assume!(start > min && start < max);
        let config = ZoomConfig {
            default_offset: start,
            step: 1,
            min_offset: min,
            max_offset: max,
        };
        let mut state = ZoomState::new(&config);
        let original = state.offset().value();
        state.zoom_in();
        state.zoom_out();
        prop_assert_eq!(state.offset().value(), original);
    }
}

// ─── Task 18: Configuration validation invariants ───────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirement 4.4**
    ///
    /// After validate(), step is always within [1, 10].
    // Feature: view-zoom, Property 10: Step Range After Validation
    #[test]
    fn validated_step_always_in_range(
        step in 0u32..=100u32,
    ) {
        let mut config = ZoomConfig {
            step,
            ..Default::default()
        };
        config.validate();
        prop_assert!(config.step >= 1);
        prop_assert!(config.step <= 10);
    }

    /// **Validates: Requirement 4.2**
    ///
    /// After validate(), min_offset < max_offset always holds.
    // Feature: view-zoom, Property 11: Min Less Than Max After Validation
    #[test]
    fn validated_min_less_than_max(
        min_offset in -50i32..=50i32,
        max_offset in -50i32..=150i32,
    ) {
        let mut config = ZoomConfig {
            min_offset,
            max_offset,
            ..Default::default()
        };
        config.validate();
        prop_assert!(config.min_offset < config.max_offset);
    }

    /// **Validates: Requirement 4.3**
    ///
    /// After validate(), default_offset is always within [min_offset, max_offset].
    // Feature: view-zoom, Property 12: Default Offset Within Range After Validation
    #[test]
    fn validated_default_offset_within_range(
        default_offset in -50i32..=150i32,
        min_offset in -20i32..=0i32,
        max_offset in 1i32..=100i32,
        step in 1u32..=10u32,
    ) {
        prop_assume!(min_offset < max_offset);
        let mut config = ZoomConfig {
            default_offset,
            step,
            min_offset,
            max_offset,
        };
        config.validate();
        prop_assert!(config.default_offset >= config.min_offset);
        prop_assert!(config.default_offset <= config.max_offset);
    }

    /// **Validates: Requirement 4.6**
    ///
    /// Hot-reload with any new config values always results in all active
    /// offsets within the new [min, max] range.
    // Feature: view-zoom, Property 13: Hot-Reload Clamping
    #[test]
    fn hot_reload_clamps_active_offsets(
        initial_offset in -20i32..=100i32,
        new_min in -20i32..=0i32,
        new_max in 1i32..=100i32,
    ) {
        prop_assume!(new_min < new_max);
        let old_config = ZoomConfig::default();
        let mut state = ZoomState::from_persisted(initial_offset, &old_config);

        let new_config = ZoomConfig {
            min_offset: new_min,
            max_offset: new_max,
            ..Default::default()
        };
        state.apply_config_change(&new_config);

        prop_assert!(state.offset().value() >= new_min);
        prop_assert!(state.offset().value() <= new_max);
    }
}

// ─── Task 19: Session persistence invariants ────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 6.1, 6.2**
    ///
    /// Persist then restore round-trip preserves offset exactly when offset
    /// is within current config range.
    // Feature: view-zoom, Property 14: Persistence Round-Trip
    #[test]
    fn persist_restore_round_trip_preserves_in_range_offset(
        config in valid_config_strategy(),
        offset in -20i32..=100i32,
    ) {
        // Only test offsets within config range for exact preservation
        let clamped = offset.clamp(config.min_offset, config.max_offset);
        let state = ZoomState::from_persisted(clamped, &config);
        let entry = ZoomSessionEntry::from_state("file:///test", &state);
        let restored = entry.restore(&config);
        prop_assert_eq!(restored.offset().value(), clamped);
    }

    /// **Validates: Requirement 6.3**
    ///
    /// Restoring a persisted offset outside current config range clamps to
    /// nearest bound (never produces out-of-range state).
    // Feature: view-zoom, Property 15: Restore Clamping
    #[test]
    fn restore_outside_range_clamps_to_nearest_bound(
        config in valid_config_strategy(),
        persisted_offset in prop::num::i32::ANY,
    ) {
        let entry = ZoomSessionEntry {
            resource_uri: "file:///test".to_string(),
            zoom_offset: persisted_offset,
        };
        let state = entry.restore(&config);
        prop_assert!(state.offset().value() >= config.min_offset);
        prop_assert!(state.offset().value() <= config.max_offset);
    }

    /// **Validates: Requirement 6.2**
    ///
    /// Restoring with no persisted entry uses default_offset from config.
    // Feature: view-zoom, Property 16: Default Offset For New State
    #[test]
    fn new_state_uses_default_offset_from_config(
        config in valid_config_strategy(),
    ) {
        let state = ZoomState::new(&config);
        prop_assert_eq!(state.offset().value(), config.default_offset);
    }
}

// ─── Task 20: Indicator model invariants ────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 7.1, 7.2**
    ///
    /// Indicator is Hidden if and only if offset is zero.
    // Feature: view-zoom, Property 17: Indicator Hidden Iff Zero
    #[test]
    fn indicator_hidden_iff_offset_zero(
        value in -20i32..=60i32,
    ) {
        let offset = ZoomOffset::new(value, -20, 60);
        let state = ZoomIndicatorState::from_offset(offset);
        if offset.is_zero() {
            prop_assert_eq!(state, ZoomIndicatorState::Hidden);
        } else {
            let is_visible = matches!(state, ZoomIndicatorState::Visible { .. });
            prop_assert!(is_visible);
        }
    }

    /// **Validates: Requirement 7.5**
    ///
    /// Indicator text always contains the absolute offset value with correct sign prefix.
    // Feature: view-zoom, Property 18: Indicator Text Contains Offset
    #[test]
    fn indicator_text_contains_correct_offset(
        value in -20i32..=60i32,
    ) {
        prop_assume!(value != 0);
        let offset = ZoomOffset::new(value, -20, 60);
        let state = ZoomIndicatorState::from_offset(offset);
        if let ZoomIndicatorState::Visible { text, offset: v } = state {
            prop_assert_eq!(v, offset.value());
            let expected = if offset.value() > 0 {
                format!("+{}", offset.value())
            } else {
                format!("{}", offset.value())
            };
            let contains_expected = text.contains(&expected);
            prop_assert!(contains_expected);
        }
    }

    /// **Validates: Requirement 7.5**
    ///
    /// Indicator text matches regex `^Zoom: [+-]\d+$` for any non-zero offset.
    // Feature: view-zoom, Property 19: Indicator Text Format
    #[test]
    fn indicator_text_matches_expected_format(
        value in -20i32..=60i32,
    ) {
        prop_assume!(value != 0);
        let offset = ZoomOffset::new(value, -20, 60);
        let state = ZoomIndicatorState::from_offset(offset);
        if let ZoomIndicatorState::Visible { text, .. } = state {
            // Format: "Zoom: +N" or "Zoom: -N"
            prop_assert!(text.starts_with("Zoom: "));
            let suffix = &text["Zoom: ".len()..];
            // Should start with + or -
            prop_assert!(suffix.starts_with('+') || suffix.starts_with('-'));
            // Rest should be digits
            let digits = &suffix[1..];
            prop_assert!(digits.chars().all(|c| c.is_ascii_digit()));
            prop_assert!(!digits.is_empty());
        }
    }
}
