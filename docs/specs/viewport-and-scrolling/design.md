# Design Document: Viewport & Scrolling (`ff-viewport-scrolling`)

## Overview

The `ff-viewport-scrolling` crate is the **GUI-independent viewport management layer** for the FileForgeWorkbench editor. It owns the logical window into a document, including vertical and horizontal scroll state, scrollbar models, caret visibility policies, smooth scrolling, and cursor-viewport coordination.

### Purpose

- Track the visible portion of a document (`top_line`, `visible_count`, `horizontal_offset`)
- Provide a full-range vertical scrollbar model with proportional thumb sizing
- Provide a horizontal scrollbar model for wide content
- Execute scroll commands (Page Up/Down, Line Up/Down, scroll-to-position) with clamping
- Enforce configurable caret visibility policies (Slop, Strict, Jumps, Even)
- Support smooth (pixel-level) scrolling as an alternative to line-level jumps
- Maintain column affinity for natural vertical cursor movement
- Integrate with `display-line-mapping` for wrapped/folded/excluded lines
- Emit viewport state-change events for GUI renderers and observers
- Persist and restore viewport state for session management

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│  Queries viewport state for rendering, forwards input events │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-viewport-scrolling ← Wave 4                 │
│  Viewport model, scrollbar, scroll commands, caret policies  │
├─────────────────────────────────────────────────────────────┤
│  Peers: ff-document-model (line count, content metrics)      │
│         ff-display-line-mapping (display↔document mapping)   │
│         ff-command (scroll command registration/dispatch)     │
│         ff-configuration-system (policy settings)            │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Zero GUI framework dependencies — viewport logic is testable without egui/winit/wgpu
- **Command-Driven (Req 4)**: All scroll operations are registered as commands via `ff-command`
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-viewport-scrolling`
- **Error Message Standards (Req 8)**: Errors follow `[viewport] operation: description` format
- **Configuration (Req 5)**: Caret policies, scroll mode, and wheel speed are configurable via `[viewport]` TOML namespace

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Input Sources"
        KB[Keyboard: PgUp/PgDn/Arrows]
        MW[Mouse Wheel]
        SB[Scrollbar Drag]
        CMD[Command Framework]
        LUA[Lua Macros]
    end

    subgraph "ff-viewport-scrolling"
        VM[ViewportModel<br/>top_line, visible_count, offsets]
        SC[ScrollCommands<br/>PageUp, PageDown, LineUp, LineDown, ScrollTo]
        CM[CursorModel<br/>cursor_line, cursor_column, affinity]
        CP[CaretPolicyEngine<br/>Slop, Strict, Jumps, Even]
        VSB[VerticalScrollbar<br/>fraction ↔ top_line mapping]
        HSB[HorizontalScrollbar<br/>offset ↔ extent mapping]
        SM[SmoothScrollEngine<br/>pixel_offset, targets]
        EV[EventEmitter<br/>ViewportChanged notifications]
        SNAP[StateSnapshot<br/>serialisation for persistence]
    end

    subgraph "Upstream / Peers"
        DM[ff-document-model<br/>line_count]
        DLM[ff-display-line-mapping<br/>DisplayLineMapper trait]
        CFG[ff-configuration-system<br/>policy settings]
        LOG[ff-logging]
    end

    KB --> CMD
    MW --> SC
    SB --> VSB
    CMD --> SC
    LUA --> CMD

    SC --> VM
    SC --> CM
    CM --> CP
    CP --> VM
    VSB --> VM
    HSB --> VM
    SM --> VM
    VM --> EV

    VM --> DM
    VM --> DLM
    CP --> CFG
    VM --> LOG
    SNAP --> VM
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **ViewportModel** | Core state: `top_line`, `visible_count`, `horizontal_offset`, `pixel_offset`, `total_display_lines`; clamping arithmetic; resize handling |
| **CursorModel** | Cursor position: `cursor_line`, `cursor_column`, `column_affinity`; arrow key movement; line-length clamping |
| **CaretPolicyEngine** | Computes viewport adjustments needed to keep the caret visible per configured policy flags |
| **VerticalScrollbar** | Pure-function mapping between `top_line` and scrollbar fraction; proportional thumb computation; precision mode for large files |
| **HorizontalScrollbar** | Pure-function mapping between `horizontal_offset` and scrollbar fraction; max-extent tracking |
| **SmoothScrollEngine** | Pixel-level scroll target computation; exposes target positions for GUI animation; manages `pixel_offset` sub-line state |
| **ScrollCommands** | Command definitions for registration with `ff-command`; translates command invocations to viewport mutations |
| **EventEmitter** | Dispatches `ViewportChanged` events to registered observers |
| **StateSnapshot** | Serialisable viewport state for session persistence and restore |

---

## Components and Interfaces

```
crates/ff-viewport-scrolling/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API re-exports, crate docs
│   ├── viewport.rs               # ViewportModel struct, core state, clamping
│   ├── cursor.rs                 # CursorModel: position, affinity, movement
│   ├── caret_policy.rs           # CaretPolicy config, CaretPolicyEngine
│   ├── scrollbar/
│   │   ├── mod.rs                # Scrollbar re-exports
│   │   ├── vertical.rs           # VerticalScrollbar: fraction ↔ top_line
│   │   └── horizontal.rs         # HorizontalScrollbar: offset ↔ extent
│   ├── smooth.rs                 # SmoothScrollEngine: pixel-level scrolling
│   ├── commands.rs               # Scroll command definitions and handlers
│   ├── events.rs                 # ViewportChanged event, observer trait
│   ├── snapshot.rs               # StateSnapshot for persistence
│   ├── display_mapper.rs         # DisplayLineMapper trait (consumed from ff-display-line-mapping)
│   ├── config.rs                 # Configuration loading from [viewport] TOML namespace
│   ├── types.rs                  # Newtypes: DisplayLine, ScrollFraction, PixelOffset
│   └── error.rs                  # ViewportError enum
└── tests/
    ├── viewport_tests.rs         # ViewportModel state + clamping tests
    ├── cursor_tests.rs           # CursorModel movement + affinity tests
    ├── caret_policy_tests.rs     # CaretPolicyEngine behaviour tests
    ├── scrollbar_tests.rs        # Vertical + horizontal scrollbar mapping tests
    ├── smooth_scroll_tests.rs    # SmoothScrollEngine target computation tests
    ├── commands_tests.rs         # Scroll command dispatch tests
    ├── snapshot_tests.rs         # Serialise/deserialise round-trip tests
    └── integration.rs            # End-to-end: scroll + cursor + policy + scrollbar
