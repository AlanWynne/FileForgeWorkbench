# Requirements Document

## Introduction

This feature specifies the **Large File Performance** subsystem for FileForgeWorkbench (`ff-large-file-performance` crate). The large-file-performance layer is a **GUI-independent rendering optimization infrastructure** that ensures the editor maintains responsive behaviour (60fps scrolling, sub-frame layout computation) when working with documents that contain very long lines (>10,000 characters), exceed one million lines, or combine both characteristics.

The subsystem provides four core capabilities:

1. **Long-line handling** — Lines exceeding 10,000 characters receive chunked measurement and partial rendering (only the visible portion within the viewport is measured and painted).
2. **Measurement caching** — Font metrics (character widths per style) and position layouts (x-positions of characters within a line) are cached to avoid redundant platform text-measurement calls.
3. **Chunked rendering** — Only lines within the visible viewport plus a configurable overscan buffer are rendered; lines outside this window are never measured or laid out.
4. **Viewport-aware lazy computation** — Expensive layout computations (measurement, wrap calculation, position mapping) are performed only for lines that are currently visible or about to become visible during scroll.

The design adapts Scintilla's `PositionCache`, `LineLayoutCache`, and `LineLayout` concepts into Rust. In Scintilla, the `PositionCache` is a hash-table of character-width measurements keyed by (style, text-content), and `LineLayoutCache` stores per-line layout results at configurable cache levels (None, Caret, Page, Document). This specification transposes those C++ patterns into a trait-based, cache-invalidation-aware Rust design that integrates with the workbench's document-model, viewport-and-scrolling, display-line-mapping, and idle-processing subsystems.

The crate is a Wave 15 (Background Processing and Performance) component. It depends on document-model for line content, viewport-and-scrolling for visible range, display-line-mapping for display-line heights, theme-and-appearance for font metrics sources, and background-io for async large-file streaming integration. It is consumed by the GUI rendering shell and idle-processing for background pre-computation.

**Source references:**
- **[SCI-PCACHE]** = Scintilla `PositionCache.h` / `PositionCache.cxx` — `IPositionCache`, `PositionCacheEntry`, `LineLayout`, `LineLayoutCache`, `BreakFinder`, hash-based measurement caching with clock eviction, two-way associative probing, mutex-guarded concurrent access
- **[SCI-EDIT-VIEW]** = Scintilla `EditView` — viewport rendering, `EnsureStyledTo`, visible-line-only painting, line subdivision for long lines (`BreakFinder::lengthStartSubdivision = 300`)
- **[WB]** = Workbench Platform Architecture Brief — GUI-independent core, responsive UI (60fps target), large-file support, memory-efficient operation

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `document-model` | **Dependency** | Provides line content, line count, byte positions, and edit notifications (insert/delete) that trigger cache invalidation. |
| `viewport-and-scrolling` | **Dependency** | Provides the visible line range (`top_line`, `visible_count`) and horizontal scroll offset that determine which lines and character ranges require measurement. |
| `display-line-mapping` | **Dependency** | Provides document-line to display-line mapping, wrap heights, and visibility state — determines which document lines contribute to display and require layout. |
| `background-io` | **Integration** | Provides async large-file streaming; large-file-performance coordinates with background-io's progressive loading to avoid measuring lines that have not yet been loaded. |
| `idle-processing` | **Integration** | Registers as a work source for background pre-computation of line layouts and measurement caches for lines near the viewport (lookahead caching). |
| `syntax-highlighting` | **Dependency** | Style slot assignments determine which font/style combination applies to each character range, affecting measurement cache keys. |
| `theme-and-appearance` | **Dependency** | Provides font metrics sources (font families, sizes, weights) whose changes trigger full measurement cache invalidation. |
| `view-zoom` | **Integration** | Zoom level changes invalidate all cached measurements because font metrics scale with zoom. |
| `configuration-system` | **Dependency** | Provides configurable parameters: long-line threshold, cache sizes, overscan buffer size, frame budget. |

## Glossary

