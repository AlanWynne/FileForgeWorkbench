//! Integration tests for ff-exclude-show-filter.
//!
//! Tests the full EXCLUDE → SHOW → RESET lifecycle, text filtering
//! workflows, line commands, and block enumeration.

use ff_display_line_mapping::{ContractionState, DisplayLine, DisplayLineMapping};
use ff_exclude_show_filter::{
    DocumentAccess, ExcludeArgs, ExcludeScope, ExclusionEngine, LineCommandExclude, ResetVariant,
    ShowArgs,
};

/// Simple in-memory document for testing.
struct TestDoc {
    lines: Vec<String>,
    tags: Vec<bool>,
}

impl TestDoc {
    fn new(lines: Vec<&str>) -> Self {
        let count = lines.len();
        Self {
            lines: lines.into_iter().map(String::from).collect(),
            tags: vec![false; count],
        }
    }

    fn with_tags(lines: Vec<&str>, tags: Vec<bool>) -> Self {
        Self {
            lines: lines.into_iter().map(String::from).collect(),
            tags,
        }
    }
}

impl DocumentAccess for TestDoc {
    fn line_content(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }
    fn line_count(&self) -> usize {
        self.lines.len()
    }
    fn is_tagged(&self, line: usize) -> bool {
        self.tags.get(line).copied().unwrap_or(false)
    }
}

/// Helper to create engine with test document.
fn make_engine(lines: Vec<&str>) -> ExclusionEngine<ContractionState, TestDoc> {
    let count = lines.len();
    let doc = TestDoc::new(lines);
    let mapping = ContractionState::new(count);
    ExclusionEngine::new(mapping, doc)
}

// ─── Lifecycle Tests ────────────────────────────────────────────────────────

#[test]
fn full_exclude_show_reset_lifecycle() {
    // Validates: Requirements 2, 3, 4 — full lifecycle
    let mut engine = make_engine(vec![
        "hello world",
        "foo bar",
        "hello again",
        "baz qux",
        "world hello",
    ]);

    // Initially all visible
    assert!(!engine.has_excluded_lines());
    assert_eq!(engine.excluded_line_count(), 0);

    // EXCLUDE 'hello'
    let args = ExcludeArgs::Text {
        pattern: "hello".to_string(),
        scope: ExcludeScope::Visible,
    };
    let result = engine.execute_exclude(&args).unwrap();
    assert_eq!(result.lines_affected, 3);
    assert!(engine.is_excluded(0));
    assert!(!engine.is_excluded(1));
    assert!(engine.is_excluded(2));
    assert!(!engine.is_excluded(3));
    assert!(engine.is_excluded(4));
    assert_eq!(engine.excluded_line_count(), 3);

    // SHOW 'again'
    let show_args = ShowArgs::Text {
        pattern: "again".to_string(),
    };
    let show_result = engine.execute_show(&show_args).unwrap();
    assert_eq!(show_result.lines_shown, 1);
    assert!(!engine.is_excluded(2)); // "hello again" now visible
    assert!(engine.is_excluded(0)); // still excluded
    assert_eq!(engine.excluded_line_count(), 2);

    // RESET EXCLUDED
    let reset_result = engine.execute_reset(ResetVariant::Excluded);
    assert_eq!(reset_result.lines_restored, 2);
    assert!(!engine.has_excluded_lines());
    assert_eq!(engine.excluded_line_count(), 0);
}

#[test]
fn exclude_all_then_show_text_filtering_workflow() {
    // Validates: Requirement 2 AC 4, Requirement 3 AC 4, Requirement 8 AC 7
    let mut engine = make_engine(vec![
        "import os",
        "import sys",
        "def main():",
        "    print('hello')",
        "    return 0",
        "# end of file",
    ]);

    // EXCLUDE ALL
    let result = engine.execute_exclude(&ExcludeArgs::All).unwrap();
    assert_eq!(result.lines_affected, 6);
    assert_eq!(engine.excluded_line_count(), 6);

    // SHOW 'import'
    let show_result = engine
        .execute_show(&ShowArgs::Text {
            pattern: "import".to_string(),
        })
        .unwrap();
    assert_eq!(show_result.lines_shown, 2);

    // Only lines with "import" are visible
    assert!(!engine.is_excluded(0)); // "import os"
    assert!(!engine.is_excluded(1)); // "import sys"
    assert!(engine.is_excluded(2)); // "def main():"
    assert!(engine.is_excluded(3));
    assert!(engine.is_excluded(4));
    assert!(engine.is_excluded(5));
}

