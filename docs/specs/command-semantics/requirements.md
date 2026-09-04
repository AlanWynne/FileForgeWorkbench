# Requirements Document

## Introduction

This feature specifies the **Command Semantics Engine** for FileForgeWorkbench (`ff-command-semantics` crate). The command semantics engine is the ISPF-inspired **primary command execution pipeline** — it accepts raw command text from the command line, parses it into structured tokens, resolves the scope of the operation, validates preconditions, builds an execution plan, executes the plan transactionally, and reports results via short status messages.

This crate is **GUI-independent** — it has no rendering or framework dependency. It operates on the abstract document model and integrates with the `command-framework` crate for registry, dispatch, and undo/redo wrapping. All commands registered by this crate are discoverable through the global `CommandRegistry` and invocable through the standard `Command_Dispatch` interface.

The command semantics engine is responsible for:

1. **Primary command parsing** — tokenising the command line into a command name and typed arguments
2. **Line command parsing** — interpreting prefix-area strings into structured line command descriptors
3. **Scope resolution** — determining which lines/region a command targets using a defined priority order
4. **Execution pipeline** — the orchestrated sequence from collection through execution to status emission
5. **Error handling** — translating failures into concise, informative status messages
6. **Configuration** — runtime-configurable behaviours for find scope, bounds, case sensitivity, and invalid command policy
7. **HELP command** — context-sensitive online help for commands, line commands, and macro API

### Design Principles

1. **GUI-independent.** This crate has no GUI dependency; it provides pure command parsing, resolution, and execution logic. [WB]
2. **All commands route through the command-framework.** Every command defined here registers via the `CommandRegistry` and executes through `Command_Dispatch`. [WB]
3. **Transactional execution.** Every mutating command is wrapped in an undo transaction. If execution fails mid-way, the transaction is rolled back — no partial state. [FFE-CMD-1]
4. **Composable with line commands.** Primary commands interact with pending line commands; the execution pipeline handles sequencing and clearing. [FFE-CMD-1]
5. **Concise error reporting.** All errors produce short (≤200 character) human-readable status messages that identify the failing command. [FFE-CMD-38]
6. **Extensible via registration.** New commands can be registered at runtime (e.g., by plugins or macros) through the command-framework's `register()` API. [FFE-CMD-1]

### Source References

- **[FFE-CMD-1]** = FileForgeEditor `core-command-semantics` Requirement 1: Command Execution Pipeline
- **[FFE-CMD-2]** = FileForgeEditor `core-command-semantics` Requirement 2: Scope Resolution
- **[FFE-CMD-36]** = FileForgeEditor `core-command-semantics` Requirement 36: Primary Command Parser
- **[FFE-CMD-37]** = FileForgeEditor `core-command-semantics` Requirement 37: Line Command Parser
- **[FFE-CMD-38]** = FileForgeEditor `core-command-semantics` Requirement 38: Error Handling
- **[FFE-CMD-39]** = FileForgeEditor `core-command-semantics` Requirement 39: Configuration Options
- **[FFE-CMD-40]** = FileForgeEditor `core-command-semantics` Requirement 40: HELP Command
- **[WB]** = Workbench Architecture Brief — GUI-independent, command-driven architecture, undo integration

### Cross-References

