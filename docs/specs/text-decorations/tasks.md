# Implementation Plan: Text Decorations (`ff-text-decorations`)

## Overview

This plan covers the complete implementation of the `ff-text-decorations` crate — the visual overlay subsystem for FileForgeWorkbench. The crate manages transient, overlapping decorations applied on top of (or beneath) rendered text to communicate semantic information such as search matches, diagnostic errors, change history, bookmarks, and custom plugin indicators.

This is a **Wave 6 (UI and Rendering)** sub-project. It depends on `ff-document-model` (Wave 4) for buffer positions and edit events, `ff-theme` (Wave 6 peer) for colour/style configuration, `ff-command` (Wave 2) for bookmark command registration, and `ff-configuration-system` (Wave 2) for hot-reload integration.

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-text-decorations/Cargo.toml` with dependencies (thiserror, serde, proptest dev-dep) and deps on `ff-logging`, `ff-document-model`, `ff-command`, `ff-configuration-system`
  - [ ] 1.2 Create `crates/ff-text-decorations/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `indicator_style.rs`, `indicator.rs`, `catalogue.rs`, `allocator.rs`, `run_styles.rs`, `decoration.rs`, `decoration_list.rs`, `marker_symbol.rs`, `line_marker.rs`, `marker_store.rs`, `edit_sync.rs`, `hover.rs`, `theme_integration.rs`, `rendering.rs`, `dpi.rs`, `commands.rs`, `constants.rs`, `error.rs`, `events.rs`
  - [ ] 1.4 Add `ff-text-decorations` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. Core newtypes, enums, and error types
  - [ ] 2.1 Define `IndicatorNumber(u8)` newtype with `MAX = 43`, `new()` constructor returning `Option<Self>`
  - [ ] 2.2 Define `MarkerNumber(u8)` newtype with `MAX = 31`, `new()` constructor returning `Option<Self>`
  - [ ] 2.3 Define `MarkerMask(u32)` with `has()`, `set()`, `clear()`, `is_empty()` methods
  - [ ] 2.4 Define `ColourRGBA { r, g, b, a }` struct with constructors and Display impl
  - [ ] 2.5 Define `DecorationError` enum with all variants: PositionOutOfRange, InvalidIndicatorNumber, InvalidMarkerNumber, LexerRangeViolation, NoAvailableIndicators, NotAllocated, LineOutOfRange, InvalidThemeValue
  - [ ] 2.6 Write unit tests for newtype construction (valid/invalid), marker mask operations, error formatting
  - Covers: Requirement 9 AC 1, Requirement 13 AC 1–2, Cross-cutting Req 8

- [ ] 3. Indicator style catalogue
  - [ ] 3.1 Define `IndicatorStyle` enum with all 23 variants: Plain, Squiggle, TT, Diagonal, Strike, Hidden, Box, RoundBox, StraightBox, Dash, Dots, SquiggleLow, DotBox, SquigglePixmap, CompositionThick, CompositionThin, FullBox, TextFore, Point, PointCharacter, Gradient, GradientCentre, PointTop
  - [ ] 3.2 Define `IndicatorFlags` struct with `value_fore: bool` field
  - [ ] 3.3 Define `StyleAppearance` struct with `style: IndicatorStyle` and `fore: ColourRGBA`
  - [ ] 3.4 Define `IndicatorConfig` struct with fields: normal, hover (StyleAppearance), under, fill_alpha, outline_alpha, stroke_width, flags
  - [ ] 3.5 Implement `IndicatorConfig::is_dynamic()` predicate (returns true when hover differs from normal)
  - [ ] 3.6 Write unit tests for IndicatorStyle completeness, is_dynamic predicate, default config values
  - Covers: Requirement 1 AC 1, Requirement 2 AC 1–7

- [ ] 4. Indicator properties and configuration
  - [ ] 4.1 Implement default `IndicatorConfig` with fore colour, under=false, fill_alpha=30, outline_alpha=50, stroke_width=1.0
  - [ ] 4.2 Implement ValueFore mode: when `flags.value_fore` is true, colour derived from lower 24 bits of indicator value as RGB
  - [ ] 4.3 Define well-known indicator constants module: SEARCH_CURRENT(8), SEARCH_ALL(9), ERROR(10), WARNING(11), INFO(12), HINT(13), IME_INPUT(32–35), HISTORY_*(36–43)
  - [ ] 4.4 Write unit tests for ValueFore colour extraction, default property values, constant definitions
  - Covers: Requirement 2 AC 8–9, Requirement 13 AC 3

