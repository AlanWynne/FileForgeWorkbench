# Implementation Plan: Large File Performance (`ff-large-file-performance`)

## Overview

This plan covers the complete implementation of the `ff-large-file-performance` crate — the rendering optimisation infrastructure for FileForgeWorkbench that ensures responsive behaviour (60fps scrolling, sub-frame layout computation) when working with documents containing very long lines (>10,000 characters), exceeding one million lines, or combining both characteristics.

The crate provides four core capabilities: long-line chunked measurement, font metrics measurement caching (PositionCache), line layout caching (LineLayoutCache), and viewport-aware lazy computation. The design adapts Scintilla's `PositionCache`, `LineLayoutCache`, and `LineLayout` concepts into a trait-based, cache-invalidation-aware Rust design.

This is a **Wave 15 (Background Processing and Performance)** sub-project that depends on `ff-document-model` (line content), `ff-viewport-and-scrolling` (visible range), `ff-display-line-mapping` (display-line heights), `ff-theme-and-appearance` (font metrics sources), `ff-background-io` (async streaming), `ff-idle-processing` (background pre-computation), `ff-syntax-highlighting` (style slots), `ff-view-zoom` (zoom level), and `ff-configuration-system` (configurable parameters).

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-large-file-performance/Cargo.toml` with dependencies (thiserror, parking_lot, proptest dev-dep) and workspace crate dependencies on `ff-document-model`, `ff-viewport-and-scrolling`, `ff-display-line-mapping`, `ff-configuration-system`, `ff-logging`
  - [x] 1.2 Create `crates/ff-large-file-performance/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `types.rs`, `error.rs`, `position_cache.rs`, `line_layout.rs`, `line_layout_cache.rs`, `chunked_measurement.rs`, `viewport_renderer.rs`, `lazy_computation.rs`, `invalidation.rs`, `scroll_performance.rs`, `status_indicator.rs`, `memory_budget.rs`, `config.rs`
  - [x] 1.4 Add `ff-large-file-performance` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Core types and configuration
  - [x] 2.1 Define `LineNumber(u64)` re-export or alias from document-model, `DisplayLine(u64)`, `BytePosition(u64)`, `CharOffset(u64)` newtypes
  - [x] 2.2 Define `StyleSlotIndex(u16)` newtype for style slot keying
  - [x] 2.3 Define `FontMetricsKey` struct with fields: font_family, font_size, font_weight, font_style, zoom_level — implement Hash, Eq
  - [x] 2.4 Define `CacheLevel` enum: `Viewport`, `Page`, `Document` with auto-selection logic
  - [x] 2.5 Define `CacheValidity` enum: `Invalid`, `CheckTextAndStyle`, `Positions`, `Lines`
  - [x] 2.6 Define `LargeFilePerformanceConfig` struct with all configurable parameters: long_line_threshold, position_cache_size, overscan_lines, frame_budget_ms, layout_cache_memory_mb, long_line_overscan_chars, render_chunk_size, line_layout_cache_level
  - [x] 2.7 Implement config loading from configuration-system with clamping for all bounded ranges
  - [x] 2.8 Write unit tests for config clamping and FontMetricsKey hashing
  - Covers: Requirement 1 (AC 1.5, 1.9), Requirement 2 (AC 2.3, 2.9), Requirement 3 (AC 3.2, 3.3, 3.5), Requirement 4 (AC 4.2, 4.6)

- [x] 3. Error types
  - [x] 3.1 Define `LargeFilePerformanceError` enum with variants: LineNotAvailable, PositionOutOfRange, CacheFull, MeasurementTimeout, ConfigInvalid, MemoryBudgetExceeded
  - [x] 3.2 Implement `Display` and `Error` traits via thiserror
  - [x] 3.3 Write unit tests for error formatting
  - Covers: Cross-cutting error handling requirements

