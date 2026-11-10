# CR-CH-071 Implementation Summary -- ISPF Save/End/Cancel/Return/Refresh for editable Contexts

Resume-and-complete of the CR-CH-071 implementation on branch
`crch071-save-cancel-refresh` (worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crch071-save-cancel-refresh`,
forked from consolidated main 42d6db8). The prior run had written the plan and
implemented all source + tests UNCOMMITTED; this run assessed, verified, finished
the documentation tasks, and committed.

## Criteria implemented (command-environments Requirement 20a; command-configurator Req 5.9 / 5.14 amended)

- 20a.1 SAVE=persist+stay; END (and F3)=save-then-leave; CANCEL/RETURN=discard+leave; REFRESH=discard+reload+stay.
- 20a.2 SAVE on an editable non-buffer Context persists the working copy through the shared seam; failed validation leaves the store UNCHANGED.
- 20a.3 CANCEL/RETURN discard working changes (no write) and leave via the per-tab Navigation_Stack.
- 20a.4 REFRESH discards, reloads last-saved from the backing store, stays; never writes.
- 20a.5 SAVE/CANCEL/REFRESH are FFCMD canonical roots (`root_commands("FFCMD")` + `contains_root`).
- 20a.6 FFCMD base routes SAVE/CANCEL/REFRESH (and END/RETURN for an editable Context) through the shared focused-editable-Context seam (persist/discard/reload), keyed by the active KindTag.
- 20a.7 FFEDIT buffer SAVE active-wins override is byte-identical (editor buffers are FFEDIT-active, claim SAVE before the FFCMD ladder).
- 20a.8 CANCEL/REFRESH are FFCMD-base-only in v1 (not FFEDIT roots).
- 20a.9 REFRESH (not UNDO) is the context-revert verb.
- 20a.10 Alias Editor private SAVE/CANCEL/END `Command ===>` interpreter retired -> one dispatch path (typed line == F3/global END).
- 20a.11 Shared seam applies uniformly; Alias Editor is the v1 implementor, the five sibling config editors are the fast-follow set (shaped-in, not wired in v1).
- 20a.12 No user alias may be named SAVE/CANCEL/REFRESH (rejected at save, skipped-with-WARN at load) -- automatic via the no-shadow invariant reading `builtin_ffcmd_roots()`.
- 20a.13 EXIT/QUIT/LOGOFF unchanged (app-exit family; END is the save-then-leave verb).
- 20a.14 Behaviour-preserving on every non-editable Context and the FFEDIT buffer.
- command-configurator Req 5.9 / 5.14: the Alias Editor's primary-command handling now delegates SAVE/CANCEL/END/RETURN/REFRESH to the single front door (FILTER/RESET stay panel-local; the I/R/D line-command pass still runs first, Req 5.22).

## Files changed (exact staging list -- stage BY NAME)

New source:
- `crates/ff-desktop/src/shell/editable_context.rs` -- the focused-editable-Context seam (EditableKind, editable_context_kind / editable_persist / editable_discard / editable_reload).
- `crates/ff-desktop/src/shell/tests_crch071.rs` -- the CR-CH-071 behaviour matrix (13 tests).

Modified source:
- `crates/ff-desktop/src/command_config/mod.rs` -- SAVE/CANCEL/REFRESH appended to `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (Req 20a.5).
- `crates/ff-desktop/src/shell/alias_editor.rs` -- Alias Editor seam impl (persist=save_user_aliases now returns Result<(),String>; discard=clear rows+error; reload=load_user_alias_rows+load_rows); save-side no-shadow test for SAVE/CANCEL/REFRESH.
- `crates/ff-desktop/src/shell/alias_editor_line_commands.rs` -- retire the private SAVE / CANCEL|END arms; fall-through routes to `dispatch_command_string`.
- `crates/ff-desktop/src/shell/commands_ladder_a.rs` -- FFCMD-base SAVE/REFRESH/CANCEL arms + amended END/RETURN, gated on `editable_context_kind()`.
- `crates/ff-desktop/src/shell/environment_registry.rs` -- CR-CH-071 root-enumeration test (`ffcmd_roots_contain_save_cancel_refresh`).
- `crates/ff-desktop/src/shell/mod.rs` -- declare `mod editable_context` + `#[cfg(test)] mod tests_crch071`.

Docs:
- `docs/specs/command-environments/tasks.md` -- marked tasks 32.1-32.4, 33.1-33.3, 35.1-35.7 `[x]` (and parents 32, 33); 34.1-34.5 (sibling fast-follow) and 35.8 (owner full-gate run) left `[ ]`.
- `.agents/tasks/alias-editor-save-semantics/crch071-impl-summary.md` -- this file.

NOTE on `environment_registry.rs`: the worktree diff showed it MODIFIED, which is the added CR-CH-071 root-enumeration test (`ffcmd_roots_contain_save_cancel_refresh`). It is part of the Commit Point A unit.

## TCR status (docs/quality/TCR.md)

The repository `docs/quality/TCR.md` is a CRATE-organised, gate-REGENERATED
coverage report (emoji status key per crate). It contains NO CR-CH-071 / Req-20a
rows -- the gate has not been run on this branch, and the owner has stated the
TCR is regenerated on every gate. There are therefore NO per-criterion
NOT-COVERED rows to flip in TCR.md (the plan's Task 10 assumed gate-seeded rows
that do not exist). The authoritative per-CR coverage record for this change is
the `docs/specs/command-environments/tasks.md` "Acceptance Criteria Coverage
(CR-CH-071)" table (criterion -> covering task mapping, already present) plus the
now-checked task boxes. TCR.md was deliberately NOT hand-edited (it is generated
and would be overwritten).

## Scoped verification commands run + results (verify-by-effect, logs read back)

All against the worktree via `--manifest-path <worktree>\Cargo.toml`, clean pwsh7
wrapper, redirected to `tools/logs/`, read back:

- `cargo check -p ff-desktop` -> `crch071-check.log`: `Finished dev` CLEAN, no diagnostics.
- `cargo clippy -p ff-desktop -- -D warnings` -> `crch071-clippy3.log`: `Finished dev` CLEAN, zero warnings.
- `cargo fmt -p ff-desktop -- --check` -> `crch071-fmt.log`: EMPTY log (no diffs) = formatting clean.
- `cargo test -p ff-desktop tests_crch071` -> `crch071-test-crch071.log`: build `Finished test in 2m 48s`, then
  **13 passed; 0 failed; 0 ignored** (full CR-CH-071 behaviour matrix: SAVE stays, END save-then-leave,
  END-fail stays+store-unchanged, CANCEL/RETURN discard+leave, REFRESH discard+reload+stay, non-editable no-op,
  FFEDIT active-wins boundary, EXIT app-exit unchanged, one-dispatch-path typed==global END, load-skip no-shadow,
  Menus editor not wired in v1).

Sibling unit tests (`ffcmd_roots_contain_save_cancel_refresh`,
`validate_rejects_save_cancel_refresh_shadow`,
`command_line_applies_line_commands_before_primary_command`) are in the SAME
compiled binary that passed; a separate scoped run was blocked by a terminal
dead-episode (logged in TERMINAL_DEFECTS.md). They were verified statically by
source read and are handed to the owner's full gate for the authoritative run.

## Sibling-editor status

v1 wires ONLY the Alias Editor. The five sibling config editors (Settings /
Menus / Theme / Keys / Kinds) are intentionally NOT wired: `editable_context_kind()`
returns `None` for their KindTags with a clearly-marked TODO per kind
(`// TODO(CR-CH-071 fast-follow, command-environments tasks 34.1-34.5)`). The
seam is shaped so each plugs in by adding its `EditableKind` arm. tasks 34.1-34.5
remain `[ ]`. A regression guard test (`menus_editor_is_not_wired_into_the_seam_in_v1`)
confirms the Menus Editor still behaves as today.

## Known limitations / follow-ups

- Sibling config editors (34.1-34.5): fast-follow, not in v1.
- The owner's authoritative full `cargo gate` (fmt + clippy + build + whole-suite)
  is task 35.8, left `[ ]` -- the agent compile/clippy/fmt-verified and ran the
  scoped CR-CH-071 suite green; the full gate is the owner's pre-merge run per
  gate-policy.md.
- Terminal dead-episodes recurred during this session (test build holding the
  shell); logged in `TERMINAL_DEFECTS.md`.

## Gate-failure fix iteration -- four "failures" were a threaded-cargo-test isolation artifact (NOT a CR-CH-071 regression)

The owner's full gate (run as THREADED `cargo test ... --bin ffwb`) reported five
failures. One -- `full_shell_theme_editor_type_name_and_save_creates_user_theme`
-- is the known out-of-scope B081 theme-editor flake (noted, not chased). The
other four were:
- `shell::alias_reader::tests::af_poc_alias_catalogue_loads_and_resolves`
- `shell::tests_locale_menu::full_shell_menu_option_shows_localised_surface_and_dispatches_root`
- `shell::tests_locale_menu_full::full_shell_per_part_fallback_never_blank`
- `shell::tests_locale_menu_full::full_shell_single_menu_auto_converts_between_locales`

### Diagnosis (ROOT CAUSE, evidence-based)

These are CR-NR-111 menu-localisation + the af alias POC tests. The root cause is
a PRE-EXISTING test-isolation race on the PROCESS-GLOBAL i18n catalogue
(`ff_i18n::set_catalogue`), which has NO serialization guard (confirmed by
search: no `#[serial]`, no shared catalogue Mutex). Mechanism: `make_shell()` ->
`install_catalogue_for_tests()` installs the ENGLISH base; ALL 13 of the new
`tests_crch071` call `make_shell()`. Under THREADED `cargo test`, one of those
English installs races on another thread against a locale test's Afrikaans/
localised install and clobbers it mid-test. The failure signatures prove exactly
this (the global was English when the test expected the localised catalogue):
- af_poc: `left: Some("FIND"), right: Some("SOEK")` -- t("verb-find") returned
  English because the af catalogue was overwritten.
- auto-converts: `left: ["F"], right: ["S"]` at the FIRST (localised) assertion.

CONCLUSIVE isolation evidence: `af_poc_alias_catalogue_loads_and_resolves` run
ALONE with `--exact` PASSES; two of the four (menu_option / per_part_fallback)
even PASSED inside a 4-test scoped `cargo test` run -- order/thread dependent, not
deterministic breaks.

CR-CH-071 added NO code touching `set_catalogue` or localised-surface derivation
-- it added three FFCMD roots + the editable-Context seam. Its ONLY effect on
these tests is MORE concurrent `make_shell()` catalogue installs, which raised
the clobber probability of a dormant race. So:

Per-test classification -- all four: pre-existing shared-global-catalogue race,
NOT a CR-CH-071 behavioural regression, and NOT a test needing an assertion
change for the new roots (they assert on localised surfaces, not on FFCMD
root-set composition). No test was weakened, changed, or deleted.

### Resolution (Option B, owner-decided) -- verified green under the canonical nextest harness

The project's canonical test harness is `cargo nextest` (`.config/nextest.toml`
present; `testing.md` names nextest), which runs each test IN ITS OWN PROCESS --
so the process-global catalogue cannot be shared and the race cannot occur.
`crates/ff-desktop/src/shell/i18n_bootstrap.rs` itself documents "nextest runs
each test in its own process". Verified by effect (logs read back):

