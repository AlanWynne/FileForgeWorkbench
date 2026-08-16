# Design Document: Syntax Highlighting (`ff-syntax-highlighting`)

## Overview

The `ff-syntax-highlighting` crate is the **lexical highlighting engine** for FileForgeWorkbench. It assigns abstract style-slot indices (u8, 0–255) to character ranges based on lexical analysis of document content. The engine is GUI-independent — it produces style data consumed by the theme system for visual attribute resolution, never referencing colours or rendering APIs directly.

### Purpose

- Define the `Lexer` trait interface for language-specific tokenization
- Maintain per-document style buffers parallel to text content
- Perform incremental re-highlighting from the first modified line's state
- Provide demand-driven styling (`ensure_styled_to`) for viewport rendering
- Support up to 9 keyword sets per language with efficient lookup
- Support sub-style allocation for fine-grained token differentiation
- Compute fold levels alongside styling for the display-line-mapping
- Coordinate idle-time background styling through the idle-processing scheduler
- Provide property-based lexer configuration with hot-reload support
- Manage lexer lifecycle and document binding

### Position in Architecture

```
Wave 7 — Language and Highlighting

┌──────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                    │
│   Viewport Renderer — queries styled spans for painting       │
├──────────────────────────────────────────────────────────────┤
│  Downstream consumers:                                        │
│    ff-theme (Wave 6) — resolves style indices to colours      │
│    ff-text-decorations (Wave 6) — coexists with syntax styles │
│    ff-display-line-mapping (Wave 4) — consumes fold levels    │
│    ff-idle-processing (Wave 15) — coordinates bg styling      │
├──────────────────────────────────────────────────────────────┤
│         THIS CRATE: ff-syntax-highlighting ← Wave 7           │
│   Lexer trait, style buffer, incremental re-highlight,        │
│   keyword matching, sub-styles, fold levels, idle styling     │
├──────────────────────────────────────────────────────────────┤
│  Upstream:                                                    │
│    ff-language-service (Wave 7 peer) — language definitions,  │
│      keyword lists, comment patterns, lexer selection          │
│    ff-document-model (Wave 4) — text buffer, line indexing,   │
│      edit notifications                                        │
│    ff-configuration-system (Wave 2) — lexer properties        │
│    ff-plugin (Wave 2) — plugin-provided lexer registration    │
│    ff-logging (Wave 0) — structured diagnostics               │
├──────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                      │
└──────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: Zero GUI dependencies — produces abstract style-slot indices (u8); visual attribute resolution is the theme system's responsibility
- **Plugin Architecture (Req 3)**: Plugin-provided lexers registered at runtime via the lexer registry
- **Configuration Namespace (Req 5)**: Lexer properties live under `[syntax]` and per-language `[syntax.<language_id>]` TOML namespaces
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-syntax-highlighting`
- **Error Message Standards (Req 8)**: All errors follow `[syntax] operation: description` format

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Input Sources"
        DOC_EVT[ff-document-model<br/>Edit notifications: insert/delete]
        LANG_SVC[ff-language-service<br/>Language detection, keyword lists,<br/>comment patterns, TOML definitions]
        CFG_EVT[ff-configuration-system<br/>Lexer property hot-reload]
        PLUGIN[ff-plugin<br/>Runtime lexer registration]
        IDLE[ff-idle-processing<br/>Idle time slices]
    end

    subgraph "ff-syntax-highlighting"
        REG[LexerRegistry<br/>Maps language_id → Lexer impl]
        ENG[HighlightEngine<br/>Per-document orchestration]
        SB[StyleBuffer<br/>Parallel u8 array, O(1) access]
        PLS[PerLineState<br/>Lexer state at end of each line]
        FL[FoldLevelStore<br/>Per-line fold level + flags]
        KW[WordListStore<br/>Up to 9 keyword sets per lexer]
        SS[SubStyleAllocator<br/>Extended style index allocation]
        SC[StyleContext<br/>Lexer helper: chars, state, assignment]
        FC[FoldContext<br/>Fold helper: line levels, flags]
        DD[DemandDriver<br/>EnsureStyledTo logic]
        IS[IdleStylingTask<br/>Background styling increments]
    end

    subgraph "Downstream Consumers"
        THEME[ff-theme<br/>Style-slot → visual attributes]
        DLM[ff-display-line-mapping<br/>Fold level queries]
        RENDER[Viewport Renderer<br/>styled_spans for painting]
    end

    DOC_EVT --> ENG
    LANG_SVC --> REG
    LANG_SVC --> KW
    CFG_EVT --> ENG
    PLUGIN --> REG
    IDLE --> IS

    REG --> ENG
    ENG --> SB
    ENG --> PLS
    ENG --> FL
    ENG --> DD
    ENG --> IS
    KW --> SC
    SS --> SC

    SB --> RENDER
    FL --> DLM
    SB --> THEME