- [ ] 5. Run-length encoded storage (`RunStyles<T>`)
  - [ ] 5.1 Define `Run<T> { value: T, length: u64 }` struct
  - [ ] 5.2 Implement `RunStyles<T>` struct with `runs: Vec<Run<T>>`, `cumulative: Vec<u64>`, `total_length: u64`
  - [ ] 5.3 Implement `RunStyles::new(initial_length)` creating a single run of T::default() with the given length
  - [ ] 5.4 Implement `value_at(position)` with O(log n) binary search on cumulative lengths
  - [ ] 5.5 Implement `run_start(position)` and `run_end(position)` returning run boundaries
  - [ ] 5.6 Implement `fill_range(position, value, length)` with run splitting, merging adjacent same-value runs, returning whether any values changed
  - [ ] 5.7 Implement `insert_space(position, length)` splitting the run at position and inserting a default-value run
  - [ ] 5.8 Implement `delete_range(position, length)` removing positions and merging boundary runs
  - [ ] 5.9 Implement `is_empty()` (all values are T::default()) and `total_length()` accessors
  - [ ] 5.10 Implement `runs_in_range(start, end)` returning iterator of (position, &Run<T>) tuples intersecting the range
  - [ ] 5.11 Write unit tests for new, value_at, fill_range, insert_space, delete_range, run merge correctness
  - Covers: Requirement 3 AC 1, 10; Requirement 4 AC 1–4, 7–8

- [ ] 6. Decoration and DecorationList
  - [ ] 6.1 Implement `DecorationList` struct with `HashMap<IndicatorNumber, RunStyles<u32>>` and `document_length`
  - [ ] 6.2 Implement `DecorationList::new(document_length)` constructor
  - [ ] 6.3 Implement `value_at(indicator, position)` returning 0 for non-existent decorations
  - [ ] 6.4 Implement `start_run(indicator, position)` and `end_run(indicator, position)` delegating to RunStyles
  - [ ] 6.5 Implement `fill_range(indicator, position, value, length)` with lazy creation on first non-zero write and removal when all values become zero
  - [ ] 6.6 Implement `all_on_for(position)` returning bitmask of all indicators with non-zero value at position
  - [ ] 6.7 Implement `insert_space(position, length)` propagating to all active decorations
  - [ ] 6.8 Implement `delete_range(position, length)` propagating to all active decorations
  - [ ] 6.9 Implement `delete_lexer_decorations()` clearing all indicator values in the lexer range (0–7)
  - [ ] 6.10 Implement `indicators_in_range(start, end)` returning Vec of (indicator_number, start, end, value) tuples
  - [ ] 6.11 Implement `active_count()` returning number of non-empty decorations
  - [ ] 6.12 Write unit tests for lazy creation/removal, value_at, all_on_for, fill_range edge cases
  - Covers: Requirement 3 AC 2–9; Requirement 4 AC 1–2; Requirement 13 AC 7; Requirement 14 AC 2

- [ ] 7. Marker symbol and line marker configuration
  - [ ] 7.1 Define `MarkerSymbol` enum with all 31 geometric variants plus `Pixmap(PixmapId)` custom variant
  - [ ] 7.2 Define `PixmapId(u32)` opaque identifier for custom pixmap markers
  - [ ] 7.3 Define `MarkerLayer` enum: Base, Overlay
  - [ ] 7.4 Define `LineMarkerConfig` struct: symbol, fore, back, back_selected, alpha, layer, stroke_width
  - [ ] 7.5 Define well-known marker constants: BOOKMARK(0), HISTORY_MODIFIED(1), HISTORY_SAVED(2), HISTORY_REVERTED_ORIGIN(3), HISTORY_REVERTED_MODIFIED(4)
  - [ ] 7.6 Write unit tests for MarkerSymbol completeness, LineMarkerConfig defaults, constant correctness
  - Covers: Requirement 9 AC 2–6; Requirement 7 AC 1; Requirement 8 AC 1

