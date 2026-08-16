# Requirements Document

## Introduction

This spec defines the **undo/redo transaction system** for FileForgeWorkbench (`ff-undo` crate). It provides the complete infrastructure for recording, coalescing, undoing, and redoing document modifications — from single-character typing through bulk operations affecting millions of records.

The transaction system is the bridge between the command framework (which produces undo records) and the document model (which receives the reversed/re-applied edit operations). It owns the undo and redo stacks, enforces transaction boundaries, implements coalescing of rapid edits, tracks the save point for dirty-flag semantics, supports bulk transaction optimisations for large-scale operations, manages tentative actions for IME composition, and persists undo state for crash recovery.

### Design Principles

1. **The source file on disk is never modified during editing.** All edits accumulate in the document model's edit buffer. The undo system records how to reverse them. [FFE-UNDO-1]
2. **Every mutating operation is wrapped in a named transaction.** This makes undo, redo, macro replay, and audit logging all possible from the same foundation. [FFE-UNDO-2]
3. **Undo reverses transactions one at a time. Redo re-applies them.** [FFE-UNDO-3, FFE-UNDO-4]
4. **Coalescing groups rapid keystrokes into a single undoable unit** — users expect Ctrl+Z to undo a "word", not a character. [SCI-UNDO-4.2]
5. **The save point tracks distance from last save** — the dirty flag is not a simple boolean but a position in the undo history. [SCI-UNDO-4.2, FFE-UNDO-5]
6. **Selection state is part of the undo record** — undo restores not just content but cursor/selection context. [SCI-EDIT-2.4]
7. **GUI-independent** — this crate has no GUI dependency; it provides pure data-structure and logic services. [WB]

### Source References

- **[FFE-UNDO-1]** through **[FFE-UNDO-11]** = FileForgeEditor `undo-redo-transactions` spec (11 requirements, priority source)
- **[SCI-UNDO-4.2]** = Scintilla `UndoHistory` module (action recording, coalescing, save-point, tentative actions, detach-point)
- **[SCI-EDIT-2.4]** = Scintilla `EditModel` (selection-at-undo stacks, UndoSelectionHistoryOption)
- **[WB]** = Workbench Architecture Brief (command-driven undo integration, GUI independence, async I/O)
- **[CF]** = `command-framework` spec Requirement 4 (undo/redo integration contract)

### Cross-References

- **`command-framework`** — Commands produce Undo_Records; this crate owns the stacks they are pushed onto. [CF]
- **`document-model`** — Edit operations are applied to / reversed from the document model's gap buffer.
- **`edit-operations`** — Defines the edit operation types (insert, delete, replace) that transactions contain.
- **`configuration-system`** — Provides `editor.undo.max_levels`, `editor.undo.coalesce_timeout_ms`, `editor.undo.selection_history`, and `editor.recovery.interval_seconds` settings.
- **`file-operations`** — Triggers save-point marking and recovery file cleanup on SAVE.
- **`logging-subsystem`** — Diagnostics for transaction recording, undo/redo execution, and recovery operations.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Transaction** | A named, atomic unit of work in the undo history. Contains one or more Edit_Operations. Either all operations are applied/reversed or none are. | [FFE-UNDO-2], [SCI-UNDO-4.2] |
| **Edit_Operation** | A single atomic change to the document — insert text at position, delete text at range, replace text at range. Carries position, length, and text data. | [FFE-UNDO-2], [SCI-UNDO-4.2] |
| **Undo_Stack** | The bounded, ordered collection of committed Transactions for the current document, most recent at the top. Undo pops from this stack. | [FFE-UNDO-3], [SCI-UNDO-4.2] |
| **Redo_Stack** | The collection of Transactions that were undone and can be re-applied. Cleared when a new edit is committed. | [FFE-UNDO-4], [SCI-UNDO-4.2] |
| **Transaction_Boundary** | The point at which one transaction ends and the next begins. Determined by coalescing rules or explicit grouping. | [FFE-UNDO-2], [SCI-UNDO-4.2] |
| **Coalescing** | The process of merging consecutive, related edit actions into a single transaction so that undo reverses them as a group. | [SCI-UNDO-4.2] |
| **Bulk_Transaction** | A single undo group wrapping a multi-edit operation (e.g., indent entire block, CHANGE ALL). | [FFE-UNDO-10] |
| **Save_Point** | A marker in the undo history indicating the position where the document was last saved. The dirty flag is derived from the current position's distance from the save point. | [SCI-UNDO-4.2], [FFE-UNDO-5] |
| **Detach_Point** | A marker indicating the last action that was before an inaccessible (lost) save point. Once detached, the saved state can never be reached again via undo/redo. | [SCI-UNDO-4.2] |
| **Dirty_Flag** | A derived boolean indicating the document has unsaved changes — true when the current undo position differs from the save point. | [FFE-UNDO-5], [SCI-UNDO-4.2] |
| **Recovery_File** | A periodic snapshot of undo state written to disk for crash recovery. | [FFE-UNDO-6] |
| **Selection_State** | The cursor position, selection range(s), and virtual space at the time a transaction was committed. Stored in the transaction for restoration on undo. | [SCI-EDIT-2.4] |
| **Non-Undoable_Operation** | A state change that bypasses the undo stack entirely (view changes, display mode changes, visibility toggling). | [FFE-UNDO-8] |
| **Rule_Transaction** | A bulk transaction storing a transformation rule rather than individual edit operations. O(1) memory. | [FFE-UNDO-10] |
| **Index_Transaction** | A bulk transaction storing a rule plus a list of affected Logical_Record_IDs. O(n) memory. | [FFE-UNDO-10] |
| **Logical_Record_ID** | A stable integer assigned to each record at file-open time, invariant under insertions/deletions of other records. | [FFE-UNDO-11] |
| **Tentative_Action** | An uncommitted action used during IME composition that can be cleanly rolled back before final commitment. | [SCI-UNDO-4.2] |
| **Undo_Record** | The opaque token produced by the command framework for each undoable command. This crate owns the stacks these records are pushed onto. | [WB], [CF] |
| **Container_Action** | An action type used to record external/plugin state changes alongside document edits, enabling coordinated undo of plugin state. Adapted from Scintilla's container action concept to Rust trait objects. | [SCI-UNDO-4.2] |
| **Scrap_Stack** | An internal text storage structure that efficiently stores the text data for all edit operations in undo history using a contiguous byte buffer with a position pointer. | [SCI-UNDO-4.2] |

