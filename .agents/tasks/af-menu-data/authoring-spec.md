# Afrikaans Menu DATA -- Authoring Spec (POM + Settings, FULL localisation)

Read-only investigation. No code or data was changed. Every claim below is
grounded in live source at `c:\workspace\apps\FileForgeWorkbench` on `main`
(HEAD 4238c52). This spec tells the orchestrator EXACTLY which files and keys to
write so the POM and Settings menus render fully in Afrikaans -- command surface
AND mnemonic AND description for every option -- correct-by-construction.

---

## SUMMARY ANSWER (the one-paragraph model)

A rendered menu option has THREE independently-localised parts, each with its
own seam and data source:

1. **COMMAND surface** (the verb column, e.g. `Settings`, `Catalogs`): resolved
   from the per-locale ALIAS table. The renderer calls
   `AliasTable::display_surface_for_canonical(root)`. To show an Afrikaans verb
   you add an `[[alias]]` row to `crates/ff-i18n/i18n/af/aliases/FFCMD.toml`
   whose `canonical` is the option's ROOT (e.g. `SETTINGS`) and whose
   `localisation_key` points at a `verb-*` key you also add to a `.ftl`
   catalogue carrying the Afrikaans surface.
2. **MNEMONIC** (the key column, e.g. `0`, `C`): resolved from the Fluent
   CATALOGUE via key `menu-<menu-slug>-opt-<command-slug>-key`. This is
   **DISPLAY ONLY** -- selection matching always uses the stored English
   `option.key`. **Recommendation: do NOT author localised mnemonics** (see Q5);
   leave them to fall back to English so option selection cannot break.
3. **DESCRIPTION** (the prose column): resolved from the Fluent CATALOGUE via key
   `menu-<menu-slug>-opt-<command-slug>-desc`. You add these keys to
   `crates/ff-i18n/i18n/af/menus.ftl`.

Catalogue (`.ftl`) and ALIAS (`.toml`) are TWO DISTINCT data stores. Missing
data falls back PER PART to English (localization Req 12.8), so partial data
degrades gracefully and never blanks a column.

**Two gaps found that will make authored data not render / be incomplete:**
- `en/menus.ftl` is MISSING `menu-settings-opt-aliases-desc` (the CR-CH-066
  Aliases option). The English base itself lacks this key, so both en and af
  fall back to the parsed TOML prose for that one option. Authoring the af key
  alone will localise it in af but the en base key should also be added for
  completeness. See Q2/Q4.
- No `menu-*-opt-*-key` (mnemonic) keys exist in ANY locale. That is fine --
  mnemonics correctly fall back to English; see Q5 for why you should leave it so.

---

## Q1 -- THE THREE LOCALISATION SEAMS (function, key string, data source)

### (a) COMMAND surface
- Shell-side producer: `WorkbenchShell::menu_option_command_surfaces()`
  -- `crates/ff-desktop/src/shell/locale_resolver.rs:206-236`.
  For each option it looks up the active environment's `AliasTable` via
  `table.display_surface_for_canonical(&option.command)` and passes the result
  to the menu-crate helper.
- Menu-crate helper: `localised_option_command(option, display_surface_source)`
  -- `crates/ff-desktop/src/menu_workspace/i18n_keys.rs:150-159`. Returns the
  surface when present and non-blank, else the raw `option.command` (root)
  (localization Req 11.7/11.8 fallback).
- The surface itself: `AliasTable::display_surface_for_canonical`
  -- `crates/ff-desktop/src/shell/environment.rs` (AliasTable impl). It finds the
  FIRST entry whose `canonical` equals the root and returns
  `AliasEntry::display_surface()` -- which is `ff_i18n::t(localisation_key)` when
  the row carries a key, else the raw `surface` string
  (`AliasEntry::display_surface`, same file).
- **Data source:** a row in `i18n/<locale>/aliases/<ENV>.toml`, loaded by
  `shell/alias_reader.rs` (`read_catalogue_from_dir` ->
  `apply_locale_overlay_keyed`). The ROOT (canonical) stays the
  dispatched/persisted identity (Req 11.9); the surface is display-only.

KEY POINT: a root that has ONLY a synthesised identity row (no overlay row)
returns the raw root as its surface -- so with NO af alias row the command
column shows the English root verb (e.g. `Settings`). To show Afrikaans you MUST
add an overlay row (Q3).

