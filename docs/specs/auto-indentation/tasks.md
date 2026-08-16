# Implementation Plan: Auto-Indentation (`ff-auto-indent`)

## Overview

This plan covers the complete implementation of the `ff-auto-indent` crate — the language-aware automatic indentation engine for FileForgeWorkbench. The crate computes indentation adjustments triggered by newline insertion, provides explicit indent/unindent commands, handles block comment auto-continuation, and supports smart indent patterns defined per-language in TOML files.

This is a **Wave 7 (Language and Highlighting)** sub-project. It depends on:
- `ff-logging` (Wave 0) — structured diagnostics and DEBUG-level indent decision logging
- `ff-command` (Wave 2) — command registration for `edit.indent` and `edit.unindent`
- `ff-config` (Wave 2) — indent settings, hot-reload callbacks, EditorConfig precedence
- `ff-document-model` (Wave 4) — line content access and line count
- `ff-edit-operations` (Wave 4) — EditorTransaction, newline trigger hook, line modification
- `ff-undo-redo` (Wave 4) — transaction grouping for single-step undo
- `ff-language-service` (Wave 7, peer) — language definitions, indent patterns, comment markers, syntax state

---

## Tasks

- [ ] 1. Crate scaffolding and core types
  - [ ] 1.1 Create `crates/ff-auto-indent/Cargo.toml` with dependencies (regex, thiserror, tracing, proptest dev-dep) and deps on `ff-logging`, `ff-command`, `ff-config`, `ff-document-model`, `ff-edit-operations`, `ff-undo-redo`, `ff-language-service`
  - [ ] 1.2 Create `crates/ff-auto-indent/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `config/mod.rs`, `config/indent_config.rs`, `config/mode.rs`, `engine.rs`, `compute/mod.rs`, `compute/maintain.rs`, `compute/smart.rs`, `compute/brace_expand.rs`, `compute/comment_continue.rs`, `pattern/mod.rs`, `pattern/matcher.rs`, `pattern/indent_patterns.rs`, `commands/mod.rs`, `commands/indent.rs`, `commands/unindent.rs`, `types.rs`, `error.rs`
  - [ ] 1.4 Add `ff-auto-indent` to workspace `Cargo.toml` members list
  - [ ] 1.5 Define `AutoIndentMode` enum (None, Maintain, Smart) with `Default` impl returning Smart
  - [ ] 1.6 Define `IndentStyle` enum (Tabs, Spaces) with `Default` impl returning Spaces
  - [ ] 1.7 Define `IndentConfig` struct with fields: `indent_size` (u8), `tab_size` (u8), `style` (IndentStyle)
  - [ ] 1.8 Implement `IndentConfig::indent_string()` returning tab char or N spaces based on style
  - [ ] 1.9 Implement `IndentConfig::whitespace_for_level(level)` computing physical whitespace for a given indent level
  - [ ] 1.10 Implement `IndentConfig::columns_to_level()` and `level_to_columns()` conversion methods
  - [ ] 1.11 Define `IndentLevel` newtype with `new()`, `value()`, `increment()`, and `decrement()` (clamped at zero)
  - [ ] 1.12 Write unit tests for AutoIndentMode default, IndentConfig indent_string computation, whitespace_for_level, columns/level conversions, IndentLevel increment/decrement clamping
  - Covers: Requirement 1 (AC 1.1, 1.5), Requirement 4 (AC 4.6)

- [ ] 2. Configuration integration and mode management
  - [ ] 2.1 Implement `AutoIndentMode::from_str()` parsing "none", "maintain", "smart" (case-insensitive) with `UnknownMode` error for unrecognised values
  - [ ] 2.2 Implement configuration-system integration: read `editor.auto_indent`, `editor.indent_size`, `editor.tab_size`, `editor.use_tabs` keys from `ff-config`
  - [ ] 2.3 Implement per-language override: when language TOML defines `[indent]` table with `indent_size`, `tab_size`, `use_tabs`, override global settings
  - [ ] 2.4 Implement hot-reload callback: on configuration change, update `AutoIndentEngine` mode and config without document close/reopen
  - [ ] 2.5 Implement EditorConfig precedence: when `indent_style` and `indent_size` from EditorConfig are active, override global config for that file
  - [ ] 2.6 Implement `effective_mode()` logic: return Smart if language has indent patterns, Maintain otherwise, unless user explicitly set mode
  - [ ] 2.7 Write unit tests for mode parsing (valid/invalid), config loading, per-language override, EditorConfig precedence, effective_mode fallback
  - Covers: Requirement 1 (AC 1.2–1.6)

- [ ] 3. Maintain-indent logic
  - [ ] 3.1 Define `LineIndentInfo` struct with fields: `whitespace` (String), `column_width` (u32), `level` (IndentLevel), `first_content_column` (u32)
  - [ ] 3.2 Implement `parse_line_indent(line_text, tab_size) -> LineIndentInfo` scanning leading whitespace, expanding tabs to column positions
  - [ ] 3.3 Implement `MaintainIndentComputer::compute(config, context) -> IndentResult` copying reference line's leading whitespace to new line
  - [ ] 3.4 Implement caret-at-column-zero case: when `caret_column == 0`, return `SimpleIndent { whitespace: "" }` (zero indent)
  - [ ] 3.5 Implement caret-within-indent case: when caret is within leading whitespace, reproduce only whitespace before caret position using `extract_whitespace_to_column()`
  - [ ] 3.6 Implement `extract_whitespace_to_column(line_text, column, tab_size) -> String` generating whitespace up to the specified column respecting use_tabs setting
  - [ ] 3.7 Write unit tests for maintain-indent with spaces-only lines, tabs-only lines, mixed tabs/spaces, caret-at-column-zero, caret-within-indent, empty reference line
  - Covers: Requirement 2 (AC 2.1–2.6)

- [ ] 4. Smart-indent pattern engine
  - [ ] 4.1 Define `CompiledPattern` struct wrapping `regex::Regex` with source string for diagnostics
  - [ ] 4.2 Implement `CompiledPattern::try_compile(source) -> Option<Self>` with WARN log on invalid regex, returning None
  - [ ] 4.3 Implement `CompiledPattern::is_match(text) -> bool` delegating to compiled regex
  - [ ] 4.4 Define `IndentPatterns` struct with optional fields: `increase_pattern`, `decrease_pattern`, `statement_pattern`, `statement_end_pattern`, `block_start`, `block_end`
  - [ ] 4.5 Implement `PatternMatcher::from_language_definition()` constructing from raw TOML pattern strings, compiling each with `try_compile`, caching results
  - [ ] 4.6 Implement `PatternMatcher` convenience methods: `matches_increase()`, `matches_decrease()`, `matches_statement()`, `matches_statement_end()`, `matches_block_start()`, `matches_block_end()`
  - [ ] 4.7 Implement pattern matching against non-comment content: strip comment/string portions of line before matching (using syntax state from language-service)
  - [ ] 4.8 Write unit tests for valid regex compilation, invalid regex handling, pattern matching against sample lines, non-comment content extraction, empty pattern (None) never matches
  - Covers: Requirement 3 (AC 3.2, 3.4), Requirement 9 (AC 9.1–9.2, 9.7)

- [ ] 5. Indent increase logic
  - [ ] 5.1 Implement `SmartIndentComputer::compute_newline(config, patterns, context) -> IndentResult` examining reference line against increase_pattern
  - [ ] 5.2 Implement indent-increase detection: if reference line matches increase_pattern (and not decrease_pattern), new line gets reference_level + 1
  - [ ] 5.3 Implement net-effect calculation via `compute_net_adjustment()`: when both increase and decrease match, effects cancel (net = 0)
  - [ ] 5.4 Implement fallback to Maintain when no increase_pattern is defined for active language
  - [ ] 5.5 Implement statement-indent logic: when reference line matches `statement_pattern`, indent only the immediately following line by one level; subsequent lines return to original
  - [ ] 5.6 Implement statement-end detection: when reference line matches `statement_end_pattern`, signal return to pre-statement indent level
  - [ ] 5.7 Write unit tests for increase-only match, decrease-only match, both match (net cancel), no patterns (fallback), statement continuation indent, statement end return
  - Covers: Requirement 3 (AC 3.1, 3.3, 3.5–3.6)

- [ ] 6. Indent decrease logic
  - [ ] 6.1 Implement `SmartIndentComputer::check_decrease_trigger(config, patterns, line_text, caret_column) -> Option<IndentLevel>` detecting when a typed character completes a decrease pattern
  - [ ] 6.2 Implement real-time decrease detection: evaluate decrease_pattern against leading whitespace + characters typed so far on the line
  - [ ] 6.3 Implement guard: decrease only triggers when line content before caret is only whitespace (no pre-existing non-whitespace content)
  - [ ] 6.4 Implement floor clamping: indent level never reduced below zero (column 0)
  - [ ] 6.5 Implement `compute_char_indent()` on AutoIndentEngine as the public API for character-typed decrease trigger
  - [ ] 6.6 Implement no-op when decrease_pattern is not defined for active language
  - [ ] 6.7 Write unit tests for decrease trigger on `}`, decrease on `end`, no trigger when line has content before caret, floor clamping at zero, no pattern defined
  - Covers: Requirement 4 (AC 4.1–4.7)

- [ ] 7. Block expansion (Enter between braces)
  - [ ] 7.1 Implement `BraceExpander::try_expand(config, patterns, context) -> Option<IndentResult>` detecting caret between block_start and block_end on same line
  - [ ] 7.2 Implement three-line expansion: (a) split at caret, (b) middle line indented one level deeper, (c) closing delimiter at original indent level
  - [ ] 7.3 Implement `BraceExpansion` variant of IndentResult with `middle_whitespace`, `closing_whitespace`, and `closing_text` fields
  - [ ] 7.4 Implement caret positioning: result indicates caret should be at end of indentation on middle line
  - [ ] 7.5 Implement no-op when block_start or block_end patterns are not defined for active language
  - [ ] 7.6 Implement pattern detection: verify character immediately before caret matches block_start and character immediately after caret matches block_end
  - [ ] 7.7 Write unit tests for `{}` expansion, `()` expansion (if configured), nested braces, no expansion when patterns undefined, no expansion when caret not between delimiters
  - Covers: Requirement 5 (AC 5.1–5.5)

- [ ] 8. Comment continuation
  - [ ] 8.1 Define `CommentMarkers` struct with fields: `block_start`, `block_end`, `block_continue`, `line_prefix`, `continue_line` (bool)
  - [ ] 8.2 Implement `CommentContinuer::compute(config, markers, context) -> Option<IndentResult>` detecting comment context and producing continuation marker
  - [ ] 8.3 Implement block comment continuation: when caret is inside block comment (not on closing line), insert `block_continue` marker aligned with preceding comment line
  - [ ] 8.4 Implement line comment continuation: when reference line is a line-comment and `continue_line` is enabled, prefix new line with `line_prefix` + space
  - [ ] 8.5 Implement end-of-block-comment detection: when reference line contains block_end (`*/`), do NOT insert continuation marker
  - [ ] 8.6 Implement double-Enter break-out: when reference line has only whitespace + continuation marker (no content after), and user presses Enter again, produce `RemovePreviousContinuation` result removing the marker from previous line
  - [ ] 8.7 Implement `is_empty_continuation(line_text, markers) -> bool` detecting lines with only whitespace + marker
  - [ ] 8.8 Implement syntax-state-based detection: consult `in_comment` / `in_block_comment` from IndentContext (sourced from language-service syntax state) rather than text-only heuristics
  - [ ] 8.9 Write unit tests for block comment continue (`/* ... */`), line comment continue (`//`), end-of-block no-continue, double-Enter break-out, continue_line disabled, markers not defined
  - Covers: Requirement 6 (AC 6.1–6.7)

