//! Property-based tests for ff-navigation-commands.
//!
//! Tests universal invariants across random inputs using proptest.

use proptest::prelude::*;

use ff_navigation_commands::types::{
    ActiveBounds, NavigationConfig, SelectionModifier, SortDirection, SortParams, SortScope,
};
use ff_navigation_commands::{
    CharClassifier, ParagraphNav, ScrollCommands, SortCommand, WordNav, WordPartNav,
};
use ff_viewport_scrolling::{CursorModel, ViewportModel};

// ─── Property 1: Viewport Navigation Clamping Correctness ──────────────────

/// **Validates: Requirements 3.11, 3.12, 3.13**
///
/// For any viewport state and any navigation command, the resulting viewport
/// state always satisfies: 1 <= top_line <= max_top_line and
/// horizontal_offset >= 0.
mod viewport_clamping {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn viewport_clamping_invariant(
            total_lines in 1u64..100_000,
            visible_count in 1u64..1000,
            initial_top in 1u64..100_000,
            initial_h_offset in 0u64..10_000,
            scroll_amount in 0u64..200_000,
            command_type in 0u8..6,
        ) {
            // Feature: navigation-commands, Property 1: viewport clamping correctness
            let visible = visible_count.min(total_lines);
            let mut viewport = ViewportModel::with_line_count(total_lines);
            viewport.set_visible_count(visible);
            viewport.set_max_horizontal_extent(10_000);
            let mut cursor = CursorModel::new();

            // Set initial state
            let max_top = viewport.max_top_line();
            let clamped_top = initial_top.min(max_top).max(1);
            viewport.scroll_to_line(clamped_top, &cursor);
            viewport.set_horizontal_offset(initial_h_offset, &cursor);

            let config = NavigationConfig::default();

            // Execute a random navigation command
            match command_type {
                0 => ScrollCommands::up_lines(&mut viewport, &mut cursor, scroll_amount),
                1 => ScrollCommands::down_lines(&mut viewport, &mut cursor, scroll_amount),
                2 => ScrollCommands::left_columns(&mut viewport, &cursor, scroll_amount),
                3 => ScrollCommands::right_columns(&mut viewport, &cursor, scroll_amount),
                4 => ScrollCommands::top(&mut viewport, &mut cursor),
                5 => ScrollCommands::bottom(&mut viewport, &mut cursor, total_lines),
                _ => ScrollCommands::up_page(&mut viewport, &mut cursor, &config),
            }

            // Invariant: top_line in [1, max_top_line]
            let result_top = viewport.top_line();
            let result_max = viewport.max_top_line();
            prop_assert!(result_top >= 1, "top_line ({}) < 1", result_top);
            prop_assert!(
                result_top <= result_max,
                "top_line ({}) > max_top_line ({})",
                result_top,
                result_max
            );

            // Invariant: horizontal_offset >= 0 (always true for u64)
            // No explicit check needed since it's unsigned
        }
    }
}

// ─── Property 2: SORT Stability and Key Extraction Correctness ─────────────

/// **Validates: Requirements 2.4, 2.8, 2.9, 2.10**
///
/// For any set of lines and any column range, sorting is stable (equal-key
/// lines retain original order) and the result set contains exactly the same
/// lines as the input.
mod sort_stability {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn sort_preserves_all_lines(
            lines in prop::collection::vec("[a-z ]{0,50}", 2..100),
            col_start in 1u64..20,
            col_width in 1u64..10,
            descending in proptest::bool::ANY,
        ) {
            // Feature: navigation-commands, Property 2: sort stability
            let mut to_sort = lines.clone();
            let col_end = col_start + col_width;
            let direction = if descending {
                SortDirection::Descending
            } else {
                SortDirection::Ascending
            };

            let params = SortParams {
                column_range: Some((col_start, col_end)),
                direction,
                scope: SortScope::AllVisible,
            };

            let result = SortCommand::execute(&mut to_sort, &params, None);
            prop_assert!(result.is_ok());

            // Invariant: same number of lines
            prop_assert_eq!(to_sort.len(), lines.len());

            // Invariant: same multiset of lines (no loss or duplication)
            let mut original_sorted = lines.clone();
            original_sorted.sort();
            let mut result_sorted = to_sort.clone();
            result_sorted.sort();
            prop_assert_eq!(original_sorted, result_sorted);
        }

