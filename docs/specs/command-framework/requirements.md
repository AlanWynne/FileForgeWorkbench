# Requirements Document

## Introduction

This feature specifies the command framework for FileForgeWorkbench (`ff-command` crate). The command framework is the **central dispatch mechanism** for all user-facing operations in the workbench. It provides a global command registry, a single dispatch entry point, rich command metadata, automatic undo/redo integration, a keyboard shortcut management system, a scripting bridge for Lua macros, and a command history log.

The command-driven architecture (cross-cutting Requirement 4 in the project-master spec) mandates that **all state-changing user operations** are routed through this framework — whether invoked via keyboard shortcuts, menus, the command line, macros, or plugins. This ensures consistent undo/redo behaviour, shortcut discoverability, macro recordability, and a single audit trail for all user actions.

The `ff-command` crate is a Wave 2 (Platform Architecture) dependency. It depends on `ff-logging` for diagnostics and is consumed by virtually every higher-level crate: `ff-core` (platform-core), `ff-plugin` (plugin-architecture), `ff-workflow` (workflow-engine), all editor subsystems, and the GUI shell.

**Source references:**
- **WB** = Workbench Architecture Brief §7 — command-driven architecture principle
- **FFE** = FileForgeEditor `core-command-semantics` (ISPF command engine, adapted)
- **SCI** = Scintilla KeyMap and command binding concepts (adapted)

## Glossary

- **Command**: A named, executable operation registered in the command registry. Each command has a unique string identifier, metadata, and an execution handler. [WB]
- **Command_ID**: A unique dot-separated string identifier for a command (e.g., `"file.save"`, `"edit.undo"`, `"find.next"`). [WB]
- **Command_Registry**: The global, thread-safe collection of all registered commands, supporting registration, lookup, and runtime discovery. [WB]
- **Command_Dispatch**: The single entry point through which all command executions are routed, regardless of invocation source. [WB]
- **Command_Params**: A typed key-value map of parameters passed to a command at execution time. [WB]
- **Execution_Context**: The ambient state available to a command during execution — active document, selection, cursor position, active panel. [WB]
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

**Source:** WB Architecture Brief §7 — command registry. [WB]

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

**Source:** WB Architecture Brief §7 — single dispatch entry point. [WB]

#### Acceptance Criteria

1. THE Command_Dispatch SHALL provide a single entry point `execute_command(id: &str, params: CommandParams) → CommandResult` through which all command invocations are routed.
2. WHEN `execute_command` is called with a Command_ID that is not registered, THE Command_Dispatch SHALL return an error result containing the unrecognized command ID, without panicking or modifying application state.
3. WHEN `execute_command` is called, THE Command_Dispatch SHALL construct an Execution_Context containing the currently active document (if any), the current selection/cursor position, and the active panel identifier, and SHALL pass this context to the command handler.
4. THE Command_Dispatch SHALL support both synchronous command execution (blocking the caller until the command completes) and asynchronous command execution (returning a future that resolves when the command completes).
5. THE Command_Dispatch SHALL validate that the command's enabled predicate returns true before executing the command; IF the command is disabled, THEN THE Command_Dispatch SHALL return an error result indicating the command is not currently available, without invoking the command handler.
6. WHEN a command handler returns an error, THE Command_Dispatch SHALL propagate the error to the caller as a `CommandResult::Err` and SHALL write a WARN-level log record containing the Command_ID and error description via the logging subsystem.
7. ALL user-facing operations that modify application state SHALL be invoked through `execute_command`; no UI code SHALL directly mutate application state without routing through the command framework.
8. THE Command_Dispatch SHALL accept Command_Params as a typed key-value map supporting string, integer, float, boolean, and nested map value types.

---

### Requirement 3: Command Metadata

**User Story:** As a workbench developer, I want rich metadata attached to each command, so that menus, keybinding UI, help systems, and command palettes can present commands with display names, descriptions, icons, and availability information without hardcoding knowledge of specific commands.

