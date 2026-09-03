# Implementation Plan: Whitespace & Guides (`ff-whitespace-guides`)

## Overview

This plan covers the complete implementation of the `ff-whitespace-guides` crate — the visual indicator layer for invisible characters, structural indentation guides, column boundary markers, and line-wrap continuation markers in FileForgeWorkbench. The crate is **GUI-independent**: it defines settings, enums, per-line metadata queries, and toggle commands, while actual rendering is delegated to the GUI shell.

This is a **Wave 6 (UI and Rendering)** sub-project. It depends on:
- `ff-configuration-system` (Wave 2) for TOML-based configuration storage, hot-reload
- `ff-theme` (Wave 6) for colour resolution of whitespace glyphs, guides, edge indicators, and wrap markers
- `ff-document-model` (Wave 4) for line content, tab size, and indent size
- `ff-display-line-mapping` (Wave 4) for sub-line identification (wrap markers)
- `ff-command` (Wave 2) for toggle command registration

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-whitespace-guides/Cargo.toml` with dependencies (serde, thiserror, proptest dev-dep) and deps on `ff-configuration-system`, `ff-theme`, `ff-document-model`, `ff-display-line-mapping`, `ff-command`, `ff-logging`
  - [x] 1.2 Create `crates/ff-whitespace-guides/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `whitespace.rs`, `indent_guides.rs`, `edge_column.rs`, `wrap_markers.rs`, `settings.rs`, `commands.rs`, `queries.rs`, `error.rs`
  - [x] 1.4 Add `ff-whitespace-guides` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements; Requirement 9 (AC 9.1)

- [x] 2. Whitespace visibility types and configuration
  - [x] 2.1 Define `WhitespaceVisibility` enum: `Invisible`, `VisibleAlways`, `VisibleAfterIndent`, `VisibleOnlyInIndent` with serde support and `Default` impl (Invisible)
  - [x] 2.2 Define `TabDrawMode` enum: `LongArrow`, `Strikeout` with serde support and `Default` impl (LongArrow)
  - [x] 2.3 Define `WhitespaceConfig` struct: `visibility: WhitespaceVisibility`, `tab_draw_mode: TabDrawMode`, `whitespace_size: u8` (min 1, default 1)
  - [x] 2.4 Implement configuration-system integration: read from `editor.whitespace_mode`, `editor.tab_draw_mode`, `editor.whitespace_size` keys
  - [x] 2.5 Implement hot-reload subscription for all three whitespace configuration keys
  - [x] 2.6 Write unit tests for enum default values, config key parsing, size validation (clamp to min 1)
  - Covers: Requirement 1 (AC 1.1, 1.2, 1.6), Requirement 2 (AC 2.1–2.6)

- [x] 3. Whitespace glyph position queries
  - [x] 3.1 Implement `WhitespaceGlyph` enum: `SpaceDot`, `TabArrow`, `TabStrikeout`
  - [x] 3.2 Implement `WhitespaceGlyphPosition` struct: `column: usize`, `glyph: WhitespaceGlyph`, `width: usize` (for tabs: tab span width)
  - [x] 3.3 Implement `compute_whitespace_glyphs(line: &str, tab_size: usize, visibility: WhitespaceVisibility, tab_draw_mode: TabDrawMode) -> Vec<WhitespaceGlyphPosition>` returning positions and glyph types for a given line based on the active visibility mode
  - [x] 3.4 Implement filtering logic: `VisibleAlways` returns all, `VisibleAfterIndent` skips leading, `VisibleOnlyInIndent` returns only leading, `Invisible` returns empty
  - [x] 3.5 Write unit tests for each visibility mode: empty line, all-whitespace line, mixed content, trailing whitespace, tabs mixed with spaces
  - Covers: Requirement 1 (AC 1.3, 1.4, 1.5), Requirement 2 (AC 2.1, 2.2)

- [x] 4. Whitespace colour resolution
  - [x] 4.1 Implement `WhitespaceColours` struct: `foreground: ColourRGBA`, `background: Option<ColourRGBA>`
  - [x] 4.2 Implement `resolve_whitespace_colours(theme: &ThemeApi) -> WhitespaceColours` that reads whitespace-specific colours from the theme, falling back to default text foreground if unset
  - [x] 4.3 Write unit tests for colour resolution: explicit theme colours, fallback to text foreground, alpha channel handling
  - Covers: Requirement 2 (AC 2.7, 2.8, 2.9)

