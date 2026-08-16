# Requirements Document

## Introduction

This feature specifies the **Find and Replace Engine** for FileForgeWorkbench — the `ff-find-and-replace` crate. The find engine provides ISPF-style FIND/RFIND/CHANGE/RCHANGE commands with literal, regular expression, and hexadecimal search modes, combined with Scintilla-derived search capabilities including Unicode case folding, whole-word matching, and regex group capture with substitution.

The engine is **GUI-independent** — it contains no rendering or UI framework dependencies. The search panel, highlight rendering, and keyboard shortcuts are concerns of separate UI and text-decoration specs. This crate provides the search algorithm, match state, replacement logic, and command integration.

This specification merges requirements from three primary sources:

- **FileForgeEditor core-command-semantics** (Requirements 3–9): FIND with literal/REGEX/hex modes and NEXT/PREV/FIRST/LAST/ALL direction modifiers, TAGGED/EXCLUDED/VISIBLE/NONTAGGED scope filtering, bounds-aware column restriction, RFIND repeat, CHANGE with ALL/scope/column modifiers, RCHANGE repeat, EXCLUDE/SHOW/RESET find-state integration
- **Scintilla Document and RESearch** (Requirement 17, Requirements 12–17): FindText forward/backward search, CaseFolder for case-insensitive UTF-8, WholeWord/WordStart matching, NFA regex with group capture (\1–\9), character classes, lazy/greedy quantifiers, SubstituteByPosition for replacement text
- **New workbench concepts** [WB]: Incremental search (search-as-you-type), Unicode case folding across all scripts, highlight-all-matches mode for live feedback

**Source references:**
- **[FFE-CMD-3]** = FileForgeEditor core-command-semantics Requirement 3: FIND Command
- **[FFE-CMD-4]** = FileForgeEditor core-command-semantics Requirement 4: RFIND Command
- **[FFE-CMD-5]** = FileForgeEditor core-command-semantics Requirement 5: CHANGE Command
- **[FFE-CMD-6]** = FileForgeEditor core-command-semantics Requirement 6: RCHANGE Command
- **[FFE-CMD-7]** = FileForgeEditor core-command-semantics Requirement 7: EXCLUDE Command (find integration)
- **[FFE-CMD-8]** = FileForgeEditor core-command-semantics Requirement 8: SHOW/INCLUDE Command (find integration)
- **[FFE-CMD-9]** = FileForgeEditor core-command-semantics Requirement 9: RESET Command (find state impact)
- **[SCI-DOC-17]** = Scintilla document-cellbuffer Requirement 17: Document Search (FindText)
- **[SCI-RES]** = Scintilla ScintillaBase/RESearch Requirements 12–17: Regex engine
- **[WB]** = Workbench Platform Architecture Brief

## Cross-References

- **`command-semantics`** — The find/replace commands are dispatched through the command framework; this spec defines the engine behaviour, not the dispatch routing
- **`document-model`** — Search operates over the document's text buffer via character indexing and line queries
- **`undo-redo-transactions`** — CHANGE operations are wrapped in undo transactions
- **`text-decorations`** — Match highlighting and incremental search feedback use the decoration/indicator system
- **`encoding-and-characters`** — Unicode case folding and multi-byte character boundary handling depend on encoding awareness
- **`exclude-show-filter`** — FIND respects line visibility state; EXCLUDE uses FIND's search engine internally
- **`display-line-mapping`** — Search results reference document lines which map through the display-line system

---

## Glossary

- **FindEngine**: The core struct that executes text searches over a document buffer. GUI-independent; provides match results as position ranges. [SCI-DOC-17]
- **FindRequest**: A value type capturing all parameters for a single find operation: search term, mode, direction, scope, column range. [FFE-CMD-3]
- **FindResult**: A value type representing a successful match: document byte range, line number, and captured groups (if regex). [SCI-DOC-17]
- **SearchMode**: An enum specifying the interpretation of the search term: Literal, Regex, or HexBytes. [FFE-CMD-3, SCI-RES]
- **SearchDirection**: An enum specifying traversal direction: Forward (NEXT), Backward (PREV), First, Last. [FFE-CMD-3]
- **SearchScope**: The set of lines eligible for searching, determined by visibility flags and explicit modifiers (ALL, VISIBLE, EXCLUDED, TAGGED, NONTAGGED). [FFE-CMD-3]
- **ColumnRange**: An optional pair (col_start, col_end) restricting search to a horizontal slice of each line. [FFE-CMD-5]
- **Bounds**: The active left/right column boundaries set by the BOUNDS command. When active and `bounds_affect_find` is enabled, restricts FIND to the bounded columns. [FFE-CMD-3]
- **CaseFolder**: A component that implements Unicode case folding for case-insensitive comparison across all Unicode scripts. Replaces Scintilla's CaseFolder. [SCI-DOC-17]
- **RegexEngine**: The NFA-based regular expression engine supporting POSIX-like syntax with extensions, group capture, backreferences, and lazy/greedy quantifiers. Replaces Scintilla's RESearch. [SCI-RES]
- **CaptureGroup**: A numbered match group (0 = entire match, 1–9 = sub-groups) captured by parenthesised regex sub-expressions. [SCI-RES]
- **FindState**: The session-persisted state of the most recent FIND/CHANGE operation, enabling RFIND/RCHANGE repetition. [FFE-CMD-4, FFE-CMD-6]
- **IncrementalSearch**: A search mode where matches update live as the user types characters into the find field, before explicit command submission. [WB]
- **HighlightAllMatches**: A mode where all visible matches of the current search term are decorated simultaneously while the find panel is open. [WB]
- **SubstitutionTemplate**: A replacement string that may contain group references (`\1`–`\9`, `\0`, `$1`–`$9`) expanded from the most recent regex match. [SCI-DOC-17, SCI-RES]

