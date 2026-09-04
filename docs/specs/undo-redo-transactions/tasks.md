# Implementation Plan: Undo/Redo Transactions (`ff-undo-redo`)

## Overview

This plan implements the full transaction system for undo and redo in FileForgeWorkbench. The `ff-undo-redo` crate owns the undo/redo stacks, transaction boundaries, coalescing, save-point tracking, bulk transaction optimisations, tentative actions (IME), selection history, logical record IDs, crash recovery, and validation.

The crate bridges `ff-command` (which produces undo records) and `ff-document-model` (which receives reversed/re-applied edits) via trait interfaces, maintaining GUI independence throughout.

---

## Tasks

- [x] 1. Crate scaffolding and core types
  - [x] 1.1 Create `crates/ff-undo-redo/Cargo.toml` with dependencies: `ff-logging`, `chrono`, `serde`, `serde_json`, `thiserror`; dev-dependencies: `proptest`, `pretty_assertions`, `tempfile`
  - [x] 1.2 Create `src/lib.rs` with crate-level docs and public re-exports
  - [x] 1.3 Create `src/error.rs` with `UndoError` enum (all variants from design §6)
  - [x] 1.4 Create `src/edit_op.rs` with `EditOperation` enum (Insert, Delete, Replace)
  - [x] 1.5 Create `src/config.rs` with `UndoConfig` struct and validation logic (Requirement 1.3, 1.6, 6.4, 8.2, 9.7)
  - [x] 1.6 Create `src/notify.rs` with `UndoNotifier` trait and `ListenerId` type

- [x] 2. Scrap Stack — contiguous text storage
  - [x] 2.1 Create `src/scrap.rs` with `ScrapStack` struct (contiguous byte buffer, position pointer)
  - [x] 2.2 Implement `push(&mut self, data: &[u8]) -> (u64, u32)` — appends data, returns offset and length (Requirement 17.1, 17.3)
  - [x] 2.3 Implement `get(&self, offset: u64, length: u32) -> &[u8]` — retrieves text by offset and length (Requirement 17.2)
  - [x] 2.4 Implement `clear(&mut self)` — releases all storage (Requirement 17.4)
  - [x] 2.5 Write unit tests for ScrapStack push/get/clear semantics

- [x] 3. Undo and Redo stacks
  - [x] 3.1 Create `src/stack.rs` with `UndoStack` (bounded `VecDeque<Transaction>`) and `RedoStack` (`Vec<Transaction>`)
  - [x] 3.2 Implement `UndoStack::push()` with bounded eviction — discard oldest when exceeding `max_levels` (Requirement 1.2, 1.4)
  - [x] 3.3 Implement `UndoStack::pop()` — removes and returns most recent transaction (Requirement 4.1)
  - [x] 3.4 Implement `RedoStack::push()`, `RedoStack::pop()`, `RedoStack::clear()` (Requirement 2.1, 2.2, 2.3)
  - [x] 3.5 Implement `UndoStack::clear()` for `DeleteUndoHistory` (Requirement 1.7)
  - [x] 3.6 Write unit tests for stack push/pop/eviction/clear operations

- [x] 4. Transaction struct and Transaction Builder
  - [x] 4.1 Create `src/transaction.rs` with `Transaction` struct (name, timestamp, operations, container_actions, selection states, may_coalesce, scrap metadata) (Requirement 3.6)
  - [x] 4.2 Implement `TransactionBuilder` — accumulates operations, tracks nesting depth, supports abort/rollback (Requirement 3.2, 3.3, 3.4, 3.5, 3.7)
  - [x] 4.3 Implement `begin_transaction()` — increments nesting depth; only outermost creates boundary (Requirement 3.3)
  - [x] 4.4 Implement `end_transaction()` — decrements depth; commits when depth reaches 0 (Requirement 3.3)
  - [x] 4.5 Implement `abort_transaction()` — rolls back all operations in current transaction (Requirement 3.4)
  - [x] 4.6 Implement orphaned transaction detection and force-close with logging warning (Requirement 3.5)
  - [x] 4.7 Write unit tests for transaction building, nesting, abort, and orphan detection