---

## Requirements

### Requirement 1: Undo Stack

**User Story:** As an editor user, I want a bounded undo stack so that I can reverse my recent edits without unbounded memory growth.

**Source:** [FFE-UNDO-2], [FFE-UNDO-7], [SCI-UNDO-4.2]

#### Acceptance Criteria

1.1. THE undo-redo system SHALL maintain an Undo_Stack for each document session, implemented as a bounded stack of Transactions ordered from oldest (bottom) to most recent (top). [FFE-UNDO-2], [SCI-UNDO-4.2]

1.2. WHEN an undoable command completes successfully, THE system SHALL push the resulting Transaction onto the top of the Undo_Stack for the active document. [FFE-UNDO-2]

1.3. THE Undo_Stack maximum depth SHALL be configurable via `editor.undo.max_levels` in the configuration system. The default SHALL be 100. The minimum SHALL be 0 (undo disabled). The maximum SHALL be 10000. [FFE-UNDO-7]

1.4. WHEN the Undo_Stack exceeds `max_levels`, THE system SHALL discard the oldest Transaction from the bottom of the stack to make room for the new Transaction. [FFE-UNDO-7]

1.5. WHEN `max_levels` is set to 0, THE system SHALL disable undo entirely — no Transactions are pushed to the Undo_Stack, and UNDO/REDO commands SHALL display a status message indicating undo is disabled. [FFE-UNDO-7]

1.6. IF `max_levels` contains a negative value, THEN THE system SHALL apply the default of 100 and emit a configuration warning via the logging subsystem. [FFE-UNDO-7]

1.7. WHEN `DeleteUndoHistory` is requested (e.g., document reload), THE system SHALL clear all Transactions from the Undo_Stack and Redo_Stack, reset the save point, reset the detach point, clear tentative state, and free associated memory (including scrap text storage). [SCI-UNDO-4.2]

---

### Requirement 2: Redo Stack

**User Story:** As an editor user, I want to redo undone edits so that I can recover work I reversed by mistake.

**Source:** [FFE-UNDO-4], [SCI-UNDO-4.2]

#### Acceptance Criteria

2.1. THE undo-redo system SHALL maintain a Redo_Stack per document session, storing Transactions that were undone and can be re-applied. [FFE-UNDO-4]

2.2. WHEN a new Transaction is committed (pushed to the Undo_Stack) while the Redo_Stack is non-empty, THE system SHALL clear the Redo_Stack entirely — the undone transactions are permanently discarded (standard branching semantics). [FFE-UNDO-4]

2.3. WHEN an undo operation completes, THE system SHALL push the reversed Transaction onto the Redo_Stack. [FFE-UNDO-3]

2.4. WHEN a redo operation is requested, THE system SHALL pop the most recent Transaction from the Redo_Stack, re-apply its Edit_Operations in original order, and push the Transaction back onto the Undo_Stack. [FFE-UNDO-4]

2.5. WHEN REDO is requested and the Redo_Stack is empty, THE system SHALL display a status message indicating there is nothing to redo and SHALL NOT modify the document. [FFE-UNDO-4]

2.6. THE Redo_Stack SHALL NOT have a separate depth limit — its maximum size is bounded by the Undo_Stack depth (you cannot redo more than you undid). [SCI-UNDO-4.2]

---

### Requirement 3: Transaction Boundaries

**User Story:** As an editor user, I want related edits to be grouped into a single undoable transaction so that Ctrl+Z reverses a meaningful unit of work, not individual characters.

**Source:** [FFE-UNDO-2] criterion 4, [SCI-UNDO-4.2] BeginUndoAction/EndUndoAction

#### Acceptance Criteria

