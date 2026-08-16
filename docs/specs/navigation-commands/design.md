# Design Document: Navigation Commands (`ff-navigation-commands`)

## Overview

The `ff-navigation-commands` crate implements **all navigation, display-artifact, and line-reorder commands** for FileForgeWorkbench. It covers LOCATE, SORT, COLS, BOUNDS, viewport scroll commands (UP/DOWN/LEFT/RIGHT/TOP/BOTTOM), paragraph navigation, word navigation, word-part (camelCase) navigation, vertical caret movement with column affinity, and document-start/end navigation.

### Purpose

- Execute LOCATE command to jump to a specific line number or named label
- Execute SORT command to reorder lines by column key (the only undoable command in this crate)
- Manage COLS_Line display artifacts (non-editable column ruler overlays)
- Manage BOUNDS/BNDS state and BNDS_Line display artifacts
- Execute viewport scroll commands (UP, DOWN, LEFT, RIGHT, TOP, BOTTOM)
- Execute paragraph navigation (PARA_UP, PARA_DOWN)
- Execute word navigation (WORD_LEFT, WORD_RIGHT, WORD_END_RIGHT)
- Execute word-part navigation (WORD_PART_LEFT, WORD_PART_RIGHT)
- Manage column affinity for natural vertical caret movement
- Execute document-start and document-end navigation (DOC_START, DOC_END)
- Expose active Bounds state via a public query API for other command executors
- Register all commands with the command framework with correct metadata

### Position in Architecture

```
Wave 5 — Command Engine

┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│    Renders COLS/BNDS overlays, forwards navigation input     │
├─────────────────────────────────────────────────────────────┤
│  Peers: ff-command-semantics (command pipeline, scope)        │
│         ff-find-and-replace (reads Bounds for FIND)           │
│         ff-line-commands (reads Bounds for shift)             │
│         ff-exclude-show-filter (excluded-line queries)        │
├─────────────────────────────────────────────────────────────┤
│      ff-navigation-commands (THIS CRATE — Wave 5)            │
│  LOCATE, SORT, COLS, BOUNDS, UP/DOWN/LEFT/RIGHT/TOP/BOTTOM, │
│  paragraph nav, word nav, word-part nav, doc-start/end       │
├─────────────────────────────────────────────────────────────┤
│  ff-viewport-scrolling (Wave 4) — viewport state mutations   │
│  ff-document-model (Wave 4) — line count, content, char nav  │
│  ff-display-line-mapping (Wave 4) — excluded-line awareness  │
│  ff-command (Wave 2) — command registration, dispatch        │
│  ff-undo-redo-transactions (Wave 4) — SORT transaction       │
│  ff-configuration-system (Wave 2) — configurable defaults    │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Zero GUI framework dependencies — all logic operates on viewport/document models
- **Command-Driven (Req 4)**: All commands register via `ff-command` CommandRegistry and dispatch through `execute_command`
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-navigation-commands`
- **Error Message Standards (Req 8)**: All errors follow `[navigation] operation: description` format; status messages ≤200 chars
- **Configuration (Req 5)**: Scroll amounts, bounds_affect_find, word-characters are configurable via TOML
- **Async I/O (Req 6)**: Not applicable — all navigation logic is synchronous (no I/O)

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Input Sources"
        CL[Command Line: LOCATE, SORT, COLS, BOUNDS, UP, DOWN...]
        KB[Keyboard: Ctrl+Home, Ctrl+End, Ctrl+Left/Right, Alt+Up/Down]
        LUA[Lua Macros: navigation API calls]
    end

    subgraph "ff-navigation-commands"
        LOC[LocateCommand<br/>line/label jump]
        SORT[SortCommand<br/>undoable line reorder]
        COLS[ColsManager<br/>column ruler overlays]
        BND[BoundsManager<br/>active bounds state + BNDS_Line]
        SCROLL[ScrollCommands<br/>UP/DOWN/LEFT/RIGHT/TOP/BOTTOM]
        PARA[ParagraphNav<br/>PARA_UP/PARA_DOWN]
        WORD[WordNav<br/>WORD_LEFT/WORD_RIGHT/WORD_END_RIGHT]
        WPART[WordPartNav<br/>WORD_PART_LEFT/WORD_PART_RIGHT]
        VERT[VerticalCaretNav<br/>line-up/down, page-up/down, affinity]
        DOCNAV[DocStartEndNav<br/>DOC_START/DOC_END]
        CHARCLASS[CharClassifier<br/>character class tables]
        REG[CommandRegistration<br/>metadata + handlers]
    end

    subgraph "Upstream Crates"
        VP[ff-viewport-scrolling<br/>ViewportModel, CursorModel]
        DOC[ff-document-model<br/>Document, line content]
        DLM[ff-display-line-mapping<br/>excluded-line queries]
        CMD[ff-command<br/>CommandRegistry, dispatch]
        UNDO[ff-undo-redo-transactions<br/>Transaction API]
        CFG[ff-configuration-system<br/>TOML settings]
        LOG[ff-logging]
    end

    CL --> REG
    KB --> CMD
    LUA --> CMD
    CMD --> REG

    REG --> LOC
    REG --> SORT
    REG --> COLS
    REG --> BND
    REG --> SCROLL
    REG --> PARA
    REG --> WORD
    REG --> WPART
    REG --> VERT
    REG --> DOCNAV

    LOC --> VP
    LOC --> DOC
    SORT --> DOC
    SORT --> BND
    SORT --> UNDO
    COLS --> VP
    BND --> CFG
    SCROLL --> VP
    PARA --> DOC
    PARA --> VP
    PARA --> DLM
    WORD --> DOC
    WORD --> CHARCLASS
    WORD --> VP
    WPART --> DOC
    WPART --> CHARCLASS
    WPART --> VP
    VERT --> VP
    DOCNAV --> VP
    DOCNAV --> DOC
    CHARCLASS --> CFG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **LocateCommand** | Parses LOCATE arguments (line number or label), validates range, delegates viewport jump to `ff-viewport-scrolling` |
