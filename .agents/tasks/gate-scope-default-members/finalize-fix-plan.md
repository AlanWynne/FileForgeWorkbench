# core-001 Finalize Fix Plan -- Classify and Resolve the 26 Failing Tests

ASCII only. All paths absolute, under the worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members`
(branch `feat/true-core-default-members`). This plan is INVESTIGATION + PLANNING
output; no source was edited in the step that produced it.

CRITICAL execution note (carry into every implementer command): on this machine
the terminal/proxy sometimes ignores `cwd` and runs from the MAIN checkout
`c:\workspace\apps\FileForgeWorkbench`, which has OLD un-stripped code. ALWAYS
target the worktree explicitly with
`--manifest-path 'c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members\Cargo.toml'`
on every cargo command, OR verify cwd is the worktree first. Verify by EFFECT
(read a log), never trust the `Exit Code` line.

---

## 1. Boundary established from the three docs

From `true-core-proposal.md` (sections 1, 2, 4) + `reentry-register.md`:

- PARKED: the Editor Context, the FFEDIT claiming command environment, the
  editor SAVE seam (host-FS / mainframe `ff_vfs::BackendEnvironment` backend),
  FileEditor/Untitled file-open, editor profile verbs (CAPS / NULLS / STATS /
  HILITE / AUTONUM / NUM / LOCK / PROFILE / EXCLUDE / SAVE) which are FFEDIT
  verbs, and editor session restore via the `WorkspaceKind::Editor` descriptor.
- RETAINED: the FFCMD base command environment and the unified dispatch spine
  (`resolve_target` -> `dispatch_command_target`, `dispatch_to_environment`'s
  base path, the active-wins claim gate structure); the Theme Editor Context;
  `edit_profile` as a bare TabState field (Option A TabState infrastructure);
  `ff-document-model` / `ff-viewport-scrolling` / `ff-edit-operations` / `ff-vfs`.

Production state verified by reading the stubbed code:

- `crates\ff-desktop\src\tab_manager.rs::open_file` (around line 1227) now
  RETURNS `Err("Opening files is not available in the core build (editor
  parked).")`. Any test that opens a file via `shell_open_file` /
  `TabManager::open_file` and `.expect("open")`s it now PANICS.
- `crates\ff-desktop\src\shell\environment_registry.rs::is_ffedit_active`
  (around line 208) always returns `false` (FFEDIT parked). The active-wins
  claim gate in `crates\ff-desktop\src\shell\commands.rs` (lines ~82-84 and
  ~160-178) is structurally intact but NEVER fires, so FFEDIT verbs (CAPS, SAVE,
  EXCLUDE, ...) fall through to the FFCMD ladder/engine and set an unknown
  command `open_error` instead of mutating `edit_profile`.
- `crates\ff-desktop\src\shell\commands_environment.rs::dispatch_to_environment`
  (around line 33) resolves only `RegisteredEnv::FfCmdBase`; addressing
  `"FFEDIT"` now returns `EnvDispatchOutcome::NoSuchEnvironment`.
- These stubs are CORRECT per the strip (single focus-latch path,
  `InteriorFocus::none()`, FFCMD base retained, no bespoke intercepts). The
  failing tests validate the now-parked FFEDIT behaviour, so they are REMOVED;
  the production code is NOT changed for them.

CONCLUSION: 25 of 26 failing tests are PARKED-remove. 1 (theme editor) is a
retained Context and is classified in isolation (flake vs regression). ZERO
genuine RETAINED-regressions were found: the FFCMD base spine is intact, proven
by the still-PASSING sibling `dispatch_to_environment_base_declines_and_unknown_name_reports`.

---

## 2. Per-test classification table

Legend: PARKED = remove (validates intentionally-removed behaviour);
TBD-ISO = run single-threaded in isolation to classify flake vs regression.

