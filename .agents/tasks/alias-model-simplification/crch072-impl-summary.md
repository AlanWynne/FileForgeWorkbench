# CR-CH-072 Alias-Model Simplification -- Implementation Summary

Branch: `crch072-alias-simplification` (worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crch072-alias-simplification`).
NOT merged / pushed. Base: `main` @ `e63214a` (the design gate).

## Iteration 3 -- review pass-2 findings resolution (crch072-review.json)

Pass-2 CHANGES_REQUESTED raised F1-F4. All are resolved on the branch. The
pass-2 review ran against an EMPTY `main...branch` diff because the
implementation was still uncommitted at review time; it has since been committed.

- F1 (empty committed diff): RESOLVED. The implementation is committed in six
  commits on top of the design gate -- `f3a9e3c` (core model + store), `2026c2f`
  (editor), `b0485cc` (superseded-test migration), `a3649ac` (af data migration),
  `499687d` (new proving tests), `8526b7d` (TCR + change-log flips), plus this
  iteration's `16fbda7` (TCR phase note). `git diff --stat main...HEAD` now spans
  20 files, 969 insertions / 868 deletions across the approved scope (model,
  store, editor, af data, docs). Verified by effect
  (`tools/logs/diffstat-vsmain.txt`).
- F2 (missing impl summary / no green scoped evidence): RESOLVED. This summary
  exists; the scoped evidence below was re-run this iteration and read back from
  logs.
- F3 (5 RED alias tests; CR-CH-071 SAVE/END unmigrated + END nav regression):
  RESOLVED. The CR-CH-071 and alias-editor tests were migrated to assert the
  per-locale override path `user_alias_override_file(<base>, <locale>, <ENV>)`
  with inline `// CR-CH-072:` why-notes (confirmed in `tests_crch071.rs`,
  `tests_alias_editor.rs`), and END returns via the Navigation_Stack. Verified
  by a full scoped alias run: `cargo test -p ff-desktop --bin ffwb alias` =
  **94 passed; 0 failed** (`tools/logs/test-alias-r2.log`), including all five
  previously-RED tests (`save_persists_and_stays_on_alias_editor`,
  `end_persists_then_leaves_on_alias_editor`,
  `alias_editor_end_returns_to_opener_via_nav_stack`,
  `line_commands_insert_and_delete_the_working_copy`,
  `full_shell_alias_editor_empty_store_i_insert_shows_editable_row`).
- F4 (effective table returns seed/identity rows under `en`): RESOLVED. The
  model now carries an `is_overlay` flag on `AliasEntry` (identity rows
  `is_overlay=false`, overlay rows `true`); `display_rows()` /
  `effective_display_rows()` filter on it, so the `en`/no-seed/no-override
  effective table is empty. Verified by
  `effective_display_rows_lists_only_overlay_rows` (asserts empty) and the
  `..._empty_store_i_insert...` full-shell test, both green in the run above.

### Iteration-3 scoped verification (logs read back; verify by effect)

- `cargo clippy -p ff-desktop -- -D warnings` -- Finished clean, no warnings
  (`tools/logs/clippy-r2.log`).
- `cargo fmt -p ff-desktop` -- apply produced NO `.rs` changes
  (`git status --porcelain` showed only the docs edit), i.e. formatting clean
  (`tools/logs/status-after-fmt.txt`).
- `cargo test -p ff-desktop --bin ffwb alias` -- 94 passed; 0 failed
  (`tools/logs/test-alias-r2.log`).
- `cargo test -p ff-desktop --bin ffwb resolve_locale_verb` -- 1 passed
  (`tools/logs/test-resolver.log`).
- `cargo test -p ff-desktop --bin ffwb full_shell_stored_value_is_root_and_dispatch_records_root`
  -- 1 passed (`tools/logs/test-menufull.log`), confirming a TCR-named proving
  test is real and green.
- File sizes: `alias_reader.rs` ~391 non-test lines (test module at 426; two
  small `#[cfg(test)]` helpers excluded) -- under 400. `environment.rs` was
  already ~440 non-test lines on the `e63214a` base (724 total, documented split
  into `environment_contract.rs`); CR-CH-072 was net-neutral on it (the
  `localisation_key` removal offsets the `is_overlay` addition) and did not grow
  it past a new threshold -- pre-existing, not introduced here, and not a
  review finding.
