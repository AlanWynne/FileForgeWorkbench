# Design review (revision) -- reconciled Alias Workspace Kind gate (CR-NR-116)

Scope: DOCS-ONLY rebase reconciliation of the Alias Workspace Kind gate on branch
`alias-kind-gate`, worktree `.worktrees/alias-kind-gate`, rebased onto main
`fd35620`. Reviewed `git diff fd35620..HEAD` (3 commits: `daecae4` + `7de41fc` +
`0b7fc2e`), the current `docs/specs/**` state, and `.agents/tasks/alias-kind-gate/`.

This is a REVISION. A prior review (`reconcile-review.json`, verdict
CHANGES_REQUESTED, 1 MEDIUM + 2 NIT) exists in the task dir. Commit `0b7fc2e`
("address reconcile-review findings") was authored to close it. I first re-check
the three prior findings against the new state, then review what changed.

Verdict: APPROVED (0 HIGH, 0 MEDIUM; 1 NIT).

---

## Re-check of prior findings

### Prior F1 [MEDIUM] -- fixture/golden omitted the mandatory `X -> EXIT` seed row -- CLOSED

The prior review found design 9.11.1 and golden task 9.6.1/9.6.2 showed only
`SS`/`CS`/scratch with `Aliases: 2`, contradicting 7.19/7.28/(e) which require the
`X -> EXIT` seed row always present. Commit `0b7fc2e` applied the PREFERRED fix:

- Design 9.11.1 now shows the seed row as the FIRST data row:
  `____ | [ffcmd v] | [EXIT v] | X | ____ | Exit the application`, followed by
  `SS`, `CS`, and the scratch row; the status band now reads
  `Aliases: 3 | Rows 1-3 visible`. Verified by reading the current design.md.
- The 9.11.1 prose now names the first row "the mandatory `X -> EXIT` FFCMD seed
  row (Requirement 8)".
- Task 9.6.1 golden working copy is now three rows (seed `X -> EXIT` first, then
  `SS`, then `CS`), status cells `Aliases: 3`, `Rows 1-3 visible`, and 9.6.1
  Validates list gained 7.28. Task 9.6.2 builds the three-row working copy and
  asserts "the `X -> EXIT` seed row is present as the first data row".

The fixture now MATCHES 7.19 ("INCLUDING the write-through `X -> EXIT` FFCMD seed
row ... editable") and 7.28 ("the `X -> EXIT` seed row present"). CLOSED.

### Prior F2 [NIT] -- gate criteria tagged `[CR-NR-115]` while owning id is CR-NR-116 -- CLOSED

Commit `0b7fc2e` added an explicit TAGGING NOTE at the top of the
command-configurator 7.14-7.28 block and the screen-snapshot-scrm 30.32-30.40
block ("the gate criteria carry the `[CR-NR-115]` tag because they EXTEND the
common-frame family ... the owning change-log id for this gate is CR-NR-116"),
and changed the ispf-table-editor Req 12 header to "CR-NR-116 (building on
CR-NR-115)". Ownership is now unambiguous. CLOSED.

### Prior F3 [NIT] -- design 9.11.2 D5 conflated `SS` with the CR-CH-116 seed -- CLOSED

D5 now reads "(3) the `SS` row is a shipped SCRM alias row (its template is
`None`; ...), distinct from the CR-CH-116 `X -> EXIT` seed row (Requirement 8)".
"Seed row" is reserved for `X -> EXIT`; `SS` is a SCRM alias row. CLOSED.

---

## Mechanical / structural checks (all PASS)

- **Clean tree, descendant, docs-only.** `git status` reports working tree clean.
  `git merge-base --is-ancestor fd35620 HEAD` returns 0 (HEAD `0b7fc2e` is a
  descendant of `fd35620`). `git diff --stat fd35620 HEAD` shows 12 files, all
  `*.md` under `.agents/tasks/`, `docs/specs/`, `docs/status/`. NO `.rs` staged or
  changed (+1349 / -2 lines).
- **Both sides of every doc conflict preserved.** command-configurator keeps
  pre-existing Req 7.1-7.13 and Req 8 (CR-CH-116 `X -> EXIT`) AND adds the gate's
  Req 7.14-7.28 (7.14-7.28 sit BEFORE Req 8). ispf-table-editor keeps Req 1-11
  (plus the out-of-order Req 8 "Scope and non-goals") AND adds Req 12.
  screen-snapshot-scrm keeps main's 30.1-30.31 (30.31 is `[CR-NR-117]`) AND adds
  30.32-30.40. Nothing from main or from the gate was dropped.
- **Renumbering is clash-free and unique.**
  - screen-snapshot-scrm: main owns 30.1-30.31 (verified by reading the file:
    30.29 is CR-NR-115's frame rule, 30.30 the grid-merge, 30.31 CR-NR-117's
    per-Kind content). The gate adds 30.32-30.40, contiguous, no gap, no overlap.
    Matches the gate's stated 30.27-30.35 -> 30.32-30.40 move.
  - command-configurator: Req 7.1-7.28 contiguous; Req 8 (CR-CH-116) follows. No
    clash.
  - ispf-table-editor: Req 12 (12.1-12.8) unique (Req 1-11 pre-exist).
- **Cross-references updated.** The 30.27-30.35 -> 30.32-30.40 move is reflected
  in command-configurator (7.17, 7.20, 7.24, 7.25, design 9.11.x), ispf-table-editor
  (Req 12 Source note "renumbered from 30.27-30.35 at rebase", 12.3, 12.7),
  workspace-framework (7.25 pointer), screen-snapshot-scrm design, Phase 2I tasks,
  and the change-log. Spot-checked; consistent.
