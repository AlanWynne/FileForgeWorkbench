# Analysis Record: command-configurator (W3.4)

- **Wave**: 3 (Shell, commands, menus, session)
- **Backing code**: NO dedicated crate -- implemented in `ff-desktop` as the
  `command_config/` module (mod.rs 152, edit.rs 339, render.rs 190, store.rs 279).
  A Custom Workspace form over `<User_Data_Dir>/commands/commands.toml`.
- **Spec files**: requirements.md (146 lines, 4 requirements), tasks.md
  (26 sub-tasks, **24 done / 2 OPEN**), design.md present
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy

N/A / NOT a split candidate. Small shell feature: 4 reqs, 146 lines. One cohesive
concern (define/store/edit/delete named commands bound to menus/shortcuts, run
internal or external targets). No file over cap (edit.rs 339 largest). No split.

---

## 2. Completeness -- PA-TRACK (bookkeeping only, NOT functional incomplete)

Tracking: 24/26 tasks done, 2 OPEN -- but BOTH are pure BOOKKEEPING:
- 6.2 "Update docs/quality/TCR.md -- set command-configurator rows to correct status"
- 6.3 "Update docs/specs/project-master/tasks.md -- mark command-configurator tasks complete"

The actual implementation + tests are DONE (17 TCR rows already exist for the
crate; the code covers all 4 reqs). These are stale tracking checkboxes -- the same
class as the Wave-0 config CQ tasks 30-31 (PA-TRACK-001) and logging Req 12
(PA-TRACK-002). NOT a functional PA-INCOMPLETE. Recorded PA-TRACK-003 (LOW): verify
the code/tests are complete and check off 6.2/6.3 + set the TCR rows to their real
status. No feature work.

All 4 reqs (store file format, configurator workspace, external execution modes,
validation/binding) are implemented.

---

## 3. Cross-unit consistency

### External process execution -- DELEGATED (clean, pre-declared)

Req 3 (External Command Execution Modes: Detached Started_Task / Captured_Run)
does NOT spawn processes in command_config -- grep found NO `Command::new`/`spawn`
here. The spec (lines 42-43) says it REUSES "the external execution engine" =
shell-command's Req 19 (External program execution + Output_Panel, W3.8). So
command-configurator delegates process spawning to shell-command. Clean,
pre-declared boundary (verify the delegation wiring at shell-command W3.8).

### Security model -- inherited from shell.mode (pre-declared)

Lines 27-29: "All external execution is gated by the existing `shell.mode`
configuration ... introduces NO separate external-execution security switch." So
the security posture for running arbitrary external programs is inherited from
shell-command, not re-invented here. Consistent single-owner security gate.
Confirm shell.mode actually gates the delegated execution at W3.8. Recorded
PA-WATCH-018 (verify shell.mode gate covers configurator-launched external commands).

### Command_Target + Command_Store -- consistent with command-framework Req 8

Each definition is a named Command_Target (command-framework Req 8). The store is
a data-driven TOML "in the same spirit as the menus/ directory" (menu-workspace
pattern, W3.5). Consistent with the Command_Target model. No duplication.

### Raw std::fs for commands.toml -- PA-WATCH-019 (menu-workspace-pattern family)

`store.rs` does raw `std::fs` (read_to_string/write/metadata/create_dir_all) for
`commands.toml` with its own hot-reload (mtime check, Req 1.7). This bypasses
ff-config's layered/hot-reload machinery -- BUT the spec frames it as the
"menus/ directory pattern" (a family of user-data TOML stores managed directly,
not through ff-config layers). This is the SAME raw-file-store pattern as ff-select
(PA-WATCH-015) and menu-workspace. Recorded PA-WATCH-019 (LOW): assess whether the
menu-workspace-pattern raw-TOML stores (commands.toml, menus/, criteria) should
share a common loader/hot-reload helper and/or route through ff-config, OR are
intentionally direct. Confirm the shared pattern at menu-workspace (W3.5). Not
an FFW-ARCH-001 file-I/O-through-VFS issue per se (these are local user-config
files, not workbench documents).

### Public types and ownership

- Command_Store (commands.toml loader), configurator workspace/form, edit/render,
  Command_Target definitions -- sole-owned by the ff-desktop command_config module.
  No duplication.
- Cross-refs to command-framework (Command_Target Req 8), shell-command (external
  exec), menu-workspace (menu binding), configuration-system (shell.mode) resolve.

---

## 4. Logging audit

Scan of `ff-desktop/src/command_config`:

- `ff_logging` / `log_*!`: 0
- `println!` / `eprintln!`: 0
- `std::fs`: 7 (commands.toml store I/O + hot-reload -- see PA-WATCH-019)
- non-ASCII: 0 (clean)

Req 1.4 (skip duplicate id), 1.6 (invalid TOML / missing field), 1.7 (hot-reload
on external modification) describe skip/reload behaviour where a WARN would aid
debugging (skipped dup, parse error) -- though the criteria do not explicitly
mandate "SHALL log". The external EXECUTION logging belongs to shell-command
(W3.8), not here. Recorded PA-LOG-021 (LOW): dev-logging on Command_Store parse
errors / skipped-duplicate / hot-reload events under the `dev-logging` gate. Low
priority (store errors are surfaced to the user; execution logging is shell-command's).

---

## 5. Task revision proposals

- **PA-TRACK-003 (LOW, bookkeeping)**: the 2 open tasks (6.2 TCR, 6.3 project-master)
  are stale tracking, not feature work. Verify code/tests complete; check off +
  set TCR rows to real status. No feature implementation.
- **PA-WATCH-018 (verify at W3.8)**: confirm the `shell.mode` gate actually covers
  external commands launched via the configurator (delegated to shell-command).
- **PA-WATCH-019 (LOW)**: assess whether the menu-workspace-pattern raw-TOML stores
  (commands.toml / menus/ / criteria) should share a loader/hot-reload helper or
  route through ff-config. Confirm the shared pattern at menu-workspace (W3.5).
- **PA-LOG-021 (LOW)**: dev-logging on Command_Store parse/skip/hot-reload events.

No PA-STD (no file over cap, 0 non-ASCII -- Req 1.8 explicitly mandates ASCII-only
store content, and the code honours it). No requirement CHANGE proposed.

---

## Summary

command-configurator is a small, essentially-COMPLETE shell feature (user-defined
named commands bound to menus/shortcuts, stored in commands.toml) implemented in
`ff-desktop/src/command_config/` -- clean on size, ASCII (Req 1.8 ASCII-only honoured),
with 17 TCR rows. The 2 open tasks are pure BOOKKEEPING (PA-TRACK-003: update TCR +
project-master), NOT unimplemented features -- same class as the Wave-0 tracking gaps.
Architecture is clean: external process execution is DELEGATED to shell-command
(Req 19, W3.8) with security inherited from `shell.mode` (no separate switch), and
Command_Targets follow command-framework Req 8. Two light watches: verify the
shell.mode gate covers configurator-launched external commands (PA-WATCH-018, W3.8)
and whether the menu-workspace-pattern raw-TOML stores (commands.toml/menus/criteria)
should share a loader (PA-WATCH-019, W3.5). Optional store-event dev-logging
(PA-LOG-021). No new conflict; no split.
