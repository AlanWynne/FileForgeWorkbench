# Requirements Document

## Introduction

This feature specifies the command framework for FileForgeWorkbench (`ff-command` crate). The command framework is the **central dispatch mechanism** for all user-facing operations in the workbench. It provides a global command registry, a single dispatch entry point, rich command metadata, automatic undo/redo integration, a keyboard shortcut management system, a scripting bridge for Lua macros, and a command history log.

The command-driven architecture (cross-cutting Requirement 4 in the project-master spec) mandates that **all state-changing user operations** are routed through this framework -- whether invoked via keyboard shortcuts, menus, the command line, macros, or plugins. This ensures consistent undo/redo behaviour, shortcut discoverability, macro recordability, and a single audit trail for all user actions.

The `ff-command` crate is a Wave 2 (Platform Architecture) dependency. It depends on `ff-logging` for diagnostics and is consumed by virtually every higher-level crate: `ff-core` (platform-core), `ff-plugin` (plugin-architecture), `ff-workflow` (workflow-engine), all editor subsystems, and the GUI shell.

**Source references:**
- **WB** = Workbench Architecture Brief §7 -- command-driven architecture principle
- **FFE** = FileForgeEditor `core-command-semantics` (ISPF command engine, adapted)
- **SCI** = Scintilla KeyMap and command binding concepts (adapted)

## Glossary

- **Command**: A named, executable operation registered in the command registry. Each command has a unique string identifier, metadata, and an execution handler. [WB]
- **Command_ID**: A unique dot-separated string identifier for a command (e.g., `"file.save"`, `"edit.undo"`, `"find.next"`). [WB]
- **Command_Registry**: The global, thread-safe collection of all registered commands, supporting registration, lookup, and runtime discovery. [WB]
- **Command_Dispatch**: The single entry point through which all command executions are routed, regardless of invocation source. [WB]
- **Command_Params**: A typed key-value map of parameters passed to a command at execution time. [WB]
- **Execution_Context**: The ambient state available to a command during execution -- active document, selection, cursor position, active panel, and (CR-CH-028) the Cursor_Context package. [WB]
- **Cursor_Context**: A per-invocation snapshot of "where the cursor/focus was" when a command was invoked, carried on the Execution_Context (Requirement 12). It has a fixed strongly-typed CORE (focused workspace/context name, focused-control semantic identity, focused-control text where meaningful, cursor line/column, selection, active scroll setting) and an open EXTRAS bag (a string-keyed map of Context_Value) so a workspace MAY attach workspace-specific context a command may consult or ignore. [WB, CR-CH-028]
- **Context_Value**: A typed value stored in the Cursor_Context extras bag (string / integer / boolean / float), mirroring Command_Params value types. [WB, CR-CH-028]
- **Command_Result**: The outcome of a command execution, containing success/failure status, optional return value, and optional undo record. [WB]
- **Command_Metadata**: Descriptive information attached to a command: display name, description, category, default shortcut, icon reference, enabled predicate, visibility predicate. [WB, FFE]
- **Undo_Record**: An opaque token produced by an undoable command during execution, encapsulating the information needed to reverse the command's effect. [WB]
- **Shortcut_Binding**: A mapping from a keyboard chord (or multi-key sequence) to a Command_ID. [WB, SCI]
- **Shortcut_Registry**: The collection of all active keyboard shortcut bindings, supporting conflict detection and user customization. [WB, FFE]
- **Scripting_Bridge**: The interface through which the Lua macro engine invokes commands and receives results. [WB, FFE]
- **Command_History**: A bounded, persistent log of recently executed commands for retrieval and audit. [FFE]
- **Reserved_Shortcut**: A keyboard binding that is globally reserved and cannot be overridden by user configuration or plugins (per cross-cutting Requirement 10). [FFE]

## Requirements

### Requirement 1: Command Registry

**User Story:** As a workbench developer, I want a global registry of all available commands, so that any subsystem can register commands at startup and any other subsystem can discover and invoke them by ID.

**Source:** WB Architecture Brief §7 -- command registry. [WB]

#### Acceptance Criteria

1. THE Command_Registry SHALL store commands indexed by their Command_ID, where each Command_ID is a non-empty UTF-8 string containing only lowercase ASCII letters, digits, dots, and underscores, with dot used as a namespace separator (e.g., `"file.save"`, `"edit.undo"`, `"view.zoom_in"`).
2. WHEN a command is registered with a Command_ID that already exists in the registry, THE Command_Registry SHALL reject the registration and return an error indicating the duplicate ID, without modifying the existing registration.
3. THE Command_Registry SHALL support registration by platform-core subsystems during application startup and by plugins during their `initialize` lifecycle phase.
4. THE Command_Registry SHALL be safe to read and write from any thread without requiring the caller to acquire an external lock (thread-safe registration and lookup).
5. WHEN a lookup is performed for a Command_ID that does not exist in the registry, THE Command_Registry SHALL return a `None` result without panicking.
6. THE Command_Registry SHALL support runtime discovery: listing all registered commands, and querying commands by category prefix (e.g., all commands whose ID starts with `"file."`).
7. THE Command_Registry SHALL support deregistration of commands by ID, allowing plugins to cleanly remove their commands during the `shutdown` lifecycle phase.

---

### Requirement 2: Command Dispatch

**User Story:** As a workbench developer, I want a single dispatch entry point for executing commands, so that all input sources (keyboard, menu, command line, macro, plugin) use the same execution path with consistent validation, context injection, and error handling.

**Source:** WB Architecture Brief §7 -- single dispatch entry point. [WB]

