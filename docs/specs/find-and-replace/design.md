# Design Document: Find and Replace (`ff-find-and-replace`)

## Overview

The `ff-find-and-replace` crate is the **search and replacement engine** for FileForgeWorkbench. It implements ISPF-style FIND/RFIND/CHANGE/RCHANGE commands with literal, regular expression, and hexadecimal search modes, combined with Unicode case folding, whole-word matching, column-bounded search, and incremental search-as-you-type.

### Purpose

- Execute literal, regex, and hex-byte searches over document buffers
- Perform text replacements with regex group substitution
- Maintain session-scoped find/change state for RFIND/RCHANGE repetition
- Provide Unicode Full Case Folding for case-insensitive search across all scripts
- Support scope filtering (TAGGED, EXCLUDED, VISIBLE, NONTAGGED) and column bounds
- Drive incremental search and highlight-all-matches for live UI feedback
- Integrate with the command framework for undo-wrapped CHANGE transactions
- Emit search events for plugins and UI status updates

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
├─────────────────────────────────────────────────────────────┤
│  UI Crates: ff-text-decorations (highlight rendering),       │
│    ff-menu-and-statusbar (find panel status)                 │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-find-and-replace ← Wave 5                    │
├─────────────────────────────────────────────────────────────┤
│  Upstream: ff-document-model (buffer access, CharacterIndexer)│
│            ff-command (command registration, dispatch)        │
│            ff-display-line-mapping (line visibility queries)  │
│            ff-undo-redo-transactions (change transactions)    │
│            ff-exclude-show-filter (line tag/visibility state) │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: No direct filesystem access — document content accessed via `CharacterIndexer` trait over the document model
- **GUI Independence (Req 2)**: Zero GUI dependencies — no egui, winit, wgpu; search panel and highlight rendering are separate crate concerns
- **Command-Driven (Req 4)**: FIND/RFIND/CHANGE/RCHANGE registered as commands in `ff-command`; CHANGE operations produce `UndoRecord`
- **Async I/O (Req 6)**: Long-running FIND ALL / CHANGE ALL support cancellation tokens and progress events
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-find-and-replace`
- **Error Message Standards (Req 8)**: All errors follow `[find-replace] operation: description` format

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Consumers [Consuming Crates]
        CMD_SEM[ff-command-semantics<br/>command parser/dispatcher]
        UI_DEC[ff-text-decorations<br/>match highlighting]
        ESF[ff-exclude-show-filter<br/>EXCLUDE/SHOW text matching]
        MACRO[ff-lua-macro-engine<br/>scripting bridge]
    end

    subgraph ff-find-and-replace [ff-find-and-replace Crate]
        FE[FindEngine]
        RE[RegexEngine<br/>NFA compiler + executor]
        CF[CaseFolder<br/>Unicode Full Case Folding]
        ST[FindState<br/>session persistence]
        SUB[SubstitutionEngine<br/>template expansion]
        INC[IncrementalSearch<br/>debounced live search]
        HAM[HighlightAllMatches<br/>viewport match collector]
        EVT[EventEmitter<br/>find/replace events]
        CI[CharacterIndexer trait<br/>buffer access abstraction]
    end

    subgraph Upstream [Upstream Crates]
        DOC[ff-document-model<br/>Document / TextBuffer]
        CMD[ff-command<br/>registry + dispatch]
        DLM[ff-display-line-mapping<br/>visibility queries]
        UNDO[ff-undo-redo-transactions]
        LOG[ff-logging]
    end

    CMD_SEM -->|execute find/change| FE
    UI_DEC -->|query matches| HAM
    ESF -->|delegate text match| FE
    MACRO -->|scripting invoke| FE

    FE --> RE
    FE --> CF
    FE --> ST
    FE --> SUB
    FE --> INC
    FE --> HAM
    FE --> EVT
    FE --> CI

    CI -->|char_at / slice / line_range| DOC
    FE -->|register commands| CMD
    FE -->|query visibility| DLM
    FE -->|wrap changes| UNDO
    FE --> LOG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **FindEngine** | Top-level orchestrator: accepts `FindRequest`/`ChangeRequest`, coordinates scope filtering, delegates to literal/regex/hex searchers, manages state transitions |
| **RegexEngine** | NFA-based regex: compiles patterns into NFA bytecode, executes against `CharacterIndexer`, captures groups 0–9, supports lazy/greedy quantifiers |
| **CaseFolder** | Unicode Full Case Folding (CaseFolding.txt status C+F): stateless, thread-safe fold function with optional locale hint |
| **SubstitutionEngine** | Expands replacement templates (`\1`–`\9`, `$1`–`$9`) against captured groups, handles escape sequences |
| **FindState** | Per-document session state: last search, last change, search history ring, replacement history ring, serialisable |
| **IncrementalSearch** | Debounced live-search coordinator: cancels in-progress searches on keystroke, enforces time budget |
| **HighlightAllMatches** | Viewport-scoped match collector: finds all matches in visible range with configurable cap (default 1000) |
| **EventEmitter** | Typed event dispatch: `find_started`, `match_found`, `find_completed`, `replace_completed` for plugins/UI |
| **CharacterIndexer** | Trait abstracting byte-level document access for the search algorithm, implemented by `ff-document-model` |

### Data Flow: FIND Command

```
1. Command layer parses "FIND 'text' NEXT TAGGED" into a FindRequest
2. FindEngine receives FindRequest + CharacterIndexer + document metadata
3. FindEngine resolves SearchScope (filters lines via DLM visibility + tag state)
4. FindEngine resolves ColumnRange (from explicit range or active Bounds)
5. If case-insensitive: CaseFolder pre-folds the search term once
6. FindEngine iterates eligible lines from cursor position in specified direction
7. For each line: extract bounded slice via CharacterIndexer.line_range + column clip
8. Literal: memchr fast-path scan; Regex: NFA execution; Hex: raw byte compare
9. On match: construct FindResult (byte range, line, captures)
10. FindEngine stores FindRequest in FindState for RFIND
11. EventEmitter fires match_found; command layer scrolls viewport
```

### Data Flow: CHANGE ALL Command

```
1. Command layer parses "CHANGE 'old' 'new' ALL TAGGED" into a ChangeRequest
2. FindEngine begins an undo transaction via ff-undo-redo-transactions
3. FindEngine iterates ALL eligible lines (scope + direction = forward from start)
4. For each match found: SubstitutionEngine expands template, computes replacement
5. Replacement applied to document via CharacterIndexer (insert/delete)
6. Byte positions adjusted for subsequent matches (length delta tracking)
7. Progress events emitted every N matches
8. On completion: commit transaction, emit replace_completed with count
9. FindState updated with ChangeRequest for RCHANGE
```

---

## Components and Interfaces

### Module Structure

```
crates/ff-find-and-replace/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── engine.rs                   # FindEngine: top-level search orchestrator
│   ├── request.rs                  # FindRequest, ChangeRequest value types
│   ├── result.rs                   # FindResult, ChangeResult, FindError
│   ├── search_mode.rs             # SearchMode enum (Literal, Regex, Hex)
│   ├── direction.rs               # SearchDirection enum (Next, Prev, First, Last)
│   ├── scope.rs                   # SearchScope, ScopeFilter, ColumnRange, Bounds
│   ├── state.rs                   # FindState: per-document session state
│   ├── indexer.rs                 # CharacterIndexer trait definition
│   ├── case_folder/
│   │   ├── mod.rs                 # CaseFolder re-exports
│   │   ├── folder.rs             # Unicode Full Case Folding implementation
│   │   ├── tables.rs             # Generated case-folding lookup tables
│   │   └── locale.rs             # Locale-sensitive folding (Turkish dotted-I)
│   ├── regex/
│   │   ├── mod.rs                 # RegexEngine re-exports
│   │   ├── compiler.rs           # Pattern → NFA bytecode compiler
│   │   ├── nfa.rs                # NFA representation and execution
│   │   ├── charset.rs            # Character class parsing and matching
│   │   ├── captures.rs           # CaptureGroup storage
│   │   └── error.rs              # Regex compilation errors
│   ├── literal.rs                 # Optimised literal search (memchr + Boyer-Moore)
│   ├── hex_search.rs             # Hex byte pattern parsing and matching
│   ├── substitution.rs           # SubstitutionEngine: template expansion
│   ├── word_boundary.rs          # Whole-word and word-start boundary checks
│   ├── incremental.rs            # IncrementalSearch: debounced live search
│   ├── highlight_all.rs          # HighlightAllMatches: viewport match collector
│   ├── events.rs                 # FindEvent enum, EventEmitter
│   ├── commands.rs               # Command framework registration (find, rfind, etc.)
│   ├── types.rs                  # BytePosition re-export, MatchRange, LineNumber
│   └── error.rs                  # FindReplaceError enum
└── tests/
    ├── literal_search_tests.rs    # Literal FIND property + unit tests
    ├── regex_tests.rs             # Regex compilation and matching tests
    ├── case_folder_tests.rs       # Unicode case folding property tests
    ├── hex_search_tests.rs        # Hex pattern parsing and matching tests
    ├── change_tests.rs            # CHANGE command tests
    ├── scope_tests.rs             # Scope and column filtering tests
    ├── state_tests.rs             # FindState persistence and RFIND/RCHANGE
    ├── incremental_tests.rs       # Incremental search debounce tests
    ├── highlight_all_tests.rs     # Highlight-all viewport collection tests
    ├── substitution_tests.rs      # Template expansion tests
    ├── word_boundary_tests.rs     # Word/WordStart boundary tests
    ├── integration.rs             # End-to-end with mock document
    └── property_tests.rs          # Cross-cutting proptest properties
