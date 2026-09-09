# Requirements Document -- Menu Workspace Pattern

## Introduction

This spec defines the **Menu Workspace** as a first-class Workspace pattern in
FileForgeWorkbench. A Menu Workspace is a Workspace whose Context is a list of
options loaded from a TOML configuration file. Each option has a short key
(1-4 characters), a command string, and a description line. The user selects an
option by typing its key into the `Command ===>` field and pressing Enter, or by
clicking the option row.

The pattern generalises the existing hardcoded Primary Option Menu (POM) and
Settings Context into a single configurable mechanism. The POM becomes an
instance of this pattern backed by `menus/pom.toml`; the Settings Context
becomes an instance backed by `menus/settings.toml`. Any number of additional
menu files can be created by the user or by plugins.

The Menu Workspace pattern is the prerequisite for Phase CV (POM Redesign) and
Phase CW (Settings as Menu Workspace). No source code changes are made until
this spec is approved and a separate implementation instruction is given.

### Design Principles

1. **Data-driven menus.** Option lists are defined in TOML files, not in Rust
   source code. Adding or reordering options requires no recompilation.
2. **Backward compatibility.** The existing POM behaviour (options 0-8, calendar,
   exit line) is preserved exactly until Phase CV explicitly revises it.
3. **Hot-reload.** Editing a menu TOML file while the workbench is running
   updates the displayed options within one frame.
4. **Chained navigation.** Options may chain to sub-menus using dotted fastpath
   notation (e.g. `=0.Themes`), consistent with the existing fastpath model.
5. **Plugin extensibility.** Plugins may register additional menu files or inject
   options into existing menus via the plugin API.

### Source References

- **[ISPF-POM]** = IBM ISPF Primary Option Menu heritage
- **[WB]** = Workbench Architecture Brief (GUI independence, plugin lifecycle)
- **[CR-NR-045]** = Change log entry for Menu Workspace Pattern requirement

### Cross-References

- **`startup-and-session`** -- Req 14.3 references the POM option list; Phase CV
  will revise that criterion to reference `menus/pom.toml`.
- **`configuration-system`** -- Req 15 defines the Settings Context; Phase CW
  will revise it to reference `menus/settings.toml`.
- **`plugin-architecture`** -- Plugin lifecycle hooks for menu injection.
- **`layout-and-docking`** -- Menu Workspace is a Workspace kind alongside
  Editor Context, File Explorer Context, etc.

---

## Glossary

| Term | Definition |
|------|-----------|
| **Menu_Workspace** | A Workspace whose Context is a list of options loaded from a Menu_File. |
| **Menu_File** | A TOML file under `menus/` in the User_Data_Dir that defines one menu's options. |
| **Menu_Option** | One entry in a Menu_File: a key (1-4 chars), a command string, and a description. |
| **Option_Key** | The short string the user types to select an option (e.g. `0`, `1`, `JES`). |
| **Option_Command** | The primary command string executed when the option is selected (e.g. `SETTINGS`, `FILES`). |
| **Option_Description** | A one-line human-readable label shown next to the Option_Key. |
| **Chained_Path** | A dotted fastpath string that navigates through nested menus (e.g. `=0.Themes`). |
| **Menu_Title** | The title line displayed at the top of a Menu_Workspace, read from the Menu_File. |
| **Default_Menu_File** | A Menu_File written automatically on first launch when no file exists at that path. |
| **Hot_Reload** | Automatic re-reading of a Menu_File when its modification timestamp changes. |

---

## Requirements

### Requirement 1: Menu File Format

**User Story:** As a user or plugin author, I want to define a menu by writing a
TOML file, so that I can customise the option list without modifying source code.

**Source:** [CR-NR-045], [WB]

#### Acceptance Criteria

1. WHEN the workbench reads a Menu_File, THE Menu_Workspace SHALL parse it as
   valid TOML with the following top-level keys:
   - `title` (string, required) -- the Menu_Title displayed at the top of the
     Workspace.
   - `[[options]]` (array of tables, required) -- the ordered list of
     Menu_Options.
