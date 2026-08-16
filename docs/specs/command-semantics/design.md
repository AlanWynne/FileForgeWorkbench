# Design Document: Command Semantics (`ff-command-semantics`)

## Overview

The `ff-command-semantics` crate is the **ISPF-inspired command execution pipeline** for FileForgeWorkbench. It accepts raw command-line text, parses it into structured tokens, resolves scope, validates preconditions, executes transactionally, and reports results via short status messages.

This crate is **GUI-independent** — it performs pure command parsing, scope resolution, and execution orchestration. It integrates with:

- `ff-command` — for registry, dispatch, and undo/redo wrapping
- `ff-document-model` — for document access, line queries, and mutations
- `ff-edit-operations` — for edit primitives invoked by commands

### Position in Architecture

```
Wave 5 — Command Engine

┌──────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                    │
├──────────────────────────────────────────────────────────────┤
│  Downstream consumers: find-and-replace, line-commands,       │
│    exclude-show-filter, navigation-commands                   │
│         (use ff-command-semantics parser + scope resolver)    │
├──────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-command-semantics ← Wave 5                    │
│    Command parser, scope resolver, execution pipeline,        │
│    session state, error handling, HELP, configuration         │
├──────────────────────────────────────────────────────────────┤
│  Upstream: ff-command (dispatch), ff-document-model (storage),│
│            ff-edit-operations (edit primitives),               │
│            ff-undo-redo-transactions (transaction wrapping)    │
├──────────────────────────────────────────────────────────────┤
│              Foundation: ff-logging, ff-configuration          │
└──────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Zero GUI dependencies
- **Command-Driven (Req 4)**: All commands register via `ff-command` CommandRegistry
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-command-semantics`
- **Error Message Standards (Req 8)**: Errors follow `[command-semantics] operation: description` format; status messages ≤200 chars
- **Transactional (Req 4)**: Every mutating command wraps in undo transaction
- **Async I/O (Req 6)**: Non-blocking where applicable; parser and scope resolver are synchronous

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Input Sources"
        A[Command Line Text]
        B[Prefix Area Input]
        C[Macro/Plugin invoke]
    end

    subgraph "ff-command-semantics"
        D[Primary Command Parser<br/>tokenise, quote handling, hex]
        E[Line Command Parser<br/>kind + count extraction]
        F[Command Normalizer<br/>case-fold, abbreviation resolve]
        G[Scope Resolver<br/>priority-ordered target resolution]
        H[Execution Pipeline<br/>collect→parse→resolve→validate→execute→report]
        I[Session State<br/>pending line cmds, last cmd, tags]
        J[Config Reader<br/>runtime config integration]
        K[Status Reporter<br/>≤200 char messages]
        L[Help Engine<br/>context-sensitive docs]
    end

    subgraph "Upstream Crates"
        M[ff-command<br/>CommandRegistry, Dispatch]
        N[ff-document-model<br/>Document, LineIndex]
        O[ff-edit-operations<br/>Edit primitives]
        P[ff-undo-redo-transactions<br/>Transaction wrapping]
        Q[ff-logging]
    end

    A --> D
    B --> E
    C --> H
    D --> F
    F --> H
    E --> I
    H --> G
    H --> I
    H --> J
    H --> K
    H --> L
    G --> N
    H --> M
    H --> O
    H --> P
    K --> Q
end
```

### Layer Placement

| Layer | Role |
|-------|------|
| **Parsing Layer** | Tokenises command-line text and prefix-area strings into structured AST nodes |
| **Normalization Layer** | Case-folds command names, resolves abbreviations to canonical forms |
| **Scope Resolution Layer** | Determines target lines/columns using priority algorithm |
| **Validation Layer** | Checks command-scope compatibility and preconditions |
| **Execution Layer** | Invokes command handler within undo transaction, manages session state |
| **Reporting Layer** | Produces ≤200-char status messages for success, error, and informational outcomes |

### Execution Pipeline Data Flow

```
1. User submits command-line text (or Enter with pending line commands)
2. CommandEngine collects pending line commands from SessionState
3. PrimaryCommandParser tokenises the text → ParsedCommand (name + args)
4. CommandNormalizer case-folds name, resolves abbreviations → canonical name
5. ScopeResolver applies priority algorithm → ResolvedScope
6. Validator checks command-scope compatibility
7. ExecutionPlan is built from (command, scope, args, session context)
8. Plan executes within undo transaction (via ff-undo-redo-transactions)
9. On success: consumed line commands cleared, status message emitted
10. On failure: transaction rolled back, line commands retained, error status emitted
```

---

## Components and Interfaces

```
crates/ff-command-semantics/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── engine.rs                   # CommandEngine — top-level pipeline orchestrator
│   ├── parser/
│   │   ├── mod.rs                  # Parser module re-exports
│   │   ├── primary.rs              # PrimaryCommandParser — tokenisation
│   │   ├── tokens.rs              # CommandToken, TokenKind, QuoteStyle enums
│   │   ├── hex_literal.rs          # Hex literal (X'...') parsing
│   │   ├── normalizer.rs           # Case-folding, abbreviation resolution
│   │   └── line_command.rs         # LineCommandParser — prefix-area parsing
│   ├── scope/
│   │   ├── mod.rs                  # Scope module re-exports
│   │   ├── resolver.rs            # ScopeResolver — priority-ordered algorithm
│   │   ├── types.rs               # ResolvedScope, ScopeSource, VisibilityModifier
│   │   └── bounds.rs              # ColumnBounds integration
│   ├── session.rs                  # SessionState — pending cmds, tags, last cmd
│   ├── plan.rs                     # ExecutionPlan construction and validation
│   ├── config.rs                   # CommandConfig — runtime configuration reader
│   ├── status.rs                   # StatusMessage, StatusKind, formatting
│   ├── help/
│   │   ├── mod.rs                  # Help module re-exports
│   │   ├── engine.rs              # HelpEngine — topic resolution, rendering
│   │   ├── topics.rs             # HelpTopic enum, built-in topic registry
│   │   └── formatter.rs           # Help text formatting (plain text output)
│   ├── registration.rs            # Command registration with ff-command
│   └── error.rs                    # CommandSemanticsError enum
└── tests/
    ├── parser_tests.rs             # Primary command parser property tests
    ├── line_command_tests.rs       # Line command parser property tests
    ├── scope_tests.rs              # Scope resolution property tests
    ├── engine_tests.rs             # Execution pipeline integration tests
    ├── config_tests.rs             # Configuration handling tests
    ├── status_tests.rs             # Status message formatting tests
    ├── help_tests.rs               # HELP command tests
    └── integration.rs              # End-to-end command execution scenarios
