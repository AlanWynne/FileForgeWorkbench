# Implementation Plan: Tabs and Mask (`ff-tabs-mask`)

## Overview

This plan implements the full tab stop management and insert mask template subsystem for FileForgeWorkbench. The `ff-tabs-mask` crate owns tab stop list management, TABS/MASK display artifact lifecycle, Tab key cursor advancement, mask application to newly inserted lines, shift-to-tab-stop computation for `>` / `<` line commands, and defaults loading from configuration and language definitions.

The crate bridges `ff-command` (command registration/dispatch), `ff-config` (global tab stop/tab size settings), `ff-language-service` (per-language defaults), `ff-document-model` (line width context), and `ff-edit-operations` (Tab key execution, line insertion) via trait interfaces, maintaining GUI independence throughout.

---

## Tasks

- [ ] 1. Crate scaffolding and error types
  - [ ] 1.1 Create `crates/ff-tabs-mask/Cargo.toml` with dependencies: `ff-logging`, `thiserror`, `serde`, `serde_derive`, `toml`; dev-dependencies: `proptest`, `pretty_assertions`, `tempfile`
  - [ ] 1.2 Create `src/lib.rs` with crate-level docs, public re-exports, and module declarations for state, tab_stops, mask, artifacts, tab_key, shift, defaults, commands, error
  - [ ] 1.3 Create `src/error.rs` with `TabsMaskError` enum (InvalidTabStops, InvalidMode, MaskNotEditable, NoMaskToClear, NoActiveMask, InvalidConfig, MaskTruncated, AnchorOutOfRange)
  - [ ] 1.4 Create `src/traits.rs` with upstream trait interfaces: `ConfigProvider` (get_tab_stops, get_tab_size), `LanguageDefinitionRef` (default_tab_stops, default_mask), `DocumentContext` (line_width, line_count, cursor_line)

- [ ] 2. TabStopList type — core data model
  - [ ] 2.1 Create `src/tab_stops.rs` with `TabStopList` struct holding `stops: Vec<u32>` (sorted, deduplicated, all > 0)
  - [ ] 2.2 Implement `TabStopList::empty()` and `TabStopList::from_columns(columns: impl IntoIterator<Item = u32>)` — filters zeros, deduplicates, sorts ascending (Requirements 2.8, 4.7)
  - [ ] 2.3 Implement `TabStopList::every_n_columns(interval: u32, max_column: u32)` — generates every-N-columns stops for built-in default (Requirement 4.2)
  - [ ] 2.4 Implement `next_stop_after(&self, current_column: u32) -> Option<u32>` — returns next stop strictly greater than current_column; past last explicit stop, repeats last interval (Requirements 5.1, 5.2)
  - [ ] 2.5 Implement `prev_stop_before(&self, current_column: u32) -> Option<u32>` — returns previous stop strictly less than current_column; None if no stop to the left (Requirements 14.2, 14.3)
  - [ ] 2.6 Implement `nth_stop_after(&self, current_column: u32, n: u32) -> Option<u32>` and `nth_stop_before(&self, current_column: u32, n: u32) -> Option<u32>` — advance/retreat by n stops (Requirement 14.4)
  - [ ] 2.7 Implement accessor methods: `stops()`, `is_empty()`, `len()`, `contains(column)`, and `Display` trait formatting as space-separated column numbers
  - [ ] 2.8 Write unit tests for TabStopList: construction with zeros/duplicates, every_n_columns, next/prev stop computation, nth_stop boundary cases, empty list behaviour

- [ ] 3. MaskLine type — mask content model
  - [ ] 3.1 Create `src/mask.rs` with `MaskLine` struct holding `content: String`
  - [ ] 3.2 Implement `MaskLine::new(content)`, `MaskLine::empty()`, `is_empty()`, `content()`, `len()`, `set_content()` (Requirements 6.4, 10.4)
  - [ ] 3.3 Implement `apply_to_width(&self, line_width: usize) -> String` — pads with spaces if shorter, truncates if longer (Requirements 9.5, 9.6)
  - [ ] 3.4 Implement `Display` trait for MaskLine
  - [ ] 3.5 Write unit tests for MaskLine: empty mask, apply_to_width padding, apply_to_width truncation, verbatim content preservation, set_content update