---

## Requirements

### Requirement 1: FIND Command — Literal Search

**User Story:** As a developer, I want to search document content using literal text with fine-grained direction and scope modifiers, so that I can quickly locate any text in large files regardless of visibility state or column boundaries.

**Source:** [FFE-CMD-3], [SCI-DOC-17]

#### Acceptance Criteria

1. WHEN `FIND 'text'` is issued, THE FindEngine SHALL search visible lines from the current cursor position forward and return the first match as a FindResult containing the byte range and line number. [FFE-CMD-3]
2. WHEN `FIND 'text' NEXT` is issued, THE FindEngine SHALL find the next match after the current cursor position in the forward direction. [FFE-CMD-3]
3. WHEN `FIND 'text' PREV` is issued, THE FindEngine SHALL find the nearest match before the current cursor position in the backward direction. [FFE-CMD-3, SCI-DOC-17]
4. WHEN `FIND 'text' FIRST` is issued, THE FindEngine SHALL find the first match in the resolved SearchScope starting from the beginning of the document. [FFE-CMD-3]
5. WHEN `FIND 'text' LAST` is issued, THE FindEngine SHALL find the last match in the resolved SearchScope. [FFE-CMD-3]
6. WHEN `FIND 'text' ALL` is issued, THE FindEngine SHALL search all lines regardless of visibility state and return the total count of matches. [FFE-CMD-3]
7. WHEN no match is found, THE FindEngine SHALL return a "not found" result and the command layer SHALL display "'text' NOT FOUND" in the status area without modifying the viewport position. [FFE-CMD-3]
8. WHEN a match is found, THE command layer SHALL scroll the viewport so the matching line is visible and SHALL position the cursor at the match start. [FFE-CMD-3]
9. THE FindEngine SHALL support case-sensitive matching as the default mode for literal search. [SCI-DOC-17]
10. WHEN case-insensitive matching is requested, THE FindEngine SHALL fold both the search term and document text using the CaseFolder before comparison. [SCI-DOC-17]

---

### Requirement 2: FIND Command — Scope and Column Modifiers

**User Story:** As a developer, I want FIND to respect line visibility flags, tag state, and column boundaries, so that I can search precisely within excluded lines, tagged lines, or bounded column ranges.

**Source:** [FFE-CMD-3]

#### Acceptance Criteria

1. WHEN `FIND 'text' TAGGED` is issued, THE FindEngine SHALL restrict the search to lines whose `tagged` flag is true. [FFE-CMD-3]
2. WHEN `FIND 'text' EXCLUDED` is issued, THE FindEngine SHALL search only lines whose `excluded` flag is true. [FFE-CMD-3]
3. WHEN `FIND 'text' VISIBLE` is issued, THE FindEngine SHALL search only lines whose `visible` flag is true. [FFE-CMD-3]
4. WHEN `FIND 'text' NONTAGGED` is issued, THE FindEngine SHALL restrict the search to lines whose `tagged` flag is false. [FFE-CMD-3]
5. WHEN active Bounds are set and the `bounds_affect_find` configuration value is true, THE FindEngine SHALL restrict the search to characters within the active column Bounds on each line. [FFE-CMD-3]
6. WHEN active Bounds are NOT set or `bounds_affect_find` is false, THE FindEngine SHALL search the full content of each eligible line. [FFE-CMD-3]
7. THE FindEngine SHALL accept an optional explicit ColumnRange that overrides Bounds for a single search operation. [FFE-CMD-3]
8. WHEN multiple scope modifiers are combined (e.g., TAGGED + Bounds), THE FindEngine SHALL apply all constraints conjunctively — a line must satisfy all active filters to be searched. [FFE-CMD-3]

---

### Requirement 3: FIND Command — Hex Byte Search

**User Story:** As a developer working with binary data or mainframe files, I want to search for raw byte sequences specified as hexadecimal, so that I can locate non-printable characters and binary patterns.

**Source:** [FFE-CMD-3]

#### Acceptance Criteria

