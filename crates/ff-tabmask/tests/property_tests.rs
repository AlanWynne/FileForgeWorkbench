//! Property-based tests for ff-tabmask.
//!
//! Tests the correctness properties specified in the design document using proptest.

use ff_tabmask::traits::{ConfigProvider, LanguageDefinitionRef};
use ff_tabmask::{
    compute_shift_left, compute_shift_right, compute_tab_action, execute_tabs_command,
    handle_reset, ArtifactKind, DefaultsLoader, DisplayArtifactManager, EditorMode, MaskLine,
    MaskManager, MaskState, TabKeyAction, TabStopList, TabStopSource, TabsMaskState, TabsState,
};
use proptest::prelude::*;

// ─── Strategies ─────────────────────────────────────────────────────────────

fn arb_columns() -> impl Strategy<Value = Vec<u32>> {
    prop::collection::vec(0u32..200, 0..20)
}

fn arb_positive_columns() -> impl Strategy<Value = Vec<u32>> {
    prop::collection::vec(1u32..200, 1..15)
}

fn arb_mask_content() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 *\t.]{0,100}"
}

fn arb_column() -> impl Strategy<Value = u32> {
    1u32..500
}

fn arb_line_width() -> impl Strategy<Value = usize> {
    1usize..200
}

// ─── Property 1: Tab Stop List Sorted and Deduplicated Invariant ────────────

proptest! {
    /// Property 1: For any input columns, the resulting TabStopList is sorted
    /// ascending with no duplicates and all values > 0.
    ///
    /// **Validates: Requirements 2.8, 4.7**
    #[test]
    fn tab_stop_list_sorted_and_deduplicated(columns in arb_columns()) {
        // Feature: tabs-and-mask, Property 1: Tab Stop List Sorted and Deduplicated Invariant
        let list = TabStopList::from_columns(columns);
        let stops = list.stops();

        // All values > 0
        for &s in stops {
            prop_assert!(s > 0, "Found zero or negative stop: {}", s);
        }

        // Sorted ascending
        for window in stops.windows(2) {
            prop_assert!(window[0] < window[1], "Not sorted: {} >= {}", window[0], window[1]);
        }

        // No duplicates (implied by strict ascending, but explicit check)
        let unique: std::collections::HashSet<u32> = stops.iter().copied().collect();
        prop_assert_eq!(unique.len(), stops.len());
    }
}

// ─── Property 2: Next Tab Stop Monotonically Advances ───────────────────────

proptest! {
    /// Property 2: For any non-empty list and current column, next_stop_after
    /// returns a value strictly greater than current_column.
    ///
    /// **Validates: Requirements 5.1**
    #[test]
    fn next_tab_stop_monotonically_advances(
        columns in arb_positive_columns(),
        current_column in arb_column(),
    ) {
        // Feature: tabs-and-mask, Property 2: Next Tab Stop Monotonically Advances
        let list = TabStopList::from_columns(columns);
        if list.is_empty() {
            return Ok(());
        }

        if let Some(next) = list.next_stop_after(current_column) {
            prop_assert!(next > current_column, "Next stop {} is not > current {}", next, current_column);
        }
    }
}

// ─── Property 3: Previous Tab Stop Monotonically Retreats ───────────────────

proptest! {
    /// Property 3: For any non-empty list and current column > 1, prev_stop_before
    /// returns a value strictly less than current_column.
    ///
    /// **Validates: Requirements 14.2, 14.3**
    #[test]
    fn prev_tab_stop_monotonically_retreats(
        columns in arb_positive_columns(),
        current_column in 2u32..500,
    ) {
        // Feature: tabs-and-mask, Property 3: Previous Tab Stop Monotonically Retreats
        let list = TabStopList::from_columns(columns);
        if list.is_empty() {
            return Ok(());
        }

        if let Some(prev) = list.prev_stop_before(current_column) {
            prop_assert!(prev < current_column, "Prev stop {} is not < current {}", prev, current_column);
        }
    }
}

// ─── Property 4: Mask Application Width Invariant ───────────────────────────

proptest! {
    /// Property 4: Applying any mask to any line_width > 0 always produces a
    /// string of exactly that width.
    ///
    /// **Validates: Requirements 9.5, 9.6**
    #[test]
    fn mask_application_width_invariant(
        content in arb_mask_content(),
        line_width in arb_line_width(),
    ) {
        // Feature: tabs-and-mask, Property 4: Mask Application Width Invariant
        let mask = MaskLine::new(content);
        let result = mask.apply_to_width(line_width);
        prop_assert_eq!(result.len(), line_width, "Result length {} != line_width {}", result.len(), line_width);
    }
}

// ─── Property 5: Tab Key Insert Mode Space Count ────────────────────────────