- [x] 4. PositionCache (font metrics measurement cache)
  - [x] 4.1 Implement `PositionCacheEntry` struct with fields: style_slot_index, unicode_flag, text_content (for verification), x_positions array, clock timestamp
  - [x] 4.2 Implement `PositionCache` struct with hash-table storage, configurable capacity (default 1024), and monotonic clock counter
  - [x] 4.3 Implement hash function for (style_slot, text_content) tuple keys
  - [x] 4.4 Implement two-way associative probing: compute two candidate slot indices per lookup, examine both
  - [x] 4.5 Implement cache lookup: on hit, copy positions to caller buffer and update entry clock
  - [x] 4.6 Implement cache insertion: on miss, evict the candidate with lower clock value, store new entry
  - [x] 4.7 Implement clock wrapping: when counter exceeds u16::MAX, reset all entry clocks to 1
  - [x] 4.8 Implement `clear()` method that invalidates all entries (called on global invalidation events)
  - [x] 4.9 Implement thread-safety via `Mutex` guard for concurrent access from render and idle threads
  - [x] 4.10 Implement per-style `FontMetricsKey` tracking; when any key component changes, invalidate entries for that style
  - [x] 4.11 Write unit tests for insertion, lookup, eviction, clock wrapping, and thread safety
  - Covers: Requirement 2 (AC 2.1–2.9)

- [x] 5. LineLayout data structure
  - [x] 5.1 Implement `LineLayout` struct with fields: line_number, text_content, style_assignments, x_positions (Vec<f32>), sub_line_breaks (Vec<usize>), wrap_indent, validity (CacheValidity), contains_caret, measured_range (Option for partial layouts)
  - [x] 5.2 Implement `is_valid_for(line_number, text_length)` reuse check: line matches, text length matches, validity permits reuse
  - [x] 5.3 Implement `invalidate(level: CacheValidity)` setting validity to the specified level
  - [x] 5.4 Implement `memory_usage()` returning estimated byte size of the entry (for memory budgeting)
  - [x] 5.5 Implement partial LineLayout support for long lines: store start_offset and measured_range_boundary
  - [x] 5.6 Write unit tests for validity checks, memory estimation, and partial layout metadata
  - Covers: Requirement 3 (AC 3.4, 3.5, 3.9), Requirement 1 (AC 1.3)

- [x] 6. LineLayoutCache
  - [x] 6.1 Implement `LineLayoutCache` struct with storage (Vec or HashMap of LineLayout entries), capacity, cache level, memory budget tracking
  - [x] 6.2 Implement auto-selection of cache level based on document size: Document for <10K lines, Page for <1M lines, Viewport for >=1M lines
  - [x] 6.3 Implement capacity computation: Viewport level = visible_count entries; Page level = visible_count + 2*overscan entries
  - [x] 6.4 Implement `get(line_number)` returning Option<&LineLayout> if cached and valid
  - [x] 6.5 Implement `insert(line_layout)` with LRU eviction when at capacity, prioritising retention of caret line and visible lines
  - [x] 6.6 Implement `invalidate_line(line_number)` setting entry validity to Invalid
  - [x] 6.7 Implement `invalidate_from(line_number)` invalidating all entries at or after the given line (for line-count-changing edits)
  - [x] 6.8 Implement `invalidate_wrap_data()` setting all entries to Positions validity (positions valid, breaks need recalc)
  - [x] 6.9 Implement `clear()` invalidating all entries (for font/zoom changes)
  - [x] 6.10 Implement memory budget enforcement: track total memory, evict LRU entries when budget exceeds configured limit, evict until below 90%
  - [x] 6.11 Implement manual cache level override via configuration
  - [x] 6.12 Write unit tests for insertion, eviction, invalidation, memory budget, and level auto-selection
  - Covers: Requirement 3 (AC 3.1–3.9), Requirement 7 (AC 7.4, 7.5)

- [x] 7. Long-line chunked measurement
  - [x] 7.1 Implement `is_long_line(line_length, threshold)` detection function
  - [x] 7.2 Implement `ChunkedMeasurement` struct tracking: line_number, measured_start_offset, measured_end_offset, x_positions for measured range, horizontal_overscan_chars
  - [x] 7.3 Implement `measure_chunk(line_content, viewport_start_char, viewport_width_chars, overscan)` computing x-positions only for the visible sub-range plus overscan margins
  - [x] 7.4 Implement chunk extension on horizontal scroll: detect overlap with current measured range, measure only the newly exposed portion, splice into existing positions
  - [x] 7.5 Implement chunk shifting when scroll moves beyond overlap: discard old positions, measure new range from scratch
  - [x] 7.6 Implement render-chunk subdivision: split visible chunk into segments of max 300 chars (configurable) for draw calls
  - [x] 7.7 Implement lazy total-line-width estimation: compute from average character width when full measurement is not available
  - [x] 7.8 Implement just-in-time measurement for horizontal scroll beyond measured chunk within frame budget, deferring extended pre-computation to idle
  - [x] 7.9 Implement sub-range line content request interface (for memory-efficient document model integration)
  - [x] 7.10 Write unit tests for chunk measurement, extension, shifting, subdivision, and width estimation
  - Covers: Requirement 1 (AC 1.1–1.9), Requirement 7 (AC 7.6)