```

---

## Data Models

### Core Newtypes and Enums

```rust
/// Re-export from ff-document-model for convenience.
pub use ff_document_model::{BytePosition, LineNumber};

/// A byte range within the document representing a match.
///
/// Addresses: Requirement 1 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchRange {
    /// Start byte position (inclusive)
    pub start: BytePosition,
    /// End byte position (exclusive)
    pub end: BytePosition,
}

impl MatchRange {
    pub fn length(&self) -> u64 {
        self.end.0 - self.start.0
    }
}

/// How the search term is interpreted.
///
/// Addresses: Requirements 1, 3, 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Plain text matching (default).
    Literal,
    /// Regular expression pattern.
    Regex,
    /// Raw hex byte sequence (e.g., X'4A5B').
    HexBytes,
}

/// Direction of traversal for FIND/CHANGE.
///
/// Addresses: Requirement 1 AC 2–5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    /// Next match after cursor (default for FIND).
    Next,
    /// Previous match before cursor.
    Prev,
    /// First match from document start.
    First,
    /// Last match from document end.
    Last,
}

/// Scope filter controlling which lines are eligible for search.
///
/// Addresses: Requirement 2 AC 1–4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeModifier {
    /// Search all lines regardless of state (FIND ALL).
    All,
    /// Search only visible lines (default for FIND without ALL).
    Visible,
    /// Search only excluded (hidden) lines.
    Excluded,
    /// Search only tagged lines.
    Tagged,
    /// Search only non-tagged lines.
    NonTagged,
}
```

### Column Range and Bounds

```rust
/// An optional column range restricting search to a horizontal slice.
///
/// Columns are 1-based and refer to character positions within a line.
///
/// Addresses: Requirement 2 AC 5–7, Requirement 7 AC 4–6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnRange {
    /// Start column (1-based, inclusive).
    pub start: u32,
    /// End column (1-based, inclusive).
    pub end: u32,
}