3.1. THE following operations SHALL each produce exactly one Transaction: [FFE-UNDO-2]
   - A single primary command execution cycle (primary command + resolved line commands)
   - A macro execution (all operations within one macro run)
   - A clipboard paste operation
   - A file insert operation
   - A shell document capture
   - All field edits to a single record in Grid_Edit_Mode during one editing pass
   - A single character insert, delete, or replace in standard text edit mode (subject to coalescing — see Requirement 6)

3.2. THE system SHALL support explicit transaction grouping via `begin_transaction()` / `end_transaction()` API calls, allowing command handlers and macro engines to wrap multiple edit operations as a single undoable unit. [SCI-UNDO-4.2]

3.3. THE system SHALL support nested `begin_transaction()` / `end_transaction()` calls; only the outermost pair creates a transaction boundary. Inner pairs are counted (depth tracking) but do not create additional boundaries. [SCI-UNDO-4.2]

3.4. WHEN a transaction is in progress and the command handler fails partway through, THE system SHALL roll back all Edit_Operations applied within the current transaction, restoring the document to its pre-transaction state. The failed transaction SHALL NOT be pushed to the Undo_Stack. [FFE-UNDO-2]

3.5. WHEN `begin_transaction()` has been called but `end_transaction()` has not (orphaned transaction), THE system SHALL detect this at the end of the command dispatch cycle and force-close the transaction with a warning logged via the logging subsystem. [WB]

3.6. EACH Transaction SHALL record: a human-readable name (e.g., `"Delete line 42"`, `"CHANGE 'ERROR' 'WARN' ALL"`), the list of Edit_Operations, and a timestamp (UTC, millisecond precision). [FFE-UNDO-2]

3.7. THE system SHALL expose a `transaction_depth() -> usize` method returning the current nesting depth, where 0 indicates no transaction is in progress. [SCI-UNDO-4.2]

---

### Requirement 4: Undo/Redo Execution

**User Story:** As an editor user, I want undo to reverse my last edit and restore my cursor/selection position, so that the document looks exactly as it did before the operation.

**Source:** [FFE-UNDO-3], [FFE-UNDO-4], [FFE-UNDO-9], [SCI-UNDO-4.2], [SCI-EDIT-2.4]

#### Acceptance Criteria

4.1. WHEN UNDO is executed, THE system SHALL pop the most recent Transaction from the Undo_Stack, reverse all its Edit_Operations in reverse order, update the document model, push the Transaction onto the Redo_Stack, and update the dirty flag. [FFE-UNDO-3]

4.2. WHEN UNDO is executed and the Undo_Stack is empty, THE system SHALL display a status message indicating there is nothing to undo and SHALL NOT modify the document. [FFE-UNDO-3]

4.3. WHEN UNDO reverses a multi-operation Transaction (e.g., CHANGE ALL that modified 500 lines), THE system SHALL reverse ALL operations in that Transaction in a single undo step — the user SHALL NOT need to press UNDO 500 times. [FFE-UNDO-3]

4.4. WHEN REDO is executed, THE system SHALL pop the most recent Transaction from the Redo_Stack, re-apply all its Edit_Operations in original order, update the document model, push the Transaction back onto the Undo_Stack, and update the dirty flag. [FFE-UNDO-4]

4.5. WHEN UNDO or REDO completes, THE system SHALL restore the Selection_State (cursor position, selection range(s), virtual space) that was recorded when the Transaction was originally committed (see Requirement 9). The viewport SHALL scroll if necessary to make the restored cursor visible. [SCI-EDIT-2.4]

4.6. THE system SHALL support `UNDO n` where `n` is a positive integer, executing `n` successive undo operations in one command dispatch. WHEN `n` exceeds available transactions, THE system SHALL undo all available and display a message indicating the actual count undone. [FFE-UNDO-9]

4.7. THE system SHALL support `REDO n` analogously to `UNDO n`. [FFE-UNDO-9]

4.8. WHEN UNDO or REDO is issued in Browse mode or View mode (read-only), THE system SHALL display a status message indicating the command is not available and SHALL NOT modify any state. [FFE-UNDO-9]

4.9. WHEN REDO re-applies a Transaction, the result SHALL be byte-identical to the original application — no data loss, no content reordering. [FFE-UNDO-4]

---

### Requirement 5: Save Point and Dirty Flag

**User Story:** As an editor user, I want the editor to accurately track whether I have unsaved changes, including after undo/redo operations, so that I always know whether I need to save.

**Source:** [FFE-UNDO-5], [SCI-UNDO-4.2] (SetSavePoint, IsSavePoint, BeforeSavePoint, AfterSavePoint, detach-point)

#### Acceptance Criteria

5.1. THE system SHALL maintain a Save_Point — a marker indicating the position in the undo history where the document was last saved (or opened, initially). [FFE-UNDO-5], [SCI-UNDO-4.2]

5.2. WHEN a file is saved successfully, THE system SHALL set the Save_Point to the current undo position and clear the detach point. The Dirty_Flag SHALL become false. [SCI-UNDO-4.2]

