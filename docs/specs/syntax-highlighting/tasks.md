# Implementation Plan: Syntax Highlighting (`ff-syntax-highlighting`)

## Overview

This plan covers the complete implementation of the `ff-syntax-highlighting` crate — the syntax highlighting engine for FileForgeWorkbench. The engine performs lexical analysis on document content, assigns style-slot indices to character ranges, supports incremental re-highlighting, demand-driven styling, keyword matching, sub-styles, fold-level assignment, and idle-time background styling.

This is a **Wave 7 (Language and Highlighting)** sub-project. It depends on:
- `ff-document-model` (Wave 3) — text buffer content, line indexing, edit notifications
- `ff-language-service` (Wave 7 peer) — language detection, TOML-based language definitions, keyword lists, comment patterns
- `ff-theme` (Wave 6) — style-slot table resolution at render time (not referenced directly by this crate)
- `ff-configuration-system` (Wave 2) — lexer property storage, hot-reload notifications
- `ff-idle-processing` (Wave 5) — idle-time scheduling for background styling

It is consumed by:
- `ff-display-line-mapping` — fold-level queries for fold region calculation
- `ff-desktop` (GUI shell) — styled span queries for viewport painting
- `ff-text-decorations` — coexists independently on the same character ranges

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-syntax-highlighting/Cargo.toml` with dependencies (serde, thiserror, proptest dev-dep) and deps on `ff-document-model`, `ff-language-service`, `ff-configuration-system`, `ff-logging`
  - [x] 1.2 Create `crates/ff-syntax-highlighting/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `lexer.rs`, `style_buffer.rs`, `per_line_state.rs`, `incremental.rs`, `demand_driven.rs`, `keyword.rs`, `comment.rs`, `sub_style.rs`, `fold.rs`, `idle_styling.rs`, `style_context.rs`, `registry.rs`, `properties.rs`, `lifecycle.rs`, `error.rs`
  - [x] 1.4 Add `ff-syntax-highlighting` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Lexer trait interface
  - [x] 2.1 Define `Lexer` trait with `style_text(&mut self, context: &mut StyleContext)` method
  - [x] 2.2 Add `fold_text(&mut self, context: &mut FoldContext)` method to `Lexer` trait
  - [x] 2.3 Add `name() -> &str` method returning unique lexer identifier
  - [x] 2.4 Add `default_style() -> StyleSlotIndex` method returning the default style index
  - [x] 2.5 Add `keyword_sets() -> &[KeywordSetDescriptor]` method returning keyword set metadata
  - [x] 2.6 Add `sub_style_bases() -> &[StyleSlotIndex]` method returning base styles supporting sub-styles
  - [x] 2.7 Add `get_property(key: &str) -> Option<&str>` and `set_property(key: &str, value: &str)` methods
  - [x] 2.8 Define `KeywordSetDescriptor` struct with set_index, name, description fields
  - [x] 2.9 Define `StyleSlotIndex` newtype wrapping u8 (range 0–255)
  - [x] 2.10 Write unit tests for trait object safety and default implementations
  - Covers: Requirement 1 (AC 1.1–1.7)

- [x] 3. Lexer registry
  - [x] 3.1 Define `LexerRegistry` struct with a `HashMap<String, Box<dyn Fn() -> Box<dyn Lexer>>>` factory map
  - [x] 3.2 Implement `register(language_id: &str, factory: impl Fn() -> Box<dyn Lexer>)` for dynamic registration
  - [x] 3.3 Implement `create_lexer(language_id: &str) -> Option<Box<dyn Lexer>>` for lexer instantiation
  - [x] 3.4 Implement `available_languages() -> Vec<&str>` listing registered lexer identifiers
  - [x] 3.5 Implement thread-safe access via `Arc<RwLock<LexerRegistry>>` wrapper
  - [x] 3.6 Write unit tests for registration, creation, unknown language returns None, duplicate registration
  - Covers: Requirement 1 (AC 1.8)

