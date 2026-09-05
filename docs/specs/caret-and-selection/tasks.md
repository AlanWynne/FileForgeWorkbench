# Implementation Plan: Caret & Selection (`ff-caret-selection`)

## Overview

This plan covers the complete implementation of the `ff-caret-selection` crate — the visual presentation layer for carets, selections, caret-line highlighting, virtual space display, and modified line markers within FileForgeWorkbench. This crate consumes the logical selection model from `ff-edit-operations` (SelectionPosition, SelectionRange, SelectionContainer) and translates it into rendering instructions that the GUI shell executes.

This is a **Wave 6 (UI and Rendering)** sub-project. It depends on:
- `ff-edit-operations` (Wave 4) — logical selection model, edit mode state, modified line flags
- `ff-theme` (Wave 6 peer) — element colours, style slots, visual mode integration
- `ff-configuration-system` (Wave 2) — configuration loading, hot-reload notifications
- `ff-display-line-mapping` (Wave 4) — wrapped sub-line information for sub-line caret highlight

It is consumed by the GUI shell layer (`ff-desktop`) for actual rendering.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-caret-selection/Cargo.toml` with dependencies (serde, thiserror, proptest dev-dep) and deps on `ff-edit-operations`, `ff-theme`, `ff-configuration-system`, `ff-logging`
  - [x] 1.2 Create `crates/ff-caret-selection/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `caret.rs`, `caret_style.rs`, `caret_line.rs`, `blink.rs`, `selection_display.rs`, `selection_colours.rs`, `virtual_space.rs`, `rectangular.rs`, `multi_caret.rs`, `modified_marker.rs`, `config.rs`, `error.rs`
  - [x] 1.4 Add `ff-caret-selection` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Caret style and shape types
  - [x] 2.1 Define `CaretStyle` enum: Invisible, Line, Block with serde support and Default impl (Line)
  - [x] 2.2 Define `CaretWidth` newtype wrapping u8 with validation constructor clamping to [1, 20], default 1
  - [x] 2.3 Define `CaretShape` struct composing style, width, and overstrike-override flag
  - [x] 2.4 Implement `CaretShape::effective_style(&self, edit_mode: EditMode) -> CaretStyle` returning Block when overstrike mode, else configured style
  - [x] 2.5 Implement `CaretShape::effective_width(&self) -> u8` returning clamped width for Line style
  - [x] 2.6 Write unit tests for style defaults, width clamping (0→1, 25→20, valid pass-through), overstrike override
  - Covers: Requirement 1 (AC 1.1–1.10)

- [x] 3. Caret colour model
  - [x] 3.1 Define `CaretColours` struct with `primary` (element: Caret) and `additional` (element: CaretAdditional) colour fields using `ColourRGBA` from ff-theme
  - [x] 3.2 Implement default colours: primary=#000000, additional=#7F7F7F
  - [x] 3.3 Implement `CaretColours::colour_for(&self, is_primary: bool) -> ColourRGBA` accessor
  - [x] 3.4 Implement block-caret text inversion logic: method `inverse_text_colour(caret_colour: ColourRGBA) -> ColourRGBA`
  - [x] 3.5 Write unit tests for default colours, colour-for selection, block text inversion
  - Covers: Requirement 2 (AC 2.1–2.7)

- [x] 4. Caret blink model
  - [x] 4.1 Define `BlinkState` struct with `period_ms: u32` and `last_reset_timestamp_ms: u64` fields
  - [x] 4.2 Implement `BlinkState::new(period_ms: u32) -> Self` with default period 530ms
  - [x] 4.3 Implement `BlinkState::is_visible(&self, current_time_ms: u64) -> bool` computing visibility from elapsed time modulo period (visible in first half of cycle)
  - [x] 4.4 Implement `BlinkState::reset(&mut self, current_time_ms: u64)` to restart cycle at visible phase
  - [x] 4.5 Implement `BlinkState::set_period(&mut self, period_ms: u32)` for runtime configuration changes
  - [x] 4.6 Implement period=0 handling: `is_visible` always returns true when period is 0
  - [x] 4.7 Write unit tests for blink visibility timing, reset behaviour, period=0 always visible, default period
  - Covers: Requirement 3 (AC 3.1–3.7)