- [ ] 9. Indent/Unindent commands
  - [ ] 9.1 Implement `IndentCommand` struct with `register(registry)` registering `edit.indent` command with Tab keybinding and display name "Indent"
  - [ ] 9.2 Implement `IndentCommand::execute(engine, line_contents) -> IndentCommandAction` prepending one indent_string to each line
  - [ ] 9.3 Implement whitespace normalisation on indent: when lines have mixed leading whitespace, normalise to current `use_tabs` setting before adding new level
  - [ ] 9.4 Implement `UnindentCommand` struct with `register(registry)` registering `edit.unindent` command with Shift+Tab keybinding and display name "Unindent"
  - [ ] 9.5 Implement `UnindentCommand::execute(engine, line_contents) -> IndentCommandAction` removing one indent_level from each line
  - [ ] 9.6 Implement unindent floor: when line has less than one full indent_level of whitespace, remove all remaining whitespace (result is column 0)
  - [ ] 9.7 Implement unindent whitespace handling: one tab counts as tab_size columns, remove spaces up to indent_size columns per unindent
  - [ ] 9.8 Implement single-line unindent: when no selection, `edit.unindent` unindents the caret's current line
  - [ ] 9.9 Implement Tab delegation: when no multi-line selection is active, Tab delegates to normal tab insertion in edit-operations (not indent command)
  - [ ] 9.10 Implement rectangular selection support: when rectangular selection is active, indent/unindent all lines spanned by the selection
  - [ ] 9.11 Define `IndentCommandAction` struct with `lines: Vec<u64>` and `new_indents: Vec<Option<String>>` describing the result
  - [ ] 9.12 Write unit tests for indent single/multi-line, unindent single/multi-line, unindent below floor, mixed whitespace normalisation, Tab delegation, rectangular selection, modified line marker setting
  - Covers: Requirement 7 (AC 7.1–7.6), Requirement 8 (AC 8.1–8.7), Requirement 10 (AC 10.6)

