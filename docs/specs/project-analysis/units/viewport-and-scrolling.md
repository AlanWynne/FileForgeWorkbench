# Analysis Record: viewport-and-scrolling

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-viewport-scrolling`
- **Spec folder**: `docs/specs/viewport-and-scrolling/`
- **Analysed**: Wave 1, task W1.2
- **Verdict**: COMPLETE (129/129 tasks `[x]`, TCR PASS). BORDERLINE SPLIT
  (14 reqs). Zero-log defensible (pure GUI-independent model). One 400-cap
  refactor (viewport.rs 572), one crate-name-drift doc note, one optional
  logging enhancement.

---

## 1. Scope summary

`ff-viewport-scrolling` is the GUI-independent viewport model (owned by the
editor session, not the GUI). 14 requirements:

- Req 1 viewport state (top_line/visible_count/horizontal_offset/cursor/
  column_affinity); Req 2 vertical scroll commands (clamped); Req 3 cursor
  movement + viewport coordination; Req 4 full-file-range vertical scrollbar
  (proportional thumb); Req 5 caret visibility policies (slop/strict/jumps/even);
  Req 6 column affinity; Req 7 horizontal scrollbar; Req 8 mouse-wheel scrolling;
  Req 9 smooth scrolling (Line/Smooth, pixel_offset); Req 10 command-framework
  integration; Req 11 display-line-mapping integration; Req 12 viewport
  persistence/restore; Req 13 large-file scrollbar precision (64-bit);
  Req 14 editor scroll-amount integration (ISPF CSR/PAGE/HALF, CR-NR-035).

317 req lines, 14 requirements, single backing crate.

## 2. Split candidacy -- BORDERLINE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 317 lines (near); 14 reqs (>12) | YES (reqs) |
| 3+ distinct responsibilities | viewport-state/scroll-commands; scrollbar mapping (vert+horiz+precision); caret policies + column affinity; smooth scrolling; display-mapping integration | Weak-Yes |
| 2+ crates | one crate `ff-viewport-scrolling` | No |
| low-cohesion clusters | scrollbar mapping (Req 4,7,13) is the most separable; the rest orbit the viewport state | Weak |
| file-size pressure | `viewport.rs` 572 non-test over cap | Yes |

~2 criteria (14 reqs; viewport.rs size; weak responsibility spread).
**BORDERLINE.** The concerns are cohesive around one viewport model; a SPEC or
crate split is NOT recommended -- the right fix is the file-size refactor
(PA-STD-006), not a split. Recorded as a WEAK note, not a split proposal.

## 3. Consistency / conflict

- Public types owned here (Viewport, ViewportError, CaretPolicy, ScrollMode,
  Cursor model, scroll commands `ScrollLineUp/Down/PageUp/PageDown/ToLine/
  ToTop/ToBottom/Horizontal`, viewport snapshot) -- sole owner
  `ff-viewport-scrolling`. Added to consistency-matrix.
- **Crate-name drift (PA-DOC-004)**: spec Introduction says the crate is
  `ff-viewport-and-scrolling`; the actual crate is `ff-viewport-scrolling`. Same
  class as PA-DOC-001/003. Reconcile the spec to the real name.
- **DisplayLineMapper (Req 11)**: viewport takes a trait object from
  display-line-mapping (analysed next, W1.4). `total_display_lines`,
  fold/exclusion skipping, and scrollbar mapping all delegate to it. WATCH that
  the DisplayLineMapper trait is owned by display-line-mapping and only consumed
  here (no duplicate definition). Record PA-WATCH-007 (resolve at W1.4).
- **Scroll commands (Req 10) vs command-framework**: scroll commands are
  non-undoable (Req 10.6) -- consistent with command-framework Req 4 and
  undo-redo Req 10 (non-undoable operations include scroll position). No conflict.
- **Req 14 scroll-amount (CSR/PAGE/HALF, CR-NR-035)** overlaps navigation-commands
  Req 20 (scroll-amount arguments, part of Phase DF / PA-INCOMPLETE-001). WATCH:
  verify the scroll-amount source of truth is not duplicated between
  viewport-and-scrolling Req 14 and navigation-commands Req 20 when nav-commands
  is analysed (W1.7). Record PA-WATCH-008.
- Cursor/caret state here vs caret-and-selection (W1.3): viewport owns
  cursor_line/cursor_column for VIEWPORT coordination; caret-and-selection owns
  the editing caret/selection model. Verify the boundary at W1.3 (relates to
  PA-WATCH-006).

## 4. Completeness

- Tasks: 129 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-viewport-scrolling` row PASS (viewport model, cursor model, caret
  policy, scroll clamping); backed by tests/integration_tests.rs +
  property_tests.rs (viewport.rs has no inline test module -- tests are external).
- No std::fs/tokio::fs (pure model). No unwrap/expect in viewport.rs.
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` IS a declared dependency (wired, unused).
- Defensible: this is a pure GUI-independent computational model (viewport
  arithmetic, scroll clamping). Errors surface via `ViewportError` (Result) to
  the caller. No requirement mandates a log record here (contrast ff-undo-redo
  Req 1.6 / ff-plugin, which have explicit logging criteria).
- Optional enhancement (PA-LOG-007): two `ViewportError` variants describe
  recovery actions in their messages -- `InvalidConfig` ("using default") and
  `SnapshotOutOfBounds` ("clamping applied"). Mirroring ff-undo-redo's config
  clamp WARN, a `log_warn!` at those recovery points would aid diagnosis. Optional,
  owner-gated; not a completeness blocker.
- No `println!`/`eprintln!`. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log for a pure model)**.

## 6. Findings logged

- **PA-STD-006** (REFACTOR -- 400-line cap): `viewport.rs` is 572 lines, all
  non-test (inline tests absent; tests in tests/), over the cap. Split by concern
  (e.g. `viewport_scroll.rs` scroll commands, `viewport_scrollbar.rs` scrollbar
  mapping/precision, keep `viewport.rs` state + coordination). REFACTOR, no gate.
  This is the right fix for the size pressure rather than a spec/crate split.
- **PA-DOC-004** (TRACKING-FIX): spec Introduction says `ff-viewport-and-scrolling`
  but the crate is `ff-viewport-scrolling`. Reconcile the spec. Doc-only, no gate.
- **PA-LOG-007** (LOGGING ENHANCEMENT, optional): add `log_warn!` at the
  `InvalidConfig` (default-applied) and `SnapshotOutOfBounds` (clamped) recovery
  points, mirroring ff-undo-redo config-clamp logging. Owner-gated; not required.
- **PA-WATCH-007** (consistency watch): DisplayLineMapper trait ownership --
  verify defined in display-line-mapping and only consumed here (resolve W1.4).
- **PA-WATCH-008** (consistency watch): scroll-amount (CSR/PAGE/HALF) source of
  truth -- viewport-and-scrolling Req 14 vs navigation-commands Req 20 (Phase DF /
  PA-INCOMPLETE-001). Verify no duplication when nav-commands is analysed (W1.7).
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-viewport-scrolling
  contributes 35 matches, including an em-dash inside the `MapperInconsistency`
  error-message literal (error.rs) that will render in output -- recommend ASCII
  `--` there during the project-wide cleanup. Rolled into PA-LOG-001.
