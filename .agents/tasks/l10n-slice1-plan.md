# Implementation Plan -- Localisation Phase 2, Slice 1 (CR-NR-103, L10N.4)

Code-mode against already-approved criteria (requirements gate CLOSED). Scope is
EXACTLY three items, in order: Task 4 loose end, Task 5 (menu title/description
localisation), Task 6 (help per-locale directories). Build ON existing seams
only; add NO new mechanism (Req 10.1); behaviour-neutral at `ui.locale = "en"`
(Req 10.2). ASCII only; no source file over 400 non-test lines.

All paths are absolute inside the worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice1`. Use
`git -C <that path>` for all git.

## Investigation findings (grounding the decisions)

- `ff-i18n` is complete: the `t()` / `t_args()` seam (`crates\ff-i18n\src\seam.rs`)
  resolves against a process-wide swappable `Catalogue`; `Catalogue::load_identity_base`
  loads `crates\ff-i18n\i18n\en\*.ftl`; a missing key returns the KEY string plus
  a WARN (NOT the English prose).
- CRITICAL: `ff_i18n::set_catalogue` has NO production caller anywhere -- only
  `crates\ff-i18n\src\tests.rs` installs a catalogue. `ff-desktop` does not even
  depend on `ff-i18n`. So the seam is installed NOWHERE in the live app. Task 5
  (the first real `t()` consumer) MUST first wire a startup bootstrap that
  resolves the shipped `i18n/` root and calls `set_catalogue`, plus register the
  `ui.locale` reload callback (`ff_config::on_locale_reload`, already present).
  This bootstrap is prerequisite infrastructure for Task 5 and for Task 6's
  hot-reload aspect.
- Task 4's loader MECHANISM is DONE and fully unit-tested: `AliasTable::apply_locale_overlay`
  (`crates\ff-desktop\src\shell\environment.rs`) and the `AliasCatalogue` loader
  (`crates\ff-desktop\src\shell\alias_overlay.rs`). Its own module scope note
  states the on-disk catalogue reader and the live locale-switch wiring are
  DEFERRED to Phase 3 (Task 10), and both carry `#[allow(dead_code)]` "no caller
  yet". Subtasks 4.1-4.3 are checked; only the PARENT `[ ]` is unticked.
  Requirement 5 is satisfied by the loader + overlay logic proven with fixtures;
  Req 5 does NOT require live locale-switch wiring (that is Phase 3). Therefore
  Task 4 is a BOOKKEEPING loose end: tick the parent. Wiring the loader into the
  live per-environment table would be Phase 3 scope and is explicitly excluded by
  "Do NOT expand beyond Requirement 5."
- Menu model: `crates\ff-desktop\src\menu_workspace\mod.rs` (`MenuFile.title`,
  `MenuOption.command`, `MenuOption.description`). Built-in content is code-only
  TOML in `crates\ff-desktop\src\menu_workspace\defaults.rs` (`DEFAULT_POM_TOML`,
  `DEFAULT_SETTINGS_TOML`). The title is displayed via `title_line_text`
  (`crates\ff-desktop\src\shell\mod_helpers.rs`, the `TabKind::MenuWorkspace` arm
  reads `mw.menu_title()`); option descriptions are displayed in
  `crates\ff-desktop\src\menu_workspace\render.rs` (two sites: the row `Label`
  from `option.description`, and `result.focused_option`).
- Help loader: `crates\ff-help\src\content_loader.rs` resolves the FIRST existing
  directory from an ordered `search_paths` list and discovers `.help.md` files
  recursively. Startup builds the search paths in
  `crates\ff-desktop\src\shell\construct.rs` (exe-dir `help/`, exe-dir
  `..\..\help`, user-data `help/`). `Topic_Key` identifiers live in the parsed
  content, not the path, so per-locale directory resolution does not touch keys.

## Design decisions (made here; no design doc supplied)

1. Catalogue key scheme (stable identifiers, never translated strings, Req 4.4):
   - Menu title: `menu-<menuname>-title` (menuname = file stem lowered: `pom`,
     `settings`).
   - Option description: `menu-<menuname>-opt-<command-slug>-desc`, where
     command-slug = the option `command` lowercased with non-alphanumerics
     collapsed to `-` (e.g. `Reset bare` -> `reset-bare`). The `command` VALUE is
     the key-building input but is itself NEVER localised (Req 7.3, 4.2). Fluent
     keys are ASCII kebab-case, consistent with the existing `ok-button` style.
2. Miss fallback to parsed English (Req 7.4, 3.2): `t(key)` returns the KEY on a
   total miss, not the English prose. So the render-time helper calls `t(key)`
   and, when the result equals `key` (miss sentinel), falls back to the parsed
   `MenuFile` text (the compiled English). A tiny `t_or(key, fallback)` helper in
   ff-desktop encapsulates this. This keeps the English catalogue the source for
   present keys and the parsed TOML the source for absent keys, so menus always
   render.
