# Design Document: Display Line Mapping (`ff-display-line-mapping`)

## Overview

The `ff-display-line-mapping` crate maintains the **bidirectional relationship between document lines and display lines** for the FileForgeWorkbench editor. It implements the Contraction State concept (adapted from Scintilla) in Rust, providing O(log n) lookup performance for both forward (doc→display) and reverse (display→doc) conversions.

### Purpose

- Map document line numbers to display line positions and vice versa
- Track per-line visibility (hidden/visible) for ISPF EXCLUDE/SHOW and code folding
- Track per-line display height (wrap sub-lines) for word wrap
- Track per-line fold expanded/collapsed state for code folding
- Support incremental updates on line insert/delete/visibility/height changes
- Provide a lazy-allocation one-to-one mode for zero-overhead when no folding/exclusion/wrapping is active
- Support 64-bit line indexing for very large documents

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
├─────────────────────────────────────────────────────────────┤
│  Consuming Crates: ff-viewport-and-scrolling,                │
│    ff-exclude-show-filter, ff-idle-processing,               │
│    ff-line-wrap-toggle                                        │
│         (consume ff-display-line-mapping public API)          │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-display-line-mapping ← Wave 4                │
├─────────────────────────────────────────────────────────────┤
│  Upstream: ff-document-model (line count, watcher API)        │
│            ff-core (runtime), ff-command (fold commands)       │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: No direct filesystem access — all content queries go through `ff-document-model`
- **GUI Independence (Req 2)**: Zero GUI dependencies — no egui, winit, wgpu
- **Command-Driven (Req 4)**: Fold/unfold operations integrate with the command framework
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-display-line-mapping`
- **Error Message Standards (Req 8)**: Errors follow `[display-mapping] operation: description` format

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Consumers [Consuming Crates]
        VP[ff-viewport-and-scrolling]
        ESF[ff-exclude-show-filter]
        IDLE[ff-idle-processing]
        LWT[ff-line-wrap-toggle]
        CMD[ff-command-framework]
    end

    subgraph ff-display-line-mapping [ff-display-line-mapping Crate]
        CS[ContractionState]
        PART[Partitioning / Fenwick Tree]
        VIS[Visibility Store]
        FOLD[Fold State Store]
        HGT[Height Store]
        FDT[Fold Display Text Store]
        NOTIFY[Change Notifier]
        TRAIT[DisplayLineMapping Trait]
    end

    subgraph Upstream [Upstream Crates]
        DOC[ff-document-model]
        LOG[ff-logging]
    end

    VP -->|display_from_doc / doc_from_display| TRAIT
    ESF -->|set_visible / get_visible| TRAIT
    IDLE -->|set_height| TRAIT
    LWT -->|set_height bulk| TRAIT
    CMD -->|set_expanded / expand_all| TRAIT

    TRAIT --> CS
    CS --> PART
    CS --> VIS
    CS --> FOLD
    CS --> HGT
    CS --> FDT
    CS --> NOTIFY
    CS -->|DocumentWatcher| DOC
    CS --> LOG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **ContractionState** | Central state machine: owns per-line visibility, heights, fold state, and the partitioning tree. Implements the `DisplayLineMapping` trait. |
| **Partitioning (Fenwick Tree)** | Prefix-sum data structure providing O(log n) cumulative height queries and O(log n) point updates. |
| **Visibility Store** | Per-line boolean array tracking visible/hidden state. Lazily allocated. |
| **Height Store** | Per-line `u32` array tracking display height (wrap sub-lines). Lazily allocated. |
| **Fold State Store** | Per-line boolean tracking expanded/collapsed state per fold header. Lazily allocated. |
| **Fold Display Text Store** | Per-line optional `String` for collapsed fold summary text. Lazily allocated. |
| **Change Notifier** | Callback registry for display-line-count changes, consumed by viewport/scrollbar. |
| **DisplayLineMapping Trait** | Public trait defining the full API surface for consumers to depend on. |

### Internal State Modes

The `ContractionState` operates in one of two modes:

1. **One-to-One Mode** (default): No per-line data structures allocated. All lookups are O(1) identity operations. Memory footprint is O(1) regardless of document size. Transitions to Full Mode on first non-trivial operation.

2. **Full Tracking Mode**: Per-line arrays and Fenwick tree allocated. All lookups are O(log n). Transitions back to One-to-One Mode via `show_all()` when all lines return to default state.

---

## Components and Interfaces

```
crates/ff-display-line-mapping/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── traits.rs               # DisplayLineMapping trait definition
│   ├── contraction_state.rs    # ContractionState struct, main implementation
│   ├── partitioning/
│   │   ├── mod.rs              # Partitioning re-exports
│   │   ├── fenwick_tree.rs     # Fenwick tree (Binary Indexed Tree)
│   │   └── fenwick_tree_64.rs  # 64-bit variant for large documents
│   ├── stores/
│   │   ├── mod.rs              # Store re-exports
│   │   ├── visibility.rs       # Per-line visibility boolean array
│   │   ├── heights.rs          # Per-line display height array
│   │   ├── fold_state.rs       # Per-line expanded/collapsed state
│   │   └── fold_text.rs        # Per-line fold display text
│   ├── notifier.rs             # Change notification dispatch
│   ├── types.rs                # DocLine, DisplayLine, SubLine newtypes
│   └── error.rs                # DisplayMappingError enum
└── tests/
    ├── one_to_one_tests.rs     # One-to-one mode behaviour
    ├── visibility_tests.rs     # Show/hide and display line count
    ├── folding_tests.rs        # Fold expand/collapse integration
    ├── wrap_tests.rs           # Height changes and sub-line mapping
    ├── incremental_tests.rs    # Insert/delete line updates
    ├── large_doc_tests.rs      # 64-bit mode, performance bounds
    ├── property_tests.rs       # proptest property-based tests
    └── integration.rs          # End-to-end with mock document model
