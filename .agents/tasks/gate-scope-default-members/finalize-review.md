# core-001 Finalize Review -- Resolution of the 26 Failing ff-desktop Tests

Reviewer pass over the uncommitted diff on branch `feat/true-core-default-members`
(worktree `c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members`).
ASCII only. All paths absolute. Verification evidence READ, not re-run (per the
workspace execution-split / gate-policy discipline).

VERDICT: APPROVED. No blocking findings.

---

## 1. Scope of the diff reviewed

`git status` (read from tools\logs\review-status.txt) shows five modified files,
all either test code or a test-module const block:

- crates/ff-desktop/src/shell/tests_command.rs   (removed parked FFEDIT tests)
- crates/ff-desktop/src/shell/tests_misc.rs       (removed parked FFEDIT profile tests)
- crates/ff-desktop/src/shell/tests_session.rs    (removed parked editor-restore tests)
- crates/ff-desktop/src/tab_manager.rs            (removed parked file-open test)
- crates/ff-volume/src/service.rs                 (clippy const-block fix, in #[cfg(test)])

NO production dispatch/env/render source was modified by this diff. The parked
production stubs (`open_file` -> Err, `is_ffedit_active` -> false,
`dispatch_to_environment` FFEDIT -> NoSuchEnvironment) were landed in prior
commits and are unchanged here. This diff is purely the TEST reconciliation plus
the one ff-volume test-const clippy fix.

Consequence: the "no bespoke dispatch intercept added", "parked arms use the
single focus-latch path with InteriorFocus::none()", "no new unwrap in lib code"
and "no non-test source file exceeds 400 lines" checks are satisfied by
construction -- no production control-flow code was touched, and the only
production-file edit (ff-volume/service.rs) is a 224-line file whose change lives
entirely inside its `#[cfg(test)]` module.

---

## 2. The single most important check -- removed tests validate PARKED behaviour only

Boundary (true-core-proposal.md s1/s2/s4 + finalize-fix-plan.md s1): PARKED =
Editor Context, FFEDIT claiming command environment, editor SAVE seam,
FileEditor/Untitled file-open, FFEDIT editor-profile verbs (CAPS/NULLS/STATS/
HILITE/AUTONUM/NUM/LOCK/PROFILE/EXCLUDE/SAVE), editor session restore via
`WorkspaceKind::Editor`. RETAINED = FFCMD base command environment + unified
dispatch spine, Theme Editor Context, `edit_profile` as a bare TabState field.

Every removed function was confirmed against the diff to validate parked
behaviour, and every removal carries a strip comment citing
docs/reentry-register.md + true-core-proposal.md s1/s2/s4:

tests_command.rs (removed):
- caps_on / nulls_on / profile / stats_on command tests -- FFEDIT edit-profile
  verbs; no longer claimed (is_ffedit_active=false). PARKED.
- autonum_on / num -- open a file via shell_open_file().expect(), which now
  returns the parked Err; both are FFEDIT verbs (CR-CH-069). PARKED.
- ffedit_verb_via_registry_matches_prior_behaviour -- exercises the FfEdit
  registry object (EXCLUDE ALL, CAPS ON). PARKED.
- editor_save_on_clean/dirty_untitled -- FFEDIT SAVE verb (E9). PARKED.
- save_routes_through_dispatch_to_environment / host_path_open /
  open_with_origin / host_bound_save / ffedit_save_routes_dirty_host_file --
  editor SAVE seam + Owning_Environment binding reached via file-open-into-editor
  (shell_open_file now Err). PARKED.

tests_misc.rs (removed): caps_no_arg / profile_caps_on / lock_on / lock_off /
hilite_on / hilite_logic / hilite_unknown -- FFEDIT editor-profile verbs. PARKED.

tests_session.rs (removed): restore_editor_descriptor_opens_file and
reopened_editor_descriptor_recaptures_owning_environment -- both depend on a
WorkspaceKind::Editor descriptor reopening a FileEditor tab, which the parked
open_file can no longer do. PARKED. (The production WorkspaceKind::Editor variant
and its reconstruct arm are correctly left in place; only the now-impossible
reopen assertions were removed.)

tab_manager.rs (removed): file_tab_has_kind_file_editor -- open_file().expect()
into a FileEditor tab, now parked. PARKED.

No removed test validated retained behaviour.

---

## 3. The four FFCMD base-dispatch tests (special scrutiny)

The brief required that the FFCMD base command environment and the unified
dispatch spine STAY -- so the four named tests should have been FIXED (or had
only their FFEDIT fixture trimmed while keeping the FFCMD assertions), not
wholesale deleted if they carried a retained FFCMD assertion. I read each removed
function body in the diff and the retained siblings in source:

- dispatch_bound_command_falls_through_for_builtin: its SOLE assertion was that a
  bound `CAPS ON` set `edit_profile.caps == On` -- i.e. the parked FFEDIT
  edit-profile outcome. There was no separable FFCMD-base assertion to keep (the
  test's only observable was the parked verb's effect). The `dispatch_bound_command`
  spine itself remains covered by `dispatch_bound_command_resolves_user_definition`
  (source line 711, confirmed present: resolves `kb.exit` -> `file.exit` through
  the pipeline without a not-found error). Removal is justified; the retained
  bound-dispatch branch is still proven.
- dispatch_to_environment_addressing_active_env_claims,
  dispatch_to_environment_name_is_case_insensitive,
  dispatch_to_environment_declines_unowned_verb: all three opened an editor/
  Untitled Context and addressed the name "FFEDIT"/"ffedit" by name, asserting a
  Claimed/NotClaimed outcome of the FFEDIT environment. With FFEDIT parked,
  addressing "FFEDIT" now returns NoSuchEnvironment, so these assert parked
  behaviour. The RETAINED FFCMD-base half of this seam is proven by the
  still-present `dispatch_to_environment_base_declines_and_unknown_name_reports`
  (source line 1657, confirmed): it asserts `dispatch_to_environment("FFCMD", ..)`
  -> NotClaimed and an unknown name -> NoSuchEnvironment. That sibling was RUN in
  isolation and PASSES (tools\logs\test-dispatch-env.log: "1 passed; 0 failed").

Conclusion: the dispatch spine and FFCMD base environment are retained and
covered; the four removals excised only FFEDIT-specific assertions. Also present
and retained: `ffcmd_verb_unaffected_by_registry` and
`non_editor_context_does_not_route_through_ffedit` (both read in source),
covering the retained FFCMD side of the registry gate.

---

## 4. Theme Editor test (RETAINED core Context)

full_shell_theme_editor_type_name_and_save_creates_user_theme was NOT removed and
NOT weakened -- tests_focus.rs is absent from the diff. It was classified as the
known B081 isolation flake and PROVEN green in isolation:
tools\logs\theme-editor-iso.log records a single-test `--test-threads=1` run:
"test ... full_shell_theme_editor_type_name_and_save_creates_user_theme ... ok /
test result: ok. 1 passed; 0 failed". This satisfies the requirement that the
retained Theme Editor test be fixed-or-justified-with-the-in-isolation-result,
never simply deleted. (The owner's full gate may still see it flake under
parallelism; that is a known out-of-scope flake per gate-policy.md, re-run green
in isolation -- correctly flagged in the fix plan item 9.)

---

## 5. Structural / framework-conformance checks

- No bespoke dispatch intercept outside the single spine: no production dispatch
  code changed in the diff. PASS.
- Parked Context arms on the single focus-latch path (InteriorFocus::none()):
  unchanged by this diff (prior commits); not regressed here. PASS.
- No non-test source file > 400 lines: only ff-volume/service.rs touched (224
  lines total; change inside #[cfg(test)]). PASS.
- No new unwrap in lib code: the diff adds none (test code + a const block only).
  PASS.
- No dangling empty test module: every touched test file retains many live tests.
  Effect-confirmed -- the ff-desktop test binary COMPILED and RAN for both the
  theme-editor-iso and test-dispatch-env isolated runs (each shows "Finished test
  profile" + "Running unittests" + a result), which is impossible with a broken
  or empty test module. PASS.

---

## 6. ff-volume const-block clippy fix

crates/ff-volume/src/service.rs lines 217-220, inside
define_volume_command_contract_is_data_only, now wraps the four
`assert!(...CONTRACT...)` in `const { ... }`:

```
const {
    assert!(DefineVolumeParams::CONTRACT.volser_required);
    assert!(DefineVolumeParams::CONTRACT.accepts_capacity);
    assert!(VtocParams::CONTRACT.volser_required);
    assert!(VolumeListingParams::CONTRACT.accepts_volser_filter);
}
```

This is exactly the clippy-suggested remedy for `assertions_on_constants` and is
behaviour-preserving (compile-time assertion of the same const data; no assertion
weakened or removed). tools\logs\clippy-ff-volume.log shows a clean "Finished"
with no assertions_on_constants error. PASS.

---

## 7. Verification evidence (read, not re-run)

Per gate-policy.md / testing.md execution-split, the agent's definition of done
is the SCOPED compile-verify trio + targeted scoped tests; the whole-suite run is
the owner's manual gate. The recorded evidence is present and passing:

- tools\logs\check-ff-desktop.log  -- cargo check -p ff-desktop: Finished, clean.
- tools\logs\clippy-ff-desktop.log -- cargo clippy -p ff-desktop: "Checking
  ff-desktop ... Finished", no warnings/errors under -D warnings.
- tools\logs\clippy-ff-volume.log  -- cargo clippy -p ff-volume: Finished, clean
  (const-block fix resolved the four assertions_on_constants errors).
- tools\logs\theme-editor-iso.log  -- retained Theme Editor test: 1 passed.
- tools\logs\test-dispatch-env.log -- retained FFCMD base-dispatch sibling: 1
  passed.

The isolated runs report "1026 filtered out", consistent with the pre-strip
1032+26 total shrinking by the removed tests; the ff-desktop test binary builds
and runs. No required verification is missing for an agent step.

Note (not a blocker): a single consolidated full `cargo test -p ff-desktop --bin
ffwb` log (fix-plan item 8) is not among the session logs; under the workspace
execution-split that whole-bin run is the owner's authoritative gate, and the
scoped compile-verify + the two targeted retained-test isolation runs are the
mandated agent evidence, which is present and green.

---

## 8. Verdict

APPROVED. Every removed test validates intentionally-parked behaviour and cites
the strip; the four FFCMD base-dispatch removals excised only FFEDIT-specific
assertions while the retained base spine stays covered and green; the retained
Theme Editor test was kept and justified with its in-isolation PASS; no assertion
was weakened; no production control-flow changed; the ff-volume const-block fix
is correct and behaviour-preserving; required scoped verification evidence is
present and passing.