- **`command-framework`** — Provides `CommandRegistry` for registration/dispatch, `Command_Dispatch` for execution routing, and undo/redo integration. All commands defined in this crate register via that framework.
- **`undo-redo-transactions`** — Every mutating command execution is wrapped in a transaction. Failure triggers rollback.
- **`find-and-replace`** — FIND/CHANGE/RFIND/RCHANGE command implementations (separate spec); this crate provides the parsing and scope resolution they depend on.
- **`line-commands`** — Line command definitions, block pairing, and pending-state management (separate spec); this crate provides the line command parser.
- **`exclude-show-filter`** — EXCLUDE/SHOW/RESET implementations (separate spec); this crate provides scope resolution with VISIBLE/EXCLUDED/ALL modifiers.
- **`navigation-commands`** — LOCATE, SORT, COLS, BOUNDS implementations (separate spec); this crate provides parsing and scope resolution they depend on.
- **`configuration-system`** — Provides the configuration keys for runtime-configurable behaviours (find scope, bounds, case sensitivity, etc.).

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Command_Engine** | The top-level orchestrator that accepts command-line text and pending line commands, then drives the full execution pipeline. | [FFE-CMD-1] |
| **Primary_Command** | A textual command entered on the command line (as opposed to a line command entered in the prefix area). | [FFE-CMD-1], [FFE-CMD-36] |
| **Line_Command** | A command entered in the prefix area of a line (e.g., `C`, `CC`, `M5`, `D`, `DD`). Consists of a kind and optional repeat count. | [FFE-CMD-37] |
| **Command_Token** | A single lexical unit from the command line: a bare word, a quoted string, or a hex literal (`X'...'`). | [FFE-CMD-36] |
| **Execution_Plan** | The validated, ordered sequence of operations to be executed for a single command invocation, ready for transactional execution. | [FFE-CMD-1] |
| **Scope** | The set of lines (or column range within lines) that a command targets. Resolved through a priority-ordered algorithm. | [FFE-CMD-2] |
| **Block_Source** | A scope derived from a block line command pair (CC/CC, MM/MM, DD/DD, RR/RR, XX/XX, TT/TT) identifying a contiguous range of lines. | [FFE-CMD-2] |
| **Visibility_Modifier** | A keyword (VISIBLE, EXCLUDED, ALL) that restricts scope to lines with specific visibility states. | [FFE-CMD-2] |
| **TAGGED_Modifier** | A keyword (TAGGED, NONTAGGED) that restricts scope to lines that have (or have not) been tagged by a previous operation. | [FFE-CMD-2] |
| **Bounds** | Column boundaries (left bound, right bound) that restrict column-sensitive operations to a sub-range of each line. | [FFE-CMD-2] |
| **Session_State** | The mutable per-document state maintained by the command engine: pending line commands, last command, last scope, cursor position, tags, and status message. | [FFE-CMD-1] |
| **Status_Message** | A short (≤200 characters) human-readable string displayed after command execution indicating success, failure, or informational status. | [FFE-CMD-38] |
| **Hex_Literal** | A token of the form `X'hh...'` representing binary data as hexadecimal digits. | [FFE-CMD-36] |
| **Command_Normalization** | The process of converting a parsed command name to its canonical form (case-insensitive, abbreviation-resolved). | [FFE-CMD-36] |
| **Invalid_Line_Command_Policy** | A configuration option determining how unrecognised line commands are handled: `reject` (error) or `ignore` (silently discard). | [FFE-CMD-39] |

---

## Requirements

### Requirement 1: Command Execution Pipeline

**User Story:** As an editor user, I want the command engine to process my command-line input through a structured pipeline (parse → normalize → resolve scope → validate → execute → report), so that commands execute predictably, transactionally, and with clear feedback on success or failure.

**Source:** [FFE-CMD-1], [WB]

#### Acceptance Criteria

1.1. WHEN the user submits command-line text, THE Command_Engine SHALL execute the following ordered pipeline steps: (1) collect pending line commands from Session_State, (2) parse the primary command text into tokens, (3) normalize the command name (case-fold, resolve abbreviations), (4) resolve scope using the priority algorithm (Requirement 2), (5) validate compatibility between the command and the resolved scope, (6) build an Execution_Plan, (7) execute the plan within an undo transaction, (8) update Session_State with the result, (9) clear consumed line commands from Session_State, (10) emit a Status_Message. [FFE-CMD-1]

1.2. WHEN the command line is empty AND there are pending line commands in Session_State, THE Command_Engine SHALL execute those pending line commands as if the user had pressed Enter to confirm them. [FFE-CMD-1]

1.3. WHEN the command line is empty AND there are no pending line commands in Session_State, THE Command_Engine SHALL emit the Status_Message "No command". [FFE-CMD-1]

