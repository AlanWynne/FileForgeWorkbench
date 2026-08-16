# Implementation Plan: Layout and Docking System (`ff-layout`)

## Overview

Implement the `ff-layout` crate — the GUI-independent layout engine for FileForgeWorkbench. This crate owns spatial arrangement of all panels, tab groups, floating windows, and dock zones. The shell layer (`ff-desktop`) renders the layout model but does not own it.

Coverage: 10 requirements, ~95 acceptance criteria, 10 correctness properties.

---

## Tasks

- [ ] 1. Crate scaffold and error types
  - [ ] 1.1 Create `crates/ff-layout/Cargo.toml` with dependencies (egui, serde, toml, thiserror, proptest dev-dep)
  - [ ] 1.2 Create `src/lib.rs` with crate-level docs and module declarations
  - [ ] 1.3 Implement `src/error.rs` with `LayoutError` enum (all variants from design §6)
    - Validates: Cross-cutting Requirement 8 (error message format)
  - [ ] 1.4 Create module structure: `panel/`, `dock/`, `tabs/`, `floating/`, `persona/`, `drag/`, `resize/`, `state/`
  - [ ] 1.5 Verify crate compiles with `cargo check`

- [ ] 2. Core data models
  - [ ] 2.1 Implement `src/dock/zone.rs` — `DockZone` enum (Left, Right, Bottom, Center, Floating)
    - Validates: Req 1 criteria 1, 3, 5
  - [ ] 2.2 Implement `src/panel/traits.rs` — `DockablePanel` trait with all methods
    - Validates: Req 1 criteria 4, 5, 6, 7, 8; Req 8 criteria 3, 4
  - [ ] 2.3 Implement `src/panel/display_state.rs` — `PanelDisplayState` and `DockState` enums
    - Validates: Req 1 criteria 8, 13
  - [ ] 2.4 Implement shared geometry types (`Position`, `Size`, `Rect`) in `src/lib.rs` or utility module
  - [ ] 2.5 Implement `src/tabs/group.rs` — `TabGroup`, `TabGroupId`, `TabGroupTree`, `SplitDirection`
    - Validates: Req 2 criteria 1, 8
  - [ ] 2.6 Implement `src/floating/window.rs` — `FloatingWindow`, `FloatingWindowId`
    - Validates: Req 3 criterion 16; Req 4 criterion 3
  - [ ] 2.7 Implement `src/resize/splitter.rs` — `Splitter`, `SplitterId`, `SplitterOrientation`
    - Validates: Req 8 criteria 1, 7, 8
  - [ ] 2.8 Implement `src/persona/definition.rs` — `Persona`, `PersonaKind`
    - Validates: Req 5 criterion 1
  - [ ] 2.9 Implement `src/drag/indicator.rs` — `DropIndicator`, `DropPlacement`, `SplitSide`
    - Validates: Req 7 criteria 4, 6; Req 10 criterion 6
  - [ ] 2.10 Implement `src/state/layout_state.rs` — `LayoutState`, `DockedPanelState`
    - Validates: Req 6 criteria 4, 11

- [ ] 3. Panel Registry
  - [ ] 3.1 Implement `src/panel/registry.rs` — `PanelRegistry::new()`, `register()`, `deregister()`, `get()`, `list_all()`, `is_registered()`
    - Validates: Req 1 criteria 2, 3, 9, 10, 14
  - [ ] 3.2 Implement panel_id validation (1–64 ASCII alphanumeric/underscore)
    - Validates: Req 1 criterion 4
  - [ ] 3.3 Implement duplicate panel_id rejection
    - Validates: Req 1 criterion 10
  - [ ] 3.4 Implement invalid dock zone rejection with error logging
    - Validates: Req 1 criterion 3
  - [ ] 3.5 Write unit tests for PanelRegistry (register, deregister, duplicate, invalid zone, format)
  - [ ] 3.6 Write property test: Panel Registration Uniqueness (Property 1)
    - Validates: Req 1 criterion 10

- [ ] 4. Layout Engine — core lifecycle
  - [ ] 4.1 Implement `src/engine.rs` — `LayoutEngine::new()` with default dock zones
    - Validates: Req 1 criterion 1
  - [ ] 4.2 Implement `LayoutEngine::from_state()` for startup restoration
    - Validates: Req 6 criteria 2, 3, 5
  - [ ] 4.3 Implement `current_state()`, `is_persona_modified()`, `active_persona_name()`
    - Validates: Req 5 criteria 9, 10
  - [ ] 4.4 Write unit tests for LayoutEngine construction and state accessors

