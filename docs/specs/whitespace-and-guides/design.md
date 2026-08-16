# Design Document: Whitespace & Guides (`ff-whitespace-guides`)

## Overview

The `ff-whitespace-guides` crate is the **whitespace visibility and structural guide subsystem** for FileForgeWorkbench. It manages the data model, configuration, and per-line metadata computation for four visual annotation concerns: whitespace glyph visibility, indent guides, edge column indicators, and wrap continuation markers.

### Purpose

- Define enums, settings structs, and configuration keys for whitespace visibility modes, tab draw styles, indent guide modes, edge column modes, wrap visual flags, and wrap indentation modes
- Compute per-line rendering metadata: whitespace glyph positions, indent guide columns, edge column hit information, and wrap marker placement
- Provide toggle commands registered with the command-framework for quick mode cycling
- Integrate with `ff-theme` for colour resolution and `ff-config` for hot-reload settings
- Remain fully GUI-independent — expose only data types and query APIs; rendering is the GUI shell's responsibility

### Position in Architecture

```
Wave 6 — UI and Rendering (depends on Wave 5 Command Engine)

┌──────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                    │
│   Viewport Renderer — draws glyphs, guides, edges, markers   │
├──────────────────────────────────────────────────────────────┤
│          THIS CRATE: ff-whitespace-guides ← Wave 6            │
│   Settings model, per-line queries, toggle commands           │
├──────────────────────────────────────────────────────────────┤
│  Upstream:                                                    │
│    ff-document-model (Wave 4) — line content, tab size        │
│    ff-display-line-mapping (Wave 4) — sub-line/wrap info      │
│    ff-theme (Wave 6, peer) — colour resolution                │
│    ff-config (Wave 2) — settings storage, hot-reload          │
│    ff-command (Wave 2) — toggle command registration          │
├──────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                      │
└──────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: No direct filesystem access — content queries go through `ff-document-model`
- **GUI Independence (Req 2)**: Zero GUI dependencies — no egui, winit, wgpu; exposes only data types and query APIs
- **Command-Driven (Req 4)**: Toggle commands registered with `ff-command` for whitespace, indent guides, and edge column
- **Configuration Namespace (Req 5)**: All settings use the `editor.*` namespace; keys are unique across crates
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-whitespace-guides`
- **Error Message Standards (Req 8)**: All errors follow `[whitespace-guides] operation: description` format

### Upstream Dependencies

| Crate | Usage |
|-------|-------|
| `ff-config` (Wave 2) | Read/write settings; hot-reload callbacks for all `editor.*` keys |
| `ff-command` (Wave 2) | Register toggle commands (`ToggleWhitespace`, `ToggleIndentGuides`, `ToggleEdgeColumn`) |
| `ff-document-model` (Wave 4) | Query line content for indent computation; read tab size and indent size |
| `ff-display-line-mapping` (Wave 4) | Query sub-line count per document line for wrap marker placement |
| `ff-theme` (Wave 6) | Resolve colours for whitespace glyphs, indent guides, edge indicators, wrap markers |
| `ff-logging` (Wave 0) | Diagnostic output on invalid config values |

### Downstream Consumers

| Crate | Usage |
|-------|-------|
| `ff-desktop` (GUI shell) | Reads `WhitespaceSettings` and per-line query results to drive painting |
| `ff-line-wrap-toggle` (Wave 9) | Notifies this crate when wrap mode changes (enables/disables wrap markers) |

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Consumers [Rendering Shell]
        SHELL[ff-desktop<br/>egui viewport renderer]
    end

    subgraph ff-whitespace-guides [ff-whitespace-guides Crate]
        SETTINGS[WhitespaceSettings<br/>aggregated effective config]
        WS_Q[WhitespaceQuery<br/>per-line glyph positions]
        IG_Q[IndentGuideQuery<br/>per-line guide columns]
        EDGE_Q[EdgeQuery<br/>edge column metadata]
        WRAP_Q[WrapMarkerQuery<br/>per-sub-line markers]
        CMDS[Toggle Commands<br/>cycle modes, persist]
        CFGINT[ConfigIntegration<br/>hot-reload listener]
        THEME_INT[ThemeIntegration<br/>colour resolver]
    end

    subgraph Upstream [Upstream Crates]
        DOC[ff-document-model<br/>line content, tab_size]
        DLM[ff-display-line-mapping<br/>sub-line heights]
        THEME[ff-theme<br/>palette, element colours]
        CFG[ff-config<br/>settings, hot-reload]
        CMD[ff-command<br/>command registry]
        LOG[ff-logging]
    end

    SHELL -->|query| SETTINGS
    SHELL -->|query per line| WS_Q
    SHELL -->|query per line| IG_Q
    SHELL -->|query| EDGE_Q
    SHELL -->|query per sub-line| WRAP_Q

    CFGINT -->|subscribe hot-reload| CFG
    CFGINT -->|update| SETTINGS
    THEME_INT -->|palette_changed| THEME
    CMDS -->|register| CMD
    CMDS -->|persist| CFG

    WS_Q -->|line content| DOC
    IG_Q -->|line content, tab_size| DOC
    IG_Q -->|sub-line span| DLM
    WRAP_Q -->|sub-line count| DLM
    SETTINGS --> LOG
