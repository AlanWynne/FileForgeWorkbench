# Design Document: Large File Performance (`ff-large-file-performance`)

## Overview

The `ff-large-file-performance` crate is the **GUI-independent rendering optimisation infrastructure** for the FileForgeWorkbench editor. It ensures responsive behaviour (60fps scrolling, sub-frame layout computation) when working with documents containing very long lines (>10,000 characters), exceeding one million lines, or combining both characteristics.

### Purpose

- Measure and cache character x-positions per font/style combination (PositionCache)
- Cache complete per-line layout results for instant re-rendering (LineLayoutCache)
- Perform chunked measurement of long lines — only the visible portion is measured
- Render only lines within the viewport plus configurable overscan buffer
- Implement viewport-aware lazy computation — no upfront measurement of all lines
- Provide large-file status indicators for user awareness
- Coordinate with background-io progressive loading and idle-processing pre-computation

### Position in Architecture (Wave 15)

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│  Queries layout results for rendering, invokes measurement   │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-large-file-performance ← Wave 15            │
│  PositionCache, LineLayoutCache, ChunkRenderer, LazyLayout   │
├─────────────────────────────────────────────────────────────┤
│  Peers/Upstream:                                             │
│    ff-document-model (line content, edit notifications)       │
│    ff-viewport-scrolling (visible range, scroll state)       │
│    ff-display-line-mapping (display↔document line mapping)   │
│    ff-background-io (progressive file loading coordination)  │
│    ff-idle-processing (background pre-computation scheduler) │
│    ff-syntax-highlighting (style slot assignments)           │
│    ff-theme-and-appearance (font metrics sources)            │
│    ff-view-zoom (zoom level changes)                         │
│    ff-config (configurable parameters)                       │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging (Wave 0)            │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Zero GUI dependencies — operates on abstract `Surface` trait for measurement
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-large-file-performance`
- **Error Message Standards (Req 8)**: Errors follow `[large-file-perf] operation: description` format
- **Thread Safety (NFR-2)**: All cache structures safe for concurrent read access; writes serialised via fine-grained locking
- **Determinism (NFR-3)**: Cache hits/misses never affect visual output — rendering is identical regardless of cache state
- **Platform Independence (NFR-4)**: Delegates actual text measurement to abstract `Surface` trait

### Upstream Dependencies

| Crate | API Consumed |
|-------|-------------|
| `ff-document-model` | `Document::line_count()`, `Document::line_start()`, `Document::line_end()`, `Document::get_range()`, `DocumentWatcher` trait for edit notifications |
| `ff-viewport-scrolling` | `ViewportModel::top_line()`, `ViewportModel::visible_count()`, `ViewportModel::horizontal_offset()`, viewport-change events |
| `ff-display-line-mapping` | `DisplayLineMapper` trait for document↔display line mapping |
| `ff-background-io` | `IoTaskHandle::progress()` for loading state, `ProgressState` for loaded-line frontier |
| `ff-idle-processing` | `IdleWorkSource` trait for registering layout pre-computation work |
| `ff-config` | Typed access API for `[performance.*]` configuration keys |
| `ff-logging` | Diagnostic output, `invalidation_count` metric at DEBUG level |
| `ff-syntax-highlighting` | Style slot assignments per character range |
| `ff-theme-and-appearance` | Font metrics (family, size, weight, style) per style slot |
| `ff-view-zoom` | Zoom level change notifications |

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Consumers [Consuming Crates]
        DESKTOP[ff-desktop: GUI rendering shell]
        IDLE[ff-idle-processing: background pre-computation]
    end

    subgraph ff-large-file-performance [ff-large-file-performance Crate]
        LPM[LayoutPerformanceManager<br/>Central coordination facade]
        PC[PositionCache<br/>Hash-table measurement cache]
        LLC[LineLayoutCache<br/>Per-line layout result store]
        CR[ChunkRenderer<br/>Long-line subdivision for drawing]
        LWC[LineWidthCache<br/>Estimated/actual total line widths]
        LM[LazyLayoutManager<br/>Viewport-aware demand-driven layout]
        INV[InvalidationCoordinator<br/>Batched cache invalidation]
        SI[StatusIndicator<br/>Large-file progress reporting]
        WS[LayoutWorkSource<br/>IdleWorkSource impl for pre-computation]
        CFG[PerfConfig<br/>Configuration reader]
    end

    subgraph Upstream [Upstream Crates]
        DOC[ff-document-model]
        VP[ff-viewport-scrolling]
        DLM[ff-display-line-mapping]
        BIO[ff-background-io]
        SYN[ff-syntax-highlighting]
        THEME[ff-theme-and-appearance]
        ZOOM[ff-view-zoom]
        CONFIG[ff-config]
        LOG[ff-logging]
    end

    DESKTOP -->|request_layout, render_line| LPM
    IDLE -->|dispatch time slice| WS

    LPM --> PC
    LPM --> LLC
    LPM --> CR
    LPM --> LWC
    LPM --> LM
    LPM --> INV
    LPM --> SI
    WS --> LM

    LM -->|line content| DOC
    LM -->|visible range| VP
    LM -->|display mapping| DLM
    LM -->|loaded frontier| BIO
    INV -->|edit notifications| DOC
    INV -->|font changes| THEME
    INV -->|zoom changes| ZOOM
    PC -->|style slots| SYN
    CFG -->|read settings| CONFIG
    SI -->|progress data| BIO
    LPM --> LOG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **LayoutPerformanceManager** | Central facade: coordinates measurement requests, cache lookups, invalidation, and status reporting |
| **PositionCache** | Hash-table storing measured character x-positions keyed by (style_slot, text_content), two-way associative probing with clock eviction |
| **LineLayoutCache** | Collection of LineLayout entries with configurable scope (Viewport/Page/Document), LRU eviction, memory-budget enforcement |
| **ChunkRenderer** | Subdivides visible portions of long lines into render chunks (≤300 chars) for efficient drawing |
| **LineWidthCache** | Stores known and estimated total widths of lines for horizontal scrollbar range computation |
| **LazyLayoutManager** | Demand-driven layout engine: computes measurements only for lines within viewport+overscan, tracks measured frontier |
| **InvalidationCoordinator** | Receives edit/font/zoom/resize events, batches invalidations within a frame, dispatches to caches |
| **StatusIndicator** | Exposes large-file status data (file size, line count, layout progress) for status bar display |
| **LayoutWorkSource** | `IdleWorkSource` implementation that pre-computes layouts for overscan lines during idle time |
| **PerfConfig** | Reads and validates all `[performance.*]` configuration keys with clamped ranges |

### Data Flow: Line Layout Request

```
1. Desktop shell requests layout for display_line N via LayoutPerformanceManager
2. LazyLayoutManager maps display_line → document_line via DisplayLineMapper
3. Check LineLayoutCache for document_line:
   a. HIT (validity == Lines): return cached LineLayout immediately
   b. HIT (validity == Positions): recalculate sub-line breaks, update, return
   c. HIT (validity == CheckTextAndStyle): verify text/style, remeasure if changed
   d. MISS: proceed to step 4
4. Obtain line content from document-model (borrowed &str if possible)
5. Determine if line exceeds Long_Line_Threshold:
   a. YES: compute visible chunk range from horizontal viewport + overscan
   b. NO: measure full line
6. For each style run in the measurement range:
   a. Check PositionCache for (style_slot, text_fragment)
   b. HIT: copy cached x-positions
   c. MISS: invoke Surface::measure_text(), store result in PositionCache
