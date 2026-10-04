# FileForgeWorkbench i18n-Posture Audit (CR-NR-103, Task 1)

Read-only investigation. No source or spec files were modified. This report is
the evidence base for the CR-NR-103 requirements gate. All counts state the exact
search used so they are reproducible; all claims cite `path:line`.

Scope searched: `c:\workspace\VSC\FileForgeWorkbench\crates` (91 crate
directories; `Get-ChildItem -Directory .../crates | Measure-Object` = 91). Intent
sources read first: `docs/specs/localization/localization-vision.md` and
`docs/specs/command-environments/requirements.md` Requirement 6a.

---

## Executive summary -- extraction size: MEDIUM (leaning small-to-medium)

The workspace is in a FAVOURABLE i18n posture and the vision document's "honest
assessment" is confirmed by code:

- There is **NO existing i18n/l10n mechanism anywhere** -- no catalogue crate, no
  `t()`/`tr!` seam, no locale key, no `fluent`/`gettext`/`rust-i18n`/`icu`/
  `unic-langid` dependency in any `Cargo.toml`. The field is genuinely greenfield.
- Hardcoded user-facing text is **real but not sprawling** and is **heavily
  concentrated**: roughly **180-260 user-facing UI string literals** are the core
  extraction target, about **~62% of them in a single crate (`ff-desktop`)**. The
  other heavy crates are a short, known list (catalog/dataset dialogs, files
  panel, explorer view, theme editor). This is a bounded, per-crate-orderable job,
  not a workspace-wide needle hunt.
- The **substrate is ready**: Rust is UTF-8 throughout, help content is already
  externalised to `.help.md` files loaded from disk, and the configuration system
  (`ff-config`) already has a schema registry + hot-reload + change callbacks that
  a `ui.locale` key drops straight into.
- The forward-build directive has **already produced a seam** for the hardest
  dimension (command-verb aliases): the per-environment `AliasTable` in
  `ff-desktop/src/shell/environment.rs` is explicitly built to receive per-locale
  alias data with "no new mechanism" (CR-NR-103 referenced in-code).

Why MEDIUM not SMALL: the ~1200 `format!("...")` sites and ~560 `Some("...")`
sites include an unknown-but-material fraction of user-facing status/error text
that is built by interpolation deep in logic (e.g. verb-prefixed error strings in
the command ladders). These are the "harder" extraction cases and dominate the
risk; the pure render-call literals (the clean cases) are small.

---

## Q1. Does any existing localization/i18n provision exist? -- NO.

**Dependency search (authoritative):**
`rg "fluent|gettext|rust-i18n|unic-langid|\bicu\b|i18n|l10n"` over `**/Cargo.toml`
-> **No matches.** No crate declares any i18n/catalogue library.

**Source mechanism search:**
`rg "\blocale\b|\btr!\b|translation|catalogue|catalog_lookup|i18n|l10n|gettext|fluent"`
over `**/*.rs` -> the only hits are **unrelated**: `IndicatorCatalogue` (text
decorations), VFS `catalogue`/reconcile vocabulary, a "scroll-amount translation"
doc comment, and a `Fluent builder` doc comment in `ff-workflow` (builder-pattern,
not fluent-rs). None is a message-catalogue or locale mechanism.

**The one genuinely i18n-aware thing that DOES exist** is a forward-build seam,
not a mechanism: `crates/ff-desktop/src/shell/environment.rs:110` comments that
the per-environment alias table "loads per-locale alias data into the SAME table
with no new mechanism," and `:127` notes "CR-NR-103 extends this per locale." The
table (`AliasTable`, `environment.rs:122`) today holds only the English identity
+ alias seed (`ffedit_english`, `environment.rs:130`).

**Conclusion:** localization is unstarted. Nothing must be ripped out; the gate
builds net-new on a clean substrate.

---

## Q2. How pervasive are hardcoded user-facing string literals? (sized)

### 2a. egui render-call literals (the clean, high-confidence target)

Search (non-test):
`rg --pcre2 '\.(label|button|heading|selectable_label|hyperlink|monospace|small|strong|weak)\(\s*"' -g '*.rs' -g '!*/tests/*' -g '!*test*' crates`
-> **184 matches.** Per-crate (top crates):

