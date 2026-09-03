# Implementation Plan: Hex Display Mode (`ff-hex`)

## Overview

This plan covers the complete implementation of the `ff-hex` crate — the hexadecimal viewing and editing subsystem for FileForgeWorkbench. The hex display mode provides a three-pane layout (offset column, hex pane, ASCII pane) with full overwrite editing, synchronised cursor movement, byte pattern search integration, hex dump export, goto-offset navigation, and undo/redo participation.

This is a **Wave 11 (Display Modes)** sub-project that depends on:
- `ff-document-model` (Wave 4) for byte buffer access via BytePosition addressing
- `ff-command` (Wave 5) for HEX command registration and dispatch
- `ff-undo-redo` (Wave 4) for recording hex edits as reversible transactions
- `ff-viewport` (Wave 4) for hex-specific viewport scrolling coordination
- `ff-find-replace` (Wave 5) for `FIND X'...'` hex search integration
- `ff-config` (Wave 2) for persistent hex mode settings

The crate is **GUI-independent** — it manages the hex display model, cursor synchronisation, and editing logic. Rendering is delegated to the UI layer.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-hex/Cargo.toml` with dependencies (thiserror, proptest dev-dep) and dependencies on `ff-document-model`, `ff-command`, `ff-undo-redo`, `ff-viewport`, `ff-config`, `ff-logging`
  - [x] 1.2 Create `crates/ff-hex/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `state.rs`, `layout.rs`, `cursor.rs`, `navigation.rs`, `editing.rs`, `search.rs`, `clipboard.rs`, `commands.rs`, `config.rs`, `session.rs`, `dump.rs`, `fileforge.rs`, `error.rs`, `types.rs`
  - [x] 1.4 Add `ff-hex` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Core types and error definitions
  - [x] 2.1 Define `ByteOffset(u64)` newtype with arithmetic ops, hex Display formatting, and From<u64> conversion
  - [x] 2.2 Define `NibblePosition` enum (High, Low) representing cursor position within a hex digit pair
  - [x] 2.3 Define `HexPane` enum (Hex, Ascii) representing which pane has editing focus
  - [x] 2.4 Define `HexDigitCase` enum (Uppercase, Lowercase) with formatting methods
  - [x] 2.5 Define `BytesPerRow` validated newtype accepting only 8, 16, 32, 64 with TryFrom<u8>
  - [x] 2.6 Define `HexModeState` enum (Active, Inactive) for tracking mode transitions
  - [x] 2.7 Define `ModifiedByte` struct tracking byte offset and original vs current value
  - [x] 2.8 Define `HexError` enum with variants: InvalidHexDigit, InvalidBytesPerRow, OffsetOutOfRange, ReadOnlyMode, InvalidHexPattern, HexModeAlreadyActive, HexModeAlreadyInactive
  - [x] 2.9 Write unit tests for ByteOffset formatting, BytesPerRow validation, NibblePosition transitions
  - Covers: Requirement 1 (AC 1.4–1.5), Requirement 2 (AC 2.3), Requirement 3 (AC 3.1–3.4), Requirement 4 (AC 4.4), Requirement 13

- [x] 3. Hex mode state management
  - [x] 3.1 Implement `HexSession` struct holding: HexModeState, active pane, cursor byte offset, viewport top row, BytesPerRow, HexDigitCase, modified bytes set
  - [x] 3.2 Implement `activate()` method that transitions from Inactive to Active, mapping current text cursor BytePosition to hex row/column
  - [x] 3.3 Implement `deactivate()` method that transitions from Active to Inactive, mapping hex cursor byte offset back to text line/column
  - [x] 3.4 Implement `toggle()` method that calls activate or deactivate based on current state
  - [x] 3.5 Implement idempotent guards: activate when already active returns status message, deactivate when already inactive returns status message
  - [x] 3.6 Implement mode validity check across Browse, Edit, and View modes
  - [x] 3.7 Implement status bar indicator query `is_hex_active() -> bool`
  - [x] 3.8 Verify hex mode state change is NOT added to undo stack (display-only state)
  - [x] 3.9 Write unit tests for state transitions, idempotent guards, cursor position mapping on activation/deactivation
  - Covers: Requirement 1 (AC 1.1–1.10)

