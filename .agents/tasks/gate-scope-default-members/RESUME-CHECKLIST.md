# core-001 Resume Checklist (after shell restart)

Status at pause: all substantive work is WRITTEN TO DISK in two worktrees and is
UNCOMMITTED (nothing reverted). The workflow wf_f44e7588cdcafe4b FAILED only
because the shell runner repeatedly wedged (swallowed commands, spurious
Exit Code -1, no side effects) -- NOT because of a code problem. The core strip
compiled clean during live shell windows. Do NOT re-run the workflow; finish the
short remaining steps directly (optionally via one lightweight wf-coder step).

Verify by EFFECT (read the log), never trust the `Exit Code` line. Use the clean
wrapper: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "..."`,
one command per job, redirecting to a log and reading it back.

---

## What is already DONE and verified

### Repo B -- FileForgeWorkbench
- Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members`
- Branch: `feat/true-core-default-members` off `main`
- Root `Cargo.toml`: 32-crate `default-members` added (confirmed on disk), with a
  core-001 explanatory comment + the Option A note. `members` UNCHANGED (confirmed).
- `crates/ff-desktop/Cargo.toml`: parked deps dropped; KEPT ff-document-model,
  ff-viewport-scrolling, ff-edit-operations, ff-vfs (TabState infra). ff-shell
  NOT touched (Option A dropped the dead-dep removal).
- ~35 source files unwired: editor_panel / exclude_manager / nav_manager /
  scroll_amount / find_manager / explorer_view / nav_model / posix_provider /
  files_panel / catalog_registry / catalog_manager_dialog / dataset_alloc_dialog
  modules removed; shell submodules dispatch_ffedit / environment_ffedit /
  mainframe_backend / construct_provider / render_nav removed; FileEditor/Untitled,
  FilesPanel, FileExplorerPanel render arms stubbed to "unavailable in core" via
  InteriorFocus::none(); FFEDIT env + ff-vfs backend SAVE seam removed
  (environment_registry builds only FFCMD base); parked shell state fields +
  construct/update/session wiring removed; file.open/PendingOpenReq reduced to
  path+env; collect_search_roots reduced to workspace roots.
- `docs/reentry-register.md` written (Option A version).
- VERIFIED CLEAN during live shell windows: `cargo check -p ff-desktop`
  (0 errors/0 warnings) and `cargo clippy -p ff-desktop -- -D warnings`.

### Repo A -- cargo-gate
- Worktree: `C:\workspace\tools\cargo-gate\.worktrees\gate-default-members`
- Branch: `feat/gate-default-members` off `main` @ `1406b98`
- `Cargo.toml`: version bumped 0.2.0 -> 0.3.0 (confirmed on disk).
- Implementation of: default scope = Cargo default-members (emit no -p/--workspace);
  new `--workspace` (alias `--all`) = full members set; preserve --crate/--app-only/
  --closure-of/derive_orphans; precedence --crate > --app-only > --workspace/--all
  > default; new "default-members" scope label value for the history CSV (columns
  unchanged); exit codes unchanged. CHANGELOG behaviour-change note.
  (Written to disk; re-verify with the checklist below since the step was cut off.)

---

## Remaining work -- Repo B (do first)