| count | crate |
|------:|-------|
| 114 | `ff-desktop` |
| 21 | `ff-catalog-dialog` |
| 10 | `ff-files-panel` |
| 10 | `ff-dataset-alloc-dialog` |
| 9 | `ff-explorer-view` |
| 8 | `ff-theme-editor` |
| 6 | `ff-mdx-installer` |
| 4 | `ff-mdx-app` |
| 2 | `ff-toolchain-panel` |

This pattern UNDERCOUNTS: it misses `egui::Button::new("...")`, `ui.checkbox(.., "..")`,
and `.on_hover_text("..")`, which the spot-reads below show are also common.
Representative additional sources (not in the 184):
- `egui::Button::new("Cancel")` / `Button::new("Confirm reset")` --
  `ff-desktop/src/shell/update_dialogs.rs:250-253`
- `egui::Button::new("Save")` -- `ff-desktop/src/keys_editor_panel/render.rs:141`,
  `ff-desktop/src/kinds_editor_panel/render.rs:194`
- `.on_hover_text("Search history")` / `"Case sensitive"` / `"Whole word"` --
  `ff-desktop/src/search_results_panel/render.rs:66,111,113`
- form labels: `ui.label("Catalog Type:")`, `"Catalog Name:  "`, `"Description:"`
  -- `ff-catalog-dialog/src/new_dialog.rs:238,255,263`
- `ui.label("Command ===>")` -- `ff-files-panel/src/tree.rs:58`

`RichText::new("...")` literals (non-test):
`rg --pcre2 'RichText::new\(\s*"'` -> **39 matches** (e.g.
`ff-catalog-dialog/src/edit_dialog.rs:88` monospace display of a field label).

**Realistic clean-literal UI target: ~230-280** once Button/checkbox/hover/RichText
are added to the 184. Still heavily `ff-desktop`-weighted.

### 2b. Status / error message strings (medium confidence, some harder)

`open_error = Some("...")` literal assignments (ladder/shell error surface):
`rg 'open_error = Some\("' crates/**/*.rs` -> ~**20** direct literal assignments
spread across `ff-desktop/src/shell/commands_ladder_a.rs`, `_c.rs`,
`commands_fastpath.rs`, `commands_scrm.rs`, `dispatch_ffedit.rs`, `actions.rs`,
`workspace_io.rs`, `update_dialogs.rs`. Representative:
- `"EDIT requires a file path"` -- `commands_ladder_a.rs:66`
- `"UNSPLIT: the Workspace is not split."` -- `commands_ladder_c.rs:71`
- `"SUBMIT: JES subsystem not yet available"` -- `commands_ladder_c.rs:149`
- `"Profile is locked -- use LOCK OFF to unlock"` -- `dispatch_ffedit.rs:202`

Broader (over-broad) bounds, non-test:
- `rg --pcre2 'Some\(\s*"'` -> **564** `Some("...")` occurrences workspace-wide.
  This is an UPPER bound and includes much non-UI (identifiers, test fixtures,
  internal keys); the user-facing subset is a minority but non-trivial.

### 2c. `format!` interpolated text (the HARD dimension)

`rg --pcre2 'format!\(\s*"'` (non-test) -> **1221** occurrences workspace-wide.
Many are log lines, path building, and internal diagnostics (NOT to be
localized), but a material fraction are user-facing status/error strings built by
interpolation (e.g. the verb-prefixed error strings above). This is the dominant
sizing UNCERTAINTY: the exact user-facing subset was not hand-classified in this
read-only pass and should be triaged crate-by-crate during extraction. It is the
reason the overall size is MEDIUM rather than SMALL.

### 2d. The `DEFAULT_*_TOML` menu titles/descriptions (bounded, data-shaped)

`DEFAULT_POM_TOML` / `DEFAULT_SETTINGS_TOML` --
`ff-desktop/src/menu_workspace/defaults.rs:29,71`. These are Rust `const &str`
blocks of TOML that embed user-facing `title = "FileForge Workbench -- Primary
Option Menu"` and per-option `description = "..."` strings
(`defaults.rs:29` and the `[[options]]` blocks). Small count (two menus, ~5-6
options each) but notable because the text is both compiled-in AND already in
data form -- the easiest category to redirect through a catalogue.

