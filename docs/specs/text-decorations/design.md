# Design Document: Text Decorations (`ff-text-decorations`)

## Overview

The `ff-text-decorations` crate is the **visual overlay subsystem** for FileForgeWorkbench. It manages transient, overlapping decorations applied on top of (or beneath) rendered text to communicate semantic information — search matches, diagnostic errors, change history, bookmarks, and custom plugin indicators.

### Purpose

- Store per-character indicator values using run-length encoding for memory-efficient sparse coverage
- Define 23 indicator visual styles adapted from Scintilla to egui rendering primitives
- Manage line markers for gutter/margin annotations (bookmarks, change history, fold markers)
- Synchronize decoration positions with document edits (insert/delete)
- Provide theme-driven colour and style configuration with hot-reload support
- Support hover interaction for dynamic indicators (tooltips, hyperlinks)
- Expose a rendering-technology-agnostic query API for the viewport renderer
- Allocate indicator numbers to prevent conflicts between multiple producers

### Position in Architecture

```
Wave 6 — UI and Rendering

┌──────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                    │
│   Viewport Renderer — draws decorations using painter API     │
├──────────────────────────────────────────────────────────────┤
│          THIS CRATE: ff-text-decorations ← Wave 6             │
│   Indicator storage, line markers, hover state, queries       │
├──────────────────────────────────────────────────────────────┤
│  Upstream:                                                    │
│    ff-document-model (Wave 4) — buffer positions, edit events │
│    ff-edit-operations (Wave 4) — edit notifications           │
│    ff-undo-redo-transactions (Wave 4) — undo sync             │
│    ff-find-and-replace (Wave 5) — match highlighting producer │
│    ff-theme (Wave 6, peer) — colour/style configuration       │
│    ff-configuration-system (Wave 2) — hot-reload              │
│    ff-command (Wave 2) — bookmark command registration         │
├──────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                      │
└──────────────────────────────────────────────────────────────┘
```


### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: No direct filesystem access — decoration data is purely in-memory, indexed by document buffer positions
- **GUI Independence (Req 2)**: Zero GUI dependencies — stores decoration data and exposes query APIs; actual rendering is performed by the shell layer
- **Command-Driven (Req 4)**: Bookmark operations (toggle, next, previous, clear) registered as commands in `ff-command`
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-text-decorations`
- **Error Message Standards (Req 8)**: All errors follow `[decorations] operation: description` format

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Producers [Decoration Producers]
        FIND[ff-find-and-replace<br/>search match highlighting]
        LANG[Language Service / Plugins<br/>diagnostic underlines]
        EDIT[ff-edit-operations<br/>change history tracking]
        USER[User Commands<br/>bookmark toggle]
    end

    subgraph ff-text-decorations [ff-text-decorations Crate]
        DL[DecorationList<br/>per-document indicator storage]
        RLE[RunStyles&lt;T&gt;<br/>run-length encoded values]
        MS[MarkerStore<br/>per-line marker bitmasks]
        IC[IndicatorCatalogue<br/>style + properties for each indicator]
        HS[HoverState<br/>mouse tracking for dynamic indicators]
        IA[IndicatorAllocator<br/>namespace + number management]
        ES[EditSync<br/>insert_space / delete_range]
        TI[ThemeIntegration<br/>colour/style resolution]
        RP[RenderingProvider<br/>query API for viewport]
    end

    subgraph Upstream [Upstream Dependencies]
        DOC[ff-document-model<br/>buffer positions, line count]
        THEME[ff-theme<br/>palette, hot-reload events]
        CMD[ff-command<br/>bookmark command registration]
        UNDO[ff-undo-redo-transactions<br/>undo/redo event sync]
        CFG[ff-configuration-system<br/>indicator config overrides]
        LOG[ff-logging]
    end

    FIND -->|fill_range: search matches| DL
    LANG -->|fill_range: diagnostics| DL
    EDIT -->|marker_add: change history| MS
    USER -->|marker_add/delete: bookmarks| MS

    DL --> RLE
    DL --> ES
    DL --> HS
    DL --> IA
    MS --> IA

    IC --> TI
    TI --> THEME
    IA --> CFG
    RP --> DL
    RP --> MS
    RP --> IC

    ES --> DOC
    ES --> UNDO
    DL --> LOG
end
```


### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **DecorationList** | Per-document collection of all active indicator decorations, indexed by indicator number. Provides aggregate queries and delegates to individual `Decoration` instances. |
| **RunStyles\<T\>** | Generic run-length-encoded storage: stores (value, length) pairs, supports split/merge on insert/delete, provides O(log n) position lookup via binary search on cumulative lengths. |
| **Decoration** | Single-indicator storage wrapping `RunStyles<u32>` for one indicator number within a document. |
| **MarkerStore** | Per-line marker bitmask storage: maps document line numbers to 32-bit marker masks. Adjusts line indices on insert/delete. |
| **IndicatorCatalogue** | Registry of indicator style definitions (style enum, colours, alpha, stroke width, under, hover state) for all 44 indicator slots. Sourced from theme + configuration. |
| **HoverState** | Tracks current mouse position and determines which dynamic indicators need redraw on hover transitions. Emits decoration-click events. |
| **IndicatorAllocator** | Manages indicator number namespaces (lexer 0–7, container 8–31, IME 32–35, history 36–43). Provides allocation API for plugins. |
| **EditSync** | Receives edit notifications and applies `insert_space` / `delete_range` to all active decorations and marker positions. |
| **ThemeIntegration** | Listens for theme-change events and refreshes all indicator/marker colours from the new palette. |
| **RenderingProvider** | Implements the `DecorationRenderer` trait exposing query methods for the viewport painter. |

### Data Flow: Search Highlighting

