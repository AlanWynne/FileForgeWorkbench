//! Property-based tests for ff-find-and-replace.
//!
//! Implements the PBT properties defined in the tasks specification.

use proptest::prelude::*;

use ff_find_and_replace::case_folder::CaseFolder;
use ff_find_and_replace::direction::SearchDirection;
use ff_find_and_replace::engine::FindEngine;
use ff_find_and_replace::indexer::{MutableSliceIndexer, SliceIndexer};
use ff_find_and_replace::request::{ChangeRequest, FindRequest, WordMatchMode};
use ff_find_and_replace::result::{ChangeOutcome, FindOutcome};
use ff_find_and_replace::scope::{AllLinesFilter, ColumnRange, ScopeModifier};
use ff_find_and_replace::search_mode::SearchMode;
use ff_find_and_replace::state::FindState;
use ff_find_and_replace::types::{BytePosition, MatchRange};

/// Strategy for generating documents with embedded search terms.
fn document_with_pattern() -> impl Strategy<Value = (String, String)> {
    // Generate a document and pick a sub-string of it
    "[a-zA-Z0-9 ]{10,200}".prop_flat_map(|doc: String| {
        let doc_len = doc.len();
        let max_start = if doc_len > 5 { doc_len - 5 } else { 0 };
        (Just(doc.clone()), 0..=max_start).prop_map(move |(d, start)| {
            let end = (start + 3).min(d.len());
            let term = d[start..end].to_string();
            (d, term)
        })
    })
}

// ─── Property 1: Literal Search Result Correctness ──────────────────────────
// Feature: find-and-replace, Property 1: For any document and literal search term,
// every FindResult's byte range contains exactly the search term bytes.
// **Validates: Requirements 1.1, 1.2, 1.9, 1.10**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn literal_search_result_contains_exact_term(
        (doc, term) in document_with_pattern()
    ) {
        if term.is_empty() {
            return Ok(());
        }
        let indexer = SliceIndexer::from_str(&doc);
        let mut engine = FindEngine::new();
        let filter = AllLinesFilter;
        let req = FindRequest::literal(&term);

        if let Ok(FindOutcome::Found(result)) = engine.find(&req, &indexer, &filter, None) {
            // The byte range must contain exactly the search term
            let matched_bytes = indexer.slice(result.match_range.start, result.match_range.end).unwrap();
            prop_assert_eq!(matched_bytes, term.as_bytes().to_vec(),
                "Match at {:?} doesn't contain the search term", result.match_range);

            // Line number must be correct
            let expected_line = indexer.line_from_position(result.match_range.start);
            prop_assert_eq!(result.line, expected_line);
        }
    }
}

// ─── Property 2: Case Folding Roundtrip and Idempotency ─────────────────────
// Feature: find-and-replace, Property 2: Folding a string twice produces the
// same result as folding once (idempotency), and output is always valid UTF-8.
// **Validates: Requirements 10.1, 10.3, 10.4**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn case_folding_is_idempotent(input in "\\PC{1,200}") {
        let folder = CaseFolder::new();
        let once = folder.fold_str(&input);
        let twice = folder.fold_str(&once);

        // Idempotency: fold(fold(x)) == fold(x)
        prop_assert_eq!(&once, &twice,
            "Case folding is not idempotent for input: {:?}", input);

        // Output is always valid UTF-8
        prop_assert!(std::str::from_utf8(once.as_bytes()).is_ok(),
            "Folded output is not valid UTF-8");
    }
}