### What must NOT be extracted (verbs / stable identifiers)

By design (localization-vision "DO NOT LOCALISE"; command-environments Req 6a.2):
command VERBS and canonical identifiers stay English. Concretely, do NOT extract
the canonical verb strings in `AliasTable::ffedit_english` (`environment.rs:138-158`:
`"FIND"`, `"EXCLUDE"`, `"SAVE"`, ...), menu-option `command` values (e.g.
`command = "Settings"` in `defaults.rs`), config KEY names (`keys.rs`), `TopicKey`
/ topic identifiers, and `command_id` strings. Only the DESCRIPTIONS, titles,
labels, and messages beside them are translatable.

---

## Q3. What is already externalised as DATA (half-way to translatable)?

- **Help content -- fully externalised, already locale-ready.** `ff-help` loads
  `.help.md` files from a resolved content directory at runtime
  (`ff-help/src/content_loader.rs:1,24` discover+`std::fs::read_to_string`,
  `:131`), parsing topic delimiters (`content_parser.rs`). Content ships under the
  repo `help/` tree (e.g. `help/commands/find.help.md`, `help/index.help.md`;
  ~11 `.help.md` files). Because the loader resolves a directory, a per-locale
  help directory is the natural translation unit -- no code change to the loader
  shape, just locale-aware path resolution. `TopicSource::FileBased`
  (`topic.rs:16`) already models disk-sourced topics.
- **Menus -- data-shaped but CODE-ONLY by policy (CR-CH-021).** Built-in menus are
  the `DEFAULT_*_TOML` constants above; a `file_search` for `menus/` on disk
  returned **no shipped menu files** (user menus override at runtime but none are
  committed). So menu text is TOML-structured (translatable in shape) yet compiled
  in -- the catalogue seam must read the title/description out of the parsed menu
  model, not expect a translated TOML file.
- **Themes / config** are externalised as data too, but carry little user-facing
  PROSE (theme = colours; config = keys/values), so they are low-value extraction
  targets except for config schema `description` fields (see Q4).

Net: help is the big already-translatable asset; menu text is structurally
data-shaped but compiled-in; themes/config carry minimal prose.

---

## Q4. Where does a `ui.locale` config key slot in?

**Crate: `ff-config`.** It is a full configuration system with a schema registry,
six-layer merge, hot-reload, and change callbacks -- a `ui.locale` key is a
one-entry addition that inherits hot-reload for free.

How keys are defined today (two coordinated spots):
1. **Key constant** in `crates/ff-config/src/keys.rs` -- namespaced modules of
   `pub const NAME: &str = "namespace.key"` (e.g. `theme::ACTIVE = "theme.active"`,
   `menu::SOFT_OPTION_LIMIT = "menu.soft_option_limit"`). There is currently **no
   `ui` namespace**; `ui.locale` would introduce `pub mod ui { pub const LOCALE:
   &str = "ui.locale"; }`. The in-crate test `all_keys_have_unique_values`
   (`keys.rs`) and `assert_valid_key` enforce the dot-path convention a new key
   must satisfy (lowercase, single dot).
2. **Schema entry** in `crates/ff-config/src/init/schema.rs::register_core_schema`
   -- each key is a `SchemaEntry { key, value_type, default, description,
   constraints }`. A `ui.locale` entry mirrors `theme::ACTIVE`/`logging::LEVEL`:
   `ValueType::String`, `default = ConfigValue::String("en")` (or a system/"auto"
   sentinel), and an optional `Constraints { allowed_values: Some([...]) }` to pin
   the shipped locales (same shape as the `logging.level` enum constraint at
   `schema.rs`).

