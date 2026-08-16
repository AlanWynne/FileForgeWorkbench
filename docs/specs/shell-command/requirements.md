# Requirements Document

## Introduction

This feature specifies the shell command subsystem for FileForgeWorkbench (`ff-shell` crate). It provides the `SHELL` primary command (with `TSO` as an alias for ISPF compatibility) that enables users to execute operating system commands, capture output into documents, pipe document content as stdin, and open interactive terminal sessions — all without leaving the workbench.

The shell command subsystem merges the FileForgeEditor `shell-command` spec (10 requirements, all incorporated) with workbench platform enhancements:

- **Panel-based terminal** — interactive terminal sessions are hosted in the docking system as a proper `DockablePanel`, not a modal overlay
- **Command framework integration** — all shell operations are dispatched through `ff-command` and recorded in command history
- **Async execution** — long-running commands use the workbench async I/O principle (cross-cutting Requirement 6) with progress reporting via `ff-workflow`
- **Configuration system integration** — shell settings are managed through the `ff-config` layered configuration (cross-cutting Requirement 5)
- **Document piping** — document content (full or selection) can be piped as stdin to external commands
- **Environment inheritance** — child processes inherit the workbench's environment with configurable augmentation
- **Working directory** — configurable working directory (project root or active file directory)

The `ff-shell` crate is a Wave 9 (Desktop Integration) component. It depends on `ff-command` (command framework), `ff-config` (configuration system), `ff-layout` (layout and docking for terminal panel), and `ff-workflow` (async progress/cancellation). It integrates with `clipboard-operations` for output-to-clipboard workflows.

**Source references:**
- **FFE** = FileForgeEditor `shell-command` specification (10 requirements — all incorporated and adapted)
- **SCI** = SciTE job/subsystem execution model (output panel, build commands — adapted)
- **WB** = Workbench Architecture Brief §7 command-driven, §9 async I/O, §12 layout

---

## Glossary

- **SHELL**: The primary command that provides OS shell access from within the workbench. Supports interactive terminal, command execution, document capture, and stdin piping modes. Registered with `ff-command` as `"shell.execute"`. [FFE, WB]
- **TSO**: An alias for `SHELL`, provided for ISPF compatibility. Treated identically in all operational modes. [FFE]
- **Shell_Engine**: The `ff-shell` crate subsystem responsible for launching OS processes, capturing their output, managing interactive terminal sessions, and coordinating with the command framework. [FFE, WB]
- **Shell_Mode**: A configuration setting controlling whether `SHELL` is available. Values: `disabled` | `prompt` | `enabled`. Read from `ff-config`. [FFE]
- **Default_Shell**: The shell executable used when no shell override is specified. Auto-detected by platform; configurable via `ff-config`. [FFE]
- **Shell_Override**: An explicit shell name supplied as the first argument to `SHELL` before the command string (e.g., `SHELL powershell Get-Date`). [FFE]
- **Capture_Output**: The stdout text produced by an OS command that is inserted into the document in document capture mode. [FFE]
- **Target_Line**: The document line carrying an `A` or `B` line command marker at the time `SHELL` is executed. [FFE]
- **Interactive_Terminal**: A terminal session opened by `SHELL` with no arguments, hosted as a `DockablePanel` in the layout system. [FFE, WB]
- **Terminal_Panel**: The `DockablePanel` implementation that renders interactive terminal sessions within the workbench docking system. [WB]
- **Output_Panel**: A read-only panel that displays command execution output (stdout/stderr), similar to SciTE's output pane. Hosted in the docking system. [SCI, WB]
- **Command_Engine**: The `ff-command` framework that parses, validates, and executes primary and line commands. [WB]
- **A (After)**: A line command target marker designating the insertion point immediately after the marked line. [FFE]
- **B (Before)**: A line command target marker designating the insertion point immediately before the marked line. [FFE]
- **Logical_Line**: A single record in the workbench document model, as defined in `document-model` spec. [FFE]
- **Stdin_Pipe**: Document content (full buffer or current selection) piped to a child process's standard input. [WB]
- **Working_Directory**: The filesystem directory used as the current working directory for spawned child processes. [WB]
- **Environment_Augmentation**: Additional environment variables injected into child processes beyond inherited OS environment. [WB]
- **Process_Handle**: An opaque reference to a running child process, supporting status query and cancellation. [WB]
- **Exit_Code**: The integer status code returned by a completed child process. [FFE]

