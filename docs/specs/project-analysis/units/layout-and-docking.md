# Analysis Record: layout-and-docking (W4.1)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-layout` (GUI-independent layout MODEL -- no egui dep; the
  shell renders it, Architecture Brief Principle 1)
- **Spec files**: requirements.md (266 lines, 11 requirements), tasks.md
  (169 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 11 reqs, 266 lines -- below thresholds. One cohesive concern
(dockable panels + tab groups/splits + detached workspaces + multi-monitor +
personas + serialization + DnD). 31 files. No split.

### Source file size violation (PA-STD-043) -- SEVERE

`engine.rs` = 812 non-test lines, ~2x the 400 cap (the LayoutEngine coordinating
panels/tab-groups/floating-windows/personas/serialization). One of the larger
single-file violations. Split by concern (panel mgmt / tab-group + split /
floating-window mgmt / persona + serialization). REFACTOR, no gate. Recorded
PA-STD-043. (Only file over cap; the other 30 are decomposed.)

---

## 2. Cross-unit consistency

### PA-WATCH-020 RESOLVED -- ff-layout OWNS tab chrome / detach / split

W3.6 flagged menu-and-statusbar Reqs 17-19 (Tab Window Chrome, Detachable Tab
Windows, ISPF Split Screen) as SPEC scope-creep with no ff-menu impl. Confirmed
here: ff-layout is the OWNER (607 detach/split/float refs):
- Req 2 Tab Groups (split views), Req 3 Detached Workspaces (matches the VCM
  canonical "Detached Workspace" terminology, W2.2), Req 11 Tab Window Chrome in
  Detached Workspaces -- these ARE the menu-statusbar Reqs 17-19 functionality,
  implemented in ff-layout (`floating/manager.rs`, `FloatingWindowManager`,
  `FloatingWindowId`).
So PA-WATCH-020 RESOLVES: the menu-and-statusbar Reqs 17-19 are MISFILED spec text;
ff-layout is the real owner. Recommendation: relocate/cross-reference those reqs
from menu-and-statusbar to layout-and-docking (spec-hygiene; no code move -- the
code is already here). Neither ff-menu nor ff-session deps on ff-layout (0 each) --
the shell (ff-desktop) wires them; no duplicate detach/split implementation exists.

### PA-WATCH-023 partial -- session Req 14 tab-container is ff-layout's

session Req 14 (ISPF POM + Tabbed Window Container): the TAB-CONTAINER functionality
is ff-layout's (tab groups); ff-session owns only the PERSISTENCE of which
tabs/workspaces are open (WorkspaceDescriptor, W3.9/W3.10). Consistent split
(container vs persistence). PA-WATCH-023 narrows: the tab-container half is
ff-layout; confirm the persistence/functionality boundary is documented.

### Layout serialization -- raw-fs (PA-WATCH-019 adjacent)

10 fs calls for layout serialization (Req 6 -- persist named layouts/personas).
ff-layout OWNS layout persistence, so this is legitimate. Another local-store family
member (PA-WATCH-019 adjacent), but layout files are the layout owner's data.

### Detached Workspace terminology -- consistent with VCM

Req 3/11 use "Detached Workspace" -- the canonical term from the VCM/three-level UI
model (W2.2: a Workspace moved to a separate OS window). Consistent terminology
across ff-layout + VCM. Good.

### Public types and ownership

- Panel/PanelId, TabGroup, FloatingWindowManager/FloatingWindowId (detached),
  Persona (layout presets), LayoutSnapshot/serialization, DnD, LayoutEngine --
  sole-owned by `ff-layout`. No duplication.
- GUI-independent (no egui dep) -- shell renders. Correct layering (like ff-menu/
  ff-completion/ff-tabmask). Deps: ff-logging only (injection-trait for shell
  integration presumably). Clean.
- Consumed by ff-shell (Output_Panel via ff-layout, W3.8), the whole shell.

### Cross-reference integrity

Cross-refs resolve. No dangling refs. Reqs 17-19 of menu-and-statusbar +
Req 14 of session point HERE (PA-WATCH-020/023 relocation).

---

## 3. Completeness

Tracking: all 169 sub-tasks `[x]`. Implementation present across all 11 reqs
(panels, tab groups, detached workspaces, multi-monitor, personas, serialization,
DnD, resizing, shortcuts, visual feedback, detached-workspace chrome). Tests
in-file. No PA-INCOMPLETE. Complete.

### TCR gap (PA-TCR-018)

TCR.md has 1 row for ff-layout against 11 reqs / ~90 criteria (large 31-file crate).
Thin. Recorded PA-TCR-018.

---

## 4. Logging audit

Scan of `crates/ff-layout/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 10 (layout serialization -- legitimate)

Layout serialization failures (Req 6) return LayoutError (error.rs has
serialization + register error variants), not logged. Layout is not a
fault-tolerance subsystem (a bad persona falls back to default), so zero-log is
largely defensible. Recorded PA-LOG-027 (LOW): wire a WARN on layout-serialization
load/save failure + dev-logging on persona switch / detach / dock events; resolve
the dead dep. Low priority.

---

## 5. Task revision proposals

- **PA-STD-043 (REFACTOR, SEVERE)**: split `engine.rs` (812 non-test, ~2x cap) by
  concern (panel / tab-group+split / floating-window / persona+serialization). No
  gate. Among the larger single-file violations (after shell PA-STD-042 files +
  VCM files_panel).
- **PA-WATCH-020 RESOLVED (relocate spec)**: move/cross-reference menu-and-statusbar
  Reqs 17-19 (tab chrome/detach/split) to layout-and-docking (the owner). Spec-
  hygiene; no code move. Do at the Wave-4 consolidation or W3-doc pass.
- **PA-WATCH-023 (narrowed)**: session Req 14 tab-CONTAINER = ff-layout; ff-session
  owns only persistence. Document the boundary.
- **PA-STD-044 (ASCII, runtime strings)**: 61 non-ASCII bytes (2 non-comment) --
  em-dashes in runtime `#[error]` strings (error.rs 40/95). NB the SPEC .md also has
  a mojibake `Â§3` (BOM artifact) in the intro -- fold into the doc ASCII sweep.
  Replace with `--`. REFACTOR, no gate.
- **PA-LOG-027 (LOW)**: layout-serialization WARN + dev-logging on persona/detach/
  dock; resolve dead dep.
- **PA-TCR-018**: enumerate per-requirement TCR rows (1 row for 11 reqs). No code.
- **PA-DOC**: the spec intro mojibake `Â§3` (should be `Section 3`).

No requirement CHANGE proposed for ff-layout; the menu-statusbar Reqs 17-19 need
RELOCATION to this spec (PA-WATCH-020), which is a menu-statusbar spec edit.

---

## Summary

layout-and-docking (`ff-layout`) is the GUI-independent layout model (dockable
panels, tab groups/splits, Detached Workspaces, multi-monitor, personas,
serialization, DnD) -- correct layering (no egui dep). It RESOLVES PA-WATCH-020: it
is the true OWNER of the tab-chrome/detach/split functionality that
menu-and-statusbar Reqs 17-19 misfiled (relocate those reqs here -- spec-hygiene, no
code move), and it narrows PA-WATCH-023 (session Req 14 tab-container = ff-layout;
ff-session owns only persistence). Uses the canonical "Detached Workspace"
terminology (consistent with VCM). Complete tracking (169/169). Findings: the SEVERE
`engine.rs` 812-line cap violation (PA-STD-043, ~2x), runtime-string + spec-mojibake
ASCII (PA-STD-044), a dead ff-logging dep on a non-fault-tolerance subsystem
(PA-LOG-027, LOW), and thin TCR (PA-TCR-018). No conflict, no split.
