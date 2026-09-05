# Design Document: Caret & Selection (`ff-caret-selection`)

## Overview

The `ff-caret-selection` crate is the **visual presentation layer** for carets, selections, caret-line highlighting, virtual space rendering, and modified-line markers within the FileForgeWorkbench editor. It consumes the logical selection model from `ff-edit-operations` and the colour/style configuration from `ff-theme`, exposing a GUI-independent query API that shell renderers use to paint these visual elements.

### Purpose

- Define caret appearance configuration (style, width, colour, blink period)
- Define selection display configuration (colours, layer mode, EOL fill, visibility)
- Provide caret-line highlight configuration (frame vs fill, sub-line, always-show)
- Compute virtual space caret/selection positions for rendering
- Compute rectangular selection column bands for multi-line display
- Provide multi-caret rendering state (primary vs additional colours)
- Provide modified-line marker rendering state
- Expose a rendering-technology-agnostic query API for the viewport renderer
- Respond to theme hot-reload events by updating visual settings immediately

### Position in Architecture

```
Wave 6 — UI and Rendering

┌──────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                    │
│   Viewport Renderer — draws carets, selections, highlights    │
├──────────────────────────────────────────────────────────────┤
│          THIS CRATE: ff-caret-selection ← Wave 6              │
│   Caret config, selection config, rendering queries           │
├──────────────────────────────────────────────────────────────┤
│  Upstream:                                                    │
│    ff-edit-operations (Wave 4) — SelectionContainer,          │
│      SelectionPosition, SelectionRange, ModifiedLineTracker,  │
│      EditMode, SelectionKind                                  │
│    ff-theme (Wave 6, peer) — element colours, hot-reload      │
│    ff-viewport-scrolling (Wave 4) — viewport geometry         │
│    ff-configuration-system (Wave 2) — config loading          │
│    ff-display-line-mapping (Wave 4) — sub-line info           │
├──────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                      │
└──────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Zero GUI dependencies — stores configuration and exposes query methods; actual drawing is performed by the shell layer
- **Command-Driven (Req 4)**: Caret style/blink/highlight settings configurable via commands registered in `ff-command`
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-caret-selection`
- **Error Message Standards (Req 8)**: All errors follow `[caret] operation: description` format
- **Configuration Namespace (Req 5)**: Settings live under `[caret]` and `[selection]` TOML namespaces

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Input Sources"
        THEME_EVT[ff-theme<br/>PaletteChanged / ElementOverridden]
        CFG_EVT[ff-configuration-system<br/>hot-reload callbacks]
        EDIT_STATE[ff-edit-operations<br/>SelectionContainer state]
        MODE_STATE[ff-edit-operations<br/>EditMode state]
        MOD_STATE[ff-edit-operations<br/>ModifiedLineTracker state]
    end

    subgraph "ff-caret-selection"
        CC[CaretConfig<br/>style, width, colour, blink]
        SC[SelectionConfig<br/>colours, layer, eol, visibility]
        CLH[CaretLineHighlightConfig<br/>mode, frame, colour, layer]
        BM[BlinkModel<br/>period, phase query]
        CRQ[CaretRenderQuery<br/>per-caret geometry]
        SRQ[SelectionRenderQuery<br/>per-range geometry]
        CLRQ[CaretLineRenderQuery<br/>highlight geometry]
        VSR[VirtualSpaceRenderer<br/>beyond-EOL positions]
        RCR[RectSelectionRenderer<br/>column band geometry]
        MCR[MultiCaretRenderer<br/>primary + additional]
        MLM[ModifiedMarkerRenderer<br/>per-line * marker]
        TI[ThemeIntegration<br/>element colour mapping]
    end

    subgraph "Shell Layer"
        GPU[ff-desktop / egui<br/>Painter draws using queries]
    end

    THEME_EVT --> TI
    CFG_EVT --> CC
    CFG_EVT --> SC
    CFG_EVT --> CLH
    EDIT_STATE --> CRQ
    EDIT_STATE --> SRQ
    EDIT_STATE --> RCR
    MODE_STATE --> CC
    MOD_STATE --> MLM

    TI --> CC
    TI --> SC
    TI --> CLH
    TI --> MLM

    CC --> CRQ
    SC --> SRQ
    CLH --> CLRQ
    CRQ --> MCR
    SRQ --> VSR
    CRQ --> BM

    CRQ --> GPU
    SRQ --> GPU
    CLRQ --> GPU
    MCR --> GPU
    RCR --> GPU
    MLM --> GPU
