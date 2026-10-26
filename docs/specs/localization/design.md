# Design Document -- Localization / i18n (CR-NR-103)

## Overview

Localization adds a presentation layer that resolves user-facing text and
command-verb aliases from per-locale DATA selected by a single config key. It
introduces ONE new GUI-free crate (`ff-i18n`) that owns the Fluent bundle
load/lookup, plus a `ui.locale` key in `ff-config`, and threads a lookup seam
(`t("key")`) incrementally into the existing render, help, menu, and alias seams.
Nothing in the framework (command dispatch, navigation stack, WorkspaceContext,
session persistence, code-only built-in menus) changes; this design builds ON the
existing seams and is behaviour-neutral when the locale is the default `"en"`.

The design follows the audit's findings
(`.agents/tasks/localization-i18n-audit/i18n-posture-audit.md`): the substrate is
ready (UTF-8 throughout, help already externalised to `.help.md`, `ff-config` has
schema + hot-reload + callbacks, and the per-environment `AliasTable` already
exists and explicitly expects CR-NR-103 data).

## Technology stack (locked once approved)

- **Catalogue mechanism: Fluent** -- `fluent-bundle` + `fluent` + `unic-langid`.
  Chosen over a gettext-style `.po`/flat-map (see "Fluent over gettext" below).
- **Catalogue files: per-locale `.ftl` DATA** under `i18n/<locale>/` in the repo
  (same "content is data" posture as the `help/` tree).
- **New crate: `ff-i18n`** -- GUI-free, dependency-light, positioned like
  `ff-config` (no egui, no editor/shell deps), so any crate can resolve messages.
- **Config: `ff-config`** -- the `ui.locale` key and its reload callback reuse the
  existing schema registry (`init/schema.rs`), key constants (`keys.rs`), and
  `CallbackRegistry` (`callback.rs`). No new config mechanism.

## The `ff-i18n` crate

`ff-i18n` owns catalogue loading and lookup. Its public shape:

- `Locale` -- a thin wrapper over `unic_langid::LanguageIdentifier` with a parse
  from the `ui.locale` string and an `en` identity constant.
- `Catalogue` -- holds, per loaded locale, a Fluent bundle built from that
  locale's `.ftl` files, plus the always-present English Identity_Base bundle.
- `t(key: &str) -> String` -- resolve `key` against the active locale, falling
  back to the Identity_Base, then to the key itself (with a WARN) if absent in
  both (Req 3.2, 3.3).
- `t_args(key: &str, args: &FluentArgsLike) -> String` -- the argument-bearing
  variant supplying named placeables for interpolation / plural / gender (Req
  3.1, 9.1).
- `load(locale) -> Result<Catalogue, I18nError>` -- discover and parse the
  locale's `.ftl` files; on a parse error, retain the prior catalogue / fall back
  to English and log a WARN (Req 2.4).

The active locale is held behind a process-wide handle (an `Arc`-shared
`RwLock<Catalogue>` or equivalent) so a hot-reload can swap the active catalogue
without threading it through every call site. `ff-i18n` is GUI-free (Req 3.5): it
returns owned `String`/`&str` values that egui widgets already accept, so render
code calls `t("key")` where it used to pass a literal, with NO egui API change
(Req 3.4).

Error handling: `I18nError` is a `thiserror` enum (catalogue file not found,
parse error, bundle build error). Load-time errors are RECOVERABLE -- the loader
retains the prior catalogue (or the Identity_Base) and logs a WARN via
`ff-logging`; it never panics and never yields blank UI (Req 2.4, 2.5). Lookup
misses are recoverable: fall back to Identity_Base, then to the key string with a
WARN (Req 3.3).

## The `ui.locale` key wiring in `ff-config`

Three coordinated edits, mirroring how `theme.active` / `logging.level` are
defined today:

1. **Key constant** -- add a `ui` namespace to `crates/ff-config/src/keys.rs`:
   `pub mod ui { pub const LOCALE: &str = "ui.locale"; }`. This satisfies the
   existing `assert_valid_key` / `all_keys_have_unique_values` dot-path tests
   (lowercase, single dot).
2. **Schema entry** -- add a `SchemaEntry` in
   `crates/ff-config/src/init/schema.rs::register_core_schema`:
   `key: keys::ui::LOCALE`, `value_type: ValueType::String`,
   `default: ConfigValue::String("en")`,
   `description: "Active UI language locale (BCP-47, e.g. en, fr, de)"`,
   `constraints: Some(Constraints { allowed_values: Some([ConfigValue::String("en")]), .. })`.
   The `allowed_values` list is the shipped-locale pin (Req 1.2); each added
   language appends its locale here (the capstone task updates this list).
3. **Reload callback** -- register
   `CallbackRegistry::on_reload(&[keys::ui::LOCALE], ...)` (the `callback.rs` API,
   the SAME mechanism `theme.active` uses) whose handler reloads BOTH the
   `ff-i18n` Message_Catalogue AND the per-locale, per-environment Alias_Catalogue
   (Req 1.4, 1.5). The callback runs under the config write-lock scope and must
   not re-enter `ConfigHandle` (documented constraint in `config_handle/mod.rs`),
   so it swaps the catalogue handle and marks the alias tables for reload rather
   than calling back into config.

Default `"en"` keeps English as the always-present Identity_Base (Req 1.3,
`command-environments` Req 6a.7). A defence-in-depth guard falls back to `"en"`
and WARNs if an out-of-schema locale value ever reaches the loader (Req 1.6).

## Threading the seam into egui render paths

The seam threads in WITHOUT changing egui APIs: a render site that today writes
`ui.label("Files")` becomes `ui.label(t("files_panel.title"))`, passing the
resolved `String` the widget already expects (Req 3.4). This is a mechanical,
per-site substitution done incrementally (tasks.md), ordered by the audit so the
clean dialog/panel crates convert before the harder `ff-desktop` sites. No render
function signature changes; no new render path is introduced (Req 10.1).

Because `ff-i18n` holds the active catalogue behind a shared handle, render code
does not receive a catalogue parameter -- it calls the free `t`/`t_args`
functions, keeping the edit at each site a one-line literal-to-lookup swap.

## Help loader per-locale directory resolution

`ff-help`'s `content_loader` already resolves a content DIRECTORY and reads
`.help.md` files from it (`crates/ff-help/src/content_loader.rs`). The design adds
LOCALE-AWARE path resolution in front of the existing loader: for active locale
`L`, resolve `help/<L>/` (e.g. `help/fr/`) and, for any topic absent there, fall
back to the English base `help/` tree (Req 6.2). The loader's PARSING, topic
model (`TopicSource::FileBased`), and delimiter handling are unchanged (Req 6.1).
`Topic_Key` identifiers and cross-reference keys are NOT localised -- only the
topic body and title prose differ per locale (Req 6.3). On a `ui.locale`
hot-reload, subsequently displayed topics resolve against the new locale's
directory (Req 6.4).

## Menu title/description lookup via the parsed menu model