- [x] 4. Style buffer and storage
  - [x] 4.1 Define `StyleBuffer` struct storing `Vec<u8>` parallel to document text
  - [x] 4.2 Implement `style_at(position: BytePosition) -> StyleSlotIndex` with O(1) lookup
  - [x] 4.3 Implement `set_style_range(start: BytePosition, end: BytePosition, style: StyleSlotIndex)` for bulk assignment
  - [x] 4.4 Implement `styled_spans(start: BytePosition, end: BytePosition) -> impl Iterator<Item = HighlightSpan>` coalescing adjacent same-style characters
  - [x] 4.5 Define `HighlightSpan` struct with start, end, and style_slot_index fields
  - [x] 4.6 Implement `insert_at(position: BytePosition, count: usize)` inserting default style (0) values
  - [x] 4.7 Implement `delete_range(start: BytePosition, end: BytePosition)` removing style values
  - [x] 4.8 Implement length synchronization invariant: style buffer length always equals document text length
  - [x] 4.9 Write unit tests for style_at, styled_spans coalescing, insert/delete sync, initial default style
  - Covers: Requirement 2 (AC 2.1–2.8)

- [x] 5. Per-line state and incremental re-highlighting
  - [x] 5.1 Define `PerLineState` struct storing `Vec<LexerState>` synchronized with document line count
  - [x] 5.2 Define `LexerState` newtype wrapping i32 for lexer state encoding
  - [x] 5.3 Implement `state_at_line_start(line: LineNumber) -> LexerState` returning the stored state for the preceding line end (or initial state for line 0)
  - [x] 5.4 Implement `set_state(line: LineNumber, state: LexerState)` for updating per-line state after lexing
  - [x] 5.5 Implement `insert_lines(at: LineNumber, count: usize)` inserting initial-state entries for new lines
  - [x] 5.6 Implement `delete_lines(at: LineNumber, count: usize)` removing entries for deleted lines
  - [x] 5.7 Define `StylingPosition` tracking the furthest styled byte offset
  - [x] 5.8 Implement `invalidate_from(position: BytePosition)` setting styling position to min(current, start of modified line)
  - [x] 5.9 Implement incremental re-highlight loop: lex from first modified line's state forward until state convergence
  - [x] 5.10 Implement state convergence detection: stop when computed end-of-line state matches previously stored state
  - [x] 5.11 Implement multi-line construct propagation: continue past viewport until convergence on state change
  - [x] 5.12 Write unit tests for state storage, invalidation, convergence detection, multi-line propagation, line insert/delete
  - Covers: Requirement 3 (AC 3.1–3.10)

- [x] 6. Demand-driven styling (EnsureStyledTo)
  - [x] 6.1 Implement `ensure_styled_to(position: BytePosition)` guaranteeing all text up to position is styled
  - [x] 6.2 Implement early return when position <= current styling_position (no work needed)
  - [x] 6.3 Implement forward lexing from styling_position using stored per-line state until target position reached
  - [x] 6.4 Implement `styling_position() -> BytePosition` accessor for current end-of-styled position
  - [x] 6.5 Implement boundary: do not style beyond requested position plus one full line
  - [x] 6.6 Implement no-lexer handling: treat all text as default style (index 0), ensure_styled_to is a no-op
  - [x] 6.7 Write unit tests for demand-driven styling at various positions, early return, no-lexer behaviour
  - Covers: Requirement 4 (AC 4.1–4.7)

- [x] 7. Keyword matching and WordList
  - [x] 7.1 Define `WordList` struct with hash-based storage for O(1) average-case keyword lookup
  - [x] 7.2 Implement `WordList::new(words: &[&str], case_sensitive: bool)` constructor populating the hash set
  - [x] 7.3 Implement `WordList::contains(word: &str) -> bool` with case-sensitive or case-insensitive matching (Unicode simple case folding for insensitive)
  - [x] 7.4 Implement `WordList::add(word: &str)` and `WordList::remove(word: &str)` for runtime modification
  - [x] 7.5 Define `KeywordSetConfig` holding up to 9 keyword sets (indexed 0–8) each with WordList and associated StyleSlotIndex
  - [x] 7.6 Implement keyword lookup during lexing: check identifier against sets in order (0 first), return first matching set's style
  - [x] 7.7 Implement case-insensitive comparison using Unicode simple case folding
  - [x] 7.8 Implement runtime keyword set modification with full document re-highlight trigger
  - [x] 7.9 Write unit tests for WordList lookup (case-sensitive/insensitive), ordered set matching, runtime modification trigger
  - Covers: Requirement 5 (AC 5.1–5.9)