### (b) MNEMONIC (key column)
- Shell-side producer: `WorkbenchShell::menu_option_mnemonic_surfaces()`
  -- `crates/ff-desktop/src/shell/locale_resolver.rs:256-275`. Derives the
  menu-name via `mw.menu_name_label()` then calls the menu-crate helper per
  option.
- Menu-crate helper: `localised_option_mnemonic(menu_name, option)`
  -- `crates/ff-desktop/src/menu_workspace/i18n_keys.rs:126-131`. Resolves key
  `menu_option_key_key(menu_name, option.command)` via `t_or`, falling back to
  the parsed English `option.key`.
- Key builder: `menu_option_key_key(menu_name, command)`
  -- `i18n_keys.rs:96-98` -> `format!("menu-{}-opt-{}-key", slug(menu_name), slug(command))`.
- **Data source:** a Fluent message id (the key) in a `.ftl` file under
  `i18n/<locale>/`.
- **Match vs display:** the STORED `option.key` is the match value; this derived
  text is DISPLAY only (docstring at `i18n_keys.rs:110-123`, and
  `menu_option_mnemonic_surfaces` docstring). See Q5.

### (c) DESCRIPTION
- Menu-crate helper: `localised_option_description(menu_name, option)`
  -- `i18n_keys.rs:102-107`. Resolves `menu_option_desc_key(...)` via `t_or`,
  falling back to parsed English `option.description`.
- Key builder: `menu_option_desc_key(menu_name, command)`
  -- `i18n_keys.rs:88-90` -> `format!("menu-{}-opt-{}-desc", slug(menu_name), slug(command))`.
- **Data source:** a Fluent message id in a `.ftl` file under `i18n/<locale>/`
  (specifically `menus.ftl`, where en/af/de already hold these keys).

### The `slug()` algorithm (authoritative)
`i18n_keys.rs:51-67`. For both the `-key` and `-desc` keys the slug is applied
to the menu NAME and to the option's ROOT `command`:
- lowercase every ASCII alphanumeric char;
- collapse every run of NON-alphanumeric chars (spaces, punctuation) to a single
  `-`;
- trim leading/trailing `-`.

Examples (from the unit tests in `i18n_keys.rs`):
- `"POM"` -> `pom`; `"Settings"` -> `settings`.
- `"Reset bare"` -> `reset-bare`; `"Reset  bare!"` -> `reset-bare`.
- `"Catalogs"` -> `catalogs`.
- `" Files / Folders "` -> `files-folders`.

### The menu-slug for built-ins is reliable
Even though built-in menus are code-only, the `MenuWorkspaceState` is ALWAYS
constructed with `file_path = menus_dir()/pom.toml` (or `settings.toml`) and
only the `.menu` field is swapped for the Recovery_Baseline
(`shell/commands_fastpath.rs:28-48`, `shell/nav_reconstruct.rs:108-128`). So
`menu_name_label()` (`menu_workspace/mod.rs:229-236`, returns the uppercased
file stem) yields `POM` / `SETTINGS`, and `slug()` lowercases them to
`pom` / `settings`. Keys are therefore stable: `menu-pom-...` / `menu-settings-...`.

### Concrete example keys (real options)
- POM option `0` / command `Settings`:
  desc key `menu-pom-opt-settings-desc`, mnemonic key `menu-pom-opt-settings-key`.
- POM option `1` / command `Catalogs`:
  desc key `menu-pom-opt-catalogs-desc`, mnemonic key `menu-pom-opt-catalogs-key`.
- Settings option `R` / command `Reset bare`:
  desc key `menu-settings-opt-reset-bare-desc`, mnemonic key `menu-settings-opt-reset-bare-key`.

---

## Q2 -- THE CATALOGUE DATA FORMAT + LOCATION

- **Format:** Fluent (`.ftl`). The key IS the Fluent message id; the value is the
  text after ` = `. Loaded by `crates/ff-i18n/src/catalogue.rs::build_bundle`,
  which merges EVERY `.ftl` file in `i18n/<locale>/` into one bundle (file name
  does not matter for lookup; only the message id does).
- **Location:** `crates/ff-i18n/i18n/<locale>/` (one dir per BCP-47 locale: `en`,
  `af`, `de`, `fr`). The English `en` tree is the Identity_Base and the fallback
  source (`catalogue.rs::resolve_args`: active bundle -> Identity_Base -> key).