```

---

## Data Models

### CommandToken

```rust
/// A single lexical unit from the command line.
/// Addresses: Requirement 3, criteria 1–8
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandToken {
    /// A bare word (unquoted whitespace-delimited string).
    Word(String),
    /// A quoted string with the quote character stripped.
    QuotedString {
        value: String,
        quote_style: QuoteStyle,
    },
    /// A hex literal: X'hh...' decoded into a byte sequence.
    HexLiteral(Vec<u8>),
}

/// Which quote character enclosed a quoted string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteStyle {
    Single,
    Double,
}
```

### ParsedCommand

```rust
/// The result of parsing a command-line string.
/// Addresses: Requirement 3, criteria 1/4/5
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedCommand {
    /// No command was entered (empty/whitespace-only input).
    Empty,
    /// A command name followed by zero or more argument tokens.
    Command {
        /// The raw command name token (before normalization).
        name: String,
        /// The argument tokens following the command name.
        args: Vec<CommandToken>,
    },
}
```

### NormalizedCommand

```rust
/// A command after case-folding and abbreviation resolution.
/// Addresses: Requirement 3, criterion 4; Requirement 1, criterion 3
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedCommand {
    /// The canonical command name (uppercase, fully expanded).
    pub canonical_name: String,
    /// The original user-entered name (for error reporting).
    pub original_name: String,
    /// The argument tokens (unchanged from parsing).
    pub args: Vec<CommandToken>,
}
```

### LineCommandDescriptor

```rust
/// A parsed line command from the prefix area.
/// Addresses: Requirement 4, criteria 1–7
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineCommandDescriptor {
    /// A recognised line command with kind and repeat count.
    Known {
        kind: LineCommandKind,
        count: u32,
    },
    /// An unrecognised prefix-area string.
    Unknown(String),
}

/// The kind of a line command (single-line or block).
/// Addresses: Requirement 4, criterion 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LineCommandKind {
    // Single-line commands
    Copy,       // C
    Move,       // M
    Delete,     // D
    Repeat,     // R
    Exclude,    // X
    Insert,     // I
    After,      // A
    Before,     // B
    Overlay,    // O
    Show,       // W (show/reveal)
    Select,     // S
    Tag,        // T
    ShiftRight, // >
    ShiftLeft,  // <
    IndentIn,   // (
    IndentOut,  // )
    Bounds,     // ]

    // Block commands (paired)
    CopyBlock,    // CC
    MoveBlock,    // MM
    DeleteBlock,  // DD
    RepeatBlock,  // RR
    ExcludeBlock, // XX
    TagBlock,     // TT
}

impl LineCommandKind {
    /// Returns true if this is a block command that requires pairing.
    pub fn is_block(&self) -> bool;

    /// Returns the text representation (e.g., "C", "CC", "M", "MM").
    pub fn as_str(&self) -> &'static str;
}
```

### ResolvedScope

```rust
/// The resolved target scope for a command execution.
/// Addresses: Requirement 2, criteria 1–9
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedScope {
    /// The lines targeted by this scope.
    pub lines: ScopeLines,
    /// Optional column bounds restriction.
    pub column_bounds: Option<ColumnBounds>,
    /// The source that determined this scope (for diagnostics).
    pub source: ScopeSource,
}

/// Which lines are included in the scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeLines {
    /// A specific contiguous range of lines (inclusive, 0-based).
    Range { start: u64, end: u64 },
    /// The cursor line only.
    CursorLine(u64),
    /// The entire document.
    EntireDocument,
    /// Lines matching a visibility/tag filter.
    Filtered {
        /// Base range to filter within.
        base: Box<ScopeLines>,
        /// The filter to apply.
        filter: ScopeFilter,
    },
}