- [x] 5. Coalescing engine
  - [x] 5.1 Create `src/coalesce.rs` with `CoalesceState` and `CoalesceOpType` (Requirement 6)
  - [x] 5.2 Implement contiguous insert coalescing — merge consecutive single-char inserts at pos+1 (Requirement 6.1)
  - [x] 5.3 Implement contiguous delete coalescing — backspace (pos-1) and delete (same pos) patterns (Requirement 6.2)
  - [x] 5.4 Implement boundary event detection — cursor move, op type change, explicit begin, timeout, save, non-char edit, tentative point (Requirement 6.3)
  - [x] 5.5 Implement coalesce timeout handling with configurable `coalesce_timeout_ms` (Requirement 6.4)
  - [x] 5.6 Implement `may_coalesce` flag checking — do not coalesce if either action has `may_coalesce=false` (Requirement 6.5)
  - [x] 5.7 Implement explicit group override — all actions within begin/end coalesce regardless of char rules (Requirement 6.6)
  - [x] 5.8 Write unit tests for all coalescing rules and boundary events

- [x] 6. Save Point and Dirty Flag tracking
  - [x] 6.1 Create `src/save_point.rs` with `SavePointState` struct (save_point, detach_point, current_action indices)
  - [x] 6.2 Implement `set_save_point()` — marks current position, clears detach point (Requirement 5.2)
  - [x] 6.3 Implement `is_dirty()` derivation — true when current position ≠ save point or detach is set (Requirement 5.3, 5.4)
  - [x] 6.4 Implement detach point logic — set when save point is in discarded redo portion (Requirement 5.5)
  - [x] 6.5 Implement query methods: `is_at_save_point()`, `before_save_point()`, `after_save_point()`, `after_detach_point()` (Requirement 5.6)
  - [x] 6.6 Implement Modified_Line_Marker tracking — set on transaction commit, clear on undo/save (Requirement 5.8, 5.9)
  - [x] 6.7 Write unit tests for save point, detach point, and dirty flag semantics

- [x] 7. Selection History
  - [x] 7.1 Create `src/selection.rs` with `SelectionState`, `CaretPosition`, `SelectionType` structs (Requirement 9)
  - [x] 7.2 Implement before/after state capture on transaction commit (Requirement 9.1, 9.2)
  - [x] 7.3 Implement before-state restoration on undo (Requirement 9.3)
  - [x] 7.4 Implement after-state restoration on redo (Requirement 9.4)
  - [x] 7.5 Implement multi-caret selection state storage and restoration (Requirement 9.6)
  - [x] 7.6 Implement configurable enable/disable via `editor.undo.selection_history` (Requirement 9.7, 9.8)
  - [x] 7.7 Implement sparse storage — only store snapshot when selection actually changed (Requirement 9.9)
  - [x] 7.8 Write unit tests for selection history capture, restore, and disable mode

- [x] 8. UndoManager — per-document orchestrator
  - [x] 8.1 Create `src/manager.rs` with `DocumentUndoManager` struct integrating all components
  - [x] 8.2 Implement `new(config: UndoConfig)` constructor
  - [x] 8.3 Implement `record_insert()`, `record_delete()`, `record_replace()` — records operations with ScrapStack and coalescing (Requirement 1.2)
  - [x] 8.4 Implement `undo()` — pop from undo stack, reverse operations, push to redo, update dirty flag and selection (Requirement 4.1, 4.2, 4.5)
  - [x] 8.5 Implement `redo()` — pop from redo stack, re-apply operations, push to undo, update dirty flag and selection (Requirement 4.4, 4.5)
  - [x] 8.6 Implement `undo_n(count)` and `redo_n(count)` — multi-step undo/redo (Requirement 4.6, 4.7)
  - [x] 8.7 Implement `can_undo()`, `can_redo()`, `undo_description()`, `redo_description()` query methods
  - [x] 8.8 Implement `delete_history()` — clears all state including scrap and record IDs (Requirement 1.7)
  - [x] 8.9 Implement undo-disabled mode when `max_levels == 0` — all recording is no-op, undo/redo returns error (Requirement 1.5)
  - [x] 8.10 Implement listener registration and notification dispatch (Requirement 18.3)
  - [x] 8.11 Write unit tests for orchestrator end-to-end undo/redo workflows

