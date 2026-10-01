# RESUME: DECOMP Wave 6 (editor_panel substrate) + editor architecture re-eval

**Written:** 2026-10-01 (session paused by owner for a break)
**Author:** Kiro
**Branch to work on:** `decomp-and-scrm-wave` (currently at `525b5e8`, pushed to
origin). This already contains DECOMP Waves 1-5 and the egui-0.33 fix.

---

## STATE (resumed 2026-10-01) -- WAVE 6 EXTRACTION LAUNCHED (HYBRID Option A)

- Wave 5 is DONE and owner-confirmed (full verify.ps1 clean), fast-forwarded
  into `decomp-and-scrm-wave` @ `525b5e8` and pushed to origin.
- Architecture review COMPLETE (workflow `wf_fc14ea718e4dbacc`, report at
  `.agents/tasks/wave6-editor-architecture-review.md`). Owner chose HYBRID:
  mechanical Option-A extraction NOW (no new editor-aspect crates), with a
  SEPARATE gated stream to wire the editor onto existing aspect crates LATER.
- CR-NR-099 LOGGED in `docs/status/change-log.md` (New Requirements, PENDING
  GATE) -- "Wire the live editor onto the existing editor-aspect crates", to be
  done WHEN EDITOR TESTING BEGINS. This is the deferred refactor the owner asked
  to "make a note" of. OUT OF SCOPE for Wave 6.
- WAVE 6 EXTRACTION DONE (code-complete, pending owner full gate). The workflow
  died on the degraded terminal mid-Task-20; Kiro finished the extraction
  directly via file tools and the OWNER ran each task's scoped gate + commit.
  All three extractions landed, each scoped-gate green (owner nextest):
  - Task 20: `scroll_amount` -> `ff-scroll-amount` (verbatim leaf; shim). Commit
    `6e8b5fe`. 1071 tests pass.
  - Task 21: `exclude_manager` -> `ff-exclude-manager` (CLEAN SEAM: `u64` id +
    `line_count` + lazy `FnOnce() -> Vec<String>`; TabManager/runtime glue stays
    as ff-desktop adapter). 1061 tests pass.
  - Task 22: `editor_panel` pure helpers -> `ff-editor-panel`; TabState stays;
    adapter split into `editor_panel/{mod,input,paint}.rs` (<400 non-test each).
    1058 tests pass.
  - Docs: decomposition-tasks.md tasks 20-22 marked done; CR-NR-099 deferral
    noted. TCR unchanged (behaviour-preserving, no new criteria).
- NEXT: owner commits Task 22 + docs, runs finalize (rebase decomp-wave6 ->
  ff-only decomp-and-scrm-wave, NO push), then the full `verify.ps1` gate.
  After a clean gate, push `decomp-and-scrm-wave`; then remove the
  `.worktrees/decomp-wave6` worktree + branch.

## HOUSEKEEPING NOTES CARRIED FORWARD

- Interactive terminal was heavily DEGRADED all of the last session (mangled
  output, processes registering without executing). RELIABLE pattern: write a
  `.ps1` under `tools/logs/`, run via a FRESH background process
  `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -File <script>`
  redirecting `*>&1` to a log, then read the log file. If a script's log never
  appears, the process did not execute -- start a FRESH process, do not reuse a
  terminal id. The OWNER has a reliable terminal and runs the authoritative
  nextest/verify when asked.
- Two other worktrees exist and are UNRELATED to Wave 6:
  - `.worktrees/egui-033-upgrade` -- DELETED this session (was empty/clean).
  - `C:\workspace\VSC\FileForgeWorkbench.worktrees\organize-project-folder-structure`
    (a SIBLING path, not under `.worktrees/`) -- STALE 2026-09-03 reorg snapshot,
    owner wants to delete LATER. Its one unique doc was rescued to
    `docs/source-documents/ftso-minix-command-environment-discussion.md`; a
    `WORKTREE-STATUS-REPORT.md` sits in that worktree with full findings + the
    deletion steps.

---

## WAVE 6 SCOPE (per docs/specs/screen-snapshot-scrm/decomposition-tasks.md, tasks 20-22)

"editor_panel substrate + panel (hardest; TabState stays)". Leaves-first:
- Task 20: `scroll_amount` -> `ff-scroll-amount`
- Task 21: `exclude_manager` -> `ff-exclude-manager`
- Task 22: `editor_panel` -> `ff-editor-panel` (tab_state STAYS in ff-desktop)

### VERIFIED COUPLING FACTS (checked at the Wave 5 tip; RE-VERIFY at start)

- `crates/ff-desktop/src/scroll_amount.rs` -- ZERO `crate::` refs. TRUE clean
  leaf. Extract first.
- `crates/ff-desktop/src/exclude_manager.rs` -- **NOT the clean leaf the spec
  claims.** Couples to `crate::tab_manager::TabManager` (line ~22 + tests ~221)
  and `crate::tab_state::{TabId, TabState}` (line ~23; free fn
  `snapshot_lines(tab: &crate::tab_state::TabState, runtime: &Runtime)` ~196).
  Resolution needed before extraction (see architecture question below).
