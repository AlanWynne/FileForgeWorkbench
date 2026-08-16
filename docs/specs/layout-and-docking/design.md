# Design Document: Layout and Docking System (`ff-layout`)

## 1. Overview

The `ff-layout` crate is the **GUI-independent layout engine** for the FileForgeWorkbench platform. It owns the spatial arrangement of all panels, tab groups, floating windows, and dock zones — expressing the entire workspace layout as a data model that the GUI shell renders but does not own.

### Purpose

- Manage dockable panels within designated dock zones (left, right, bottom, center)
- Support tab groups with horizontal and vertical split views in the center editor area
- Enable floating OS-level windows for multi-monitor workflows
- Provide named layout personas for instant workspace switching
- Handle drag-and-drop rearrangement with visual drop indicators
- Serialize and restore layout state across sessions
- Enforce resizing constraints with proportional splitters

### Position in Architecture

```
Wave 2 — Platform Architecture

┌─────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)               │
│         Renders layout model; does NOT own it             │
├─────────────────────────────────────────────────────────┤
│  ff-layout (THIS CRATE) │ ff-core │ ff-command │ ff-plugin│
│  Layout engine, panels, dock zones, personas             │
├─────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                 │
└─────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: The layout engine owns the model; `egui` is used only for the `DockablePanel::render` trait method signature
- **Plugin Architecture (Req 3)**: Panels are contributed by plugins via `PluginContext`
- **Command-Driven (Req 4)**: All layout operations are invokable as commands
- **Async I/O (Req 6)**: Serialization I/O runs on Tokio workers; layout mutations are synchronous on the main thread
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-layout`
- **Error Message Standards (Req 8)**: Errors follow `[layout] operation: description` format

---

## 2. Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Shell Layer
        DESKTOP[ff-desktop<br/>GUI Shell / Renderer]
    end

    subgraph ff-layout
        LE[LayoutEngine<br/>central coordinator]
        PR[PanelRegistry<br/>panel types + defaults]
        LS[LayoutState<br/>serializable snapshot]
        TG[TabGroupManager<br/>splits + tabs]
        FW[FloatingWindowManager<br/>OS viewports]
        PM[PersonaManager<br/>named presets]
        DD[DragDropCoordinator<br/>indicators + gestures]
        SP[SplitterManager<br/>resize logic]
        SER[Serializer<br/>TOML persistence]
    end

    subgraph Peers
        CMD[ff-command<br/>shortcut registry]
        CORE[ff-core<br/>event bus, lifecycle]
        PLUGIN[ff-plugin<br/>panel contribution]
        LOG[ff-logging<br/>diagnostics]
    end

    DESKTOP -->|render requests| LE
    DESKTOP -->|user input| DD
    LE --> PR
    LE --> LS
    LE --> TG
    LE --> FW
    LE --> PM
    LE --> DD
    LE --> SP
    LE --> SER
    PLUGIN -->|register panels| PR
    CMD -->|layout commands| LE
    LE -->|state-change events| CORE
    LE --> LOG
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **LayoutEngine** | Central coordinator — owns the layout tree, orchestrates all transitions |
| **PanelRegistry** | Tracks registered panel types and their default zone assignments |
| **LayoutState** | Serializable snapshot of the complete layout for persistence |
| **TabGroupManager** | Manages center-area splits, tab ordering, group lifecycle |
| **FloatingWindowManager** | Tracks floating OS windows, position/size, monitor assignment |
| **PersonaManager** | Named presets — load, save, activate, track modifications |
| **DragDropCoordinator** | Hit testing, drop indicators, gesture detection |
| **SplitterManager** | Proportional resizing, minimum constraints, double-click reset |
| **Serializer** | TOML read/write, schema versioning, graceful degradation |

---

## 3. Module Structure

```
crates/ff-layout/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── engine.rs               # LayoutEngine struct, top-level orchestration
│   ├── panel/
│   │   ├── mod.rs              # Panel re-exports
│   │   ├── traits.rs           # DockablePanel trait, DockState enum
│   │   ├── registry.rs         # PanelRegistry — registration, lookup, validation
│   │   └── display_state.rs    # PanelDisplayState enum (minimized, normal, maximized)
│   ├── dock/
│   │   ├── mod.rs              # Dock zone re-exports
│   │   ├── zone.rs             # DockZone enum, zone content management
│   │   └── layout_tree.rs      # Hierarchical layout tree (zones + splitters)
│   ├── tabs/
│   │   ├── mod.rs              # Tab group re-exports
│   │   ├── group.rs            # TabGroup struct, tab ordering
│   │   ├── split.rs            # SplitDirection, split/merge operations
│   │   └── manager.rs          # TabGroupManager — split tree coordination
│   ├── floating/
│   │   ├── mod.rs              # Floating window re-exports
│   │   ├── window.rs           # FloatingWindow struct, lifecycle
│   │   ├── manager.rs          # FloatingWindowManager — creation, limit enforcement
│   │   └── monitor.rs          # Monitor detection, DPI, repositioning logic
│   ├── persona/
│   │   ├── mod.rs              # Persona re-exports
│   │   ├── definition.rs       # Persona struct, built-in definitions
│   │   ├── manager.rs          # PersonaManager — load, save, activate, track
│   │   └── storage.rs          # TOML file I/O for persona files
│   ├── drag/
│   │   ├── mod.rs              # Drag-and-drop re-exports
│   │   ├── coordinator.rs      # DragDropCoordinator — gesture state machine
│   │   ├── indicator.rs        # DropIndicator rendering model
│   │   └── hit_test.rs         # Zone/group hit testing, insertion index calc
│   ├── resize/
│   │   ├── mod.rs              # Resize re-exports
│   │   ├── splitter.rs         # Splitter struct, position tracking
│   │   └── manager.rs          # SplitterManager — constraint enforcement
│   ├── state/
│   │   ├── mod.rs              # State re-exports
│   │   ├── layout_state.rs     # LayoutState struct, in-memory representation
│   │   └── serializer.rs       # TOML serialization, schema version, migration
│   ├── commands.rs             # Layout command registrations (dock, undock, split, persona)
│   └── error.rs                # LayoutError enum
└── tests/
    ├── panel_registry_tests.rs     # Registration property tests
    ├── tab_group_tests.rs          # Split/merge property tests
    ├── floating_window_tests.rs    # Window lifecycle property tests
    ├── persona_tests.rs            # Persona activation property tests
    ├── serialization_tests.rs      # Round-trip property tests
    ├── splitter_tests.rs           # Resize constraint property tests
    ├── drag_drop_tests.rs          # Hit testing property tests
    └── integration.rs              # End-to-end layout scenario tests
```

