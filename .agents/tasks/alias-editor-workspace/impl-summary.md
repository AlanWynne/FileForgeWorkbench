# CR-NR-113 Alias Editor Workspace Context -- Implementation Summary

Branch: `crnr113-alias-editor` (worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr113-alias-editor`).
First iteration (no prior `review.json`). CR-NR-112 is landed in main
(`command_template` exists in `AliasEntry`, `AliasRowToml`, and the pre-pass), so
the template-dependent subtasks (5.1 template field, 5.3 template edit, 5.8
expansion) were implementable and are implemented.

## Criteria implemented (Requirement 5.1-5.12)

All twelve criteria implemented and covered by passing scoped tests.

- 5.1 User_Alias_Store: `<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`,
  locale-independent, NEW `UserAliasRowToml` + `parse_user_alias_toml` +
  `serialise_user_aliases` (the shipped 3-field `AliasRowToml`/`parse_alias_toml`
  and the `i18n/` layout are untouched). Absent -> empty; created on first save.
- 5.2 Opened by the `ALIASES` verb through the single front door
  (`resolve_target` -> `CustomWorkspace("aliases")` -> `dispatch_command_target`
  -> `open_alias_editor`). No new WorkspaceKind, no new dispatcher.
- 5.3 Lists rows (surface/root/environment/template); Add, per-row Delete (with a
  confirm window), per-field edits bound directly on the working rows.
- 5.4 Save validation: empty surface / empty canonical / within-SAME-environment
  surface->two-roots / built-in-root shadow (`builtin_ffcmd_roots()`); on failure
  the on-disk file is left unchanged.
- 5.5 Delete removes from the working copy; a subsequent save writes without it.
- 5.6 Live resolver refreshed IN PLACE via an EXTENDED re-seed
  (`refresh_alias_store_in_place`): one re-seed re-applies the identity/per-locale
  base AND the user-store overlay, so neither layer is dropped and the alias
  resolves without a restart.
- 5.7 User store loaded as a THIRD overlay, routed per-row to ffcmd/ffedit;
  resolves through the existing pre-pass; NO-SHADOW invariant (skip-with-WARN at
  load, reject at save); cross-layer collision = per-row apply so SAME-root is a
  no-op and DIFFERENT-root keeps the existing (per-locale/base) surface.
- 5.8 A parameterised alias authored in the editor expands through the existing
  CR-NR-112 pre-pass: `ecob helloW` -> `EDIT FFWB.COBOL.MASTER(helloW)` (no new
  expansion/dispatch behaviour).
- 5.9 Tab title `[ALIASES]`; END returns to the opener via the per-tab
  Navigation_Stack (Menus Editor parity, in-place `navigate_to`).
- 5.10 Interior focus via the single `WorkspaceContext::render -> InteriorFocus`
  path; proven by a full-shell egui_kittest first-Tab test.
- 5.11 v1 authors the user store ONLY; the editor never reads/writes the
  per-locale `i18n/<locale>/aliases/` catalogues.
- 5.12 Structural keys ASCII; field VALUES may be any valid UTF-8 (round-trip
  test with a non-ASCII value).

## Files changed

New:
- `crates/ff-desktop/src/shell/user_aliases.rs` -- data layer (AliasEnvironment,
  UserAliasRowToml, parse/serialise, `user_aliases_file()` + env-var test seam).
- `crates/ff-desktop/src/shell/alias_editor.rs` -- shell-side open/apply/save/
  refresh + `validate_user_alias_rows` (+ validation unit tests).
- `crates/ff-desktop/src/alias_editor_panel/{mod,state,render}.rs` -- state,
  action enum, pure render + `WorkspaceContext` impl (stable first/last ids).
- `crates/ff-desktop/src/shell/tests_alias_editor.rs` -- shell-level tests.

Modified (wiring, full sibling parity with the Menus Editor):
- `shell/alias_reader.rs` -- third-overlay load (`apply_user_overlay_from_file`,
  `apply_user_rows`), `seed_and_overlay_for_locale` extended, `ffcmd_roots`/
  `builtin_ffcmd_roots` visibility.
- `shell/mod.rs` (module decls + test module), `shell/state.rs` + `construct.rs`
  (panel field), `shell/render_body.rs` (owned-panel-swap arm),
  `shell/nav_stack.rs` + `nav_reconstruct.rs` (descriptor + reconstruct arms),
  `shell/target_dispatch.rs` ("aliases" arm), `shell/helpers.rs` +
  `mod_helpers.rs` + `titles.rs` (title/context lists), `shell/state_groups.rs`
  (`dir_overrides.user_aliases`).
- `session_manager.rs` (non-persist arms), `tab_state.rs` + `tab_state_ctors.rs`
  (TabKind/KindTag + ctor), `workspace_kind/mod.rs` (`BuiltinKind::Aliases`),
  `command_config/mod.rs` (`ALIASES` classifier + VERB_ROOTS), `main.rs`
  (module decl).
- `docs/quality/TCR.md` -- 12 CR-NR-113 rows flipped to covered (CR-NR-112 table
  untouched).

## Commands run (scoped, logs read back)

- `cargo check -p ff-desktop` -> Finished (clean).
- `cargo check -p ff-desktop --tests` -> Finished (clean).
- `cargo test -p ff-desktop --bin ffwb tests_alias_editor::` -> 9 passed.
- `cargo test -p ff-desktop --bin ffwb -- user_aliases:: alias_editor::
  full_shell_alias_editor_first_tab` -> 22 passed (incl. the full-shell
  egui_kittest first-Tab test and the end-to-end expansion).
- `cargo clippy -p ff-desktop --all-targets -- -D warnings` -> Finished (clean).
- `cargo fmt -p ff-desktop -- --check` -> clean (fmt applied no changes to the
  committed source).

Per the testing.md execution split, the authoritative full `cargo gate` /
`cargo nextest` run is the owner's. The owner ran the full `cargo gate` against
THIS worktree (`crnr113-alias-editor`) and it was CLEAN: `9959 run, 9959 passed,
0 failed` in 0:21:09 (fmt ok, clippy clean, build ok, test 9959/9959). That gate
proves the committed CR-NR-113 code compiles and all tests pass.

Resume-run re-verification (agent, scoped, logs read back):
- `cargo check -p ff-desktop` -> Finished clean.
- `cargo clippy -p ff-desktop -- -D warnings` -> Finished clean.
- `cargo fmt -p ff-desktop -- --check` -> clean (no diff).
- `cargo test -p ff-desktop --bins alias` -> 60 passed; 0 failed; 0 ignored,
  including all 12 CR-NR-113 proving tests, the full-shell egui_kittest
  first-Tab test (`full_shell_alias_editor_first_tab_focuses_first_interior`),
  the Req 5.6 re-seed test (`refresh_keeps_base_layer_and_adds_user_layer`),
  and the Req 5.8 end-to-end expansion
  (`parameterised_alias_authored_in_editor_expands_end_to_end`).

## Known limitations / follow-ups

- The free-function `reload_active_alias_store_for_locale` (the `ui.locale`
  hot-reload seam) reads the user store via `user_aliases_file()` (env-var or
  default path); the editor's save-time refresh uses the shell's resolved path
  (`dir_overrides`-aware) so tests isolate correctly. A `ui.locale` change in
  production re-reads the default user path -- consistent, since the editor
  writes there in production.
- No full dockable visual styling pass on the panel beyond the functional table
  (matches the Menus Editor's functional-first posture).

## Commits (on `crnr113-alias-editor`, not pushed/merged)

- `422a19a` feat(ff-desktop): Alias Editor Context + user alias store (CR-NR-113)
- `b30f984` test(ff-desktop): Alias Editor resolver/save/focus/end-to-end tests
- `247c5b2` docs(tcr): flip 12 CR-NR-113 alias-editor rows to covered

All three commits are present on `crnr113-alias-editor` (fork point `c7b9c26`);
the working tree is clean (`git status`: "nothing to commit, working tree
clean"). Not pushed, not merged, no PR.