end
```

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| **LexerRegistry** | Maps language identifiers to `Lexer` trait objects; supports dynamic registration at runtime for plugin-provided lexers |
| **HighlightEngine** | Per-document orchestrator: binds lexer, manages style buffer, coordinates incremental re-highlighting and demand-driven styling |
| **StyleBuffer** | Parallel array of `u8` style-slot indices matching document length; O(1) positional access; synchronized with document edits |
| **PerLineState** | Stores `LexerState` at the end of each line; enables incremental re-highlighting from any line |
| **FoldLevelStore** | Stores fold level (12-bit) and fold flags per line; synchronized with document line count |
| **WordListStore** | Holds up to 9 `WordList` instances per lexer; hash-based O(1) keyword lookup with case-sensitivity per set |
| **SubStyleAllocator** | Manages allocation of contiguous style-index blocks from the extended range (above base styles) for sub-style differentiation |
| **StyleContext** | Helper struct for lexer authors: exposes current/next/prev chars, state transitions, style assignment, keyword matching |
| **FoldContext** | Helper struct for fold computation: exposes line content, level assignment, flag setting |
| **DemandDriver** | Implements `ensure_styled_to` logic: tracks styling position, invokes lexer forward to requested position |
| **IdleStylingTask** | Background styling coordinator: styles bounded line batches during idle slices, respects time budgets |

---

## Module Structure

```
crates/ff-syntax-highlighting/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── engine/
│   │   ├── mod.rs                  # Engine re-exports
│   │   ├── highlight_engine.rs     # HighlightEngine: per-document orchestrator
│   │   ├── demand_driver.rs        # DemandDriver: ensure_styled_to logic
│   │   └── idle_styling.rs         # IdleStylingTask: background styling
│   ├── lexer/
│   │   ├── mod.rs                  # Lexer trait, registry re-exports
│   │   ├── traits.rs              # Lexer trait definition
│   │   ├── registry.rs            # LexerRegistry: language_id → Lexer mapping
│   │   └── lifecycle.rs           # Lexer binding/unbinding, document association
│   ├── style/
│   │   ├── mod.rs                  # Style re-exports
│   │   ├── buffer.rs              # StyleBuffer: parallel u8 array
│   │   ├── context.rs             # StyleContext: lexer helper struct
│   │   └── sub_styles.rs          # SubStyleAllocator: extended range management
│   ├── state/
│   │   ├── mod.rs                  # State re-exports
│   │   └── per_line.rs            # PerLineState: lexer state persistence
│   ├── fold/
│   │   ├── mod.rs                  # Fold re-exports
│   │   ├── context.rs             # FoldContext: fold helper struct
│   │   └── store.rs               # FoldLevelStore: per-line fold data
│   ├── keywords/
│   │   ├── mod.rs                  # Keyword re-exports
│   │   └── word_list.rs           # WordList: hash-based keyword storage
│   ├── types.rs                    # Shared types: StyleSlotIndex, LexerState, etc.
│   └── error.rs                    # SyntaxHighlightError enum
└── tests/
    ├── style_buffer_tests.rs       # StyleBuffer synchronization tests
    ├── incremental_tests.rs        # Incremental re-highlighting tests
    ├── demand_driver_tests.rs      # EnsureStyledTo tests
    ├── keyword_tests.rs            # WordList lookup and case-sensitivity tests
    ├── sub_style_tests.rs          # Sub-style allocation tests
    ├── fold_level_tests.rs         # Fold level computation tests
    ├── idle_styling_tests.rs       # Idle-time styling tests
    ├── lifecycle_tests.rs          # Lexer binding/unbinding tests
    ├── property_tests.rs           # Property-based tests (proptest)
    └── integration.rs              # End-to-end highlighting pipeline tests
```

---

## Data Models

### StyleSlotIndex

```rust
/// A style-slot index (0–255) assigned to character positions by the lexer.
/// The theme system resolves each index to visual attributes.
/// Addresses: Requirement 2, criterion 2.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StyleSlotIndex(pub u8);

impl StyleSlotIndex {
    /// The default/unstyled index.
    pub const DEFAULT: Self = Self(0);

    /// Maximum valid index.
    pub const MAX: Self = Self(255);

    /// Get the raw u8 value.
    pub fn value(self) -> u8 {
        self.0
    }
}

impl Default for StyleSlotIndex {
    fn default() -> Self {
        Self::DEFAULT
    }
}
```

### LexerState

```rust
/// An opaque integer representing the lexer's parsing state at a position.
/// Stored per-line for incremental re-highlighting.
/// Addresses: Requirement 3, criterion 3.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LexerState(pub i32);

impl LexerState {
    /// Initial state for the beginning of a document or unknown state.
    pub const INITIAL: Self = Self(0);
}

impl Default for LexerState {
    fn default() -> Self {
        Self::INITIAL
    }
}
```

### BytePosition

```rust
/// A byte offset into the document text buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytePosition(pub usize);
```

### LineNumber

```rust
/// A zero-based line index into the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineNumber(pub usize);
```

### HighlightSpan

```rust
/// A contiguous range of characters sharing the same style-slot index.
/// Produced by styled_spans() for the viewport renderer.
/// Addresses: Requirement 2, criterion 2.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    /// Byte offset of the span start.
    pub start: BytePosition,
    /// Byte offset of the span end (exclusive).
    pub end: BytePosition,
    /// The style-slot index for this span.
    pub style: StyleSlotIndex,
}
```

### FoldFlags

```rust
/// Flags associated with a line's fold level.
/// Addresses: Requirement 8, criterion 8.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FoldFlags(u8);

impl FoldFlags {
    /// No flags set.
    pub const NONE: Self = Self(0);
    /// Line is a fold header (begins a foldable region).
    pub const FOLD_HEADER: Self = Self(1 << 0);
    /// Line contains only whitespace.
    pub const FOLD_WHITESPACE: Self = Self(1 << 1);

    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }

    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }
}

impl Default for FoldFlags {
    fn default() -> Self {
        Self::NONE
    }
}
```

### FoldLevel

```rust
/// A 12-bit fold level (0–4095) representing nesting depth at end of line.
/// Addresses: Requirement 8, criterion 8.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FoldLevel(u16);

impl FoldLevel {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(4095);

    /// Create a fold level, clamping to [0, 4095].
    pub fn new(level: u16) -> Self {
        Self(level.min(4095))
    }

    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for FoldLevel {
    fn default() -> Self {
        Self::MIN
    }
}
```

### KeywordSetDescriptor

```rust
/// Metadata about a keyword set supported by a lexer.
/// Addresses: Requirement 1, criterion 1.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordSetDescriptor {
    /// Set index (0–8).
    pub index: u8,
    /// Human-readable name (e.g., "Primary keywords", "Type names").
    pub name: String,
    /// Description of what this keyword set represents.
    pub description: String,
}
```

### KeywordSetIndex

```rust
/// Index identifying which keyword set (0–8) matched.
/// Addresses: Requirement 5, criterion 5.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeywordSetIndex(pub u8);

