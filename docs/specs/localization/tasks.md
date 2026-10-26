# Implementation Tasks -- Localization / i18n (CR-NR-103)

Ordered, independently-completable tasks realising `requirements.md`. The order
follows the audit's 7-step incremental extraction ordering: config + seam first,
then the data-shaped menu titles, then help per-locale directories, then the
clean dialog/panel crates, then `ff-desktop` render literals, then the hard
status/error messages, then verb alias catalogues as data. The FINAL task group
(Task 10) is the capstone: build out the other-language catalogue DATA.

TDD applies: write the failing test first, then the minimum implementation, per
`testing.md`. Each task cross-references the criteria it satisfies.

## Phase 1 -- Mechanism, config key, English base, lookup seam, alias loader

- [x] 1. Create the `ff-i18n` crate (catalogue mechanism + lookup seam)
  - [x] 1.1 Scaffold the GUI-free `ff-i18n` crate with `fluent-bundle` / `fluent`
        / `unic-langid` deps and an `I18nError` thiserror enum. Satisfies Req
        2.1, 3.5.
  - [x] 1.2 Implement `Locale` (parse from the `ui.locale` string; `en` identity
        constant) and `Catalogue` holding per-locale Fluent bundles plus the
        always-present English Identity_Base bundle. Satisfies Req 2.3, 2.5.
  - [x] 1.3 Implement `load(locale)` discovering and parsing per-locale `.ftl`
        files; on parse error retain the prior catalogue / fall back to English
        and WARN. Satisfies Req 2.2, 2.4.
  - [x] 1.4 Implement the Catalogue_Lookup_Seam `t(key)` and the argument-bearing
        `t_args(key, args)`, with active-locale -> Identity_Base -> key-string
        fallback and a WARN on a both-missing key. Satisfies Req 3.1, 3.2, 3.3.
  - [x] 1.5 Hold the active catalogue behind a shared swappable handle so lookup
        is a free call with no per-call catalogue threading. Satisfies Req 3.4.

- [x] 2. Add the `ui.locale` config key in `ff-config`
  - [x] 2.1 Add the `ui` namespace + `LOCALE` constant in `keys.rs` (satisfies
        the dot-path key tests). Satisfies Req 1.1.
  - [x] 2.2 Register the `ui.locale` `SchemaEntry` in `register_core_schema`
        (String, default `"en"`, `allowed_values` pinned to shipped locales =
        `["en"]`). Satisfies Req 1.1, 1.2, 1.3.
  - [x] 2.3 Register the `CallbackRegistry::on_reload(&["ui.locale"], ...)`
        handler that reloads BOTH the Message_Catalogue and the Alias_Catalogue
        on a locale change. Satisfies Req 1.4, 1.5.
    - NOTE: `ff-config::on_locale_reload` registers the callback on the existing
      `CallbackRegistry` keyed on `ui.locale`; the concrete reload target (the
      ff-i18n catalogue swap + the per-environment alias loader) is a
      workbench-supplied closure wired in Task 4 / Task 8. The callback-fires
      behaviour is proven with a spy in `locale_reload.rs` tests.
  - [x] 2.4 Add the defence-in-depth fallback: an out-of-schema locale value
        falls back to `"en"` and WARNs. Satisfies Req 1.6.
    - NOTE: Reuses the existing `validate_table` allowed_values path (applies the
      schema default and logs a WARN naming the key); no parallel path added.

- [x] 3. Ship the English Identity_Base catalogue and prove the seam
  - [x] 3.1 Author the initial `i18n/en/` `.ftl` catalogue (the baseline keys the
        first converted sites need) as DATA. Satisfies Req 2.2, 2.5.
  - [x] 3.2 Add a seam smoke test: `t("key")` resolves the English string;
        behaviour at `ui.locale = "en"` is unchanged. Satisfies Req 3.2, 10.2.

- [x] 4. Per-environment, per-locale alias-catalogue LOADER (no new mechanism)
  - [x] 4.1 Implement the loader that overlays a per-environment set of
        surface-form -> canonical-verb rows onto the EXISTING `AliasTable`
        (base English + locale overlay), one catalogue per locale, organised per
        environment. Satisfies Req 5.1, 5.2, 5.3, 5.5.
  - [x] 4.2 Enforce NO per-row locale tag (the `ui.locale` selection picks the
        catalogue) and reject within-environment collisions with a diagnostic,
        retaining the Identity_Base. Satisfies Req 5.4, 5.6.
  - [x] 4.3 Assert the canonical English verb remains the recorded/persisted value
        regardless of locale. Satisfies Req 10.3.

## Phase 2 -- Incremental extraction (audit's recommended order)

- [x] 5. Localise the `DEFAULT_*_TOML` menu titles and descriptions
  - [x] 5.1 Read menu TITLE / option DESCRIPTIONS to display from the
        Message_Catalogue keyed by menu + option, out of the PARSED menu model
        (no translated TOML on disk; built-in menus stay code-only). Satisfies
        Req 7.1, 7.2.
  - [x] 5.2 Leave option `command` values unlocalised; fall back to Identity_Base
        English text for a missing key. Satisfies Req 7.3, 7.4, 4.2.