- [x] 4. Hex layout model (offset, hex, ASCII columns)
  - [x] 4.1 Implement `HexRow` struct containing: row_offset (ByteOffset), bytes (Vec<u8>), byte_count (number of valid bytes in this row)
  - [x] 4.2 Implement `HexLayout` struct computing column positions: offset_column_width, hex_pane_start, hex_pane_width, ascii_pane_start, ascii_pane_width based on BytesPerRow
  - [x] 4.3 Implement offset column formatting: zero-padded 8-digit hex (expanding to more digits for documents > 4 GB)
  - [x] 4.4 Implement hex pane formatting: two uppercase/lowercase hex digits per byte, single space separator, additional space after every 8-byte group when BytesPerRow >= 16
  - [x] 4.5 Implement ASCII pane formatting: printable chars (0x20–0x7E) shown directly, non-printable bytes shown as `.`
  - [x] 4.6 Implement column separator character/theme token placement between offset, hex, and ASCII regions
  - [x] 4.7 Implement final-row padding: incomplete rows padded with spaces to maintain column alignment in both hex and ASCII panes
  - [x] 4.8 Implement empty document handling: single row with offset `00000000` and empty panes
  - [x] 4.9 Implement `total_row_count(document_length, bytes_per_row)` calculation as `ceil(doc_len / bytes_per_row)`
  - [x] 4.10 Implement `byte_offset_to_row_col(offset, bytes_per_row)` mapping and inverse `row_col_to_byte_offset`
  - [x] 4.11 Implement layout recalculation on BytesPerRow change preserving cursor byte offset
  - [x] 4.12 Write unit tests for layout calculations, offset formatting, row generation, padding, empty document, and large-file offset expansion
  - Covers: Requirement 2 (AC 2.1–2.10), Requirement 3 (AC 3.3, 3.6)

- [x] 5. Hex cursor and navigation
  - [x] 5.1 Implement `HexCursor` struct with fields: byte_offset (ByteOffset), nibble_position (NibblePosition), active_pane (HexPane)
  - [x] 5.2 Implement pane switching via configurable key (Tab default): changes active_pane while preserving byte_offset
  - [x] 5.3 Implement hex pane arrow key navigation: Left/Right move by nibble, Up/Down move by BytesPerRow bytes
  - [x] 5.4 Implement ASCII pane arrow key navigation: Left/Right move by one byte, Up/Down move by BytesPerRow bytes
  - [x] 5.5 Implement row wrapping: Right at end of row advances to start of next row, Left at start of row moves to end of previous row
  - [x] 5.6 Implement cursor clamping: cursor cannot move past document end (last byte)
  - [x] 5.7 Implement offset column row highlighting: current row indicated when cursor moves
  - [x] 5.8 Implement cursor synchronisation: moving in hex pane updates ASCII pane highlight and vice versa
  - [x] 5.9 Write unit tests for pane switching, nibble navigation, byte navigation, wrapping, clamping, and synchronisation
  - Covers: Requirement 6 (AC 6.1–6.8)

- [x] 6. Hex editing (byte modification with undo)
  - [x] 6.1 Implement hex pane input handler: valid hex digit (0–9, A–F, a–f) overwrites current nibble, advances to next nibble
  - [x] 6.2 Implement nibble-to-byte completion: after low nibble entry, cursor advances to high nibble of next byte
  - [x] 6.3 Implement ASCII pane input handler: printable character (0x20–0x7E) overwrites current byte, updates hex pane
  - [x] 6.4 Implement invalid hex digit rejection: non-hex input ignored with status message "Invalid hex digit"
  - [x] 6.5 Implement read-only mode guard: reject edits in Browse/View mode with "Cannot edit in Browse/View mode" message
  - [x] 6.6 Implement Edit_Operation recording: each byte modification emitted as a reversible operation to undo-redo system
  - [x] 6.7 Implement nibble coalescing: consecutive high+low nibble edits on same byte coalesced into single undo transaction
  - [x] 6.8 Implement modified byte tracking: mark edited bytes in modified set with distinct highlight until save
  - [x] 6.9 Implement EBCDIC warning: display "Hex editing on EBCDIC files modifies raw bytes directly" warning on first hex pane entry when EBCDIC encoding is active
  - [x] 6.10 Write unit tests for hex digit entry, nibble advancement, ASCII overwrite, invalid input rejection, read-only guard, undo recording, and coalescing
  - Covers: Requirement 4 (AC 4.1–4.9), Requirement 7 (AC 7.1–7.6), Requirement 8 (AC 8.1–8.5)

