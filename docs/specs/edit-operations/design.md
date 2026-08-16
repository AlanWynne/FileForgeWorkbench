# Design Document: Edit Operations (`ff-edit-operations`)

## Overview

The `ff-edit-operations` crate implements all text editing behaviour for the FileForgeWorkbench editor. It sits between the low-level document buffer (`ff-document-model`) and the user-facing command dispatch (`ff-command`), providing:

- **Edit mode management** — Insert, Overstrike, and Browse mode state machine
- **Character insertion and deletion** — single character, word, line, and range operations
- **Selection model** — stream, rectangular, and multi-caret selection with position adjustment
- **Multi-caret coordination** — simultaneous editing at multiple positions with reverse-order processing
- **Edit boundaries (BOUNDS)** — ISPF-heritage column-range protection
- **Line manipulation** — transpose, duplicate, case change
- **Transaction recording** — defining undo boundaries and grouping multi-caret operations
- **Clipboard integration** — edit-side cut/copy/paste semantics for all selection types

### Position in Architecture

```
Wave 4 — Core Editor

┌──────────────────────────────────────────────────────────┐
│                 ff-command (Wave 2)                        │
│          Command dispatch, shortcut resolution            │
├──────────────────────────────────────────────────────────┤
│            ff-edit-operations (this crate)                 │
│   Edit modes, insertion, deletion, selection, carets      │
├──────────────────────────────────────────────────────────┤
│            ff-document-model (Wave 4, upstream)            │
│     Gap buffer, line index, character navigation          │
├──────────────────────────────────────────────────────────┤
│        ff-undo-redo-transactions (Wave 4, peer)           │
│   TransactionStack, coalescing, UndoGroup, save point     │
└──────────────────────────────────────────────────────────┘
```


### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Zero GUI dependencies — operates on abstract document/selection types only
- **Command-Driven (Req 4)**: All edit operations are registered commands dispatched via `ff-command`
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-edit-operations`
- **Error Message Standards (Req 8)**: All errors follow `[edit] operation: description` format
- **Async I/O (Req 6)**: Save operation delegates to VFS async path via `ff-document-model`

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Input Sources (via ff-command)"
        A[Keyboard → Shortcut Registry]
        B[Menu / Toolbar]
        C[Lua Macro Script]
        D[Command Palette]
    end

    subgraph "ff-edit-operations"
        E[EditModeManager<br/>Insert/Overstrike/Browse]
        F[InsertionEngine<br/>char, tab, newline, virtual space]
        G[DeletionEngine<br/>char, word, line, range]
        H[SelectionContainer<br/>ranges, main, trim, adjust]
        I[MultiCaretCoordinator<br/>reverse-order dispatch]
        J[BoundsEnforcer<br/>column-range protection]
        K[LineManipulator<br/>transpose, duplicate, case]
        L[ClipboardSemantics<br/>cut/copy/paste logic]
        M[TransactionRecorder<br/>EditorTransaction, UndoGroup]
    end

    subgraph "Downstream"
        N[ff-document-model<br/>TextBuffer, Document, LineIndex]
        O[ff-undo-redo-transactions<br/>TransactionStack]
        P[ff-command<br/>CommandRegistry, UndoRecord]
    end

    A --> E
    B --> F
    C --> G
    D --> H

    E --> F
    E --> G
    F --> J
    G --> J
    F --> I
    G --> I
    I --> H
    F --> M
    G --> M
    K --> M
    L --> F
    L --> G

    F --> N
    G --> N
    H --> N
    M --> O
    E --> P
end
```


### Layer Placement

| Layer | Role |
|-------|------|
| **Command Layer** | Edit command handlers registered with `ff-command`; translate `CommandParams` → engine calls |
| **Mode Layer** | `EditModeManager` gates operations by current mode (Insert/Overstrike/Browse) |
| **Bounds Layer** | `BoundsEnforcer` validates column positions before allowing edits |
| **Engine Layer** | `InsertionEngine`, `DeletionEngine`, `LineManipulator` — core edit logic |
| **Selection Layer** | `SelectionContainer` manages caret/anchor positions, adjustment on modification |
| **Coordination Layer** | `MultiCaretCoordinator` dispatches edits across multiple carets in reverse order |
| **Transaction Layer** | `TransactionRecorder` wraps edits into `EditorTransaction` / `UndoGroup` for undo system |

---

## Components and Interfaces

