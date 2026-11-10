# Findings: Afrikaans menu options render in English (CR-NR-109, Req 11.7)

## Summary answer (verdict first)

The root cause is **(a) missing alias DATA, not a code/wiring bug.** The
locale-aware alias layer for the menu-kind command column is wired correctly
end to end: the render path computes a per-option display surface from the
active environment's active-locale `AliasTable`, the store is re-seeded and
overlaid when `ui.locale` changes, and the display accessor returns a localised
surface whenever a localised alias row exists. It renders English for the
built-in POM/Settings options because **the only Afrikaans alias row shipped on
main is `SOEK -> FIND` (FFCMD), and `FIND` is not a command of any built-in menu
option.** Every built-in menu option command (`Settings`, `Catalogs`, `Files`,
`Help`, `X`, `Config`, `Theme`, `Menus`, `Keys`, `Kinds`, `Reset bare`)
therefore resolves only to its synthesised English identity row and displays the
English root surface. There is no `af/aliases/FFEDIT.toml` and no `af` rows for
any menu-option root. The fix is a **gated DATA authoring task** (author Afrikaans
alias rows with localisation keys for the menu-option roots); no code change is
required for the mechanism to work.

---

## Evidence

### Q1. Does `render_menu_workspace` consult the localised surface, or format `option.command` raw?

It consults a precomputed localised surface. The renderer takes a
`command_surfaces: &[String]` parameter and, per option, prefers that slice over
the raw command:

- `crates/ff-desktop/src/menu_workspace/render.rs:66-73` -- `render_menu_workspace(..., command_surfaces: &[String])`.
- `render.rs` (option loop) -- `let command_surface = command_surfaces.get(opt_index).map(String::as_str).unwrap_or(option.command.as_str());` then `option_prefix_job(option, command_surface, ...)` builds the key+command prefix from `command_surface`, not the raw command.

The surfaces are computed shell-side and passed in by the body arm:

- `crates/ff-desktop/src/shell/render_body_arms.rs` (`render_body_menu_workspace`)
  -- `let command_surfaces = self.menu_option_command_surfaces();` then
  `render_menu_workspace(mw, ui, calendar_offset, menu_cal, &command_surfaces)`.

The shell-side computation is the production caller of the display accessor:

- `crates/ff-desktop/src/shell/locale_resolver.rs:146-170` -- `menu_option_command_surfaces()`:
  locks `self.i18n.alias_store`, gets `table_for(env_name)` for the active
  environment, and for each option does
  `table.display_surface_for_canonical(&option.command)` then
  `i18n_keys::localised_option_command(option, source.as_deref())`.

Conclusion: the render path is correctly wired to the alias surface. **Not a
render bug.**

### Q2. Built-in menu option command values vs available `af` alias rows

Built-in option commands (read from the compiled constants in
`crates/ff-desktop/src/menu_workspace/defaults.rs`):

- `DEFAULT_POM_TOML` options: `Settings`, `Catalogs`, `Files`, `Help`, `X`.
- `DEFAULT_SETTINGS_TOML` options: `Config`, `Theme`, `Menus`, `Keys`, `Kinds`, `Reset bare`.

(Confirmed by the module's own tests `recovery_pom_menu_has_barebones_options`
and `recovery_settings_menu_has_barebones_options`.)

Shipped `af` alias DATA (`crates/ff-i18n/i18n/af/aliases/FFCMD.toml`): exactly one
row --

```toml
[[alias]]
surface = "SOEK"
canonical = "FIND"
localisation_key = "verb-find"
```

The file's own header states the POC scope is "a single reviewed row
(SOEK -> FIND)". There is **no** `af/aliases/FFEDIT.toml`.

Cross-check: `FIND` is **not** among the built-in menu-option commands above.
Therefore **every** built-in menu option (POM and Settings) necessarily falls
back to English, because no `af` alias row exists for any of their root commands.
(Even an `af` FIND menu option would localise, but none of the built-in menus
has a FIND option.)

### Q3. Does the display accessor require a localisation key + catalogue entry to return a non-English surface?

Yes, effectively. Trace:

- `crates/ff-desktop/src/shell/environment.rs` -- `display_surface_for_canonical(canonical)`
  finds the first entry whose `canonical` matches (case-insensitive) and returns
  `AliasEntry::display_surface()`. Returns `None` only when no row matches the
  canonical at all.
- `AliasEntry::display_surface()` (same file): `Some(key) => ff_i18n::t(key)`,
  `None => self.surface.clone()`.

For a built-in menu option (e.g. `Settings`), the FFCMD table was seeded with an
**identity row** via `AliasTable::identity_for(ffcmd_roots)` -- surface
`"SETTINGS"`, canonical `"SETTINGS"`, `localisation_key = None`
(`builtin_ffcmd_roots()` uppercases each option command). Because no `af` overlay
row exists for `SETTINGS`, the identity row is unchanged, so
`display_surface_for_canonical("Settings")` returns `Some("SETTINGS")` -- the raw
English surface. `localised_option_command` then shows `"SETTINGS"` (a non-empty
`Some`), which is English.

So a canonical with **no keyed alias row** at locale `af` yields the English
identity surface. A localised surface appears **only** when an overlay row for
that canonical carries a `localisation_key` whose `t(key)` resolves in the active
catalogue (the `SOEK -> FIND` row's `verb-find` is the one shipped example, and
it is keyed to `FIND`, not to a menu option). This behaviour is correct per the
design (Req 11.8 fallback to root), just starved of data.

### Q4. Is the live store actually re-seeded on `ui.locale` change, and does render read the active store/locale?

Yes, both.

- Seed + publish at construction: `crates/ff-desktop/src/shell/construct.rs`
  (the `locale_alias_store` block) calls
  `LocaleAliasStore::seed_and_overlay_for_locale(&root_refs, &help_locale)`,
  wraps it in `Arc<Mutex<..>>`, and calls `publish_active_alias_store(&shared)`.
- Reload wiring: `crates/ff-desktop/src/main.rs` registers an
  `on_locale_reload` callback that calls
  `shell::alias_reader::reload_active_alias_store_for_locale(&help_locale)`.
- `reload_active_alias_store_for_locale` (`crates/ff-desktop/src/shell/alias_reader.rs`)
  locks the published Arc, reads back the existing FFCMD root set
  (`ffcmd_roots()`), and reassigns `*guard = LocaleAliasStore::seed_and_overlay_for_locale(&root_refs, locale)`
  -- an in-place swap every Arc holder (the shell) observes.
- `seed_and_overlay_for_locale` calls `apply_overlay_from_dir(<root>/<locale>/aliases/)`,
  which reads `<env>.toml` per environment (`FFCMD.toml`, `FFEDIT.toml`) by the
  exact stable name.

The render path reads the SAME shell-owned `self.i18n.alias_store` (the published
Arc) and uses `self.active_environment_name()` (FFCMD for a menu Context) -- so it
reads the active store for the active locale. The store is **not** English-only by
accident; at `af` it is correctly re-seeded and overlaid with whatever `af` data
exists -- which is just `SOEK -> FIND`.

Therefore after switching to `af` the FFCMD table holds: identity rows for all
menu-option roots (English surfaces, no key) PLUS the one overlaid `FIND` row
carrying `verb-find` (which displays `SOEK`). No menu option references `FIND`, so
no menu option changes.

### Q5. Verdict

**(a) Purely missing alias DATA.** The code is correct:

- Render consults the localised surface (Q1).
- The display accessor returns a localised surface when a keyed overlay row
  exists, and the English root otherwise, exactly as Req 11.7/11.8 specify (Q3).
- The store is re-seeded/overlaid on `ui.locale` change and the render reads the
  active store/locale (Q4).

The English display is produced solely because the shipped `af` alias set has one
row (`SOEK -> FIND`) and none of the built-in menu-option roots has an `af` alias
row with a localisation key (Q2). The richer 21-row Afrikaans alias set that
would have covered the menu options existed on the retired `feature/l10n-phase4`
worktree branch and was deliberately not merged, which is exactly why the
canonical `main` implementation shows English menu options.

### Q6. Recommended fix (scoped to verdict (a): author alias DATA)

