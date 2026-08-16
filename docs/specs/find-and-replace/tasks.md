# Implementation Plan: Find and Replace (`ff-find-and-replace`)

## Overview

This plan covers the complete implementation of the `ff-find-and-replace` crate — the search and replacement engine for FileForgeWorkbench. The crate provides ISPF-style FIND/RFIND/CHANGE/RCHANGE commands with literal, regex, and hex byte search modes, Unicode case folding, whole-word matching, incremental search, highlight-all-matches, and command framework integration.

This is a **Wave 5 (Command Engine)** sub-project that depends on Wave 4 (`ff-document-model`) for buffer access via the `CharacterIndexer` trait, `ff-command` for command registration, and `ff-undo-redo` for transaction wrapping of CHANGE operations.

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-find-and-replace/Cargo.toml` with dependencies (thiserror, regex, memchr, unicode-casefold, proptest dev-dep) and deps on `ff-document-model`, `ff-command`, `ff-logging`
  - [ ] 1.2 Create `crates/ff-find-and-replace/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `find_engine.rs`, `find_request.rs`, `find_result.rs`, `find_state.rs`, `search_mode.rs`, `direction.rs`, `scope.rs`, `column_range.rs`, `case_folder.rs`, `regex_engine.rs`, `substitution.rs`, `character_indexer.rs`, `incremental.rs`, `highlight.rs`, `commands.rs`, `error.rs`, `types.rs`
  - [ ] 1.4 Add `ff-find-and-replace` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. Core types and enums
  - [ ] 2.1 Define `SearchMode` enum (Literal, Regex, HexBytes) with Display impl
  - [ ] 2.2 Define `SearchDirection` enum (Forward, Backward, First, Last) with conversion from NEXT/PREV/FIRST/LAST command tokens
  - [ ] 2.3 Define `SearchScope` enum (All, Visible, Excluded, Tagged, NonTagged) with filter predicate method
  - [ ] 2.4 Define `ColumnRange { start: u64, end: u64 }` struct with intersection logic for Bounds overlap
  - [ ] 2.5 Define `FindResult { byte_range: Range<u64>, line: LineNumber, captures: Vec<CaptureGroup> }` struct
  - [ ] 2.6 Define `CaptureGroup { index: u8, byte_range: Range<u64> }` struct for regex groups 0–9
  - [ ] 2.7 Define `FindRequest` value type capturing search term, mode, direction, scope, case sensitivity, word matching, and column range
  - [ ] 2.8 Write unit tests for enum conversions, ColumnRange intersection, and FindRequest construction
  - Covers: Requirement 1 (AC 1.1–1.5), Requirement 2 (AC 2.1–2.7), Requirement 3 (AC 3.4–3.5)

- [ ] 3. CharacterIndexer trait definition
  - [ ] 3.1 Define `CharacterIndexer` trait with `char_at(position: u64) -> u8` method
  - [ ] 3.2 Add `slice(start: u64, end: u64) -> Option<&[u8]>` method with fallback semantics
  - [ ] 3.3 Add `move_position_outside_char(position: u64, direction: Direction) -> u64` method
  - [ ] 3.4 Add `line_range(line: LineNumber) -> (u64, u64)` method for line-scoped searches
  - [ ] 3.5 Add `length() -> u64` method for document bounds checking
  - [ ] 3.6 Implement a `SliceIndexer` adapter over `&[u8]` for testing purposes
  - [ ] 3.7 Write unit tests for SliceIndexer verifying trait contract
  - Covers: Requirement 18 (AC 18.1–18.6)

- [ ] 4. CaseFolder implementation
  - [ ] 4.1 Implement `CaseFolder` struct with Unicode Full Case Folding (status C + F mappings from CaseFolding.txt)
  - [ ] 4.2 Implement `fold(&self, text: &[u8]) -> Vec<u8>` producing case-folded UTF-8 output
  - [ ] 4.3 Implement one-to-many case mappings (e.g., ß → ss) correctly expanding output length
  - [ ] 4.4 Implement multi-byte UTF-8 handling — never split code points across fold boundaries
  - [ ] 4.5 Implement stateless, `Send + Sync` design for concurrent use
  - [ ] 4.6 Implement configurable locale hint for Turkish dotted-I rules with locale-independent default
  - [ ] 4.7 Implement `fold_char(&self, ch: char) -> SmallVec<[char; 3]>` for per-character folding in regex engine
  - [ ] 4.8 Write unit tests for ASCII folding, German ß, Turkish İ/ı, multi-byte sequences, and thread safety
  - Covers: Requirement 10 (AC 10.1–10.8)