- [ ] 8. MarkerStore (per-line marker bitmask storage)
  - [ ] 8.1 Implement `MarkerStore` struct with `BTreeMap<u64, MarkerMask>` and `line_count`
  - [ ] 8.2 Implement `marker_add(line, marker_number)` adding marker bit to line's mask
  - [ ] 8.3 Implement `marker_delete(line, marker_number)` clearing marker bit, removing entry if mask becomes empty
  - [ ] 8.4 Implement `marker_delete_all(marker_number)` removing a specific marker from all lines
  - [ ] 8.5 Implement `marker_get(line)` returning MarkerMask for the line (empty if no markers)
  - [ ] 8.6 Implement `marker_next(from_line, mask)` finding next line at or after from_line with any marker in mask
  - [ ] 8.7 Implement `marker_previous(from_line, mask)` finding previous line at or before from_line with any marker in mask
  - [ ] 8.8 Implement `lines_inserted(from_line, count)` shifting markers on lines >= from_line upward by count
  - [ ] 8.9 Implement `lines_deleted(from_line, count)` removing markers on deleted lines and shifting remaining downward
  - [ ] 8.10 Implement `all_lines_with_marker(marker_number)` returning Vec of all lines with the specified marker
  - [ ] 8.11 Implement `clear_all()` removing all markers from all lines
  - [ ] 8.12 Write unit tests for add/delete, navigation, line insertion/deletion, clear_all
  - Covers: Requirement 9 AC 7–10; Requirement 8 AC 4–7, 10

- [ ] 9. Indicator allocator and namespace management
  - [ ] 9.1 Implement `IndicatorAllocator` struct with allocated flags and owner tracking for container range (8–31)
  - [ ] 9.2 Implement `allocate(plugin_id)` returning next available IndicatorNumber from container range, or error if exhausted
  - [ ] 9.3 Implement `release(indicator)` freeing a previously allocated indicator number
  - [ ] 9.4 Implement range predicates: `is_lexer_range(0–7)`, `is_container_range(8–31)`, `is_ime_range(32–35)`, `is_history_range(36–43)`
  - [ ] 9.5 Write unit tests for allocation/release lifecycle, range boundary checks, exhaustion error
  - Covers: Requirement 13 AC 1, 4–6

- [ ] 10. Edit synchronization
  - [ ] 10.1 Implement `EditSync` module receiving edit events (insert at position P with length L, delete at position P with length L)
  - [ ] 10.2 Implement insert_space propagation: call `DecorationList::insert_space(P, L)` and `MarkerStore::lines_inserted` for line-level markers
  - [ ] 10.3 Implement delete_range propagation: call `DecorationList::delete_range(P, L)` and `MarkerStore::lines_deleted` for line-level markers
  - [ ] 10.4 Implement undo integration: when undo reverses an insertion, a matching delete_range is applied; when undo reverses a deletion, a matching insert_space is applied
  - [ ] 10.5 Write unit tests for insert/delete propagation, undo round-trip, multi-indicator consistency
  - Covers: Requirement 4 AC 1–8

- [ ] 11. IndicatorCatalogue and theme integration
  - [ ] 11.1 Implement `IndicatorCatalogue` struct with 44-slot array of `IndicatorConfig`
  - [ ] 11.2 Implement `new()` with compiled default configurations for all well-known indicators (search=StraightBox yellow, error=Squiggle red, warning=Squiggle amber, info=Plain blue, hint=Dots grey)
  - [ ] 11.3 Implement `get(indicator)` and `set(indicator, config)` accessors
  - [ ] 11.4 Implement `is_dynamic(indicator)` predicate delegating to IndicatorConfig
  - [ ] 11.5 Define `ThemeDecorationProvider` trait with methods: indicator_fore, indicator_fill_alpha, indicator_outline_alpha, indicator_stroke_width, indicator_style, marker_fore, marker_back, marker_back_selected, marker_alpha, marker_symbol
  - [ ] 11.6 Implement `reload_from_theme(&mut self, theme: &dyn ThemeDecorationProvider)` applying theme overrides to catalogue, validating values (alpha 0–255, stroke_width 0.5–10.0), falling back to defaults for invalid entries with warning log
  - [ ] 11.7 Write unit tests for default catalogue correctness, theme reload, invalid value handling/fallback
  - Covers: Requirement 2 AC 9–10; Requirement 15 AC 1–8