impl KeywordSetIndex {
    /// Maximum supported keyword set index.
    pub const MAX: u8 = 8;

    pub fn new(index: u8) -> Option<Self> {
        if index <= Self::MAX {
            Some(Self(index))
        } else {
            None
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }
}
```

### WordList

```rust
/// Hash-based keyword storage for O(1) average-case lookup during lexing.
/// Addresses: Requirement 5, criterion 5.3
pub struct WordList {
    /// Keywords stored in a HashSet for fast lookup.
    words: HashSet<String>,
    /// Whether lookups are case-insensitive.
    case_insensitive: bool,
    /// The style-slot index assigned when a keyword matches.
    style: StyleSlotIndex,
}

impl WordList {
    pub fn new(style: StyleSlotIndex, case_insensitive: bool) -> Self;

    /// Add a keyword to the list.
    pub fn add(&mut self, word: &str);

    /// Remove a keyword from the list.
    pub fn remove(&mut self, word: &str) -> bool;

    /// Check if an identifier matches a keyword in this set.
    /// Performs case-folded comparison when case_insensitive is true.
    /// Addresses: Requirement 5, criteria 5.6–5.7
    pub fn contains(&self, word: &str) -> bool;

    /// Get the style-slot index for matches in this set.
    pub fn style(&self) -> StyleSlotIndex;

    /// Get the number of keywords in this set.
    pub fn len(&self) -> usize;

    /// Check if the word list is empty.
    pub fn is_empty(&self) -> bool;
}
```

### SubStyleRange

```rust
/// A contiguous block of style-slot indices allocated for sub-style differentiation.
/// Addresses: Requirement 7, criterion 7.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubStyleRange {
    /// The base style this sub-style range belongs to.
    pub base_style: StyleSlotIndex,
    /// First allocated style index in the range.
    pub start: StyleSlotIndex,
    /// Number of allocated indices.
    pub count: u8,
}

impl SubStyleRange {
    /// Get the style index at position `offset` within this range.
    pub fn index_at(&self, offset: u8) -> Option<StyleSlotIndex> {
        if offset < self.count {
            Some(StyleSlotIndex(self.start.0 + offset))
        } else {
            None
        }
    }

    /// Check if a style index falls within this sub-style range.
    pub fn contains(&self, style: StyleSlotIndex) -> bool {
        style.0 >= self.start.0 && style.0 < self.start.0 + self.count
    }
}
```

### PropertyDescriptor

```rust
/// Metadata about a lexer property for auto-discovery.
/// Addresses: Requirement 10, criterion 10.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyDescriptor {
    /// Property key (e.g., "fold.comment").
    pub name: String,
    /// Property type hint.
    pub property_type: PropertyType,
    /// Human-readable description.
    pub description: String,
    /// Default value as string.
    pub default_value: String,
}

/// Type hint for lexer properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyType {
    /// String value.
    String,
    /// Integer value.
    Integer,
    /// Boolean value ("0"/"1" or "true"/"false").
    Boolean,
}
```

### IdleStylingConfig

```rust
/// Configuration for idle-time background styling.
/// Addresses: Requirement 9, criteria 9.3–9.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdleStylingConfig {
    /// Maximum lines to style per idle slice.
    pub lines_per_slice: usize,
    /// Maximum time budget per idle slice in milliseconds.
    pub time_budget_ms: u32,
}

impl Default for IdleStylingConfig {
    fn default() -> Self {
        Self {
            lines_per_slice: 256,
            time_budget_ms: 10,
        }
    }
}
```

---

## Public API Surface

### Lexer Trait

```rust
/// The core lexer trait that language-specific implementations must satisfy.
/// Each supported language has one or more Lexer implementations.
/// Addresses: Requirement 1
pub trait Lexer: Send + Sync {
    /// Returns the unique identifier of this lexer (e.g., "rust", "cpp", "cobol").
    /// Addresses: Requirement 1, criterion 1.3
    fn name(&self) -> &str;

    /// Perform lexical analysis on the specified text range, assigning
    /// StyleSlotIndex values to each character position via the StyleContext.
    /// Addresses: Requirement 1, criterion 1.1
    fn style_text(&self, context: &mut StyleContext);

    /// Compute FoldLevel values for each line within the specified range.
    /// Addresses: Requirement 1, criterion 1.2
    fn fold_text(&self, context: &mut FoldContext);

    /// Returns the default style-slot index for unstyled text in this language.
    /// Addresses: Requirement 1, criterion 1.4
    fn default_style(&self) -> StyleSlotIndex;

    /// Returns metadata about the keyword sets this lexer supports.
    /// Addresses: Requirement 1, criterion 1.5
    fn keyword_sets(&self) -> &[KeywordSetDescriptor];

    /// Returns the base style indices that support sub-style differentiation.
    /// Addresses: Requirement 1, criterion 1.6
    fn sub_style_bases(&self) -> &[StyleSlotIndex];

    /// Get a lexer-specific property value.
    /// Addresses: Requirement 1, criterion 1.7
    fn get_property(&self, key: &str) -> Option<&str>;

    /// Set a lexer-specific property value.
    /// Addresses: Requirement 1, criterion 1.7
    fn set_property(&mut self, key: &str, value: &str);

    /// Returns metadata about all supported properties for auto-discovery.
    /// Addresses: Requirement 10, criterion 10.6
    fn property_names(&self) -> &[PropertyDescriptor];

    /// Returns the number of base style indices this lexer uses.
    /// Addresses: Requirement 12, criterion 12.4
    fn style_slot_count(&self) -> u8;
}
```

### LexerRegistry

```rust
/// Registry mapping language identifiers to Lexer implementations.
/// Supports dynamic registration for plugin-provided lexers.
/// Addresses: Requirement 1, criterion 1.8
pub struct LexerRegistry { /* ... */ }