- [x] 8. Viewport renderer (chunked viewport rendering)
  - [x] 8.1 Implement `ViewportRenderer` struct tracking visible_range (top_line, visible_count), overscan_buffer_lines, and frame budget
  - [x] 8.2 Implement `visible_lines()` returning iterator over only the lines in the visible viewport (O(visible_count) complexity)
  - [x] 8.3 Implement `lines_to_render()` returning the visible lines that need paint (excludes overscan-only lines)
  - [x] 8.4 Implement `overscan_lines()` returning lines in the overscan buffer (above and below viewport) for pre-measurement
  - [x] 8.5 Implement frame budget tracking: start timer at frame begin, check remaining budget before each line measurement, defer if budget exceeded
  - [x] 8.6 Implement viewport-change detection and notification emission for triggering overscan pre-computation
  - [x] 8.7 Implement significant-line tracking: mark caret line, top line, and visible lines for priority cache retention
  - [x] 8.8 Implement repaint-only-visible on full repaint triggers (resize, theme change)
  - [x] 8.9 Write unit tests for visible range computation, overscan range, frame budget enforcement, and O(visible_count) complexity
  - Covers: Requirement 4 (AC 4.1–4.8)

- [x] 9. Viewport-aware lazy computation
  - [x] 9.1 Implement `LazyComputationEngine` struct tracking measured_frontier, unmeasured_count, and scroll direction
  - [x] 9.2 Implement `ensure_layout_to(display_line)` that computes layouts on demand for all lines up to the target without computing intermediate lines when jumping
  - [x] 9.3 Implement navigation-triggered computation: on GOTO or FIND, compute layout for target line and its overscan buffer only
  - [x] 9.4 Implement progressive-loading coordination: skip measurement for lines not yet delivered by background-io, reporting them as "not yet available"
  - [x] 9.5 Implement measured-frontier tracking: record the furthest line with valid layout, never measure beyond background-io delivery boundary
  - [x] 9.6 Implement idle-time pre-computation: measure overscan lines ahead of scroll direction (predictive pre-fetch based on recent scroll momentum)
  - [x] 9.7 Implement unmeasured-line count exposure for status bar progress display
  - [x] 9.8 Write unit tests for lazy computation, frontier tracking, navigation jumps, and progressive loading coordination
  - Covers: Requirement 5 (AC 5.1–5.7)

- [x] 10. Cache invalidation engine
  - [x] 10.1 Implement `InvalidationEvent` enum: LineEdit(line), LineCountChange(from_line, delta), FontChange, ZoomChange, ViewportWidthChange, StyleChange(line), FoldToggle(line)
  - [x] 10.2 Implement single-line edit invalidation: set edited line's LineLayout to Invalid
  - [x] 10.3 Implement line-count-change invalidation: invalidate all entries at or after edit position
  - [x] 10.4 Implement font/zoom change invalidation: clear entire PositionCache and invalidate all LineLayout entries
  - [x] 10.5 Implement viewport-width change: set all entries to Positions validity (wrap breaks need recalc, positions remain valid), do NOT clear PositionCache
  - [x] 10.6 Implement style-change invalidation: set affected line to CheckTextAndStyle, triggering re-measurement only if style actually differs
  - [x] 10.7 Implement batch coalescing: multiple edits within a single frame produce a single coalesced invalidation covering the affected range
  - [x] 10.8 Implement fold/visibility-change handling: do NOT invalidate cached data for hidden lines (cache remains valid for when line becomes visible again)
  - [x] 10.9 Implement `invalidation_count` metric tracking (events per second) at DEBUG log level
  - [x] 10.10 Write unit tests for each invalidation type, batch coalescing, and fold-toggle non-invalidation
  - Covers: Requirement 9 (AC 9.1–9.9)

