# Implementation Plan -- CR-CH-072 Alias-Model Simplification

Governing docs (worktree copies; design.md governs): localization Req 13 +
design "CR-CH-072" section; command-environments Req 2b + design; command-
configurator Req 6; amended localization Req 11.1/11.2/12.2/12.4, command-
semantics Req 14.1, command-framework Req 18.1. Task source: localization
tasks.md group 23-30, command-environments tasks.md group 32. Coverage:
docs/quality/TCR.md Phase (alias-model-simplification), 18 NOT-COVERED rows.

## Scope and worktree

All work targets the worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crch072-alias-simplification`
on branch `crch072-alias-simplification`. Use ABSOLUTE paths; pass
`-C <worktree>` to every git command. Agent verification is compile/clippy/fmt
clean + SCOPED fast tests per `gate-policy.md` and `testing.md` (the execution
split): run `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D
warnings`, `cargo fmt -p ff-desktop -- --check`, and SCOPED named tests
(`cargo test -p ff-desktop <name>`). Do NOT run the whole suite or `cargo gate`
as a verification step -- the authoritative full gate is the owner's manual run.
Build/test commands run through the clean pwsh7 wrapper per
`terminal-discipline.md`; read logs back rather than trusting exit codes.

## Owner-confirmed decisions (settled; do NOT reopen)

1. RESOLUTION is exactly TWO levels on the single existing front door (active
   env alias -> active roots -> reject -> FFCMD alias -> FFCMD roots), reusing
   `rewrite_verb_to_root` + `resolve_target`. No new dispatcher.
2. ONE-LAYER alias: surface lives IN the row; drop the `localisation_key`
   command-surface indirection and the `verb-*` ftl keys. Keep `command_template`
   (CR-NR-112). `.ftl` still localises DESCRIPTIONS and TITLES.
3. MENU OPTION stores the ROOT + a resolve-as-alias marker; render resolves the
   active-locale surface by root; persisted identity stays the ROOT (no saved-
   menu migration).
4. ONE store: shipped read-only seed `i18n/<locale>/aliases/<ENV>.toml` overlaid
   by a USER per-locale override `<User_Data_Dir>/FileForgeWorkbench/aliases/
   <locale>/<ENV>.toml`. The Alias Editor shows the active-locale EFFECTIVE table
   and writes to the USER override.

## Key code facts established during exploration

- `AliasEntry` (`shell/environment.rs`) and `AliasRowToml` (`shell/alias_reader.rs`)
  BOTH currently carry `localisation_key: Option<String>`; `AliasEntry` also has
  `command_template: Option<String>`. `AliasEntry::display_surface()` returns
  `ff_i18n::t(key)` when a key is present, else the raw `surface`.
- Resolution already runs through `shell/locale_resolver.rs`
  (`resolve_locale_verb` / `rewrite_verb_to_root` / `resolve_template_entry`)
  BEFORE `ff_command::resolve_target`. This is the single pre-pass to re-point.
- The store `LocaleAliasStore` (`shell/alias_reader.rs`) seeds identity tables,
  overlays the per-locale seed via `apply_overlay_from_dir`, and applies a THIRD
  layer: the CR-NR-113 user store at the flat, locale-INDEPENDENT path
  `<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`
  (`shell/user_aliases.rs::user_aliases_file`, overridable by
  `FFWB_USER_ALIASES_PATH`). CR-CH-072 replaces that third layer with a per-
  LOCALE override store.
- The Alias Editor (`shell/alias_editor.rs` + `alias_editor_panel/`) currently
  loads/saves the CR-NR-113 `user.toml` via `UserAliasRowToml` and shows columns
  Act / CE / Root / L18N / ALIAS / Parameters. CR-CH-072 must show the active-
  locale EFFECTIVE table and drop the L18N column.
- `display_surface_for_canonical` + `localised_option_command`
  (`menu_workspace/i18n_keys.rs`, called from
  `locale_resolver.rs::menu_option_command_surfaces`) are the menu command-column
  render path. After CR-CH-072 the per-root display surface is the in-row surface
  (no `t(key)`), so `display_surface` collapses to `self.surface.clone()`.
- af DATA: `i18n/af/aliases/FFCMD.toml` already stores the localised surface in
  each row (e.g. `surface = "INSTELLINGS"`) PLUS a redundant `localisation_key =
  "verb-settings"`; `af/ff-desktop-verbs.ftl` and `en/ff-desktop-verbs.ftl`
  carry the now-redundant `verb-*` keys. `i18n/af/aliases/FFEDIT.toml` carries
  `SOEK -> FIND` with `localisation_key = "verb-find"`. `af/menus.ftl`
  (descriptions/titles) is UNCHANGED.
- rust-standards: no `.rs` over 400 non-test lines; split test modules before
  ~200 lines; derive Debug; no `unwrap`/`expect` in non-test code. af DATA files
  are out of ASCII scope; all `.rs` and docs are ASCII.

## Tests the design supersedes (update in place with a why-note; do NOT weaken)

These existing tests assert the three-layer / `localisation_key` / `t(key)`
behaviour and MUST be rewritten to the in-row-surface model, each with an inline
`// CR-CH-072:` note explaining the supersession. Do not merely delete
assertions -- re-express them against the one-layer model so coverage is kept.

