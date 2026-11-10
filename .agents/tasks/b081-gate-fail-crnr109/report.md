# B081 Gate Failure on `feature/crnr109-locale-alias` -- Root-Cause Report

## Summary answer (verdict first)

**(b) PRE-EXISTING failure on `main`, unrelated to the CR-NR-109 branch.**

The single failing test
`ff-desktop shell::tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme`
**fails identically on a clean `main` checkout (commit `1d10e13`)**, run in
isolation as a single scoped test. The CR-NR-109 locale-alias branch did not
cause it, did not touch the production code path it drives, and did not modify
the test. The branch merely surfaced a bug that was already red on `main` --
either the branch is the first full `cargo gate` run since the defect landed, or
earlier runs on `main` did not surface it to the owner.

It is **not flaky**: it failed deterministically on `main` in a clean,
single-test, single-process run (`0.13s`, `1030 filtered out`), and the owner saw
it fail on the branch. Same line, same assertion message, both times.

Recommended next action: **merge/ship CR-NR-109 is NOT blocked by a defect it
introduced, but the branch tree still contains the red test (inherited from
`main`).** Fix B081 SEPARATELY against `main` (it is a pre-existing bug on the
mainline), then the fix flows to the branch by rebase/merge. See Recommendations.

---

## Evidence

### 1. The branch's 4 commits are all localization-scoped; none touch theme code

`git log --oneline main..feature/crnr109-locale-alias`
(`tools/logs/b081-gitlog.log`):

```
a3dbe67 docs(localization): land CR-NR-109 spec deltas, plan, and verification tool scripts
397f243 docs(localization): flip CR-NR-109 Req 11.1-11.10 TCR rows to PASS + tick Task 11 subtasks
4ae52da feat(localization): locale-aware command-alias layer + menu-kind localised options (CR-NR-109 Task 11.1-11.7)
f93063a feat(localization): add Afrikaans POC command-alias data + verb keys (CR-NR-109 Task 11.7)
```

`git diff main..feature/crnr109-locale-alias --stat`
(`tools/logs/b081-diffstat.log`) -- 38 files, and **none** of them is a
theme-editor, theme-save, or `ff-theme`/`ff-theme-editor` file. The changed
production files are command-dispatch / locale-overlay / menu-render:
`command_config/mod.rs`, `menu_workspace/*`, `shell/alias_overlay.rs`,
`shell/alias_reader.rs` (new), `shell/dispatch.rs`, `shell/environment.rs`,
`shell/environment_contract.rs` (new), `shell/locale_resolver.rs` (new),
`shell/construct.rs`, `shell/commands_menu.rs`, `main.rs`, plus i18n DATA and
docs. The theme Save seam (`shell/commands_theme.rs`,
`theme_editor_panel.rs`, `ff-theme-editor/src/lib.rs`, `ff-theme/*`,
`shell/render_theme.rs`) is **untouched** by the branch.

### 2. The failing test is byte-identical on `main` and the branch

`git diff main..feature/crnr109-locale-alias -- crates/ff-desktop/src/shell/tests_focus.rs`
produced an **empty diff** (`tools/logs/b081-testsfocus-diff.log` -- empty). The
test `full_shell_theme_editor_type_name_and_save_creates_user_theme` exists on
`main` too (`git show main:...tests_focus.rs` contains it at line 1208 --
`tools/logs/b081-main-hastest.log`). The branch neither added nor changed the
test; it predates the branch (it was authored as the B081 TDD fix, per
`.agents/tasks/b081-theme-save/findings.md` section E, which called for exactly
this "type a name and click Save/Save As" full-shell kittest).

### 3. What the test asserts and which production path it drives

`crates/ff-desktop/src/shell/tests_focus.rs`, `fn
full_shell_theme_editor_type_name_and_save_creates_user_theme` (panic at line
1256). It:

1. Isolates the themes dir to a `TempDir` via
   `harness.state_mut().dir_overrides.themes`.
2. Opens the Theme Editor with `dispatch_command_string("THEME")` and runs 4
   frames; asserts the active tab is `KindTag::ThemeEditor`.