Built-in menus stay code-only `DEFAULT_*_TOML` constants in
`crates/ff-desktop/src/menu_workspace/defaults.rs` (CR-CH-021); localization does
NOT write a translated TOML file to disk (Req 7.2). Instead, at render time the
menu-render path reads the TITLE and option DESCRIPTIONS to DISPLAY from the
Message_Catalogue, keyed by menu + option (e.g. `menu.pom.title`,
`menu.pom.option.0.description`), taking the text out of the PARSED menu model
rather than from the compiled TOML string (Req 7.1). The option `command` values
are NOT localised (Req 7.3); a missing key falls back to the Identity_Base
English text (Req 7.4), which for the built-in menus is exactly the text in the
`DEFAULT_*_TOML` constant.

## Per-environment, per-locale alias catalogue loading

The per-environment `AliasTable`
(`crates/ff-desktop/src/shell/environment.rs`) already resolves a surface form to
a canonical English verb and is explicitly built to receive per-locale data "with
no new mechanism" (its own doc comment references CR-NR-103). The design adds a
LOADER that, for the active locale, loads a per-environment set of surface-form ->
canonical-verb rows into the CORRESPONDING environment's `AliasTable` as an
OVERLAY on the English identity entries (base English + locale overlay, Req 5.5).

- One Alias_Catalogue per locale, loaded on selection; a different locale loads a
  different catalogue, so only the active locale's surface forms are resident (Req
  5.2).
- A per-locale catalogue is a SET of per-environment tables (French FFEDIT, French
  FFCMD, ...), each loading into its matching environment; the same surface form
  may map to different canonicals across environments (Req 5.3).
- No per-row locale tag; the single `ui.locale` selection picks the catalogue (Req
  5.4).
- Collisions are rejected per environment using the existing `AliasTable`
  collision discipline (the `ffedit_english` constructor already asserts on a
  surface form mapping to two canonicals); the loader surfaces the authoring error
  and retains the Identity_Base for that environment (Req 5.6).

The canonical verb remains what is recorded/persisted (Req 10.3,
`command-environments` Req 6a.2); the alias layer is command-line INPUT
convenience only.

Alias catalogue DATA ships alongside the message `.ftl` files, e.g.
`i18n/<locale>/aliases/<environment>.toml` (data shape mirrors the in-code
`ffedit_english` pairs). The loader reads these on locale selection and on the
`ui.locale` hot-reload.

## Plugin localisation

Plugin-authored UI calls the SAME `ff-i18n` seam (Req 8.1); a plugin ships its own
`.ftl` catalogue entries (namespaced keys) that merge into the active bundle. A
plugin-authored command environment (CR-CH-053 Req 9.4) ships per-locale alias
DATA that loads into that environment's `AliasTable` through the same loader (Req
8.2). No new capability is granted: supplying catalogue DATA and calling the seam
are the only affordances, within the existing plugin permission/security model
(Req 8.3).

## Argument-bearing and verb-embedding messages

Messages that interpolate runtime values are Fluent entries with named placeables,
resolved via `t_args` (Req 9.1). A message that embeds a command verb passes the
verb as a NON-translated placeable argument (e.g.
`profile-locked = Profile is locked -- use { $verb } to unlock` with
`$verb = "LOCK OFF"`), so the prose localises while the verb stays English (Req
9.2, 4.2). Messages originating in `thiserror` enums are localised at the DISPLAY
boundary (where the shell shows them), not at the error source (Req 9.3), so the
error types remain unchanged and GUI-free.

## Fluent over gettext (decision, with the rejected tradeoff recorded)

Fluent is CHOSEN. The hard cases in this codebase are argument-bearing,
verb-embedding status/error messages (audit Q5): Fluent's `{ $var }` placeables
carry a non-translated verb as an argument directly, matching the verb/text split
(Req 4, `command-environments` Req 6a.2), and Fluent is ICU-grade for
plural/gender that future non-English/French languages need. Fluent catalogues are
per-locale `.ftl` DATA, matching the vision's "adding a language is adding a
catalogue" and composing cleanly with the per-locale alias catalogue model.

REJECTED alternative -- a gettext-style `.po` / flat key->string map: lighter
runtime and fewer dependencies, but weaker for the "keep this token English"
verb-embedding requirement (printf `%s` positional args do not express a named,
explicitly-non-translated verb as cleanly) and weaker plural/gender handling. The
tradeoff Fluent carries is extra dependencies (`fluent`, `intl-memoizer`,
`unic-langid`) and a slightly heavier runtime; given the workspace already pulls
large deps (egui) and the argument/plural needs are real, the cost is justified.
This is recorded as considered-and-rejected, not an open question.

## Testability

- `ff-i18n`: unit-testable without egui -- catalogue load, English fallback,
  missing-key fallback-to-key + WARN, `.ftl` parse-error retention, `t_args`
  placeable interpolation including a non-translated verb argument.
- `ff-config`: unit tests for the `ui.locale` schema entry (default `"en"`,
  `allowed_values` rejects an unknown locale) and a reload-callback test asserting
  the `ui.locale` change fires the handler (mirrors existing `on_reload` tests).
- `ff-desktop`: the alias loader is unit-testable (load a per-environment overlay,
  assert `CHERCHER` resolves to `FIND`; assert a collision is rejected and the
  Identity_Base retained; assert the canonical verb is what persists). Menu
  title/description lookup is unit-testable against the parsed menu model.
  Render-site conversions keep their existing `egui_kittest` behaviour tests; the
  behaviour-neutrality at `ui.locale = "en"` (Req 10.2) is assertable by rendering
  with the default locale and confirming unchanged text.
- `ff-help`: per-locale directory resolution + English fallback is unit-testable
  against a temp content tree.

## Framework-conformance statement

This design changes NO framework mechanism (Req 10.1). It adds a GUI-free crate
and a config key, and it threads a lookup seam and loaders into EXISTING seams:
the `ff-config` schema/hot-reload, the per-environment `AliasTable`, the `ff-help`
content loader, the parsed menu model, and the egui render paths. There is no
second dispatcher, no second navigation stack, no new persistence format, and no
per-Context focus mechanism. With `ui.locale = "en"` the workbench is
behaviour-identical to today (Req 10.2), and the canonical English verb remains
the recorded/persisted value regardless of locale (Req 10.3).

## Locale-aware alias resolution + menu-kind options via localised aliases (CR-NR-109, Requirement 11)

CR-NR-109 makes the command-alias layer LOCALE-AWARE and routes every menu-kind
Context's options THROUGH that layer. It builds ON the existing per-environment
`AliasTable` (`crates/ff-desktop/src/shell/environment.rs`), the existing
`ff_i18n::t()` seam, and the single command-dispatch front door
(`shell/dispatch.rs`). It adds NO second dispatcher, NO second navigation stack,
NO new persistence format, and NO per-Context focus mechanism. The ROOT
(canonical) command stays the stable dispatched/persisted identity (Req 10.3).

This is a FRAMEWORK CHANGE (owner-confirmed in-thread, per
`framework-conformance.md`): it reshapes the `AliasTable` ENTRY model and the
menu-option display model. The two reshaped shapes are described below; both are
additive (the English/`en` default path is behaviour-neutral, Req 10.2).

### 1. The `AliasTable` entry grows a localisation key

Today an entry is `(surface-form-UPPERCASED, canonical-verb)` (a `Vec<(String,
String)>`). CR-NR-109 grows it to carry an OPTIONAL localisation key:

```
struct AliasEntry {
    surface: String,          // UPPERCASED typeable/displayed surface form
    canonical: String,        // the root (canonical) verb -- the dispatched identity
    localisation_key: Option<String>, // ff-i18n key -> displayed surface per locale
}
```

