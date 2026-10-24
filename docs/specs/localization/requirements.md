# Requirements Document -- Localization / i18n (CR-NR-103)

## Introduction

This sub-project defines the localization (i18n) MECHANISM for
FileForgeWorkbench: the ability to convert FFWB user-facing text to other human
languages (French, German, Spanish, ...) by selecting a locale, with text drawn
from a swappable message catalogue rather than from hardcoded Rust string
literals. It realises the goals captured in `localization-vision.md` and is
consistent with the Command Environment verb/text split defined in
`command-environments` Requirement 6a.

Localization is an ADDITIVE PRESENTATION LAYER. It changes NO core framework
mechanism (single command dispatch, per-tab Navigation_Stack, WorkspaceContext
focus latch, descriptor-based session persistence, code-only built-in menus). It
builds ON existing seams: the `ff-config` schema registry + hot-reload callbacks,
the per-environment `AliasTable` in `ff-desktop/src/shell/environment.rs`, the
`ff-help` content loader, the parsed menu model, and the egui render paths.

### First-gate scope (deliberately NARROW)

This gate specifies the MECHANISM and proves it with ENGLISH ONLY. Concretely it
covers:

- the `ui.locale` configuration key (String, default `"en"`, hot-reloadable);
- the message-catalogue mechanism (Fluent) and per-locale `.ftl` catalogue files
  as DATA;
- the Catalogue_Lookup_Seam (a `t("key")`-style call) threaded incrementally
  through render paths;
- the always-present English Identity_Base catalogue;
- the verb/text split (verbs stay English; descriptions, chrome, labels,
  messages, help translate) consistent with `command-environments` Req 6a;
- the per-locale, per-environment Alias_Catalogue LOADER into the EXISTING
  per-environment `AliasTable` (no new mechanism);
- help content localisation via per-locale directories in the `ff-help` loader;
- menu `DEFAULT_*_TOML` title/description localisation via the parsed menu model;
- plugin-authored UI and CR-CH-053 plugin-authored command environments
  localising through the SAME seam;
- argument-bearing / verb-embedding messages handled via Fluent placeables with
  any embedded verb kept as a NON-translated argument.

This gate does NOT attempt the full string extraction across all crates in one
shot. Per-crate string extraction is INCREMENTAL follow-on work (see tasks.md,
following the audit's 7-step ordering). It ships ONLY the `"en"` catalogue (the
baseline that proves the seam). It does NOT commit to French or German CONTENT
in this gate; building out other-language catalogue data is the final capstone
task group, dependent on the extraction being substantially complete.

### Source references

- **[CR-NR-103]** = this change request (localization / i18n).
- **[vision]** = `docs/specs/localization/localization-vision.md`.
- **[audit]** = `.agents/tasks/localization-i18n-audit/i18n-posture-audit.md`.
- **[CR-CH-053 Req 6a]** = `docs/specs/command-environments/requirements.md`
  Requirement 6a (verb aliases resolve to a canonical English verb per
  environment; per-locale alias catalogues load into those SAME tables).

### Cross-references

- `configuration-system` -- the `ff-config` schema registry, six-layer merge,
  hot-reload, and `CallbackRegistry::on_reload` the `ui.locale` key reuses.
- `command-environments` Req 6a -- the per-environment `AliasTable` and the
  per-environment-per-locale alias-catalogue model.
- `context-help` -- help content is data on disk (`.help.md`), localised by a
  per-locale directory.
- `menu-and-statusbar` / `menu-workspace` -- the `DEFAULT_*_TOML` menu titles and
  option descriptions localised via the parsed menu model.
- `plugin-architecture` -- plugin UI and plugin-authored command environments
  localise through the same seam.
- `theme-and-appearance` -- the egui render paths the seam threads into.

## Glossary

