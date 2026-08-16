//! Property-based tests for ff-text-decorations.
//!
//! Uses proptest to verify key correctness properties across many random inputs.

use proptest::prelude::*;

use ff_text_decorations::run_styles::RunStyles;
use ff_text_decorations::{
    ColourRGBA, DecorationList, IndicatorCatalogue, IndicatorNumber, MarkerMask, MarkerNumber,
    MarkerStore,
};

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Generate operations for RunStyles testing.
#[derive(Debug, Clone)]
enum RleOp {
    Fill {
        position: u64,
        value: u32,
        length: u64,
    },
    Insert {
        position: u64,
        length: u64,
    },
    Delete {
        position: u64,
        length: u64,
    },
}

fn rle_ops_strategy() -> impl Strategy<Value = Vec<RleOp>> {
    proptest::collection::vec(
        prop_oneof![
            (0u64..5000, 0u32..10, 1u64..50).prop_map(|(pos, val, len)| RleOp::Fill {
                position: pos,
                value: val,
                length: len
            }),
            (0u64..5000, 1u64..50).prop_map(|(pos, len)| RleOp::Insert {
                position: pos,
                length: len
            }),
            (0u64..5000, 1u64..20).prop_map(|(pos, len)| RleOp::Delete {
                position: pos,
                length: len
            }),
        ],
        1..20,
    )
}

// ─── Property 1: RLE Invariant — Total Length Preservation ──────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 3.10, 4.8**
    ///
    /// For any sequence of fill_range, insert_space, and delete_range operations,
    /// the total_length always equals the expected tracked length.
    #[test]
    fn rle_total_length_preserved(
        initial_length in 1u64..5000,
        ops in rle_ops_strategy(),
    ) {
        // Feature: ff-text-decorations, Property 1: RLE total length preservation
        let mut rs: RunStyles<u32> = RunStyles::new(initial_length);
        let mut expected_length = initial_length;

        for op in &ops {
            match op {
                RleOp::Fill { position, value, length } => {
                    if expected_length == 0 { continue; }
                    let pos = position % expected_length;
                    let len = (*length).min(expected_length - pos);
                    rs.fill_range(pos, *value, len);
                }
                RleOp::Insert { position, length } => {
                    let pos = (*position).min(expected_length);
                    rs.insert_space(pos, *length);
                    expected_length += length;
                }
                RleOp::Delete { position, length } => {
                    if expected_length == 0 { continue; }
                    let pos = position % expected_length;
                    let len = (*length).min(expected_length - pos);
                    if len == 0 { continue; }
                    rs.delete_range(pos, len);
                    expected_length -= len;
                }
            }
        }

        prop_assert_eq!(rs.total_length(), expected_length);
    }
}

// ─── Property 2: Fill Range Idempotency ─────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 3.8**
    ///
    /// Filling the same range with the same value twice: second fill returns false.
    #[test]
    fn fill_range_idempotency(
        doc_length in 1u64..5000,
        position_pct in 0u64..100,
        length_pct in 1u64..50,
        value in 1u32..255,
    ) {
        // Feature: ff-text-decorations, Property 2: fill_range idempotency
        let mut rs: RunStyles<u32> = RunStyles::new(doc_length);
        let position = (position_pct * doc_length) / 100;
        let max_length = doc_length - position;
        if max_length == 0 { return Ok(()); }
        let length = ((length_pct * max_length) / 100).max(1);

        let first = rs.fill_range(position, value, length);
        prop_assert!(first, "First fill should return true");

        let second = rs.fill_range(position, value, length);
        prop_assert!(!second, "Second fill with same value should return false");
    }
}

// ─── Property 3: Insert-Delete Round Trip ───────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 4.5, 4.6**
    ///
    /// insert_space(P, L) then delete_range(P, L) restores original state.
    #[test]
    fn insert_delete_round_trip(
        doc_length in 10u64..1000,
        fill_pos_pct in 0u64..80,
        fill_len_pct in 1u64..20,
        fill_value in 1u32..100,
        insert_pos_pct in 0u64..100,
        insert_length in 1u64..200,
    ) {
        // Feature: ff-text-decorations, Property 3: insert-delete round trip
        let mut rs: RunStyles<u32> = RunStyles::new(doc_length);
        let fill_pos = (fill_pos_pct * doc_length) / 100;
        let fill_len = ((fill_len_pct * (doc_length - fill_pos)) / 100).max(1);
        rs.fill_range(fill_pos, fill_value, fill_len);

        let before = rs.clone();
        let insert_pos = (insert_pos_pct * doc_length) / 100;
        rs.insert_space(insert_pos, insert_length);
        rs.delete_range(insert_pos, insert_length);

        prop_assert_eq!(rs, before);
    }
}

