# B081 Theme-Editor Test Flake -- Root-Cause Findings (READ-ONLY)

Investigation of `shell::tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme`
(crates/ff-desktop/src/shell/tests_focus.rs:1318). No code was changed.

Repo state read: `main`, HEAD `e63214a` (the worktree `.worktrees/crch072-alias-simplification`
is checked out at the cwd, but the two files that matter here -- `tests_focus.rs` and
`i18n_bootstrap.rs` -- are byte-identical between main and the branch; see "CR-CH-072
independence" below). All `file:line` citations are from the live source read this session.

---

## 1. Summary answer (root cause + fix in one paragraph)

The failure is the SAME CLASS as the previously-fixed i18n-catalogue race: a process-global,
mutated by a concurrent test under threaded `cargo test`, that this test transitively depends on.
The dependency is the **i18n message catalogue** read by the Theme Editor render to label its
buttons: the Save button's accessible name is `ff_i18n::t("theme-editor-btn-save")`
(crates/ff-theme-editor/src/lib.rs:290), which the test locates with
`harness.get_by_label("Save")` (tests_focus.rs:1370). The repo already fixed the general case
with a PER-THREAD catalogue override (`ff_i18n::set_test_catalogue`, crates/ff-i18n/src/seam.rs:48),
installed for every full-shell harness by `install_catalogue_for_tests`
(crates/ff-desktop/src/shell/i18n_bootstrap.rs:227). The leak is that this isolation is NOT
airtight: (a) `install_catalogue_for_tests` falls back to the **process-global**
`install_catalogue_for_locale` when the shipped root/base does not resolve (i18n_bootstrap.rs:230),
losing the per-thread shield; and (b) a concurrent test --
`locale_change_applies_afrikaans_end_to_end_through_reload_callback` (i18n_bootstrap.rs:472) --
installs an **Afrikaans** catalogue on the process-global via `ff_i18n::set_catalogue`
(i18n_bootstrap.rs:135) and never restores it, while holding a lock
(`GLOBAL_CATALOGUE_SEAM_LOCK`, i18n_bootstrap.rs:246) that the theme test does NOT acquire. When
the theme test reads the global (fallback branch, or any thread without the override), the Save
button renders as the Afrikaans "Stoor" (crates/ff-i18n/i18n/af/theme-editor.ftl:22), so
`get_by_label("Save")` finds zero nodes and the test fails. Pass-isolated and pass-under-nextest
are exactly what a shared-global race predicts: one test per process means nothing concurrently
rewrites the global. **This is a shared-state race, not GUI-harness timing.** The minimal, correct
fix mirrors the existing per-thread precedent: make `install_catalogue_for_tests` ALWAYS install a
thread-local override (never the global fallback), AND stop the one global-mutating locale test
from leaving the global dirty (restore it, or convert it to the thread-local seam / make every
full-shell label test share its serialization lock). Smallest honest change: the two i18n seams,
no product behaviour change -> refactor/test-infra class, no requirements gate.

---

## 2. What the test asserts and what state it touches (Question 1)

The test (tests_focus.rs:1318-1402):
1. Creates a `tempfile::TempDir`, seeds default theme files into it
   (`crate::theme_defaults::ensure_default_theme_files`), and points the shell's themes dir at it:
   `harness.state_mut().dir_overrides.themes = Some(themes)` (tests_focus.rs:1326). So the theme
   file path is **fully isolated per test** -- not a shared dir.
2. Opens the Theme Editor via `dispatch_command_string("THEME")` and runs 4 frames.
3. Asserts the editor opened on a built-in and that `name_buffer` is pre-filled (the original
   B081 logic fix).
4. Runs one more frame, then **`harness.get_by_label("Save").click()`** (tests_focus.rs:1370),
   runs a frame to apply.
5. Asserts the user `.toml` exists at `theme_slug(new_name)` (tests_focus.rs:1253-1256), that the
   built-in's own file was NOT written, that the new theme is in `available`, and that it can be
   Set Active.

Shared / global / filesystem state the test reads or writes:
- **Themes dir**: ISOLATED to a TempDir via `dir_overrides.themes` -> `themes_dir()`
  (crates/ff-desktop/src/shell/commands_theme.rs:16). Refutes a themes-dir race.
- **User-config + history paths**: ISOLATED per call to unique temp files in `make_shell`
  (tests_common.rs:118-141, B048 isolation). Refutes a user-data-dir race.
- **Active palette / theme state**: lives on the `WorkbenchShell` INSTANCE
  (`self.palette`, `self.theme_editor_panel`); `ff-theme`'s `ThemeApi` holds its palette in an
  instance-level `Arc<RwLock<...>>` (crates/ff-theme/src/api.rs:36), NOT a process-global static.
  Refutes an active-theme-global race.
- **i18n catalogue**: PROCESS-GLOBAL. The render resolves every button label through the
  `ff_i18n::t` seam, including `theme-editor-btn-save` -> "Save" (en) / "Stoor" (af). This is the
  only process-global mutable state the test's render transitively depends on.
- **egui/fonts/Context**: created fresh per `Harness` instance; not shared across tests.

## 3. The SAVE path it exercises (Question 2)

- Render produces `ThemeEditorAction::Save` when the Save button is clicked
  (crates/ff-theme-editor/src/lib.rs:290-292).
- The shell applies it in `apply_theme_editor_action` (commands_theme.rs:66). For a built-in
  selection with a non-empty `name_buffer`, `A::Save` re-dispatches as `A::SaveAs(new_name)`
  (commands_theme.rs:131-134), which calls `write_theme_file` (commands_theme.rs:274) ->
  `std::fs::write(<themes_dir>/<slug>.toml, ...)`.
- `themes_dir()` returns the **test override** when set (commands_theme.rs:16). So when the Save
  action actually fires, the file lands in the TempDir and the assertion passes.

Conclusion: the save path and its destination are correctly isolated. The ONLY way the file is
not written is if the `Save` action never fires -- i.e. the click did not land on the Save button,
which happens when the button is not labelled "Save" (Afrikaans global leaked in) so
`get_by_label("Save")` matches zero nodes and panics BEFORE the click. (`kittest` `get_by_label`
is an exact-name match and panics on zero-or-many; the sibling button is "Save As" / "Stoor As",
never an accidental match.)

## 4. The concurrent mutator (Question 3)

Grep of every `set_catalogue` / `install_catalogue*` caller in the ff-desktop test binary
(crates/ff-desktop/src/shell/i18n_bootstrap.rs and tests_common.rs):

- `install_catalogue_for_tests` (i18n_bootstrap.rs:227): used by `harness_shell` and `make_shell`.
  Normal branch installs the English base on the **per-thread** seam (`set_test_catalogue`,
  i18n_bootstrap.rs:229). FALLBACK branch (root/base unresolved) installs on the **process-global**
  seam (`install_catalogue_for_locale`, i18n_bootstrap.rs:230) -- NO thread-local shield.
- `install_verb_find_soek_catalogue` / `install_find_menu_localised_catalogue` (tests_common.rs:393,
  449) and the `environment.rs` / `alias_reader.rs` helpers: all use `set_test_catalogue`
  (per-thread). Safe.
- **`locale_change_applies_afrikaans_end_to_end_through_reload_callback` (i18n_bootstrap.rs:472)**:
  calls `install_catalogue(&config)` with `ui.locale = "af"` ->
  `install_catalogue_for_locale("af")` -> **`ff_i18n::set_catalogue(af)`** on the PROCESS-GLOBAL
  (i18n_bootstrap.rs:135). It does NOT clear or restore the global afterwards. It takes
  `GLOBAL_CATALOGUE_SEAM_LOCK` (i18n_bootstrap.rs:479), but that lock is only shared with the ONE
  other global-path test; the full-shell theme test does not acquire it.
- `install_catalogue_for_locale_en_installs_the_seam` (i18n_bootstrap.rs:411): installs English on
  the global (also under the lock). Harmless to the theme test, but shows the global is written by
  tests.

So the mutator that turns the global non-English is
`locale_change_applies_afrikaans_end_to_end_through_reload_callback`. Note the whole-process-global
is left in Afrikaans permanently after it runs. `clear_test_catalogue` (seam.rs:54) exists but is
called NOWHERE in the codebase, so even thread-local overrides are only "reset" by the next
`set_test_catalogue` on the same thread -- the global is never reset at all.

## 5. Why pass-isolated / pass-under-nextest is consistent with a shared-global race (Question 4)

- **Isolated `cargo test <name>`**: only this test runs; the af-installing locale test never runs,
  so the global stays English (or the per-thread override stays English). Label = "Save". Passes.
- **nextest**: process-per-test. Each test gets a fresh process with an uninstalled/English seam;
  no other test can rewrite this process's global mid-run. Passes. (This is exactly why
  `harness_shell` force-installs English -- see the comment at tests_common.rs:321-327 -- so a
  fresh nextest process shows English labels rather than key fallbacks.)