---

## 4. Key Data Models and Types

### LayoutEngine

```rust
/// The central coordinator for the layout system. Owns the layout tree,
/// manages all transitions between layout states, and serves as the primary
/// API surface for the shell and command framework.
///
/// Addresses: Requirement 1 criterion 1, Requirement 1 criterion 9
pub struct LayoutEngine {
    /// The current in-memory layout state
    state: LayoutState,
    /// Registry of all known panel types
    panel_registry: PanelRegistry,
    /// Tab group management for center area splits
    tab_groups: TabGroupManager,
    /// Floating window tracking
    floating_windows: FloatingWindowManager,
    /// Persona management (presets)
    personas: PersonaManager,
    /// Drag-and-drop coordination
    drag_drop: DragDropCoordinator,
    /// Splitter/resize management
    splitters: SplitterManager,
    /// Whether the current layout diverges from the active persona
    persona_modified: bool,
    /// The currently active persona name (if any)
    active_persona: Option<String>,
}
```

### DockZone

```rust
/// Designated areas within the primary window where panels can be attached.
/// Addresses: Requirement 1 criteria 1/3/5
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum DockZone {
    Left,
    Right,
    Bottom,
    Center,
    Floating,
}
```

### DockablePanel Trait

```rust
/// Trait that all dockable panels must implement. The Layout_Engine interacts
/// with panels exclusively through this interface.
///
/// Addresses: Requirement 1 criteria 4–9
pub trait DockablePanel: Send + Sync {
    /// Returns the unique panel identifier (1–64 ASCII alphanumeric/underscore chars).
    /// Addresses: Requirement 1 criterion 4
    fn panel_id(&self) -> &str;

    /// Returns the preferred default dock zone.
    /// Addresses: Requirement 1 criterion 5
    fn default_dock_zone(&self) -> DockZone;

    /// Renders panel content into the given UI region.
    /// Must produce valid output regardless of dock state.
    /// Addresses: Requirement 1 criterion 6
    fn render(&mut self, ui: &mut egui::Ui);

    /// Returns the display title (1–128 characters).
    /// Addresses: Requirement 1 criterion 7
    fn title(&self) -> &str;

    /// Called when the panel transitions between dock states.
    /// Addresses: Requirement 1 criterion 8
    fn on_dock_state_changed(&mut self, state: DockState);

    /// Returns the minimum size constraint in logical pixels (width, height).
    /// Returns None to use the default minimum of 48×48.
    /// Addresses: Requirement 8 criteria 3/4
    fn minimum_size(&self) -> Option<(f32, f32)> {
        None
    }
}
```

### DockState

```rust
/// The current state of a panel within the layout system.
/// Addresses: Requirement 1 criteria 8/11/13
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockState {
    /// Panel is attached to a dock zone and visible at normal size
    Docked,
    /// Panel is in a floating OS window
    Floating,
    /// Panel is collapsed to a tab/icon in the zone header
    Minimized,
    /// Panel is hidden from view (position preserved in state)
    Hidden,
    /// Panel is expanded to fill the entire primary window content area
    Maximized,
}
```

### PanelDisplayState

```rust
/// Display states for panels within dock zones.
/// Addresses: Requirement 1 criterion 13
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PanelDisplayState {
    /// Collapsed to tab/icon in dock zone header
    Minimized,
    /// Rendered at assigned size
    Normal,
    /// Expanded to fill entire primary window content area
    Maximized,
}
```

### FloatingWindow

```rust
/// Represents an OS-level window containing one or more detached panels/tabs.
/// Addresses: Requirement 3 criteria 1–16, Requirement 4 criteria 1–8
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FloatingWindow {
    /// Unique identifier for this floating window
    pub id: FloatingWindowId,
    /// Panels contained in this floating window
    pub panels: Vec<String>,
    /// Position in logical pixels (screen coordinates)
    pub position: Position,
    /// Size in logical pixels (minimum 200×150)
    pub size: Size,
    /// Monitor identifier for multi-monitor persistence
    pub monitor_id: Option<String>,
    /// The dock zone the panel(s) originated from (for redock)
    pub origin_zone: DockZone,
    /// Original tab index within the origin group (for tab redock)
    pub origin_tab_index: Option<usize>,
}

/// Opaque identifier for a floating window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FloatingWindowId(u32);

/// Logical pixel position (x, y).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Logical pixel size (width, height).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}
```

### PanelRegistry

```rust
/// Registry of all known panel types and their default assignments.
/// Plugins register panels here during initialization.
///
/// Addresses: Requirement 1 criteria 2/3/9/10/14
pub struct PanelRegistry {
    /// Map of panel_id → PanelRegistration
    panels: HashMap<String, PanelRegistration>,
}

/// Information stored for each registered panel type.
#[derive(Debug, Clone)]
pub struct PanelRegistration {
    /// The unique panel identifier
    pub panel_id: String,
    /// Default dock zone assignment
    pub default_zone: DockZone,
    /// Display title
    pub title: String,
    /// The panel instance (trait object)
    pub panel: Arc<Mutex<dyn DockablePanel>>,
}
```

### LayoutState

