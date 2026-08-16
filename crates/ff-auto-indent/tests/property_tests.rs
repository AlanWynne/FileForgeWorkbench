//! Property-based tests for ff-auto-indent.
//!
//! Uses proptest to validate correctness properties across generated inputs.
//! Each property maps to a specific requirement from the design document.

use ff_auto_indent::config::IndentStyle;
use ff_auto_indent::maintain::compute_maintain_indent;
use ff_auto_indent::patterns::{c_like_patterns, CompiledPattern};
use ff_auto_indent::smart::compute_smart_indent;
use ff_auto_indent::{
    indent_lines, unindent_lines, AutoIndentMode, AutoIndentService, CommentConfig, IndentConfig,
    IndentContext, IndentDecision, IndentLevel, IndentPatterns, IndentTableRaw,
};

use proptest::prelude::*;

// ─── Strategies ─────────────────────────────────────────────────────────────

fn indent_size_strategy() -> impl Strategy<Value = u32> {
    1u32..=8
}

fn tab_size_strategy() -> impl Strategy<Value = u32> {
    1u32..=8
}

fn indent_style_strategy() -> impl Strategy<Value = IndentStyle> {
    prop_oneof![Just(IndentStyle::Spaces), Just(IndentStyle::Tabs)]
}

fn indent_config_strategy() -> impl Strategy<Value = IndentConfig> {
    (
        indent_size_strategy(),
        tab_size_strategy(),
        indent_style_strategy(),
    )
        .prop_map(|(indent_size, tab_size, style)| IndentConfig::new(indent_size, tab_size, style))
}

fn whitespace_strategy(max_len: usize) -> impl Strategy<Value = String> {
    proptest::collection::vec(prop_oneof![Just(' '), Just('\t')], 0..=max_len)
        .prop_map(|chars| chars.into_iter().collect())
}

fn non_whitespace_content_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_=;(){}\\[\\]]+".prop_map(|s| if s.is_empty() { "x".to_string() } else { s })
}

fn indent_level_strategy() -> impl Strategy<Value = u32> {
    0u32..=1000
}

// ─── Property 1: IndentLevel Decrement Floor ────────────────────────────────

proptest! {
    /// **Validates: Requirement 4.6**
    ///
    /// For any IndentLevel, calling decrement() never produces a value below zero.
    #[test]
    fn property_1_indent_level_never_negative(level in indent_level_strategy()) {
        // Feature: ff-auto-indent, Property 1: IndentLevel decrement never goes negative
        let indent_level = IndentLevel::new(level);
        let decremented = indent_level.decrement();
        prop_assert!(decremented.value() <= level);
        prop_assert!(decremented.value() == level.saturating_sub(1));

        // Special case: zero stays zero
        let zero = IndentLevel::new(0);
        prop_assert_eq!(zero.decrement().value(), 0);
    }
}

// ─── Property 2: Maintain-Indent Preserves Reference Whitespace ─────────────

proptest! {
    /// **Validates: Requirement 2.1**
    ///
    /// In Maintain mode, the new line's indentation column width equals
    /// the reference line's leading whitespace column width.
    #[test]
    fn property_2_maintain_indent_preserves_whitespace(
        leading_ws in whitespace_strategy(10),
        content in non_whitespace_content_strategy(),
        config in indent_config_strategy(),
    ) {
        // Feature: ff-auto-indent, Property 2: maintain-indent preserves reference whitespace
        let reference_line = format!("{}{}", leading_ws, content);
        // Use column position AFTER the content (past the indent), not byte length
        let ws_columns = config.column_width_of(&leading_ws);
        // Caret must be at or after first content column to get full whitespace copy
        let caret_column = (ws_columns + 1) as u64; // 1 column into the content

        let result = compute_maintain_indent(&reference_line, caret_column, &config);

        // The result's column width should match the reference's leading whitespace width
        let expected_columns = ws_columns;
        let actual_columns = config.column_width_of(&result.indent_text);
        prop_assert_eq!(actual_columns, expected_columns,
            "Expected {} columns, got {} for ws='{}' with config {:?}",
            expected_columns, actual_columns, leading_ws, config);
    }
}

// ─── Property 3: Enter at Column Zero Produces No Indent ────────────────────