---

## Requirements

### Requirement 1: SHELL Command Recognition and Aliasing

**User Story:** As a workbench user, I want to type `SHELL` or `TSO` in the command line to access OS shell functionality, so that I can use either the modern cross-platform name or the familiar ISPF alias.

**Source:** FFE Requirement 1 — command recognition. [FFE-SHELL]

#### Acceptance Criteria

1. THE Command_Engine SHALL recognise `SHELL` as a valid primary command name in a case-insensitive manner and route it to the Shell_Engine.
2. THE Command_Engine SHALL recognise `TSO` as an alias for `SHELL` and treat it identically in all operational modes.
3. WHEN the `SHELL` or `TSO` command is resolved, THE Command_Engine SHALL normalise it to the canonical command ID `"shell.execute"` before dispatch.
4. WHEN `SHELL` is invoked and Shell_Mode is `disabled`, THE Command_Engine SHALL NOT execute any shell operation and SHALL display an error message stating that shell access is disabled by configuration.
5. THE Shell_Engine SHALL register the `"shell.execute"` command with the `ff-command` Command_Registry during workbench startup, including metadata (display name: "Execute Shell Command", category: "shell", description: "Run an OS command or open a terminal session").

---

### Requirement 2: Shell Mode Security Control

**User Story:** As a system administrator, I want to control whether the SHELL command is available via a dedicated configuration setting, so that I can disable shell access in regulated environments independently of the macro security policy.

**Source:** FFE Requirement 2 — security control. [FFE-SHELL]

#### Acceptance Criteria

1. THE Shell_Engine SHALL respect a `[shell]` / `mode` configuration key (path: `shell.mode`) in the `ff-config` system with three permitted values: `"disabled"`, `"prompt"`, and `"enabled"`.
2. WHEN `shell.mode` is `"disabled"`, THE Shell_Engine SHALL refuse all `SHELL` invocations and display an informative error message via the status/message area.
3. WHEN `shell.mode` is `"prompt"`, THE Shell_Engine SHALL display a confirmation dialog before executing any `SHELL` invocation; IF the user declines, THEN the command SHALL be cancelled and the document SHALL remain unmodified.
4. WHEN `shell.mode` is `"enabled"`, THE Shell_Engine SHALL execute `SHELL` invocations without prompting.
5. WHEN `shell.mode` is not set in any configuration layer, THE Shell_Engine SHALL default to `"prompt"`.
6. THE `shell.mode` setting SHALL be independent of any macro security mode; neither setting implies the other.
7. WHEN `SHELL` is invoked from a Lua macro, THE Shell_Engine SHALL require BOTH the macro security mode AND the `shell.mode` to permit execution; IF either prohibits it, THEN the invocation SHALL be refused with an appropriate error message.

---

### Requirement 3: Platform Shell Detection and Default Shell

**User Story:** As a workbench user, I want the workbench to automatically use the correct shell for my operating system, so that SHELL commands work without requiring any configuration.

**Source:** FFE Requirement 3 — platform detection. [FFE-SHELL]

#### Acceptance Criteria