7. Assemble LineLayout (x-positions, sub-line breaks, style assignments)
8. Store LineLayout in LineLayoutCache
9. Return LineLayout to caller
```

---

## Components and Interfaces

```
crates/ff-large-file-performance/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API re-exports, crate docs
│   ├── manager.rs                # LayoutPerformanceManager facade
│   ├── position_cache/
│   │   ├── mod.rs                # PositionCache re-exports
│   │   ├── cache.rs              # PositionCache struct: hash-table, probing, clock eviction
│   │   ├── entry.rs              # PositionCacheEntry: style, text, x-positions, clock
│   │   └── hash.rs              # Hash function for (style_slot, text_content) keys
│   ├── line_layout/
│   │   ├── mod.rs                # LineLayout re-exports
│   │   ├── layout.rs             # LineLayout struct: positions, breaks, styles, validity
│   │   ├── cache.rs              # LineLayoutCache: LRU eviction, level-based scoping
│   │   ├── validity.rs           # ValidLevel enum and transitions
│   │   └── memory.rs             # Memory budget tracking and enforcement
│   ├── chunk/
│   │   ├── mod.rs                # Chunking re-exports
│   │   ├── renderer.rs           # ChunkRenderer: subdivide visible range into draw chunks
│   │   ├── measurement.rs        # Chunked measurement logic for long lines
│   │   └── line_width.rs         # LineWidthCache: actual/estimated total widths
│   ├── lazy/
│   │   ├── mod.rs                # Lazy computation re-exports
│   │   ├── layout_manager.rs     # LazyLayoutManager: demand-driven, frontier tracking
│   │   ├── work_source.rs        # LayoutWorkSource: IdleWorkSource impl
│   │   └── scroll_predictor.rs   # Scroll direction detection for predictive pre-fetch
│   ├── invalidation/
│   │   ├── mod.rs                # Invalidation re-exports
│   │   ├── coordinator.rs        # InvalidationCoordinator: batching, dispatch
│   │   └── events.rs             # InvalidationEvent enum, coalescing logic
│   ├── status.rs                 # StatusIndicator: large-file progress data
│   ├── surface.rs                # Surface trait: abstract text measurement interface
│   ├── config.rs                 # PerfConfig: read/validate performance settings
│   ├── types.rs                  # Newtypes: StyleSlot, ClockValue, ChunkRange, etc.
│   └── error.rs                  # LargeFilePerfError enum
└── tests/
    ├── position_cache_tests.rs   # PositionCache hit/miss, eviction, clock wrap
    ├── line_layout_tests.rs      # LineLayout validity transitions, reuse
    ├── line_layout_cache_tests.rs # LineLayoutCache LRU, levels, memory budget
    ├── chunk_renderer_tests.rs   # Render chunk subdivision for long lines
    ├── chunked_measurement_tests.rs # Long-line partial measurement
    ├── lazy_layout_tests.rs      # LazyLayoutManager demand-driven computation
    ├── invalidation_tests.rs     # Invalidation batching, event handling
    ├── scroll_predictor_tests.rs # Scroll velocity and direction prediction
    ├── config_tests.rs           # Configuration clamping and defaults
    ├── integration.rs            # End-to-end: layout request → cache → render
    └── property_tests.rs         # Property-based tests (proptest)
```

---

## Data Models

### Core Newtypes

```rust
/// A style slot index identifying a font/style combination.
/// Corresponds to syntax-highlighting style assignments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StyleSlot(pub u16);

/// A monotonic clock value for cache eviction ordering.
/// Wraps at u16::MAX and resets all entries to prevent stale comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClockValue(pub u16);

/// A character offset within a line (0-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharOffset(pub u64);

/// An x-position in fractional pixels from the left margin of a line.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct XPosition(pub f64);

/// A range of characters within a line for chunked measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRange {
    /// Start character offset (inclusive)
    pub start: CharOffset,
    /// End character offset (exclusive)
    pub end: CharOffset,
}

/// The render chunk size limit for text drawing calls.
/// Clamped to [50, 1000]. Default: 300.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderChunkSize(pub u32);

impl RenderChunkSize {
    pub const MIN: u32 = 50;
    pub const MAX: u32 = 1000;
    pub const DEFAULT: u32 = 300;

    /// Create a RenderChunkSize, clamping to valid range.
    pub fn new(chars: u32) -> Self {
        Self(chars.clamp(Self::MIN, Self::MAX))
    }
}

/// The long-line threshold in characters.
/// Clamped to [1_000, 100_000]. Default: 10_000.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LongLineThreshold(pub u32);

impl LongLineThreshold {
    pub const MIN: u32 = 1_000;
    pub const MAX: u32 = 100_000;
    pub const DEFAULT: u32 = 10_000;

    pub fn new(chars: u32) -> Self {
        Self(chars.clamp(Self::MIN, Self::MAX))
    }
}
```

### PositionCache

```rust
/// A hash-table cache storing measured character x-positions keyed by
/// (style_slot, text_content). Uses two-way associative probing with
/// clock-based eviction. Thread-safe via internal Mutex.
///
/// Adapted from Scintilla's IPositionCache.
/// Addresses: Requirement 2 (all criteria)
pub struct PositionCache {
    /// Cache entries indexed by hash probe slots
    entries: Vec<Option<PositionCacheEntry>>,
    /// Total number of slots (configurable, power of 2)
    capacity: usize,
    /// Monotonic clock for eviction ordering
    clock: ClockValue,
    /// Mutex for thread-safe access
    lock: Mutex<()>,
}

impl PositionCache {
    /// Create a new PositionCache with the given capacity.
    /// Capacity is rounded up to the next power of 2.
    /// Addresses: Req 2 AC 3
    pub fn new(capacity: usize) -> Self;

    /// Look up cached x-positions for the given style+text combination.
    /// On hit: copies positions to `output`, updates clock, returns true.
    /// On miss: returns false.
    /// Addresses: Req 2 AC 6
    pub fn lookup(
        &self,
        style: StyleSlot,
        text: &str,
        unicode: bool,
        output: &mut [XPosition],
    ) -> bool;

    /// Store measured x-positions for the given style+text combination.
    /// On collision: evicts the entry with the lower clock value.
    /// Addresses: Req 2 AC 2
    pub fn store(
        &self,
        style: StyleSlot,
        text: &str,
        unicode: bool,
        positions: &[XPosition],
    );

    /// Clear all entries. Called on global invalidation (font/zoom/theme change).
    /// Addresses: Req 2 AC 8
    pub fn clear(&self);

    /// Current number of occupied slots.
    pub fn len(&self) -> usize;

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool;
}
```

### PositionCacheEntry

```rust
/// A single entry in the PositionCache.
/// Stores the measurement key (style + text) and cached x-positions.
///
/// Addresses: Requirement 2 AC 4
pub struct PositionCacheEntry {
    /// The style slot this measurement applies to
    pub style: StyleSlot,
    /// Whether the text contains non-ASCII unicode characters
    pub unicode: bool,
    /// The text content (for verification on retrieval)
    pub text: String,
    /// Measured x-positions for each character in the text
    pub positions: Vec<XPosition>,
    /// Clock timestamp at last access (for eviction ordering)
    pub clock: ClockValue,
}
```

### LineLayout

```rust
/// Per-line layout data: measured x-positions, sub-line breaks, style runs,
/// and validity state. Cached by LineLayoutCache for instant re-rendering.
///
/// Adapted from Scintilla's LineLayout.
/// Addresses: Requirement 3 AC 4
pub struct LineLayout {
    /// The document line number this layout represents
    pub line_number: u64,
    /// Character content length (for reuse validation)
    pub text_length: u64,
    /// Measured x-positions for each character (may be partial for long lines)
    pub positions: Vec<XPosition>,
    /// For long lines: the measured chunk range (None = full line measured)
    pub measured_range: Option<ChunkRange>,
    /// Sub-line break points for wrapped lines (character offsets where wraps occur)
    pub sub_line_breaks: Vec<CharOffset>,
    /// Style slot assignments per character run: (start_offset, style_slot)
    pub style_runs: Vec<(CharOffset, StyleSlot)>,
    /// Wrap indent in pixels (for continuation lines)
    pub wrap_indent: XPosition,
    /// Current validity level
    pub validity: ValidLevel,
    /// Whether this line contains the caret (prioritised for retention)
    pub contains_caret: bool,
    /// Estimated memory consumption of this entry in bytes
    pub memory_bytes: usize,
}

impl LineLayout {
    /// Check if this layout is reusable for the given line.
    /// Addresses: Req 3 AC 9
    pub fn is_reusable_for(&self, line_number: u64, text_length: u64) -> bool;

    /// Get the x-position for a character offset within this layout.
    pub fn x_position_at(&self, offset: CharOffset) -> Option<XPosition>;

    /// Get the character offset nearest to an x-position (for hit-testing).
    pub fn offset_at_x(&self, x: XPosition) -> CharOffset;

