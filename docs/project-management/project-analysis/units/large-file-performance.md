# Analysis Record: large-file-performance (W4.13)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-large-file-performance` (GUI-independent rendering
  optimization infrastructure -- responsive 60fps for very long lines
  >10,000 chars, documents >1,000,000 lines, or both)
- **Spec files**: requirements.md (249 lines, 9 requirements), tasks.md
  (135 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 9 reqs, 249 lines. Cohesive concern (render optimization:
chunk rendering, line-layout + position caches, scroll prediction, invalidation).
12 files, no file over 400, sole dep `thiserror`. No split.

---

## 2. Cross-unit consistency -- PA-CONFLICT-013 (THIRD orphan crate, unwired)

`ff-large-file-performance` provides a full render-optimization toolkit:
`ChunkRenderer`, `LineLayoutCache`, `PositionCache`, `ScrollPredictor`,
`InvalidationCoordinator`, `StatusIndicator`, `Surface` trait, `PerfConfig`. But:

- It is used by NO crate (grep of all Cargo.toml: only its own; 0
  `ff_large_file_performance` refs anywhere else). ORPHAN.
- The shell / render path (`ff-desktop`) does NOT depend on it (0) and has 0
  `LineLayoutCache`/`PositionCache`/`ChunkRenderer`/`ScrollPredictor` references --
  so it is neither WIRED nor reimplemented inline in the shell. Unlike idle-styling
  (PA-CONFLICT-012, which had an inline reimplementation) this is orphan +
  MISSING integration: the editor render path uses none of it.
- document-model has 80 chunk/piece/LineIndex refs, but that is the piece-table /
  rope DOCUMENT STORAGE virtualization (a DIFFERENT concern from render-side layout
  caches) -- NOT a duplication of this crate. viewport-scrolling has 1 incidental ref.

Consequence: the spec's promise (60fps scrolling + sub-frame layout for >1M lines /
>10k-char lines) is very likely NOT realized in the running app -- the optimization
layer exists as a library but the render path does not call it.

Recorded PA-CONFLICT-013 (owner-gated, MEDIUM-HIGH): wire the shell/editor render
path onto `ff-large-file-performance` (galley/line-layout caching, position cache,
chunk rendering, scroll prediction, invalidation) so large-file responsiveness is
actually delivered; OR, if the perf strategy changed, reconcile the spec + delete
the orphan. This is the THIRD orphan crate (ff-file-tree PA-CONFLICT-011,
ff-idle-processing PA-CONFLICT-012, now ff-large-file-performance) -- a clear Wave-4
pattern of complete infrastructure crates that the shell never integrated.

### Relationship to the idle scheduler (PA-CONFLICT-012)

Large-file work (incremental layout, chunk render beyond viewport) is exactly the
kind of background work the idle scheduler (`ff-idle-processing`) is meant to drive.
Both are orphaned. A single integration effort would wire BOTH: idle scheduler grants
slices; large-file-performance caches/chunks the layout. Cross-referenced.

### Public types and ownership

- `ChunkRenderer`, `LineLayoutCache`, `PositionCache`, `ScrollPredictor`,
  `InvalidationCoordinator`, `StatusIndicator`, `Surface` trait, `PerfConfig` --
  sole-owned by `ff-large-file-performance` but ORPHANED (PA-CONFLICT-013).
- No deps (thiserror only) -- pure, GUI-independent (correct for infra; the problem
  is the missing consumer wiring).

### Cross-reference integrity

Cross-refs (viewport-scrolling, display-line-mapping, document-model, syntax) resolve
as sub-projects, but none depend on ff-large-file-performance (PA-CONFLICT-013).

---

## 3. Completeness

Tracking: all 135 sub-tasks `[x]`. Crate complete + tested (chunk render, caches,
scroll prediction, invalidation). FUNCTIONALLY complete AS A CRATE -- but not wired to
any consumer (PA-CONFLICT-013), so the large-file responsiveness the spec describes
is not realized in the app. No PA-INCOMPLETE for the crate itself; the gap is the
missing render-path wiring (architectural).

### TCR gap (PA-TCR-025) -- TOTAL ABSENCE

TCR.md has ZERO rows for ff-large-file-performance (grep = 0) across 9 reqs /
~70 criteria, despite the crate's tests. Recorded PA-TCR-025. (Same total-absence
pattern as idle-processing PA-TCR-024 -- both orphan infra crates.)

---

## 4. Logging audit

Scan of `crates/ff-large-file-performance/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: ABSENT (sole dep thiserror) -- NOT a dead-dep case. Clean.
- `println!` / `eprintln!`: 0
- `std::fs`: 0

Pure render-optimization infra -- zero-log defensible. Once wired
(PA-CONFLICT-013), dev-logging on cache hit/miss, invalidation, chunk-render decisions,
and scroll prediction would materially aid diagnosing large-file jank (CR-NR-058).
Recorded PA-LOG-038 (LOW): add dev-logging on cache/invalidation/chunk decisions under
the `dev-logging` gate when wired; the crate would add ff-logging then. No dead dep now.

---

## 5. Task revision proposals

- **PA-CONFLICT-013 (owner-gated, MEDIUM-HIGH)**: `ff-large-file-performance` is an
  ORPHAN (used by no crate; shell render path does not call it and has no inline
  equivalent). Wire the shell/editor render path onto it (layout/position caches,
  chunk render, scroll prediction, invalidation) so 60fps for >1M lines / >10k-char
  lines is actually delivered; OR reconcile spec + delete. Pairs with PA-CONFLICT-012
  (idle scheduler) and PA-CONFLICT-011 (ff-file-tree) -- THREE orphan crates. Code +
  owner decision.
- **PA-STD-053 (ASCII, runtime strings)**: 34 non-ASCII bytes (2 non-comment) --
  status.rs:65, status.rs:124 (likely em-dash / arrow in status text). Replace with
  ASCII substitutes. REFACTOR, no gate.
- **PA-TCR-025**: add TCR rows (0 for 9 reqs). No code.
- **PA-LOG-038 (LOW)**: dev-logging on cache/invalidation/chunk decisions once wired.

No PA-STD size item (no file over 400). No requirement CHANGE proposed; the crate is
correct -- the issue is that it is not wired to the render path.

---

## Summary

large-file-performance (`ff-large-file-performance`) is a clean, complete,
GUI-independent render-optimization toolkit (chunk rendering, line-layout +
position caches, scroll prediction, invalidation; sole dep thiserror, no dead
ff-logging, no cap violation, 135/135). The headline finding is PA-CONFLICT-013
(MEDIUM-HIGH): it is the THIRD orphan crate -- used by NO crate, and the shell render
path (`ff-desktop`) neither depends on it NOR reimplements it inline (0 layout-cache/
chunk-render refs). So the spec's promise (60fps for >1M lines / >10k-char lines) is
very likely NOT realized in the running app. document-model's 80 chunk/piece refs are
the piece-table DOCUMENT STORAGE (a different concern, not a duplication). This pairs
with the idle-scheduler orphan (PA-CONFLICT-012) -- large-file layout is exactly the
background work the idle scheduler should drive; one integration effort would wire
both. Three orphan crates now (ff-file-tree, ff-idle-processing,
ff-large-file-performance) form a clear Wave-4 pattern. Minor: total TCR absence
(PA-TCR-025), runtime-string ASCII (PA-STD-053), dev-logging deferred to wiring
(PA-LOG-038). The crate itself is sound; the gap is architectural (unwired).