- [ ] 10. AutoIndentEngine facade and integration
  - [ ] 10.1 Implement `AutoIndentEngine` struct composing mode, config, patterns, comment_markers with constructor `new()` and `with_config()`
  - [ ] 10.2 Implement `compute_newline_indent(context) -> IndentResult` as the main entry point coordinating mode selection: None → NoIndent, Maintain → MaintainIndentComputer, Smart → priority order (BraceExpander → CommentContinuer → SmartIndentComputer)
  - [ ] 10.3 Implement `compute_char_indent(line_text, caret_column) -> Option<String>` as the character-typed entry point delegating to SmartIndentComputer decrease check
  - [ ] 10.4 Implement `set_language_patterns()` and `set_comment_markers()` for language change events
  - [ ] 10.5 Implement `set_mode()` and `set_config()` for hot-reload callbacks
  - [ ] 10.6 Define `IndentContext` struct with fields: `reference_line`, `reference_text`, `caret_column`, `in_comment`, `in_block_comment`, `is_empty_comment_continuation`
  - [ ] 10.7 Implement EditorTransaction integration: auto-indent result is applied within the same transaction as the newline insertion (coordination with edit-operations)
  - [ ] 10.8 Implement multi-caret support: compute indent independently for each caret's reference line, all within the same UndoGroup
  - [ ] 10.9 Implement "don't fight the user" logic: after auto-indent is applied, do not re-indent if user immediately edits the whitespace
  - [ ] 10.10 Implement DEBUG-level logging for each indent decision: reference line number, matched pattern (if any), resulting indent level
  - [ ] 10.11 Write unit tests for engine facade coordination, mode dispatch, None mode returns NoIndent, multi-caret independence, transaction grouping, DEBUG logging output
  - Covers: Requirement 10 (AC 10.1–10.7), Requirement 1 (AC 1.4)

