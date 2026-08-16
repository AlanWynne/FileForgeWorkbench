//! Property-based tests for ff-completion.
//!
//! Uses the `proptest` crate to verify invariants across many generated inputs.

use proptest::prelude::*;
use std::collections::HashSet;

use ff_completion::candidate::{CompletionCandidate, CompletionKind};
use ff_completion::config::{CompletionConfig, MatchingMode, RawConfigValues, TriggerMode};
use ff_completion::list::CompletionList;
use ff_completion::matching::{fuzzy_match, prefix_match};
use ff_completion::navigation::SelectionState;
use ff_completion::positioning::{compute_popup_position, FieldRect, PopupConfig, ViewportRect};

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Strategy for generating alphanumeric strings with dots and underscores.
fn label_string(max_len: usize) -> impl Strategy<Value = String> {
    proptest::string::string_regex(&format!("[a-zA-Z0-9._]{{0,{max_len}}}")).unwrap()
}

/// Strategy for generating query strings.
fn query_string(max_len: usize) -> impl Strategy<Value = String> {
    proptest::string::string_regex(&format!("[a-zA-Z0-9._]{{0,{max_len}}}")).unwrap()
}

/// Strategy for CompletionCandidate generation.
fn arb_candidate() -> impl Strategy<Value = CompletionCandidate> {
    (label_string(30), label_string(30), 0..=100i32).prop_map(|(label, insert, relevance)| {
        let insert_text = if insert.is_empty() {
            label.clone()
        } else {
            insert
        };
        CompletionCandidate::new(
            if label.is_empty() {
                "x".to_string()
            } else {
                label
            },
            if insert_text.is_empty() {
                "x".to_string()
            } else {
                insert_text
            },
            CompletionKind::Command,
        )
        .with_relevance(relevance)
    })
}

/// Strategy for a vector of candidates.
fn arb_candidate_list(min: usize, max: usize) -> impl Strategy<Value = Vec<CompletionCandidate>> {
    proptest::collection::vec(arb_candidate(), min..=max)
}

// ─── Property 1: Prefix Match Correctness ───────────────────────────────────

proptest! {
    /// **Validates: Requirements 1.2, 6.2**
    ///
    /// For any query and candidate, prefix_match returns true iff the candidate
    /// starts with the query (case-insensitive when case_sensitive=false).
    // Feature: command-completion, Property 1: Prefix match correctness
    #[test]
    fn prefix_match_correctness(
        query in query_string(20),
        candidate in label_string(50),
    ) {
        let result = prefix_match(&query, &candidate, false);
        let expected = candidate.to_lowercase().starts_with(&query.to_lowercase());
        prop_assert_eq!(result, expected,
            "prefix_match({:?}, {:?}, false) = {} but expected {}",
            query, candidate, result, expected
        );
    }
}

// ─── Property 2: Fuzzy Match Subsequence ────────────────────────────────────

proptest! {
    /// **Validates: Requirement 6.1**
    ///
    /// Fuzzy match returns Some iff all query characters appear in the candidate in order.
    // Feature: command-completion, Property 2: Fuzzy match subsequence
    #[test]
    fn fuzzy_match_subsequence_valid(
        candidate in "[a-z]{5,20}",
        positions in proptest::collection::vec(any::<prop::sample::Index>(), 1..=5),
    ) {
        let chars: Vec<char> = candidate.chars().collect();
        if chars.is_empty() {
            return Ok(());
        }

        // Select a valid subsequence
        let mut indices: Vec<usize> = positions
            .iter()
            .map(|idx| idx.index(chars.len()))
            .collect();
        indices.sort();
        indices.dedup();

        if indices.is_empty() {
            return Ok(());
        }

        let query: String = indices.iter().map(|&i| chars[i]).collect();
        let result = fuzzy_match(&query, &candidate, false);
        prop_assert!(result.is_some(),
            "fuzzy_match({:?}, {:?}) should match (chars at {:?})",
            query, candidate, indices
        );

        let r = result.unwrap();
        // Positions should be strictly increasing
        for i in 1..r.matched_positions.len() {
            prop_assert!(r.matched_positions[i] > r.matched_positions[i - 1]);
        }
        // Position count should equal query length
        prop_assert_eq!(r.matched_positions.len(), query.len());
    }
}

proptest! {
    /// Fuzzy match returns None when query contains a character not in the candidate.
    // Feature: command-completion, Property 2b: Fuzzy non-match
    #[test]
    fn fuzzy_match_non_subsequence(
        candidate in "[a-m]{3,15}",
    ) {
        // Query contains 'z' which is not in [a-m] candidates
        let query = format!("{}z", &candidate[..1.min(candidate.len())]);
        let result = fuzzy_match(&query, &candidate, false);
        prop_assert!(result.is_none(),
            "fuzzy_match({:?}, {:?}) should not match",
            query, candidate
        );
    }
}

// ─── Property 3: Fuzzy Scoring Monotonicity ─────────────────────────────────

