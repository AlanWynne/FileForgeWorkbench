# POST-REORG VERIFICATION (2026-10-15) -- READ BEFORE RESUMING

The workspace was reorganized and a copy script ran out of disk space mid-run.
This project moved to `c:\workspace\apps\FileForgeWorkbench` (was
`c:\workspace\VSC\FileForgeWorkbench`). A read-only verification pass
(`c:\workspace\_steering\verify-repos.ps1` + `check-branches.ps1`) confirmed the
following.

## This copy is the canonical, COMPLETE repo -- nothing was lost
- `HEAD = main`, tip `8ada2dc` ("docs+tools: re-sequenced core roadmap, status
  sync, liveness-probe tool, orchestration notes"), ahead of `origin/main` by 1.
- All 10 local branches are present and intact, including the two WIP feature
  branches that had been checked out as worktrees in the old location:
    - `feature/windowed-record-f3`     -> af2927e ("WIP: editor F3 ... pre-stall checkpoint")
    - `feature/localization-phase2`    -> d666b59 ("WIP: l10n FEAT-001 ... pre-stall checkpoint")
  Both are REAL branch refs (not dangling objects) -- verified reachable. Safe.
- All remote-tracking refs and tags (phase-ax, wrf-foundation-pre-rebase, ...)
  are present.
- Untracked work preserved: `.agents/tasks/*`, `docs/source-documents/
  FFWB-OWL-Agent-Concept.md`, `help/features/large-file-editing.help.md`, and the
  `tools/powershell/*` + `tools/python/*` scripts.
- `git fsck` shows only dangling objects (normal, not corruption).

## Two cleanup items (NON-destructive to branches) -- do before parallel work
1. STALE WORKTREE ADMIN ENTRIES. `git worktree list` still shows two worktrees
   pointing at the dead old path and marked `prunable`:
       C:/workspace/VSC/FileForgeWorkbench/.worktrees/editor-f3        [feature/windowed-record-f3] prunable
       C:/workspace/VSC/FileForgeWorkbench/.worktrees/l10n-extraction  [feature/localization-phase2] prunable
   The `.worktrees\` dirs did NOT survive the crash, so these registrations are
   dead. Clear them with `git worktree prune` -- this removes the admin entries
   ONLY and keeps both branches. To work on those branches in parallel again,
   re-create local worktrees afterwards, e.g.:
       git -C c:\workspace\apps\FileForgeWorkbench worktree prune
       git -C c:\workspace\apps\FileForgeWorkbench worktree add .worktrees/editor-f3 feature/windowed-record-f3
       git -C c:\workspace\apps\FileForgeWorkbench worktree add .worktrees/l10n-extraction feature/localization-phase2
2. STRAY `git\` FOLDER. There is a plain `git\` directory (NOT `.git\`) at the
   repo root -- it is a copy of PortableGit (git-bash.exe, mingw64, usr, ...),
   bundled binary cruft, not repository data. Safe to delete:
       Remove-Item -Recurse -Force c:\workspace\apps\FileForgeWorkbench\git

## Note on the dead original
`c:\workspace\VSC\FileForgeWorkbench` is effectively destroyed by the crash --
only a partial `git\etc` fragment survived, no `.git`, no working tree. There is
nothing to recover from it; this `apps\` copy is the real repo.

## FIRST STEP THIS SESSION (local verification before continuing)
    git -C c:\workspace\apps\FileForgeWorkbench status
    git -C c:\workspace\apps\FileForgeWorkbench worktree list
    git -C c:\workspace\apps\FileForgeWorkbench branch -vv
    git -C c:\workspace\apps\FileForgeWorkbench log -1
Confirm tip == 8ada2dc and all 10 branches present, then run the worktree prune
above before any parallel-branch work.