```

---

## Data Models

### Core Newtypes

```rust
/// A zero-based document line index.
///
/// Addresses: Requirement 1 AC 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocLine(pub usize);

/// A zero-based display line index (contiguous across visible content).
///
/// Addresses: Requirement 1 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayLine(pub usize);

/// A zero-based sub-line offset within a wrapped document line.
/// Sub-line 0 is the first visual line of a wrapped document line.
///
/// Addresses: Requirement 4 AC 8
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubLine(pub usize);

/// Result of a display-to-document lookup, including the sub-line offset.
///
/// Addresses: Requirement 1 AC 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocPosition {
    /// The document line containing this display line.
    pub doc_line: DocLine,
    /// The sub-line offset within the document line (0 for unwrapped).
    pub sub_line: SubLine,
}
```

### Partitioning Data Structure (Fenwick Tree)

```rust
/// A Fenwick tree (Binary Indexed Tree) storing per-line display heights.
/// Supports O(log n) prefix-sum queries and O(log n) point updates.
///
/// The cumulative sum at index `i` gives the display line offset for
/// the start of document line `i`. This enables:
/// - display_from_doc(i) = prefix_sum(i)
/// - doc_from_display(d) = binary search for smallest i where prefix_sum(i) > d
///
/// Addresses: Requirement 5 AC 1, AC 2, AC 3
pub struct FenwickTree {
    /// Internal storage: tree[i] holds partial sums.
    /// Length = lines_in_doc + 1 (1-indexed internally).
    tree: Vec<u32>,
    /// Total number of document lines tracked.
    line_count: usize,
}

impl FenwickTree {
    /// Create a new Fenwick tree with `n` elements, all initialized to 1.
    pub fn new(n: usize) -> Self;

    /// Query the prefix sum from index 0 to `idx` (exclusive).
    /// Returns the cumulative display height before `idx`.
    /// O(log n) time.
    pub fn prefix_sum(&self, idx: usize) -> usize;

    /// Get the value at a specific index.
    /// O(log n) time.
    pub fn get(&self, idx: usize) -> u32;

    /// Update the value at `idx` by adding `delta` (can be negative).
    /// O(log n) time.
    pub fn update(&mut self, idx: usize, delta: i64);

    /// Find the smallest index where prefix_sum(idx) > target.
    /// Used for doc_from_display. O(log n) time.
    pub fn find_prefix(&self, target: usize) -> usize;

    /// Total sum of all elements (= total display line count).
    pub fn total(&self) -> usize;