Run everything with cwd = the worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members`.

1. Clean up STALE `#[cfg(test)]` modules that still reference removed items
   (these do NOT block `cargo check` but WILL block the owner's full gate):
   - catalog registry tests, the tab_manager save-seam test, and any editor_panel/
     exclude/nav tests in ff-desktop that referenced removed modules/types.
   - Find them: `cargo test -p ff-desktop --no-run` surfaces the compile errors in
     the test cfg; fix or remove the stale tests (do not weaken live assertions).
2. `cargo fmt` then `cargo fmt --check` (scoped is fine) -- must be clean.
3. Re-confirm `cargo check -p ff-desktop` and `cargo clippy -p ff-desktop -- -D warnings`
   are still clean after the test-module cleanup.
4. END-TO-END: `cargo build` with NO -p / NO --workspace (so it uses the new
   default-members). Confirm it builds ONLY the 32-crate core and that the binary
   is produced -- check by EFFECT with `Test-Path` on the `ffwb`/`FFWB.exe` target
   path (target\debug\ffwb.exe), not the exit code.
5. Commit per domain on `feat/true-core-default-members` (reversible commits),
   order Editor -> Navigator -> Catalog -> VFS-providers -> Volume, plus the
   default-members + reentry-register commit. Stage specific files (not `git add -A`).

## Remaining work -- Repo A (cargo-gate)

Run with cwd = `C:\workspace\tools\cargo-gate\.worktrees\gate-default-members`.

1. Verify the scope implementation compiles and behaves: `cargo check`,
   `cargo clippy -- -D warnings`, `cargo fmt --check` (read logs back).
2. Update/confirm the tests encoding the OLD default were flipped to the new
   default + tests added for --workspace/--all (config.rs default_scope_*,
   nextest.rs scope_args, fmt_clippy.rs clippy_args_*). Run the focused unit
   tests where quick.
3. Confirm CHANGELOG [Unreleased] has the behaviour-change note + the 0.3.0 head;
   confirm docs/ updated (STATUS.md created if still missing).
4. Commit on `feat/gate-default-members`.

## Cross-verify (after both compile/commit)

Build the gate from repo A's worktree, run it against repo B's worktree:
- bare `cargo gate` selects ONLY the 32 core crates,
- `cargo gate --all` and `cargo gate --workspace` select the full members set,
- `cargo gate --app-only` still works.
Capture which crates each mode selected to this folder. If a full `--all` run
risks a long/quiet hang, demonstrate the SELECTION (crate list per mode) without
running the whole suite to completion, and say so.

## Finalize (per repo, independently)

For EACH repo: rebase its branch onto `main`, then fast-forward `main` in the
user's primary checkout: `git -C <repo-root> merge --ff-only <branch>`.
(The branch is checked out in a worktree, so merge from the primary checkout.)
Do NOT delete the worktrees until the owner confirms; then remove each with
`git -C <repo-root> worktree remove <path>` and delete the branch.

Report per repo: branch, files changed, version bump (repo A), final 32-crate
default-members (repo B) + parked list, UI capabilities removed, verification
evidence (the fmt/build/Test-Path effects + the gate selection lists).

## Milestone note
This is milestone core-001. The deferred TabState refactor that would fully park
ff-document-model/ff-viewport-scrolling/ff-edit-operations/ff-vfs is ~core-002,
to be done WITH the editor rework. Recorded in docs/reentry-register.md.

---

## RESUME SESSION FINDINGS (post-restart)

- Shell is ALIVE again after restart. A PreToolUse proxy hook now runs commands
  and writes structured results to `C:\Users\A142367\.kiro\terminal-log\results\*.txt`
  (returncode + stdout/stderr). Read that result file after each proxied call.
- CRITICAL: the proxy/runner IGNORES the `cwd` parameter -- it runs every command
  from `c:\workspace\apps\FileForgeWorkbench` (the MAIN checkout), NOT the worktree.
  A bare `cargo ...` therefore compiles the MAIN checkout's (old, un-unwired) code
  and gives misleading errors. ALWAYS target the worktree explicitly with
  `--manifest-path 'c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members\Cargo.toml'`.
- With the correct manifest-path, the worktree PRODUCTION code compiles clean.
  `cargo test --manifest-path <worktree> -p ff-desktop --no-run` reports 68 errors,
  ALL in `#[cfg(test)]` modules referencing removed/parked items. Full log:
  `.agents/tasks/gate-scope-default-members/stale-test-errors.log`.
- Stale test modules to clean (remove/adjust tests validating now-parked behaviour;
  do NOT weaken live assertions for retained behaviour): tests_crch061.rs (catalog),
  tests_crch071.rs (environment_ffedit / FFEDIT), scroll_amount refs in
  tests_command.rs / tests_misc.rs / tests_session.rs, save_active_tab_via_backend
  tests in tab_manager.rs, environment_registry.rs tests (FfEdit / mainframe /
  backend_for / with_builtins_and_mainframe), catalog_registry refs in update.rs
  and session_manager.rs, and provider_registry / nav_open_path / files_panel /
  apply_pending_open_identity / PendingOpenReq.identity+recfm_lrecl refs.
- NOTE a possible production item: update.rs:338 references `ensure_default_home_catalog`
  inside a test `use super::{...}` -- confirm whether that symbol was legitimately
  removed (test-only cleanup) or is a real missing production fn before deleting.
