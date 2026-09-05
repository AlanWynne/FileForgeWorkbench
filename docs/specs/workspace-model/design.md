# Design -- Workspace Model

## Architectural Decisions

### 1. No new crate -- extend ff-session

The workspace model extends `ff-session` rather than introducing a new crate. The session
manager already owns `session.toml` persistence and the catalog registry save/load path.
Adding `WorkspaceState` alongside `SessionState` keeps the dependency graph flat.

### 2. Workspace_File format

A `.ffwb-workspace` TOML file with three top-level keys:

```toml
name = "MyProject"
roots = ["/home/user/projects/myapp", "/home/user/shared-libs"]

[settings]
editor.tab_size = 4
theme.active = "dark"

[[recent_files]]
path = "/home/user/projects/myapp/src/main.rs"
opened_at = "2026-09-05T10:00:00Z"
```

### 3. Configuration layer wiring

`WorkspaceSettings` is injected into `ff-config` as the Workspace layer (highest priority)
when a workspace is loaded, and removed when the workspace is closed. This reuses the
existing `ConfigHandle` layered merge -- no new config machinery needed.

### 4. Catalog mount point integration

Each `roots` entry is registered as a Native catalog via `CatalogRegistry::add_catalog()`
on workspace load and removed via `remove_catalog()` on workspace close. The catalog name
is derived from the last path component of the root (e.g., `/home/user/myapp` -> `myapp`).

### 5. Session persistence

`session.toml` gains one optional field: `active_workspace_path`. The startup block in
`shell/update.rs` loads the workspace before restoring tabs, so workspace roots are
available when tab file paths are resolved.

## Data Structures

```
WorkspaceState {
    name: String,
    file_path: Option<PathBuf>,   // None = unsaved new workspace
    roots: Vec<PathBuf>,
    settings: HashMap<String, ConfigValue>,
    recent_files: VecDeque<RecentFileEntry>,
    is_modified: bool,
}

RecentFileEntry {
    path: PathBuf,
    opened_at: DateTime<Utc>,
}
```

## Module Layout (ff-session)

```
ff-session/src/
  lib.rs          -- re-exports
  session.rs      -- SessionState, save/load (existing)
  workspace.rs    -- WorkspaceState, load_workspace(), save_workspace()  [NEW]
```
