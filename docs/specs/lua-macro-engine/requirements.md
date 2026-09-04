# Requirements Document

## Introduction

This feature specifies the Lua macro engine for FileForgeWorkbench (`ff-macro` crate). The macro engine is the **scripting and automation layer** that enables users and plugins to extend editor behaviour through Lua scripts. It provides a Lua 5.4 runtime (via the `mlua` crate), a rich editor API surface, a comprehensive event hook system, per-buffer state isolation, automatic script reloading during development, macro directory scanning, security modes, and debugging support.

The macro engine merges and extends two sources:
1. **FileForgeEditor mvp Requirement 7** — the existing `LuaMacroEngine` with its `editor.*` global API, MACRO command, and file-lifecycle event hooks.
2. **SciTE LuaExtension** — per-keystroke hooks (OnChar, OnKey), per-buffer Lua state tables, automatic reload of modified scripts, multiple extension scripts, editor properties access, and pane-object-style API design.

The `ff-macro` crate depends on `ff-command` (command dispatch and scripting bridge), `ff-document` (document model for buffer access), `ff-undo` (undo/redo transactions), `ff-config` (configuration reading), `ff-logging` (diagnostics), and `ff-plugin` (plugin lifecycle — the macro engine itself registers as a plugin providing the `MacroCapability`).

**Source references:**
- **FFE-MVP-7** = FileForgeEditor mvp-implementation Requirement 7 (Lua Macro API)
- **SCI-STE-LUA** = SciTE LuaExtension (OnChar, OnKey, per-buffer state, auto-reload, IFaceTable, pane object)
- **WB** = Workbench Architecture Brief §7 (command-driven architecture), §10 (plugin model)

## Glossary

- **LuaMacroEngine**: The core struct that owns the Lua runtime (`mlua::Lua`), manages script loading, provides the editor API to scripts, and dispatches event hooks. [FFE-MVP-7]
- **Lua_Runtime**: The embedded Lua 5.4 interpreter instance provided by the `mlua` crate. [FFE-MVP-7]
- **Editor_API**: The set of Lua global functions and objects exposed to macros for reading and modifying buffer content, querying state, and dispatching commands. [FFE-MVP-7, SCI-STE-LUA]
- **Event_Hook**: A named Lua function that the engine invokes automatically when a specific editor event occurs (e.g., file open, character typed, save). [FFE-MVP-7, SCI-STE-LUA]
- **Hook_Registry**: The internal data structure that maps event names to registered Lua handler functions, supporting priority ordering and multiple handlers per event. [FFE-MVP-7]
- **Per_Buffer_State**: A Lua table associated with each open buffer/tab that persists across buffer switches, providing script-local storage tied to a specific document. [SCI-STE-LUA]
- **Macro_Script**: A `.lua` file containing user-written automation code, located in a configured macro directory. [FFE-MVP-7]
- **Macro_Directory**: One or more filesystem directories scanned for `.lua` macro scripts at startup and on demand. [FFE-MVP-7, WB]
- **Auto_Reload**: The mechanism that detects changes to loaded macro scripts on disk and re-executes them to pick up modifications without application restart. [SCI-STE-LUA]
- **Security_Mode**: A configuration-driven restriction level that controls which macros may execute: Disabled, Prompt, TrustedOnly, or Enabled. [WB]
- **Macro_Transaction**: A group of document modifications performed by a single macro invocation, wrapped in a single undo/redo transaction so that all changes revert atomically. [WB, FFE-MVP-7]
- **Cancellable_Hook**: An event hook that returns a boolean; when any handler returns `false`, the triggering operation is cancelled. [FFE-MVP-7]
- **Script_Error**: A Lua runtime error captured by the engine and propagated to the user via the status bar or diagnostics panel without crashing the host. [FFE-MVP-7]

## Requirements

### Requirement 1: Lua Runtime Embedding

**User Story:** As a workbench developer, I want a safely embedded Lua 5.4 runtime that scripts can use for automation, so that macros execute in a controlled sandbox with access only to approved APIs.

**Source:** FFE-MVP-7 (mlua-based LuaMacroEngine), WB §10 (plugin sandboxing). [FFE-MVP-7, WB]

#### Acceptance Criteria