3. Asserts the editor opened on a built-in (`ff_theme::is_builtin_theme(&selected)`)
   and that the **New-name field is PRE-FILLED** (`name_buffer` non-empty) -- the
   B081 UX fix that makes Save/Save As enabled on open.
4. Clicks the real rendered button: `harness.get_by_label("Save").click()` (the
   label resolves via `ff_i18n::t("theme-editor-btn-save")` = "Save").
5. **Asserts (line 1250-1256) that a USER `.toml` at the pre-filled name's slug
   now exists in the themes dir** -- "clicking Save on a built-in writes a USER
   theme (Save As), resolving B081". **This is the assertion that panics.**

Production path driven: the pure render in
`crates/ff-theme-editor/src/lib.rs` produces `ThemeEditorAction::Save` from the
always-enabled Save button (`ui.button(ff_i18n::t("theme-editor-btn-save"))`);
the shell adapter `theme_editor_panel.rs` stashes it; `render_body` drains it;
`crates/ff-desktop/src/shell/commands_theme.rs::apply_theme_editor_action(A::Save)`
is supposed to (for a built-in selection with a non-empty `name_buffer`) recurse
into `A::SaveAs(name)` and `write_theme_file(...)` into the themes dir. The test
failing at the file-exists assertion means that end-to-end render->action->write
chain did not leave a user `.toml` on disk under the test's harness conditions.

### 4. The test fails on a CLEAN `main` checkout, in isolation

Scoped single-test run in the main workspace (on `main`, HEAD `1d10e13`, source
tree clean -- `git status --short` shows only doc edits, no source changes:
`tools/logs/b081-mainws-head.log`, `tools/logs/b081-mainws-status.log`):

```
cargo test -p ff-desktop --bins full_shell_theme_editor_type_name_and_save_creates_user_theme
```

Result (`tools/logs/b081-maintest2.log`):

```
running 1 test
test shell::tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme ... FAILED

thread '...full_shell_theme_editor_type_name_and_save_creates_user_theme' panicked at
  crates\ff-desktop\src\shell\tests_focus.rs:1256:5:
clicking Save on a built-in writes a USER theme (Save As), resolving B081

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1030 filtered out; finished in 0.13s
```

Same test, same line (1256), same assertion message as the branch gate failure.
`1030 filtered out` = only this one test ran in the process, so **cross-test
state contamination is excluded as the cause** -- the test is red on its own on
`main`.

(The branch worktree was not separately re-run for this single test: it is
unnecessary. The owner already observed the failure there via `cargo gate`, the
test body is byte-identical to `main`, and the branch changes none of the
production code the test drives. Reproducing the same deterministic failure on
clean `main` is the decisive comparison the brief asked for.)

### 5. Could the CR-NR-109 changes plausibly affect this path? -- No

Traced each branch change class against the theme-save path:

- **Command-alias / dispatch rewrite** (`shell/dispatch.rs`,
  `shell/environment.rs`, `environment_contract.rs`, `alias_overlay.rs`,
  `alias_reader.rs`, `command_config/mod.rs`): these sit on the command
  classification/alias-overlay seam. `THEME` is dispatched the same way on both
  trees; the test reaches `KindTag::ThemeEditor` successfully (the assertion that
  the editor opened is NOT what fails). The failure is downstream of opening, at
  the Save->write step, which goes through `apply_theme_editor_action`, a seam the
  branch does not modify.
- **Locale overlay / construct.rs seed** (`construct.rs` adds an
  `i18n: I18nState { alias_store }` and seeds a locale alias store;
  `main.rs` adds a locale hot-reload callback): behaviour-neutral under default
  `en` with no `i18n/en/aliases/` tree (identity-only, Req 10.2, per the diff's
  own comments). It feeds the command-alias resolver, not the theme save seam.