- [x] 7. Undo/redo in hex mode
  - [x] 7.1 Implement `HexEditOperation` struct implementing the Edit_Operation trait with byte offset, old value, new value
  - [x] 7.2 Implement undo handler: restore original byte value, update both hex and ASCII panes
  - [x] 7.3 Implement redo handler: re-apply byte change, update both panes
  - [x] 7.4 Implement nibble pair coalescing into single transaction (high nibble + low nibble = one undo unit)
  - [x] 7.5 Implement modified byte indicator update on undo/redo: compare current value to saved-state value
  - [x] 7.6 Implement shared undo stack: hex edits and text edits share same undo stack
  - [x] 7.7 Write unit tests for undo/redo cycle, coalescing, modified indicator tracking, and shared stack behaviour
  - Covers: Requirement 7 (AC 7.1–7.6)

- [x] 8. Modified byte indicators
  - [x] 8.1 Implement `ModifiedByteTracker` maintaining a HashSet<ByteOffset> of bytes differing from last-saved state
  - [x] 8.2 Implement `mark_modified(offset, original_value)` when byte is edited
  - [x] 8.3 Implement `clear_all()` on document save — all indicators removed
  - [x] 8.4 Implement `unmark_if_restored(offset, current_value, saved_value)` on undo — remove indicator if value matches saved state
  - [x] 8.5 Implement multi-modify-undo cycle correctness: byte modified, undone, re-modified tracks correctly
  - [x] 8.6 Implement theme token query for modified byte highlight (`hex.modified_byte`)
  - [x] 8.7 Write unit tests for mark, clear, unmark, and multi-cycle tracking
  - Covers: Requirement 8 (AC 8.1–8.5)

- [x] 9. Hex search integration (FIND X'...')
  - [x] 9.1 Implement `HexPattern` parser: validate hex string is even length, parse digit pairs into byte sequence
  - [x] 9.2 Implement odd-digit-count error: "Hex pattern must contain an even number of digits"
  - [x] 9.3 Implement byte sequence matcher: raw byte matching without Unicode case folding
  - [x] 9.4 Implement auto-activate: when hex match found and hex mode is inactive, activate hex mode before highlighting
  - [x] 9.5 Implement match highlighting: highlight matched byte range in both hex pane (digit pairs) and ASCII pane (characters)
  - [x] 9.6 Implement scope modifier support: ALL, NEXT, PREV, FIRST, LAST passed through to find engine
  - [x] 9.7 Implement viewport scroll to match: scroll to reveal matching row, position cursor at first byte of match
  - [x] 9.8 Write unit tests for pattern parsing, validation, byte matching, auto-activate, highlighting, and scope modifiers
  - Covers: Requirement 5 (AC 5.1–5.8)

- [x] 10. Hex copy/paste (clipboard integration)
  - [x] 10.1 Implement hex pane copy: selected byte range copied as hex digit string (e.g., "0D0A4F")
  - [x] 10.2 Implement ASCII pane copy: selected byte range copied as ASCII text (non-printable as `.`)
  - [x] 10.3 Implement hex pane paste: validate clipboard content as hex digit pairs, overwrite bytes starting at cursor
  - [x] 10.4 Implement ASCII pane paste: overwrite bytes with pasted character ASCII values
  - [x] 10.5 Implement paste validation: reject invalid hex digit sequences on hex pane paste with error message
  - [x] 10.6 Write unit tests for copy format, paste overwrite, and validation
  - Covers: Requirement 4 (AC 4.1–4.3), cross-cutting clipboard integration