```
crates/ff-edit-operations/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── mode.rs                 # EditMode enum, EditModeManager
│   ├── position.rs             # SelectionPosition (real + virtual space)
│   ├── range.rs                # SelectionRange (anchor, caret)
│   ├── selection.rs            # SelectionContainer (Add, Drop, Trim, MovePositions)
│   ├── insertion.rs            # InsertionEngine — character, tab, newline, virtual space
│   ├── deletion.rs             # DeletionEngine — char, word, line, range granularities
│   ├── multi_caret.rs          # MultiCaretCoordinator — reverse-order dispatch
│   ├── bounds.rs               # BoundsEnforcer — ISPF column-range protection
│   ├── line_ops.rs             # LineManipulator — transpose, duplicate, case change
│   ├── clipboard.rs            # ClipboardSemantics — cut/copy/paste edit-side logic
│   ├── transaction.rs          # TransactionRecorder, EditorTransaction, UndoGroup bridge
│   ├── commands/
│   │   ├── mod.rs              # Re-exports for all command handlers
│   │   ├── insert_char.rs      # edit.insert_char command handler
│   │   ├── delete.rs           # edit.delete_* command handlers (back, forward, word, line)
│   │   ├── newline.rs          # edit.newline command handler
│   │   ├── mode_toggle.rs      # edit.toggle_mode command handler
│   │   ├── line_ops.rs         # edit.line_transpose, edit.line_duplicate, edit.case_*
│   │   ├── selection.rs        # edit.select_all, edit.select_next_occurrence
│   │   ├── clipboard.rs        # edit.cut, edit.copy, edit.paste
│   │   ├── bounds.rs           # edit.bounds command handler
│   │   └── caret.rs            # edit.add_caret_above, edit.add_caret_below, edit.clear_carets
│   ├── error.rs                # EditError enum
│   └── markers.rs              # ModifiedLineTracker — per-line modification state
└── tests/
    ├── mode_tests.rs           # Edit mode property tests
    ├── insertion_tests.rs      # Insertion property tests
    ├── deletion_tests.rs       # Deletion property tests
    ├── selection_tests.rs      # Selection container property tests
    ├── multi_caret_tests.rs    # Multi-caret coordination property tests
    ├── bounds_tests.rs         # BOUNDS enforcement property tests
    ├── line_ops_tests.rs       # Line manipulation property tests
    ├── clipboard_tests.rs      # Clipboard semantics property tests
    ├── transaction_tests.rs    # Transaction recording property tests
    └── integration.rs          # End-to-end edit scenarios
```


---

## Data Models

### EditMode

```rust
/// The current editing mode for an editor instance.
/// Addresses: Requirements 1, 3; Criteria 1.4, 3.3, 3.8
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditMode {
    /// Characters are inserted at the caret, pushing text rightward.
    Insert,
    /// Characters replace the character at the caret position.
    Overstrike,
    /// Document is read-only; no edits permitted. Navigation only.
    Browse,
}

impl Default for EditMode {
    fn default() -> Self { EditMode::Insert }
}
```

### EditModeManager

```rust
/// Manages per-editor-instance edit mode state.
/// Addresses: Requirements 1.4, 3.3, 3.4, 3.8
pub struct EditModeManager {
    mode: EditMode,
}

impl EditModeManager {
    pub fn new() -> Self;
    pub fn mode(&self) -> EditMode;
    pub fn set_mode(&mut self, mode: EditMode);
    pub fn toggle_insert_overstrike(&mut self);
    pub fn is_editable(&self) -> bool; // true if Insert or Overstrike
}
```

### SelectionPosition

```rust
/// A document position that includes both a real position and
/// a virtual space offset for positions beyond line ends.
/// Addresses: Requirement 6, criteria 6.1, 6.2; Requirement 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectionPosition {
    /// 0-based line number in the document.
    pub line: u64,
    /// 0-based column offset (byte offset within the line's content).
    pub column: u64,
    /// Virtual space columns beyond the end of the line's actual content.
    /// Non-negative. When > 0, the caret is in virtual space.
    pub virtual_space: u64,
}

impl SelectionPosition {
    pub fn new(line: u64, column: u64) -> Self;
    pub fn with_virtual_space(line: u64, column: u64, vs: u64) -> Self;
    pub fn effective_column(&self) -> u64; // column + virtual_space
    pub fn is_in_virtual_space(&self) -> bool;
}
```


### SelectionRange

```rust
/// An ordered pair (anchor, caret) defining a contiguous selected region.
/// The selected text spans between anchor and caret regardless of
/// document order (anchor may be after caret for backward selections).
/// Addresses: Requirement 6, criterion 6.1; Requirement 14
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionRange {
    /// The fixed end of the selection (start point).
    pub anchor: SelectionPosition,
    /// The moving end of the selection (cursor position).
    pub caret: SelectionPosition,
}

impl SelectionRange {
    pub fn new(anchor: SelectionPosition, caret: SelectionPosition) -> Self;
    pub fn collapsed(position: SelectionPosition) -> Self; // anchor == caret
    pub fn is_collapsed(&self) -> bool;
    pub fn start(&self) -> SelectionPosition; // min(anchor, caret)
    pub fn end(&self) -> SelectionPosition;   // max(anchor, caret)
    pub fn contains(&self, pos: &SelectionPosition) -> bool;
    pub fn overlaps(&self, other: &SelectionRange) -> bool;
    pub fn merge(&self, other: &SelectionRange) -> SelectionRange;
}
```

### SelectionContainer

```rust
/// The top-level structure holding all active SelectionRanges.
/// Maintains ranges sorted by document position with a designated main range.
/// Addresses: Requirement 6, criterion 6.3; Requirement 14, all criteria
pub struct SelectionContainer {
    ranges: Vec<SelectionRange>,
    main_index: usize,
}

impl SelectionContainer {
    pub fn new(initial: SelectionRange) -> Self;

    /// Add a new range, maintaining sorted order. (Req 14.1)
    pub fn add(&mut self, range: SelectionRange);

    /// Remove range at index. Fails if it would leave zero ranges. (Req 14.2)
    pub fn drop_range(&mut self, index: usize) -> Result<(), EditError>;

    /// Remove duplicate/overlapping ranges by merging. (Req 14.3)
    pub fn trim(&mut self);

    /// Adjust all positions given a document modification. (Req 14.4, Req 7)
    pub fn move_positions(&mut self, modification: &DocumentModification);

    /// Get the main (primary) selection range. (Req 14.5)
    pub fn main_range(&self) -> &SelectionRange;

    /// Set which range index is the main range. (Req 14.6)
    pub fn set_main_range(&mut self, index: usize);

    /// Iterate all ranges in document order. (Req 14.7)
    pub fn ranges(&self) -> &[SelectionRange];

    /// Iterate all ranges in reverse document order (for multi-caret edits).
    pub fn ranges_reverse(&self) -> impl Iterator<Item = &SelectionRange>;

    /// Number of active selections. (Req 14.8)
    pub fn count(&self) -> usize;

    /// Collapse to a single caret (the main range's caret). (Req 8.9)
    pub fn clear_to_main(&mut self);

    /// Check if multi-caret mode is active.
    pub fn is_multi_caret(&self) -> bool;
}
```