```

---

## Data Models

### Core Newtypes

```rust
/// A 1-based display line number (accounts for wrapping/folding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayLine(pub u64);

/// A scrollbar position as a fraction in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ScrollFraction(pub f64);

impl ScrollFraction {
    /// Create a clamped fraction in [0.0, 1.0].
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

/// A pixel offset for sub-line smooth scrolling.
/// Range: [0, line_height).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PixelOffset(pub u32);

/// Scroll mode: line-level jumps or pixel-level smooth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollMode {
    /// Traditional whole-line scrolling (integer top_line values).
    Line,
    /// Pixel-level sub-line scrolling with animation targets.
    Smooth,
}

impl Default for ScrollMode {
    fn default() -> Self {
        Self::Line
    }
}
```

### ViewportModel

```rust
/// The core viewport state. GUI-independent, owned by the editor session.
///
/// Addresses: Requirement 1
pub struct ViewportModel {
    /// First visible display line (1-based).
    top_line: u64,
    /// Number of display lines that fit vertically.
    visible_count: u64,
    /// Horizontal scroll position in pixels.
    horizontal_offset: u64,
    /// Total display lines in the document (from DisplayLineMapper or raw line_count).
    total_display_lines: u64,
    /// Current scroll mode (Line or Smooth).
    scroll_mode: ScrollMode,
    /// Sub-line pixel offset for smooth scrolling (0 when scroll_mode is Line).
    pixel_offset: PixelOffset,
    /// Line height in pixels (set by GUI shell for smooth scroll calculations).
    line_height: u32,
    /// Viewport width in pixels (set by GUI shell).
    viewport_width: u64,
    /// Maximum horizontal extent (longest line width minus viewport width).
    max_horizontal_extent: u64,
    /// Whether a DisplayLineMapper is active.
    has_display_mapper: bool,
}
```

### CursorModel

```rust
/// Cursor position and column affinity tracking.
///
/// Addresses: Requirement 1 (criteria 4–6), Requirement 6
pub struct CursorModel {
    /// Current cursor line (1-based document line).
    cursor_line: u64,
    /// Current cursor column (1-based).
    cursor_column: u64,
    /// Remembered column for vertical movement (column affinity / lastXChosen).
    column_affinity: u64,
    /// Whether column_affinity is measured in pixels (proportional) or columns (monospace).
    affinity_mode: AffinityMode,
}

/// Whether column affinity is tracked in pixel or column units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffinityMode {
    /// Column-based affinity (monospace/character-grid editors).
    Columns,
    /// Pixel-based affinity (proportional-font editors).
    Pixels,
}

impl Default for AffinityMode {
    fn default() -> Self {
        Self::Columns
    }
}
```

### CaretPolicy

```rust
/// Configurable policy controlling how the viewport scrolls to keep the caret visible.
/// Modelled after Scintilla's caret policy flags.
///
/// Addresses: Requirement 5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaretPolicy {
    /// If true, a slop zone is defined near edges.
    pub slop: bool,
    /// If true, the slop zone is enforced strictly (always scroll if in zone).
    pub strict: bool,
    /// If true, scroll by larger jumps (3× slop) to reduce scroll frequency.
    pub jumps: bool,
    /// If true, apply slop symmetrically to both edges.
    pub even: bool,
    /// Number of lines (vertical) or pixels (horizontal) for the slop zone.
    pub slop_value: u32,
}

impl Default for CaretPolicy {
    fn default() -> Self {
        Self {
            slop: false,
            strict: false,
            jumps: false,
            even: false,
            slop_value: 0,
        }
    }
}

/// Separate policies for vertical and horizontal axes.
///
/// Addresses: Requirement 5 AC 7
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaretPolicyConfig {
    pub vertical: CaretPolicy,
    pub horizontal: CaretPolicy,
}
```

### VerticalScrollbar

```rust
/// Pure-function vertical scrollbar model. Maps between top_line and a visual fraction.
///
/// Addresses: Requirement 4, Requirement 13
pub struct VerticalScrollbar;

