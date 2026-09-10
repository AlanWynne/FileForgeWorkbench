# Analysis Record: menu-and-statusbar (W3.6)

- **Wave**: 3 (Shell, commands, menus, session)
- **Backing crate**: `ff-menu` (GUI-independent model -- no egui dep; rendering in
  the shell)
- **Spec files**: requirements.md (473 lines, 16 requirements -- numbered
  1-11, 13, 16-19; 12/14/15 renumbered away, reqs appended out of order), tasks.md
  (208 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy

Borderline; not a forced split.

- Requirement volume: 16 reqs, 473 lines. Above thresholds. (signal 1)
- Responsibilities: menu bar + status bar are two related shell surfaces; the core
  (Reqs 1-11, 13, 16) is cohesive. BUT Reqs 17-19 (tab chrome, detachable windows,
  ISPF split-screen) are OUT OF SCOPE for a menu/statusbar crate (see PA-SPLIT/
  PA-WATCH below) -- they belong to layout-and-docking / multi-tab-editor. (partial)
- Crates: single crate `ff-menu`; well-sized (no file over 350).
- Cohesion: high for the core; Reqs 17-19 are misfiled.

The real issue is SCOPE (Reqs 17-19), not a size-split. No file-size split needed.

---

## 2. Cross-unit consistency

### PA-WATCH-020 (NEW) -- Reqs 17-19 misfiled / belong to Wave-4 layout+tab units

Reqs 17 (Tab Window Chrome), 18 (Detachable Tab Windows), 19 (ISPF Navigation
Enhancements: SCROLL field, Fastpath, Split Screen, List Panel LOCATE) are in the
menu-and-statusbar SPEC, but `ff-menu` does NOT implement them (grep: 0
detach/split/floating refs). These overlap directly with:
- **layout-and-docking** (Wave 4): detachable/floating windows, split screen.
- **multi-tab-editor** (Wave 4): per-tab title/command-line chrome.
- **navigation-commands / menu-workspace**: Fastpath (= chained path, PA-INCOMPLETE-008),
  SCROLL field (viewport, W1.2/W1.7).

So Reqs 17-19 are CR-appended requirements that landed in the wrong spec; the
functionality is owned/implemented by Wave-4 units. Recorded PA-WATCH-020 (MEDIUM):
at Wave 4 (layout-and-docking + multi-tab-editor), confirm Reqs 17-19 are
implemented THERE and either move these requirements to the owning specs or mark
them as cross-references in menu-and-statusbar (avoid double-ownership). This is a
spec-hygiene issue, not a code duplication (ff-menu has no competing impl).

### Menu-Command integration -- CLEAN (Command_Target routing)

Req 2 (Menu-Command Integration): all menu items route through command-framework
dispatch -- no menu action directly mutates state (design principle). Consistent
with menu-workspace (W3.5) and command-configurator (W3.4): the Command_Target
model (command-framework Req 8) is the shared invocation contract across the menu
family. No duplication. Menu extensibility (Req 10) + status-bar segment
extensibility (Req 8) via ff-plugin -- consistent with plugin-architecture.

### GUI-independence -- CLEAN

`ff-menu` has NO egui/eframe dep: it is the menu/statusbar MODEL (structure,
segments, state); rendering is the shell's job. Correct layering (like ff-tabmask/
ff-completion). Good.

### Recent-files store -- minor raw-fs (PA-WATCH-019 adjacent)

`recent_files.rs` does 2 raw `std::fs` calls (read/write a JSON recent-files list)
with a proper `MenuError::RecentFilesIoError` error type. A small menu-local
persistence, distinct from the menus/ config store but the same raw-file family
(PA-WATCH-019). Low concern (has error handling). Folded into PA-WATCH-019.

### Public types and ownership

- Menu bar structure, menu items, context menus, status-bar segments/layout,
  recent-files list, primary command field, About dialog -- sole-owned by ff-menu.
  No duplication (Reqs 17-19 have NO ff-menu impl -- PA-WATCH-020).