- [ ] 5. Literal search algorithm
  - [ ] 5.1 Implement case-sensitive literal search using memchr + memcmp for fast byte scanning
  - [ ] 5.2 Implement forward search from a given byte position returning first match
  - [ ] 5.3 Implement backward search from a given byte position returning nearest preceding match
  - [ ] 5.4 Implement case-insensitive literal search with pre-folded search term and lazy-folded document segments
  - [ ] 5.5 Implement FIRST direction (search from document start)
  - [ ] 5.6 Implement LAST direction (search backward from document end)
  - [ ] 5.7 Implement ALL mode counting total matches across scope
  - [ ] 5.8 Implement column-bounded search — extract bounded slice once per line
  - [ ] 5.9 Write unit tests for all directions, case sensitivity modes, and column bounds
  - Covers: Requirement 1 (AC 1.1–1.10), Requirement 19 (AC 19.3, 19.7)

- [ ] 6. Hex byte search
  - [ ] 6.1 Implement hex string parser: convert pairs of hex digits to raw byte sequence
  - [ ] 6.2 Implement validation: reject odd-length hex strings with "Invalid hex pattern: odd number of digits"
  - [ ] 6.3 Implement validation: reject non-hex characters with "Invalid hex pattern: non-hex character"
  - [ ] 6.4 Implement hex byte search using the same direction/scope modifiers as literal search
  - [ ] 6.5 Ensure hex search does NOT apply Unicode case folding — operates on raw bytes
  - [ ] 6.6 Write unit tests for valid hex parsing, invalid hex errors, and hex search with all directions
  - Covers: Requirement 3 (AC 3.1–3.7)

- [ ] 7. Scope and visibility filtering
  - [ ] 7.1 Implement `LineFilter` trait with `is_eligible(line: LineNumber) -> bool` method
  - [ ] 7.2 Implement TAGGED filter checking line `tagged` flag
  - [ ] 7.3 Implement EXCLUDED filter checking line `excluded` flag
  - [ ] 7.4 Implement VISIBLE filter checking line `visible` flag
  - [ ] 7.5 Implement NONTAGGED filter checking line `tagged == false`
  - [ ] 7.6 Implement conjunctive composition — multiple scope modifiers combine with AND logic
  - [ ] 7.7 Implement Bounds integration: when `bounds_affect_find` is true, restrict search to active Bounds columns
  - [ ] 7.8 Implement explicit ColumnRange override that takes precedence over Bounds for single operation
  - [ ] 7.9 Write unit tests for each filter, combinations, and Bounds/ColumnRange intersection
  - Covers: Requirement 2 (AC 2.1–2.8), Requirement 7 (AC 7.5–7.6)

- [ ] 8. Whole-word and word-start matching
  - [ ] 8.1 Implement word-boundary detection using character classification table (word vs non-word transitions)
  - [ ] 8.2 Implement WORD modifier: verify transitions at both start and end of match
  - [ ] 8.3 Implement WORDSTART modifier: verify transition at start only
  - [ ] 8.4 Implement multi-byte UTF-8 character classification — classify by full code point, not individual bytes
  - [ ] 8.5 Implement correct interaction with case folding: fold first, verify boundaries on original positions
  - [ ] 8.6 Write unit tests for word boundaries with ASCII, multi-byte characters, and combined case+word mode
  - Covers: Requirement 11 (AC 11.1–11.5)

- [ ] 9. RegexEngine — NFA compilation
  - [ ] 9.1 Implement regex pattern parser supporting: `.`, `^`, `$`, `*`, `+`, `?`, lazy variants `*?`, `+?`, `??`
  - [ ] 9.2 Implement character class parsing: `[set]`, `[^set]`, ranges `[a-z]`, dash/bracket at boundaries
  - [ ] 9.3 Implement escape sequences: `\d`, `\D`, `\s`, `\S`, `\w`, `\W`, `\b`, `\<`, `\>`
  - [ ] 9.4 Implement hex escape `\xHH` and C-style escapes `\a`, `\f`, `\n`, `\r`, `\t`, `\v`
  - [ ] 9.5 Implement group capture with parentheses `(...)` supporting groups 0–9
  - [ ] 9.6 Implement backreferences `\1`–`\9` within pattern
  - [ ] 9.7 Implement NFA compilation from parsed AST with size limit check ("Pattern too long")
  - [ ] 9.8 Implement error reporting: "Unmatched (", "Unmatched )", "Empty closure", "Illegal closure", "Undetermined reference", "Cyclical reference"
  - [ ] 9.9 Implement empty-pattern reuse (reuse last compiled NFA) and "No previous regular expression" error
  - [ ] 9.10 Write unit tests for each metacharacter, error case, and compiled NFA structure
  - Covers: Requirement 4 (AC 4.1–4.11), Requirement 12 (AC 12.1–12.9)