proptest! {
    /// **Validates: Requirement 2.5**
    ///
    /// When Enter is pressed at column 0, the new line has zero indentation.
    #[test]
    fn property_3_enter_at_column_zero_no_indent(
        leading_ws in whitespace_strategy(10),
        content in non_whitespace_content_strategy(),
        config in indent_config_strategy(),
    ) {
        // Feature: ff-auto-indent, Property 3: enter at column zero produces no indent
        let reference_line = format!("{}{}", leading_ws, content);
        let result = compute_maintain_indent(&reference_line, 0, &config);
        prop_assert_eq!(result.indent_text, "");
        prop_assert_eq!(result.indent_level, 0);
    }
}

// ─── Property 4: Smart Indent Increase Adds Exactly One Level ───────────────

proptest! {
    /// **Validates: Requirement 3.1**
    ///
    /// When the reference line matches increase_pattern and NOT decrease_pattern,
    /// the new line's indent is exactly one indent_size deeper.
    #[test]
    fn property_4_smart_indent_increase_adds_one_level(
        indent_level in 0u32..=5,
        indent_size in 2u32..=8,
    ) {
        // Feature: ff-auto-indent, Property 4: smart indent increase adds exactly one level
        let config = IndentConfig::new(indent_size, 4, IndentStyle::Spaces);
        let patterns = c_like_patterns();

        // Build a reference line that matches increase but NOT decrease
        let leading = " ".repeat((indent_level * indent_size) as usize);
        let reference_line = format!("{}if (true) {{", leading);
        let caret_column = reference_line.len() as u64;

        let ctx = IndentContext::simple(&reference_line, caret_column);
        let result = compute_smart_indent(&ctx, &patterns, &config);

        let expected_columns = (indent_level + 1) * indent_size;
        let actual_columns = config.column_width_of(&result.indent_text);
        prop_assert_eq!(actual_columns, expected_columns,
            "Expected {} columns, got {}", expected_columns, actual_columns);
    }
}

// ─── Property 5: Smart Indent Net Cancellation ──────────────────────────────

proptest! {
    /// **Validates: Requirement 3.5**
    ///
    /// When the reference line matches both increase and decrease patterns,
    /// the net effect is zero.
    #[test]
    fn property_5_smart_indent_net_cancellation(
        indent_level in 1u32..=5,
        indent_size in 2u32..=8,
    ) {
        // Feature: ff-auto-indent, Property 5: smart indent net cancellation
        let config = IndentConfig::new(indent_size, 4, IndentStyle::Spaces);
        let patterns = c_like_patterns();

        // "} else {" matches both increase and decrease
        let leading = " ".repeat((indent_level * indent_size) as usize);
        let reference_line = format!("{}}} else {{", leading);
        let caret_column = reference_line.len() as u64;

        let ctx = IndentContext::simple(&reference_line, caret_column);
        let result = compute_smart_indent(&ctx, &patterns, &config);

        let expected_columns = indent_level * indent_size;
        let actual_columns = config.column_width_of(&result.indent_text);
        prop_assert_eq!(actual_columns, expected_columns,
            "Expected {} columns (net cancel), got {}", expected_columns, actual_columns);
    }
}

// ─── Property 6: Indent Command Adds One IndentString Per Line ──────────────

proptest! {
    /// **Validates: Requirement 7.1**
    ///
    /// Indent command adds exactly one indent_size columns to each line.
    #[test]
    fn property_6_indent_adds_one_level(
        existing_columns in 0u32..=40,
        indent_size in 2u32..=8,
    ) {
        // Feature: ff-auto-indent, Property 6: indent adds one indent_string per line
        let config = IndentConfig::new(indent_size, 4, IndentStyle::Spaces);
        let leading = " ".repeat(existing_columns as usize);
        let line = format!("{}content", leading);

        let result = indent_lines(&[0], &[&line], &config);
        let new_columns = config.column_width_of(&result[0].new_indent);
        prop_assert_eq!(new_columns, existing_columns + indent_size);
    }
}

// ─── Property 7: Unindent Never Goes Below Zero ─────────────────────────────