- **PositionCache**: A hash-table data structure that stores measured character x-positions keyed by (style_slot, text_content) tuples, avoiding redundant platform font measurement calls for repeated text patterns. Adapted from Scintilla's `IPositionCache`. [SCI-PCACHE]
- **LineLayout**: A per-line data structure containing the measured x-positions of all characters in a document line, the sub-line break points (for wrapped lines), style assignments, and validity state. Adapted from Scintilla's `LineLayout`. [SCI-PCACHE]
- **LineLayoutCache**: A collection of LineLayout entries with configurable caching scope (viewport-only, viewport+overscan, or document-wide for small files). Adapted from Scintilla's `LineLayoutCache`. [SCI-PCACHE]
- **MeasurementCache**: The combined caching infrastructure comprising the PositionCache (short text fragment widths) and LineLayoutCache (full line layouts). [SCI-PCACHE]
- **Overscan_Buffer**: A configurable number of lines above and below the visible viewport that are pre-measured and cached, ensuring smooth scrolling without measurement stalls. [WB]
- **Long_Line_Threshold**: The character count (default 10,000) above which a line is treated as a "long line" and receives chunked measurement rather than full-line measurement. [SCI-PCACHE, WB]
- **Chunked_Measurement**: The process of measuring only a visible sub-range of a long line (the characters within the horizontal viewport plus a horizontal overscan margin) rather than measuring all characters from position 0 to end-of-line. [SCI-PCACHE]
- **Frame_Budget**: The maximum time (default 12ms, targeting 60fps with headroom) that layout and measurement operations may consume per frame before deferring remaining work to the next frame or idle time. [WB]
- **Cache_Validity**: A per-entry state indicating whether a cached measurement is current (`Valid`), needs style recheck (`CheckTextAndStyle`), needs remeasurement (`InvalidPositions`), or is completely stale (`Invalid`). Adapted from Scintilla's `LineLayout::ValidLevel`. [SCI-PCACHE]
- **Clock_Eviction**: The cache replacement strategy using a monotonic clock counter; entries that have not been accessed recently (lowest clock value among probe candidates) are evicted first. [SCI-PCACHE]
- **Font_Metrics_Key**: A composite key comprising (font_family, font_size, font_weight, font_style, zoom_level) that uniquely identifies a set of character-width measurements. A change in any component invalidates all measurements under that key. [SCI-PCACHE, WB]
- **Horizontal_Viewport**: The visible character range within a single line, determined by the horizontal scroll offset and the viewport pixel width. Characters outside this range on a long line need not be measured for rendering. [SCI-EDIT-VIEW]
- **Visible_Range**: The set of display lines currently within the viewport (from `top_line` to `top_line + visible_count - 1`), which are the only lines that require active measurement for rendering. [WB]
- **Pre_Computation**: Background measurement of lines in the overscan buffer during idle time, so that when the user scrolls those lines into view, their layouts are already cached and rendering is immediate. [SCI-PCACHE, WB]
- **Large_File_Indicator**: A status bar element displaying file size, line count, and operation progress for files exceeding the large-file threshold. [WB]
- **Memory_Mapped_Integration**: Coordination with the document-model's memory-efficient storage (streaming/mmap from background-io) to avoid requiring full file content in memory before measurement can begin. [WB]
- **Render_Chunk**: A subdivision of a long line into segments of manageable length (default 300 characters) for efficient text drawing and hit-testing. Adapted from Scintilla's `BreakFinder::lengthStartSubdivision`. [SCI-PCACHE]

---

## Requirements

### Requirement 1: Long-Line Chunked Measurement

**User Story:** As a user editing files with very long lines (log files, minified code, data dumps), I want the editor to remain responsive when navigating these lines, so that I can scroll and edit without freezing.

**Source:** [SCI-PCACHE] `LineLayout`, `BreakFinder`; [SCI-EDIT-VIEW] long-line rendering; [WB] responsive UI.

#### Acceptance Criteria