| Term | Definition |
|------|-----------|
| **Locale** | A BCP-47 language identifier (e.g. `en`, `fr`, `de`) selecting which catalogues are active. Chosen once via the `ui.locale` config key; there is no per-string or per-alias-row locale tag. |
| **Message_Catalogue** | The set of translatable user-facing strings for ONE locale, authored as Fluent `.ftl` DATA files (keyed messages, some with placeable arguments), loaded into a Fluent bundle at runtime. |
| **Catalogue_Lookup_Seam** | The single lookup API (`t("key")` and an argument-bearing variant) that resolves a message key against the active locale's Message_Catalogue, falling back to the Identity_Base when a key is absent. UI render code calls this seam instead of embedding literals. |
| **Locale_Key** | The `ui.locale` configuration key (String, default `"en"`), registered in the `ff-config` schema with `allowed_values` constrained to the shipped locales, hot-reloadable via the existing `CallbackRegistry`. |
| **Alias_Catalogue** | The per-locale, per-environment set of command-verb surface forms (aliases) that load into the EXISTING per-environment `AliasTable` (surface form -> canonical English verb). One catalogue per locale, organised per environment; no per-row locale indicator. |
| **Identity_Base** | The always-present English (`"en"`) layer: the base Message_Catalogue and the base alias identity entries. Every locale resolves against English as the identity base, with the locale's surface/message forms layered on top; English is never removed. |

---

## Requirements

### Requirement 1: The ui.locale configuration key

**User Story:** As a user, I want to choose my UI language with a single config
setting, and have the change take effect without restarting.

#### Acceptance Criteria

1. THE configuration system SHALL define a `ui.locale` key as a String with a
   built-in default of `"en"`, registered in the `ff-config` core schema the same
   way other core keys (e.g. `theme.active`, `logging.level`) are registered.
2. THE `ui.locale` key SHALL constrain its accepted values via an `allowed_values`
   constraint limited to the SHIPPED locales (initially `"en"` only), so an
   unknown locale is rejected by schema validation like any other constrained key.
3. WHEN no user layer sets `ui.locale`, THE effective locale SHALL be the default
   `"en"` (the Identity_Base), so a fresh install runs in English.
4. WHEN the effective value of `ui.locale` changes at runtime, THE workbench SHALL
   be notified via the EXISTING `ff-config` reload-callback mechanism
   (`CallbackRegistry::on_reload(&["ui.locale"], ...)`), with NO new config
   mechanism introduced.
5. WHEN the `ui.locale` reload callback fires, THE workbench SHALL reload BOTH the
   active Message_Catalogue AND the per-locale, per-environment Alias_Catalogue,
   so a locale change is hot (consistent with how `theme.active` hot-reload
   propagates today).
6. WHERE a configured `ui.locale` value is not a shipped locale (bypassing schema
   validation is not possible, so this is a defence-in-depth case), THE workbench
   SHALL fall back to the Identity_Base `"en"` and SHALL log a WARN naming the
   rejected locale, rather than failing to render.

### Requirement 2: Message catalogue mechanism (Fluent)

**User Story:** As a translator, I want to add a language by authoring a
catalogue file, not by editing Rust code.

#### Acceptance Criteria

1. THE localization layer SHALL use Fluent (`fluent-bundle` / `fluent` with
   `unic-langid`) as the Message_Catalogue mechanism, so messages are keyed
   entries that MAY carry named placeable arguments (`{ $arg }`) for
   interpolation, plural, and gender.
2. A Message_Catalogue SHALL be authored as per-locale `.ftl` DATA files shipped
   under a per-locale directory (e.g. `i18n/<locale>/`), NOT compiled into Rust,
   so adding a language is adding catalogue files.
3. THE localization layer SHALL load the `.ftl` files for the active locale into a
   Fluent bundle keyed by that locale's `unic-langid` identifier at startup and on
   a `ui.locale` reload.
4. WHEN a `.ftl` file for a locale fails to parse, THE localization layer SHALL
   retain the previously loaded catalogue (or the Identity_Base on first load),
   SHALL log a WARN naming the file and parse error, and SHALL NOT crash or render
   blank text.
5. THE Identity_Base English catalogue (`en`) SHALL always be present and SHALL be
   loadable even when no other locale is installed.

### Requirement 3: The catalogue-lookup seam

**User Story:** As a developer, I want one lookup call to turn a key into the
active-locale string, so render code never embeds a raw literal.

#### Acceptance Criteria