- [ ] 10. RegexEngine — NFA execution
  - [ ] 10.1 Implement NFA execution against CharacterIndexer within a byte range [start, end)
  - [ ] 10.2 Implement fast-path: when NFA starts with literal character, use memchr to locate first candidate
  - [ ] 10.3 Implement greedy matching: consume maximum then backtrack
  - [ ] 10.4 Implement lazy matching: attempt shortest first then extend
  - [ ] 10.5 Implement match-attempt limit per position (default 10,000 steps) to prevent catastrophic backtracking
  - [ ] 10.6 Implement step-limit exceeded handling: skip position, log warning, continue search
  - [ ] 10.7 Implement UTF-8 boundary validation: reject matches starting/ending inside multi-byte characters
  - [ ] 10.8 Implement case-insensitive regex via CaseFolder integration during NFA character comparison
  - [ ] 10.9 Write unit tests for greedy/lazy matching, backtracking limits, UTF-8 boundary rejection
  - Covers: Requirement 4 (AC 4.12–4.13), Requirement 12 (AC 12.10–12.13), Requirement 19 (AC 19.4–19.5)

- [ ] 11. SubstitutionTemplate and replacement logic
  - [ ] 11.1 Implement `SubstitutionTemplate` parser recognizing `\0`–`\9` and `$0`–`$9` group references
  - [ ] 11.2 Implement `substitute(template, captures) -> String` expanding group references against CaptureGroups
  - [ ] 11.3 Implement unmatched group substitution: replace with empty string
  - [ ] 11.4 Implement invalid escape sequence detection in replacement with descriptive error
  - [ ] 11.5 Write unit tests for group expansion, unmatched groups, mixed `\N`/`$N` syntax, and error cases
  - Covers: Requirement 8 (AC 8.1–8.8)

- [ ] 12. FindEngine core — unified search dispatch
  - [ ] 12.1 Implement `FindEngine` struct holding CaseFolder, last compiled regex, and configuration
  - [ ] 12.2 Implement `find(&self, request: &FindRequest, indexer: &dyn CharacterIndexer, filter: &dyn LineFilter) -> Result<FindResult, FindError>` dispatching to literal/regex/hex by mode
  - [ ] 12.3 Implement `find_all(&self, request: &FindRequest, indexer: &dyn CharacterIndexer, filter: &dyn LineFilter) -> Result<Vec<FindResult>, FindError>` for ALL mode
  - [ ] 12.4 Implement empty-search-term handling: reuse previous term or error if none exists
  - [ ] 12.5 Implement empty-document short-circuit returning "not found" immediately
  - [ ] 12.6 Implement null-byte tolerance — treat 0x00 as regular byte value
  - [ ] 12.7 Implement incomplete UTF-8 in literal search term — search raw bytes as-is
  - [ ] 12.8 Write unit tests for dispatch, empty term, empty document, and null byte handling
  - Covers: Requirement 1 (AC 1.6–1.10), Requirement 20 (AC 20.1–20.3, 20.7–20.8)

- [ ] 13. CHANGE (replacement) engine
  - [ ] 13.1 Implement `change(&self, request: &FindRequest, replacement: &str, indexer: &mut dyn CharacterIndexer) -> Result<ChangeResult, FindError>` for single replacement
  - [ ] 13.2 Implement `change_all(...)` iterating non-overlapping matches with position adjustment for length deltas
  - [ ] 13.3 Implement zero-length match advancement (advance by at least one character to prevent infinite loops)
  - [ ] 13.4 Implement regex replacement with SubstitutionTemplate expansion per match
  - [ ] 13.5 Implement read-only document check: return "Document is read-only" error before searching
  - [ ] 13.6 Implement CHANGE ALL returning total substitution count
  - [ ] 13.7 Implement "not found" result with "'old' NOT FOUND" message when zero replacements made
  - [ ] 13.8 Write unit tests for single/all replacement, length delta handling, zero-length matches, and read-only guard
  - Covers: Requirement 6 (AC 6.1–6.8), Requirement 7 (AC 7.1–7.8), Requirement 8 (AC 8.5–8.7)