- [x] 8. Comment detection and multi-line state
  - [x] 8.1 Implement line-comment detection: match `line_comment` pattern from language definition, style to end-of-line
  - [x] 8.2 Implement block-comment detection: match `block_comment_start`/`block_comment_end` patterns, style enclosed text
  - [x] 8.3 Implement multi-line block comment state: encode "inside block comment" in Per_Line_State for intermediate lines
  - [x] 8.4 Implement unclosed block comment handling: style from opener to document end, propagate open-comment state
  - [x] 8.5 Implement block-comment close insertion: propagate re-highlight forward reverting comment style until convergence
  - [x] 8.6 Implement nested block comment support with nesting depth tracked in LexerState
  - [x] 8.7 Implement multiple comment style support (e.g., `///` doc comments vs `//` regular comments) with distinct StyleSlotIndex values
  - [x] 8.8 Write unit tests for line comments, block comments, multi-line state, unclosed comments, nested comments, multiple styles
  - Covers: Requirement 6 (AC 6.1–6.7)

- [x] 9. Sub-styles
  - [x] 9.1 Define `SubStyleRange` struct with base_style, start_index, count fields
  - [x] 9.2 Define `SubStyleAllocator` managing available style index pool (0–255 shared between base and sub-styles)
  - [x] 9.3 Implement `allocate_sub_styles(base_style: StyleSlotIndex, count: u8) -> Result<SubStyleRange>` reserving contiguous indices
  - [x] 9.4 Implement `free_sub_styles(base_style: StyleSlotIndex)` releasing allocated indices back to pool
  - [x] 9.5 Implement `sub_style_base(sub_style: StyleSlotIndex) -> Option<StyleSlotIndex>` returning base for a sub-style index
  - [x] 9.6 Implement sub-style identifier matching: check token against sub-style WordLists when base style has sub-styles allocated
  - [x] 9.7 Implement allocation failure when requested count exceeds available indices (return error)
  - [x] 9.8 Write unit tests for allocation, freeing, base lookup, identifier matching, allocation overflow error
  - Covers: Requirement 7 (AC 7.1–7.8)

- [x] 10. Fold-level assignment
  - [x] 10.1 Define `FoldContext` struct with line-level access and `set_level(line, level, flags)` method
  - [x] 10.2 Define `FoldLevel` as u16 (12-bit range 0–4095) and `FoldFlags` with FOLD_HEADER and FOLD_WHITESPACE constants
  - [x] 10.3 Define `FoldData` struct storing per-line `(u16, FoldFlags)` pairs synchronized with document line count
  - [x] 10.4 Implement `fold_level_at(line: LineNumber) -> (u16, FoldFlags)` accessor for single-line queries
  - [x] 10.5 Implement `fold_level_range(start_line, end_line) -> impl Iterator<Item = (LineNumber, u16, FoldFlags)>` for bulk queries
  - [x] 10.6 Implement FOLD_HEADER auto-marking: mark line when its level > following line's level and line has visible content
  - [x] 10.7 Implement incremental fold-level recomputation on document edits (same modified-range logic as styling)
  - [x] 10.8 Implement on-demand fold computation (not eagerly for entire document at load time)
  - [x] 10.9 Implement fold-level-changed notification emission with affected line range
  - [x] 10.10 Write unit tests for level storage, header detection, incremental recomputation, bulk range query
  - Covers: Requirement 8 (AC 8.1–8.8), Requirement 15 (AC 15.1–15.2, 15.6)

