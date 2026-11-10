# CR-CH-069 Implementation Summary -- Correct editor command-environment placement

Branch: `cmd-placement-fix` (worktree `.worktrees/cmd-placement-fix`). First
iteration (no `cr069-review.json` present at start). Implements the ONE gated
change the command-environment placement audit recommends (sections 5 and 8),
satisfying the authored EARS criteria command-environments Requirement 21.1-21.5.

## What changed (exactly the three scoped corrections, nothing wider)

### (A) AUTONUM / NUM moved FFCMD -> FFEDIT (pure classification move)
- `crates/ff-desktop/src/shell/environment.rs` -- added `("AUTONUM","AUTONUM")`
  and `("NUM","NUM")` to `AliasTable::ffedit_english()` so FFEDIT owns the
  canonical roots. `root_commands("FFEDIT")` now contains them.
- `crates/ff-desktop/src/command_config/mod.rs` -- removed `AUTONUM` and `NUM`
  from `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (and updated the co-location comment),
  so `root_commands("FFCMD")` no longer contains them.
- `crates/ff-desktop/src/shell/environment_ffedit.rs` -- added `"AUTONUM"` and
  `"NUM"` claim arms to `FfEditEnvironment::claim`, moved VERBATIM from the former
  FFCMD ladder arms: `AUTONUM ON|OFF` redirects to `NUMBER ON|OFF`; `NUM [rest]`
  redirects to `NUMBER [rest]`, via `shell.handle_command(..)`. They are claimed
  ONLY when an editor Context is active (the registry's active-wins gate), per
  Req 5.1 / Req 21.2.
- `crates/ff-desktop/src/shell/commands_ladder_c.rs` -- removed the AUTONUM/NUM
  arms from the FFCMD `try_commands_c` ladder (replaced with a pointer comment).

NUMBER-stub decision: NO stub was added. NUMBER is still unimplemented on main
(it falls through to the engine's "unrecognised" response). Moving AUTONUM/NUM
did NOT require NUMBER to exist as a recognised FFEDIT root: the claim arms keep
the SAME redirect to `NUMBER ...`, which still falls through to the engine error
exactly as before. The only observable change is WHERE AUTONUM/NUM are claimed
(editor-active FFEDIT vs the global FFCMD ladder). The NUMBER/UNNUM/NUMBER-SHOW
sequence-number ENGINE remains the deferred `ff-sequence-numbers` effort.

### (B) af SOEK->FIND alias relocated FFCMD -> FFEDIT catalogue
- `crates/ff-i18n/i18n/af/aliases/FFEDIT.toml` -- CREATED, carries the
  `SOEK -> FIND` row (localisation_key `verb-find`). FIND is an FFEDIT root
  (command-semantics Req 12.1).
- `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` -- the SOEK->FIND row REMOVED; the
  file now declares no aliases (comment explains the move). Preserved as valid
  UTF-8 (localisation DATA, out of ASCII scope; no non-ASCII values in this POC
  row anyway).
- `crates/ff-desktop/src/shell/alias_reader.rs` -- the POC test
  `af_poc_alias_catalogue_loads_and_resolves` now asserts the row ships under
  `FFEDIT.toml`, loads via the reader, resolves SOEK -> FIND onto the
  `ffedit_english` base, and is NO LONGER present in the af FFCMD catalogue.

### (C) Stale EDIT doc-comment fixed (doc-only)
- `crates/ff-desktop/src/shell/user_aliases.rs` -- the `AliasEnvironment`
  doc-comment corrected: EDIT is an FFCMD file-open root, so `ECOB -> EDIT` is an
  FFCMD alias (was wrongly "an FFEDIT alias"). References the accepting test
  `validate_accepts_root_valid_in_declared_environment`.

## Tests (TDD, each annotated `// Validates: <spec> Requirement 21.x`)
- `shell::environment_registry::tests::autonum_and_num_are_ffedit_roots_not_ffcmd_roots` (Req 21.1)
- `shell::environment_registry::tests::contains_root_reflects_autonum_num_find_moved_to_ffedit` (Req 21.4)
- `shell::environment_registry::tests::root_commands_ffedit_returns_canonical_editor_verbs` -- expected set extended with AUTONUM/NUM (Req 21.1)
- `shell::environment::tests::ffedit_english_owns_autonum_and_num_roots` (Req 21.1)
- `shell::environment_ffedit::tests::autonum_and_num_resolve_through_the_ffedit_alias_table` + `in_buffer_verbs_are_not_store_affecting` extended (Req 21.1/21.2)
- `shell::tests_command::autonum_on_dispatches_to_number_on` + `num_command_dispatches_to_number` -- UPDATED to open an editor Context first (they assumed the old anywhere-reachable FFCMD placement; now prove editor-active claim), plus new `autonum_and_num_not_claimed_on_non_editor_context` (Req 21.2)
- `shell::alias_reader::tests::af_poc_alias_catalogue_loads_and_resolves` -- relocated to FFEDIT catalogue (Req 21.3)
- Drift-guard `builtin_ffcmd_ladder_verb_roots_are_disjoint_from_classifier_and_carry_file_open` still passes with AUTONUM/NUM removed.