3. Localise at render time out of the PARSED model (Req 7.1): do NOT mutate
   `MenuFile`, do NOT ship translated TOML. The title site (`title_line_text`)
   and the description sites (`render.rs`) call the helper. Because the helper
   needs the menu-name, pass it through: `title_line_text` already has the
   `MenuWorkspaceState` (which exposes `menu_name_label()`); the render path has
   `state.menu` and the backing file path.
4. i18n root resolution mirrors the help search-path pattern (Req 2.2): resolve
   the first existing `i18n/` dir from exe-dir `i18n/`, exe-dir `..\..\i18n`
   (dev `target/<profile>` -> crate `crates\ff-i18n\i18n` is NOT on that path;
   instead add `..\..\..\crates\ff-i18n\i18n` dev fallback), and user-data
   `i18n/`. On a resolve/load failure, log a WARN and leave the seam uninstalled
   (it degrades to key fallback, which decision 2 turns into parsed-English), so
   startup never fails. A new `crates\ff-desktop\src\shell\i18n_bootstrap.rs`
   module owns this (keeps `construct.rs` under 400 lines).
5. Help per-locale resolution (Req 6.1, 6.2): add a locale-aware variant to
   `ContentLoader` that, given a base search path and an active locale tag,
   probes `<base>\<locale>` then `<base>` (English fallback) per base path. Keep
   `load_all` unchanged for the English-only call; add a new entry point so the
   topic model and parsing are untouched (Req 6.1). Hot-reload (Req 6.4): the
   shell re-resolves and reloads the registry inside the `ui.locale` callback.
6. Active-locale source: read `ui.locale` from the `ConfigHandle` at bootstrap
   and in the reload callback. With no user override the effective value is `"en"`
   (Req 1.3), so decisions 4 and 5 both resolve English and behaviour is identical
   (Req 10.2).

---

## Plan

- [ ] 1. Tick the Task 4 parent checkbox (bookkeeping loose end; Req 5 already met).
      Change `- [ ] 4. Per-environment, per-locale alias-catalogue LOADER` to
      `- [x] 4. ...` in the tasks file. Subtasks 4.1-4.3 are already `[x]` and the
      loader + overlay logic (`alias_overlay.rs`, `AliasTable::apply_locale_overlay`)
      fully satisfy Requirement 5 with fixture-proven tests; live locale-switch
      wiring is Phase 3 (Task 10), out of scope.
      Files: `c:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice1\docs\specs\localization\tasks.md`
      Verify: re-read the file and confirm line 4 is `- [x] 4.` and no other
      marker changed; `git -C <worktree> diff --stat docs\specs\localization\tasks.md`
      shows a one-line change.

- [ ] 2. Add the `ff-i18n` dependency and the startup catalogue bootstrap.
      Add `ff-i18n = { path = "../ff-i18n" }` to ff-desktop's `Cargo.toml`. Create
      `i18n_bootstrap.rs` with `resolve_i18n_root()` (first existing of the three
      probed dirs per decision 4) and `install_catalogue(config: &ConfigHandle)`
      that reads `ui.locale`, loads `Catalogue::load_identity_base(root)`, applies
      `load_overlay` when the locale is not `en` (WARN + retain base on error,
      Req 1.6/2.4), and calls `ff_i18n::set_catalogue`. Call `install_catalogue`
      from `main.rs` right after schema registration (step 2a) and BEFORE the
      shell is built. Register `ff_config::on_locale_reload(config.callbacks(), ...)`
      so a `ui.locale` change re-runs `install_catalogue` (Req 1.4, 1.5).
      Files: `...\crates\ff-desktop\Cargo.toml`,
      `...\crates\ff-desktop\src\shell\i18n_bootstrap.rs`,
      `...\crates\ff-desktop\src\shell\mod.rs` (add `mod i18n_bootstrap;`),
      `...\crates\ff-desktop\src\main.rs`.
      TDD FIRST: in `i18n_bootstrap.rs` `#[cfg(test)]`, write
      `install_catalogue_with_en_locale_resolves_a_shipped_key`
      (`// Validates: Requirement 3.2, 10.2`) -- point a test root at a temp
      `i18n/en` with one key, call the install path with `ui.locale="en"`, assert
      `ff_i18n::t("that-key")` resolves to the English value; and
      `resolve_i18n_root_returns_none_when_no_dir_exists`
      (`// Validates: Requirement 2.4`). Deterministic (temp dirs, no clock).
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo test -p ff-desktop i18n_bootstrap *> tools\logs\t2.log"`
      then read `tools\logs\t2.log` -- new tests pass, crate compiles.

