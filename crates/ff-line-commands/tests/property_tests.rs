//! Property-based tests for ff-line-commands.
//!
//! Each property test validates correctness properties from the design document.
//! Uses proptest with a minimum of 100 iterations per property.

use proptest::prelude::*;

use ff_document_model::{BytePosition, Document};
use ff_edit_operations::EditBounds;
use ff_line_commands::command::{classify, LineCommandCategory, LineCommandKind};
use ff_line_commands::config::LineCommandConfig;
use ff_line_commands::execution::delete::get_line_content;
use ff_line_commands::pending::{PendingCommandStore, PendingReason};
use ff_line_commands::resolution::ResolutionEngine;
use ff_line_commands::{
    BlockPairValidator, ExecutableCommand, LineCommandParser, ParsedLineCommand, SourceOperation,
    SourceTarget, TargetPosition,
};

// ─── Test Utilities ─────────────────────────────────────────────────────────

fn make_document(lines: &[&str]) -> Document {
    let mut doc = Document::new();
    let content = lines.join("\n");
    if !content.is_empty() {
        doc.insert(BytePosition::ZERO, content.as_bytes()).unwrap();
    }
    doc
}

fn make_document_from_strings(lines: &[String]) -> Document {
    let mut doc = Document::new();
    let content = lines.join("\n");
    if !content.is_empty() {
        doc.insert(BytePosition::ZERO, content.as_bytes()).unwrap();
    }
    doc
}

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Strategy for valid line command strings.
fn arb_line_command_string() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("D".to_string()),
        (1u32..999).prop_map(|n| format!("D{}", n)),
        Just("DD".to_string()),
        Just("I".to_string()),
        (1u32..99).prop_map(|n| format!("I{}", n)),
        Just("R".to_string()),
        (1u32..99).prop_map(|n| format!("R{}", n)),
        Just("RR".to_string()),
        Just("C".to_string()),
        Just("CC".to_string()),
        Just("M".to_string()),
        Just("MM".to_string()),
        Just("A".to_string()),
        Just("B".to_string()),
        Just("X".to_string()),
        (1u32..999).prop_map(|n| format!("X{}", n)),
        Just("XX".to_string()),
        Just("T".to_string()),
        Just("TT".to_string()),
        Just("U".to_string()),
        Just("UU".to_string()),
        Just(">".to_string()),
        (1u32..99).prop_map(|n| format!(">{}", n)),
        Just(">>".to_string()),
        Just("<".to_string()),
        (1u32..99).prop_map(|n| format!("<{}", n)),
        Just("<<".to_string()),
        Just(")".to_string()),
        Just("))".to_string()),
        Just("(".to_string()),
        Just("((".to_string()),
    ]
}

/// Strategy for document content (multi-line).
fn arb_document_lines(min_lines: usize, max_lines: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-zA-Z0-9 ]{1,40}", min_lines..=max_lines)
}

// ─── Property 1: Parser Round-Trip Consistency ──────────────────────────────
// **Validates: Requirements 1.1, 2.1, 3.1, 4.1, 5.1, 6.1, 7.1, 8.1, 9.1, 10.1, 11.1, 14.7**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 1: For any valid line command string, parse produces a command
    /// that classifies into exactly one category deterministically.
    #[test]
    fn property_1_parser_round_trip_consistency(input in arb_line_command_string()) {
        // Feature: ff-line-commands, Property 1: Parser Round-Trip Consistency
        let cmd = LineCommandParser::parse(&input, 0).unwrap();
        let category = classify(&cmd.kind);

        // Category is one of the four valid values
        prop_assert!(matches!(
            category,
            LineCommandCategory::Immediate
                | LineCommandCategory::Block
                | LineCommandCategory::Source
                | LineCommandCategory::Target
        ));

        // Deterministic: same input always produces same category
        let cmd2 = LineCommandParser::parse(&input, 0).unwrap();
        let category2 = classify(&cmd2.kind);
        prop_assert_eq!(category, category2);
    }
}

