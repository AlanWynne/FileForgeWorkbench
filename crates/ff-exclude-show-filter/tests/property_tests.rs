//! Property-based tests for ff-exclude-show-filter.
//!
//! Uses `proptest` crate with minimum 100 iterations per property.
//! Each property validates specific acceptance criteria from the
//! requirements document.

use ff_display_line_mapping::{ContractionState, DisplayLine, DisplayLineMapping};
use ff_exclude_show_filter::{
    DocumentAccess, ExcludeArgs, ExcludeScope, ExclusionEngine, ResetVariant, ShowArgs,
};
use proptest::prelude::*;

/// Simple in-memory document for property testing.
#[derive(Debug, Clone)]
struct PropDoc {
    lines: Vec<String>,
}

impl PropDoc {
    fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

impl DocumentAccess for PropDoc {
    fn line_content(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }
    fn line_count(&self) -> usize {
        self.lines.len()
    }
    fn is_tagged(&self, _line: usize) -> bool {
        false
    }
}

/// Strategy for generating random operations on an exclusion engine.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum RandomOp {
    ExcludeRange { start: usize, end: usize },
    ShowRange { start: usize, end: usize },
    ShowAll,
}

/// Generate a random operation given a document size.
#[allow(dead_code)]
fn arb_op(max_line: usize) -> impl Strategy<Value = RandomOp> {
    prop_oneof![
        3 => (0..max_line, 0..max_line).prop_map(|(a, b)| {
            let start = a.min(b);
            let end = a.max(b);
            RandomOp::ExcludeRange { start, end }
        }),
        2 => (0..max_line, 0..max_line).prop_map(|(a, b)| {
            let start = a.min(b);
            let end = a.max(b);
            RandomOp::ShowRange { start, end }
        }),
        1 => Just(RandomOp::ShowAll),
    ]
}

/// Apply a random operation to an engine.
#[allow(dead_code)]
fn apply_op<D: DisplayLineMapping, A: DocumentAccess>(
    engine: &mut ExclusionEngine<D, A>,
    op: &RandomOp,
) {
    match *op {
        RandomOp::ExcludeRange { start, end } => {
            engine.exclude_range(start, end);
        }
        RandomOp::ShowRange { start, end } => {
            engine.show_range(start, end);
        }
        RandomOp::ShowAll => {
            engine.show_all();
        }
    }
}

// ─── Property 1: Exclusion State Consistency ────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 7.1**
    ///
    /// For any sequence of exclude/show operations, a line marked as excluded
    /// SHALL always report get_visible(line) == false in the display-line-mapping,
    /// and a line not excluded SHALL report get_visible(line) == true.
    #[test]
    fn exclusion_state_consistency(
        line_count in 1usize..200,
        ops in prop::collection::vec(any::<(usize, usize, bool)>(), 1..20),
    ) {
        // Feature: exclude-show-filter, Property 1: Exclusion State Consistency
        let doc = PropDoc::new(vec!["line".to_string(); line_count]);
        let mapping = ContractionState::new(line_count);
        let mut engine = ExclusionEngine::new(mapping, doc);

        // Apply random operations
        for (a, b, exclude) in &ops {
            let start = *a % line_count;
            let end = *b % line_count;
            let (start, end) = (start.min(end), start.max(end));
            if *exclude {
                engine.exclude_range(start, end);
            } else {
                engine.show_range(start, end);
            }
        }

        // Verify invariant: is_excluded consistent with display mapping
        for line in 0..line_count {
            let excluded = engine.is_excluded(line);
            let visible = engine.display_mapping().get_visible(
                ff_display_line_mapping::DocLine(line),
            );
            prop_assert_eq!(
                excluded, !visible,
                "Line {}: is_excluded={} but get_visible={}",
                line, excluded, visible
            );
        }
    }
}

