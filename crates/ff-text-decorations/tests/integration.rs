//! Integration tests for ff-text-decorations.
//!
//! These tests verify end-to-end scenarios with multiple producers,
//! edit synchronization, and complete lifecycle operations.

use ff_text_decorations::constants::{indicators, markers};
use ff_text_decorations::{
    ColourRGBA, DecorationList, EditSync, IndicatorAllocator, IndicatorCatalogue, IndicatorNumber,
    MarkerMask, MarkerNumber, MarkerStore,
};

// ─── Integration Test 20.1: Multi-Producer Scenario ─────────────────────────

#[test]
fn multi_producer_search_and_diagnostics_coexist() {
    // Validates: Multi-producer scenario (search highlights + diagnostic underlines)
    let mut dl = DecorationList::new(1000);

    // Simulate search engine highlighting matches at positions 100, 300, 500
    dl.fill_range(indicators::SEARCH_ALL, 100, 1, 10);
    dl.fill_range(indicators::SEARCH_ALL, 300, 1, 10);
    dl.fill_range(indicators::SEARCH_ALL, 500, 1, 10);
    // Current match at 300
    dl.fill_range(indicators::SEARCH_CURRENT, 300, 1, 10);

    // Simulate diagnostic producer adding error underline at 150..170
    dl.fill_range(indicators::ERROR, 150, 1, 20);
    // Warning at 400..420
    dl.fill_range(indicators::WARNING, 400, 1, 20);

    // Verify all coexist without interference
    assert_eq!(dl.value_at(indicators::SEARCH_ALL, 105), 1);
    assert_eq!(dl.value_at(indicators::SEARCH_CURRENT, 305), 1);
    assert_eq!(dl.value_at(indicators::ERROR, 155), 1);
    assert_eq!(dl.value_at(indicators::WARNING, 410), 1);

    // Verify all_on_for at overlap-free position
    let mask_at_105 = dl.all_on_for(105);
    assert!(mask_at_105 & (1u64 << indicators::SEARCH_ALL.0) != 0);
    assert!(mask_at_105 & (1u64 << indicators::ERROR.0) == 0);

    // At position 305, both SEARCH_ALL and SEARCH_CURRENT are active
    let mask_at_305 = dl.all_on_for(305);
    assert!(mask_at_305 & (1u64 << indicators::SEARCH_ALL.0) != 0);
    assert!(mask_at_305 & (1u64 << indicators::SEARCH_CURRENT.0) != 0);

    // Five active decorations total
    assert_eq!(dl.active_count(), 4); // SEARCH_ALL, SEARCH_CURRENT, ERROR, WARNING
}

// ─── Integration Test 20.2: Edit Synchronization ────────────────────────────

#[test]
fn edit_synchronization_with_multiple_indicators() {
    // Validates: insert/delete text with active decorations from multiple indicators
    let mut dl = DecorationList::new(200);
    let mut ms = MarkerStore::new(20);

    // Set up decorations
    dl.fill_range(indicators::SEARCH_ALL, 50, 1, 10); // [50..60)
    dl.fill_range(indicators::ERROR, 80, 1, 20); // [80..100)

    // Set up markers
    ms.marker_add(5, markers::BOOKMARK);
    ms.marker_add(10, markers::HISTORY_MODIFIED);

    // Insert text at position 70 (between search and error decorations)
    EditSync::handle_insert(&mut dl, &mut ms, 70, 30, 2, 7);

    // Search decoration should be unchanged (before insertion)
    assert_eq!(dl.value_at(indicators::SEARCH_ALL, 55), 1);
    // Error decoration should have shifted right by 30
    assert_eq!(dl.value_at(indicators::ERROR, 110), 1); // was at 80, now at 110
    assert_eq!(dl.value_at(indicators::ERROR, 80), 0); // old position cleared

    // Marker on line 5 unchanged (before line 8 insertion)
    assert!(ms.marker_get(5).has(markers::BOOKMARK));
    // Marker on line 10 shifted to 12 (line_of_insert=7, +1=8, 2 lines added)
    assert!(ms.marker_get(12).has(markers::HISTORY_MODIFIED));
}

// ─── Integration Test 20.3: Undo/Redo Cycle ─────────────────────────────────