- [x] 6. Localise help content via per-locale directories
  - [x] 6.1 Add locale-aware path resolution in `ff-help` `content_loader`
        (resolve `help/<locale>/`, English `help/` fallback) with NO change to
        parsing or the topic model. Satisfies Req 6.1, 6.2.
  - [x] 6.2 Keep `Topic_Key` / cross-reference keys unlocalised (body/title prose
        only); resolve subsequently displayed topics against the new locale on a
        hot-reload. Satisfies Req 6.3, 6.4.

- [x] 7. Convert the clean dialog/panel-crate render literals (one crate per step)
  - [x] 7.1 Convert `ff-catalog-dialog` literals to `t("key")` lookups; keep its
        `egui_kittest` behaviour tests green. Satisfies Req 3.4, 4.1, 10.1.
  - [x] 7.2 Convert `ff-dataset-alloc-dialog`. Satisfies Req 3.4, 4.1.
  - [x] 7.3 Convert `ff-files-panel`. Satisfies Req 3.4, 4.1.
  - [x] 7.4 Convert `ff-explorer-view`. Satisfies Req 3.4, 4.1.
  - [x] 7.5 Convert `ff-theme-editor`. Satisfies Req 3.4, 4.1.
  - [x] 7.6 Convert `ff-toolchain-panel`. Satisfies Req 3.4, 4.1.

- [x] 8. Convert the `ff-desktop` render literals (labels / buttons / hover text)
  - [x] 8.1 Convert the editor-panel and shell render labels/buttons/hover text to
        `t("key")` lookups, keeping render APIs and behaviour unchanged; keep
        focus/behaviour `egui_kittest` tests green. Satisfies Req 3.4, 4.1, 10.1,
        10.2.
    - DONE (20 areas, per-area commits on feature/l10n-phase2-task8). User-facing
      render prose across the ff-desktop panels, shell chrome, dialogs, tab bar,
      POM calendar, and SCRM viewer now resolves through the ff_i18n seam; glyphs,
      enum/mnemonic captions, ids, proper nouns, and menu/POM descriptions
      (Task 5) left as-is. Argument-bearing STATUS/ERROR messages (open_error,
      status-bar Ln/Col/lines/CAPS/phase) are deferred to Task 9 by design.
      Agent-verified compile + ftl-dupe clean per area; OWNER runs the full gate.

- [x] 9. Convert the hard status/error messages (argument-bearing, verb-embedding)
  - [x] 9.1 Triage the `open_error` / ladder `format!` sites crate-by-crate,
        classifying each as user-facing-translatable vs internal-diagnostic.
        Satisfies Req 4.1.
  - [x] 9.2 Convert user-facing interpolated messages to Fluent entries with named
        placeables resolved via `t_args`. Satisfies Req 9.1.
  - [x] 9.3 Pass embedded command verbs as NON-translated placeable arguments
        (verb stays English); localise `thiserror`-sourced text at the display
        boundary. Satisfies Req 9.2, 9.3, 4.2.
  - [x] 9.4 Make the Catalogue_Lookup_Seam available to plugin-authored UI and
        route a plugin-authored command environment's aliases through the same
        per-environment Alias_Catalogue loader. Satisfies Req 8.1, 8.2, 8.3.
    - DONE (merged to main at 210409f). Decisions D1 (verb baked for single-verb-
      prefix messages), D2 (ProfileError/ShellError display-boundary keys, enums
      unchanged), D3 (SCRM engine-report text deferred), D5 all approved. The
      shell STATUS/ERROR messages resolve through t_args; the plugin seam is
      proven by plugin_authored_environment_localises_through_the_same_alias_seam.

## Phase 3 -- Capstone: build out the other-language catalogue DATA (LAST)