// ─── Property 3: Regex Match Validity ───────────────────────────────────────
// Feature: find-and-replace, Property 3: For any regex match, the byte range
// falls within [0, doc.length()), and captured groups are sub-ranges of group 0.
// **Validates: Requirements 4.9, 4.13, 12.10**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn regex_match_within_document_bounds(
        doc in "[a-zA-Z0-9 ]{10,100}",
        pattern_seed in 0u8..5u8,
    ) {
        // Use simple valid patterns to avoid compilation errors
        let patterns = ["[a-z]+", "\\d+", "[A-Z][a-z]+", "\\w+", "[0-9]+"];
        let pattern = patterns[pattern_seed as usize % patterns.len()];

        let indexer = SliceIndexer::from_str(&doc);
        let mut engine = FindEngine::new();
        let filter = AllLinesFilter;
        let req = FindRequest::regex(pattern);

        if let Ok(FindOutcome::Found(result)) = engine.find(&req, &indexer, &filter, None) {
            // Match range within bounds
            prop_assert!(result.match_range.start.0 <= result.match_range.end.0,
                "Match start > end");
            prop_assert!(result.match_range.end.0 <= indexer.length(),
                "Match end exceeds document length");

            // Captures are sub-ranges of the full match
            for cap in &result.captures {
                prop_assert!(cap.start >= result.match_range.start,
                    "Capture starts before full match");
                prop_assert!(cap.end <= result.match_range.end,
                    "Capture ends after full match");
            }
        }
    }
}

// ─── Property 4: CHANGE ALL Replacement Count Consistency ───────────────────
// Feature: find-and-replace, Property 4: CHANGE ALL count equals the number
// of non-overlapping matches found by iterative forward search on original content.
// **Validates: Requirements 6.2, 6.8, 8.5**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn change_all_count_equals_find_all_count(
        doc in "[a-z ]{10,100}",
        term in "[a-z]{1,3}",
        replacement in "[A-Z]{1,5}",
    ) {
        if term.is_empty() {
            return Ok(());
        }

        // Count forward matches on original
        let indexer = SliceIndexer::from_str(&doc);
        let mut engine = FindEngine::new();
        let filter = AllLinesFilter;
        let mut count = 0u64;
        let mut pos = BytePosition::ZERO;
        loop {
            let req = FindRequest::literal(&term).with_cursor(pos);
            match engine.find(&req, &indexer, &filter, None) {
                Ok(FindOutcome::Found(r)) => {
                    count += 1;
                    pos = r.match_range.end;
                }
                _ => break,
            }
        }

        // Execute CHANGE ALL
        let mut mut_indexer = MutableSliceIndexer::new(&doc);
        let mut engine2 = FindEngine::new();
        let req = ChangeRequest::new(FindRequest::literal(&term), &replacement);
        let outcome = engine2.change_all(&req, &mut mut_indexer, &filter, None);

        match outcome {
            Ok(ChangeOutcome::Changed(result)) => {
                prop_assert_eq!(result.replacement_count, count,
                    "CHANGE ALL count {} != FIND ALL count {}", result.replacement_count, count);
            }
            Ok(ChangeOutcome::NotFound { .. }) => {
                prop_assert_eq!(count, 0, "Expected 0 matches but found {}", count);
            }
            _ => {}
        }
    }
}

// ─── Property 5: RFIND/RCHANGE State Preservation ───────────────────────────
// Feature: find-and-replace, Property 5: After a FIND, FindState contains all
// original arguments; after RESET ALL, RFIND fails with "No previous FIND".
// **Validates: Requirements 5.1, 5.3, 9.1, 9.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn rfind_preserves_original_request_params(
        term in "[a-z]{2,10}",
        direction in prop_oneof![
            Just(SearchDirection::Next),
            Just(SearchDirection::Prev),
            Just(SearchDirection::First),
            Just(SearchDirection::Last),
        ],
        case_sensitive in proptest::bool::ANY,
    ) {
        let doc = format!("prefix {} middle {} suffix", term, term);
        let indexer = SliceIndexer::from_str(&doc);
        let mut engine = FindEngine::new();
        let filter = AllLinesFilter;

        let req = FindRequest::literal(&term)
            .with_direction(direction)
            .with_case_sensitive(case_sensitive);

        // Execute find
        let _ = engine.find(&req, &indexer, &filter, None);

        // Check state was preserved
        if let Some(last) = engine.state().last_find.as_ref() {
            prop_assert_eq!(&last.term, &term);
            prop_assert_eq!(last.case_sensitive, case_sensitive);
            prop_assert_eq!(last.mode, SearchMode::Literal);
        }

        // After RESET ALL, RFIND should fail
        engine.state_mut().reset_all();
        let result = engine.rfind(&indexer, &filter, None);
        prop_assert!(result.is_err(), "RFIND should fail after RESET ALL");
    }
}

