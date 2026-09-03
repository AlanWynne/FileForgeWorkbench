# Implementation Plan: Line Wrap Toggle (`ff-wrap`)

## Overview

This plan covers the complete implementation of the `ff-wrap` crate — the line wrap toggle subsystem for FileForgeWorkbench. It provides per-editor-instance wrap mode (None/Word/Character), configurable wrap boundaries (viewport width or fixed column), wrap indent modes for continuation lines, display-line-mapping integration for sub-line height tracking, viewport recalculation on mode change, WRAP primary command with sub-commands, horizontal scrollbar interaction, status bar indicator, View menu integration, wrap visual flags, session persistence, and configuration defaults.

This is a **Wave 9 (Desktop Integration)** sub-project. It depends on `ff-display-line-mapping` for document-to-display coordinate translation and height tracking, `ff-config` for wrap configuration keys, `ff-command` for WRAP command registration, `ff-viewport` for scroll recalculation, `ff-logging` for configuration warnings, and integrates with `ff-multi-tab` for per-editor-instance state, `ff-session` for persistence, `ff-statusbar` for indicator rendering, and `ff-whitespace-guides` for visual flag rendering.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-wrap/Cargo.toml` with dependencies (ff-display-line-mapping, ff-config, ff-command, ff-viewport, ff-logging, thiserror, serde, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-wrap/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `mode.rs`, `boundary.rs`, `indent.rs`, `state.rs`, `layout.rs`, `commands.rs`, `config.rs`, `indicator.rs`, `visual_flags.rs`, `persistence.rs`, `scrollbar.rs`, `error.rs`
  - [x] 1.4 Add `ff-wrap` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. WrapMode enum and WrapConfig model
  - [x] 2.1 Define `WrapMode` enum with variants: `None`, `Word`, `Character` — derive Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
  - [x] 2.2 Implement `WrapMode::is_active(&self) -> bool` returning true for Word and Character
  - [x] 2.3 Implement `WrapMode::default_enabled() -> Self` returning `Word` (the default when user enables wrapping)
  - [x] 2.4 Implement `WrapMode::display_label(&self) -> &'static str` returning "Off", "Word", or "Char"
  - [x] 2.5 Define `WrapBoundary` enum with variants: `Viewport`, `Column(u32)` — derive Debug, Clone, Copy, PartialEq, Eq
  - [x] 2.6 Implement `WrapBoundary::from_column(n: i32) -> Self` returning Viewport for 0, Column(n) for valid positive, Viewport with warning for invalid
  - [x] 2.7 Define `WrapIndentMode` enum with variants: `Fixed`, `Same`, `Indent`, `DeepIndent` — derive Debug, Clone, Copy, PartialEq, Eq
  - [x] 2.8 Define `WrapVisualFlags` enum with variants: `None`, `End`, `Start`, `StartEnd`, `Margin` — derive Debug, Clone, Copy, PartialEq, Eq
  - [x] 2.9 Write unit tests for WrapMode predicates, display labels, WrapBoundary construction and validation
  - Covers: Requirement 1 (AC 1.1–1.6), Requirement 4 (AC 4.1), Requirement 5 (AC 5.1), Requirement 10 (AC 10.1)

- [x] 3. Wrap configuration integration
  - [x] 3.1 Define `WrapConfig` struct with fields: `default_mode: WrapMode`, `wrap_column: u32`, `indent_mode: WrapIndentMode`, `indent_amount: u32`, `visual_flags: WrapVisualFlags`
  - [x] 3.2 Implement `WrapConfig::default()` returning `{ default_mode: None, wrap_column: 0, indent_mode: Fixed, indent_amount: 0, visual_flags: None }`
  - [x] 3.3 Implement `WrapConfig::validate(&mut self) -> Vec<ConfigWarning>` applying rules: wrap_column negative or >10000 resets to 0; indent_amount clamped to 0–40; invalid enum strings reset to defaults
  - [x] 3.4 Implement `WrapConfig::from_config_store(store: &ConfigStore) -> Self` reading `[view.wrap]` table keys and calling validate
  - [x] 3.5 Implement `WrapConfig::boundary(&self) -> WrapBoundary` returning Viewport when wrap_column is 0, Column(n) otherwise
  - [x] 3.6 Implement hot-reload: `WrapConfig::on_config_changed(new_store: &ConfigStore) -> (Self, Vec<ConfigWarning>)` — new defaults apply to newly opened documents only
  - [x] 3.7 Write unit tests for defaults, each validation rule (column out of range, indent_amount clamping, invalid mode strings, layered overrides), hot-reload behaviour
  - Covers: Requirement 12 (AC 12.1–12.5), Requirement 4 (AC 4.5–4.7), Requirement 5 (AC 5.6–5.8)