- [ ] 5. Panel operations (show, hide, toggle, minimize, maximize, restore)
  - [ ] 5.1 Implement `show_panel()` — make hidden panel visible in last known zone
    - Validates: Req 1 criterion 11
  - [ ] 5.2 Implement `hide_panel()` — remove from view, preserve position in LayoutState
    - Validates: Req 1 criterion 11
  - [ ] 5.3 Implement `toggle_panel()` — show if hidden, hide if visible
    - Validates: Req 1 criterion 12
  - [ ] 5.4 Implement `minimize_panel()` — collapse to tab/icon in zone header
    - Validates: Req 1 criterion 13
  - [ ] 5.5 Implement `maximize_panel()` — expand to fill primary window content area
    - Validates: Req 1 criterion 13
  - [ ] 5.6 Implement `restore_panel()` — return to normal display state
    - Validates: Req 1 criterion 13
  - [ ] 5.7 Write unit tests for panel visibility operations
  - [ ] 5.8 Write property test: Panel Visibility Toggle Idempotence (Property 10)
    - Validates: Req 1 criterion 12

- [ ] 6. Tab Group Manager
  - [ ] 6.1 Implement `src/tabs/manager.rs` — `TabGroupManager` with split tree coordination
    - Validates: Req 2 criterion 1
  - [ ] 6.2 Implement `split_horizontal()` — divide active group side-by-side, move active tab
    - Validates: Req 2 criterion 2
  - [ ] 6.3 Implement `split_vertical()` — divide active group top/bottom, move active tab
    - Validates: Req 2 criterion 3
  - [ ] 6.4 Implement `move_tab()` — relocate tab between groups, close empty groups
    - Validates: Req 2 criteria 4, 5
  - [ ] 6.5 Implement `add_tab()` — add tab to active or specified group
    - Validates: Req 2 criterion 9
  - [ ] 6.6 Implement `active_tab_group()` and `set_active_tab_group()`
  - [ ] 6.7 Implement minimum tab group size enforcement (100 logical px)
    - Validates: Req 2 criterion 7
  - [ ] 6.8 Implement tab group tree serialization support
    - Validates: Req 2 criterion 8
  - [ ] 6.9 Write unit tests for tab group split/merge operations
  - [ ] 6.10 Write property test: Tab Group Split Preserves Total Tab Count (Property 3)
    - Validates: Req 2 criteria 2, 3
  - [ ] 6.11 Write property test: Empty Tab Group Elimination (Property 4)
    - Validates: Req 2 criterion 5

- [ ] 7. Floating Window Manager
  - [ ] 7.1 Implement `src/floating/manager.rs` — `FloatingWindowManager` creation and tracking
    - Validates: Req 3 criterion 1
  - [ ] 7.2 Implement `undock_panel()` — remove from dock zone, create floating window
    - Validates: Req 3 criteria 1, 2, 4
  - [ ] 7.3 Implement `undock_panel_at()` — undock to specific position (drag-to-float)
    - Validates: Req 3 criterion 9; Req 7 criterion 9
  - [ ] 7.4 Implement `redock_panel()` — close floating window, reattach to recent zone
    - Validates: Req 3 criteria 5, 6, 7
  - [ ] 7.5 Implement `undock_tab()` and `undock_tab_at()` — tab tear-off to float
    - Validates: Req 3 criterion 9; Req 9 criterion 3
  - [ ] 7.6 Implement `redock_tab()` — return tab to originating group at original index
    - Validates: Req 3 criterion 11
  - [ ] 7.7 Implement `update_floating_window()` — track position/size updates
    - Validates: Req 3 criterion 4; Req 6 criterion 9
  - [ ] 7.8 Implement `on_floating_window_close()` — OS close button redock logic
    - Validates: Req 3 criteria 8, 11, 12
  - [ ] 7.9 Implement floating window count limit enforcement (MAX = 16)
    - Validates: Req 3 criterion 14
  - [ ] 7.10 Implement OS window creation failure handling
    - Validates: Req 3 criterion 15
  - [ ] 7.11 Implement cascade offset positioning (50×N pixels)
    - Validates: Req 3 criterion 2
  - [ ] 7.12 Implement full interactivity assertion for floating panels
    - Validates: Req 3 criteria 3, 10, 13
  - [ ] 7.13 Write unit tests for floating window lifecycle
  - [ ] 7.14 Write property test: Dock/Undock Round-Trip Preserves Panel Identity (Property 2)
    - Validates: Req 3 criteria 1, 5, 7
  - [ ] 7.15 Write property test: Floating Window Count Bound (Property 5)
    - Validates: Req 3 criterion 14

