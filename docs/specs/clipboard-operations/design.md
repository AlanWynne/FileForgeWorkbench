# Design Document: Clipboard Operations (`ff-clipboard`)

## Overview

The `ff-clipboard` crate is the **unified clipboard subsystem** for FileForgeWorkbench. It provides platform-independent system clipboard access, standard desktop clipboard operations (copy/cut/paste), ISPF COPY command routing (clipboard-paste, file-insert, shell-capture), and advanced clipboard behaviours for rectangular selections and multi-caret editing.

### Purpose

- Abstract platform-specific clipboard APIs behind a testable trait (`ClipboardProvider`)
- Implement copy/cut/paste operations with mode-aware behaviour (Stream, Line, Rectangular)
- Route the COPY primary command between in-document copy, clipboard-paste, file-insert, and shell-capture modes
- Handle multi-caret clipboard distribution (segment-per-caret matching)
- Manage rectangular clipboard content with column-block paste semantics
- Provide line-copy-when-no-selection behaviour with configurable opt-out
- Integrate with undo/redo via single-transaction recording for all paste/cut operations
- Maintain a clipboard history ring for recent clipboard entries

### Position in Architecture

```
Wave 9 — Desktop Integration

┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│     Context menu rendering, shortcut detection               │
├─────────────────────────────────────────────────────────────┤
│         ff-clipboard (THIS CRATE — Wave 9)                   │
│   Clipboard engine, copy/cut/paste, COPY command routing,    │
│   rectangular paste, multi-caret distribution                │
├─────────────────────────────────────────────────────────────┤
│  ff-edit-operations (Wave 4) — selection model, edit engine  │
│  ff-document-model (Wave 4) — document buffer access         │
│  ff-undo-redo (Wave 4) — transaction recording               │
│  ff-command (Wave 2) — command dispatch, shortcut registry   │
│  ff-line-commands (Wave 5) — pending C/CC/A/B state          │
│  ff-vfs (Wave 3) — file-insert mode file reading             │
│  ff-config (Wave 2) — clipboard configuration keys           │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging (Wave 0)            │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Clipboard access abstracted via `ClipboardProvider` trait; no platform-specific code in core logic
- **Command-Driven (Req 4)**: All operations registered as commands (`clipboard.copy`, `clipboard.cut`, `clipboard.paste`, `clipboard.copy-command`)
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-clipboard`
- **Error Message Standards (Req 8)**: All errors follow `[clipboard] operation: description` format
- **Async I/O (Req 6)**: File-insert mode uses async VFS read; clipboard access has configurable timeout

### Upstream Dependencies

| Crate | Usage |
|-------|-------|
| `ff-command` | Command registration, dispatch, `CommandHandler` trait, `CommandId`, `CommandParams` |
| `ff-edit-operations` | `SelectionContainer`, `SelectionRange`, `SelectionPosition`, `InsertionEngine`, `DeletionEngine`, `MultiCaretCoordinator` |
| `ff-document-model` | `Document`, `TextBuffer`, `LineIndex` for content access and insertion |
| `ff-undo-redo` | `UndoManager` for recording paste/cut operations as `UndoRecord` |
| `ff-line-commands` | `PendingCommandStore` for C/CC/A/B target state queries |
| `ff-vfs` | `VfsProvider::read_to_string` for file-insert mode |
| `ff-config` | `ConfigProvider` for clipboard-related settings |
| `ff-logging` | Structured diagnostics |


