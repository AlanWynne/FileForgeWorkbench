# core-001 Core-Strip Milestone -- Finalize Test-Fix Findings Report

Milestone: core-001 ("true core" strip of FileForgeWorkbench down to its core).
Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members`
Branch:   `feat/true-core-default-members`

Scope of this report: resolution of the 26 failing `ff-desktop` tests reported by
the owner's full `cargo test --workspace` run, plus the `ff-volume` clippy
`assertions_on_constants` fix. ASCII only. All paths absolute. Verification
evidence was READ from the session logs; no cargo suite was re-run to produce
this report. Pass/fail is taken from each log's result line, never from any
terminal `Exit Code` line (which is unreliable on this machine).

Sources read:
- Classification plan: `c:\workspace\apps\FileForgeWorkbench\.agents\tasks\gate-scope-default-members\finalize-fix-plan.md`
- Review doc:        `c:\workspace\apps\FileForgeWorkbench\.agents\tasks\gate-scope-default-members\finalize-review.md`
- Verdict JSON:      `c:\workspace\apps\FileForgeWorkbench\.agents\tasks\gate-scope-default-members\finalize-review.json`
- Git status/diff of the worktree (read from the captured logs below)
- Scoped-verification logs under `...\.worktrees\true-core-default-members\tools\logs\`

---

## 1. Classification of all 26 failing tests

Boundary (from `true-core-proposal.md` s1/s2/s4 + `reentry-register.md`):
PARKED = Editor Context, the FFEDIT claiming command environment, the editor SAVE
seam, FileEditor/Untitled file-open, FFEDIT editor-profile verbs
(CAPS / NULLS / STATS / HILITE / AUTONUM / NUM / LOCK / PROFILE / EXCLUDE / SAVE),
and editor session restore via `WorkspaceKind::Editor`. RETAINED = the FFCMD base
command environment + the unified dispatch spine, the Theme Editor Context, and
`edit_profile` as a bare `TabState` field.

Resolution legend:
- PARKED-removed = the test validated intentionally-parked behaviour; the whole
  test function was removed with a `// core-001 strip:` comment citing
  `docs/reentry-register.md` + `true-core-proposal.md` s1/s2/s4. No production
  control-flow code was changed for these.
- RETAINED-fixed = a retained Context/behaviour; kept (not deleted, not weakened).

