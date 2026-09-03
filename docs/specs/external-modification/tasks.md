# Implementation Plan: External Modification Detection (`ff-external-mod`)

## Overview

This plan covers the complete implementation of the `ff-external-mod` crate — the external file modification detection system for FileForgeWorkbench. The crate detects when files currently open in the workbench have been modified, renamed, or deleted by external tools (other editors, build systems, version control, shell scripts), and presents the user with appropriate options to handle the situation.

The system leverages the VFS file-watcher infrastructure from `ff-vfs` and `ff-connector-local-fs` rather than implementing OS-native watching directly. It subscribes to VFS watch events, maintains per-document mtime snapshots, detects discrepancies between in-memory and on-disk state, and coordinates with the document-model and file-operations subsystems for reload/revert operations.

This is a **Wave 8 (File I/O and Session)** sub-project. It depends on `ff-vfs` (VFS abstraction and file watching), `ff-document` (document model and dirty state), `ff-file-ops` (revert/reload operations), `ff-config` (configuration system), `ff-command` (command framework), and `ff-logging` (diagnostics).

---

## Tasks

- [x] 1. Crate scaffolding and core types
  - [x] 1.1 Create `crates/ff-external-mod/Cargo.toml` with dependencies (ff-vfs, ff-document, ff-file-ops, ff-config, ff-command, ff-logging, thiserror, serde, tokio, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-external-mod/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `detector.rs`, `mtime_tracker.rs`, `change_event.rs`, `reload_policy.rs`, `batch_coalescer.rs`, `focus_check.rs`, `prompt.rs`, `config.rs`, `error.rs`
  - [x] 1.4 Add `ff-external-mod` to workspace `Cargo.toml` members list
  - [x] 1.5 Define `ExternalModError` enum with variants: VfsStatFailed, VfsWatchFailed, WatchCancellationFailed, ReloadFailed, DocumentNotFound, ProviderUnsupported, ConfigInvalid, PollingTimeout, BatchOverflow
  - [x] 1.6 Implement `Display` and `thiserror::Error` derives with descriptive messages for all error variants
  - Covers: Structural foundation for all requirements

- [x] 2. Mtime tracking and snapshot management
  - [x] 2.1 Define `MtimeSnapshot` struct with fields: mtime (SystemTime with sub-second precision), document_id (DocumentId), resource_uri (ResourceUri), recorded_at (Instant)
  - [x] 2.2 Define `MtimeTracker` struct holding a `HashMap<DocumentId, MtimeSnapshot>` for all open documents
  - [x] 2.3 Implement `MtimeTracker::record_snapshot(doc_id, uri, vfs) -> Result<MtimeSnapshot>` — query VFS stat and store mtime
  - [x] 2.4 Implement `MtimeTracker::update_snapshot(doc_id, new_mtime)` — update after save or reload
  - [x] 2.5 Implement `MtimeTracker::remove_snapshot(doc_id)` — clean up on document close
  - [x] 2.6 Implement `MtimeTracker::get_snapshot(doc_id) -> Option<&MtimeSnapshot>` — retrieve stored snapshot
  - [x] 2.7 Implement `MtimeTracker::check_mtime(doc_id, vfs) -> Result<MtimeComparison>` — compare stored vs current on-disk mtime
  - [x] 2.8 Define `MtimeComparison` enum with variants: Unchanged, Changed { old: SystemTime, new: SystemTime }, StatFailed(VfsError)
  - [x] 2.9 Implement sub-second precision handling — use nanosecond timestamps where filesystem supports it
  - [x] 2.10 Implement WARN-level logging and pessimistic assumption (treat as changed) when VFS stat fails
  - [x] 2.11 Write unit tests for snapshot lifecycle, comparison logic, sub-second precision, and stat failure handling
  - Covers: Requirement 2 (AC 2.1–2.8)

- [x] 3. VFS watcher integration
  - [x] 3.1 Define `WatchRegistry` struct holding a `HashMap<DocumentId, VfsWatchHandle>` for active watches
  - [x] 3.2 Implement `WatchRegistry::register_watch(doc_id, uri, vfs) -> Result<()>` — call VFS `watch()` and store handle
  - [x] 3.3 Implement `WatchRegistry::cancel_watch(doc_id) -> Result<()>` — call `cancel()` on stored handle, remove entry
  - [x] 3.4 Implement `WatchRegistry::cancel_all()` — cancel all watches on shutdown
  - [x] 3.5 Implement async event stream processing — spawn task to read Watch_Events from each handle's stream
  - [x] 3.6 Implement event dispatch — route VFS Watch_Events (Created, Modified, Deleted, Renamed) to the detector
  - [x] 3.7 Implement fallback polling when VFS provider returns `VfsError::UnsupportedOperation` — spawn periodic timer task
  - [x] 3.8 Implement polling at configurable interval (default 5s) — call VFS stat and compare mtime each tick
  - [x] 3.9 Implement INFO-level log record when falling back to polling mode
  - [x] 3.10 Verify no direct `std::fs` or `tokio::fs` usage — all interaction flows through VFS layer (FFW-ARCH-001)
  - [x] 3.11 Write unit tests for watch registration, cancellation, event routing, polling fallback, and VFS-only enforcement
  - Covers: Requirement 1 (AC 1.1–1.6)

- [x] 4. External change detection logic
  - [x] 4.1 Define `ExternalChange` event struct with fields: document_id, change_type (ChangeType enum), old_mtime, new_mtime, is_dirty, metadata
  - [x] 4.2 Define `ChangeType` enum with variants: ContentChanged, FileDeleted, FileRenamed { old_uri, new_uri }
  - [x] 4.3 Implement `ExternalModificationDetector` struct — central coordinator for detection logic
  - [x] 4.4 Implement Modified event handling — on VFS Modified event, query stat, compare mtime, emit ExternalChange if changed
  - [x] 4.5 Implement spurious event filtering — discard events where mtime matches stored snapshot
  - [x] 4.6 Implement deduplication — track "pending notification" state per document, suppress duplicate events for same mtime change
  - [x] 4.7 Implement `last_asked_mtime` tracking — prevent re-prompting for dismissed changes (analogous to SciTE's fileModLastAsk)
  - [x] 4.8 Implement dirty-state enrichment — query document model for dirty flag and include in ExternalChange event
  - [x] 4.9 Implement post-response mtime update — after user responds to prompt or auto-reload occurs, update snapshot to current on-disk mtime
  - [x] 4.10 Implement Deleted event handling — emit ExternalChange with type FileDeleted
  - [x] 4.11 Implement Renamed event handling — emit ExternalChange with type FileRenamed containing old and new URIs
  - [x] 4.12 Implement document open hook — register watch and record mtime snapshot when document opens
  - [x] 4.13 Implement document close hook — cancel watch and remove mtime snapshot when document closes
  - [x] 4.14 Implement document save hook — update mtime snapshot after successful save
  - [x] 4.15 Write unit tests for detection logic, spurious filtering, deduplication, last-asked tracking, and lifecycle hooks
  - Covers: Requirement 3 (AC 3.1–3.7), Requirement 1 (AC 1.2, 1.3, 1.4), Requirement 2 (AC 2.2, 2.3, 2.4, 2.5, 2.6)

- [x] 5. Reload policy engine
  - [x] 5.1 Define `ReloadPolicy` enum with variants: Prompt, Auto, Ignore
  - [x] 5.2 Implement `ReloadPolicyEngine` struct — evaluates policy against ExternalChange events
  - [x] 5.3 Implement `evaluate(change, policy) -> PolicyAction` — determine action based on policy and dirty state
  - [x] 5.4 Define `PolicyAction` enum with variants: ShowPrompt(PromptOptions), AutoReload, Suppress, UpdateSnapshotOnly
  - [x] 5.5 Implement Prompt policy — always returns ShowPrompt regardless of dirty state
  - [x] 5.6 Implement Auto policy for clean buffers — returns AutoReload when buffer is not dirty
  - [x] 5.7 Implement Auto policy for dirty buffers — falls back to ShowPrompt when buffer is dirty (data loss prevention)
  - [x] 5.8 Implement Ignore policy — returns UpdateSnapshotOnly (suppress notification, update mtime to prevent re-detection)
  - [x] 5.9 Write unit tests for all policy/dirty-state combinations
  - Covers: Requirement 3 (AC 3.2–3.5), Requirement 5 (AC 5.1)

- [x] 6. Reload prompt UI abstraction
  - [x] 6.1 Define `ReloadPrompt` trait — GUI-independent interface for presenting reload notifications to the user
  - [x] 6.2 Define `PromptOptions` struct with fields: file_name (short name), is_dirty (bool), change_type (ChangeType), available_actions (Vec<PromptAction>)
  - [x] 6.3 Define `PromptAction` enum with variants: Reload, Keep, Diff, SaveAs, KeepEditing, Close, FollowRename, KeepOldPath
  - [x] 6.4 Define `PromptResponse` enum mirroring `PromptAction` — represents the user's selection
  - [x] 6.5 Implement dirty-document prompt options — Reload, Keep, Diff (with "unsaved changes" indicator)
  - [x] 6.6 Implement clean-document prompt options — Reload (default), Keep
  - [x] 6.7 Implement deleted-file prompt options — SaveAs, KeepEditing, Close
  - [x] 6.8 Implement renamed-file prompt options — FollowRename, KeepOldPath
  - [x] 6.9 Implement prompt response handling — dispatch to appropriate action handler based on PromptResponse
  - [x] 6.10 Implement Reload response — invoke file-operations Revert command for the document
  - [x] 6.11 Implement Keep response — dismiss notification, update mtime snapshot to current on-disk mtime
  - [x] 6.12 Implement Diff response — invoke compare-and-merge subsystem with in-memory vs on-disk content
  - [x] 6.13 Implement `reload.preserves.undo` handling — preserve or clear undo history based on configuration
  - [x] 6.14 Write unit tests for prompt construction, response dispatch, undo preservation toggling
  - Covers: Requirement 4 (AC 4.1–4.8)

- [x] 7. Auto-reload implementation
  - [x] 7.1 Implement `AutoReloader` struct coordinating automatic reloads for clean buffers
  - [x] 7.2 Implement auto-reload execution — read new content from VFS and replace document buffer
  - [x] 7.3 Implement viewport preservation — restore scroll position and cursor position after reload as closely as possible
  - [x] 7.4 Implement status bar notification — emit brief non-blocking message ("file.rs reloaded") visible for 3 seconds
  - [x] 7.5 Implement mtime snapshot update after successful auto-reload
  - [x] 7.6 Implement auto-reload failure handling — display warning notification, do not mark buffer dirty
  - [x] 7.7 Implement `reload.preserves.undo` respect during auto-reload — preserve or clear undo based on config
  - [x] 7.8 Implement user-edit cancellation — cancel pending auto-reload if user begins editing before reload completes
  - [x] 7.9 Write unit tests for auto-reload success, viewport preservation, failure handling, undo config, and edit cancellation
  - Covers: Requirement 5 (AC 5.1–5.6)

- [x] 8. Deleted file handling
  - [x] 8.1 Implement FileDeleted event processing — receive VFS Deleted event and emit ExternalChange of type FileDeleted
  - [x] 8.2 Implement deleted-file notification presentation — show SaveAs, KeepEditing, Close options
  - [x] 8.3 Implement SaveAs response — delegate to file-operations Save As command for the document
  - [x] 8.4 Implement KeepEditing response — mark buffer dirty, clear backing resource URI (orphan buffer)
  - [x] 8.5 Implement Close response for dirty buffer — trigger standard "save before close?" dialog before discard
  - [x] 8.6 Implement Close response for clean buffer — close document tab directly
  - [x] 8.7 Implement post-deletion watch cancellation — cancel VFS watch for the deleted resource
  - [x] 8.8 Implement reappearance guard — if same URI receives a Created event after Deleted, do NOT auto-associate with open buffer
  - [x] 8.9 Write unit tests for deletion detection, all response paths, watch cancellation, and reappearance guard
  - Covers: Requirement 6 (AC 6.1–6.6)

- [x] 9. Renamed file handling
  - [x] 9.1 Implement FileRenamed event processing — receive VFS Renamed { old_uri, new_uri } and emit ExternalChange
  - [x] 9.2 Implement renamed-file notification presentation — show FollowRename, KeepOldPath options
  - [x] 9.3 Implement FollowRename response — update document backing URI to new_uri, update tab title, re-register watch on new URI, cancel watch on old URI, update mtime snapshot
  - [x] 9.4 Implement KeepOldPath response — mark document dirty, cancel watch on old URI, treat as orphaned buffer
  - [x] 9.5 Implement `auto_follow_rename` config — when enabled and buffer is not dirty, auto-follow without prompt
  - [x] 9.6 Implement `auto_follow_rename` dirty-buffer guard — prompt when buffer is dirty even if auto-follow enabled
  - [x] 9.7 Write unit tests for rename detection, both response paths, auto-follow clean, auto-follow dirty guard
  - Covers: Requirement 7 (AC 7.1–7.6)

- [x] 10. Batch notification coalescing
  - [x] 10.1 Define `BatchCoalescer` struct with configurable debounce window and pending event buffer
  - [x] 10.2 Implement debounce timer — collect events arriving within the configurable window (default 500ms)
  - [x] 10.3 Implement batch assembly — group collected events into a single `BatchNotification` with summary counts
  - [x] 10.4 Define `BatchNotification` struct with fields: modified_files (Vec), renamed_files (Vec), deleted_files (Vec), dirty_files (Vec), total_count (usize)
  - [x] 10.5 Implement batch prompt presentation — display summary with Reload All, Keep All, Review Individually options
  - [x] 10.6 Implement Reload All response — reload all non-dirty buffers in the batch, skip dirty ones with user notification
  - [x] 10.7 Implement Keep All response — dismiss all notifications, update mtime snapshots for all affected documents
  - [x] 10.8 Implement Review Individually response — present each change one at a time in sequence
  - [x] 10.9 Implement dirty-file exclusion from Reload All — dirty documents always require individual confirmation
  - [x] 10.10 Implement streaming cutoff — if events keep arriving after debounce expires, process current batch and start new window
  - [x] 10.11 Implement configurable debounce window — `batch_debounce_ms` setting (range 100–5000ms)
  - [x] 10.12 Write unit tests for debounce timing, batch assembly, dirty exclusion, streaming cutoff, and all response paths
  - Covers: Requirement 8 (AC 8.1–8.7)

- [x] 11. Focus-gained check
  - [x] 11.1 Implement `FocusGainedChecker` struct coordinating synchronous mtime validation on window focus
  - [x] 11.2 Implement focus-gained trigger — on window activation (inactive → active), initiate mtime scan
  - [x] 11.3 Implement bulk mtime check — query VFS stat for all open documents' backing resources
  - [x] 11.4 Implement async background execution — run scan on background task to avoid blocking UI (target <100ms for 50 files)
  - [x] 11.5 Implement change detection — compare each document's on-disk mtime against stored snapshot, emit ExternalChange events for mismatches
  - [x] 11.6 Implement batch coalescing integration — when multiple changes detected, route through BatchCoalescer
  - [x] 11.7 Implement `check_on_focus` config respect — when false, skip focus-gained scan entirely
  - [x] 11.8 Implement tab-switch check — on active tab change, check mtime of newly activated document only
  - [x] 11.9 Implement `last_asked_mtime` respect — do not re-prompt for changes already dismissed by user
  - [x] 11.10 Write unit tests for focus trigger, bulk check, performance target, batch integration, config disable, tab-switch, and dismissed-change guard
  - Covers: Requirement 9 (AC 9.1–9.7)

- [x] 12. Configuration integration
  - [x] 12.1 Define `ExternalModConfig` struct with all configurable fields and validation logic
  - [x] 12.2 Implement `policy` setting — parse `"prompt"`, `"auto"`, `"ignore"` from TOML (default: `"prompt"`)
  - [x] 12.3 Implement `reload_preserves_undo` boolean setting (default: `false`)
  - [x] 12.4 Implement `check_on_focus` boolean setting (default: `true`)
  - [x] 12.5 Implement `auto_follow_rename` boolean setting (default: `false`)
  - [x] 12.6 Implement `batch_debounce_ms` integer setting (default: 500, range: 100–5000)
  - [x] 12.7 Implement `polling_interval_ms` integer setting (default: 5000, range: 1000–60000)
  - [x] 12.8 Implement range clamping — clamp out-of-range values to nearest valid bound with WARN log
  - [x] 12.9 Implement hot-reload support — register Reload_Callback to apply config changes immediately without restart
  - [x] 12.10 Implement layer override support — user, project, and workspace layers per configuration-system precedence
  - [x] 12.11 Implement config namespace registration under `[editor.external_modification]`
  - [x] 12.12 Write unit tests for parsing, defaults, clamping, hot-reload callback, and layer precedence
  - Covers: Requirement 10 (AC 10.1–10.10)

- [x] 13. Property-based tests
  - [x] 13.1 Write PBT: Mtime comparison correctness property
  - [x] 13.2 Write PBT: Deduplication — at most one notification per mtime change property
  - [x] 13.3 Write PBT: Reload policy evaluation completeness property
  - [x] 13.4 Write PBT: Batch coalescing bounded-size property
  - [x] 13.5 Write PBT: Auto-reload dirty-buffer safety property
  - [x] 13.6 Write PBT: Focus-gained check consistency property
  - [x] 13.7 Write PBT: Configuration clamping invariant property
  - [x] 13.8 Write PBT: Watch lifecycle cleanup property
  - Covers: All requirements (property-based validation)

- [x] 14. Integration tests
  - [x] 14.1 Write integration test: full open → external modify → detect → prompt → reload cycle
  - [x] 14.2 Write integration test: auto-reload for clean buffer with viewport preservation
  - [x] 14.3 Write integration test: dirty buffer protection — auto policy falls back to prompt
  - [x] 14.4 Write integration test: file deletion detection and KeepEditing response
  - [x] 14.5 Write integration test: file rename detection and FollowRename response
  - [x] 14.6 Write integration test: batch notification coalescing for multi-file git checkout simulation
  - [x] 14.7 Write integration test: focus-gained check detects missed changes
  - [x] 14.8 Write integration test: polling fallback when VFS watch unsupported
  - [x] 14.9 Write integration test: configuration hot-reload changes behaviour immediately
  - [x] 14.10 Write integration test: document close cancels watch and cleans up mtime snapshot
  - Covers: Cross-requirement interaction validation

---

## Property-Based Test Definitions

### Property 1: Mtime Comparison Correctness

**Validates: Requirement 2.4, 2.5, 2.6**

- **Statement:** For any stored mtime snapshot and any current on-disk mtime, the comparison correctly identifies changes (different mtime → Changed) and non-changes (equal mtime → Unchanged). Sub-second precision differences are correctly detected.
- **Strategy:** Generate:
  - Stored mtime: SystemTime values with nanosecond components in [0, 999_999_999]
  - Current mtime: either identical to stored (50% probability) or different (varying by 1ns to hours)
  - Precision scenarios: full nanosecond, millisecond-only, second-only precision filesystems
- **Invariant:** `check_mtime() == Changed` ⟺ `stored_mtime != current_mtime`. When precision is limited, comparison uses available precision without false negatives on the supported range.

### Property 2: Deduplication — At Most One Notification Per Change

**Validates: Requirement 3.6**

- **Statement:** For any sequence of VFS Modified events for the same document, the detector emits at most one ExternalChange event per distinct mtime transition. Duplicate events for the same mtime change are suppressed.
- **Strategy:** Generate:
  - Event sequences: 5–100 VFS Modified events for a single document
  - Mtime values: 2–5 distinct mtime values (events may repeat the same mtime)
  - User response timing: immediate, delayed, or no response (pending state)
- **Invariant:** Count of emitted ExternalChange events ≤ count of distinct mtime transitions. While a notification is pending (user hasn't responded), no additional event is emitted for the same change.

### Property 3: Reload Policy Evaluation Completeness

**Validates: Requirement 3.2, 3.3, 3.4, 3.5**

- **Statement:** For every combination of (ReloadPolicy, dirty_state, change_type), the policy engine produces a defined PolicyAction. No input combination results in an undefined or panic state.
- **Strategy:** Generate:
  - Policy: {Prompt, Auto, Ignore}
  - Dirty state: {true, false}
  - Change type: {ContentChanged, FileDeleted, FileRenamed}
- **Invariant:** `evaluate()` returns a valid `PolicyAction` for all 18 combinations. Auto+dirty always falls back to ShowPrompt. Ignore always returns UpdateSnapshotOnly. Prompt always returns ShowPrompt.

### Property 4: Batch Coalescing Bounded-Size

**Validates: Requirement 8.1, 8.7**

- **Statement:** For any rate of incoming ExternalChange events, the batch coalescer processes events in bounded windows. No batch accumulates indefinitely — when the debounce window expires, the current batch is emitted regardless of ongoing events.
- **Strategy:** Generate:
  - Debounce window: integer in [100, 5000] ms
  - Event arrival pattern: 10–500 events with inter-arrival times in [0, 2000] ms
  - Document count: 1–50 distinct documents
- **Invariant:** Every event is included in exactly one emitted batch. The time between batch window open and batch emission never exceeds `debounce_window + 1ms` (allowing for timer precision). No event is lost or duplicated across batches.

### Property 5: Auto-Reload Dirty-Buffer Safety

**Validates: Requirement 3.4, 5.1, 5.6**

- **Statement:** Auto-reload never silently replaces content in a dirty buffer. Any document with unsaved local changes always requires explicit user confirmation before its content can be replaced, regardless of policy setting.
- **Strategy:** Generate:
  - Document state sequences: 10–50 operations from {Edit, Save, ExternalModify, AutoReloadTrigger}
  - Track dirty state after each operation
- **Invariant:** `auto_reload_executed` → `is_dirty == false` at the moment of reload. If dirty at reload trigger time, `prompt_shown == true` OR reload was cancelled.

### Property 6: Focus-Gained Check Consistency

**Validates: Requirement 9.1, 9.2, 9.7**

- **Statement:** After a focus-gained check completes, every open document's mtime comparison result is consistent with emitted ExternalChange events. Documents with changed mtimes produce events; documents with unchanged mtimes do not. Previously dismissed changes are not re-prompted.
- **Strategy:** Generate:
  - Open document count: 1–50
  - Per-document mtime state: unchanged (60%), changed (30%), stat-failed (10%)
  - Previously dismissed mtimes: subset of documents with last_asked_mtime matching current mtime
- **Invariant:** `events_emitted` == set of documents where `on_disk_mtime != stored_snapshot AND on_disk_mtime != last_asked_mtime`. Documents in `previously_dismissed` set produce no event.

### Property 7: Configuration Clamping Invariant

**Validates: Requirement 10.6, 10.7, 10.9**

- **Statement:** For any integer configuration value provided (including out-of-range values), the parsed configuration always produces a valid value within the defined range. Out-of-range values are clamped, not rejected.
- **Strategy:** Generate:
  - `batch_debounce_ms`: integers in [-10000, 100000] (far outside valid range [100, 5000])
  - `polling_interval_ms`: integers in [-10000, 1000000] (far outside valid range [1000, 60000])
- **Invariant:** Parsed config satisfies: `100 <= batch_debounce_ms <= 5000` AND `1000 <= polling_interval_ms <= 60000`. A WARN log is emitted for every clamped value.

### Property 8: Watch Lifecycle Cleanup

**Validates: Requirement 1.2, 1.3**

- **Statement:** For any sequence of document open/close operations, the number of active VFS watches equals the number of currently open documents with backing resources. No watches leak after document close.
- **Strategy:** Generate:
  - Operation sequences: 20–200 operations from {Open(doc_id, uri), Close(doc_id)}
  - Document IDs: pool of 1–30 unique IDs
  - URIs: pool of 1–30 unique URIs
- **Invariant:** After each operation, `active_watch_count == open_document_with_uri_count`. After all closes, `active_watch_count == 0`. No duplicate watches for same URI.

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Mtime Tracking", "tasks": ["2"], "dependsOn": [0] },
    { "id": 2, "label": "VFS Watcher Integration", "tasks": ["3"], "dependsOn": [0] },
    { "id": 3, "label": "Change Detection Logic", "tasks": ["4"], "dependsOn": [1, 2] },
    { "id": 4, "label": "Policy Engine", "tasks": ["5"], "dependsOn": [3] },
    { "id": 5, "label": "Prompt UI Abstraction", "tasks": ["6"], "dependsOn": [4] },
    { "id": 6, "label": "Auto-Reload", "tasks": ["7"], "dependsOn": [4, 5] },
    { "id": 7, "label": "Deleted and Renamed Handling", "tasks": ["8", "9"], "dependsOn": [3, 5] },
    { "id": 8, "label": "Batch Coalescing", "tasks": ["10"], "dependsOn": [3, 5] },
    { "id": 9, "label": "Focus-Gained Check", "tasks": ["11"], "dependsOn": [1, 8] },
    { "id": 10, "label": "Configuration", "tasks": ["12"], "dependsOn": [4, 6, 8, 9] },
    { "id": 11, "label": "Property-Based Tests", "tasks": ["13"], "dependsOn": [1, 2, 3, 4, 8, 9, 10] },
    { "id": 12, "label": "Integration Tests", "tasks": ["14"], "dependsOn": [5, 6, 7, 8, 9, 10, 11] }
  ]
}
```

---

## Notes

- This is a Wave 8 (File I/O and Session) crate depending on `ff-vfs` (Wave 3), `ff-document` (Wave 4), `ff-file-ops` (Wave 8), `ff-config` (Wave 2), `ff-command` (Wave 2), and `ff-logging` (Wave 0)
- All file-system interaction goes through the VFS abstraction — no direct `std::fs` or `tokio::fs` calls allowed (FFW-ARCH-001)
- The `connector-local-fs` crate provides the actual OS-native file-watcher; this crate consumes its events via the VFS Watch_Event stream
- The `ReloadPrompt` trait is GUI-independent — concrete implementations live in the GUI shell crate; this crate defines the contract only
- The compare-and-merge subsystem (Wave 14) provides the diff display; this crate invokes it via a trait when the user selects "Diff"
- The `file-operations` crate provides the Revert command; this crate coordinates when to invoke it based on detection and policy
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Mock VFS implementations are used extensively in tests to simulate watch events, stat responses, and provider capabilities
- The batch coalescer uses `tokio::time::sleep` for debounce timing — tests use `tokio::time::pause()` for deterministic timing
- SciTE's `fileModLastAsk` pattern is extended here: we track both the last-notified mtime and the pending-notification state per document
- The polling fallback is a safety net for VFS providers that don't support `watch()` — it should be rare in practice since the local-fs connector supports watching
- Hot-reload of configuration settings means the policy can change mid-session; the detector re-evaluates using current config on each event

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: VFS Watcher Integration | AC 1.1 | Task 3 |
| Req 1: VFS Watcher Integration | AC 1.2 | Tasks 3, 4 |
| Req 1: VFS Watcher Integration | AC 1.3 | Tasks 3, 4 |
| Req 1: VFS Watcher Integration | AC 1.4 | Task 3 |
| Req 1: VFS Watcher Integration | AC 1.5 | Task 3 |
| Req 1: VFS Watcher Integration | AC 1.6 | Task 3 |
| Req 2: Mtime Tracking | AC 2.1 | Task 2 |
| Req 2: Mtime Tracking | AC 2.2 | Tasks 2, 4 |
| Req 2: Mtime Tracking | AC 2.3 | Tasks 2, 4 |
| Req 2: Mtime Tracking | AC 2.4 | Tasks 2, 4 |
| Req 2: Mtime Tracking | AC 2.5 | Tasks 2, 4 |
| Req 2: Mtime Tracking | AC 2.6 | Tasks 2, 4 |
| Req 2: Mtime Tracking | AC 2.7 | Task 2 |
| Req 2: Mtime Tracking | AC 2.8 | Task 2 |
| Req 3: External Modification Detection | AC 3.1 | Task 4 |
| Req 3: External Modification Detection | AC 3.2 | Tasks 4, 5 |
| Req 3: External Modification Detection | AC 3.3 | Tasks 5, 7 |
| Req 3: External Modification Detection | AC 3.4 | Tasks 5, 7 |
| Req 3: External Modification Detection | AC 3.5 | Task 5 |
| Req 3: External Modification Detection | AC 3.6 | Task 4 |
| Req 3: External Modification Detection | AC 3.7 | Task 4 |
| Req 4: User Prompt | AC 4.1 | Task 6 |
| Req 4: User Prompt | AC 4.2 | Task 6 |
| Req 4: User Prompt | AC 4.3 | Task 6 |
| Req 4: User Prompt | AC 4.4 | Task 6 |
| Req 4: User Prompt | AC 4.5 | Task 6 |
| Req 4: User Prompt | AC 4.6 | Task 6 |
| Req 4: User Prompt | AC 4.7 | Task 6 |
| Req 4: User Prompt | AC 4.8 | Task 6 |
| Req 5: Auto-Reload | AC 5.1 | Tasks 5, 7 |
| Req 5: Auto-Reload | AC 5.2 | Task 7 |
| Req 5: Auto-Reload | AC 5.3 | Task 7 |
| Req 5: Auto-Reload | AC 5.4 | Task 7 |
| Req 5: Auto-Reload | AC 5.5 | Task 7 |
| Req 5: Auto-Reload | AC 5.6 | Task 7 |
| Req 6: Deleted Files | AC 6.1 | Tasks 4, 8 |
| Req 6: Deleted Files | AC 6.2 | Task 8 |
| Req 6: Deleted Files | AC 6.3 | Task 8 |
| Req 6: Deleted Files | AC 6.4 | Task 8 |
| Req 6: Deleted Files | AC 6.5 | Task 8 |
| Req 6: Deleted Files | AC 6.6 | Task 8 |
| Req 7: Renamed Files | AC 7.1 | Tasks 4, 9 |
| Req 7: Renamed Files | AC 7.2 | Task 9 |
| Req 7: Renamed Files | AC 7.3 | Task 9 |
| Req 7: Renamed Files | AC 7.4 | Task 9 |
| Req 7: Renamed Files | AC 7.5 | Task 9 |
| Req 7: Renamed Files | AC 7.6 | Task 9 |
| Req 8: Batch Notification | AC 8.1 | Task 10 |
| Req 8: Batch Notification | AC 8.2 | Task 10 |
| Req 8: Batch Notification | AC 8.3 | Task 10 |
| Req 8: Batch Notification | AC 8.4 | Task 10 |
| Req 8: Batch Notification | AC 8.5 | Task 10 |
| Req 8: Batch Notification | AC 8.6 | Task 10 |
| Req 8: Batch Notification | AC 8.7 | Task 10 |
| Req 9: Focus-Gained Check | AC 9.1 | Task 11 |
| Req 9: Focus-Gained Check | AC 9.2 | Task 11 |
| Req 9: Focus-Gained Check | AC 9.3 | Task 11 |
| Req 9: Focus-Gained Check | AC 9.4 | Task 11 |
| Req 9: Focus-Gained Check | AC 9.5 | Task 11 |
| Req 9: Focus-Gained Check | AC 9.6 | Task 11 |
| Req 9: Focus-Gained Check | AC 9.7 | Task 11 |
| Req 10: Configuration | AC 10.1 | Task 12 |
| Req 10: Configuration | AC 10.2 | Tasks 5, 12 |
| Req 10: Configuration | AC 10.3 | Tasks 6, 7, 12 |
| Req 10: Configuration | AC 10.4 | Tasks 11, 12 |
| Req 10: Configuration | AC 10.5 | Tasks 9, 12 |
| Req 10: Configuration | AC 10.6 | Tasks 10, 12 |
| Req 10: Configuration | AC 10.7 | Tasks 3, 12 |
| Req 10: Configuration | AC 10.8 | Task 12 |
| Req 10: Configuration | AC 10.9 | Task 12 |
| Req 10: Configuration | AC 10.10 | Task 12 |