impl ColumnRange {
    /// Create a new column range, enforcing start <= end.
    pub fn new(start: u32, end: u32) -> Option<Self>;

    /// Compute the intersection of two column ranges.
    /// Returns None if they don't overlap.
    pub fn intersect(&self, other: &ColumnRange) -> Option<ColumnRange>;
}

/// Active BOUNDS settings affecting find operations.
///
/// Addresses: Requirement 2 AC 5–6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    /// Left boundary column (1-based, inclusive).
    pub left: u32,
    /// Right boundary column (1-based, inclusive).
    pub right: u32,
}
```

### FindRequest

```rust
/// Complete specification for a single FIND operation.
///
/// Addresses: Requirements 1–5
#[derive(Debug, Clone)]
pub struct FindRequest {
    /// The search term (literal text, regex pattern, or hex string).
    pub term: String,
    /// How to interpret the search term.
    pub mode: SearchMode,
    /// Traversal direction.
    pub direction: SearchDirection,
    /// Scope filter (which lines to search).
    pub scope: ScopeModifier,
    /// Case sensitivity flag (true = case-sensitive, default).
    pub case_sensitive: bool,
    /// Whole-word matching flag.
    pub word_match: WordMatchMode,
    /// Optional explicit column range override.
    pub column_range: Option<ColumnRange>,
    /// Current cursor position (byte offset) for NEXT/PREV.
    pub cursor_position: BytePosition,
}

/// Word-matching mode for boundary constraints.
///
/// Addresses: Requirement 11 AC 1–2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordMatchMode {
    /// No word boundary constraints.
    #[default]
    None,
    /// Match must be a complete word (boundaries at both ends).
    WholeWord,
    /// Match must start at a word boundary.
    WordStart,
}
```

### ChangeRequest

```rust
/// Complete specification for a single CHANGE operation.
///
/// Addresses: Requirements 6–9
#[derive(Debug, Clone)]
pub struct ChangeRequest {
    /// The search portion (same semantics as FindRequest).
    pub find: FindRequest,
    /// The replacement text or template.
    pub replacement: String,
}
```

### FindResult and ChangeResult

```rust
/// The result of a successful FIND operation.
///
/// Addresses: Requirement 1 AC 1, Requirement 4 AC 9
#[derive(Debug, Clone)]
pub struct FindResult {
    /// The byte range of the match in the document.
    pub match_range: MatchRange,
    /// The document line containing the match start.
    pub line: LineNumber,
    /// Captured groups (index 0 = entire match, 1–9 = sub-groups).
    /// Empty for literal and hex searches.
    pub captures: Vec<MatchRange>,
}

/// The result of a successful CHANGE operation.
///
/// Addresses: Requirement 6 AC 1–2
#[derive(Debug, Clone)]
pub struct ChangeResult {
    /// Number of replacements made.
    pub replacement_count: u64,
    /// The position after the last replacement (for cursor placement).
    pub final_position: BytePosition,
    /// The line of the last replacement.
    pub final_line: LineNumber,
}

/// The outcome of a find/change operation.
///
/// Addresses: Requirement 1 AC 7, Requirement 20
#[derive(Debug, Clone)]
pub enum FindOutcome {
    /// A single match was found.
    Found(FindResult),
    /// Multiple matches found (for ALL direction).
    FoundAll {
        count: u64,
        first: FindResult,
    },
    /// No match found.
    NotFound {
        /// The search term that was not found (for error message).
        term: String,
    },
}

/// The outcome of a change operation.
#[derive(Debug, Clone)]
pub enum ChangeOutcome {
    /// Replacements were made.
    Changed(ChangeResult),
    /// No match found to replace.
    NotFound { term: String },
    /// Document is read-only.
    ReadOnly,
}
```

### CaptureGroup

```rust
/// A numbered capture group from a regex match.
///
/// Addresses: Requirement 4 AC 9, Requirement 8 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureGroup {
    /// Group index (0 = entire match, 1–9 = sub-expressions).
    pub index: u8,
    /// Byte range of the captured text.
    pub range: MatchRange,
}
```

### FindState

```rust
/// Per-document session state for RFIND/RCHANGE repetition.
///
/// Addresses: Requirements 5, 9, 13
#[derive(Debug, Clone)]
pub struct FindState {
    /// The most recent FindRequest (for RFIND).
    pub last_find: Option<FindRequest>,
    /// The most recent ChangeRequest (for RCHANGE).
    pub last_change: Option<ChangeRequest>,
    /// Position of the last match (for advancing RFIND/RCHANGE).
    pub last_match_position: Option<BytePosition>,
    /// Ring buffer of recent search terms (configurable size, default 20).
    pub search_history: VecDeque<String>,
    /// Ring buffer of recent replacement texts.
    pub replacement_history: VecDeque<String>,
    /// Maximum history size.
    pub history_capacity: usize,
}

impl FindState {
    pub fn new(history_capacity: usize) -> Self;

    /// Record a new find request as the last search.
    pub fn record_find(&mut self, request: &FindRequest, match_pos: BytePosition);

    /// Record a new change request.
    pub fn record_change(&mut self, request: &ChangeRequest, final_pos: BytePosition);

    /// Clear highlights and incremental state (RESET without ALL).
    /// Addresses: Requirement 13 AC 4
    pub fn reset(&mut self);

