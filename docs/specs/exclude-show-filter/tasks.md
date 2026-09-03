# Implementation Plan: Exclude/Show Filter (`ff-exclude-show-filter`)

## Overview

This plan covers the complete implementation of the `ff-exclude-show-filter` crate — the ISPF-style line visibility management engine for FileForgeWorkbench. The crate provides EXCLUDE/SHOW/RESET primary commands and X/Xn/XX line commands for hiding and revealing document lines without modifying document content.

This is a **Wave 5 (Command Engine)** sub-project that depends on `ff-display-line-mapping` (Wave 4) for per-line visibility storage, `ff-document-model` (Wave 4) for line content access during text-matching operations, and `ff-command` (Wave 2) for command registration.

Key design principles:
- **GUI-independent** — pure logical layer with no rendering dependencies
- **Non-undoable** — all operations modify transient session state only
- **Flat exclusion** — no hierarchy or fold levels; distinct from code folding
- **Delegation** — visibility storage lives in `display-line-mapping`; this crate drives state transitions

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-exclude-show-filter/Cargo.toml` with dependencies (thiserror, regex, proptest dev-dep) and deps on `ff-display-line-mapping`, `ff-document-model`, `ff-command`, `ff-logging`
  - [x] 1.2 Create `crates/ff-exclude-show-filter/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `exclusion_engine.rs`, `exclude_command.rs`, `show_command.rs`, `reset_command.rs`, `line_commands.rs`, `placeholder.rs`, `scope_iterators.rs`, `text_matcher.rs`, `commands.rs`, `error.rs`, `types.rs`
  - [x] 1.4 Add `ff-exclude-show-filter` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Core types and exclusion state model
  - [x] 2.1 Define `ExclusionBlock { start_line: usize, end_line: usize }` struct representing a contiguous range of excluded lines
  - [x] 2.2 Define `ExcludeScope` enum (Visible, All, Tagged, Range { start: usize, end: usize }) for command argument parsing
  - [x] 2.3 Define `TextMatchMode` enum (Literal, Regex, None) for exclude/show text matching
  - [x] 2.4 Define `ExcludeResult { lines_affected: usize, message: String }` struct for operation outcomes
  - [x] 2.5 Define `ShowResult { lines_shown: usize, message: String }` struct for show operation outcomes
  - [x] 2.6 Define `ResetVariant` enum (Default, Excluded, All) for RESET command parsing
  - [x] 2.7 Write unit tests for type construction and Display impls
  - Covers: Requirement 1 (AC 1.1–1.8), Requirement 2 (AC 2.6–2.9), Requirement 3 (AC 3.6–3.8)

- [x] 3. ExclusionEngine — state model and delegation layer
  - [x] 3.1 Implement `ExclusionEngine` struct holding a reference/trait object to `DisplayLineMapping` and `DocumentModel`
  - [x] 3.2 Implement `is_excluded(doc_line: usize) -> bool` delegating to `display_line_mapping.get_visible(doc_line) == false`
  - [x] 3.3 Implement `has_excluded_lines() -> bool` delegating to `display_line_mapping.hidden_lines()`
  - [x] 3.4 Implement `excluded_line_count() -> usize` iterating visibility state to count excluded lines
  - [x] 3.5 Implement `exclude_range(start_line: usize, end_line: usize)` calling `display_line_mapping.set_visible(start, end, false)`
  - [x] 3.6 Implement `show_range(start_line: usize, end_line: usize)` calling `display_line_mapping.set_visible(start, end, true)`
  - [x] 3.7 Implement `show_all()` calling `display_line_mapping.show_all()`
  - [x] 3.8 Write unit tests for all delegation methods with mock DisplayLineMapping
  - Covers: Requirement 1 (AC 1.1–1.8), Requirement 7 (AC 7.1–7.2, 7.5)

- [x] 4. Text matcher — literal and regex line matching
  - [x] 4.1 Implement `TextMatcher` struct with methods for literal text search within a line
  - [x] 4.2 Implement case-insensitive literal matching (default behaviour)
  - [x] 4.3 Implement case-sensitive literal matching (configurable)
  - [x] 4.4 Implement regex matching with compiled pattern against line content
  - [x] 4.5 Implement regex error handling — return descriptive error for invalid patterns
  - [x] 4.6 Implement `matches_line(line_content: &str, term: &str, mode: TextMatchMode) -> bool` unified interface
  - [x] 4.7 Write unit tests for literal matching (case-sensitive/insensitive), regex matching, and error cases
  - Covers: Requirement 2 (AC 2.1, 2.3), Requirement 3 (AC 3.4–3.5), Requirement 9 (AC 9.8)