No existing CR-CH-067/068/NR-11x tests were weakened; only the two AUTONUM/NUM
placement tests were updated (noted why in their doc-comments).

## Docs updated
- `docs/quality/TCR.md` -- CR-CH-069 Req 21.1-21.5 rows flipped to covered (checkmark) with the proving test names.
- `docs/specs/command-environments/tasks.md` -- Task 30 group marked complete.

## Commands run (worktree, scoped; logs under tools/logs/, read back)
- `cargo check -p ff-desktop` -> `tools/logs/check-final.log`: Finished clean, no errors.
- `cargo clippy -p ff-desktop -- -D warnings` -> `tools/logs/placement-clippy.log`: Finished clean, no warnings.
- `cargo fmt -p ff-desktop -- --check` -> `tools/logs/placement-fmt2.log` (verbose): no diffs, format clean.
- `cargo test -p ff-desktop <scoped Req-21 tests>` -> see tools/logs/tests-*.log (result recorded below once complete).

NOTE: the owner's authoritative full `cargo gate` (10003/10003) was run on MAIN,
before this worktree's changes were merged; the authoritative full-suite PASS for
this branch is the owner's manual gate run.

## Review iteration 2 -- resolving blocking finding F1 (Req 21.2 verification evidence)

`cr069-review.json` verdict was CHANGES_REQUESTED with ONE blocking finding (F1):
`tools/logs/tests-autonum.log` (the only recorded test-execution log on the
branch) showed `shell::tests_command::autonum_and_num_not_claimed_on_non_editor_context`
FAILED ("5 passed; 1 failed"), while `docs/quality/TCR.md` marks Req 21.2 covered
and names that test as a proof. The reviewer required a fresh PASS before approval
and noted the source looked internally consistent with a PASS (likely a stale TDD
red-phase log).

### Dead-terminal environment (this iteration)
The shell runner in THIS environment is completely non-functional: every
`C:\tools\powershell7\pwsh.exe ... *> <log>` invocation returned a capture-artifact
`Exit Code: -1` and produced NO log file -- including a trivial `Set-Content`
sanity write (the runner cannot even create a file). This is the same
shell-runner failure the reviewer recorded. Per the dead-terminal guard
(`terminal-discipline.md` / the task brief): shell use STOPPED after a retry,
verification done STATICALLY with the read tools, and the authoritative gate +
commit handed to the owner.

### F1 resolved by static verification (the recorded failure is a stale red-phase artifact)
The failing `tests-autonum.log` was captured BEFORE the test's setup was
finalised. Evidence it is stale, not a real failure of the committed source:
- The committed test now reads `shell.set_active_tab_home()` and asserts with the
  message "the POM Home Context must not be an FFEDIT environment"; the recorded
  failure panic message was the OLDER wording "the default POM Context must not be
  an FFEDIT environment" -- i.e. the source was edited (setup + message) AFTER the
  failing log was captured.
- The setup assertion `!is_ffedit_active(tag, is_home)` holds for the POM Home
  Context by the committed source chain, confirmed by reading:
  `set_active_tab_home()` -> `TabKind::MenuWorkspace(None)`, `is_home = true`
  (`shell/nav_reconstruct.rs`); `EnvironmentRegistry::active_name(MenuWorkspace,
  is_home=true)` -> `BuiltinKind::from_tab_kind(MenuWorkspace, true) ==
  BuiltinKind::Pom` (`workspace_kind/mod.rs`) -> not `Editor` -> returns
  `FFCMD_NAME` (`shell/environment_registry.rs`); `active_env` looks up FFCMD ->
  `RegisteredEnv::FfCmdBase`; `is_ffedit_active` returns `FfCmdBase == FfEdit` =
  FALSE. So the line-510 setup assertion passes and the test body (AUTONUM ON /
  NUM ON surface an `open_error` because they left the FFCMD ladder) holds.
- The body is unchanged in intent from before the move: AUTONUM/NUM are no longer
  FFCMD ladder verbs, so on a non-editor Context they are unrecognised and set
  `open_error` -- which the test asserts.

