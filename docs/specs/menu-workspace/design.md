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

`PersistedTabKind` gains a new variant:

```rust
MenuWorkspace { file_path: String },
```

On session restore, the shell creates a `MenuWorkspaceState` from the persisted
`file_path` and loads the menu file. If the file is absent, the tab is restored
with the load-error state (Requirement 1.5).

---

## 11. Tab Kind and Routing

New `TabKind` variant:

```rust
TabKind::MenuWorkspace(MenuWorkspaceState),
```

New shell commands (to be implemented in Phase CX or later):

- `MENU <name>` -- opens a Menu_Workspace backed by `menus/<name>.toml`.
- `MENU` (no argument) -- opens the POM menu (`menus/pom.toml`).

These commands are NOT implemented in Phase CU (spec only).

---

## 12. No Design Changes Required for Phase CU

Phase CU is a specification-only phase. No source files are modified. The design
above describes the target architecture that will be implemented when a separate
implementation instruction is given.