- [ ] 4. TabsMaskState — per-session state model
  - [ ] 4.1 Create `src/state.rs` with `TabsState` struct (tab_stops: TabStopList, source: TabStopSource, default_tab_stops: TabStopList) and `TabStopSource` enum (BuiltIn, GlobalConfig, LanguageDefinition, SessionOverride)
  - [ ] 4.2 Implement `TabsState::new()`, `tab_stops()`, `set_tab_stops()` (sets source to SessionOverride), `reset_to_defaults()`, `source()` (Requirements 2.1, 2.4, 12.1, 12.2)
  - [ ] 4.3 Implement `MaskState` struct (mask: Option<MaskLine>, from_language: bool) with `with_mask()`, `empty()`, `mask()`, `is_active()`, `update_mask()`, `clear()` (Requirements 6.4, 7.1, 10.1, 10.2, 10.5)
  - [ ] 4.4 Implement `ArtifactPosition` struct (anchor_line: usize, from_line_command: bool) for tracking display artifact locations
  - [ ] 4.5 Implement `TabsMaskState` struct combining TabsState, MaskState, tabs_lines: Vec<ArtifactPosition>, mask_lines: Vec<ArtifactPosition>
  - [ ] 4.6 Implement TabsMaskState methods: `add_tabs_line()`, `remove_all_tabs_lines()`, `add_mask_line()`, `remove_all_mask_lines()`, `has_tabs_lines()`, `has_mask_lines()`, `tabs_lines()`, `mask_lines()` (Requirements 1.1, 1.4, 1.7, 6.1, 6.5, 6.8, 11.1, 11.2)
  - [ ] 4.7 Write unit tests for state model: TabsState set/reset, MaskState clear/update, TabsMaskState artifact add/remove, toggle logic

- [ ] 5. Tab key handler — cursor advancement computation
  - [ ] 5.1 Create `src/tab_key.rs` with `TabKeyAction` enum (InsertSpacesTo, MoveCursorTo, DelegateToIndent, StandardNavigation, AdvanceBySize) and `EditMode` enum (Insert, Overstrike, Browse, View)
  - [ ] 5.2 Implement `compute_tab_action(tab_stops, current_column, mode, has_selection, tab_size, line_width) -> TabKeyAction` — full Tab key decision logic (Requirements 5.1–5.6)
  - [ ] 5.3 Implement Insert mode path: advance to next stop, insert spaces to fill (Requirement 5.5)
  - [ ] 5.4 Implement Overstrike mode path: move cursor without inserting characters (Requirement 5.6)
  - [ ] 5.5 Implement Browse/View mode path: return StandardNavigation (Requirement 5.4)
  - [ ] 5.6 Implement selection-active path: return DelegateToIndent (Requirement 5.4)
  - [ ] 5.7 Implement empty tab stop list fallback: advance by tab_size (Requirement 5.3)
  - [ ] 5.8 Write unit tests for tab key: each EditMode, with/without selection, empty list fallback, past-last-stop extension, line width clamping

