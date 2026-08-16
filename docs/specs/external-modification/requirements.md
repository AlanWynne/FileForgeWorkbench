# Requirements Document

## Introduction

This feature specifies the external file modification detection system for FileForgeWorkbench — the `ff-external-modification` module (part of the `ff-file-operations` crate or a standalone crate depending on final architecture). This module is responsible for detecting when files that are currently open in the workbench have been modified, renamed, or deleted by external tools (other editors, build systems, version control, shell scripts, etc.), and presenting the user with appropriate options to handle the situation.

The external modification system **leverages the VFS file-watcher** infrastructure provided by the `virtual-file-system` and `connector-local-fs` crates (FFW-ARCH-001). It does not implement its own OS-native file watching — instead it subscribes to VFS watch events and correlates them with open document state. It tracks per-document modification times (mtime), detects discrepancies between in-memory and on-disk state, and coordinates with the document-model and file-operations subsystems for reload/revert operations.

This is a NEW feature identified from the SciTE gap analysis. SciTE implements `CheckReload()` on focus-gained and tab-switch events, comparing `fileModTime` against the current `ModifiedTime()` of the file on disk. FileForgeWorkbench extends this pattern with: VFS-integrated file watching (not just focus-gained polling), batch notification coalescing, configurable reload policies, diff preview, and handling for renamed/deleted files.

**Source references:**
- **[SCI-STE-EXT]** = SciTE `CheckReload()` implementation in `SciTEIO.cxx` — mtime comparison, reload prompt, `load.on.activate`, `reload.preserves.undo`, `are.you.sure.on.reload` properties
- **[WB]** = Workbench Platform Architecture Brief — VFS file-watcher (FFW-ARCH-001), async I/O principle, configuration system

**Cross-references:**
- `virtual-file-system` (Requirement 7: File Watching — WatchHandle, Watch_Event, debounce)
- `connector-local-fs` (Requirement 3: File Watching — OS-native watcher, Debounce_Window, Watch_Event delivery)
- `document-model` (DocumentHandle, DocumentWatcher trait, modified state tracking)
- `file-operations` (Open, Revert, Save — reload mechanics and dirty buffer handling)
- `configuration-system` (TOML config, hot-reload, namespaced settings under `[editor.external_modification]`)

## Glossary

- **External_Modification**: A change to a file's content, metadata, name, or existence that originates from outside the FileForgeWorkbench process (e.g., another editor, a build tool, git operations, shell commands). [SCI-STE-EXT]
- **Modification_Time (mtime)**: The filesystem-reported last-modified timestamp of a file, used as the primary indicator for detecting content changes. Stored per open document and compared against the live filesystem value. [SCI-STE-EXT]
- **Mtime_Snapshot**: The modification time recorded when a document was last loaded from or saved to disk. Used as the baseline for comparison. [SCI-STE-EXT]
- **External_Modification_Detector**: The component that subscribes to VFS watch events, maintains mtime snapshots for all open documents, and emits ExternalChange notifications when discrepancies are detected. [WB]
- **ExternalChange**: An event emitted by the External_Modification_Detector when an open document's backing file has been externally modified, renamed, or deleted. Contains the document identifier, the change type, and relevant metadata. [WB]
- **Reload_Policy**: A configurable strategy that determines how the system responds to external modifications: `prompt` (ask the user), `auto` (reload automatically if buffer is unmodified), `ignore` (do nothing). [SCI-STE-EXT, WB]
- **Dirty_Buffer**: A document buffer that contains unsaved local changes. Dirty buffers always require user confirmation before reload to avoid data loss. [SCI-STE-EXT]
- **Focus_Gained_Check**: A synchronous mtime revalidation performed when the application window regains focus, ensuring detection even if VFS watch events were missed. [SCI-STE-EXT]
- **Batch_Notification**: A coalesced notification that groups multiple rapid external changes into a single user prompt, avoiding notification storms during bulk operations (e.g., `git checkout`). [WB]
- **Debounce_Window**: A configurable time interval during which rapid successive external modification events for the same document are coalesced into a single ExternalChange event. [WB]
- **VFS_Watch_Handle**: The handle returned by the VFS file-watching subsystem, used to subscribe to and cancel file change notifications for a specific resource. [WB]
- **Reload_With_Undo**: A reload mode that replaces the document content while preserving the undo history, enabling the user to undo back to the pre-reload state. [SCI-STE-EXT]

## Requirements