2. EACH entry in `[[options]]` SHALL contain:
   - `key` (string, required) -- 1 to 4 characters, case-insensitive for
     matching, displayed in uppercase.
   - `command` (string, required) -- the primary command string executed when
     the option is selected.
   - `description` (string, required) -- a one-line label displayed next to the
     key.
3. EACH entry in `[[options]]` MAY contain:
   - `enabled` (boolean, optional, default `true`) -- when `false`, the option
     is displayed in a disabled style and cannot be selected.
   - `group` (string, optional) -- a group label used to visually separate
     options with a blank line and optional group header.
4. WHEN a Menu_File contains a key that is not listed in criteria 1-3, THE
   workbench SHALL ignore the unknown key and log a DEBUG-level record.
5. WHEN a Menu_File is absent or cannot be read, THE Menu_Workspace SHALL
   display an error message in the option area: `Menu file not found: <path>`.
6. WHEN a Menu_File contains invalid TOML or a required field is missing, THE
   Menu_Workspace SHALL display an error message: `Menu file error: <reason>`.
7. THE Menu_File path SHALL be resolved relative to the User_Data_Dir. A path
   of `menus/pom.toml` resolves to `<User_Data_Dir>/menus/pom.toml`.

---

### Requirement 2: Menu Workspace Rendering

**User Story:** As an ISPF-familiar operator, I want a Menu Workspace to look
and behave like the existing POM -- a title line, a numbered/keyed option list,
and a `Command ===>` field -- so that the pattern is immediately familiar.

**Source:** [ISPF-POM], [CR-NR-045]

#### Acceptance Criteria

1. WHEN a Menu_Workspace is the active Workspace, THE shell SHALL render the
   following elements in order from top to bottom:
   - The standard Title_Line (as defined by `menu-and-statusbar` Req 17).
   - The Menu_Title from the Menu_File, centred, in the Legacy theme's title
     colour.
   - The option list: one row per Menu_Option, each row showing the Option_Key
     left-aligned, followed by the Option_Description.
   - The `Command ===>` field.
   - The standard Key_Label_Bar.
2. EACH option row SHALL be rendered as an interactive element: the user can
   click it or tab to it and press Enter to select it.
3. WHEN an option has `enabled = false`, THE option row SHALL be rendered in a
   visually distinct disabled style (greyed out) and SHALL NOT respond to click
   or keyboard activation.
4. WHEN a `group` field is present on an option, THE renderer SHALL insert a
   blank line before the first option in each new group. An optional group
   header label MAY be rendered if the group string is non-empty.
5. THE option list SHALL be scrollable when the number of options exceeds the
   visible area.
6. WHEN the Menu_File defines zero options (empty `[[options]]` array), THE
   Menu_Workspace SHALL display the message `No options defined in this menu.`
   in the option area.

---

### Requirement 3: Option Selection and Command Dispatch

**User Story:** As an operator, I want to select a menu option by typing its key
or clicking its row, so that the associated command is executed immediately.

**Source:** [ISPF-POM], [CR-NR-045]

#### Acceptance Criteria

1. WHEN the user types an Option_Key (case-insensitive) into the `Command ===>` 
   field of a Menu_Workspace and presses Enter, THE shell SHALL execute the
   Option_Command associated with that key.
2. WHEN the user clicks an option row in a Menu_Workspace, THE shell SHALL
   execute the Option_Command for that row, identical to typing the key and
   pressing Enter.
3. WHEN the user tabs to an option row and presses Enter or Space, THE shell
   SHALL execute the Option_Command for that row.
4. WHEN the Option_Command is a primary command recognised by the shell (e.g.
   `SETTINGS`, `FILES`, `EDIT`), THE shell SHALL dispatch it through the
   standard command pipeline.
5. WHEN the Option_Command begins with `=` (fastpath notation), THE shell SHALL
   route it as a fastpath navigation command.
6. WHEN the typed key does not match any Option_Key in the current menu, THE
   shell SHALL display the message `Option '<key>' not found in this menu.` in
   the status area and leave the Workspace unchanged.