- [ ] 12. Hover state and decoration events
  - [ ] 12.1 Implement `HoverState` struct tracking current_position, previous_position, click_notified
  - [ ] 12.2 Implement `update_position(position, decoration_list, catalogue)` returning true when redraw needed (dynamic indicators changed)
  - [ ] 12.3 Implement `notify_click()` and `reset_click()` for click tracking
  - [ ] 12.4 Define `DecorationEvent` enum: Click { position, indicators }, HoverEnter { position, indicator }, HoverLeave { position, indicator }
  - [ ] 12.5 Define `DecorationEventListener` trait with `on_decoration_event(&self, event: &DecorationEvent)` method
  - [ ] 12.6 Implement hover-to-event logic: emit HoverEnter when moving into a dynamic indicator range, HoverLeave when leaving
  - [ ] 12.7 Implement click event emission: emit Click with all active indicator numbers at the clicked position
  - [ ] 12.8 Write unit tests for hover transitions, click notification, event emission, non-dynamic indicator no-redraw
  - Covers: Requirement 11 AC 1–7

- [ ] 13. Search match highlighting support
  - [ ] 13.1 Define search indicator defaults: SEARCH_CURRENT = StraightBox bright yellow/orange, SEARCH_ALL = RoundBox pale yellow
  - [ ] 13.2 Document the producer contract: fill_range(SEARCH_ALL, ...) for all matches, fill_range(SEARCH_CURRENT, ...) for focused match
  - [ ] 13.3 Implement search decoration clear helper: reset all values for search indicators to zero
  - [ ] 13.4 Implement theme-driven search highlight colours with distinct defaults for light, dark, high-contrast modes
  - [ ] 13.5 Write unit tests for search indicator fill/clear, current-match move (clear old, set new), incremental update
  - Covers: Requirement 5 AC 1–10

- [ ] 14. Diagnostic underline support
  - [ ] 14.1 Define diagnostic indicator defaults: ERROR = Squiggle red, WARNING = Squiggle amber, INFO = Plain blue, HINT = Dots grey
  - [ ] 14.2 Set diagnostic indicators to `under = true` by default (render below text)
  - [ ] 14.3 Document the producer contract: fill_range with appropriate severity indicator for diagnostic ranges
  - [ ] 14.4 Write unit tests for diagnostic indicator defaults, under property, fill/clear lifecycle
  - Covers: Requirement 6 AC 1–10

- [ ] 15. Change history markers and indicators
  - [ ] 15.1 Define `ChangeHistoryState` enum: Modified, Saved, RevertedToOrigin, RevertedToModified
  - [ ] 15.2 Define `ChangeType` enum: Insertion, Deletion
  - [ ] 15.3 Implement change history line marker transitions: edit sets Modified, save transitions Modified→Saved, undo transitions to RevertedToOrigin or RevertedToModified
  - [ ] 15.4 Implement character-level change history indicators using dedicated indicator numbers (36–43) for each combination of state × change type
  - [ ] 15.5 Define default marker colours: Modified=orange, Saved=green, RevertedToOrigin=blue, RevertedToModified=yellow
  - [ ] 15.6 Write unit tests for state transitions (edit, save, undo), character-level indicator fill, colour defaults
  - Covers: Requirement 7 AC 1–11; Requirement 12 AC 1–7

- [ ] 16. Bookmark operations
  - [ ] 16.1 Implement bookmark toggle: add BOOKMARK marker if not present, remove if present
  - [ ] 16.2 Implement `next_bookmark(from_line)` and `previous_bookmark(from_line)` with document wrapping
  - [ ] 16.3 Implement `clear_all_bookmarks()` removing all bookmark markers
  - [ ] 16.4 Implement `all_bookmarked_lines()` returning sorted list of bookmarked line numbers
  - [ ] 16.5 Register bookmark commands with command-framework: `decorations.bookmark.toggle`, `decorations.bookmark.next`, `decorations.bookmark.previous`, `decorations.bookmark.clear_all`
  - [ ] 16.6 Write unit tests for toggle, navigation with wrapping, clear_all, bookmark list query
  - Covers: Requirement 8 AC 1–10