- [ ] 11. Language TOML definition support
  - [ ] 11.1 Implement loading `[indent]` table from language TOML: `increase_pattern`, `decrease_pattern`, `statement_pattern`, `statement_end_pattern`, `block_start`, `block_end`
  - [ ] 11.2 Implement loading `[indent]` override keys: `indent_size`, `tab_size`, `use_tabs` taking precedence over global editor settings
  - [ ] 11.3 Implement loading `[comment]` table: `block_start`, `block_end`, `block_continue`, `line_prefix`, `continue_line`
  - [ ] 11.4 Implement language change response: when active language changes (via language-service event), reload patterns and markers from new definition
  - [ ] 11.5 Implement fallback: when language definition lacks `[indent]` table, use global settings with Maintain behaviour
  - [ ] 11.6 Implement regex caching: compile all patterns at language load time, cache compiled regexes, log WARN for invalid patterns
  - [ ] 11.7 Write unit tests for TOML loading (complete definition, partial definition, missing indent table, invalid regex pattern handling, comment table loading)
  - Covers: Requirement 9 (AC 9.1–9.7)

- [ ] 12. Error handling
  - [ ] 12.1 Define `AutoIndentError` enum: `PatternCompileError`, `InvalidConfig`, `LineOutOfRange`, `UnknownMode`, `CommandRegistration`
  - [ ] 12.2 Implement `thiserror::Error` derive with `[auto-indent] operation: description` message format for all variants
  - [ ] 12.3 Implement graceful degradation: invalid patterns treated as non-matching (WARN log), invalid config values fall back to defaults (WARN log)
  - [ ] 12.4 Write unit tests for all error variants, message formatting (≤200 chars), and graceful degradation paths
  - Covers: Cross-cutting error handling standard

- [ ] 13. Property-based tests
  - [ ] 13.1 Write PBT: indent level never goes negative (Property 1)
  - [ ] 13.2 Write PBT: maintain-indent exactly reproduces reference line's leading whitespace (Property 2)
  - [ ] 13.3 Write PBT: enter at column zero produces no indent (Property 3)
  - [ ] 13.4 Write PBT: smart indent with increase_pattern always adds exactly one indent_level (Property 4)
  - [ ] 13.5 Write PBT: smart indent net cancellation when both patterns match (Property 5)
  - [ ] 13.6 Write PBT: indent command adds one indent_string per line (Property 6)
  - [ ] 13.7 Write PBT: unindent never goes below zero (Property 7)
  - [ ] 13.8 Write PBT: unindent removes exactly one level when possible (Property 8)
  - [ ] 13.9 Write PBT: None mode always produces NoIndent (Property 9)
  - [ ] 13.10 Write PBT: indent string consistency with style (Property 10)
  - [ ] 13.11 Write PBT: brace expansion middle line is one level deeper (Property 11)
  - [ ] 13.12 Write PBT: indent/unindent roundtrip is identity (Property 12)
  - [ ] 13.13 Write PBT: invalid regex safety — try_compile returns None, matcher never matches (Property 13)
  - [ ] 13.14 Write PBT: caret-within-indent preserves partial whitespace (Property 14)
  - Covers: Requirements 1–10 (see Property-Based Test Definitions below)

