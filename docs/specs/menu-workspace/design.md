# Design Document -- Menu Workspace Pattern

## 1. Overview

The Menu Workspace pattern introduces a data-driven alternative to hardcoded
option menus. A Menu_Workspace is a Workspace whose Context is loaded from a
TOML file at runtime. The POM and Settings Context become instances of this
pattern; no new Workspace kind is introduced for them -- they are re-implemented
as Menu_Workspace instances backed by named TOML files.

This document covers the architecture, data model, TOML loader, hot-reload
mechanism, chained path resolver, and the rendering contract. It does NOT cover
the final content of `pom.toml` or `settings.toml` -- those are defined in
Phase CV and Phase CW respectively.

---

## 2. Architectural Position

The Menu Workspace pattern sits in the `ff-desktop` binary layer. It does not
introduce a new library crate. The components are:

```
ff-desktop/src/
  menu_workspace/
    mod.rs          -- MenuWorkspaceState, public API, re-exports
    loader.rs       -- TOML parsing, MenuFile -> MenuWorkspaceState
    hot_reload.rs   -- file-watch integration, reload trigger
    render.rs       -- egui rendering of option list
    commands.rs     -- option selection dispatch
```

The `MenuWorkspaceState` struct is stored on the `TabState` alongside the
existing `EditorState`, `FilesPanelState`, etc. A new `TabKind::MenuWorkspace`
variant is added.

---

## 3. Data Model

### 3.1 MenuFile (TOML schema)

```toml
title = "Primary Option Menu"

[[options]]
key = "0"
command = "SETTINGS"
description = "FFWB Settings and Client Parameters"
group = "System"

[[options]]
key = "1"
command = "CATALOGS"
description = "Virtual File Catalogs"
enabled = true
```

### 3.2 Rust types

```rust
pub struct MenuFile {
    pub title: String,
    pub options: Vec<MenuOption>,
}

pub struct MenuOption {
    pub key: String,           // 1-4 chars, stored uppercase
    pub command: String,
    pub description: String,
    pub enabled: bool,         // default true
    pub group: Option<String>,
}
```

### 3.3 MenuWorkspaceState

```rust
pub struct MenuWorkspaceState {
    pub file_path: PathBuf,
    pub menu: Option<MenuFile>,   // None = load error
    pub load_error: Option<String>,
    pub last_modified: Option<SystemTime>,
}
```

`MenuWorkspaceState` is stored in `TabState` and serialised to session TOML as
`PersistedTabKind::MenuWorkspace { file_path: String }`.

---

## 4. TOML Loader

`loader.rs` exposes:

```rust
pub fn load_menu_file(path: &Path) -> Result<MenuFile, String>
```

- Reads the file with `std::fs::read_to_string`.
- Parses with `toml::from_str`.
- Validates: `title` present, each option has `key` (1-4 chars), `command`,
  `description`.
- Normalises `key` to uppercase.
- Unknown TOML keys are silently ignored (serde `deny_unknown_fields` is NOT
  used -- forward compatibility).
- Returns `Err(human_readable_message)` on any failure.

---

## 5. Hot-Reload

`hot_reload.rs` integrates with the existing `ff-config` file-watch
infrastructure rather than introducing a new `notify` watcher.

On each egui frame, `MenuWorkspaceState::poll_reload()` checks:

```rust
fn poll_reload(&mut self) {
    if let Ok(meta) = std::fs::metadata(&self.file_path) {
        if let Ok(modified) = meta.modified() {
            if Some(modified) != self.last_modified {
                self.last_modified = Some(modified);
                match load_menu_file(&self.file_path) {
                    Ok(menu) => { self.menu = Some(menu); self.load_error = None; }
                    Err(e)   => { self.load_error = Some(e); }
                }
            }
        }
    }
}
```

This is a stat-based poll (one `metadata()` call per frame per open
Menu_Workspace). It is cheap enough for the expected number of open menu tabs
(typically 1-2). If profiling shows overhead, the poll can be throttled to once
per second using a `last_poll: Instant` field.

---

## 6. Rendering

`render.rs` renders the Menu_Workspace inside the standard Workspace chrome
(Title_Line, Command_Field, Key_Label_Bar). The option list area:

- Wraps in `egui::ScrollArea::vertical()`.
- Renders each `MenuOption` as a row: key column (fixed width, monospace) +
  description column.
- Uses `ui.selectable_label(false, ...)` for enabled options so they respond to
  click.
- Uses `ui.add_enabled(false, ...)` for disabled options.
- Inserts a `ui.separator()` between groups when the `group` field changes.
- On click, calls `commands::execute_option(option, shell_state)`.

The focused option (Tab-cycle stop) is highlighted with the existing
`render_focus_indicator` helper (Phase CO, Req 3.3).

---

## 7. Command Dispatch

`commands.rs` exposes:

```rust
pub fn execute_option(option: &MenuOption, shell: &mut WorkbenchShell)
```

- If `!option.enabled`: sets status message `Option '<key>' is not available.`
  and returns.
- Otherwise: calls `shell.handle_command(&option.command)` -- the same path as
  typing the command into the `Command ===>` field.

The `Command ===>` field handler in `shell/commands.rs` is extended to:

1. Check if the active tab is a `MenuWorkspace`.
2. Look up the typed text (trimmed, uppercased) in `state.menu.options` by key.
3. If found: call `execute_option`.
4. If not found: fall through to the standard command pipeline (so `EXIT`,
   `HELP`, etc. still work from a menu tab).

---

## 8. Chained Path Resolver

The existing fastpath handler in `shell/commands.rs` already handles `=N`
single-segment paths. It is extended to handle `=<seg1>.<seg2>...<segN>`:

1. Split the path on `.` after stripping the leading `=`.
2. Resolve segment 1 against the current menu (or POM if no menu is active).
3. Execute the resulting command. If that command opens a new Menu_Workspace,
   immediately resolve segment 2 against it, and so on.
4. Maximum depth: 4 segments. Deeper paths are rejected with a status message.

The resolver is a pure function:

```rust
pub fn resolve_chained_path(
    path: &str,
    menus: &HashMap<String, MenuFile>,
) -> Result<String, String>
```

It returns the final `Option_Command` to execute, or an error string.

---

## 9. Default Menu Files

On startup, `shell/update.rs` calls:

```rust
fn ensure_default_menu_files(user_data_dir: &Path)
```

This function creates `menus/pom.toml` and `menus/settings.toml` if absent,
using embedded default content strings. The default content is defined as
`const` strings in `menu_workspace/defaults.rs`.

Until Phase CV defines the final POM content, `pom.toml` is NOT created by this
function -- the existing hardcoded POM (`TabKind::PrimaryOptionMenu`) continues
to be used. The `ensure_default_menu_files` function is a stub that creates only
`settings.toml` (also a stub until Phase CW).

This avoids any behaviour change in Phase CU: the spec and architecture are
defined, but the POM and Settings remain hardcoded until their respective phases
explicitly migrate them.

---

## 10. Session Persistence

> SUPERSEDED by startup-and-session Requirement 21 (Phase DB, CR-CH-012).
> The original plan below (adding a `PersistedTabKind::MenuWorkspace { file_path }`
> variant) is NOT the approach taken. That variant was never implemented, and
> the closed `PersistedTabKind` enum is replaced by the Workspace_Descriptor
> model. A Menu_Workspace now persists as a `MenuWorkspace { name }`
> Workspace_Descriptor (the menu name, not a raw file path) and is re-opened on
> restore via the `MENU <name>` command (Requirement 11). If the backing
> `menus/<name>.toml` is absent on restore, the Workspace opens in the
> load-error state (Requirement 1.5). Retained below for historical context.

~~`PersistedTabKind` gains a new variant `MenuWorkspace { file_path: String }`;
on restore the shell creates a `MenuWorkspaceState` from the persisted
`file_path`.~~ (Superseded -- see the note above.)

---

## 11. Tab Kind and Routing

The runtime tab kind is a **data-free** variant:

```rust
TabKind::MenuWorkspace,   // Copy; carries no payload
```

The actual `MenuWorkspaceState` is held in a separate `menu_workspace:
Option<MenuWorkspaceState>` field on `TabState`, populated only when
`kind == TabKind::MenuWorkspace`. (This corrects the earlier draft that showed a
data-carrying `TabKind::MenuWorkspace(MenuWorkspaceState)`, which does not match
the implemented enum: `TabKind` is `Copy` and cannot carry non-`Copy` state.)

The `MENU` / `MENU <name>` shell command (Command_ID `"menu.open"`) opens a
Menu_Workspace and is defined by menu-workspace Requirement 11:

- `MENU` (no argument) -- returns to the Home Context (POM), backed by
  `menus/pom.toml`.
- `MENU <name>` -- opens the Menu_Workspace backed by `menus/<name>.toml`
  (`MENU POM`, `MENU SETTINGS` are the named built-in forms).

A `Menu_Target { name }` (command-framework Requirement 8) is executed by
invoking `MENU <name>`. Wiring this command and opening a `TabKind::MenuWorkspace`
tab at runtime is a Phase DB implementation step (DB.8/DB.11); it was spec-only
in Phase CU.

---

## 12A. Configurable Option Limits (Phase DA, Requirement 9)

Phase DA adds two configuration-driven bounds on Menu_File option count. This is
a design delta to the loader (Section 4) and render (Section 6); no new module,
data flow, or Workspace kind is introduced.

### 12A.1 Configuration keys

Two keys are registered in the `ff-config` schema (`configuration-system`
Requirement 9) under the `menu` namespace:

```
menu.soft_option_limit  (u32, default 64)
menu.hard_option_limit  (u32, default 256)
```

They resolve through the standard layered model, so a project or profile layer
may override the defaults. The loader reads them via the typed access API
(`configuration-system` Requirement 7); a missing or invalid value falls back to
the default (Requirement 9.6, 9.7).

### 12A.2 Loader change

`load_menu_file` gains awareness of the two limits. The signature is extended to
accept the resolved limits so the function stays pure and unit-testable:

```rust
pub struct OptionLimits {
    pub soft: u32,
    pub hard: u32,
}

pub fn load_menu_file_with_limits(
    path: &Path,
    limits: OptionLimits,
) -> Result<LoadedMenu, String>
```

where `LoadedMenu` carries the parsed `MenuFile` plus an optional advisory:

```rust
pub struct LoadedMenu {
    pub menu: MenuFile,
    pub advisory: Option<String>,   // Some(..) when soft limit exceeded
}
```

Evaluation order inside the loader, after TOML parse and field validation:

1. Normalise `hard` and `soft`: `effective_soft = min(soft, hard)` (Req 9.5).
2. Let `n = menu.options.len()` counted before `enabled` filtering (Req 9.8).
3. If `n > hard`: return `Err("too many options: <n> exceeds hard limit <hard>")`
   (Req 9.4).
4. If `n > effective_soft`: set `advisory = Some(...)` and log WARN (Req 9.3).
5. Otherwise `advisory = None` (Req 9.2).

The existing `load_menu_file(path)` is retained as a thin wrapper that reads the
limits from config and calls `load_menu_file_with_limits`, so existing call
sites and tests that do not care about limits are unaffected.

### 12A.3 State change

