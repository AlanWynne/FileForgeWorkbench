# Implementation Plan: Navigation Commands (`ff-navigation-commands`)

## Overview

This plan covers the complete implementation of the `ff-navigation-commands` crate — the navigation command subsystem for FileForgeWorkbench. The crate provides ISPF-style LOCATE, SORT, COLS, and BOUNDS commands, viewport navigation (UP/DOWN/LEFT/RIGHT/TOP/BOTTOM), paragraph navigation, word and word-part navigation, vertical caret movement with column affinity, and document start/end navigation. It also registers delegation-only commands (SAVE, CANCEL, END, LOAD, RELOAD, DELETE, COPY, MOVE, MACRO/EXEC/RUN, UNDO, REDO) that are dispatched to their owning crates.

This is a **Wave 5 (Command Engine)** sub-project that depends on `ff-viewport-scrolling` (Wave 4) for viewport state delegation, `ff-document-model` (Wave 4) for line content and character classification, `ff-command` (Wave 2) for command registration, and `ff-undo-redo` (Wave 4) for SORT transaction wrapping.

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-navigation-commands/Cargo.toml` with dependencies (thiserror, proptest dev-dep) and deps on `ff-document-model`, `ff-viewport-scrolling`, `ff-command`, `ff-undo-redo`, `ff-configuration-system`, `ff-logging`
  - [ ] 1.2 Create `crates/ff-navigation-commands/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `locate.rs`, `sort.rs`, `cols.rs`, `bounds.rs`, `viewport_nav.rs`, `paragraph_nav.rs`, `word_nav.rs`, `word_part_nav.rs`, `vertical_caret.rs`, `doc_nav.rs`, `delegation.rs`, `commands.rs`, `config.rs`, `session_state.rs`, `char_class.rs`, `error.rs`, `types.rs`
  - [ ] 1.4 Add `ff-navigation-commands` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. Core types and session state
  - [ ] 2.1 Define `Bounds { left: u64, right: u64 }` struct with validation (left >= 1, right > left) and intersection logic
  - [ ] 2.2 Define `ColsLine` struct representing a synthetic column ruler display artifact with anchor position
  - [ ] 2.3 Define `BndsLine` struct representing a synthetic bounds display artifact with left/right markers
  - [ ] 2.4 Define `SessionState` struct holding active bounds, COLS_Line list, BNDS_Line reference, and display artifact state
  - [ ] 2.5 Define `SortOrder` enum (Ascending, Descending) and `SortScope` enum (All, Visible, Tagged, Block)
  - [ ] 2.6 Define `SortRequest { col_start: Option<u64>, col_end: Option<u64>, order: SortOrder, scope: SortScope }` struct
  - [ ] 2.7 Define `NavigationDirection` enum and `ScrollAmount` types for viewport nav commands
  - [ ] 2.8 Write unit tests for Bounds validation, intersection, and SortRequest construction
  - Covers: Requirement 2 (AC 2.1–2.4), Requirement 4 (AC 4.1–4.3), Requirement 5 (AC 5.1–5.6)

- [ ] 3. Character classification system
  - [ ] 3.1 Define `CharacterClass` enum (Space, NewLine, Word, Punctuation) per SCI-DOC-16
  - [ ] 3.2 Implement `CharClassify` struct with configurable ASCII classification table (256-entry lookup)
  - [ ] 3.3 Implement `classify(ch: char) -> CharacterClass` method with Unicode fallback for code points >= 0x80
  - [ ] 3.4 Implement `SetCharClasses` API for application-level customisation of word characters
  - [ ] 3.5 Implement `SetDefaultCharClasses` API to reset to built-in defaults
  - [ ] 3.6 Implement integration with `editor.navigation.word_characters` config key to extend default word chars
  - [ ] 3.7 Write unit tests for ASCII classification, Unicode categories, custom char class overrides
  - Covers: Requirement 7 (AC 7.1, 7.9), Requirement 8 (AC 8.5), Requirement 18 (AC 18.4)