- [ ] 17. High-DPI pixel alignment
  - [ ] 17.1 Implement `PixelAligner` struct with `scale_factor` and `pixel_division` fields
  - [ ] 17.2 Implement `align(coord)` snapping to nearest device-pixel boundary
  - [ ] 17.3 Implement `align_rect_outward(x, y, w, h)` expanding rectangle to pixel boundaries
  - [ ] 17.4 Implement `scale_stroke(logical_width)` scaling stroke width for current DPI
  - [ ] 17.5 Implement `set_scale_factor(factor)` for monitor changes
  - [ ] 17.6 Write unit tests for alignment at 1x, 1.5x, 2x scale factors, stroke scaling, rect expansion
  - Covers: Requirement 10 AC 1–8

- [ ] 18. Rendering pipeline integration
  - [ ] 18.1 Define `DecorationRenderer` trait with methods: indicators_in_range, marker_mask_for_line, indicator_config, marker_config, hover_position, is_hovered_dynamic
  - [ ] 18.2 Implement `RenderingProvider` struct implementing `DecorationRenderer` trait, composing DecorationList, MarkerStore, IndicatorCatalogue, and HoverState
  - [ ] 18.3 Document layer ordering contract: Background markers → under-indicators → text → over-indicators → selection → gutter markers
  - [ ] 18.4 Implement indicator overlap rendering: indicators drawn in number order (lower first), all overlapping indicators visible simultaneously
  - [ ] 18.5 Write unit tests for RenderingProvider query correctness, layer ordering validation, overlap scenarios
  - Covers: Requirement 14 AC 1–7

- [ ] 19. Property-based tests
  - [ ] 19.1 Write PBT: RLE invariant — total length preservation after fill/insert/delete operations
  - [ ] 19.2 Write PBT: fill_range idempotency — second fill with same value returns false
  - [ ] 19.3 Write PBT: insert-delete round trip — insert_space then delete_range restores original state
  - [ ] 19.4 Write PBT: value consistency after edit — positions before/in/after insertion have correct values
  - [ ] 19.5 Write PBT: lazy creation and removal — decoration created on first non-zero write, removed when all zero
  - [ ] 19.6 Write PBT: marker line tracking — markers shift correctly on line insert/delete
  - [ ] 19.7 Write PBT: all_on_for consistency — bitmask matches individual value_at queries
  - [ ] 19.8 Write PBT: bookmark next/previous wrapping — correct navigation with wrapping
  - [ ] 19.9 Write PBT: run merge optimality — no adjacent runs with same value after any operation
  - [ ] 19.10 Write PBT: theme reload preserves decoration data — only visual properties change, not stored values
  - Covers: Correctness Properties 1–10 (see Property-Based Test Definitions below)

- [ ] 20. Integration tests
  - [ ] 20.1 Write integration test: multi-producer scenario — search highlights + diagnostic underlines coexist on same document
  - [ ] 20.2 Write integration test: edit synchronization — insert/delete text with active decorations from multiple indicators
  - [ ] 20.3 Write integration test: undo/redo cycle — decorations track positions through undo and redo operations
  - [ ] 20.4 Write integration test: theme hot-reload — change theme, verify visual properties update, verify decoration data unchanged
  - [ ] 20.5 Write integration test: bookmark lifecycle — toggle, navigate, insert lines, verify bookmark movement, clear all
  - [ ] 20.6 Write integration test: change history lifecycle — edit, save, undo, verify marker transitions
  - [ ] 20.7 Write integration test: indicator allocation — allocate multiple plugin indicators, exhaust container range, verify error
  - Covers: End-to-end validation across Requirements 1–15

---

## Property-Based Test Definitions

### Property 1: RLE Invariant — Total Length Preservation

**Validates: Requirements 3.10, 4.8**

- **Statement:** For any sequence of `fill_range`, `insert_space`, and `delete_range` operations on a `RunStyles<T>`, the sum of all run lengths always equals the tracked document length.
- **Strategy:** Generate:
  - initial_length: u64 in [1, 10_000]
  - operations: Vec of random Fill(pos, val, len) | Insert(pos, len) | Delete(pos, len) with valid positions