7. WHEN an option with `enabled = false` is selected by any means, THE shell
   SHALL display the message `Option '<key>' is not available.` and take no
   further action.

---

### Requirement 4: Default Menu Files and Hot-Reload

**User Story:** As a first-time user, I want the workbench to create default
menu files automatically so that the POM and Settings menus work out of the box,
and I want changes I make to those files to take effect immediately without
restarting.

**Source:** [CR-NR-045], [WB]

#### Acceptance Criteria

1. WHEN the workbench starts and `<User_Data_Dir>/menus/pom.toml` does not
   exist, THE workbench SHALL create it with the Default_POM_Content defined in
   the design document (Phase CV will define the final content; until then the
   existing hardcoded POM options are used).
2. WHEN the workbench starts and `<User_Data_Dir>/menus/settings.toml` does not
   exist, THE workbench SHALL create it with the Default_Settings_Content
   defined in the design document (Phase CW will define the final content).
3. WHEN a Menu_File is written or modified on disk while the workbench is
   running, THE Menu_Workspace backed by that file SHALL reload its option list
   within one egui frame of the modification being detected.
4. THE hot-reload mechanism SHALL use the same file-watch infrastructure as the
   configuration system (`ff-config` hot-reload) -- no separate file-watch
   thread is introduced.
5. WHEN a hot-reload produces a parse error, THE Menu_Workspace SHALL display
   the error message from Requirement 1.6 and retain the previously loaded
   option list until the file is corrected.
6. THE `menus/` directory SHALL be created automatically inside User_Data_Dir
   when it does not exist, at the same time as other User_Data_Dir
   subdirectories (Requirement 3 of `startup-and-session`).

---

### Requirement 5: Chained Navigation

**User Story:** As an operator, I want to type a dotted path like `=0.Themes`
to navigate directly to a nested menu option, consistent with the existing
fastpath model.

**Source:** [ISPF-POM] fastpath notation, [CR-NR-045]

#### Acceptance Criteria

1. WHEN the user types a Chained_Path of the form `=<key1>.<key2>` in any
   `Command ===>` field, THE shell SHALL first navigate to the menu identified
   by `<key1>` in the current menu, then immediately execute the option
   identified by `<key2>` in that sub-menu.
2. Chained_Paths SHALL support up to 4 levels of nesting
   (e.g. `=0.Themes.Dark`).
3. WHEN any segment of a Chained_Path does not match an option in its menu,
   THE shell SHALL stop at the last successfully resolved level and display the
   message `Option '<segment>' not found.` in the status area.
4. THE existing fastpath notation for POM options (e.g. `=0`, `=1`, `=2`) SHALL
   continue to work unchanged -- they are single-segment Chained_Paths.
5. A Chained_Path SHALL be equivalent to the chained `MENU` command form
   (Requirement 11.7): resolving `=<key1>.<key2>` activates the same option as
   `MENU <menu-of-key1> <key2>`, so the fastpath and the MENU-argument forms
   share one navigation-and-activation path.