end
```

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| **CaretConfig** | Holds caret style (Invisible/Line/Block), width, primary/additional colours, overstrike override |
| **SelectionConfig** | Holds selection visibility, layer mode, EOL fill, all element colour pairs |
| **CaretLineHighlightConfig** | Holds highlight mode (None/Frame/Fill), frame width, colours, layer, always-show, sub-line flags |
| **BlinkModel** | Stores blink period, exposes `visible_phase(elapsed_ms)` query, reset-on-move |
| **CaretRenderQuery** | Computes per-caret screen rectangles given viewport geometry and font metrics |
| **SelectionRenderQuery** | Computes per-line selection rectangles for stream and multi-caret selections |
| **CaretLineRenderQuery** | Computes caret-line highlight rectangle or frame border geometry |
| **VirtualSpaceRenderer** | Extends caret/selection positions into virtual space beyond line-end |
| **RectSelectionRenderer** | Computes column-band geometry for rectangular selections |
| **MultiCaretRenderer** | Iterates all carets, assigns primary vs additional colour per caret |
| **ModifiedMarkerRenderer** | Queries ModifiedLineTracker and produces marker positions for the prefix area |
| **ThemeIntegration** | Maps theme element colours to config fields, handles hot-reload updates |

---

## Module Structure

```
crates/ff-caret-selection/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API re-exports, crate docs
│   ├── config/
│   │   ├── mod.rs                # Config re-exports
│   │   ├── caret.rs              # CaretConfig, CaretStyle, CaretWidth
│   │   ├── selection.rs          # SelectionConfig, LayerMode, element colours
│   │   └── caret_line.rs         # CaretLineHighlightConfig, HighlightMode
│   ├── blink.rs                  # BlinkModel: period, phase computation
│   ├── render/
│   │   ├── mod.rs                # Render query re-exports
│   │   ├── caret_query.rs        # CaretRenderQuery: per-caret geometry
│   │   ├── selection_query.rs    # SelectionRenderQuery: per-range geometry
│   │   ├── caret_line_query.rs   # CaretLineRenderQuery: highlight geometry
│   │   ├── virtual_space.rs      # VirtualSpaceRenderer: beyond-EOL computation
│   │   ├── rect_selection.rs     # RectSelectionRenderer: column band
│   │   ├── multi_caret.rs        # MultiCaretRenderer: primary + additional
│   │   └── modified_marker.rs    # ModifiedMarkerRenderer: * marker positions
│   ├── theme_integration.rs      # ThemeIntegration: element colour mapping
│   ├── types.rs                  # Shared types: ScreenRect, PixelPosition, etc.
│   └── error.rs                  # CaretSelectionError enum
└── tests/
    ├── caret_config_tests.rs     # CaretConfig property tests
    ├── selection_config_tests.rs # SelectionConfig property tests
    ├── blink_tests.rs            # BlinkModel property tests
    ├── caret_query_tests.rs      # Caret geometry computation tests
    ├── selection_query_tests.rs  # Selection geometry computation tests
    ├── virtual_space_tests.rs    # Virtual space position tests
    ├── rect_selection_tests.rs   # Rectangular selection band tests
    ├── multi_caret_tests.rs      # Multi-caret colour assignment tests
    ├── modified_marker_tests.rs  # Modified marker rendering tests
    └── integration.rs            # End-to-end rendering query tests
```

---

## Data Models

### CaretStyle

```rust
/// The visual shape of the caret.
/// Addresses: Requirement 1, criteria 1.1–1.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaretStyle {
    /// Caret is not drawn.
    Invisible,
    /// Vertical bar with configurable width.
    Line,
    /// Solid rectangle spanning one character cell.
    Block,
}

impl Default for CaretStyle {
    fn default() -> Self {
        CaretStyle::Line
    }
}
```

### CaretWidth

```rust
/// Pixel width for a Line-style caret, clamped to [1, 20].
/// Addresses: Requirement 1, criteria 1.4–1.6
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CaretWidth(u8);

impl CaretWidth {
    /// Create a caret width, clamping to [1, 20].
    pub fn new(pixels: u8) -> Self {
        Self(pixels.clamp(1, 20))
    }

    /// Get the pixel width value.
    pub fn pixels(&self) -> u8 {
        self.0
    }
}

impl Default for CaretWidth {
    fn default() -> Self {
        Self(1)
    }
}
```

### CaretConfig

```rust
/// Complete caret appearance configuration.
/// Addresses: Requirements 1, 2, 3
pub struct CaretConfig {
    /// Current caret style (Invisible, Line, Block).
    style: CaretStyle,
    /// Pixel width for Line style.
    width: CaretWidth,
    /// Primary caret colour (for main SelectionRange caret).
    colour: ColourRGBA,
    /// Additional caret colour (for non-main carets in multi-caret).
    additional_colour: ColourRGBA,
    /// Blink period in milliseconds (0 = no blink).
    blink_period_ms: u32,
    /// Whether overstrike mode forces Block style.
    overstrike_forces_block: bool,
}

impl CaretConfig {
    pub fn new() -> Self;

    /// Get the effective caret style, considering overstrike mode.
    /// When in overstrike mode and overstrike_forces_block is true, returns Block.
    /// Addresses: Requirement 1, criterion 1.3
    pub fn effective_style(&self, edit_mode: EditMode) -> CaretStyle;

    pub fn style(&self) -> CaretStyle;
    pub fn set_style(&mut self, style: CaretStyle);
    pub fn width(&self) -> CaretWidth;
    pub fn set_width(&mut self, width: CaretWidth);
    pub fn colour(&self) -> ColourRGBA;
    pub fn set_colour(&mut self, colour: ColourRGBA);
    pub fn additional_colour(&self) -> ColourRGBA;
    pub fn set_additional_colour(&mut self, colour: ColourRGBA);
    pub fn blink_period_ms(&self) -> u32;
    pub fn set_blink_period_ms(&mut self, period_ms: u32);
}

