# Analysis Record: caret-and-selection

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-caret-selection`
- **Spec folder**: `docs/specs/caret-and-selection/`
- **Analysed**: Wave 1, task W1.3
- **Verdict**: COMPLETE (127/127 tasks `[x]`, TCR PASS). NOT a split candidate
  (borderline on size but cohesive + well-factored). Zero-log defensible (pure
  rendering-config model). PA-WATCH-006 partially resolved (see section 3).

---

## 1. Scope summary

`ff-caret-selection` is the VISUAL PRESENTATION layer for the caret and
selection. Its spec explicitly scopes OUT the logical selection model (owned by
edit-operations) and covers only rendering config + query. 14 requirements:

- Req 1 caret shape/style (Invisible/Line/Block, overstrike); Req 2 caret colour
  (primary/additional); Req 3 caret blink (period only; GUI owns timer);
  Req 4 caret-line highlight (Frame/Fill, layer, sub_line); Req 5 selection
  display (colours/layers, eol_filled); Req 6 selection element colours
  (primary/additional/secondary/inactive); Req 7 virtual space display;
  Req 8 rectangular selection display; Req 9 multi-caret display; Req 10 modified
  line marker rendering; Req 11 theme integration + config (GUI-independent);
  Req 12 caret keyboard focus integration; Req 13 mouse text selection in editor
  canvas (CR-NR-034); Req 14 selectable text in read-only panels (CR-NR-034).

412 req lines, 14 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 412 lines AND 14 reqs | YES (size) |
| 3+ distinct responsibilities | ONE responsibility: visual presentation of caret + selection. Req 1-12 are all rendering config for the same concern; Req 13-14 are mouse/selectable-text UI but still selection presentation | No |
| 2+ crates | one crate `ff-caret-selection` | No |
| low-cohesion clusters | high cohesion -- all facets serve caret/selection rendering | No |
| file-size pressure | largest non-test file caret_line.rs 162 -- ALL well under the 400 cap | No |

Only the size criterion. **NOT a split candidate.** Despite 14 reqs/412 lines,
the crate is a single cohesive rendering concern and is exceptionally
well-factored (15 focused modules, none near the cap). Splitting would fragment a
coherent presentation layer. No split proposal.

## 3. Consistency / conflict -- PA-WATCH-006 (partial resolution)

- Public types owned here (CaretStyle, CaretColour, CaretLine config, BlinkState,
  SelectionColours, SelectionDisplay config, RectangularSelection display,
  MultiCaret display, ModifiedMarker, VirtualSpace display) -- sole owner
  `ff-caret-selection`. All are RENDERING config/query types. Added to matrix.
- **PA-WATCH-006 (SelectionState/Selection ownership) -- partially resolved**:
  VERIFIED that `ff-caret-selection` does NOT define `SelectionRange`,
  `SelectionPosition`, or `Selection` (grep = 0). The spec scope boundary is
  explicit: the logical selection model is owned by edit-operations; this crate
  CONSUMES it for rendering. So the VISUAL layer does not duplicate the logical
  model -- good. Three distinct concepts are now identified, each with a clear owner:
  1. undo-redo `SelectionState` (W1.1) -- the undo/redo restore snapshot.
  2. edit-operations `Selection`/`SelectionRange`/`SelectionPosition` (W1.5) --
     the logical selection model (positions, ranges, multi-caret coordination).
  3. caret-and-selection (here) -- the visual presentation of (2).
  The remaining check is whether undo-redo's SelectionState (1) duplicates or
  correctly references edit-operations' Selection (2). CARRY PA-WATCH-006 to
  W1.5 (edit-operations) for final resolution.
- Consumer relationships (correct direction): edit-operations (logical model),
  theme-and-appearance (colour palette / element colours), viewport-and-scrolling
  (scroll-to-caret), configuration-system (hot-reload), display-line-mapping
  (sub_line info for Req 4.10), clipboard-operations (Req 13.7 write contract),
  whitespace-and-guides (Req 7.6 exclusion). All match the spec Cross-References.
  No conflict.
- Req 13/14 (CR-NR-034 mouse selection + selectable read-only panels) are
  ff-desktop UI behaviours specified here but implemented in the shell; the
  SelectionBack element colour (Req 13.4) is consistent with Req 5/6. No conflict.

## 4. Completeness

- Tasks: 127 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-caret-selection` row PASS (caret rendering, selection highlight, blink
  state).
- Well-factored: 15 modules, none over the 400 cap.
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` IS a declared dependency (wired, unused).
- Defensible: this is a pure GUI-independent rendering-config model (Req 11.4
  explicitly mandates GUI independence -- stores config, exposes query methods;
  GUI shells do the drawing). No failure paths, no requirement mandates a log.
  Same class as viewport-and-scrolling (PA-LOG-007) and encoding-and-characters
  (correctly zero-log).
- No `println!`/`eprintln!`. GUI-independence upheld.
- Logging verdict: **adequate (correctly zero-log for a pure rendering-config
  model)**.

## 6. Findings logged

- **PA-WATCH-006 UPDATED** (partial resolution): visual layer confirmed NOT to
  duplicate the logical selection model. Remaining resolution deferred to W1.5
  (edit-operations): confirm undo-redo SelectionState references, not duplicates,
  edit-operations Selection. (Register row updated.)
- No new defects. caret-and-selection is a clean, well-factored, complete unit.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-caret-selection contributes
  42 matches (doc-comment prose incl. the `anchor != caret` glyph in Req 9.5
  echoed in code comments, separators). Rolled into project-wide PA-LOG-001;
  check for a genuine `!=`-vs-U+2260 case during cleanup.
