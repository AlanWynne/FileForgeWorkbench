# Requirements Document

## Introduction

This feature specifies the **file operations** for FileForgeWorkbench (`ff-file-ops` crate). File operations encompass all user-facing commands for creating, opening, saving, closing, and reverting documents: New, Open, Save, Save As, Revert, and Recent Files — plus the underlying persistence mechanisms (atomic rename-on-write, backup copies, read-only detection).

**All file operations route through the VFS abstraction layer** (FFW-ARCH-001). No code in this crate ever calls `std::fs` or `tokio::fs` directly. File reads and writes are mediated by `ResourceUri` addressing and the VFS provider registry, ensuring that the same operations work identically whether the resource is on local disk, in the dataset catalog, or on a future remote connector.

**All user-facing operations are dispatched through the command framework** (cross-cutting Requirement 4). Menu items, keyboard shortcuts, the command line, macros, and plugins all invoke the same command handlers — ensuring consistent dirty-flag tracking, undo integration, and event notification.

This specification merges requirements from two primary sources and adapts them to the workbench VFS-first architecture:

- **FileForgeEditor `file-menu-operations`** (5 requirements): Save As, New File, Recent Files, Revert to Saved, File Menu Layout — all with unsaved-changes confirmation dialogs, atomic write, and command engine integration.
- **SciTE I/O patterns** (`SciTEIO.cxx`): Async save via background worker thread, save-point tracking, read-only detection via properties, external-modification time checks before save, large-file background loading threshold, save-on-deactivate, `save.deletes.first` strategy, progress indication during background I/O.

### Design Principles

1. **VFS-first** — Every file operation uses `ResourceUri` and the VFS provider API. Bare paths are converted to `vfs://local/...` URIs transparently. [WB, FFW-ARCH-001]
2. **Atomic persistence** — Saves use write-to-temp + atomic rename to prevent data corruption on crash or power loss. [FFE-FILEMENU-1]
3. **Async I/O** — Large saves and all VFS writes are async to avoid blocking the GUI thread. Small saves may complete synchronously below a configurable threshold. [SCI-STE-IO]
4. **Command-driven** — All operations are registered commands (`file.new`, `file.open`, `file.save`, `file.save_as`, `file.revert`, `file.open_recent`). [WB]
5. **Unsaved-changes guard** — Operations that discard modifications (New, Open, Revert, Close) always prompt save/discard/cancel when the document is dirty. [FFE-FILEMENU-2, FFE-FILEMENU-4]
6. **Read-only awareness** — The system detects read-only resources and prevents mutation attempts before they reach the VFS. [SCI-STE-IO]
7. **Backup copies** — Optionally, the original file is backed up before overwrite, providing a safety net beyond atomic rename. [SCI-STE-IO]

### Source References

- **[FFE-FILEMENU]** = FileForgeEditor `file-menu-operations` spec (5 requirements, priority source)
- **[SCI-STE-IO]** = SciTE `SciTEIO.cxx` (async save, read-only detection, external-modification check, background I/O thresholds)
- **[WB]** = Workbench Platform Architecture Brief (VFS-first, command-driven, async I/O, GUI independence)

### Cross-References