impl Default for CaretConfig {
    fn default() -> Self {
        Self {
            style: CaretStyle::Line,
            width: CaretWidth::default(),            // 1px
            colour: ColourRGBA::rgb(0, 0, 0),        // black
            additional_colour: ColourRGBA::rgb(127, 127, 127), // grey
            blink_period_ms: 530,
            overstrike_forces_block: true,
        }
    }
}
```

### LayerMode

```rust
/// Controls how a colour overlay is composited with underlying content.
/// Addresses: Requirement 5, criteria 5.5–5.8; Requirement 4, criterion 4.6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerMode {
    /// Opaque background drawn under text, replacing the default background.
    #[default]
    Base,
    /// Translucent overlay alpha-blended over text and background.
    OverText,
}
```

### SelectionConfig

```rust
/// Complete selection display configuration.
/// Addresses: Requirements 5, 6
pub struct SelectionConfig {
    /// Whether selections are rendered visually.
    visible: bool,
    /// Compositing layer mode for selection background.
    layer: LayerMode,
    /// Whether selection background extends past line-end to right edge.
    eol_filled: bool,
    /// Primary selection colours.
    selection_back: ColourRGBA,
    selection_text: Option<ColourRGBA>,
    /// Additional (non-primary multi-selection) colours.
    additional_back: ColourRGBA,
    additional_text: Option<ColourRGBA>,
    /// Secondary selection colours (e.g., find-all highlights).
    secondary_back: ColourRGBA,
    secondary_text: Option<ColourRGBA>,
    /// Inactive pane selection colours.
    inactive_back: ColourRGBA,
    inactive_text: Option<ColourRGBA>,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            visible: true,
            layer: LayerMode::Base,
            eol_filled: false,
            selection_back: ColourRGBA::rgb(192, 192, 192),       // #C0C0C0
            selection_text: None,
            additional_back: ColourRGBA::rgb(215, 215, 215),      // #D7D7D7
            additional_text: None,
            secondary_back: ColourRGBA::rgb(176, 176, 176),       // #B0B0B0
            secondary_text: None,
            inactive_back: ColourRGBA::rgba(128, 128, 128, 0x3F), // #8080803F
            inactive_text: None,
        }
    }
}

impl SelectionConfig {
    pub fn new() -> Self;
    pub fn visible(&self) -> bool;
    pub fn set_visible(&mut self, visible: bool);
    pub fn layer(&self) -> LayerMode;
    pub fn set_layer(&mut self, layer: LayerMode);
    pub fn eol_filled(&self) -> bool;
    pub fn set_eol_filled(&mut self, eol_filled: bool);
    pub fn selection_back(&self) -> ColourRGBA;
    pub fn set_selection_back(&mut self, colour: ColourRGBA);
    pub fn selection_text(&self) -> Option<ColourRGBA>;
    pub fn set_selection_text(&mut self, colour: Option<ColourRGBA>);
    pub fn additional_back(&self) -> ColourRGBA;
    pub fn additional_text(&self) -> Option<ColourRGBA>;
    pub fn secondary_back(&self) -> ColourRGBA;
    pub fn secondary_text(&self) -> Option<ColourRGBA>;
    pub fn inactive_back(&self) -> ColourRGBA;
    pub fn inactive_text(&self) -> Option<ColourRGBA>;
    // set_* methods for all colour pairs...
}
```

### HighlightMode

```rust
/// Caret-line highlight mode.
/// Addresses: Requirement 4, criteria 4.1–4.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightMode {
    /// No caret-line highlighting.
    None,
    /// Border/outline around the caret line.
    Frame,
    /// Solid background fill on the caret line.
    Fill,
}

impl Default for HighlightMode {
    fn default() -> Self {
        HighlightMode::Frame
    }
}
```

### CaretLineHighlightConfig

```rust
/// Complete caret-line highlight configuration.
/// Addresses: Requirement 4
pub struct CaretLineHighlightConfig {
    /// Highlight mode: None, Frame, or Fill.
    mode: HighlightMode,
    /// Frame border width in pixels (clamped to [1, line_height/3]).
    frame_width: u8,
    /// Background colour for Fill mode / frame colour for Frame mode.
    colour: ColourRGBA,
    /// Compositing layer mode.
    layer: LayerMode,
    /// Whether highlight shows when pane is unfocused.
    always_show: bool,
    /// Whether highlight applies to wrapped sub-line only (vs full document line).
    sub_line: bool,
}

impl Default for CaretLineHighlightConfig {
    fn default() -> Self {
        Self {
            mode: HighlightMode::Frame,
            frame_width: 1,
            colour: ColourRGBA::rgba(0, 0, 0, 30), // subtle highlight
            layer: LayerMode::Base,
            always_show: false,
            sub_line: false,
        }
    }
}

impl CaretLineHighlightConfig {
    pub fn new() -> Self;
    pub fn mode(&self) -> HighlightMode;
    pub fn set_mode(&mut self, mode: HighlightMode);
    pub fn frame_width(&self) -> u8;
    /// Set frame width, clamping to [1, max_width].
    /// Addresses: Requirement 4, criterion 4.5
    pub fn set_frame_width(&mut self, width: u8, line_height: u8);
    pub fn colour(&self) -> ColourRGBA;
    pub fn set_colour(&mut self, colour: ColourRGBA);
    pub fn layer(&self) -> LayerMode;
    pub fn set_layer(&mut self, layer: LayerMode);
    pub fn always_show(&self) -> bool;
    pub fn set_always_show(&mut self, always_show: bool);
    pub fn sub_line(&self) -> bool;
    pub fn set_sub_line(&mut self, sub_line: bool);
}
```

### BlinkModel

```rust
/// Manages caret blink state computation.
/// The model is timer-agnostic — the GUI shell drives the clock.
/// Addresses: Requirement 3
pub struct BlinkModel {
    /// Blink period in milliseconds (0 = always visible).
    period_ms: u32,
    /// Timestamp (ms) of last caret movement / blink reset.
    last_reset_ms: u64,
}