- **The menu-option keys live in `menus.ftl`.** Existing en base
  (`crates/ff-i18n/i18n/en/menus.ftl`) holds:
  ```
  menu-pom-title = FileForge Workbench -- Primary Option Menu
  menu-pom-opt-settings-desc = FFWB Settings and Client Parameters
  menu-pom-opt-catalogs-desc = Virtual File Catalogs -- Mainframe, POSIX, Native
  menu-pom-opt-files-desc = File Explorer -- Browse catalogs and files in a tree view
  menu-pom-opt-help-desc = Help -- workbench help and topics
  menu-pom-opt-x-desc = Collapse to the visual root / close workspace (exit when last)
  menu-settings-title = Settings
  menu-settings-opt-config-desc = All settings -- browse every configuration key
  menu-settings-opt-theme-desc = Theme editor -- copy, edit, save and select themes
  menu-settings-opt-menus-desc = Menus editor -- create, change and save menus
  menu-settings-opt-keys-desc = Keys -- edit and save per-workspace key assignments
  menu-settings-opt-kinds-desc = Workspace kinds -- configure title, menu bar, key list, profile
  menu-settings-opt-reset-bare-desc = Reset to barebones -- archive config and start fresh
  ```
  The existing af base (`crates/ff-i18n/i18n/af/menus.ftl`) already mirrors all
  of these EXCEPT the Aliases option (see gap below).
- **On-disk syntax to add a key:** one line `key = value`, e.g.
  ```
  menu-settings-opt-aliases-desc = Alias-redigeerder -- skep, redigeer en stoor opdrag-aliasse
  ```
  Non-ASCII that could conflict with Fluent syntax uses the escape form the
  existing af/de files use: a Fluent string placeable, e.g. `kli{ "\u00eb" }nt`
  for the e-diaeresis. Plain accented UTF-8 bytes are also valid (see Q6).
- **GAP (en + af):** the Settings menu gained an `A` -> `Aliases` option
  (CR-CH-066, `menu_workspace/defaults.rs` DEFAULT_SETTINGS_TOML), but NEITHER
  `en/menus.ftl` NOR `af/menus.ftl` has `menu-settings-opt-aliases-desc`. Today
  that description falls back to the parsed TOML prose
  ("Alias editor -- create, edit and save command aliases"). To fully localise,
  add `menu-settings-opt-aliases-desc` to `af/menus.ftl` (and, for base
  completeness, `en/menus.ftl`).

Distinct from the ALIAS files: `.ftl` carries title/mnemonic/description and the
`verb-*` surface strings. The command-surface MAPPING (surface -> root) lives in
the TOML ALIAS files under `i18n/<locale>/aliases/` (Q3).

---

## Q3 -- THE COMMAND-SURFACE ALIAS DATA

- **File:** `crates/ff-i18n/i18n/af/aliases/FFCMD.toml`. (ALL POM and Settings
  option roots are FFCMD roots -- see Q4 -- so the POM/Settings command surfaces
  go in FFCMD.toml, NOT FFEDIT.toml. FFEDIT.toml holds editor verbs like the
  existing SOEK -> FIND.)
- **Row shape (per-locale file = the 4-field `AliasRowToml`,
  `shell/alias_reader.rs:48-62`):**
  ```toml
  [[alias]]
  surface = "INSTELLINGS"          # Afrikaans typeable/displayed surface (UPPERCASED on load)
  canonical = "SETTINGS"           # the REAL FFCMD root (UPPERCASED on load)
  localisation_key = "verb-settings"  # optional; resolves the DISPLAYED surface via t(key)
  # command_template = "..."       # optional (CR-NR-112); omit for a plain verb alias
  ```
  The per-locale reader deserialises `surface`, `canonical`,
  `localisation_key?`, `command_template?`. (This is NOT the user-store 5-field
  `UserAliasRowToml`; the per-locale path uses the 4-field shape above.)