- `cargo nextest run -p ff-desktop -E 'test(af_poc...) | test(full_shell_menu_option...) | test(full_shell_per_part_fallback...) | test(full_shell_single_menu_auto...)'`
  -> `crch071-nextest4.log`: **4 tests run: 4 passed** (process-per-test).
- `cargo nextest run -p ff-desktop -E 'test(tests_crch071)'` ->
  `crch071-nextest-mod.log`: **13 tests run: 13 passed**.
- `cargo check -p ff-desktop` -> `crch071-check2.log`: `Finished dev` CLEAN.
- `cargo clippy -p ff-desktop -- -D warnings` -> `crch071-clippy5.log`:
  `Finished dev` CLEAN, zero warnings.
- `cargo fmt -p ff-desktop -- --check` -> `crch071-fmt2.log`: EMPTY (no diffs) =
  clean.

NO production or test code change was required for Option B; the fix is to run
the suite under the canonical nextest harness.

### IMPORTANT -- which harness does the owner's `cargo gate` use?

`cargo gate` / `cargo full-gate` (`= gate --build`) is an EXTERNAL `cargo-gate`
tool on PATH; whether its internal test phase runs `cargo nextest` or threaded
`cargo test` is NOT visible from the repo. The owner's PASTED failing run was
clearly THREADED `cargo test` (`test result: FAILED ... --bin ffwb`). Therefore:
- IF the owner's authoritative `cargo gate` uses NEXTEST: these four are already
  green under it and CR-CH-071 is mergeable (this is the expected case, given the
  nextest config + steering).
