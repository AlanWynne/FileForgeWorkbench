# Design Document: Exclude/Show Filter (`ff-exclude-show-filter`)

## Overview

The `ff-exclude-show-filter` crate is the **line visibility management engine** for FileForgeWorkbench. It implements ISPF-style EXCLUDE/SHOW/RESET primary commands and X/Xn/XX line commands, providing a logical layer that drives per-line visibility state through the `display-line-mapping` subsystem without modifying document content.

### Purpose

- Manage per-line exclusion state (hidden/visible) as transient session state
- Execute EXCLUDE commands: hide lines by text match, regex, range, ALL, TAGGED
- Execute SHOW/INCLUDE commands: reveal excluded lines by text match, regex, ALL
- Execute RESET commands: clear exclusion state (all lines visible)
- Process X/Xn/XX line commands for per-line and block exclusion
- Provide exclusion block enumeration for placeholder rendering
- Expose scope iterators (visible/excluded) for find-and-replace integration
- Emit change notifications for viewport and scrollbar synchronization

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
├─────────────────────────────────────────────────────────────┤
│  Downstream: ff-viewport-and-scrolling (placeholder render), │
│    ff-navigation-commands (visibility-aware scroll)           │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-exclude-show-filter ← Wave 5                 │
├─────────────────────────────────────────────────────────────┤
│  Upstream: ff-display-line-mapping (visibility storage),      │
│            ff-document-model (line content access),           │
│            ff-find-and-replace (text-match delegation),       │
│            ff-command-semantics (scope resolver, session),    │
│            ff-command (command registration)                  │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: No direct filesystem access — document content accessed via `ff-document-model`
- **GUI Independence (Req 2)**: Zero GUI dependencies — no egui, winit, wgpu; placeholder rendering is a downstream concern
- **Command-Driven (Req 4)**: EXCLUDE/SHOW/RESET registered as commands in `ff-command`; explicitly non-undoable (session state only)
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-exclude-show-filter`
- **Error Message Standards (Req 8)**: All errors follow `[exclude-show] operation: description` format

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Consumers [Consuming Crates]
        VP[ff-viewport-and-scrolling<br/>placeholder rendering]
        NAV[ff-navigation-commands<br/>visibility-aware locate]
        FAR[ff-find-and-replace<br/>ScopeFilterProvider impl]
    end

    subgraph ff-exclude-show-filter [ff-exclude-show-filter Crate]
        ENG[ExclusionEngine<br/>top-level orchestrator]
        CMD[CommandHandlers<br/>EXCLUDE/SHOW/RESET handlers]
        LC[LineCommandHandler<br/>X/Xn/XX processing]
        MATCH[TextMatcher<br/>literal + regex delegation]
        BLK[BlockEnumerator<br/>contiguous block tracking]
        ITER[ScopeIterators<br/>visible/excluded line iterators]
        NOTIFY[ChangeNotifier<br/>exclusion-change events]
        REG[Registration<br/>command framework integration]
    end

    subgraph Upstream [Upstream Crates]
        DLM[ff-display-line-mapping<br/>set_visible / get_visible / show_all]
        DOC[ff-document-model<br/>line content access]
        FAR_ENG[ff-find-and-replace<br/>find_for_filter delegation]
        CS[ff-command-semantics<br/>scope resolver, session state]
        CMD_FW[ff-command<br/>registry + dispatch]
        LOG[ff-logging]
    end

    VP -->|exclusion_blocks / placeholder_text| ENG
    NAV -->|is_excluded / visible_lines_iter| ENG
    FAR -->|ScopeFilterProvider| ENG

    ENG --> CMD
    ENG --> LC
    ENG --> MATCH
    ENG --> BLK
    ENG --> ITER
    ENG --> NOTIFY
    ENG --> REG

    MATCH -->|find_for_filter| FAR_ENG
    CMD -->|set_visible / show_all| DLM
    LC -->|set_visible| DLM
    ENG -->|get_visible / hidden_lines| DLM
    MATCH -->|line_content| DOC
    REG -->|register commands| CMD_FW
    ENG -->|scope resolution| CS
    ENG --> LOG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **ExclusionEngine** | Top-level orchestrator: owns references to display-line-mapping and document-model, coordinates all exclusion operations, provides the public API surface |
| **CommandHandlers** | Implementation of EXCLUDE/SHOW/RESET primary command logic; translates parsed arguments into visibility mutations |
| **LineCommandHandler** | Processes X/Xn/XX line commands received from the line-command subsystem; translates into `set_visible` calls |
| **TextMatcher** | Adapts text-matching requests to the find-and-replace engine's `find_for_filter` API for literal and regex searches |
| **BlockEnumerator** | Efficiently enumerates contiguous exclusion blocks for placeholder rendering; maintains O(k) enumeration where k = block count |
| **ScopeIterators** | Provides `visible_lines_iter()` and `excluded_lines_iter()` for scoped find/change operations |
| **ChangeNotifier** | Emits `ExclusionChanged` events when visibility state changes, consumed by viewport/scrollbar |
| **Registration** | Registers EXCLUDE, SHOW, RESET commands (and aliases X, INCLUDE) with the command framework |

### Data Flow: EXCLUDE 'text' Command

```
1. User enters "EXCLUDE 'pattern'" on the command line
2. ff-command-semantics parses and dispatches to ExclusionEngine
3. ExclusionEngine resolves scope (default: all visible lines)
4. TextMatcher calls find_for_filter on each line in scope
5. For each matching line: set_visible(line, line, false) via display-line-mapping
6. BlockEnumerator updates block boundaries
7. ChangeNotifier fires ExclusionChanged event
8. Status message: "N line(s) excluded"
```

### Data Flow: SHOW 'text' Command

```
1. User enters "SHOW 'pattern'" on the command line
2. ff-command-semantics parses and dispatches to ExclusionEngine
3. ExclusionEngine resolves scope (default: all excluded lines)
4. TextMatcher calls find_for_filter on each excluded line in scope
5. For each matching line: set_visible(line, line, true) via display-line-mapping
6. BlockEnumerator updates block boundaries
7. ChangeNotifier fires ExclusionChanged event
8. Status message: "N line(s) shown"
```

### Data Flow: RESET EXCLUDED Command

```
1. User enters "RESET EXCLUDED" on the command line
2. ff-command-semantics parses and dispatches to ExclusionEngine
3. ExclusionEngine calls display_line_mapping.show_all()
4. BlockEnumerator clears all blocks (count = 0)
5. ChangeNotifier fires ExclusionChanged event
6. Status message: "RESET: N line(s) restored to view"
```

---

## Components and Interfaces

### Module Structure

```
crates/ff-exclude-show-filter/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── engine.rs                   # ExclusionEngine: top-level orchestrator
│   ├── commands/
│   │   ├── mod.rs                  # Command handler re-exports
│   │   ├── exclude.rs              # EXCLUDE command handler (text, regex, ALL, TAGGED, range)
│   │   ├── show.rs                 # SHOW/INCLUDE command handler (text, regex, ALL, EXCLUDED)
│   │   └── reset.rs               # RESET command handler (no-arg, EXCLUDED, ALL)
│   ├── line_commands.rs            # X/Xn/XX line command processing
│   ├── matcher.rs                  # TextMatcher: literal + regex delegation to find engine
│   ├── blocks.rs                   # BlockEnumerator: contiguous block tracking, placeholder text
│   ├── iterators.rs               # ScopeIterators: visible_lines_iter, excluded_lines_iter
│   ├── notifier.rs                # ChangeNotifier: exclusion-change event dispatch
│   ├── scope_provider.rs          # ScopeFilterProvider implementation for find-and-replace
│   ├── registration.rs            # Command registration with ff-command framework
│   ├── types.rs                   # ExclusionBlock, ExcludeArgs, ShowArgs, ResetVariant
│   └── error.rs                   # ExcludeShowError enum
└── tests/
    ├── exclude_tests.rs            # EXCLUDE command unit + property tests
    ├── show_tests.rs               # SHOW/INCLUDE command tests
    ├── reset_tests.rs              # RESET command tests
    ├── line_command_tests.rs       # X/Xn/XX line command tests
    ├── block_tests.rs              # Block enumeration and merging tests
    ├── iterator_tests.rs           # Scope iterator tests
    ├── integration.rs              # End-to-end with mock document and display-line-mapping
    └── property_tests.rs           # Cross-cutting proptest properties