impl BlinkModel {
    pub fn new(period_ms: u32) -> Self;

    /// Query whether the caret should be visible at the given elapsed time.
    /// Returns true when period_ms is 0 (no blink).
    /// Addresses: Requirement 3, criteria 3.3, 3.5
    pub fn is_visible(&self, current_time_ms: u64) -> bool;

    /// Reset the blink cycle to the visible phase (called on caret move).
    /// Addresses: Requirement 3, criterion 3.6
    pub fn reset(&mut self, current_time_ms: u64);

    /// Update the blink period.
    pub fn set_period_ms(&mut self, period_ms: u32);

    /// Get the current blink period.
    pub fn period_ms(&self) -> u32;
}
```

### Rendering Geometry Types

```rust
/// A screen-space rectangle for rendering.
/// GUI-independent — uses logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// A pixel position on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelPosition {
    pub x: f32,
    pub y: f32,
}

/// Font metrics needed for caret/selection geometry computation.
/// Provided by the GUI shell to the render queries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// Width of a single character cell (monospace assumed).
    pub char_width: f32,
    /// Line height in pixels.
    pub line_height: f32,
    /// Baseline offset from top of line.
    pub baseline: f32,
}
```

### CaretRenderInfo

```rust
/// Rendering data for a single caret, ready for the shell to draw.
/// Addresses: Requirements 1, 2, 9
#[derive(Debug, Clone, PartialEq)]
pub struct CaretRenderInfo {
    /// Screen rectangle for the caret graphic.
    pub rect: ScreenRect,
    /// Colour to use for this caret.
    pub colour: ColourRGBA,
    /// Whether this is the primary caret.
    pub is_primary: bool,
    /// The caret style to render.
    pub style: CaretStyle,
    /// Character underneath (for Block style inverse rendering).
    /// None if at end of line or invisible style.
    pub char_under: Option<char>,
}
```

### SelectionRenderInfo

```rust
/// Rendering data for a single line-segment of a selection.
/// A selection range spanning multiple lines produces one SelectionRenderInfo per line.
/// Addresses: Requirements 5, 6, 7, 8
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionRenderInfo {
    /// Screen rectangle for this selection segment.
    pub rect: ScreenRect,
    /// Background colour for this segment.
    pub back_colour: ColourRGBA,
    /// Optional foreground override for text within this segment.
    pub text_colour: Option<ColourRGBA>,
    /// Layer mode for compositing.
    pub layer: LayerMode,
    /// Whether this extends into virtual space.
    pub in_virtual_space: bool,
}
```

### CaretLineRenderInfo

```rust
/// Rendering data for the caret-line highlight.
/// Addresses: Requirement 4
#[derive(Debug, Clone, PartialEq)]
pub struct CaretLineRenderInfo {
    /// Screen rectangle for the highlight (full viewport width).
    pub rect: ScreenRect,
    /// Colour/frame colour.
    pub colour: ColourRGBA,
    /// Highlight mode (Frame or Fill).
    pub mode: HighlightMode,
    /// Frame border width (only relevant for Frame mode).
    pub frame_width: f32,
    /// Layer mode for compositing.
    pub layer: LayerMode,
}
```

### ModifiedMarkerRenderInfo

```rust
/// Rendering data for a single modified-line marker.
/// Addresses: Requirement 10
#[derive(Debug, Clone, PartialEq)]
pub struct ModifiedMarkerRenderInfo {
    /// Screen position where the '*' marker should be drawn.
    pub position: PixelPosition,
    /// Colour for the marker character.
    pub colour: ColourRGBA,
    /// The line number (1-based) this marker is on.
    pub line: u64,
}
```

---

## Public API Surface

### CaretSelectionModel (Top-Level Facade)

```rust
/// The top-level model aggregating all caret and selection visual state.
/// GUI-independent. Owned per editor instance.
/// Addresses: Requirement 11, criterion 11.4
pub struct CaretSelectionModel {
    caret_config: CaretConfig,
    selection_config: SelectionConfig,
    caret_line_config: CaretLineHighlightConfig,
    blink: BlinkModel,
    /// Whether the containing pane currently has keyboard focus.
    has_focus: bool,
}

impl CaretSelectionModel {
    /// Create with default configuration.
    pub fn new() -> Self;

    /// Create from theme-derived settings.
    pub fn from_theme(theme: &ThemeHandle) -> Self;

    // --- Config accessors ---
    pub fn caret_config(&self) -> &CaretConfig;
    pub fn caret_config_mut(&mut self) -> &mut CaretConfig;
    pub fn selection_config(&self) -> &SelectionConfig;
    pub fn selection_config_mut(&mut self) -> &mut SelectionConfig;
    pub fn caret_line_config(&self) -> &CaretLineHighlightConfig;
    pub fn caret_line_config_mut(&mut self) -> &mut CaretLineHighlightConfig;
    pub fn blink(&self) -> &BlinkModel;
    pub fn blink_mut(&mut self) -> &mut BlinkModel;

    // --- Focus state ---
    /// Addresses: Requirement 12, criterion 12.3
    pub fn set_focused(&mut self, focused: bool, current_time_ms: u64);
    pub fn has_focus(&self) -> bool;