- IF the owner's `cargo gate` uses THREADED `cargo test`: Option B does NOT
  unblock the merge on its own -- the gate must either switch its test phase to
  nextest, or the Option A hardening below must land first. This needs the
  owner's confirmation of the gate's actual runner.

### FOLLOW-UP Option A test-isolation hardening -- LANDED on this branch (commit a7f5472)

The latent defect described here (catalogue-dependent tests unsafe under a
threaded runner because `ff_i18n::set_catalogue` is a process-global) was
addressed on this branch by commit **a7f5472** `fix(i18n): per-thread catalogue
override for test isolation`, with the lessons entry in **b0997df**. The chosen
mechanism is a per-thread catalogue override in `ff-i18n` (seam + lib) consumed
by the test bootstrap (`i18n_bootstrap.rs`, `tests_common.rs`) and the readers
(`alias_reader.rs`, `environment.rs`), so catalogue-dependent tests no longer
clobber each other's process-global state under threaded `cargo test`. This is
test-infra/i18n-seam work added AFTER the CR-CH-071 behavioural review of
`9f3676d`; it does not alter the ISPF Save/End/Cancel/Return/Refresh behaviour
that was reviewed and approved. The owner's full nextest gate remains the
authoritative pre-merge run for the whole branch.

## FINAL VERDICT -- APPROVED (owner's authoritative gate uses nextest)