- **Two ways to supply the Afrikaans surface text:**
  1. With `localisation_key = "verb-settings"` AND a matching `.ftl` key
     `verb-settings = Instellings` -- the display surface is `t("verb-settings")`
     resolved under the active locale. This is the pattern the shipped
     SOEK -> FIND POC uses (`verb-find = SOEK` in `af/ff-desktop-verbs.ftl`).
     RECOMMENDED: add the `verb-*` keys to `crates/ff-i18n/i18n/af/ff-desktop-verbs.ftl`
     (and the identity `verb-* = <ROOT>` to `en/ff-desktop-verbs.ftl`).
  2. WITHOUT a `localisation_key` -- the raw `surface` string is displayed
     verbatim. Simpler (no `.ftl` round-trip) but the surface only shows under
     the af locale's table; the `en` display stays the root. Acceptable because
     the af aliases file is only loaded when af is the active locale.
- **`display_surface_for_canonical` returns the FIRST entry matching the
  canonical, and the keyed overlay inserts overlay rows at the FRONT**
  (`AliasTable::apply_locale_overlay_keyed`, `environment.rs`), so an authored af
  row is preferred over the synthesised identity row. Good.
- **The canonical MUST be a REAL FFCMD root.** The per-locale reader itself does
  NOT enforce this (Q6), but to render the command column it must match an entry
  the table was seeded with (the identity rows are seeded from
  `builtin_ffcmd_roots()`, which includes every menu-option root). Use the EXACT
  root casing/spelling from Q4 (e.g. `RESET BARE`, two words).

**Example row (maps Afrikaans surface to a canonical FFCMD menu root):**
```toml
# i18n/af/aliases/FFCMD.toml
[[alias]]
surface = "KATALOGUSSE"
canonical = "CATALOGS"
localisation_key = "verb-catalogs"
```
with `verb-catalogs = Katalogusse` in `af/ff-desktop-verbs.ftl` and
`verb-catalogs = CATALOGS` in `en/ff-desktop-verbs.ftl`.

(Existing precedent for a non-keyed FFCMD row is in the alias_reader tests:
`surface = "KATALOGUSSE"`, `canonical = "CATALOGS"`.)

---

## Q4 -- THE EXACT CURRENT MENU ROOT SET (post CR-CH-066/069/070)

From `crates/ff-desktop/src/menu_workspace/defaults.rs` (DEFAULT_POM_TOML and
DEFAULT_SETTINGS_TOML), confirmed by the parse/order unit tests in the same file.

### POM (`menu-pom`), menu-slug `pom`, group `Core` for all
| key (mnemonic) | command (root) | command-slug | description (English, current) |
|---|---|---|---|
| `0` | `Settings` | `settings` | FFWB Settings and Client Parameters |
| `1` | `Catalogs` | `catalogs` | Virtual File Catalogs -- Mainframe, POSIX, Native |
| `2` | `Files` | `files` | File Explorer -- Browse catalogs and files in a tree view |
| `3` | `Help` | `help` | Help -- workbench help and topics |
| `X` | `X` | `x` | Collapse to the visual root / close workspace (exit when last) |

(`X` option has `show_in_menu_bar = false`.)

### Settings (`menu-settings`), menu-slug `settings`
| key | command (root) | command-slug | group | description (English, current) |
|---|---|---|---|---|
| `C` | `Config` | `config` | Core | All settings -- browse every configuration key |
| `A` | `Aliases` | `aliases` | Core | Alias editor -- create, edit and save command aliases |
| `T` | `Theme` | `theme` | Core | Theme editor -- copy, edit, save and select themes |
| `M` | `Menus` | `menus` | Core | Menus editor -- create, change and save menus |
| `K` | `Keys` | `keys` | Core | Keys -- edit and save per-workspace key assignments |
| `W` | `Kinds` | `kinds` | Core | Workspace kinds -- configure title, menu bar, key list, profile |
| `R` | `Reset bare` | `reset-bare` | Recovery | Reset to barebones -- archive config and start fresh |

Canonical ROOT verbs (as uppercased for alias `canonical`): `SETTINGS`,
`CATALOGS`, `FILES`, `HELP`, `X`, `CONFIG`, `ALIASES`, `THEME`, `MENUS`, `KEYS`,
`KINDS`, `RESET BARE`. All are FFCMD roots -- confirmed present in
`command_config/mod.rs::BUILTIN_WORKSPACE_VERB_ROOTS` (CONFIG, KEYS, THEME,
FILES, CATALOGS, KINDS, MENUS, ALIASES) plus the two built-in menus' option
roots seeded by `builtin_ffcmd_roots()` (`shell/alias_reader.rs:152-186`), which
unions both menus' option commands (SETTINGS, HELP, X, RESET BARE) with the verb
lists. So every menu-option root is a seeded FFCMD root and a valid `canonical`.