- [x] 5. Indent guide types and configuration
  - [x] 5.1 Define `IndentGuideMode` enum: `None`, `Real`, `LookForward`, `LookBoth` with serde support and `Default` impl (None)
  - [x] 5.2 Define `IndentGuideConfig` struct: `mode: IndentGuideMode`
  - [x] 5.3 Implement configuration-system integration: read from `editor.indent_guides` key
  - [x] 5.4 Implement hot-reload subscription for indent guide configuration key
  - [x] 5.5 Write unit tests for enum default, config key parsing, mode transitions
  - Covers: Requirement 3 (AC 3.1, 3.2, 3.7)

- [x] 6. Indent guide column computation
  - [x] 6.1 Implement `compute_indent_level(line: &str, tab_size: usize) -> usize` returning the indentation column count for a single line
  - [x] 6.2 Implement `compute_real_guides(line: &str, tab_size: usize) -> Vec<usize>` returning guide columns for a line using `Real` mode (guides at each tab-stop within leading whitespace)
  - [x] 6.3 Implement `compute_look_forward_guides(lines: &[&str], line_index: usize, tab_size: usize) -> Vec<usize>` that scans forward through blank/short-indent lines to determine guide columns
  - [x] 6.4 Implement `compute_look_both_guides(lines: &[&str], line_index: usize, tab_size: usize) -> Vec<usize>` that scans both forward and backward, using the max indent level
  - [x] 6.5 Implement `IndentGuideQuery::guides_for_line(line_index: usize) -> Vec<usize>` dispatcher that delegates to the appropriate computation based on active mode
  - [x] 6.6 Write unit tests for each mode: empty lines, lines with only whitespace, deeply nested blocks, tab/space mixing, blank-line spanning for LookForward and LookBoth
  - Covers: Requirement 3 (AC 3.3, 3.4, 3.5, 3.6, 3.8)

- [x] 7. Active indent guide highlighting
  - [x] 7.1 Define `ActiveGuideState` struct: `column: Option<usize>` representing the currently highlighted guide column (None = no highlight)
  - [x] 7.2 Implement `compute_active_guide(caret_line: usize, caret_column: usize, lines: &[&str], tab_size: usize) -> Option<usize>` that determines the innermost scope indent level at the caret position
  - [x] 7.3 Implement scope detection heuristic: walk outward from caret line to find brace/scope boundaries and return the indentation column of the enclosing block
  - [x] 7.4 Implement `update_active_guide(new_caret_line: usize, new_caret_column: usize) -> bool` returning whether the active guide changed (for incremental repaint signalling)
  - [x] 7.5 Write unit tests for: caret at column 0 (no guide), caret inside nested block, caret movement changes guide, no matching braces found
  - Covers: Requirement 4 (AC 4.1, 4.2, 4.3, 4.4, 4.5)

- [x] 8. Edge column indicator types and configuration
  - [x] 8.1 Define `EdgeMode` enum: `None`, `Line`, `Background`, `MultiLine` with serde support and `Default` impl (None)
  - [x] 8.2 Define `EdgeProperties` struct: `column: usize`, `colour: ColourRGBA`
  - [x] 8.3 Define `EdgeConfig` struct: `mode: EdgeMode`, `column: usize` (default 80), `colour: ColourRGBA`, `multi_edges: Vec<EdgeProperties>`
  - [x] 8.4 Implement configuration-system integration: read from `editor.edge_mode`, `editor.edge_column`, `editor.edge_colour`, `editor.edge_columns` keys
  - [x] 8.5 Implement hot-reload subscription for all edge configuration keys
  - [x] 8.6 Write unit tests for enum defaults, config parsing, multi-edge array deserialization
  - Covers: Requirement 5 (AC 5.1, 5.2, 5.6, 5.7, 5.8)

- [x] 9. Edge column queries
  - [x] 9.1 Implement `EdgeIndicator` enum: `VerticalLine { column: usize, colour: ColourRGBA }`, `BackgroundShading { start_column: usize, colour: ColourRGBA }`, `MultiVerticalLine { lines: Vec<EdgeProperties> }`
  - [x] 9.2 Implement `compute_edge_indicator(config: &EdgeConfig) -> Option<EdgeIndicator>` returning the appropriate indicator type based on the active mode
  - [x] 9.3 Implement multi-edge clearing: `clear_multi_edges()` resetting the list to empty
  - [x] 9.4 Write unit tests for: None mode returns None, Line mode returns single column, Background mode returns shading start, MultiLine mode returns all entries, clear resets
  - Covers: Requirement 5 (AC 5.3, 5.4, 5.5, 5.9, 5.10)