---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Invocation Sources"
        KS[Keyboard Shortcut<br/>Ctrl+C/X/V]
        CM[Context Menu<br/>Cut/Copy/Paste]
        PRI[Primary Command Line<br/>COPY / SHELL]
        LUA[Lua Macro Script]
    end

    subgraph "ff-clipboard"
        CP[ClipboardProvider Trait<br/>platform abstraction]
        CE[ClipboardEngine<br/>read/write/availability]
        ENTRY[ClipboardEntry<br/>text + mode + segments]
        HIST[ClipboardHistoryRing<br/>bounded recent entries]
        COPY_OP[CopyHandler<br/>stream/rect/multi/line]
        CUT_OP[CutHandler<br/>stream/rect/multi/line]
        PASTE_OP[PasteHandler<br/>mode-aware insertion]
        ROUTER[CopyCommandRouter<br/>disambiguation logic]
        FI[FileInsertHandler<br/>VFS file → lines]
        SPLIT[LineSplitter<br/>LF/CRLF/CR → LogicalLines]
        CFG_R[ConfigReader<br/>clipboard.* keys]
    end

    subgraph "Downstream / Upstream"
        SEL[ff-edit-operations<br/>SelectionContainer]
        DOC[ff-document-model<br/>Document, TextBuffer]
        UNDO[ff-undo-redo<br/>UndoManager]
        CMD[ff-command<br/>CommandRegistry]
        LC[ff-line-commands<br/>PendingCommandStore]
        VFS[ff-vfs<br/>VfsProvider]
        CONFIG[ff-config<br/>ConfigProvider]
        LOG[ff-logging]
    end

    KS --> CMD
    CM --> CMD
    PRI --> CMD
    LUA --> CMD
    CMD --> COPY_OP
    CMD --> CUT_OP
    CMD --> PASTE_OP
    CMD --> ROUTER

    CE --> CP
    COPY_OP --> CE
    COPY_OP --> SEL
    COPY_OP --> ENTRY
    COPY_OP --> HIST
    CUT_OP --> CE
    CUT_OP --> SEL
    CUT_OP --> DOC
    CUT_OP --> UNDO
    PASTE_OP --> CE
    PASTE_OP --> ENTRY
    PASTE_OP --> SPLIT
    PASTE_OP --> DOC
    PASTE_OP --> UNDO
    ROUTER --> LC
    ROUTER --> PASTE_OP
    ROUTER --> FI
    FI --> VFS
    FI --> SPLIT
    FI --> DOC
    FI --> UNDO
    CFG_R --> CONFIG
    CE --> LOG
end
```


### Layer Placement

| Layer | Role |
|-------|------|
| **Command Layer** | Command handlers for `clipboard.copy`, `clipboard.cut`, `clipboard.paste`, `clipboard.copy-command` — translate shortcut/menu invocation into engine calls |
| **Routing Layer** | `CopyCommandRouter` — disambiguates COPY primary command into in-document, clipboard-paste, file-insert, or shell-capture mode |
| **Engine Layer** | `CopyHandler`, `CutHandler`, `PasteHandler` — core clipboard operation logic per mode |
| **Clipboard Access Layer** | `ClipboardEngine` + `ClipboardProvider` trait — platform-independent read/write/timeout |
| **Content Layer** | `ClipboardEntry`, `LineSplitter`, `ClipboardHistoryRing` — structured content representation and splitting |
| **Integration Layer** | Bridges to undo/redo, document model, selection container, pending line commands, VFS, config |

---

## Components and Interfaces

```
crates/ff-clipboard/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── provider.rs             # ClipboardProvider trait definition
│   ├── engine.rs               # ClipboardEngine — read/write/detect availability
│   ├── entry.rs                # ClipboardEntry, ClipboardMode, per-segment storage
│   ├── history.rs              # ClipboardHistoryRing — bounded ring buffer
│   ├── splitter.rs             # LineSplitter — LF/CRLF/CR splitting logic
│   ├── copy.rs                 # CopyHandler — stream/rect/multi-caret/line-copy
│   ├── cut.rs                  # CutHandler — stream/rect/multi-caret/line-cut
│   ├── paste.rs                # PasteHandler — mode-aware paste (stream/line/rect/multi)
│   ├── router.rs               # CopyCommandRouter — COPY primary command disambiguation
│   ├── file_insert.rs          # FileInsertHandler — VFS read + line insertion
│   ├── config.rs               # ClipboardConfig — typed config access
│   ├── commands/
│   │   ├── mod.rs              # Re-exports for all command handlers
│   │   ├── copy_cmd.rs         # clipboard.copy command handler
│   │   ├── cut_cmd.rs          # clipboard.cut command handler
│   │   ├── paste_cmd.rs        # clipboard.paste command handler
│   │   └── copy_primary.rs     # clipboard.copy-command (COPY primary) handler
│   ├── error.rs                # ClipboardError enum
│   └── context_menu.rs         # Context menu enablement logic
└── tests/
    ├── provider_tests.rs       # ClipboardProvider trait contract tests
    ├── entry_tests.rs          # ClipboardEntry construction property tests
    ├── splitter_tests.rs       # LineSplitter property tests
    ├── copy_tests.rs           # Copy operation property tests
    ├── cut_tests.rs            # Cut operation property tests
    ├── paste_tests.rs          # Paste operation (all modes) property tests
    ├── router_tests.rs         # COPY disambiguation property tests
    ├── file_insert_tests.rs    # File-insert mode property tests
    ├── history_tests.rs        # History ring property tests
    ├── multi_caret_tests.rs    # Multi-caret distribution property tests
    ├── rectangular_tests.rs    # Rectangular clipboard property tests
    └── integration.rs          # End-to-end clipboard scenarios
