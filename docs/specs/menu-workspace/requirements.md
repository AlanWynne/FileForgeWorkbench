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
   - `show_in_menu_bar` (boolean, optional, default `true`, CR-NR-080) -- when
     `false`, the option is hidden when the menu is rendered as a horizontal
     Menu_Bar (Requirement 17) but still appears in the vertical menu. Used to
     keep a terminal option like `RETURN` out of the bar. The vertical render
     ignores this flag; only the horizontal bar render honours it.
8. THE top-level table MAY contain a `show_calendar` key (boolean, optional,
   default `true`) that controls whether the shared menu renderer draws the
   calendar panel for this menu (Requirement 2). This makes the calendar a
   per-menu, config-driven choice: a menu author may set `show_calendar = false`
   to render the option columns full-width with no calendar. (CR-CH-018.) The
   POM shows the calendar by default. The compiled Settings menu default
   (Requirement 12) SHALL set `show_calendar = false` (Requirement 16.1,
   CR-CH-026); a user or author may still set it `true`, in which case
   Requirement 16 governs its visibility and Tab participation. (CR-CH-026.)
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
   following elements in order from top to bottom, using ONE shared menu
   renderer for ALL menus including the POM (CR-CH-018 -- the POM is not a
   separate bespoke renderer; it is a Menu_Workspace backed by `menus/pom.toml`
   rendered by this same renderer):
   - The standard Title_Line (as defined by `menu-and-statusbar` Req 17).
   - The Menu_Title from the Menu_File, centred, in the Legacy theme's title
     colour.
   - The option area: a THREE-COLUMN option list on the left and, when
     `show_calendar` is true (Requirement 1.8), the calendar panel on the right,
     laid out identically to the POM home screen.
   - The `Command ===>` field.
   - The standard Key_Label_Bar.
1a. THE three-column option list SHALL show, for each Menu_Option, in aligned
    columns: the Option_Key, the Option_Command, and the Option_Description
    (key | command | description). This applies to every menu including the POM
    and Settings menus, so the command each option runs is visible alongside its
    description. (CR-CH-018.)
1b. WHEN `show_calendar` is true for the menu (Requirement 1.8), THE shared
    renderer SHALL draw the calendar panel to the right of the option list --
    the same live calendar (header with `<`/`>` month navigation, day-of-week
    header, month grid with today highlighted, time, day-of-year) currently
    rendered on the POM. WHEN `show_calendar` is false, THE option columns SHALL
    use the full width and no calendar is drawn. (CR-CH-018.)
1c. THE POM (`menus/pom.toml`) and Settings (`menus/settings.toml`) SHALL be
    rendered by this shared renderer; the previous bespoke POM renderer
    (`primary_option_menu`) column+calendar layout is folded into the shared
    renderer so there is a single code path. POM options become add/remove-able
    purely by editing `menus/pom.toml`. (CR-CH-018.)
1d. THE Home Context (POM) SHALL retain its `TabKind::PrimaryOptionMenu` tab
    identity and its POM-specific chrome -- the `[POM]` tab-header title and the
    Title_Line styling (black background, blue centred text) -- even though its
    option list is now rendered by the shared menu renderer from a
    `MenuWorkspaceState` backed by `menus/pom.toml`. Rendering the POM through
    the shared renderer SHALL NOT change the tab kind, tab title, or Title_Line
    appearance. (CR-CH-018.)
1e. THE behaviour of selecting a Menu_Option -- in the POM OR any other
    Menu_Workspace, by mouse click, by typing its Option_Key, or by the keyboard
    focus ring (Enter/Space) -- SHALL be driven SOLELY by that option's
    Option_Command string dispatched through the shell command pipeline. NO
    option behaviour SHALL be keyed to the option's position, its Option_Key
    character, or the identity of the menu. There SHALL be no compiled-in
    per-option behaviour: editing the `command` value in the Menu_File is the
    only thing that changes what an option does. (Command-driven principle;
    CR-CH-018.) The pre-existing digit-keyed shell dispatch arms (e.g. matching
    `"1"`, `"6"`, `"8"` to open specific panels) that couple an option's key to
    its behaviour SHALL be removed, as they violate this principle and command
    parity (architecture-brief Principle 2).
1f. THE keyboard focus ring and Enter/Space activation on the POM (Tab/Shift+Tab
    cycling through option rows and the calendar `<`/`>` stops, as required by
    function-keys-and-history Req 16) SHALL be driven by the loaded
    `menus/pom.toml` option list, NOT by a compiled-in option array. The number
    of focusable option stops SHALL equal the number of options in the loaded
    POM menu, and activating the focused option SHALL dispatch that option's
    Option_Command exactly as a mouse click does (per Requirement 2.1e).
    (CR-CH-018.)
1g. THE terminate action SHALL be an ordinary data-driven `menus/pom.toml`
    option (default Option_Key `X`, Option_Command `RETURN`) -- NOT a bespoke
    exit line. Selecting it dispatches the `RETURN` command, which returns to the
    Home Context from any Workspace and, when issued from the POM as the only
    Workspace, terminates the application (function-keys-and-history Req 17.3/
    17.4; consistent with `=X`). The previous bespoke "Enter X to Terminate"
    line, `PomAction::Exit`, and `FocusStop::PomExit` special-casing SHALL be
    removed. (CR-CH-018.)
1h. EVERY Option_Command in the default `menus/pom.toml` SHALL resolve through
    the shell command pipeline by command NAME (independent of any key). WHERE a
    needed command name does not already resolve, the command dispatcher SHALL be
    extended to accept it (e.g. `CATALOGS` opening the File Catalogs Context). IN
    NO CASE SHALL a default POM option dispatch to an "unknown command" error.
    THE default `menus/pom.toml` SHALL contain ONLY options whose Option_Command
    maps to built, testable functionality; options for unimplemented features
    (e.g. Utilities, Compilers, Terminals, Databases, Jobs, interactive Batch)
    SHALL be omitted and added later as their functionality is built and tested.
    (CR-CH-018.)
1i. THE fastpath `=<key>` notation (e.g. `=0`, `=1`, `=2`) SHALL continue to
    resolve to "the option whose Option_Key is `<key>` in the target menu, then
    dispatch that option's Option_Command", so fastpath navigation from the Home
    Context is itself config-driven and behaves as before. (CR-CH-018.)
2. EACH option row SHALL be rendered as an interactive element: the user can
   click it or tab to it and press Enter to select it.
3. WHEN an option has `enabled = false`, THE option row SHALL be rendered in a
   visually distinct disabled style (greyed out) and SHALL NOT respond to click
   or keyboard activation.
4. WHEN a `group` field is present on an option AND the group value differs from
   the immediately preceding option's group, THE renderer SHALL insert a visual
   group boundary before that option. (REVISED by CR-CH-021: the boundary is a
   per-menu, config-driven style -- see 4a/4b -- not an unconditional divider.)
4a. THE separator style SHALL be controlled by an optional top-level Menu_File
    key `group_separator` with values: `line` (a horizontal rule), `space` (a
    blank line only), or `none` (no visual boundary). WHEN `group_separator` is
    absent, THE default SHALL be `space` (a blank line), matching the ISPF-style
    grouped-list look and avoiding a heavy divider. A boundary is only ever
    drawn between two DIFFERENT non-empty groups (never before the first option,
    and never for options that carry no `group`).
4b. WHEN an option is the first of a new non-empty group AND that group value
    has not already been shown as a header in this menu, THE renderer MAY render
    the group value as a header label above the group when the Menu_File sets
    the optional top-level key `group_headers = true` (default `false`, so
    existing menus are unchanged). The header uses the menu's description colour.
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
6. WHEN the typed string does not match any Option_Key in the current menu, THE
   shell SHALL NOT immediately error; instead it SHALL fall through to the
   remaining stages of the command-resolution chain (command-framework
   Requirement 8.3: built-in command / Command_ID, then Menu_Name, then Macro).
   (REVISED by CR-CH-025: the current-menu Option_Key lookup is the FIRST stage of
   the unified chain, not a terminal check. Only when NO stage resolves the string
   SHALL the shell display an unresolved-command error naming the string, leaving
   the Workspace unchanged.) A single-token string that looks like an Option_Key
   but is absent from the current menu MAY still resolve as a built-in command or
   a Menu_Name; a genuinely unknown token produces the unresolved-command error.