1. WHEN running on Windows and no `shell.default_shell` is configured, THE Shell_Engine SHALL use `cmd.exe` as the default shell.
2. WHEN running on Linux or macOS and no `shell.default_shell` is configured, THE Shell_Engine SHALL use the value of the `$SHELL` environment variable IF it is set and the path is executable.
3. WHEN running on Linux or macOS and `$SHELL` is not set or the path is not executable, THE Shell_Engine SHALL fall back to `bash` if available on PATH, or `sh` as the final fallback.
4. WHERE a `shell.default_shell` key is present in the `ff-config` system, THE Shell_Engine SHALL use the configured value as the default shell on all platforms, overriding auto-detection.
5. WHEN a Shell_Override is specified as the first argument to `SHELL` (e.g., `SHELL powershell Get-Date`), THE Shell_Engine SHALL use the specified shell for that invocation only, without modifying the configured default.
6. WHEN the resolved shell executable is not found on the system PATH or at the configured absolute path, THE Shell_Engine SHALL NOT execute the command and SHALL display an error message identifying the missing shell executable.

---

### Requirement 4: Command Execution Mode

**User Story:** As a workbench user, I want to type `SHELL <command>` to run an OS command and see its output, so that I can run build scripts, check git status, or inspect files without leaving the workbench.

**Source:** FFE Requirement 4 — command execution. [FFE-SHELL, SCI-STE-JOBS]

#### Acceptance Criteria

1. WHEN the `SHELL` primary command is entered with one or more arguments and no `A` or `B` target line command is present, THE Shell_Engine SHALL execute the arguments as a command in the Default_Shell (or Shell_Override if supplied) and SHALL capture both stdout and stderr.
2. WHEN command execution completes, THE Shell_Engine SHALL display the combined stdout and stderr output in the Output_Panel (a dockable panel in the `ff-layout` system). IF the Output_Panel is not visible, THE Shell_Engine SHALL make it visible in its default dock zone (bottom).
3. WHEN command execution completes, THE Shell_Engine SHALL display the command's exit code in the Output_Panel header alongside the command text.
4. WHEN command execution produces no output, THE Shell_Engine SHALL display a message in the Output_Panel indicating that the command completed with no output, along with the exit code.
5. WHEN command execution is in progress, THE Shell_Engine SHALL NOT block the workbench UI; the command SHALL run asynchronously with a visible progress indicator in the status bar and the Output_Panel.
6. THE Shell_Engine SHALL impose a configurable timeout (`shell.timeout_seconds`) on command execution; WHEN the command exceeds the timeout, THE Shell_Engine SHALL terminate the process and display an error message identifying the timeout in the Output_Panel.
7. WHEN a new command is executed while the Output_Panel already contains previous output, THE Shell_Engine SHALL append the new output below a separator line showing the command and timestamp, preserving the history within the panel.

---

### Requirement 5: Document Capture Mode

**User Story:** As a workbench user, I want to type `SHELL <command>` with an `A` or `B` target marker so that the command's standard output is inserted directly into my document, so that I can capture command output as editable content.

**Source:** FFE Requirement 5 — document capture. [FFE-SHELL]

#### Acceptance Criteria

1. WHEN the `SHELL` primary command is entered with one or more arguments and exactly one `A` or `B` target line command is present, THE Shell_Engine SHALL execute the arguments as a command and SHALL capture stdout only (not stderr) for document insertion.
2. WHEN document capture mode is active and the `A` target is present, THE Command_Engine SHALL insert the captured stdout lines immediately after the Target_Line.
3. WHEN document capture mode is active and the `B` target is present, THE Command_Engine SHALL insert the captured stdout lines immediately before the Target_Line.
4. WHEN the command produces stdout output containing multiple lines separated by line endings (`LF`, `CRLF`, or `CR`), THE Command_Engine SHALL split the output into individual Logical_Lines before insertion.
5. WHEN the captured stdout ends with a trailing line ending, THE Command_Engine SHALL NOT insert an additional empty Logical_Line for that trailing terminator.
6. THE Command_Engine SHALL preserve the exact content of each Logical_Line derived from stdout without trimming or modifying whitespace.
7. WHEN document capture completes successfully, THE Command_Engine SHALL clear the resolved `A` or `B` target line command from the prefix area.
8. WHEN document capture completes successfully, THE Command_Engine SHALL display a status message indicating the number of lines inserted.
9. WHEN document capture mode is active and the command produces no stdout output, THE Command_Engine SHALL NOT modify the document and SHALL display a message indicating that the command produced no output.
10. WHEN document capture mode is active and stderr is non-empty, THE Shell_Engine SHALL display the stderr content in the Output_Panel separately from the insertion confirmation, without inserting it into the document.