    /// Number of sub-lines (1 for unwrapped, >1 for wrapped).
    pub fn sub_line_count(&self) -> usize;
}
```

### ValidLevel

```rust
/// Validity levels for a LineLayout entry.
/// Determines what must be recomputed before the entry can be reused.
///
/// Adapted from Scintilla's LineLayout::ValidLevel.
/// Addresses: Requirement 3 AC 5
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidLevel {
    /// Completely stale — must remeasure from scratch
    Invalid = 0,
    /// Text or style may have changed — verify before reuse
    CheckTextAndStyle = 1,
    /// Positions valid but sub-line breaks need recalculation (e.g., after resize)
    Positions = 2,
    /// Fully valid — positions and sub-line breaks are current
    Lines = 3,
}
```

### LineLayoutCache

```rust
/// Collection of LineLayout entries with level-based scoping, LRU eviction,
/// and memory budget enforcement.
///
/// Adapted from Scintilla's LineLayoutCache.
/// Addresses: Requirement 3 (all criteria)
pub struct LineLayoutCache {
    /// Cached layouts indexed by document line number
    entries: HashMap<u64, LineLayoutEntry>,
    /// LRU ordering (most-recent at back)
    lru_order: VecDeque<u64>,
    /// Current cache level
    level: CacheLevel,
    /// Maximum entry count (derived from level + viewport size)
    max_entries: usize,
    /// Memory budget in bytes
    memory_budget: usize,
    /// Current memory usage in bytes
    memory_used: usize,
    /// Lock for thread-safe access
    lock: RwLock<()>,
}

/// Cache scoping level — determines how many lines are cached.
/// Addresses: Requirement 3 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLevel {
    /// Cache only visible viewport lines (for files > 1M lines)
    Viewport,
    /// Cache visible + overscan buffer (default for files < 1M lines)
    Page,
    /// Cache all lines (only for files < 10,000 lines)
    Document,
}

/// Wrapper for a cached LineLayout with LRU metadata.
struct LineLayoutEntry {
    layout: LineLayout,
    last_access: u64, // monotonic counter for LRU
}

impl LineLayoutCache {
    /// Create a new LineLayoutCache with the given level and viewport size.
    /// Addresses: Req 3 AC 8
    pub fn new(level: CacheLevel, visible_count: usize, overscan: usize) -> Self;

    /// Set the memory budget in bytes.
    /// Addresses: Req 7 AC 4
    pub fn set_memory_budget(&mut self, budget_bytes: usize);

    /// Look up a cached LineLayout for the given line number.
    /// Returns None on cache miss; updates LRU on hit.
    pub fn get(&mut self, line_number: u64) -> Option<&LineLayout>;

    /// Store a LineLayout. May evict LRU entries if at capacity or memory budget.
    /// Addresses: Req 3 AC 7
    pub fn insert(&mut self, layout: LineLayout);

    /// Invalidate a single line's entry.
    /// Addresses: Req 9 AC 1
    pub fn invalidate_line(&mut self, line_number: u64);

    /// Invalidate all entries at or after the given line (for line-count changes).
    /// Addresses: Req 9 AC 2
    pub fn invalidate_from(&mut self, line_number: u64);

    /// Set validity level for all entries (e.g., Positions after resize).
    /// Addresses: Req 9 AC 5
    pub fn downgrade_all_to(&mut self, level: ValidLevel);

    /// Invalidate a specific line to CheckTextAndStyle.
    /// Addresses: Req 9 AC 6
    pub fn mark_check_style(&mut self, line_number: u64);

    /// Clear all entries (full invalidation).
    /// Addresses: Req 9 AC 3, AC 4
    pub fn clear(&mut self);

    /// Update the cache level (auto-selected based on document size).
    /// Addresses: Req 3 AC 3
    pub fn set_level(&mut self, level: CacheLevel, visible_count: usize, overscan: usize);

    /// Current memory usage in bytes.
    pub fn memory_used(&self) -> usize;

    /// Number of cached entries.
    pub fn len(&self) -> usize;
}
```

### ChunkRenderer

```rust
/// Subdivides the visible portion of a long line into render chunks
/// of manageable length for efficient text drawing and hit-testing.
///
/// Adapted from Scintilla's BreakFinder with lengthStartSubdivision = 300.
/// Addresses: Requirement 1 AC 6
pub struct ChunkRenderer {
    /// Maximum characters per render chunk
    chunk_size: RenderChunkSize,
}

impl ChunkRenderer {
    /// Create a ChunkRenderer with the given maximum chunk size.
    pub fn new(chunk_size: RenderChunkSize) -> Self;

    /// Subdivide a character range into render chunks.
    /// Returns an iterator of ChunkRange values, each ≤ chunk_size characters.
    pub fn subdivide(&self, range: ChunkRange) -> Vec<ChunkRange>;

    /// Determine the render chunk containing a given character offset.
    pub fn chunk_containing(&self, range: ChunkRange, offset: CharOffset) -> ChunkRange;
}
```

### LineWidthCache

```rust
/// Stores known and estimated total widths of document lines.
/// Used for horizontal scrollbar range computation without measuring
/// every character of every line.
///
/// Addresses: Requirement 1 AC 7
pub struct LineWidthCache {
    /// Known widths for lines that have been fully measured
    known_widths: HashMap<u64, XPosition>,
    /// Running average character width (for estimation)
    average_char_width: f64,
    /// Maximum known width across all lines (for scrollbar range)
    max_known_width: XPosition,
}

impl LineWidthCache {
    /// Create a new LineWidthCache.
    pub fn new() -> Self;

    /// Record a known total width for a line.
    pub fn set_known_width(&mut self, line_number: u64, width: XPosition);

    /// Get the known width for a line, or estimate from average char width.
    pub fn width_for(&self, line_number: u64, char_count: u64) -> XPosition;

    /// Get the maximum content width for horizontal scrollbar range.
    pub fn max_content_width(&self) -> XPosition;

    /// Update average character width from a measurement sample.
    pub fn update_average(&mut self, sample_width: f64, sample_chars: u64);

    /// Invalidate width for a specific line.
    pub fn invalidate_line(&mut self, line_number: u64);

    /// Clear all cached widths (full invalidation).
    pub fn clear(&mut self);
}
```

### LazyLayoutManager

```rust
/// Demand-driven layout engine. Computes measurements only for lines within
/// the viewport + overscan buffer. Tracks measured frontier for progressive
/// loading coordination.
///
/// Addresses: Requirement 5 (all criteria)
pub struct LazyLayoutManager {
    /// The furthest line for which a valid layout has been computed
    measured_frontier: u64,
    /// Total lines available (from document-model loading state)
    available_lines: u64,
    /// Current overscan buffer size in lines
    overscan_lines: u32,
    /// Frame budget for measurement per frame
    frame_budget: Duration,
    /// Scroll direction predictor for pre-fetch
    scroll_predictor: ScrollPredictor,
    /// Count of unmeasured lines (for status indicator)
    unmeasured_count: u64,
}

impl LazyLayoutManager {
    /// Create a new LazyLayoutManager with configuration.
    pub fn new(overscan_lines: u32, frame_budget: Duration) -> Self;

    /// Ensure layout exists for all lines up to the given display line.
    /// Computes missing layouts on demand.
    /// Addresses: Req 5 AC 2
    pub fn ensure_layout_to(
        &mut self,
        display_line: u64,
        cache: &mut LineLayoutCache,
        position_cache: &PositionCache,
        surface: &dyn Surface,
        doc: &dyn LineContentProvider,
        mapper: &dyn DisplayLineMapper,
    ) -> Result<(), LargeFilePerfError>;

    /// Compute layouts for overscan lines in the predicted scroll direction.
    /// Called during idle time via LayoutWorkSource.
    /// Addresses: Req 5 AC 6
    pub fn pre_compute_overscan(
        &mut self,
        viewport_top: u64,
        visible_count: u64,
        cache: &mut LineLayoutCache,
        position_cache: &PositionCache,
        surface: &dyn Surface,
        doc: &dyn LineContentProvider,
        budget: Duration,
    ) -> WorkStatus;

    /// Update the available lines count (from background-io progress).
    /// Addresses: Req 5 AC 5
    pub fn set_available_lines(&mut self, count: u64);

    /// Get the current measured frontier.
    pub fn measured_frontier(&self) -> u64;