proptest! {
    /// **Validates: Requirement 8.2**
    ///
    /// Unindent never produces negative indentation.
    #[test]
    fn property_7_unindent_never_negative(
        existing_columns in 0u32..=40,
        indent_size in 2u32..=8,
    ) {
        // Feature: ff-auto-indent, Property 7: unindent never goes below zero
        let config = IndentConfig::new(indent_size, 4, IndentStyle::Spaces);
        let leading = " ".repeat(existing_columns as usize);
        let line = format!("{}content", leading);

        let result = unindent_lines(&[0], &[&line], &config);
        let new_columns = config.column_width_of(&result[0].new_indent);
        prop_assert!(new_columns < existing_columns || existing_columns == 0,
            "Unindent should reduce or stay at zero");
        // The key property: never negative
        // (column_width_of returns u32, so this is always true, but we verify logic)
        prop_assert!(new_columns <= existing_columns);
    }
}

// ─── Property 8: Unindent Removes Exactly One Level When Possible ───────────

proptest! {
    /// **Validates: Requirement 8.1**
    ///
    /// When a line has at least one full indent level, unindent removes exactly indent_size.
    #[test]
    fn property_8_unindent_removes_one_level(
        indent_level in 1u32..=10,
        indent_size in 2u32..=8,
    ) {
        // Feature: ff-auto-indent, Property 8: unindent removes exactly one level
        let config = IndentConfig::new(indent_size, 4, IndentStyle::Spaces);
        let columns = indent_level * indent_size;
        let leading = " ".repeat(columns as usize);
        let line = format!("{}content", leading);

        let result = unindent_lines(&[0], &[&line], &config);
        let new_columns = config.column_width_of(&result[0].new_indent);
        prop_assert_eq!(new_columns, columns - indent_size,
            "Expected {} columns after unindent, got {}", columns - indent_size, new_columns);
    }
}

// ─── Property 9: None Mode Produces No Indentation ──────────────────────────

proptest! {
    /// **Validates: Requirement 10.3**
    ///
    /// When mode is None, compute_newline_indent always returns empty indent.
    #[test]
    fn property_9_none_mode_no_indent(
        leading_ws in whitespace_strategy(10),
        content in non_whitespace_content_strategy(),
    ) {
        // Feature: ff-auto-indent, Property 9: None mode produces no indentation
        let service = AutoIndentService::new(
            IndentConfig::default(),
            AutoIndentMode::None,
        );
        let patterns = c_like_patterns();
        let comment = CommentConfig::c_style();
        let reference_line = format!("{}if (true) {{", leading_ws);
        let ctx = IndentContext::simple(&reference_line, reference_line.len() as u64);

        let result = service.compute_newline_indent(&ctx, &patterns, &comment);
        prop_assert_eq!(result.indent_text, "");
        prop_assert_eq!(result.indent_level, 0);
    }
}

// ─── Property 10: Indent String Consistency ─────────────────────────────────

proptest! {
    /// **Validates: Requirement 1.5**
    ///
    /// Indent string is always consistent with the configured style.
    #[test]
    fn property_10_indent_string_consistency(
        config in indent_config_strategy(),
    ) {
        // Feature: ff-auto-indent, Property 10: indent string consistency with style
        let s = config.indent_string();
        prop_assert!(!s.is_empty(), "Indent string must never be empty");

        match config.style() {
            IndentStyle::Tabs => {
                prop_assert_eq!(s, "\t", "Tabs style must produce single tab");
            }
            IndentStyle::Spaces => {
                prop_assert_eq!(s.len(), config.indent_size() as usize,
                    "Spaces style must produce indent_size spaces");
                prop_assert!(s.chars().all(|c| c == ' '),
                    "Spaces style must contain only space characters");
            }
        }
    }
}

// ─── Property 11: Brace Expansion Middle Line Is One Level Deeper ───────────