- **Threaded `cargo test` (one process, N threads)**: the locale test's `set_catalogue(af)` is
  visible to every thread's global read. Any theme-test run that reads the global (fallback branch,
  or a thread whose override is stale) sees "Stoor" and `get_by_label("Save")` fails. Fails
  intermittently depending on interleaving -- the reported signature.

This is a SHARED-STATE race, not a GUI-harness timing issue. Distinguishing evidence:
- The test already settles frames deliberately (runs a frame before querying, tests_focus.rs:1367)
  and the label-width concern noted in its own comment is handled; the dependency that remains is
  the label TEXT, which is catalogue-driven, not layout-driven.
- The egui_kittest `Harness` runs the app `update()` and builds the AccessKit tree SYNCHRONOUSLY
  on the calling thread, so there is no cross-thread frame-settling nondeterminism inside one test.
- The only mutable state shared across tests that the render reads is the i18n global.
- A pure timing bug would also flake under nextest (timing is per-process); it does not. Only the
  shared-process threaded runner flakes. That is the fingerprint of shared process state.

Honest caveat: the single captured FAILURE log this session (tools/logs/b081-maintest2.log) shows
the panic at the FILE-EXISTS assertion (tests_focus.rs:1256), not at `get_by_label`. That log is a
`--bin ffwb` filtered run from an EARLIER point in the B081 work (bugs.md now records B081 FIXED
with a clean owner gate), so it is not a clean capture of the current threaded flake. If a current
threaded failure ever panics at the file-exists line rather than inside `get_by_label`, the
mechanism is the same root cause one step later: `get_by_label("Save")` matched a DIFFERENT node
because the exact label "Save" was absent and a stale AccessKit label coincidentally matched, so
the click did not produce `A::Save` and no file was written. Either panic site resolves to the same
cause -- the Save button's label not being English "Save" at query time. The recommended fix
removes that possibility entirely by guaranteeing the English per-thread catalogue.