    /// Get the count of unmeasured lines (for status indicator).
    /// Addresses: Req 5 AC 7
    pub fn unmeasured_count(&self) -> u64;
}

/// Predicts scroll direction from recent scroll events for pre-fetch prioritisation.
/// Addresses: Requirement 8 AC 4
pub struct ScrollPredictor {
    /// Recent scroll deltas (ring buffer)
    recent_deltas: VecDeque<i64>,
    /// Predicted direction: positive = scrolling down, negative = scrolling up
    predicted_direction: i64,
    /// Detected velocity in lines per frame
    velocity: f64,
}

impl ScrollPredictor {
    /// Record a scroll event (positive = down, negative = up).
    pub fn record_scroll(&mut self, delta: i64);

    /// Get the predicted scroll direction.
    pub fn predicted_direction(&self) -> ScrollDirection;

    /// Get the current scroll velocity (lines per frame).
    pub fn velocity(&self) -> f64;

    /// Whether scrolling is considered "fast" (> 20 lines/frame).
    /// Addresses: Req 8 AC 4
    pub fn is_fast_scrolling(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Down,
    Up,
    Stationary,
}
```

### InvalidationCoordinator

```rust
/// Coordinates cache invalidation across all subsystems. Batches multiple
/// invalidation events within a single frame into coalesced operations.
///
/// Addresses: Requirement 9 (all criteria)
pub struct InvalidationCoordinator {
    /// Pending invalidation events for the current frame
    pending_events: Vec<InvalidationEvent>,
    /// Whether we are within a frame batch window
    in_batch: bool,
    /// Metric: invalidation events per second
    invalidation_count: u64,
    /// Timestamp of last metric reset
    last_metric_reset: Instant,
}

/// An invalidation event that affects cached measurements.
/// Addresses: Requirement 9
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InvalidationEvent {
    /// A single line was edited (content changed, same line count).
    LineEdited { line_number: u64 },
    /// Lines were inserted or deleted (line count changed).
    LinesChanged { from_line: u64, lines_delta: i64 },
    /// A style change occurred on a specific line.
    StyleChanged { line_number: u64 },
    /// Font metrics changed (font family, size, weight, or style).
    FontChanged,
    /// Zoom level changed.
    ZoomChanged,
    /// Viewport width changed (affects sub-line breaks only).
    ViewportResized,
    /// Display-line visibility changed (fold/unfold/exclude).
    VisibilityChanged { line_number: u64 },
}

impl InvalidationCoordinator {
    /// Create a new InvalidationCoordinator.
    pub fn new() -> Self;

    /// Begin a batch window (call at start of frame).
    /// Addresses: Req 9 AC 7
    pub fn begin_batch(&mut self);

    /// Submit an invalidation event to the current batch.
    pub fn submit(&mut self, event: InvalidationEvent);

    /// End the batch window and dispatch coalesced invalidations to caches.
    /// Addresses: Req 9 AC 7
    pub fn flush(
        &mut self,
        position_cache: &PositionCache,
        line_layout_cache: &mut LineLayoutCache,
        line_width_cache: &mut LineWidthCache,
    );

    /// Get the invalidation count metric (events per second).
    /// Addresses: Req 9 AC 9
    pub fn invalidation_rate(&self) -> u64;
}
```

### StatusIndicator

```rust
/// Exposes large-file status data for status bar display.
/// Tracks file size, line count, loading progress, and layout progress.
///
/// Addresses: Requirement 6 (all criteria)
pub struct StatusIndicator {
    /// Whether the current file exceeds the large-file threshold
    pub is_large_file: bool,
    /// File size in bytes (for display)
    pub file_size_bytes: u64,
    /// Total line count (None while counting)
    pub total_lines: Option<u64>,
    /// Loading progress percentage (None if not loading)
    pub loading_progress: Option<u8>,
    /// Layout computation progress (fraction of lines with layouts)
    pub layout_progress: Option<f64>,
    /// Whether layout computation is paused (user is editing)
    pub layout_paused: bool,
    /// Timestamp when last progress indicator completed (for fade timer)
    pub completion_time: Option<Instant>,
}

impl StatusIndicator {
    /// Create a new StatusIndicator (inactive state).
    pub fn new() -> Self;

    /// Format file size as human-readable string (e.g., "245 MB").
    /// Addresses: Req 6 AC 1
    pub fn formatted_file_size(&self) -> Option<String>;

    /// Format line count for display (or "counting…" placeholder).
    /// Addresses: Req 6 AC 2
    pub fn formatted_line_count(&self) -> String;

    /// Whether the status indicator should be visible.
    /// Addresses: Req 6 AC 5
    pub fn is_visible(&self) -> bool;

    /// Whether a completion indicator should still be shown (within 5s fade).
    /// Addresses: Req 6 AC 6
    pub fn is_showing_completion(&self) -> bool;
}
```

### Surface Trait

```rust
/// Abstract text measurement interface. The caching layer delegates actual
/// platform-specific font measurement to implementors of this trait.
/// GUI shells (egui, test mocks) provide concrete implementations.
///
/// Addresses: NFR-4 (Platform Independence)
pub trait Surface: Send + Sync {
    /// Measure the x-positions of characters in `text` using the given style.
    /// Returns a vector of x-positions (one per character, cumulative from left).
    ///
    /// `positions[i]` = the x-coordinate of the right edge of character `i`.
    fn measure_text(
        &self,
        style: StyleSlot,
        text: &str,
        positions: &mut [XPosition],
    );

    /// Get the average character width for a style (for estimation).
    fn average_char_width(&self, style: StyleSlot) -> f64;

    /// Get the line height for the current font configuration.
    fn line_height(&self) -> f64;
}
```

### LineContentProvider Trait

```rust
/// Abstraction over document-model for obtaining line content.
/// Enables testing without full Document dependency.
///
/// Addresses: Requirement 7 AC 1, AC 2
pub trait LineContentProvider {
    /// Get the character count for a line.
    fn line_char_count(&self, line_number: u64) -> Option<u64>;

    /// Get line content as a borrowed string slice.
    /// Returns None if the line is not yet loaded (progressive loading).
    /// Addresses: Req 7 AC 2, AC 3
    fn line_content(&self, line_number: u64) -> Option<&str>;

    /// Get a sub-range of line content (for long-line chunked measurement).
    /// Returns None if the line or range is not available.
    /// Addresses: Req 7 AC 6
    fn line_content_range(
        &self,
        line_number: u64,
        start: CharOffset,
        end: CharOffset,
    ) -> Option<&str>;

    /// Total line count in the document.
    fn line_count(&self) -> u64;

    /// Whether a line is available (loaded from background-io).
    /// Addresses: Req 7 AC 3
    fn is_line_available(&self, line_number: u64) -> bool;
}
```

### DisplayLineMapper Trait

```rust
/// Abstraction over display-line-mapping for document↔display line conversion.
/// Consumed from ff-display-line-mapping.
pub trait DisplayLineMapper {
    /// Convert a display line to a document line number.
    fn display_to_document(&self, display_line: u64) -> Option<u64>;

    /// Convert a document line to its first display line.
    fn document_to_display(&self, doc_line: u64) -> Option<u64>;

    /// Total display line count.
    fn total_display_lines(&self) -> u64;

    /// Whether a document line is currently visible (not folded/excluded).
    fn is_visible(&self, doc_line: u64) -> bool;
}
```

### LayoutWorkSource

```rust
/// IdleWorkSource implementation that performs background pre-computation
/// of line layouts for overscan lines during idle time.
///
/// Addresses: Requirement 4 AC 3, Requirement 5 AC 6
pub struct LayoutWorkSource {
    /// Reference to the lazy layout manager
    layout_manager: Arc<RwLock<LazyLayoutManager>>,
    /// Reference to caches
    line_layout_cache: Arc<RwLock<LineLayoutCache>>,
    position_cache: Arc<PositionCache>,
    /// Surface for measurement
    surface: Arc<dyn Surface>,
    /// Document content provider
    doc: Arc<dyn LineContentProvider>,
    /// Current viewport state for overscan computation
    viewport_top: u64,
    visible_count: u64,
}

impl IdleWorkSource for LayoutWorkSource {
    /// Perform a bounded unit of pre-computation work.
    /// Measures lines in the overscan buffer ahead of scroll direction.
    fn perform_work(&mut self, context: &mut IdleWorkContext) -> WorkStatus;

