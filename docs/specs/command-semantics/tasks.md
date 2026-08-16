# Implementation Plan: Command Semantics Engine (`ff-command-semantics`)

## Overview

This plan covers the complete implementation of the `ff-command-semantics` crate — the ISPF-inspired primary command execution pipeline for FileForgeWorkbench. The command semantics engine accepts raw command-line text, parses it into structured tokens, resolves target scope, validates preconditions, builds execution plans, executes transactionally via the undo system, and reports results as concise status messages.

This is a **Wave 5 (Command Engine)** sub-project. It depends on `ff-command` (command-framework), `ff-document-model`, `ff-undo-redo`, `ff-logging`, and `ff-configuration`. It is consumed by `find-and-replace`, `line-commands`, `exclude-show-filter`, and `navigation-commands`.

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-command-semantics/Cargo.toml` with dependencies (ff-command, ff-document-model, ff-undo-redo, ff-logging, ff-configuration, thiserror, proptest dev-dep)
  - [ ] 1.2 Create `crates/ff-command-semantics/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `parser.rs`, `line_parser.rs`, `scope.rs`, `engine.rs`, `session.rs`, `config.rs`, `help.rs`, `error.rs`, `status.rs`
  - [ ] 1.4 Add `ff-command-semantics` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. Primary Command Parser — tokenizer core
  - [ ] 2.1 Define `CommandToken` enum with variants: BareWord(String), QuotedString(String), HexLiteral(Vec<u8>)
  - [ ] 2.2 Define `ParsedCommand` struct with fields: command_name (String), arguments (Vec<CommandToken>)
  - [ ] 2.3 Implement whitespace-delimited tokenization splitting command line into first token (command name) and remaining argument tokens
  - [ ] 2.4 Implement case-insensitive command name normalization (uppercase the command name after extraction)
  - [ ] 2.5 Implement empty/whitespace-only input detection returning `None` result
  - [ ] 2.6 Write unit tests for basic tokenization: single command, command with bare args, empty input, whitespace-only input
  - Covers: Requirement 3 (AC 3.1, 3.4, 3.5)

- [ ] 3. Primary Command Parser — quoted strings and hex literals
  - [ ] 3.1 Implement single-quote delimited string parsing with content preserved as single token (quotes stripped)
  - [ ] 3.2 Implement double-quote delimited string parsing with content preserved as single token (quotes stripped)
  - [ ] 3.3 Implement escaped quote handling: doubled quote within a quoted string represents a single literal quote character
  - [ ] 3.4 Implement hex literal parsing: pattern `X'hh...'` (case-insensitive X) decoded to byte vector
  - [ ] 3.5 Implement unclosed quote detection returning syntax error with descriptive message
  - [ ] 3.6 Implement invalid hex literal detection (odd digit count, non-hex characters) returning syntax error
  - [ ] 3.7 Write unit tests for quoted strings, escaped quotes, hex literals, unclosed quotes, and invalid hex
  - Covers: Requirement 3 (AC 3.2, 3.3, 3.7, 3.8)

- [ ] 4. Primary Command Parser — round-trip property and edge cases
  - [ ] 4.1 Implement `CommandToken::reconstruct()` method that produces text which re-parses to the same token
  - [ ] 4.2 Implement `ParsedCommand::reconstruct()` that joins command name and reconstructed argument tokens
  - [ ] 4.3 Write unit tests validating the round-trip property for various command lines
  - [ ] 4.4 Write edge case tests: multiple consecutive spaces, mixed quote types, hex literal as first argument, command name with digits
  - Covers: Requirement 3 (AC 3.6)

- [ ] 5. Line Command Parser
  - [ ] 5.1 Define `LineCommandKind` enum with all defined kinds: single-line (C, M, D, R, X, I, A, B, O, W, S, T, ShiftRight, ShiftLeft, ParenOpen, ParenClose, BracketClose) and block forms (CC, MM, DD, RR, XX, TT)
  - [ ] 5.2 Define `ParsedLineCommand` struct with fields: kind (LineCommandKind), count (u32), and `Unknown(String)` variant
  - [ ] 5.3 Implement parsing logic: maximal leading alphabetic prefix as kind, trailing digits as count (default 1)
  - [ ] 5.4 Implement case-insensitive kind normalization (uppercase)
  - [ ] 5.5 Implement unknown kind detection producing `Unknown` variant with original text preserved
  - [ ] 5.6 Implement empty/whitespace-only input returning `None`
  - [ ] 5.7 Implement repeat count range validation: counts 1–99999 accepted, >99999 produces error
  - [ ] 5.8 Write unit tests: all defined kinds, counts, unknown kinds, empty input, count overflow, case insensitivity
  - Covers: Requirement 4 (AC 4.1–4.7)