```
1. User types FIND 'text' → ff-find-and-replace executes search
2. FindEngine computes HighlightAllResult (list of MatchRange)
3. FindEngine calls DecorationList::fill_range(INDICATOR_SEARCH_ALL, start, 1, length)
   for each match, and fill_range(INDICATOR_SEARCH_CURRENT, ...) for the focused match
4. Viewport renderer queries RenderingProvider for visible range
5. RenderingProvider returns iterator of (indicator_number, start, end, value)
6. Renderer draws each indicator using IndicatorCatalogue style definitions
7. On RFIND: clear old current-match, fill new current-match, old reverts to all-matches
```

### Data Flow: Edit Synchronization

```
1. User inserts text at position P with length L
2. ff-edit-operations emits edit event (Insert, position=P, length=L)
3. EditSync receives event and calls DecorationList::insert_space(P, L)
4. Each active Decoration splits the run containing P:
   - Run before P retains original value
   - New run of length L inserted with value 0 (no decoration)
   - Run after P+L retains original value at shifted positions
5. MarkerStore shifts all markers on lines after the insertion point
6. On undo: matching delete_range(P, L) reverses the operation
```

---

## Module Structure

```
crates/ff-text-decorations/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── indicator_style.rs          # IndicatorStyle enum (23 variants)
│   ├── indicator.rs                # IndicatorConfig: style + properties per slot
│   ├── catalogue.rs                # IndicatorCatalogue: all 44 slots
│   ├── allocator.rs                # IndicatorAllocator: namespace management
│   ├── run_styles.rs               # RunStyles<T>: generic RLE storage
│   ├── decoration.rs               # Decoration: single-indicator RLE wrapper
│   ├── decoration_list.rs          # DecorationList: per-document aggregate
│   ├── marker_symbol.rs            # MarkerSymbol enum (31 geometric shapes)
│   ├── line_marker.rs              # LineMarkerConfig: per-marker-number properties
│   ├── marker_store.rs             # MarkerStore: per-line bitmask storage
│   ├── edit_sync.rs                # EditSync: insert_space / delete_range
│   ├── hover.rs                    # HoverState: mouse tracking, click events
│   ├── theme_integration.rs        # ThemeIntegration: palette reload
│   ├── rendering.rs                # DecorationRenderer trait + RenderingProvider
│   ├── dpi.rs                      # PixelAligner: high-DPI coordinate snapping
│   ├── commands.rs                 # Bookmark commands registration
│   ├── constants.rs                # Well-known indicator/marker number constants
│   ├── error.rs                    # DecorationError enum
│   └── events.rs                   # DecorationEvent enum (click, hover)
└── tests/
    ├── run_styles_tests.rs         # RunStyles RLE property + unit tests
    ├── decoration_tests.rs         # Decoration fill_range, value_at tests
    ├── decoration_list_tests.rs    # Aggregate queries, lazy creation/removal
    ├── edit_sync_tests.rs          # insert_space / delete_range correctness
    ├── marker_store_tests.rs       # Line marker add/delete/move tests
    ├── allocator_tests.rs          # Indicator number allocation tests
    ├── catalogue_tests.rs          # Theme integration, hot-reload tests
    ├── hover_tests.rs              # Hover state transition tests
    ├── integration.rs              # End-to-end with mock document
    └── property_tests.rs           # Cross-cutting proptest properties
```


---

## Data Models

### Core Newtypes and Enums

```rust
/// Character position within the document buffer (0-based byte offset).
/// Re-exported from ff-document-model.
pub use ff_document_model::BytePosition;

/// 1-based document line number.
/// Re-exported from ff-document-model.
pub use ff_document_model::LineNumber;

/// Indicator number (0–43).
///
/// Addresses: Requirement 13
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IndicatorNumber(pub u8);

impl IndicatorNumber {
    pub const MAX: u8 = 43;

    pub fn new(n: u8) -> Option<Self> {
        if n <= Self::MAX { Some(Self(n)) } else { None }
    }
}

/// Marker number (0–31).
///
/// Addresses: Requirement 9 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MarkerNumber(pub u8);

impl MarkerNumber {
    pub const MAX: u8 = 31;

    pub fn new(n: u8) -> Option<Self> {
        if n <= Self::MAX { Some(Self(n)) } else { None }
    }
}

/// Bitmask of active markers on a line (bits 0–31).
///
/// Addresses: Requirement 9 AC 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarkerMask(pub u32);

impl MarkerMask {
    pub fn has(&self, marker: MarkerNumber) -> bool {
        (self.0 >> marker.0) & 1 == 1
    }

    pub fn set(&mut self, marker: MarkerNumber) {
        self.0 |= 1 << marker.0;
    }

    pub fn clear(&mut self, marker: MarkerNumber) {
        self.0 &= !(1 << marker.0);
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}
```


### Indicator Style Enum

```rust
/// Visual style for an indicator decoration.
///
/// Addresses: Requirement 1 AC 1–24
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IndicatorStyle {
    Plain,
    Squiggle,
    TT,
    Diagonal,
    Strike,
    Hidden,
    Box,
    RoundBox,
    StraightBox,
    Dash,
    Dots,
    SquiggleLow,
    DotBox,
    SquigglePixmap,
    CompositionThick,
    CompositionThin,
    FullBox,
    TextFore,
    Point,
    PointCharacter,
    Gradient,
    GradientCentre,
    PointTop,
}
```

### Indicator Configuration