    /// Insert `count` new elements at position `idx`, each with value `val`.
    /// O(count × log n) time (rebuilds affected portion).
    pub fn insert(&mut self, idx: usize, count: usize, val: u32);

    /// Remove `count` elements starting at position `idx`.
    /// O(count × log n) time.
    pub fn remove(&mut self, idx: usize, count: usize);
}
```

### 64-bit Variant

```rust
/// 64-bit variant of FenwickTree for documents exceeding 2^31 lines.
///
/// Addresses: Requirement 8 AC 1, AC 2
pub struct FenwickTree64 {
    tree: Vec<u64>,
    line_count: u64,
}
```

### Per-Line Stores

```rust
/// Per-line visibility tracking. Lazily allocated.
///
/// Addresses: Requirement 2, Requirement 9 AC 2
pub(crate) struct VisibilityStore {
    /// Bit vector: true = visible, false = hidden.
    /// None when in one-to-one mode (all visible).
    bits: Option<Vec<bool>>,
}

/// Per-line display height tracking. Lazily allocated.
///
/// Addresses: Requirement 4, Requirement 9 AC 2
pub(crate) struct HeightStore {
    /// Per-line height values. None when in one-to-one mode (all height 1).
    heights: Option<Vec<u32>>,
}

/// Per-line fold expanded/collapsed state. Lazily allocated.
///
/// Addresses: Requirement 3, Requirement 10 AC 1
pub(crate) struct FoldStateStore {
    /// Per-line expanded flag. None when in one-to-one mode (all expanded).
    /// true = expanded (or not a fold header), false = collapsed.
    expanded: Option<Vec<bool>>,
}

/// Per-line fold display text. Lazily allocated.
///
/// Addresses: Requirement 3 AC 7, AC 8
pub(crate) struct FoldTextStore {
    /// Sparse map from doc line to display text.
    /// Uses a HashMap since only a few lines will have fold text.
    texts: Option<std::collections::HashMap<usize, String>>,
}
```

### ContractionState

```rust
/// The central state machine tracking the document-to-display line mapping.
/// Implements the `DisplayLineMapping` trait.
///
/// Starts in One_To_One_Mode with O(1) memory. Lazily transitions to
/// Full Tracking Mode on the first non-trivial operation (hide, fold, wrap).
///
/// Addresses: Requirements 1–10
pub struct ContractionState {
    /// Total number of document lines tracked.
    line_count: usize,

    /// Whether we are in optimized one-to-one mode.
    one_to_one: bool,

    /// Whether this instance uses 64-bit indexing.
    large_document: bool,

    /// Fenwick tree for prefix-sum display line calculations.
    /// None in one-to-one mode.
    partitioning: Option<FenwickTree>,

    /// Per-line visibility (visible/hidden).
    visibility: VisibilityStore,

    /// Per-line display heights (wrap sub-lines).
    heights: HeightStore,

    /// Per-line fold expanded/collapsed state.
    fold_state: FoldStateStore,

    /// Per-line fold display text (sparse).
    fold_text: FoldTextStore,

    /// Registered change listeners.
    listeners: Vec<Box<dyn Fn(DisplayLineCountChange) + Send + Sync>>,
}

/// Notification payload when display line count changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayLineCountChange {
    /// Previous total display lines.
    pub old_count: usize,
    /// New total display lines.
    pub new_count: usize,
}
```

---

## Public API Surface

### DisplayLineMapping Trait

```rust
/// Public trait defining the full display-line-mapping API.
/// Consumers depend on this trait rather than the concrete ContractionState.
///
/// Addresses: Requirement 7 AC 10
pub trait DisplayLineMapping: Send + Sync {
    // --- Document-to-Display Conversion ---

    /// Returns the first display line index for the given document line.
    /// Equals the cumulative sum of display heights of all preceding visible lines.
    ///
    /// Addresses: Requirement 1 AC 1
    fn display_from_doc(&self, doc_line: DocLine) -> DisplayLine;

    /// Returns the display line index for a specific sub-line within a document line.
    /// Clamps sub_line to height - 1 if it exceeds the line's display height.
    ///
    /// Addresses: Requirement 1 AC 2
    fn display_from_doc_sub(&self, doc_line: DocLine, sub_line: SubLine) -> DisplayLine;

