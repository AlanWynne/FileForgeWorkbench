# ISPF Editor Redesign -- Read-Only Investigation Findings

Repo: `c:/workspace/apps/FileForgeWorkbench` @ `a497100` (branch main, SCRM Phase 2E merged).
Scope: factual grounding for a forthcoming requirements effort covering the owner's
Snapshot-considerations items 2-7 (Config / Alias / Menu / Theme / Keys / Workspace-Kinds
editor redesigns, expanded key model, per-workspace keylists). READ-ONLY -- no code changed.

> NOTE on the source doc: `Snapshot-considerations.md` was NOT found anywhere in the repo
> (searched the whole tree) and the `scrm-2e-dialog-capture` worktree it lived in has been
> removed after the merge (`git worktree list` shows only the main worktree). The file was
> never committed. This report is therefore grounded in the codebase plus the owner's
> item descriptions carried in the task brief and the original user request (which contains
> the menu mock and the colour-names question). Where a precise mock detail matters it is
> flagged as "owner mock (from brief)".

---

## Summary verdicts (one line each)

| # | Area | Verdict |
|---|------|---------|
| 1 | Theme colour NAMES | **MISSING** -- no runtime hex->name source exists; "name if it exists" is effectively ALWAYS ABSENT. Separately, the theme snapshot VALUES are **MISSING** (provider writes empty strings) -- a real capture-fidelity bug. |
| 2 | Menu TOML structure | **ONE TOML PER MENU** (not one combined file). Model fully known; owner's "Alias" menu column has **no backing field**. |
| 3 | Per-workspace named keylists | **PARTIAL** -- a Kind can reference a named `key_list` and it IS wired to the resolver, but there is NO UI to create/save an arbitrary NAMED keylist; the Keys editor is locked to 12 fixed kind-context names. |
| 4 | Key assignment model | **PARTIAL** -- model is F1-F24 x {None,Shift,Ctrl,Alt} (96 slots), NOT "only F1-F12". But only FUNCTION keys exist (no letters), no modifier COMBOS, and NO protected-binding concept. Ctrl+C/V/S are not in the model at all. |
| 5 | Editor Contexts + line commands | **PARTIAL** -- the Alias editor ALREADY has ISPF line commands (I/R/D) and the owner's column model; a full ISPF line-command engine (incl. C/M/sort) exists in `ff-line-commands` but is used only by the text editor. Menu/Kinds editors use buttons, not line commands. |
| 6 | Markdown-as-table capture | **EXISTS (renderer) / MISSING (providers)** -- `ff-screen-model` fully supports `Table` in the model and the Markdown/HTML/text/YAML renderers; NO editor provider populates a `Table` yet (all use flat label/value fields). |

---

## 1. THEME COLOUR NAMES + captured values (item 4)

### 1a. Where colours are stored; is there any name source?
- Colours are the value type `ColourRGBA { r, g, b, a: u8 }` with hex parse/serialise only --
  `crates/ff-theme/src/colour.rs` (`from_hex`, `to_hex`, `Display` = hex). There is **no name
  field** anywhere on the colour type.
- The palette is a plain struct of `ColourRGBA` groups (editor/syntax/ui/...), addressed by the
  `ColourToken` enum in `crates/ff-theme/src/token.rs`. Each token has a `key_path()` returning a
  SETTING path like `"editor.background"`, `"syntax.keyword"`. **These are setting identifiers,
  not colour names.**
- The only "names" near colours are:
  - Rust `const` identifiers in `crates/ff-theme/src/defaults_solarized.rs`
    (`SOL_YELLOW`, `SOL_BASE03`, `SOL_BLUE`, ...), each a COMPILE-TIME constant with the hex in a
    trailing comment. These are not a runtime lookup; they are source symbols for ~16 Solarized
    accent/base tones, and the editable surface has far more colours than these.
  - THEME names (`Default Dark`, `Default Light`, `High Contrast`, `Default Legacy`, `Legacy Soft`)
    in `crates/ff-theme/src/discovery.rs` -- names of whole themes, not of individual colours.
