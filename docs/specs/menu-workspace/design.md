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

---

## 15. Menu Workspace Tab Order and Focusable Calendar (Requirement 15, CR-CH-023)

Design delta to Section 6 (Rendering) and the calendar helpers.

### Shared tab-order model
The Menu_Workspace does NOT own a focus ring. It relies on the shared shell model
(menu-and-statusbar Requirement 16, unified per CR-CH-023): the shell Boundary_Policy handles
command-line entry, menu-bar-last, and wrap; the interior order is whatever egui-native
traversal produces over the controls as `render_menu_workspace` creates them. The prior
`FocusStop::PomOption`/`CalendarPrev`/`CalendarNext` enumeration is removed.

### Option rows (already correct)
Enabled options are already real focusable `egui::Button`s (transparent fill, no stroke), so
egui walks them in declared order for free. Disabled options are `ui.add_enabled(false, Label)`
and are non-focusable, so Tab skips them (Requirement 15.4). No change needed to make options
tab-reachable; the change is the REMOVAL of the shell ring that previously drove their focus.

### Calendar: painted hotspot -> two focusable buttons
Today `primary_option_menu::render_calendar` draws the month header and detects `<`/`>` by
hit-testing which sixth of the header rect was clicked, returning `Option<CalendarNav>`. Under
CR-CH-023 the header renders two REAL `egui::Button`s labelled `<` and `>` (created after the
option column and before any trailing content), each returning its `CalendarNav` on click OR on
keyboard activate (Enter/Space) when focused (Requirement 15.7, 15.8). Because they are ordinary
focusable buttons created in visual order, egui-native traversal reaches them right after the
last option (Requirement 15.5). The `pom_calendar_offset` state and `CalendarNav::Prev`/`Next`
side effect are unchanged; only the input surface changes from a hit-region to two buttons. The
optional legacy click-region MAY be retained for mouse users but MUST produce the same month
change. Day cells remain non-focusable (Requirement 15.10) -- the calendar stays minimal pending
a future redesign.

### No new crate dependency
Uses existing egui button/focus APIs and the existing calendar helpers.
---

## Design Delta: Keyword-less menu-name resolution + Settings baseline reorder (Requirement 3.6, 11.11-11.12, 12.3, CR-CH-025)

This delta REVISES Section 7/13 (Command Dispatch / Command Target Integration) and the
Recovery_Baseline content (Section "Code-only menus"). It removes the hardcoded `SETTINGS`
special-casing and reorders the compiled Settings baseline. No new module or Workspace kind.

### 1. Menu name is resolved by the chain, not a hardcoded verb

The bare-token menu-name form (Requirement 11.11) is implemented as stage 3 of the unified
command-resolution chain (command-framework Req 8.3 / design delta). When a typed token is
not claimed by the current-menu Option_Key stage or the built-in/Command_ID stage, the shell
asks the resolver whether the token is a resolvable Menu_Name (a user `menus/<name>.toml`
that exists, or a compiled built-in `pom`/`settings`); if so it dispatches
`Menu_Target { name }` via the existing `open_menu_by_name`. A trailing token is forwarded as
the chained Option_Key (the MENU Argument Chaining delta above), so `SETTINGS T` == open
Settings + activate `T`.

Consequently the following hardcoded `handle_command` intercepts are REMOVED:
`if upper == "SETTINGS"`, `if upper.starts_with("SETTINGS ")`, and `if upper == "A"`. Opening
the Settings menu is now stage-3 resolution of `SETTINGS`, identical to `POM` and to any user
menu. `open_settings_menu()` is retained as the helper the Menu_Target/`open_menu_by_name`
path calls for the `settings` name; `open_settings_view(..)` is superseded by the `CONFIG`
command handler (configuration-system Req 20).

Precedence (Req 11.12 / command-framework Req 8.10): built-ins beat same-named menus, so a
user cannot shadow a core verb by naming a menu after it. The current-menu Option_Key lookup
stays first, so an option key like `T` on the Settings menu still runs `THEME` before any
global interpretation.

### 2. Option-key lookup becomes the first chain stage, not a terminal check

Section 7's on-menu block (`if active tab is MenuWorkspace { find_option(..) }`) is MOVED to
the TOP of `handle_command` (before the built-in intercepts) so it is stage 1 of the chain.
Requirement 3.6 changes accordingly: a non-matching token no longer errors immediately -- it
falls through to the remaining stages (built-in, menu name, macro) and only errors when NO
stage resolves it.

### 3. Settings Recovery_Baseline reorder (Requirement 12.3)

`DEFAULT_SETTINGS_TOML` in `menu_workspace/defaults.rs` is rewritten to:

```toml
title = "Settings"
group_separator = "line"

[[options]]
key = "A"
command = "CONFIG"
description = "All settings -- browse every configuration key"
group = "Core"

[[options]]
key = "T"
command = "THEME"
description = "Theme editor -- copy, edit, save and select themes"
group = "Core"

[[options]]
key = "M"
command = "MENUS"
description = "Menus editor -- create, change and save menus"
group = "Core"

[[options]]
key = "R"
command = "RESET BARE"
description = "Reset to barebones -- archive config and start fresh"
group = "Recovery"
```

Order A, T, M (group `Core`) then R (group `Recovery`); the group change draws a boundary
(Requirement 2.4). Corrects the stale `T -> THEMES` to `T -> THEME` (CR-CH-024) and replaces
`A -> A` with `A -> CONFIG`. Built-ins stay CODE-ONLY (CR-CH-021); RESET BARE is UNCHANGED
(archive/move, no file writes). `recovery_settings_menu()` continues to parse this single
constant (Req 12.7).

### No new crate dependency
Reuses `open_menu_by_name`, `find_option`, `menus_dir`, and the existing chained-key helper.

---

## Design Delta: Calendar Visibility and Fit (Requirement 16, CR-CH-026, B060)

Design delta to Section 6 (Rendering), the "Unified config-driven menu renderer"
delta, and Section 15 (Tab order). Fixes B060.

### Problem

`render_menu_workspace` renders the option list inside
`ui.horizontal_top(|ui| { ui.vertical(|ui| ScrollArea::vertical(...)) ; add_space(32) ; calendar })`.
The `ScrollArea::vertical()` has no width constraint, so the option column takes
its natural width; the calendar is then placed to its right at a FIXED offset
that does not reflow to the visible clip width. In a workspace narrower than the
option-list natural width, the calendar (and its focusable `<`/`>` buttons) is
positioned past the right clip edge: invisible but still in the Tab ring -- two
phantom stops after the last option (confirmed empirically: the `>` button x
stays fixed regardless of panel width and falls outside the clip rect at narrow
widths).

### Fix

1. Settings default: `DEFAULT_SETTINGS_TOML` in `menu_workspace/defaults.rs` adds
   `show_calendar = false` (Requirement 16.1). The `MenuFile` field default stays
   `true` (the POM and any author-enabled menu keep it). The serialiser already
   omits `show_calendar` only when it equals the default `true`, so an explicit
   `false` is written -- no serialiser change required, but a round-trip test is
   added.

2. Layout: reserve the calendar column FIRST from the right within the visible
   width, then give the option list the remaining width. Concretely, compute the
   available width at the top of the option/calendar row; define
   `CALENDAR_MIN_WIDTH` (approx 160-180px, the `<  Month YYYY  >` header + 7-column
   grid) and a minimum readable option-list width. WHEN `show_calendar` is true
   AND `available_width >= option_min + gap + CALENDAR_MIN_WIDTH`, render the
   calendar in a right-hand column sized to `CALENDAR_MIN_WIDTH` and constrain the
   option `ScrollArea` to the remaining width (Requirement 16.2, 16.5, 16.6).
   Options: `ui.columns`-style split, or an explicit right-to-left allocation
   (`ui.with_layout(Layout::right_to_left(...))` for the calendar, remainder for
   the options), or a `ScrollArea::vertical().max_width(option_width)`. The chosen
   approach must keep the calendar rect inside `ui.clip_rect()`.

3. Fit gate + focus contract: WHEN `show_calendar` is true BUT
   `available_width < option_min + gap + CALENDAR_MIN_WIDTH`, OMIT the calendar
   for that frame (Requirement 16.3, mirroring the deferred CR-NR-059
   responsive-hide) and set `result.last_interior_id = last_enabled_option_id`
   (do NOT report the calendar `<`/`>` ids). The existing `else` branch (when
   `show_calendar` is false) already does this; the new narrow-fit branch joins
   it. Net effect (Requirement 16.4): the calendar contributes a Tab stop ONLY
   when it is actually displayed within the visible area.

### Tests (egui_kittest, per testing.md)

- `full_shell_settings_first_to_last_tab_walks_options_only` (full-shell, Settings
  workspace): from the command field, Tab walks exactly the option rows then the
  menu bar; the reported `last_interior_id` is the last option, not a calendar id
  (Requirement 16.1, 16.4).
- `menu_calendar_shown_when_wide_next_button_is_on_screen`: a calendar-on menu at
  a wide size reports the `>` id AND its rect is within `ui.clip_rect()`
  (Requirement 16.2, 16.5) -- the regression guard for B060.
- `menu_calendar_omitted_when_too_narrow_no_calendar_tab_stops`: the same menu at
  a narrow size omits the calendar and reports the last option as last interior,
  with no calendar ids (Requirement 16.3, 16.4).
- Serialiser round-trip: `show_calendar = false` on Settings survives
  serialise/parse.

---

