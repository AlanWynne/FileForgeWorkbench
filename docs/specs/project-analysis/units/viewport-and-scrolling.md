# Analysis Record: viewport-and-scrolling

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-viewport-scrolling`
- **Spec folder**: `docs/specs/viewport-and-scrolling/`
- **Analysed**: Wave 1, task W1.2 (CR-NR-057/058 re-baseline)
- **Verdict**: COMPLETE (129/129 tasks `[x]`, TCR PASS). NOT a split candidate
  (borderline on size but cohesive). Zero-log defensible (pure GUI-independent
  model). One 400-cap refactor (viewport.rs 572), one crate-name-drift note, and
  a duplicate-trait CONFLICT (PA-CONFLICT-001, resolves at W1.4).
- **CR-NR-057/058 impact**: NONE (viewport specs untouched). Re-verified.

---

## 1. Scope summary

`ff-viewport-scrolling` is the GUI-independent viewport model (owned by the
editor session). 14 requirements: viewport state, vertical scroll commands
(clamped), cursor movement + coordination, full-file-range vertical scrollbar,
caret visibility policies, column affinity, horizontal scrollbar, mouse-wheel,
smooth scrolling, command-framework integration, display-line-mapping
integration, viewport persistence, large-file scrollbar precision, editor
scroll-amount (ISPF CSR/PAGE/HALF, CR-NR-035).

317 req lines, 14 requirements, single backing crate.

## 2. Split candidacy -- BORDERLINE, NOT recommended

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 317 lines (near); 14 reqs (>12) | YES (reqs) |
| 3+ distinct responsibilities | viewport-state/scroll-commands; scrollbar mapping; caret policies + affinity; smooth scrolling; display-mapping integration | Weak-Yes |
| 2+ crates | one crate | No |
| low-cohesion clusters | scrollbar mapping most separable; rest orbit viewport state | Weak |
| file-size pressure | `viewport.rs` 572 non-test over cap | Yes |

~2 criteria. **BORDERLINE, NOT a split candidate** -- cohesive around one viewport
model. The right fix is the file-size refactor (PA-STD-008), not a split.

## 3. Consistency / conflict -- PA-CONFLICT-001 (present, resolves W1.4)

- Public types owned here (Viewport, ViewportError, CaretPolicy, ScrollMode,
  Cursor model, scroll commands, viewport snapshot) -- sole owner. Added to
  consistency-matrix.
- **PA-CONFLICT-001 (present)**: viewport defines its OWN `pub trait
  DisplayLineMapper` (display_mapper.rs) with an IdentityMapper and stores
  `Option<Box<dyn DisplayLineMapper>>` -- it does NOT import the canonical
  `DisplayLineMapping` trait from ff-display-line-mapping (grep 0). viewport Req
  11.1 says the viewport SHALL accept "a `DisplayLineMapper` (FROM the
  display-line-mapping crate)", but that crate exposes `DisplayLineMapping` (W1.4)
  -- so this is a duplicate local abstraction, not the shared trait. Owner
  decision at W1.4: consume the canonical trait (drop the duplicate) OR document
  the thin viewport-facing trait + add an adapter.
- **PA-DOC-004 (crate-name drift)**: spec says `ff-viewport-and-scrolling`; crate
  is `ff-viewport-scrolling`. Reconcile the spec.
- Scroll commands (Req 10) are non-undoable (Req 10.6) -- consistent with
  command-framework Req 4 and undo-redo Req 10. No conflict.
- **PA-WATCH-008 (carried)**: Req 14 scroll-amount (CSR/PAGE/HALF, CR-NR-035)
  overlaps navigation-commands Req 20 (scroll-amount arguments, Phase DF). Verify
  no duplicate source of truth at W1.7.
- cursor_line/cursor_column here (viewport coordination) vs the editing caret
  (caret-and-selection W1.3 / edit-operations W1.5): verify boundary at W1.3.

## 4. Completeness

- Tasks: 129 `[x]`, 0 `[ ]`. TCR PASS (viewport model, cursor model, caret policy,
  scroll clamping; tests in tests/integration_tests.rs + property_tests.rs).
- No std::fs/tokio::fs (pure model).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` declared (wired, unused). DEFENSIBLE: pure
  GUI-independent computational model (viewport arithmetic, scroll clamping);
  errors surface via `ViewportError` (Result) to the caller. No requirement
  mandates a log record.
- Optional (PA-LOG-007): two `ViewportError` variants describe recovery actions
  (`InvalidConfig` "using default", `SnapshotOutOfBounds` "clamping applied"); a
  `log_warn!` there (or `log_debug!` under CR-NR-058 dev-logging) would aid
  diagnosis. Owner-gated; not a blocker.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log for a pure model)**.

## 6. Findings logged

- **PA-CONFLICT-001** (CONSISTENCY -- cross-unit; resolves at W1.4): viewport
  defines a local `DisplayLineMapper` trait instead of consuming the canonical
  `DisplayLineMapping` from ff-display-line-mapping, contrary to viewport Req
  11.1. Not bridged. Owner decision: consume the canonical trait (drop the
  duplicate) OR document the thin viewport-facing trait + adapter.
- **PA-STD-008** (REFACTOR -- 400-line cap): `viewport.rs` 572 non-test lines
  (tests external). Split by concern (viewport_scroll / viewport_scrollbar; keep
  viewport.rs state + coordination). REFACTOR, no gate. This is the fix for the
  size pressure, not a spec/crate split.
- **PA-DOC-004** (TRACKING-FIX): spec says `ff-viewport-and-scrolling`; crate is
  `ff-viewport-scrolling`. Reconcile. Doc-only.
- **PA-LOG-007** (LOGGING ENHANCEMENT, optional): `log_warn!`/`log_debug!` at the
  InvalidConfig + SnapshotOutOfBounds recovery points. Owner-gated.
- **PA-WATCH-008** (CONSISTENCY WATCH): scroll-amount (CSR/PAGE/HALF) source of
  truth -- Req 14 vs navigation-commands Req 20. Verify at W1.7.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-viewport-scrolling
  contributes matches incl. an em-dash inside the `MapperInconsistency` error
  literal (error.rs) that renders in output. Rolled into PA-LOG-001.