- A codebase-wide search for any hex->name map / named-colour palette
  (`colour_name|color_name|named_colour|ColourName|hex_to_name|name_for_colour|Solarized|Catppuccin`)
  found only: the theme-name usages above, the `SOL_*` consts, and a config test using
  `theme.active = "solarized"`. **No hex->name lookup table exists.**

### 1b. Are the theme setting VALUES even captured today?
**No -- this is the blank-field bug the owner reports.** The Theme Editor provider is:

```rust
// crates/ff-desktop/src/shell/screen_providers.rs  (theme_editor_screen_model)
for token in EditableToken::all() {
    model = model.with_field(Field::new(token.label().to_string(), String::new())); // <-- empty value
}
```

Every field value is `String::new()`. The hex is never read from the theme. The capability to read
it is right there and already used by the live panel:
`EditableToken::get(&palette) -> ColourRGBA` then `.to_hex()`
(`crates/ff-theme-editor/src/editable_surface.rs`; used by `ThemeEditorState::load_working` to seed
`hex_buffers` in `crates/ff-theme-editor/src/lib.rs`). The provider simply does not fetch the
theme/working palette.

### 1c. Definitive answer on "name if it exists"
- There is **no existing hex->name source** to display from. The nearest thing (the `SOL_*` consts)
  is compile-time, covers only ~16 tones, is not keyed by hex at runtime, and names Solarized roles
  rather than user-facing colour names.
- Therefore **"name if it exists" would resolve to NO NAME for essentially every colour.** It is
  not viable as a feature with today's data. If the owner wants names, they must be authored (the
  exact thing the owner said they do NOT want to do), or a third-party hex->name crate/table would
  have to be added (a new dependency + new data, not an existing source).

**Recommendation:** split this into (i) a capture-fidelity FIX -- populate the hex value in the
Theme provider (trivial, uses `EditableToken::get(...).to_hex()`); and (ii) drop the colour-NAME
column, or show the token KEY-PATH / label as the "name" column (which DOES exist:
`ColourToken::key_path()` / `EditableToken::label()`), since a real colour-name source does not exist.

---

## 2. MENU TOML STRUCTURE (item 5)

**Verdict: ONE TOML file PER MENU kind.** There is no single combined file.

- A menu file deserialises to one `MenuFile` -- `crates/ff-desktop/src/menu_workspace/mod.rs`:
  `MenuFile { title, options: Vec<MenuOption>, show_calendar, group_separator, group_headers }`.
- Built-ins are CODE-ONLY, one const per menu:
  `DEFAULT_POM_TOML` and `DEFAULT_SETTINGS_TOML` in
  `crates/ff-desktop/src/menu_workspace/defaults.rs` (parsed by `recovery_pom_menu()` /
  `recovery_settings_menu()`). They are NEVER written to disk (framework rule, menu-workspace Req 4.1/4.2).
- User menus are per-file on disk: `menus/<name>.toml`, each loaded by
  `load_menu_file(path)` (`crates/ff-desktop/src/menu_workspace/loader.rs`), which parses a
  `RawMenuFile` -> validates -> `MenuFile`. A user `menus/<name>.toml` OVERRIDES the compiled default
  of the same name.
- The Menus editor's `available` list is `"POM"`, `"Settings"`, plus every user menu file name
  (`crates/ff-desktop/src/menus_editor_panel/state.rs`). One menu is edited at a time
  (`working: Option<MenuFile>`).

**Actual `MenuOption` fields** (`menu_workspace/mod.rs`, confirmed in `loader.rs` `RawMenuOption`):