- [ ] 8. Multi-Monitor Support
  - [ ] 8.1 Implement `src/floating/monitor.rs` — `MonitorInfo` struct and detection helpers
    - Validates: Req 4 criterion 1
  - [ ] 8.2 Implement `update_window_monitor()` — record monitor assignment on move
    - Validates: Req 4 criterion 2
  - [ ] 8.3 Implement monitor identifier persistence in LayoutState
    - Validates: Req 4 criterion 3
  - [ ] 8.4 Implement DPI scale factor tracking per floating window
    - Validates: Req 4 criteria 4, 5
  - [ ] 8.5 Implement `on_monitor_disconnected()` — relocate windows to primary monitor
    - Validates: Req 4 criterion 6
  - [ ] 8.6 Implement `validate_window_positions()` — startup repositioning for missing monitors
    - Validates: Req 4 criterion 7
  - [ ] 8.7 Implement 50% visibility check for window positioning at startup
    - Validates: Req 4 criterion 8
  - [ ] 8.8 Write unit tests for multi-monitor repositioning logic

- [ ] 9. Persona Manager
  - [ ] 9.1 Implement `src/persona/manager.rs` — `PersonaManager` with built-in persona definitions
    - Validates: Req 5 criterion 2
  - [ ] 9.2 Implement `activate_persona()` — transition layout to match persona config
    - Validates: Req 5 criteria 4, 5
  - [ ] 9.3 Implement open document preservation during persona switch (excess tabs to last group)
    - Validates: Req 5 criterion 5
  - [ ] 9.4 Implement `save_persona()` — save current LayoutState as custom persona
    - Validates: Req 5 criterion 3
  - [ ] 9.5 Implement `delete_persona()` — delete custom; reject built-in deletion
    - Validates: Req 5 criterion 6
  - [ ] 9.6 Implement `update_active_persona()` and `revert_to_persona()`
    - Validates: Req 5 criterion 10
  - [ ] 9.7 Implement `list_personas()` — return all built-in and custom
  - [ ] 9.8 Implement persona modification tracking (mark as "modified" on layout change)
    - Validates: Req 5 criterion 10
  - [ ] 9.9 Implement missing panel_id graceful skip during persona activation
    - Validates: Req 5 criterion 8
  - [ ] 9.10 Implement active persona name tracking for status bar display
    - Validates: Req 5 criterion 9
  - [ ] 9.11 Write unit tests for persona lifecycle
  - [ ] 9.12 Write property test: Persona Activation Preserves Open Tabs (Property 8)
    - Validates: Req 5 criterion 5

- [ ] 10. Layout Serialization
  - [ ] 10.1 Implement `src/state/serializer.rs` — TOML serialize/deserialize for LayoutState
    - Validates: Req 6 criteria 1, 4, 11
  - [ ] 10.2 Implement `save_session()` — serialize to `config/layout_state.toml`
    - Validates: Req 6 criterion 1
  - [ ] 10.3 Implement startup restoration from persisted file
    - Validates: Req 6 criterion 2
  - [ ] 10.4 Implement graceful degradation (invalid TOML, schema mismatch → default layout + WARN)
    - Validates: Req 6 criterion 3
  - [ ] 10.5 Implement unregistered panel_id skip during restoration (INFO log)
    - Validates: Req 6 criterion 5
  - [ ] 10.6 Implement `export_layout()` — serialize to user-specified path
    - Validates: Req 6 criterion 6
  - [ ] 10.7 Implement `import_layout()` — apply imported state with graceful degradation
    - Validates: Req 6 criterion 7
  - [ ] 10.8 Implement `reset_to_default()` — discard state, restore built-in default
    - Validates: Req 6 criterion 8
  - [ ] 10.9 Implement in-memory state update within 500ms of floating window move/resize
    - Validates: Req 6 criterion 9
  - [ ] 10.10 Implement I/O failure handling at exit (WARN log, allow exit)
    - Validates: Req 6 criterion 10
  - [ ] 10.11 Implement schema version field in serialization format
    - Validates: Req 6 criterion 11
  - [ ] 10.12 Write unit tests for serialization round-trip and error paths
  - [ ] 10.13 Write property test: Layout Serialization Round-Trip (Property 7)
    - Validates: Req 6 criteria 1, 2, 4

