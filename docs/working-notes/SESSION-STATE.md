# Session State / Handoff

Last updated: end of the split/detach bug-fix session.

## Where we are

`main` is at commit `d81a844`, pushed and in sync with `origin/main`. Working
tree clean except 5 pre-existing untracked files (see "Loose ends").

Core-building mode: work directly on `main` (no feature branches for now, owner
preference).

## Done this session (all on main, verify.ps1 CLEAN, no existing test modified)

- **CR-CH-041** (commit `78dc42d`): instance owns its chrome; region is
  placement; per-instance chrome (incl. Kind menu bar) renders in-region while
  split; placement derived from the layout snapshot; core has one tab system.
  Requirements gate ran (layout-and-docking Req 16, workspace-kinds Req 4.6,
  menu-and-statusbar Req 16.15) + two-slice implementation (IRP-a foundation,
  IRP-b visible chrome). 12 tests.
- **B072 / B073 / B074** (commit `daefd91`): three post-split/detach bugs.
  - B072: split command line now renders under the Title_Line (was
    region-bottom). Pure `split_region_strip_rects` helper.
  - B073: while split, Tab/Shift+Tab stays within the focused region
    (`handle_split_region_tab` in update.rs cycles command field <-> region
    menu-first, consumes Tab so egui never walks cross-region). Region switching
    via FOCUS/click.
  - B074: `with_workspace_context` saves/restores the primary active tab by
    stable `TabId` (was index) so a detached `=0.m` (insert_pom_tab index shift)
    no longer changes the primary window's context.
- **CR-NR-095** (commit `d81a844`): logged (PENDING GATE, deferred) --
  configurable command-line position (Top | Bottom) as a Workspace Kind profile
  attribute. NOT built.

## RECOMMENDED NEXT STEP

Do a **core-health triage pass over `docs/project-management/core-acceptance-test-plan.md`**:
walk the `[ ]` / `[B]` / `[F]` rows, establish what actually passes right now,
then fix the highest-severity real failures from evidence rather than picking a
CR blind.

If you'd rather keep momentum on a single scoped item instead, the cleanest pick
is **CR-NR-087** (bare UP/DOWN honour the active SCROLL amount) -- small,
well-scoped, and it fixes a currently-failing core test row (7.3a).

## Other queued candidates

- Per-instance **keylist** in-region: the third leg of the chrome trio (menu bar
  + command line already per-region after CR-CH-041; keylist still shell-level).
  Natural continuation while the split code is fresh. Needs a small CR.
- **CR-NR-095**: the command-line Top/Bottom feature just logged (deferred).
- **CR-NR-062**: File Explorer View/Edit + context-menu commands (larger).

## Loose ends (not blocking)

5 untracked files, unrelated to our work, left uncommitted -- decide per file:
- `TestData/ispf_editor_circular_change_tests.txt` and
  `tests/ispf_editor_circular_change_tests.txt` -- ISPF test data (maybe track).
- `docs/working-notes/The-three-level-model.md` -- design note (maybe track).
- `docs/source-documents/Integrating-ADVENT-as-a-Hidden-Easter-Egg-in-FileForgeWorkbench.md`
  -- your call.
- `tools/powershell/Add-EguiKittest - Copy.ps1` -- looks like an accidental
  duplicate; probably delete.

## Environment notes (for the next session)

- `ff-desktop` is a BINARY crate: `cargo test -p ff-desktop <filter>` (no --lib).
- verify gate: `tools\powershell\verify.ps1` (FULL, ~4-5 min via nextest). It
  DELETES `tools\logs\*.log` at start, so use `.txt` marker files for status.
  Authoritative signals: `tools\logs\verify.timing.log` (TOTAL line = done) and
  `tools\logs\ai-review.log` (EMPTY = clean).
- Terminal `get_process_output` echo is unreliable (concatenates the command
  buffer); write results to files and read them via the file reader instead.
- If a verify run reports a weird exit like `-1073741510`
  (STATUS_CONTROL_C_EXIT), it was interrupted -- re-run it; not a real result.