- [x] 5. EXCLUDE / X primary command implementation
  - [x] 5.1 Implement `exclude_text(term: &str, scope: ExcludeScope)` — excludes visible lines containing literal text
  - [x] 5.2 Implement `exclude_text_all(term: &str)` — excludes ALL lines (regardless of current visibility) matching text
  - [x] 5.3 Implement `exclude_regex(pattern: &str, scope: ExcludeScope)` — excludes visible lines matching regex pattern
  - [x] 5.4 Implement `exclude_all()` — excludes every line in the document
  - [x] 5.5 Implement `exclude_tagged()` — excludes every line with `tagged = true`
  - [x] 5.6 Implement `exclude_range_by_number(start: usize, end: usize)` — excludes document lines n through m inclusive (1-based)
  - [x] 5.7 Implement status message generation: "{N} line(s) excluded" or "No lines matched"
  - [x] 5.8 Implement `X` alias registration ensuring identical argument parsing
  - [x] 5.9 Write unit tests for each EXCLUDE variant, status messages, and zero-match handling
  - Covers: Requirement 2 (AC 2.1–2.10)

- [x] 6. SHOW / INCLUDE primary command implementation
  - [x] 6.1 Implement `show_all_lines()` — clears excluded flag on every line (SHOW ALL)
  - [x] 6.2 Implement `show_excluded()` — clears excluded flag on all currently excluded lines (SHOW EXCLUDED)
  - [x] 6.3 Implement `show_nonexcluded()` — no-op with confirmation message (SHOW NONEXCLUDED)
  - [x] 6.4 Implement `show_text(term: &str)` — clears excluded flag on excluded lines containing literal text
  - [x] 6.5 Implement `show_regex(pattern: &str)` — clears excluded flag on excluded lines matching regex
  - [x] 6.6 Implement `INCLUDE` alias registration with identical argument parsing
  - [x] 6.7 Implement status message generation: "{N} line(s) shown" or "No excluded lines matched"
  - [x] 6.8 Write unit tests for each SHOW variant, alias equivalence, and zero-match handling
  - Covers: Requirement 3 (AC 3.1–3.9)

- [x] 7. RESET command implementation (exclusion aspects)
  - [x] 7.1 Implement `reset_default()` — clears all exclusion state (RESET with no args)
  - [x] 7.2 Implement `reset_excluded()` — clears only excluded flags, preserving tags and pending commands
  - [x] 7.3 Implement `reset_all()` — clears exclusion state as part of broader RESET ALL
  - [x] 7.4 Implement delegation to `display_line_mapping.show_all()` for efficient bulk reset
  - [x] 7.5 Implement status message: "RESET: {N} line(s) restored to view"
  - [x] 7.6 Write unit tests for each RESET variant, message formatting, and state preservation invariants
  - Covers: Requirement 4 (AC 4.1–4.7)

- [x] 8. X / Xn / XX line command implementation
  - [x] 8.1 Implement `exclude_single_line(doc_line: usize)` — excludes one line (X command)
  - [x] 8.2 Implement `exclude_n_lines(start_line: usize, count: usize)` — excludes n consecutive lines (Xn command)
  - [x] 8.3 Implement `exclude_block(start_line: usize, end_line: usize)` — excludes all lines in XX..XX block
  - [x] 8.4 Implement unpaired XX detection and pending state error message "XX requires a matching pair"
  - [x] 8.5 Implement status message reporting count of excluded lines
  - [x] 8.6 Implement immediate execution semantics (no primary command required to resolve)
  - [x] 8.7 Write unit tests for X, Xn, XX (paired and unpaired), and immediate execution
  - Covers: Requirement 5 (AC 5.1–5.7)