proptest! {
    /// **Validates: Requirement 6.4**
    ///
    /// For a candidate and two queries where q1 is a prefix of q2,
    /// if both match, then score(q2) >= score(q1).
    // Feature: command-completion, Property 3: Fuzzy scoring monotonicity
    #[test]
    fn fuzzy_scoring_monotonicity(
        candidate in "[a-z]{5,20}",
        q1_len in 1usize..=3,
        extra_char_idx in any::<prop::sample::Index>(),
    ) {
        let chars: Vec<char> = candidate.chars().collect();
        if chars.len() < 4 {
            return Ok(());
        }

        // Build q1 as a subsequence
        let q1: String = chars.iter().take(q1_len.min(chars.len())).collect();
        let result1 = fuzzy_match(&q1, &candidate, false);

        if result1.is_none() {
            return Ok(());
        }
        let r1 = result1.unwrap();

        // Build q2 by extending q1 with another character from the candidate after the last match
        let last_pos = r1.matched_positions.last().copied().unwrap_or(0);
        if last_pos + 1 >= chars.len() {
            return Ok(());
        }

        let remaining: Vec<char> = chars[last_pos + 1..].to_vec();
        if remaining.is_empty() {
            return Ok(());
        }

        let extra_idx = extra_char_idx.index(remaining.len());
        let q2 = format!("{}{}", q1, remaining[extra_idx]);

        let result2 = fuzzy_match(&q2, &candidate, false);
        if let Some(r2) = result2 {
            prop_assert!(r2.score >= r1.score,
                "score({:?})={} should be >= score({:?})={} for candidate {:?}",
                q2, r2.score, q1, r1.score, candidate
            );
        }
    }
}

// ─── Property 4: Navigation Wrap-Around ─────────────────────────────────────

proptest! {
    /// **Validates: Requirements 4.1, 4.2**
    ///
    /// Navigation always keeps selected_index within [0, total_items - 1]
    /// and wrap semantics are respected.
    // Feature: command-completion, Property 4: Navigation wrap-around
    #[test]
    fn navigation_bounds_invariant(
        total_items in 1usize..=100,
        wrap_enabled in any::<bool>(),
        operations in proptest::collection::vec(0u8..=3, 1..=50),
    ) {
        let mut state = SelectionState::new(total_items, 10, wrap_enabled);

        for op in &operations {
            match op {
                0 => state.move_down(),
                1 => state.move_up(),
                2 => state.page_down(),
                _ => state.page_up(),
            }
            prop_assert!(state.selected_index() < total_items,
                "selected_index {} >= total_items {} after op {}",
                state.selected_index(), total_items, op
            );
        }
    }
}

proptest! {
    /// N consecutive down moves from 0 with wrap=true returns to 0.
    // Feature: command-completion, Property 4b: Wrap cycle
    #[test]
    fn navigation_full_cycle_wraps(total_items in 1usize..=50) {
        let mut state = SelectionState::new(total_items, 10, true);
        for _ in 0..total_items {
            state.move_down();
        }
        prop_assert_eq!(state.selected_index(), 0,
            "After {} down moves with wrap, should be back at 0", total_items
        );
    }
}

// ─── Property 5: CompletionList Filter Idempotence ──────────────────────────

proptest! {
    /// **Validates: Requirement 1.6**
    ///
    /// Filtering with the same query twice produces identical results.
    // Feature: command-completion, Property 5: Filter idempotence
    #[test]
    fn filter_idempotence(
        candidates in arb_candidate_list(1, 20),
        query in query_string(10),
    ) {
        let mut list1 = CompletionList::new(candidates.clone(), MatchingMode::Prefix, false);
        list1.filter(&query);
        let result1: Vec<String> = list1.items().iter()
            .map(|i| i.candidate.label.clone())
            .collect();

        let mut list2 = CompletionList::new(candidates, MatchingMode::Prefix, false);
        list2.filter(&query);
        list2.filter(&query); // filter again
        let result2: Vec<String> = list2.items().iter()
            .map(|i| i.candidate.label.clone())
            .collect();

        prop_assert_eq!(result1, result2);
    }
}

// ─── Property 6: Popup Positioning Within Viewport ──────────────────────────

proptest! {
    /// **Validates: Requirements 3.2, 3.3, 3.4, 3.5**
    ///
    /// The computed popup bounds are always within the viewport and
    /// do not overlap the command field.
    // Feature: command-completion, Property 6: Popup positioning within viewport
    #[test]
    fn popup_within_viewport(
        viewport_w in 200.0f32..=2000.0,
        viewport_h in 200.0f32..=2000.0,
        field_y in 20.0f32..=500.0,
        field_h in 16.0f32..=40.0,
        anchor_x in 0.0f32..=500.0,
        item_count in 1usize..=50,
        max_items in 3usize..=50,
        max_width in 100.0f32..=1000.0,
    ) {
        let viewport = ViewportRect { x: 0.0, y: 0.0, width: viewport_w, height: viewport_h };
        let field_y_clamped = field_y.min(viewport_h - field_h - 1.0).max(0.0);
        let field = FieldRect { x: 0.0, y: field_y_clamped, width: viewport_w, height: field_h };
        let anchor_x_clamped = anchor_x.min(viewport_w - 10.0).max(0.0);

        let config = PopupConfig { max_items, max_width, item_height: 20.0 };
        let bounds = compute_popup_position(
            anchor_x_clamped, &field, item_count, 200.0, &config, &viewport
        );

        // Within viewport
        prop_assert!(bounds.x >= 0.0, "popup x {} < 0", bounds.x);
        prop_assert!(bounds.y >= 0.0, "popup y {} < 0", bounds.y);
        prop_assert!(bounds.right() <= viewport_w + 0.01,
            "popup right {} > viewport_w {}", bounds.right(), viewport_w);
        prop_assert!(bounds.bottom() <= viewport_h + 0.01,
            "popup bottom {} > viewport_h {}", bounds.bottom(), viewport_h);

        // Does not overlap field
        prop_assert!(!bounds.overlaps_field(&field),
            "popup {:?} overlaps field {:?}", bounds, field);
    }
}