| **SortCommand** | Parses SORT arguments, resolves scope, extracts column key, performs stable sort, records undo transaction |
| **ColsManager** | Manages COLS_Line display artifacts — insertion, removal, toggle, position tracking, RESET handling |
| **BoundsManager** | Manages active Bounds state in session, BNDS_Line display artifact, public query API for other crates |
| **ScrollCommands** | Implements UP/DOWN/LEFT/RIGHT/TOP/BOTTOM by delegating to ViewportModel scroll methods |
| **ParagraphNav** | Detects paragraph boundaries (blank lines), moves caret across paragraphs, skips excluded lines |
| **WordNav** | Moves caret by word boundaries using character class transitions |
| **WordPartNav** | Moves caret by sub-word boundaries (camelCase, snake_case transitions) |
| **VerticalCaretNav** | Manages line-up/down, page-up/down with column affinity via CursorModel |
| **DocStartEndNav** | Jumps caret to document start (position 0) or end (last line end) |
| **CharClassifier** | Character classification engine: space, newLine, word, punctuation; configurable per document |
| **CommandRegistration** | Registers all commands with `ff-command` with metadata, help text, mode validity |

---

## Components and Interfaces

```
crates/ff-navigation-commands/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── locate.rs                   # LocateCommand — line number and label navigation
│   ├── sort.rs                     # SortCommand — undoable line reorder
│   ├── cols.rs                     # ColsManager — COLS_Line display artifacts
│   ├── bounds.rs                   # BoundsManager — active bounds state + BNDS_Line
│   ├── scroll/
│   │   ├── mod.rs                  # ScrollCommands re-exports
│   │   ├── vertical.rs            # UP, DOWN, TOP, BOTTOM command handlers
│   │   └── horizontal.rs          # LEFT, RIGHT command handlers
│   ├── paragraph.rs               # ParagraphNav — PARA_UP, PARA_DOWN
│   ├── word.rs                     # WordNav — WORD_LEFT, WORD_RIGHT, WORD_END_RIGHT
│   ├── word_part.rs                # WordPartNav — WORD_PART_LEFT, WORD_PART_RIGHT
│   ├── vertical_caret.rs          # VerticalCaretNav — line/page up/down with affinity
│   ├── doc_nav.rs                  # DocStartEndNav — DOC_START, DOC_END
│   ├── char_class.rs              # CharClassifier — character classification engine
│   ├── selection.rs               # Selection extension helpers (Extend modifier)
│   ├── registration.rs            # Command framework registration and metadata
│   ├── config.rs                   # Configuration keys and defaults
│   └── error.rs                    # NavigationError enum
└── tests/
    ├── locate_tests.rs             # LOCATE command tests
    ├── sort_tests.rs               # SORT command property tests
    ├── cols_tests.rs               # COLS display artifact tests
    ├── bounds_tests.rs             # BOUNDS state management tests
    ├── scroll_tests.rs             # UP/DOWN/LEFT/RIGHT/TOP/BOTTOM tests
    ├── paragraph_tests.rs          # Paragraph navigation property tests
    ├── word_tests.rs               # Word navigation property tests
    ├── word_part_tests.rs          # Word-part navigation property tests
    ├── vertical_caret_tests.rs     # Vertical caret + affinity property tests
    ├── doc_nav_tests.rs            # Document start/end navigation tests
    ├── char_class_tests.rs         # Character classification property tests
    └── integration.rs              # End-to-end navigation scenarios
```

---

## Data Models

### Character Classification

```rust
/// Character class categories for word navigation.
/// Addresses: Requirement 7 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterClass {
    /// Whitespace characters (space, tab, etc.)
    Space,
    /// Line ending characters (LF, CR)
    NewLine,
    /// Word characters (alphanumeric + configured extras)
    Word,
    /// Punctuation/symbol characters (everything else)
    Punctuation,
}

/// Configurable character classification table.
/// ASCII characters (0x00–0x7F) use a lookup table;
/// Unicode characters (>= 0x80) use Unicode category tables.
/// Addresses: Requirement 7 AC 1, AC 9
pub struct CharClassifier {
    /// Classification for each ASCII byte (0–127).
    ascii_table: [CharacterClass; 128],
    /// Additional characters to treat as word characters (from config).
    extra_word_chars: Vec<char>,
}

impl CharClassifier {
    /// Create with default classification (alphanumeric = Word, whitespace = Space,
    /// newlines = NewLine, everything else = Punctuation).
    pub fn new() -> Self;

    /// Classify a single character.
    /// Addresses: Requirement 7 AC 1
    pub fn classify(&self, ch: char) -> CharacterClass;

    /// Set custom character classes for a set of characters.
    /// Addresses: Requirement 7 AC 9
    pub fn set_char_classes(&mut self, chars: &str, class: CharacterClass);

    /// Reset to default classification.
    /// Addresses: Requirement 7 AC 9
    pub fn set_default_classes(&mut self);

    /// Add extra word characters from configuration.
    /// Addresses: Requirement 18 AC 4
    pub fn add_word_characters(&mut self, chars: &str);
}
```

