# CR-CH-070 -- `=FILES` root-token fix: implementation summary

Iteration 1 (no `cr070-review.json` present). Implemented from the gate docs
authored in the prior step inside the `eqfiles-fix` worktree.

Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\eqfiles-fix`
Branch: `eqfiles-fix`

## What changed (scope: exactly Req 22, no wider)

`=FILES` was registered as a LITERAL root token where `=` is the ISPF JUMP
PREFIX and `FILES` is the real root command. The `=` prefix is already handled
at the single front door (`shell/dispatch.rs::dispatch_command_string`): it
strips the `=`, reinitialises the active tab's Navigation_Stack to the POM, then
re-dispatches the stripped remainder (`FILES`) through the generic bare-`FILES`
classifier arm. The literal `=FILES` was therefore redundant and only polluted
the FFCMD root set enumerated by `EnvironmentRegistry::root_commands("FFCMD")`
(CR-CH-067 Req 20).

Confirmed the `=` jump step covers the path, so removing the literal does NOT
break `=FILES` (no behaviour change, no warning needed):

1. `crates/ff-desktop/src/command_config/mod.rs`
   - Removed `"=FILES"` from `BUILTIN_WORKSPACE_VERB_ROOTS` (Req 22.1). Because
     `builtin_ffcmd_roots()` unions this const, `root_commands("FFCMD")` now
     returns `FILES` and no longer returns `=FILES`, and
     `contains_root("FFCMD", "=FILES")` is `false`.
   - Removed the dedicated `is("=FILES")` arm in `builtin_workspace_target_for`
     (Req 22.1); replaced it with an explanatory comment. The bare `is("FILES")`
     arm (-> `file_explorer`) and the `FILE CATALOGS` two-word arm are UNCHANGED
     (Req 22.2, Req 22.4).
   - Updated the two doc-comments that referenced `=FILES` as a classifier/root.
   - Updated the existing nav-verb test assertion from
     `builtin_workspace_target_for("=FILES") == Some(...)` to `== None`, with a
     CR-CH-070 comment (it asserted `=FILES` as a root).
   - Added Req-22 tests (classifier level): `equals_files_is_not_a_standalone_root_token`
     (22.1), `bare_files_still_opens_file_explorer_after_equals_files_removal`
     (22.2), `file_catalogs_two_word_verb_left_unchanged_after_equals_files_removal`
     (22.4).

2. `crates/ff-desktop/src/shell/environment_registry.rs`
   - UPDATED the existing FFCMD root-set enumeration test
     `root_commands_ffcmd_returns_complete_builtin_root_set` to also assert
     `=FILES` is NOT in the set (no parallel test added) (22.1).
   - Added `contains_root_ffcmd_files_true_equals_files_false` (22.1):
     `contains_root("FFCMD","FILES") == true`, `contains_root("FFCMD","=FILES") == false`.

3. `crates/ff-desktop/src/shell/locale_resolver.rs`
   - UPDATED the existing seed test `every_registered_ffcmd_root_resolves_to_itself`:
     removed the `=FILES` self-resolve assertion (it asserted `=FILES` as a seeded
     root) with a CR-CH-070 comment. `FILE CATALOGS` and the menu-option roots
     remain asserted (regression guard). This was the one other existing test
     asserting `=FILES` as a root; it had to change because the identity-row seed
     is driven by `BUILTIN_WORKSPACE_VERB_ROOTS`.

4. `crates/ff-desktop/src/shell/tests_nav.rs`
   - Added dispatch regression tests: `equals_files_jump_path_still_opens_file_explorer`
     (22.3 -- from a non-POM Keys Context, `=FILES` jumps to POM then opens the
     File Explorer) and `bare_files_still_opens_file_explorer_via_front_door`
     (22.2 full-shell front-door path).

5. `crates/ff-desktop/src/shell/commands_ladder_b.rs`
   - Corrected the historical DELETED-ARM NOTE comment (`FILES/=FILES` ->
     `FILES`). Doc-only.

6. `docs/quality/TCR.md`
   - Flipped the four CR-CH-070 rows (Req 22.1-22.4) to COVERED (green) with the
     proving test names. Updated the note paragraph with the verification caveat
     below. No other CR table touched.

## Tests NOT touched (deliberately)

- `tests_command.rs::equals_files_command_is_shell_intercept` /
  `files_no_prefix_command_is_shell_intercept` (Req 19.2/19.3) and the test-only
  helper `tests_common.rs::is_shell_command` (which lists `=FILES`). These assert
  a SEPARATE shell-intercept helper, not the FFCMD root set / classifier. They do
  NOT assert `=FILES` as an FFCMD root and stay green with the behaviour-preserved
  `=` jump path. Left unchanged per "do NOT weaken existing tests; update ONLY
  those asserting `=FILES` as a root".

## The FILE CATALOGS finding

`FILE CATALOGS` (`is("FILE") && arg == "CATALOGS"` -> the `files` workspace) is a
LEGITIMATE two-word classifier entry, NOT malformed. The gate docs (requirements
Req 22.4, design, change-log) explicitly leave it UNCHANGED. It was NOT fixed
(no justification to). Covered by
`file_catalogs_two_word_verb_left_unchanged_after_equals_files_removal`.

## Follow-up: Afrikaans alias data (`=` downstream)

The change-log records the sequencing: CR-CH-070 lands BEFORE the Afrikaans FFCMD
alias-data authoring, so that the corrected (clean) FFCMD root set is the anchor
the localisation DATA is authored against. The Afrikaans FFCMD alias-data
authoring is a SEPARATE, later DATA deliverable (out of scope here); with `=FILES`
no longer a root, an af alias row anchored to `=FILES` would now be correctly
rejected by the CR-CH-067 `contains_root` validation. No alias data was changed
in this task.

## Fix pass (iteration 2) -- resolving review blocker B1

`cr070-review.json` raised ONE blocker, B1: the required agent-level
compile/lint/format verification evidence was missing, and the TCR note
overstated the result ("finished CLEAN this session") with no log to back it,
contradicting the iteration-1 caveat. Iteration 1's code/doc changes were
reviewed as correct against every Req-22 criterion; only the verification gap
blocked.

Correction applied: the terminal was healthy this pass, so I RAN the scoped
agent-level commands and the Req-22 tests, redirected each to `tools/logs/`, and
read every log back (verify by EFFECT, not the Exit Code line). I then rewrote
the TCR note so it matches the actual, logged evidence.

## Commands run + log results (this fix pass -- all verified by reading the log)

- `cargo check -p ff-desktop *> tools/logs/eqfiles-check.log` -> log:
  "Finished `dev` profile ... in 0.47s". CLEAN, no errors.
- `cargo clippy -p ff-desktop -- -D warnings *> tools/logs/eqfiles-clippy.log`
  -> log: "Finished `dev` profile ... in 0.62s". CLEAN, no warnings (built with
  `-D warnings`, so any warning would have failed the build).
- `cargo fmt -p ff-desktop -- --check *> tools/logs/eqfiles-fmt.log` -> log
  EMPTY. `fmt --check` prints a diff only on a formatting violation and prints
  nothing when clean; the terminal was confirmed alive by the surrounding
  commands' non-empty logs, so the empty log means format CLEAN (verified by
  effect).
- `cargo test -p ff-desktop equals_files *> tools/logs/eqfiles-test-equals.log`
  -> 6 passed; 0 failed. Includes `equals_files_is_not_a_standalone_root_token`
  (22.1), `bare_files_still_opens_file_explorer_after_equals_files_removal`
  (22.2), `file_catalogs_two_word_verb_left_unchanged_after_equals_files_removal`
  (22.4), `contains_root_ffcmd_files_true_equals_files_false` (22.1),
  `equals_files_jump_path_still_opens_file_explorer` (22.3), and the retained
  shell-intercept guard `equals_files_command_is_shell_intercept`.
- `cargo test -p ff-desktop files *> tools/logs/eqfiles-test-files.log`
  -> 30 passed; 0 failed. Includes
  `shell::tests_nav::bare_files_still_opens_file_explorer_via_front_door` (22.2).
- `cargo test -p ff-desktop root_commands_ffcmd *> tools/logs/eqfiles-test-roots.log`
  -> 1 passed; 0 failed:
  `root_commands_ffcmd_returns_complete_builtin_root_set` (the enumeration test
  UPDATED in place, 22.1).

All edits are plain ASCII (`--`, straight quotes), per documentation-charset.

The AUTHORITATIVE full `cargo gate` / whole-suite run remains the OWNER's manual
run (the owner is the authority on a green run per testing.md's execution split).
The agent-level portion (compile-clean + lint/format-clean + scoped new tests),
which the split makes the agent's responsibility, is now DONE and evidenced by
the logs above.

## Exact staging list (committed on branch eqfiles-fix, NOT git add -A)

- crates/ff-desktop/src/command_config/mod.rs
- crates/ff-desktop/src/shell/environment_registry.rs
- crates/ff-desktop/src/shell/locale_resolver.rs
- crates/ff-desktop/src/shell/tests_nav.rs
- crates/ff-desktop/src/shell/commands_ladder_b.rs
- docs/quality/TCR.md

`tools/logs/` NOT committed. No merge/push/PR.