    /// Clear last-search/change params (RESET ALL).
    /// Addresses: Requirement 13 AC 5
    pub fn reset_all(&mut self);

    /// Serialise for session persistence.
    /// Addresses: Requirement 13 AC 7
    pub fn serialize(&self) -> Vec<u8>;

    /// Deserialise from session data.
    pub fn deserialize(data: &[u8]) -> Result<Self, FindReplaceError>;
}
```

### RegexEngine Types

```rust
/// Compiled NFA bytecode ready for execution.
///
/// Addresses: Requirement 12 AC 1
pub struct CompiledRegex {
    /// NFA bytecode instructions.
    bytecode: Vec<NfaInstruction>,
    /// Number of capture groups defined.
    group_count: u8,
    /// Optional literal prefix for fast-path scanning.
    literal_prefix: Option<Vec<u8>>,
    /// Maximum compiled size limit.
    max_size: usize,
}

/// NFA instruction set (internal representation).
pub(crate) enum NfaInstruction {
    Literal(u8),
    CharClass(CharClassId),
    AnyChar,
    Anchor(AnchorKind),
    Split(usize, usize),        // greedy: try first, then second
    SplitLazy(usize, usize),    // lazy: try second first
    Jump(usize),
    GroupStart(u8),
    GroupEnd(u8),
    BackRef(u8),
    Match,
}

/// Anchor types for ^ $ \b etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnchorKind {
    LineStart,          // ^
    LineEnd,            // $
    WordBoundary,       // \b
    WordStart,          // \<
    WordEnd,            // \>
}
```

### SubstitutionTemplate

```rust
/// A parsed replacement template with group references.
///
/// Addresses: Requirement 8 AC 2–4
#[derive(Debug, Clone)]
pub struct SubstitutionTemplate {
    /// Parsed segments of the replacement string.
    segments: Vec<TemplateSegment>,
}

/// A segment within a substitution template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TemplateSegment {
    /// Literal text to insert as-is.
    Literal(String),
    /// Group reference (\0–\9 or $0–$9).
    GroupRef(u8),
}

impl SubstitutionTemplate {
    /// Parse a replacement string into a template.
    /// Addresses: Requirement 8 AC 2–3
    pub fn parse(text: &str) -> Result<Self, FindReplaceError>;

    /// Expand the template against captured groups, producing replacement text.
    /// Addresses: Requirement 8 AC 8
    pub fn expand(&self, captures: &[MatchRange], source: &dyn CharacterIndexer) -> String;
}
```

### Events

```rust
/// Events emitted by the FindEngine for plugins and UI.
///
/// Addresses: Requirement 17 AC 7
#[derive(Debug, Clone)]
pub enum FindEvent {
    /// A find operation has started.
    FindStarted {
        term: String,
        mode: SearchMode,
    },
    /// A match was found.
    MatchFound {
        result: FindResult,
    },
    /// A find operation completed.
    FindCompleted {
        term: String,
        total_matches: u64,
        elapsed_ms: u64,
    },
    /// A replace operation completed.
    ReplaceCompleted {
        term: String,
        replacement_count: u64,
        elapsed_ms: u64,
    },
    /// Progress update during long operations.
    Progress {
        matches_so_far: u64,
        lines_scanned: u64,
    },
}

/// Trait for receiving find/replace events.
pub trait FindEventListener: Send + Sync {
    fn on_event(&self, event: &FindEvent);
}
```

---

## Public API Surface

### CharacterIndexer Trait

```rust
/// Abstract byte-level access to the document buffer.
/// Implemented by ff-document-model over its GapBuffer/SplitView.
/// Enables the FindEngine to search without depending on a specific buffer type.
///
/// Addresses: Requirement 18
pub trait CharacterIndexer: Send + Sync {
    /// Read a single byte at the given position.
    /// Addresses: Requirement 18 AC 1
    fn char_at(&self, position: BytePosition) -> Option<u8>;

    /// Read a contiguous slice of bytes. Returns None if range is invalid.
    /// For non-contiguous buffers (gap buffer), may copy into internal buffer.
    /// Addresses: Requirement 18 AC 2
    fn slice(&self, start: BytePosition, end: BytePosition) -> Option<Vec<u8>>;

    /// Align a byte position to the nearest UTF-8 character boundary.
    /// Addresses: Requirement 18 AC 3
    fn move_position_outside_char(
        &self,
        position: BytePosition,
        direction: Direction,
    ) -> BytePosition;

    /// Get the byte range [start, end) of a given line.
    /// Addresses: Requirement 18 AC 5
    fn line_range(&self, line: LineNumber) -> Option<(BytePosition, BytePosition)>;

    /// Total byte length of the document.
    fn length(&self) -> u64;

    /// Total line count.
    fn line_count(&self) -> u64;

    /// Determine which line a byte position belongs to.
    fn line_from_position(&self, position: BytePosition) -> LineNumber;
}
```

### FindEngine — Construction and Configuration

```rust
impl FindEngine {
    /// Create a new FindEngine with default configuration.
    pub fn new() -> Self;

    /// Create with custom configuration.
    pub fn with_config(config: FindEngineConfig) -> Self;

    /// Register an event listener.
    /// Addresses: Requirement 17 AC 7
    pub fn add_listener(&mut self, listener: Box<dyn FindEventListener>);

    /// Remove a previously registered listener.
    pub fn remove_listener(&mut self, id: ListenerId);
}

