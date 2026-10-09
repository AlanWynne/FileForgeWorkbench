# Requirements Document

## Introduction

This feature specifies the **Document Model** for FileForgeWorkbench -- the `ff-document-model` crate. The document model is the foundational text storage layer that underpins the entire editing experience. It provides gap-buffer-based text storage, efficient line indexing, large-file streaming support, buffer lifecycle management, and encoding-aware character navigation.

The document model is **GUI-independent** -- it has no rendering or framework dependency. It operates behind the Virtual File System abstraction (FFW-ARCH-001) for all file access, integrates with the command framework for mutation operations, and supports multi-view sharing through reference-counted ownership.

This specification merges requirements from two primary sources:

- **FileForgeEditor MVP** (Requirements 1–2): File loading via streaming reader, sparse line indexing with background construction, progressive display, viewport scrolling with top_line pointer management
- **Scintilla Document/CellBuffer** (Requirements 1–3, 7, 8, 11): Gap-buffer text storage, insertion/deletion with line tracking, O(log n) line indexing via partitioning, configurable line-end recognition, reference-counted lifecycle, and encoding-aware character navigation

The design adapts Scintilla's C++ patterns to idiomatic Rust: traits replace virtual methods, `Arc` replaces raw reference counting, iterators replace pointer-based range access, and the message-passing API (WndProc) is excluded entirely.

**Source references:**
- **[FFE-MVP-1]** = FileForgeEditor mvp-implementation Requirement 1: Open a Real File
- **[FFE-MVP-2]** = FileForgeEditor mvp-implementation Requirement 2: Real Viewport Scrolling
- **[SCI-DOC-1]** = Scintilla document-cellbuffer Requirement 1: CellBuffer Text Storage
- **[SCI-DOC-2]** = Scintilla document-cellbuffer Requirement 2: CellBuffer Insertion and Deletion
- **[SCI-DOC-3]** = Scintilla document-cellbuffer Requirement 3: CellBuffer Line Tracking
- **[SCI-DOC-7]** = Scintilla document-cellbuffer Requirement 7: Line End Type Support
- **[SCI-DOC-8]** = Scintilla document-cellbuffer Requirement 8: Document Reference Counting and Lifecycle
- **[SCI-DOC-11]** = Scintilla document-cellbuffer Requirement 11: Character and Encoding Navigation
- **[WB]** = Workbench Platform Architecture Brief

## Glossary