end
```

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| **WhitespaceSettings** | Aggregates effective values of all config keys into a single immutable snapshot. Rebuilt on hot-reload. |
| **WhitespaceQuery** | Given a line's text and the current settings, returns positions and types of whitespace glyphs to render. |
| **IndentGuideQuery** | Given a line's text, surrounding lines' indentation, and settings, returns the set of guide column positions (including active guide). |
| **EdgeQuery** | Returns edge column positions and colours for the current configuration. Viewport-level (not per-line). |
| **WrapMarkerQuery** | Given a document line's sub-line count and settings, returns which sub-lines need start/end/margin markers. |
| **Toggle Commands** | Implements `ToggleWhitespace`, `ToggleIndentGuides`, `ToggleEdgeColumn` command handlers. Cycles modes and persists. |
| **ConfigIntegration** | Registers hot-reload callbacks for `editor.whitespace_*`, `editor.indent_*`, `editor.edge_*`, `editor.wrap_*` keys. |
| **ThemeIntegration** | Subscribes to `ThemeEvent::PaletteChanged` and refreshes resolved colours for all visual elements. |

---

## Module Structure

```
crates/ff-whitespace-guides/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── settings.rs                 # WhitespaceSettings struct, builder
│   ├── modes/
│   │   ├── mod.rs                  # Mode enum re-exports
│   │   ├── whitespace_visibility.rs # WhitespaceVisibility enum
│   │   ├── tab_draw_mode.rs        # TabDrawMode enum
│   │   ├── indent_guide_mode.rs    # IndentGuideMode enum
│   │   ├── edge_mode.rs           # EdgeMode enum
│   │   ├── wrap_visual_flag.rs    # WrapVisualFlag bitfield
│   │   ├── wrap_visual_location.rs # WrapVisualLocation enum
│   │   └── wrap_indent_mode.rs    # WrapIndentMode enum
│   ├── query/
│   │   ├── mod.rs                  # Query module re-exports
│   │   ├── whitespace.rs          # WhitespaceQuery: per-line glyph positions
│   │   ├── indent_guides.rs      # IndentGuideQuery: guide columns
│   │   ├── edge.rs               # EdgeQuery: edge column metadata
│   │   └── wrap_markers.rs       # WrapMarkerQuery: sub-line markers
│   ├── indent/
│   │   ├── mod.rs                  # Indent computation re-exports
│   │   ├── level.rs              # indent_level_of() line analysis
│   │   └── scan.rs               # forward/backward blank-line scanning
│   ├── config_integration.rs      # Hot-reload callback registration
│   ├── theme_integration.rs       # ThemeEvent subscription, colour cache
│   ├── commands.rs                 # Toggle command implementations
│   ├── colours.rs                  # Resolved colour cache struct
│   ├── keys.rs                     # Configuration key constants
│   ├── types.rs                    # GlyphPosition, GuideColumn newtypes
│   └── error.rs                    # WhitespaceGuidesError enum
└── tests/
    ├── whitespace_query_tests.rs   # Whitespace glyph position tests
    ├── indent_guide_tests.rs       # Indent guide column computation tests
    ├── edge_query_tests.rs         # Edge column tests
    ├── wrap_marker_tests.rs        # Wrap marker placement tests
    ├── settings_tests.rs           # Settings construction and validation
    ├── toggle_command_tests.rs     # Command cycling behaviour tests
    ├── property_tests.rs           # proptest property-based tests
    └── integration.rs              # End-to-end with mock config/theme
