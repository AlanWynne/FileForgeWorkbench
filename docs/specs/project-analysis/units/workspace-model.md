# Analysis Record: workspace-model (W3.10)

- **Wave**: 3 (Shell, commands, menus, session) -- last Wave-3 analysis unit
- **Backing code**: NO dedicated crate -- `WorkspaceState` model in
  `ff-session/src/workspace.rs` (238 lines, extends ff-session) + wired into the
  `ff-desktop` shell (`WorkbenchShell` coordinator).
- **Spec files**: requirements.md (214 lines, 6 requirements), tasks.md
  (26 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy

NOT a split candidate. Small, focused: 6 reqs, 214 lines; the model is
`workspace.rs` at 238 non-test lines (under cap). One cohesive concern (a named,
persistable grouping of root dirs + workspace-scoped settings + recent files +
workspace file). No split.

### Cross-cutting shell-coordinator size violation (PA-STD-042) -- SEVERE, recorded here

workspace-model is "wired into ff-desktop" -- and the ff-desktop `shell/` module
(the `WorkbenchShell` coordinator that hosts workspaces/tabs, into which
workspace-model + ff-session + menu-workspace all wire) contains the WORST 400-cap
violations in the entire codebase:

| File | non-test |
|------|----------|
| `shell/commands.rs` | 1255 (worst in the codebase) |
| `shell/update.rs` | 1011 |
| `shell/mod.rs` | 922 |
| `shell/render.rs` | 873 |
| `shell/render_chrome.rs` | 580 |
| (`shell/tests.rs` 4105 -- test module, excluded from cap) |

These belong to the ff-desktop SHELL COORDINATOR (cross-cutting -- command dispatch,
update loop, render, chrome), not to workspace-model specifically, but recorded here
as the last shell/session unit. rust-standards.md EXPLICITLY prescribes splitting
`shell/` into `state.rs` / `render.rs` / `commands.rs` / `dialogs.rs` (the
ff-desktop module layout), and these files blow past that (commands.rs 1255 is ~3x
the cap; the shell layout is already violated in-place). Recorded PA-STD-042
(REFACTOR, HIGH within the STD class): split the ff-desktop `shell/` coordinator per
the prescribed layout -- alongside the VCM PA-STD-025 (files_panel 1195) these are
the two largest shell refactors. Cross-cutting; not owned by one requirements unit.

---

## 2. Cross-unit consistency

### WorkspaceState vs SessionState (both in ff-session) -- distinct, adjacent

Both `WorkspaceState` (this unit) and `SessionState` (startup-and-session W3.9) live
in ff-session but model DIFFERENT things:
- `WorkspaceState` = a named, shareable WORKSPACE FILE: root directories (catalog
  mount points) + workspace-scoped settings + workspace recent-files (Req 1-6).
- `SessionState` = WHICH tabs/workspaces are open + geometry, for session restore
  across restarts (W3.9).

Req 5 (Workspace Session Persistence) links them (a workspace carries session
state). Distinct concerns, no duplication -- but they are adjacent and BOTH in
ff-session, reinforcing the PA-SPLIT-011 picture (ff-session bundles startup-lifecycle
+ session-state + workspace-model). Recorded: workspace-model is a THIRD concern in
ff-session, strengthening PA-SPLIT-011 (session-state + workspace-model could
co-extract as the "persistence" crate vs startup-lifecycle).

### Workspace root set unblocks Command Palette + Global Search

The spec states workspace-model is "the foundational layer that unblocks the Command
Palette (needs a workspace scope for search) and Global Search (needs a workspace
root set)". Command Palette (W3.3) reads the command registry (not obviously the
workspace root -- confirm), Global Search (Wave 5) needs the root set. Recorded
PA-WATCH-024 (LOW): confirm Command Palette + Global Search actually consume the
workspace root set from WorkspaceState (the declared unblock), at Wave 5
(global-search) + the palette. Cross-cutting dependency.

### Workspace file store -- raw-fs (PA-WATCH-019 family)