- WHERE `localisation_key` is `Some(k)`, the DISPLAYED surface for the entry is
  `ff_i18n::t(k)` resolved against the active locale (Req 11.1), reusing the
  EXISTING seam (no new lookup mechanism). The resolved string is ALSO the
  typeable surface for that locale (one source of truth, Req 11.2).
- WHERE `localisation_key` is `None`, the `surface` string itself is both
  displayed and typeable (the pre-CR-NR-109 behaviour, so the `en` default is
  unchanged).
- The existing `canonical_verb(surface)` resolver and `apply_locale_overlay`
  stay; `apply_locale_overlay` gains the ability to carry the per-row
  localisation key through to the entry (its collision discipline is unchanged,
  Req 11.10, localization Req 5.6). The seed constructors (`ffedit_english`)
  default `localisation_key` to `None`, so the English base is byte-identical.

### 2. Auto-synthesised, editable identity rows

Every ROOT command is SEEDED with an identity alias row (`surface == canonical`,
`localisation_key = None`) automatically at table construction, so the root is
always resolvable in whatever source language it was authored (Req 11.3; this is
the existing `ffedit_english` identity-row pattern made universal and explicit).

A seeded row is EDITABLE: an authored/overlaid row for the SAME canonical
REPLACES the synthesised default and is persisted (Req 11.4). The Russian-root /
English-surface example is exactly this: the root stays the Russian canonical;
an `en` overlay row `(surface="FIND", canonical="<russian-root>",
localisation_key="verb.find")` gives it an English face without touching the
root. Persistence rides the existing per-locale alias DATA on disk (part 3); the
synthesised default exists only in memory as the fallback when no row overrides
it.

### 3. The on-disk alias-catalogue reader (unblocks Task 10.2)

A new reader parses per-locale, per-environment alias DATA into an in-memory
`AliasCatalogue` and loads it via the EXISTING `AliasTable::apply_locale_overlay`
(today `#[allow(dead_code)]`, no non-test caller -- this reader is its first
production caller). Data shape, mirroring the help/`.ftl` "content is data"
posture:

```
i18n/<locale>/aliases/<environment>.toml
  # one [[alias]] per surface form
  [[alias]]
  surface = "SOEK"
  canonical = "FIND"
  localisation_key = "verb.find"   # optional
```

- The reader is wired into environment construction and into the `ui.locale`
  reload callback (the reload target the Task 2.3 callback already fires, design
  "The `ui.locale` key wiring" above). On a locale switch it loads that locale's
  per-environment catalogues; a different locale loads a DIFFERENT catalogue
  (localization Req 5.2).
- A within-environment collision is rejected by `apply_locale_overlay`'s existing
  discipline; the reader surfaces the diagnostic and retains the Identity_Base
  for that environment (Req 11.10).
- This reader is MECHANISM (gate + code + tests), which is precisely why
  localization Task 10.2 recorded the on-disk reader as BLOCKED; CR-NR-109 is the
  gated change that builds it.

### 4. The unified locale-aware resolver (one path, two entry points)

A single resolver function implements the Req 11.6 order for an entered verb `V`
at active locale `L`, and is called from BOTH the command-line entry
(`shell/dispatch.rs::dispatch_command_string`, before `resolve_target`) and the
menu-kind option-selection entry (the `MenuRenderResult::selected` handling in
the shell render path):

```
resolve_locale_verb(V, L):
    1. if alias table (scoped to localisation key L) has surface V
           -> return its canonical (root) command            # Req 11.6.1.1
    2. else look up V in the root command table
           2.1 if present -> return V as the root command     # Req 11.6.1.2.1
           2.2 else        -> UnknownCommand(V)                # Req 11.6.1.2.2
```

The returned root command is then dispatched through the UNCHANGED front door
(`resolve_target` / `dispatch_command_target`), so there is no parallel dispatch
path (Req 11.6, framework mechanism 1). The root command is what history /
session / macro `command=` record (Req 11.9, 10.3). Because the menu-selection
entry feeds the option's alias surface into the SAME resolver, a clicked option
and a typed verb take the identical path.

### 5. Menu-kind render refactor: options reference an alias, display the locale surface

Today `menu_workspace/render.rs` paints `option.command` (the raw root verb) in
the command column via `option_prefix_job`, and localises only the DESCRIPTION
via `i18n_keys::localised_option_description`. CR-NR-109 changes the COMMAND
column to render the active-locale alias surface:

- A menu-kind option references a command ALIAS (its `command` field remains the
  root value on disk/in the parsed model -- unchanged persistence -- but the
  render path resolves the DISPLAYED verb through the alias layer for `L`).