- [ ] 4. LOCATE command implementation
  - [ ] 4.1 Implement `LocateCommand` struct with `execute(target: &str, viewport: &mut ViewportModel, cursor: &mut CursorModel, doc: &dyn DocumentModel)` method
  - [ ] 4.2 Implement numeric argument parsing: interpret positive integer as target line number
  - [ ] 4.3 Implement line-number validation: reject < 1 or > line_count with "Line number out of range" error
  - [ ] 4.4 Implement label argument parsing: interpret non-numeric text as named label lookup
  - [ ] 4.5 Implement label-not-found error: "Label not found: <label>" with viewport unchanged
  - [ ] 4.6 Implement successful navigation: set top_line to target, update cursor_line to target, reset cursor_column to 1
  - [ ] 4.7 Register LOCATE with command framework as non-undoable, valid in Browse and Edit modes
  - [ ] 4.8 Write unit tests for numeric locate, out-of-range, label found, label not found, cursor reset
  - Covers: Requirement 1 (AC 1.1–1.6)

- [ ] 5. SORT command implementation
  - [ ] 5.1 Implement `SortCommand` struct with SORT argument parser: `SORT [col1 col2] [A|D] [TAGGED|VISIBLE]`
  - [ ] 5.2 Implement scope resolution: no qualifier → all visible lines, TAGGED → tagged-only, VISIBLE → non-excluded, CC block → block range
  - [ ] 5.3 Implement column-key extraction: slice characters col1..col2 from each line for comparison
  - [ ] 5.4 Implement ascending (A) and descending (D) stable sort using the extracted column key
  - [ ] 5.5 Implement Bounds integration: when no explicit columns given, use active Bounds as default key range
  - [ ] 5.6 Implement Bounds intersection: when explicit columns AND Bounds are set, use intersection as effective range
  - [ ] 5.7 Implement zero/one-line scope guard: display "Nothing to sort" without recording a transaction
  - [ ] 5.8 Implement undo transaction recording: wrap the line reordering as a single undoable Transaction
  - [ ] 5.9 Implement TAGGED scope: sort only tagged lines in-place, non-tagged lines retain positions
  - [ ] 5.10 Register SORT with command framework as undoable, valid in Edit mode only
  - [ ] 5.11 Write unit tests for all scope variants, column extraction, bounds interaction, stable sort, undo recording
  - Covers: Requirement 2 (AC 2.1–2.13)

- [ ] 6. Viewport navigation commands (UP, DOWN, LEFT, RIGHT, TOP, BOTTOM)
  - [ ] 6.1 Implement `UpCommand` with optional integer argument: no arg = page scroll (visible_count lines), with arg = n lines
  - [ ] 6.2 Implement `DownCommand` with optional integer argument: no arg = page scroll, with arg = n lines
  - [ ] 6.3 Implement page overlap: subtract `editor.navigation.page_overlap_lines` from page scroll amount
  - [ ] 6.4 Implement `LeftCommand` with optional integer argument: no arg = configured default columns, with arg = n columns
  - [ ] 6.5 Implement `RightCommand` with optional integer argument: no arg = configured default columns, with arg = n columns
  - [ ] 6.6 Implement `TopCommand`: scroll to line 1, update cursor_line to 1, reset cursor_column to 1
  - [ ] 6.7 Implement `BottomCommand`: scroll to max_top_line, update cursor_line to last line, reset cursor_column to 1
  - [ ] 6.8 Implement vertical clamping: top_line never < 1 and never > max_top_line, no error on overshoot
  - [ ] 6.9 Implement horizontal clamping: horizontal_offset never < 0, no error on undershoot
  - [ ] 6.10 Register all navigation commands with command framework as non-undoable, valid in both Browse and Edit modes
  - [ ] 6.11 Implement configurable default horizontal scroll via `editor.navigation.horizontal_scroll_columns` (default 8)
  - [ ] 6.12 Write unit tests for page scroll, line scroll, column scroll, clamping at boundaries, TOP/BOTTOM cursor updates
  - Covers: Requirement 3 (AC 3.1–3.16), Requirement 18 (AC 18.1–18.2, 18.5)