1. WHEN a document line exceeds the Long_Line_Threshold (default 10,000 characters, configurable), THE system SHALL apply chunked measurement mode for that line rather than measuring all characters from position 0 to end-of-line. [SCI-PCACHE, WB]
2. IN chunked measurement mode, THE system SHALL measure only the characters within the Horizontal_Viewport (determined by horizontal scroll offset and viewport pixel width) plus a horizontal overscan margin of 500 characters on each side (configurable). [SCI-PCACHE]
3. THE system SHALL maintain a partial LineLayout for long lines that stores x-positions only for the measured chunk, with a recorded start offset and measured-range boundary, enabling position lookups within the visible region. [SCI-PCACHE]
4. WHEN the horizontal scroll position changes on a long line, THE system SHALL extend or shift the measured chunk to cover the new visible region, reusing previously measured positions where the chunks overlap. [SCI-PCACHE]
5. THE Long_Line_Threshold SHALL be configurable via the configuration-system (`performance.long_line_threshold`), with a minimum of 1,000 and maximum of 100,000 characters; values outside this range SHALL be clamped. [WB]
6. WHEN rendering a long line, THE system SHALL subdivide the visible chunk into Render_Chunks of at most 300 characters (configurable) for text drawing, ensuring that a single draw call does not exceed manageable segment length. [SCI-PCACHE]
7. THE system SHALL compute the total line width of a long line lazily — the full width is only calculated when explicitly needed (e.g., for horizontal scrollbar range) and SHALL be estimated from the average character width when not fully measured. [WB]
8. IF the user scrolls horizontally beyond the currently measured chunk on a long line, THE system SHALL perform just-in-time measurement of the newly visible region within the current frame's budget, deferring extended pre-computation to idle time. [WB]
9. THE horizontal overscan margin for long-line chunked measurement SHALL be configurable via `performance.long_line_overscan_chars`, with a default of 500 characters and a range of [100, 5000]. [WB]

---

### Requirement 2: Font Metrics Measurement Cache (PositionCache)

**User Story:** As the rendering system, I need to cache character-width measurements by font and style, so that repeated text patterns (common keywords, indentation sequences, operator sequences) are measured once and reused across all lines.

**Source:** [SCI-PCACHE] `IPositionCache`, `PositionCacheEntry`, hash-based caching with clock eviction.

#### Acceptance Criteria

1. THE system SHALL maintain a PositionCache that stores measured character x-positions keyed by (style_slot_index, text_content) tuples, avoiding redundant calls to the platform text-measurement API for identical text+style combinations. [SCI-PCACHE]
2. THE PositionCache SHALL use a hash-table with two-way associative probing: for each lookup, two candidate slots are examined, and on insertion the entry with the lower clock value is evicted. [SCI-PCACHE]
3. THE PositionCache size SHALL be configurable via `performance.position_cache_size`, with a default of 1024 entries and a range of [256, 16384]. [SCI-PCACHE]
4. EACH PositionCache entry SHALL store: the style slot index, a unicode flag, the text content (for verification on retrieval), the measured x-positions array, and a clock timestamp for eviction ordering. [SCI-PCACHE]
5. THE PositionCache SHALL be thread-safe: measurement queries SHALL be safe to invoke from any thread (render thread, idle-processing thread) using a mutex guard. [SCI-PCACHE]
6. WHEN a cache hit occurs (matching style + text content), THE system SHALL copy cached positions to the caller's buffer without invoking platform measurement, and SHALL update the entry's clock. [SCI-PCACHE]
7. WHEN the clock counter wraps (exceeds 16-bit range), THE system SHALL reset all entry clocks to 1 to prevent stale entries from appearing newer than fresh ones. [SCI-PCACHE]
8. THE PositionCache SHALL expose a `Clear()` method that invalidates all entries, called when a global invalidation event occurs (font change, zoom change, theme switch). [SCI-PCACHE]
9. THE system SHALL track a per-style Font_Metrics_Key comprising (font_family, font_size, font_weight, font_style, zoom_level); WHEN any component of this key changes, ALL PositionCache entries for that style SHALL be invalidated. [SCI-PCACHE, WB]

---

### Requirement 3: Line Layout Cache

**User Story:** As the rendering system, I need to cache complete per-line layout results (character positions, sub-line breaks, styles) so that scrolling through previously visited lines is instantaneous.

**Source:** [SCI-PCACHE] `LineLayoutCache`, `LineLayout`, cache levels (None, Caret, Page, Document).

#### Acceptance Criteria