// ─── Property 2: Block Pair Normalization ───────────────────────────────────
// **Validates: Requirements 12.2**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 2: For any two line numbers, normalize produces start <= end
    /// and the span is exactly max(l1,l2) - min(l1,l2) + 1.
    #[test]
    fn property_2_block_pair_normalization(line1 in 0u64..100_000, line2 in 0u64..100_000) {
        // Feature: ff-line-commands, Property 2: Block Pair Normalization
        let (start, end) = BlockPairValidator::normalize(line1, line2);
        prop_assert!(start <= end);
        let expected_span = line1.max(line2) - line1.min(line2) + 1;
        prop_assert_eq!(end - start + 1, expected_span);
    }
}

// ─── Property 3: Delete Preserves Document Integrity ────────────────────────
// **Validates: Requirements 1.1, 1.2, 1.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 3: After delete of n lines at L, doc has T-n lines
    /// and lines outside the range are unchanged.
    #[test]
    fn property_3_delete_preserves_document_integrity(
        lines in arb_document_lines(2, 50),
        delete_params in (0usize..50, 1usize..10)
    ) {
        // Feature: ff-line-commands, Property 3: Delete Preserves Document Integrity
        let (start_idx, count_raw) = delete_params;
        let total = lines.len();
        if start_idx >= total {
            return Ok(());
        }
        let count = count_raw.min(total - start_idx);
        if count == 0 {
            return Ok(());
        }

        let mut doc = make_document_from_strings(&lines);
        let before_count = doc.line_count();
        prop_assert_eq!(before_count, total as u64);

        // Capture content before delete
        let before_content: Vec<String> = (0..total as u64)
            .map(|i| get_line_content(&doc, i))
            .collect();

        let result = ff_line_commands::execution::delete::execute_delete(
            &mut doc,
            start_idx as u64,
            count as u64,
        );
        prop_assert!(result.is_ok());

        // Line count decreased by exactly count
        // Note: Document model always has at least 1 line (even when "empty")
        let expected_count = if before_count as usize == count {
            1 // Empty document still reports 1 line
        } else {
            before_count - count as u64
        };
        prop_assert_eq!(doc.line_count(), expected_count);

        // Lines before the deleted range are unchanged
        for i in 0..start_idx {
            prop_assert_eq!(
                get_line_content(&doc, i as u64),
                before_content[i].clone(),
                "Line {} should be unchanged after delete", i
            );
        }

        // Lines after the deleted range are shifted up
        for i in (start_idx + count)..total {
            let new_idx = i - count;
            prop_assert_eq!(
                get_line_content(&doc, new_idx as u64),
                before_content[i].clone(),
                "Line {} (shifted from {}) should match original", new_idx, i
            );
        }
    }
}

// ─── Property 4: Insert Line Count ─────────────────────────────────────────
// **Validates: Requirements 2.1, 2.2**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 4: After insert of n blank lines after L, doc has T+n lines.
    #[test]
    fn property_4_insert_line_count(
        lines in arb_document_lines(1, 50),
        insert_params in (0usize..50, 1u32..20)
    ) {
        // Feature: ff-line-commands, Property 4: Insert Line Count
        let (after_line_raw, count) = insert_params;
        let total = lines.len();
        let after_line = after_line_raw % total;

        let mut doc = make_document_from_strings(&lines);
        let before_count = doc.line_count();

        let result = ff_line_commands::execution::insert::execute_insert(
            &mut doc,
            after_line as u64,
            count,
        );
        prop_assert!(result.is_ok());

        // Line count increased by exactly count
        prop_assert_eq!(doc.line_count(), before_count + count as u64);

        // Lines up to and including after_line are unchanged
        for i in 0..=after_line {
            prop_assert_eq!(
                get_line_content(&doc, i as u64),
                lines[i].clone(),
                "Line {} should be unchanged after insert", i
            );
        }

        // Inserted lines are blank
        for i in 0..count as u64 {
            let line_num = after_line as u64 + 1 + i;
            prop_assert_eq!(
                get_line_content(&doc, line_num),
                "",
                "Inserted line {} should be blank", line_num
            );
        }
    }
}