1.4. WHEN the command line contains text that does not match any registered command name (after normalization), THE Command_Engine SHALL emit an error Status_Message identifying the unrecognised text and SHALL NOT modify document state. [FFE-CMD-1]

1.5. WHEN a command executes successfully, THE Command_Engine SHALL clear all line commands that were consumed by the execution from Session_State. [FFE-CMD-1]

1.6. WHEN a command execution fails (handler returns an error), THE Command_Engine SHALL retain all pending line commands in Session_State so that the user can correct the error and retry. [FFE-CMD-1]

1.7. THE Command_Engine SHALL wrap every mutating command execution in an undo transaction via the `undo-redo-transactions` system; IF the command fails mid-execution, THEN the transaction SHALL be rolled back and no partial state change SHALL persist. [FFE-CMD-1], [WB]

1.8. THE Command_Engine SHALL support runtime registration of new commands via the command-framework's `register()` API, allowing plugins and macros to extend the command set without recompilation. [FFE-CMD-1], [WB]

1.9. ALL commands registered by the Command_Engine SHALL be accessible through the `command-framework` Command_Dispatch interface, ensuring consistent invocation from keyboard shortcuts, menus, command line, macros, and plugins. [WB]

---

### Requirement 2: Scope Resolution

**User Story:** As an editor user, I want commands to automatically determine their target lines using a well-defined priority order, so that I can use line commands, explicit ranges, visibility modifiers, or cursor position to control scope without ambiguity.

**Source:** [FFE-CMD-2]

#### Acceptance Criteria

2.1. WHEN resolving scope for a command, THE Command_Engine SHALL apply the following priority order (highest to lowest): (1) explicit line range specified in the command arguments, (2) block source from a paired block line command (CC/CC, MM/MM, DD/DD, RR/RR, XX/XX, TT/TT), (3) single source line command, (4) TAGGED modifier, (5) visibility modifier (VISIBLE, EXCLUDED, ALL), (6) cursor line, (7) entire document (when the command allows whole-document scope). [FFE-CMD-2]

2.2. WHEN the ALL modifier is specified, THE resolved scope SHALL include all lines in the document regardless of their visibility state (both visible and excluded lines). [FFE-CMD-2]

2.3. WHEN the VISIBLE modifier is specified, THE resolved scope SHALL include only lines whose visibility state is visible (not excluded). [FFE-CMD-2]

2.4. WHEN the EXCLUDED modifier is specified, THE resolved scope SHALL include only lines whose visibility state is excluded (hidden). [FFE-CMD-2]

2.5. WHEN the TAGGED modifier is specified, THE resolved scope SHALL include only lines that have been tagged by a prior tagging operation. [FFE-CMD-2]

2.6. WHEN the NONTAGGED modifier is specified, THE resolved scope SHALL include only lines that have NOT been tagged by a prior tagging operation. [FFE-CMD-2]

2.7. WHEN column-sensitive operations are active AND Bounds have been set, THE resolved scope SHALL restrict operations to the column range defined by the left and right bounds within each targeted line. [FFE-CMD-2]

2.8. IF no scope can be resolved from any of the priority levels AND the command does not allow default-to-entire-document, THEN THE Command_Engine SHALL emit an error Status_Message indicating that no valid scope was found. [FFE-CMD-2]

2.9. WHEN multiple scope sources conflict (e.g., both explicit range and block source are present), THE higher-priority source SHALL take precedence and the lower-priority source SHALL be ignored without error. [FFE-CMD-2]

---

### Requirement 3: Primary Command Parser

**User Story:** As an editor user, I want the command parser to handle quoted strings, hex literals, and case-insensitive command names, so that I can express complex arguments naturally and not worry about letter case.

**Source:** [FFE-CMD-36]

#### Acceptance Criteria

3.1. WHEN the command line text is submitted, THE Primary_Command_Parser SHALL tokenise it into a command name (first token) followed by zero or more argument tokens, separated by whitespace. [FFE-CMD-36]

3.2. WHEN a token is enclosed in matching single quotes (`'...'`) or double quotes (`"..."`), THE parser SHALL treat the entire enclosed content (including whitespace) as a single Command_Token with the quotes stripped. [FFE-CMD-36]