```

---

## Data Models

### Core Newtypes and Enums

```rust
/// Re-export from ff-display-line-mapping for convenience.
pub use ff_display_line_mapping::DocLine;

/// Arguments parsed from an EXCLUDE primary command.
///
/// Addresses: Requirement 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcludeArgs {
    /// EXCLUDE 'text' — literal text match on visible lines.
    Text {
        pattern: String,
        scope: ExcludeScope,
    },
    /// EXCLUDE REGEX 'pattern' — regex match on visible lines.
    Regex {
        pattern: String,
        scope: ExcludeScope,
    },
    /// EXCLUDE ALL — exclude every line in the document.
    All,
    /// EXCLUDE TAGGED — exclude lines with tag flag set.
    Tagged,
    /// EXCLUDE n m — exclude a specific line range (1-based inclusive).
    Range {
        start_line: u64,
        end_line: u64,
    },
}

/// Scope modifier for EXCLUDE text/regex operations.
///
/// Addresses: Requirement 2 AC 1–2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExcludeScope {
    /// Search only visible lines (default for EXCLUDE without ALL).
    #[default]
    Visible,
    /// Search all lines regardless of current visibility (EXCLUDE 'text' ALL).
    All,
}

/// Arguments parsed from a SHOW/INCLUDE primary command.
///
/// Addresses: Requirement 3
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShowArgs {
    /// SHOW ALL — make all lines visible.
    All,
    /// SHOW EXCLUDED — make all excluded lines visible (same as ALL effectively).
    Excluded,
    /// SHOW NONEXCLUDED — no-op, confirms current state.
    NonExcluded,
    /// SHOW 'text' — show excluded lines matching literal text.
    Text { pattern: String },
    /// SHOW REGEX 'pattern' — show excluded lines matching regex.
    Regex { pattern: String },
}