### Bounds State

```rust
/// Active column boundaries for column-sensitive operations.
/// Addresses: Requirement 5 AC 1, AC 15
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveBounds {
    /// Left column boundary (1-based, inclusive).
    pub left: u64,
    /// Right column boundary (1-based, inclusive).
    pub right: u64,
}

impl ActiveBounds {
    /// Create validated bounds. Returns None if left < 1 or right <= left.
    /// Addresses: Requirement 5 AC 13
    pub fn new(left: u64, right: u64) -> Option<Self> {
        if left >= 1 && right > left {
            Some(Self { left, right })
        } else {
            None
        }
    }

    /// Compute the intersection of these bounds with an explicit column range.
    /// Returns None if the intersection is empty.
    /// Addresses: Requirement 2 AC 10
    pub fn intersect(&self, col1: u64, col2: u64) -> Option<(u64, u64)> {
        let effective_left = self.left.max(col1);
        let effective_right = self.right.min(col2);
        if effective_left <= effective_right {
            Some((effective_left, effective_right))
        } else {
            None
        }
    }
}

/// Session-level bounds state manager.
/// Addresses: Requirement 5 AC 1–15
pub struct BoundsManager {
    /// Currently active bounds (None = no bounds set).
    active_bounds: Option<ActiveBounds>,
    /// Whether to affect FIND operations.
    /// Addresses: Requirement 5 AC 8, Requirement 18 AC 3
    affect_find: bool,
    /// Positions of BNDS_Lines in the display (anchored line numbers).
    bnds_line_positions: Vec<u64>,
}

impl BoundsManager {
    /// Create with no active bounds.
    pub fn new() -> Self;

    /// Set active bounds. Returns error for invalid values.
    /// Addresses: Requirement 5 AC 1, AC 13
    pub fn set_bounds(&mut self, left: u64, right: u64) -> Result<(), NavigationError>;

    /// Clear active bounds and remove BNDS_Line.
    /// Addresses: Requirement 5 AC 4, AC 11
    pub fn clear_bounds(&mut self);

    /// Query current active bounds (public API for other crates).
    /// Addresses: Requirement 5 AC 15
    pub fn active_bounds(&self) -> Option<ActiveBounds>;

    /// Whether bounds should affect FIND operations.
    /// Addresses: Requirement 5 AC 8
    pub fn bounds_affect_find(&self) -> bool;

    /// Update configuration (called on config reload).
    pub fn update_config(&mut self, affect_find: bool);
}
```

### COLS Display Artifacts

```rust
/// A single COLS_Line display artifact, anchored to a document position.
/// Addresses: Requirement 4 AC 1–11
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColsLine {
    /// The document line number this COLS_Line is anchored above.
    pub anchor_line: u64,
    /// Unique identifier for this COLS_Line instance.
    pub id: u64,
}

/// Manages all COLS_Line display artifacts for a session.
/// Addresses: Requirement 4
pub struct ColsManager {
    /// Active COLS_Lines ordered by anchor position.
    cols_lines: Vec<ColsLine>,
    /// Next unique ID for new COLS_Lines.
    next_id: u64,
}

impl ColsManager {
    /// Create with no active COLS_Lines.
    pub fn new() -> Self;

    /// Insert a COLS_Line at the given anchor position (or toggle off if already present).
    /// Addresses: Requirement 4 AC 1, AC 4
    pub fn toggle_at(&mut self, anchor_line: u64) -> ColsToggleResult;

    /// Insert a COLS_Line above a specific document line (from line command).
    /// Addresses: Requirement 4 AC 7
    pub fn insert_above(&mut self, doc_line: u64);

    /// Remove all COLS_Lines (RESET command).
    /// Addresses: Requirement 4 AC 5, AC 10
    pub fn reset_all(&mut self);

    /// Query all active COLS_Lines.
    pub fn active_cols_lines(&self) -> &[ColsLine];

    /// Format the COLS ruler string.
    /// Addresses: Requirement 4 AC 2
    pub fn format_ruler() -> &'static str;
}

/// Result of a COLS toggle operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColsToggleResult {
    /// A new COLS_Line was inserted.
    Inserted(ColsLine),
    /// An existing COLS_Line was removed.
    Removed(u64),
}
```

### SORT Data Types

```rust
/// Sort direction.
/// Addresses: Requirement 2 AC 3, AC 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Ascending
    }
}

/// Sort scope qualifier.
/// Addresses: Requirement 2 AC 5, AC 6, AC 7, AC 12
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortScope {
    /// Sort all visible lines (default).
    AllVisible,
    /// Sort only tagged lines.
    Tagged,
    /// Sort only currently visible (non-excluded) lines.
    Visible,
    /// Sort lines within a pending CC block.
    Block { start_line: u64, end_line: u64 },
}

/// Parsed SORT command parameters.
/// Addresses: Requirement 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortParams {
    /// Optional explicit column range for the sort key.
    pub column_range: Option<(u64, u64)>,
    /// Sort direction (A or D).
    pub direction: SortDirection,
    /// Scope qualifier.
    pub scope: SortScope,
}

/// The undo record for a SORT operation.
/// Addresses: Requirement 2 AC 11
#[derive(Debug)]
pub struct SortUndoRecord {
    /// The original line ordering (indices before sort).
    pub original_order: Vec<u64>,
    /// The scope that was sorted.
    pub scope: SortScope,
    /// Description for undo history.
    pub description: String,
}
```