// ─── Property 5: Repeat Produces Exact Duplicates ───────────────────────────
// **Validates: Requirements 3.1, 3.2**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 5: After repeat of line L with count n, doc has n additional lines
    /// with identical content to the original.
    #[test]
    fn property_5_repeat_produces_exact_duplicates(
        lines in arb_document_lines(1, 30),
        repeat_params in (0usize..30, 1u32..10)
    ) {
        // Feature: ff-line-commands, Property 5: Repeat Produces Exact Duplicates
        let (line_raw, count) = repeat_params;
        let total = lines.len();
        let line = line_raw % total;

        let mut doc = make_document_from_strings(&lines);
        let before_count = doc.line_count();
        let original_content = get_line_content(&doc, line as u64);

        let result = ff_line_commands::execution::repeat::execute_repeat(
            &mut doc,
            line as u64,
            count,
        );
        prop_assert!(result.is_ok());

        // Line count increased by count
        prop_assert_eq!(doc.line_count(), before_count + count as u64);

        // Each duplicated line has the same content as the original
        for i in 1..=count as u64 {
            prop_assert_eq!(
                get_line_content(&doc, line as u64 + i),
                original_content.clone(),
                "Duplicated line at {} should match original", line as u64 + i
            );
        }
    }
}

// ─── Property 6: Shift Right Adds Exactly N Spaces ──────────────────────────
// **Validates: Requirements 9.1, 9.2**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 6: After shift-right of n columns on line L, the content is
    /// prefixed with exactly n spaces.
    #[test]
    fn property_6_shift_right_adds_exactly_n_spaces(
        lines in arb_document_lines(1, 20),
        shift_params in (0usize..20, 1u32..20)
    ) {
        // Feature: ff-line-commands, Property 6: Shift Right Adds Exactly N Spaces
        let (line_raw, columns) = shift_params;
        let total = lines.len();
        let line = line_raw % total;

        let mut doc = make_document_from_strings(&lines);
        let original = get_line_content(&doc, line as u64);

        let result = ff_line_commands::execution::shift_right::execute_shift_right(
            &mut doc,
            line as u64,
            line as u64,
            columns,
        );
        prop_assert!(result.is_ok());

        let shifted = get_line_content(&doc, line as u64);
        let expected = format!("{}{}", " ".repeat(columns as usize), original);
        prop_assert_eq!(shifted, expected);
    }
}

// ─── Property 7: Shift Left Non-Destructive ─────────────────────────────────
// **Validates: Requirements 10.1, 10.2, 10.8**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 7: Shift-left never removes non-whitespace characters.
    #[test]
    fn property_7_shift_left_non_destructive(
        leading_ws in 0usize..20,
        content in "[a-zA-Z0-9]{1,30}",
        columns in 1u32..30
    ) {
        // Feature: ff-line-commands, Property 7: Shift Left Non-Destructive
        let line_content = format!("{}{}", " ".repeat(leading_ws), content);
        let mut doc = make_document(&[&line_content]);

        let result = ff_line_commands::execution::shift_left::execute_shift_left(
            &mut doc,
            0,
            0,
            columns,
        );
        prop_assert!(result.is_ok());

        let shifted = get_line_content(&doc, 0);
        let actual_shift = (columns as usize).min(leading_ws);
        let expected = &line_content[actual_shift..];
        prop_assert_eq!(shifted, expected);
    }
}

// ─── Property 8: Copy Does Not Modify Source ────────────────────────────────
// **Validates: Requirements 4.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 8: After a copy operation, source lines are unchanged
    /// and the document grows by (source_end - source_start + 1) lines.
    #[test]
    fn property_8_copy_does_not_modify_source(
        lines in arb_document_lines(5, 30),
        params in (0usize..30, 0usize..30, 0usize..30)
    ) {
        // Feature: ff-line-commands, Property 8: Copy Does Not Modify Source
        let (src_start_raw, src_end_offset, target_raw) = params;
        let total = lines.len();
        let src_start = src_start_raw % total;
        let src_end = (src_start + (src_end_offset % 5)).min(total - 1);
        let copy_count = src_end - src_start + 1;

        // Target must be outside source range
        let target = if target_raw % total <= src_end {
            (src_end + 1).min(total - 1)
        } else {
            target_raw % total
        };
        // Ensure target is valid
        if target >= total || (target >= src_start && target <= src_end) {
            return Ok(());
        }

        let mut doc = make_document_from_strings(&lines);
        let before_count = doc.line_count();

        // Capture source content
        let source_content: Vec<String> = (src_start..=src_end)
            .map(|i| get_line_content(&doc, i as u64))
            .collect();

        let st = SourceTarget {
            operation: SourceOperation::Copy,
            source_start: src_start as u64,
            source_end: src_end as u64,
            target_line: target as u64,
            target_position: TargetPosition::After,
        };

        let result = ff_line_commands::execution::copy::execute_copy(&mut doc, &st);
        prop_assert!(result.is_ok());

        // Document grew by exactly copy_count lines
        prop_assert_eq!(doc.line_count(), before_count + copy_count as u64);

        // Source content is unchanged (adjusting for position shift if target < source)
        let shift = if target < src_start { copy_count } else { 0 };
        for (idx, content) in source_content.iter().enumerate() {
            let actual_line = (src_start + idx + shift) as u64;
            if actual_line < doc.line_count() {
                prop_assert_eq!(
                    &get_line_content(&doc, actual_line),
                    content,
                    "Source line {} should be unchanged", actual_line
                );
            }
        }
    }
}