7. WHEN an option with `enabled = false` is selected by any means, THE shell
   SHALL display the message `Option '<key>' is not available.` and take no
   further action.

> **CR-CH-043 note:** All means of selection in criteria 1-3 (typed key, click,
> Tab + Enter/Space) resolve to ONE Option_Dispatch_Path -- executing the
> option's Option_Command through the standard pipeline -- shared by the POM and
> every other menu. See Requirement 19.

---

### Requirement 4: Default Menu Files and Hot-Reload

**User Story:** As a user, I want the POM and Settings menus to work out of the
box from compiled built-in content (never written to disk), and I want any menu
file I create to take effect immediately without restarting.

**Source:** [CR-NR-045], [WB], [CR-CH-021]

#### Acceptance Criteria

1. THE workbench SHALL NOT materialise the built-in menus (`pom.toml`,
   `settings.toml`) to `<User_Data_Dir>/menus/` on first launch or at any other
   time. The built-in POM and Settings content are COMPILED-ONLY (the
   Recovery_Baseline, Requirement 12). (REVISED by CR-CH-021 -- this supersedes
   the earlier requirement to write `pom.toml`/`settings.toml` on first launch;
   it mirrors the themes code-only rule, theme-and-appearance Req 18.2/19.2.)
2. THE `menus/` directory SHALL contain ONLY user-authored Menu_Files. It MAY be
   empty. When a user Menu_File exists at a well-known path (`menus/pom.toml`,
   `menus/settings.toml`, or `menus/<name>.toml`) and parses successfully, it
   OVERRIDES the compiled built-in for that menu. (REVISED by CR-CH-021.)
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

**Source:** [ISPF-POM] fastpath notation, [CR-NR-045], [CR-NR-057]

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
7. THE leading `=` in a Chained_Path SHALL mean "begin navigation from the POM
   (Home Context)", making the POM the Navigation_Origin (command-framework
   Requirement 10) regardless of the Workspace that is currently active.
8. WHEN the user types `=0.E` from any Workspace, THE shell SHALL open the Editor
   configuration Context with intermediate Contexts collapsed (STOP), so that
   pressing END returns directly to the POM.
9. WHEN the user types `=0;E` from any Workspace, THE shell SHALL open the Editor
   configuration Context with each intermediate Context pushed (PUSH), so that
   pressing END returns to the Settings_Menu and a second END returns to the POM.
10. WHEN a navigation command does not begin with `=` (for example `EDITOR` typed
    from an Edit Workspace), THE shell SHALL use the current Workspace as the
    Navigation_Origin, so that pressing END returns to that Workspace; a
    multi-hop non-`=` chain (for example `SETTINGS ; EDITOR`) SHALL push each
    intermediate Context exactly as an `=` chain does, with the current Workspace
    at the bottom of the stack.
11. A Chained_Path MAY mix separators (for example `=0;E.T`); EACH separator SHALL
    independently determine the push (`;`) or collapse (`.`) behaviour of the
    segment it precedes (command-framework Requirement 10, criteria 5 and 6).
12. THE fastpath Chained_Path form and the chained `MENU` command form
    (Requirement 11.7) SHALL share one navigation-and-activation helper that
    applies the separator semantics, so the two notations cannot diverge
    (restating Requirement 5.5 for the separator/stack behaviour).

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
   Requirement 8.3) and execute that target. (REVISED by CR-CH-043: this
   resolution happens on the single Option_Dispatch_Path via `handle_command`,
   shared by every means of selection and by the POM and every other menu -- the
   Menu Workspace does not resolve the target kind itself; see Requirement 19.)
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

> **CR-CH-043 note:** The inline `[options.target]` capability (criterion 6) is
> preserved, but it is applied within the single command pipeline rather than by
> a click-only dispatch fork (Requirement 19.3; command-framework Requirement
> 8.6). A menu option that opens another menu (criterion 4, a Menu_Target) does
> so because the COMMAND it runs owns that effect, not because the Menu Workspace
> inspects the target (Requirement 19.4-19.5).

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
11. **(CR-CH-025 -- keyword-less menu-name form.)** WHEN the user types a bare
    token in any `Command ===>` field that is NOT claimed by an earlier stage of
    the command-resolution chain (command-framework Requirement 8.3: current-menu
    Option_Key, then built-in command / Command_ID) AND that token matches a
    resolvable Menu_Name (a user `menus/<name>.toml` that exists, or a compiled
    built-in menu name `POM`/`SETTINGS`), THE shell SHALL open that Menu_Workspace
    exactly as `MENU <name>` does -- WITHOUT requiring the `MENU` keyword. The
    keyword-less form `<name> <option-key>` SHALL be equivalent to
    `MENU <name> <option-key>` (criterion 7): e.g. `SETTINGS` opens the Settings
    menu and `SETTINGS T` opens it and activates option `T`. This is what allows
    the hardcoded `SETTINGS` / `SETTINGS <ns>` intercepts to be REMOVED: `SETTINGS`
    is resolved as a Menu_Name like any other menu (including user-created menus),
    so no menu is a special case. The `MENU <name>` explicit form (criterion 2)
    continues to work unchanged for disambiguation.
12. **(CR-CH-025 -- built-in command precedence.)** WHERE a Menu_Name would
    collide with a built-in command name (command-framework Requirement 8.10
    shadowing rule), the BUILT-IN command SHALL win; a user cannot shadow a core
    verb by naming a menu after it. The keyword-less menu-name form (criterion 11)
    resolves ONLY when no earlier chain stage claims the token.

---

### Requirement 12: Recovery Baseline (Barebones Menus)

**User Story:** As an operator whose menu configuration is missing, deleted, or
corrupted, I want the workbench to open with a minimal built-in menu set so that
I can always reach Settings, Catalogs, Files, the event Log, and the Menus editor
to rebuild or recover my configuration, instead of being locked out.

**Source:** [CR-CH-021]

#### Acceptance Criteria

1. THE workbench SHALL define a compiled Recovery_Baseline for the POM and for
   the Settings menu. These are the code-only built-in menus (Requirement 4.1)
   and are never written to disk.
2. THE Recovery_Baseline POM SHALL contain exactly these options, in order
   (REVISED by CR-NR-080): `0` -> `SETTINGS` (Settings), `1` -> `CATALOGS`
   (Catalogs), `2` -> `FILES` (Files), `3` -> `HELP` (Help), `X` -> `RETURN`
   (Return / exit when last, marked `show_in_menu_bar = false` so it is hidden
   from the horizontal Menu_Bar, Requirement 17.2a). All in a single group (no
   stray boundary). `MENUS` is NOT a POM option -- it belongs to the Settings
   menu (criterion 12.3); `LOG` was dropped from the barebones set (still
   reachable via the `LOG` command and Settings). This barebones POM is also the
   Default_Menu_Bar (Requirement 17.2).
3. THE Recovery_Baseline Settings menu SHALL contain exactly these options, in
   order (REVISED by CR-CH-025 -- reordered, regrouped, and repointed; REVISED
   again by CR-CH-029 -- added the `K` -> `KEYS` option):
   group `Core` -- `A` -> `CONFIG` (All settings -- browse every configuration
   key), `T` -> `THEME` (Theme editor -- copy, edit, save and select themes),
   `M` -> `MENUS` (Menus editor -- create, change and save menus), `K` -> `KEYS`
   (Keys -- the Key assignments editor, function-keys-and-history Requirement 22);
   then group `Recovery` -- `R` -> `RESET BARE` (Reset to barebones -- archive
   config and start fresh). NOTES: (a) the former `A` -> `A` opaque command is
   replaced by `A` -> `CONFIG` (configuration-system Requirement 15; the flat
   config-key browser is now the first-class `CONFIG` command); (b) `T` -> `THEME`
   corrects the stale `THEMES` (removed by CR-CH-024); (c) `K` -> `KEYS` opens the
   Keys Workspace (CR-CH-029); (d) the two groups render with a
   boundary between them (Requirement 2.4). Built-ins remain code-only
   (Requirement 4.1); a user may Save this menu via the Menus Editor to obtain an
   editable `menus/settings.toml` override.