`MenuWorkspaceState` gains one field:

```rust
pub advisory: Option<String>,   // soft-limit advisory, shown above the option list
```

`poll_reload` (Section 5) is updated to call `load_menu_file_with_limits` and to
set both `menu`/`load_error` and `advisory` on each reload (Req 9.9). A file
edited past the hard limit transitions to `load_error`; a file edited back under
the limit clears `load_error` on the next poll.

### 12A.4 Render change

`render.rs` renders `state.advisory`, when present, as a single non-blocking
advisory line styled like a warning notice, positioned immediately above the
option list and below the Menu_Title. It does not block interaction with the
options. The hard-limit case reuses the existing load-error rendering path
(Requirement 1.6) and shows no option rows.

### 12A.5 Why 64 / 256

- 64 (soft): roughly 5x the current 12-option POM; comfortably scrollable and
  still practical to address by key. Beyond it, sub-menus are the better tool.
- 256 (hard): generous headroom for generated or plugin-injected menus while
  protecting the per-frame reload path from a pathologically large file.

Both are defaults, not constants -- any deployment may raise or lower them via
configuration.

---

## 12. No Design Changes Required for Phase CU

Phase CU is a specification-only phase. No source files are modified. The design
above describes the target architecture that will be implemented when a separate
implementation instruction is given.

---

## 13. Command Target Integration (Requirement 10, CR-NR-051)

This is a design delta to Section 7 (Command Dispatch). Option selection no
longer hands a bare string straight to `handle_command`; it first resolves the
string to a `CommandTarget` (command-framework Requirement 8) and then executes
that target. This makes the "option leads to a sub-menu vs a custom workspace vs
an external program" distinction explicit in the resolved target instead of
being an implicit consequence of the command string.

### Option -> target resolution

`commands::execute_option` becomes:

1. If the `MenuOption` carries an inline `[options.target]` table, use it
   directly (Requirement 10.6).
2. Otherwise call `ff_command::resolve_target(option.command, registry,
   user_commands)` (Requirement 10.1). A bare string still resolves to the same
   equivalent target it produces today (Requirement 10.2), and a string equal to
   a user-defined command id resolves to that definition's target
   (Requirement 10.3).
3. Execute via `execute_target`. A `Menu_Target` opens the referenced menu
   (Requirement 10.4); a `CustomWorkspace`/`Function`/`Macro`/`External` target
   routes as defined in command-framework.
4. On resolution failure, show `Option '<key>' could not be resolved: <reason>`
   (Requirement 10.5).

### Data model addition

`MenuOption` gains an optional inline target:

```rust
pub struct MenuOption {
    pub key: String,
    pub command: String,                 // still required (Requirement 1.2)
    pub description: String,
    pub enabled: bool,
    pub group: Option<String>,
    /// Optional inline Command_Target; when present it wins over `command`.
    /// Validates: menu-workspace Requirement 10.6
    pub target: Option<ff_command::CommandTarget>,
}
```

`command` remains required for backward compatibility and forward readability;
`target` is the escape hatch for authors who want to embed a full target (e.g.
an external program) directly in a menu file without first defining it in the
Command_Store.

### Relationship to Section 11 (MENU commands)

The still-unimplemented `MENU <name>` command (Section 11) resolves to a
`Menu_Target { name }` under this model. Wiring the Menu Workspace pattern to run
at runtime (opening a `TabKind::MenuWorkspace` from `menus/<name>.toml`) is the
prerequisite for `Menu_Target` execution and is tracked in the Phase DB task
list; it is a separate implementation step from this spec delta.

---

## Design Delta: MENU Argument Chaining (Requirement 5.5-5.6, 11.7-11.10, CR-NR-054)

The `MENU` command becomes argument-aware via the general command-argument
mechanism (command-framework Requirement 9). `MENU <name> <key>` opens the menu
`<name>` and immediately activates the option whose key equals `<key>`
(case-insensitive, key match only -- not a word alias).

### MENU dispatch with a chained key

```text
MENU                       -> Home Context (POM)                 (Req 11.1)
MENU <name>                -> open menus/<name>.toml             (Req 11.2)
MENU <name> <key>          -> open menus/<name>.toml, then
                              activate option whose key == <key> (Req 11.7)
MENU <name> <key> <rest..> -> as above; <rest..> forwarded to the
                              activated option's own command as its argument
                              (Req 11.10 -- deeper chains compose)
```

Implementation: `open_menu_by_name` (already added for Requirement 11) gains an
optional trailing argument. After the menu tab is opened/loaded, if a chained key
is present the shell looks it up in the freshly loaded `MenuFile` (case-insensitive
key match via the existing `menu_workspace::commands::find_option`) and dispatches
that option exactly as a click/typed selection would (through the existing
option-dispatch path, so Target_Resolution and inline `[options.target]` still
apply). An unknown key opens the menu and reports `Option '<key>' not found.`
(Req 11.9), reusing the existing not-found message.

### Equivalence of the three forms (Req 11.8, 5.5)

All three activate the same option, because all three resolve to
"open menu <name>, then activate option <key>":

```text
MENU SETTINGS E          (chained MENU command)
=0.E                     (Chained_Path fastpath, Req 5; 0 is the POM key for SETTINGS)
type "E" + press a key bound to "MENU SETTINGS"   (command-framework Req 9.8)
```

The Chained_Path resolver (Requirement 5) and the chained-MENU path share one
navigation-and-activation helper so the two notations cannot diverge (Req 5.5).

### Option command forwarding (Req 5.6)

A menu option whose `command` value is itself a chained `MENU <name> <key>` (or a
Chained_Path) forwards its argument when selected, so one option can jump directly
into a specific option of another menu. This is the same forwarding the general
argument mechanism provides (command-framework Requirement 9.7); no menu-specific
argument store is introduced.