/// Variants of the RESET command relevant to exclusion.
///
/// Addresses: Requirement 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetVariant {
    /// RESET (no args) — clear exclusion state + delegate to other subsystems.
    Default,
    /// RESET EXCLUDED — clear only exclusion state.
    Excluded,
    /// RESET ALL — clear exclusion as part of full session reset.
    All,
}
```

### ExclusionBlock

```rust
/// A contiguous range of excluded document lines.
/// Used by the viewport renderer to display placeholder lines.
///
/// Addresses: Requirement 6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExclusionBlock {
    /// First excluded document line in this block (0-based).
    pub start: DocLine,
    /// Last excluded document line in this block (0-based, inclusive).
    pub end: DocLine,
}

impl ExclusionBlock {
    /// Number of excluded lines in this block.
    pub fn line_count(&self) -> usize {
        self.end.0 - self.start.0 + 1
    }

    /// Generate placeholder text for viewport display.
    /// Format: "-- N line(s) excluded --"
    ///
    /// Addresses: Requirement 6 AC 2
    pub fn placeholder_text(&self) -> String {
        let count = self.line_count();
        format!("-- {} line(s) excluded --", count)
    }

    /// Check if a document line falls within this block.
    pub fn contains(&self, doc_line: DocLine) -> bool {
        doc_line.0 >= self.start.0 && doc_line.0 <= self.end.0
    }
}
```

### ExclusionResult

```rust
/// Result of an EXCLUDE or SHOW operation.
///
/// Addresses: Requirements 2 AC 8–9, 3 AC 7–8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExclusionResult {
    /// Number of lines whose visibility state was changed.
    pub lines_affected: u64,
    /// Status message for display in the status bar.
    pub status_message: String,
}

impl ExclusionResult {
    /// Create a result for an EXCLUDE operation.
    pub fn excluded(count: u64) -> Self {
        let message = if count > 0 {
            format!("{} line(s) excluded", count)
        } else {
            "No lines matched".to_string()
        };
        Self { lines_affected: count, status_message: message }
    }

    /// Create a result for a SHOW operation.
    pub fn shown(count: u64) -> Self {
        let message = if count > 0 {
            format!("{} line(s) shown", count)
        } else {
            "No excluded lines matched".to_string()
        };
        Self { lines_affected: count, status_message: message }
    }

    /// Create a result for a RESET operation.
    pub fn reset(count: u64) -> Self {
        Self {
            lines_affected: count,
            status_message: format!("RESET: {} line(s) restored to view", count),
        }
    }
}
```

### ExclusionChanged Event

```rust
/// Event emitted when exclusion state changes.
/// Consumed by viewport/scrollbar for synchronization.
///
/// Addresses: Requirement 7 AC 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExclusionChanged {
    /// Total number of currently excluded lines after the change.
    pub total_excluded: u64,
    /// Total number of exclusion blocks after the change.
    pub block_count: usize,
    /// Number of lines whose state changed in this operation.
    pub lines_changed: u64,
}

/// Trait for receiving exclusion-change events.
pub trait ExclusionListener: Send + Sync {
    fn on_exclusion_changed(&self, event: &ExclusionChanged);
}
```

### LineCommandExclude

```rust
/// A resolved X/Xn/XX line command ready for execution.
///
/// Addresses: Requirement 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineCommandExclude {
    /// X — exclude a single line.
    Single { line: DocLine },
    /// Xn — exclude n consecutive lines starting at line.
    Count { line: DocLine, count: u32 },
    /// XX...XX — exclude a block of lines (inclusive range).
    Block { start: DocLine, end: DocLine },
}
```

---

## Public API Surface

### ExclusionEngine — Construction and Configuration

```rust
/// The top-level exclusion engine orchestrating all visibility operations.
/// Holds references to display-line-mapping and delegates text matching
/// to the find-and-replace engine.
///
/// Addresses: Requirements 1–10
pub struct ExclusionEngine {
    /// Reference to the display-line-mapping for visibility storage.
    display_mapping: Arc<RwLock<dyn DisplayLineMapping>>,
    /// Reference to the document model for line content access.
    document: Arc<RwLock<dyn DocumentAccess>>,
    /// Registered exclusion-change listeners.
    listeners: Vec<Box<dyn ExclusionListener>>,
}

impl ExclusionEngine {
    /// Create a new ExclusionEngine.
    pub fn new(
        display_mapping: Arc<RwLock<dyn DisplayLineMapping>>,
        document: Arc<RwLock<dyn DocumentAccess>>,
    ) -> Self;

    /// Register an exclusion-change listener.
    /// Addresses: Requirement 7 AC 5
    pub fn add_listener(&mut self, listener: Box<dyn ExclusionListener>);

    /// Remove a previously registered listener.
    pub fn remove_listener(&mut self, id: ListenerId);
}
```

### ExclusionEngine — Query Methods

```rust
impl ExclusionEngine {
    /// Check if a specific document line is excluded.
    /// Delegates to display_line_mapping.get_visible(doc_line) == false.
    ///
    /// Addresses: Requirement 1 AC 4
    pub fn is_excluded(&self, doc_line: DocLine) -> bool;

    /// Check if any lines in the document are currently excluded.
    /// Delegates to display_line_mapping.hidden_lines().
    ///
    /// Addresses: Requirement 1 AC 5
    pub fn has_excluded_lines(&self) -> bool;

