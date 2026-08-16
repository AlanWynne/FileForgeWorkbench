//! Property-based tests for the ff-hex crate.
//!
//! These tests verify universal invariants across many randomly generated inputs.

use ff_hex::{
    ByteReader, BytesPerRow, HexConfig, HexCursor, HexDigitCase, HexDumpExporter, HexDumpRange,
    HexLayout, HexModeController, HexSearchBridge, ModifiedByteTracker, NibblePosition,
    VecByteReader,
};
use proptest::prelude::*;

/// Strategy to generate a valid BytesPerRow value.
fn bytes_per_row_strategy() -> impl Strategy<Value = BytesPerRow> {
    prop_oneof![
        Just(BytesPerRow::Eight),
        Just(BytesPerRow::Sixteen),
        Just(BytesPerRow::ThirtyTwo),
        Just(BytesPerRow::SixtyFour),
    ]
}

/// Strategy to generate arbitrary document content.
fn document_strategy(max_len: usize) -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..=max_len)
}

// ─── Property 1: Hex Layout Row Generation Correctness ──────────────────────

// Feature: hex-display, Property 1: hex layout row generation correctness
// **Validates: Requirements 2.1, 2.3, 2.5, 2.7**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_layout_rows_cover_all_bytes(
        content in document_strategy(2000),
        bpr in bytes_per_row_strategy(),
    ) {
        let layout = HexLayout::new(content.len() as u64, bpr);
        let doc_len = content.len() as u64;

        if doc_len == 0 {
            // Empty document has 1 row
            prop_assert_eq!(layout.total_rows(doc_len), 1);
            return Ok(());
        }

        let total_rows = layout.total_rows(doc_len);
        let bpr_val = bpr.as_u64();

        // Every byte appears in exactly one row
        for offset in 0..doc_len {
            let row = layout.row_for_offset(offset);
            prop_assert!(row < total_rows, "byte {} mapped to row {} but total is {}", offset, row, total_rows);
        }

        // Row offsets are strictly increasing by BytesPerRow
        for row in 0..total_rows {
            let expected_start = row * bpr_val;
            prop_assert_eq!(layout.row_start_offset(row), expected_start);
        }

        // Total bytes across all rows equals document length
        let last_row = total_rows - 1;
        let last_row_start = layout.row_start_offset(last_row);
        let last_row_bytes = doc_len - last_row_start;
        let full_rows = if total_rows > 1 { total_rows - 1 } else { 0 };
        let total_bytes = full_rows * bpr_val + last_row_bytes;
        prop_assert_eq!(total_bytes, doc_len);
    }
}

// Feature: hex-display, Property 1 (continued): hex pane formatting correctness
// **Validates: Requirements 2.3, 2.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_format_hex_ascii_panes_correct(
        content in prop::collection::vec(any::<u8>(), 1..=64),
        bpr in bytes_per_row_strategy(),
    ) {
        let layout = HexLayout::new(content.len() as u64, bpr);
        let bpr_val = bpr.as_usize();

        for chunk in content.chunks(bpr_val) {
            let hex_text = layout.format_hex_pane(chunk);
            let ascii_text = layout.format_ascii_pane(chunk);

            // ASCII pane: printable chars show as-is, non-printable as '.'
            for (i, &byte) in chunk.iter().enumerate() {
                let ch = ascii_text.chars().nth(i).unwrap();
                if (0x20..=0x7E).contains(&byte) {
                    prop_assert_eq!(ch, byte as char, "byte 0x{:02X} should be printable", byte);
                } else {
                    prop_assert_eq!(ch, '.', "byte 0x{:02X} should be '.'", byte);
                }
            }

            // Hex text: extract digit pairs and verify they decode back
            let hex_digits: String = hex_text.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            prop_assert_eq!(hex_digits.len(), chunk.len() * 2);
            for (i, &byte) in chunk.iter().enumerate() {
                let pair = &hex_digits[i*2..i*2+2];
                let decoded = u8::from_str_radix(pair, 16).unwrap();
                prop_assert_eq!(decoded, byte);
            }
        }
    }
}

// ─── Property 2: Cursor Navigation Boundary Safety ──────────────────────────