- [x] 11. Scroll performance optimisation
  - [x] 11.1 Implement scroll-velocity detection: categorise as slow (<5 lines/frame), medium, or fast (>20 lines/frame)
  - [x] 11.2 Implement simplified layout fallback for cache misses during fast scroll: use monospace approximation or last-known average character width
  - [x] 11.3 Implement scroll-stop detection (no scroll event for 100ms) triggering a refinement pass
  - [x] 11.4 Implement refinement pass: replace simplified layouts with accurate measurements for all visible lines, repaint only if visual differences detected
  - [x] 11.5 Implement per-line measurement budget enforcement: defer measurement exceeding 2ms per line, render with approximate metrics
  - [x] 11.6 Implement adaptive overscan pre-computation: slow scroll pre-computes exact layouts, fast scroll uses simplified until stop
  - [x] 11.7 Implement horizontal scroll 60fps target using chunked measurement (delegate to task 7) with idle-time horizontal overscan pre-computation
  - [x] 11.8 Implement warm-cache guarantee for normal-speed scrolling (<3 lines/event) via idle-time overscan pre-computation
  - [x] 11.9 Write unit tests for velocity detection, fallback layout, refinement triggering, and per-line budget
  - Covers: Requirement 8 (AC 8.1–8.8)

- [x] 12. Large-file status indicators
  - [x] 12.1 Implement `LargeFileStatus` struct with fields: file_size_display, line_count_display, loading_progress, layout_progress, is_paused
  - [x] 12.2 Implement file-size display logic: show human-readable format (e.g., "245 MB") only when file exceeds large-file threshold (100 MB)
  - [x] 12.3 Implement line-count display: show "counting…" placeholder during progressive loading, switch to actual count when complete
  - [x] 12.4 Implement loading progress indicator: percentage loaded + estimated time remaining during background-io streaming
  - [x] 12.5 Implement layout progress indicator: fraction of lines with computed layouts (e.g., "Layout: 60%")
  - [x] 12.6 Implement completion transition: progress → static summary → fade/remove after 5 seconds
  - [x] 12.7 Implement paused state display when idle-processing yields to user input
  - [x] 12.8 Implement suppression for files below the large-file threshold
  - [x] 12.9 Write unit tests for status formatting, threshold suppression, and state transitions
  - Covers: Requirement 6 (AC 6.1–6.7)

- [x] 13. Memory-efficient document model integration
  - [x] 13.1 Implement line-content access via document-model's line API using borrowed `&str` slices (avoid owned String allocation for measurement-only access)
  - [x] 13.2 Implement coordination with background-io progressive loading: query line availability before measurement, handle "not yet available" gracefully
  - [x] 13.3 Implement sub-range character request for long lines: request only needed character range from document model when range-access is supported
  - [x] 13.4 Implement 64-bit line number support throughout all cache keys and lookup indices (documents exceeding 2^31 lines)
  - [x] 13.5 Write unit tests for borrowed-slice access, unavailable-line handling, and 64-bit line number edge cases
  - Covers: Requirement 7 (AC 7.1–7.7)

- [x] 14. Surface trait and measurement abstraction
  - [x] 14.1 Define `MeasurementSurface` trait with methods: `measure_text(text: &str, style: StyleSlotIndex) -> Vec<f32>` (x-positions), `measure_range(text: &str, start: usize, end: usize, style: StyleSlotIndex) -> Vec<f32>`, `average_char_width(style: StyleSlotIndex) -> f32`
  - [x] 14.2 Implement mock `MeasurementSurface` for testing (fixed-width character metrics)
  - [x] 14.3 Integrate MeasurementSurface into PositionCache and ChunkedMeasurement as the measurement backend
  - [x] 14.4 Write unit tests verifying platform-independence (all cache logic works with mock surface)
  - Covers: NFR-4 (Platform Independence)

- [x] 15. Property-based tests
  - [x] 15.1 Write PBT: PositionCache eviction fairness
  - [x] 15.2 Write PBT: LineLayoutCache consistency after invalidation sequences
  - [x] 15.3 Write PBT: chunked measurement overlap correctness
  - [x] 15.4 Write PBT: viewport rendering O(visible_count) complexity
  - [x] 15.5 Write PBT: cache invalidation completeness under random edits
  - [x] 15.6 Write PBT: scroll clamping and frame budget adherence
  - Covers: Requirements 1, 2, 3, 4, 8, 9 (see Property-Based Test Definitions below)

