# CR-CH-070 `=FILES` root-token fix -- review

Branch: `eqfiles-fix`
Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\eqfiles-fix`
Diff reviewed: `git diff main...eqfiles-fix` (captured to `tools/logs/cr070-diff.log`).

Verdict: APPROVED -- no blocking findings.

## Scope of change (11 files)

Production code (3 files):
- `command_config/mod.rs` -- removed `"=FILES"` from `BUILTIN_WORKSPACE_VERB_ROOTS`; removed the `is("=FILES")` classifier arm (replaced with an explanatory comment); doc-comments updated.
- `shell/commands_ladder_b.rs` -- doc-only DELETED-ARM NOTE corrected (`FILES/=FILES` -> `FILES`).
- (`shell/dispatch.rs` is UNCHANGED -- the `=` jump step it owns is not in the diff.)

Test code (3 files, all `#[cfg(test)]`):
- `command_config/mod.rs` tests -- existing nav-verb assertion flipped from `Some(...)` to `None`; three Req-22 tests added.
- `shell/environment_registry.rs` tests -- existing enumeration test UPDATED in place (added the `=FILES`-absent assertion); one new `contains_root` test added.
- `shell/locale_resolver.rs` tests -- existing seed test's `=FILES` self-resolve assertion removed (FILE CATALOGS + menu-option guards retained).
- `shell/tests_nav.rs` -- two dispatch regression tests added.

Gate docs (5 files): requirements.md (Req 22), design.md, tasks.md, change-log.md (CR-CH-070), ROADMAP.md, TCR.md.

No i18n/alias data, no unrelated crates, no secrets. The committed staging list in the summary (6 files) is a subset of the full `main...eqfiles-fix` diff because the 5 gate-doc files were committed in the prior gate-authoring step on the same branch; expected.

## Criterion-by-criterion

### `=FILES` removed as a standalone root token -- PASS
`BUILTIN_WORKSPACE_VERB_ROOTS` (command_config/mod.rs) no longer lists `"=FILES"`; `FILES` remains. The dedicated `is("=FILES")` arm in `builtin_workspace_target_for` is gone, replaced by a comment explaining the `=` front-door handling. Verified by reading the source at lines ~172 (const) and ~278 (classifier). (Req 22.1)

### `root_commands("FFCMD")` contains FILES and NOT `=FILES`; enumeration test UPDATED -- PASS
`builtin_ffcmd_roots()` unions `BUILTIN_WORKSPACE_VERB_ROOTS`, so removal propagates. The existing enumeration test `root_commands_ffcmd_returns_complete_builtin_root_set` was UPDATED in place (added a `=FILES`-absent assertion) rather than duplicated. New `contains_root_ffcmd_files_true_equals_files_false` asserts FILES true / `=FILES` false. Log `eqfiles-test-roots.log` shows 1 passed. (Req 22.1)

### `=FILES` jump path PRESERVED; dispatch regression test proves it -- PASS
`shell/dispatch.rs::dispatch_command_string` (unchanged) strips a leading `=`, calls `reinitialise_active_tab_to_pom()`, then re-dispatches the remainder -- so `=FILES` -> jump-to-POM -> `FILES` -> `is("FILES")` arm -> file_explorer. `tests_nav::equals_files_jump_path_still_opens_file_explorer` drives this from a non-POM (Keys) Context and asserts `FileExplorerPanel`. Passes (`eqfiles-test-equals.log`, `eqfiles-test-files.log`). (Req 22.3)

### FILES alone still opens File Explorer, unchanged -- PASS
The bare `_ if is("FILES") => custom("file_explorer")` arm is retained. `bare_files_still_opens_file_explorer_after_equals_files_removal` (classifier) and `bare_files_still_opens_file_explorer_via_front_door` (full shell) both pass. (Req 22.2)

### No dispatcher added; framework mechanisms unchanged -- PASS
No new dispatch path, nav stack, or persistence format. The change builds ON the existing `=` front-door step and the existing bare-FILES classifier arm, and only removes a redundant literal. CommandTarget/nav/persistence untouched. Conforms to framework-conformance.md. (Not a framework change.)

### FILE CATALOGS left alone with justification -- PASS
The `is("FILE") && arg == "CATALOGS"` two-word arm is unchanged; the gate docs (Req 22.4, design.md) explicitly justify leaving it as a legitimate two-word verb. Guarded by `file_catalogs_two_word_verb_left_unchanged_after_equals_files_removal`. (Req 22.4)

### Existing tests not weakened -- PASS
The `tests_command.rs` shell-intercept tests (`equals_files_command_is_shell_intercept`, `files_no_prefix_command_is_shell_intercept`) are untouched and still pass; they assert the separate shell-intercept helper, not the FFCMD root set. The two assertions that DID change (nav-verb in command_config; seed in locale_resolver) both asserted `=FILES` AS A ROOT, which is precisely the behaviour being corrected -- updating them is required, not a weakening. FILE CATALOGS and menu-option regression guards are retained in both.

### No collateral damage; ASCII; no unwrap/expect in non-test code -- PASS
Changed-file set is scoped to CR-CH-070; no CR-NR-111/112/113 or CR-CH-066/067/068/069 source touched. Rust source greps clean for non-ASCII. The only non-test production changes are a const-entry removal, an arm removal, and comment edits -- no unwrap/expect introduced. No source file grew in non-test lines (changes are net-removal + cfg(test) additions), so the 400-line rule is not breached.

### TCR CR-CH-070 rows flipped to COVERED; other tables untouched -- PASS
The diff adds a single new "Phase (eqfiles-fix) -- CR-CH-070" section with four COVERED rows naming the proving tests for Req 22.1-22.4. No other CR table was modified (the diff is purely additive to TCR.md). Status glyphs use the TCR's established emoji convention, which documentation-charset.md explicitly permits for that one file.

## Verification evidence (read back, not re-run)

Per the step instruction I did not re-run suites; I read the coder's logs:
- `eqfiles-check.log`: `cargo check -p ff-desktop` finished clean, no errors.
- `eqfiles-clippy.log` / `eqfiles-clippy-final.log`: `cargo clippy -p ff-desktop -- -D warnings` full rebuild finished clean, no warning lines (any warning would have failed under `-D warnings`).
- `eqfiles-fmt.log`: empty -> `fmt --check` clean (verified by effect; surrounding logs confirm the shell was alive).
- `eqfiles-test-equals.log`: 6 passed / 0 failed (incl. the jump regression and the retained shell-intercept guard).
- `eqfiles-test-files.log`: 30 passed / 0 failed (incl. `bare_files_still_opens_file_explorer_via_front_door`).
- `eqfiles-test-roots.log`: 1 passed / 0 failed (the updated enumeration test).

This resolves the prior review's single blocker B1 (missing agent-level verification evidence and an overstated TCR note): the logs now exist and the TCR note's claims match them. The authoritative full-suite gate remains the owner's manual run per the testing.md execution split; that is expected and does not block.

## Findings

None. No blocking correctness, security, data, scope, framework, or documentation issues.