- [ ] 7. COLS command implementation
  - [ ] 7.1 Implement `ColsCommand` struct managing COLS_Line insertion/removal in SessionState
  - [ ] 7.2 Implement COLS_Line formatting: `----+----1----+----2----+----3...` ruler pattern with prefix indicator
  - [ ] 7.3 Implement toggle behaviour: issuing COLS at same position removes existing COLS_Line
  - [ ] 7.4 Implement multiple COLS_Lines: allow separate COLS at different cursor positions
  - [ ] 7.5 Implement COLS line command: insert COLS_Line above the specified document line from prefix area
  - [ ] 7.6 Implement COLS_Line as display-only artifact: excluded from document operations, not saved to disk
  - [ ] 7.7 Implement COLS_Line scrolling: anchor to document lines so it scrolls with content
  - [ ] 7.8 Implement RESET/RESET ALL/RESET COMMANDS integration: clear all COLS_Lines
  - [ ] 7.9 Implement non-editable prefix cell display for COLS_Line showing "COLS" indicator
  - [ ] 7.10 Register COLS with command framework as non-undoable, valid in both Browse and Edit modes
  - [ ] 7.11 Write unit tests for insertion, toggle, multiple lines, RESET clearing, prefix display
  - Covers: Requirement 4 (AC 4.1–4.11)

- [ ] 8. BOUNDS / BNDS command implementation
  - [ ] 8.1 Implement `BoundsCommand` with argument parser: `BOUNDS [left right]` or `BNDS [left right]`
  - [ ] 8.2 Implement bounds validation: left >= 1, right > left, both positive integers; error message for invalid input
  - [ ] 8.3 Implement bounds storage in SessionState with set/clear operations
  - [ ] 8.4 Implement BNDS_Line display artifact: show `<` at left column and `>` at right column
  - [ ] 8.5 Implement no-argument clearing: issuing BOUNDS/BNDS with no args clears bounds and removes BNDS_Line
  - [ ] 8.6 Implement BNDS_Line as display-only artifact: not a real document line, not saved to disk
  - [ ] 8.7 Implement public query API: `get_active_bounds() -> Option<Bounds>` for other command executors
  - [ ] 8.8 Implement bounds-affect-find integration via `editor.bounds.affect_find` config key
  - [ ] 8.9 Implement non-undoable session state: bounds changes never recorded as transactions
  - [ ] 8.10 Register BOUNDS and BNDS as non-undoable commands with alias support, valid in both modes
  - [ ] 8.11 Write unit tests for set bounds, clear bounds, validation errors, BNDS_Line display, query API
  - Covers: Requirement 5 (AC 5.1–5.15), Requirement 18 (AC 18.3)

- [ ] 9. Paragraph navigation implementation
  - [ ] 9.1 Implement `ParagraphUpCommand`: move caret to beginning of previous paragraph boundary
  - [ ] 9.2 Implement `ParagraphDownCommand`: move caret to beginning of next paragraph boundary
  - [ ] 9.3 Implement paragraph boundary detection: blank/whitespace-only lines define boundaries
  - [ ] 9.4 Implement boundary traversal: skip contiguous blank lines to reach the first content line after the gap
  - [ ] 9.5 Implement document-start clamping: if no prior boundary, move to position 0 (first char of first line)
  - [ ] 9.6 Implement document-end clamping: if no further boundary, move to last line of document
  - [ ] 9.7 Implement excluded-line skipping: treat contiguous excluded lines as non-present for boundary detection
  - [ ] 9.8 Implement viewport scroll-to-keep-caret-visible after paragraph navigation
  - [ ] 9.9 Implement selection extension: Extend modifier extends selection from anchor to new caret position
  - [ ] 9.10 Register PARA_UP and PARA_DOWN with command framework as non-undoable, valid in both modes
  - [ ] 9.11 Write unit tests for boundary detection, excluded line skipping, clamping, selection extension
  - Covers: Requirement 6 (AC 6.1–6.9)

