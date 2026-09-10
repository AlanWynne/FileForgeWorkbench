# Analysis Record: view-zoom (W4.5)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-zoom`
- **Spec files**: requirements.md (204 lines, 9 requirements), tasks.md
  (145 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 9 reqs, 204 lines. One small cohesive concern (per-editor
integer point-offset zoom: model, shortcuts, wheel, range, per-instance, session
persist, status indicator, ZOOM command, DPI). 9 files, no file over 400. No split.

---

## 2. Cross-unit consistency -- CLEAN

- Zoom persistence (Req 6): `ff-zoom` exposes `ZoomSessionEntry` (a serialisable
  snapshot) that ff-session persists -- consistent producer(contract)/consumer
  (persistence) split, same pattern as ff-tabs (W4.2) + the session model (W3.9).
  Clean.
- ZOOM primary command (Req 8): dispatched via command-framework (Command_Target),
  consistent with the menu/command family. Deps: ff-command + ff-config + ff-logging.
- Status-bar zoom indicator (Req 7): ff-zoom provides the state; ff-menu status bar
  renders it (W3.6). Consistent (like the wrap indicator, W1.14).
- Display-only, per-editor-instance, NOT undoable (Req: does not modify content) --
  consistent with the other display-only per-instance features (line-wrap W1.14,
  view state). Zoom applies an integer point OFFSET to the theme base font (ff-theme
  W4.4 owns the base font). Clean layering.
- DPI/multi-monitor (Req 9): zoom is a logical point offset; DPI scaling is the
  renderer's -- clean separation.

### Public types and ownership

- `ZoomState` (per-editor mutable), `ZoomSessionEntry` (serialisable), ZOOM command,
  zoom range/clamping -- sole-owned by `ff-zoom`. No duplication. 0 fs (state model;
  persistence via ff-session contract).

### Cross-reference integrity

Cross-refs (theme base font, command-framework, configuration-system,
startup-and-session, menu-and-statusbar) resolve. No dangling refs.

---

## 3. Completeness

Tracking: all 145 sub-tasks `[x]`. Implementation present across all 9 reqs (offset
model, keyboard shortcuts, Ctrl+wheel, range limits, per-instance, session persist,
status indicator, ZOOM command, DPI). Tests in-file. No PA-INCOMPLETE. Complete.

### TCR gap (PA-TCR-021)

TCR.md has 5 rows for ff-zoom against 9 reqs / ~60 criteria. Modest. Recorded
PA-TCR-021 (LOW -- small surface).

---

## 4. Logging audit

Scan of `crates/ff-zoom/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 0

Zoom is a pure display-state model -- zero-log defensible (invalid ZOOM arg + range
clamping surface via status/error). The zoom range-clamping (Req 4) is a mild
status/dev-logging candidate. Recorded PA-LOG-031 (LOW): resolve the dead dep;
optional dev-logging on ZOOM command + clamp events. Low priority.

---

## 5. Task revision proposals

- **PA-STD-047 (ASCII, runtime string)**: 112 non-ASCII bytes (2 non-comment) --
  em-dash in a runtime `#[error]` string (error.rs:25; 68 is a test). Non-ASCII in
  user-facing output. Rest doc-comment dashes. Replace with `--`. REFACTOR, no gate.
- **PA-LOG-031 (LOW)**: resolve dead ff-logging dep; optional dev-logging on ZOOM/
  clamp.
- **PA-TCR-021 (LOW)**: extend TCR rows (5 for 9 reqs). No code.

No PA-STD size item (no file over 400). No requirement CHANGE proposed.

---

## Summary

view-zoom (`ff-zoom`) is a small, clean, complete display-only per-editor zoom model
(integer point offset to the theme base font; shortcuts/wheel/command/status/DPI).
Clean cross-unit story: session persistence via a `ZoomSessionEntry` contract
(ff-session consumes), ZOOM command via command-framework, status indicator via
ff-menu, base font from ff-theme -- all producer/consumer, no duplication, 0 fs.
Findings are the familiar mechanical trio: runtime-string ASCII (PA-STD-047), dead
ff-logging dep (PA-LOG-031, LOW), and modest TCR (PA-TCR-021, LOW). No conflict, no
split, no size violation.
