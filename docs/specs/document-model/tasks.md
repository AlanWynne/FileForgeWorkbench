# Implementation Plan: Document Model (`ff-document-model`)

## Overview

This plan covers the complete implementation of the `ff-document-model` crate — the foundational text storage layer for FileForgeWorkbench. The document model provides gap-buffer text storage, efficient line indexing with O(log n) lookups, streaming file loading via the VFS, encoding-aware character navigation, viewport position management, document lifecycle with shared ownership, and a watcher notification system.

This is a **Wave 4 (Core Editor)** sub-project that depends on Wave 3 (`ff-vfs`) for all file access and will integrate with `ff-command` for mutation routing and `ff-undo-redo` for transaction recording.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-document-model/Cargo.toml` with dependencies (thiserror, tokio, arc-swap, proptest dev-dep) and dependency on `ff-vfs` and `ff-logging`
  - [x] 1.2 Create `crates/ff-document-model/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `gap_buffer.rs`, `text_buffer.rs`, `line_index.rs`, `sparse_line_index.rs`, `streaming.rs`, `document.rs`, `handle.rs`, `watcher.rs`, `encoding_nav.rs`, `viewport.rs`, `save_point.rs`, `line_end.rs`, `error.rs`, `types.rs`
  - [x] 1.4 Add `ff-document-model` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Core newtypes and shared types
  - [x] 2.1 Define `BytePosition(u64)` newtype with arithmetic ops and Display
  - [x] 2.2 Define `LineNumber(u64)` newtype with 0-based internal storage and 1-based display conversion
  - [x] 2.3 Define `CharacterExtracted { code_point: char, byte_width: u8 }` struct
  - [x] 2.4 Define `LineEndMode` enum (Default, Unicode) with associated utility methods
  - [x] 2.5 Define `LoadingState` enum (NotStarted, InProgress { bytes_loaded, estimated_total }, Complete, Failed { error })
  - [x] 2.6 Define `SplitView` struct with two-segment access (before-gap slice, after-gap slice)
  - [x] 2.7 Write unit tests for newtype arithmetic, display formatting, and conversions
  - Covers: Requirement 1 (AC 1.2), Requirement 3 (AC 3.7), Requirement 4 (AC 4.4), Requirement 5 (AC 5.1), Requirement 8 (AC 8.4)

- [x] 3. GapBuffer data structure
  - [x] 3.1 Implement `GapBuffer` struct with contiguous storage, gap_start, gap_end fields using u64 positions
  - [x] 3.2 Implement `allocate(capacity)` pre-allocation method
  - [x] 3.3 Implement gap movement to target position with `memmove`-style byte shifting
  - [x] 3.4 Implement `insert_at(position, bytes)` with automatic gap growth (configurable factor, default 2x)
  - [x] 3.5 Implement `delete_at(position, length)` by expanding gap over deleted range
  - [x] 3.6 Implement `char_at(position)` returning `Option<u8>` for safe byte access
  - [x] 3.7 Implement `get_range(position, length)` returning `Vec<u8>` without exposing gap internals
  - [x] 3.8 Implement `contiguous_view()` that compacts gap and returns `&[u8]`
  - [x] 3.9 Implement `split_view()` returning `SplitView` without gap movement
  - [x] 3.10 Implement `length()` method returning total content size (excluding gap)
  - [x] 3.11 Write unit tests for insertion, deletion, gap movement, range access, and growth
  - Covers: Requirement 1 (AC 1.1–1.10)

- [x] 4. LineIndex (balanced partitioning structure)
  - [x] 4.1 Implement `LineIndex` struct using a sorted Vec or B-tree of line-start byte positions
  - [x] 4.2 Implement `line_count()` returning total lines (minimum 1 for empty document)
  - [x] 4.3 Implement `line_start(line: LineNumber)` with O(log n) lookup returning BytePosition
  - [x] 4.4 Implement `line_end(line: LineNumber)` returning position before line-end sequence
  - [x] 4.5 Implement `line_from_position(position: BytePosition)` with O(log n) binary search
  - [x] 4.6 Implement `insert_line(line: LineNumber, position: BytePosition)` for adding new line records
  - [x] 4.7 Implement `remove_lines(start_line, count)` for removing line records on deletion
  - [x] 4.8 Implement `adjust_positions(from_line, delta: i64)` for shifting positions after edits
  - [x] 4.9 Implement out-of-range handling: `line_start` past last line returns document length
  - [x] 4.10 Write unit tests for lookups, insertions, removals, and boundary conditions
  - Covers: Requirement 3 (AC 3.1–3.6, 3.8)