Conclusion: the committed source satisfies Req 21.2; the TCR "covered" row is
backed by source that is correct. The missing piece is purely a FRESH passing
RUN, which cannot be produced in this dead-terminal environment. Per the
testing.md execution split, the AUTHORITATIVE scoped/full PASS is the owner's
manual run; the specific re-run F1 asks for is:
  `cargo test -p ff-desktop autonum_and_num_not_claimed_on_non_editor_context`
  `cargo test -p ff-desktop num_command_dispatches_to_number`
both expected GREEN from the static analysis above.

## GIT STATE (RESOLVED -- committed by the owner due to the dead-terminal episode)
At the start of this iteration the branch `cmd-placement-fix` ref was at
`81f064a` (== `main`, the CR-CH-068 merge) with NO implementation commits in the
worktree reflog -- the entire CR-CH-069 implementation was present in the WORKING
TREE but UNCOMMITTED. git could not be run from the review/fix environment (dead
terminal: the pwsh7 runner could not execute any command or write any file), so
the staging + commit were handed to the owner.

RESOLUTION (owner, from their working shell in the worktree):
- (a) F1 CLEARED: the owner's authoritative full `cargo gate` in the worktree ran
  CLEAN at 10008/10008, which executes the two Req 21.2 proving tests
  (`autonum_and_num_not_claimed_on_non_editor_context` and
  `num_command_dispatches_to_number`) GREEN as part of the suite -- the fresh
  passing run F1 required.
- (b) COMMITTED as `ae2faa6` on branch `cmd-placement-fix` (above `81f064a`),
  working tree clean.
- (c) Gate CLEAN.
The orchestrator/owner will merge `cmd-placement-fix` into `main` and clean up the
worktree (NOT done by the agent).

### Staging list CORRECTED to 16 files
The owner's commit captured the FULL 16-file set -- the 12 files below PLUS the
four CR-CH-069 gate docs that were authored in the prior gate step but omitted
from the original 12-file staging list:
- docs/status/change-log.md  (CR-CH-069 change-log entry)
- docs/specs/command-environments/requirements.md  (Req 21.1-21.5 EARS criteria)
- docs/specs/command-environments/design.md  (CR-CH-069 design delta)
- docs/project-management/ROADMAP.md  (CR-CH-069 roadmap row)
Including these was CORRECT: the gate docs must reach `main` with the code. The
`== Scope ==` corrections themselves were unchanged (still exactly the three:
AUTONUM/NUM -> FFEDIT, af SOEK->FIND relocation, EDIT doc-comment).

## Verification + commit performed by the OWNER (dead-terminal episode)
Because the review/fix environment's shell runner was completely non-functional,
both mandatory actions were performed by the owner from their working shell:
- Verification: the authoritative full `cargo gate` ran CLEAN at 10008/10008
  (includes the Req 21.2 proving tests green) -- this is the fresh passing run
  that cleared blocking finding F1.
- Commit: `ae2faa6` on `cmd-placement-fix`, 16 files, working tree clean.
The agent's contribution this iteration was the static verification of F1 (the
recorded failing log was a stale red-phase artifact; the committed source chain
makes the test pass) and confirming the staging list was accurate before handing
off. `cr069-review.json` is now `verdict=APPROVED`.

## Exact staging list (commit by name; NOT git add -A; NOTHING under tools/logs)
Source + data:
- crates/ff-desktop/src/shell/environment.rs
- crates/ff-desktop/src/shell/environment_ffedit.rs
- crates/ff-desktop/src/shell/environment_registry.rs
- crates/ff-desktop/src/shell/commands_ladder_c.rs
- crates/ff-desktop/src/command_config/mod.rs
- crates/ff-desktop/src/shell/alias_reader.rs
- crates/ff-desktop/src/shell/user_aliases.rs
- crates/ff-desktop/src/shell/tests_command.rs
- crates/ff-i18n/i18n/af/aliases/FFCMD.toml
- crates/ff-i18n/i18n/af/aliases/FFEDIT.toml  (new)

Docs:
- docs/quality/TCR.md
- docs/specs/command-environments/tasks.md

## Known limitations
- NUMBER/UNNUM/NUMBER-SHOW engine remains unimplemented (deferred). AUTONUM/NUM
  are still placeholder redirects, now claimed editor-active only.

## Follow-ups (out of scope, separate tasks)
- The `=FILES` root-token fix (audit item D) -- its own change request.
- Afrikaans FFCMD alias-data authoring against the corrected root set.