```rust
/// RGBA colour representation (0–255 per component).
///
/// Addresses: Requirement 15 (theme integration)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColourRGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Flags controlling indicator behaviour.
///
/// Addresses: Requirement 2 AC 8
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndicatorFlags {
    /// When true, colour is derived from the indicator value (lower 24 bits = RGB).
    pub value_fore: bool,
}

/// Style + colour state for normal or hover appearance.
///
/// Addresses: Requirement 2 AC 6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleAppearance {
    pub style: IndicatorStyle,
    pub fore: ColourRGBA,
}

/// Complete configuration for a single indicator slot.
///
/// Addresses: Requirement 2 AC 1–9
#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorConfig {
    /// Normal-state appearance.
    pub normal: StyleAppearance,
    /// Hover-state appearance (if different from normal, indicator is "dynamic").
    pub hover: StyleAppearance,
    /// Whether indicator renders below text glyphs.
    pub under: bool,
    /// Interior fill opacity for box-style indicators (0–255, default 30).
    pub fill_alpha: u8,
    /// Border/outline opacity for box-style indicators (0–255, default 50).
    pub outline_alpha: u8,
    /// Line thickness in logical pixels (default 1.0).
    pub stroke_width: f32,
    /// Behaviour flags (ValueFore, etc.).
    pub flags: IndicatorFlags,
}

impl IndicatorConfig {
    /// Returns true when the hover state differs from normal state.
    ///
    /// Addresses: Requirement 2 AC 7
    pub fn is_dynamic(&self) -> bool {
        self.normal != self.hover
    }
}
```


### Marker Symbol Enum

```rust
/// Geometric shape for a line marker rendered in the gutter margin.
///
/// Addresses: Requirement 9 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MarkerSymbol {
    Circle,
    RoundRect,
    Arrow,
    SmallRect,
    ShortArrow,
    Empty,
    ArrowDown,
    Minus,
    Plus,
    VLine,
    LCorner,
    TCorner,
    BoxPlus,
    BoxPlusConnected,
    BoxMinus,
    BoxMinusConnected,
    LCornerCurve,
    TCornerCurve,
    CirclePlus,
    CirclePlusConnected,
    CircleMinus,
    CircleMinusConnected,
    Background,
    DotDotDot,
    Arrows,
    FullRect,
    LeftRect,
    Underline,
    Bookmark,
    VerticalBookmark,
    Bar,
    /// Custom RGBA pixmap image.
    Pixmap(PixmapId),
}

/// Opaque identifier for a registered custom pixmap marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixmapId(pub u32);
```

### Line Marker Configuration

```rust
/// Rendering layer for markers.
///
/// Addresses: Requirement 9 AC 4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkerLayer {
    /// Render in the base layer (behind text).
    #[default]
    Base,
    /// Render in the overlay layer (above everything except selection).
    Overlay,
}

/// Complete configuration for a single marker number slot.
///
/// Addresses: Requirement 9 AC 2–6
#[derive(Debug, Clone, PartialEq)]
pub struct LineMarkerConfig {
    /// The geometric shape or pixmap to render.
    pub symbol: MarkerSymbol,
    /// Foreground colour (used for outlines and geometric shapes).
    pub fore: ColourRGBA,
    /// Background fill colour.
    pub back: ColourRGBA,
    /// Background colour when the line is selected.
    pub back_selected: ColourRGBA,
    /// Opacity (0–255).
    pub alpha: u8,
    /// Rendering layer (base or overlay).
    pub layer: MarkerLayer,
    /// Stroke width for geometric outlines.
    pub stroke_width: f32,
}
```

### Run-Length Encoded Storage

```rust
/// A single run in the RLE storage: a contiguous range of positions with the same value.
///
/// Addresses: Requirement 3 AC 1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run<T: Clone + Eq> {
    pub value: T,
    pub length: u64,
}

/// Generic run-length-encoded sequence supporting efficient position queries and edits.
///
/// Addresses: Requirement 3 AC 1, 10
pub struct RunStyles<T: Clone + Eq + Default> {
    /// Ordered sequence of runs; total of all lengths == document length.
    runs: Vec<Run<T>>,
    /// Cached cumulative lengths for O(log n) binary search.
    cumulative: Vec<u64>,
    /// Total length (sum of all run lengths).
    total_length: u64,
}

impl<T: Clone + Eq + Default> RunStyles<T> {
    /// Create storage for a document of the given initial length (all values = T::default()).
    pub fn new(initial_length: u64) -> Self;

    /// Get the value at the given position.
    /// O(log n) via binary search on cumulative lengths.
    pub fn value_at(&self, position: u64) -> T;

    /// Get the start position of the run containing `position`.
    pub fn run_start(&self, position: u64) -> u64;

    /// Get the end position (exclusive) of the run containing `position`.
    pub fn run_end(&self, position: u64) -> u64;

    /// Set all positions in [position, position+length) to `value`.
    /// Returns true if any values actually changed.
    /// Merges adjacent runs with the same value.
    pub fn fill_range(&mut self, position: u64, value: T, length: u64) -> bool;

    /// Insert `length` positions with T::default() at `position`.
    /// Splits the run containing position; shifts subsequent runs rightward.
    ///
    /// Addresses: Requirement 4 AC 1, 3, 4
    pub fn insert_space(&mut self, position: u64, length: u64);

    /// Remove `length` positions starting at `position`.
    /// Merges the runs on either side of the deleted range.
    ///
    /// Addresses: Requirement 4 AC 2
    pub fn delete_range(&mut self, position: u64, length: u64);

    /// Returns true if the entire sequence has T::default() values (effectively empty).
    pub fn is_empty(&self) -> bool;

    /// Total length of the sequence.
    pub fn total_length(&self) -> u64;

    /// Iterator over runs intersecting [start, end).
    pub fn runs_in_range(&self, start: u64, end: u64) -> impl Iterator<Item = (u64, &Run<T>)>;
}
```


### Change History State