- [x] 5. Caret line highlight model
  - [x] 5.1 Define `CaretLineMode` enum: None, Frame, Fill with serde support, Default=Frame
  - [x] 5.2 Define `CaretLineLayer` enum: Base, OverText with serde support, Default=Base
  - [x] 5.3 Define `CaretLineConfig` struct with mode, frame_width, layer, always_show (bool, default false), sub_line (bool, default false), colour (CaretLineBack element)
  - [x] 5.4 Implement frame width clamping: [1, line_height/3] via `CaretLineConfig::effective_frame_width(&self, line_height: u32) -> u32`
  - [x] 5.5 Implement `CaretLineConfig::should_show(&self, pane_focused: bool) -> bool` respecting always_show flag
  - [x] 5.6 Implement `CaretLineConfig::applies_to_subline(&self) -> bool` returning sub_line flag state
  - [x] 5.7 Write unit tests for mode defaults, frame width clamping, focus-dependent visibility, sub-line flag
  - Covers: Requirement 4 (AC 4.1–4.13)

- [x] 6. Selection display model
  - [x] 6.1 Define `SelectionLayer` enum: Base, OverText with serde support, Default=Base
  - [x] 6.2 Define `SelectionDisplayConfig` struct with visible (bool, default true), layer, eol_filled (bool, default false)
  - [x] 6.3 Implement `SelectionDisplayConfig::is_visible(&self) -> bool` accessor
  - [x] 6.4 Implement `SelectionDisplayConfig::extends_to_eol(&self) -> bool` accessor
  - [x] 6.5 Implement `SelectionDisplayConfig::is_translucent(&self) -> bool` returning true when layer is OverText
  - [x] 6.6 Write unit tests for default visibility, EOL fill flag, layer translucent check
  - Covers: Requirement 5 (AC 5.1–5.10)

- [x] 7. Selection element colours
  - [x] 7.1 Define `SelectionColourSet` struct with text/back pairs for primary, additional, secondary, and inactive selections
  - [x] 7.2 Implement default colours: SelectionBack=#C0C0C0 opaque, SelectionAdditionalBack=#D7D7D7 opaque, SelectionSecondaryBack=#B0B0B0 opaque, SelectionInactiveBack=#808080 alpha=0x3F
  - [x] 7.3 Implement `SelectionColourSet::colours_for_context(&self, context: SelectionContext) -> (Option<ColourRGBA>, ColourRGBA)` returning (text_override, background) pair
  - [x] 7.4 Define `SelectionContext` enum: Primary, Additional, Secondary, Inactive
  - [x] 7.5 Implement text colour override semantics: return None when no SelectionText is configured (retain syntax colours)
  - [x] 7.6 Implement alpha support: all colour values support translucent alpha channels
  - [x] 7.7 Write unit tests for default colours, context-based resolution, None text override, alpha preservation
  - Covers: Requirement 6 (AC 6.1–6.10)

- [x] 8. Virtual space display logic
  - [x] 8.1 Define `VirtualSpaceRenderer` struct (stateless helper) with methods for virtual space position calculation
  - [x] 8.2 Implement `horizontal_offset(&self, line_end_x: f32, virtual_space: u64, space_width: f32) -> f32` computing caret X position in virtual space
  - [x] 8.3 Implement `selection_rect_in_virtual_space(&self, line_end_x: f32, vs_start: u64, vs_end: u64, space_width: f32, line_height: f32) -> Rect` computing highlight region for virtual space selection
  - [x] 8.4 Implement rule: no whitespace indicators rendered in virtual space region
  - [x] 8.5 Implement rule: caret in virtual space uses same style/width/colour as in real text
  - [x] 8.6 Write unit tests for offset calculation, selection rect computation, zero virtual space pass-through
  - Covers: Requirement 7 (AC 7.1–7.6)