- [ ] 10. Word navigation implementation
  - [ ] 10.1 Implement `WordLeftCommand`: skip whitespace backwards, then skip same-class chars backwards to class transition
  - [ ] 10.2 Implement `WordRightCommand`: skip current-class chars forwards, then skip whitespace to next non-space
  - [ ] 10.3 Implement `WordEndRightCommand`: skip whitespace forwards, then skip word chars to next class transition
  - [ ] 10.4 Implement line-boundary crossing: continue navigation on adjacent line when reaching line start/end
  - [ ] 10.5 Implement document boundary clamping: clamp at position 0 (start) and document end without error
  - [ ] 10.6 Implement character classification integration: use CharClassify for word/punctuation/space transitions
  - [ ] 10.7 Implement viewport scroll-to-keep-caret-visible after word navigation
  - [ ] 10.8 Implement selection extension: Extend modifier extends selection from anchor to new caret position
  - [ ] 10.9 Register WORD_LEFT, WORD_RIGHT with command framework as non-undoable, valid in both modes
  - [ ] 10.10 Write unit tests for word boundaries, line crossing, document clamping, Unicode chars, selection extend
  - Covers: Requirement 7 (AC 7.1–7.11)

- [ ] 11. Word-part (camelCase/sub-word) navigation implementation
  - [ ] 11.1 Implement `WordPartLeftCommand`: detect sub-word boundaries moving backwards within current word
  - [ ] 11.2 Implement `WordPartRightCommand`: detect sub-word boundaries moving forwards within current word
  - [ ] 11.3 Implement camelCase boundary detection: lowercase→uppercase transition (e.g., `get|Value`)
  - [ ] 11.4 Implement UPPER_UPPER_lower boundary: last uppercase before lowercase run (e.g., `XML|Parser`)
  - [ ] 11.5 Implement alpha↔non-alpha boundary: transitions between alphanumeric and separator characters
  - [ ] 11.6 Implement digit↔alpha boundary: transitions between digits and letters
  - [ ] 11.7 Implement word-boundary crossing: when at word start/end, cross to adjacent word's last/first part
  - [ ] 11.8 Implement selection extension: Extend modifier extends selection from anchor to new caret position
  - [ ] 11.9 Implement viewport scroll-to-keep-caret-visible after word-part navigation
  - [ ] 11.10 Register WORD_PART_LEFT, WORD_PART_RIGHT with command framework as non-undoable, valid in both modes
  - [ ] 11.11 Write unit tests for camelCase, snake_case, UPPER runs, digit boundaries, word crossing, selection
  - Covers: Requirement 8 (AC 8.1–8.8)

- [ ] 12. Vertical caret movement and column affinity
  - [ ] 12.1 Implement column affinity storage: maintain `column_affinity` value across vertical movements
  - [ ] 12.2 Implement vertical movement target computation: use column_affinity rather than current cursor_column
  - [ ] 12.3 Implement short-line clamping: when target line is shorter than affinity, place caret at line end without modifying affinity
  - [ ] 12.4 Implement affinity reset on horizontal movement: update column_affinity on char left/right, word nav, home, end
  - [ ] 12.5 Implement line-up caret movement with affinity and clamping at line 1
  - [ ] 12.6 Implement line-down caret movement with affinity and clamping at last document line
  - [ ] 12.7 Implement page-up caret movement: move up by visible_count lines maintaining affinity
  - [ ] 12.8 Implement page-down caret movement: move down by visible_count lines maintaining affinity
  - [ ] 12.9 Implement viewport scroll delegation when caret moves off-screen
  - [ ] 12.10 Implement selection extension for all vertical movements
  - [ ] 12.11 Write unit tests for affinity preservation, short line clamping, page movements, boundary clamping
  - Covers: Requirement 9 (AC 9.1–9.10)