- [x] 16. Integration tests
  - [x] 16.1 Write integration test: full measurement lifecycle (measure → cache → invalidate → re-measure)
  - [x] 16.2 Write integration test: long-line horizontal scroll with chunk extension and re-measurement
  - [x] 16.3 Write integration test: viewport scroll through 1M-line document with cache warm/cold transitions
  - [x] 16.4 Write integration test: concurrent access from render thread and idle-processing thread
  - [x] 16.5 Write integration test: memory budget enforcement under sustained measurement load
  - Covers: End-to-end validation across Requirements 1–9

---

## Property-Based Test Definitions

### Property 1: PositionCache Eviction Fairness

**Validates: Requirement 2.2, 2.6, 2.7**

- **Statement:** For any sequence of cache insertions and lookups, the PositionCache SHALL never contain more entries than its configured capacity, and when the clock wraps, no stale entry SHALL appear newer than a freshly-inserted entry.
- **Strategy:** Generate:
  - Cache capacity: [256, 2048]
  - Operation sequence: 500–5000 operations of Insert(style, text, positions) or Lookup(style, text) with random style slots [0, 64] and text content [1, 50] chars
- **Invariant:** `cache.entry_count() <= capacity` always; after clock wrap, all entries have clock >= 1 and freshly-touched entries have higher clock than untouched entries

### Property 2: LineLayoutCache Consistency After Invalidation Sequences

**Validates: Requirement 3.1, 3.5, 3.6, 3.7**

- **Statement:** After any sequence of insert/invalidate/evict operations, every entry returned by `get(line_number)` SHALL have validity level >= the minimum required level (i.e., the cache never returns an Invalid entry as valid), and LRU eviction SHALL never evict the caret line while non-caret entries exist.
- **Strategy:** Generate:
  - Document line count: [100, 10000]
  - Visible range: random window of [20, 100] lines within document
  - Operation sequence: 100–1000 operations: insert_layout, invalidate_line, invalidate_from, get, set_caret_line
- **Invariant:** `get(n)` never returns entry with validity == Invalid; caret line is evicted only when cache contains only caret entries

### Property 3: Chunked Measurement Overlap Correctness

**Validates: Requirement 1.3, 1.4**

- **Statement:** When the horizontal scroll position changes and the measured chunk is extended or shifted, the x-positions for characters that were in both the old and new measured ranges SHALL remain identical (no measurement drift from chunk boundaries).
- **Strategy:** Generate:
  - Line length: [10000, 100000] characters
  - Initial viewport: random start position and width [500, 2000] chars
  - Scroll sequence: 10–50 horizontal scroll movements of varying magnitude [1, 5000] chars left or right
- **Invariant:** For all character positions that appear in both pre-scroll and post-scroll measured ranges, `old_x_positions[char] == new_x_positions[char]`

### Property 4: Viewport Rendering O(visible_count) Complexity

**Validates: Requirement 4.1, 4.4**

- **Statement:** The number of lines accessed during a single paint cycle SHALL never exceed `visible_count + 2 * overscan_buffer_lines`, regardless of total document line count.
- **Strategy:** Generate:
  - Document line count: [1000, 5000000]
  - Visible count: [20, 100]
  - Overscan buffer: [0, 50]
  - Viewport position: random top_line within document
- **Invariant:** `lines_accessed_during_paint <= visible_count + 2 * overscan_buffer_lines`

### Property 5: Cache Invalidation Completeness Under Random Edits

**Validates: Requirement 9.1, 9.2, 9.7**

- **Statement:** After any batch of document edits (insertions/deletions that may change line count), no LineLayout entry in the cache SHALL have stale content — every entry either has validity == Invalid or its stored text matches the current document line content.
- **Strategy:** Generate:
  - Initial document: [100, 5000] lines of random content
  - Edit batch: 1–20 random edits (insert chars, delete chars, insert newlines, delete newlines)
  - Apply edits then run invalidation engine
- **Invariant:** For every cached entry with validity != Invalid: `entry.text_content == document.line_content(entry.line_number)`

### Property 6: Scroll Clamping and Frame Budget Adherence

**Validates: Requirement 8.1, 8.6**