- [ ] 14. Integration tests
  - [ ] 14.1 Write integration test: full newline indent cycle — configure engine with C-like patterns, insert newline after `{`, verify indent increased by one level
  - [ ] 14.2 Write integration test: decrease on closing brace — type `}` on indented blank line, verify indent decreased by one level
  - [ ] 14.3 Write integration test: enter-between-braces — press Enter between `{}`, verify three-line expansion with correct relative indentation
  - [ ] 14.4 Write integration test: block comment continuation — press Enter inside `/* ... */`, verify `* ` marker inserted and aligned
  - [ ] 14.5 Write integration test: line comment continuation — press Enter after `// comment`, verify `// ` prefix on new line
  - [ ] 14.6 Write integration test: double-Enter comment break-out — press Enter twice on empty continuation line, verify marker removed
  - [ ] 14.7 Write integration test: indent/unindent multi-line selection — select 5 lines, Tab indents all, Shift+Tab unindents all, verify roundtrip
  - [ ] 14.8 Write integration test: language change — switch from C to Python patterns, verify next indent uses new language rules
  - [ ] 14.9 Write integration test: hot-reload — change indent_size from 4 to 2 via config, verify subsequent indents use new size
  - [ ] 14.10 Write integration test: multi-caret indent — two carets on different lines, Enter pressed, each gets independent correct indent
  - [ ] 14.11 Write integration test: None mode — configure None mode, press Enter, verify new line at column 0 with no whitespace
  - Covers: End-to-end validation across Requirements 1–10

---

## Property-Based Test Definitions

### Property 1: IndentLevel Decrement Floor

**Validates: Requirement 4.6**

- **Statement:** For any `IndentLevel`, calling `decrement()` never produces a value below zero. When the level is already zero, `decrement()` returns zero.
- **Strategy:** Generate `level: u32` in range [0, 1000]. Construct `IndentLevel::new(level)`.
- **Invariant:** `level.decrement().value() >= 0` AND `IndentLevel::new(0).decrement().value() == 0`

### Property 2: Maintain-Indent Preserves Reference Whitespace

**Validates: Requirement 2.1**

- **Statement:** In Maintain mode, when Enter is pressed at or after the first non-whitespace character, the new line's indentation exactly equals the reference line's leading whitespace (reproduced using the current use_tabs setting).
- **Strategy:** Generate:
  - `indent_size`: u8 in [1, 8]
  - `tab_size`: u8 in [1, 8]
  - `style`: IndentStyle (Tabs or Spaces)
  - `leading_ws`: random whitespace string (tabs/spaces, length 0–20)
  - `content`: random non-whitespace ASCII string (length 1–40)
  - `reference_text`: `leading_ws + content`
  - `caret_column`: column >= first_non_ws_column
- **Invariant:** `MaintainIndentComputer::compute(config, context).whitespace` has the same column width as `leading_ws` when expanded with tab_size

### Property 3: Enter at Column Zero Produces No Indent

**Validates: Requirement 2.5**

- **Statement:** When Enter is pressed at column 0, the resulting new line has zero indentation regardless of reference line content or mode (Maintain or Smart).
- **Strategy:** Generate:
  - `reference_text`: random string with arbitrary leading whitespace
  - `mode`: Maintain or Smart
  - `caret_column`: always 0
- **Invariant:** Result whitespace is empty string OR result is NoIndent

### Property 4: Smart Indent Increase Adds Exactly One Level

**Validates: Requirement 3.1**

- **Statement:** When the reference line matches the indent-increase pattern and does NOT match the decrease pattern, the new line's indent level is exactly one more than the reference line's level.
- **Strategy:** Generate:
  - `base_indent`: random whitespace (0–5 levels)
  - `content`: line content that matches a known increase pattern (e.g., ends with `{`)
  - `indent_size`: u8 in [2, 8]
  - Ensure content does NOT match decrease pattern
- **Invariant:** `new_indent_columns == reference_indent_columns + indent_size`

### Property 5: Smart Indent Net Cancellation

**Validates: Requirement 3.5**

- **Statement:** When the reference line matches both the indent-increase and indent-decrease patterns, the net effect is zero — the new line has the same indent level as the reference line.
- **Strategy:** Generate:
  - `base_indent`: random whitespace (0–5 levels)
  - `content`: line matching both increase and decrease (e.g., `} else {`)
  - `indent_size`: u8 in [2, 8]