#### Acceptance Criteria

1. THE Command_Dispatch SHALL provide a single entry point `execute_command(id: &str, params: CommandParams) → CommandResult` through which all command invocations are routed.
2. WHEN `execute_command` is called with a Command_ID that is not registered, THE Command_Dispatch SHALL return an error result containing the unrecognized command ID, without panicking or modifying application state.
3. WHEN `execute_command` is called, THE Command_Dispatch SHALL construct an Execution_Context containing the currently active document (if any), the current selection/cursor position, the active panel identifier, and (CR-CH-028) the Cursor_Context package (Requirement 12), and SHALL pass this context to the command handler.
4. THE Command_Dispatch SHALL support both synchronous command execution (blocking the caller until the command completes) and asynchronous command execution (returning a future that resolves when the command completes).
5. THE Command_Dispatch SHALL validate that the command's enabled predicate returns true before executing the command; IF the command is disabled, THEN THE Command_Dispatch SHALL return an error result indicating the command is not currently available, without invoking the command handler.
6. WHEN a command handler returns an error, THE Command_Dispatch SHALL propagate the error to the caller as a `CommandResult::Err` and SHALL write a WARN-level log record containing the Command_ID and error description via the logging subsystem.
7. ALL user-facing operations that modify application state SHALL be invoked through `execute_command`; no UI code SHALL directly mutate application state without routing through the command framework.
8. THE Command_Dispatch SHALL accept Command_Params as a typed key-value map supporting string, integer, float, boolean, and nested map value types.

---

### Requirement 3: Command Metadata

**User Story:** As a workbench developer, I want rich metadata attached to each command, so that menus, keybinding UI, help systems, and command palettes can present commands with display names, descriptions, icons, and availability information without hardcoding knowledge of specific commands.

**Source:** WB Architecture Brief §7 -- command metadata for runtime inspection. [WB, FFE]

#### Acceptance Criteria

1. EACH registered command SHALL have associated metadata containing: a display name (human-readable, localizable string), a description (one-sentence summary of what the command does), and a category (dot-separated namespace matching the Command_ID prefix, e.g., `"file"`, `"edit"`, `"view"`).
2. EACH registered command SHALL optionally have a default keyboard shortcut binding specified in its metadata; IF no shortcut is specified, THEN the command has no default binding.
3. EACH registered command SHALL optionally have an icon reference (a string identifier referencing an icon asset) for display in menus, toolbars, and command palettes.
4. EACH registered command SHALL have an enabled predicate -- a function that, given the current Execution_Context, returns a boolean indicating whether the command can currently execute. IF no predicate is provided, THEN the command SHALL be considered always enabled.
5. EACH registered command SHALL have a visibility predicate -- a function that, given the current Execution_Context, returns a boolean indicating whether the command should appear in menus and command palettes. IF no predicate is provided, THEN the command SHALL be considered always visible.
6. THE Command_Registry SHALL provide a method to query the metadata for any registered command by Command_ID, returning all metadata fields without executing the command.
7. WHEN the enabled or visibility predicate for a command is evaluated, THE evaluation SHALL NOT produce side effects and SHALL complete within 1 millisecond to avoid blocking UI rendering.

---

### Requirement 4: Undo/Redo Integration

**User Story:** As a workbench developer, I want the command framework to automatically integrate with the undo/redo system, so that every undoable command produces an undo record as part of its execution without requiring each command to manually manage the undo stack.

**Source:** WB Architecture Brief §7 -- undo/redo integration. Cross-references `undo-redo-transactions` crate. [WB]

#### Acceptance Criteria

1. EACH registered command SHALL declare whether it is undoable (produces an Undo_Record) or non-undoable (view changes, settings modifications, navigation) at registration time.
2. WHEN an undoable command executes successfully, THE command handler SHALL return an Undo_Record as part of its CommandResult, and THE Command_Dispatch SHALL automatically push that Undo_Record onto the active undo stack for the relevant document or context.
3. WHEN a non-undoable command executes, THE Command_Dispatch SHALL NOT push any record to the undo stack and SHALL NOT clear or modify the existing undo/redo history.
4. THE combination of command execution and undo record creation SHALL be atomic: IF the command handler returns an error, THEN no Undo_Record SHALL be pushed to the undo stack, and application state SHALL remain unchanged (no partial state).
5. WHEN the built-in `"edit.undo"` command is executed, THE Command_Dispatch SHALL pop the most recent Undo_Record from the active undo stack and apply it to reverse the effect of the original command, moving the record to the redo stack.
6. WHEN the built-in `"edit.redo"` command is executed, THE Command_Dispatch SHALL pop the most recent record from the redo stack and re-apply the command, moving the record back to the undo stack.
7. WHEN an undoable command is executed after one or more undo operations, THE Command_Dispatch SHALL clear the redo stack for the active context (standard undo semantics -- executing a new command invalidates the redo history).

---

### Requirement 5: Keyboard Shortcut Management

**User Story:** As a user, I want a keyboard shortcut system that prevents conflicts, supports user customization, and handles multi-key sequences, so that I can efficiently invoke commands without memorizing arbitrary bindings or encountering unexpected behaviour.

**Source:** WB Architecture Brief §7, Cross-cutting Requirement 10 (keyboard shortcut registry). [WB, FFE, SCI]

#### Acceptance Criteria