1. WHEN `FIND X'hexdigits'` is issued, THE FindEngine SHALL search for the raw byte sequence represented by the hex string. [FFE-CMD-3]
2. IF the hex digit string contains an odd number of hex digits, THEN THE FindEngine SHALL return an error "Invalid hex pattern: odd number of digits" and SHALL NOT execute the search. [FFE-CMD-3]
3. IF the hex digit string contains non-hex characters (outside 0–9, A–F, a–f), THEN THE FindEngine SHALL return an error "Invalid hex pattern: non-hex character" and SHALL NOT execute the search. [FFE-CMD-3]
4. THE hex byte search SHALL apply the same direction modifiers (NEXT, PREV, FIRST, LAST, ALL) as literal search. [FFE-CMD-3]
5. THE hex byte search SHALL apply the same scope modifiers (TAGGED, EXCLUDED, VISIBLE, NONTAGGED) as literal search. [FFE-CMD-3]
6. THE hex byte search SHALL be case-insensitive with respect to the hex digits themselves (X'4A' and X'4a' are equivalent). [FFE-CMD-3]
7. THE hex byte search SHALL NOT apply Unicode case folding — it operates on raw bytes regardless of encoding. [FFE-CMD-3]

---

### Requirement 4: FIND Command — Regular Expression Search

**User Story:** As a developer, I want to search using regular expression patterns with full group capture, so that I can locate complex textual patterns and use captured groups in subsequent replacement operations.

**Source:** [FFE-CMD-3], [SCI-RES], [SCI-DOC-17]

#### Acceptance Criteria

1. WHEN `FIND REGEX 'pattern'` is issued, THE FindEngine SHALL interpret the search string as a regular expression and apply the same direction and scope modifiers as literal FIND. [FFE-CMD-3]
2. THE RegexEngine SHALL support the following metacharacters: `.` (any char except newline), `^` (beginning of line), `$` (end of line), `*` (zero or more, greedy), `+` (one or more, greedy), `?` (zero or one, greedy). [SCI-RES]
3. THE RegexEngine SHALL support lazy quantifiers: `*?` (zero or more, lazy), `+?` (one or more, lazy), `??` (zero or one, lazy). [SCI-RES]
4. THE RegexEngine SHALL support character classes: `[set]` for inclusion, `[^set]` for negation, ranges `[a-z]`, and literal dash/bracket at set boundaries. [SCI-RES]
5. THE RegexEngine SHALL support escape sequences: `\d` (digits), `\D` (non-digits), `\s` (whitespace), `\S` (non-whitespace), `\w` (word characters), `\W` (non-word characters). [SCI-RES]
6. THE RegexEngine SHALL support word boundary anchors: `\b` (word boundary), `\<` (beginning of word), `\>` (end of word), using the document's character classification table. [SCI-RES]
7. THE RegexEngine SHALL support hex escape `\xHH` for specifying characters by hexadecimal code point. [SCI-RES]
8. THE RegexEngine SHALL support C-style escape sequences: `\a`, `\f`, `\n`, `\r`, `\t`, `\v`. [SCI-RES]
9. THE RegexEngine SHALL support group capture with parentheses `(...)`, with a maximum of 10 groups (group 0 = entire match, groups 1–9 = sub-expressions). [SCI-RES]
10. THE RegexEngine SHALL support backreferences `\1` through `\9` within a pattern, referring to previously captured groups. [SCI-RES]
11. IF a regex pattern is syntactically invalid, THEN THE FindEngine SHALL return a descriptive error message (e.g., "Unmatched (", "Empty closure", "Pattern too long") and SHALL NOT execute the search. [SCI-RES]
12. WHEN case-insensitive regex search is requested, THE RegexEngine SHALL fold characters using the CaseFolder during matching. [SCI-RES, SCI-DOC-17]
13. WHEN a regex match starts or ends inside a multi-byte UTF-8 character, THE FindEngine SHALL reject that match and continue scanning for a valid match at a character boundary. [SCI-RES]

---

### Requirement 5: RFIND Command — Repeat Previous Find

**User Story:** As a developer, I want to repeat the previous FIND with a single command, so that I can quickly scan through multiple occurrences without retyping the search text or modifiers.

**Source:** [FFE-CMD-4]

#### Acceptance Criteria

1. WHEN `RFIND` is issued, THE FindEngine SHALL repeat the most recently executed FIND operation with all its original arguments and modifiers, advancing to the next match in the same direction. [FFE-CMD-4]
2. IF no previous FIND operation exists in the current session, THEN THE FindEngine SHALL return an error "No previous FIND to repeat" and SHALL NOT modify the viewport or document state. [FFE-CMD-4]
3. THE FindEngine SHALL store the most recent FindRequest in the FindState so that RFIND is available across multiple command submissions within the same editing session. [FFE-CMD-4]
4. WHEN `RFIND` is issued and the previous FIND used `FIRST` direction, THE FindEngine SHALL re-execute as `NEXT` from the last match position (not restart from document beginning). [FFE-CMD-4]
5. WHEN `RFIND` is issued and the previous FIND used `LAST` direction, THE FindEngine SHALL re-execute as `PREV` from the last match position. [FFE-CMD-4]
6. WHEN `RFIND` wraps past the document boundary without finding a match, THE FindEngine SHALL report "'text' NOT FOUND" without wrapping around to the other end. [FFE-CMD-4]