1. THE LuaMacroEngine SHALL embed a Lua 5.4 runtime using the `mlua` crate, initialized at engine startup.
2. THE Lua_Runtime SHALL load only the Lua standard libraries explicitly approved by the Security_Mode: `base`, `string`, `table`, `math`, `utf8`, and `coroutine` SHALL always be available; `io`, `os`, and `debug` SHALL only be available when Security_Mode is `Enabled`.
3. THE Lua_Runtime SHALL enforce a configurable instruction count limit (default: 10 million instructions per invocation) to prevent infinite loops from freezing the application.
4. THE Lua_Runtime SHALL enforce a configurable memory limit (default: 64 MB) per macro invocation to prevent memory exhaustion.
5. WHEN the instruction count or memory limit is exceeded, THE LuaMacroEngine SHALL terminate the running script, return a Script_Error describing the violation, and leave the document in a consistent state by rolling back the current Macro_Transaction.
6. THE LuaMacroEngine SHALL be instantiated once per application lifetime and SHALL reuse the same Lua_Runtime across macro invocations (preserving global state between calls unless explicitly reset).
7. THE LuaMacroEngine SHALL register itself as a plugin providing the `MacroCapability` through the `ff-plugin` plugin architecture.

---

### Requirement 2: Editor API Surface

**User Story:** As a macro author, I want a comprehensive Lua API for reading and modifying buffer content, querying editor state, and invoking commands, so that I can automate any editing task programmable in Lua.

**Source:** FFE-MVP-7 (editor.lines, get_line, set_line, tag, command), SCI-STE-LUA (pane object, properties, cursor, selection). [FFE-MVP-7, SCI-STE-LUA]

#### Acceptance Criteria

1. THE LuaMacroEngine SHALL expose a Lua global table `editor` with the following core functions available to every executing macro: `editor.lines()`, `editor.get_line(n)`, `editor.set_line(n, text)`, `editor.insert_line(n, text)`, `editor.delete_line(n)`, `editor.tag(n)`, and `editor.command(str)`.
2. WHEN `editor.lines()` is called, THE function SHALL return the total number of lines in the active document buffer as a Lua integer.
3. WHEN `editor.get_line(n)` is called with a 1-based line number, THE function SHALL return the text content of that line as a Lua string, excluding the line terminator.
4. WHEN `editor.set_line(n, text)` is called, THE function SHALL replace the content of line `n` in the active buffer with `text` and record the change within the current Macro_Transaction.
5. WHEN `editor.insert_line(n, text)` is called, THE function SHALL insert a new line with content `text` before line `n`, shifting subsequent lines down, and record the change in the current Macro_Transaction.
6. WHEN `editor.delete_line(n)` is called, THE function SHALL remove line `n` from the buffer, shifting subsequent lines up, and record the change in the current Macro_Transaction.
7. WHEN `editor.tag(n)` is called, THE function SHALL set the `tagged` flag in the line metadata for line `n` in the active document.
8. WHEN `editor.command(str)` is called, THE function SHALL dispatch `str` through the command framework's Scripting_Bridge exactly as if the user had typed it in the primary command line, and SHALL return a Lua boolean indicating whether the command succeeded.
9. THE `editor` table SHALL additionally expose the following state query functions: `editor.cursor_line()` (returns 1-based line number of cursor), `editor.cursor_col()` (returns 1-based column of cursor), `editor.selection()` (returns start_line, start_col, end_line, end_col or nil if no selection), `editor.language()` (returns the detected language name as a string), and `editor.file_path()` (returns the absolute path of the active buffer or nil for untitled buffers).
10. THE `editor` table SHALL expose `editor.config(key)` that reads the effective value of a configuration key (via ff-config) and returns it as the appropriate Lua type (string, number, boolean, or nil if undefined).
11. IF a line number argument passed to any `editor.*` function is out of range (less than 1 or greater than `editor.lines()`), THEN THE function SHALL raise a Lua error with a descriptive message including the invalid index and the valid range.

---

### Requirement 3: Event Hook System

**User Story:** As a macro author, I want to register Lua functions that fire automatically on editor events (file open, save, character typed, key pressed), so that I can implement reactive automation like auto-formatting on save or bracket-matching on keystroke.