| # | Test (module::name) | File | Class | Why |
|---|---------------------|------|-------|-----|
| 1 | tests_command::caps_on_command_sets_caps_mode_on | tests_command.rs:360 | PARKED | FFEDIT CAPS verb; `is_ffedit_active`=false so not claimed; asserts `edit_profile.caps==On` + `open_error.is_none()`. |
| 2 | tests_command::nulls_on_command_sets_nulls_mode_on | tests_command.rs:382 | PARKED | FFEDIT NULLS verb; same mechanism. |
| 3 | tests_command::profile_command_sets_open_error_to_summary | tests_command.rs:404 | PARKED | FFEDIT PROFILE summary verb; not claimed now. |
| 4 | tests_command::stats_on_command_sets_stats_mode_on | tests_command.rs:421 | PARKED | FFEDIT STATS verb. |
| 5 | tests_command::autonum_on_dispatches_to_number_on | tests_command.rs:449 | PARKED | Opens a file (`shell_open_file().expect`) -> now Err -> panic; AUTONUM is an FFEDIT verb (CR-CH-069). |
| 6 | tests_command::num_command_dispatches_to_number | tests_command.rs:478 | PARKED | Same: file open panics; NUM is FFEDIT. |
| 7 | tests_command::dispatch_bound_command_falls_through_for_builtin | tests_command.rs:862 | PARKED | Asserts bound `CAPS ON` sets `edit_profile.caps==On` -- CAPS is now a parked FFEDIT verb. NOT a base-dispatch regression (base path intact). |
| 8 | tests_command::ffedit_verb_via_registry_matches_prior_behaviour | tests_command.rs:1605 | PARKED | Explicitly exercises `FfEditEnvironment` claim (EXCLUDE ALL, CAPS ON). |
| 9 | tests_command::editor_save_on_dirty_untitled_errors_and_stays | tests_command.rs:1751 | PARKED | FFEDIT SAVE verb (E9); unclaimed now. |
| 10 | tests_command::dispatch_to_environment_addressing_active_env_claims | tests_command.rs:1875 | PARKED | Addresses `"FFEDIT"` -> `NoSuchEnvironment`; asserts `Claimed` + `edit_profile.caps`. No FFCMD assertion to keep. |
| 11 | tests_command::dispatch_to_environment_name_is_case_insensitive | tests_command.rs:1896 | PARKED | Addresses `"ffedit"` -> expects `Claimed`; FFEDIT parked. |
| 12 | tests_command::dispatch_to_environment_declines_unowned_verb | tests_command.rs:1909 | PARKED | Addresses `"FFEDIT"` FILES -> expects `NotClaimed`; now `NoSuchEnvironment`. FFEDIT-specific. |
| 13 | tests_command::save_routes_through_dispatch_to_environment_and_is_behaviour_preserving | tests_command.rs:1945 | PARKED | Addresses `"FFEDIT"` SAVE; FFEDIT parked. |
| 14 | tests_command::host_path_open_binds_owning_environment_to_host_fs | tests_command.rs:1967 | PARKED | `shell_open_file().expect` -> Err -> panic (editor open parked). |
| 15 | tests_command::open_with_origin_binds_that_owning_environment | tests_command.rs:1990 | PARKED | `shell_open_file_with_env().expect` -> Err -> panic. |
| 16 | tests_command::host_bound_save_is_behaviour_preserving | tests_command.rs:2012 | PARKED | `shell_open_file().expect` -> panic; FFEDIT SAVE. |
| 17 | tests_command::ffedit_save_routes_dirty_host_file_write_through_owning_env | tests_command.rs:2045 | PARKED | `shell_open_file().expect` -> panic; FFEDIT SAVE through owning env (uses `document`/`ff_document_model`). |
| 18 | tests_misc::caps_no_arg_toggles_state | tests_misc.rs:29 | PARKED | FFEDIT CAPS toggle on the welcome editor tab; unclaimed now. |
| 19 | tests_misc::profile_caps_on_keyword_updates_caps | tests_misc.rs:42 | PARKED | FFEDIT PROFILE CAPS ON. |
| 20 | tests_misc::lock_off_re_enables_profile_changes | tests_misc.rs:66 | PARKED | FFEDIT LOCK + CAPS profile verbs. |
| 21 | tests_misc::hilite_on_sets_hilite_mode | tests_misc.rs:78 | PARKED | FFEDIT HILITE verb. |
| 22 | tests_misc::hilite_logic_sets_logic_mode | tests_misc.rs:89 | PARKED | FFEDIT HILITE LOGIC verb. |
| 23 | tests_session::reopened_editor_descriptor_recaptures_owning_environment | tests_session.rs:692 | PARKED | `WorkspaceKind::Editor` descriptor reopen -> `open_file` Err -> no FileEditor tab -> `.expect` panic. |
| 24 | tests_session::restore_editor_descriptor_opens_file | tests_session.rs:471 | PARKED | Same: Editor descriptor cannot reopen a file now. |
| 25 | tab_manager::tests::file_tab_has_kind_file_editor | tab_manager.rs:1416 | PARKED | `mgr.open_file().expect("open")` -> Err -> panic; validates parked file-open-into-editor. |
| 26 | tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme | tests_focus.rs:1318 | TBD-ISO | Theme Editor is RETAINED core. Run single-threaded in isolation: PASS -> B081 isolation flake (keep, do not weaken); FAIL -> real regression (fix production render/action seam, do not delete). |