```

---

## Data Models

### Mode Enums

```rust
/// Controls when whitespace characters are rendered with visible glyphs.
/// Addresses: Requirement 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WhitespaceVisibility {
    /// No whitespace glyphs rendered (default).
    #[default]
    Invisible,
    /// All spaces and tabs rendered.
    VisibleAlways,
    /// Only spaces/tabs after the first non-whitespace character per line.
    VisibleAfterIndent,
    /// Only leading spaces/tabs before the first non-whitespace character.
    VisibleOnlyInIndent,
}

/// The rendering style for visible tab characters.
/// Addresses: Requirement 2 AC 2, AC 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TabDrawMode {
    /// Rightward arrow spanning the full tab width (default).
    #[default]
    LongArrow,
    /// Horizontal line through the vertical centre of the tab span.
    Strikeout,
}

/// Controls which lines display indent guides.
/// Addresses: Requirement 3 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IndentGuideMode {
    /// No indent guides drawn (default).
    #[default]
    None,
    /// Guides only on lines with actual indentation at that column.
    Real,
    /// Extend guides through blank lines by scanning forward.
    LookForward,
    /// Extend guides through blank lines by scanning both directions.
    LookBoth,
}

/// The rendering style for the edge column indicator.
/// Addresses: Requirement 5 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EdgeMode {
    /// No edge indicator (default).
    #[default]
    None,
    /// Thin vertical line at the configured column.
    Line,
    /// Shaded background beyond the configured column.
    Background,
    /// Multiple vertical lines, each with its own column and colour.
    MultiLine,
}

/// Bitfield controlling which wrap markers are displayed.
/// Addresses: Requirement 6 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WrapVisualFlag(u8);

impl WrapVisualFlag {
    pub const NONE: Self = Self(0);
    pub const END: Self = Self(1);
    pub const START: Self = Self(2);
    pub const MARGIN: Self = Self(4);

    pub fn has_end(self) -> bool { self.0 & 1 != 0 }
    pub fn has_start(self) -> bool { self.0 & 2 != 0 }
    pub fn has_margin(self) -> bool { self.0 & 4 != 0 }

    pub fn from_bits(bits: u8) -> Self { Self(bits & 0x07) }
    pub fn bits(self) -> u8 { self.0 }
}

/// Controls positioning of wrap markers relative to text or display edge.
/// Addresses: Requirement 6 AC 6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WrapVisualLocation {
    /// Markers placed at display edges (default).
    #[default]
    Default,
    /// End marker placed adjacent to last character.
    EndByText,
    /// Start marker placed adjacent to first character of continuation.
    StartByText,
}

/// Controls indentation of continuation sub-lines.
/// Addresses: Requirement 7 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WrapIndentMode {
    /// Fixed offset defined by Wrap_Start_Indent (default).
    #[default]
    Fixed,
    /// Same indentation as the first sub-line.
    Same,
    /// One additional tab stop beyond the first sub-line.
    Indent,
    /// Two additional tab stops beyond the first sub-line.
    DeepIndent,
}
```

### Edge Properties

```rust
/// A column + colour pair for multi-edge configurations.
/// Addresses: Requirement 5 AC 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeProperties {
    /// The column position (0-based character column).
    pub column: u32,
    /// The colour for this edge line (resolved from config or theme).
    pub colour: ColourRGBA,
}
```

### WhitespaceSettings

```rust
/// Aggregated snapshot of all effective whitespace-and-guides settings.
/// Rebuilt on hot-reload. Immutable after construction.
/// Addresses: Requirement 9 AC 2
#[derive(Debug, Clone, PartialEq)]
pub struct WhitespaceSettings {
    // -- Whitespace visibility --
    pub visibility: WhitespaceVisibility,
    pub tab_draw_mode: TabDrawMode,
    pub whitespace_size: u8,

    // -- Indent guides --
    pub indent_guide_mode: IndentGuideMode,
    pub active_guide_column: Option<u32>,

    // -- Edge column --
    pub edge_mode: EdgeMode,
    pub edge_column: u32,
    pub edge_columns: Vec<EdgeProperties>,

    // -- Wrap markers --
    pub wrap_visual_flags: WrapVisualFlag,
    pub wrap_visual_location: WrapVisualLocation,
    pub wrap_indent_mode: WrapIndentMode,
    pub wrap_start_indent: u32,