1. THE localization layer SHALL expose a Catalogue_Lookup_Seam: a function
   `t(key)` returning the active locale's string for a message key, and an
   argument-bearing variant (e.g. `t_args(key, args)`) that supplies named Fluent
   placeable arguments.
2. WHEN a key is present in the active locale's Message_Catalogue, THE seam SHALL
   return that locale's string; WHEN the key is absent in the active locale, THE
   seam SHALL fall back to the Identity_Base English string for that key.
3. WHEN a key is absent in BOTH the active locale and the Identity_Base, THE seam
   SHALL return a stable, visible fallback (the key itself) and SHALL log a WARN
   naming the missing key, so a missing translation is diagnosable and never
   produces blank UI.
4. THE Catalogue_Lookup_Seam SHALL be callable from egui render code WITHOUT
   changing the egui render APIs: a render site replaces a string literal with a
   `t("key")` call that yields the same `String`/`&str` the widget already
   expects.
5. THE Catalogue_Lookup_Seam SHALL be GUI-free in its own crate (usable without
   egui), so non-GUI crates and tests can resolve messages the same way.

### Requirement 4: The verb/text split (verbs stay English)

**User Story:** As a mainframe-minded user, I want command VERBS to stay English
(stable identifiers) while only their DESCRIPTIONS and surrounding chrome
translate.

#### Acceptance Criteria

1. THE localization layer SHALL localise ONLY user-facing TEXT: UI chrome,
   labels, menu option titles and descriptions, status and error messages, help
   content, dialog text, and button captions.
2. THE localization layer SHALL NOT localise command VERBS or stable identifiers:
   canonical verb strings (e.g. the `AliasTable::ffedit_english` canonical verbs),
   menu-option `command` values, config KEY names, `Topic_Key` identifiers, and
   `command_id` strings SHALL remain source-language and SHALL NOT be routed
   through the Message_Catalogue.
   - AMENDED by CR-NR-109: the CANONICAL (root) verb remains source-language and
     is NOT routed through the Message_Catalogue (unchanged). What a user SEES for
     a verb (a menu option's displayed verb, and the typeable surface form) MAY be
     the active-locale alias surface, resolved via the alias row's localisation
     key (Requirement 11). The stable identifier underneath is unchanged; only the
     displayed surface is locale-aware.
3. WHERE displayed text and a command verb coexist (e.g. a menu option's
   description beside its `command` value), ONLY the description SHALL be
   localised; the `command` value SHALL be unchanged, preserving the verb/text
   split of `command-environments` Req 6a.
   - AMENDED by CR-NR-109: a menu-kind option MAY also display its VERB in the
     active locale (the alias surface via its localisation key, English fallback),
     in addition to its localised description. The option's underlying ROOT
     command value SHALL remain unchanged and is what resolution/persistence use
     (Requirement 11, Requirement 10.3). The verb/text split is preserved at the
     IDENTITY level (root verb stable); it is relaxed only at the DISPLAY level.
4. THE localization layer SHALL NOT derive any displayed text by matching on a
   translated string: displayed text is resolved FROM a stable key, never used AS
   a key, so a locale change cannot alter behaviour.

### Requirement 5: Per-locale, per-environment alias catalogues

**User Story:** As a user in a localized install, I want to type a command verb
in my own language and have it behave exactly like the canonical English verb.

#### Acceptance Criteria

1. THE localization layer SHALL load LOCALIZED alias data into the EXISTING
   per-environment `AliasTable` (`ff-desktop/src/shell/environment.rs`), adding NO
   new alias mechanism, so a localized surface form resolves to the SAME canonical
   English verb and takes the IDENTICAL code path (behaviour-neutral), per
   `command-environments` Req 6a.1 and 6a.5.
2. THE Alias_Catalogue SHALL be organised as ONE catalogue PER LOCALE, loaded when
   that locale is selected; selecting a different locale SHALL load a DIFFERENT
   Alias_Catalogue into the tables, so only the active locale's surface forms are
   resident (NOT a monolithic union of all locales), per `command-environments`
   Req 6a.6.
3. A per-locale Alias_Catalogue SHALL itself be organised PER ENVIRONMENT (a set
   of per-environment alias tables, e.g. French FFEDIT verbs, French FFCMD verbs),
   and each SHALL load into the CORRESPONDING environment's `AliasTable`; the SAME
   surface form MAY map to a DIFFERENT canonical verb in a different environment
   within the same locale, per `command-environments` Req 6a.6.
4. THE Alias_Catalogue SHALL carry NO per-row locale indicator: the single
   `ui.locale` selection determines which catalogue loads; individual alias rows
   are language-agnostic data, per `command-environments` Req 6a.6.
5. THE English CANONICAL verbs SHALL remain resolvable in EVERY locale (as the
   Identity_Base identity entries), with the active locale's surface forms LAYERED
   ON TOP (base English + locale overlay), NOT a full replacement; in a French
   install both `FIND` and `CHERCHER` SHALL resolve to canonical `FIND`, per
   `command-environments` Req 6a.7.
6. WHEN loading a per-locale Alias_Catalogue produces a collision WITHIN one
   environment's table (a surface form mapping to two different canonicals, or
   colliding with an existing canonical/alias), THE loader SHALL reject the
   catalogue for that environment with a diagnostic and retain the Identity_Base,
   using the same conflict discipline as `command-environments` Req 6a.3; the
   collision check is scoped WITHIN one environment's table, not across
   environments.