#[test]
fn exclude_text_with_all_scope_ignores_visibility() {
    // Validates: Requirement 2 AC 2
    let mut engine = make_engine(vec!["alpha hello", "beta", "gamma hello", "delta"]);

    // First exclude "hello" lines
    let args = ExcludeArgs::Text {
        pattern: "hello".to_string(),
        scope: ExcludeScope::Visible,
    };
    engine.execute_exclude(&args).unwrap();
    assert_eq!(engine.excluded_line_count(), 2);

    // Now exclude "beta" with ALL scope — should still work on visible lines
    let args2 = ExcludeArgs::Text {
        pattern: "beta".to_string(),
        scope: ExcludeScope::All,
    };
    let result = engine.execute_exclude(&args2).unwrap();
    assert_eq!(result.lines_affected, 1);
    assert_eq!(engine.excluded_line_count(), 3);
}

#[test]
fn exclude_tagged_only_affects_tagged_lines() {
    // Validates: Requirement 2 AC 5
    let doc = TestDoc::with_tags(
        vec!["line 1", "line 2", "line 3", "line 4"],
        vec![false, true, false, true],
    );
    let mapping = ContractionState::new(4);
    let mut engine = ExclusionEngine::new(mapping, doc);

    let result = engine.execute_exclude(&ExcludeArgs::Tagged).unwrap();
    assert_eq!(result.lines_affected, 2);
    assert!(!engine.is_excluded(0));
    assert!(engine.is_excluded(1));
    assert!(!engine.is_excluded(2));
    assert!(engine.is_excluded(3));
}

#[test]
fn exclude_range_by_number_one_based() {
    // Validates: Requirement 2 AC 6
    let mut engine = make_engine(vec!["line 1", "line 2", "line 3", "line 4", "line 5"]);

    let args = ExcludeArgs::Range {
        start_line: 2,
        end_line: 4,
    };
    let result = engine.execute_exclude(&args).unwrap();
    assert_eq!(result.lines_affected, 3);
    assert!(!engine.is_excluded(0)); // line 1
    assert!(engine.is_excluded(1)); // line 2
    assert!(engine.is_excluded(2)); // line 3
    assert!(engine.is_excluded(3)); // line 4
    assert!(!engine.is_excluded(4)); // line 5
}

#[test]
fn exclude_range_invalid_returns_error() {
    // Validates: Requirement 9 AC 8
    let mut engine = make_engine(vec!["a", "b", "c"]);

    // start > end
    let result = engine.execute_exclude(&ExcludeArgs::Range {
        start_line: 3,
        end_line: 1,
    });
    assert!(result.is_err());

    // out of bounds
    let result = engine.execute_exclude(&ExcludeArgs::Range {
        start_line: 1,
        end_line: 10,
    });
    assert!(result.is_err());

    // zero start
    let result = engine.execute_exclude(&ExcludeArgs::Range {
        start_line: 0,
        end_line: 2,
    });
    assert!(result.is_err());
}

// ─── SHOW Command Tests ─────────────────────────────────────────────────────

#[test]
fn show_all_clears_all_exclusions() {
    // Validates: Requirement 3 AC 1
    let mut engine = make_engine(vec!["a", "b", "c", "d"]);
    engine.execute_exclude(&ExcludeArgs::All).unwrap();
    assert_eq!(engine.excluded_line_count(), 4);

    let result = engine.execute_show(&ShowArgs::All).unwrap();
    assert_eq!(result.lines_shown, 4);
    assert_eq!(engine.excluded_line_count(), 0);
}

#[test]
fn show_nonexcluded_is_noop() {
    // Validates: Requirement 3 AC 3
    let mut engine = make_engine(vec!["a", "b", "c"]);
    engine.execute_exclude(&ExcludeArgs::All).unwrap();

    let result = engine.execute_show(&ShowArgs::NonExcluded).unwrap();
    assert_eq!(result.lines_shown, 0);
    assert_eq!(result.message, "No excluded lines were modified");
    // Exclusion state unchanged
    assert_eq!(engine.excluded_line_count(), 3);
}

#[test]
fn show_no_match_reports_zero() {
    // Validates: Requirement 3 AC 8
    let mut engine = make_engine(vec!["alpha", "beta", "gamma"]);
    engine.execute_exclude(&ExcludeArgs::All).unwrap();

    let result = engine
        .execute_show(&ShowArgs::Text {
            pattern: "xyz".to_string(),
        })
        .unwrap();
    assert_eq!(result.lines_shown, 0);
    assert_eq!(result.message, "No excluded lines matched");
}

// ─── Line Command Tests ─────────────────────────────────────────────────────