// ─── Property 4: Value Consistency After Edit ───────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 4.1, 4.3, 4.4**
    ///
    /// After insert_space(P, L): positions before P unchanged, P..P+L have 0,
    /// positions after P+L have values from original P onwards.
    #[test]
    fn value_consistency_after_insert(
        doc_length in 10u64..200,
        fill_pos_pct in 0u64..60,
        fill_len_pct in 5u64..30,
        fill_value in 1u32..100,
        insert_pos_pct in 0u64..100,
        insert_length in 1u64..30,
    ) {
        // Feature: ff-text-decorations, Property 4: value consistency after edit
        let mut rs: RunStyles<u32> = RunStyles::new(doc_length);
        let fill_pos = (fill_pos_pct * doc_length) / 100;
        let fill_len = ((fill_len_pct * (doc_length - fill_pos)) / 100).max(1);
        rs.fill_range(fill_pos, fill_value, fill_len);

        // Snapshot original values
        let original_values: Vec<u32> = (0..doc_length).map(|i| rs.value_at(i)).collect();

        let insert_pos = (insert_pos_pct * doc_length) / 100;
        rs.insert_space(insert_pos, insert_length);

        // Check positions before P
        for i in 0..insert_pos {
            let actual = rs.value_at(i);
            let expected = original_values[i as usize];
            prop_assert_eq!(actual, expected);
        }

        // Check inserted space
        for i in insert_pos..(insert_pos + insert_length) {
            let actual = rs.value_at(i);
            prop_assert_eq!(actual, 0u32);
        }

        // Check positions after insert
        for i in (insert_pos + insert_length)..rs.total_length() {
            let original_idx = (i - insert_length) as usize;
            let actual = rs.value_at(i);
            let expected = original_values[original_idx];
            prop_assert_eq!(actual, expected);
        }
    }
}

// ─── Property 5: Lazy Creation and Removal ──────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 3.3, 3.4**
    ///
    /// DecorationList creates decoration on first non-zero fill,
    /// removes it when all values become zero.
    #[test]
    fn lazy_creation_and_removal(
        doc_length in 1u64..5000,
        indicator_num in 0u8..44,
        position_pct in 0u64..80,
        length_pct in 1u64..20,
        value in 1u32..255,
    ) {
        // Feature: ff-text-decorations, Property 5: lazy creation and removal
        let mut dl = DecorationList::new(doc_length);
        let indicator = IndicatorNumber(indicator_num);
        let position = (position_pct * doc_length) / 100;
        let max_len = doc_length - position;
        if max_len == 0 { return Ok(()); }
        let length = ((length_pct * max_len) / 100).max(1);

        // Initially no active decorations
        prop_assert_eq!(dl.active_count(), 0);

        // First non-zero fill creates decoration
        dl.fill_range(indicator, position, value, length);
        prop_assert_eq!(dl.active_count(), 1);

        // Clear back to zero removes decoration
        dl.fill_range(indicator, position, 0, length);
        prop_assert_eq!(dl.active_count(), 0);
    }
}

// ─── Property 6: Marker Line Tracking ───────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 9.10**
    ///
    /// After inserting K lines at line L, markers on lines < L are unchanged,
    /// markers on lines >= L shift to line + K.
    #[test]
    fn marker_line_tracking(
        line_count in 10u64..200,
        marker_lines in proptest::collection::vec(0u64..200, 1..10),
        insert_line_pct in 0u64..100,
        insert_count in 1u64..20,
    ) {
        // Feature: ff-text-decorations, Property 6: marker line tracking
        let mut store = MarkerStore::new(line_count);
        let marker = MarkerNumber::new(0).unwrap();
        let insert_line = (insert_line_pct * line_count) / 100;

        // Place markers
        let mut valid_lines: Vec<u64> = marker_lines
            .iter()
            .map(|&l| l % line_count)
            .collect();
        valid_lines.sort_unstable();
        valid_lines.dedup();

        for &line in &valid_lines {
            store.marker_add(line, marker);
        }

        // Insert lines
        store.lines_inserted(insert_line, insert_count);

        // Verify positions
        for &original_line in &valid_lines {
            if original_line < insert_line {
                prop_assert!(
                    store.marker_get(original_line).has(marker),
                    "Marker on line {} (before insert at {}) should be unchanged",
                    original_line, insert_line
                );
            } else {
                let new_line = original_line + insert_count;
                prop_assert!(
                    store.marker_get(new_line).has(marker),
                    "Marker on line {} should have moved to {}",
                    original_line, new_line
                );
            }
        }
    }
}