/// A filter applied to scope lines.
/// Addresses: Requirement 2, criteria 2–6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeFilter {
    /// Include only visible (non-excluded) lines.
    Visible,
    /// Include only excluded (hidden) lines.
    Excluded,
    /// Include all lines regardless of visibility.
    All,
    /// Include only tagged lines.
    Tagged,
    /// Include only non-tagged lines.
    NonTagged,
}

/// How the scope was determined (for diagnostics and priority tracking).
/// Addresses: Requirement 2, criterion 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeSource {
    /// Priority 1: Explicit line range in command arguments.
    ExplicitRange,
    /// Priority 2: Block line command pair (CC/CC, MM/MM, etc.).
    BlockSource,
    /// Priority 3: Single line command.
    SingleLineCommand,
    /// Priority 4: TAGGED/NONTAGGED modifier.
    TaggedModifier,
    /// Priority 5: VISIBLE/EXCLUDED/ALL modifier.
    VisibilityModifier,
    /// Priority 6: Cursor line.
    CursorLine,
    /// Priority 7: Entire document (default for commands that allow it).
    EntireDocument,
}

/// Column boundaries for column-sensitive operations.
/// Addresses: Requirement 2, criterion 7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnBounds {
    /// Left bound (1-based column number, inclusive).
    pub left: u32,
    /// Right bound (1-based column number, inclusive).
    pub right: u32,
}
```

### SessionState

```rust
/// Per-document mutable state maintained by the command engine.
/// Tracks pending line commands, last command, tags, and cursor.
/// Addresses: Requirement 1, criteria 1/2/5/6
pub struct SessionState {
    /// Line commands awaiting execution (line_number → descriptor).
    pending_line_commands: Vec<PendingLineCommand>,
    /// The last successfully executed command (for repeat).
    last_command: Option<NormalizedCommand>,
    /// The scope from the last execution (for RFIND/RCHANGE).
    last_scope: Option<ResolvedScope>,
    /// Per-line tag state (line numbers that are tagged).
    tagged_lines: HashSet<u64>,
    /// Current cursor line (0-based).
    cursor_line: u64,
    /// Current cursor column (0-based).
    cursor_column: u64,
    /// The last status message produced.
    last_status: Option<StatusMessage>,
}

/// A pending line command associated with a specific line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingLineCommand {
    /// The line number where this command was entered (0-based).
    pub line: u64,
    /// The parsed line command descriptor.
    pub descriptor: LineCommandDescriptor,
}

impl SessionState {
    /// Create a new empty session state.
    pub fn new() -> Self;

    /// Add a pending line command.
    pub fn add_pending(&mut self, line: u64, descriptor: LineCommandDescriptor);

    /// Drain and return all pending line commands, clearing them.
    pub fn take_pending(&mut self) -> Vec<PendingLineCommand>;

    /// Check if there are any pending line commands.
    pub fn has_pending(&self) -> bool;

    /// Clear consumed line commands (by line numbers).
    pub fn clear_consumed(&mut self, consumed_lines: &[u64]);

    /// Record a successful command execution.
    pub fn record_success(
        &mut self,
        command: NormalizedCommand,
        scope: ResolvedScope,
    );

    /// Tag a set of lines.
    pub fn tag_lines(&mut self, lines: impl IntoIterator<Item = u64>);

    /// Clear all tags.
    pub fn clear_tags(&mut self);

    /// Check if a line is tagged.
    pub fn is_tagged(&self, line: u64) -> bool;

    /// Update cursor position.
    pub fn set_cursor(&mut self, line: u64, column: u64);
}
```

### ExecutionPlan

```rust
/// A validated, ready-to-execute plan for a single command invocation.
/// Addresses: Requirement 1, criteria 1/7
#[derive(Debug)]
pub struct ExecutionPlan {
    /// The normalized command to execute.
    pub command: NormalizedCommand,
    /// The resolved scope for this execution.
    pub scope: ResolvedScope,
    /// Pending line commands collected for this execution.
    pub pending_line_commands: Vec<PendingLineCommand>,
    /// Whether this command mutates document state (determines undo wrapping).
    pub is_mutating: bool,
}
```

### StatusMessage

```rust
/// A status message produced by the command engine.
/// Guaranteed to be ≤200 characters.
/// Addresses: Requirement 5, criteria 1–7
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMessage {
    /// The message text (≤200 characters, truncated with "..." if needed).
    pub text: String,
    /// The severity/kind of this message.
    pub kind: StatusKind,
}

/// Categorisation of status messages.
/// Addresses: Requirement 5, criterion 7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    /// Informational success message (e.g., "CHANGE - 3 occurrences changed").
    Info,
    /// Syntax error from parsing (prefix: "Syntax error").
    SyntaxError,
    /// Structural error from command pairing (prefix: "Structure error").
    StructureError,
    /// Runtime error during execution (prefix: "Error").
    RuntimeError,
}