- **`virtual-file-system`** — All reads and writes go through the VFS API (`ff-vfs`). Resource addresses use `ResourceUri`.
- **`document-model`** — File operations create, load, and reload `Document` instances. Save extracts content from the document.
- **`undo-redo-transactions`** — Revert clears the undo/redo stacks. Save marks the save-point in the transaction history.
- **`multi-tab-editor`** — Open creates a new tab; unsaved-changes dialogs interact with tab close. Recent Files opens into a tab.
- **`configuration-system`** — Provides settings for atomic save strategy, backup copies, recent file count, async threshold, read-only policy.
- **`command-framework`** — All file operations are registered commands with metadata, shortcuts, and undo integration.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **File_Operation** | Any user-facing action that creates, opens, saves, reverts, or closes a document resource through the VFS. | [FFE-FILEMENU], [WB] |
| **Atomic_Write** | The strategy of writing document content to a temporary file in the same directory, then performing an atomic rename over the target — preventing partial writes from corrupting the original. | [FFE-FILEMENU], [SCI-STE-IO] |
| **Backup_Copy** | An optional copy of the original file made before overwrite, stored either alongside the original (with a configurable suffix) or in a dedicated backup directory. | [SCI-STE-IO] |
| **Recent_Files_List** | An ordered, bounded collection of the most recently opened or saved resource URIs, persisted between sessions. | [FFE-FILEMENU] |
| **Unsaved_Changes_Dialog** | A modal confirmation dialog (Save / Discard / Cancel) presented when an operation would discard in-memory modifications. | [FFE-FILEMENU] |
| **Save_Point** | A marker in the undo history indicating the position where the document was last saved. The dirty flag is derived from distance to the save point. | [SCI-STE-IO] |
| **Read_Only_Resource** | A resource that cannot be written to — either because the VFS provider reports it as non-writable, or because the user/configuration has marked it read-only. | [SCI-STE-IO] |
| **Async_Save_Threshold** | A configurable file size (in bytes) above which save operations execute asynchronously on a background task rather than blocking the GUI thread. | [SCI-STE-IO] |
| **Resource_URI** | The unified `vfs://provider/path` address for any resource, as defined by the `virtual-file-system` spec. | [WB] |
| **File_Picker** | The native or custom dialog for selecting resource paths (open or save mode), integrated with VFS providers for browsing. | [FFE-FILEMENU] |

---

## Requirements

### Requirement 1: Save

**User Story:** As a user, I want to persist the current document's content to its associated resource, so that my edits survive application exit and are available the next time I open the file.

**Source:** [FFE-FILEMENU], [SCI-STE-IO], [WB]

#### Acceptance Criteria

1. WHEN the `file.save` command is executed and the document has an associated Resource_URI, THE system SHALL write the document content to that URI via the VFS abstraction layer using the Atomic_Write strategy.
2. WHEN an Atomic_Write completes successfully, THE system SHALL mark the save-point in the undo/redo transaction system, clearing the dirty flag on the document.
3. WHEN an Atomic_Write completes successfully, THE system SHALL update the document's recorded modification time to match the newly written resource's modification time (as reported by VFS `stat`).
4. IF the document has no associated Resource_URI (untitled document), WHEN `file.save` is executed, THE system SHALL delegate to `file.save_as` behaviour (open a File_Picker in save mode).
5. IF the VFS write fails (I/O error, permission denied, provider unavailable), THEN THE system SHALL preserve all in-memory modifications and dirty state, emit a user-visible error notification containing the VFS error description and resource URI, and log an ERROR-level diagnostic.
6. WHEN the document size is at or below the Async_Save_Threshold configuration value, THE system SHALL perform the save synchronously (blocking the command until complete) to minimise latency for small files.
7. WHEN the document size exceeds the Async_Save_Threshold, THE system SHALL perform the save asynchronously on a background task, displaying progress indication in the status area, and SHALL prevent concurrent save operations on the same document until the background save completes.
8. IF a save is already in progress for the current document WHEN `file.save` is executed again, THEN THE system SHALL notify the user that a save is already in progress and take no further action.
9. WHEN `save.check_modified_time` is enabled in configuration AND the resource's modification time on the VFS differs from the document's recorded modification time, THE system SHALL prompt the user with a confirmation dialog ("File was modified externally. Save anyway?") before proceeding with the write.
10. WHEN `file.save` completes successfully, THE system SHALL emit a `file.saved` event (through the command framework event bus) containing the Resource_URI, enabling other subsystems to react (e.g., language service re-analysis, file tree refresh).

---

### Requirement 2: Save As

**User Story:** As a user, I want to save the current document to a new resource location, so that I can create copies, rename files, or choose a different storage provider without leaving the editor.