---

### Requirement 6: Undo Support for Document Capture Mode

**User Story:** As a workbench user, I want the document capture insertion to be undoable, so that I can reverse an accidental or incorrect capture.

**Source:** FFE Requirement 6 — undo integration. [FFE-SHELL, WB]

#### Acceptance Criteria

1. WHEN a document capture operation via `SHELL` + `A`/`B` is executed successfully, THE Shell_Engine SHALL record the operation as a single undoable transaction via the `ff-command` undo/redo integration — the command's `Command_Result` SHALL include an `Undo_Record`.
2. WHEN the undo command is issued and the most recent recorded operation is a document capture, THE Command_Engine SHALL remove all lines that were inserted by the capture and restore the document to its pre-capture state.
3. THE undo record SHALL include metadata identifying the shell command that produced the captured output, for display in the undo history.

---

### Requirement 7: Interactive Terminal Mode

**User Story:** As a workbench user, I want to type `SHELL` with no arguments to open an interactive terminal session, so that I can run multiple commands, inspect output interactively, and then return to the editor.

**Source:** FFE Requirement 7 — interactive terminal. [FFE-SHELL, WB]

#### Acceptance Criteria

1. WHEN the `SHELL` primary command is entered with no arguments and no `A` or `B` target line command is present, THE Shell_Engine SHALL launch an interactive terminal session using the Default_Shell (or Shell_Override if supplied).
2. THE Shell_Engine SHALL present the interactive terminal as a Terminal_Panel — a `DockablePanel` registered with the `ff-layout` system. The Terminal_Panel's default dock zone SHALL be `Bottom`.
3. WHEN the interactive terminal session ends (the shell process exits or the user closes the Terminal_Panel), THE Shell_Engine SHALL return focus to the previously active editor panel.
4. WHEN the interactive terminal is active and has focus, THE Shell_Engine SHALL pass all keyboard input to the terminal process and SHALL NOT route it to the workbench command engine.
5. WHEN running on Windows, THE Shell_Engine SHALL use the platform's pseudo-console (ConPTY) API or equivalent to host the interactive terminal.
6. WHEN running on Linux or macOS, THE Shell_Engine SHALL use a PTY (pseudo-terminal) to host the interactive terminal.
7. THE Terminal_Panel SHALL support multiple concurrent terminal sessions, displayed as tabs within the panel. WHEN `SHELL` is invoked while a terminal is already open, THE Shell_Engine SHALL open a new terminal tab.
8. THE Terminal_Panel SHALL support basic terminal emulation (ANSI/VT100 escape sequences for colour, cursor positioning, and screen clearing).

---

### Requirement 8: Shell Command Error Handling

**User Story:** As a workbench user, I want clear error messages when a shell command fails to launch or produces an error, so that I understand what went wrong and can correct my input.

**Source:** FFE Requirement 8 — error handling. [FFE-SHELL]

#### Acceptance Criteria