- [x] 4. WrapState per-document management
  - [x] 4.1 Define `WrapState` struct with fields: `mode: WrapMode`, `boundary: WrapBoundary`, `indent_mode: WrapIndentMode`, `indent_amount: u32`, `visual_flags: WrapVisualFlags`
  - [x] 4.2 Implement `WrapState::new(config: &WrapConfig) -> Self` initialising from configuration defaults
  - [x] 4.3 Implement `WrapState::from_persisted(entry: &WrapSessionEntry, config: &WrapConfig) -> Self` restoring session state with fallback to config defaults
  - [x] 4.4 Implement `WrapState::mode(&self) -> WrapMode` accessor
  - [x] 4.5 Implement `WrapState::is_active(&self) -> bool` delegating to mode.is_active()
  - [x] 4.6 Implement `WrapState::set_mode(&mut self, mode: WrapMode) -> WrapModeChange` returning old and new mode for event emission
  - [x] 4.7 Implement `WrapState::set_boundary(&mut self, boundary: WrapBoundary)` setter
  - [x] 4.8 Implement `WrapState::effective_wrap_width(&self, viewport_width: u32) -> u32` returning viewport_width for Viewport boundary, column value for Column(n)
  - [x] 4.9 Write unit tests for construction from config, independence of instances, mode change events, effective width computation
  - Covers: Requirement 2 (AC 2.1–2.5), Requirement 4 (AC 4.1–4.4)

- [x] 5. Wrap column computation and line-breaking logic
  - [x] 5.1 Define `WrapBreaker` trait with method `compute_breaks(line: &str, wrap_width: u32, mode: WrapMode) -> Vec<usize>` returning character offsets where breaks occur
  - [x] 5.2 Implement `WordWrapBreaker` that breaks at word boundaries (whitespace, punctuation adjacent to alphanumeric) within wrap_width
  - [x] 5.3 Implement word-overflow fallback: when a single word exceeds wrap_width, break at exact character position (Req 1 AC 1.4)
  - [x] 5.4 Implement `CharWrapBreaker` that breaks at exact character position filling wrap_width without regard to word boundaries
  - [x] 5.5 Implement `compute_sub_line_count(line: &str, wrap_width: u32, mode: WrapMode, indent: &IndentInfo) -> u32` returning the display height (number of sub-lines)
  - [x] 5.6 Implement indent-aware width reduction: continuation lines subtract indent offset from available width (Req 5 AC 5.9)
  - [x] 5.7 Define `IndentInfo` struct computed from `WrapIndentMode` and `indent_amount` — holds resolved pixel/character offset for continuation lines
  - [x] 5.8 Implement `IndentInfo::compute(mode: WrapIndentMode, amount: u32, line_indent: u32, indent_width: u32) -> Self` resolving Fixed/Same/Indent/DeepIndent
  - [x] 5.9 Write unit tests for word breaking (normal, long word overflow), character breaking, sub-line count, indent modes (Fixed, Same, Indent, DeepIndent), edge cases (empty line, single-char line, line exactly at boundary)
  - Covers: Requirement 1 (AC 1.2–1.5), Requirement 5 (AC 5.1–5.9)

- [x] 6. Display-line-mapping integration
  - [x] 6.1 Implement `WrapHeightCalculator` struct that computes display heights for document lines based on current WrapState
  - [x] 6.2 Implement `WrapHeightCalculator::compute_height(line: &str, state: &WrapState, viewport_width: u32) -> u32` returning sub-line count (1 when mode is None)
  - [x] 6.3 Implement `on_wrap_mode_enabled(doc_lines: &[&str], state: &WrapState, viewport_width: u32) -> Vec<(usize, u32)>` computing heights for all visible lines, returning (doc_line, height) pairs for set_height calls
  - [x] 6.4 Implement `on_wrap_mode_disabled(visible_line_count: usize) -> Vec<(usize, u32)>` returning (doc_line, 1) for all visible lines to reset heights
  - [x] 6.5 Implement `on_line_edited(line_idx: usize, new_content: &str, state: &WrapState, viewport_width: u32) -> Option<(usize, u32)>` recomputing single-line height, returning Some if changed
  - [x] 6.6 Implement `on_boundary_changed(doc_lines: &[&str], state: &WrapState, new_viewport_width: u32) -> Vec<(usize, u32)>` recomputing all visible line heights for new boundary
  - [x] 6.7 Implement provisional height strategy: lines not yet computed assume height 1 until idle-processing computes them (Req 6 AC 6.6)
  - [x] 6.8 Write unit tests for height computation (unwrapped=1, short line=1, long line>1), mode enable/disable bulk updates, single-line edit recomputation, boundary change recomputation
  - Covers: Requirement 6 (AC 6.1–6.8)