**Source:** FFE-MVP-7 (on_open, on_before_save, on_after_save), SCI-STE-LUA (OnChar, OnKey, OnDwellStart, OnOpen, OnBeforeSave, OnSave, OnClose, OnSwitchFile, OnExecute). [FFE-MVP-7, SCI-STE-LUA]

#### Acceptance Criteria

1. THE LuaMacroEngine SHALL support the following event hooks, invoked at the appropriate editor lifecycle points: `OnOpen(file_path)`, `OnBeforeSave(file_path)`, `OnAfterSave(file_path)`, `OnClose(file_path)`, `OnSwitchBuffer(file_path)`, `OnChar(character)`, `OnKey(key_code, shift, ctrl, alt)`, `OnCommand(command_id, params)`, and `OnError(error_message)`.
2. MACRO scripts SHALL register event handlers by defining global Lua functions with the corresponding event name (e.g., `function OnChar(ch) ... end`); THE engine SHALL discover these functions after a script is loaded and register them in the Hook_Registry.
3. WHEN multiple scripts define handlers for the same event, THE Hook_Registry SHALL invoke all handlers in script-load order (first loaded = first invoked); IF any handler returns `false` for a Cancellable_Hook, THE engine SHALL stop invoking subsequent handlers and cancel the triggering operation.
4. THE `OnBeforeSave` hook SHALL be a Cancellable_Hook: IF any registered handler returns `false`, THEN THE save operation SHALL be cancelled and the engine SHALL display a message in the status bar indicating the macro that cancelled the save.
5. THE `OnCommand` hook SHALL be a Cancellable_Hook: IF any registered handler returns `false`, THEN the command SHALL NOT be executed, allowing macros to intercept and override built-in commands.
6. THE `OnChar(character)` hook SHALL fire after a character is inserted into the buffer, providing the inserted character as a single-character Lua string; the hook is NOT cancellable (the character is already inserted).
7. THE `OnKey(key_code, shift, ctrl, alt)` hook SHALL fire before a keypress is processed; it SHALL be a Cancellable_Hook — if a handler returns `false`, the default key action is suppressed.
8. THE `OnOpen(file_path)` hook SHALL fire after a file has been fully loaded into a buffer and is ready for editing.
9. THE `OnClose(file_path)` hook SHALL fire before a buffer is discarded, allowing scripts to perform cleanup.
10. THE `OnSwitchBuffer(file_path)` hook SHALL fire when the user switches between open tabs/buffers, providing the path of the newly active buffer.
11. THE `OnError(error_message)` hook SHALL fire when any other hook or macro invocation raises a Script_Error, allowing error-logging macros to capture all failures.

---

### Requirement 4: Per-Buffer State Isolation

**User Story:** As a macro author, I want per-buffer storage that persists as I switch between tabs, so that my macros can track state (parse caches, counters, custom flags) for each open document independently.

**Source:** SCI-STE-LUA (SciTE_BufferData_Array, per-buffer Lua tables). [SCI-STE-LUA]

#### Acceptance Criteria

1. THE LuaMacroEngine SHALL maintain a Lua table called `buffer` in the global scope; this table SHALL be automatically swapped to the correct per-buffer instance whenever the active buffer changes.
2. WHEN a new buffer is opened (new file or file load), THE engine SHALL create a fresh empty Lua table as the `buffer` global for that buffer.
3. WHEN the user switches between buffers (tab switch), THE engine SHALL save the current `buffer` table for the departing buffer and restore the `buffer` table associated with the arriving buffer.
4. WHEN a buffer is closed, THE engine SHALL discard its associated `buffer` table and release the memory.
5. MACRO scripts MAY freely read and write keys on the `buffer` table (e.g., `buffer.parse_cache = {...}`, `buffer.counter = 0`); the engine SHALL NOT impose restrictions on table structure.
6. THE `buffer` table SHALL be nil during engine startup scripts (before any buffer is active); scripts that access `buffer` during startup SHALL receive `nil` and SHALL NOT crash.
7. WHEN the `OnSwitchBuffer` hook fires, THE new buffer's `buffer` table SHALL already be active before hook handlers execute, so that handlers can immediately read/write per-buffer state.

