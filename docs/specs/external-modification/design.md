# Design Document: External Modification Detection (`ff-external-mod`)

## 1. Overview

The `ff-external-mod` crate detects when files open in the workbench are modified, renamed, or deleted by external processes (other editors, build tools, version control, shell scripts). It subscribes to VFS file-watcher events, tracks per-document modification times, and coordinates reload/notification decisions with configurable policies.

### Purpose

- Subscribe to VFS watch events for all open documents, correlating file-system changes with in-memory state
- Maintain per-document mtime snapshots for reliable change detection (even when watch events are missed)
- Emit `ExternalChange` events when discrepancies between disk and in-memory state are detected
- Implement configurable reload policies: `prompt`, `auto`, `ignore`
- Resolve conflicts when a document buffer is dirty (unsaved local changes)
- Coalesce rapid change events into batch notifications to avoid notification storms
- Perform focus-gained and tab-switch mtime revalidation as a safety net
- Handle file deletion and rename scenarios with appropriate user options
- Fall back to polling when the VFS provider does not support file watching

### Position in Architecture

```
Wave 8 — File I/O and Session

┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│   (provides ExternalModDialog, BatchNotificationDialog)      │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-external-mod ← Wave 8                        │
│  (change detection, mtime tracking, policy engine, batching) │
├─────────────────────────────────────────────────────────────┤
│  ff-vfs (watch/stat)  │  ff-document-model (dirty state)    │
│  ff-file-ops (revert) │  ff-background-io (async reload)    │
│  ff-config (policies) │  ff-logging (diagnostics)           │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: ALL filesystem interaction flows through `ff-vfs` — no `std::fs` or `tokio::fs` calls for watching or stat
- **GUI Independence (Req 2)**: Core detection logic is GUI-independent; user prompts are abstracted behind traits that the shell layer implements
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-external-mod`
- **Error Message Standards (Req 8)**: All errors follow `[external-mod] operation: description` format with resource URI context

### Upstream Dependencies

| Crate | Usage |
|-------|-------|
| `ff-vfs` | `ResourceUri`, `VfsProvider` (via `ProviderRegistry`), `stat()`, `watch()`, `WatchHandle`, `WatchEvent`, `WatchOptions`, `VfsError`, `VfsMetadata` |
| `ff-document-model` | `DocumentHandle`, dirty state queries, content replacement on reload |
| `ff-file-ops` | `revert_file()` for reload operations |
| `ff-background-io` | `BackgroundIoService::spawn_load()` for async reload of large files |
| `ff-config` | `[editor.external_modification]` namespace — policy, debounce, polling settings |
| `ff-logging` | Structured diagnostics at INFO/WARN/DEBUG levels |

---

## 2. Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Events [Event Sources]
        VFS_WATCH[VFS WatchHandle<br/>async event stream]
        FOCUS[Window Focus-Gained<br/>platform event]
        TAB[Tab Switch<br/>UI event]
        POLL[Fallback Poller<br/>periodic mtime check]
    end

    subgraph ff-external-mod [ff-external-mod Crate]
        DETECTOR[ExternalModificationDetector<br/>central coordination]
        MTIME[MtimeTracker<br/>per-document snapshots]
        POLICY[PolicyEngine<br/>prompt / auto / ignore]
        BATCH[BatchCoalescer<br/>debounce window grouping]
        FOCUS_CHK[FocusChecker<br/>mtime scan on activate]
        POLLER[FallbackPoller<br/>periodic stat loop]
        NOTIFIER[NotificationEmitter<br/>ExternalChange events]
        CONFIG[ConfigReader<br/>policy + timing settings]
    end

    subgraph Upstream [Upstream Crates]
        VFS[ff-vfs<br/>watch, stat, ResourceUri]
        DOC[ff-document-model<br/>DocumentHandle, dirty state]
        FOPS[ff-file-ops<br/>revert_file]
        BIO[ff-background-io<br/>async reload]
        CFGSYS[ff-config<br/>settings]
        LOG[ff-logging]
    end

    subgraph Shell [GUI Shell — ff-desktop]
        MOD_DIALOG[Reload / Keep / Diff Dialog]
        DEL_DIALOG[Deleted File Dialog]
        REN_DIALOG[Renamed File Dialog]
        BATCH_DIALOG[Batch Notification Dialog]
        STATUS[Status Bar Message]
    end

    VFS_WATCH --> DETECTOR
    FOCUS --> FOCUS_CHK
    TAB --> FOCUS_CHK
    POLL --> POLLER

    DETECTOR --> MTIME
    DETECTOR --> POLICY
    DETECTOR --> BATCH
    DETECTOR --> NOTIFIER
    FOCUS_CHK --> MTIME
    FOCUS_CHK --> BATCH
    POLLER --> MTIME

    MTIME -->|stat| VFS
    DETECTOR -->|watch/cancel| VFS
    POLICY --> CONFIG
    CONFIG --> CFGSYS

    NOTIFIER -->|ExternalChange| POLICY
    POLICY -->|auto-reload| FOPS
    POLICY -->|prompt| MOD_DIALOG
    POLICY -->|prompt| DEL_DIALOG
    POLICY -->|prompt| REN_DIALOG
    BATCH -->|batch prompt| BATCH_DIALOG
    POLICY -->|status msg| STATUS

    FOPS -->|reload content| BIO
    DETECTOR -->|dirty query| DOC
    DETECTOR --> LOG