- [ ] 6. TABS primary command handler
  - [ ] 6.1 Create `src/commands/tabs.rs` with `execute_tabs_command(state, args, cursor_line, line_width) -> Result<TabsCommandResult>` and `TabsCommandResult` enum (LinesAdded, LinesRemoved, StopsUpdated)
  - [ ] 6.2 Implement no-args path: toggle TABS_Lines — if displayed remove all, if none displayed insert at cursor (Requirements 1.1, 1.4, 1.7)
  - [ ] 6.3 Implement column arguments path: parse args via `TabStopManager::parse_tab_stops()`, validate positive integers, replace active tab stops, update displayed TABS_Lines (Requirements 2.1–2.8)
  - [ ] 6.4 Implement `TabStopManager::parse_tab_stops(args: &[&str]) -> Result<TabStopList>` — returns InvalidTabStops error for non-positive/non-integer values (Requirement 2.7)
  - [ ] 6.5 Implement non-undoable classification: command does not create undo transactions (Requirements 1.9, 2.6)
  - [ ] 6.6 Implement mode validation: TABS valid in Browse, Edit, and View modes (Requirement 1.10)
  - [ ] 6.7 Write unit tests for TABS command: no-args toggle on/off, column args set stops, invalid column rejection, multiple TABS_Lines at different positions, mode validation

- [ ] 7. MASK primary command handler (including MASK OFF)
  - [ ] 7.1 Create `src/commands/mask.rs` with `execute_mask_command(state, args, cursor_line, line_width) -> Result<MaskCommandResult>` and `MaskCommandResult` enum (LinesAdded, LinesRemoved, MaskCleared, NoActiveMask, NoMaskToClear)
  - [ ] 7.2 Implement no-args path with active mask: toggle MASK_Lines — if displayed remove all, if none displayed insert at cursor (Requirements 6.1, 6.5, 6.8)
  - [ ] 7.3 Implement no-args path with no active mask: return NoActiveMask status message (Requirement 6.2)
  - [ ] 7.4 Implement `MASK OFF` path: clear mask from Session_State, remove all MASK_Lines (Requirements 7.1, 7.2)
  - [ ] 7.5 Implement `MASK OFF` with no active mask: return NoMaskToClear status message (Requirement 7.3)
  - [ ] 7.6 Implement non-undoable classification: MASK display and content changes not recorded as undo transactions (Requirements 6.10, 7.4)
  - [ ] 7.7 Implement mode validation: MASK valid in Edit (display+editable) and Browse (display-only) modes (Requirement 6.11)
  - [ ] 7.8 Write unit tests for MASK command: toggle with active mask, no-active-mask message, MASK OFF clears, MASK OFF with no mask, Browse mode read-only

- [ ] 8. TABS and MASK line commands
  - [ ] 8.1 Create `src/commands/line_commands.rs` with `execute_line_command(state, kind: ArtifactKind, anchor_line, line_width) -> Result<()>`
  - [ ] 8.2 Implement TABS line command: insert TABS_Line immediately above target line, reflecting active tab stops (Requirements 3.1, 3.2)
  - [ ] 8.3 Implement MASK line command: insert editable MASK_Line immediately above target line, reflecting active mask (Requirements 8.1, 8.2, 8.3)
  - [ ] 8.4 Implement non-undoable classification for both line commands (Requirements 3.5, 8.6)
  - [ ] 8.5 Implement prefix area indicator: TABS_Line gets `TABS` prefix, MASK_Line gets `MASK` prefix, both non-editable (Requirements 1.8, 3.4, 6.9, 8.5)
  - [ ] 8.6 Write unit tests for line commands: TABS insertion at position, MASK insertion at position, prefix indicators, anchor validation

- [ ] 9. Mask application to inserted lines
  - [ ] 9.1 Implement `MaskManager::apply_mask(mask_state, line_width) -> Option<String>` — returns mask content padded/truncated to line width, or None if no mask active (Requirements 9.1, 9.3, 9.5, 9.6)
  - [ ] 9.2 Implement `MaskManager::apply_mask_to_n_lines(mask_state, line_width, count) -> Vec<String>` — produces n identical mask-filled lines for In command (Requirement 9.2)
  - [ ] 9.3 Implement transaction integration: mask content is returned to I/In execution path as initial line content, sharing the insert transaction — no separate undo entry (Requirement 9.4)
  - [ ] 9.4 Write unit tests for mask application: single line with active mask, n lines with active mask, no active mask returns None/empty, truncation at line width, padding to line width