impl VerticalScrollbar {
    /// Compute the scrollbar position fraction from viewport state.
    /// Returns 0.0 when top_line == 1, 1.0 when top_line == max_top_line.
    ///
    /// Addresses: Requirement 4 AC 1
    pub fn position_fraction(top_line: u64, max_top_line: u64) -> ScrollFraction;

    /// Compute the thumb size ratio (visible_count / total_display_lines).
    /// Returns 1.0 when entire document fits in viewport.
    ///
    /// Addresses: Requirement 4 AC 2
    pub fn thumb_ratio(visible_count: u64, total_display_lines: u64) -> f64;

    /// Convert a scrollbar fraction to a top_line value.
    /// Uses 64-bit integer arithmetic for precision with large files.
    ///
    /// Addresses: Requirement 4 AC 3, Requirement 13 AC 1
    pub fn fraction_to_top_line(fraction: ScrollFraction, max_top_line: u64) -> u64;

    /// Whether the scrollbar should be disabled (document fits in viewport).
    ///
    /// Addresses: Requirement 4 AC 7
    pub fn is_disabled(total_display_lines: u64, visible_count: u64) -> bool;

    /// Precision drag: given a pixel delta and track height, compute fine-grained top_line change.
    ///
    /// Addresses: Requirement 13 AC 3
    pub fn precision_drag_delta(
        pixel_delta: i32,
        track_height: u32,
        total_display_lines: u64,
        max_top_line: u64,
    ) -> i64;
}

/// Feedback data for tooltip display during scrollbar drag.
///
/// Addresses: Requirement 13 AC 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollbarFeedback {
    /// Current top_line during drag.
    pub current_line: u64,
    /// Total document lines.
    pub total_lines: u64,
}
```

### HorizontalScrollbar

```rust
/// Pure-function horizontal scrollbar model.
///
/// Addresses: Requirement 7
pub struct HorizontalScrollbar;

impl HorizontalScrollbar {
    /// Compute horizontal scrollbar position fraction.
    ///
    /// Addresses: Requirement 7 AC 1
    pub fn position_fraction(horizontal_offset: u64, max_horizontal_extent: u64) -> ScrollFraction;

    /// Convert a scrollbar fraction to a horizontal_offset value.
    ///
    /// Addresses: Requirement 7 AC 2
    pub fn fraction_to_offset(fraction: ScrollFraction, max_horizontal_extent: u64) -> u64;

    /// Whether the horizontal scrollbar should be disabled.
    ///
    /// Addresses: Requirement 7 AC 3
    pub fn is_disabled(max_horizontal_extent: u64) -> bool;
}
```

### SmoothScrollEngine

```rust
/// Manages pixel-level smooth scrolling state and target computation.
/// The viewport model computes targets; the GUI shell performs animation interpolation.
///
/// Addresses: Requirement 9
pub struct SmoothScrollEngine {
    /// Whether smooth scrolling is currently active.
    enabled: bool,
    /// Current sub-line pixel offset [0, line_height).
    pixel_offset: PixelOffset,
    /// Target top_line for ongoing animation (None if idle).
    target_top_line: Option<u64>,
    /// Target pixel offset for ongoing animation.
    target_pixel_offset: Option<PixelOffset>,
}

impl SmoothScrollEngine {
    /// Compute the target pixel position for a scroll-to-line command.
    ///
    /// Addresses: Requirement 9 AC 4
    pub fn compute_scroll_target(
        &self,
        target_line: u64,
        line_height: u32,
    ) -> SmoothScrollTarget;

    /// Get the pixel-accurate scrollbar fraction (accounts for sub-line offset).
    ///
    /// Addresses: Requirement 9 AC 6
    pub fn pixel_accurate_fraction(
        &self,
        top_line: u64,
        max_top_line: u64,
        line_height: u32,
    ) -> ScrollFraction;

    /// Reset to line-level scrolling (pixel_offset = 0).
    pub fn reset(&mut self);
}

/// Target for smooth scroll animation, exposed to the GUI shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmoothScrollTarget {
    /// Target top_line after animation completes.
    pub target_line: u64,
    /// Target pixel offset within that line.
    pub target_pixel_offset: PixelOffset,
    /// Total pixel distance to animate.
    pub pixel_distance: i64,
}
```

### DisplayLineMapper Trait

```rust
/// Trait consumed from ff-display-line-mapping. Translates between
/// document lines and display lines for correct scrolling with wrapping/folding.
///
/// Addresses: Requirement 11
pub trait DisplayLineMapper: Send + Sync {
    /// Total number of display lines (accounting for wraps and folds).
    fn total_display_lines(&self) -> u64;

    /// Convert a document line to its first display line.
    fn doc_to_display(&self, doc_line: u64) -> u64;

    /// Convert a display line to its document line.
    fn display_to_doc(&self, display_line: u64) -> u64;

    /// Whether a document line is currently visible (not folded/excluded).
    fn is_visible(&self, doc_line: u64) -> bool;