- [x] 10. Wrap marker types and configuration
  - [x] 10.1 Define `WrapVisualFlag` bitfield struct supporting combinations of: `NONE`, `END`, `START`, `MARGIN` with serde support and `Default` impl (NONE)
  - [x] 10.2 Define `WrapVisualLocation` enum: `Default`, `EndByText`, `StartByText` with serde support
  - [x] 10.3 Define `WrapIndentMode` enum: `Fixed`, `Same`, `Indent`, `DeepIndent` with serde support and `Default` impl (Fixed)
  - [x] 10.4 Define `WrapMarkerConfig` struct: `flags: WrapVisualFlag`, `location: WrapVisualLocation`, `indent_mode: WrapIndentMode`, `start_indent: usize` (default 0)
  - [x] 10.5 Implement configuration-system integration: read from `editor.wrap_visual_flags`, `editor.wrap_visual_location`, `editor.wrap_indent_mode`, `editor.wrap_start_indent` keys
  - [x] 10.6 Implement hot-reload subscription for all wrap marker configuration keys
  - [x] 10.7 Write unit tests for bitfield combinations, enum defaults, config parsing, start_indent validation (non-negative)
  - Covers: Requirement 6 (AC 6.1, 6.2, 6.6, 6.7), Requirement 7 (AC 7.1, 7.2, 7.3, 7.4)

- [x] 11. Wrap marker and indentation queries
  - [x] 11.1 Implement `WrapMarkerPosition` struct: `sub_line_index: usize`, `marker_type: WrapMarkerType` (enum: End, Start, Margin), `position: WrapMarkerPlacement` (enum: AtEdge, ByText)
  - [x] 11.2 Implement `compute_wrap_markers(sub_line_count: usize, flags: WrapVisualFlag, location: WrapVisualLocation) -> Vec<WrapMarkerPosition>` returning markers for a wrapped document line
  - [x] 11.3 Implement `compute_continuation_indent(first_subline_indent: usize, tab_size: usize, mode: WrapIndentMode, start_indent: usize, viewport_width: usize) -> usize` returning the effective indentation for continuation sub-lines, clamped to 3/4 of viewport width
  - [x] 11.4 Implement guard: when word wrap is not active, all wrap marker queries return empty
  - [x] 11.5 Write unit tests for: no flags returns empty, End flag marks sub-lines, Start flag marks continuations, Margin flag, location variants, indent modes (Fixed/Same/Indent/DeepIndent), 3/4 viewport clamping, wrap-inactive guard
  - Covers: Requirement 6 (AC 6.3, 6.4, 6.5, 6.8, 6.9), Requirement 7 (AC 7.5, 7.6)

- [x] 12. Toggle commands
  - [x] 12.1 Implement `ToggleWhitespace` command: cycles `WhitespaceVisibility` through Invisible → VisibleAlways → VisibleAfterIndent → VisibleOnlyInIndent → Invisible
  - [x] 12.2 Implement `ToggleIndentGuides` command: cycles `IndentGuideMode` through None → Real → LookForward → LookBoth → None
  - [x] 12.3 Implement `ToggleEdgeColumn` command: toggles between `None` and previous non-None mode (defaulting to `Line` if no prior mode set)
  - [x] 12.4 Register all toggle commands with the `command-framework` including metadata (display name, description, category)
  - [x] 12.5 Implement configuration persistence: each toggle writes the new state to the user layer of the configuration-system
  - [x] 12.6 Implement change notification emission on each toggle so the viewport repaints immediately
  - [x] 12.7 Write unit tests for: whitespace cycling order, indent guide cycling order, edge toggle remembers previous mode, persistence to config, notification emission
  - Covers: Requirement 8 (AC 8.1, 8.2, 8.3, 8.4, 8.5, 8.6)