- [ ] 6. Scope Resolution — priority algorithm
  - [ ] 6.1 Define `Scope` struct representing a resolved set of target lines and optional column bounds
  - [ ] 6.2 Define `ScopeSource` enum with variants: ExplicitRange, BlockSource, SingleLineCommand, Tagged, Visibility, CursorLine, EntireDocument
  - [ ] 6.3 Define `VisibilityModifier` enum: Visible, Excluded, All
  - [ ] 6.4 Define `TagModifier` enum: Tagged, NonTagged
  - [ ] 6.5 Implement the priority-ordered resolution algorithm evaluating sources from highest to lowest priority
  - [ ] 6.6 Write unit tests for each priority level resolving independently
  - Covers: Requirement 2 (AC 2.1)

- [ ] 7. Scope Resolution — modifiers and bounds
  - [ ] 7.1 Implement ALL modifier: include all lines regardless of visibility state
  - [ ] 7.2 Implement VISIBLE modifier: include only visible (non-excluded) lines
  - [ ] 7.3 Implement EXCLUDED modifier: include only excluded (hidden) lines
  - [ ] 7.4 Implement TAGGED modifier: include only lines with tag flag set
  - [ ] 7.5 Implement NONTAGGED modifier: include only lines without tag flag set
  - [ ] 7.6 Implement column bounds restriction: apply left/right bound to column-sensitive operations
  - [ ] 7.7 Implement no-scope-found error: emit error Status_Message when no scope resolves and command doesn't allow document-wide scope
  - [ ] 7.8 Implement conflict resolution: higher-priority source takes precedence, lower-priority ignored without error
  - [ ] 7.9 Write unit tests for each modifier in isolation and combined with other scope sources
  - Covers: Requirement 2 (AC 2.2–2.9)

- [ ] 8. Session State management
  - [ ] 8.1 Define `SessionState` struct with fields: pending_line_commands, last_command, last_scope, cursor_position, line_tags, status_message
  - [ ] 8.2 Implement `SessionState::new()` constructor with empty/default state
  - [ ] 8.3 Implement `add_line_command()` for accumulating pending line commands from the prefix area
  - [ ] 8.4 Implement `consume_line_commands()` for clearing consumed line commands after successful execution
  - [ ] 8.5 Implement `retain_line_commands()` for preserving pending commands on failure
  - [ ] 8.6 Write unit tests for session state lifecycle: add, consume, retain, clear
  - Covers: Requirement 1 (AC 1.2, 1.5, 1.6)

- [ ] 9. Command Engine — execution pipeline core
  - [ ] 9.1 Define `CommandEngine` struct holding references to `CommandRegistry`, `SessionState`, undo transaction manager, and configuration
  - [ ] 9.2 Implement the 10-step pipeline: collect → parse → normalize → scope → validate → plan → execute → update state → clear consumed → emit status
  - [ ] 9.3 Implement empty command line with pending line commands: execute pending line commands
  - [ ] 9.4 Implement empty command line with no pending line commands: emit "No command" status
  - [ ] 9.5 Implement unrecognised command handling: emit error status without modifying document state
  - [ ] 9.6 Write unit tests for pipeline steps: successful execution, empty with pending, empty without pending, unrecognised
  - Covers: Requirement 1 (AC 1.1–1.4)

- [ ] 10. Command Engine — transactional execution and undo integration
  - [ ] 10.1 Implement undo transaction wrapping: start transaction before execution, commit on success, rollback on failure
  - [ ] 10.2 Implement rollback guarantee: on mid-execution failure, no partial state persists
  - [ ] 10.3 Implement line command retention on failure: pending commands preserved in SessionState
  - [ ] 10.4 Implement line command clearing on success: consumed commands removed from SessionState
  - [ ] 10.5 Write unit tests for transaction commit, rollback, line command lifecycle during success and failure
  - Covers: Requirement 1 (AC 1.5, 1.6, 1.7)