    /// Number of display lines produced by a document line (wrapping).
    fn display_lines_for_doc_line(&self, doc_line: u64) -> u64;
}
```

### StateSnapshot

```rust
/// Serialisable viewport state for session persistence.
///
/// Addresses: Requirement 12
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ViewportSnapshot {
    /// Top line at time of snapshot.
    pub top_line: u64,
    /// Cursor line at time of snapshot.
    pub cursor_line: u64,
    /// Cursor column at time of snapshot.
    pub cursor_column: u64,
    /// Horizontal offset at time of snapshot.
    pub horizontal_offset: u64,
    /// Column affinity at time of snapshot.
    pub column_affinity: u64,
}
```

### ViewportChanged Event

```rust
/// Event emitted after any viewport state mutation.
///
/// Addresses: Requirement 10 AC 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportChanged {
    /// New top_line value.
    pub top_line: u64,
    /// New cursor_line value.
    pub cursor_line: u64,
    /// New cursor_column value.
    pub cursor_column: u64,
    /// New horizontal_offset value.
    pub horizontal_offset: u64,
    /// Whether this change was triggered by a cursor move (vs. explicit scroll).
    pub cursor_triggered: bool,
}

/// Observer trait for viewport state changes.
pub trait ViewportObserver: Send + Sync {
    /// Called after any viewport state mutation.
    fn on_viewport_changed(&self, event: &ViewportChanged);
}
```

---

## Public API Surface

### ViewportModel — Construction and Geometry

```rust
impl ViewportModel {
    /// Create a new viewport model with default state.
    pub fn new() -> Self;

    /// Create with a known document line count.
    pub fn with_line_count(total_display_lines: u64) -> Self;

    /// Update the visible line count (called when GUI window resizes).
    /// Clamps top_line if it now exceeds max_top_line.
    ///
    /// Addresses: Requirement 1 AC 8
    pub fn set_visible_count(&mut self, count: u64);

    /// Update the total display line count (called when document changes or mapper updates).
    /// Clamps top_line if it now exceeds max_top_line.
    pub fn set_total_display_lines(&mut self, total: u64);

    /// Set the line height in pixels (for smooth scroll calculations).
    pub fn set_line_height(&mut self, height: u32);

    /// Set the viewport width in pixels.
    pub fn set_viewport_width(&mut self, width: u64);

    /// Set the maximum horizontal extent (longest line - viewport width).
    pub fn set_max_horizontal_extent(&mut self, extent: u64);

    /// Attach a DisplayLineMapper for wrapped/folded content.
    ///
    /// Addresses: Requirement 11 AC 1
    pub fn set_display_mapper(&mut self, mapper: Option<Box<dyn DisplayLineMapper>>);
}
```

### ViewportModel — Accessors

```rust
impl ViewportModel {
    /// Current top_line (1-based).
    /// Addresses: Requirement 1 AC 1
    pub fn top_line(&self) -> u64;

    /// Current visible_count.
    /// Addresses: Requirement 1 AC 2
    pub fn visible_count(&self) -> u64;

    /// Current horizontal_offset in pixels.
    /// Addresses: Requirement 1 AC 3
    pub fn horizontal_offset(&self) -> u64;

    /// Total display lines in the document.
    pub fn total_display_lines(&self) -> u64;

    /// Maximum valid top_line: max(1, total_display_lines - visible_count + 1).
    /// Addresses: Requirement 1 AC 10
    pub fn max_top_line(&self) -> u64;

    /// Current scroll mode (Line or Smooth).
    pub fn scroll_mode(&self) -> ScrollMode;

    /// Current sub-line pixel offset (0 in Line mode).
    pub fn pixel_offset(&self) -> PixelOffset;

    /// Whether the vertical scrollbar should be disabled.
    pub fn is_vertical_scrollbar_disabled(&self) -> bool;

    /// Whether the horizontal scrollbar should be disabled.
    pub fn is_horizontal_scrollbar_disabled(&self) -> bool;
}
```

### ViewportModel — Vertical Scrolling

```rust
impl ViewportModel {
    /// Scroll down by one page (visible_count lines).
    /// Addresses: Requirement 2 AC 1
    pub fn scroll_page_down(&mut self);

    /// Scroll up by one page (visible_count lines).
    /// Addresses: Requirement 2 AC 2
    pub fn scroll_page_up(&mut self);

    /// Scroll down by one line.
    /// Addresses: Requirement 2 AC 3
    pub fn scroll_line_down(&mut self);

    /// Scroll up by one line.
    /// Addresses: Requirement 2 AC 4
    pub fn scroll_line_up(&mut self);

    /// Scroll to a specific line (clamped to [1, max_top_line]).
    /// Addresses: Requirement 2 AC 5
    pub fn scroll_to_line(&mut self, line: u64);

    /// Scroll to the top of the document.
    pub fn scroll_to_top(&mut self);

    /// Scroll to the bottom of the document.
    pub fn scroll_to_bottom(&mut self);

    /// Handle mouse wheel vertical scroll (configurable lines per tick).
    /// Addresses: Requirement 8 AC 1
    pub fn scroll_wheel_vertical(&mut self, ticks: i32, lines_per_tick: u32);