- [ ] 14. FindState and session persistence
  - [ ] 14.1 Implement `FindState` struct storing last FindRequest, last replacement, and per-document state
  - [ ] 14.2 Implement search history ring buffer (configurable size, default 20 entries)
  - [ ] 14.3 Implement replacement history ring buffer (configurable size, default 20 entries)
  - [ ] 14.4 Implement RFIND logic: repeat last FIND advancing in same direction; FIRST→NEXT, LAST→PREV conversion
  - [ ] 14.5 Implement RCHANGE logic: repeat last CHANGE on next occurrence; FIRST→NEXT, LAST→PREV conversion
  - [ ] 14.6 Implement "No previous FIND to repeat" and "No previous CHANGE to repeat" errors
  - [ ] 14.7 Implement RFIND wrap detection — report "NOT FOUND" without wrapping around document
  - [ ] 14.8 Implement RESET clearing highlight/incremental state while retaining RFIND/RCHANGE parameters
  - [ ] 14.9 Implement RESET ALL clearing last-search parameters but retaining history list
  - [ ] 14.10 Implement per-document FindState isolation
  - [ ] 14.11 Implement serialisation for session persistence across restarts
  - [ ] 14.12 Write unit tests for RFIND/RCHANGE cycling, history overflow, RESET variants, and serialisation round-trip
  - Covers: Requirement 5 (AC 5.1–5.6), Requirement 9 (AC 9.1–9.6), Requirement 13 (AC 13.1–13.7)

- [ ] 15. Incremental search
  - [ ] 15.1 Implement `IncrementalSearch` struct managing partial-text state, start position, and cancellation token
  - [ ] 15.2 Implement forward search from cursor with partial text within configurable time budget (default 50ms)
  - [ ] 15.3 Implement cancellation on text change: abort in-progress search, restart with updated text
  - [ ] 15.4 Implement backspace handling: re-search from original start position, not current match
  - [ ] 15.5 Implement debouncing: only search the latest keystroke state when input faster than search
  - [ ] 15.6 Implement empty-field handling: clear highlights and restore pre-search viewport
  - [ ] 15.7 Implement mode respect: honour current case-sensitivity and literal/regex mode settings
  - [ ] 15.8 Write unit tests for incremental search lifecycle, cancellation, debounce, and mode interaction
  - Covers: Requirement 14 (AC 14.1–14.8)

- [ ] 16. Highlight-all-matches mode
  - [ ] 16.1 Implement viewport-scoped match computation: find all matches in visible byte range
  - [ ] 16.2 Implement async/time-budgeted execution to avoid blocking rendering
  - [ ] 16.3 Implement viewport scroll update: recompute match set on scroll events
  - [ ] 16.4 Implement search-term change: clear previous highlights, recompute for new term
  - [ ] 16.5 Implement panel-close cleanup: clear all highlight-all decorations
  - [ ] 16.6 Implement configurable match threshold (default 1000) with overflow reporting
  - [ ] 16.7 Implement distinct decoration style separation from current-match highlight
  - [ ] 16.8 Write unit tests for viewport match computation, threshold enforcement, and cleanup
  - Covers: Requirement 15 (AC 15.1–15.8)

- [ ] 17. EXCLUDE/SHOW/RESET integration
  - [ ] 17.1 Implement `find_for_exclude(...)` method: delegates to FindEngine literal/regex logic for line matching
  - [ ] 17.2 Implement `find_for_show(...)` method: identifies excluded lines containing the search term
  - [ ] 17.3 Ensure EXCLUDE/SHOW respect current case-sensitivity settings
  - [ ] 17.4 Ensure EXCLUDE/SHOW do NOT update FindState (no RFIND/RCHANGE side effects)
  - [ ] 17.5 Implement RESET integration: clear highlight-all and incremental state
  - [ ] 17.6 Write unit tests for exclude/show delegation, state isolation, and RESET clearing
  - Covers: Requirement 16 (AC 16.1–16.6)