impl StatusMessage {
    /// Create a new status message, truncating to 200 chars if necessary.
    /// Addresses: Requirement 5, criterion 4
    pub fn new(text: impl Into<String>, kind: StatusKind) -> Self;

    /// Create an info message.
    pub fn info(text: impl Into<String>) -> Self;

    /// Create a syntax error message identifying the command.
    /// Addresses: Requirement 5, criterion 1
    pub fn syntax_error(command: &str, detail: &str) -> Self;

    /// Create a structure error message.
    /// Addresses: Requirement 5, criterion 2
    pub fn structure_error(command: &str, detail: &str) -> Self;

    /// Create a runtime error message.
    /// Addresses: Requirement 5, criterion 3
    pub fn runtime_error(command: &str, detail: &str) -> Self;
}
```

### CommandConfig

```rust
/// Runtime configuration for the command semantics engine.
/// Addresses: Requirement 6, criteria 1–6
#[derive(Debug, Clone, PartialEq)]
pub struct CommandConfig {
    /// Default scope for FIND/CHANGE when no explicit scope given.
    /// Addresses: Requirement 6, criterion 1
    pub find_default_scope: ScopeFilter,

    /// Whether column bounds restrict FIND/CHANGE search area.
    /// Addresses: Requirement 6, criterion 1
    pub bounds_affect_find: bool,

    /// Whether FIND/CHANGE defaults to case-sensitive matching.
    /// Addresses: Requirement 6, criterion 1
    pub case_sensitive_find: bool,

    /// Number of columns for > and < shift commands (1–72).
    /// Addresses: Requirement 6, criterion 1
    pub default_shift_width: u32,

    /// Whether RESET clears line tags in addition to exclusion state.
    /// Addresses: Requirement 6, criterion 1
    pub reset_clears_tags: bool,

    /// How unrecognised line commands are handled.
    /// Addresses: Requirement 6, criteria 4/5
    pub invalid_line_command_policy: InvalidLineCommandPolicy,
}

/// Policy for handling unrecognised line commands.
/// Addresses: Requirement 6, criteria 4/5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidLineCommandPolicy {
    /// Produce an error and abort the pipeline.
    Reject,
    /// Silently discard and continue.
    Ignore,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            find_default_scope: ScopeFilter::Visible,
            bounds_affect_find: true,
            case_sensitive_find: false,
            default_shift_width: 2,
            reset_clears_tags: false,
            invalid_line_command_policy: InvalidLineCommandPolicy::Reject,
        }
    }
}

impl CommandConfig {
    /// Load from configuration system values, applying defaults and clamping.
    /// Logs WARN for invalid values.
    /// Addresses: Requirement 6, criteria 2/3/6
    pub fn from_config_values(values: &ConfigValues) -> Self;

    /// Validate and clamp shift_width to [1, 72].
    /// Addresses: Requirement 6, criterion 6
    fn clamp_shift_width(value: u32) -> u32;
}
```

---

## Public API Surface

### CommandEngine

```rust
/// The top-level command execution pipeline orchestrator.
/// Accepts command-line text and pending line commands, drives the full pipeline.
/// Addresses: Requirement 1, all criteria
pub struct CommandEngine {
    config: CommandConfig,
    session: SessionState,
}

impl CommandEngine {
    /// Create a new CommandEngine with default configuration.
    pub fn new() -> Self;

    /// Create with explicit configuration.
    pub fn with_config(config: CommandConfig) -> Self;

    /// Execute a primary command from command-line text.
    /// Drives the full pipeline: parse → normalize → resolve → validate → execute → report.
    /// Addresses: Requirement 1, criterion 1
    pub fn execute_command_line(
        &mut self,
        text: &str,
        document: &mut Document,
        dispatch: &CommandDispatch,
    ) -> StatusMessage;

    /// Submit a line command from the prefix area.
    /// Adds it to pending state for later execution.
    /// Addresses: Requirement 1, criterion 2
    pub fn submit_line_command(
        &mut self,
        line: u64,
        prefix_text: &str,
    ) -> Result<(), StatusMessage>;

    /// Get the current session state (read-only).
    pub fn session(&self) -> &SessionState;

    /// Get mutable session state.
    pub fn session_mut(&mut self) -> &mut SessionState;

    /// Update configuration (e.g., on hot-reload notification).
    /// Addresses: Requirement 6, criterion 3
    pub fn update_config(&mut self, config: CommandConfig);

    /// Get current configuration.
    pub fn config(&self) -> &CommandConfig;
}
```

### PrimaryCommandParser

```rust
/// Tokenises command-line text into structured command representation.
/// Addresses: Requirement 3, all criteria
pub struct PrimaryCommandParser;

impl PrimaryCommandParser {
    /// Parse command-line text into a ParsedCommand.
    /// Addresses: Requirement 3, criteria 1–8
    pub fn parse(input: &str) -> Result<ParsedCommand, ParseError>;

    /// Reconstruct command-line text from tokens (for round-trip testing).
    /// Addresses: Requirement 3, criterion 6
    pub fn reconstruct(command: &ParsedCommand) -> String;
}
```

### LineCommandParser

```rust
/// Parses prefix-area strings into line command descriptors.
/// Addresses: Requirement 4, all criteria
pub struct LineCommandParser;