### Requirement 6: Help content localisation

**User Story:** As a user, I want help content in my language when my locale
provides it, falling back to English otherwise.

#### Acceptance Criteria

1. THE help loader (`ff-help` `content_loader`) SHALL resolve help content from a
   PER-LOCALE content directory derived from the active locale, reusing the
   existing directory-resolution loader shape with NO change to its parsing or
   topic model.
2. WHEN the active locale has a help content directory, THE help loader SHALL load
   `.help.md` files from THAT directory; WHEN a topic is absent for the active
   locale, THE help loader SHALL fall back to the English Identity_Base content
   for that topic.
3. THE help content localisation SHALL NOT change `Topic_Key` identifiers or
   topic cross-reference keys: only the topic BODY and TITLE prose are localised,
   preserving the verb/identifier split (Requirement 4).
4. WHEN the active locale changes at runtime, THE help loader SHALL resolve
   subsequently displayed topics against the new locale's content directory,
   consistent with the `ui.locale` hot-reload (Requirement 1.5).

### Requirement 7: Menu title and description localisation

**User Story:** As a user, I want built-in menu titles and option descriptions in
my language, without changing how menus are defined.

#### Acceptance Criteria

1. THE menu localisation SHALL read the TITLE and option DESCRIPTIONS to display
   from the Message_Catalogue keyed by menu + option, taking the text out of the
   PARSED menu model at render time, NOT by shipping a translated TOML file.
2. THE built-in menus SHALL remain code-only (`DEFAULT_*_TOML` constants, per
   CR-CH-021): localization SHALL NOT write a translated menu file to disk and
   SHALL NOT alter the compiled default menu definitions.
3. THE menu option `command` values SHALL NOT be localised (Requirement 4.2);
   only the TITLE and DESCRIPTION prose displayed beside them SHALL be localised.
4. WHEN a menu title or description key is absent for the active locale, THE menu
   SHALL display the Identity_Base English text (Requirement 3.2), so menus always
   render.

### Requirement 8: Plugin UI and plugin command-environment localisation

**User Story:** As a plugin author, I want my plugin's UI and command environment
to localise through the same mechanism as the core.

#### Acceptance Criteria

1. THE Catalogue_Lookup_Seam SHALL be available to plugin-authored UI, so a
   plugin's user-facing text localises through the SAME seam as the core (no
   plugin-specific catalogue mechanism).
2. A plugin-authored command environment (CR-CH-053 Req 9.4) SHALL localise its
   verb aliases through the SAME per-environment Alias_Catalogue model
   (Requirement 5), loading a plugin environment's per-locale aliases into that
   environment's `AliasTable`.
3. THE plugin localisation SHALL NOT grant a plugin any capability outside the
   existing plugin permission/security model: supplying catalogue DATA and calling
   the lookup seam are the only localization affordances.

### Requirement 9: Argument-bearing and verb-embedding messages