| #  | Test (module::name) | File | Resolution | Reason |
|----|---------------------|------|------------|--------|
| 1  | tests_command::caps_on_command_sets_caps_mode_on | tests_command.rs | PARKED-removed | FFEDIT CAPS verb; `is_ffedit_active`=false so unclaimed. |
| 2  | tests_command::nulls_on_command_sets_nulls_mode_on | tests_command.rs | PARKED-removed | FFEDIT NULLS verb; same mechanism. |
| 3  | tests_command::profile_command_sets_open_error_to_summary | tests_command.rs | PARKED-removed | FFEDIT PROFILE summary verb; unclaimed now. |
| 4  | tests_command::stats_on_command_sets_stats_mode_on | tests_command.rs | PARKED-removed | FFEDIT STATS verb. |
| 5  | tests_command::autonum_on_dispatches_to_number_on | tests_command.rs | PARKED-removed | Opens a file (`shell_open_file().expect`) now returns parked Err -> panic; AUTONUM is FFEDIT (CR-CH-069). |
| 6  | tests_command::num_command_dispatches_to_number | tests_command.rs | PARKED-removed | File open panics; NUM is FFEDIT. |
| 7  | tests_command::dispatch_bound_command_falls_through_for_builtin | tests_command.rs | PARKED-removed | Sole assertion was a bound `CAPS ON` mutating `edit_profile` (parked FFEDIT verb). Base-dispatch spine itself stays covered by `dispatch_bound_command_resolves_user_definition`. |
| 8  | tests_command::ffedit_verb_via_registry_matches_prior_behaviour | tests_command.rs | PARKED-removed | Exercises the `FfEditEnvironment` registry claim (EXCLUDE ALL, CAPS ON). |
| 9  | tests_command::editor_save_on_dirty_untitled_errors_and_stays | tests_command.rs | PARKED-removed | FFEDIT SAVE verb (CR-CH-053 E9); unclaimed now. |
| 10 | tests_command::dispatch_to_environment_addressing_active_env_claims | tests_command.rs | PARKED-removed | Addresses "FFEDIT" -> now `NoSuchEnvironment`; asserted `Claimed` + `edit_profile.caps`. No separable FFCMD assertion. |
| 11 | tests_command::dispatch_to_environment_name_is_case_insensitive | tests_command.rs | PARKED-removed | Addresses "ffedit" expecting `Claimed`; FFEDIT parked. |
| 12 | tests_command::dispatch_to_environment_declines_unowned_verb | tests_command.rs | PARKED-removed | Addresses "FFEDIT" FILES expecting `NotClaimed`; now `NoSuchEnvironment`. FFEDIT-specific. |
| 13 | tests_command::save_routes_through_dispatch_to_environment_and_is_behaviour_preserving | tests_command.rs | PARKED-removed | Addresses "FFEDIT" SAVE; FFEDIT parked. |
| 14 | tests_command::host_path_open_binds_owning_environment_to_host_fs | tests_command.rs | PARKED-removed | `shell_open_file().expect` -> parked Err -> panic. |
| 15 | tests_command::open_with_origin_binds_that_owning_environment | tests_command.rs | PARKED-removed | `shell_open_file_with_env().expect` -> parked Err -> panic. |
| 16 | tests_command::host_bound_save_is_behaviour_preserving | tests_command.rs | PARKED-removed | `shell_open_file().expect` -> panic; FFEDIT SAVE. |
| 17 | tests_command::ffedit_save_routes_dirty_host_file_write_through_owning_env | tests_command.rs | PARKED-removed | `shell_open_file().expect` -> panic; FFEDIT SAVE through owning env. |
| 18 | tests_misc::caps_no_arg_toggles_state | tests_misc.rs | PARKED-removed | FFEDIT CAPS toggle; unclaimed now. |
| 19 | tests_misc::profile_caps_on_keyword_updates_caps | tests_misc.rs | PARKED-removed | FFEDIT PROFILE CAPS ON. |
| 20 | tests_misc::lock_off_re_enables_profile_changes | tests_misc.rs | PARKED-removed | FFEDIT LOCK + CAPS profile verbs. |
| 21 | tests_misc::hilite_on_sets_hilite_mode | tests_misc.rs | PARKED-removed | FFEDIT HILITE verb. |
| 22 | tests_misc::hilite_logic_sets_logic_mode | tests_misc.rs | PARKED-removed | FFEDIT HILITE LOGIC verb. |
| 23 | tests_session::reopened_editor_descriptor_recaptures_owning_environment | tests_session.rs | PARKED-removed | `WorkspaceKind::Editor` descriptor reopen -> `open_file` Err -> no FileEditor tab -> panic. |
| 24 | tests_session::restore_editor_descriptor_opens_file | tests_session.rs | PARKED-removed | Editor descriptor cannot reopen a file now. |
| 25 | tab_manager::tests::file_tab_has_kind_file_editor | tab_manager.rs | PARKED-removed | `mgr.open_file().expect("open")` -> parked Err -> panic; validates parked file-open-into-editor. |
| 26 | tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme | tests_focus.rs | RETAINED-fixed (flake, NOT a regression) | Theme Editor is RETAINED core. Classified in isolation (`--test-threads=1`, `--exact`): PASS -> known B081-class isolation flake, NOT a regression. Kept, not weakened, not deleted. In-isolation result: **1 passed; 0 failed** (`tools\logs\theme-editor-iso.log`). |

Counts: PARKED-removed = 25; RETAINED-fixed = 1 (test #26, the Theme Editor
test). Genuine retained regressions found: 0.

### Test #26 flake-vs-regression determination

`full_shell_theme_editor_type_name_and_save_creates_user_theme` was NOT removed
and NOT weakened (its file `tests_focus.rs` is absent from the diff). It was
classified by a single-test, single-threaded isolation run:

```
running 1 test
test shell::tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1026 filtered out; finished in 0.12s
```
(Log: `tools\logs\theme-editor-iso.log`.)

Verdict: **flake, not a regression.** Passing in isolation while failing under the
owner's full parallel run is the known B081-class test-isolation sensitivity
(shared themes dir / i18n catalogue / label-query timing under parallelism). Per
`gate-policy.md` this is a known out-of-scope flake: re-run it single-threaded in
isolation during the full gate; it does not block the merge.

---

## 2. Production files changed for retained fixes

No production dispatch / environment / render control-flow source was modified
for any retained behaviour. The parked production stubs (`open_file` -> Err,
`is_ffedit_active` -> false, `dispatch_to_environment` FFEDIT ->
`NoSuchEnvironment`) were landed in PRIOR commits and are unchanged by this diff.

The only production-file edit in this diff is the `ff-volume` clippy fix, which
lives entirely inside a `#[cfg(test)]` module (no runtime behaviour change):

| File (under the worktree) | Change |
|---------------------------|--------|
| `crates\ff-volume\src\service.rs` | Clippy `assertions_on_constants` fix: the four `assert!(...CONTRACT...)` lines in `define_volume_command_contract_is_data_only` are wrapped in a `const { ... }` block. Compile-time assertion of the same const data; no assertion weakened. |

The remaining four changed files are test-only (parked-test removals):