- [x] 11. Idle-time background styling
  - [x] 11.1 Define `IdleStylingConfig` struct with lines_per_slice (default 256) and time_budget_ms (default 10) fields
  - [x] 11.2 Implement idle work source registration with `idle-processing` scheduler
  - [x] 11.3 Implement `perform_idle_styling(&mut self, time_budget_ms: u64) -> IdleStylingResult` styling a bounded chunk from styling_position
  - [x] 11.4 Implement time-budget enforcement: stop before exceeding configured time budget per slice
  - [x] 11.5 Implement deregistration from idle scheduler when entire document is fully styled
  - [x] 11.6 Implement edit interruption: cancel current idle work, process edit re-highlight, resume on next idle
  - [x] 11.7 Implement completion notification emission when full-document styling finishes
  - [x] 11.8 Write unit tests for chunk-bounded styling, time budget compliance, completion detection, edit interruption
  - Covers: Requirement 9 (AC 9.1–9.7)

- [x] 12. Property-based lexer configuration
  - [x] 12.1 Define `PropertyStorage` struct with `HashMap<String, String>` per lexer instance
  - [x] 12.2 Implement property population from language-definition TOML (via language-service) at lexer bind time
  - [x] 12.3 Implement user-override loading from configuration-system
  - [x] 12.4 Implement `set_property(key, value)` calling through to bound lexer, storing in property map
  - [x] 12.5 Implement `get_property(key) -> Option<&str>` for runtime introspection
  - [x] 12.6 Implement hot-reload: on property change, invalidate all styling/fold data and trigger full re-highlight
  - [x] 12.7 Implement `property_names() -> &[PropertyDescriptor]` trait method with name, type, description, default fields
  - [x] 12.8 Implement unknown property handling: store value, log DEBUG message, no error
  - [x] 12.9 Write unit tests for property set/get, hot-reload invalidation, unknown key handling, descriptor listing
  - Covers: Requirement 10 (AC 10.1–10.7)

- [x] 13. GUI-independent engine architecture and SyntaxHighlighter trait
  - [x] 13.1 Define `SyntaxHighlighter` public trait with methods: `ensure_styled_to`, `style_at`, `styled_spans`, `styling_position`, `fold_level_at`, `fold_level_range`, `style_slot_count`
  - [x] 13.2 Implement `HighlightEngine` struct as the concrete implementation of `SyntaxHighlighter`
  - [x] 13.3 Implement thread-safety: protect style buffer and per-line state with `RwLock` for concurrent read (GUI) and write (idle background styling)
  - [x] 13.4 Implement multi-document support: `HighlightEngine` manages per-document state (style buffer, per-line state, lexer) independently with no global mutable state
  - [x] 13.5 Verify zero GUI dependencies: no egui, wgpu, winit, or platform windowing references in Cargo.toml
  - [x] 13.6 Write unit tests exercising all public API through the `SyntaxHighlighter` trait (in-memory document, no GUI)
  - Covers: Requirement 11 (AC 11.1–11.6)

- [x] 14. Theme integration and style resolution
  - [x] 14.1 Implement `style_slot_count() -> u8` method reporting how many base style indices the active lexer uses
  - [x] 14.2 Ensure engine produces only StyleSlotIndex values — no colour/font references in engine output
  - [x] 14.3 Implement semantic name mapping support: provide style index to token name mapping for language-service
  - [x] 14.4 Implement sub-style inheritance: document that unthemed sub-styles inherit from base style in theme system
  - [x] 14.5 Write unit tests verifying no colour references in engine output, style_slot_count correctness
  - Covers: Requirement 12 (AC 12.1–12.6)

- [x] 15. Lexer lifecycle and document binding
  - [x] 15.1 Implement document-lexer binding: instantiate lexer from registry when language detected, bind to document style context
  - [x] 15.2 Implement unbound state: documents with unknown language have default style (0) for all text
  - [x] 15.3 Implement language change: unbind previous lexer, bind new lexer, invalidate all styling/fold data, trigger full re-highlight
  - [x] 15.4 Implement document close cleanup: release lexer instance, style buffer, and per-line state
  - [x] 15.5 Implement keyword set population from language definition at bind time
  - [x] 15.6 Implement property population from configuration-system at bind time
  - [x] 15.7 Implement runtime lexer registration: new lexers available immediately for binding without restart
  - [x] 15.8 Write unit tests for bind, unbind, language change, close cleanup, keyword population
  - Covers: Requirement 13 (AC 13.1–13.7)