```rust
/// A serializable snapshot of the complete layout. Persisted at exit,
/// restored at startup, used for persona definitions.
///
/// Addresses: Requirement 6 criteria 1–11
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LayoutState {
    /// Schema version for forward-compatible migration
    /// Addresses: Requirement 6 criterion 11
    pub schema_version: u32,
    /// Dock zone contents: panel_id → zone assignment with dimensions
    pub docked_panels: Vec<DockedPanelState>,
    /// Tab group arrangement in the center area
    pub tab_groups: TabGroupTree,
    /// Floating window positions and contents
    pub floating_windows: Vec<FloatingWindow>,
    /// Splitter positions as proportional values [0.0, 1.0]
    /// Addresses: Requirement 8 criterion 7
    pub splitter_positions: HashMap<SplitterId, f32>,
    /// Panel visibility map (hidden panels tracked here)
    pub panel_visibility: HashMap<String, bool>,
    /// Panel display states (minimized/normal/maximized)
    pub panel_display_states: HashMap<String, PanelDisplayState>,
}

/// State for a single docked panel within the layout.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DockedPanelState {
    pub panel_id: String,
    pub zone: DockZone,
    /// Zone width or height in logical pixels
    pub zone_dimension: f32,
}
```

### TabGroup

```rust
/// A subdivision of the center editor area holding one or more tabs.
/// Multiple TabGroups coexist via horizontal or vertical splits.
///
/// Addresses: Requirement 2 criteria 1–9
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TabGroup {
    /// Unique identifier for this tab group
    pub id: TabGroupId,
    /// Ordered list of tab identifiers within this group
    pub tabs: Vec<String>,
    /// Index of the currently active tab (0-based)
    pub active_tab: usize,
}

/// Opaque identifier for a tab group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TabGroupId(u32);

/// Hierarchical tree representing tab group splits.
/// Addresses: Requirement 2 criteria 1/8
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TabGroupTree {
    /// A leaf node containing a single tab group
    Leaf(TabGroup),
    /// A split node containing two children with a split direction and proportion
    Split {
        direction: SplitDirection,
        /// Proportion allocated to the first child [0.0, 1.0]
        proportion: f32,
        first: Box<TabGroupTree>,
        second: Box<TabGroupTree>,
    },
}

/// Direction of a tab group split.
/// Addresses: Requirement 2 criteria 2/3
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SplitDirection {
    /// Side-by-side (left/right)
    Horizontal,
    /// Stacked (top/bottom)
    Vertical,
}
```

### Persona

```rust
/// A named layout configuration that can be activated to switch
/// the entire workspace appearance with a single action.
///
/// Addresses: Requirement 5 criteria 1–10
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Persona {
    /// Unique name for this persona (e.g., "Editor Focus", "Debug")
    pub name: String,
    /// Whether this is a built-in persona (cannot be deleted)
    pub built_in: bool,
    /// The layout state defining this persona's arrangement
    pub layout: LayoutState,
    /// Optional description for UI display
    pub description: Option<String>,
}

/// Identifies whether a persona is built-in or user-created.
/// Addresses: Requirement 5 criterion 6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonaKind {
    BuiltIn,
    Custom,
}
```

### DropIndicator

```rust
/// Visual overlay shown during drag-and-drop to highlight valid targets.
///
/// Addresses: Requirement 7 criteria 1/4/5/6, Requirement 10 criterion 6
#[derive(Debug, Clone)]
pub struct DropIndicator {
    /// The target area in logical screen coordinates
    pub bounds: Rect,
    /// Where the panel/tab will be placed upon release
    pub placement: DropPlacement,
    /// Whether the indicator is currently visible
    pub visible: bool,
}

/// Describes where a dropped item will be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPlacement {
    /// Dock into specified zone
    DockZone(DockZone),
    /// Insert as tab at given index in a tab group
    TabInsertion { group_id: TabGroupId, index: usize },
    /// Split the target group in the specified direction
    SplitGroup { group_id: TabGroupId, direction: SplitDirection, side: SplitSide },
}

/// Which side of a split the dropped item goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitSide {
    First,
    Second,
}

/// A rectangle in logical pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

### Splitter

```rust
/// A draggable border between adjacent dock zones or tab groups.
///
/// Addresses: Requirement 8 criteria 1–9
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Splitter {
    /// Unique identifier for this splitter
    pub id: SplitterId,
    /// Current proportional position [0.0, 1.0]
    pub proportion: f32,
    /// Default proportional position (for double-click reset)
    /// Addresses: Requirement 8 criterion 8
    pub default_proportion: f32,
    /// Orientation of the splitter
    pub orientation: SplitterOrientation,
    /// Minimum size constraints from adjacent panels (logical pixels)
    pub min_first: f32,
    pub min_second: f32,
}

/// Opaque identifier for a splitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SplitterId(u32);

/// Orientation of a splitter handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitterOrientation {
    /// Horizontal splitter (divides top/bottom)
    Horizontal,
    /// Vertical splitter (divides left/right)
    Vertical,
}
```

---

## 5. Public API Surface

### LayoutEngine — Construction and Lifecycle

```rust
impl LayoutEngine {
    /// Create a new LayoutEngine with default layout (dock zones: left, right, bottom, center).
    /// Addresses: Requirement 1 criterion 1
    pub fn new() -> Self;

    /// Initialize from a persisted LayoutState (startup restoration).
    /// Applies graceful degradation for missing panels.
    /// Addresses: Requirement 6 criteria 2/3/5
    pub fn from_state(state: LayoutState, registry: &PanelRegistry) -> Self;

    /// Returns the current LayoutState as a serializable snapshot.
    pub fn current_state(&self) -> &LayoutState;

    /// Returns whether the layout has been modified from the active persona.
    /// Addresses: Requirement 5 criterion 10
    pub fn is_persona_modified(&self) -> bool;

    /// Returns the active persona name, if any.
    /// Addresses: Requirement 5 criterion 9
    pub fn active_persona_name(&self) -> Option<&str>;
}
```

### Panel Operations

```rust
impl LayoutEngine {
    /// Show a hidden panel in its last known dock zone.
    /// Addresses: Requirement 1 criterion 11
    pub fn show_panel(&mut self, panel_id: &str) -> Result<(), LayoutError>;