### Settings_Menu note (B032)

With chaining in place, the Settings_Menu (menus/settings.toml) namespace options
may be expressed either as the existing `SETTINGS <ns>` verb (opens the filtered
Settings_Namespace_View) or, equivalently, reached by `MENU SETTINGS <key>`. The
B032 fix (bare `SETTINGS`/`0` open the Settings_Menu Menu_Workspace) is unchanged;
this delta only adds the chained-activation path on top of it.

## Design Delta: Chained Path Separator Semantics (Requirement 5.7-5.12, CR-NR-057)

This extends Section 8 (Chained Path Resolver) and the MENU Argument Chaining
delta. The resolver already walks a dotted path left-to-right; this delta adds
the `;` (PUSH) separator alongside `.` (STOP) and the `=`-origin rule, and routes
the push/collapse decision to the Context Navigation Stack (command-framework
Requirement 10). The fastpath resolver and the command-line chain executor share
one helper (command-semantics `split_chain`, Requirement 11.10) so the two
notations cannot diverge.

### Resolver signature change

`resolve_chained_path` gains per-segment separators and cooperates with the
navigation stack rather than returning a single command string:

```rust
/// Resolve a Chained_Path (leading `=`) into an ordered list of option
/// activations, each tagged with the separator that preceded it.
/// `.` segments are STOP (collapse), `;` segments are PUSH.
/// Validates: menu-workspace Requirement 5.7-5.12
pub fn resolve_chained_path(
    path: &str,
    menus: &HashMap<String, MenuFile>,
) -> Result<Vec<PathStep>, String>;

pub struct PathStep {
    pub option_command: String,
    pub separator: ff_command_semantics::ChainSeparator, // Stop | Push
}
```

The shell drives each PathStep in order: it activates the option, and when the
activation opens/changes a Context (produces_visible_workspace true), it pushes
the prior Context onto the stack for a PUSH step and does not for a STOP step
(command-framework Requirement 10.5, 10.6).

### The `=` origin

A leading `=` means "begin from the POM": before walking the segments the shell
sets the Navigation_Origin to the POM (Requirement 5.7 / command-framework 10.2).
A non-`=` navigation command (typed verb, no leading `=`) uses the current
Workspace as origin (command-framework 10.3), so the same PathStep machinery
serves both notations.

### Worked examples

From an Edit Workspace:

```text
=0.E   -> POM -> (0) Settings -> (E) Editor Config
          stack: [POM]                 END -> POM
=0;E   -> POM -> (0) Settings -> (E) Editor Config
          stack: [POM, Settings]       END -> Settings, END -> POM
editor -> Editor Config directly (no =, origin = current Edit Workspace)
          stack: [Edit WS]             END -> Edit WS
settings ; editor  (no =, two hops)
          stack: [Edit WS, Settings]   END -> Settings, END -> Edit WS
```

From the Editor Config Workspace, `END ; EDIT` pops the stack (returns to the
previous navigation point) and then runs EDIT from there (command-framework
Requirement 10.9).

### Mixed separators

A path may mix separators (for example `=0;E.T`); each separator independently
controls the push/collapse of the segment it precedes (Requirement 5.11). The
existing 4-level nesting limit (Requirement 5.2) is unchanged.


---

## Design Delta: Unified config-driven menu renderer (Requirement 2.1a-2.1c, CR-CH-018)

Today there are TWO renderers: `primary_option_menu.rs` (POM: three columns
key/command/description + live calendar) and `menu_workspace/render.rs`
(Menu_Workspace: a single `key  description` line, no command column, no
calendar). The owner directive is ONE renderer: the POM is just a Menu_Workspace
backed by `menus/pom.toml`; Settings is the same renderer with `settings.toml`;
both files share the identical structure so POM options are add/removable by
editing config.

Design:
- Fold the POM's column + calendar layout (`primary_option_menu::render` and its
  `render_calendar_row` / calendar helpers) into the shared menu renderer
  (`menu_workspace/render.rs`), so `render_menu_workspace` draws:
  - a three-column option list (Option_Key | Option_Command | Option_Description),
    aligned, replacing the current single-line `{key:<4}  {description}` row;
  - the live calendar panel on the right when the menu's `show_calendar`
    (Requirement 1.8) is true, reusing the existing calendar date/grid helpers
    (day_of_year, days_in_month, first_weekday_of_month, offset_month,
    format_calendar_header, render_calendar_row) -- these stay as pure helpers,
    now called from the shared renderer.
- The calendar month-navigation state (`pom_calendar_offset`) and the
  `CalendarNav` return become part of the menu renderer's return so any menu
  showing the calendar can navigate months.
- The POM stops using a bespoke render path: the Home Context is rendered via
  the shared `render_menu_workspace` against the `pom.toml`-backed
  MenuWorkspaceState. POM option routing continues to dispatch each option's
  `command` through the standard pipeline (Requirement 3/10), so existing POM
  option behaviour is preserved while the option list itself becomes data-driven
  from `menus/pom.toml`.
- `show_calendar` defaults to true (so POM and Settings show the calendar with
  no config change); a menu author sets `show_calendar = false` for a
  full-width, calendar-less menu.
- Focus/keyboard model (Tab into option rows, calendar `<`/`>` hotspots, POM
  option focus reversal) is preserved by the shared renderer.

Migration/back-compat: `pom.toml` is written from `DEFAULT_POM_TOML`
(cv-requirements Req 7) via `write_if_absent`; existing user `pom.toml` files are
not overwritten. The `primary_option_menu` module's pure calendar/date helpers
are retained (moved or re-exported) since the shared renderer depends on them;
only its bespoke top-level `render` layout is superseded.