impl LexerRegistry {
    /// Create an empty registry.
    pub fn new() -> Self;

    /// Register a lexer factory for a language identifier.
    /// Returns the previous factory if one was registered for this language_id.
    pub fn register(
        &mut self,
        language_id: &str,
        factory: Box<dyn Fn() -> Box<dyn Lexer> + Send + Sync>,
    ) -> Option<Box<dyn Fn() -> Box<dyn Lexer> + Send + Sync>>;

    /// Unregister a lexer for a language identifier.
    pub fn unregister(&mut self, language_id: &str) -> bool;

    /// Create a new lexer instance for the given language.
    /// Returns None if no lexer is registered for this language_id.
    pub fn create_lexer(&self, language_id: &str) -> Option<Box<dyn Lexer>>;

    /// Check if a lexer is registered for the given language.
    pub fn has_lexer(&self, language_id: &str) -> bool;

    /// List all registered language identifiers.
    pub fn registered_languages(&self) -> Vec<&str>;
}
```

### SyntaxHighlighter Trait (Consumer-Facing)

```rust
/// The public trait exposed to consumers (viewport renderer, minimap, export).
/// Consumers depend on this trait rather than the concrete HighlightEngine.
/// Addresses: Requirement 11, criterion 11.4
pub trait SyntaxHighlighter: Send + Sync {
    /// Guarantee all text up to `position` has valid style data.
    /// Addresses: Requirement 4, criterion 4.1
    fn ensure_styled_to(&mut self, position: BytePosition);

    /// Returns the current end-of-styled-text position.
    /// Addresses: Requirement 4, criterion 4.4
    fn styling_position(&self) -> BytePosition;

    /// Get the style index at a specific byte position. O(1).
    /// Addresses: Requirement 2, criterion 2.3
    fn style_at(&self, position: BytePosition) -> StyleSlotIndex;

    /// Get contiguous styled spans within a range.
    /// Addresses: Requirement 2, criterion 2.4
    fn styled_spans(
        &self,
        start: BytePosition,
        end: BytePosition,
    ) -> Vec<HighlightSpan>;

    /// Get the fold level and flags for a specific line.
    /// Addresses: Requirement 8, criterion 8.5
    fn fold_level_at(&self, line: LineNumber) -> (FoldLevel, FoldFlags);

    /// Get fold levels for a range of lines (bulk query).
    /// Addresses: Requirement 15, criterion 15.6
    fn fold_level_range(
        &self,
        start_line: LineNumber,
        end_line: LineNumber,
    ) -> Vec<(LineNumber, FoldLevel, FoldFlags)>;

    /// Get the number of base style slots the active lexer uses.
    /// Addresses: Requirement 12, criterion 12.4
    fn style_slot_count(&self) -> u8;
}
```

### HighlightEngine

```rust
/// Per-document highlighting engine that implements SyntaxHighlighter.
/// Manages style buffer, per-line state, fold levels, and lexer binding.
/// Addresses: Requirements 2, 3, 4, 8, 11, 13
pub struct HighlightEngine { /* ... */ }

impl HighlightEngine {
    /// Create a new engine for a document with the given initial text length.
    /// Addresses: Requirement 11, criterion 11.6
    pub fn new(document_length: usize, line_count: usize) -> Self;

    /// Bind a lexer to this engine for a specific language.
    /// Populates keyword sets and properties from the language definition.
    /// Addresses: Requirement 13, criterion 13.1
    pub fn bind_lexer(
        &mut self,
        lexer: Box<dyn Lexer>,
        keyword_sets: &[Vec<String>],
        properties: &[(&str, &str)],
    );

    /// Unbind the current lexer, resetting all style data to default.
    /// Addresses: Requirement 13, criterion 13.3
    pub fn unbind_lexer(&mut self);

    /// Returns true if a lexer is currently bound.
    pub fn has_lexer(&self) -> bool;

    /// Notify the engine of a text insertion at the given position.
    /// Addresses: Requirement 2, criteria 2.6–2.7; Requirement 3, criterion 3.8
    pub fn notify_insert(
        &mut self,
        position: BytePosition,
        length: usize,
        lines_inserted: usize,
    );

    /// Notify the engine of a text deletion at the given position.
    /// Addresses: Requirement 2, criterion 2.8; Requirement 3, criterion 3.9
    pub fn notify_delete(
        &mut self,
        position: BytePosition,
        length: usize,
        lines_deleted: usize,
    );

    /// Update a keyword set at runtime.
    /// Addresses: Requirement 5, criterion 5.8
    pub fn set_keywords(&mut self, set_index: KeywordSetIndex, words: &[&str]);

    /// Update a lexer property at runtime.
    /// Addresses: Requirement 10, criterion 10.3
    pub fn set_lexer_property(&mut self, key: &str, value: &str);

    /// Allocate sub-styles for a base style.
    /// Addresses: Requirement 7, criterion 7.2
    pub fn allocate_sub_styles(
        &mut self,
        base_style: StyleSlotIndex,
        count: u8,
    ) -> Result<SubStyleRange, SyntaxHighlightError>;

    /// Free sub-styles for a base style.
    /// Addresses: Requirement 7, criterion 7.5
    pub fn free_sub_styles(&mut self, base_style: StyleSlotIndex);

    /// Get the base style for a sub-style index.
    /// Addresses: Requirement 7, criterion 7.7
    pub fn sub_style_base(
        &self,
        sub_style: StyleSlotIndex,
    ) -> Option<StyleSlotIndex>;

    /// Perform one idle styling increment. Returns true if more work remains.
    /// Addresses: Requirement 9, criterion 9.3
    pub fn idle_style_increment(
        &mut self,
        config: &IdleStylingConfig,
        text: &str,
    ) -> bool;

