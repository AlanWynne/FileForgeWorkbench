//! Property-based tests for the ff-wrap crate.
//!
//! Uses proptest to verify universal invariants hold across all valid inputs.

use proptest::prelude::*;

use ff_wrap::{
    compute_breaks, compute_char_breaks, compute_height_from_width, compute_sub_line_count,
    compute_word_breaks, execute_wrap_operation, format_indicator, parse_wrap_args,
    scrollbar_visibility, should_reset_horizontal_offset, RawWrapConfig, ScrollbarVisibility,
    WrapBoundary, WrapColumn, WrapConfig, WrapIndentMode, WrapMode, WrapOperation, WrapSnapshot,
    WrapState, WrapVisualFlags,
};

// --- Strategies ---

fn wrap_mode_strategy() -> impl Strategy<Value = WrapMode> {
    prop_oneof![
        Just(WrapMode::None),
        Just(WrapMode::Word),
        Just(WrapMode::Character),
    ]
}

fn active_wrap_mode_strategy() -> impl Strategy<Value = WrapMode> {
    prop_oneof![Just(WrapMode::Word), Just(WrapMode::Character),]
}

fn wrap_boundary_strategy() -> impl Strategy<Value = WrapBoundary> {
    prop_oneof![
        Just(WrapBoundary::Viewport),
        (1u16..=10_000u16).prop_map(|n| WrapBoundary::Column(WrapColumn::new(n).unwrap())),
    ]
}

fn wrap_config_strategy() -> impl Strategy<Value = WrapConfig> {
    (
        wrap_mode_strategy(),
        wrap_boundary_strategy(),
        prop_oneof![
            Just(WrapIndentMode::Fixed),
            Just(WrapIndentMode::Same),
            Just(WrapIndentMode::Indent),
            Just(WrapIndentMode::DeepIndent),
        ],
        0u8..=40u8,
        prop_oneof![
            Just(WrapVisualFlags::None),
            Just(WrapVisualFlags::End),
            Just(WrapVisualFlags::Start),
            Just(WrapVisualFlags::StartEnd),
            Just(WrapVisualFlags::Margin),
        ],
    )
        .prop_map(
            |(mode, boundary, indent_mode, indent_amount, visual_flags)| WrapConfig {
                default_mode: mode,
                wrap_column: boundary,
                indent_mode,
                indent_amount,
                visual_flags,
            },
        )
}

// === Task 18: WrapMode and WrapState invariants ===

