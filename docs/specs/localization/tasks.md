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
    - BLOCKED (needs decision): the in-memory overlay loader (AliasCatalogue::
      load_into / AliasTable::apply_locale_overlay, Task 4) exists and is tested,
      but there is NO on-disk reader that parses `i18n/<locale>/aliases/<env>.toml`
      into an AliasCatalogue and wires it into environment construction. Authoring
      the .toml data now would be dead data until that reader is built. The reader
      is MECHANISM (needs the requirements gate + code + tests), not the light
      DATA work this task scopes. Deferred pending owner direction.
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