- [ ] 10. Shift-to-tab-stop for `>` and `<` commands
  - [ ] 10.1 Create `src/shift.rs` with `ShiftAction` struct (target_column: u32, delta: i32)
  - [ ] 10.2 Implement `compute_shift_right(tab_stops, first_nonspace_column, count) -> ShiftAction` — shifts content rightward to next tab stop position(s) (Requirements 14.1, 14.4)
  - [ ] 10.3 Implement `compute_shift_left(tab_stops, first_nonspace_column, count) -> ShiftAction` — shifts content leftward to previous tab stop position(s); floors at column 1 (Requirements 14.2, 14.3, 14.4)
  - [ ] 10.4 Implement indent delegation guard: Tab with selection delegates to auto-indentation using editor.indent_size, NOT the TABS tab stop list (Requirement 14.5)
  - [ ] 10.5 Write unit tests for shift: right by 1, right by n, left by 1, left by n, left past column 1, empty tab stop list behaviour

- [ ] 11. Defaults loader — configuration and language integration
  - [ ] 11.1 Create `src/defaults.rs` with `DefaultsLoader` struct
  - [ ] 11.2 Implement `load_tab_stops(config, language_def, max_column) -> (TabStopList, TabStopSource)` — precedence: language definition > global config > every-8-columns (Requirements 4.1–4.7, 13.1–13.6)
  - [ ] 11.3 Implement `load_mask(language_def) -> MaskState` — reads `default_mask` key; returns empty if absent or invalid type (Requirements 10.1–10.6)
  - [ ] 11.4 Implement `init_session(config, language_def, max_column) -> TabsMaskState` — combines tab stops and mask loading for full session initialization (Requirements 4, 10, 15)
  - [ ] 11.5 Implement invalid value handling: log warnings for non-positive integers in tab stops and non-string mask values, skip invalid entries (Requirements 4.6, 10.6, 13.3)
  - [ ] 11.6 Implement hot-reload boundary: new defaults apply only to newly opened sessions, not retroactively (Requirement 13.7)
  - [ ] 11.7 Implement `MaskManager::from_language_default(value: &toml::Value) -> Option<MaskLine>` — validates type and creates MaskLine (Requirements 10.3, 10.6)
  - [ ] 11.8 Write unit tests for defaults: language def takes precedence, global config used when no language def, every-8-columns fallback, invalid values filtered with warning, mask loaded verbatim, non-string mask rejected

- [ ] 12. RESET interaction handling
  - [ ] 12.1 Implement `handle_reset(state: &mut TabsMaskState)` — removes all TABS_Lines and MASK_Lines from viewport, preserves tab stops and mask content (Requirements 11.1, 11.2, 11.3, 11.4)
  - [ ] 12.2 Create `src/commands/reset_tabs.rs` with `execute_reset_tabs(state, line_width) -> Result<()>` — restores default tab stops per precedence rules, updates displayed TABS_Lines (Requirements 12.1, 12.2, 12.3)
  - [ ] 12.3 Implement RESET TABS non-undoable classification (Requirement 12.4)
  - [ ] 12.4 Implement RESET COMMANDS handling: clear pending TABS/MASK line commands from prefix area without removing already-inserted artifacts or clearing state (Requirement 11.5)
  - [ ] 12.5 Write unit tests for RESET: removes all display lines, tab stops unchanged, mask unchanged, RESET TABS restores defaults, RESET COMMANDS clears pending only