    // -- Derived state --
    pub wrap_active: bool,
    pub tab_size: u32,
    pub indent_size: u32,
}

impl WhitespaceSettings {
    /// Create settings from the current effective configuration.
    pub fn from_config(config: &ConfigHandle) -> Self;

    /// Check whether any whitespace glyphs would be rendered.
    pub fn is_whitespace_visible(&self) -> bool {
        self.visibility != WhitespaceVisibility::Invisible
    }

    /// Check whether indent guides would be rendered.
    pub fn has_indent_guides(&self) -> bool {
        self.indent_guide_mode != IndentGuideMode::None
    }

    /// Check whether any edge indicator is active.
    pub fn has_edge_indicator(&self) -> bool {
        self.edge_mode != EdgeMode::None
    }

    /// Check whether wrap markers can appear (requires wrap active + flags set).
    pub fn has_wrap_markers(&self) -> bool {
        self.wrap_active && self.wrap_visual_flags.bits() != 0
    }
}
```

### Resolved Colour Cache

```rust
/// Colours resolved from the active theme for all visual elements.
/// Refreshed on theme change events.
/// Addresses: Requirement 2 AC 7–9, Requirement 3 AC 6, Requirement 4 AC 4,
///            Requirement 5 (edge colours), Requirement 6 AC 8
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedColours {
    /// Foreground colour for whitespace glyphs (dot, arrow, strikeout).
    pub whitespace_foreground: ColourRGBA,
    /// Background colour for whitespace glyphs (optional highlight).
    pub whitespace_background: Option<ColourRGBA>,
    /// Colour for inactive indent guide lines.
    pub indent_guide: ColourRGBA,
    /// Colour for the active (highlighted) indent guide.
    pub indent_guide_highlight: ColourRGBA,
    /// Colour for single-edge line or background shading.
    pub edge_colour: ColourRGBA,
    /// Colour for wrap marker glyphs.
    pub wrap_marker: ColourRGBA,
}
```

### Query Result Types

```rust
/// The type of whitespace glyph to render at a position.
/// Addresses: Requirement 2 AC 1, AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceGlyph {
    /// Centred dot for a space character.
    SpaceDot,
    /// Arrow spanning the full tab width.
    TabArrow { width_chars: u32 },
    /// Horizontal strikeout through the tab span.
    TabStrikeout { width_chars: u32 },
}

/// A whitespace glyph at a specific column position within a line.
/// Addresses: Requirement 9 AC 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphPosition {
    /// 0-based column within the line.
    pub column: u32,
    /// The glyph to render.
    pub glyph: WhitespaceGlyph,
}

/// The set of indent guide columns for a line.
/// Addresses: Requirement 3 AC 3–5, Requirement 4 AC 1–2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentGuideInfo {
    /// Columns at which inactive guides should be drawn.
    pub guide_columns: Vec<u32>,
    /// The column of the active (highlighted) guide, if any.
    pub active_column: Option<u32>,
}

/// Information about wrap markers for a document line's sub-lines.
/// Addresses: Requirement 6 AC 1–6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapMarkerInfo {
    /// Sub-line indices that need an end marker (continuing to next sub-line).
    pub end_markers: Vec<u32>,
    /// Sub-line indices that need a start marker (continuation from previous).
    pub start_markers: Vec<u32>,
    /// Whether a margin marker should appear for this document line.
    pub margin_marker: bool,
    /// Location positioning for markers.
    pub location: WrapVisualLocation,
}

/// Continuation sub-line indentation info.
/// Addresses: Requirement 7 AC 1–6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrapIndentInfo {
    /// Mode in use.
    pub mode: WrapIndentMode,
    /// Effective indentation in character widths for continuation sub-lines.
    pub indent_chars: u32,
    /// Whether the indent was clamped at 3/4 viewport width.
    pub clamped: bool,
}
```

---

## Public API Surface

### Initialization and Lifecycle

```rust
/// Initialize the whitespace-and-guides subsystem.
/// Reads current config, resolves theme colours, registers commands.
///
/// Addresses: Requirement 8 AC 5, Requirement 9 AC 2
pub fn init(
    config: &ConfigHandle,
    theme: &ThemeHandle,
    commands: &CommandRegistry,
) -> Result<WhitespaceGuidesHandle, WhitespaceGuidesError>;