```


---

## Data Models

### ClipboardMode

```rust
/// Indicates how clipboard content was acquired, affecting paste behaviour.
/// Addresses: Requirements 1.4, 2.1–2.4, 4.1–4.3, 14.1–14.4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClipboardMode {
    /// Normal character-stream selection copy. Paste inserts inline at caret.
    Stream,
    /// Full-line copy (no selection active). Paste inserts as new line(s) above caret line.
    Line,
    /// Rectangular (column) selection copy. Paste inserts as column block.
    Rectangular,
}

impl Default for ClipboardMode {
    fn default() -> Self {
        ClipboardMode::Stream
    }
}
```

### ClipboardEntry

```rust
/// A structured clipboard content unit with mode and optional per-segment storage.
/// Addresses: Requirements 1.4, 2.2–2.3, 12.1, 13.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEntry {
    /// The full text content written to/read from the system clipboard.
    text: String,
    /// How the content was acquired — determines paste semantics.
    mode: ClipboardMode,
    /// Independent line segments for Rectangular or Multi-Caret modes.
    /// Empty for Stream/Line modes (text is used directly).
    segments: Vec<String>,
    /// Timestamp of when this entry was created (for history ordering).
    created_at: std::time::Instant,
}

impl ClipboardEntry {
    pub fn stream(text: String) -> Self;
    pub fn line(text: String) -> Self;
    pub fn rectangular(segments: Vec<String>) -> Self;
    pub fn multi_caret(segments: Vec<String>) -> Self;
    pub fn text(&self) -> &str;
    pub fn mode(&self) -> ClipboardMode;
    pub fn segments(&self) -> &[String];
    pub fn segment_count(&self) -> usize;
}
```

### ClipboardHistoryRing

```rust
/// Bounded ring buffer of recent clipboard entries.
/// Provides clipboard history navigation (paste-from-history).
/// Addresses: clipboard history ring requirement
#[derive(Debug)]
pub struct ClipboardHistoryRing {
    entries: VecDeque<ClipboardEntry>,
    capacity: usize,
}

impl ClipboardHistoryRing {
    pub fn new(capacity: usize) -> Self;
    pub fn push(&mut self, entry: ClipboardEntry);
    pub fn latest(&self) -> Option<&ClipboardEntry>;
    pub fn iter(&self) -> impl Iterator<Item = &ClipboardEntry>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn clear(&mut self);
}
```


### LineSplitResult

```rust
/// Result of splitting clipboard or file text into logical lines.
/// Addresses: Requirements 4.6–4.8, 9.9–9.10, 16.1–16.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineSplitResult {
    /// Individual logical lines after splitting. Content is preserved exactly.
    pub lines: Vec<String>,
    /// Whether the source text ended with a trailing line terminator
    /// (used to suppress empty trailing line creation).
    pub had_trailing_terminator: bool,
}
```

### CopyCommandMode

```rust
/// The resolved mode of the COPY primary command after disambiguation.
/// Addresses: Requirement 8 (8.1–8.8)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyCommandMode {
    /// In-document copy: C/CC source + A/B target (delegates to ff-line-commands).
    InDocument,
    /// Clipboard paste: no args, no source, A/B target present.
    ClipboardPaste,
    /// File insert: path argument + A/B target, no source.
    FileInsert { path: String },
    /// Shell capture: SHELL command + A/B target (delegates to ff-shell-command).
    ShellCapture { command: String },
}
```

### ClipboardConfig

```rust
/// Typed configuration for clipboard behaviour.
/// Addresses: Requirement 19 (19.1–19.4)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardConfig {
    /// Whether Ctrl+C with no selection copies the entire line.
    pub line_copy_when_no_selection: bool,    // default: true
    /// Whether rectangular paste creates new lines beyond document end.
    pub rectangular_paste_adds_lines: bool,   // default: true
    /// Timeout in milliseconds for clipboard access operations.
    pub access_timeout_ms: u32,              // default: 500
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            line_copy_when_no_selection: true,
            rectangular_paste_adds_lines: true,
            access_timeout_ms: 500,
        }
    }
}
```


---

## Public API Surface

### ClipboardProvider Trait

```rust
/// Platform-independent clipboard access abstraction.
/// Implementors wrap OS-specific clipboard APIs (Win32, X11/Wayland, NSPasteboard).
/// GUI shells provide the concrete implementation; tests use a mock.
/// Addresses: Requirements 1.1, 1.6, 1.7
pub trait ClipboardProvider: Send + Sync {
    /// Write plain UTF-8 text to the system clipboard.
    fn write_text(&self, text: &str) -> Result<(), ClipboardError>;