```rust
/// Change history state for a line.
///
/// Addresses: Requirement 7 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeHistoryState {
    /// Line has unsaved modifications.
    Modified,
    /// Line was modified and then saved.
    Saved,
    /// Line was reverted to original file content.
    RevertedToOrigin,
    /// Line was reverted to a previously modified state.
    RevertedToModified,
}

/// Change type for character-level history indicators.
///
/// Addresses: Requirement 7 AC 6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Insertion,
    Deletion,
}
```

---

## Public API Surface

### DecorationList

```rust
/// Per-document aggregate of all active indicator decorations.
///
/// Addresses: Requirement 3 AC 2–9
pub struct DecorationList {
    /// Lazily populated map: indicator_number → Decoration storage.
    decorations: HashMap<IndicatorNumber, RunStyles<u32>>,
    /// Document length for invariant enforcement.
    document_length: u64,
}

impl DecorationList {
    /// Create a new DecorationList for a document of the given length.
    pub fn new(document_length: u64) -> Self;

    /// Get the indicator value at a position for a specific indicator.
    /// Returns 0 if no decoration exists for that indicator.
    ///
    /// Addresses: Requirement 3 AC 5
    pub fn value_at(&self, indicator: IndicatorNumber, position: u64) -> u32;

    /// Get the start of the run containing `position` for the given indicator.
    ///
    /// Addresses: Requirement 3 AC 6
    pub fn start_run(&self, indicator: IndicatorNumber, position: u64) -> u64;

    /// Get the end (exclusive) of the run containing `position`.
    ///
    /// Addresses: Requirement 3 AC 7
    pub fn end_run(&self, indicator: IndicatorNumber, position: u64) -> u64;

    /// Set indicator values for a contiguous range.
    /// Creates the Decoration lazily if this is the first non-zero write.
    /// Removes the Decoration if all values become zero.
    ///
    /// Addresses: Requirement 3 AC 3, 4, 8
    pub fn fill_range(
        &mut self,
        indicator: IndicatorNumber,
        position: u64,
        value: u32,
        length: u64,
    ) -> bool;

    /// Returns a bitmask of all indicator numbers with non-zero values at `position`.
    ///
    /// Addresses: Requirement 3 AC 9
    pub fn all_on_for(&self, position: u64) -> u64;

    /// Insert space at position across all active decorations.
    ///
    /// Addresses: Requirement 4 AC 1
    pub fn insert_space(&mut self, position: u64, length: u64);

    /// Delete a range across all active decorations.
    ///
    /// Addresses: Requirement 4 AC 2
    pub fn delete_range(&mut self, position: u64, length: u64);

    /// Clear all values for indicators in the lexer range (0–7).
    ///
    /// Addresses: Requirement 13 AC 7
    pub fn delete_lexer_decorations(&mut self);

    /// Query all active indicator ranges intersecting [start, end).
    /// Returns an iterator of (indicator_number, run_start, run_end, value).
    ///
    /// Addresses: Requirement 14 AC 2
    pub fn indicators_in_range(
        &self,
        start: u64,
        end: u64,
    ) -> Vec<(IndicatorNumber, u64, u64, u32)>;

    /// Number of active (non-empty) decorations.
    pub fn active_count(&self) -> usize;
}
```


### MarkerStore

```rust
/// Per-document storage of line marker assignments.
///
/// Addresses: Requirement 9 AC 7–10
pub struct MarkerStore {
    /// Map from document line number to marker bitmask.
    /// Lines without markers are not present in the map.
    markers: BTreeMap<u64, MarkerMask>,
    /// Total line count (for bounds checking).
    line_count: u64,
}

impl MarkerStore {
    /// Create a new MarkerStore for a document with the given line count.
    pub fn new(line_count: u64) -> Self;

    /// Add a marker to a line.
    ///
    /// Addresses: Requirement 9 AC 7
    pub fn marker_add(&mut self, line: u64, marker: MarkerNumber);

    /// Remove a marker from a line.
    ///
    /// Addresses: Requirement 9 AC 7
    pub fn marker_delete(&mut self, line: u64, marker: MarkerNumber);

    /// Delete all markers with the given number from all lines.
    pub fn marker_delete_all(&mut self, marker: MarkerNumber);

    /// Get the marker bitmask for a line.
    ///
    /// Addresses: Requirement 9 AC 7
    pub fn marker_get(&self, line: u64) -> MarkerMask;

    /// Find the next line at or after `from_line` with any marker in `mask`.
    ///
    /// Addresses: Requirement 9 AC 8
    pub fn marker_next(&self, from_line: u64, mask: MarkerMask) -> Option<u64>;

    /// Find the previous line at or before `from_line` with any marker in `mask`.
    ///
    /// Addresses: Requirement 9 AC 9
    pub fn marker_previous(&self, from_line: u64, mask: MarkerMask) -> Option<u64>;

    /// Shift all markers on lines >= `from_line` by `delta` lines (for line insertion).
    ///
    /// Addresses: Requirement 9 AC 10
    pub fn lines_inserted(&mut self, from_line: u64, count: u64);

    /// Remove markers on deleted lines and shift subsequent lines.
    ///
    /// Addresses: Requirement 9 AC 10
    pub fn lines_deleted(&mut self, from_line: u64, count: u64);

    /// Query all lines with bookmark markers (convenience for bookmark list).
    ///
    /// Addresses: Requirement 8 AC 5
    pub fn all_lines_with_marker(&self, marker: MarkerNumber) -> Vec<u64>;

    /// Clear all markers on all lines.
    pub fn clear_all(&mut self);
}
```

### IndicatorCatalogue