- [x] 11. Hex dump export
  - [x] 11.1 Implement `HexDump` formatter producing three-column text output matching hex view layout
  - [x] 11.2 Implement full-document dump (no arguments)
  - [x] 11.3 Implement byte-range dump with start/end offset arguments
  - [x] 11.4 Implement selection-based dump: export only currently selected bytes when selection exists
  - [x] 11.5 Implement output destinations: new editor tab (`HEX DUMP EDIT`), clipboard (`HEX DUMP CLIP`), file (`HEX DUMP FILE 'path'`)
  - [x] 11.6 Implement dump formatting respecting current BytesPerRow and HexDigitCase settings
  - [x] 11.7 Write unit tests for dump output format, range extraction, and setting application
  - Covers: Requirement 11 (AC 11.1–11.7)

- [x] 12. Goto offset command
  - [x] 12.1 Implement offset parser accepting hex (`X'1A4F'` or `0x1A4F`) and decimal (plain number) formats
  - [x] 12.2 Implement `goto_offset(target)`: position cursor at target byte, scroll viewport to reveal row
  - [x] 12.3 Implement auto-activate: if hex mode not active, activate before navigating to offset
  - [x] 12.4 Implement out-of-range guard: reject offset exceeding document size with "Offset X'...' exceeds document size" error
  - [x] 12.5 Implement offset column indication and cursor placement on successful navigation
  - [x] 12.6 Write unit tests for offset parsing (hex/decimal), navigation, auto-activate, and out-of-range rejection
  - Covers: Requirement 12 (AC 12.1–12.6)

- [x] 13. HEX command handler (command framework registration)
  - [x] 13.1 Register `HEX ON` command with handler that delegates to `HexSession::activate()`
  - [x] 13.2 Register `HEX OFF` command with handler that delegates to `HexSession::deactivate()`
  - [x] 13.3 Register `HEX` (no arg) command with handler that delegates to `HexSession::toggle()`
  - [x] 13.4 Register `HEX DUMP` command with subcommand routing (EDIT, CLIP, FILE, range args)
  - [x] 13.5 Register `GOTO` command with offset argument parsing (X'...' and 0x and decimal forms)
  - [x] 13.6 Implement command compatibility: ensure FIND, CHANGE, SORT, EXCLUDE, line commands operate normally when hex mode active
  - [x] 13.7 Implement FIND text highlighting in hex mode: highlight matching hex digits and ASCII pane characters
  - [x] 13.8 Implement CHANGE live update: when text is changed while hex mode active, hex display refreshes immediately
  - [x] 13.9 Write unit tests for command registration, dispatch, argument parsing, and cross-command compatibility
  - Covers: Requirement 1 (AC 1.1–1.3, 1.6), Requirement 11, Requirement 12, Requirement 16 (AC 16.1–16.4)

- [x] 14. Scrolling and viewport in hex mode
  - [x] 14.1 Implement hex viewport row count calculation: `ceil(document_byte_length / bytes_per_row)`
  - [x] 14.2 Implement Page Down: advance viewport by visible hex row count
  - [x] 14.3 Implement Page Up: move back by visible hex row count, clamped to row 0
  - [x] 14.4 Implement vertical scrollbar integration: map [0, total_rows) with proportional thumb
  - [x] 14.5 Implement cursor-follows-viewport: scroll to keep cursor row visible when cursor moves outside viewport
  - [x] 14.6 Implement horizontal scrollbar: hidden when row fits window width, enabled when it does not
  - [x] 14.7 Implement BytesPerRow change scrollbar recalculation without changing top byte offset
  - [x] 14.8 Implement complete-row rendering: never render partial rows at top or bottom of viewport
  - [x] 14.9 Write unit tests for row count, page navigation, clamping, scrollbar mapping, and layout change recalculation
  - Covers: Requirement 9 (AC 9.1–9.8)

- [x] 15. Binary file detection and auto-activation
  - [x] 15.1 Implement binary detection query: check document for null bytes or non-text sequences (delegates to encoding-and-characters)
  - [x] 15.2 Implement auto-activation behaviour: prompt user when binary detected (configurable: always, prompt, never)
  - [x] 15.3 Implement text file hex handling: show line-ending bytes faithfully (CR=0D, LF=0A visible)
  - [x] 15.4 Implement binary file hex handling: strict byte-stream display, no line-end interpretation
  - [x] 15.5 Implement chunked/streamed access for large files: only load visible byte range from VFS
  - [x] 15.6 Implement byte offset correctness accounting for gap buffer gap position
  - [x] 15.7 Write unit tests for binary detection, auto-activation config, text vs binary display behaviour, and offset correctness
  - Covers: Requirement 10 (AC 10.1–10.6)