    /// Priority: lower than syntax highlighting, higher than search indexing.
    fn priority(&self) -> WorkPriority;

    /// Human-readable name for diagnostics.
    fn name(&self) -> &str { "layout-precomputation" }

    /// Current progress (measured lines / total lines).
    fn progress(&self) -> WorkProgress;

    /// Reset progress on invalidation.
    fn invalidate(&mut self);
}
```

### PerfConfig

```rust
/// Configuration values for the large-file-performance subsystem.
/// Read from ff-config `[performance.*]` namespace with clamped ranges.
///
/// Addresses: Requirements 1–9 configuration criteria
#[derive(Debug, Clone)]
pub struct PerfConfig {
    /// Long-line threshold in characters. Default: 10,000. Range: [1000, 100000].
    /// Config key: `performance.long_line_threshold`
    pub long_line_threshold: LongLineThreshold,

    /// Horizontal overscan margin for long-line chunked measurement.
    /// Default: 500 characters. Range: [100, 5000].
    /// Config key: `performance.long_line_overscan_chars`
    pub long_line_overscan_chars: u32,

    /// Render chunk size for long-line subdivision. Default: 300.
    /// Config key: `performance.render_chunk_size`
    pub render_chunk_size: RenderChunkSize,

    /// PositionCache capacity (number of entries). Default: 1024. Range: [256, 16384].
    /// Config key: `performance.position_cache_size`
    pub position_cache_size: usize,

    /// LineLayoutCache level override (None = auto-select based on file size).
    /// Config key: `performance.line_layout_cache_level`
    pub line_layout_cache_level: Option<CacheLevel>,

    /// Overscan buffer size in lines. Default: 5. Range: [0, 50].
    /// Config key: `performance.overscan_lines`
    pub overscan_lines: u32,

    /// Frame budget in milliseconds. Default: 12. Range: [4, 32].
    /// Config key: `performance.frame_budget_ms`
    pub frame_budget_ms: u32,

    /// Layout cache memory budget in MB. Default: 64. Range: [16, 512].
    /// Config key: `performance.layout_cache_memory_mb`
    pub layout_cache_memory_mb: u32,
}

impl PerfConfig {
    /// Load configuration from ff-config, applying clamping to all values.
    pub fn from_config(config: &dyn ConfigProvider) -> Self;

    /// Get the frame budget as a Duration.
    pub fn frame_budget(&self) -> Duration {
        Duration::from_millis(self.frame_budget_ms as u64)
    }

    /// Get the memory budget in bytes.
    pub fn memory_budget_bytes(&self) -> usize {
        self.layout_cache_memory_mb as usize * 1024 * 1024
    }
}
```

### LayoutPerformanceManager

```rust
/// Central facade coordinating all large-file performance subsystems.
/// Entry point for desktop shell layout requests, invalidation handling,
/// and status queries.
///
/// Thread-safe, owns all internal caches behind Arc+Lock.
pub struct LayoutPerformanceManager {
    /// Font metrics measurement cache
    position_cache: Arc<PositionCache>,
    /// Per-line layout result cache
    line_layout_cache: Arc<RwLock<LineLayoutCache>>,
    /// Estimated/actual line width cache
    line_width_cache: Arc<RwLock<LineWidthCache>>,
    /// Long-line chunk renderer
    chunk_renderer: ChunkRenderer,
    /// Demand-driven layout engine
    lazy_manager: Arc<RwLock<LazyLayoutManager>>,
    /// Invalidation batching coordinator
    invalidation: Arc<RwLock<InvalidationCoordinator>>,
    /// Large-file status data
    status: Arc<RwLock<StatusIndicator>>,
    /// Configuration
    config: Arc<PerfConfig>,
}

impl LayoutPerformanceManager {
    /// Create a new manager with the given configuration.
    pub fn new(config: PerfConfig) -> Self;

    /// Request layout for a display line. Returns cached or freshly computed layout.
    /// This is the primary entry point for the rendering shell.
    pub fn request_layout(
        &self,
        display_line: u64,
        surface: &dyn Surface,
        doc: &dyn LineContentProvider,
        mapper: &dyn DisplayLineMapper,
    ) -> Result<Arc<LineLayout>, LargeFilePerfError>;

    /// Request layouts for the entire visible range.
    /// Addresses: Req 4 AC 1
    pub fn request_visible_layouts(
        &self,
        top_line: u64,
        visible_count: u64,
        surface: &dyn Surface,
        doc: &dyn LineContentProvider,
        mapper: &dyn DisplayLineMapper,
    ) -> Result<Vec<Arc<LineLayout>>, LargeFilePerfError>;

    /// Notify of a viewport change (triggers overscan pre-computation).
    /// Addresses: Req 4 AC 8
    pub fn notify_viewport_changed(
        &self,
        top_line: u64,
        visible_count: u64,
        horizontal_offset: f64,
    );

    /// Submit an invalidation event.
    pub fn invalidate(&self, event: InvalidationEvent);

    /// Begin frame batch for invalidation coalescing.
    pub fn begin_frame(&self);

    /// End frame batch, flush pending invalidations.
    pub fn end_frame(&self);

    /// Get the status indicator data.
    pub fn status(&self) -> StatusIndicator;

    /// Get the idle work source for registration with ff-idle-processing.
    pub fn work_source(
        &self,
        surface: Arc<dyn Surface>,
        doc: Arc<dyn LineContentProvider>,
    ) -> Box<dyn IdleWorkSource>;

    /// Handle a font/theme change: clear all caches.
    /// Addresses: Req 9 AC 3
    pub fn on_font_changed(&self);

    /// Handle a zoom change: clear all caches.
    /// Addresses: Req 9 AC 4
    pub fn on_zoom_changed(&self);

    /// Handle a viewport resize: downgrade sub-line breaks.
    /// Addresses: Req 9 AC 5
    pub fn on_viewport_resized(&self);

    /// Update configuration (hot-reload).
    pub fn update_config(&self, config: PerfConfig);
}
```

---

## Public API Surface

### Construction and Lifecycle

```rust
impl LayoutPerformanceManager {
    /// Create a new LayoutPerformanceManager with default configuration.
    pub fn with_defaults() -> Self;

    /// Create from explicit PerfConfig.
    pub fn new(config: PerfConfig) -> Self;

    /// Shutdown: release all cached memory, unregister work source.
    pub fn shutdown(&self);
}
```

### Layout Requests (Primary Interface)

```rust
impl LayoutPerformanceManager {
    /// Request layout for a single display line.
    /// If cached and valid, returns immediately (O(1)).
    /// If cache miss, measures on demand within frame budget.
    pub fn request_layout(
        &self,
        display_line: u64,
        surface: &dyn Surface,
        doc: &dyn LineContentProvider,
        mapper: &dyn DisplayLineMapper,
    ) -> Result<Arc<LineLayout>, LargeFilePerfError>;

    /// Request layouts for the visible viewport range.
    /// Optimised batch operation.
    pub fn request_visible_layouts(
        &self,
        top_line: u64,
        visible_count: u64,
        surface: &dyn Surface,
        doc: &dyn LineContentProvider,
        mapper: &dyn DisplayLineMapper,
    ) -> Result<Vec<Arc<LineLayout>>, LargeFilePerfError>;

    /// Ensure layout exists up to a specific line (for GOTO/FIND).
    /// Addresses: Req 5 AC 3
    pub fn ensure_layout_to(
        &self,
        display_line: u64,
        surface: &dyn Surface,
        doc: &dyn LineContentProvider,
        mapper: &dyn DisplayLineMapper,
    ) -> Result<(), LargeFilePerfError>;
}
```

### Chunked Long-Line Measurement

```rust
impl LayoutPerformanceManager {
    /// For a long line, compute the visible chunk range given horizontal scroll state.
    /// Returns the ChunkRange that should be measured.
    /// Addresses: Req 1 AC 2
    pub fn compute_visible_chunk(
        &self,
        line_char_count: u64,
        horizontal_offset: f64,
        viewport_width: f64,
        surface: &dyn Surface,
        style: StyleSlot,
    ) -> ChunkRange;