- [ ] 10. Build out the other-language catalogue data (DATA authoring, not mechanism)
  - NOTE: This capstone depends on Phase 2 (the incremental extraction) being
    SUBSTANTIALLY complete, so the English `i18n/en/` catalogue is populated
    enough to translate against. It adds DATA only (per-locale `.ftl` message
    catalogues plus per-locale, per-environment alias catalogues); it introduces
    NO new mechanism.
  - POC SCOPE (owner-approved): Afrikaans (`af`) is the hand-reviewed proof-of-
    concept locale (owner reads Afrikaans); French (`fr`) and German (`de`) are
    AI-DRAFTED and flagged "DRAFT -- pending native-speaker review" in every file
    header, to be handed to native speakers later.
  - [x] 10.0 Author the Afrikaans (`af`) Message_Catalogue (`i18n/af/`, 29 files,
        full 464-key parity) as the reviewed POC locale. Dupe-clean, escapes/ASCII
        clean. (Owner-added POC locale; not in the original task list.)
  - [x] 10.1 Author the French (`fr`) Message_Catalogue (`i18n/fr/` `.ftl` files)
        translating the populated English keys. 29 files, full 464-key parity,
        dupe-clean, escapes/ASCII clean (ftl_locale_check + ftl_dupe_check). Satisfies Req 2.2, 2.3.
  - [ ] 10.2 Author the French per-locale, per-environment Alias_Catalogue
        (e.g. `i18n/fr/aliases/<environment>.toml`), loading into the existing
        per-environment `AliasTable` (both `FIND` and `CHERCHER` resolve to
        canonical `FIND`). Satisfies Req 5.1, 5.2, 5.3, 5.5.
    - WAS BLOCKED (on-disk reader missing); the reader is now the gated mechanism
      work of CR-NR-109 Task 11.2 below. Once Task 11.2 lands, this task is the
      pure DATA authoring (the `fr` alias `.toml` files) it was always scoped to.
  - [ ] 10.3 Author the German (`de`) Message_Catalogue (`i18n/de/`) and its
        per-environment Alias_Catalogue, same pattern as French. Satisfies Req
        2.2, 5.1, 5.3.
    - PARTIAL: the `de` Message_Catalogue is DONE (29 files, full 464-key parity,
      dupe/escape/ASCII clean). The `de` alias catalogue is BLOCKED for the same
      on-disk-reader reason as 10.2.
  - [x] 10.4 Add each new language to the `ui.locale` `allowed_values` list in the
        `ff-config` schema so the locale is selectable. `allowed_values` extended
        to `["en","af","fr","de"]`; allowed_values test updated to loop the shipped
        set and reject `"xx"`. Satisfies Req 1.2.
  - [x] 10.5 Author per-locale help content directories (`help/af/`, `help/fr/`,
        `help/de/`) for the translated topics, with English fallback for
        untranslated topics. The ff-help loader resolves `help/<locale>/` then the
        English `help/` (Task 6, wired + tested). Satisfies Req 6.1, 6.2.
    - DONE (POC scope): the getting-started topic (`index.help.md`) is translated
      per locale (af reviewed; fr/de AI-draft, flagged). TOPIC key + topic/feature:/
      line: cross-references stay unlocalised (Req 6.3); every other topic falls
      back to the English `help/` directory (Req 6.2). The remaining ~dozens of
      topics are left to the native-speaker translation pass, which the fallback
      supports first-class.

## Phase 4 -- Locale-aware alias resolution + menu-kind options via localised aliases (CR-NR-109)

Realises Requirement 11 (and the Req 4.2 / 7.3 display amendments). FRAMEWORK
CHANGE (owner-confirmed): reshapes the `AliasTable` entry model and the
menu-option display model; builds ON the single dispatch front door, the existing
`AliasTable` seam, and the `ff_i18n::t()` seam (no new dispatcher / nav stack /
persistence format / focus mechanism). TDD applies; each task writes the failing
test first. Scoped `cargo check -p ff-desktop` / `cargo clippy -p ff-desktop` /
`cargo fmt` for agents; the OWNER runs the full gate.

- [x] 11. Locale-aware alias layer + menu-kind localised option rendering
  - [x] 11.1 Grow the `AliasTable` entry to `(surface, canonical, localisation_key?)`
        in `crates/ff-desktop/src/shell/environment.rs`. Keep `canonical_verb`
        and `apply_locale_overlay`; default `localisation_key` to `None` in the
        seed constructors so the English base is byte-identical. Add a
        display-surface accessor that returns `ff_i18n::t(key)` when a
        localisation key is present (active locale), else the raw surface.
        Satisfies Req 11.1, 11.2; behaviour-neutral at `en` (Req 10.2).
  - [x] 11.2 Build the on-disk alias-catalogue reader: parse
        `i18n/<locale>/aliases/<environment>.toml` (`[[alias]]` rows carrying
        surface / canonical / optional localisation_key) into an in-memory
        `AliasCatalogue`, and load it into the corresponding environment's
        `AliasTable` via the EXISTING `apply_locale_overlay` (its first
        production caller). Wire it into environment construction and the
        `ui.locale` reload callback. A within-environment collision is rejected
        and the Identity_Base retained. Satisfies Req 11.5, 11.10; unblocks
        Task 10.2 / 10.3 alias data.
  - [x] 11.3 Make the synthesised identity rows explicit and editable: seed an
        identity row (`surface == canonical`, `localisation_key = None`) for every
        root command; an authored/overlaid row for the same canonical REPLACES
        the synthesised default (persisted via the on-disk alias data). Prove the
        Russian-root / English-surface override case. Satisfies Req 11.3, 11.4.
  - [x] 11.4 Implement the unified locale-aware resolver `resolve_locale_verb(V, L)`
        applying the Req 11.6 order (alias-table-for-L -> root -> root-table ->
        unknown-command message), returning the ROOT command. Call it from BOTH
        the command-line entry (`shell/dispatch.rs`, before `resolve_target`) and
        the menu-selection entry. Dispatch the returned root through the UNCHANGED
        front door; the root is what persists. Satisfies Req 11.6, 11.9, 10.3.
  - [x] 11.5 Refactor the menu-kind render path: add
        `i18n_keys::localised_option_command(environment, option, L)` returning
        the active-locale alias surface for the option's root command (root/English
        fallback), and call it in `menu_workspace/render.rs` where the command
        column currently formats `option.command`. Selecting an option still
        yields the ROOT `command`, fed through the unified resolver (11.4).
        Applies to ALL menu kinds via the shared `render_menu_workspace`.
        Satisfies Req 11.7, 11.8, 4.2-amended, 7.3-amended.
  - [x] 11.6 Full-shell `egui_kittest` test: with a non-`en` locale + a loaded
        alias catalogue, a menu option's command column shows the localised
        surface and selecting it dispatches + records the ROOT command; with `en`
        / no overlay the column shows the root verb unchanged (Req 10.2). Keep the
        menu-kind first-Tab focus test green (workspace-conformance.md).
        Satisfies Req 11.7, 11.9, 10.2.
  - [x] 11.7 Author the Afrikaans (`af`) per-environment alias catalogue
        (`i18n/af/aliases/<environment>.toml`) as the reviewed POC data, now that
        the reader (11.2) exists. (French/German alias data follow as Task 10.2 /
        10.3 DATA work.) Satisfies Req 5.1, 5.5, 11.1.

