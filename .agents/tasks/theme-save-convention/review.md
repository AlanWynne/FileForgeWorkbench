# Review -- Theme Editor ISPF explicit-save convention (B081, project-master Req 11)

Verdict: APPROVED

## Scope reviewed
- Diff: `git diff main...HEAD` on worktree `theme-save-convention` (529 lines).
- Commit `037177a`: fix + recorded verification.
- Supporting unchanged files read to establish consequences:
  `shell/commands_ladder_a.rs` (SAVE/END/RETURN/REFRESH dispatch),
  `shell/commands_theme.rs` (A::EditToken live preview, A::Save/A::SaveAs),
  `crates/ff-theme-editor/src/lib.rs` (action enum + panel state),
  `theme_defaults.rs` (load_theme_by_name / theme_slug), test helpers in
  `tests_common.rs`.

## Task requirements vs implementation

1. KindTag::ThemeEditor wired into the shared seam -- MET.
   `editable_context.rs` adds `EditableKind::ThemeEditor` and arms in
   `editable_context_kind` / `editable_persist` / `editable_discard` /
   `editable_reload`, mirroring the AliasEditor reference arm. The AliasEditor
   arms are byte-identical in behaviour (only the `if let` in discard/reload was
   rewritten to a `match` that routes the same calls; AliasEditor path unchanged).

2. persist delegates to the single existing write path -- MET.
   `theme_editor_persist` (new `commands_theme_save.rs`) reuses
   `write_theme_file` (now `pub(super)`) and the built-in Save As redirect, and
   returns `Result<(), String>`. END branches on this Result. No duplicated
   write logic: `A::Save` now calls `theme_editor_persist`, `A::SaveAs` calls
   `save_theme_as`; both ultimately call the one `write_theme_file`.

3. discard reverts the live preview -- MET.
   `theme_editor_discard` drops the working copy AND sets `self.palette` back to
   the persisted theme via `load_theme_by_name`. The live preview mutates
   `self.palette` on `A::EditToken` (confirmed in `commands_theme.rs`), so the
   revert is the correct counterpart; after discard the running palette equals
   the persisted theme.

4. reload re-opens the working copy -- MET.
   `theme_editor_reload` re-seeds the working copy and running palette from the
   persisted theme, reachable via REFRESH (dispatch unchanged in the ladder).

5. In-panel Save/Save As consistent, single path -- MET.
   `A::Save` -> `theme_editor_persist`, `A::SaveAs` -> `save_theme_as`. No second
   divergent path; the prior inline Save/Save As bodies were removed.

6. Preview kept as a view, reverted on discard -- MET (see 3).

## Command-seam behaviour (unchanged ladder, now reached for ThemeEditor)
- SAVE: `editable_persist` + stay, error surfaced inline (Result ignored).
- END: `editable_persist`; Ok -> `nav_end` (exit); Err -> stay, error shown.
- RETURN: `editable_discard` + `nav_return` (exit).
- REFRESH: `editable_discard` + `editable_reload` + stay.
Because `editable_context_kind()` now returns `Some(ThemeEditor)`, the Theme
Editor flows through these exact paths. No bespoke intercept or second
dispatcher added (framework-conformance honoured).

## Tests
`tests_theme_save_convention.rs` (6 tests, each annotated
`// Validates: project-master Requirement 11.x`):
- 11.3 SAVE persists + stays open.
- 11.4 END persists then exits.
- 11.4 END stays open + error when persist fails (built-in, cleared name buffer).
- 11.5/11.6 RETURN discards, reverts preview palette to persisted, exits, file
  unchanged.
- 11.2 bare edit does not autosave.
- 11.8 in-panel Save button shares the SAVE command persist path.
All five required behaviours covered plus the Save-button equivalence. Helpers
and APIs used (`dir_overrides.themes`, `load_from_toml`, `EditableToken::all/label`,
`ThemeEditorAction::{Copy,EditToken,Save}`) all exist and match pre-existing
theme tests.

## File-size (400-line rule)
- `commands_theme.rs`: 307 lines (was 409; split out the seam ops).
- `commands_theme_save.rs`: 130 lines.
- `editable_context.rs`: ~117 lines.
- `tests_theme_save_convention.rs`: 219 lines (dedicated `#[cfg(test)]` file).
All production files under 400 non-test lines.

## Recorded verification (commit message, not re-run)
- cargo check -p ff-desktop: clean.
- cargo clippy -p ff-desktop --all-targets -- -D warnings: clean (rc 0).
- cargo fmt -p ff-desktop -- --check: clean (rc 0).
- cargo test -p ff-desktop tests_theme_save_convention: 6 passed; 12 pre-existing
  theme tests pass.
- One pre-existing egui_kittest flake (`full_shell_theme_editor_type_name_and_
  save_creates_user_theme`) documented as failing identically on clean main;
  out of scope, deferred to the owner's full gate per gate-policy.

## Findings
None blocking. ASCII-only, scope limited to the Theme Editor, AliasEditor
behaviour preserved, single persist path, annotations present.