    /// Extend or shift a previously measured chunk for horizontal scroll.
    /// Reuses overlapping positions from the existing layout.
    /// Addresses: Req 1 AC 4
    pub fn extend_chunk(
        &self,
        existing: &LineLayout,
        new_range: ChunkRange,
        surface: &dyn Surface,
        doc: &dyn LineContentProvider,
        line_number: u64,
    ) -> Result<LineLayout, LargeFilePerfError>;
}
```

### Invalidation Interface

```rust
impl LayoutPerformanceManager {
    /// Submit an invalidation event (may be batched).
    pub fn invalidate(&self, event: InvalidationEvent);

    /// Bulk invalidation: document edit notification.
    pub fn on_document_edit(&self, line: u64, lines_delta: i64);

    /// Bulk invalidation: style re-highlight.
    pub fn on_style_changed(&self, line: u64);

    /// Global invalidation: font/theme/zoom.
    pub fn on_font_changed(&self);
    pub fn on_zoom_changed(&self);

    /// Partial invalidation: viewport resize (sub-line breaks only).
    pub fn on_viewport_resized(&self);
}
```

### Status and Metrics

```rust
impl LayoutPerformanceManager {
    /// Get current status indicator data for status bar.
    pub fn status(&self) -> StatusIndicator;

    /// Get invalidation rate metric (events/sec).
    pub fn invalidation_rate(&self) -> u64;

    /// Get cache hit rate for PositionCache (0.0–1.0).
    pub fn position_cache_hit_rate(&self) -> f64;

    /// Get cache hit rate for LineLayoutCache (0.0–1.0).
    pub fn line_layout_cache_hit_rate(&self) -> f64;

    /// Get current memory usage of all caches combined.
    pub fn total_memory_used(&self) -> usize;
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-large-file-performance crate.
/// Formatted per Error Message Standards: `[large-file-perf] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LargeFilePerfError {
    /// Line is not yet available (progressive loading in progress).
    #[error("[large-file-perf] layout: line {line_number} not yet loaded (frontier: {frontier})")]
    LineNotAvailable {
        line_number: u64,
        frontier: u64,
    },

    /// Display line number is out of valid range.
    #[error("[large-file-perf] layout: display line {display_line} out of range (total: {total_display_lines})")]
    DisplayLineOutOfRange {
        display_line: u64,
        total_display_lines: u64,
    },

    /// Frame budget exceeded during measurement — layout deferred.
    #[error("[large-file-perf] measurement: frame budget exceeded after {measured_lines} lines (budget: {budget_ms}ms)")]
    FrameBudgetExceeded {
        measured_lines: u64,
        budget_ms: u32,
    },

    /// Memory budget exceeded — eviction required before new layouts can be stored.
    #[error("[large-file-perf] cache: memory budget exceeded ({used_mb}MB / {budget_mb}MB)")]
    MemoryBudgetExceeded {
        used_mb: u64,
        budget_mb: u64,
    },

    /// Surface measurement failed (platform error).
    #[error("[large-file-perf] measurement: surface measurement failed for style {style}: {reason}")]
    MeasurementFailed {
        style: u16,
        reason: String,
    },