- [x] 5. Character-count index (optional UTF-16/UTF-32 tracking)
  - [x] 5.1 Implement `CharCountIndex` struct with per-line UTF-16 and UTF-32 character counts
  - [x] 5.2 Implement reference-counted allocation (`allocate_char_count_index`, `release_char_count_index`)
  - [x] 5.3 Implement incremental maintenance on insert/delete operations
  - [x] 5.4 Implement deallocation when reference count drops to zero
  - [x] 5.5 Write unit tests for allocation lifecycle and incremental updates
  - Covers: Requirement 3 (AC 3.9, 3.10, 3.11)

- [x] 6. TextBuffer assembly (GapBuffer + LineIndex coordination)
  - [x] 6.1 Implement `TextBuffer` struct owning a `GapBuffer` and `LineIndex`
  - [x] 6.2 Implement `insert(position, text)` that detects line endings, updates LineIndex, and respects read-only state
  - [x] 6.3 Implement `delete(position, length)` that removes line records for deleted line endings and respects read-only state
  - [x] 6.4 Implement CRLF split handling: insertion between CR and LF creates new line boundary
  - [x] 6.5 Implement CRLF merge handling: deletion that causes CR to become adjacent to LF merges line records
  - [x] 6.6 Implement `set_read_only(bool)` and `is_read_only()` control methods
  - [x] 6.7 Implement read-only guard that returns error on mutation attempts
  - [x] 6.8 Write unit tests for insert/delete with line tracking, CRLF edge cases, and read-only enforcement
  - Covers: Requirement 2 (AC 2.1–2.8)

- [x] 7. Line end mode support
  - [x] 7.1 Implement line-end detection for Default mode (CR, LF, CRLF)
  - [x] 7.2 Implement line-end detection for Unicode mode (additionally LS, PS, NEL)
  - [x] 7.3 Implement `set_line_end_mode(mode)` triggering full LineIndex rebuild via rescan
  - [x] 7.4 Implement `line_end_mode()` query
  - [x] 7.5 Implement `contains_line_end(text)` utility method
  - [x] 7.6 Implement Unicode line-end overlap handling on insertion (multi-byte sequence break detection)
  - [x] 7.7 Write unit tests for mode switching, Unicode line endings, and overlap edge cases
  - Covers: Requirement 5 (AC 5.1–5.6)

- [x] 8. Encoding-aware character navigation
  - [x] 8.1 Implement `char_length_at(position)` returning byte length (2 for CRLF, 1–4 for UTF-8, 1 for invalid)
  - [x] 8.2 Implement `move_position_outside_char(position, direction)` for adjusting mid-character positions
  - [x] 8.3 Implement `next_position(position, direction)` advancing to next valid character boundary
  - [x] 8.4 Implement `character_at(position)` returning `CharacterExtracted`
  - [x] 8.5 Implement `character_before(position)` scanning backwards through UTF-8
  - [x] 8.6 Implement `relative_position(start, char_offset)` advancing by character count
  - [x] 8.7 Implement CRLF atomic navigation (never landing between CR and LF)
  - [x] 8.8 Implement invalid UTF-8 fallback (treat each invalid byte as one character)
  - [x] 8.9 Write unit tests for all navigation methods including multi-byte, CRLF, and invalid sequences
  - Covers: Requirement 8 (AC 8.1–8.8)

- [x] 9. Streaming file loading
  - [x] 9.1 Implement `StreamingFileReader` that reads from VFS `read_stream()` in configurable chunks (default 64 KB)
  - [x] 9.2 Implement progressive content availability — already-loaded portions readable while loading continues
  - [x] 9.3 Implement `loading_progress()` returning `LoadingState` enum with current state
  - [x] 9.4 Implement cancellation support — dropping the reader or explicit cancel stops the background task without leaks
  - [x] 9.5 Implement completion notification to all watchers when streaming finishes
  - [x] 9.6 Implement error-state transition with partial content preservation on VFS I/O failure
  - [x] 9.7 Implement empty-session initialization (no file path → empty buffer, single-line index)
  - [x] 9.8 Write unit tests using mock VFS provider for streaming, cancellation, and error paths
  - Covers: Requirement 4 (AC 4.1–4.9)