/// Configuration for the FindEngine.
#[derive(Debug, Clone)]
pub struct FindEngineConfig {
    /// Whether BOUNDS affect FIND (default: true).
    /// Addresses: Requirement 2 AC 5
    pub bounds_affect_find: bool,
    /// Maximum matches for highlight-all (default: 1000).
    /// Addresses: Requirement 15 AC 6
    pub highlight_all_max: u64,
    /// Incremental search time budget in ms (default: 50).
    /// Addresses: Requirement 14 AC 1
    pub incremental_time_budget_ms: u64,
    /// Regex match-attempt limit per position (default: 10_000).
    /// Addresses: Requirement 19 AC 4
    pub regex_step_limit: u64,
    /// Search history capacity (default: 20).
    /// Addresses: Requirement 13 AC 2
    pub history_capacity: usize,
    /// Progress report interval (matches between events).
    /// Addresses: Requirement 19 AC 2
    pub progress_interval: u64,
}

impl Default for FindEngineConfig {
    fn default() -> Self {
        Self {
            bounds_affect_find: true,
            highlight_all_max: 1000,
            incremental_time_budget_ms: 50,
            regex_step_limit: 10_000,
            history_capacity: 20,
            progress_interval: 100,
        }
    }
}
```

### FindEngine — Core Search Operations

```rust
impl FindEngine {
    /// Execute a FIND operation.
    /// Addresses: Requirements 1–4
    pub fn find(
        &mut self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<FindOutcome, FindReplaceError>;

    /// Execute an RFIND (repeat previous find).
    /// Addresses: Requirement 5
    pub fn rfind(
        &mut self,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<FindOutcome, FindReplaceError>;

    /// Execute a CHANGE operation.
    /// Addresses: Requirements 6–8
    pub fn change(
        &mut self,
        request: &ChangeRequest,
        indexer: &mut dyn CharacterIndexerMut,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<ChangeOutcome, FindReplaceError>;

    /// Execute an RCHANGE (repeat previous change).
    /// Addresses: Requirement 9
    pub fn rchange(
        &mut self,
        indexer: &mut dyn CharacterIndexerMut,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<ChangeOutcome, FindReplaceError>;

    /// Execute a find for EXCLUDE/SHOW delegation (does NOT update FindState).
    /// Addresses: Requirement 16 AC 1–4
    pub fn find_for_filter(
        &self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<FindOutcome, FindReplaceError>;

    /// Get the current FindState (for serialisation / UI display).
    /// Addresses: Requirement 13
    pub fn state(&self) -> &FindState;

    /// Get mutable access to FindState (for RESET operations).
    pub fn state_mut(&mut self) -> &mut FindState;
}
```

### FindEngine — Incremental Search

```rust
impl FindEngine {
    /// Start or update an incremental search.
    /// Cancels any in-progress search and begins a new one with the partial term.
    /// Addresses: Requirement 14 AC 1–2, AC 5–8
    pub fn incremental_search(
        &mut self,
        partial_term: &str,
        start_position: BytePosition,
        mode: SearchMode,
        case_sensitive: bool,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
    ) -> Result<Option<FindResult>, FindReplaceError>;

    /// Clear incremental search state and highlights.
    /// Addresses: Requirement 14 AC 7
    pub fn clear_incremental(&mut self);
}
```

### FindEngine — Highlight All Matches

```rust
impl FindEngine {
    /// Compute all matches within a viewport range for highlight-all mode.
    /// Addresses: Requirement 15
    pub fn highlight_all(
        &self,
        term: &str,
        mode: SearchMode,
        case_sensitive: bool,
        viewport_start: BytePosition,
        viewport_end: BytePosition,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
    ) -> Result<HighlightAllResult, FindReplaceError>;

    /// Clear all highlight-all decorations.
    /// Addresses: Requirement 15 AC 5
    pub fn clear_highlights(&mut self);
}

/// Result of a highlight-all computation.
///
/// Addresses: Requirement 15 AC 1, AC 6–7
#[derive(Debug, Clone)]
pub struct HighlightAllResult {
    /// Matches found within the viewport.
    pub matches: Vec<MatchRange>,
    /// Whether the total match count exceeds the configured maximum.
    pub truncated: bool,
    /// Total match count (may exceed matches.len() if truncated).
    pub total_count: u64,
}
```

### ScopeFilterProvider Trait

```rust
/// Trait for querying line visibility and tag state.
/// Implemented by the exclude-show-filter or display-line-mapping layer.
///
/// Addresses: Requirement 2 AC 1–4
pub trait ScopeFilterProvider: Send + Sync {
    /// Whether the line is visible (not excluded).
    fn is_visible(&self, line: LineNumber) -> bool;

    /// Whether the line is excluded (hidden).
    fn is_excluded(&self, line: LineNumber) -> bool;

    /// Whether the line is tagged.
    fn is_tagged(&self, line: LineNumber) -> bool;
}
```

### CharacterIndexerMut Trait

```rust
/// Mutable extension of CharacterIndexer for CHANGE operations.
/// Provides document mutation primitives used by the replacement engine.
///
/// Addresses: Requirement 6 (replacement requires mutation)
pub trait CharacterIndexerMut: CharacterIndexer {
    /// Replace bytes in range [start, end) with new_bytes.
    /// Returns the length delta (new_len - old_len).
    fn replace_range(
        &mut self,
        start: BytePosition,
        end: BytePosition,
        new_bytes: &[u8],
    ) -> Result<i64, FindReplaceError>;

    /// Check if the document is read-only.
    fn is_read_only(&self) -> bool;
}
```

### CaseFolder

```rust
/// Unicode Full Case Folding for case-insensitive comparison.
/// Stateless and thread-safe. Implements CaseFolding.txt status C + F mappings.
///
/// Addresses: Requirement 10
pub struct CaseFolder {
    /// Optional locale hint for locale-sensitive rules.
    locale: Option<LocaleHint>,
}

/// Locale hint for case-sensitive folding adjustments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleHint {
    /// Turkish/Azerbaijani: special İ/I/ı/i rules.
    Turkish,
    /// Lithuanian: special dot-above handling.
    Lithuanian,
}

impl CaseFolder {
    /// Create a locale-independent case folder.
    pub fn new() -> Self;

    /// Create with a locale hint.
    /// Addresses: Requirement 10 AC 8
    pub fn with_locale(locale: LocaleHint) -> Self;

    /// Fold a single character, returning one or more folded characters.
    /// Addresses: Requirement 10 AC 1, AC 3
    pub fn fold_char(&self, ch: char) -> SmallVec<[char; 3]>;

    /// Fold an entire string, producing the folded output.
    /// Handles one-to-many mappings (e.g., ß → ss).
    /// Addresses: Requirement 10 AC 3–4
    pub fn fold_str(&self, input: &str) -> String;

    /// Fold a byte slice assumed to be valid UTF-8.
    /// Returns folded bytes. Used for pre-folding search terms.
    /// Addresses: Requirement 10 AC 6
    pub fn fold_bytes(&self, input: &[u8]) -> Vec<u8>;

    /// Compare two strings for equality under case folding.
    /// Addresses: Requirement 10 AC 2
    pub fn eq_folded(&self, a: &str, b: &str) -> bool;
}
```

### RegexEngine

```rust
/// NFA-based regular expression engine with group capture.
///
/// Addresses: Requirements 4, 12
pub struct RegexEngine {
    /// The most recently compiled pattern (for reuse on empty-pattern submission).
    last_compiled: Option<CompiledRegex>,
    /// Maximum compiled NFA size in instructions.
    max_nfa_size: usize,
    /// Step limit per position to prevent catastrophic behaviour.
    step_limit: u64,
}

impl RegexEngine {
    /// Create with default limits.
    pub fn new() -> Self;

    /// Create with custom limits.
    pub fn with_limits(max_nfa_size: usize, step_limit: u64) -> Self;

    /// Compile a regex pattern into NFA bytecode.
    /// Addresses: Requirement 12 AC 1–9
    pub fn compile(&mut self, pattern: &str) -> Result<&CompiledRegex, FindReplaceError>;

    /// Execute the compiled regex against a character indexer within a byte range.
    /// Returns the first match found (or None).
    /// Addresses: Requirement 12 AC 10–13
    pub fn execute(
        &self,
        compiled: &CompiledRegex,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        case_folder: Option<&CaseFolder>,
    ) -> Option<FindResult>;

    /// Execute in reverse (backward search).
    pub fn execute_reverse(
        &self,
        compiled: &CompiledRegex,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        case_folder: Option<&CaseFolder>,
    ) -> Option<FindResult>;

    /// Find all non-overlapping matches within a range.
    pub fn find_all(
        &self,
        compiled: &CompiledRegex,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        case_folder: Option<&CaseFolder>,
        cancel: &CancellationToken,
    ) -> Vec<FindResult>;
}
```

### SubstitutionEngine

```rust
/// Expands replacement templates against regex capture groups.
///
/// Addresses: Requirement 8
pub struct SubstitutionEngine;

impl SubstitutionEngine {
    /// Parse a replacement string into a SubstitutionTemplate.
    /// Addresses: Requirement 8 AC 2–3
    pub fn parse_template(replacement: &str) -> Result<SubstitutionTemplate, FindReplaceError>;

    /// Expand a template using captured groups from a match.
    /// Addresses: Requirement 8 AC 8
    pub fn substitute(
        template: &SubstitutionTemplate,
        captures: &[MatchRange],
        indexer: &dyn CharacterIndexer,
    ) -> String;
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-find-and-replace crate.
/// Formatted per Error Message Standards (Req 8): `[find-replace] operation: description`
///
/// Addresses: Cross-cutting Requirement 8, Requirements 3, 4, 5, 9, 12, 20
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FindReplaceError {
    /// No search term specified and no previous search to reuse.
    /// Addresses: Requirement 20 AC 1
    #[error("[find-replace] find: no search term specified")]
    NoSearchTerm,

    /// No previous FIND to repeat (RFIND with no history).
    /// Addresses: Requirement 5 AC 2
    #[error("[find-replace] rfind: no previous FIND to repeat")]
    NoPreviousFind,

    /// No previous CHANGE to repeat (RCHANGE with no history).
    /// Addresses: Requirement 9 AC 2
    #[error("[find-replace] rchange: no previous CHANGE to repeat")]
    NoPreviousChange,

    /// Document is read-only; CHANGE not permitted.
    /// Addresses: Requirement 20 AC 4
    #[error("[find-replace] change: document is read-only")]
    DocumentReadOnly,

    /// Invalid hex pattern: odd number of digits.
    /// Addresses: Requirement 3 AC 2
    #[error("[find-replace] find: invalid hex pattern: odd number of digits")]
    HexOddDigits,

    /// Invalid hex pattern: non-hex character encountered.
    /// Addresses: Requirement 3 AC 3
    #[error("[find-replace] find: invalid hex pattern: non-hex character '{0}'")]
    HexInvalidChar(char),

    /// Regex compilation error.
    /// Addresses: Requirement 12 AC 2–9
    #[error("[find-replace] regex: {message}")]
    RegexCompile { message: String },

    /// Regex pattern too long (NFA exceeds max size).
    /// Addresses: Requirement 12 AC 2
    #[error("[find-replace] regex: pattern too long")]
    RegexPatternTooLong,

    /// Invalid substitution template escape.
    /// Addresses: Requirement 20 AC 5
    #[error("[find-replace] replace: invalid escape in replacement template: {detail}")]
    InvalidSubstitution { detail: String },

    /// Search was cancelled via cancellation token.
    /// Addresses: Requirement 19 AC 1
    #[error("[find-replace] find: operation cancelled")]
    Cancelled,

    /// Internal error from document access.
    #[error("[find-replace] {operation}: document error: {detail}")]
    DocumentAccess { operation: String, detail: String },

    /// Serialisation/deserialisation error for FindState.
    #[error("[find-replace] state: serialisation error: {0}")]
    Serialization(String),
}
```

---

## Integration Points

### Upstream Crate Dependencies

| Crate | Usage |
|-------|-------|
| **ff-document-model** | `CharacterIndexer` implementation over `Document`/`TextBuffer`/`SplitView`; `BytePosition`, `LineNumber` types; `Document.is_read_only()` checks |
| **ff-command** | Command registration (`find`, `rfind`, `change`, `rchange`, `find_next`, `find_prev`, `find_all`, `replace_all`); `CommandHandler` trait implementation; `CommandMetadata` with category "Search" |
| **ff-display-line-mapping** | `DisplayLineMapping` trait for line visibility queries (via `ScopeFilterProvider` adapter); document→display line conversion for viewport scrolling on match |
| **ff-undo-redo-transactions** | Transaction wrapping for all CHANGE operations; `UndoRecord` production for single and ALL replacements |
| **ff-logging** | Diagnostic logging for regex step-limit warnings, search performance metrics |
| **ff-exclude-show-filter** | Provides `ScopeFilterProvider` implementation exposing per-line `tagged`, `excluded`, `visible` state |

### Downstream Consumers

| Crate | Integration |
|-------|-------------|
| **ff-command-semantics** | Parses FIND/CHANGE command text and constructs `FindRequest`/`ChangeRequest` structs |
| **ff-text-decorations** | Subscribes to `FindEvent::MatchFound` and `highlight_all` results to render match indicators |
| **ff-exclude-show-filter** | Calls `find_for_filter()` to delegate EXCLUDE/SHOW text-matching to the find engine |
| **ff-lua-macro-engine** | Invokes find/change commands via the scripting bridge with programmatic `CommandParams` |
| **ff-startup-and-session** | Serialises/deserialises `FindState` for session persistence across restarts |

### Command Registration

The crate registers these commands with `ff-command` at initialisation:

| Command ID | Display Name | Default Shortcut | Undoable |
|-----------|-------------|-----------------|----------|
| `search.find` | Find | Ctrl+F (focus find) | No |
| `search.rfind` | Repeat Find | F3 (suggested) | No |
| `search.find_next` | Find Next | — | No |
| `search.find_prev` | Find Previous | Shift+F3 (suggested) | No |
| `search.find_all` | Find All | — | No |
| `search.change` | Change | Ctrl+H (focus change) | Yes |
| `search.rchange` | Repeat Change | — | Yes |
| `search.replace_all` | Replace All | — | Yes |

---

## Correctness Properties

These properties define invariants that must hold across all valid inputs. Each property maps to one or more acceptance criteria and is suitable for implementation using the `proptest` crate.

### Property 1: Literal Find Idempotence

**Statement:** For any document content and literal search term, executing FIND FIRST followed by RFIND from position 0 yields the same match range as executing FIND NEXT from position 0.

**Validates: Requirements 1.1, 1.2, 5.1**

**Strategy:** Generate arbitrary UTF-8 strings (1–500 bytes) for documents, pick random sub-strings (1–20 chars) as search terms, exercise FIRST then RFIND.

---

### Property 2: FIND ALL Count Consistency

**Statement:** For any document and search term, `FIND ALL` returns a count equal to the number of non-overlapping forward matches found by iterating `FIND NEXT` from position 0 until no more matches are found.

**Validates: Requirements 1.6**

**Strategy:** Generate documents with repeated patterns, verify count agreement between ALL and iterative NEXT.

---

### Property 3: Case Folding Symmetry

**Statement:** For any two strings A and B, `case_folder.eq_folded(A, B)` implies `case_folder.eq_folded(B, A)`. Case folding comparison is symmetric.

**Validates: Requirements 10.2**

**Strategy:** Generate pairs of Unicode strings (including non-ASCII: Cyrillic, Greek, Turkish), verify symmetry.

---

### Property 4: Case Folding Stability

**Statement:** For any string S, `fold_str(fold_str(S)) == fold_str(S)`. Folding is idempotent.

**Validates: Requirements 10.1**

**Strategy:** Generate arbitrary Unicode strings, fold twice, compare to single fold.

---

### Property 5: Column Range Intersection Commutativity

**Statement:** For any two ColumnRanges A and B, `A.intersect(B) == B.intersect(A)`.

**Validates: Requirements 7.6**

**Strategy:** Generate random ColumnRange pairs with start ∈ [1, 200], end ∈ [start, 200].

---

### Property 6: CHANGE ALL Preserves Non-Matching Content

**Statement:** For any document D and CHANGE operation replacing `old` with `new`, after CHANGE ALL, every byte in the result that was not part of a match in D remains unchanged at its (adjusted) position.

**Validates: Requirements 6.8**

**Strategy:** Generate documents with known patterns interspersed with marker bytes; verify markers survive unchanged.

---

### Property 7: Regex Group Capture Containment

**Statement:** For any regex match with N captured groups, each group's byte range is contained within group 0's range (the entire match). `group[i].start >= group[0].start && group[i].end <= group[0].end` for all i in 1..N.

**Validates: Requirements 4.9**

**Strategy:** Generate simple regex patterns with 1–3 groups (e.g., `(a+)(b+)`), run against generated matching strings, verify containment.

---

### Property 8: Substitution Template Roundtrip

**Statement:** For any replacement string with no group references, `SubstitutionTemplate::parse(s).expand([], _)` returns the original string unchanged.

**Validates: Requirements 8.2**

**Strategy:** Generate arbitrary strings that do not contain `\` or `$` followed by digits; verify identity expansion.

---

### Property 9: Hex Pattern Byte Equivalence

**Statement:** For any byte sequence B, searching with hex pattern `X'<hex_encode(B)>'` finds matches at exactly the same positions as searching with the raw bytes in literal mode (case-sensitive).

**Validates: Requirements 3.1, 3.7**

**Strategy:** Generate random byte sequences (1–20 bytes), embed them in larger random documents, compare literal byte-search results with hex-search results.

---

### Property 10: RFIND Direction Normalisation

**Statement:** After a FIND with direction FIRST, subsequent RFIND uses direction NEXT. After a FIND with direction LAST, subsequent RFIND uses direction PREV. The stored direction in FindState is always normalised.

**Validates: Requirements 5.4, 5.5**

**Strategy:** Generate FindRequests with all four directions, execute find, then check FindState.last_find.direction.

---

### Property 11: Word Boundary Consistency

**Statement:** For any match found with `WordMatchMode::WholeWord`, the character immediately before the match start (if any) is NOT a word character, and the character immediately after the match end (if any) is NOT a word character.

**Validates: Requirements 11.1**

**Strategy:** Generate documents with word and non-word characters, search with whole-word mode, verify boundary characters.

---

### Property 12: Scope Filter Exclusivity

**Statement:** For any document with mixed visible/excluded/tagged lines, FIND VISIBLE and FIND EXCLUDED never return matches on the same line. FIND TAGGED and FIND NONTAGGED never return matches on the same line.

**Validates: Requirements 2.1, 2.2, 2.3, 2.4**

**Strategy:** Generate documents with randomised per-line flags, run both scope variants, verify disjoint line sets.

---

### Property 13: CHANGE ALL Transaction Atomicity

**Statement:** After a successful CHANGE ALL producing N replacements, a single UNDO operation restores the document to its exact pre-change state (byte-for-byte equality).

**Validates: Requirements 7.7, 17.4**

**Strategy:** Generate documents with multiple occurrences, execute CHANGE ALL, record state, UNDO, compare byte-for-byte.

---

### Property 14: Incremental Search Position Reset

**Statement:** When the search field text is shortened (characters deleted), the incremental search re-starts from the original start position, not from the current match position. The match found for a prefix P is always the first match at or after the original start.

**Validates: Requirements 14.5**

**Strategy:** Generate documents, perform incremental search with progressively longer terms, then shorten; verify match positions against a fresh search from original start.

---

### Property 15: Zero-Length Regex Match Termination

**Statement:** When a regex that can match zero length (e.g., `a*`) is used with CHANGE ALL, the operation terminates in at most `document.length() + 1` iterations (one per character position plus end).

**Validates: Requirements 8.7**

**Strategy:** Generate short documents (1–100 bytes), use zero-length-capable patterns, verify CHANGE ALL terminates and returns bounded replacement count.

---

### Property 16: Highlight All Viewport Containment

**Statement:** Every match returned by `highlight_all()` has `match_range.start >= viewport_start` and `match_range.end <= viewport_end`.

**Validates: Requirements 15.1**

**Strategy:** Generate documents, pick random viewport ranges, verify all returned matches are within bounds.

---

## Testing Strategy

### Unit Tests

- **Literal search**: Verify forward/backward/first/last directions with known documents and expected match positions
- **Hex search**: Verify byte pattern parsing, odd-digit rejection, non-hex rejection, and matching against embedded byte sequences
- **Regex compilation**: Verify error messages for invalid patterns (unmatched parens, empty closures, cyclical refs)
- **Regex execution**: Verify NFA matching for all metacharacters, character classes, anchors, and quantifiers (greedy/lazy)
- **Case folding**: Verify German ß→ss, Turkish İ→i, Greek sigma variants, and ASCII a–z folding
- **Substitution**: Verify template parsing, group expansion, unmatched group handling, and escape sequences
- **Word boundary**: Verify WholeWord and WordStart modes with various character transitions
- **Scope filtering**: Verify TAGGED/EXCLUDED/VISIBLE/NONTAGGED scope filters with mock line state
- **Column range**: Verify column clipping, bounds intersection, and out-of-range handling
- **FindState**: Verify RFIND/RCHANGE state recording, history ring behaviour, reset/reset-all semantics
- **Incremental search**: Verify debounce, cancellation, position-reset on backspace
- **Highlight-all**: Verify viewport containment, truncation at threshold, decoration clearing

### Property-Based Tests (proptest)

All 16 properties listed in the Correctness Properties section are implemented as proptest tests with ≥100 cases. Strategies generate:

- Arbitrary UTF-8 documents (1–5000 bytes)
- Random sub-strings as search terms (from the document itself for guaranteed matches)
- Random column ranges and bounds
- Mixed per-line visibility/tag flags
- Simple regex patterns (concatenation, alternation, repetition, 1–3 groups)

### Integration Tests

- End-to-end FIND/CHANGE flow with a mock `CharacterIndexer` and `ScopeFilterProvider`
- Command registration and dispatch through `ff-command` mock registry
- CHANGE ALL with undo verification (byte-for-byte restoration)
- Session state serialisation roundtrip

### Performance Tests

- Literal search on 1 MB document completes in < 50ms (benchmark, not CI gate)
- Regex search with step-limit prevents hang on pathological patterns
- FIND ALL on 100K-line document with progress events