### Navigation Configuration

```rust
/// Configuration values for navigation commands.
/// Addresses: Requirement 18
#[derive(Debug, Clone, PartialEq)]
pub struct NavigationConfig {
    /// Columns to scroll for LEFT/RIGHT without explicit count.
    /// Addresses: Requirement 18 AC 1
    pub horizontal_scroll_columns: u64,
    /// Lines of overlap to retain when scrolling by page.
    /// Addresses: Requirement 18 AC 2
    pub page_overlap_lines: u64,
    /// Whether active Bounds restrict FIND operations.
    /// Addresses: Requirement 18 AC 3
    pub bounds_affect_find: bool,
    /// Additional characters to treat as word characters.
    /// Addresses: Requirement 18 AC 4
    pub extra_word_characters: String,
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self {
            horizontal_scroll_columns: 8,
            page_overlap_lines: 2,
            bounds_affect_find: false,
            extra_word_characters: String::new(),
        }
    }
}
```

### Selection Extension

```rust
/// Modifier indicating whether a navigation operation should extend selection.
/// Addresses: Requirements 6 AC 7, 7 AC 8, 8 AC 6, 9 AC 10, 10 AC 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionModifier {
    /// Move caret without changing selection (collapse).
    Move,
    /// Extend selection from anchor to new caret position.
    Extend,
}
```

### Word Navigation Types

```rust
/// Direction for word/word-part navigation.
/// Addresses: Requirements 7, 8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordDirection {
    /// Move towards the beginning of the document.
    Left,
    /// Move towards the end of the document.
    Right,
}

/// Word navigation variant.
/// Addresses: Requirement 7 AC 2–4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordNavKind {
    /// Move to start of previous/next word.
    WordStart,
    /// Move to end of current/next word.
    WordEnd,
}

/// Sub-word boundary detection result.
/// Addresses: Requirement 8 AC 5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordPartBoundary {
    /// Lowercase to uppercase transition (camelCase).
    LowerToUpper,
    /// End of uppercase run before lowercase (XMLParser → XML|Parser).
    UpperRunBeforeLower,
    /// Alphanumeric to non-alphanumeric transition.
    AlphaToNonAlpha,
    /// Digit to alpha or alpha to digit transition.
    DigitAlphaTransition,
    /// Start or end of word (no internal boundary found).
    WordEdge,
}
```

---

## Public API Surface

### BoundsManager — Public Query API

```rust
/// Public API for other crates to query active bounds.
/// Addresses: Requirement 5 AC 15
impl BoundsManager {
    /// Returns the current active bounds, if set.
    pub fn active_bounds(&self) -> Option<ActiveBounds>;

    /// Returns whether bounds should affect FIND operations.
    pub fn bounds_affect_find(&self) -> bool;

    /// Computes the effective column range for a SORT operation.
    /// If explicit range given, intersects with bounds; otherwise uses bounds directly.
    /// Addresses: Requirement 2 AC 9, AC 10
    pub fn effective_sort_range(&self, explicit: Option<(u64, u64)>) -> Option<(u64, u64)>;
}
```

### LocateCommand — Public API

```rust
/// LOCATE command executor.
/// Addresses: Requirement 1
pub struct LocateCommand;

impl LocateCommand {
    /// Execute LOCATE with a line number target.
    /// Scrolls viewport so target line is top, updates cursor_line to target, cursor_column to 1.
    /// Addresses: Requirement 1 AC 1, AC 6
    pub fn locate_line(
        viewport: &mut ViewportModel,
        cursor: &mut CursorModel,
        target_line: u64,
        doc_line_count: u64,
    ) -> Result<(), NavigationError>;

    /// Execute LOCATE with a label target.
    /// Addresses: Requirement 1 AC 3
    pub fn locate_label(
        viewport: &mut ViewportModel,
        cursor: &mut CursorModel,
        label: &str,
        label_registry: &LabelRegistry,
        doc_line_count: u64,
    ) -> Result<(), NavigationError>;
}

/// A registry of named labels mapped to document line numbers.
/// Labels are defined by the session (e.g., via .LABEL line command).
pub trait LabelRegistry {
    /// Resolve a label name to a line number.
    fn resolve_label(&self, name: &str) -> Option<u64>;
}
```

### SortCommand — Public API

```rust
/// SORT command executor.
/// Addresses: Requirement 2
pub struct SortCommand;

impl SortCommand {
    /// Execute SORT on the resolved scope.
    /// Returns the undo record for transaction recording.
    /// Addresses: Requirement 2 AC 1–13
    pub fn execute(
        document: &mut Document,
        params: &SortParams,
        bounds: Option<ActiveBounds>,
        visible_lines: &[u64],
        tagged_lines: &[u64],
    ) -> Result<SortUndoRecord, NavigationError>;

    /// Parse SORT command arguments into SortParams.
    pub fn parse_args(args: &[CommandToken]) -> Result<SortParams, NavigationError>;
}
```

### ScrollCommands — Public API