1. THE Shortcut_Registry SHALL maintain a mapping from keyboard chords to Command_IDs, where a chord is defined as a combination of zero or more modifier keys (Ctrl, Alt, Shift, Super/Win) plus a primary key.
   *(Phase DB, CR-NR-051: a chord's binding target is a Command_Target (Requirement 8). A bare Command_ID is the `Function` variant of a Command_Target, so this criterion is the Command_ID special case of the general rule in Requirement 8.5; a chord MAY equally bind to a Menu, CustomWorkspace, Macro, or External target.)*
2. THE Shortcut_Registry SHALL support multi-key sequences (e.g., Ctrl+K followed by Ctrl+C), where the first chord enters a pending state and the framework waits for the second chord to complete the binding or times out after 2 seconds (reverting to no pending state).
3. THE following shortcuts SHALL be reserved globally and SHALL NOT be overridden by user configuration, plugins, or any sub-project registration: F1 (Help), Ctrl+Plus/Ctrl+Minus/Ctrl+0 (Zoom), Ctrl+Z/Ctrl+Y/Ctrl+Shift+Z (Undo/Redo), Ctrl+C/Ctrl+X/Ctrl+V/Ctrl+A (Clipboard), Ctrl+S (Save), Ctrl+F (Find), Ctrl+H (Change), Ctrl+G (Go to line), Ctrl+Tab/Ctrl+Shift+Tab (Tab switch), Ctrl+W (Close tab), Ctrl+N (New tab), Ctrl+Shift+D (Dock/undock), Ctrl+Shift+T (Undock/redock tab).
4. WHEN a shortcut binding is registered that conflicts with an existing binding (same chord sequence already mapped to a different Command_ID), THE Shortcut_Registry SHALL reject the registration and return an error indicating the conflict, identifying both the new and existing Command_IDs.
5. WHEN a shortcut binding is registered that conflicts with a reserved shortcut, THE Shortcut_Registry SHALL reject the registration and return an error indicating that the shortcut is reserved and cannot be overridden.
6. THE Shortcut_Registry SHALL support user-configurable shortcut overrides for all non-reserved commands, loaded from the workbench configuration system (TOML-based key map file).
7. WHEN a keyboard chord is received that matches a registered shortcut, THE Shortcut_Registry SHALL resolve it to the bound Command_Target and execute it (for a `Function` target this is `execute_command` through the Command_Dispatch; other variants route via `execute_target`, Requirement 8.2). A binding stored as a bare Command_ID is treated as a `Function` target.
8. FUNCTION keys F2–F24 SHALL be user-configurable via the key map system, and plugins SHALL be able to register shortcut bindings for their commands through the Shortcut_Registry (subject to conflict detection and reserved shortcut rules).

---

### Requirement 6: Scripting Bridge

**User Story:** As a macro developer, I want to invoke any registered command from a Lua script and receive structured results, so that macros can automate workflows by composing commands without reimplementing their logic.

**Source:** WB Architecture Brief §7 -- scripting bridge. Cross-references `lua-macro-engine` crate. [WB, FFE]

#### Acceptance Criteria

1. THE Scripting_Bridge SHALL expose all registered commands to the Lua macro engine, allowing scripts to invoke commands by their Command_ID using a function call syntax (e.g., `workbench.execute("file.save", {path = "/tmp/out.txt"})`).
2. THE Scripting_Bridge SHALL convert Lua table parameters to Command_Params and pass them to `execute_command` through the standard Command_Dispatch path.
3. THE Scripting_Bridge SHALL convert the CommandResult back to a Lua-compatible return value: success results are returned as Lua values (strings, numbers, booleans, tables), and error results raise a Lua error with the error description string.
4. THE Scripting_Bridge SHALL support batch execution: a Lua script may invoke multiple commands in sequence, and each command is dispatched independently through the Command_Dispatch (with individual undo records per command).
5. WHEN a command invoked from a script fails, THE Scripting_Bridge SHALL propagate the error to the Lua runtime as a catchable Lua error, allowing the script to handle or re-raise it.
6. THE Scripting_Bridge SHALL provide a query function (e.g., `workbench.commands()`) that returns a Lua table listing all registered Command_IDs and their metadata (display name, category, description), enabling scripts to discover available commands at runtime.

---

### Requirement 7: Command History

**User Story:** As a user, I want a record of recently executed commands, so that I can recall previous actions (via RETRIEVE or a history panel), audit what was done in a session, and restore history across application restarts.

**Source:** FFE `function-keys-and-command-history` -- RETRIEVE command history. [FFE]

#### Acceptance Criteria

1. THE Command_History SHALL record every successfully executed command invocation, storing the Command_ID, a timestamp (UTC, millisecond precision), and the Command_Params that were passed.
2. THE Command_History SHALL have a configurable maximum depth (number of entries retained), specified via the workbench configuration system under `commands.history_depth`, with a default of 500 entries.
3. IF the `commands.history_depth` setting contains a value less than 10 or greater than 10000, THEN THE Command_History SHALL clamp the value to the nearest bound (10 or 10000) and write a WARN-level log record indicating the adjustment.
4. WHEN the history reaches its maximum depth and a new entry is recorded, THE Command_History SHALL discard the oldest entry to make room for the new one (FIFO eviction).
5. THE Command_History SHALL be persistent across application sessions: WHEN the application shuts down normally, THE Command_History SHALL serialize its entries to a file in the workbench data directory; WHEN the application starts, THE Command_History SHALL load the persisted entries and resume from where it left off.
6. IF the history persistence file cannot be read at startup (corrupted, missing, or permission error), THEN THE Command_History SHALL start with an empty history and write a WARN-level log record indicating the reason.
7. THE Command_History SHALL be safe to read and write from any thread without requiring the caller to acquire an external lock (thread-safe access).
8. THE Command_History SHALL provide a query interface: retrieve the last N entries, retrieve entries matching a Command_ID prefix, and retrieve entries within a time range.

---

### Requirement 8: Unified Command Target

**User Story:** As a user and as a plugin author, I want a single way to describe "what a command does" -- open a menu, open a built-in workspace, run an internal function, run a macro, or run an external program -- so that menu options and keyboard bindings can point at any of these targets through one consistent mechanism.

**Source:** [CR-NR-051], [WB]

#### Glossary additions

- **Command_Target**: A typed description of the action a command performs. Exactly one of five variants: Menu_Target, Custom_Workspace_Target, Function_Target, Macro_Target, or External_Target.
- **Target_Resolution**: The process of converting a bare command string (as typed in a `Command ===>` field or written in a Menu_File `command` value) into a Command_Target.
- **Visible_Workspace**: A Command_Target whose execution results in an open, user-visible Workspace (Menu_Target, Custom_Workspace_Target, and External_Target in Captured mode). Distinguished from Started_Tasks and pure side-effect functions, which produce no persisted Workspace.

#### Acceptance Criteria

1. THE command framework SHALL define a Command_Target type with exactly five variants:
   - `Menu_Target` -- carries a Menu_Name; opens the Menu_Workspace backed by `menus/<name>.toml`.
   - `Custom_Workspace_Target` -- carries a Workspace_Kind and an optional typed parameter map (Params); opens the corresponding built-in Context (e.g. Editor, Files, Settings, Search).
   - `Function_Target` -- carries a Command_ID and optional Command_Params; invokes a registered internal command via `execute_command` (Requirement 2).
   - `Macro_Target` -- carries either a macro Name or an absolute/workspace-relative Path; runs it through the existing `macro.run_named` / `macro.run_file` dispatch (lua-macro-engine Requirement 5).
   - `External_Target` -- carries a Program, an argument list, an optional Working_Directory, and an Execution_Mode (Detached or Captured); runs an external process (defined in command-configurator Requirement 3 and shell-command Requirement 19).
2. WHEN a Command_Target is executed, THE Command_Dispatch SHALL route it to the handling path for its variant and SHALL return a Command_Result consistent with Requirement 2.
3. WHEN a bare command string is submitted (typed in a `Command ===>` field or read from a Menu_File `command` value), THE framework SHALL perform Target_Resolution through ONE explicit, ordered precedence chain (first match wins; matching is case-insensitive), REVISED by CR-CH-025 to make the ordering authoritative and to add the menu-name and macro stages:
   1. **Current-menu Option_Key** -- WHEN the active Workspace is a Menu_Workspace AND the trimmed string matches an Option_Key of the CURRENTLY-displayed menu, THE framework SHALL activate that option (menu-workspace Requirement 3.1). This stage applies only on a Menu_Workspace and is context-local.
   2. **Built-in command / Command_ID** -- a string that matches a registered Command_ID resolves to a Function_Target; a string that matches a built-in workspace verb or fastpath resolves to a Custom_Workspace_Target; a string that matches a user-defined command definition (command-configurator Requirement 1) resolves to that definition's Command_Target.
   3. **Menu name** -- a string whose first token matches a resolvable Menu_Name (a user `menus/<name>.toml` that exists, or a compiled built-in menu name such as `POM` or `SETTINGS`) resolves to a Menu_Target for that menu; any trailing token is forwarded as an Option_Key of the opened menu (Requirement 9; menu-workspace Requirement 11.7).
   4. **Macro** -- a string whose first token matches a macro in the Macro_Library (by name) resolves to a Macro_Target. (SPECIFIED by CR-CH-025; the macro-run wiring is DEFERRED -- see criterion 11 -- so until Lua execution is available this stage does not match and resolution proceeds to the error stage.)
   5. **Unresolved** -- otherwise, resolution fails per criterion 8 (an error naming the unresolved string; no state mutation).
4. THE introduction of Command_Target SHALL NOT change the behaviour of any existing command string: every command string that resolves today SHALL resolve to an equivalent Command_Target and produce the same observable result (backward compatibility).
5. A keyboard Shortcut_Binding (Requirement 5) SHALL be bindable to any Command_Target, not only to a Command_ID, so that a function key or chord may open a menu, open a custom workspace, run a macro, or run an external program.
6. A Menu_Option (menu-workspace Requirement 1) SHALL resolve its `command` value to a Command_Target via Target_Resolution, so that a single option may target any of the five variants.
7. THE Command_Target type SHALL be serialisable to and deserialisable from TOML, so that user-defined command definitions (command-configurator) and persisted Workspace descriptors (startup-and-session Requirement 21) can store targets in data files.
8. WHEN Target_Resolution fails to resolve a string to any variant, THE framework SHALL return an error result naming the unresolved string, without panicking or mutating application state (consistent with Requirement 2 criterion 2).
9. THE Command_Target SHALL classify each variant as producing a Visible_Workspace or not: Menu_Target, Custom_Workspace_Target, and External_Target in Captured mode are Visible_Workspaces; Function_Target, Macro_Target, and External_Target in Detached mode are not. This classification SHALL be queryable without executing the target (for use by session persistence, Requirement 21 of startup-and-session).
10. **(CR-CH-025 -- Shadowing rule.)** THE resolution chain of criterion 3 SHALL be strictly ordered so that an EARLIER stage always wins over a LATER one: a built-in command / Command_ID (stage 2) SHALL take precedence over a same-named Menu_Name (stage 3), which SHALL take precedence over a same-named Macro (stage 4). A user therefore CANNOT override a built-in command by creating a menu or macro with the same name. Within a single stage, the FIRST match wins. All name matching in the chain SHALL be case-insensitive. This makes the previously implicit ordering authoritative and prevents a user-authored menu/macro from hijacking a core verb.
11. **(CR-CH-025 -- Menu-name resolution stage.)** WHEN the first token of the submitted string matches a resolvable Menu_Name (stage 3 of criterion 3) AND that token is NOT claimed by an earlier stage, THE framework SHALL resolve it to a `Menu_Target { name }` and open (or return to) that Menu_Workspace exactly as the `MENU <name>` command does (Requirement 9; menu-workspace Requirement 11.2). A Menu_Name resolves when EITHER a user file `menus/<name>.toml` exists OR `<name>` is a compiled built-in menu (`POM`, `SETTINGS`). This is what makes `SETTINGS` (and any user menu name) a first-class command with NO hardcoded special case: opening the Settings menu is stage-3 resolution of the token `SETTINGS`, identical to `POM` or a user's `REPORTS` menu.
12. **(CR-CH-025 -- Macro stage; DEFERRED implementation.)** THE macro stage (stage 4 of criterion 3) SHALL be part of the SPECIFIED chain order, positioned after menu-name resolution and before the unresolved-command error. UNTIL the shell gains Lua/macro execution (a separate change; ff-desktop does not yet depend on the macro engine), THE macro stage SHALL match nothing, so a bare token that is neither a built-in nor a menu name SHALL fall through to the unresolved-command error (criterion 8) rather than silently doing nothing. WHEN macro execution is later wired, this stage SHALL resolve a first-token match against the Macro_Library to a `Macro_Target` without any further change to the chain order.
13. **(CR-CH-025 -- Trailing-token forwarding.)** WHEN a stage-3 Menu_Name match carries a trailing token (e.g. `SETTINGS T`), THE framework SHALL open the named menu and immediately activate the option whose Option_Key equals the trailing token (menu-workspace Requirement 11.7), so that `<menu> <key>` is observably identical to opening the menu and typing `<key>`. Deeper chains compose through each activated option's own command (menu-workspace Requirement 11.10).

---

### Requirement 9: Command Arguments

**User Story:** As a user, I want to pass an argument to a command from the
`Command ===>` field (for example `DOWN 8`, `DOWN M`, or `MENU SETTINGS EDITOR`),
so that one command verb can be parameterised at the point of invocation rather
than requiring a separate command per variation.

**Source:** [CR-NR-054], [ISPF] scroll-amount and fastpath conventions. [WB]

#### Acceptance Criteria

1. WHEN a Command_Invocation string is submitted from a `Command ===>` field,
   THE framework SHALL parse it into a Command_Verb (the first whitespace-delimited
   token) and an Argument_String (the remainder of the line after the first run
   of whitespace, with surrounding whitespace trimmed; empty when no argument was
   typed).
2. WHEN a Command_Verb resolves to a registered Command_ID, THE Command_Dispatch
   SHALL place the Argument_String into Command_Params under the reserved key
   `arg` (a string value) before invoking the handler, so a command that accepts
   an argument reads it from `params.arg` (extending Requirement 2.1, 2.8).
3. WHEN no argument was typed, THE Argument_String SHALL be the empty string and
   the `arg` param SHALL be absent, so a command receiving no argument behaves
   exactly as it does today (backward compatibility).
4. THE argument parsing SHALL NOT change the behaviour of any command that does
   not read the `arg` param: a verb-only invocation (e.g. `SAVE`, `CANCEL`)
   SHALL produce the same observable result as before this requirement.
5. A Command_Definition (command-configurator Requirement 1) and a
   Shortcut_Binding (Requirement 5) MAY carry a fixed Argument_String; WHEN such
   a binding is invoked, THE framework SHALL forward that argument to the command
   exactly as if it had been typed after the verb.
6. WHEN a command that does not accept an argument is invoked with a non-empty
   Argument_String, THE command SHALL ignore the surplus argument and SHALL NOT
   error solely because an argument was present (a command MAY choose to validate
   its own argument and return a Command_Result error for a malformed value).
7. Argument parsing SHALL be performed once, at the dispatch boundary, so every
   input source that routes through `execute_command` (command line, menu option,
   keyboard binding, macro) shares one consistent verb/argument split.
8. WHEN a function key (or keyboard shortcut) bound to a command is pressed, THE
   shell SHALL invoke that command with the current `Command ===>` field contents
   as its Argument_String -- observably identical to typing `<command> <field>`
   in the command field and pressing Enter. WHEN the field is empty, the command
   SHALL be invoked with no argument. This is the single general mechanism by
   which a typed value parameterises a key-invoked command (e.g. type `8`, press
   the DOWN key -> `DOWN 8`; type `LIST`, press the RETRIEVE key -> `RETRIEVE LIST`).
9. **(REVISED by CR-CH-033.)** THE disposition of the `Command ===>` field after
   an invocation (whether typed + Enter or key-forwarded) SHALL be governed by
   the Command_Line_Outcome of Requirement 13, applied identically on both input
   paths. The framework SHALL NOT hardcode a per-path clear rule here; instead it
   applies the outcome (default: clear on success, keep on unresolved, restore on
   error; overridable per command). This makes `1` + F9 -> `SWAP 1` leave the
   field empty exactly as `1` + Enter does, while letting a command that sets the
   field (RETRIEVE) win. (SUPERSEDES the prior rule that the framework never
   force-cleared the field; the earlier rule left a typed argument such as `1`
   visible after `1` + F9, inconsistent with the Enter path.)
10. A key-forwarded invocation SHALL be indistinguishable, from the command's
    point of view, from a typed `<command> <argument>` invocation: both deliver
    the same `arg` param, so a command needs no special handling for the two
    input paths.

---

### Requirement 10: Context Navigation Stack

**User Story:** As an operator, I want a return stack so that pressing END (or F3) walks me back through the contexts I navigated through, with the depth of the walk controlled by whether I chained with `.` or `;`.

**Source:** [CR-NR-057], integrates with Requirement 8 (Unified Command Target) and command-semantics Requirement 11 (Command Chain).

#### Glossary additions

- **Context_Navigation_Stack**: A per-Workbench return stack of Contexts that RETURN (END/F3) pops. Session state only; never persisted, never undoable. [CR-NR-057]
- **Navigation_Origin**: The Context at the bottom of the Context_Navigation_Stack for a navigation produced by a command chain: the POM when the chain begins with `=`, otherwise the Context active when the command was issued. [CR-NR-057]

#### Acceptance Criteria

1. THE framework SHALL maintain a per-Workbench Context_Navigation_Stack of Contexts that RETURN (END/F3) pops.
2. WHEN a navigation chain begins with `=`, THE Navigation_Origin SHALL be the POM (Home Context).
3. WHEN a navigation command does not begin with `=`, THE Navigation_Origin SHALL be the Workspace or Context active when the command was issued.
4. THE Navigation_Origin SHALL always be the bottom entry of the Context_Navigation_Stack for the navigation produced by the chain.
5. WHEN a chain segment reached via a `;` (PUSH) separator opens or changes a Context, THE framework SHALL push the prior (intermediate) Context onto the Context_Navigation_Stack before switching.
6. WHEN a chain segment reached via a `.` (STOP) separator opens or changes a Context, THE framework SHALL NOT push the intermediate Context, so only the Navigation_Origin remains beneath the destination.
7. WHEN a command does not open or change a Context (for example a state-mutating command such as SORT or CHANGE), THE Context_Navigation_Stack SHALL be unchanged regardless of the separator.
8. WHEN RETURN (END/F3) is issued and the Context_Navigation_Stack is non-empty, THE framework SHALL pop one entry and switch to that Context.
9. END / RETURN SHALL itself be a chainable command usable within a Command_Chain (for example `END ; EDIT`): it SHALL pop the Context_Navigation_Stack, and the next chain segment SHALL then run from the resulting Context.
10. THE predicate `produces_visible_workspace` (Requirement 8.9) SHALL determine whether a target counts as opening or changing a Context for Context_Navigation_Stack purposes.
11. THE Context_Navigation_Stack SHALL have a configurable maximum depth via `navigation.stack_max_depth` (positive integer, default 32); WHEN a push would exceed the maximum, THE framework SHALL drop the oldest entry and log one WARN-level record.
12. THE Context_Navigation_Stack SHALL be session state only: it SHALL NOT be persisted across sessions and SHALL NOT be recorded as an undoable transaction.
13. A single unchained navigation command SHALL behave as a STOP (`.`) invocation for stack purposes: it SHALL push only the Navigation_Origin, so one RETURN returns to the origin.

---

### Requirement 11: Uniform Command Execution Instrumentation

**User Story:** As a developer debugging the workbench, I want every command that flows through the dispatcher to log its start (id and parameters), and its completion (success or failure, a result summary, and how long it took), at a development-only level, so that I have a single consistent trace of user actions during testing without adding logging to each command by hand and without any of this overhead in a production release build.

**Source:** NEW -- derived from CR-NR-058. Generalises the existing per-dispatch logging (Requirement 2 criterion 6 error logging, Requirement 9 criterion 4 TRACE of id and params) into a uniform start/completion instrumentation applied once at the dispatch boundary. Depends on logging-subsystem Requirement 13 for the compile-time gate. [WB]

#### Glossary additions

- **Instrumentation_Point**: The single location inside `Command_Dispatch::execute_command` (and its async counterpart) where start and completion Log_Records are emitted for every command, regardless of invocation source.
- **Result_Summary**: A bounded, non-sensitive description of a Command_Result: for success, the result kind and value shape; for failure, the error description. Never the full contents of a large return value.
- **Sensitive_Param**: A Command_Param whose key or command declares it as carrying secret or privacy-relevant data (for example a password operand, or free-text search content), which must be redacted rather than logged verbatim. Aligns with command-semantics Requirement 9 criterion 17 (secret operand redaction).

#### Acceptance Criteria

1. WHEN `execute_command` is invoked for a registered command, THE Command_Dispatch SHALL emit, at the Instrumentation_Point before invoking the handler, a Development_Level (DEBUG) Log_Record containing the Command_ID and the Command_Params.
2. WHEN a command handler returns, THE Command_Dispatch SHALL emit, at the Instrumentation_Point, a completion Log_Record containing the Command_ID, a success-or-failure indicator, a Result_Summary, and the elapsed execution duration in milliseconds; for a successful command this record SHALL be at Development_Level (DEBUG), and for a failed command it SHALL be at WARN level (consistent with, and not duplicating, Requirement 2 criterion 6).
3. THE start and completion instrumentation SHALL be applied uniformly to every invocation regardless of source (keyboard shortcut, menu option, `Command ===>` line, Lua macro via the Scripting_Bridge, or plugin), because all sources route through the single `execute_command` entry point (Requirement 2 criterion 1).
4. WHEN the `dev-logging` feature is absent (release build per logging-subsystem Requirement 13), THE start record and the success completion record SHALL contribute no runtime overhead (they are compiled out), WHILE the failure completion record SHALL remain because it is emitted at WARN level (a Retained_Level).
5. WHEN Command_Params contain a Sensitive_Param, THE Command_Dispatch SHALL redact that parameter's value in both the start and completion Log_Records (for example replacing the value with `***`), and SHALL NOT write the raw value to the log.
6. THE Command_Params rendering in a Log_Record SHALL be bounded in length using the logging subsystem's existing per-record truncation (logging-subsystem Requirement 2 criterion 3), so a command invoked with a very large parameter map does not produce an unbounded log line.
7. WHEN a command is rejected before its handler runs (unregistered id per Requirement 2 criterion 2, or disabled per Requirement 2 criterion 5), THE Command_Dispatch SHALL still emit the start Log_Record and SHALL emit a completion Log_Record whose Result_Summary names the rejection reason, so the trace shows both attempted-but-rejected and executed commands.
8. THE instrumentation SHALL NOT alter command semantics: the presence or absence of the `dev-logging` feature SHALL NOT change any Command_Result, undo behaviour, or Command_History recording (Requirement 7), and SHALL NOT change the observable ordering of side effects.
9. THE completion duration SHALL be measured across the handler invocation only (from just before the handler is called to just after it returns), so the reported duration reflects command work and excludes dispatch bookkeeping.

---

### Requirement 12: Cursor_Context Package on Every Command Execution

**User Story:** As a user, I want the command I invoke -- whether by typing it, pressing a function key, clicking a menu option, or selecting it from a menu bar -- to be handed a package describing where my cursor/focus was at the moment of invocation, so that a command MAY choose to act on that context (for example, a cursor-relative scroll, or context-sensitive help) while commands that do not care simply ignore it.

**Source:** NEW -- CR-CH-028. Owner: "Pressing the function key should be treated the same as typing the command and passing it the parameters on the command line ... perhaps all commands should get the cursor context like the help key ... if all commands when executed carry a package of the context of where the cursor was placed." + "Commands should be given the context aware package, but they may ignore it or make use of it ... each command will have its own decisions to make on whether to use it or not. Lets make this work for the command handler first then all existing commands will have to be refactored to receive it ... could it be designed flexible enough as in some workspaces might want to pass more through the context than others?" Generalises the HELP-only `EditorContext` (context-help) into a package every command receives. Reuses the existing (currently unused) `ExecutionContext` / `ContextProvider` seam.

#### Acceptance Criteria

1. THE Execution_Context passed to every command handler SHALL carry a Cursor_Context package (fixed CORE + open EXTRAS, per the glossary). Because the handler signature already receives the Execution_Context, a handler that does not consult the Cursor_Context SHALL require no change and SHALL behave exactly as before (this requirement is ADDITIVE and behaviour-preserving).

2. THE Cursor_Context CORE SHALL provide, when known at invocation time: (a) the focused Workspace context name (the stable per-Workspace-kind identifier, e.g. `pom`, `editor`, `menu`); (b) a focused-control semantic identity (e.g. the command string / label of the focused Menu_Option, or an identifier of the focused field), or none when nothing interior is focused; (c) the focused control's text where meaningful (e.g. the command-line text, or the focused option's label); (d) the editor cursor line/column and selection when an editor document is active; (e) the active scroll setting (so a future cursor-relative scroll command can consult it). Any CORE field MAY be absent (represented as optional) when not applicable to the current Workspace.