    /// Hide a panel while preserving its position in the LayoutState.
    /// Addresses: Requirement 1 criterion 11
    pub fn hide_panel(&mut self, panel_id: &str) -> Result<(), LayoutError>;

    /// Toggle panel visibility (show if hidden, hide if visible).
    /// Addresses: Requirement 1 criterion 12
    pub fn toggle_panel(&mut self, panel_id: &str) -> Result<(), LayoutError>;

    /// Minimize a panel (collapse to tab/icon in zone header).
    /// Addresses: Requirement 1 criterion 13
    pub fn minimize_panel(&mut self, panel_id: &str) -> Result<(), LayoutError>;

    /// Maximize a panel (expand to fill primary window content area).
    /// Addresses: Requirement 1 criterion 13
    pub fn maximize_panel(&mut self, panel_id: &str) -> Result<(), LayoutError>;

    /// Restore a panel to normal display state.
    /// Addresses: Requirement 1 criterion 13
    pub fn restore_panel(&mut self, panel_id: &str) -> Result<(), LayoutError>;
}
```

### Floating Window Operations

```rust
impl LayoutEngine {
    /// Undock a panel from its dock zone into a new floating window.
    /// Addresses: Requirement 3 criteria 1/2/4
    pub fn undock_panel(&mut self, panel_id: &str) -> Result<FloatingWindowId, LayoutError>;

    /// Undock a panel to a specific position (drag-to-float).
    /// Addresses: Requirement 3 criterion 9, Requirement 7 criterion 9
    pub fn undock_panel_at(
        &mut self,
        panel_id: &str,
        position: Position,
    ) -> Result<FloatingWindowId, LayoutError>;

    /// Redock a floating panel back to its most recent dock zone.
    /// Addresses: Requirement 3 criteria 5/6/7
    pub fn redock_panel(&mut self, window_id: FloatingWindowId) -> Result<(), LayoutError>;

    /// Undock a tab from a TabGroup into a new floating window.
    /// Addresses: Requirement 3 criterion 9, Requirement 9 criterion 3
    pub fn undock_tab(
        &mut self,
        group_id: TabGroupId,
        tab_index: usize,
    ) -> Result<FloatingWindowId, LayoutError>;

    /// Undock a tab to a specific position.
    pub fn undock_tab_at(
        &mut self,
        group_id: TabGroupId,
        tab_index: usize,
        position: Position,
    ) -> Result<FloatingWindowId, LayoutError>;

    /// Redock a tab from a floating window back to its originating TabGroup.
    /// Addresses: Requirement 3 criterion 11
    pub fn redock_tab(&mut self, window_id: FloatingWindowId) -> Result<(), LayoutError>;

    /// Update a floating window's position and size after a move/resize.
    /// Addresses: Requirement 3 criterion 4, Requirement 6 criterion 9
    pub fn update_floating_window(
        &mut self,
        window_id: FloatingWindowId,
        position: Position,
        size: Size,
    ) -> Result<(), LayoutError>;

    /// Handle OS window close button — redock rather than destroy.
    /// Addresses: Requirement 3 criteria 8/11/12
    pub fn on_floating_window_close(
        &mut self,
        window_id: FloatingWindowId,
    ) -> Result<CloseAction, LayoutError>;

    /// Returns the count of currently floating windows.
    pub fn floating_window_count(&self) -> usize;

    /// Maximum number of simultaneous floating windows.
    /// Addresses: Requirement 3 criterion 14
    pub const MAX_FLOATING_WINDOWS: usize = 16;
}

/// Result of handling a floating window close.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseAction {
    /// Panel was redocked successfully
    Redocked,
    /// Unsaved changes — show save confirmation dialog
    NeedsSaveConfirmation { tab_id: String },
}
```

### Tab Group Operations

```rust
impl LayoutEngine {
    /// Split the active tab group horizontally (side-by-side).
    /// Moves the active tab to the new group.
    /// Addresses: Requirement 2 criterion 2
    pub fn split_horizontal(&mut self) -> Result<TabGroupId, LayoutError>;

    /// Split the active tab group vertically (stacked).
    /// Moves the active tab to the new group.
    /// Addresses: Requirement 2 criterion 3
    pub fn split_vertical(&mut self) -> Result<TabGroupId, LayoutError>;

    /// Move a tab from one group to another at the specified index.
    /// Closes empty groups automatically.
    /// Addresses: Requirement 2 criteria 4/5
    pub fn move_tab(
        &mut self,
        source_group: TabGroupId,
        tab_index: usize,
        target_group: TabGroupId,
        insert_index: usize,
    ) -> Result<(), LayoutError>;

    /// Add a new tab to the active tab group (or specified group).
    /// Addresses: Requirement 2 criterion 9
    pub fn add_tab(
        &mut self,
        tab_id: &str,
        target_group: Option<TabGroupId>,
    ) -> Result<(), LayoutError>;

    /// Returns the currently active tab group ID.
    pub fn active_tab_group(&self) -> TabGroupId;

    /// Set the active tab group.
    pub fn set_active_tab_group(&mut self, group_id: TabGroupId) -> Result<(), LayoutError>;
}
```

### Persona Operations

```rust
impl LayoutEngine {
    /// Activate a persona by name, transitioning the layout.
    /// Open documents are preserved (excess tabs placed in last group).
    /// Addresses: Requirement 5 criteria 4/5
    pub fn activate_persona(&mut self, name: &str) -> Result<(), LayoutError>;

    /// Save the current layout as a custom persona.
    /// Addresses: Requirement 5 criterion 3
    pub fn save_persona(&mut self, name: &str) -> Result<(), LayoutError>;

    /// Delete a custom persona. Returns error for built-in personas.
    /// Addresses: Requirement 5 criterion 6
    pub fn delete_persona(&mut self, name: &str) -> Result<(), LayoutError>;

    /// Update the active persona to match the current layout.
    /// Addresses: Requirement 5 criterion 10
    pub fn update_active_persona(&mut self) -> Result<(), LayoutError>;