- [ ] 11. Command Engine — runtime registration and dispatch integration
  - [ ] 11.1 Implement command registration via `command-framework` `register()` API
  - [ ] 11.2 Implement all commands accessible through `Command_Dispatch` interface for keyboard, menu, macro, and plugin invocation
  - [ ] 11.3 Implement runtime extensibility: new commands registrable without recompilation
  - [ ] 11.4 Write unit tests for registration, dispatch invocation, and runtime extension scenarios
  - Covers: Requirement 1 (AC 1.8, 1.9)

- [ ] 12. Error Handling — status message system
  - [ ] 12.1 Define `StatusMessage` struct with fields: text (String), severity (Severity enum), command_name (Option<String>)
  - [ ] 12.2 Define `Severity` enum: Info, SyntaxError, StructureError, RuntimeError
  - [ ] 12.3 Implement 200-character length enforcement with trailing ellipsis truncation
  - [ ] 12.4 Implement syntax error formatting: prefix "Syntax error", includes problematic text and command name
  - [ ] 12.5 Implement structure error formatting: prefix "Structure error", includes conflicting command info
  - [ ] 12.6 Implement runtime error formatting: prefix "Error", includes command name and failure description
  - [ ] 12.7 Implement success message formatting for informational status (e.g., "CHANGE - 3 occurrences changed")
  - [ ] 12.8 Implement command name inclusion guarantee: all error messages identify the failing command
  - [ ] 12.9 Write unit tests for each severity, truncation at 200 chars, command name presence, and success messages
  - Covers: Requirement 5 (AC 5.1–5.7)

- [ ] 13. Configuration Options
  - [ ] 13.1 Define `CommandConfig` struct with all six configuration keys and their default values
  - [ ] 13.2 Implement `commands.find_default_scope` parsing: "visible" | "all" | "excluded", default "visible"
  - [ ] 13.3 Implement `commands.bounds_affect_find` parsing: boolean, default true
  - [ ] 13.4 Implement `commands.case_sensitive_find` parsing: boolean, default false
  - [ ] 13.5 Implement `commands.default_shift_width` parsing: integer 1–72, default 2, with clamping and WARN log on out-of-range
  - [ ] 13.6 Implement `commands.reset_clears_tags` parsing: boolean, default false
  - [ ] 13.7 Implement `commands.invalid_line_command_policy` parsing: "reject" | "ignore", default "reject"
  - [ ] 13.8 Implement fallback-to-default on invalid values with WARN-level log including key name and applied default
  - [ ] 13.9 Implement startup reading and hot-reload notification subscription for re-reading config
  - [ ] 13.10 Write unit tests for default values, invalid value fallback, clamping, hot-reload application
  - Covers: Requirement 6 (AC 6.1–6.6)

- [ ] 14. Invalid Line Command Policy enforcement
  - [ ] 14.1 Implement "reject" policy: produce error StatusMessage for unrecognised line commands, halt pipeline
  - [ ] 14.2 Implement "ignore" policy: silently discard unrecognised line commands, continue pipeline
  - [ ] 14.3 Write unit tests for both policies with unknown line command input
  - Covers: Requirement 6 (AC 6.4, 6.5)

- [ ] 15. HELP Command implementation
  - [ ] 15.1 Register HELP command with Command_ID `"help.show"` via command-framework
  - [ ] 15.2 Implement `HELP` with no arguments: display summary of all primary commands grouped by category
  - [ ] 15.3 Implement `HELP <commandname>`: display full help text for a registered command (syntax, modifiers, arguments, examples)
  - [ ] 15.4 Implement `HELP LINECOMMANDS`: display summary of all line commands with abbreviations, block forms, descriptions
  - [ ] 15.5 Implement `HELP MACRO` / `HELP API`: display summary of Lua macro API functions
  - [ ] 15.6 Implement unknown topic handling: display message listing available topics with close-match suggestions
  - [ ] 15.7 Implement mode safety: HELP valid in edit, view, and browse modes; does not modify document state
  - [ ] 15.8 Implement history exclusion: HELP command not recorded in command history
  - [ ] 15.9 Write unit tests for each HELP variant, unknown topic, mode safety, and history exclusion
  - Covers: Requirement 7 (AC 7.1–7.8)