## Design Delta: Description-driven Menu Layout (Requirement 16.7-16.11, CR-CH-032)

Design delta to the CR-CH-026 delta above and Section 6 (Rendering). It REPLACES
the fixed-minimum-constant fit rule (`OPTION_LIST_MIN_WIDTH + GAP +
CALENDAR_MIN_WIDTH`) with a decision driven by the descriptions' natural one-line
width.

### Root cause of the current behaviour

`render_menu_workspace` currently computes:
```
display_calendar   = show_calendar && available_w >= OPTION_LIST_MIN_WIDTH + GAP + CALENDAR_MIN_WIDTH
option_list_max_w  = display_calendar ? (available_w - GAP - CALENDAR_MIN_WIDTH).max(OPTION_LIST_MIN_WIDTH) : INFINITY
```
The option `ScrollArea` uses `.max_width(option_list_max_w)` / `set_max_width`, a
MAX not a fixed width, so it shrinks to content when descriptions are short --
the calendar then follows the shrunk column, leaving blank space to its right.
And when descriptions are long the fixed cap wraps them EVEN when hiding the
calendar would have freed enough width to keep them on one line. Neither matches
the owner's intent.

### DM.C1 Natural width measurement (new pure helper)

Add `natural_option_list_width(options: &[MenuOption], cmd_width: usize, fonts:
&egui::text::Fonts) -> f32`: for each option lay out the prefix (`option_prefix_job`
text) and the description as NON-WRAPPING galleys in `option_font()`, take the max
row width (prefix galley width + description galley width), and add a scrollbar
allowance (`ui.spacing().scroll.bar_width` + a small pad). Uncapped
(Requirement 16.7). It reuses the existing `option_prefix_job` / `command_column_width`
so the measured prefix matches what is painted. A thin pure variant that takes the
per-row measured widths is unit-testable without a live `Ui` (the monospace metric
is deterministic under the test fonts).

### DM.C2 Three-tier decision (replaces display_calendar/option_list_max_w)

Compute once per frame from `available_w` = `ui.available_width()`, `N` =
Natural_Option_Width, `G` = `CALENDAR_GAP`, `C` = `CALENDAR_MIN_WIDTH`:
```
Tier 1 (calendar + one-line):  show_calendar && N + G + C <= available_w
    -> display_calendar = true;  option_col_w = N
Tier 2 (one-line, no calendar): else if N <= available_w
    -> display_calendar = false; option_col_w = available_w
Tier 3 (wrapped, no calendar):  else
    -> display_calendar = false; option_col_w = available_w   (descriptions wrap)
```
`option_col_w` is applied to the option column as a DEFINITE width (`ui.set_width`
/ `ScrollArea::max_width(option_col_w)` combined with an inner `set_width`) so:
- in Tier 1 the column is exactly `N` (leftover width trails as blank space to the
  RIGHT of the calendar -- the calendar is NOT pinned to the window edge,
  Requirement 16.8 Tier 1);
- in Tiers 2/3 the column is the full width.

The calendar block is UNCHANGED (`add_space(GAP)` + `render_calendar(...)`), still
gated on `display_calendar`; the focus-contract branch (`last_interior_id` = `>`
id when shown, else last option) is UNCHANGED and now simply keys off the
tier-derived `display_calendar` (Requirement 16.4, 16.5 preserved).

### DM.C3 Wrap stays a fallback (Requirement 16.10)

The row-render is UNCHANGED (B065: prefix Button + `Label::new(...).wrap()`). No
wrap-mode flag is introduced. Because Tiers 1/2 give the column `>= N`, the
`.wrap()` never fires there; in Tier 3 it fires naturally. A mis-measurement
degrades into a wrap rather than clipping (fault tolerance).

### DM.C4 Ordering invariant (Requirement 16.9)

Because Tier 1 is the ONLY tier that shows the calendar and it requires the FULL
one-line width `N` to also fit, the renderer can never be in "calendar shown AND
a description wrapped": hiding (Tier 2) always precedes wrapping (Tier 3).

### Tests (egui_kittest + pure, per testing.md)

- `natural_option_list_width_*` (pure): widest row drives the width; longer
  descriptions increase it; scrollbar allowance included.
- `menu_wide_shows_calendar_and_one_line_descriptions` (harness, wide size): a
  calendar-on menu with a long description reports the `>` id (calendar shown) AND
  the option-column rect width is approximately `N` (< available), leaving trailing
  space (Tier 1).
- `menu_medium_hides_calendar_keeps_one_line` (harness, width between `N` and
  `N+G+C`): calendar omitted (no `>` id), option column spans full width, no wrap
  (Tier 2).
- `menu_narrow_hides_calendar_and_wraps` (harness, width `< N`): calendar omitted,
  descriptions wrap (Tier 3). Assert calendar-hidden before wrap via the tier
  crossing (no frame shows calendar + a wrapped row).