3. THE Cursor_Context SHALL provide an open EXTRAS bag: a string-keyed map of Context_Value, so a Workspace MAY attach workspace-specific context beyond the CORE without any change to the CORE type. A command SHALL read EXTRAS by key and SHALL ignore keys it does not recognise. A Workspace that has nothing extra to add SHALL leave EXTRAS empty.

4. THE workbench SHALL populate the Cursor_Context from the live focus/selection state at the moment of invocation and SHALL supply it through the Command_Dispatch context seam (the `ContextProvider`) so that commands dispatched through the registry receive a populated package rather than an empty one. The command-line (`handle_command`) path and the bound-invocation paths (function key, menu option, menu bar, palette) SHALL all populate the SAME package, so a command receives identical context regardless of how it was invoked (command parity, architecture-brief Principle 2).

5. THE Cursor_Context capture SHALL be a per-invocation SNAPSHOT: it reflects focus/selection as of when the command was invoked and SHALL NOT be retained or mutated by a command across invocations.

6. A command MAY use the Cursor_Context to decide behaviour when its own explicit parameters are absent, but explicit parameters SHALL take precedence. (Illustrative, NOT implemented in this slice: `LEFT 10` scrolls 10 columns regardless of context; a bare `LEFT` MAY, once made context-sensitive, consult the Cursor_Context and the active scroll setting -- e.g. `CSR` cursor-relative -- to decide the amount. This slice only guarantees the package is DELIVERED; per-command consumption such as the CSR behaviour is deferred to later command work.)

