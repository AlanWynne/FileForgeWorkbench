# Design Document: File Operations (`ff-file-ops`)

## 1. Overview

The `ff-file-ops` crate implements all **user-facing file operations** for the FileForgeWorkbench platform: New, Open, Save, Save As, Revert, and Recent Files. It also provides the underlying persistence mechanisms — atomic rename-on-write, backup copies, read-only detection, and unsaved-changes guards.

### Purpose

- Implement the `file.new`, `file.open`, `file.save`, `file.save_as`, `file.revert`, and `file.open_recent` commands
- Coordinate between the VFS layer (`ff-vfs`) and the document model (`ff-document-model`) for all file I/O
- Provide atomic write strategies (temp + rename, delete-first, direct) to prevent data corruption
- Manage the recent-files list with bounded LRU semantics and persistence
- Detect and enforce read-only status from VFS metadata and configuration
- Present unsaved-changes guards before destructive operations
- Delegate large-file async I/O to `ff-background-io` with progress reporting

### Position in Architecture

```
Wave 8 — File I/O and Session

┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│   (provides File_Picker, Unsaved_Changes_Dialog, status UI)  │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-file-ops ← Wave 8                            │
│  (command handlers, persistence strategies, recent files)    │
├─────────────────────────────────────────────────────────────┤
│  ff-vfs (I/O)  │  ff-document-model (content)               │
│  ff-command (dispatch)  │  ff-undo-redo (save-point)        │
│  ff-config (settings)   │  ff-background-io (async I/O)     │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: ALL file I/O goes through `ff-vfs` — no `std::fs` or `tokio::fs` calls in this crate
- **GUI Independence (Req 2)**: Core operations are GUI-independent; dialogs (File_Picker, Unsaved_Changes_Dialog) are abstracted behind traits that the shell layer implements
- **Command-Driven (Req 4)**: All operations are registered commands with metadata, shortcuts, and enabled-state predicates
- **Async I/O (Req 6)**: Large saves/loads delegate to `ff-background-io`; small operations complete synchronously below configurable threshold
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-file-ops`
- **Error Message Standards (Req 8)**: All errors follow `[file-ops] operation: description` format with resource URI context

### Upstream Dependencies

| Crate | Usage |
|-------|-------|
| `ff-vfs` | `ResourceUri`, `VfsProvider`, `stat`, `read_stream`, `write`, `rename`, `VfsMetadata`, `VfsCapabilities` |
| `ff-document-model` | `Document`, `DocumentHandle`, `TextBuffer`, content extraction, `StreamingFileReader` |
| `ff-command` | `CommandRegistry`, `CommandHandler`, `CommandId`, `CommandParams`, `CommandResult`, `ExecutionContext` |
| `ff-undo-redo` | `UndoManager::set_save_point()`, `UndoManager::clear()` (on revert) |
| `ff-config` | Settings: `file.save_strategy`, `file.backup.*`, `file.recent_files.max_count`, `file.async_threshold_bytes`, `save.check_modified_time`, `file.unsaved_prompt` |
| `ff-background-io` | `BackgroundTask`, `TaskHandle`, progress reporting for large file operations |
| `ff-logging` | Structured diagnostics at ERROR/WARN/INFO/DEBUG levels |

---