- [x] 9. Placeholder display model
  - [x] 9.1 Implement `exclusion_blocks() -> Vec<ExclusionBlock>` enumerating all contiguous excluded ranges
  - [x] 9.2 Implement `placeholder_text(block: &ExclusionBlock) -> String` generating "-- N line(s) excluded --" format
  - [x] 9.3 Implement `block_count() -> usize` returning total number of exclusion blocks
  - [x] 9.4 Implement `block_at_doc_line(doc_line: usize) -> Option<ExclusionBlock>` looking up block containing a given line
  - [x] 9.5 Implement automatic block merging when adjacent excluded lines create or extend a block
  - [x] 9.6 Implement automatic block splitting when a line within a block is made visible
  - [x] 9.7 Write unit tests for block enumeration, merging, splitting, placeholder text, and empty-document edge case
  - Covers: Requirement 6 (AC 6.1–6.8)

- [x] 10. Scope integration iterators
  - [x] 10.1 Implement `visible_lines_iter() -> impl Iterator<Item = usize>` iterating all currently visible line indices
  - [x] 10.2 Implement `excluded_lines_iter() -> impl Iterator<Item = usize>` iterating all currently excluded line indices
  - [x] 10.3 Implement efficient iteration using display-line-mapping visibility queries
  - [x] 10.4 Implement scope filter support for FIND/CHANGE with EXCLUDED/VISIBLE modifiers
  - [x] 10.5 Write unit tests for iterator correctness with various exclusion patterns
  - Covers: Requirement 8 (AC 8.1–8.7)

- [x] 11. Command framework integration
  - [x] 11.1 Register EXCLUDE command (and X alias) with metadata: name, aliases, syntax help, argument schema, non-undoable flag
  - [x] 11.2 Register SHOW command (and INCLUDE alias) with metadata: name, aliases, syntax help, argument schema, non-undoable flag
  - [x] 11.3 Register RESET command with metadata for variants (no-arg, EXCLUDED, TAGS, COMMANDS, ALL)
  - [x] 11.4 Register X, Xn, XX in the line-command parser's recognized command set
  - [x] 11.5 Implement argument parsing and validation: unterminated quotes, invalid regex, non-numeric range
  - [x] 11.6 Implement Edit mode and Browse/View mode support (non-destructive — valid in all modes)
  - [x] 11.7 Implement Lua scripting bridge compatibility via standard command dispatch API
  - [x] 11.8 Write unit tests for command registration, argument parsing, error messages, and mode compatibility
  - Covers: Requirement 9 (AC 9.1–9.8)

- [x] 12. Display-line integration and change notification
  - [x] 12.1 Implement change notification emission when exclusion state changes (trigger display-line-mapping notifications)
  - [x] 12.2 Implement verification that `doc_from_display` never resolves to an excluded line
  - [x] 12.3 Implement placeholder occupying exactly one display line per exclusion block
  - [x] 12.4 Implement scrollbar range calculation support (visible lines + one per placeholder)
  - [x] 12.5 Write unit tests for notification emission, display-line consistency, and placeholder display-line contribution
  - Covers: Requirement 7 (AC 7.1–7.7)

- [x] 13. Error handling
  - [x] 13.1 Define `ExcludeFilterError` enum: InvalidRegex, UnterminatedQuote, InvalidRange, InvalidArgument, LineOutOfRange
  - [x] 13.2 Implement error message formatting per `[exclude-filter] operation: description` standard
  - [x] 13.3 Implement argument validation guard: reject invalid input before modifying state
  - [x] 13.4 Write unit tests for all error variants and message formatting
  - Covers: Requirement 9 (AC 9.8), Cross-cutting Requirement 8

- [x] 14. Performance optimizations
  - [x] 14.1 Implement O(n) EXCLUDE ALL using range-based `set_visible(0, last_line, false)`
  - [x] 14.2 Implement O(1) amortized SHOW ALL / RESET via `display_line_mapping.show_all()`
  - [x] 14.3 Implement O(k) block enumeration using boundary tracking (not scanning all lines)
  - [x] 14.4 Implement efficient text-matching with early termination for large documents
  - [x] 14.5 Implement memory efficiency: O(1) overhead when no exclusions active (delegated to display-line-mapping one-to-one mode)
  - [x] 14.6 Write benchmarks for EXCLUDE ALL → SHOW 'text' workflow on 1M+ line documents
  - Covers: Requirement 10 (AC 10.1–10.6)