- [ ] 18. Command framework integration
  - [ ] 18.1 Register commands: `find`, `rfind`, `change`, `rchange`, `find_next`, `find_prev`, `find_all`, `replace_all` with metadata (display name, description, default keybinding, category "Search")
  - [ ] 18.2 Implement undo transaction wrapping: all CHANGE/RCHANGE operations create a single undo transaction
  - [ ] 18.3 Implement CHANGE ALL batch grouping: entire multi-replacement batch as one undo transaction
  - [ ] 18.4 Implement FIND as read-only (no undo records created)
  - [ ] 18.5 Implement RCHANGE as its own separate undo transaction per invocation
  - [ ] 18.6 Implement event emission: find_started, match_found, find_completed, replace_completed
  - [ ] 18.7 Implement Lua scripting bridge compatibility — same argument semantics as command-line input
  - [ ] 18.8 Write unit tests for command registration, undo grouping, and event emission
  - Covers: Requirement 17 (AC 17.1–17.7)

- [ ] 19. Performance and cancellation
  - [ ] 19.1 Implement cancellation token for in-progress FIND ALL and CHANGE ALL operations
  - [ ] 19.2 Implement periodic progress reporting (every N matches or M milliseconds, configurable)
  - [ ] 19.3 Implement pre-allocated/amortised result collection for FIND ALL (avoid per-line allocation)
  - [ ] 19.4 Implement bounded-slice-per-line optimisation for column-restricted searches
  - [ ] 19.5 Write unit tests for cancellation mid-search, progress callback invocation, and allocation efficiency
  - Covers: Requirement 19 (AC 19.1–19.2, 19.6–19.7)

- [ ] 20. Error handling and edge cases
  - [ ] 20.1 Define `FindError` enum: NotFound, NoSearchTerm, NoPreviousFind, NoPreviousChange, InvalidHexPattern, InvalidRegex, DocumentReadOnly, InvalidEscape, PatternTooLong
  - [ ] 20.2 Implement error message formatting per `[find] operation: description` standard
  - [ ] 20.3 Implement empty document short-circuit
  - [ ] 20.4 Implement read-only document guard for CHANGE commands
  - [ ] 20.5 Implement zero-replacements message matching single-match "NOT FOUND" format
  - [ ] 20.6 Write unit tests for all error variants and edge case responses
  - Covers: Requirement 20 (AC 20.1–20.8)

- [ ] 21. Property-based tests
  - [ ] 21.1 Write PBT: literal search result correctness
  - [ ] 21.2 Write PBT: case folding roundtrip and idempotency
  - [ ] 21.3 Write PBT: regex match validity (matches within bounds, at character boundaries)
  - [ ] 21.4 Write PBT: CHANGE ALL replacement count consistency
  - [ ] 21.5 Write PBT: RFIND/RCHANGE state preservation
  - [ ] 21.6 Write PBT: scope filter conjunction correctness
  - [ ] 21.7 Write PBT: hex byte search equivalence to raw byte matching
  - Covers: Requirements 1–6, 8, 10, 11 (see Property-Based Test Definitions below)

- [ ] 22. Integration tests
  - [ ] 22.1 Write integration test: full FIND → RFIND → CHANGE → RCHANGE lifecycle
  - [ ] 22.2 Write integration test: regex search with group capture and substitution
  - [ ] 22.3 Write integration test: incremental search with debounce and cancellation
  - [ ] 22.4 Write integration test: highlight-all with viewport scroll updates
  - [ ] 22.5 Write integration test: large document (100K+ lines) FIND ALL with cancellation and progress
  - [ ] 22.6 Write integration test: EXCLUDE/SHOW delegation through FindEngine
  - Covers: End-to-end validation across Requirements 1–20

---

## Property-Based Test Definitions

### Property 1: Literal Search Result Correctness

**Validates: Requirements 1.1, 1.2, 1.9, 1.10**

- **Statement:** For any document content and any literal search term, every FindResult returned by a forward search SHALL satisfy: (a) the byte range in the document contains exactly the search term bytes (or case-folded equivalent), (b) the result line number matches `line_from_position(result.start)`, and (c) no match exists between the start position and the first result position.
- **Strategy:** Generate:
  - Document content: arbitrary UTF-8 strings (0–10000 bytes) with embedded known patterns
  - Search terms: substrings of the content (1–50 bytes) and random non-occurring strings
  - Case sensitivity: randomly chosen