```rust
/// Registry of indicator style configurations for all 44 slots.
///
/// Addresses: Requirements 1, 2, 15
pub struct IndicatorCatalogue {
    /// Configuration for each indicator number (0–43).
    configs: [IndicatorConfig; 44],
}

impl IndicatorCatalogue {
    /// Create catalogue with compiled default configurations.
    pub fn new() -> Self;

    /// Get the configuration for an indicator.
    pub fn get(&self, indicator: IndicatorNumber) -> &IndicatorConfig;

    /// Update an indicator's configuration (typically from theme reload).
    pub fn set(&mut self, indicator: IndicatorNumber, config: IndicatorConfig);

    /// Check if an indicator is dynamic (has hover state).
    ///
    /// Addresses: Requirement 2 AC 7
    pub fn is_dynamic(&self, indicator: IndicatorNumber) -> bool;

    /// Reload all configurations from theme palette.
    ///
    /// Addresses: Requirement 15 AC 3
    pub fn reload_from_theme(&mut self, theme: &dyn ThemeDecorationProvider);
}
```


### IndicatorAllocator

```rust
/// Manages indicator number allocation and namespace enforcement.
///
/// Addresses: Requirement 13 AC 1–6
pub struct IndicatorAllocator {
    /// Tracks which container-range indicators (8–31) are allocated.
    allocated: [bool; 24],
    /// Plugin ID associated with each allocated slot.
    owners: [Option<String>; 24],
}

impl IndicatorAllocator {
    pub fn new() -> Self;

    /// Allocate an indicator number from the container range (8–31) for a plugin.
    ///
    /// Addresses: Requirement 13 AC 4, 5
    pub fn allocate(&mut self, plugin_id: &str) -> Result<IndicatorNumber, DecorationError>;

    /// Release a previously allocated indicator number.
    pub fn release(&mut self, indicator: IndicatorNumber) -> Result<(), DecorationError>;

    /// Check if an indicator number is in the lexer range (0–7).
    ///
    /// Addresses: Requirement 13 AC 6
    pub fn is_lexer_range(indicator: IndicatorNumber) -> bool;

    /// Check if an indicator number is in the container range (8–31).
    pub fn is_container_range(indicator: IndicatorNumber) -> bool;

    /// Check if an indicator number is in the IME range (32–35).
    pub fn is_ime_range(indicator: IndicatorNumber) -> bool;

    /// Check if an indicator number is in the history range (36–43).
    pub fn is_history_range(indicator: IndicatorNumber) -> bool;
}
```

### HoverState

```rust
/// Tracks mouse hover position and dynamic indicator interaction.
///
/// Addresses: Requirement 11 AC 1–7
pub struct HoverState {
    /// Current character position under the mouse cursor, or None if outside text.
    current_position: Option<u64>,
    /// Previous position (for detecting transitions).
    previous_position: Option<u64>,
    /// Whether a click has been notified for the current hover position.
    click_notified: bool,
}

impl HoverState {
    pub fn new() -> Self;

    /// Update the hover position. Returns true if a redraw is needed (dynamic indicators changed).
    ///
    /// Addresses: Requirement 11 AC 1, 2
    pub fn update_position(
        &mut self,
        position: Option<u64>,
        decoration_list: &DecorationList,
        catalogue: &IndicatorCatalogue,
    ) -> bool;

    /// Mark a click as dispatched at the current position.
    ///
    /// Addresses: Requirement 11 AC 4
    pub fn notify_click(&mut self);

    /// Get the current hover position.
    pub fn position(&self) -> Option<u64>;

    /// Check if click has been notified for current position.
    pub fn is_click_notified(&self) -> bool;

    /// Reset click notification state.
    pub fn reset_click(&mut self);
}
```

### DecorationRenderer Trait

```rust
/// Trait defining the query interface the viewport renderer uses
/// to obtain decoration data for painting.
///
/// Addresses: Requirement 14 AC 5, 6
pub trait DecorationRenderer: Send + Sync {
    /// Get all active indicator ranges intersecting the character range [start, end).
    /// Returns tuples of (indicator_number, range_start, range_end, value).
    ///
    /// Addresses: Requirement 14 AC 2
    fn indicators_in_range(
        &self,
        start: u64,
        end: u64,
    ) -> Vec<(IndicatorNumber, u64, u64, u32)>;

    /// Get the marker bitmask for a given document line.
    ///
    /// Addresses: Requirement 14 AC 3
    fn marker_mask_for_line(&self, line: u64) -> MarkerMask;

    /// Get the indicator configuration for a given indicator number.
    fn indicator_config(&self, indicator: IndicatorNumber) -> &IndicatorConfig;

    /// Get the line marker configuration for a given marker number.
    fn marker_config(&self, marker: MarkerNumber) -> &LineMarkerConfig;

    /// Get the current hover position (for dynamic indicator rendering).
    fn hover_position(&self) -> Option<u64>;

    /// Check if a given indicator is dynamic at the current hover position.
    fn is_hovered_dynamic(&self, indicator: IndicatorNumber, position: u64) -> bool;
}
```


### PixelAligner

```rust
/// High-DPI pixel alignment utility.
///
/// Addresses: Requirement 10 AC 1–8
pub struct PixelAligner {
    /// Display scale factor (e.g., 1.0, 1.5, 2.0).
    scale_factor: f32,
    /// Pixel divisions (1.0 / scale_factor) for sub-pixel snapping.
    pixel_division: f32,
}

impl PixelAligner {
    pub fn new(scale_factor: f32) -> Self;

    /// Snap a coordinate to the nearest device-pixel boundary.
    ///
    /// Addresses: Requirement 10 AC 1
    pub fn align(&self, coord: f32) -> f32;

    /// Snap a rectangle outward to device-pixel boundaries.
    ///
    /// Addresses: Requirement 10 AC 4
    pub fn align_rect_outward(&self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32);

    /// Scale stroke width for the current DPI.
    ///
    /// Addresses: Requirement 10 AC 3
    pub fn scale_stroke(&self, logical_width: f32) -> f32;

    /// Update the scale factor (e.g., when moving to a different monitor).
    pub fn set_scale_factor(&mut self, factor: f32);

    /// Get the current scale factor.
    pub fn scale_factor(&self) -> f32;
}
```

