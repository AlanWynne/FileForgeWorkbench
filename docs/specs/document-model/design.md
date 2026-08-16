# Design Document: Document Model (`ff-document-model`)

## Overview

The `ff-document-model` crate is the **foundational text storage layer** for the FileForgeWorkbench editor. It provides gap-buffer-based text storage, efficient O(log n) line indexing, large-file streaming support, encoding-aware character navigation, document lifecycle management, and a watcher notification system.

### Purpose

- Store and manage document text using a gap-buffer data structure
- Provide efficient bidirectional mapping between line numbers and byte positions
- Support incremental streaming file loading via the VFS (FFW-ARCH-001)
- Expose encoding-aware character navigation for cursor movement
- Manage document lifecycle through `Arc<RwLock<Document>>` shared ownership
- Deliver modification notifications to registered watchers
- Track viewport position (top_line) with clamped scroll arithmetic
- Track save-point state for modification indicators

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
├─────────────────────────────────────────────────────────────┤
│  Higher Feature Crates: ff-edit-operations, ff-undo-redo,    │
│    ff-display-line-mapping, ff-viewport-scrolling            │
│         (consume ff-document-model public API)               │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-document-model ← Wave 4                     │
├─────────────────────────────────────────────────────────────┤
│  Core Layer: ff-vfs (content access), ff-core (runtime),     │
│              ff-command (mutation routing)                    │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: ALL file access goes through `ff-vfs` — no `std::fs` or `tokio::fs` in this crate
- **GUI Independence (Req 2)**: Zero GUI dependencies — no egui, winit, wgpu
- **Command-Driven (Req 4)**: Mutation primitives are designed for command-framework integration; higher layers route edits through commands
- **Async I/O (Req 6)**: Streaming file loading uses async I/O via the VFS `read_stream` API
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-document-model`
- **Error Message Standards (Req 8)**: Errors follow `[document] operation: description` format

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Consumers [Consuming Crates]
        EDIT[ff-edit-operations]
        UNDO[ff-undo-redo-transactions]
        DLM[ff-display-line-mapping]
        VP[ff-viewport-and-scrolling]
        FIO[ff-file-operations]
    end

    subgraph ff-document-model [ff-document-model Crate]
        DOC[Document]
        HANDLE[DocumentHandle]
        TB[TextBuffer]
        GB[GapBuffer]
        LI[LineIndex]
        SLI[SparseLineIndex]
        SFR[StreamingFileReader]
        WATCH[Watcher Registry]
        NAV[Character Navigator]
        VP_MGR[Viewport Manager]
        SP[SavePoint Tracker]
    end

    subgraph Upstream [Upstream Crates]
        VFS[ff-vfs]
        LOG[ff-logging]
    end

    EDIT -->|insert/delete| DOC
    UNDO -->|save_point queries| DOC
    DLM -->|line_start/line_end| LI
    VP -->|scroll operations| VP_MGR
    FIO -->|open/save via VFS| SFR

    DOC --> HANDLE
    DOC --> TB
    DOC --> WATCH
    DOC --> NAV
    DOC --> VP_MGR
    DOC --> SP
    TB --> GB
    TB --> LI
    SFR --> VFS
    SFR --> SLI
    SLI -->|finalize| LI
    DOC --> LOG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **Document** | High-level text model: wraps TextBuffer, encoding navigation, watcher notifications, lifecycle, viewport |
| **DocumentHandle** | `Arc<RwLock<Document>>` shared ownership for multi-view/multi-thread access |
| **TextBuffer** | Gap-buffer text storage + line index maintenance + read-only guard |
| **GapBuffer** | Low-level contiguous allocation with movable gap for O(1) amortized edits |
| **LineIndex** | Balanced-tree mapping between line numbers and byte positions (O(log n)) |
| **SparseLineIndex** | Incremental checkpoint index built during streaming load (1 entry per N lines) |
| **StreamingFileReader** | Async chunked reader consuming VFS `read_stream`, feeding GapBuffer + SparseLineIndex |
| **Watcher Registry** | Trait-object registry for `DocumentWatcher` subscribers with notification dispatch |
| **Character Navigator** | UTF-8/CRLF-aware character boundary navigation |
| **Viewport Manager** | top_line tracking, clamped scroll arithmetic |
| **SavePoint Tracker** | Undo-position-based save-point tracking for modification state |

### Data Flow: File Open

```
1. Consumer calls Document::open(uri) with a VFS ResourceUri
2. StreamingFileReader calls vfs.read_stream(uri) → AsyncRead stream
3. StreamingFileReader reads chunks (default 64 KB) from the stream
4. Each chunk is appended to the GapBuffer (gap at end during loading)
5. SparseLineIndex scans each chunk for line endings, records checkpoints
6. Document exposes partially-loaded content for progressive display
7. On stream completion, SparseLineIndex finalizes into full LineIndex
8. Document notifies watchers that loading is complete
9. Document sets the initial save point
```

---

## Components and Interfaces

```
crates/ff-document-model/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── document.rs             # Document struct, public API, watcher dispatch
│   ├── handle.rs               # DocumentHandle type alias, constructor helpers
│   ├── buffer/
│   │   ├── mod.rs              # TextBuffer re-exports
│   │   ├── text_buffer.rs      # TextBuffer struct: owns GapBuffer + LineIndex
│   │   ├── gap_buffer.rs       # GapBuffer: low-level storage with movable gap
│   │   └── split_view.rs       # SplitView: two-segment read access
│   ├── index/
│   │   ├── mod.rs              # LineIndex re-exports
│   │   ├── line_index.rs       # LineIndex: balanced-tree line↔position mapping
│   │   ├── sparse_index.rs     # SparseLineIndex: incremental checkpoint builder
│   │   └── char_index.rs       # Optional character-count index (UTF-16/UTF-32)
│   ├── streaming/
│   │   ├── mod.rs              # Streaming re-exports
│   │   ├── reader.rs           # StreamingFileReader: async chunked VFS reader
│   │   └── progress.rs         # LoadingProgress enum and tracking
│   ├── navigation/
│   │   ├── mod.rs              # Navigation re-exports
│   │   └── character.rs        # UTF-8/CRLF character navigation functions
│   ├── watcher.rs              # DocumentWatcher trait, WatcherHandle, registry
│   ├── viewport.rs             # Viewport position management, scroll clamping
│   ├── save_point.rs           # Save-point tracking
│   ├── line_end.rs             # LineEndMode enum, line-end detection utilities
│   ├── types.rs                # BytePosition, LineNumber, CharacterExtracted newtypes
│   └── error.rs                # DocumentError enum
└── tests/
    ├── gap_buffer_tests.rs     # GapBuffer unit + property tests
    ├── text_buffer_tests.rs    # TextBuffer insertion/deletion tests
    ├── line_index_tests.rs     # LineIndex lookup property tests
    ├── streaming_tests.rs      # StreamingFileReader with mock VFS
    ├── navigation_tests.rs     # Character navigation property tests
    ├── viewport_tests.rs       # Viewport scroll clamping property tests
    ├── watcher_tests.rs        # Watcher notification delivery tests
    ├── save_point_tests.rs     # Save-point state transition tests
    └── integration.rs          # End-to-end document open/edit/save flow