**User Story:** As a developer, I want status and error messages that interpolate
values or embed a command verb to localise cleanly while keeping the verb
English.

#### Acceptance Criteria

1. WHEN a user-facing message interpolates runtime values, THE message SHALL be a
   Fluent catalogue entry with named placeables (`{ $arg }`) supplied via the
   argument-bearing lookup variant (Requirement 3.1), NOT a Rust `format!` string
   literal.
2. WHEN a user-facing message embeds a command verb or stable identifier (e.g.
   "Profile is locked -- use LOCK OFF to unlock"), THE embedded verb SHALL be
   passed as a NON-translated placeable ARGUMENT, so the surrounding prose
   localises while the verb stays English (Requirement 4.2).
3. THE argument-bearing messages SHALL localise at the DISPLAY boundary: a message
   whose text originates in a `thiserror` enum SHALL be routed through the
   Catalogue_Lookup_Seam where it is shown to the user, NOT translated at the error
   source.

### Requirement 10: Additive, framework-conformant layer

**User Story:** As a maintainer, I want confidence that localization adds a
presentation layer and changes no framework mechanism.

#### Acceptance Criteria

1. THE localization layer SHALL NOT introduce a second command-dispatch path, a
   second navigation stack, a new session-persistence format, or a per-Context
   focus mechanism; it SHALL build ON the existing seams only (the `ff-config`
   schema/hot-reload, the per-environment `AliasTable`, the `ff-help` content
   loader, the parsed menu model, and the egui render paths).
2. THE localization layer SHALL be purely ADDITIVE to behaviour: with `ui.locale`
   = `"en"` (the default), the workbench SHALL render and behave IDENTICALLY to
   its pre-localization behaviour for any given screen, so the English baseline is
   behaviour-neutral.
3. THE localization layer SHALL NOT persist any localized surface form as the
   recorded/persisted value: command history, macro `command=` values, menu option
   `command` values, keybindings, and Workspace descriptors SHALL store the
   canonical English verb regardless of the active locale (Requirement 4.4,
   `command-environments` Req 6a.2).

### Requirement 11: Locale-aware alias resolution and menu-kind options rendered via localised aliases (CR-NR-109)

**User Story:** As a user in a localized install, I want to type a command verb
in my own language OR pick a menu option whose verb shows in my own language, and
in both cases have it resolve to the same root command, falling back to English
when no localised alias exists.

This requirement makes the command-alias layer locale-aware and makes every
menu-kind Context render its options THROUGH that layer. It builds ON the existing
per-environment `AliasTable` (`ff-desktop/src/shell/environment.rs`) and the
single command-dispatch front door; it adds NO second dispatcher, NO second
navigation stack, and NO new persistence format. The ROOT (canonical) command
remains the stable identifier used for dispatch and persistence (Requirement 10.3
unchanged). This requirement AMENDS the DISPLAY claims of Requirement 4.2 and 7.3
(see their amendment notes); it does not change the identity-level verb/text
split.

#### Acceptance Criteria

1. THE `AliasTable` entry SHALL carry an OPTIONAL localisation key in addition to
   its surface form and canonical (root) verb, so an entry is
   `(surface-form, canonical-verb, localisation_key?)`. WHERE a localisation key
   is present, THE displayed surface text for that alias SHALL be resolved through
   the EXISTING `ff-i18n` Catalogue_Lookup_Seam (`t(key)`, Requirement 3) against
   the active locale, adding NO new lookup mechanism.