- **Invariant:** `new_indent_columns == reference_indent_columns`

### Property 6: Indent Command Adds One IndentString Per Line

**Validates: Requirement 7.1**

- **Statement:** For any set of lines, the indent command prepends exactly one `indent_string()` worth of whitespace columns to each line's existing leading whitespace.
- **Strategy:** Generate:
  - `line_count`: usize in [1, 20]
  - `lines`: random strings with varying leading whitespace (0–40 columns)
  - `indent_size`: u8 in [2, 8]
  - `style`: IndentStyle
- **Invariant:** For each line: `new_leading_columns == old_leading_columns + indent_size`

### Property 7: Unindent Never Goes Below Zero

**Validates: Requirement 8.2**

- **Statement:** For any line, the unindent command never produces negative indentation — the minimum result is zero leading whitespace (empty string).
- **Strategy:** Generate:
  - `lines`: random strings with varying leading whitespace (0–40 columns, including lines with < indent_size whitespace)
  - `indent_size`: u8 in [2, 8]
- **Invariant:** `new_leading_columns >= 0` AND when `old_leading_columns < indent_size` then `new_leading_whitespace == ""`

### Property 8: Unindent Removes Exactly One Level When Possible

**Validates: Requirement 8.1**

- **Statement:** When a line has at least one full indent level of leading whitespace, unindent removes exactly `indent_size` columns of whitespace.
- **Strategy:** Generate:
  - `indent_level`: u32 in [1, 10]
  - `indent_size`: u8 in [2, 8]
  - `line`: string with `indent_level * indent_size` spaces + random content
- **Invariant:** `new_leading_columns == old_leading_columns - indent_size`

### Property 9: None Mode Produces No Indentation

**Validates: Requirement 10.3**

- **Statement:** When the auto-indent mode is None, `compute_newline_indent` always returns `NoIndent` regardless of reference line content, patterns, or caret position.
- **Strategy:** Generate:
  - `reference_text`: any random string (with/without indent patterns matching)
  - `caret_column`: any valid column
  - `patterns`: any IndentPatterns (including ones that would normally trigger increase/decrease)
- **Invariant:** `compute_newline_indent(context) == IndentResult::NoIndent`

### Property 10: Indent String Consistency

**Validates: Requirement 1.5**

- **Statement:** The indent string produced by `IndentConfig` is always consistent with the configured style: Tabs → single tab character, Spaces → exactly `indent_size` space characters. The indent string is never empty.
- **Strategy:** Generate:
  - `indent_size`: u8 in [1, 8]
  - `tab_size`: u8 in [1, 8]
  - `style`: IndentStyle (Tabs or Spaces)
- **Invariant:** `style == Tabs ⟹ indent_string() == "\t"` AND `style == Spaces ⟹ indent_string() == " ".repeat(indent_size)` AND `indent_string().len() > 0`

### Property 11: Brace Expansion Middle Line Is One Level Deeper

**Validates: Requirement 5.1**

- **Statement:** When enter-between-braces expansion produces a `BraceExpansion` result, the middle line's indent is exactly one indent_size deeper than the reference line, and the closing line has the same indent as the reference.
- **Strategy:** Generate:
  - `base_indent_level`: u32 in [0, 10]
  - `indent_size`: u8 in [2, 8]
  - `style`: IndentStyle
  - Context simulating caret between `{` and `}`
- **Invariant:** `columns(middle_whitespace) == columns(reference_indent) + indent_size` AND `columns(closing_whitespace) == columns(reference_indent)`

### Property 12: Indent/Unindent Roundtrip

**Validates: Requirements 7.1, 8.1**

- **Statement:** For any line with at least one indent level of leading whitespace, applying indent followed by unindent returns the line to its original indentation (roundtrip identity).
- **Strategy:** Generate:
  - `indent_size`: u8 in [2, 8]
  - `style`: IndentStyle::Spaces (tabs roundtrip is also tested separately)
  - `original_indent`: random multiple of indent_size in [indent_size, indent_size*10]
  - `content`: random non-whitespace content
  - `line`: spaces(original_indent) + content
- **Invariant:** `unindent(indent(line)).leading_whitespace == line.leading_whitespace`

### Property 13: Invalid Regex Safety

**Validates: Requirement 9.7**

- **Statement:** When given an invalid regex string, `CompiledPattern::try_compile` returns `None` and does not panic. A `PatternMatcher` constructed with a failed pattern never matches any input.
- **Strategy:** Generate:
  - `invalid_regex`: strings containing unbalanced brackets, invalid escapes, or other regex syntax errors (e.g., `"[unclosed"`, `"*invalid"`, `"(?P<"`, `"\\Q"`)
  - `test_input`: any random string to match against