/// Handle providing thread-safe access to the settings and query API.
/// Clonable, shareable with the rendering shell.
#[derive(Clone)]
pub struct WhitespaceGuidesHandle {
    inner: Arc<RwLock<WhitespaceGuidesState>>,
    config: ConfigHandle,
    theme: ThemeHandle,
}

/// Shut down the subsystem. Deregisters config callbacks and commands.
pub fn shutdown(handle: &WhitespaceGuidesHandle);
```

### Settings Access

```rust
impl WhitespaceGuidesHandle {
    /// Get a snapshot of the current effective settings.
    /// Addresses: Requirement 9 AC 2
    pub fn settings(&self) -> WhitespaceSettings;

    /// Get the resolved colour cache.
    pub fn colours(&self) -> ResolvedColours;

    /// Set the active indent guide column (called by caret/scope tracker).
    /// Addresses: Requirement 4 AC 2, AC 5
    pub fn set_active_guide_column(&self, column: Option<u32>);

    /// Notify that wrap mode has changed (enables/disables wrap markers).
    /// Addresses: Requirement 6 AC 9
    pub fn set_wrap_active(&self, active: bool);
}
```

### Whitespace Glyph Query

```rust
impl WhitespaceGuidesHandle {
    /// Compute the whitespace glyph positions for a single line.
    /// Returns an empty vec when visibility is Invisible.
    ///
    /// Addresses: Requirement 1 AC 1–5, Requirement 2 AC 1–2,
    ///            Requirement 9 AC 4
    pub fn whitespace_glyphs(&self, line_text: &[u8]) -> Vec<GlyphPosition>;
}
```

### Indent Guide Query

```rust
impl WhitespaceGuidesHandle {
    /// Compute indent guide columns for a line given its context.
    ///
    /// `line_text` — the content of the line being rendered.
    /// `prev_indent` — indent level of the nearest preceding non-blank line
    ///                 (used by LookBoth mode; None if unavailable).
    /// `next_indent` — indent level of the nearest following non-blank line
    ///                 (used by LookForward/LookBoth; None if unavailable).
    ///
    /// Addresses: Requirement 3 AC 1–8, Requirement 4 AC 1–5
    pub fn indent_guides(
        &self,
        line_text: &[u8],
        prev_indent: Option<u32>,
        next_indent: Option<u32>,
    ) -> IndentGuideInfo;
}
```

### Edge Column Query

```rust
impl WhitespaceGuidesHandle {
    /// Get edge column configuration for the current viewport.
    /// Returns None when EdgeMode is None.
    ///
    /// Addresses: Requirement 5 AC 1–10
    pub fn edge_columns(&self) -> Option<EdgeInfo>;
}

/// Edge column information for the viewport renderer.
/// Addresses: Requirement 5 AC 3–5
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeInfo {
    /// Single vertical line at the specified column.
    Line { column: u32, colour: ColourRGBA },
    /// Background shading beyond the specified column.
    Background { column: u32, colour: ColourRGBA },
    /// Multiple vertical lines at different columns.
    MultiLine { edges: Vec<EdgeProperties> },
}
```

### Wrap Marker Query

```rust
impl WhitespaceGuidesHandle {
    /// Compute wrap marker info for a document line with the given sub-line count.
    /// Returns None when wrap is not active or flags are None.
    ///
    /// Addresses: Requirement 6 AC 1–9, Requirement 7 AC 1–6
    pub fn wrap_markers(&self, sub_line_count: u32) -> Option<WrapMarkerInfo>;

    /// Compute the continuation sub-line indentation for a document line.
    /// `first_line_indent` — the leading whitespace width (in char units) of the
    ///                       document line's first sub-line.
    /// `viewport_width_chars` — the viewport width in character units (for 3/4 clamp).
    ///
    /// Addresses: Requirement 7 AC 1–6
    pub fn wrap_indent(
        &self,
        first_line_indent: u32,
        viewport_width_chars: u32,
    ) -> WrapIndentInfo;
}
```

### Toggle Commands

```rust
impl WhitespaceGuidesHandle {
    /// Cycle whitespace visibility: Invisible → VisibleAlways → VisibleAfterIndent
    /// → VisibleOnlyInIndent → Invisible.
    /// Persists the result to the user config layer.
    ///
    /// Addresses: Requirement 8 AC 1, AC 4, AC 6
    pub fn toggle_whitespace(&self) -> WhitespaceVisibility;

