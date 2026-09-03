# Implementation Plan: Sequence Numbers (`ff-sequence-numbers`)

## Overview

This plan implements the full sequence number subsystem for FileForgeWorkbench. The `ff-sequence-numbers` crate owns detection, stripping, re-insertion, display overlay, and save-time preservation of legacy sequence numbers found in mainframe source files (COBOL, JCL, FORTRAN, PL/I).

The crate bridges `ff-language-service` (column definitions), `ff-document-model` (edit buffer access), `ff-command` (command registration), `ff-undo` (transaction recording), and `ff-config` (detection rules and save behaviour) via trait interfaces, maintaining GUI independence throughout.

---

## Tasks

- [x] 1. Crate scaffolding and core types
  - [x] 1.1 Create `crates/ff-sequence-numbers/Cargo.toml` with dependencies: `ff-logging`, `thiserror`, `serde`, `serde_derive`; dev-dependencies: `proptest`, `pretty_assertions`, `tempfile`
  - [x] 1.2 Create `src/lib.rs` with crate-level docs, public re-exports, and module declarations
  - [x] 1.3 Create `src/error.rs` with `SeqNumError` enum (InvalidColumnRange, NoSequenceColumns, PrefixTooLong, OverflowWarning, GridEditModeNotAllowed, ConfigOutOfRange)
  - [x] 1.4 Create `src/types.rs` with `ColumnRange` struct (start: u32, end: u32), `SequenceFormat` enum (Numeric, AlphaPrefix { prefix: String }), and `DetectionResult` enum (Present, Absent)
  - [x] 1.5 Create `src/config.rs` with `SeqNumConfig` struct (detection_threshold, sample_size, highlight_columns, default_format, restore_on_save) and validation logic clamping threshold to 50–100 (Requirements 2.8, 12.1)
  - [x] 1.6 Create `src/traits.rs` with `DocumentAccess` trait (line_count, line_content, replace_columns), `LanguageProfile` trait (sequence_cols_front, sequence_cols_back, auto_unnum, language_id), and `UndoRecorder` trait (begin_sequence_transaction, record_column_change, commit, abort)

- [x] 2. Column range parsing and validation
  - [x] 2.1 Create `src/column_range.rs` with `ColumnRange::parse(s: &str) -> Result<ColumnRange>` parsing `"start-end"` format (Requirement 1.1, 1.2)
  - [x] 2.2 Implement validation: start ≤ end, both > 0; return `Err(InvalidColumnRange)` for malformed values (Requirement 1.4)
  - [x] 2.3 Implement `ColumnRange::width(&self) -> u32` helper returning end - start + 1
  - [x] 2.4 Write unit tests for valid parses (`"1-6"`, `"73-80"`, `"1-5"`), invalid parses (`"0-6"`, `"8-3"`, `"abc"`, `""`)

- [x] 3. Sequence number detection engine
  - [x] 3.1 Create `src/detector.rs` with `SequenceDetector` struct holding `SeqNumConfig`
  - [x] 3.2 Implement `detect_range(&self, lines: &[&str], range: &ColumnRange) -> DetectionResult` — samples up to `sample_size` non-blank lines, checks numeric criterion against threshold (Requirements 2.1, 2.2)
  - [x] 3.3 Implement numeric criterion: column range fully populated with digit or space chars, at least one line all-digits in that range (Requirement 2.2)
  - [x] 3.4 Implement short-file rule: require 100% match when fewer than 5 non-blank lines (Requirement 2.3)
  - [x] 3.5 Implement short-line handling: lines shorter than range end column do not match (Requirement 2.5)
  - [x] 3.6 Implement independent front/back evaluation: `detect(&self, lines: &[&str], profile: &dyn LanguageProfile) -> (DetectionResult, DetectionResult)` evaluating front and back independently (Requirement 2.4)
  - [x] 3.7 Implement alphanumeric prefix detection: consistent alphabetic prefix followed by digits across threshold (Requirement 2.9)
  - [x] 3.8 Implement read-only guarantee: detection never modifies input data (Requirement 2.7)
  - [x] 3.9 Write unit tests for detection: COBOL with valid seq nums, file without seq nums, short file (<5 lines), lines shorter than range, alphanumeric prefix pattern