---

### Requirement 5: MACRO, EXEC, and RUN Commands

**User Story:** As a user, I want primary commands that let me invoke Lua macros by name from the command line, execute inline Lua expressions, and run macro files by path, so that I have multiple convenient ways to trigger automation.

**Source:** FFE-MVP-7 (MACRO primary command), SCI-STE-LUA (OnExecute, dostring). [FFE-MVP-7, SCI-STE-LUA]

#### Acceptance Criteria

1. WHEN the `MACRO <name>` primary command is issued, THE LuaMacroEngine SHALL locate the named `.lua` file in the configured Macro_Directories, load it, and execute it in the current Lua_Runtime context.
2. WHEN the `EXEC <lua_expression>` primary command is issued, THE LuaMacroEngine SHALL evaluate the Lua expression string directly (equivalent to `dostring`) and display any returned value in the status bar.
3. WHEN the `RUN <path>` primary command is issued, THE LuaMacroEngine SHALL load and execute the `.lua` file at the specified absolute or workspace-relative path, regardless of whether it resides in a Macro_Directory.
4. WHEN a MACRO/EXEC/RUN command is invoked, THE engine SHALL wrap the entire execution in a single Macro_Transaction so that all document modifications are atomically undoable with a single UNDO command.
5. IF the macro name in `MACRO <name>` does not resolve to any file in the configured Macro_Directories, THEN the engine SHALL return an error displayed in the status bar: "Macro not found: <name>".
6. IF the file path in `RUN <path>` does not exist or is not readable, THEN the engine SHALL return an error displayed in the status bar: "Cannot open macro file: <path>".
7. THE `MACRO`, `EXEC`, and `RUN` commands SHALL be registered with the command framework (command IDs: `"macro.run_named"`, `"macro.exec_inline"`, `"macro.run_file"`) and SHALL be invocable from keyboard shortcuts, menus, and other macros via `editor.command()`.

---

### Requirement 6: Macro Error Handling and Rollback

**User Story:** As a user, I want macro errors to be reported clearly without losing my work, and I want partial changes from a failed macro to be automatically rolled back, so that a buggy script never leaves my document in an inconsistent state.

**Source:** FFE-MVP-7 (error propagation to status bar without crash), WB (command-driven undo/redo integration). [FFE-MVP-7, WB]

#### Acceptance Criteria

1. IF a Lua macro raises a runtime error during execution, THEN THE LuaMacroEngine SHALL catch the error, roll back the current Macro_Transaction (undoing all document changes made by the macro so far), and propagate the error message to the status bar.
2. THE error message displayed SHALL include: the macro name or expression that failed, the Lua error message, and the Lua stack traceback (when debug mode is enabled via configuration).
3. WHEN a macro error occurs, THE LuaMacroEngine SHALL NOT crash, panic, or leave the Lua_Runtime in an unrecoverable state — subsequent macro invocations SHALL continue to function correctly.
4. IF a Cancellable_Hook handler raises a runtime error, THEN the hook SHALL be treated as if it returned `true` (do not cancel), the error SHALL be reported, and subsequent handlers SHALL still be invoked.
5. THE LuaMacroEngine SHALL fire the `OnError(error_message)` hook after any macro error, providing the full error string to any registered error-handling scripts.
6. WHEN debug mode is enabled (configuration key `macro.debug_traceback = true`), THE error output SHALL include the full Lua call stack with source file paths and line numbers.
7. IF rollback fails (internal inconsistency), THE engine SHALL log an ERROR-level diagnostic, display a user-visible warning that manual undo may be needed, and continue operating.

---

### Requirement 7: Macro Security Modes

**User Story:** As a user, I want control over which macros can run and what system access they have, so that I can safely open projects containing untrusted macro files without risking data loss or unwanted side effects.

**Source:** WB Architecture Brief §10 (sandboxing), FFE architecture (MacroSecurityMode). [WB]

#### Acceptance Criteria