- [ ] 13. Document start and end navigation
  - [ ] 13.1 Implement `DocStartCommand`: move caret to position 0 (line 1, column 1), scroll viewport to top
  - [ ] 13.2 Implement `DocEndCommand`: move caret to end of last line, scroll viewport to show last page
  - [ ] 13.3 Implement column affinity update: DOC_START resets affinity to 1, DOC_END updates to last-line position
  - [ ] 13.4 Implement selection extension: Extend modifier extends selection from anchor to new position
  - [ ] 13.5 Register DOC_START, DOC_END with command framework as non-undoable, valid in both modes
  - [ ] 13.6 Write unit tests for position updates, affinity changes, viewport scroll, selection extension
  - Covers: Requirement 10 (AC 10.1–10.6)

- [ ] 14. Delegation command registrations
  - [ ] 14.1 Register SAVE, CANCEL, END commands as delegation-only with appropriate metadata (owned by `file-operations`)
  - [ ] 14.2 Register LOAD, RELOAD commands as delegation-only (owned by `file-operations`)
  - [ ] 14.3 Register DELETE command as delegation-only (owned by `edit-operations`)
  - [ ] 14.4 Register COPY command as delegation-only (owned by `edit-operations`)
  - [ ] 14.5 Register MOVE command as delegation-only (owned by `edit-operations`)
  - [ ] 14.6 Register MACRO, EXEC, RUN commands as delegation-only with alias support (owned by `lua-macro-engine`)
  - [ ] 14.7 Register UNDO, REDO commands as delegation-only with special no-history flag (owned by `undo-redo-transactions`)
  - [ ] 14.8 Implement dispatch routing to owning crate modules via command framework delegation trait
  - [ ] 14.9 Write unit tests verifying delegation registrations and dispatch routing
  - Covers: Requirements 11–17 (AC 11.1–11.4, 12.1–12.5, 13.1–13.5, 14.1–14.4, 15.1–15.4, 16.1–16.6, 17.1–17.4)

- [ ] 15. Configuration integration
  - [ ] 15.1 Implement config loading for `editor.navigation.horizontal_scroll_columns` (default 8)
  - [ ] 15.2 Implement config loading for `editor.navigation.page_overlap_lines` (default 2)
  - [ ] 15.3 Implement config loading for `editor.bounds.affect_find` (default false)
  - [ ] 15.4 Implement config loading for `editor.navigation.word_characters` (default empty)
  - [ ] 15.5 Implement fallback-to-default with warning emission when config values are missing or invalid
  - [ ] 15.6 Write unit tests for config parsing, invalid value fallback, and warning emission
  - Covers: Requirement 18 (AC 18.1–18.5)

- [ ] 16. Command registration and metadata
  - [ ] 16.1 Implement command metadata for all owned commands: display name, help_text, aliases, mode validity
  - [ ] 16.2 Implement non-undoable classification for: LOCATE, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM, COLS, BOUNDS, BNDS, PARA_UP, PARA_DOWN, WORD_LEFT, WORD_RIGHT, WORD_PART_LEFT, WORD_PART_RIGHT, DOC_START, DOC_END
  - [ ] 16.3 Implement undoable classification for SORT
  - [ ] 16.4 Implement delegation-only classification for: SAVE, CANCEL, END, LOAD, RELOAD, DELETE, COPY, MOVE, MACRO, EXEC, RUN, UNDO, REDO
  - [ ] 16.5 Implement help_text field for each command providing syntax and description
  - [ ] 16.6 Implement alias registration: BOUNDS/BNDS, MACRO/EXEC/RUN
  - [ ] 16.7 Implement mode validity: navigation commands valid in Browse+Edit, SORT valid in Edit only
  - [ ] 16.8 Write unit tests verifying all registrations, metadata, aliases, and mode flags
  - Covers: Requirement 19 (AC 19.1–19.6)