The owner confirmed the authoritative `cargo gate` runs with the NEXTEST runner
(its banner reads `cargo-gate: mode=FULL runner=nextest scope=workspace`, seen on
the CR-CH-068 / 069 / 070 gate runs). The failing run that triggered this fix
iteration was a DIFFERENT command -- a plain threaded `cargo test -p ff-desktop`
(output `test result: FAILED ... --bin ffwb`, not the cargo-gate nextest banner).
Under the authoritative nextest harness the four tests are GREEN (verified 4/4
here). CR-CH-071 is therefore VERIFIED-COMPLETE. `crch071-review.json`
verdict = APPROVED.

Recorded conclusions:
1. The four tests (`af_poc_alias_catalogue_loads_and_resolves`,
   `full_shell_menu_option_shows_localised_surface_and_dispatches_root`,
   `full_shell_per_part_fallback_never_blank`,
   `full_shell_single_menu_auto_converts_between_locales`) were a PRE-EXISTING
   process-global `ff_i18n::set_catalogue` test-isolation race, exposed ONLY under
   threaded `cargo test`, GREEN under the canonical nextest gate. `af_poc` passes
   with `--exact`. NO production or test code change was needed -- not a CR-CH-071
   behavioural regression.
2. `full_shell_theme_editor_type_name_and_save_creates_user_theme` is the known
   out-of-scope B081 theme-editor flake (noted, not chased).
3. Option A (a process-wide catalogue test-lock to make the catalogue-dependent
   tests safe under ANY threaded runner) is a recommended SEPARATE follow-up CR,
   NOT done here.
4. Sibling editors (command-environments tasks 34.1-34.5) are deliberate
   fast-follow, not in v1.

## Commits (branch range 42d6db8..HEAD, HEAD = b0997df)

Three commits on `crch071-save-cancel-refresh`, confirmed at finalize time
(`git --no-pager log --oneline 42d6db8..HEAD`, log read back from
`tools/logs/crch071-finalize-check.txt`):