| Field | Type | Notes |
|-------|------|-------|
| `key` | `String` | 1-4 chars, stored uppercase (Act/Option key) |
| `command` | `String` | primary command to run |
| `description` | `String` | one-line description |
| `enabled` | `bool` | default true |
| `group` | `Option<String>` | group label |
| `show_in_menu_bar` | `bool` | default true (owner's "Bar" column) |
| `target` | `Option<ff_command::CommandTarget>` | inline Command_Target override (Req 10.6) |

**Menu-level fields** (owner's "Menu name / Show Calendar / Group Headers / Group Separator"):
`title`, `show_calendar: bool`, `group_headers: bool`, `group_separator: GroupSeparator`
(`Space` | `Line`, serde lowercase). All present.

**Gap vs owner's Menu-editor mock (from brief):** the mock's columns are
`Act / Option / Command / Alias / Description / Group / Bar`. Mapping:
Act = line-command field (NEW -- see section 5), Option = `key`, Command = `command`,
Description = `description`, Group = `group`, Bar = `show_in_menu_bar`. **`Alias` has NO backing
field on `MenuOption`** -- it would be a new field, OR it is the aliases-for-a-command concept that
lives in the separate Alias editor (`AliasTable`), not on the menu. This needs an owner decision and
(if a per-option alias) a framework-touching schema addition to `MenuOption` + its TOML.

---

## 3. PER-WORKSPACE KEYLISTS (item 7)

**Verdict: PARTIAL. The "named keylist referenced by a workspace" plumbing EXISTS and is wired;
the "create/save an arbitrary NAMED keylist and reuse it" authoring UI does NOT.** The owner's
belief that it "may have been lost" is close: the REFERENCE side is live, the AUTHORING side was
never general -- the Keys editor only ever edited the fixed per-kind contexts.

### What EXISTS
- `KindConfig.key_list: Option<String>` -- a named key-list reference per Workspace Kind
  (`crates/ff-desktop/src/workspace_kind/mod.rs`; round-trips through `KindConfigToml`). Editable in
  the Kinds editor (`kinds_editor_panel/render.rs` -> `edit_optional(... &mut cfg.key_list)`).
- It IS consumed: `key_list_context_for_tab` (`crates/ff-desktop/src/shell/titles.rs`) returns the
  Kind's configured `key_list` when set, else the base kind context; the render loop calls
  `self.key_map_resolver.set_context(ctx_name)` each frame
  (`crates/ff-desktop/src/shell/render_tab_bar.rs`). Test proof:
  `tests_menu_workspace.rs` -- "a Kind's configured key_list selects that keymap context" (`"mf"`).
- `KeyMapResolver` (`crates/ff-keys/src/key_map_resolver.rs`) resolves
  **context > profile > global** (full-replacement). Context maps come from config
  `[context_key_maps]` and `keymaps/<context>.toml` FILES, loaded at startup
  (`crates/ff-desktop/src/shell/construct.rs` -> `load_context_maps_from_config` /
  `load_context_maps_from_keymaps_dir`). A `KeyMap` even carries a source NAME (`KeyMap::empty(name)`).

### What is MISSING vs the owner's intent ("each workspace can have its own customisable, named,
reusable keylist")
- The Keys editor edits a key list keyed by a **fixed set of 12 kind-context names**
  (`KIND_NAMES` in `crates/ff-desktop/src/keys_editor_panel/state.rs`:
  `pom, editor, config, files, search, plugins, log, macros, menu, commands, theme, menus`),
  writing `keymaps/<kind>.toml` (`crates/ff-desktop/src/shell/keys_editor.rs`
  `load_key_map_for_kind` / `save_keys_for_kind`). There is NO way to type a brand-new keylist name
  (e.g. `mylist`), save it, and then point a Kind's `key_list` at it from the UI.
- So the two halves do not meet: `KindConfig.key_list` can name ANY context, but the only contexts a
  user can author through the UI are the 12 fixed kind names. A keylist file could be created by hand
  on disk, but that is not the "named, reusable, user-managed" story.
- Net: reference + resolver EXIST; a NAMED-keylist registry/authoring surface (list, create, rename,
  save-as, pick) is MISSING and is genuinely new work (not a framework change -- it reuses the
  existing resolver context mechanism and `keymaps/*.toml` file model).

---

## 4. KEY ASSIGNMENT MODEL (item 6)

### 4a. What the current model supports
- `crates/ff-keys/src/function_key.rs`:
  - `FunctionKey` = **F1-F24** (not just F1-F12). F1 is a SOFT reservation ("reserved for context
    help"; `MIN_ASSIGNABLE = F2`, `is_assignable()` = number >= 2). This is the only "reserved"
    concept and it is just F1.
  - `KeyModifier` = `None | Shift | Ctrl | Alt` (exactly 4; `KeyModifier::ALL`).
  - `ModifiedKey` = one `FunctionKey` + one `KeyModifier` = **96 addressable slots**
    (`ModifiedKey::all()` = None x24, Shift x24, Ctrl x24, Alt x24). TOML prefixes `F/SF/CF/AF`.
- `KeyMap` (`crates/ff-keys/src/key_map.rs`) maps `ModifiedKey -> KeyBinding { command, label?, description? }`.

### 4b. Representable vs the owner's expanded vision
- **Representable today:** F1-F24 with a SINGLE modifier each (None/Shift/Ctrl/Alt).
- **NOT representable (owner wants, gap):**
  - Any NON-function key (letters, digits, Enter, punctuation) -- the key set is function-keys only.
    So "all assignable keys" is NOT representable.
  - Modifier COMBOS such as Shift+Ctrl -- `KeyModifier` is a single-choice enum, no bitflags; there
    is no `Shift+Ctrl`/`Ctrl+Alt` variant.
  - Consequently Ctrl+C / Ctrl+V / Ctrl+S cannot even be EXPRESSED in the key map (no letter keys).

### 4c. Protected / reserved bindings today
- The ONLY reservation is F1 = Help (a soft rule in `FunctionKey`, not a general mechanism). There is
  **no "protected/reserved binding" concept** -- no flag, list, or guard preventing reassignment of
  specific combos.
- Ctrl+C / Ctrl+V / Ctrl+S are handled OUTSIDE the key map (egui/clipboard and SAVE handling in the
  shell), so they are implicitly "always on" simply because they are not in the configurable map.
  A "these defaults are protected from reassignment" rule would be entirely NEW, and it only becomes
  meaningful once letters + modifier combos are added to the model (4b).

**Gap summary:** expanding to "all assignable keys + Base/Shift/Ctrl/Alt + combos + protected
defaults" requires a model change in `ff-keys` (a richer key enum beyond `FunctionKey`, a
combinable modifier set, and a new protected-binding concept). This is a genuine new-feature +
model change, the largest of the six.

---

## 5. EDITOR CONTEXTS + LINE COMMANDS (items 2,3,5,7)

### 5a. Do any Contexts already implement ISPF line commands?
**Yes, two independent facts:**
1. **A complete ISPF line-command ENGINE exists** -- `crates/ff-line-commands` (GUI-free): parser,
   classification, block pairing, pending store, compatibility matrix, execution for
   **D/I/R, C (copy), M (move), X, T/U, shift `<`/`>`/`(`/`)`** (`src/lib.rs` header +
   `command`/`parser`/`block_pair`/`commands` modules). It is used by the TEXT editor's prefix area,
   NOT by the config/table editors.
2. **The Alias editor already implements a MINI line-command flow** on its table --
   `AliasRowEdit.act` is a "Transient ISPF line-command letter (I / R / D)" applied as a batch:
   `crates/ff-desktop/src/shell/alias_editor_line_commands.rs` (`apply_pending_line_commands`:
   process by DESCENDING index; `D`=remove, `I`=insert blank after, `R`=field-copy after). It
   supports **I/R/D only -- NOT C, M, or S (sort).**

### 5b. The editable-Context seam and how each editor is built today
- Shared seam CR-CH-071: `crates/ff-desktop/src/shell/editable_context.rs`
  (`editable_context_kind` / `editable_persist` / `editable_discard` / `editable_reload`), with one
  `commands_*_save.rs` per editor (Alias/Theme/Config/Menus/Keys/Kinds/CommandConfigurator). SAVE
  persists + stays, END persists + exits, RETURN/CANCEL discards + exits (ISPF save-convention rule).
- Each editor is a pure `render(...) -> Action` panel + a shell `apply_*_action`
  (`WorkspaceContext` framework, CR-NR-078). Current field sets:

| Editor | Current fields / UI | Owner target (mock) | Gap |
|--------|---------------------|---------------------|-----|
| **Alias** (`alias_editor_panel/`) | Command===> line, FILTER row, columns **Act / CE / Root / ALIAS / Parameters**; line cmds **I/R/D**; empty-store scratch row | Act / CE / Command / Alias / Parameters / **Description** + **sort** | Add **Description** column (was L18N, removed in CR-CH-072); add **S (sort)** line command |
| **Menu** (`menus_editor_panel/`) | per-option **key / command / description / group / enabled / show_in_menu_bar**, structural **Up(^)/Down(v)/Delete(x) BUTTONS** + Add; menu-level title/show_calendar/group_separator/group_headers | Act / Option / Command / **Alias** / Description / Group / Bar + line cmds incl **move** | Replace buttons with ISPF line cmds (I/D/C/**M**); add **Alias** column (NO backing field, see section 2) |
| **Kinds** (`kinds_editor_panel/`) | vertical FORM: selector, "new modelled on <base>", Title, Menu Bar, Key List, profile **CAPS/NULLS/STATS/LOCK/HILITE**, tab size, line endings, cmd-line pos | table: Act / Workspace / Modeled / Title / Menu Bar / Keylist + profile flags; **move** | Re-present as an ISPF table with Act line cmds (incl **M**); all underlying fields already exist |
| **Config** (`config_panel` / `screen_providers_config.rs`) | tree of config rows | fuller ISPF field set | (not deeply inspected; same seam) |
| **Theme** (`theme_editor_panel` / `ff-theme-editor`) | token rows: label + hex buffer + swatch | hex + name-if-exists | see section 1 (name source absent; value capture bug) |

- Owner's profile flags ALREADY EXIST in the model: `EditProfile { caps, nulls, stats, lock, hilite }`
  in `crates/ff-edit-operations/src/profile.rs`, carried on `KindProfile.edit_profile`
  (`workspace_kind/mod.rs`) and already edited by the Kinds panel.

### 5c. Is a shared ISPF line-command + table-editor component warranted?
**Yes.** Three editors (Alias/Menu/Kinds) want the same thing -- a table with an Act prefix column,
a `Command ===>` line, a filter row, and batch line commands (I/D/R/C/M + sort/move). Today only the
Alias editor has a bespoke I/R/D implementation and the real engine
(`ff-line-commands`) is wired only to the text editor. A shared table-editor + line-command adapter
(reusing `ff-line-commands` parsing/classification where possible) would remove duplication and give
all three editors C/M/sort uniformly. This is a Context-redesign, built ON the existing
`WorkspaceContext` + editable-Context seam (not a framework change).

---

## 6. MARKDOWN-AS-TABLE CAPTURE (items 3,4)

**Verdict: renderer support EXISTS; provider usage is MISSING.**

- `ff-screen-model` already models tables: `Table { caption: Option<String>, headers: Vec<String>,
  rows: Vec<Vec<String>> }` and `ScreenModel.tables: Vec<Table>`
  (`crates/ff-screen-model/src/model.rs`; re-exported in `src/lib.rs`).
- Every text renderer emits tables:
  - Markdown -- `src/render/markdown.rs`: a code-block of screen text PLUS a property table for
    `fields` PLUS one Markdown table per `Table` element (test
    `table_element_renders_as_markdown_table`).
  - HTML -- `src/render/html.rs` (`<table class="ffwb-table">`), text grid -- `src/render/text.rs`,
    YAML -- `src/render/yaml.rs`. (This is the SCRM work the owner recalls adding tables.)
- BUT **no editor provider populates `model.tables`.** All editor providers in
  `crates/ff-desktop/src/shell/screen_providers.rs` (Theme/Menus/Keys/Kinds/Alias/Config/...) build
  only flat `with_field(label, value)` pairs. So the tabular editor snapshots currently render as a
  flat field list, exactly as the owner observed.

**Recommendation:** this is a PROVIDER change only (populate a `Table` with the editor's real
columns) -- NO new renderer work. Pairs naturally with the capture-fidelity fixes in sections 1b/5.

---

## Recommended requirements structure (how to slice items 2-7)

Grouped by nature of the work. "Framework change" is flagged per
`framework-conformance.md` (needs explicit owner confirmation).

### Group A -- Capture-fidelity FIXES (small, no framework change, do first)
These make SNAPSHOT/CAPTURE of the editor Contexts faithful; they are bug-class fixes, not redesigns.
1. **Theme snapshot shows blank values** (item 4, section 1b). Populate hex via
   `EditableToken::get(working_palette).to_hex()` in `theme_editor_screen_model`. The provider needs
   access to the Theme Editor working palette (`ThemeEditorState.working`) -- wire that in.
2. **Editor snapshots as TABLES** (items 3,4, section 6). Populate `model.tables` in the
   Theme/Alias/Menu/Kinds/Keys/Config providers with their real columns. Renderer already supports it.
   (Gate as capture-fidelity, reusing `ff-screen-model::Table`.)

### Group B -- Context REDESIGNS (medium; built ON WorkspaceContext + editable-Context seam; NOT framework changes)
3. **Shared ISPF table-editor + line-command component** (item 5, section 5c). Generalise the Alias
   mini-engine (or adapt `ff-line-commands`) into a reusable table editor with Act column,
   `Command ===>`, filter, and batch I/D/R/C/M + sort/move. Then:
   - **Alias editor**: add Description column, add S (sort). (NEW column is additive.)
   - **Menu editor**: replace Up/Down/Delete buttons with ISPF line commands incl. Move.
   - **Kinds editor**: re-present the existing form as an ISPF table (Act/Workspace/Modeled/Title/
     Menu Bar/Keylist + profile flags) with Move. All fields already exist -- presentation only.
4. **Config editor ISPF redesign** (items 2,3) -- same seam; fuller field set (scope TBD after a
   closer Config-panel read).

### Group C -- Genuinely NEW features (larger; some touch core models)
5. **Named, reusable keylists** (item 7, section 3). NEW authoring surface: list/create/rename/
   save-as named keylists written as `keymaps/<name>.toml`, and let a Kind's `key_list` pick from
   them. Reuses the existing `KeyMapResolver` context mechanism and file model -- **NOT a framework
   change**, but new UI + new registry. (The reference + resolver halves already exist.)
6. **Expanded key-assignment model** (item 6, section 4). Extend `ff-keys` beyond `FunctionKey`
   to all assignable keys, a combinable modifier set (Shift+Ctrl etc.), and a NEW protected/reserved
   binding concept (Ctrl+C/V/S etc.). **This is the one that changes a core public model** (`ff-keys`
   `FunctionKey`/`KeyModifier`/`ModifiedKey`, serialised `keymaps/*.toml` schema). Treat as a
   FRAMEWORK/CORE-MODEL CHANGE requiring explicit owner confirmation and a migration story for
   existing `F/SF/CF/AF` TOML keys. Protected bindings only become meaningful once letters + combos
   exist, so sequence 6 after the model widening.

### Flagged framework / core-model changes (need explicit owner confirmation)
- **Item 6 key-model widening (Group C.6)** -- reshapes public `ff-keys` types and the persisted
  keymap TOML schema. FRAMEWORK CHANGE.
- **Menu-option "Alias" column (item 5, section 2)** -- if it means a per-option alias, it adds a
  field to `MenuOption` and the menu TOML schema. Schema addition -- confirm whether it is a new
  field or simply surfacing the existing separate Alias table. Decision needed before specifying.
- Everything else (Groups A, B, C.5) builds ON existing framework seams
  (`WorkspaceContext`, editable-Context CR-CH-071, `KeyMapResolver` contexts, `ff-screen-model::Table`)
  and is additive.

### Explicit note on the colour-name question (owner's direct ask)
"Colour name only if it exists" is **not viable** with the current codebase: there is NO hex->name
source (only ~16 compile-time `SOL_*` role constants and whole-theme names). For practically every
colour it would resolve to "no name". Recommend dropping the name column or substituting the token
KEY-PATH/label (which exist) as the descriptive column, and treating the blank-hex issue as the real
bug to fix.