- [ ] 13. Display artifact lifecycle and rendering
  - [ ] 13.1 Create `src/artifacts.rs` with `DisplayArtifactManager` struct, `ArtifactKind` enum (TabsLine, MaskLine), `ArtifactMetadata` struct, `UndoClassification` enum, `EditorMode` enum
  - [ ] 13.2 Implement `render_tabs_line(tab_stops, line_width, indicator_char, filler_char) -> String` — places indicator at each stop, filler elsewhere, extends to full viewport width (Requirements 1.2, 1.3, 17.1–17.5)
  - [ ] 13.3 Implement `render_mask_line(mask, line_width) -> String` — displays mask content padded to line width (Requirements 6.3, 16.1, 16.4)
  - [ ] 13.4 Implement `should_toggle_off(existing_lines) -> bool` — returns true if lines already displayed (Requirements 1.4, 6.5)
  - [ ] 13.5 Implement `artifact_metadata(kind) -> ArtifactMetadata` — returns command registration metadata including category "display", non-undoable classification, applicable modes (Requirement 18.7)
  - [ ] 13.6 Implement display artifact exclusion properties: TABS_Lines and MASK_Lines are not real document lines — not counted in line numbers, not included in command scope, not saved to disk (Requirements 18.1–18.4)
  - [ ] 13.7 Implement scrolling anchor behaviour: artifacts remain visually anchored to their document position as the viewport scrolls (Requirements 1.6, 6.7)
  - [ ] 13.8 Write unit tests for artifacts: render_tabs_line with various stops, render_mask_line padding, toggle logic, metadata generation, artifact exclusion from scope

- [ ] 14. Command registration and metadata
  - [ ] 14.1 Create `src/commands/mod.rs` with `register_commands(registry: &mut dyn CommandRegistry)` function
  - [ ] 14.2 Register `edit.tabs` primary command — valid in Edit, Browse, View modes (Requirement 1.10)
  - [ ] 14.3 Register `edit.mask` primary command — valid in Edit (editable), Browse (display-only) modes (Requirement 6.11)
  - [ ] 14.4 Register `edit.mask_off` primary command — valid in Edit mode (Requirement 7)
  - [ ] 14.5 Register `edit.reset_tabs` primary command — valid in Edit mode (Requirement 12)
  - [ ] 14.6 Register TABS line command in line-command pipeline (Requirement 3)
  - [ ] 14.7 Register MASK line command in line-command pipeline (Requirement 8)
  - [ ] 14.8 Include TABS and MASK in Display Helper Line Commands for HELP LINECOMMANDS output (Requirement 18.5)
  - [ ] 14.9 Implement execution ordering: primary command executes before display artifact insertion when both are pending (Requirement 18.6)
  - [ ] 14.10 Write unit tests for registration: all commands registered, metadata correct, mode validation, execution ordering

- [ ] 15. Configuration integration
  - [ ] 15.1 Implement `editor.default_tab_stops` config key support — TOML array of positive integers (Requirements 4.1, 13.1, 13.2)
  - [ ] 15.2 Implement missing config fallback: when `editor.default_tab_stops` absent, behave as `[]` and fall back to every-8-columns (Requirements 4.2, 13.2)
  - [ ] 15.3 Implement language TOML `default_tab_stops` key support — array of positive integers (Requirements 4.5, 13.4)
  - [ ] 15.4 Implement language TOML `default_mask` key support — plain string value (Requirements 10.3, 13.5)
  - [ ] 15.5 Implement language precedence: Language_Definition tab stops override global config for files of that language type (Requirements 4.3, 4.4, 13.6)
  - [ ] 15.6 Implement `editor.tab_size` fallback: when tab stop list is empty, Tab key advances by tab_size (Requirement 5.3)
  - [ ] 15.7 Write unit tests for config integration: TOML array parsing, missing key fallback, language override precedence, tab_size fallback, invalid entry filtering