- **GapBuffer**: A data structure that stores text with a movable gap, providing O(1) amortized insertion and deletion at the cursor position. Rust equivalent of Scintilla's SplitVector. [SCI-DOC-1] REVISED (CR-CH-058): under the windowed piece-table model the GapBuffer NO LONGER holds the whole file; it survives ONLY as the Append_Buffer (section Requirement 12) that stores the bytes of edited/inserted records. Original record bytes are not resident in the GapBuffer -- they are read on demand through the Immutable_Original_Index. See the FOUNDATION-DESIGN at `.agents/tasks/windowed-record-foundation/FOUNDATION-DESIGN.md` section 2.
- **TextBuffer**: The primary text storage struct that owns the gap buffer, manages line tracking, and coordinates with undo recording. Replaces Scintilla's CellBuffer. [SCI-DOC-1] REVISED (CR-CH-058): the TextBuffer is reframed as the coordinator of the three windowed layers (Immutable_Original_Index + Piece_List + Append_Buffer); the single-contiguous-buffer assumption no longer holds. See Requirement 12.
- **RecordFormat**: The framing of the editable unit (a RECORD) supplied by the owning Command Environment: `Delimited { terminator: Crlf | Lf | Cr | Mixed }` (native, generalising LineEndMode), `Fixed { lrecl }` (mainframe FB, position-terminated), or `Variable { max_lrecl, rdw }` (mainframe VB, length-prefixed). The editor edits records uniformly regardless of framing. [CR-CH-058]
- **Record**: The universal editable unit of a document. For a Delimited document a record is a delimiter-terminated line (byte-identical to today's "line"); for Fixed it is one fixed-length slot; for Variable it is one length-prefixed logical record. "Line" remains a synonym for a Delimited-document record. [CR-CH-058]
- **Immutable_Original_Index**: A lean flat index built on open, one entry per ORIGINAL record (file_offset u64 + byte_length u32 + flags u32, ~16 bytes), never mutated after the open scan; provides O(1) "where is original record K". The one large structure, and because it is immutable it is mmap/spill-friendly. [CR-CH-058]
- **Piece_List**: A small ordered list of pieces presenting the CURRENT document -- `Original { first_record, count }` (resolves via the Immutable_Original_Index) or `Edited { buf_range, count }` (bytes in the Append_Buffer) -- each carrying a running record-count. Every edit is a piece-list SPLICE; unedited spans stay one Original piece, so the list is small (one entry per contiguous unedited run plus one per edit). [CR-CH-058]
- **Append_Buffer**: The append-only byte store holding only inserted/edited record bytes (the surviving role of the GapBuffer). Grows with edits, not with file size. [CR-CH-058]
- **Window_Band**: The physically resident byte cache: a 3-page band (current page + one prefetched page above + one below), a page being the maximum records that fit a full screen at the SMALLEST zoom. Byte residency is decoupled from the logical index. [CR-CH-058]
- **Total_Records**: The whole-document record count from the index (the authority for scrollbar/navigation extents), DISTINCT from the resident window size. During a background delimited scan it may be an estimate (see Requirement 4). [CR-CH-058]
- **Document**: The high-level text model struct that wraps TextBuffer and adds encoding awareness, watcher notifications, lifecycle management, and the public API surface. Replaces Scintilla's Document class. [SCI-DOC-8]
- **DocumentHandle**: An `Arc<RwLock<Document>>` that enables shared ownership across multiple views and background threads. Rust equivalent of Scintilla's reference-counted Document pointer. [SCI-DOC-8]
- **LineIndex**: The partitioning structure that maps line numbers to byte positions, providing O(log n) lookups in both directions. Replaces Scintilla's LineVector/Partitioning. [SCI-DOC-3]
- **SparseLineIndex**: The incremental line index built in a background thread during large-file streaming, recording one checkpoint per N lines. [FFE-MVP-1] REVISED (CR-CH-058): repurposed as the RESIDENT checkpoint layer / seed for the above-budget sparse mode; `finalize` NO LONGER rescans the whole buffer into a dense fully-resident LineIndex (that assumed full residency -- findings.md Q2). See Requirement 4 and Requirement 12.
- **StreamingFileReader**: The async file reader that loads content from the VFS in chunks, enabling progressive display before the full file is indexed. [FFE-MVP-1] REVISED (CR-CH-058): reframed from progressive-APPEND-to-EOF (which left the whole file resident -- findings.md Q2) to a RANGED/windowed reader that reads records N..M on demand and builds the Immutable_Original_Index without full residency. See Requirement 4 and Requirement 12.
- **LineEndMode**: An enum specifying which line-end sequences are recognised: Default (CR, LF, CRLF) or Unicode (additionally LS, PS, NEL). [SCI-DOC-7]
- **BytePosition**: A newtype wrapper around `u64` representing a byte offset within the document buffer. Uses 64-bit to support large documents (>2 GB). [SCI-DOC-1]
- **LineNumber**: A newtype wrapper around `u64` representing a 0-based line number within the document. [SCI-DOC-3]
- **CharacterExtracted**: A struct containing a Unicode code point and its byte width, returned by character navigation methods. [SCI-DOC-11]
- **DocumentWatcher**: A trait that consumers implement to receive notifications about document modifications, save-point changes, and lifecycle events. [SCI-DOC-8]
- **TopLine**: The 1-based line number identifying the first line currently visible in a viewport. [FFE-MVP-2]
- **SplitView**: A two-segment view over the gap buffer that provides read access to the entire text content without compacting the gap. [SCI-DOC-1]
- **VFS**: Virtual File System -- the abstraction layer through which all file access flows (FFW-ARCH-001). [WB]

---

## Requirements

### Requirement 1: Gap-Buffer Text Storage

**User Story:** As a document model consumer, I want text stored in a gap buffer, so that insertions and deletions at the cursor position are O(1) amortized and the buffer can handle documents of any size including those exceeding 2 GB.

**Source:** [SCI-DOC-1], [WB]

#### Acceptance Criteria

1. THE TextBuffer SHALL store document text in a GapBuffer structure providing O(1) amortized insertion and deletion at the gap position, with O(n) gap movement when the edit position changes. [SCI-DOC-1]
2. THE GapBuffer SHALL use `u64` byte positions internally, supporting documents larger than 2 GB without overflow or truncation. [SCI-DOC-1]
3. WHEN `char_at(position)` is called with a valid BytePosition, THE TextBuffer SHALL return the byte value at that position from the buffer. [SCI-DOC-1]
4. IF `char_at(position)` is called with a position outside the valid range `[0, length())`, THEN THE TextBuffer SHALL return `None`. [SCI-DOC-1]
5. WHEN `get_range(position, length)` is called with a valid range, THE TextBuffer SHALL return a byte slice (or copied Vec<u8>) containing the requested content without requiring the caller to understand the gap structure. [SCI-DOC-1]
6. WHEN `contiguous_view()` is called, THE TextBuffer SHALL compact the gap and return a reference to a contiguous byte slice of all document text. [SCI-DOC-1]
7. WHEN `split_view()` is called, THE TextBuffer SHALL return a SplitView providing two-segment access to the text (before-gap and after-gap) without moving the gap, enabling efficient read-only iteration. [SCI-DOC-1]
8. WHEN `allocate(capacity)` is called, THE GapBuffer SHALL pre-allocate storage for at least `capacity` bytes, reducing reallocation during bulk loading. [SCI-DOC-1]
9. THE TextBuffer SHALL track its total content length (excluding the gap) and expose it via a `length()` method. [SCI-DOC-1]
10. THE GapBuffer SHALL grow by a configurable factor (default 2x) when the gap is exhausted, amortizing allocation cost over many insertions. [SCI-DOC-1]

---

### Requirement 2: Text Insertion and Deletion

**User Story:** As a document model consumer, I want text insertion and deletion operations that maintain line tracking integrity, detect line endings, and integrate with the undo system, so that all modifications keep the document in a consistent state and are reversible.

**Source:** [SCI-DOC-2], [SCI-DOC-7], [WB]

#### Acceptance Criteria

1. WHEN `insert(position, text)` is called on a non-read-only document, THE TextBuffer SHALL insert the text at the specified byte position, update the LineIndex for any line-end characters in the inserted text, and notify the undo system (if undo collection is enabled). [SCI-DOC-2]
2. WHEN `delete(position, length)` is called on a non-read-only document, THE TextBuffer SHALL remove `length` bytes starting at `position`, update the LineIndex by removing line records for any line-end characters in the deleted range, and notify the undo system. [SCI-DOC-2]
3. WHILE inserting text, THE TextBuffer SHALL detect line-end characters (CR, LF, CRLF, and -- when Unicode line-end mode is active -- LS U+2028, PS U+2029, NEL U+0085) and insert corresponding line records into the LineIndex. [SCI-DOC-2, SCI-DOC-7]
4. WHILE deleting text, THE TextBuffer SHALL remove line records for any line-end characters contained within the deleted range and fix up adjacent CR+LF pairs that may be split or joined by the deletion. [SCI-DOC-2]
5. WHEN an insertion splits a CRLF pair (inserting between the CR and LF), THE TextBuffer SHALL create a new line boundary after the LF, treating the CR and LF as separate line endings. [SCI-DOC-2]
6. WHEN a deletion causes a CR to become adjacent to a LF (merging previously separate characters), THE TextBuffer SHALL merge them into a single CRLF line ending by removing the extra line record. [SCI-DOC-2]
7. IF the document is in read-only mode, THEN `insert()` and `delete()` SHALL return an error without modifying content. [SCI-DOC-2]
8. THE TextBuffer SHALL expose a `set_read_only(bool)` method and an `is_read_only()` query to control and inspect the read-only state. [SCI-DOC-2]
9. ALL mutation operations SHALL be routable through the workbench command framework -- the document model crate SHALL provide the operation primitives but SHALL NOT bypass the command dispatch path when invoked from higher layers. [WB]

---

### Requirement 3: Line Index and Position Tracking

**User Story:** As a document model consumer, I want O(log n) bidirectional lookups between line numbers and byte positions, so that viewport rendering, scrolling, and command operations can efficiently map between the two coordinate systems.

**Source:** [SCI-DOC-3], [FFE-MVP-1], [FFE-MVP-2]

#### Acceptance Criteria

1. THE TextBuffer SHALL maintain a LineIndex that maps line numbers to byte positions using a balanced tree or partitioning structure, providing O(log n) lookups in both directions. [SCI-DOC-3]
2. WHEN `line_count()` is called, THE LineIndex SHALL return the total number of lines in the buffer (minimum 1 for an empty document). [SCI-DOC-3]
3. WHEN `line_start(line)` is called with a valid LineNumber, THE LineIndex SHALL return the BytePosition of the first byte on that line. [SCI-DOC-3]
4. IF `line_start(line)` is called with a line number beyond the last line, THEN THE LineIndex SHALL return the BytePosition equal to the document length (one past the last byte). [SCI-DOC-3]
5. WHEN `line_end(line)` is called with a valid LineNumber, THE LineIndex SHALL return the BytePosition of the last content byte on that line (before the line-end sequence), accounting for CR, LF, CRLF, and Unicode line endings when active. [SCI-DOC-3]
6. WHEN `line_from_position(position)` is called with a valid BytePosition, THE LineIndex SHALL return the LineNumber containing that byte position via O(log n) search. [SCI-DOC-3]
7. THE LineIndex SHALL support correct 1-based line numbers for display purposes -- the API SHALL use 0-based LineNumber internally but provide a conversion method to 1-based display numbers. [FFE-MVP-1]
8. WHEN the document is loaded incrementally (streaming), THE LineIndex SHALL be usable for already-indexed lines without waiting for the full index to complete -- partial results are valid. [FFE-MVP-1]
9. THE LineIndex SHALL support an optional character-count index (UTF-16 and UTF-32 character counts per line) for translation between byte offsets and character offsets, allocatable on demand. [SCI-DOC-3]
10. WHEN a character-count index is allocated, THE LineIndex SHALL calculate character widths for all existing lines and maintain them incrementally during subsequent insertions and deletions. [SCI-DOC-3]
11. WHEN a character-count index reference count drops to zero after release, THE LineIndex SHALL deallocate the index storage to reclaim memory. [SCI-DOC-3]

---

### Requirement 4: Windowed File Loading and Index Build

**User Story:** As a developer, I want to open any file from the VFS and begin viewing content instantly while only the visible window (plus overscan) is resident in memory, so that even a multi-gigabyte file opens immediately without being read fully into memory.

**Source:** [FFE-MVP-1], [WB], CR-CH-058 (FOUNDATION-DESIGN sections 4-5)

**REVISION NOTE (CR-CH-058):** This requirement is REVISED IN PLACE. The former
model (progressive-append streaming that left the whole file resident and
"finalize into a COMPLETE index" at 4.5) assumed FULL RESIDENCY and contradicted
universal windowed loading (findings.md Q2, Q8). The criteria below are reworked
to windowed/evict-off-window loading with the Immutable_Original_Index as the
surviving authority. The whole-file-resident guarantees are withdrawn.

#### Acceptance Criteria

1. WHEN a file is opened, THE Document SHALL load ONLY the first Window_Band (first page plus prefetch) via a ranged read from the VFS, and SHALL NOT read the whole file into memory. Chunk size for ranged reads SHALL remain configurable (default 64 KB). [FFE-MVP-1, WB] REVISED (CR-CH-058)
2. WHILE the index build is in progress, THE Document SHALL make the resident window readable immediately -- consumers SHALL NOT be blocked waiting for the whole file to be scanned or loaded. [FFE-MVP-1] REVISED (CR-CH-058)
3. WHEN the owning Command Environment's RecordFormat is `Fixed { lrecl }`, THE Document SHALL build the Immutable_Original_Index ARITHMETICALLY (`offset = K * lrecl`), producing an exact, instant Total_Records with NO scan regardless of file size. [CR-CH-058]
4. WHEN the owning Command Environment's RecordFormat is `Delimited` or `Variable`, THE Document SHALL build the Immutable_Original_Index by a BACKGROUND record-boundary scan (delimiter scan for Delimited, RDW walk for Variable) scheduled through idle processing, and SHALL expose an ESTIMATED Total_Records (file_size / sampled average record length) until the scan completes. [FFE-MVP-1, CR-CH-058] REVISED (was: loading_progress with estimated-total)
5. WHEN the background record-boundary scan completes, THE Document SHALL publish the EXACT Total_Records from the completed Immutable_Original_Index and notify all watchers; THE Document SHALL NOT rescan the whole resident buffer into a dense fully-resident index. [FFE-MVP-1, CR-CH-058] REVISED (was 4.5 "finalize into a COMPLETE index")
6. THE Document SHALL expose an `index_progress()` method returning: not-started, scanning (with records-indexed and estimated-total), complete, or failed; for a `Fixed` document this SHALL report complete immediately. [FFE-MVP-1, CR-CH-058]
7. IF the VFS reports an error during a ranged read or the index scan (file not found, permission denied, I/O error), THEN THE Document SHALL transition to a failed state, preserve any already-resident window, and notify watchers with the error details. [FFE-MVP-1]
8. WHEN no file path is provided (empty session), THE Document SHALL initialize with an empty Append_Buffer, a single Original piece of count 0 (or an empty Piece_List) and a Total_Records of 0 presenting a single empty record. [FFE-MVP-1] REVISED (CR-CH-058)
9. ALL file I/O operations SHALL flow through the VFS abstraction (the `ff-vfs` crate) -- the document model SHALL NOT use `std::fs`, `tokio::fs`, or any platform-specific I/O directly. [WB]
10. THE background index scan and window prefetch SHALL be cancellable -- if the document is closed or replaced before the scan completes, the background task SHALL terminate without resource leaks. [WB] REVISED (CR-CH-058)
11. THE Immutable_Original_Index and Piece_List SHALL be built on open and dropped on close (rebuilt on each open); the first build SHALL NOT persist an on-disk index. [CR-CH-058]

---

### Requirement 5: Line End Type Support

**User Story:** As a document model consumer, I want configurable line-end recognition including Unicode line separators, so that documents using NEL, LS, or PS line endings are handled correctly alongside standard CR/LF/CRLF.

**Source:** [SCI-DOC-7]

#### Acceptance Criteria

1. THE Document SHALL support two LineEndMode values: `Default` (recognises CR, LF, CRLF) and `Unicode` (additionally recognises LS U+2028, PS U+2029, NEL U+0085 as line endings). [SCI-DOC-7]
2. WHEN `set_line_end_mode(mode)` is called with a different mode than the current setting, THE Document SHALL rebuild the LineIndex by rescanning the entire buffer for line endings using the new mode. [SCI-DOC-7]
3. THE Document SHALL expose a `line_end_mode()` query that returns the current LineEndMode. [SCI-DOC-7]
4. WHEN `contains_line_end(text)` is called, THE Document SHALL return `true` if the text contains any recognized line-end sequence for the current LineEndMode. [SCI-DOC-7]
5. THE default LineEndMode for new documents SHALL be `Default` (CR, LF, CRLF only). Unicode line-end mode SHALL be opt-in. [SCI-DOC-7]
6. WHEN Unicode line-end mode is active and an insertion breaks a multi-byte Unicode line-end sequence (e.g., inserting between the bytes of NEL U+0085 encoded as 0xC2 0x85 in UTF-8), THE TextBuffer SHALL detect the overlap and correct the LineIndex accordingly. [SCI-DOC-7]

---

### Requirement 6: Document Lifecycle and Shared Ownership

**User Story:** As a workbench component, I want reference-counted document ownership through `Arc`, so that multiple views, background tasks, and plugins can share a document and it is properly cleaned up when the last reference is dropped.

**Source:** [SCI-DOC-8], [WB]

#### Acceptance Criteria

1. THE Document SHALL be wrapped in a `DocumentHandle` type (defined as `Arc<RwLock<Document>>`) enabling shared ownership across multiple views and threads. [SCI-DOC-8]
2. WHEN a DocumentHandle is cloned, THE reference count SHALL increment, allowing multiple consumers to hold the same document simultaneously. [SCI-DOC-8]
3. WHEN the last DocumentHandle is dropped, THE Document SHALL be deallocated -- no explicit `release()` or `destroy()` call is required (Rust's `Drop` semantics handle this). [SCI-DOC-8]
4. BEFORE a Document is dropped, THE system SHALL notify all registered DocumentWatcher instances via a `notify_deleted()` callback, giving them an opportunity to clean up references. [SCI-DOC-8]
5. THE Document SHALL expose an `add_watcher(watcher)` method that registers a trait object implementing DocumentWatcher, returning a WatcherHandle for later removal. [SCI-DOC-8]
6. THE Document SHALL expose a `remove_watcher(handle)` method that unregisters a previously registered watcher. [SCI-DOC-8]
7. THE Document SHALL be `Send + Sync` -- it SHALL be safe to share DocumentHandle across threads and access it from any thread (with the RwLock providing interior mutability synchronization). [WB]
8. THE Document SHALL support a read-only mode where no mutations are accepted -- multiple views can read concurrently via RwLock read guards. [SCI-DOC-8]

---

### Requirement 7: Document Watcher and Notification System

**User Story:** As a view or plugin, I want to register for document change notifications, so that I can update my state incrementally when the document is modified rather than polling for changes.

**Source:** [SCI-DOC-8], [WB]

#### Acceptance Criteria

1. THE `DocumentWatcher` trait SHALL define the following callback methods: `notify_modify_attempt()`, `notify_insert(position, text, lines_added)`, `notify_delete(position, length, lines_removed)`, `notify_save_point(at_save_point)`, `notify_deleted()`, and `notify_style_needed(end_position)`. [SCI-DOC-8]
2. WHEN text is inserted, THE Document SHALL notify all registered watchers with the insertion position, the inserted text reference, and the number of lines added. [SCI-DOC-8]
3. WHEN text is deleted, THE Document SHALL notify all registered watchers with the deletion position, the number of bytes deleted, and the number of lines removed. [SCI-DOC-8]
4. WHEN a modification is attempted on a read-only document, THE Document SHALL notify all watchers via `notify_modify_attempt()` so that the application can prompt the user or unlock the file. [SCI-DOC-8]
5. WHEN the document reaches or leaves its save point (the state matching the on-disk content), THE Document SHALL notify all watchers via `notify_save_point(at_save_point)`. [SCI-DOC-8]
6. THE watcher notification system SHALL be non-blocking -- watchers that perform expensive work in response to notifications SHALL be responsible for deferring that work off the notification path. [WB]
7. IF a watcher is added that is already registered (same trait object), THEN `add_watcher()` SHALL return an error without duplicating the registration. [SCI-DOC-8]

---

### Requirement 8: Character and Encoding Navigation

**User Story:** As a cursor movement system, I want encoding-aware character navigation that handles UTF-8 multi-byte sequences and CRLF pairs as atomic units, so that cursor movement never lands inside a character or splits a line-ending pair.

**Source:** [SCI-DOC-11]

#### Acceptance Criteria

1. WHEN `char_length_at(position)` is called, THE Document SHALL return the byte length of the character at that position: 2 for CR+LF pairs, 1–4 for valid UTF-8 sequences, and 1 for invalid bytes (replacement character treatment). [SCI-DOC-11]
2. WHEN `move_position_outside_char(position, direction)` is called with a position inside a multi-byte UTF-8 sequence, THE Document SHALL adjust the position in the indicated direction to the nearest valid character boundary. [SCI-DOC-11]
3. WHEN `next_position(position, direction)` is called, THE Document SHALL advance to the next valid character position in the given direction, treating each code point (and CRLF pairs) as atomic units. [SCI-DOC-11]
4. WHEN `character_at(position)` is called, THE Document SHALL return a CharacterExtracted containing the Unicode code point (as `char`) and the byte width of the character at that position. [SCI-DOC-11]
5. WHEN `character_before(position)` is called, THE Document SHALL return a CharacterExtracted for the character immediately before the given position by scanning backwards through the UTF-8 encoding. [SCI-DOC-11]
6. WHEN `relative_position(start, character_offset)` is called, THE Document SHALL advance `character_offset` characters from `start`, returning `None` if the result would be out of bounds. [SCI-DOC-11]
7. THE Document SHALL treat CR+LF pairs as a single atomic unit for navigation -- `next_position` SHALL never land between a CR and its following LF. [SCI-DOC-11]
8. THE Document SHALL validate UTF-8 sequences during navigation -- invalid byte sequences SHALL be treated as individual bytes (one byte = one character) rather than causing errors or panics. [SCI-DOC-11]

---

### Requirement 9: Viewport Position Management

**User Story:** As a viewport renderer, I want the document to maintain a top-line pointer and provide clamped scrolling arithmetic, so that scroll operations always produce valid display positions.

**Source:** [FFE-MVP-2]

#### Acceptance Criteria

1. THE Document SHALL maintain a `top_line` value (1-based LineNumber) that identifies the first line currently visible in the viewport. [FFE-MVP-2]
2. WHEN `scroll_page_down(visible_count)` is called, THE Document SHALL advance `top_line` by `visible_count` lines, clamped so that the last line of the document is visible on the last page. [FFE-MVP-2]
3. WHEN `scroll_page_up(visible_count)` is called, THE Document SHALL retreat `top_line` by `visible_count` lines, clamped to line 1 (the first line). [FFE-MVP-2]
4. WHEN `scroll_line_down(count)` is called, THE Document SHALL advance `top_line` by `count` lines, clamped to prevent scrolling past the last displayable page. [FFE-MVP-2]
5. WHEN `scroll_line_up(count)` is called, THE Document SHALL retreat `top_line` by `count` lines, clamped to line 1. [FFE-MVP-2]
6. WHEN `set_top_line(line)` is called with a specific line number, THE Document SHALL set `top_line` to that value, clamped to the valid range [1, max_top_line]. [FFE-MVP-2]
7. THE Document SHALL expose a `max_top_line(visible_count)` method that returns the maximum valid `top_line` given a viewport of `visible_count` lines -- computed as `max(1, line_count - visible_count + 1)`. [FFE-MVP-2]
8. ALL scroll operations SHALL be deterministic and idempotent at boundaries -- calling `scroll_page_up` when already at line 1 SHALL have no effect on `top_line`. [FFE-MVP-2]

---

### Requirement 10: Save Point and Modification State

**User Story:** As an editor UI component, I want to know whether the document has unsaved modifications, so that I can display modification indicators and prompt before closing.

**Source:** [FFE-MVP-1], [SCI-DOC-8]

#### Acceptance Criteria

1. THE Document SHALL maintain a save-point marker that records the undo state corresponding to the on-disk content. [SCI-DOC-8]
2. WHEN `set_save_point()` is called (typically after a successful save), THE Document SHALL record the current undo position as the save point. [SCI-DOC-8]
3. WHEN `is_at_save_point()` is called, THE Document SHALL return `true` if the current undo position matches the save-point marker, indicating no unsaved changes. [SCI-DOC-8]
4. WHEN the document transitions away from the save point (first modification after save), THE Document SHALL notify all watchers via `notify_save_point(false)`. [SCI-DOC-8]
5. WHEN the document returns to the save point (e.g., via undo back to saved state), THE Document SHALL notify all watchers via `notify_save_point(true)`. [SCI-DOC-8]
6. WHEN a file is first loaded successfully, THE Document SHALL set the save point after loading completes, marking the loaded state as unmodified. [FFE-MVP-1]

---

### Requirement 11: Universal Record Abstraction and RecordFormat

**User Story:** As a workbench platform component, I want the editable unit of a document to be a universal RECORD whose framing is supplied by the owning Command Environment, so that a native delimited line and a mainframe fixed-length record are edited through the same model and native editing stays byte-identical.

**Source:** CR-CH-058 (FOUNDATION-DESIGN section 3), framework-conformance.md, findings.md Q6

#### Acceptance Criteria

11.1 THE document model SHALL define a `RecordFormat` type in `ff-document-model` with variants `Delimited { terminator: Crlf | Lf | Cr | Mixed }`, `Fixed { lrecl }`, and `Variable { max_lrecl, rdw }`, and the editable unit SHALL be a RECORD described by that format. [CR-CH-058]

11.2 WHEN a document is opened, THE Document SHALL obtain its RecordFormat from the owning Command Environment (the native CE SHALL supply `Delimited`, generalising the existing LineEndMode; mainframe CEs supply `Fixed`/`Variable` under a later V-stream gate). [CR-CH-058]

11.3 WHEN the RecordFormat is `Delimited`, THE Document SHALL treat each delimiter-terminated line as a record such that record navigation, line navigation, and the observable "line" API are byte-for-byte equivalent to the pre-CR-CH-058 behaviour (a Delimited record IS a line). [CR-CH-058]

11.4 THE document model SHALL expose record-oriented queries (`total_records()`, `record_start(k)`, `record_byte_length(k)`, `record_from_position(pos)`) alongside the retained line API, with the line API delegating to the record API for Delimited documents. [CR-CH-058]

11.5 THE document model SHALL NOT flatten Fixed/Variable records to delimiter-terminated bytes before the editor sees them; record framing SHALL remain the authority so a later Fixed-format save can re-pad to LRECL (this is the universal replacement for the provider-side flatten-to-newline described in findings.md Q6; the mainframe save itself is OUT OF SCOPE of this gate). [CR-CH-058]

11.6 THE `RecordFormat` and record API SHALL be GUI-independent and SHALL NOT change the shell command-dispatch, navigation-stack, focus-latch, or WorkspaceDescriptor-persistence framework seams. [framework-conformance.md]

---

### Requirement 12: Piece-Table Spine and Windowed Byte Residency

**User Story:** As a developer editing very large files, I want the document represented by a small immutable original index plus a small piece list of changes, with only a windowed band of record bytes resident, so that navigation is fast, edits are cheap anywhere, and memory stays bounded regardless of file size.

**Source:** CR-CH-058 (FOUNDATION-DESIGN sections 2, 4, 5), findings.md section 3

**FRAMEWORK-TOUCHING NOTE:** This requirement reworks the PUBLIC surface of
`ff-document-model` (`Document` / `GapBuffer` / `TextBuffer`). Per
framework-conformance.md this is an owner-confirmed foundational change to a
core type; it does NOT alter the shell dispatch, navigation, focus, or
persistence mechanisms.

#### Acceptance Criteria

12.1 THE document model SHALL represent a document as three layers: (A) an Immutable_Original_Index (one lean ~16-byte entry per original record, never mutated after open), (B) an ordered Piece_List of `Original { first_record, count }` / `Edited { buf_range, count }` pieces each carrying a running record-count, and (C) an append-only Append_Buffer holding only edited/inserted record bytes. [CR-CH-058]

12.2 WHEN a line/record is inserted, deleted, moved, copied, or overtyped, THE document model SHALL perform the edit as a Piece_List SPLICE (never an O(file) byte-array shift and never a whole-file rewrite); unedited spans SHALL remain a single Original piece. [CR-CH-058]

12.3 THE record numbers SHALL be `u64` everywhere (no type ceiling); the design SHALL target a ceiling of ~100,000,000 resident-index records (~1.6 GB lean index) as a MEMORY-BUDGET target, not a type limit. [CR-CH-058]

12.4 THE physically resident byte cache SHALL be a Window_Band of 3 pages (current page + one prefetched page above + one below), a page being the maximum records that fit a full screen at the SMALLEST zoom; byte residency SHALL be decoupled from the logical index. [CR-CH-058]

12.5 THE scrollbar and navigation extents SHALL size on Total_Records from the index, NEVER on the resident Window_Band; `total_records` (whole file) and the resident window size SHALL never be fused. [CR-CH-058] (fixes the owner-observed scrollbar-collapses-to-window bug, findings.md section 1)

12.6 ZOOM SHALL NEVER trigger a load: changing zoom SHALL change only how many resident records are RENDERED, never which records are LOADED. [CR-CH-058]

12.7 WHEN scrolling, THE document model SHALL load and evict record bytes at the Window_Band edges with hysteresis and overscan, debounced so that a fast jump performs ONE window load rather than a cascade of boundary loads; dirty (edited) pieces SHALL be PINNED and never evicted until saved. [CR-CH-058]

12.8 WHEN a `down N` / `up N` jump targets a record outside the resident band, THE document model SHALL resolve the target record via the index (O(1) for Fixed, O(log pieces)/linear-over-pieces otherwise), load that window, and SHALL NOT read the intermediate records. [CR-CH-058]

12.9 THE Piece_List search mapping "current record N -> piece" SHALL start as a LINEAR scan over pieces; an order-statistic tree over PIECES MAY be introduced ONLY if fragmentation profiling demands it. [CR-CH-058]

12.10 THE document model SHALL expose configurable budgets matching the existing configuration pattern: window size (records), overscan, checkpoint interval, `max_resident_records` (default ~150,000,000), and spill threshold. [CR-CH-058]

12.11 WHEN the resident index would exceed `max_resident_records`, THE document model SHALL degrade gracefully rather than crash or hang, behind an index interface that admits either a reserved sparse/windowed-index mode OR mmap/spill of the Immutable_Original_Index; this scalability guard SHALL be DESIGNED-FOR now and the first build SHALL ship the lean fully-resident index only. [CR-CH-058]

12.12 FOR a native (Delimited) document, opening, editing, and saving SHALL produce BYTE-IDENTICAL output to the pre-CR-CH-058 behaviour (same bytes, same line endings, same dirty/save-point behaviour), windowing and the record model notwithstanding. [CR-CH-058] (non-negotiable safety rule)

---

### Requirement 13: SAVE-walk store-call selection over the record-aware store contract (CR-CH-060)

**User Story:** As a developer wiring the editor SAVE walk, I want the walk to
hand the owning Command Environment either a byte buffer (native/Delimited) or the
re-framed RECORDS (Fixed/Variable) through the owning CE's store contract, so that
native SAVE stays byte-identical while a mainframe document's records reach the
owning CE for RECFM/LRECL-correct storage.

**Source:** CR-CH-060 (the record-aware `ff-vfs::BackendEnvironment` store
contract, command-environments Requirement 18); FOUNDATION-DESIGN section 6 (SAVE
walk); CR-CH-058 Req 11-12.

**Marking:** This requirement is the document-model/editor SIDE of the
owner-directed CR-CH-060 framework change. It does NOT itself reshape the
`BackendEnvironment` trait (that is command-environments Req 18); it specifies how
the SAVE walk SELECTS and CALLS the contract. ADDITIVE: Delimited/native SAVE is
unchanged (Req 12.12).

#### Acceptance Criteria

13.1 WHEN the editor saves a document, THE SAVE walk SHALL re-frame the Piece_List
   per the owning Command Environment's RecordFormat (Req 12 / FOUNDATION-DESIGN
   section 6) and SHALL deliver the result to the owning CE through the CR-CH-060
   record-aware store contract (command-environments Req 18), NOT by writing a
   flat byte buffer for every document. [CR-CH-060]

13.2 WHEN the owning CE advertises a `Delimited` RecordFormat (native/host), THE
   SAVE walk SHALL call the owning CE's BYTE store entry (`save(path, bytes)`), and
   the emitted bytes SHALL be byte-identical to the pre-CR-CH-058 behaviour
   (Req 12.12). [CR-CH-060]

13.3 WHEN the owning CE advertises a `Fixed` or `Variable` RecordFormat, THE SAVE
   walk SHALL call the owning CE's RECORD-aware store entry with the re-framed
   records plus the dataset identity and record attributes (RECFM/LRECL/encoding).
   [CR-CH-060]

13.4 THE SAVE walk SHALL obtain the RecordFormat used for the byte-vs-record
   selection from the SAME value the owning CE supplied at OPEN (Req 11.2); it
   SHALL NOT re-derive record framing independently of the editor's RecordFormat.
   [CR-CH-060]

13.5 THE SAVE walk SHALL ride the SINGLE existing SAVE-addressing seam (CR-CH-053
   Task 20/21: FFEDIT addresses SAVE to the owning CE); it SHALL NOT add a parallel
   save path or a second dispatcher, and SHALL NOT change the shell command
   -dispatch, navigation-stack, focus-latch, or WorkspaceDescriptor-persistence
   framework seams. [CR-CH-060, framework-conformance.md]

13.6 THE record attributes delivered on a Fixed/Variable save SHALL be GUI
   -independent plain data (RECFM/LRECL/encoding), carrying no editor/shell type,
   so the owning CE (the mainframe CE in `ff-idcams`) can store the records over
   `ff_dscatalog::DatasetAccess` without a dependency back into the editor or the
   shell. [CR-CH-060]

---

## Cross-References

- **`virtual-file-system`**: The document-model uses VFS for all file access (ranged/windowed reads, saves). [WB]
- **`command-environments`**: The owning Command Environment SUPPLIES the RecordFormat (Delimited for native, Fixed/Variable for mainframe) and is the SAVE-addressing seam (CR-CH-053 Task 20/21). The record-aware `BackendEnvironment` store contract the SAVE walk calls (Req 13 here) is owned by command-environments Req 18 (CR-CH-060). [CR-CH-058, CR-CH-060]
- **`large-file-performance`**: Consumes the windowed record/line access (its Req 7 range access becomes mandatory under CR-CH-058); supplies the render/measurement cache over the Window_Band. [CR-CH-058]
- **`idle-processing`**: Schedules the background record-boundary scan and the Window_Band prefetch. [CR-CH-058]
- **`undo-redo-transactions`**: The document-model integrates with the undo system -- insert/delete operations record undo actions. The undo-redo-transactions spec is authoritative for transaction semantics. [SCI-DOC-2]
- **`edit-operations`**: Higher-level edit operations (character typing, selection replacement, multi-caret edits) use the document-model's insert/delete primitives. [WB]
- **`display-line-mapping`**: The display-line-mapping crate consumes the LineIndex to map document lines to display lines (accounting for folding, wrapping, exclusion). [SCI-DOC-3]
- **`encoding-and-characters`**: Detailed encoding detection, BOM handling, and encoding conversion are specified in encoding-and-characters. The document-model provides UTF-8 character navigation; encoding-and-characters handles the broader encoding surface. [SCI-DOC-11]
- **`background-io`**: Large-file async loading coordination is specified in background-io. The document-model defines the streaming interface; background-io provides the task scheduling. [FFE-MVP-1]