- Deps: ff-command + ff-config + ff-core + ff-logging + ff-plugin. Correct
  (routes through command-framework, config for segment settings, plugin for
  extensibility). Clean layering.

### Cross-reference integrity

Cross-refs resolve. Reqs 17-19 imply cross-unit ownership (PA-WATCH-020).

---

## 3. Completeness

Tracking: all 208 sub-tasks `[x]`. The CORE menu/statusbar (Reqs 1-11, 13, 16) is
implemented in ff-menu. Reqs 17-19 (tab chrome / detach / ISPF nav) are tracked
`[x]` here but have NO ff-menu implementation -- suggesting either (a) they were
implemented in the Wave-4 layout/tab crates and the tasks here are cross-cutting
markers, or (b) a partial false-positive. Since ff-menu has 0 detach/split code,
this is PA-WATCH-020 territory (misfiled reqs) rather than a clear PA-INCOMPLETE --
confirm at Wave 4 whether the functionality exists in layout-and-docking/
multi-tab-editor. Recorded as a WATCH, not an INCOMPLETE, pending that confirmation.

### TCR gap (PA-TCR-016)

TCR.md has 2 rows for ff-menu against 16 reqs / ~130 criteria. Thin. Recorded
PA-TCR-016.

---

## 4. Logging audit

Scan of `crates/ff-menu/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 2 (recent_files.rs -- has RecentFilesIoError)

Menu actions dispatch through command-framework, so the CR-NR-058 per-command
instrumentation (PA-W0.2) covers menu-initiated command execution automatically.
The recent-files I/O errors are returned as MenuError (not logged). Zero-log is
largely defensible for a GUI-independent model. Recorded PA-LOG-023 (LOW): wire a
WARN on recent-files read/write failure (recent_files.rs) + drop-or-use the dead
ff-logging dep; dev-logging on menu open is optional.

---

## 5. Task revision proposals

- **PA-WATCH-020 (MEDIUM)**: Reqs 17-19 (Tab Window Chrome, Detachable Tab Windows,
  ISPF Navigation/Split Screen) are misfiled in menu-and-statusbar -- ff-menu has no
  impl. At Wave 4 (layout-and-docking + multi-tab-editor), confirm the functionality
  lives there and move/cross-reference these requirements to the owning specs to
  avoid double-ownership. Spec-hygiene.
- **PA-STD-036 (ASCII, runtime string)**: 17 non-ASCII bytes (1 non-comment) --
  em-dash in a runtime `#[error]` string (error.rs:21). Non-ASCII in user-facing
  output. Replace with `--`. REFACTOR, no gate.
- **PA-LOG-023 (LOW)**: WARN on recent-files I/O failure + resolve the dead
  ff-logging dep; optional menu-open dev-logging.
- **PA-TCR-016**: enumerate per-requirement TCR rows (2 rows for 16 reqs). No code.

No PA-STD size item (no file over cap). No requirement CHANGE proposed for the core;
Reqs 17-19 need OWNERSHIP relocation (PA-WATCH-020), not a content change.

---

## Summary

menu-and-statusbar (`ff-menu`) is a GUI-independent menu-bar + multi-segment
status-bar model (no egui dep -- correct layering), routing all menu actions through
command-framework (Command_Target, consistent with the menu family), with
plugin-extensible menus + status segments. Core (Reqs 1-11, 13, 16) is complete and
well-sized. The notable finding is PA-WATCH-020: Reqs 17-19 (tab chrome, detachable
windows, ISPF split-screen) are SPEC scope-creep -- they have no ff-menu
implementation and belong to the Wave-4 layout-and-docking / multi-tab-editor units;
confirm+relocate there. Minor items: runtime-string ASCII (PA-STD-036), dead
ff-logging dep + recent-files WARN (PA-LOG-023), thin TCR (PA-TCR-016). The
recent-files raw-fs store is a minor member of the PA-WATCH-019 raw-store family.
No split, no duplication conflict.