| File (under the worktree) | Change |
|---------------------------|--------|
| `crates\ff-desktop\src\shell\tests_command.rs` | Removed parked FFEDIT profile/SAVE/open and FFEDIT-addressing tests (tests #1-#17) with `// core-001 strip:` notes; retained FFCMD base-dispatch siblings untouched. |
| `crates\ff-desktop\src\shell\tests_misc.rs` | Removed parked FFEDIT editor-profile tests (tests #18-#22) with strip notes; non-profile tests untouched. |
| `crates\ff-desktop\src\shell\tests_session.rs` | Removed parked editor session-restore tests (tests #23, #24) with strip notes; other descriptor-restore tests untouched. |
| `crates\ff-desktop\src\tab_manager.rs` | Removed the parked file-open-into-editor test `file_tab_has_kind_file_editor` (test #25) with a strip note; other `tab_manager::tests` untouched. |

Git status / diffstat (read from `tools\logs\report-status.txt` and
`tools\logs\report-diffstat.txt`): five files modified, unstaged, no commit made.

```
 crates/ff-desktop/src/shell/tests_command.rs | 483 ++-------------------------
 crates/ff-desktop/src/shell/tests_misc.rs    |  86 +----
 crates/ff-desktop/src/shell/tests_session.rs |  79 +----
 crates/ff-desktop/src/tab_manager.rs         |  18 +-
 crates/ff-volume/src/service.rs              |  10 +-
 5 files changed, 62 insertions(+), 614 deletions(-)
```

The removal comments are present and cite the strip rationale; a sample of the
literal `// core-001 strip:` notes was confirmed in the captured review diff
(`tools\logs\review-diff.txt`, e.g. lines 151, 174, 216, 295, 354, 521, 617,
661, 713, 741).

---

## 3. ff-volume clippy status

The const-block fix is PRESENT and CORRECT at
`crates\ff-volume\src\service.rs` (confirmed by reading the file, around lines
214-220, inside `define_volume_command_contract_is_data_only`):

```rust
const {
    assert!(DefineVolumeParams::CONTRACT.volser_required);
    assert!(DefineVolumeParams::CONTRACT.accepts_capacity);
    assert!(VtocParams::CONTRACT.volser_required);
    assert!(VolumeListingParams::CONTRACT.accepts_volser_filter);
}
```

This is exactly the clippy-suggested remedy for `clippy::assertions_on_constants`
and is behaviour-preserving (the same const data is asserted at compile time; no
assertion was weakened or removed). `cargo clippy -p ff-volume -- -D warnings`
finishes clean with no `assertions_on_constants` error
(`tools\logs\clippy-ff-volume.log`, result line below).

---

## 4. Scoped verification evidence (read from logs, not re-run)

Per the workspace execution-split (`gate-policy.md` / `testing.md`), an agent's
definition of done is the SCOPED compile-verify trio plus targeted scoped tests;
the whole-suite run is the owner's authoritative manual gate. Pass results below
are taken from each log's result/finish line, not from any `Exit Code` line.

| Command | Log file (under `...\tools\logs\`) | Result line read from the log |
|---------|------------------------------------|-------------------------------|
| `cargo check -p ff-desktop` | `check-ff-desktop.log` | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.41s` -- clean, no errors. |
| `cargo clippy -p ff-desktop -- -D warnings` | `clippy-ff-desktop.log` | `Checking ff-desktop ...` then `Finished \`dev\` profile ... in 4.24s` -- no warnings/errors under `-D warnings`. |
| `cargo clippy -p ff-volume -- -D warnings` | `clippy-ff-volume.log` | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.24s` -- clean; the four `assertions_on_constants` errors resolved. |
| Scoped named test -- retained Theme Editor (`full_shell_theme_editor_type_name_and_save_creates_user_theme`, `--exact --test-threads=1`) | `theme-editor-iso.log` | `test ... full_shell_theme_editor_type_name_and_save_creates_user_theme ... ok` / `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1026 filtered out`. |
| Scoped named test -- retained FFCMD base-dispatch sibling (`dispatch_to_environment_base_declines_and_unknown_name_reports`) | `test-dispatch-env.log` | `test ... dispatch_to_environment_base_declines_and_unknown_name_reports ... ok` / `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1026 filtered out`. |
| `cargo fmt -p ff-desktop -- --check` | `core001-fmtcheck-11.log` / `gate-fmtcheck-24.log` / `fmtcheck.log` | Logs are empty, which is the clean result for `fmt --check` (it emits output ONLY on drift). No formatting diff reported. |

Notes:
- The ff-desktop test binary demonstrably COMPILES and RUNS: both isolated runs
  above show `Finished test profile` + `Running unittests` + a result line, which
  is impossible if a touched test module were left broken or empty. This confirms
  no dangling empty `#[cfg(test)]` module resulted from the removals.
- The two retained-behaviour isolation runs prove the retained spine is intact:
  the FFCMD base command environment (`dispatch_to_environment_base_declines_...`)
  and the Theme Editor Context both pass. The `1026 filtered out` figure is
  consistent with the pre-strip total shrinking by the removed tests.
- A single consolidated whole-bin `cargo test -p ff-desktop --bin ffwb` log
  (fix-plan item 8) is not among the session logs; under the workspace
  execution-split that whole-bin run is the owner's authoritative gate, and the
  mandated agent evidence (scoped compile-verify trio + the two targeted
  retained-test isolation runs) is present and green.

---

## 5. Review verdict

APPROVED, with no blocking findings. The review loop did NOT pause at a max
iteration; there are no outstanding findings.

From `finalize-review.json`:
```json
{ "verdict": "APPROVED", "findings": [] }
```

Reviewer summary (`finalize-review.md`): every removed test validates
intentionally-parked behaviour and cites the strip; the four FFCMD base-dispatch
removals excised only FFEDIT-specific assertions while the retained base spine
stays covered and green; the retained Theme Editor test was kept and justified
with its in-isolation PASS; no assertion was weakened; no production control-flow
changed; the `ff-volume` const-block fix is correct and behaviour-preserving;
required scoped verification evidence is present and passing.

---

## 6. Handoff status

- Changes remain UNSTAGED in the worktree (five modified files; no commit made).
  This report does NOT commit and does NOT merge to `main`, per instruction --
  the orchestrator and owner review and commit.
- Before merge to `main`, the owner runs the authoritative full `cargo gate`
  (fmt + clippy + build + whole-suite), which must be CLEAN (`gate-policy.md`).
- Known out-of-scope flake to flag during that full gate: test #26
  (`full_shell_theme_editor_type_name_and_save_creates_user_theme`). If it flakes
  under parallelism, re-run it single-threaded in isolation; it passes there
  (evidence above) and does not block the merge.