- `shell/environment.rs` tests: `display_surface_resolves_localisation_key_else_raw_surface`,
  `overlay_row_replaces_synthesised_default_without_changing_root`,
  `overlay_row_matching_identity_surface_updates_without_collision`, and the
  `install_verb_find_soek_catalogue` helper -- all exercise the `t(verb-find)`
  surface. Re-express: the display surface is the in-row `surface` string; the
  overlay still replaces the identity default and keeps the root; drop the
  `verb-find` catalogue dependency.
- `shell/alias_reader.rs` tests: `reader_parses_toml_rows_into_catalogue`,
  `reader_row_without_template_parses_to_none`,
  `reader_row_with_template_parses_to_some_verbatim`,
  `reader_rejects_within_environment_collision_and_retains_base`,
  `af_poc_alias_catalogue_loads_and_resolves`,
  `missing_environment_file_leaves_that_environment_identity_only` -- the keyed
  tuple `(surface, canonical, key, template)` loses its `key` element. Re-express
  the tuples to `(surface, canonical, template)` and drop the `verb-find`
  catalogue install in the af POC test (assert the in-row surface `SOEK`
  directly).
- `menu_workspace/i18n_keys.rs` tests referencing `localised_option_command`
  with a `Some("SOEK")` surface stay valid (the helper signature is unchanged);
  confirm they still pass after `display_surface` collapses.
- `shell/alias_editor.rs` tests `l18n_value_round_trips_through_validate_and_serialise`
  and any assertion that the L18N column persists -- the L18N column is removed,
  so delete this test with a `// CR-CH-072: L18N column removed (Req 6.2)` note
  and keep the other validation tests (empty surface/root, collision, no-shadow,
  root-validity) which remain valid over the override rows.
- `shell/user_aliases.rs` tests asserting `localisation_key` round-trips -- the
  field is dropped from the row; update the round-trip tests to the new shape.

## TCR rows to flip (docs/quality/TCR.md, Phase alias-model-simplification)

All 18 rows start 🔴 NOT COVERED. Flip to ✅ PASS as the corresponding test
lands, naming the test file/function in the "Test files" column:
loc Req 13.1 (item 2), 13.2 (item 4), 13.3 (item 13), 13.4 (items 6/7), 13.5
(item 9), 13.6 (items 9/14), 13.7 (item 12), 13.8 (item 5), 13.9 (items 14/15);
command-environments Req 2b.1-2b.3 (item 4), 2b.4 (item 4), 2b.5 (item 4);
configurator Req 6.1 (item 11), 6.2 (item 11), 6.3/6.4 (item 12), 6.5 (item 12),
6.6 (item 11), 6.7 / semantics 14.1 (item 2). Update the Summary counts.

---

# Plan

- [ ] 1. Write the failing unit tests for the unified `AliasEntry` / `AliasRowToml`
      one-lookup model (RED). Add tests asserting an entry resolves surface->root by
      a SINGLE lookup with NO `localisation_key` / `t(key)` consulted, that the in-row
      `surface` is the authoritative display/typeable text (`display_surface` returns
      `self.surface`), and that `command_template` is retained. These fail to compile
      until item 2 removes the field -- that compile failure IS the red state; annotate
      each `// Validates: localization Req 13.1; command-semantics Req 14.1 (amended)`.
      Files: crates/ff-desktop/src/shell/environment.rs (test module),
      crates/ff-desktop/src/shell/alias_reader.rs (test module).
      Verify: `cargo test -p ff-desktop alias` fails to compile / tests red, confirming
      the assertions reference the post-change shape.