    /// Set scroll mode (Line or Smooth).
    /// Addresses: Requirement 9 AC 1
    pub fn set_scroll_mode(&mut self, mode: ScrollMode);
}
```

### ViewportModel — Horizontal Scrolling

```rust
impl ViewportModel {
    /// Set horizontal offset (clamped to [0, max_horizontal_extent]).
    /// Addresses: Requirement 7 AC 4
    pub fn set_horizontal_offset(&mut self, offset: u64);

    /// Handle mouse wheel horizontal scroll.
    /// Addresses: Requirement 8 AC 4
    pub fn scroll_wheel_horizontal(&mut self, ticks: i32, pixels_per_tick: u32);

    /// Scroll horizontally to ensure a column is visible.
    /// Addresses: Requirement 3 AC 12
    pub fn ensure_column_visible(&mut self, column_pixel_position: u64);
}
```

### ViewportModel — Scrollbar Interaction

```rust
impl ViewportModel {
    /// Get the vertical scrollbar fraction for the current state.
    /// Addresses: Requirement 4 AC 1
    pub fn vertical_scrollbar_fraction(&self) -> ScrollFraction;

    /// Get the vertical scrollbar thumb ratio.
    /// Addresses: Requirement 4 AC 2
    pub fn vertical_scrollbar_thumb_ratio(&self) -> f64;

    /// Apply a vertical scrollbar drag to a fraction position.
    /// Addresses: Requirement 4 AC 3
    pub fn apply_scrollbar_drag(&mut self, fraction: ScrollFraction);

    /// Apply a precision scrollbar drag (Shift+drag).
    /// Addresses: Requirement 13 AC 3
    pub fn apply_precision_drag(&mut self, pixel_delta: i32, track_height: u32);

    /// Get scrollbar feedback data for tooltip.
    /// Addresses: Requirement 13 AC 5
    pub fn scrollbar_feedback(&self) -> ScrollbarFeedback;

    /// Get horizontal scrollbar fraction.
    /// Addresses: Requirement 7 AC 1
    pub fn horizontal_scrollbar_fraction(&self) -> ScrollFraction;

    /// Apply a horizontal scrollbar drag.
    /// Addresses: Requirement 7 AC 2
    pub fn apply_horizontal_drag(&mut self, fraction: ScrollFraction);
}
```

### CursorModel — Position and Movement

```rust
impl CursorModel {
    /// Create a new cursor at line 1, column 1.
    pub fn new() -> Self;

    /// Current cursor line (1-based).
    /// Addresses: Requirement 1 AC 4
    pub fn cursor_line(&self) -> u64;

    /// Current cursor column (1-based).
    /// Addresses: Requirement 1 AC 5
    pub fn cursor_column(&self) -> u64;

    /// Current column affinity value.
    /// Addresses: Requirement 1 AC 6
    pub fn column_affinity(&self) -> u64;

    /// Move cursor down one line. Returns the new cursor_line.
    /// Applies column affinity to determine target column.
    ///
    /// Addresses: Requirement 3 AC 1, Requirement 6 AC 1
    pub fn move_down(&mut self, current_line_length: u64, total_lines: u64) -> u64;

    /// Move cursor up one line. Returns the new cursor_line.
    ///
    /// Addresses: Requirement 3 AC 2, Requirement 6 AC 1
    pub fn move_up(&mut self, target_line_length: u64) -> u64;

    /// Move cursor left one column.
    /// Updates column_affinity.
    ///
    /// Addresses: Requirement 3 AC 6, Requirement 6 AC 2
    pub fn move_left(&mut self);

    /// Move cursor right one column.
    /// Updates column_affinity.
    ///
    /// Addresses: Requirement 3 AC 7, Requirement 6 AC 2
    pub fn move_right(&mut self, current_line_length: u64);

    /// Set cursor to a specific position (e.g., click).
    /// Resets column_affinity.
    ///
    /// Addresses: Requirement 3 AC 5, Requirement 6 AC 5
    pub fn set_position(&mut self, line: u64, column: u64);

    /// Move cursor to the beginning of the current line.
    /// Resets column_affinity.
    pub fn move_home(&mut self);

    /// Move cursor to the end of the current line.
    /// Resets column_affinity.
    pub fn move_end(&mut self, current_line_length: u64);
}
```

### CaretPolicyEngine — Viewport Adjustment

```rust
impl CaretPolicyEngine {
    /// Create an engine with the given policy configuration.
    pub fn new(config: CaretPolicyConfig) -> Self;

    /// Compute the top_line adjustment needed after a vertical cursor move.
    /// Returns the new top_line (or the current one if no scroll needed).
    ///
    /// Addresses: Requirement 5 AC 1–6
    pub fn compute_vertical_scroll(
        &self,
        cursor_line: u64,
        top_line: u64,
        visible_count: u64,
        max_top_line: u64,
    ) -> u64;

    /// Compute the horizontal_offset adjustment needed after a horizontal cursor move.
    ///
    /// Addresses: Requirement 5 AC 7
    pub fn compute_horizontal_scroll(
        &self,
        cursor_pixel_x: u64,
        horizontal_offset: u64,
        viewport_width: u64,
        max_horizontal_extent: u64,
    ) -> u64;