- **Invariant:** After all operations, `rs.total_length() == expected_length` (initial + sum of inserts − sum of deletes)

### Property 2: Fill Range Idempotency

**Validates: Requirements 3.8**

- **Statement:** Filling the same range with the same value twice produces the same state as filling once; the second fill returns `false` (no change).
- **Strategy:** Generate:
  - doc_length: u64 in [1, 5_000]
  - position: u64 in [0, doc_length)
  - length: u64 in [1, doc_length - position]
  - value: u32 in [0, 255]
- **Invariant:** `rs.fill_range(pos, val, len)` returns true first time, `false` second time; state unchanged after second call

### Property 3: Insert-Delete Round Trip

**Validates: Requirements 4.5, 4.6**

- **Statement:** Inserting space at position P with length L, then deleting the same range, restores the original decoration state.
- **Strategy:** Generate:
  - doc_length: u64 in [1, 5_000]
  - pre-fills: random set of fill_range calls to create non-trivial state
  - P: u64 in [0, doc_length]
  - L: u64 in [1, 1_000]
- **Invariant:** `clone → insert_space(P, L) → delete_range(P, L)` produces state equal to clone

### Property 4: Value Consistency After Edit

**Validates: Requirements 4.1, 4.3, 4.4**

- **Statement:** After `insert_space(P, L)`, positions before P retain original values, positions P..P+L have value 0, positions after P+L have original values from positions after P.
- **Strategy:** Generate:
  - doc_length: u64 in [10, 1_000]
  - pre-fills: random fill_range calls
  - P: u64 in [0, doc_length]
  - L: u64 in [1, 100]
- **Invariant:** For all i < P: value_at(i) unchanged; for i in P..P+L: value_at(i) == 0; for i >= P+L: value_at(i) == original[i-L]

### Property 5: Lazy Creation and Removal

**Validates: Requirements 3.3, 3.4**

- **Statement:** A decoration is created only on the first non-zero `fill_range` and removed when all values become zero.
- **Strategy:** Generate:
  - doc_length: u64 in [1, 5_000]
  - indicator: IndicatorNumber in [0, 43]
  - fill operations: random sequence of fill_range with value in [0, 5]
- **Invariant:** `active_count() == 0` initially; increases to 1 after first non-zero fill; returns to 0 after clearing all values to zero

### Property 6: Marker Line Tracking

**Validates: Requirements 9.10**

- **Statement:** After inserting K lines at line L, all markers originally on lines ≥ L move to line + K; markers on lines < L are unchanged.
- **Strategy:** Generate:
  - line_count: u64 in [10, 500]
  - markers: random set of (line, marker_number) placements
  - insert_line: u64 in [0, line_count]
  - insert_count: u64 in [1, 50]
- **Invariant:** After `lines_inserted(L, K)`, markers on lines < L unchanged, markers on lines >= L found at line + K

### Property 7: All-On-For Consistency

**Validates: Requirements 3.9**

- **Statement:** The `all_on_for(position)` bitmask is consistent with individual `value_at` queries: bit N is set iff `value_at(N, position) != 0`.
- **Strategy:** Generate:
  - doc_length: u64 in [1, 1_000]
  - random fill_range calls across multiple indicators
  - query_position: u64 in [0, doc_length)
- **Invariant:** For each indicator 0..=43: `(mask >> indicator) & 1 == (value_at(indicator, position) != 0) as u64`

### Property 8: Bookmark Next/Previous Wrapping

**Validates: Requirements 8.6**

- **Statement:** `marker_next` with bookmark mask returns the nearest bookmarked line at or after `from_line`, wrapping around the document. `marker_previous` wraps in reverse.
- **Strategy:** Generate:
  - line_count: u64 in [5, 200]
  - bookmarked_lines: non-empty sorted Vec of unique lines in [0, line_count)
  - from_line: u64 in [0, line_count)
- **Invariant:** `marker_next` returns smallest bookmarked line >= from_line, or smallest overall if none after; `marker_previous` returns largest bookmarked line <= from_line, or largest overall if none before