1. WHEN the shell process fails to start due to a missing shell executable, THE Shell_Engine SHALL NOT modify the document and SHALL display an error message in the status area identifying the missing executable.
2. WHEN the shell process fails to start due to a permission or OS error, THE Shell_Engine SHALL NOT modify the document and SHALL display an error message describing the failure.
3. WHEN a command is executed in command execution mode and exits with a non-zero exit code, THE Shell_Engine SHALL display the exit code and stderr output in the Output_Panel as an error indication without treating the non-zero exit as a fatal workbench error.
4. WHEN a document capture command exits with a non-zero exit code, THE Shell_Engine SHALL NOT insert any output into the document and SHALL display an error message including the exit code and stderr output; THE Command_Engine SHALL retain the `A` or `B` target line command so the user can correct and retry.
5. IF an OS-level error occurs while reading stdout during document capture mode, THEN THE Shell_Engine SHALL NOT insert partial output into the document and SHALL display an error message describing the I/O failure.

---

### Requirement 9: Compatibility Matrix Entries

**User Story:** As a workbench developer, I want the command compatibility matrix to formally document all SHELL command forms, so that the behaviour is unambiguously specified for implementers.

**Source:** FFE Requirement 9 — compatibility matrix. [FFE-SHELL]

#### Acceptance Criteria

1. THE Command_Engine SHALL recognise `SHELL` with no arguments and no `A`/`B` target as the interactive terminal mode form (command ID: `"shell.terminal"`).
2. THE Command_Engine SHALL recognise `SHELL <command>` with no `A`/`B` target as the command execution mode form (command ID: `"shell.execute"`).
3. THE Command_Engine SHALL recognise `SHELL <command>` combined with exactly one `A` or `B` target as the document capture mode form (command ID: `"shell.capture"`).
4. THE Command_Engine SHALL treat `SHELL` combined with any `C`, `CC`, `M`, or `MM` source line commands as an invalid command form and SHALL display an error message; source line commands are incompatible with `SHELL`.
5. THE Command_Engine SHALL treat `SHELL` with no arguments combined with an `A` or `B` target as an invalid command form and SHALL display an error message stating that a command argument is required for document capture.
6. THE Command_Engine SHALL treat `SHELL` with multiple `A`/`B` targets as an invalid command form and SHALL display an error message stating that only one target is permitted.

---

### Requirement 10: Configuration Keys

**User Story:** As a workbench administrator, I want shell-related settings to be managed through the standard configuration system, so that I can control and customise shell behaviour using the same layered TOML configuration used by all other workbench subsystems.

**Source:** FFE Requirement 10 — configuration. Adapted to `ff-config` namespaced model. [FFE-SHELL, WB]

#### Acceptance Criteria

1. THE Shell_Engine SHALL read the `shell.mode` configuration key from `ff-config`; valid values are `"disabled"`, `"prompt"`, and `"enabled"`; WHEN the value is invalid, THE Shell_Engine SHALL emit a configuration warning via the logging subsystem and fall back to `"prompt"`.
2. THE Shell_Engine SHALL read the `shell.default_shell` configuration key from `ff-config` as an optional string value; WHEN present, it SHALL override platform auto-detection for all `SHELL` invocations.
3. THE Shell_Engine SHALL read the `shell.timeout_seconds` configuration key from `ff-config` as a positive integer; WHEN not set, THE Shell_Engine SHALL default to 30 seconds; WHEN the value is invalid (non-positive or non-integer), THE Shell_Engine SHALL emit a configuration warning and fall back to 30 seconds.
4. THE Shell_Engine SHALL read the `shell.working_directory` configuration key from `ff-config`; valid values are `"project_root"` and `"file_directory"`; WHEN not set, THE Shell_Engine SHALL default to `"project_root"`.
5. THE Shell_Engine SHALL support hot-reload of all `shell.*` configuration keys via the `ff-config` Reload_Callback mechanism; configuration changes SHALL take effect for the next `SHELL` invocation without workbench restart.
6. WHEN the `ff-config` system reports unknown keys within the `[shell]` table, THE Shell_Engine SHALL log a warning and continue without error.

---

### Requirement 11: Working Directory

**User Story:** As a workbench user, I want shell commands to execute in a predictable working directory, so that relative paths in my commands resolve correctly.

**Source:** Workbench architecture adaptation. [WB]

#### Acceptance Criteria