- [ ] 2. Remove the command-surface `localisation_key` layer from the alias model (GREEN
      for item 1). Drop `localisation_key` from `AliasEntry` (environment.rs) and
      `AliasRowToml` (alias_reader.rs). Collapse `AliasEntry::display_surface()` to return
      `self.surface.clone()`. Change `KeyedAliasRow` / the overlay tuple from
      `(surface, canonical, key, template)` to `(surface, canonical, template)` in
      `shell/alias_overlay.rs` and update `apply_locale_overlay_keyed` /
      `apply_locale_overlay` / `parse_alias_toml` / the test overlay helpers accordingly.
      Keep `command_template` and the auto-synthesised identity rows. No `unwrap`/`expect`
      in non-test code; keep `derive(Debug)`. Update the superseded tests listed above
      with inline `// CR-CH-072:` notes.
      Files: crates/ff-desktop/src/shell/environment.rs,
      crates/ff-desktop/src/shell/alias_reader.rs,
      crates/ff-desktop/src/shell/alias_overlay.rs,
      crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs (tuple arity).
      Verify: `cargo check -p ff-desktop` compiles; `cargo test -p ff-desktop alias`
      green for the item-1 tests.

- [ ] 3. Re-point the menu command-column surface at the in-row surface. After item 2,
      `display_surface_for_canonical` already returns the in-row surface; confirm
      `menu_option_command_surfaces` (shell/locale_resolver.rs) and
      `localised_option_command` (menu_workspace/i18n_keys.rs) need no `t(key)` and keep
      the root fallback. Adjust only if the tuple change broke a call site.
      Files: crates/ff-desktop/src/shell/locale_resolver.rs,
      crates/ff-desktop/src/menu_workspace/i18n_keys.rs.
      Verify: `cargo test -p ff-desktop i18n_keys` and
      `cargo test -p ff-desktop menu_option_command_surfaces` pass.

- [ ] 4. Write + satisfy the two-level resolution-path tests over the in-row surface
      (RED then GREEN) -- command-environments Req 2b. Add/adapt unit tests in
      locale_resolver.rs asserting the order ACTIVE alias -> ACTIVE root -> reject ->
      FFCMD alias -> FFCMD root -> unresolved, IDENTICAL for a typed verb and a menu-
      selected option's root, over the in-row surface, under the diacritic-insensitive
      fold (CR-CH-065, Req 11.11), honouring prelude/shadowing precedence (Req 6a.4),
      with the resolved ROOT the persisted identity. Confirm `rewrite_verb_to_root` is
      the single pre-pass (no new dispatcher; `CommandTarget`/`resolve_target`
      unchanged) and still expands `command_template` before `resolve_target`. The
      current `resolve_locale_verb` already folds the "active env table then FFCMD" into
      per-environment lookups; verify the FFCMD-fallback leg is exercised (a surface
      unknown in the active env resolving against FFCMD). Annotate tests
      `// Validates: command-environments Req 2b.1-2b.5; localization Req 13.2`.
      Files: crates/ff-desktop/src/shell/locale_resolver.rs (test module; split the test
      module into a `_tests.rs` sibling if it approaches ~200 lines per rust-standards).
      Verify: `cargo test -p ff-desktop resolve_locale_verb` and
      `cargo test -p ff-desktop rewrite_verb_to_root` pass.

- [ ] 5. Confirm `command_template` expansion is byte-for-byte retained (Req 13.8). The
      existing `rewrite_verb_to_root_expands_parameterised_alias` / `_empty_arg_` /
      `_templateless_` / `_single_pass` tests already cover `&1..&N`/`&*`; verify they
      still pass unchanged after the tuple drop and keep them.
      Files: crates/ff-desktop/src/shell/locale_resolver.rs (existing tests).
      Verify: `cargo test -p ff-desktop rewrite_verb_to_root` all green.