    /// Revert the layout to the active persona's saved state.
    /// Addresses: Requirement 5 criterion 10
    pub fn revert_to_persona(&mut self) -> Result<(), LayoutError>;

    /// List all available personas (built-in and custom).
    pub fn list_personas(&self) -> Vec<&Persona>;
}
```

### Serialization Operations

```rust
impl LayoutEngine {
    /// Serialize the current layout state to the session file.
    /// Addresses: Requirement 6 criterion 1
    pub fn save_session(&self, path: &Path) -> Result<(), LayoutError>;

    /// Export the current layout state to a user-specified path.
    /// Addresses: Requirement 6 criterion 6
    pub fn export_layout(&self, path: &Path) -> Result<(), LayoutError>;

    /// Import and apply a layout from a file.
    /// Missing panels are skipped gracefully.
    /// Addresses: Requirement 6 criterion 7
    pub fn import_layout(&mut self, path: &Path) -> Result<(), LayoutError>;

    /// Reset to the built-in default layout.
    /// Addresses: Requirement 6 criterion 8
    pub fn reset_to_default(&mut self);
}
```

### Drag-and-Drop Operations

```rust
impl LayoutEngine {
    /// Begin a drag operation from a panel header or tab.
    /// Addresses: Requirement 7 criterion 11
    pub fn begin_drag(&mut self, item: DragItem, origin: Position);

    /// Update the drag position — triggers hit testing and indicator display.
    /// Addresses: Requirement 7 criteria 1/5/6/13
    pub fn update_drag(&mut self, cursor: Position);

    /// End a drag operation — executes the drop or cancels.
    /// Addresses: Requirement 7 criteria 2/3/7/8/9/10/12
    pub fn end_drag(&mut self, cursor: Position) -> Result<DragResult, LayoutError>;

    /// Cancel an in-progress drag operation.
    pub fn cancel_drag(&mut self);

    /// Returns the current drop indicator (for rendering by the shell).
    pub fn current_drop_indicator(&self) -> Option<&DropIndicator>;

    /// Returns whether a drag is currently in progress.
    pub fn is_dragging(&self) -> bool;
}

/// Items that can be dragged.
#[derive(Debug, Clone)]
pub enum DragItem {
    /// A docked panel being dragged from its header
    Panel { panel_id: String },
    /// A tab being dragged from a tab group
    Tab { group_id: TabGroupId, tab_index: usize },
    /// A floating window being dragged by its title bar
    FloatingWindow { window_id: FloatingWindowId },
}

/// Result of a completed drag operation.
#[derive(Debug, Clone)]
pub enum DragResult {
    /// Item was docked into a zone
    Docked { panel_id: String, zone: DockZone },
    /// Tab was moved to a different group
    TabMoved { tab_id: String, target_group: TabGroupId, index: usize },
    /// Item was floated at the release position
    Floated { window_id: FloatingWindowId },
    /// Drag was cancelled (released in invalid location)
    Cancelled,
}
```

### Splitter Operations

```rust
impl LayoutEngine {
    /// Begin dragging a splitter.
    /// Addresses: Requirement 8 criterion 9
    pub fn begin_splitter_drag(&mut self, splitter_id: SplitterId);

    /// Update splitter position during drag (real-time resize).
    /// Enforces minimum size constraints.
    /// Addresses: Requirement 8 criteria 3/4/5/6/9
    pub fn update_splitter(
        &mut self,
        splitter_id: SplitterId,
        new_proportion: f32,
    ) -> Result<(), LayoutError>;

    /// End splitter drag — finalizes the position.
    pub fn end_splitter_drag(&mut self, splitter_id: SplitterId);

    /// Reset a splitter to its default position (double-click).
    /// Addresses: Requirement 8 criterion 8
    pub fn reset_splitter(&mut self, splitter_id: SplitterId) -> Result<(), LayoutError>;

    /// Handle primary window resize — proportional redistribution.
    /// Addresses: Requirement 8 criteria 5/6
    pub fn on_window_resize(&mut self, new_size: Size);
}
```

### PanelRegistry API

```rust
impl PanelRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self;

    /// Register a panel type. Validates panel_id format and default zone.
    /// Returns error on duplicate ID or invalid zone.
    /// Addresses: Requirement 1 criteria 2/3/10
    pub fn register(
        &mut self,
        panel: Arc<Mutex<dyn DockablePanel>>,
    ) -> Result<(), LayoutError>;

    /// Deregister a panel (plugin unload).
    /// Addresses: Requirement 1 criterion 14
    pub fn deregister(&mut self, panel_id: &str) -> bool;

    /// Look up a panel by ID.
    pub fn get(&self, panel_id: &str) -> Option<&PanelRegistration>;

    /// Returns all registered panel IDs.
    pub fn list_all(&self) -> Vec<&str>;

    /// Returns whether a panel_id is currently registered.
    pub fn is_registered(&self, panel_id: &str) -> bool;
}
```

### Multi-Monitor Support

```rust
impl LayoutEngine {
    /// Handle monitor disconnection — relocate affected windows.
    /// Addresses: Requirement 4 criterion 6
    pub fn on_monitor_disconnected(&mut self, monitor_id: &str);

    /// Update a floating window's monitor assignment after a move.
    /// Addresses: Requirement 4 criterion 2
    pub fn update_window_monitor(
        &mut self,
        window_id: FloatingWindowId,
        monitor_id: &str,
    ) -> Result<(), LayoutError>;

    /// Validate window positions during startup restoration.
    /// Repositions windows with less than 50% visibility.
    /// Addresses: Requirement 4 criteria 7/8
    pub fn validate_window_positions(&mut self, available_monitors: &[MonitorInfo]);
}