#[test]
fn line_command_x_excludes_single_line() {
    // Validates: Requirement 5 AC 1
    let mut engine = make_engine(vec!["a", "b", "c", "d"]);

    let cmd = LineCommandExclude::Single { line: 2 };
    let result = engine.execute_line_command(&cmd).unwrap();
    assert_eq!(result.lines_affected, 1);
    assert!(!engine.is_excluded(1));
    assert!(engine.is_excluded(2));
    assert!(!engine.is_excluded(3));
}

#[test]
fn line_command_xn_excludes_consecutive_lines() {
    // Validates: Requirement 5 AC 2
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);

    let cmd = LineCommandExclude::Count { line: 1, count: 3 };
    let result = engine.execute_line_command(&cmd).unwrap();
    assert_eq!(result.lines_affected, 3);
    assert!(!engine.is_excluded(0));
    assert!(engine.is_excluded(1));
    assert!(engine.is_excluded(2));
    assert!(engine.is_excluded(3));
    assert!(!engine.is_excluded(4));
}

#[test]
fn line_command_xx_block_excludes_range() {
    // Validates: Requirement 5 AC 3
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);

    let cmd = LineCommandExclude::Block { start: 1, end: 3 };
    let result = engine.execute_line_command(&cmd).unwrap();
    assert_eq!(result.lines_affected, 3);
    assert!(!engine.is_excluded(0));
    assert!(engine.is_excluded(1));
    assert!(engine.is_excluded(2));
    assert!(engine.is_excluded(3));
    assert!(!engine.is_excluded(4));
}

#[test]
fn line_command_out_of_range_returns_error() {
    // Validates: Requirement 5
    let mut engine = make_engine(vec!["a", "b", "c"]);

    let cmd = LineCommandExclude::Single { line: 10 };
    let result = engine.execute_line_command(&cmd);
    assert!(result.is_err());
}

// ─── Block Enumeration Tests ────────────────────────────────────────────────

#[test]
fn exclusion_blocks_enumerates_contiguous_ranges() {
    // Validates: Requirement 6 AC 1, AC 5
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e", "f", "g"]);

    // Exclude lines 1-2 and 4-5 (creating two blocks)
    engine.exclude_range(1, 2);
    engine.exclude_range(4, 5);

    let blocks = engine.exclusion_blocks();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].start_line, 1);
    assert_eq!(blocks[0].end_line, 2);
    assert_eq!(blocks[1].start_line, 4);
    assert_eq!(blocks[1].end_line, 5);
}

#[test]
fn block_merging_when_adjacent_excluded() {
    // Validates: Requirement 6 AC 5
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);

    engine.exclude_range(1, 2);
    engine.exclude_line(3); // adjacent to block [1,2]

    let blocks = engine.exclusion_blocks();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].start_line, 1);
    assert_eq!(blocks[0].end_line, 3);
}

#[test]
fn block_splitting_when_middle_line_shown() {
    // Validates: Requirement 6 AC 5
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);

    engine.exclude_range(1, 3);
    assert_eq!(engine.block_count(), 1);

    // Show line 2 (middle of block)
    engine.show_line(2);

    let blocks = engine.exclusion_blocks();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].start_line, 1);
    assert_eq!(blocks[0].end_line, 1);
    assert_eq!(blocks[1].start_line, 3);
    assert_eq!(blocks[1].end_line, 3);
}

#[test]
fn block_at_doc_line_returns_full_block() {
    // Validates: Requirement 6 AC 7
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);
    engine.exclude_range(1, 3);

    let block = engine.block_at_doc_line(2).unwrap();
    assert_eq!(block.start_line, 1);
    assert_eq!(block.end_line, 3);
    assert_eq!(block.line_count(), 3);
}

#[test]
fn block_at_doc_line_none_for_visible() {
    // Validates: Requirement 6 AC 7
    let mut engine = make_engine(vec!["a", "b", "c"]);
    engine.exclude_line(1);

    assert!(engine.block_at_doc_line(0).is_none());
    assert!(engine.block_at_doc_line(2).is_none());
    assert!(engine.block_at_doc_line(1).is_some());
}

#[test]
fn no_blocks_when_no_exclusions() {
    // Validates: Requirement 6 AC 8
    let engine = make_engine(vec!["a", "b", "c"]);
    assert_eq!(engine.block_count(), 0);
    assert!(engine.exclusion_blocks().is_empty());
}

// ─── Scope Iterator Tests ───────────────────────────────────────────────────

#[test]
fn visible_lines_iter_returns_only_visible() {
    // Validates: Requirement 8 AC 5
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);
    engine.exclude_range(1, 3);

    let visible: Vec<usize> = engine.visible_lines_iter().collect();
    assert_eq!(visible, vec![0, 4]);
}