**Source:** [FFE-FILEMENU] Requirement 1, [WB]

#### Acceptance Criteria

1. WHEN the `file.save_as` command is executed without arguments, THE system SHALL open a File_Picker dialog in save mode, pre-populated with the current document's directory (or a default location if untitled).
2. WHEN the user confirms a target Resource_URI in the File_Picker, THE system SHALL write the document content to that URI via the VFS using Atomic_Write.
3. WHEN a Save As write completes successfully, THE system SHALL update the document's associated Resource_URI to the new target URI.
4. WHEN a Save As write completes successfully, THE system SHALL mark the save-point in the undo/redo system, clear all dirty indicators, and update the tab title and window title to reflect the new resource name.
5. WHEN the user cancels the File_Picker dialog during Save As, THE system SHALL take no action and leave the document unchanged.
6. IF the Atomic_Write fails during Save As, THEN THE system SHALL preserve all in-memory modifications, retain the original Resource_URI, and display an error notification indicating the failure reason.
7. WHEN the `file.save_as` command is executed with a Resource_URI argument (e.g., from a macro or command line), THE system SHALL write directly to that URI without opening the File_Picker.
8. IF the target URI refers to an existing resource, THEN THE system SHALL present an overwrite confirmation dialog before proceeding with the write. IF the user declines, THE system SHALL cancel the Save As operation.
9. WHEN Save As completes successfully, THE system SHALL add the new URI to the Recent_Files_List.
10. THE `file.save_as` command SHALL be available regardless of whether the document is dirty — it permits saving a clean document to a new location.

---

### Requirement 3: New File

**User Story:** As a user, I want to create a new empty document, so that I can start writing content from scratch without opening an existing file.

**Source:** [FFE-FILEMENU] Requirement 2, [WB]

#### Acceptance Criteria

1. WHEN the `file.new` command is executed and the active document has no unsaved modifications, THE system SHALL create a new tab containing an empty Document with no associated Resource_URI, an empty undo/redo stack, and default encoding settings.
2. IF the active document has unsaved modifications WHEN `file.new` is executed, THEN THE system SHALL display an Unsaved_Changes_Dialog with three options: Save, Discard, and Cancel.
3. WHEN the user selects "Save" in the Unsaved_Changes_Dialog and the save completes successfully, THE system SHALL proceed to create the new empty document in a new tab.
4. IF the Save triggered by the Unsaved_Changes_Dialog fails (VFS write error or user cancels a required Save As), THEN THE system SHALL abandon the New operation and leave the current document unchanged.
5. WHEN the user selects "Discard" in the Unsaved_Changes_Dialog, THE system SHALL discard modifications and create the new empty document immediately in a new tab.
6. WHEN the user selects "Cancel" in the Unsaved_Changes_Dialog, THE system SHALL take no action and return to the current document.
7. WHEN a new empty document is created, THE system SHALL set the status bar to show "(Untitled)", set the dirty indicator to off, position the cursor at line 1 column 1, and display a status message "New file".
8. THE system SHALL assign a sequential untitled identifier to each new document (e.g., "Untitled-1", "Untitled-2") to distinguish multiple untitled tabs.

---

### Requirement 4: Open

**User Story:** As a user, I want to open an existing file from any VFS provider, so that I can view and edit its content.

**Source:** [FFE-FILEMENU], [SCI-STE-IO], [WB]

#### Acceptance Criteria