### Property 9: Run Merge Optimality

**Validates: Requirements 3.1**

- **Statement:** After any `fill_range` operation, no two adjacent runs have the same value (runs are always maximally merged).
- **Strategy:** Generate:
  - doc_length: u64 in [1, 5_000]
  - operations: random sequence of fill_range calls
- **Invariant:** For all i in 0..runs.len()-1: `runs[i].value != runs[i+1].value`

### Property 10: Theme Reload Preserves Decoration Data

**Validates: Requirements 2.10, 15.3**

- **Statement:** Reloading theme colours does not modify any stored indicator values or marker assignments — only visual rendering properties change.
- **Strategy:** Generate:
  - doc_length: u64 in [1, 1_000]
  - random fills across multiple indicators
  - random marker placements
  - new theme: random valid ThemeDecorationProvider values
- **Invariant:** After `catalogue.reload_from_theme(new_theme)`, all `value_at` and `marker_get` calls return identical results to before reload

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2", "3"], "dependsOn": [0] },
    { "id": 2, "label": "Storage Layer", "tasks": ["4", "5", "7"], "dependsOn": [1] },
    { "id": 3, "label": "Aggregation Layer", "tasks": ["6", "8", "9"], "dependsOn": [2] },
    { "id": 4, "label": "Synchronization and Configuration", "tasks": ["10", "11"], "dependsOn": [3] },
    { "id": 5, "label": "Interaction and Features", "tasks": ["12", "13", "14", "15", "16", "17"], "dependsOn": [4] },
    { "id": 6, "label": "Rendering Integration", "tasks": ["18"], "dependsOn": [5] },
    { "id": 7, "label": "Validation", "tasks": ["19", "20"], "dependsOn": [6] }
  ]
}
```

---

## Notes

- This is a Wave 6 (UI and Rendering) crate; the crate itself is GUI-independent (stores data, exposes query APIs)
- Actual rendering of decorations is performed by the shell layer (ff-desktop/egui) using the `DecorationRenderer` trait
- The 23 indicator styles are adapted from Scintilla's architecture to egui rendering primitives
- Run-length encoding ensures memory efficiency: a 1MB document with 10 error underlines uses O(20) runs, not O(1M)
- Marker positions track document lines — when lines are inserted/deleted, markers move with their content
- The `ThemeDecorationProvider` trait decouples this crate from the concrete theme implementation
- Property-based tests use the `proptest` crate with a minimum of 256 cases per property
- Bookmark commands are registered via `ff-command` for keyboard/menu access but the toggle/navigate logic lives in this crate
- Change history markers and character-level history indicators share the same state data, displayed at different granularities
- The IndicatorAllocator prevents indicator number conflicts between independent plugins at runtime

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Indicator Style Catalogue | AC 1.1–1.24 | Task 3 |
| Req 2: Indicator Properties and Configuration | AC 2.1–2.10 | Tasks 3, 4, 11 |
| Req 3: Decoration Storage (RLE) | AC 3.1–3.10 | Tasks 5, 6 |
| Req 4: Decoration Edit Synchronization | AC 4.1–4.8 | Tasks 5, 6, 10 |
| Req 5: Search Match Highlighting | AC 5.1–5.10 | Task 13 |
| Req 6: Diagnostic Underlines | AC 6.1–6.10 | Task 14 |
| Req 7: Change History Markers | AC 7.1–7.11 | Task 15 |
| Req 8: Bookmark Markers | AC 8.1–8.10 | Tasks 7, 8, 16 |
| Req 9: Line Marker System | AC 9.1–9.10 | Tasks 7, 8 |
| Req 10: High-DPI Rendering | AC 10.1–10.8 | Task 17 |
| Req 11: Hover Interaction | AC 11.1–11.7 | Task 12 |
| Req 12: Modified Line Indicator | AC 12.1–12.7 | Task 15 |
| Req 13: Indicator Number Allocation | AC 13.1–13.7 | Tasks 4, 6, 9 |
| Req 14: Rendering Pipeline Integration | AC 14.1–14.7 | Task 18 |
| Req 15: Theme Integration | AC 15.1–15.8 | Task 11 |
| Cross-cutting Req 8: Error Message Standards | All | Task 2 |