- [x] 16. StyleContext helper
  - [x] 16.1 Define `StyleContext` struct with current position, start position, character accessors, and state tracking
  - [x] 16.2 Implement `ch() -> char`, `ch_next() -> char`, `ch_prev() -> char` character accessors with boundary safety (return '\0' at document edges)
  - [x] 16.3 Implement `state() -> LexerState` and `set_state(new_state: LexerState)` with automatic style assignment from token start to current position
  - [x] 16.4 Implement `forward()` advancing by one character (handling multi-byte UTF-8)
  - [x] 16.5 Implement `forward_bytes(count: usize)` advancing by specified byte count
  - [x] 16.6 Implement `match_keyword(word_list: &WordList) -> Option<KeywordSetIndex>` checking current token against keyword sets
  - [x] 16.7 Implement `at_line_start() -> bool` and `at_line_end() -> bool` position queries
  - [x] 16.8 Implement `more() -> bool` returning true if more characters remain in range
  - [x] 16.9 Implement `start_position() -> BytePosition` accessor for current token start
  - [x] 16.10 Write unit tests for character access, state transitions, forward movement, boundary handling, keyword matching
  - Covers: Requirement 14 (AC 14.1–14.9)

- [x] 17. Integration with display-line-mapping and text-decorations
  - [x] 17.1 Implement fold-level-changed notification with affected line range for display-line-mapping consumption
  - [x] 17.2 Ensure style data and indicator data are independently queryable (no interference between storage systems)
  - [x] 17.3 Ensure re-highlighting does not modify or invalidate indicator decorations
  - [x] 17.4 Write unit tests for notification emission, independent storage verification
  - Covers: Requirement 15 (AC 15.1–15.6)

- [x] 18. Error handling
  - [x] 18.1 Define `SyntaxHighlightError` enum: LexerNotFound, SubStyleAllocationFailed, PropertyError, StyleBufferSyncError, InvalidPosition
  - [x] 18.2 Implement error message formatting per `[syntax-highlighting] operation: description` standard (≤200 chars)
  - [x] 18.3 Implement graceful degradation: errors during lexing fall back to default style without crashing
  - [x] 18.4 Write unit tests for all error variants and graceful degradation behaviour
  - Covers: Cross-cutting error handling standards

- [x] 19. Property-based tests
  - [x] 19.1 Write PBT: style buffer length invariant
  - [x] 19.2 Write PBT: incremental re-highlight convergence correctness
  - [x] 19.3 Write PBT: keyword lookup consistency
  - [x] 19.4 Write PBT: demand-driven styling idempotency
  - [x] 19.5 Write PBT: sub-style allocation pool integrity
  - [x] 19.6 Write PBT: fold-level header detection correctness
  - [x] 19.7 Write PBT: styled spans coalescing completeness
  - [x] 19.8 Write PBT: per-line state synchronization with line count
  - Covers: Requirements 2–9, 11 (see Property-Based Test Definitions below)

- [x] 20. Integration tests
  - [x] 20.1 Write integration test: full document load → language detection → lexer binding → demand-driven styling lifecycle
  - [x] 20.2 Write integration test: incremental edit (single char insert) with re-highlight convergence within one line
  - [x] 20.3 Write integration test: multi-line block comment insertion and closure with state propagation
  - [x] 20.4 Write integration test: idle-time styling progressively styles entire document in chunks
  - [x] 20.5 Write integration test: language change rebinding with full re-highlight
  - [x] 20.6 Write integration test: keyword set runtime modification with document-wide re-highlight
  - [x] 20.7 Write integration test: sub-style allocation, identifier matching, and freeing lifecycle
  - [x] 20.8 Write integration test: fold-level computation alongside styling with notification emission
  - Covers: End-to-end validation across Requirements 1–15

---

## Property-Based Test Definitions

### Property 1: Style Buffer Length Invariant

**Validates: Requirements 2.6, 2.7, 2.8**

- **Statement:** For any sequence of document insertions and deletions, the style buffer length SHALL always equal the document text length. After inserting N bytes at any valid position, the style buffer grows by N. After deleting a range [start, end), the style buffer shrinks by (end - start).
- **Strategy:** Generate:
  - initial_text: String of length [0, 10000]
  - operations: Vec of Insert(position, text) and Delete(start, end) with valid bounds