1. THE system SHALL maintain a LineLayoutCache that stores computed LineLayout entries for recently accessed document lines, avoiding full re-measurement when scrolling back to previously displayed content. [SCI-PCACHE]
2. THE LineLayoutCache SHALL support configurable cache levels: `Viewport` (cache only visible lines — default for files > 1M lines), `Page` (visible + overscan — default for files < 1M lines), and `Document` (all lines — only for files < 10,000 lines). [SCI-PCACHE, WB]
3. THE cache level SHALL be automatically selected based on document size, with manual override available via `performance.line_layout_cache_level`. [SCI-PCACHE]
4. EACH LineLayout entry SHALL store: the document line number, character content, style assignments, measured x-position array, sub-line break points (for wrapped lines), wrap indent, validity level, and whether the line contains the caret. [SCI-PCACHE]
5. THE LineLayoutCache SHALL support validity levels per entry: `Invalid` (must remeasure), `CheckTextAndStyle` (text may have changed — verify before reuse), `Positions` (positions valid but sub-line breaks need recalculation), `Lines` (fully valid). [SCI-PCACHE]
6. WHEN a document edit occurs, THE LineLayoutCache SHALL invalidate entries for the edited line and any lines whose style state may have changed (lines between the edit and the next style-stable point). [SCI-PCACHE]
7. THE LineLayoutCache SHALL evict entries using LRU ordering when the cache reaches capacity, prioritising retention of the caret line and visible viewport lines. [SCI-PCACHE]
8. THE LineLayoutCache capacity for `Page` level SHALL be `visible_count + 2 * overscan_buffer_lines` entries; for `Viewport` level SHALL be `visible_count` entries. [SCI-PCACHE]
9. A LineLayout entry SHALL be reusable for a given document line if the line number matches, the stored text length equals the current line length, and the validity level permits reuse at the requested level. [SCI-PCACHE]

---

### Requirement 4: Chunked Viewport Rendering

**User Story:** As a user scrolling through a million-line file, I want the editor to render only what I can see, so that rendering performance is independent of total file size.

**Source:** [SCI-EDIT-VIEW] visible-line-only rendering; [WB] 60fps scroll target; [SCI-PCACHE] `SignificantLines`.

#### Acceptance Criteria

1. THE rendering system SHALL compute layout and paint ONLY for document lines that map to display lines within the visible viewport (from `top_line` to `top_line + visible_count - 1`). [SCI-EDIT-VIEW, WB]
2. THE system SHALL maintain an Overscan_Buffer of lines above and below the viewport (default 5 lines, configurable via `performance.overscan_lines` in range [0, 50]) that are pre-measured but not painted, ready for immediate display on scroll. [WB]
3. WHEN the user scrolls, THE system SHALL render newly visible lines from the overscan cache if available, and SHALL measure new overscan lines in the background via idle-processing. [WB]
4. THE system SHALL NOT iterate over all document lines during a paint cycle — only the visible range (plus overscan) SHALL be accessed, ensuring O(visible_count) rendering complexity regardless of total line count. [WB]
5. WHEN a full repaint is triggered (window resize, theme change), THE system SHALL repaint only the visible viewport, invalidating and recomputing overscan in the background. [WB]
6. THE rendering frame budget SHALL be configurable via `performance.frame_budget_ms` (default 12ms, range [4, 32]), and the system SHALL defer measurement of overscan lines to the next frame or idle time if the budget is exceeded. [WB]
7. THE system SHALL track which document lines are "significant" (caret line, top line, lines on screen) and prioritise their layout computation and cache retention. [SCI-PCACHE]
8. WHEN the visible viewport changes (scroll, resize), THE system SHALL emit a viewport-change notification that triggers pre-computation of the new overscan range via the idle-processing scheduler. [WB]

---

### Requirement 5: Viewport-Aware Lazy Computation

**User Story:** As the workbench platform, I need layout computation to be demand-driven rather than eager, so that opening a million-line file does not trigger measurement of all lines upfront.

**Source:** [SCI-EDIT-VIEW] `EnsureStyledTo` pattern; [WB] lazy computation principle.

#### Acceptance Criteria

1. THE system SHALL NOT measure or compute layout for ANY line outside the visible viewport + overscan buffer until that line is explicitly requested (scrolled into view, searched, or navigated to). [WB]
2. THE system SHALL implement an `EnsureLayoutTo(display_line)` method that guarantees all lines up to the specified display line have valid layout data, computing missing layouts on demand. [SCI-EDIT-VIEW]
3. WHEN a GOTO-line or FIND command navigates to a line outside the current viewport, THE system SHALL compute layout for the target line and its surrounding overscan buffer, transitioning the viewport without computing intermediate lines. [WB]
4. DURING progressive file loading (via background-io streaming), THE system SHALL only measure lines that are currently within the viewport — lines loaded but not yet visible SHALL remain unmeasured until scrolled into view. [WB]
5. THE system SHALL track the "measured frontier" — the furthest line for which a valid layout exists — and SHALL NOT attempt to measure beyond lines that have been delivered by background-io. [WB]
6. WHEN idle-processing grants a time slice to the layout pre-computation work source, THE system SHALL measure lines in the overscan buffer ahead of the scroll direction (predictive pre-fetch), prioritising the direction of recent scroll momentum. [WB]
7. THE system SHALL maintain a count of unmeasured lines and expose it to the status bar for large-file progress indication (e.g., "Layout: 45,000 / 1,200,000 lines measured"). [WB]