    /// Returns the last display line index occupied by the given document line.
    ///
    /// Addresses: Requirement 1 AC 3
    fn display_last_from_doc(&self, doc_line: DocLine) -> DisplayLine;

    // --- Display-to-Document Conversion ---

    /// Returns the document line and sub-line offset for a given display line.
    /// Always returns a visible line. Clamps out-of-range display lines.
    ///
    /// Addresses: Requirement 1 AC 4, AC 5, AC 6
    fn doc_from_display(&self, display_line: DisplayLine) -> DocPosition;

    // --- Line Counts ---

    /// Total number of document lines in the mapping.
    ///
    /// Addresses: Requirement 1 AC 7
    fn lines_in_doc(&self) -> usize;

    /// Total display line count (sum of heights of all visible lines).
    ///
    /// Addresses: Requirement 1 AC 8
    fn lines_displayed(&self) -> usize;

    // --- Visibility ---

    /// Set visibility for a range of document lines [start, end] inclusive.
    /// Returns true if any line's visibility actually changed.
    ///
    /// Addresses: Requirement 2 AC 1
    fn set_visible(&mut self, start: DocLine, end: DocLine, visible: bool) -> bool;

    /// Query visibility for a single document line.
    ///
    /// Addresses: Requirement 2 AC 2
    fn get_visible(&self, doc_line: DocLine) -> bool;

    /// Returns true if any document line is currently hidden.
    ///
    /// Addresses: Requirement 2 AC 5
    fn hidden_lines(&self) -> bool;

    /// Make all lines visible and reset to one-to-one mode.
    ///
    /// Addresses: Requirement 2 AC 6
    fn show_all(&mut self);

    // --- Fold State ---

    /// Set the expanded/collapsed state of a fold header line.
    /// Returns true if the state changed.
    ///
    /// Addresses: Requirement 3 AC 1
    fn set_expanded(&mut self, doc_line: DocLine, expanded: bool) -> bool;

    /// Query the expanded state of a document line.
    /// Returns true for non-fold-header lines and expanded fold headers.
    ///
    /// Addresses: Requirement 3 AC 2
    fn get_expanded(&self, doc_line: DocLine) -> bool;

    /// Set all fold headers to expanded state.
    /// Returns true if any fold state changed.
    ///
    /// Addresses: Requirement 3 AC 3
    fn expand_all(&mut self) -> bool;

    /// Find the next collapsed fold header at or after start_line.
    /// Returns None if no contracted fold exists beyond that point.
    ///
    /// Addresses: Requirement 3 AC 4
    fn contracted_next(&self, start_line: DocLine) -> Option<DocLine>;

    /// Set fold display text for a collapsed fold header.
    /// Returns true if the text changed.
    ///
    /// Addresses: Requirement 3 AC 7
    fn set_fold_display_text(&mut self, doc_line: DocLine, text: Option<&str>) -> bool;

    /// Get fold display text for a line. Returns None if not set.
    ///
    /// Addresses: Requirement 3 AC 8
    fn get_fold_display_text(&self, doc_line: DocLine) -> Option<&str>;

    // --- Wrap Height ---

    /// Set the display height (number of sub-lines) for a document line.
    /// Returns true if the height changed.
    ///
    /// Addresses: Requirement 4 AC 1
    fn set_height(&mut self, doc_line: DocLine, height: u32) -> bool;

    /// Get the current display height of a document line.
    ///
    /// Addresses: Requirement 4 AC 2
    fn get_height(&self, doc_line: DocLine) -> u32;

    // --- Incremental Updates ---

    /// Insert new document lines at the given position.
    /// New lines are initialized as visible with height 1.
    ///
    /// Addresses: Requirement 6 AC 1
    fn insert_lines(&mut self, doc_line: DocLine, count: usize);

    /// Remove document lines starting at the given position.
    ///
    /// Addresses: Requirement 6 AC 2
    fn delete_lines(&mut self, doc_line: DocLine, count: usize);

    // --- Change Notification ---

    /// Register a listener for display-line-count changes.
    /// Returns a handle for later removal.
    ///
    /// Addresses: Requirement 7 AC 9
    fn on_display_count_change(
        &mut self,
        callback: Box<dyn Fn(DisplayLineCountChange) + Send + Sync>,
    ) -> ListenerHandle;