7. WHEN a bound key or menu affordance dispatches a command that does not yet exist (is not built), THE workbench SHALL surface a single canonical "command not implemented yet" message rather than a bespoke per-command string. (The complementary "out of context" message -- a command that exists but is meaningless in the current Workspace -- is the responsibility of each command to emit as it is built, using the Cursor_Context, and is NOT owned by the dispatcher; it is out of scope for this slice.)

8. THE context-sensitive HELP behaviour SHALL be the first consumer of the Cursor_Context (CR-NR-079): WHEN HELP is invoked (F1 or the `HELP` command) with a focused Menu_Option, THE resolved Help Topic_Key SHALL reflect that option (e.g. focus on the `FILES` option resolves the help topic for `FILES`), generalising the existing HELP `EditorContext` to read from the Cursor_Context. Absent a specific focused control, HELP SHALL behave as today (the existing "not available yet" fallback, function-keys-and-history Requirement 18).

---

### Requirement 13: Command_Line_Outcome (the command decides what returns to the Command Field)

**User Story:** As a user, I want the command line to be cleared when I run a
command, but I want each command to be able to decide what (if anything) goes
back into it -- so that RETRIEVE can recall a command into the field, a command
that fails can put itself back for me to edit, and a future command can offer a
suggested next prompt -- and I want that same mechanism to be usable by macros
(Lua, REXX) and by commands written in other languages, through an adequately
documented contract.