1. THE LuaMacroEngine SHALL support four Security_Modes configured via the configuration system key `macro.security_mode`: `Disabled`, `Prompt`, `TrustedOnly`, and `Enabled`.
2. WHEN Security_Mode is `Disabled`, THE engine SHALL refuse to execute any macro, returning an error message: "Macro execution is disabled by security policy."
3. WHEN Security_Mode is `Prompt`, THE engine SHALL display a confirmation dialog to the user before executing any macro that is not in the trusted list; the user may Allow Once, Always Trust, or Deny.
4. WHEN Security_Mode is `TrustedOnly`, THE engine SHALL only execute macros whose file paths are listed in the trusted-scripts configuration (`macro.trusted_paths` array) or located within user-level Macro_Directories.
5. WHEN Security_Mode is `Enabled`, THE engine SHALL execute any requested macro without restriction.
6. REGARDLESS of Security_Mode, THE engine SHALL never expose `os.execute`, `io.popen`, `loadfile` (for arbitrary paths outside Macro_Directories), or `dofile` (for arbitrary paths) unless the script is explicitly trusted.
7. THE configuration key `macro.security_mode` SHALL default to `Prompt` for new installations.

---

### Requirement 8: Auto-Reload of Modified Scripts

**User Story:** As a macro developer, I want my scripts to automatically reload when I save changes to them, so that I can iterate on macro code without restarting the editor or manually re-running a load command.

**Source:** SCI-STE-LUA (ext.lua.auto.reload, OnSave reload check). [SCI-STE-LUA]

#### Acceptance Criteria

1. WHEN `macro.auto_reload` is enabled in configuration (default: `true`), THE LuaMacroEngine SHALL monitor all loaded macro script files for modifications using the platform file watcher (via `ff-vfs` connector-local-fs watcher or OS-native watcher).
2. WHEN a loaded macro script is modified on disk, THE engine SHALL re-execute the script within 2 seconds of detecting the change, re-registering any event hooks the script defines.
3. WHEN a script is auto-reloaded, THE engine SHALL first unregister all event hooks previously registered by that specific script, then re-run the script to register fresh hooks — preventing duplicate handler registrations.
4. IF a script fails to load during auto-reload (syntax error or runtime error), THE engine SHALL retain the previously loaded version's hooks, display the error in the status bar, and log a WARN-level diagnostic.
5. WHEN `macro.auto_reload` is disabled, THE engine SHALL NOT monitor script files and SHALL only reload scripts when explicitly requested via the `MACRO` command or application restart.
6. THE auto-reload mechanism SHALL NOT interfere with per-buffer state — the `buffer` tables SHALL be preserved across script reloads.

---

### Requirement 9: Macro Directory Scanning

**User Story:** As a user, I want the editor to automatically discover macro scripts from configured directories, so that I can organize my macros in folders and have them available without manual registration.

**Source:** FFE-MVP-7 (macros/ directory), SCI-STE-LUA (ext.lua.startup.script, extension script paths). [FFE-MVP-7, SCI-STE-LUA]

#### Acceptance Criteria

1. THE LuaMacroEngine SHALL scan one or more configured Macro_Directories for `.lua` files at startup, specified by the configuration key `macro.directories` (an array of paths).
2. THE default Macro_Directories SHALL include: the user-level macro directory (`~/.config/ffworkbench/macros/`), and if a workspace is open, a workspace-level `macros/` subdirectory relative to the workspace root.
3. WHEN scanning a Macro_Directory, THE engine SHALL discover all `.lua` files recursively (up to 3 levels of subdirectory depth) and register them as available macros keyed by filename without extension (e.g., `macros/format_cobol.lua` → macro name `"format_cobol"`).
4. IF two macro scripts in different directories share the same base name, THE engine SHALL prefer the script from the higher-priority directory (workspace > user), and log a DEBUG-level message noting the shadowing.
5. THE engine SHALL support a designated startup script (`macro.startup_script` configuration key) that is executed once at engine initialization, before any buffer is loaded — useful for defining global utility functions.
6. THE engine SHALL support a per-extension auto-load pattern (`macro.auto_load_for.<extension>` configuration key pointing to a script name) that automatically executes a script when a file with a matching extension is opened.
7. WHEN a new `.lua` file appears in a monitored Macro_Directory while the application is running, THE engine SHALL detect it within 5 seconds and make it available for `MACRO <name>` invocation (hot-discovery).

---

### Requirement 10: Script Debugging Support