proptest! {
    /// Property 5: When InsertSpacesTo is returned, target_column is always
    /// strictly greater than current_column.
    ///
    /// **Validates: Requirements 5.5**
    #[test]
    fn tab_key_insert_mode_target_greater_than_current(
        columns in arb_positive_columns(),
        current_column in 1u32..200,
    ) {
        // Feature: tabs-and-mask, Property 5: Tab Key Insert Mode Space Count
        let list = TabStopList::from_columns(columns);
        let action = compute_tab_action(&list, current_column, EditorMode::Insert, false, 8, 500);

        if let TabKeyAction::InsertSpacesTo { target_column } = action {
            prop_assert!(
                target_column > current_column,
                "InsertSpacesTo target {} not > current {}",
                target_column,
                current_column
            );
        }
    }
}

// ─── Property 6: Tab Stops Persist Across RESET ─────────────────────────────

proptest! {
    /// Property 6: handle_reset removes all artifact lines but tab stop list
    /// and mask content remain unchanged.
    ///
    /// **Validates: Requirements 11.3, 11.4**
    #[test]
    fn tab_stops_persist_across_reset(
        columns in arb_positive_columns(),
        mask_content in arb_mask_content(),
    ) {
        // Feature: tabs-and-mask, Property 6: Tab Stops Persist Across RESET
        let stops = TabStopList::from_columns(columns);
        let mask = if mask_content.is_empty() {
            MaskState::empty()
        } else {
            MaskState::with_mask(MaskLine::new(&mask_content), false)
        };

        let mut state = TabsMaskState::new(
            TabsState::new(stops.clone(), TabStopSource::BuiltIn),
            mask,
        );

        // Add some artifacts
        state.add_tabs_line(ff_tabmask::ArtifactPosition { anchor_line: 5, from_line_command: false });
        state.add_mask_line(ff_tabmask::ArtifactPosition { anchor_line: 10, from_line_command: true });

        let tabs_before = state.tabs().tab_stops().clone();
        let mask_before = state.mask().mask().cloned();

        handle_reset(&mut state);

        prop_assert_eq!(state.tabs().tab_stops(), &tabs_before);
        prop_assert_eq!(state.mask().mask().cloned(), mask_before);
        prop_assert!(state.tabs_lines().is_empty());
        prop_assert!(state.mask_lines().is_empty());
    }
}

// ─── Property 7: RESET TABS Restores Defaults ───────────────────────────────

proptest! {
    /// Property 7: After any number of session overrides, reset_to_defaults
    /// restores the original default list.
    ///
    /// **Validates: Requirements 12.1**
    #[test]
    fn reset_tabs_restores_defaults(
        default_cols in arb_positive_columns(),
        override_cols in prop::collection::vec(arb_positive_columns(), 1..5),
    ) {
        // Feature: tabs-and-mask, Property 7: RESET TABS Restores Defaults
        let defaults = TabStopList::from_columns(default_cols);
        let mut tabs_state = TabsState::new(defaults.clone(), TabStopSource::LanguageDefinition);

        for override_col in override_cols {
            tabs_state.set_tab_stops(TabStopList::from_columns(override_col));
        }

        tabs_state.reset_to_defaults();
        prop_assert_eq!(tabs_state.tab_stops(), &defaults);
    }
}

// ─── Property 8: MASK OFF Clears Regardless of Source ───────────────────────

proptest! {
    /// Property 8: After clear(), mask is_active returns false and mask()
    /// returns None regardless of whether mask was from language or manual.
    ///
    /// **Validates: Requirements 7.1, 10.5**
    #[test]
    fn mask_off_clears_regardless_of_source(
        content in arb_mask_content(),
        from_language in proptest::bool::ANY,
    ) {
        // Feature: tabs-and-mask, Property 8: MASK OFF Clears Regardless of Source
        let mask = MaskLine::new(content);
        let mut mask_state = MaskState::with_mask(mask, from_language);

        mask_state.clear();
        prop_assert!(!mask_state.is_active());
        prop_assert!(mask_state.mask().is_none());
    }
}

// ─── Property 9: Tab Stop List Filters Invalid Values ───────────────────────

proptest! {
    /// Property 9: Zeros and duplicates are never present; result length equals
    /// count of distinct positive values in input.
    ///
    /// **Validates: Requirements 2.7, 2.8, 4.6**
    #[test]
    fn tab_stop_list_filters_invalid_values(columns in arb_columns()) {
        // Feature: tabs-and-mask, Property 9: Tab Stop List Filters Invalid Values
        let list = TabStopList::from_columns(columns.clone());

        let expected_count = columns
            .iter()
            .filter(|&&c| c > 0)
            .collect::<std::collections::HashSet<_>>()
            .len();

        prop_assert_eq!(list.len(), expected_count);

        for &s in list.stops() {
            prop_assert!(s > 0);
        }
    }
}

// ─── Property 10: Toggle Behaviour Idempotence ──────────────────────────────