---

### Requirement 6: CHANGE Command — Literal Replacement

**User Story:** As a developer, I want to replace text in the document with precise scope, direction, and column-range control, so that I can safely transform content without affecting unintended lines or columns.

**Source:** [FFE-CMD-5]

#### Acceptance Criteria

1. WHEN `CHANGE 'old' 'new'` is issued, THE FindEngine SHALL find and replace the first occurrence of `old` on or after the current cursor position with `new`. [FFE-CMD-5]
2. WHEN `CHANGE 'old' 'new' ALL` is issued, THE FindEngine SHALL replace every occurrence of `old` in the resolved SearchScope and SHALL return the total substitution count. [FFE-CMD-5]
3. WHEN `CHANGE 'old' 'new' NEXT` is issued, THE FindEngine SHALL replace the next occurrence after the current cursor position. [FFE-CMD-5]
4. WHEN `CHANGE 'old' 'new' PREV` is issued, THE FindEngine SHALL replace the nearest occurrence before the current cursor position. [FFE-CMD-5]
5. WHEN `CHANGE 'old' 'new' FIRST` is issued, THE FindEngine SHALL replace the first occurrence in the resolved SearchScope. [FFE-CMD-5]
6. WHEN `CHANGE 'old' 'new' LAST` is issued, THE FindEngine SHALL replace the last occurrence in the resolved SearchScope. [FFE-CMD-5]
7. WHEN no match is found, THE FindEngine SHALL return a "not found" result and the command layer SHALL display "'old' NOT FOUND" without modifying the document. [FFE-CMD-5]
8. WHEN a literal CHANGE produces a replacement that differs in length from the original, THE FindEngine SHALL update all subsequent byte positions correctly so that multi-match ALL operations remain consistent. [FFE-CMD-5]

---

### Requirement 7: CHANGE Command — Scope and Column Modifiers

**User Story:** As a developer, I want CHANGE to respect tag state, visibility flags, and column boundaries, so that bulk replacements apply only to the intended subset of lines and columns.

**Source:** [FFE-CMD-5]

#### Acceptance Criteria

1. WHEN `CHANGE 'old' 'new' TAGGED` is issued, THE FindEngine SHALL restrict replacements to lines whose `tagged` flag is true. [FFE-CMD-5]
2. WHEN `CHANGE 'old' 'new' EXCLUDED` is issued, THE FindEngine SHALL restrict replacements to lines whose `excluded` flag is true. [FFE-CMD-5]
3. WHEN `CHANGE 'old' 'new' VISIBLE` is issued, THE FindEngine SHALL restrict replacements to lines whose `visible` flag is true. [FFE-CMD-5]
4. WHEN `CHANGE 'old' 'new' IN col1 col2` is issued, THE FindEngine SHALL restrict substitutions to matches whose byte positions fall within columns col1 through col2 inclusive on each line. [FFE-CMD-5]
5. WHILE active Bounds are set, THE FindEngine SHALL restrict CHANGE operations to characters within the active column Bounds even when no explicit `IN col1 col2` clause is present. [FFE-CMD-5]
6. WHEN `CHANGE 'old' 'new' IN col1 col2` is issued and active Bounds are set, THE FindEngine SHALL use the intersection of [col1, col2] and the active Bounds as the effective column range. [FFE-CMD-5]
7. WHEN a CHANGE command completes successfully (one or more replacements made), THE command layer SHALL record the entire operation as a single undoable Transaction via the undo-redo-transactions system. [FFE-CMD-5]
8. WHEN `CHANGE 'old' 'new' ALL TAGGED` combines multiple modifiers, THE FindEngine SHALL apply all constraints conjunctively. [FFE-CMD-5]

---

### Requirement 8: CHANGE Command — Regular Expression Replacement

**User Story:** As a developer, I want regex-based replacements with group substitution, so that I can restructure text using captured patterns without manual editing.

**Source:** [FFE-CMD-5], [SCI-DOC-17], [SCI-RES]

#### Acceptance Criteria