    /// Configuration error.
    #[error("[large-file-perf] config: {description}")]
    ConfigError {
        description: String,
    },
}
```

---

## Integration Points

### With `ff-document-model` (Wave 4 — upstream dependency)

- **Dependency direction**: ff-large-file-performance depends on ff-document-model
- **API consumed**: `Document::line_count()`, `Document::line_start()`, `Document::line_end()`, `Document::get_range()` for line content access
- **LineContentProvider**: Implemented by a wrapper around `DocumentHandle` that provides borrowed `&str` slices (via `SplitView` or direct access) without requiring owned `String` copies
- **DocumentWatcher**: Registered for edit notifications (`notify_insert`, `notify_delete`) to trigger cache invalidation via InvalidationCoordinator
- **64-bit line numbers**: All line references use `u64` (matching `LineNumber(u64)` from document-model) for files exceeding 2^31 lines
- **Progressive loading**: Coordinates with `LoadingProgress` to avoid measuring lines not yet delivered by `StreamingFileReader`

### With `ff-background-io` (Wave 8 — integration)

- **Dependency direction**: ff-large-file-performance queries ff-background-io progress
- **API consumed**: `IoTaskHandle::progress()` for `ProgressState` (bytes_transferred, total_bytes, percentage)
- **Large-file coordination**: StatusIndicator reads loading progress for status bar display; LazyLayoutManager uses loaded-line frontier to avoid measuring beyond available content
- **Memory pressure**: Registers memory-pressure callback to pause layout pre-computation when background-io is consuming significant memory during large-file streaming

### With `ff-viewport-scrolling` (Wave 4 — upstream dependency)

- **Dependency direction**: ff-large-file-performance depends on ff-viewport-scrolling
- **API consumed**: `ViewportModel::top_line()`, `ViewportModel::visible_count()`, `ViewportModel::horizontal_offset()` for determining visible range
- **Viewport events**: Subscribes to `ViewportChanged` events to trigger overscan pre-computation and horizontal chunk adjustment
- **Scroll velocity**: Uses scroll event frequency to detect fast scrolling and switch to simplified measurement mode

### With `ff-display-line-mapping` (Wave 4 — upstream dependency)

- **Dependency direction**: ff-large-file-performance depends on ff-display-line-mapping
- **API consumed**: `DisplayLineMapper` trait for display↔document line conversion
- **Visibility state**: Queries whether a document line is visible (not folded/excluded) to skip measurement for hidden lines
- **Wrap integration**: Sub-line breaks computed in LineLayout are fed back to display-line-mapping for accurate display line counts

### With `ff-idle-processing` (Wave 15 — integration)

- **Dependency direction**: ff-large-file-performance implements `IdleWorkSource` trait from ff-idle-processing
- **Registration**: `LayoutWorkSource` is registered with the idle scheduler for background pre-computation of overscan layouts
- **Priority**: Layout pre-computation has lower priority than syntax highlighting but higher than search indexing
- **Cooperative yielding**: `LayoutWorkSource::perform_work()` checks time budget via `IdleWorkContext` and yields before exceeding the time slice

### With `ff-config` (Wave 2 — upstream dependency)

- **Dependency direction**: ff-large-file-performance depends on ff-config
- **API consumed**: Typed access API for reading `[performance.*]` namespace keys
- **Configuration keys**: `performance.long_line_threshold`, `performance.long_line_overscan_chars`, `performance.render_chunk_size`, `performance.position_cache_size`, `performance.line_layout_cache_level`, `performance.overscan_lines`, `performance.frame_budget_ms`, `performance.layout_cache_memory_mb`
- **Hot-reload**: Subscribes to configuration change callbacks; updates PerfConfig and resizes caches without restart

### With `ff-logging` (Wave 0 — upstream dependency)

- **Dependency direction**: ff-large-file-performance depends on ff-logging
- **Usage**: DEBUG-level logging for invalidation_count metric, WARN-level for frame budget overruns, INFO-level for cache resize events
- **Error standards**: All log messages prefixed with `[large-file-perf]`

### With `ff-syntax-highlighting` (Wave 7 — upstream dependency)

- **Dependency direction**: ff-large-file-performance depends on ff-syntax-highlighting
- **API consumed**: Style slot assignments per character range for a line — determines which PositionCache entries to look up/store
- **Invalidation trigger**: Style re-highlighting on a line triggers `InvalidationEvent::StyleChanged` for that line's LineLayout entry

### With `ff-theme-and-appearance` (Wave 6 — upstream dependency)

- **Dependency direction**: ff-large-file-performance depends on ff-theme-and-appearance
- **API consumed**: Font metrics (family, size, weight, style) per style slot — used to construct Font_Metrics_Key
- **Invalidation trigger**: Any change to font metrics triggers full PositionCache clear and LineLayoutCache invalidation to `Invalid`

### With `ff-view-zoom` (Wave 9 — integration)

- **Dependency direction**: ff-large-file-performance subscribes to ff-view-zoom notifications
- **Invalidation trigger**: Zoom level change invalidates all measurements (character widths scale with zoom), triggering full PositionCache clear and LineLayoutCache invalidation

---

## Correctness Properties

The following properties are designed for verification using the `proptest` crate. Each property maps to one or more acceptance criteria from `requirements.md`.

### Property 1: PositionCache Determinism

**Statement**: For any (style_slot, text) pair, storing positions and then looking them up returns the identical positions that were stored, regardless of cache state or eviction history.

**Validates: Requirements 2.6**

```
∀ style: StyleSlot, text: String, positions: Vec<XPosition>
  cache.store(style, text, unicode, positions)
  ⟹ cache.lookup(style, text, unicode, output) == true
     ∧ output == positions
  (provided no intervening clear() or eviction of this entry)
```

### Property 2: PositionCache Two-Way Eviction Correctness

**Statement**: When the cache is full and a new entry is inserted, exactly one of the two probe candidates is evicted — specifically the one with the lower clock value. The surviving entry remains retrievable.

**Validates: Requirements 2.2**

```
∀ entries filling cache to capacity, new_entry
  insert(new_entry) ⟹
    evicted_entry.clock ≤ surviving_entry.clock
    ∧ lookup(surviving_entry.style, surviving_entry.text) == true
```

### Property 3: Clock Wrap Safety

**Statement**: After clock wraps (exceeds u16::MAX), all entry clocks are reset to 1, and subsequent lookups still return correct cached data (no stale entries appear fresher than new ones).

**Validates: Requirements 2.7**

```
∀ sequence of N store operations where N > u16::MAX
  after_wrap: ∀ entry in cache: entry.clock ≥ 1
  ∧ no entry with clock == 0 exists
  ∧ lookup correctness maintained for all surviving entries
```

### Property 4: LineLayoutCache LRU Ordering

**Statement**: When the cache is at capacity and a new entry is inserted, the least-recently-used entry is evicted first, except that caret-line and visible-viewport entries are prioritised for retention.

**Validates: Requirements 3.7**

```
∀ access_sequence, new_insert at capacity
  evicted_line = argmin(last_access) among non-prioritised entries
  ∧ (caret_line is not evicted unless all entries are caret/visible)
```

### Property 5: LineLayout Validity Transitions

**Statement**: Validity levels only decrease (Invalid < CheckTextAndStyle < Positions < Lines). An entry at `Lines` can be downgraded to any lower level, but never upgraded without explicit recomputation.

**Validates: Requirements 3.5**

```
∀ entry, invalidation_event
  entry.validity_after ≤ entry.validity_before
  ∨ (explicit recomputation occurred ∧ entry.validity_after == Lines)
```

### Property 6: Chunked Measurement Coverage

**Statement**: For any long line and horizontal scroll position, the measured chunk always fully covers the horizontal viewport (all characters visible to the user have measured x-positions).

**Validates: Requirements 1.2, 1.4**

```
∀ line_length > threshold, horizontal_offset, viewport_width
  let chunk = compute_visible_chunk(...)
  ⟹ chunk.start ≤ first_visible_char
     ∧ chunk.end ≥ last_visible_char
```

### Property 7: Chunk Overlap Reuse

**Statement**: When extending a chunk due to horizontal scroll, all character positions in the overlap region between old and new chunks are preserved exactly (not re-measured).

**Validates: Requirements 1.4**

```
∀ old_chunk, new_chunk where overlap(old_chunk, new_chunk) ≠ ∅
  ∀ offset in overlap:
    new_layout.positions[offset] == old_layout.positions[offset]
```

### Property 8: Render Chunk Partition Completeness

**Statement**: Subdividing a character range into render chunks produces a complete partition — the union of all chunks equals the original range with no gaps and no overlaps.

**Validates: Requirements 1.6**

```
∀ range: ChunkRange
  let chunks = chunk_renderer.subdivide(range)
  ⟹ chunks[0].start == range.start
     ∧ chunks[last].end == range.end
     ∧ ∀ i: chunks[i].end == chunks[i+1].start
     ∧ ∀ chunk: chunk.end - chunk.start ≤ render_chunk_size
```

### Property 9: Viewport-Only Rendering Complexity

**Statement**: The number of lines measured/rendered per frame is bounded by `visible_count + 2 * overscan_lines`, independent of total document line count.

**Validates: Requirements 4.4**

```
∀ doc_line_count, visible_count, overscan_lines
  lines_measured_per_frame ≤ visible_count + 2 * overscan_lines
```

### Property 10: Invalidation Idempotence

**Statement**: Applying the same invalidation event multiple times has the same effect as applying it once — cache state after N applications equals cache state after 1 application.

**Validates: Requirements 9.7**

```
∀ event: InvalidationEvent, initial_state
  apply(event, initial_state) == apply(event, apply(event, initial_state))
```

### Property 11: Memory Budget Enforcement

**Statement**: After any sequence of layout insertions, the total memory used by LineLayoutCache never exceeds the configured memory budget. When budget is exceeded, entries are evicted until usage drops below 90% of budget.

**Validates: Requirements 7.4, 7.5**

```
∀ insertion_sequence
  cache.memory_used() ≤ config.memory_budget_bytes()
  ∨ (eviction_triggered ∧ post_eviction_memory ≤ 0.9 * budget)
```

### Property 12: Lazy Computation Boundary

**Statement**: No line outside (viewport + overscan + 1 for ensure_layout_to targets) is ever measured. The measured frontier never advances beyond available_lines.

**Validates: Requirements 5.1, 5.5**

```
∀ layout_request_sequence
  ∀ measured_line:
    measured_line ∈ viewport_range ∪ overscan_range ∪ explicit_targets
    ∧ measured_line ≤ available_lines
```

### Property 13: Cache Hit/Miss Visual Equivalence

**Statement**: The visual output (x-positions returned) for a given line is identical whether the data came from cache or from fresh measurement — caching never alters rendered positions.

**Validates: Requirements 2.6, 3.5**

```
∀ line, style_runs, text_content
  layout_from_cache(line) == layout_from_fresh_measurement(line)
  (pixel-exact equality of all XPosition values)
```

### Property 14: Configuration Clamping

**Statement**: All configurable values are clamped to their specified ranges. No out-of-range configuration value propagates to runtime behaviour.

**Validates: Requirements 1.5, 1.9, 2.3, 4.2, 4.6**

```
∀ raw_value: i64
  let clamped = ConfigType::new(raw_value)
  ⟹ ConfigType::MIN ≤ clamped.0 ≤ ConfigType::MAX
```

### Property 15: Scroll Velocity Mode Switching

**Statement**: When scroll velocity exceeds 20 lines/frame, simplified measurement is used. When velocity drops below 5 lines/frame, exact measurement resumes. The system never uses simplified measurement when scrolling is slow.

**Validates: Requirements 8.4, 8.5**

```
∀ scroll_event_sequence
  velocity > 20 ⟹ simplified_mode_active
  velocity < 5 ⟹ exact_mode_active
  stopped_for_100ms ⟹ refinement_pass_triggered
```

---

## Performance Considerations

### Hot Path Optimisation

The critical path for rendering is: `request_layout → cache lookup → return`. This path must complete in O(1) for cached lines:

1. **LineLayoutCache lookup**: HashMap get by u64 key — O(1) amortized
2. **PositionCache lookup**: Two hash probes with string comparison — O(text_length) but text fragments are short (keyword-length)
3. **No allocation on cache hit**: Cached data is returned by reference (`Arc<LineLayout>`) — zero allocation on hot path

### Memory Layout

- `PositionCache` entries use `Vec<XPosition>` (contiguous f64 array) for cache-friendly sequential access during rendering
- `LineLayout` positions stored in a single contiguous `Vec<XPosition>` to avoid pointer chasing
- `LineLayoutCache` eviction targets 90% of budget to avoid thrashing at the boundary

### Concurrency Strategy

- **PositionCache**: Protected by `Mutex` (short critical sections: hash probe + memcpy). Lock held for ~microseconds.
- **LineLayoutCache**: Protected by `RwLock` — multiple concurrent readers (render thread, status queries), exclusive writes (invalidation, insertion). Lock-free reads via `Arc<LineLayout>` after initial lookup.
- **InvalidationCoordinator**: Only accessed from the main thread (event dispatch) — no lock contention.
- **LayoutWorkSource**: Acquires write locks on caches during idle time only (no UI-thread contention because idle means no rendering is active).

### Fallback Strategy for Cache Misses During Fast Scroll

When scrolling fast and layouts are not cached:
1. Use monospace approximation (average_char_width × char_count) for immediate rendering
2. Schedule accurate measurement for idle time
3. On scroll stop: measure accurately, repaint only if visual difference detected
4. This ensures zero-stall scrolling at the cost of brief visual imprecision during fast scroll

---

## Testing Strategy

### Unit Tests

- PositionCache: store/lookup round-trip, two-way probing, clock wrapping, eviction ordering, clear
- LineLayoutCache: insert/get, LRU eviction, memory budget enforcement, level transitions, invalidation
- ChunkRenderer: subdivision completeness, boundary cases (chunk_size divides evenly, remainder)
- LineWidthCache: known width storage, estimation from average, invalidation
- InvalidationCoordinator: event batching, coalescing, dispatch to correct caches
- ScrollPredictor: direction detection, velocity calculation, mode switching
- PerfConfig: clamping for all parameters, from_config with mock

### Integration Tests

- End-to-end layout request: mock Surface + mock LineContentProvider → verify correct layout returned
- Invalidation flow: simulate edit → verify affected cache entries invalidated → verify re-measurement on next request
- Long-line chunked measurement: create 50,000-char line → request layout at various horizontal offsets → verify only chunks are measured
- Viewport rendering: create 1M-line mock document → request visible range → verify only visible+overscan lines touched
- Progressive loading: simulate partially-loaded document → verify lines beyond frontier return `LineNotAvailable` error

### Property-Based Tests

All 15 properties from Section 8 are implemented using `proptest` with a minimum of 100 cases per property. Each property test carries a requirement coverage annotation:

```rust
// Feature: large-file-performance, Property 1: PositionCache Determinism
// Validates: Requirement 2 AC 6
proptest! {
    #[test]
    fn position_cache_store_lookup_roundtrip(
        style in 0u16..256,
        text in "[a-zA-Z0-9 ]{1,64}",
        positions in proptest::collection::vec(0.0f64..1000.0, 1..64),
    ) {
        // ... property assertion
    }
}
```

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Primary Component | Property Test |
|-------------|----------|-------------------|---------------|
| Req 1 AC 1 | Long-line detection | LazyLayoutManager | — |
| Req 1 AC 2 | Chunked measurement range | LazyLayoutManager, ChunkRenderer | Property 6 |
| Req 1 AC 3 | Partial LineLayout storage | LineLayout | — |
| Req 1 AC 4 | Chunk extend/shift on scroll | LayoutPerformanceManager | Property 7 |
| Req 1 AC 5 | Threshold configuration | PerfConfig | Property 14 |
| Req 1 AC 6 | Render chunk subdivision | ChunkRenderer | Property 8 |
| Req 1 AC 7 | Lazy total width estimation | LineWidthCache | — |
| Req 1 AC 8 | JIT measurement within budget | LazyLayoutManager | — |
| Req 1 AC 9 | Overscan config | PerfConfig | Property 14 |
| Req 2 AC 1 | PositionCache keying | PositionCache | Property 1 |
| Req 2 AC 2 | Two-way probing + eviction | PositionCache | Property 2 |
| Req 2 AC 3 | Size configuration | PerfConfig | Property 14 |
| Req 2 AC 4 | Entry structure | PositionCacheEntry | — |
| Req 2 AC 5 | Thread safety | PositionCache (Mutex) | — |
| Req 2 AC 6 | Cache hit behaviour | PositionCache | Property 1 |
| Req 2 AC 7 | Clock wrap | PositionCache | Property 3 |
| Req 2 AC 8 | Clear method | PositionCache | — |
| Req 2 AC 9 | Font metrics key | InvalidationCoordinator | — |
| Req 3 AC 1 | LineLayoutCache existence | LineLayoutCache | — |
| Req 3 AC 2 | Cache levels | LineLayoutCache, CacheLevel | — |
| Req 3 AC 3 | Auto-level selection | LayoutPerformanceManager | — |
| Req 3 AC 4 | LineLayout structure | LineLayout | — |
| Req 3 AC 5 | Validity levels | ValidLevel | Property 5 |
| Req 3 AC 6 | Edit invalidation | InvalidationCoordinator | — |
| Req 3 AC 7 | LRU eviction | LineLayoutCache | Property 4 |
| Req 3 AC 8 | Capacity calculation | LineLayoutCache | — |
| Req 3 AC 9 | Reuse validation | LineLayout | — |
| Req 4 AC 1 | Viewport-only rendering | LayoutPerformanceManager | Property 9 |
| Req 4 AC 2 | Overscan buffer | LazyLayoutManager | Property 14 |
| Req 4 AC 3 | Scroll from overscan cache | LayoutWorkSource | — |
| Req 4 AC 4 | O(visible_count) complexity | LayoutPerformanceManager | Property 9 |
| Req 4 AC 5 | Full repaint scope | LayoutPerformanceManager | — |
| Req 4 AC 6 | Frame budget enforcement | LazyLayoutManager | — |
| Req 4 AC 7 | Significant line tracking | LineLayoutCache | — |
| Req 4 AC 8 | Viewport-change notification | LayoutPerformanceManager | — |
| Req 5 AC 1 | No measurement outside range | LazyLayoutManager | Property 12 |
| Req 5 AC 2 | EnsureLayoutTo method | LazyLayoutManager | — |
| Req 5 AC 3 | GOTO/FIND navigation | LayoutPerformanceManager | — |
| Req 5 AC 4 | Progressive loading coordination | LazyLayoutManager | Property 12 |
| Req 5 AC 5 | Measured frontier tracking | LazyLayoutManager | Property 12 |
| Req 5 AC 6 | Predictive pre-fetch | LayoutWorkSource, ScrollPredictor | — |
| Req 5 AC 7 | Unmeasured count exposure | LazyLayoutManager, StatusIndicator | — |
| Req 6 AC 1 | File size indicator | StatusIndicator | — |
| Req 6 AC 2 | Line count display | StatusIndicator | — |
| Req 6 AC 3 | Loading progress | StatusIndicator | — |
| Req 6 AC 4 | Layout progress | StatusIndicator | — |
| Req 6 AC 5 | Threshold suppression | StatusIndicator | — |
| Req 6 AC 6 | Completion fade | StatusIndicator | — |
| Req 6 AC 7 | Paused state | StatusIndicator | — |
| Req 7 AC 1 | Line access via document-model | LineContentProvider | — |
| Req 7 AC 2 | Borrowed references | LineContentProvider | — |
| Req 7 AC 3 | Progressive loading coordination | LazyLayoutManager | — |
| Req 7 AC 4 | Memory budget | LineLayoutCache | Property 11 |
| Req 7 AC 5 | Budget eviction to 90% | LineLayoutCache | Property 11 |
| Req 7 AC 6 | Sub-range access for long lines | LineContentProvider | — |
| Req 7 AC 7 | 64-bit line indexing | All types (u64) | — |
| Req 8 AC 1 | 60fps scrolling | LayoutPerformanceManager | — |
| Req 8 AC 2 | O(1) per-line cached rendering | LineLayoutCache | — |
| Req 8 AC 3 | Simplified measurement fallback | LazyLayoutManager | — |
| Req 8 AC 4 | Velocity-based strategy | ScrollPredictor | Property 15 |
| Req 8 AC 5 | Refinement on scroll stop | LayoutPerformanceManager | Property 15 |
| Req 8 AC 6 | Non-blocking scroll handler | LazyLayoutManager | — |
| Req 8 AC 7 | Horizontal 60fps | ChunkRenderer + PositionCache | — |
| Req 8 AC 8 | Overscan pre-computation | LayoutWorkSource | — |
| Req 9 AC 1 | Single-line invalidation | InvalidationCoordinator | — |
| Req 9 AC 2 | Line-count change invalidation | InvalidationCoordinator | — |
| Req 9 AC 3 | Font change full clear | InvalidationCoordinator | — |
| Req 9 AC 4 | Zoom change full clear | InvalidationCoordinator | — |
| Req 9 AC 5 | Resize downgrade | InvalidationCoordinator | — |
| Req 9 AC 6 | Style change per-line | InvalidationCoordinator | — |
| Req 9 AC 7 | Batch coalescing | InvalidationCoordinator | Property 10 |
| Req 9 AC 8 | Visibility change no-invalidate | InvalidationCoordinator | — |
| Req 9 AC 9 | Invalidation count metric | InvalidationCoordinator | — |