    /// Read plain text from the system clipboard.
    /// Returns `Err(ClipboardError::NoTextContent)` if clipboard holds non-text data.
    /// Returns `Err(ClipboardError::Empty)` if clipboard is empty.
    fn read_text(&self) -> Result<String, ClipboardError>;

    /// Check whether the clipboard currently contains text content.
    fn has_text(&self) -> Result<bool, ClipboardError>;

    /// Check whether the clipboard is accessible (permissions, platform availability).
    fn is_available(&self) -> bool;
}
```

### ClipboardEngine

```rust
/// Orchestrates clipboard read/write with structured ClipboardEntry metadata.
/// Stores the last-written entry locally to detect internal vs external clipboard content.
/// Addresses: Requirements 1.2–1.5, 6.1–6.4, 19.3
pub struct ClipboardEngine {
    provider: Box<dyn ClipboardProvider>,
    last_written: Option<ClipboardEntry>,
    history: ClipboardHistoryRing,
    config: ClipboardConfig,
}

impl ClipboardEngine {
    pub fn new(provider: Box<dyn ClipboardProvider>, config: ClipboardConfig) -> Self;

    /// Write a ClipboardEntry to the system clipboard and record in history.
    pub fn write(&mut self, entry: ClipboardEntry) -> Result<(), ClipboardError>;

    /// Read from system clipboard, returning a structured ClipboardEntry.
    /// If the system clipboard text matches our last write, returns the
    /// original entry with its mode. Otherwise returns Stream mode.
    pub fn read(&self) -> Result<ClipboardEntry, ClipboardError>;

    /// Check if system clipboard has text content available for paste.
    pub fn has_content(&self) -> Result<bool, ClipboardError>;

    /// Access the clipboard history ring.
    pub fn history(&self) -> &ClipboardHistoryRing;

    /// Update configuration (e.g., after hot-reload).
    pub fn update_config(&mut self, config: ClipboardConfig);
}
```


### CopyHandler

```rust
/// Implements copy operations for all selection types.
/// Does NOT modify the document. Writes to ClipboardEngine.
/// Addresses: Requirements 2, 12.1, 13.1, 14.1
pub struct CopyHandler;

impl CopyHandler {
    /// Copy active stream selection text to clipboard.
    pub fn copy_stream(
        engine: &mut ClipboardEngine,
        selection: &SelectionRange,
        document: &Document,
    ) -> Result<(), ClipboardError>;

    /// Copy rectangular selection as per-line segments.
    pub fn copy_rectangular(
        engine: &mut ClipboardEngine,
        selection: &RectangularSelection,
        document: &Document,
    ) -> Result<(), ClipboardError>;

    /// Copy multi-caret selections as independent segments.
    pub fn copy_multi_caret(
        engine: &mut ClipboardEngine,
        selections: &[SelectionRange],
        document: &Document,
    ) -> Result<(), ClipboardError>;

    /// Copy entire current line (line-copy-when-no-selection mode).
    pub fn copy_line(
        engine: &mut ClipboardEngine,
        line_number: u64,
        document: &Document,
    ) -> Result<(), ClipboardError>;
}
```

### CutHandler

```rust
/// Implements cut operations — copies to clipboard then deletes from document.
/// Records a single UndoRecord for the combined operation.
/// Addresses: Requirements 3, 12.1, 13.3, 14.4
pub struct CutHandler;

impl CutHandler {
    /// Cut active stream selection.
    pub fn cut_stream(
        engine: &mut ClipboardEngine,
        selection: &SelectionRange,
        document: &mut Document,
        undo: &mut dyn UndoManager,
    ) -> Result<CutResult, ClipboardError>;

    /// Cut rectangular selection.
    pub fn cut_rectangular(
        engine: &mut ClipboardEngine,
        selection: &RectangularSelection,
        document: &mut Document,
        undo: &mut dyn UndoManager,
    ) -> Result<CutResult, ClipboardError>;

    /// Cut multi-caret selections simultaneously.
    pub fn cut_multi_caret(
        engine: &mut ClipboardEngine,
        selections: &[SelectionRange],
        document: &mut Document,
        undo: &mut dyn UndoManager,
    ) -> Result<CutResult, ClipboardError>;

    /// Cut entire current line (line-cut-when-no-selection).
    pub fn cut_line(
        engine: &mut ClipboardEngine,
        line_number: u64,
        document: &mut Document,
        undo: &mut dyn UndoManager,
    ) -> Result<CutResult, ClipboardError>;
}