// ─── Property 9: Move Preserves Line Count ──────────────────────────────────
// **Validates: Requirements 5.3, 5.4**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 9: After a move operation, the document line count is unchanged.
    #[test]
    fn property_9_move_preserves_line_count(
        lines in arb_document_lines(5, 30),
        params in (0usize..30, 0usize..5, 0usize..30)
    ) {
        // Feature: ff-line-commands, Property 9: Move Preserves Line Count
        let (src_start_raw, src_end_offset, target_raw) = params;
        let total = lines.len();
        let src_start = src_start_raw % total;
        let src_end = (src_start + src_end_offset).min(total - 1);

        // Target must be outside source range
        let target = target_raw % total;
        if target >= src_start && target <= src_end {
            return Ok(());
        }

        let mut doc = make_document_from_strings(&lines);
        let before_count = doc.line_count();

        let st = SourceTarget {
            operation: SourceOperation::Move,
            source_start: src_start as u64,
            source_end: src_end as u64,
            target_line: target as u64,
            target_position: TargetPosition::After,
        };

        let result = ff_line_commands::execution::move_cmd::execute_move(&mut doc, &st);
        prop_assert!(result.is_ok());

        // Line count is unchanged
        prop_assert_eq!(doc.line_count(), before_count);
    }
}

// ─── Property 10: Bounds-Aware Shift Preserves Outer Content ────────────────
// **Validates: Requirements 11.1, 11.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 10: After bounds-aware shift right, characters outside bounds are unchanged.
    #[test]
    fn property_10_bounds_shift_preserves_outer_content(
        content in "[A-Z]{10,40}",
        bounds_params in (1u64..30, 2u64..40)
    ) {
        // Feature: ff-line-commands, Property 10: Bounds-Aware Shift Preserves Outer Content
        let (left_raw, right_offset) = bounds_params;
        let line_len = content.len() as u64;
        if left_raw >= line_len {
            return Ok(());
        }
        let right = (left_raw + right_offset).min(line_len);
        if right <= left_raw {
            return Ok(());
        }
        let bounds = match EditBounds::new(left_raw, right) {
            Some(b) => b,
            None => return Ok(()),
        };

        let mut doc = make_document(&[&content]);
        let original = get_line_content(&doc, 0);

        let result = ff_line_commands::execution::bounds_shift::execute_bounds_shift_right(
            &mut doc, 0, 0, &bounds,
        );
        prop_assert!(result.is_ok());

        let shifted = get_line_content(&doc, 0);
        let orig_chars: Vec<char> = original.chars().collect();
        let shifted_chars: Vec<char> = shifted.chars().collect();

        // Characters before bounds.left (1-based) are unchanged
        let left_idx = (left_raw as usize).saturating_sub(1);
        for i in 0..left_idx {
            if i < orig_chars.len() && i < shifted_chars.len() {
                prop_assert_eq!(
                    shifted_chars[i], orig_chars[i],
                    "Char at idx {} should be unchanged (before bounds)", i
                );
            }
        }

        // Characters after bounds.right (1-based) are unchanged
        let right_idx = right as usize; // 1-based right, so index right_idx and beyond are outside
        for i in right_idx..orig_chars.len().min(shifted_chars.len()) {
            prop_assert_eq!(
                shifted_chars[i], orig_chars[i],
                "Char at idx {} should be unchanged (after bounds)", i
            );
        }
    }
}