    /// Check if idle styling is complete (entire document styled).
    /// Addresses: Requirement 9, criterion 9.5
    pub fn is_fully_styled(&self) -> bool;
}

impl SyntaxHighlighter for HighlightEngine {
    // All trait methods implemented...
}
```

### StyleContext

```rust
/// Helper structure for lexer authors providing convenient character access,
/// state tracking, and style assignment methods.
/// Addresses: Requirement 14
pub struct StyleContext<'a> {
    // Internal fields: text slice, position, state, style buffer reference
}

impl<'a> StyleContext<'a> {
    /// Get the current character.
    /// Addresses: Requirement 14, criterion 14.1
    pub fn ch(&self) -> char;

    /// Get the next character (lookahead). Returns '\0' at document end.
    /// Addresses: Requirement 14, criteria 14.1, 14.9
    pub fn ch_next(&self) -> char;

    /// Get the previous character. Returns '\0' at document start.
    /// Addresses: Requirement 14, criterion 14.1
    pub fn ch_prev(&self) -> char;

    /// Get the current lexer state.
    /// Addresses: Requirement 14, criterion 14.1
    pub fn state(&self) -> LexerState;

    /// Get the byte position of the current token start.
    /// Addresses: Requirement 14, criterion 14.1
    pub fn start_position(&self) -> BytePosition;

    /// Assign style to characters from token start to current position,
    /// then transition to new_state.
    /// Addresses: Requirement 14, criterion 14.2
    pub fn set_state(&mut self, new_state: LexerState);

    /// Advance position by one character (handles multi-byte UTF-8).
    /// Addresses: Requirement 14, criterion 14.3
    pub fn forward(&mut self);

    /// Advance position by the specified number of bytes.
    /// Addresses: Requirement 14, criterion 14.4
    pub fn forward_bytes(&mut self, count: usize);

    /// Check current token against keyword sets. Returns matching set index.
    /// Addresses: Requirement 14, criterion 14.5
    pub fn match_keyword(&self, word_lists: &[WordList]) -> Option<KeywordSetIndex>;

    /// Returns true if current position is at the beginning of a line.
    /// Addresses: Requirement 14, criterion 14.6
    pub fn at_line_start(&self) -> bool;

    /// Returns true if current character is a line-ending character.
    /// Addresses: Requirement 14, criterion 14.7
    pub fn at_line_end(&self) -> bool;

    /// Returns true if there are more characters to process.
    /// Addresses: Requirement 14, criterion 14.8
    pub fn more(&self) -> bool;

    /// Get the text of the current token (from start to current position).
    pub fn current_token(&self) -> &str;
}
```

### FoldContext

```rust
/// Helper structure for fold-level computation.
/// Addresses: Requirement 8
pub struct FoldContext<'a> {
    // Internal fields: text slice, line range, fold level store reference
}

impl<'a> FoldContext<'a> {
    /// Set the fold level and flags for a line.
    /// Addresses: Requirement 8, criterion 8.1
    pub fn set_level(
        &mut self,
        line: LineNumber,
        level: FoldLevel,
        flags: FoldFlags,
    );

    /// Get the current fold level for a line (from previous computation).
    pub fn current_level(&self, line: LineNumber) -> FoldLevel;

    /// Get the text content of a line for analysis.
    pub fn line_text(&self, line: LineNumber) -> &str;

    /// Get the range of lines to process.
    pub fn line_range(&self) -> (LineNumber, LineNumber);
}
```

### SubStyleAllocator

```rust
/// Manages allocation of contiguous style-index blocks from the extended range.
/// The total budget is 256 indices shared between base styles and sub-styles.
/// Addresses: Requirement 7
pub struct SubStyleAllocator { /* ... */ }

impl SubStyleAllocator {
    /// Create an allocator with the given number of base styles already in use.
    pub fn new(base_style_count: u8) -> Self;

    /// Allocate a contiguous block of sub-style indices.
    /// Addresses: Requirement 7, criterion 7.2
    pub fn allocate(
        &mut self,
        base_style: StyleSlotIndex,
        count: u8,
    ) -> Result<SubStyleRange, SyntaxHighlightError>;

    /// Free all sub-style allocations for a base style.
    /// Addresses: Requirement 7, criterion 7.5
    pub fn free(&mut self, base_style: StyleSlotIndex);

    /// Get the base style for a given sub-style index.
    /// Addresses: Requirement 7, criterion 7.7
    pub fn base_for(&self, sub_style: StyleSlotIndex) -> Option<StyleSlotIndex>;

    /// Get the allocated range for a base style.
    pub fn range_for(&self, base_style: StyleSlotIndex) -> Option<&SubStyleRange>;

    /// Get the number of available style indices remaining.
    pub fn available(&self) -> u8;
}
```

### StyleBuffer

```rust
/// Parallel array of style-slot indices matching document text length.
/// Provides O(1) positional access.
/// Addresses: Requirement 2
pub struct StyleBuffer { /* ... */ }

impl StyleBuffer {
    /// Create a style buffer of the given length, initialized to DEFAULT (0).
    /// Addresses: Requirement 2, criterion 2.5
    pub fn new(length: usize) -> Self;

    /// Get the style at a byte position. O(1).
    /// Addresses: Requirement 2, criterion 2.3
    pub fn get(&self, position: BytePosition) -> StyleSlotIndex;

    /// Set the style for a byte range.
    /// Addresses: Requirement 2, criterion 2.2
    pub fn set_range(
        &mut self,
        start: BytePosition,
        end: BytePosition,
        style: StyleSlotIndex,
    );

    /// Insert default style values at a position (for text insertion).
    /// Addresses: Requirement 2, criterion 2.7
    pub fn insert(&mut self, position: BytePosition, count: usize);

    /// Remove style values at a position (for text deletion).
    /// Addresses: Requirement 2, criterion 2.8
    pub fn delete(&mut self, position: BytePosition, count: usize);