4. WHEN a user Menu_File for the POM or Settings is ABSENT, THE workbench SHALL
   render the corresponding Recovery_Baseline (the compiled built-in), without a
   load error, so the menu always has its options.
5. WHEN a user Menu_File for the POM or Settings is PRESENT but fails to parse
   (invalid TOML, missing required field, or exceeds the hard option limit), THE
   workbench SHALL fall back to the corresponding Recovery_Baseline AND surface a
   non-blocking notice identifying the file and the parse error, so the operator
   knows their file was bypassed and why. (This closes the current gap where a
   corrupt `settings.toml` shows no options and no fallback.)
6. WHEN the `MENUS` command is invoked before the Menus editor Workspace exists
   (it is delivered by a separate change request), THE shell SHALL display a
   non-blocking notice `Menus editor is not yet available.` and leave the current
   Workspace unchanged. The `MENUS` command name is reserved by this requirement
   so the Recovery_Baseline rows resolve.
7. THE Recovery_Baseline SHALL be the single source of the compiled default menu
   content; the POM fastpath resolver and the Settings opener SHALL both use it
   as their fallback (no duplicate hardcoded option lists).

---

### Requirement 13: Menus Editor Context

**User Story:** As an operator, I want an in-app Menus editor so that I can
create, change, reorder and save my menus (the POM, Settings, and any custom
menu) without hand-editing TOML files, and have my changes take effect
immediately.