- [ ] 16. Per-session state invariants (non-undoable, non-persisted)
  - [ ] 16.1 Implement Session_State storage: TabStopList and InsertMask are per-session only, not part of document model or undo history (Requirements 15.1, 15.2)
  - [ ] 16.2 Implement undo/redo isolation: undo/redo operations never modify tab stops or mask (Requirement 15.3)
  - [ ] 16.3 Implement save exclusion: tab stops and mask never persisted to file on save (Requirement 15.4)
  - [ ] 16.4 Implement session reinitialisation: close/reopen reinitialises from defaults, not previous session state (Requirement 15.5)
  - [ ] 16.5 Implement MASK OFF permanence: clearing the mask during session is permanent for that session even if language definition provides a default (Requirement 10.5)
  - [ ] 16.6 Write unit tests for session invariants: state not affected by undo, not persisted on save, reinitialised on reopen, MASK OFF permanent within session

- [ ] 17. Property-based tests — Correctness Properties
  - [ ] 17.1 Write property test: Tab Stop List Sorted and Deduplicated Invariant (Property 1) — for any input columns, resulting TabStopList is sorted ascending with no duplicates and all > 0
    - **Validates: Requirements 2.8, 4.7**
  - [ ] 17.2 Write property test: Next Tab Stop Monotonically Advances (Property 2) — for any non-empty list and current column, next_stop_after returns value strictly greater than current_column
    - **Validates: Requirements 5.1**
  - [ ] 17.3 Write property test: Previous Tab Stop Monotonically Retreats (Property 3) — for any non-empty list and current column > 1, prev_stop_before returns value strictly less than current_column
    - **Validates: Requirements 14.2, 14.3**
  - [ ] 17.4 Write property test: Mask Application Width Invariant (Property 4) — applying any mask to any line_width > 0 always produces a string of exactly that width
    - **Validates: Requirements 9.5, 9.6**
  - [ ] 17.5 Write property test: Tab Key Insert Mode Space Count (Property 5) — InsertSpacesTo target is always strictly greater than current_column
    - **Validates: Requirements 5.5**
  - [ ] 17.6 Write property test: Tab Stops Persist Across RESET (Property 6) — handle_reset removes all artifact lines but tab stop list and mask content remain unchanged
    - **Validates: Requirements 11.3, 11.4**
  - [ ] 17.7 Write property test: RESET TABS Restores Defaults (Property 7) — after any number of session overrides, reset_to_defaults restores the original default list
    - **Validates: Requirements 12.1**
  - [ ] 17.8 Write property test: MASK OFF Clears Regardless of Source (Property 8) — after clear(), mask is_active returns false and mask() returns None regardless of origin
    - **Validates: Requirements 7.1, 10.5**
  - [ ] 17.9 Write property test: Tab Stop List Filters Invalid Values (Property 9) — zeros and duplicates are never present; result length equals count of distinct positive values in input
    - **Validates: Requirements 2.7, 2.8, 4.6**
  - [ ] 17.10 Write property test: Toggle Behaviour Idempotence (Property 10) — issuing TABS twice returns display to original state with no artifact lines remaining
    - **Validates: Requirements 1.4, 6.5**
  - [ ] 17.11 Write property test: Shift Right Then Shift Left Returns to Original (Property 11) — for columns at tab stop positions, shift_right(1) then shift_left(1) returns to original column
    - **Validates: Requirements 14.1, 14.2**
  - [ ] 17.12 Write property test: Language Definition Precedence Over Global Config (Property 12) — when both language def and global config provide stops, language def values are used
    - **Validates: Requirements 4.3, 4.4, 13.6**
  - [ ] 17.13 Write property test: Display Artifact Lines Excluded from Command Scope (Property 13) — TABS_Lines and MASK_Lines are never real document lines and metadata confirms non-document status
    - **Validates: Requirements 18.1, 18.2, 18.3, 18.4**
  - [ ] 17.14 Write property test: Mask Application Part of Insert Transaction (Property 14) — apply_mask returns content for embedding in insertion, never creates a separate transaction
    - **Validates: Requirements 9.4**