### Requirement 1: VFS File-Watcher Integration

**User Story:** As a workbench developer, I want the external modification system to leverage the existing VFS file-watcher infrastructure rather than implementing its own OS-level watching, so that file change detection is consistent across all VFS providers and avoids duplicated platform-specific code.

**Source:** [WB] FFW-ARCH-001, `virtual-file-system` Requirement 7, `connector-local-fs` Requirement 3. [WB]

#### Acceptance Criteria

1. THE External_Modification_Detector SHALL subscribe to VFS watch events by calling the `watch()` method on the VFS provider for each open document's backing resource URI, receiving a VFS_Watch_Handle for event delivery.
2. WHEN a document is opened (loaded from a VFS resource), THE External_Modification_Detector SHALL register a watch on the resource URI and store the returned VFS_Watch_Handle associated with that document.
3. WHEN a document is closed, THE External_Modification_Detector SHALL cancel the watch by calling `cancel()` on the associated VFS_Watch_Handle, releasing all resources.
4. THE External_Modification_Detector SHALL process VFS Watch_Events (Created, Modified, Deleted, Renamed) delivered via the async stream attached to each VFS_Watch_Handle.
5. WHEN the VFS provider does not support the `watch` capability (returns `VfsError::UnsupportedOperation`), THE External_Modification_Detector SHALL fall back to polling the resource's mtime at the configured polling interval (default: 5 seconds), logging an INFO-level record indicating the fallback.
6. THE External_Modification_Detector SHALL NOT use `std::fs`, `tokio::fs`, or any other direct filesystem API for watching — all file-system interaction SHALL flow through the VFS layer (FFW-ARCH-001).

---

### Requirement 2: Modification Time (mtime) Tracking

**User Story:** As a workbench user, I want the system to track when each open file was last known to be in sync with disk, so that it can reliably detect external changes even if watch events are missed or delayed.

**Source:** [SCI-STE-EXT] — SciTE `fileModTime` and `fileModLastAsk` fields per buffer. [SCI-STE-EXT, WB]

#### Acceptance Criteria

1. THE External_Modification_Detector SHALL maintain an Mtime_Snapshot for each open document, recording the modification time of the backing file as reported by VFS `stat()` at the time the document was last loaded or saved.
2. WHEN a document is opened (initial load), THE External_Modification_Detector SHALL query the resource's mtime via VFS `stat()` and store the result as the Mtime_Snapshot for that document.
3. WHEN a document is saved to disk (via file-operations Save or Save As), THE External_Modification_Detector SHALL update the Mtime_Snapshot to the new mtime reported by VFS `stat()` after the save completes.
4. WHEN a VFS `Modified` watch event is received for an open document, THE External_Modification_Detector SHALL query the resource's current mtime via VFS `stat()` and compare it against the stored Mtime_Snapshot.
5. IF the current mtime differs from the stored Mtime_Snapshot, THEN THE External_Modification_Detector SHALL emit an ExternalChange event of type `ContentChanged` for that document.
6. IF the current mtime equals the stored Mtime_Snapshot (spurious watch event), THEN THE External_Modification_Detector SHALL discard the event without notifying the user.
7. THE Mtime_Snapshot SHALL use sub-second precision where the underlying filesystem and VFS provider support it (e.g., nanosecond precision on ext4/NTFS), to avoid false negatives on rapid save cycles.
8. WHEN a document's Mtime_Snapshot cannot be obtained (e.g., VFS `stat()` returns an error), THE External_Modification_Detector SHALL log a WARN-level record and treat the file as potentially changed, emitting an ExternalChange event.

---

### Requirement 3: External Modification Detection

**User Story:** As a workbench user, I want to be notified when an external tool modifies a file I have open, so that I can decide whether to reload the file or keep my in-memory version.

**Source:** [SCI-STE-EXT] — SciTE's `CheckReload()` comparison logic. [SCI-STE-EXT, WB]

#### Acceptance Criteria