- [x] 9. Bulk Transactions
  - [x] 9.1 Create `src/bulk.rs` with `BulkTransaction`, `RuleTransaction`, `IndexTransaction`, `TransformRule`, `BulkScope` types (Requirement 7)
  - [x] 9.2 Implement automatic strategy selection — Rule for deterministic scopes (ALL, Range, Block), Index for transient scopes (Visible, Excluded, Tagged, Filtered) (Requirement 7.2, 7.3, 7.4)
  - [x] 9.3 Implement `begin_bulk_transaction()`, `record_bulk_affected()`, `end_bulk_transaction()` API (Requirement 7.5)
  - [x] 9.4 Implement `abort_bulk_transaction()` — rollback of in-progress bulk operation (Requirement 7.10)
  - [x] 9.5 Implement Rule_Transaction undo — re-scan document and apply inverse rule (Requirement 7.6)
  - [x] 9.6 Implement Index_Transaction undo — look up current position of each LogicalRecordId and apply inverse (Requirement 7.7)
  - [x] 9.7 Write unit tests for bulk transaction creation, strategy selection, undo, and abort

- [x] 10. Tentative Actions (IME composition)
  - [x] 10.1 Create `src/tentative.rs` with `TentativeState` struct (Requirement 12)
  - [x] 10.2 Implement `tentative_start()` — records tentative point in action sequence (Requirement 12.2)
  - [x] 10.3 Implement `tentative_commit()` — clears tentative point, truncates redo history (Requirement 12.3)
  - [x] 10.4 Implement `tentative_rollback()` — undoes all actions back to tentative point without undo history trace (Requirement 12.4)
  - [x] 10.5 Implement `tentative_active()` and `tentative_steps()` queries (Requirement 12.5)
  - [x] 10.6 Implement tentative point as coalescing barrier (Requirement 12.6)
  - [x] 10.7 Write unit tests for tentative start/commit/rollback/query

- [x] 11. Container Actions (Plugin/Extension State)
  - [x] 11.1 Create `src/container.rs` with `UndoableState` trait definition (Requirement 13.2)
  - [x] 11.2 Implement `record_container_action()` — interleaves container actions with edit operations in transaction (Requirement 13.1)
  - [x] 11.3 Implement undo ordering — invoke container `undo()` in reverse order interleaved with edit reversals (Requirement 13.4)
  - [x] 11.4 Implement redo ordering — invoke container `redo()` in original order (Requirement 13.5)
  - [x] 11.5 Implement container coalescing participation — forward may_coalesce state (Requirement 13.3)
  - [x] 11.6 Ensure container actions do not affect dirty flag or modified line markers (Requirement 13.6)
  - [x] 11.7 Write unit tests for container action recording, undo/redo ordering, and dirty flag isolation

- [x] 12. Logical Record ID system
  - [x] 12.1 Create `src/record_id.rs` with `LogicalRecordId` newtype and `RecordIdMap` struct (Requirement 14)
  - [x] 12.2 Implement `RecordIdMap::new(initial_line_count)` — assigns sequential IDs from 1 (Requirement 14.1)
  - [x] 12.3 Implement `assign_id()` — assigns next available ID, never reuses retired IDs (Requirement 14.2, 14.3)
  - [x] 12.4 Implement `retire_id()` — marks ID as retired (Requirement 14.3)
  - [x] 12.5 Implement `offset_for()` and `update_offsets()` — O(1) lookup with offset tracking on modifications (Requirement 14.4)
  - [x] 12.6 Implement `serialize()` and `deserialize()` for recovery file inclusion (Requirement 14.7)
  - [x] 12.7 Write unit tests for ID assignment, retirement, offset tracking, and serialization round-trip