```rust
/// Viewport scroll command executors.
/// Addresses: Requirement 3
pub struct ScrollCommands;

impl ScrollCommands {
    /// Scroll viewport up by page (visible_count - overlap).
    /// Addresses: Requirement 3 AC 1
    pub fn up_page(viewport: &mut ViewportModel, config: &NavigationConfig);

    /// Scroll viewport up by n lines.
    /// Addresses: Requirement 3 AC 2
    pub fn up_lines(viewport: &mut ViewportModel, n: u64);

    /// Scroll viewport down by page (visible_count - overlap).
    /// Addresses: Requirement 3 AC 3
    pub fn down_page(viewport: &mut ViewportModel, config: &NavigationConfig);

    /// Scroll viewport down by n lines.
    /// Addresses: Requirement 3 AC 4
    pub fn down_lines(viewport: &mut ViewportModel, n: u64);

    /// Scroll viewport left by configured amount.
    /// Addresses: Requirement 3 AC 5
    pub fn left_default(viewport: &mut ViewportModel, config: &NavigationConfig);

    /// Scroll viewport left by n columns.
    /// Addresses: Requirement 3 AC 6
    pub fn left_columns(viewport: &mut ViewportModel, n: u64);

    /// Scroll viewport right by configured amount.
    /// Addresses: Requirement 3 AC 7
    pub fn right_default(viewport: &mut ViewportModel, config: &NavigationConfig);

    /// Scroll viewport right by n columns.
    /// Addresses: Requirement 3 AC 8
    pub fn right_columns(viewport: &mut ViewportModel, n: u64);

    /// Scroll to first line and update cursor.
    /// Addresses: Requirement 3 AC 9, AC 16
    pub fn top(viewport: &mut ViewportModel, cursor: &mut CursorModel);

    /// Scroll to last page and update cursor.
    /// Addresses: Requirement 3 AC 10, AC 16
    pub fn bottom(
        viewport: &mut ViewportModel,
        cursor: &mut CursorModel,
        doc_line_count: u64,
    );
}
```

### ParagraphNav — Public API

```rust
/// Paragraph navigation executor.
/// Addresses: Requirement 6
pub struct ParagraphNav;

impl ParagraphNav {
    /// Move caret to the previous paragraph boundary.
    /// Addresses: Requirement 6 AC 1, AC 4
    pub fn paragraph_up(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        display_mapper: Option<&dyn DisplayLineMapper>,
        selection: SelectionModifier,
    );

    /// Move caret to the next paragraph boundary.
    /// Addresses: Requirement 6 AC 2, AC 5
    pub fn paragraph_down(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        display_mapper: Option<&dyn DisplayLineMapper>,
        selection: SelectionModifier,
    );

    /// Check if a line is a paragraph boundary (empty or whitespace-only).
    /// Addresses: Requirement 6 AC 3
    pub fn is_paragraph_boundary(line_content: &[u8]) -> bool;
}
```

### WordNav — Public API

```rust
/// Word navigation executor.
/// Addresses: Requirement 7
pub struct WordNav;

impl WordNav {
    /// Move caret to the start of the previous word.
    /// Addresses: Requirement 7 AC 2
    pub fn word_left(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        classifier: &CharClassifier,
        selection: SelectionModifier,
    );

    /// Move caret to the start of the next word.
    /// Addresses: Requirement 7 AC 3
    pub fn word_right(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        classifier: &CharClassifier,
        selection: SelectionModifier,
    );

    /// Move caret to the end of the current or next word.
    /// Addresses: Requirement 7 AC 4
    pub fn word_end_right(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        classifier: &CharClassifier,
        selection: SelectionModifier,
    );
}
```

### WordPartNav — Public API

```rust
/// Word-part (sub-word / camelCase) navigation executor.
/// Addresses: Requirement 8
pub struct WordPartNav;

impl WordPartNav {
    /// Move caret to the previous sub-word boundary.
    /// Addresses: Requirement 8 AC 1, AC 3
    pub fn word_part_left(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        classifier: &CharClassifier,
        selection: SelectionModifier,
    );

    /// Move caret to the next sub-word boundary.
    /// Addresses: Requirement 8 AC 2, AC 4
    pub fn word_part_right(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        classifier: &CharClassifier,
        selection: SelectionModifier,
    );

    /// Detect the type of sub-word boundary at a given position.
    /// Addresses: Requirement 8 AC 5
    pub fn detect_boundary(
        document: &Document,
        position: u64,
        direction: WordDirection,
    ) -> WordPartBoundary;
}
```

### VerticalCaretNav — Public API

```rust
/// Vertical caret movement with column affinity.
/// Addresses: Requirement 9
pub struct VerticalCaretNav;

impl VerticalCaretNav {
    /// Move caret up one line, maintaining column affinity.
    /// Addresses: Requirement 9 AC 1, AC 3, AC 4, AC 8
    pub fn line_up(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        selection: SelectionModifier,
    );

    /// Move caret down one line, maintaining column affinity.
    /// Addresses: Requirement 9 AC 1, AC 3, AC 4, AC 9
    pub fn line_down(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        selection: SelectionModifier,
    );

    /// Move caret up one page, maintaining column affinity.
    /// Addresses: Requirement 9 AC 6, AC 8
    pub fn page_up(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        selection: SelectionModifier,
    );

    /// Move caret down one page, maintaining column affinity.
    /// Addresses: Requirement 9 AC 7, AC 9
    pub fn page_down(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        selection: SelectionModifier,
    );
}
```

### DocStartEndNav — Public API

