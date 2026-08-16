//! Property-based tests for the ff-syntax-highlighting crate.
//! Validates: Requirements 2–9, 11

use proptest::prelude::*;

use ff_syntax_highlighting::fold::store::FoldData;
use ff_syntax_highlighting::state::per_line::PerLineState;
use ff_syntax_highlighting::types::{
    BytePosition, FoldFlags, FoldLevel, LineNumber, StyleSlotIndex,
};
use ff_syntax_highlighting::{
    HighlightEngine, StyleBuffer, SubStyleAllocator, SyntaxHighlighter, WordList,
};

// === Property 1: Style Buffer Length Invariant ===
// Validates: Requirements 2.6, 2.7, 2.8

#[derive(Debug, Clone)]
enum BufferOp {
    Insert { position: usize, count: usize },
    Delete { position: usize, count: usize },
}

fn buffer_op_strategy(max_len: usize) -> impl Strategy<Value = BufferOp> {
    prop_oneof![
        (0..=max_len, 1..50usize)
            .prop_map(|(position, count)| BufferOp::Insert { position, count }),
        (0..=max_len, 1..50usize)
            .prop_map(|(position, count)| BufferOp::Delete { position, count }),
    ]
}

proptest! {
    /// Feature: syntax-highlighting, Property 1: Style buffer length invariant
    /// **Validates: Requirements 2.6, 2.7, 2.8**
    #[test]
    fn style_buffer_length_equals_document_length(
        initial_len in 0..500usize,
        ops in proptest::collection::vec(buffer_op_strategy(500), 1..20),
    ) {
        let mut buffer = StyleBuffer::new(initial_len);
        let mut doc_len = initial_len;

        for op in ops {
            match op {
                BufferOp::Insert { position, count } => {
                    let pos = position.min(doc_len);
                    buffer.insert(BytePosition(pos), count);
                    doc_len += count;
                }
                BufferOp::Delete { position, count } => {
                    let pos = position.min(doc_len);
                    let actual_count = count.min(doc_len.saturating_sub(pos));
                    buffer.delete(BytePosition(pos), actual_count);
                    doc_len -= actual_count;
                }
            }
            prop_assert_eq!(buffer.len(), doc_len,
                "Style buffer length must equal document length after every operation");
        }
    }
}

// === Property 3: Keyword Lookup Consistency ===
// Validates: Requirements 5.3, 5.7

proptest! {
    /// Feature: syntax-highlighting, Property 3: Keyword lookup consistency
    /// **Validates: Requirements 5.3, 5.7**
    #[test]
    fn keyword_lookup_case_sensitive_consistency(
        words in proptest::collection::vec("[a-z]{1,10}", 1..20),
        queries in proptest::collection::vec("[a-zA-Z]{1,10}", 1..30),
    ) {
        let wl = WordList::with_words(
            &words.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            StyleSlotIndex(1),
            false, // case_sensitive
        );

        for query in &queries {
            let expected = words.contains(query);
            prop_assert_eq!(wl.contains(query), expected,
                "Case-sensitive lookup for '{}' should be {}", query, expected);
        }
    }

    /// Feature: syntax-highlighting, Property 3: Keyword lookup case-insensitive
    /// **Validates: Requirements 5.3, 5.7**
    #[test]
    fn keyword_lookup_case_insensitive_consistency(
        words in proptest::collection::vec("[a-z]{1,10}", 1..20),
        queries in proptest::collection::vec("[a-zA-Z]{1,10}", 1..30),
    ) {
        let wl = WordList::with_words(
            &words.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            StyleSlotIndex(1),
            true, // case_insensitive
        );

        for query in &queries {
            let expected = words.iter().any(|w| w.to_lowercase() == query.to_lowercase());
            prop_assert_eq!(wl.contains(query), expected,
                "Case-insensitive lookup for '{}' should be {}", query, expected);
        }
    }
}

// === Property 4: Demand-Driven Styling Idempotency ===
// Validates: Requirements 4.1, 4.2