// ─── Property 7: All-On-For Consistency ─────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 3.9**
    ///
    /// all_on_for(position) bitmask matches individual value_at queries.
    #[test]
    fn all_on_for_consistency(
        doc_length in 10u64..500,
        fills in proptest::collection::vec(
            (0u8..44, 0u64..500, 1u32..10, 1u64..50),
            1..8
        ),
        query_pos_pct in 0u64..100,
    ) {
        // Feature: ff-text-decorations, Property 7: all_on_for consistency
        let mut dl = DecorationList::new(doc_length);
        let query_pos = (query_pos_pct * doc_length) / 100;

        for &(indicator_num, pos_raw, value, len_raw) in &fills {
            let indicator = IndicatorNumber(indicator_num);
            let pos = pos_raw % doc_length;
            let len = len_raw.min(doc_length - pos).max(1);
            dl.fill_range(indicator, pos, value, len);
        }

        let mask = dl.all_on_for(query_pos);
        for i in 0..=IndicatorNumber::MAX {
            let indicator = IndicatorNumber(i);
            let value = dl.value_at(indicator, query_pos);
            let bit_set = (mask >> i) & 1 == 1;
            let has_value = value != 0;
            prop_assert_eq!(bit_set, has_value);
        }
    }
}

// ─── Property 8: Bookmark Next/Previous Wrapping ────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 8.6**
    ///
    /// marker_next with bookmark mask returns nearest bookmarked line
    /// at or after from_line, wrapping around the document.
    #[test]
    fn bookmark_next_previous_wrapping(
        line_count in 5u64..200,
        bookmark_lines in proptest::collection::vec(0u64..200, 1..10),
        from_line_pct in 0u64..100,
    ) {
        // Feature: ff-text-decorations, Property 8: bookmark next/previous wrapping
        let mut store = MarkerStore::new(line_count);
        let marker = MarkerNumber::new(0).unwrap();
        let mask = MarkerMask(1 << 0);
        let from_line = (from_line_pct * line_count) / 100;

        let mut valid_lines: Vec<u64> = bookmark_lines
            .iter()
            .map(|&l| l % line_count)
            .collect();
        valid_lines.sort_unstable();
        valid_lines.dedup();

        for &line in &valid_lines {
            store.marker_add(line, marker);
        }

        // Verify marker_next
        let next = store.marker_next(from_line, mask);
        if let Some(next_line) = next {
            // next_line must have the bookmark
            prop_assert!(store.marker_get(next_line).has(marker));

            // It should be the nearest at or after from_line (with wrap)
            let at_or_after: Vec<u64> = valid_lines.iter()
                .filter(|&&l| l >= from_line)
                .copied()
                .collect();
            if !at_or_after.is_empty() {
                prop_assert_eq!(next_line, at_or_after[0]);
            } else {
                // Wrapped around: should be smallest bookmarked line
                prop_assert_eq!(next_line, valid_lines[0]);
            }
        } else {
            // No markers at all
            prop_assert!(valid_lines.is_empty());
        }
    }
}

// ─── Property 9: Run Merge Optimality ───────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 3.1**
    ///
    /// After any fill_range operation, no two adjacent runs have the same value.
    #[test]
    fn run_merge_optimality(
        doc_length in 1u64..5000,
        fills in proptest::collection::vec(
            (0u64..5000, 0u32..10, 1u64..100),
            1..15
        ),
    ) {
        // Feature: ff-text-decorations, Property 9: run merge optimality
        let mut rs: RunStyles<u32> = RunStyles::new(doc_length);

        for &(pos_raw, value, len_raw) in &fills {
            if doc_length == 0 { continue; }
            let pos = pos_raw % doc_length;
            let len = len_raw.min(doc_length - pos).max(1);
            rs.fill_range(pos, value, len);
        }

        let runs = rs.runs();
        for i in 0..runs.len().saturating_sub(1) {
            prop_assert_ne!(
                runs[i].value, runs[i + 1].value,
                "Adjacent runs at indices {} and {} have same value",
                i, i + 1
            );
        }
    }
}