- A new `i18n_keys` helper, e.g. `localised_option_command(environment, option,
  L)`, returns the alias surface for the option's root command in locale `L`
  (the alias row's `localisation_key` resolution), falling back to the root
  command string when no localised alias exists (Req 11.7, 11.8). `render.rs`
  calls it where it currently formats `option.command`.
- Selecting the option still yields `MenuOption` carrying the ROOT `command`;
  the shell feeds it through the unified resolver (part 4). What the user SEES is
  the locale surface; what dispatches/persists is the root (Req 11.9).
- This applies to ALL menu kinds, because every menu kind renders through the one
  shared `render_menu_workspace` (POM, Settings, and user-defined menus all use
  it -- menu-workspace Req 2.1). No per-kind special-casing.

### Framework-conformance statement (CR-NR-109)

The change reshapes two load-bearing shapes (the `AliasTable` entry model and the
menu-option DISPLAY model) and is therefore an owner-confirmed FRAMEWORK CHANGE.
It does NOT: add a second command-dispatch path (resolution ends at the existing
front door), add a second navigation stack, introduce a new persistence format
(the root command remains the persisted identity), or add a per-Context focus
mechanism (the menu-kind focus latch is unchanged). With `ui.locale = "en"` and
`localisation_key = None` on the seed rows, every menu renders its root verbs
exactly as today (Req 10.2 behaviour-neutral).

### Testability (CR-NR-109)

- `AliasTable`: unit test that an entry with a `localisation_key` displays the
  `t(key)` resolution and that `None` displays the raw surface; that an overlay
  row replaces the synthesised identity default for the same canonical.
- On-disk reader: unit test parsing a temp `i18n/<locale>/aliases/<env>.toml`
  into a catalogue and loading it via `apply_locale_overlay`; a collision in the
  data is rejected and the Identity_Base retained.
- Unified resolver: table-driven unit test covering Req 11.6's four outcomes
  (1.1 localised alias -> root; 1.2.1 root-table hit; 1.2.2 unknown-command
  message), asserting the root command is what is returned regardless of surface.
- Menu-kind render: an `egui_kittest` full-shell test that, with a non-`en`
  locale and an alias catalogue loaded, a menu option's command column shows the
  localised surface; selecting it dispatches the ROOT command (assert history
  records the root). With `en` / no overlay the column shows the root verb
  unchanged (Req 10.2).

## Menu Kinds localise FULLY -- mnemonic + command alias + description (CR-NR-111, Requirement 12)

CR-NR-111 is the DELTA on top of CR-NR-109 (Requirement 11). CR-NR-109 already
made the alias layer locale-aware, rendered a menu option's displayed COMMAND
surface through that layer, and wired the unified localised-alias -> root resolver
for both the command line and menu selection. Requirement 12 extends FULL
localisation to the remaining observable parts of a menu option (the MNEMONIC and
the DESCRIPTION alongside the COMMAND), settles the stored-command shape that lets
a SINGLE saved menu auto-convert to the active locale, and makes the
no-localised-data fallback and the per-language saved-menu fallback explicit.

This section is the GOVERNING design for the owner's "menu-kinds localisation
redesign"; menu-workspace/design.md and menu-and-statusbar/design.md carry the
render-side and chrome-side deltas that build on it.

### Not a framework change (grounded in the investigation)

CR-NR-109 was an owner-confirmed framework change because it reshaped the
`AliasTable` entry model and the menu-option display model. CR-NR-111 reshapes
NEITHER: the alias-entry model (`surface`, `canonical`, `localisation_key?`) and
the unified resolver already exist and already satisfy the owner's "this should
not affect how commands are routed." The investigation
(`.agents/tasks/menu-kind-localisation/findings.md`, "Framework-delta question")
established, with code references, that:

- `ff_command::resolve_target` does NOT itself consult the alias table (its
  `TargetResolver` trait has no alias lookup), BUT the shell ALREADY translates a
  localised alias to its root BEFORE calling `resolve_target`, via a single
  pre-pass `rewrite_verb_to_root` -> `AliasTable::canonical_verb`
  (`shell/locale_resolver.rs`) run on BOTH seams
  (`dispatch_command_string`, `activate_menu_option`). End to end a localised
  alias already resolves to the root with routing unchanged (the ROOT is
  dispatched and persisted).

Therefore CR-NR-111 adds NO new dispatcher, NO parallel resolver, NO new
persistence format, and NO Workspace-Kinds-registry change. It is NOT a framework
change and needs no further owner confirmation beyond the CR-NR-111 requirements
gate. Moving the alias lookup INSIDE `resolve_target` WOULD be a framework change
(reshaping the `TargetResolver` contract / adding an alias dependency to the
dispatch classifier) and is explicitly REJECTED here: the established pre-pass
seam already delivers the behaviour, so there is nothing to confirm. The remaining
work is (a) a MNEMONIC localisation seam mirroring the existing description seam,
and (b) a formal statement of the stored-command shape (settled below) -- both
NON-framework, building on CR-NR-109.

### The crux: what a saved menu stores for the "command" field -- DECISION

Two options were analysed for what a saved menu option persists so that
first-prize auto-conversion works (one saved definition renders in whatever locale
is active):

- **Option A -- store the ROOT command** (locale-independent). The displayed
  alias is DERIVED at render time from the per-locale alias surface for that root;
  the root is also exactly what the pre-pass resolver returns and what dispatch /
  history / session persist.
- **Option B -- store a LOCALISATION KEY.** Both the displayed alias and the root
  are derived at render/dispatch time by resolving the key.

**DECISION: Option A -- store the ROOT command.** Justification:

1. **First-prize auto-conversion only works cleanly with a locale-INDEPENDENT
   stored value, and the ROOT is that value.** The alias DATA is keyed by root
   (canonical) -> localised surface + optional localisation key
   (`AliasTable`, `i18n/<locale>/aliases/<env>.toml`); the root is the join key
   shared across EVERY menu that references that command. Storing the root lets
   the render path derive the active-locale surface via
   `display_surface_for_canonical(option.command)` with NO change to the saved
   file. A saved English menu already renders localised the moment the active
   locale has alias rows -- this IS the owner's first prize.

2. **It is the shape saved menus ALREADY use** (`MenuOption.command` is the root
   English string on disk and in the built-in `DEFAULT_*_TOML` constants), so the
   decision requires NO migration of existing saved menus (see Back-compat below)
   and NO new `MenuOption` field.

3. **It satisfies Requirement 10.3 / 11.9 directly** (the root is the
   recorded/persisted identity). Option B would persist a localisation key as the
   "command", which only satisfies 11.9 if the key still resolves to a stable
   root -- an indirection that buys nothing over storing the root itself, while
   DUPLICATING per-menu what the per-locale alias catalogue already holds once per
   locale and REQUIRING a new `MenuOption` field plus a key->root resolution step
   on the dispatch path.

4. **It keeps the saved menu locale-neutral.** A menu that stored a localised
   surface (or a locale-pinned key) would render wrong under a different locale,
   defeating auto-conversion. The root is stable across all locales; the surface
   is derived. This is the Locale_Independent_Stored_Value the requirement names
   (Req 12.4): it is the ROOT command.

Option B is recorded as considered-and-rejected, not an open question.

### Displayed alias derivation (COMMAND part, Req 12.1/12.2)

Unchanged from CR-NR-109. At render time the menu path resolves the option's
DISPLAYED command verb from the option's stored ROOT via
`AliasTable::display_surface_for_canonical(option.command)` for the active locale
(the alias row's `localisation_key` -> `ff_i18n::t(key)` resolution), computed
shell-side in `menu_option_command_surfaces` (`shell/locale_resolver.rs`) and
passed positionally into the shared renderer as `command_surfaces`
(`menu_workspace/render.rs`). When the active locale provides no alias row for the
root, the surface falls back to the raw root string (Identity_Base, Req 3.2,
11.8). Selection feeds the option's ROOT through the unified pre-pass resolver, so
the displayed-vs-dispatched split holds (Req 12.5).

### Mnemonic localisation (MNEMONIC part, Req 12.3) -- new seam, mirrors the description seam

Today the mnemonic (`MenuOption.key`) is rendered RAW (`{:<4}` of `option.key` in
`render_layout.rs::option_prefix_job`) and has NO per-locale source anywhere. This
is the one genuinely new seam CR-NR-111 adds. The requirement fixes only the
observable contract (the mnemonic HAS a per-locale source and a defined
Identity_Base fallback); the source shape is this design's decision.

Two candidate sources were considered:

- **(i) Derive the mnemonic from the localised command surface** (e.g. first
  non-space character of the active-locale alias surface).
- **(ii) A dedicated per-locale mnemonic value**, authored in the catalogue
  mirroring the description seam.

**DECISION: Option (ii) -- a dedicated per-locale mnemonic catalogue key, mirroring
the existing description seam.** Justification: the description already localises
via a catalogue key (`i18n_keys::menu_option_desc_key(menu_name, command)` ->
`menu-<menu>-opt-<command>-desc`, resolved by
`localised_option_description` = `t_or(key, &option.description)`). Mirroring that
seam with a sibling key -- `menu_option_key_key(menu_name, command)` ->
`menu-<menu>-opt-<command>-key`, resolved by a new
`localised_option_mnemonic(menu_name, option)` = `t_or(key, &option.key)` -- keeps
the mnemonic INDEPENDENTLY authorable (a locale's mnemonic need not be a substring
of its command verb; ISPF-style mnemonics are often digits like `0 1 2 3` that
bear no relation to the verb), diagnosable (a missing key logs a WARN and falls
back), and consistent with the one localisation mechanism already in place.
Option (i) is rejected: deriving the mnemonic FROM the localised surface couples
two independent columns, breaks for digit mnemonics, and would silently shift the
mnemonic whenever a translator edits the verb surface.

Keyed by menu + ROOT command (not by the raw `key`) so the key is stable across
locales and matches the description seam's keying. The fallback value is the
option's own `key` string (the Identity_Base English mnemonic), so an option is
always keyable even with no locale data (Req 12.3, 12.8).

REALISED CONTRACT (authoritative -- a coder MUST implement this and MUST NOT pick
the rejected option). The per-locale mnemonic SOURCE that Req 12.3 defers to
design, and the "no mnemonic source" case in Req 12.8, are BOTH realised as the
`menu-<menu>-opt-<command>-key` catalogue key resolved by
`localised_option_mnemonic(menu_name, option)` = `t_or(key, &option.key)` (Option
ii). "No mnemonic source at locale `L`" means precisely "that catalogue key is
absent at `L`", and the fallback is the option's stored `option.key`. Deriving the
mnemonic from the localised command surface (Option i) is REJECTED and MUST NOT be
implemented. (The menu-workspace CR-NR-111 delta repeats this one-line contract so
the two documents cannot diverge.)

`render_layout.rs::option_prefix_job` is changed to accept the resolved mnemonic
surface (positionally, exactly as it already accepts the resolved command surface)
rather than reading `option.key` raw; the shell computes the mnemonic surfaces
beside the command surfaces and passes both into the shared renderer. NOTE: the
mnemonic displayed for the user and the KEY used to MATCH a typed `MENU <name>
<key>` selection or an in-menu key press are separate concerns -- the match
continues to use the stored `option.key` (menu-workspace Req 1/19), so localising
the DISPLAYED mnemonic does not change which keystroke selects the option. If a
locale's displayed mnemonic should also be typeable, that is authored as an
additional key-match alias and is OUT of this gate's scope (noted under Out of
Scope in the requirements).

### First-prize auto-conversion, end to end (Req 12.6)

With Option A (stored root) plus the per-locale alias + description + mnemonic
DATA, a menu Kind authored/saved once renders fully in the active locale from the
SAME stored definition:

1. The saved option stores `{ key = "<root mnemonic>", command = "<ROOT>",
   description = "<English fallback>" }` (unchanged on-disk shape).
2. On a `ui.locale` change, `on_locale_reload` (`main.rs`) re-seeds the alias
   store (CR-NR-109) and sets the new i18n locale.
3. On the next frame the shared renderer receives, positionally per option:
   the localised mnemonic surface (`localised_option_mnemonic`), the localised
   command surface (`display_surface_for_canonical`), and the localised
   description (`localised_option_description`) -- all three for the active
   locale, each falling back per part to the Identity_Base.
4. Selection feeds the ROOT through the unified resolver; dispatch and
   persistence are unchanged.

No separate per-language menu file is required (Req 12.6). This applies to ALL
menu kinds because every menu renders through the one shared
`render_menu_workspace` (POM, Settings, user menus -- Req 12.9, menu-workspace
Req 2.1); no per-kind special-casing.

### Second-prize per-language saved menus (Req 12.7) -- FALLBACK

Supported as a fallback for a user who prefers to maintain a distinct saved menu
definition per locale rather than rely on auto-conversion. The mechanism reuses
the EXISTING menu-name resolution and the per-locale directory convention already
used for help content and alias DATA: for a menu named `<name>` at active locale
`L`, the menu loader resolves a locale-qualified definition if one exists before
falling back to the base definition.

- RESOLUTION ORDER (chosen to mirror the help loader's per-locale directory
  resolution, localization Req 6.2, so there is one directory-fallback pattern in
  the product): resolve `menus/<L>/<name>.toml` (a per-locale saved menu) when it
  exists; otherwise fall back to the base `menus/<name>.toml`; otherwise (for a
  built-in) the code-only `DEFAULT_*_TOML` Identity_Base. This adds a locale
  segment to the EXISTING user-menu file lookup (`MenuWorkspaceState.file_path`
  derivation); it does NOT introduce a new menu mechanism or a second dispatcher,
  and built-in menus remain code-only and are NEVER written to disk (CR-CH-021).

- RESOLUTION TIME AND PERSISTENCE (the load-vs-persist contract -- MANDATORY, Req
  12.7a). The locale segment is resolved at menu LOAD / RECONSTRUCT time, keyed by
  the menu NAME plus the active locale -- NOT baked into a persisted path. A
  Menu_Workspace persists in the session as a locale-NEUTRAL
  `WorkspaceDescriptor::Menu { name }` (the menu NAME, e.g. `pom`/`settings`/a user
  name), exactly as it does today (startup-and-session Req 21, CR-CH-012); the
  session NEVER stores a locale-qualified or absolute `file_path`.
- RESOLUTION SITES (the complete set -- enumerated, not "one site"). On restore /
  reconstruct, the user-authored menu load paths in `shell/nav_reconstruct.rs`
  apply the locale segment. There are TWO such reconstruct-by-name seams, and both
  MUST carry the segment identically:
  - `reconstruct_settings_menu` (the Settings menu), and
  - `reconstruct_named_menu` (every user-defined named menu),
  which today build `self.menus_dir().join("<name>.toml")` and become
  `menus_dir().join(L).join("<name>.toml")` when that exists, else
  `menus_dir().join("<name>.toml")`, else the code-only baseline -- re-reading the
  THEN-active locale `L` each time.
- THE POM / HOME CONTEXT IS OUT OF SCOPE FOR THE SECOND-PRIZE FILE OVERRIDE
  (deliberate, consistent with CR-CH-021). The POM does NOT reconstruct through
  either seam above: `reconstruct_pom` calls `ensure_pom_menu_loaded`
  (`shell/commands_fastpath.rs`), a distinct load path that builds
  `menus_dir().join("pom.toml")` directly. That path is INTENTIONALLY left
  unchanged -- the POM is localised by FIRST-PRIZE auto-conversion ONLY. A code-only
  built-in needs no per-locale FILE (its stored roots render localised via the
  per-locale alias/mnemonic/description DATA), so the optional second-prize FILE
  override (`menus/<L>/pom.toml`) is not wired for the POM and would have no
  effect; writing a base `menus/pom.toml` to let a locale file sit beside it is
  forbidden under CR-CH-021. This keeps full localisation applying to ALL menu
  kinds (Req 12.9) via first prize, while scoping the OPTIONAL second-prize FILE
  override to the two user-authored reconstruct seams. The
  `MenuWorkspaceState.file_path` that results at either reconstruct seam is a
  derived, transient field computed from name + active locale at load, not a
  persisted key.
- Consequence (the behaviour the gate requires): a session SAVED under locale `af`
  and RESTORED after the user has switched back to `en` re-resolves against `en`
  and shows the English/base menu, because the descriptor carried only the name;
  it is never pinned to the save-time locale.
- A per-locale saved menu is still subject to first-prize derivation for any part
  it does not itself translate: its options still store ROOT commands, so its
  command column still auto-converts; the per-locale file is the user's override
  for cases where they want a wholly different option SET or ordering per locale,
  not a substitute for the alias layer.

- SECOND PRIZE FOR A BUILT-IN MENU (POM / Settings) -- CR-CH-021 preserved (Req
  12.7b). A built-in has NO base `menus/<name>.toml` on disk normally: built-ins
  are code-only `DEFAULT_*_TOML` constants and `ensure_menus_dir` only creates the
  empty directory (it never writes a built-in file --
  `menu_workspace/defaults.rs`). The second-prize path for a built-in is therefore
  EXACTLY: resolve `menus/<L>/<name>.toml` IF the user authored one, otherwise fall
  STRAIGHT THROUGH to the code-only `DEFAULT_*_TOML` Identity_Base. No base
  `menus/<name>.toml` is ever written to let a locale file "sit beside" it -- doing
  so would violate CR-CH-021, and is explicitly forbidden here. In practice a
  built-in needs NO per-locale file at all: FIRST-PRIZE auto-conversion already
  localises the code-only built-in in full (its stored roots render localised via
  the per-locale alias/mnemonic/description DATA), so authoring a
  `menus/<L>/pom.toml` is a rare power-user override for a wholly different option
  SET/ordering per locale, not the normal way to localise a built-in.

First prize (a single auto-converting definition) is PREFERRED; second prize is
the explicit fallback the owner asked for. Both coexist because they compose: the
per-locale file selection happens at menu LOAD (by name + active locale), the
three-part surface derivation happens at RENDER, and dispatch is the same resolver
for both.

### Behaviour when no localised data exists (Req 12.8) -- per-part, per-option

The fallback is PER PART and PER OPTION, never whole-row or whole-menu:

- No alias row for the root at locale `L` -> command column shows the raw ROOT
  string (Identity_Base).
- No `menu-<menu>-opt-<command>-key` catalogue entry at `L` -> mnemonic shows the
  option's own `key` (Identity_Base English mnemonic).
- No `menu-<menu>-opt-<command>-desc` catalogue entry at `L` -> description shows
  the option's parsed English `description`.
- No per-locale saved menu for `L` -> the base definition is used.

A menu NEVER renders a blank, missing, or unselectable option because a locale
lacks data; each missing part independently falls back to its Identity_Base value
while the other parts may still be localised. This is the explicit fix for the
reported "the application is not consistently Afrikaans" inconsistency: an absent
`af` surface yields the English surface for THAT part only, never an empty or
broken row. The practical reason a localised POM still shows English today is
MISSING DATA (no `af` alias/mnemonic/description rows for the built-in roots), not
a code gap -- authoring that DATA is a separate gated deliverable (Out of Scope
here).

### Migration / back-compat for existing saved menus

Because the DECISION keeps the EXISTING stored shape (Option A = ROOT command),
there is NO migration of existing saved menu files:

- Every menu already on disk stores the root English command in `MenuOption.command`
  and an English mnemonic/description. Under CR-NR-111 these files load unchanged
  and render identically under `ui.locale = "en"` (behaviour-neutral, Req 10.2).
- When the active locale has alias/mnemonic/description DATA for those roots, the
  SAME unmodified file auto-converts -- which is precisely first prize. An old
  file gains localisation for free as the per-locale DATA is authored; it is never
  rewritten.
- The ONLY new on-disk artefacts are the per-locale DATA
  (`i18n/<locale>/aliases/<env>.toml`, the `menu-*-key` / `menu-*-desc` `.ftl`
  entries) and, for the optional second-prize path, a `menus/<L>/<name>.toml` a
  user chooses to create. None of these touch an existing `menus/<name>.toml`.
- A hypothetical menu authored under a mistaken "store the localised surface"
  assumption would still DISPATCH correctly (the resolver maps any surface back to
  the root) but would be locale-pinned in its display; such a file is not produced
  by this product and no migration path is specified for it beyond the standing
  advice to store roots.

### Framework-conformance statement (CR-NR-111)

This delta changes NO framework mechanism. It builds ON the single command
dispatch (the unchanged `rewrite_verb_to_root` -> `resolve_target` pre-pass), the
code-only built-in menus (CR-CH-021, never written to disk), menus as CONTENT via
the shared `render_menu_workspace`, and the existing i18n catalogue + per-locale
alias DATA seams. It adds one new catalogue key family (the mnemonic key) that
mirrors the existing description key, and one locale segment in the user-menu file
lookup for the optional second-prize path. The ROOT command remains the stored,
dispatched, and persisted identity (Req 10.3, 11.9). With `ui.locale = "en"`
every menu renders exactly as today (Req 10.2).

### Testability (CR-NR-111)

- `i18n_keys` unit tests: `localised_option_mnemonic` returns the catalogue value
  when the `menu-<menu>-opt-<command>-key` key is present and the option's `key`
  fallback when absent (mirrors the existing `localised_option_description` test).
- Shell unit test: the mnemonic-surface vector computed beside
  `menu_option_command_surfaces` is positionally aligned with `menu.options` and
  falls back per option.
- Full-shell `egui_kittest`: with a non-`en` locale and mnemonic/alias/description
  DATA loaded, a menu option row shows all three localised; selecting it
  dispatches and records the ROOT command. With `en` / no overlay, the row shows
  the raw root mnemonic/command/English description unchanged (Req 10.2).
- Menu-loader unit test (second prize): `menus/<L>/<name>.toml` is loaded when
  present and the base `menus/<name>.toml` otherwise; a built-in with no file
  falls back to the code-only `DEFAULT_*_TOML` (no base file written -- CR-CH-021).
- Second-prize persistence test (Req 12.7a): a session persisted under locale `af`
  carries the locale-NEUTRAL `WorkspaceDescriptor::Menu { name }`; reconstructing
  it under `en` re-resolves by name + active locale and opens the English/base
  menu, proving the restored menu follows the then-active locale and is NOT pinned
  to the save-time locale.
- Per-part fallback test: an option with an alias row but no mnemonic key shows
  the localised command and the Identity_Base mnemonic, proving fallback is
  per-part, not whole-row (Req 12.8).

## Review responses -- CR-NR-111 design review (iteration 2)

This section records how the design-review findings
(`.agents/tasks/menu-kind-localisation/design-review.json`, verdict
CHANGES_REQUESTED: 3 MEDIUM, 2 NIT) were resolved. All findings are ADDRESSED.

- **Finding 1 (MEDIUM -- second-prize locale segment vs session persistence):
  ADDRESSED.** The second-prize sections of this file and of
  `menu-workspace/design.md` now state explicitly that the locale segment is
  resolved at menu LOAD / RECONSTRUCT time keyed by menu NAME + active locale, that
  the session persists the locale-NEUTRAL `WorkspaceDescriptor::Menu { name }` (not
  a locale-qualified or absolute `file_path`), and that the reconstruct-by-name
  seam (`shell/nav_reconstruct.rs`: `reconstruct_named_menu` /
  `reconstruct_settings_menu` + `menus_dir()`) is the single site that applies the
  locale segment. A session saved under `af` and restored under `en` re-resolves to
  the English/base menu. A persistence test was added to Testability, and
  requirements Req 12.7a (localization) / Req 12.7 (menu-workspace) back it.
- **Finding 2 (MEDIUM -- second-prize path for a BUILT-IN risks CR-CH-021):
  ADDRESSED.** Both second-prize sections now state that a built-in resolves
  `menus/<L>/<name>.toml` only if the user authored one, otherwise falls STRAIGHT
  THROUGH to the code-only `DEFAULT_*_TOML` baseline, that no base file is ever
  written to let a locale file sit beside it (forbidden under CR-CH-021), and that
  a built-in needs NO per-locale file because first-prize auto-conversion localises
  the code-only built-in from its roots + per-locale DATA. Recorded as Req 12.7b.
- **Finding 3 (MEDIUM -- Req 12.8 wording does not pin the catalogue-key design):
  ADDRESSED.** A single authoritative REALISED CONTRACT statement was added to the
  mnemonic decision here and mirrored in the menu-workspace delta: the per-locale
  mnemonic source IS the `menu-<menu>-opt-<command>-key` catalogue key (Option ii)
  resolved by `localised_option_mnemonic` = `t_or(key, &option.key)`; "no mnemonic
  source" means that key is absent; deriving the mnemonic from the command surface
  (Option i) is REJECTED and MUST NOT be implemented.
- **Finding 4 (NIT -- mnemonic DISPLAY vs key MATCH asymmetry as a criterion):
  ADDRESSED.** A new acceptance criterion menu-workspace Req 21.8 states the
  displayed mnemonic is display-only and the stored `key` remains the selection
  token; the menu-workspace design delta now cross-references it and the full-shell
  test asserts it. The localised-mnemonic-is-typeable case stays out of scope with
  a forward reference to localization Req 12 Out of Scope.
- **Finding 5 (NIT -- missing `design-files.md` handoff):
  ADDRESSED.** `.agents/tasks/menu-kind-localisation/design-files.md` now lists
  every design.md and requirements.md touched, so a future reviewer need not
  recover the set from git.

## Review responses -- CR-NR-111 design review (iteration 3)

This section records how the later design-review findings
(`.agents/tasks/menu-kind-localisation/design-review.json`, verdict
CHANGES_REQUESTED: 1 MEDIUM, 2 NIT) were resolved. All findings are ADDRESSED.

- **Finding 1 (MEDIUM -- second-prize locale-segment site omits the POM load
  path): ADDRESSED by option (b).** Verified against source: the POM reconstructs
  via `reconstruct_pom` -> `ensure_pom_menu_loaded` (`shell/commands_fastpath.rs`),
  which builds `menus_dir().join("pom.toml")` directly -- NOT through
  `reconstruct_named_menu` / `reconstruct_settings_menu`. The second-prize sections
  of this file and of `menu-workspace/design.md` now state EXPLICITLY that the POM
  is OUT of scope for the optional second-prize FILE override and is localised by
  first-prize auto-conversion only (consistent with CR-CH-021: a code-only built-in
  needs no per-locale file), so `ensure_pom_menu_loaded` is intentionally left
  unchanged and a `menus/<L>/pom.toml` is not wired. This removes the apparent
  contradiction with Req 12.9's "all menu kinds": full localisation still applies
  to every menu kind via first prize, while the OPTIONAL second-prize FILE override
  is scoped to the two user-authored reconstruct seams.
- **Finding 2 (NIT -- "the ONE site" wording overstated): ADDRESSED.** Both
  second-prize sections now use a "RESOLUTION SITES (the complete set --
  enumerated, not 'one site')" heading that names the TWO user-menu reconstruct
  seams (`reconstruct_settings_menu`, `reconstruct_named_menu`) as the sites that
  carry the locale segment, and explicitly lists `ensure_pom_menu_loaded` (the
  POM's third load path) as out of scope. No "one site" claim remains.
- **Finding 3 (NIT -- mnemonic catalogue-key slug convention not restated in the
  implementable delta): ADDRESSED.** The menu-workspace CR-NR-111 delta now states
  that `menu_option_key_key` uses the SAME `slug(command)` helper as
  `menu_option_desc_key` (verified: `menu_option_desc_key` calls `slug(command)` in
  `menu_workspace/i18n_keys.rs`), giving `menu-settings-opt-reset-bare-key` to
  parallel `menu-settings-opt-reset-bare-desc`, and forbids building the key from
  the raw command string.

## Relationship to structured-messages (ff-message)

See `docs/specs/structured-messages`. The new `ff-message` crate DEPENDS ON
`ff-i18n` and is the concrete realization of localization Req 9.3 (route
error/structured text through the lookup seam at the display boundary). It adds a
stable `{class, number}` message IDENTITY and a severity axis -- the two things
`ff-i18n` deliberately omits -- and renders via `ff_i18n::t_args` at the edge. It
does NOT change `ff-i18n`.

PROPOSED (future) cross-spec change: structured-message text lives as ordinary
`ff-i18n` `.ftl` DATA keyed `vfs-0007`-style (and optional per-locale overlays).
Growing that catalogue data beyond the English seed is future work under this
spec's incremental extraction; it is marked PROPOSED here, not committed in this
gate.

---

## CR-CH-072 -- Single-layer surface-in-row alias + ONE locale-aware store

### Overview

CR-CH-072 is a SIMPLIFICATION that unwinds complexity the owner rejected. The
delivered command-alias localisation used THREE layers of interpretation (alias
row -> `localisation_key` -> `verb-*` Fluent entry) and TWO stores (the shipped
per-locale `i18n/<locale>/aliases/*.toml` catalogue, which actually localises the
menus but is invisible/uneditable in-app; and the CR-NR-113 user store, which the
Alias Editor edits but which does NOT localise the menus). This design collapses
command-surface localisation to ONE layer (the surface lives in the alias row) and
the alias data to ONE locale-aware store per environment (shipped read-only seed
overlaid by a user per-locale override), and makes the Alias Editor edit THAT
effective table. DESCRIPTION and TITLE localisation (ordinary `.ftl` via the
Catalogue_Lookup_Seam) is UNCHANGED; routing/dispatch/persistence is UNCHANGED (the
ROOT stays the dispatched/persisted identity).

### The unified AliasEntry shape

The in-memory alias entry (`shell/environment.rs` `AliasEntry`) and the on-disk row
(`shell/alias_reader.rs` `AliasRowToml`) collapse to:

```
AliasEntry / AliasRowToml:
  surface          : String            // the In_Row_Surface (localised, authoritative display/typeable text)
  canonical (root) : String            // the ROOT command (dispatched + persisted identity)
  command_template : Option<String>    // CR-NR-112 parameter template, RETAINED (e.g. "FFWB.COBOL.MASTER(&1)")
  environment      : AliasEnvironment  // ffcmd | ffedit (per-row target CE; default ffcmd)
```

The `localisation_key` field is REMOVED. There is NO `verb-*` lookup for command
surfaces. The surface is matched under the existing diacritic-insensitive fold
(localization Req 11.11, CR-CH-065) so a plain-keyboard user can type the folded
form. Identity rows (canonical -> itself) are still auto-synthesised (Req 11.3) so
the root is always resolvable; an auto-identity row is overridable by an explicit
row (Req 11.4).

Decision -- surface vs localisation_key as authoritative: the SURFACE is chosen as
authoritative (owner point 2: "surface in the alias row"). Rationale: it removes an
indirection and a duplicate store of the same text (today `surface = "INSTELLINGS"`
AND `verb-settings = INSTELLINGS` both exist); one source of truth cannot drift.

### The single resolution algorithm and where it lives

Resolution is the EXISTING `rewrite_verb_to_root` pre-pass (`shell/locale_resolver.rs`)
called at the two seams it already serves -- the typed front door
(`shell/dispatch.rs`) and menu selection (`shell/commands_menu.rs`) -- BEFORE
`ff_command::resolve_target`. It is re-pointed at the In_Row_Surface; NO new
dispatcher is added. The two-level contract (command-environments Req 2b):

```
resolve(V, locale L):
  # Level 1 -- the ACTIVE Command Environment
  if V matches an In_Row_Surface in ACTIVE.alias_table[L]   -> root = that row's canonical; execute(root); done
  elif V is a root of ACTIVE                                 -> execute(V); done
  else  # ACTIVE rejects, pass back
  # Level 2 -- the FFCMD base
  if V matches an In_Row_Surface in FFCMD.alias_table[L]     -> root = that row's canonical; execute(root); done
  elif V is a root of FFCMD                                  -> execute(V); done
  else                                                       -> unresolved-command result
  # each surface comparison uses the diacritic-insensitive fold (Req 11.11);
  # a matched alias that carries a command_template expands it (CR-NR-112) before resolve_target
```

Exactly two levels, one path, identical for typed and menu-selected commands.

### How a menu option references an alias (owner reservation #3)

Today a menu option stores the ROOT in its `command` field and the localised
surface is reverse-derived at render time (a bespoke special case). The redesign
makes the option reference an alias so render AND dispatch use the one path.

Decision -- OPEN (recommended, for owner CONFIRM): store the ROOT in the option's
`command` field AND mark the option "resolve-as-alias", rather than storing an
alias id/surface-key. At render the Menu_Workspace looks up the active-locale
In_Row_Surface whose canonical equals the option's root (per-part fallback to the
root/English surface when absent, Req 12.8) and DISPLAYS it; on selection the SAME
referenced verb is fed through the Req 2b resolver to the root. Rationale: the
persisted identity stays the ROOT (Req 10.3 / 11.9 unchanged, no migration of
saved menus), the render path stops reverse-deriving a surface from a translated
string and instead reads the row's authoritative surface, and no new option field
or id scheme is introduced. The alternative (store an alias id/surface-key on the
option) was rejected: it would make the persisted value locale-dependent and would
need a migration of every saved menu.