This is a **gated DATA authoring task** (translated alias data is content, and
authoring/altering it is gated per the localization spec; it is not a code
change). To make the Afrikaans menu options display Afrikaans surfaces:

1. Add Afrikaans FFCMD alias rows for every built-in menu-option root in
   `crates/ff-i18n/i18n/af/aliases/FFCMD.toml`, each with a `localisation_key`,
   covering the roots:
   - POM: `Settings`, `Catalogs`, `Files`, `Help` (and optionally the `X`
     terminate verb).
   - Settings: `Config`, `Theme`, `Menus`, `Keys`, `Kinds`, `Reset bare`.
   Each row is `surface = "<Afrikaans surface, UPPERCASE>"`,
   `canonical = "<root, e.g. SETTINGS>"`, `localisation_key = "<catalogue key>"`.
   The surface must resolve to the SAME canonical as the English identity row
   (so the overlay UPDATES the identity row in place via
   `apply_locale_overlay_keyed`, Req 11.4 -- not a collision).
2. Add the corresponding Afrikaans catalogue entries for each `localisation_key`
   in the `af` message catalogue (the `.ftl`/catalogue the `ff_i18n::t(key)` seam
   reads), so `display_surface()` resolves the key to the Afrikaans surface rather
   than falling back to the key string. Without the catalogue entry, `t(key)`
   returns the key itself and the column would show the raw key, not Afrikaans.
3. (Optional, for the editor Context) add `crates/ff-i18n/i18n/af/aliases/FFEDIT.toml`
   for FFEDIT verbs if editor-Context localised surfaces are also wanted; not
   required for the POM/Settings menu bug.

No code defect to fix. The mechanism already renders the overlaid surface the
moment the data (alias row + catalogue key) is present, as proven by the
existing `SOEK -> FIND` path and the unit test
`localised_option_command_resolves_alias_surface_else_root`
(`crates/ff-desktop/src/menu_workspace/i18n_keys.rs`).

Caveat to confirm during the DATA task: built-in menu option commands are mixed
case in the TOML (e.g. `Settings`, `Reset bare`) while `builtin_ffcmd_roots()`
seeds identity canonicals UPPERCASED. Matching is case-insensitive
(`display_surface_for_canonical` / `apply_locale_overlay_keyed` use
`eq_ignore_ascii_case`), so an overlay row with `canonical = "SETTINGS"` will
correctly match the `Settings` option. Author canonicals in uppercase to match
the seeded identity rows and the shipped `SOEK -> FIND` convention.

---

## Files inspected (with the key symbols)

- `crates/ff-desktop/src/menu_workspace/render.rs` -- `render_menu_workspace`, `command_surfaces` consumption.
- `crates/ff-desktop/src/shell/render_body_arms.rs` -- `render_body_menu_workspace` computes and passes `command_surfaces`.
- `crates/ff-desktop/src/shell/locale_resolver.rs` -- `menu_option_command_surfaces`, `display_surface_for_canonical` call site.
- `crates/ff-desktop/src/menu_workspace/i18n_keys.rs` -- `localised_option_command` (surface-or-root fallback).
- `crates/ff-desktop/src/shell/environment.rs` -- `AliasEntry::display_surface`, `AliasTable::display_surface_for_canonical`, `apply_locale_overlay_keyed`, `identity_for`.
- `crates/ff-desktop/src/shell/alias_reader.rs` -- `LocaleAliasStore::seed`, `seed_and_overlay_for_locale`, `apply_overlay_from_dir`, `read_catalogue_from_dir`, `builtin_ffcmd_roots`, `ACTIVE_ALIAS_STORE`, `publish_active_alias_store`, `reload_active_alias_store_for_locale`.
- `crates/ff-desktop/src/shell/construct.rs` -- seed + publish of the live store.
- `crates/ff-desktop/src/main.rs` -- `on_locale_reload` -> `reload_active_alias_store_for_locale` wiring.
- `crates/ff-desktop/src/menu_workspace/defaults.rs` -- `DEFAULT_POM_TOML` / `DEFAULT_SETTINGS_TOML` option commands.
- `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` -- the single shipped `af` row (`SOEK -> FIND`).

Investigation was read-only; no code was modified.
