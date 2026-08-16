# Design Document: Line Commands (`ff-line-commands`)

## Overview

The `ff-line-commands` crate implements the **ISPF line command engine** for FileForgeWorkbench. It provides prefix-area command parsing, block pairing, pending state management, and execution logic for all line commands: delete, insert, repeat, copy, move, after/before targets, exclude, tag/untag, shift, and bounds-aware shift.

### Purpose

- Parse line command strings from the prefix area into typed command structures
- Validate block command pairs and normalize line ranges
- Manage pending command state within DocumentSession
- Execute immediate commands (D, I, R, X, T, U, >, <, ), () without requiring a primary command
- Resolve source/target marker pairs (C/CC + A/B, M/MM + A/B) on target entry
- Integrate with the command framework for undo/redo on undoable operations
- Enforce command compatibility rules between line commands and primary commands
- Dispatch all operations through the workbench command framework (Req 14.8)

### Position in Architecture

```
Wave 5 — Command Engine

┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│     Prefix area rendering, user input collection             │
├─────────────────────────────────────────────────────────────┤
│  ff-command-semantics (peer — primary command pipeline)       │
│  ff-exclude-show-filter (downstream — SHOW/RESET)            │
├─────────────────────────────────────────────────────────────┤
│         ff-line-commands (THIS CRATE — Wave 5)               │
│   Line command parser, block pairing, pending state,         │
│   execution engine                                           │
├─────────────────────────────────────────────────────────────┤
│  ff-edit-operations (Wave 4) — edit primitives               │
│  ff-document-model (Wave 4) — buffer access                  │
│  ff-display-line-mapping (Wave 4) — visibility state         │
│  ff-command (Wave 2) — command dispatch, undo integration    │
│  ff-undo-redo-transactions (Wave 4) — transaction wrapping   │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Zero GUI dependencies — prefix-area rendering is UI shell responsibility
- **Command-Driven (Req 4)**: All line command executions dispatch through `ff-command`
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-line-commands`
- **Error Message Standards (Req 8)**: All errors follow `[line-cmd] operation: description` format
- **Configuration (Req 5)**: ShiftWidth default configurable via configuration system

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Input (Prefix Area)"
        PA[Prefix Area Input<br/>raw string per line]
    end

    subgraph "ff-line-commands"
        P[LineCommandParser<br/>string → ParsedLineCommand]
        V[BlockPairValidator<br/>pair matching, normalization]
        PS[PendingCommandStore<br/>per-session state]
        CM[CompatibilityMatrix<br/>line cmd ↔ primary cmd rules]
        RE[ResolutionEngine<br/>immediate, source+target, block]
        EX[ExecutionEngine<br/>delete, insert, repeat, shift, etc.]
    end

    subgraph "Downstream / Upstream"
        DOC[ff-document-model<br/>Document, TextBuffer]
        EDIT[ff-edit-operations<br/>EditorTransaction]
        CMD[ff-command<br/>CommandDispatch]
        DLM[ff-display-line-mapping<br/>visibility state]
        UNDO[ff-undo-redo-transactions<br/>TransactionStack]
        CFG[ff-configuration-system<br/>ShiftWidth]
    end

    PA --> P
    P --> V
    P --> PS
    V --> RE
    PS --> RE
    CM --> RE
    RE --> EX

    EX -->|delete/insert/shift lines| DOC
    EX -->|transaction recording| EDIT
    EX -->|command dispatch| CMD
    EX -->|exclude flag| DLM
    EX -->|undo push| UNDO
    EX -->|read shift_width| CFG
end
```

### Layer Placement

| Layer | Role |
|-------|------|
| **Parsing Layer** | `LineCommandParser` converts raw prefix-area strings into typed `ParsedLineCommand` |
| **Validation Layer** | `BlockPairValidator` checks pairs, normalizes ranges, detects conflicts |
| **State Layer** | `PendingCommandStore` holds unresolved commands, supports querying by type |
| **Compatibility Layer** | `CompatibilityMatrix` defines allowed primary + line command combinations |
| **Resolution Layer** | `ResolutionEngine` determines which commands can execute this cycle |
| **Execution Layer** | `ExecutionEngine` performs the document mutations and records transactions |

---

## Components and Interfaces

```
crates/ff-line-commands/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── parser.rs               # LineCommandParser — string → ParsedLineCommand
│   ├── command.rs              # ParsedLineCommand, LineCommandKind, BlockCommandKind enums
│   ├── pending.rs              # PendingCommandStore — per-session pending state
│   ├── block_pair.rs           # BlockPairValidator — pair matching, normalization
│   ├── compatibility.rs        # CompatibilityMatrix — primary ↔ line cmd rules
│   ├── resolution.rs           # ResolutionEngine — determines executable commands
│   ├── execution/
│   │   ├── mod.rs              # ExecutionEngine dispatcher
│   │   ├── delete.rs           # Delete line command execution (D, Dn, DD)
│   │   ├── insert.rs           # Insert line command execution (I, In)
│   │   ├── repeat.rs           # Repeat line command execution (R, Rn, RR)
│   │   ├── copy.rs             # Copy resolution and execution (C, CC + A/B)
│   │   ├── move_cmd.rs         # Move resolution and execution (M, MM + A/B)
│   │   ├── exclude.rs          # Exclude line command execution (X, Xn, XX)
│   │   ├── tag.rs              # Tag/Untag execution (T, TT, U, UU)
│   │   ├── shift_right.rs      # Shift right execution (>, >n, >>)
│   │   ├── shift_left.rs       # Shift left execution (<, <n, <<)
│   │   └── bounds_shift.rs     # Bounds-aware shift execution (), )), (, ((
│   ├── commands/
│   │   ├── mod.rs              # Command framework registration
│   │   └── handlers.rs         # CommandHandler impls for line-command operations
│   ├── config.rs               # Configuration keys (shift_width, etc.)
│   └── error.rs                # LineCommandError enum
└── tests/
    ├── parser_tests.rs         # Parser unit + property tests
    ├── block_pair_tests.rs     # Block pairing property tests
    ├── pending_tests.rs        # Pending state management property tests
    ├── compatibility_tests.rs  # Compatibility matrix tests
    ├── resolution_tests.rs     # Resolution engine property tests
    ├── delete_tests.rs         # Delete execution tests
    ├── insert_tests.rs         # Insert execution tests
    ├── repeat_tests.rs         # Repeat execution tests
    ├── copy_move_tests.rs      # Copy/Move resolution tests
    ├── exclude_tests.rs        # Exclude execution tests
    ├── tag_tests.rs            # Tag/Untag execution tests
    ├── shift_tests.rs          # Shift left/right property tests
    ├── bounds_shift_tests.rs   # Bounds-aware shift property tests
    └── integration.rs          # End-to-end line command scenarios