(12 options total: 5 POM + 7 Settings.)

---

## Q5 -- THE MNEMONIC CAVEAT (match vs display)

- **Match uses the STORED English `option.key`, never the localised mnemonic.**
  The derived mnemonic from `menu_option_key_key` is DISPLAY ONLY. Evidence:
  - `i18n_keys.rs:110-123` docstring: "Selection matching still uses the stored
    `option.key`, never this derived display text."
  - localization Req 12.3 (`docs/specs/localization/requirements.md:538-545`):
    the mnemonic is localisable DISPLAY with a per-part fallback; the match/stored
    value is the Locale_Independent_Stored_Value (Req 12.4). CR-NR-111 fixed the
    CODE contract only.
- **Therefore you CAN author localised mnemonics safely** (selection will not
  break -- the user still types the English key), BUT:
- **RECOMMENDATION: do NOT author localised mnemonics.** Reasons:
  1. The displayed mnemonic would then differ from the key the user must type,
     which is confusing (display says one letter, selection needs another).
  2. No `menu-*-opt-*-key` keys exist in any locale today, so omitting them is
     the already-tested, graceful path: `localised_option_mnemonic` falls back
     per-option to the English `option.key` (unit test
     `localised_mnemonic_falls_back_to_parsed_english_on_miss`, `i18n_keys.rs`).
  3. The owner asked for a fully Afrikaans menu, but the mnemonic is a single
     matchable character; keeping it English preserves keyability while command +
     description carry the visible Afrikaans.

  If the orchestrator nonetheless wants localised mnemonics, author
  `menu-<menu-slug>-opt-<command-slug>-key = <char>` in `af/menus.ftl` -- but
  prefer keeping the SAME character as English so display and match agree.

---

## Q6 -- LOADER VALIDATION THAT COULD REJECT OR IGNORE THE AF DATA

### contains_root is NOT run on the per-locale alias path
Confirmed. `EnvironmentRegistry::contains_root` is called ONLY by:
- `shell/alias_editor.rs:249` (Alias Editor SAVE path, CR-CH-067 Req 20.8),
- `alias_editor_panel/rows.rs:72` and `render.rs:182` (Alias Editor UI),
- `shell/alias_reader/alias_user_overlay.rs:83` (the USER alias store overlay,
  Req 20.7).

The PER-LOCALE shipped reader path -- `read_catalogue_from_dir` ->
`apply_overlay_from_dir` -> `apply_locale_overlay_keyed` (all in
`shell/alias_reader.rs` / `shell/environment.rs`) -- does NOT call
`contains_root`. So the shipped `i18n/af/aliases/FFCMD.toml` is NOT rejected for
an "invalid root". (This matches the CR-CH-069 note that the per-locale reader
did not run contains_root; still true on main.) Consequence: if you misspell a
`canonical`, the row loads silently but the command column will NOT change
(because `display_surface_for_canonical` finds no matching seeded root for that
misspelling, OR adds a surface for a root no option uses). So correctness depends
on YOU matching the exact Q4 roots -- the loader will not warn.

### The only per-locale rejection is a within-environment COLLISION
`apply_locale_overlay_keyed` rejects (and retains the Identity_Base for that one
environment, logging a WARN) when one surface maps to TWO different canonicals
within the same file (`alias_reader.rs` tests
`reader_rejects_within_environment_collision_and_retains_base`). So: do not map
the same Afrikaans surface to two roots, and do not reuse a surface that already
resolves to a different root in the base table.

### ASCII rules -- Afrikaans non-ASCII values ARE allowed in these DATA files
Per `.kiro/steering/documentation-charset.md` "Out of scope (CR-CH-064)": the
per-locale localization DATA -- per-locale message catalogues (`.ftl` under
`i18n/<locale>/` for non-`en` locales) and the per-locale alias-surface DATA
(`i18n/<locale>/aliases/*.toml`) -- are OUT of ASCII scope and MAY contain
non-ASCII UTF-8; they need only be valid UTF-8. STILL IN ASCII SCOPE: the English
`i18n/en/` catalogue and all Rust source. So:
- af values in `af/menus.ftl`, `af/ff-desktop-verbs.ftl`, and
  `af/aliases/FFCMD.toml` MAY use accented Afrikaans letters (e, e-diaeresis,
  etc.). The existing af/de files use the Fluent escape form `{ "\u00ea" }` --
  you may follow that or use raw UTF-8; both are valid Fluent.