## Phase 5 -- Diacritic-insensitive command input (CR-NR-109 input-accessibility amendment)

Realises the amended Requirement 11.2 and 11.6 plus the new Requirements
11.11-11.13 (diacritic-insensitive input matching). BEHAVIOUR CHANGE to the
resolver (owner-confirmed): the DISPLAY surface keeps correct diacritics; the
RESOLVER folds typed input (Unicode NFD -> strip combining marks -> ASCII case
fold) so a plain-keyboard user may type the ASCII form of a diacritic-bearing
verb and still reach the same root command. Builds ON the existing unified
resolver (Task 11.4, `shell/locale_resolver.rs`) and the existing `AliasTable`
lookup; adds NO new dispatcher / nav stack / persistence format. Exact matching
stays a subset of folded matching, so all existing ASCII/exact input is
behaviour-preserving. TDD applies; each task writes the failing test first.
Scoped `cargo check -p ff-desktop` / `cargo clippy -p ff-desktop` / `cargo fmt`
for agents; the OWNER runs the full gate.

- [ ] 12. Fold typed command input diacritic-insensitively in the resolver
  - [ ] 12.1 Add a `fold_for_match(s)` helper (Unicode NFD normalisation, strip
        combining marks (category Mn), ASCII case fold) in the alias/resolver
        layer and unit-test the fold (accented surface and plain-ASCII input fold
        to the same value; ASCII-only input is unchanged). Satisfies Req 11.11.
  - [ ] 12.2 Apply the fold to BOTH sides of each lookup step in
        `resolve_locale_verb(V, L)` (the locale-`L` alias-table lookup and the
        root-command-table lookup), preserving the Req 11.6 order and the
        unknown-command fall-through; prove an exact match still resolves (fold is
        a superset) so `en` / ASCII behaviour is unchanged (Req 10.2). Satisfies
        Req 11.2-amended, 11.6-amended, 11.11.
  - [ ] 12.3 Handle fold collisions: when two distinct surface forms in the same
        environment+locale scope fold to the same value, prefer an exact match;
        when an entered verb folds ambiguously with no exact disambiguation, issue
        a diagnostic naming the colliding candidates rather than silently
        resolving. Satisfies Req 11.12.
  - [ ] 12.4 Confirm the DISPLAY surface is untouched by the fold (the fold
        affects input matching only); keep the menu-kind render showing full
        diacritics. Add the optional explicit-ASCII-alias path only where a word
        needs a genuinely different ASCII spelling (data-only, no new mechanism).
        Satisfies Req 11.13.
  - [ ] 12.5 Full-shell `egui_kittest` test: with the `af` locale + alias
        catalogue, typing the plain-ASCII form of a diacritic-bearing Afrikaans
        verb resolves to and records the ROOT command, and the displayed surface
        still shows the accented form; keep the menu-kind first-Tab focus test
        green (workspace-conformance.md). Satisfies Req 11.2-amended, 11.13.

## Phase 6 -- ASCII policy scope follow-on (CR-NR-103 / CR-NR-109 ASCII amendment)

Realises the amended context-help Requirement 17.9 / 17.9c (ASCII policy exempts
the WHOLE `help/` tree -- English base INCLUDED -- as rendered product content;
per-locale and English help content both allowed non-ASCII UTF-8). The amendment
itself is a docs change landed by this gate; the single ff-help TEST relaxation
below has ALSO landed (owner-requested fix), and Task 14 is the
data-already-satisfies note.

- [x] 13. Remove/relax the ff-help ASCII test for the WHOLE help/ tree (context-help Req 17.9c)
  - [x] 13.1 Changed `shipped_content_files_are_valid_utf8` (formerly
        `shipped_content_files_are_ascii`) in
        `crates/ff-help/tests/shipped_content_tests.rs` to REMOVE the ASCII
        enforcement over the ENTIRE `help/` tree (the English base in the `help/`
        ROOT directory AND every per-locale `help/<locale>/` subdirectory), not
        merely narrow it to the English base. The test now walks every `.help.md`
        file and asserts only valid UTF-8; the `// Validates:` annotation keys to
        the amended whole-tree Requirement 17.9. Satisfies context-help Req 17.9
        (amended whole-tree), 17.9c. (Unblocks the red cargo gate caused by B085.)
    - NOTE: this is the SCOPED code/test change the amendment described; it is the
      ONLY source/test change implied by Amendment 1 and it has LANDED (the
      owner-requested ASCII-test fix: renamed test present, ASCII assertion
      removed, fmt/clippy/build clean). The symbol `shipped_content_files_are_ascii`
      no longer exists.
