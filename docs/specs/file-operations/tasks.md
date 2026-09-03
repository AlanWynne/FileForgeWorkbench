# Implementation Plan: File Operations (`ff-file-ops`)

## Overview

This plan covers the complete implementation of the `ff-file-ops` crate — the user-facing file operation commands for FileForgeWorkbench. The crate provides New, Open, Save, Save As, Revert, and Recent Files commands, plus the underlying persistence mechanisms (atomic rename-on-write, backup copies, read-only detection). All file I/O routes through the VFS abstraction layer (`ff-vfs`), and all commands are dispatched through the command framework (`ff-command`).

This is a **Wave 8 (File I/O and Session)** sub-project. It depends on `ff-vfs` (VFS abstraction), `ff-command` (command registration and dispatch), `ff-document` (document model), `ff-undo` (undo/redo transactions), `ff-config` (configuration system), and `ff-logging` (diagnostics).

---

## Tasks

- [x] 1. Crate scaffolding and core types
  - [x] 1.1 Create `crates/ff-file-ops/Cargo.toml` with dependencies (ff-vfs, ff-command, ff-document, ff-undo, ff-config, ff-logging, thiserror, serde, tokio, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-file-ops/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `resource_uri.rs`, `options.rs`, `save_strategy.rs`, `backup.rs`, `recent_files.rs`, `read_only.rs`, `unsaved_guard.rs`, `commands.rs`, `error.rs`
  - [x] 1.4 Add `ff-file-ops` to workspace `Cargo.toml` members list
  - [x] 1.5 Define `FileOpsError` enum with variants: VfsReadError, VfsWriteError, AtomicRenameFailed, BackupFailed, ResourceNotFound, PermissionDenied, ProviderUnavailable, ReadOnlyResource, SaveInProgress, UserCancelled, InvalidUri
  - [x] 1.6 Implement `Display` and `thiserror::Error` derives with descriptive messages for all error variants
  - Covers: Structural foundation for all requirements

- [x] 2. Core types — ResourceUri integration and options structs
  - [x] 2.1 Define `FileOpenOptions` struct with fields: uri (ResourceUri), encoding (Option), read_only_override (Option<bool>), activate_tab (bool)
  - [x] 2.2 Define `FileSaveOptions` struct with fields: uri (ResourceUri), strategy (SaveStrategy), create_backup (bool), async_threshold_bytes (u64), check_modified_time (bool)
  - [x] 2.3 Define `SaveStrategy` enum with variants: AtomicRename, DeleteFirst, Direct
  - [x] 2.4 Define `SaveResult` struct with fields: uri (ResourceUri), bytes_written (u64), modification_time (SystemTime), was_async (bool)
  - [x] 2.5 Define `FilePickerMode` enum with variants: Open, Save, and `FilePickerOptions` struct for dialog configuration
  - [x] 2.6 Implement `Default` for `FileSaveOptions` (AtomicRename strategy, backup from config, async threshold from config)
  - [x] 2.7 Write unit tests for options construction, defaults, and builder patterns
  - Covers: Requirement 1 (AC 1.1, 1.6, 1.7), Requirement 2 (AC 2.1, 2.7), Requirement 7 (AC 7.1, 7.6, 7.7)

- [x] 3. Atomic write implementation
  - [x] 3.1 Implement `AtomicWriter` struct encapsulating the write-to-temp + rename strategy
  - [x] 3.2 Implement `AtomicWriter::write(uri, content, vfs) -> Result<SaveResult>` — create temp file in same directory, write content, flush, fsync, atomic rename
  - [x] 3.3 Implement temp file naming: target filename with `.tmp` suffix appended (e.g., `file.txt.tmp`)
  - [x] 3.4 Implement flush and fsync via VFS provider API before rename
  - [x] 3.5 Implement atomic rename via VFS `rename` operation
  - [x] 3.6 Implement fallback for providers without rename support — direct overwrite with flush+fsync and WARN log
  - [x] 3.7 Implement `delete_first` strategy — delete target then write new content
  - [x] 3.8 Implement `direct` strategy — write content directly to target without temp file
  - [x] 3.9 Implement temp file cleanup — remove temp files left by interrupted writes (on startup or on error)
  - [x] 3.10 Write unit tests for each strategy (atomic rename, delete_first, direct, fallback) using mock VFS
  - Covers: Requirement 7 (AC 7.1, 7.2, 7.6, 7.7, 7.8, 7.9)

- [x] 4. Backup copy mechanism
  - [x] 4.1 Define `BackupConfig` struct with fields: enabled (bool), location (BackupLocation), suffix (String)
  - [x] 4.2 Define `BackupLocation` enum with variants: Alongside, Directory(PathBuf)
  - [x] 4.3 Implement `BackupManager::create_backup(uri, vfs, config) -> Result<()>` — copy original resource before overwrite
  - [x] 4.4 Implement "alongside" backup — same directory with configured suffix (default `.bak`)
  - [x] 4.5 Implement "directory" backup — preserve relative structure within dedicated backup directory
  - [x] 4.6 Implement graceful failure — log WARN on backup failure but do not abort the save operation
  - [x] 4.7 Write unit tests for both backup locations, suffix configuration, and failure tolerance
  - Covers: Requirement 7 (AC 7.3, 7.4, 7.5)

- [x] 5. Open command implementation
  - [x] 5.1 Implement `OpenCommand` handler struct implementing `CommandHandler` trait
  - [x] 5.2 Implement URI argument extraction — open directly when URI provided via params
  - [x] 5.3 Implement File_Picker integration — open dialog in open mode when no URI argument
  - [x] 5.4 Implement VFS read — load resource content via `read_stream` API
  - [x] 5.5 Implement document creation — construct new Document from loaded content with encoding detection
  - [x] 5.6 Implement tab integration — create new tab for opened document
  - [x] 5.7 Implement duplicate detection — activate existing tab if resource already open
  - [x] 5.8 Implement multi-select support — open multiple URIs in single invocation, one tab per resource
  - [x] 5.9 Implement read-only detection on open — query VFS metadata and config for write permission
  - [x] 5.10 Implement modification time recording on open — store VFS stat mtime for later external-modification detection
  - [x] 5.11 Implement error handling — display notification on VFS read failure, do not create tab
  - [x] 5.12 Write unit tests for URI-based open, picker-based open, duplicate detection, multi-open, read-only detection, and error paths
  - Covers: Requirement 4 (AC 4.1–4.10)

- [x] 6. Save command implementation
  - [x] 6.1 Implement `SaveCommand` handler struct implementing `CommandHandler` trait
  - [x] 6.2 Implement save-to-existing-URI path — write via VFS using configured SaveStrategy
  - [x] 6.3 Implement untitled-document delegation — invoke `file.save_as` when document has no URI
  - [x] 6.4 Implement save-point marking — update undo/redo transaction save-point and clear dirty flag on success
  - [x] 6.5 Implement modification time update — record new mtime from VFS stat after successful write
  - [x] 6.6 Implement sync save path — for documents at or below Async_Save_Threshold, block until complete
  - [x] 6.7 Implement async save path — for documents above threshold, spawn background task with progress indication
  - [x] 6.8 Implement concurrent-save guard — reject second save if one is already in progress, notify user
  - [x] 6.9 Implement external-modification check — when enabled, compare mtime before write and prompt user
  - [x] 6.10 Implement `file.saved` event emission on success via command framework event bus
  - [x] 6.11 Implement error handling — preserve dirty state and modifications on failure, emit error notification
  - [x] 6.12 Write unit tests for all save paths (existing URI, untitled delegation, sync, async, concurrent guard, mtime check, error)
  - Covers: Requirement 1 (AC 1.1–1.10)

- [x] 7. Save As command implementation
  - [x] 7.1 Implement `SaveAsCommand` handler struct implementing `CommandHandler` trait
  - [x] 7.2 Implement File_Picker integration — open dialog in save mode, pre-populated with current directory
  - [x] 7.3 Implement URI argument path — write directly to provided URI without picker
  - [x] 7.4 Implement overwrite confirmation — prompt when target URI refers to existing resource
  - [x] 7.5 Implement document URI reassignment — update associated URI to new target on success
  - [x] 7.6 Implement save-point marking and dirty flag clearing on success
  - [x] 7.7 Implement tab title and window title update to reflect new resource name
  - [x] 7.8 Implement Recent Files update — add new URI to list on success
  - [x] 7.9 Implement cancellation handling — no-op when user cancels picker or overwrite dialog
  - [x] 7.10 Implement error handling — preserve original URI and dirty state on failure
  - [x] 7.11 Implement availability regardless of dirty state — allow saving clean document to new location
  - [x] 7.12 Write unit tests for picker path, URI argument path, overwrite confirmation, URI reassignment, cancellation, and error handling
  - Covers: Requirement 2 (AC 2.1–2.10)

- [x] 8. New command implementation
  - [x] 8.1 Implement `NewCommand` handler struct implementing `CommandHandler` trait
  - [x] 8.2 Implement new document creation — empty Document with no URI, empty undo stack, default encoding
  - [x] 8.3 Implement new tab creation — open new tab with empty document
  - [x] 8.4 Implement sequential untitled naming — assign "Untitled-1", "Untitled-2", etc.
  - [x] 8.5 Implement status bar update — show "(Untitled)", dirty indicator off, cursor at line 1 col 1
  - [x] 8.6 Implement unsaved-changes guard — invoke Unsaved_Changes_Dialog when active document is dirty
  - [x] 8.7 Implement "Save" dialog response — save then proceed with new document creation
  - [x] 8.8 Implement "Discard" dialog response — discard modifications and create new document
  - [x] 8.9 Implement "Cancel" dialog response — abort new operation, return to current document
  - [x] 8.10 Implement save-failure abort — if save fails during dialog flow, abandon New operation
  - [x] 8.11 Write unit tests for clean-document new, dirty-document dialog flows (Save/Discard/Cancel), sequential naming
  - Covers: Requirement 3 (AC 3.1–3.8)

- [x] 9. Revert command implementation
  - [x] 9.1 Implement `RevertCommand` handler struct implementing `CommandHandler` trait
  - [x] 9.2 Implement confirmation dialog — warn user that all changes will be lost (when document is dirty)
  - [x] 9.3 Implement no-confirmation path — reload immediately when document has no unsaved changes
  - [x] 9.4 Implement VFS reload — re-read resource content and replace document buffer entirely
  - [x] 9.5 Implement post-reload state reset — clear dirty flag, reset undo/redo stacks, reset viewport to line 1
  - [x] 9.6 Implement modification time update after reload
  - [x] 9.7 Implement status message display — "Reverted to saved"
  - [x] 9.8 Implement disabled state for untitled documents — command not executable when no URI
  - [x] 9.9 Implement async reload for large files with progress indication
  - [x] 9.10 Implement error handling — display error notification on VFS read failure, preserve current state
  - [x] 9.11 Implement cancellation handling — no-op when user cancels confirmation dialog
  - [x] 9.12 Write unit tests for dirty-revert, clean-revert, untitled-disabled, async reload, error, and cancellation paths
  - Covers: Requirement 5 (AC 5.1–5.9)

- [x] 10. Recent Files list management
  - [x] 10.1 Define `RecentFilesList` struct with bounded, ordered storage of ResourceUri entries
  - [x] 10.2 Implement `add(uri)` — add to top, deduplicate existing same-URI entry, evict oldest when over max
  - [x] 10.3 Implement `remove(uri)` — remove specific entry (for inaccessible resources)
  - [x] 10.4 Implement `list() -> Vec<ResourceUri>` — return ordered list (most recent first)
  - [x] 10.5 Implement configurable max count from `file.recent_files.max_count` (default 10)
  - [x] 10.6 Implement persistence — serialize list to user-level config store asynchronously on modification
  - [x] 10.7 Implement startup loading — deserialize list from persisted config on workbench start
  - [x] 10.8 Implement graceful degradation — initialize empty on missing or invalid persisted data, no error
  - [x] 10.9 Implement `OpenRecentCommand` handler — open selected URI via `file.open` semantics
  - [x] 10.10 Implement inaccessible-resource handling — error notification and list removal when resource no longer exists
  - [x] 10.11 Implement full ResourceUri storage — no bare paths, preserve provider-specific URIs
  - [x] 10.12 Write unit tests for add/deduplicate, eviction, persistence round-trip, startup loading, graceful degradation, and inaccessible removal
  - Covers: Requirement 6 (AC 6.1–6.10)

- [x] 11. Read-only detection and enforcement
  - [x] 11.1 Implement VFS-based read-only detection — query provider capabilities and resource metadata on open
  - [x] 11.2 Implement configuration-based read-only — check `read.only` property per file-pattern matching
  - [x] 11.3 Implement provider-level read-only — mark all documents from write-incapable providers as read-only
  - [x] 11.4 Implement mutation prevention — silently reject all buffer mutations (insert, delete, paste, undo) with status notification
  - [x] 11.5 Implement visual indicators — status bar lock icon/`[RO]` suffix on tab
  - [x] 11.6 Implement `file.toggle_read_only` command — manual override of detected state
  - [x] 11.7 Implement save-time warning — warn when user saves a toggled-writable document to a VFS-reported read-only resource
  - [x] 11.8 Write unit tests for VFS detection, config detection, provider-level detection, mutation blocking, toggle override, and save-time warning
  - Covers: Requirement 8 (AC 8.1–8.7)

- [x] 12. Unsaved-changes guard
  - [x] 12.1 Define `UnsavedChangesGuard` trait and default implementation for reusable dialog logic
  - [x] 12.2 Implement three-option dialog: Save, Discard, Cancel — with document name displayed prominently
  - [x] 12.3 Implement "Save" path — invoke save, proceed on success, abort on failure
  - [x] 12.4 Implement "Discard" path — proceed immediately without saving
  - [x] 12.5 Implement "Cancel" path — abort the calling operation entirely
  - [x] 12.6 Implement batch mode — "Save All / Discard All / Cancel" for Exit and Close All operations
  - [x] 12.7 Implement `file.unsaved_prompt` configuration toggle — when false, skip dialog and proceed
  - [x] 12.8 Implement integration with New, Open, Revert, Close, and Exit operations
  - [x] 12.9 Write unit tests for each dialog response path, batch mode, and configuration bypass
  - Covers: Requirement 9 (AC 9.1–9.8)

- [x] 13. Command registration and menu integration
  - [x] 13.1 Register all file commands with IDs: `file.new`, `file.open`, `file.open_recent`, `file.save`, `file.save_as`, `file.revert`, `file.close`, `file.exit`
  - [x] 13.2 Implement command metadata — display name, description, category "file", default shortcuts, enabled predicates
  - [x] 13.3 Implement default keyboard shortcuts: New=Ctrl+N, Open=Ctrl+O, Save=Ctrl+S, SaveAs=Ctrl+Shift+S, Close=Ctrl+W, Exit=Alt+F4
  - [x] 13.4 Implement enabled-state predicates: `file.revert` disabled for untitled; `file.save` disabled when clean+has-URI
  - [x] 13.5 Implement menu layout contribution: New, Open, Recent Files (submenu), separator, Save, Save As, separator, Revert, separator, Close, Exit
  - [x] 13.6 Implement ISPF command line aliases: NEW, OPEN, SAVE, SAVEAS, REVERT
  - [x] 13.7 Implement event emission: `file.opened`, `file.saved`, `file.new_created`, `file.reverted` events via command framework
  - [x] 13.8 Implement `file.toggle_read_only` command registration with metadata
  - [x] 13.9 Write unit tests for registration, metadata correctness, predicate evaluation, and event emission
  - Covers: Requirement 10 (AC 10.1–10.8)

- [x] 14. Property-based tests
  - [x] 14.1 Write PBT: Atomic write crash safety property
  - [x] 14.2 Write PBT: Recent Files bounded-list invariant property
  - [x] 14.3 Write PBT: Recent Files deduplication property
  - [x] 14.4 Write PBT: Save-point dirty flag consistency property
  - [x] 14.5 Write PBT: Read-only mutation rejection property
  - [x] 14.6 Write PBT: Unsaved-changes guard completeness property
  - [x] 14.7 Write PBT: Backup copy creation property
  - [x] 14.8 Write PBT: Save strategy selection property
  - Covers: All requirements (property-based validation)

- [x] 15. Integration tests
  - [x] 15.1 Write integration test: full open-edit-save cycle via mock VFS
  - [x] 15.2 Write integration test: Save As with URI reassignment and Recent Files update
  - [x] 15.3 Write integration test: New with unsaved-changes guard (all three responses)
  - [x] 15.4 Write integration test: Revert with undo stack clearing verification
  - [x] 15.5 Write integration test: Read-only document open and mutation rejection
  - [x] 15.6 Write integration test: Recent Files persistence round-trip across startup/shutdown
  - [x] 15.7 Write integration test: Atomic write failure and state preservation
  - [x] 15.8 Write integration test: Concurrent save rejection
  - [x] 15.9 Write integration test: External modification detection prompt during save
  - [x] 15.10 Write integration test: Command registration and dispatch for all file commands
  - Covers: Cross-requirement interaction validation

---

## Property-Based Test Definitions

### Property 1: Atomic Write Crash Safety

**Validates: Requirement 7.1, 7.2**

- **Statement:** For any document content and target URI, an atomic write either fully succeeds (target contains exact new content) or fully fails (target retains its original content unchanged). No partial writes are observable.
- **Strategy:** Generate:
  - Document content: byte vectors of length 0–100,000
  - Simulated failures: inject failure at write-to-temp, at fsync, at rename (each independently)
  - Original target content: random byte vectors (to verify preservation)
- **Invariant:** After operation, `vfs.read(target) == new_content` (success) OR `vfs.read(target) == original_content` (failure). Never a mix.

### Property 2: Recent Files Bounded-List Invariant

**Validates: Requirement 6.2, 6.4**

- **Statement:** For any sequence of add/remove operations on the Recent Files list with a configured maximum of N entries, the list length never exceeds N.
- **Strategy:** Generate:
  - Max count: integer in [1, 50]
  - Operation sequence: 10–200 operations mixing Add(uri) and Remove(uri) from a pool of 5–30 URIs
- **Invariant:** `recent_files.len() <= max_count` after every operation

### Property 3: Recent Files Deduplication

**Validates: Requirement 6.3**

- **Statement:** The Recent Files list never contains duplicate URIs. Adding a URI that already exists moves it to the top without creating a second entry.
- **Strategy:** Generate:
  - URI pool: 3–10 unique URIs
  - Add sequence: 20–100 random selections from the pool (with repeats)
- **Invariant:** After every add, `recent_files.list()` contains no duplicates; `recent_files.list()[0] == last_added_uri`

### Property 4: Save-Point Dirty Flag Consistency

**Validates: Requirement 1.2, 5.3**

- **Statement:** A document's dirty flag is `true` if and only if the undo history position differs from the save-point position. After a successful save, dirty is `false`. After any edit, dirty is `true`. After undo back to save-point, dirty is `false` again.
- **Strategy:** Generate:
  - Operation sequences: 5–50 operations from {Edit, Save, Undo, Redo}
  - Track save-point and current position in model
- **Invariant:** `document.is_dirty() == (current_undo_position != save_point_position)` after every operation

### Property 5: Read-Only Mutation Rejection

**Validates: Requirement 8.2**

- **Statement:** When a document is marked read-only, all mutation operations (insert, delete, paste, line commands) are rejected without modifying the buffer. The buffer content remains identical before and after the rejected operation.
- **Strategy:** Generate:
  - Initial document content: strings of 0–1000 characters
  - Mutation operations: Insert(pos, text), Delete(range), Paste(pos, text)
  - Mark document read-only, then apply all operations
- **Invariant:** `document.content() == initial_content` after all mutation attempts on a read-only document

### Property 6: Unsaved-Changes Guard Completeness

**Validates: Requirement 9.1**

- **Statement:** For any operation that would discard unsaved modifications (New, Open, Revert, Close, Exit), the unsaved-changes dialog is always presented when the document is dirty and `file.unsaved_prompt` is true. When dirty is false or prompt is disabled, no dialog is shown.
- **Strategy:** Generate:
  - Document dirty state: true/false
  - Prompt config: enabled/disabled
  - Operation type: {New, Open, Revert, Close, Exit}
  - Dialog response (when shown): {Save, Discard, Cancel}
- **Invariant:** Dialog shown ⟺ (dirty == true AND prompt_enabled == true). When Cancel selected, operation is aborted and document is unchanged.

### Property 7: Backup Copy Creation

**Validates: Requirement 7.3, 7.4, 7.5**

- **Statement:** When backup is enabled, a save operation creates a backup copy of the original content before overwriting. Backup failure never aborts the save. When backup is disabled, no backup is created.
- **Strategy:** Generate:
  - Backup enabled: true/false
  - Backup location: Alongside/Directory
  - Original content: byte vectors of 0–10,000 bytes
  - New content: byte vectors of 0–10,000 bytes
  - Simulated backup failure: true/false
- **Invariant:** When enabled and backup succeeds, `vfs.read(backup_path) == original_content`. When backup fails, save still completes (target has new content). When disabled, no backup file exists.

### Property 8: Save Strategy Selection

**Validates: Requirement 7.1, 7.6, 7.7**

- **Statement:** The save operation uses exactly the strategy configured for the current provider and settings. AtomicRename uses a temp file and rename. DeleteFirst deletes then writes. Direct writes in-place. The strategy never changes mid-operation.
- **Strategy:** Generate:
  - Strategy: {AtomicRename, DeleteFirst, Direct}
  - Document content: byte vectors of 0–50,000 bytes
  - Provider rename support: true/false
- **Invariant:** VFS operation sequence matches the selected strategy exactly. When AtomicRename is selected but provider lacks rename, falls back to Direct with WARN log (never DeleteFirst).

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2"], "dependsOn": [0] },
    { "id": 2, "label": "Persistence Mechanisms", "tasks": ["3", "4"], "dependsOn": [1] },
    { "id": 3, "label": "Open Command", "tasks": ["5"], "dependsOn": [1] },
    { "id": 4, "label": "Save Commands", "tasks": ["6", "7"], "dependsOn": [2] },
    { "id": 5, "label": "New and Revert Commands", "tasks": ["8", "9"], "dependsOn": [3, 4] },
    { "id": 6, "label": "Recent Files", "tasks": ["10"], "dependsOn": [3, 4] },
    { "id": 7, "label": "Read-Only and Unsaved Guard", "tasks": ["11", "12"], "dependsOn": [3, 4, 5] },
    { "id": 8, "label": "Command Registration", "tasks": ["13"], "dependsOn": [4, 5, 6, 7] },
    { "id": 9, "label": "Property-Based Tests", "tasks": ["14"], "dependsOn": [2, 6, 7] },
    { "id": 10, "label": "Integration Tests", "tasks": ["15"], "dependsOn": [8, 9] }
  ]
}
```

---

## Notes

- This is a Wave 8 (File I/O and Session) crate depending on `ff-vfs` (Wave 3), `ff-command` (Wave 2), `ff-document` (Wave 4), `ff-undo` (Wave 4), `ff-config` (Wave 2), and `ff-logging` (Wave 0)
- All file I/O operations go through the VFS abstraction — no direct `std::fs` or `tokio::fs` calls allowed (FFW-ARCH-001)
- The `background-io` crate (sibling Wave 8) handles async I/O infrastructure; `ff-file-ops` uses its async threshold and background task spawning APIs
- The `encoding-and-characters` crate (sibling Wave 8) provides encoding detection; `ff-file-ops` delegates encoding concerns to it during open/save
- The `multi-tab-editor` crate (sibling Wave 8) manages tab lifecycle; `ff-file-ops` integrates via tab creation/activation APIs
- File_Picker is abstracted as a trait to maintain GUI independence — concrete implementations live in the GUI shell crate
- The Unsaved_Changes_Dialog is similarly abstracted — `ff-file-ops` defines the dialog contract, GUI shell provides the presentation
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Mock VFS implementations are used extensively in unit and integration tests to simulate various provider capabilities and failure modes
- The `file.close` and `file.exit` commands are registered by this crate but their full implementation involves coordination with the tab manager and application lifecycle — only the unsaved-changes guard portion lives here
- External-modification detection (`save.check_modified_time`) uses mtime comparison against the recorded value from the last open/save — no file watching is performed by this crate (that's `connector-local-fs` responsibility)

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Save | AC 1.1 | Tasks 3, 6 |
| Req 1: Save | AC 1.2 | Task 6 |
| Req 1: Save | AC 1.3 | Task 6 |
| Req 1: Save | AC 1.4 | Task 6 |
| Req 1: Save | AC 1.5 | Task 6 |
| Req 1: Save | AC 1.6 | Tasks 2, 6 |
| Req 1: Save | AC 1.7 | Tasks 2, 6 |
| Req 1: Save | AC 1.8 | Task 6 |
| Req 1: Save | AC 1.9 | Task 6 |
| Req 1: Save | AC 1.10 | Tasks 6, 13 |
| Req 2: Save As | AC 2.1 | Tasks 2, 7 |
| Req 2: Save As | AC 2.2 | Task 7 |
| Req 2: Save As | AC 2.3 | Task 7 |
| Req 2: Save As | AC 2.4 | Task 7 |
| Req 2: Save As | AC 2.5 | Task 7 |
| Req 2: Save As | AC 2.6 | Task 7 |
| Req 2: Save As | AC 2.7 | Tasks 2, 7 |
| Req 2: Save As | AC 2.8 | Task 7 |
| Req 2: Save As | AC 2.9 | Tasks 7, 10 |
| Req 2: Save As | AC 2.10 | Task 7 |
| Req 3: New File | AC 3.1 | Task 8 |
| Req 3: New File | AC 3.2 | Task 8 |
| Req 3: New File | AC 3.3 | Task 8 |
| Req 3: New File | AC 3.4 | Task 8 |
| Req 3: New File | AC 3.5 | Task 8 |
| Req 3: New File | AC 3.6 | Task 8 |
| Req 3: New File | AC 3.7 | Task 8 |
| Req 3: New File | AC 3.8 | Task 8 |
| Req 4: Open | AC 4.1 | Task 5 |
| Req 4: Open | AC 4.2 | Task 5 |
| Req 4: Open | AC 4.3 | Tasks 5, 12 |
| Req 4: Open | AC 4.4 | Task 5 |
| Req 4: Open | AC 4.5 | Task 5 |
| Req 4: Open | AC 4.6 | Task 5 |
| Req 4: Open | AC 4.7 | Tasks 5, 11 |
| Req 4: Open | AC 4.8 | Task 5 |
| Req 4: Open | AC 4.9 | Tasks 5, 10 |
| Req 4: Open | AC 4.10 | Task 5 |
| Req 5: Revert | AC 5.1 | Task 9 |
| Req 5: Revert | AC 5.2 | Task 9 |
| Req 5: Revert | AC 5.3 | Task 9 |
| Req 5: Revert | AC 5.4 | Task 9 |
| Req 5: Revert | AC 5.5 | Task 9 |
| Req 5: Revert | AC 5.6 | Tasks 9, 13 |
| Req 5: Revert | AC 5.7 | Task 9 |
| Req 5: Revert | AC 5.8 | Task 9 |
| Req 5: Revert | AC 5.9 | Task 9 |
| Req 6: Recent Files | AC 6.1 | Task 10 |
| Req 6: Recent Files | AC 6.2 | Task 10 |
| Req 6: Recent Files | AC 6.3 | Task 10 |
| Req 6: Recent Files | AC 6.4 | Task 10 |
| Req 6: Recent Files | AC 6.5 | Task 10 |
| Req 6: Recent Files | AC 6.6 | Task 10 |
| Req 6: Recent Files | AC 6.7 | Task 10 |
| Req 6: Recent Files | AC 6.8 | Task 10 |
| Req 6: Recent Files | AC 6.9 | Task 10 |
| Req 6: Recent Files | AC 6.10 | Task 10 |
| Req 7: Atomic Write | AC 7.1 | Task 3 |
| Req 7: Atomic Write | AC 7.2 | Task 3 |
| Req 7: Atomic Write | AC 7.3 | Task 4 |
| Req 7: Atomic Write | AC 7.4 | Task 4 |
| Req 7: Atomic Write | AC 7.5 | Task 4 |
| Req 7: Atomic Write | AC 7.6 | Task 3 |
| Req 7: Atomic Write | AC 7.7 | Task 3 |
| Req 7: Atomic Write | AC 7.8 | Task 3 |
| Req 7: Atomic Write | AC 7.9 | Task 3 |
| Req 8: Read-Only | AC 8.1 | Task 11 |
| Req 8: Read-Only | AC 8.2 | Task 11 |
| Req 8: Read-Only | AC 8.3 | Task 11 |
| Req 8: Read-Only | AC 8.4 | Task 11 |
| Req 8: Read-Only | AC 8.5 | Tasks 11, 13 |
| Req 8: Read-Only | AC 8.6 | Task 11 |
| Req 8: Read-Only | AC 8.7 | Task 11 |
| Req 9: Unsaved Guard | AC 9.1 | Task 12 |
| Req 9: Unsaved Guard | AC 9.2 | Task 12 |
| Req 9: Unsaved Guard | AC 9.3 | Task 12 |
| Req 9: Unsaved Guard | AC 9.4 | Task 12 |
| Req 9: Unsaved Guard | AC 9.5 | Task 12 |
| Req 9: Unsaved Guard | AC 9.6 | Task 12 |
| Req 9: Unsaved Guard | AC 9.7 | Task 12 |
| Req 9: Unsaved Guard | AC 9.8 | Task 12 |
| Req 10: Menu & Commands | AC 10.1 | Task 13 |
| Req 10: Menu & Commands | AC 10.2 | Task 13 |
| Req 10: Menu & Commands | AC 10.3 | Task 13 |
| Req 10: Menu & Commands | AC 10.4 | Task 13 |
| Req 10: Menu & Commands | AC 10.5 | Tasks 9, 13 |
| Req 10: Menu & Commands | AC 10.6 | Tasks 6, 13 |
| Req 10: Menu & Commands | AC 10.7 | Task 13 |
| Req 10: Menu & Commands | AC 10.8 | Task 13 |