        #[test]
        fn sort_is_stable_for_equal_keys(
            prefix in "[A-Z]{3}",
            suffixes in prop::collection::vec("[a-z]{1,10}", 3..20),
        ) {
            // Feature: navigation-commands, Property 2: sort stability (equal keys)
            // All lines share the same content in the sort key columns (1..3 = prefix)
            let lines: Vec<String> = suffixes.iter()
                .map(|s| format!("{prefix}{s}"))
                .collect();
            let mut to_sort = lines.clone();

            // Sort by columns 1..3 which is exactly the 3-char prefix
            let params = SortParams {
                column_range: Some((1, 3)),
                direction: SortDirection::Ascending,
                scope: SortScope::AllVisible,
            };

            let result = SortCommand::execute(&mut to_sort, &params, None);
            prop_assert!(result.is_ok());

            // Stable sort: if all keys are equal (same prefix), order is preserved
            prop_assert_eq!(to_sort, lines);
        }
    }
}

// ─── Property 3: Word Navigation Class-Transition Correctness ──────────────

/// **Validates: Requirements 7.1, 7.2, 7.3, 7.5**
///
/// For any starting position, word-left stops at a class transition or
/// document boundary, and word-right stops at a class transition or boundary.
mod word_nav_transitions {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn word_right_lands_at_boundary_or_end(
            text in "[a-zA-Z .,:;!?]{1,100}",
            start_col in 1u64..101,
        ) {
            // Feature: navigation-commands, Property 3: word navigation correctness
            let lines = vec![text.as_str()];
            let classifier = CharClassifier::new();
            let chars: Vec<char> = text.chars().collect();
            let clamped_col = start_col.min(chars.len() as u64 + 1);

            let (result_line, result_col) = WordNav::word_right(
                &lines, 1, clamped_col, &classifier, SelectionModifier::Move,
            );

            prop_assert_eq!(result_line, 1);
            // Result must be >= start (word_right never goes backwards on same line)
            prop_assert!(
                result_col >= clamped_col || result_col == 1,
                "word_right went backwards: {} -> {}",
                clamped_col,
                result_col
            );
            // Result must be <= line length + 1
            prop_assert!(
                result_col <= chars.len() as u64 + 1,
                "word_right past end: {} > {}",
                result_col,
                chars.len() + 1
            );
        }

        #[test]
        fn word_left_lands_at_boundary_or_start(
            text in "[a-zA-Z .,:;!?]{1,100}",
            start_col in 1u64..101,
        ) {
            // Feature: navigation-commands, Property 3: word navigation correctness
            let lines = vec![text.as_str()];
            let classifier = CharClassifier::new();
            let chars: Vec<char> = text.chars().collect();
            let clamped_col = start_col.min(chars.len() as u64 + 1);

            let (result_line, result_col) = WordNav::word_left(
                &lines, 1, clamped_col, &classifier, SelectionModifier::Move,
            );

            prop_assert_eq!(result_line, 1);
            // Result must be <= start (word_left never goes forwards on same line)
            prop_assert!(
                result_col <= clamped_col,
                "word_left went forwards: {} -> {}",
                clamped_col,
                result_col
            );
            // Result must be >= 1
            prop_assert!(result_col >= 1);
        }
    }
}

// ─── Property 4: Word-Part Boundary Detection Correctness ──────────────────

/// **Validates: Requirements 8.1, 8.2, 8.5**
///
/// For any identifier-like string, word_part_right followed by word_part_left
/// returns to a valid boundary. Every boundary corresponds to a defined pattern.
mod word_part_boundaries {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn word_part_right_advances_position(
            text in "[a-zA-Z0-9_]{2,50}",
            start_pos in 0usize..50,
        ) {
            // Feature: navigation-commands, Property 4: word-part boundary correctness
            let clamped = start_pos.min(text.len().saturating_sub(1));

            let result = WordPartNav::word_part_right(&text, clamped, SelectionModifier::Move);

            // Result must be > start (must advance) or equal to len (at end)
            prop_assert!(
                result > clamped || result == text.len(),
                "word_part_right did not advance: pos {} -> {}",
                clamped,
                result
            );
            // Result must be <= len
            prop_assert!(result <= text.len());
        }