### SelectionKind

```rust
/// Distinguishes the kind of selection active in the editor.
/// Addresses: Requirements 6, 9
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionKind {
    /// Normal stream selection flowing across line boundaries.
    Stream,
    /// Rectangular (column) selection defined by corner positions.
    Rectangular,
}
```

### DocumentModification

```rust
/// Descriptor for a document change, used by SelectionContainer::move_positions.
/// Addresses: Requirement 7, all criteria
#[derive(Debug, Clone, Copy)]
pub struct DocumentModification {
    /// Byte offset where the modification occurred.
    pub offset: u64,
    /// Line number where the modification occurred.
    pub line: u64,
    /// Column where the modification occurred.
    pub column: u64,
    /// Number of characters (columns) inserted at the offset.
    pub inserted_length: u64,
    /// Number of characters (columns) deleted at the offset.
    pub deleted_length: u64,
    /// Number of lines inserted (for line splits).
    pub lines_inserted: u64,
    /// Number of lines deleted (for line joins).
    pub lines_deleted: u64,
}
```

### EditBounds

```rust
/// ISPF-style column boundaries that restrict where edits can be applied.
/// Addresses: Requirement 13, all criteria
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditBounds {
    /// Left boundary column (1-based, inclusive). Must be >= 1.
    pub left: u64,
    /// Right boundary column (1-based, inclusive). Must be > left.
    pub right: u64,
}

impl EditBounds {
    /// Create new bounds with validation. Returns None if invalid.
    /// Addresses: Requirement 13, criterion 13.12
    pub fn new(left: u64, right: u64) -> Option<Self>;

    /// Check if a column position (1-based) is within the bounds.
    pub fn contains_column(&self, col: u64) -> bool;

    /// Clamp a range to fit within bounds.
    pub fn clamp_range(&self, start_col: u64, end_col: u64) -> (u64, u64);
}
```


### BoundsEnforcer

```rust
/// Enforces edit boundary constraints on all edit operations.
/// Addresses: Requirement 13, all criteria
pub struct BoundsEnforcer {
    bounds: Option<EditBounds>,
}

impl BoundsEnforcer {
    pub fn new() -> Self; // No bounds active
    pub fn set_bounds(&mut self, bounds: EditBounds);
    pub fn clear_bounds(&mut self);
    pub fn bounds(&self) -> Option<&EditBounds>;
    pub fn is_active(&self) -> bool;

    /// Check if an edit at the given column is permitted.
    pub fn allows_edit_at(&self, column: u64) -> bool;

    /// Clip paste content to fit within bounds, truncating at right boundary.
    /// Addresses: Requirement 13, criterion 13.10
    pub fn clip_paste_content(&self, content: &str, start_col: u64) -> String;
}
```

### EditorTransaction

```rust
/// A single transaction unit for the undo system.
/// Contains before/after snapshots of affected lines.
/// Addresses: Requirement 11, criteria 11.1–11.3
#[derive(Debug, Clone)]
pub struct EditorTransaction {
    /// Lines affected by this transaction (0-based line numbers).
    pub affected_lines: Vec<u64>,
    /// Snapshot of line content before the edit.
    pub before_snapshot: Vec<LineSnapshot>,
    /// Snapshot of line content after the edit.
    pub after_snapshot: Vec<LineSnapshot>,
    /// Description for undo history display.
    pub description: String,
}

/// A snapshot of a single line's state.
#[derive(Debug, Clone)]
pub struct LineSnapshot {
    pub line_number: u64,
    pub content: String,
}
```

### ModifiedLineTracker

```rust
/// Tracks which lines have been modified since the last save.
/// Addresses: Requirement 11, criteria 11.6–11.8
pub struct ModifiedLineTracker {
    modified_lines: HashSet<u64>,
}

impl ModifiedLineTracker {
    pub fn new() -> Self;
    pub fn mark_modified(&mut self, line: u64);
    pub fn is_modified(&self, line: u64) -> bool;
    pub fn clear_all(&mut self);          // Called on save (Req 11.7)
    pub fn clear_line(&mut self, line: u64); // Called on undo to saved state (Req 11.8)
    pub fn modified_lines(&self) -> impl Iterator<Item = u64> + '_;
}
```


### MultiCaretCoordinator

```rust
/// Coordinates simultaneous edits across multiple carets.
/// Processes carets in reverse document order to avoid position drift.
/// Addresses: Requirement 8, criteria 8.4, 8.5, 8.13, 8.15, 8.16
pub struct MultiCaretCoordinator;

impl MultiCaretCoordinator {
    /// Execute an edit operation at all caret positions in the selection container.
    /// Wraps all sub-operations in a single UndoGroup.
    /// Skips protected ranges rather than failing. (Req 8.15)
    /// Realises virtual space before editing. (Req 8.16)
    pub fn execute_at_all_carets<F>(
        selection: &mut SelectionContainer,
        document: &mut DocumentHandle,
        bounds: &BoundsEnforcer,
        operation: F,
    ) -> Result<EditorTransaction, EditError>
    where
        F: Fn(&mut DocumentHandle, &SelectionPosition) -> Result<SingleEditResult, EditError>;
}

/// Result of a single edit at one caret position.
#[derive(Debug, Clone)]
pub struct SingleEditResult {
    /// The document modification descriptor for position adjustment.
    pub modification: DocumentModification,
    /// New caret position after the edit.
    pub new_caret: SelectionPosition,
}
```