**Source:** CR-CH-033. Owner: "the command can decide what to do with the command
line ... clear the command just before handing control to the command; the
command then decides whether to put anything back ... if the handler cannot find
the command it remains for typing correction; if the command itself errors (e.g.
FIND does not find the string) it must populate the command back ... the most
flexible design would allow the command to determine what goes back ... a command
that could create suggested prompts ... LUA/REXX macro scripts should be able to
address these ... commands written in other languages should be able to make use
of this if adequately documented." Owner confirmed the sliced delivery and the
serialisable data shape (confident it backs bridges for other languages).

**Glossary additions:**
- **Command_Line_Outcome** -- a value RETURNED by an invocation describing what
  the `Command ===>` field should contain AFTER the command runs. Semantic model
  (four variants): `Clear` (empty the field), `Restore` (put back exactly what
  was executed -- for correction / retry), `Set(text)` (put arbitrary text back
  -- e.g. RETRIEVE recall or a suggested prompt), `Leave` (do not touch the field
  -- the command manages it itself).
- **Outcome_Data_Shape** -- the serialisable representation of a
  Command_Line_Outcome: a tagged object `{ "action": "clear" | "restore" | "set"
  | "leave", "text": "<present only when action = set>" }`. It is the documented
  public boundary that non-Rust producers (macros, external commands) emit; it
  maps totally and losslessly to and from the native Command_Line_Outcome.

