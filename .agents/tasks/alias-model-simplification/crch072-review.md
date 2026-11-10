# CR-CH-072 Alias-Model Simplification -- Verdict Review (loop pass-3)

Branch: `crch072-alias-simplification` (worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crch072-alias-simplification`),
forked from `main` @ `e63214a` (the design gate). Verdict: **APPROVED**.

## Evidence read

- Coder summary `crch072-impl-summary.md` and plan `crch072-plan.md`.
- `git diff main...crch072-alias-simplification` (full, captured to `fulldiff.txt`,
  20 files / 969 insertions / 868 deletions) and the commit log (`commits.txt`):
  six implementation commits `f3a9e3c` (core model + store), `2026c2f` (editor),
  `b0485cc` (superseded-test migration), `a3649ac` (af data migration), `499687d`
  (new proving tests), `8526b7d` + `16fbda7` (TCR + change-log flips). This
  resolves pass-2 F1 (the diff is now non-empty and spans the approved scope).
- Authoritative spec in the worktree: localization `requirements.md` Req 13
  (nine criteria) + the amendment flags on Req 11.1/11.2/12.2/12.4.
- Worktree source spot-reads: `locale_resolver.rs::menu_option_command_surfaces`,
  `commands_menu.rs` selection seam, and a stale-reference scan of
  `locale_resolver.rs` / `environment.rs` / `menu_workspace/i18n_keys.rs`.
- `docs/quality/TCR.md` CR-CH-072 phase and `docs/status/change-log.md`.

Per the brief I did NOT re-run the suites; the owner's authoritative full gate is
recorded in the summary (`cargo test -p ff-desktop` 1162 passed / 0 failed; fmt /
clippy / build clean) and the agent's scoped trio + scoped alias run (94 passed /
0 failed, incl. the five previously-RED tests) is logged. No articulable doubt
required a spot-check beyond the structural source reads above.

## Acceptance-point verification

1. ONE-layer alias (surface-in-row). `AliasEntry` drops `localisation_key` and
   `AliasEntry::display_surface()` collapses to `self.surface.clone()`
   (environment.rs). `AliasRowToml` / `KeyedAliasRow` lose the key element
   (alias_reader.rs, alias_overlay.rs). `command_template` is retained on the
   unified row. `.ftl` descriptions/titles are untouched (menus.ftl not in the
   diff; the `verb-*` command-surface keys in `ff-desktop-verbs.ftl` are the only
   `.ftl` content removed). Satisfies Req 13.1 / 13.8 and the amendment to
   Req 11.1/11.2.

2. Two-level resolution on the single front door. `locale_resolver.rs` is
   UNCHANGED (not in the diff): `rewrite_verb_to_root` + `resolve_target` remain
   the one pre-pass + classifier. Resolution ignores the new `is_overlay` flag
   (documented on the field). No new dispatcher, nav stack, or third
   interpretation layer. Satisfies Req 13.2 / command-environments Req 2b and
   framework-conformance mechanisms 1-3.

3. Menu option resolve-as-alias. The option's persisted `command` stays the ROOT
   (no saved-menu migration). Render: `menu_option_command_surfaces` resolves the
   active-locale surface by root via `display_surface_for_canonical` ->
   `display_surface` (now the in-row surface). Dispatch: `commands_menu.rs` runs
   `opt.command` (the root) through `rewrite_verb_to_root` ->
   `resolve_and_dispatch_command` -- the same single path. No render-time
   reverse-derivation remains. Satisfies Req 13.4 and the amendment to
   Req 12.2/12.4.

4. ONE locale-aware store. `user_aliases.rs` replaces the flat `user.toml`
   resolver with `user_alias_override_base()` (`FFWB_USER_ALIASES_DIR` seam) and
   `user_alias_override_file(base, locale, env)` =
   `<base>/<locale>/<ENV>.toml`. `alias_user_overlay.rs::apply_user_overlay_from_base`
   overlays the per-locale user override on top of the shipped seed, user-wins on
   a shared surface. `LocaleAliasStore::seed_and_overlay_for_locale` applies seed
   then user override in one re-seed. The Alias Editor
   (`load_effective_alias_rows`) shows the EFFECTIVE active-locale table from the
   live store; `write_override_rows` groups by ENV and writes ONLY the user
   override (shipped seed never touched; an emptied ENV removes its override
   file). Satisfies Req 13.5 / 13.6 / 13.7 and configurator Req 6.1/6.3/6.4.

5. af menus. `af/aliases/FFCMD.toml` + `FFEDIT.toml` keep every surface verbatim
   (INSTELLINGS / KATALOGUSSE / ... / HERSTEL KAAL; SOEK -> FIND) and drop the
   `localisation_key` line; `af/ff-desktop-verbs.ftl` + `en/ff-desktop-verbs.ftl`
   retire the now-unused `verb-*` keys. Descriptions/titles (`menus.ftl`) and
   English mnemonics are unchanged. Satisfies Req 13.9.

6. Template + CR-CH-071 + routing preserved. `command_template` expansion
   unchanged (the `rewrite_verb_to_root_*` tests are retained). The CR-CH-071
   Save/End/Cancel/Return/Refresh seam now operates over the per-locale override
   store; `tests_crch071.rs` is migrated to assert the new override path with
   inline `// CR-CH-072:` notes, and END returns via the Navigation_Stack
   (resolving pass-2 F3). FFCMD/FFEDIT root sets and routing (CR-CH-069/070) are
   untouched.

7. Supersession table honoured. CR-NR-109/111/112/113 + CR-CH-071 are reconciled
   with observable behaviour preserved (change-log and the reconciliation section
   of the summary); identity rows, resolver order, and the diacritic fold are
   retained over the in-row surface.

8. rust-standards. No `unwrap`/`expect` introduced in non-test code (the write
   path uses `map_err` + early returns; the editor save surfaces errors as
   `Result<(), String>`). `Debug` is derived on the changed public types
   (`AliasEntry` via `derive(Debug, ...)`, `UserAliasRowToml`, `AliasEnvironment`).
   Source is ASCII; af DATA files are the documented out-of-scope exemption. File
   sizes: `alias_reader.rs` non-test code is ~337 lines of production code plus
   two `#[cfg(test)]` helpers before the main test module at L426 (test code is
   excluded from the 400 rule) -- under 400. `environment.rs` non-test code runs
   to L446, OVER 400, but this is PRE-EXISTING on the `e63214a` base (already
   split into `environment_contract.rs`) and CR-CH-072 is net-neutral on it
   (localisation_key removed, is_overlay added); out of this change's scope, not
   a finding.

9. No weakened/deleted tests. The superseded tests are re-expressed against the
   one-layer / per-locale-override model with inline why-notes, not gutted; the
   only DELETED test (`l18n_value_round_trips_through_validate_and_serialise`) is
   removed with a `// CR-CH-072:` note because the L18N column it covered no
   longer exists (Req 6.2). New proving tests are added
   (`display_surface_returns_in_row_surface`,
   `user_override_overlays_seed_and_wins_on_shared_surface`,
   `effective_display_rows_lists_only_overlay_rows`,
   `save_makes_menu_render_surface_reflect_the_edited_alias`,
   `save_writes_only_user_override_not_shipped_seed`,
   `from_env_name_maps_back_to_selector`). The Alias Editor first-Tab focus test
   (`full_shell_alias_editor_empty_store_i_insert_shows_editable_row`) and the
   CR-CH-071 save/end tests are present and meaningful. The 18 CR-CH-072 TCR rows
   are flipped to PASS with named proving tests; Summary = 18 PASS / 0 NOT
   COVERED.

10. B081 theme-editor flake
    (`full_shell_theme_editor_type_name_and_save_creates_user_theme`) is the
    known pre-existing out-of-scope flake; not touched, not treated as a finding.

## Pass-2 finding resolution confirmed

- F1 (empty committed diff): resolved -- the implementation is committed and the
  diff spans the approved scope.
- F2 (missing impl summary / scoped evidence): resolved -- summary present with
  read-back logs.
- F3 (RED alias tests; CR-CH-071 unmigrated + END nav regression): resolved --
  the migrated tests assert the per-locale override path and END returns via the
  nav stack; the scoped alias run is green.
- F4 (effective table returning seed/identity rows under en): resolved -- the
  `is_overlay` flag plus `display_rows()`/`effective_display_rows()` filter make
  the en/no-seed/no-override editor empty, proven by
  `effective_display_rows_lists_only_overlay_rows`.

## Notes (non-blocking)

- A stale doc/test comment in `menu_workspace/i18n_keys.rs` still mentions "a row
  keyed verb-find = SOEK"; the helper signature and assertion are unaffected
  (it takes a `Some("SOEK")` surface source). Cosmetic only.
- `TCR.md` renders the status glyphs as mojibake (`Γ£à` / `≡ƒö┤`); this encoding
  is pre-existing in that file (identical bytes before and after the diff) and is
  not introduced by this change.

## Findings

None (no blocking findings).