    /// Remove a previously registered listener.
    fn remove_listener(&mut self, handle: ListenerHandle);
}

/// Handle for a registered change listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerHandle(pub u64);
```

### ContractionState — Construction

```rust
impl ContractionState {
    /// Create a new ContractionState in one-to-one mode for a document
    /// with the given number of lines.
    ///
    /// Addresses: Requirement 9 AC 1
    pub fn new(line_count: usize) -> Self;

    /// Create a ContractionState with large-document (64-bit) mode enabled.
    ///
    /// Addresses: Requirement 8 AC 1, AC 2
    pub fn new_large(line_count: usize) -> Self;

    /// Check whether the state is currently in one-to-one mode.
    ///
    /// Addresses: Requirement 9 AC 4
    pub fn is_one_to_one(&self) -> bool;
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-display-line-mapping crate.
/// Formatted per Error Message Standards (Req 8): `[display-mapping] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DisplayMappingError {
    /// Document line index is out of valid range.
    #[error("[display-mapping] {operation}: doc_line {line} out of range (total: {total})")]
    LineOutOfRange {
        operation: String,
        line: usize,
        total: usize,
    },

    /// Display line index is out of valid range.
    #[error("[display-mapping] {operation}: display_line {line} out of range (total: {total})")]
    DisplayLineOutOfRange {
        operation: String,
        line: usize,
        total: usize,
    },

    /// Height value is invalid (must be >= 1).
    #[error("[display-mapping] set_height: height 0 is not valid for doc_line {line}")]
    InvalidHeight {
        line: usize,
    },