- [x] 7. Viewport recalculation on mode change
  - [x] 7.1 Define `WrapViewportEvent` struct with fields: `editor_instance_id`, `old_mode: WrapMode`, `new_mode: WrapMode`, `total_display_lines: u32`, `requires_scroll_reset: bool`
  - [x] 7.2 Implement `WrapViewportEvent::from_mode_change(id, old, new, total_display_lines) -> Self` constructor — `requires_scroll_reset` is true when transitioning between None↔active modes
  - [x] 7.3 Implement scroll position adjustment logic: when wrap enables, translate current `top_line` from document coordinates to display coordinates via display-line-mapping
  - [x] 7.4 Implement scrollbar range update: emit new total display line count for vertical scrollbar range recalculation
  - [x] 7.5 Implement resize-triggered recalculation: when viewport width changes while wrap is active (Viewport boundary), recompute all heights and emit new total
  - [x] 7.6 Write unit tests for mode-change event construction, scroll position translation, scrollbar range recalculation, resize-triggered updates
  - Covers: Requirement 6 (AC 6.4, 6.7), Requirement 7 (AC 7.1–7.2)

- [x] 8. Horizontal scrollbar interaction
  - [x] 8.1 Define `ScrollbarVisibility` enum with variants: `Visible`, `Hidden`
  - [x] 8.2 Implement `scrollbar_state(state: &WrapState, viewport_width: u32) -> ScrollbarVisibility` — Hidden when wrap active with Viewport boundary; Visible when wrap None; Visible when wrap active with Column(n) and viewport < n
  - [x] 8.3 Implement `on_wrap_enabled_viewport(h_offset: &mut u32)` resetting horizontal_offset to 0 when wrap activates with Viewport boundary
  - [x] 8.4 Implement `on_wrap_disabled() -> ScrollbarVisibility` returning Visible
  - [x] 8.5 Implement Column(n) narrow-viewport case: scrollbar remains visible when viewport is narrower than column n even when wrap is active
  - [x] 8.6 Write unit tests for scrollbar visibility rules (None→Visible, Word+Viewport→Hidden, Word+Column(80) with viewport=60→Visible), horizontal_offset reset
  - Covers: Requirement 7 (AC 7.1–7.5)

- [x] 9. WRAP primary command handler
  - [x] 9.1 Implement `WrapCommand` struct implementing the command handler trait with Command_ID `"view.wrap"`
  - [x] 9.2 Implement argument parsing: `WRAP` (no args) → toggle, `WRAP ON` → enable Word, `WRAP OFF` → disable, `WRAP TOGGLE` → toggle, `WRAP WORD` → set Word, `WRAP CHAR` → set Character, `WRAP COL n` → set column boundary
  - [x] 9.3 Implement toggle logic: if None → set Word; if Word or Character → set None (Req 3 AC 3.4, 3.5)
  - [x] 9.4 Implement `WRAP ON` idempotency: if already active, return confirmation with current mode (Req 3 AC 3.9)
  - [x] 9.5 Implement `WRAP OFF` idempotency: if already None, return "Wrap is already off" (Req 3 AC 3.10)
  - [x] 9.6 Implement status message generation: "Wrap: Word", "Wrap: Character", "Wrap: Off" (Req 3 AC 3.8)
  - [x] 9.7 Implement `WRAP COL n` parsing: set Column(n) for positive n, revert to Viewport for 0 (Req 4 AC 4.6)
  - [x] 9.8 Implement invalid sub-command handling: display error listing valid sub-commands ON, OFF, TOGGLE, WORD, CHAR, COL (Req 3 AC 3.14)
  - [x] 9.9 Implement mode validation: WRAP valid in Browse, Edit, View, and all special modes (Req 3 AC 3.11)
  - [x] 9.10 Implement non-undoable rule: WRAP does not produce an UndoRecord (Req 3 AC 3.12)
  - [x] 9.11 Implement non-history rule: WRAP not added to command history (Req 3 AC 3.13)
  - [x] 9.12 Write unit tests for each sub-command variant, toggle logic, idempotency cases, invalid argument handling, mode validation, non-undoable and non-history guarantees
  - Covers: Requirement 3 (AC 3.1–3.14), Requirement 4 (AC 4.6–4.7)