### Design Delta: POM rendered via the shared renderer (Requirement 2.1c-2.1h, task 22.4)

Investigation of the live code (Phase pom-via-shared-renderer) established that
the POM is NOT a thin wrapper today: its 12 options are compiled into
`primary_option_menu::BUILT_IN_OPTIONS`, its keyboard focus ring and Enter/Space
activation read that array directly, it has a dedicated Exit line, and its tab
carries `menu_workspace: None`. A naive "make the POM a MenuWorkspace tab" would
regress function-keys-and-history Req 16 (focus ring), the Exit behaviour, the
`[POM]` tab title, the black/blue Title_Line, and about six tests. It would also
mis-dispatch, because the existing `DEFAULT_POM_TOML` command strings do not all
resolve (`CATALOGS` and `TERMINALS` have no matching `handle_command` arm; the
digit `6` maps to the Macro Library, not "Terminals").

Chosen approach (preserves all behaviour; Req 2.1d-2.1h):

- KEEP `TabKind::PrimaryOptionMenu` as the POM tab kind. Attach a
  `MenuWorkspaceState` (loaded from `menus/pom.toml`) to the POM tab -- the tab
  gains a populated `menu_workspace: Some(state)`. The tab title stays `[POM]`
  and the Title_Line stays POM-styled (Req 2.1d). `insert_pom_tab` /
  `ensure_pom_tab_present` load the POM menu-workspace state.
- In the `TabKind::PrimaryOptionMenu` render arm, call the SHARED
  `render_menu_workspace(state, ui, calendar_offset, menu_colours)` against the
  POM's `MenuWorkspaceState`, replacing the `primary_option_menu::render` call.
  A clicked option flows into `pending_menu_option` (same as any Menu_Workspace)
  and dispatches through the existing deferred path.
- PORT the focus ring to the loaded options (Req 2.1e): `FocusStop::next/prev`
  and the Enter/Space handler read the POM tab's `menu_workspace` option count
  and the selected option's `command`, instead of `BUILT_IN_OPTIONS.len()` and
  `BUILT_IN_OPTIONS[i].key`. When the POM has no loaded menu (load error) the
  ring degrades to CommandField + menu bar + tabs only.
- COMMAND-DRIVEN principle (Req 2.1e): selection dispatches ONLY the option's
  `command` string. DELETE the digit-keyed dispatch arms in `handle_command`
  that couple an option's key to a panel (`"1" | "=1" | "FILE CATALOGS"`,
  `"2" | "=2"`, `"3"`, `"4"`, `"6" | "MACROS"`, `"7"`, `"8" | "PLUGINS"`,
  `"9"`, `"B"`, etc.). Dispatch resolves by command NAME only. `=<key>`
  resolves to "the option whose key is `<key>`, then run its command"
  (Req 2.1i), so fastpaths stay config-driven.
- COMMAND-NAME resolution (Req 2.1h): ensure every default `pom.toml` command
  resolves by name. `SETTINGS`, `FILES`, `MACROS`, `PLUGINS`, `SEARCH`,
  `RETURN` already resolve; ADD a `CATALOGS` arm opening the File Catalogs
  Context (today only the digit `1`/`FILE CATALOGS` did). No behaviour keyed to
  the key character remains.
- TERMINATE action (Req 2.1g): a data-driven `pom.toml` option (key `X`,
  command `RETURN`); selecting it routes through `handle_command("RETURN")`,
  which returns to the POM / exits when the POM is the only Workspace
  (CR-CH-016, function-keys Req 17.3/17.4). Remove the bespoke `EXIT_LINE_TEXT`
  / `PomAction::Exit` / `FocusStop::PomExit` so terminate is one code path.
- TRIMMED default `pom.toml` (Req 2.1h): ship ONLY built + testable options.
  Kept: `0 SETTINGS`, `1 CATALOGS`, `2 FILES`, `5 MACROS`, `8 PLUGINS`,
  `S SEARCH`, `X RETURN`. Removed (add back when built/tested): `3 Utilities`,
  `4 Compilers`, `6 Terminals`, `7 Databases`, `9 Jobs`, `B Batch`. Keys are
  NOT renumbered (0/1/2/5/8/S/X retained) to preserve muscle memory and `=N`.
- RETIRE `primary_option_menu::render` (the bespoke top-level layout),
  `BUILT_IN_OPTIONS`, `PomAction`, `PomRenderResult`, and `EXIT_LINE_TEXT` once
  the POM renders through the shared path and its tests are migrated. The pure
  calendar/date helpers and `PomColours` stay (the shared renderer uses them).
- Session/startup: the POM tab is still guaranteed by `ensure_pom_tab_present`;
  it now also ensures the POM `MenuWorkspaceState` is loaded. Persistence is
  unchanged (POM is guaranteed, not restored from a descriptor).

Risk controls: implement behind the existing tests, migrate the focus-ring and
option-dispatch tests to assert against the loaded pom.toml options, and verify
`=0`/`=1`/`=2`, Tab cycling, Enter activation, the calendar, and END-terminate
all still work before retiring the bespoke module.

---

## Design Delta: Code-only menus + Recovery Baseline + configurable group separator (CR-CH-021)

This delta REVISES Section 9 (Default Menu Files) and Section 6 (Rendering group
separator), and adds the Recovery_Baseline concept. It mirrors the themes
code-only decision (theme-and-appearance CR-CH-019).

### 1. Menus become code-only (revises Section 9)