- **Invariant:** `CompiledPattern::try_compile(invalid) == None` AND `matcher.matches_increase(any_input) == false` when increase pattern failed compilation

### Property 14: Caret-Within-Indent Preserves Partial Whitespace

**Validates: Requirement 2.6**

- **Statement:** When Enter is pressed within the leading whitespace (caret_column < first_content_column), the new line receives only the whitespace corresponding to columns before the caret position.
- **Strategy:** Generate:
  - `leading_ws`: random whitespace (spaces/tabs, column width 4–40)
  - `content`: random non-whitespace content
  - `reference_text`: leading_ws + content
  - `caret_column`: u32 in [1, column_width(leading_ws) - 1] (within the indent, not at 0 or beyond)
  - `tab_size`: u8 in [2, 8]
- **Invariant:** `columns(result.whitespace) == caret_column`

---

## Task Dependency Graph

```json
{
  "phases": [
    {
      "id": 1,
      "label": "Core Types and Configuration",
      "tasks": ["1", "2", "12"],
      "dependsOn": []
    },
    {
      "id": 2,
      "label": "Maintain-Indent Logic",
      "tasks": ["3"],
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "Smart-Indent Pattern Engine",
      "tasks": ["4"],
      "dependsOn": [1]
    },
    {
      "id": 4,
      "label": "Indent Increase Logic",
      "tasks": ["5"],
      "dependsOn": [2, 3]
    },
    {
      "id": 5,
      "label": "Indent Decrease Logic",
      "tasks": ["6"],
      "dependsOn": [3, 4]
    },
    {
      "id": 6,
      "label": "Block Expansion",
      "tasks": ["7"],
      "dependsOn": [3, 4]
    },
    {
      "id": 7,
      "label": "Comment Continuation",
      "tasks": ["8"],
      "dependsOn": [2]
    },
    {
      "id": 8,
      "label": "Indent/Unindent Commands",
      "tasks": ["9"],
      "dependsOn": [1]
    },
    {
      "id": 9,
      "label": "Integration and Facade",
      "tasks": ["10", "11"],
      "dependsOn": [4, 5, 6, 7, 8]
    },
    {
      "id": 10,
      "label": "Property-Based Tests and Validation",
      "tasks": ["13", "14"],
      "dependsOn": [9]
    }
  ]
}
```

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Auto-Indent Mode Configuration | AC 1.1 | Task 1 (1.5–1.6) |
| Req 1: Auto-Indent Mode Configuration | AC 1.2 | Task 2 (2.6) |
| Req 1: Auto-Indent Mode Configuration | AC 1.3 | Task 2 (2.1–2.2) |
| Req 1: Auto-Indent Mode Configuration | AC 1.4 | Task 2 (2.4), Task 10 (10.5) |
| Req 1: Auto-Indent Mode Configuration | AC 1.5 | Task 1 (1.7–1.10), Task 2 (2.2) |
| Req 1: Auto-Indent Mode Configuration | AC 1.6 | Task 2 (2.5) |
| Req 2: Maintain Indent | AC 2.1 | Task 3 (3.3) |
| Req 2: Maintain Indent | AC 2.2 | Task 3 (3.2) |
| Req 2: Maintain Indent | AC 2.3 | Task 3 (3.6) |
| Req 2: Maintain Indent | AC 2.4 | Task 10 (10.7) |
| Req 2: Maintain Indent | AC 2.5 | Task 3 (3.4) |
| Req 2: Maintain Indent | AC 2.6 | Task 3 (3.5–3.6) |
| Req 3: Smart Indent — Increase | AC 3.1 | Task 5 (5.1–5.2) |
| Req 3: Smart Indent — Increase | AC 3.2 | Task 4 (4.4), Task 11 (11.1) |
| Req 3: Smart Indent — Increase | AC 3.3 | Task 5 (5.4) |
| Req 3: Smart Indent — Increase | AC 3.4 | Task 4 (4.7) |
| Req 3: Smart Indent — Increase | AC 3.5 | Task 5 (5.3) |
| Req 3: Smart Indent — Increase | AC 3.6 | Task 5 (5.5–5.6) |
| Req 4: Smart Indent — Decrease | AC 4.1 | Task 6 (6.1–6.2) |
| Req 4: Smart Indent — Decrease | AC 4.2 | Task 4 (4.4), Task 11 (11.1) |
| Req 4: Smart Indent — Decrease | AC 4.3 | Task 6 (6.6) |
| Req 4: Smart Indent — Decrease | AC 4.4 | Task 10 (10.7) |
| Req 4: Smart Indent — Decrease | AC 4.5 | Task 6 (6.2) |
| Req 4: Smart Indent — Decrease | AC 4.6 | Task 1 (1.11), Task 6 (6.4) |
| Req 4: Smart Indent — Decrease | AC 4.7 | Task 6 (6.3) |
| Req 5: Block Expansion | AC 5.1 | Task 7 (7.1–7.2) |
| Req 5: Block Expansion | AC 5.2 | Task 11 (11.1) |
| Req 5: Block Expansion | AC 5.3 | Task 10 (10.7) |
| Req 5: Block Expansion | AC 5.4 | Task 7 (7.5) |
| Req 5: Block Expansion | AC 5.5 | Task 7 (7.4) |
| Req 6: Comment Continuation | AC 6.1 | Task 8 (8.3) |
| Req 6: Comment Continuation | AC 6.2 | Task 8 (8.4) |
| Req 6: Comment Continuation | AC 6.3 | Task 8 (8.1), Task 11 (11.3) |
| Req 6: Comment Continuation | AC 6.4 | Task 8 (8.5) |
| Req 6: Comment Continuation | AC 6.5 | Task 10 (10.7) |
| Req 6: Comment Continuation | AC 6.6 | Task 8 (8.6–8.7) |
| Req 6: Comment Continuation | AC 6.7 | Task 8 (8.8) |
| Req 7: Indent Command | AC 7.1 | Task 9 (9.2) |
| Req 7: Indent Command | AC 7.2 | Task 9 (9.9) |
| Req 7: Indent Command | AC 7.3 | Task 9 (9.1) |
| Req 7: Indent Command | AC 7.4 | Task 10 (10.7) |
| Req 7: Indent Command | AC 7.5 | Task 9 (9.3) |
| Req 7: Indent Command | AC 7.6 | Task 9 (9.12) |
| Req 8: Unindent Command | AC 8.1 | Task 9 (9.5) |
| Req 8: Unindent Command | AC 8.2 | Task 9 (9.6) |
| Req 8: Unindent Command | AC 8.3 | Task 9 (9.8) |
| Req 8: Unindent Command | AC 8.4 | Task 9 (9.4) |
| Req 8: Unindent Command | AC 8.5 | Task 10 (10.7) |
| Req 8: Unindent Command | AC 8.6 | Task 9 (9.12) |
| Req 8: Unindent Command | AC 8.7 | Task 9 (9.7) |
| Req 9: Language TOML Rules | AC 9.1–9.2 | Task 11 (11.1) |
| Req 9: Language TOML Rules | AC 9.3 | Task 11 (11.2) |
| Req 9: Language TOML Rules | AC 9.4 | Task 11 (11.3) |
| Req 9: Language TOML Rules | AC 9.5 | Task 11 (11.4) |
| Req 9: Language TOML Rules | AC 9.6 | Task 11 (11.5) |
| Req 9: Language TOML Rules | AC 9.7 | Task 4 (4.2), Task 11 (11.6) |
| Req 10: Integration | AC 10.1 | Task 10 (10.7) |
| Req 10: Integration | AC 10.2 | Task 10 (10.9) |
| Req 10: Integration | AC 10.3 | Task 10 (10.2) |
| Req 10: Integration | AC 10.4 | Task 10 (10.1) |
| Req 10: Integration | AC 10.5 | Task 10 (10.8) |
| Req 10: Integration | AC 10.6 | Task 9 (9.10) |
| Req 10: Integration | AC 10.7 | Task 10 (10.10) |

---

## Notes

- This is a Wave 7 (Language and Highlighting) crate that is **GUI-independent** — no rendering framework dependency.
- The auto-indent engine operates purely on line content and metadata. The GUI shell triggers auto-indent through `edit-operations`; the subsystem returns the indentation to apply.
- All indent modifications are wrapped in EditorTransactions for single-step undo. The transaction grouping is coordinated with `ff-edit-operations` and `ff-undo-redo`.
- The `proptest` crate is used for property-based testing with a minimum of 100 iterations per property.
- Regex patterns use Rust's `regex` crate syntax. Invalid patterns are logged as WARN and treated as non-matching (graceful degradation).
- The `PatternMatcher` compiles and caches regexes at language load time to avoid per-keystroke compilation cost.
- Comment continuation relies on syntax state from `ff-language-service` to determine whether the caret is inside a comment, rather than using text-only heuristics.
- The "don't fight the user" principle (Req 10.2) means that once auto-indent is applied, the system does not re-indent if the user immediately modifies the whitespace on the same line.
- Multi-caret support computes indent independently for each caret's reference line context, all within the same UndoGroup.
- EditorConfig values (`indent_style`, `indent_size`) take precedence over global config when active for a file (detected by `ff-config`).