### The per-locale store model (precedence, merge)

For an active locale L and environment ENV the EFFECTIVE table is:

```
shipped seed  i18n/<locale>/aliases/<ENV>.toml     (read-only, ships af out of the box)
   overlaid by
user override <User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml  (editable)
```

User override WINS where a surface is defined in both (it is the user's explicit
customisation, localization Req 13.5). The Identity_Base + per-locale overlay +
user-third-overlay THREE-layer stack (CR-NR-113 Req 5.6/5.7) collapses to this ONE
overlay chain per environment-per-locale. The existing `apply_locale_overlay` seam
is reused for the seed->override merge (no new overlay mechanism); the reload seam
(`seed_and_overlay_for_locale`) re-applies seed + override in one pass so a save
goes live without restart.

Decision -- OPEN (recommended, for owner CONFIRM): KEEP the shipped catalogues in
`i18n/<locale>/aliases/` as a READ-ONLY seed with the user override layered on top,
rather than copying them into the user store on first run. Rationale: preserves
out-of-box `af` and gives clean upgrades (a new shipped surface appears
automatically unless the user has overridden it).

### The Alias Editor's active-locale effective view

The Alias Editor (command-configurator Req 6) shows the EFFECTIVE table for the
active locale -- seed overlaid by override -- so an `af` user SEES the shipped `af`
surfaces (the fix for "I don't see any Afrikaans aliases"). Edits write to the user
override for the active locale/ENV only; the seed is never written. The CR-CH-071
ISPF semantics operate over the override: SAVE persists the override and stays;
CANCEL/RETURN discards; REFRESH reloads the effective table. The CR-CH-068 column
layout drops the L18N (localisation-key) column (there is no command-surface key
any more); the remaining columns are Act / CE / Root / ALIAS(surface) / Parameters.