- [x] 10. Command registration
  - [x] 10.1 Register `"view.wrap"` command with Command_ID in command-framework registry
  - [x] 10.2 Register WRAP command metadata: description, valid modes, non-undoable flag, non-history flag
  - [x] 10.3 Ensure WRAP command handler delegates to `WrapState` operations on active editor instance
  - [x] 10.4 Write integration test verifying command dispatch triggers correct wrap state changes
  - Covers: Requirement 3 (AC 3.1, 3.11–3.13)

- [x] 11. Status bar wrap indicator model
  - [x] 11.1 Define `WrapIndicatorState` enum with variants: `Hidden`, `Visible { text: String, mode: WrapMode }`
  - [x] 11.2 Implement `WrapIndicatorState::from_mode(mode: WrapMode) -> Self` — Hidden when None, Visible("Wrap: Word") for Word, Visible("Wrap: Char") for Character
  - [x] 11.3 Implement click-to-cycle behaviour model: `cycle_mode(current: WrapMode) -> WrapMode` returning None→Word→Character→None
  - [x] 11.4 Implement tab-switch update: indicator reflects newly active editor instance's wrap mode (Req 8 AC 8.6)
  - [x] 11.5 Write unit tests for indicator visibility rules, text formatting, click-cycle progression, tab-switch update
  - Covers: Requirement 8 (AC 8.1–8.6)

- [x] 12. View menu integration model
  - [x] 12.1 Define `WrapMenuState` struct with fields: `active_mode: WrapMode`, representing the radio-selection state for the View → Word Wrap submenu
  - [x] 12.2 Implement `WrapMenuState::from_mode(mode: WrapMode) -> Self` producing submenu state with radio indicator on current mode
  - [x] 12.3 Implement `WrapMenuState::apply_selection(selection: &str) -> Option<WrapMode>` mapping "Off"→None, "Word"→Word, "Character"→Character
  - [x] 12.4 Implement tab-switch update: menu radio indicator reflects newly active editor instance (Req 9 AC 9.6)
  - [x] 12.5 Write unit tests for menu state construction, selection mapping, tab-switch update
  - Covers: Requirement 9 (AC 9.1–9.6)

- [x] 13. Wrap visual flags rendering model
  - [x] 13.1 Define `WrapMarkerPosition` struct with fields: `sub_line_index: u32`, `location: WrapMarkerLocation` (enum: End, Start, Margin)
  - [x] 13.2 Implement `compute_markers(line_height: u32, flags: WrapVisualFlags) -> Vec<WrapMarkerPosition>` producing marker positions for all continuation lines
  - [x] 13.3 Implement End flag: marker at right edge of each sub-line that continues (Req 10 AC 10.2)
  - [x] 13.4 Implement Start flag: marker at left side of each continuation line (Req 10 AC 10.3)
  - [x] 13.5 Implement StartEnd flag: both End and Start markers on applicable sub-lines
  - [x] 13.6 Implement Margin flag: marker in line-number margin adjacent to continuation lines (Req 10 AC 10.4)
  - [x] 13.7 Implement None flag: no markers produced (Req 10 AC 10.5)
  - [x] 13.8 Write unit tests for each flag mode marker generation, height=1 produces no markers, height=3 produces correct count
  - Covers: Requirement 10 (AC 10.1–10.7)

- [x] 14. Session persistence model
  - [x] 14.1 Define `WrapSessionEntry` struct with fields: `resource_uri: String`, `mode: WrapMode`, `boundary: Option<WrapBoundary>` (None means use global config default)
  - [x] 14.2 Implement `WrapSessionEntry::from_state(uri: &str, state: &WrapState, config: &WrapConfig) -> Self` capturing current wrap state; omit boundary if it matches config default
  - [x] 14.3 Implement `WrapSessionEntry::restore(config: &WrapConfig) -> WrapState` creating state from persisted entry; fallback to config default_mode when entry absent
  - [x] 14.4 Implement invalid-variant fallback: unrecognized mode string → None with warning (Req 11 AC 11.3)
  - [x] 14.5 Implement serialization/deserialization (serde) for session store integration
  - [x] 14.6 Implement batch operations: `persist_all(entries: &[WrapSessionEntry]) -> Vec<u8>` and `restore_all(data: &[u8]) -> Vec<WrapSessionEntry>`
  - [x] 14.7 Write unit tests for persistence round-trip, missing entry defaults, invalid variant fallback, boundary persistence when non-default
  - Covers: Requirement 11 (AC 11.1–11.5)

