//! Property-based tests for the ff-seqnum crate.
//!
//! Uses proptest to verify correctness properties across many random inputs.

use proptest::prelude::*;

use ff_seqnum::{
    generate_sequence, restore_from_side_table, strip_document, ColumnRange, DetectionResult,
    DocumentAccess, DocumentMutate, LanguageProfile, SeqNumConfig, SeqNumState, SequenceDetector,
    SequenceFormat,
};

// ─── Test Helpers ───────────────────────────────────────────────────────────

struct MockDoc {
    lines: Vec<String>,
}

impl MockDoc {
    fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

impl DocumentAccess for MockDoc {
    fn line_count(&self) -> usize {
        self.lines.len()
    }
    fn line_content(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }
}

impl DocumentMutate for MockDoc {
    fn replace_columns(&mut self, line_index: usize, range: &ColumnRange, content: &str) {
        if let Some(line) = self.lines.get_mut(line_index) {
            let start = range.start_offset();
            let end = range.end_offset();
            if line.len() <= start {
                return;
            }
            let actual_end = end.min(line.len());
            let mut new_line = String::with_capacity(line.len());
            new_line.push_str(&line[..start]);
            new_line.push_str(content);
            if actual_end < line.len() {
                new_line.push_str(&line[actual_end..]);
            }
            *line = new_line;
        }
    }
}

struct MockProfile {
    front: Option<ColumnRange>,
    back: Option<ColumnRange>,
    auto_unnum_val: bool,
}

impl LanguageProfile for MockProfile {
    fn sequence_cols_front(&self) -> Option<ColumnRange> {
        self.front
    }
    fn sequence_cols_back(&self) -> Option<ColumnRange> {
        self.back
    }
    fn auto_unnum(&self) -> bool {
        self.auto_unnum_val
    }
    fn language_id(&self) -> &str {
        "test"
    }
}

// ─── Property Tests ─────────────────────────────────────────────────────────

proptest! {
    /// Property 1: Detection Threshold Consistency.
    /// For any set of lines where exactly threshold% have numeric columns,
    /// detector reports Present; below threshold reports Absent.
    ///
    /// **Validates: Requirements 2.1, 2.2, 2.8**
    #[test]
    fn detection_threshold_consistency(
        threshold in 50u8..=100,
        total_lines in 5usize..=20,
        match_pct in 0u8..=100,
    ) {
        // Feature: sequence-numbers, Property 1: Detection Threshold Consistency
        let config = SeqNumConfig {
            detection_threshold: threshold,
            sample_size: 100,
            ..SeqNumConfig::default()
        };
        let detector = SequenceDetector::new(&config);
        let range = ColumnRange::new(1, 6).unwrap();

        // Create lines: match_pct% have numeric content, rest don't
        let matching_count = (total_lines as f64 * match_pct as f64 / 100.0).floor() as usize;
        let mut lines: Vec<String> = Vec::new();
        for i in 0..matching_count {
            lines.push(format!("{:06} body text here", (i + 1) * 100));
        }
        for _ in matching_count..total_lines {
            lines.push(format!("ABCDEF body text here"));
        }

        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let (result, _) = detector.detect_range(&line_refs, &range);

        let required_matches = (total_lines as f64 * threshold as f64 / 100.0).ceil() as usize;
        if matching_count >= required_matches && matching_count > 0 {
            prop_assert_eq!(result, DetectionResult::Present,
                "Expected Present: matching={}, required={}, total={}, threshold={}",
                matching_count, required_matches, total_lines, threshold);
        } else {
            prop_assert_eq!(result, DetectionResult::Absent,
                "Expected Absent: matching={}, required={}, total={}, threshold={}",
                matching_count, required_matches, total_lines, threshold);
        }
    }

    /// Property 2: Strip Idempotency.
    /// Stripping an already-stripped document produces no further modifications.
    ///
    /// **Validates: Requirements 3.2, 5.8**
    #[test]
    fn strip_idempotency(
        line_count in 1usize..=20,
        seq_val in 1u32..=9999,
    ) {
        // Feature: sequence-numbers, Property 2: Strip Idempotency
        let range = ColumnRange::new(1, 6).unwrap();

        // Create document with sequence numbers
        let lines: Vec<String> = (0..line_count)
            .map(|i| format!("{:06} line body content", seq_val + i as u32))
            .collect();
        let mut doc = MockDoc::new(lines);
        let mut state = SeqNumState::new();

        // Strip once
        let result1 = strip_document(&mut doc, &[range], &mut state);
        prop_assert!(result1.lines_modified > 0);

        // Strip again — should modify nothing
        let mut state2 = SeqNumState::new();
        let result2 = strip_document(&mut doc, &[range], &mut state2);
        prop_assert_eq!(result2.lines_modified, 0,
            "Second strip should modify 0 lines, got {}", result2.lines_modified);
    }

    /// Property 3: Strip-Restore Round-Trip.
    /// Stripping and restoring from side-table produces byte-identical original.
    ///
    /// **Validates: Requirements 3.9, 9.5, 11.5**
    #[test]
    fn strip_restore_round_trip(
        line_count in 1usize..=15,
        seq_start in 100u32..=9000,
    ) {
        // Feature: sequence-numbers, Property 3: Strip-Restore Round-Trip
        let range = ColumnRange::new(1, 6).unwrap();

        let lines: Vec<String> = (0..line_count)
            .map(|i| format!("{:06} body content for line {}", seq_start + i as u32 * 100, i))
            .collect();
        let original_lines = lines.clone();
        let mut doc = MockDoc::new(lines);
        let mut state = SeqNumState::new();

        // Strip
        strip_document(&mut doc, &[range], &mut state);

        // Restore
        restore_from_side_table(&mut doc, &state.side_table, Some(&range), None);

        // Verify byte-identical
        for (i, original) in original_lines.iter().enumerate() {
            prop_assert_eq!(
                doc.line_content(i).unwrap(),
                original.as_str(),
                "Line {} mismatch after round-trip", i
            );
        }
    }

    /// Property 4: Number Generation Column Fit.
    /// All generated values have exactly `width` characters when no overflow.
    ///
    /// **Validates: Requirements 6.6, 7.1, 7.2**
    #[test]
    fn number_generation_column_fit(
        width in 3u32..=10,
        start in 1u32..=100,
        increment in 1u32..=10,
        count in 1usize..=20,
    ) {
        // Feature: sequence-numbers, Property 4: Number Generation Column Fit
        let format = SequenceFormat::Numeric;
        let max_val = format.max_value(width);
        let last_val = start as u64 + ((count - 1) as u64 * increment as u64);

        // Only test when no overflow occurs
        if last_val <= max_val {
            let (values, overflow) = generate_sequence(width, start, increment, count, &format);
            prop_assert!(!overflow);
            for value in &values {
                prop_assert_eq!(value.len(), width as usize,
                    "Generated value '{}' has length {} but expected {}",
                    value, value.len(), width);
            }
        }
    }

    /// Property 5: Number Overflow Detection.
    /// When sequence exceeds 10^width - 1, overflow is flagged.
    ///
    /// **Validates: Requirement 6.11**
    #[test]
    fn number_overflow_detection(
        width in 2u32..=6,
        start in 1u32..=999,
        increment in 1u32..=100,
        count in 2usize..=50,
    ) {
        // Feature: sequence-numbers, Property 5: Number Overflow Detection
        let format = SequenceFormat::Numeric;
        let max_val = format.max_value(width);
        let last_val = start as u64 + ((count - 1) as u64 * increment as u64);

        let (_, overflow) = generate_sequence(width, start, increment, count, &format);

        if last_val > max_val {
            prop_assert!(overflow,
                "Expected overflow: last_val={}, max_val={}, width={}",
                last_val, max_val, width);
        } else {
            prop_assert!(!overflow,
                "Unexpected overflow: last_val={}, max_val={}, width={}",
                last_val, max_val, width);
        }
    }

    /// Property 6: Alpha-Prefix Width Constraint.
    /// prefix_len + digits always equals column width; prefix too long is rejected.
    ///
    /// **Validates: Requirements 7.2, 7.4**
    #[test]
    fn alpha_prefix_width_constraint(
        prefix_len in 1usize..=8,
        width in 2u32..=10,
    ) {
        // Feature: sequence-numbers, Property 6: Alpha-Prefix Width Constraint
        let prefix: String = "A".repeat(prefix_len);
        let format = SequenceFormat::AlphaPrefix { prefix: prefix.clone() };

        let valid = format.validate_for_width(width);
        let digit_width = format.digit_width(width);

        if prefix_len as u32 >= width {
            // Prefix fills or exceeds column width — should be invalid
            prop_assert!(!valid,
                "prefix_len={} >= width={} should be invalid", prefix_len, width);
        } else {
            prop_assert!(valid);
            prop_assert_eq!(prefix_len as u32 + digit_width, width,
                "prefix({}) + digits({}) should equal width({})",
                prefix_len, digit_width, width);

            // Generated values should have correct length
            if let Some(formatted) = format.format_value(1, width) {
                prop_assert_eq!(formatted.len(), width as usize);
                prop_assert!(formatted.starts_with(&prefix));
            }
        }
    }

    /// Property 7: Independent Front/Back Detection.
    /// Detection result for front range is independent of back range content.
    ///
    /// **Validates: Requirement 2.4**
    #[test]
    fn independent_front_back_detection(
        front_has_nums in proptest::bool::ANY,
        back_has_nums in proptest::bool::ANY,
    ) {
        // Feature: sequence-numbers, Property 7: Independent Front/Back Detection
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);

        // Build 10 lines (>5, so normal threshold applies)
        let lines: Vec<String> = (0..10).map(|i| {
            let front = if front_has_nums {
                format!("{:06}", (i + 1) * 100)
            } else {
                "ABCDEF".to_string()
            };
            let body = format!("{:<66}", " CODE.");
            let back = if back_has_nums {
                format!("{:08}", (i + 1) * 100)
            } else {
                "COMMENTS".to_string()
            };
            format!("{}{}{}", front, &body[..66], back)
        }).collect();

        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum_val: true,
        };