    // --- Theme integration ---
    /// Apply updated theme settings. Called on theme hot-reload.
    /// Addresses: Requirement 11, criteria 11.2, 11.3
    pub fn apply_theme(&mut self, theme: &ThemeHandle);
}
```

### CaretRenderQuery

```rust
/// Computes rendering geometry for all carets in the current selection state.
/// Addresses: Requirements 1, 2, 7, 9
pub struct CaretRenderQuery;

impl CaretRenderQuery {
    /// Compute render info for all visible carets.
    /// Returns one CaretRenderInfo per caret that falls within the viewport.
    ///
    /// Parameters:
    /// - `model`: The caret/selection model with configuration
    /// - `selection`: The logical selection state from ff-edit-operations
    /// - `edit_mode`: Current edit mode (for overstrike block override)
    /// - `metrics`: Font metrics from the shell
    /// - `viewport_top_line`: First visible line in the viewport
    /// - `viewport_lines`: Number of visible lines
    /// - `line_lengths`: Callback to query line content length (for virtual space)
    /// - `current_time_ms`: Current timestamp for blink computation
    ///
    /// Addresses: Requirement 1 (shape), 2 (colour), 3 (blink), 7 (virtual space), 9 (multi-caret)
    pub fn compute_carets(
        model: &CaretSelectionModel,
        selection: &SelectionContainer,
        edit_mode: EditMode,
        metrics: &FontMetrics,
        viewport_top_line: u64,
        viewport_lines: u64,
        line_lengths: &dyn Fn(u64) -> u64,
        current_time_ms: u64,
    ) -> Vec<CaretRenderInfo>;

    /// Compute render info for a single caret position.
    /// Used internally and for testing.
    pub fn compute_single_caret(
        position: &SelectionPosition,
        is_primary: bool,
        model: &CaretSelectionModel,
        edit_mode: EditMode,
        metrics: &FontMetrics,
        viewport_top_line: u64,
        line_length: u64,
    ) -> CaretRenderInfo;
}
```

### SelectionRenderQuery

```rust
/// Computes rendering geometry for all selection ranges.
/// Addresses: Requirements 5, 6, 7, 8
pub struct SelectionRenderQuery;

impl SelectionRenderQuery {
    /// Compute render info for all visible selection segments.
    /// Returns one SelectionRenderInfo per visible line-segment.
    ///
    /// Addresses: Requirement 5 (display), 6 (colours), 7 (virtual space)
    pub fn compute_selections(
        model: &CaretSelectionModel,
        selection: &SelectionContainer,
        selection_kind: SelectionKind,
        has_focus: bool,
        metrics: &FontMetrics,
        viewport_top_line: u64,
        viewport_lines: u64,
        line_lengths: &dyn Fn(u64) -> u64,
    ) -> Vec<SelectionRenderInfo>;

    /// Compute render info for a rectangular selection.
    /// Returns one SelectionRenderInfo per line in the rectangle.
    ///
    /// Addresses: Requirement 8
    pub fn compute_rectangular_selection(
        model: &CaretSelectionModel,
        selection: &SelectionContainer,
        metrics: &FontMetrics,
        viewport_top_line: u64,
        viewport_lines: u64,
        line_lengths: &dyn Fn(u64) -> u64,
    ) -> Vec<SelectionRenderInfo>;
}
```

### CaretLineRenderQuery

```rust
/// Computes rendering geometry for the caret-line highlight.
/// Addresses: Requirement 4
pub struct CaretLineRenderQuery;

impl CaretLineRenderQuery {
    /// Compute caret-line highlight geometry for the primary caret.
    /// Returns None if mode is None, or if pane is unfocused and always_show is false.
    ///
    /// Addresses: Requirement 4, criteria 4.1–4.13
    pub fn compute_caret_line(
        model: &CaretSelectionModel,
        selection: &SelectionContainer,
        metrics: &FontMetrics,
        viewport_top_line: u64,
        viewport_width: f32,
        sub_line_index: Option<u32>,
    ) -> Option<CaretLineRenderInfo>;
}
```

### ModifiedMarkerRenderer

```rust
/// Computes rendering positions for modified-line markers.
/// Addresses: Requirement 10
pub struct ModifiedMarkerRenderer;

impl ModifiedMarkerRenderer {
    /// Compute marker positions for all modified lines visible in the viewport.
    ///
    /// Addresses: Requirement 10, criteria 10.1–10.5
    pub fn compute_markers(
        tracker: &ModifiedLineTracker,
        marker_colour: ColourRGBA,
        metrics: &FontMetrics,
        prefix_area_x: f32,
        viewport_top_line: u64,
        viewport_lines: u64,
    ) -> Vec<ModifiedMarkerRenderInfo>;
}
```

### ThemeIntegration

```rust
/// Maps theme element colours and config keys to caret/selection settings.
/// Addresses: Requirement 11
pub struct ThemeIntegration;

impl ThemeIntegration {
    /// Load all caret/selection settings from the theme handle.
    /// Falls back to defaults for any unset elements.
    ///
    /// Addresses: Requirement 11, criterion 11.3
    pub fn load_from_theme(theme: &ThemeHandle) -> CaretSelectionModel;

    /// Update an existing model from a theme change event.
    /// Applies only the changed elements.
    ///
    /// Addresses: Requirement 11, criterion 11.2
    pub fn apply_theme_event(
        model: &mut CaretSelectionModel,
        theme: &ThemeHandle,
        event: &ThemeEvent,
    );