- [x] 15. Rendering behaviour coordination model
  - [x] 15.1 Define `WrapRenderInfo` struct with fields: `doc_line: usize`, `sub_lines: Vec<SubLineInfo>` containing break offsets and indent offsets per sub-line
  - [x] 15.2 Implement `SubLineInfo` struct with fields: `start_offset: usize`, `end_offset: usize`, `indent_px: u32`, `is_continuation: bool`
  - [x] 15.3 Implement `WrapRenderInfo::compute(line: &str, state: &WrapState, viewport_width: u32) -> Self` producing full sub-line layout for rendering
  - [x] 15.4 Implement line-number gutter rule: only first sub-line shows line number; continuations show blank or marker (Req 13 AC 13.2)
  - [x] 15.5 Implement horizontal_offset suppression: when wrap is active with Viewport boundary, horizontal_offset is not applied (Req 13 AC 13.6)
  - [x] 15.6 Write unit tests for sub-line computation, gutter number assignment, horizontal_offset suppression, cursor position within sub-lines
  - Covers: Requirement 13 (AC 13.1–13.6)

- [x] 16. Error types
  - [x] 16.1 Define `WrapError` enum with variants: `InvalidSubCommand { input: String }`, `InvalidColumn { value: i32 }`, `NoActiveEditor`, `ConfigWarning { key: String, detail: String }`
  - [x] 16.2 Implement `Display` and `thiserror::Error` for all variants
  - [x] 16.3 Write unit tests for error formatting
  - Covers: Error handling across all requirements

- [x] 17. Per-editor-instance independence validation
  - [x] 17.1 Implement integration test: two editor instances with different wrap modes — changing one does not affect the other (Req 2 AC 2.3)
  - [x] 17.2 Implement integration test: tab switch updates indicator and menu to reflect new active instance's mode (Req 2 AC 2.4)
  - [x] 17.3 Implement integration test: new editor instance initialises at default_mode from config (Req 2 AC 2.2)
  - [x] 17.4 Implement integration test: missing/invalid config default_mode falls back to None (Req 2 AC 2.5)
  - [x] 17.5 Write unit tests for independence invariant using multiple WrapState instances
  - Covers: Requirement 2 (AC 2.1–2.5)

- [x] 18. Property-based tests — WrapMode and WrapState invariants
  - [x] 18.1 Write property test: for any WrapMode value, `is_active()` returns true if and only if mode is Word or Character
    - **Validates: Requirement 1.1**
  - [x] 18.2 Write property test: for any WrapState constructed from any valid WrapConfig, mode is always one of the three valid enum variants
    - **Validates: Requirements 1.1, 2.1**
  - [x] 18.3 Write property test: toggling WrapMode twice returns to the original mode (None→Word→None, Word→None→Word, Character→None→Character pending last-used mode)
    - **Validates: Requirement 3.4**
  - [x] 18.4 Write property test: effective_wrap_width returns viewport_width when boundary is Viewport, and returns the column value when boundary is Column(n), for any positive viewport_width and any column value
    - **Validates: Requirements 4.1, 4.2, 4.3**

- [x] 19. Property-based tests — line-breaking invariants
  - [x] 19.1 Write property test: for any non-empty string and any wrap_width >= 1, compute_breaks produces breaks such that no sub-line exceeds wrap_width characters (accounting for indent), regardless of WrapMode (Word or Character)
    - **Validates: Requirements 1.2, 1.3, 1.5**
  - [x] 19.2 Write property test: for any string and Word mode, break positions never split within a word unless the word itself exceeds wrap_width (word-overflow fallback)
    - **Validates: Requirements 1.3, 1.4**
  - [x] 19.3 Write property test: for any string and Character mode, breaks occur at exact multiples of wrap_width (adjusted for indent) regardless of word boundaries
    - **Validates: Requirement 1.5**
  - [x] 19.4 Write property test: concatenating all sub-line segments reconstructs the original line content exactly (no characters lost or duplicated) for any string and any WrapMode
    - **Validates: Requirements 1.2, 1.3, 1.5**
  - [x] 19.5 Write property test: compute_sub_line_count returns 1 for any string shorter than or equal to wrap_width, and >1 for any string longer than wrap_width (with indent=0)
    - **Validates: Requirements 1.2, 6.1**

- [x] 20. Property-based tests — display-line-mapping height invariants
  - [x] 20.1 Write property test: when WrapMode is None, compute_height returns exactly 1 for any string of any length
    - **Validates: Requirements 1.2, 6.2**
  - [x] 20.2 Write property test: when WrapMode is Word or Character, compute_height >= 1 for any input string and any valid wrap_width
    - **Validates: Requirement 6.1**
  - [x] 20.3 Write property test: on_wrap_mode_disabled produces height 1 for all lines, regardless of line content, for any number of visible lines
    - **Validates: Requirement 6.2**
  - [x] 20.4 Write property test: total display lines (sum of all heights) >= document line count, for any document with wrap active and any valid wrap_width
    - **Validates: Requirement 6.7**