- **CR-NR-116 is unique.** `git grep CR-NR-116 fd35620` finds it ONLY in main's
  change-log as RESERVATION notes in the CR-NR-117 and CR-CH-116 entries
  ("CR-NR-116 is used/held by the parked branch alias-kind-gate"). Main never took
  CR-NR-116 for a different change. The gate's "nothing on main since took it"
  claim is correct.
- **Framework conformance (CR-NR-115 / 117 / CH-116).** The criteria build ON the
  framework and do not reshape a public type:
  - The design 9.11.1 fixture shows the MERGED frame/grid rule under the
    `Command ===>` line (7.16, 30.35: the grid's own top rule merges with the
    frame-owned rule, not "frame suppresses its rule") and a
    `Last: 12:06 | Prof: def | Zoom: 100% | Aliases: 3 | Rows 1-3 visible` status
    band -- Zoom as a COMMON segment, the alias cells APPENDED after
    `Last | Prof | Zoom` (7.25, decision (d), screen-snapshot-scrm 30.10).
  - 7.17 states the Alias grid reuses the SHARED framed renderer
    (screen-snapshot-scrm 30.32-30.39, ispf-table-editor 12.1-12.3) and
    introduces no bespoke grid.
  - 7.24 / 30.40 reuse the `markdown_sections` clipboard mechanism (the same one
    the Menu Kind uses, CR-NR-117).
  - The `X -> EXIT` seed row (CR-CH-116 Req 8) is accounted for as present and
    editable in 7.19 and asserted present in 7.28, and is the first data row in
    the normative fixture and golden.
  - No framework change; five additive items correctly flagged in design 9.11.8
    and the change-log FRAMEWORK VERDICT.
- **Seven owner decisions (a)-(g).** Internally consistent; (c), (d), (e) and (g)
  are each flagged as basis-changed-by-rebase. (d) Zoom as a common segment is
  reflected in 7.25 and the fixture status band. (e) value fidelity vs the seed
  row is reconciled: the change-log records the seed row "accounted for in the
  Alias grid (value fidelity + sort)", 7.19 mirrors stored values with original
  case AND includes the seed row, and the seed is now the first fixture/golden
  row. Summary ends `GATE_READY_FOR_APPROVAL`.

---

## Findings

### 1. [NIT] The task-folder `gate-summary.md` target-layout fixture is now stale vs the normative design 9.11.1

Commit `0b7fc2e` updated the NORMATIVE fixture (design 9.11.1), the golden tasks,
and added a "Review findings addressed" section to `gate-summary.md`, but it did
NOT update the ASCII target-layout block INSIDE `gate-summary.md`. That block
still shows the pre-fix state: three content rows (`SS`/`CS`/scratch), no
`X -> EXIT` seed row, status `Last: 12:06 | Prof: default | Aliases: 2 | Rows 1-2
visible`, and NO `Zoom:` segment. The normative spec (design 9.11.1) now shows the
seed row first, `Aliases: 3 | Rows 1-3 visible`, and the `Zoom: 100%` common cell.

This is a working-note inconsistency, not a normative-spec defect: the gate-summary
is an orientation document and the spec files (which an implementer builds against)
are correct and self-consistent. Severity NIT -- it does not change the verdict,
but a reader comparing the two will see a contradiction.

Concrete fix: replace the ASCII fixture block in `gate-summary.md` with the current
design 9.11.1 fixture (seed `X -> EXIT` as the first data row, `Zoom: 100%` present,
status `Aliases: 3 | Rows 1-3 visible`), OR add a one-line note under the block
"Normative fixture is design 9.11.1; this copy is illustrative and may trail it."

---

## Verified Assumptions

- HEAD `0b7fc2e` is a clean descendant of `fd35620`; working tree clean; no `.rs`
  in the diff (12 `*.md` files only). [git status / merge-base / diff --stat]
- screen-snapshot-scrm Req 30 runs 30.1-30.31 on main (30.31 `[CR-NR-117]`) and
  the gate adds a contiguous 30.32-30.40 with no clash. [read requirements.md]
- command-configurator keeps Req 7.1-7.13 + Req 8 (CR-CH-116) and adds 7.14-7.28
  before Req 8; no clash. [read requirements.md]
- ispf-table-editor adds a unique Req 12 (12.1-12.8); Req 1-11 pre-exist.
  [read requirements.md]
- CR-NR-116 appears on main (fd35620) only as reservation notes pointing at this
  branch; it was not taken by any other change. [git grep CR-NR-116 fd35620]
- The design 9.11.1 fixture shows the merged frame/grid rule under the Command
  line, the `X -> EXIT` seed row as the first data row, and a
  `Last | Prof | Zoom | Aliases | Rows visible` status band; 7.17 reuses the
  shared framed renderer; 7.24/30.40 reuse `markdown_sections`.
  [read design.md / requirements.md / diff]
- Prior F1/F2/F3 are each addressed by `0b7fc2e` and re-verified against the
  current spec text. [read requirements.md / design.md / tasks.md diff]
- The `X -> EXIT` seed row is file-backed, editable, written on first run and
  rebuilt by Reset bare (CR-CH-116 Req 8), and is now accounted for in the Alias
  grid value-fidelity (7.19) and verification (7.28). [read requirements.md]

## Unverified / Wrong Assumptions

- Not independently re-verified (out of scope for a docs reconcile, no build/test
  run): that `alias_reader.rs` / `render/framed.rs` / `render/markdown.rs` /
  `ColumnSpec` behave exactly as the investigation sections (design 9.11.2,
  ispf design "Investigation verdict") describe. These are stated as
  read-against-the-code findings; this review did not re-read those source files.
  They do not affect the verdict.
- The stale `gate-summary.md` fixture (finding 1) is the only remaining internal
  inconsistency found; it is a working note, not a normative spec, so it is a NIT.