- [x] 10. SparseLineIndex (incremental background indexing)
  - [x] 10.1 Implement `SparseLineIndex` that records one checkpoint per N lines (default 1000)
  - [x] 10.2 Implement incremental building in a background task as chunks arrive from streaming reader
  - [x] 10.3 Implement partial usability — already-indexed lines queryable before full index is complete
  - [x] 10.4 Implement finalization into complete LineIndex when streaming load finishes
  - [x] 10.5 Write unit tests for checkpoint accuracy, partial queries, and finalization correctness
  - Covers: Requirement 3 (AC 3.8), Requirement 4 (AC 4.3, 4.5)

- [x] 11. Document struct and lifecycle
  - [x] 11.1 Implement `Document` struct wrapping `TextBuffer` with encoding awareness, watcher list, and lifecycle state
  - [x] 11.2 Implement `DocumentHandle` as `Arc<RwLock<Document>>` type alias
  - [x] 11.3 Implement `Document::new()` for empty documents and `Document::from_streaming(reader)` for file loading
  - [x] 11.4 Implement `Send + Sync` bounds verification (compile-time assertion)
  - [x] 11.5 Implement read-only mode that blocks all mutation via RwLock read guards
  - [x] 11.6 Implement `Drop` notification to registered watchers via `notify_deleted()`
  - [x] 11.7 Write unit tests for handle cloning, drop semantics, and thread-safety
  - Covers: Requirement 6 (AC 6.1–6.8)

- [x] 12. DocumentWatcher notification system
  - [x] 12.1 Define `DocumentWatcher` trait with callbacks: `notify_modify_attempt`, `notify_insert`, `notify_delete`, `notify_save_point`, `notify_deleted`, `notify_style_needed`
  - [x] 12.2 Implement `add_watcher(watcher)` returning `WatcherHandle` with duplicate detection
  - [x] 12.3 Implement `remove_watcher(handle)` for unregistration
  - [x] 12.4 Wire insert/delete operations to dispatch notifications to all watchers
  - [x] 12.5 Implement `notify_modify_attempt()` dispatch on read-only mutation attempt
  - [x] 12.6 Implement non-blocking notification dispatch (watchers must not block the notification path)
  - [x] 12.7 Write unit tests for watcher registration, deduplication, notification delivery, and removal
  - Covers: Requirement 7 (AC 7.1–7.7)

- [x] 13. Viewport position management
  - [x] 13.1 Implement `top_line` field (1-based) with getter
  - [x] 13.2 Implement `scroll_page_down(visible_count)` with clamping to last displayable page
  - [x] 13.3 Implement `scroll_page_up(visible_count)` with clamping to line 1
  - [x] 13.4 Implement `scroll_line_down(count)` and `scroll_line_up(count)` with boundary clamping
  - [x] 13.5 Implement `set_top_line(line)` with clamping to valid range [1, max_top_line]
  - [x] 13.6 Implement `max_top_line(visible_count)` computed as `max(1, line_count - visible_count + 1)`
  - [x] 13.7 Implement idempotent boundary behavior (repeated scroll at boundaries has no effect)
  - [x] 13.8 Write unit tests for all scroll operations including boundary clamping and idempotency
  - Covers: Requirement 9 (AC 9.1–9.8)

- [x] 14. Save point and modification state
  - [x] 14.1 Implement save-point marker tracking current undo position
  - [x] 14.2 Implement `set_save_point()` recording current state as saved
  - [x] 14.3 Implement `is_at_save_point()` comparing current undo position to marker
  - [x] 14.4 Implement watcher notification on save-point transitions (`notify_save_point(bool)`)
  - [x] 14.5 Implement automatic save-point setting after successful file load
  - [x] 14.6 Write unit tests for save-point state transitions and watcher notifications
  - Covers: Requirement 10 (AC 10.1–10.6)

- [x] 15. Error types and VFS integration
  - [x] 15.1 Define `DocumentModelError` enum with variants: LineOutOfRange, PositionOutOfRange, ReadOnly, LoadFailed, IoError, WatcherAlreadyRegistered
  - [x] 15.2 Implement `From<VfsError>` conversion for transparent VFS error propagation
  - [x] 15.3 Implement error message format following `[document-model] operation: description` standard
  - [x] 15.4 Ensure all VFS calls go through `ff-vfs` — no `std::fs` or `tokio::fs` usage
  - [x] 15.5 Write unit tests for error formatting and conversion
  - Covers: Cross-cutting Requirement 8 (error standards), Requirement 4 (AC 4.8)