- [x] 16. Configuration system integration
  - [x] 16.1 Implement `editor.hex.bytes_per_row` config key with default 16, valid values [8, 16, 32, 64]
  - [x] 16.2 Implement `editor.hex.digit_case` config key with default "uppercase", valid values ["uppercase", "lowercase"]
  - [x] 16.3 Implement `editor.hex.auto_activate_binary` config key with default "prompt", valid values ["always", "prompt", "never"]
  - [x] 16.4 Implement live config reload: changing settings while hex mode active updates display immediately
  - [x] 16.5 Implement invalid config value rejection with error message and retention of current value
  - [x] 16.6 Write unit tests for config loading, validation, live update, and invalid value handling
  - Covers: Requirement 3 (AC 3.1–3.6), Requirement 13 (AC 13.1–13.7), Requirement 10 (AC 10.2)

- [x] 17. Hex mode session state persistence
  - [x] 17.1 Implement per-file session state struct: hex mode on/off, BytesPerRow, cursor byte offset, viewport top row, active pane
  - [x] 17.2 Implement session save: persist hex state to session history on file close
  - [x] 17.3 Implement session restore: re-activate hex mode and restore cursor/viewport when file reopened
  - [x] 17.4 Implement status bar HEX indicator query for persistent display
  - [x] 17.5 Write unit tests for session save/restore cycle, state completeness, and indicator correctness
  - Covers: Requirement 15 (AC 15.1–15.5)

- [x] 18. FileForge structured file integration
  - [x] 18.1 Implement field boundary detection from active Record_Structure offset/length definitions
  - [x] 18.2 Implement field boundary visual indicators in hex pane (separator or colour change at field boundaries)
  - [x] 18.3 Implement cell-to-hex mapping: selecting a FileForge grid cell highlights corresponding byte range in hex pane
  - [x] 18.4 Implement COMP-3 packed decimal annotation: decode and display numeric value as tooltip/inline annotation
  - [x] 18.5 Write unit tests for boundary detection, highlight mapping, and COMP-3 decoding
  - Covers: Requirement 14 (AC 14.1–14.4)

- [x] 19. Property-based tests
  - [x] 19.1 Write PBT: hex layout row generation correctness
  - [x] 19.2 Write PBT: cursor navigation boundary safety
  - [x] 19.3 Write PBT: hex edit undo/redo round-trip integrity
  - [x] 19.4 Write PBT: hex pattern search byte-level accuracy
  - [x] 19.5 Write PBT: modified byte indicator correctness under edit/undo cycles
  - [x] 19.6 Write PBT: viewport scroll clamping in hex mode
  - [x] 19.7 Write PBT: hex dump export content fidelity
  - Covers: Requirements 2, 4, 5, 6, 7, 8, 9, 11 (see Property-Based Test Definitions below)

- [x] 20. Integration tests
  - [x] 20.1 Write integration test: full hex mode lifecycle (activate → navigate → edit → undo → deactivate)
  - [x] 20.2 Write integration test: hex search with auto-activation and match highlighting
  - [x] 20.3 Write integration test: goto offset with viewport scroll and cursor positioning
  - [x] 20.4 Write integration test: hex dump export for full document, byte range, and selection
  - [x] 20.5 Write integration test: session state save and restore across file close/reopen
  - [x] 20.6 Write integration test: command compatibility — FIND, CHANGE, SORT operate correctly while hex mode active
  - [x] 20.7 Write integration test: binary file auto-detection and hex mode activation prompt
  - Covers: End-to-end validation across Requirements 1–16

---

## Property-Based Test Definitions

### Property 1: Hex Layout Row Generation Correctness

**Validates: Requirement 2.1, 2.3, 2.5, 2.7**