## 2. Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Invocation [Command Sources]
        MENU[Menu Bar]
        KBD[Keyboard Shortcut]
        PAL[Command Palette]
        LUA[Lua Macro]
        CLI[ISPF Command Line]
    end

    subgraph ff-file-ops [ff-file-ops Crate]
        REG[Command Registration<br/>file.new, file.open, etc.]
        NEW[NewFileHandler]
        OPEN[OpenFileHandler]
        SAVE[SaveHandler]
        SAVEAS[SaveAsHandler]
        REVERT[RevertHandler]
        RECENT[RecentFilesManager]
        GUARD[UnsavedChangesGuard]
        PERSIST[PersistenceStrategy<br/>Atomic / DeleteFirst / Direct]
        BACKUP[BackupManager]
        RO[ReadOnlyDetector]
        CFG[ConfigReader]
    end

    subgraph Upstream [Upstream Crates]
        VFS[ff-vfs<br/>ResourceUri, VfsProvider]
        DOC[ff-document-model<br/>Document, TextBuffer]
        CMD[ff-command<br/>CommandRegistry]
        UNDO[ff-undo-redo<br/>UndoManager]
        CONFIG[ff-config<br/>Settings]
        BIO[ff-background-io<br/>Async Tasks]
    end

    subgraph Shell [GUI Shell — ff-desktop]
        PICKER[File Picker Dialog]
        DIALOG[Unsaved Changes Dialog]
        STATUS[Status Bar / Progress]
    end

    MENU --> REG
    KBD --> REG
    PAL --> REG
    LUA --> REG
    CLI --> REG

    REG --> NEW
    REG --> OPEN
    REG --> SAVE
    REG --> SAVEAS
    REG --> REVERT
    REG --> RECENT

    SAVE --> GUARD
    SAVEAS --> GUARD
    NEW --> GUARD
    OPEN --> GUARD
    REVERT --> GUARD

    SAVE --> PERSIST
    SAVEAS --> PERSIST
    PERSIST --> BACKUP
    PERSIST --> VFS

    OPEN --> RO
    RO --> VFS
    RO --> CONFIG

    OPEN --> DOC
    SAVE --> DOC
    REVERT --> DOC

    SAVE --> UNDO
    REVERT --> UNDO
    SAVEAS --> UNDO

    GUARD --> DIALOG
    OPEN --> PICKER
    SAVEAS --> PICKER

    SAVE --> BIO
    OPEN --> BIO
    REVERT --> BIO

    CFG --> CONFIG
    RECENT --> CONFIG
end
```

### Request Flow: file.save

```
1. User triggers file.save (Ctrl+S / menu / command palette)
2. CommandDispatch routes to SaveHandler
3. SaveHandler checks: does Document have a ResourceUri?
   - No → delegate to SaveAsHandler (opens File_Picker)
   - Yes → continue
4. SaveHandler queries config: save.check_modified_time enabled?
   - Yes → VFS stat(uri), compare mtime with document's recorded mtime
     - Mismatch → prompt user via ExternalModificationDialog trait
     - User declines → abort
5. SaveHandler queries config: file.async_threshold_bytes
   - Document size ≤ threshold → synchronous save path
   - Document size > threshold → async save via ff-background-io
6. Check if save already in progress for this document → reject if so
7. PersistenceStrategy executes based on file.save_strategy config:
   - "atomic" → write to .tmp, fsync, rename over target
   - "delete_first" → delete target, write new, fsync
   - "direct" → overwrite target in place, fsync
8. If file.backup.enabled → BackupManager creates backup before overwrite
9. On success:
   - UndoManager.set_save_point()
   - Update document recorded mtime from VFS stat
   - Emit file.saved event via command framework
   - Update RecentFilesList
10. On failure:
    - Preserve all in-memory state
    - Emit error notification + ERROR log
```

### Request Flow: file.open

```
1. User triggers file.open (Ctrl+O / menu / command palette / URI argument)
2. CommandDispatch routes to OpenFileHandler
3. If no URI argument → invoke File_Picker trait (shell provides implementation)
4. If active document is dirty → UnsavedChangesGuard displays dialog
   - Save → save first, then continue on success
   - Discard → continue
   - Cancel → abort
5. Check if URI already open in another tab → activate existing tab, done
6. VFS stat(uri) → check existence, read-only status
7. VFS read_stream(uri) → load content into new Document
   - Size > async threshold → delegate to ff-background-io with progress
8. ReadOnlyDetector evaluates: VFS writable? + config read.only patterns?
   - Read-only → mark Document read-only
9. Record resource mtime from VFS stat
10. Create new tab, add URI to RecentFilesList
11. Emit file.opened event
```

---

## 3. Module Structure

```
crates/ff-file-ops/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API re-exports, crate docs
│   ├── open.rs             # OpenFileHandler: file.open command implementation
│   ├── save.rs             # SaveHandler: file.save command implementation
│   ├── save_as.rs          # SaveAsHandler: file.save_as command implementation
│   ├── new.rs              # NewFileHandler: file.new command implementation
│   ├── revert.rs           # RevertHandler: file.revert command implementation
│   ├── recent.rs           # RecentFilesManager: recent file list + file.open_recent
│   ├── commands.rs         # Command registration, metadata, enabled predicates
│   ├── persistence.rs      # PersistenceStrategy trait + Atomic/DeleteFirst/Direct impls
│   ├── backup.rs           # BackupManager: backup copy creation
│   ├── read_only.rs        # ReadOnlyDetector: VFS + config read-only evaluation
│   ├── guard.rs            # UnsavedChangesGuard: dirty-check + dialog dispatch
│   ├── config.rs           # Configuration key constants and typed access helpers
│   ├── error.rs            # FileOpsError enum
│   └── traits.rs           # Shell-provided trait abstractions (FilePicker, DialogProvider)
└── tests/
    ├── save_tests.rs           # Save operation unit + property tests
    ├── open_tests.rs           # Open operation unit + property tests
    ├── new_tests.rs            # New file operation tests
    ├── revert_tests.rs         # Revert operation tests
    ├── recent_tests.rs         # Recent files list property tests
    ├── persistence_tests.rs    # Atomic write strategy property tests
    ├── backup_tests.rs         # Backup creation tests
    ├── read_only_tests.rs      # Read-only detection tests
    ├── guard_tests.rs          # Unsaved changes guard tests
    └── integration.rs          # End-to-end file operation flows with mock VFS
