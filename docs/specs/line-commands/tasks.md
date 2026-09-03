# Implementation Plan: Line Commands (`ff-line-commands`)

## Overview

This plan covers the complete implementation of the `ff-line-commands` crate — the ISPF line command engine for FileForgeWorkbench. The crate provides prefix-area command parsing, block pairing, pending state management, compatibility validation, and execution logic for all line commands: delete, insert, repeat, copy, move, after/before targets, exclude, tag/untag, shift, and bounds-aware shift.

This is a **Wave 5 (Command Engine)** sub-project that depends on Wave 4 (`ff-document-model`, `ff-edit-operations`, `ff-display-line-mapping`, `ff-undo-redo-transactions`) for buffer access, edit primitives, visibility state, and transaction wrapping, plus Wave 2 (`ff-command`) for command framework registration and dispatch.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-line-commands/Cargo.toml` with dependencies (thiserror, proptest dev-dep) and deps on `ff-document-model`, `ff-edit-operations`, `ff-display-line-mapping`, `ff-undo-redo-transactions`, `ff-command`, `ff-configuration-system`, `ff-logging`
  - [x] 1.2 Create `crates/ff-line-commands/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `parser.rs`, `command.rs`, `pending.rs`, `block_pair.rs`, `compatibility.rs`, `resolution.rs`, `config.rs`, `error.rs`
  - [x] 1.4 Create execution submodule: `execution/mod.rs`, `execution/delete.rs`, `execution/insert.rs`, `execution/repeat.rs`, `execution/copy.rs`, `execution/move_cmd.rs`, `execution/exclude.rs`, `execution/tag.rs`, `execution/shift_right.rs`, `execution/shift_left.rs`, `execution/bounds_shift.rs`
  - [x] 1.5 Create `commands/mod.rs` and `commands/handlers.rs` for command framework registration
  - [x] 1.6 Add `ff-line-commands` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Error types and configuration
  - [x] 2.1 Define `LineCommandError` enum with all variants (InvalidCommand, AwaitingPair, TooManyMarkers, OverlappingBlocks, TargetInsideSource, DuplicateTarget, IncompatibleCommands, SourceWithFilePath, NoBoundsActive, LineOutOfRange, AwaitingTarget, AwaitingSource, DocumentError) using thiserror
  - [x] 2.2 Define `LineCommandConfig` struct with `shift_width: u32` field and Default impl (default 2)
  - [x] 2.3 Implement configuration integration — read `editor.shift_width` from configuration system with hot-reload support
  - [x] 2.4 Write unit tests for error Display formatting verifying `[line-cmd] operation: description` format
  - Covers: Cross-cutting Requirement 8, Requirements 9.7, 10.7

- [x] 3. Command types and classification enums
  - [x] 3.1 Define `LineCommandKind` enum with all variants (Delete, DeleteCount, DeleteBlock, Insert, InsertCount, Repeat, RepeatCount, RepeatBlock, Copy, CopyBlock, Move, MoveBlock, After, Before, Exclude, ExcludeCount, ExcludeBlock, Tag, TagBlock, Untag, UntagBlock, ShiftRight, ShiftRightCount, ShiftRightBlock, ShiftLeft, ShiftLeftCount, ShiftLeftBlock, BoundsShiftRight, BoundsShiftRightBlock, BoundsShiftLeft, BoundsShiftLeftBlock)
  - [x] 3.2 Define `ParsedLineCommand` struct with `line: u64` and `kind: LineCommandKind`
  - [x] 3.3 Define `LineCommandCategory` enum (Immediate, Block, Source, Target) with classification logic
  - [x] 3.4 Define `BlockCommandKind` enum (Delete, Repeat, Exclude, Tag, Untag, ShiftRight, ShiftLeft, BoundsRight, BoundsLeft, Copy, Move)
  - [x] 3.5 Define `BlockPair` struct with `kind`, `start_line`, `end_line`
  - [x] 3.6 Define `SourceTarget` struct with `operation`, `source_start`, `source_end`, `target_line`, `target_position`
  - [x] 3.7 Define `SourceOperation` enum (Copy, Move) and `TargetPosition` enum (After, Before)
  - [x] 3.8 Define `ExecutableCommand` enum with all resolved command variants
  - [x] 3.9 Write unit tests for category classification of each LineCommandKind variant
  - Covers: Requirements 1–11, 12.1, 14.7