- **Invariant:** After every operation, `style_buffer.len() == document.len()`

### Property 2: Incremental Re-Highlight Convergence Correctness

**Validates: Requirements 3.4, 3.10**

- **Statement:** For any single-character edit that does not change the end-of-line lexer state, incremental re-highlighting SHALL produce identical results to full re-highlighting of the entire document, and SHALL complete within O(line_length) work (re-highlighting stops at the modified line when state converges).
- **Strategy:** Generate:
  - document: multi-line text with known lexer states
  - edit_position: valid position within a line
  - edit_char: character that does not change line-end state (e.g., space within a string literal)
- **Invariant:** `style_buffer_after_incremental == style_buffer_after_full_rehighlight` AND `lines_rehighlighted <= 1`

### Property 3: Keyword Lookup Consistency

**Validates: Requirements 5.3, 5.7**

- **Statement:** For any word inserted into a case-insensitive WordList, lookups with any case variation of that word SHALL return true. For any word NOT in the WordList, lookups SHALL return false regardless of case. For case-sensitive WordLists, only exact matches return true.
- **Strategy:** Generate:
  - words: Vec<String> of [1, 100] ASCII identifiers
  - case_sensitive: bool
  - query: random string (mix of words in list with case variations, and words not in list)
- **Invariant:** If case_sensitive: `contains(query) == words.contains(&query)`. If !case_sensitive: `contains(query) == words.iter().any(|w| w.eq_ignore_ascii_case(&query))`

### Property 4: Demand-Driven Styling Idempotency

**Validates: Requirements 4.1, 4.2**

- **Statement:** Calling `ensure_styled_to(position)` multiple times with the same position SHALL produce identical style buffer contents after each call. The second call SHALL perform no work (early return when position <= styling_position).
- **Strategy:** Generate:
  - document: text of length [100, 5000]
  - position: BytePosition in [0, document.len()]
- **Invariant:** `style_buffer_after_first_call == style_buffer_after_second_call` AND second call is a no-op

### Property 5: Sub-Style Allocation Pool Integrity

**Validates: Requirements 7.1, 7.5, 7.6**

- **Statement:** For any sequence of sub-style allocations and frees, the total number of allocated style indices SHALL never exceed 256. Allocated ranges SHALL never overlap. After freeing a base style's sub-styles, those indices SHALL be available for reallocation.
- **Strategy:** Generate:
  - operations: Vec of Allocate(base, count) and Free(base) in random order
  - base styles: distinct StyleSlotIndex values
  - counts: u8 in [1, 30]
- **Invariant:** No two active SubStyleRanges overlap AND total allocated + base styles <= 256 AND freed indices become reusable

### Property 6: Fold-Level Header Detection Correctness

**Validates: Requirements 8.3, 8.4**

- **Statement:** A line SHALL be marked with FOLD_HEADER if and only if its fold level is greater than the following line's fold level AND the line has visible (non-whitespace) content. Lines with only whitespace SHALL never be marked FOLD_HEADER regardless of level relationships.
- **Strategy:** Generate:
  - levels: Vec<u16> of [2, 100] values in range [0, 4095]
  - line_contents: Vec<String> (mix of whitespace-only and content lines)
- **Invariant:** For each line i: `has_header_flag(i) == (levels[i] > levels[i+1] && !is_whitespace_only(content[i]))`

### Property 7: Styled Spans Coalescing Completeness

**Validates: Requirements 2.4**

- **Statement:** For any style buffer content and any query range [start, end), the styled_spans iterator SHALL produce spans that: (a) completely cover the range with no gaps, (b) have no adjacent spans with the same style index, and (c) each span has a uniform style index throughout.
- **Strategy:** Generate:
  - style_buffer: Vec<u8> of [1, 5000] random values in [0, 10]
  - start: valid position
  - end: valid position > start
- **Invariant:** `spans_cover_range_completely(spans, start, end)` AND `no_adjacent_same_style(spans)` AND `each_span_uniform(spans, style_buffer)`

### Property 8: Per-Line State Synchronization with Line Count