- [x] 15. Property-based tests
  - [x] 15.1 Write PBT: exclusion state consistency — excluded lines are invisible in display-line-mapping
  - [x] 15.2 Write PBT: SHOW reverses EXCLUDE — EXCLUDE then SHOW on same lines restores visibility
  - [x] 15.3 Write PBT: block contiguity invariant — no two adjacent blocks can merge further
  - [x] 15.4 Write PBT: RESET restores all visibility — after RESET, no line is excluded
  - [x] 15.5 Write PBT: excluded line count consistency — count matches actual number of excluded lines
  - [x] 15.6 Write PBT: EXCLUDE ALL + SHOW text filters correctly — only matching lines visible after workflow
  - [x] 15.7 Write PBT: display-line-mapping doc_from_display never returns excluded line
  - Covers: Requirements 1–7, 10 (see Property-Based Test Definitions below)

- [x] 16. Integration tests
  - [x] 16.1 Write integration test: full EXCLUDE → SHOW → RESET lifecycle
  - [x] 16.2 Write integration test: EXCLUDE ALL → SHOW 'text' filtering workflow
  - [x] 16.3 Write integration test: X/Xn/XX line commands with block merging
  - [x] 16.4 Write integration test: scope integration with find (VISIBLE/EXCLUDED modifiers)
  - [x] 16.5 Write integration test: large document (100K+ lines) EXCLUDE ALL performance
  - [x] 16.6 Write integration test: command framework dispatch of EXCLUDE/SHOW/RESET with argument parsing
  - Covers: End-to-end validation across Requirements 1–10

---

## Property-Based Test Definitions

### Property 1: Exclusion State Consistency

**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 7.1**

- **Statement:** For any sequence of exclude/show operations, a line marked as excluded SHALL always report `get_visible(line) == false` in the display-line-mapping, and a line not excluded SHALL report `get_visible(line) == true`.
- **Strategy:** Generate:
  - Document line count: [1, 5000]
  - Operation sequence: random mix of `exclude_range`, `show_range`, `show_all` (1–50 operations)
  - Query lines: random sample of document lines
- **Invariant:** `∀ line: engine.is_excluded(line) == !display_mapping.get_visible(line)`

### Property 2: SHOW Reverses EXCLUDE

**Validates: Requirements 2.1, 3.1, 3.4, 4.2**

- **Statement:** For any set of lines that are excluded via EXCLUDE and then shown via SHOW (with matching criteria), the lines SHALL return to visible state. Specifically: EXCLUDE 'text' followed by SHOW 'text' on the same document restores all excluded lines that matched.
- **Strategy:** Generate:
  - Document content: random lines (1–1000 lines, 0–200 chars each)
  - Search term: randomly chosen substring present in some lines
  - Initial state: all lines visible
- **Invariant:** After `exclude_text(term)` → `show_text(term)`: `∀ line that matched term: !engine.is_excluded(line)`

### Property 3: Block Contiguity Invariant

**Validates: Requirements 6.1, 6.5, 6.6**

- **Statement:** The list of ExclusionBlocks returned by `exclusion_blocks()` SHALL be maximally contiguous: no two adjacent blocks can be merged (there is always at least one visible line between any two blocks), and each block covers a contiguous range where every line is excluded.
- **Strategy:** Generate:
  - Document line count: [1, 2000]
  - Exclusion pattern: random subset of lines excluded
- **Invariant:** `∀ i: blocks[i].end_line + 1 < blocks[i+1].start_line` (gap between blocks) AND `∀ line in block: engine.is_excluded(line)`

### Property 4: RESET Restores All Visibility

**Validates: Requirements 4.1, 4.2, 4.4, 4.7**

- **Statement:** After any sequence of EXCLUDE operations followed by `reset_excluded()`, no line in the document SHALL remain excluded.
- **Strategy:** Generate:
  - Document line count: [1, 5000]
  - Operation sequence: random EXCLUDE operations (1–30 operations) of any variant
- **Invariant:** After `reset_excluded()`: `∀ line: !engine.is_excluded(line)` AND `engine.excluded_line_count() == 0`

### Property 5: Excluded Line Count Consistency

**Validates: Requirements 1.7, 1.8, 10.1**

- **Statement:** The value returned by `excluded_line_count()` SHALL always equal the number of lines for which `is_excluded(line)` returns true.
- **Strategy:** Generate:
  - Document line count: [1, 5000]
  - Operation sequence: random mix of exclude/show operations