```

---

## Data Models

### Core Newtypes

```rust
/// A byte offset within the document buffer. Uses u64 to support >2 GB documents.
///
/// Addresses: Requirement 1 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytePosition(pub u64);

/// A 0-based line number within the document.
///
/// Addresses: Requirement 3 AC 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineNumber(pub u64);

impl LineNumber {
    /// Convert to 1-based display number.
    pub fn to_display(self) -> u64 {
        self.0 + 1
    }

    /// Create from a 1-based display number.
    pub fn from_display(display: u64) -> Self {
        Self(display.saturating_sub(1))
    }
}

/// A Unicode code point extracted from the buffer with its byte width.
///
/// Addresses: Requirement 8 AC 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterExtracted {
    /// The Unicode code point (or U+FFFD for invalid bytes)
    pub character: char,
    /// Number of bytes this character occupies in UTF-8
    pub byte_width: u8,
}

/// Direction for character navigation.
///
/// Addresses: Requirement 8 AC 2, AC 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}
```

### LineEndMode

```rust
/// Configures which byte sequences are recognised as line endings.
///
/// Addresses: Requirement 5 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEndMode {
    /// Recognises CR (0x0D), LF (0x0A), and CRLF (0x0D 0x0A)
    Default,
    /// Additionally recognises LS (U+2028), PS (U+2029), NEL (U+0085)
    Unicode,
}

impl Default for LineEndMode {
    fn default() -> Self {
        Self::Default
    }
}
```

### GapBuffer

```rust
/// Low-level text storage with a movable gap for O(1) amortized editing.
/// The gap sits at the current edit position; insertions fill the gap,
/// deletions expand it.
///
/// Addresses: Requirement 1, criteria 1–10
pub struct GapBuffer {
    /// Raw byte storage (includes gap region)
    storage: Vec<u8>,
    /// Start of the gap (byte offset in storage)
    gap_start: u64,
    /// End of the gap (byte offset in storage, exclusive)
    gap_end: u64,
    /// Growth factor when gap is exhausted (default: 2.0)
    growth_factor: f64,
}

impl GapBuffer {
    /// Create an empty gap buffer with initial capacity.
    pub fn new(initial_capacity: u64) -> Self;

    /// Pre-allocate storage for at least `capacity` bytes.
    /// Addresses: Requirement 1 AC 8
    pub fn allocate(&mut self, capacity: u64);

    /// Total content length (excluding the gap).
    /// Addresses: Requirement 1 AC 9
    pub fn length(&self) -> u64;

    /// Insert bytes at the given position. Moves gap if needed.
    pub fn insert(&mut self, position: u64, data: &[u8]);

    /// Delete `length` bytes starting at `position`. Expands gap.
    pub fn delete(&mut self, position: u64, length: u64);

    /// Get a single byte at a content position.
    /// Addresses: Requirement 1 AC 3, AC 4
    pub fn byte_at(&self, position: u64) -> Option<u8>;

    /// Copy a range of content bytes into a Vec.
    /// Addresses: Requirement 1 AC 5
    pub fn get_range(&self, position: u64, length: u64) -> Option<Vec<u8>>;

    /// Compact the gap and return a contiguous slice.
    /// Addresses: Requirement 1 AC 6
    pub fn contiguous_view(&mut self) -> &[u8];

    /// Return a two-segment view without moving the gap.
    /// Addresses: Requirement 1 AC 7
    pub fn split_view(&self) -> SplitView<'_>;
}
```

### SplitView

```rust
/// Two-segment read-only view over the gap buffer content.
/// Segment 1 = bytes before the gap; Segment 2 = bytes after the gap.
/// Enables efficient iteration without gap compaction.
///
/// Addresses: Requirement 1 AC 7
pub struct SplitView<'a> {
    pub before_gap: &'a [u8],
    pub after_gap: &'a [u8],
}