```

---

## Data Models

### ParsedLineCommand

```rust
/// A line command parsed from a prefix-area input string.
/// Addresses: Requirements 1–11
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLineCommand {
    /// The line number where this command was entered (0-based document line).
    pub line: u64,
    /// The kind of line command parsed.
    pub kind: LineCommandKind,
}
```

### LineCommandKind

```rust
/// All possible line command types with their parameters.
/// Addresses: Requirements 1–11
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LineCommandKind {
    // --- Delete (Requirement 1) ---
    /// Delete a single line.
    Delete,
    /// Delete n consecutive lines starting at this line.
    DeleteCount(u32),
    /// Block delete marker (DD). Requires matching pair.
    DeleteBlock,

    // --- Insert (Requirement 2) ---
    /// Insert one blank line after this line.
    Insert,
    /// Insert n blank lines after this line.
    InsertCount(u32),

    // --- Repeat (Requirement 3) ---
    /// Duplicate this line once.
    Repeat,
    /// Duplicate this line n times.
    RepeatCount(u32),
    /// Block repeat marker (RR). Requires matching pair.
    RepeatBlock,

    // --- Copy (Requirement 4) ---
    /// Single-line copy source marker.
    Copy,
    /// Block copy source marker (CC). Requires matching pair.
    CopyBlock,

    // --- Move (Requirement 5) ---
    /// Single-line move source marker.
    Move,
    /// Block move source marker (MM). Requires matching pair.
    MoveBlock,

    // --- Target (Requirement 6) ---
    /// After-insertion target.
    After,
    /// Before-insertion target.
    Before,

    // --- Exclude (Requirement 7) ---
    /// Exclude a single line from the viewport.
    Exclude,
    /// Exclude n consecutive lines.
    ExcludeCount(u32),
    /// Block exclude marker (XX). Requires matching pair.
    ExcludeBlock,

    // --- Tag/Untag (Requirement 8) ---
    /// Tag a single line.
    Tag,
    /// Block tag marker (TT). Requires matching pair.
    TagBlock,
    /// Untag a single line.
    Untag,
    /// Block untag marker (UU). Requires matching pair.
    UntagBlock,

    // --- Shift Right (Requirement 9) ---
    /// Shift right by default ShiftWidth.
    ShiftRight,
    /// Shift right by n columns.
    ShiftRightCount(u32),
    /// Block shift right marker (>>). Requires matching pair.
    ShiftRightBlock,

    // --- Shift Left (Requirement 10) ---
    /// Shift left by default ShiftWidth.
    ShiftLeft,
    /// Shift left by n columns.
    ShiftLeftCount(u32),
    /// Block shift left marker (<<). Requires matching pair.
    ShiftLeftBlock,

    // --- Bounds-Aware Shift (Requirement 11) ---
    /// Bounds-aware shift right by one position.
    BoundsShiftRight,
    /// Block bounds-aware shift right marker ()). Requires matching pair.
    BoundsShiftRightBlock,
    /// Bounds-aware shift left by one position.
    BoundsShiftLeft,
    /// Block bounds-aware shift left marker (((). Requires matching pair.
    BoundsShiftLeftBlock,
}
```

### LineCommandCategory

```rust
/// Classification of line commands for resolution and compatibility logic.
/// Addresses: Requirements 12, 13, 14
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineCommandCategory {
    /// Commands that execute immediately without a partner or target.
    /// D, Dn, I, In, R, Rn, X, Xn, T, U, >, >n, <, <n, ), (
    Immediate,
    /// Block markers that require exactly one matching pair to execute.
    /// DD, RR, XX, TT, UU, >>, <<, )), ((
    Block,
    /// Source markers that require a target (A or B) to resolve.
    /// C, CC, M, MM
    Source,
    /// Target markers that resolve pending source markers.
    /// A, B
    Target,
}

### PendingCommand

