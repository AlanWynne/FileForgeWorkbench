# Requirements Document

## Introduction

This feature specifies the **Syntax Highlighting** subsystem for FileForgeWorkbench — the `ff-syntax-highlighting` crate. The syntax highlighting engine is responsible for assigning visual style information (colour, bold, italic, underline, case) to character ranges based on lexical analysis of document content. It operates as a **GUI-independent highlighting engine** that produces styled spans consumed by the rendering layer through the theme system.

The engine supports incremental re-highlighting (re-lexing only from the first modified line's state forward), per-line lexer state persistence, demand-driven styling, multiple keyword sets per language, sub-styles for fine-grained token differentiation, fold-level assignment alongside styling, and idle-time background styling.

This specification merges requirements from two primary sources:

- **FileForgeEditor MVP Requirement 6**: `LexicalHighlighter` producing `HighlightSpan` values for keyword occurrences, keywords rendered in distinct colour, comment-span detection via `line_comment` patterns
- **Lexilla/Scintilla lexer infrastructure**: Incremental re-highlighting, per-line state persistence, demand-driven styling (`EnsureStyledTo`), multiple keyword sets (up to 9), sub-styles, style slots (0–255), fold-level assignment, idle-time styling, and property-based lexer configuration

The design adapts Scintilla's C++ lexer architecture to idiomatic Rust: trait-based lexer interface replaces virtual methods, TOML-based language definitions replace property strings, and the message-passing coordination is replaced by direct method calls on a shared styling context.

**Source references:**
- **[FFE-MVP-6]** = FileForgeEditor mvp-implementation Requirement 6: Syntax highlighting with LexicalHighlighter, HighlightSpan, keyword detection, comment detection
- **[LEX-INFRA]** = Lexilla lexer infrastructure: incremental lexing, per-line state, keyword sets, sub-styles, fold assignment
- **[LEX-SUPPORT]** = Lexilla lexer-support utilities: WordList, LexAccessor, StyleContext, property management
- **[SCI-DOC-13]** = Scintilla document styling API: EnsureStyledTo, GetEndStyled, idle-time styling coordination, style byte storage
- **[WB]** = Workbench Platform Architecture Brief: GUI-independent core, command-driven architecture, theme integration

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `language-service` | **Dependency** | Provides language detection, TOML-based language definitions (keyword lists, comment patterns, lexer selection), and multi-line state rules that the highlighting engine consumes. |
| `document-model` | **Dependency** | Provides the text buffer content, line indexing, and edit notifications (insert/delete positions) that trigger incremental re-highlighting. |
| `theme-and-appearance` | **Consumer** | Style results (style slot indices) are resolved to visual attributes (colours, fonts, bold/italic) through the theme system's style-slot table and syntax colour group. |
| `display-line-mapping` | **Consumer** | Fold levels produced by the highlighting engine are consumed by the display-line-mapping to identify fold regions and fold headers. |
| `text-decorations` | **Peer** | Syntax styles and indicator decorations coexist on the same text; the indicator `under` property determines rendering order relative to syntax-coloured text. |
| `idle-processing` | **Integration** | Background idle-time styling is coordinated through the idle-processing scheduler, which grants time slices for incremental highlighting of unstyled regions. |
| `configuration-system` | **Integration** | Lexer properties and per-language configuration overrides are stored in the configuration system, supporting hot-reload of highlighting behaviour. |

## Glossary

- **Lexer**: A trait implementation that performs lexical analysis on a text range, assigning style-slot indices to character ranges and optionally computing fold levels. Each supported language has one or more associated lexer implementations. [LEX-INFRA]
- **Style_Slot_Index**: An integer (0–255) identifying a visual style class. The lexer assigns these indices to character ranges; the theme system resolves each index to concrete visual attributes (colour, font weight, etc.). [SCI-DOC-13, LEX-INFRA]
- **HighlightSpan**: A contiguous range of characters sharing the same Style_Slot_Index, representing the output of the lexer for a given text region. [FFE-MVP-6]
- **Lexer_State**: An integer representing the lexer's parsing state at a given position. Stored per-line to enable incremental re-highlighting from any point in the document. [LEX-INFRA]
- **Per_Line_State**: The Lexer_State value stored at the end of each document line, enabling the engine to resume lexing from any line without re-processing the entire document from the beginning. [LEX-INFRA]
- **Incremental_Rehighlight**: The process of re-lexing text starting from the first modified line (using its stored state), continuing until the computed state matches the previously stored state for a subsequent line — at which point re-highlighting can stop because subsequent styling remains valid. [LEX-INFRA]
- **EnsureStyledTo**: The demand-driven styling API that guarantees all text up to a given position has been styled. Called by the viewport renderer before painting to ensure visible text has valid style data. [SCI-DOC-13]
- **Keyword_Set**: An ordered collection of keywords associated with a specific style class within a language definition. Languages may define up to 9 keyword sets (numbered 0–8), each mapped to a distinct Style_Slot_Index. [LEX-INFRA, LEX-SUPPORT]
- **WordList**: The internal data structure storing a Keyword_Set for efficient O(1) average-case lookup during lexing. Keywords are case-sensitive or case-insensitive per set configuration. [LEX-SUPPORT]
- **Sub_Style**: A mechanism for fine-grained differentiation of tokens within a single base style class. Sub-styles allow users or plugins to assign distinct colours to subsets of identifiers (e.g., differentiating local variables from global variables within the "identifier" style class). [LEX-INFRA]
- **Sub_Style_Range**: A contiguous block of Style_Slot_Index values allocated from the extended range (above the lexer's base styles) for sub-style differentiation within one base style. [LEX-INFRA]
- **Fold_Level**: An integer indicating the nesting depth at a given line, used by the display-line-mapping to identify fold regions. Computed by the lexer alongside styling. [LEX-INFRA]
- **Fold_Header**: A line marked with the fold-header flag, indicating it is the first line of a foldable region (the next line has a higher fold level). [LEX-INFRA]
- **Idle_Styling**: Background highlighting that occurs during idle periods (no user input), progressively styling unstyled regions of the document without blocking interactive editing. [SCI-DOC-13]
- **Style_Context**: A helper structure providing convenient access to the current character, next character, previous character, current state, and style-assignment methods during lexing. Simplifies lexer implementation. [LEX-SUPPORT]
- **Lexer_Property**: A named configuration value (string, integer, or boolean) that parameterizes lexer behaviour, stored in the configuration system and queryable by the lexer at runtime. [LEX-SUPPORT]
- **Styling_Position**: The byte offset up to which the document has been fully styled. Text beyond this position may have stale or absent style data. [SCI-DOC-13]

## Requirements

### Requirement 1: Lexer Trait Interface

**User Story:** As a language extension developer, I want a well-defined lexer trait that I can implement for new languages, so that the highlighting engine can invoke my lexer without coupling to a specific language's parsing logic.

**Source:** [LEX-INFRA] Lexilla ILexer5 interface; [WB] trait-based extensibility.

#### Acceptance Criteria

1. THE syntax-highlighting crate SHALL define a `Lexer` trait with a `style_text(context: &mut StyleContext)` method that performs lexical analysis on a specified text range, assigning Style_Slot_Index values to each character position within the range.
2. THE `Lexer` trait SHALL include a `fold_text(context: &mut FoldContext)` method that computes Fold_Level values for each line within the specified range, separate from but invocable alongside styling.
3. THE `Lexer` trait SHALL include a `name() → &str` method returning the unique identifier of the lexer (e.g., `"rust"`, `"cpp"`, `"cobol"`).
4. THE `Lexer` trait SHALL include a `default_style() → StyleSlotIndex` method returning the Style_Slot_Index used for unstyled/default text in this language.
5. THE `Lexer` trait SHALL include a `keyword_sets() → &[KeywordSetDescriptor]` method returning metadata about the keyword sets the lexer supports (set index, name, description), enabling the language-service to populate keyword lists from TOML definitions.
6. THE `Lexer` trait SHALL include a `sub_style_bases() → &[StyleSlotIndex]` method returning the base style indices that support sub-style differentiation, enabling the engine to allocate Sub_Style_Range blocks.
7. THE `Lexer` trait SHALL include a `get_property(key: &str) → Option<&str>` and `set_property(key: &str, value: &str)` methods for lexer-specific property configuration.
8. THE syntax-highlighting crate SHALL provide a lexer registry that maps language identifiers to `Lexer` trait implementations, supporting dynamic registration at runtime for plugin-provided lexers.

---

### Requirement 2: Style Assignment and Storage

**User Story:** As the viewport renderer, I need to query the style assigned to any character position so that I can render text with the correct colour, font weight, and attributes as defined by the active theme.

**Source:** [SCI-DOC-13] Style byte storage; [FFE-MVP-6] HighlightSpan production; [LEX-INFRA] style slots 0–255.

#### Acceptance Criteria

1. THE syntax-highlighting engine SHALL maintain a style buffer parallel to the document text buffer, storing one Style_Slot_Index (u8, range 0–255) per character position.
2. WHEN the lexer assigns a style to a character range via `StyleContext::set_style(start, end, style_index)`, THE engine SHALL update the corresponding positions in the style buffer.
3. THE engine SHALL provide a `style_at(position: BytePosition) → StyleSlotIndex` method that returns the style index assigned to the given character position in O(1) time.
4. THE engine SHALL provide a `styled_spans(start: BytePosition, end: BytePosition) → impl Iterator<Item = HighlightSpan>` method that returns contiguous spans of uniformly-styled text within the specified range, coalescing adjacent characters with the same style index into a single HighlightSpan.
5. WHEN the document is first loaded and no lexer has been invoked, ALL positions in the style buffer SHALL have the default style index (0) until styled by the lexer.
6. THE style buffer SHALL grow and shrink in synchronization with document insertions and deletions, maintaining the invariant that the style buffer length equals the document text length at all times.
7. WHEN text is inserted at a position, THE engine SHALL insert default style values (0) at the corresponding positions in the style buffer and mark the affected region for re-highlighting.
8. WHEN text is deleted, THE engine SHALL remove the corresponding style values from the style buffer.

---

### Requirement 3: Incremental Re-Highlighting

**User Story:** As a user editing a large document, I want only the affected region to be re-highlighted after each edit, so that syntax colouring updates instantly without processing the entire file.

**Source:** [LEX-INFRA] Incremental lexing from first modified line's state; [SCI-DOC-13] GetEndStyled, partial re-lex.

#### Acceptance Criteria

1. THE engine SHALL store Per_Line_State values (the Lexer_State at the end of each line) in a per-line data structure synchronized with the document-model's line count.
2. WHEN a document edit occurs (insertion or deletion), THE engine SHALL invalidate the Styling_Position to no later than the start of the first modified line, marking all subsequent text as potentially unstyled.
3. WHEN re-highlighting is triggered for a modified region, THE engine SHALL begin lexing from the start of the first modified line using the Per_Line_State stored for the preceding line (or the initial state for line 0).
4. THE engine SHALL continue re-highlighting line by line until the computed Lexer_State at the end of a line matches the previously stored Per_Line_State for that line — at which point re-highlighting SHALL stop because subsequent styling remains valid.
5. WHEN re-highlighting stops due to state convergence, THE engine SHALL update the Styling_Position to reflect the furthest styled position.
6. WHEN an edit changes a multi-line construct (e.g., opening a block comment without closing it), THE engine SHALL propagate re-highlighting forward until state convergence is achieved, even if this extends beyond the visible viewport.
7. THE engine SHALL update Per_Line_State values for each re-highlighted line as part of the re-highlighting pass.
8. WHEN lines are inserted into the document, THE engine SHALL insert default Per_Line_State entries (initial state) for the new lines and trigger re-highlighting from the insertion point.
9. WHEN lines are deleted from the document, THE engine SHALL remove the corresponding Per_Line_State entries and trigger re-highlighting from the deletion point.
10. FOR a single-character insertion on a line that does not change the Lexer_State at end-of-line, THE re-highlighting SHALL complete within that single line (O(line_length) work, not O(document_length)).

---

### Requirement 4: Demand-Driven Styling (EnsureStyledTo)

**User Story:** As the viewport renderer, I want to request that text be styled only up to the point I need for display, so that off-screen text is not unnecessarily processed and the editor remains responsive during initial file load.

**Source:** [SCI-DOC-13] EnsureStyledTo API, demand-driven styling, GetEndStyled.

#### Acceptance Criteria

1. THE engine SHALL provide an `ensure_styled_to(position: BytePosition)` method that guarantees all text from the beginning of the document up to the given position has valid style data.
2. IF the requested position is at or before the current Styling_Position, THEN `ensure_styled_to` SHALL return immediately without performing any work.
3. IF the requested position is beyond the current Styling_Position, THEN `ensure_styled_to` SHALL invoke the lexer starting from the Styling_Position (using the stored Per_Line_State) and lex forward until the requested position is fully styled.
4. THE engine SHALL provide a `styling_position() → BytePosition` method that returns the current end-of-styled-text position, enabling callers to check whether styling is needed before calling `ensure_styled_to`.
5. WHEN the viewport scrolls to reveal previously unstyled text, THE viewport renderer SHALL call `ensure_styled_to` with the end of the visible region before painting, ensuring all visible text has valid style data.
6. THE `ensure_styled_to` method SHALL NOT style text beyond the requested position plus one full line (to complete the current line's state), avoiding unnecessary work for regions not yet needed for display.
7. WHEN the document has no lexer assigned (unknown language), THE engine SHALL treat all text as having the default style (index 0) and `ensure_styled_to` SHALL be a no-op.

---

### Requirement 5: Keyword Matching

**User Story:** As a user, I want language keywords to be highlighted in distinct styles, with support for multiple keyword categories (e.g., control-flow keywords vs. type keywords vs. built-in functions), so that I can visually distinguish different syntactic roles.

**Source:** [FFE-MVP-6] Keywords rendered in distinct colour, LexicalHighlighter keyword detection; [LEX-INFRA] Multiple keyword sets (up to 9); [LEX-SUPPORT] WordList.

#### Acceptance Criteria

1. THE engine SHALL support up to 9 Keyword_Sets per language (indexed 0–8), each associated with a distinct Style_Slot_Index for rendering.
2. EACH Keyword_Set SHALL be populated from the language definition provided by the `language-service` subsystem, which loads keyword lists from TOML-based language definition files.
3. THE engine SHALL store keywords internally in a WordList data structure providing O(1) average-case lookup during lexing (hash-based or trie-based).
4. WHEN the lexer encounters an identifier token, IT SHALL check the identifier against all active Keyword_Sets in order (set 0 first, then set 1, etc.), assigning the Style_Slot_Index of the first matching set.
5. IF an identifier does not match any Keyword_Set, THE lexer SHALL assign the language's default identifier style index.
6. EACH Keyword_Set SHALL be independently configurable as case-sensitive or case-insensitive, controlled by the language definition.
7. WHEN a Keyword_Set is configured as case-insensitive, THE WordList lookup SHALL perform case-folded comparison (Unicode simple case folding) so that `BEGIN`, `Begin`, and `begin` all match a keyword `begin`.
8. THE engine SHALL support runtime modification of keyword sets (adding or removing keywords) to allow plugins and user configuration to extend the keyword list without restarting the workbench.
9. WHEN a Keyword_Set is modified at runtime, THE engine SHALL invalidate styling for the entire document and trigger re-highlighting from the beginning, since keyword changes may affect any position.

---

### Requirement 6: Comment Detection and Multi-Line State

**User Story:** As a user, I want comments (both line comments and block comments) to be correctly highlighted, including multi-line block comments that span many lines, so that I can visually distinguish commentary from executable code.

**Source:** [FFE-MVP-6] Comment spans via line_comment detection; [LEX-INFRA] Multi-line state persistence.

#### Acceptance Criteria

1. THE engine SHALL detect line-comment spans using the `line_comment` pattern defined in the language definition (e.g., `//` for Rust/C++, `--` for SQL, `*` in column 1 for COBOL), styling all text from the comment marker to end-of-line with the comment style index.
2. THE engine SHALL detect block-comment spans using the `block_comment_start` and `block_comment_end` patterns defined in the language definition (e.g., `/*` and `*/` for C-family languages), styling all text within the delimiters with the comment style index.
3. WHEN a block comment spans multiple lines, THE Per_Line_State at the end of each intermediate line SHALL encode "inside block comment" so that re-highlighting from any intermediate line correctly continues the comment style.
4. WHEN a block comment is opened but not closed within the document, THE engine SHALL style all text from the opening delimiter to the end of the document as comment, and the Per_Line_State for all subsequent lines SHALL reflect the open-comment state.
5. WHEN a user inserts a block-comment close delimiter, THE Incremental_Rehighlight SHALL propagate forward from the modified line, reverting subsequent lines from comment style to their correct non-comment styles until state convergence is achieved.
6. THE engine SHALL support nested block comments for languages that define them (e.g., Rust's `/* /* */ */`), tracking nesting depth in the Lexer_State.
7. THE engine SHALL support languages with multiple comment styles (e.g., documentation comments `///` vs regular comments `//` in Rust), each assigned a distinct Style_Slot_Index.

---

### Requirement 7: Sub-Styles

**User Story:** As a user, I want to visually distinguish sub-categories within a token type (e.g., different classes of identifiers: local variables vs. global variables vs. type names), so that I can apply fine-grained colour differentiation beyond what the base lexer styles provide.

**Source:** [LEX-INFRA] Sub-styles for fine-grained token differentiation within a style class.

#### Acceptance Criteria

1. THE engine SHALL support Sub_Style allocation: given a base Style_Slot_Index that supports sub-styles (as declared by `Lexer::sub_style_bases()`), THE engine SHALL allocate a contiguous block of Style_Slot_Index values from the available extended range.
2. THE engine SHALL provide an `allocate_sub_styles(base_style: StyleSlotIndex, count: u8) → SubStyleRange` method that reserves `count` contiguous style indices for sub-style differentiation of the specified base style.
3. EACH allocated Sub_Style_Range SHALL have its own associated Keyword_Set (a "sub-style identifier list") so that identifiers matching the sub-style's word list are rendered with the sub-style's index instead of the base style.
4. WHEN the lexer produces a token with a base style that has sub-styles allocated, THE engine SHALL check the token text against the sub-style identifier lists, assigning the matching sub-style index if found, or the base style index if no sub-style matches.
5. THE engine SHALL provide a `free_sub_styles(base_style: StyleSlotIndex)` method that releases all sub-style allocations for the given base style, returning those indices to the available pool.
6. THE engine SHALL support a maximum total of 256 style indices (0–255) shared between base styles and sub-styles; sub-style allocation SHALL fail with an error if the requested count would exceed available indices.
7. THE engine SHALL provide a `sub_style_base(sub_style: StyleSlotIndex) → Option<StyleSlotIndex>` method that returns the base style for a given sub-style index, or `None` if the index is not a sub-style.
8. THE theme system SHALL resolve sub-style indices to visual attributes: sub-styles inherit all attributes from their base style's theme entry unless explicitly overridden in the theme file (e.g., `[syntax.sub_styles.identifier_0]` overrides colour for the first identifier sub-style).

---

### Requirement 8: Fold-Level Assignment

**User Story:** As the code-folding UI, I need the highlighting engine to compute fold levels alongside styling, so that fold regions are identified correctly based on language syntax (braces, indentation, keywords) without requiring a separate parsing pass.

**Source:** [LEX-INFRA] Fold level assignment alongside styling; [SCI-DOC-13] SetLevel, fold flags.

#### Acceptance Criteria

1. WHEN the `Lexer` trait's `fold_text` method is invoked, THE lexer SHALL compute a Fold_Level for each line in the specified range and store it via the `FoldContext::set_level(line, level, flags)` method.
2. THE Fold_Level SHALL be an integer (12-bit value, range 0–4095) representing the nesting depth at the end of the line, with higher values indicating deeper nesting.
3. THE engine SHALL define fold flags: `FOLD_HEADER` (the line begins a foldable region), `FOLD_WHITESPACE` (the line contains only whitespace), enabling the display-line-mapping to identify fold points and blank-line folding behaviour.
4. WHEN a line's Fold_Level is greater than the following line's Fold_Level and the line has visible content (non-whitespace), THE engine SHALL mark the line with the `FOLD_HEADER` flag.
5. THE engine SHALL store computed Fold_Levels in a per-line data structure synchronized with the document-model's line count, accessible via `fold_level_at(line: LineNumber) → (u16, FoldFlags)`.
6. WHEN document edits occur, THE engine SHALL re-compute fold levels incrementally for the affected region (using the same modified-range logic as style re-highlighting).
7. THE fold-level data SHALL be computed on-demand or during idle-time styling (not eagerly for the entire document at load time), consistent with the demand-driven styling approach.
8. THE `display-line-mapping` subsystem SHALL query fold levels from the syntax-highlighting engine to determine fold region extents and fold header positions, using the public `fold_level_at` API.

---

### Requirement 9: Idle-Time Background Styling

**User Story:** As a user opening a large file, I want syntax highlighting to proceed incrementally in the background during idle periods, so that the editor is immediately responsive and highlighting appears progressively without blocking my interactions.

**Source:** [SCI-DOC-13] Idle-time styling, background styling during idle periods; [LEX-INFRA] incremental styling.

#### Acceptance Criteria

1. THE engine SHALL support an idle-time styling mode where unstyled regions beyond the current viewport are highlighted in small increments during idle periods (no user input activity).
2. THE engine SHALL integrate with the `idle-processing` scheduler, registering as an idle work source that receives time slices when the application is idle.
3. WHEN granted an idle time slice, THE engine SHALL style a bounded number of lines (configurable, default 256 lines per idle slice) starting from the current Styling_Position, advancing the Styling_Position forward.
4. THE engine SHALL NOT exceed the time budget for a single idle slice (configurable, default 10 milliseconds) to avoid introducing latency if the user resumes interaction.
5. WHEN the entire document has been styled (Styling_Position equals document length), THE engine SHALL deregister from the idle-processing scheduler until the next edit invalidates the Styling_Position.
6. WHEN a user edit occurs during idle styling, THE engine SHALL cancel the current idle work, process the edit-triggered re-highlight for the visible region immediately, and resume idle styling from the new Styling_Position on subsequent idle slices.
7. THE engine SHALL emit a notification when idle styling completes for the entire document, enabling consumers (e.g., minimap rendering) to know when full-document style data is available.

---

### Requirement 10: Property-Based Lexer Configuration

**User Story:** As a user, I want to configure language-specific highlighting behaviour (e.g., enabling/disabling nested comments, choosing brace-folding vs. indentation-folding) through configuration properties, so that I can tailor highlighting to my preferred coding style.

**Source:** [LEX-SUPPORT] Property-based lexer configuration; [LEX-INFRA] per-lexer properties.

#### Acceptance Criteria

1. THE engine SHALL provide a property storage mechanism that associates string key-value pairs with each active lexer instance (e.g., `"fold.comment" = "1"`, `"styling.within.preprocessor" = "1"`).
2. LEXER properties SHALL be populated from the language definition TOML file (loaded by `language-service`) and from user overrides in the `configuration-system`.
3. THE `Lexer` trait's `set_property(key, value)` method SHALL be called during lexer initialization for each configured property, and again when a property value changes due to configuration hot-reload.
4. WHEN a lexer property changes at runtime, THE engine SHALL invalidate all styling and fold-level data for documents using that lexer and trigger full re-highlighting (since property changes may affect any position).
5. THE engine SHALL provide a `get_property(key) → Option<&str>` method on the active lexer for introspection by the configuration UI or diagnostic tools.
6. THE engine SHALL provide a `property_names() → &[PropertyDescriptor]` method on the `Lexer` trait that returns metadata about all supported properties (name, type, description, default value), enabling auto-discovery by configuration UI.
7. WHEN an unknown property key is set on a lexer, THE engine SHALL log a DEBUG-level message and store the value without error (forward-compatible: future lexer versions may recognize it).

---

### Requirement 11: GUI-Independent Engine Architecture

**User Story:** As a workbench platform developer, I want the highlighting engine to operate without any GUI framework dependency, so that it can be used in headless testing, CI pipelines, and alternative GUI shells without modification.

**Source:** [WB] GUI-independent highlighting engine; [FFE-MVP-6] LexicalHighlighter architecture.

#### Acceptance Criteria

1. THE `ff-syntax-highlighting` crate SHALL have zero dependencies on any GUI framework (no egui, no platform windowing, no rendering API references).
2. THE engine SHALL produce style data as abstract Style_Slot_Index values (u8) that are resolved to visual attributes by the `theme-and-appearance` subsystem at render time — the highlighting engine does not reference colours, fonts, or pixels directly.
3. THE engine SHALL be fully testable without a running GUI: unit tests and property tests SHALL exercise all highlighting functionality using in-memory documents and asserting on style buffer contents.
4. THE engine SHALL expose its public API through a trait (`SyntaxHighlighter`) so that consumers (viewport renderer, minimap, export functions) depend on the trait rather than a concrete implementation.
5. THE engine SHALL be thread-safe: the style buffer and per-line state SHALL be protected by appropriate synchronization primitives (`RwLock` or equivalent) to allow background idle-styling on a separate thread while the GUI thread reads style data for rendering.
6. THE engine SHALL support multiple simultaneous documents, each with its own independent style buffer, per-line state, and lexer instance — no global mutable state.

---

### Requirement 12: Theme Integration and Style Resolution

**User Story:** As the viewport renderer, I want style indices produced by the lexer to be resolved to concrete visual attributes (colour, bold, italic) through the theme system, so that all syntax colours respect the user's chosen theme and update correctly on theme changes.

**Source:** [FFE-MVP-6] Keywords rendered in distinct colour; [SCI-DOC-13] style slots resolved to visual attributes; [WB] theme system integration.

#### Acceptance Criteria

1. THE syntax-highlighting engine SHALL NOT embed or reference any colour values — all visual attribute resolution is the responsibility of the `theme-and-appearance` subsystem via its Style_Slot table.
2. WHEN the viewport renderer needs to paint a HighlightSpan, IT SHALL use the span's Style_Slot_Index to query the `theme-and-appearance` subsystem's style-slot table (Requirement 3 of theme-and-appearance) for foreground colour, background colour, bold, italic, underline, and case transformation.
3. WHEN the active theme changes (hot-reload, mode switch, or theme switch), THE viewport renderer SHALL invalidate its style caches and repaint using the new theme's style-slot values without the highlighting engine needing to re-lex the document.
4. THE engine SHALL provide a `style_slot_count() → u8` method reporting how many base style indices the active lexer uses, enabling the theme system to provide defaults for unthemed indices.
5. THE language-service subsystem SHALL provide a mapping from style index to semantic token name (e.g., index 1 → "comment", index 2 → "keyword") so that theme files can define styles by semantic name rather than numeric index.
6. WHEN a sub-style index is queried from the theme system and no explicit override exists for that sub-style, THE theme system SHALL inherit all visual attributes from the sub-style's base style entry.

---

### Requirement 13: Lexer Lifecycle and Document Binding

**User Story:** As the workbench managing multiple open documents, I want each document to have its own lexer instance bound when the language is detected, and for the lexer to be replaced when the language changes, so that highlighting is always correct for the active language.

**Source:** [LEX-INFRA] Lexer instantiation and binding; [SCI-DOC-13] per-document lexer; language-service integration.

#### Acceptance Criteria

1. WHEN a document is opened and the `language-service` detects its language, THE engine SHALL instantiate the corresponding `Lexer` implementation from the lexer registry and bind it to the document's style context.
2. WHEN the `language-service` cannot detect a language (unknown file type), THE engine SHALL leave the document unbound (no lexer) and all text SHALL have the default style index (0).
3. WHEN the user manually changes the language assignment for a document, THE engine SHALL unbind the previous lexer, bind the new lexer, invalidate all styling and fold-level data, and trigger full re-highlighting.
4. WHEN a document is closed, THE engine SHALL release the associated lexer instance, style buffer, and per-line state, freeing all memory associated with that document's highlighting state.
5. THE engine SHALL populate the bound lexer's Keyword_Sets from the language definition provided by `language-service` at bind time, calling `set_keywords(set_index, words)` for each defined set.
6. THE engine SHALL set all configured Lexer_Properties on the bound lexer at bind time, reading values from the `configuration-system` via the language definition.
7. WHEN a new lexer is registered at runtime (e.g., via a plugin), THE engine SHALL make it available for language-service detection and binding without requiring a restart; documents currently assigned to that language SHALL be re-bound on next language detection.

---

### Requirement 14: Style Context Helper

**User Story:** As a lexer implementor, I want a convenient helper structure (StyleContext) that provides the current character, lookahead, state tracking, and style assignment methods, so that I can write lexers concisely without manual position management.

**Source:** [LEX-SUPPORT] StyleContext helper; Scintilla Lexer.txt (character-based lexer approach).

#### Acceptance Criteria

1. THE engine SHALL provide a `StyleContext` struct that exposes: `ch()` (current character), `ch_next()` (next character), `ch_prev()` (previous character), `state()` (current Lexer_State), `start_position()` (byte position of current token start).
2. THE `StyleContext` SHALL provide a `set_state(new_state: LexerState)` method that assigns the current style to all characters from the token start to the current position, then transitions to the new state.
3. THE `StyleContext` SHALL provide a `forward()` method that advances the position by one character (handling multi-byte UTF-8 sequences correctly).
4. THE `StyleContext` SHALL provide a `forward_bytes(count: usize)` method that advances the position by the specified number of bytes.
5. THE `StyleContext` SHALL provide a `match_keyword(word_list: &WordList) → Option<KeywordSetIndex>` method that checks whether the current token (from start to current position) matches any keyword in any active set, returning the matching set index.
6. THE `StyleContext` SHALL provide a `at_line_start() → bool` method that returns true if the current position is at the beginning of a line.
7. THE `StyleContext` SHALL provide a `at_line_end() → bool` method that returns true if the current character is a line-ending character (CR, LF, or the last character before a CRLF pair).
8. THE `StyleContext` SHALL provide a `more() → bool` method that returns true if there are more characters to process within the specified range.
9. THE `StyleContext` SHALL handle document boundaries safely: `ch_next()` at the end of the document SHALL return a null/sentinel character (e.g., `'\0'`) rather than panicking or returning invalid data.

---

### Requirement 15: Integration with Display-Line-Mapping and Text-Decorations

**User Story:** As the viewport renderer, I want syntax highlighting to integrate cleanly with the fold system and text decorations, so that folded regions use correct fold levels from the lexer and indicators render in the correct layer relative to syntax-coloured text.

**Source:** [WB] Integration with display-line-mapping for fold-level detection; Integration with text-decorations for indicator layer ordering.

#### Acceptance Criteria

1. THE `display-line-mapping` subsystem SHALL query fold levels exclusively from the syntax-highlighting engine's `fold_level_at(line)` API to determine fold region boundaries and fold headers — the display-line-mapping SHALL NOT compute fold levels independently.
2. WHEN the syntax-highlighting engine updates fold levels for a range of lines (due to edit or re-highlight), IT SHALL emit a fold-level-changed notification containing the affected line range, enabling the display-line-mapping to update fold state incrementally.
3. THE rendering pipeline SHALL apply indicator decorations (from `text-decorations`) either above or below syntax-coloured text depending on each indicator's `under` property: indicators with `under = true` render beneath syntax colours; indicators with `under = false` render above.
4. THE syntax-highlighting engine's style data and the text-decorations engine's indicator data SHALL be independently queryable for the same character range — they do not interfere with each other's storage.
5. WHEN the syntax-highlighting engine re-highlights a region, IT SHALL NOT modify or invalidate indicator decorations applied to that region — indicator lifecycle is managed independently by the text-decorations subsystem.
6. THE engine SHALL provide a `fold_level_range(start_line, end_line) → impl Iterator<Item = (LineNumber, u16, FoldFlags)>` method for efficient bulk fold-level queries by the display-line-mapping during fold-region calculation.