- **Statement:** For any document byte content and any valid BytesPerRow, the generated hex rows SHALL satisfy: (a) every byte in the document appears in exactly one row, (b) row offsets are strictly increasing by BytesPerRow, (c) the hex pane contains exactly two hex digits per byte, and (d) the ASCII pane shows `.` for non-printable bytes (outside 0x20–0x7E) and the actual character for printable bytes.
- **Strategy:** Generate:
  - Document content: arbitrary byte sequences (0–5000 bytes)
  - BytesPerRow: one of [8, 16, 32, 64]
- **Invariant:** Total bytes across all rows equals document length; offset[i] == i * bytes_per_row; hex digit pairs decode back to original bytes; ASCII pane chars match printability rules

### Property 2: Cursor Navigation Boundary Safety

**Validates: Requirement 6.6, 6.7, 6.8**

- **Statement:** For any document length and any sequence of navigation operations (left, right, up, down, pane switch), the cursor byte offset SHALL always remain in [0, document_length - 1] (or 0 for empty documents), the nibble position SHALL always be High or Low, and wrapping at row boundaries SHALL produce the correct adjacent byte.
- **Strategy:** Generate:
  - Document length: [0, 10000]
  - BytesPerRow: one of [8, 16, 32, 64]
  - Active pane: Hex or Ascii
  - Operation sequence: 50–200 random navigation operations
- **Invariant:** `0 <= cursor.byte_offset < max(1, doc_length)` after every operation; pane switch preserves byte_offset; row wrap moves to correct adjacent byte

### Property 3: Hex Edit Undo/Redo Round-Trip Integrity

**Validates: Requirement 7.1, 7.2, 7.3, 7.4**

- **Statement:** For any sequence of hex byte edits followed by an equal number of undo operations, the document content SHALL be byte-for-byte identical to its original state. For any undo followed by redo, the content SHALL match the post-edit state.
- **Strategy:** Generate:
  - Initial content: arbitrary bytes (1–2000 bytes)
  - Edit sequence: 5–50 random byte modifications at random valid offsets
- **Invariant:** After N edits + N undos: content == original; After N edits + K undos + K redos (K <= N): content == state_after_N_edits

### Property 4: Hex Pattern Search Byte-Level Accuracy

**Validates: Requirement 5.1, 5.6, 5.7**

- **Statement:** For any document content and any even-length hex search pattern, the search SHALL find a match at byte offset P if and only if the bytes at positions [P, P+pattern_len) are equal to the decoded pattern bytes. The search SHALL NOT apply Unicode case folding.
- **Strategy:** Generate:
  - Document content: arbitrary bytes (0–3000 bytes) with some known injected patterns
  - Search pattern: random even-length hex digit string (2–16 digits)
- **Invariant:** Every match position is a true byte-for-byte match; no false positives; no missed matches where the pattern exists

### Property 5: Modified Byte Indicator Correctness Under Edit/Undo Cycles

**Validates: Requirement 8.1, 8.4, 8.5**

- **Statement:** After any sequence of edit and undo operations, a byte SHALL be marked as modified if and only if its current value differs from its last-saved value. The indicator set SHALL exactly equal the set of byte offsets where `current_value != saved_value`.
- **Strategy:** Generate:
  - Initial content (saved state): arbitrary bytes (1–1000 bytes)
  - Operation sequence: 10–100 random edits and undos at random valid offsets
- **Invariant:** `modified_set == { offset | content[offset] != saved_content[offset] }` after every operation

### Property 6: Viewport Scroll Clamping in Hex Mode

**Validates: Requirement 9.1, 9.2, 9.3, 9.7, 9.8**

- **Statement:** For any document length, BytesPerRow, and viewport height, all scroll operations SHALL produce a top_row value in [0, max(0, total_rows - viewport_height)], and the viewport SHALL never display partial rows.
- **Strategy:** Generate:
  - Document length: [0, 100000]
  - BytesPerRow: one of [8, 16, 32, 64]
  - Viewport height (in rows): [1, 100]
  - Operation sequence: 20–100 random scroll operations (page_up, page_down, scroll_to_row, cursor_follow)
- **Invariant:** `0 <= top_row <= max(0, total_rows - viewport_height)` after every operation; repeated boundary scroll is idempotent

### Property 7: Hex Dump Export Content Fidelity

**Validates: Requirement 11.2, 11.4, 11.6**