- [ ] 6. Write the failing full-shell egui_kittest for menu-option resolve-as-alias
      (RED). Assert that under `af` a menu option renders the active-locale in-row
      surface (e.g. INSTELLINGS for the SETTINGS root) AND, on selection, dispatches the
      ROOT via the single pre-pass path (history/persisted value = root), with NO render-
      time reverse-derivation. Model it on the existing full-shell focus/menu tests under
      `shell/tests_focus` / `shell/tests_nav`. Annotate
      `// Validates: localization Req 13.4, 10.3; command-framework Req 18.1 (amended)`.
      Files: crates/ff-desktop/src/shell/ (new or existing full-shell test module, e.g.
      tests_nav.rs).
      Verify: `cargo test -p ff-desktop <new_test_name>` is RED (fails on the expected
      surface / dispatch assertion).

- [ ] 7. Make the menu option reference its alias by root + resolve-as-alias (GREEN for
      item 6). The persisted `MenuOption.command` stays the ROOT; at render resolve the
      active-locale in-row surface by that root (per-part fallback to root/English per
      Req 12.8) via the existing `menu_option_command_surfaces` path; on selection feed
      the referenced verb through the Req 2b resolver. Remove any bespoke render-time
      reverse-derivation special case in the menu render/selection seam if present.
      Confirm persisted identity = root (no saved-menu migration).
      Files: crates/ff-desktop/src/menu_workspace/render.rs,
      crates/ff-desktop/src/shell/commands_menu.rs (selection seam),
      crates/ff-desktop/src/shell/locale_resolver.rs.
      Verify: the item-6 full-shell test passes; `cargo test -p ff-desktop menu` green.

- [ ] 8. Write the failing per-locale store merge tests (RED) -- Req 13.5/13.6. Assert the
      EFFECTIVE table for an active locale+ENV is the shipped seed
      (`i18n/<locale>/aliases/<ENV>.toml`) overlaid by the USER per-locale override
      (`<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`), that the user
      override WINS on a shared surface, and that the old three-overlay stack is collapsed
      to one chain per environment-per-locale. Use a `TempDir` + the existing
      `FFWB_USER_ALIASES_*` override seam pattern (introduce a new env override for the
      per-locale override dir, e.g. `FFWB_USER_ALIASES_DIR`, mirroring
      `user_aliases_file`). Annotate `// Validates: localization Req 13.5, 13.6`.
      Files: crates/ff-desktop/src/shell/user_aliases.rs (path helper + test),
      crates/ff-desktop/src/shell/alias_reader.rs (store test),
      crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs (test).
      Verify: `cargo test -p ff-desktop override` / `cargo test -p ff-desktop store` RED.

- [ ] 9. Implement the one locale-aware store (GREEN for item 8). Add a per-locale user
      override path resolver in user_aliases.rs
      (`<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`, env-overridable
      for tests). Change `LocaleAliasStore::seed_and_overlay_for_locale` and the
      editor's `refresh_alias_store_in_place` (shell/alias_editor.rs) so the third layer
      is the per-LOCALE override (not the flat `user.toml`): seed -> per-locale seed
      overlay -> per-locale user override, user override wins. Keep reusing
      `apply_locale_overlay`/`apply_user_overlay_from_file` seams; one re-seed re-applies
      seed + override so a save goes live without restart. Adjust
      `alias_user_overlay.rs::apply_user_overlay_from_file`/`apply_user_rows` to read the
      per-locale override rows (surface/root/template; no `localisation_key`; the
      override rows are per-locale so `environment` is derived from the file name
      `<ENV>.toml`). Keep the shipped seed read-only (never written). Split any file that
      would exceed 400 non-test lines per rust-ffwb-layout.
      Files: crates/ff-desktop/src/shell/user_aliases.rs,
      crates/ff-desktop/src/shell/alias_reader.rs,
      crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs,
      crates/ff-desktop/src/shell/alias_editor.rs.
      Verify: `cargo check -p ff-desktop`; `cargo test -p ff-desktop override` and
      `store` green; `cargo clippy -p ff-desktop -- -D warnings` clean.

- [ ] 10. Confirm the shipped seed is never written by a save (Req 13.6/config Req 6.3).
      Add/adapt a test that a SAVE targets only the user per-locale override and the
      shipped `i18n/<locale>/aliases/<ENV>.toml` file content is byte-unchanged on disk
      after a save. Annotate `// Validates: localization Req 13.6; configurator Req 6.3`.
      Files: crates/ff-desktop/src/shell/alias_editor.rs (test module).
      Verify: `cargo test -p ff-desktop seed_not_written` passes.