- [x] 13. Aggregated settings struct and public API
  - [x] 13.1 Define `WhitespaceSettings` struct aggregating: `WhitespaceConfig`, `IndentGuideConfig`, `EdgeConfig`, `WrapMarkerConfig`, `ActiveGuideState`
  - [x] 13.2 Implement `WhitespaceSettings::from_config(config_system: &ConfigSystem) -> Self` constructor loading all settings from configuration
  - [x] 13.3 Implement hot-reload coordinator: subscribe to all relevant config keys and update `WhitespaceSettings` atomically on change
  - [x] 13.4 Implement `WhitespaceSettingsApi` facade with query methods: `whitespace_glyphs_for_line(...)`, `guides_for_line(...)`, `active_guide()`, `edge_indicator()`, `wrap_markers_for_line(...)`, `continuation_indent(...)`
  - [x] 13.5 Write unit tests for: settings construction from config, atomic update, facade delegates correctly
  - Covers: Requirement 9 (AC 9.2, 9.3, 9.4, 9.5)

- [x] 14. Error handling
  - [x] 14.1 Define `WhitespaceGuidesError` enum: `InvalidWhitespaceMode`, `InvalidTabDrawMode`, `InvalidIndentGuideMode`, `InvalidEdgeMode`, `InvalidEdgeColumn`, `InvalidWrapFlags`, `ConfigReadError`
  - [x] 14.2 Implement error message formatting per `[whitespace-guides] operation: description` standard (≤200 chars)
  - [x] 14.3 Implement graceful fallback: invalid configuration values use defaults with WARN-level logging
  - [x] 14.4 Write unit tests for all error variants, message formatting, fallback behaviour
  - Covers: Cross-cutting Requirement 8 (Error Message Standards)

- [x] 15. Property-based tests
  - [x] 15.1 Write PBT: whitespace glyph completeness — for any line and VisibleAlways mode, every space/tab character produces exactly one glyph
  - [x] 15.2 Write PBT: indent guide column alignment — all guide columns are multiples of tab_size
  - [x] 15.3 Write PBT: LookBoth produces superset of Real guides — for any document, guides in LookBoth mode at any line are a superset of guides in Real mode for that line
  - [x] 15.4 Write PBT: continuation indent clamping — effective wrap indent never exceeds 3/4 of viewport width
  - [x] 15.5 Write PBT: toggle command cycling — applying toggle N times returns to original state (N = number of enum variants)
  - [x] 15.6 Write PBT: edge indicator mode consistency — EdgeMode::None always yields no indicator, non-None always yields an indicator
  - Covers: Requirements 1, 3, 5, 6, 7, 8 (see Property-Based Test Definitions below)

- [x] 16. Integration tests
  - [x] 16.1 Write integration test: full lifecycle — construct settings from config, toggle whitespace, verify glyph output changes
  - [x] 16.2 Write integration test: indent guide spanning — multi-line document with blank lines, verify LookBoth guides extend through blanks
  - [x] 16.3 Write integration test: edge column multi-line — configure multiple edges, verify all are returned
  - [x] 16.4 Write integration test: wrap marker end-to-end — enable wrap, set flags, compute markers for wrapped line
  - [x] 16.5 Write integration test: hot-reload cycle — modify config key, verify settings update and notification fires
  - [x] 16.6 Write integration test: headless testability — all queries work without windowing system
  - Covers: End-to-end validation across Requirements 1–9

---

## Property-Based Test Definitions

### Property 1: Whitespace Glyph Completeness

**Validates: Requirements 1.3**

- **Statement:** For any line containing N whitespace characters (spaces + tabs) and `VisibleAlways` mode, `compute_whitespace_glyphs` SHALL return exactly N glyph positions, one for each whitespace character.
- **Strategy:** Generate:
  - line: arbitrary string of printable ASCII and whitespace (length 0–200)
  - tab_size: [1, 8]
- **Invariant:** `glyphs.len() == line.chars().filter(|c| c == ' ' || c == '\t').count()`

### Property 2: Indent Guide Column Alignment

**Validates: Requirements 3.3**

- **Statement:** For any line and any tab_size, all guide columns returned by `compute_real_guides` SHALL be exact multiples of tab_size.
- **Strategy:** Generate:
  - line: string with leading whitespace (spaces and tabs, 0–50 chars) followed by non-whitespace
  - tab_size: [1, 8]
- **Invariant:** `guides.iter().all(|col| col % tab_size == 0)`

### Property 3: LookBoth Superset of Real Guides

**Validates: Requirements 3.4, 3.5**