1. WHEN a VFS `Modified` event is received for an open document AND the mtime has changed (confirmed by stat), THE External_Modification_Detector SHALL determine the document's dirty state (has unsaved local changes) and emit an ExternalChange event containing: the document identifier, the change type (`ContentChanged`), the old mtime, the new mtime, and the dirty state.
2. WHEN the Reload_Policy is `prompt`, THE system SHALL present the user with a notification dialog offering choices based on dirty state (see Requirement 4).
3. WHEN the Reload_Policy is `auto` AND the document buffer is NOT dirty (no unsaved local changes), THE system SHALL automatically reload the document content from the VFS without prompting the user.
4. WHEN the Reload_Policy is `auto` AND the document buffer IS dirty, THE system SHALL fall back to prompting the user (same as `prompt` policy) to prevent silent data loss.
5. WHEN the Reload_Policy is `ignore`, THE system SHALL NOT notify the user of external modifications — the in-memory content is kept as-is. The Mtime_Snapshot SHALL still be updated to avoid repeated detection of the same change.
6. THE External_Modification_Detector SHALL emit at most ONE ExternalChange event per document per detected change — if the user has already been prompted about a particular mtime change and has not yet responded, no duplicate notification SHALL be emitted for that same change.
7. AFTER the user responds to a reload prompt (or an auto-reload occurs), THE External_Modification_Detector SHALL update the Mtime_Snapshot to the current on-disk mtime regardless of whether the user chose to reload or keep.

---

### Requirement 4: User Prompt (Reload / Keep / Diff)

**User Story:** As a workbench user, when a file I'm editing is modified externally, I want to choose between reloading the file, keeping my version, or viewing a diff, so that I can make an informed decision without risk of data loss.

**Source:** [SCI-STE-EXT] — SciTE's yes/no reload dialog, extended with diff option. [SCI-STE-EXT, WB]

#### Acceptance Criteria

1. WHEN an ExternalChange event of type `ContentChanged` is received for a document that IS dirty, THE system SHALL present a notification with the following options: **Reload** (discard local changes, load from disk), **Keep** (ignore the external change, continue with local content), and **Diff** (show side-by-side comparison of local vs. disk content).
2. WHEN an ExternalChange event of type `ContentChanged` is received for a document that is NOT dirty, THE system SHALL present a notification with the following options: **Reload** (load from disk) and **Keep** (ignore the external change), with Reload pre-selected as the default action.
3. THE reload notification SHALL display the file name (short name, not full path) and indicate whether the local buffer has unsaved changes.
4. IF the user selects **Reload**, THEN THE system SHALL invoke the file-operations Revert command for the document, replacing the buffer content with the current on-disk content.
5. IF the user selects **Keep**, THEN THE system SHALL dismiss the notification, retain the in-memory content, and update the Mtime_Snapshot to the current on-disk mtime (preventing repeated prompts for the same change).
6. IF the user selects **Diff**, THEN THE system SHALL invoke the compare-and-merge subsystem to display a diff between the in-memory document content and the current on-disk content (read via VFS).
7. WHEN `reload.preserves.undo` configuration is enabled, THE Reload operation SHALL preserve the document's undo history so the user can undo back through the reload boundary. [SCI-STE-EXT]
8. WHEN `reload.preserves.undo` configuration is disabled (default), THE Reload operation SHALL clear the undo history for that document, establishing a fresh undo baseline from the reloaded content. [SCI-STE-EXT]

---

### Requirement 5: Auto-Reload for Unmodified Buffers

**User Story:** As a workbench user working alongside build tools and version control, I want files that I haven't edited to automatically reload when changed externally, so that I always see up-to-date content without manual intervention.

**Source:** [SCI-STE-EXT] — SciTE's `load.on.activate` with automatic reload for clean buffers. [SCI-STE-EXT, WB]

#### Acceptance Criteria

1. WHEN the Reload_Policy is `auto` AND an ExternalChange event is received for a document whose buffer is NOT dirty, THE system SHALL automatically reload the document content from the VFS without displaying any notification or prompt to the user.
2. AFTER an auto-reload, THE system SHALL update the Mtime_Snapshot to the new mtime and preserve the document's viewport position (scroll position, cursor position) as closely as possible.
3. AFTER an auto-reload, THE system SHALL emit a brief, non-blocking status bar message indicating the file was reloaded (e.g., "file.rs reloaded"), visible for 3 seconds.
4. IF an auto-reload fails (VFS read error, file became inaccessible), THEN THE system SHALL display a warning notification to the user indicating the reload failed and the buffer content may be stale, and SHALL NOT mark the buffer as dirty.
5. THE auto-reload mechanism SHALL respect the `reload.preserves.undo` configuration setting — if enabled, undo history is preserved across auto-reloads; if disabled, undo history is cleared.
6. WHEN a document has a pending auto-reload AND the user begins editing that document before the reload completes, THE system SHALL cancel the pending reload and retain the user's edits (user input takes priority over auto-reload).