3.3. WHEN a token matches the pattern `X'hh...'` (case-insensitive X, followed by single-quoted pairs of hexadecimal digits), THE parser SHALL interpret it as a Hex_Literal containing the decoded byte sequence. [FFE-CMD-36]

3.4. THE Primary_Command_Parser SHALL normalize the command name to uppercase before lookup, providing case-insensitive command recognition (e.g., `find`, `Find`, `FIND` all resolve to the same command). [FFE-CMD-36]

3.5. WHEN the command line is empty (zero-length or whitespace-only after trimming), THE parser SHALL return a `None` result indicating no command was entered. [FFE-CMD-36]

3.6. THE Primary_Command_Parser SHALL satisfy the round-trip property: for any valid command line input, parsing and then reconstructing the text from the parsed tokens SHALL produce output that, when re-parsed, yields the same token sequence. [FFE-CMD-36]

3.7. THE Primary_Command_Parser SHALL handle escape sequences within quoted strings: a doubled quote character within a quoted string (e.g., `'it''s'`) SHALL represent a single literal quote character in the token value. [FFE-CMD-36]

3.8. IF the command line contains an unclosed quoted string (opening quote with no matching closing quote), THEN THE parser SHALL return a syntax error rather than silently truncating or accepting malformed input. [FFE-CMD-36]

---

### Requirement 4: Line Command Parser

**User Story:** As an editor user, I want the line command parser to reliably interpret prefix-area input into a command kind and optional repeat count, so that line commands like `C`, `C5`, `CC`, `DD`, `M`, `M3` are unambiguous and consistently handled.

**Source:** [FFE-CMD-37]

#### Acceptance Criteria

4.1. WHEN a prefix-area string is submitted, THE Line_Command_Parser SHALL parse it into a kind (the alphabetic prefix) and an optional count (the trailing numeric digits), where the count defaults to 1 if not specified. [FFE-CMD-37]

4.2. THE Line_Command_Parser SHALL normalize the kind to uppercase, providing case-insensitive recognition (e.g., `c`, `C`, `cc`, `CC` are all valid forms). [FFE-CMD-37]

4.3. THE Line_Command_Parser SHALL recognise all defined line command kinds: single-line commands (C, M, D, R, X, I, A, B, O, W, S, T, >, <, (, ), ]) and their block forms (CC, MM, DD, RR, XX, TT). [FFE-CMD-37]

4.4. WHEN the kind portion of a prefix-area string does not match any defined line command, THE parser SHALL produce an `Unknown` variant containing the original text, rather than panicking or silently discarding. [FFE-CMD-37]

4.5. THE Line_Command_Parser SHALL unambiguously distinguish between alphabetic kind characters and numeric count digits: the kind is the maximal leading alphabetic prefix, and the count is the remaining trailing digits (e.g., `M10` → kind=M, count=10; `CC` → kind=CC, count=1). [FFE-CMD-37]

4.6. IF the prefix-area string is empty or whitespace-only, THEN THE parser SHALL return `None` indicating no line command is present. [FFE-CMD-37]

4.7. THE Line_Command_Parser SHALL handle repeat counts up to 99999 without overflow; IF the count exceeds 99999, THEN the parser SHALL produce an error indicating the count is out of range. [FFE-CMD-37]

---

### Requirement 5: Error Handling

**User Story:** As an editor user, I want command errors to be reported as concise, informative status messages that identify what went wrong and which command failed, so that I can quickly understand and correct the problem.

**Source:** [FFE-CMD-38]

#### Acceptance Criteria

5.1. WHEN a syntax error is detected during parsing (unclosed quote, invalid hex literal, malformed token), THE Command_Engine SHALL produce a Status_Message beginning with "Syntax error" that identifies the problematic text and is at most 200 characters long. [FFE-CMD-38]

5.2. WHEN a structural error is detected (block command mismatch — e.g., CC without a matching CC, overlapping blocks), THE Command_Engine SHALL produce a Status_Message beginning with "Structure error" that identifies the conflicting commands and is at most 200 characters long. [FFE-CMD-38]