### ClipboardContent

```rust
/// Represents clipboard content with metadata about its source.
/// Addresses: Requirement 10, criteria 10.5–10.10
#[derive(Debug, Clone)]
pub struct ClipboardContent {
    /// The text content.
    pub text: String,
    /// Whether this was a "line copy" (entire line with no selection).
    pub is_line_copy: bool,
    /// Whether this has rectangular selection metadata.
    pub is_rectangular: bool,
    /// Individual segments (for multi-caret or rectangular copies).
    pub segments: Vec<String>,
}
```

---

## Public API Surface

### InsertionEngine

```rust
/// Handles character insertion logic for both Insert and Overstrike modes.
/// Addresses: Requirements 1, 2, 3
pub struct InsertionEngine;

impl InsertionEngine {
    /// Insert a character at the caret position in Insert Mode.
    /// Handles virtual space realisation (Req 1.7) and BOUNDS checking.
    /// Addresses: Requirement 1, criteria 1.1–1.3, 1.5–1.7
    pub fn insert_char(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
        ch: char,
    ) -> Result<EditorTransaction, EditError>;

    /// Replace the character at the caret position in Overstrike Mode.
    /// Addresses: Requirement 3, criteria 3.1, 3.2, 3.5, 3.6
    pub fn overstrike_char(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
        ch: char,
    ) -> Result<EditorTransaction, EditError>;

    /// Handle newline insertion (Enter key) based on current mode.
    /// Insert Mode: splits line. Overstrike Mode: moves to next line.
    /// Addresses: Requirement 2, criteria 2.1–2.6
    pub fn handle_newline(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        mode: EditMode,
        bounds: &BoundsEnforcer,
    ) -> Result<EditorTransaction, EditError>;

    /// Insert a tab character or equivalent spaces based on settings.
    /// Addresses: Requirement 1, criterion 1.8
    pub fn insert_tab(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
        use_spaces: bool,
        tab_width: u32,
    ) -> Result<EditorTransaction, EditError>;

    /// Replace a selection with the given text (selection replacement).
    /// Addresses: Requirement 6, criterion 6.10; Requirement 2, criterion 2.4
    pub fn replace_selection(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
        text: &str,
    ) -> Result<EditorTransaction, EditError>;
}
```


### DeletionEngine

```rust
/// Handles text deletion at various granularities.
/// Addresses: Requirement 4, all criteria
pub struct DeletionEngine;

impl DeletionEngine {
    /// Delete the grapheme cluster before the caret (Backspace).
    /// Addresses: Requirement 4, criteria 4.1, 4.2, 4.12
    pub fn delete_back(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
    ) -> Result<EditorTransaction, EditError>;

    /// Delete the grapheme cluster at the caret (Delete key).
    /// Addresses: Requirement 4, criteria 4.3, 4.4
    pub fn delete_forward(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
    ) -> Result<EditorTransaction, EditError>;

    /// Delete the word before the caret (Ctrl+Backspace).
    /// Addresses: Requirement 4, criterion 4.5
    pub fn delete_word_left(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
    ) -> Result<EditorTransaction, EditError>;

    /// Delete the word after the caret (Ctrl+Delete).
    /// Addresses: Requirement 4, criterion 4.6
    pub fn delete_word_right(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
    ) -> Result<EditorTransaction, EditError>;

    /// Delete the entire current line (Ctrl+Shift+K).
    /// Addresses: Requirement 4, criterion 4.7
    pub fn delete_line(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
    ) -> Result<EditorTransaction, EditError>;

    /// Delete from caret to end of line (Ctrl+Shift+Delete).
    /// Addresses: Requirement 4, criterion 4.8
    pub fn delete_to_line_end(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
    ) -> Result<EditorTransaction, EditError>;

    /// Delete from start of line to caret (Ctrl+Shift+Backspace).
    /// Addresses: Requirement 4, criterion 4.9
    pub fn delete_to_line_start(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
    ) -> Result<EditorTransaction, EditError>;

    /// Delete the active selection content.
    /// Addresses: Requirement 4, criterion 4.10
    pub fn delete_selection(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
    ) -> Result<EditorTransaction, EditError>;
}
```

### LineManipulator

```rust
/// Line-level manipulation commands.
/// Addresses: Requirement 5, all criteria
pub struct LineManipulator;

impl LineManipulator {
    /// Swap the current line with the line above (Ctrl+T).
    /// Returns no-op if on line 1. (Req 5.1, 5.8)
    pub fn transpose_line(
        document: &mut DocumentHandle,
        selection: &SelectionContainer,
    ) -> Result<Option<EditorTransaction>, EditError>;

    /// Duplicate the current line (or selected lines) below. (Req 5.2)
    pub fn duplicate_line(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
    ) -> Result<EditorTransaction, EditError>;

    /// Convert selection (or current line) to uppercase. (Req 5.3)
    pub fn to_uppercase(
        document: &mut DocumentHandle,
        selection: &SelectionContainer,
    ) -> Result<EditorTransaction, EditError>;

    /// Convert selection (or current line) to lowercase. (Req 5.4)
    pub fn to_lowercase(
        document: &mut DocumentHandle,
        selection: &SelectionContainer,
    ) -> Result<EditorTransaction, EditError>;

    /// Toggle case of selection (or current line). (Req 5.5)
    pub fn toggle_case(
        document: &mut DocumentHandle,
        selection: &SelectionContainer,
    ) -> Result<EditorTransaction, EditError>;
}
```