    /// Listener handle not found for removal.
    #[error("[display-mapping] remove_listener: handle {handle_id} not found")]
    ListenerNotFound {
        handle_id: u64,
    },
}
```

---

## Integration Points

### With `ff-document-model` (Wave 4 — upstream)

- **Dependency direction**: ff-display-line-mapping depends on ff-document-model
- **API consumed**: `Document::line_count()` for initial line count; `DocumentWatcher` trait for insert/delete notifications
- **Integration pattern**: `ContractionState` subscribes as a `DocumentWatcher` on the associated document. When `notify_insert` fires with `lines_added > 0`, it calls `self.insert_lines(line, lines_added)`. When `notify_delete` fires with `lines_removed > 0`, it calls `self.delete_lines(line, lines_removed)`.
- **No content access**: The display-line-mapping does NOT read line content — it only tracks counts, visibility, and heights. Line content is accessed by the wrap calculator (in `ff-idle-processing`) which then calls `set_height`.

### With `ff-logging` (Foundation Layer — upstream)

- **Dependency direction**: ff-display-line-mapping depends on ff-logging
- **API consumed**: `log_info!`, `log_warn!`, `log_debug!` macros
- **Usage**: Mode transitions (one-to-one → full, full → one-to-one) logged at INFO; out-of-range clamping logged at DEBUG
- **Log prefix**: `[display-mapping]`

### With `ff-viewport-and-scrolling` (Wave 4 — downstream)

- **Dependency direction**: ff-viewport-and-scrolling depends on ff-display-line-mapping
- **API consumed**: `display_from_doc`, `doc_from_display`, `lines_displayed` for scroll position translation, viewport bounds, and scrollbar range
- **Integration**: The viewport uses `lines_displayed()` for the scrollbar maximum, `display_from_doc` to translate a document cursor position to a scroll offset, and `doc_from_display` to determine which document lines are visible in the viewport

### With `ff-exclude-show-filter` (Wave 5 — downstream)

- **Dependency direction**: ff-exclude-show-filter depends on ff-display-line-mapping
- **API consumed**: `set_visible`, `get_visible`, `hidden_lines`, `show_all`
- **Integration**: When EXCLUDE hides lines, it calls `set_visible(start, end, false)`. SHOW/RESET calls `set_visible(start, end, true)` or `show_all()`. The exclude-show-filter does NOT maintain its own visibility state — it delegates entirely to the display-line-mapping layer.

### With `ff-idle-processing` (Wave 15 — downstream)

- **Dependency direction**: ff-idle-processing depends on ff-display-line-mapping
- **API consumed**: `set_height` for background wrap height recalculation
- **Integration**: When the idle processor computes the wrap height for a line (based on content width and viewport width), it calls `set_height(line, new_height)` to update the mapping incrementally

### With `ff-line-wrap-toggle` (Wave 9 — downstream)

- **Dependency direction**: ff-line-wrap-toggle depends on ff-display-line-mapping
- **API consumed**: `set_height` in bulk when wrap mode is toggled
- **Integration**: When word wrap is disabled, calls `set_height(line, 1)` for all lines. When enabled, triggers a background wrap recalculation via `ff-idle-processing`

### With `ff-command-framework` (Wave 2 — peer integration)

- **Integration**: Fold/Unfold/Expand All/Collapse All commands are registered in the command framework. Command handlers invoke `set_expanded` and `set_visible` on the display-line-mapping.
- **Addresses**: Requirement 7 AC 8

### Dependency Direction Summary

```
ff-logging            ← ff-display-line-mapping
ff-document-model     ← ff-display-line-mapping (watcher subscription)
ff-display-line-mapping ← ff-viewport-and-scrolling
ff-display-line-mapping ← ff-exclude-show-filter
ff-display-line-mapping ← ff-idle-processing
ff-display-line-mapping ← ff-line-wrap-toggle
```

---

## Configuration

ff-display-line-mapping owns the `[display-mapping]` namespace in the workbench TOML configuration file.

### TOML Schema

```toml
[display-mapping]
# Threshold document line count for enabling large-document (64-bit) mode.
# Documents with more lines than this value use u64 internal indexing.
# Range: 1000000–4294967295. Default: 2147483647 (2^31 - 1)
large_document_threshold = 2147483647
```

### Config Resolution Rules

| Setting | Absent | Invalid Value | Out of Range |
|---------|--------|---------------|--------------|
| `large_document_threshold` | Default to 2^31-1 | Default + WARN log | Clamp to [1M–u32::MAX] + WARN |

---

## Design Decisions

### Decision 1: Fenwick Tree over Segment Tree

**Chosen: Fenwick Tree (Binary Indexed Tree)**

Rationale:
1. **Memory efficient**: Uses a single flat array — no per-node pointers or child references
2. **Cache friendly**: Sequential array access pattern during prefix-sum traversal
3. **Simple implementation**: ~50 lines of code for the core operations (query + update)
4. **Proven O(log n)**: Both prefix-sum queries and point updates are exactly O(log n)
5. **Scintilla precedent**: Scintilla's `Partitioning` uses a similar cumulative approach

Trade-offs accepted:
- No lazy propagation (range updates are O(k × log n) not O(log n)) — acceptable since bulk visibility changes affect a bounded number of lines per operation
- Insertion/deletion requires partial rebuild — acceptable since line insertions are rare relative to lookups

### Decision 2: Visibility in Fenwick Tree vs. Separate Bitmap

**Chosen: Effective height in Fenwick Tree (height × visible)**

The Fenwick tree stores the *effective* display height for each line: `height` if visible, `0` if hidden. This means:
- `set_visible(line, false)` updates the Fenwick tree by subtracting the line's height
- `set_visible(line, true)` updates the Fenwick tree by adding the line's stored height
- The actual height is preserved in the HeightStore regardless of visibility

This avoids needing a separate prefix-sum tree for visibility and keeps all lookups in a single tree traversal.

### Decision 3: Fold State Orthogonal to Visibility

The fold expanded/collapsed state is stored **independently** from line visibility. The mapping layer does not enforce fold semantics — it only stores the boolean. The consuming fold engine (or `exclude-show-filter`) is responsible for calling `set_visible` appropriately when folds are toggled. This matches Requirement 10 (Dual Hiding Mechanism Support).

---

## Correctness Properties

The following properties are suitable for property-based testing with the `proptest` crate. Each property is universal — it must hold for all valid inputs.

### Property 1: Display Line Count Invariant

**Statement:** The total display line count always equals the sum of effective heights (height × visible) across all document lines.

```
∀ ContractionState CS:
    CS.lines_displayed() == Σ(if CS.get_visible(d) { CS.get_height(d) } else { 0 })
    for d in 0..CS.lines_in_doc()