/// Information about a connected monitor for positioning decisions.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Unique identifier for this monitor
    pub id: String,
    /// Whether this is the primary monitor
    pub is_primary: bool,
    /// Work area bounds (excluding taskbar etc.)
    pub work_area: Rect,
    /// DPI scale factor for this monitor
    pub dpi_scale: f32,
}
```

---

## 6. Error Types

```rust
/// Errors produced by the layout engine.
/// Formatted per Error Message Standards: `[layout] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LayoutError {
    /// Panel ID is not registered in the PanelRegistry
    #[error("[layout] panel: '{panel_id}' is not registered")]
    PanelNotFound { panel_id: String },

    /// Attempted to register a duplicate panel_id
    /// Addresses: Requirement 1 criterion 10
    #[error("[layout] register: panel '{panel_id}' is already registered")]
    DuplicatePanelId { panel_id: String },

    /// Invalid dock zone specified for registration
    /// Addresses: Requirement 1 criterion 3
    #[error("[layout] register: invalid dock zone '{zone}' for panel '{panel_id}'")]
    InvalidDockZone { panel_id: String, zone: String },

    /// Invalid panel_id format (must be 1–64 ASCII alphanumeric/underscore)
    #[error("[layout] register: invalid panel_id format '{panel_id}' — {reason}")]
    InvalidPanelId { panel_id: String, reason: String },

    /// Maximum floating windows reached
    /// Addresses: Requirement 3 criterion 14
    #[error("[layout] undock: maximum floating windows ({max}) reached")]
    MaxFloatingWindows { max: usize },

    /// OS failed to create a floating window
    /// Addresses: Requirement 3 criterion 15
    #[error("[layout] undock: OS window creation failed for panel '{panel_id}'")]
    WindowCreationFailed { panel_id: String },

    /// Floating window not found
    #[error("[layout] floating: window {window_id:?} not found")]
    FloatingWindowNotFound { window_id: FloatingWindowId },

    /// Tab group not found
    #[error("[layout] tabs: group {group_id:?} not found")]
    TabGroupNotFound { group_id: TabGroupId },

    /// Cannot split — would create empty editor area
    /// Addresses: Requirement 9 criterion 4
    #[error("[layout] split: cannot undock the only tab in the only group")]
    CannotEmptyEditor,

    /// Persona not found
    #[error("[layout] persona: '{name}' not found")]
    PersonaNotFound { name: String },

    /// Cannot delete a built-in persona
    /// Addresses: Requirement 5 criterion 6
    #[error("[layout] persona: cannot delete built-in persona '{name}'")]
    CannotDeleteBuiltIn { name: String },

    /// Serialization/deserialization failure
    /// Addresses: Requirement 6 criteria 3/10
    #[error("[layout] serialization: {operation} failed — {reason}")]
    SerializationFailed { operation: String, reason: String },

    /// I/O error during file operations
    #[error("[layout] io: {0}")]
    Io(#[from] std::io::Error),

    /// Splitter not found
    #[error("[layout] splitter: {splitter_id:?} not found")]
    SplitterNotFound { splitter_id: SplitterId },

    /// Tab index out of bounds
    #[error("[layout] tab: index {index} out of bounds for group {group_id:?} (has {count} tabs)")]
    TabIndexOutOfBounds {
        group_id: TabGroupId,
        index: usize,
        count: usize,
    },
}
```

---

## 7. Integration Points

### With `ff-logging` (Foundation Layer — upstream)

- **Dependency direction**: ff-layout depends on ff-logging
- **API consumed**: `log_info!`, `log_warn!`, `log_error!` macros
- **Usage**:
  - INFO on panel registration/deregistration
  - INFO on persona activation
  - WARN when persisted layout references unregistered panel_id (Requirement 6 criterion 5)
  - WARN when layout file fails to parse (Requirement 6 criterion 3)
  - WARN when session save fails at exit (Requirement 6 criterion 10)
  - ERROR on invalid dock zone registration (Requirement 1 criterion 3)

### With `ff-core` (Platform Core — same wave, consumer)

- **Dependency direction**: ff-core initializes ff-layout as a registered subsystem
- **Integration**:
  - ff-core calls `LayoutEngine::new()` during startup sequence
  - ff-core calls `LayoutEngine::save_session()` during shutdown
  - ff-core provides `LayoutState` from config at startup for restoration
  - ff-layout dispatches `WorkbenchEvent::LayoutChanged` through the Event Bus when layout mutates
- **Event Bus events emitted**:
  - `LayoutChanged` — any structural layout change (dock, undock, split, persona switch)
  - `PanelStateChanged { panel_id, new_state }` — individual panel state transitions

### With `ff-command` (Command Framework — same wave, peer)

- **Dependency direction**: ff-layout registers layout commands with ff-command
- **Commands registered**:
  - `layout.undock` — undock focused panel (Ctrl+Shift+D)
  - `layout.redock` — redock focused floating panel
  - `layout.toggle_panel` — toggle a named panel
  - `layout.split_horizontal` — split active tab group horizontally
  - `layout.split_vertical` — split active tab group vertically
  - `layout.undock_tab` — undock active tab (Ctrl+Shift+T)
  - `layout.redock_tab` — redock floating tab
  - `layout.persona.activate` — activate a persona by name
  - `layout.persona.save` — save current layout as persona
  - `layout.reset` — reset to default layout
  - `layout.export` — export layout to file
  - `layout.import` — import layout from file
- **Shortcut registrations**: All shortcuts registered with `ShortcutRegistry` (Requirement 9 criterion 6)

### With `ff-plugin` (Plugin Architecture — same wave, peer)

- **Dependency direction**: ff-plugin uses ff-layout's `PanelRegistry` to register panels
- **Integration**:
  - Plugins call `PanelRegistry::register()` during their `initialize` phase (Requirement 1 criterion 14)
  - During plugin unload, ff-plugin calls `PanelRegistry::deregister()` and `LayoutEngine::hide_panel()` to clean up (Requirement 1 criterion 14)
  - `PanelRegistry` is accessible via `PluginContext`

### With `ff-config` (Configuration System — same wave, peer)

- **Dependency direction**: ff-layout reads layout config from ff-config at startup
- **Configuration consumed**:
  - Session file path (`config/layout_state.toml`)
  - Persona directory path (`layouts/`)
  - Default persona to activate on first launch

### With `ff-desktop` (Shell Layer — downstream)

- **Dependency direction**: ff-desktop depends on ff-layout; ff-layout NEVER depends on ff-desktop
- **Shell responsibilities**:
  - Render the layout tree (dock zones, tab groups, splitters) based on `LayoutState`
  - Create/destroy OS-level floating windows as directed by `FloatingWindowManager`
  - Forward user input (drag events, splitter drags, window moves) to `LayoutEngine`
  - Render `DropIndicator` overlays during drag operations
  - Display persona name and modification indicator in status bar
  - Render placeholder indicators for floating panels (Requirement 10 criterion 1)
  - Handle DPI-aware rendering per monitor (Requirement 4 criteria 4/5)

### Dependency Direction Summary

```
ff-logging ← ff-layout ← ff-desktop
              ff-layout ← ff-core (lifecycle)
              ff-layout → ff-command (command registration)
              ff-layout ← ff-plugin (panel registration)
              ff-layout ← ff-config (settings)
