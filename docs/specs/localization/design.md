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
