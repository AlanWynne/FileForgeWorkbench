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

UPDATE (2026-10-09, Option A executed): the other chat committed+pushed the
dataset/volume work to origin/main as CR-CH-059 (main advanced cdb0389 ->
03fd2e1). F1 then DIVERGED from main (no longer a fast-forward). Per owner
"Option A", F1 was REBASED onto main (03fd2e1). Rebase CLEAN: the only conflicts
were in docs/status/change-log.md (both streams added a CR-NR-107 entry) --
resolved by KEEPING main's authoritative "Walrus external project" CR-NR-107 and
DROPPING the superseded FFWB-internal WAL/WBL version from this branch (that
design is preserved in .agents/tasks/wal-wbl/DESIGN-NOTE.md and git history). All
7 F1 commits replayed; the 4 ff-document-model code commits applied with NO
conflict (zero code-file overlap with CR-CH-059). Rebased tip = fb30f22. Scoped
re-verify on the rebased tree: cargo test -p ff-document-model --lib = 186
passed, 0 failed (ff-vfs recompiled against CR-CH-059 and F1 still green).

STILL PENDING: fast-forward main -> fb30f22. BLOCKED AGAIN (2026-10-09): main's
working tree now has FRESH uncommitted edits in
crates/ff-dscatalog/src/dataset_access/* -- the other chat is ACTIVELY working on
main right now. A fast-forward would move main's HEAD under a live session; the
agent STOPPED rather than disrupt it. The FF files (ff-document-model + 3 docs)
do NOT overlap the dataset_access edits, so no clobber risk, but moving main mid
-session needs owner confirmation. When the other session is at rest (or you say
go), FF is: git -C <main> merge --ff-only feature/windowed-record-foundation.
Full `cargo gate --build` on the rebased tree is still the owner's step before
calling the merge done. Safety: pre-rebase F1 tip was 628fb32 (recoverable).

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