2. THE resolved localised string for an alias SHALL be the displayed surface
   form for that locale, AND SHALL be ACCEPTED as a typeable surface form for that
   locale: what a user sees for a verb is a form the user can type to invoke it.
   - AMENDED by CR-NR-109 (diacritic-insensitive input, owner-confirmed decision
     to realise; see the input-accessibility amendment section and the new
     resolver criteria 11.11-11.13 below). The original wording made the resolved
     localised string the ONE AND ONLY typeable surface ("what a user sees is
     exactly what the user must type"). On keyboards that cannot easily enter
     accented letters (common in South Africa and generally), a localised surface
     carrying diacritics (e.g. Afrikaans LEERS written with a circumflex) would be
     the only typeable form and would LOCK OUT plain-keyboard users. The amendment
     keeps DISPLAY CORRECTNESS (the displayed surface keeps full, correct
     diacritics) but relaxes INPUT so the resolver also accepts the
     diacritic-folded (plain-ASCII-foldable) form of the same surface. Two
     explicit guarantees result: (a) DISPLAY CORRECTNESS -- the shown verb keeps
     its correct accented spelling; (b) INPUT TOLERANCE -- the user MAY type the
     plain-ASCII form and still resolve to the same root command. The displayed
     localised string remains A typeable form; it is no longer the ONLY one.
   - FLAG: this is a BEHAVIOUR CHANGE to the resolver. It does NOT change the
     DISPLAY surface, the root command, or persistence (Requirement 10.3 / 11.9
     unchanged). It is realised by a LATER GATED IMPLEMENTATION TASK (Phase 5,
     Task 12 in tasks.md), not by this gate. The CR-NR-109 Phase 4 resolver
     (Requirement 11.6) and its TCR rows describe the pre-amendment exact-match
     behaviour; see the surfaced contradiction in the authoring summary.
3. THE alias layer SHALL AUTO-SYNTHESISE an identity alias row for every root
   command (canonical -> itself), so the root command is always resolvable in
   whatever source language it was authored, WITHOUT requiring an author to hand-
   write an identity row (this preserves "English/root is always resolvable",
   Requirement 5.5).
4. AN auto-synthesised identity row SHALL be EDITABLE and, once edited, PERSISTED
   and SHALL override the synthesised default: a user MAY rename a root command's
   surface form and assign it a localisation key (e.g. give a Russian-authored
   root command an English surface with an `en` localisation key), WITHOUT
   changing the root command itself.
5. THE localization layer SHALL provide an ON-DISK alias-catalogue reader that
   parses per-locale, per-environment alias DATA (e.g.
   `i18n/<locale>/aliases/<environment>.toml`, each row carrying its surface form,
   root verb, and optional localisation key) into an in-memory catalogue and
   loads it into the CORRESPONDING environment's `AliasTable` via the EXISTING
   `AliasTable::apply_locale_overlay` seam (localization Requirement 5.1, 5.5),
   adding NO new overlay mechanism. (This is the reader localization Task 10.2
   recorded as BLOCKED.)
6. THE resolver SHALL, for an entered verb `V` at active locale `L`, apply this
   order, which is IDENTICAL whether `V` originates from the command line or from
   a selected menu-kind option:
   1. look up `V` in the alias table scoped to localisation key `L`;
      1. WHEN found, THE resolver SHALL resolve `V` to its root command and
         execute that root command;
      2. WHEN not found, THE resolver SHALL look up `V` in the root command table;
         1. WHEN found, THE resolver SHALL execute that root command;
         2. WHEN not found, THE resolver SHALL issue an appropriate unknown-command
            message.
   - AMENDED by CR-NR-109 (diacritic-insensitive input). Each lookup step above
     (both the locale-`L` alias-table lookup and the root-command-table lookup)
     SHALL be performed under the diacritic-insensitive comparison defined in the
     new Requirement 11.11, so a plain-ASCII-folded entry of a diacritic-bearing
     surface still matches. The ORDER and the fall-through to the unknown-command
     message are UNCHANGED; only the per-step comparison is folded. An exact
     (byte-for-byte) match MUST continue to resolve (the fold is a superset of
     exact matching), so this is behaviour-preserving for all existing ASCII and
     exact-form input and only ADDS matches for the folded forms.
7. A menu-kind Context (POM/Home, Settings, and user-defined menus rendered via
   the Workspace Kinds / menu model -- ALL menu kinds) SHALL reference a command
   ALIAS as each option (not a raw command string). AT render THE option SHALL
   DISPLAY the active-locale surface for that alias (its localisation-key
   resolution, Identity_Base/root fallback when absent, Requirement 3.2); WHEN the
   option is selected THE same verb SHALL be fed through the resolver of criterion
   6, reaching the root command.
8. THE displayed localised verb and localised description of a menu option SHALL
   both fall back to the Identity_Base/root text when the active locale provides
   no value (Requirement 3.2, 7.4), so a menu-kind screen always renders a
   complete, invocable option set.
9. THE root (canonical) command SHALL remain the recorded/persisted value for a
   command issued via either entry point (command line or menu selection),
   regardless of the active locale or the alias surface used (Requirement 10.3,
   `command-environments` Req 6a.2); a locale change SHALL NOT alter which root
   command a given option resolves to.
10. WHEN loading a per-locale alias catalogue produces a within-environment
    collision (a surface form mapping to two different roots), THE reader SHALL
    reject that environment's catalogue with a diagnostic and retain the
    Identity_Base, reusing the existing `apply_locale_overlay` collision
    discipline (localization Requirement 5.6); the menu-kind render SHALL then
    fall back to the root/English surface for the affected options rather than
    failing to render.

#### Acceptance Criteria -- Diacritic-insensitive input (CR-NR-109 input-accessibility amendment)

The following criteria ADD diacritic-insensitive command matching as the baseline
so that a user on a plain keyboard can type the ASCII-folded form of any localised
verb. They AMEND the resolver behaviour of Requirement 11.2 and 11.6 (see those
amendment notes). They are a BEHAVIOUR CHANGE to the resolver realised by a LATER
GATED IMPLEMENTATION TASK (Phase 5, Task 12), not by this gate. The DISPLAY
surface (Requirement 11.1, 11.7) and the root command / persistence
(Requirement 10.3, 11.9) are UNCHANGED.

11. THE resolver SHALL compare an entered verb `V` against each candidate surface
    form under a DIACRITIC-INSENSITIVE FOLD defined as: Unicode NFD
    normalisation, then removal of combining marks (Unicode category Mn), then an
    ASCII case fold. A candidate matches `V` WHEN the fold of `V` equals the fold
    of the candidate. THE fold SHALL be applied to BOTH sides of the comparison so
    that an exact (unfolded) match is always ALSO a folded match (the fold is a
    superset of exact matching). This makes the baseline cover EVERY localised
    word without requiring any extra per-word data.
    - Example: the Afrikaans display surface `LEERS` written with a circumflex
      folds to `LEERS`; a user typing the plain-ASCII `LEERS` folds to the same
      value and resolves to the same root command.

12. WHERE two DISTINCT surface forms within the SAME environment+locale scope fold
    to the SAME value (a fold collision -- two verbs that differ only by
    diacritics or case), THE resolver SHALL PREFER an exact (unfolded) match when
    one exists, and only fall back to a folded match when no exact match is
    present. WHEN an entered `V` folds to a value shared by two or more candidates
    and no exact match disambiguates it, THE resolver SHALL treat the input as
    AMBIGUOUS and issue a diagnostic naming the colliding candidates rather than
    silently resolving to one. This keeps the fold from masking a genuine two-verb
    distinction.

13. THE DISPLAY surface of an alias (Requirement 11.1, 11.7) SHALL retain its full
    correct diacritics regardless of the input fold: the fold affects ONLY input
    MATCHING, never what is rendered. WHERE a word genuinely needs a different
    ASCII spelling (not merely a diacritic-stripped form), an author MAY ADD an
    explicit additional ASCII alias surface row for the same root command (via the
    existing per-locale alias DATA, Requirement 11.5); such an explicit ASCII
    alias is OPTIONAL because diacritic-insensitive matching (Requirement 11.11)
    already covers the diacritic-only case for every word without extra data.

---

## Out of Scope (this gate)

- The full per-crate extraction of all existing hardcoded string literals in one
  pass. Extraction is incremental follow-on work (tasks.md, audit's 7-step order).
- Authoring French, German, or any non-English CONTENT. This gate ships the `"en"`
  Identity_Base only; other-language catalogue DATA is the final capstone task
  group, dependent on the extraction being substantially complete.
- A system/OS "auto" locale sentinel (a future allowed value, not this gate).
- Any change to the Command Environment MODEL itself (CR-CH-053); this gate only
  LOADS alias DATA into the existing `AliasTable`.
- Right-to-left layout, locale-specific number/date formatting beyond Fluent's
  built-in plural/gender handling, and font coverage for non-Latin scripts.