- [ ] 11. Drag-and-Drop Coordinator
  - [ ] 11.1 Implement `src/drag/coordinator.rs` — `DragDropCoordinator` state machine (idle, dragging, preview)
    - Validates: Req 7 criterion 11
  - [ ] 11.2 Implement `begin_drag()` — initiate drag from panel header or tab
    - Validates: Req 7 criterion 11
  - [ ] 11.3 Implement `update_drag()` — hit testing and drop indicator placement
    - Validates: Req 7 criteria 1, 5, 6, 13
  - [ ] 11.4 Implement `end_drag()` — execute drop or cancel; return DragResult
    - Validates: Req 7 criteria 2, 3, 7, 8, 9, 10, 12
  - [ ] 11.5 Implement `cancel_drag()` and `is_dragging()`
  - [ ] 11.6 Implement `src/drag/hit_test.rs` — zone/group hit testing and insertion index calculation
    - Validates: Req 7 criteria 7, 12
  - [ ] 11.7 Implement `src/drag/indicator.rs` rendering model — semi-transparent overlay with border
    - Validates: Req 7 criteria 4, 5, 6; Req 10 criterion 6
  - [ ] 11.8 Implement tab tear-off detection (30px vertical threshold)
    - Validates: Req 7 criterion 11
  - [ ] 11.9 Implement tab tear-off cancel (return within 30px)
    - Validates: Req 7 criterion 12
  - [ ] 11.10 Implement drag-to-float (release 20px outside primary window)
    - Validates: Req 3 criterion 9; Req 7 criterion 9
  - [ ] 11.11 Implement drag-to-dock (floating window title bar → dock zone)
    - Validates: Req 7 criteria 1, 2, 10
  - [ ] 11.12 Implement dock zone highlight during drag (2px distinct border)
    - Validates: Req 7 criterion 13
  - [ ] 11.13 Implement 16ms drop indicator responsiveness
    - Validates: Req 7 criterion 5
  - [ ] 11.14 Write unit tests for drag state machine and hit testing

- [ ] 12. Splitter / Resize Manager
  - [ ] 12.1 Implement `src/resize/manager.rs` — `SplitterManager` with constraint enforcement
    - Validates: Req 8 criteria 1, 2
  - [ ] 12.2 Implement `begin_splitter_drag()` and `end_splitter_drag()`
    - Validates: Req 8 criterion 9
  - [ ] 12.3 Implement `update_splitter()` — enforce min size constraints during drag
    - Validates: Req 8 criteria 3, 4, 9
  - [ ] 12.4 Implement default minimum size enforcement (48 logical px)
    - Validates: Req 8 criterion 4
  - [ ] 12.5 Implement `on_window_resize()` — proportional redistribution of all zones
    - Validates: Req 8 criterion 5
  - [ ] 12.6 Implement priority resize logic (center area preserved, side/bottom reduced first)
    - Validates: Req 8 criterion 6
  - [ ] 12.7 Implement splitter position persistence as proportional values [0.0, 1.0]
    - Validates: Req 8 criterion 7
  - [ ] 12.8 Implement `reset_splitter()` — double-click reset to default position
    - Validates: Req 8 criterion 8
  - [ ] 12.9 Implement real-time visual feedback (resize both sides each frame)
    - Validates: Req 8 criterion 9
  - [ ] 12.10 Write unit tests for splitter constraint enforcement
  - [ ] 12.11 Write property test: Splitter Proportion Invariant (Property 6)
    - Validates: Req 8 criteria 3, 4, 5
  - [ ] 12.12 Write property test: Proportional Resize Maintains Ratios (Property 9)
    - Validates: Req 8 criterion 5

- [ ] 13. Persona storage (TOML file I/O)
  - [ ] 13.1 Implement `src/persona/storage.rs` — read/write persona TOML files from `layouts/` directory
    - Validates: Req 5 criterion 7
  - [ ] 13.2 Implement built-in persona definitions (Editor Focus, Debug, FileForge, Database)
    - Validates: Req 5 criterion 2
  - [ ] 13.3 Implement persona file discovery and loading at startup
  - [ ] 13.4 Write unit tests for persona TOML serialization/deserialization