impl LineCommandParser {
    /// Parse a prefix-area string into a LineCommandDescriptor.
    /// Returns None for empty/whitespace-only input.
    /// Addresses: Requirement 4, criteria 1–7
    pub fn parse(input: &str) -> Option<LineCommandDescriptor>;
}
```

### ScopeResolver

```rust
/// Resolves the target scope for a command using the priority algorithm.
/// Addresses: Requirement 2, all criteria
pub struct ScopeResolver;

impl ScopeResolver {
    /// Resolve scope for a command given the current context.
    /// Applies the priority order defined in Requirement 2.1.
    /// Addresses: Requirement 2, criteria 1–9
    pub fn resolve(
        command_args: &[CommandToken],
        pending_line_commands: &[PendingLineCommand],
        session: &SessionState,
        document: &Document,
        config: &CommandConfig,
        allows_whole_document: bool,
    ) -> Result<ResolvedScope, ScopeError>;
}
```

### HelpEngine

```rust
/// Context-sensitive help system for commands, line commands, and macro API.
/// Addresses: Requirement 7, all criteria
pub struct HelpEngine {
    /// Registry of help topics.
    topics: Vec<HelpTopic>,
}

/// A single help topic entry.
#[derive(Debug, Clone)]
pub struct HelpTopic {
    /// The topic identifier (command name, "LINECOMMANDS", "MACRO", "API").
    pub key: String,
    /// Category for grouping.
    pub category: String,
    /// One-line description.
    pub summary: String,
    /// Full help text (syntax, modifiers, examples).
    pub full_text: String,
}

impl HelpEngine {
    /// Create a new HelpEngine and register built-in topics.
    pub fn new() -> Self;

    /// Register a help topic for a command.
    pub fn register_topic(&mut self, topic: HelpTopic);

    /// Show all commands grouped by category.
    /// Addresses: Requirement 7, criterion 1
    pub fn show_all(&self) -> String;

    /// Show help for a specific command.
    /// Addresses: Requirement 7, criterion 2
    pub fn show_command(&self, name: &str) -> Option<String>;

    /// Show all line commands.
    /// Addresses: Requirement 7, criterion 3
    pub fn show_line_commands(&self) -> String;

    /// Show macro API help.
    /// Addresses: Requirement 7, criterion 4
    pub fn show_macro_api(&self) -> String;

    /// Find close matches for an unknown topic.
    /// Addresses: Requirement 7, criterion 5
    pub fn suggest_matches(&self, query: &str) -> Vec<String>;
}
```

---

## Error Handling

```rust
/// Errors produced by the command semantics engine.
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CommandSemanticsError {
    /// A syntax error during command-line parsing.
    /// Addresses: Requirement 5, criterion 1
    #[error("[command-semantics] parse: {detail}")]
    ParseError { detail: String },

    /// A structural error (block command mismatch, overlapping blocks).
    /// Addresses: Requirement 5, criterion 2
    #[error("[command-semantics] structure: {detail}")]
    StructureError { detail: String },

    /// A scope resolution failure.
    /// Addresses: Requirement 2, criterion 8
    #[error("[command-semantics] scope: no valid scope for command '{command}'")]
    NoValidScope { command: String },

    /// Command name not recognised after normalization.
    /// Addresses: Requirement 1, criterion 4
    #[error("[command-semantics] dispatch: unknown command '{name}'")]
    UnknownCommand { name: String },

    /// Command and scope are incompatible.
    /// Addresses: Requirement 1, criterion 1 (step 5)
    #[error("[command-semantics] validate: command '{command}' incompatible with {scope_desc}")]
    IncompatibleScope {
        command: String,
        scope_desc: String,
    },

    /// Runtime execution failure.
    /// Addresses: Requirement 5, criterion 3
    #[error("[command-semantics] execute '{command}': {detail}")]
    ExecutionFailed { command: String, detail: String },

    /// Invalid line command (when policy is reject).
    /// Addresses: Requirement 6, criterion 4
    #[error("[command-semantics] line-command: unrecognised '{text}'")]
    InvalidLineCommand { text: String },

    /// Line command count out of range.
    /// Addresses: Requirement 4, criterion 7
    #[error("[command-semantics] line-command: count {count} exceeds maximum 99999")]
    LineCommandCountOverflow { count: u64 },

    /// Configuration value invalid (informational — default applied).
    /// Addresses: Requirement 6, criterion 2
    #[error("[command-semantics] config: invalid value for '{key}', using default")]
    ConfigInvalid { key: String },
}

/// Parser-specific error (subset, for parser return type clarity).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// Unclosed quoted string.
    /// Addresses: Requirement 3, criterion 8
    #[error("unclosed quote starting at position {position}")]
    UnclosedQuote { position: usize },

    /// Invalid hex literal format.
    /// Addresses: Requirement 3, criterion 3
    #[error("invalid hex literal at position {position}: {detail}")]
    InvalidHexLiteral { position: usize, detail: String },
}