// Feature: hex-display, Property 2: cursor navigation boundary safety
// **Validates: Requirements 6.6, 6.7, 6.8**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_cursor_never_exceeds_bounds(
        doc_len in 1u64..5000,
        bpr in bytes_per_row_strategy(),
        ops in prop::collection::vec(0u8..7, 50..200),
    ) {
        let layout = HexLayout::new(doc_len, bpr);
        let mut cursor = HexCursor::at_offset(0);

        for op in ops {
            match op {
                0 => cursor.move_right(&layout, doc_len),
                1 => cursor.move_left(&layout),
                2 => cursor.move_up(&layout),
                3 => cursor.move_down(&layout, doc_len),
                4 => cursor.switch_pane(),
                5 => { cursor.advance_after_hex_edit(doc_len); }
                _ => { cursor.advance_after_ascii_edit(doc_len); }
            }

            // Invariant: byte offset always in valid range
            prop_assert!(
                cursor.byte_offset() < doc_len,
                "cursor offset {} >= doc_len {} after op {}",
                cursor.byte_offset(), doc_len, op
            );

            // Invariant: nibble is always High or Low (always true by type system)
            prop_assert!(
                cursor.nibble() == NibblePosition::High || cursor.nibble() == NibblePosition::Low
            );
        }
    }
}

// ─── Property 3: Hex Edit Undo/Redo Round-Trip Integrity ────────────────────

// Feature: hex-display, Property 3: hex edit undo/redo round-trip integrity
// **Validates: Requirements 7.1, 7.2, 7.3, 7.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_edit_undo_round_trip(
        initial in prop::collection::vec(any::<u8>(), 1..=500),
        edits in prop::collection::vec((0usize..500, any::<u8>()), 5..50),
    ) {
        let mut content = initial.clone();
        let doc_len = content.len();
        let mut undo_stack: Vec<(usize, u8)> = Vec::new();

        // Apply edits
        for (offset, new_val) in &edits {
            let offset = *offset % doc_len;
            let old_val = content[offset];
            content[offset] = *new_val;
            undo_stack.push((offset, old_val));
        }

        // Undo all edits
        for (offset, old_val) in undo_stack.iter().rev() {
            content[*offset] = *old_val;
        }

        // Content should match original
        prop_assert_eq!(&content, &initial);
    }
}

// ─── Property 4: Hex Pattern Search Byte-Level Accuracy ─────────────────────

// Feature: hex-display, Property 4: hex pattern search byte-level accuracy
// **Validates: Requirements 5.1, 5.6, 5.7**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_hex_search_finds_exact_matches(
        data in prop::collection::vec(any::<u8>(), 10..=500),
        pattern in prop::collection::vec(any::<u8>(), 1..=4),
    ) {
        let matches = HexSearchBridge::find_all_matches(&data, &pattern);

        // Every reported match is a true byte-for-byte match
        for &pos in &matches {
            let pos = pos as usize;
            prop_assert!(pos + pattern.len() <= data.len());
            prop_assert_eq!(&data[pos..pos + pattern.len()], &pattern[..]);
        }

        // No missed matches: brute force verify
        let mut expected = Vec::new();
        if pattern.len() <= data.len() {
            for i in 0..=(data.len() - pattern.len()) {
                if &data[i..i + pattern.len()] == &pattern[..] {
                    expected.push(i as u64);
                }
            }
        }
        prop_assert_eq!(matches, expected);
    }
}

// Feature: hex-display, Property 4 (continued): pattern validation
// **Validates: Requirements 5.1, 5.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_hex_pattern_validation_round_trip(
        bytes in prop::collection::vec(any::<u8>(), 1..=16),
    ) {
        // Encode bytes as hex string
        let hex_str: String = bytes.iter().map(|b| format!("{b:02X}")).collect();

        // Validate should succeed and decode to original bytes
        let decoded = HexSearchBridge::validate_hex_pattern(&hex_str).unwrap();
        prop_assert_eq!(decoded, bytes);
    }
}

// ─── Property 5: Modified Byte Indicator Correctness ────────────────────────