- [ ] 14. Command registration
  - [ ] 14.1 Implement `src/commands.rs` — register all layout commands with ff-command
    - Validates: Req 9 criterion 6
  - [ ] 14.2 Register `layout.undock` (Ctrl+Shift+D toggle dock/float)
    - Validates: Req 9 criteria 1, 2
  - [ ] 14.3 Register `layout.undock_tab` (Ctrl+Shift+T toggle tab float)
    - Validates: Req 9 criteria 3, 4, 5
  - [ ] 14.4 Register `layout.split_horizontal` and `layout.split_vertical`
    - Validates: Req 9 criterion 8
  - [ ] 14.5 Register `layout.persona.activate`, `layout.persona.save`
    - Validates: Req 9 criterion 7
  - [ ] 14.6 Register `layout.reset`, `layout.export`, `layout.import`, `layout.toggle_panel`
  - [ ] 14.7 Implement no-op guard when no panel has focus (Ctrl+Shift+D)
    - Validates: Req 9 criterion 2
  - [ ] 14.8 Implement empty-editor guard (Ctrl+Shift+T on only tab in only group)
    - Validates: Req 9 criterion 4
  - [ ] 14.9 Write unit tests for command dispatch behavior

- [ ] 15. Visual feedback and indicators
  - [ ] 15.1 Implement placeholder indicator model for floating panels (panel name + redock button)
    - Validates: Req 10 criterion 1
  - [ ] 15.2 Implement tooltip model for placeholder hover (300ms delay, "Click to redock [name]")
    - Validates: Req 10 criterion 2
  - [ ] 15.3 Implement placeholder click → redock behavior
    - Validates: Req 10 criterion 3
  - [ ] 15.4 Implement floating window title format ("{title} — FileForge", max 80 chars)
    - Validates: Req 10 criterion 4
  - [ ] 15.5 Implement status bar persona indicator model (name + "modified" flag)
    - Validates: Req 10 criterion 5
  - [ ] 15.6 Implement drop indicator placement precision (exact position preview)
    - Validates: Req 10 criterion 6
  - [ ] 15.7 Implement minimized panel icon/label in dock zone header (click to restore)
    - Validates: Req 10 criterion 7
  - [ ] 15.8 Write unit tests for visual feedback data models

- [ ] 16. Integration wiring
  - [ ] 16.1 Implement event bus emission (LayoutChanged, PanelStateChanged) via ff-core
  - [ ] 16.2 Implement plugin panel registration/deregistration via PluginContext
    - Validates: Req 1 criterion 14
  - [ ] 16.3 Implement configuration loading from ff-config ([layout] section)
  - [ ] 16.4 Implement async serialization via Tokio worker (save on exit with 3s timeout)
  - [ ] 16.5 Implement auto-save debounce (2s after layout changes)
  - [ ] 16.6 Write integration tests for cross-crate wiring

- [ ] 17. End-to-end integration tests
  - [ ] 17.1 Write integration test: full panel lifecycle (register → dock → undock → float → redock → hide → show)
  - [ ] 17.2 Write integration test: tab group workflow (open tabs → split → move tabs → close empty group)
  - [ ] 17.3 Write integration test: persona workflow (activate → modify → revert → save custom → delete)
  - [ ] 17.4 Write integration test: serialization workflow (save session → quit → restore → verify state)
  - [ ] 17.5 Write integration test: drag-and-drop scenarios (dock zone, tab group, float)
  - [ ] 17.6 Write integration test: multi-monitor disconnect/reconnect repositioning
  - [ ] 17.7 Write integration test: window resize proportional redistribution with min constraints

---

## Property-Based Test Definitions

