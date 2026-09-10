# Analysis Record: menu-workspace (W3.5)

- **Wave**: 3 (Shell, commands, menus, session) -- CR-NR-057-flagged
- **Backing code**: NO dedicated crate -- `ff-desktop/src/menu_workspace/` module
  (loader.rs 311, defaults.rs 195, mod.rs 167, render.rs 102, commands.rs 93) +
  `primary_option_menu.rs` (537). Menu Workspaces load options from `menus/*.toml`.
- **Spec files**: requirements.md (428 lines, 8 requirements -- numbered 1-5, 9-11;
  6-8 renumbered away), tasks.md (90 sub-tasks, **75 done / 15 OPEN**), design.md
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy

NOT a split candidate. 8 reqs, 428 lines. One cohesive concern (a configurable
TOML-driven Menu Workspace pattern generalising POM + Settings Context). Module is
well-decomposed (loader/defaults/render/commands). No split.

### Source file size violation (PA-STD-035)

`primary_option_menu.rs` = 537 non-test lines, over the 400 cap (the POM instance
of the pattern). The menu_workspace/ module files are all under cap. Split the POM
by concern (options model / render / dispatch). REFACTOR, no gate. Recorded
PA-STD-035.

---

## 2. Completeness -- PA-INCOMPLETE-008 (CR-NR-057 chained navigation, 3rd leg)

75/90 tasks done, **15 OPEN**. They split into:

BOOKKEEPING (tracking gaps, not features): 1.3 (persistence tests), 8.1/8.2
(TCR + project-master), 21.5 (TCR for CR-NR-057 Req 5).

MENU PERSISTENCE (small feature): 1.2 persist a Menu_Workspace across sessions +
1.3 round-trip tests. Minor; tie into startup-and-session (W3.9).

CR-NR-057 CHAINED NAVIGATION (the substantive unbuilt work): DF.1-DF.5 + Task
21.1-21.4:
- Base fastpath IS built: `resolve_chained_path("=0.Themes")` returns a command
  String and passes tests (Req 5.1-5.4). CR-NR-045 layer done.
- UNBUILT (CR-NR-057 upgrade): make it SEPARATOR-AWARE -- Task 21.1 return an
  ordered `Vec<PathStep>` (each carrying its preceding separator STOP `.` / PUSH
  `;`), 21.2 set Navigation_Origin (POM for leading `=`, current Workspace
  otherwise), 21.3 drive each PathStep through option dispatch pushing the
  intermediate Context for a PUSH; DF.1 `MENU <name> <key>` chained form, DF.3
  SHARE one navigation-and-activation helper between the Chained_Path resolver and
  the MENU command form (Req 5.5), DF.4 option-command forwarding.

This is the MENU-WORKSPACE LEG of the CR-NR-057 command-chaining bundle, alongside:
- command-semantics Req 11 (split_chain/execute_chain, PA-INCOMPLETE-007, W3.1)
- command-framework Req 10 (Context Navigation Stack -- the STOP/PUSH behaviour,
  PA-INCOMPLETE-003, W0.7 = Phase DH)

All three are INTERDEPENDENT: the separator (`.`/`;`) drives the Context Navigation
Stack (Req 10), and the shared `split_chain`/navigation-activation helper
(PA-WATCH-017, tasks 28.6 + DF.3) must be reused across command-line chains and the
menu fastpath. HONEST `[ ]` tracking (not a false-positive). Recorded
PA-INCOMPLETE-008 (HIGH): implement the CR-NR-057 separator-aware Chained_Path
(Task 21 + DF.1-5), coordinated with PA-INCOMPLETE-007 (Req 11) + PA-INCOMPLETE-003
(Req 10 nav stack) as the unified Phase DH chaining bundle.

Reqs 1-4, 9-11 (menu format, rendering, option dispatch, defaults/hot-reload,
option limits, Command_Target, MENU command) are implemented.

---

## 3. Cross-unit consistency

### PA-WATCH-017 (shared split_chain / navigation helper) -- CONFIRMED as unbuilt-shared-seam

W3.1 raised PA-WATCH-017: the menu fastpath must reuse command-semantics'
`split_chain`, not roll its own. Confirmed: today `resolve_chained_path` (commands.rs)
is menu-workspace's OWN resolver (returns a String, base fastpath). The CR-NR-057
tasks EXPLICITLY require sharing one helper (Req 5.5 "share one navigation-and-
activation path"; Task DF.3 "share one navigation-and-activation helper between the
Chained_Path resolver and the [MENU] command form"; Task 28.6 in command-semantics
"the fastpath resolver calls the same split_chain helper"). So the shared-helper
wiring is a DESIGNED requirement, currently UNBUILT on both sides
(PA-INCOMPLETE-007 + -008). PA-WATCH-017 folds into the Phase DH bundle: implement
`split_chain` once (command-semantics) and have the menu fastpath consume it.
Recorded as resolved-into-PA-INCOMPLETE-008 (the watch is now an explicit task).

### PA-WATCH-019 (menu-workspace-pattern raw-TOML stores) -- CONFIRMED family owner