// ─── Property 2: SHOW Reverses EXCLUDE ──────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 2.1, 3.1, 3.4, 4.2**
    ///
    /// For any set of lines excluded via EXCLUDE 'text' and then shown via
    /// SHOW 'text' on the same document, those lines SHALL return to visible.
    #[test]
    fn show_reverses_exclude_for_text_match(
        lines in prop::collection::vec("[a-z]{1,20}", 1..50),
        search_idx in 0usize..50,
    ) {
        // Feature: exclude-show-filter, Property 2: SHOW Reverses EXCLUDE
        let line_count = lines.len();
        let doc = PropDoc::new(lines.clone());
        let mapping = ContractionState::new(line_count);
        let mut engine = ExclusionEngine::new(mapping, doc);

        // Pick a search term from an existing line (substring)
        let idx = search_idx % line_count;
        let line_str = &lines[idx];
        let term = if line_str.len() >= 2 {
            &line_str[..2]
        } else {
            line_str.as_str()
        };

        // EXCLUDE 'text'
        let exclude_args = ExcludeArgs::Text {
            pattern: term.to_string(),
            scope: ExcludeScope::Visible,
        };
        engine.execute_exclude(&exclude_args).unwrap();

        // SHOW 'text'
        let show_args = ShowArgs::Text {
            pattern: term.to_string(),
        };
        engine.execute_show(&show_args).unwrap();

        // After EXCLUDE then SHOW with same term, lines that matched
        // should be visible again
        for line in 0..line_count {
            if let Some(content) = engine.document().line_content(line) {
                if content.to_lowercase().contains(&term.to_lowercase()) {
                    prop_assert!(
                        !engine.is_excluded(line),
                        "Line {} contains '{}' but is still excluded after SHOW",
                        line, term
                    );
                }
            }
        }
    }
}

// ─── Property 3: Block Contiguity Invariant ─────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 6.1, 6.5, 6.6**
    ///
    /// The list of ExclusionBlocks SHALL be maximally contiguous: no two
    /// adjacent blocks can be merged, and each block covers a contiguous
    /// range where every line is excluded.
    #[test]
    fn block_contiguity_invariant(
        line_count in 1usize..200,
        excluded_bits in prop::collection::vec(any::<bool>(), 1..200),
    ) {
        // Feature: exclude-show-filter, Property 3: Block Contiguity Invariant
        let actual_count = line_count.min(excluded_bits.len());
        let doc = PropDoc::new(vec!["line".to_string(); actual_count]);
        let mapping = ContractionState::new(actual_count);
        let mut engine = ExclusionEngine::new(mapping, doc);

        // Apply random exclusion pattern
        for (i, &exclude) in excluded_bits.iter().take(actual_count).enumerate() {
            if exclude {
                engine.exclude_line(i);
            }
        }

        let blocks = engine.exclusion_blocks();

        // Check: all lines in each block are excluded
        for block in &blocks {
            for line in block.start_line..=block.end_line {
                prop_assert!(
                    engine.is_excluded(line),
                    "Block {:?} claims line {} but it's not excluded",
                    block, line
                );
            }
        }

        // Check: blocks are ordered and non-adjacent (gap between them)
        for i in 1..blocks.len() {
            let prev = &blocks[i - 1];
            let curr = &blocks[i];
            prop_assert!(
                prev.end_line + 1 < curr.start_line,
                "Blocks {:?} and {:?} should have a gap between them",
                prev, curr
            );
        }

        // Check: no excluded line outside blocks
        let mut block_idx = 0;
        for line in 0..actual_count {
            let in_block = blocks
                .get(block_idx)
                .map(|b| line >= b.start_line && line <= b.end_line)
                .unwrap_or(false);
            if in_block {
                if line == blocks[block_idx].end_line {
                    block_idx += 1;
                }
            } else {
                prop_assert!(
                    !engine.is_excluded(line),
                    "Line {} is excluded but not in any block",
                    line
                );
            }
        }
    }
}

// ─── Property 4: RESET Restores All Visibility ──────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 4.1, 4.2, 4.4, 4.7**
    ///
    /// After any sequence of EXCLUDE operations followed by reset_excluded(),
    /// no line in the document SHALL remain excluded.
    #[test]
    fn reset_restores_all_visibility(
        line_count in 1usize..500,
        ops in prop::collection::vec((0usize..500, 0usize..500), 1..15),
    ) {
        // Feature: exclude-show-filter, Property 4: RESET Restores All Visibility
        let doc = PropDoc::new(vec!["line".to_string(); line_count]);
        let mapping = ContractionState::new(line_count);
        let mut engine = ExclusionEngine::new(mapping, doc);

        // Apply random exclude operations
        for (a, b) in &ops {
            let start = *a % line_count;
            let end = *b % line_count;
            let (start, end) = (start.min(end), start.max(end));
            engine.exclude_range(start, end);
        }

        // RESET
        engine.execute_reset(ResetVariant::Excluded);

        // Verify all lines visible
        for line in 0..line_count {
            prop_assert!(
                !engine.is_excluded(line),
                "Line {} still excluded after RESET",
                line
            );
        }
        prop_assert_eq!(engine.excluded_line_count(), 0);
    }
}