- **Invariant:** `indexer.slice(result.start, result.end) == search_term` (after folding if case-insensitive); `line_from_position(result.start) == result.line`

### Property 2: Case Folding Roundtrip and Idempotency

**Validates: Requirements 10.1, 10.3, 10.4**

- **Statement:** For any valid UTF-8 string, folding the string twice SHALL produce the same result as folding once (idempotency), and the folded output SHALL always be valid UTF-8 that never splits a code point boundary.
- **Strategy:** Generate:
  - Input strings: arbitrary Unicode text (0–5000 chars) including Latin, Greek, Cyrillic, Turkish, German
  - Locale hints: None, Turkish
- **Invariant:** `fold(fold(text)) == fold(text)` AND `std::str::from_utf8(folded).is_ok()`

### Property 3: Regex Match Validity

**Validates: Requirements 4.9, 4.13, 12.10**

- **Statement:** For any regex match result, the reported byte range SHALL: (a) fall within [0, document.length()), (b) start and end at valid UTF-8 character boundaries, and (c) all captured groups SHALL be sub-ranges of the full match range (group 0).
- **Strategy:** Generate:
  - Document content: arbitrary UTF-8 strings (0–5000 bytes) with mixed multi-byte characters
  - Regex patterns: randomly generated valid patterns from a grammar (concatenation, alternation, quantifiers, character classes)
- **Invariant:** `0 <= result.start <= result.end <= doc.length()` AND character-boundary checks pass AND `group[i].range ⊆ group[0].range` for all i

### Property 4: CHANGE ALL Replacement Count Consistency

**Validates: Requirements 6.2, 6.8, 8.5**

- **Statement:** For any CHANGE ALL operation, the reported replacement count SHALL equal the number of non-overlapping matches found by a FIND ALL with the same parameters on the original (pre-change) content, and the final document content SHALL contain zero remaining matches of the original search term (unless the replacement text reintroduces the pattern).
- **Strategy:** Generate:
  - Document content: arbitrary text (0–5000 bytes) with known repeated patterns
  - Search term: literal strings (1–20 bytes) guaranteed to occur in content
  - Replacement text: arbitrary strings (0–30 bytes) not containing the search term
  - Scope/direction modifiers: random valid combinations