---

### Requirement 6: Handling Deleted Files

**User Story:** As a workbench user, I want to be notified when a file I have open is deleted externally, so that I can save it to a new location or acknowledge that the backing file no longer exists.

**Source:** [SCI-STE-EXT] — SciTE's deletion detection when `newModTime == 0`. [SCI-STE-EXT, WB]

#### Acceptance Criteria

1. WHEN a VFS `Deleted` watch event is received for an open document's backing resource, THE External_Modification_Detector SHALL emit an ExternalChange event of type `FileDeleted` for that document.
2. WHEN a `FileDeleted` ExternalChange event is received, THE system SHALL present a notification informing the user that the backing file has been deleted, with the following options: **Save As** (save the buffer content to a new location), **Keep Editing** (continue editing the in-memory content with no backing file), and **Close** (close the document tab).
3. IF the user selects **Keep Editing**, THEN THE document SHALL be marked as dirty (unsaved) and the document's backing resource URI SHALL be cleared (it is now an untitled/orphaned buffer), requiring Save As for future saves.
4. IF the user selects **Close** AND the buffer is dirty, THEN THE system SHALL prompt with the standard "save before close?" dialog before discarding the buffer.
5. AFTER a file deletion is detected, THE system SHALL cancel the VFS watch for that resource (as the watch target no longer exists) and update the document's state to reflect the absence of a backing file.
6. IF a deleted file reappears (a new `Created` event for the same URI after a `Deleted` event), THE system SHALL NOT automatically associate the open buffer with the new file — the user must explicitly re-save or re-open.

---

### Requirement 7: Handling Renamed Files

**User Story:** As a workbench user, I want the workbench to detect when a file I have open is renamed or moved externally, so that subsequent saves go to the correct location and my tab title reflects the new name.

**Source:** [SCI-STE-EXT], [WB] — VFS Renamed watch event. [SCI-STE-EXT, WB]

#### Acceptance Criteria