    /// Cycle indent guide mode: None → Real → LookForward → LookBoth → None.
    /// Persists the result to the user config layer.
    ///
    /// Addresses: Requirement 8 AC 2, AC 4, AC 6
    pub fn toggle_indent_guides(&self) -> IndentGuideMode;

    /// Toggle edge column: None ↔ last non-None mode (default Line).
    /// Persists the result to the user config layer.
    ///
    /// Addresses: Requirement 8 AC 3, AC 4, AC 6
    pub fn toggle_edge_column(&self) -> EdgeMode;

    /// Clear all multi-edge entries, resetting to an empty list.
    /// Addresses: Requirement 5 AC 9
    pub fn clear_multi_edges(&self);
}
```

### Indent Level Computation (Internal Utility, pub(crate))

```rust
/// Compute the indent level (in columns) of a line given tab_size.
/// Stops at the first non-whitespace byte.
///
/// Used internally by IndentGuideQuery and exposed for testing.
/// Addresses: Requirement 3 AC 3
pub(crate) fn indent_level_of(line: &[u8], tab_size: u32) -> u32;

/// Scan forward from a line to find the next non-blank line's indent level.
/// Used by LookForward and LookBoth modes.
///
/// Addresses: Requirement 3 AC 4
pub(crate) fn scan_forward_indent<F>(
    start_line: u64,
    line_count: u64,
    get_line: F,
    tab_size: u32,
) -> Option<u32>
where
    F: Fn(u64) -> Vec<u8>;

/// Scan backward from a line to find the previous non-blank line's indent level.
/// Used by LookBoth mode.
///
/// Addresses: Requirement 3 AC 5
pub(crate) fn scan_backward_indent<F>(
    start_line: u64,
    get_line: F,
    tab_size: u32,
) -> Option<u32>
where
    F: Fn(u64) -> Vec<u8>;