### ClipboardSemantics

```rust
/// Edit-side clipboard operation logic (not system clipboard access).
/// Addresses: Requirement 10, all criteria
pub struct ClipboardSemantics;

impl ClipboardSemantics {
    /// Prepare content for clipboard from current selection(s).
    /// Handles stream, rectangular, multi-caret, and line-copy modes.
    /// Addresses: Requirement 10, criteria 10.1, 10.5, 10.7, 10.9
    pub fn prepare_copy(
        document: &DocumentHandle,
        selection: &SelectionContainer,
        selection_kind: SelectionKind,
    ) -> ClipboardContent;

    /// Perform cut: prepare copy content and delete the selection.
    /// Addresses: Requirement 10, criterion 10.2
    pub fn perform_cut(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        selection_kind: SelectionKind,
        bounds: &BoundsEnforcer,
    ) -> Result<(ClipboardContent, EditorTransaction), EditError>;

    /// Perform paste at the current caret/selection position(s).
    /// Handles line-copy, rectangular, and multi-caret distribution.
    /// Addresses: Requirement 10, criteria 10.3, 10.4, 10.6, 10.8, 10.10
    pub fn perform_paste(
        document: &mut DocumentHandle,
        selection: &mut SelectionContainer,
        bounds: &BoundsEnforcer,
        content: &ClipboardContent,
    ) -> Result<EditorTransaction, EditError>;
}
```

### TransactionRecorder

```rust
/// Bridge between edit operations and the undo-redo-transactions system.
/// Addresses: Requirement 11, all criteria; Requirement 8, criterion 8.13
pub struct TransactionRecorder;

impl TransactionRecorder {
    /// Record a single edit as an EditorTransaction on the TransactionStack.
    /// Addresses: Requirement 11, criteria 11.2, 11.3
    pub fn record(
        transaction_stack: &mut dyn TransactionStack,
        transaction: EditorTransaction,
    );

    /// Begin an UndoGroup for multi-caret operations.
    /// All subsequent record() calls until end_group() are grouped.
    /// Addresses: Requirement 11, criterion 11.9
    pub fn begin_group(transaction_stack: &mut dyn TransactionStack);

    /// End the current UndoGroup.
    pub fn end_group(transaction_stack: &mut dyn TransactionStack);
}
```

---

## Error Handling

```rust
/// Errors produced by the edit-operations crate.
/// Addresses: Cross-cutting Requirement 8 (error format: "[edit] operation: description")
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EditError {
    /// Attempted to edit in Browse mode (read-only).
    #[error("[edit] {operation}: document is in Browse mode (read-only)")]
    ReadOnly { operation: String },

    /// Edit position is outside the active BOUNDS range.
    /// Addresses: Requirement 13, criteria 13.2, 13.3, 13.5
    #[error("[edit] {operation}: column {column} is outside BOUNDS ({left}–{right})")]
    OutsideBounds {
        operation: String,
        column: u64,
        left: u64,
        right: u64,
    },

    /// Invalid BOUNDS values supplied.
    /// Addresses: Requirement 13, criterion 13.12
    #[error("[edit] bounds: invalid range ({left}, {right}) — left must be >= 1 and right > left")]
    InvalidBounds { left: u64, right: u64 },

    /// Cannot drop the last remaining selection range.
    /// Addresses: Requirement 14, criterion 14.2
    #[error("[edit] selection: cannot remove last remaining caret")]
    LastCaretRemoval,

    /// The document buffer reported an error during mutation.
    #[error("[edit] {operation}: document error — {description}")]
    DocumentError {
        operation: String,
        description: String,
    },

    /// Clipboard operation failed (system clipboard unavailable).
    /// Addresses: Requirement 10, criterion 10.12
    #[error("[edit] clipboard: {description}")]
    ClipboardError { description: String },

    /// Line transpose at document start (no-op, not an error to user).
    /// This is used internally to signal no action taken.
    /// Addresses: Requirement 5, criterion 5.8
    #[error("[edit] line_transpose: already at first line — no action taken")]
    NoOpAtBoundary { operation: String },
}
```


---

## Integration Points

### With `ff-document-model` (upstream — Wave 4)

- `ff-edit-operations` uses the `Document` / `DocumentHandle` API for all buffer mutations:
  - `insert(position, text)` — character and text insertion
  - `delete(position, length)` — character and range deletion
  - `char_at(position)` / `character_at(position)` — character inspection for overstrike
  - `line_start(line)` / `line_end(line)` — line boundary resolution
  - `line_count()` — validation of line numbers
  - `next_position(position, direction)` — grapheme-aware cursor movement
  - `split_view()` — read access for copy operations
- The document model provides the `DocumentHandle` (`Arc<RwLock<Document>>`) shared between edit-operations and other consumers
- Line index lookups (`line_from_position`, `line_start`, `line_end`) are used for line split/join operations
- Character navigation (`char_length_at`, `move_position_outside_char`) ensures edits respect grapheme cluster boundaries

### With `ff-undo-redo-transactions` (peer — Wave 4)

- `ff-edit-operations` defines `EditorTransaction` as the unit of undo work
- The `TransactionStack` trait (defined by `ff-undo-redo-transactions`) is used to push/pop transactions
- `UndoGroup` wrapping is used for multi-caret operations (all sub-edits become one undo step)
- Save-point marking integrates with the save command to track modified state
- `ff-edit-operations` does NOT own the `TransactionStack` — it receives a reference through the command context