- [x] 4. SeqNumState model and side-table storage
  - [x] 4.1 Create `src/state.rs` with `SeqNumState` struct per document: stripped_front (Option<ColumnRange>), stripped_back (Option<ColumnRange>), side_table (HashMap<usize, SideTableEntry>), auto_numbering_active (bool), number_show_active (bool)
  - [x] 4.2 Define `SideTableEntry` struct: front_content (Option<String>), back_content (Option<String>) — stores original stripped values per line (Requirement 3.9)
  - [x] 4.3 Implement `store_stripped_values(&mut self, line_idx: usize, front: Option<&str>, back: Option<&str>)` — populates side-table during strip operations
  - [x] 4.4 Implement `get_original_values(&self, line_idx: usize) -> Option<&SideTableEntry>` — retrieves stored values for NUMBER SHOW overlay
  - [x] 4.5 Implement `clear_side_table(&mut self)` — releases all stored original values
  - [x] 4.6 Write unit tests for side-table store/retrieve/clear operations

- [x] 5. Strip engine — core column clearing logic
  - [x] 5.1 Create `src/strip.rs` with `strip_columns(line: &str, range: &ColumnRange) -> String` — replaces column range bytes with spaces; lines shorter than range start are unchanged (Requirements 3.1, 3.2)
  - [x] 5.2 Implement `strip_document(doc: &mut dyn DocumentAccess, ranges: &[ColumnRange], state: &mut SeqNumState) -> usize` — strips all lines, stores originals in side-table, returns count of modified lines (Requirements 3.1, 3.9)
  - [x] 5.3 Implement skip-if-already-blank: lines where range is entirely spaces are left unchanged and not counted (Requirement 5.8)
  - [x] 5.4 Implement scoped strip: `strip_range(doc: &mut dyn DocumentAccess, ranges: &[ColumnRange], start_line: usize, end_line: usize, state: &mut SeqNumState) -> usize` — restricts operation to CC block (Requirement 5.7)
  - [x] 5.5 Write unit tests for strip: single range, both ranges, already-blank lines skipped, lines shorter than range, scoped range

- [x] 6. Auto-strip on file open
  - [x] 6.1 Create `src/auto_strip.rs` with `auto_strip_on_open(doc: &mut dyn DocumentAccess, profile: &dyn LanguageProfile, config: &SeqNumConfig, state: &mut SeqNumState) -> AutoStripResult` orchestration function
  - [x] 6.2 Implement auto-strip flow: check auto_unnum flag → run detector → if present, strip and store originals → return status message (Requirements 3.1, 3.4, 3.6)
  - [x] 6.3 Implement `AutoStripResult` enum: Stripped { front: Option<ColumnRange>, back: Option<ColumnRange>, message: String }, Detected { message: String }, NoSequenceNumbers, NoColumnsConfigured
  - [x] 6.4 Implement non-undoable classification: auto-strip does NOT record undo transaction (Requirement 3.5)
  - [x] 6.5 Implement BOUNDS preservation: strip does not modify any BOUNDS state (Requirement 3.8)
  - [x] 6.6 Write unit tests for auto-strip: enabled + detected, enabled + not detected, disabled + detected (message only), no columns configured