- [x] 13. Recovery File system
  - [x] 13.1 Create `src/recovery.rs` with `RecoveryWriter` and `RecoveryReader` types (Requirement 8)
  - [x] 13.2 Implement `serialize_for_recovery()` — serializes undo state (stacks, save point, scrap, record IDs) with CRC32 checksum (Requirement 8.1, 8.7)
  - [x] 13.3 Implement `restore_from_recovery()` — deserializes and validates integrity before accepting (Requirement 8.5, 16.4)
  - [x] 13.4 Implement recovery file naming: `.<source_stem>.recovery` alongside source file (Requirement 8.1)
  - [x] 13.5 Implement recovery for unsaved documents — write to `~/.fileforgewb/recovery/` with session-unique name (Requirement 8.8)
  - [x] 13.6 Implement recovery file deletion on save or session close with discard (Requirement 8.3)
  - [x] 13.7 Implement configurable recovery interval via `editor.recovery.interval_seconds`; 0 disables (Requirement 8.2)
  - [x] 13.8 Write unit tests for serialization round-trip, checksum validation, and corruption detection

- [x] 14. History Validation
  - [x] 14.1 Create `src/validate.rs` with validation logic (Requirement 16)
  - [x] 14.2 Implement `validate(document_length: u64) -> bool` — checks cumulative size delta consistency (Requirement 16.1, 16.2)
  - [x] 14.3 Implement position bounds checking — no action references position beyond document bounds at its point in sequence (Requirement 16.2)
  - [x] 14.4 Implement negative-length detection — cumulative document length never goes negative (Requirement 16.2)
  - [x] 14.5 Implement validation-failure handling — clear history, log warning, do not modify document content (Requirement 16.3)
  - [x] 14.6 Write unit tests for validation pass/fail scenarios

- [x] 15. WorkbenchUndoManager and command framework integration
  - [x] 15.1 Create `src/undo_manager_trait.rs` with `WorkbenchUndoManager` struct implementing `UndoManager` trait from `ff-command` (Requirement 15, 18.2)
  - [x] 15.2 Implement `register_document()` and `unregister_document()` — per-document manager registration (Requirement 11.1, 11.3)
  - [x] 15.3 Implement `set_active_document()` — routes undo/redo to active document's stack (Requirement 11.2, 11.5)
  - [x] 15.4 Implement `push_undo()`, `pop_undo()`, `push_redo()`, `pop_redo()`, `clear_redo()` trait methods routing to active document
  - [x] 15.5 Implement `EditTarget` trait definition for document model integration (Requirement 18.5)
  - [x] 15.6 Write unit tests for multi-document routing and per-document isolation

- [x] 16. Non-Undoable Operations guard
  - [x] 16.1 Implement command metadata check — commands with `undoable: false` bypass undo recording entirely (Requirement 10.4)
  - [x] 16.2 Verify non-undoable operations do not modify Undo_Stack, Redo_Stack, Dirty_Flag, or Save_Point (Requirement 10.2)
  - [x] 16.3 Write unit tests confirming non-undoable operations leave undo state untouched