        let doc = MockDoc::new(lines.clone());
        let result = detector.detect(&doc, &profile);

        if front_has_nums {
            prop_assert_eq!(result.front, DetectionResult::Present,
                "Front should be Present when front_has_nums=true");
        } else {
            prop_assert_eq!(result.front, DetectionResult::Absent,
                "Front should be Absent when front_has_nums=false");
        }

        if back_has_nums {
            prop_assert_eq!(result.back, DetectionResult::Present,
                "Back should be Present when back_has_nums=true");
        } else {
            prop_assert_eq!(result.back, DetectionResult::Absent,
                "Back should be Absent when back_has_nums=false");
        }
    }

    /// Property 8: Short File Strict Threshold.
    /// Files with <5 non-blank lines require 100% match.
    ///
    /// **Validates: Requirement 2.3**
    #[test]
    fn short_file_strict_threshold(
        total_lines in 1usize..=4,
        non_matching_index in 0usize..4,
    ) {
        // Feature: sequence-numbers, Property 8: Short File Strict Threshold
        let config = SeqNumConfig {
            detection_threshold: 50, // Even with low threshold...
            ..SeqNumConfig::default()
        };
        let detector = SequenceDetector::new(&config);
        let range = ColumnRange::new(1, 6).unwrap();

        // Build lines, all matching except possibly one
        let mut lines: Vec<String> = (0..total_lines)
            .map(|i| format!("{:06} body text", (i + 1) * 100))
            .collect();

        if non_matching_index < total_lines {
            lines[non_matching_index] = "ABCDEF body text".to_string();
        }

        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let (result, _) = detector.detect_range(&line_refs, &range);

        if non_matching_index < total_lines {
            // Any single non-match in <5 lines → Absent (100% required)
            prop_assert_eq!(result, DetectionResult::Absent,
                "Short file with non-match at index {} should be Absent (total={})",
                non_matching_index, total_lines);
        }
    }

    /// Property 10: Config Clamping Invariant.
    /// detection_threshold is always in [50, 100] regardless of input.
    ///
    /// **Validates: Requirement 2.8**
    #[test]
    fn config_clamping_invariant(value in 0u8..=255) {
        // Feature: sequence-numbers, Property 10: Config Clamping Invariant
        let (clamped, _) = SeqNumConfig::clamp_threshold(value);
        prop_assert!(clamped >= 50, "Clamped value {} < 50", clamped);
        prop_assert!(clamped <= 100, "Clamped value {} > 100", clamped);

        if value >= 50 && value <= 100 {
            prop_assert_eq!(clamped, value);
        }
    }

    /// Property 11: Column Range Validity.
    /// Parsed ColumnRange always satisfies start <= end and start > 0.
    ///
    /// **Validates: Requirement 1.4**
    #[test]
    fn column_range_validity(
        start in 1u32..=200,
        end in 1u32..=200,
    ) {
        // Feature: sequence-numbers, Property 11: Column Range Validity
        let result = ColumnRange::new(start, end);

        if start == 0 || end == 0 || start > end {
            prop_assert!(result.is_err(),
                "ColumnRange::new({}, {}) should fail", start, end);
        } else {
            let range = result.unwrap();
            prop_assert!(range.start() > 0);
            prop_assert!(range.start() <= range.end());
            prop_assert_eq!(range.width(), end - start + 1);
        }
    }

    /// Property 12: Side-Table Completeness.
    /// After stripping, side-table has entries for exactly the modified lines.
    ///
    /// **Validates: Requirements 3.9, 5.8**
    #[test]
    fn side_table_completeness(
        line_count in 1usize..=15,
        blank_indices in proptest::collection::vec(0usize..15, 0..5),
    ) {
        // Feature: sequence-numbers, Property 12: Side-Table Completeness
        let range = ColumnRange::new(1, 6).unwrap();

        let mut lines: Vec<String> = (0..line_count)
            .map(|i| format!("{:06} body content line {}", (i + 1) * 100, i))
            .collect();

        // Make some lines have blank sequence columns
        for &idx in &blank_indices {
            if idx < line_count {
                lines[idx] = format!("       body content line {}", idx);
            }
        }

        let mut doc = MockDoc::new(lines);
        let mut state = SeqNumState::new();

        let result = strip_document(&mut doc, &[range], &mut state);

        // Count expected modifications (lines that had non-blank content in range)
        prop_assert_eq!(
            state.side_table.len(),
            result.lines_modified,
            "Side-table entries ({}) should equal lines_modified ({})",
            state.side_table.len(),
            result.lines_modified
        );
    }
}