1. WHEN the `file.open` command is executed without arguments, THE system SHALL open a File_Picker dialog in open mode, browsing the VFS provider (defaulting to the local filesystem).
2. WHEN the user confirms a Resource_URI in the File_Picker, THE system SHALL load the resource content via the VFS `read_stream` API into a new Document and display it in a new tab.
3. IF the active document has unsaved modifications WHEN `file.open` is executed, THEN THE system SHALL display an Unsaved_Changes_Dialog before proceeding.
4. WHEN the `file.open` command is executed with a Resource_URI argument (from command line, macro, recent file, or drag-and-drop), THE system SHALL open that resource directly without showing the File_Picker.
5. IF the resource is already open in another tab, WHEN `file.open` is executed for that URI, THE system SHALL activate the existing tab rather than opening a duplicate.
6. IF the VFS read fails (resource not found, permission denied, provider unavailable), THEN THE system SHALL display an error notification with the failure reason and not create a new tab.
7. WHEN the document is successfully loaded, THE system SHALL detect and apply read-only status: IF the VFS provider reports the resource as non-writable, OR the `read.only` property is set for the resource path in configuration, THEN THE system SHALL mark the Document as read-only, preventing mutation operations.
8. WHEN the document is loaded successfully, THE system SHALL record the resource's modification time (from VFS `stat`) for later external-modification detection.
9. WHEN the document is loaded successfully, THE system SHALL add the Resource_URI to the Recent_Files_List.
10. THE system SHALL support opening multiple resources in a single `file.open` invocation (multi-select in File_Picker or multiple URI arguments), creating one tab per resource.

---

### Requirement 5: Revert to Saved

**User Story:** As a user, I want to discard all my unsaved changes and reload the file from disk, so that I can recover from unwanted edits without manually closing and reopening.

**Source:** [FFE-FILEMENU] Requirement 4, [SCI-STE-IO], [WB]

#### Acceptance Criteria

1. WHEN the `file.revert` command is executed and the document has unsaved modifications, THE system SHALL display a confirmation dialog warning that all changes will be lost.
2. WHEN the user confirms the Revert confirmation dialog, THE system SHALL reload the resource content from the VFS, replacing the document's buffer entirely.
3. WHEN a Revert reload completes successfully, THE system SHALL clear all dirty indicators, reset the undo/redo stacks (clearing both undo and redo history), reset the viewport to line 1 and horizontal offset 0, and display a status message "Reverted to saved".
4. WHEN the user cancels the Revert confirmation dialog, THE system SHALL take no action and return to the current document with all modifications preserved.
5. IF the document has no unsaved modifications WHEN `file.revert` is executed, THE system SHALL reload the file immediately without displaying a confirmation dialog (to support explicit refresh use cases).
6. IF the document has no associated Resource_URI (untitled), THEN THE `file.revert` command SHALL be disabled (not executable), and its menu item SHALL appear greyed out.
7. IF the VFS read fails during Revert (resource deleted, permission denied, I/O error), THEN THE system SHALL display an error notification indicating the failure reason and leave the current document and its modifications unchanged.
8. WHEN Revert reloads content, THE system SHALL update the document's recorded modification time to match the resource's current modification time on the VFS.
9. WHEN Revert is performed on a large file (exceeding the async load threshold), THE system SHALL load asynchronously with progress indication, blocking further edits until the reload completes.

---

### Requirement 6: Recent Files

**User Story:** As a user, I want to quickly reopen recently used files, so that I can resume work without navigating the file system each time.

**Source:** [FFE-FILEMENU] Requirement 3, [SCI-STE-IO], [WB]

#### Acceptance Criteria

1. THE system SHALL maintain a Recent_Files_List containing the most recently opened or saved Resource_URIs, ordered from most recent to least recent.
2. THE Recent_Files_List SHALL store a maximum number of entries as configured by `file.recent_files.max_count` (default: 10, configurable via configuration-system).
3. WHEN a file is successfully opened or saved (including Save As), THE system SHALL add that resource's canonical Resource_URI to the top of the Recent_Files_List, removing any duplicate entry with the same URI.
4. WHEN the Recent_Files_List exceeds the configured maximum, THE system SHALL remove the oldest (least recent) entry.
5. WHEN the user selects a URI from the Recent Files list (via menu, command palette, or `file.open_recent` command with index argument), THE system SHALL open that resource as if the user had executed `file.open` with that URI — including the Unsaved_Changes_Dialog if the active document is dirty.
6. IF a resource selected from the Recent Files list no longer exists or cannot be read via the VFS, THEN THE system SHALL display an error notification indicating the resource is inaccessible and remove that entry from the Recent_Files_List.
7. WHEN the Recent_Files_List is modified (entry added or removed), THE system SHALL persist the updated list to the user-level configuration store asynchronously.
8. WHEN the workbench starts, THE system SHALL load the Recent_Files_List from the persisted configuration.
9. IF the persisted Recent_Files_List is missing or contains invalid data at startup, THEN THE system SHALL initialize the list as empty and continue startup normally without error.
10. THE Recent_Files_List SHALL store full Resource_URIs (not bare paths), ensuring provider-specific resources (dataset catalog entries, future remote files) are correctly represented.