## 6. Minimal, correct fix (Question 5) -- mirror the existing per-thread precedent

This is a shared-global race, so the fix is isolation, NOT a harness timing change. Two small,
honest changes, smallest first:

Primary (makes the per-thread shield airtight):
- `crates/ff-desktop/src/shell/i18n_bootstrap.rs::install_catalogue_for_tests` (line 227). Remove
  the process-global fallback: ALWAYS install via `ff_i18n::set_test_catalogue` (thread-local). If
  the shipped root cannot be resolved, build/install an EMPTY or key-fallback catalogue on the
  thread-local seam rather than calling `install_catalogue_for_locale` (which writes the global).
  Rationale: the fallback is the hole -- it is the one path by which a full-shell test depends on
  the shared global and can be clobbered. The normal branch is already correct.

Secondary (stop the one test leaving the global dirty):
- `crates/ff-desktop/src/shell/i18n_bootstrap.rs::locale_change_applies_afrikaans_end_to_end_through_reload_callback`
  (line 472). This test legitimately exercises the PRODUCTION global `install_catalogue` seam, so
  it must write the global. Make it leave no residue: after the assertions, restore the global to
  English (or `clear`/reinstall the base) before returning, so a later full-shell test that falls
  through to the global never sees Afrikaans. `clear_test_catalogue` only clears the thread-local;
  the global has no public reset, so the cleanest in-repo pattern is to re-install the English base
  on the global at the end of this test (and of
  `install_catalogue_for_locale_en_installs_the_seam`, which already leaves English -- harmless but
  worth making explicit). With the primary fix in place this secondary change becomes belt-and-
  braces, because no full-shell test would read the global any more.

Either change alone closes the reported race; doing BOTH matches the repo's defence-in-depth style
(the i18n per-thread fix plus the `GLOBAL_CATALOGUE_SEAM_LOCK` serialization already coexist).

Exact seams named:
- `ff_i18n::set_test_catalogue` (crates/ff-i18n/src/seam.rs:48) -- the per-thread install.
- `ff_i18n::clear_test_catalogue` (crates/ff-i18n/src/seam.rs:54) -- currently unused; the fix may
  start using it, or rely on the next `set_test_catalogue` overwrite as today.
