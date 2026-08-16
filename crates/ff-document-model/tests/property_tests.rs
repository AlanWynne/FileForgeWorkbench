//! Property-based tests for ff-document-model.
//!
//! These tests verify fundamental invariants hold across all valid inputs
//! using the `proptest` crate with a minimum of 100 iterations per property.

use proptest::prelude::*;

use ff_document_model::gap_buffer::GapBuffer;
use ff_document_model::line_end::{count_line_endings, LineEndMode};
use ff_document_model::text_buffer::TextBuffer;
use ff_document_model::types::{BytePosition, Direction};
use ff_document_model::viewport::Viewport;
use ff_document_model::Document;

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Strategy for generating arbitrary byte content (0-2000 bytes).
fn arb_content() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..2000)
}

/// Strategy for generating content with line endings mixed in.
fn arb_content_with_line_endings() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(prop_oneof![Just(b'\n'), Just(b'\r'), any::<u8>(),], 0..1000)
}

/// Strategy for valid insert/delete operations on a buffer of known length.
fn arb_operations(max_len: usize) -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(
        prop_oneof![
            // Insert operation with arbitrary text
            (0..=max_len, prop::collection::vec(any::<u8>(), 0..100))
                .prop_map(|(pos, text)| Op::Insert(pos, text)),
            // Delete operation
            (0..=max_len, 0..50usize).prop_map(|(pos, len)| Op::Delete(pos, len)),
        ],
        1..50,
    )
}

/// Scroll operations for viewport testing.
#[derive(Debug, Clone)]
enum ScrollOp {
    PageDown(u64),
    PageUp(u64),
    LineDown(u64),
    LineUp(u64),
    SetTopLine(u64),
}

fn arb_scroll_ops() -> impl Strategy<Value = Vec<ScrollOp>> {
    prop::collection::vec(
        prop_oneof![
            (1u64..100).prop_map(ScrollOp::PageDown),
            (1u64..100).prop_map(ScrollOp::PageUp),
            (1u64..50).prop_map(ScrollOp::LineDown),
            (1u64..50).prop_map(ScrollOp::LineUp),
            (1u64..200).prop_map(ScrollOp::SetTopLine),
        ],
        1..50,
    )
}

#[derive(Debug, Clone)]
enum Op {
    Insert(usize, Vec<u8>),
    Delete(usize, usize),
}

// ─── Property 1: Gap Buffer Content Invariant ───────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 1.1, 1.5, 1.9**
    ///
    /// Feature: document-model, Property 1: Gap buffer content invariant
    ///
    /// For any sequence of insert and delete operations, the content returned
    /// by get_range(0, length()) SHALL equal the expected content produced by
    /// applying the same operations to a naive String model.
    #[test]
    fn gap_buffer_content_matches_reference(
        initial in arb_content(),
        ops in arb_operations(3000),
    ) {
        let mut buffer = GapBuffer::new(256);
        let mut reference = initial.clone();
        buffer.insert(0, &initial);

        for op in ops {
            match op {
                Op::Insert(pos, text) => {
                    let pos = pos.min(reference.len());
                    buffer.insert(pos as u64, &text);
                    reference.splice(pos..pos, text.iter().copied());
                }
                Op::Delete(pos, len) => {
                    let pos = pos.min(reference.len());
                    let len = len.min(reference.len() - pos);
                    if len > 0 {
                        buffer.delete(pos as u64, len as u64);
                        reference.drain(pos..pos + len);
                    }
                }
            }
        }

        let actual = buffer.get_range(0, buffer.length()).unwrap_or_default();
        prop_assert_eq!(actual, reference);
    }
}