### With `ff-command` (upstream — Wave 2)

- All edit operations are registered as named commands in the `CommandRegistry`:
  - `edit.insert_char`, `edit.delete_back`, `edit.delete_forward`
  - `edit.delete_word_left`, `edit.delete_word_right`
  - `edit.delete_line`, `edit.delete_to_line_end`, `edit.delete_to_line_start`
  - `edit.newline`, `edit.toggle_mode`
  - `edit.line_transpose`, `edit.line_duplicate`
  - `edit.uppercase`, `edit.lowercase`, `edit.toggle_case`
  - `edit.select_all`, `edit.select_next_occurrence`
  - `edit.cut`, `edit.copy`, `edit.paste`
  - `edit.add_caret_above`, `edit.add_caret_below`, `edit.clear_extra_carets`
  - `edit.bounds`
- Each command handler implements `CommandHandler` trait from `ff-command`
- Each undoable command returns `CommandResult::OkUndoable` with an `UndoRecord`
- The `ExecutionContext` provides current cursor position and active document URI
- Commands use `CommandParams` for parameters (e.g., `edit.bounds` receives `left` and `right` params)

### With `ff-plugin` (via command framework)

- Plugins can invoke edit operations through `execute_command("edit.*", params)`
- Plugins cannot bypass the command framework to directly call edit engines
- The `edit.bounds` command is available for plugins managing fixed-format file editing

### With `clipboard-operations` (downstream)

- `ClipboardSemantics` prepares content; `clipboard-operations` handles system clipboard access
- The edit crate produces `ClipboardContent` structs; the clipboard crate serializes to/from system clipboard
- Rectangular metadata tagging enables round-trip rectangular paste

### With `caret-and-selection` (downstream — Wave 6)

- The GUI rendering layer reads `SelectionContainer` state to draw selection highlights
- Modified line markers from `ModifiedLineTracker` are rendered by the caret-and-selection system
- This crate is the authoritative source of selection state; the rendering crate is read-only

### With `navigation-commands` (downstream — Wave 5)

- Navigation commands (arrow keys, Home, End, word movement) update `SelectionContainer` positions
- When Shift is held, navigation extends selection rather than collapsing it (Req 6.4–6.8)
- The `BOUNDS` primary command parsing is handled by `navigation-commands`; this crate enforces the constraint

### With `encoding-and-characters` (peer)

- Grapheme cluster boundary detection for deletion operations
- Word character classification for word-delete operations (Ctrl+Backspace, Ctrl+Delete)

### Dependency Direction

```
ff-logging ← ff-command ← ff-edit-operations ← ff-navigation-commands
                        ← ff-document-model      ← ff-caret-and-selection
                        ← ff-undo-redo-transactions
```

`ff-edit-operations` depends on: `ff-document-model`, `ff-command`, `ff-logging`.
It exposes types consumed by: `ff-caret-and-selection`, `ff-navigation-commands`, `ff-clipboard-operations`.


---

## Configuration

All configuration consumed by `ff-edit-operations` is provided through `ff-core` at initialization time. The crate does not directly read configuration files.

### Relevant Configuration Keys

```toml
[editor]
# Default editing mode for new documents.
# Values: "insert", "overstrike"
# Default: "insert"
# Addresses: Requirement 1, criterion 1.4
default_mode = "insert"

# Whether to use spaces or tabs for Tab key insertion.
# Default: true (spaces)
# Addresses: Requirement 1, criterion 1.8
use_spaces_for_tabs = true

# Number of spaces per tab stop.
# Range: 1–16. Default: 4
# Addresses: Requirement 1, criterion 1.8
tab_width = 4

# Line ending style for new lines created by Enter.
# Values: "lf", "crlf", "cr"
# Default: platform-dependent ("crlf" on Windows, "lf" on Unix)
# Addresses: Requirement 2, criterion 2.6
line_ending = "crlf"

# Virtual space mode for caret positioning beyond line end.
# Values: "none", "rectangular_only", "always"
# Default: "rectangular_only"
virtual_space = "rectangular_only"
```

---

## Testing Strategy

### Thread-Safety Approach

| Component | Mechanism | Rationale |
|-----------|-----------|-----------|
| `SelectionContainer` | Owned by editor instance (single-threaded access) | Selections are per-view; only the owning editor mutates them |
| `EditModeManager` | Owned by editor instance | Mode is per-editor-instance state |
| `BoundsEnforcer` | Owned by editor instance | Bounds are per-document |
| `ModifiedLineTracker` | `Mutex<HashSet<u64>>` | May be read by rendering thread for marker display |
| `DocumentHandle` | `Arc<RwLock<Document>>` (from `ff-document-model`) | Shared between editor, background tasks, and watchers |
| `TransactionStack` | Accessed via trait object with `Send + Sync` | Undo system manages its own synchronization |

### Edit Execution Flow

1. Command dispatch invokes edit command handler on the **main thread**
2. Handler acquires `DocumentHandle` write lock for the minimum duration
3. Handler mutates document, updates selection, records transaction
4. Handler releases document lock
5. Document watchers are notified (non-blocking; they defer expensive work)
6. GUI rendering thread reads selection/markers on next frame via read lock

### Multi-Caret Lock Strategy

For multi-caret operations, the document write lock is held for the entire group of sub-edits (not released between carets). This ensures:
- No interleaving from other threads between caret edits
- Position adjustment within the group is deterministic
- The UndoGroup is atomic from the document's perspective