`shell/update.rs` no longer calls `ensure_default_menu_files()` to WRITE
`pom.toml`/`settings.toml`. The `menus/` directory is still created (empty is
fine) so a user has a place to author menus. `ensure_default_menu_files()` is
retired for writing built-ins; if a thin dir-ensure is still wanted it becomes a
`ensure_menus_dir()` that only `create_dir_all`s the folder (no file writes),
matching `theme_defaults::ensure_default_theme_files`.

Removed: the `write_if_absent(pom.toml)` / `write_if_absent(settings.toml)`
calls and the `DEFAULT_*_TOML`-to-disk behaviour. The `DEFAULT_POM_TOML` /
`DEFAULT_SETTINGS_TOML` constants are repurposed as the compiled
Recovery_Baseline (below) and shrink to the barebones option set.

### 2. Recovery_Baseline (new; menu-workspace Req 12)

`menu_workspace::defaults` exposes:

```rust
pub fn recovery_pom_menu() -> MenuFile;       // 0 Settings,1 Catalogs,2 Files,L Log,M Menus,X Return
pub fn recovery_settings_menu() -> MenuFile;  // T Themes, M Menus, A All
```

These build a `MenuFile` directly (single group, no stray separator). The
existing `DEFAULT_POM_TOML`/`DEFAULT_SETTINGS_TOML` string constants are updated
to the same barebones content and remain the parse source used by
`parse_menu_str` fallbacks, so there is ONE source of the compiled content
(Req 12.7). `MENUS`/`M` and `LOG`/`L` rows are added; `CATALOGS`, `SETTINGS`,
`FILES`, `THEMES`, `RETURN`, `A` commands already resolve.

### 3. Fallback wiring (POM already has it; Settings gains it)

- `ensure_pom_menu_loaded()` (commands.rs) already falls back to
  `parse_menu_str(DEFAULT_POM_TOML)` when the on-disk file is missing/invalid.
  It keeps doing so; only the content changes.
- `open_settings_menu()` (commands.rs) currently surfaces `load_error` with NO
  fallback. Change: when the loaded Settings `MenuWorkspaceState` has
  `menu.is_none()`, fall back to `parse_menu_str(DEFAULT_SETTINGS_TOML)` and, if
  the on-disk file EXISTED but failed to parse, push a non-blocking notification
  (`"settings.toml: <error> -- using built-in Settings menu"`). A simply-absent
  file falls back silently (Req 12.4 vs 12.5; startup Req 11.8).
- `MenuWorkspaceState` gains no new field; the fallback is applied at the open
  site (same pattern as the POM) to keep the state type unchanged.

### 4. `MENUS` command reservation (Req 12.6)

`handle_command` gains an `upper == "MENUS"` arm that, until the Menus editor CR
lands, pushes a notification `"Menus editor is not yet available."` and returns
without changing the Workspace. The command name is reserved so the
Recovery_Baseline `M` rows dispatch cleanly.

### 5. Configurable group separator (revises Section 6, Req 2.4/4a/4b)

`RawMenuFile`/`MenuFile` gain two optional top-level fields:

```rust
group_separator: GroupSeparator,  // Line | Space | None; default Space
group_headers: bool,              // default false
```

`GroupSeparator` is a small enum deserialised from the strings `"line"`,
`"space"`, `"none"` (serde rename_all = "lowercase"), defaulting to `Space`.

`render_menu_workspace` replaces the unconditional `ui.separator()` at a group
change with:
- `Space` -> `ui.add_space(row_gap)` (a blank line; the new default),
- `Line`  -> `ui.separator()` (the previous behaviour),
- `None`  -> nothing.

The boundary is still only drawn when `current_group != last_group &&
last_group.is_some()` AND both groups are non-empty (options with no `group` do
not trigger a boundary). When `group_headers == true` and a new non-empty group
begins, a header label (menu description colour) is drawn above the group.

### 6. Why this shape

- Code-only removes the stale-file class of bug entirely: there is no on-disk
  built-in to go stale, so the reported "line between 8 and 9" (an old
  materialised pom.toml with a Core/Extended boundary) cannot recur.
- The Recovery_Baseline guarantees the app is never option-less: absent OR
  corrupt user files both resolve to a usable barebones menu, and the operator
  is told when their file was bypassed.
- `Space` default matches the ISPF grouped-list look without a heavy divider,
  and `line`/`none` remain available per menu.

## Design Delta: RESET BARE (configuration-system Req 19)

`RESET BARE` is a new shell command (`config.reset_bare`) with a confirmation
dialog and a non-destructive archive step.

### Flow

1. `handle_command` intercepts `upper == "RESET BARE"` and opens a modal
   confirmation dialog (`reset_bare_confirm_open = true`), taking no other action.
2. On Confirm, the shell calls an archive helper (in ff-desktop, or a thin
   helper in ff-session that owns the User_Data_Dir):
   `archive_config(user_data_dir) -> Result<PathBuf, Vec<String>>` which:
   - computes `config-archive/<UTC-timestamp>/` (timestamp filesystem-safe),
   - `fs::rename` (fall back to copy+remove across volumes) each of: `menus/`,
     `themes/`, `session.toml`, `config.toml`, catalog registry file, when
     present; missing items skipped; per-item failures collected best-effort.
3. The shell then resets in-memory state: reload config to defaults, drop menu
   workspace state (so the POM/Settings reload from the compiled
   Recovery_Baseline), reset the active palette to the compiled Default (Legacy
   fallback), clear the catalog registry and re-run `ensure_default_home_catalog`,
   and reopen the Home Context showing the Recovery_Baseline POM -- no process
   relaunch.
4. A Settings affordance (button/menu option) dispatches `RESET BARE` through the
   same command path (parity, workflow.md 1b); it does not bypass the dialog.

### Notes

- Archive is move-not-delete and never prunes prior archives (Req 19.8), so the
  operator can manually restore.
