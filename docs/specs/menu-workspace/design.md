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