- [x] 14. Confirm the existing per-locale help files satisfy the amended rule (context-help Req 17.9)
  - NOTE (no code): `help/af/index.help.md`, `help/fr/index.help.md`, and
    `help/de/index.help.md` already carry accented translated prose (localization
    Task 10.5). Under the amended whole-tree Req 17.9 the entire `help/` tree
    (English base included) is exempt from the ASCII rule, so these files are
    CONFORMING as-is -- now literally proven by the passing
    `shipped_content_files_are_valid_utf8` test; no edit to the translated files
    is required. B085 is resolved by Task 13 (the test relaxation that has
    landed), not by changing these files.

## Phase 7 -- Author the Afrikaans menu-option alias DATA + catalogue keys (downstream DATA)

Downstream DATA authoring that the Afrikaans menu surface needs end-to-end. The
mechanism (Phase 4) exists; this is DATA only (no new mechanism).

- [ ] 15. Author the `af` menu-option alias DATA + the verb-* catalogue keys (Req 11.5, 11.7, 11.8)
  - [ ] 15.1 Author/extend the `af` per-environment alias DATA
        (`i18n/af/aliases/<environment>.toml`) so every menu-kind option a POM /
        Settings / user menu presents has an Afrikaans surface row mapping to its
        root command (surface / canonical / optional localisation_key). Satisfies
        Req 11.5, 11.7.
  - [ ] 15.2 Author the matching `verb-*` message-catalogue keys in `i18n/af/`
        (and the `i18n/en/` identity values, e.g. `verb-find = FIND` in en,
        `verb-find = SOEK` in af) so each alias row's localisation key resolves
        end-to-end; without the keys `t(key)` returns the key and the surface
        correctly falls back to the root (Req 11.8). Satisfies Req 11.7, 11.8.
  - [ ] 15.3 Author the diacritic-bearing Afrikaans surfaces with their full
        correct accents (display correctness, Req 11.13); input tolerance for the
        plain-ASCII forms is provided by the Phase 5 resolver fold (Task 12), so
        no duplicate ASCII alias rows are needed except where a word needs a
        genuinely different ASCII spelling. Satisfies Req 11.13.

## Phase 8 -- Full menu-Kind localisation: mnemonic + command alias + description (CR-NR-111)

Realises Requirement 12 (full menu-option localisation) -- the DELTA on top of
CR-NR-109 (Requirement 11). CR-NR-109 already made the alias layer locale-aware,
rendered the option COMMAND surface via the active-locale alias, and wired the
unified localised-alias -> root resolver for both the command line and menu
selection. Requirement 12 extends FULL localisation to the MNEMONIC (the one
genuinely new seam), formalises the displayed-COMMAND derivation, settles the
Locale_Independent_Stored_Value as the ROOT command (store Option A -- NO
migration, NO new `MenuOption` field), and makes per-part fallback plus the
optional second-prize per-language saved menu explicit.

NOT a framework change (the `rewrite_verb_to_root` -> `resolve_target` pre-pass
already delivers routing invariance on both seams); NO new dispatcher, NO parallel
resolver, NO new persistence format, NO Workspace-Kinds-registry change. Built-in
menus stay code-only (`DEFAULT_*_TOML`, never written to disk, CR-CH-021). The
ROOT command remains the stored/dispatched/persisted identity.

TDD applies: write the failing test first, then the minimum implementation
(`testing.md`). Agents compile-verify SCOPED (`cargo check -p ff-desktop`,
`cargo clippy -p ff-desktop -- -D warnings`, `cargo fmt -- --check`) and run the
single new scoped test where quick; the OWNER runs the full gate and suite.