- [x] 9. Rectangular selection display
  - [x] 9.1 Define `RectangularSelectionDisplay` struct with methods for column-band rendering computation
  - [x] 9.2 Implement `column_band_for_line(&self, left_col: u64, right_col: u64, line_content_len: u64, space_width: f32) -> (f32, f32)` computing pixel extents including virtual space
  - [x] 9.3 Implement thin-selection rendering: return zero-width rect (thin line) at column position for thin selection type
  - [x] 9.4 Implement per-line caret placement at the caret-column edge of rectangular selection
  - [x] 9.5 Write unit tests for column band calculation, virtual space extension, thin selection line, caret placement
  - Covers: Requirement 8 (AC 8.1–8.5)

- [x] 10. Multi-caret display coordination
  - [x] 10.1 Define `MultiCaretDisplay` struct wrapping selection container reference and caret configuration
  - [x] 10.2 Implement `caret_render_list(&self) -> Vec<CaretRenderInfo>` producing ordered list of all caret positions with primary/additional colour assignment
  - [x] 10.3 Define `CaretRenderInfo` struct: position (line, column, virtual_space), is_primary (bool), colour, style, width
  - [x] 10.4 Implement `selection_render_list(&self) -> Vec<SelectionRenderInfo>` producing list of selection ranges with appropriate colour context (Primary for main, Additional for others)
  - [x] 10.5 Define `SelectionRenderInfo` struct: start, end, colour_context (SelectionContext), is_rectangular (bool)
  - [x] 10.6 Implement uniform blink rule: all carets share the same blink phase (all visible or all hidden simultaneously)
  - [x] 10.7 Write unit tests for single caret (primary), multi-caret colour assignment, selection list context, uniform blink
  - Covers: Requirement 9 (AC 9.1–9.6)

- [x] 11. Modified line marker rendering
  - [x] 11.1 Define `ModifiedMarkerConfig` struct with marker_char (default '*'), colour (from theme), prefix_position
  - [x] 11.2 Implement `ModifiedMarkerConfig::should_render(&self, line: u64, tracker: &ModifiedLineTracker) -> bool` checking logical state
  - [x] 11.3 Implement `ModifiedMarkerConfig::render_char(&self) -> char` returning the marker character
  - [x] 11.4 Implement marker position: fixed within prefix area, not shifting with line number width changes
  - [x] 11.5 Implement marker visibility rule: marker not obscured by caret-line highlight (drawn after/above caret-line background)
  - [x] 11.6 Write unit tests for marker visibility check, position stability, draw-order over caret-line
  - Covers: Requirement 10 (AC 10.1–10.5)

- [x] 12. Configuration and theme integration
  - [x] 12.1 Define `CaretSelectionConfig` aggregate struct composing CaretShape, CaretColours, BlinkState, CaretLineConfig, SelectionDisplayConfig, SelectionColourSet, ModifiedMarkerConfig
  - [x] 12.2 Implement `CaretSelectionConfig::from_theme(theme: &ThemeApi) -> Self` loading all settings from the theme system with defaults for missing values
  - [x] 12.3 Implement `CaretSelectionConfig::apply_theme_update(&mut self, theme: &ThemeApi)` for hot-reload: re-read all settings from theme on next frame
  - [x] 12.4 Implement GUI-independence: CaretSelectionConfig stores pure data with no rendering framework types
  - [x] 12.5 Implement programmatic change support: all fields have public setters that take effect on next render frame
  - [x] 12.6 Write unit tests for config construction from defaults, theme application, hot-reload update
  - Covers: Requirement 11 (AC 11.1–11.5)