5.3. THE Dirty_Flag SHALL be true whenever the current undo position differs from the Save_Point. This includes: after committing a new transaction, after undoing past the save point, or after redoing past the save point. [FFE-UNDO-5], [SCI-UNDO-4.2]

5.4. WHEN a series of undo operations returns the document to the exact Save_Point position, THE Dirty_Flag SHALL become false — the document matches its on-disk state. [SCI-UNDO-4.2]

5.5. WHEN a new Transaction is committed that truncates the redo history, and the Save_Point was located in the discarded redo portion, THE system SHALL set a Detach_Point at the current action position. The Save_Point becomes unreachable. The Dirty_Flag SHALL remain true regardless of future undo operations (the saved state can no longer be reached). [SCI-UNDO-4.2]

5.6. THE system SHALL provide query methods for save-point state: `is_save_point()` (at save point), `before_save_point()` (undo position is before the save point in history), `after_save_point()` (undo position is after), and `after_detach_point()` (save point is unreachable). [SCI-UNDO-4.2]

5.7. THE Dirty_Flag SHALL be displayed in the status bar (e.g., `[Modified]` or asterisk indicator), per the cross-cutting status bar requirement. [FFE-UNDO-5]

5.8. WHEN a Transaction modifies one or more lines, THE system SHALL set a Modified_Line_Marker on each affected line. WHEN UNDO reverses a Transaction, Modified_Line_Markers SHALL be cleared for lines that are no longer modified relative to the Save_Point. [FFE-UNDO-5]

5.9. WHEN SAVE succeeds, THE system SHALL clear all Modified_Line_Markers (all lines now match on-disk state). [FFE-UNDO-5]

---

### Requirement 6: Coalescing Rules

**User Story:** As an editor user, I want rapid consecutive keystrokes to be undone as a group (like a word), not one character at a time, so that undo is efficient and matches my mental model of "what I just typed."

**Source:** [SCI-UNDO-4.2] (contiguous typing, single-char backspace/delete coalescing), [FFE-UNDO-2] (transaction boundaries for typing)

#### Acceptance Criteria