1. WHEN `CHANGE REGEX 'pattern' 'replacement'` is issued, THE FindEngine SHALL interpret the first argument as a regular expression and apply the same direction, scope, and column modifiers as literal CHANGE. [FFE-CMD-5]
2. THE SubstitutionTemplate SHALL support group references `\0` (entire match), `\1` through `\9` (captured sub-groups) in the replacement string. [SCI-DOC-17, SCI-RES]
3. THE SubstitutionTemplate SHALL additionally support `$0` through `$9` as alternative group reference syntax for compatibility. [WB]
4. WHEN a group reference in the replacement refers to an unmatched group, THE FindEngine SHALL substitute an empty string for that group reference. [SCI-RES]
5. WHEN `CHANGE REGEX 'pattern' 'replacement' ALL` is issued, THE FindEngine SHALL iterate through all non-overlapping matches in the resolved scope, expanding the SubstitutionTemplate for each match independently. [FFE-CMD-5, SCI-DOC-17]
6. WHEN a regex replacement changes the length of matched text, THE FindEngine SHALL adjust search positions for subsequent matches in an ALL operation to account for the length delta. [SCI-DOC-17]
7. IF a zero-length regex match occurs (e.g., from `a*` matching empty), THE FindEngine SHALL advance by at least one character to prevent infinite loops during ALL replacement. [SCI-RES]
8. THE FindEngine SHALL expose a `substitute(template, captures)` method that expands a SubstitutionTemplate against a set of CaptureGroups, returning the expanded replacement text. [SCI-DOC-17]

---

### Requirement 9: RCHANGE Command — Repeat Previous Change

**User Story:** As a developer, I want to repeat the previous CHANGE with a single command, so that I can apply the same substitution to the next occurrence without retyping the search and replacement text.

**Source:** [FFE-CMD-6]

#### Acceptance Criteria

1. WHEN `RCHANGE` is issued, THE FindEngine SHALL repeat the most recently executed CHANGE operation with all its original arguments and modifiers, applying it to the next applicable occurrence. [FFE-CMD-6]
2. IF no previous CHANGE operation exists in the current session, THEN THE FindEngine SHALL return an error "No previous CHANGE to repeat" and SHALL NOT modify the document. [FFE-CMD-6]
3. THE FindEngine SHALL store the most recent CHANGE arguments (search term, replacement, mode, modifiers) in the FindState so that RCHANGE is available across multiple command submissions within the same session. [FFE-CMD-6]
4. WHEN `RCHANGE` is issued after a previous `CHANGE ... FIRST`, THE FindEngine SHALL re-execute as NEXT from the last replacement position. [FFE-CMD-6]
5. WHEN `RCHANGE` is issued after a previous `CHANGE ... LAST`, THE FindEngine SHALL re-execute as PREV from the last replacement position. [FFE-CMD-6]
6. EACH `RCHANGE` execution SHALL be recorded as its own undoable Transaction. [FFE-CMD-6]

---

### Requirement 10: Unicode Case Folding

**User Story:** As a developer working with multilingual documents, I want case-insensitive search to correctly handle all Unicode scripts (not just ASCII), so that searching for "straße" matches "STRASSE" and Turkish "İ" matches correctly under Turkish locale rules.

**Source:** [SCI-DOC-17], [WB]

#### Acceptance Criteria

1. THE CaseFolder SHALL implement Unicode Full Case Folding as defined by the Unicode CaseFolding.txt data file (status C + F mappings). [SCI-DOC-17, WB]
2. WHEN case-insensitive search is active, THE FindEngine SHALL fold both the search term and each document segment through the CaseFolder before byte comparison. [SCI-DOC-17]
3. THE CaseFolder SHALL handle one-to-many case mappings (e.g., German "ß" folds to "ss", producing a folded string longer than the input). [WB]
4. THE CaseFolder SHALL handle multi-byte UTF-8 sequences correctly, never splitting a code point across fold boundaries. [WB]
5. THE CaseFolder SHALL be stateless and thread-safe, enabling concurrent use by multiple search operations. [WB]
6. THE FindEngine SHALL pre-fold the search term once per FindRequest and compare against lazily-folded document segments, avoiding redundant folding of the search term on each line. [WB]
7. WHEN case-insensitive regex search is active, THE RegexEngine SHALL fold characters during NFA execution using the same CaseFolder. [SCI-DOC-17, SCI-RES]
8. THE CaseFolder SHALL support a configurable locale hint for locale-sensitive folding (e.g., Turkish dotted-I rules), defaulting to locale-independent Unicode folding when no hint is provided. [WB]

---

### Requirement 11: Whole Word and Word Start Matching

**User Story:** As a developer, I want to restrict search matches to whole words or word-start positions, so that searching for "log" does not match "logging" or "catalog" when I need exact word matches.

**Source:** [SCI-DOC-17], [SCI-RES]

#### Acceptance Criteria

1. WHEN the WORD modifier is specified on a FIND or CHANGE command, THE FindEngine SHALL verify that character-class transitions (word ↔ non-word) exist at both the start and end of any match. [SCI-DOC-17]
2. WHEN the WORDSTART modifier is specified, THE FindEngine SHALL verify that a character-class transition exists at the start of the match but not require one at the end. [SCI-DOC-17]
3. THE FindEngine SHALL use the document's character classification table (from `encoding-and-characters`) to determine word-character membership for boundary detection. [SCI-RES]
4. WHEN whole-word matching is combined with case-insensitive mode, THE FindEngine SHALL apply case folding first and then verify word boundaries on the original document positions. [SCI-DOC-17]
5. THE word boundary check SHALL handle multi-byte UTF-8 characters correctly — the character before/after the match SHALL be classified by its full code point, not individual bytes. [SCI-RES]