end
```

### Request Flow: VFS Modified Event

```
1. VFS WatchHandle delivers WatchEvent::Modified { uri } via async channel
2. ExternalModificationDetector receives event, looks up DocumentHandle by URI
3. MtimeTracker queries VFS stat(uri) → new_mtime
4. Compare new_mtime against stored Mtime_Snapshot:
   - Equal → spurious event, discard silently
   - Different → external change confirmed
5. Check deduplication: is there already a pending (unanswered) prompt for this mtime?
   - Yes → discard (Requirement 3 AC 6)
   - No → continue
6. Emit ExternalChange { doc_id, ContentChanged, old_mtime, new_mtime, is_dirty }
7. BatchCoalescer checks: are we within the debounce window?
   - Yes → buffer the event, wait for window expiry
   - No → start new batch window, pass through immediately if single event
8. PolicyEngine evaluates Reload_Policy:
   - "ignore" → update Mtime_Snapshot, done
   - "auto" + NOT dirty → auto-reload via ff-file-ops revert_file(), update snapshot
   - "auto" + dirty → fall through to "prompt" behaviour
   - "prompt" → invoke NotificationDialog trait (shell provides UI)
9. After user response (or auto-reload completion):
   - Update Mtime_Snapshot to current on-disk mtime
   - Update "last-asked mtime" to prevent re-prompting
```

### Request Flow: Focus-Gained Check

```
1. Platform shell reports focus-gained event to ff-external-mod
2. FocusChecker verifies config: check_on_focus == true?
   - No → skip entirely
3. FocusChecker collects all open documents with backing URIs
4. For each document: VFS stat(uri) → current_mtime
5. Compare against Mtime_Snapshot:
   - Equal → skip
   - Different → check "last-asked mtime" to avoid re-prompting
6. Collect all changed documents into a batch
7. If batch.len() > 1 → route through BatchCoalescer
8. If batch.len() == 1 → route directly to PolicyEngine
9. Processing follows same policy logic as the VFS event flow
```

### Request Flow: File Deletion

```
1. VFS WatchHandle delivers WatchEvent::Deleted { uri }
2. ExternalModificationDetector looks up DocumentHandle by URI
3. Emit ExternalChange { doc_id, FileDeleted }
4. Cancel the VFS watch for this resource (target no longer exists)
5. Present deletion notification via shell trait:
   - Save As → user picks new location, save content
   - Keep Editing → mark dirty, clear backing URI (orphaned buffer)
   - Close → if dirty, trigger standard unsaved-changes guard first
6. If deleted file later reappears (Created event for same URI):
   - Do NOT auto-associate with the orphaned buffer
   - The buffer remains untitled until user explicitly saves
```

### Request Flow: File Rename

```
1. VFS WatchHandle delivers WatchEvent::Renamed { old_uri, new_uri }
2. ExternalModificationDetector matches old_uri to an open document
3. Emit ExternalChange { doc_id, FileRenamed { old_uri, new_uri } }
4. Check config: auto_follow_rename == true AND buffer NOT dirty?
   - Yes → automatically follow rename (skip prompt)
   - No → present rename notification via shell trait