- [x] 16. Command framework integration surface
  - [x] 16.1 Define `DocumentCommand` trait for mutation operations routable through the command framework
  - [x] 16.2 Implement `InsertCommand` and `DeleteCommand` structs wrapping TextBuffer primitives
  - [x] 16.3 Implement undo-record emission hook (trait method that downstream `ff-undo-redo` will consume)
  - [x] 16.4 Document integration pattern — document-model provides primitives, command framework routes them
  - [x] 16.5 Write unit tests for command struct construction and execution
  - Covers: Requirement 2 (AC 2.9)

- [x] 17. Property-based tests
  - [x] 17.1 Write PBT: gap buffer content invariant
  - [x] 17.2 Write PBT: line index consistency after edits
  - [x] 17.3 Write PBT: character navigation boundary safety
  - [x] 17.4 Write PBT: viewport scroll clamping
  - [x] 17.5 Write PBT: CRLF atomicity under random edits
  - [x] 17.6 Write PBT: streaming load content integrity
  - Covers: Requirements 1, 2, 3, 8, 9 (see Property-Based Test Definitions below)

- [x] 18. Integration tests
  - [x] 18.1 Write integration test: full document lifecycle (create → load → edit → save-point → drop)
  - [x] 18.2 Write integration test: streaming load with mock VFS provider and progressive reading
  - [x] 18.3 Write integration test: multi-view shared ownership via DocumentHandle
  - [x] 18.4 Write integration test: large document stress test (>100K lines, verify O(log n) lookups)
  - Covers: End-to-end validation across Requirements 1–10

---

## Property-Based Test Definitions

### Property 1: Gap Buffer Content Invariant

**Validates: Requirement 1.1, 1.5, 1.9**

- **Statement:** For any sequence of insert and delete operations at arbitrary positions, the content returned by `get_range(0, length())` SHALL always equal the expected content produced by applying the same operations to a naive String model.
- **Strategy:** Generate:
  - Initial content: arbitrary UTF-8 strings (0–10000 bytes)
  - Operation sequence: 10–200 operations of Insert(position, text) or Delete(position, length) with positions clamped to valid range
- **Invariant:** `gap_buffer.get_range(0, gap_buffer.length()) == reference_string.as_bytes()`

### Property 2: Line Index Consistency After Edits

**Validates: Requirement 3.1, 3.2, 3.3, 3.6**

- **Statement:** After any sequence of insert/delete operations, the LineIndex SHALL satisfy: (a) `line_count()` equals the number of line-end sequences in the content + 1, (b) `line_start(n)` points to the byte after the nth line ending, and (c) `line_from_position(line_start(n)) == n` for all valid n.
- **Strategy:** Generate:
  - Initial content: arbitrary bytes containing a mix of CR, LF, CRLF sequences (0–5000 bytes)
  - Operation sequence: 5–100 insert/delete operations with random line-ending content
- **Invariant:** Line count matches reference count; bidirectional lookup round-trips for all valid line numbers

### Property 3: Character Navigation Boundary Safety

**Validates: Requirement 8.1, 8.3, 8.7, 8.8**

- **Statement:** For any document content and any position within `[0, length()]`, calling `next_position(pos, Forward)` followed by `next_position(result, Backward)` SHALL return the original position, and no navigation call SHALL ever return a position inside a multi-byte UTF-8 sequence or between a CR and its following LF.
- **Strategy:** Generate:
  - Content: arbitrary byte sequences (0–2000 bytes) including valid UTF-8, invalid bytes, and CRLF pairs
  - Positions: random positions in [0, content.length()]
- **Invariant:** Round-trip navigation returns original position; result positions are always at character boundaries

### Property 4: Viewport Scroll Clamping

**Validates: Requirement 9.2, 9.3, 9.4, 9.5, 9.8**

- **Statement:** For any document with N lines and any viewport size V, all scroll operations SHALL produce a `top_line` value in the range `[1, max(1, N - V + 1)]`, and repeating a boundary-clamped scroll SHALL not change `top_line`.
- **Strategy:** Generate:
  - Line count: [1, 100000]
  - Viewport size: [1, 1000]
  - Operation sequence: 20–100 random scroll operations (page_up, page_down, line_up(n), line_down(n), set_top_line(n))
