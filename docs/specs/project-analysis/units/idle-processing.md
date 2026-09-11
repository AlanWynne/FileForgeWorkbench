# Analysis Record: idle-processing (W4.12)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-idle-processing` (GUI-independent cooperative idle-time
  background-work scheduler -- priority dispatch, time-budget enforcement,
  cancellation, progress)
- **Spec files**: requirements.md (261 lines, 12 requirements), tasks.md
  (157 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 12 reqs, 261 lines. One cohesive concern (idle scheduler:
work sources, time budget, priority, cancellation, progress). 9 files, no file over
400, sole dep `thiserror`. No split.

---

## 2. Cross-unit consistency -- PA-CONFLICT-012 (orphan scheduler vs inline reimpl)

The spec positions ff-idle-processing as THE background work coordinator that grants
time slices to "syntax re-highlighting beyond the viewport, word-wrap height
calculation, fold-level computation, search index building" (Req intro). Consumers
(syntax-highlighting Req 9 idle styling, line-wrap Req 6.5 incremental wrap-height)
SPEC integration with it. But:

- `ff-idle-processing` is used by NO crate (grep of all Cargo.toml: only its own
  entry; 0 `IdleWorkContext`/`ff_idle_processing` references anywhere else). ORPHAN.
- syntax-highlighting has its OWN inline idle scheduler:
  `ff-syntax-highlighting/src/engine/idle_styling.rs` with `IdleStylingConfig`
  (`time_budget_ms: 10` -- the SAME 10ms budget as ff-idle-processing's
  `time_budget`) + `IdleStylingResult`, NOT using ff-idle-processing (0 dep, 0 ref).
- line-wrap (ff-wrap) has NO ff-idle dep either (its Req 6.5 incremental wrap-height
  is either inline or unwired).

So there are TWO idle-scheduling models: the standalone `ff-idle-processing` crate
(orphaned, tested, complete) and syntax-highlighting's inline idle-styling
(duplicating the time-budget/slice concept). This is the SAME anti-pattern as
ff-file-tree (PA-CONFLICT-011, W4.3): a complete standalone crate abandoned while a
consumer reimplements the concept inline. The 10ms budget match confirms they model
the same thing.

Recorded PA-CONFLICT-012 (owner-gated, MEDIUM-HIGH): either (a) rewire the idle
CONSUMERS onto `ff-idle-processing` (syntax-highlighting's idle_styling implements
the `IdleWorkSource`/context and registers with the scheduler; line-wrap likewise) --
preferred, unifies time-budget/priority/cancellation across all background work; OR
(b) delete the orphan crate if inline per-consumer idle is intentional. Option (a)
strongly preferred: a SINGLE cooperative scheduler is the whole point (Req intro) --
otherwise syntax idle-styling + wrap-height + fold + search-index each compete for
the frame budget independently with no coordination. Pairs with the W4.3 orphan
finding: TWO orphaned Wave-4 crates (ff-file-tree, ff-idle-processing) whose
consumers reimplement inline.

### Public types and ownership

- `IdleWorkContext` (time budget + cancellation flag), scheduler, priority dispatch,
  progress -- sole-owned by `ff-idle-processing` but ORPHANED (PA-CONFLICT-012).
- No deps (thiserror only) -- pure, GUI-independent, standalone. Correct design for
  a scheduler; the problem is it's not WIRED to its consumers.

### Cross-reference integrity

Cross-refs (syntax-highlighting, line-wrap, display-line-mapping fold, global-search)
resolve as sub-projects, but NONE depend on ff-idle-processing (PA-CONFLICT-012).

---

## 3. Completeness

Tracking: all 157 sub-tasks `[x]`. The scheduler crate is complete + tested (work
sources, time budget, priority, cancellation, progress). FUNCTIONALLY complete AS A
CRATE -- but it is not wired to any consumer (PA-CONFLICT-012), so the "grants time
slices to syntax/wrap/fold/search" integration the spec describes is NOT realized in
the codebase (the consumers idle independently). Not a false-positive tracking issue
for the crate's own tasks; the gap is the missing consumer wiring (architectural).
No PA-INCOMPLETE for the crate itself.

### TCR gap (PA-TCR-024) -- TOTAL ABSENCE

TCR.md has ZERO rows for ff-idle-processing (grep = 0) across 12 reqs / ~90 criteria,
despite the crate's tests. RECORDING gap. Recorded PA-TCR-024.

---

## 4. Logging audit

Scan of `crates/ff-idle-processing/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: ABSENT (sole dep thiserror) -- NOT a dead-dep case (it
  doesn't declare ff-logging). Clean.
- `println!` / `eprintln!`: 0
- `std::fs`: 0

A pure scheduler -- zero-log defensible. Once wired (PA-CONFLICT-012), dev-logging on
slice dispatch / budget-exceeded / cancellation would aid debugging background-work
starvation (CR-NR-058). Recorded PA-LOG-037 (LOW): add dev-logging on
dispatch/budget/cancel under the `dev-logging` gate when the scheduler is wired to
consumers; the crate would need to add ff-logging then. No dead dep now.

---

## 5. Task revision proposals

- **PA-CONFLICT-012 (owner-gated, MEDIUM-HIGH)**: `ff-idle-processing` is an ORPHAN
  scheduler (used by no crate) while syntax-highlighting reimplements idle-styling
  inline (`idle_styling.rs`, same 10ms budget) and line-wrap has no idle dep. Rewire
  the idle consumers (syntax re-highlight, wrap-height, fold, search-index) onto the
  ONE `ff-idle-processing` scheduler (preferred -- coordinates the frame budget across
  all background work) OR delete the orphan. Pairs with PA-CONFLICT-011 (ff-file-tree
  orphan). Code + owner decision.
- **PA-STD-052 (ASCII, runtime string + Cargo desc)**: 18 non-ASCII bytes
  (1 non-comment) -- em-dash in a runtime `#[error]` string (error.rs:18); ALSO the
  Cargo.toml `description` has a mojibake artifact ("... progress" preceded by a
  bad byte sequence). Replace with `--`; fix the Cargo desc. REFACTOR, no gate.
- **PA-TCR-024**: add TCR rows (0 for 12 reqs). No code.
- **PA-LOG-037 (LOW)**: dev-logging on dispatch/budget/cancel once wired.

No PA-STD size item (no file over 400). No requirement CHANGE proposed; the crate is
correct -- the issue is that it is not wired to its intended consumers.

---

## Summary

idle-processing (`ff-idle-processing`) is a clean, complete, GUI-independent
cooperative idle-time scheduler (time budget, priority, cancellation, progress;
sole dep thiserror, no dead ff-logging, no cap violation). The significant finding
is PA-CONFLICT-012 (MEDIUM-HIGH): it is an ORPHAN crate -- used by NO crate -- while
syntax-highlighting reimplements idle-styling INLINE (`idle_styling.rs`, same 10ms
budget) and line-wrap has no idle dependency. The spec's premise (ONE scheduler
grants slices to syntax/wrap/fold/search so they don't compete for the frame budget)
is NOT realized: the consumers idle independently. This is the SECOND orphan-crate-vs-
inline-reimplementation in Wave 4 (after ff-file-tree PA-CONFLICT-011) -- rewire the
consumers onto the one scheduler (preferred) or delete the orphan. Minor: total TCR
absence (PA-TCR-024), runtime-string ASCII + a Cargo-description mojibake (PA-STD-052),
and dispatch dev-logging deferred to the wiring (PA-LOG-037). The crate itself is
sound; the gap is architectural (unwired).