- Cross-volume `rename` failure is handled by copy-then-remove; a failure to
  remove the source after copy is reported but leaves a recoverable copy.

---

## Design Delta: Menus Editor Context (Requirement 13, CR-NR-075)

A new in-app editor for menu TOML files, mirroring the Theme editor
(theme-and-appearance Req 20) exactly: a pure render returning an Action, with
all side effects (validation, serialisation, file write) applied by the shell
command layer.

### 1. New MenuFile -> TOML serialiser (`menu_workspace/serialiser.rs`)

No serialiser exists today (the loader is Deserialize-only; built-in content is
hand-written string consts). Add `serialise(menu: &MenuFile) -> String` mirroring
`ff_theme::serialiser::serialise`:
- Emit `title`, then `show_calendar` / `group_separator` / `group_headers` only
  when they differ from their defaults (keeps files clean; loader defaults fill
  the rest).
- Emit one `[[options]]` block per option: `key`, `command`, `description`,
  `enabled` (only when false), `group` (only when Some), and an inline
  `[options.target]` table only when `target` is Some.
- `group_separator` serialises to its lowercase string (`line`/`space`/`none`).
- Round-trip guarantee (Req 13.10): `parse_menu_str(serialise(&m))` equals `m`.
  A property/unit test asserts this. NOTE: `ff_command::CommandTarget` must
  serialise for the inline-target case; if it does not derive `Serialize`, the
  serialiser emits only the `command` string for options that carry a target and
  logs a DEBUG note (the loader already treats an inline target as authoritative,
  Req 10.6) -- to be confirmed at implementation and reflected in the tests.

### 2. New panel (`menus_editor_panel/`, pre-split for the 400-line rule)

`menus_editor_panel/{mod.rs, state.rs, render.rs}` mirroring `theme_editor_panel`:

```rust
pub struct MenusEditorState {
    pub available: Vec<String>,     // "POM", "Settings", user menu names
    pub selected: Option<String>,   // selected menu name
    pub working: Option<MenuFile>,  // unsaved edits live here
    pub name_buffer: String,        // Save As new name
    pub error: Option<String>,      // inline validation/save error
}

pub enum MenusEditorAction {
    None,
    Select(String),
    EditTitle(String),
    SetShowCalendar(bool),
    SetGroupSeparator(GroupSeparator),
    SetGroupHeaders(bool),
    EditOption { index: usize, field: OptionField, value: String },
    SetOptionEnabled { index: usize, enabled: bool },
    AddOption,
    DeleteOption(usize),
    MoveOptionUp(usize),
    MoveOptionDown(usize),
    Save,
    SaveAs(String),
}
```

`render(ui, state) -> MenusEditorAction` is pure. It uses the same two-slot
pattern as the Theme editor (B052): an explicit `action` for button clicks
(add/delete/move/save/selector) that WINS over a `field_action` produced by a
row text-field's commit-on-`lost_focus`, so clicking a button never loses to a
same-frame field commit.

### 3. Shell wiring (`shell/menus_editor.rs`, a new shell submodule)

To avoid growing the already-large `shell/commands.rs`, put the editor impl
methods in a new `shell/menus_editor.rs` (mirroring `shell/reset_bare.rs`):
- `open_menus_editor()`: build `available` from `["POM","Settings"]` + user
  files in `menus_dir()`; load the selected menu's working copy (built-in ->
  Recovery_Baseline when no file); `transform_active_pom_tab(TabKind::MenusEditor,
  "[MENUS]")` on a POM tab else `open_menus_editor_tab`.
- `apply_menus_editor_action(action)`: mutate `working` for edit/add/delete/move
  actions; `Save`/`SaveAs` VALIDATE (shared rule with the loader -- see below)
  then `write_menu_file(name, &menu)`; refresh the available list.
- `write_menu_file(name, menu)`: `create_dir_all(menus_dir())`, serialise, write
  `menus/<menu_slug(name)>.toml` (POM/Settings map to pom.toml/settings.toml).
- Add a `menus_dir_override: Option<PathBuf>` field on the shell (mirroring
  `themes_dir_override`) so save-file tests are isolated to a TempDir; production
  is `None` (real user dir).

Shared validation (Req 13.7/13.13): factor the loader's per-option checks into a
reusable `menu_workspace::loader::validate_menu(&MenuFile, limits) -> Result<(),
String>` that both the (existing) load path and the (new) editor Save path call,
so the editor cannot produce an unloadable file. This is a refactor of the
existing `validate_option` logic into a MenuFile-level check; the load path keeps
its current behaviour.

### 4. Tab + command + return wiring

- `TabKind::MenusEditor` in `tab_state.rs`; `TabState::menus_editor(id, document)`
  with title `"[MENUS]"`; `tab_manager::open_menus_editor_tab` (dedup by kind).
- Replace the `upper == "MENUS"` notice arm (shell/commands.rs) with
  `self.open_menus_editor();`. Both the POM `M` row and the Settings `M` row
  dispatch `MENUS`, so both entry points light up automatically.
- Add `TabKind::MenusEditor` to the END/RETURN return-to-POM branch so F3/END
  from a transformed POM restores the Home Context.
- Render dispatch: a `TabKind::MenusEditor` arm in `shell/render.rs` calling
  `menus_editor_panel::render` and `apply_menus_editor_action`.

### 5. Closing CR-CH-021 task 23.9 (RESET BARE affordance)

Add a Settings-side affordance (a button in the Menus/Settings area, or a
`RESET BARE` option row) that dispatches the existing `RESET BARE` command
through the command path -- it opens the confirmation dialog, it does NOT call
the archive/reset internals directly (command parity, workflow.md 1b). The
command and dialog already exist (CR-CH-021); this only adds the affordance.