- [x] 13. Keyboard focus integration
  - [x] 13.1 Implement `FocusState` struct tracking pane focus and caret visibility state
  - [x] 13.2 Implement `FocusState::on_focus_gained(&mut self, blink: &mut BlinkState, current_time_ms: u64)` resetting blink cycle to visible
  - [x] 13.3 Implement `FocusState::on_caret_moved(&mut self, blink: &mut BlinkState, current_time_ms: u64)` resetting blink cycle to visible
  - [x] 13.4 Implement `FocusState::is_caret_visible(&self) -> bool` returning true when pane focused and caret within viewport
  - [x] 13.5 Write unit tests for focus gain blink reset, caret move blink reset, visibility when unfocused
  - Covers: Requirement 12 (AC 12.1–12.3)

- [x] 14. Error handling
  - [x] 14.1 Define `CaretSelectionError` enum: InvalidCaretWidth, InvalidFrameWidth, ConfigurationError
  - [x] 14.2 Implement error message formatting per `[caret-selection] operation: description` standard (≤200 chars)
  - [x] 14.3 Implement graceful degradation: invalid configuration values revert to defaults with WARN log
  - [x] 14.4 Write unit tests for all error variants and message formatting
  - Covers: Cross-cutting Requirement 8 (Error Message Standards)

- [x] 15. Property-based tests
  - [x] 15.1 Write PBT: caret width clamping correctness
  - [x] 15.2 Write PBT: blink visibility timing correctness
  - [x] 15.3 Write PBT: caret-line frame width clamping correctness
  - [x] 15.4 Write PBT: virtual space horizontal offset calculation correctness
  - [x] 15.5 Write PBT: selection colour context resolution correctness
  - [x] 15.6 Write PBT: overstrike mode forces Block caret style
  - [x] 15.7 Write PBT: configuration round-trip from theme and back
  - Covers: Requirements 1–9, 11 (see Property-Based Test Definitions below)

- [x] 16. Integration tests
  - [x] 16.1 Write integration test: full caret configuration load from theme → render info generation lifecycle
  - [x] 16.2 Write integration test: multi-caret scenario with mixed primary/additional colours and selections
  - [x] 16.3 Write integration test: rectangular selection spanning lines with virtual space extension
  - [x] 16.4 Write integration test: caret-line highlight mode switching (None → Frame → Fill) with hot-reload
  - [x] 16.5 Write integration test: focus gain/loss cycle with blink reset and inactive selection colour switch
  - [x] 16.6 Write integration test: modified line markers with save-clear cycle
  - Covers: End-to-end validation across Requirements 1–12

- [x] 17. Mouse selection in editor canvas (Requirement 13)
  - [x] 17.1 Add `canvas_selection: Option<(usize, usize, usize, usize)>` (anchor_line, anchor_col, end_line, end_col) to `TabState` in `tab_state.rs`
  - [x] 17.2 Add `selection_drag_active: bool` to `TabState` for drag-in-progress tracking
  - [x] 17.3 Implement `screen_to_doc_pos(x: f32, y: f32, top_line: usize, line_height: f32, char_width: f32) -> (usize, usize)` pure function in `editor_panel.rs`
  - [x] 17.4 Wire `drag_started()` / `drag_delta()` / `drag_released()` on the canvas `Response` in `editor_panel.rs` to update `canvas_selection`
  - [x] 17.5 Render selection highlight rects behind text for each visible line intersecting the selection range
  - [x] 17.6 Wire Escape key to clear `canvas_selection` in the editor panel key-event loop
  - [x] 17.7 Wire Ctrl+C in the editor panel: when `canvas_selection` is Some, extract text from document model and write to OS clipboard via `ff-clipboard`
  - [x] 17.8 Display "Copied N characters" in status bar after successful copy
  - [x] 17.9 Clear `canvas_selection` on tab switch in `tab_manager.rs`
  - [x] 17.10 Write unit tests: `screen_to_doc_pos` coordinate conversion, selection range extraction, multi-line join
  - Covers: Requirement 13 (AC 13.1-13.10)

