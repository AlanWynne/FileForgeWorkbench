# Implementation Plan: Clipboard Operations (`ff-clipboard`)

## Overview

This plan covers the complete implementation of the `ff-clipboard` crate — the clipboard subsystem for FileForgeWorkbench. It unifies system clipboard access, standard keyboard shortcuts (Ctrl+C/X/V), context menu integration, COPY primary command modes (clipboard-paste, file-insert, shell-capture routing), rectangular clipboard handling, multi-caret clipboard distribution, line-copy mode, clipboard history ring, and undoable clipboard transactions.

This is a **Wave 9 (Desktop Integration)** sub-project. It depends on `ff-edit-operations` for selection model and edit semantics, `ff-document-model` for buffer access, `ff-command` for command registration, `ff-undo-redo` for transaction recording, `ff-vfs` for file-insert mode, and `ff-config` for clipboard configuration keys.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-clipboard/Cargo.toml` with dependencies (ff-document-model, ff-edit-operations, ff-command, ff-undo-redo, ff-vfs, ff-config, ff-logging, thiserror, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-clipboard/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `engine.rs`, `provider.rs`, `entry.rs`, `copy.rs`, `cut.rs`, `paste.rs`, `line_copy.rs`, `rectangular.rs`, `multi_caret.rs`, `copy_command.rs`, `file_insert.rs`, `shell_capture.rs`, `history.rs`, `context_menu.rs`, `config.rs`, `commands.rs`, `error.rs`
  - [x] 1.4 Add `ff-clipboard` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Clipboard provider trait and entry types
  - [x] 2.1 Define `ClipboardMode` enum with variants: `Stream`, `Line`, `Rectangular`
  - [x] 2.2 Define `ClipboardEntry` struct with fields: `text: String`, `mode: ClipboardMode`, `segments: Option<Vec<String>>` (per-line segments for rectangular/multi-caret)
  - [x] 2.3 Define `ClipboardProvider` trait with methods: `read_text() -> Result<Option<String>>`, `write_text(text: &str) -> Result<()>`, `is_available() -> bool`
  - [x] 2.4 Define `ClipboardMetadata` struct storing `mode`, `segments`, `source_instance_id` for distinguishing internal vs external clipboard writes
  - [x] 2.5 Implement `InMemoryClipboardProvider` for testing (stores text in memory, always available)
  - [x] 2.6 Implement `ClipboardEntry::from_text(text, mode)` constructor and `ClipboardEntry::with_segments(text, mode, segments)` constructor
  - [x] 2.7 Write unit tests for entry construction, mode defaults, segment storage
  - Covers: Requirement 1 (AC 1.1, 1.4, 1.5, 1.7)

- [x] 3. Clipboard engine — read/write abstraction
  - [x] 3.1 Define `ClipboardEngine` struct holding `Box<dyn ClipboardProvider>`, internal `ClipboardMetadata` cache, and `access_timeout_ms` config
  - [x] 3.2 Implement `write(entry: &ClipboardEntry) -> Result<(), ClipboardError>` that writes text to provider and caches metadata internally
  - [x] 3.3 Implement `read() -> Result<ClipboardEntry, ClipboardError>` that reads text from provider and attaches cached metadata (or defaults to Stream mode for external content)
  - [x] 3.4 Implement external-vs-internal detection: compare provider text against last-written text to determine if clipboard was modified externally
  - [x] 3.5 Implement timeout wrapper: fail with `ClipboardError::Timeout` if provider read/write exceeds `access_timeout_ms`
  - [x] 3.6 Implement `is_available() -> bool` delegating to provider
  - [x] 3.7 Ensure no panic paths — all failures return descriptive `ClipboardError`
  - [x] 3.8 Write unit tests for read/write cycle, external detection, timeout, unavailability
  - Covers: Requirement 1 (AC 1.2, 1.3, 1.5, 1.6, 1.7)

- [x] 4. Error types and clipboard unavailability handling
  - [x] 4.1 Define `ClipboardError` enum with variants: `Unavailable { reason: String }`, `Timeout { operation: String, timeout_ms: u64 }`, `NonTextContent`, `IoError(std::io::Error)`, `Empty`, `FileNotFound { path: String }`, `FileAccessDenied { path: String }`, `BinaryFile { path: String }`, `InvalidPath { detail: String }`, `NoTarget`, `ConflictingCommands { detail: String }`
  - [x] 4.2 Implement `Display` for all variants following `[clipboard] operation: description` format
  - [x] 4.3 Implement `From<std::io::Error>` conversion
  - [x] 4.4 Write unit tests for error formatting and conversion
  - Covers: Requirement 6 (AC 6.1–6.5), Requirement 10 (AC 10.1–10.4)

- [x] 5. Copy operation (Ctrl+C)
  - [x] 5.1 Implement `copy_stream(doc, selection_range) -> Result<ClipboardEntry, ClipboardError>` copying selected text with mode Stream
  - [x] 5.2 Implement `copy_rectangular(doc, rectangular_selection) -> Result<ClipboardEntry, ClipboardError>` copying each line segment independently with mode Rectangular
  - [x] 5.3 Implement `copy_multi_caret(doc, selections) -> Result<ClipboardEntry, ClipboardError>` copying each caret's selection as separate segment
  - [x] 5.4 Implement `copy_line(doc, line_number) -> Result<ClipboardEntry, ClipboardError>` copying entire line (including line ending) with mode Line when no selection exists
  - [x] 5.5 Implement `execute_copy(engine, doc, container) -> Result<(), ClipboardError>` dispatching to appropriate copy variant based on selection state
  - [x] 5.6 Ensure copy never modifies document content or selection state
  - [x] 5.7 Write unit tests for stream copy, rectangular copy, multi-caret copy, line-copy-no-selection, non-modification guarantee
  - Covers: Requirement 2 (AC 2.1–2.6), Requirement 14 (AC 14.1)

- [x] 6. Cut operation (Ctrl+X)
  - [x] 6.1 Implement `cut_stream(doc, selection_range) -> Result<(ClipboardEntry, UndoRecord), ClipboardError>` copying then deleting selection
  - [x] 6.2 Implement `cut_rectangular(doc, rectangular_selection) -> Result<(ClipboardEntry, UndoRecord), ClipboardError>` copying then deleting column block
  - [x] 6.3 Implement `cut_multi_caret(doc, selections) -> Result<(ClipboardEntry, UndoRecord), ClipboardError>` copying each segment then deleting all as single UndoRecord
  - [x] 6.4 Implement `cut_line(doc, line_number) -> Result<(ClipboardEntry, UndoRecord), ClipboardError>` cutting entire line with mode Line when no selection exists
  - [x] 6.5 Implement `execute_cut(engine, doc, container) -> Result<UndoRecord, ClipboardError>` dispatching to appropriate cut variant
  - [x] 6.6 Ensure caret is placed at position where deleted text began after cut
  - [x] 6.7 Ensure clipboard write failure does not delete document text
  - [x] 6.8 Write unit tests for stream cut, rectangular cut, multi-caret cut, line-cut, caret placement, failure safety
  - Covers: Requirement 3 (AC 3.1–3.6), Requirement 14 (AC 14.4)

- [x] 7. Paste operation — stream and line modes (Ctrl+V)
  - [x] 7.1 Implement `paste_stream(doc, caret_position, text) -> Result<UndoRecord, ClipboardError>` inserting text inline at caret, replacing active selection if present
  - [x] 7.2 Implement `paste_line(doc, caret_line, text) -> Result<UndoRecord, ClipboardError>` inserting clipboard content as new lines above caret line without splitting current line
  - [x] 7.3 Implement line-ending splitting logic: split clipboard text on LF, CRLF, or CR into Logical_Lines
  - [x] 7.4 Implement trailing-line-ending handling: do not produce empty line for trailing terminator
  - [x] 7.5 Implement mixed-line-ending normalisation: normalise inserted lines to document's configured line-ending style
  - [x] 7.6 Ensure whitespace is preserved exactly as received (no trimming)
  - [x] 7.7 Place caret at end of inserted content with no active selection after paste
  - [x] 7.8 Record operation as single UndoRecord
  - [x] 7.9 Write unit tests for stream paste, line paste, line splitting, trailing terminator, whitespace preservation, caret placement
  - Covers: Requirement 4 (AC 4.1, 4.2, 4.6–4.9), Requirement 16 (AC 16.1–16.5), Requirement 18 (AC 18.1, 18.2)

- [x] 8. Paste operation — rectangular mode
  - [x] 8.1 Implement `paste_rectangular(doc, caret_position, segments) -> Result<UndoRecord, ClipboardError>` inserting each segment on successive lines at caret column
  - [x] 8.2 Implement rightward-push logic: existing text on each line shifts right by segment width
  - [x] 8.3 Implement short-line padding: pad with spaces up to caret column when caret is beyond line end
  - [x] 8.4 Implement new-line creation: create new lines when segments exceed remaining lines below caret
  - [x] 8.5 Implement rectangular-replace: when pasting with active rectangular selection, replace selected region adjusting for width differences
  - [x] 8.6 Record operation as single UndoRecord
  - [x] 8.7 Write unit tests for column paste, rightward push, short-line padding, new-line creation, rectangular replace
  - Covers: Requirement 4 (AC 4.3), Requirement 12 (AC 12.1–12.6)

- [x] 9. Paste operation — multi-caret distribution
  - [x] 9.1 Implement `paste_multi_caret_matched(doc, carets, segments) -> Result<UndoRecord, ClipboardError>` distributing segment[i] to caret[i] when counts match
  - [x] 9.2 Implement `paste_multi_caret_broadcast(doc, carets, full_text) -> Result<UndoRecord, ClipboardError>` pasting full content at each caret when counts mismatch
  - [x] 9.3 Implement reverse-document-order processing to prevent earlier insertions from invalidating later caret positions
  - [x] 9.4 Wrap all individual insertions in single UndoRecord for atomic undo
  - [x] 9.5 Implement `execute_paste(engine, doc, container) -> Result<UndoRecord, ClipboardError>` dispatching to stream/line/rectangular/multi-caret based on mode and caret count
  - [x] 9.6 Write unit tests for matched distribution, broadcast, reverse-order correctness, atomic undo
  - Covers: Requirement 4 (AC 4.4, 4.5), Requirement 13 (AC 13.1–13.5)

- [x] 10. COPY command — disambiguation and routing
  - [x] 10.1 Define `CopyCommandMode` enum with variants: `InDocument`, `ClipboardPaste`, `FileInsert { path: String }`, `ShellCapture`
  - [x] 10.2 Implement `resolve_copy_mode(args, pending_sources, target) -> Result<CopyCommandMode, ClipboardError>` disambiguation logic
  - [x] 10.3 Implement rule: pending C/CC + A/B → InDocument (route to line-commands)
  - [x] 10.4 Implement rule: no pending C/CC + no args + A/B → ClipboardPaste
  - [x] 10.5 Implement rule: no pending C/CC + path arg + A/B → FileInsert
  - [x] 10.6 Implement rule: pending C/CC + path arg → error (conflicting commands)
  - [x] 10.7 Implement rule: no pending C/CC + no A/B + no args → error (target required)
  - [x] 10.8 Implement rule: pending C/CC + no A/B → incomplete (retain pending, request target)
  - [x] 10.9 Implement file-insert precedence over clipboard-paste when path argument is present
  - [x] 10.10 Write unit tests for all disambiguation paths and error conditions
  - Covers: Requirement 8 (AC 8.1–8.8)

- [x] 11. COPY command — clipboard-paste mode
  - [x] 11.1 Implement `execute_clipboard_paste(engine, doc, target_line, target_type) -> Result<UndoRecord, ClipboardError>` reading clipboard and inserting at target
  - [x] 11.2 Implement A-target insertion: insert clipboard lines immediately after target line
  - [x] 11.3 Implement B-target insertion: insert clipboard lines immediately before target line
  - [x] 11.4 Implement line-splitting using same rules as Ctrl+V paste (LF/CRLF/CR, trailing terminator handling)
  - [x] 11.5 Clear resolved A/B target line command from prefix area on success
  - [x] 11.6 Display status message with number of lines inserted on success
  - [x] 11.7 Handle empty clipboard: do not modify document, display error, retain A/B target
  - [x] 11.8 Record operation as single UndoRecord
  - [x] 11.9 Write unit tests for A-insert, B-insert, line splitting, empty clipboard, target clearing, undo
  - Covers: Requirement 7 (AC 7.1–7.8)

- [x] 12. COPY command — file-insert mode
  - [x] 12.1 Implement `execute_file_insert(vfs, doc, path, target_line, target_type) -> Result<UndoRecord, ClipboardError>` reading file via VFS and inserting at target
  - [x] 12.2 Implement path resolution: relative paths resolved relative to current document's directory; absolute paths used as-is
  - [x] 12.3 Implement quoted-path parsing: strip surrounding double quotes from paths containing spaces
  - [x] 12.4 Implement A-target insertion: insert file lines immediately after target line
  - [x] 12.5 Implement B-target insertion: insert file lines immediately before target line
  - [x] 12.6 Implement file content line-splitting using same rules as clipboard paste
  - [x] 12.7 Preserve exact whitespace content of each line from file (no trimming)
  - [x] 12.8 Clear resolved A/B target line command from prefix area on success
  - [x] 12.9 Display status message with line count and resolved file path on success
  - [x] 12.10 Record operation as single UndoRecord
  - [x] 12.11 Write unit tests for relative path, absolute path, quoted path, A-insert, B-insert, line splitting, whitespace preservation
  - Covers: Requirement 9 (AC 9.1–9.11)

- [x] 13. File-insert error handling
  - [x] 13.1 Implement file-not-found detection: return `ClipboardError::FileNotFound` with resolved path when file does not exist
  - [x] 13.2 Implement permission/IO error handling: return `ClipboardError::FileAccessDenied` when file cannot be read
  - [x] 13.3 Implement binary-file detection: check for null bytes or non-UTF-8 content, return `ClipboardError::BinaryFile`
  - [x] 13.4 Ensure document is never modified on any file-insert error
  - [x] 13.5 Retain A/B target line command in prefix area on error so user can correct path and retry
  - [x] 13.6 Write unit tests for file-not-found, permission error, binary detection, non-modification guarantee, target retention
  - Covers: Requirement 10 (AC 10.1–10.4)

- [x] 14. COPY command — shell-capture mode routing
  - [x] 14.1 Define `ShellCaptureResult` struct with `stdout_lines: Vec<String>`, `line_count: usize`
  - [x] 14.2 Implement `execute_shell_capture_insert(doc, capture_result, target_line, target_type) -> Result<UndoRecord, ClipboardError>` inserting captured output at target
  - [x] 14.3 Implement A-target insertion: insert captured lines immediately after target line
  - [x] 14.4 Implement B-target insertion: insert captured lines immediately before target line
  - [x] 14.5 Handle empty output: do not modify document, display message indicating no output
  - [x] 14.6 Clear resolved A/B target from prefix area on success and display line-count status
  - [x] 14.7 Record operation as single UndoRecord
  - [x] 14.8 Write unit tests for A-insert, B-insert, empty output, target clearing, undo
  - Covers: Requirement 11 (AC 11.1–11.7)
  - Note: Shell execution mechanics are defined in `shell-command`; this task defines only the document-insertion contract

- [x] 15. Line-copy mode
  - [x] 15.1 Implement line-copy detection in copy path: when no selection exists, copy entire current line with mode Line
  - [x] 15.2 Implement line-cut detection in cut path: when no selection exists, cut entire current line with mode Line
  - [x] 15.3 Implement line-paste behaviour: when mode is Line, insert as new line(s) above caret line without splitting
  - [x] 15.4 Implement multi-line line-paste: insert all lines as a block above caret line
  - [x] 15.5 Record line paste as single UndoRecord
  - [x] 15.6 Respect `clipboard.line_copy_when_no_selection` config: when false, Ctrl+C with no selection does nothing
  - [x] 15.7 Write unit tests for line-copy trigger, line-cut trigger, line-paste insertion, multi-line block, config disable
  - Covers: Requirement 14 (AC 14.1–14.5)

- [x] 16. Clipboard history ring
  - [x] 16.1 Define `ClipboardHistoryRing` struct holding a `VecDeque<ClipboardEntry>` with configurable max capacity
  - [x] 16.2 Implement `push(entry: ClipboardEntry)` adding to front of ring, evicting oldest if at capacity
  - [x] 16.3 Implement `current() -> Option<&ClipboardEntry>` returning most recent entry
  - [x] 16.4 Implement `cycle_back() -> Option<&ClipboardEntry>` moving to previous entry in ring
  - [x] 16.5 Implement `cycle_forward() -> Option<&ClipboardEntry>` moving to next entry in ring
  - [x] 16.6 Implement `entries() -> impl Iterator<Item = &ClipboardEntry>` yielding all entries newest-first
  - [x] 16.7 Implement `clear()` emptying the ring
  - [x] 16.8 Integrate with ClipboardEngine: every write operation pushes to history ring
  - [x] 16.9 Write unit tests for push, eviction, cycle navigation, clear, capacity enforcement
  - Covers: Internal quality feature supporting clipboard workflow efficiency

- [x] 17. Context menu clipboard operations
  - [x] 17.1 Define `ClipboardContextMenuState` struct with fields: `can_cut: bool`, `can_copy: bool`, `can_paste: bool`
  - [x] 17.2 Implement `compute_menu_state(container, engine) -> ClipboardContextMenuState` determining enabled/disabled items
  - [x] 17.3 Implement rule: Cut and Copy disabled when no selection is active
  - [x] 17.4 Implement rule: Paste disabled when system clipboard is empty or contains no text
  - [x] 17.5 Implement context menu Cut → delegate to `execute_cut`
  - [x] 17.6 Implement context menu Copy → delegate to `execute_copy`
  - [x] 17.7 Implement context menu Paste → delegate to `execute_paste`
  - [x] 17.8 Write unit tests for menu state computation, delegation correctness, disabled states
  - Covers: Requirement 5 (AC 5.1–5.7)

- [x] 18. Selection interaction with clipboard operations
  - [x] 18.1 Implement paste-with-selection: delete active selection first, then insert clipboard content at resulting caret, as single UndoRecord
  - [x] 18.2 Implement post-paste caret placement: caret at end of inserted content, no active selection
  - [x] 18.3 Implement post-copy preservation: copy does not modify or clear current selection
  - [x] 18.4 Implement post-cut caret placement: caret at position where deleted text began, no active selection
  - [x] 18.5 Write unit tests for paste-replacing-selection, caret after paste, selection after copy, caret after cut
  - Covers: Requirement 18 (AC 18.1–18.5)

- [x] 19. Undoable clipboard operations
  - [x] 19.1 Ensure all paste operations (Ctrl+V, COPY clipboard-paste, file-insert, shell-capture) produce a single UndoRecord
  - [x] 19.2 Implement paste-undo: remove all inserted content, restore document to pre-paste state
  - [x] 19.3 Ensure all cut operations produce a single UndoRecord
  - [x] 19.4 Implement cut-undo: restore deleted text at original position and restore pre-cut selection
  - [x] 19.5 Implement Ctrl+Z mapping to UNDO command dispatch
  - [x] 19.6 Implement Ctrl+Y / Ctrl+Shift+Z mapping to REDO command dispatch
  - [x] 19.7 Write unit tests for paste undo/redo round-trip, cut undo/redo round-trip, selection restoration
  - Covers: Requirement 15 (AC 15.1–15.6)

- [x] 20. Configuration integration
  - [x] 20.1 Implement reading `clipboard.line_copy_when_no_selection` config key (boolean, default true)
  - [x] 20.2 Implement reading `clipboard.rectangular_paste_adds_lines` config key (boolean, default true); when false, discard excess rectangular segments
  - [x] 20.3 Implement reading `clipboard.access_timeout_ms` config key (positive integer, default 500)
  - [x] 20.4 Implement invalid-value fallback: log warning and use documented defaults
  - [x] 20.5 Wire config values into ClipboardEngine at construction and support runtime reload
  - [x] 20.6 Write unit tests for each config key effect, invalid value fallback, default behaviour
  - Covers: Requirement 19 (AC 19.1–19.4)

- [x] 21. Command registration and shortcut bindings
  - [x] 21.1 Register `"clipboard.copy"` command with default shortcut Ctrl+C, metadata, and handler delegating to `execute_copy`
  - [x] 21.2 Register `"clipboard.cut"` command with default shortcut Ctrl+X, metadata, and handler delegating to `execute_cut`
  - [x] 21.3 Register `"clipboard.paste"` command with default shortcut Ctrl+V, metadata, and handler delegating to `execute_paste`
  - [x] 21.4 Register `"clipboard.copy-command"` (COPY primary command) with handler delegating to disambiguation and routing logic
  - [x] 21.5 Ensure shortcut bindings are overridable via user configuration in Shortcut_Registry
  - [x] 21.6 Ensure commands execute identically when invoked via scripting bridge (Lua macro) producing same UndoRecord
  - [x] 21.7 Ensure commands are logged in Command_History for RETRIEVE access
  - [x] 21.8 Write integration tests verifying command dispatch triggers correct clipboard operations
  - Covers: Requirement 17 (AC 17.1–17.5)

- [x] 22. Clipboard unavailability and error UX
  - [x] 22.1 Implement status-bar message display when Ctrl+V is pressed and clipboard is empty or unavailable
  - [x] 22.2 Implement error message display when clipboard access fails due to platform/permission error
  - [x] 22.3 Implement error message display when clipboard contains non-text content
  - [x] 22.4 Ensure copy/cut failure does not lose selected text or modify document
  - [x] 22.5 Ensure clipboard failure retains pending A/B line commands for retry
  - [x] 22.6 Write unit tests for each error condition's user-facing message and non-modification guarantee
  - Covers: Requirement 6 (AC 6.1–6.5)

- [x] 23. Property-based tests — clipboard engine invariants
  - [x] 23.1 Write property test: any text written to ClipboardEngine and immediately read back produces identical text content for arbitrary UTF-8 strings
    - **Validates: Requirements 1.2, 1.3**
  - [x] 23.2 Write property test: clipboard mode is preserved through write/read cycle for any ClipboardMode variant and arbitrary entry content
    - **Validates: Requirements 1.4, 1.5**
  - [x] 23.3 Write property test: external clipboard modification (simulated by direct provider write) always results in Stream mode on read regardless of prior internal mode
    - **Validates: Requirement 1.5**
  - [x] 23.4 Write property test: ClipboardEngine never panics for any sequence of read/write/availability-check calls with any provider state
    - **Validates: Requirement 1.6**

- [x] 24. Property-based tests — line splitting and paste invariants
  - [x] 24.1 Write property test: splitting text on line endings and rejoining with a single separator produces equivalent logical content — no content is lost or added for arbitrary multi-line text
    - **Validates: Requirements 16.1, 16.4**
  - [x] 24.2 Write property test: text ending with a trailing line ending produces exactly N lines (not N+1) where N is the number of line-ending separators for arbitrary text with trailing terminators
    - **Validates: Requirement 16.3**
  - [x] 24.3 Write property test: paste followed by undo restores document to exact pre-paste state for any clipboard content and any valid caret position
    - **Validates: Requirements 15.1, 15.2**
  - [x] 24.4 Write property test: line-mode paste inserts exactly the number of logical lines derived from clipboard content (no extra, no fewer) for any multi-line text
    - **Validates: Requirements 4.2, 14.2, 14.3**

- [x] 25. Property-based tests — multi-caret and rectangular invariants
  - [x] 25.1 Write property test: multi-caret copy with N carets produces exactly N segments in ClipboardEntry for any N >= 1 and arbitrary selection content
    - **Validates: Requirement 13.1**
  - [x] 25.2 Write property test: multi-caret paste with matching segment count distributes exactly one segment per caret and total inserted text equals total segment text
    - **Validates: Requirements 13.2, 13.3**
  - [x] 25.3 Write property test: rectangular paste produces exactly `segments.len()` affected lines and each affected line grows by exactly the segment width at the insertion column
    - **Validates: Requirements 12.2, 12.4**
  - [x] 25.4 Write property test: multi-caret paste in reverse document order produces identical result to a naive forward-order simulation (correctness of reverse processing)
    - **Validates: Requirement 13.5**

- [x] 26. Property-based tests — COPY command disambiguation invariants
  - [x] 26.1 Write property test: resolve_copy_mode with pending C/CC and any target always returns InDocument regardless of arguments
    - **Validates: Requirements 8.1, 8.3**
  - [x] 26.2 Write property test: resolve_copy_mode with no pending C/CC, no args, and valid A/B target always returns ClipboardPaste
    - **Validates: Requirement 8.4**
  - [x] 26.3 Write property test: resolve_copy_mode with a path argument always takes precedence over clipboard-paste when no pending C/CC exists
    - **Validates: Requirements 8.5, 8.6**
  - [x] 26.4 Write property test: resolve_copy_mode never returns Ok for the combination pending C/CC + path argument (always error)
    - **Validates: Requirement 8.7**

- [x] 27. Property-based tests — clipboard history ring invariants
  - [x] 27.1 Write property test: history ring never exceeds configured max capacity after any number of push operations
    - **Validates: Internal invariant (history ring bounded size)**
  - [x] 27.2 Write property test: `current()` always returns the most recently pushed entry when ring is non-empty
    - **Validates: Internal invariant (LIFO ordering)**
  - [x] 27.3 Write property test: cycling through entire ring and back returns to original current entry (ring wraps correctly)
    - **Validates: Internal invariant (ring cycle correctness)**

- [x] 28. Integration tests — end-to-end clipboard workflows
  - [x] 28.1 Write integration test: copy stream text → paste at different position → verify document state
  - [x] 28.2 Write integration test: cut selection → paste elsewhere → undo both → verify original document restored
  - [x] 28.3 Write integration test: line-copy (no selection) → paste → verify new line inserted above caret line
  - [x] 28.4 Write integration test: rectangular copy → paste as column block → verify column alignment preserved
  - [x] 28.5 Write integration test: multi-caret copy (3 carets) → paste with 3 carets → verify segment distribution
  - [x] 28.6 Write integration test: multi-caret copy (3 carets) → paste with 2 carets → verify full-text broadcast
  - [x] 28.7 Write integration test: COPY command clipboard-paste mode (A target) → verify lines inserted after target
  - [x] 28.8 Write integration test: COPY command clipboard-paste mode (B target) → verify lines inserted before target
  - [x] 28.9 Write integration test: COPY command file-insert mode with relative path → verify file content inserted
  - [x] 28.10 Write integration test: COPY command file-insert mode with non-existent file → verify error and no modification
  - [x] 28.11 Write integration test: COPY command disambiguation — pending C/CC + A target → routes to line-commands (InDocument)
  - [x] 28.12 Write integration test: clipboard unavailable during paste → verify error message and no document change
  - [x] 28.13 Write integration test: shell-capture insert at A target → verify captured lines inserted correctly
  - [x] 28.14 Write integration test: config `line_copy_when_no_selection = false` → Ctrl+C with no selection does nothing
  - [x] 28.15 Write integration test: config `rectangular_paste_adds_lines = false` → excess segments discarded silently
  - Covers: End-to-end validation of Requirements 1–19

---

## Notes

- The `ff-clipboard` crate has zero GUI dependencies — it operates on abstract document model types and produces UndoRecords for `ff-undo-redo`.
- The `ClipboardProvider` trait enables platform-specific implementations (Win32, X11/Wayland, macOS) to be injected at application startup while keeping the crate testable with `InMemoryClipboardProvider`.
- Shell-capture mode (Task 14) defines only the document-insertion contract; actual shell execution mechanics are owned by the `shell-command` sub-project.
- File-insert mode (Tasks 12–13) reads files through the VFS abstraction layer; the `ff-vfs` crate provides the `VfsProvider` trait.
- The clipboard history ring (Task 16) is an internal quality feature that supports paste-cycling workflows; it is not exposed via the COPY primary command.
- COPY command disambiguation (Task 10) is the critical routing logic that determines which of four modes (in-document, clipboard-paste, file-insert, shell-capture) the COPY command operates in.
- Multi-caret reverse-order processing (Task 9.3) is essential for correctness: forward-order processing causes position drift as earlier insertions shift later caret positions.
- Property-based tests (Tasks 23–27) use the `proptest` crate and are configured for a minimum of 256 iterations.
- Configuration (Task 20) integrates with the `ff-config` crate's key-value system; defaults are used when keys are absent or invalid.
- Context menu state (Task 17) provides the data for UI rendering but does not depend on any GUI framework.

---

## Acceptance Criteria Coverage Map

| Task | Requirements Covered |
|------|---------------------|
| 1 | Structural scaffolding (all) |
| 2 | Req 1 (AC 1.1, 1.4, 1.5, 1.7) |
| 3 | Req 1 (AC 1.2, 1.3, 1.5, 1.6, 1.7) |
| 4 | Req 6 (AC 6.1–6.5), Req 10 (AC 10.1–10.4) |
| 5 | Req 2 (AC 2.1–2.6), Req 14 (AC 14.1) |
| 6 | Req 3 (AC 3.1–3.6), Req 14 (AC 14.4) |
| 7 | Req 4 (AC 4.1, 4.2, 4.6–4.9), Req 16 (AC 16.1–16.5), Req 18 (AC 18.1, 18.2) |
| 8 | Req 4 (AC 4.3), Req 12 (AC 12.1–12.6) |
| 9 | Req 4 (AC 4.4, 4.5), Req 13 (AC 13.1–13.5) |
| 10 | Req 8 (AC 8.1–8.8) |
| 11 | Req 7 (AC 7.1–7.8) |
| 12 | Req 9 (AC 9.1–9.11) |
| 13 | Req 10 (AC 10.1–10.4) |
| 14 | Req 11 (AC 11.1–11.7) |
| 15 | Req 14 (AC 14.1–14.5) |
| 16 | Internal (clipboard history ring) |
| 17 | Req 5 (AC 5.1–5.7) |
| 18 | Req 18 (AC 18.1–18.5) |
| 19 | Req 15 (AC 15.1–15.6) |
| 20 | Req 19 (AC 19.1–19.4) |
| 21 | Req 17 (AC 17.1–17.5) |
| 22 | Req 6 (AC 6.1–6.5) |
| 23 | PBT: Req 1 (clipboard engine invariants) |
| 24 | PBT: Req 4, 14, 15, 16 (line splitting and paste invariants) |
| 25 | PBT: Req 12, 13 (multi-caret and rectangular invariants) |
| 26 | PBT: Req 8 (COPY command disambiguation invariants) |
| 27 | PBT: Internal (clipboard history ring invariants) |
| 28 | Integration: Req 1–19 (end-to-end workflows) |

---

## Task Dependency Graph

```json
{
  "taskDependencies": {
    "1": [],
    "2": ["1"],
    "3": ["2"],
    "4": ["1"],
    "5": ["2", "3"],
    "6": ["2", "3", "5"],
    "7": ["3", "4"],
    "8": ["3", "4", "7"],
    "9": ["3", "7", "8"],
    "10": ["4"],
    "11": ["3", "7", "10"],
    "12": ["4", "7", "10"],
    "13": ["12"],
    "14": ["4", "7", "10"],
    "15": ["5", "6", "7"],
    "16": ["3"],
    "17": ["5", "6", "7"],
    "18": ["5", "6", "7"],
    "19": ["7", "8", "9", "11", "12", "14"],
    "20": ["3"],
    "21": ["5", "6", "7", "10", "11"],
    "22": ["3", "4"],
    "23": ["3", "16"],
    "24": ["7", "8", "9", "15", "19"],
    "25": ["8", "9"],
    "26": ["10", "11", "12"],
    "27": ["16"],
    "28": ["5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "17", "18", "19", "20", "21", "22"]
  },
  "externalDependencies": {
    "ff-document-model": "Provides Document, TextBuffer, LineIndex — all clipboard operations read/write through this API",
    "ff-edit-operations": "Provides SelectionContainer, SelectionRange, RectangularSelection — clipboard operations consume selection state",
    "ff-command": "Command registry, dispatch, metadata, Shortcut_Registry — clipboard commands are registered here",
    "ff-undo-redo": "TransactionStack, UndoRecord — all paste/cut operations produce undo records",
    "ff-vfs": "VfsProvider trait — file-insert mode reads files through this abstraction",
    "ff-config": "Configuration key-value store — clipboard config keys are read from here",
    "ff-logging": "Structured logging for error reporting, warnings, and diagnostics"
  },
  "waves": [
    {
      "id": 0,
      "label": "Foundation types and engine",
      "tasks": ["1", "2", "3", "4"],
      "description": "Crate scaffolding, provider trait, clipboard engine, error types"
    },
    {
      "id": 1,
      "label": "Core clipboard operations",
      "tasks": ["5", "6", "7", "8", "9"],
      "description": "Copy, cut, paste (stream, line, rectangular, multi-caret)",
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "COPY command modes",
      "tasks": ["10", "11", "12", "13", "14"],
      "description": "Disambiguation, clipboard-paste, file-insert, file errors, shell-capture routing",
      "dependsOn": [0, 1]
    },
    {
      "id": 3,
      "label": "Behaviour refinements",
      "tasks": ["15", "16", "17", "18"],
      "description": "Line-copy mode, history ring, context menu, selection interaction",
      "dependsOn": [1]
    },
    {
      "id": 4,
      "label": "Undo, config, and command registration",
      "tasks": ["19", "20", "21", "22"],
      "description": "Undoable operations, configuration, command framework integration, error UX",
      "dependsOn": [1, 2, 3]
    },
    {
      "id": 5,
      "label": "Property-based tests",
      "tasks": ["23", "24", "25", "26", "27"],
      "description": "Property tests validating invariants across engine, paste, multi-caret, disambiguation, and history",
      "dependsOn": [0, 1, 2, 3]
    },
    {
      "id": 6,
      "label": "Integration tests",
      "tasks": ["28"],
      "description": "End-to-end workflow validation covering all requirements",
      "dependsOn": [0, 1, 2, 3, 4, 5]
    }
  ]
}
```