- [x] 7. NUMBER SHOW display mode
  - [x] 7.1 Create `src/number_show.rs` with `NumberShowMode` struct: active (bool)
  - [x] 7.2 Implement `toggle(&mut self) -> bool` — toggles state, returns new value (Requirement 8.1)
  - [x] 7.3 Implement `get_overlay_content(&self, state: &SeqNumState, line_idx: usize) -> Option<OverlayEntry>` — returns original values from side-table for viewport rendering (Requirement 8.2)
  - [x] 7.4 Define `OverlayEntry` struct: front_text (Option<String>), back_text (Option<String>), indicating content for overlay rendering
  - [x] 7.5 Implement no-effect rule: when no stripping occurred, overlay returns None (Requirement 8.7)
  - [x] 7.6 Implement non-undoable classification: toggle does not affect undo stack (Requirement 8.6)
  - [x] 7.7 Write unit tests for NUMBER SHOW: toggle on/off, overlay retrieval with stripped data, overlay with no stripped data, display-only guarantee

- [x] 8. NUMBER command — sequence generation engine
  - [x] 8.1 Create `src/number.rs` with `NumberEngine` struct
  - [x] 8.2 Implement `generate_sequence(width: u32, start: u32, increment: u32, count: usize, format: &SequenceFormat) -> Vec<String>` — produces zero-padded numeric or alpha-prefix sequences (Requirements 6.6, 7.1, 7.2)
  - [x] 8.3 Implement overflow detection: when sequence value exceeds column width, truncate and flag warning (Requirement 6.11)
  - [x] 8.4 Implement alpha-prefix validation: prefix_length + 1 must not exceed column width (Requirement 7.4)
  - [x] 8.5 Implement `apply_numbering(doc: &mut dyn DocumentAccess, range: &ColumnRange, start: u32, increment: u32, format: &SequenceFormat, scope: Option<(usize, usize)>) -> NumberResult` — writes sequences into column range on all/scoped lines (Requirements 6.3, 6.4, 6.12)
  - [x] 8.6 Implement `NumberResult` struct: lines_modified (usize), overflow_occurred (bool), overflow_message (Option<String>)
  - [x] 8.7 Write unit tests for: numeric generation (6-col, 8-col), alpha-prefix generation, overflow detection, scoped numbering, zero/negative start/increment rejection

- [x] 9. UNNUM command implementation
  - [x] 9.1 Create `src/unnum.rs` with `UnnumCommand` struct implementing command execution
  - [x] 9.2 Implement argument parsing: no args (use profile), `COLS start end`, `FRONT`, `BACK`, `ALL` variants (Requirements 5.2–5.6)
  - [x] 9.3 Implement error cases: no sequence columns defined, FRONT not defined, BACK not defined (Requirements 5.2, 5.4, 5.5)
  - [x] 9.4 Implement undo integration: wrap strip in Sequence_Transaction via UndoRecorder trait (Requirement 5.9)
  - [x] 9.5 Implement status message generation: `UNNUM: N lines modified` (Requirement 5.10)
  - [x] 9.6 Implement Browse mode handling: strip display buffer only, no persisted state change (Requirement 5.11)
  - [x] 9.7 Write unit tests for: each argument variant, error cases, skip-blank lines, undo transaction wrapping, status messages

- [x] 10. NUMBER command implementation
  - [x] 10.1 Create `src/number_cmd.rs` with `NumberCommand` struct implementing command execution
  - [x] 10.2 Implement argument parsing: no args (show usage), `COLS start end`, `STD [start increment]`, `ON`, `OFF`, `SHOW`, `COLS start end FORMAT format` (Requirements 6.2–6.8, 7.3)
  - [x] 10.3 Implement confirmation prompt flow: return `NeedsConfirmation` state requiring YES/NO before buffer modification (Requirement 6.9)
  - [x] 10.4 Implement `NUMBER STD` column resolution: prefer back, fallback to front, error if neither defined (Requirement 6.4)
  - [x] 10.5 Implement auto-numbering state management: `NUMBER ON` / `NUMBER OFF` toggle on SeqNumState (Requirements 6.7, 6.8)
  - [x] 10.6 Implement auto-numbering hook: when ON and line inserted, assign next sequence number to new line (Requirement 6.7)
  - [x] 10.7 Implement undo integration: wrap numbering in Sequence_Transaction via UndoRecorder trait; auto-number insertions join the insert transaction (Requirements 6.10, 9.2, 9.4)
  - [x] 10.8 Implement Grid_Edit_Mode rejection: return error if in Grid_Edit_Mode (Requirement 13.2)
  - [x] 10.9 Write unit tests for: usage display, each sub-command variant, confirmation flow, STD fallback, ON/OFF toggle, auto-numbering on insert, overflow warning, format specification