- [ ] 16. Add the per-locale mnemonic catalogue seam (mirror the description seam)
  - [x] 16.1 Add `menu_option_key_key(menu_name, command)` in
        `crates/ff-desktop/src/menu_workspace/i18n_keys.rs` building the key
        `menu-<menu>-opt-<command>-key` using the SAME `slug(command)` helper the
        existing `menu_option_desc_key` uses (so `Reset bare` -> `reset-bare`),
        and MUST NOT build the key from the raw command string. (Req 12.3)
  - [x] 16.2 Add `localised_option_mnemonic(menu_name, option)` =
        `t_or(menu_option_key_key(menu_name, option.command), &option.key)` in the
        same file, returning the catalogue value when present and the option's own
        `key` (Identity_Base English mnemonic) when absent. This is the authored
        REALISED CONTRACT -- Option (ii); deriving the mnemonic from the command
        surface (Option i) MUST NOT be implemented. (Req 12.3, 12.8)
  - [x] 16.3 Unit tests: `localised_option_mnemonic` returns the catalogue value
        when the `menu-<menu>-opt-<command>-key` key is present and the option's
        `key` fallback when absent (mirror the existing
        `localised_option_description` test); assert `menu_option_key_key` slugs
        the command identically to `menu_option_desc_key`
        (`menu-settings-opt-reset-bare-key`). (Validates Req 12.3, 12.8)
  - [ ] 16.4 Author the `i18n/en/` identity mnemonic keys for the built-in
        menu-option roots so the English base resolves end-to-end (a missing key
        correctly falls back to `option.key`). English DATA only; other-locale
        mnemonic DATA is the out-of-scope downstream deliverable (Phase 7-style).
        (Req 12.3, 12.8)
    - NOT DONE BY DESIGN: the English catalogue ships NO `menu-*-opt-*-key`
      entries today, and the CODE contract (step 16.1-16.3) proves the
      `option.key` fallback is the CORRECT English-base behaviour (a missing key
      yields the stored root mnemonic). Authoring English `-key` DATA is the
      separate out-of-scope DATA deliverable; the in-test seeding in
      `tests_locale_menu_full.rs` supplies the keys needed to prove localisation.

- [x] 17. Render all three option parts localised through the shared renderer
  - [x] 17.1 Change `menu_workspace/render_layout.rs::option_prefix_job` to accept
        the resolved MNEMONIC surface positionally (exactly as it already accepts
        the resolved command surface), replacing the raw `{:<4}` of `option.key`.
        (Req 12.1, 12.3)
  - [x] 17.2 Compute the mnemonic-surface vector shell-side beside
        `menu_option_command_surfaces` (`shell/locale_resolver.rs`), positionally
        aligned with `menu.options`, each entry falling back per option; pass both
        the mnemonic and command surfaces into the shared `render_menu_workspace`.
        (Req 12.1, 12.2, 12.3, 12.8)
  - [x] 17.3 Confirm the displayed COMMAND surface derivation is unchanged from
        CR-NR-109 (`display_surface_for_canonical(option.command)` with root/
        English fallback) and the DESCRIPTION derivation
        (`localised_option_description`) is unchanged; all three parts now flow
        through the one shared renderer for ALL menu kinds (POM/Settings/user),
        no per-kind branch. (Req 12.1, 12.2, 12.9)
  - [x] 17.4 Keep the displayed MNEMONIC DISPLAY-ONLY: the keystroke that selects
        an option (typed `MENU <name> <key>` or in-menu key press) continues to
        match the stored `option.key`, NOT the displayed localised mnemonic.
        (Req 12.5; menu-workspace Req 21.8)
  - [x] 17.5 Unit test: the shell-computed mnemonic-surface vector is positionally
        aligned with `menu.options` and falls back per option. (Validates Req
        12.1, 12.3, 12.8)

- [x] 18. Record and prove the Locale_Independent_Stored_Value = ROOT (Option A)
  - [x] 18.1 Assert (test, no model change) that `MenuOption.command` stores the
        locale-independent ROOT and that the displayed command/mnemonic/description
        are DERIVED at render time, so a SINGLE saved English menu renders in the
        active locale WITHOUT being re-saved and WITHOUT a new `MenuOption` field.
        (Req 12.4, 12.6)
  - [x] 18.2 Assert selection feeds the option's ROOT through the existing unified
        resolver (`rewrite_verb_to_root` -> `resolve_target`) so dispatch, history,
        and persistence record the ROOT regardless of the active locale or the
        displayed surface -- NO new or parallel dispatch path. (Req 12.5;
        command-framework Req 18.1-18.3; command-semantics Req 13.1-13.4)
  - [x] 18.3 Confirm NO migration of existing saved menu files is required (Option
        A is the existing on-disk shape) and that files load unchanged and render
        identically at `ui.locale = "en"`. (Req 12.4; Req 10.2 behaviour-neutral)

- [x] 19. First-prize auto-conversion, end to end
  - [x] 19.1 On a `ui.locale` change, confirm `on_locale_reload` (`main.rs`)
        re-seeds the alias store (CR-NR-109) and sets the new i18n locale so the
        next frame recomputes all three derived surfaces from the SAME stored
        definition, with NO separate per-language menu file required. (Req 12.6)
  - [x] 19.2 Full-shell `egui_kittest` test: with a non-`en` locale plus
        mnemonic/alias/description DATA loaded, a menu option row shows all THREE
        parts localised and selecting it dispatches + records the ROOT command;
        with `en` / no overlay the row shows the raw root mnemonic/command and the
        English description unchanged. (Validates Req 12.1, 12.5, 12.6, 10.2)
  - [x] 19.3 Per-part fallback test: an option with an alias row but no mnemonic
        key shows the localised command and the Identity_Base mnemonic, proving
        fallback is per-part, not whole-row. (Validates Req 12.8)
  - [x] 19.4 Keep the menu-kind first-Tab focus `egui_kittest` test green
        (workspace-conformance.md); confirm full-row localisation adds no phantom
        focus stop. (Validates workspace-conformance; menu-workspace Req 21.7)