- [x] 18. Selectable text in read-only panels (Requirement 14)
  - [x] 18.1 Replace `ui.label()` calls in `primary_option_menu.rs` with selectable label equivalents for calendar text
  - [x] 18.2 Replace `ui.label()` calls in `settings_panel.rs` for key names, values, and descriptions
  - [x] 18.3 Replace `ui.label()` calls in the status bar render path for phase, session start, line/col, encoding, line count, and version fields
  - [x] 18.4 Verify that POM option button click-to-navigate still works after label change (POM options remain egui::Button -- no change needed)
  - [x] 18.5 Write unit tests confirming selectable label state is set on POM and Settings panel text items
  - Covers: Requirement 14 (AC 14.1-14.5)

---

## Property-Based Test Definitions

### Property 1: Caret Width Clamping Correctness

**Validates: Requirements 1.5, 1.6**

- **Statement:** For any input caret width value (u8 range), the validated width SHALL always be within [1, 20]. Values below 1 are clamped to 1, values above 20 are clamped to 20, and values in [1, 20] pass through unchanged.
- **Strategy:** Generate:
  - width: u8 in [0, 255]
- **Invariant:** `1 <= validated_width <= 20` AND `validated_width == width.clamp(1, 20)`

### Property 2: Blink Visibility Timing Correctness

**Validates: Requirements 3.3, 3.5**

- **Statement:** For any blink period > 0 and any elapsed time since reset, the caret SHALL be visible in the first half of the cycle (elapsed % period < period / 2) and hidden in the second half. For period = 0, the caret SHALL always be visible regardless of elapsed time.
- **Strategy:** Generate:
  - period_ms: u32 in [0, 5000]
  - reset_time_ms: u64 in [0, 100_000]
  - current_time_ms: u64 in [reset_time_ms, reset_time_ms + 10_000]
- **Invariant:** If period == 0 → visible == true. If period > 0 → visible == ((current - reset) % period < period / 2)

### Property 3: Caret-Line Frame Width Clamping Correctness

**Validates: Requirements 4.3, 4.5**

- **Statement:** For any configured frame width and any line height, the effective frame width SHALL be clamped to [1, line_height / 3]. This ensures the frame never visually overflows the line.
- **Strategy:** Generate:
  - frame_width: u32 in [0, 100]
  - line_height: u32 in [6, 120]
- **Invariant:** `1 <= effective_width <= line_height / 3` AND `effective_width == frame_width.clamp(1, line_height / 3)`

### Property 4: Virtual Space Horizontal Offset Calculation Correctness

**Validates: Requirements 7.1**

- **Statement:** For any line-end X position, virtual space offset ≥ 0, and space width > 0, the horizontal caret position SHALL equal `line_end_x + virtual_space * space_width`. When virtual_space is 0, the result SHALL equal line_end_x exactly.
- **Strategy:** Generate:
  - line_end_x: f32 in [0.0, 10000.0]
  - virtual_space: u64 in [0, 500]
  - space_width: f32 in [1.0, 50.0]
- **Invariant:** `result == line_end_x + (virtual_space as f32 * space_width)` (within f32 tolerance)

### Property 5: Selection Colour Context Resolution Correctness

**Validates: Requirements 6.1, 6.6**

- **Statement:** For any SelectionContext and any SelectionColourSet configuration, the resolved colour pair SHALL match the context: Primary → SelectionBack/SelectionText, Additional → SelectionAdditionalBack/SelectionAdditionalText, Secondary → SelectionSecondaryBack/SelectionSecondaryText, Inactive → SelectionInactiveBack/SelectionInactiveText. When a text colour element is not set (None), the text override SHALL be None (preserving syntax colours).
- **Strategy:** Generate:
  - context: random SelectionContext variant
  - colour_set: random SelectionColourSet with random Some/None for text fields and random ColourRGBA for back fields