- Any NEW `en` base keys you add (e.g. `menu-settings-opt-aliases-desc`,
  `verb-*` identities) MUST stay ASCII.

### Per-part, per-option fallback (partial data degrades gracefully)
Confirmed (localization Req 12.8, CR-NR-111):
- Missing desc key -> `t_or` returns parsed English `option.description`
  (`i18n_keys.rs::t_or`, test `t_or_falls_back_to_english_when_key_absent`).
- Missing mnemonic key -> falls back to English `option.key`
  (test `localised_mnemonic_falls_back_to_parsed_english_on_miss`).
- Missing/absent alias row -> command column shows the raw root
  (`localised_option_command` None/blank branch, test
  `localised_option_command_resolves_alias_surface_else_root`).
So you can author parts incrementally; each missing part independently shows
English, never blank. The `t()` seam also falls back af -> en -> key
(`catalogue.rs::resolve_args`).

### A `.ftl` parse error is recoverable, not fatal
`build_bundle` surfaces a parse error as a recoverable `I18nError`; a duplicate
message id within a locale's merged bundle is a bundle-build error. So: do NOT
define the same `menu-*` / `verb-*` key twice across the af `.ftl` files (they
all merge into one af bundle).

---

## CONCLUSIONS -- the exact authoring worklist (fill in Afrikaans values next)

### File A -- `crates/ff-i18n/i18n/af/menus.ftl` (DESCRIPTIONS; already exists)
Add the ONE missing description key (gap), then the file covers all 12 options:
```
menu-settings-opt-aliases-desc = <AF: Alias editor -- create, edit and save command aliases>
```
(The other 11 desc keys already exist in af/menus.ftl and are translated. Verify
they still match the current English prose in Q4; they do as of HEAD.)

Optional mnemonic keys (NOT recommended, see Q5). If authored, 12 keys of the
form `menu-pom-opt-<slug>-key` / `menu-settings-opt-<slug>-key`.

### File B -- `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` (COMMAND surfaces; exists, currently no rows)
Add one `[[alias]]` row per menu-option root you want shown in Afrikaans. ROOTS
(exact `canonical` values): `SETTINGS`, `CATALOGS`, `FILES`, `HELP`, `X`,
`CONFIG`, `ALIASES`, `THEME`, `MENUS`, `KEYS`, `KINDS`, `RESET BARE`. Pattern
(keyed form recommended):
```toml
[[alias]]
surface = "<AF-SURFACE-UPPER>"
canonical = "<ROOT>"
localisation_key = "verb-<root-slug>"
```
Note the `X` root: its Afrikaans surface is a judgment call (it is a close/exit
affordance); you may leave `X` un-aliased so it shows `X` in every locale.

### File C -- `crates/ff-i18n/i18n/af/ff-desktop-verbs.ftl` (verb surfaces; exists, has verb-find)
Add one `verb-<root-slug> = <AF surface>` per row in File B that uses a
`localisation_key`. Example: `verb-settings = Instellings`.

### File D -- `crates/ff-i18n/i18n/en/ff-desktop-verbs.ftl` (en identity; exists, has verb-find = FIND)
For EACH new `verb-*` key, add the identity `verb-<root-slug> = <ROOT>` (ASCII)
so the en display stays the English root and the key is never missing in the
base. Example: `verb-settings = SETTINGS`.

### File E -- `crates/ff-i18n/i18n/en/menus.ftl` (en base completeness; exists)
Add `menu-settings-opt-aliases-desc = Alias editor -- create, edit and save command aliases`
(ASCII) so the English base carries every current option's description key.

### Things that would make authored data NOT render (flags)
1. **Misspelled `canonical`** in FFCMD.toml -- silently ignored (no contains_root
   on this path); command column stays English. Use exact Q4 roots.
2. **Wrong alias file** -- POM/Settings roots are FFCMD; putting them in
   FFEDIT.toml will not localise the menu command column (the active environment
   for a menu Context is FFCMD, `locale_resolver.rs::active_environment_name`).