- [x] 17. Property-based tests — Correctness Properties
  - [x] 17.1 Write property test: Undo/Redo Stack Depth Invariant (Property 1) — undo depth ≤ max_levels at all times; after N > M commits, depth == M
    - **Validates: Requirements 1.3, 1.4**
  - [x] 17.2 Write property test: Undo-Redo Symmetry (Property 2) — undo then redo produces byte-identical state to pre-undo
    - **Validates: Requirements 4.1, 4.4, 4.9**
  - [x] 17.3 Write property test: Redo Stack Cleared on New Commit (Property 3) — committing after undo clears redo entirely
    - **Validates: Requirement 2.2**
  - [x] 17.4 Write property test: Save Point Dirty Flag Derivation (Property 4) — is_dirty() == (position != save_point || detach_point.is_some())
    - **Validates: Requirements 5.1, 5.3, 5.4**
  - [x] 17.5 Write property test: Detach Point Semantics (Property 5) — once detached, is_dirty() always true regardless of undo/redo
    - **Validates: Requirement 5.5**
  - [x] 17.6 Write property test: Coalescing Contiguity Rule (Property 6) — contiguous single-char inserts merge into one transaction
    - **Validates: Requirements 6.1, 6.7**
  - [x] 17.7 Write property test: Coalescing Boundary Events (Property 7) — boundary events break coalescing into separate transactions
    - **Validates: Requirement 6.3**
  - [x] 17.8 Write property test: Transaction Nesting Depth Tracking (Property 8) — depth == begins - ends; commits only at depth 0
    - **Validates: Requirements 3.3, 3.7**
  - [x] 17.9 Write property test: Bulk Transaction Memory Efficiency (Property 9) — RuleTransaction is O(1), IndexTransaction is O(n)
    - **Validates: Requirement 7.8**
  - [x] 17.10 Write property test: Selection History Restoration (Property 10) — before-state on undo, after-state on redo; disabled mode skips selection
    - **Validates: Requirements 9.1, 9.3, 9.4, 9.7, 9.8**
  - [x] 17.11 Write property test: Tentative Action Isolation (Property 11) — rollback leaves no trace; commit makes permanent
    - **Validates: Requirements 12.1, 12.3, 12.4**
  - [x] 17.12 Write property test: Recovery Round-Trip Integrity (Property 12) — serialize then deserialize produces equivalent state
    - **Validates: Requirements 8.5, 8.7, 16.4**
  - [x] 17.13 Write property test: Validation Detects Inconsistency (Property 13) — valid histories pass, corrupted histories fail
    - **Validates: Requirements 16.1, 16.2**
  - [x] 17.14 Write property test: Per-Document Isolation (Property 14) — operations on one document do not affect another
    - **Validates: Requirement 11.1**
  - [x] 17.15 Write property test: Logical Record ID Stability (Property 15) — IDs unique, never reused, offsets track correctly
    - **Validates: Requirements 14.1, 14.2, 14.3, 14.4**

- [x] 18. Integration tests
  - [x] 18.1 Write end-to-end test: full editing session — type, undo, redo, save, undo past save, verify dirty flag transitions
  - [x] 18.2 Write end-to-end test: bulk operation — CHANGE ALL with Rule_Transaction, undo in one step, verify document restored
  - [x] 18.3 Write end-to-end test: IME composition — tentative start, compose, rollback, verify no trace; then compose + commit, verify permanent
  - [x] 18.4 Write end-to-end test: crash recovery — build undo state, serialize to recovery, simulate fresh open, restore, verify undo/redo still works
  - [x] 18.5 Write end-to-end test: multi-document — two documents with independent undo stacks, interleaved operations, verify isolation

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered By Task(s) |
|-------------|----------|---------------------|
| Req 1: Undo Stack | 1.1–1.7 | 3.1–3.6, 8.3, 8.8, 8.9, 17.1 |
| Req 2: Redo Stack | 2.1–2.6 | 3.4, 8.4, 8.5, 17.3 |
| Req 3: Transaction Boundaries | 3.1–3.7 | 4.1–4.7, 17.8 |
| Req 4: Undo/Redo Execution | 4.1–4.9 | 8.4–8.6, 17.2 |
| Req 5: Save Point & Dirty Flag | 5.1–5.9 | 6.1–6.7, 17.4, 17.5 |
| Req 6: Coalescing Rules | 6.1–6.7 | 5.1–5.8, 17.6, 17.7 |
| Req 7: Bulk Transactions | 7.1–7.10 | 9.1–9.7, 17.9 |
| Req 8: Recovery Files | 8.1–8.8 | 13.1–13.8, 17.12 |
| Req 9: Selection History | 9.1–9.9 | 7.1–7.8, 17.10 |
| Req 10: Non-Undoable Operations | 10.1–10.4 | 16.1–16.3 |
| Req 11: Per-Document Undo | 11.1–11.5 | 15.1–15.6, 17.14 |
| Req 12: Tentative Actions | 12.1–12.6 | 10.1–10.7, 17.11 |
| Req 13: Container Actions | 13.1–13.6 | 11.1–11.7 |
| Req 14: Logical Record ID | 14.1–14.7 | 12.1–12.7, 17.15 |
| Req 15: UNDO/REDO Commands | 15.1–15.7 | 15.1–15.4 |
| Req 16: History Validation | 16.1–16.4 | 14.1–14.6, 17.13 |
| Req 17: Scrap Stack | 17.1–17.5 | 2.1–2.5 |
| Req 18: Crate API & GUI Independence | 18.1–18.5 | 1.1–1.6, 15.5 |