    /// Get the buffer length.
    pub fn len(&self) -> usize;

    /// Check if empty.
    pub fn is_empty(&self) -> bool;

    /// Get styled spans: coalesce adjacent positions with same style.
    /// Addresses: Requirement 2, criterion 2.4
    pub fn spans(
        &self,
        start: BytePosition,
        end: BytePosition,
    ) -> Vec<HighlightSpan>;
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-syntax-highlighting crate.
/// Formatted per Error Message Standards (Req 8): `[syntax] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SyntaxHighlightError {
    /// Sub-style allocation failed: not enough available indices.
    #[error("[syntax] allocate_sub_styles: requested {requested} indices for base style {base_style} but only {available} available (max 256 total)")]
    SubStyleAllocationExhausted {
        base_style: u8,
        requested: u8,
        available: u8,
    },

    /// Sub-style allocation failed: base style does not support sub-styles.
    #[error("[syntax] allocate_sub_styles: base style {base_style} is not declared as a sub-style base by the active lexer")]
    InvalidSubStyleBase { base_style: u8 },

    /// Lexer not bound: operation requires a bound lexer.
    #[error("[syntax] {operation}: no lexer bound to this document (language unknown or unset)")]
    NoLexerBound { operation: String },

    /// Invalid keyword set index (must be 0–8).
    #[error("[syntax] set_keywords: set index {index} is out of range (valid: 0–8)")]
    InvalidKeywordSetIndex { index: u8 },

    /// Position out of range for the style buffer.
    #[error("[syntax] {operation}: byte position {position} exceeds document length {length}")]
    PositionOutOfRange {
        operation: String,
        position: usize,
        length: usize,
    },

    /// Line number out of range for per-line data.
    #[error("[syntax] {operation}: line {line} exceeds document line count {line_count}")]
    LineOutOfRange {
        operation: String,
        line: usize,
        line_count: usize,
    },

    /// Lexer registration conflict.
    #[error("[syntax] register: lexer for language '{language_id}' is already registered")]
    LexerAlreadyRegistered { language_id: String },

    /// Configuration property has invalid value.
    #[error("[syntax] set_property: property '{key}' has invalid value '{value}' — {reason}")]
    InvalidPropertyValue {
        key: String,
        value: String,
        reason: String,
    },
}
```

---

## Integration Points

### With `ff-language-service` (Wave 7 — peer)

- **Consumed types**: `LanguageDefinition`, `LanguageId`, keyword lists, comment patterns, lexer selection
- **Data flow**: The language-service detects document language, provides keyword lists (up to 9 sets) and language properties. The syntax-highlighting engine consumes these to configure its bound lexer.
- **Dependency direction**: `ff-syntax-highlighting` depends on types/traits from `ff-language-service`
- **Key interactions**:
  - `LanguageService::detect_language(path)` → determines which lexer to bind
  - `LanguageDefinition::keywords(set_index)` → populates `WordList` instances
  - `LanguageDefinition::line_comment()` → informs lexer comment detection
  - `LanguageDefinition::block_comment_start/end()` → multi-line comment patterns
  - `LanguageDefinition::properties()` → lexer property initialization
  - Language change notification → triggers `unbind_lexer` + `bind_lexer`

### With `ff-document-model` (Wave 4 — upstream)

- **Consumed types**: Text buffer content, line indexing, edit notification events
- **Data flow**: The document-model provides text content for lexing and emits edit notifications (insert/delete with position, length, line count) that trigger incremental re-highlighting.
- **Dependency direction**: `ff-syntax-highlighting` depends on `ff-document-model` for text access
- **Key interactions**:
  - `Document::text_slice(start, end)` → provides text for `StyleContext`
  - `Document::line_start(line)` → byte position of line start for incremental rehighlight
  - `Document::line_count()` → synchronizes per-line state and fold level arrays
  - Edit events → `HighlightEngine::notify_insert()` / `notify_delete()`

### With `ff-configuration-system` (Wave 2 — upstream)

- **Consumed API**: Config loading, hot-reload callbacks, typed key access
- **Data flow**: Lexer properties and per-language configuration overrides are stored in the config system. Hot-reload events trigger property updates on the active lexer.
- **Dependency direction**: `ff-syntax-highlighting` depends on `ff-configuration-system`
- **Key config keys**:
  - `syntax.idle_lines_per_slice` → usize (default: 256)
  - `syntax.idle_time_budget_ms` → u32 (default: 10)
  - `syntax.<language_id>.<property_key>` → lexer-specific properties
- **Key interactions**:
  - Config hot-reload → `HighlightEngine::set_lexer_property()` → invalidate + re-highlight
  - Initial load → populate all lexer properties at bind time

### With `ff-plugin` (Wave 2 — upstream)

- **Consumed types**: Plugin lifecycle hooks, registration API
- **Data flow**: Plugins register new lexer implementations at runtime via the `LexerRegistry`.
- **Dependency direction**: `ff-syntax-highlighting` exposes registration API; plugins call it
- **Key interactions**:
  - `Plugin::on_activate()` → calls `LexerRegistry::register(language_id, factory)`
  - `Plugin::on_deactivate()` → calls `LexerRegistry::unregister(language_id)`
  - New registration → makes lexer available for language-service detection

### With `ff-theme` (Wave 6 — downstream consumer)

- **Provided data**: `StyleSlotIndex` values per character position
- **Data flow**: The theme system queries style indices from HighlightSpan data and resolves each to visual attributes (colour, bold, italic, underline, case). The highlighting engine never references colours.
- **Dependency direction**: `ff-theme` depends on style index types from this crate; no reverse dependency
- **Key interactions**:
  - `styled_spans(start, end)` → theme resolves each span's style index to visual attributes
  - `style_slot_count()` → theme provides defaults for unthemed indices
  - `sub_style_base(index)` → theme inherits base style attributes for sub-styles
  - Theme changes do NOT require re-highlighting (style indices are stable)

### With `ff-display-line-mapping` (Wave 4 — downstream consumer)

- **Provided data**: Fold levels and fold flags per line
- **Data flow**: The display-line-mapping queries fold levels exclusively from this crate to determine fold region boundaries and fold headers.
- **Dependency direction**: `ff-display-line-mapping` depends on this crate's fold-level API
- **Key interactions**:
  - `fold_level_at(line)` → single-line fold level query
  - `fold_level_range(start, end)` → bulk fold level query for efficiency
  - Fold-level-changed notification → display-line-mapping updates fold state incrementally

### With `ff-text-decorations` (Wave 6 — peer)

- **Relationship**: Coexistence — syntax styles and indicator decorations operate on the same text independently
- **Data flow**: No direct data exchange. Both produce visual attributes for the same character ranges; the rendering pipeline composites them based on the indicator's `under` property.
- **Key interactions**:
  - Re-highlighting does NOT invalidate indicators (Requirement 15, criterion 15.5)
  - Style data and indicator data are independently queryable for the same range

### With `ff-idle-processing` (Wave 15 — downstream coordinator)

- **Consumed API**: Idle work source registration, time slice grants
- **Data flow**: The idle-processing scheduler grants time slices during idle periods; the highlighting engine styles bounded line batches per slice.
- **Key interactions**:
  - Register as idle work source when document has unstyled regions
  - `idle_style_increment()` called per time slice → styles up to N lines
  - Deregister when `is_fully_styled()` returns true
  - Edit events → cancel current idle work, re-register from new styling position

### With `ff-logging` (Wave 0 — upstream)

- **Consumed API**: Structured logging macros
- **Data flow**: Diagnostic messages for lexer registration, property changes, errors
- **Key interactions**:
  - DEBUG: unknown property key set on lexer (Requirement 10, criterion 10.7)
  - WARN: sub-style allocation near capacity
  - INFO: lexer bound/unbound for document
  - ERROR: lexer panic recovery (if applicable)

---

## Correctness Properties

These properties are suitable for property-based testing using the `proptest` crate.

### Property 1: Style Buffer Length Invariant

**Statement**: After any sequence of insert and delete operations, the style buffer length always equals the document text length.

**Validates**: Requirement 2, criterion 2.6

```
∀ operations ∈ [insert(pos, len), delete(pos, len)]*:
  style_buffer.len() == document.len()
