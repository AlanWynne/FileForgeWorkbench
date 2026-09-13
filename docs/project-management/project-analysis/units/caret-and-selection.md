# Analysis Record: caret-and-selection

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-caret-selection`
- **Spec folder**: `docs/specs/caret-and-selection/`
- **Analysed**: Wave 1, task W1.3 (CR-NR-057/058 re-baseline)
- **Verdict**: COMPLETE (127/127 tasks `[x]`, TCR PASS). NOT a split candidate
  (cohesive rendering layer, well-factored). Zero-log defensible (pure
  rendering-config model). PA-WATCH-006 partially resolved (visual layer does not
  duplicate the logical Selection model; final check at W1.5).
- **CR-NR-057/058 impact**: NONE (caret specs untouched). Re-verified.

---

## 1. Scope summary

`ff-caret-selection` is the VISUAL PRESENTATION layer for the caret and
selection. Its spec explicitly scopes OUT the logical selection model (owned by
edit-operations) and covers only rendering config + query. 14 requirements: caret
shape/style, caret colour, caret blink (period only; GUI owns timer), caret-line
highlight, selection display (colours/layers), selection element colours, virtual
space display, rectangular selection display, multi-caret display, modified line
marker rendering, theme integration (GUI-independent), caret keyboard focus, mouse
text selection in editor canvas (CR-NR-034), selectable text in read-only panels
(CR-NR-034).

412 req lines, 14 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 412 lines AND 14 reqs | YES (size) |
| 3+ distinct responsibilities | ONE responsibility: visual presentation of caret + selection (all 14 reqs are rendering config for the same concern) | No |
| 2+ crates | one crate | No |
| low-cohesion clusters | high cohesion | No |
| file-size pressure | largest non-test caret_line.rs 162 -- ALL well under the cap | No |

Only the size criterion. **NOT a split candidate** -- a single cohesive rendering
concern, exceptionally well-factored (15 focused modules, none near the cap).
Splitting would fragment a coherent presentation layer.

## 3. Consistency / conflict -- PA-WATCH-006 (partial resolution)

- Public types owned here (CaretStyle, CaretColour, caret-line config, BlinkState,
  SelectionColours, SelectionDisplay config, RectangularSelection display,
  MultiCaret display, ModifiedMarker, VirtualSpace display) -- all RENDERING
  config/query types. Sole owner. Added to consistency-matrix.
- **PA-WATCH-006 (partially resolved)**: VERIFIED `ff-caret-selection` does NOT
  define `SelectionRange`/`SelectionPosition`/`Selection` (grep 0) -- it CONSUMES
  them from edit-operations, per the spec scope boundary. So the VISUAL layer does
  not duplicate the logical model. Three distinct concepts confirmed:
  (1) undo-redo `SelectionState` (undo/redo restore snapshot),
  (2) edit-operations `Selection`/`SelectionRange`/`SelectionPosition` (logical
      model),
  (3) caret-and-selection visual rendering of (2).
  Remaining: confirm undo-redo SelectionState references (not duplicates)
  edit-operations Selection at W1.5.
- Consumer relationships (correct direction): edit-operations (logical model),
  theme-and-appearance (element colours), viewport-and-scrolling (scroll-to-caret),
  configuration-system (hot-reload), display-line-mapping (sub_line for Req 4.10),
  clipboard-operations (Req 13.7), whitespace-and-guides (Req 7.6). No conflict.
- Req 13/14 (CR-NR-034 mouse selection + selectable read-only panels) are
  ff-desktop UI behaviours specified here; SelectionBack element colour (Req 13.4)
  consistent with Req 5/6. No conflict.

## 4. Completeness

- Tasks: 127 `[x]`, 0 `[ ]`. TCR PASS (caret rendering, selection highlight, blink
  state). Well-factored (15 modules, none over the cap).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` declared (wired, unused). DEFENSIBLE: pure
  GUI-independent rendering-config model (Req 11.4 mandates GUI independence --
  stores config, exposes query methods; GUI shells do the drawing). No failure
  paths, no requirement mandates a log. Same class as viewport / encoding.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (correctly zero-log for a pure rendering-config
  model)**.

## 6. Findings logged

- **PA-WATCH-006 UPDATED** (partial resolution): visual layer confirmed NOT to
  duplicate the logical selection model. Remaining resolution deferred to W1.5
  (edit-operations): confirm undo-redo SelectionState references, not duplicates,
  edit-operations Selection.
- No new defects. caret-and-selection is a clean, well-factored, complete unit.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-caret-selection contributes
  matches (doc-comment prose, separators). Rolled into project-wide PA-LOG-001.