- [x] 4. Line command parser
  - [x] 4.1 Implement `LineCommandParser::parse()` — case-insensitive parsing of all valid line command strings to `ParsedLineCommand`
  - [x] 4.2 Implement numeric count extraction for D, I, R, X, >, < (e.g., "D5" → DeleteCount(5))
  - [x] 4.3 Implement block marker recognition for doubled commands (DD, RR, XX, TT, UU, >>, <<, )), (()
  - [x] 4.4 Implement `LineCommandParser::classify()` — map `LineCommandKind` to `LineCommandCategory`
  - [x] 4.5 Implement `LineCommandParser::is_block_marker()` helper
  - [x] 4.6 Return `LineCommandError::InvalidCommand` for unrecognised strings
  - [x] 4.7 Write unit tests for every recognised pattern (all variants, upper/lowercase, with/without counts)
  - [x] 4.8 Write unit tests for rejection of invalid inputs (gibberish, partial matches, empty strings)
  - Covers: Requirements 1–11 (parsing), 14.6

- [x] 5. Pending command store
  - [x] 5.1 Implement `PendingCommandStore::new()` with empty HashMap and counter
  - [x] 5.2 Implement `PendingCommandStore::add()` — store command with PendingReason and monotonic timestamp
  - [x] 5.3 Implement `PendingCommandStore::remove()` — remove command at a line, return Option
  - [x] 5.4 Implement `PendingCommandStore::get()` — lookup by line number
  - [x] 5.5 Implement `PendingCommandStore::by_category()` — filter all pending by LineCommandCategory
  - [x] 5.6 Implement `PendingCommandStore::pending_sources()` and `pending_targets()` — specialized queries
  - [x] 5.7 Implement `PendingCommandStore::pending_blocks()` — filter by specific block kind
  - [x] 5.8 Implement `PendingCommandStore::clear_all()` — reset all pending state
  - [x] 5.9 Implement `PendingCommandStore::all_pending()` — iterator over all entries
  - [x] 5.10 Implement `count()` and `is_empty()` helpers
  - [x] 5.11 Write unit tests for add/remove/query/clear operations
  - Covers: Requirement 14 (all criteria)

- [x] 6. Block pair validator
  - [x] 6.1 Implement `BlockPairValidator::normalize()` — ensure start ≤ end regardless of entry order
  - [x] 6.2 Implement `BlockPairValidator::validate_pair()` — form BlockPair from exactly two matching markers in pending store
  - [x] 6.3 Implement single-marker detection — return AwaitingPair error when only one marker present
  - [x] 6.4 Implement excess-marker detection — return TooManyMarkers error when >2 markers of same type present
  - [x] 6.5 Implement `BlockPairValidator::check_overlaps()` — detect overlapping ranges from different block types
  - [x] 6.6 Write unit tests for normalization, valid pairs, single marker pending, too many markers, and overlaps
  - Covers: Requirement 12 (all criteria)

- [x] 7. Compatibility matrix
  - [x] 7.1 Implement `CommandCompatibilityMatrix::check_compatibility()` — validate primary command against pending line commands
  - [x] 7.2 Define compatibility rules: COPY primary + C/CC source + A/B target = valid; COPY path + C/CC = error
  - [x] 7.3 Define compatibility rules: MOVE primary + M/MM source + A/B target = valid; MOVE path + M/MM = error
  - [x] 7.4 Implement `CommandCompatibilityMatrix::all_immediate()` — check if all pending are immediate commands
  - [x] 7.5 Implement blank primary command rule: only immediate commands may execute without a primary command
  - [x] 7.6 Write unit tests for compatible/incompatible combinations and error messages
  - Covers: Requirement 13 (all criteria)

- [x] 8. Resolution engine
  - [x] 8.1 Implement `ResolutionEngine::resolve()` — main entry point processing new inputs and existing pending state
  - [x] 8.2 Implement parsing of new prefix-area inputs into pending store (step 1 of resolution)
  - [x] 8.3 Implement block pair resolution — form BlockPair when two markers present (step 2)
  - [x] 8.4 Implement source+target resolution — form SourceTarget when both C/CC/M/MM and A/B present (step 3)
  - [x] 8.5 Implement compatibility check against primary command (step 4)
  - [x] 8.6 Implement immediate command extraction — resolve D, Dn, I, In, R, Rn, X, Xn, T, U, >, <, ), ( without primary command
  - [x] 8.7 Define `ResolutionResult` struct with `executable`, `errors`, `still_pending` fields
  - [x] 8.8 Write unit tests for resolution of immediate commands, block pairs, source+target, and mixed scenarios
  - Covers: Requirements 6.3, 6.4, 12–14

- [x] 9. Delete execution
  - [x] 9.1 Implement `ExecutionEngine::execute_delete()` — remove lines from document, return EditorTransaction
  - [x] 9.2 Handle single-line delete (D)
  - [x] 9.3 Handle counted delete (Dn) — delete n consecutive lines
  - [x] 9.4 Handle block delete (DD pair) — delete range [start, end]
  - [x] 9.5 Validate line range is within document bounds; return LineOutOfRange on failure
  - [x] 9.6 Write unit tests for single, counted, block delete, and out-of-range error
  - Covers: Requirement 1 (AC 1.1–1.6)

- [x] 10. Insert execution
  - [x] 10.1 Implement `ExecutionEngine::execute_insert()` — insert blank lines after specified line, return EditorTransaction
  - [x] 10.2 Handle single insert (I) — one blank line
  - [x] 10.3 Handle counted insert (In) — n blank lines
  - [x] 10.4 Validate insertion point is within document bounds
  - [x] 10.5 Write unit tests for single insert, counted insert, and boundary validation
  - Covers: Requirement 2 (AC 2.1–2.4)

- [x] 11. Repeat execution
  - [x] 11.1 Implement `ExecutionEngine::execute_repeat()` — duplicate line(s) in place, return EditorTransaction
  - [x] 11.2 Handle single repeat (R) — one duplicate after source line
  - [x] 11.3 Handle counted repeat (Rn) — n duplicates after source line
  - [x] 11.4 Implement `ExecutionEngine::execute_repeat_block()` — duplicate entire block range and insert after last line of block
  - [x] 11.5 Validate source line range is within document bounds
  - [x] 11.6 Write unit tests for single, counted, block repeat, and content verification
  - Covers: Requirement 3 (AC 3.1–3.6)

- [x] 12. Copy execution
  - [x] 12.1 Implement `ExecutionEngine::execute_copy()` — copy source lines to target position, return EditorTransaction
  - [x] 12.2 Handle single-line copy (C + A/B)
  - [x] 12.3 Handle block copy (CC pair + A/B) — copy entire range
  - [x] 12.4 Handle After target — insert copies after target line
  - [x] 12.5 Handle Before target — insert copies before target line
  - [x] 12.6 Verify source lines are unchanged after copy
  - [x] 12.7 Write unit tests for single/block copy with A and B targets
  - Covers: Requirement 4 (AC 4.1–4.6), Requirement 6 (AC 6.1–6.3)

- [x] 13. Move execution
  - [x] 13.1 Implement `ExecutionEngine::execute_move()` — remove source lines and insert at target position, return EditorTransaction
  - [x] 13.2 Handle single-line move (M + A/B)
  - [x] 13.3 Handle block move (MM pair + A/B) — move entire range
  - [x] 13.4 Validate target is not inside source block — return TargetInsideSource error
  - [x] 13.5 Handle After target — insert moved lines after target line
  - [x] 13.6 Handle Before target — insert moved lines before target line
  - [x] 13.7 Verify document line count is unchanged after move
  - [x] 13.8 Write unit tests for single/block move, target-inside-source rejection, and line count preservation
  - Covers: Requirement 5 (AC 5.1–5.7), Requirement 6 (AC 6.1–6.5)

- [x] 14. Exclude execution
  - [x] 14.1 Implement `ExecutionEngine::execute_exclude()` — set excluded flag on lines via DisplayLineMapping
  - [x] 14.2 Handle single exclude (X) — one line
  - [x] 14.3 Handle counted exclude (Xn) — n consecutive lines
  - [x] 14.4 Handle block exclude (XX pair) — range [start, end]
  - [x] 14.5 Verify operation does NOT produce an EditorTransaction (session-state only, bypasses undo)
  - [x] 14.6 Write unit tests for single, counted, block exclude, and no-transaction verification
  - Covers: Requirement 7 (AC 7.1–7.6)

- [x] 15. Tag and Untag execution
  - [x] 15.1 Implement `ExecutionEngine::execute_tag()` — set tagged flag on lines
  - [x] 15.2 Handle single tag (T) and block tag (TT pair)
  - [x] 15.3 Implement `ExecutionEngine::execute_untag()` — clear tagged flag on lines
  - [x] 15.4 Handle single untag (U) and block untag (UU pair)
  - [x] 15.5 Verify operations do NOT produce EditorTransactions (session-state only, bypasses undo)
  - [x] 15.6 Write unit tests for tag/untag single and block, and no-transaction verification
  - Covers: Requirement 8 (AC 8.1–8.8)

- [x] 16. Shift right execution
  - [x] 16.1 Implement `ExecutionEngine::execute_shift_right()` — prepend spaces to line content, return EditorTransaction
  - [x] 16.2 Handle single shift (>) — shift by configured ShiftWidth
  - [x] 16.3 Handle counted shift (>n) — shift by n columns
  - [x] 16.4 Handle block shift (>> pair) — shift all lines in range by ShiftWidth
  - [x] 16.5 Read ShiftWidth from LineCommandConfig
  - [x] 16.6 Write unit tests for single, counted, block shift right with content verification
  - Covers: Requirement 9 (AC 9.1–9.7)

- [x] 17. Shift left execution
  - [x] 17.1 Implement `ExecutionEngine::execute_shift_left()` — remove leading whitespace, return EditorTransaction
  - [x] 17.2 Handle single shift (<) — shift by configured ShiftWidth
  - [x] 17.3 Handle counted shift (<n) — shift by n columns
  - [x] 17.4 Handle block shift (<< pair) — shift all lines in range by ShiftWidth
  - [x] 17.5 Implement data-loss prevention: truncate only up to first non-whitespace character
  - [x] 17.6 Read ShiftWidth from LineCommandConfig
  - [x] 17.7 Write unit tests for single, counted, block shift left, and non-destructive truncation verification
  - Covers: Requirement 10 (AC 10.1–10.8)

- [x] 18. Bounds-aware shift execution
  - [x] 18.1 Implement `ExecutionEngine::execute_bounds_shift_right()` — shift content within bounds right by one position, preserve content outside bounds
  - [x] 18.2 Implement `ExecutionEngine::execute_bounds_shift_left()` — shift content within bounds left by one position, preserve content outside bounds
  - [x] 18.3 Handle block bounds-shift right ()) pair) — apply to all lines in range
  - [x] 18.4 Handle block bounds-shift left ((( pair) — apply to all lines in range
  - [x] 18.5 Validate active bounds are set — return NoBoundsActive error if not
  - [x] 18.6 Return EditorTransaction for successful operations
  - [x] 18.7 Write unit tests for bounds-shift right/left, block variants, no-bounds error, and outer content preservation
  - Covers: Requirement 11 (AC 11.1–11.8)

- [x] 19. Command framework integration
  - [x] 19.1 Register all line command operations as commands with `CommandRegistry` (linecmd.delete, linecmd.insert, linecmd.repeat, linecmd.copy, linecmd.move, linecmd.exclude, linecmd.tag, linecmd.untag, linecmd.shift_right, linecmd.shift_left, linecmd.bounds_shift_right, linecmd.bounds_shift_left, linecmd.resolve_cycle, linecmd.reset)
  - [x] 19.2 Implement `CommandHandler` trait for each registered command
  - [x] 19.3 Wire undoable operations to return `CommandResult::OkUndoable` with transaction
  - [x] 19.4 Wire session-state operations (exclude, tag, untag) to return `CommandResult::Ok` without undo records
  - [x] 19.5 Implement `linecmd.resolve_cycle` command — main entry point invoked by primary command execution cycle
  - [x] 19.6 Implement `linecmd.reset` command — clear all pending commands (RESET COMMANDS / RESET ALL)
  - [x] 19.7 Write unit tests verifying command dispatch and undo record production
  - Covers: Requirement 14.8, Cross-cutting Requirement 4

- [x] 20. Integration tests
  - [x] 20.1 Write end-to-end test: enter D3 on line 5 → resolve → verify 3 lines deleted starting at line 5
  - [x] 20.2 Write end-to-end test: enter CC on line 2, CC on line 5, A on line 8 → resolve → verify copy of lines 2–5 inserted after line 8
  - [x] 20.3 Write end-to-end test: enter MM on line 3, MM on line 6, B on line 1 → resolve → verify move of lines 3–6 inserted before line 1
  - [x] 20.4 Write end-to-end test: enter >> on line 4, >> on line 7 → resolve → verify shift right on lines 4–7
  - [x] 20.5 Write end-to-end test: enter M on line 5, A on line 5 (target inside source) → verify TargetInsideSource error
  - [x] 20.6 Write end-to-end test: enter RR on line 2, no second RR → verify pending state retained with AwaitingPair reason
  - [x] 20.7 Write end-to-end test: verify RESET COMMANDS clears all pending state
  - [x] 20.8 Write end-to-end test: enter incompatible primary command with pending line commands → verify error
  - Covers: Requirements 1–14 (cross-requirement integration scenarios)

- [x] 21. Property-based tests
  - [x] 21.1 Write property test: Parser Round-Trip Consistency (Property 1)
  - [x] 21.2 Write property test: Block Pair Normalization (Property 2)
  - [x] 21.3 Write property test: Delete Preserves Document Integrity (Property 3)
  - [x] 21.4 Write property test: Insert Line Count (Property 4)
  - [x] 21.5 Write property test: Repeat Produces Exact Duplicates (Property 5)
  - [x] 21.6 Write property test: Shift Right Adds Exactly N Spaces (Property 6)
  - [x] 21.7 Write property test: Shift Left Non-Destructive (Property 7)
  - [x] 21.8 Write property test: Copy Does Not Modify Source (Property 8)
  - [x] 21.9 Write property test: Move Preserves Line Count (Property 9)
  - [x] 21.10 Write property test: Bounds-Aware Shift Preserves Outer Content (Property 10)
  - [x] 21.11 Write property test: Pending Store Size Monotonicity on Clear (Property 11)
  - [x] 21.12 Write property test: Resolution Engine Idempotence for Pending-Only State (Property 12)
  - [x] 21.13 Write property test: Compatibility Matrix Symmetry (Property 13)
  - Covers: All correctness properties from design.md

---

## Property-Based Test Definitions

### Property 1: Parser Round-Trip Consistency

**Validates: Requirements 1.1, 2.1, 3.1, 4.1, 5.1, 6.1, 7.1, 8.1, 9.1, 10.1, 11.1, 14.7**

- **Statement:** For any valid line command string that `LineCommandParser::parse` accepts, the resulting `ParsedLineCommand` can be classified into exactly one `LineCommandCategory`, and the category is deterministic for a given `LineCommandKind`.
- **Strategy:** Generate valid line command strings from all recognised patterns (D, Dn, DD, I, In, R, Rn, RR, C, CC, M, MM, A, B, X, Xn, XX, T, TT, U, UU, >, >n, >>, <, <n, <<, ), )), (, (() with random counts [1, 999] and random case.
- **Invariant:** `classify(parse(input).kind) ∈ {Immediate, Block, Source, Target}` — same input always produces same category.

### Property 2: Block Pair Normalization

**Validates: Requirements 12.2**

- **Statement:** For any two line numbers used as block markers, `BlockPairValidator::normalize` always produces `start_line ≤ end_line`, and the resulting pair spans exactly `(end_line - start_line + 1)` lines.
- **Strategy:** Generate pairs `(line1, line2)` where each value is in [0, 100_000].
- **Invariant:** `let (s, e) = normalize(l1, l2); s <= e ∧ (e - s + 1) == max(l1, l2) - min(l1, l2) + 1`

### Property 3: Delete Preserves Document Integrity

**Validates: Requirements 1.1, 1.2, 1.3**

- **Statement:** After executing a delete command on n lines starting at line L in a document with T lines (where L + n ≤ T), the resulting document has exactly T - n lines, and all lines outside the deleted range retain their original content.
- **Strategy:** Generate documents with [1, 200] lines of random content, random L in [0, T-1], random n in [1, T-L].
- **Invariant:** `doc.line_count() == T - n ∧ lines before L unchanged ∧ lines after L+n shifted up by n`

### Property 4: Insert Line Count

**Validates: Requirements 2.1, 2.2**

- **Statement:** After executing an insert of n blank lines after line L in a document with T lines, the document has exactly T + n lines. All original lines retain their content (shifted by n after the insertion point).
- **Strategy:** Generate documents with [1, 200] lines, random L in [0, T-1], random n in [1, 50].
- **Invariant:** `doc.line_count() == T + n ∧ lines ≤ L unchanged ∧ inserted lines are blank ∧ lines > L+n shifted down by n`

### Property 5: Repeat Produces Exact Duplicates

**Validates: Requirements 3.1, 3.2**

- **Statement:** After executing a repeat of line L with count n, the document has n additional lines immediately after L, and each inserted line has identical content to the original line L.
- **Strategy:** Generate documents with [1, 100] lines, random L in [0, T-1], random n in [1, 20].
- **Invariant:** `doc.line_count() == T + n ∧ ∀ i in 1..=n: line_content(L+i) == original_content(L)`

### Property 6: Shift Right Adds Exactly N Spaces

**Validates: Requirements 9.1, 9.2**

- **Statement:** After executing a shift-right of n columns on line L, the line content is the original content prefixed with exactly n space characters.
- **Strategy:** Generate documents with [1, 100] lines of random content (0–80 chars), random L in [0, T-1], random n in [1, 40].
- **Invariant:** `line_content(L) == " ".repeat(n) + &original_content(L)`

### Property 7: Shift Left Non-Destructive

**Validates: Requirements 10.1, 10.2, 10.8**

- **Statement:** After executing a shift-left of n columns on a line, if the line has at least n leading whitespace characters, the result has those n characters removed. If fewer than n leading whitespace characters exist, content is shifted only up to the first non-whitespace character — no non-whitespace content is ever lost.
- **Strategy:** Generate lines with random leading whitespace [0, 40 spaces] followed by random non-whitespace content, random n in [1, 50].
- **Invariant:** `let shifted = actual_shift = min(n, leading_ws_count); result == original[shifted..]`

### Property 8: Copy Does Not Modify Source

**Validates: Requirements 4.3**

- **Statement:** After executing a copy operation from source lines [S_start, S_end] to target T, the content at the original source positions is unchanged, and the document grows by exactly (S_end - S_start + 1) lines.
- **Strategy:** Generate documents with [5, 100] lines, random source range within bounds, random target outside source range.
- **Invariant:** `doc.line_count() == T_before + (S_end - S_start + 1) ∧ source content unchanged (adjusted for position shift)`

### Property 9: Move Preserves Line Count

**Validates: Requirements 5.3, 5.4**

- **Statement:** After executing a move operation from source lines [S_start, S_end] to target T (where target is outside the source range), the total document line count remains unchanged, and the source content appears at the new target position.
- **Strategy:** Generate documents with [5, 100] lines, random source range, random target outside [S_start, S_end].
- **Invariant:** `doc.line_count() == T_before ∧ source_content appears contiguously at adjusted target position`

### Property 10: Bounds-Aware Shift Preserves Outer Content

**Validates: Requirements 11.1, 11.3**

- **Statement:** After executing a bounds-aware shift right on line L with bounds [left, right], all characters outside columns [left, right] are unchanged.
- **Strategy:** Generate lines of [10, 120] characters, random bounds where left < right and both within line length, random L in valid range.
- **Invariant:** `shifted[..left-1] == original[..left-1] ∧ shifted[right..] == original[right..]`

### Property 11: Pending Store Size Monotonicity on Clear

**Validates: Requirements 14.1, 14.2, 14.5**

- **Statement:** After `clear_all()` is called on the PendingCommandStore, the store is empty (count == 0). Adding n commands results in count == n. Removing one command decrements count by 1.
- **Strategy:** Generate sequences of add/remove/clear operations with random commands and line numbers.
- **Invariant:** `after clear: count == 0 ∧ after n adds: count == n ∧ after remove(existing): count == n - 1`

### Property 12: Resolution Engine Idempotence for Pending-Only State

**Validates: Requirements 13.4, 14.3**

- **Statement:** If the resolution engine is called with no new inputs and no primary command, the pending store does not change (commands remain pending). No commands are executed.
- **Strategy:** Generate pending stores containing only source markers (C, CC, M, MM) or unpaired block markers — no immediate commands.
- **Invariant:** `resolve([], &mut store, None).executable.is_empty() ∧ store unchanged`

### Property 13: Compatibility Matrix Symmetry

**Validates: Requirements 13.1, 13.2, 13.3**

- **Statement:** If a primary command P is incompatible with a line command set S, then `check_compatibility(Some(P), S)` always returns Err. If compatible, it always returns Ok.
- **Strategy:** Generate (primary_command, pending_commands) pairs from known incompatible/compatible combinations defined in the matrix.
- **Invariant:** `incompatible → Err ∧ compatible → Ok` for all generated pairs.

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Types and Configuration", "tasks": ["2", "3"], "dependsOn": [0] },
    { "id": 2, "label": "Parser", "tasks": ["4"], "dependsOn": [1] },
    { "id": 3, "label": "State Management", "tasks": ["5", "6", "7"], "dependsOn": [2] },
    { "id": 4, "label": "Resolution", "tasks": ["8"], "dependsOn": [3] },
    { "id": 5, "label": "Execution — Document Mutations", "tasks": ["9", "10", "11", "12", "13"], "dependsOn": [4] },
    { "id": 6, "label": "Execution — Session State", "tasks": ["14", "15"], "dependsOn": [4] },
    { "id": 7, "label": "Execution — Shift Operations", "tasks": ["16", "17", "18"], "dependsOn": [4] },
    { "id": 8, "label": "Command Framework Integration", "tasks": ["19"], "dependsOn": [5, 6, 7] },
    { "id": 9, "label": "Integration Tests", "tasks": ["20"], "dependsOn": [8] },
    { "id": 10, "label": "Property-Based Tests", "tasks": ["21"], "dependsOn": [9] }
  ]
}
```

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Delete Line Commands (D, Dn, DD) | AC 1.1–1.6 | Tasks 4, 8, 9, 21.3 |
| Req 2: Insert Line Commands (I, In) | AC 2.1–2.4 | Tasks 4, 8, 10, 21.4 |
| Req 3: Repeat Line Commands (R, Rn, RR) | AC 3.1–3.6 | Tasks 4, 8, 11, 21.5 |
| Req 4: Copy Markers (C, CC) | AC 4.1–4.6 | Tasks 4, 5, 8, 12, 21.8 |
| Req 5: Move Markers (M, MM) | AC 5.1–5.7 | Tasks 4, 5, 8, 13, 21.9 |
| Req 6: After/Before Target Markers (A, B) | AC 6.1–6.5 | Tasks 4, 5, 8, 12, 13 |
| Req 7: Exclude Line Commands (X, Xn, XX) | AC 7.1–7.6 | Tasks 4, 8, 14 |
| Req 8: Tag/Untag Line Commands (T, TT, U, UU) | AC 8.1–8.8 | Tasks 4, 8, 15 |
| Req 9: Shift Right (>, >n, >>) | AC 9.1–9.7 | Tasks 4, 8, 16, 21.6 |
| Req 10: Shift Left (<, <n, <<) | AC 10.1–10.8 | Tasks 4, 8, 17, 21.7 |
| Req 11: Bounds-Aware Shift (), )), (, (( | AC 11.1–11.8 | Tasks 4, 8, 18, 21.10 |
| Req 12: Block Command Pairing | AC 12.1–12.7 | Tasks 3, 6, 8, 21.2 |
| Req 13: Command Compatibility Validation | AC 13.1–13.7 | Tasks 7, 8, 21.13 |
| Req 14: Pending Command State Management | AC 14.1–14.8 | Tasks 5, 8, 19, 21.11, 21.12 |
| Cross-cutting Req 4: Command-Driven Architecture | — | Task 19 |
| Cross-cutting Req 7: Multi-Crate Workspace Structure | — | Task 1 |
| Cross-cutting Req 8: Error Message Standards | — | Task 2 |

---

## Notes

- This is a Wave 5 (Command Engine) crate depending on `ff-document-model`, `ff-edit-operations`, `ff-display-line-mapping`, `ff-undo-redo-transactions` (Wave 4) for buffer access, edit primitives, visibility state, and transaction wrapping
- The `ff-command` (Wave 2) dependency is for command framework registration and dispatch
- The prefix-area UI rendering is the responsibility of `ff-desktop` (Shell Layer) — this crate is purely logical
- Session-state operations (exclude, tag/untag) bypass the undo stack intentionally
- The `DisplayLineMapping` trait is the interface boundary with `ff-display-line-mapping` — this crate depends on the trait, not the concrete implementation
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The BOUNDS/BNDS state is read from session state set by `ff-navigation-commands` — this crate reads but never modifies bounds
- The `ff-exclude-show-filter` crate is downstream and handles SHOW/INCLUDE/RESET restoration — this crate only sets `excluded = true`