- [x] 11. Column position configuration and per-language overrides
  - [x] 11.1 Create `src/profile_config.rs` with `ResolvedSequenceConfig` struct merging language profile TOML and configuration-system overrides (Requirement 12.2, 12.3)
  - [x] 11.2 Implement precedence resolution: config-system per-language > config-system global > language profile TOML (Requirement 12.3)
  - [x] 11.3 Implement `resolve_config(profile: &dyn LanguageProfile, config: &SeqNumConfig, language_overrides: Option<&LanguageOverride>) -> ResolvedSequenceConfig`
  - [x] 11.4 Implement `auto_unnum` override: per-language false suppresses auto-stripping regardless of TOML (Requirement 12.4)
  - [x] 11.5 Implement hot-reload support: display settings (highlight_columns, overlay style) apply immediately; detection settings apply to next open (Requirement 12.5)
  - [x] 11.6 Write unit tests for: precedence merge, auto_unnum override, missing overrides fallback to defaults

- [x] 12. Save-time preservation and restoration
  - [x] 12.1 Create `src/save_handler.rs` with `SaveSequenceHandler` struct
  - [x] 12.2 Implement default save behaviour: edit buffer written as-is, stripped columns remain as spaces (Requirement 11.1, 11.2)
  - [x] 12.3 Implement `restore_on_save` mode: when enabled, restore original sequence numbers from side-table into save output without modifying edit buffer (Requirement 11.5)
  - [x] 12.4 Implement restore with modifications: for inserted/modified lines, generate new sequence numbers using detected format/increment; preserve originals for unmodified lines (Requirement 11.6)
  - [x] 12.5 Implement NUMBER ON save: when auto-numbering active, save includes the auto-generated numbers already in the buffer (Requirement 11.3)
  - [x] 12.6 Write unit tests for: default save (no restore), restore_on_save with all-original lines, restore with mixed modified/original, NUMBER ON active save

- [x] 13. Format-specific detection profiles (COBOL, JCL, FORTRAN, PL/I)
  - [x] 13.1 Create `src/profiles.rs` with built-in language profile constants for COBOL: front="1-6", back="73-80", auto_unnum=true (Requirement 1.5)
  - [x] 13.2 Define FORTRAN profile: front="1-5", back="73-80", auto_unnum=true (Requirement 1.6)
  - [x] 13.3 Define JCL profile: no front, back="73-80", auto_unnum=true (Requirement 1.7)
  - [x] 13.4 Define PL/I profile: no front, back="73-80", auto_unnum=true (Requirement 1.8)
  - [x] 13.5 Implement no-columns guard: profiles with no sequence columns defined cause detector to skip (Requirement 1.9)
  - [x] 13.6 Write unit tests for: each profile's column definitions, no-columns profile returns early from detection

- [x] 14. Command registration and dispatch integration
  - [x] 14.1 Create `src/commands.rs` with command registration functions: `register_commands(registry: &mut dyn CommandRegistry)`
  - [x] 14.2 Register `UNNUM` (Command_ID: `sequence.unnum`) — valid in Edit and Browse modes (Requirement 5.1)
  - [x] 14.3 Register `NUMBER` (Command_ID: `sequence.number`) — valid in Edit mode only (Requirement 6.1)
  - [x] 14.4 Register `NUMBER SHOW` (Command_ID: `sequence.number_show`) — valid in Edit and Browse modes (Requirement 8.1)
  - [x] 14.5 Implement command metadata: compatibility matrix entries for UNNUM, UNNUM COLS, UNNUM+CC, NUMBER COLS, NUMBER STD, NUMBER+CC, NUMBER ON/OFF, NUMBER SHOW (Requirements 14.1–14.8)
  - [x] 14.6 Implement Grid_Edit_Mode guard: reject UNNUM/NUMBER with appropriate error in grid mode (Requirement 13.1, 13.2)
  - [x] 14.7 Write unit tests for: command registration, mode validation, compatibility matrix entries, grid mode rejection