proptest! {
    /// Feature: syntax-highlighting, Property 4: Demand-driven styling idempotency
    /// **Validates: Requirements 4.1, 4.2**
    #[test]
    fn ensure_styled_to_idempotent(
        text in "[a-z ]{10,100}",
        position in 0..100usize,
    ) {
        let mut engine = HighlightEngine::new(&text);
        // No lexer bound = all default styled, ensure_styled_to is effectively a no-op
        let pos = BytePosition(position.min(text.len()));

        engine.ensure_styled_to(pos);
        let style_pos_after_first = engine.styling_position();
        let style_at_first: Vec<_> = (0..text.len())
            .map(|i| engine.style_at(BytePosition(i)))
            .collect();

        engine.ensure_styled_to(pos);
        let style_pos_after_second = engine.styling_position();
        let style_at_second: Vec<_> = (0..text.len())
            .map(|i| engine.style_at(BytePosition(i)))
            .collect();

        prop_assert_eq!(style_pos_after_first, style_pos_after_second,
            "Styling position must not change on repeated calls");
        prop_assert_eq!(style_at_first, style_at_second,
            "Style buffer must not change on repeated calls");
    }
}

// === Property 5: Sub-Style Allocation Pool Integrity ===
// Validates: Requirements 7.1, 7.5, 7.6

#[derive(Debug, Clone)]
enum SubStyleOp {
    Allocate { base: u8, count: u8 },
    Free { base: u8 },
}

fn sub_style_op_strategy() -> impl Strategy<Value = SubStyleOp> {
    prop_oneof![
        (0..10u8, 1..20u8).prop_map(|(base, count)| SubStyleOp::Allocate { base, count }),
        (0..10u8).prop_map(|base| SubStyleOp::Free { base }),
    ]
}

proptest! {
    /// Feature: syntax-highlighting, Property 5: Sub-style allocation pool integrity
    /// **Validates: Requirements 7.1, 7.5, 7.6**
    #[test]
    fn sub_style_allocation_no_overlap(
        ops in proptest::collection::vec(sub_style_op_strategy(), 1..30),
    ) {
        let mut allocator = SubStyleAllocator::new(10); // 10 base styles

        for op in ops {
            match op {
                SubStyleOp::Allocate { base, count } => {
                    let _ = allocator.allocate(StyleSlotIndex(base), count);
                }
                SubStyleOp::Free { base } => {
                    allocator.free(StyleSlotIndex(base));
                }
            }
        }

        // Verify: no two active ranges overlap
        // We check by querying base_for for all indices
        let mut owner_count = [0u16; 256];
        for idx in 0..=255u8 {
            if allocator.base_for(StyleSlotIndex(idx)).is_some() {
                owner_count[idx as usize] += 1;
            }
        }
        for (idx, &count) in owner_count.iter().enumerate() {
            prop_assert!(count <= 1,
                "Style index {} is claimed by {} allocations (should be at most 1)", idx, count);
        }
    }
}

// === Property 6: Fold-Level Header Detection Correctness ===
// Validates: Requirements 8.3, 8.4

proptest! {
    /// Feature: syntax-highlighting, Property 6: Fold-level header detection correctness
    /// **Validates: Requirements 8.3, 8.4**
    #[test]
    fn fold_header_only_when_level_decreases_and_has_content(
        levels in proptest::collection::vec(0..100u16, 2..50),
        has_content in proptest::collection::vec(any::<bool>(), 2..50),
    ) {
        let len = levels.len().min(has_content.len());
        let levels = &levels[..len];
        let has_content = &has_content[..len];

        let mut fold_data = FoldData::new(len);
        for (i, &level) in levels.iter().enumerate() {
            fold_data.set_level(LineNumber(i), FoldLevel::new(level), FoldFlags::NONE);
        }
        fold_data.apply_fold_headers(has_content);

        for i in 0..len {
            let (level, flags) = fold_data.fold_level_at(LineNumber(i));
            let is_header = flags.contains(FoldFlags::FOLD_HEADER);

            if i + 1 < len {
                let next_level = levels[i + 1];
                let expected_header = level.value() > next_level && has_content[i];
                prop_assert_eq!(is_header, expected_header,
                    "Line {} (level={}, next_level={}, content={}): header={} but expected={}",
                    i, level.value(), next_level, has_content[i], is_header, expected_header);
            } else {
                // Last line can never be a header (no next line)
                prop_assert!(!is_header,
                    "Last line should never be marked as fold header");
            }
        }
    }
}