- [ ] 18. Integration tests
  - [ ] 18.1 Write end-to-end test: Session initialization with COBOL language profile — loads per-language tab stops and default mask, verifies TabsMaskState populated correctly
  - [ ] 18.2 Write end-to-end test: TABS command then Tab key — set tab stops via TABS command, press Tab, verify cursor advances to correct column in Insert mode
  - [ ] 18.3 Write end-to-end test: TABS command then Tab key in Overstrike mode — verify cursor moves without inserting characters
  - [ ] 18.4 Write end-to-end test: MASK command then I line command — display mask, insert new line via I, verify inserted line contains mask content
  - [ ] 18.5 Write end-to-end test: MASK edit in place — display MASK_Line, modify mask content via editing, verify subsequent I inserts use updated mask
  - [ ] 18.6 Write end-to-end test: MASK OFF then I — clear mask with MASK OFF, insert new line, verify it is blank
  - [ ] 18.7 Write end-to-end test: RESET clears display but preserves state — add TABS and MASK lines, issue RESET, verify lines removed but tab stops and mask remain active
  - [ ] 18.8 Write end-to-end test: RESET TABS restores language defaults — override tab stops with TABS command, issue RESET TABS, verify defaults restored and display updated
  - [ ] 18.9 Write end-to-end test: Shift right/left with tab stops — set tab stops, execute `>` and `<` on lines, verify content shifts to tab stop columns
  - [ ] 18.10 Write end-to-end test: Multiple TABS_Lines at different positions — issue TABS at multiple cursor positions, verify separate TABS_Lines exist, toggle removes all
  - [ ] 18.11 Write end-to-end test: Tab key with empty stop list — no tab stops configured, press Tab, verify cursor advances by tab_size
  - [ ] 18.12 Write end-to-end test: Language definition precedence over global config — configure both global and language tab stops, open file, verify language stops used
  - [ ] 18.13 Write end-to-end test: MASK in Browse mode — issue MASK in Browse mode, verify displayed but not editable (read-only)
  - [ ] 18.14 Write end-to-end test: In command with active mask — insert 3 lines via I3, verify all 3 lines contain mask content, undo removes all 3 as one transaction

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered By Task(s) |
|-------------|----------|---------------------|
| Req 1: TABS Primary — Display/Toggle | 1.1–1.10 | 6.1–6.7, 13.2, 13.7, 17.10 |
| Req 2: TABS Primary — Configure | 2.1–2.8 | 2.1–2.8, 6.3–6.4, 17.1, 17.9 |
| Req 3: TABS Line Command | 3.1–3.5 | 8.1–8.6 |
| Req 4: Default Tab Stops / Language Integration | 4.1–4.7 | 11.1–11.8, 15.1–15.5, 17.1, 17.12 |
| Req 5: Tab Key Behaviour | 5.1–5.6 | 5.1–5.8, 17.2, 17.5 |
| Req 6: MASK Primary — Display/Toggle | 6.1–6.11 | 7.1–7.8, 13.3, 13.7, 17.10 |
| Req 7: MASK OFF — Clear Mask | 7.1–7.4 | 7.4–7.5, 17.8 |
| Req 8: MASK Line Command | 8.1–8.6 | 8.1–8.6 |
| Req 9: Mask Applied to Inserted Lines | 9.1–9.6 | 9.1–9.4, 17.4, 17.14 |
| Req 10: Default Mask / Language Integration | 10.1–10.6 | 11.3, 11.5, 11.7, 15.4, 17.8 |
| Req 11: RESET Interaction | 11.1–11.5 | 12.1–12.5, 17.6 |
| Req 12: RESET TABS — Restore Defaults | 12.1–12.4 | 12.2–12.3, 17.7 |
| Req 13: Configurable Defaults Per Language | 13.1–13.7 | 11.2, 11.6, 15.1–15.7, 17.12 |
| Req 14: TABS Interaction with Shift Commands | 14.1–14.5 | 10.1–10.5, 17.3, 17.11 |
| Req 15: Per-Session State (Non-Undoable) | 15.1–15.5 | 4.1–4.7, 16.1–16.6, 17.6 |
| Req 16: MASK as Visual Aid | 16.1–16.4 | 13.3, 7.7, 3.3 |
| Req 17: TABS Display Ruler | 17.1–17.5 | 13.2, 6.3 |
| Req 18: Display_Artifact_Line Compatibility | 18.1–18.7 | 13.5–13.6, 14.1–14.10, 17.13 |