```rust
/// A line command that has been entered but not yet resolved.
/// Stored in PendingCommandStore until resolved, cleared, or corrected.
/// Addresses: Requirement 14
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCommand {
    /// The parsed command.
    pub command: ParsedLineCommand,
    /// A message describing why this command is pending.
    pub reason: PendingReason,
    /// Timestamp when the command was entered (monotonic, for ordering).
    pub entered_at: u64,
}

/// Reason a command is pending (for display in prefix area and status).
/// Addresses: Requirement 14, criteria 3–6
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingReason {
    /// Block marker waiting for its matching pair.
    AwaitingPair,
    /// Source marker (C/CC/M/MM) waiting for a target (A/B).
    AwaitingTarget,
    /// Target marker (A/B) waiting for a source (C/CC/M/MM).
    AwaitingSource,
    /// Invalid command text retained for user correction.
    InvalidCommand(String),
}
```

### PendingCommandStore

```rust
/// Per-session storage for unresolved line commands.
/// Provides query, insertion, removal, and reset operations.
/// Addresses: Requirement 14, all criteria
pub struct PendingCommandStore {
    /// All pending commands, indexed by line number for O(1) lookup.
    commands: HashMap<u64, PendingCommand>,
    /// Monotonically increasing counter for ordering.
    next_id: u64,
}

impl PendingCommandStore {
    pub fn new() -> Self;

    /// Add a pending command for a line.
    /// Addresses: Requirement 14.1
    pub fn add(&mut self, command: ParsedLineCommand, reason: PendingReason);

    /// Remove a pending command at a line (on successful resolution).
    /// Addresses: Requirement 14.2
    pub fn remove(&mut self, line: u64) -> Option<PendingCommand>;

    /// Get the pending command for a specific line, if any.
    pub fn get(&self, line: u64) -> Option<&PendingCommand>;

    /// Query all pending commands of a given category.
    /// Addresses: Requirement 14.7
    pub fn by_category(&self, category: LineCommandCategory) -> Vec<&PendingCommand>;

    /// Query all pending source markers (C, CC, M, MM).
    /// Addresses: Requirement 14.7
    pub fn pending_sources(&self) -> Vec<&PendingCommand>;

    /// Query all pending target markers (A, B).
    /// Addresses: Requirement 14.7
    pub fn pending_targets(&self) -> Vec<&PendingCommand>;

    /// Query all pending block markers of a specific kind.
    pub fn pending_blocks(&self, kind: &LineCommandKind) -> Vec<&PendingCommand>;

    /// Clear all pending commands (RESET COMMANDS / RESET ALL).
    /// Addresses: Requirement 14.5
    pub fn clear_all(&mut self);

    /// Returns all pending commands as an iterator (for prefix area display).
    /// Addresses: Requirement 14.4
    pub fn all_pending(&self) -> impl Iterator<Item = (&u64, &PendingCommand)>;

    /// Returns the number of pending commands.
    pub fn count(&self) -> usize;

    /// Returns true if there are no pending commands.
    pub fn is_empty(&self) -> bool;
}
```

### BlockPair

```rust
/// A validated and normalized block command pair.
/// Addresses: Requirement 12
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPair {
    /// The block command kind (DD, RR, XX, TT, UU, >>, <<, )), (().
    pub kind: BlockCommandKind,
    /// The start line of the block (inclusive, normalized to min).
    /// Addresses: Requirement 12.2
    pub start_line: u64,
    /// The end line of the block (inclusive, normalized to max).
    /// Addresses: Requirement 12.2
    pub end_line: u64,
}

/// Block command kinds (the paired variants).
/// Addresses: Requirement 12.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockCommandKind {
    Delete,      // DD
    Repeat,      // RR
    Exclude,     // XX
    Tag,         // TT
    Untag,       // UU
    ShiftRight,  // >>
    ShiftLeft,   // <<
    BoundsRight, // ))
    BoundsLeft,  // ((
    Copy,        // CC
    Move,        // MM
}
```

### SourceTarget

```rust
/// A resolved source + target combination for copy/move operations.
/// Addresses: Requirements 4, 5, 6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTarget {
    /// The operation type.
    pub operation: SourceOperation,
    /// Source lines (single line or block range, inclusive).
    pub source_start: u64,
    pub source_end: u64,
    /// Target insertion point.
    pub target_line: u64,
    /// Whether to insert after (A) or before (B) the target line.
    pub target_position: TargetPosition,
}

/// Whether the source operation is copy or move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceOperation {
    Copy,
    Move,
}

/// Insertion position relative to the target line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPosition {
    After,
    Before,
}
```

### LineCommandConfig

```rust
/// Configuration values for the line commands subsystem.
/// Read from the configuration system at startup and on hot-reload.
/// Addresses: Requirements 9.7, 10.7
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCommandConfig {
    /// Default shift width for > and < commands (default: 2).
    pub shift_width: u32,
}

impl Default for LineCommandConfig {
    fn default() -> Self {
        Self { shift_width: 2 }
    }
}
```

### CommandCompatibilityMatrix

```rust
/// Defines which primary commands are compatible with which line commands.
/// Addresses: Requirement 13
pub struct CommandCompatibilityMatrix;

impl CommandCompatibilityMatrix {
    /// Check if a primary command is compatible with the set of pending line commands.
    /// Returns Ok if compatible, Err with description if not.
    /// Addresses: Requirement 13.1, 13.2
    pub fn check_compatibility(
        primary_command: Option<&str>,
        pending: &PendingCommandStore,
    ) -> Result<(), LineCommandError>;

    /// Returns true if the given pending commands are all immediate
    /// and can execute without a primary command.
    /// Addresses: Requirement 13.4
    pub fn all_immediate(pending: &PendingCommandStore) -> bool;
}
```

---

## Public API Surface

### LineCommandParser