5. If Follow Rename:
   - Update document backing URI to new_uri
   - Update tab title
   - Cancel watch on old_uri, register watch on new_uri
   - Update Mtime_Snapshot from VFS stat(new_uri)
6. If Keep Old Path:
   - Mark document dirty (backing file no longer at old path)
   - Cancel watch on old_uri
   - Document treated as orphaned (like deletion)
```

---

## 3. Module Structure

```
crates/ff-external-mod/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API re-exports, crate docs
│   ├── detector.rs         # ExternalModificationDetector: central coordinator
│   ├── mtime.rs            # MtimeTracker: per-document mtime snapshots, stat calls
│   ├── policy.rs           # PolicyEngine: reload policy evaluation and dispatch
│   ├── batch.rs            # BatchCoalescer: debounce window, event grouping
│   ├── focus.rs            # FocusChecker: focus-gained and tab-switch mtime scans
│   ├── poller.rs           # FallbackPoller: periodic stat loop for non-watch providers
│   ├── events.rs           # ExternalChange enum, ChangeType, BatchNotification
│   ├── config.rs           # Configuration key constants and typed access helpers
│   ├── traits.rs           # Shell-provided trait abstractions (dialog providers)
│   ├── error.rs            # ExternalModError enum
│   └── types.rs            # MtimeSnapshot, DocumentRegistration, PendingPrompt
└── tests/
    ├── detector_tests.rs       # Detector integration tests with mock VFS
    ├── mtime_tests.rs          # Mtime tracking property tests
    ├── policy_tests.rs         # Policy engine decision tests
    ├── batch_tests.rs          # Batch coalescing property tests
    ├── focus_tests.rs          # Focus-gained check tests
    ├── poller_tests.rs         # Fallback poller timing tests
    ├── deletion_tests.rs       # Deletion handling tests
    ├── rename_tests.rs         # Rename handling tests
    └── integration.rs          # End-to-end flows with mock VFS + mock shell
```

---

## 4. Key Data Models and Types

### MtimeSnapshot

```rust
/// A recorded modification timestamp for a document's backing file.
/// Used as the baseline for detecting external changes.
///
/// Addresses: Requirement 2, criteria 1–8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MtimeSnapshot {
    /// The modification time from VFS stat, with sub-second precision where supported.
    pub mtime: SystemTime,
    /// Whether this snapshot was obtained successfully (false if stat failed).
    pub is_valid: bool,
}

impl MtimeSnapshot {
    /// Create a snapshot from a successful VFS stat result.
    pub fn from_metadata(metadata: &VfsMetadata) -> Self;

    /// Create an invalid snapshot (used when stat fails).
    pub fn invalid() -> Self;
}
```

### DocumentRegistration

```rust
/// Tracks an open document's external modification state.
///
/// Addresses: Requirements 1–3
#[derive(Debug)]
pub struct DocumentRegistration {
    /// The document handle for content and dirty-state queries.
    pub handle: DocumentHandle,
    /// The resource URI being watched.
    pub uri: ResourceUri,
    /// The VFS watch handle (None if using fallback polling).
    pub watch_handle: Option<WatchHandle>,
    /// The last-known mtime when the file was loaded or saved.
    pub mtime_snapshot: MtimeSnapshot,
    /// The mtime at which the user was last prompted (prevents re-prompting).
    pub last_asked_mtime: Option<SystemTime>,
    /// Whether there is a pending (unanswered) prompt for this document.
    pub pending_prompt: bool,
}
```

### ExternalChange

```rust
/// An event indicating an open document's backing file was externally changed.
///
/// Addresses: Requirement 3, all acceptance criteria
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ExternalChange {
    /// File content was modified externally.
    ContentChanged {
        /// Identifier for the affected document.
        doc_id: DocumentId,
        /// The previously recorded mtime.
        old_mtime: SystemTime,
        /// The new mtime detected on disk.
        new_mtime: SystemTime,
        /// Whether the in-memory buffer has unsaved local changes.
        is_dirty: bool,
    },

    /// File was deleted externally.
    FileDeleted {
        /// Identifier for the affected document.
        doc_id: DocumentId,
    },

    /// File was renamed/moved externally.
    FileRenamed {
        /// Identifier for the affected document.
        doc_id: DocumentId,
        /// The old resource URI (where the document was).
        old_uri: ResourceUri,
        /// The new resource URI (where the file moved to).
        new_uri: ResourceUri,
    },
}
```

### ReloadPolicy

```rust
/// Configurable strategy for responding to external modifications.
///
/// Addresses: Requirement 10, criterion 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadPolicy {
    /// Always ask the user what to do.
    Prompt,
    /// Auto-reload if buffer is clean; prompt if dirty.
    Auto,
    /// Never notify — keep in-memory content as-is.
    Ignore,
}