impl<'a> SplitView<'a> {
    /// Total content length across both segments.
    pub fn length(&self) -> u64;

    /// Get byte at a logical content position.
    pub fn byte_at(&self, position: u64) -> Option<u8>;

    /// Iterate over all content bytes in order.
    pub fn iter(&self) -> impl Iterator<Item = u8> + 'a;
}
```

### TextBuffer

```rust
/// Primary text storage: owns the GapBuffer and maintains the LineIndex.
/// Coordinates insertion/deletion with line tracking and read-only guards.
///
/// Addresses: Requirements 1–3
pub struct TextBuffer {
    /// The underlying gap buffer storing raw bytes
    buffer: GapBuffer,
    /// Line number ↔ byte position mapping
    line_index: LineIndex,
    /// Current line-end recognition mode
    line_end_mode: LineEndMode,
    /// Whether the buffer is read-only
    read_only: bool,
}

impl TextBuffer {
    /// Create an empty text buffer.
    pub fn new() -> Self;

    /// Create a text buffer with pre-allocated capacity.
    pub fn with_capacity(capacity: u64) -> Self;

    /// Total byte length of content.
    pub fn length(&self) -> u64;

    /// Number of lines in the buffer (minimum 1).
    /// Addresses: Requirement 3 AC 2
    pub fn line_count(&self) -> u64;

    /// Insert text at position, updating line index.
    /// Addresses: Requirement 2 AC 1
    pub fn insert(&mut self, position: BytePosition, text: &[u8])
        -> Result<InsertResult, DocumentError>;

    /// Delete bytes at position, updating line index.
    /// Addresses: Requirement 2 AC 2
    pub fn delete(&mut self, position: BytePosition, length: u64)
        -> Result<DeleteResult, DocumentError>;

    /// Get byte at position.
    pub fn char_at(&self, position: BytePosition) -> Option<u8>;

    /// Get range of bytes.
    pub fn get_range(&self, position: BytePosition, length: u64) -> Option<Vec<u8>>;

    /// Compact and return contiguous view.
    pub fn contiguous_view(&mut self) -> &[u8];

    /// Return split view without compaction.
    pub fn split_view(&self) -> SplitView<'_>;

    /// Set read-only mode.
    /// Addresses: Requirement 2 AC 8
    pub fn set_read_only(&mut self, read_only: bool);

    /// Query read-only state.
    pub fn is_read_only(&self) -> bool;

    /// Get the byte position of the start of a line.
    /// Addresses: Requirement 3 AC 3, AC 4
    pub fn line_start(&self, line: LineNumber) -> BytePosition;

    /// Get the byte position of the end of a line (before line ending).
    /// Addresses: Requirement 3 AC 5
    pub fn line_end(&self, line: LineNumber) -> BytePosition;

    /// Find which line contains a byte position.
    /// Addresses: Requirement 3 AC 6
    pub fn line_from_position(&self, position: BytePosition) -> LineNumber;

    /// Set line-end mode, rescanning if changed.
    /// Addresses: Requirement 5 AC 2
    pub fn set_line_end_mode(&mut self, mode: LineEndMode);

    /// Get current line-end mode.
    pub fn line_end_mode(&self) -> LineEndMode;
}

/// Result of an insertion operation.
#[derive(Debug, Clone)]
pub struct InsertResult {
    /// Number of lines added by the insertion
    pub lines_added: u64,
    /// Byte length of inserted content
    pub bytes_inserted: u64,
}

/// Result of a deletion operation.
#[derive(Debug, Clone)]
pub struct DeleteResult {
    /// Number of lines removed by the deletion
    pub lines_removed: u64,
    /// Byte length of deleted content
    pub bytes_deleted: u64,
}
```

### LineIndex

```rust
/// Balanced-tree structure mapping line numbers to byte positions.
/// Provides O(log n) lookups in both directions.
///
/// Addresses: Requirement 3, criteria 1–11
pub struct LineIndex {
    /// Internal balanced tree (B-tree or similar) storing line start positions
    entries: BTreeMap<u64, u64>,  // line_number → byte_position
    /// Optional UTF-16 character count index (allocated on demand)
    char_index: Option<CharacterCountIndex>,
    /// Reference count for character index consumers
    char_index_refs: u32,
}

impl LineIndex {
    /// Create a new line index with a single line at position 0.
    pub fn new() -> Self;

    /// Total number of lines.
    pub fn line_count(&self) -> u64;

    /// Byte position of the first byte on a line.
    /// Addresses: Requirement 3 AC 3, AC 4
    pub fn line_start(&self, line: LineNumber) -> BytePosition;

    /// Byte position of the end of content on a line.
    /// Addresses: Requirement 3 AC 5
    pub fn line_end(&self, line: LineNumber, buffer: &GapBuffer) -> BytePosition;

    /// Find line containing a byte position via O(log n) search.
    /// Addresses: Requirement 3 AC 6
    pub fn line_from_position(&self, position: BytePosition) -> LineNumber;

    /// Insert a new line record at the given byte position.
    pub fn insert_line(&mut self, after_line: LineNumber, position: BytePosition);

    /// Remove a line record.
    pub fn remove_line(&mut self, line: LineNumber);

    /// Allocate or increment reference to character count index.
    /// Addresses: Requirement 3 AC 9
    pub fn allocate_char_index(&mut self, buffer: &GapBuffer);

    /// Release one reference to character count index.
    /// Addresses: Requirement 3 AC 11
    pub fn release_char_index(&mut self);