    /// Update the policy configuration (e.g., from hot-reloaded settings).
    ///
    /// Addresses: Requirement 5 AC 9
    pub fn set_config(&mut self, config: CaretPolicyConfig);
}
```

### Scroll Command Definitions

```rust
/// Scroll commands registered with the command framework.
///
/// Addresses: Requirement 10 AC 1
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollCommand {
    /// Scroll viewport up by one line.
    ScrollLineUp,
    /// Scroll viewport down by one line.
    ScrollLineDown,
    /// Scroll viewport up by one page.
    ScrollPageUp,
    /// Scroll viewport down by one page.
    ScrollPageDown,
    /// Scroll viewport to a specific line.
    ScrollToLine(u64),
    /// Scroll viewport to the top.
    ScrollToTop,
    /// Scroll viewport to the bottom.
    ScrollToBottom,
    /// Set horizontal scroll offset.
    ScrollHorizontal(u64),
}
```

### ViewportModel — Snapshot and Restore

```rust
impl ViewportModel {
    /// Create a serialisable snapshot of the current viewport state.
    ///
    /// Addresses: Requirement 12 AC 1
    pub fn snapshot(&self, cursor: &CursorModel) -> ViewportSnapshot;

    /// Restore from a persisted snapshot, clamping to current document bounds.
    ///
    /// Addresses: Requirement 12 AC 2–4
    pub fn restore(&mut self, snapshot: &ViewportSnapshot, cursor: &mut CursorModel);
}
```

### ViewportModel — Event Emission

```rust
impl ViewportModel {
    /// Register a viewport observer.
    pub fn add_observer(&mut self, observer: Box<dyn ViewportObserver>);

    /// Remove a viewport observer.
    pub fn remove_observer(&mut self, id: u64);
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-viewport-scrolling crate.
/// Formatted per Error Message Standards (Req 8): `[viewport] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ViewportError {
    /// Attempted to set visible_count to zero.
    #[error("[viewport] set_visible_count: visible_count must be at least 1")]
    ZeroVisibleCount,

    /// Attempted to set line_height to zero.
    #[error("[viewport] set_line_height: line_height must be at least 1")]
    ZeroLineHeight,

    /// Scroll target line is invalid (zero).
    #[error("[viewport] scroll_to_line: target line 0 is invalid (must be >= 1)")]
    InvalidScrollTarget,

    /// Display line mapper returned inconsistent data.
    #[error("[viewport] {operation}: display mapper inconsistency — {detail}")]
    MapperInconsistency {
        operation: String,
        detail: String,
    },

    /// Snapshot restoration failed due to incompatible state.
    #[error("[viewport] restore: snapshot field '{field}' value {value} exceeds document bounds (max: {max})")]
    SnapshotOutOfBounds {
        field: String,
        value: u64,
        max: u64,
    },

    /// Configuration value is invalid.
    #[error("[viewport] config: key '{key}' has invalid value '{value}' — using default {default}")]
    InvalidConfig {
        key: String,
        value: String,
        default: String,
    },
}
```

---

## Integration Points

### With `ff-document-model` (Wave 4 — peer/upstream)

- **Dependency direction**: ff-viewport-scrolling depends on ff-document-model
- **API consumed**: `Document::line_count()` for `total_display_lines` when no display mapper is active; `Document::line_end()` for line-length queries (horizontal extent calculation)
- **Integration**: The viewport model queries line count to compute `max_top_line`. When the document changes (inserts/deletions), the owning editor session updates `total_display_lines`
- **Note**: The `ff-document-model` viewport manager (`Document::scroll_page_down`, etc.) is the MVP-level viewport. This crate (`ff-viewport-scrolling`) provides the full-featured replacement with caret policies, smooth scrolling, and scrollbar models. The owning session delegates to this crate instead of the document model's simple viewport methods

### With `ff-display-line-mapping` (Wave 4 — peer)

- **Dependency direction**: ff-viewport-scrolling depends on the `DisplayLineMapper` trait (defined in ff-display-line-mapping or re-exported here)
- **API consumed**: `DisplayLineMapper::total_display_lines()`, `doc_to_display()`, `display_to_doc()`, `is_visible()`
- **Integration**: When wrapping or folding is active, the viewport operates on display lines rather than document lines. The mapper translates scroll positions between coordinate systems
- **Fallback**: When no mapper is provided, identity mapping is assumed (1 doc line = 1 display line)

### With `ff-command` (Wave 2 — upstream)

- **Dependency direction**: ff-viewport-scrolling depends on ff-command
- **API consumed**: `CommandRegistry::register()` for scroll command registration; `CommandId` for command identity
- **Integration**: Scroll commands (`ScrollLineUp`, `ScrollPageDown`, etc.) are registered at session startup. The command framework dispatches them; this crate handles execution
- **Undo integration**: Scroll commands are NOT recorded on the undo stack (Requirement 10 AC 6)

### With `ff-configuration-system` (Wave 2 — upstream)

- **Dependency direction**: ff-viewport-scrolling depends on ff-configuration-system
- **API consumed**: Typed config access for `[viewport]` namespace settings
- **Integration**: Caret policies, scroll mode, lines-per-wheel-tick, and smooth scroll settings are loaded from config and hot-reloadable

### With `ff-logging` (Foundation — upstream)

- **Dependency direction**: ff-viewport-scrolling depends on ff-logging
- **API consumed**: `log_info!`, `log_warn!`, `log_debug!` macros
- **Usage**: Scroll mode changes logged at INFO; config warnings at WARN; caret policy decisions at DEBUG
- **Log prefix**: `[viewport]`

### With `ff-edit-operations` (Wave 4 — downstream consumer)

- **Dependency direction**: ff-edit-operations may consume viewport state for cursor-relative operations
- **Integration**: The editor session coordinates between edit-operations (which moves cursor) and viewport-scrolling (which scrolls to follow). The session calls `CaretPolicyEngine::compute_vertical_scroll()` after each cursor move

### With `ff-startup-and-session` (Wave 8 — downstream consumer)

- **Dependency direction**: ff-startup-and-session consumes `ViewportSnapshot` for persistence
- **Integration**: On file close, the session serialises `ViewportSnapshot`. On file reopen, it calls `restore()` with the saved snapshot

### Dependency Direction Summary

```
ff-logging           ← ff-viewport-scrolling
ff-document-model    ← ff-viewport-scrolling (line_count queries)
ff-display-line-mapping ← ff-viewport-scrolling (DisplayLineMapper trait)
ff-command           ← ff-viewport-scrolling (command registration)
ff-configuration-system ← ff-viewport-scrolling (policy config)
ff-viewport-scrolling ← ff-edit-operations (session coordination)
ff-viewport-scrolling ← ff-startup-and-session (snapshot persistence)
```

---

## Configuration

ff-viewport-scrolling owns the `[viewport]` namespace in the workbench TOML configuration file.

### TOML Schema

```toml
[viewport]
# Scroll mode: "line" or "smooth"
# Default: "line"
scroll_mode = "line"