- **Statement:** For any document (list of lines) and any line index, the set of guide columns produced by `LookBoth` mode SHALL be a superset of (or equal to) the set produced by `Real` mode for the same line.
- **Strategy:** Generate:
  - document: Vec of 1–20 lines with random indentation (0–40 spaces)
  - line_index: valid index into document
  - tab_size: [1, 8]
- **Invariant:** `real_guides.is_subset(&look_both_guides)`

### Property 4: Continuation Indent Clamping

**Validates: Requirements 7.6**

- **Statement:** For any combination of first-subline indent, tab_size, WrapIndentMode, wrap_start_indent, and viewport_width, the computed continuation indent SHALL never exceed `viewport_width * 3 / 4`.
- **Strategy:** Generate:
  - first_subline_indent: [0, 200]
  - tab_size: [1, 8]
  - mode: random WrapIndentMode
  - start_indent: [0, 50]
  - viewport_width: [20, 300]
- **Invariant:** `continuation_indent <= viewport_width * 3 / 4`

### Property 5: Toggle Command Cycling

**Validates: Requirements 8.1, 8.2**

- **Statement:** For `ToggleWhitespace` applied 4 times starting from any `WhitespaceVisibility` value, the result SHALL equal the starting value. For `ToggleIndentGuides` applied 4 times from any `IndentGuideMode`, the result SHALL equal the starting value.
- **Strategy:** Generate:
  - start_whitespace: random WhitespaceVisibility variant
  - start_indent: random IndentGuideMode variant
- **Invariant:** `toggle^4(start) == start` for both enums

### Property 6: Edge Indicator Mode Consistency

**Validates: Requirements 5.1, 5.2**

- **Statement:** For any `EdgeConfig`, if `mode == None` then `compute_edge_indicator` returns `None`; if `mode != None` then it returns `Some(indicator)`.
- **Strategy:** Generate:
  - mode: random EdgeMode
  - column: [1, 300]
  - colour: random ColourRGBA
  - multi_edges: Vec of 0–5 random EdgeProperties (only relevant for MultiLine)
- **Invariant:** `(mode == None) == indicator.is_none()`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Config", "tasks": ["2", "5", "8", "10", "14"], "dependsOn": [0] },
    { "id": 2, "label": "Queries and Computation", "tasks": ["3", "4", "6", "7", "9", "11"], "dependsOn": [1] },
    { "id": 3, "label": "Commands and API", "tasks": ["12", "13"], "dependsOn": [2] },
    { "id": 4, "label": "Validation", "tasks": ["15", "16"], "dependsOn": [3] }
  ]
}
```

---

## Notes

- This is a Wave 6 (UI and Rendering) crate; it exposes only data types and query functions — no rendering code
- The crate has zero GUI framework dependencies (no `egui`, `winit`, `wgpu`) per Requirement 9
- All colours are resolved from `ff-theme` via the element/token system; no hardcoded colour values
- Indent guide computation for `LookForward` and `LookBoth` modes requires access to neighbouring lines from the document model
- The active indent guide depends on caret position, which is updated by the `caret-and-selection` subsystem calling into this crate
- Wrap markers are only meaningful when word wrap is active (controlled by `line-wrap-toggle`); the guard prevents rendering when wrap is off
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Hot-reload leverages the configuration-system file watcher — `ff-whitespace-guides` does not implement its own watcher
- The design.md for this crate may be generated concurrently; task structure is derived from requirements.md

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Whitespace Visibility Modes | AC 1.1–1.6 | Tasks 2, 3 |
| Req 2: Whitespace Glyph Appearance | AC 2.1–2.9 | Tasks 2, 3, 4 |
| Req 3: Indent Guide Display | AC 3.1–3.8 | Tasks 5, 6 |
| Req 4: Active Indent Guide Highlighting | AC 4.1–4.5 | Task 7 |
| Req 5: Edge Column Indicator | AC 5.1–5.10 | Tasks 8, 9 |
| Req 6: Wrap Visual Markers | AC 6.1–6.9 | Tasks 10, 11 |
| Req 7: Wrap Indentation for Continuation Sub-Lines | AC 7.1–7.6 | Tasks 10, 11 |
| Req 8: Toggle Commands | AC 8.1–8.6 | Task 12 |
| Req 9: GUI-Independent Model | AC 9.1–9.5 | Tasks 1, 13 |
| Cross-cutting Req 8: Error Message Standards | All | Task 14 |