- [ ] 16. Integration with upstream crates
  - [ ] 16.1 Wire CommandEngine to `ff-command` CommandRegistry for command lookup during normalization step
  - [ ] 16.2 Wire CommandEngine to `ff-undo-redo` for transaction wrapping during execution step
  - [ ] 16.3 Wire CommandEngine to `ff-configuration` for reading and hot-reloading configuration keys
  - [ ] 16.4 Wire scope resolution to `ff-document-model` for line visibility state and tag queries
  - [ ] 16.5 Write integration tests validating end-to-end pipeline with mocked upstream interfaces
  - Covers: Requirement 1 (AC 1.7, 1.8, 1.9), Requirement 2 (AC 2.2–2.6)

- [ ] 17. Thread safety and concurrency
  - [ ] 17.1 Ensure `CommandEngine`, `SessionState`, and `CommandConfig` are `Send + Sync`
  - [ ] 17.2 Write multi-threaded test: concurrent command execution from multiple sources
  - [ ] 17.3 Write multi-threaded test: concurrent config hot-reload during command execution
  - Covers: Cross-cutting (GUI independence, async I/O principle)

- [ ] 18. Property-based tests
  - [ ] 18.1 Write PBT: Primary command parser round-trip property
  - [ ] 18.2 Write PBT: Primary command parser case-insensitive normalization property
  - [ ] 18.3 Write PBT: Line command parser kind/count decomposition property
  - [ ] 18.4 Write PBT: Line command parser count range validation property
  - [ ] 18.5 Write PBT: Scope resolution priority ordering property
  - [ ] 18.6 Write PBT: Status message length invariant property
  - [ ] 18.7 Write PBT: Configuration clamping property
  - [ ] 18.8 Write PBT: Empty input detection property
  - Covers: All requirements (property-based validation)

---

## Property-Based Test Definitions

### Property 1: Primary Command Parser Round-Trip

**Validates: Requirement 3.6**

- **Statement:** For any valid command line input (containing a command name and zero or more arguments — bare words, quoted strings, hex literals), parsing the input and then reconstructing text from the parsed tokens SHALL produce output that, when re-parsed, yields the same token sequence.
- **Strategy:** Generate:
  - Command names: uppercase strings of 1–20 alphabetic characters
  - Arguments: mix of bare words (alphanumeric, no spaces), quoted strings (arbitrary content including spaces, doubled-quotes for escapes), hex literals (even-length hex digit strings wrapped in X'...')
  - Combine into command lines with 0–8 arguments separated by single spaces
- **Invariant:** `parse(reconstruct(parse(input))) == parse(input)` for all generated inputs

### Property 2: Primary Command Parser Case-Insensitive Normalization

**Validates: Requirement 3.4**

- **Statement:** For any command name string, parsing SHALL normalize it to uppercase such that any case variation of the same characters resolves to the identical normalized command name.
- **Strategy:** Generate:
  - Base command names: strings of 1–15 ASCII letters
  - For each base name, generate 2–5 random case permutations (e.g., "find", "FIND", "Find", "fInD")
- **Invariant:** For all case permutations of the same base name, `parse(permutation).command_name` is identical (all uppercase)

### Property 3: Line Command Parser Kind/Count Decomposition

**Validates: Requirement 4.5**

- **Statement:** For any prefix-area string consisting of a valid alphabetic kind prefix followed by numeric digits, the parser SHALL unambiguously split it into the maximal alphabetic prefix (kind) and remaining digits (count). Reconstructing the original by concatenating kind and count digits SHALL produce the original string (case-normalized).
- **Strategy:** Generate:
  - Kind: one of the defined line command kinds (C, M, D, R, X, I, A, B, O, W, S, T, CC, MM, DD, RR, XX, TT, >, <, (, ), ])
  - Count: integers 1–99999
  - Concatenate to form input strings with random case
- **Invariant:** `parse(kind + digits).kind == uppercase(kind)` AND `parse(kind + digits).count == digits_as_integer`

### Property 4: Line Command Parser Count Range Validation

**Validates: Requirement 4.7**