- **Invariant:** `engine.excluded_line_count() == (0..line_count).filter(|&l| engine.is_excluded(l)).count()`

### Property 6: EXCLUDE ALL + SHOW Text Filtering

**Validates: Requirements 2.4, 3.4, 8.7**

- **Statement:** After `exclude_all()` followed by `show_text(term)`, the set of visible lines SHALL be exactly those lines whose content contains the search term.
- **Strategy:** Generate:
  - Document content: random lines (1–500 lines, 1–100 chars each)
  - Search term: randomly generated string (1–10 chars)
- **Invariant:** `∀ line: !engine.is_excluded(line) ↔ line_content(line).contains(term)` (case-insensitive)

### Property 7: Doc-from-Display Never Returns Excluded Line

**Validates: Requirements 7.4, 7.7**

- **Statement:** For any valid display line index, `doc_from_display(display_line)` SHALL never return a document line that is currently excluded.
- **Strategy:** Generate:
  - Document line count: [1, 2000]
  - Exclusion pattern: random subset of lines excluded (at least one visible line remains)
  - Query: random display line indices in [0, lines_displayed)
- **Invariant:** `∀ d in 0..lines_displayed(): !engine.is_excluded(display_mapping.doc_from_display(d).doc_line)`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and State Model", "tasks": ["2", "3", "13"], "dependsOn": [0] },
    { "id": 2, "label": "Text Matching", "tasks": ["4"], "dependsOn": [1] },
    { "id": 3, "label": "Primary Commands", "tasks": ["5", "6", "7"], "dependsOn": [2] },
    { "id": 4, "label": "Line Commands", "tasks": ["8"], "dependsOn": [1] },
    { "id": 5, "label": "Display Model", "tasks": ["9", "10", "12"], "dependsOn": [3, 4] },
    { "id": 6, "label": "Framework Integration", "tasks": ["11", "14"], "dependsOn": [5] },
    { "id": 7, "label": "Validation and PBT", "tasks": ["15", "16"], "dependsOn": [6] }
  ]
}
```

---

## Notes

- This is a Wave 5 (Command Engine) crate depending on `ff-display-line-mapping` (Wave 4) for visibility storage via `set_visible`, `get_visible`, `hidden_lines`, and `show_all`
- The exclude-show-filter does NOT maintain its own per-line visibility state — it delegates entirely to the display-line-mapping layer
- Exclusion operations are explicitly non-undoable (transient session state only)
- The `X` alias for EXCLUDE and `INCLUDE` alias for SHOW share identical argument parsing and dispatch logic
- The EXCLUDE ALL → SHOW 'text' workflow is the primary ISPF-style filtering pattern and must be optimized
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Text matching for EXCLUDE/SHOW reuses concepts from `ff-find-and-replace` but is implemented independently (no crate dependency) to avoid circular dependencies
- Placeholder rendering is the viewport's responsibility — this crate provides only the data model (block ranges and placeholder text)
- The line-command integration (X/Xn/XX) assumes the line-command parser from `ff-line-commands` dispatches to this crate's `exclude_single_line`, `exclude_n_lines`, and `exclude_block` methods
- All error messages follow the `[exclude-filter] operation: description` format per cross-cutting Requirement 8

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Exclusion State Model | AC 1.1–1.8 | Tasks 2, 3 |
| Req 2: EXCLUDE / X Primary Command | AC 2.1–2.10 | Tasks 4, 5 |
| Req 3: SHOW / INCLUDE Primary Command | AC 3.1–3.9 | Tasks 4, 6 |
| Req 4: RESET Command (Exclusion Aspects) | AC 4.1–4.7 | Task 7 |
| Req 5: X / Xn / XX Line Commands | AC 5.1–5.7 | Task 8 |
| Req 6: Placeholder Display Model | AC 6.1–6.8 | Task 9 |
| Req 7: Display-Line Integration | AC 7.1–7.7 | Tasks 3, 12 |
| Req 8: Scope Integration with Find and Change | AC 8.1–8.7 | Task 10 |
| Req 9: Command Framework Integration | AC 9.1–9.8 | Tasks 11, 13 |
| Req 10: Performance and Scalability | AC 10.1–10.6 | Task 14 |