5.3. WHEN a runtime error occurs during command execution (I/O failure, invalid line range, incompatible scope), THE Command_Engine SHALL produce a Status_Message beginning with "Error" that identifies the command name and describes the failure, at most 200 characters long. [FFE-CMD-38]

5.4. ALL Status_Messages produced by the error handling system SHALL be at most 200 characters in length. Messages that would exceed this limit SHALL be truncated with a trailing ellipsis ("..."). [FFE-CMD-38]

5.5. ALL error Status_Messages SHALL include the command name (or line command text) that caused the error, enabling the user to identify which operation failed. [FFE-CMD-38]

5.6. WHEN a command completes successfully, THE Command_Engine SHALL produce an informational Status_Message confirming the operation (e.g., "CHANGE - 3 occurrences changed", "COPY - 5 lines copied"). Success messages SHALL also be at most 200 characters. [FFE-CMD-38]

5.7. THE error handling system SHALL categorise errors into three severity levels: syntax errors (parsing failures), structural errors (command pairing/sequencing issues), and runtime errors (execution-time failures). Each category SHALL use a distinct message prefix for immediate identification. [FFE-CMD-38]

---

### Requirement 6: Configuration Options

**User Story:** As an editor user, I want configurable behaviours for command execution (default find scope, bounds handling, case sensitivity, line command error policy), so that I can adapt the editor to my preferred workflow through the configuration system.

**Source:** [FFE-CMD-39]

#### Acceptance Criteria

6.1. THE Command_Engine SHALL support the following configuration keys via the `configuration-system`, each with a defined default value:
- `commands.find_default_scope` (string: `"visible"` | `"all"` | `"excluded"`) — default scope for FIND/CHANGE when no explicit scope is specified. Default: `"visible"`. [FFE-CMD-39]
- `commands.bounds_affect_find` (boolean) — whether column bounds restrict FIND/CHANGE search area. Default: `true`. [FFE-CMD-39]
- `commands.case_sensitive_find` (boolean) — whether FIND/CHANGE defaults to case-sensitive matching. Default: `false`. [FFE-CMD-39]
- `commands.default_shift_width` (integer, 1–72) — number of columns for > and < shift line commands when no count is specified. Default: `2`. [FFE-CMD-39]
- `commands.reset_clears_tags` (boolean) — whether the RESET command clears line tags in addition to exclusion state. Default: `false`. [FFE-CMD-39]
- `commands.invalid_line_command_policy` (string: `"reject"` | `"ignore"`) — how unrecognised line commands are handled. Default: `"reject"`. [FFE-CMD-39]

6.2. WHEN a configuration key contains an invalid value (out of range, wrong type, or unknown enum variant), THE Command_Engine SHALL fall back to the defined default value for that key and SHALL write a WARN-level log record indicating the invalid value and the default being used. [FFE-CMD-39]

6.3. THE Command_Engine SHALL read configuration values at startup and SHALL re-read them when the configuration system emits a hot-reload notification, applying new values to subsequent command executions without requiring application restart. [FFE-CMD-39]

6.4. WHEN `commands.invalid_line_command_policy` is `"reject"`, THE Command_Engine SHALL produce an error Status_Message for any unrecognised line command and SHALL NOT execute the command pipeline. [FFE-CMD-39]

6.5. WHEN `commands.invalid_line_command_policy` is `"ignore"`, THE Command_Engine SHALL silently discard unrecognised line commands without error and SHALL proceed with the remainder of the pipeline. [FFE-CMD-39]

6.6. WHEN `commands.default_shift_width` is less than 1 or greater than 72, THE Command_Engine SHALL clamp the value to the nearest valid bound (1 or 72) and write a WARN-level log record. [FFE-CMD-39]

---

### Requirement 8: Command Field Submission

**User Story:** As a user, I want pressing Enter in the Command ===> field to submit the typed command, so that the command is executed immediately without requiring a mouse click.

**Source:** ISPF command field behaviour — Enter submits the command line.

#### Acceptance Criteria