#### Acceptance Criteria

1. THE framework SHALL clear the `Command ===>` field JUST BEFORE handing control
   to a RESOLVED command, then apply the command's Command_Line_Outcome AFTER the
   command runs. This ordering is uniform across BOTH the typed-`Enter` path and
   the key-forwarded (function-key / shortcut) path (command parity, Requirement
   9.10): a command is invoked identically and its outcome applied identically
   regardless of input source.

2. WHEN a command cannot be RESOLVED (no matching handler / built-in / menu /
   macro), THE framework SHALL NOT clear the field: the unresolved text SHALL
   remain so the user can correct it. An unresolved command has no outcome
   because it never ran.

3. THE DEFAULT Command_Line_Outcome the framework applies when a command does not
   return an explicit one SHALL be: `Clear` when the command SUCCEEDED; `Restore`
   (the original executed text) when the command RESOLVED but reported an ERROR
   (so, e.g., FIND that does not find its string comes back for editing). A
   command MAY OVERRIDE either default by returning an explicit outcome.

4. A command SHALL be able to RETURN an explicit Command_Line_Outcome that the
   framework applies verbatim: `Clear`, `Restore`, `Set(text)`, or `Leave`. In
   particular RETRIEVE SHALL return `Set(<recalled command>)` (and RETRIEVE LIST
   its existing clear-and-open-picker behaviour), so recall is expressed through
   this one mechanism rather than a special case.