```

---

## 8. Configuration

Layout configuration is managed through two channels: the workbench TOML config (system settings) and per-persona TOML files (layout presets).

### Workbench TOML Schema (`[layout]` section)

```toml
[layout]
# Path to the session state file (relative to workbench config dir).
# Default: "config/layout_state.toml"
session_file = "config/layout_state.toml"

# Path to persona definitions directory.
# Default: "layouts/"
persona_directory = "layouts/"

# Default persona to activate on first launch (no session file exists).
# Default: "Editor Focus"
default_persona = "Editor Focus"

# Maximum floating windows allowed simultaneously.
# Range: 1–16. Default: 16
max_floating_windows = 16

# Minimum tab group size in logical pixels (split direction).
# Range: 50–500. Default: 100
# Addresses: Requirement 2 criterion 7
min_tab_group_size = 100

# Default minimum panel size in logical pixels (both dimensions).
# Range: 24–200. Default: 48
# Addresses: Requirement 8 criterion 4
default_min_panel_size = 48
```

### Persona TOML Format (`layouts/<name>.toml`)

```toml
[persona]
name = "Editor Focus"
built_in = true
description = "Minimal panels, maximized editor area"

[persona.layout]
schema_version = 1

[[persona.layout.docked_panels]]
panel_id = "file_tree"
zone = "Left"
zone_dimension = 250.0

[[persona.layout.docked_panels]]
panel_id = "output"
zone = "Bottom"
zone_dimension = 200.0

[persona.layout.tab_groups]
# Encoded as the TabGroupTree structure
direction = "Horizontal"
proportion = 0.5

[persona.layout.splitter_positions]
"left_center" = 0.2
"center_right" = 0.8
"center_bottom" = 0.75