```

### Property 2: Style Buffer Insert Preserves Surrounding Styles

**Statement**: When text is inserted at position `p`, all style values at positions `< p` are unchanged, and all style values at positions `≥ p` (in the original buffer) are shifted to positions `≥ p + inserted_length` in the new buffer.

**Validates**: Requirement 2, criterion 2.7

```
∀ p, len, ∀ i < p: buffer_after[i] == buffer_before[i]
∀ i ≥ p: buffer_after[i + len] == buffer_before[i]
∀ i ∈ [p, p+len): buffer_after[i] == StyleSlotIndex::DEFAULT
```

### Property 3: Style Buffer Delete Preserves Surrounding Styles

**Statement**: When text is deleted at position `p` with length `len`, all style values at positions `< p` are unchanged, and all style values at positions `≥ p + len` (in the original buffer) are shifted to positions `≥ p` in the new buffer.

**Validates**: Requirement 2, criterion 2.8

```
∀ p, len, ∀ i < p: buffer_after[i] == buffer_before[i]
∀ i ≥ p + len: buffer_after[i - len] == buffer_before[i]
```

### Property 4: Incremental Re-Highlight State Convergence

**Statement**: After re-highlighting from a modified line, if the computed `LexerState` at the end of line `L` matches the stored `PerLineState` for line `L`, then re-highlighting stops and all subsequent per-line states remain unchanged.

**Validates**: Requirement 3, criterion 3.4

```
∀ edit at line M, ∀ L > M:
  computed_state(L) == stored_state(L) ⟹
    ∀ K > L: stored_state(K) unchanged ∧ styling[K] unchanged
```

### Property 5: EnsureStyledTo Idempotence

**Statement**: Calling `ensure_styled_to(p)` when `styling_position() >= p` performs no work and does not change any state.

**Validates**: Requirement 4, criterion 4.2

```
∀ p ≤ styling_position():
  ensure_styled_to(p) is a no-op
  styling_position() unchanged
  style_buffer unchanged
```

### Property 6: EnsureStyledTo Monotonicity

**Statement**: After `ensure_styled_to(p)`, `styling_position() >= p`. The styling position never decreases except when invalidated by an edit.

**Validates**: Requirement 4, criteria 4.1, 4.3

```
∀ p: ensure_styled_to(p) ⟹ styling_position() ≥ p
∀ t₁ < t₂ (no edits between): styling_position(t₂) ≥ styling_position(t₁)
```

### Property 7: WordList Case-Insensitive Matching

**Statement**: For a case-insensitive WordList containing keyword `k`, `contains(w)` returns true for any string `w` where `w.to_lowercase() == k.to_lowercase()`.

**Validates**: Requirement 5, criterion 5.7

```
∀ k ∈ word_list (case_insensitive=true), ∀ w:
  w.to_lowercase() == k.to_lowercase() ⟹ contains(w) == true
  w.to_lowercase() ≠ any keyword's lowercase ⟹ contains(w) == false
```

### Property 8: Keyword Set Priority Order

**Statement**: When an identifier matches keywords in multiple sets, the set with the lowest index wins (set 0 checked first, then set 1, etc.).

**Validates**: Requirement 5, criterion 5.4

```
∀ identifier matching sets S₁ and S₂ where S₁.index < S₂.index:
  assigned_style == S₁.style
