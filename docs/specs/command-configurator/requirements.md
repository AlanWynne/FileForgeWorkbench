# Requirements Document -- Command Configurator

## Introduction

The Command Configurator lets a user define, store, edit, and delete named
commands from within FileForge Workbench, and bind them to menu options and
keyboard shortcuts. Each definition is a named Command_Target
(command-framework Requirement 8) that may point at internal FFWB behaviour
(a menu, a custom workspace, an internal function, or a macro) or at an external
program (a PowerShell / batch / bash script, a Python script, or any executable).

Definitions are stored in a data-driven, hot-reloadable TOML file at
`<User_Data_Dir>/commands/commands.toml`, in the same spirit as the `menus/`
directory used by the Menu Workspace pattern. The Command Configurator itself is
a Custom Workspace (a new Context) that presents a form over that file.

External commands run in one of two modes:

- **Detached** -- a fire-and-forget Started_Task. FFWB spawns the process and
  forgets it: no output is captured, no Workspace is opened, and the task is
  never persisted or restarted. Monitoring a running Started_Task is out of
  scope (the operating system's own task manager owns process lifecycle).
- **Captured** -- FFWB runs the process asynchronously without blocking the UI,
  collects its stdout, stderr, and exit code, and displays them in the existing
  shell Output_Panel (shell-command Requirement 15).

All external execution is gated by the existing `shell.mode` configuration
(`disabled` | `prompt` | `enabled`, shell-command Requirement 2); the Command
Configurator introduces no separate external-execution security switch.

### Source References

- **[CR-NR-052]** = Change log entry for the Command Configurator requirement.
- **[CR-NR-051]** = Unified Command Target (command-framework Requirement 8).
- **[WB]** = Workbench Architecture Brief (command-driven, data-as-config).

### Cross-References

- **`command-framework`** Requirement 8 -- defines the Command_Target this
  sub-project stores and dispatches.
- **`shell-command`** Requirement 2 (shell.mode gate), Requirement 15
  (Output_Panel), Requirement 19 (External program execution) -- the external
  execution engine reused for the External target.
- **`menu-workspace`** Requirement 10 -- menu options may reference a
  user-defined command by its id.
- **`startup-and-session`** Requirement 21 -- the Command Configurator Context
  persists as a Custom_Workspace_Target; Started_Tasks do not persist.

---

## Glossary

| Term | Definition |
|------|-----------|
| **Command_Definition** | A named, user-authored entry pairing a Command_Id with a Command_Target and display metadata, stored in the Command_Store. |
| **Command_Store** | The TOML file `<User_Data_Dir>/commands/commands.toml` holding all Command_Definitions. |
| **Command_Configurator** | The Custom Workspace (Context) that lists, adds, edits, and deletes Command_Definitions. |
| **Started_Task** | A Detached external process spawned fire-and-forget: no capture, no Workspace, never persisted or restarted. |
| **Captured_Run** | A Captured external process run asynchronously with stdout/stderr/exit-code shown in the Output_Panel. |
| **Execution_Mode** | Detached or Captured -- an attribute of an External Command_Target. |

---

## Requirements

### Requirement 1: Command Store File Format

**User Story:** As a user, I want my custom commands saved in a readable file so
that I can back them up, share them, or edit them outside the app.

**Source:** [CR-NR-052], [WB]

#### Acceptance Criteria

1. THE Command_Store SHALL be a TOML file at `<User_Data_Dir>/commands/commands.toml` containing an array of Command_Definition tables under the key `[[command]]`.
2. EACH `[[command]]` entry SHALL contain: `id` (string, required, unique within the file, matching the Command_ID naming rule -- lowercase ASCII letters, digits, dots, underscores), `label` (string, required, human-readable), and a `[command.target]` table (required) that is a serialised Command_Target (command-framework Requirement 8.7).
3. EACH `[[command]]` entry MAY contain: `description` (string, optional) and `category` (string, optional, default `"user"`).
4. WHEN two entries share the same `id`, THE Command_Store loader SHALL keep the first, skip the duplicate, and log a WARN-level record naming the duplicate id.
5. WHEN the Command_Store file is absent, THE workbench SHALL treat the store as empty and SHALL create the `commands/` directory and an empty `commands.toml` on first save.
6. WHEN the Command_Store file contains invalid TOML or an entry is missing a required field, THE loader SHALL skip the invalid entry, retain all valid entries, and surface a load-error message identifying the offending entry.
7. WHEN a Command_Store file is written or modified on disk while the workbench is running, THE loaded definitions SHALL reload using the same hot-reload infrastructure as the Menu Workspace pattern (menu-workspace Requirement 4.3, 4.4).
8. THE Command_Store SHALL use only plain ASCII in its default/example content (documentation.md character rules).

---

### Requirement 2: Command Configurator Workspace

**User Story:** As a user, I want a screen inside FFWB where I can see all my
custom commands and add, edit, or remove them, so that I can extend the
workbench without hand-editing files.

**Source:** [CR-NR-052], [WB]

#### Acceptance Criteria

1. THE Command_Configurator SHALL be a Custom Workspace (Context) openable via a primary command (`COMMANDS`) and bindable as a Command_Target, displaying the list of Command_Definitions from the Command_Store.
2. THE Command_Configurator SHALL display, for each Command_Definition, its `id`, `label`, target variant (menu / custom-workspace / function / macro / external), and -- for External targets -- the Execution_Mode (Detached or Captured).
3. THE Command_Configurator SHALL provide an Add action that creates a new Command_Definition, a per-row Edit action that modifies an existing definition, and a per-row Delete action that removes it (with confirmation).
4. WHEN the user saves an Add or Edit, THE Command_Configurator SHALL validate the definition (Requirement 1.2 rules and Requirement 4 for External targets) and, on success, write the full Command_Store back to `commands.toml`; on validation failure it SHALL display the error and leave the file unchanged.
5. WHEN the user deletes a Command_Definition, THE Command_Configurator SHALL remove it from the Command_Store and write the file back.
6. THE Command_Configurator SHALL present the target editor as variant-specific fields: Menu (menu name), Custom Workspace (workspace kind + optional params), Function (command id + optional params), Macro (name or path), External (program, args, working directory, Execution_Mode).
7. WHEN the tab title is shown, THE Command_Configurator Context SHALL display `[COMMANDS]`.
8. WHEN F3 or `END` is entered in the Command_Configurator command field, THE shell SHALL return the Workspace to the Home Context (POM), consistent with other Custom Workspaces.

---

### Requirement 3: External Command Execution Modes

**User Story:** As a user, I want to run external scripts and programs from FFWB,
choosing whether to just launch them and forget, or to wait and see their
output, so that I can both kick off background tasks and run tools whose results
I need to read.

**Source:** [CR-NR-052], [WB]

#### Acceptance Criteria

1. AN External Command_Target SHALL carry a `program` (string, required), an `args` list (array of strings, optional), a `working_dir` (string, optional), and a `mode` that is exactly one of `detached` or `captured`.
2. WHEN an External Command_Target with `mode = detached` is executed AND `shell.mode` permits execution, THE workbench SHALL spawn the process as a Started_Task and return immediately without waiting, without capturing output, and without opening a Workspace.
3. WHEN a Started_Task is spawned, THE workbench SHALL NOT track, monitor, restart, or persist it; the process's lifecycle after spawn is owned by the operating system.
4. WHEN an External Command_Target with `mode = captured` is executed AND `shell.mode` permits execution, THE workbench SHALL run the process asynchronously (never blocking the GUI render thread), capture stdout and stderr, and display them together with the exit code in the shell Output_Panel (shell-command Requirement 15), reusing the same execution path as `shell.execute` (shell-command Requirement 19).
5. WHEN a `working_dir` is specified, THE workbench SHALL set the child process's working directory to that path; WHEN it is absent, THE workbench SHALL apply the `shell.working_directory` resolution rules (shell-command Requirement 11).
6. THE workbench SHALL expand `${workspace_root}` and `${file_dir}` placeholders in `program`, `args`, and `working_dir` to the active workspace root and active file directory respectively; an unresolved placeholder SHALL expand to an empty string and log a DEBUG-level record.
7. WHEN `shell.mode` is `disabled`, THE workbench SHALL refuse to execute any External Command_Target (Detached or Captured) and SHALL display the standard shell-disabled message (shell-command Requirement 2.2).
8. WHEN `shell.mode` is `prompt`, THE workbench SHALL display the shell confirmation dialog before executing an External Command_Target; IF the user declines, THEN the process SHALL NOT be spawned.
9. WHEN a Detached spawn fails to launch (program not found, permission denied), THE workbench SHALL display an error message identifying the program and SHALL NOT open a Workspace.
10. THE classification of an External Command_Target as a Visible_Workspace SHALL follow command-framework Requirement 8.9: Captured produces a Visible_Workspace (the Output_Panel result), Detached does not.

---

### Requirement 4: Definition Validation and Binding

**User Story:** As a user, I want the configurator to catch mistakes and let me
reuse my commands in menus and on keys, so that my custom commands are reliable
and reachable.

**Source:** [CR-NR-052], [CR-NR-051]

#### Acceptance Criteria

1. WHEN a Command_Definition is validated, THE workbench SHALL reject an empty `id`, an `id` that violates the Command_ID naming rule, an empty `label`, or a `[command.target]` that does not deserialise to a valid Command_Target, with a specific error message per failure.
2. WHEN an External target is validated, THE workbench SHALL reject an empty `program` and reject a `mode` that is not `detached` or `captured`.
3. A Command_Definition's `id` SHALL be usable as a Menu_Option `command` value (menu-workspace Requirement 10) so that selecting the option executes the definition's Command_Target.
4. A Command_Definition's `id` SHALL be bindable to a keyboard Shortcut_Binding (command-framework Requirement 8.5) so that a key or chord executes the definition's Command_Target.
5. WHEN a Menu_Option or Shortcut references a Command_Definition id that does not exist in the Command_Store, THE workbench SHALL display `Command '<id>' is not defined.` and take no further action.
6. THE workbench SHALL NOT allow a user-defined `id` to shadow a reserved built-in Command_ID; WHEN a definition uses a reserved id, THE workbench SHALL reject it with a message naming the conflict.