```

**Validates: Requirements 1.8, 2.8, 6.7**

### Property 2: Doc-to-Display Round-Trip

**Statement:** For any visible document line `d`, converting to display and back to doc yields the same document line.

```
∀ ContractionState CS, ∀ d where 0 ≤ d < CS.lines_in_doc() ∧ CS.get_visible(d):
    CS.doc_from_display(CS.display_from_doc(d)).doc_line == d
```

**Validates: Requirement 1 AC 10**

### Property 3: Display-to-Doc Monotonicity

**Statement:** `doc_from_display` is monotonically non-decreasing: for display lines `a < b`, the resulting document lines satisfy `doc(a) ≤ doc(b)`.

```
∀ ContractionState CS, ∀ a, b where 0 ≤ a < b < CS.lines_displayed():
    CS.doc_from_display(a).doc_line <= CS.doc_from_display(b).doc_line
```

**Validates: Requirement 1 AC 4**

### Property 4: Doc-to-Display Monotonicity

**Statement:** For visible document lines `a < b`, their display positions satisfy `display(a) < display(b)`.

```
∀ ContractionState CS, ∀ a, b where a < b ∧ CS.get_visible(a) ∧ CS.get_visible(b):
    CS.display_from_doc(a) < CS.display_from_doc(b)
```

**Validates: Requirement 1 AC 1**

### Property 5: Hidden Lines Contribute Zero Display Lines

**Statement:** Hiding a line decreases the display line count by exactly that line's height.

```
∀ ContractionState CS, ∀ d where CS.get_visible(d):
    let old_count = CS.lines_displayed();
    let h = CS.get_height(d);
    CS.set_visible(d, d, false);
    CS.lines_displayed() == old_count - h
```

**Validates: Requirements 2.3, 2.8**

### Property 6: Show Restores Display Lines

**Statement:** Showing a hidden line increases the display line count by exactly that line's stored height.

```
∀ ContractionState CS, ∀ d where ¬CS.get_visible(d):
    let old_count = CS.lines_displayed();
    let h = CS.get_height(d);
    CS.set_visible(d, d, true);
    CS.lines_displayed() == old_count + h
```

**Validates: Requirement 2 AC 4**

### Property 7: One-to-One Mode Identity

**Statement:** In one-to-one mode, `display_from_doc(n) == n` and `doc_from_display(n).doc_line == n` for all valid n.

```
∀ ContractionState CS where CS.is_one_to_one(), ∀ n where 0 ≤ n < CS.lines_in_doc():
    CS.display_from_doc(DocLine(n)) == DisplayLine(n)
    ∧ CS.doc_from_display(DisplayLine(n)).doc_line == DocLine(n)
    ∧ CS.doc_from_display(DisplayLine(n)).sub_line == SubLine(0)
```

**Validates: Requirement 1 AC 9, Requirement 9 AC 5**

### Property 8: Insert Lines Preserves Existing Mapping

**Statement:** Inserting lines at position `p` does not change the display line offset for any line before `p`.

```
∀ ContractionState CS, ∀ p where 0 ≤ p ≤ CS.lines_in_doc(), ∀ count > 0:
    let before = [CS.display_from_doc(DocLine(d)) for d in 0..p];
    CS.insert_lines(DocLine(p), count);
    for d in 0..p:
        CS.display_from_doc(DocLine(d)) == before[d]
```

**Validates: Requirement 6 AC 1**

### Property 9: Delete Lines Adjusts Count

**Statement:** Deleting `count` lines reduces `lines_in_doc()` by `count` and reduces `lines_displayed()` by the sum of effective heights of the deleted lines.

```
∀ ContractionState CS, ∀ p, count where p + count ≤ CS.lines_in_doc():
    let old_doc = CS.lines_in_doc();
    let old_disp = CS.lines_displayed();
    let deleted_heights = Σ(effective_height(d) for d in p..p+count);
    CS.delete_lines(DocLine(p), count);
    CS.lines_in_doc() == old_doc - count
    ∧ CS.lines_displayed() == old_disp - deleted_heights
```

**Validates: Requirement 6 AC 2**

### Property 10: Height Change Adjusts Display Count

**Statement:** Changing a visible line's height from `old_h` to `new_h` adjusts display line count by `new_h - old_h`.

```
∀ ContractionState CS, ∀ d where CS.get_visible(d), ∀ new_h ≥ 1:
    let old_count = CS.lines_displayed();
    let old_h = CS.get_height(d);
    CS.set_height(d, new_h);
    CS.lines_displayed() == old_count + (new_h - old_h)