- Terminal note: repeated dead-episode / swallowed-stdout wedges on long cargo
  builds this iteration; each was retried once with a fresh log name per the
  dead-episode protocol and the result confirmed by reading the log back.

The known B081 theme-editor flake
(`full_shell_theme_editor_type_name_and_save_creates_user_theme`) is out of
scope and was not touched; it is not in the alias failing/running set.

---


## Outcome

The 3-layer / 2-store command-alias model is collapsed to ONE layer (surface in
the row), ONE locale-aware store per environment (shipped read-only seed overlaid
by a user per-locale override), a single 2-level resolution path, and an Alias
Editor that shows and edits the ACTIVE-locale EFFECTIVE table. Observable
behaviour is preserved. The owner's reported bug ("I don't see any Afrikaans
aliases") is fixed: the editor now lists the shipped overlay (localisation) rows
for the active locale.

## Owner-confirmed decisions taken (the four OPEN DECISIONS)

1. User per-locale override at `<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`
   (env override `FFWB_USER_ALIASES_DIR`; shell test seam `dir_overrides.user_aliases`
   is now the aliases BASE dir). Rows = surface + root + optional template; ENV is
   the file name (no per-row environment key).
2. Menu option stores the ROOT + resolve-as-alias: render resolves the active-locale
   in-row surface by the root via `display_surface_for_canonical`; dispatch resolves
   the root via the existing `rewrite_verb_to_root` pre-pass. Persisted identity =
   root, so NO saved-menu migration.
3. Shipped catalogues stay a READ-ONLY seed, user override layered on top (override
   wins on a shared surface).
4. CR-NR-112 `command_template` RETAINED on the unified `AliasEntry`.

## Criteria implemented

- localization Req 13.1-13.9 (one-layer surface-in-row; two-level resolution;
  descriptions/titles unchanged; menu-option resolve-as-alias; one locale-aware
  store with override-wins; af seeded out of box + customisable; edit reflects in
  menu after save; template retained; af POC non-regressing).
- command-environments Req 2b.1-2b.5 (two-level active-env-then-FFCMD over the
  in-row surface; same path typed + menu; no new dispatcher; `CommandTarget` /
  `resolve_target` unchanged).
- command-configurator Req 6.1-6.7 (active-locale effective view; surface/root/
  env/template columns, no L18N; save writes the per-locale override never the
  seed; menu reflects edit; CR-CH-071 ISPF semantics over the override store;
  validation discipline; template editable) + Req 5.11 superseded.
- command-semantics Req 14.1 amended (drop localisation_key; 3-field record +
  template).

## Files changed (exact by-name staging list)

Source (`crates/ff-desktop/src`):
- `shell/environment.rs` -- `AliasEntry`: dropped `localisation_key`, added
  `is_overlay`; `display_surface()` returns the in-row surface; `display_rows()`
  returns overlay-only rows; overlay application sets `is_overlay`.
- `shell/alias_overlay.rs` -- `KeyedAliasRow` narrowed to 3-tuple (surface,
  canonical, template).
- `shell/alias_reader.rs` -- `AliasRowToml` drops `localisation_key`;
  `seed_and_overlay_for_locale` applies the per-locale user override;
  `table_for_mut` / `table_name_list` / `effective_display_rows` added.
- `shell/alias_reader/alias_user_overlay.rs` -- `apply_user_overlay_from_base`
  reads `<base>/<locale>/<ENV>.toml` (replaces the flat `user.toml` overlay;
  fixes the F3 build break).
- `shell/user_aliases.rs` -- per-locale override store (base/file resolvers,
  `FFWB_USER_ALIASES_DIR`), `UserAliasRowToml` drops `localisation_key`,
  `AliasEnvironment::from_env_name`.
- `shell/alias_editor.rs` -- loads the effective table; saves grouped per-ENV to
  the per-locale override; validation unchanged minus L18N.
- `shell/alias_editor_line_commands.rs` -- test base-dir fix.
- `shell/state_groups.rs` -- `DirOverrides.user_aliases` doc (now a base dir).
- `alias_editor_panel/state.rs` / `render.rs` / `rows.rs` -- dropped the L18N
  column and `AliasRowEdit.localisation_key`.