- [x] 21. Property-based tests — configuration validation invariants
  - [x] 21.1 Write property test: after validate(), wrap_column is either 0 or within [1, 10000] for any input integer value
    - **Validates: Requirements 4.5, 4.7**
  - [x] 21.2 Write property test: after validate(), indent_amount is always within [0, 40] for any input integer value
    - **Validates: Requirements 5.7, 5.8**
  - [x] 21.3 Write property test: after validate(), default_mode is always a valid WrapMode variant for any input string value
    - **Validates: Requirement 12.2**
  - [x] 21.4 Write property test: hot-reload with any new config values produces a valid WrapConfig where all fields are within their allowed ranges
    - **Validates: Requirement 12.3**

- [x] 22. Property-based tests — session persistence invariants
  - [x] 22.1 Write property test: persist then restore round-trip preserves mode and boundary exactly when values are valid variants, for any valid WrapMode and WrapBoundary
    - **Validates: Requirements 11.1, 11.2**
  - [x] 22.2 Write property test: restoring an invalid/unrecognized mode variant always produces WrapMode::None (never panics, never produces invalid state) for any arbitrary string input
    - **Validates: Requirement 11.3**
  - [x] 22.3 Write property test: restoring with no persisted entry produces a WrapState matching the config default_mode for any valid WrapConfig
    - **Validates: Requirement 11.2**

- [x] 23. Property-based tests — indicator and scrollbar invariants
  - [x] 23.1 Write property test: indicator is Hidden if and only if WrapMode is None, for any WrapMode value
    - **Validates: Requirements 8.1, 8.2, 8.3**
  - [x] 23.2 Write property test: scrollbar is Hidden if and only if wrap is active AND boundary is Viewport, for any combination of WrapMode and WrapBoundary with any viewport_width
    - **Validates: Requirements 7.1, 7.4, 7.5**
  - [x] 23.3 Write property test: when wrap activates with Viewport boundary, horizontal_offset is always reset to 0, for any prior horizontal_offset value
    - **Validates: Requirement 7.1**
  - [x] 23.4 Write property test: indicator text matches "Wrap: Word" or "Wrap: Char" when visible, for any non-None WrapMode
    - **Validates: Requirements 8.1, 8.2**

- [x] 24. Property-based tests — WRAP command invariants
  - [x] 24.1 Write property test: WRAP command always produces a valid WrapMode result (None, Word, or Character) and never leaves state in an invalid/intermediate state, for any valid sub-command string
    - **Validates: Requirements 3.1, 3.8**
  - [x] 24.2 Write property test: WRAP ON from any starting mode always results in an active mode (Word or Character), never None
    - **Validates: Requirements 3.2, 3.9**
  - [x] 24.3 Write property test: WRAP OFF from any starting mode always results in WrapMode::None
    - **Validates: Requirements 3.3, 3.10**
  - [x] 24.4 Write property test: WRAP COL n with n in [1, 10000] always results in boundary Column(n), and WRAP COL 0 always results in Viewport, for any valid integer
    - **Validates: Requirements 4.6, 4.7**