- Existing B060 tests (`menu_calendar_shown_when_wide_next_button_is_on_screen`,
  `menu_calendar_omitted_when_too_narrow_no_calendar_tab_stops`) continue to hold
  under the new decision (retargeted widths if needed).

---

## Design Delta: Configurable Named Menu Bars (Requirement 17, CR-NR-080)

The menu bar becomes a Menu_File rendered horizontally. The menu MODEL and
command RESOLUTION are unchanged: nesting is already expressed through command
resolution (an option whose command names a menu -- Requirement 3/5/11 -- and
chained fastpaths like `=0.K`, B061). This delta is a RENDER + configuration +
assignment change, plus one net-new dynamic-options hook. Design is written
whole for Requirement 17; implementation is SLICED (A first).

### DM.1 Slice A -- data-driven horizontal render (the immediate work)

Today `WorkbenchShell::render_menu_bar` (`shell/render_chrome.rs`) hand-wires 13
`ui.menu_button(...)` calls and `super::MENU_BAR_TOP_LEVEL_LABELS` is a fixed
array asserted by a `debug_assert_eq!`. Slice A replaces that with a render
driven by a Menu_File:

- NEW compiled default `DEFAULT_MENUBAR_TOML` in `menu_workspace/defaults.rs`
  (code-only, parsed once via `parse_menu_str`, mirroring `DEFAULT_POM_TOML` /
  `DEFAULT_SETTINGS_TOML`, with a `default_menubar_menu()` accessor and an
  ASCII-only + valid-TOML unit test). Its top-level options reproduce the current
  bar's entries in order and INCLUDE a trailing `Help` entry (Requirement 17.2).
  Each top-level option's `command` names the submenu it opens (e.g. Settings
  option command = `SETTINGS`, resolvable to the Settings menu), so peeking can
  resolve the referenced menu's options.
- NEW render `render_menu_bar_from_menu(ctx, menu: &MenuFile, ...)` (in
  `menu_workspace/render.rs` or a `menu_bar` submodule): `egui::TopBottomPanel::
  top("menu_bar")` + `egui::menu::bar`, iterating `menu.options` in order. For
  each top-level option it draws a `ui.menu_button(option.description, |ui| {
  ... })`. Inside the button (Requirement 17.3, PEEK): resolve the option's
  command to a menu (reuse the resolver used by `try_menu_name_dispatch` --
  `ff_command::TargetResolver::menu_name_target` -- and load that menu's options
  via the loader / compiled default); render each of the referenced menu's
  options as `ui.button(child.description)`. On click of a child (Requirement
  17.4): `self.handle_command(&child.command)` then `ui.close_menu()`. WHERE a
  top-level option's command does NOT resolve to a menu, render it as a direct
  `ui.button` dispatching its own command.
- Keyboard nav inside a dropdown is egui-native (Requirement 17.5): `menu_button`
  dropdowns already support arrow/Enter/Escape; no bespoke handling.
- Boundary_Policy (Requirement 17.6, CR-CH-023): capture the FIRST top-level
  button's `response.id` into `self.menu_first_id` and the LAST into
  `self.menu_last_id` (today only `menu_first_id` = Settings is captured; the
  last must also be captured from the data-driven loop -- the LAST option, which
  is `Help`). The existing `MENU_BAR_TOP_LEVEL_LABELS` array + `debug_assert_eq!`
  are removed or replaced by "the bar has >= 1 top-level option"; tests that
  assert specific labels are retargeted to assert against the Default_Menu_Bar's
  option list instead.
- The vertical POM/Settings Menu_Workspace render is untouched (Requirement 17.7).

Borrow note: peeking calls `self.handle_command` from within the closure passed
to `menu_button`; follow the existing pattern (the current bar already calls
`self.handle_command("THEME ...")` inside `menu_button` closures), so no new
borrow structure is needed.

### DM.2 Slice B -- named + editable (later)

A menu-bar file is just `menus/<name>.toml` loaded via the existing loader; a
user file overrides `DEFAULT_MENUBAR_TOML` (code-only fallback, CR-CH-021). It is
editable via the existing Menus Editor (Requirement 13) + serialiser (CR-NR-075).
Convention `MB-<name>` (e.g. `MB-POM`), NOT enforced. Add a `menubar_name` the
shell resolves to a file (default when absent).

### DM.3 Slice C -- per-kind assignment (later)

Mirror the keymaps per-kind pattern (CR-CH-027): a config mapping workspace-kind
(context name, Requirement 14.6) -> menu-bar name. The active tab's kind selects
the bar; a kind with no assignment uses the default bar name. Resolution mirrors
`keymaps_dir` / `context_name_for_kind`.

### DM.4 Slice D -- dynamic option sources (later; delivers ex-CR-NR-077)