impl Default for ReloadPolicy {
    fn default() -> Self {
        Self::Prompt
    }
}
```

### ReloadAction

```rust
/// The user's response to an external modification prompt.
///
/// Addresses: Requirement 4, criteria 1–8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadAction {
    /// Reload from disk (discard local changes if any).
    Reload,
    /// Keep in-memory content, ignore the external change.
    Keep,
    /// Show a diff between in-memory and on-disk content.
    Diff,
}
```

### DeleteAction

```rust
/// The user's response to a file-deleted notification.
///
/// Addresses: Requirement 6, criteria 1–6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteAction {
    /// Save buffer content to a new location.
    SaveAs,
    /// Continue editing with no backing file.
    KeepEditing,
    /// Close the document tab.
    Close,
}
```

### RenameAction

```rust
/// The user's response to a file-renamed notification.
///
/// Addresses: Requirement 7, criteria 1–6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameAction {
    /// Update the document's URI to track the new location.
    FollowRename,
    /// Keep the original URI (document becomes orphaned).
    KeepOldPath,
}
```

### BatchNotification

```rust
/// A coalesced group of external changes within a debounce window.
///
/// Addresses: Requirement 8, criteria 1–7
#[derive(Debug, Clone)]
pub struct BatchNotification {
    /// Documents with content changes.
    pub modified: Vec<ExternalChange>,
    /// Documents whose backing files were deleted.
    pub deleted: Vec<ExternalChange>,
    /// Documents whose backing files were renamed.
    pub renamed: Vec<ExternalChange>,
}

impl BatchNotification {
    /// Total count of affected documents.
    pub fn total_count(&self) -> usize;

    /// Documents in the batch that have dirty buffers.
    pub fn dirty_documents(&self) -> Vec<DocumentId>;

    /// Documents in the batch that are clean (safe for auto-reload).
    pub fn clean_documents(&self) -> Vec<DocumentId>;
}
```

### BatchAction

```rust
/// The user's bulk response to a batch notification.
///
/// Addresses: Requirement 8, criteria 3–5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchAction {
    /// Reload all non-dirty documents in the batch.
    ReloadAll,
    /// Dismiss all notifications (keep everything as-is).
    KeepAll,
    /// Present each change individually for review.
    ReviewIndividually,
}
```

### ExternalModConfig

```rust
/// Typed configuration for the external modification subsystem.
///
/// Addresses: Requirement 10, all criteria
#[derive(Debug, Clone)]
pub struct ExternalModConfig {
    /// Reload policy: "prompt", "auto", "ignore".
    pub policy: ReloadPolicy,
    /// Whether reload preserves undo history.
    pub reload_preserves_undo: bool,
    /// Whether to perform mtime scan on focus-gained.
    pub check_on_focus: bool,
    /// Whether to auto-follow renames for non-dirty buffers.
    pub auto_follow_rename: bool,
    /// Debounce window for batch coalescing (ms).
    pub batch_debounce_ms: u64,
    /// Fallback polling interval when VFS watch unavailable (ms).
    pub polling_interval_ms: u64,
}