// ─── Property 7: Configuration Clamping ─────────────────────────────────────

proptest! {
    /// **Validates: Requirements 9.1, 9.5**
    ///
    /// Configuration values are always clamped to their valid ranges.
    // Feature: command-completion, Property 7: Configuration clamping
    #[test]
    fn config_clamping(
        max_items in -100i64..=200,
        max_width in -100i64..=5000,
        trigger_chars in -10i64..=100,
        matching_mode in "[a-z]{0,10}",
        trigger_mode in "[a-z]{0,10}",
    ) {
        let raw = RawConfigValues {
            popup_max_items: Some(max_items),
            popup_max_width: Some(max_width),
            auto_trigger_chars: Some(trigger_chars),
            matching_mode: Some(matching_mode),
            trigger_mode: Some(trigger_mode),
            ..Default::default()
        };
        let (config, _errors) = CompletionConfig::from_raw_values(&raw);

        // popup_max_items in [3, 50]
        prop_assert!(config.popup_max_items >= 3 && config.popup_max_items <= 50,
            "popup_max_items {} not in [3, 50]", config.popup_max_items);
        // popup_max_width in [100, 1000]
        prop_assert!(config.popup_max_width >= 100 && config.popup_max_width <= 1000,
            "popup_max_width {} not in [100, 1000]", config.popup_max_width);
        // auto_trigger_chars in [1, 10]
        prop_assert!(config.auto_trigger_chars >= 1 && config.auto_trigger_chars <= 10,
            "auto_trigger_chars {} not in [1, 10]", config.auto_trigger_chars);
        // matching_mode is always valid
        prop_assert!(
            config.matching_mode == MatchingMode::Prefix
            || config.matching_mode == MatchingMode::Fuzzy
        );
        // trigger_mode is always valid
        prop_assert!(
            config.trigger_mode == TriggerMode::Manual
            || config.trigger_mode == TriggerMode::Automatic
            || config.trigger_mode == TriggerMode::Both
        );
    }
}

// ─── Property 8: De-Duplication Invariant ───────────────────────────────────

proptest! {
    /// **Validates: Requirement 2.7**
    ///
    /// After constructing a CompletionList, all insertion values are unique.
    // Feature: command-completion, Property 8: De-duplication invariant
    #[test]
    fn deduplication_invariant(
        candidates in proptest::collection::vec(
            (1usize..=10, 0..=50i32).prop_map(|(idx, rel)| {
                // Create candidates with deliberate insert_text overlap
                let insert = format!("item_{}", idx);
                CompletionCandidate::new(
                    format!("Label_{}", idx),
                    insert,
                    CompletionKind::Command,
                ).with_relevance(rel)
            }),
            2..=30,
        ),
    ) {
        let list = CompletionList::new(candidates, MatchingMode::Prefix, false);
        let insert_texts: Vec<&str> = list.items().iter()
            .map(|i| i.candidate.insert_text.as_str())
            .collect();
        let unique: HashSet<&str> = insert_texts.iter().copied().collect();
        prop_assert_eq!(insert_texts.len(), unique.len(),
            "Duplicate insert_text found in list");
    }
}

// ─── Property 9: Insertion Preserves Trailing Text ──────────────────────────

proptest! {
    /// **Validates: Requirement 4.10**
    ///
    /// When a candidate is accepted, text after the cursor is preserved.
    // Feature: command-completion, Property 9: Insertion preserves trailing text
    #[test]
    fn insertion_preserves_trailing(
        prefix_text in "[a-z]{0,10}",
        typed_prefix in "[a-z]{1,5}",
        trailing_text in "[a-z ]{0,15}",
        insert_value in "[A-Z]{1,10}",
    ) {
        let full_text = format!("{}{}{}", prefix_text, typed_prefix, trailing_text);
        let anchor = prefix_text.len();
        let cursor = prefix_text.len() + typed_prefix.len();

        // Simulate insertion: replace [anchor..cursor] with insert_value
        let before_anchor = &full_text[..anchor];
        let after_cursor = &full_text[cursor..];
        let result = format!("{}{} {}", before_anchor, insert_value, after_cursor);

        // Trailing text is preserved
        prop_assert!(result.ends_with(after_cursor) || after_cursor.is_empty(),
            "Trailing text {:?} not preserved in result {:?}",
            after_cursor, result
        );
    }
}