- [ ] 17. Error handling
  - [ ] 17.1 Define `NavigationError` enum: LineOutOfRange, LabelNotFound, NothingToSort, InvalidBounds, InvalidArgument, DelegationFailed
  - [ ] 17.2 Implement error message formatting per `[navigation] command: description` standard (≤200 chars)
  - [ ] 17.3 Implement LOCATE error messages: "Line number out of range", "Label not found: <label>"
  - [ ] 17.4 Implement SORT error message: "Nothing to sort"
  - [ ] 17.5 Implement BOUNDS error message: "Invalid bounds: left must be >= 1 and right must be > left"
  - [ ] 17.6 Implement config warning format: identify key name and applied default
  - [ ] 17.7 Write unit tests for all error variants and message formatting
  - Covers: Cross-cutting Requirement 8 (Error Message Standards)

- [ ] 18. Property-based tests
  - [ ] 18.1 Write PBT: viewport navigation clamping correctness
  - [ ] 18.2 Write PBT: SORT stability and key extraction correctness
  - [ ] 18.3 Write PBT: word navigation class-transition correctness
  - [ ] 18.4 Write PBT: word-part boundary detection correctness
  - [ ] 18.5 Write PBT: column affinity preservation across vertical movements
  - [ ] 18.6 Write PBT: paragraph boundary detection correctness
  - [ ] 18.7 Write PBT: bounds validation and intersection correctness
  - Covers: Requirements 1–10, 18 (see Property-Based Test Definitions below)

- [ ] 19. Integration tests
  - [ ] 19.1 Write integration test: LOCATE → viewport scroll → cursor update lifecycle
  - [ ] 19.2 Write integration test: SORT with bounds interaction and undo/redo round-trip
  - [ ] 19.3 Write integration test: word and word-part navigation across multi-line document
  - [ ] 19.4 Write integration test: paragraph navigation with excluded lines
  - [ ] 19.5 Write integration test: COLS/BOUNDS display artifact lifecycle (insert, toggle, RESET)
  - [ ] 19.6 Write integration test: full navigation sequence (TOP → DOWN → LOCATE → BOTTOM) with clamping
  - [ ] 19.7 Write integration test: vertical caret movement with affinity across lines of varying length
  - Covers: End-to-end validation across Requirements 1–19

---

## Property-Based Test Definitions

### Property 1: Viewport Navigation Clamping Correctness

**Validates: Requirements 3.11, 3.12, 3.13**

- **Statement:** For any viewport state (top_line, visible_count, total_lines) and any navigation command (UP n, DOWN n, LEFT n, RIGHT n, TOP, BOTTOM), the resulting viewport state SHALL always satisfy: `1 <= top_line <= max_top_line` and `horizontal_offset >= 0`. No navigation command shall produce an out-of-bounds viewport state.
- **Strategy:** Generate:
  - total_lines: [1, 100_000]
  - visible_count: [1, total_lines]
  - initial top_line: [1, max_top_line]
  - initial horizontal_offset: [0, 10_000]
  - command: random choice of UP/DOWN/LEFT/RIGHT/TOP/BOTTOM with random n in [0, total_lines * 2]
- **Invariant:** `1 <= result.top_line <= max(1, total_lines - visible_count + 1)` AND `result.horizontal_offset >= 0`

### Property 2: SORT Stability and Key Extraction Correctness

**Validates: Requirements 2.4, 2.8, 2.9, 2.10**

- **Statement:** For any set of lines and any column range, sorting SHALL be stable (equal-key lines retain original order), the sort key for each line SHALL be exactly the characters in [col_start, col_end] (or the bounds-intersected range), and the result set SHALL contain exactly the same lines as the input (no loss or duplication).
- **Strategy:** Generate:
  - Line count: [2, 500]
  - Line content: arbitrary strings (0–200 chars) with some lines sharing identical key columns
  - Column range: random [1, max_line_length] with col_start <= col_end
  - Sort order: random (Ascending, Descending)
  - Optional bounds: random valid Bounds or None