#[test]
fn undo_redo_cycle_preserves_decorations() {
    // Validates: decorations track positions through undo and redo
    let mut dl = DecorationList::new(100);
    let mut ms = MarkerStore::new(10);

    // Set up initial state
    dl.fill_range(indicators::ERROR, 20, 1, 10); // [20..30)

    // Simulate insert at 25 with length 5
    let before_values: Vec<u32> = (0..100)
        .map(|i| dl.value_at(indicators::ERROR, i))
        .collect();
    EditSync::handle_insert(&mut dl, &mut ms, 25, 5, 0, 2);

    // Error now split: [20..25) = 1, [25..30) = 0 (inserted), [30..35) = 1
    assert_eq!(dl.value_at(indicators::ERROR, 22), 1);
    assert_eq!(dl.value_at(indicators::ERROR, 27), 0);
    assert_eq!(dl.value_at(indicators::ERROR, 32), 1);

    // Undo the insert (delete the 5 chars at 25)
    EditSync::handle_undo_insert(&mut dl, &mut ms, 25, 5, 0, 2);

    // Should restore original state
    for i in 0..100 {
        assert_eq!(
            dl.value_at(indicators::ERROR, i),
            before_values[i as usize],
            "Mismatch at position {}",
            i
        );
    }
}

// ─── Integration Test 20.4: Theme Hot-Reload ────────────────────────────────

#[test]
fn theme_hot_reload_preserves_data_updates_visuals() {
    // Validates: theme change updates visual properties, data unchanged
    let mut dl = DecorationList::new(100);
    dl.fill_range(indicators::ERROR, 10, 1, 10);
    dl.fill_range(indicators::SEARCH_ALL, 50, 1, 5);

    let mut catalogue = IndicatorCatalogue::new();

    // Verify initial error indicator is red squiggle
    let error_config = catalogue.get(indicators::ERROR);
    assert_eq!(error_config.normal.fore, ColourRGBA::new(255, 0, 0));

    // Apply new theme with different colours
    struct DarkTheme;
    impl ff_text_decorations::ThemeDecorationProvider for DarkTheme {
        fn indicator_fore(&self, ind: IndicatorNumber) -> Option<ColourRGBA> {
            if ind == indicators::ERROR {
                Some(ColourRGBA::new(255, 100, 100)) // lighter red for dark theme
            } else {
                None
            }
        }
        fn indicator_fill_alpha(&self, _: IndicatorNumber) -> Option<u8> {
            None
        }
        fn indicator_outline_alpha(&self, _: IndicatorNumber) -> Option<u8> {
            None
        }
        fn indicator_stroke_width(&self, _: IndicatorNumber) -> Option<f32> {
            None
        }
        fn indicator_style(
            &self,
            _: IndicatorNumber,
        ) -> Option<ff_text_decorations::IndicatorStyle> {
            None
        }
        fn marker_fore(&self, _: MarkerNumber) -> Option<ColourRGBA> {
            None
        }
        fn marker_back(&self, _: MarkerNumber) -> Option<ColourRGBA> {
            None
        }
        fn marker_back_selected(&self, _: MarkerNumber) -> Option<ColourRGBA> {
            None
        }
        fn marker_alpha(&self, _: MarkerNumber) -> Option<u8> {
            None
        }
        fn marker_symbol(&self, _: MarkerNumber) -> Option<ff_text_decorations::MarkerSymbol> {
            None
        }
    }

    catalogue.reload_from_theme(&DarkTheme);

    // Visual properties changed
    let error_config = catalogue.get(indicators::ERROR);
    assert_eq!(error_config.normal.fore, ColourRGBA::new(255, 100, 100));

    // Decoration data unchanged
    assert_eq!(dl.value_at(indicators::ERROR, 15), 1);
    assert_eq!(dl.value_at(indicators::SEARCH_ALL, 52), 1);
}

// ─── Integration Test 20.5: Bookmark Lifecycle ──────────────────────────────