    /// Rebuild the entire index by scanning the buffer for line endings.
    pub fn rebuild(&mut self, buffer: &GapBuffer, mode: LineEndMode);
}
```

### SparseLineIndex

```rust
/// Incremental checkpoint index built during streaming file loading.
/// Records one entry per N lines (default 1000), enabling partial
/// line lookups before the full index is available.
///
/// Addresses: Requirement 4 AC 3
pub struct SparseLineIndex {
    /// Checkpoint entries: (line_number, byte_position)
    checkpoints: Vec<(u64, u64)>,
    /// Lines per checkpoint
    checkpoint_interval: u64,
    /// Total lines seen so far
    total_lines: u64,
    /// Total bytes processed
    bytes_processed: u64,
}

impl SparseLineIndex {
    /// Create a new sparse index with the given checkpoint interval.
    pub fn new(checkpoint_interval: u64) -> Self;

    /// Process a chunk of bytes, recording checkpoints as line endings are found.
    pub fn process_chunk(&mut self, chunk: &[u8], chunk_offset: u64, mode: LineEndMode);

    /// Finalize into a complete LineIndex.
    /// Addresses: Requirement 4 AC 5
    pub fn finalize(self, buffer: &GapBuffer, mode: LineEndMode) -> LineIndex;

    /// Query an approximate line number for a byte position (using checkpoints).
    pub fn approximate_line(&self, position: BytePosition) -> Option<LineNumber>;

    /// Total lines counted so far.
    pub fn lines_counted(&self) -> u64;
}
```

### StreamingFileReader

```rust
/// Async chunked file reader that loads content from the VFS in
/// configurable chunks, feeding the GapBuffer and SparseLineIndex.
///
/// Addresses: Requirement 4, criteria 1–9
pub struct StreamingFileReader {
    /// Chunk size in bytes (default 64 KB)
    chunk_size: usize,
    /// Cancellation token for cooperative shutdown
    cancel_token: CancellationToken,
}

impl StreamingFileReader {
    /// Create a reader with the specified chunk size.
    pub fn new(chunk_size: usize) -> Self;

    /// Create with default chunk size (64 KB).
    pub fn default() -> Self;

    /// Load a file from the VFS into the provided buffer and sparse index.
    /// Returns a progress stream that reports loading state.
    ///
    /// Addresses: Requirement 4 AC 1, AC 8
    pub async fn load(
        &self,
        vfs: &Vfs,
        uri: &ResourceUri,
        buffer: &mut GapBuffer,
        sparse_index: &mut SparseLineIndex,
    ) -> Result<(), DocumentError>;

    /// Cancel an in-progress load.
    /// Addresses: Requirement 4 AC 9
    pub fn cancel(&self);

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool;
}
```

### LoadingProgress

```rust
/// Current state of a streaming file load operation.
///
/// Addresses: Requirement 4 AC 4
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingProgress {
    /// Load has not started
    NotStarted,
    /// Load is in progress
    InProgress {
        /// Bytes loaded so far
        bytes_loaded: u64,
        /// Estimated total bytes (from VFS metadata, may be None)
        estimated_total: Option<u64>,
    },
    /// Load completed successfully
    Complete {
        /// Total bytes loaded
        total_bytes: u64,
        /// Total lines in the document
        total_lines: u64,
    },
    /// Load failed with an error
    Failed {
        /// Error description
        reason: String,
        /// Bytes loaded before failure
        bytes_loaded: u64,
    },
}
```

### Document

```rust
/// The high-level text model. Wraps TextBuffer and adds encoding navigation,
/// watcher notifications, lifecycle management, viewport, and save-point.
///
/// Addresses: Requirements 1–10
pub struct Document {
    /// The text storage and line index
    buffer: TextBuffer,
    /// Registered document watchers
    watchers: Vec<Box<dyn DocumentWatcher>>,
    /// Current loading state
    loading_progress: LoadingProgress,
    /// Viewport top-line position (1-based)
    top_line: u64,
    /// Save-point marker (undo position at last save)
    save_point: Option<u64>,
    /// Current undo position counter
    undo_position: u64,
    /// The VFS URI this document was loaded from (None for untitled)
    source_uri: Option<ResourceUri>,
}
```

### DocumentHandle

```rust
/// Shared ownership handle for a Document. Enables multi-view and
/// multi-thread access with interior mutability via RwLock.
///
/// Addresses: Requirement 6, criteria 1–8
pub type DocumentHandle = Arc<RwLock<Document>>;

/// Create a new DocumentHandle wrapping an empty document.
pub fn new_document() -> DocumentHandle;

/// Create a DocumentHandle by loading from a VFS URI.
pub async fn open_document(vfs: &Vfs, uri: &ResourceUri) -> Result<DocumentHandle, DocumentError>;
```

### DocumentWatcher Trait

```rust
/// Trait for receiving document change notifications.
/// Implementations must be non-blocking; expensive work should be deferred.
///
/// Addresses: Requirement 7, criteria 1–7
pub trait DocumentWatcher: Send + Sync {
    /// Called when a modification is attempted on a read-only document.
    fn notify_modify_attempt(&self) {}

    /// Called after text is inserted.
    fn notify_insert(&self, position: BytePosition, text: &[u8], lines_added: u64) {}

    /// Called after text is deleted.
    fn notify_delete(&self, position: BytePosition, length: u64, lines_removed: u64) {}

    /// Called when the document reaches or leaves its save point.
    fn notify_save_point(&self, at_save_point: bool) {}