    /// Map element colour queries for caret-specific elements.
    /// Returns the colour for: Caret, CaretAdditional, CaretLineBack,
    /// SelectionBack, SelectionText, etc.
    pub fn resolve_element_colour(
        theme: &ThemeHandle,
        element: Element,
    ) -> Option<ColourRGBA>;
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-caret-selection crate.
/// Formatted per Error Message Standards (Req 8): `[caret] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CaretSelectionError {
    /// Invalid caret width value (outside [1, 20] before clamping).
    #[error("[caret] set_width: value {value} is outside range [1, 20], clamped to {clamped}")]
    CaretWidthClamped { value: u8, clamped: u8 },

    /// Invalid frame width (exceeds line_height / 3).
    #[error("[caret] set_frame_width: value {value} exceeds max ({max}) for line height {line_height}")]
    FrameWidthClamped { value: u8, max: u8, line_height: u8 },

    /// Theme element colour not found (non-fatal, falls back to default).
    #[error("[caret] theme: element '{element}' not defined in theme, using default")]
    ElementNotInTheme { element: String },

    /// Configuration key has invalid value.
    #[error("[caret] config: key '{key}' has invalid value '{value}' — using default {default}")]
    InvalidConfig { key: String, value: String, default: String },

    /// Font metrics have zero or negative values.
    #[error("[caret] render: invalid font metrics — char_width={char_width}, line_height={line_height}")]
    InvalidFontMetrics { char_width: f32, line_height: f32 },
}
```

---

## Integration Points

### With `ff-edit-operations` (Wave 4 — upstream)

- **Consumed types**: `SelectionContainer`, `SelectionRange`, `SelectionPosition`, `EditMode`, `SelectionKind`, `ModifiedLineTracker`
- **Data flow**: The caret-selection crate reads the logical selection state to determine what carets and selections to render. It does NOT modify selection state.
- **Dependency direction**: `ff-caret-selection` depends on `ff-edit-operations` (read-only consumer)
- **Key interactions**:
  - `SelectionContainer::ranges()` → iterate all ranges for multi-caret/multi-selection rendering
  - `SelectionContainer::main_range()` → identify primary caret for colour assignment
  - `SelectionPosition::virtual_space` → compute virtual space caret offset
  - `EditMode` → determine if overstrike block caret applies
  - `ModifiedLineTracker::is_modified(line)` → determine which lines show `*` marker

### With `ff-theme` (Wave 6 — peer)

- **Consumed types**: `ThemeHandle`, `ColourRGBA`, `Element`, `ThemeEvent`
- **Data flow**: Theme provides all colour values for caret, selection, caret-line, and modified marker rendering. Theme hot-reload events trigger visual setting updates.
- **Dependency direction**: `ff-caret-selection` depends on `ff-theme`
- **Key interactions**:
  - `ThemeHandle::element_colour(Element::CaretForeground)` → primary caret colour
  - `ThemeHandle::element_colour(Element::AdditionalCaretForeground)` → additional caret colour
  - `ThemeHandle::element_colour(Element::CaretLineBackground)` → caret-line fill colour
  - `ThemeHandle::element_colour(Element::SelectionBackground)` → selection background
  - `ThemeHandle::element_colour(Element::SelectionForeground)` → selection text override
  - `ThemeEvent::PaletteChanged` → full visual refresh
  - `ThemeEvent::ElementOverridden` → targeted element update
  - `ThemeHandle::colour(ColourToken::EditorModifiedIndicator)` → modified marker colour

### With `ff-viewport-scrolling` (Wave 4 — upstream)

- **Consumed data**: `top_line`, `visible_count`, `viewport_width`, `line_height` (from `ViewportModel`)
- **Data flow**: Viewport geometry determines which carets/selections are visible and where they are positioned on screen.
- **Dependency direction**: `ff-caret-selection` reads viewport state (no dependency on the crate itself — values passed as parameters to render queries)
- **Key interactions**:
  - `ViewportModel::top_line()` → viewport_top_line parameter for clipping
  - `ViewportModel::visible_count()` → viewport_lines parameter for clipping
  - Scroll-to-caret policies in viewport-scrolling ensure the caret is always visible after movement (cross-reference; not a compile-time dependency)

### With `ff-configuration-system` (Wave 2 — upstream)

- **Consumed API**: Config hot-reload callbacks, typed key access
- **Data flow**: Configuration provides initial values and hot-reload notifications for all settings under `[caret]` and `[selection]` namespaces.
- **Key config keys**:
  - `caret.style` → CaretStyle (default: "line")
  - `caret.width` → CaretWidth (default: 1)
  - `caret.blink_period_ms` → u32 (default: 530)
  - `caret.line_highlight_mode` → HighlightMode (default: "frame")
  - `caret.line_frame_width` → u8 (default: 1)
  - `caret.line_always_show` → bool (default: false)
  - `caret.line_sub_line` → bool (default: false)
  - `selection.visible` → bool (default: true)
  - `selection.layer` → LayerMode (default: "base")
  - `selection.eol_filled` → bool (default: false)

### With `ff-display-line-mapping` (Wave 4 — upstream)

- **Consumed information**: Wrapped sub-line indices for `sub_line` caret-line highlight
- **Data flow**: When `sub_line` is true and word-wrap is active, the display-line-mapping provides the sub-line index for the caret position so the highlight covers only the wrapped sub-line.
- **Key interactions**:
  - Sub-line index is passed as a parameter to `CaretLineRenderQuery::compute_caret_line()`
  - No compile-time crate dependency — sub-line info is passed by the shell layer

### With `ff-desktop` (Shell Layer — downstream consumer)

- **Provided API**: All render query methods
- **Data flow**: The shell layer calls render query APIs each frame, passing font metrics and viewport geometry, and receives render info structures to draw with egui.
- **Key interactions**:
  - Shell calls `CaretRenderQuery::compute_carets()` → draws caret rectangles
  - Shell calls `SelectionRenderQuery::compute_selections()` → draws selection backgrounds
  - Shell calls `CaretLineRenderQuery::compute_caret_line()` → draws highlight
  - Shell calls `ModifiedMarkerRenderer::compute_markers()` → draws `*` characters
  - Shell manages the blink timer and passes `current_time_ms` to blink queries
  - Shell gives/removes focus via `CaretSelectionModel::set_focused()`

---

## Correctness Properties

These properties are suitable for property-based testing using the `proptest` crate.

### Property 1: Caret Width Clamping

**Statement**: For any input value `v: u8`, `CaretWidth::new(v).pixels()` is always in [1, 20].

**Validates**: Requirement 1.6

```
∀ v ∈ u8: 1 ≤ CaretWidth::new(v).pixels() ≤ 20
```

### Property 2: Blink Visibility Determinism

**Statement**: For any `period_ms > 0` and `current_time_ms`, the blink model returns a deterministic result: visible when `(current_time_ms - last_reset_ms) % period_ms < period_ms / 2`, hidden otherwise. For `period_ms == 0`, always returns true.

**Validates**: Requirements 3.3, 3.5

```
∀ period ∈ u32, ∀ t ∈ u64:
  period == 0 ⟹ is_visible(t) == true
  period > 0 ⟹ is_visible(t) == ((t - last_reset) % period < period / 2)