#[test]
fn bookmark_lifecycle_toggle_navigate_insert_clear() {
    // Validates: bookmark toggle, navigate, insert lines, verify movement, clear
    let mut store = MarkerStore::new(100);
    let bookmark = markers::BOOKMARK;
    let mask = MarkerMask(1 << bookmark.0);

    // Toggle bookmarks on lines 10, 30, 60
    store.marker_add(10, bookmark);
    store.marker_add(30, bookmark);
    store.marker_add(60, bookmark);
    assert_eq!(store.all_lines_with_marker(bookmark), vec![10, 30, 60]);

    // Navigate next from line 20 → should find 30
    assert_eq!(store.marker_next(20, mask), Some(30));
    // Navigate previous from line 50 → should find 30
    assert_eq!(store.marker_previous(50, mask), Some(30));
    // Navigate next from line 70 → should wrap to 10
    assert_eq!(store.marker_next(70, mask), Some(10));

    // Insert 5 lines at line 25 → bookmarks on 30 and 60 shift
    store.lines_inserted(25, 5);
    assert_eq!(store.all_lines_with_marker(bookmark), vec![10, 35, 65]);

    // Toggle off bookmark at 35 (was 30)
    store.marker_delete(35, bookmark);
    assert_eq!(store.all_lines_with_marker(bookmark), vec![10, 65]);

    // Clear all
    store.marker_delete_all(bookmark);
    assert!(store.all_lines_with_marker(bookmark).is_empty());
}

// ─── Integration Test 20.6: Change History Lifecycle ────────────────────────

#[test]
fn change_history_lifecycle_edit_save_undo() {
    // Validates: edit sets Modified, save transitions to Saved, undo to Reverted
    let mut store = MarkerStore::new(100);

    // User edits line 5 → Modified marker set
    store.marker_add(5, markers::HISTORY_MODIFIED);
    assert!(store.marker_get(5).has(markers::HISTORY_MODIFIED));

    // User edits line 10 → Modified marker set
    store.marker_add(10, markers::HISTORY_MODIFIED);

    // Save: Modified → Saved
    let modified_lines = store.all_lines_with_marker(markers::HISTORY_MODIFIED);
    for &line in &modified_lines {
        store.marker_delete(line, markers::HISTORY_MODIFIED);
        store.marker_add(line, markers::HISTORY_SAVED);
    }
    assert!(!store.marker_get(5).has(markers::HISTORY_MODIFIED));
    assert!(store.marker_get(5).has(markers::HISTORY_SAVED));

    // Undo reverts line 10 to original → RevertedToOrigin
    store.marker_delete(10, markers::HISTORY_SAVED);
    store.marker_add(10, markers::HISTORY_REVERTED_ORIGIN);
    assert!(store.marker_get(10).has(markers::HISTORY_REVERTED_ORIGIN));
    assert!(!store.marker_get(10).has(markers::HISTORY_SAVED));
}

// ─── Integration Test 20.7: Indicator Allocation ────────────────────────────

#[test]
fn indicator_allocation_lifecycle() {
    // Validates: allocate multiple plugin indicators, exhaust range, verify error
    let mut alloc = IndicatorAllocator::new();

    // Allocate indicators for multiple plugins
    let spell_check = alloc.allocate("spell-check").unwrap();
    let coverage = alloc.allocate("code-coverage").unwrap();
    let lint = alloc.allocate("linter").unwrap();

    assert_eq!(spell_check.0, 8);
    assert_eq!(coverage.0, 9);
    assert_eq!(lint.0, 10);

    // Release one
    alloc.release(coverage).unwrap();

    // Next allocation reuses the freed slot
    let new_plugin = alloc.allocate("new-plugin").unwrap();
    assert_eq!(new_plugin.0, 9);

    // Exhaust the rest of the container range
    // We have spell_check(8), new_plugin(9), lint(10) = 3 allocated
    // Need to fill remaining 21 slots (11–31)
    for i in 0..21 {
        alloc.allocate(&format!("plugin-{}", i)).unwrap();
    }

    // Now all 24 slots (8–31) should be full
    let result = alloc.allocate("one-too-many");
    assert!(result.is_err());

    // Verify range predicates
    assert!(IndicatorAllocator::is_container_range(spell_check));
    assert!(!IndicatorAllocator::is_lexer_range(spell_check));
    assert!(!IndicatorAllocator::is_ime_range(spell_check));
}