- [ ] 3. Add the English catalogue keys for menu titles and option descriptions.
      Append to the shipped English `.ftl` (DATA only; English base, NO fr/de) the
      keys for both built-in menus per decision 1: `menu-pom-title`,
      `menu-pom-opt-settings-desc`, `menu-pom-opt-catalogs-desc`,
      `menu-pom-opt-files-desc`, `menu-pom-opt-help-desc`, `menu-pom-opt-x-desc`;
      `menu-settings-title`, `menu-settings-opt-config-desc`,
      `menu-settings-opt-theme-desc`, `menu-settings-opt-menus-desc`,
      `menu-settings-opt-keys-desc`, `menu-settings-opt-kinds-desc`,
      `menu-settings-opt-reset-bare-desc`. Each value is the EXACT current English
      prose from `defaults.rs` (so `t()` returns byte-identical text, Req 10.2).
      ASCII only (use `--` for the POM title dash).
      Files: `...\crates\ff-i18n\i18n\en\main.ftl` (or a new
      `...\crates\ff-i18n\i18n\en\menus.ftl`; the loader merges all `.ftl` in the
      dir -- prefer `menus.ftl` to keep concerns separated).
      TDD FIRST: in `crates\ff-i18n\src\tests.rs` add
      `shipped_base_resolves_menu_title_and_option_keys`
      (`// Validates: Requirement 7.1, 7.4`) loading the real shipped `i18n/`
      root and asserting `resolve("menu-pom-title")` equals the POM banner text
      and `resolve("menu-settings-opt-reset-bare-desc")` equals its English prose.
      Verify: `cargo test -p ff-i18n *> tools\logs\t3.log` then read the log --
      new test passes; confirm the `.ftl` is ASCII with
      `Select-String -Path crates\ff-i18n\i18n\en\*.ftl -Pattern '[^\x00-\x7F]'`
      returning no matches.

- [ ] 4. Add the `t_or` render helper and localise the menu TITLE.
      Add `pub(crate) fn t_or(key: &str, fallback: &str) -> String` (decision 2:
      `t(key)`, if it equals `key` return `fallback.to_string()`) and
      `menu_title_key(menu_name: &str)` + `menu_option_desc_key(menu_name, command)`
      slug builders (decision 1) in a new `...\crates\ff-desktop\src\menu_workspace\i18n_keys.rs`.
      In `title_line_text` (`mod_helpers.rs`), for the `TabKind::MenuWorkspace`
      arm, when a menu + menu-name are present, return
      `t_or(&menu_title_key(name), &raw_title)` instead of the raw title.
      Files: `...\crates\ff-desktop\src\menu_workspace\i18n_keys.rs`,
      `...\crates\ff-desktop\src\menu_workspace\mod.rs` (add `pub mod i18n_keys;`),
      `...\crates\ff-desktop\src\shell\mod_helpers.rs`.
      TDD FIRST: in `i18n_keys.rs` tests, `option_desc_key_slugs_command`
      (`// Validates: Requirement 7.1`) asserting `menu_option_desc_key("settings","Reset bare")`
      == `menu-settings-opt-reset-bare-desc`, and
      `t_or_falls_back_to_parsed_english_on_miss` (`// Validates: Requirement 7.4, 3.2`)
      asserting a bogus key returns the fallback. In `tests_focus.rs` /
      `tests_nav.rs` keep the existing `title_line_pom_tab_shows_menu_title_not_banner`
      and `pom_and_settings_title_line_derive_from_loaded_menu_title` GREEN (they
      assert the exact English banner, which the installed en catalogue must
      reproduce -- this IS the behaviour-neutrality check for the title).
      Verify: `cargo test -p ff-desktop title_line *> tools\logs\t4.log` then read
      the log -- existing title tests still pass, new key tests pass.

- [ ] 5. Localise the option DESCRIPTIONS at the render sites.
      In `crates\ff-desktop\src\menu_workspace\render.rs`, derive the menu-name
      once from the backing state and replace `option.description.clone()` at BOTH
      display sites (the row `Label` `desc`, and the `result.focused_option`
      description) with `t_or(&menu_option_desc_key(name, &option.command),
      &option.description)`. Do NOT change `option.command` usage anywhere
      (Req 7.3). If render.rs would exceed 400 non-test lines, move the per-row
      localisation into a small helper in `i18n_keys.rs` per the rust-standards
      split order.
      Files: `...\crates\ff-desktop\src\menu_workspace\render.rs`
      (and `i18n_keys.rs` if a helper is extracted).
      TDD FIRST: add a pure unit test on the extracted helper (not egui): e.g.
      `localised_description_uses_catalogue_then_parsed_fallback`
      (`// Validates: Requirement 7.1, 7.4`) -- install a catalogue with one
      option-desc key, assert that option returns the catalogue text and an
      uncovered option returns its parsed English. Keep the existing menu-render
      egui_kittest tests GREEN (behaviour-neutrality, Req 10.2) -- with en
      installed every description equals the parsed English.
      Verify: `cargo test -p ff-desktop menu_workspace *> tools\logs\t5.log` then
      read the log -- new helper test passes, existing render/focus tests pass.