6.1. THE system SHALL coalesce consecutive single-character insert operations into one Transaction when they are contiguous (each new character is inserted immediately after the previous one's end position) and no boundary event intervenes. [SCI-UNDO-4.2]

6.2. THE system SHALL coalesce consecutive single-character delete operations (backspace or delete key) into one Transaction when they form a contiguous removal pattern: [SCI-UNDO-4.2]
   - Backspace: each deletion position + length equals the previous deletion position (removing backwards)
   - Delete: each deletion is at the same position (removing forwards at a fixed point)
   - Only single-character (1 or 2 byte) removals are eligible for coalescing; multi-character removals break the sequence.

6.3. THE following events SHALL terminate coalescing and start a new Transaction boundary: [SCI-UNDO-4.2], [FFE-UNDO-2]
   - Cursor movement (arrow keys, mouse click, go-to-line) without typing
   - A change in edit operation type (switching from insert to delete or vice versa)
   - An explicit `begin_transaction()` call (command or macro initiating a grouped operation)
   - A pause exceeding the coalesce timeout
   - The document being saved (save-point change)
   - A non-character edit operation (paste, cut, line command)
   - The undo position being at the save point or tentative point

6.4. THE coalesce timeout SHALL be configurable via `editor.undo.coalesce_timeout_ms` in the configuration system. The default SHALL be 2000 milliseconds (2 seconds). The minimum SHALL be 100ms. The maximum SHALL be 10000ms. [WB]

6.5. WHEN two actions are being considered for coalescing, THE system SHALL NOT coalesce if either action has `may_coalesce=false` (indicating an explicit boundary was set by `end_transaction()` or a save-point). [SCI-UNDO-4.2]

6.6. WHEN inside an explicit `begin_transaction()` / `end_transaction()` group, ALL actions within the group SHALL coalesce regardless of the above character-level rules — the explicit grouping overrides character-level boundary detection. [SCI-UNDO-4.2]

6.7. WHEN coalescing is active and a new character is typed, THE system SHALL NOT push a new Transaction to the Undo_Stack; instead it SHALL extend the current (in-progress) Transaction with the additional Edit_Operation. [SCI-UNDO-4.2]

---

### Requirement 7: Bulk Transactions

**User Story:** As an editor user, I want multi-edit operations (like indenting a block or replacing all occurrences) to be a single undo step, and I want large bulk operations to be stored efficiently.

**Source:** [FFE-UNDO-10], [FFE-UNDO-11]

#### Acceptance Criteria

7.1. THE system SHALL wrap multi-edit operations in a single Transaction (bulk transaction) so that UNDO reverses the entire operation in one step. Examples: indent/outdent block, CHANGE ALL, SORT, multi-line paste. [FFE-UNDO-10]

7.2. THE system SHALL support two bulk transaction storage strategies, selected automatically based on scope: [FFE-UNDO-10]
   - **Rule_Transaction**: stores the transformation rule (pattern, replacement, scope) with O(1) memory. Used when scope is deterministic and re-scannable.
   - **Index_Transaction**: stores the rule plus a materialised list of Logical_Record_IDs. O(n) memory. Used when scope depends on transient state.

7.3. THE following scope types SHALL use Rule_Transaction (re-scan on undo): [FFE-UNDO-10]
   - `ALL` — applies to every record; scope is fully deterministic from command arguments
   - Explicit line range (e.g., `CHANGE ... IN 100 500`) — range is deterministic
   - `CC` block scope — block boundaries are deterministic from the command context

7.4. THE following scope types SHALL use Index_Transaction (materialise record IDs): [FFE-UNDO-10]
   - `VISIBLE` / `NX` (non-excluded) — depends on transient visibility state
   - `X` (excluded only) — depends on transient visibility state
   - `TAGGED` — depends on transient tag state
   - Any scope combined with an active Record_Filter, Record_Type_Filter, or Criteria_Set

7.5. WHEN an Index_Transaction is built, THE system SHALL record the Logical_Record_ID of each affected record — not line numbers or byte offsets — so that undo remains correct after intervening insertions or deletions. [FFE-UNDO-10], [FFE-UNDO-11]

7.6. WHEN UNDO reverses a Rule_Transaction, THE system SHALL re-scan the document, apply the inverse rule, and update the document. Undo cost: one document pass. [FFE-UNDO-10]

7.7. WHEN UNDO reverses an Index_Transaction, THE system SHALL look up the current position of each Logical_Record_ID and apply the inverse operation at each position. Undo cost: O(n affected records). [FFE-UNDO-10]

7.8. THE Rule_Transaction memory cost SHALL be O(1) — constant regardless of how many records are affected. The Index_Transaction memory cost SHALL be O(n) where n is the number of affected records. [FFE-UNDO-10]

7.9. WHEN a bulk operation is in progress and takes more than 1 second, THE system SHALL execute asynchronously with a progress indicator in the status bar. The UI SHALL remain responsive. [FFE-UNDO-10], [WB]

7.10. WHEN the user cancels an in-progress bulk operation, THE system SHALL roll back all Edit_Operations applied so far, restoring the document to its pre-operation state. The cancelled Transaction SHALL NOT be pushed to the Undo_Stack. [FFE-UNDO-10]

---

### Requirement 8: Recovery Files

**User Story:** As an editor user, I want the editor to periodically save undo state so that I can recover unsaved work after a crash or power loss.

**Source:** [FFE-UNDO-6]

#### Acceptance Criteria

8.1. WHEN the Dirty_Flag is true and a configurable interval has elapsed (default: 60 seconds), THE system SHALL write the current undo state (edit buffer operations, undo stack, save point) to a recovery file named `.<source_stem>.recovery` in the same directory as the source file. [FFE-UNDO-6]

8.2. THE recovery interval SHALL be configurable via `editor.recovery.interval_seconds` in the configuration system. Setting it to 0 SHALL disable recovery file writing. [FFE-UNDO-6]

8.3. WHEN SAVE or session close (with discard) completes successfully, THE system SHALL delete the recovery file if one exists. [FFE-UNDO-6]

8.4. WHEN the editor opens a file and a recovery file exists for that file, THE system SHALL notify the user and offer to restore or discard it. [FFE-UNDO-6]

8.5. IF the user chooses to restore, THE system SHALL load the recovery file's undo state and set the Dirty_Flag. The Undo_Stack and redo history SHALL be restored from the recovery data. [FFE-UNDO-6]

8.6. IF the user chooses to discard, THE system SHALL delete the recovery file and open the source in its on-disk state with empty undo history. [FFE-UNDO-6]

8.7. THE recovery file format SHALL store Edit_Operations, Undo_Stack state, Save_Point position, and any Index_Transaction Logical_Record_ID mappings in a compact binary or JSON format compatible with the document model's patch representation. [FFE-UNDO-6]

8.8. WHEN a new unsaved document (no source path yet) has unsaved changes, THE system SHALL write the recovery file to the workbench data directory (`~/.fileforgewb/recovery/`) using a session-unique filename. WHEN the document is first saved to a path, THE system SHALL delete the temporary recovery file and begin writing recovery alongside the saved path. [FFE-UNDO-6]

---

### Requirement 9: Selection History (Undo Restores Selection)

**User Story:** As an editor user, I want undo to restore my cursor and selection to where they were before the operation, so that I have full context of what I was doing.

**Source:** [SCI-EDIT-2.4] (ModelState, SelectionHistory, UndoSelectionHistoryOption, RememberSelectionForUndo), [FFE-UNDO-3] (undo restores editing context)

#### Acceptance Criteria

9.1. WHEN a Transaction is committed, THE system SHALL capture and store the current Selection_State (all caret positions, anchor positions, virtual space values, selection type) as part of the Transaction record. [SCI-EDIT-2.4]

9.2. THE system SHALL store two Selection_States per Transaction: the **before-state** (selection at the start of the transaction) and the **after-state** (selection at the end of the transaction, which is the state to restore on redo). [SCI-EDIT-2.4]

9.3. WHEN UNDO reverses a Transaction, THE system SHALL restore the **before-state** Selection_State — the cursor/selection returns to where it was before the operation was performed. [SCI-EDIT-2.4]

9.4. WHEN REDO re-applies a Transaction, THE system SHALL restore the **after-state** Selection_State — the cursor/selection returns to where it was after the operation was originally performed. [SCI-EDIT-2.4]

9.5. WHEN the restored Selection_State references a position that is off-screen, THE system SHALL scroll the viewport to make the restored cursor position visible (centered or near-center). [SCI-EDIT-2.4]

9.6. THE Selection_State SHALL include multi-caret/multi-selection state: if the user had multiple carets when the transaction was committed, undo/redo SHALL restore all caret positions. [SCI-EDIT-2.4]

9.7. THE selection history feature SHALL be configurable via `editor.undo.selection_history` in the configuration system with the following options: [SCI-EDIT-2.4]
   - `"enabled"` (default) — selection state is recorded with each transaction and restored on undo/redo
   - `"disabled"` — selection state is NOT recorded; undo/redo does not restore cursor/selection position

9.8. WHEN selection history is disabled, UNDO and REDO SHALL still function correctly for document content — only the selection/cursor restoration is skipped. The cursor SHALL remain at its current position after undo/redo. [SCI-EDIT-2.4]

9.9. THE selection history stacks SHALL be sparse — only transactions where the selection actually changed relative to the previous state need to store a full Selection_State snapshot. Transactions with no selection change MAY reference the previous state. [SCI-EDIT-2.4]

---

### Requirement 10: Non-Undoable Operations

**User Story:** As an editor developer, I want a clear definition of which operations bypass the undo stack, so that view-only changes don't pollute undo history and users have predictable undo behaviour.

**Source:** [FFE-UNDO-8]

#### Acceptance Criteria

10.1. THE following operations SHALL NOT be pushed to the Undo_Stack and SHALL NOT be reversible via UNDO: [FFE-UNDO-8]
   - Exclude/show visibility state changes
   - Tag and untag operations
   - Bounds and tab stop setting changes
   - Syntax/language mode changes
   - Active Record_Filter and Record_Type_Filter changes
   - Active Criteria_Set changes
   - Command history navigation (RETRIEVE)
   - Session metadata changes (window size, panel layout, recent files)
   - Display mode switches (e.g., entering/leaving HEX mode)
   - Zoom level changes
   - Theme and appearance changes
   - Scroll position changes

10.2. WHEN a non-undoable operation is performed, THE system SHALL NOT modify the Undo_Stack, Redo_Stack, Dirty_Flag, or Save_Point. [FFE-UNDO-8]

10.3. EACH non-undoable operation category SHALL have its own dedicated reset or clear mechanism as defined in its relevant spec (e.g., `RESET` for visibility, `CRITERIA CLEAR` for criteria sets). [FFE-UNDO-8]

10.4. THE command framework SHALL identify non-undoable commands at registration time (via the `undoable: false` declaration in command metadata), ensuring the undo system never attempts to record them. [WB], [CF]

---

### Requirement 11: Per-Document Undo

**User Story:** As a multi-tab editor user, I want each open document to have its own independent undo stack, so that undoing in one document never affects another.

**Source:** [FFE-UNDO-1] criterion 3, [SCI-UNDO-4.2] (per-document UndoHistory)

#### Acceptance Criteria

11.1. EACH open document SHALL have its own independent Undo_Stack, Redo_Stack, Save_Point, Detach_Point, coalescing state, and tentative action state. Undo operations in one document SHALL NOT affect any other document's undo history. [FFE-UNDO-1], [SCI-UNDO-4.2]

11.2. WHEN a document tab is activated, THE system SHALL restore the undo/redo state for that document — UNDO and REDO SHALL operate on the active document's stacks. [FFE-UNDO-1]

11.3. WHEN a document is closed (after save or discard), THE system SHALL release the undo/redo stacks and all associated memory for that document (including scrap text, selection history, and logical record ID mappings). [FFE-UNDO-1]

11.4. THE `max_levels` configuration SHALL apply independently to each document's Undo_Stack. If 5 documents are open with `max_levels=100`, the total maximum undo storage is 500 transactions across all documents. [FFE-UNDO-7]

11.5. WHEN the command framework dispatches UNDO or REDO, IT SHALL route the operation to the undo stack of the currently active document as identified by the Execution_Context. [WB], [CF]

---

### Requirement 12: Tentative Actions (IME Composition)

**User Story:** As an editor user composing text via an Input Method Editor (IME), I want my in-progress composition to be cleanly reversible without polluting the undo history, so that cancelled compositions leave no trace.

**Source:** [SCI-UNDO-4.2] (TentativeStart, TentativeCommit, TentativeActive, TentativeSteps)

#### Acceptance Criteria

12.1. THE system SHALL support a tentative action mode for IME composition, where actions appended after `tentative_start()` are marked as uncommitted and can be cleanly removed without a full undo cycle. [SCI-UNDO-4.2]

12.2. WHEN `tentative_start()` is called, THE system SHALL record a tentative point at the current action position. All subsequent Edit_Operations are tentative until `tentative_commit()` or rollback. [SCI-UNDO-4.2]

12.3. WHEN `tentative_commit()` is called, THE system SHALL clear the tentative point and truncate the undo history to the current position — the tentative actions become permanent and the redo history beyond them is discarded. [SCI-UNDO-4.2]

12.4. WHEN IME composition is cancelled (rollback), THE system SHALL undo all tentative steps (from current action back to the tentative point), restoring the document to its pre-composition state. The tentative actions SHALL NOT remain in the undo history. [SCI-UNDO-4.2]

12.5. THE system SHALL provide `tentative_active() -> bool` to query whether tentative mode is in progress, and `tentative_steps() -> Option<usize>` to return the number of actions since the tentative point (or None if not active). [SCI-UNDO-4.2]

12.6. WHEN the tentative point is active, coalescing boundary detection SHALL treat the tentative point as a coalescing barrier — new actions SHALL NOT coalesce with pre-tentative actions. [SCI-UNDO-4.2]

---

### Requirement 13: Container Actions (Plugin/Extension State)

**User Story:** As a plugin developer, I want to record plugin-specific state alongside document edits in the undo history, so that undoing a command also undoes the plugin state change (e.g., fold state, annotation position, decoration state).

**Source:** [SCI-UNDO-4.2] (ActionType::container), adapted to Rust trait objects

#### Acceptance Criteria

13.1. THE system SHALL support a Container_Action type alongside insert and remove action types, allowing external subsystems (plugins, extensions) to record opaque state changes as part of a transaction. [SCI-UNDO-4.2]

13.2. A Container_Action SHALL implement a Rust trait (`UndoableState`) with methods: `undo(&self)` to reverse the state change, `redo(&self)` to re-apply it, and `description(&self) -> &str` for diagnostic display. [WB]

13.3. Container_Actions SHALL participate in coalescing — they MAY forward the coalesce state of adjacent document actions. A coalescible container action does not break a typing sequence. [SCI-UNDO-4.2]

13.4. WHEN UNDO reverses a transaction containing Container_Actions, THE system SHALL invoke `undo()` on each Container_Action in reverse order, interleaved with the document edit reversals at the correct position in the sequence. [SCI-UNDO-4.2]

13.5. WHEN REDO re-applies a transaction containing Container_Actions, THE system SHALL invoke `redo()` on each Container_Action in original order. [SCI-UNDO-4.2]

13.6. Container_Actions SHALL NOT affect the Dirty_Flag or Modified_Line_Markers — only document edit operations (insert/remove) affect dirty state. [SCI-UNDO-4.2]

---

### Requirement 14: Logical Record Identity

**User Story:** As an editor developer, I want records to be identified by stable logical IDs rather than line numbers, so that undo and redo remain correct even after preceding lines have been inserted or deleted.

**Source:** [FFE-UNDO-11]

#### Acceptance Criteria

14.1. WHEN a file is opened, THE system SHALL assign a Logical_Record_ID to each record. IDs are assigned sequentially from 1 and are stable for the lifetime of the session — they do not change when other records are inserted, deleted, or reordered. [FFE-UNDO-11]

14.2. WHEN a new record is inserted (e.g., via `I` line command, clipboard paste, or file insert), THE system SHALL assign it the next available Logical_Record_ID. Existing IDs are never renumbered. [FFE-UNDO-11]

14.3. WHEN a record is deleted, its Logical_Record_ID is retired for that session. Retired IDs are never reused. [FFE-UNDO-11]

14.4. THE system SHALL maintain a byte-offset index mapping from Logical_Record_ID to current byte offset, updated whenever the document model is modified. This allows O(1) lookup of any record's current position by its stable ID. [FFE-UNDO-11]

14.5. ALL Index_Transactions (Requirement 7, criterion 7.4) SHALL store Logical_Record_IDs, not line numbers or byte offsets. This ensures undo correctness after intervening insertions or deletions. [FFE-UNDO-11]

14.6. THE Logical_Record_ID mapping SHALL be held in memory only — it is session state, not persisted to disk. On file reopen, IDs are reassigned from scratch. [FFE-UNDO-11]

14.7. WHEN the Edit_Buffer is written to the Recovery_File (Requirement 8), THE system SHALL include the current Logical_Record_ID mapping in the recovery data so that any pending Index_Transactions stored in the recovery file remain valid after restoration. [FFE-UNDO-11]

---

### Requirement 15: UNDO and REDO Command Integration

**User Story:** As a keyboard-driven user, I want `UNDO` and `REDO` to work from the primary command line and from configurable function keys, consistent with the ISPF-style editing model.

**Source:** [FFE-UNDO-9]

#### Acceptance Criteria

15.1. THE command framework SHALL register `UNDO` as a primary command that triggers the undo operation defined in Requirement 4. [FFE-UNDO-9]

15.2. THE command framework SHALL register `REDO` as a primary command that triggers the redo operation defined in Requirement 4. [FFE-UNDO-9]

15.3. THE `UNDO` command SHALL NOT be added to command history (it is a navigation command, not an edit command). [FFE-UNDO-9]

15.4. THE `REDO` command SHALL NOT be added to command history. [FFE-UNDO-9]

15.5. WHEN `UNDO` or `REDO` is issued in Browse mode or View mode, THE command framework SHALL display a status message indicating the command is not available in read-only mode and SHALL NOT modify any state. [FFE-UNDO-9]

15.6. THE command framework SHALL support `UNDO n` where `n` is a positive integer, causing `n` successive undo operations in a single command. WHEN `n` exceeds the number of available transactions, THE system SHALL undo all available transactions and display a message indicating the actual count. [FFE-UNDO-9]

15.7. THE command framework SHALL support `REDO n` analogously to `UNDO n`. [FFE-UNDO-9]

---

### Requirement 16: History Validation and Integrity

**User Story:** As an editor developer, I want the undo history to be validated for internal consistency, so that corrupted state is detected and handled gracefully rather than causing data loss.

**Source:** [SCI-UNDO-4.2] (Validate method, Delta calculation)

#### Acceptance Criteria

16.1. THE system SHALL provide a `validate(document_length: usize) -> bool` method that checks the internal consistency of the undo history against the current document size. [SCI-UNDO-4.2]

16.2. THE validation SHALL verify that: [SCI-UNDO-4.2]
   - The cumulative size delta (sum of inserts minus deletes up to current action) is consistent with the current document length minus the original length
   - No action references a position beyond the document bounds at its point in the action sequence
   - The cumulative document length never becomes negative at any point in the action sequence

16.3. IF validation fails (e.g., after loading a corrupted recovery file), THE system SHALL clear the undo history entirely (equivalent to `DeleteUndoHistory`) and log a warning via the logging subsystem. The document content SHALL NOT be modified. [SCI-UNDO-4.2]

16.4. THE system SHALL perform validation after restoring undo state from a recovery file, before allowing user interaction with the restored undo history. [SCI-UNDO-4.2], [FFE-UNDO-6]

---

### Requirement 17: Text Storage for Undo (Scrap Stack)

**User Story:** As an editor developer, I want undo text data stored in a memory-efficient contiguous buffer, so that the undo system minimises allocation overhead and cache misses when storing and retrieving text for thousands of edit operations.

**Source:** [SCI-UNDO-4.2] (ScrapStack), adapted to Rust idioms

#### Acceptance Criteria

17.1. THE system SHALL store all text data associated with Edit_Operations (inserted text that must be preserved for undo, deleted text that must be preserved for redo) in a contiguous byte buffer (Scrap_Stack) rather than individual heap allocations per operation. [SCI-UNDO-4.2]

17.2. THE Scrap_Stack SHALL maintain a current-position pointer that advances as new text is pushed and retreats during undo traversal, enabling sequential access to text data during undo/redo without random-access lookups. [SCI-UNDO-4.2]

17.3. WHEN an Edit_Operation is recorded, THE system SHALL append its text data to the Scrap_Stack and store the length (not a pointer or offset) in the action record. Text for any action is located by summing lengths of preceding actions. [SCI-UNDO-4.2]

17.4. WHEN `DeleteUndoHistory` is called, THE system SHALL clear the Scrap_Stack, releasing all text storage. [SCI-UNDO-4.2]

17.5. THE action record storage SHALL use scaled vectors (variable-width integer storage) for positions and lengths, choosing the minimum byte width that can represent the largest value in the collection. This reduces memory usage for histories containing only small edits. [SCI-UNDO-4.2]

---

### Requirement 18: Crate API and GUI Independence

**User Story:** As a workbench developer, I want the undo-redo-transactions crate to provide a clean, GUI-independent API that can be consumed by the command framework, document model, and any future GUI shell without coupling to rendering details.

**Source:** [WB] (GUI-independent crate, multi-crate workspace)

#### Acceptance Criteria

18.1. THE `ff-undo` crate SHALL have zero dependencies on GUI frameworks (no `egui`, `winit`, `eframe`, or platform-specific rendering crates). [WB]

18.2. THE crate SHALL expose its public API through a single `UndoManager` (or equivalent) type per document that encapsulates: undo stack, redo stack, save point, detach point, tentative state, coalescing state, selection history, and configuration. [WB]

18.3. THE crate SHALL communicate state changes to the GUI layer via a notification trait (observer pattern) rather than direct UI calls. Notifications SHALL include: dirty-flag changed, undo-available changed, redo-available changed, transaction committed, transaction undone, transaction redone. [WB]

18.4. THE crate SHALL be usable in a headless/test context without any GUI infrastructure — all functionality SHALL be exercisable through unit tests operating on the public API alone. [WB]

18.5. THE crate SHALL depend only on: `ff-logging` (diagnostics), `ff-configuration` (settings access), standard library, and serialisation crates (for recovery file I/O). It SHALL NOT depend on `ff-document-model` directly — instead it SHALL accept Edit_Operations via a trait interface, allowing the document model to implement the trait. [WB]

---