- **9f3676d** -- `feat(cmdenv): ISPF Save/End/Cancel/Return/Refresh for editable
  Contexts (CR-CH-071 Req 20a)` -- 9 files changed, 767 insertions(+), 39
  deletions(-). THIS is the commit the reviewer reviewed. Staged BY NAME:
  `command_config/mod.rs`, `shell/{alias_editor.rs, alias_editor_line_commands.rs,
  commands_ladder_a.rs, environment_registry.rs, mod.rs, editable_context.rs,
  tests_crch071.rs}`, `docs/specs/command-environments/tasks.md`.
- **a7f5472** -- `fix(i18n): per-thread catalogue override for test isolation` --
  6 files changed (`shell/alias_reader.rs`, `shell/environment.rs`,
  `shell/i18n_bootstrap.rs`, `shell/tests_common.rs`, `ff-i18n/src/lib.rs`,
  `ff-i18n/src/seam.rs`). This is the Option A test-isolation hardening that the
  follow-up section below originally recommended as a separate CR -- it was
  landed on THIS branch to make the catalogue-dependent tests safe under a
  threaded runner (per-thread catalogue override) rather than relying solely on
  nextest's process-per-test isolation.
- **b0997df** -- `docs(lessons): test-isolation fix taxonomy + production/
  multi-instance boundary (A1, C3)` -- 1 file changed
  (`docs/PATTERNS-AND-LESSONS.md`, +84). Lessons-log entry for the above.

`tools/logs/` and `TERMINAL_DEFECTS.md` were NOT committed (status at finalize:
only `?? TERMINAL_DEFECTS.md` untracked; working tree otherwise clean). The
owner/orchestrator runs the full nextest `cargo gate` as the final pre-merge
confirmation and merges `crch071-save-cancel-refresh` into main (this step does
NOT merge/push).

NOTE on review scope: `crch071-review.json`/`.md` reviewed range
`42d6db8..HEAD` when HEAD was the single commit `9f3676d` and returned verdict
APPROVED with zero findings. The two later commits (`a7f5472`, `b0997df`) were
added after that review; they are the test-isolation hardening + its lessons
entry (test-infra and docs only -- no change to the CR-CH-071 behavioural
surface the review approved).

REMINDER: three stale worktrees (`eqfiles-fix`, `alias-ghost-text`,
`cmdenv-root-enum`) still need owner cleanup.

## Post-approval confirmation -- FULL ff-desktop suite GREEN under nextest

A later threaded `cargo test -p ff-desktop` run reported a DIFFERENT set of four
failures (`display_surface_resolves_localisation_key_else_raw_surface`,
`overlay_row_matching_identity_surface_updates_without_collision`,
`full_shell_menu_option_localises_mnemonic_command_and_description`,
`full_shell_per_part_fallback_never_blank`) than the first run. The failing set
CHANGING between runs of the same unchanged committed code (9f3676d) is the
definitive signature of a shared-global-state race, not a regression (a real
regression fails the same tests every time). All four are catalogue-dependent
localisation tests sharing the process-global `ff_i18n::set_catalogue`.

Verified under the authoritative harness (logs read back):
- `cargo nextest run -p ff-desktop` (the new failing four) ->
  `crch071-nextest-newset.log`: 4 tests run, 4 PASSED.
- `cargo nextest run -p ff-desktop` (WHOLE ff-desktop suite) ->
  `crch071-nextest-full.log`: **Summary [116.0s] 1162 tests run: 1162 passed, 0
  skipped** -- the entire suite is deterministically GREEN under process-per-test
  nextest, including every locale/af/theme-editor test that flakes under threaded
  `cargo test`.

Conclusion unchanged: CR-CH-071 is verified-complete under the authoritative
nextest gate; the threaded-`cargo test` failures are a pre-existing shared-global
test-isolation artifact (Option A follow-up CR), NOT a CR-CH-071 regression. No
code change was made. Note: a bare `cargo fmt` (not `--check`) run in the worktree
produced NO changes (the committed code was already rustfmt-clean; working tree
clean except the untracked `TERMINAL_DEFECTS.md`).