- **Statement:** For any valid line command kind followed by a numeric count, the parser SHALL accept counts in [1, 99999] and reject counts exceeding 99999 with an error.
- **Strategy:** Generate:
  - Kind: any valid kind string
  - Count: integers in [0, 200000]
- **Invariant:** `(count >= 1 && count <= 99999) → parse succeeds with that count` AND `(count > 99999) → parse returns error`

### Property 5: Scope Resolution Priority Ordering

**Validates: Requirement 2.1, 2.9**

- **Statement:** When multiple scope sources are present simultaneously, the resolver SHALL always select the highest-priority source. The result is deterministic and independent of the order in which sources are presented.
- **Strategy:** Generate:
  - Sets of 2–4 scope sources drawn from: ExplicitRange, BlockSource, SingleLineCommand, Tagged, Visibility, CursorLine, EntireDocument
  - Randomize presentation order
- **Invariant:** The resolved scope's source always equals the highest-priority source in the set, regardless of presentation order

### Property 6: Status Message Length Invariant

**Validates: Requirement 5.4**

- **Statement:** For any error condition (syntax error, structure error, runtime error) with any command name and description text of any length, the resulting StatusMessage SHALL be at most 200 characters, with truncation indicated by trailing "...".
- **Strategy:** Generate:
  - Command names: strings 1–50 characters
  - Error descriptions: strings 1–500 characters
  - Severity: randomly selected from SyntaxError, StructureError, RuntimeError
- **Invariant:** `status_message.text.len() <= 200` AND `(original_would_exceed_200 → status_message.text.ends_with("..."))`

### Property 7: Configuration Clamping

**Validates: Requirement 6.2, 6.6**

- **Statement:** For the `commands.default_shift_width` configuration key, any integer value SHALL be clamped to [1, 72]. Values within range are unchanged; values outside are clamped to the nearest bound.
- **Strategy:** Generate:
  - Input values: integers in [-1000, 1000]
- **Invariant:** `effective_value ∈ [1, 72]` AND `(input ∈ [1, 72] → effective_value == input)` AND `(input < 1 → effective_value == 1)` AND `(input > 72 → effective_value == 72)`

### Property 8: Empty Input Detection

**Validates: Requirement 3.5, 4.6**

- **Statement:** For any input consisting entirely of whitespace characters (spaces, tabs, carriage returns, newlines) or the empty string, both the primary command parser and line command parser SHALL return `None` indicating no command/line-command is present.
- **Strategy:** Generate:
  - Whitespace-only strings: combinations of ' ', '\t', '\r', '\n' with lengths 0–50