```

**Validates: Requirement 4 AC 5**

### Property 11: Height Change on Hidden Line Does Not Affect Display Count

**Statement:** Changing a hidden line's height does not affect the total display line count.

```
∀ ContractionState CS, ∀ d where ¬CS.get_visible(d), ∀ new_h ≥ 1:
    let old_count = CS.lines_displayed();
    CS.set_height(d, new_h);
    CS.lines_displayed() == old_count
```

**Validates: Requirement 4 AC 6**

### Property 12: Show All Restores One-to-One Mode

**Statement:** After `show_all()`, the state is in one-to-one mode, all lines are visible, and `lines_displayed() == lines_in_doc()`.

```
∀ ContractionState CS:
    CS.show_all();
    CS.is_one_to_one() == true
    ∧ CS.lines_displayed() == CS.lines_in_doc()
    ∧ CS.hidden_lines() == false
    ∧ ∀ d in 0..CS.lines_in_doc(): CS.get_visible(d) == true
```

**Validates: Requirements 2.6, 9.3**

### Property 13: Sub-Line Contiguity

**Statement:** For a visible line with height `h > 1`, the display lines for sub-lines 0 through h-1 form a contiguous increasing sequence.

```
∀ ContractionState CS, ∀ d where CS.get_visible(d) ∧ CS.get_height(d) > 1:
    let h = CS.get_height(d);
    for s in 0..h-1:
        CS.display_from_doc_sub(d, SubLine(s+1)).0
            == CS.display_from_doc_sub(d, SubLine(s)).0 + 1
```

**Validates: Requirement 4 AC 8**

### Property 14: Doc-from-Display Never Returns Hidden Line

**Statement:** `doc_from_display` always returns a visible document line.

```
∀ ContractionState CS, ∀ display_line where 0 ≤ display_line < CS.lines_displayed():
    let result = CS.doc_from_display(DisplayLine(display_line));
    CS.get_visible(result.doc_line) == true
```

**Validates: Requirement 1 AC 4**

---

## Testing Strategy

### Unit Tests

- `one_to_one_tests.rs`: Identity mapping, O(1) behaviour, no allocation, transition trigger
- `visibility_tests.rs`: Hide/show single lines, ranges, display count adjustment, show_all reset
- `folding_tests.rs`: set_expanded, get_expanded, expand_all, contracted_next, fold text
- `wrap_tests.rs`: set_height, get_height, sub-line mapping, height on hidden lines
- `incremental_tests.rs`: insert_lines, delete_lines, effects on display count and mapping
- `large_doc_tests.rs`: 64-bit mode construction, large line counts, performance assertions

### Property-Based Tests (proptest)

- Display line count invariant (Property 1)
- Doc-to-display round-trip (Property 2)
- Display-to-doc monotonicity (Property 3)
- Doc-to-display monotonicity (Property 4)
- Hidden lines contribute zero (Property 5)
- Show restores display lines (Property 6)
- One-to-one mode identity (Property 7)
- Insert preserves prior mapping (Property 8)
- Delete adjusts count (Property 9)
- Height change adjusts display count (Property 10)
- Height on hidden line has no effect (Property 11)
- Show all restores one-to-one (Property 12)
- Sub-line contiguity (Property 13)
- Doc-from-display never returns hidden (Property 14)

### Integration Tests

- End-to-end: create ContractionState, simulate document edits (insert/delete lines), apply visibility changes, verify mapping consistency
- Fold simulation: simulate nested folds with hide/show sequences, verify display counts
- Wrap simulation: simulate word wrap height changes across bulk operations, verify display line total
- DocumentWatcher integration: mock document model firing notifications, verify ContractionState updates itself

### Test Infrastructure

- **Testing framework**: `proptest` for property-based tests
- **Minimum proptest iterations**: 100 per property
- **Performance benchmarks**: Criterion.rs benchmarks for lookup latency on 1M-line documents
- **Strategies**: Custom proptest strategies generating valid ContractionState configurations with random visibility/height patterns
