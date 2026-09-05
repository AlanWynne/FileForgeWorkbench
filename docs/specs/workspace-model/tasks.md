# Tasks -- Workspace Model

## Task 1. WorkspaceState data model and serialisation (Req 1, 5)

- [x] 1.1 Add `WorkspaceState` struct and `RecentFileEntry` to `ff-session/src/workspace.rs`
  - Satisfies: Req 1.1, 1.2
- [x] 1.2 Implement `load_workspace(path) -> Result<WorkspaceState>` -- parse `.ffwb-workspace` TOML,
  validate required fields, resolve relative root paths
  - Satisfies: Req 1.3, 1.4
- [x] 1.3 Implement `save_workspace(state, path) -> Result<()>` -- serialise to TOML
  - Satisfies: Req 1.1, 2.2
- [x] 1.4 Write unit tests: round-trip serialisation, missing required field error, relative path resolution
  - Satisfies: Req 1.2, 1.3, 1.4

## Task 2. Session persistence for active workspace (Req 5)

- [x] 2.1 Add `active_workspace_path: Option<String>` to `SessionState` in `ff-session`
  - Satisfies: Req 5.1
- [x] 2.2 Persist `active_workspace_path` in `session.toml` on exit
  - Satisfies: Req 5.1
- [x] 2.3 Restore workspace at startup: load workspace before restoring tabs; handle missing path gracefully
  - Satisfies: Req 5.2, 5.3, 5.4
- [x] 2.4 Write unit tests: session round-trip with workspace path, missing path fallback
  - Satisfies: Req 5.2, 5.3

## Task 3. Workspace lifecycle commands in ff-desktop (Req 2)

- [x] 3.1 Add `WorkspaceState` field to `WorkbenchShell`; implement `open_workspace()`,
  `save_workspace()`, `close_workspace()` helpers
  - Satisfies: Req 2.1, 2.2, 2.4
- [x] 3.2 Wire `WORKSPACE OPEN/SAVE/SAVE AS/CLOSE` command parsing in `shell/commands.rs`
  - Satisfies: Req 2.1, 2.2, 2.3, 2.4
- [x] 3.3 Add `File > Open Workspace...`, `File > Save Workspace`, `File > Save Workspace As...`,
  `File > Close Workspace` menu items
  - Satisfies: Req 2.1, 2.2, 2.3, 2.4
- [x] 3.4 Implement unsaved-changes prompt when switching workspaces
  - Satisfies: Req 2.5
- [x] 3.5 Write unit tests: open/close lifecycle, unsaved-changes guard
  - Satisfies: Req 2.1, 2.4, 2.5

## Task 4. Workspace root management (Req 3)

- [x] 4.1 On workspace load, register each root as a Native catalog via `CatalogRegistry::add_catalog()`
  - Satisfies: Req 3.4
- [x] 4.2 On workspace close, unregister workspace roots from `CatalogRegistry`
  - Satisfies: Req 3.4
- [x] 4.3 Wire `WORKSPACE ADD ROOT` and `WORKSPACE REMOVE ROOT` commands
  - Satisfies: Req 3.1, 3.2
- [x] 4.4 Display workspace roots as top-level nodes in File Explorer sidebar
  - Satisfies: Req 3.3
- [x] 4.5 Handle missing root path at load time: warn in status bar, continue loading remaining roots
  - Satisfies: Req 3.5
- [x] 4.6 Write unit tests: root registration, missing root warning
  - Satisfies: Req 3.4, 3.5

## Task 5. Workspace-scoped settings (Req 4)

- [x] 5.1 Inject workspace `[settings]` table into `ff-config` as the Workspace layer on load
  - Satisfies: Req 4.1
- [x] 5.2 Remove Workspace layer from `ff-config` on workspace close; trigger hot-reload callbacks
  - Satisfies: Req 4.3
- [x] 5.3 Write unit tests: workspace settings override user settings, removal restores user value
  - Satisfies: Req 4.1, 4.3

## Task 6. Workspace-scoped recent files (Req 6)

- [x] 6.1 Maintain workspace MRU list in `WorkspaceState`; add files on open while workspace active
  - Satisfies: Req 6.1
- [x] 6.2 Persist MRU list in Workspace_File `[[recent_files]]` array
  - Satisfies: Req 6.2
- [x] 6.3 Revert to global recent-files list on workspace close
  - Satisfies: Req 6.3
- [x] 6.4 Write unit tests: MRU list accumulation, persistence round-trip
  - Satisfies: Req 6.1, 6.2