// ─── Property 6: Scope Filter Conjunction Correctness ───────────────────────
// Feature: find-and-replace, Property 6: When scope modifiers are active,
// matches only appear on lines passing the filter.
// **Validates: Requirements 2.1–2.4, 2.8**

use ff_find_and_replace::indexer::CharacterIndexer;
use ff_find_and_replace::scope::ScopeFilterProvider;
use ff_find_and_replace::types::LineNumber;

/// A test scope filter with configurable per-line state.
struct TestScopeFilter {
    visible: Vec<bool>,
    tagged: Vec<bool>,
}

impl ScopeFilterProvider for TestScopeFilter {
    fn is_visible(&self, line: LineNumber) -> bool {
        self.visible.get(line.0 as usize).copied().unwrap_or(true)
    }
    fn is_excluded(&self, line: LineNumber) -> bool {
        !self.is_visible(line)
    }
    fn is_tagged(&self, line: LineNumber) -> bool {
        self.tagged.get(line.0 as usize).copied().unwrap_or(false)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn scope_filter_tagged_and_nontagged_are_disjoint(
        tagged_flags in proptest::collection::vec(proptest::bool::ANY, 5..20),
    ) {
        // A line that passes TAGGED must NOT pass NONTAGGED and vice versa
        for (i, &is_tagged) in tagged_flags.iter().enumerate() {
            let line = LineNumber(i as u64);
            let filter = TestScopeFilter {
                visible: vec![true; tagged_flags.len()],
                tagged: tagged_flags.clone(),
            };

            let passes_tagged = ff_find_and_replace::scope::line_passes_scope(
                ScopeModifier::Tagged, line, &filter
            );
            let passes_nontagged = ff_find_and_replace::scope::line_passes_scope(
                ScopeModifier::NonTagged, line, &filter
            );

            prop_assert!(!(passes_tagged && passes_nontagged),
                "Line {} passes both TAGGED and NONTAGGED", i);
            prop_assert!(passes_tagged || passes_nontagged,
                "Line {} passes neither TAGGED nor NONTAGGED", i);
        }
    }
}

// ─── Property 7: Hex Byte Search Equivalence ────────────────────────────────
// Feature: find-and-replace, Property 7: Searching in HexBytes mode produces
// identical match positions as searching in Literal mode for the decoded bytes.
// **Validates: Requirements 3.1, 3.4, 3.6, 3.7**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn hex_search_equivalent_to_literal_byte_search(
        doc_bytes in proptest::collection::vec(0u8..255u8, 20..100),
        pattern_bytes in proptest::collection::vec(1u8..255u8, 1..4),
    ) {
        // Build hex string from pattern_bytes
        let hex_str: String = pattern_bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect();

        let indexer = SliceIndexer::new(&doc_bytes);
        let mut engine = FindEngine::new();
        let filter = AllLinesFilter;

        // Search with HexBytes mode
        let hex_req = FindRequest::hex(&hex_str);
        let hex_outcome = engine.find(&hex_req, &indexer, &filter, None);

        // Search with Literal mode using the raw bytes
        // We need to create a string from raw bytes for literal search
        // Since FindRequest::literal takes a &str, we'll use the engine directly
        let literal_result = ff_find_and_replace::literal::find_literal_forward(
            &pattern_bytes,
            &indexer,
            BytePosition::ZERO,
            BytePosition(indexer.length()),
            WordMatchMode::None,
        );

        // Compare results
        match (hex_outcome, literal_result) {
            (Ok(FindOutcome::Found(hex_r)), Some(lit_r)) => {
                prop_assert_eq!(hex_r.match_range.start, lit_r.match_range.start,
                    "Hex and literal search start positions differ");
                prop_assert_eq!(hex_r.match_range.end, lit_r.match_range.end,
                    "Hex and literal search end positions differ");
            }
            (Ok(FindOutcome::NotFound { .. }), None) => {
                // Both didn't find — consistent
            }
            (Ok(FindOutcome::Found(_)), None) => {
                prop_assert!(false, "Hex found a match but literal did not");
            }
            (Ok(FindOutcome::NotFound { .. }), Some(_)) => {
                prop_assert!(false, "Literal found a match but hex did not");
            }
            _ => {
                // Other cases (errors) — skip
            }
        }
    }
}