- **Statement:** For any document content and any export range, parsing the hex dump output back into bytes SHALL produce byte-for-byte identical content to the original range. The dump SHALL use the configured BytesPerRow and HexDigitCase.
- **Strategy:** Generate:
  - Document content: arbitrary bytes (0–5000 bytes)
  - Export range: random valid [start, end) within document, or full document
  - BytesPerRow: one of [8, 16, 32, 64]
  - HexDigitCase: Uppercase or Lowercase
- **Invariant:** `parse_hex_dump(export(content, range, settings)) == content[range]`; digit case matches setting; rows contain exactly BytesPerRow bytes (except final row)

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2"], "dependsOn": [0] },
    { "id": 2, "label": "State and Layout", "tasks": ["3", "4"], "dependsOn": [1] },
    { "id": 3, "label": "Cursor and Navigation", "tasks": ["5"], "dependsOn": [2] },
    { "id": 4, "label": "Editing and Undo", "tasks": ["6", "7", "8"], "dependsOn": [3] },
    { "id": 5, "label": "Search and Goto", "tasks": ["9", "12"], "dependsOn": [4] },
    { "id": 6, "label": "Clipboard and Export", "tasks": ["10", "11"], "dependsOn": [4] },
    { "id": 7, "label": "Commands and Scrolling", "tasks": ["13", "14"], "dependsOn": [5, 6] },
    { "id": 8, "label": "Detection and Config", "tasks": ["15", "16"], "dependsOn": [7] },
    { "id": 9, "label": "Session and FileForge", "tasks": ["17", "18"], "dependsOn": [8] },
    { "id": 10, "label": "Validation and PBT", "tasks": ["19", "20"], "dependsOn": [9] }
  ]
}
```

---

## Notes

- This is a Wave 11 (Display Modes) crate depending on multiple upstream crates from Waves 2–5
- The crate is GUI-independent — all rendering is delegated to the UI layer; this crate provides model/state/logic only
- Hex edits share the same undo stack as text edits — no separate undo system
- The `FIND X'...'` integration is a coordination layer; the actual search engine lives in `ff-find-replace`
- FileForge integration (Task 18) is optional and only activated when FileForge_Mode is concurrently active
- Binary detection delegates to `ff-encoding` — this crate consumes the detection result
- Session state persistence uses the session history system from `ff-startup-session`
- Hex mode state changes (on/off) are NOT undoable — they are display-only state transitions
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The `ByteOffset(u64)` type supports documents up to 2^64 bytes, matching the document model's u64 addressing
- Offset column formatting auto-expands beyond 8 hex digits for files larger than 4 GB (0xFFFFFFFF)
- COMP-3 packed decimal annotation is an enhancement specific to FileForge-mode files with defined record structures

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: HEX ON/OFF/Toggle Commands | AC 1.1–1.10 | Tasks 3, 13 |
| Req 2: Hex View Layout | AC 2.1–2.10 | Task 4 |
| Req 3: Configurable Bytes Per Row | AC 3.1–3.6 | Tasks 2, 4, 16 |
| Req 4: Hex Editing (Overwrite Mode) | AC 4.1–4.9 | Tasks 6, 10 |
| Req 5: Hex Search Integration | AC 5.1–5.8 | Task 9 |
| Req 6: Cursor Synchronisation | AC 6.1–6.8 | Task 5 |
| Req 7: Undo/Redo in Hex Mode | AC 7.1–7.6 | Tasks 6, 7 |
| Req 8: Modified Byte Indicators | AC 8.1–8.5 | Task 8 |
| Req 9: Scrolling and Viewport | AC 9.1–9.8 | Task 14 |
| Req 10: Binary vs Text Files | AC 10.1–10.6 | Task 15 |
| Req 11: Hex Dump Export | AC 11.1–11.7 | Task 11 |
| Req 12: Goto Offset Command | AC 12.1–12.6 | Task 12 |
| Req 13: Uppercase/Lowercase Config | AC 13.1–13.7 | Tasks 2, 16 |
| Req 14: FileForge Structured Files | AC 14.1–14.4 | Task 18 |
| Req 15: Session State | AC 15.1–15.5 | Task 17 |
| Req 16: Command Compatibility | AC 16.1–16.4 | Task 13 |