- **Invariant:** `parse_primary(whitespace_input) == None` AND `parse_line_command(whitespace_input) == None`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Parsers", "tasks": ["2", "3", "4", "5"], "dependsOn": [0] },
    { "id": 2, "label": "Scope and Session", "tasks": ["6", "7", "8"], "dependsOn": [1] },
    { "id": 3, "label": "Engine Core", "tasks": ["9", "10", "11"], "dependsOn": [2] },
    { "id": 4, "label": "Error and Config", "tasks": ["12", "13", "14"], "dependsOn": [3] },
    { "id": 5, "label": "HELP Command", "tasks": ["15"], "dependsOn": [4] },
    { "id": 6, "label": "Integration", "tasks": ["16", "17"], "dependsOn": [5] },
    { "id": 7, "label": "Property-Based Tests", "tasks": ["18"], "dependsOn": [6] }
  ]
}
```

---

## Notes

- This is a Wave 5 (Command Engine) crate depending on `ff-command` (Wave 2), `ff-document-model` (Wave 4), `ff-undo-redo` (Wave 4), `ff-logging` (Wave 0), and `ff-configuration` (Wave 2)
- The `find-and-replace`, `line-commands`, `exclude-show-filter`, and `navigation-commands` crates (all Wave 5) consume the parsing and scope resolution APIs defined here
- The primary command parser is the tokenization layer only — actual command implementations (FIND, CHANGE, LOCATE, etc.) live in their respective crates
- The line command parser defines the parse logic; line command execution, block pairing, and pending-state management are in the `line-commands` crate
- Scope resolution integrates with `ff-document-model` for line visibility state queries and with `exclude-show-filter` for visibility modifiers
- The HELP command is registered with Command_ID `"help.show"` and is accessible from both the command line and the command dispatch system
- Configuration keys are namespaced under `[commands]` in the TOML configuration file
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Thread safety relies on `std::sync::RwLock` and `std::sync::Arc` for shared state
- All error messages follow the cross-cutting format requirement: `[subsystem] operation: description` and are at most 200 characters

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Command Execution Pipeline | AC 1.1 | Task 9 |
| Req 1: Command Execution Pipeline | AC 1.2 | Tasks 8, 9 |
| Req 1: Command Execution Pipeline | AC 1.3 | Task 9 |
| Req 1: Command Execution Pipeline | AC 1.4 | Task 9 |
| Req 1: Command Execution Pipeline | AC 1.5 | Tasks 8, 10 |
| Req 1: Command Execution Pipeline | AC 1.6 | Tasks 8, 10 |
| Req 1: Command Execution Pipeline | AC 1.7 | Task 10 |
| Req 1: Command Execution Pipeline | AC 1.8 | Task 11 |
| Req 1: Command Execution Pipeline | AC 1.9 | Task 11 |
| Req 2: Scope Resolution | AC 2.1 | Task 6 |
| Req 2: Scope Resolution | AC 2.2 | Task 7 |
| Req 2: Scope Resolution | AC 2.3 | Task 7 |
| Req 2: Scope Resolution | AC 2.4 | Task 7 |
| Req 2: Scope Resolution | AC 2.5 | Task 7 |
| Req 2: Scope Resolution | AC 2.6 | Task 7 |
| Req 2: Scope Resolution | AC 2.7 | Task 7 |
| Req 2: Scope Resolution | AC 2.8 | Task 7 |
| Req 2: Scope Resolution | AC 2.9 | Task 7 |
| Req 3: Primary Command Parser | AC 3.1 | Task 2 |
| Req 3: Primary Command Parser | AC 3.2 | Task 3 |
| Req 3: Primary Command Parser | AC 3.3 | Task 3 |
| Req 3: Primary Command Parser | AC 3.4 | Task 2 |
| Req 3: Primary Command Parser | AC 3.5 | Task 2 |
| Req 3: Primary Command Parser | AC 3.6 | Task 4 |
| Req 3: Primary Command Parser | AC 3.7 | Task 3 |
| Req 3: Primary Command Parser | AC 3.8 | Task 3 |
| Req 4: Line Command Parser | AC 4.1 | Task 5 |
| Req 4: Line Command Parser | AC 4.2 | Task 5 |
| Req 4: Line Command Parser | AC 4.3 | Task 5 |
| Req 4: Line Command Parser | AC 4.4 | Task 5 |
| Req 4: Line Command Parser | AC 4.5 | Task 5 |
| Req 4: Line Command Parser | AC 4.6 | Task 5 |
| Req 4: Line Command Parser | AC 4.7 | Task 5 |
| Req 5: Error Handling | AC 5.1 | Task 12 |
| Req 5: Error Handling | AC 5.2 | Task 12 |
| Req 5: Error Handling | AC 5.3 | Task 12 |
| Req 5: Error Handling | AC 5.4 | Task 12 |
| Req 5: Error Handling | AC 5.5 | Task 12 |
| Req 5: Error Handling | AC 5.6 | Task 12 |
| Req 5: Error Handling | AC 5.7 | Task 12 |
| Req 6: Configuration Options | AC 6.1 | Task 13 |
| Req 6: Configuration Options | AC 6.2 | Task 13 |
| Req 6: Configuration Options | AC 6.3 | Task 13 |
| Req 6: Configuration Options | AC 6.4 | Task 14 |
| Req 6: Configuration Options | AC 6.5 | Task 14 |
| Req 6: Configuration Options | AC 6.6 | Task 13 |
| Req 7: HELP Command | AC 7.1 | Task 15 |
| Req 7: HELP Command | AC 7.2 | Task 15 |
| Req 7: HELP Command | AC 7.3 | Task 15 |
| Req 7: HELP Command | AC 7.4 | Task 15 |
| Req 7: HELP Command | AC 7.5 | Task 15 |
| Req 7: HELP Command | AC 7.6 | Task 15 |
| Req 7: HELP Command | AC 7.7 | Task 15 |
| Req 7: HELP Command | AC 7.8 | Task 15 |