---

### Requirement 7: Atomic Write and Backup

**User Story:** As a user, I want my saves to be crash-safe and optionally backed up, so that I never lose data due to a partial write or accidental overwrite.

**Source:** [FFE-FILEMENU] (atomic rename), [SCI-STE-IO] (backup copies, `save.deletes.first`), [WB]

#### Acceptance Criteria

1. THE default save strategy SHALL be Atomic_Write: write content to a temporary file in the same directory as the target (using a `.tmp` suffix or platform temporary file), flush and fsync the temporary file, then atomically rename it over the target resource.
2. IF the VFS provider does not support atomic rename (e.g., some remote providers), THEN THE system SHALL fall back to direct overwrite with explicit flush and fsync, and log a WARN-level diagnostic indicating reduced crash safety.
3. WHEN `file.backup.enabled` is `true` in configuration, THE system SHALL create a Backup_Copy of the existing target resource before the atomic rename overwrites it.
4. THE Backup_Copy SHALL be stored according to the `file.backup.location` setting: `"alongside"` (same directory, with suffix from `file.backup.suffix`, default `.bak`) or `"directory"` (in the path specified by `file.backup.directory`, preserving relative structure).
5. IF creating the Backup_Copy fails, THE system SHALL log a WARN-level diagnostic but SHALL NOT abort the save operation — the save itself proceeds regardless.
6. WHEN `file.save_strategy` is set to `"delete_first"` in configuration, THE system SHALL delete the target resource before writing the new content (SciTE `save.deletes.first` equivalent), instead of using atomic rename.
7. WHEN `file.save_strategy` is set to `"direct"`, THE system SHALL write content directly to the target resource without using a temporary file or atomic rename — suitable for providers that do not support rename semantics.
8. THE system SHALL clean up temporary files left behind by interrupted Atomic_Write operations: on startup, any `.tmp` files matching the pattern used by the save strategy in known directories SHALL be logged as WARN and optionally removed.
9. ALL write operations (temporary file creation, flush, rename) SHALL go through the VFS provider API — the `ff-file-ops` crate SHALL NOT call platform filesystem APIs directly.

---

### Requirement 8: Read-Only Detection

**User Story:** As a user, I want the system to detect read-only files and prevent accidental edits, so that I am not surprised by save failures after making changes to a protected file.

**Source:** [SCI-STE-IO] (Scintilla `SetReadOnly`, `IsReadOnly`), [WB]

#### Acceptance Criteria

1. WHEN a resource is opened, THE system SHALL query the VFS provider's capabilities and the resource metadata to determine write permission. IF the resource is non-writable, THE system SHALL mark the Document as read-only.
2. WHEN a Document is marked read-only, THE system SHALL prevent all mutation operations (insertion, deletion, paste, line commands, undo) from modifying the buffer — attempts SHALL be silently rejected with a status bar notification "Read-only".
3. WHEN a Document is read-only, THE status bar and tab SHALL display a visual read-only indicator (e.g., a lock icon or "[RO]" suffix).
4. THE system SHALL support a `read.only` configuration property (per file-pattern matching) that forces specific resources to be treated as read-only regardless of VFS permissions.
5. THE system SHALL provide a `file.toggle_read_only` command that allows the user to manually toggle read-only status on the current document — overriding the VFS-detected or configuration-detected state.
6. WHEN a user toggles a VFS-reported read-only resource to writable mode, THE system SHALL allow editing but SHALL warn at save time if the VFS provider still reports the resource as non-writable, and the save may fail.
7. IF the VFS provider's capability set does not include `write` for the active provider, THEN all documents opened from that provider SHALL be marked read-only automatically.

