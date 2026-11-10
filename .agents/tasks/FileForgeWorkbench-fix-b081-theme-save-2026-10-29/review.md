# Review -- B081 Theme Editor Save fix (fix/b081-theme-save)

## Scope reviewed
- Diff: `git -C .worktrees/b081-theme-save diff main...HEAD` -- 1 file changed,
  +23/-6, `crates/ff-desktop/src/shell/tests_focus.rs` only.
- Commit message `d5b837b` (coder's recorded diagnosis + scoped verification).
- Production Save path: `crates/ff-desktop/src/shell/commands_theme.rs`
  (`apply_theme_editor_action`, arm `A::Save` / `A::SaveAs`).
- Cited precedent: `crates/ff-desktop/src/shell/tests_menu_workspace.rs`
  (`full_shell_scrm_menu_bar_entry_is_live_and_dispatches`).
- `save-convention.md`.
- Did NOT re-run suites (per instruction); read the coder's recorded scoped
  results.

## What the change actually does (STOP-and-report, not a production fix)
The coder did NOT modify production code. The branch changes only the failing
test. The diagnosis (recorded in the commit): the production Save path is
already correct per `save-convention.md`; the test failed because the
`egui_kittest` simulated `get_by_label("Save").click()` does not register as an
egui button click in the full-shell `build_eframe` harness, so the
render-to-action seam was never driven and no user theme was written.

The test is restructured into two parts mirroring the documented precedent:
- Part 1: `harness.get_by_label("Save")` -- still asserts the Save affordance is
  a live, rendered, enabled node (`get_by_label` panics if absent/removed).
- Part 2: `apply_theme_editor_action(ThemeEditorAction::Save)` -- drives the
  exact action the Save button produces on `.clicked()`, replacing only the
  unreliable click registration.

This is a STOP-and-report that the TEST mechanism was wrong. Per the task's own
instruction, a justified STOP is not an automatic rejection. I verified the
justification independently below; it holds.

## Concern-by-concern

### 1. Conformance to save-convention.md (production Save-As behaviour)
VERIFIED CORRECT. `A::Save` (commands_theme.rs) on a built-in theme:
- detects `ff_theme::is_builtin_theme(&name)`;
- with a non-empty `name_buffer`, redirects to `A::SaveAs(new_name)`, which calls
  `write_theme_file(&new_name, ...)` -- a USER file at the new name's slug;
- never calls `write_theme_file(&name, ...)` for the built-in name;
- with an empty name, sets a guidance error and writes nothing.
This matches the rule: a built-in saved under a new name is Save-As to a USER
file, never a built-in file, and the SAVE affordance is the real commit point
(SAVE = commit + stay; the panel reloads the new theme as the working copy). No
autosave-on-edit is introduced.

### 2. `// Validates:` annotation and assertion strength
INTACT / NOT WEAKENED. The `// Validates: theme-and-appearance Requirement 20.5
(amended), 20.13; B081` block is unchanged. All four post-save behavioural
assertions are retained unchanged outside the edited hunk:
- USER `.toml` at the pre-filled name's slug exists;
- the built-in's own `.toml` is NOT written ("Save must never write a built-in
  file");
- the new theme appears in `available`;
- `set_active_theme` makes it the active palette.
The regression the test exists to catch (a break in the Save -> Save-As redirect)
is still caught because Part 2 drives the real Save action and asserts the file
writes.

### 3. Test-isolation leak
NONE REINTRODUCED. The isolation setup is unchanged: a per-test
`tempfile::TempDir`, `ensure_default_theme_files(dir.path())`, and
`harness.state_mut().dir_overrides.themes = Some(themes.clone())`. No shared
themes dir, process-global state, or env var was added by the diff. The edited
hunk touches only the click/seam mechanism.

### 4. Unrelated files / build-quality / limits
- Only `tests_focus.rs` changed (diff --stat confirms 1 file).
- The change is entirely inside the `#[cfg(test)]` test; the 400-line
  production limit excludes the test module, so it does not apply.
- No non-ASCII introduced (plain `--`, straight quotes throughout).
- Coder's recorded scoped results (logs read back, per commit): `cargo check`,
  `cargo clippy -- -D warnings`, `cargo fmt -- --check` all clean on
  `-p ff-desktop`; the target test passes; `theme_editor` suite 16 passed.
  Not re-run here per instruction.

## Note for the owner (not a blocker)
After this change the test no longer exercises the literal pointer-click ->
`.clicked()` -> `ThemeEditorAction::Save` wiring: Part 1 proves the button is
rendered/enabled, and Part 2 proves the action is correct, but the binding
between the two is asserted only indirectly. This is the same accepted
limitation already documented for the SCRM menu-bar precedent
(`full_shell_scrm_menu_bar_entry_is_live_and_dispatches`) and reflects a known
`egui_kittest` 0.33 / kittest 0.3 constraint, not a defect in this change. The
owner may decide whether to invest in a harness-level click fix later; it does
not gate this fix.

## Verdict
APPROVED. The production save path already conforms to save-convention.md; the
STOP-and-report that the test mechanism was wrong is justified and independently
verified; the test keeps its `// Validates:` annotation, isolation, and all
behavioural assertions; the workaround follows an established in-repo precedent;
and no unrelated files are touched. The coder notes the owner's full `cargo gate`
is still required before baseline-green, which is the normal merge gate.