- [x] 25. Integration tests — end-to-end wrap workflows
  - [x] 25.1 Write integration test: create editor instance → WRAP ON → verify mode is Word, scrollbar hidden, indicator shows "Wrap: Word"
  - [x] 25.2 Write integration test: WRAP ON when already Word → verify mode unchanged, confirmation message returned
  - [x] 25.3 Write integration test: WRAP OFF when already None → verify "Wrap is already off" message
  - [x] 25.4 Write integration test: WRAP TOGGLE from None → Word → WRAP TOGGLE → back to None
  - [x] 25.5 Write integration test: WRAP WORD → verify mode Word; WRAP CHAR → verify mode Character
  - [x] 25.6 Write integration test: WRAP COL 80 → verify boundary Column(80); WRAP COL 0 → verify boundary Viewport
  - [x] 25.7 Write integration test: WRAP BANANA → verify error message listing valid sub-commands
  - [x] 25.8 Write integration test: enable wrap on 100-char line with viewport 40 → verify display height is 3 sub-lines
  - [x] 25.9 Write integration test: enable wrap → edit line to shorter → verify display height decreases
  - [x] 25.10 Write integration test: enable wrap → resize viewport → verify all heights recomputed
  - [x] 25.11 Write integration test: wrap active + Viewport boundary → verify horizontal scrollbar hidden and h_offset is 0
  - [x] 25.12 Write integration test: wrap active + Column(80) + viewport width 60 → verify horizontal scrollbar visible
  - [x] 25.13 Write integration test: two editors different modes → tab switch → verify indicator and menu update
  - [x] 25.14 Write integration test: persist wrap state → simulate restart → restore → verify mode matches
  - [x] 25.15 Write integration test: persist mode → change config default → restore missing entry → verify uses new config default
  - [x] 25.16 Write integration test: wrap with indent mode Same on indented line → verify continuation indented to first non-whitespace
  - [x] 25.17 Write integration test: wrap with indent mode DeepIndent → verify continuation indented by 2 extra indent levels
  - [x] 25.18 Write integration test: visual flags End enabled → verify markers produced for continuation lines only
  - [x] 25.19 Write integration test: View menu select "Character" → verify mode changes to Character and radio indicator updates
  - [x] 25.20 Write integration test: status bar click cycles None→Word→Character→None
  - [x] 25.21 Write integration test: WRAP command is not recorded in command history
  - [x] 25.22 Write integration test: WRAP command does not produce UndoRecord
  - [x] 25.23 Write integration test: hot-reload config with new defaults → verify open documents retain current mode, new document uses new default
  - Covers: End-to-end validation of Requirements 1–13

---

## Notes

- The `ff-wrap` crate has zero GUI dependencies — it operates on abstract types and emits `WrapViewportEvent` structs that the rendering layer consumes to trigger re-layout.
- Line wrapping is a **display-only state change**: it never modifies document content, never produces UndoRecords, and is never recorded in command history.
- The `WrapBreaker` trait and height calculator provide the computational bridge between wrap state and the display-line-mapping layer. The display-line-mapping crate (`ff-display-line-mapping`) consumes `set_height` calls to update its contraction state.
- Per-editor-instance independence (Requirement 2) is architecturally enforced by each `WrapState` being owned by its editor instance. The wrap crate does not maintain a global wrap registry.
- The `WrapHeightCalculator` (Task 6) implements incremental computation: only visible and near-viewport lines are computed immediately; remaining lines use provisional height 1 until background idle-processing fills them in.
- Wrap visual flags (Task 13) produce rendering data for the `ff-whitespace-guides` crate to render. The wrap crate computes marker positions; the whitespace crate renders the glyphs.
- Session persistence (Requirement 11) integrates with the session store owned by `ff-session`. The wrap crate provides `WrapSessionEntry` as the serializable payload.
- The WRAP primary command (Task 9) supports keyword sub-commands (ON, OFF, TOGGLE, WORD, CHAR, COL n). It does not interact with the undo system.
- Property-based tests (Tasks 18–24) use the `proptest` crate and are configured for a minimum of 256 iterations.
- The Column(n) wrap boundary mode (Requirement 4) allows horizontal scrolling even when wrap is active — this is the one case where the horizontal scrollbar remains visible during wrapping.
- Hot-reload of configuration (Requirement 12) applies new defaults only to newly opened documents; already-open documents retain their current wrap settings.

---

## Acceptance Criteria Coverage Map

| Task | Requirements Covered |
|------|---------------------|
| 1 | Structural scaffolding (all) |
| 2 | Req 1 (AC 1.1–1.6), Req 4 (AC 4.1), Req 5 (AC 5.1), Req 10 (AC 10.1) |
| 3 | Req 12 (AC 12.1–12.5), Req 4 (AC 4.5–4.7), Req 5 (AC 5.6–5.8) |
| 4 | Req 2 (AC 2.1–2.5), Req 4 (AC 4.1–4.4) |
| 5 | Req 1 (AC 1.2–1.5), Req 5 (AC 5.1–5.9) |
| 6 | Req 6 (AC 6.1–6.8) |
| 7 | Req 6 (AC 6.4, 6.7), Req 7 (AC 7.1–7.2) |
| 8 | Req 7 (AC 7.1–7.5) |
| 9 | Req 3 (AC 3.1–3.14), Req 4 (AC 4.6–4.7) |
| 10 | Req 3 (AC 3.1, 3.11–3.13) |
| 11 | Req 8 (AC 8.1–8.6) |
| 12 | Req 9 (AC 9.1–9.6) |
| 13 | Req 10 (AC 10.1–10.7) |
| 14 | Req 11 (AC 11.1–11.5) |
| 15 | Req 13 (AC 13.1–13.6) |
| 16 | Error handling (all) |
| 17 | Req 2 (AC 2.1–2.5) |
| 18 | PBT: Req 1, 2, 3, 4 (WrapMode and WrapState invariants) |
| 19 | PBT: Req 1, 5, 6 (line-breaking invariants) |
| 20 | PBT: Req 1, 6 (display-line-mapping height invariants) |
| 21 | PBT: Req 4, 5, 12 (configuration validation invariants) |
| 22 | PBT: Req 11 (session persistence invariants) |
| 23 | PBT: Req 7, 8 (indicator and scrollbar invariants) |
| 24 | PBT: Req 3, 4 (WRAP command invariants) |
| 25 | Integration: Req 1–13 (end-to-end workflows) |