---

## Notes

- This crate has zero GUI dependencies — all functionality is testable via unit and property-based tests against the public API (Requirement 18.4)
- The crate depends only on `ff-logging`, `chrono`, `serde`, `serde_json`, `thiserror`, and the standard library. All other crates are connected via traits at runtime (Requirement 18.5)
- Property tests use `proptest` crate with a minimum of 100 iterations per property
- Bulk transaction tests (Task 9) depend on Logical Record ID (Task 12) because Index_Transactions store record IDs
- The `EditTarget` trait (Task 15.5) enables document model integration without a compile-time dependency on `ff-document-model`
- Recovery file format uses CRC32 checksum for corruption detection (design Appendix B)

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Crate scaffolding and core types", "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6"] },
    { "id": 1, "label": "Foundation data structures", "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7"], "dependsOn": [0] },
    { "id": 2, "label": "Transaction and Coalescing", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8"], "dependsOn": [1] },
    { "id": 3, "label": "Save Point and Selection History", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8"], "dependsOn": [1, 2] },
    { "id": 4, "label": "UndoManager orchestrator", "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "8.9", "8.10", "8.11"], "dependsOn": [2, 3] },
    { "id": 5, "label": "Advanced features", "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7"], "dependsOn": [4] },
    { "id": 6, "label": "Recovery, Validation, and Integration", "tasks": ["13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7", "13.8", "14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "16.1", "16.2", "16.3"], "dependsOn": [4, 5] },
    { "id": 7, "label": "Property-based tests", "tasks": ["17.1", "17.2", "17.3", "17.4", "17.5", "17.6", "17.7", "17.8", "17.9", "17.10", "17.11", "17.12", "17.13", "17.14", "17.15"], "dependsOn": [6] },
    { "id": 8, "label": "Integration tests", "tasks": ["18.1", "18.2", "18.3", "18.4", "18.5"], "dependsOn": [7] }
  ]
}
```

- [ ] 19. SETUNDO command
  - [ ] 19.1 Register SETUNDO primary command with ON/OFF/n operand parsing
    - Validates: Requirement 19.1
  - [ ] 19.2 Implement SETUNDO ON -- re-enable undo, restore configured max_levels
    - Validates: Requirement 19.1
  - [ ] 19.3 Implement SETUNDO OFF -- disable undo for current session (max_levels=0)
    - Validates: Requirement 19.1
  - [ ] 19.4 Implement SETUNDO n -- set max_levels to n (0-10000), immediate effect
    - Validates: Requirement 19.1
  - [ ] 19.5 Write unit tests for SETUNDO: ON/OFF/n operand parsing, immediate effect on stack behaviour, range validation
    - Validates: Requirement 19.1

- [ ] 20. RECOVERY command
  - [ ] 20.1 Register RECOVERY primary command with ON/OFF/n operand parsing
    - Validates: Requirement 19.2
  - [ ] 20.2 Implement RECOVERY ON -- re-enable recovery file writing, restore configured interval
    - Validates: Requirement 19.2
  - [ ] 20.3 Implement RECOVERY OFF -- disable recovery file writing for current session
    - Validates: Requirement 19.2
  - [ ] 20.4 Implement RECOVERY n -- set recovery interval to n seconds, immediate effect
    - Validates: Requirement 19.2
  - [ ] 20.5 Write unit tests for RECOVERY: ON/OFF/n operand parsing, immediate effect on recovery writer, interval=0 disables
    - Validates: Requirement 19.2