- [ ] 11. Write the failing Alias-Editor active-locale-effective-view tests (RED) --
      configurator Req 6.1/6.2/6.6. Full-shell test: opening the Alias Editor under `af`
      SHOWS the shipped `af` surfaces (INSTELLINGS / KATALOGUSSE / LEERS / ...), the fix
      for "I don't see any Afrikaans aliases". Unit/row tests: rows present SURFACE / ROOT
      / ENV / TEMPLATE and NO L18N column; validation discipline (empty surface/root,
      within-env surface->two-roots collision, no-shadow built-in root, valid (env,root)
      via `contains_root`) applies to override rows; an override surface equal to a seed
      surface is a customisation (override wins), not a collision. Annotate
      `// Validates: configurator Req 6.1, 6.2, 6.6`.
      Files: crates/ff-desktop/src/shell/alias_editor.rs (tests),
      crates/ff-desktop/src/alias_editor_panel/render.rs (column test if applicable),
      crates/ff-desktop/src/shell/ (full-shell test).
      Verify: the new tests are RED.

- [ ] 12. Make the Alias Editor edit the active-locale effective table (GREEN for item 11,
      and Req 6.3/6.4/6.5, 13.7). Change `open_alias_editor` / `load_user_alias_rows` to
      load the EFFECTIVE active-locale table (seed overlaid by the per-locale override)
      for display, and `save_user_aliases` / the CR-CH-071 seam methods
      (`alias_editor_persist` / `alias_editor_discard` / `alias_editor_reload`) to write
      the per-locale USER override for the active locale+ENV, then refresh the live store
      in place so the menu reflects the edit after save. Drop the L18N column from the
      panel layout (alias_editor_panel/render.rs + state.rs `AliasRowEdit.localisation_key`
      and the heading) keeping Act / CE / Root / ALIAS / Parameters. Preserve CR-CH-071
      SAVE(stay) / END(save+leave) / CANCEL/RETURN(discard+leave) / REFRESH(discard+reload)
      over the override store. Keep each file under 400 non-test lines.
      Files: crates/ff-desktop/src/shell/alias_editor.rs,
      crates/ff-desktop/src/alias_editor_panel/render.rs,
      crates/ff-desktop/src/alias_editor_panel/state.rs,
      crates/ff-desktop/src/alias_editor_panel/rows.rs.
      Verify: item-11 tests green; `cargo test -p ff-desktop alias_editor` green;
      `cargo clippy -p ff-desktop -- -D warnings` clean.

- [ ] 13. Confirm DESCRIPTION / TITLE `.ftl` localisation is unchanged (Req 13.3). Add/keep
      a test that menu titles and option descriptions still resolve through the
      Catalogue_Lookup_Seam (`localised_option_description` / `menu_title_key`) against
      `af/menus.ftl`, i.e. only the `verb-*` command-surface indirection is removed.
      Annotate `// Validates: localization Req 13.3`.
      Files: crates/ff-desktop/src/menu_workspace/i18n_keys.rs (test module).
      Verify: `cargo test -p ff-desktop description` and `title` tests pass.

- [ ] 14. Migrate the af DATA to the one-layer shape (Req 13.9, 28.1). In
      `i18n/af/aliases/FFCMD.toml` and `i18n/af/aliases/FFEDIT.toml` delete the
      `localisation_key = "verb-*"` line from every `[[alias]]` row (the `surface` already
      holds the af text; FFEDIT keeps `SOEK -> FIND`). Delete the now-unused `verb-*`
      command-surface keys from `i18n/af/ff-desktop-verbs.ftl` and
      `i18n/en/ff-desktop-verbs.ftl` (both `verb-find` and the POM/Settings `verb-*`
      keys). `af/menus.ftl` and `en/menus.ftl` (descriptions/titles) UNCHANGED. These are
      DATA files (out of ASCII scope); do not touch the `.rs` here.
      Files: crates/ff-i18n/i18n/af/aliases/FFCMD.toml,
      crates/ff-i18n/i18n/af/aliases/FFEDIT.toml,
      crates/ff-i18n/i18n/af/ff-desktop-verbs.ftl,
      crates/ff-i18n/i18n/en/ff-desktop-verbs.ftl.
      Verify: `cargo test -p ff-desktop af_poc` and the full-shell af menu test
      (item 15) still pass with the `verb-*` keys gone; confirm no `.rs` references a
      removed key (`grep_search` for `verb-settings`/`verb-find` in `crates/ff-desktop`
      returns only removed/updated test references).