3. **Duplicate Fluent key** across af `.ftl` files -- bundle-build error for af;
   the whole af overlay may fail to load. Keep each key unique.
4. **Surface collision** -- one surface mapped to two roots in FFCMD.toml rejects
   the whole FFCMD af overlay (retains English). Keep surfaces unique per root.
5. **Localised mnemonic that differs from the English key** -- renders fine but
   the displayed letter will not match the key the user must type (match uses the
   stored English `option.key`). Prefer not to localise mnemonics.
6. **Non-ASCII in an `en` base file** -- violates documentation-charset ASCII
   scope and would fail the charset check. Keep en values ASCII.

### Fallback safety net
Every part falls back independently to English (desc -> parsed prose; mnemonic ->
`option.key`; command -> raw root; catalogue -> af then en then key). So the menu
always renders; incomplete af data simply shows English for the missing part.

---

## EVIDENCE INDEX (file:line / symbol)
- `crates/ff-desktop/src/menu_workspace/defaults.rs` -- DEFAULT_POM_TOML (lines ~34-70),
  DEFAULT_SETTINGS_TOML (lines ~90-140); order/root unit tests confirm Q4.
- `crates/ff-desktop/src/menu_workspace/i18n_keys.rs` -- `slug` (51-67),
  `menu_option_desc_key` (88-90), `menu_option_key_key` (96-98),
  `localised_option_description` (102-107), `localised_option_mnemonic` (126-131),
  `localised_option_command` (150-159), `t_or` (34-43).
- `crates/ff-desktop/src/shell/locale_resolver.rs` -- `menu_option_command_surfaces`
  (206-236), `menu_option_mnemonic_surfaces` (256-275), resolver order (module docs).
- `crates/ff-desktop/src/shell/environment.rs` -- `AliasEntry::display_surface`,
  `AliasTable::display_surface_for_canonical`, `apply_locale_overlay_keyed`
  (front-insert + collision), `ffedit_english`, `identity_for`.
- `crates/ff-desktop/src/shell/alias_reader.rs` -- `AliasRowToml` (48-62),
  `parse_alias_toml`, `read_catalogue_from_dir`, `builtin_ffcmd_roots` (152-186),
  `LocaleAliasStore::seed_and_overlay_for_locale`; tests confirm no contains_root
  and collision rejection.
- `crates/ff-desktop/src/command_config/mod.rs` -- `BUILTIN_WORKSPACE_VERB_ROOTS`
  (173), `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (218).
- `crates/ff-desktop/src/shell/environment_registry.rs` -- `root_commands` (303-),
  `contains_root` (334-); tests for FFCMD/FFEDIT root placement.
- `crates/ff-desktop/src/menu_workspace/mod.rs` -- `menu_name_label` (229-236),
  `MenuWorkspaceState.file_path` (118).
- `crates/ff-desktop/src/shell/commands_fastpath.rs` (28-48) +
  `shell/nav_reconstruct.rs` (108-128) -- built-in menus keep file_path = pom/settings.toml.
- `crates/ff-i18n/src/catalogue.rs` -- `build_bundle` (merges all .ftl),
  `resolve_args` (active -> base -> key fallback).
- `crates/ff-i18n/src/seam.rs` -- `t` / `t_args` (seam, key-on-miss).
- `crates/ff-i18n/i18n/en/menus.ftl`, `af/menus.ftl`, `de/menus.ftl` -- existing
  desc/title keys; af/de use `{ "\u00XX" }` escapes; aliases-desc MISSING in en+af.
- `crates/ff-i18n/i18n/en|af/ff-desktop-verbs.ftl` -- `verb-find` (en = FIND,
  af = SOEK) precedent.
- `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` (currently no rows) and `FFEDIT.toml`
  (SOEK -> FIND, keyed verb-find).
- `docs/specs/localization/requirements.md` -- Req 12 (493-613): full-localisation
  CODE contract, mnemonic per-locale DISPLAY source + fallback (538-545), DATA
  declared out of ASCII scope and a separate deliverable (606-613).
- `.kiro/steering/documentation-charset.md` -- CR-CH-064 out-of-scope: per-locale
  `.ftl` and `aliases/*.toml` may be non-ASCII; en base + Rust stay ASCII.