[persona.layout.panel_visibility]
file_tree = true
output = false
properties = false
```

### Config Resolution Rules

| Setting | Absent | Invalid Value | Out of Range |
|---------|--------|---------------|--------------|
| `session_file` | Default path | Default path + WARN | N/A |
| `persona_directory` | Default path | Default path + WARN | N/A |
| `default_persona` | "Editor Focus" | "Editor Focus" + WARN | N/A |
| `max_floating_windows` | 16 | 16 + WARN | Clamp to [1–16] + WARN |
| `min_tab_group_size` | 100 | 100 + WARN | Clamp to [50–500] + WARN |
| `default_min_panel_size` | 48 | 48 + WARN | Clamp to [24–200] + WARN |

---

## 9. Concurrency Model

### Thread-Safety Approach

The layout engine operates primarily on the **main/GUI thread**. Layout mutations are synchronous and immediate to provide real-time visual feedback (Requirement 8 criterion 9). Background operations are limited to serialization I/O.

| Component | Thread Context | Mechanism |
|-----------|---------------|-----------|
| **LayoutEngine** | Main thread | Single-threaded; all mutations via method calls from shell event loop |
| **PanelRegistry** | Main thread + plugin thread | `Arc<Mutex<PanelRegistry>>` for safe registration from plugin init |
| **Serialization** | Tokio worker | Async file I/O for save/load; results delivered via channel |
| **DragDropCoordinator** | Main thread | State machine driven by frame-rate input events |
| **FloatingWindowManager** | Main thread | Window creation/destruction dispatched to OS; positions tracked synchronously |

### Communication Channels

| Channel | Direction | Purpose |
|---------|-----------|---------|
| `LayoutEngine` → `EventBus` | Core → Shell | Notify shell of layout changes requiring re-render |
| `Serializer` → `LayoutEngine` | Tokio → Main | Deliver loaded LayoutState from disk at startup |
| `LayoutEngine` → `Serializer` | Main → Tokio | Dispatch save operations without blocking shutdown |

### Why Main-Thread for Layout Mutations

- Layout changes must be reflected in the same frame they occur (Requirement 7 criterion 5: 16ms indicator)
- Drag-and-drop state machine requires frame-coherent updates
- Splitter dragging needs zero-latency feedback (Requirement 8 criterion 9)
- Panel render calls (`DockablePanel::render`) happen on the GUI thread

### Serialization Strategy

- **Save on exit**: `save_session()` serializes to TOML on a Tokio worker with 3-second timeout. On timeout or error, logs WARN and allows exit (Requirement 6 criterion 10)
- **Auto-save debounce**: After layout changes, a 2-second debounce timer triggers background save to prevent data loss on crash
- **Startup restore**: `LayoutState` is loaded from disk before the shell renders. If loading fails, default layout is used (Requirement 6 criterion 3)

---

## 10. Correctness Properties

These properties are suitable for property-based testing with `proptest`. They validate invariants that must hold across all valid inputs.

### Property 1: Panel Registration Uniqueness

**Statement**: For any sequence of panel registrations, the PanelRegistry contains at most one entry per `panel_id`. A registration with an existing ID always returns `DuplicatePanelId` error without modifying state.

**Validates**: Requirement 1, criterion 10

```rust
// proptest strategy: generate sequences of (panel_id, zone) registration attempts
// assertion: after all registrations, registry.list_all() has no duplicates
//            AND every duplicate attempt returned Err(DuplicatePanelId)
```

### Property 2: Dock/Undock Round-Trip Preserves Panel Identity

**Statement**: For any panel that is undocked into a floating window and then redocked, the panel remains in the PanelRegistry with its original `panel_id`, and its final dock zone matches its pre-undock zone.

**Validates**: Requirement 3, criteria 1/5/7

```rust
// proptest strategy: generate initial docked panels, pick one to undock then redock
// assertion: panel_id remains registered AND zone == original zone
```

### Property 3: Tab Group Split Preserves Total Tab Count

**Statement**: For any split operation (horizontal or vertical) on a tab group containing N tabs, the total number of tabs across the resulting two groups equals N.

**Validates**: Requirement 2, criteria 2/3

```rust
// proptest strategy: generate tab group with 1..20 tabs, apply split
// assertion: sum of tab counts in both child groups == original N
```

### Property 4: Empty Tab Group Elimination

**Statement**: After any sequence of tab moves between groups, no empty TabGroup exists in the TabGroupTree. Moving the last tab from a group causes that group to be removed.

**Validates**: Requirement 2, criterion 5

```rust
// proptest strategy: generate tab group tree, apply sequence of move_tab operations
// assertion: all leaf nodes in TabGroupTree have tabs.len() >= 1
```

### Property 5: Floating Window Count Bound

**Statement**: The number of active floating windows never exceeds `MAX_FLOATING_WINDOWS` (16). Any undock attempt beyond this limit returns `MaxFloatingWindows` error.

**Validates**: Requirement 3, criterion 14

```rust
// proptest strategy: generate sequence of undock operations (up to 20)
// assertion: floating_window_count() <= 16 at all times
//            AND attempts 17+ return Err(MaxFloatingWindows)
```

### Property 6: Splitter Proportion Invariant

**Statement**: For any splitter drag operation, the resulting proportion is clamped to respect both adjacent minimum sizes. The proportion is always in [0.0, 1.0], and neither side can be reduced below its declared minimum.

**Validates**: Requirement 8, criteria 3/4/5

```rust
// proptest strategy: generate splitter with min_first, min_second, total_size,
//                    and arbitrary drag target proportion
// assertion: result proportion ∈ [min_first/total, 1.0 - min_second/total]
```

### Property 7: Layout Serialization Round-Trip

**Statement**: For any valid `LayoutState`, serializing to TOML and deserializing produces an equivalent `LayoutState`. Fields are preserved including panel positions, tab group tree structure, floating window coordinates, and splitter proportions.

**Validates**: Requirement 6, criteria 1/2/4

```rust
// proptest strategy: generate arbitrary valid LayoutState
// assertion: deserialize(serialize(state)) == state
```

### Property 8: Persona Activation Preserves Open Tabs

**Statement**: For any persona activation with N open tabs and a target persona defining M tab groups, all N tabs are present in the resulting layout. No tab is lost or duplicated.

**Validates**: Requirement 5, criterion 5

```rust
// proptest strategy: generate current state with N tabs and target persona with M groups
// assertion: set of tab_ids after activation == set of tab_ids before activation
```

### Property 9: Proportional Resize Maintains Ratios

**Statement**: When the primary window is resized, the relative proportions between dock zones remain unchanged (within floating-point tolerance), subject to minimum size constraints.

**Validates**: Requirement 8, criterion 5

```rust
// proptest strategy: generate initial layout with splitter proportions, apply window resize
// assertion: new_proportion ≈ old_proportion (within epsilon)
//            OR minimum constraint was active (zone at minimum size)
```

### Property 10: Panel Visibility Toggle Idempotence

**Statement**: For any panel, calling `toggle_panel` twice in sequence (show then hide, or hide then show) returns the panel to its original visibility state.

**Validates**: Requirement 1, criterion 12

```rust
// proptest strategy: generate panel with random initial visibility
// assertion: toggle(toggle(state)) == state
```

---

## Appendix A: External Crate Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `egui` | 0.28+ | `DockablePanel::render` trait method signature only |
| `serde` | 1.0 | Serialization derives for LayoutState, Persona, etc. |
| `toml` | 0.8 | TOML serialization/deserialization for layout files |
| `thiserror` | 2.0 | Error type derivation |
| `proptest` | 1.0 | Property-based testing (dev-dependency only) |

## Appendix B: Built-In Personas

| Name | Description | Key Characteristics |
|------|-------------|-------------------|
| Editor Focus | Minimal panels, maximized editor area | Only center tab groups visible; file tree collapsed |
| Debug | Output and variable panels visible | Bottom panel expanded; left panel shows call stack |
| FileForge | File tree and structure panels prominent | Left panel (file tree) expanded; right panel (structure) visible |
| Database | Schema browser, SQL editor, result grid | Left (schema), center (SQL tabs), bottom (results) |

## Appendix C: Keyboard Shortcuts (Default Bindings)

| Shortcut | Command | Description |
|----------|---------|-------------|
| Ctrl+Shift+D | `layout.undock` / `layout.redock` | Toggle dock/float for focused panel |
| Ctrl+Shift+T | `layout.undock_tab` / `layout.redock_tab` | Toggle dock/float for active tab |
| Ctrl+\| | `layout.split_horizontal` | Split active tab group horizontally |
| Ctrl+- | `layout.split_vertical` | Split active tab group vertically |

All shortcuts are registered with `ff-command::ShortcutRegistry` and can be remapped via the user key map (Requirement 9 criterion 6).

## Appendix D: Drag Gesture Thresholds

| Gesture | Threshold | Reference |
|---------|-----------|-----------|
| Tab tear-off (vertical) | 30px from tab bar | Requirement 7 criterion 11 |
| Tab cancel (return) | Within 30px of tab bar | Requirement 7 criterion 12 |
| Drag-to-float (outside window) | 20px beyond window boundary | Requirement 3 criterion 9 |
| Drop indicator appearance | 16ms (one frame at 60 FPS) | Requirement 7 criterion 5 |