// ─── Property 5: Excluded Line Count Consistency ────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 1.7, 1.8, 10.1**
    ///
    /// The value returned by excluded_line_count() SHALL always equal the
    /// number of lines for which is_excluded(line) returns true.
    #[test]
    fn excluded_line_count_consistency(
        line_count in 1usize..300,
        ops in prop::collection::vec(any::<(usize, usize, bool)>(), 1..20),
    ) {
        // Feature: exclude-show-filter, Property 5: Excluded Line Count Consistency
        let doc = PropDoc::new(vec!["line".to_string(); line_count]);
        let mapping = ContractionState::new(line_count);
        let mut engine = ExclusionEngine::new(mapping, doc);

        // Apply random operations
        for (a, b, exclude) in &ops {
            let start = *a % line_count;
            let end = *b % line_count;
            let (start, end) = (start.min(end), start.max(end));
            if *exclude {
                engine.exclude_range(start, end);
            } else {
                engine.show_range(start, end);
            }
        }

        // Verify count consistency
        let reported_count = engine.excluded_line_count();
        let actual_count = (0..line_count)
            .filter(|&l| engine.is_excluded(l))
            .count();
        prop_assert_eq!(
            reported_count, actual_count,
            "excluded_line_count() = {} but actual count = {}",
            reported_count, actual_count
        );
    }
}

// ─── Property 6: EXCLUDE ALL + SHOW Text Filtering ──────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 2.4, 3.4, 8.7**
    ///
    /// After exclude_all() followed by show_text(term), the set of visible
    /// lines SHALL be exactly those lines whose content contains the search term.
    #[test]
    fn exclude_all_show_text_filters_correctly(
        lines in prop::collection::vec("[a-z]{1,30}", 1..50),
        term in "[a-z]{1,5}",
    ) {
        // Feature: exclude-show-filter, Property 6: EXCLUDE ALL + SHOW Text Filtering
        let line_count = lines.len();
        let doc = PropDoc::new(lines.clone());
        let mapping = ContractionState::new(line_count);
        let mut engine = ExclusionEngine::new(mapping, doc);

        // EXCLUDE ALL
        engine.execute_exclude(&ExcludeArgs::All).unwrap();

        // SHOW 'term'
        engine
            .execute_show(&ShowArgs::Text {
                pattern: term.clone(),
            })
            .unwrap();

        // Verify: visible lines are exactly those containing term
        let term_lower = term.to_lowercase();
        for (i, line) in lines.iter().enumerate() {
            let should_be_visible = line.to_lowercase().contains(&term_lower);
            let is_visible = !engine.is_excluded(i);
            prop_assert_eq!(
                is_visible, should_be_visible,
                "Line {} '{}': expected visible={} but got visible={} (term='{}')",
                i, line, should_be_visible, is_visible, term
            );
        }
    }
}

// ─── Property 7: Doc-from-Display Never Returns Excluded Line ────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 7.4, 7.7**
    ///
    /// For any valid display line index, doc_from_display(display_line) SHALL
    /// never return a document line that is currently excluded.
    #[test]
    fn doc_from_display_never_returns_excluded_line(
        line_count in 2usize..100,
        excluded_bits in prop::collection::vec(any::<bool>(), 2..100),
    ) {
        // Feature: exclude-show-filter, Property 7: Doc-from-Display Never Returns Excluded Line
        let actual_count = line_count.min(excluded_bits.len());
        let doc = PropDoc::new(vec!["line".to_string(); actual_count]);
        let mapping = ContractionState::new(actual_count);
        let mut engine = ExclusionEngine::new(mapping, doc);

        // Ensure at least one line stays visible
        let mut has_visible = false;
        for (i, &exclude) in excluded_bits.iter().take(actual_count).enumerate() {
            if exclude && i < actual_count - 1 {
                engine.exclude_line(i);
            } else {
                has_visible = true;
            }
        }

        if !has_visible {
            // Make last line visible to avoid empty display
            engine.show_line(actual_count - 1);
        }

        let displayed = engine.display_mapping().lines_displayed();
        prop_assert!(displayed > 0, "Must have at least one display line");

        for d in 0..displayed {
            let pos = engine
                .display_mapping()
                .doc_from_display(DisplayLine(d));
            prop_assert!(
                !engine.is_excluded(pos.doc_line.0),
                "Display line {} mapped to excluded doc line {}",
                d, pos.doc_line.0
            );
        }
    }
}
