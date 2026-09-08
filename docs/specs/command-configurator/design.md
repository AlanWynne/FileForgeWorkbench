# Design Document -- Command Configurator

## 1. Overview

The Command Configurator adds a user-authored command layer on top of the
unified Command_Target (command-framework Requirement 8). It has three parts:

1. A **Command_Store** -- a hot-reloadable TOML file of named Command_Definitions.
2. A **Command Configurator Context** -- a Custom Workspace that edits the store.
3. An **External execution path** with two modes (Detached, Captured) that
   reuses the `ff-shell` engine and Output_Panel.

No new library crate is required. The store loader and the Context live in the
`ff-desktop` binary layer alongside `menu_workspace/`; external execution reuses
`ff-shell`. The Command_Target type itself is owned by `ff-command`
(command-framework), so this sub-project stores and dispatches targets but does
not redefine them.

---

## 2. Architectural Position

```
ff-desktop/src/
  command_config/
    mod.rs        -- CommandDefinition, CommandStore, public API, re-exports
    store.rs      -- commands.toml load/save, validation, hot-reload poll
    render.rs     -- egui rendering of the Command Configurator Context
    edit.rs       -- add/edit form state and the variant-specific target editor
```

- `CommandTarget` (from `ff-command`) is serialised inside each definition.
- External execution calls into `ff-shell` (Requirement 19 of shell-command),
  which already owns process spawning, async capture, timeout, and the
  Output_Panel.
- Hot-reload reuses the `MenuWorkspaceState::poll_reload` pattern (mtime check
  per frame), not a new file watcher.

---

## 3. Data Model

```rust
/// One user-authored command.
/// Validates: command-configurator Requirement 1.2, 1.3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_category")]
    pub category: String,
    pub target: ff_command::CommandTarget,   // [command.target]
}

fn default_category() -> String { "user".to_string() }

/// The loaded store plus load diagnostics.
/// Validates: command-configurator Requirement 1.1, 1.6, 1.7
pub struct CommandStore {
    pub file_path: PathBuf,
    pub definitions: Vec<CommandDefinition>,   // valid entries only
    pub load_error: Option<String>,
    pub last_modified: Option<SystemTime>,
}
```

TOML shape:

```toml
[[command]]
id = "build.release"
label = "Build Release"
description = "Build the workspace in Release mode"
category = "build"
[command.target]
kind = "external"
program = "pwsh"
args = ["-File", "scripts/build.ps1", "-Config", "Release"]
working_dir = "${workspace_root}"
mode = "captured"

[[command]]
id = "open.notes"
label = "Open Notes Menu"
[command.target]
kind = "menu"
name = "notes"
```

Store save writes the full definition list back (Requirement 2.4/2.5),
preserving order. Load skips invalid entries and records `load_error`
(Requirement 1.6); duplicate ids keep the first (Requirement 1.4).

---

## 4. Target Resolution Integration

`command_config::store` exposes the loaded definitions to
`ff_command::resolve_target` via a `UserCommandStore` view (command-framework
design, Target resolution step 1). When the user types or a menu option carries
a bare id equal to a definition's `id`, resolution returns that definition's
stored `CommandTarget`. Reserved built-in ids cannot be shadowed
(Requirement 4.6); the loader checks each definition id against the
`CommandRegistry` reserved set and rejects conflicts.

---

## 5. External Execution (Requirement 3)

External targets are dispatched by `execute_target` (command-framework) to a
thin adapter that calls `ff-shell`:

- **Detached** -> `ff_shell::spawn_detached(program, args, working_dir)`:
  spawns the child, drops the handle, returns immediately. No capture, no tab.
  A `TaskHandle` is defined by `ff-shell` and returned, but the desktop layer
  drops it -- this is the named seam for possible future Started_Task
  monitoring, which is explicitly out of scope now.
- **Captured** -> the existing `shell.execute` async path (shell-command
  Requirement 4, 13, 15): run async, stream stdout/stderr to the Output_Panel
  with a header and exit code. The only difference from a typed `SHELL` string
  is that program+args are explicit rather than parsed from a shell line.

Placeholder expansion (`${workspace_root}`, `${file_dir}`) happens in the
adapter before spawn (Requirement 3.6). The `shell.mode` gate and the
prompt-confirm dialog are applied by the shell engine, unchanged
(Requirement 3.7, 3.8; shell-command Requirement 2).

### Why reuse ff-shell rather than a new engine

`ff-shell` already implements platform shell detection, async execution,
timeout, cancellation, ConPTY/PTY, and the Output_Panel. Captured mode is a
strict subset of `shell.execute`. Detached mode adds only a spawn-and-drop
variant. This avoids a parallel process-management implementation and a second
security switch.

---

## 6. Command Configurator Context (Requirement 2)

A new `WorkspaceKind::CommandConfigurator` (session-layer) / runtime
`TabKind::CommandConfigurator` renders the store as a table: id, label, variant,
and (for external) mode. Add/Edit opens `edit.rs` form state with a
variant-selector; the form shows only the fields for the chosen variant
(Requirement 2.6). Save validates then writes the store. The Context title is
`[COMMANDS]`; F3/END returns to the POM (Requirement 2.7, 2.8), matching other
Custom Workspaces.

Opening command: `COMMANDS` (primary command), and the Context is itself a
`CustomWorkspace { workspace_kind: CommandConfigurator }` target, so it is
persisted by descriptor (startup-and-session Requirement 21) like any other
visible Workspace.

---

## 7. Session Persistence

The Command Configurator Context persists as
`CustomWorkspace { workspace_kind: CommandConfigurator }` (no params needed).
Started_Tasks (Detached runs) and Captured_Run Output_Panel results are NOT
persisted (command-framework Requirement 8.9; startup-and-session
Requirement 21). The `commands.toml` store is user data on disk and is loaded at
startup independent of session restore.

---

## 8. Security

External execution inherits shell-command Requirement 2 entirely:
`shell.mode = disabled` refuses all external runs; `prompt` confirms first;
`enabled` runs without prompting. There is no Command-Configurator-specific
security setting. When an External target is invoked from a macro, the combined
macro-security + shell.mode rule (shell-command Requirement 2.7) applies.