---

### Requirement 12: Regex Engine — NFA Compilation and Execution

**User Story:** As the find engine, I want a compiled NFA-based regex that supports the full POSIX-like syntax with extensions, so that regex searches execute efficiently across large documents without backtracking explosion.

**Source:** [SCI-RES]

#### Acceptance Criteria

1. WHEN a regex pattern is submitted for search, THE RegexEngine SHALL compile it into an NFA representation suitable for execution against a character indexer. [SCI-RES]
2. IF the NFA buffer would exceed the maximum compiled size, THE RegexEngine SHALL return the error "Pattern too long". [SCI-RES]
3. IF an unmatched opening parenthesis is detected at end of pattern, THE RegexEngine SHALL return "Unmatched (". [SCI-RES]
4. IF an unmatched closing parenthesis is detected, THE RegexEngine SHALL return "Unmatched )". [SCI-RES]
5. IF a quantifier (*, +, ?) appears at the start of a pattern or after an anchor, THE RegexEngine SHALL return "Empty closure" or "Illegal closure" respectively. [SCI-RES]
6. IF a backreference `\n` refers to an uncaptured group, THE RegexEngine SHALL return "Undetermined reference". [SCI-RES]
7. IF a backreference creates a cyclical reference (referring to the currently open group), THE RegexEngine SHALL return "Cyclical reference". [SCI-RES]
8. WHEN a previously compiled pattern exists and an empty pattern is submitted, THE RegexEngine SHALL reuse the previously compiled NFA. [SCI-RES]
9. IF no previously compiled pattern exists and an empty pattern is submitted, THE RegexEngine SHALL return "No previous regular expression". [SCI-RES]
10. THE RegexEngine SHALL execute the compiled NFA against a CharacterIndexer trait, searching within a specified byte range [start, end). [SCI-RES]
11. WHEN the NFA starts with a literal character, THE RegexEngine SHALL use a fast-path scan (memchr) to locate the first candidate position before attempting a full NFA match. [SCI-RES]
12. WHEN a greedy closure is matched, THE RegexEngine SHALL consume maximum characters first, then backtrack attempting shorter matches until success or exhaustion. [SCI-RES]
13. WHEN a lazy closure is matched, THE RegexEngine SHALL attempt the shortest match first, extending forward until success or exhaustion. [SCI-RES]

---

### Requirement 13: Find State and Session Persistence

**User Story:** As a developer, I want the find/replace engine to remember my last search and replacement across command submissions within a session, so that RFIND, RCHANGE, and the find panel can display the previous search context.

**Source:** [FFE-CMD-4], [FFE-CMD-6], [FFE-CMD-9]

#### Acceptance Criteria

1. THE FindState SHALL persist the following across command submissions within a single editing session: search term, search mode (Literal/Regex/Hex), case sensitivity flag, direction, scope modifiers, replacement text (if CHANGE), and column range. [FFE-CMD-4, FFE-CMD-6]
2. THE FindState SHALL store a history of the last N search terms (configurable, default 20) for user recall in the find panel. [WB]
3. THE FindState SHALL store a separate history of the last N replacement texts. [WB]
4. WHEN `RESET` is issued with no arguments, THE FindState SHALL clear any active find highlight and temporary search filters but SHALL retain the stored search history and last-search parameters for RFIND/RCHANGE. [FFE-CMD-9]
5. WHEN `RESET ALL` is issued, THE FindState SHALL clear the last-search parameters (RFIND/RCHANGE will report "No previous FIND/CHANGE") but SHALL retain the search history list. [FFE-CMD-9]
6. THE FindState SHALL be per-document — each open document maintains its own last-search and last-change state. [WB]
7. THE FindState SHALL be serialisable for session persistence across application restarts (via the startup-and-session system). [WB]

---

### Requirement 14: Incremental Search (Search-as-You-Type)

**User Story:** As a developer, I want matches to update live as I type characters into the find field, so that I get immediate visual feedback about where my search term appears without needing to press Enter.

**Source:** [WB]

#### Acceptance Criteria