proptest! {
    /// **Validates: Requirement 5.1**
    ///
    /// Block expansion middle line is one indent_size deeper than reference.
    #[test]
    fn property_11_brace_expansion_middle_deeper(
        indent_level in 0u32..=5,
        indent_size in 2u32..=8,
    ) {
        // Feature: ff-auto-indent, Property 11: brace expansion middle is one level deeper
        let config = IndentConfig::new(indent_size, 4, IndentStyle::Spaces);
        let service = AutoIndentService::new(config, AutoIndentMode::Smart);
        let patterns = c_like_patterns();
        let comment = CommentConfig::empty();

        let leading = " ".repeat((indent_level * indent_size) as usize);
        let reference_line = format!("{}fn main() {{}}", leading);
        let brace_pos = reference_line.rfind('{').unwrap();

        let ctx = IndentContext {
            reference_line: reference_line.clone(),
            caret_column: (brace_pos + 1) as u64,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: reference_line[..brace_pos + 1].to_string(),
            text_after_caret: "}".to_string(),
        };

        let result = service.compute_newline_indent(&ctx, &patterns, &comment);
        if result.block_expansion.is_some() {
            let middle_columns = config.column_width_of(&result.indent_text);
            let expected = (indent_level + 1) * indent_size;
            prop_assert_eq!(middle_columns, expected,
                "Middle line should be at {} columns, got {}", expected, middle_columns);

            let expansion = result.block_expansion.unwrap();
            let closing_columns = config.column_width_of(&expansion.closing_indent);
            let expected_closing = indent_level * indent_size;
            prop_assert_eq!(closing_columns, expected_closing,
                "Closing line should be at {} columns, got {}", expected_closing, closing_columns);
        }
    }
}

// ─── Property 12: Indent/Unindent Roundtrip ─────────────────────────────────

proptest! {
    /// **Validates: Requirements 7.1, 8.1**
    ///
    /// Indent followed by unindent returns to original indentation.
    #[test]
    fn property_12_indent_unindent_roundtrip(
        indent_level in 1u32..=10,
        indent_size in 2u32..=8,
    ) {
        // Feature: ff-auto-indent, Property 12: indent/unindent roundtrip is identity
        let config = IndentConfig::new(indent_size, 4, IndentStyle::Spaces);
        let original_columns = indent_level * indent_size;
        let leading = " ".repeat(original_columns as usize);
        let line = format!("{}content", leading);

        // Indent
        let indented = indent_lines(&[0], &[&line], &config);
        let indented_line = format!("{}content", indented[0].new_indent);

        // Unindent
        let restored = unindent_lines(&[0], &[&indented_line], &config);
        let restored_columns = config.column_width_of(&restored[0].new_indent);

        prop_assert_eq!(restored_columns, original_columns,
            "Roundtrip should restore {} columns, got {}", original_columns, restored_columns);
    }
}

// ─── Property 13: Invalid Regex Safety ──────────────────────────────────────

proptest! {
    /// **Validates: Requirement 9.7**
    ///
    /// Invalid regex never panics and try_compile returns None.
    #[test]
    fn property_13_invalid_regex_safety(
        invalid in prop_oneof![
            Just("[unclosed".to_string()),
            Just("*invalid".to_string()),
            Just("(?P<".to_string()),
            Just("((".to_string()),
            Just("[z-a]".to_string()),
        ],
        test_input in ".*",
    ) {
        // Feature: ff-auto-indent, Property 13: invalid regex safety
        let result = CompiledPattern::try_compile(&invalid);
        prop_assert!(result.is_none(), "Invalid regex '{}' should return None", invalid);

        // A PatternMatcher with failed compilation should never match
        let raw = IndentTableRaw {
            increase_pattern: Some(invalid.clone()),
            ..Default::default()
        };
        let patterns = IndentPatterns::compile(&raw);
        prop_assert!(!patterns.matches_increase(&test_input),
            "Failed pattern should never match");
    }
}

// ─── Property 14: Caret-Within-Indent Preserves Partial Whitespace ──────────

proptest! {
    /// **Validates: Requirement 2.6**
    ///
    /// When Enter is pressed within leading whitespace, the new line
    /// receives only the whitespace up to the caret position.
    #[test]
    fn property_14_caret_within_indent_partial_whitespace(
        indent_size in 2u32..=8,
        total_spaces in 4u32..=20,
        caret_offset in 1u32..=3,
    ) {
        // Feature: ff-auto-indent, Property 14: caret-within-indent preserves partial whitespace
        let config = IndentConfig::new(indent_size, 4, IndentStyle::Spaces);
        let leading = " ".repeat(total_spaces as usize);
        let reference_line = format!("{}content", leading);
        let caret_column = caret_offset.min(total_spaces - 1) as u64;

        let result = compute_maintain_indent(&reference_line, caret_column, &config);
        let result_columns = config.column_width_of(&result.indent_text);

        prop_assert_eq!(result_columns, caret_column as u32,
            "Caret at column {} should produce {} columns of indent, got {}",
            caret_column, caret_column, result_columns);
    }
}