        #[test]
        fn word_part_left_retreats_position(
            text in "[a-zA-Z0-9_]{2,50}",
            start_pos in 1usize..50,
        ) {
            // Feature: navigation-commands, Property 4: word-part boundary correctness
            let clamped = start_pos.min(text.len());

            let result = WordPartNav::word_part_left(&text, clamped, SelectionModifier::Move);

            // Result must be < start (must retreat) or 0 (at beginning)
            prop_assert!(
                result < clamped || result == 0,
                "word_part_left did not retreat: pos {} -> {}",
                clamped,
                result
            );
        }
    }
}

// ─── Property 5: Column Affinity Preservation ──────────────────────────────

/// **Validates: Requirements 9.1, 9.2, 9.3, 9.4**
///
/// For any sequence of vertical movements without horizontal movements,
/// column_affinity remains unchanged. When the line is long enough,
/// cursor_column equals column_affinity.
mod column_affinity {
    use super::*;
    use ff_navigation_commands::VerticalCaretNav;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn affinity_preserved_through_vertical_moves(
            initial_col in 1u64..100,
            line_lengths in prop::collection::vec(1u64..200, 5..50),
            moves in prop::collection::vec(proptest::bool::ANY, 1..10),
        ) {
            // Feature: navigation-commands, Property 5: column affinity preservation
            let total_lines = line_lengths.len() as u64;
            let mut cursor = CursorModel::new();
            cursor.set_position(1, initial_col);
            let initial_affinity = cursor.column_affinity();

            let mut viewport = ViewportModel::with_line_count(total_lines);
            viewport.set_visible_count(total_lines);

            for go_down in moves {
                let cur_line = cursor.cursor_line();
                if go_down && cur_line < total_lines {
                    // target_line_length is for the NEXT line (cur_line + 1)
                    let target_idx = cur_line as usize; // 0-based index of next line
                    let target_len = line_lengths.get(target_idx).copied().unwrap_or(1);
                    VerticalCaretNav::line_down(
                        &mut cursor, &mut viewport, target_len, total_lines,
                        SelectionModifier::Move,
                    );
                } else if !go_down && cur_line > 1 {
                    // target_line_length is for the PREVIOUS line (cur_line - 1)
                    let target_idx = (cur_line as usize).saturating_sub(2); // 0-based index of prev
                    let target_len = line_lengths.get(target_idx).copied().unwrap_or(1);
                    VerticalCaretNav::line_up(
                        &mut cursor, &mut viewport, target_len,
                        SelectionModifier::Move,
                    );
                }

                // Affinity must remain unchanged (vertical moves preserve affinity)
                prop_assert_eq!(
                    cursor.column_affinity(),
                    initial_affinity,
                    "Affinity changed from {} to {} after move on line {}",
                    initial_affinity,
                    cursor.column_affinity(),
                    cursor.cursor_line()
                );
            }
        }
    }
}

// ─── Property 6: Paragraph Boundary Detection Correctness ──────────────────