---

### Requirement 9: Unsaved Changes Guard

**User Story:** As a user, I want the system to warn me before discarding unsaved work, so that I never accidentally lose edits through New, Open, Revert, Close, or Exit operations.

**Source:** [FFE-FILEMENU] Requirements 2, 4; [SCI-STE-IO] (`SaveIfUnsure`, `are.you.sure`), [WB]

#### Acceptance Criteria

1. WHEN any operation would discard unsaved modifications (New, Open over current, Revert, Close Tab, Exit), THE system SHALL display an Unsaved_Changes_Dialog presenting three options: Save, Discard, Cancel.
2. WHEN the user selects "Save" and the save succeeds, THE system SHALL proceed with the originally requested operation.
3. WHEN the user selects "Save" but the save fails (VFS error or user cancels a required Save As for an untitled document), THE system SHALL abort the originally requested operation and leave the document unchanged.
4. WHEN the user selects "Discard", THE system SHALL immediately proceed with the originally requested operation, discarding all unsaved modifications.
5. WHEN the user selects "Cancel", THE system SHALL abort the originally requested operation and return focus to the current document with all modifications intact.
6. WHEN multiple documents have unsaved modifications during an Exit or Close All operation, THE system SHALL present the dialog for each dirty document in turn (or provide a "Save All / Discard All / Cancel" batch option).
7. THE `file.unsaved_prompt` configuration key (default: `true`) SHALL control whether the unsaved-changes prompt is shown. WHEN set to `false`, operations that discard changes SHALL proceed without prompting (equivalent to SciTE `are.you.sure=0`).
8. THE Unsaved_Changes_Dialog SHALL display the document's resource name (or "Untitled" for new documents) prominently so the user knows which document has unsaved changes.

---

### Requirement 10: File Menu Layout and Command Registration

**User Story:** As a user, I want file operations presented in a logical, standard menu order with proper keyboard shortcuts, so that I can find and invoke commands where I expect them.

**Source:** [FFE-FILEMENU] Requirement 5, [WB]

#### Acceptance Criteria

1. THE file operations SHALL be registered as commands with the following IDs: `file.new`, `file.open`, `file.open_recent`, `file.save`, `file.save_as`, `file.revert`, `file.close`, `file.exit`.
2. EACH registered command SHALL include metadata: display name, description, category (`"file"`), default keyboard shortcut, and an enabled-state predicate.
3. THE default keyboard shortcuts SHALL be: New = `Ctrl+N`, Open = `Ctrl+O`, Save = `Ctrl+S`, Save As = `Ctrl+Shift+S`, Revert = none, Close = `Ctrl+W`, Exit = `Alt+F4`.
4. THE menu layout SHALL present items in the order: New, Open, Recent Files (submenu), separator, Save, Save As, separator, Revert, separator, Close, Exit.
5. THE `file.revert` command's enabled-state predicate SHALL return `false` when the document has no associated Resource_URI (untitled document), causing the menu item and command palette entry to appear disabled.
6. THE `file.save` command's enabled-state predicate SHALL return `false` when the document is not dirty AND has an associated Resource_URI (nothing to save), to provide visual feedback.
7. ALL file operation commands SHALL be invocable from the command palette, menu bar, keyboard shortcuts, Lua macros (via scripting bridge), and the ISPF primary command line (as `NEW`, `OPEN`, `SAVE`, `SAVEAS`, `REVERT`).
8. THE command framework SHALL emit `file.*` events for each operation (e.g., `file.opened`, `file.saved`, `file.new_created`, `file.reverted`) allowing plugins and other subsystems to react.