### Migration of the current af data

The shipped `af/aliases/FFCMD.toml` rows currently carry
`localisation_key = "verb-settings"` etc., and `af/ff-desktop-verbs.ftl` carries
`verb-settings = INSTELLINGS` etc. The migration:

1. DROP the `localisation_key` line from each `[[alias]]` row; the existing
   `surface = "INSTELLINGS"` already holds the localised text, so the row becomes
   `surface` + `canonical` (+ optional `command_template`). No surface text is lost
   (the surface was already duplicated into the row).
2. The `verb-*` keys in `af/ff-desktop-verbs.ftl` and `en/ff-desktop-verbs.ftl`
   become UNUSED for command surfaces and are retired (DESCRIPTION/TITLE keys in
   `menus.ftl` are UNCHANGED and keep localising descriptions/titles).
3. The menus must still render `af` -- verified by the test matrix: because the
   surface is unchanged and is now read directly, POM + Settings render the same
   `af` surfaces with ONE lookup instead of three.

This design BUILDS ON the current uncommitted `af` data (the surfaces are reused);
it does not discard it. See the gate-summary sequencing note: that uncommitted data
should be committed/carried before the rework so it is not lost.

### What is NOT changing

FFCMD/FFEDIT root sets (CR-CH-069/070 placement + `=FILES` fix), command routing,
the single front door, active-wins, code-only built-in menus (`DEFAULT_*_TOML`),
the DESCRIPTION/TITLE `.ftl` localisation, the WorkspaceContext focus contract, and
the ISPF Save/End/Cancel/Return/Refresh BEHAVIOUR (CR-CH-071). The CR-NR-112
parameter-template expansion is retained byte-for-byte. The mnemonic stays English
(it does not localise; CR-NR-111 Req 12.3 source unchanged).

