# FFWB Localization (i18n) -- Vision and Forward-Build Directive

STATUS: VISION / FORWARD-BUILD DIRECTIVE. Localization is a GENUINE FUTURE GOAL.
Full implementation is a separate gated effort (CR-NR-103), but this document
captures the intent AND a directive that applies NOW: all FUTURE build work
SHALL be written localization-aware so we stop accumulating hardcoded UI strings
that would have to be extracted later. Owner: "capture localization as a genuine
future goal; future build should be built with it in mind; we should probably
start the localization customization sooner than later."

Plain ASCII only (this doc). (Note: localized CONTENT itself will contain
non-ASCII; the ASCII rule governs project DOCS/source, not the translated
message catalogues that are the i18n deliverable.)

---

## Goal

FFWB SHALL be convertible to other human languages (French, German, Spanish,
...) without rewriting UI code: user-facing text comes from a swappable message
catalogue selected by a locale setting, not from hardcoded string literals.

## Current posture (honest assessment, to be verified)

- SUBSTRATE IS READY: Rust strings are UTF-8 throughout; the app is already
  UTF-8/encoding-clean. Menus, themes, and config are externalised to TOML, so
  menu option text and titles are already DATA (half-way to translatable).
- NOT YET WIRED: user-facing strings in the Rust UI code appear to be inline
  literals (`ui.label("Files")`, `open_error = Some("EDIT requires a file
  path")`, status messages, the `DEFAULT_*_TOML` menu titles). There is (as far
  as reviewed) no message-catalogue / translation-lookup mechanism yet.
- A full read-only i18n-posture audit is still pending (grep for any existing
  localization provision across the ~68 crates; gauge how pervasive hardcoded
  strings are; size the extraction). That audit is the first task of CR-NR-103.

## What localization does and does NOT cover

- LOCALISE: UI chrome, labels, menu option descriptions and titles, status and
  error messages, help content, dialog text, button captions.
- DO NOT LOCALISE (by design, mainframe convention): command VERBS themselves
  (FIND, EXIT, LOCATE, the command-environment vocabularies). Commands stay in
  the source language the way mainframe commands stay English; only their
  DESCRIPTIONS and surrounding chrome are translated. This clean split (verbs =
  stable identifiers, descriptions = translatable) is a natural fit for the
  Command Environment model (CR-CH-053).

## FORWARD-BUILD DIRECTIVE (applies to ALL new work from now)

Until the full i18n framework lands, new code SHALL be written so the eventual
extraction is small, not large:

1. Prefer routing NEW user-facing strings through a single seam rather than
   scattering raw literals, OR at minimum keep them in clearly-marked, easily
   greppable spots (not buried in format!() deep in logic). When the catalogue
   mechanism lands, these become catalogue lookups.
2. Keep user-facing TEXT separate from command VERBS and stable identifiers (do
   not derive displayed text by matching on a translated string).
3. Author help/menu/description content as DATA (TOML/markdown) where practical,
   it is already the externalised, translatable path.
4. Do NOT build the full catalogue system ad hoc per feature; that is CR-NR-103's
   job. The directive here is "write extraction-friendly," not "invent i18n now."

## Likely implementation shape (for CR-NR-103 to decide at its gate)

- A locale config key (e.g. `ui.locale`, default system/English) in the existing
  configuration-system (hot-reloadable like other config).
- A message-catalogue mechanism -- candidates: Fluent (`fluent-rs`, ICU-grade,
  good for plurals/gender) or a gettext-style catalogue. Decision deferred to the
  gate.
- A translation-lookup seam threaded through the render paths (a `t("key")` /
  `tr!` style call) replacing inline literals, extracted incrementally.
- Catalogue files shipped per locale (data, not code), so adding a language is
  adding a catalogue, not editing Rust.
- Command-verb ALIAS sets follow the same per-locale-catalogue model (see
  command-environments Requirement 6a.6-6a.7): selecting a locale loads THAT
  locale's alias catalogue into the per-environment alias tables, not a monolithic
  union of all locales. Only the useful (active-locale) surface forms are resident,
  and there is no per-entry locale tag -- the single `ui.locale` selection picks
  the catalogue. The English canonical verbs stay resolvable in every locale with
  the locale's surface forms layered on top (base English + locale overlay), so
  `FIND` and its localized alias both resolve to canonical `FIND`. A locale's alias
  catalogue is itself organised PER ENVIRONMENT (a set of per-environment tables:
  French FFEDIT verbs, French FFCMD verbs, ...), so the same surface form may map to
  a different canonical verb in a different environment within the same locale (e.g.
  `X` -> EXCLUDE in FFEDIT but EXIT in FFCMD); the dimension is
  per-environment-per-locale, not one flat per-locale list.
- Plugin-authored UI (and the plugin-authored command environments of CR-CH-053)
  localise through the same catalogue seam, so third-party contexts are
  translatable too.

## Sequencing

- NOW: this vision + the forward-build directive (CR-NR-103 logged PENDING GATE).
- SOON (owner: "sooner than later"): run the i18n-posture audit and the gate
  (requirements/design/tasks) so the catalogue mechanism + locale config land
  EARLY, before more hardcoded strings accumulate. Starting the mechanism early
  is cheaper than a big-bang extraction later.
- ONGOING: as the catalogue seam exists, extract strings incrementally (each
  feature that touches UI text converts its literals), rather than one massive
  extraction pass.

## Relationship to other work

- Composes with CR-CH-053 (Command Environments): verbs stay source-language,
  descriptions localise.
- Composes with context-help / markdown-rendering: help content is data and
  localises per-locale.
- Composes with configuration-system: the locale is a config key.
- Composes with plugin-architecture: plugin UI localises through the same seam.
- Does NOT change any framework mechanism; it is an additive presentation layer.