- `install_catalogue_for_tests` (i18n_bootstrap.rs:227) -- drop the global fallback.
- `locale_change_applies_afrikaans_end_to_end_through_reload_callback` (i18n_bootstrap.rs:472) --
  restore the global on exit.

NOT recommended: adding a `std::sync::Mutex` that the theme test (and every full-shell label test)
must acquire. It would serialise a large swath of the suite for no gain over the per-thread fix,
and it is the pattern the repo deliberately moved AWAY from (seam.rs:30-40 comment: "the race is
removed rather than serialised"). A harness-level frame-settling change is also NOT the fix -- the
symptom is a label-text race, not a layout race.

## 7. Blast radius, gate class, CR-CH-072 independence (Question 6)

Blast radius:
- Files touched by the recommended fix: `crates/ff-desktop/src/shell/i18n_bootstrap.rs` (the
  `install_catalogue_for_tests` fallback + the one locale test's teardown). Optionally a one-line
  use of `ff_i18n::clear_test_catalogue` (already `pub`, crates/ff-i18n/src/seam.rs:54) -- no new
  public API.
- Tests affected: `full_shell_theme_editor_type_name_and_save_creates_user_theme` (the fix target)
  plus every other full-shell `get_by_label(...)` test that uses `harness_shell` -- they all become
  strictly more isolated, not less. No production code path changes (`set_test_catalogue` /
  `clear_test_catalogue` are test-only seams; production uses `set_catalogue` unchanged).

Gate class: **No requirements gate.** There is no observable product behaviour change -- this is a
test-isolation / infrastructure fix (refactor-class per workflow.md "what does NOT need the gate").
If the team prefers, treat it as a plain BUG fix against the existing B081 test (criteria
theme-and-appearance Req 20.5/20.13 already exist); either way, no new/changed criteria.

Independence from CR-CH-072: **Independent.** `tests_focus.rs` and `i18n_bootstrap.rs` are
byte-identical between `main` and the `crch072-alias-simplification` worktree (the diff logs
tools/logs/b081-testsfocus-diff.log and tools/logs/b081-i18nboot-diff.log are empty). The theme
test is not in CR-CH-072's diff (CR-CH-072 is the alias-model simplification). This fix should land
as its OWN small change on a clean `main`, NOT bundled into CR-CH-072. Per gate-policy.md it needs
a full `cargo gate` green before merge; the authoritative threaded/full-suite run is the owner's.

---

## Appendix -- key citations

- Test: crates/ff-desktop/src/shell/tests_focus.rs:1318 (fn), :1370 (`get_by_label("Save")`),
  :1253-1256 (file-exists assertion).
- Harness: crates/ff-desktop/src/shell/tests_common.rs:318 (`harness_shell`), :321-327 (why English
  is force-installed), :99 (`make_shell`), :118-141 (per-test temp config/history, B048).
- Save button label: crates/ff-theme-editor/src/lib.rs:290 (`ff_i18n::t("theme-editor-btn-save")`).
- en/af label data: crates/ff-i18n/i18n/en/theme-editor.ftl:29 ("Save"),
  crates/ff-i18n/i18n/af/theme-editor.ftl:22 ("Stoor").
- Save side effect: crates/ff-desktop/src/shell/commands_theme.rs:66 (`apply_theme_editor_action`),
  :131-134 (built-in Save -> SaveAs), :274 (`write_theme_file`), :16 (`themes_dir` test override).
- i18n seam: crates/ff-i18n/src/seam.rs:21 (process-global `active`), :33 (`TEST_OVERRIDE`
  thread-local), :48 (`set_test_catalogue`), :54 (`clear_test_catalogue`, unused), :77
  (`set_catalogue` global), :118 (`t_args` override-first).
- Bootstrap: crates/ff-desktop/src/shell/i18n_bootstrap.rs:227 (`install_catalogue_for_tests`, with
  the global fallback at :230), :135 (`set_catalogue` global install), :246
  (`GLOBAL_CATALOGUE_SEAM_LOCK`), :472 (the af-installing locale test).
- Theme state is per-instance (no global): crates/ff-theme/src/api.rs:36.
- B081 recorded FIXED (logic fix): docs/status/bugs.md:288.
- CR-CH-072 independence: empty diff logs tools/logs/b081-testsfocus-diff.log,
  tools/logs/b081-i18nboot-diff.log; main HEAD e63214a.
