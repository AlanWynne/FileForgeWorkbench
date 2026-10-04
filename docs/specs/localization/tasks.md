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

- [ ] 2. Add the `ui.locale` config key in `ff-config`
  - [ ] 2.1 Add the `ui` namespace + `LOCALE` constant in `keys.rs` (satisfies
        the dot-path key tests). Satisfies Req 1.1.
  - [ ] 2.2 Register the `ui.locale` `SchemaEntry` in `register_core_schema`
        (String, default `"en"`, `allowed_values` pinned to shipped locales =
        `["en"]`). Satisfies Req 1.1, 1.2, 1.3.
  - [ ] 2.3 Register the `CallbackRegistry::on_reload(&["ui.locale"], ...)`
        handler that reloads BOTH the Message_Catalogue and the Alias_Catalogue
        on a locale change. Satisfies Req 1.4, 1.5.
  - [ ] 2.4 Add the defence-in-depth fallback: an out-of-schema locale value
        falls back to `"en"` and WARNs. Satisfies Req 1.6.

- [x] 3. Ship the English Identity_Base catalogue and prove the seam
  - [x] 3.1 Author the initial `i18n/en/` `.ftl` catalogue (the baseline keys the
        first converted sites need) as DATA. Satisfies Req 2.2, 2.5.
  - [x] 3.2 Add a seam smoke test: `t("key")` resolves the English string;
        behaviour at `ui.locale = "en"` is unchanged. Satisfies Req 3.2, 10.2.

- [ ] 4. Per-environment, per-locale alias-catalogue LOADER (no new mechanism)
  - [ ] 4.1 Implement the loader that overlays a per-environment set of
        surface-form -> canonical-verb rows onto the EXISTING `AliasTable`
        (base English + locale overlay), one catalogue per locale, organised per
        environment. Satisfies Req 5.1, 5.2, 5.3, 5.5.
  - [ ] 4.2 Enforce NO per-row locale tag (the `ui.locale` selection picks the
        catalogue) and reject within-environment collisions with a diagnostic,
        retaining the Identity_Base. Satisfies Req 5.4, 5.6.
  - [ ] 4.3 Assert the canonical English verb remains the recorded/persisted value
        regardless of locale. Satisfies Req 10.3.

## Phase 2 -- Incremental extraction (audit's recommended order)

- [ ] 5. Localise the `DEFAULT_*_TOML` menu titles and descriptions
  - [ ] 5.1 Read menu TITLE / option DESCRIPTIONS to display from the
        Message_Catalogue keyed by menu + option, out of the PARSED menu model
        (no translated TOML on disk; built-in menus stay code-only). Satisfies
        Req 7.1, 7.2.
  - [ ] 5.2 Leave option `command` values unlocalised; fall back to Identity_Base
        English text for a missing key. Satisfies Req 7.3, 7.4, 4.2.

- [ ] 6. Localise help content via per-locale directories
  - [ ] 6.1 Add locale-aware path resolution in `ff-help` `content_loader`
        (resolve `help/<locale>/`, English `help/` fallback) with NO change to
        parsing or the topic model. Satisfies Req 6.1, 6.2.
  - [ ] 6.2 Keep `Topic_Key` / cross-reference keys unlocalised (body/title prose
        only); resolve subsequently displayed topics against the new locale on a
        hot-reload. Satisfies Req 6.3, 6.4.

- [ ] 7. Convert the clean dialog/panel-crate render literals (one crate per step)
  - [ ] 7.1 Convert `ff-catalog-dialog` literals to `t("key")` lookups; keep its
        `egui_kittest` behaviour tests green. Satisfies Req 3.4, 4.1, 10.1.
  - [ ] 7.2 Convert `ff-dataset-alloc-dialog`. Satisfies Req 3.4, 4.1.
  - [ ] 7.3 Convert `ff-files-panel`. Satisfies Req 3.4, 4.1.
  - [ ] 7.4 Convert `ff-explorer-view`. Satisfies Req 3.4, 4.1.
  - [ ] 7.5 Convert `ff-theme-editor`. Satisfies Req 3.4, 4.1.
  - [ ] 7.6 Convert `ff-toolchain-panel`. Satisfies Req 3.4, 4.1.

- [ ] 8. Convert the `ff-desktop` render literals (labels / buttons / hover text)
  - [ ] 8.1 Convert the editor-panel and shell render labels/buttons/hover text to
        `t("key")` lookups, keeping render APIs and behaviour unchanged; keep
        focus/behaviour `egui_kittest` tests green. Satisfies Req 3.4, 4.1, 10.1,
        10.2.

- [ ] 9. Convert the hard status/error messages (argument-bearing, verb-embedding)
  - [ ] 9.1 Triage the `open_error` / ladder `format!` sites crate-by-crate,
        classifying each as user-facing-translatable vs internal-diagnostic.
        Satisfies Req 4.1.
  - [ ] 9.2 Convert user-facing interpolated messages to Fluent entries with named
        placeables resolved via `t_args`. Satisfies Req 9.1.
  - [ ] 9.3 Pass embedded command verbs as NON-translated placeable arguments
        (verb stays English); localise `thiserror`-sourced text at the display
        boundary. Satisfies Req 9.2, 9.3, 4.2.
  - [ ] 9.4 Make the Catalogue_Lookup_Seam available to plugin-authored UI and
        route a plugin-authored command environment's aliases through the same
        per-environment Alias_Catalogue loader. Satisfies Req 8.1, 8.2, 8.3.

## Phase 3 -- Capstone: build out the other-language catalogue DATA (LAST)

- [ ] 10. Build out the other-language catalogue data (DATA authoring, not mechanism)
  - NOTE: This capstone depends on Phase 2 (the incremental extraction) being
    SUBSTANTIALLY complete, so the English `i18n/en/` catalogue is populated
    enough to translate against. It adds DATA only (per-locale `.ftl` message
    catalogues plus per-locale, per-environment alias catalogues); it introduces
    NO new mechanism.
  - [ ] 10.1 Author the French (`fr`) Message_Catalogue (`i18n/fr/` `.ftl` files)
        translating the populated English keys. Satisfies Req 2.2, 2.3.
  - [ ] 10.2 Author the French per-locale, per-environment Alias_Catalogue
        (e.g. `i18n/fr/aliases/<environment>.toml`), loading into the existing
        per-environment `AliasTable` (both `FIND` and `CHERCHER` resolve to
        canonical `FIND`). Satisfies Req 5.1, 5.2, 5.3, 5.5.
  - [ ] 10.3 Author the German (`de`) Message_Catalogue (`i18n/de/`) and its
        per-environment Alias_Catalogue, same pattern as French. Satisfies Req
        2.2, 5.1, 5.3.
  - [ ] 10.4 Add each new language to the `ui.locale` `allowed_values` list in the
        `ff-config` schema so the locale is selectable. Satisfies Req 1.2.
  - [ ] 10.5 Author per-locale help content directories (`help/fr/`, `help/de/`)
        for the translated topics, with English fallback for untranslated topics.
        Satisfies Req 6.1, 6.2.