1. WHEN a VFS `Renamed { old_uri, new_uri }` watch event is received AND `old_uri` matches an open document's backing resource, THE External_Modification_Detector SHALL emit an ExternalChange event of type `FileRenamed` containing both the old and new URIs.
2. WHEN a `FileRenamed` ExternalChange event is received, THE system SHALL present a notification informing the user that the file was renamed, with the following options: **Follow Rename** (update the document's backing URI to the new location) and **Keep Old Path** (retain the original URI; the document becomes effectively orphaned from the renamed file).
3. IF the user selects **Follow Rename**, THEN THE system SHALL update the document's backing resource URI to `new_uri`, update the tab title to reflect the new file name, re-register a VFS watch on the new URI, cancel the watch on the old URI, and update the Mtime_Snapshot.
4. IF the user selects **Keep Old Path**, THEN THE system SHALL mark the document as dirty (since the backing file no longer exists at the old path), cancel the watch on the old URI, and treat the document as having no valid backing file (similar to deletion handling).
5. WHEN the `auto_follow_rename` configuration option is enabled AND the buffer is NOT dirty, THE system SHALL automatically follow the rename without prompting the user.
6. WHEN the `auto_follow_rename` configuration option is enabled AND the buffer IS dirty, THE system SHALL prompt the user (same as when disabled) to prevent silent path changes on modified content.

---

### Requirement 8: Batch Notification Coalescing

**User Story:** As a workbench user running a build or git operation that modifies many open files simultaneously, I want the modification notifications to be grouped into a single prompt rather than receiving dozens of individual dialogs, so that I can handle bulk changes efficiently.

**Source:** [WB] — workbench UX principle, extends SciTE's single-file approach. [WB]

#### Acceptance Criteria

1. WHEN multiple ExternalChange events are received within the Debounce_Window (default: 500ms, configurable), THE system SHALL coalesce them into a single Batch_Notification containing all affected documents.
2. THE Batch_Notification SHALL present a summary showing the count of modified files, renamed files, and deleted files, along with a list of affected file names.
3. THE Batch_Notification SHALL offer the following bulk actions: **Reload All** (reload all externally modified non-dirty buffers), **Keep All** (dismiss all notifications), and **Review Individually** (present each change one at a time).
4. IF any document in the batch is dirty (has unsaved local changes), THE Batch_Notification SHALL highlight those documents separately and exclude them from the **Reload All** action — dirty documents always require individual confirmation.
5. WHEN **Reload All** is selected, THE system SHALL reload only the non-dirty documents in the batch; dirty documents SHALL remain unchanged and the user SHALL be informed that N dirty files were skipped.
6. THE Debounce_Window for batch coalescing SHALL be configurable via `editor.external_modification.batch_debounce_ms` (default: 500ms, range: 100–5000ms).
7. IF rapid events continue arriving after the Debounce_Window expires (streaming changes), THE system SHALL process the current batch and start a new batch window for subsequent events rather than holding notifications indefinitely.

---

### Requirement 9: Focus-Gained Check

**User Story:** As a workbench user switching back to the application after using another tool, I want a synchronous check that verifies all open files are still in sync with disk, so that external modifications are detected even if VFS watch events were missed or delayed.

**Source:** [SCI-STE-EXT] — SciTE's `Activate(true)` calling `CheckReload()`. [SCI-STE-EXT, WB]

#### Acceptance Criteria

1. WHEN the workbench application window gains focus (transitions from inactive to active), THE External_Modification_Detector SHALL perform a synchronous mtime check on all open documents by querying VFS `stat()` for each document's backing resource.
2. THE focus-gained check SHALL compare each document's current on-disk mtime against its stored Mtime_Snapshot and emit ExternalChange events for any documents where the mtime has changed.
3. THE focus-gained check SHALL be performed asynchronously on a background task (not blocking the UI thread) and SHALL complete within a reasonable time even with many open documents (target: <100ms for up to 50 open files on local filesystem).
4. IF the focus-gained check detects changes to multiple documents, THE system SHALL apply batch notification coalescing (Requirement 8) rather than prompting for each file individually.
5. THE focus-gained check SHALL respect the `editor.external_modification.check_on_focus` configuration option (default: `true`); WHEN set to `false`, THE system SHALL rely solely on VFS watch events and skip the focus-gained scan.
6. WHEN tab-switching within the workbench (changing the active document tab), THE External_Modification_Detector SHALL perform an mtime check on the newly activated document only (not all open documents). [SCI-STE-EXT]
7. THE focus-gained check SHALL NOT re-prompt for a change that the user has already been notified about and dismissed (tracked via a "last-asked mtime" per document, analogous to SciTE's `fileModLastAsk`). [SCI-STE-EXT]

---

### Requirement 10: Configurable Policies

**User Story:** As a workbench user, I want to configure how external modifications are handled — including whether to auto-reload, whether to prompt, and whether to preserve undo history on reload — so that the behaviour matches my workflow preferences.

**Source:** [SCI-STE-EXT] — SciTE's `load.on.activate`, `reload.preserves.undo`, `are.you.sure.on.reload` properties. [WB]

#### Acceptance Criteria

1. THE configuration namespace `[editor.external_modification]` SHALL contain all external modification settings, conforming to the configuration-system's TOML schema and layer model.
2. THE setting `editor.external_modification.policy` SHALL accept values: `"prompt"` (always ask the user — default), `"auto"` (auto-reload clean buffers, prompt for dirty), `"ignore"` (never notify about external changes).
3. THE setting `editor.external_modification.reload_preserves_undo` SHALL be a boolean (default: `false`); WHEN `true`, reload operations preserve the undo history. [SCI-STE-EXT]
4. THE setting `editor.external_modification.check_on_focus` SHALL be a boolean (default: `true`); WHEN `true`, a focus-gained mtime scan is performed. [SCI-STE-EXT]
5. THE setting `editor.external_modification.auto_follow_rename` SHALL be a boolean (default: `false`); WHEN `true`, renames of non-dirty files are followed automatically without prompting.
6. THE setting `editor.external_modification.batch_debounce_ms` SHALL be an integer (default: 500, range: 100–5000); specifies the debounce window for batch notification coalescing.
7. THE setting `editor.external_modification.polling_interval_ms` SHALL be an integer (default: 5000, range: 1000–60000); specifies the fallback polling interval when VFS watch is unavailable.
8. ALL configuration settings SHALL support hot-reload via the configuration-system's Reload_Callback mechanism — changes take effect immediately without restarting the application.
9. IF a configuration value is outside its valid range, THEN THE system SHALL clamp to the nearest valid bound and emit a WARN-level log record indicating the adjustment.
10. THE configuration settings SHALL be overridable at user, project, and workspace layers (per configuration-system layer precedence), allowing different policies for different projects.
