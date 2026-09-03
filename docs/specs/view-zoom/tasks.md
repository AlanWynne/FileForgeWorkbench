# Implementation Plan: View Zoom (`ff-zoom`)

## Overview

This plan covers the complete implementation of the `ff-zoom` crate — the view zoom subsystem for FileForgeWorkbench. It provides per-editor-instance integer point-offset zoom, keyboard shortcuts (Ctrl+=, Ctrl+-, Ctrl+0), Ctrl+Mouse Wheel zoom, the ZOOM primary command, configurable range limits, status bar indicator model, and session persistence.

This is a **Wave 9 (Desktop Integration)** sub-project. It depends on `ff-config` for zoom configuration keys, `ff-command` for ZOOM command registration and shortcut bindings, `ff-logging` for warning/diagnostic output, and integrates with `ff-multi-tab` for per-editor-instance state and `ff-session` for persistence.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-zoom/Cargo.toml` with dependencies (ff-config, ff-command, ff-logging, thiserror, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-zoom/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `level.rs`, `state.rs`, `operations.rs`, `config.rs`, `indicator.rs`, `persistence.rs`, `commands.rs`, `error.rs`
  - [x] 1.4 Add `ff-zoom` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. ZoomLevel type and validation
  - [x] 2.1 Define `ZoomOffset` newtype wrapping `i32` representing the signed point offset
  - [x] 2.2 Implement `ZoomOffset::new(value: i32, min: i32, max: i32) -> Self` constructor that clamps to [min, max] range
  - [x] 2.3 Implement `ZoomOffset::zero() -> Self` returning offset of 0
  - [x] 2.4 Implement `ZoomOffset::value(&self) -> i32` accessor
  - [x] 2.5 Implement `ZoomOffset::is_zero(&self) -> bool` predicate
  - [x] 2.6 Implement `ZoomOffset::effective_font_size(&self, base_size: u32) -> u32` computing `max(1, base_size as i32 + self.0) as u32`
  - [x] 2.7 Derive `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` on `ZoomOffset`
  - [x] 2.8 Write unit tests for construction, clamping, zero, effective size computation (including edge cases where base + offset < 1)
  - Covers: Requirement 1 (AC 1.1, 1.2, 1.5)

- [x] 3. Zoom configuration model
  - [x] 3.1 Define `ZoomConfig` struct with fields: `default_offset: i32`, `step: u32`, `min_offset: i32`, `max_offset: i32`
  - [x] 3.2 Implement `ZoomConfig::default()` returning `{ default_offset: 0, step: 1, min_offset: -10, max_offset: 60 }`
  - [x] 3.3 Implement `ZoomConfig::validate(&mut self) -> Vec<ConfigWarning>` applying validation rules: step clamped to 1–10, min_offset clamped to -20..=0, max_offset clamped to 0..=100, min >= max resets both to defaults, default_offset clamped to [min, max]
  - [x] 3.4 Implement `ZoomConfig::from_config_store(store: &ConfigStore) -> Self` reading `[view.zoom]` table keys and calling validate
  - [x] 3.5 Implement hot-reload support: `ZoomConfig::on_config_changed(new_store: &ConfigStore) -> (Self, Vec<ConfigWarning>)` reloading and re-validating
  - [x] 3.6 Write unit tests for defaults, each validation rule (step out of range, min >= max, default out of bounds, invalid types), hot-reload
  - Covers: Requirement 4 (AC 4.1–4.6)

- [x] 4. Zoom state management (per-editor-instance)
  - [x] 4.1 Define `ZoomState` struct with fields: `offset: ZoomOffset`, `config: ZoomConfig`
  - [x] 4.2 Implement `ZoomState::new(config: &ZoomConfig) -> Self` initialising offset to config.default_offset (clamped)
  - [x] 4.3 Implement `ZoomState::from_persisted(offset: i32, config: &ZoomConfig) -> Self` restoring persisted offset (clamped to current config range)
  - [x] 4.4 Implement `ZoomState::offset(&self) -> ZoomOffset` accessor
  - [x] 4.5 Implement `ZoomState::effective_font_size(&self, base_size: u32) -> u32` delegating to `ZoomOffset::effective_font_size`
  - [x] 4.6 Implement `ZoomState::apply_config_change(&mut self, new_config: &ZoomConfig)` clamping current offset to new range
  - [x] 4.7 Write unit tests for construction, persisted restoration with clamping, config change clamping
  - Covers: Requirement 1 (AC 1.1, 1.4, 1.5), Requirement 5 (AC 5.1, 5.2)

- [x] 5. Zoom in/out/reset operations
  - [x] 5.1 Implement `ZoomState::zoom_in(&mut self) -> ZoomResult` incrementing offset by config.step, clamped at max; returns `ZoomResult::Applied { new_offset }` or `ZoomResult::AtLimit { limit, message }`
  - [x] 5.2 Implement `ZoomState::zoom_out(&mut self) -> ZoomResult` decrementing offset by config.step, clamped at min; returns `ZoomResult::Applied { new_offset }` or `ZoomResult::AtLimit { limit, message }`
  - [x] 5.3 Implement `ZoomState::zoom_reset(&mut self) -> ZoomResult` setting offset to zero; returns `ZoomResult::Applied { new_offset: 0 }`
  - [x] 5.4 Implement `ZoomState::set_offset(&mut self, value: i32) -> ZoomResult` setting absolute offset (clamped)
  - [x] 5.5 Define `ZoomResult` enum with variants: `Applied { new_offset: i32 }`, `AtLimit { limit: i32, message: String }`
  - [x] 5.6 Implement limit messages: "Maximum zoom reached (+{max})" and "Minimum zoom reached ({min})"
  - [x] 5.7 Write unit tests for zoom_in (normal, at limit), zoom_out (normal, at limit), zoom_reset, set_offset (within range, above max, below min), step sizes > 1
  - Covers: Requirement 1 (AC 1.5), Requirement 2 (AC 2.1–2.3, 2.6, 2.7), Requirement 3 (AC 3.1, 3.2), Requirement 8 (AC 8.2–8.5)

- [x] 6. Font scaling coordination
  - [x] 6.1 Define `ZoomFontMetrics` struct with fields: `base_font_size: u32`, `effective_font_size: u32`, `zoom_offset: i32`
  - [x] 6.2 Implement `ZoomFontMetrics::compute(base_size: u32, state: &ZoomState) -> Self` calculating effective size
  - [x] 6.3 Implement `ZoomFontMetrics::visible_lines_changed(&self, viewport_height_px: f32, line_height_px: f32) -> u32` computing visible_count from effective metrics
  - [x] 6.4 Define `ZoomChangeEvent` struct carrying `editor_instance_id`, `old_offset`, `new_offset`, `effective_font_size`, `requires_relayout: bool`
  - [x] 6.5 Implement `ZoomChangeEvent::from_state_change(id, old, new, base_size) -> Self` constructor
  - [x] 6.6 Ensure `requires_relayout` is true when offset actually changed, false when clamped to same value
  - [x] 6.7 Write unit tests for metrics computation, visible line count changes (larger offset → fewer lines, smaller offset → more lines), event construction
  - Covers: Requirement 1 (AC 1.2, 1.6, 1.8), Requirement 9 (AC 9.1, 9.4)

- [x] 7. Status bar zoom indicator model
  - [x] 7.1 Define `ZoomIndicatorState` enum with variants: `Hidden` (offset is zero), `Visible { text: String, offset: i32 }`
  - [x] 7.2 Implement `ZoomIndicatorState::from_offset(offset: ZoomOffset) -> Self` returning Hidden when zero, Visible with formatted text otherwise
  - [x] 7.3 Implement indicator text formatting: `"Zoom: +N"` for positive, `"Zoom: -N"` for negative
  - [x] 7.4 Define `ZoomQuickPickOption` struct with fields: `label: String`, `offset: i32`
  - [x] 7.5 Implement `ZoomQuickPickOption::defaults() -> Vec<Self>` returning common offsets: -5, -2, 0, +2, +5, +10 plus "Reset to 0" action
  - [x] 7.6 Write unit tests for indicator hidden at zero, visible formatting, quick-pick options
  - Covers: Requirement 7 (AC 7.1–7.5)

- [x] 8. Session persistence model
  - [x] 8.1 Define `ZoomSessionEntry` struct with fields: `resource_uri: String`, `zoom_offset: i32`
  - [x] 8.2 Implement `ZoomSessionEntry::from_state(uri: &str, state: &ZoomState) -> Self` capturing current offset
  - [x] 8.3 Implement `ZoomSessionEntry::restore(config: &ZoomConfig) -> ZoomState` creating state from persisted offset (clamped to current config range)
  - [x] 8.4 Implement serialization/deserialization for `ZoomSessionEntry` (serde-compatible format for session store integration)
  - [x] 8.5 Implement batch operations: `persist_all(entries: &[ZoomSessionEntry]) -> Vec<u8>` and `restore_all(data: &[u8]) -> Vec<ZoomSessionEntry>`
  - [x] 8.6 Write unit tests for persistence round-trip, clamping on restore when config has changed, missing entry defaults to default_offset
  - Covers: Requirement 6 (AC 6.1–6.4)

- [x] 9. Keyboard shortcut handling
  - [x] 9.1 Define `ZoomShortcutAction` enum with variants: `ZoomIn`, `ZoomOut`, `Reset`
  - [x] 9.2 Implement `ZoomShortcutAction::from_key_event(key, modifiers) -> Option<Self>` mapping Ctrl+= → ZoomIn, Ctrl+- → ZoomOut, Ctrl+0 → Reset
  - [x] 9.3 Implement focus-gate logic: return None when editor does not have focus (non-editor panel has focus)
  - [x] 9.4 Implement shortcut execution: dispatch to `ZoomState::zoom_in/zoom_out/zoom_reset` and produce `ZoomChangeEvent`
  - [x] 9.5 Write unit tests for shortcut recognition, focus gating, mapping correctness, limit message production
  - Covers: Requirement 2 (AC 2.1–2.7)

- [x] 10. Ctrl+Mouse wheel handling
  - [x] 10.1 Define `ZoomScrollAction` struct with fields: `direction: ScrollDirection`, `editor_instance_id: Option<EditorInstanceId>`
  - [x] 10.2 Implement `ZoomScrollAction::from_scroll_event(scroll_delta, modifiers, mouse_position, editor_bounds) -> Option<Self>` detecting Ctrl+Scroll over editor
  - [x] 10.3 Implement rule: return None when Ctrl is not held (normal scroll passthrough)
  - [x] 10.4 Implement rule: return None when mouse cursor is not over any editor instance
  - [x] 10.5 Implement rule: apply zoom to the editor instance under cursor regardless of keyboard focus
  - [x] 10.6 Implement per-event processing: no debouncing — each scroll step triggers one zoom step
  - [x] 10.7 Write unit tests for Ctrl+Scroll detection, direction mapping, non-Ctrl passthrough, out-of-bounds rejection, per-event stepping
  - Covers: Requirement 3 (AC 3.1–3.5)

- [x] 11. ZOOM primary command implementation
  - [x] 11.1 Implement `ZoomCommand` struct implementing the command handler trait
  - [x] 11.2 Implement argument parsing: `ZOOM` (no args) → display current state, `ZOOM n` → set offset, `ZOOM IN` → increment, `ZOOM OUT` → decrement, `ZOOM RESET` → zero
  - [x] 11.3 Implement `ZOOM n` with signed integer parsing (supports `+3`, `-2`, `0`, `10`)
  - [x] 11.4 Implement `ZOOM` (no args) status display: format "Zoom offset: +N (effective size: Mpt)" message
  - [x] 11.5 Implement mode validation: ZOOM valid in Browse, Edit, View, and all special modes
  - [x] 11.6 Implement non-history rule: ZOOM command is not added to command history
  - [x] 11.7 Implement non-undoable rule: ZOOM does not produce an UndoRecord
  - [x] 11.8 Write unit tests for each argument form, signed integer parsing, mode validation, non-history, non-undoable guarantees
  - Covers: Requirement 8 (AC 8.1–8.9)

- [x] 12. Command registration and shortcut bindings
  - [x] 12.1 Register `"view.zoom"` command with Command_ID in command-framework registry
  - [x] 12.2 Register `"view.zoom_in"` command with reserved shortcut Ctrl+=
  - [x] 12.3 Register `"view.zoom_out"` command with reserved shortcut Ctrl+-
  - [x] 12.4 Register `"view.zoom_reset"` command with reserved shortcut Ctrl+0
  - [x] 12.5 Ensure Ctrl+=, Ctrl+-, Ctrl+0 are marked as reserved (non-reassignable by user config or plugins)
  - [x] 12.6 Ensure ZOOM command handler delegates to `ZoomState` operations on active editor instance
  - [x] 12.7 Write integration tests verifying command dispatch triggers correct zoom operations
  - Covers: Requirement 2 (AC 2.5), Requirement 8 (AC 8.1)

- [x] 13. Error types
  - [x] 13.1 Define `ZoomError` enum with variants: `InvalidArgument { detail: String }`, `NoActiveEditor`, `ConfigWarning { key: String, detail: String }`
  - [x] 13.2 Implement `Display` for all variants
  - [x] 13.3 Write unit tests for error formatting
  - Covers: Error handling across all requirements

- [x] 14. Per-editor-instance independence validation
  - [x] 14.1 Implement integration-level test: two editor instances with different zoom offsets — changing one does not affect the other
  - [x] 14.2 Implement integration-level test: tab switch updates indicator to reflect new active instance's offset
  - [x] 14.3 Implement integration-level test: new editor instance initialises at default_offset from config
  - [x] 14.4 Implement integration-level test: split view creates independent zoom state from source instance
  - [x] 14.5 Write unit tests for independence invariant using multiple ZoomState instances
  - Covers: Requirement 5 (AC 5.1–5.5)

- [x] 15. DPI interaction model
  - [x] 15.1 Implement `ZoomDpiModel` documentation-struct confirming: offset is typographical points, not pixels; DPI conversion is delegated to the rendering engine
  - [x] 15.2 Implement unit test asserting: `ZoomOffset` value is preserved unchanged across simulated DPI context switches
  - [x] 15.3 Implement unit test asserting: `effective_font_size` in points is constant regardless of DPI scale factor passed to rendering
  - [x] 15.4 Document in `ZoomState` API: "Zoom offset is applied as point-size delta; physical pixel rendering is handled by the platform rendering layer"
  - Covers: Requirement 9 (AC 9.1–9.4)

- [x] 16. Property-based tests — ZoomOffset invariants
  - [x] 16.1 Write property test: for any i32 value and any valid [min, max] range, `ZoomOffset::new` always produces an offset within [min, max] inclusive
    - **Validates: Requirements 1.5, 4.1**
  - [x] 16.2 Write property test: `effective_font_size` is always >= 1 for any base_size >= 1 and any ZoomOffset value (including extreme negatives)
    - **Validates: Requirement 1.2**
  - [x] 16.3 Write property test: `ZoomOffset::zero().is_zero()` is always true and `ZoomOffset::new(n, min, max).is_zero()` is true if and only if the clamped value is 0
    - **Validates: Requirement 1.4**
  - [x] 16.4 Write property test: for any base_size and two offsets a < b, `effective_font_size(base, a) <= effective_font_size(base, b)` — effective size is monotonically non-decreasing with offset
    - **Validates: Requirements 1.2, 1.8**

- [x] 17. Property-based tests — zoom operation invariants
  - [x] 17.1 Write property test: after any sequence of zoom_in calls, offset never exceeds max_offset for any config with valid step and range
    - **Validates: Requirements 1.5, 2.1, 2.6**
  - [x] 17.2 Write property test: after any sequence of zoom_out calls, offset never goes below min_offset for any config with valid step and range
    - **Validates: Requirements 1.5, 2.2, 2.7**
  - [x] 17.3 Write property test: zoom_reset always produces offset 0 regardless of prior state for any initial offset value
    - **Validates: Requirements 2.3, 8.5**
  - [x] 17.4 Write property test: set_offset(n) followed by offset() returns clamped(n, min, max) for any integer n and any valid config range
    - **Validates: Requirements 1.5, 8.2**
  - [x] 17.5 Write property test: zoom_in followed by zoom_out (with step=1) returns to original offset when not at either limit, for any starting offset strictly within (min, max)
    - **Validates: Requirements 2.1, 2.2**

- [x] 18. Property-based tests — configuration validation invariants
  - [x] 18.1 Write property test: after validate(), step is always within [1, 10] for any input step value
    - **Validates: Requirement 4.4**
  - [x] 18.2 Write property test: after validate(), min_offset < max_offset always holds for any input combination
    - **Validates: Requirement 4.2**
  - [x] 18.3 Write property test: after validate(), default_offset is always within [min_offset, max_offset] for any input combination
    - **Validates: Requirement 4.3**
  - [x] 18.4 Write property test: hot-reload with any new config values always results in all active offsets within the new [min, max] range
    - **Validates: Requirement 4.6**

- [x] 19. Property-based tests — session persistence invariants
  - [x] 19.1 Write property test: persist then restore round-trip preserves offset exactly when offset is within current config range, for any valid offset and config
    - **Validates: Requirements 6.1, 6.2**
  - [x] 19.2 Write property test: restoring a persisted offset outside current config range clamps to nearest bound (never produces out-of-range state) for any persisted value and any valid config
    - **Validates: Requirement 6.3**
  - [x] 19.3 Write property test: restoring with no persisted entry uses default_offset from config for any valid config
    - **Validates: Requirement 6.2**

- [x] 20. Property-based tests — indicator model invariants
  - [x] 20.1 Write property test: indicator is Hidden if and only if offset is zero, for any ZoomOffset value
    - **Validates: Requirements 7.1, 7.2**
  - [x] 20.2 Write property test: indicator text always contains the absolute offset value with correct sign prefix for any non-zero offset
    - **Validates: Requirement 7.5**
  - [x] 20.3 Write property test: indicator text matches regex `^Zoom: [+-]\d+$` for any non-zero offset value
    - **Validates: Requirement 7.5**

- [x] 21. Integration tests — end-to-end zoom workflows
  - [x] 21.1 Write integration test: create editor instance → zoom in 3 times → verify offset is +3 and effective size is base+3
  - [x] 21.2 Write integration test: zoom in to max → attempt one more zoom in → verify AtLimit result with correct message
  - [x] 21.3 Write integration test: zoom out to min → attempt one more zoom out → verify AtLimit result with correct message
  - [x] 21.4 Write integration test: zoom in 5 times → reset → verify offset is 0 and indicator is Hidden
  - [x] 21.5 Write integration test: set offset via ZOOM command (`ZOOM 7`) → verify offset is 7 and indicator shows "Zoom: +7"
  - [x] 21.6 Write integration test: set offset via ZOOM command with negative value (`ZOOM -3`) → verify offset is -3 and indicator shows "Zoom: -3"
  - [x] 21.7 Write integration test: ZOOM with no args → verify status message contains current offset and effective size
  - [x] 21.8 Write integration test: two editor instances at different offsets → verify indicator updates on tab switch
  - [x] 21.9 Write integration test: persist two editor zoom states → simulate restart → restore → verify offsets match
  - [x] 21.10 Write integration test: persist offset +50 → change config max to +30 → restore → verify offset clamped to +30
  - [x] 21.11 Write integration test: config step=3 → zoom in once → verify offset increases by 3
  - [x] 21.12 Write integration test: Ctrl+Scroll up 4 times with step=1 → verify offset is +4
  - [x] 21.13 Write integration test: Ctrl+Scroll without Ctrl held → verify offset unchanged (normal scroll passthrough)
  - [x] 21.14 Write integration test: ZOOM command not recorded in command history
  - [x] 21.15 Write integration test: hot-reload config with narrower range → verify active instances clamped
  - Covers: End-to-end validation of Requirements 1–9

---

## Notes

- The `ff-zoom` crate has zero GUI dependencies — it operates on abstract types and emits `ZoomChangeEvent` structs that the rendering layer consumes to trigger re-layout.
- `ZoomOffset` is a display-only state change: it never modifies document content, never produces UndoRecords, and is never recorded in command history.
- The `ZoomFontMetrics` struct provides the data bridge between zoom state and viewport/rendering. The viewport crate (`ff-viewport`) consumes the `effective_font_size` to recalculate `visible_count` and `top_line` adjustments.
- Per-editor-instance independence (Requirement 5) is architecturally enforced by each `ZoomState` being owned by its editor instance. The zoom crate does not maintain a global zoom registry.
- DPI handling (Requirement 9) is entirely passive from the zoom crate's perspective — the offset is expressed in typographical points, and the rendering engine handles DPI-to-pixel conversion.
- Session persistence (Requirement 6) integrates with the session store owned by `ff-session`. The zoom crate provides `ZoomSessionEntry` as the serializable payload.
- Keyboard shortcuts (Ctrl+=, Ctrl+-, Ctrl+0) are registered as **reserved** in the shortcut registry and cannot be remapped by users or plugins.
- The status bar indicator model (Task 7) produces data for rendering but does not depend on any GUI framework — the actual rendering is handled by `ff-statusbar`.
- Property-based tests (Tasks 16–20) use the `proptest` crate and are configured for a minimum of 256 iterations.
- The ZOOM primary command (Task 11) supports signed integer arguments and keyword forms (IN, OUT, RESET). It does not interact with the undo system.

---

## Acceptance Criteria Coverage Map

| Task | Requirements Covered |
|------|---------------------|
| 1 | Structural scaffolding (all) |
| 2 | Req 1 (AC 1.1, 1.2, 1.5) |
| 3 | Req 4 (AC 4.1–4.6) |
| 4 | Req 1 (AC 1.1, 1.4, 1.5), Req 5 (AC 5.1, 5.2) |
| 5 | Req 1 (AC 1.5), Req 2 (AC 2.1–2.3, 2.6, 2.7), Req 3 (AC 3.1, 3.2), Req 8 (AC 8.2–8.5) |
| 6 | Req 1 (AC 1.2, 1.6, 1.8), Req 9 (AC 9.1, 9.4) |
| 7 | Req 7 (AC 7.1–7.5) |
| 8 | Req 6 (AC 6.1–6.4) |
| 9 | Req 2 (AC 2.1–2.7) |
| 10 | Req 3 (AC 3.1–3.5) |
| 11 | Req 8 (AC 8.1–8.9) |
| 12 | Req 2 (AC 2.5), Req 8 (AC 8.1) |
| 13 | Error handling (all) |
| 14 | Req 5 (AC 5.1–5.5) |
| 15 | Req 9 (AC 9.1–9.4) |
| 16 | PBT: Req 1 (ZoomOffset invariants) |
| 17 | PBT: Req 1, 2, 8 (zoom operation invariants) |
| 18 | PBT: Req 4 (configuration validation invariants) |
| 19 | PBT: Req 6 (session persistence invariants) |
| 20 | PBT: Req 7 (indicator model invariants) |
| 21 | Integration: Req 1–9 (end-to-end workflows) |

---

## Task Dependency Graph

```json
{
  "taskDependencies": {
    "1": [],
    "2": ["1"],
    "3": ["1"],
    "4": ["2", "3"],
    "5": ["2", "4"],
    "6": ["2", "4"],
    "7": ["2"],
    "8": ["4"],
    "9": ["5"],
    "10": ["5"],
    "11": ["5", "13"],
    "12": ["9", "10", "11"],
    "13": ["1"],
    "14": ["4", "5", "7"],
    "15": ["2", "6"],
    "16": ["2"],
    "17": ["4", "5"],
    "18": ["3"],
    "19": ["4", "8"],
    "20": ["7"],
    "21": ["5", "6", "7", "8", "9", "10", "11", "12", "14", "15"]
  },
  "externalDependencies": {
    "ff-config": "Provides ConfigStore, key-value configuration access, hot-reload notification — zoom config keys are read from here",
    "ff-command": "Command registry, dispatch, metadata, Shortcut_Registry — ZOOM command and reserved shortcuts are registered here",
    "ff-logging": "Structured logging for configuration warnings and diagnostics",
    "ff-multi-tab": "Per-editor-instance lifecycle — each tab owns its own ZoomState",
    "ff-session": "Session persistence store — zoom offsets are serialised/deserialised through this system",
    "ff-viewport": "Consumes ZoomChangeEvent to recalculate visible_count and adjust top_line",
    "ff-theme": "Provides Base_Font_Size (monospace editor font point size) from theme configuration",
    "ff-statusbar": "Renders ZoomIndicatorState data in the status bar UI"
  },
  "waves": [
    {
      "id": 0,
      "label": "Foundation types and configuration",
      "tasks": ["1", "2", "3", "13"],
      "description": "Crate scaffolding, ZoomOffset type, ZoomConfig model, error types"
    },
    {
      "id": 1,
      "label": "State management and operations",
      "tasks": ["4", "5", "6"],
      "description": "ZoomState per-instance struct, zoom in/out/reset operations, font scaling coordination",
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "UI models and persistence",
      "tasks": ["7", "8"],
      "description": "Status bar indicator model, session persistence entry format",
      "dependsOn": [0, 1]
    },
    {
      "id": 3,
      "label": "Input handling and commands",
      "tasks": ["9", "10", "11", "12"],
      "description": "Keyboard shortcuts, Ctrl+Scroll, ZOOM primary command, command registration",
      "dependsOn": [1]
    },
    {
      "id": 4,
      "label": "Integration validation",
      "tasks": ["14", "15"],
      "description": "Per-editor-instance independence, DPI interaction model",
      "dependsOn": [1, 2]
    },
    {
      "id": 5,
      "label": "Property-based tests",
      "tasks": ["16", "17", "18", "19", "20"],
      "description": "Property tests validating invariants across offset, operations, config, persistence, and indicator",
      "dependsOn": [0, 1, 2]
    },
    {
      "id": 6,
      "label": "Integration tests",
      "tasks": ["21"],
      "description": "End-to-end workflow validation covering all requirements",
      "dependsOn": [0, 1, 2, 3, 4, 5]
    }
  ]
}
```