menu-workspace IS the origin of the "menus/ directory pattern": `loader.rs` does
raw `std::fs` (12 fs calls) to load `menus/*.toml` with hot-reload (Req 4). This is
the pattern that command-configurator's `commands.toml` (PA-WATCH-019, W3.4) and
ff-select's `.criteria.json` (PA-WATCH-015, W2.6) mirror. So there are THREE
raw-TOML user-data stores (menus/, commands.toml, criteria) each with their own
std::fs loader + hot-reload, bypassing ff-config layers. Recorded PA-WATCH-019
CONFIRMATION: menu-workspace is the reference pattern; assess a shared
loader/hot-reload helper across the three (LOW; these are local user-config files,
NOT workbench documents -- no FFW-ARCH-001 VFS issue). Unlike ff-select, menu-workspace
DOES use ff-logging (4 calls) -- so its store errors are logged.

### Command_Target + POM/Settings unification -- consistent

Menu options reference a Command_Target (Req 10, command-framework Req 8). POM +
Settings Context are now instances of this one pattern (backed by `menus/pom.toml`).
Consistent with the command-framework Command_Target model and command-configurator
(W3.4, same Command_Target). No duplication.

### Public types and ownership

- MenuFile/MenuOption loader, Menu_Workspace, `resolve_chained_path`, POM instance,
  MENU command -- sole-owned by the ff-desktop menu_workspace module. No duplication
  (the chained-path resolver will SHARE the command-semantics split_chain once built).
- Cross-refs (command-framework Command_Target, command-semantics split_chain,
  startup-and-session persistence, configuration-system) resolve.

---

## 4. Logging audit

Scan of `ff-desktop/src/menu_workspace`:

- `ff_logging` / `log_*!`: 4 (NOT a dead dep -- actually logs; contrast most
  W2-3 units). Menu-load/hot-reload errors are logged.
- `println!` / `eprintln!`: 0
- `std::fs`: 12 (menus/*.toml load + hot-reload -- PA-WATCH-019 family)
- non-ASCII: 0 (clean)

This is one of the FEW W2-3 shell units with real logging. The gap: CR-NR-058
dev-logging on menu navigation / option dispatch / chained-path resolution (once
built) would aid debugging the fastpath. Recorded PA-LOG-022 (LOW): add dev-logging
on option dispatch + (post-build) chained-path PathStep resolution under the
`dev-logging` gate; the existing menu-load logging is a good baseline.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-008 (HIGH)**: implement CR-NR-057 separator-aware Chained_Path
  navigation (Task 21.1-21.4 + DF.1-DF.5): `Vec<PathStep>` with STOP/PUSH
  separators, Navigation_Origin, PathStep dispatch pushing intermediate Contexts,
  MENU chained form, and the SHARED navigation-activation helper. Coordinate as ONE
  Phase DH bundle with PA-INCOMPLETE-007 (command-semantics Req 11 split_chain/
  execute_chain) and PA-INCOMPLETE-003 (command-framework Req 10 Context Navigation
  Stack). Also finish menu persistence (1.2/1.3, tie to W3.9).
- **PA-STD-035 (REFACTOR)**: split `primary_option_menu.rs` (537 non-test). No gate.
- **PA-WATCH-019 (CONFIRMED, LOW)**: menu-workspace is the reference "menus/ pattern";
  assess a shared raw-TOML store loader/hot-reload helper across menus/ + commands.toml
  + criteria (local config files, not VFS documents).
- **PA-LOG-022 (LOW)**: dev-logging on option dispatch + chained-path resolution
  (menu-load logging already present -- good baseline).
- **PA-TRACK-004 (bookkeeping)**: the tracking-only open tasks (1.3 tests, 8.1/8.2,
  21.5 TCR) -- verify + check off; distinct from the PA-INCOMPLETE-008 feature work.
- **PA-DOC (numbering)**: req list skips 6-8 (renumbered). Minor; note in the spec.

No requirement CHANGE proposed; Req 5 is a correctly-gated CR-NR-057 upgrade pending
implementation (honest `[ ]`).

---

## Summary

menu-workspace (`ff-desktop/src/menu_workspace/` + primary_option_menu.rs) is the
configurable TOML-driven Menu Workspace pattern that generalises POM + Settings
Context, and it is the REFERENCE for the "menus/ directory" raw-TOML store family
(PA-WATCH-019 -- shared by command-configurator commands.toml and ff-select criteria).
It is one of the few W2-3 shell units with REAL logging (4 calls, not a dead dep).
The headline is PA-INCOMPLETE-008 (HIGH): the CR-NR-057 separator-aware Chained_Path
navigation (Task 21 + DF.1-5) is UNBUILT -- the base `=0.Themes` fastpath works, but
the STOP/PUSH-separator-aware, Navigation_Origin, shared-helper upgrade does not.
This is the THIRD leg of the CR-NR-057 command-chaining bundle
(command-semantics Req 11 = PA-INCOMPLETE-007; command-framework Req 10 =
PA-INCOMPLETE-003), all interdependent and honestly tracked -- PA-WATCH-017's
shared `split_chain`/navigation helper is now an explicit task within it. Minor:
primary_option_menu.rs over the cap (PA-STD-035), option-dispatch dev-logging
(PA-LOG-022), bookkeeping tracking (PA-TRACK-004), and the Req 6-8 numbering gap.