proptest! {
    /// Property: is_active() is true iff mode is Word or Character.
    ///
    /// **Validates: Requirement 1.1**
    #[test]
    fn prop_is_active_iff_word_or_character(mode in wrap_mode_strategy()) {
        let expected = matches!(mode, WrapMode::Word | WrapMode::Character);
        prop_assert_eq!(mode.is_active(), expected);
    }

    /// Property: WrapState from any valid config has a valid WrapMode variant.
    ///
    /// **Validates: Requirements 1.1, 2.1**
    #[test]
    fn prop_state_from_config_has_valid_mode(config in wrap_config_strategy()) {
        let state = WrapState::from_config(&config);
        let mode = state.mode();
        prop_assert!(matches!(mode, WrapMode::None | WrapMode::Word | WrapMode::Character));
    }

    /// Property: Toggling twice returns to original mode.
    /// (None→Word→None; Word→None→Word)
    ///
    /// **Validates: Requirement 3.4**
    #[test]
    fn prop_toggle_twice_returns_to_original(mode in wrap_mode_strategy()) {
        let config = WrapConfig {
            default_mode: mode,
            ..WrapConfig::default()
        };
        let mut state = WrapState::from_config(&config);
        let original_mode = state.mode();

        execute_wrap_operation(&WrapOperation::Toggle, &mut state);
        execute_wrap_operation(&WrapOperation::Toggle, &mut state);

        // For None: None→Word→None ✓
        // For Word: Word→None→Word ✓
        // For Character: Character→None→Word (because toggle enables Word, not Character)
        if original_mode == WrapMode::Character {
            // Character toggles to None, then None toggles to Word (default enabled)
            prop_assert_eq!(state.mode(), WrapMode::Word);
        } else {
            prop_assert_eq!(state.mode(), original_mode);
        }
    }

    /// Property: effective_wrap_width returns viewport_width for Viewport boundary,
    /// column value for Column(n).
    ///
    /// **Validates: Requirements 4.1, 4.2, 4.3**
    #[test]
    fn prop_effective_wrap_width(
        boundary in wrap_boundary_strategy(),
        viewport_width in 1u16..=10_000u16,
    ) {
        let config = WrapConfig {
            wrap_column: boundary,
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        let effective = state.effective_wrap_width(viewport_width);

        match boundary {
            WrapBoundary::Viewport => prop_assert_eq!(effective, viewport_width),
            WrapBoundary::Column(col) => prop_assert_eq!(effective, col.value()),
        }
    }
}

// === Task 19: Line-breaking invariants ===

proptest! {
    /// Property: No sub-line exceeds wrap_width (accounting for indent).
    ///
    /// **Validates: Requirements 1.2, 1.3, 1.5**
    #[test]
    fn prop_no_sub_line_exceeds_wrap_width(
        line in "[a-z ]{1,100}",
        wrap_width in 1u32..=50u32,
        mode in prop_oneof![Just(WrapMode::Word), Just(WrapMode::Character)],
    ) {
        let breaks = compute_breaks(&line, wrap_width, mode, 0);
        let chars: Vec<char> = line.chars().collect();
        let total = chars.len();

        let mut prev = 0;
        for &b in &breaks {
            let segment_len = b - prev;
            prop_assert!(
                segment_len <= wrap_width as usize,
                "segment [{}, {}) has length {} > wrap_width {}",
                prev, b, segment_len, wrap_width
            );
            prev = b;
        }
        // Last segment
        if !breaks.is_empty() {
            let last_segment = total - prev;
            prop_assert!(last_segment <= wrap_width as usize);
        }
    }

    /// Property: In Word mode, breaks don't split words unless word > wrap_width.
    ///
    /// **Validates: Requirements 1.3, 1.4**
    #[test]
    fn prop_word_mode_preserves_words_when_possible(
        line in "[a-z]{1,8}( [a-z]{1,8}){0,10}",
        wrap_width in 5u32..=20u32,
    ) {
        let breaks = compute_word_breaks(&line, wrap_width, 0);
        let chars: Vec<char> = line.chars().collect();

        for &b in &breaks {
            if b < chars.len() {
                // Break should be at a space boundary or forced (word too long)
                let at_space = b > 0 && chars[b - 1].is_whitespace();
                let word_too_long = {
                    // Check if the segment before break has no whitespace
                    let prev_break = breaks.iter().rfind(|&&x| x < b).copied().unwrap_or(0);
                    let segment: String = chars[prev_break..b].iter().collect();
                    !segment.contains(char::is_whitespace) && segment.len() >= wrap_width as usize
                };
                prop_assert!(
                    at_space || word_too_long,
                    "break at {} is not at word boundary and word is not too long",
                    b
                );
            }
        }
    }

    /// Property: In Character mode, breaks occur at exact width intervals.
    ///
    /// **Validates: Requirement 1.5**
    #[test]
    fn prop_char_mode_breaks_at_exact_positions(
        line in ".{1,100}",
        wrap_width in 1u32..=20u32,
    ) {
        let breaks = compute_char_breaks(&line, wrap_width, 0);
        let total_chars = line.chars().count();

        if total_chars <= wrap_width as usize {
            prop_assert!(breaks.is_empty());
        } else {
            // First break at wrap_width
            if !breaks.is_empty() {
                prop_assert_eq!(breaks[0], wrap_width as usize);
            }
            // Subsequent breaks at wrap_width intervals
            for i in 1..breaks.len() {
                prop_assert_eq!(breaks[i] - breaks[i - 1], wrap_width as usize);
            }
        }
    }

    /// Property: Concatenating all sub-line segments reconstructs original content.
    ///
    /// **Validates: Requirements 1.2, 1.3, 1.5**
    #[test]
    fn prop_sub_lines_reconstruct_original(
        line in ".{0,100}",
        wrap_width in 1u32..=30u32,
        mode in prop_oneof![Just(WrapMode::Word), Just(WrapMode::Character)],
    ) {
        let breaks = compute_breaks(&line, wrap_width, mode, 0);
        let chars: Vec<char> = line.chars().collect();
        let total = chars.len();

        let mut segments = Vec::new();
        let mut prev = 0;
        for &b in &breaks {
            segments.push(&chars[prev..b]);
            prev = b;
        }
        segments.push(&chars[prev..total]);

        let reconstructed: String = segments.iter().flat_map(|s| s.iter()).collect();
        prop_assert_eq!(reconstructed, line);
    }

    /// Property: sub_line_count is 1 for short lines, >1 for long lines.
    ///
    /// **Validates: Requirements 1.2, 6.1**
    #[test]
    fn prop_sub_line_count_short_vs_long(
        line_len in 0usize..=200usize,
        wrap_width in 1u32..=50u32,
        mode in prop_oneof![Just(WrapMode::Word), Just(WrapMode::Character)],
    ) {
        let line: String = "a".repeat(line_len);
        let count = compute_sub_line_count(&line, wrap_width, mode, 0);

        if line_len <= wrap_width as usize {
            prop_assert_eq!(count, 1);
        } else {
            prop_assert!(count > 1);
        }
    }
}

// === Task 20: Display-line-mapping height invariants ===

proptest! {
    /// Property: When mode is None, height is always 1.
    ///
    /// **Validates: Requirements 1.2, 6.2**
    #[test]
    fn prop_none_mode_height_always_one(
        line_width in 0usize..=10_000usize,
        viewport in 1u16..=10_000u16,
    ) {
        let height = compute_height_from_width(line_width, viewport, WrapMode::None, 0);
        prop_assert_eq!(height, 1);
    }

    /// Property: When mode is active, height >= 1.
    ///
    /// **Validates: Requirement 6.1**
    #[test]
    fn prop_active_mode_height_at_least_one(
        line_width in 0usize..=1000usize,
        viewport in 1u16..=200u16,
        mode in active_wrap_mode_strategy(),
    ) {
        let height = compute_height_from_width(line_width, viewport, mode, 0);
        prop_assert!(height >= 1);
    }

    /// Property: Total display lines >= document line count when wrap active.
    ///
    /// **Validates: Requirement 6.7**
    #[test]
    fn prop_total_display_lines_gte_doc_lines(
        line_count in 1usize..=50usize,
        line_width in 0usize..=200usize,
        viewport in 1u16..=100u16,
        mode in active_wrap_mode_strategy(),
    ) {
        let total: u32 = (0..line_count)
            .map(|_| compute_height_from_width(line_width, viewport, mode, 0))
            .sum();
        prop_assert!(total >= line_count as u32);
    }
}

// === Task 21: Configuration validation invariants ===

proptest! {
    /// Property: After from_raw, wrap_column is valid.
    ///
    /// **Validates: Requirements 4.5, 4.7**
    #[test]
    fn prop_config_wrap_column_valid_after_validate(value in -100_000i64..=100_000i64) {
        let raw = RawWrapConfig {
            wrap_column: Some(value),
            ..Default::default()
        };
        let (config, _warnings) = WrapConfig::from_raw(raw);
        match config.wrap_column {
            WrapBoundary::Viewport => {} // Always valid
            WrapBoundary::Column(col) => {
                prop_assert!(col.value() >= 1);
                prop_assert!(col.value() <= 10_000);
            }
        }
    }

    /// Property: After from_raw, indent_amount is always 0–40.
    ///
    /// **Validates: Requirements 5.7, 5.8**
    #[test]
    fn prop_config_indent_amount_valid_after_validate(value in -1000i64..=1000i64) {
        let raw = RawWrapConfig {
            indent_amount: Some(value),
            ..Default::default()
        };
        let (config, _warnings) = WrapConfig::from_raw(raw);
        prop_assert!(config.indent_amount <= 40);
    }

    /// Property: After from_raw, default_mode is always a valid variant.
    ///
    /// **Validates: Requirement 12.2**
    #[test]
    fn prop_config_default_mode_valid_after_validate(value in "[a-z]{0,20}") {
        let raw = RawWrapConfig {
            default_mode: Some(value),
            ..Default::default()
        };
        let (config, _warnings) = WrapConfig::from_raw(raw);
        prop_assert!(matches!(
            config.default_mode,
            WrapMode::None | WrapMode::Word | WrapMode::Character
        ));
    }

    /// Property: Hot-reload with any raw config produces valid fields.
    ///
    /// **Validates: Requirement 12.3**
    #[test]
    fn prop_config_from_raw_always_valid(
        mode in proptest::option::of("[a-z]{0,15}"),
        col in proptest::option::of(-50_000i64..=50_000i64),
        indent_mode in proptest::option::of("[a-z_]{0,15}"),
        indent_amount in proptest::option::of(-100i64..=100i64),
        flags in proptest::option::of("[a-z_]{0,15}"),
    ) {
        let raw = RawWrapConfig {
            default_mode: mode,
            wrap_column: col,
            indent_mode,
            indent_amount,
            visual_flags: flags,
        };
        let (config, _warnings) = WrapConfig::from_raw(raw);

        // All fields must be within valid ranges
        prop_assert!(matches!(config.default_mode, WrapMode::None | WrapMode::Word | WrapMode::Character));
        prop_assert!(config.indent_amount <= 40);
        match config.wrap_column {
            WrapBoundary::Viewport => {}
            WrapBoundary::Column(col) => {
                prop_assert!(col.value() >= 1 && col.value() <= 10_000);
            }
        }
        prop_assert!(matches!(
            config.indent_mode,
            WrapIndentMode::Fixed | WrapIndentMode::Same | WrapIndentMode::Indent | WrapIndentMode::DeepIndent
        ));
        prop_assert!(matches!(
            config.visual_flags,
            WrapVisualFlags::None | WrapVisualFlags::End | WrapVisualFlags::Start | WrapVisualFlags::StartEnd | WrapVisualFlags::Margin
        ));
    }
}

// === Task 22: Session persistence invariants ===

proptest! {
    /// Property: Persist-restore roundtrip preserves valid mode and boundary.
    ///
    /// **Validates: Requirements 11.1, 11.2**
    #[test]
    fn prop_persistence_roundtrip(
        config in wrap_config_strategy(),
    ) {
        let state = WrapState::from_config(&config);
        let snapshot = WrapSnapshot::from_state(&state);
        let restored = snapshot.restore(&config);
        prop_assert_eq!(state.mode(), restored.mode());
        prop_assert_eq!(state.boundary(), restored.boundary());
    }

    /// Property: Restoring invalid mode always produces None.
    ///
    /// **Validates: Requirement 11.3**
    #[test]
    fn prop_restore_invalid_mode_is_none(mode_str in "[a-z]{5,20}") {
        // Skip valid mode strings
        if matches!(mode_str.as_str(), "none" | "word" | "character") {
            return Ok(());
        }
        let snapshot = WrapSnapshot {
            mode: mode_str,
            boundary: "viewport".to_string(),
        };
        let config = WrapConfig::default();
        let state = snapshot.restore(&config);
        prop_assert_eq!(state.mode(), WrapMode::None);
    }

    /// Property: Restoring with no entry uses config default.
    ///
    /// **Validates: Requirement 11.2**
    #[test]
    fn prop_restore_uses_config_default(config in wrap_config_strategy()) {
        let state = WrapState::from_config(&config);
        prop_assert_eq!(state.mode(), config.default_mode);
    }
}

// === Task 23: Indicator and scrollbar invariants ===

proptest! {
    /// Property: Indicator is hidden iff mode is None.
    ///
    /// **Validates: Requirements 8.1, 8.2, 8.3**
    #[test]
    fn prop_indicator_hidden_iff_none(mode in wrap_mode_strategy()) {
        let config = WrapConfig {
            default_mode: mode,
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        let indicator = format_indicator(&state);
        match mode {
            WrapMode::None => prop_assert!(indicator.is_none()),
            _ => prop_assert!(indicator.is_some()),
        }
    }

    /// Property: Scrollbar hidden iff wrap active AND boundary is Viewport.
    ///
    /// **Validates: Requirements 7.1, 7.4, 7.5**
    #[test]
    fn prop_scrollbar_hidden_iff_wrap_active_viewport(
        mode in wrap_mode_strategy(),
        boundary in wrap_boundary_strategy(),
        viewport_width in 1u16..=10_000u16,
    ) {
        let config = WrapConfig {
            default_mode: mode,
            wrap_column: boundary,
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        let visibility = scrollbar_visibility(&state, viewport_width);

        let expected = match (mode, boundary) {
            (WrapMode::None, _) => ScrollbarVisibility::Visible,
            (_, WrapBoundary::Viewport) => ScrollbarVisibility::Hidden,
            (_, WrapBoundary::Column(col)) => {
                if viewport_width < col.value() {
                    ScrollbarVisibility::Visible
                } else {
                    ScrollbarVisibility::Hidden
                }
            }
        };
        prop_assert_eq!(visibility, expected);
    }

    /// Property: When wrap activates with Viewport boundary, h_offset should reset.
    ///
    /// **Validates: Requirement 7.1**
    #[test]
    fn prop_h_offset_resets_when_wrap_active_viewport(mode in active_wrap_mode_strategy()) {
        let config = WrapConfig {
            default_mode: mode,
            wrap_column: WrapBoundary::Viewport,
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        prop_assert!(should_reset_horizontal_offset(&state));
    }

    /// Property: Indicator text matches expected format when visible.
    ///
    /// **Validates: Requirements 8.1, 8.2**
    #[test]
    fn prop_indicator_text_format(mode in active_wrap_mode_strategy()) {
        let config = WrapConfig {
            default_mode: mode,
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        let text = format_indicator(&state).unwrap();
        match mode {
            WrapMode::Word => prop_assert_eq!(text, "Wrap: Word"),
            WrapMode::Character => prop_assert_eq!(text, "Wrap: Char"),
            _ => unreachable!(),
        }
    }
}

// === Task 24: WRAP command invariants ===

proptest! {
    /// Property: WRAP command produces valid WrapMode result.
    ///
    /// **Validates: Requirements 3.1, 3.8**
    #[test]
    fn prop_wrap_command_valid_result(
        sub_cmd in prop_oneof![
            Just("ON".to_string()),
            Just("OFF".to_string()),
            Just("TOGGLE".to_string()),
            Just("WORD".to_string()),
            Just("CHAR".to_string()),
        ],
    ) {
        let op = parse_wrap_args(&sub_cmd).unwrap();
        let mut state = WrapState::from_config(&WrapConfig::default());
        let result = execute_wrap_operation(&op, &mut state);
        prop_assert!(matches!(
            result.new_mode,
            WrapMode::None | WrapMode::Word | WrapMode::Character
        ));
    }

    /// Property: WRAP ON always results in an active mode.
    ///
    /// **Validates: Requirements 3.2, 3.9**
    #[test]
    fn prop_wrap_on_always_active(start_mode in wrap_mode_strategy()) {
        let config = WrapConfig {
            default_mode: start_mode,
            ..WrapConfig::default()
        };
        let mut state = WrapState::from_config(&config);
        execute_wrap_operation(&WrapOperation::On, &mut state);
        prop_assert!(state.is_active());
    }

    /// Property: WRAP OFF always results in None.
    ///
    /// **Validates: Requirements 3.3, 3.10**
    #[test]
    fn prop_wrap_off_always_none(start_mode in wrap_mode_strategy()) {
        let config = WrapConfig {
            default_mode: start_mode,
            ..WrapConfig::default()
        };
        let mut state = WrapState::from_config(&config);
        execute_wrap_operation(&WrapOperation::Off, &mut state);
        prop_assert_eq!(state.mode(), WrapMode::None);
    }

    /// Property: WRAP COL n sets correct boundary.
    ///
    /// **Validates: Requirements 4.6, 4.7**
    #[test]
    fn prop_wrap_col_sets_boundary(n in 0u16..=10_000u16) {
        let mut state = WrapState::from_config(&WrapConfig::default());
        execute_wrap_operation(&WrapOperation::SetColumn(n), &mut state);
        if n == 0 {
            prop_assert_eq!(state.boundary(), WrapBoundary::Viewport);
        } else {
            prop_assert_eq!(state.boundary(), WrapBoundary::Column(WrapColumn::new(n).unwrap()));
        }
    }
}