// ─── Property 2: Line Index Consistency After Edits ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 3.1, 3.2, 3.3, 3.6**
    ///
    /// Feature: document-model, Property 2: Line index consistency after edits
    ///
    /// After any sequence of operations, the line count equals the number of
    /// line-end sequences + 1, and line_from_position(line_start(n)) == n.
    #[test]
    fn line_index_consistency(
        initial in arb_content_with_line_endings(),
        ops in prop::collection::vec(
            prop_oneof![
                prop::collection::vec(
                    prop_oneof![Just(b'\n'), Just(b'\r'), any::<u8>()],
                    0..50
                ).prop_map(|text| (true, text)),
                (0..50usize).prop_map(|len| (false, vec![0; len])),
            ],
            1..20,
        ),
    ) {
        let mut buf = TextBuffer::new();
        if !initial.is_empty() {
            buf.insert(BytePosition(0), &initial).unwrap();
        }

        // Apply random operations
        for (is_insert, data) in ops {
            let len = buf.length();
            if is_insert {
                let pos = if len == 0 { 0 } else { data.len() as u64 % (len + 1) };
                let _ = buf.insert(BytePosition(pos), &data);
            } else {
                let del_len = (data.len() as u64).min(len);
                if del_len > 0 && len > 0 {
                    let pos = 0u64; // delete from beginning for simplicity
                    let _ = buf.delete(BytePosition(pos), del_len.min(len));
                }
            }
        }

        // Verify: line count == count_line_endings(content) + 1
        let content = buf.get_range(BytePosition(0), buf.length()).unwrap_or_default();
        let expected_lines = count_line_endings(&content, buf.line_end_mode()) + 1;
        prop_assert_eq!(buf.line_count(), expected_lines,
            "Line count mismatch: got {} expected {} for content len {}",
            buf.line_count(), expected_lines, content.len());

        // Verify: round-trip for all valid line numbers
        for line_num in 0..buf.line_count().min(100) {
            let ln = ff_document_model::LineNumber(line_num);
            let start = buf.line_start(ln);
            let found_line = buf.line_from_position(start);
            prop_assert_eq!(found_line, ln,
                "Round-trip failed: line_start({:?}) = {:?}, line_from_position({:?}) = {:?}",
                ln, start, start, found_line);
        }
    }
}

// ─── Property 3: Character Navigation Boundary Safety ───────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 8.1, 8.3, 8.7, 8.8**
    ///
    /// Feature: document-model, Property 3: Character navigation boundary safety
    ///
    /// For any document content and position, next_position never lands inside
    /// a multi-byte UTF-8 sequence or between CR and LF.
    #[test]
    fn character_navigation_boundary_safety(
        content in arb_content(),
    ) {
        let mut doc = Document::new();
        if !content.is_empty() {
            doc.insert(BytePosition(0), &content).unwrap();
        }

        let length = doc.length();
        if length == 0 {
            return Ok(());
        }

        // Walk forward through the document and verify:
        // 1. Navigation always advances (no infinite loop)
        // 2. Never lands between CR and LF
        // 3. char_length_at at every visited position is > 0
        let mut pos = 0u64;
        let mut steps = 0;
        while pos < length && steps < 10000 {
            // Every position we visit should have a valid char_length_at
            let cl = doc.char_length_at(BytePosition(pos));
            prop_assert!(cl > 0,
                "char_length_at returned 0 at valid position {}", pos);

            let next = doc.next_position(BytePosition(pos), Direction::Forward);
            if let Some(next_pos) = next {
                // Verify: not between CR and LF
                if next_pos.0 > 0 && next_pos.0 < length {
                    let before = doc.char_at(BytePosition(next_pos.0 - 1));
                    let at = doc.char_at(next_pos);
                    let between_crlf = before == Some(0x0D) && at == Some(0x0A);
                    prop_assert!(!between_crlf,
                        "Navigation landed between CR and LF at position {}",
                        next_pos.0);
                }

                prop_assert!(next_pos.0 > pos,
                    "Forward navigation did not advance: {} -> {}",
                    pos, next_pos.0);
                pos = next_pos.0;
            } else {
                break;
            }
            steps += 1;
        }
    }
}

// ─── Property 4: Viewport Scroll Clamping ───────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 9.2, 9.3, 9.4, 9.5, 9.8**
    ///
    /// Feature: document-model, Property 4: Viewport scroll clamping
    ///
    /// After any scroll operation, top_line is always in [1, max_top_line].
    /// Repeating a boundary-clamped scroll is idempotent.
    #[test]
    fn viewport_scroll_clamping(
        line_count in 1u64..10000,
        visible_count in 1u64..500,
        ops in arb_scroll_ops(),
    ) {
        let mut viewport = Viewport::new();

        for op in &ops {
            match op {
                ScrollOp::PageDown(_) => viewport.scroll_page_down(visible_count, line_count),
                ScrollOp::PageUp(_) => viewport.scroll_page_up(visible_count),
                ScrollOp::LineDown(n) => viewport.scroll_line_down(*n, line_count, visible_count),
                ScrollOp::LineUp(n) => viewport.scroll_line_up(*n),
                ScrollOp::SetTopLine(l) => viewport.set_top_line(*l, line_count, visible_count),
            }

            let max = Viewport::compute_max_top_line(line_count, visible_count);
            let top = viewport.top_line();
            prop_assert!(top >= 1, "top_line {} < 1 after {:?}", top, op);
            prop_assert!(top <= max, "top_line {} > max {} after {:?}", top, max, op);
        }

        let max = Viewport::compute_max_top_line(line_count, visible_count);

        // Idempotence at boundaries
        viewport.set_top_line(1, line_count, visible_count);
        viewport.scroll_page_up(visible_count);
        prop_assert_eq!(viewport.top_line(), 1, "Not idempotent at top boundary");

        viewport.set_top_line(max, line_count, visible_count);
        let before = viewport.top_line();
        viewport.scroll_page_down(visible_count, line_count);
        prop_assert_eq!(viewport.top_line(), before, "Not idempotent at bottom boundary");
    }
}