```rust
/// Document start/end navigation.
/// Addresses: Requirement 10
pub struct DocStartEndNav;

impl DocStartEndNav {
    /// Move caret to position 0 (first char of first line), scroll viewport to top.
    /// Addresses: Requirement 10 AC 1, AC 5
    pub fn document_start(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        selection: SelectionModifier,
    );

    /// Move caret to end of last line, scroll viewport to last page.
    /// Addresses: Requirement 10 AC 2, AC 6
    pub fn document_end(
        cursor: &mut CursorModel,
        viewport: &mut ViewportModel,
        document: &Document,
        selection: SelectionModifier,
    );
}
```

### CharClassifier — Public API

```rust
impl CharClassifier {
    /// Classify a single character.
    pub fn classify(&self, ch: char) -> CharacterClass;

    /// Classify a byte (ASCII fast path).
    pub fn classify_byte(&self, byte: u8) -> CharacterClass;

    /// Check if a character is classified as Word.
    pub fn is_word_char(&self, ch: char) -> bool;

    /// Check if a character is classified as Space.
    pub fn is_space(&self, ch: char) -> bool;
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-navigation-commands crate.
/// Formatted per Error Message Standards (Req 8): `[navigation] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NavigationError {
    /// Line number out of range for LOCATE.
    /// Addresses: Requirement 1 AC 2
    #[error("[navigation] LOCATE: line number out of range")]
    LineOutOfRange {
        requested: u64,
        max_line: u64,
    },

    /// Label not found for LOCATE.
    /// Addresses: Requirement 1 AC 4
    #[error("[navigation] LOCATE: label not found: {label}")]
    LabelNotFound {
        label: String,
    },

    /// Invalid bounds values.
    /// Addresses: Requirement 5 AC 13
    #[error("[navigation] BOUNDS: invalid bounds: left must be >= 1 and right must be > left")]
    InvalidBounds {
        left: u64,
        right: u64,
    },

    /// Nothing to sort (scope has 0 or 1 lines).
    /// Addresses: Requirement 2 AC 13
    #[error("[navigation] SORT: nothing to sort")]
    NothingToSort,

    /// Invalid SORT arguments.
    #[error("[navigation] SORT: {reason}")]
    InvalidSortArgs {
        reason: String,
    },

    /// Configuration value invalid (fallback to default applied).
    /// Addresses: Requirement 18 AC 5
    #[error("[navigation] config: invalid value for {key}, using default")]
    InvalidConfig {
        key: String,
    },
}
```

---

## Integration Points

### Upstream Dependencies

| Crate | What This Crate Uses |
|-------|---------------------|
| `ff-command` | `CommandRegistry::register()`, `CommandId`, `CommandMetadata`, `CommandHandler` trait, `CommandParams`, `ExecutionContext`, `CommandResult` |
| `ff-viewport-scrolling` | `ViewportModel` (top_line, visible_count, horizontal_offset mutations, clamping), `CursorModel` (cursor_line, cursor_column, column_affinity) |
| `ff-document-model` | `Document::line_count()`, `Document::line_start()`, `Document::line_end()`, `Document::get_range()`, `Document::character_at()`, line content access for paragraph detection and word classification |
| `ff-display-line-mapping` | `DisplayLineMapper` trait — `is_visible()` method for skipping excluded lines during paragraph nav |
| `ff-undo-redo-transactions` | Transaction recording API for SORT undo record |
| `ff-configuration-system` | TOML key reads for `editor.navigation.*` and `editor.bounds.*` settings |
| `ff-command-semantics` | `CommandToken` type for parsing SORT arguments; `SessionState` for pending CC block queries |
| `ff-logging` | Warning emission for invalid configuration values |

### Downstream Consumers

| Crate | What It Reads From This Crate |
|-------|-------------------------------|
| `ff-find-and-replace` | `BoundsManager::active_bounds()` and `bounds_affect_find()` to constrain FIND column range |
| `ff-line-commands` | `BoundsManager::active_bounds()` for bounds-aware shift operations |
| `ff-command-semantics` | Uses `ColsManager` and `BoundsManager` for RESET command handling |
| `ff-desktop` (shell) | Reads `ColsManager::active_cols_lines()` and `BoundsManager` BNDS_Line positions for rendering overlays |

### Command Registration Table

| Command Name | Aliases | Undoable | Modes | Handler |
|-------------|---------|----------|-------|---------|
| LOCATE | LOC | No | Browse, Edit | `LocateCommand` |
| SORT | — | Yes | Edit only | `SortCommand` |
| COLS | — | No | Browse, Edit | `ColsManager` |
| BOUNDS | BNDS | No | Browse, Edit | `BoundsManager` |
| UP | — | No | Browse, Edit | `ScrollCommands::up_*` |
| DOWN | — | No | Browse, Edit | `ScrollCommands::down_*` |
| LEFT | — | No | Browse, Edit | `ScrollCommands::left_*` |
| RIGHT | — | No | Browse, Edit | `ScrollCommands::right_*` |
| TOP | — | No | Browse, Edit | `ScrollCommands::top` |
| BOTTOM | BOT | No | Browse, Edit | `ScrollCommands::bottom` |
| PARA_UP | — | No | Browse, Edit | `ParagraphNav::paragraph_up` |
| PARA_DOWN | — | No | Browse, Edit | `ParagraphNav::paragraph_down` |
| WORD_LEFT | — | No | Browse, Edit | `WordNav::word_left` |
| WORD_RIGHT | — | No | Browse, Edit | `WordNav::word_right` |
| WORD_PART_LEFT | — | No | Browse, Edit | `WordPartNav::word_part_left` |
| WORD_PART_RIGHT | — | No | Browse, Edit | `WordPartNav::word_part_right` |
| DOC_START | — | No | Browse, Edit | `DocStartEndNav::document_start` |
| DOC_END | — | No | Browse, Edit | `DocStartEndNav::document_end` |

---

## Correctness Properties

These properties can be verified via property-based tests using the `proptest` crate.

### Property 1: LOCATE Clamping — Valid Targets Always Succeed

**Statement:** For any line number `n` where `1 <= n <= document.line_count()`, LOCATE n SHALL succeed and set `top_line = n` and `cursor_line = n`.

**Validates: Requirement 1 AC 1, AC 6**

```
∀ n ∈ [1, line_count]: locate_line(n) → Ok ∧ viewport.top_line == n ∧ cursor.cursor_line == n
```

### Property 2: LOCATE Out-of-Range — Invalid Targets Always Error

**Statement:** For any line number `n` where `n < 1` or `n > document.line_count()`, LOCATE n SHALL return `NavigationError::LineOutOfRange` and viewport/cursor SHALL remain unchanged.

**Validates: Requirement 1 AC 2**

```
∀ n ∉ [1, line_count]: locate_line(n) → Err(LineOutOfRange) ∧ viewport unchanged ∧ cursor unchanged
```

### Property 3: SORT Stability — Equal Keys Preserve Order

**Statement:** For any sequence of lines where multiple lines have equal sort keys, the relative order of those lines SHALL be unchanged after SORT.

**Validates: Requirement 2 AC 8**

```
∀ lines, key: sort_stable(lines, key) → ∀ i < j where key(lines[i]) == key(lines[j]): position(i) < position(j) in output
```

### Property 4: SORT Bounds Intersection

**Statement:** When active bounds [L, R] are set and explicit column range [C1, C2] is given, the effective sort key SHALL use columns [max(L, C1), min(R, C2)]. If max(L, C1) > min(R, C2), the effective range is empty and the sort uses the empty string as key for all lines (all equal → stable order preserved).

**Validates: Requirement 2 AC 9, AC 10**

```
∀ bounds, range: effective_range == (max(bounds.left, range.0), min(bounds.right, range.1))
```

### Property 5: Scroll Clamping Invariants

**Statement:** After any scroll command (UP, DOWN, LEFT, RIGHT, TOP, BOTTOM), `top_line` SHALL be in [1, max_top_line] and `horizontal_offset` SHALL be >= 0.

**Validates: Requirement 3 AC 11, AC 12, AC 13**

```
∀ scroll_command: 1 <= viewport.top_line <= max_top_line ∧ viewport.horizontal_offset >= 0
```

### Property 6: Bounds Validation Invariant

**Statement:** ActiveBounds can only exist with `left >= 1` and `right > left`. Any attempt to set bounds violating this yields an error and no state change.

**Validates: Requirement 5 AC 13**

```
∀ (left, right): (left < 1 ∨ right <= left) → set_bounds(left, right) == Err ∧ active_bounds unchanged
```

### Property 7: Paragraph Boundary Definition Consistency

**Statement:** A line is a paragraph boundary if and only if it is empty or contains only whitespace characters. `is_paragraph_boundary` agrees with this definition for all byte sequences.

**Validates: Requirement 6 AC 3**

```
∀ line_bytes: is_paragraph_boundary(line_bytes) ⟺ line_bytes.iter().all(|b| b.is_ascii_whitespace())
```

### Property 8: Word Navigation Never Exceeds Document Bounds

**Statement:** After any word navigation operation (left, right, end-right), the caret position SHALL be in [0, document_length] inclusive.

**Validates: Requirement 7 AC 6, AC 7**

```
∀ word_nav: 0 <= caret_position <= document.length()
```

### Property 9: Word-Part Navigation Detects All Boundary Types

**Statement:** For any string containing camelCase identifiers, word-part navigation SHALL stop at every lower→upper transition, uppercase-run-before-lower transition, alpha↔non-alpha transition, and digit↔alpha transition.

**Validates: Requirement 8 AC 5**

```
∀ identifier with known boundaries: word_part_right traversal visits all expected boundary positions
```

### Property 10: Column Affinity Preservation During Vertical Movement

**Statement:** When the caret moves vertically and the target line is at least as long as the column_affinity value, the caret SHALL be placed at the column_affinity position. When the target is shorter, the caret is clamped but column_affinity is NOT modified.

**Validates: Requirement 9 AC 1, AC 3, AC 4**

```
∀ vertical_move:
  target_line_length >= affinity → cursor_column == affinity
  target_line_length < affinity → cursor_column == target_line_length ∧ affinity unchanged