### 6. Hot-reload interaction (Req 13.11)

No new mechanism: the editor writes `menus/<name>.toml`; the existing
`MenuWorkspaceState::poll_reload` (mtime-based) picks up the change for any open
Menu_Workspace (including the POM/Settings) within the existing reload window.
The editor is not itself a Menu_Workspace, so it does not poll; it re-reads on
Select.

---

## Design Delta: Per-Tab Navigation_Stack (Requirement 14, CR-CH-022)

Replaces three ad-hoc END mechanisms with one uniform per-tab stack. Grounded in
the investigation: only `TabState.menu_workspace` is per-tab; all other Context
editing state is shell-global, so a stack entry must carry the params to
re-derive the shared state. `WorkspaceDescriptor` (ff-session) already does this
and is the entry type.

### 1. Data model

Add to `TabState` (tab_state.rs):

```rust
/// Ordered ancestors of the current Context (most-recent last). The current
/// Context is NOT on the stack. Empty = this tab is at its root.
pub nav_stack: Vec<ff_session::WorkspaceDescriptor>,
```

Initialised empty in every constructor / `base_tab!`. A POM tab created by
`START` (no arg) has an empty stack. A tab created by `START =0` has
`[POM-descriptor]` after drilling to Settings.

A helper `current_descriptor(&TabState) -> WorkspaceDescriptor` derives the
descriptor for the tab's CURRENT Context (reusing the existing
`descriptor_for_tab` logic in session_manager.rs; for transient kinds
ThemeEditor/MenusEditor that currently return None, add a descriptor so they can
sit on the stack -- a lightweight `CustomWorkspace { workspace_kind }` with the
kind, since their editing buffer is shell-global and re-derived on reconstruct).

### 2. The single navigation primitive

`TabManager::navigate_here(descriptor, push: bool)`:
- if `push`, `self.active_tab_mut().nav_stack.push(current_descriptor(active))`,
- then reconstruct `descriptor` IN PLACE on the active tab (set kind, title, and
  per-tab `menu_workspace` when it is a Menu; the shell re-derives shell-global
  state for Settings/Theme/Menus/etc. exactly as the open_* helpers do today).

Every navigation arm in `handle_command` (SETTINGS, A, SETTINGS ns, FILES,
=FILES, CATALOGS, PLUGINS, MACROS, THEMES, MENUS, COMMANDS, LOG, MENU, the POM
fastpath, chained paths) routes through `navigate_here` on the CURRENT tab. The
`transform_active_pom_tab` / `else { open_*_tab }` split is retired: there is no
longer an "else open a new tab" branch -- navigation always transforms in place.
The `open_*_tab` methods remain only for START and session-restore (and the
dedup-by-kind logic is no longer used for navigation).

Because the shell applies shell-global Context state, the shell wraps
`navigate_here` in per-Context helpers (e.g. `nav_to_settings(namespace, push)`
sets `settings_panel.namespace_filter`/`filter` then calls `navigate_here`),
so the existing re-derivation code (open_settings_view body, open_theme_editor
body, open_menus_editor body) is reused, not duplicated.

### 3. END / RETURN

`handle_command` END arm becomes uniform:

```text
END:
  if active.nav_stack is empty:
      if tabs.len() <= 1: file.exit           # last workspace -> terminate
      else: close_current_and_navigate_back    # close this tab
  else:
      let parent = active.nav_stack.pop()
      reconstruct(parent) in place on the active tab
```

RETURN arm:

```text
RETURN:
  if active.nav_stack is empty: (same as END-at-root)
  else:
      let root = active.nav_stack.drain(..).next()   # bottom entry
      clear the stack; reconstruct(root) in place
```

The `pending_return_to_pom` flag (mod.rs + update.rs processing), the
`namespace_filter.is_some()` END branch, and the `opened_from_settings` field
(menus_editor_panel) are DELETED. Files-panel `ReturnToPom` action instead calls
the same END-pop path.

Reconstruction of a parent descriptor reuses the open_* helper bodies (kind,
title, per-tab menu_workspace, shell-global params). A `.` collapse pushes
nothing (so END skips it); a `;` push adds the intermediate; per Req 5.

### 4. START argument parsing

`START` arm:
- no arg -> `insert_pom_tab` (new POM tab, empty stack) -- unchanged.
- `START =<path>` -> `insert_pom_tab`, then apply the chained path to the NEW tab
  (POM becomes the stack bottom via the `=` origin rule, Req 14.7).
- `START <arg>` (no `=`) -> create a new tab, resolve `<arg>` (POM option key via
  `resolve_pom_option_key`, else a known command), and root the new tab DIRECTLY
  at that Context with an EMPTY stack (no POM beneath). Unresolved -> new POM tab
  + status message.

A small `TabManager::insert_rooted_tab(descriptor)` creates a new tab rooted at a
given Context with an empty stack (used by `START <arg>`).

### 5. Session persistence interaction

The per-tab `nav_stack` is `Vec<WorkspaceDescriptor>` -- already serialisable.
Extend `SessionTabState` with an optional `nav_stack` field (default empty) so a
drilled-in Workspace restores its back-path; older sessions load with an empty
stack (graceful). Not required for the core behaviour; called out so restore
does not silently flatten stacks.

### 6. Why WorkspaceDescriptor (not a bespoke enum)

It already exists, already round-trips through TOML, already carries the exact
params that vary (Settings namespace, Menu name, Editor uri), and is what
session-restore reconstructs from -- so navigation and restore share one
reconstruction path. The only additions are descriptors for ThemeEditor /
MenusEditor (kind-only) so they can appear on a stack.