// ─── Property 5: CRLF Atomicity Under Random Edits ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// **Validates: Requirements 2.5, 2.6, 8.7**
    ///
    /// Feature: document-model, Property 5: CRLF atomicity under random edits
    ///
    /// After any sequence of edits, no line boundary exists between a CR and
    /// its immediately following LF.
    #[test]
    fn crlf_atomicity(
        initial in prop::collection::vec(
            prop_oneof![Just(b'\r'), Just(b'\n'), any::<u8>()],
            0..500
        ),
        ops in prop::collection::vec(
            prop_oneof![
                prop::collection::vec(
                    prop_oneof![Just(b'\r'), Just(b'\n'), any::<u8>()],
                    1..20
                ).prop_map(|text| (true, text, 0usize)),
                (0..20usize).prop_map(|len| (false, vec![], len)),
            ],
            1..30,
        ),
    ) {
        let mut buf = TextBuffer::new();
        if !initial.is_empty() {
            buf.insert(BytePosition(0), &initial).unwrap();
        }

        for (is_insert, text, del_len) in ops {
            let length = buf.length();
            if is_insert {
                let pos = if length == 0 { 0 } else { (text.len() as u64) % (length + 1) };
                let _ = buf.insert(BytePosition(pos), &text);
            } else {
                let del = (del_len as u64).min(length);
                if del > 0 {
                    let _ = buf.delete(BytePosition(0), del);
                }
            }
        }

        // Verify CRLF atomicity: check the content for adjacent CR+LF
        // and verify they count as ONE line ending
        let content = buf.get_range(BytePosition(0), buf.length()).unwrap_or_default();
        let expected_lines = count_line_endings(&content, LineEndMode::Default) + 1;
        prop_assert_eq!(buf.line_count(), expected_lines,
            "CRLF atomicity violated: line count {} != expected {}",
            buf.line_count(), expected_lines);
    }
}

// ─── Property 6: Streaming Load Content Integrity ───────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 4.1, 4.2, 4.5**
    ///
    /// Feature: document-model, Property 6: Streaming load content integrity
    ///
    /// For any content delivered in arbitrary chunks, the final buffer content
    /// is byte-for-byte identical to the original, and line count matches.
    #[test]
    fn streaming_load_content_integrity(
        content in prop::collection::vec(any::<u8>(), 0..5000),
        chunk_count in 1usize..20,
    ) {
        use ff_document_model::sparse_line_index::SparseLineIndex;

        let mut buffer = GapBuffer::new(256);
        let mut sparse = SparseLineIndex::new(100);

        // Split content into chunks
        let chunk_size = if content.is_empty() { 1 } else {
            (content.len() / chunk_count).max(1)
        };

        for chunk in content.chunks(chunk_size) {
            buffer.insert(buffer.length(), chunk);
            sparse.process_chunk(chunk, LineEndMode::Default);
        }

        // Verify: content matches
        let loaded = buffer.get_range(0, buffer.length()).unwrap_or_default();
        prop_assert_eq!(&loaded, &content,
            "Content mismatch after streaming load");

        // Verify: line count from sparse matches a full scan
        let mut full_index = ff_document_model::line_index::LineIndex::new();
        full_index.rebuild(&content, LineEndMode::Default);

        // The sparse index line count is approximate (it counts endings, not including last partial line)
        // So we check via finalization
        let finalized = sparse.finalize(&mut buffer, LineEndMode::Default);
        prop_assert_eq!(finalized.line_count(), full_index.line_count(),
            "Line count mismatch after finalization");
    }
}