```

### Property 3: Blink Reset Always Makes Visible

**Statement**: After calling `blink.reset(t)`, `blink.is_visible(t)` always returns true regardless of previous state.

**Validates**: Requirement 3.6

```
∀ state, ∀ t ∈ u64: reset(t) ⟹ is_visible(t) == true
```

### Property 4: Overstrike Mode Forces Block

**Statement**: When `edit_mode == Overstrike` and `overstrike_forces_block == true`, `effective_style()` always returns `Block`, regardless of the configured style.

**Validates**: Requirement 1.3

```
∀ style ∈ CaretStyle:
  effective_style(Overstrike) == Block when overstrike_forces_block
```

### Property 5: Selection Colour Assignment by Context

**Statement**: For any selection range, the assigned colours depend on context: primary range uses `selection_back`, non-primary ranges use `additional_back`, unfocused pane uses `inactive_back`.

**Validates**: Requirements 6.1, 6.10

```
∀ range ∈ selection.ranges():
  is_main(range) ∧ focused ⟹ colour == selection_back
  ¬is_main(range) ∧ focused ⟹ colour == additional_back
  ¬focused ⟹ colour == inactive_back
```

### Property 6: Virtual Space Caret Position

**Statement**: For any `SelectionPosition` with `virtual_space > 0` on a line of length `L`, the rendered caret x-position equals `(L + virtual_space) × char_width`.

**Validates**: Requirement 7.1

```
∀ pos with vs > 0, ∀ line_length L:
  caret_x == (L + pos.virtual_space) × metrics.char_width
```

### Property 7: Primary Caret Uses Primary Colour

**Statement**: In the output of `compute_carets()`, exactly one caret has `is_primary == true` and uses `caret_config.colour`; all others use `caret_config.additional_colour`.

**Validates**: Requirements 9.2, 9.3

```
∀ output of compute_carets():
  count(c | c.is_primary) == 1
  ∀ c: c.is_primary ⟹ c.colour == config.colour
  ∀ c: ¬c.is_primary ⟹ c.colour == config.additional_colour
```

### Property 8: Caret Line Highlight Visibility Rule

**Statement**: `compute_caret_line()` returns `None` when mode is `None`, or when `always_show` is false and `has_focus` is false. It returns `Some` in all other cases where the primary caret line is in the viewport.

**Validates**: Requirements 4.1, 4.8, 4.9

```
∀ state:
  mode == None ⟹ result == None
  ¬always_show ∧ ¬has_focus ⟹ result == None
  mode ≠ None ∧ (always_show ∨ has_focus) ∧ in_viewport ⟹ result == Some(_)
```

### Property 9: Frame Width Clamping

**Statement**: For any frame width input `w` and line height `h`, the stored frame width is clamped to `[1, h / 3]`.

**Validates**: Requirement 4.5

```
∀ w ∈ u8, ∀ h ∈ u8 (h ≥ 3):
  set_frame_width(w, h) ⟹ 1 ≤ frame_width ≤ h / 3
```

### Property 10: EOL Fill Extends Selection to Viewport Edge

**Statement**: When `eol_filled` is true and a selected line's content ends before the viewport edge, the selection rectangle width extends to `viewport_width`. When false, it ends at the last selected character position.

**Validates**: Requirement 5.9

```
∀ selected_line with content_end < viewport_width:
  eol_filled ⟹ selection_rect.x + selection_rect.width == viewport_width
  ¬eol_filled ⟹ selection_rect.x + selection_rect.width == content_end × char_width