```

### Property 9: Sub-Style Allocation Non-Overlap

**Statement**: All allocated sub-style ranges are non-overlapping — no two ranges share any style index.

**Validates**: Requirement 7, criteria 7.1–7.2

```
∀ ranges R₁, R₂ (R₁ ≠ R₂):
  R₁.start + R₁.count ≤ R₂.start ∨ R₂.start + R₂.count ≤ R₁.start
```

### Property 10: Sub-Style Total Budget

**Statement**: The sum of all base style count plus all allocated sub-style counts never exceeds 256.

**Validates**: Requirement 7, criterion 7.6

```
base_style_count + Σ(allocated_range.count) ≤ 256
```

### Property 11: Fold Level Clamping

**Statement**: For any input value, `FoldLevel::new(v).value()` is always in [0, 4095].

**Validates**: Requirement 8, criterion 8.2

```
∀ v ∈ u16: 0 ≤ FoldLevel::new(v).value() ≤ 4095
```

### Property 12: Idle Styling Progress

**Statement**: Each call to `idle_style_increment()` either advances `styling_position()` or returns false (indicating completion). It never moves styling_position backwards.

**Validates**: Requirement 9, criteria 9.3, 9.5

```
∀ call to idle_style_increment():
  let sp_before = styling_position();
  let more = idle_style_increment();
  more ⟹ styling_position() > sp_before
  ¬more ⟹ styling_position() == document_length
```

---

## Configuration Keys

All configuration keys live under reserved namespaces to avoid conflicts (Cross-cutting Req 5).

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `syntax.idle_lines_per_slice` | usize | `256` | Max lines styled per idle time slice |
| `syntax.idle_time_budget_ms` | u32 | `10` | Max milliseconds per idle slice |
| `syntax.<language_id>.fold.comment` | bool | `true` | Enable fold level computation in comments |
| `syntax.<language_id>.fold.compact` | bool | `false` | Compact fold levels (remove empty lines from fold end) |
| `syntax.<language_id>.<property_key>` | string | varies | Per-lexer property overrides |

Lexer-specific property keys are defined by each `Lexer` implementation via `property_names()` and documented in the language definition TOML files.

---

## Testing Strategy

### Unit Tests

- `StyleBuffer`: insert/delete synchronization, span coalescing, boundary conditions
- `PerLineState`: state storage/retrieval, line insert/delete synchronization
- `WordList`: contains with case-sensitive and case-insensitive modes, add/remove
- `SubStyleAllocator`: allocation, freeing, overflow detection, base-for lookup
- `FoldLevelStore`: level storage, flag manipulation, line insert/delete
- `HighlightEngine`: bind/unbind lifecycle, notify_insert/delete, ensure_styled_to
- `StyleContext`: character access, state transitions, keyword matching, boundary safety

### Property-Based Tests (proptest)

- Properties 1–12 as defined in Correctness Properties above
- Generators for random edit sequences (insert/delete at arbitrary positions)
- Generators for keyword sets with case-sensitivity variations
- Generators for sub-style allocation sequences
- Generators for fold level values (u16 clamped to 12-bit)

### Integration Tests

- Full pipeline: document edit → incremental re-highlight → style query
- Lexer binding: language detection → bind → style → unbind → rebind
- Idle styling: start → partial progress → edit interrupt → resume
- Keyword modification: runtime change → full re-highlight → verify new styles
- Multi-document: multiple engines operating independently

---

## Design Decisions and Rationale

### Decision 1: Parallel Style Buffer (Vec<u8>) Over Run-Length Encoding

The style buffer is a plain `Vec<u8>` with one entry per byte position, rather than a run-length-encoded or interval-tree structure. This ensures:
- O(1) random access for `style_at()` (critical for the viewport renderer)
- Simple synchronization with document insert/delete operations
- Predictable memory usage (1 byte per character)
- Avoids complex rebalancing during incremental updates
- Trade-off: higher memory usage for large files with uniform styling, but acceptable for typical editor workloads

### Decision 2: Per-Line State Enables Incremental Restart

Storing `LexerState` at the end of each line (not at arbitrary positions) provides:
- Natural alignment with document line operations (insert/delete lines)
- Sufficient granularity for most languages (state changes rarely mid-line without affecting the rest)
- Bounded memory: one `i32` per line
- Simple state convergence check: compare one value per line during re-highlight propagation

### Decision 3: Factory-Based Lexer Registry

The registry stores `Box<dyn Fn() -> Box<dyn Lexer>>` factories rather than pre-instantiated lexers because:
- Each document needs its own lexer instance (lexers carry mutable property state)
- Factories allow lazy instantiation — no resources consumed until a language is actually used
- Plugin lifecycle: factory remains valid even if the lexer's internal state is complex

### Decision 4: Trait-Based Consumer API (SyntaxHighlighter)

Consumers depend on the `SyntaxHighlighter` trait rather than `HighlightEngine` directly because:
- Enables mock implementations for testing downstream crates without full lexer setup
- Allows future alternative implementations (e.g., LSP-backed semantic highlighting)
- Maintains clean dependency inversion between layers

### Decision 5: Fold Levels Computed Alongside Styling But Separately Invocable

Fold computation is a separate `fold_text()` method (not merged into `style_text()`) because:
- Not all use cases need fold levels (e.g., minimap rendering, export)
- Fold computation may be deferred to idle time even when viewport styling is immediate
- Separating concerns keeps individual lexer methods simpler
- The engine can choose when to invoke fold computation (demand-driven or idle-time)

### Decision 6: Sub-Style Allocator With Contiguous Blocks

Sub-styles are allocated as contiguous blocks rather than individual indices because:
- Contiguous allocation enables efficient range-check for `sub_style_base()` lookup
- Matches the Scintilla model for compatibility with existing lexer designs
- Simplifies the theme system's inheritance logic (base + offset = sub-style visual)
- Trade-off: potential fragmentation if sub-styles are repeatedly allocated/freed with different counts