    /// Called before the document is deallocated.
    fn notify_deleted(&self) {}

    /// Called when syntax styling needs to be extended to a position.
    fn notify_style_needed(&self, end_position: BytePosition) {}
}

/// Handle returned by add_watcher for later removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WatcherHandle(u64);
```

---

## Public API Surface

### Document — Construction and Lifecycle

```rust
impl Document {
    /// Create a new empty document.
    /// Addresses: Requirement 4 AC 7
    pub fn new() -> Self;

    /// Create a document with pre-allocated buffer capacity.
    pub fn with_capacity(capacity: u64) -> Self;

    /// Get the VFS URI this document was loaded from.
    pub fn source_uri(&self) -> Option<&ResourceUri>;

    /// Get the current loading progress.
    /// Addresses: Requirement 4 AC 4
    pub fn loading_progress(&self) -> &LoadingProgress;

    /// Register a document watcher. Returns a handle for removal.
    /// Addresses: Requirement 6 AC 5, Requirement 7 AC 7
    pub fn add_watcher(&mut self, watcher: Box<dyn DocumentWatcher>)
        -> Result<WatcherHandle, DocumentError>;

    /// Remove a previously registered watcher.
    /// Addresses: Requirement 6 AC 6
    pub fn remove_watcher(&mut self, handle: WatcherHandle) -> Result<(), DocumentError>;
}
```

### Document — Text Access

```rust
impl Document {
    /// Total byte length of document content.
    pub fn length(&self) -> u64;

    /// Total number of lines (minimum 1).
    pub fn line_count(&self) -> u64;

    /// Get byte at position.
    pub fn char_at(&self, position: BytePosition) -> Option<u8>;

    /// Get a range of bytes.
    pub fn get_range(&self, position: BytePosition, length: u64) -> Option<Vec<u8>>;

    /// Get contiguous view (compacts gap).
    pub fn contiguous_view(&mut self) -> &[u8];

    /// Get split view (no compaction).
    pub fn split_view(&self) -> SplitView<'_>;

    /// Get line start position.
    pub fn line_start(&self, line: LineNumber) -> BytePosition;

    /// Get line end position.
    pub fn line_end(&self, line: LineNumber) -> BytePosition;

    /// Find line from byte position.
    pub fn line_from_position(&self, position: BytePosition) -> LineNumber;

    /// Check if text contains a line ending for current mode.
    /// Addresses: Requirement 5 AC 4
    pub fn contains_line_end(&self, text: &[u8]) -> bool;
}
```

### Document — Mutation

```rust
impl Document {
    /// Insert text at position. Notifies watchers and returns result.
    /// Addresses: Requirement 2 AC 1, AC 7
    pub fn insert(&mut self, position: BytePosition, text: &[u8])
        -> Result<InsertResult, DocumentError>;

    /// Delete bytes at position. Notifies watchers and returns result.
    /// Addresses: Requirement 2 AC 2, AC 7
    pub fn delete(&mut self, position: BytePosition, length: u64)
        -> Result<DeleteResult, DocumentError>;

    /// Set read-only mode.
    pub fn set_read_only(&mut self, read_only: bool);

    /// Query read-only state.
    pub fn is_read_only(&self) -> bool;

    /// Set line-end recognition mode.
    /// Addresses: Requirement 5 AC 2
    pub fn set_line_end_mode(&mut self, mode: LineEndMode);

    /// Get current line-end mode.
    pub fn line_end_mode(&self) -> LineEndMode;
}
```

### Document — Character Navigation

```rust
impl Document {
    /// Get the byte length of the character at position.
    /// Addresses: Requirement 8 AC 1
    pub fn char_length_at(&self, position: BytePosition) -> u8;

    /// Move position outside a multi-byte sequence to nearest boundary.
    /// Addresses: Requirement 8 AC 2
    pub fn move_position_outside_char(
        &self,
        position: BytePosition,
        direction: Direction,
    ) -> BytePosition;

    /// Advance to next valid character position.
    /// Addresses: Requirement 8 AC 3
    pub fn next_position(
        &self,
        position: BytePosition,
        direction: Direction,
    ) -> Option<BytePosition>;

    /// Extract the character at position.
    /// Addresses: Requirement 8 AC 4
    pub fn character_at(&self, position: BytePosition) -> Option<CharacterExtracted>;

    /// Extract the character before position.
    /// Addresses: Requirement 8 AC 5
    pub fn character_before(&self, position: BytePosition) -> Option<CharacterExtracted>;

    /// Move by character offset from start position.
    /// Addresses: Requirement 8 AC 6
    pub fn relative_position(
        &self,
        start: BytePosition,
        character_offset: i64,
    ) -> Option<BytePosition>;
}
```

### Document — Viewport Management

```rust
impl Document {
    /// Get the current top-line (1-based).
    /// Addresses: Requirement 9 AC 1
    pub fn top_line(&self) -> u64;

    /// Scroll down by `visible_count` lines (page down).
    /// Addresses: Requirement 9 AC 2
    pub fn scroll_page_down(&mut self, visible_count: u64);

    /// Scroll up by `visible_count` lines (page up).
    /// Addresses: Requirement 9 AC 3
    pub fn scroll_page_up(&mut self, visible_count: u64);

    /// Scroll down by `count` lines.
    /// Addresses: Requirement 9 AC 4
    pub fn scroll_line_down(&mut self, count: u64);

    /// Scroll up by `count` lines.
    /// Addresses: Requirement 9 AC 5
    pub fn scroll_line_up(&mut self, count: u64);