| # | Property Name | Statement | Validates | Strategy |
|---|--------------|-----------|-----------|----------|
| 1 | Panel Registration Uniqueness | For any sequence of panel registrations, the PanelRegistry contains at most one entry per `panel_id`. Duplicate attempts return `DuplicatePanelId` error without modifying state. | Req 1.10 | Generate sequences of (panel_id, zone) registration attempts; assert no duplicates in registry and duplicate attempts return Err |
| 2 | Dock/Undock Round-Trip Preserves Panel Identity | For any panel undocked then redocked, the panel remains registered with original panel_id, and final zone matches pre-undock zone. | Req 3.1, 3.5, 3.7 | Generate initial docked panels, pick one to undock then redock; assert identity and zone preserved |
| 3 | Tab Group Split Preserves Total Tab Count | For any split (horizontal or vertical) on a group with N tabs, total tabs across resulting groups equals N. | Req 2.2, 2.3 | Generate tab group with 1..20 tabs, apply split; assert sum equals original |
| 4 | Empty Tab Group Elimination | After any sequence of tab moves, no empty TabGroup exists. Moving last tab from a group removes it. | Req 2.5 | Generate tab group tree, apply move_tab sequences; assert all leaves have ≥1 tab |
| 5 | Floating Window Count Bound | Active floating windows never exceed 16. Attempts beyond limit return MaxFloatingWindows error. | Req 3.14 | Generate up to 20 undock operations; assert count ≤ 16 and overflow returns Err |
| 6 | Splitter Proportion Invariant | For any splitter drag, result proportion respects both adjacent minimum sizes and stays in [0.0, 1.0]. | Req 8.3, 8.4, 8.5 | Generate splitter with mins and arbitrary target; assert clamped correctly |
| 7 | Layout Serialization Round-Trip | For any valid LayoutState, serialize→deserialize produces equivalent state. | Req 6.1, 6.2, 6.4 | Generate arbitrary valid LayoutState; assert round-trip equality |
| 8 | Persona Activation Preserves Open Tabs | For persona activation with N tabs and M target groups, all N tabs present after — none lost or duplicated. | Req 5.5 | Generate state with N tabs and persona with M groups; assert tab set equality |
| 9 | Proportional Resize Maintains Ratios | On window resize, relative proportions unchanged (within ε) unless minimum constraint active. | Req 8.5 | Generate layout with proportions, resize; assert proportions preserved or min active |
| 10 | Panel Visibility Toggle Idempotence | toggle_panel twice returns panel to original visibility state. | Req 1.12 | Generate panel with random visibility; assert double-toggle is identity |