    /// Return the total count of currently excluded lines.
    ///
    /// Addresses: Requirement 1 AC 7
    pub fn excluded_line_count(&self) -> u64;

    /// Return the number of exclusion blocks in the document.
    ///
    /// Addresses: Requirement 6 AC 6
    pub fn block_count(&self) -> usize;

    /// Get all exclusion blocks ordered by document position.
    ///
    /// Addresses: Requirement 6 AC 1
    pub fn exclusion_blocks(&self) -> Vec<ExclusionBlock>;

    /// Get the exclusion block containing a specific document line.
    /// Returns None if the line is not excluded.
    ///
    /// Addresses: Requirement 6 AC 7
    pub fn block_at_doc_line(&self, doc_line: DocLine) -> Option<ExclusionBlock>;
}
```

### ExclusionEngine — Scope Iterators

```rust
impl ExclusionEngine {
    /// Iterate over all currently visible line indices.
    /// Used by find-and-replace for VISIBLE scope.
    ///
    /// Addresses: Requirement 8 AC 5
    pub fn visible_lines_iter(&self) -> impl Iterator<Item = DocLine> + '_;

    /// Iterate over all currently excluded line indices.
    /// Used by find-and-replace for EXCLUDED scope.
    ///
    /// Addresses: Requirement 8 AC 6
    pub fn excluded_lines_iter(&self) -> impl Iterator<Item = DocLine> + '_;
}
```

### ExclusionEngine — EXCLUDE Operations

```rust
impl ExclusionEngine {
    /// Execute an EXCLUDE command with the given arguments.
    /// Returns the operation result including count of lines affected.
    ///
    /// Addresses: Requirement 2
    pub fn execute_exclude(
        &mut self,
        args: &ExcludeArgs,
        session: &SessionState,
        find_engine: &FindEngine,
    ) -> Result<ExclusionResult, ExcludeShowError>;

    /// Exclude a single line by index (used by line commands).
    ///
    /// Addresses: Requirement 1 AC 2
    pub fn exclude_line(&mut self, doc_line: DocLine) -> bool;

    /// Exclude a contiguous range of lines (inclusive).
    /// Maps to display_line_mapping.set_visible(start, end, false).
    ///
    /// Addresses: Requirement 1 AC 8
    pub fn exclude_range(&mut self, start: DocLine, end: DocLine) -> u64;
}
```

### ExclusionEngine — SHOW Operations

```rust
impl ExclusionEngine {
    /// Execute a SHOW/INCLUDE command with the given arguments.
    /// Returns the operation result including count of lines revealed.
    ///
    /// Addresses: Requirement 3
    pub fn execute_show(
        &mut self,
        args: &ShowArgs,
        find_engine: &FindEngine,
    ) -> Result<ExclusionResult, ExcludeShowError>;

    /// Show (un-exclude) a single line by index.
    ///
    /// Addresses: Requirement 1 AC 3
    pub fn show_line(&mut self, doc_line: DocLine) -> bool;

    /// Show a contiguous range of lines (inclusive).
    pub fn show_range(&mut self, start: DocLine, end: DocLine) -> u64;
}
```

### ExclusionEngine — RESET Operations

```rust
impl ExclusionEngine {
    /// Execute a RESET command variant.
    /// Returns the operation result including count of lines restored.
    ///
    /// Addresses: Requirement 4
    pub fn execute_reset(
        &mut self,
        variant: ResetVariant,
    ) -> ExclusionResult;
}
```

### ExclusionEngine — Line Command Processing

```rust
impl ExclusionEngine {
    /// Process a resolved X/Xn/XX line command.
    /// Returns the count of lines excluded.
    ///
    /// Addresses: Requirement 5
    pub fn execute_line_command(
        &mut self,
        command: &LineCommandExclude,
    ) -> ExclusionResult;
}
```

### ScopeFilterProvider Implementation

```rust
/// Implementation of the ScopeFilterProvider trait from ff-find-and-replace.
/// Bridges the exclusion engine's state to the find engine's scope filtering.
///
/// Addresses: Requirement 8
pub struct ExclusionScopeProvider<'a> {
    engine: &'a ExclusionEngine,
    session: &'a SessionState,
}

impl<'a> ExclusionScopeProvider<'a> {
    pub fn new(engine: &'a ExclusionEngine, session: &'a SessionState) -> Self;
}

impl<'a> ScopeFilterProvider for ExclusionScopeProvider<'a> {
    /// Returns true if the line is visible (not excluded).
    fn is_visible(&self, line: LineNumber) -> bool;

    /// Returns true if the line is excluded (hidden).
    fn is_excluded(&self, line: LineNumber) -> bool;

    /// Returns true if the line is tagged (delegates to session state).
    fn is_tagged(&self, line: LineNumber) -> bool;
}
```

### DocumentAccess Trait

```rust
/// Minimal trait for accessing document line content.
/// Implemented by ff-document-model's Document type.
/// Used by the TextMatcher for line-by-line content comparison.
///
/// Addresses: Requirement 2 (text matching needs line content)
pub trait DocumentAccess: Send + Sync {
    /// Get the text content of a specific document line.
    fn line_content(&self, line: DocLine) -> Option<&str>;

    /// Total number of lines in the document.
    fn line_count(&self) -> u64;