- `crates/ff-desktop/src/editor_panel.rs` (~1650 lines) -- couples to
  `crate::scroll_amount::ScrollAmount`, `crate::exclude_manager::ExcludeManager`,
  and `crate::tab_state::{TabId, TabState, UndoEntry}`. Many render/scroll/edit
  free fns take `&mut TabState` / `TabId` / `&ScrollAmount`. tab_state
  (TabId/TabState/UndoEntry) is the shell runtime model and STAYS in ff-desktop.

### KEY DECISION POINTS (unchanged, must be resolved before coding)

1. **exclude_manager tab-coupling.** It must not drag `tab_state` into a crate
   (tab_state stays in ff-desktop). Prefer refactoring its public surface so the
   shell passes in the already-snapshotted lines + a TabId, rather than a
   `&TabState`. Keep behaviour identical; weaken no test.
2. **tab_state boundary for ff-editor-panel.** Either make the moved editor
   functions generic over the tab type, OR share TabState/TabId/UndoEntry via a
   small shared types crate that both ff-desktop and ff-editor-panel depend on.
   Shell-entangled glue (live TabManager/shell) stays in ff-desktop as adapter.

---

## OWNER'S ARCHITECTURE QUESTION (RE-EVALUATE BEFORE PROCEEDING) -- DISCUSS FIRST

> "Perhaps in the process of decoupling the editor, we should re-evaluate the
> technical architecture of how to arrange all the editor requirements and
> possibly create crates for the different aspects of the editor even if they
> are sub-crates of the editor crate."

This is a scope-and-architecture decision that should be settled with the owner
BEFORE launching the mechanical Wave 6 extraction, because it changes the target
shape. Options to weigh when we resume:

- **(A) Mechanical Wave 6 as specced** -- one `ff-editor-panel` crate (plus the
  two leaf crates), behaviour-preserving, no re-architecture. Lowest risk, keeps
  the decomposition cadence, defers architecture to a later dedicated effort.
- **(B) Editor crate family / sub-crates** -- decompose the editor by ASPECT
  rather than one monolithic ff-editor-panel. Candidate aspects to map against
  the existing editor-related requirements/crates:
  - viewport/scroll (already `ff-viewport-scrolling`, `ff-scroll-amount` leaf)
  - caret + selection (already `ff-caret-selection`)
  - edit operations / undo-redo (already `ff-edit-operations`, `ff-undo-redo`)
  - exclude/show filter (`ff-exclude-show-filter`, `ff-exclude-manager`)
  - syntax highlighting, auto-indent, whitespace/guides, line-wrap, sequence
    numbers, display-line-mapping, text-decorations (each already has a crate or
    a spec under docs/specs/)
  - the egui render surface + the shell-tab glue (the genuinely shell-entangled
    part)
  Much of the editor's "aspects" ALREADY live in dedicated `ff-*` crates; the
  monolith in `ff-desktop/src/editor_panel.rs` is largely the egui RENDER layer
  + the wiring that binds those crates to the shell's TabState. So option (B) may
  be less "create many new crates" and more "define the editor render crate's
  seams cleanly so it composes the existing aspect crates, with TabState as the
  one shell-owned type passed in."

### Recommended next action when resuming

1. Do a READ-ONLY architecture review of editor_panel.rs + the existing
   editor-aspect crates and specs, and produce a short options doc: what the
   editor is actually composed of today, which aspects are already crates, what
   `ff-editor-panel` would own vs delegate, and whether sub-crates add value or
   just ceremony. (A `bundled://investigate` run is a good fit -- read-only,
   writes a findings report under `.agents/tasks/`.)
2. Owner picks (A) mechanical, (B) aspect-sliced, or a hybrid.
3. If (A): launch the Wave 6 workflow with the brief already drafted (see the
   aborted launch in session history -- it has the full coupling facts, the
   validation-critical forward-slash fileCheck path rule
   `.agents/tasks/decomp-wave6/task{N}-review.json`, base = decomp-and-scrm-wave,
   leaves-first order, scoped-gate contract, tab_state-stays decision).
   If (B)/hybrid: update decomposition-tasks.md Wave 6 (gate not required -- it
   is a refactor plan, no new behaviour) to reflect the chosen aspect crates,
   THEN launch.

## Wave 6 workflow launch brief (ready to reuse for option A)

The full self-contained workflow brief was composed in the session (the aborted
`run_workflow` call). Key reusable parameters:
- runLabel: `decomp-wave6-editor-panel`
- worktree: `git worktree add .worktrees/decomp-wave6 -b decomp-wave6 decomp-and-scrm-wave`
- fileCheck paths (forward-slash, main-root-relative):
  `.agents/tasks/decomp-wave6/task20-review.json` (and 21, 22), plan at
  `.agents/tasks/decomp-wave6/wave6-plan.md`
- shape: setup-worktree -> plan (wf-planner, resolves the 2 decisions) ->
  task20/21/22 repeat loops (wf-coder implement, semantic_reviewer last,
  stop on verdict=APPROVED, max 4; task22 max 6 since it may slice) ->
  docs-update -> finalize (rebase+ff onto decomp-and-scrm-wave).

-- End of resume note.