1. WHEN the find panel is open and the user types a character into the search field, THE FindEngine SHALL execute a forward search from the current cursor position with the partial search text and navigate to the first match within a configurable time budget (default ≤50 ms). [WB]
2. WHEN the search text changes during incremental search, THE FindEngine SHALL cancel any in-progress search and start a new search with the updated text. [WB]
3. IF no match is found during incremental search, THE find panel SHALL indicate "no match" visually (e.g., red background on the search field) without displaying a status message. [WB]
4. WHEN incremental search finds a match, THE viewport SHALL scroll to reveal the match and the match SHALL be highlighted using the text-decorations indicator system. [WB]
5. WHEN the user deletes characters from the search field during incremental search, THE FindEngine SHALL re-execute the search from the original start position (not from the current match position) to ensure consistent behaviour. [WB]
6. THE FindEngine SHALL debounce incremental search requests — if keystrokes arrive faster than the search can complete, only the latest state SHALL be searched. [WB]
7. WHEN the search field is empty, THE FindEngine SHALL clear all incremental search highlights and restore the viewport to its pre-search position. [WB]
8. THE incremental search SHALL respect the current case-sensitivity and mode settings (literal/regex) configured in the find panel. [WB]

---

### Requirement 15: Highlight All Matches Mode

**User Story:** As a developer, I want all visible matches of my current search term highlighted simultaneously while the find panel is open, so that I can see the distribution and frequency of matches across the visible document area.

**Source:** [WB]

#### Acceptance Criteria

1. WHEN "Highlight All" mode is enabled and the find panel has a non-empty search term, THE FindEngine SHALL locate all matches within the currently visible viewport range and report them to the text-decorations system for rendering. [WB]
2. THE highlight-all computation SHALL execute asynchronously or within a time budget so that it does not block viewport rendering or typing responsiveness. [WB]
3. WHEN the viewport scrolls while "Highlight All" is active, THE FindEngine SHALL update the set of highlighted matches to reflect the newly visible region. [WB]
4. WHEN the search term changes, THE FindEngine SHALL clear previous highlights and recompute matches for the new term. [WB]
5. WHEN the find panel is closed, THE FindEngine SHALL clear all highlight-all decorations. [WB]
6. THE FindEngine SHALL limit the maximum number of highlighted matches to a configurable threshold (default 1000) to prevent performance degradation on large documents with many matches. [WB]
7. IF the number of matches exceeds the threshold, THE FindEngine SHALL highlight matches in the visible viewport and report "N+ matches (showing first 1000)" in the find panel status. [WB]
8. THE highlight-all decorations SHALL use a distinct decoration style (configurable via theme) that differs from the current-match highlight, so that the active match remains visually distinguishable. [WB]

---

### Requirement 16: Find Integration with Exclude/Show/Reset

**User Story:** As a developer, I want the EXCLUDE command to use the same search engine as FIND, and I want RESET to correctly clear find-related state, so that the search subsystem is consistent across all commands that locate text.

**Source:** [FFE-CMD-7], [FFE-CMD-8], [FFE-CMD-9]

#### Acceptance Criteria

1. WHEN `EXCLUDE 'text'` or `EXCLUDE REGEX 'pattern'` is issued, THE command layer SHALL delegate the text-matching operation to the FindEngine using the same literal/regex matching logic as FIND. [FFE-CMD-7]
2. WHEN `SHOW 'text'` or `SHOW REGEX 'pattern'` is issued, THE command layer SHALL delegate the text-matching operation to the FindEngine to identify which excluded lines contain the text. [FFE-CMD-8]
3. THE EXCLUDE and SHOW text-matching operations SHALL respect the current case-sensitivity setting of the FindEngine. [FFE-CMD-7, FFE-CMD-8]
4. THE EXCLUDE and SHOW operations SHALL NOT update the FindState (they do not affect RFIND/RCHANGE state). [FFE-CMD-7, FFE-CMD-8]
5. WHEN `RESET` is issued, THE command layer SHALL clear any active highlight-all decorations and incremental search state maintained by the FindEngine. [FFE-CMD-9]
6. WHEN `RESET` is issued, THE FindState's last-search parameters SHALL remain intact for RFIND unless `RESET ALL` is specifically issued. [FFE-CMD-9]

---

### Requirement 17: Command Framework Integration

**User Story:** As a workbench platform component, I want all find/replace commands routable through the command framework with proper metadata, so that they integrate with keybindings, menus, scripting, and the undo system.

**Source:** [WB]

#### Acceptance Criteria

1. THE find-and-replace crate SHALL register the following commands with the command framework: `find`, `rfind`, `change`, `rchange`, `find_next`, `find_prev`, `find_all`, `replace_all`. [WB]
2. EACH registered command SHALL include metadata (display name, description, default keybinding suggestion, category "Search") for menu and keybinding systems. [WB]
3. ALL CHANGE operations (including RCHANGE) SHALL be wrapped in an undo transaction before mutating the document. [FFE-CMD-5, FFE-CMD-6]
4. WHEN a CHANGE ALL operation makes multiple replacements, THE entire batch SHALL be grouped as a single undo transaction — one UNDO reverses all replacements from that command. [FFE-CMD-5]
5. FIND operations SHALL NOT create undo transactions — they are read-only viewport/cursor movements. [FFE-CMD-3]
6. THE find/replace commands SHALL be invocable from Lua macros via the scripting bridge with the same argument semantics as command-line input. [WB]
7. THE FindEngine SHALL emit events (find_started, match_found, find_completed, replace_completed) that plugins and the UI can subscribe to for status updates and progress reporting. [WB]

