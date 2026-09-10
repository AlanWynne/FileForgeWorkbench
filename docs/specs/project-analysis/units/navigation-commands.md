# Analysis Record: navigation-commands

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-navigation-commands`
- **Spec folder**: `docs/specs/navigation-commands/`
- **Analysed**: Wave 1, task W1.7 (CR-NR-057/058 re-baseline)
- **Verdict**: INCOMPLETE (Req 20 Scroll Amount Arguments, Phase DF, unimplemented
  -- 5 open tasks). Req 1-19 done + TCR-PASS. SPLIT CANDIDATE (20 reqs, 624 req
  lines). PA-WATCH-008 RESOLVED (no duplicate scroll-amount source of truth).
  Zero-log defensible.
- **CR-NR-057/058 impact**: NONE directly; Req 20 is CR-NR-054 (Phase DF), same
  family as command-framework Req 9.

---

## 1. Scope summary

`ff-navigation-commands` covers viewport/caret movement (non-content) + SORT + COLS
+ BOUNDS. 20 requirements: LOCATE; SORT (only undoable cmd here); UP/DOWN/LEFT/
RIGHT/TOP/BOTTOM navigation; COLS ruler; BOUNDS/BNDS; paragraph nav; word nav;
word-part (camelCase) nav; vertical caret + column affinity; doc start/end; SAVE/
CANCEL/END references; ... through Req 20 (Scroll Amount Arguments, CR-NR-054
Phase DF: M/MAX/n for UP/DOWN/LEFT/RIGHT).

624 req lines, 20 requirements, single backing crate.

## 2. Split candidacy -- CANDIDATE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 624 lines AND 20 reqs | YES (both) |
| 3+ distinct responsibilities | viewport navigation (UP/DOWN/etc); caret movement + affinity + word/word-part nav; SORT (document mutation); COLS/BOUNDS session-state display | YES |
| 2+ crates | one crate | No |
| low-cohesion clusters | SORT (undoable doc mutation) sits oddly beside pure viewport/caret navigation; COLS/BOUNDS are display/session-state | YES |
| file-size pressure | largest non-test word.rs 238 -- all under cap | No |

3+ criteria. **SPLIT CANDIDATE** (PA-SPLIT-006, proposal only). The clearest seam:
SORT (undoable document reorder) and COLS/BOUNDS (display/session-state) are only
loosely related to the caret/viewport navigation core. Proposal: keep
`navigation-commands` = LOCATE + UP/DOWN/LEFT/RIGHT/TOP/BOTTOM + paragraph/word/
word-part/vertical/doc-start-end nav + Req 20 scroll args; move SORT to a small
`sort-command` spec (or edit-operations) and COLS/BOUNDS to a `column-tools` spec
(or fold BOUNDS into edit-operations, which already owns bounds-aware shift).
Owner approval + own gate. Crate split lower value (files well-sized).

## 3. Consistency / conflict -- PA-WATCH-008 RESOLVED

- Public types owned here (LOCATE/SORT/COLS/BOUNDS command executors, paragraph/
  word/word-part nav, VerticalCaretNav, ScrollAmount [pending]) -- sole owner.
  Added to consistency-matrix.
- **PA-WATCH-008 (RESOLVED -- no conflict)**: scroll-amount concern splits cleanly.
  navigation-commands owns the scroll COMMANDS (UP/DOWN/LEFT/RIGHT + M/MAX/n arg
  parsing, Req 3 + Req 20); viewport-and-scrolling owns the viewport STATE +
  clamping + the ISPF scroll-amount field (CSR/PAGE/HALF, viewport Req 14).
  VERIFIED navigation-commands DELEGATES to `ff_viewport_scrolling::{CursorModel,
  ViewportModel}` -- calls `viewport.scroll_to_line()`, `max_top_line()`,
  `visible_count()` for actual state mutation/clamping (vertical_caret.rs). No
  duplicate source of truth; correct consumer relationship. Req 20 (command-arg
  M/MAX/n) and viewport Req 14 (field CSR/PAGE/HALF) are COMPLEMENTARY layers.
- BOUNDS (Req 5) is shared session-state: navigation-commands OWNS the BOUNDS/BNDS
  command + the active-Bounds query API (Req 5.15); line-commands (bounds-aware
  shift, W1.6) and CHANGE/FIND (find-and-replace) CONSUME it. Consistent -- single
  owner, multiple readers. No conflict.
- SORT (Req 2) produces an undoable transaction via undo-redo-transactions API --
  relates to PA-CONFLICT-002 (which transaction model). Note: SORT is the one
  undoable command here; verify it uses the same transaction type the editing
  layer uses (EditorTransaction, per PA-CONFLICT-002) at task-revision.
- Word/word-part nav uses document-model char classification (Req 7.1, 8.5) --
  consistent with encoding-and-characters ownership (W0.19). No conflict.

## 4. Completeness -- INCOMPLETE (Req 20 / Phase DF)

- Tasks: 184 `[x]`, 5 `[ ]`. The 5 open are Phase DF (Req 20 Scroll Amount
  Arguments, CR-NR-054): DF.1 `parse_scroll_amount` (M/MAX/n), DF.2 UP/DOWN read
  arg, DF.3 LEFT/RIGHT read arg, DF.4 field-clear signal (command Req 9.9), DF.5
  unit tests.
- Verified UNIMPLEMENTED: no `parse_scroll_amount`/`ScrollAmount` in src (grep 0).
  This is the navigation-commands half of the same Phase DF feature as
  command-framework Req 9 (PA-INCOMPLETE-001) -- they ship together.
- Req 1-19 complete: LOCATE, SORT, nav commands, COLS, BOUNDS, paragraph/word/
  word-part/vertical/doc-start-end nav -- task groups `[x]`, 5 TCR rows PASS.
- Completeness verdict: **INCOMPLETE** -- Req 20 is real outstanding code
  (part of the Phase DF bundle). Logged PA-INCOMPLETE-005.

## 5. Logging audit

- Zero log call sites; `ff-logging` declared (wired, unused). DEFENSIBLE: pure
  viewport/caret command logic delegating to viewport-and-scrolling; LOCATE
  out-of-range / label-not-found and BOUNDS invalid surface via status messages
  (Req 1.2/1.4/5.13) + Result, not logs. No requirement mandates a log record.
- CR-NR-058 relevance: navigation/scroll dispatch would be good `log_trace!`
  candidates under the `dev-logging` gate; optional, folds into PA-LOG-002 re-scope.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log)**.

## 6. Findings logged

- **PA-INCOMPLETE-005** (INCOMPLETE, Req 20 -- code gap, gate complete, CR-NR-054):
  Scroll Amount Arguments (M/MAX/n for UP/DOWN/LEFT/RIGHT) unimplemented; 5 Phase
  DF tasks open (DF.1-DF.5); no `parse_scroll_amount`/`ScrollAmount` in code. Ships
  with command-framework Req 9 (PA-INCOMPLETE-001) as the Phase DF bundle. Owner:
  schedule Phase DF (TDD).
- **PA-SPLIT-006** (SPLIT PROPOSAL): 20 reqs / 624 lines / 3+ responsibilities.
  SORT (undoable doc mutation) and COLS/BOUNDS (display/session-state) are the
  separable seams from the caret/viewport navigation core. SPEC split proposal;
  crate split lower value. Owner-gated.
- **PA-WATCH-008 RESOLVED**: scroll-amount has a single source of truth --
  navigation-commands owns the commands + arg parsing and DELEGATES viewport state
  to viewport-and-scrolling; no duplication. Req 20 (M/MAX/n) and viewport Req 14
  (CSR/PAGE/HALF) are complementary. Register row updated.
- **PA-CONFLICT-002 (relates)**: SORT (Req 2.11) is undoable via the transaction
  API; verify it uses `EditorTransaction` (per PA-CONFLICT-002) at task-revision.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-navigation-commands
  contributes matches (doc-comment arrows/em-dashes e.g. `getValue -> get|Value`,
  separators). Rolled into project-wide PA-LOG-001.