/// Result of a cut operation including the new caret position.
#[derive(Debug)]
pub struct CutResult {
    pub caret_position: SelectionPosition,
}
```


### PasteHandler

```rust
/// Implements paste operations with mode-aware insertion logic.
/// Addresses: Requirements 4, 12.2–12.5, 13.2–13.5, 14.2–14.3, 18.1–18.2
pub struct PasteHandler;

impl PasteHandler {
    /// Paste stream content at the caret, replacing any active selection.
    pub fn paste_stream(
        entry: &ClipboardEntry,
        caret: SelectionPosition,
        active_selection: Option<&SelectionRange>,
        document: &mut Document,
        undo: &mut dyn UndoManager,
    ) -> Result<PasteResult, ClipboardError>;

    /// Paste line-mode content as new lines above the caret line.
    pub fn paste_line(
        entry: &ClipboardEntry,
        caret_line: u64,
        document: &mut Document,
        undo: &mut dyn UndoManager,
    ) -> Result<PasteResult, ClipboardError>;

    /// Paste rectangular content as column block at caret position.
    pub fn paste_rectangular(
        entry: &ClipboardEntry,
        caret: SelectionPosition,
        active_selection: Option<&RectangularSelection>,
        document: &mut Document,
        config: &ClipboardConfig,
        undo: &mut dyn UndoManager,
    ) -> Result<PasteResult, ClipboardError>;

    /// Paste with multi-caret distribution logic.
    pub fn paste_multi_caret(
        entry: &ClipboardEntry,
        carets: &[SelectionPosition],
        active_selections: &[Option<SelectionRange>],
        document: &mut Document,
        undo: &mut dyn UndoManager,
    ) -> Result<PasteResult, ClipboardError>;
}

/// Result of a paste operation.
#[derive(Debug)]
pub struct PasteResult {
    /// New caret position(s) after paste.
    pub caret_positions: Vec<SelectionPosition>,
    /// Number of logical lines inserted (for status display).
    pub lines_inserted: usize,
}
```

### CopyCommandRouter

```rust
/// Disambiguates the COPY primary command into its four modes.
/// Addresses: Requirement 8 (8.1–8.8)
pub struct CopyCommandRouter;

impl CopyCommandRouter {
    /// Given the command arguments and current pending line-command state,
    /// determine which COPY mode should execute.
    ///
    /// Returns an error message if the combination is invalid.
    pub fn resolve(
        args: &str,
        pending_store: &PendingCommandStore,
    ) -> Result<CopyCommandMode, ClipboardError>;
}
```

### FileInsertHandler

```rust
/// Reads a file via VFS and inserts its content at a target line.
/// Addresses: Requirements 9, 10
pub struct FileInsertHandler;

impl FileInsertHandler {
    /// Read a file and insert its lines at the target position.
    pub async fn insert(
        path: &str,
        target_line: u64,
        target_position: TargetPosition,
        document: &mut Document,
        vfs: &dyn VfsProvider,
        undo: &mut dyn UndoManager,
    ) -> Result<FileInsertResult, ClipboardError>;
}

/// Whether insertion is before or after the target line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPosition {
    After,
    Before,
}

/// Result of a file-insert operation.
#[derive(Debug)]
pub struct FileInsertResult {
    pub lines_inserted: usize,
    pub resolved_path: String,
}
```


### LineSplitter

```rust
/// Splits text into logical lines handling all standard line-ending conventions.
/// Addresses: Requirements 4.6–4.8, 9.9–9.10, 16.1–16.5
pub struct LineSplitter;

impl LineSplitter {
    /// Split text on LF, CRLF, or CR boundaries.
    /// A trailing line terminator does NOT produce an empty final line.
    /// Content of each line is preserved without trimming.
    pub fn split(text: &str) -> LineSplitResult;

    /// Split and normalize line endings to the document's configured style.
    pub fn split_and_normalize(text: &str, target_ending: LineEnding) -> LineSplitResult;
}