- [x] 20. Second-prize per-language saved menus (FALLBACK, load/reconstruct time)
  - [x] 20.1 Add the per-locale directory segment to the TWO user-menu reconstruct
        seams in `shell/nav_reconstruct.rs`: `reconstruct_settings_menu` and
        `reconstruct_named_menu` resolve `menus_dir().join(L).join("<name>.toml")`
        when present, else `menus_dir().join("<name>.toml")`, else the code-only
        `DEFAULT_*_TOML` baseline -- reading the THEN-active locale `L` from the
        same active-locale source `on_locale_reload` / `menu_option_command_surfaces`
        use. (Req 12.7)
  - [x] 20.2 Leave the POM load path (`reconstruct_pom` -> `ensure_pom_menu_loaded`
        in `shell/commands_fastpath.rs`, which builds `menus_dir().join("pom.toml")`
        directly) UNCHANGED: the POM is OUT of scope for the second-prize FILE
        override and is localised by first-prize auto-conversion only; no
        `menus/<L>/pom.toml` is wired and no base `menus/pom.toml` is written
        (CR-CH-021). (Req 12.7b, 12.9)
  - [x] 20.3 Keep the session descriptor locale-NEUTRAL: a Menu_Workspace persists
        as `WorkspaceDescriptor::Menu { name }` (no locale-qualified or absolute
        `file_path`); the locale segment is resolved at LOAD/RECONSTRUCT time by
        name + active locale, never baked into a persisted path. (Req 12.7a)
  - [x] 20.4 Menu-loader unit test: `menus/<L>/<name>.toml` is loaded when present
        and the base `menus/<name>.toml` otherwise; a built-in with no file falls
        STRAIGHT THROUGH to the code-only `DEFAULT_*_TOML` (no base file written --
        CR-CH-021). (Validates Req 12.7, 12.7b)
  - [ ] 20.5 Second-prize persistence test: a session persisted under locale `af`
        carries the locale-NEUTRAL `WorkspaceDescriptor::Menu { name }`;
        reconstructing it under `en` re-resolves by name + active locale and opens
        the English/base menu (proves the restored menu follows the then-active
        locale, not the save-time locale). (Validates Req 12.7a)
    - PARTIAL: the locale-NEUTRAL descriptor is preserved by CONSTRUCTION (this CR
      did NOT change `WorkspaceDescriptor::Menu { name }` -- no locale baked into a
      persisted path), and `pom_load_path_ignores_per_locale_file` +
      `locale_qualified_menu_path_prefers_per_locale_file_else_base` prove the
      locale segment is resolved at LOAD/RECONSTRUCT time by name + active locale.
      A dedicated full session af-save/en-restore roundtrip test is a FOLLOW-UP
      (the existing session persistence tests already pin the descriptor shape).

- [x] 21. Confirm chrome / Kinds-registry invariance (no new field)
  - [x] 21.1 Confirm (test or assertion) that full menu-option-content localisation
        requires NO new field on the Kind configuration data model and NO change to
        the Kind registry mechanism; per-Kind chrome keeps resolving via its
        existing catalogue-keyed seams. (Validates workspace-kinds Req 9.1, 9.2,
        9.3)

- [x] 22. Coverage and documentation update (CR-NR-111)
  - [x] 22.1 Update the coverage record (the CR-NR-111 NOT-COVERED rows in
        `docs/quality/TCR.md`) to the correct status as each criterion's test
        lands. (Project standards)
  - [x] 22.2 Update `docs/status/change-log.md` CR-NR-111 Status from GATE
        AUTHORING to IN PROGRESS / DONE as implementation proceeds and mark the
        completed tasks here. (Project standards)

---

## CR-CH-072 -- Single-layer surface-in-row alias + ONE locale-aware store

TDD-first: write the failing test before the implementation for each task. Agent
verification is compile/clippy/fmt clean + scoped tests; the authoritative full
gate is the owner's manual run (gate-policy.md). Implementation follows owner
approval of this gate (and the four OPEN DECISIONS) as a SEPARATE run.

- [ ] 23. Unify the alias model and drop the command-surface localisation layer
  - [ ] 23.1 Write failing unit tests that an `AliasEntry` / `AliasRowToml` with
        `surface` + `canonical` (+ optional `command_template`) resolves surface
        -> root by ONE lookup with NO `localisation_key` / `t(key)` consulted.
        (Validates loc Req 13.1, semantics Req 14.1 amended)
  - [ ] 23.2 Remove the `localisation_key` field from `AliasEntry` (environment.rs)
        and `AliasRowToml` (alias_reader.rs); make the in-row `surface` the
        authoritative display/typeable text. Keep `command_template`. (Validates
        loc Req 13.1, 13.8)
  - [ ] 23.3 Confirm the diacritic-insensitive fold (CR-CH-065) still applies to
        the in-row surface (test a folded-input match). (Validates loc Req 11.11
        retained)
  - [ ] 23.4 Confirm auto-synthesised identity rows still resolve the root and
        remain overridable by an explicit row. (Validates loc Req 11.3, 11.4)