- **Invariant:** Returned pair matches the correct context field AND text=None when text element unset

### Property 6: Overstrike Mode Forces Block Caret Style

**Validates: Requirements 1.3**

- **Statement:** For any configured CaretStyle (Invisible, Line, or Block) and the edit mode Overstrike, the effective caret style SHALL always be Block. For Insert or Browse modes, the effective style SHALL equal the configured style.
- **Strategy:** Generate:
  - configured_style: random CaretStyle variant
  - edit_mode: random EditMode variant (Insert, Overstrike, Browse)
- **Invariant:** If edit_mode == Overstrike → effective == Block. If edit_mode != Overstrike → effective == configured_style

### Property 7: Configuration Round-Trip from Theme and Back

**Validates: Requirements 11.1, 11.3**

- **Statement:** For any CaretSelectionConfig constructed from valid theme values, applying a theme update with the same values SHALL produce an identical configuration. Default values SHALL be used when the theme does not specify a setting.
- **Strategy:** Generate:
  - Random valid theme values for all configurable settings
  - Construct CaretSelectionConfig from those values
  - Apply theme update with same values
- **Invariant:** `config_after_update == original_config`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2", "3", "4", "14"], "dependsOn": [0] },
    { "id": 2, "label": "Display Models", "tasks": ["5", "6", "7", "8"], "dependsOn": [1] },
    { "id": 3, "label": "Advanced Display", "tasks": ["9", "10", "11"], "dependsOn": [2] },
    { "id": 4, "label": "Configuration and Focus", "tasks": ["12", "13"], "dependsOn": [3] },
    { "id": 5, "label": "Validation", "tasks": ["15", "16"], "dependsOn": [4] }
  ]
}
```

---

## Notes

- This is a Wave 6 (UI and Rendering) crate that is purely a **model and configuration layer** — it does not perform actual rendering. The GUI shell (`ff-desktop`) reads the model's state and renders accordingly.
- GUI independence is a strict requirement: no `egui`, `wgpu`, `winit` or other rendering types in this crate's public API.
- The logical selection model (SelectionPosition, SelectionRange, SelectionContainer, SelectionKind) is owned by `ff-edit-operations` — this crate only references those types for computing render output.
- Element colours (Caret, CaretAdditional, CaretLineBack, SelectionBack, etc.) are resolved through `ff-theme`'s element colour API.
- The blink timer is NOT implemented here — only the period value and `is_visible(current_time)` query. The GUI shell owns the clock.
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property.
- Hot-reload of theme settings leverages the configuration-system file watcher and theme change notifications from `ff-theme`.
- Modified line marker rendering consumes `ModifiedLineTracker` from `ff-edit-operations` — no duplication of modification tracking logic.

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Caret Shape and Style | AC 1.1–1.10 | Task 2 |
| Req 2: Caret Colour | AC 2.1–2.7 | Task 3 |
| Req 3: Caret Blink | AC 3.1–3.7 | Task 4 |
| Req 4: Caret Line Highlight | AC 4.1–4.13 | Task 5 |
| Req 5: Selection Display — Colours and Layers | AC 5.1–5.10 | Task 6 |
| Req 6: Selection Element Colours | AC 6.1–6.10 | Task 7 |
| Req 7: Virtual Space Display | AC 7.1–7.6 | Task 8 |
| Req 8: Rectangular Selection Display | AC 8.1–8.5 | Task 9 |
| Req 9: Multi-Caret Display | AC 9.1–9.6 | Task 10 |
| Req 10: Modified Line Marker Rendering | AC 10.1–10.5 | Task 11 |
| Req 11: Theme Integration and Configuration | AC 11.1–11.5 | Task 12 |
| Req 12: Caret Keyboard Focus Integration | AC 12.1–12.3 | Task 13 |
| Cross-cutting Req 8: Error Message Standards | All | Task 14 |