```

### Property 11: All Carets Blink In Phase

**Statement**: At any given timestamp, either all carets are visible or all are hidden — never a mix.

**Validates**: Requirement 9.6

```
∀ t, ∀ carets in compute_carets(t):
  (∀ c: c is visible) ∨ (∀ c: c is hidden)
  [equivalently: blink model applies uniformly to all carets]
```

### Property 12: Rectangular Selection Column Consistency

**Statement**: For a rectangular selection, all rendered selection segments have the same left x-position and the same width (the column band is vertically aligned).

**Validates**: Requirement 8.1

```
∀ segments in compute_rectangular_selection():
  ∀ s₁, s₂: s₁.rect.x == s₂.rect.x ∧ s₁.rect.width == s₂.rect.width
```

---

## Configuration Keys

All configuration keys live under reserved namespaces to avoid conflicts (Cross-cutting Req 5).

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `caret.style` | string | `"line"` | Caret shape: "invisible", "line", "block" |
| `caret.width` | u8 | `1` | Line-style caret width in pixels [1, 20] |
| `caret.blink_period_ms` | u32 | `530` | Blink cycle duration; 0 = no blink |
| `caret.overstrike_forces_block` | bool | `true` | Block caret in overstrike mode |
| `caret.line_highlight_mode` | string | `"frame"` | Caret-line highlight: "none", "frame", "fill" |
| `caret.line_frame_width` | u8 | `1` | Frame border width in pixels |
| `caret.line_layer` | string | `"base"` | Caret-line layer: "base", "over_text" |
| `caret.line_always_show` | bool | `false` | Show highlight when unfocused |
| `caret.line_sub_line` | bool | `false` | Highlight wrapped sub-line only |
| `selection.visible` | bool | `true` | Whether selections are rendered |
| `selection.layer` | string | `"base"` | Selection layer: "base", "over_text" |
| `selection.eol_filled` | bool | `false` | Extend selection past line-end |

Colour values are defined by theme element colours (not config keys) — see `ff-theme` Element enum.

---

## Testing Strategy

### Unit Tests

- `CaretConfig`: verify defaults, clamping, effective_style logic
- `SelectionConfig`: verify defaults, colour assignment rules
- `CaretLineHighlightConfig`: verify frame width clamping, mode transitions
- `BlinkModel`: verify visibility computation, reset behaviour, zero-period always-visible

### Property-Based Tests (proptest)

- Properties 1–12 as defined in Correctness Properties above
- Generators for `SelectionPosition` (arbitrary line/column/virtual_space)
- Generators for `FontMetrics` (positive non-zero values)
- Generators for viewport parameters (top_line, visible_count)

### Integration Tests

- Full render query pipeline: construct model + selection state → compute all render infos → verify consistency
- Theme hot-reload: apply theme change → verify all colours update immediately
- Focus transitions: focused → unfocused → verify inactive colours apply

---

## Mouse Selection Design (Requirements 13-14)

### Mouse Selection in the Editor Canvas

The editor canvas renders text via `ui.painter()` calls -- egui's built-in selection machinery does not apply. A custom selection layer is required in `ff-desktop`'s editor panel render loop:

- **State**: `canvas_selection: Option<(DocPos, DocPos)>` on `TabState` -- anchor and end as (line, col) pairs.
- **Input handling**: `egui::Response::drag_started()`, `drag_delta()`, `drag_released()` on the canvas `Rect`. Screen coordinates are converted to (line, col) using the viewport's `top_line`, `line_height`, and `char_width`.
- **Rendering**: For each visible line that intersects the selection range, a filled `Rect` is drawn behind the text using `SelectionBack` colour from the active theme.
- **Ctrl+C**: Detected in the editor panel's key-event loop. When `canvas_selection` is `Some`, the selected text is extracted from the document model and written to the OS clipboard via `ff-clipboard`.

### Read-Only Panel Selectability (Requirement 14)

For POM, Settings, and status bar panels, the change is minimal: replace `ui.label(text)` calls with `ui.label(egui::RichText::new(text)).selectable(true)` (or use `egui::SelectableLabel`). egui handles selection and Ctrl+C automatically for these widgets. No custom code is needed beyond the widget swap.

---

## Design Decisions and Rationale

### Decision 1: Query-Based API (No Retained Render State)

The crate does **not** maintain a retained cache of render geometries. Instead, all rendering data is computed on each frame via query methods. This ensures:
- No stale state after selection changes, theme reloads, or viewport scrolls
- Simpler model with no invalidation logic
- Consistent with egui's immediate-mode rendering philosophy
- Performance is acceptable because compute is O(visible_carets + visible_selection_segments), typically very small

### Decision 2: GUI-Independent Model, Shell-Driven Timer

The `BlinkModel` stores only the period and last-reset timestamp. The actual timer (animation frame callback) is owned by the GUI shell. This maintains GUI independence per Cross-cutting Requirement 2 and allows different shells to use their native timing mechanisms.

### Decision 3: Colour Resolution Via Theme Elements

All colours are resolved through `ff-theme`'s Element colour system rather than storing raw colour values in configuration files. This ensures themes have full control over visual appearance and that colour changes propagate immediately on theme switch/hot-reload.

### Decision 4: Parameter-Passing Over Crate Dependencies

Integration with `ff-viewport-scrolling` and `ff-display-line-mapping` is done via parameters (viewport_top_line, sub_line_index) rather than compile-time crate dependencies. This keeps the crate's dependency graph minimal and testable in isolation.