**Source:** WB Architecture Brief §7 — command metadata for runtime inspection. [WB, FFE]

#### Acceptance Criteria

1. EACH registered command SHALL have associated metadata containing: a display name (human-readable, localizable string), a description (one-sentence summary of what the command does), and a category (dot-separated namespace matching the Command_ID prefix, e.g., `"file"`, `"edit"`, `"view"`).
2. EACH registered command SHALL optionally have a default keyboard shortcut binding specified in its metadata; IF no shortcut is specified, THEN the command has no default binding.
3. EACH registered command SHALL optionally have an icon reference (a string identifier referencing an icon asset) for display in menus, toolbars, and command palettes.
4. EACH registered command SHALL have an enabled predicate — a function that, given the current Execution_Context, returns a boolean indicating whether the command can currently execute. IF no predicate is provided, THEN the command SHALL be considered always enabled.
5. EACH registered command SHALL have a visibility predicate — a function that, given the current Execution_Context, returns a boolean indicating whether the command should appear in menus and command palettes. IF no predicate is provided, THEN the command SHALL be considered always visible.
6. THE Command_Registry SHALL provide a method to query the metadata for any registered command by Command_ID, returning all metadata fields without executing the command.
7. WHEN the enabled or visibility predicate for a command is evaluated, THE evaluation SHALL NOT produce side effects and SHALL complete within 1 millisecond to avoid blocking UI rendering.

---

### Requirement 4: Undo/Redo Integration

**User Story:** As a workbench developer, I want the command framework to automatically integrate with the undo/redo system, so that every undoable command produces an undo record as part of its execution without requiring each command to manually manage the undo stack.

**Source:** WB Architecture Brief §7 — undo/redo integration. Cross-references `undo-redo-transactions` crate. [WB]

#### Acceptance Criteria

1. EACH registered command SHALL declare whether it is undoable (produces an Undo_Record) or non-undoable (view changes, settings modifications, navigation) at registration time.
2. WHEN an undoable command executes successfully, THE command handler SHALL return an Undo_Record as part of its CommandResult, and THE Command_Dispatch SHALL automatically push that Undo_Record onto the active undo stack for the relevant document or context.
3. WHEN a non-undoable command executes, THE Command_Dispatch SHALL NOT push any record to the undo stack and SHALL NOT clear or modify the existing undo/redo history.
4. THE combination of command execution and undo record creation SHALL be atomic: IF the command handler returns an error, THEN no Undo_Record SHALL be pushed to the undo stack, and application state SHALL remain unchanged (no partial state).
5. WHEN the built-in `"edit.undo"` command is executed, THE Command_Dispatch SHALL pop the most recent Undo_Record from the active undo stack and apply it to reverse the effect of the original command, moving the record to the redo stack.
6. WHEN the built-in `"edit.redo"` command is executed, THE Command_Dispatch SHALL pop the most recent record from the redo stack and re-apply the command, moving the record back to the undo stack.
7. WHEN an undoable command is executed after one or more undo operations, THE Command_Dispatch SHALL clear the redo stack for the active context (standard undo semantics — executing a new command invalidates the redo history).

---

### Requirement 5: Keyboard Shortcut Management

**User Story:** As a user, I want a keyboard shortcut system that prevents conflicts, supports user customization, and handles multi-key sequences, so that I can efficiently invoke commands without memorizing arbitrary bindings or encountering unexpected behaviour.

**Source:** WB Architecture Brief §7, Cross-cutting Requirement 10 (keyboard shortcut registry). [WB, FFE, SCI]

#### Acceptance Criteria