proptest! {
    /// Property 10: Issuing TABS twice returns display to original state with
    /// no artifact lines remaining.
    ///
    /// **Validates: Requirements 1.4, 6.5**
    #[test]
    fn toggle_behaviour_idempotence(
        columns in arb_positive_columns(),
        cursor_line in 0usize..100,
    ) {
        // Feature: tabs-and-mask, Property 10: Toggle Behaviour Idempotence
        let stops = TabStopList::from_columns(columns);
        let mut state = TabsMaskState::new(
            TabsState::new(stops, TabStopSource::BuiltIn),
            MaskState::empty(),
        );

        // Initially no lines
        prop_assert!(!state.has_tabs_lines());

        // First toggle: adds lines
        execute_tabs_command(&mut state, &[], Some(cursor_line), 80).unwrap();
        prop_assert!(state.has_tabs_lines());

        // Second toggle: removes lines
        execute_tabs_command(&mut state, &[], Some(cursor_line), 80).unwrap();
        prop_assert!(!state.has_tabs_lines());
    }
}

// ─── Property 11: Shift Right Then Left Returns to Original ─────────────────

proptest! {
    /// Property 11: For columns at tab stop positions, shift_right(1) then
    /// shift_left(1) returns to the original column.
    ///
    /// **Validates: Requirements 14.1, 14.2**
    #[test]
    fn shift_right_then_left_returns_to_original(
        columns in arb_positive_columns().prop_filter("need at least 2 stops", |c| {
            let list = TabStopList::from_columns(c.clone());
            list.len() >= 2
        }),
    ) {
        // Feature: tabs-and-mask, Property 11: Shift Right Then Shift Left Returns to Original
        let list = TabStopList::from_columns(columns);
        let stops = list.stops();

        // Pick a stop that has a next stop (not the last one)
        if stops.len() < 2 {
            return Ok(());
        }

        for &column in &stops[..stops.len() - 1] {
            let right = compute_shift_right(&list, column, 1);
            let back = compute_shift_left(&list, right.target_column, 1);
            prop_assert_eq!(back.target_column, column,
                "Shift right then left from {} → {} → {} (expected {})",
                column, right.target_column, back.target_column, column
            );
        }
    }
}

// ─── Property 12: Language Definition Precedence Over Global Config ─────────

proptest! {
    /// Property 12: When both language def and global config provide stops,
    /// language def values are used.
    ///
    /// **Validates: Requirements 4.3, 4.4, 13.6**
    #[test]
    fn language_definition_precedence_over_global_config(
        global_stops in arb_positive_columns(),
        lang_stops in arb_positive_columns(),
    ) {
        // Feature: tabs-and-mask, Property 12: Language Definition Precedence Over Global Config
        struct TestConfig(Vec<u32>);
        impl ConfigProvider for TestConfig {
            fn get_tab_stops(&self) -> Vec<u32> { self.0.clone() }
            fn get_tab_size(&self) -> u32 { 8 }
        }

        let config = TestConfig(global_stops);
        // Build a TOML value with default_tab_stops
        let toml_str = format!(
            "default_tab_stops = [{}]",
            lang_stops.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
        );
        let toml_val: toml::Value = toml::from_str(&toml_str).unwrap();
        let lang_def = LanguageDefinitionRef::new(&toml_val);

        let (result, source) = DefaultsLoader::load_tab_stops(&config, Some(&lang_def), 200);

        let expected = TabStopList::from_columns(lang_stops);
        if expected.is_empty() {
            // If lang stops are all invalid/empty, config or builtin is used
            return Ok(());
        }

        prop_assert_eq!(&result, &expected);
        prop_assert_eq!(source, TabStopSource::LanguageDefinition);
    }
}

// ─── Property 13: Display Artifact Lines Excluded from Command Scope ────────

proptest! {
    /// Property 13: TABS_Lines and MASK_Lines are never real document lines.
    ///
    /// **Validates: Requirements 18.1, 18.2, 18.3, 18.4**
    #[test]
    fn display_artifact_lines_excluded_from_command_scope(
        kind in prop_oneof![Just(ArtifactKind::TabsLine), Just(ArtifactKind::MaskLine)],
    ) {
        // Feature: tabs-and-mask, Property 13: Display Artifact Lines Excluded from Command Scope
        let meta = DisplayArtifactManager::artifact_metadata(kind);
        prop_assert!(!meta.is_real_document_line);
        prop_assert_eq!(meta.category, "display");
    }
}

// ─── Property 14: Mask Application Part of Insert Transaction ────────────────

proptest! {
    /// Property 14: apply_mask returns content for embedding in insertion,
    /// never creates a separate transaction. (Structural property — verified
    /// by checking that apply_mask returns Option<String> and not a transaction.)
    ///
    /// **Validates: Requirements 9.4**
    #[test]
    fn mask_application_part_of_insert_transaction(
        content in arb_mask_content(),
        line_width in arb_line_width(),
    ) {
        // Feature: tabs-and-mask, Property 14: Mask Application Part of Insert Transaction
        let state = MaskState::with_mask(MaskLine::new(&content), false);
        let result = MaskManager::apply_mask(&state, line_width);

        // The result is just a String (content), not a transaction object.
        // This verifies the structural property that mask content is returned
        // for embedding in the line insertion path.
        prop_assert!(result.is_some());
        prop_assert_eq!(result.unwrap().len(), line_width);
    }
}