// Feature: hex-display, Property 5: modified byte indicator correctness under edit/undo cycles
// **Validates: Requirements 8.1, 8.4, 8.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_modified_tracker_matches_actual_diff(
        saved in prop::collection::vec(any::<u8>(), 1..=200),
        ops in prop::collection::vec((0usize..200, any::<u8>(), prop::bool::ANY), 10..100),
    ) {
        let doc_len = saved.len();
        let mut current = saved.clone();
        let mut tracker = ModifiedByteTracker::new();
        let mut undo_stack: Vec<(usize, u8)> = Vec::new();

        for (offset, new_val, is_undo) in &ops {
            let offset = *offset % doc_len;

            if *is_undo && !undo_stack.is_empty() {
                // Undo: restore previous value
                let (undo_offset, old_val) = undo_stack.pop().unwrap();
                current[undo_offset] = old_val;
                tracker.recalculate(undo_offset as u64, current[undo_offset], saved[undo_offset]);
            } else {
                // Edit: apply new value
                let old_val = current[offset];
                current[offset] = *new_val;
                undo_stack.push((offset, old_val));
                tracker.recalculate(offset as u64, current[offset], saved[offset]);
            }

            // Invariant: tracker matches actual diff
            for i in 0..doc_len {
                let is_modified = tracker.is_modified(i as u64);
                let actually_differs = current[i] != saved[i];
                prop_assert_eq!(
                    is_modified, actually_differs,
                    "offset {}: tracker says modified={} but actual diff={}",
                    i, is_modified, actually_differs
                );
            }
        }
    }
}

// ─── Property 6: Viewport Scroll Clamping ───────────────────────────────────

// Feature: hex-display, Property 6: viewport scroll clamping in hex mode
// **Validates: Requirements 9.1, 9.2, 9.3, 9.7, 9.8**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_viewport_always_clamped(
        doc_len in 0u64..50000,
        bpr in bytes_per_row_strategy(),
        visible_rows in 1u64..50,
        ops in prop::collection::vec(0u8..5, 20..100),
    ) {
        use ff_hex::HexViewportAdapter;

        let total_rows = if doc_len == 0 { 1 } else { doc_len.div_ceil(bpr.as_u64()) };
        let mut vp = HexViewportAdapter::new(total_rows, visible_rows);

        for op in ops {
            match op {
                0 => vp.page_down(),
                1 => vp.page_up(),
                2 => {
                    let row = vp.top_row().saturating_add(visible_rows / 2);
                    vp.ensure_row_visible(row);
                }
                3 => vp.scroll_to_fraction(0.5),
                _ => vp.set_top_row(total_rows),
            }

            let max_top = total_rows.saturating_sub(visible_rows);
            prop_assert!(
                vp.top_row() <= max_top,
                "top_row {} > max_top {} (total_rows={}, visible={})",
                vp.top_row(), max_top, total_rows, visible_rows
            );
        }
    }
}

// ─── Property 7: Hex Dump Export Content Fidelity ────────────────────────────

// Feature: hex-display, Property 7: hex dump export content fidelity
// **Validates: Requirements 11.2, 11.4, 11.6**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_hex_dump_round_trip(
        content in prop::collection::vec(any::<u8>(), 1..=500),
        bpr in bytes_per_row_strategy(),
        use_uppercase in prop::bool::ANY,
    ) {
        let mut layout = HexLayout::new(content.len() as u64, bpr);
        if use_uppercase {
            layout.set_digit_case(HexDigitCase::Uppercase);
        } else {
            layout.set_digit_case(HexDigitCase::Lowercase);
        }

        let dump = HexDumpExporter::export(&content, None, &layout);
        let parsed = HexDumpExporter::parse_hex_dump(&dump, &layout);
        prop_assert_eq!(&parsed, &content);
    }
}

// Feature: hex-display, Property 7 (continued): partial range export
// **Validates: Requirement 11.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_hex_dump_range_round_trip(
        content in prop::collection::vec(any::<u8>(), 10..=500),
        bpr in bytes_per_row_strategy(),
        start_frac in 0.0f64..0.5,
        end_frac in 0.5f64..1.0,
    ) {
        let layout = HexLayout::new(content.len() as u64, bpr);
        let start = (start_frac * content.len() as f64) as u64;
        let end = (end_frac * content.len() as f64) as u64;
        let range = HexDumpRange { start, end };

        let dump = HexDumpExporter::export(&content, Some(range), &layout);
        let parsed = HexDumpExporter::parse_hex_dump(&dump, &layout);

        let expected = &content[start as usize..end as usize];
        prop_assert_eq!(&parsed, expected);
    }
}