1. THE Shortcut_Registry SHALL maintain a mapping from keyboard chords to Command_IDs, where a chord is defined as a combination of zero or more modifier keys (Ctrl, Alt, Shift, Super/Win) plus a primary key.
2. THE Shortcut_Registry SHALL support multi-key sequences (e.g., Ctrl+K followed by Ctrl+C), where the first chord enters a pending state and the framework waits for the second chord to complete the binding or times out after 2 seconds (reverting to no pending state).
3. THE following shortcuts SHALL be reserved globally and SHALL NOT be overridden by user configuration, plugins, or any sub-project registration: F1 (Help), Ctrl+Plus/Ctrl+Minus/Ctrl+0 (Zoom), Ctrl+Z/Ctrl+Y/Ctrl+Shift+Z (Undo/Redo), Ctrl+C/Ctrl+X/Ctrl+V/Ctrl+A (Clipboard), Ctrl+S (Save), Ctrl+F (Find), Ctrl+H (Change), Ctrl+G (Go to line), Ctrl+Tab/Ctrl+Shift+Tab (Tab switch), Ctrl+W (Close tab), Ctrl+N (New tab), Ctrl+Shift+D (Dock/undock), Ctrl+Shift+T (Undock/redock tab).
4. WHEN a shortcut binding is registered that conflicts with an existing binding (same chord sequence already mapped to a different Command_ID), THE Shortcut_Registry SHALL reject the registration and return an error indicating the conflict, identifying both the new and existing Command_IDs.
5. WHEN a shortcut binding is registered that conflicts with a reserved shortcut, THE Shortcut_Registry SHALL reject the registration and return an error indicating that the shortcut is reserved and cannot be overridden.
6. THE Shortcut_Registry SHALL support user-configurable shortcut overrides for all non-reserved commands, loaded from the workbench configuration system (TOML-based key map file).
7. WHEN a keyboard chord is received that matches a registered shortcut, THE Shortcut_Registry SHALL resolve it to the bound Command_ID and invoke `execute_command` through the Command_Dispatch.
8. FUNCTION keys F2–F24 SHALL be user-configurable via the key map system, and plugins SHALL be able to register shortcut bindings for their commands through the Shortcut_Registry (subject to conflict detection and reserved shortcut rules).

---

### Requirement 6: Scripting Bridge

**User Story:** As a macro developer, I want to invoke any registered command from a Lua script and receive structured results, so that macros can automate workflows by composing commands without reimplementing their logic.

**Source:** WB Architecture Brief §7 — scripting bridge. Cross-references `lua-macro-engine` crate. [WB, FFE]

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

**Source:** FFE `function-keys-and-command-history` — RETRIEVE command history. [FFE]

#### Acceptance Criteria

1. THE Command_History SHALL record every successfully executed command invocation, storing the Command_ID, a timestamp (UTC, millisecond precision), and the Command_Params that were passed.
2. THE Command_History SHALL have a configurable maximum depth (number of entries retained), specified via the workbench configuration system under `commands.history_depth`, with a default of 500 entries.
3. IF the `commands.history_depth` setting contains a value less than 10 or greater than 10000, THEN THE Command_History SHALL clamp the value to the nearest bound (10 or 10000) and write a WARN-level log record indicating the adjustment.
4. WHEN the history reaches its maximum depth and a new entry is recorded, THE Command_History SHALL discard the oldest entry to make room for the new one (FIFO eviction).
5. THE Command_History SHALL be persistent across application sessions: WHEN the application shuts down normally, THE Command_History SHALL serialize its entries to a file in the workbench data directory; WHEN the application starts, THE Command_History SHALL load the persisted entries and resume from where it left off.
6. IF the history persistence file cannot be read at startup (corrupted, missing, or permission error), THEN THE Command_History SHALL start with an empty history and write a WARN-level log record indicating the reason.
7. THE Command_History SHALL be safe to read and write from any thread without requiring the caller to acquire an external lock (thread-safe access).
8. THE Command_History SHALL provide a query interface: retrieve the last N entries, retrieve entries matching a Command_ID prefix, and retrieve entries within a time range.