8.1. WHEN the Command ===> field has keyboard focus AND the user presses Enter AND the field is non-empty, THE shell SHALL submit the command text to `handle_command`, clear the field, and execute the command.

8.2. WHEN the Command ===> field is empty AND the user presses Enter, THE shell SHALL NOT submit or execute any command.

---

### Requirement 7: HELP Command

**User Story:** As an editor user, I want a HELP command that provides context-sensitive documentation for commands, line commands, and the macro API, so that I can discover capabilities without leaving the editor.

**Source:** [FFE-CMD-40]

#### Acceptance Criteria

7.1. WHEN `HELP` is entered with no arguments, THE Command_Engine SHALL display a summary of all available primary commands, grouped by category, including their names and one-line descriptions. [FFE-CMD-40]

7.2. WHEN `HELP <commandname>` is entered where `<commandname>` matches a registered command, THE Command_Engine SHALL display the full help text for that command, including syntax, available modifiers, valid arguments, and examples. [FFE-CMD-40]

7.3. WHEN `HELP LINECOMMANDS` is entered, THE Command_Engine SHALL display a summary of all defined line commands with their abbreviations, block forms, and one-line descriptions. [FFE-CMD-40]

7.4. WHEN `HELP MACRO` or `HELP API` is entered, THE Command_Engine SHALL display a summary of the Lua macro API functions available for scripting. [FFE-CMD-40]

7.5. WHEN `HELP <topic>` is entered where `<topic>` does not match any registered command or known help topic, THE Command_Engine SHALL display a message listing available help topics and suggesting close matches (if any). [FFE-CMD-40]

7.6. THE HELP command SHALL be valid in all editor modes (edit mode, view mode, browse mode) and SHALL NOT modify document state. [FFE-CMD-40]

7.7. THE HELP command SHALL NOT be recorded in command history, as it is informational and not a repeatable action. [FFE-CMD-40]

7.8. THE HELP command SHALL be registered in the command-framework with Command_ID `"help.show"` and SHALL be invocable both from the command line (via `HELP` text) and through the command dispatch system. [FFE-CMD-40], [WB]

### Requirement 9: TSO Commands and FTSO Operand Parsing