### ThemeDecorationProvider Trait

```rust
/// Trait abstracting theme palette access for decoration colours.
/// Implemented by ff-theme's palette to avoid hard-coupling to the theme crate.
///
/// Addresses: Requirement 15 AC 1–8
pub trait ThemeDecorationProvider: Send + Sync {
    /// Get the configured colour for an indicator number.
    fn indicator_fore(&self, indicator: IndicatorNumber) -> Option<ColourRGBA>;

    /// Get the configured fill alpha for an indicator.
    fn indicator_fill_alpha(&self, indicator: IndicatorNumber) -> Option<u8>;

    /// Get the configured outline alpha for an indicator.
    fn indicator_outline_alpha(&self, indicator: IndicatorNumber) -> Option<u8>;

    /// Get the configured stroke width for an indicator.
    fn indicator_stroke_width(&self, indicator: IndicatorNumber) -> Option<f32>;

    /// Get the configured style override for an indicator.
    fn indicator_style(&self, indicator: IndicatorNumber) -> Option<IndicatorStyle>;

    /// Get the configured colours for a marker number.
    fn marker_fore(&self, marker: MarkerNumber) -> Option<ColourRGBA>;
    fn marker_back(&self, marker: MarkerNumber) -> Option<ColourRGBA>;
    fn marker_back_selected(&self, marker: MarkerNumber) -> Option<ColourRGBA>;
    fn marker_alpha(&self, marker: MarkerNumber) -> Option<u8>;
    fn marker_symbol(&self, marker: MarkerNumber) -> Option<MarkerSymbol>;
}
```

### Decoration Events

```rust
/// Events emitted by the text-decorations system.
///
/// Addresses: Requirement 11 AC 5
#[derive(Debug, Clone)]
pub enum DecorationEvent {
    /// A click occurred on a decorated position.
    Click {
        /// Character position that was clicked.
        position: u64,
        /// Indicator numbers active at the click position.
        indicators: Vec<IndicatorNumber>,
    },
    /// Hover entered a dynamic indicator range.
    HoverEnter {
        position: u64,
        indicator: IndicatorNumber,
    },
    /// Hover left a dynamic indicator range.
    HoverLeave {
        position: u64,
        indicator: IndicatorNumber,
    },
}

/// Trait for receiving decoration events.
pub trait DecorationEventListener: Send + Sync {
    fn on_decoration_event(&self, event: &DecorationEvent);
}
```


---

## Error Types

```rust
/// Errors produced by the text-decorations crate.
///
/// Addresses: Cross-cutting Req 8 (error format)
#[derive(Debug, thiserror::Error)]
pub enum DecorationError {
    /// Position is beyond document length.
    #[error("[decorations] value_at: position {position} exceeds document length {document_length}")]
    PositionOutOfRange {
        position: u64,
        document_length: u64,
    },

    /// Indicator number is out of the valid range (0–43).
    #[error("[decorations] indicator: number {0} exceeds maximum {}", IndicatorNumber::MAX)]
    InvalidIndicatorNumber(u8),

    /// Marker number is out of the valid range (0–31).
    #[error("[decorations] marker: number {0} exceeds maximum {}", MarkerNumber::MAX)]
    InvalidMarkerNumber(u8),

    /// Attempted to write to the lexer range (0–7) from non-lexer code.
    #[error("[decorations] fill_range: indicator {0} is in the lexer range (0–7), reserved for syntax-highlighting")]
    LexerRangeViolation(u8),

    /// No available indicator slots in the container range.
    #[error("[decorations] allocate: all container-range indicator numbers (8–31) are allocated")]
    NoAvailableIndicators,

    /// Attempted to release an indicator that was not allocated.
    #[error("[decorations] release: indicator {0} was not allocated")]
    NotAllocated(u8),

    /// Line number out of range.
    #[error("[decorations] marker: line {line} exceeds document line count {line_count}")]
    LineOutOfRange {
        line: u64,
        line_count: u64,
    },

    /// Theme value validation failure.
    #[error("[decorations] theme: invalid value for {field}: {reason}")]
    InvalidThemeValue {
        field: String,
        reason: String,
    },
}
```

---

## Well-Known Constants