1. WHEN `shell.working_directory` is `"project_root"` and a project is open, THE Shell_Engine SHALL set the child process's working directory to the project root path.
2. WHEN `shell.working_directory` is `"project_root"` and no project is open, THE Shell_Engine SHALL fall back to the user's home directory.
3. WHEN `shell.working_directory` is `"file_directory"` and the active document has a file path, THE Shell_Engine SHALL set the child process's working directory to the parent directory of the active file.
4. WHEN `shell.working_directory` is `"file_directory"` and the active document has no file path (unsaved buffer), THE Shell_Engine SHALL fall back to the project root or home directory (in that order of precedence).
5. THE Terminal_Panel SHALL display the current working directory in its title or status area.

---

### Requirement 12: Environment Variable Inheritance

**User Story:** As a workbench user, I want shell commands to inherit my OS environment variables and allow me to add custom variables, so that build tools and scripts find the correct paths and settings.

**Source:** Workbench architecture adaptation. [WB]

#### Acceptance Criteria

1. WHEN the Shell_Engine spawns a child process, THE child process SHALL inherit the full environment of the workbench process (all OS environment variables).
2. THE Shell_Engine SHALL support an optional `shell.env` configuration table (key-value pairs) that defines additional environment variables to inject into every child process.
3. WHEN a key in `shell.env` matches an existing OS environment variable, THE configured value SHALL override the inherited value for that process.
4. THE Shell_Engine SHALL expand references to existing environment variables within `shell.env` values using platform syntax (`%VAR%` on Windows, `$VAR` or `${VAR}` on POSIX).
5. WHEN environment variable expansion references an undefined variable, THE Shell_Engine SHALL substitute an empty string and emit a DEBUG-level log message.

---

### Requirement 13: Async Execution with Progress and Cancellation

**User Story:** As a workbench user, I want to see progress while a long-running command executes and be able to cancel it, so that I'm never stuck waiting for a command that takes too long or hangs.

**Source:** Workbench cross-cutting Requirement 6 (Async I/O). [WB]

#### Acceptance Criteria

1. ALL shell command executions (command execution mode and document capture mode) SHALL run asynchronously on a background task, never blocking the GUI render thread.
2. WHEN an async shell command is running, THE Shell_Engine SHALL display an indeterminate progress indicator in the status bar showing the command text.
3. WHEN an async shell command is running, THE Shell_Engine SHALL provide a cancel action (button in the Output_Panel and/or keyboard shortcut) that terminates the child process.
4. WHEN the user triggers cancellation, THE Shell_Engine SHALL send a SIGTERM (POSIX) or TerminateProcess (Windows) to the child process, wait up to 5 seconds for exit, then send SIGKILL / force-terminate if the process has not exited.
5. WHEN a command is cancelled, THE Shell_Engine SHALL display a message indicating cancellation in the Output_Panel and SHALL NOT insert any partial output into the document (in document capture mode).
6. THE Shell_Engine SHALL stream stdout/stderr output to the Output_Panel incrementally as it becomes available, rather than waiting for process completion.
7. WHEN multiple shell commands are queued, THE Shell_Engine SHALL execute them sequentially by default; concurrent execution requires explicit user action (opening multiple terminal tabs).

---

### Requirement 14: Stdin Piping from Document

**User Story:** As a workbench user, I want to pipe document content (or my current selection) as stdin to a shell command, so that I can use external tools like `sort`, `wc`, or custom scripts to process my text.

**Source:** Workbench enhancement — not present in FFE. [WB]

#### Acceptance Criteria