# Lines scrolled per mouse wheel tick.
# Range: 1–20. Default: 3
lines_per_wheel_tick = 3

# Pixels scrolled per horizontal wheel tick.
# Range: 1–100. Default: 20
pixels_per_horizontal_tick = 20

# Vertical caret policy flags
[viewport.caret_policy.vertical]
slop = false
strict = false
jumps = false
even = false
slop_value = 0

# Horizontal caret policy flags
[viewport.caret_policy.horizontal]
slop = false
strict = false
jumps = false
even = false
slop_value = 0

# Column affinity mode: "columns" or "pixels"
# Default: "columns"
affinity_mode = "columns"
```

### Config Resolution Rules

| Setting | Absent | Invalid Value | Out of Range |
|---------|--------|---------------|--------------|
| `scroll_mode` | Default to "line" | Default to "line" + WARN | N/A |
| `lines_per_wheel_tick` | Default to 3 | Default to 3 + WARN | Clamp to [1–20] + WARN |
| `pixels_per_horizontal_tick` | Default to 20 | Default to 20 + WARN | Clamp to [1–100] + WARN |
| `slop_value` | Default to 0 | Default to 0 + WARN | Clamp to [0–50] + WARN |
| `affinity_mode` | Default to "columns" | Default to "columns" + WARN | N/A |

---

## Correctness Properties

The following properties are suitable for property-based testing with the `proptest` crate. Each property is universal — it must hold for all valid inputs.

### Property 1: Scroll Clamping Invariant

**Statement:** After any scroll operation, `top_line` is always in the valid range `[1, max_top_line]`.

```
∀ viewport V, ∀ scroll_operation S:
    apply(S, V);
    V.top_line() >= 1 ∧ V.top_line() <= V.max_top_line()
```

**Validates: Requirements 1.10, 2.1, 2.2, 2.3, 2.4, 2.5**

### Property 2: Page Down/Up Symmetry

**Statement:** If `top_line` is at position P, scrolling page down then page up returns to the same position (unless clamped at boundaries).

```
∀ viewport V where V.top_line() > V.visible_count() ∧ V.top_line() + V.visible_count() <= V.max_top_line():
    let original = V.top_line();
    V.scroll_page_down();
    V.scroll_page_up();
    V.top_line() == original
```

**Validates: Requirements 2.1, 2.2**

### Property 3: Scroll Idempotence at Boundaries

**Statement:** Scrolling up when at line 1 has no effect. Scrolling down when at max_top_line has no effect.

```
∀ viewport V where V.top_line() == 1:
    V.scroll_page_up();
    V.top_line() == 1
    ∧ V.scroll_line_up();
    V.top_line() == 1

∀ viewport V where V.top_line() == V.max_top_line():
    V.scroll_page_down();
    V.top_line() == V.max_top_line()
    ∧ V.scroll_line_down();
    V.top_line() == V.max_top_line()
```

**Validates: Requirements 2.8, 2.9**

### Property 4: Scrollbar Round-Trip

**Statement:** Converting `top_line` to a fraction and back produces the original `top_line`.

```
∀ top_line T ∈ [1, max_top_line], ∀ max_top_line M > 1:
    let f = VerticalScrollbar::position_fraction(T, M);
    let T2 = VerticalScrollbar::fraction_to_top_line(f, M);
    T2 == T
```

**Validates: Requirements 4.8**

### Property 5: Scrollbar Monotonicity

**Statement:** The scrollbar fraction is monotonically non-decreasing as `top_line` increases.

```
∀ T1, T2 where 1 ≤ T1 < T2 ≤ max_top_line:
    VerticalScrollbar::position_fraction(T1, max_top_line).0
    <= VerticalScrollbar::position_fraction(T2, max_top_line).0