// ─── Property 11: Pending Store Size Monotonicity on Clear ──────────────────
// **Validates: Requirements 14.1, 14.2, 14.5**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 11: clear_all → count == 0; adding n → count == n; remove → count - 1.
    #[test]
    fn property_11_pending_store_size_monotonicity(
        num_adds in 1usize..50,
    ) {
        // Feature: ff-line-commands, Property 11: Pending Store Size Monotonicity on Clear
        let mut store = PendingCommandStore::new();

        // After clear, count is 0
        store.clear_all();
        prop_assert_eq!(store.count(), 0);

        // Adding n commands gives count == n
        for i in 0..num_adds {
            store.add(
                ParsedLineCommand { line: i as u64, kind: LineCommandKind::Copy },
                PendingReason::AwaitingTarget,
            );
        }
        prop_assert_eq!(store.count(), num_adds);

        // Removing one decrements by 1
        store.remove(0);
        prop_assert_eq!(store.count(), num_adds - 1);

        // clear_all resets to 0
        store.clear_all();
        prop_assert_eq!(store.count(), 0);
    }
}

// ─── Property 12: Resolution Engine Idempotence ─────────────────────────────
// **Validates: Requirements 13.4, 14.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 12: If resolution is called with no new inputs and no primary command
    /// on a store containing only source/block markers, pending store doesn't change.
    #[test]
    fn property_12_resolution_idempotence_for_pending_only(
        num_sources in 1usize..5,
    ) {
        // Feature: ff-line-commands, Property 12: Resolution Engine Idempotence for Pending-Only State
        let mut store = PendingCommandStore::new();
        let config = LineCommandConfig::default();

        // Add source markers (they can't execute without a target)
        for i in 0..num_sources {
            store.add(
                ParsedLineCommand { line: i as u64 * 10, kind: LineCommandKind::Copy },
                PendingReason::AwaitingTarget,
            );
        }

        let before_count = store.count();

        let result = ResolutionEngine::resolve(&[], &mut store, None, &config);

        // No commands executed
        prop_assert!(result.executable.is_empty());

        // Store unchanged
        prop_assert_eq!(store.count(), before_count);
    }
}

// ─── Property 13: Compatibility Matrix Symmetry ─────────────────────────────
// **Validates: Requirements 13.1, 13.2, 13.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 13: Incompatible pairs always error, compatible pairs always succeed.
    #[test]
    fn property_13_compatibility_matrix_symmetry(
        case_idx in 0usize..4,
    ) {
        // Feature: ff-line-commands, Property 13: Compatibility Matrix Symmetry
        use ff_line_commands::CommandCompatibilityMatrix;

        match case_idx {
            0 => {
                // COPY primary + Move source = incompatible
                let mut store = PendingCommandStore::new();
                store.add(
                    ParsedLineCommand { line: 0, kind: LineCommandKind::Move },
                    PendingReason::AwaitingTarget,
                );
                let result = CommandCompatibilityMatrix::check_compatibility(Some("COPY"), &store);
                prop_assert!(result.is_err());
            }
            1 => {
                // MOVE primary + Copy source = incompatible
                let mut store = PendingCommandStore::new();
                store.add(
                    ParsedLineCommand { line: 0, kind: LineCommandKind::Copy },
                    PendingReason::AwaitingTarget,
                );
                let result = CommandCompatibilityMatrix::check_compatibility(Some("MOVE"), &store);
                prop_assert!(result.is_err());
            }
            2 => {
                // COPY primary + Copy source = compatible
                let mut store = PendingCommandStore::new();
                store.add(
                    ParsedLineCommand { line: 0, kind: LineCommandKind::Copy },
                    PendingReason::AwaitingTarget,
                );
                let result = CommandCompatibilityMatrix::check_compatibility(Some("COPY"), &store);
                prop_assert!(result.is_ok());
            }
            3 => {
                // COPY path + source markers = incompatible
                let mut store = PendingCommandStore::new();
                store.add(
                    ParsedLineCommand { line: 0, kind: LineCommandKind::Copy },
                    PendingReason::AwaitingTarget,
                );
                let result = CommandCompatibilityMatrix::check_compatibility(
                    Some("COPY /tmp/file.txt"),
                    &store,
                );
                prop_assert!(result.is_err());
            }
            _ => {}
        }
    }
}