- [ ] 6. Add locale-aware path resolution to the `ff-help` content loader.
      In `crates\ff-help\src\content_loader.rs` add
      `resolve_help_directory_for_locale(&self, locale: &str) -> Option<PathBuf>`
      and `load_all_for_locale(&self, locale) -> Result<ContentLoadResult, HelpError>`
      that, for each base search path, probe `<base>\<locale>` first then `<base>`
      (English fallback), with NO change to `discover_files`, `ContentParser`, or
      the topic model (Req 6.1, 6.3). `Topic_Key` and cross-references stay
      untouched because they come from parsed content, not the path.
      Files: `...\crates\ff-help\src\content_loader.rs`
      (split a `_locale.rs` sibling if it crosses 400 non-test lines).
      TDD FIRST (`// Validates: Requirement 6.1, 6.2`): in that file's tests,
      `locale_directory_resolves_when_present` (temp `help\fr` exists -> picks it),
      `locale_directory_falls_back_to_english_when_absent` (no `help\fr` -> picks
      `help`), and `english_locale_resolves_base_directory_unchanged` (locale
      `"en"` -> same dir `load_all` would pick, the behaviour-neutrality check,
      Req 10.2). Deterministic (TempDir, no clock/network).
      Verify: `cargo test -p ff-help content_loader *> tools\logs\t6.log` then
      read the log -- new tests pass, existing `resolve_help_directory_*` tests
      unchanged and green.

- [ ] 7. Wire the shell to load help for the active locale and re-resolve on reload.
      In `crates\ff-desktop\src\shell\construct.rs`, read `ui.locale` from config
      and call `loader.load_all_for_locale(&locale)` instead of `load_all()` when
      building the help registry (English stays the effective locale by default,
      so the resolved content is identical -- Req 10.2). Extend the `ui.locale`
      reload callback from item 2 (or add a sibling registration) so that on a
      locale change the shell re-resolves and reloads the help registry, so
      subsequently displayed topics use the new locale (Req 6.4). Keep this small;
      if it pushes construct.rs over 400 lines, factor the help-registry build
      into a helper (e.g. in `help.rs`).
      Files: `...\crates\ff-desktop\src\shell\construct.rs`
      (and `...\crates\ff-desktop\src\shell\help.rs` if a helper is factored).
      TDD FIRST (`// Validates: Requirement 6.4, 10.2`): because registry reload
      is shell-entangled, add a focused test on the factored helper (seed two
      temp dirs `help\` and `help\fr\`, build the registry for `en` then `fr`,
      assert the topic set switches). If a headless shell harness is needed and
      not available, assert on the extracted helper rather than the full shell,
      and record why in the FEAT findings. Keep existing help tests green.
      Verify: `cargo test -p ff-desktop help *> tools\logs\t7.log` then read the
      log -- new test passes, existing help tests pass.

- [ ] 8. Full behaviour-neutrality + gate verification.
      Confirm no source file exceeds 400 non-test lines
      (`rg --count-matches '' <changed .rs files> | awk -F: '$2>400'`), all docs
      ASCII, and the English baseline is behaviour-neutral. Run the scoped checks
      and read each log: `cargo fmt --check`, `cargo clippy -p ff-i18n -p ff-help
      -p ff-desktop -- -D warnings`, `cargo test -p ff-i18n -p ff-help -p
      ff-desktop`. Hand off for the owner's full `cargo gate --build`.
      Files: none (verification only).
      Verify: each command redirected to `tools\logs\t8-*.log` and read back;
      zero clippy warnings, all scoped tests pass, fmt clean. The egui_kittest
      focus/behaviour tests passing is the Req 10.2 behaviour-neutrality proof.

## Behaviour-neutrality note (Req 10.2, applies to every item)

With `ui.locale = "en"` (the default, no user override): item 3 populates the en
catalogue with the EXACT parsed English prose, so `t_or` returns byte-identical
text at every menu title/description site; the help loader resolves the same
`help/` directory English already used. Therefore the workbench renders and
behaves identically to before, and the existing egui_kittest focus/behaviour
tests (title_line_*, menu_workspace render, help) must stay green unchanged.