A top-level (or nested) option MAY carry a dynamic-source marker (e.g. an option
whose command is `THEME LIST`, or a `source = "themes"` field). When peeked, the
dropdown items are generated at runtime: for themes, one `ui.button(name)` per
`ff_theme::list_all_themes(&self.themes_dir())` entry, each dispatching
`THEME <name>` via `handle_command` (Requirement 17.10, 17.11; command parity).
This uses the shared `set_active_theme` apply+persist path and delivers the
theme-and-appearance Req 17.8-17.13 behaviour without a bespoke popup. The
command-line `THEME LIST` (optional) can render the same generated list as a
centred popup for parity, but the menu-bar dropdown is the primary presentation.

### DM.5 No model / resolution change; no contradiction

- Requirement 1 (Menu_File format), 3/5/11 (option -> command -> menu
  resolution), 12 (compiled defaults), 13 (Menus Editor), 15/16 (Tab order /
  calendar) are unchanged. Requirement 17 ADDS a horizontal render role + naming
  + per-kind assignment + dynamic sources.
- The PEEK behaviour is the one behavioural distinction from normal option
  activation: a top-level bar button shows a submenu's options in place instead
  of navigating. Leaf activation is identical to normal command dispatch.
- CR-CH-023 Boundary_Policy is preserved by capturing first/last button ids from
  the data-driven loop (DM.1).

---

## Design Delta: Unified Menu Workspace -- remove TabKind::PrimaryOptionMenu (Requirement 18, CR-NR-082 Slice 1)

Behaviour-preserving unification: the POM becomes the Menu Workspace whose menu
name is `pom`. Owner decision (b): fully remove the separate POM tab kind.

### As-is (the duplication being removed)

- `crates/ff-desktop/src/tab_state.rs`: `TabKind` has BOTH `PrimaryOptionMenu`
  and `MenuWorkspace`; both `TabState` variants already carry
  `menu_workspace: Option<MenuWorkspaceState>`.
- `crates/ff-desktop/src/shell/render.rs`: TWO near-identical match arms
  (`TabKind::PrimaryOptionMenu` and `TabKind::MenuWorkspace`) both call
  `render_menu_workspace`; the POM arm additionally calls
  `ensure_pom_menu_loaded()`.
- `shell/commands.rs::ensure_pom_menu_loaded` seeds `menus/pom.toml` with the
  `recovery_pom_menu()` barebones fallback.
- Persistence divergence (`session_manager.rs::descriptor_for_tab`): POM ->
  `CustomWorkspace { WorkspaceKind::PrimaryOptionMenu }`; other menus ->
  `Menu { name }`.
- Three enums duplicate the POM: `TabKind::PrimaryOptionMenu`,
  `ff_session::WorkspaceKind::PrimaryOptionMenu`,
  `PersistedTabKind::PrimaryOptionMenu`. Only `TabKind` has `MenuWorkspace`.

### To-be

- Remove `TabKind::PrimaryOptionMenu`. The Home Context is a `TabKind::MenuWorkspace`
  tab whose `menu_workspace` menu name is `pom`. `TabState::pom(...)` becomes a
  thin constructor that builds a MenuWorkspace tab tagged as the Home menu
  (menu name `pom`) -- or `insert_pom_tab` builds a MenuWorkspace tab and seeds
  it via the shared menu-load-with-barebones-fallback helper.
- Collapse the two render arms into ONE `TabKind::MenuWorkspace` arm. The
  barebones-fallback seeding (`ensure_pom_menu_loaded`) generalises to "ensure
  this menu workspace has a loaded menu; if its name is `pom` and no file, use
  `recovery_pom_menu()`" -- applied for any menu workspace whose tab has no
  loaded `menu_workspace` yet. The `is_pom` title-line special-case keys off the
  menu name (`pom`) instead of the removed tab kind.
- Keymap context: `context_name_for_kind` no longer has a `PrimaryOptionMenu`
  arm; the Home Context resolves to the `pom` keymap context by menu name (a
  `context_name_for_menu(menu_name)` helper, or the MenuWorkspace arm returns
  `pom` when the menu name is `pom`, else `menu`). No key binding changes.
- nav_stack.rs POM fallback (`set_active_tab_context(PrimaryOptionMenu, "[POM]")`
  + `ensure_pom_menu_loaded`) becomes "open/transform to the Home Menu Workspace
  (menu `pom`)". END/RETURN unwind to Home unchanged.
- Persistence: the Home Context persists as `WorkspaceDescriptor::Menu { name:
  "pom" }` like any menu; `descriptor_for_tab`'s POM arm is removed.
  `WorkspaceKind::PrimaryOptionMenu` and `PersistedTabKind::PrimaryOptionMenu`
  are RETAINED for reading legacy sessions (`from_legacy` maps a legacy POM to
  the Home Menu descriptor), but NEW saves never emit them. This keeps
  backward-compatible load (Req 21.10) while removing the runtime POM kind.