/// Scope resolution specific error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScopeError {
    /// No scope could be resolved from any priority level.
    /// Addresses: Requirement 2, criterion 8
    #[error("no valid scope found")]
    NoScope,

    /// Block commands not properly paired.
    /// Addresses: Requirement 5, criterion 2
    #[error("block command '{kind}' at line {line} has no matching pair")]
    UnpairedBlock { kind: String, line: u64 },

    /// Overlapping block command ranges.
    #[error("overlapping block commands: {first} and {second}")]
    OverlappingBlocks { first: String, second: String },
}
```

---

## Integration Points

### With `ff-command` (upstream — Wave 2)

- `ff-command-semantics` registers all its commands with `CommandRegistry` during initialization via `registration.rs`
- All registered commands are discoverable via the standard `CommandRegistry::list_all()` / `list_by_category()` API
- The `CommandEngine` invokes commands through `CommandDispatch::execute_command()` for actual execution
- The HELP command is registered with Command_ID `"help.show"` (Requirement 7.8)
- Undo/redo wrapping uses the `UndoManager` trait provided through `CommandDispatch`

### With `ff-document-model` (upstream — Wave 4)

- `ScopeResolver` queries document `line_count()` to validate line ranges
- `ScopeResolver` queries document `line_start()` and `line_end()` for column bounds
- The execution pipeline reads document content for scope-dependent validation
- `SessionState` cursor position corresponds to document `LineNumber` values

### With `ff-edit-operations` (upstream — Wave 4)

- The execution pipeline delegates actual edit operations (insert, delete, shift) to `ff-edit-operations` engines
- Column bounds from scope resolution map to the `BoundsEnforcer` in edit-operations
- Line manipulation commands (shift >, <) use edit-operations primitives

### With `ff-undo-redo-transactions` (upstream — Wave 4)

- Every mutating command execution is wrapped in a transaction (Requirement 1.7)
- On failure, the transaction is rolled back — no partial state persists
- Non-mutating commands (HELP, informational queries) are NOT wrapped

### With `ff-configuration` (upstream — Wave 2)

- Configuration keys `commands.*` are read at startup and on hot-reload
- `CommandConfig::from_config_values()` translates raw config into typed struct
- Invalid values trigger WARN log and fallback to defaults (Requirement 6.2)

### With `ff-logging` (upstream — Wave 0)

- WARN-level logs for: invalid configuration values, config clamping, parse recovery
- ERROR-level logs for: unrecoverable execution failures
- DEBUG-level logs for: pipeline step tracing, scope resolution details

### With downstream Wave 5 crates (consumers)

| Consumer | What it uses |
|----------|-------------|
| `find-and-replace` | `PrimaryCommandParser`, `ScopeResolver`, `CommandConfig` (find scope defaults, bounds, case) |
| `line-commands` | `LineCommandParser`, `LineCommandKind`, `PendingLineCommand`, `SessionState` pending management |
| `exclude-show-filter` | `ScopeResolver` (visibility modifiers), `ScopeFilter`, `SessionState` tag management |
| `navigation-commands` | `PrimaryCommandParser`, `ScopeResolver`, `ColumnBounds` |

### Dependency Direction

```
ff-logging ← ff-command ← ff-document-model ← ff-edit-operations
                         ← ff-undo-redo-transactions
                                    ↑
                         ff-command-semantics (this crate)
                                    ↓
                         find-and-replace, line-commands,
                         exclude-show-filter, navigation-commands
```

`ff-command-semantics` depends on: `ff-command`, `ff-document-model`, `ff-edit-operations`, `ff-undo-redo-transactions`, `ff-logging`.

---

## Configuration

All configuration consumed by `ff-command-semantics` is provided through the configuration system. The crate reads values at initialization and on hot-reload notification.

### Configuration Keys

```toml
[commands]
# Default scope for FIND/CHANGE when no explicit scope specified.
# Valid values: "visible", "all", "excluded"
# Default: "visible"
# Addresses: Requirement 6, criterion 1
find_default_scope = "visible"

# Whether column bounds restrict FIND/CHANGE search area.
# Default: true
# Addresses: Requirement 6, criterion 1
bounds_affect_find = true

# Whether FIND/CHANGE defaults to case-sensitive matching.
# Default: false
# Addresses: Requirement 6, criterion 1
case_sensitive_find = false

# Number of columns for > and < shift line commands.
# Range: 1–72. Values outside range are clamped with WARN.
# Default: 2
# Addresses: Requirement 6, criteria 1/6
default_shift_width = 2

# Whether RESET clears line tags in addition to exclusion state.
# Default: false
# Addresses: Requirement 6, criterion 1
reset_clears_tags = false