Hot-reload path (already built): `ReloadManager` (`src/reload/`) +
`CallbackRegistry::on_reload(&["ui.locale"], ...)` (`callback.rs`,
exercised in `tests/project_tests.rs` and `tests/integration_tests.rs`) delivers a
changed-keys event. So a locale CHANGE can swap the active catalogue live, exactly
as `theme.active` / `editor.tab_size` changes already propagate. The gate should
specify the key as **`ui.locale`**, String, default `"en"` (English identity base
always present per Req 6a.7), allowed-values constrained to shipped locales, with
a reload callback that (a) selects the per-locale message catalogue and (b)
reloads the per-environment alias catalogue (Req 6a.6).

---

## Q5. Patterns that make extraction HARDER vs EASIER

**Harder:**
- **Interpolated status/error text built in logic.** ~1221 `format!("...")` sites;
  the user-facing subset (e.g. `"UNSPLIT: the Workspace is not split."`,
  verb-prefixed ladder errors in `commands_ladder_*.rs`) mixes a translatable
  template with runtime values and a stable verb token. These need
  argument-bearing catalogue entries (Fluent-style `{ $arg }`), not plain
  key->string. This is the main cost driver and the sizing uncertainty.
- **Text that embeds a command verb or identifier** (e.g. `"Profile is locked --
  use LOCK OFF to unlock"`, `dispatch_ffedit.rs:202`): the message localizes but
  the embedded verb (`LOCK OFF`) must stay English -- the catalogue entry must
  treat the verb as a non-translated argument, not inline prose, to honour the
  verb/text split (Req 6a.2).
- **Error strings produced via `e.to_string()`** (`dispatch_ffedit.rs:205`,
  `commands_ladder_c.rs`): these originate in `thiserror` enums across many
  crates; localizing them means routing through the catalogue at the display
  boundary rather than at the error source.
- **Spread across many small files in `ff-desktop`** (the `shell/commands_ladder_*`,
  `*_editor_panel/render.rs`, dialog modules): no single chokepoint for messages;
  each render/handler site is its own edit.

**Easier:**
- **Centralised render literals** in the dialog/panel crates
  (`ff-catalog-dialog`, `ff-dataset-alloc-dialog`, `ff-files-panel`,
  `ff-explorer-view`, `ff-theme-editor`): literals sit directly on `ui.label(..)`
  / `ui.button(..)` calls, easy to wrap in a `t("key")` lookup one file at a time.
- **The alias seam already exists** (`environment.rs` `AliasTable`): the hardest
  conceptual piece (per-environment, per-locale verb aliases) has a built,
  unit-tested home that explicitly expects CR-NR-103 data -- no new mechanism.
- **Help is already data on disk** (`.help.md`): translate by adding a per-locale
  directory; the loader shape already resolves a content dir.
- **Menu text is structured TOML** (`DEFAULT_*_TOML`): title/description are
  already discrete fields in a parsed model, so a catalogue lookup keyed by
  menu+option is clean.
- **Config system is ready** for the `ui.locale` key with hot-reload callbacks
  (Q4), so the locale-switch plumbing is largely pre-built.

---

## Recommendations for the CR-NR-103 gate

### Catalogue mechanism: Fluent vs gettext-style

Given the findings, **Fluent (`fluent-rs` / `fluent-bundle` + `unic-langid`) is the
better fit**, for concrete reasons rooted in this codebase:
- The hard cases here are **argument-bearing, verb-embedding messages** (Q5).
  Fluent's `{ $var }` placeables and its ability to carry a non-translated verb as
  an argument match the verb/text split (Req 6a.2) directly; gettext's
  printf-style `%s` is workable but weaker for the "keep this token English"
  requirement and for plural/gender.
- Fluent is **ICU-grade for plurals/gender** (the vision explicitly flags this),
  which future languages (not English/French) will need.
- Fluent catalogues are **per-locale `.ftl` data files**, matching the vision's
  "adding a language is adding a catalogue, not editing Rust" and composing with
  the per-locale ALIAS catalogue model (Req 6a.6) cleanly: one locale -> one
  bundle of message tables + one bundle of per-environment alias tables.

Tradeoff to record: Fluent adds `fluent`/`intl-memoizer`/`unic-langid` deps and a
slightly heavier runtime than a flat gettext map. Given the workspace already
pulls large deps (egui, etc.) and the argument/plural needs are real, the cost is
justified. If the gate wants the smallest possible footprint and accepts weaker
plural handling, a gettext-style `.po`/flat-map is the fallback -- but it will fight
the verb-embedding cases.