- **Invariant:** `sorted_lines.len() == input_lines.len()` AND stable ordering AND key extraction matches `line[col_start-1..col_end]`

### Property 3: Word Navigation Class-Transition Correctness

**Validates: Requirements 7.1, 7.2, 7.3, 7.5**

- **Statement:** For any document content and any starting caret position, word-left navigation SHALL stop at a position where the character to the left of the caret has a different CharacterClass than the character at the caret (or at position 0), and word-right navigation SHALL stop at a position where the character at the caret is non-space and the character before it is space or a different class (or at document end).
- **Strategy:** Generate:
  - Document content: arbitrary text (1–500 chars) with mixed word/punctuation/space
  - Starting position: random valid position [0, content.len()]
- **Invariant:** At the result position, a class transition exists (or document boundary reached)

### Property 4: Word-Part Boundary Detection Correctness

**Validates: Requirements 8.1, 8.2, 8.5**

- **Statement:** For any identifier-like string (alphanumeric + separators), word-part-right followed by word-part-left from the result position SHALL return to the original position (or a valid sub-word boundary between original and result). Every detected boundary SHALL correspond to one of the defined transition patterns: lowerUpper, UPPER_UPPER_lower, alpha_nonalpha, digit_alpha, or alpha_digit.
- **Strategy:** Generate:
  - Identifiers: random strings from alphabet [a-z, A-Z, 0-9, _, -] of length [2, 50]
  - Starting position: random valid position within the identifier
- **Invariant:** Each boundary position satisfies at least one of the five transition patterns; total parts cover the entire identifier without gaps

### Property 5: Column Affinity Preservation Across Vertical Movements

**Validates: Requirements 9.1, 9.2, 9.3, 9.4**

- **Statement:** For any sequence of vertical movements (line-up, line-down, page-up, page-down) without intervening horizontal movements, the column_affinity value SHALL remain unchanged from when it was last set by a horizontal movement. When the caret lands on a line at least as long as the affinity, cursor_column SHALL equal column_affinity. When the line is shorter, cursor_column SHALL be clamped to line length but affinity is preserved.
- **Strategy:** Generate:
  - Document: random lines of varying length [1, 200 chars], count [5, 100]
  - Initial horizontal position: random column on starting line (sets affinity)
  - Movement sequence: random sequence of [line_up, line_down, page_up, page_down] of length [1, 20]
- **Invariant:** `column_affinity == initial_affinity` throughout sequence; `cursor_column == min(affinity, target_line_length)` after each step

### Property 6: Paragraph Boundary Detection Correctness

**Validates: Requirements 6.1, 6.2, 6.3**

- **Statement:** For any document content, paragraph-down from any position SHALL land the caret on the first non-blank line after the next blank-line group (or at document end), and paragraph-up SHALL land on the first non-blank line after the previous blank-line group (or at document start). A blank line is defined as empty or whitespace-only.
- **Strategy:** Generate:
  - Document: random lines mixing content lines and blank/whitespace-only lines, count [3, 200]
  - Starting caret line: random [1, line_count]
- **Invariant:** Result line is either a document boundary or is preceded (for down) / followed (for up) by at least one blank line in the appropriate direction

### Property 7: Bounds Validation and Intersection Correctness

**Validates: Requirements 5.1, 5.13, 2.9, 2.10**

- **Statement:** For any left/right pair, bounds validation SHALL accept iff left >= 1 AND right > left. For any valid Bounds and any explicit column range [col1, col2], the intersection SHALL produce a range where `effective_start = max(left, col1)` and `effective_end = min(right, col2)`, and if effective_start > effective_end the result is empty (no columns to sort/search).
- **Strategy:** Generate:
  - left: random i64 in [-5, 500], right: random i64 in [-5, 500]
  - col1: random [1, 200], col2: random [1, 200] with col1 <= col2
  - bounds_left: random [1, 200], bounds_right: random [bounds_left+1, 300]