impl Default for ExternalModConfig {
    fn default() -> Self {
        Self {
            policy: ReloadPolicy::Prompt,
            reload_preserves_undo: false,
            check_on_focus: true,
            auto_follow_rename: false,
            batch_debounce_ms: 500,
            polling_interval_ms: 5000,
        }
    }
}
```

### DocumentId

```rust
/// Opaque identifier for an open document within the external modification system.
/// Maps 1:1 to a DocumentHandle in ff-document-model.
///
/// Used internally to decouple detection logic from the full document API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(u64);
```

---

## 5. Public API Surface

### Service Lifecycle

```rust
/// Create and start the external modification detection service.
///
/// Called during workbench initialization after VFS and document-model are ready.
pub fn create_detector(ctx: ExternalModContext) -> ExternalModificationDetector;
```

### ExternalModContext

```rust
/// Shared context holding references to upstream services.
pub struct ExternalModContext {
    /// VFS provider registry for stat() and watch() calls.
    pub vfs: Arc<ProviderRegistry>,
    /// Configuration access for external modification settings.
    pub config: Arc<dyn ConfigAccess>,
    /// Background I/O service for async reload operations.
    pub background_io: Arc<BackgroundIoService>,
    /// Dialog provider for shell-layer notifications.
    pub dialog_provider: Arc<dyn ExternalModDialogProvider>,
    /// Event bus for emitting document state change events.
    pub event_bus: Arc<dyn EventBus>,
}
```

### ExternalModificationDetector

```rust
/// The central service managing external modification detection for all open documents.
///
/// Addresses: Requirements 1–10
pub struct ExternalModificationDetector {
    // internal fields omitted
}

impl ExternalModificationDetector {
    /// Register a document for external modification tracking.
    /// Called when a document is opened/loaded from a VFS resource.
    ///
    /// Subscribes to VFS watch events and records the initial mtime.
    /// Addresses: Requirement 1 AC 2, Requirement 2 AC 2
    pub async fn register_document(
        &self,
        handle: DocumentHandle,
        uri: ResourceUri,
    ) -> Result<DocumentId, ExternalModError>;

    /// Unregister a document (called when document is closed).
    /// Cancels the VFS watch and removes all tracking state.
    ///
    /// Addresses: Requirement 1 AC 3
    pub async fn unregister_document(
        &self,
        doc_id: DocumentId,
    ) -> Result<(), ExternalModError>;

    /// Notify the detector that a document was saved.
    /// Updates the mtime snapshot to the post-save value.
    ///
    /// Addresses: Requirement 2 AC 3
    pub async fn notify_document_saved(
        &self,
        doc_id: DocumentId,
    ) -> Result<(), ExternalModError>;

    /// Trigger a focus-gained check on all open documents.
    /// Called by the shell layer when the application window gains focus.
    ///
    /// Addresses: Requirement 9, criteria 1–7
    pub async fn on_focus_gained(&self) -> Result<(), ExternalModError>;

    /// Trigger a single-document mtime check (tab-switch).
    /// Called when the user switches to a different document tab.
    ///
    /// Addresses: Requirement 9 AC 6
    pub async fn on_tab_activated(
        &self,
        doc_id: DocumentId,
    ) -> Result<(), ExternalModError>;

    /// Update configuration (hot-reload callback).
    /// Called when configuration values change at runtime.
    ///
    /// Addresses: Requirement 10 AC 8
    pub fn update_config(&self, new_config: ExternalModConfig);

    /// Shut down the detector, cancelling all watches and stopping the poller.
    pub async fn shutdown(&self);
}
```

### Shell Trait Abstractions

```rust
/// Trait abstraction for external modification dialogs.
/// The GUI shell (ff-desktop) provides the concrete implementation.
///
/// Addresses: GUI Independence cross-cutting requirement
#[async_trait::async_trait]
pub trait ExternalModDialogProvider: Send + Sync {
    /// Show a reload/keep/diff dialog for a content-changed document.
    ///
    /// Addresses: Requirement 4, criteria 1–8
    async fn show_reload_prompt(
        &self,
        file_name: &str,
        is_dirty: bool,
    ) -> ReloadAction;

    /// Show a notification for a deleted file.
    ///
    /// Addresses: Requirement 6, criteria 1–5
    async fn show_deleted_prompt(
        &self,
        file_name: &str,
        is_dirty: bool,
    ) -> DeleteAction;

    /// Show a notification for a renamed file.
    ///
    /// Addresses: Requirement 7, criteria 1–6
    async fn show_rename_prompt(
        &self,
        old_name: &str,
        new_name: &str,
        is_dirty: bool,
    ) -> RenameAction;

    /// Show a batch notification for multiple concurrent changes.
    ///
    /// Addresses: Requirement 8, criteria 1–7
    async fn show_batch_prompt(
        &self,
        notification: &BatchNotification,
    ) -> BatchAction;