```

---

## Error Handling

```rust
/// Errors originating from the ff-whitespace-guides crate.
/// Formatted per Error Message Standards (Req 8):
/// `[whitespace-guides] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WhitespaceGuidesError {
    /// Configuration key has an invalid value.
    #[error("[whitespace-guides] config: invalid value for key '{key}' — using default '{default}'")]
    InvalidConfigValue {
        key: String,
        default: String,
    },

    /// Command registration failed.
    #[error("[whitespace-guides] init: failed to register command '{command}': {reason}")]
    CommandRegistration {
        command: String,
        reason: String,
    },

    /// Theme colour resolution failed (fallback applied).
    #[error("[whitespace-guides] theme: element '{element}' not found — using fallback")]
    ThemeElementMissing {
        element: String,
    },

    /// Configuration system interaction failed.
    #[error("[whitespace-guides] config: {0}")]
    Config(#[from] ConfigError),
}
```

---

## Configuration Keys

All keys reside in the `editor.*` namespace per cross-cutting Requirement 5.

| Key | Type | Default | Requirement |
|-----|------|---------|-------------|
| `editor.whitespace_mode` | String enum | `"invisible"` | Req 1 AC 6 |
| `editor.tab_draw_mode` | String enum | `"long_arrow"` | Req 2 AC 4 |
| `editor.whitespace_size` | Integer (≥1) | `1` | Req 2 AC 5 |
| `editor.indent_guides` | String enum | `"none"` | Req 3 AC 7 |
| `editor.edge_mode` | String enum | `"none"` | Req 5 AC 8 |
| `editor.edge_column` | Integer (≥1) | `80` | Req 5 AC 7 |
| `editor.edge_colour` | Colour string | Theme default | Req 5 AC 7 |
| `editor.edge_columns` | Array of {column, colour} | `[]` | Req 5 AC 6 |
| `editor.wrap_visual_flags` | Integer (bitfield 0–7) | `0` | Req 6 AC 7 |
| `editor.wrap_visual_location` | String enum | `"default"` | Req 6 AC 7 |
| `editor.wrap_indent_mode` | String enum | `"fixed"` | Req 7 AC 4 |
| `editor.wrap_start_indent` | Integer (≥0) | `0` | Req 7 AC 3 |

### Enum String Mappings

| Enum | Values |
|------|--------|
| `WhitespaceVisibility` | `"invisible"`, `"visible_always"`, `"visible_after_indent"`, `"visible_only_in_indent"` |
| `TabDrawMode` | `"long_arrow"`, `"strikeout"` |
| `IndentGuideMode` | `"none"`, `"real"`, `"look_forward"`, `"look_both"` |
| `EdgeMode` | `"none"`, `"line"`, `"background"`, `"multi_line"` |
| `WrapVisualLocation` | `"default"`, `"end_by_text"`, `"start_by_text"` |
| `WrapIndentMode` | `"fixed"`, `"same"`, `"indent"`, `"deep_indent"` |

---

## Integration Points

### Theme Integration (`ff-theme`)

The crate resolves its visual colours from the active theme palette:

| Visual Element | Theme Source | Fallback |
|----------------|-------------|----------|
| Whitespace foreground | `Element::WhitespaceForeground` | `EditorPalette::foreground` |
| Whitespace background | `Element::WhitespaceBackground` | None (transparent) |
| Indent guide line | `StyleSlot[INDENT_GUIDE_STYLE_INDEX].foreground` | `EditorPalette::muted` |
| Active indent guide | Dedicated highlight colour from theme | `EditorPalette::accent` |
| Edge line/background | `editor.edge_colour` config or `ChromePalette::cursor_column_indicator` | `EditorPalette::muted` |
| Wrap marker glyph | Wrap marker colour from theme | `EditorPalette::muted` |

On `ThemeEvent::PaletteChanged` or `ThemeEvent::ModeChanged`, the `ResolvedColours` cache is rebuilt.

### Document Model Integration (`ff-document-model`)

- **Line content**: `Document::get_range(line_start, line_end - line_start)` to read a line's bytes for whitespace and indent analysis
- **Tab size**: Read from `editor.tab_size` config key (shared with document model)
- **Indent size**: Read from `editor.indent_size` config key

### Display Line Mapping Integration (`ff-display-line-mapping`)

- **Sub-line count**: Query `ContractionState::display_lines_for_doc_line(line)` to determine how many sub-lines a document line occupies (for wrap marker computation)
- **Wrap state**: The `line-wrap-toggle` crate notifies this subsystem via `set_wrap_active()` when word wrap is toggled

### Configuration System Integration (`ff-config`)

- Registers hot-reload callbacks for all `editor.whitespace_*`, `editor.indent_*`, `editor.edge_*`, and `editor.wrap_*` keys
- On callback, rebuilds `WhitespaceSettings` and emits a repaint notification
- Persists toggle command results to the user config layer via `config.set(key, value, ConfigLayer::User)`

### Command Framework Integration (`ff-command`)

Three commands are registered at `init()`:

| Command ID | Display Name | Default Shortcut | Category |
|------------|-------------|-----------------|----------|
| `toggle_whitespace` | Toggle Whitespace Visibility | (none — user-assignable) | View |
| `toggle_indent_guides` | Toggle Indent Guides | (none — user-assignable) | View |
| `toggle_edge_column` | Toggle Edge Column | (none — user-assignable) | View |

Commands are also accessible via the menu system under View → Visual Aids.

---

## Correctness Properties

These properties define invariants that property-based tests must verify.

### Property 1: Whitespace Glyph Coverage

**Statement:** For any line text and any non-Invisible visibility mode, `whitespace_glyphs()` returns exactly one `GlyphPosition` for each whitespace character that is eligible under the mode — no more, no fewer.

**Relates to:** Requirement 1 AC 3–5, Requirement 9 AC 4

**Strategy:** Generate random byte strings containing spaces, tabs, and printable ASCII. For each mode, compute which characters are eligible (VisibleAlways = all whitespace; VisibleAfterIndent = whitespace after first non-WS; VisibleOnlyInIndent = whitespace before first non-WS). Assert `glyphs.len() == eligible_count` and each glyph column matches the character's column.

### Property 2: Indent Guide Columns Are Tab-Stop Aligned

**Statement:** For any IndentGuideMode other than None, every column in `IndentGuideInfo::guide_columns` is a multiple of `tab_size` (i.e., `column % tab_size == 0`).

**Relates to:** Requirement 3 AC 3

**Strategy:** Generate random lines with leading whitespace of varying lengths and random tab_size (2, 4, 8). Assert all returned guide columns are exact multiples of tab_size.

### Property 3: LookBoth Dominates LookForward

**Statement:** For any line and surrounding context, the set of guide columns produced by `LookBoth` mode is a superset of (or equal to) the set produced by `LookForward` mode.

**Relates to:** Requirement 3 AC 4–5

**Strategy:** Generate sequences of lines with varying indent levels and blank lines. Compute guides under both modes. Assert `LookBoth_guides ⊇ LookForward_guides`.

### Property 4: Edge Column Mode Mutual Exclusion

**Statement:** `edge_columns()` returns exactly one `EdgeInfo` variant matching the current `EdgeMode`, or `None` when mode is `None`. It never returns `Line` when mode is `MultiLine`, etc.

**Relates to:** Requirement 5 AC 1–5

**Strategy:** For random `EdgeMode` settings, assert the returned `EdgeInfo` variant matches the mode enum.

### Property 5: Wrap Markers Suppressed When Wrap Inactive

**Statement:** When `wrap_active` is false, `wrap_markers()` always returns `None`, regardless of `WrapVisualFlag` settings.

**Relates to:** Requirement 6 AC 9

**Strategy:** Generate random WrapVisualFlag values with wrap_active=false. Assert result is always None.

### Property 6: Wrap Indent Clamp at 3/4 Viewport

**Statement:** The effective `indent_chars` in `WrapIndentInfo` never exceeds `3 * viewport_width_chars / 4`.

**Relates to:** Requirement 7 AC 6

**Strategy:** Generate random first_line_indent (0..1000), viewport_width_chars (20..200), and WrapIndentMode values. Assert `result.indent_chars <= 3 * viewport_width_chars / 4`.

### Property 7: Toggle Cycle Returns to Start

**Statement:** Calling `toggle_whitespace()` exactly 4 times returns the mode to its original value. Similarly, `toggle_indent_guides()` × 4 and `toggle_edge_column()` × 2 restore the original mode.

**Relates to:** Requirement 8 AC 1–3

**Strategy:** Start from each possible mode value, apply the toggle the required number of times, assert equality with the starting value.

### Property 8: Invisible Mode Produces Empty Glyphs

**Statement:** When `WhitespaceVisibility` is `Invisible`, `whitespace_glyphs()` returns an empty vector for any input line.

**Relates to:** Requirement 1 AC 1 (Invisible definition)

**Strategy:** Generate arbitrary line content. Assert `whitespace_glyphs(line).is_empty()` when visibility is Invisible.

### Property 9: Settings Reflect Config Keys

**Statement:** For any valid combination of config key values, `WhitespaceSettings::from_config()` produces a settings struct where each field matches the corresponding config value (after enum parsing).

**Relates to:** Requirement 9 AC 2

**Strategy:** Construct a mock config with random valid enum string values and integers. Build settings. Assert each field matches the input.

---

## Testing Strategy

### Unit Tests

- **Whitespace query**: Verify glyph positions for each visibility mode with known inputs (tabs, spaces, mixed lines, empty lines, trailing whitespace).
- **Indent guide computation**: Verify guide columns for Real/LookForward/LookBoth with controlled line sequences.
- **Edge query**: Verify correct EdgeInfo variant and colour for each EdgeMode.
- **Wrap markers**: Verify marker sub-line indices for various sub-line counts and flag combinations.
- **Settings construction**: Verify from_config with explicit mock values.
- **Toggle commands**: Verify cycling behaviour and persistence.

### Property-Based Tests (proptest)

All nine correctness properties above are implemented as proptest tests in `tests/property_tests.rs`. Each test runs a minimum of 256 cases.

### Integration Tests

End-to-end test with mock `ConfigHandle` and `ThemeHandle`:
1. Initialize the subsystem
2. Verify initial settings match defaults
3. Simulate config hot-reload → verify settings update
4. Simulate theme change → verify colour cache update
5. Exercise toggle commands → verify cycling and persistence

---

## Performance Considerations

- **No allocations in hot path**: `whitespace_glyphs()` and `indent_guides()` accept a pre-allocated output buffer option to avoid per-line allocations during rendering. The `Vec` return API allocates but is suitable for non-critical paths.
- **Settings snapshot**: The `WhitespaceSettings` struct is small (< 128 bytes) and cheap to clone. The renderer holds a local snapshot to avoid lock contention during frame rendering.
- **Colour cache**: `ResolvedColours` is rebuilt only on theme change events, not per-frame.
- **Indent scanning**: `scan_forward_indent` and `scan_backward_indent` are bounded — they stop after scanning a configurable maximum number of blank lines (default 2000) to avoid O(n) pathological cases in files with large blank regions.