---

## Task Dependency Graph

```json
{
  "taskDependencies": {
    "1": [],
    "2": ["1"],
    "3": ["1"],
    "4": ["2", "3"],
    "5": ["2"],
    "6": ["4", "5"],
    "7": ["4", "6"],
    "8": ["4"],
    "9": ["4", "5", "16"],
    "10": ["9"],
    "11": ["2", "4"],
    "12": ["2", "4"],
    "13": ["2", "5"],
    "14": ["4", "3"],
    "15": ["4", "5"],
    "16": ["1"],
    "17": ["4", "11", "12"],
    "18": ["2", "4"],
    "19": ["5"],
    "20": ["6"],
    "21": ["3"],
    "22": ["4", "14"],
    "23": ["4", "8", "11"],
    "24": ["9"],
    "25": ["4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "17"]
  },
  "externalDependencies": {
    "ff-display-line-mapping": "Provides set_height, doc_from_display, display_from_doc — wrap height updates are communicated through this layer",
    "ff-config": "Provides ConfigStore, key-value configuration access, hot-reload notification — wrap config keys ([view.wrap]) are read from here",
    "ff-command": "Command registry, dispatch, metadata — WRAP command is registered here",
    "ff-viewport": "Consumes WrapViewportEvent to recalculate visible_count, scrollbar range, and adjust top_line",
    "ff-logging": "Structured logging for configuration warnings and diagnostics",
    "ff-multi-tab": "Per-editor-instance lifecycle — each tab owns its own WrapState",
    "ff-session": "Session persistence store — wrap state is serialised/deserialised through this system",
    "ff-statusbar": "Renders WrapIndicatorState data in the status bar UI",
    "ff-whitespace-guides": "Renders wrap visual flag glyphs (continuation markers) using computed WrapMarkerPosition data",
    "ff-idle-processing": "Background wrap height recalculation for large files — incremental computation in idle cycles"
  },
  "waves": [
    {
      "id": 0,
      "label": "Foundation types and configuration",
      "tasks": ["1", "2", "3", "16"],
      "description": "Crate scaffolding, WrapMode/WrapBoundary/WrapIndentMode enums, WrapConfig model, error types"
    },
    {
      "id": 1,
      "label": "State management and line-breaking",
      "tasks": ["4", "5"],
      "description": "WrapState per-instance struct, WrapBreaker trait, word/character breaking algorithms, indent computation",
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "Display-line-mapping and viewport integration",
      "tasks": ["6", "7", "8"],
      "description": "Height calculation, display-line-mapping set_height integration, viewport event emission, scrollbar visibility",
      "dependsOn": [0, 1]
    },
    {
      "id": 3,
      "label": "Commands and input handling",
      "tasks": ["9", "10"],
      "description": "WRAP primary command handler, argument parsing, command registration",
      "dependsOn": [1]
    },
    {
      "id": 4,
      "label": "UI models, persistence, and rendering",
      "tasks": ["11", "12", "13", "14", "15"],
      "description": "Status bar indicator, View menu model, visual flags, session persistence, rendering coordination",
      "dependsOn": [0, 1]
    },
    {
      "id": 5,
      "label": "Integration validation",
      "tasks": ["17"],
      "description": "Per-editor-instance independence validation tests",
      "dependsOn": [1, 4]
    },
    {
      "id": 6,
      "label": "Property-based tests",
      "tasks": ["18", "19", "20", "21", "22", "23", "24"],
      "description": "Property tests validating invariants across mode, breaking, heights, config, persistence, indicator, and commands",
      "dependsOn": [0, 1, 2, 3, 4]
    },
    {
      "id": 7,
      "label": "Integration tests",
      "tasks": ["25"],
      "description": "End-to-end workflow validation covering all requirements",
      "dependsOn": [0, 1, 2, 3, 4, 5, 6]
    }
  ]
}
```