- [ ] 24. The single two-level resolution path (active env -> FFCMD)
  - [ ] 24.1 Write failing tests for the two-level order: ACTIVE alias -> ACTIVE
        root -> reject -> FFCMD alias -> FFCMD root -> unresolved, over the in-row
        surface, identical for typed and menu-selected input. (Validates
        command-environments Req 2b.1-2b.3, loc Req 13.2)
  - [ ] 24.2 Re-point the existing `rewrite_verb_to_root` pre-pass
        (`shell/locale_resolver.rs`) at the in-row surface; add NO new dispatcher;
        keep `resolve_target` / `CommandTarget` unchanged. (Validates
        command-environments Req 2b.5)
  - [ ] 24.3 Confirm a matched alias carrying a `command_template` still expands
        (&1..&N, &*) before `resolve_target`. (Validates loc Req 13.8,
        command-semantics Req 14.2-14.4 retained)

- [ ] 25. Menu option references an alias (retire render-time reverse-derivation)
  - [ ] 25.1 Write a failing full-shell egui_kittest that a menu option renders
        the active-locale in-row surface AND, on selection, dispatches the root via
        the one path (no reverse-derivation). (Validates loc Req 13.4,
        command-framework Req 18.1 amended)
  - [ ] 25.2 Change the menu render path (`menu_workspace/`) to look up the
        active-locale in-row surface by the option's root (per-part fallback to
        root/English, loc Req 12.8) instead of reverse-deriving it from a
        translated string. Keep the persisted value = root (recommended OPEN
        DECISION 2). (Validates loc Req 13.4, 10.3)

- [ ] 26. One locale-aware store: shipped seed + user per-locale override
  - [ ] 26.1 Write failing tests for the seed-overlaid-by-override merge (user
        override wins on a shared surface) and the collapse of the 3-overlay stack
        to one chain per environment-per-locale. (Validates loc Req 13.5)
  - [ ] 26.2 Add the user per-locale override reader
        (`<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`,
        recommended OPEN DECISION 1) layered on the shipped
        `i18n/<locale>/aliases/<ENV>.toml` seed via the EXISTING
        `apply_locale_overlay` seam; extend the reload seam to re-apply seed +
        override in one pass. (Validates loc Req 13.5, 13.6)
  - [ ] 26.3 Confirm the shipped seed is never written by a save (save targets the
        user override only). (Validates loc Req 13.6, configurator Req 6.3)

- [ ] 27. Alias Editor edits the active-locale effective table
  - [ ] 27.1 Write a failing full-shell test that opening the Alias Editor under
        `af` SHOWS the shipped `af` surfaces (the "I don't see Afrikaans aliases"
        fix). (Validates configurator Req 6.1)
  - [ ] 27.2 Make the editor display the active-locale effective table (seed +
        override); drop the L18N (localisation-key) column from the CR-CH-068
        layout; keep Act / CE / Root / ALIAS(surface) / Parameters. (Validates
        configurator Req 6.1, 6.2)
  - [ ] 27.3 Write a failing end-to-end test: edit an `af` alias surface, SAVE, and
        assert the user override carries it AND the menu renders the edited surface.
        (Validates configurator Req 6.3, 6.4, loc Req 13.7)
  - [ ] 27.4 Wire the editor save/discard/reload to the user override store; confirm
        CR-CH-071 Save/End/Cancel/Return/Refresh semantics operate over it.
        (Validates configurator Req 6.5)
  - [ ] 27.5 Confirm the validation discipline (empty surface/root, within-env
        surface->two-roots collision, no-shadow built-in root, valid (env,root)) on
        override rows. (Validates configurator Req 6.6)

- [ ] 28. Migrate the current af DATA to the one-layer shape
  - [ ] 28.1 Remove the `localisation_key` line from each `[[alias]]` row in
        `i18n/af/aliases/FFCMD.toml` (the `surface` already holds the af text);
        retire the command-surface `verb-*` keys in `af`/`en`
        `ff-desktop-verbs.ftl` (DESCRIPTION/TITLE keys in `menus.ftl` UNCHANGED).
        (Validates loc Req 13.9)
  - [ ] 28.2 Prove POM + Settings still render fully in `af` with ONE lookup
        (INSTELLINGS/KATALOGUSSE/LEERS/HULP/KONFIG/ALIASSE/TEMA/KIESLYSTE/SLEUTELS/
        SOORTE/HERSTEL KAAL; descriptions af; mnemonics English). (Validates loc
        Req 13.9)

- [ ] 29. Test matrix and coverage (the no-regression proof)
  - [ ] 29.1 af menus still render fully; typed `af` alias resolves to root;
        template expands; editor edits an `af` alias and the menu reflects it after
        save; resolution falls back active-env -> FFCMD; mnemonics stay English.
        (Validates loc Req 13.9, command-environments Req 2b)
  - [ ] 29.2 Update the CR-CH-072 NOT-COVERED rows in `docs/quality/TCR.md` to PASS
        as each test lands; flip the change-log Status. (Project standards)

- [ ] 30. Deferrable / separable items
  - [ ] 30.1 DEFERRABLE: authoring additional non-`af` locale alias seeds (fr/de)
        under the one-layer shape -- separate DATA task, not required for this
        rework. (Validates loc Req 13.6 for future locales)