1. WHEN the `SHELL` command is suffixed with the `|` pipe indicator followed by a command (e.g., `SHELL | sort`), THE Shell_Engine SHALL pipe the entire active document content as stdin to the specified command.
2. WHEN a selection is active and the `SHELL` command uses the pipe indicator, THE Shell_Engine SHALL pipe only the selected text as stdin to the command.
3. WHEN stdin piping is combined with an `A` or `B` target line command, THE Shell_Engine SHALL pipe stdin to the command AND insert the resulting stdout at the target position (combined pipe + capture mode).
4. WHEN stdin piping is used without an `A` or `B` target, THE Shell_Engine SHALL display the command output in the Output_Panel (same as standard command execution mode).
5. THE stdin content SHALL be written to the child process's stdin and the stdin handle SHALL be closed after all content is written, signalling EOF to the child process.
6. WHEN the document is empty and stdin piping is requested, THE Shell_Engine SHALL pipe an empty stdin (immediate EOF) and proceed normally.

---

### Requirement 15: Output Panel

**User Story:** As a workbench user, I want a dedicated output panel that displays shell command results with scrollback, timestamps, and click-to-navigate support, so that I can review command output history and jump to referenced files/lines.

**Source:** SciTE output pane concept, adapted for workbench docking. [SCI-STE-JOBS, WB]

#### Acceptance Criteria

1. THE Shell_Engine SHALL register an Output_Panel as a `DockablePanel` with the `ff-layout` system, with panel ID `"shell.output"` and default dock zone `Bottom`.
2. THE Output_Panel SHALL maintain a scrollback buffer of command output history, with a configurable maximum size (`shell.output_buffer_lines`, default: 10000 lines).
3. WHEN the scrollback buffer exceeds the configured maximum, THE Output_Panel SHALL discard the oldest entries to maintain the limit.
4. EACH command execution entry in the Output_Panel SHALL be prefixed with a header line showing: the command text, the working directory, and a timestamp.
5. THE Output_Panel SHALL support text selection and copy-to-clipboard of selected output text, using the `clipboard-operations` subsystem.
6. THE Output_Panel SHALL provide a "Clear" action (command ID: `"shell.output.clear"`) that empties the scrollback buffer.
7. WHEN the Output_Panel contains file path references matching the pattern `<path>:<line>` or `<path>(<line>)`, THE Output_Panel SHALL render them as navigable links that, when activated, open the referenced file at the specified line in the editor.

---

### Requirement 16: Configurable Shell Path

**User Story:** As a power user, I want to configure multiple named shell profiles (e.g., bash, PowerShell, zsh) and select between them, so that I can use different shells for different tasks without typing the full path each time.

**Source:** Workbench enhancement. [WB]

#### Acceptance Criteria

1. THE Shell_Engine SHALL support a `[shell.profiles]` configuration table where each key is a profile name and the value is a table containing at minimum a `path` key with the shell executable path.
2. WHEN a Shell_Override matches a defined profile name (e.g., `SHELL pwsh ls`), THE Shell_Engine SHALL resolve the override to the profile's configured `path` rather than treating it as a raw executable name.
3. WHEN a Shell_Override does not match any profile name, THE Shell_Engine SHALL treat it as a raw executable name and attempt to resolve it on PATH.
4. EACH shell profile MAY include an optional `args` array specifying default arguments passed to the shell before the user's command (e.g., `["-c"]` for bash, `["/C"]` for cmd.exe).
5. EACH shell profile MAY include an optional `env` table specifying additional environment variables for that profile.
6. THE Terminal_Panel SHALL allow the user to select which shell profile to use when opening a new terminal tab.


---

### Requirement 17: Exit Code Reporting

**User Story:** As a workbench user, I want the exit code of every executed command to be clearly reported, so that I can determine whether a build succeeded or failed at a glance.

**Source:** FFE Requirement 4.3, SciTE job completion reporting. [FFE-SHELL, SCI-STE-JOBS]

#### Acceptance Criteria