- **Invariant:** `change_all_count == find_all_count` AND post-change `find_all_count == 0` (when replacement doesn't contain search term)

### Property 5: RFIND/RCHANGE State Preservation

**Validates: Requirements 5.1, 5.3, 9.1, 9.3**

- **Statement:** After executing a FIND or CHANGE, the FindState SHALL contain all original arguments such that RFIND/RCHANGE re-executes with identical parameters (term, mode, scope, case sensitivity). After RESET, FindState SHALL retain parameters; after RESET ALL, RFIND SHALL fail with "No previous FIND".
- **Strategy:** Generate:
  - Random FindRequest/ChangeRequest values (mode, direction, scope, case flag, column range)
  - Operation sequences: FIND → RFIND, CHANGE → RCHANGE, FIND → RESET → RFIND, FIND → RESET ALL → RFIND
- **Invariant:** `find_state.last_request() == original_request` after FIND; `find_state.last_request().is_none()` after RESET ALL

### Property 6: Scope Filter Conjunction Correctness

**Validates: Requirements 2.1–2.4, 2.8**

- **Statement:** When multiple scope modifiers are active, the set of lines searched SHALL be exactly the intersection of all individual filter sets — no line is searched that fails any single filter, and no eligible line is skipped.
- **Strategy:** Generate:
  - Line count: [1, 1000]
  - Per-line flags: random (tagged: bool, excluded: bool, visible: bool)
  - Active filters: random subset of {Tagged, Excluded, Visible, NonTagged}
  - Search term: present on some random subset of lines
- **Invariant:** `searched_lines == lines.filter(|l| all_filters_pass(l))` — match results only appear on lines passing all active filters

### Property 7: Hex Byte Search Equivalence

**Validates: Requirements 3.1, 3.4, 3.6, 3.7**

- **Statement:** For any valid hex pattern, searching in HexBytes mode SHALL produce identical match positions as searching in Literal mode for the raw byte sequence represented by that hex string, regardless of document encoding.
- **Strategy:** Generate:
  - Document content: arbitrary bytes (0–5000 bytes) including non-UTF-8 sequences
  - Hex patterns: random even-length hex digit strings representing 1–50 bytes
  - Direction: random (Forward, Backward, First, Last)
- **Invariant:** `find(hex_mode, hex_string).positions == find(literal_mode, decoded_bytes).positions`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Trait", "tasks": ["2", "3", "20"], "dependsOn": [0] },
    { "id": 2, "label": "Case Folding", "tasks": ["4"], "dependsOn": [1] },
    { "id": 3, "label": "Search Algorithms", "tasks": ["5", "6", "7", "8"], "dependsOn": [2] },
    { "id": 4, "label": "Regex Engine", "tasks": ["9", "10"], "dependsOn": [2] },
    { "id": 5, "label": "Replacement Engine", "tasks": ["11", "13"], "dependsOn": [3, 4] },
    { "id": 6, "label": "Find Engine Assembly", "tasks": ["12"], "dependsOn": [3, 4] },
    { "id": 7, "label": "State and Session", "tasks": ["14"], "dependsOn": [5, 6] },
    { "id": 8, "label": "Interactive Features", "tasks": ["15", "16"], "dependsOn": [6] },
    { "id": 9, "label": "Integration Layer", "tasks": ["17", "18", "19"], "dependsOn": [7, 8] },
    { "id": 10, "label": "Validation and PBT", "tasks": ["21", "22"], "dependsOn": [9] }
  ]
}
```

---

## Notes

- This is a Wave 5 (Command Engine) crate depending on `ff-document-model` (Wave 4) for buffer access via `CharacterIndexer`
- The undo/redo integration wraps CHANGE operations in transactions provided by `ff-undo-redo-transactions`
- The UI rendering of matches (highlighting, find panel) is handled by `ff-text-decorations` (Wave 6) — this crate only emits match positions
- The `CharacterIndexer` trait is defined in this crate and implemented by `ff-document-model` to decouple search from buffer internals
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The regex engine is custom NFA-based (not using the `regex` crate directly) to support ISPF/Scintilla-compatible syntax including `\<`, `\>`, and backreferences
- All async operations (incremental search, highlight-all) use cancellation tokens compatible with the Tokio runtime in `ff-core`
- The CaseFolder uses Unicode 15.0 CaseFolding.txt data; the fold table can be generated at build time or embedded as a static lookup

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: FIND — Literal Search | AC 1.1–1.10 | Tasks 5, 12, 2 |
| Req 2: FIND — Scope and Column Modifiers | AC 2.1–2.8 | Tasks 7, 12 |
| Req 3: FIND — Hex Byte Search | AC 3.1–3.7 | Task 6 |
| Req 4: FIND — Regular Expression Search | AC 4.1–4.13 | Tasks 9, 10 |
| Req 5: RFIND — Repeat Previous Find | AC 5.1–5.6 | Task 14 |
| Req 6: CHANGE — Literal Replacement | AC 6.1–6.8 | Task 13 |
| Req 7: CHANGE — Scope and Column Modifiers | AC 7.1–7.8 | Tasks 7, 13, 18 |
| Req 8: CHANGE — Regex Replacement | AC 8.1–8.8 | Tasks 11, 13 |
| Req 9: RCHANGE — Repeat Previous Change | AC 9.1–9.6 | Tasks 14, 18 |
| Req 10: Unicode Case Folding | AC 10.1–10.8 | Task 4 |
| Req 11: Whole Word and Word Start Matching | AC 11.1–11.5 | Task 8 |
| Req 12: Regex Engine — NFA Compilation/Execution | AC 12.1–12.13 | Tasks 9, 10 |
| Req 13: Find State and Session Persistence | AC 13.1–13.7 | Task 14 |
| Req 14: Incremental Search | AC 14.1–14.8 | Task 15 |
| Req 15: Highlight All Matches Mode | AC 15.1–15.8 | Task 16 |
| Req 16: Find Integration with Exclude/Show/Reset | AC 16.1–16.6 | Task 17 |
| Req 17: Command Framework Integration | AC 17.1–17.7 | Task 18 |
| Req 18: Character Indexer Abstraction | AC 18.1–18.6 | Task 3 |
| Req 19: Performance and Large-File Considerations | AC 19.1–19.7 | Tasks 5, 10, 19 |
| Req 20: Error Handling and Edge Cases | AC 20.1–20.8 | Tasks 12, 13, 20 |