- **i18n catalogue / `get_by_label("Save")`**: the "Save" label resolves via
  `ff_i18n::t("theme-editor-btn-save")` = "Save" from
  `crates/ff-i18n/i18n/en/theme-editor.ftl` (`theme-editor-btn-save = Save`). Both
  that key and the `t()`-based render already exist on `main`
  (confirmed present in the `main`-checked-out workspace tree), and the harness
  installs the full English Identity_Base catalogue via
  `i18n_bootstrap::install_catalogue_for_tests()` -- a file **identical on both
  trees** (`git diff ... -- shell/i18n_bootstrap.rs` empty). So the label query
  resolves to "Save" on `main` too; the branch's new `ff-desktop-verbs.ftl`
  (verb-* keys) does not define or shadow `theme-editor-btn-save`. The
  `tests_common::install_verb_find_soek_catalogue` helper the branch adds copies
  the full shipped `en/` tree (so "Save" keeps resolving) and is only called by
  the branch's new locale-menu tests, not by this theme test.

Conclusion: no branch change can plausibly flip this assertion. The proof is
empirical -- it is already red on `main` without any branch code present.

---

## Conclusions

1. **The branch did NOT cause the failure.** B081's test is red on clean `main`.
2. **The failure is a genuine, deterministic B081 Save-As regression on the
   mainline** (the render->`apply_theme_editor_action(Save)`->`write_theme_file`
   chain does not leave a user `.toml` under the full-shell harness), not a flake
   and not environment-dependent -- it reproduced in 0.13s in a clean isolated
   run. The prior `.agents/tasks/b081-theme-save/findings.md` documented the fix
   design (make Save on a built-in behave as Save As, pre-fill the name field);
   the test encoding that fix is present but currently NOT satisfied on `main`,
   so either the production fix was never fully landed or a later change on `main`
   re-broke it. This report did not run the suite to bisect which `main` commit
   introduced the red; that is the natural follow-up for the B081 fix owner.
3. **The CR-NR-109 work itself is clean**: fmt/clippy/build green, 2793/2794
   tests pass, and the one failure is inherited from `main` in a subsystem the
   branch does not touch.

## Recommendations

1. **Do not attribute the gate failure to CR-NR-109.** The branch's deliverable
   (locale-aware command-alias layer) is unaffected. Record B081 as a
   pre-existing `main` failure, not a branch regression.
2. **Fix B081 separately on `main`** (it is a mainline defect). Start from
   `.agents/tasks/b081-theme-save/findings.md`: verify the Save-on-built-in ->
   Save-As behaviour and the name-buffer pre-fill actually fire through the
   full-shell render path the test drives, and that `write_theme_file` targets
   the harness `dir_overrides.themes` dir. The fix then reaches the branch via the
   normal rebase/merge of `main`.
3. **For the CR-NR-109 branch specifically**: it can merge on its own merits once
   the owner accepts that its one red test is an inherited `main` failure. If a
   fully-green gate on the branch is required before merge, rebase the branch onto
   a `main` that already carries the B081 fix (option 2) rather than patching
   B081 inside the locale-alias branch (keep the branch scope clean).
4. Optional: a quick `git bisect` on `main` between the B081-fix commit and
   `1d10e13`, running only this one scoped test, would pinpoint the commit that
   left the test red -- cheaper than a suite run and bounded.

---

### Commands run (all read-only; logs under `tools/logs/`)

- `git -C <worktree> log --oneline main..feature/crnr109-locale-alias` -> `b081-gitlog.log`
- `git -C <worktree> diff main..feature/crnr109-locale-alias --stat` -> `b081-diffstat.log`
- `git -C <worktree> diff main..branch -- .../tests_focus.rs` -> `b081-testsfocus-diff.log` (empty)
- `git -C <worktree> diff main..branch -- construct.rs main.rs tests_common.rs` -> `b081-construct-diff.log`
- `git -C <worktree> diff main..branch -- shell/i18n_bootstrap.rs` -> `b081-i18nboot-diff.log` (empty)
- `git -C <worktree> show main:.../tests_focus.rs` + grep -> `b081-main-hastest.log`
- `git -C <mainws> rev-parse HEAD / status --short / rev-parse --abbrev-ref HEAD` -> `b081-mainws-*.log`
- `cargo test -p ff-desktop --bins full_shell_theme_editor_type_name_and_save_creates_user_theme` (on `main`) -> `b081-maintest2.log` (FAILED, line 1256)