```

---

## 4. Key Data Models and Types

### FileOpenOptions

```rust
/// Options for opening a file resource.
///
/// Addresses: Requirement 4 (Open)
#[derive(Debug, Clone)]
pub struct FileOpenOptions {
    /// The resource URI to open. If None, a File_Picker is displayed.
    pub uri: Option<ResourceUri>,
    /// Whether to force read-only mode regardless of VFS capabilities.
    pub force_read_only: bool,
    /// Encoding override (if None, auto-detected from BOM/content).
    pub encoding: Option<String>,
    /// Whether to reuse an existing tab if the URI is already open.
    pub reuse_existing_tab: bool,
}

impl Default for FileOpenOptions {
    fn default() -> Self {
        Self {
            uri: None,
            force_read_only: false,
            encoding: None,
            reuse_existing_tab: true,
        }
    }
}
```

### FileSaveOptions

```rust
/// Options controlling how a document is persisted.
///
/// Addresses: Requirement 1 (Save), Requirement 7 (Atomic Write)
#[derive(Debug, Clone)]
pub struct FileSaveOptions {
    /// Target URI. If None, uses the document's current URI.
    pub target_uri: Option<ResourceUri>,
    /// Override the configured save strategy for this operation.
    pub strategy_override: Option<SaveStrategy>,
    /// Whether to create a backup copy before overwriting.
    pub create_backup: Option<bool>,
    /// Whether to skip the external-modification-time check.
    pub skip_mtime_check: bool,
}

impl Default for FileSaveOptions {
    fn default() -> Self {
        Self {
            target_uri: None,
            strategy_override: None,
            create_backup: None,
            skip_mtime_check: false,
        }
    }
}
```

### SaveStrategy

```rust
/// The persistence strategy used when writing file content.
///
/// Addresses: Requirement 7, criteria 1–9
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SaveStrategy {
    /// Write to temp file, fsync, atomic rename over target (default).
    Atomic,
    /// Delete target first, then write new content, fsync.
    DeleteFirst,
    /// Overwrite target in place, fsync (for providers without rename).
    Direct,
}

impl Default for SaveStrategy {
    fn default() -> Self {
        Self::Atomic
    }
}
```

### RecentFileEntry

```rust
/// An entry in the recent files list.
///
/// Addresses: Requirement 6, criteria 1–10
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentFileEntry {
    /// The canonical resource URI.
    pub uri: ResourceUri,
    /// Display name (filename portion, for menu rendering).
    pub display_name: String,
    /// Timestamp when this entry was last accessed (opened or saved).
    pub last_accessed: SystemTime,
}
```

### RecentFilesList

```rust
/// A bounded, ordered list of recently accessed file URIs.
/// Most-recently-used ordering with configurable max capacity.
///
/// Addresses: Requirement 6, criteria 1–10
pub struct RecentFilesList {
    /// Ordered entries (index 0 = most recent).
    entries: Vec<RecentFileEntry>,
    /// Maximum number of entries (from configuration).
    max_count: usize,
}

impl RecentFilesList {
    /// Create a new list with the given capacity.
    pub fn new(max_count: usize) -> Self;

    /// Add or promote a URI to the top of the list.
    /// Removes duplicate entries. Evicts oldest if at capacity.
    /// Addresses: Requirement 6 AC 3, AC 4
    pub fn touch(&mut self, uri: ResourceUri, display_name: String);

    /// Remove a specific URI from the list.
    /// Addresses: Requirement 6 AC 6
    pub fn remove(&mut self, uri: &ResourceUri) -> bool;

    /// Get all entries in most-recent-first order.
    pub fn entries(&self) -> &[RecentFileEntry];

    /// Get entry at a specific index (0-based, most recent first).
    pub fn get(&self, index: usize) -> Option<&RecentFileEntry>;