    /// Set top_line to a specific value, clamped.
    /// Addresses: Requirement 9 AC 6
    pub fn set_top_line(&mut self, line: u64);

    /// Maximum valid top_line for a given viewport height.
    /// Addresses: Requirement 9 AC 7
    pub fn max_top_line(&self, visible_count: u64) -> u64;
}
```

### Document — Save Point

```rust
impl Document {
    /// Record the current undo position as the save point.
    /// Addresses: Requirement 10 AC 2
    pub fn set_save_point(&mut self);

    /// Check if at save point (no unsaved modifications).
    /// Addresses: Requirement 10 AC 3
    pub fn is_at_save_point(&self) -> bool;
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-document-model crate.
/// Formatted per Error Message Standards (Req 8): `[document] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DocumentError {
    /// Attempted mutation on a read-only document.
    #[error("[document] {operation}: document is read-only")]
    ReadOnly {
        operation: String,
    },

    /// Byte position is out of valid range.
    #[error("[document] {operation}: position {position} out of range (length: {length})")]
    PositionOutOfRange {
        operation: String,
        position: u64,
        length: u64,
    },

    /// Line number is out of valid range.
    #[error("[document] {operation}: line {line} out of range (total: {total})")]
    LineOutOfRange {
        operation: String,
        line: u64,
        total: u64,
    },