```rust
/// Well-known indicator number allocations.
///
/// Addresses: Requirement 13 AC 3
pub mod indicators {
    use super::IndicatorNumber;

    // Container range (8–31): application-managed
    pub const SEARCH_CURRENT: IndicatorNumber = IndicatorNumber(8);
    pub const SEARCH_ALL: IndicatorNumber = IndicatorNumber(9);
    pub const ERROR: IndicatorNumber = IndicatorNumber(10);
    pub const WARNING: IndicatorNumber = IndicatorNumber(11);
    pub const INFO: IndicatorNumber = IndicatorNumber(12);
    pub const HINT: IndicatorNumber = IndicatorNumber(13);
    // 14–31: available for plugins

    // IME range (32–35)
    pub const IME_INPUT: IndicatorNumber = IndicatorNumber(32);
    pub const IME_TARGET: IndicatorNumber = IndicatorNumber(33);
    pub const IME_CONVERTED: IndicatorNumber = IndicatorNumber(34);
    pub const IME_TARGET_NON_CONVERTED: IndicatorNumber = IndicatorNumber(35);

    // History range (36–43)
    pub const HISTORY_MODIFIED_INSERTION: IndicatorNumber = IndicatorNumber(36);
    pub const HISTORY_MODIFIED_DELETION: IndicatorNumber = IndicatorNumber(37);
    pub const HISTORY_SAVED_INSERTION: IndicatorNumber = IndicatorNumber(38);
    pub const HISTORY_SAVED_DELETION: IndicatorNumber = IndicatorNumber(39);
    pub const HISTORY_REVERTED_ORIGIN_INSERTION: IndicatorNumber = IndicatorNumber(40);
    pub const HISTORY_REVERTED_ORIGIN_DELETION: IndicatorNumber = IndicatorNumber(41);
    pub const HISTORY_REVERTED_MODIFIED_INSERTION: IndicatorNumber = IndicatorNumber(42);
    pub const HISTORY_REVERTED_MODIFIED_DELETION: IndicatorNumber = IndicatorNumber(43);
}

/// Well-known marker number allocations.
///
/// Addresses: Requirements 7, 8
pub mod markers {
    use super::MarkerNumber;

    pub const BOOKMARK: MarkerNumber = MarkerNumber(0);
    pub const HISTORY_MODIFIED: MarkerNumber = MarkerNumber(1);
    pub const HISTORY_SAVED: MarkerNumber = MarkerNumber(2);
    pub const HISTORY_REVERTED_ORIGIN: MarkerNumber = MarkerNumber(3);
    pub const HISTORY_REVERTED_MODIFIED: MarkerNumber = MarkerNumber(4);
    // 5–31: available for fold markers, plugins, etc.
}
```


---

## Integration Points

### Integration with `ff-find-and-replace`

The find engine is a **producer** of decoration data. After executing a search:

1. `HighlightAllMatches` computes all match ranges in the visible viewport
2. The find engine calls `DecorationList::fill_range(indicators::SEARCH_ALL, start, 1, length)` for each match
3. For the current/focused match, calls `DecorationList::fill_range(indicators::SEARCH_CURRENT, start, 1, length)`
4. On RFIND navigation: clears the old SEARCH_CURRENT range, sets the new one, old position reverts to SEARCH_ALL
5. On search cancel/clear: fills all search indicator ranges with 0

The integration contract:
- `ff-find-and-replace` depends on `ff-text-decorations` for the `DecorationList` API and indicator constants
- `ff-text-decorations` does NOT depend on `ff-find-and-replace`
- The find engine holds a `&mut DecorationList` reference (or receives it via function parameter) during highlight operations

### Integration with `ff-edit-operations`

The edit system triggers decoration synchronization:

1. After any text insertion/deletion, `ff-edit-operations` emits an edit event
2. `EditSync` receives the event and calls `DecorationList::insert_space` or `delete_range`
3. `EditSync` also calls `MarkerStore::lines_inserted` or `lines_deleted` for line-level markers
4. Change history tracking: edit operations set `markers::HISTORY_MODIFIED` on affected lines and fill character-level history indicators

The integration contract:
- `ff-text-decorations` receives edit notifications (position + length) but does NOT depend on `ff-edit-operations` directly
- The hosting layer (document session) connects the two via an event bus or direct method calls
- Undo/redo operations trigger the inverse space adjustments

### Integration with `ff-theme` (`theme-and-appearance`)

Theme provides all colours and style overrides:

1. At startup, `IndicatorCatalogue::reload_from_theme()` reads all indicator/marker colours from the palette
2. The theme's `[decorations]` section maps to indicator colours (search highlight, error, warning, etc.)
3. The theme's `[indicators]` section provides per-slot overrides (fore, fill_alpha, outline_alpha, stroke_width, style)
4. On hot-reload or mode switch, the `ThemeIntegration` listener triggers `reload_from_theme()` and requests viewport repaint

The integration contract:
- `ff-text-decorations` depends on `ff-theme` via the `ThemeDecorationProvider` trait (not concrete types)
- Theme changes are communicated via the theme-change event/notification system
- Invalid theme values are clamped and logged (alpha: 0–255, stroke_width: 0.5–10.0)

### Integration with `ff-command` (Command Framework)

Bookmark operations are registered as commands:

| Command ID | Description | Shortcut |
|---|---|---|
| `decorations.bookmark.toggle` | Toggle bookmark on current line | (user-configurable) |
| `decorations.bookmark.next` | Navigate to next bookmark | (user-configurable) |
| `decorations.bookmark.previous` | Navigate to previous bookmark | (user-configurable) |
| `decorations.bookmark.clear_all` | Remove all bookmarks | (user-configurable) |

### Integration with `ff-undo-redo-transactions`

- Decoration position adjustments (`insert_space` / `delete_range`) are applied in response to undo/redo operations
- When undo reverses an insertion, a matching `delete_range` is applied to decorations
- When undo reverses a deletion, a matching `insert_space` is applied
- Change history markers transition state on undo (Modified → RevertedToOrigin or RevertedToModified)

### Integration with `ff-configuration-system`

- Per-indicator default overrides stored in `[decorations]` config namespace
- Modified-line gutter visibility: `decorations.change_margin.visible` (bool, default true)
- Bookmark margin width: `decorations.bookmark_margin.width` (float, default 16.0)
- Hot-reload updates decoration configuration without restart

---

## Rendering Pipeline Layer Order