6. WHEN a menu option's own `command` value is itself a chained `MENU <name>
   <key>` (or a Chained_Path), selecting that option SHALL forward the chained
   argument so that one option can navigate directly into a specific option of
   another menu (command-framework Requirement 9.7).

---

### Requirement 9: Configurable Menu Option Limits

**User Story:** As a user or plugin author, I want the number of options in a
Menu_File to be bounded by configurable limits rather than a hardcoded count,
so that I can build large menus when I need to while the workbench protects
itself against unusably large or runaway menu files.

**Source:** [CR-NR-050], [WB]

**Rationale:** The Menu Workspace pattern (Requirements 1-5) places no explicit
cap on the number of Menu_Options. The figures "9" (legacy POM) and "12" (Phase
CV POM) are the content of specific Menu_Files, not constraints of the pattern.
An unbounded list is impractical for three reasons: single-key selection
(Requirement 3.1) degrades when hundreds of keys compete; the per-frame
hot-reload poll (Requirement 4.3) re-parses the whole file on change; and a very
long flat list is a poor menu when sub-menus (Requirement 5) exist for
structure. This requirement introduces a soft advisory limit and a hard error
limit, both configurable.

#### Glossary additions

| Term | Definition |
|------|-----------|
| **Soft_Option_Limit** | The configured option count above which a Menu_File loads successfully but triggers a WARN log and an in-panel advisory. Default 64. |
| **Hard_Option_Limit** | The configured option count above which a Menu_File is rejected as a load error. Default 256. |

#### Acceptance Criteria

1. THE workbench SHALL define two configuration keys resolved through the
   layered configuration system (`configuration-system` Requirement 2):
   - `menu.soft_option_limit` (unsigned integer, default `64`) -- the
     Soft_Option_Limit.
   - `menu.hard_option_limit` (unsigned integer, default `256`) -- the
     Hard_Option_Limit.

2. WHEN a Menu_File is loaded and its `[[options]]` count is less than or equal
   to the Soft_Option_Limit, THE Menu_Workspace SHALL load and render every
   option with no warning.

3. WHEN a Menu_File is loaded and its `[[options]]` count is greater than the
   Soft_Option_Limit AND less than or equal to the Hard_Option_Limit, THE
   Menu_Workspace SHALL load and render every option, log one WARN-level record
   naming the file path and the option count, and display a non-blocking
   advisory line in the option area:
   `This menu has <count> options (advised maximum <soft>). Consider grouping options into sub-menus.`

4. WHEN a Menu_File is loaded and its `[[options]]` count is greater than the
   Hard_Option_Limit, THE Menu_Workspace SHALL reject the file with the load
   error message from Requirement 1.6, using the reason:
   `too many options: <count> exceeds hard limit <hard>`,
   and SHALL NOT render any option row.

5. WHEN `menu.hard_option_limit` is configured to a value less than
   `menu.soft_option_limit`, THE workbench SHALL treat the effective
   Soft_Option_Limit as equal to the Hard_Option_Limit (the hard limit always
   dominates) and SHALL log one WARN-level record noting the misconfiguration.

6. WHEN either limit key is absent from all configuration layers, THE workbench
   SHALL apply the default value defined in criterion 9.1.

7. WHEN either limit key is present but not a non-negative integer, THE
   workbench SHALL ignore the invalid value, apply the default from criterion
   9.1 for that key, and log one WARN-level record naming the offending key.

8. THE option-count limits SHALL be evaluated against the number of parsed
   `[[options]]` entries before any `enabled = false` filtering, so that
   disabled options count toward both limits.

9. THE limit evaluation SHALL be re-applied on every hot-reload
   (Requirement 4.3): a file edited to exceed the Hard_Option_Limit while the
   workbench is running SHALL transition to the load-error state per criterion
   9.4, and a file edited back to within limits SHALL recover on the next
   reload.

---

### Requirement 10: Menu Options Reference a Command Target

**User Story:** As a menu author, I want a menu option to be able to open
another menu, open a built-in workspace, run an internal function, run a macro,
or run an external program, so that a menu can drive any FFWB action through one
consistent mechanism.

**Source:** [CR-NR-051], [WB]

#### Acceptance Criteria

1. WHEN a Menu_Option is selected, THE shell SHALL resolve the option's
   `command` value to a Command_Target via Target_Resolution (command-framework
   Requirement 8.3) and execute that target.
2. THE existing Menu_File format (Requirement 1) SHALL remain valid unchanged: a
   `command` value that is a bare command string SHALL resolve to an equivalent
   Command_Target and produce the same observable result as before
   (command-framework Requirement 8.4).
3. A Menu_Option `command` value that equals the `id` of a user-defined
   Command_Definition (command-configurator Requirement 1) SHALL resolve to that
   definition's stored Command_Target.
4. WHEN a Menu_Option resolves to a Menu_Target, selecting it SHALL open the
   referenced Menu_Workspace, so that one menu leading to another menu is
   expressed by an option whose target is a Menu_Target (this makes the
   sub-menu-versus-custom-workspace distinction explicit in the resolved target
   rather than implicit in the command string).
5. WHEN a Menu_Option's `command` value cannot be resolved to any Command_Target,
   THE shell SHALL display the message `Option '<key>' could not be resolved: <reason>`
   in the status area and leave the Workspace unchanged (consistent with
   Requirement 3.6 for unknown keys).
6. THE Menu_File format MAY, as an alternative to a bare `command` string,
   specify an inline `[options.target]` table that is a serialised Command_Target
   (command-framework Requirement 8.7); WHEN both `command` and
   `[options.target]` are present, THE loader SHALL use `[options.target]` and
   log a DEBUG-level record noting that `command` was ignored for that option.

---

### Requirement 11: The MENU Command

**User Story:** As an operator, I want a single `MENU` command that returns me to
the Home Context or opens any named menu, so that all menus -- built-in or
user-created -- are reachable through one consistent verb.

**Source:** [CR-NR-051], [ISPF-POM], [WB]

#### Acceptance Criteria

1. WHEN the user types `MENU` with no argument in any `Command ===>` field and
   presses Enter, THE shell SHALL open (or return to) the Home Context (POM),
   which is the Menu_Workspace backed by `menus/pom.toml`.
2. WHEN the user types `MENU <name>` in any `Command ===>` field, THE shell SHALL
   open the Menu_Workspace backed by `menus/<name>.toml`; the name `POM` SHALL
   resolve to `menus/pom.toml` and `SETTINGS` SHALL resolve to
   `menus/settings.toml`, so `MENU POM` and `MENU SETTINGS` are the named forms
   of those built-in menus.
3. THE `MENU` command SHALL have identical semantics in every Context: it is a
   global command, not reinterpreted per Context. A Context that needs a
   "return to my own main panel" action SHALL use a Context-specific command
   name rather than overriding `MENU`.
4. WHEN `MENU <name>` references a `menus/<name>.toml` that does not exist, THE
   shell SHALL open the Menu_Workspace in the load-error state (Requirement 1.5)
   showing `Menu file not found: <path>`, rather than silently doing nothing.
5. A `Menu_Target { name }` (command-framework Requirement 8.1) SHALL be executed
   by invoking the `MENU <name>` command, so that the Command Configurator and
   any keyboard binding that targets a menu reuse this single command path.
6. THE `MENU` command SHALL be registered with the command framework
   (Command_ID `"menu.open"`) so it is dispatchable from the command line, menu
   options, keyboard bindings, and macros.
7. WHEN the user types `MENU <name> <option-key>` (a name followed by a further
   argument), THE shell SHALL open the Menu_Workspace `<name>` and immediately
   activate the option in that menu whose key equals `<option-key>`
   (case-insensitive), as if the user had opened the menu and selected that
   option. This is the argument-chaining form of the MENU command
   (command-framework Requirement 9). Example: `MENU SETTINGS EDITOR` opens the
   Settings_Menu and activates its `EDITOR` option.
8. THE chained form `MENU <name> <option-key>` SHALL be equivalent in observable
   result to the fastpath `=<name-fastpath>.<option-key>` (Requirement 5) and to
   typing `<option-key>` in the `Command ===>` field and pressing a function key
   bound to `MENU <name>` (command-framework Requirement 9.5): all three activate
   the same option.
9. WHEN the `<option-key>` in a chained `MENU <name> <option-key>` does not match
   any option in menu `<name>`, THE shell SHALL open the Menu_Workspace `<name>`
   and display the message `Option '<option-key>' not found.` in the status area,
   leaving the menu open (consistent with Requirement 5.3), rather than failing
   silently.
10. THE argument forwarded by a chained `MENU` invocation SHALL be the option
    key only (a single token); any further tokens SHALL be passed on to the
    activated option's own command as its argument, so deeper chains
    (e.g. `MENU <a> <b> <c>`) compose through each option's dispatch.