---

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 0,
      "label": "Foundation types",
      "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5"],
      "dependsOn": []
    },
    {
      "id": 1,
      "label": "Core data models",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8", "2.9", "2.10"],
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "Panel Registry",
      "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6"],
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "Layout Engine core lifecycle",
      "tasks": ["4.1", "4.2", "4.3", "4.4"],
      "dependsOn": [1, 2]
    },
    {
      "id": 4,
      "label": "Panel operations",
      "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8"],
      "dependsOn": [2, 3]
    },
    {
      "id": 5,
      "label": "Tab Group Manager",
      "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "6.10", "6.11"],
      "dependsOn": [1, 3]
    },
    {
      "id": 6,
      "label": "Floating Window Manager",
      "tasks": ["7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8", "7.9", "7.10", "7.11", "7.12", "7.13", "7.14", "7.15"],
      "dependsOn": [3, 4]
    },
    {
      "id": 7,
      "label": "Multi-Monitor Support",
      "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8"],
      "dependsOn": [6]
    },
    {
      "id": 8,
      "label": "Persona Manager",
      "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "9.9", "9.10", "9.11", "9.12"],
      "dependsOn": [3, 5]
    },
    {
      "id": 9,
      "label": "Layout Serialization",
      "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "10.9", "10.10", "10.11", "10.12", "10.13"],
      "dependsOn": [1, 3]
    },
    {
      "id": 10,
      "label": "Drag-and-Drop Coordinator",
      "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "11.8", "11.9", "11.10", "11.11", "11.12", "11.13", "11.14"],
      "dependsOn": [4, 5, 6]
    },
    {
      "id": 11,
      "label": "Splitter / Resize Manager",
      "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8", "12.9", "12.10", "12.11", "12.12"],
      "dependsOn": [1, 3, 5]
    },
    {
      "id": 12,
      "label": "Persona storage (TOML I/O)",
      "tasks": ["13.1", "13.2", "13.3", "13.4"],
      "dependsOn": [8, 9]
    },
    {
      "id": 13,
      "label": "Command registration",
      "tasks": ["14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "14.7", "14.8", "14.9"],
      "dependsOn": [4, 5, 6, 8]
    },
    {
      "id": 14,
      "label": "Visual feedback and indicators",
      "tasks": ["15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "15.7", "15.8"],
      "dependsOn": [4, 6, 8, 10]
    },
    {
      "id": 15,
      "label": "Integration wiring",
      "tasks": ["16.1", "16.2", "16.3", "16.4", "16.5", "16.6"],
      "dependsOn": [2, 9, 12, 13]
    },
    {
      "id": 16,
      "label": "End-to-end integration tests",
      "tasks": ["17.1", "17.2", "17.3", "17.4", "17.5", "17.6", "17.7"],
      "dependsOn": [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    }
  ]
}
```

---

## Notes

- This crate is GUI-independent — `egui` is referenced only in the `DockablePanel::render` trait signature. The shell layer (`ff-desktop`) is responsible for actual rendering.
- All layout mutations occur on the main thread for frame-coherent feedback. Only serialization I/O runs on Tokio workers.
- Property-based tests use `proptest` with minimum 100 iterations per property.
- Integration with peer crates (`ff-command`, `ff-plugin`, `ff-core`, `ff-config`) is deferred to task 16 to allow independent development of the layout engine core.
- Multi-monitor DPI handling (Req 4 criteria 4/5) requires runtime OS queries — unit tests will use mock `MonitorInfo` data.
- The `layouts/` directory for persona TOML files is relative to the workbench config root.

---

## Coverage Matrix

| Requirement | Criteria | Covered By Tasks |
|-------------|----------|-----------------|
| **Req 1: Panel System** | 1.1 Default layout init | 4.1 |
| | 1.2 Panel_Registry maintenance | 3.1 |
| | 1.3 Default zone assignment + invalid zone rejection | 3.1, 3.4 |
| | 1.4 panel_id method (1–64 chars) | 2.2, 3.2 |
| | 1.5 default_dock_zone method | 2.1, 2.2 |
| | 1.6 render method | 2.2 |
| | 1.7 title method | 2.2 |
| | 1.8 on_dock_state_changed method | 2.2, 2.3 |
| | 1.9 Trait-only interaction (no Layout_Engine changes) | 3.1 |
| | 1.10 Duplicate panel_id rejection | 3.3, 3.6 (Property 1) |
| | 1.11 Show/hide commands | 5.1, 5.2 |
| | 1.12 Toggle command | 5.3, 5.8 (Property 10) |
| | 1.13 Minimized/normal/maximized states | 2.3, 5.4, 5.5, 5.6 |
| | 1.14 Plugin load/unload panel lifecycle | 3.1, 16.2 |
| **Req 2: Tab Groups** | 2.1 Center zone subdivision | 6.1 |
| | 2.2 Split horizontal | 6.2, 6.10 (Property 3) |
| | 2.3 Split vertical | 6.3, 6.10 (Property 3) |
| | 2.4 Tab drag between groups | 6.4 |
| | 2.5 Empty group elimination | 6.4, 6.11 (Property 4) |
| | 2.6 Splitter between groups | 12.1 |
| | 2.7 Minimum 100px tab group size | 6.7 |
| | 2.8 Tab group arrangement serialization | 6.8 |
| | 2.9 New file added to active group | 6.5 |
| **Req 3: Floating Windows** | 3.1 Undock action | 7.2, 7.14 (Property 2) |
| | 3.2 Cascade offset positioning | 7.11 |
| | 3.3 Full interactivity in float | 7.12 |
| | 3.4 LayoutState update within 500ms | 7.2, 7.7 |
| | 3.5 Redock action | 7.4, 7.14 (Property 2) |
| | 3.6 Restore zone dimensions on redock | 7.4 |
| | 3.7 Stack as tab on redock if zone occupied | 7.4, 7.14 (Property 2) |
| | 3.8 OS close → redock | 7.8 |
| | 3.9 Drag-to-float (20px outside) | 7.3, 7.5, 11.10 |
| | 3.10 Full editing in floating tab | 7.12 |
| | 3.11 Close floating tab → redock to origin | 7.6, 7.8 |
| | 3.12 Unsaved changes save dialog | 7.8 |
| | 3.13 OS-level windows (taskbar, move, resize) | 7.12 |
| | 3.14 Max 16 floating windows | 7.9, 7.15 (Property 5) |
| | 3.15 OS window creation failure | 7.10 |
| | 3.16 Floating window state persistence | 2.6 |
| **Req 4: Multi-Monitor** | 4.1 Position on any monitor | 8.1 |
| | 4.2 Update monitor ID on move | 8.2 |
| | 4.3 Monitor ID in serialization | 8.3 |
| | 4.4 Per-monitor DPI rendering | 8.4 |
| | 4.5 DPI adjustment on monitor change | 8.4 |
| | 4.6 Monitor disconnect → relocate | 8.5 |
| | 4.7 Missing monitor at startup → reposition | 8.6 |
| | 4.8 50% visibility check | 8.7 |
| **Req 5: Personas** | 5.1 Persona configuration definition | 2.8 |
| | 5.2 Built-in personas | 9.1, 13.2 |
| | 5.3 Save custom persona | 9.4 |
| | 5.4 Activate persona within 500ms | 9.2 |
| | 5.5 Preserve open docs on switch | 9.3, 9.12 (Property 8) |
| | 5.6 Delete custom / protect built-in | 9.5 |
| | 5.7 TOML files in layouts/ directory | 13.1 |
| | 5.8 Skip unregistered panel_id | 9.9 |
| | 5.9 Active persona tracking + display | 9.10 |
| | 5.10 Modified indicator + update/revert | 9.6, 9.8 |
| **Req 6: Serialization** | 6.1 Save at exit | 10.2, 10.13 (Property 7) |
| | 6.2 Restore at startup | 10.3, 10.13 (Property 7) |
| | 6.3 Fallback on parse failure | 10.4 |
| | 6.4 LayoutState contents | 2.10, 10.1, 10.13 (Property 7) |
| | 6.5 Skip unregistered panel_id | 10.5 |
| | 6.6 Export layout | 10.6 |
| | 6.7 Import layout | 10.7 |
| | 6.8 Reset to default | 10.8 |
| | 6.9 State update within 500ms | 10.9 |
| | 6.10 I/O failure at exit | 10.10 |
| | 6.11 Schema version number | 10.11 |
| **Req 7: Drag-and-Drop** | 7.1 Drop indicator on valid zone | 11.3, 11.11 |
| | 7.2 Release over zone → dock | 11.4, 11.11 |
| | 7.3 Release outside → stay floating | 11.4 |
| | 7.4 Semi-transparent overlay indicator | 11.7 |
| | 7.5 16ms indicator appearance | 11.13 |
| | 7.6 Indicator disappears on leave | 11.3, 11.7 |
| | 7.7 Tab drag between groups | 11.4, 11.6 |
| | 7.8 Panel drag to different zone | 11.4 |
| | 7.9 Drag outside → float | 11.10 |
| | 7.10 Float title bar drag → dock | 11.11 |
| | 7.11 Tab tear-off at 30px | 11.8 |
| | 7.12 Tear-off cancel within 30px | 11.9 |
| | 7.13 Dock zone highlight (2px border) | 11.12 |
| **Req 8: Resizing** | 8.1 Zone/center splitter handle | 12.1 |
| | 8.2 Tab group splitter handle | 12.1 |
| | 8.3 Minimum size constraint enforcement | 12.3, 12.11 (Property 6) |
| | 8.4 Default minimum 48px | 12.4, 12.11 (Property 6) |
| | 8.5 Proportional resize on window resize | 12.5, 12.11 (Property 6), 12.12 (Property 9) |
| | 8.6 Priority: center area preserved first | 12.6 |
| | 8.7 Splitter positions in LayoutState | 12.7 |
| | 8.8 Double-click reset to default | 12.8 |
| | 8.9 Real-time visual feedback | 12.2, 12.9 |
| **Req 9: Keyboard Shortcuts** | 9.1 Ctrl+Shift+D toggle dock/float | 14.2 |
| | 9.2 No-op if no panel focused | 14.7 |
| | 9.3 Ctrl+Shift+T undock tab | 14.3 |
| | 9.4 No-op on only tab in only group | 14.8 |
| | 9.5 Ctrl+Shift+T in float → redock | 14.3 |
| | 9.6 Shortcuts registered with command framework | 14.1 |
| | 9.7 Persona switch via shortcut/palette | 14.5 |
| | 9.8 Split commands via shortcut | 14.4 |
| **Req 10: Visual Feedback** | 10.1 Placeholder indicator for floating panels | 15.1 |
| | 10.2 Hover tooltip (300ms) | 15.2 |
| | 10.3 Placeholder click → redock | 15.3 |
| | 10.4 Float window title format | 15.4 |
| | 10.5 Status bar persona display | 15.5 |
| | 10.6 Drop indicator placement precision | 15.6, 11.7 |
| | 10.7 Minimized panel icon in zone header | 15.7 |