---

## Correctness Properties

These properties are suitable for property-based testing with `proptest`. They validate invariants that must hold across all valid inputs.

### Property 1: Insert Mode Preserves Document Length Invariant

**Statement**: For any document of length L and any single character insertion in Insert Mode at a valid position, the resulting document length is L + (byte length of the character). The line count increases by 0 (non-newline) or 1 (newline character).

**Validates: Requirements 1.1, 1.2**

```rust
// proptest strategy: generate document content, valid caret position, printable char
// assertion: new_length == old_length + char.len_utf8()
//            new_line_count == old_line_count + (1 if char is newline else 0)
```

### Property 2: Overstrike Mode Preserves Line Length (Non-EOL)

**Statement**: For any document line of length L and any single character overstrike at a position within the line (not at/past EOL), the line length remains L. The character at the overstriked position equals the new character.

**Validates: Requirements 3.1, 3.2**

```rust
// proptest strategy: generate line content, position within line, printable char
// assertion: line_length unchanged for mid-line overstrike
//            char_at(position) == new_char
```

### Property 3: Selection Position Adjustment Monotonicity

**Statement**: For any set of SelectionPositions sorted by document order, after applying `MovePositions` for any single insertion or deletion, the positions remain in non-decreasing order. No two originally-distinct positions that are not within the edited range become inverted.

**Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

```rust
// proptest strategy: generate sorted Vec<SelectionPosition>, single DocumentModification
// assertion: after move_positions, positions remain sorted
```

### Property 4: Multi-Caret Reverse-Order Independence

**Statement**: For any document with N carets (N >= 2) at distinct positions, performing the same single-character insert at all carets in reverse document order produces a document where each insertion position contains the expected character and all carets are correctly spaced (each shifted by the cumulative length of insertions after it in document order).

**Validates: Requirements 8.4, 8.5**

```rust
// proptest strategy: generate document, N distinct caret positions, single char
// assertion: after multi-caret insert, char appears at each original position
//            caret[i].column == original[i].column + (N - 1 - i) * char_len
```

### Property 5: Selection Container Trim Idempotence

**Statement**: For any SelectionContainer state, calling `trim()` twice produces the same result as calling it once. After trim, no two ranges overlap.

**Validates: Requirements 14.3, 8.8**

```rust
// proptest strategy: generate Vec<SelectionRange> with possible overlaps
// assertion: trim(); trim() == trim() (idempotent)
//            for all pairs (a, b) in result: !a.overlaps(b)
```

### Property 6: BOUNDS Enforcement Completeness

**Statement**: For any EditBounds(left, right) and any edit operation at column C, the operation succeeds if and only if left <= C <= right. No edit is permitted outside the bounds; all edits within bounds proceed normally.

**Validates: Requirements 13.2, 13.3, 13.5**

```rust
// proptest strategy: generate valid EditBounds, column positions inside and outside
// assertion: allows_edit_at(c) == (left <= c && c <= right)
```

### Property 7: Edit Mode Toggle Involution

**Statement**: For any starting mode in {Insert, Overstrike}, toggling the mode twice returns to the original mode. Browse mode is unaffected by toggle (toggle only switches between Insert and Overstrike).

**Validates: Requirements 3.3**

```rust
// proptest strategy: generate starting EditMode
// assertion: toggle(toggle(mode)) == mode for Insert/Overstrike
//            toggle(Browse) is a no-op
```

### Property 8: Delete-Back at Position 0 Joins with Previous Line

**Statement**: For any document with at least 2 lines, performing delete_back when the caret is at column 0 of line N (N > 0) produces a document with one fewer line, where the content of line N is appended to the end of line N-1, and the caret is at the junction column.

**Validates: Requirements 4.1, 4.2**

```rust
// proptest strategy: generate multi-line document, caret at col 0 of line > 0
// assertion: line_count decreases by 1
//            new_line_content == old_line[n-1] + old_line[n]
//            caret.column == old_line[n-1].length
```

### Property 9: Selection Replacement Atomicity

**Statement**: For any document with an active selection covering text T, replacing the selection with new text R produces a document where: the selected region is removed, R appears at the selection start, the caret is positioned at the end of R, and the operation is a single EditorTransaction.

**Validates: Requirements 6.10, 2.4**

```rust
// proptest strategy: generate document, valid selection range, replacement text
// assertion: content_at(selection_start, R.len()) == R
//            document_length == old_length - selection_length + R.len()
//            caret == selection_start + R.len()
```

### Property 10: Modified Line Marker Save-Point Consistency

**Statement**: For any sequence of edits followed by a save operation, all modified line markers are cleared after save. For any sequence of edits followed by undo operations that restore a line to its saved state, the modified marker for that line is cleared.

**Validates: Requirements 11.6, 11.7, 11.8**

```rust
// proptest strategy: generate edit sequence, save, then undo sequence
// assertion: after save → no modified markers
//            after undo that restores line to saved content → that line's marker cleared
```

### Property 11: Multi-Caret Merge on Collision

**Statement**: For any set of N carets, if an operation causes two or more carets to occupy the same position, the `trim()` operation reduces them to a single caret at that position. The total caret count decreases by the number of collisions.

**Validates: Requirements 8.8, 14.3**

```rust
// proptest strategy: generate carets that will collide after an operation (e.g., all on same line, delete to line start)
// assertion: after operation + trim, count decreases, no duplicates
```

### Property 12: Rectangular Selection Produces One Segment Per Line