    /// Check if the line is tagged (delegates to session/tag state).
    fn is_tagged(&self, line: DocLine) -> bool;
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-exclude-show-filter crate.
/// Formatted per Error Message Standards (Req 8):
/// `[exclude-show] operation: description`
///
/// Addresses: Cross-cutting Requirement 8, Requirement 9 AC 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExcludeShowError {
    /// Invalid regex pattern in EXCLUDE REGEX or SHOW REGEX.
    #[error("[exclude-show] exclude: invalid regex pattern: {detail}")]
    InvalidRegex { detail: String },

    /// Invalid line range arguments (start > end, non-numeric, out of bounds).
    #[error("[exclude-show] exclude: invalid line range {start}–{end} (document has {total} lines)")]
    InvalidRange { start: u64, end: u64, total: u64 },

    /// Unterminated quote in text argument.
    #[error("[exclude-show] {command}: unterminated quote in argument")]
    UnterminatedQuote { command: String },

    /// Invalid argument syntax.
    #[error("[exclude-show] {command}: invalid arguments: {detail}")]
    InvalidArguments { command: String, detail: String },

    /// Display-line-mapping error (line out of range).
    #[error("[exclude-show] {operation}: {detail}")]
    MappingError { operation: String, detail: String },

    /// Document access error.
    #[error("[exclude-show] {operation}: document access failed: {detail}")]
    DocumentError { operation: String, detail: String },
}
```

---

## Integration Points

### With `ff-display-line-mapping` (upstream — Wave 4)

- **Dependency direction**: ff-exclude-show-filter depends on ff-display-line-mapping
- **API consumed**: `set_visible(start, end, visible)`, `get_visible(doc_line)`, `hidden_lines()`, `show_all()`, `lines_in_doc()`
- **Integration pattern**: The ExclusionEngine does NOT maintain its own per-line visibility state. It delegates entirely to the `DisplayLineMapping` trait. When EXCLUDE hides lines, it calls `set_visible(start, end, false)`. When SHOW reveals lines, it calls `set_visible(start, end, true)`. RESET calls `show_all()`.
- **Notification**: The display-line-mapping emits `DisplayLineCountChange` when visibility changes; this crate additionally emits `ExclusionChanged` with higher-level semantics (block count, lines changed).

### With `ff-document-model` (upstream — Wave 4)

- **Dependency direction**: ff-exclude-show-filter depends on ff-document-model
- **API consumed**: `Document::line_content(line)` for text matching, `Document::line_count()` for bounds validation
- **Integration pattern**: Text-matching EXCLUDE and SHOW operations read line content to determine which lines match the search pattern. No document mutations are performed by this crate.

### With `ff-find-and-replace` (peer — Wave 5)

- **Dependency direction**: Bidirectional peer integration
  - ff-exclude-show-filter calls `FindEngine::find_for_filter()` for text-matching delegation
  - ff-find-and-replace calls `ExclusionScopeProvider` for EXCLUDED/VISIBLE scope filtering
- **API consumed from find-and-replace**: `find_for_filter(request, indexer, scope_filter, bounds)` — executes a search without updating FindState
- **API provided to find-and-replace**: `ScopeFilterProvider` implementation (`is_visible`, `is_excluded`, `is_tagged`)
- **Design note**: To avoid circular crate dependencies, the `ScopeFilterProvider` trait is defined in `ff-find-and-replace` and implemented by this crate. The `find_for_filter` call is injected via a trait or function pointer rather than a direct crate dependency.

### With `ff-command-semantics` (peer — Wave 5)

- **Dependency direction**: ff-exclude-show-filter depends on ff-command-semantics
- **API consumed**: `ScopeResolver` for resolving scope modifiers (VISIBLE, EXCLUDED, ALL, TAGGED), `SessionState` for tag queries, `StatusMessage` for formatted output
- **Integration pattern**: When EXCLUDE receives scope modifiers, it uses the `ScopeFilter` enum from command-semantics to determine which lines to operate on. Tag state is queried from `SessionState` for `EXCLUDE TAGGED`.

### With `ff-command` (upstream — Wave 2)

- **Dependency direction**: ff-exclude-show-filter depends on ff-command
- **API consumed**: `CommandRegistry::register()` for command registration
- **Commands registered**:

| Command ID | Display Name | Aliases | Undoable | Category |
|-----------|-------------|---------|----------|----------|
| `filter.exclude` | Exclude | `X` | No | filter |
| `filter.show` | Show | `INCLUDE` | No | filter |
| `filter.reset` | Reset | — | No | filter |

### With `ff-line-commands` (peer — Wave 5)

- **Dependency direction**: ff-line-commands calls into ff-exclude-show-filter
- **API provided**: `execute_line_command(LineCommandExclude)` — processes X/Xn/XX commands
- **Integration pattern**: The line-command subsystem parses and resolves X/Xn/XX pairs, then delegates the actual exclusion operation to this crate's `ExclusionEngine`.

### With `ff-viewport-and-scrolling` (downstream — Wave 4)

- **Dependency direction**: ff-viewport-and-scrolling depends on ff-exclude-show-filter
- **API consumed**: `exclusion_blocks()`, `block_at_doc_line()`, `ExclusionBlock::placeholder_text()`
- **Integration pattern**: The viewport renderer queries exclusion blocks to determine where to render placeholder lines and what text they contain.

### With `ff-logging` (upstream — Wave 0)

- **Dependency direction**: ff-exclude-show-filter depends on ff-logging
- **Usage**: INFO-level logs for bulk operations (EXCLUDE ALL, RESET), DEBUG for per-operation details
- **Log prefix**: `[exclude-show]`

### Dependency Direction Summary

```
ff-logging            ← ff-exclude-show-filter
ff-display-line-mapping ← ff-exclude-show-filter
ff-document-model     ← ff-exclude-show-filter
ff-command            ← ff-exclude-show-filter
ff-command-semantics  ← ff-exclude-show-filter
ff-find-and-replace   ↔ ff-exclude-show-filter (via trait injection)
ff-exclude-show-filter ← ff-viewport-and-scrolling
ff-exclude-show-filter ← ff-navigation-commands
ff-line-commands      → ff-exclude-show-filter
```

---

## Configuration

The `ff-exclude-show-filter` crate reads settings from the `[exclude]` namespace in the workbench TOML configuration.

### TOML Schema

```toml
[exclude]
# Whether EXCLUDE text matching is case-sensitive by default.
# Default: false (case-insensitive)
# Addresses: Requirement 2 AC 1
case_sensitive = false