---

### Requirement 6: Large-File Status Indicators

**User Story:** As a user working with large files, I want the status bar to show file size, line count, and progress of background operations, so that I understand the state of the file and can gauge how long operations will take.

**Source:** [WB] large-file UX; [SCI-STE-IO] progress indication.

#### Acceptance Criteria

1. WHEN a file exceeds the large-file threshold (as defined by background-io, default 100 MB), THE system SHALL display a large-file indicator in the status bar showing the file size in human-readable format (e.g., "245 MB"). [WB]
2. THE status bar SHALL display the total line count once the document model has completed line-index construction, showing a placeholder ("counting…") during progressive loading. [WB]
3. DURING background file loading (via background-io streaming), THE status bar SHALL display a progress indicator showing percentage loaded and estimated time remaining. [WB]
4. DURING background layout computation (via idle-processing), THE status bar SHALL display a secondary progress indicator showing the fraction of lines with computed layouts (e.g., "Layout: 60%"). [WB]
5. THE large-file indicator SHALL be suppressed for files below the large-file threshold — normal-sized files SHALL not show size or progress indicators. [WB]
6. WHEN a long-running operation completes (file fully loaded, layout fully computed), THE status bar indicator SHALL transition from progress display to a static summary and then fade or remove after 5 seconds. [WB]
7. IF layout computation is paused because the user is actively editing (idle-processing yields to input), THE progress indicator SHALL show "paused" state rather than appearing frozen. [WB]

---

### Requirement 7: Memory-Efficient Document Model Integration

**User Story:** As the workbench platform, I need the large-file-performance layer to work with the memory-efficient document storage (streaming, gap-buffer, potential mmap integration from background-io), so that large files are usable without requiring the entire file to reside in contiguous memory.

**Source:** [WB] memory efficiency; [SCI-PCACHE] streaming integration.

#### Acceptance Criteria

1. THE system SHALL obtain line content for measurement through the document-model's line-access API (which may provide content from a gap-buffer, rope, or memory-mapped region) without requiring a contiguous copy of the entire file. [WB]
2. WHEN requesting line content for measurement, THE system SHALL use borrowed references (`&str` slices) where possible, avoiding allocation of owned `String` copies for lines that are only being measured (not edited). [WB]
3. THE system SHALL coordinate with background-io's progressive loading: lines that have not yet been delivered to the document model SHALL be reported as "not yet available" and excluded from layout computation until delivered. [WB]
4. THE system SHALL implement a memory budget for the LineLayoutCache: the total memory consumed by cached LineLayout entries SHALL NOT exceed a configurable limit (default 64 MB, configurable via `performance.layout_cache_memory_mb` in range [16, 512]). [WB]
5. WHEN the memory budget is exceeded, THE system SHALL evict the least-recently-used LineLayout entries until memory usage falls below 90% of the budget. [WB]
6. FOR lines exceeding the Long_Line_Threshold, THE system SHALL request only the needed sub-range of characters from the document model (if the document model supports range access), avoiding allocation of the full line content into a temporary buffer. [WB]
7. THE system SHALL support documents with line counts exceeding 2^31 (using 64-bit line indexing from document-model and display-line-mapping), ensuring all cache keys and lookup indices use 64-bit line numbers. [WB]

---

### Requirement 8: Scroll Performance (60fps Target)

**User Story:** As a user scrolling through a file with more than one million lines, I want scrolling to feel smooth and responsive at 60fps, so that navigation is fluid regardless of file size.

**Source:** [WB] 60fps scroll target; [SCI-PCACHE] cache-driven rendering.

#### Acceptance Criteria