**User Story:** As a macro developer, I want diagnostic tools — verbose tracebacks, a console for evaluating expressions, and execution timing — so that I can efficiently debug and profile my scripts.

**Source:** SCI-STE-LUA (ext.lua.debug.traceback, trace function), WB (developer tooling). [SCI-STE-LUA, WB]

#### Acceptance Criteria

1. WHEN `macro.debug_traceback` is enabled in configuration, THE engine SHALL include full Lua stack tracebacks in all error messages (not just the immediate error line).
2. THE LuaMacroEngine SHALL expose a Lua global function `trace(message)` that outputs the message to the workbench diagnostic log at INFO level, prefixed with the calling script name and line number.
3. THE LuaMacroEngine SHALL expose a Lua global function `print(...)` that outputs its arguments (concatenated with tabs) to the output panel or diagnostic log, providing a convenient debugging print facility.
4. WHEN a macro is executed via the `MACRO`, `EXEC`, or `RUN` commands, THE engine SHALL measure and report the execution duration in a DEBUG-level log record (e.g., "Macro 'format_cobol' completed in 12ms").
5. THE `EXEC` command's inline evaluation mode SHALL display the return value of the expression in the status bar, formatted using Lua's `tostring()` — useful as a quick REPL for testing expressions.
6. WHEN a macro exceeds the configured instruction count limit, THE error message SHALL report how many instructions were executed before termination, aiding the developer in identifying the runaway code path.

---

## Cross-References

| Dependency | Relationship |
|------------|--------------|
| `command-framework` | The macro engine uses the Scripting_Bridge to dispatch commands from `editor.command()`. MACRO/EXEC/RUN are registered as commands. |
| `undo-redo-transactions` | All macro modifications are wrapped in a Macro_Transaction; rollback on error uses the transaction system's abort mechanism. |
| `document-model` | The `editor.*` API reads and writes buffer content through the document model's line access interface. |
| `configuration-system` | Security mode, directory paths, debug flags, auto-reload, and limit settings are read from `ff-config`. |
| `plugin-architecture` | The macro engine registers as a plugin providing `MacroCapability`; its lifecycle follows the plugin state machine. |
| `connector-local-fs` | File watching for auto-reload and directory scanning uses the local filesystem connector's watcher. |

---

---

### Requirement 11: ISPF Edit Macro API and REXX Execution Bridge

**User Story:** As a macro author, I want to invoke ISPF edit macro services (ISREDIT, ISPEXEC, IMACRO) and execute REXX execs with full host command environments, TSO built-in functions, and EXECIO I/O, so that existing ISPF macros and REXX execs can run inside FileForge Workbench with minimal modification.

**Source:** ISPF-EARS macros (ISREDIT, ISPEXEC, IMACRO, LINENUM, CURSOR), TSO-EARS REXX (REXX-1 through REXX-4), FFCMD scripting. [FFE-MVP-7, SCI-STE-LUA, WB]

#### Acceptance Criteria