1. WHEN a command completes in command execution mode, THE Shell_Engine SHALL report the exit code in the Output_Panel header for that command entry, formatted as `Exit code: <N>`.
2. WHEN the exit code is zero, THE Output_Panel SHALL display the exit code with a success indicator (e.g., green colour or checkmark icon based on theme).
3. WHEN the exit code is non-zero, THE Output_Panel SHALL display the exit code with an error indicator (e.g., red colour or error icon based on theme).
4. WHEN a command is terminated by a signal (POSIX) or force-killed (Windows), THE Shell_Engine SHALL report the signal number or forced termination status in place of the exit code.
5. THE Shell_Engine SHALL emit the exit code as part of the `Command_Result` returned to the command framework, making it accessible to macros and scripting.

---

### Requirement 18: Command Timeout

**User Story:** As a workbench user, I want commands to be automatically killed if they exceed a configured time limit, so that runaway or hung processes do not consume resources indefinitely.

**Source:** FFE Requirement 4.6, Requirement 10.3. [FFE-SHELL]

#### Acceptance Criteria

1. THE Shell_Engine SHALL start a timeout timer when a command begins execution in command execution mode or document capture mode.
2. WHEN the elapsed time exceeds `shell.timeout_seconds`, THE Shell_Engine SHALL terminate the child process using the same escalation sequence as manual cancellation (SIGTERM → wait 5s → SIGKILL on POSIX; TerminateProcess on Windows).
3. WHEN a command is terminated due to timeout, THE Shell_Engine SHALL display a message in the Output_Panel stating that the command was terminated due to exceeding the configured timeout of N seconds.
4. THE timeout SHALL NOT apply to interactive terminal sessions (Terminal_Panel). Interactive terminals run indefinitely until the user exits or closes the panel.
5. WHEN `shell.timeout_seconds` is set to `0`, THE Shell_Engine SHALL disable the timeout entirely for non-interactive commands (no automatic termination).

---

## Cross-References

| Dependency | Relationship |
|------------|-------------|
| `command-framework` | Shell commands registered in Command_Registry; dispatch via `"shell.execute"`, `"shell.terminal"`, `"shell.capture"` command IDs; undo record integration for document capture. [WB] |
| `clipboard-operations` | Output_Panel supports copy-to-clipboard of selected output text. Stdin piping interacts with selection model. [FFE-SHELL] |
| `layout-and-docking` | Terminal_Panel and Output_Panel are `DockablePanel` implementations registered with the layout system; default dock zone: Bottom. [WB] |
| `configuration-system` | All `shell.*` keys managed via `ff-config` layered system; hot-reload support; namespace scoping. [WB] |
| `workflow-engine` | Async command execution uses workflow progress/cancellation primitives. [WB] |
| `undo-redo-transactions` | Document capture mode produces undo records for inserted lines. [FFE-SHELL] |
| `document-model` | Insertion of captured lines uses the document model's line insertion API. [FFE-SHELL] |
| `line-commands` | `A`/`B` target line commands used for document capture positioning. [FFE-SHELL] |
| `lua-macro-engine` | Macros can invoke `SHELL` via the scripting bridge; subject to both `shell.mode` and macro security. [FFE-SHELL] |

---

## Notes

- FFE Requirements 1–10 are all incorporated. Requirements 11–18 are new workbench-specific additions.
- The FFE concept of displaying output in the "status/message area" is enhanced to use the dockable Output_Panel for multi-line output, with single-line summaries still shown in the status bar.
- The Terminal_Panel replaces the FFE concept of a "popup overlay" with a proper dockable panel that supports multiple tabs.
- SciTE's output pane concept (job output capture, clickable error lines) is adapted as the Output_Panel with file:line navigation.
- All shell operations respect the workbench async I/O principle — the GUI thread is never blocked.
- The `shell.env` table supports per-project overrides via the `ff-config` layering (project config can add PATH entries for project-specific tools).
- Interactive terminal emulation (Requirement 7.8) targets ANSI/VT100 as a baseline; full xterm-256color support is a future enhancement.
- The stdin piping feature (Requirement 14) is new to the workbench — it was not present in the FFE shell-command spec.
