# Owner review of snapshot-fidelity-impl (b9fc0e8) vs the LIVE window

Owner pasted the Alias snapshot + a screenshot of the live window, 2026-11-03.
Big improvement (chrome + grid + cells + info bar + F-keys all present now), but
several discrepancies remain. Listed by severity.

## MUST-FIX before merge

### D2 (BUG, the reviewer's finding #2 -- data rendered TWICE)
The snapshot still emits the OLD fields block:
  `SS . . . : SNAPSHOT`   and a `Aliases` heading
ABOVE the grid, AND the same alias appears in the grid row
  `____ | [ffcmd v] | SNAPSHOT | SS | ... `.
So the alias data is shown TWICE (dot-leader fields block + grid). The LIVE
window has NO dot-leader line -- the alias data lives ONLY in the grid. This
violates the single-faithful-view principle. FIX: the Alias provider must stop
emitting model.fields (the `SS . . . :` dot-leader rows) and the "Aliases"
heading when it already emits the grid; the grid is the single representation.

### D5 (DATA FIDELITY -- Parameters value lost)
Live row Parameters = `(&1)`. Snapshot grid shows Parameters = `____` (empty).
The grid is not picking up the Parameters value present in the live row. The
snapshot must show the real cell value, not blank.

## SHOULD-FIX / CONFIRM (fidelity gaps vs live)

### D6 (status bar content differs)
Live status bar: `RUNNING | Started: 10:00 | Profile: default | Ln 1, Col 1 |
UTF-8 | Zoom: +8 | 1 lines`.
Snapshot status bar: `Running | Ln 1, Col 1 | UTF-8 | 1 lines`.
Snapshot is a reduced/different status line (missing Started / Profile / Zoom,
and casing differs). A faithful snapshot should mirror the live status bar text.

## DESIGN QUESTIONS (owner to decide -- these shape the model)

### D3 (tab bar: all open tabs vs active tag only)
Live tab bar shows TWO tabs: `[ALIASES] x   [CONFIG] x` (with close glyphs).
Snapshot shows only `[ALIASES]`. Should the snapshot reproduce the full tab bar
(all open tabs + close affordance), or just the active Context's `[XXX]` tag?
Current spec (29.2) says the active tag only.

### D4 (menu bar)
Live window has a top menu bar: `Settings  Help  SCRM`. Snapshot omits it.
Should a faithful snapshot include the menu bar line?

### D1 (title line vs frame caption -- minor)
Snapshot shows `Alias Editor` as the box-border caption AND the chrome lines
inside; live shows the title once (centered below the tab bar). Confirm there is
no unintended double-title; keep single chrome (one title).

## NOTE
D3/D4/(and arguably the menu bar + full tab bar + exact status content) are
WORKSPACE-MODEL decisions: "what frame does a faithful snapshot reproduce" -- a
first-class input to the workspace-architecture alignment. D2 and D5 are
straight bugs to fix now. D6 is a fidelity fix. Decide D3/D4/D1 before the
fix run so the implementation targets the agreed frame.