1. THE LuaMacroEngine SHALL provide an `ISREDIT` host command environment that accepts ISPF Edit macro service calls as strings (e.g., `ISREDIT "CURSOR = 5 10"`) and dispatches them to the corresponding editor API operations.
2. THE LuaMacroEngine SHALL provide an `ISPEXEC` host command environment that accepts ISPF dialog service calls as strings and routes them to the appropriate workbench service (panel display, variable pool, message display).
3. WHEN the `IMACRO <name>` edit profile setting is active, THE engine SHALL automatically execute the named macro at the start of every edit session before the user gains control of the buffer.
4. THE edit profile SHALL support an `IMACRO` setting that stores the name of the initial macro to run on edit session open; WHEN set to blank, no initial macro is executed.
5. THE engine SHALL expose a `LINENUM` function that accepts a label or relative line reference and returns the absolute 1-based line number of the referenced line in the active buffer.
6. THE engine SHALL extend the existing `editor.cursor_line()` and `editor.cursor_col()` API with a `CURSOR` function that both gets and sets the cursor position: `CURSOR()` returns `(line, col)` and `CURSOR(line, col)` moves the cursor to the specified 1-based position.
7. WHEN the `EXEC <member>` command is issued, THE engine SHALL locate the named exec in the configured SYSEXEC or SYSPROC library paths and execute it as a REXX-compatible script.
8. WHEN a member name is entered on the command line without a recognized primary command prefix, THE engine SHALL attempt implicit exec invocation by searching SYSEXEC/SYSPROC for a member with that name before returning a command-not-found error.
9. WHEN a command is prefixed with `%`, THE engine SHALL bypass the primary command table and search SYSEXEC/SYSPROC directly, reducing search time for known exec names.
10. WHEN the `EXEC <member> <args>` form is used, THE engine SHALL pass the argument string to the exec as its invocation argument list, accessible via the exec's ARG instruction or equivalent.
11. THE engine SHALL support a `TSO` host command environment that routes unrecognized commands to the workbench TSO command dispatcher (ff-command), returning the command's return code in the `RC` special variable.
12. THE engine SHALL support `ADDRESS <environment-name>` syntax to switch the default host command environment for subsequent commands within the same exec invocation.
13. THE engine SHALL support an `ISPEXEC` environment name within ADDRESS that routes service calls to the ISPF dialog service layer.
14. THE engine SHALL support an `ISREDIT` environment name within ADDRESS that routes edit macro calls to the ISREDIT host command environment defined in criterion 11.1.
15. WHEN a host command completes, THE engine SHALL set the `RC` special variable to the integer return code of that command, accessible to the calling exec.
16. THE engine SHALL expose a `LISTDSI` built-in function that returns dataset information (DSORG, RECFM, LRECL, BLKSIZE, DSNAME, VOLSER, MEMBER count) for a named dataset, querying the ff-dscatalog registry.
17. THE engine SHALL expose a `MSG` built-in function that displays a message in the workbench status bar or message area, accepting a message ID or literal string.
18. THE engine SHALL expose a `MVSVAR` built-in function that returns system variable values (SYSNAME, SYSPLEX, SYSCLONE, SYSOPSYS) mapped to workbench equivalents (application name, workspace name, host OS).
19. THE engine SHALL expose an `OUTTRAP` built-in function that captures TSO command output into a Lua/REXX stem variable rather than displaying it, enabling programmatic processing of command output.
20. THE engine SHALL expose a `PROMPT` built-in function that controls whether the exec may prompt the user for input; WHEN set to OFF, any attempt to read from the terminal returns an empty string.
21. THE engine SHALL expose a `SYSDSN` built-in function that returns `OK` if the named dataset exists and is accessible in the catalog, or an error string (`DATASET NOT FOUND`, `MEMBER NOT FOUND`, etc.) otherwise.
22. THE engine SHALL expose a `SYSVAR` built-in function that returns ISPF system variable values (SYSUID, SYSDATE, SYSTIME, SYSPREF, SYSENV) mapped to workbench equivalents.
23. THE engine SHALL expose a `USERID` built-in function that returns the current user's login name as a string.
24. THE engine SHALL support `EXECIO DISKR <ddname> <count> STEM <stem>` syntax that reads up to `<count>` records from the dataset allocated to `<ddname>` into a stem variable array.
25. THE engine SHALL support `EXECIO DISKW <ddname> <count> STEM <stem>` syntax that writes `<count>` records from a stem variable array to the dataset allocated to `<ddname>`.
26. THE engine SHALL support `EXECIO * DISKR <ddname> FINIS` and `EXECIO * DISKW <ddname> FINIS` variants that read/write all remaining records and close the file.
27. THE engine SHALL support `EXECIO SKIP <ddname> <count>` that advances the read position by `<count>` records without returning data.
28. WHEN an EXECIO operation completes, THE engine SHALL set `RC` to 0 on success, 2 when end-of-file is reached before `<count>` records are read, and non-zero on I/O error, consistent with TSO EXECIO return code conventions.
29. THE engine SHALL support FFCMD command files: plain-text files with a `.ffcmd` extension containing one workbench primary command per line, executed sequentially as a batch script via the `RUN` command or the `FFCMD <path>` primary command.
30. WHEN an FFCMD file is executed, THE engine SHALL wrap the entire file execution in a single Macro_Transaction so that all document modifications are atomically undoable, consistent with Requirement 5.4.