```

**Validates: Requirements 13.2**

### Property 6: Column Affinity Preservation

**Statement:** When moving through a short line, column_affinity is preserved and restored on the next sufficiently long line.

```
∀ cursor C with column_affinity A, ∀ short_line_length < A, ∀ long_line_length >= A:
    C.move_down(short_line_length, total_lines);
    C.cursor_column() == short_line_length;   // clamped to end
    C.column_affinity() == A;                  // preserved
    C.move_down(long_line_length, total_lines);
    C.cursor_column() == A;                    // restored
```

**Validates: Requirements 6.1, 6.3, 6.4**

### Property 7: Horizontal Offset Clamping

**Statement:** After any horizontal scroll operation, `horizontal_offset` is always in `[0, max_horizontal_extent]`.

```
∀ viewport V, ∀ horizontal_scroll_operation H:
    apply(H, V);
    V.horizontal_offset() >= 0 ∧ V.horizontal_offset() <= V.max_horizontal_extent()
```

**Validates: Requirements 7.4**

### Property 8: Caret Policy — Cursor Always Visible After Scroll

**Statement:** After the caret policy engine computes a new `top_line`, the cursor line is within the visible range `[top_line, top_line + visible_count - 1]` (accounting for slop if strict is false).

```
∀ cursor_line CL, ∀ visible_count VC, ∀ max_top_line M:
    let new_top = engine.compute_vertical_scroll(CL, top_line, VC, M);
    CL >= new_top ∧ CL < new_top + VC
```

**Validates: Requirements 5.1, 5.6**

### Property 9: Resize Clamps Top Line

**Statement:** After a resize that reduces `visible_count`, `top_line` never exceeds the new `max_top_line`.

```
∀ viewport V, ∀ new_visible_count NVC where NVC >= 1:
    V.set_visible_count(NVC);
    V.top_line() <= V.max_top_line()
```

**Validates: Requirements 1.8**

### Property 10: Smooth Scroll Pixel Offset Range

**Statement:** When smooth scrolling is active, `pixel_offset` is always in `[0, line_height)`.

```
∀ viewport V where V.scroll_mode() == Smooth:
    V.pixel_offset().0 < V.line_height()
```

**Validates: Requirements 9.3**

### Property 11: Snapshot Restore Clamping

**Statement:** After restoring a snapshot on a shorter document, all values are clamped to valid bounds.

```
∀ snapshot S, ∀ viewport V with total_display_lines T, cursor C:
    V.restore(&S, &mut C);
    V.top_line() <= V.max_top_line()
    ∧ C.cursor_line() <= T
    ∧ C.cursor_line() >= 1
```

**Validates: Requirements 12.2, 12.3, 12.4**

### Property 12: Thumb Ratio Bounds

**Statement:** The vertical scrollbar thumb ratio is always in `(0.0, 1.0]`.

```
∀ visible_count VC >= 1, ∀ total_display_lines TDL >= 1:
    let ratio = VerticalScrollbar::thumb_ratio(VC, TDL);
    ratio > 0.0 ∧ ratio <= 1.0
```

**Validates: Requirements 4.2**

---

## Testing Strategy

### Unit Tests

- `viewport_tests.rs`: Scroll clamping at all boundaries, max_top_line computation, resize behaviour, mode switching
- `cursor_tests.rs`: Arrow key movement, boundary clamping, position setting, affinity reset
- `caret_policy_tests.rs`: All policy flag combinations, slop zone enforcement, jump sizing, symmetry
- `scrollbar_tests.rs`: Fraction ↔ top_line round-trips, thumb ratio, precision drag, monotonicity, large-file integer precision
- `smooth_scroll_tests.rs`: Target computation, pixel offset maintenance, mode transitions
- `commands_tests.rs`: Each scroll command dispatches correctly, non-undo-stack behaviour
- `snapshot_tests.rs`: Serialise/deserialise round-trip, clamping on restore to shorter document

### Property-Based Tests (proptest)

- Scroll clamping invariant (Property 1)
- Page Down/Up symmetry (Property 2)
- Scroll idempotence at boundaries (Property 3)
- Scrollbar round-trip (Property 4)
- Scrollbar monotonicity (Property 5)
- Column affinity preservation (Property 6)
- Horizontal offset clamping (Property 7)
- Caret policy cursor visibility (Property 8)
- Resize clamps top line (Property 9)
- Smooth scroll pixel offset range (Property 10)
- Snapshot restore clamping (Property 11)
- Thumb ratio bounds (Property 12)

### Integration Tests

- End-to-end: create viewport + cursor → scroll → verify caret policy → verify scrollbar state
- Display mapper integration: attach a mock mapper with folded lines → verify scroll skips them
- Session lifecycle: create viewport → scroll → snapshot → restore on shorter document → verify clamping

### Test Infrastructure

- **Mock DisplayLineMapper**: In-memory implementation with configurable fold/wrap state
- **Testing framework**: `proptest` for property-based tests, standard `#[test]` for unit tests
- **Minimum proptest iterations**: 100 per property
- **Fixtures**: Viewport configurations covering edge cases (1-line file, file == viewport, file > viewport, million-line files)