### Suggested incremental extraction ordering (by crate / category)

Order maximises early value and matches the forward-build "extract incrementally"
directive:
1. **Config + locale plumbing first**: add `ui.locale` key + schema + reload
   callback in `ff-config`; stand up the catalogue seam (`t("key")`) and load the
   English base catalogue. No visible change yet, but every later step depends on
   this.
2. **`DEFAULT_*_TOML` menu titles/descriptions** (`ff-desktop/menu_workspace/
   defaults.rs`): tiny, data-shaped, high visibility (POM/Settings), proves the
   menu-model catalogue lookup.
3. **Help content**: add per-locale `.help.md` directory resolution in
   `ff-help/content_loader.rs` -- big translatable surface, low code churn.
4. **The dialog/panel crates** (clean render literals), one per step:
   `ff-catalog-dialog`, `ff-dataset-alloc-dialog`, `ff-files-panel`,
   `ff-explorer-view`, `ff-theme-editor`, `ff-toolchain-panel`.
5. **`ff-desktop` render literals** (the 114): editor-panel and shell render
   labels/buttons/hover text.
6. **Status/error messages last** (`open_error`, ladder `format!` strings): the
   hard, argument-bearing cases, triaged crate-by-crate; classify each `format!`
   site as user-facing-translatable vs internal-diagnostic during this pass.
7. **Verb alias catalogues** (per-environment-per-locale) land as DATA into the
   existing `AliasTable` once the catalogue loader exists -- no new mechanism
   (Req 6a.5/6a.6).

### The `ui.locale` config key shape (concrete, for the gate to pin)

- Constant: new `pub mod ui { pub const LOCALE: &str = "ui.locale"; }` in
  `ff-config/src/keys.rs` (satisfies the `assert_valid_key` dot-path tests).
- Schema entry in `register_core_schema` (`ff-config/src/init/schema.rs`):
  `SchemaEntry { key: ui::LOCALE, value_type: String, default:
  ConfigValue::String("en"), description: "Active UI language locale (BCP-47,
  e.g. en, fr, de)", constraints: Some(Constraints { allowed_values:
  Some([shipped locales]) }) }`.
- Default `"en"` keeps English as the always-present identity base (Req 6a.7); a
  future `"auto"`/system sentinel can be added as an allowed value.
- Wire a `CallbackRegistry::on_reload(&["ui.locale"], ...)` handler that reloads
  BOTH the message catalogue and the per-locale/per-environment alias catalogues,
  so a locale change is hot (matches existing `theme.active` hot-reload behaviour).

---

## Reproducibility appendix (exact searches)

- Crate count: `Get-ChildItem -Directory crates | Measure-Object` = 91.
- i18n deps: `rg "fluent|gettext|rust-i18n|unic-langid|\bicu\b|i18n|l10n"` over
  `**/Cargo.toml` = no matches.
- i18n source mechanism: `rg "\blocale\b|\btr!\b|translation|catalogue|i18n|l10n|
  gettext|fluent"` over `**/*.rs` = only unrelated hits (see Q1).
- egui render literals (non-test): `rg --pcre2 '\.(label|button|heading|
  selectable_label|hyperlink|monospace|small|strong|weak)\(\s*"' -g '*.rs'
  -g '!*/tests/*' -g '!*test*' crates` = 184 (per-crate table in Q2a).
- `RichText::new("...")` non-test = 39.
- `Some("...")` non-test (upper bound, over-broad) = 564.
- `format!("...")` non-test (upper bound, over-broad) = 1221.
- `open_error = Some("...")` = ~20 literal assignments (ff-desktop shell/ladders).
- Shipped menu files: `file_search "menus/"` = none (menus are code-only).

Uncertainty called out: the user-facing SUBSET of the 564 `Some("...")` and 1221
`format!("...")` sites was not hand-classified in this read-only pass; those two
figures are upper bounds, and the user-facing fraction is the main sizing risk
(it is why the overall estimate is MEDIUM, not SMALL).