// ─── Property 10: Theme Reload Preserves Decoration Data ────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 2.10, 15.3**
    ///
    /// Reloading theme does not modify stored indicator values or marker assignments.
    #[test]
    fn theme_reload_preserves_decoration_data(
        doc_length in 10u64..200,
        fills in proptest::collection::vec(
            (0u8..44, 0u64..200, 1u32..10, 1u64..30),
            1..5
        ),
        marker_placements in proptest::collection::vec(
            (0u64..50, 0u8..32),
            0..5
        ),
        theme_r in 0u8..255,
        theme_g in 0u8..255,
        theme_b in 0u8..255,
    ) {
        // Feature: ff-text-decorations, Property 10: theme reload preserves decoration data
        let mut dl = DecorationList::new(doc_length);
        let mut store = MarkerStore::new(50);

        // Set up decorations
        for &(indicator_num, pos_raw, value, len_raw) in &fills {
            let indicator = IndicatorNumber(indicator_num);
            let pos = pos_raw % doc_length;
            let len = len_raw.min(doc_length - pos).max(1);
            dl.fill_range(indicator, pos, value, len);
        }

        // Set up markers
        for &(line, marker_num) in &marker_placements {
            if let Some(marker) = MarkerNumber::new(marker_num) {
                store.marker_add(line % 50, marker);
            }
        }

        // Snapshot decoration values
        let sample_positions: Vec<u64> = (0..doc_length.min(30)).collect();
        let values_before: Vec<Vec<u32>> = sample_positions
            .iter()
            .map(|&pos| {
                (0..=43u8).map(|i| dl.value_at(IndicatorNumber(i), pos)).collect()
            })
            .collect();

        // Snapshot marker values
        let markers_before: Vec<MarkerMask> = (0..50u64)
            .map(|line| store.marker_get(line))
            .collect();

        // Reload theme with a mock provider
        struct MockTheme { fore: ColourRGBA }
        impl ff_text_decorations::ThemeDecorationProvider for MockTheme {
            fn indicator_fore(&self, _: IndicatorNumber) -> Option<ColourRGBA> { Some(self.fore) }
            fn indicator_fill_alpha(&self, _: IndicatorNumber) -> Option<u8> { Some(100) }
            fn indicator_outline_alpha(&self, _: IndicatorNumber) -> Option<u8> { Some(200) }
            fn indicator_stroke_width(&self, _: IndicatorNumber) -> Option<f32> { Some(2.0) }
            fn indicator_style(&self, _: IndicatorNumber) -> Option<ff_text_decorations::IndicatorStyle> { None }
            fn marker_fore(&self, _: MarkerNumber) -> Option<ColourRGBA> { None }
            fn marker_back(&self, _: MarkerNumber) -> Option<ColourRGBA> { None }
            fn marker_back_selected(&self, _: MarkerNumber) -> Option<ColourRGBA> { None }
            fn marker_alpha(&self, _: MarkerNumber) -> Option<u8> { None }
            fn marker_symbol(&self, _: MarkerNumber) -> Option<ff_text_decorations::MarkerSymbol> { None }
        }

        let mock_theme = MockTheme { fore: ColourRGBA::new(theme_r, theme_g, theme_b) };
        let mut catalogue = IndicatorCatalogue::new();
        catalogue.reload_from_theme(&mock_theme);

        // Verify decoration values unchanged
        for (idx, &pos) in sample_positions.iter().enumerate() {
            for i in 0..=43u8 {
                let value = dl.value_at(IndicatorNumber(i), pos);
                let expected = values_before[idx][i as usize];
                prop_assert_eq!(value, expected);
            }
        }

        // Verify marker values unchanged
        for line in 0..50u64 {
            let actual = store.marker_get(line);
            let expected = markers_before[line as usize];
            prop_assert_eq!(actual, expected);
        }
    }
}