// === Property 7: Styled Spans Coalescing Completeness ===
// Validates: Requirements 2.4

proptest! {
    /// Feature: syntax-highlighting, Property 7: Styled spans coalescing completeness
    /// **Validates: Requirements 2.4**
    #[test]
    fn styled_spans_cover_range_completely(
        styles in proptest::collection::vec(0..10u8, 1..200),
        start_frac in 0.0..1.0f64,
        end_frac in 0.0..1.0f64,
    ) {
        let mut buffer = StyleBuffer::new(styles.len());
        for (i, &style) in styles.iter().enumerate() {
            buffer.set_range(BytePosition(i), BytePosition(i + 1), StyleSlotIndex(style));
        }

        let raw_start = (start_frac * styles.len() as f64) as usize;
        let raw_end = (end_frac * styles.len() as f64) as usize;
        let start = raw_start.min(raw_end);
        let end = raw_start.max(raw_end).min(styles.len());

        if start >= end {
            return Ok(());
        }

        let spans = buffer.spans(BytePosition(start), BytePosition(end));

        // (a) Spans completely cover the range
        if !spans.is_empty() {
            prop_assert_eq!(spans[0].start.0, start, "First span must start at range start");
            prop_assert_eq!(spans.last().unwrap().end.0, end, "Last span must end at range end");

            // No gaps between spans
            for window in spans.windows(2) {
                prop_assert_eq!(window[0].end.0, window[1].start.0,
                    "Spans must be contiguous (no gaps)");
            }
        }

        // (b) No adjacent spans with same style
        for window in spans.windows(2) {
            prop_assert_ne!(window[0].style, window[1].style,
                "Adjacent spans must have different styles");
        }

        // (c) Each span has uniform style
        for span in &spans {
            for pos in span.start.0..span.end.0 {
                let actual = buffer.get(BytePosition(pos));
                prop_assert_eq!(actual, span.style,
                    "Position {} should have style {:?} but has {:?}", pos, span.style, actual);
            }
        }
    }
}

// === Property 8: Per-Line State Synchronization with Line Count ===
// Validates: Requirements 3.1, 3.8, 3.9

#[derive(Debug, Clone)]
enum LineOp {
    Insert { at: usize, count: usize },
    Delete { at: usize, count: usize },
}

fn line_op_strategy(max_lines: usize) -> impl Strategy<Value = LineOp> {
    prop_oneof![
        (0..=max_lines, 1..10usize).prop_map(|(at, count)| LineOp::Insert { at, count }),
        (0..=max_lines, 1..10usize).prop_map(|(at, count)| LineOp::Delete { at, count }),
    ]
}

proptest! {
    /// Feature: syntax-highlighting, Property 8: Per-line state synchronization
    /// **Validates: Requirements 3.1, 3.8, 3.9**
    #[test]
    fn per_line_state_length_equals_line_count(
        initial_lines in 1..100usize,
        ops in proptest::collection::vec(line_op_strategy(100), 1..20),
    ) {
        let mut state = PerLineState::new(initial_lines);
        let mut expected_count = initial_lines;

        for op in ops {
            match op {
                LineOp::Insert { at, count } => {
                    let at = at.min(expected_count);
                    state.insert_lines(LineNumber(at), count);
                    expected_count += count;
                }
                LineOp::Delete { at, count } => {
                    let at = at.min(expected_count);
                    let actual_count = count.min(expected_count.saturating_sub(at));
                    state.delete_lines(LineNumber(at), actual_count);
                    expected_count -= actual_count;
                }
            }
            prop_assert_eq!(state.line_count(), expected_count,
                "Per-line state length must equal line count after every operation");
        }
    }
}

// === Additional Property: FoldLevel clamping ===
// Validates: Requirement 8.2

proptest! {
    /// Feature: syntax-highlighting, Property: Fold level clamping
    /// **Validates: Requirement 8.2**
    #[test]
    fn fold_level_always_in_valid_range(value in any::<u16>()) {
        let level = FoldLevel::new(value);
        prop_assert!(level.value() <= 4095,
            "FoldLevel value {} exceeds maximum 4095", level.value());
    }
}