- **Statement:** During any scroll operation, the total time spent in measurement calls SHALL NOT exceed the configured frame budget, and lines rendered with simplified fallback layouts SHALL be replaced with accurate layouts within 2 subsequent idle cycles.
- **Strategy:** Generate:
  - Frame budget: [4, 32] ms
  - Document line count: [10000, 1000000]
  - Scroll velocity: random [1, 100] lines per frame
  - Measurement cost per line: random [0.1, 5.0] ms (simulated)
- **Invariant:** `total_measurement_time_per_frame <= frame_budget_ms`; after scroll stops, `simplified_layout_count == 0` within 2 idle cycles

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Config", "tasks": ["2", "3"], "dependsOn": [0] },
    { "id": 2, "label": "Measurement Abstraction", "tasks": ["14"], "dependsOn": [1] },
    { "id": 3, "label": "Cache Data Structures", "tasks": ["4", "5", "6"], "dependsOn": [2] },
    { "id": 4, "label": "Measurement Strategies", "tasks": ["7", "8"], "dependsOn": [3] },
    { "id": 5, "label": "Computation Engine", "tasks": ["9", "10"], "dependsOn": [4] },
    { "id": 6, "label": "Performance and Integration", "tasks": ["11", "12", "13"], "dependsOn": [5] },
    { "id": 7, "label": "Validation and PBT", "tasks": ["15", "16"], "dependsOn": [6] }
  ]
}
```

---

## Notes

- This is a Wave 15 (Background Processing and Performance) crate depending on `ff-document-model` (Wave 4), `ff-viewport-and-scrolling` (Wave 4), `ff-display-line-mapping` (Wave 4), `ff-syntax-highlighting` (Wave 7), `ff-theme-and-appearance` (Wave 6), `ff-background-io` (Wave 8), `ff-idle-processing` (Wave 15), `ff-view-zoom` (Wave 9), and `ff-configuration-system` (Wave 2)
- The `MeasurementSurface` trait provides platform independence — all cache logic operates on abstract x-position arrays without knowledge of the rendering backend (egui, Win32, etc.)
- The PositionCache uses Scintilla's two-way associative probing with clock eviction for O(1) amortised lookup and minimal memory fragmentation
- The LineLayoutCache auto-selects caching scope based on document size to balance memory usage and cache-hit rates
- Long-line chunked measurement is the key enabler for responsive editing of minified JavaScript, log files, and data dumps — only the visible horizontal slice is ever measured
- The scroll performance layer implements a graceful degradation strategy: fast scrolling uses approximate layouts (instant), accurate layouts are computed in the background, and refinement repaints only occur when visual differences exist
- Cache invalidation uses batch coalescing to avoid per-keystroke invalidation storms during rapid editing
- Memory budgeting (default 64 MB for LineLayoutCache) ensures the performance layer does not consume unbounded memory on very large files
- All line indices use 64-bit (`u64`) to support documents exceeding 2^31 lines
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The `invalidation_count` metric enables performance profiling during development without runtime overhead in release builds (behind DEBUG log level)

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Long-Line Chunked Measurement | AC 1.1–1.9 | Tasks 7, 2, 5 |
| Req 2: Font Metrics Measurement Cache (PositionCache) | AC 2.1–2.9 | Tasks 4, 2, 14 |
| Req 3: Line Layout Cache | AC 3.1–3.9 | Tasks 5, 6, 2 |
| Req 4: Chunked Viewport Rendering | AC 4.1–4.8 | Tasks 8, 2, 9 |
| Req 5: Viewport-Aware Lazy Computation | AC 5.1–5.7 | Tasks 9, 12 |
| Req 6: Large-File Status Indicators | AC 6.1–6.7 | Task 12 |
| Req 7: Memory-Efficient Document Model Integration | AC 7.1–7.7 | Tasks 13, 6, 7 |
| Req 8: Scroll Performance (60fps Target) | AC 8.1–8.8 | Tasks 11, 7, 8 |
| Req 9: Cache Invalidation | AC 9.1–9.9 | Task 10 |
| NFR-1: Memory Efficiency | — | Tasks 6, 13 |
| NFR-2: Thread Safety | — | Tasks 4, 6 |
| NFR-3: Deterministic Behaviour | — | Tasks 15, 16 |
| NFR-4: Platform Independence | — | Task 14 |