- The three enums: `TabKind` loses `PrimaryOptionMenu`. `WorkspaceKind` and
  `PersistedTabKind` keep their POM variants ONLY as legacy-read mappings
  (documented), mapping to the unified Menu Workspace on restore.

### Behaviour preservation + testing

Every existing POM/menu test must pass, adjusted only where it names
`TabKind::PrimaryOptionMenu` (retarget to the Home Menu Workspace: kind
`MenuWorkspace` + menu name `pom`). Add a test asserting the Home Context is a
single MenuWorkspace (no `PrimaryOptionMenu` kind exists), that a fresh launch
seeds the barebones POM menu, that END/RETURN returns to the Home Menu Workspace,
and that a legacy session with a persisted POM restores the Home Context. No new
user-visible menu behaviour. Slices 2-4 (named workspaces, per-workspace
menu-bar/keymap, Profile store) build on this single kind.

---

## Design Delta: One Option-Selection Path (Requirement 19, CR-CH-043)

Behaviour-preserving convergence of the divergent option-dispatch paths onto ONE
`handle_command`-based path. Pairs with command-framework Requirement 14 (the
command-framework half). No new user-visible option behaviour; it removes the
per-affordance and POM-vs-non-POM forks that caused the B056-B059 / B075 class.

### As-is (the divergence being removed)

Selecting the SAME logical option can take THREE different code shapes today:

- **Typed Option_Key, non-POM menu:** `handle_command` stage 1
  `try_current_menu_option(cmd)` (`shell/commands.rs`) -- but it EARLY-RETURNS
  `false` when `active_tab().is_home`, so it never handles POM keys.
- **Typed Option_Key, POM:** falls past stage 1, through the shell intercepts, to
  `resolve_pom_option_key(&upper)` (the Navigation_Origin POM fastpath) which
  looks the key up against the loaded `pom` menu / on-disk `pom.toml` / compiled
  default and re-enters `handle_command(pom_command)`.
- **Mouse CLICK (any menu):** `shell/update.rs` (~line 331) takes
  `pending_menu_option`; if `option.target` is set it calls
  `dispatch_command_target(&target)` DIRECTLY; else
  `resolve_and_dispatch_command(&option.command)` -> on `FallThrough` ->
  `handle_command(&option.command)`. So a click runs a pre-branch (inline target
  / user-target resolution) that a typed key does not.

Menu-vs-tab placement is then chosen by a DISPATCHER-level router: the
`CommandTarget::Menu { name }` arm of `dispatch_command_target` and
`try_menu_name_dispatch` both call `open_named_menu(name)`, which switches on the
NAME -- `pom` -> Home Context, `settings` -> `open_settings_menu()` (in place),
else `open_menu_by_name()` (new tab). That router is the B075 patch: it narrowed
the click/typed divergence for Settings but left the decision in a shared branch
rather than in the command.

### To-be (one path; command owns placement)

- **One current-menu Option_Key resolver.** Replace the `try_current_menu_option`
  (non-POM only) + `resolve_pom_option_key` (POM only) pair with a SINGLE resolver
  that looks the selected key up against the ACTIVE menu regardless of `is_home`
  (POM, Settings, or user menu) and dispatches the option's `command` via
  `handle_command`. The Navigation_Origin `=` fastpath (Requirement 5) stays a
  command-STRING parsing concern feeding the same resolver, not a second path;
  `=`-origin chains against the POM (e.g. `=0.K`) keep resolving because the POM
  is just the active/home menu the resolver consults.