**Validates: Requirements 3.1, 3.8, 3.9**

- **Statement:** For any sequence of line insertions and deletions, the per-line state storage length SHALL always equal the document's line count. Inserted lines receive initial state. Deleted lines remove their state entries.
- **Strategy:** Generate:
  - initial_line_count: usize in [1, 1000]
  - operations: Vec of InsertLines(at, count) and DeleteLines(at, count) with valid bounds
- **Invariant:** After every operation, `per_line_state.len() == document.line_count()`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Trait", "tasks": ["2", "3", "18"], "dependsOn": [0] },
    { "id": 2, "label": "Storage Layer", "tasks": ["4", "5"], "dependsOn": [1] },
    { "id": 3, "label": "Demand-Driven and Incremental Engine", "tasks": ["6", "16"], "dependsOn": [2] },
    { "id": 4, "label": "Language Features", "tasks": ["7", "8", "9", "10"], "dependsOn": [3] },
    { "id": 5, "label": "Runtime and Lifecycle", "tasks": ["11", "12", "15"], "dependsOn": [4] },
    { "id": 6, "label": "Public API and Integration", "tasks": ["13", "14", "17"], "dependsOn": [5] },
    { "id": 7, "label": "Validation", "tasks": ["19", "20"], "dependsOn": [6] }
  ]
}
```

---

## Notes

- This is a Wave 7 (Language and Highlighting) crate that is a **GUI-independent highlighting engine** — it produces abstract style-slot indices, never colour values or font references.
- GUI independence is a strict requirement: no `egui`, `wgpu`, `winit`, or platform rendering types in this crate's public API or dependencies.
- The logical document model (text buffer, line indexing, edit notifications) is owned by `ff-document-model` — this crate only consumes it for text access and edit events.
- Style resolution (mapping style-slot indices to colours/fonts) is the responsibility of `ff-theme` at render time — the highlighting engine is not aware of themes.
- The `StyleContext` helper struct simplifies lexer implementation by providing convenient character access, state management, and keyword matching — lexer implementors work with this API.
- Fold-level data is computed alongside styling by the lexer's `fold_text` method but stored independently, queryable by `ff-display-line-mapping` for fold region identification.
- Thread safety is achieved via `RwLock` on the style buffer and per-line state, allowing background idle-styling on a worker thread while the GUI thread reads style data for rendering.
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property.
- Idle-time background styling integrates with `ff-idle-processing` scheduler — the engine registers/deregisters as an idle work source based on whether unstyled regions remain.
- Runtime lexer registration supports plugin-provided lexers without restart; keyword set and property changes trigger full document re-highlight.

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Lexer Trait Interface | AC 1.1–1.7 | Task 2 |
| Req 1: Lexer Registry | AC 1.8 | Task 3 |
| Req 2: Style Assignment and Storage | AC 2.1–2.8 | Task 4 |
| Req 3: Incremental Re-Highlighting | AC 3.1–3.10 | Task 5 |
| Req 4: Demand-Driven Styling | AC 4.1–4.7 | Task 6 |
| Req 5: Keyword Matching | AC 5.1–5.9 | Task 7 |
| Req 6: Comment Detection and Multi-Line State | AC 6.1–6.7 | Task 8 |
| Req 7: Sub-Styles | AC 7.1–7.8 | Task 9 |
| Req 8: Fold-Level Assignment | AC 8.1–8.8 | Task 10 |
| Req 9: Idle-Time Background Styling | AC 9.1–9.7 | Task 11 |
| Req 10: Property-Based Lexer Configuration | AC 10.1–10.7 | Task 12 |
| Req 11: GUI-Independent Engine Architecture | AC 11.1–11.6 | Task 13 |
| Req 12: Theme Integration and Style Resolution | AC 12.1–12.6 | Task 14 |
| Req 13: Lexer Lifecycle and Document Binding | AC 13.1–13.7 | Task 15 |
| Req 14: Style Context Helper | AC 14.1–14.9 | Task 16 |
| Req 15: Integration with Display-Line-Mapping | AC 15.1–15.6 | Tasks 10, 17 |
| Cross-cutting: Error Handling | All | Task 18 |