**Statement**: For any rectangular selection defined by (top_line, left_col, bottom_line, right_col), the selection produces exactly (bottom_line - top_line + 1) segments, each spanning columns [left_col, right_col] on its respective line (clamped to line length or extended with virtual space).

**Validates: Requirements 9.1, 9.2, 9.8**

```rust
// proptest strategy: generate document, valid rectangle coordinates
// assertion: segment_count == bottom_line - top_line + 1
//            each segment spans exactly [left_col, right_col] (with virtual space for short lines)
```


---

## Appendix A: External Crate Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `thiserror` | 2.0 | Error type derivation |
| `unicode-segmentation` | 1.0 | Grapheme cluster boundary detection |
| `proptest` | 1.0 | Property-based testing (dev-dependency only) |
| `pretty_assertions` | 1.0 | Enhanced test assertion output (dev-dependency only) |

Note: `ff-edit-operations` has minimal external dependencies. Most functionality comes from upstream workspace crates (`ff-document-model`, `ff-command`, `ff-logging`).

---

## Appendix B: Command Registration Table

All commands registered by `ff-edit-operations` during crate initialization:

| Command_ID | Display Name | Undoable | Default Shortcut | Description |
|-----------|-------------|----------|-----------------|-------------|
| `edit.insert_char` | Insert Character | Yes | (typed character) | Insert character at caret |
| `edit.delete_back` | Delete Back | Yes | Backspace | Delete character before caret |
| `edit.delete_forward` | Delete Forward | Yes | Delete | Delete character at caret |
| `edit.delete_word_left` | Delete Word Left | Yes | Ctrl+Backspace | Delete word before caret |
| `edit.delete_word_right` | Delete Word Right | Yes | Ctrl+Delete | Delete word after caret |
| `edit.delete_line` | Delete Line | Yes | Ctrl+Shift+K | Delete entire current line |
| `edit.delete_to_line_end` | Delete to Line End | Yes | Ctrl+Shift+Delete | Delete to end of line |
| `edit.delete_to_line_start` | Delete to Line Start | Yes | Ctrl+Shift+Backspace | Delete to start of line |
| `edit.newline` | New Line | Yes | Enter | Insert newline / move to next line |
| `edit.toggle_mode` | Toggle Insert/Overstrike | No | Insert | Toggle between Insert and Overstrike |
| `edit.line_transpose` | Line Transpose | Yes | Ctrl+T | Swap current line with line above |
| `edit.line_duplicate` | Line Duplicate | Yes | Ctrl+Shift+D | Duplicate current line below |
| `edit.uppercase` | Uppercase | Yes | Ctrl+Shift+U | Convert selection to uppercase |
| `edit.lowercase` | Lowercase | Yes | Ctrl+U | Convert selection to lowercase |
| `edit.toggle_case` | Toggle Case | Yes | — | Toggle case of selection |
| `edit.select_all` | Select All | No | Ctrl+A | Select all document content |
| `edit.select_next_occurrence` | Select Next Occurrence | No | Ctrl+D | Add caret at next occurrence |
| `edit.add_caret_above` | Add Caret Above | No | Ctrl+Alt+Up | Add caret one line above |
| `edit.add_caret_below` | Add Caret Below | No | Ctrl+Alt+Down | Add caret one line below |
| `edit.clear_extra_carets` | Clear Extra Carets | No | Escape | Reduce to single caret |
| `edit.bounds` | Set Bounds | No | — | Set/clear edit boundaries |
| `edit.cut` | Cut | Yes | Ctrl+X | Cut selection to clipboard |
| `edit.copy` | Copy | No | Ctrl+C | Copy selection to clipboard |
| `edit.paste` | Paste | Yes | Ctrl+V | Paste from clipboard |
| `edit.tab` | Insert Tab | Yes | Tab | Insert tab/spaces |

---

## Appendix C: Edit Mode State Diagram

```
┌─────────────┐   Insert Key   ┌──────────────────┐
│             │ ──────────────► │                  │
│ Insert Mode │                 │ Overstrike Mode  │
│             │ ◄────────────── │                  │
└─────────────┘   Insert Key   └──────────────────┘
       │                                │
       │  set_mode(Browse)              │  set_mode(Browse)
       ▼                                ▼
┌──────────────────────────────────────────────────┐
│                  Browse Mode                      │
│        (read-only, no edits permitted)            │
│   Exit: set_mode(Insert) or set_mode(Overstrike) │
└──────────────────────────────────────────────────┘
```

- Insert ↔ Overstrike: toggled by Insert key press
- Browse is entered/exited programmatically (e.g., by read-only file, ISPF VIEW command)
- The Insert key toggle does NOT cycle through Browse — Browse is a distinct state

---

## Appendix D: Multi-Caret Processing Order

When multiple carets exist and an edit operation is dispatched:

1. Acquire document write lock
2. Begin UndoGroup on TransactionStack
3. Sort carets by document position (descending — last to first)
4. For each caret (reverse document order):
   a. Check if position is in a protected range → skip if protected (Req 8.15)
   b. If caret is in virtual space → realise virtual space (pad with spaces) (Req 8.16)
   c. Check BOUNDS enforcement → skip if outside bounds
   d. Perform the edit operation at the caret position
   e. Record sub-transaction
   f. Adjust all remaining caret positions via `MovePositions`
5. End UndoGroup
6. Call `trim()` on SelectionContainer to merge any collided carets (Req 8.8)
7. Release document write lock
8. Notify document watchers

Processing in reverse order ensures that edits at later positions do not invalidate the byte offsets of earlier positions (which haven't been processed yet).