- [x] 15. Visual indicators and status bar integration
  - [x] 15.1 Create `src/indicators.rs` with `SeqNumIndicator` enum: Stripped { ranges: String }, Detected, NumberShow, None
  - [x] 15.2 Implement `get_indicator(state: &SeqNumState) -> SeqNumIndicator` — returns appropriate indicator based on current state (Requirements 4.1, 4.2, 4.4)
  - [x] 15.3 Implement indicator text formatting: `SEQNUM 1-6,73-80` for stripped, `SEQNUM?` for detected-not-stripped, `SEQSHOW` for NUMBER SHOW active (Requirements 4.1, 4.2, 4.4)
  - [x] 15.4 Implement column highlighting support flag: expose `should_highlight_columns(config: &SeqNumConfig) -> bool` (Requirement 4.5)
  - [x] 15.5 Write unit tests for: each indicator state, text formatting, highlight flag

- [x] 16. BOUNDS interaction enforcement
  - [x] 16.1 Create `src/bounds.rs` with BOUNDS interaction logic
  - [x] 16.2 Implement no-modify-bounds guarantee: strip/number operations never alter BOUNDS state (Requirements 10.1, 10.2, 10.3)
  - [x] 16.3 Implement overlap detection for NUMBER ON: when sequence columns overlap with active BOUNDS, display warning and disable auto-numbering for overlapping range (Requirement 10.4)
  - [x] 16.4 Write unit tests for: BOUNDS unchanged after strip, BOUNDS unchanged after number, overlap warning generation

- [x] 17. Undo/Redo integration
  - [x] 17.1 Create `src/undo_integration.rs` with `SequenceTransaction` type and integration logic
  - [x] 17.2 Implement UNNUM undo: single transaction wrapping all line modifications; UNDO restores exact original byte content (Requirements 9.1, 9.5)
  - [x] 17.3 Implement NUMBER undo: single transaction wrapping all line modifications; UNDO restores pre-NUMBER column content (Requirements 9.2, 9.6)
  - [x] 17.4 Implement auto-strip non-undoable: auto-strip at open bypasses undo entirely (Requirement 9.3)
  - [x] 17.5 Implement NUMBER ON auto-number undo: auto-assigned sequence numbers undo together with the line insertion (Requirement 9.4)
  - [x] 17.6 Write unit tests for: UNNUM undo restores original, NUMBER undo restores pre-state, auto-strip not on undo stack, auto-number joins insert transaction

