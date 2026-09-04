# Implementation Plan: Edit Operations (`ff-edit-operations`)

## Overview

This plan covers the complete implementation of the `ff-edit-operations` crate — the text editing behaviour layer for FileForgeWorkbench. It implements insert/overstrike modes, character insertion/deletion at multiple granularities, the selection model (stream, rectangular, multi-caret), edit boundaries (BOUNDS), line manipulation commands, clipboard integration (edit-side semantics), transaction recording, and command framework integration.

This is a **Wave 4 (Core Editor)** sub-project. It depends on `ff-document-model` for buffer access, `ff-command` for command registration, and `ff-undo-redo` for transaction recording.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-edit-operations/Cargo.toml` with dependencies (ff-document-model, ff-command, ff-logging, thiserror, unicode-segmentation, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-edit-operations/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `mode.rs`, `insert.rs`, `delete.rs`, `newline.rs`, `selection_position.rs`, `selection_range.rs`, `selection_container.rs`, `multi_caret.rs`, `rectangular.rs`, `bounds.rs`, `line_commands.rs`, `clipboard.rs`, `transaction.rs`, `commands.rs`, `error.rs`
  - [x] 1.4 Add `ff-edit-operations` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. SelectionPosition type
  - [x] 2.1 Define `SelectionPosition` struct with `line: u64`, `column: u64`, `virtual_space: u64` fields
  - [x] 2.2 Implement `new()`, `with_virtual_space()`, and `at_line_end()` constructors
  - [x] 2.3 Implement `Ord` / `PartialOrd` with document-order comparison (line first, then column + virtual_space)
  - [x] 2.4 Implement `shift_forward(amount)` and `shift_backward(amount)` for position adjustment
  - [x] 2.5 Implement `realise_virtual_space() -> u64` that returns the number of space characters needed to materialise virtual space
  - [x] 2.6 Write unit tests for ordering, construction, and virtual space operations
  - Covers: Requirement 6 (AC 6.2), Requirement 7 (AC 7.1–7.4)

- [x] 3. SelectionRange type
  - [x] 3.1 Define `SelectionRange` struct with `anchor: SelectionPosition` and `caret: SelectionPosition`
  - [x] 3.2 Implement `is_collapsed()` returning true when anchor == caret (no selection)
  - [x] 3.3 Implement `ordered_start()` / `ordered_end()` returning the lesser/greater position regardless of anchor/caret order
  - [x] 3.4 Implement `contains(position: &SelectionPosition) -> bool`
  - [x] 3.5 Implement `overlaps(other: &SelectionRange) -> bool` for overlap detection
  - [x] 3.6 Implement `merge(other: &SelectionRange) -> SelectionRange` producing the union of two overlapping ranges
  - [x] 3.7 Write unit tests for collapsed detection, ordering, containment, overlap, and merge
  - Covers: Requirement 6 (AC 6.1), Requirement 14 (AC 14.3)

- [x] 4. Selection container — core operations
  - [x] 4.1 Define `SelectionContainer` struct holding `Vec<SelectionRange>` and `main_index: usize`
  - [x] 4.2 Implement `new()` initialising with a single collapsed range at document start
  - [x] 4.3 Implement `add(range: SelectionRange)` that inserts in sorted order by document position
  - [x] 4.4 Implement `drop(index: usize) -> Result<(), EditError>` that removes a range (fails if only one remains)
  - [x] 4.5 Implement `trim()` that merges overlapping/identical ranges into their union
  - [x] 4.6 Implement `main_range() -> &SelectionRange` and `set_main_range(index: usize)`
  - [x] 4.7 Implement `ranges() -> impl Iterator<Item = &SelectionRange>` yielding all ranges in document order
  - [x] 4.8 Implement `count() -> usize` returning the number of active selections
  - [x] 4.9 Write unit tests for add, drop, trim, main range, and count
  - Covers: Requirement 14 (AC 14.1–14.9)

- [x] 5. Selection container — position adjustment (MovePositions)
  - [x] 5.1 Define `DocumentModification` struct with `offset: u64`, `inserted_length: u64`, `deleted_length: u64`
  - [x] 5.2 Implement `move_positions(modification: &DocumentModification)` on SelectionContainer adjusting all positions in all ranges
  - [x] 5.3 Implement position-shift logic: positions before offset unchanged, positions within deleted range collapse to offset, positions after shift by (inserted - deleted)
  - [x] 5.4 After adjustment, invoke `trim()` to merge any newly-overlapping ranges
  - [x] 5.5 Write unit tests: insertion before/at/after positions, deletion spanning positions, collapse-to-zero scenarios
  - Covers: Requirement 7 (AC 7.1–7.7), Requirement 14 (AC 14.4)

- [x] 6. Edit mode management
  - [x] 6.1 Define `EditMode` enum with `Insert` and `Overstrike` variants
  - [x] 6.2 Define `EditModeManager` struct holding the current mode (default: Insert)
  - [x] 6.3 Implement `toggle()` method switching between Insert and Overstrike
  - [x] 6.4 Implement `mode() -> EditMode` accessor
  - [x] 6.5 Implement `is_insert() -> bool` and `is_overstrike() -> bool` convenience predicates
  - [x] 6.6 Write unit tests for default mode, toggle, and predicate methods
  - Covers: Requirement 1 (AC 1.4), Requirement 3 (AC 3.3, 3.4, 3.8)

- [x] 7. Insert mode — character insertion
  - [x] 7.1 Implement `insert_char(doc, position, ch, mode_manager) -> EditResult` that inserts a character at the caret in Insert Mode
  - [x] 7.2 Handle grapheme cluster detection: use `unicode-segmentation` to treat multi-code-point sequences as single units
  - [x] 7.3 Handle virtual space realisation: pad with spaces when caret is beyond line end
  - [x] 7.4 Advance caret one grapheme cluster position after insertion
  - [x] 7.5 Set modified line marker on the affected line
  - [x] 7.6 Return `EditorTransaction` with before/after snapshots
  - [x] 7.7 Implement tab insertion (literal tab or configured space count)
  - [x] 7.8 Write unit tests for basic insertion, grapheme clusters, virtual space, tab handling
  - Covers: Requirement 1 (AC 1.1–1.8)

- [x] 8. Overstrike mode — character replacement
  - [x] 8.1 Implement `overstrike_char(doc, position, ch) -> EditResult` that replaces the character at caret position
  - [x] 8.2 Handle end-of-line case: append character when caret is at or beyond line end
  - [x] 8.3 Handle active selection case: delete selection and insert character (same as Insert Mode with selection)
  - [x] 8.4 Set modified line marker and produce EditorTransaction with original character in before-snapshot
  - [x] 8.5 Write unit tests for mid-line replacement, end-of-line append, and selection-active case
  - Covers: Requirement 3 (AC 3.1, 3.2, 3.5, 3.6, 3.7)

- [x] 9. NewLine handling
  - [x] 9.1 Implement `newline_insert_mode(doc, position, line_ending) -> EditResult` that splits line at caret
  - [x] 9.2 Text before caret stays on current line; text from caret becomes new line below
  - [x] 9.3 Move caret to column 1 of new line; use document's configured line ending style
  - [x] 9.4 Handle active selection: delete selection first, then split
  - [x] 9.5 Implement `newline_overstrike_mode(doc, position) -> EditResult` that moves caret to beginning of next line without splitting
  - [x] 9.6 Record EditorTransaction with before/after snapshots of all affected lines
  - [x] 9.7 Write unit tests for insert-mode split, overstrike-mode move, selection-then-split, line ending styles
  - Covers: Requirement 2 (AC 2.1–2.6)

- [x] 10. Delete operations — character granularity
  - [x] 10.1 Implement `delete_back(doc, position) -> EditResult` (Backspace: delete grapheme before caret)
  - [x] 10.2 Implement line-join on Backspace at column 1: join current line to end of previous
  - [x] 10.3 Implement `delete_forward(doc, position) -> EditResult` (Delete: delete grapheme at caret)
  - [x] 10.4 Implement line-join on Delete at end of line: join next line to current
  - [x] 10.5 Handle virtual space Backspace: move caret to actual line end without modifying document
  - [x] 10.6 Handle active selection: delete entire selection, collapse caret to selection start
  - [x] 10.7 Set modified line markers and record EditorTransactions
  - [x] 10.8 Write unit tests for each delete scenario including line joins and virtual space
  - Covers: Requirement 4 (AC 4.1–4.4, 4.10–4.12)

- [x] 11. Delete operations — word and line granularity
  - [x] 11.1 Implement `delete_word_left(doc, position) -> EditResult` (Ctrl+Backspace)
  - [x] 11.2 Implement `delete_word_right(doc, position) -> EditResult` (Ctrl+Delete)
  - [x] 11.3 Implement `delete_line(doc, line_number) -> EditResult` (Ctrl+Shift+K)
  - [x] 11.4 Implement `delete_to_line_end(doc, position) -> EditResult` (Ctrl+Shift+Delete)
  - [x] 11.5 Implement `delete_to_line_start(doc, position) -> EditResult` (Ctrl+Shift+Backspace)
  - [x] 11.6 Write unit tests for word boundary detection, full-line delete, partial-line deletes
  - Covers: Requirement 4 (AC 4.5–4.9, 4.11)

- [x] 12. Line manipulation commands
  - [x] 12.1 Implement `line_transpose(doc, line_number) -> EditResult` swapping current line with line above
  - [x] 12.2 Handle first-line edge case: no-op, no transaction recorded
  - [x] 12.3 Implement `line_duplicate(doc, line_number_or_selection) -> EditResult` inserting copy below
  - [x] 12.4 Implement `uppercase(doc, range) -> EditResult` converting selection to Unicode uppercase
  - [x] 12.5 Implement `lowercase(doc, range) -> EditResult` converting selection to Unicode lowercase
  - [x] 12.6 Implement `toggle_case(doc, range) -> EditResult` inverting case of each alphabetic character
  - [x] 12.7 Handle no-selection case: operate on entire current line for all commands
  - [x] 12.8 Record single EditorTransaction and set modified line markers for all affected lines
  - [x] 12.9 Write unit tests for transpose, duplicate, case operations, no-selection fallback, first-line no-op
  - Covers: Requirement 5 (AC 5.1–5.8)

- [x] 13. Selection model — keyboard-driven selection
  - [x] 13.1 Implement `extend_selection(container, direction, shift_held)` for Shift+Arrow extending
  - [x] 13.2 Implement Shift+Home (extend to line start) and Shift+End (extend to line end)
  - [x] 13.3 Implement Shift+Ctrl+Left/Right (extend by word)
  - [x] 13.4 Implement Shift+PageUp/PageDown (extend by viewport page)
  - [x] 13.5 Implement `select_all(doc) -> SelectionRange` selecting entire document
  - [x] 13.6 Implement selection collapse on unshifted arrow key (caret moves to appropriate end)
  - [x] 13.7 Implement selection replacement: typing with selection active deletes selection and inserts
  - [x] 13.8 Write unit tests for each extension direction, collapse behaviour, and replacement
  - Covers: Requirement 6 (AC 6.4–6.11, 6.17)

- [x] 14. Selection model — mouse-driven selection
  - [x] 14.1 Implement `click_set_caret(position)` placing caret and clearing selection
  - [x] 14.2 Implement `shift_click_extend(position)` extending selection from anchor to clicked position
  - [x] 14.3 Implement `drag_select(start, current)` creating stream selection from drag
  - [x] 14.4 Implement `double_click_select_word(position)` selecting entire word at position
  - [x] 14.5 Implement `triple_click_select_line(line)` selecting entire line including line ending
  - [x] 14.6 Write unit tests for click, shift-click, drag, double-click, triple-click
  - Covers: Requirement 6 (AC 6.12–6.16)

- [x] 15. Multi-caret editing — caret management
  - [x] 15.1 Implement `add_caret(container, position)` via Ctrl+Click adding new caret
  - [x] 15.2 Implement `remove_caret(container, position)` via Ctrl+Click on existing caret (Drop, min 1 remains)
  - [x] 15.3 Implement `add_caret_above(container, main_range)` (Ctrl+Alt+Up)
  - [x] 15.4 Implement `add_caret_below(container, main_range)` (Ctrl+Alt+Down)
  - [x] 15.5 Implement `escape_to_single_caret(container)` reducing to main range only
  - [x] 15.6 Implement `select_next_occurrence(doc, container)` (Ctrl+D) adding caret at next occurrence
  - [x] 15.7 Write unit tests for add, remove, above/below, escape, and select-next-occurrence
  - Covers: Requirement 8 (AC 8.1–8.3, 8.6, 8.9–8.11, 8.14)

- [x] 16. Multi-caret editing — coordinated operations
  - [x] 16.1 Implement reverse-document-order processing for multi-caret insert/delete
  - [x] 16.2 Implement multi-caret character insertion (same char at all carets)
  - [x] 16.3 Implement multi-caret deletion (Backspace/Delete at all carets)
  - [x] 16.4 Implement multi-caret navigation (all carets move same direction)
  - [x] 16.5 Implement post-operation trim: merge carets that collapse to same position
  - [x] 16.6 Handle protected range skipping: skip insertion at protected caret positions
  - [x] 16.7 Handle virtual space realisation at individual caret positions
  - [x] 16.8 Wrap all sub-operations in single UndoGroup for atomic undo
  - [x] 16.9 Write unit tests for multi-caret insert, delete, merge, protected skip, virtual space, UndoGroup
  - Covers: Requirement 8 (AC 8.4, 8.5, 8.7, 8.8, 8.12, 8.13, 8.15, 8.16)

- [x] 17. Rectangular/column selection
  - [x] 17.1 Define `RectangularSelection` struct with `top_line`, `bottom_line`, `left_column`, `right_column`
  - [x] 17.2 Implement `from_alt_drag(start, current)` creating rectangular selection from Alt+drag
  - [x] 17.3 Implement `extend(direction)` for Alt+Shift+Arrow extending the rectangle
  - [x] 17.4 Implement `to_selection_ranges() -> Vec<SelectionRange>` converting rectangle to one range per line
  - [x] 17.5 Implement column insert: insert character at left edge on every line
  - [x] 17.6 Implement column delete: remove selected column region on every affected line
  - [x] 17.7 Handle short lines: treat missing columns as virtual space
  - [x] 17.8 Implement `collapse_to_caret()` on Escape
  - [x] 17.9 Implement column select mode toggle (stream ↔ rectangular)
  - [x] 17.10 Write unit tests for creation, extension, conversion, insert, delete, short lines, toggle
  - Covers: Requirement 9 (AC 9.1–9.10)

- [x] 18. Clipboard integration (edit-side semantics)
  - [x] 18.1 Implement `copy(container, doc) -> ClipboardContent` copying selected text (or full line if no selection)
  - [x] 18.2 Implement `cut(container, doc) -> (ClipboardContent, EditResult)` copying and deleting selected text
  - [x] 18.3 Implement `paste(container, doc, content) -> EditResult` inserting clipboard content at caret (or replacing selection)
  - [x] 18.4 Implement line-copy detection: copy full line when no selection, paste as new line above
  - [x] 18.5 Implement multi-caret copy: concatenate from each caret's selection with separator
  - [x] 18.6 Implement multi-caret paste distribution: segment count matches caret count → one segment per caret
  - [x] 18.7 Implement rectangular copy: preserve rectangular structure with metadata
  - [x] 18.8 Implement rectangular paste: insert as column block starting at caret position
  - [x] 18.9 Handle clipboard failure: return descriptive error without modifying document
  - [x] 18.10 Write unit tests for copy, cut, paste, line-copy, multi-caret, rectangular, and failure handling
  - Covers: Requirement 10 (AC 10.1–10.12)

- [x] 19. Edit boundaries (BOUNDS)
  - [x] 19.1 Define `EditBounds` struct with `left_column: u64` and `right_column: u64` (both 1-based)
  - [x] 19.2 Implement `set_bounds(left, right) -> Result<(), EditError>` with validation (left >= 1, right > left)
  - [x] 19.3 Implement `clear_bounds()` resetting to unrestricted editing
  - [x] 19.4 Implement bounds-checking wrapper for insert operations: reject characters outside bounds
  - [x] 19.5 Implement bounds-checking wrapper for overstrike operations
  - [x] 19.6 Implement bounds-restricted line split (Enter): content outside bounds stays on original line
  - [x] 19.7 Implement bounds-restricted delete: only affect characters within bounded range
  - [x] 19.8 Implement bounds-restricted paste: clip pasted content to fit within bounds
  - [x] 19.9 Implement bounds-restricted selection edit: only affect portion within bounds
  - [x] 19.10 Implement per-document bounds state (each document/tab has independent BOUNDS)
  - [x] 19.11 Write unit tests for set/clear, insert/overstrike/delete/paste within and outside bounds, validation
  - Covers: Requirement 13 (AC 13.1–13.12)

- [x] 20. Transaction recording and modified line markers
  - [x] 20.1 Define `EditorTransaction` struct with `before_snapshot: Vec<LineSnapshot>` and `after_snapshot: Vec<LineSnapshot>`
  - [x] 20.2 Define `LineSnapshot` struct with `line_number: u64` and `content: String`
  - [x] 20.3 Implement `TransactionStack` with push, pop (undo), and redo operations
  - [x] 20.4 Implement save-point tracking: record which transaction index corresponds to last save
  - [x] 20.5 Implement `UndoGroup` wrapper that wraps multiple sub-transactions for multi-caret undo
  - [x] 20.6 Implement modified line marker logic: set on edit, clear on save, recalculate on undo/redo relative to save point
  - [x] 20.7 Write unit tests for transaction push/pop, redo, save points, UndoGroup, and marker state
  - Covers: Requirement 11 (AC 11.1–11.9)

- [x] 21. Save operations integration
  - [x] 21.1 Implement `save_command_handler(doc, path)` performing atomic temp-file write and rename
  - [x] 21.2 Clear all modified line markers on successful save
  - [x] 21.3 Record save point on TransactionStack after successful save
  - [x] 21.4 Handle save failure: preserve markers, display error, document stays modified
  - [x] 21.5 Register save command with command framework (Ctrl+S binding, metadata)
  - [x] 21.6 Write unit tests for successful save, failed save, save-point recording
  - Covers: Requirement 12 (AC 12.1–12.5)

- [x] 22. Command framework integration
  - [x] 22.1 Register insert-character command with CommandRegistry (metadata, handler, key binding)
  - [x] 22.2 Register delete commands (Backspace, Delete, Ctrl+Backspace, Ctrl+Delete, LineDelete, DelLineRight, DelLineLeft)
  - [x] 22.3 Register mode toggle command (Insert key → toggle insert/overstrike)
  - [x] 22.4 Register line manipulation commands (LineTranspose, LineDuplicate, Uppercase, Lowercase, ToggleCase)
  - [x] 22.5 Register selection commands (SelectAll, context menu Cut/Copy/Paste)
  - [x] 22.6 Register BOUNDS command handler parsing left/right column arguments
  - [x] 22.7 Ensure all handlers return success/failure result for status bar reporting
  - [x] 22.8 Ensure no GUI dependency in handlers — operate on logical document model only
  - [x] 22.9 Write integration tests verifying command dispatch triggers correct edit operations
  - Covers: Requirement 15 (AC 15.1–15.6)

- [x] 23. Error types and error handling
  - [x] 23.1 Define `EditError` enum with variants: `LineOutOfRange`, `ColumnOutOfRange`, `BoundsViolation`, `ReadOnlyDocument`, `ClipboardUnavailable`, `InvalidBounds`, `SelectionContainerEmpty`
  - [x] 23.2 Implement `Display` for all variants following `[edit-operations] operation: description` format
  - [x] 23.3 Implement `From` conversions for document-model errors
  - [x] 23.4 Write unit tests for error formatting and conversion
  - Covers: Cross-cutting Requirement 8 (Error Message Standards)

- [x] 24. Property-based tests — selection model invariants
  - [x] 24.1 Write property test: SelectionContainer always maintains ranges in sorted document order after any Add/Drop/Trim sequence
    - **Validates: Requirement 14.1, 14.3**
  - [x] 24.2 Write property test: MovePositions never produces negative positions (all positions remain >= 0) for arbitrary DocumentModification inputs
    - **Validates: Requirement 7.1–7.4, 14.4**
  - [x] 24.3 Write property test: Trim operation is idempotent (trim(trim(container)) == trim(container)) and eliminates all overlaps
    - **Validates: Requirement 14.3, 7.7**
  - [x] 24.4 Write property test: SelectionContainer always has count() >= 1 after any sequence of Add/Drop operations
    - **Validates: Requirement 14.2, 14.8**

- [x] 25. Property-based tests — insert/delete invariants
  - [x] 25.1 Write property test: insert_char followed by delete_back at same position is identity (document unchanged) for any valid position and printable character
    - **Validates: Requirement 1.1, 4.1**
  - [x] 25.2 Write property test: in Insert Mode, inserting N characters advances the caret exactly N grapheme positions forward
    - **Validates: Requirement 1.2, 1.3**
  - [x] 25.3 Write property test: in Overstrike Mode, line length never increases when character is replaced at a position before end-of-line
    - **Validates: Requirement 3.1**
  - [x] 25.4 Write property test: every edit operation produces a non-empty EditorTransaction with valid before/after snapshots
    - **Validates: Requirement 11.1–11.3**

- [x] 26. Property-based tests — multi-caret and bounds invariants
  - [x] 26.1 Write property test: multi-caret insert in reverse order produces the same result regardless of the number of carets (no position drift)
    - **Validates: Requirement 8.4, 8.5**
  - [x] 26.2 Write property test: after multi-caret operation, Trim merges any coincident carets, and count never exceeds the pre-operation count
    - **Validates: Requirement 8.8, 8.13**
  - [x] 26.3 Write property test: BOUNDS enforcement — any character insertion with BOUNDS active never modifies columns outside [left, right] range for any line content and caret position
    - **Validates: Requirement 13.2, 13.3, 13.5**
  - [x] 26.4 Write property test: rectangular selection to_selection_ranges() produces exactly (bottom_line - top_line + 1) ranges, each spanning [left_column, right_column]
    - **Validates: Requirement 9.1, 9.2**

- [x] 27. Property-based tests — transaction and undo invariants
  - [x] 27.1 Write property test: undo followed by redo restores document to post-edit state for any single edit operation
    - **Validates: Requirement 11.4, 11.5**
  - [x] 27.2 Write property test: modified line markers are set for every line whose content differs from saved state, and cleared for every line matching saved state, after any sequence of edits and undos
    - **Validates: Requirement 11.6, 11.7, 11.8**
  - [x] 27.3 Write property test: UndoGroup atomicity — undoing a multi-caret operation reverses ALL sub-operations in a single undo step
    - **Validates: Requirement 11.9, 8.13**

---

## Notes

- The `ff-edit-operations` crate has zero GUI dependencies — it operates on abstract types from `ff-document-model` and produces transaction records for `ff-undo-redo`.
- Tasks 20–21 (TransactionStack, save) coordinate closely with `ff-undo-redo` — the boundary is that this crate defines what constitutes a transaction unit, while `ff-undo-redo` owns coalescing and recovery mechanics.
- Property-based tests (Tasks 24–27) use the `proptest` crate and are configured for a minimum of 256 iterations to catch edge cases in position arithmetic and multi-caret coordination.
- BOUNDS (Task 19) is an ISPF heritage feature unique to FileForgeWorkbench — no equivalent in Scintilla or mainstream editors.
- Multi-caret reverse-order processing (Task 16) is critical for correctness: forward-order processing causes position drift as earlier insertions shift later positions.
- The `unicode-segmentation` crate is used for grapheme cluster boundaries — essential for correct cursor movement and deletion with multi-code-point characters (emoji, combining marks).

---

## Acceptance Criteria Coverage Map

| Task | Requirements Covered |
|------|---------------------|
| 1 | Structural scaffolding (all) |
| 2 | Req 6 (AC 6.2), Req 7 (AC 7.1–7.4) |
| 3 | Req 6 (AC 6.1), Req 14 (AC 14.3) |
| 4 | Req 14 (AC 14.1–14.9) |
| 5 | Req 7 (AC 7.1–7.7), Req 14 (AC 14.4) |
| 6 | Req 1 (AC 1.4), Req 3 (AC 3.3, 3.4, 3.8) |
| 7 | Req 1 (AC 1.1–1.8) |
| 8 | Req 3 (AC 3.1, 3.2, 3.5, 3.6, 3.7) |
| 9 | Req 2 (AC 2.1–2.6) |
| 10 | Req 4 (AC 4.1–4.4, 4.10–4.12) |
| 11 | Req 4 (AC 4.5–4.9, 4.11) |
| 12 | Req 5 (AC 5.1–5.8) |
| 13 | Req 6 (AC 6.4–6.11, 6.17) |
| 14 | Req 6 (AC 6.12–6.16) |
| 15 | Req 8 (AC 8.1–8.3, 8.6, 8.9–8.11, 8.14) |
| 16 | Req 8 (AC 8.4, 8.5, 8.7, 8.8, 8.12, 8.13, 8.15, 8.16) |
| 17 | Req 9 (AC 9.1–9.10) |
| 18 | Req 10 (AC 10.1–10.12) |
| 19 | Req 13 (AC 13.1–13.12) |
| 20 | Req 11 (AC 11.1–11.9) |
| 21 | Req 12 (AC 12.1–12.5) |
| 22 | Req 15 (AC 15.1–15.6) |
| 23 | Cross-cutting Req 8 (Error Message Standards) |
| 24 | PBT: Req 7, 14 (selection model invariants) |
| 25 | PBT: Req 1, 3, 4, 11 (insert/delete invariants) |
| 26 | PBT: Req 8, 9, 13 (multi-caret and bounds invariants) |
| 27 | PBT: Req 11, 8 (transaction and undo invariants) |

---

## Task Dependency Graph

```json
{
  "taskDependencies": {
    "1": [],
    "2": ["1"],
    "3": ["2"],
    "4": ["2", "3"],
    "5": ["4"],
    "6": ["1"],
    "7": ["6", "2"],
    "8": ["6", "7"],
    "9": ["7"],
    "10": ["7", "2"],
    "11": ["10"],
    "12": ["7", "2"],
    "13": ["4", "3"],
    "14": ["4", "3"],
    "15": ["4"],
    "16": ["15", "5", "7", "10"],
    "17": ["4", "7", "10"],
    "18": ["4", "7", "10", "17"],
    "19": ["7", "8", "10"],
    "20": ["7", "10"],
    "21": ["20"],
    "22": ["7", "8", "9", "10", "11", "12", "6", "19"],
    "23": ["1"],
    "24": ["4", "5"],
    "25": ["7", "8", "10", "20"],
    "26": ["16", "17", "19"],
    "27": ["20", "16"]
  },
  "externalDependencies": {
    "ff-document-model": "Provides GapBuffer, TextBuffer, Document, LineIndex — all edit operations mutate through this API",
    "ff-command": "Command registry, dispatch, metadata — all edit operations are registered commands",
    "ff-undo-redo": "TransactionStack, UndoGroup — transaction recording mechanics (Tasks 20-21 coordinate with this crate)",
    "ff-logging": "Structured logging for error reporting and diagnostics"
  },
  "waves": [
    {
      "id": 0,
      "label": "Foundation types",
      "tasks": ["1", "2", "3", "23"],
      "description": "Crate scaffolding, position/range types, error types"
    },
    {
      "id": 1,
      "label": "Selection container and edit mode",
      "tasks": ["4", "5", "6"],
      "description": "Selection container with all operations, position adjustment, mode management",
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "Core editing operations",
      "tasks": ["7", "8", "9", "10", "11", "12"],
      "description": "Insert, overstrike, newline, delete, line manipulation",
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "Selection model and multi-caret",
      "tasks": ["13", "14", "15", "16"],
      "description": "Keyboard/mouse selection, multi-caret management and coordination",
      "dependsOn": [1, 2]
    },
    {
      "id": 4,
      "label": "Advanced editing features",
      "tasks": ["17", "18", "19", "20", "21"],
      "description": "Rectangular selection, clipboard, BOUNDS, transactions, save",
      "dependsOn": [2, 3]
    },
    {
      "id": 5,
      "label": "Integration and registration",
      "tasks": ["22"],
      "description": "Command framework registration for all edit operations",
      "dependsOn": [2, 3, 4]
    },
    {
      "id": 6,
      "label": "Property-based tests",
      "tasks": ["24", "25", "26", "27"],
      "description": "Property tests validating invariants across all major subsystems",
      "dependsOn": [1, 2, 3, 4]
    }
  ]
}
```

---

## Phase BW Tasks -- EARS Integration (Requirements 16-17)

- [ ] 28. CAPS mode
  - [ ] 28.1 Write failing test: CAPS ON converts typed characters to uppercase before insert
    - // Validates: Requirement 16.1
  - [ ] 28.2 Write failing test: CAPS OFF reverts to case-preserving input
    - // Validates: Requirement 16.1
  - [ ] 28.3 Write failing test: CAPS with no argument toggles state
    - // Validates: Requirement 16.2
  - [ ] 28.4 Implement `CapsMode` flag on editor state; apply in insert_char path
  - [ ] 28.5 Register CAPS command with command framework
  - [ ] 28.6 cargo test green; cargo clippy clean

- [ ] 29. NULLS mode
  - [ ] 29.1 Write failing test: NULLS ON treats trailing nulls as trailing spaces
    - // Validates: Requirement 16.4
  - [ ] 29.2 Write failing test: NULLS OFF displays null characters as visible placeholders
    - // Validates: Requirement 16.4
  - [ ] 29.3 Implement `NullsMode` flag; apply in display and edit paths
  - [ ] 29.4 Register NULLS command with command framework
  - [ ] 29.5 cargo test green; cargo clippy clean

- [ ] 30. PROFILE command
  - [ ] 30.1 Write failing test: PROFILE command returns current profile settings as a structured value
    - // Validates: Requirement 16.5
  - [ ] 30.2 Write failing test: PROFILE CAPS ON updates CAPS setting
    - // Validates: Requirement 16.6
  - [ ] 30.3 Implement `EditProfile` struct holding all profile settings
  - [ ] 30.4 Implement PROFILE command handler (display and update paths)
  - [ ] 30.5 Register PROFILE command with command framework
  - [ ] 30.6 cargo test green; cargo clippy clean

- [ ] 31. STATS mode
  - [ ] 31.1 Write failing test: STATS ON sets stats_visible flag on editor state
    - // Validates: Requirement 16.7
  - [ ] 31.2 Write failing test: STATS OFF clears stats_visible flag
    - // Validates: Requirement 16.7
  - [ ] 31.3 Implement `StatsMode` flag; wire into prefix area rendering
  - [ ] 31.4 Register STATS command with command framework
  - [ ] 31.5 cargo test green; cargo clippy clean

- [ ] 32. LOCK setting
  - [ ] 32.1 Write failing test: LOCK ON prevents profile setting changes
    - // Validates: Requirement 16.8
  - [ ] 32.2 Write failing test: LOCK OFF re-enables profile changes
    - // Validates: Requirement 16.8
  - [ ] 32.3 Implement `ProfileLock` flag; guard all profile-mutating commands
  - [ ] 32.4 Register LOCK command with command framework
  - [ ] 32.5 cargo test green; cargo clippy clean

- [ ] 33. Edit profile persistence
  - [ ] 33.1 Write failing test: EditProfile round-trips through session TOML
    - // Validates: Requirement 16.9
  - [ ] 33.2 Implement EditProfile serialisation/deserialisation via ff-session
  - [ ] 33.3 Wire profile save on file close; profile restore on file open
  - [ ] 33.4 cargo test green; cargo clippy clean

- [ ] 34. AUTONUM and NUM aliases
  - [ ] 34.1 Write failing test: AUTONUM ON dispatches to NUMBER ON handler
    - // Validates: Requirement 16.10
  - [ ] 34.2 Write failing test: NUM SHOW dispatches to NUMBER SHOW handler
    - // Validates: Requirement 16.11
  - [ ] 34.3 Register AUTONUM and NUM as aliases in command framework
  - [ ] 34.4 cargo test green; cargo clippy clean

- [ ] 35. HILITE delegation
  - [ ] 35.1 Write failing test: HILITE ON dispatches to syntax-highlighting subsystem
    - // Validates: Requirement 16.12
  - [ ] 35.2 Write failing test: HILITE LOGIC dispatches with LOGIC mode argument
    - // Validates: Requirement 16.12
  - [ ] 35.3 Implement HILITE command handler delegating to ff-syntax
  - [ ] 35.4 Register HILITE command with command framework
  - [ ] 35.5 cargo test green; cargo clippy clean

- [ ] 36. SUBMIT primary command
  - [ ] 36.1 Write failing test: SUBMIT dispatches to JES subsystem with current buffer content
    - // Validates: Requirement 17.1
  - [ ] 36.2 Write failing test: SUBMIT with no JES available returns descriptive error
    - // Validates: Requirement 17.8
  - [ ] 36.3 Implement SUBMIT command handler; wire to ff-jes job submission API
  - [ ] 36.4 Register SUBMIT command with command framework
  - [ ] 36.5 cargo test green; cargo clippy clean

- [ ] 37. CREATE and REPLACE primary commands
  - [ ] 37.1 Write failing test: CREATE with dataset name creates dataset from selected lines
    - // Validates: Requirement 17.2
  - [ ] 37.2 Write failing test: REPLACE with dataset name replaces dataset content
    - // Validates: Requirement 17.3
  - [ ] 37.3 Write failing test: CREATE/REPLACE with missing argument returns error
    - // Validates: Requirement 17.8
  - [ ] 37.4 Implement CREATE and REPLACE command handlers
  - [ ] 37.5 Register CREATE and REPLACE commands with command framework
  - [ ] 37.6 cargo test green; cargo clippy clean

- [ ] 38. Nested EDIT, BROWSE, VIEW, COMPARE commands
  - [ ] 38.1 Write failing test: EDIT <dsn> from editor opens named dataset in new tab
    - // Validates: Requirement 17.4
  - [ ] 38.2 Write failing test: BROWSE <dsn> opens dataset in read-only tab
    - // Validates: Requirement 17.5
  - [ ] 38.3 Write failing test: VIEW <dsn> opens dataset in view tab
    - // Validates: Requirement 17.6
  - [ ] 38.4 Write failing test: COMPARE <dsn> opens compare view
    - // Validates: Requirement 17.7
  - [ ] 38.5 Write failing test: all four commands with invalid dsn return error, no tab opened
    - // Validates: Requirement 17.8
  - [ ] 38.6 Implement command handlers; wire to tab manager and compare subsystem
  - [ ] 38.7 Register all four commands with command framework
  - [ ] 38.8 cargo test green; cargo clippy clean

- [ ] 39. TCR.md and project-master updated; cargo test --workspace green