**Source:** [CR-NR-075]; owner request during CR-CH-021 ("The Menus Option will
open a Workspace to create, change and save Menus"); reserved by Requirement 12.6.

#### Acceptance Criteria

1. WHEN the `MENUS` command is invoked, THE shell SHALL open the Menus Editor
   Context (title `[MENUS]`). On a POM tab it transforms in place (so END/RETURN
   returns to the POM); otherwise it opens or activates a dedicated tab. This
   replaces the "not yet available" placeholder of Requirement 12.6.
2. THE Menus Editor SHALL present a menu SELECTOR listing the editable menus:
   the built-in names `POM` and `Settings`, plus every user `menus/<name>.toml`
   discovered on disk. Selecting an entry loads it as the working copy.
3. WHEN a selected built-in menu (`POM` / `Settings`) has NO user file on disk,
   THE editor SHALL load the compiled Recovery_Baseline (Requirement 12) as the
   working copy, so the user starts from the current built-in content.
4. THE Menus Editor SHALL display the working menu's options in an ORDERED,
   editable list; for each option THE editor SHALL allow editing the Option_Key,
   Option_Command, Option_Description, the `enabled` flag, and the `group` label.
5. THE Menus Editor SHALL allow ADDING a new option, DELETING an option, and
   MOVING an option up or down to change its order.
6. THE Menus Editor SHALL allow editing the menu Title and the per-menu display
   settings: `show_calendar` (Requirement 1.8), `group_separator`
   (Requirement 4a: line / space / none), and `group_headers` (Requirement 4b).
7. WHEN the user requests Save, THE editor SHALL VALIDATE the working menu with
   the same rules as the loader (Requirement 1.2/1.3, 9.4): each Option_Key is
   1-4 characters (stored uppercase), each Option_Command and Option_Description
   is non-empty, and the option count does not exceed the hard limit. WHEN
   validation fails, THE editor SHALL display the first failure inline and SHALL
   NOT write the file.
8. WHEN validation passes and the user requests Save, THE editor SHALL SERIALISE
   the working menu to TOML and write it to `menus/<name>.toml` under the
   User_Data_Dir. Saving a built-in name (`POM` / `Settings`) writes
   `menus/pom.toml` / `menus/settings.toml`; that saved file is a USER OVERRIDE
   that the renderer then prefers over the compiled Recovery_Baseline
   (consistent with Requirement 4.2). The built-in remains a code-only fallback
   (a Save does not "materialise a default"; deleting the file restores the
   baseline).
9. THE Menus Editor SHALL support Save As `<new-name>`, writing a new
   `menus/<new-name>.toml` and selecting it as the working copy.
10. THE serialised TOML SHALL round-trip: loading a file the editor wrote SHALL
    reproduce an equal `MenuFile` (title, ordered options with all fields, and
    the display settings), consistent with the loader (Requirement 1.1-1.4).
11. WHEN a menu is saved, THE corresponding open Menu_Workspace (including the
    POM or Settings) SHALL reflect the change within the existing hot-reload
    window (Requirement 4.3), so edits take effect without a restart.
12. THE Menus Editor render SHALL be a pure function returning an editor Action;
    all file writes and state changes SHALL be applied by the shell command
    layer (mirroring the Theme editor, theme-and-appearance Requirement 20), so
    the render has no side effects. A button click SHALL take priority over a
    same-frame text-field commit (the B052 two-slot rule).
13. THE Menus Editor SHALL NOT be able to save a menu that the loader would
    reject; the editor's validation and the loader's validation SHALL share one
    rule set so the editor cannot produce an unloadable file.

---

### Requirement 14: Per-Tab Navigation Stack

**User Story:** As an operator, I want each Workspace to remember the path of
Contexts I traversed to reach the current one, so that END always walks me back
up that path one level at a time, and so that navigating never unexpectedly
opens a new tab.

**Source:** [CR-CH-022]; owner ("Every Workspace should surely have a path
variable store... Stack per tab; Transform in place... the only time a new tab
is created is if we type start"); reconciles Requirement 5 (Chained Navigation).

#### Glossary additions

| Term | Definition |
|------|-----------|
| **Navigation_Stack** | An ordered, per-tab list of `WorkspaceDescriptor` entries recording the Contexts traversed to reach the current one. The current Context is NOT on the stack; the stack holds only the ancestors, most-recent last. |
| **Navigate_Here** | The single operation that changes a tab's Context: it pushes the current Context's descriptor onto that tab's Navigation_Stack and transforms the tab in place to the new Context. |

#### Acceptance Criteria

1. EACH Workspace tab SHALL own its own Navigation_Stack. Navigation state is
   per-tab, never global; two tabs have independent stacks.
2. WHEN the user navigates from the current Context to another Context (by any
   means: a menu option, a command such as `SETTINGS`/`FILES`/`MENUS`, a POM
   fastpath key, or a chained path), THE shell SHALL transform the CURRENT tab
   in place to the new Context and SHALL NOT open a new tab. Navigation never
   creates a tab (see Requirement 14.8 for the sole exception, START).
3. WHEN a `;` (PUSH) navigation step occurs, THE shell SHALL push the current
   Context's descriptor onto the tab's Navigation_Stack before transforming.
   WHEN a `.` (collapse / STOP) navigation step occurs, THE shell SHALL transform
   without pushing (the intermediate Context is collapsed). A bare
   single-Context navigation (e.g. `SETTINGS`) is a PUSH of the current Context.
   This implements the separator semantics of Requirement 5.8-5.11.
4. WHEN the user issues END (or F3), THE shell SHALL POP the top entry of the
   active tab's Navigation_Stack and reconstruct that Context in place on the
   same tab (restoring the parent). END pops exactly one level per press.
5. WHEN the user issues END (or F3) AND the active tab's Navigation_Stack is
   EMPTY, THE shell SHALL close the Workspace (the tab). WHEN that tab is the
   last open Workspace, THE shell SHALL terminate the application instead
   (preserving CR-CH-016). This replaces the POM-specific and
   per-context END rules.
6. THE Navigation_Stack entries SHALL be `WorkspaceDescriptor` values carrying
   the kind and the params needed to reconstruct the Context in place (e.g. the
   Menu name for a Menu_Workspace, the namespace for a Settings view). Because
   most Context editing state is shell-global (only `menu_workspace` state is
   per-tab), reconstruction SHALL re-derive the shared Context state from the
   descriptor exactly as opening that Context does.
7. THE leading `=` of a chained path SHALL set the tab's Navigation_Origin to the
   POM: before applying the path segments, THE shell SHALL reset the active tab
   to the POM Context with an empty Navigation_Stack, then apply each segment
   per criterion 14.3. A non-`=` navigation SHALL keep the current Context as the
   origin (its descriptor becomes the bottom of the stack on the first PUSH).
   This restates Requirement 5.7/5.10 in stack terms.
8. THE `START` command SHALL be the ONLY command that creates a new tab (a new
   Workspace with its own Navigation_Stack). Its forms:
   (CR-CH-042: a first-class `POM` command opens/returns to the Home Context and
   `START` is accepted as its alias; the POM tab derives its short-form `POM`
   label from this command -- see Requirement 20.5/20.6. The START tab-creation
   forms below are unchanged.)
   - `START` (no argument): a new tab rooted at the POM Context, Navigation_Stack
     empty (END closes it, or exits when last).
   - `START =<path>`: a new tab rooted at the POM, then navigated along `<path>`
     applying criterion 14.3/14.7 (the POM is on the stack, so END walks back to
     the POM then closes).
   - `START <arg>` / `START <name>` (no leading `=`): a new tab rooted DIRECTLY
     at the Context named by `<arg>` (e.g. `START Settings`, `START 0`), with an
     EMPTY Navigation_Stack (no POM beneath it), so END from that Context ends
     the Workspace (exits when last).
9. WHEN `START <arg>` names a POM option key or a known Context/command, THE
   shell SHALL resolve it to that Context (reusing the POM option resolution and
   command routing) and root the new tab there. WHEN `<arg>` cannot be resolved,
   THE shell SHALL open the new tab at the POM and report the unresolved argument
   in the status area.
10. THE RETURN command SHALL remain distinct from END, and (REVISED by CR-CH-038)
    SHALL target the POM (Home Context) rather than the tab's arbitrary root:
    - WHEN RETURN is issued in a Workspace that is NOT the Home Context (POM),
      THE shell SHALL navigate that Workspace to its POM (Home Context) in one
      step, clearing its Navigation_Stack, REGARDLESS of the stack depth or
      whether the Workspace was rooted directly (e.g. via `START <arg>`). The
      Workspace stays open, now showing the POM.
    - WHEN RETURN is issued while the active Workspace IS a POM, THE shell SHALL
      close that ONE Workspace; when it is the last open Workspace the
      application terminates (Option A: one Workspace closed per RETURN, NOT a
      recursive tear-down), preserving Requirement 17.3/17.4 (CR-CH-016).
    END is unchanged: it pops ONE Navigation_Stack level, and at a Workspace root
    closes that one Workspace (walk-back one step at a time). This RETURN
    behaviour is IDENTICAL in a docked Workspace and a Detached_Workspace
    (menu-and-statusbar Requirement 18.3/18.11).
11. THE three former ad-hoc END mechanisms -- the global `pending_return_to_pom`
    flag, the `settings_panel.namespace_filter`-based END branch, and the
    `menus_editor_panel.opened_from_settings` boolean (B053) -- SHALL be removed
    and replaced by the single Navigation_Stack pop of criterion 14.4/14.5.
12. THE tab Title_Line and tab-header title SHALL reflect the current Context
    after every Navigate_Here and every END pop, so the header never goes stale
    (consistent with menu-and-statusbar Req 17; related to B050).

---

### Requirement 15: Menu Workspace Tab Order and Calendar Navigation

**User Story:** As a keyboard-centric operator, I want Tab in any Menu_Workspace (the POM,
Settings, or any custom menu) to move from the command line through each selectable option and
then, when the calendar is shown, to the calendar's previous/next buttons, so that I can reach
every interactive control with the keyboard without a per-menu focus ring.

**Source:** [CR-CH-023]; owner ("we should not need a focus tab ring for any menu workspace...
In a Menu workspace the tab order should be from Command line to each option in the menu, then
if the calendar is visible we should be able to tab to the left and right button on the
calendar"). Implements the shared model of menu-and-statusbar Requirement 16 for Menu_Workspaces.

#### Acceptance Criteria

1. THE Menu_Workspace SHALL rely on the shared shell Boundary_Policy (menu-and-statusbar
   Requirement 16) for entry from and return to the Primary_Command_Field; it SHALL NOT
   define its own focus ring.

2. WHEN a Menu_Workspace is the active Workspace and the user presses Tab from the
   Primary_Command_Field, THE focus SHALL move to the FIRST enabled Menu_Option row.

3. WHEN an enabled Menu_Option row has focus and the user presses Tab (forward), THE focus
   SHALL advance to the NEXT enabled Menu_Option row in the menu's declared order.

4. A disabled Menu_Option (Requirement 1.3, `enabled = false`) SHALL NOT be a keyboard focus
   stop; Tab SHALL skip it (it is rendered as a non-interactive Label, Requirement 2.3).

5. WHEN the calendar is DISPLAYED (`show_calendar` true AND it fits, Requirement 16.2) AND the
   LAST enabled Menu_Option row has focus and the user presses Tab (forward), THE focus SHALL
   move to the calendar previous-month (`<`) button, then on the next Tab to the calendar
   next-month (`>`) button, and then on the next Tab to the first Menu_Bar item (per the
   shared Boundary_Policy).

6. WHEN the calendar is NOT displayed (either `show_calendar` false OR it cannot fit and is
   omitted per Requirement 16.3), THE last enabled Menu_Option row SHALL be the last
   Interior_Control: Tab from it moves directly to the first Menu_Bar item (no calendar stops).
   The omitted calendar's `<`/`>` ids SHALL NOT appear in the reported interior focus contract
   (Requirement 16.4).

7. THE calendar previous-month and next-month controls SHALL be rendered as REAL focusable
   buttons (`<` and `>`), so that they participate in the toolkit's native Tab traversal as
   ordinary Interior_Controls. This replaces the prior single painted calendar widget whose
   month navigation was reachable only by clicking a hit-region.

8. WHEN the calendar `<` button has focus and is activated by Enter, Space, or a mouse click,
   THE calendar SHALL navigate to the previous month. WHEN the calendar `>` button has focus
   and is activated by Enter, Space, or a mouse click, THE calendar SHALL navigate to the next
   month. Existing mouse click-through-hit-region behaviour, if retained, SHALL produce the
   same month change.

9. Shift+Tab (Back Tab) within a Menu_Workspace SHALL be the exact reverse of criteria 2-6:
   from the calendar `>` button to the calendar `<` button, from `<` to the last enabled
   option, from the first enabled option to the Primary_Command_Field.

10. THE calendar SHALL be kept minimal for now: only the `<`/`>` month-navigation buttons are
    focusable Interior_Controls; individual day cells are NOT focus stops. (Future calendar
    enhancements are out of scope for CR-CH-023.)

11. THE POM (`TabKind::PrimaryOptionMenu`) and every other Menu_Workspace SHALL obtain this
    Tab order from the shared renderer and the shell Boundary_Policy alone; adding, removing,
    or reordering options by editing the Menu_File SHALL change the Tab order accordingly with
    no code change (consistent with Requirement 2.1c).

---

### Requirement 16: Calendar Visibility and Fit (no off-screen phantom Tab stops)

**User Story:** As an operator, I want a menu's calendar, when I turn it on, to
actually be visible and its month-navigation buttons reachable; and when it is
off (or the workspace is too narrow to fit it) I do not want it leaving invisible
Tab stops after the last option.

**Source:** [CR-CH-026]; owner ("in the settings menu, after the last item in the
menu there are 2 invisible tabs ... I think these are the Calendar tabs but
calendar is hidden"; "the calendar for the settings menu says true [but] the
calendar is invisible or not displayed"; "the Settings menu will default to show
calendar as false, but if i change it to true the calendar must become visible.
if i create other menus the calendar option should remain optional and if
selected be visible"). Fixes B060. Relates to Requirement 15 (Tab order) and the
deferred CR-NR-059 calendar-responsive-hide.

Background (root cause of B060): the shared renderer laid the calendar to the
RIGHT of the option list at the option column's natural width plus a fixed gap,
a position that did NOT reflow to the visible panel width. In a workspace
narrower than the option-list natural width the calendar was drawn past the
right clip edge -- invisible, yet its `<`/`>` buttons remained focusable, so
Tab reached two "phantom" stops after the last option. This was confirmed
empirically (a throwaway harness probe): the `>` button stayed at a fixed x
regardless of panel width, falling outside the clip rect at narrow widths.

#### Acceptance Criteria

1. THE compiled Settings menu default (Requirement 12) SHALL set
   `show_calendar = false`, so the Settings Menu_Workspace shows no calendar and
   has no calendar Tab stops by default.

2. WHEN a menu has `show_calendar = true` AND the Menu_Workspace has enough
   horizontal room for the calendar alongside the option list, THE renderer
   SHALL lay out the calendar ENTIRELY within the visible (clip) width, with its
   `<` and `>` month-navigation buttons fully on-screen and reachable, for the
   POM, Settings, and any custom menu alike.

3. WHEN a menu has `show_calendar = true` BUT the Menu_Workspace is too narrow
   to fit the calendar alongside the option list (below the description-driven
   threshold of criterion 16.8, REVISED by CR-CH-032 -- formerly a fixed minimum
   constant), THE renderer SHALL OMIT the calendar for that frame (rather than
   draw it clipped or off-screen) and SHALL restore it on a later frame once
   there is room (based solely on the current frame's available width, no
   persisted state).

4. WHEN the calendar is omitted for either reason in criterion 3 OR because
   `show_calendar = false`, THE renderer SHALL NOT include the calendar `<`/`>`
   button ids in the reported interior focus contract
   (`first_interior_id`/`last_interior_id`), so the calendar contributes ZERO
   Tab stops: the last enabled Menu_Option row is the last Interior_Control
   (consistent with Requirement 15.6).

5. WHEN the calendar IS displayed (criterion 2), THE reported last
   Interior_Control SHALL be the calendar next-month (`>`) button, and that
   button's on-screen position SHALL be within the visible width (it SHALL NOT
   be an off-screen focus stop). This is the invariant B060 violated.

6. THE calendar, when displayed, SHALL NOT visually overlap the option list; the
   option list SHALL retain at least its readable natural width (or a scroll
   region) and the calendar SHALL occupy a reserved column to its right within
   the visible area.

#### Description-driven layout (CR-CH-032)

The following criteria REVISE how the calendar-fit decision and the option-column
width are computed. They REPLACE the fixed-minimum-constant rule that criteria
16.3 and 16.6 originally relied on (a constant `OPTION_LIST_MIN_WIDTH` reserve)
with a decision DRIVEN BY the descriptions' natural one-line width. Criteria 16.1,
16.2, 16.4, 16.5 (calendar default off; on-screen when shown; no phantom Tab
stops when omitted; last Interior_Control is the `>` button) are UNCHANGED and
continue to hold under the new decision.

**Glossary additions:**
- **Natural_Option_Width** -- the width (px) the option list needs to render its
  WIDEST row entirely on ONE line: the fixed key+command prefix (Requirement
  2.1a) plus the widest single-line description, measured with no wrapping, plus
  an allowance for the option-list vertical scrollbar when one may be shown.
- **Layout_Tier** -- the per-frame layout choice among Tier 1 (one-line
  descriptions + calendar), Tier 2 (one-line descriptions, calendar hidden), and
  Tier 3 (wrapped descriptions, calendar hidden), selected by criteria 16.8.

7. THE renderer SHALL compute the Natural_Option_Width each frame from the menu's
   options (widest key+command prefix + widest single-line description + the
   option-list scrollbar allowance). The Natural_Option_Width SHALL be UNCAPPED
   (a very long single description simply increases it; there is no maximum after
   which a description is forced to wrap while width is still available).

8. THE renderer SHALL select the Layout_Tier for the frame from the available
   width `W`, the Natural_Option_Width `N`, the calendar gap `G`, the calendar
   minimum width `C`, and `show_calendar`, in this PRIORITY ORDER:
   - **Tier 1** WHEN `show_calendar` is true AND `N + G + C <= W`: show the
     calendar; size the option column to `N`; lay the calendar immediately to
     the right of the option column; any remaining width (`W - N - G -
     calendar_width`) is left as blank space to the RIGHT of the calendar (the
     calendar is NOT pinned to the right window edge).
   - **Tier 2** OTHERWISE WHEN `N <= W`: HIDE the calendar and give the option
     column the FULL available width `W`; descriptions remain on one line.
   - **Tier 3** OTHERWISE (`N > W`, even with the calendar hidden): hide the
     calendar, give the option column the full available width `W`, and allow
     descriptions to WRAP to further lines.

9. THE calendar-hide-before-description-wrap ordering of criterion 16.8 SHALL be
   strict: descriptions SHALL NOT wrap while the calendar is still shown. That
   is, the renderer SHALL never be in a state where the calendar is displayed AND
   any description is wrapped -- hiding the calendar (Tier 2) always precedes
   wrapping (Tier 3).

10. IN Tiers 1 and 2 THE option column SHALL be allocated at least the
    Natural_Option_Width, so no description wraps. The description control SHALL
    RETAIN its wrapping capability (Requirement 2.1a, B065) as a fault-tolerant
    fallback: "descriptions do not wrap in Tiers 1 and 2" is a CONSEQUENCE of the
    width allocation, NOT a hard non-wrapping mode -- a width mis-measurement
    SHALL degrade gracefully into a wrap rather than clip or overflow text off
    the visible edge.

11. THE Layout_Tier SHALL be recomputed each frame from the current available
    width only (no persisted state), so widening or narrowing the window moves
    between tiers reactively (e.g. narrowing crosses Tier 1 -> Tier 2 -> Tier 3;
    widening reverses it), consistent with the no-persisted-state rule of
    criterion 16.3.

---

### Requirement 17: Configurable Named Menu Bars (a menu rendered horizontally)

**User Story:** As an operator, I want the application menu bar to be a menu like
any other -- configurable, savable, and nameable -- so that I can change what the
bar contains by editing configuration, and potentially assign different menu bars
to different kinds of workspace, instead of the bar being hardcoded chrome.

**Source:** CR-NR-080 (supersedes CR-NR-077). Owner: "I want the menu bar to be
configurable and I want to be able to change it at any time. It could potentially
be a menu like any other. With each item in the menu either an actual command or
a menu in itself ... Menu Bars should have names so that we can select them and
assign them to different workspaces. so each workspace kind could potentially have
it's own menu bar?" + clarifications: nesting already works via command resolution
(an option whose command names a menu opens that menu -- Requirement 3, 5, 11), so
NO nested option data structure is introduced; the menu MODEL and command
RESOLUTION are unchanged. The only new render behaviour is that a menu bar's
top-level buttons PEEK their referenced submenu as a dropdown (they do not
navigate the workspace), and leaf options DISPATCH their command. A compiled
default menu bar (built from the current POM + Settings content) is the code-only
fallback, exactly like the POM and Settings defaults (Requirement 12); editing a
user menu file overrides it. Menu-bar menus are conventionally named with an
`MB-` prefix (e.g. `MB-POM`) -- a naming CONVENTION, not an enforced rule.

**Glossary addition:**
- **Menu_Bar** -- a Menu_File (Requirement 1) rendered HORIZONTALLY as a row of
  dropdown buttons at the top of the Workbench, rather than as a vertical option
  list in a Menu_Workspace. A Menu_Bar is not a distinct type; it is a Menu_File
  used in the bar role.
- **Peek** -- rendering a referenced menu's options inside an open menu-bar
  dropdown WITHOUT navigating the active Workspace to that menu. Only selecting a
  leaf option dispatches a command.
- **Default_Menu_Bar** -- the compiled code-only Menu_File used for the bar when
  no user menu-bar file exists. Per owner decision (CR-NR-080) the barebones
  Menu_Bar IS the barebones POM: the Default_Menu_Bar is the compiled POM
  (Requirement 12), so there is a SINGLE source of truth and the two cannot
  diverge.

#### Acceptance Criteria

**Slice A -- horizontal peek-dropdown render of a (default) menu-bar menu.**

1. THE Workbench menu bar SHALL be rendered from a Menu_File (Requirement 1),
   not from hardcoded button definitions. THE bar SHALL render each bar-visible
   top-level Menu_Option (criterion 17.2a) as a dropdown button, in the
   Menu_File's option order, left to right. EACH top-level button SHALL be
   LABELLED by the option's `command` (the verb the user would type), NOT its
   `description`.

2. THE workbench SHALL provide a compiled Default_Menu_Bar. Per owner decision
   the barebones Menu_Bar is the SAME as the barebones POM: `default_menubar_menu()`
   SHALL return the compiled POM (`recovery_pom_menu` / `DEFAULT_POM_TOML`,
   Requirement 12), a single source of truth (no separate menu-bar constant).
   WHEN no user menu-bar Menu_File is available, THE bar SHALL render from this
   Default_Menu_Bar.

2a. THE bar SHALL render ONLY options whose `show_in_menu_bar` is `true`
   (Requirement 1.3). An option with `show_in_menu_bar = false` (e.g. the
   barebones POM's `RETURN`) SHALL NOT appear on the bar, while remaining in the
   vertical menu. The barebones POM (Requirement 12) SHALL contain, in order,
   `SETTINGS`, `CATALOGS`, `FILES`, `HELP`, `RETURN`, with `RETURN` marked
   `show_in_menu_bar = false`; so the bar-visible entries are Settings, Catalogs,
   Files, Help.

3. WHEN a top-level menu-bar button is opened (by mouse hover/click or keyboard
   focus), THE bar SHALL PEEK the menu referenced by that option's command: it
   SHALL render that referenced menu's options as the dropdown's items WITHOUT
   navigating the active Workspace. WHERE the option's command does not resolve
   to a menu (it names a plain command), the option SHALL appear as a directly
   actionable item rather than a submenu.

4. WHEN a leaf item in an open menu-bar dropdown is selected, THE workbench SHALL
   DISPATCH that item's command through the SAME command path as typing it on
   the command line (`handle_command`; command parity, architecture-brief
   Principle 2), and SHALL close the dropdown. Selecting a leaf SHALL NOT require
   any bar-specific code path distinct from normal command dispatch.

5. WHILE a menu-bar dropdown is open, THE items within it SHALL be
   keyboard-navigable using the egui-native menu behaviour (arrow keys move
   between items, Enter activates, Escape closes), consistent with a standard
   application menu. (No bespoke Tab-ring entry for dropdown items is added.)

6. THE menu bar SHALL continue to participate in the unified Tab-order model
   (Requirement 15, CR-CH-023): the FIRST and LAST top-level menu-bar buttons'
   ids SHALL still be captured each frame for the shell Boundary_Policy, so
   Tab from the last Interior_Control reaches the bar and Tab from the last bar
   button wraps to the command field, exactly as before. Making the bar
   data-driven SHALL NOT regress the Boundary_Policy.

7. THE POM and Settings VERTICAL Menu_Workspaces (Requirement 2) SHALL be
   UNCHANGED by this requirement. A Menu_Bar is a separate, horizontally-rendered
   menu; the vertical Menu_Workspace rendering is not affected.

**Slice B -- named + editable menu-bar files.**

8. A Menu_Bar Menu_File SHALL be a user file under `<User_Data_Dir>/menus/`
   (Requirement 1.7), loaded via the existing loader and editable/savable via the
   existing Menus Editor (Requirement 13). WHERE a user menu-bar file exists, it
   OVERRIDES the compiled Default_Menu_Bar (a saved file is a user override; the
   built-in remains a code-only fallback, consistent with Requirement 12 /
   CR-CH-021). Menu-bar files are conventionally named with an `MB-` prefix
   (e.g. `MB-POM`); the prefix is a CONVENTION and SHALL NOT be enforced.

**Slice C -- per-workspace-kind assignment.**

9. THE workbench SHALL support assigning a Menu_Bar (by name) to a workspace KIND
   (the stable context names of Requirement 14.6), mirroring the per-kind keymaps
   pattern (function-keys Req 14.9-14.12, CR-CH-027). WHEN a Workspace of a given
   kind is active, THE bar SHALL render the Menu_Bar assigned to that kind; WHERE
   a kind has no assignment, THE Default_Menu_Bar (or a configured default bar
   name) SHALL be used.

**Slice D -- dynamic option sources (delivers the theme picker, ex-CR-NR-077).**

10. A Menu_Option MAY declare a DYNAMIC option source instead of (or in addition
    to) a static referenced menu: WHEN such an option is peeked, THE dropdown's
    items SHALL be generated at runtime from the named source. THE first source
    SHALL be the available-themes list: a `Themes` menu-bar item whose peeked
    dropdown lists every theme from the theme list (theme-and-appearance
    Requirement 14.6, in list order), each item dispatching `THEME <name>`.

11. Selecting a theme from the dynamic `Themes` dropdown SHALL apply and persist
    it through the SAME `THEME <name>` command path (theme-and-appearance
    Requirement 17.2, `set_active_theme`) -- command parity -- delivering the
    theme-picker behaviour previously specified as theme-and-appearance
    Requirement 17.8-17.13 (CR-NR-077), now via the menu-bar mechanism rather
    than a bespoke popup.

---

### Requirement 18: Unified Menu Workspace (the POM is a Menu Workspace)

**User Story:** As a maintainer, I want a SINGLE Menu Workspace concept instead
of a separate Primary Option Menu type, so that the Home Context (POM) and every
other menu behave identically and there is one code path to maintain -- the POM
is simply the Menu Workspace whose menu is named `pom`, seeded with the compiled
barebones baseline when no user menu file exists.

**Source:** [CR-NR-082] Slice 1. Owner: "why do we have a
TabKind::PrimaryOptionsMenu and TabKind::MenuWorkspace? There should be only one,
their behaviour should be the same ... if the application configuration goes
missing we should set up a POM with a set of barebones defaults." Owner decision:
option (b) -- FULLY remove the separate POM kind.

**Note:** This is a behaviour-PRESERVING unification (Slice 1 of CR-NR-082). It
introduces no new user-visible menu behaviour; it removes the duplicate POM
render arm / tab kind / persistence path. Named Workspaces (id/name/kind/menu),
per-workspace menu-bar + keymap, and the per-workspace Profile store are LATER
slices of CR-NR-082 and are NOT part of this requirement.

#### Glossary additions

- **Home_Menu_Name**: the reserved menu name `pom` identifying the Home Context
  Menu Workspace. Opening the Home Context is opening the Menu Workspace named
  `pom`.
- **Unified_Menu_Workspace**: the single runtime Context kind that renders any
  data-driven menu (formerly split across the `PrimaryOptionMenu` and
  `MenuWorkspace` tab kinds).

#### Acceptance Criteria

1. THE workbench SHALL represent every data-driven menu -- including the Home
   Context (POM) -- with a SINGLE Menu Workspace Context kind. The separate
   Primary-Option-Menu tab kind (`TabKind::PrimaryOptionMenu`) SHALL be removed;
   the Home Context SHALL be the Menu Workspace whose menu name is the
   Home_Menu_Name (`pom`).
2. THE Home Context Menu Workspace SHALL load its menu from `menus/pom.toml` when
   present, and SHALL fall back to the compiled barebones Recovery_Baseline POM
   (Requirement 12) when the file is absent or fails to parse -- identical to the
   pre-unification `ensure_pom_menu_loaded` behaviour (a parse error surfaces a
   non-blocking notice; an absent file is silent).
3. THE unified Menu Workspace SHALL render every menu (Home or otherwise) through
   the SINGLE shared menu renderer (Requirement 2), with identical option-column
   layout, calendar behaviour (Requirement 16), option selection (Requirement 3),
   chained navigation (Requirement 5), and Tab-order / focus behaviour
   (Requirement 15). No menu SHALL have a distinct render path by virtue of being
   the Home Context.
4. THE always-present-Home guarantee SHALL be preserved: on launch, and after
   END/RETURN unwinds to the origin, and as the universal navigation fallback,
   the workbench SHALL ensure a Home Context Menu Workspace (menu `pom`) exists
   and is reachable (startup-and-session Requirement 14.1; Requirement 14
   Navigation_Stack), exactly as before unification.
5. THE Title_Line and tab header SHALL present the Home Context identically to
   before (its ISPF-style Home identity), derived from the Menu Workspace rather
   than a distinct POM tab kind.
6. THE per-Context keymap selection (function-keys-and-history Requirement 14.6,
   `context_name_for_kind`) SHALL continue to resolve the Home Context to the
   `pom` keymap context and every other menu to the `menu` keymap context, so no
   key binding changes as a result of unification.
7. SESSION persistence SHALL round-trip the Home Context and every other menu
   through ONE Menu Workspace descriptor keyed by menu name (the Home Context as
   menu name `pom`), reconciling the prior divergence where the POM persisted as
   a `CustomWorkspace` kind and other menus as `Menu { name }`. Sessions written
   before unification (a persisted `PrimaryOptionMenu` kind / `CustomWorkspace`
   POM) SHALL still restore the Home Context without error (backward
   compatibility, startup-and-session Requirement 21.10).
8. THE overlapping Context enumerations SHALL be reconciled so the POM concept is
   not duplicated: the runtime tab-kind enumeration SHALL NOT carry a separate
   Primary-Option-Menu variant, and the session-layer Workspace_Kind /
   legacy-persistence enumerations SHALL map the Home Context to the unified Menu
   Workspace (menu `pom`) rather than a separate POM kind, while still reading
   legacy sessions (criterion 18.7).
9. THE unification SHALL be behaviour-preserving: every existing menu-workspace
   and POM acceptance criterion (Requirements 2, 3, 5, 12, 14, 15, 16, 17) SHALL
   continue to hold, verified by the existing tests continuing to pass (adjusted
   only where they referenced the removed POM tab kind by name).

---

### Requirement 19: One Option-Selection Path (the Menu Workspace is a dumb dispatcher)

**User Story:** As a maintainer, I want selecting an option in ANY Menu Workspace
-- the POM, the Settings menu, or a user menu -- to flow through ONE piece of
code regardless of how it was triggered (click, key, Tab+Enter, menu bar, or a
typed command), so that a POM option click is not handled differently from a
Settings option click, and the phantom-stop / divergent-dispatch class of bug
(B056-B059, B075) cannot recur because there is only one path to get wrong.

**Source:** [CR-CH-043]. Owner: "a POM option click should not be any different
from a settings option click, they should be handled by the same code, they are
both menu workspaces. Also selecting an option on a menu should not be any
different from executing a command. the menu workspace should not care if the
option being selected is into another menu workspace or another custom workspace.
it should just execute the command. The command should know that it is executing
a menu workspace and behave accordingly." Supersedes the B075 `open_named_menu`
router patch (which narrowed but did not remove the divergence).

**Note:** This requirement UNIFIES the dispatch behaviour; it introduces no new
user-visible option behaviour. It reconciles the divergence in which POM option
keys were resolved by one path (the Navigation_Origin POM fastpath) and non-POM
menu option keys by another (the current-menu Option_Key lookup that skipped the
Home Context), and in which a mouse CLICK ran a pre-branch (inline target /
user-target resolution) that a typed key did not. After this requirement, all of
these resolve to the same single act: execute the option's command string.

#### Glossary additions

- **Option_Selection**: the act of choosing a Menu_Option by ANY means -- typing
  its Option_Key and pressing Enter, clicking its row, tabbing to it and pressing
  Enter/Space, picking it from a rendered menu bar, or a chained/fastpath form
  that lands on it.
- **Option_Dispatch_Path**: the single code path that turns an Option_Selection
  into the execution of that option's Option_Command.

#### Acceptance Criteria

1. THE shell SHALL provide ONE Option_Dispatch_Path shared by every
   Option_Selection means (click, Option_Key + Enter, Tab + Enter/Space, menu-bar
   pick, and a typed command that lands on an option), such that all means with
   the SAME target option produce an IDENTICAL observable result. Selecting an
   option SHALL be defined as executing that option's Option_Command through the
   standard command pipeline (`handle_command`), i.e. Option_Selection is
   observably identical to typing the Option_Command in the `Command ===>` field
   and pressing Enter (Requirement 3.2; command-framework Requirement 14).
2. THE Option_Dispatch_Path SHALL be the SAME for the Home Context (POM) and for
   every non-Home Menu Workspace (Settings, user menus). The former split -- POM
   Option_Keys resolved by the Navigation_Origin POM resolver while non-POM
   Option_Keys were resolved by a separate current-menu lookup that skipped the
   Home Context -- SHALL be removed: ONE current-menu Option_Key resolver SHALL
   look up the selected key against the ACTIVE menu (whether that menu is `pom`,
   `settings`, or a user menu) and dispatch its Option_Command. A POM option
   click SHALL therefore run the same code as a Settings option click.
3. THE mouse-CLICK Option_Selection path SHALL NOT run any dispatch pre-branch
   that the typed path does not: a click SHALL resolve to the option's
   Option_Command and call `handle_command(Option_Command)` exactly as a typed
   Option_Key does. The former click-only pre-step (dispatching an inline
   `[options.target]` or a resolved user Command_Target BEFORE reaching
   `handle_command`) SHALL be removed as a separate click path; the inline-target
   capability of Requirement 10.6 SHALL be preserved by resolving it within the
   single command pipeline (command-framework Requirement 8.6), not by a
   click-only fork.
4. THE Menu Workspace SHALL act as a DUMB dispatcher: it SHALL NOT decide the
   KIND of the target (another menu, a custom workspace, a function, a macro, or
   an external program) and SHALL NOT decide whether the effect is an in-place
   navigation or a new tab. Its sole responsibility on an Option_Selection SHALL
   be to determine the selected option's Option_Command and hand it to
   `handle_command`. The Navigation_Origin `=` fastpath semantics (Requirement 5)
   and the `<menu> <key>` chaining (Requirement 11.7) SHALL be preserved as
   command-string parsing, not as separate dispatch paths.
5. THE decision of in-place navigation VERSUS opening a new tab SHALL be owned by
   the COMMAND that the option runs (command-framework Requirement 14.3), NOT by
   the Menu Workspace and NOT by a dispatcher-level name router. Specifically, the
   `open_named_menu` routing that today chooses `open_settings_menu` (in place)
   for `settings`, the Home Context for `pom`, and `open_menu_by_name` (new tab)
   otherwise SHALL be folded into the respective command handlers, so that after
   this requirement a menu-opening command "knows" it is opening a menu workspace
   and applies its own in-place-vs-new-tab behaviour, while the menu that launched
   it is unaffected by that choice.
6. WHEN a selected option is `enabled = false`, THE single Option_Dispatch_Path
   SHALL display `Option '<key>' is not available.` and take no further action,
   for EVERY means of selection and for POM and non-POM menus alike
   (Requirement 3.7 applied uniformly on the one path).
7. WHEN a selected option's Option_Command cannot be resolved by any stage of the
   command-resolution chain (command-framework Requirement 8.3), THE single
   Option_Dispatch_Path SHALL surface the unresolved-command error naming the
   command and leave the Workspace unchanged (Requirement 3.6 / 10.5), for every
   means of selection.
8. THE unification SHALL be behaviour-preserving for every already-built option:
   the existing menu-workspace acceptance criteria (Requirements 3, 5, 10, 11,
   14, 15, 16, 18) and the workspace-conformance first-Tab focus tests SHALL
   continue to hold. In particular, clicking the POM `Settings` option SHALL open
   the Settings menu IN PLACE (the B075 behaviour), now achieved by the command
   owning that effect (criterion 5) rather than by the `open_named_menu` router.

---

### Requirement 20: Single config-driven centered title; short-form POM tab; POM command

**User Story:** As a user, I want a Menu Workspace to show ONE title -- sourced
from its menu configuration file and centered -- instead of two near-identical
banners (the Title_Line above the command line AND a second heading above the
options); I want the POM's tab heading to be a short form like every other
workspace's tab; and I want the POM to be command-addressable like every other
workspace, so the whole title chrome is consistent and config-driven.

**Source:** [CR-CH-042]. Owner: "On the POM the tab text is 'FileForge Workbench
v0.1.0'. There is also a heading above the command line of the same text, and
another heading above the options taken from the menu config file 'FileForge
Workbench -- Primary Option Menu'. The Settings menu follows a different format:
the heading above the command line just says 'SETTINGS' (not centered like the
POM), and the heading above the options just says 'Settings'. This doubling up of
title is redundant -- one should be removed, the title should come from the menu
configuration file, and should be centered on the line. Also the Tab heading
should be a short form even for the POM. Other workspaces' tab heading comes from
the command; the POM has no command except perhaps start. Perhaps add a command
'POM' with an alias 'START', and then the POM tab heading could be 'POM'."

**Note:** This aligns the title chrome; it removes redundant/inconsistent titles
and regularises the POM. It does NOT change option behaviour or navigation. The
current state (context-gathered): a Menu Workspace renders its `MenuFile.title`
as a CENTERED heading above the options (`menu_workspace/render.rs`, "Req 2.1 --
Menu_Title centred") AND the shell Title_Line above the command line shows a
SEPARATE string -- for the POM the hardcoded app banner
`FileForge Workbench  vX.Y.Z` (via `title_line_text` is_home early-return), for a
non-POM menu the bracketed uppercased menu title `[<TITLE>]` (via `tab_title()`).
The POM tab header also shows the long app banner. This requirement makes the
Title_Line the single title for a Menu Workspace, sourced from the config file
and centered, and drops the duplicate.

#### Glossary additions

- **Menu_Title**: the `title` field of a Menu_File (Requirement 1), the single
  human-readable name of that menu (e.g. `FileForge Workbench -- Primary Option
  Menu`, `Settings`).
- **Short_Tab_Label**: the concise text shown on a Workspace's Tab_Header, as
  distinct from the full Title_Line title.

#### Acceptance Criteria

1. THE Menu_Title (the Menu_File `title`) SHALL be the SINGLE source of a Menu
   Workspace's displayed title. A Menu Workspace SHALL render its title in
   EXACTLY ONE position; the previous duplication -- the centered heading above
   the option list AND a separate Title_Line above the command line -- SHALL be
   removed so only one remains. (Decided at the gate: the surviving position is
   the shell Title_Line above the command line; see criterion 2. The separate
   centered `menu.title` heading above the option list in the menu body is
   REMOVED.)
2. WHEN a Menu Workspace (the POM, the Settings menu, or any user menu) is the
   active/rendered Context, THE Title_Line SHALL display that menu's Menu_Title
   text, CENTERED on the line, using the same format for EVERY menu including the
   POM and Settings (the current POM-centered vs Settings-left inconsistency is
   removed). The Title_Line text SHALL be the raw Menu_Title (not bracketed, not
   force-uppercased), so the POM shows its configured title (e.g. `FileForge
   Workbench -- Primary Option Menu`) and Settings shows `Settings`.
3. THE POM Title_Line SHALL NO LONGER display the hardcoded application banner
   `FileForge Workbench  vX.Y.Z`; it SHALL display the POM Menu_Title from
   `menus/pom.toml` (or the compiled Recovery_Baseline POM title when no user
   file exists), consistent with criterion 2. (This REVISES menu-and-statusbar
   Requirement 17.3, which is amended in lock-step; the application name/version
   remains available elsewhere in the shell -- e.g. an About affordance or the
   status area -- and is not lost, but it is no longer the POM Title_Line.)
4. THE Tab_Header label of EVERY Menu Workspace SHALL be derived by ONE uniform
   rule with NO POM-specific branch: the Menu_Name (the backing
   `menus/<name>.toml` stem -- the name the OPENING COMMAND uses), uppercased.
   The POM is NOT a special case here: it is the Menu Workspace named `pom`, so
   the same rule yields `POM`; the Settings menu yields `SETTINGS`; a user menu
   `reports.toml` yields `REPORTS`. There SHALL be NO code that checks
   `is_home` (or any POM flag) to force the header to `POM` -- the label falls
   out of the general Menu_Name derivation. The former long application-banner
   POM header is removed as a consequence of applying the general rule.
5. THE workbench SHALL provide a `POM` command that opens (or returns to) the
   Home Context, so the POM is command-addressable like every other menu
   (command parity, architecture-brief Principle 2). `START` SHALL be accepted as
   an ALIAS that opens the Home Context (its existing tab-creation forms of
   menu-workspace Requirement 14.8-14.9 are preserved: bare `START` and `START
   =<path>` / `START <arg>`; the bare form is the POM-opening alias). The `POM`
   command SHALL be registered/dispatchable by name (command parity) so a menu
   option, key binding, or typed command can invoke it. The command name `POM`
   is the Menu_Name of the `pom` menu, which is why the general Tab_Header rule
   (criterion 4) yields `POM` -- the header comes from the command/menu name, not
   from a POM special case.
6. THE Tab_Header derivation of criterion 4 SHALL be the SAME single code path
   for the POM and every other Menu Workspace (and SHALL sit alongside the
   general per-Kind header derivation for non-menu Contexts, workspace-kinds
   Requirement 3). The ONLY sanctioned POM-specific behaviour in the whole
   workbench is the load-time guarantee that at least one POM instance exists
   (menu-workspace Requirement 18.4 / startup-and-session Requirement 14.1);
   title and tab-header derivation SHALL contain no POM special case.
7. THE single-title change SHALL preserve the existing Title_Line behaviour for
   NON-menu Contexts unchanged: a file editor Title_Line still shows the file
   path / `[Untitled]` (menu-and-statusbar Requirement 17.4/17.5) and a
   system/panel Context still shows its Kind title (Requirement 17.6;
   workspace-kinds Requirement 3), so only the Menu Workspace title source and
   the duplicate-heading removal change.
8. THE in-place context-switch guarantee (menu-and-statusbar Requirement 17.10 /
   CR-CH-034) SHALL continue to hold with the single title: after an in-place
   Context switch, the Title_Line SHALL reflect the NEW Context's Menu_Title
   immediately, never a stale title, derived from the live loaded menu rather
   than a cached string.
9. THE change SHALL be behaviour-preserving for menu option selection,
   navigation, calendar, and Tab-order (Requirements 3, 5, 15, 16, 19): removing
   the body-heading and re-sourcing the Title_Line SHALL NOT alter the option
   list layout decisions, the calendar fit tiers (Requirement 16), or the focus
   contract; the removed heading simply frees vertical space above the options.