- [x] 18. Property-based tests — Correctness Properties
  - [x] 18.1 Write property test: Detection Threshold Consistency (Property 1) — for any set of lines where exactly threshold% have numeric columns, detector reports Present; below threshold reports Absent
    - **Validates: Requirements 2.1, 2.2, 2.8**
  - [x] 18.2 Write property test: Strip Idempotency (Property 2) — stripping an already-stripped document produces no further modifications (lines_modified == 0)
    - **Validates: Requirements 3.2, 5.8**
  - [x] 18.3 Write property test: Strip-Restore Round-Trip (Property 3) — for any document, stripping columns and then restoring from side-table produces byte-identical original content
    - **Validates: Requirements 3.9, 9.5, 11.5**
  - [x] 18.4 Write property test: Number Generation Column Fit (Property 4) — for any (width, start, increment, count) where no overflow occurs, all generated values have exactly `width` characters
    - **Validates: Requirements 6.6, 7.1, 7.2**
  - [x] 18.5 Write property test: Number Overflow Detection (Property 5) — when start + (count-1)*increment exceeds 10^width - 1, overflow is flagged
    - **Validates: Requirement 6.11**
  - [x] 18.6 Write property test: Alpha-Prefix Width Constraint (Property 6) — prefix_len + digits always equals column width; prefix too long is rejected
    - **Validates: Requirements 7.2, 7.4**
  - [x] 18.7 Write property test: Independent Front/Back Detection (Property 7) — detection result for front range is independent of back range content and vice versa
    - **Validates: Requirement 2.4**
  - [x] 18.8 Write property test: Short File Strict Threshold (Property 8) — files with <5 non-blank lines require 100% match; any single non-matching line causes Absent
    - **Validates: Requirement 2.3**
  - [x] 18.9 Write property test: UNNUM-NUMBER Inverse (Property 9) — UNNUM followed by NUMBER STD with same start/increment produces sequential content in the same columns
    - **Validates: Requirements 5.2, 6.4**
  - [x] 18.10 Write property test: Config Clamping Invariant (Property 10) — detection_threshold is always in [50, 100] regardless of input value; out-of-range values are clamped
    - **Validates: Requirement 2.8**
  - [x] 18.11 Write property test: Column Range Validity (Property 11) — parsed ColumnRange always satisfies start <= end and start > 0; invalid strings are rejected
    - **Validates: Requirement 1.4**
  - [x] 18.12 Write property test: Side-Table Completeness (Property 12) — after stripping N lines, side-table has entries for exactly the N modified lines (not blank-skipped ones)
    - **Validates: Requirements 3.9, 5.8**

- [x] 19. Integration tests
  - [x] 19.1 Write end-to-end test: COBOL file open — auto-detect front+back seq nums, auto-strip, verify edit buffer clean, verify side-table populated, verify status message
  - [x] 19.2 Write end-to-end test: JCL file open — auto-detect back-only seq nums, auto-strip cols 73-80, front columns untouched
  - [x] 19.3 Write end-to-end test: UNNUM + UNDO cycle — strip via UNNUM, verify modified count, UNDO restores exact originals
  - [x] 19.4 Write end-to-end test: NUMBER STD + UNDO cycle — number all lines with STD, verify sequential values, UNDO restores pre-number content
  - [x] 19.5 Write end-to-end test: NUMBER SHOW overlay — strip on open, activate NUMBER SHOW, verify overlay returns original values, toggle off returns None
  - [x] 19.6 Write end-to-end test: NUMBER ON auto-numbering — enable NUMBER ON, simulate line insert, verify new line gets next sequence number, UNDO removes line and number together
  - [x] 19.7 Write end-to-end test: restore_on_save — strip on open, enable restore_on_save, simulate save, verify output contains original seq nums for unmodified lines
  - [x] 19.8 Write end-to-end test: no sequence columns language — open file with no seq cols defined, verify detector does not run, no strip occurs
  - [x] 19.9 Write end-to-end test: configuration override — set per-language auto_unnum=false, open COBOL file, verify detection runs but no stripping occurs, SEQNUM? indicator shown
  - [x] 19.10 Write end-to-end test: Grid_Edit_Mode rejection — attempt UNNUM and NUMBER in grid mode, verify error messages returned

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered By Task(s) |
|-------------|----------|---------------------|
| Req 1: Language Profile Configuration | 1.1–1.10 | 2.1–2.4, 13.1–13.6, 11.1–11.6 |
| Req 2: Sequence Number Detection | 2.1–2.9 | 3.1–3.9, 18.1, 18.7, 18.8, 18.10 |
| Req 3: Auto-Strip on File Open | 3.1–3.9 | 5.1–5.5, 6.1–6.6, 4.1–4.6, 18.2, 18.3 |
| Req 4: Visual Indication | 4.1–4.5 | 15.1–15.5 |
| Req 5: UNNUM Command | 5.1–5.11 | 9.1–9.7, 18.2, 18.9, 18.12 |
| Req 6: NUMBER Command | 6.1–6.12 | 8.1–8.7, 10.1–10.9, 18.4, 18.5 |
| Req 7: Sequence Format Options | 7.1–7.5 | 8.2–8.4, 10.2, 18.6 |
| Req 8: NUMBER SHOW Display Mode | 8.1–8.7 | 7.1–7.7 |
| Req 9: Undo/Redo Interaction | 9.1–9.6 | 17.1–17.6, 18.3 |
| Req 10: BOUNDS Interaction | 10.1–10.4 | 16.1–16.4 |
| Req 11: SAVE Interaction | 11.1–11.6 | 12.1–12.6 |
| Req 12: Configuration Per Language | 12.1–12.5 | 1.5, 11.1–11.6, 18.10 |
| Req 13: Standard Text Mode Only | 13.1–13.3 | 14.6, 10.8 |
| Req 14: Command Compatibility Matrix | 14.1–14.8 | 14.1–14.7 |