### Migration / supersession table

| Prior criterion | Disposition under CR-CH-072 | How observable behaviour is preserved |
|-----------------|------------------------------|----------------------------------------|
| CR-NR-109 loc Req 11.1/11.2 (`localisation_key` + `t(key)` surface) | SUPERSEDED for command surfaces | surface stored in row, resolved by one lookup; same displayed text |
| CR-NR-109 loc Req 11.3/11.4 (identity rows) | RETAINED (over in-row surface) | root always resolvable; identity overridable |
| CR-NR-109 loc Req 11.6 (resolver order) | RETAINED (= Req 2b two-level) | same active-env-then-FFCMD order |
| CR-NR-109 loc Req 11.11-11.13 (diacritic fold) | RETAINED (applies to in-row surface) | plain-keyboard input still resolves |
| CR-NR-111 loc Req 12.2/12.4 (menu stores root, reverse-derive surface) | AMENDED (option references alias; one path) | menus render full af; per-part fallback kept |
| CR-NR-111 loc Req 12.3 (mnemonic source) | UNCHANGED | mnemonics stay English |
| CR-NR-112 semantics Req 14.1 (4-field record) | AMENDED (drop localisation_key; 3 fields) | template feature unchanged |
| CR-NR-112 template expansion (14.2-14.4) | RETAINED byte-for-byte | ecob &1 still expands |
| CR-NR-113 configurator Req 5.11 (user-store-only) | SUPERSEDED (editor edits per-locale aliases) | editor now shows/edits af surfaces |
| CR-NR-113 configurator Req 5.6/5.7 (3-overlay store) | SUPERSEDED (one seed + user override) | save goes live; af seeded out of box |
| CR-CH-071 Req 20a (ISPF save semantics) | PRESERVED (over the override store) | Save/End/Cancel/Return/Refresh unchanged |