- **Click == typed.** In `shell/update.rs`, drop the `option.target` /
  `resolve_and_dispatch_command` pre-branch; a click resolves the row to its
  `option.command` and calls `handle_command(option.command)` -- identical to
  typing the key. The inline `[options.target]` capability (Requirement 10.6) is
  preserved by resolving it inside the pipeline (the resolver still consults the
  option's target when present), not by a click-only fork.
- **Command owns in-place-vs-new-tab.** Fold `open_named_menu`'s name switch into
  the command handlers: the `SETTINGS` command owns "navigate in place"
  (`open_settings_menu`), `POM` owns "Home Context", a user-menu-opening command
  owns "new tab" (`open_menu_by_name` / `open_menu_workspace_tab`). The
  `CommandTarget::Menu { name }` dispatch and the typed-name path both route
  through those commands, so the placement decision lives with the command; the
  Menu Workspace and the key-dispatch seam no longer choose it.
- **The menu is a dumb dispatcher.** After this delta the Menu Workspace's only
  job on selection is: find the option's `command` and call `handle_command`. It
  does not inspect the target kind and does not choose placement.

### Preserved invariants

- Observable B075 behaviour (clicking POM `Settings` opens Settings IN PLACE) --
  now because the `SETTINGS` command owns that effect.
- `=` Navigation_Origin semantics (Requirement 5), `<menu> <key>` chaining
  (Requirement 11.7), disabled-option message (Requirement 3.7), and the
  unresolved-command error (Requirement 3.6 / 10.5) -- all now emitted on the one
  path for every affordance.
- The workspace-conformance first-Tab focus contract is unaffected: this delta
  changes dispatch wiring, not the render arms' `first_interior_id` /
  `honour_interior_focus_latch` reporting.

### Files touched (implementation, when built)

`shell/commands.rs` (collapse the two resolvers into one; `SETTINGS`/`POM`/menu
commands own placement), `shell/target_dispatch.rs` (`Menu` arm + `open_named_menu`
folded into the command handlers), `shell/update.rs` (click path calls
`handle_command(option.command)`), `menu_workspace/nav_stack.rs`
(`open_settings_menu` in-place vs `open_menu_by_name` new-tab now selected by the
command). No new crate dependency; reuses existing helpers.

---

## Design Delta: Single config-driven centered title + short POM tab + POM command (Requirement 20, CR-CH-042)

De-duplicates the doubled Menu Workspace title into ONE centered, config-sourced
Title_Line; makes the POM tab a short `POM` label; adds a first-class `POM`
command with `START` as its alias. Pairs with menu-and-statusbar Req 17 (3/6/11
revised/added). Behaviour-preserving for options/navigation/calendar/Tab-order.

### As-is (the duplication being removed)

For a Menu Workspace the title renders TWICE:
- **Title_Line** (`shell/render.rs::render_title_line_into_ui`, ~line 132): text
  from `WorkbenchShell::kind_title` -> for a Menu Workspace it delegates to the
  free fn `title_line_text` (`shell/mod.rs`, ~line 1159). POM (`tab.is_home`) =>
  hardcoded `format!("FileForge Workbench  v{}", CARGO_PKG_VERSION)`; a non-Home
  menu => `mw.tab_title()` = `[<UPPERCASE menu.title>]`. The POM branch paints a
  black-bg / blue centered label (`is_pom`); the non-POM branch paints
  left-aligned. This is the POM-centered vs Settings-left inconsistency.
- **Menu body heading** (`menu_workspace/render.rs`, ~line 258, "Req 2.1 --
  Menu_Title centred"): `ui.vertical_centered(|ui| ui.label(RichText::new(&menu
  .title).strong().size(14.0)))` -- the raw `MenuFile.title`, centered, ABOVE the
  option list. POM => `FileForge Workbench -- Primary Option Menu`; Settings =>
  `Settings`.

The POM tab header (`shell/render_chrome.rs`, ~line 538) also routes through
`kind_title` -> the app banner, so the POM tab shows the long banner. `TabState
::pom` caches `title = "[POM]"` but that string is currently never displayed.

### To-be

- **Title_Line is the single title, sourced from the menu file, centered
  (Req 20.1/20.2, m&s 17.11).** `title_line_text` (and `kind_title`) for a
  Menu_Workspace -- INCLUDING the POM -- returns the LIVE loaded `menu.title` raw
  string (not bracketed, not uppercased), falling back to the compiled
  Recovery_Baseline title / cached title when no menu is loaded. The POM
  `is_home` early-return of the app banner is REMOVED (Req 20.3): the POM title
  now comes from `menus/pom.toml` (`FileForge Workbench -- Primary Option Menu`)
  the same way Settings comes from `settings.toml` (`Settings`).
- **One standardised, theme-driven menu heading for ALL menus (Req 20.10).**
  `render_title_line_into_ui` collapses to ONE `is_menu_workspace` branch: a
  filled heading bar + centered strong monospace title, coloured from the THEME
  tokens `primary_menu_bg` / `menu_bar_fg` (the ISPF primary-menu heading pair).
  The former POM-only HARDCODED `Color32::BLACK` / `#0055FF` literals are removed
  and there is NO `is_home` styling branch -- the POM, Settings, and user menus
  render the identical themed heading (owner: "standardise on the POM look and
  feel"). Because it is theme-token-driven, the Theme Workspace
  (theme-and-appearance Req 20) already caters for it -- editing those two tokens
  restyles the heading for every menu -- and the theme contrast guard keeps the
  `menu_bar_fg / primary_menu_bg` pair WCAG-AA legible. Non-menu Contexts (editor
  path, panel Kind title) keep their existing themed left-aligned Title_Line
  (Req 20.7).
- **Remove the duplicate body heading (Req 20.1, m&s 17.11).** Delete the
  `ui.vertical_centered(... RichText::new(&menu.title) ...)` block + its
  `add_space` in `menu_workspace/render.rs`; the option list moves up. The
  Layout_Tier / calendar-fit computation (CR-CH-032, Req 16) and the focus
  contract are untouched (Req 20.9) -- only the heading row is gone.
- **Uniform command/Menu_Name-derived tab header, NO POM branch (Req 20.4/20.6).**
  `tab_header_label` derives EVERY Menu Workspace tab header from the Menu_Name --
  the backing `menus/<name>.toml` stem, uppercased (`MenuWorkspaceState
  ::menu_name_label()`): `pom` -> `POM`, `settings` -> `SETTINGS`,
  `reports` -> `REPORTS`. The POM is NOT special-cased: there is NO `if
  tab.is_home` branch in the header path; `POM` falls out because the POM is the
  menu named `pom` (the name its opening command uses). The former app-banner POM
  header disappears as a consequence. Non-menu Contexts keep the Kind-registry
  header (`kind_title`). The only sanctioned POM special case anywhere is the
  load-time "always have >=1 POM instance" guarantee -- never in title/header
  derivation.
- **`POM` command + `START` alias (Req 20.5).** A `POM` command already exists
  (`shell/commands.rs`, `if upper == "POM"` -> `insert_pom_tab`). Make it the
  first-class Home opener and accept `START` (bare) as its alias for opening the
  Home Context, preserving START's tab-creation forms (`START =<path>` / `START
  <arg>`, menu-workspace Req 14.8-14.9). Register `POM` for dispatch parity so a
  menu option / key / typed command can invoke it. The POM tab's short label is
  derivable from this command (Req 20.6).

### Preserved invariants

- Editor Title_Line = path / `[Untitled]`; panel Title_Line = Kind title
  (Req 20.7; m&s 17.4/17.5/17.6 non-menu part).
- In-place live-derivation (m&s 17.10 / CR-CH-034): the single title is derived
  from the live loaded menu, never a stale cached string (Req 20.8).
- Option selection, `=` navigation, calendar tiers, Tab-order (Req 3/5/16/19)
  unchanged (Req 20.9).
- The application name/version is not lost; it remains available in an About
  affordance / status area (not the POM Title_Line).

### Files touched (implementation, when built)

`shell/mod.rs` (`title_line_text` / `kind_title`: Menu_Workspace incl. POM ->
live `menu.title`; remove is_home banner), `shell/render.rs`
(`render_title_line_into_ui`: center all Menu_Workspace titles),
`shell/render_chrome.rs` (POM tab short `POM` label), `menu_workspace/render.rs`
(remove the centered body-heading block), `shell/commands.rs` (`POM` command as
Home opener + `START` alias; register `POM`), `tab_state.rs` (POM short label if
needed). Docs already updated. No new crate.

### Tests

- `full_shell_pom_title_line_shows_menu_title_not_banner` -- POM Title_Line text
  == the loaded `menus/pom.toml` title, NOT `FileForge Workbench  v...`.
- `full_shell_menu_workspace_title_is_centered_for_pom_and_settings` -- both the
  POM and Settings Title_Line use the centered path (same format).
- `menu_body_has_no_duplicate_title_heading` -- the option-list render no longer
  emits the centered `menu.title` heading above the options.
- `pom_tab_header_is_short_label_pom` -- the POM tab header == `POM`, derived by
  the general Menu_Name rule (no POM branch), not the banner.
- `pom_command_opens_home_context` + `start_is_alias_of_pom_for_bare_form` --
  the `POM` command opens the Home Context and bare `START` does the same;
  `START =<path>` / `START <arg>` forms unchanged.
- Existing menu / B050 stale-title / focus-conformance tests stay green.

---

## Design Delta: POM opens via Menu_Name resolution, not a bespoke command arm (Requirement 20.5 revised, CR-CH-044)

Small delta pairing with command-framework Requirement 15. The `POM` menu-open
intercept in `handle_command` is retired; `POM` resolves as `Menu { name: "pom" }`
through the single Target_Resolution classifier (the shell resolver's
`menu_name_target` already recognises the built-in `pom` and `settings` names),
opening/returning to the Home Context via the menu opener (`open_menu_by_name`,
placement owned by the command per CR-CH-043 Req 19.5). This makes the POM open
exactly like `SETTINGS` and any user menu -- one path, no POM special case in
dispatch.

`START` is unaffected: it remains the sole tab-creator (menu-workspace Req
14.8-14.9); its handler creates the new tab and routes `<arg>` through the same
classifier. The load-time "always have >=1 POM" guarantee (Req 18.4) is the only
POM special case and is unchanged.

Behaviour-preserving: typing `POM` still opens the Home Context; the POM tab
header (`POM`) and Title_Line (pom.toml Menu_Title) already derive from the menu
name/title (CR-CH-042), so nothing observable changes.