/// Line ending style for normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}
```

---

## Error Handling

```rust
/// All errors produced by the clipboard subsystem.
/// Follows the `[clipboard] operation: description` format standard.
/// Addresses: Requirements 1.6, 6.1–6.5, 10.1–10.4
#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    /// System clipboard is empty.
    #[error("[clipboard] read: clipboard is empty")]
    Empty,

    /// System clipboard contains non-text content (image, binary, etc.).
    #[error("[clipboard] read: clipboard contains non-text content")]
    NoTextContent,

    /// System clipboard cannot be accessed (permissions, platform error).
    #[error("[clipboard] access: clipboard unavailable — {reason}")]
    Unavailable { reason: String },

    /// Clipboard access timed out.
    #[error("[clipboard] access: operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u32 },

    /// Write to system clipboard failed.
    #[error("[clipboard] write: failed to write to clipboard — {reason}")]
    WriteFailed { reason: String },

    /// File not found for file-insert mode.
    #[error("[clipboard] file-insert: file not found — {path}")]
    FileNotFound { path: String },

    /// File access permission error for file-insert mode.
    #[error("[clipboard] file-insert: access denied — {path}")]
    FileAccessDenied { path: String },

    /// File is binary/non-text for file-insert mode.
    #[error("[clipboard] file-insert: file is not plain text — {path}")]
    FileBinary { path: String },

    /// File I/O error for file-insert mode.
    #[error("[clipboard] file-insert: I/O error reading {path} — {source}")]
    FileIo { path: String, source: std::io::Error },

    /// COPY command requires an A or B target line command.
    #[error("[clipboard] COPY: target line command A or B is required")]
    NoTarget,

    /// COPY command has conflicting source line commands with file path.
    #[error("[clipboard] COPY: source line commands cannot be combined with a file path argument")]
    ConflictingSourceAndPath,

    /// COPY command is incomplete — source pending but no target.
    #[error("[clipboard] COPY: pending source commands require a target (A or B)")]
    IncompleteSourceTarget,

    /// Configuration value is invalid (logged as warning, fallback applied).
    #[error("[clipboard] config: invalid value for {key}, using default")]
    InvalidConfig { key: String },
}
```


---

## Integration Points

### 7.1 Command Framework (`ff-command`)

| Command ID | Default Shortcut | Handler | Description |
|-----------|-----------------|---------|-------------|
| `clipboard.copy` | Ctrl+C | `commands::copy_cmd` | Copy selection or current line to clipboard |
| `clipboard.cut` | Ctrl+X | `commands::cut_cmd` | Cut selection or current line to clipboard |
| `clipboard.paste` | Ctrl+V | `commands::paste_cmd` | Paste from clipboard at caret |
| `clipboard.copy-command` | *(none — primary command)* | `commands::copy_primary` | COPY primary command dispatcher |

All commands are registered via `CommandRegistry::register()` at crate initialization. Each command:
- Has a `CommandHandler` implementation with `execute(&self, ctx: &ExecutionContext) -> CommandResult`
- Returns `CommandResult::Success { undo_record }` for undoable operations (cut, paste)
- Returns `CommandResult::Success { undo_record: None }` for copy (no document change)
- Returns `CommandResult::Error { message }` on failure (clipboard unavailable, etc.)

Commands are logged in `CommandHistory` per Requirement 17.5.

### 7.2 Edit Operations (`ff-edit-operations`)

The clipboard crate consumes from `ff-edit-operations`:
- **`SelectionContainer`** — queries active selections (stream, rectangular, multi-caret)
- **`SelectionRange`** / **`SelectionPosition`** — position types for caret and anchor
- **`MultiCaretCoordinator`** — reverse-order processing for multi-caret paste
- **`InsertionEngine`** — text insertion primitives used by paste operations
- **`DeletionEngine`** — text deletion primitives used by cut operations

The clipboard crate does NOT own selection state — it queries the current selection from the `SelectionContainer` and uses edit-operations engines to perform insertions/deletions.

### 7.3 Document Model (`ff-document-model`)

The clipboard crate interacts with:
- **`Document`** — top-level document access for reading content and applying edits
- **`TextBuffer`** — raw content access for extracting selected text
- **`LineIndex`** — line number ↔ byte offset mapping for line-aware operations

All document mutations go through the edit-operations layer or through `Document::insert_lines()` / `Document::delete_range()` primitives.

### 7.4 Undo/Redo Transactions (`ff-undo-redo`)

Every cut and paste operation produces a single `UndoRecord`:
- **Cut**: records the deleted text, its original position, and the selection state before cut
- **Paste**: records the inserted text range so undo can remove it and restore pre-paste state
- **File-insert**: records inserted line range for removal on undo
- **COPY clipboard-paste**: records inserted line range for removal on undo

Multi-caret operations wrap all individual insertions/deletions into a single `UndoGroup` via `TransactionBuilder::begin_group()` / `end_group()`.

### 7.5 Line Commands (`ff-line-commands`)

The clipboard crate queries `PendingCommandStore` to:
- Check for pending `C`/`CC` source commands (determines COPY routing)
- Check for pending `A`/`B` target commands (determines insertion point)
- Clear resolved targets after successful clipboard-paste or file-insert

The in-document copy mode (C/CC + A/B) is NOT handled by this crate — the router detects this case and delegates to `ff-line-commands`.

### 7.6 Virtual File System (`ff-vfs`)

File-insert mode uses `VfsProvider::read_to_string(path)` to read file content:
- Relative paths are resolved against the current document's parent directory
- Absolute paths are used as-is
- The VFS may return errors for non-existent files, permission issues, or binary content detection

### 7.7 Configuration System (`ff-config`)

The clipboard crate reads the following configuration keys at initialization and on hot-reload:

| Key | Type | Default | Effect |
|-----|------|---------|--------|
| `clipboard.line_copy_when_no_selection` | `bool` | `true` | Controls line-copy-on-Ctrl+C behaviour |
| `clipboard.rectangular_paste_adds_lines` | `bool` | `true` | Controls whether rect paste extends document |
| `clipboard.access_timeout_ms` | `u32` | `500` | Timeout for system clipboard access |

Configuration is read via `ConfigProvider::get_bool()` / `ConfigProvider::get_u32()`. Invalid values trigger a warning log and fallback to defaults.


---

## Correctness Properties

The following properties are designed for verification with the `proptest` crate. Each property maps to one or more acceptance criteria from `requirements.md`.

### Property 1: Line Splitter Round-Trip Preservation

**Statement:** For any non-empty string `s`, joining `LineSplitter::split(s).lines` with the appropriate separator reconstructs the original content (minus a trailing terminator if present).

**Validates: Requirements 4.6, 4.7, 4.8, 16.1, 16.2, 16.3, 16.4**

**Strategy:** Generate arbitrary UTF-8 strings containing mixtures of `\n`, `\r\n`, `\r`, and non-newline characters.

---

### Property 2: Trailing Terminator Suppression

**Statement:** For any string `s` that ends with a line terminator (`\n`, `\r\n`, or `\r`), `LineSplitter::split(s).lines` does NOT contain an empty string as its final element solely due to that trailing terminator.

**Validates: Requirements 4.7, 16.3**

**Strategy:** Generate strings guaranteed to end with a line terminator.

---

### Property 3: Copy Does Not Modify Document

**Statement:** For any document state `D` and any valid selection `S` within `D`, executing a copy operation produces `ClipboardEntry` content equal to the text of `S` in `D`, and the document content after the operation is byte-identical to before.

**Validates: Requirements 2.5, 18.3**

**Strategy:** Generate random document content (1–100 lines) and valid selection ranges within it.

---

### Property 4: Cut Then Paste Restores Document

**Statement:** For any document state `D` and valid selection `S`, if cut is executed (producing clipboard content `C`) and then paste is executed at the same position with the same `C`, the resulting document is identical to `D`.

**Validates: Requirements 3.1, 4.1, 15.1, 15.2, 15.3, 15.4**

**Strategy:** Generate documents and valid stream selections; verify cut→paste round-trip.

---

### Property 5: COPY Command Disambiguation Is Total

**Statement:** For all combinations of (has_pending_source: bool, has_target: bool, has_path_arg: bool), `CopyCommandRouter::resolve` returns either a valid `CopyCommandMode` or a descriptive `ClipboardError` — it never panics.

**Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8**

**Strategy:** Enumerate the boolean product space (8 combinations) plus edge cases on path content.

---

### Property 6: Multi-Caret Segment Distribution

**Statement:** When pasting with `N` active carets and the clipboard contains exactly `N` segments, segment `i` is pasted at caret `i` (ordered by document position). When the counts differ, full content is pasted at each caret.

**Validates: Requirements 13.2, 13.3, 4.4, 4.5**

**Strategy:** Generate N in [1..8], clipboard with M segments (M = N and M ≠ N), verify distribution.

---

### Property 7: Rectangular Paste Column Alignment

**Statement:** For a rectangular `ClipboardEntry` with `K` segments each of width `W`, pasting at position `(line, col)` results in segment `i` appearing at column `col` on line `line + i` for all `i in 0..K`.

**Validates: Requirements 12.2, 12.3, 12.4**

**Strategy:** Generate rectangular entries (1–20 segments, width 1–40), paste positions, and verify column alignment in the resulting document.

---

### Property 8: Line-Copy Paste Inserts Above

**Statement:** When clipboard mode is `Line` and paste is executed, the pasted lines appear immediately above the caret line, and the original caret line's content is unchanged.

**Validates: Requirements 14.2, 14.3, 4.2**

**Strategy:** Generate documents, line-mode clipboard content (1–10 lines), caret positions; verify insertion point and original line preservation.

---

### Property 9: Clipboard Write/Read Consistency

**Statement:** For any `ClipboardEntry` written to the engine, immediately reading back (without external modification) returns an entry with identical text, mode, and segments.

**Validates: Requirements 1.2, 1.3, 1.4**

**Strategy:** Generate arbitrary `ClipboardEntry` values across all modes.

---

### Property 10: External Clipboard Defaults to Stream

**Statement:** When the system clipboard text does not match the last internally written text, reading returns `ClipboardMode::Stream` regardless of the text content.

**Validates: Requirements 1.5**

**Strategy:** Write an entry, externally modify the mock clipboard text, read back and verify mode is Stream.

---

### Property 11: Paste Produces Single UndoRecord

**Statement:** For any paste operation (stream, line, rectangular, multi-caret), exactly one `UndoRecord` is pushed to the undo stack, and undoing it restores the document to its pre-paste state.

**Validates: Requirements 4.9, 12.6, 13.4, 14.5, 15.1, 15.2**

**Strategy:** Generate paste scenarios across all modes, verify undo stack depth increases by 1, and undo restores original document.

---

### Property 12: File-Insert Line Count Matches File

**Statement:** For any valid text file with `N` logical lines, `FileInsertHandler::insert` inserts exactly `N` lines into the document (applying trailing-terminator suppression).

**Validates: Requirements 9.1, 9.8, 9.9, 9.10**

**Strategy:** Generate file content (1–200 lines, with/without trailing newline), verify inserted line count.

---

### Property 13: Configuration Fallback on Invalid Values

**Statement:** For any invalid configuration value (wrong type, out of range, absent key), `ClipboardConfig` resolves to its documented default without returning an error to the caller.

**Validates: Requirements 19.4**

**Strategy:** Generate configuration maps with missing keys, wrong types, and boundary values.

---

### Property 14: Context Menu Enablement Consistency

**Statement:** Cut/Copy are enabled iff a non-empty selection exists. Paste is enabled iff the clipboard has text content. These predicates never contradict the actual operation outcome.

**Validates: Requirements 5.6, 5.7**

**Strategy:** Generate (selection_state, clipboard_state) pairs and verify enablement matches operation success/failure.

---

## Testing Strategy

### Unit Tests

- `provider_tests.rs` — `ClipboardProvider` trait contract with a `MockClipboardProvider`
- `entry_tests.rs` — `ClipboardEntry` construction, accessors, mode inference
- `splitter_tests.rs` — `LineSplitter` edge cases (empty string, single char, mixed endings)
- `copy_tests.rs` — Copy handler with all selection types
- `cut_tests.rs` — Cut handler with undo verification
- `paste_tests.rs` — All paste modes with selection replacement
- `router_tests.rs` — COPY disambiguation truth table
- `file_insert_tests.rs` — File read, split, insert with error conditions
- `history_tests.rs` — Ring buffer capacity enforcement, FIFO eviction

### Property-Based Tests

All 14 properties above implemented with `proptest` crate, minimum 256 iterations each. Regression files committed alongside tests.

### Integration Tests

- End-to-end clipboard round-trips (copy → paste, cut → undo)
- COPY command dispatching across all four modes
- Configuration hot-reload effect on behaviour
- Multi-caret clipboard workflows

---

## Design Decisions and Rationale

| Decision | Rationale |
|----------|-----------|
| `ClipboardProvider` is a trait, not a concrete type | Enables testing without real OS clipboard; GUI shell injects platform implementation |
| Last-written entry stored locally in `ClipboardEngine` | Detecting internal vs external clipboard content requires comparing current system clipboard to last write |
| Segments stored as `Vec<String>` not `Vec<&str>` | Clipboard content lifetime is independent of document — must be owned |
| `LineSplitter` is a standalone utility | Reused by clipboard-paste, file-insert, and shell-capture — avoids duplication |
| `CopyCommandRouter` is pure logic (no I/O) | Testable in isolation; actual execution delegated to appropriate handler |
| Reverse-order processing for multi-caret | Earlier insertions shift positions of later carets; processing in reverse avoids invalidation |
| Single `UndoRecord` per operation | User mental model: one Ctrl+Z undoes the entire paste/cut regardless of complexity |
| Async file-insert via VFS | Large files should not block the UI thread; consistent with FFW-ARCH-001 async I/O principle |
| Configuration defaults are compile-time constants | Ensures predictable behaviour even if config system fails to load |