    /// Number of entries currently stored.
    pub fn len(&self) -> usize;

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool;

    /// Update the maximum capacity. Truncates if needed.
    pub fn set_max_count(&mut self, max_count: usize);

    /// Serialize to a format suitable for config persistence.
    pub fn serialize(&self) -> Vec<String>;

    /// Deserialize from persisted config format.
    /// Addresses: Requirement 6 AC 8, AC 9
    pub fn deserialize(data: &[String], max_count: usize) -> Self;
}
```

### UnsavedChangesAction

```rust
/// The user's response to an unsaved-changes dialog.
///
/// Addresses: Requirement 9, criteria 1–8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChangesAction {
    /// Save the document before proceeding with the destructive operation.
    Save,
    /// Discard modifications and proceed immediately.
    Discard,
    /// Cancel the operation entirely; leave document unchanged.
    Cancel,
}
```

### FileOperationResult

```rust
/// Result of a file operation, returned to the command framework.
///
/// Addresses: Requirements 1–6
#[derive(Debug, Clone)]
pub enum FileOperationResult {
    /// Operation completed successfully.
    Success {
        /// The resource URI involved (if applicable).
        uri: Option<ResourceUri>,
        /// Human-readable status message for the status bar.
        message: String,
    },
    /// Operation was cancelled by the user (e.g., Cancel in dialog).
    Cancelled,
    /// Operation failed with an error.
    Failed {
        /// The underlying error.
        error: FileOpsError,
    },
}
```

### BackupConfig

```rust
/// Configuration for backup copy creation.
///
/// Addresses: Requirement 7, criteria 3–5
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Whether backups are enabled.
    pub enabled: bool,
    /// Where to store backups.
    pub location: BackupLocation,
    /// Suffix for alongside backups.
    pub suffix: String,
    /// Directory path for directory-mode backups.
    pub directory: Option<String>,
}

/// Where backup copies are stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupLocation {
    /// Same directory as the original file, with a configurable suffix.
    Alongside,
    /// Dedicated backup directory, preserving relative structure.
    Directory,
}
```

### SaveState

```rust
/// Tracks the in-progress state of a save operation for a document.
/// Prevents concurrent saves on the same document.
///
/// Addresses: Requirement 1 AC 7, AC 8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveState {
    /// No save in progress; document is idle.
    Idle,
    /// A synchronous save is executing.
    SavingSync,
    /// An async background save is in progress.
    SavingAsync,
}
```

### ReadOnlyStatus

```rust
/// The read-only status of a document, indicating the source of the restriction.
///
/// Addresses: Requirement 8, criteria 1–7
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOnlyStatus {
    /// Document is writable.
    Writable,
    /// Read-only due to VFS provider reporting non-writable.
    VfsRestricted,
    /// Read-only due to configuration pattern match.
    ConfigRestricted,
    /// Read-only due to provider not supporting write capability.
    ProviderLacksWrite,
    /// Manually toggled read-only by user.
    UserToggled,
}

impl ReadOnlyStatus {
    /// Whether the document is effectively read-only.
    pub fn is_read_only(&self) -> bool {
        !matches!(self, Self::Writable)
    }
}
```

---

## 5. Public API Surface

### Command Registration

```rust
/// Register all file operation commands with the command framework.
///
/// Addresses: Requirement 10
pub fn register_file_commands(
    registry: &CommandRegistry,
    ctx: FileOpsContext,
) -> Result<(), FileOpsError>;
```

### FileOpsContext

```rust
/// Shared context required by all file operation handlers.
/// Holds references to upstream services.
///
/// Wired at application startup; passed to command handlers.
pub struct FileOpsContext {
    /// VFS provider registry for all I/O operations.
    pub vfs: Arc<ProviderRegistry>,
    /// Undo manager for save-point marking and stack clearing.
    pub undo_manager: Arc<dyn UndoManagerAccess>,
    /// Configuration access for file operation settings.
    pub config: Arc<dyn ConfigAccess>,
    /// Background I/O task runner for async operations.
    pub background_io: Arc<dyn BackgroundIoRunner>,
    /// Dialog provider for File_Picker and Unsaved_Changes_Dialog.
    pub dialog_provider: Arc<dyn DialogProvider>,
    /// Recent files list (shared, mutable).
    pub recent_files: Arc<RwLock<RecentFilesList>>,
    /// Event emitter for file.* events.
    pub event_bus: Arc<dyn EventBus>,
}
```

### Shell Trait Abstractions

```rust
/// Trait abstraction for file picker dialogs.
/// The GUI shell (ff-desktop) provides the concrete implementation.
///
/// Addresses: GUI Independence cross-cutting requirement
#[async_trait::async_trait]
pub trait DialogProvider: Send + Sync {
    /// Show a file picker in open mode. Returns selected URIs or empty if cancelled.
    async fn show_open_picker(
        &self,
        options: &OpenPickerOptions,
    ) -> Vec<ResourceUri>;

