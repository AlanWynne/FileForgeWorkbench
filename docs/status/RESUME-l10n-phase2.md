# RESUME -- Localisation Phase 2 (CR-NR-103 L10N.4)

Updated 2026-10-21 after Slice 1 completed and merged. Open FileForgeWorkbench in
a FRESH Kiro session (picks up the consolidated steering + the compile-only
verification policy in testing.md "Execution split").

## One-line status

SLICE 1 (FEAT-001/002/003) is DONE and MERGED to local `main`
(`44312e4`), FULL-GATE CLEAN (9885/9885 passed, 0 failed, owner-run 2026-10-21).
UNPUSHED. Next: Slice 2 = task 7 (panel-crate render literals).

## Where things stand

- `main` is at `44312e4` -- carries FEAT-001 (ff-i18n bootstrap wired in),
  FEAT-002 (menu title/description localisation), FEAT-003 (per-locale help).
  `main` is now 3 commits AHEAD of `origin/main` (UNPUSHED -- push when ready;
  that is 8ec2217..44312e4 = FEAT-001/002/003 plus the earlier session's commits
  already counted). Confirm with `git -C <repo> status -sb` next session.
- Branch `feature/l10n-phase2-slice1` is fully merged (same tip as main). It can
  be deleted: `git -C <repo> branch -d feature/l10n-phase2-slice1`.
- Worktree `.worktrees\l10n-phase2-slice1`: the git registration + merge are
  done, but the on-disk directory removal kept hitting terminal dead-episodes and
  may still be present. LOOSE END -- remove it when convenient:
  `git -C <repo> worktree remove .worktrees/l10n-phase2-slice1 --force`
  (or delete the folder in Explorer, then `git worktree prune`).
- Do NOT confuse with the OLD pre-reorg branch `feature/localization-phase2`
  (`d666b59`) -- stale, unrelated.

## Gate status

OWNER-run full gate 2026-10-21 16:08 from the worktree: CLEAN -- 9885 run,
9885 passed, 0 failed (fmt ok, clippy clean, build ok, app-build ok). The +13
over the 9872 baseline are FEAT-002 (5 i18n_keys tests) + FEAT-003 help tests.
No gate re-run is needed unless main is touched again.

## What Slice 1 delivered

- FEAT-001 (2c748ed): wired `ff-i18n` into `ff-desktop` (shell/i18n_bootstrap.rs
  installs the English catalogue at startup + registers the ui.locale hot-reload
  callback -- the first production caller of set_catalogue); 13 English menu keys
  in crates/ff-i18n/i18n/en/menus.ftl; closed task-4 bookkeeping.
- FEAT-002 (ee7c7bf): menu TITLE + option DESCRIPTIONS resolve through t() at
  render time, keyed by menu+command, with English fallback; command values stay
  unlocalised; logic in new menu_workspace/i18n_keys.rs (render.rs only swaps
  call sites). Req 7.1-7.4, 4.2.
- FEAT-003 (44312e4): ff-help content_loader resolves help/<locale>/ then English
  help/; registry gained additive replace_file_topics for the ui.locale
  hot-reload; ff-desktop help wiring passes the active locale. Topic_Key /
  cross-ref keys stay unlocalised. Req 6.1-6.4.
- Tasks 4, 5, 6 ticked in docs/specs/localization/tasks.md.

## What is NEXT -- Slice 2 and beyond (tasks.md Phase 2)

Each is its OWN slice, SAME compile-only verification policy (agents prove
compile-clean: scoped cargo check + clippy -D warnings + fmt --check, reading the
log; the OWNER runs the full gate manually -- NEVER have an agent run cargo
test/nextest/gate/full-gate, that wedges the terminal).

- Task 7: convert the clean dialog/panel-crate render literals to t("key")
  lookups -- one crate per step: ff-catalog-dialog, ff-dataset-alloc-dialog,
  ff-files-panel, ff-explorer-view, ff-theme-editor, ff-toolchain-panel. Keep
  each crate's egui_kittest behaviour tests green (behaviour-neutral at
  ui.locale=en). Add the English keys to crates/ff-i18n/i18n/en/ as data.
- Task 8: convert the ff-desktop render literals (labels/buttons/hover text).
- Task 9: convert the hard status/error messages (argument-bearing via t_args;
  embedded command verbs stay English as non-translated placeables).
- Task 10 (LATER capstone): author other-language DATA (fr/de .ftl catalogues +
  per-environment alias catalogues + per-locale help dirs). NOT until the
  extraction (tasks 7-9) is substantially complete.

## Known workflow caveat (recurring this project)

Workflow coder steps reliably do the WORK and compile clean, but have TWICE hung
at the step's final verify/commit/SIGNAL boundary (FEAT-001 and FEAT-003), not
advancing even after nudge messages. Recovery that worked both times: confirm the
edits compile (owner ran cargo check/clippy/build in the worktree), commit the
agent's staged work by hand, abort the stuck workflow, then ff-merge. If a slice-2
workflow hangs the same way, use the same recovery -- the work is on disk, just
uncommitted. (The per-step commit-after-each-FEAT structure makes this safe.)

## After L10N.4 -- ROADMAP Section 6a forward path

Volume UI (step 2), Volume-in-POM (3), Catalog UI rework (4), ff-vfs
volume/catalog wiring (5), File Navigator dataset/volume duality (6), editor
windowed-record F3-F5 (7, parallel), converging on the CORE #4 editor test pass.