#[test]
fn excluded_lines_iter_returns_only_excluded() {
    // Validates: Requirement 8 AC 6
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);
    engine.exclude_range(1, 3);

    let excluded: Vec<usize> = engine.excluded_lines_iter().collect();
    assert_eq!(excluded, vec![1, 2, 3]);
}

// ─── Regex EXCLUDE/SHOW Tests ───────────────────────────────────────────────

#[test]
fn exclude_regex_hides_matching_lines() {
    // Validates: Requirement 2 AC 3
    let mut engine = make_engine(vec!["foo123bar", "hello", "foo456bar", "world"]);

    let args = ExcludeArgs::Regex {
        pattern: "foo.*bar".to_string(),
        scope: ExcludeScope::Visible,
    };
    let result = engine.execute_exclude(&args).unwrap();
    assert_eq!(result.lines_affected, 2);
    assert!(engine.is_excluded(0));
    assert!(!engine.is_excluded(1));
    assert!(engine.is_excluded(2));
    assert!(!engine.is_excluded(3));
}

#[test]
fn show_regex_reveals_matching_excluded_lines() {
    // Validates: Requirement 3 AC 5
    let mut engine = make_engine(vec![
        "error: bad input",
        "info: started",
        "error: timeout",
        "info: done",
    ]);
    engine.execute_exclude(&ExcludeArgs::All).unwrap();

    let result = engine
        .execute_show(&ShowArgs::Regex {
            pattern: "^error".to_string(),
        })
        .unwrap();
    assert_eq!(result.lines_shown, 2);
    assert!(!engine.is_excluded(0));
    assert!(engine.is_excluded(1));
    assert!(!engine.is_excluded(2));
    assert!(engine.is_excluded(3));
}

#[test]
fn invalid_regex_returns_error_without_modifying_state() {
    // Validates: Requirement 9 AC 8
    let mut engine = make_engine(vec!["a", "b", "c"]);

    let args = ExcludeArgs::Regex {
        pattern: "(unclosed".to_string(),
        scope: ExcludeScope::Visible,
    };
    let result = engine.execute_exclude(&args);
    assert!(result.is_err());
    assert_eq!(engine.excluded_line_count(), 0); // state unchanged
}

// ─── Performance / Large Document Tests ─────────────────────────────────────

#[test]
fn exclude_all_on_large_document() {
    // Validates: Requirement 10 AC 1
    let line_count = 100_000;
    let lines: Vec<&str> = vec!["sample line content"; line_count];
    let mut engine = make_engine(lines);

    let result = engine.execute_exclude(&ExcludeArgs::All).unwrap();
    assert_eq!(result.lines_affected, line_count);
    assert_eq!(engine.excluded_line_count(), line_count);
}

#[test]
fn show_all_on_large_document_resets_efficiently() {
    // Validates: Requirement 10 AC 2
    let line_count = 100_000;
    let lines: Vec<&str> = vec!["sample line content"; line_count];
    let mut engine = make_engine(lines);

    engine.execute_exclude(&ExcludeArgs::All).unwrap();
    let result = engine.execute_show(&ShowArgs::All).unwrap();
    assert_eq!(result.lines_shown, line_count);
    assert_eq!(engine.excluded_line_count(), 0);
}

// ─── Display-Line Integration Tests ─────────────────────────────────────────

#[test]
fn excluded_lines_reduce_display_line_count() {
    // Validates: Requirement 7 AC 1
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);
    assert_eq!(engine.display_mapping().lines_displayed(), 5);

    engine.exclude_range(1, 3);
    assert_eq!(engine.display_mapping().lines_displayed(), 2);
}

#[test]
fn show_lines_restores_display_line_count() {
    // Validates: Requirement 7 AC 2
    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);
    engine.exclude_range(1, 3);
    assert_eq!(engine.display_mapping().lines_displayed(), 2);

    engine.show_range(1, 3);
    assert_eq!(engine.display_mapping().lines_displayed(), 5);
}

#[test]
fn doc_from_display_never_returns_excluded_line() {
    // Validates: Requirement 7 AC 7

    let mut engine = make_engine(vec!["a", "b", "c", "d", "e"]);
    engine.exclude_range(1, 3); // lines 1,2,3 excluded

    let displayed = engine.display_mapping().lines_displayed();
    for d in 0..displayed {
        let pos = engine.display_mapping().doc_from_display(DisplayLine(d));
        assert!(
            !engine.is_excluded(pos.doc_line.0),
            "display line {d} mapped to excluded doc line {}",
            pos.doc_line.0
        );
    }
}