- **Invariant:** `1 <= top_line <= max_top_line(visible_count)` after every operation; `scroll(x); scroll(x)` at boundary is idempotent

### Property 5: CRLF Atomicity Under Random Edits

**Validates: Requirement 2.5, 2.6, Requirement 8.7**

- **Statement:** After any sequence of edits, no line boundary SHALL exist between a CR byte and an immediately following LF byte — all adjacent CR+LF pairs SHALL be treated as a single CRLF line ending with exactly one line record.
- **Strategy:** Generate:
  - Initial content: byte sequences with CRLF pairs, lone CR, lone LF (0–3000 bytes)
  - Operation sequence: 10–100 random insert/delete operations, some inserting CR or LF adjacent to existing endings
- **Invariant:** For every line boundary at position P, it is NOT the case that `content[P-1] == CR && content[P] == LF` with separate line records for each

### Property 6: Streaming Load Content Integrity

**Validates: Requirement 4.1, 4.2, 4.5**

- **Statement:** For any file content delivered in arbitrary chunk sizes, the final document content after streaming load completes SHALL be byte-for-byte identical to the original content, and the LineIndex SHALL be consistent with a full single-pass index of the same content.
- **Strategy:** Generate:
  - File content: arbitrary bytes (0–50000 bytes) with mixed line endings
  - Chunk sizes: random partition of content into 1–100 chunks of varying size [1, 10000]
- **Invariant:** `document.get_range(0, document.length()) == original_content` AND `document.line_count() == reference_line_count`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Error", "tasks": ["2", "15"], "dependsOn": [0] },
    { "id": 2, "label": "Data Structures", "tasks": ["3", "4"], "dependsOn": [1] },
    { "id": 3, "label": "Buffer Assembly", "tasks": ["5", "6", "7"], "dependsOn": [2] },
    { "id": 4, "label": "Navigation and Viewport", "tasks": ["8", "13"], "dependsOn": [3] },
    { "id": 5, "label": "Streaming and Sparse Index", "tasks": ["9", "10"], "dependsOn": [3] },
    { "id": 6, "label": "Document Lifecycle", "tasks": ["11", "12", "14"], "dependsOn": [4, 5] },
    { "id": 7, "label": "Command Integration", "tasks": ["16"], "dependsOn": [6] },
    { "id": 8, "label": "Validation and PBT", "tasks": ["17", "18"], "dependsOn": [7] }
  ]
}
```

---

## Notes

- This is a Wave 4 (Core Editor) crate depending on `ff-vfs` (Wave 3) for all file access
- The undo/redo integration is specified in `undo-redo-transactions` — this crate defines the hook interface but does not implement transaction logic
- The `encoding-and-characters` crate (Wave 8) handles encoding detection and conversion; this crate provides UTF-8 character navigation only
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- All async operations use Tokio, compatible with the runtime managed by `ff-core`
- The `DocumentHandle` (`Arc<RwLock<Document>>`) enables multiple views to share a document — this is critical for split-view and background processing scenarios
- The SparseLineIndex enables progressive display of large files before full indexing completes — this is the key UX differentiator for large-file support
- GapBuffer uses `u64` positions throughout to support documents exceeding 2 GB

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Gap-Buffer Text Storage | AC 1.1–1.10 | Tasks 3, 2 |
| Req 2: Text Insertion and Deletion | AC 2.1–2.9 | Tasks 6, 7, 16, 17 |
| Req 3: Line Index and Position Tracking | AC 3.1–3.11 | Tasks 4, 5, 10 |
| Req 4: Streaming File Loading | AC 4.1–4.9 | Tasks 9, 10, 15, 17 |
| Req 5: Line End Type Support | AC 5.1–5.6 | Task 7 |
| Req 6: Document Lifecycle and Shared Ownership | AC 6.1–6.8 | Task 11 |
| Req 7: Document Watcher and Notification System | AC 7.1–7.7 | Task 12 |
| Req 8: Character and Encoding Navigation | AC 8.1–8.8 | Tasks 8, 17 |
| Req 9: Viewport Position Management | AC 9.1–9.8 | Tasks 13, 17 |
| Req 10: Save Point and Modification State | AC 10.1–10.6 | Task 14 |
