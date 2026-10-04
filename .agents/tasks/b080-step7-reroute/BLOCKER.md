# B080 Step 7 -- BLOCKER (root cause confirmed): the split/dispatch-unify work is UNCOMMITTED in the main checkout; the branch 3b75928 does NOT contain it

Date: first implementation iteration (updated after parent push-back + full git verification).
Worktree: .worktrees/b080-step7, branch simplify/b080-step7-reroute.

## The one-line conclusion

The worktree is NOT corrupt and my file reads were accurate. The branch
`simplify/b080-step7-reroute` is based on commit **3b75928**, whose COMMITTED
tree still has the OLD monolithic shell layout (no `dispatch_command_string`
wired in, `dispatch_command_target` CustomWorkspace arm is a stub, ladder is
monolithic in `commands.rs`). The split + dispatch-unify work the plan assumes
exists ONLY as **uncommitted changes in the main checkout working directory**.
Step 7 cannot run against the committed branch until that work is committed.

## How this was proven (git, not inference)

1. Worktree HEAD == main HEAD, byte-identical:
   both `git rev-parse HEAD` = `3b7592897c3b14b578d64cf7ccae2ac40c87e2a2`.

2. The COMMITTED mod.rs at that commit is the OLD layout.
   `git show HEAD:crates/ff-desktop/src/shell/mod.rs` (run in the worktree)
   lists ONLY: command_line_outcome, commands, configurator, external_adapter,
   help, helpers, keys_editor, kinds_editor, menus_editor, nav_stack, render,
   render_chrome, reset_bare, target_dispatch, update, tests.
   There is NO `mod dispatch;`, NO `mod commands_ladder_a/b/c;`, NO
   `mod commands_fastpath;`, NO `mod commands_menu;`.

3. The worktree on-disk mod.rs blob == the committed blob.
   `git hash-object .../mod.rs` = `git rev-parse HEAD:.../mod.rs`
   = `94f0479dc7a885a1b17f84bc3c44239597c49e54`. So the worktree faithfully
   matches its commit -- not corrupt.

4. The committed shell TREE at 3b75928 has no split files.
   `git ls-tree HEAD:crates/ff-desktop/src/shell` matched only `dispatch.rs`
   and `target_dispatch.rs` for the pattern dispatch|commands_ladder|
   commands_fastpath -- i.e. the committed `dispatch.rs` exists but is NOT
   declared as a module (orphaned) and references `run_command_prelude` /
   `run_command_ladder` which do not exist in the committed tree.

5. The split work is UNCOMMITTED in the MAIN checkout.
   `git status` in the main checkout (branch decomp-and-scrm-wave, ahead 1)
   shows `M crates/ff-desktop/src/shell/mod.rs` AND dozens of untracked NEW
   files: commands_fastpath.rs, commands_ladder_a.rs, commands_ladder_b.rs,
   commands_ladder_c.rs, commands_menu.rs, commands_scrm.rs, commands_session.rs,
   commands_theme.rs, dispatch_ffedit.rs, construct.rs, handlers.rs,
   render_body.rs, render_split*.rs, state.rs, tests_common.rs, tests_focus.rs,
   tests_session.rs, ... (the entire split).
   Main on-disk mod.rs blob = `91b668f9cacc622e8f35b98f836863569960a245`
   != committed `94f0479...`. So the parent's "verified at 3b75928 in the main
   checkout" read the UNCOMMITTED working-tree version, not commit 3b75928.

## Why the earlier remedies did not help

- `git checkout -f HEAD` in the worktree was a no-op for mod.rs because disk
  already matched the committed blob (nothing to restore).
- Removing and recreating the worktree (`git worktree add ... 
  simplify/b080-step7-reroute`) re-materialised commit 3b75928 faithfully --
  which is the OLD layout. A fresh checkout cannot produce files that are not in
  the commit.

## What this means for Step 7

The plan is correct ABOUT THE INTENDED ARCHITECTURE, but that architecture is
not yet committed on the branch the worktree tracks. On the committed branch:
- `dispatch_command_string` is not a method on WorkbenchShell (orphaned file,
  not a module, references nonexistent helpers) -> reroute does not compile.
- `dispatch_command_target`'s CustomWorkspace arm sets an error string, so
  routing an in-scope verb through resolve_target would regress (error instead
  of opening the workspace).
- The ladder arms the plan deletes live in a monolithic `commands.rs`, not in
  `commands_ladder_a/b.rs`.

No functional change was committed. The trial reroute was reverted.

## What the owner / caller must decide

The split + dispatch-unify work must be COMMITTED before Step 7 can proceed as
written. Concretely, one of:

A. Commit the uncommitted split/dispatch-unify work currently in the MAIN
   checkout to `decomp-and-scrm-wave` (or a dedicated base commit), then re-base
   / re-create `simplify/b080-step7-reroute` on that new commit. Then Step 7 runs
   exactly as the plan specifies. (RECOMMENDED -- it is the state the plan was
   written against.)

B. Point the worktree at whatever commit/branch actually CONTAINS the committed
   split (if one exists elsewhere), and confirm `git show HEAD:.../shell/mod.rs`
   lists `mod dispatch;` + `mod commands_ladder_a;` before I retry.

I need the base commit that actually contains the committed split before I can
implement Step 7. Please confirm A or B (or provide the correct base ref).