**User Story:** As a TSO-familiar operator, I want the command engine to support TSO dataset management commands (ALLOCATE, FREE, DELETE, RENAME, LISTCAT, LISTDS, LISTALC, SUBMIT, STATUS), FTSO-style operand parsing, dataset prefix management, and advanced command features (continuation, ds:// URIs, namespace conflict resolution, capability model, secret operands, audit events), so that the workbench provides a complete TSO command environment.

**Source:** EARS integration Phase CB (coverage-classification.md B10)

#### Acceptance Criteria

1. THE command engine SHALL support the `ALLOCATE` command, routing to the dataset allocator subsystem with TSO-style keyword operands (DATASET, SPACE, TRACKS, CYLINDERS, RECFM, LRECL, BLKSIZE, DSORG, UNIT, VOLUME). [TSO-CMD-1]
2. THE command engine SHALL support the `FREE` command, routing to the dataset allocator to release a dataset allocation by name. [TSO-CMD-2]
3. THE command engine SHALL support the `DELETE` command, routing to the VFS/catalog layer to delete a dataset or member by name. [TSO-CMD-3]
4. THE command engine SHALL support the `RENAME` command with syntax `RENAME oldname newname`, routing to the VFS/catalog layer. [TSO-CMD-4]
5. THE command engine SHALL support the `LISTCAT` command, routing to the catalog registry to list catalog entries matching an optional filter pattern. [TSO-CMD-5]
6. THE command engine SHALL support the `LISTDS` command with syntax `LISTDS dsname [MEMBERS]`, routing to the VFS layer to list dataset attributes and optionally its members. [TSO-CMD-6]
7. THE command engine SHALL support the `LISTALC` command, routing to the dataset allocator to list currently allocated datasets. [TSO-CMD-7]
8. THE command engine SHALL support the `SUBMIT` command with syntax `SUBMIT dsname`, routing to the FFW-JES subsystem to submit the named dataset as a batch job. [TSO-CMD-8]
9. THE command engine SHALL support the `STATUS` command with optional jobname argument, routing to the FFW-JES job status panel. [TSO-CMD-9]
10. WHEN the `EDIT` command is issued from the command line with a dataset name argument, THE command engine SHALL route to the file-operations pipeline to open the named dataset in an editor tab, extending the existing EDIT routing criterion. [TSO-EDIT-1]
11. THE command engine SHALL support TSO-style positional and keyword operand parsing: positional operands are space-separated values in defined order; keyword operands are in the form `KEYWORD(value)` or `KEYWORD value`. [FTSO-operand-parse]
12. THE command engine SHALL support a session-level dataset prefix (`SET PREFIX dsn-prefix`): WHEN a prefix is set, unqualified dataset names in commands are automatically qualified by prepending the prefix. [FTSO-prefix]
13. THE command engine SHALL support command continuation using a trailing backslash (`\`): WHEN a command line ends with `\`, THE engine SHALL treat the next submitted line as a continuation of the current command. [FTSO-continuation]
14. THE command engine SHALL support the `ds://` URI scheme for dataset references: WHEN a command argument begins with `ds://`, THE engine SHALL resolve it through the VFS catalog layer without applying the session prefix. [FTSO-ds-uri]
15. WHEN two registered commands have the same name (namespace conflict), THE command engine SHALL resolve the conflict using a defined priority order: built-in commands > plugin commands > macro commands; the lower-priority command SHALL be accessible via a qualified name (`plugin:commandname`). [FTSO-ns-conflict]
16. THE command engine SHALL support a capability model: EACH registered command SHALL declare the capabilities it requires (e.g., `dataset.write`, `jes.submit`); WHEN a command is invoked, THE engine SHALL verify the required capabilities are available in the current session context. [FTSO-capability]
17. THE command engine SHALL support secret operand handling: WHEN a command operand is declared as secret (e.g., a password field), THE engine SHALL redact that operand from command history, log records, and status messages. [FTSO-secret]
18. THE command engine SHALL emit structured audit events for every command execution: event SHALL include command name, arguments (with secrets redacted), timestamp, user context, and outcome (success/failure). [FTSO-audit]

### Requirement 10: TSO P2 Commands (OUTPUT, CANCEL, SEND, PROFILE, PRINTDS)

**User Story:** As a TSO-familiar operator, I want the command engine to support the P2 TSO output and communication commands (OUTPUT, CANCEL, SEND, PROFILE, PRINTDS), so that the workbench provides a complete TSO command environment for job output management and user communication.

**Source:** EARS integration Phase CI (coverage-classification.md B16, TSO-CMD-10 through TSO-CMD-14)

#### Acceptance Criteria

10.1. THE command engine SHALL support the `OUTPUT` command with syntax `OUTPUT jobname [options]`, routing to the FFW-JES subsystem to display or retrieve job output for the named job. [TSO-CMD-10]

10.2. THE command engine SHALL support the `CANCEL` command with syntax `CANCEL jobname [PURGE]`, routing to the FFW-JES subsystem to cancel a batch job; WHEN the PURGE operand is specified, THE engine SHALL also request purge of the job's output. [TSO-CMD-11]

10.3. THE command engine SHALL support the `SEND` command with syntax `SEND 'message' [USER(userid) | LOGON | BROADCAST]`, routing to the messaging subsystem to send a message to a user, all logged-on users, or the system broadcast queue. [TSO-CMD-12]

10.4. THE command engine SHALL support the `PROFILE` command with syntax `PROFILE [operands]`, routing to the session profile subsystem to display or update TSO session profile settings (MSGID, INTERCOM, NOINTERCOM, PREFIX, SIZE, WTPMSG). [TSO-CMD-13]

10.5. THE command engine SHALL support the `PRINTDS` command with syntax `PRINTDS DATASET(dsname) [options]`, routing to the file-operations pipeline to print the contents of a dataset to the system printer or a specified output destination. [TSO-CMD-14]