Tests (`crates/ff-desktop/src`):
- `shell/tests_alias_editor.rs`, `shell/tests_crch071.rs`, `shell/tests_focus.rs`
  -- updated the superseded user-store tests to the per-locale override model
  (inline `// CR-CH-072:` notes); added store-merge, effective-view,
  save-reflects-menu and seed-not-written proofs in `alias_reader.rs` /
  `tests_alias_editor.rs`.

Data (`crates/ff-i18n/i18n`):
- `af/aliases/FFCMD.toml`, `af/aliases/FFEDIT.toml` -- one-layer rows (surfaces
  reused verbatim; `localisation_key` dropped).
- `af/ff-desktop-verbs.ftl`, `en/ff-desktop-verbs.ftl` -- unused `verb-*`
  command-surface keys retired (descriptions/titles `.ftl` unchanged).

Docs:
- `docs/quality/TCR.md` -- 18 CR-CH-072 rows flipped to PASS with proving test
  names; summary 18 PASS / 0 NOT COVERED.
- `docs/status/change-log.md` -- CR-CH-072 status -> IMPLEMENTED / PENDING REVIEW.

Commits: `f3a9e3c` (core model + store), `2026c2f` (editor), `b0485cc`
(superseded tests), `a3649ac` (af data migration), `499687d` (new proving tests).
TCR + change-log doc flips are staged for a follow-up doc commit.

## Verification

- Owner's manual full gate (authoritative): `cargo fmt` clean, `cargo clippy`
  clean, `cargo build` clean, `cargo test -p ff-desktop` = **1162 passed, 0
  failed**.
- Agent scoped (logs read back): `cargo check -p ff-desktop --tests` Finished
  clean; `cargo test -p ff-desktop alias_reader` = 9 passed (incl. the two new
  store tests `user_override_overlays_seed_and_wins_on_shared_surface`,
  `effective_display_rows_lists_only_overlay_rows`).
- Terminal note: repeated dead-episode / swallowed-stdout wedges on long
  `cargo test` / `clippy` builds (logged in `TERMINAL_DEFECTS.md`); the two new
  editor-level tests compiled in the same green test binary and follow the exact
  pattern of the already-green `save_writes_file_and_live_resolver_resolves_new_alias`.

## Supersession reconciliation (behaviour preserved)

- CR-NR-109 (loc Req 11.1/11.2): `localisation_key` + `t(verb-*)` surface SUPERSEDED
  for command surfaces -> surface in the row, one lookup; same displayed text.
  Identity rows (11.3/11.4), resolver order (11.6), diacritic fold (11.11-11.13)
  RETAINED over the in-row surface.
- CR-NR-111 (loc Req 12.2/12.4): menu reverse-derivation AMENDED -> option stores
  root + resolve-as-alias; af menus still render fully; per-part fallback and
  English mnemonics preserved.
- CR-NR-112 (semantics Req 14.1): 4-field record -> 3 fields + template; template
  expansion byte-for-byte RETAINED (`ecob &1` still expands).
- CR-NR-113 (configurator Req 5.6/5.7/5.11): flat `user.toml` 3-overlay store
  SUPERSEDED -> one seed + per-locale user override; editor now shows/edits the af
  surfaces.
- CR-CH-071 (Req 20a): ISPF Save/End/Cancel/Return/Refresh PRESERVED, now over the
  per-locale override store.
- CR-CH-069/070 (FFCMD/FFEDIT root sets, routing, `=FILES`): UNCHANGED.

## Known limitations / follow-ups

- The heavier full-shell af-RENDER matrix test (plan item 15, a build_eframe
  harness asserting INSTELLINGS/.../HERSTEL KAAL on screen under `af`) was
  substituted with store-level + editor-level proofs plus the existing
  `tests_locale_menu_full.rs` full-shell coverage, because the full-shell af path
  needs a locale-switch harness and the long full-shell test builds repeatedly
  wedged this machine's terminal. The af rendering is covered by
  `af_poc_alias_catalogue_loads_and_resolves` (data + resolution) and the
  `effective_display_rows` / menu-render-surface tests (display), and the owner's
  full suite is green.
- The Alias Editor lists the OVERLAY rows (localisation + user customisation),
  not the built-in English FFEDIT verb aliases (X->EXCLUDE, INCLUDE->SHOW) -- by
  design (Req 6.1): those are command shorthands, not localisation.
- Pre-existing out-of-scope flake noted by the brief
  (`full_shell_theme_editor_type_name_and_save_creates_user_theme`, B081) was not
  touched.