    /// Show a brief status bar message (non-blocking).
    ///
    /// Addresses: Requirement 5 AC 3
    fn show_status_message(&self, message: &str, duration_secs: u32);
}
```

### Reload Operation API

```rust
/// Execute a reload for a document, respecting undo-preservation settings.
///
/// Addresses: Requirement 4 AC 4, AC 7, AC 8; Requirement 5 AC 1, AC 5
pub async fn reload_document(
    ctx: &ExternalModContext,
    doc_id: DocumentId,
    preserve_undo: bool,
) -> Result<(), ExternalModError>;

/// Execute a batch reload for all clean documents in a batch.
///
/// Addresses: Requirement 8 AC 5
pub async fn batch_reload_clean(
    ctx: &ExternalModContext,
    doc_ids: &[DocumentId],
    preserve_undo: bool,
) -> Result<BatchReloadResult, ExternalModError>;
```

### BatchReloadResult

```rust
/// Result of a batch reload operation.
///
/// Addresses: Requirement 8, criteria 4–5
#[derive(Debug)]
pub struct BatchReloadResult {
    /// Documents successfully reloaded.
    pub reloaded: Vec<DocumentId>,
    /// Documents skipped because they are dirty.
    pub skipped_dirty: Vec<DocumentId>,
    /// Documents that failed to reload (with error details).
    pub failed: Vec<(DocumentId, ExternalModError)>,
}
```

---

## 6. Error Types

```rust
/// Error type for all external modification detection failures.
///
/// Display format: `[external-mod] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExternalModError {
    /// VFS operation failed (watch, stat, or read).
    #[error("[external-mod] {operation}: VFS error for {uri} — {source}")]
    Vfs {
        operation: String,
        uri: ResourceUri,
        #[source]
        source: VfsError,
    },

    /// Document is not registered for external modification tracking.
    #[error("[external-mod] {operation}: document {doc_id:?} is not registered")]
    DocumentNotRegistered {
        operation: String,
        doc_id: DocumentId,
    },

    /// Watch registration failed and fallback polling also failed.
    #[error("[external-mod] watch: unable to monitor {uri} — watch unsupported and polling failed")]
    MonitoringUnavailable {
        uri: ResourceUri,
    },

    /// Reload operation failed (VFS read or document update error).
    #[error("[external-mod] reload: failed to reload {uri} — {reason}")]
    ReloadFailed {
        uri: ResourceUri,
        reason: String,
    },

    /// Configuration value is invalid (out of range, wrong type).
    #[error("[external-mod] config: invalid value for '{key}' — {reason}")]
    ConfigError {
        key: String,
        reason: String,
    },

    /// The detector has been shut down; operations are no longer accepted.
    #[error("[external-mod] {operation}: detector has been shut down")]
    Shutdown {
        operation: String,
    },
}
```

---

## 7. Integration Points

### Integration with `ff-vfs` (File Watcher)

| Operation | VFS API Used | Notes |
|-----------|-------------|-------|
| Register watch | `vfs.watch(uri, WatchOptions { recursive: false })` | Returns `WatchHandle` with async event stream |
| Cancel watch | `WatchHandle::cancel()` | On document close, deletion, or rename |
| Receive events | `WatchHandle::recv()` | Yields `WatchEvent::Modified`, `Deleted`, `Renamed`, `Created` |
| Query mtime | `vfs.stat(uri)` → `VfsMetadata::modified` | For snapshot comparison |
| Detect capability | Handle `VfsError::UnsupportedOperation` from `watch()` | Triggers fallback poller |

**Key constraint**: This crate does NOT call `std::fs::metadata()` or any direct FS API. All mtime queries go through `vfs.stat()`.

### Integration with `ff-document-model`

| Operation | Document API Used | Notes |
|-----------|------------------|-------|
| Query dirty state | `Document::is_modified()` | Determines auto-reload eligibility |
| Get document handle | `DocumentHandle` (Arc<RwLock<Document>>) | Shared ownership for reload |
| Replace content on reload | Delegated to `ff-file-ops::revert_file()` | Not called directly |
| Mark dirty (on deletion/keep-old-path) | `Document::mark_modified()` | Via command framework |
| Update backing URI (rename) | `Document::set_resource_uri(new_uri)` | Via command framework |
| Preserve viewport on auto-reload | Read `Document::viewport()` before, restore after | Requirement 5 AC 2 |

### Integration with `ff-file-ops`

| Operation | File-Ops API Used | Notes |
|-----------|-------------------|-------|
| Reload content | `revert_file(ctx, document)` | Handles full reload flow including undo |
| Save As (deletion) | `save_file_as(ctx, document, options)` | User chooses new location after deletion |

The `ff-external-mod` crate does NOT directly read file content or write to disk — all reload mechanics are delegated to `ff-file-ops` which handles the VFS read, buffer replacement, and undo-point management.

### Integration with `ff-background-io`

| Scenario | Background-IO API | Notes |
|----------|-------------------|-------|
| Large file reload | `BackgroundIoService::spawn_load(uri, options)` | Auto-reload of large files uses async path |
| Cancel pending reload | `IoTaskHandle::cancel()` | If user starts editing before reload completes (Req 5 AC 6) |
| Progress | `IoTaskHandle::progress()` | Status bar shows reload progress for large files |

### Integration with `ff-config` (Configuration System)

| Setting Key | Type | Default | Range | Purpose |
|-------------|------|---------|-------|---------|
| `editor.external_modification.policy` | String | `"prompt"` | prompt/auto/ignore | Reload policy |
| `editor.external_modification.reload_preserves_undo` | bool | `false` | — | Undo history across reloads |
| `editor.external_modification.check_on_focus` | bool | `true` | — | Focus-gained mtime scan |
| `editor.external_modification.auto_follow_rename` | bool | `false` | — | Auto-follow renames |
| `editor.external_modification.batch_debounce_ms` | u64 | `500` | 100–5000 | Batch coalescing window |
| `editor.external_modification.polling_interval_ms` | u64 | `5000` | 1000–60000 | Fallback polling interval |

**Hot-reload**: All settings are reloaded via the configuration system's `Reload_Callback` mechanism. The detector's `update_config()` method is registered as a callback and applies changes immediately.

**Clamping**: Out-of-range values are clamped to the nearest valid bound with a WARN-level log record (Requirement 10 AC 9).

### Integration with `ff-logging`

| Level | Usage |
|-------|-------|
| DEBUG | Watch registration/cancellation, mtime comparisons, policy decisions |
| INFO | Fallback to polling (Req 1 AC 5), auto-reload completed, config hot-reload applied |
| WARN | Stat failure (Req 2 AC 8), config value clamped (Req 10 AC 9), reload failure |
| ERROR | Critical failures: watch system unavailable, detector shutdown errors |

---

## 8. Correctness Properties

These properties define invariants that property-based tests should verify:

### Property 1: Mtime Snapshot Consistency

**Statement**: For any registered document, after a successful load, save, or reload, the stored `Mtime_Snapshot` SHALL equal the mtime returned by `vfs.stat()` for that document's backing URI at the time of the operation.

**Validates**: Requirement 2 AC 1, AC 2, AC 3

### Property 2: No Duplicate Prompts

**Statement**: For any document, the system SHALL emit at most one `ExternalChange` event per distinct mtime value. Once a user has been prompted (or auto-action taken) for a given mtime, the same mtime SHALL NOT trigger another notification until a new distinct mtime is detected.

**Validates**: Requirement 3 AC 6, Requirement 9 AC 7

### Property 3: Dirty Buffer Safety

**Statement**: A document with `is_modified() == true` (dirty buffer) SHALL NEVER be auto-reloaded without user confirmation, regardless of the configured `ReloadPolicy`.

**Validates**: Requirement 3 AC 4, Requirement 5 AC 6

### Property 4: Batch Coalescing Completeness

**Statement**: All `ExternalChange` events received within a single debounce window SHALL be included in exactly one `BatchNotification`. No events SHALL be lost or duplicated across batch boundaries.

**Validates**: Requirement 8 AC 1, AC 7

### Property 5: Watch Lifecycle Correctness

**Statement**: For every `register_document()` call that succeeds, there SHALL be exactly one active VFS watch (or one active poll registration). For every `unregister_document()` call, the corresponding watch SHALL be cancelled and no further events processed for that document.

**Validates**: Requirement 1 AC 2, AC 3

### Property 6: Spurious Event Rejection

**Statement**: A VFS `Modified` event whose associated `vfs.stat()` returns an mtime equal to the stored `Mtime_Snapshot` SHALL be discarded without emitting any `ExternalChange` event or user notification.

**Validates**: Requirement 2 AC 6

### Property 7: Policy Determinism

**Statement**: Given the same inputs (ReloadPolicy, is_dirty, change_type), the `PolicyEngine` SHALL always produce the same action outcome. The mapping is:
- `Ignore` → always suppress notification, update snapshot
- `Auto` + not dirty → always auto-reload
- `Auto` + dirty → always prompt
- `Prompt` → always prompt

**Validates**: Requirement 3 AC 2, AC 3, AC 4, AC 5

### Property 8: Configuration Clamping Invariant

**Statement**: After applying configuration, `batch_debounce_ms` SHALL always be in [100, 5000] and `polling_interval_ms` SHALL always be in [1000, 60000], regardless of the raw input value provided.

**Validates**: Requirement 10 AC 9

### Property 9: Focus-Gained Idempotency

**Statement**: Calling `on_focus_gained()` multiple times in rapid succession (without intervening file changes) SHALL produce the same result as a single call — no duplicate prompts, no duplicate reloads.

**Validates**: Requirement 9 AC 7

### Property 10: Deletion Orphans Buffer

**Statement**: After a `FileDeleted` event is processed and the user selects "Keep Editing", the document SHALL have no backing URI, SHALL be marked dirty, and the VFS watch for the original URI SHALL be cancelled.

**Validates**: Requirement 6 AC 3, AC 5

---

## 9. Testing Strategy

### Unit Test Coverage

| Module | Key Scenarios |
|--------|---------------|
| `mtime.rs` | Snapshot creation, comparison (equal, different, sub-second precision), invalid snapshot handling |
| `policy.rs` | All policy × dirty × change-type combinations, config hot-reload transitions |
| `batch.rs` | Single event passthrough, multiple events within window, events spanning windows, dirty exclusion |
| `focus.rs` | All-clear scan, single change detected, multiple changes batch, config disabled |
| `poller.rs` | Timer fires at correct interval, config change adjusts interval, cancellation on unregister |
| `detector.rs` | Register/unregister lifecycle, concurrent event processing, shutdown behaviour |
| `events.rs` | ExternalChange construction, BatchNotification aggregation |
| `config.rs` | Valid values pass through, out-of-range values clamped, type mismatches reported |

### Property-Based Test Focus

- **Mtime tracking** (Property 1): Generate random sequences of load/save/external-change events; verify snapshot always matches final stat result
- **Batch coalescing** (Property 4): Generate random event streams with timestamps; verify partitioning into batches is correct and lossless
- **Policy determinism** (Property 7): Generate all combinations of (policy, dirty, change_type); verify output is deterministic
- **Config clamping** (Property 8): Generate arbitrary u64 values; verify post-clamp is always within valid range
- **Spurious rejection** (Property 6): Generate events where stat mtime equals snapshot; verify zero ExternalChange emissions

### Integration Test Scenarios

1. **Happy path**: Open file → external modification → prompt → reload → verify content updated
2. **Auto-reload clean buffer**: policy=auto, clean buffer → verify silent reload + status message
3. **Dirty buffer protection**: policy=auto, dirty buffer → verify prompt shown, not auto-reloaded
4. **File deletion flow**: Delete backing file → verify notification → Keep Editing → verify orphaned state
5. **File rename flow**: Rename backing file → Follow Rename → verify URI updated + watch re-registered
6. **Batch coalescing**: git checkout modifying 5 open files → verify single batch prompt
7. **Focus-gained catch-up**: Miss VFS events → focus gained → verify changes detected via stat
8. **Fallback polling**: Provider without watch support → verify polling at configured interval
9. **Config hot-reload**: Change policy at runtime → verify new policy applied to next event

### Testing Framework

- **Unit tests**: `#[cfg(test)] mod tests` in each module
- **Property tests**: `proptest` crate with minimum 100 iterations
- **Mock infrastructure**: Mock `VfsProvider` with controllable stat/watch behaviour, mock `ExternalModDialogProvider` with scripted responses