    /// Show a file picker in save mode. Returns selected URI or None if cancelled.
    async fn show_save_picker(
        &self,
        options: &SavePickerOptions,
    ) -> Option<ResourceUri>;

    /// Show the unsaved changes dialog. Returns the user's chosen action.
    async fn show_unsaved_changes(
        &self,
        document_name: &str,
    ) -> UnsavedChangesAction;

    /// Show an overwrite confirmation dialog. Returns true if user confirms.
    async fn show_overwrite_confirmation(
        &self,
        uri: &ResourceUri,
    ) -> bool;

    /// Show an external modification confirmation dialog.
    async fn show_external_modification_warning(
        &self,
        uri: &ResourceUri,
    ) -> bool;
}
```

### Core Operation Functions

```rust
/// Create a new empty document.
///
/// Addresses: Requirement 3
pub async fn new_file(
    ctx: &FileOpsContext,
    active_document: Option<&DocumentHandle>,
) -> FileOperationResult;

/// Open a file from the VFS.
///
/// Addresses: Requirement 4
pub async fn open_file(
    ctx: &FileOpsContext,
    options: FileOpenOptions,
    active_document: Option<&DocumentHandle>,
) -> FileOperationResult;

/// Save the current document to its associated URI.
///
/// Addresses: Requirement 1
pub async fn save_file(
    ctx: &FileOpsContext,
    document: &DocumentHandle,
    options: FileSaveOptions,
) -> FileOperationResult;

/// Save the current document to a new URI (Save As).
///
/// Addresses: Requirement 2
pub async fn save_file_as(
    ctx: &FileOpsContext,
    document: &DocumentHandle,
    options: FileSaveOptions,
) -> FileOperationResult;

/// Revert document to the last saved state from VFS.
///
/// Addresses: Requirement 5
pub async fn revert_file(
    ctx: &FileOpsContext,
    document: &DocumentHandle,
) -> FileOperationResult;

/// Open a recent file by index.
///
/// Addresses: Requirement 6
pub async fn open_recent(
    ctx: &FileOpsContext,
    index: usize,
    active_document: Option<&DocumentHandle>,
) -> FileOperationResult;
```

### Persistence Strategy API

```rust
/// Trait for persistence strategy implementations.
///
/// Addresses: Requirement 7
#[async_trait::async_trait]
pub trait PersistenceStrategy: Send + Sync {
    /// Write document content to the target URI using this strategy.
    async fn write(
        &self,
        vfs: &ProviderRegistry,
        uri: &ResourceUri,
        content: &[u8],
        backup_config: &BackupConfig,
    ) -> Result<(), FileOpsError>;
}

/// Atomic write strategy: temp file + fsync + rename.
pub struct AtomicWriteStrategy;

/// Delete-first strategy: delete target, write new, fsync.
pub struct DeleteFirstStrategy;

/// Direct overwrite strategy: write in place, fsync.
pub struct DirectWriteStrategy;
```

### Read-Only Detection API

```rust
/// Evaluate the read-only status of a resource.
///
/// Addresses: Requirement 8
pub fn detect_read_only(
    vfs: &ProviderRegistry,
    uri: &ResourceUri,
    metadata: &VfsMetadata,
    config: &dyn ConfigAccess,
) -> ReadOnlyStatus;

/// Toggle read-only status for a document (user override).
///
/// Addresses: Requirement 8 AC 5
pub fn toggle_read_only(document: &DocumentHandle) -> ReadOnlyStatus;
```

### Unsaved Changes Guard API

```rust
/// Check if a document has unsaved changes and prompt if needed.
/// Returns Ok(true) to proceed, Ok(false) if user cancelled.
///
/// Addresses: Requirement 9
pub async fn guard_unsaved_changes(
    ctx: &FileOpsContext,
    document: &DocumentHandle,
) -> Result<bool, FileOpsError>;