# Placeholder text format. Supports {count} substitution.
# Default: "-- {count} line(s) excluded --"
# Addresses: Requirement 6 AC 2
placeholder_format = "-- {count} line(s) excluded --"
```

### Config Resolution Rules

| Setting | Absent | Invalid Value |
|---------|--------|---------------|
| `case_sensitive` | Default to `false` | Default + WARN log |
| `placeholder_format` | Default to `"-- {count} line(s) excluded --"` | Default + WARN log (must contain `{count}`) |

---

## Concurrency Model

### Thread-Safety Approach

| Component | Mechanism | Rationale |
|-----------|-----------|-----------|
| `ExclusionEngine` | Owned per-editor-tab; not shared across threads | Each document session has its own engine instance |
| `DisplayLineMapping` access | `Arc<RwLock<dyn DisplayLineMapping>>` | Display mapping may be read by viewport thread |
| `DocumentAccess` | `Arc<RwLock<dyn DocumentAccess>>` | Document may be accessed from background threads |
| Listeners | `Vec<Box<dyn ExclusionListener>>` | Listeners are invoked synchronously on the command thread |
| `ExclusionScopeProvider` | Short-lived borrow (`&'a ExclusionEngine`) | Created per find operation, not persisted |

### Operation Atomicity

EXCLUDE/SHOW/RESET operations acquire the display-mapping write lock once and perform all visibility mutations within that single lock acquisition. This ensures that:
- No intermediate states are visible to viewport rendering
- Display line count changes are emitted as a single notification after all mutations complete
- Block enumeration is consistent with visibility state at all times

---

## Design Decisions

### Decision 1: No Separate Visibility State — Delegate to display-line-mapping

**Chosen: Delegate all per-line visibility storage to `ff-display-line-mapping`**

Rationale:
1. The display-line-mapping already maintains per-line visibility as its core responsibility
2. Duplicating state would create consistency risks between the two layers
3. The `set_visible`/`get_visible` API provides exactly the primitives needed
4. `show_all()` gives O(1) bulk reset by returning to one-to-one mode
5. The Fenwick tree in display-line-mapping handles display-line-count updates automatically

Trade-offs accepted:
- Querying exclusion state requires going through the display-mapping layer (minor indirection)
- Cannot distinguish "excluded by EXCLUDE command" from "hidden by code fold" without additional metadata — acceptable because ISPF exclusion is flat and orthogonal to folding

### Decision 2: Text Matching via find-for-filter Delegation

**Chosen: Delegate text matching to `ff-find-and-replace::find_for_filter()`**

Rationale:
1. The find engine already implements literal search with case folding and regex matching
2. Reusing the find engine avoids duplicating search logic in this crate
3. `find_for_filter` specifically does NOT update FindState — ideal for exclusion use
4. Consistent matching semantics between FIND and EXCLUDE commands

Trade-offs accepted:
- Creates a cross-dependency between Wave 5 crates (resolved via trait injection)
- Slightly more complex wiring than a standalone text scanner
- For simple literal contains-check, delegating to the find engine has marginal overhead

### Decision 3: Block Enumeration as Computed View

**Chosen: Compute exclusion blocks on demand by scanning visibility state**

Rationale:
1. Exclusion blocks change infrequently relative to how often they are queried
2. Scanning is O(n) but block enumeration is O(k) where k << n in practice
3. Avoids maintaining a separate block data structure that must be kept in sync
4. For very large documents, a cached block list can be added as an optimization without API change

Trade-offs accepted:
- First call to `exclusion_blocks()` after a mutation scans all lines to find boundaries
- Could be optimized with a cached block list invalidated on `ExclusionChanged` events
- For typical usage (< 100 blocks), scan time is negligible

### Decision 4: Non-Undoable Operations

**Chosen: EXCLUDE/SHOW/RESET are explicitly non-undoable session state**