The viewport renderer draws decorations in this strict layer order per line:

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 6: Gutter/Margin Markers (bookmark, change bars)       │
├─────────────────────────────────────────────────────────────┤
│ Layer 5: Selection overlay (translucent)                     │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Over-indicators (under = false, drawn on top)       │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Text glyphs with syntax highlighting                │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Under-indicators (under = true, drawn below text)   │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Line background markers (Background-symbol markers) │
├─────────────────────────────────────────────────────────────┤
│ Layer 0: Editor background                                   │
└─────────────────────────────────────────────────────────────┘
```

Within each indicator layer, indicators are drawn in indicator-number order (lower numbers first). Multiple overlapping indicators are all drawn independently so all remain visible.


---

## Correctness Properties

These properties are suitable for property-based testing with `proptest`.

### Property 1: RLE Invariant — Total Length Preservation

**Statement:** For any sequence of `fill_range`, `insert_space`, and `delete_range` operations on a `RunStyles<T>`, the sum of all run lengths always equals the tracked document length.

**Validates:** Requirement 3 AC 10, Requirement 4 AC 8

```
∀ ops ∈ Seq<Operation>, initial_length > 0:
  let rs = RunStyles::new(initial_length)
  apply(ops, &mut rs)
  ⇒ rs.total_length() == expected_length_after_ops
```

### Property 2: Fill Range Idempotency

**Statement:** Filling the same range with the same value twice produces the same state as filling once (second fill returns `false` — no change).

**Validates:** Requirement 3 AC 8

```
∀ position, value, length:
  let rs = RunStyles::new(N)
  rs.fill_range(position, value, length)  // first fill
  let changed = rs.fill_range(position, value, length)  // second fill
  ⇒ changed == false
```

### Property 3: Insert-Delete Round Trip

**Statement:** Inserting space at position P with length L, then deleting the same range, restores the original decoration state.

**Validates:** Requirement 4 AC 5, 6

```
∀ P, L, initial_state:
  let before = rs.clone()
  rs.insert_space(P, L)
  rs.delete_range(P, L)
  ⇒ rs == before
```

### Property 4: Value Consistency After Edit

**Statement:** After `insert_space(P, L)`, all positions before P retain their original values, positions P..P+L have value 0, and positions after P+L have the values that were originally at positions after P.

**Validates:** Requirement 4 AC 1, 3, 4

```
∀ P, L:
  let original_values = (0..doc_len).map(|i| rs.value_at(i))
  rs.insert_space(P, L)
  ⇒ ∀ i < P: rs.value_at(i) == original_values[i]
  ⇒ ∀ i in P..P+L: rs.value_at(i) == 0
  ⇒ ∀ i >= P+L: rs.value_at(i) == original_values[i - L]
```

### Property 5: Lazy Creation and Removal

**Statement:** A decoration is created only on the first non-zero `fill_range` and removed when all values become zero.

**Validates:** Requirement 3 AC 3, 4

```
∀ indicator:
  let dl = DecorationList::new(N)
  ⇒ dl.active_count() == 0
  dl.fill_range(indicator, 0, 1, 5)
  ⇒ dl.active_count() == 1
  dl.fill_range(indicator, 0, 0, 5)  // clear
  ⇒ dl.active_count() == 0
```

### Property 6: Marker Line Tracking

**Statement:** After inserting K lines at line L, all markers originally on lines ≥ L move to line + K; markers on lines < L are unchanged.

**Validates:** Requirement 9 AC 10

```
∀ markers_before, insert_line, insert_count:
  store.lines_inserted(insert_line, insert_count)
  ⇒ ∀ (line, mask) in markers_before:
       if line < insert_line: store.marker_get(line) == mask
       else: store.marker_get(line + insert_count) == mask
```

### Property 7: All-On-For Consistency

**Statement:** The `all_on_for(position)` bitmask is consistent with individual `value_at` queries: bit N is set iff `value_at(N, position) != 0`.

**Validates:** Requirement 3 AC 9

```
∀ position, indicator_set:
  let mask = dl.all_on_for(position)
  ⇒ ∀ indicator in 0..=43:
       (mask >> indicator) & 1 == (dl.value_at(indicator, position) != 0) as u64
```

### Property 8: Bookmark Next/Previous Wrapping

**Statement:** `next_bookmark(from_line)` returns the nearest bookmarked line at or after `from_line`, wrapping around the document end. `previous_bookmark` wraps around the document start.

**Validates:** Requirement 8 AC 6

```
∀ bookmarked_lines (non-empty), from_line:
  let next = store.marker_next(from_line, BOOKMARK_MASK)
  ⇒ next is the smallest line ≥ from_line with bookmark, or smallest overall if none after
```

### Property 9: Run Merge Optimality

**Statement:** After any `fill_range` operation, no two adjacent runs have the same value (runs are always maximally merged).

**Validates:** Requirement 3 AC 1 (optimal RLE)

```
∀ ops:
  apply(ops, &mut rs)
  ⇒ ∀ i in 0..rs.runs.len()-1:
       rs.runs[i].value != rs.runs[i+1].value
```

### Property 10: Theme Reload Preserves Decoration Data

**Statement:** Reloading theme colours does not modify any stored indicator values or marker assignments — only visual rendering properties change.

**Validates:** Requirement 2 AC 10, Requirement 15 AC 3

```
∀ decoration_state, new_theme:
  let values_before = snapshot(decoration_list)
  catalogue.reload_from_theme(new_theme)
  ⇒ snapshot(decoration_list) == values_before
```

---

## Testing Strategy

| Test Type | Framework | Focus |
|-----------|-----------|-------|
| Unit tests | `#[cfg(test)]` modules | Individual method correctness for RunStyles, DecorationList, MarkerStore |
| Property tests | `proptest` | RLE invariants, edit sync round-trips, marker tracking |
| Integration tests | `tests/` directory | Multi-producer scenarios (search + diagnostics), theme reload |
| Benchmark tests | `criterion` (optional) | `fill_range` and `indicators_in_range` performance on large documents |

### Test Configuration

- Property tests: minimum 256 cases per property
- RunStyles operations: generate random sequences of fill/insert/delete and verify invariants
- MarkerStore: generate random line insert/delete sequences and verify marker positions
- Theme integration: mock `ThemeDecorationProvider` returning various valid/invalid values