# How unrecognised line commands are handled.
# Valid values: "reject", "ignore"
# Default: "reject"
# Addresses: Requirement 6, criteria 4/5
invalid_line_command_policy = "reject"
```

---

## Concurrency Model

### Thread-Safety Approach

| Component | Mechanism | Rationale |
|-----------|-----------|-----------|
| `CommandEngine` | Owned per-editor-tab; not shared across threads | Each document session has its own engine instance |
| `SessionState` | Owned by `CommandEngine`; single-writer | Session state is per-document, mutated only by the command pipeline |
| `CommandConfig` | `Arc<RwLock<CommandConfig>>` for hot-reload | Config may be updated from config watcher thread |
| `HelpEngine` | `Arc<RwLock<HelpEngine>>` | Topics may be registered by plugins at runtime |
| Parsers | Stateless, pure functions | No shared state; thread-safe by construction |
| `ScopeResolver` | Stateless, borrows inputs | No shared state; thread-safe by construction |

### Hot-Reload Flow

1. Configuration system detects file change
2. Config system sends notification to subscribed crates
3. `CommandEngine` receives notification, calls `CommandConfig::from_config_values()` with new values
4. New config replaces old config via `update_config()`
5. Subsequent command executions use new config values

---

## Correctness Properties

These properties are suitable for property-based testing with `proptest`. They validate invariants that must hold across all valid inputs.

### Property 1: Parser Round-Trip

**Statement**: For any valid command-line input (no unclosed quotes, no invalid hex literals), parsing the text and then reconstructing it from the resulting tokens produces output that, when re-parsed, yields the same token sequence.

**Validates: Requirements 3.6**

```rust
// proptest strategy: generate valid command-line strings (bare words, quoted strings, hex literals)
// assertion: parse(reconstruct(parse(input))) == parse(input)
```

### Property 2: Parser Rejects Unclosed Quotes

**Statement**: For any input string that contains an opening quote character (`'` or `"`) with no matching closing quote, `PrimaryCommandParser::parse()` returns `Err(ParseError::UnclosedQuote { .. })`.

**Validates: Requirements 3.8**

```rust
// proptest strategy: generate strings with deliberate unclosed quotes
// assertion: parse returns Err(UnclosedQuote)
```

### Property 3: Command Name Case Insensitivity

**Statement**: For any command name string, parsing and normalizing produces the same canonical name regardless of the input case (upper, lower, mixed).

**Validates: Requirements 3.4**

```rust
// proptest strategy: generate command names, vary case randomly
// assertion: normalize(upper_case) == normalize(lower_case) == normalize(mixed_case)
```

### Property 4: Line Command Kind-Count Separation

**Statement**: For any prefix-area string consisting of an alphabetic prefix followed by digits, the parser extracts the maximal alphabetic prefix as the kind and the remaining digits as the count. The count defaults to 1 when no digits are present.

**Validates: Requirements 4.5**

```rust
// proptest strategy: generate (kind_letters, count_digits) pairs, concatenate
// assertion: parsed.kind == expected_kind && parsed.count == expected_count (or 1 if no digits)
```

### Property 5: Line Command Count Overflow Rejection

**Statement**: For any prefix-area string where the numeric suffix exceeds 99999, the parser produces an error rather than silently truncating or wrapping.

**Validates: Requirements 4.7**

```rust
// proptest strategy: generate valid kind + count > 99999
// assertion: parse returns error indicating count out of range
```

### Property 6: Scope Priority Ordering

**Statement**: For any combination of scope sources (explicit range, block source, single line command, tagged modifier, visibility modifier, cursor line, entire document), the highest-priority source always wins. If a higher-priority source is present, lower-priority sources are ignored.

**Validates: Requirements 2.1, 2.9**

```rust
// proptest strategy: generate combinations of scope inputs at multiple priority levels
// assertion: resolved scope source == highest priority present source
```

### Property 7: Status Message Length Invariant

**Statement**: For any error or success condition, the produced `StatusMessage.text` is at most 200 characters in length. Messages that would exceed this limit are truncated with a trailing "...".

**Validates: Requirements 5.4**

```rust
// proptest strategy: generate arbitrary error descriptions and command names of varying length
// assertion: StatusMessage::new(text, kind).text.len() <= 200
```

### Property 8: Empty Input Produces "No command"

**Statement**: For any input string that is empty or consists only of whitespace, and when there are no pending line commands, `execute_command_line` produces a status message with text "No command".

**Validates: Requirements 1.3**

```rust
// proptest strategy: generate whitespace-only strings of varying length
// assertion: execute_command_line(whitespace, empty_session) produces "No command"
```

### Property 9: Failed Execution Retains Pending Line Commands

**Statement**: For any command execution that fails (handler returns error), all pending line commands in SessionState remain unchanged — none are cleared.

**Validates: Requirements 1.6**

```rust
// proptest strategy: generate pending line commands + failing command
// assertion: session.pending_line_commands unchanged after failure
```

### Property 10: Successful Execution Clears Consumed Line Commands

**Statement**: For any command execution that succeeds, all line commands that were consumed by the execution are removed from SessionState. Line commands not consumed remain pending.

**Validates: Requirements 1.5**

```rust
// proptest strategy: generate pending line commands (some consumed, some not) + succeeding command
// assertion: consumed commands removed; non-consumed remain
```

### Property 11: Configuration Clamping Invariant

**Statement**: For any integer value for `commands.default_shift_width`, the effective value is always within [1, 72]. Values < 1 are clamped to 1, values > 72 are clamped to 72.

**Validates: Requirements 6.6**