```

### Property 11: Column Affinity Update on Horizontal Movement

**Statement:** When the caret moves horizontally (word nav, char nav, home, end), column_affinity SHALL be updated to the new cursor_column.

**Validates: Requirement 9 AC 2**

```
∀ horizontal_move: column_affinity == cursor_column (after move)
```

### Property 12: Document Start/End Positions

**Statement:** DOC_START always places caret at (line=1, column=1) with top_line=1. DOC_END always places caret at (last_line, last_line_length) with viewport showing the last page.

**Validates: Requirement 10 AC 1, AC 2**

```
doc_start → cursor == (1, 1) ∧ top_line == 1
doc_end → cursor == (last_line, last_line_len) ∧ last_line is visible
```

### Property 13: COLS Toggle Idempotence

**Statement:** Toggling COLS at the same position twice returns to the original state (no COLS_Lines at that position).

**Validates: Requirement 4 AC 4**

```
∀ pos: toggle(pos); toggle(pos) → cols_at(pos) == ∅
```

### Property 14: Navigation Commands Non-Undoable Invariant

**Statement:** No non-undoable navigation command (all except SORT) SHALL produce an UndoRecord or modify the undo stack.

**Validates: Requirements 1 AC 5, 3 AC 14, 4 AC 11, 5 AC 12, 6 AC 8, 7 AC 11, 8 AC 8, 10 AC 4, 19 AC 1**

```
∀ non_sort_command: execute(cmd) → undo_record == None
```

---

## Design Decisions

### D1: Bounds as Shared Session State

Active Bounds are session-level state (not document-level) because they affect command behaviour across multiple crates (FIND, SORT, shift). The `BoundsManager` exposes a public query API so other crates can read bounds without taking a dependency on the full navigation-commands crate — they depend only on the `ActiveBounds` type re-exported at crate root.

**Rationale:** Bounds affect FIND (in ff-find-and-replace) and shift (in ff-line-commands). A shared API avoids circular dependencies while keeping bounds ownership in a single location.

### D2: COLS/BNDS as Display Artifacts, Not Document Lines

COLS_Lines and BNDS_Lines are never inserted into the document model. They exist only in the display layer as overlay artifacts tracked by `ColsManager` and `BoundsManager`. This avoids polluting document operations (SORT, DELETE, line count queries) with synthetic lines.

**Rationale:** ISPF COLS/BNDS are visual-only. Mixing them into the document buffer would require filtering them from every line operation and would break line-number-based addressing.

### D3: SORT Uses Stable Sort Algorithm

The SORT implementation uses Rust's `sort_by` (which is a stable merge sort) to guarantee that lines with equal keys retain their original order. This matches ISPF behaviour and user expectations.

**Rationale:** Stability is explicitly required (Req 2.8) and is the only reasonable behaviour for a columnar sort where many lines may have identical keys.

### D4: Character Classification Defaults Follow Scintilla

The default character class table classifies ASCII alphanumeric as Word, ASCII whitespace as Space, LF/CR as NewLine, and everything else as Punctuation. Unicode code points >= 0x80 use Unicode general categories (Letter/Number → Word, Separator → Space, otherwise Punctuation).

**Rationale:** This matches Scintilla's well-tested defaults and provides familiar word navigation behaviour for developers.

### D5: Paragraph Boundary Skips Excluded Lines

Paragraph navigation treats excluded (hidden) lines as non-existent for boundary detection. This means if a block of excluded lines separates two paragraphs, the navigation will not stop at those hidden lines.

**Rationale:** Excluded lines are logically hidden from the user. Stopping at invisible boundaries would be confusing. This matches the ISPF model where excluded lines don't participate in navigation.

### D6: Selection Extension as a Modifier Pattern

All navigation operations accept a `SelectionModifier` parameter rather than having separate "move" and "extend" variants. This halves the number of public functions while keeping the API explicit.

**Rationale:** The Move vs. Extend distinction is orthogonal to the navigation direction. A single parameter avoids combinatorial explosion of API methods.

---

## Configuration Keys

| TOML Key | Type | Default | Description | Requirement |
|----------|------|---------|-------------|-------------|
| `editor.navigation.horizontal_scroll_columns` | u64 | 8 | Columns scrolled by LEFT/RIGHT without argument | 18.1 |
| `editor.navigation.page_overlap_lines` | u64 | 2 | Lines of overlap retained on page scroll | 18.2 |
| `editor.bounds.affect_find` | bool | false | Whether active Bounds restrict FIND | 18.3 |
| `editor.navigation.word_characters` | String | "" | Additional characters treated as word chars | 18.4 |

---

## Testing Strategy

| Test File | Coverage | Approach |
|-----------|----------|----------|
| `locate_tests.rs` | Req 1 (AC 1–6) | Unit tests for valid/invalid line numbers, label resolution |
| `sort_tests.rs` | Req 2 (AC 1–13) | Property tests for stability, bounds intersection, scope filtering |
| `cols_tests.rs` | Req 4 (AC 1–11) | Unit tests for toggle, insert, reset, formatting |
| `bounds_tests.rs` | Req 5 (AC 1–15) | Property tests for validation, set/clear, query API |
| `scroll_tests.rs` | Req 3 (AC 1–16) | Property tests for clamping, page overlap, top/bottom |
| `paragraph_tests.rs` | Req 6 (AC 1–9) | Property tests for boundary detection, excluded-line skipping |
| `word_tests.rs` | Req 7 (AC 1–11) | Property tests for character class transitions, boundary behaviour |
| `word_part_tests.rs` | Req 8 (AC 1–8) | Property tests for camelCase/snake_case boundary detection |
| `vertical_caret_tests.rs` | Req 9 (AC 1–10) | Property tests for affinity preservation/update |
| `doc_nav_tests.rs` | Req 10 (AC 1–6) | Unit tests for start/end positions |
| `char_class_tests.rs` | Req 7.1, 7.9, 18.4 | Property tests for classification consistency |
| `integration.rs` | Reqs 11–19 | End-to-end: command registration, delegation stubs, metadata validation |