- [ ] 15. Write/confirm the no-regression test matrix proof (Req 13.9, task 29.1). A
      full-shell test (or small set) asserting: POM + Settings render fully in `af`
      (INSTELLINGS / KATALOGUSSE / LEERS / HULP / KONFIG / ALIASSE / TEMA / KIESLYSTE /
      SLEUTELS / SOORTE / HERSTEL KAAL; descriptions in af; mnemonics stay English); a
      typed af alias resolves to its root and dispatches via the single path; an unknown-
      in-active-env alias falls back to FFCMD then resolves/rejects; `&1` template still
      expands (ecob-style); the Alias Editor under `af` shows the effective table, an edit
      writes the user override, and after SAVE the menu reflects the edited surface. Reuse
      the shipped `i18n/` root and a `TempDir` override for the user store. Annotate each
      `// Validates: localization Req 13.9; command-environments Req 2b`.
      Files: crates/ff-desktop/src/shell/ (full-shell test module),
      crates/ff-desktop/src/shell/locale_resolver.rs (resolution legs if unit-level).
      Verify: `cargo test -p ff-desktop af` and the named matrix tests all green.

- [ ] 16. Full scoped verification sweep. Run `cargo fmt -p ff-desktop -- --check`,
      `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D warnings`, and the
      scoped CR-CH-072 test names (alias / resolve / menu / editor / af). Confirm no `.rs`
      exceeds 400 non-test lines (`rg --count-matches '' crates/ff-desktop/src/**/*.rs`
      style check per rust-ffwb-layout; split per the `_state`/`_render`/`_commands`/
      `_tests` rule if any file grew past the limit). Read all logs back.
      Files: none (verification only).
      Verify: fmt/check/clippy clean in the logs; scoped tests green; no file over 400
      non-test lines.

- [ ] 17. Flip the TCR rows and the change-log status (task 29.2). In
      docs/quality/TCR.md Phase (alias-model-simplification), change each covered 🔴
      NOT COVERED row to ✅ PASS and name the test file/function in the Test files column
      (mapping in the "TCR rows to flip" section above); update the Summary counts. Flip
      the CR-CH-072 Status in docs/status/change-log.md to reflect implementation landed.
      These are the main-workspace docs; edit the worktree copies.
      Files: docs/quality/TCR.md, docs/status/change-log.md (worktree copies).
      Verify: `grep_search` confirms 0 remaining 🔴 rows in the CR-CH-072 phase whose
      criterion has a landed test; the Summary PASS count matches the flipped rows.

## Notes, gaps, assumptions

- ASSUMPTION (OPEN DECISION 1, owner-confirmed default in Req 13.4 NOTE and the
  design): the user per-locale override path is
  `<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml` and the ENV is
  derived from the file name, so override rows need only surface/root/template
  (no per-row `environment`). If the implementer finds the existing
  `UserAliasRowToml.environment` field is relied on elsewhere, keep the field but
  default it from the file name rather than re-introducing a flat store.
- ASSUMPTION: the CR-NR-113 flat `user.toml` third-layer is REPLACED (not kept in
  parallel) by the per-locale override, per the design's "collapse ... to this
  one overlay chain". If a separate locale-independent user shorthand store is
  still required by an un-superseded CR-NR-113 criterion, raise it rather than
  silently keeping both -- but the gate's supersession table marks Req 5.6/5.7
  SUPERSEDED, so replacement is the expected path.
- Framework-conformance: single front door, active-wins, `pending_action` /
  `apply_<editor>_action` pattern, FFCMD/FFEDIT root sets + routing all unchanged;
  do NOT disturb CR-CH-066/067/068/069/070 beyond the intended alias rework. No
  new `CommandTarget` variant, no second dispatcher, no new nav stack.
- The `theme editor` full-shell test the owner saw fail
  (`full_shell_theme_editor_type_name_and_save_creates_user_theme`) is unrelated
  to this rework; do not modify it. If it is a flaky full-shell timing test it is
  out of scope (gate-policy flake handling).
- TDD discipline: for every code item, the matching test item (or the test lines
  within it) is written and confirmed RED before the implementation, then GREEN
  after. Items are ordered so the model change (1-2) precedes resolution (3-5),
  menu option (6-7), store (8-10), editor (11-12), descriptions (13), data
  migration (14), the matrix proof (15), verification (16), and coverage (17).