1. THE system SHALL maintain a sustained frame rate of at least 60 frames per second during continuous vertical scrolling through a file of 1,000,000+ lines, measured as: no frame exceeds 16.6ms from scroll-event receipt to paint completion. [WB]
2. DURING scrolling, IF a line's layout is available in the LineLayoutCache, THE system SHALL use the cached layout directly without any measurement calls, achieving O(1) per-line rendering cost. [SCI-PCACHE]
3. DURING scrolling, IF a line's layout is NOT cached (cache miss on fast scroll), THE system SHALL render the line with a simplified measurement (monospace approximation or last-known average character width) and schedule accurate measurement for the next idle period, then repaint when accurate data is available. [WB]
4. THE system SHALL detect scroll velocity and adjust the overscan pre-computation strategy: slow scrolling (< 5 lines/frame) pre-computes exact layouts; fast scrolling (> 20 lines/frame) uses simplified layouts until scrolling stops. [WB]
5. WHEN the user stops scrolling (no scroll event for 100ms), THE system SHALL trigger a refinement pass that replaces any simplified layouts with accurately measured layouts for all visible lines, triggering a repaint only if visual differences are detected. [WB]
6. THE system SHALL NOT block the scroll event handler for measurement — all measurement that cannot complete within 2ms per line SHALL be deferred, and the line SHALL be rendered with approximate metrics until accurate measurement completes. [WB]
7. HORIZONTAL scrolling through long lines SHALL maintain the same 60fps target by using the chunked measurement approach (Requirement 1) and pre-computing horizontal overscan during idle time. [WB]
8. THE system SHALL pre-compute layouts for the overscan buffer in the scroll direction during idle time, ensuring that normal-speed scrolling (< 3 lines per scroll event) always hits warm cache. [WB]

---

### Requirement 9: Cache Invalidation

**User Story:** As the editor system, I need measurement caches to be invalidated when underlying data changes (edits, font changes, zoom), so that stale layout data is never displayed.

**Source:** [SCI-PCACHE] `Invalidate`, `Clear`, validity levels; [WB] correctness guarantee.

#### Acceptance Criteria

1. WHEN a document edit occurs (character insertion, deletion, or replacement), THE system SHALL invalidate the LineLayout entry for the edited line, setting its validity to `Invalid`. [SCI-PCACHE]
2. WHEN a document edit occurs that changes line count (newline insertion or deletion), THE system SHALL invalidate all LineLayout entries for lines at or after the edit position (line numbers shift). [SCI-PCACHE]
3. WHEN the font family, font size, or font weight changes for ANY style slot (via theme change or configuration update), THE system SHALL clear the ENTIRE PositionCache and invalidate ALL LineLayout entries to `Invalid`. [SCI-PCACHE]
4. WHEN the zoom level changes (via view-zoom), THE system SHALL clear the ENTIRE PositionCache and invalidate ALL LineLayout entries, since all character measurements are zoom-dependent. [SCI-PCACHE]
5. WHEN the viewport width changes (window resize, panel dock/undock), THE system SHALL invalidate sub-line break data for all cached LineLayout entries (setting validity to `Positions` — positions are valid but wrap breaks need recalculation), WITHOUT clearing the PositionCache. [SCI-PCACHE]
6. WHEN a style change occurs on a line (syntax re-highlighting produces different style assignments), THE system SHALL invalidate that line's LineLayout entry to `CheckTextAndStyle`, triggering re-measurement only if the style actually changed between the cached and current state. [SCI-PCACHE]
7. THE system SHALL batch invalidation events during rapid editing: multiple edits within a single frame SHALL produce a single coalesced invalidation covering the affected range, rather than individual invalidations per keystroke. [WB]
8. WHEN the display-line-mapping reports a visibility change (line excluded/shown or fold toggled), THE system SHALL NOT invalidate cached measurements for the affected lines — the cached data remains valid for when the line becomes visible again. [SCI-PCACHE]
9. THE system SHALL expose an `invalidation_count` metric (number of invalidation events per second) for performance profiling, accessible via the logging subsystem at DEBUG level. [WB]

---

## Non-Functional Requirements

### NFR-1: Memory Efficiency

THE combined memory usage of the PositionCache and LineLayoutCache SHALL NOT exceed the configured memory budget under normal operation. THE system SHALL degrade gracefully (increased cache misses, not crashes) when memory pressure is detected.

### NFR-2: Thread Safety

ALL cache data structures SHALL be safe for concurrent read access from multiple threads (render thread, idle-processing thread). Write access (invalidation, insertion) SHALL be serialised via fine-grained locking that does not block reads for more than 1ms.

### NFR-3: Deterministic Behaviour

Cache hits and misses SHALL NOT affect the visual output — the rendered text SHALL be identical whether measurements come from cache or from fresh platform measurement calls. Caching is a performance optimisation only, never a correctness factor.

### NFR-4: Platform Independence

THE caching layer SHALL be GUI-independent. It operates on abstract measurement results (x-position arrays) and delegates actual text measurement to a platform-abstracted `Surface` trait. The cache logic has no dependency on egui, Win32, or any specific rendering backend.