Rationale:
1. ISPF semantics treat exclusion as a view filter, not a content modification
2. Exclusion state is transient — lost on session close, not saved to disk
3. Including exclusion in undo history would pollute the undo stack with non-content changes
4. RESET provides the "undo" path for exclusion operations
5. Command metadata explicitly marks these as `undoable: false`

---

## Correctness Properties

The following properties are suitable for property-based testing with the `proptest` crate. Each property is universal — it must hold for all valid inputs.

### Property 1: Exclude-Show Roundtrip

**Statement:** For any document and set of lines excluded via `exclude_range(start, end)`, calling `show_range(start, end)` restores all those lines to visible, and `excluded_line_count()` returns the same value as before the exclude.

```
∀ ExclusionEngine E, ∀ start, end where start ≤ end < line_count:
    let before_count = E.excluded_line_count();
    E.exclude_range(start, end);
    E.show_range(start, end);
    E.excluded_line_count() == before_count
    ∧ ∀ line in start..=end: E.is_excluded(line) == false
```

**Validates: Requirements 1.2, 1.3, 1.8**

### Property 2: Exclude All Then Show All Identity

**Statement:** After `EXCLUDE ALL` followed by `SHOW ALL`, no lines are excluded and `excluded_line_count() == 0`.

```
∀ ExclusionEngine E:
    E.execute_exclude(&ExcludeArgs::All, ...);
    E.execute_show(&ShowArgs::All, ...);
    E.excluded_line_count() == 0
    ∧ E.has_excluded_lines() == false
    ∧ ∀ line in 0..line_count: E.is_excluded(line) == false
```

**Validates: Requirements 2.4, 3.1**

### Property 3: Excluded Line Count Consistency

**Statement:** `excluded_line_count()` always equals the number of lines for which `is_excluded(line) == true`.

```
∀ ExclusionEngine E:
    E.excluded_line_count() == count(line for line in 0..line_count where E.is_excluded(line))
```

**Validates: Requirements 1.4, 1.5, 1.7**

### Property 4: Block Contiguity Invariant

**Statement:** Every `ExclusionBlock` returned by `exclusion_blocks()` represents a maximally contiguous range — the line before `start` (if it exists) is visible, and the line after `end` (if it exists) is visible.

```
∀ ExclusionEngine E, ∀ block in E.exclusion_blocks():
    (block.start.0 == 0 ∨ E.is_excluded(DocLine(block.start.0 - 1)) == false)
    ∧ (block.end.0 == line_count - 1 ∨ E.is_excluded(DocLine(block.end.0 + 1)) == false)
    ∧ ∀ line in block.start.0..=block.end.0: E.is_excluded(DocLine(line)) == true
```

**Validates: Requirements 6.1, 6.5**

### Property 5: Block Count Equals Transition Count

**Statement:** The number of exclusion blocks equals the number of visible→excluded transitions when scanning lines from top to bottom.

```
∀ ExclusionEngine E:
    let transitions = count(i for i in 0..line_count where
        E.is_excluded(DocLine(i)) == true
        ∧ (i == 0 ∨ E.is_excluded(DocLine(i - 1)) == false)
    );
    E.block_count() == transitions
```

**Validates: Requirements 6.6, 6.8**

### Property 6: RESET Clears All Exclusion State

**Statement:** After any RESET variant that clears exclusion (Default, Excluded, All), `has_excluded_lines()` returns false and all lines are visible.

```
∀ ExclusionEngine E, ∀ variant in [Default, Excluded, All]:
    E.execute_reset(variant);
    E.has_excluded_lines() == false
    ∧ E.excluded_line_count() == 0
    ∧ E.block_count() == 0
```

**Validates: Requirements 4.1, 4.2, 4.3, 4.4**

### Property 7: Exclude Range Count Correctness

**Statement:** `exclude_range(start, end)` returns a count equal to the number of lines in the range that were previously visible.

```
∀ ExclusionEngine E, ∀ start, end where start ≤ end < line_count:
    let previously_visible = count(line for line in start..=end where E.is_excluded(DocLine(line)) == false);
    let result = E.exclude_range(DocLine(start), DocLine(end));
    result == previously_visible
```

**Validates: Requirements 2.8, 1.8**

### Property 8: Show Only Affects Excluded Lines

**Statement:** A SHOW operation never changes the visibility of an already-visible line. After any SHOW operation, every line that was visible before remains visible.

```
∀ ExclusionEngine E, ∀ ShowArgs args:
    let visible_before = set(line for line in 0..line_count where !E.is_excluded(DocLine(line)));
    E.execute_show(&args, ...);
    ∀ line in visible_before: E.is_excluded(DocLine(line)) == false
```

**Validates: Requirements 3.1, 3.2, 3.4, 3.5**

### Property 9: Visible and Excluded Iterators Partition All Lines

**Statement:** The union of `visible_lines_iter()` and `excluded_lines_iter()` equals the set of all document lines, and their intersection is empty.

```
∀ ExclusionEngine E:
    let visible = set(E.visible_lines_iter());
    let excluded = set(E.excluded_lines_iter());
    visible ∪ excluded == set(0..line_count)
    ∧ visible ∩ excluded == ∅
```

**Validates: Requirements 8.5, 8.6**

### Property 10: EXCLUDE Text Matches Subset of Visible Lines