---

## Notes

- This crate has zero GUI dependencies — all functionality is testable via unit and property-based tests against the public API
- The crate depends on `ff-logging`, `thiserror`, `serde`, `serde_derive`, `toml`, and the standard library. Upstream crates (`ff-command`, `ff-config`, `ff-language-service`, `ff-document-model`, `ff-edit-operations`) are connected via trait interfaces defined in `src/traits.rs`
- Property tests use `proptest` crate with a minimum of 100 iterations per property
- The TabStopList (Task 2) is the foundation for Tab key handling (Task 5), shift computation (Task 10), and TABS display rendering (Task 13)
- The MaskLine (Task 3) is the foundation for mask application (Task 9) and MASK display rendering (Task 13)
- DefaultsLoader (Task 11) depends on configuration integration (Task 15) and the core types (Tasks 2, 3)
- Command handlers (Tasks 6, 7, 8) depend on the state model (Task 4) and core types (Tasks 2, 3)
- RESET interaction (Task 12) depends on the state model (Task 4) and display artifact manager (Task 13)
- The `DocumentContext` trait enables document model integration without a compile-time dependency on `ff-document-model`
- TABS/MASK line commands follow the same pattern as COLS/BNDS line commands from `navigation-commands`
- MASK_Line editability is a downstream rendering concern — this crate manages the state update when content changes
- The TABS tab stop list is distinct from `editor.indent_size` used by `auto-indentation`: Tab stops are for single-cursor Tab navigation, indent_size is for selection indent/unindent

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Crate scaffolding and error types", "tasks": ["1.1", "1.2", "1.3", "1.4"] },
    { "id": 1, "label": "Core data models", "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8", "3.1", "3.2", "3.3", "3.4", "3.5"], "dependsOn": [0] },
    { "id": 2, "label": "Per-session state model", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7"], "dependsOn": [1] },
    { "id": 3, "label": "Tab key handler and shift logic", "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "10.1", "10.2", "10.3", "10.4", "10.5"], "dependsOn": [1] },
    { "id": 4, "label": "Display artifact lifecycle", "tasks": ["13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7", "13.8"], "dependsOn": [1, 2] },
    { "id": 5, "label": "Command handlers", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8", "8.1", "8.2", "8.3", "8.4", "8.5", "8.6"], "dependsOn": [2, 4] },
    { "id": 6, "label": "Mask application and RESET", "tasks": ["9.1", "9.2", "9.3", "9.4", "12.1", "12.2", "12.3", "12.4", "12.5"], "dependsOn": [2, 5] },
    { "id": 7, "label": "Defaults loader and configuration", "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "11.8", "15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "15.7"], "dependsOn": [1, 2] },
    { "id": 8, "label": "Command registration and session invariants", "tasks": ["14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "14.7", "14.8", "14.9", "14.10", "16.1", "16.2", "16.3", "16.4", "16.5", "16.6"], "dependsOn": [5, 6, 7] },
    { "id": 9, "label": "Property-based tests", "tasks": ["17.1", "17.2", "17.3", "17.4", "17.5", "17.6", "17.7", "17.8", "17.9", "17.10", "17.11", "17.12", "17.13", "17.14"], "dependsOn": [8] },
    { "id": 10, "label": "Integration tests", "tasks": ["18.1", "18.2", "18.3", "18.4", "18.5", "18.6", "18.7", "18.8", "18.9", "18.10", "18.11", "18.12", "18.13", "18.14"], "dependsOn": [9] }
  ]
}
```