---

### Requirement 18: Character Indexer Abstraction

**User Story:** As the find engine, I want an abstract character access interface over the document buffer, so that the search algorithm works with any buffer representation (gap buffer, rope, or streaming view) without depending on contiguous memory.

**Source:** [SCI-RES], [SCI-DOC-17]

#### Acceptance Criteria

1. THE FindEngine SHALL define a `CharacterIndexer` trait with a `char_at(position: BytePosition) -> u8` method for single-byte access. [SCI-RES]
2. THE `CharacterIndexer` trait SHALL include a `slice(start, end) -> &[u8]` method for bulk access when the range is contiguous, with a fallback to byte-by-byte access when it is not. [SCI-RES]
3. THE `CharacterIndexer` trait SHALL include a `move_position_outside_char(position, direction) -> BytePosition` method to align positions to UTF-8 character boundaries. [SCI-RES]
4. THE document-model crate SHALL provide a `CharacterIndexer` implementation over its GapBuffer/SplitView that is usable by the FindEngine without requiring gap compaction for each search. [SCI-DOC-17]
5. THE `CharacterIndexer` SHALL provide a `line_range(line: LineNumber) -> (BytePosition, BytePosition)` method returning the start and end byte positions of a given line, enabling line-scoped searches. [SCI-DOC-17]
6. THE `CharacterIndexer` SHALL be safe to use from background threads when protected by the document's read lock. [WB]

---

### Requirement 19: Performance and Large-File Considerations

**User Story:** As a developer working with large files (100K+ lines), I want FIND and CHANGE operations to remain responsive, so that search does not freeze the editor even on multi-megabyte documents.

**Source:** [WB], [SCI-DOC-17]

#### Acceptance Criteria

1. THE FindEngine SHALL support cancellation of in-progress search operations via a cancellation token, enabling the user to abort long-running FIND ALL or CHANGE ALL operations. [WB]
2. WHEN `FIND ALL` or `CHANGE ALL` executes on a large document, THE FindEngine SHALL report progress periodically (every N matches or every M milliseconds, configurable) via the event system. [WB]
3. THE literal search (case-sensitive) SHALL use an optimised byte-scanning algorithm (e.g., memchr + memcmp or Boyer-Moore variant) achieving sub-linear average-case performance. [SCI-DOC-17]
4. THE regex search SHALL avoid catastrophic backtracking by implementing NFA-based matching (not backtracking-only) and enforcing a configurable match-attempt limit per position (default 10,000 steps). [SCI-RES, WB]
5. IF the match-attempt limit is exceeded at a position, THE FindEngine SHALL skip that position and continue to the next candidate, logging a warning but not aborting the entire search. [WB]
6. THE FindEngine SHALL avoid allocating per-line during FIND ALL — match results SHALL be accumulated in a pre-allocated or amortised collection. [WB]
7. WHEN searching within column bounds, THE FindEngine SHALL extract the bounded slice once per line rather than re-checking column bounds for each character position. [WB]

---

### Requirement 20: Error Handling and Edge Cases

**User Story:** As a developer, I want clear error messages when search operations fail due to invalid input, and I want the engine to handle edge cases (empty documents, zero-length matches, binary content) gracefully.

**Source:** [FFE-CMD-3], [SCI-RES], [WB]

#### Acceptance Criteria

1. WHEN a FIND or CHANGE command is issued with an empty search term and no previous search exists, THE FindEngine SHALL return an error "No search term specified". [FFE-CMD-3]
2. WHEN a FIND or CHANGE command is issued with an empty search term and a previous search exists, THE FindEngine SHALL reuse the previous search term (same as RFIND/RCHANGE behaviour). [FFE-CMD-4, FFE-CMD-6]
3. WHEN the document is empty (zero lines), THE FindEngine SHALL immediately return "not found" without error for any search operation. [WB]
4. WHEN the document is read-only and a CHANGE command is issued, THE FindEngine SHALL return an error "Document is read-only" and SHALL NOT attempt the search or modification. [WB]
5. WHEN a regex replacement template contains an invalid escape sequence, THE FindEngine SHALL return an error describing the invalid escape rather than producing corrupt output. [SCI-RES]
6. WHEN CHANGE ALL produces zero replacements (search term found nowhere), THE FindEngine SHALL report "'old' NOT FOUND" — the same message as single-match CHANGE with no match. [FFE-CMD-5]
7. THE FindEngine SHALL handle documents containing null bytes (0x00) without truncating the search — null is treated as a regular byte value. [WB]
8. WHEN a search term contains characters that are incomplete UTF-8 sequences, THE FindEngine SHALL search for the raw bytes as-is in literal mode (the encoding layer handles validation separately). [WB]

---