---

## Notes

- This crate has zero GUI dependencies — all functionality is testable via unit and property-based tests against the public API
- The crate depends on `ff-logging`, `thiserror`, `serde`, `serde_derive`, and the standard library. All upstream crates (`ff-document-model`, `ff-language-service`, `ff-command`, `ff-undo`, `ff-config`) are connected via trait interfaces defined in `src/traits.rs`
- Property tests use `proptest` crate with a minimum of 100 iterations per property
- The strip engine (Task 5) is the foundation for both auto-strip (Task 6) and UNNUM (Task 9)
- NUMBER SHOW (Task 7) depends on the side-table populated by the strip engine (Tasks 4, 5)
- Save-time restoration (Task 12) depends on the side-table and the number generation engine
- The `DocumentAccess` trait enables document model integration without a compile-time dependency on `ff-document-model`
- BOUNDS interaction (Task 16) defines constraints only — actual BOUNDS state is owned by `ff-navigation-commands`
- Command registration (Task 14) depends on the `CommandRegistry` trait from `ff-command`; the actual registry is injected at runtime

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Crate scaffolding and core types", "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6"] },
    { "id": 1, "label": "Column range and config", "tasks": ["2.1", "2.2", "2.3", "2.4", "13.1", "13.2", "13.3", "13.4", "13.5", "13.6"], "dependsOn": [0] },
    { "id": 2, "label": "Detection engine", "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "3.9"], "dependsOn": [1] },
    { "id": 3, "label": "State model and strip engine", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "5.1", "5.2", "5.3", "5.4", "5.5"], "dependsOn": [1] },
    { "id": 4, "label": "Auto-strip and NUMBER SHOW", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7"], "dependsOn": [2, 3] },
    { "id": 5, "label": "Number generation engine", "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7"], "dependsOn": [1] },
    { "id": 6, "label": "UNNUM and NUMBER commands", "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "10.9"], "dependsOn": [3, 4, 5] },
    { "id": 7, "label": "Configuration, save, BOUNDS, and undo integration", "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "16.1", "16.2", "16.3", "16.4", "17.1", "17.2", "17.3", "17.4", "17.5", "17.6"], "dependsOn": [4, 6] },
    { "id": 8, "label": "Command registration and indicators", "tasks": ["14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "14.7", "15.1", "15.2", "15.3", "15.4", "15.5"], "dependsOn": [6, 7] },
    { "id": 9, "label": "Property-based tests", "tasks": ["18.1", "18.2", "18.3", "18.4", "18.5", "18.6", "18.7", "18.8", "18.9", "18.10", "18.11", "18.12"], "dependsOn": [8] },
    { "id": 10, "label": "Integration tests", "tasks": ["19.1", "19.2", "19.3", "19.4", "19.5", "19.6", "19.7", "19.8", "19.9", "19.10"], "dependsOn": [9] }
  ]
}
```