5. THE Command_Line_Outcome SHALL have a serialisable Outcome_Data_Shape (the
   tagged `{action, text?}` object) with a TOTAL, LOSSLESS mapping in BOTH
   directions to the native value: every native variant serialises to exactly one
   data shape and every valid data shape deserialises to exactly one native
   variant. An invalid or absent shape SHALL map to the framework DEFAULT
   (criterion 3), never to a panic.

6. THE Outcome_Data_Shape SHALL be the documented contract by which a NON-Rust
   producer drives the command line: a Lua macro, a REXX macro, or an
   out-of-process / other-language command (an `External` Command_Target,
   Requirement 8) that emits a valid shape SHALL have it applied exactly as a
   native command's returned outcome. The shape SHALL contain no Rust-specific
   construct, so any producer able to emit a key/value object can conform.

7. THE delivery SHALL be SLICED: Slice 1 -- the native Command_Line_Outcome and
   its application at the single dispatch decision point on both paths, with the
   default rules and RETRIEVE (`Set`) + FIND (error-`Restore`) as references
   (criteria 1-4). Slice 2 -- the serialisable Outcome_Data_Shape and its total
   round-trip mapping, documented as the public boundary (criterion 5, and the
   documentation half of criterion 6). Slice 3+ -- per-engine bridges (Lua, then
   External, then REXX) that map the shape to the native outcome (the enforcement
   half of criterion 6), each gated WHEN that engine's execution exists (Lua and
   External execution are deferred per Requirement 12; no REXX engine exists yet).

8. THE Command_Line_Outcome contract SHALL NOT require any existing command to
   change: a command that returns no outcome gets the default (criterion 3), so
   the mechanism is ADDITIVE and behaviour-preserving except for the intended
   change (a successful command now clears the field on both paths, fixing the
   `1`-remains-after-`1`+F9 inconsistency).