Counts: PARKED-remove = 25; TBD-in-isolation = 1; RETAINED-fix (confirmed
production regression up front) = 0.

---

## 3. Implementation plan (ordered, each item independently verifiable)

Verification uses the SCOPED trio + scoped tests per gate-policy.md; do NOT run
the whole-workspace suite in an agent step. Always pass the worktree
`--manifest-path`. The authoritative full gate is the owner's manual run.

- [ ] 1. Verify the ff-volume clippy fix is present and correct.
      It IS already applied: `crates\ff-volume\src\service.rs` wraps the four
      `assert!(... CONTRACT ...)` in a `const { ... }` block inside
      `define_volume_command_contract_is_data_only` (around lines 215-220),
      resolving `clippy::assertions_on_constants`. Confirm, do not re-edit unless
      the block is missing.
      Files: `crates\ff-volume\src\service.rs`
      Verify: run (from/with the worktree manifest)
      `cargo clippy --manifest-path 'c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members\Cargo.toml' -p ff-volume -- -D warnings`
      redirected to `tools\logs\clippy-ff-volume.log`; READ the log; expect no
      `assertions_on_constants` error and a clean finish. (If the `const {}`
      block is absent, re-apply exactly: wrap the four `assert!` lines in
      `const { ... }`.)

- [ ] 2. Classify the Theme Editor test in isolation (test #26) BEFORE deleting
      anything, since its resolution (keep vs fix) differs from the parked work.
      Files: none yet (diagnosis only).
      Verify: run single-threaded, exact:
      `cargo test --manifest-path 'c:\workspace\apps\FileForgeWorkbench\.worktrees\true-core-default-members\Cargo.toml' -p ff-desktop --bin ffwb full_shell_theme_editor_type_name_and_save_creates_user_theme -- --exact --test-threads=1`
      redirected to `tools\logs\theme-editor-iso.log`; READ the log.
      - If PASS in isolation -> B081-class test-isolation flake (shared themes
        dir / i18n catalogue / label-query timing under parallelism). Do NOT
        delete or weaken. Record the classification in the summary and leave the
        test as-is (the owner's full gate may still flake; note it as a known
        out-of-scope flake per gate-policy.md, re-run green in isolation).
      - If FAIL in isolation -> genuine regression in the RETAINED Theme Editor
        interactive render-to-action seam. FIX the production code (candidates:
        `crates\ff-desktop\src\theme_editor*` / the Theme Editor
        `WorkspaceContext` render + `apply_theme_editor_action`, and the Save
        button label/render path exercised by `harness.get_by_label("Save")`).
        Do NOT delete the test. This becomes a RETAINED-fix item; diagnose root
        cause from the isolation log before editing.

- [ ] 3. Remove the parked FFEDIT profile/SAVE/open tests in tests_command.rs
      (tests #1-#17). Delete the whole function for each failing test listed in
      the table, each with a one-line note. Because several of these sit in a
      block of sibling profile tests that currently PASS only COINCIDENTALLY
      (e.g. `caps_off_command_sets_caps_mode_off`, `nulls_off_command_sets_nulls_mode_off`,
      `stats_off_command_clears_stats_mode` pass because a fresh `edit_profile`
      already defaults to Off and they assert nothing about `open_error`), also
      remove those coincidental-pass FFEDIT-profile siblings in the same cluster
      so the suite does not retain misleading parked-behaviour tests. Leave the
      genuinely-FFCMD tests untouched (e.g.
      `dispatch_to_environment_base_declines_and_unknown_name_reports`,
      `dispatch_bound_command_resolves_user_definition`, the EDIT/CLOSE ladder
      tests, `ffcmd_verb_unaffected_by_registry`, `non_editor_context_does_not_route_through_ffedit`).
      Add at the top of each removed region a comment:
      `// core-001 strip: removed -- validated the parked FFEDIT editor command`
      `// environment (see docs/reentry-register.md, true-core-proposal.md s1/s2/s4).`
      No dangling empty module: tests_command.rs keeps hundreds of other live
      tests, so `#[cfg(test)]`/`mod` stays non-empty. Remove any now-unused
      `use ff_edit_operations::...;` / `use tempfile::TempDir;` imports that were
      local to the deleted functions (they are per-fn `use` statements, so they
      disappear with the function body -- confirm no file-level unused-import
      warning remains under `-D warnings`).
      Files: `crates\ff-desktop\src\shell\tests_command.rs`
      Verify:
      `cargo test --manifest-path '...\Cargo.toml' -p ff-desktop --bin ffwb --no-run`
      -> log shows it compiles; then run the file's module
      `cargo test --manifest-path '...\Cargo.toml' -p ff-desktop --bin ffwb shell::tests_command -- --test-threads=1`
      -> READ log; the 17 removed names are gone and the remaining
      tests_command tests pass.

- [ ] 4. Remove the parked FFEDIT profile tests in tests_misc.rs (tests #18-#22):
      `caps_no_arg_toggles_state`, `profile_caps_on_keyword_updates_caps`,
      `lock_off_re_enables_profile_changes`, `hilite_on_sets_hilite_mode`,
      `hilite_logic_sets_logic_mode`. Also remove the coincidentally-passing
      FFEDIT-profile siblings in the same cluster that validate parked behaviour
      (e.g. `lock_on_prevents_profile_changes`, `hilite_unknown_mode_sets_error`)
      so no parked-behaviour profile tests linger. Add the same one-line
      core-001 removal note above the cluster. Keep the non-profile tests in this
      file (F-key resolver, SUBMIT/JES error, etc.) untouched.
      Files: `crates\ff-desktop\src\shell\tests_misc.rs`
      Verify:
      `cargo test --manifest-path '...\Cargo.toml' -p ff-desktop --bin ffwb shell::tests_misc -- --test-threads=1`
      -> READ log; removed names gone, remaining tests pass; no unused-import
      warning under clippy `-D warnings`.

- [ ] 5. Remove the parked editor session-restore tests in tests_session.rs
      (tests #23, #24): `reopened_editor_descriptor_recaptures_owning_environment`
      and `restore_editor_descriptor_opens_file`. Both depend on a
      `WorkspaceKind::Editor` descriptor reopening a FileEditor tab, which the
      parked `open_file` stub can no longer do. Add the core-001 removal note.
      Leave the other descriptor-restore tests (CommandConfigurator, PluginManager,
      EventLog, history recall, etc.) untouched -- those Contexts are retained.
      Do NOT remove the `WorkspaceKind::Editor` variant or its reconstruct arm in
      production: it lives in `ff-session` and is still referenced; the parked
      degradation (descriptor present but file cannot reopen) is intended and
      needs no production change.
      Files: `crates\ff-desktop\src\shell\tests_session.rs`
      Verify:
      `cargo test --manifest-path '...\Cargo.toml' -p ff-desktop --bin ffwb shell::tests_session -- --test-threads=1`
      -> READ log; removed names gone, remaining session tests pass.

- [ ] 6. Remove the parked file-open-into-editor test in tab_manager.rs (test
      #25): `file_tab_has_kind_file_editor` (around line 1416). It calls
      `mgr.open_file(&path,&runtime).expect("open")`, which now returns the
      parked Err. Add the core-001 removal note. Keep
      `new_untitled_tab_adds_untitled_kind` and the other tab_manager tests --
      `new_untitled_tab` still creates an Untitled tab and those pass. The
      `tab_manager::tests` module keeps many other tests, so no dangling empty
      module. Remove the now-unused `use std::io::Write;` / `use tempfile::NamedTempFile;`
      if they were local to the deleted test (per-fn `use`, so they vanish with
      the body -- confirm no unused-import warning).
      Files: `crates\ff-desktop\src\tab_manager.rs`
      Verify:
      `cargo test --manifest-path '...\Cargo.toml' -p ff-desktop --bin ffwb tab_manager::tests -- --test-threads=1`
      -> READ log; `file_tab_has_kind_file_editor` gone, remaining tab_manager
      tests pass.

- [ ] 7. Scoped trio on ff-desktop after the removals (gate-policy.md fast loop).
      Confirms no unused imports / dead code / fmt drift were introduced by the
      deletions.
      Files: none (verification).
      Verify, each redirected to its own log under `tools\logs\` and READ back:
      - `cargo fmt --manifest-path '...\Cargo.toml' -p ff-desktop -- --check`
      - `cargo clippy --manifest-path '...\Cargo.toml' -p ff-desktop -- -D warnings`
      - `cargo check --manifest-path '...\Cargo.toml' -p ff-desktop`
      Expect all clean. Fix any unused-import / dead-code fallout from the
      removed tests (narrowest change; do not weaken live assertions).

- [ ] 8. Full ff-desktop bin test run to confirm the 25 parked failures are gone
      (and, if test #26 was an isolation flake, that it is the only remaining
      sensitivity). This is the per-crate scoped test, acceptable here because it
      is bounded to one bin; if it risks a long/quiet run, leave the
      whole-suite confirmation to the owner's full gate.
      Files: none (verification).
      Verify:
      `cargo test --manifest-path '...\Cargo.toml' -p ff-desktop --bin ffwb -- --test-threads=1`
      redirected to `tools\logs\ff-desktop-tests.log`; READ the log. Expect 0
      failures (test #26 passes under `--test-threads=1`). If #26 was classified
      a real regression in item 2, confirm the production fix made it pass here
      too.

- [ ] 9. Hand off for the owner's authoritative full gate (gate-policy.md: full
      `cargo gate` must be CLEAN before merge to main). Report: which tests were
      removed (25) with the parked-behaviour justification, the theme-editor
      classification (flake-kept vs regression-fixed), and the ff-volume clippy
      confirmation. If the theme-editor test remains a parallel-only flake, flag
      it to the owner as a known out-of-scope flake to re-run single-threaded in
      isolation during the full gate, per gate-policy.md.

---

## 4. Correctness guardrails honoured (testing.md)

- No test is weakened or deleted merely to go green. Every removed test is
  justified as validating behaviour INTENTIONALLY parked by the core-001 strip
  (editor / FFEDIT environment / editor SAVE seam / editor session restore /
  file-open-into-editor), each citing docs/reentry-register.md and
  true-core-proposal.md sections 1/2/4.
- The one retained-Context failure (Theme Editor) is NOT deleted; it is
  classified in isolation and either kept (flake) or fixed in production
  (regression) -- never weakened.
- No production dispatch/env code is altered for the parked tests: the FFCMD
  base spine, `is_ffedit_active`, `dispatch_to_environment` base path, and the
  claim gate are already correctly stubbed per framework-conformance.md (single
  focus-latch path, `InteriorFocus::none()`, no bespoke intercept), proven by
  the still-passing base-only sibling test.
- No dangling empty `#[cfg(test)]` modules: every touched test file retains many
  other live tests.