```rust
// proptest strategy: generate u32 values across full range
// assertion: effective_shift_width ∈ [1, 72]
```

### Property 12: Hex Literal Byte Fidelity

**Statement**: For any even-length string of hexadecimal digit characters, wrapping them as `X'...'` and parsing produces a `HexLiteral` token whose byte vector has length equal to half the hex digit count, with each byte matching the corresponding pair of hex digits.

**Validates: Requirements 3.3**

```rust
// proptest strategy: generate even-length hex digit strings
// assertion: parsed hex literal bytes == expected decoded bytes
```

---

## Testing Strategy

### Unit Tests

Unit tests are co-located with source modules using `#[cfg(test)] mod tests { ... }`:

- `parser/primary.rs` — tokenisation of bare words, quoted strings, hex literals, edge cases
- `parser/line_command.rs` — kind extraction, count parsing, boundary cases
- `parser/normalizer.rs` — case folding, abbreviation resolution
- `scope/resolver.rs` — priority ordering, filter application, bounds integration
- `session.rs` — pending command management, tag state, cursor tracking
- `status.rs` — message truncation, prefix formatting
- `config.rs` — clamping, default fallback, invalid value handling
- `help/engine.rs` — topic resolution, close-match suggestions

### Property-Based Tests

Property tests use `proptest` and live in the `tests/` directory:

- `parser_tests.rs` — Properties 1, 2, 3, 12 (round-trip, unclosed quotes, case insensitivity, hex fidelity)
- `line_command_tests.rs` — Properties 4, 5 (kind-count separation, overflow rejection)
- `scope_tests.rs` — Property 6 (priority ordering)
- `status_tests.rs` — Property 7 (message length invariant)
- `engine_tests.rs` — Properties 8, 9, 10 (empty input, failure retention, success clearing)
- `config_tests.rs` — Property 11 (clamping invariant)

### Integration Tests

`tests/integration.rs` exercises end-to-end scenarios:

- Full pipeline from text input through execution to status message
- Line command submission through pending state to execution
- Configuration hot-reload affecting subsequent executions
- HELP command in all modes (edit, browse, view)

### Test Framework

- Framework: `proptest` 1.0 with minimum 100 cases per property
- Assertions: `pretty_assertions::assert_eq!` for readable diffs
- No mocking: tests operate on real `Document` instances (in-memory, no VFS)

---

## Appendix A: External Crate Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `thiserror` | 2.0 | Error type derivation |
| `proptest` | 1.0 | Property-based testing (dev-dependency only) |
| `pretty_assertions` | 1.4 | Enhanced test assertion diffs (dev-dependency only) |

### Workspace Dependencies (from upstream crates)

| Crate | Purpose |
|-------|---------|
| `ff-logging` | Structured logging (WARN for config issues, DEBUG for pipeline tracing) |
| `ff-command` | Command registry, dispatch, undo integration |
| `ff-document-model` | Document access, line queries, content reads |
| `ff-edit-operations` | Edit primitives for mutating commands |
| `ff-undo-redo-transactions` | Transaction wrapping for rollback semantics |

---

## Appendix B: Command Registration Table

Commands registered by `ff-command-semantics` with the global `CommandRegistry`:

| Command_ID | Display Name | Undoable | Category | Description |
|-----------|-------------|----------|----------|-------------|
| `help.show` | Help | No | help | Display context-sensitive help |

> Note: Most ISPF primary commands (FIND, CHANGE, LOCATE, SORT, etc.) are registered by their respective downstream crates (`find-and-replace`, `navigation-commands`). This crate provides the **infrastructure** (parser, scope resolver, pipeline) that those crates consume, plus the HELP meta-command.

---

## Appendix C: Command Name Normalization Rules

1. The raw command name is converted to uppercase: `find` → `FIND`
2. Known abbreviations are expanded to their canonical form:
   - `F` → `FIND`, `C` → `CHANGE` (context: primary command position)
   - `L` → `LOCATE`, `X` → `EXCLUDE`
   - `RES` → `RESET`, `SUB` → `SUBMIT`
3. Abbreviation resolution is only applied to the **first token** (command name); argument tokens are never expanded
4. If no abbreviation match is found, the uppercase name is used as-is for registry lookup
5. Abbreviation definitions are extensible — new abbreviations can be registered alongside new commands

---

## Appendix D: Scope Resolution Priority Decision Table

| Priority | Source | Example |
|----------|--------|---------|
| 1 (highest) | Explicit line range in args | `CHANGE 'a' 'b' 10 20` (lines 10–20) |
| 2 | Block line command pair | CC on lines 5 and 10 → range 5–10 |
| 3 | Single line command | C on line 7 → line 7 only |
| 4 | TAGGED modifier | `CHANGE 'a' 'b' TAGGED` → tagged lines only |
| 5 | Visibility modifier | `FIND 'x' EXCLUDED` → excluded lines only |
| 6 | Cursor line | No other source → cursor line |
| 7 (lowest) | Entire document | Commands that default to whole-doc scope |

When multiple sources are present, the highest-priority source wins. Lower-priority sources are silently ignored (no error for conflict — Requirement 2.9).