`workspace.rs` load_workspace/save_workspace do raw `std::fs` (6 calls) for the
workspace file. Another local raw-store in the PA-WATCH-019 family (menus/ +
commands.toml + criteria + history + recent-files + session + now workspace file).
Folded into PA-WATCH-019. Legitimate (user-config file, not a document).

### Public types and ownership

- `WorkspaceState`, `WorkspaceRecentFile`, load/save_workspace -- sole-owned by
  ff-session (workspace-model extension). No duplication with SessionState.
- Workspace lifecycle commands (Req 2) dispatch through command-framework.
  Workspace-scoped settings (Req 4) layer over ff-config. Consistent.

### Cross-reference integrity

Cross-refs resolve. No dangling refs.

---

## 3. Completeness

Tracking: all 26 sub-tasks `[x]`. Implementation present for all 6 reqs (workspace
file format, lifecycle commands, root management, scoped settings, session
persistence, scoped recent files) in workspace.rs + shell wiring. Tests in-file.
No PA-INCOMPLETE. Complete.

### TCR gap (PA-TCR-017)

TCR.md has ZERO rows for workspace-model (grep = 0) across 6 reqs. Total absence
(small surface). Recorded PA-TCR-017 (LOW).

---

## 4. Logging audit

Scan of `ff-session/src/workspace.rs`:

- `ff_logging` / `log_*!`: 0 (part of the ff-session DEAD ff-logging dep,
  PA-LOG-026)
- `println!` / `eprintln!`: 0
- `std::fs`: 6 (workspace file load/save)
- non-ASCII: 0 (clean)

Workspace file load/save failures return `SessionError` (not logged) -- folds into
the ff-session PA-LOG-026 (the fault-tolerance logging gap). No separate PA-LOG
raised; workspace load/save WARN should be part of the PA-LOG-026 ff-session logging
pass. Recorded as a note under PA-LOG-026.

---

## 5. Task revision proposals

- **PA-STD-042 (REFACTOR, HIGH within STD)**: split the ff-desktop `shell/`
  coordinator per the prescribed `state/render/commands/dialogs` layout --
  commands.rs 1255, update.rs 1011, mod.rs 922, render.rs 873, render_chrome.rs 580
  (worst cap violations in the codebase). Cross-cutting (ff-desktop shell), recorded
  at the last shell/session unit. Pairs with VCM PA-STD-025.
- **PA-TCR-017 (LOW)**: add TCR rows for the 6 workspace-model reqs (0 today). Small.
- **PA-WATCH-024 (LOW)**: confirm Command Palette + Global Search consume the
  workspace root set from WorkspaceState (the declared unblock) at Wave 5 + palette.
- **PA-SPLIT-011 reinforcement**: workspace-model is a THIRD concern in ff-session
  (with startup-lifecycle + session-state); note in the PA-SPLIT-011 proposal that
  session-state + workspace-model could co-extract as a "persistence" crate.
- **PA-LOG-026 note**: workspace file load/save WARN folds into the ff-session
  fault-tolerance logging pass.

No PA-STD for workspace.rs itself (238, under cap; 0 non-ASCII). No requirement
CHANGE proposed.

---

## Summary

workspace-model is a small, complete unit -- a named/persistable WorkspaceState
(root dirs + scoped settings + recent files + workspace file) implemented in
`ff-session/src/workspace.rs` (238 lines, under cap, clean ASCII), distinct from but
adjacent to SessionState (W3.9), reinforcing the PA-SPLIT-011 picture (ff-session now
bundles THREE concerns: startup-lifecycle + session-state + workspace-model). The
significant finding is cross-cutting, not workspace-model-specific: PA-STD-042 -- the
ff-desktop `shell/` coordinator holds the WORST 400-cap violations in the codebase
(commands.rs 1255, update.rs 1011, mod.rs 922, render.rs 873), recorded here as the
last shell unit and prescribed for the standard shell-module split. Minor: total TCR
absence (PA-TCR-017, small), a declared-unblock watch for Command Palette + Global
Search consuming the workspace root (PA-WATCH-024), the workspace-file raw-store
(PA-WATCH-019 family), and workspace load/save logging folded into ff-session
PA-LOG-026. No conflict, no split of workspace-model itself.
