# RESUME STATE -- CR-CH-058 windowed/record foundation (saved for the owner)

Saved at the owner's request ("save state, going to bed, continue as you can").
Nothing destructive was done; the owner's uncommitted main-workspace work is
UNTOUCHED.

## Where everything is (git)

- `main` = `cdb0389` (pre-F1 gate commit). Its WORKING TREE has a LARGE body of
  UNCOMMITTED, UNRELATED work (dataset/volume/mainframe: ff-dscatalog, ff-dsalloc,
  ff-posix-provider, many docs/specs/{dataset-*, idcams-emulator, jes-emulator,
  volume-model, virtual-catalog-manager, virtual-file-system}, new untracked
  crates vsam_service.rs / service.rs / commands_environment.rs etc.). This is the
  owner's active work -- DO NOT stash/commit/overwrite it.
- `feature/environment-registry` (worktree .worktrees/env-registry) = `cdb0389`.
- `feature/windowed-record-foundation` (worktree .worktrees/wrf-foundation) =
  `20af25e`. THIS IS F1. Full gate CLEAN (owner ran it 2026-10-08: fmt ok, clippy
  clean, 9711/9711 passed, app-build ok). Working tree clean.
- `feature/vsam-service-wiring` (worktree .worktrees/vsam-wiring) = `bcad883`
  (holds the mainframe DESIGN docs + Task 17-22 impl notes).

## CR-CH-058 F1 = DONE (owner-gate CLEAN)

Piece-table spine + universal record model + byte-identical native SAVE, in
ff-document-model only, whole file still resident (windowing is F2). Commits on
the branch (newest last): 68f8f62 (structures) -> 7d7fde9 + 00f7086 (WAL/WBL
docs, CR-NR-107) -> 58b1b3f (integration + SAVE) -> 20af25e (proptests + TCR +
tasks + impl-note). 186 ff-document-model tests; clippy --all-targets -D warnings
clean; all files < 400 non-test lines. Impl note: `impl-note.md` (same folder).

## BLOCKED: F1 merge to main

`git merge --ff-only feature/windowed-record-foundation` from the main workspace
ABORTED: "local changes to docs/quality/TCR.md and docs/status/change-log.md
would be overwritten". Cause = the owner's uncommitted main-workspace work edited
those two shared files. The FF is clean at the commit level (main is a direct
ancestor of 20af25e, 5 commits behind); only the dirty working tree blocks it.

RESOLUTION (owner decision needed): either (a) owner commits/stashes the
main-workspace dataset/volume work first, then F1 fast-forwards cleanly; or
(b) leave main as-is and keep F1 on its branch (RECOMMENDED -- F1 is safe and
loses nothing by waiting; avoids entangling it with the active dataset work).
Agent will NOT touch the main-workspace changes without explicit instruction.

## Also pending (not blocking)

- CR-NR-107 (WAL/WBL self-logging save formats + walx CLI + ffmdx-style viewer):
  design note at .worktrees/wrf-foundation/.agents/tasks/wal-wbl/DESIGN-NOTE.md,
  logged PENDING GATE in change-log. Gate + build sequenced AFTER CR-CH-058 F2+.
- CR-NR-106 (configurable addressable-CE registry Context): logged, future.

## NEXT (when the owner is back)

1. Decide the F1-merge resolution (a or b above).
2. CR-CH-058 F2 (the windowing payoff, already in the approved gate -- document
   -model tasks 20.5/20.6, + viewport-and-scrolling task 17, + large-file
   -performance task 17): Window_Band (3-page band, prefetch above/below),
   scrollbar/extents sized on index Total_Records NEVER the resident window (FIXES
   the owner's scrollbar-collapse bug), zoom-never-loads, scroll load/evict at band
   edges with hysteresis+overscan, dirty pieces pinned, down/up N index-resolved
   jump with no intermediate reads. Build directly in the wrf-foundation worktree
   (agent-stall experience -> drive directly, not delegated). F1's save_image/
   rebaseline + OriginalIndex trait + PieceList are the base to build on.
3. Validates for F2: document-model Req 12.4-12.8; viewport Req 15.1-15.6;
   large-file-performance Req 10.1-10.5.

## Process reminders (this environment)

- Terminal stdout capture is flaky (Exit Code: -1). Redirect to a file and read
  it back; confirm every result against files/tests, never assume.
- The pwsh guard BLOCKS ';'-chained lines -- one command per call; for commits
  write the message to a file and use `git commit -F <file>`.
- Target a specific worktree with `git -C '<abs worktree path>' ...` because the
  shell wrapper's cwd may land elsewhere.
- Full `cargo gate --build` is the OWNER's manual step; agent runs only SCOPED
  `-p ff-document-model` checks and hands off.