**Statement:** When `EXCLUDE 'text'` is issued (with default Visible scope), only previously visible lines become excluded. No already-excluded line changes state.

```
∀ ExclusionEngine E, ∀ text pattern:
    let excluded_before = set(line for line in 0..line_count where E.is_excluded(DocLine(line)));
    E.execute_exclude(&ExcludeArgs::Text { pattern: text, scope: Visible }, ...);
    ∀ line in excluded_before: E.is_excluded(DocLine(line)) == true
    // Previously excluded lines remain excluded (state unchanged)
```

**Validates: Requirements 2.1, 2.9**

### Property 11: Placeholder Text Reflects Block Size

**Statement:** For every exclusion block, `placeholder_text()` contains the correct line count as a decimal number.

```
∀ ExclusionEngine E, ∀ block in E.exclusion_blocks():
    block.placeholder_text().contains(&block.line_count().to_string())
```

**Validates: Requirements 6.2**

### Property 12: Line Command Count Correctness

**Statement:** An `Xn` line command starting at line L excludes exactly min(n, remaining_lines) consecutive lines and reports the correct count.

```
∀ ExclusionEngine E, ∀ L, n where L < line_count:
    let expected = min(n, line_count - L);
    let result = E.execute_line_command(&LineCommandExclude::Count { line: DocLine(L), count: n });
    result.lines_affected <= expected
    ∧ ∀ i in L..L+expected where i < line_count:
        E.is_excluded(DocLine(i)) == true
```

**Validates: Requirements 5.2, 5.7**

---

## Testing Strategy

### Unit Tests

Unit tests are co-located with source modules using `#[cfg(test)] mod tests { ... }`:

- `commands/exclude.rs` — EXCLUDE text/regex/ALL/TAGGED/range operations
- `commands/show.rs` — SHOW ALL/EXCLUDED/NONEXCLUDED/text/regex operations
- `commands/reset.rs` — RESET Default/Excluded/All operations
- `line_commands.rs` — X/Xn/XX processing, boundary cases
- `blocks.rs` — Block enumeration, merge/split, placeholder text generation
- `iterators.rs` — Visible/excluded iterator correctness, empty document edge case
- `matcher.rs` — Text matching delegation, case sensitivity, regex errors
- `scope_provider.rs` — ScopeFilterProvider implementation verification

### Property-Based Tests (proptest)

All 12 properties listed in the Correctness Properties section are implemented as proptest tests with ≥100 cases. Strategies generate:

- Random document sizes (1–10000 lines)
- Random exclusion patterns (ranges, text matches against generated content)
- Random sequences of EXCLUDE/SHOW/RESET operations
- Mixed visibility states (partial exclusion, full exclusion, no exclusion)

### Integration Tests

`tests/integration.rs` exercises end-to-end scenarios:

- EXCLUDE ALL → SHOW 'text' workflow (ISPF filtering pattern)
- Multiple EXCLUDE operations with overlapping ranges → block merging
- RESET after complex exclusion patterns
- X/Xn/XX line commands via the full execution path
- ScopeFilterProvider integration with mock find engine

### Test Infrastructure

- **Testing framework**: `proptest` 1.0 with minimum 100 cases per property
- **Assertions**: `pretty_assertions::assert_eq!` for readable diffs
- **No mocking of display-line-mapping**: Tests use a real `ContractionState` instance for end-to-end verification
- **Mock document**: Simple in-memory document implementation for text-matching tests

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
| `ff-logging` | Structured logging (`[exclude-show]` prefix) |
| `ff-display-line-mapping` | Per-line visibility storage (`DisplayLineMapping` trait) |
| `ff-document-model` | Line content access for text matching |
| `ff-command` | Command registration and dispatch |
| `ff-command-semantics` | Scope resolution, session state, status messages |
| `ff-find-and-replace` | `find_for_filter` delegation, `ScopeFilterProvider` trait |

---

## Appendix B: Command Registration Table

Commands registered by `ff-exclude-show-filter` with the global `CommandRegistry`:

| Command ID | Display Name | Aliases | Undoable | Category | Modes |
|-----------|-------------|---------|----------|----------|-------|
| `filter.exclude` | Exclude | `X` | No | filter | Edit, Browse, View |
| `filter.show` | Show | `INCLUDE` | No | filter | Edit, Browse, View |
| `filter.reset` | Reset | — | No | filter | Edit, Browse, View |

Line commands registered with the line-command parser:

| Prefix | Kind | Block | Description |
|--------|------|-------|-------------|
| `X` | Exclude | No | Exclude single line (Xn for n lines) |
| `XX` | ExcludeBlock | Yes | Exclude block of lines (paired) |

---

## Appendix C: ISPF Filtering Workflow

The canonical ISPF filtering workflow supported by this crate:

```
1. EXCLUDE ALL              → All lines hidden (placeholder: "-- N line(s) excluded --")
2. FIND 'keyword' ALL       → Cursor moves to first match
3. SHOW 'keyword'           → Only lines containing 'keyword' become visible
4. [User reviews filtered view]
5. RESET EXCLUDED           → All lines restored to visible
```

This workflow is the primary use case driving the ExcludeArgs::All + ShowArgs::Text combination. It enables rapid filtering of large files without modifying content.