/// Batch guard for multiple documents (Close All / Exit).
/// Returns Ok(true) if all documents are handled, Ok(false) if user cancelled.
///
/// Addresses: Requirement 9 AC 6
pub async fn guard_unsaved_changes_batch(
    ctx: &FileOpsContext,
    documents: &[&DocumentHandle],
) -> Result<bool, FileOpsError>;
```

---

## 6. Error Types

```rust
/// Error type for all file operation failures.
///
/// All variants include sufficient context for diagnostics.
/// Display format: `[file-ops] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FileOpsError {
    /// VFS operation failed (wraps VfsError with file-ops context).
    #[error("[file-ops] {operation}: VFS error for {uri} — {source}")]
    Vfs {
        operation: String,
        uri: ResourceUri,
        #[source]
        source: VfsError,
    },

    /// Document is read-only; mutation was rejected.
    #[error("[file-ops] {operation}: document is read-only ({status:?})")]
    ReadOnly {
        operation: String,
        status: ReadOnlyStatus,
    },

    /// A save is already in progress for the target document.
    #[error("[file-ops] save: operation already in progress for {uri}")]
    SaveInProgress {
        uri: ResourceUri,
    },

    /// Document has no associated URI (e.g., revert on untitled document).
    #[error("[file-ops] {operation}: document has no associated resource URI")]
    NoUri {
        operation: String,
    },

    /// The resource was not found on the VFS.
    #[error("[file-ops] {operation}: resource not found — {uri}")]
    ResourceNotFound {
        operation: String,
        uri: ResourceUri,
    },

    /// Atomic rename is not supported by the provider; fell back to direct write.
    #[error("[file-ops] save: provider '{provider}' does not support atomic rename — using direct write")]
    AtomicRenameUnsupported {
        provider: String,
    },

    /// Backup copy creation failed (non-fatal, logged as WARN).
    #[error("[file-ops] backup: failed to create backup for {uri} — {reason}")]
    BackupFailed {
        uri: ResourceUri,
        reason: String,
    },

    /// External modification detected; user declined to proceed.
    #[error("[file-ops] save: external modification detected for {uri} — user declined")]
    ExternalModificationDeclined {
        uri: ResourceUri,
    },

    /// Configuration error (invalid setting value).
    #[error("[file-ops] config: invalid value for '{key}' — {reason}")]
    ConfigError {
        key: String,
        reason: String,
    },

    /// The recent files list could not be persisted.
    #[error("[file-ops] recent: failed to persist recent files list — {reason}")]
    RecentPersistFailed {
        reason: String,
    },
}
```

---

## 7. Integration Points

### Integration with `ff-vfs`

| Operation | VFS API Used | Notes |
|-----------|-------------|-------|
| Open (read) | `read_stream(uri)` | Streaming read for large files via background-io |
| Open (stat) | `stat(uri)` | Retrieve mtime, size, read-only status |
| Save (write) | `write(tmp_path, data)` | Write to temp file in atomic strategy |
| Save (rename) | `rename(tmp_path, target)` | Atomic rename-over-target |
| Save (delete) | `delete(target)` | For delete-first strategy |
| Backup | `read(uri)` + `write(backup_uri, data)` | Copy original before overwrite |
| Revert | `read_stream(uri)` | Reload from storage |
| Exists check | `exists(uri)` | Overwrite confirmation in Save As |
| Capability | `provider_capabilities(scheme)` | Check WRITE/RENAME support |

All operations use `ResourceUri` for addressing. Bare paths from user input are converted via `ResourceUri::from_bare_path()`.

### Integration with `ff-document-model`

| Operation | Document API Used | Notes |
|-----------|------------------|-------|
| Open | `Document::new()`, `StreamingFileReader::load()` | Create document, load content |
| Save | `TextBuffer::contiguous_view()` or `split_view()` | Extract content bytes for writing |
| Save As | Same as Save + update document's URI | Reassociate document with new resource |
| Revert | `TextBuffer::delete()` + `insert()` (full replace) | Replace buffer content entirely |
| New | `Document::new()` | Create empty document with no URI |
| Read-only | `TextBuffer::set_read_only(true)` | Prevent mutations on read-only files |

### Integration with `ff-undo-redo`

| Operation | Undo API Used | Notes |
|-----------|--------------|-------|
| Save (success) | `UndoManager::set_save_point()` | Marks current position as "clean" |
| Save As (success) | `UndoManager::set_save_point()` | Same — new URI but same undo history |
| Revert | `UndoManager::clear()` | Completely resets undo/redo stacks |
| New | (fresh document) | New documents start with empty undo stack |

### Integration with `ff-command`

| Aspect | Details |
|--------|---------|
| Command IDs | `file.new`, `file.open`, `file.open_recent`, `file.save`, `file.save_as`, `file.revert`, `file.close`, `file.exit` |
| Handler type | `AsyncCommandHandler` (all operations are async due to dialog + I/O) |
| Enabled predicates | `file.save` → dirty AND has URI; `file.revert` → has URI; others → always enabled |
| Events emitted | `file.new_created`, `file.opened`, `file.saved`, `file.reverted` |
| Category | `"file"` |
| Shortcuts | New=Ctrl+N, Open=Ctrl+O, Save=Ctrl+S, SaveAs=Ctrl+Shift+S, Close=Ctrl+W, Exit=Alt+F4 |

### Integration with `ff-config`

| Setting Key | Type | Default | Purpose |
|-------------|------|---------|---------|
| `file.save_strategy` | `String` | `"atomic"` | Persistence strategy selection |
| `file.backup.enabled` | `bool` | `false` | Enable/disable backup copies |
| `file.backup.location` | `String` | `"alongside"` | Backup storage mode |
| `file.backup.suffix` | `String` | `".bak"` | Suffix for alongside backups |
| `file.backup.directory` | `String` | `""` | Path for directory-mode backups |
| `file.recent_files.max_count` | `u32` | `10` | Maximum recent files entries |
| `file.async_threshold_bytes` | `u64` | `1048576` | Size threshold for async I/O (1 MB) |
| `save.check_modified_time` | `bool` | `true` | External modification check before save |
| `file.unsaved_prompt` | `bool` | `true` | Whether to show unsaved-changes dialog |
| `read.only` | `String` (glob pattern) | `""` | Force read-only by file pattern |

### Integration with `ff-background-io`

| Scenario | Background-IO API | Notes |
|----------|-------------------|-------|
| Large file save | `BackgroundIoRunner::submit_write(uri, content)` | Returns TaskHandle with progress |
| Large file open | `BackgroundIoRunner::submit_read(uri)` | Returns TaskHandle + async stream |
| Large file revert | `BackgroundIoRunner::submit_read(uri)` | Same as open, replacing buffer |
| Progress | `TaskHandle::progress()` → percentage/bytes | Status bar integration |
| Cancellation | `TaskHandle::cancel()` | User-initiated cancel |
| Concurrency guard | Check `SaveState` before submitting | Prevent duplicate saves |

### Integration with `multi-tab-editor` (downstream)

- `file.open` creates a new tab via the tab manager
- `file.open` checks for existing tabs with the same URI (dedup)
- `file.new` creates a new tab with sequential "Untitled-N" naming
- `file.close` / `file.exit` triggers unsaved-changes guard for each dirty tab

---

## 8. Correctness Properties

These properties are suitable for property-based testing with the `proptest` crate.

### Property 1: Recent Files List Bounded Size

**Statement**: For any sequence of `touch` operations on a `RecentFilesList` with max capacity `N`, the list length never exceeds `N`. After any `touch`, `len() <= max_count` holds.

**Validates**: Requirement 6 AC 2, AC 4

```rust
// proptest strategy: generate N in 1..50, then sequence of random URIs to touch
// assertion: after every touch, list.len() <= N
```

### Property 2: Recent Files List MRU Ordering

**Statement**: After `touch(uri)`, the touched URI is always at index 0 (most recent position). No duplicates exist in the list — the same URI never appears twice.

**Validates**: Requirement 6 AC 1, AC 3

```rust
// proptest strategy: generate sequence of URIs (some repeated)
// assertion: after touch(uri), list.entries()[0].uri == uri
// assertion: no two entries have the same URI
```

### Property 3: Atomic Write Produces Identical Content

**Statement**: For any byte content `C` and target URI, after a successful atomic write, reading the target via VFS yields exactly `C`. The content is never partially written (no intermediate states observable).

**Validates**: Requirement 7 AC 1

```rust
// proptest strategy: generate arbitrary byte vectors (0..1MB)
// test with mock VFS: atomic write + read back, assert equality
// simulate crash (abort between write and rename) → target unchanged
```

### Property 4: Save Clears Dirty Flag

**Statement**: For any document that is dirty (has unsaved modifications), after a successful `save_file` operation, the undo manager reports `is_at_save_point() == true` (dirty flag is cleared).

**Validates**: Requirement 1 AC 2

```rust
// proptest strategy: generate edit sequences that make document dirty
// action: save_file succeeds
// assertion: undo_manager.is_at_save_point() == true
```

### Property 5: Revert Restores Original Content

**Statement**: For any document loaded from a VFS resource, after a sequence of edits followed by `revert_file`, the document content equals the content of the resource on the VFS (the last-saved state).

**Validates**: Requirement 5 AC 2, AC 3

```rust
// proptest strategy: generate initial file content, then random edits
// action: revert_file
// assertion: document content == original VFS content
// assertion: undo stack is empty, dirty flag is false
```

### Property 6: Read-Only Detection Consistency

**Statement**: For any resource where the VFS provider reports `!capabilities.contains(WRITE)` or the VFS metadata indicates non-writable, `detect_read_only` returns a non-`Writable` status. Conversely, if the provider supports WRITE and metadata allows it and no config override applies, the status is `Writable`.

**Validates**: Requirement 8 AC 1, AC 4, AC 7

```rust
// proptest strategy: generate combinations of (provider_capabilities, metadata_writable, config_pattern_match)
// assertion: read-only iff any restricting condition is true
```

### Property 7: Unsaved Changes Guard Idempotency

**Statement**: If a document is not dirty (`is_at_save_point() == true`), calling `guard_unsaved_changes` never displays a dialog and always returns `Ok(true)` (proceed). If `file.unsaved_prompt` is `false`, the guard never displays a dialog regardless of dirty state.

**Validates**: Requirement 9 AC 1, AC 7

```rust
// proptest strategy: generate (dirty: bool, unsaved_prompt: bool) pairs
// assertion: dialog shown only when dirty == true AND unsaved_prompt == true
```

### Property 8: Save Strategy Selection

**Statement**: The `PersistenceStrategy` used for a save operation always matches the effective configuration: if `file.save_strategy == "atomic"` and the provider supports RENAME capability, `AtomicWriteStrategy` is used. If the provider lacks RENAME, the system falls back to `DirectWriteStrategy` regardless of config.

**Validates**: Requirement 7 AC 1, AC 2, AC 6, AC 7

```rust
// proptest strategy: generate (config_strategy, provider_capabilities) combinations
// assertion: effective strategy is config choice when provider supports it,
//            otherwise falls back to Direct
```

### Property 9: Save Prevents Concurrent Operations

**Statement**: If `SaveState` for a document is `SavingSync` or `SavingAsync`, a second `save_file` call returns `FileOpsError::SaveInProgress` without performing any VFS operation. The save state returns to `Idle` only after the in-progress save completes or fails.

**Validates**: Requirement 1 AC 7, AC 8

```rust
// proptest strategy: simulate concurrent save attempts with varying timing
// assertion: only one save executes; others receive SaveInProgress error
```

### Property 10: Backup Copy Preservation

**Statement**: When `file.backup.enabled == true` and a backup copy is created before save, the backup content equals the original file content (the pre-save state). Backup failure does not abort the save — the save proceeds regardless.

**Validates**: Requirement 7 AC 3, AC 4, AC 5

```rust
// proptest strategy: generate file content, modify, save with backup enabled
// assertion: backup file content == original content before save
// assertion: backup failure → save still succeeds
```

---

## Appendix A: Configuration Keys Reference

All keys live under the `file.*` namespace in the configuration system:

```toml
[file]
save_strategy = "atomic"          # "atomic" | "delete_first" | "direct"
async_threshold_bytes = 1048576   # 1 MB default
unsaved_prompt = true

[file.backup]
enabled = false
location = "alongside"            # "alongside" | "directory"
suffix = ".bak"
directory = ""

[file.recent_files]
max_count = 10

[save]
check_modified_time = true
```

## Appendix B: Event Bus Messages

| Event ID | Payload | Emitted By |
|----------|---------|------------|
| `file.new_created` | `{ uri: None, title: "Untitled-N" }` | `new_file` |
| `file.opened` | `{ uri: ResourceUri, read_only: bool }` | `open_file` |
| `file.saved` | `{ uri: ResourceUri }` | `save_file`, `save_file_as` |
| `file.reverted` | `{ uri: ResourceUri }` | `revert_file` |
| `file.recent_updated` | `{ count: usize }` | `RecentFilesList::touch` |

## Appendix C: Untitled Document Naming

New documents are assigned sequential identifiers: "Untitled-1", "Untitled-2", etc. The counter is maintained per session (resets on application restart). The counter increments monotonically — closed untitled documents do not reclaim their numbers within the same session.