- **Invariant:** Validation rejects iff left < 1 OR right <= left; intersection produces `max(bounds_left, col1)..min(bounds_right, col2)` with empty result when max > min

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Config", "tasks": ["2", "3", "15", "17"], "dependsOn": [0] },
    { "id": 2, "label": "LOCATE Command", "tasks": ["4"], "dependsOn": [1] },
    { "id": 3, "label": "SORT Command", "tasks": ["5"], "dependsOn": [1] },
    { "id": 4, "label": "Viewport Navigation", "tasks": ["6"], "dependsOn": [1] },
    { "id": 5, "label": "Display Artifacts", "tasks": ["7", "8"], "dependsOn": [1] },
    { "id": 6, "label": "Caret Navigation", "tasks": ["9", "10", "11", "12", "13"], "dependsOn": [1] },
    { "id": 7, "label": "Command Registration", "tasks": ["14", "16"], "dependsOn": [2, 3, 4, 5, 6] },
    { "id": 8, "label": "Validation and PBT", "tasks": ["18", "19"], "dependsOn": [7] }
  ]
}
```

---

## Notes

- This is a Wave 5 (Command Engine) crate depending on `ff-viewport-scrolling` (Wave 4) for viewport state delegation
- The `ff-document-model` (Wave 4) provides line count, line content, and character classification tables
- SORT is the only undoable command in this crate; all other commands modify viewport/session state only
- Delegation commands (Requirements 11–17) register metadata but dispatch execution to owning crates
- COLS_Line and BNDS_Line are pure display artifacts — they are never part of the document model and are not persisted
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The design.md for this crate was generated concurrently; if unavailable, the task structure is derived solely from requirements.md
- Column affinity integrates with the CursorModel from `ff-viewport-scrolling` — this crate extends the cursor movement logic
- Word and word-part navigation reuse the CharClassify system from `ff-document-model`; this crate adds the navigation algorithms on top

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: LOCATE Command | AC 1.1–1.6 | Task 4 |
| Req 2: SORT Command | AC 2.1–2.13 | Task 5 |
| Req 3: Navigation Commands (UP/DOWN/LEFT/RIGHT/TOP/BOTTOM) | AC 3.1–3.16 | Task 6 |
| Req 4: COLS Command | AC 4.1–4.11 | Task 7 |
| Req 5: BOUNDS/BNDS Command | AC 5.1–5.15 | Task 8 |
| Req 6: Paragraph Navigation | AC 6.1–6.9 | Task 9 |
| Req 7: Word Navigation | AC 7.1–7.11 | Tasks 3, 10 |
| Req 8: Word-Part Navigation | AC 8.1–8.8 | Task 11 |
| Req 9: Vertical Caret Movement and Column Affinity | AC 9.1–9.10 | Task 12 |
| Req 10: Document Start and End Navigation | AC 10.1–10.6 | Task 13 |
| Req 11: SAVE, CANCEL, END (Delegation) | AC 11.1–11.4 | Task 14 |
| Req 12: LOAD, RELOAD (Delegation) | AC 12.1–12.5 | Task 14 |
| Req 13: DELETE (Delegation) | AC 13.1–13.5 | Task 14 |
| Req 14: COPY (Delegation) | AC 14.1–14.4 | Task 14 |
| Req 15: MOVE (Delegation) | AC 15.1–15.4 | Task 14 |
| Req 16: MACRO/EXEC/RUN (Delegation) | AC 16.1–16.6 | Task 14 |
| Req 17: UNDO/REDO (Delegation) | AC 17.1–17.4 | Task 14 |
| Req 18: Configuration Options | AC 18.1–18.5 | Tasks 3, 6, 8, 15 |
| Req 19: Command Registration and Metadata | AC 19.1–19.6 | Task 16 |