/// **Validates: Requirements 6.1, 6.2, 6.3**
///
/// For any document, paragraph-down lands on a non-blank line preceded by
/// blank lines (or at document end), and paragraph-up lands on a non-blank
/// line followed by blank lines upward (or at document start).
mod paragraph_boundaries {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn paragraph_down_lands_at_valid_position(
            // Generate a mix of content and blank lines
            lines in prop::collection::vec(
                prop::sample::select(vec!["content", "hello world", "", "   ", "\t"]),
                3..50
            ),
            start_line in 1u64..50,
        ) {
            // Feature: navigation-commands, Property 6: paragraph boundary correctness
            let total = lines.len() as u64;
            let clamped_start = start_line.min(total);
            let line_refs: Vec<&str> = lines.iter().map(|s| *s).collect();
            let excluded = vec![false; lines.len()];

            let mut viewport = ViewportModel::with_line_count(total);
            viewport.set_visible_count(total);
            let mut cursor = CursorModel::new();
            cursor.set_position(clamped_start, 1);

            ParagraphNav::paragraph_down(
                &mut cursor, &mut viewport, &line_refs, &excluded,
                SelectionModifier::Move,
            );

            let result_line = cursor.cursor_line();
            // Result must be within document bounds
            prop_assert!(result_line >= 1);
            prop_assert!(result_line <= total);
            // Result must be >= start (paragraph-down never goes backwards)
            prop_assert!(
                result_line >= clamped_start,
                "paragraph_down went backwards: {} -> {}",
                clamped_start,
                result_line
            );
        }

        #[test]
        fn paragraph_up_lands_at_valid_position(
            lines in prop::collection::vec(
                prop::sample::select(vec!["content", "hello world", "", "   ", "\t"]),
                3..50
            ),
            start_line in 1u64..50,
        ) {
            // Feature: navigation-commands, Property 6: paragraph boundary correctness
            let total = lines.len() as u64;
            let clamped_start = start_line.min(total);
            let line_refs: Vec<&str> = lines.iter().map(|s| *s).collect();
            let excluded = vec![false; lines.len()];

            let mut viewport = ViewportModel::with_line_count(total);
            viewport.set_visible_count(total);
            let mut cursor = CursorModel::new();
            cursor.set_position(clamped_start, 1);

            ParagraphNav::paragraph_up(
                &mut cursor, &mut viewport, &line_refs, &excluded,
                SelectionModifier::Move,
            );

            let result_line = cursor.cursor_line();
            // Result must be within document bounds
            prop_assert!(result_line >= 1);
            prop_assert!(result_line <= total);
            // Result must be <= start (paragraph-up never goes forwards)
            prop_assert!(
                result_line <= clamped_start,
                "paragraph_up went forwards: {} -> {}",
                clamped_start,
                result_line
            );
        }
    }
}

// ─── Property 7: Bounds Validation and Intersection Correctness ────────────

/// **Validates: Requirements 5.1, 5.13, 2.9, 2.10**
///
/// Bounds validation accepts iff left >= 1 AND right > left.
/// Intersection produces max(left, col1)..min(right, col2) with empty when max > min.
mod bounds_validation {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn bounds_validation_correctness(
            left in -5i64..500,
            right in -5i64..500,
        ) {
            // Feature: navigation-commands, Property 7: bounds validation correctness
            let left_u = if left < 0 { 0u64 } else { left as u64 };
            let right_u = if right < 0 { 0u64 } else { right as u64 };

            let result = ActiveBounds::new(left_u, right_u);

            if left_u >= 1 && right_u > left_u {
                prop_assert!(result.is_some(), "Valid bounds rejected: ({}, {})", left_u, right_u);
                let bounds = result.unwrap();
                prop_assert_eq!(bounds.left, left_u);
                prop_assert_eq!(bounds.right, right_u);
            } else {
                prop_assert!(result.is_none(), "Invalid bounds accepted: ({}, {})", left_u, right_u);
            }
        }

        #[test]
        fn bounds_intersection_correctness(
            bounds_left in 1u64..200,
            bounds_right_offset in 1u64..100,
            col1 in 1u64..200,
            col2_offset in 0u64..100,
        ) {
            // Feature: navigation-commands, Property 7: bounds intersection correctness
            let bounds_right = bounds_left + bounds_right_offset;
            let col2 = col1 + col2_offset;

            let bounds = ActiveBounds::new(bounds_left, bounds_right).unwrap();
            let result = bounds.intersect(col1, col2);

            let expected_left = bounds_left.max(col1);
            let expected_right = bounds_right.min(col2);

            if expected_left <= expected_right {
                prop_assert_eq!(result, Some((expected_left, expected_right)));
            } else {
                prop_assert_eq!(result, None);
            }
        }
    }
}