    /// VFS I/O error during streaming load or save.
    #[error("[document] {operation}: VFS error for {uri}: {source}")]
    VfsIo {
        operation: String,
        uri: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Streaming load was cancelled.
    #[error("[document] load: cancelled after {bytes_loaded} bytes")]
    LoadCancelled {
        bytes_loaded: u64,
    },

    /// Watcher already registered (duplicate).
    #[error("[document] add_watcher: watcher is already registered")]
    DuplicateWatcher,

    /// Watcher handle not found for removal.
    #[error("[document] remove_watcher: handle {handle_id} not found")]
    WatcherNotFound {
        handle_id: u64,
    },

    /// Document is still loading; operation not available.
    #[error("[document] {operation}: document is still loading ({bytes_loaded} bytes loaded)")]
    StillLoading {
        operation: String,
        bytes_loaded: u64,
    },
}
```

---

## Integration Points

### With `ff-vfs` (Core Layer — upstream)

- **Dependency direction**: ff-document-model depends on ff-vfs
- **API consumed**: `Vfs::read_stream(&ResourceUri)` for streaming file loading; `ResourceUri` for document identity
- **Usage pattern**: `StreamingFileReader` calls `vfs.read_stream(uri)` to obtain a `Pin<Box<dyn AsyncRead + Send>>`, then reads chunks in a loop
- **FFW-ARCH-001 compliance**: ALL file I/O flows through the VFS — no `std::fs` or `tokio::fs` in this crate
- **Save operations**: Document model does NOT own save logic directly — `ff-file-operations` coordinates saves via VFS. The document model provides `contiguous_view()` or `split_view()` for content extraction during save

### With `ff-logging` (Foundation Layer — upstream)

- **Dependency direction**: ff-document-model depends on ff-logging
- **API consumed**: `log_info!`, `log_warn!`, `log_error!`, `log_debug!` macros
- **Usage**: Loading progress milestones logged at INFO; line-end mode changes at INFO; errors at ERROR; character navigation edge cases at DEBUG
- **Log prefix**: `[document]`

### With `ff-core` (Core Layer — peer)

- **Dependency direction**: ff-document-model uses the Tokio runtime managed by ff-core for async streaming loads
- **Integration**: Streaming file loads are spawned as tracked Tokio tasks via `TokioRuntime::spawn_tracked`. Cancellation tokens from ff-core provide cooperative shutdown
- **Event Bus**: Loading progress updates and document-changed signals are dispatched via the Event Bus

### With `ff-edit-operations` (Wave 4 — downstream)

- **Dependency direction**: ff-edit-operations depends on ff-document-model
- **API consumed**: `Document::insert()`, `Document::delete()`, `Document::char_at()`, character navigation methods
- **Integration**: Edit operations use the low-level insert/delete primitives. The edit-operations crate adds selection handling, multi-caret coordination, and command-framework integration

### With `ff-undo-redo-transactions` (Wave 4 — downstream)

- **Dependency direction**: ff-undo-redo-transactions depends on ff-document-model
- **API consumed**: `InsertResult`, `DeleteResult` for building undo records; `Document::set_save_point()`, `Document::is_at_save_point()` for save-point integration
- **Integration**: The undo system wraps document mutations in transaction records. The document model's `undo_position` counter is managed by the undo crate

### With `ff-display-line-mapping` (Wave 4 — downstream)

- **Dependency direction**: ff-display-line-mapping depends on ff-document-model
- **API consumed**: `LineIndex` lookups via `Document::line_start()`, `Document::line_end()`, `Document::line_count()`, watcher notifications for incremental updates
- **Integration**: Display-line-mapping subscribes as a `DocumentWatcher` to receive insert/delete notifications and update its display-line mapping incrementally

### With `ff-background-io` (Wave 8 — downstream)

- **Dependency direction**: ff-background-io depends on ff-document-model
- **Integration**: background-io wraps `StreamingFileReader` with progress reporting, task scheduling, and cancellation coordination
- **API consumed**: `StreamingFileReader`, `LoadingProgress`, `DocumentHandle`

### Dependency Direction Summary

```
ff-logging ← ff-document-model ← ff-edit-operations
              ff-document-model ← ff-undo-redo-transactions
              ff-document-model ← ff-display-line-mapping
              ff-document-model ← ff-background-io
              ff-document-model ← ff-file-operations
              ff-document-model → ff-vfs (streaming reads)
              ff-document-model → ff-logging (structured logging)
```

---

## Configuration

ff-document-model owns the `[document]` namespace in the workbench TOML configuration file.

### TOML Schema

```toml
[document]
# Streaming load chunk size in bytes.
# Range: 4096–1048576 (4 KB – 1 MB). Default: 65536 (64 KB)
stream_chunk_size = 65536

# Sparse line index checkpoint interval (lines between checkpoints).
# Range: 100–10000. Default: 1000
sparse_checkpoint_interval = 1000

# GapBuffer initial capacity in bytes.
# Range: 1024–104857600 (1 KB – 100 MB). Default: 65536 (64 KB)
initial_buffer_capacity = 65536

# GapBuffer growth factor when gap is exhausted.
# Range: 1.5–4.0. Default: 2.0
gap_growth_factor = 2.0

# Default line-end mode for new documents: "default" or "unicode"
# Default: "default"
line_end_mode = "default"
```

### Config Resolution Rules

| Setting | Absent | Invalid Value | Out of Range |
|---------|--------|---------------|--------------|
| `stream_chunk_size` | Default to 65536 | Default to 65536 + WARN log | Clamp to [4096–1048576] + WARN |
| `sparse_checkpoint_interval` | Default to 1000 | Default to 1000 + WARN log | Clamp to [100–10000] + WARN |
| `initial_buffer_capacity` | Default to 65536 | Default to 65536 + WARN log | Clamp to [1024–104857600] + WARN |
| `gap_growth_factor` | Default to 2.0 | Default to 2.0 + WARN log | Clamp to [1.5–4.0] + WARN |
| `line_end_mode` | Default to "default" | Default to "default" + WARN log | N/A |

---

## Large-File Support Strategy

The document model supports files of arbitrary size through a layered approach:

### Tier 1: Standard Files (< 10 MB)

- Loaded fully into the GapBuffer via streaming reader
- Full LineIndex built during load
- All operations (insert, delete, navigation) available immediately after load

### Tier 2: Large Files (10 MB – 2 GB)

- Loaded via streaming reader with configurable chunk size
- Progressive display: consumers can read already-loaded content while loading continues
- SparseLineIndex provides approximate navigation during load; finalized on completion
- GapBuffer pre-allocated based on VFS metadata `size` hint to reduce reallocations

### Tier 3: Very Large Files (> 2 GB)

- GapBuffer uses `u64` addressing — no 32-bit overflow
- Streaming reader processes chunks without holding entire file in working memory during load
- After load completes, full content is in the GapBuffer (memory-mapped alternatives deferred to `ff-large-file-performance`)
- For files exceeding available RAM, the `ff-large-file-performance` crate (Wave 15) will provide chunked/paged buffer strategies that override the default GapBuffer — this is a future extension point

### Design Decision: Gap Buffer vs. Rope vs. Piece Table

**Chosen: Gap Buffer** (matching Scintilla's proven approach)

Rationale:
1. **Simplicity**: Gap buffer is straightforward to implement correctly in Rust with safe abstractions
2. **Cache locality**: Contiguous memory layout provides excellent cache behaviour for sequential reads
3. **Proven at scale**: Scintilla uses this approach for files up to multiple GB successfully
4. **Edit locality**: Real editing is highly localised (cursor position); gap buffer exploits this with O(1) amortized edits
5. **Split view**: Two-segment view enables efficient read-only iteration without gap movement

Trade-offs accepted:
- O(n) gap movement when edit position jumps — acceptable because real edits cluster
- Full content in memory after load — very large file paging deferred to Wave 15
- Rope or piece table would offer O(log n) random inserts — not needed for cursor-driven editing

---

## Correctness Properties

The following properties are suitable for property-based testing with the `proptest` crate. Each property is universal — it must hold for all valid inputs.

### Property 1: GapBuffer Insert-Delete Round-Trip

**Statement:** For any valid position and text, inserting text then deleting the same range restores the original buffer content.

```
∀ buffer B, ∀ position P ∈ [0, B.length()], ∀ text T:
    let original = B.contiguous_view().to_vec();
    B.insert(P, T);
    B.delete(P, T.len());
    B.contiguous_view() == original
```

**Validates: Requirements 1.1, 2.1, 2.2**

### Property 2: GapBuffer Length Invariant

**Statement:** After any sequence of insertions and deletions, the buffer length equals the sum of all inserted bytes minus the sum of all deleted bytes (starting from initial length).

```
∀ operations [op₁, op₂, ..., opₙ]:
    final_length == initial_length + Σ(insert_lengths) - Σ(delete_lengths)
```

**Validates: Requirements 1.9**

### Property 3: Line Index Consistency — Insert

**Statement:** After inserting text containing N line endings, the line count increases by exactly N.

```
∀ buffer B, ∀ position P, ∀ text T:
    let N = count_line_endings(T, B.line_end_mode());
    let old_count = B.line_count();
    B.insert(P, T);
    B.line_count() == old_count + N
```

**Validates: Requirements 2.3, 3.2**

### Property 4: Line Index Consistency — Delete

**Statement:** After deleting a range containing N line endings, the line count decreases by exactly N (accounting for CRLF merge/split adjustments).

```
∀ buffer B, ∀ position P, ∀ length L where P+L ≤ B.length():
    let deleted_text = B.get_range(P, L);
    let N = count_line_endings(deleted_text, B.line_end_mode()) - crlf_adjustment;
    let old_count = B.line_count();
    B.delete(P, L);
    B.line_count() == old_count - N
```

**Validates: Requirements 2.4, 3.2**

### Property 5: Line Position Round-Trip

**Statement:** For any valid line number L, `line_from_position(line_start(L)) == L`.

```
∀ buffer B, ∀ L ∈ [0, B.line_count()):
    B.line_from_position(B.line_start(L)) == L
```

**Validates: Requirements 3.3, 3.6**

### Property 6: Character Navigation Never Lands Inside Multi-Byte Sequence

**Statement:** For any position returned by `next_position`, the position is always at a valid UTF-8 character boundary and never between a CR and its following LF.

```
∀ buffer B, ∀ position P, ∀ direction D:
    let next = B.next_position(P, D);
    if let Some(pos) = next {
        is_valid_char_boundary(B, pos) == true
        ∧ ¬is_between_crlf(B, pos)
    }
```

**Validates: Requirements 8.3, 8.7**

### Property 7: Viewport Scroll Clamping

**Statement:** After any scroll operation, top_line is always in the valid range [1, max_top_line(visible_count)].

```
∀ document D, ∀ scroll_operation, ∀ visible_count > 0:
    apply(scroll_operation, D, visible_count);
    D.top_line() >= 1 ∧ D.top_line() <= D.max_top_line(visible_count)
```

**Validates: Requirements 9.2, 9.3, 9.4, 9.5, 9.6, 9.8**

### Property 8: Scroll Idempotence at Boundaries

**Statement:** Calling scroll_page_up when already at line 1 has no effect; calling scroll_page_down when already at max has no effect.

```
∀ document D where D.top_line() == 1, ∀ visible_count:
    D.scroll_page_up(visible_count);
    D.top_line() == 1

∀ document D where D.top_line() == D.max_top_line(vc), ∀ vc:
    D.scroll_page_down(vc);
    D.top_line() == D.max_top_line(vc)
```

**Validates: Requirements 9.8**

### Property 9: Save-Point State Transitions

**Statement:** After `set_save_point()`, `is_at_save_point()` returns true. After any mutation, `is_at_save_point()` returns false.

```
∀ document D:
    D.set_save_point();
    D.is_at_save_point() == true;
    D.insert(some_position, some_text);
    D.is_at_save_point() == false
```

**Validates: Requirements 10.2, 10.3, 10.4**

### Property 10: SplitView Content Equivalence

**Statement:** The content accessible via `split_view()` is byte-for-byte identical to the content returned by `contiguous_view()`, regardless of gap position.

```
∀ buffer B:
    let split = B.split_view();
    let contiguous = B.contiguous_view();
    concat(split.before_gap, split.after_gap) == contiguous
```

**Validates: Requirements 1.6, 1.7**

### Property 11: Line End Mode Change Preserves Content

**Statement:** Changing the line-end mode rebuilds the LineIndex but never modifies the buffer content.

```
∀ document D, ∀ mode M:
    let content_before = D.contiguous_view().to_vec();
    D.set_line_end_mode(M);
    D.contiguous_view() == content_before
```

**Validates: Requirements 5.2**

### Property 12: Watcher Notification Count

**Statement:** Every insert operation notifies exactly all registered watchers once (and only once).

```
∀ document D with N watchers, ∀ insert operation:
    D.insert(pos, text);
    each_watcher_received_exactly_one_notify_insert == true
    total_notifications == N
```

**Validates: Requirements 7.2, 7.6**

---

## Testing Strategy

### Unit Tests

- `gap_buffer_tests.rs`: Insert at various positions, delete, length tracking, growth, split_view, contiguous_view
- `text_buffer_tests.rs`: Insert/delete with line tracking, read-only guard, CRLF merge/split
- `line_index_tests.rs`: Lookup correctness, rebuild, character-count index allocation/release
- `streaming_tests.rs`: Mock VFS read_stream, progressive loading, cancellation, error handling
- `navigation_tests.rs`: UTF-8 boundary detection, CRLF atomic navigation, invalid byte handling
- `viewport_tests.rs`: Scroll clamping at all boundaries, max_top_line computation
- `watcher_tests.rs`: Registration, notification delivery, duplicate rejection, removal
- `save_point_tests.rs`: State transitions, notification dispatch

### Property-Based Tests (proptest)

- GapBuffer insert-delete round-trip (Property 1)
- GapBuffer length invariant (Property 2)
- Line index insert consistency (Property 3)
- Line index delete consistency (Property 4)
- Line position round-trip (Property 5)
- Character navigation boundary safety (Property 6)
- Viewport scroll clamping (Property 7)
- Scroll idempotence at boundaries (Property 8)
- Save-point state transitions (Property 9)
- SplitView content equivalence (Property 10)
- Line-end mode change content preservation (Property 11)
- Watcher notification count (Property 12)

### Integration Tests

- End-to-end: open file via mock VFS → progressive load → full line index → insert/delete → verify
- Multi-handle: create DocumentHandle, clone to two readers, verify shared state
- Large-file simulation: stream 100 MB of generated content, verify line count and navigation

### Test Infrastructure

- **Mock VFS**: An in-memory VFS provider for deterministic streaming tests
- **Testing framework**: `proptest` for property-based tests, `#[tokio::test]` for async tests
- **Minimum proptest iterations**: 100 per property
- **Fixtures**: Pre-built text samples with known line counts, encoding edge cases, and CRLF variants