```rust
/// Parses raw prefix-area input strings into typed line commands.
/// Case-insensitive. Supports optional numeric counts.
/// Addresses: Requirements 1–11 (parsing aspect)
pub struct LineCommandParser;

impl LineCommandParser {
    /// Parse a single prefix-area input string for a given line.
    /// Returns Ok(ParsedLineCommand) on success.
    /// Returns Err(LineCommandError::InvalidCommand) if unrecognised.
    /// Addresses: Requirement 14.6
    ///
    /// # Recognised Patterns
    /// - `D`, `D<n>`, `DD` — Delete
    /// - `I`, `I<n>` — Insert
    /// - `R`, `R<n>`, `RR` — Repeat
    /// - `C`, `CC` — Copy source
    /// - `M`, `MM` — Move source
    /// - `A` — After target
    /// - `B` — Before target
    /// - `X`, `X<n>`, `XX` — Exclude
    /// - `T`, `TT` — Tag
    /// - `U`, `UU` — Untag
    /// - `>`, `><n>`, `>>` — Shift right
    /// - `<`, `<<n>`, `<<` — Shift left
    /// - `)`, `))` — Bounds-aware shift right
    /// - `(`, `((` — Bounds-aware shift left
    pub fn parse(input: &str, line: u64) -> Result<ParsedLineCommand, LineCommandError>;

    /// Classify a parsed command into its category (Immediate, Block, Source, Target).
    pub fn classify(kind: &LineCommandKind) -> LineCommandCategory;

    /// Returns true if the given kind is a block marker requiring a pair.
    pub fn is_block_marker(kind: &LineCommandKind) -> bool;
}
```

### BlockPairValidator

```rust
/// Validates and normalizes block command pairs from the pending store.
/// Addresses: Requirement 12, all criteria
pub struct BlockPairValidator;

impl BlockPairValidator {
    /// Attempt to form a valid block pair from pending block markers.
    /// Returns Ok(BlockPair) if exactly two matching markers exist.
    /// Returns Err if 0, 1, or >2 markers exist for a block type.
    /// Addresses: Requirement 12.2, 12.3, 12.4
    pub fn validate_pair(
        pending: &PendingCommandStore,
        block_kind: BlockCommandKind,
    ) -> Result<BlockPair, LineCommandError>;

    /// Check for overlapping block ranges across different block types.
    /// Returns Err if overlapping ranges are found.
    /// Addresses: Requirement 12.5
    pub fn check_overlaps(pairs: &[BlockPair]) -> Result<(), LineCommandError>;

    /// Normalize a pair so start_line <= end_line regardless of entry order.
    /// Addresses: Requirement 12.2
    pub fn normalize(line1: u64, line2: u64) -> (u64, u64);
}
```

### ResolutionEngine

```rust
/// Determines which pending line commands can be executed in the current
/// command cycle, resolves source/target pairs, and dispatches execution.
/// Addresses: Requirements 6, 13, 14
pub struct ResolutionEngine;

impl ResolutionEngine {
    /// Process all newly entered line commands and existing pending commands.
    /// Resolves immediate commands, block pairs, and source+target combinations.
    /// Returns a list of executable operations or errors.
    ///
    /// # Resolution Order
    /// 1. Parse new prefix-area inputs → add to pending store or reject
    /// 2. Validate block pairs → form BlockPair if two markers present
    /// 3. Check source+target resolution → form SourceTarget if both present
    /// 4. Verify compatibility with primary command (if any)
    /// 5. Return resolved operations for execution
    ///
    /// Addresses: Requirements 12–14
    pub fn resolve(
        new_inputs: &[(u64, String)],
        pending: &mut PendingCommandStore,
        primary_command: Option<&str>,
    ) -> ResolutionResult;
}

/// The outcome of resolution: either executable operations or errors.
/// Addresses: Requirements 12–14
#[derive(Debug)]
pub struct ResolutionResult {
    /// Commands ready to execute (immediate, paired blocks, resolved source+target).
    pub executable: Vec<ExecutableCommand>,
    /// Errors encountered during resolution (display to user).
    pub errors: Vec<LineCommandError>,
    /// Commands that remain pending after this cycle.
    pub still_pending: Vec<PendingCommand>,
}

/// A command that has been fully resolved and is ready for execution.
#[derive(Debug, Clone)]
pub enum ExecutableCommand {
    /// Delete lines (D, Dn, or resolved DD pair).
    Delete { start_line: u64, count: u64 },
    /// Insert blank lines after a line.
    Insert { after_line: u64, count: u32 },
    /// Repeat (duplicate) lines.
    Repeat { start_line: u64, count: u32 },
    /// Repeat block (duplicate a range).
    RepeatBlock { start_line: u64, end_line: u64 },
    /// Copy lines to a target position.
    CopyToTarget(SourceTarget),
    /// Move lines to a target position.
    MoveToTarget(SourceTarget),
    /// Exclude lines from viewport.
    Exclude { start_line: u64, count: u64 },
    /// Tag lines.
    Tag { start_line: u64, end_line: u64 },
    /// Untag lines.
    Untag { start_line: u64, end_line: u64 },
    /// Shift lines right.
    ShiftRight { start_line: u64, end_line: u64, columns: u32 },
    /// Shift lines left.
    ShiftLeft { start_line: u64, end_line: u64, columns: u32 },
    /// Bounds-aware shift right.
    BoundsShiftRight { start_line: u64, end_line: u64 },
    /// Bounds-aware shift left.
    BoundsShiftLeft { start_line: u64, end_line: u64 },
}
```

### ExecutionEngine

```rust
/// Executes resolved line commands against the document model.
/// Wraps undoable operations in transactions; session-state operations
/// (exclude, tag) bypass the undo stack.
/// Addresses: Requirements 1–11
pub struct ExecutionEngine;

impl ExecutionEngine {
    /// Execute a resolved command against the document.
    /// Returns Ok with an optional transaction (None for session-state ops).
    /// Returns Err on failure (document unchanged).
    pub fn execute(
        command: &ExecutableCommand,
        document: &mut Document,
        display_mapping: &mut dyn DisplayLineMapping,
        config: &LineCommandConfig,
        bounds: Option<&EditBounds>,
    ) -> Result<Option<EditorTransaction>, LineCommandError>;

    /// Execute a delete operation (D, Dn, DD).
    /// Addresses: Requirement 1, criteria 1–6
    pub fn execute_delete(
        document: &mut Document,
        start_line: u64,
        count: u64,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute an insert operation (I, In).
    /// Addresses: Requirement 2, criteria 1–4
    pub fn execute_insert(
        document: &mut Document,
        after_line: u64,
        count: u32,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute a repeat operation (R, Rn).
    /// Addresses: Requirement 3, criteria 1–2, 5–6
    pub fn execute_repeat(
        document: &mut Document,
        line: u64,
        count: u32,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute a block repeat operation (RR).
    /// Addresses: Requirement 3, criteria 3–5
    pub fn execute_repeat_block(
        document: &mut Document,
        start_line: u64,
        end_line: u64,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute a copy-to-target operation.
    /// Addresses: Requirement 4, criteria 3, 6
    pub fn execute_copy(
        document: &mut Document,
        source_target: &SourceTarget,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute a move-to-target operation.
    /// Addresses: Requirement 5, criteria 3, 4, 7
    pub fn execute_move(
        document: &mut Document,
        source_target: &SourceTarget,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute an exclude operation (X, Xn, XX). Session-state, no transaction.
    /// Addresses: Requirement 7, criteria 1–6
    pub fn execute_exclude(
        display_mapping: &mut dyn DisplayLineMapping,
        start_line: u64,
        count: u64,
    ) -> Result<(), LineCommandError>;

    /// Execute a tag operation (T, TT). Session-state, no transaction.
    /// Addresses: Requirement 8, criteria 1–2, 7–8
    pub fn execute_tag(
        document: &mut Document,
        start_line: u64,
        end_line: u64,
    ) -> Result<(), LineCommandError>;

    /// Execute an untag operation (U, UU). Session-state, no transaction.
    /// Addresses: Requirement 8, criteria 3–4, 7–8
    pub fn execute_untag(
        document: &mut Document,
        start_line: u64,
        end_line: u64,
    ) -> Result<(), LineCommandError>;

    /// Execute a shift-right operation (>, >n, >>).
    /// Addresses: Requirement 9, criteria 1–6
    pub fn execute_shift_right(
        document: &mut Document,
        start_line: u64,
        end_line: u64,
        columns: u32,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute a shift-left operation (<, <n, <<).
    /// Truncates at first non-whitespace to prevent data loss.
    /// Addresses: Requirement 10, criteria 1–8
    pub fn execute_shift_left(
        document: &mut Document,
        start_line: u64,
        end_line: u64,
        columns: u32,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute a bounds-aware shift right (), )).
    /// Requires active Bounds; preserves content outside bounds.
    /// Addresses: Requirement 11, criteria 1–2, 5–6
    pub fn execute_bounds_shift_right(
        document: &mut Document,
        start_line: u64,
        end_line: u64,
        bounds: &EditBounds,
    ) -> Result<EditorTransaction, LineCommandError>;

    /// Execute a bounds-aware shift left ((, (().
    /// Requires active Bounds; preserves content outside bounds.
    /// Addresses: Requirement 11, criteria 3–6
    pub fn execute_bounds_shift_left(
        document: &mut Document,
        start_line: u64,
        end_line: u64,
        bounds: &EditBounds,
    ) -> Result<EditorTransaction, LineCommandError>;
}
```

---

## Error Handling

```rust
/// Errors produced by the ff-line-commands crate.
/// Formatted per Error Message Standards (Req 8): "[line-cmd] operation: description"
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LineCommandError {
    /// Unrecognised line command string entered in prefix area.
    /// Addresses: Requirement 14.6
    #[error("[line-cmd] parse: unrecognised command '{input}'")]
    InvalidCommand { input: String },

    /// Block command has only one marker — awaiting matching pair.
    /// Addresses: Requirements 1.5, 3.4, 7.4, 8.5, 8.6, 9.5, 10.5, 11.7, 11.8, 12.3
    #[error("[line-cmd] pair: {kind} requires a matching pair")]
    AwaitingPair { kind: String },

    /// More than two markers of the same block type present.
    /// Addresses: Requirement 12.4
    #[error("[line-cmd] pair: only two {kind} markers are permitted")]
    TooManyMarkers { kind: String },

    /// Overlapping block ranges from different block command types.
    /// Addresses: Requirement 12.5
    #[error("[line-cmd] pair: overlapping block ranges for {kind1} and {kind2}")]
    OverlappingBlocks { kind1: String, kind2: String },

    /// Move target falls inside the source block.
    /// Addresses: Requirement 5.4
    #[error("[line-cmd] move: target cannot be inside the source block")]
    TargetInsideSource,

    /// More than one A or B target marker pending.
    /// Addresses: Requirement 6.5
    #[error("[line-cmd] target: only one target marker is permitted per operation")]
    DuplicateTarget,

    /// Primary command is incompatible with pending line commands.
    /// Addresses: Requirement 13.2
    #[error("[line-cmd] compatibility: '{primary}' cannot be used with pending {line_cmd}")]
    IncompatibleCommands { primary: String, line_cmd: String },

    /// Source line commands combined with a file path argument on COPY/MOVE.
    /// Addresses: Requirement 13.7
    #[error("[line-cmd] compatibility: source line commands cannot be combined with a file path argument")]
    SourceWithFilePath,

    /// Bounds-aware shift attempted without active BOUNDS.
    /// Addresses: Requirement 11.5
    #[error("[line-cmd] bounds_shift: bounds-aware shift requires active BOUNDS")]
    NoBoundsActive,

    /// Line number is out of range for the document.
    #[error("[line-cmd] {operation}: line {line} is out of range (document has {total} lines)")]
    LineOutOfRange { operation: String, line: u64, total: u64 },

    /// Source markers awaiting target.
    /// Addresses: Requirements 4.4, 5.5
    #[error("[line-cmd] resolve: waiting for A or B target")]
    AwaitingTarget,

    /// Target markers awaiting source.
    /// Addresses: Requirement 6.4
    #[error("[line-cmd] resolve: A/B target entered with no pending source")]
    AwaitingSource,

    /// Document mutation failed.
    #[error("[line-cmd] {operation}: document error — {description}")]
    DocumentError { operation: String, description: String },
}
```

---

## Integration Points

### With `ff-document-model` (upstream — Wave 4)

- `ff-line-commands` uses the `Document` API for all buffer mutations:
  - `document.insert(position, text)` — insert blank lines (I, In)
  - `document.delete(position, length)` — delete lines (D, Dn, DD)
  - `document.get_range(position, length)` — read line content for copy, repeat, shift
  - `document.line_start(line)` / `document.line_end(line)` — compute line byte ranges
  - `document.line_count()` — validate line numbers are in range
- Line content for shift operations is read via `split_view()` or `get_range()`
- Insertions position new content at `line_end(line) + newline_length` (after semantics) or `line_start(line)` (before semantics)
- The `DocumentHandle` (`Arc<RwLock<Document>>`) provides thread-safe access

### With `ff-command` (upstream — Wave 2)

- All line command operations are registered as commands via `CommandRegistry`:
  - `linecmd.delete`, `linecmd.insert`, `linecmd.repeat`, `linecmd.copy`, `linecmd.move`
  - `linecmd.exclude`, `linecmd.tag`, `linecmd.untag`
  - `linecmd.shift_right`, `linecmd.shift_left`
  - `linecmd.bounds_shift_right`, `linecmd.bounds_shift_left`
  - `linecmd.resolve_cycle` — main entry point for processing pending commands
  - `linecmd.reset` — clear all pending commands
- Undoable operations return `CommandResult::OkUndoable` with a `Box<dyn UndoRecord>`
- Session-state operations (exclude, tag) return `CommandResult::Ok` without undo records
- Addresses: Requirement 14.8 — all operations dispatched through command framework

### With `ff-edit-operations` (upstream — Wave 4)

- `ff-line-commands` reuses `EditorTransaction` and `LineSnapshot` types from `ff-edit-operations` for transaction recording
- Shift operations leverage the BOUNDS concept from `EditBounds` in `ff-edit-operations`
- `TransactionRecorder::record()` is used to push transactions to the undo stack
- The `BoundsEnforcer` API provides bounds state for bounds-aware shift commands

### With `ff-display-line-mapping` (upstream — Wave 4)

- Exclude commands (X, Xn, XX) call `DisplayLineMapping::set_visible(doc_line, false)` to hide lines
- The `DisplayLineMapping` trait is the interface — `ff-line-commands` depends on the trait, not the concrete `ContractionState`
- Line number references in line commands refer to document lines, not display lines — the mapping is used only for visibility mutations

### With `ff-undo-redo-transactions` (upstream — Wave 4)

- Undoable line commands (D, DD, I, R, RR, C+A/B, M+A/B, >, >>, <, <<, ), )), (, (() wrap mutations in a single `EditorTransaction`
- The transaction is pushed via `TransactionStack` (trait from `ff-undo-redo-transactions`)
- Session-state commands (X, XX, T, TT, U, UU) explicitly bypass the undo stack

### With `ff-command-semantics` (peer — Wave 5)

- The primary command execution cycle (defined in `ff-command-semantics`) includes a "collect line commands" step that invokes the resolution engine
- `ResolutionEngine::resolve()` is called during the command cycle with the primary command context
- The `CompatibilityMatrix` gates execution based on the primary command in progress
- Line command parsing may share infrastructure with the primary command parser (prefix-area vs. command-line parsing are separate code paths but share the `LineCommandParser`)

### With `ff-exclude-show-filter` (downstream — Wave 5)

- Exclude line commands (X, Xn, XX) set the visibility flag; the `ff-exclude-show-filter` crate owns SHOW/INCLUDE/RESET EXCLUDED restoration
- `ff-line-commands` only sets `excluded = true`; it never restores visibility (that's `ff-exclude-show-filter`'s job)
- Both crates operate on the same `DisplayLineMapping` trait interface

### With `ff-navigation-commands` (peer — Wave 5)

- The BOUNDS/BNDS command (defined in `ff-navigation-commands`) establishes active column bounds
- Bounds-aware shift commands (), )), (, (( read the current bounds from the session state set by BOUNDS
- `ff-line-commands` reads bounds via the `EditBounds` struct but does not modify bounds

### With `ff-configuration-system` (upstream — Wave 2)

- Configuration key `editor.shift_width` (default: 2) controls the default shift distance
- The configuration system provides hot-reload notification — `LineCommandConfig` is refreshed on change
- Namespace: `[editor]` section in TOML

---

## Correctness Properties

The following properties should be verified using the `proptest` crate with a minimum of 100 iterations per property.

### Property 1: Parser Round-Trip Consistency

**Statement:** For any valid line command string that `LineCommandParser::parse` accepts, the resulting `ParsedLineCommand` can be classified into exactly one `LineCommandCategory`, and the category is deterministic for a given `LineCommandKind`.

**Validates: Requirements 1.1, 2.1, 3.1, 4.1, 5.1, 6.1, 7.1, 8.1, 9.1, 10.1, 11.1, 14.7**

```
∀ input ∈ valid_line_command_strings:
  let cmd = parse(input, line).unwrap();
  classify(&cmd.kind) ∈ {Immediate, Block, Source, Target}
  ∧ classify is pure (same input → same category)
```

### Property 2: Block Pair Normalization

**Statement:** For any two line numbers used as block markers, `BlockPairValidator::normalize` always produces `start_line ≤ end_line`, and the resulting pair spans exactly `(end_line - start_line + 1)` lines.

**Validates: Requirements 12.2**

```
∀ (line1, line2) ∈ u64 × u64:
  let (start, end) = normalize(line1, line2);
  start ≤ end
  ∧ (end - start + 1) == max(line1, line2) - min(line1, line2) + 1
```

### Property 3: Delete Preserves Document Integrity

**Statement:** After executing a delete command on n lines starting at line L in a document with T lines (where L + n ≤ T), the resulting document has exactly T - n lines, and all lines outside the deleted range retain their original content.

**Validates: Requirements 1.1, 1.2, 1.3**

```
∀ (doc, L, n) where L + n ≤ doc.line_count():
  let before = snapshot(doc);
  execute_delete(doc, L, n);
  doc.line_count() == before.line_count() - n
  ∧ ∀ i < L: line_content(doc, i) == before.line_content(i)
  ∧ ∀ i ≥ L: line_content(doc, i) == before.line_content(i + n)
```

### Property 4: Insert Line Count

**Statement:** After executing an insert of n blank lines after line L in a document with T lines, the document has exactly T + n lines. All original lines retain their content (shifted by n after the insertion point).

**Validates: Requirements 2.1, 2.2**

```
∀ (doc, L, n) where L < doc.line_count() ∧ n > 0:
  let before = snapshot(doc);
  execute_insert(doc, L, n);
  doc.line_count() == before.line_count() + n
  ∧ ∀ i ≤ L: line_content(doc, i) == before.line_content(i)
  ∧ ∀ i in (L+1)..=(L+n): line_content(doc, i) == ""
  ∧ ∀ i > L+n: line_content(doc, i) == before.line_content(i - n)
```

### Property 5: Repeat Produces Exact Duplicates

**Statement:** After executing a repeat of line L with count n, the document has n additional lines immediately after L, and each inserted line has identical content to the original line L.

**Validates: Requirements 3.1, 3.2**

```
∀ (doc, L, n) where L < doc.line_count() ∧ n > 0:
  let original_content = line_content(doc, L);
  let before_count = doc.line_count();
  execute_repeat(doc, L, n);
  doc.line_count() == before_count + n
  ∧ ∀ i in 1..=n: line_content(doc, L + i) == original_content
```

### Property 6: Shift Right Adds Exactly N Spaces

**Statement:** After executing a shift-right of n columns on line L, the line content is the original content prefixed with exactly n space characters.

**Validates: Requirements 9.1, 9.2**

```
∀ (doc, L, n) where L < doc.line_count() ∧ n > 0:
  let original = line_content(doc, L);
  execute_shift_right(doc, L, L, n);
  line_content(doc, L) == " ".repeat(n) + &original
```

### Property 7: Shift Left Non-Destructive

**Statement:** After executing a shift-left of n columns on a line that has at least n leading whitespace characters, the result is the original content with the first n characters removed. If the line has fewer than n leading whitespace characters, content is shifted only up to the first non-whitespace character.

**Validates: Requirements 10.1, 10.2, 10.8**

```
∀ (doc, L, n) where L < doc.line_count():
  let original = line_content(doc, L);
  let leading_ws = count_leading_whitespace(&original);
  execute_shift_left(doc, L, L, n);
  let shifted = line_content(doc, L);
  let actual_shift = min(n, leading_ws);
  shifted == original[actual_shift..]
```

### Property 8: Copy Does Not Modify Source

**Statement:** After executing a copy operation from source lines [S_start, S_end] to target T, the content of lines in [S_start, S_end] is unchanged, and the document grows by (S_end - S_start + 1) lines.

**Validates: Requirements 4.3**

```
∀ (doc, S_start, S_end, T) where valid_copy(doc, S_start, S_end, T):
  let source_content: Vec<_> = (S_start..=S_end).map(|i| line_content(doc, i)).collect();
  let before_count = doc.line_count();
  execute_copy(doc, SourceTarget { source_start: S_start, source_end: S_end, target_line: T, .. });
  doc.line_count() == before_count + (S_end - S_start + 1)
  ∧ source lines still contain source_content (adjusted for position shift if target < source)
```

### Property 9: Move Preserves Line Count

**Statement:** After executing a move operation from source lines [S_start, S_end] to target T (where target is outside the source range), the total document line count remains unchanged, and the source content appears at the new target position.

**Validates: Requirements 5.3, 5.4**

```
∀ (doc, S_start, S_end, T) where valid_move(doc, S_start, S_end, T) ∧ T ∉ [S_start, S_end]:
  let source_content: Vec<_> = (S_start..=S_end).map(|i| line_content(doc, i)).collect();
  let before_count = doc.line_count();
  execute_move(doc, SourceTarget { source_start: S_start, source_end: S_end, target_line: T, .. });
  doc.line_count() == before_count
  ∧ source_content appears contiguously at the adjusted target position
```

### Property 10: Bounds-Aware Shift Preserves Outer Content

**Statement:** After executing a bounds-aware shift right on line L with bounds [left, right], all characters outside columns [left, right] are unchanged.

**Validates: Requirements 11.1, 11.3**

```
∀ (doc, L, bounds) where L < doc.line_count() ∧ bounds.left < bounds.right:
  let original = line_content(doc, L);
  execute_bounds_shift_right(doc, L, L, &bounds);
  let shifted = line_content(doc, L);
  // Characters before bounds.left are identical
  shifted[..bounds.left-1] == original[..bounds.left-1]
  // Characters after bounds.right are identical
  shifted[bounds.right..] == original[bounds.right..]
```

### Property 11: Pending Store Size Monotonicity on Clear

**Statement:** After `clear_all()` is called on the PendingCommandStore, the store is empty (count == 0). Adding n commands results in count == n. Removing one command decrements count by 1.

**Validates: Requirements 14.1, 14.2, 14.5**

```
∀ store:
  store.clear_all();
  store.count() == 0
  ∧ ∀ n commands added sequentially: store.count() == n
  ∧ store.remove(existing_line) → store.count() == n - 1
```

### Property 12: Resolution Engine Idempotence for Pending-Only State

**Statement:** If the resolution engine is called with no new inputs and no primary command, the pending store does not change (commands remain pending). No commands are executed.

**Validates: Requirements 13.4, 14.3**

```
∀ pending_store containing only source/block markers (no immediate commands):
  let before = pending_store.clone();
  let result = resolve(&[], &mut pending_store, None);
  result.executable.is_empty()
  ∧ pending_store == before
```

### Property 13: Compatibility Matrix Symmetry

**Statement:** If a primary command P is incompatible with a line command set S, then executing `check_compatibility(Some(P), S)` always returns Err. If compatible, the executable commands in the resolution result are non-empty.

**Validates: Requirements 13.1, 13.2, 13.3**

```
∀ (P, S) ∈ incompatible_pairs:
  check_compatibility(Some(P), S).is_err()
∀ (P, S) ∈ compatible_pairs with resolved commands:
  check_compatibility(Some(P), S).is_ok()
```

---

## Testing Strategy

### Test Framework

- Property-based tests: `proptest` crate with minimum 100 iterations per property
- Unit tests: `#[cfg(test)] mod tests { ... }` within each source module
- Integration tests: `tests/` directory at crate root

### Test Generators (proptest strategies)

```rust
// Strategy for valid line command strings
fn arb_line_command_string() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("D".to_string()),
        (1u32..999).prop_map(|n| format!("D{}", n)),
        Just("DD".to_string()),
        Just("I".to_string()),
        (1u32..99).prop_map(|n| format!("I{}", n)),
        Just("R".to_string()),
        (1u32..99).prop_map(|n| format!("R{}", n)),
        Just("RR".to_string()),
        Just("C".to_string()),
        Just("CC".to_string()),
        Just("M".to_string()),
        Just("MM".to_string()),
        Just("A".to_string()),
        Just("B".to_string()),
        Just("X".to_string()),
        (1u32..999).prop_map(|n| format!("X{}", n)),
        Just("XX".to_string()),
        Just("T".to_string()),
        Just("TT".to_string()),
        Just("U".to_string()),
        Just("UU".to_string()),
        Just(">".to_string()),
        (1u32..99).prop_map(|n| format!(">{}", n)),
        Just(">>".to_string()),
        Just("<".to_string()),
        (1u32..99).prop_map(|n| format!("<{}", n)),
        Just("<<".to_string()),
        Just(")".to_string()),
        Just("))".to_string()),
        Just("(".to_string()),
        Just("((".to_string()),
    ]
}

// Strategy for document content (multi-line)
fn arb_document(min_lines: usize, max_lines: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-zA-Z0-9 ]{0,80}", min_lines..max_lines)
}

// Strategy for valid line numbers within a document
fn arb_line_in_doc(doc_lines: u64) -> impl Strategy<Value = u64> {
    0..doc_lines
}
```

### Key Test Scenarios

1. **Parser exhaustive coverage**: Every recognised command string parses successfully
2. **Parser rejection**: Random gibberish strings are rejected with `InvalidCommand`
3. **Block pair validation**: Two markers form a valid pair; one marker stays pending; three+ error
4. **Move target-inside-source rejection**: Confirms error when target ∈ [S_start, S_end]
5. **Shift left data preservation**: Never removes non-whitespace characters
6. **Bounds shift boundary preservation**: Characters outside bounds are untouched
7. **Pending store FIFO ordering**: Commands are iterable in entry order
8. **Resolution with no primary command**: Only immediate commands execute
9. **Copy/Move + target resolution**: Source + target pair resolves correctly
10. **Undo roundtrip**: Delete/Insert/Repeat/Shift → undo → document identical to before
