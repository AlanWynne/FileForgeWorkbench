# Implementation Plan -- Localisation Phase 2, Task 8 / 8.1 (CR-NR-103)

Convert the `ff-desktop` crate's user-facing render literals (labels, buttons,
hover/hint text, headings, status strings, window titles) to the `ff_i18n`
Catalogue_Lookup_Seam. Behaviour-neutral extraction: at `ui.locale = "en"` the
rendered text MUST stay BYTE-IDENTICAL (Req 10.2). Keeps render APIs and
behaviour unchanged; keeps focus/behaviour `egui_kittest` tests green
(Req 3.4, 4.1, 10.1, 10.2).

Worktree root (all paths relative to it):
`c:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-task8`

---

## Established idiom (copy Task 7 exactly -- do NOT reinvent)

`ff-i18n` is ALREADY a dependency of `ff-desktop` (see `crates/ff-desktop/Cargo.toml`,
line `ff-i18n = { path = "../ff-i18n" }`). No Cargo edit needed.

At each render site replace the literal:
- Plain string -> `ff_i18n::t("key")` (returns `String`; egui `.label(...)`,
  `.button(...)`, `.heading(...)`, `.on_hover_text(...)`, `Window::new(...)`,
  `RichText::new(...)` all accept it directly).
- `format!`/interpolated -> `ff_i18n::t_args("key", &ff_i18n::MessageArgs::new().set("name", value))`
  or `.set_number("n", x as f64)`. Pattern model: `crates/ff-theme-editor/src/lib.rs`
  (`theme-editor-advisory-row`, `theme-editor-reset-confirm`) and
  `crates/ff-i18n/i18n/en/catalog-dialog.ftl` (`catalog-dialog-new-err-duplicate`).
- A pre-formatted numeric (e.g. a version or ratio already rendered to text in
  Rust) is passed as a TEXT arg via `.set(...)`, NOT `.set_number(...)`, so the
  rendering stays byte-identical (model: `theme-editor-advisory-row`).

`.ftl` values are the EXACT current English literal, byte-for-byte, ASCII ONLY.
One `.ftl` file per area under `crates/ff-i18n/i18n/en/`, named
`ff-desktop-<area>.ftl`, every key prefixed `ff-desktop-<area>-...` for a unique
namespace.

---

## CRITICAL pitfalls (designed-against; verify every area)

1. **KEY COLLISIONS BREAK THE WHOLE `en` BUNDLE.** `ff-i18n` merges EVERY `.ftl`
   in `i18n/en/` into one Fluent bundle; a duplicate message id across ANY two
   files makes the entire English bundle fail (surfaces as unrelated tests
   panicking with `BundleBuild ... bundle error(s)`). Namespace every new key
   with a unique `ff-desktop-<area>-` prefix. Do NOT reuse any existing id.
   Reserved ids already in the bundle that MUST NOT be reused: `app-name`,
   `files-panel-title`, `ok-button`, `cancel-button`, `welcome-user`,
   `profile-locked` (main.ftl); plus the `menus-`, `catalog-dialog-`,
   `dataset-alloc-dialog-`, `explorer-view-`, `files-panel-`, `theme-editor-`,
   `toolchain-panel-` prefixes owned by Task 5/7 files. GUARD: after EVERY `.ftl`
   change run
   `C:\tools\python\python.exe tools\python\ftl_dupe_check.py crates\ff-i18n\i18n\en`
   and confirm it reports `no duplicate message ids`.

2. **ASCII-ONLY `.ftl`.** Express any typographic / non-ASCII char with a Fluent
   unicode-escape placeable so RENDERED text stays byte-identical while source is
   ASCII: `{ "\u2014" }` em dash, `{ "\u2026" }` ellipsis, `{ "\u00d7" }` the
   `\u{00d7}` multiply sign (tab close `X`), `{ "\u00a9" }` the copyright sign.
   Preserve significant surrounding whitespace Fluent would otherwise trim with a
   quoted placeable, e.g. the About Close button `"  Close  "` ->
   `{ "  " }Close{ "  " }` (model: `catalog-dialog-new-label-name` uses `{ "  " }`).

3. **FULL-SHELL TESTS (biggest risk).** `harness_shell()` already installs the
   catalogue (`crates/ff-desktop/src/shell/tests_common.rs` ->
   `i18n_bootstrap::install_catalogue_for_tests()`), so byte-identical `.ftl`
   values keep every test green. Two sub-risks:
   - Most `ff-desktop` focus tests assert on `first_interior_id` /
     `assert_first_tab_lands_on_reported_interior`, NOT on label text, so they
     are insensitive to the swap as long as ids are unchanged. Do NOT touch any
     `.id(...)` / `id_salt` / `egui::Id::new(...)` value.
   - CLICK-THEN-ASSERT tests that find a widget by English label and click it
     are the fragile ones. The ONLY such test currently in `ff-desktop` is
     `full_shell_theme_editor_type_name_and_save_creates_user_theme`
     (`crates/ff-desktop/src/shell/tests_focus.rs:1251`, `get_by_label("Save")`),
     and its "Save" comes from `ff-theme-editor` (Task 7, OUT of scope here) --
     Task 8 does not change it. No new click-by-label test is required; keep the
     existing settle-frame pattern if any area later adds one.

---

## What to LEAVE UNLOCALISED (do NOT convert; note in commit)

- **Command verbs / typed surface forms.** The menu bar buttons are labelled by
  `option.command` (the verb), not a description -- see
  `crates/ff-desktop/src/shell/render_chrome.rs` (`ui.menu_button(option.command...)`).
  Leave untouched (Req 4.2, 4.3).
- **Menu / POM option titles and descriptions** -- ALREADY localised in Task 5
  via `menus.ftl` and `crates/ff-desktop/src/menu_workspace/i18n_keys.rs`. Do NOT
  double-localise. (`menu_workspace/render.rs` keeps only the
  "No options defined in this menu." placeholder, which IS in scope -- area M.)
- **egui ids / `id_salt` / `Id::new(...)` values, enum logic keys, `command_id`
  strings, `Topic_Key`s, `toml_name()` map keys, config key names** (Req 4.2, 4.4).
- **Proper nouns:** the product name string `"FileForge Workbench"`, `AI_CREDIT`
  (contains "Amazon Q Developer", "AWS"), OS/tool names (Explorer / CMD /
  PowerShell / Terminal), "Rust"/"GCC". Localise only the surrounding prose, keep
  the proper noun as source text (or a non-translated arg).
- **Technical codes / mnemonics / glob examples** (`**/*.rs`, `**/target/**`,
  `Aa`/`W`/`.*` checkbox captions are mnemonic toggles -- leave; their HOVER text
  IS prose and IS in scope).
- **Clipboard / interchange text formats.**
- **Any model-layer string / helper fn / `pub const` with a unit test asserting
  its English text.** Before converting ANY non-inline string (a `pub const`, a
  `fn ...() -> &'static str` mapping, an enum-to-label helper), grep the file's
  own `#[cfg(test)]` module for an assertion on that English value
  (`.contains("...")`, `assert_eq!(f(...), "...")`). If one exists, LEAVE the
  string unlocalised and note it in the commit -- localising it would break the
  test, and Task 8 must keep all tests green. Confirmed instances already found:
  `about_dialog::{CREATOR, AI_CREDIT, COPYRIGHT, DESCRIPTION}` (area 18) and
  `plugin_manager_panel::state_label` (area 5). When unsure whether a string is
  translatable UI prose vs a logic/identity value, LEAVE it and note the decision.
- **Out of scope (Task 7, already done):** crates `ff-catalog-dialog`,
  `ff-dataset-alloc-dialog`, `ff-files-panel`, `ff-explorer-view`,
  `ff-theme-editor`, `ff-toolchain-panel`. Their thin `ff-desktop` adapter/render
  arms and anything `ff-desktop` renders directly ARE in scope (e.g. the
  "Toolchain Panel" / "File Explorer" headings rendered in `shell/render.rs` and
  `shell/render_nav.rs`).

---

## Verification policy (this machine's terminal wedges on long/quiet commands)

Agents COMPILE-VERIFY only, SCOPED, reading the log back. For each command run
via the clean wrapper and then READ the log with the read tool:

```
C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<cmd> *> tools\logs\<name>.log"
```

Scoped commands (one job each, never `;`-chained):
- `cargo fmt -p ff-desktop --manifest-path Cargo.toml`  (log: fmt.log)
- `cargo check -p ff-desktop --manifest-path Cargo.toml` (log: check.log)
- `cargo clippy -p ff-desktop --manifest-path Cargo.toml` (log: clippy.log)
- The FTL guard after every `.ftl` edit:
  `C:\tools\python\python.exe tools\python\ftl_dupe_check.py crates\ff-i18n\i18n\en`
  (log: ftl-dupe.log) -- MUST say `no duplicate message ids`.
- A SCOPED, bounded label-finding / focus `egui_kittest` test per changed panel
  MAY be run: `cargo test -p ff-desktop --bin ffwb <test_name> *> tools\logs\<name>.log`.

Do NOT run whole `cargo test` / `nextest` / `cargo gate` -- the OWNER runs the
authoritative `cargo gate --build` at the end. Confirm success by the log
CONTENT, never by the reported exit code.

Commit PER AREA, staging specific files only (never `git add -A`):
`feat(l10n): task8 <area> localise render literals`. Keep each source file under
400 non-test lines (the swaps are in-place and do not grow files materially;
verify no file crosses 400 after edit).

---

## Ordered AREAS (each = one implement + scoped-verify + commit unit)

Ordering rationale: self-contained panels with pure first-interior focus tests
first (lowest risk, establish the pattern + the first `ff-desktop-*.ftl` files),
then the shell chrome, then the POM/calendar and dialogs last (more prose, more
interpolation). Areas are INDEPENDENT of each other (no area depends on
another's output); order is for risk-laddering and reviewability only.

---

- [x] 1. Area COMMAND-CONFIG: Command Configurator panel.
      Swap render literals in the configurator list/toolbar and the edit form:
      `render.rs` -- "Add", "No user-defined commands. Use Add to create one.",
      "Delete", "Cancel", "Save" (+ the `format!("Delete command definition '{id}'?")`
      -> `t_args` with `{ $id }`); `edit.rs` -- field labels "Id", "Label",
      "Description", "Variant", "Menu name", "Workspace kind", "Command id",
      "Macro", "Is path", "treat as file path", "Program", "Args", "Working dir",
      "Captured", "capture output". Leave `TargetVariant` enum match logic and
      `self.variant.label()` identity alone.
      Files: `crates/ff-desktop/src/command_config/render.rs`,
      `crates/ff-desktop/src/command_config/edit.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-command-config.ftl` (prefix
      `ff-desktop-command-config-`).
      Verify: `ftl_dupe_check` reports no duplicates; `cargo check -p ff-desktop`
      clean; run the configurator first-Tab focus test
      (`cargo test -p ff-desktop --bin ffwb command_config` -- the
      first-interior-on-Add test) and confirm PASS in the log.

- [x] 2. Area CONFIG-PANEL: Configuration Context panel + value widgets.
      `render.rs` -- "Filter:", "Source File:"; `widgets.rs` --
      "[complex value -- edit TOML file directly]" (the `--` is ASCII, no escape
      needed). Do NOT touch the filter field `.id(...)` (B058 anchor).
      Files: `crates/ff-desktop/src/config_panel/render.rs`,
      `crates/ff-desktop/src/config_panel/widgets.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-config-panel.ftl` (prefix
      `ff-desktop-config-panel-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run
      `cargo test -p ff-desktop --bin ffwb full_shell_config_first_tab_focuses_filter_field`
      and confirm PASS (the filter field id is unchanged, so first-Tab still lands
      on it).

- [x] 3. Area EVENT-LOG: Event Log panel.
      `event_log_panel.rs` -- heading `RichText::new("Event Log")`, filter
      `hint_text("search...")`, "No log entries." Leave log-level enum captions
      that are logic/identity; convert only displayed prose.
      Files: `crates/ff-desktop/src/event_log_panel.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-event-log.ftl` (prefix
      `ff-desktop-event-log-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      event-log first-Tab focus test
      (`cargo test -p ff-desktop --bin ffwb event_log`) and confirm PASS.

- [ ] 4. Area MACRO-LIBRARY: Macro Library panel.
      `macro_library_panel.rs` -- "Filter:", "No macros found. Place .lua files
      in your macro directories.", header `RichText::new("Name")` and
      `RichText::new("Directory")`. Keep the `"x"` clear small_button mnemonic as
      a glyph (it is a UI glyph, not prose) OR localise consistently with other
      panels' "Clear" -- LEAVE "x" (mnemonic glyph) and note the decision. Do NOT
      touch `filter_field_id()`.
      Files: `crates/ff-desktop/src/macro_library_panel.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-macro-library.ftl` (prefix
      `ff-desktop-macro-library-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      macro-library first-Tab focus test
      (`cargo test -p ff-desktop --bin ffwb macro_library`) and confirm PASS.

- [ ] 5. Area PLUGIN-MANAGER: Plugin Manager panel.
      `plugin_manager_panel.rs` -- heading `RichText::new("Plugin Manager")`,
      "Filter:", `hint_text("plugin name...")`, "Clear", "State:",
      "No plugins registered.", "No additional metadata available.", "Select a
      plugin to view details." LEAVE `state_label(...)` UNLOCALISED: it has unit
      tests asserting exact equality (`active_plugin_state_label_is_active` ->
      `assert_eq!(state_label(&PluginState::Active), "Active")`,
      `shutdown_plugin_state_label_is_shutdown` -> `"Shutdown"`), so it falls
      under the "model string with an asserting test -- LEAVE it" rule; record
      the decision. Do NOT touch `filter_field_id()`.
      Files: `crates/ff-desktop/src/plugin_manager_panel.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-plugin-manager.ftl` (prefix
      `ff-desktop-plugin-manager-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      plugin-manager first-Tab focus test
      (`cargo test -p ff-desktop --bin ffwb plugin_manager`) and confirm PASS.

- [ ] 6. Area KEYS-EDITOR: Keys Editor panel.
      `keys_editor_panel/render.rs` -- "Workspace kind:", "Select a workspace
      kind to edit its key list.", `hint_text("unassigned")`, "Reset to default",
      `RichText::new("Saves to keymaps/<kind>.toml")`. The Save button text
      ("Save") -- localise. Leave `mk.toml_name()` and the per-row
      `egui::Id::new(("keys_editor_cmd", ...))` ids. Treat `keymaps/<kind>.toml`
      as prose with an embedded path token; keep it as one literal value (ASCII,
      no interpolation needed).
      Files: `crates/ff-desktop/src/keys_editor_panel/render.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-keys-editor.ftl` (prefix
      `ff-desktop-keys-editor-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      keys-editor first-Tab focus test
      (`cargo test -p ff-desktop --bin ffwb keys_editor`) and confirm PASS.

- [ ] 7. Area KINDS-EDITOR: Workspace Kinds Editor panel.
      `kinds_editor_panel/render.rs` -- "(create a new Kind to change the base)",
      `RichText::new("Profile")`, "Saves to workspace-kinds/<name>.toml", and any
      other displayed prose/button in this file (sweep the whole file for
      `.label(`/`.button(`/`RichText::new("`). Leave `base_label` if it is a
      Kind identity string with an asserting test (check
      `crates/ff-desktop/src/workspace_kind/tests.rs` --
      `display_title()` values like "Configuration" ARE asserted there, so do NOT
      route `display_title()` through `t()`; only localise literals rendered
      directly in this panel's render fn).
      Files: `crates/ff-desktop/src/kinds_editor_panel/render.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-kinds-editor.ftl` (prefix
      `ff-desktop-kinds-editor-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      kinds-editor first-Tab focus test
      (`cargo test -p ff-desktop --bin ffwb kinds_editor`) and confirm PASS.

- [ ] 8. Area MENUS-EDITOR: Menus Editor panel.
      `menus_editor_panel/render.rs` -- "Menu:", "Select a menu to edit.",
      "Title:", "Group separator:", the group-separator option labels
      ("Space", ...), `hint_text("key")`, `hint_text("command")`, the
      description `hint_text`, `RichText::new("Options (key | command |
      description | group)")`, and any Save/Reset buttons (sweep the whole file).
      The `"key"`/`"command"` hints are prose hints (localise); the per-row
      `egui::Id::new(("menus_opt_*", i))` ids stay. Leave `menu.title` buffer
      value (user data) untouched -- only the "Title:" LABEL is localised.
      Files: `crates/ff-desktop/src/menus_editor_panel/render.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-menus-editor.ftl` (prefix
      `ff-desktop-menus-editor-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      menus-editor first-Tab focus test
      (`cargo test -p ff-desktop --bin ffwb menus_editor`) and confirm PASS.

- [ ] 9. Area COMMAND-PALETTE: Command Palette overlay.
      `command_palette/render.rs` -- `hint_text("Type to search commands...")`,
      `RichText::new("Recently Used")`. Leave command id/label rows that come
      from the command store (those are data, and their labels are the command
      definition's own text, not a render literal here). Do NOT touch the
      `search_id`.
      Files: `crates/ff-desktop/src/command_palette/render.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-command-palette.ftl` (prefix
      `ff-desktop-command-palette-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      command-palette test(s)
      (`cargo test -p ff-desktop --bin ffwb command_palette`) and confirm PASS.

- [ ] 10. Area SEARCH-RESULTS: Search Results panel.
      `search_results_panel/render.rs` -- "Search:",
      `hint_text("Search across files...")`, "Search", "Cancel",
      `on_hover_text("Search history")`, the checkbox HOVER texts
      ("Case sensitive", "Whole word", "Use regex") [keep the `Aa`/`W`/`.*`
      captions as mnemonics -- LEAVE], "Include:", "Exclude:" [keep the
      `**/*.rs` / `**/target/**` glob hint_texts as technical examples -- LEAVE],
      `hint_text("Replacement text")`, "Replace All", the
      `format!("Replace {mc} occurrence(s) in {fc} file(s)?")` ->
      `t_args` with `{ $mc }`/`{ $fc }` as numbers, "Replace", "Cancel",
      "Search cancelled." Do NOT touch `query_field_id()`.
      Files: `crates/ff-desktop/src/search_results_panel/render.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-search-results.ftl` (prefix
      `ff-desktop-search-results-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      search-results first-Tab focus test
      (`cargo test -p ff-desktop --bin ffwb search_results`) and confirm PASS.

- [ ] 11. Area HELP-CONTEXT: Help Context panel.
      `help_context.rs` -- "Search:", `hint_text("type to search help")`,
      "No help topic is open." Leave `topic.body()` (localised content comes from
      the per-locale help directory, Task 6) and `help_search_field_id()`.
      Files: `crates/ff-desktop/src/help_context.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-help-context.ftl` (prefix
      `ff-desktop-help-context-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run any
      help-context focus test (`cargo test -p ff-desktop --bin ffwb help`) and
      confirm PASS.

- [ ] 12. Area MENU-WORKSPACE: Menu Workspace render placeholder only.
      `menu_workspace/render.rs` -- ONLY "No options defined in this menu."
      (the empty-options placeholder, Req 2.6). Everything else in menu_workspace
      (titles/descriptions) is ALREADY localised via `i18n_keys.rs` (Task 5) --
      do NOT touch those paths.
      Files: `crates/ff-desktop/src/menu_workspace/render.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-menu-workspace.ftl` (prefix
      `ff-desktop-menu-workspace-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run a
      menu-workspace test (`cargo test -p ff-desktop --bin ffwb menu_workspace`)
      and confirm PASS.

- [ ] 13. Area SHELL-COMMAND-LINE: Command / Scroll field chrome.
      `shell/render_command_line.rs` -- "Command ===>" (two sites), "SCROLL ===>".
      These are the field PROMPT labels (prose chrome), NOT the typed command;
      localise them. Do NOT touch `command_field_input` / `scroll_field_input`
      ids or the command buffer values.
      Files: `crates/ff-desktop/src/shell/render_command_line.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-shell-commandline.ftl` (prefix
      `ff-desktop-shell-commandline-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      command-field / scroll focus tests
      (`cargo test -p ff-desktop --bin ffwb command_field` and
      `... scroll_field`) and confirm PASS (ids unchanged so entry-focus and
      first-Tab assertions hold).

- [ ] 14. Area SHELL-STATUS: Status bar.
      `shell/render_status.rs` -- the version banner "FileForge Workbench v0.1.0"
      and the `RichText::new("LOG!")` logging indicator. The banner embeds the
      product name + version: keep the product name and version as NON-translated
      args and localise any surrounding prose; here the string is literally the
      product name + version with a space, so either keep it verbatim as
      `ff-desktop-shell-status-banner = FileForge Workbench v0.1.0` (byte-identical
      value, product-name-only -- acceptable as an Identity_Base value) OR build
      it from `app-name` + version. PREFER the single byte-identical value keyed
      `ff-desktop-shell-status-banner` and note that the product name stays
      English. "LOG!" is a status mnemonic indicator -- localise as prose.
      Files: `crates/ff-desktop/src/shell/render_status.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-shell-status.ftl` (prefix
      `ff-desktop-shell-status-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      status-bar tests
      (`cargo test -p ff-desktop --bin ffwb status_bar` /
      `... full_shell_status`) and confirm PASS.

- [ ] 15. Area SHELL-NAV-OPS: New/Rename/Delete nav dialogs + nav headings.
      `shell/render_nav_ops.rs` -- "Name:", "Create", "Cancel", the
      `format!("Delete '{label}'? This cannot be undone.")` -> `t_args` with
      `{ $label }`, "Delete", "Cancel", "New name:", "Rename", "Cancel".
      `shell/render_nav.rs` -- `RichText::new("File Explorer")` heading.
      `shell/render.rs` -- `RichText::new("Toolchain Panel")` heading and the
      `small_button("\u{2715}")` close glyph (keep as `{ "\u2715" }` escaped
      value if localised, or LEAVE as a glyph button and note the decision --
      PREFER leave the raw glyph, it is a UI close icon not prose).
      Files: `crates/ff-desktop/src/shell/render_nav_ops.rs`,
      `crates/ff-desktop/src/shell/render_nav.rs`,
      `crates/ff-desktop/src/shell/render.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-shell-nav.ftl` (prefix
      `ff-desktop-shell-nav-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      nav / explorer focus tests
      (`cargo test -p ff-desktop --bin ffwb render_nav` /
      `... file_explorer`) and confirm PASS.

- [ ] 16. Area SHELL-TAB-BAR: Tab bar + context-menu actions.
      `shell/render_tab_bar.rs` -- `on_hover_text("Close tab")`, and the tab
      context-menu buttons: "Close", "Close All BUT This", "Close All to the
      Left", "Close All to the Right", "Close All Unchanged", "Clone to Other
      Tab", "Move to Other View", "Pin Tab", "Exit", "Open Containing Folder in
      Explorer", "... in CMD", "... in PowerShell", "... in Terminal", "Copy Name
      to Clipboard", "Copy Path to Clipboard", "Save", "Save As", "Reload"; and
      the empty-bar context menu "New", "New File". Keep the OS/app names
      (Explorer/CMD/PowerShell/Terminal) as NON-translated args embedded in a
      localised "Open Containing Folder in { $target }" pattern, OR keep the full
      strings verbatim as byte-identical Identity_Base values and note that the
      tool names stay English -- PREFER the arg form for the four "Open Containing
      Folder in X" so the prose localises and the tool name stays English
      (Req 4.2, 9.2). Keep the `RichText::new("\u{00d7}")` close glyph as
      `{ "\u00d7" }`. Do NOT touch any `pending_tab_actions` logic.
      Files: `crates/ff-desktop/src/shell/render_tab_bar.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-shell-tab-bar.ftl` (prefix
      `ff-desktop-shell-tab-bar-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      tab-bar context-menu tests
      (`cargo test -p ff-desktop --bin ffwb tab_context_menu` /
      `... context_menu`) and confirm PASS. NOTE: these context-menu tests assert
      which ITEMS are present (universal vs file-specific) via the harness; if any
      asserts the English item TEXT, the byte-identical `.ftl` value keeps it
      green -- confirm in the log.

- [ ] 17. Area SHELL-DIALOGS: update_dialogs overlays.
      `shell/update_dialogs.rs` -- window titles "Command History", "Swap to
      Tab", "Run external program?", "Unsaved Workspace Changes"; and bodies:
      "No command history.", "Cancel", "Number:", "Cancel", "An external program
      is about to run:", "Run", "Cancel", the RESET-BARE prose
      (`format!("Profile to be reset: {}")` -> `t_args {$name}`,
      `format!("{} profiles will be reset:")` -> `t_args {$count}` number,
      `format!("    - {name}")` -> `t_args {$name}` keeping the 4 leading spaces
      with `{ "    " }`), and the unsaved-workspace dialog
      (`format!("Workspace '{}' has unsaved changes.")` -> `t_args {$name}`,
      "Save", "Discard", "Cancel"). The swap-row
      `format!("{n}: {label}")` is a data row (number + tab label) -- LEAVE it
      (pure data formatting, no translatable prose). Do NOT touch the
      `swap_list_number_input` id.
      Files: `crates/ff-desktop/src/shell/update_dialogs.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-shell-dialogs.ftl` (prefix
      `ff-desktop-shell-dialogs-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      dialog tests
      (`cargo test -p ff-desktop --bin ffwb unsaved_workspace` /
      `... swap` / `... history`) and confirm PASS.

- [ ] 18. Area ABOUT: About dialog (PARTIAL -- prose consts stay unlocalised).
      CONVERT ONLY these render sites in `about_dialog.rs`:
      - window title "About FileForge Workbench" -- keep verbatim as a single
        byte-identical value `ff-desktop-about-window-title` (it embeds only the
        product name, which stays English);
      - heading "FileForge Workbench" -- keep verbatim as
        `ff-desktop-about-heading` (product name);
      - `format!("Version {VERSION}")` -> `ff_i18n::t_args("ff-desktop-about-version",
        &ff_i18n::MessageArgs::new().set("version", VERSION))` with the `.ftl`
        value `Version { $version }` (TEXT arg; VERSION is a `&str`);
      - the "  Close  " button -> `ff-desktop-about-close` with value
        `{ "  " }Close{ "  " }` to preserve the two leading/trailing spaces.
      DO **NOT** localise the four prose CONSTANTS `CREATOR`, `AI_CREDIT`,
      `COPYRIGHT`, `DESCRIPTION`, nor reroute their `ui.label(CONST)` sites
      through `t()`, and do NOT remove or change the `pub const` definitions:
      `about_dialog.rs` has UNIT TESTS asserting their English substrings
      (`about_dialog_contains_creator_credit` -> `CREATOR.contains("Alan R Wynne")`,
      `about_dialog_ai_credit_*` -> `AI_CREDIT.contains("Amazon Q Developer")`/`"AWS"`,
      `about_dialog_copyright_contains_creator_name` -> `COPYRIGHT.contains("Alan R Wynne")`,
      `about_dialog_description_is_nonempty`). These fall squarely under the
      "model-layer string with a unit test asserting its English substring --
      LEAVE it" rule. Record this decision in the commit. (A later task may
      localise credits/copyright by moving the asserting tests to the `.ftl`
      value; that is out of scope for Task 8's behaviour-neutral extraction.)
      Files: `crates/ff-desktop/src/about_dialog.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-about.ftl` (prefix `ff-desktop-about-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      about-dialog tests (`cargo test -p ff-desktop --bin ffwb about`) and confirm
      ALL PASS (the four const-substring tests remain green because the consts are
      untouched).

- [ ] 19. Area POM-CALENDAR: Primary Option Menu calendar chrome.
      `primary_option_menu.rs` -- the calendar nav buttons `RichText::new("<")`
      and `RichText::new(">")` (navigation glyphs -- LEAVE as glyphs, note the
      decision; they are directional icons, not prose), the day-of-week header
      `RichText::new("Su Mo Tu We Th Fr Sa")` (locale-sensitive but a fixed
      English weekday header; localise as ONE value `ff-desktop-pom-weekday-header`
      keeping byte-identical spacing), and the blank/space cells
      `RichText::new("   ")` / `RichText::new(" ")` (pure layout whitespace --
      LEAVE). Sweep the whole file for any other displayed POM option prose that
      is NOT already sourced from the menu model (POM option descriptions come
      from the menu model and are ALREADY localised -- do NOT double-localise).
      Files: `crates/ff-desktop/src/primary_option_menu.rs`, new
      `crates/ff-i18n/i18n/en/ff-desktop-pom.ftl` (prefix `ff-desktop-pom-`).
      Verify: `ftl_dupe_check` clean; `cargo check -p ff-desktop` clean; run the
      POM focus/calendar tests
      (`cargo test -p ff-desktop --bin ffwb primary_option` / `... pom` /
      `... calendar`) and confirm PASS.

- [ ] 20. Final sweep + file-size + full compile-verify (no new behaviour).
      Re-grep `crates/ff-desktop/src` for any residual literal in `.label(`,
      `.button(`, `.heading(`, `.on_hover_text(`, `.hint_text(`,
      `.selectable_label(`, `RichText::new("`, `Window::new("`, and `format!`
      strings passed to a label/button that were MISSED by areas 1-19; convert or
      explicitly annotate each as intentionally-unlocalised per the "LEAVE
      UNLOCALISED" rules, recording the decision. Confirm NO source file in
      `crates/ff-desktop/src` exceeds 400 non-test lines after the swaps
      (`rg --count-matches '' <file>` style check, or inspect the files touched).
      Files: any residual site found; no new `.ftl` unless a genuinely missed
      area emerges (then add its own prefixed file).
      Verify: final `ftl_dupe_check` reports `no duplicate message ids`;
      `cargo fmt -p ff-desktop` clean; `cargo check -p ff-desktop` clean;
      `cargo clippy -p ff-desktop` clean (read each log). Leave the authoritative
      `cargo gate --build` full-suite run to the OWNER.

---

## Assumptions / open decisions recorded for the implementer

- **Focus tests are label-insensitive.** `ff-desktop`'s full-shell focus tests
  assert on `first_interior_id` / first-Tab landing, not on label text, so the
  byte-identical swap keeps them green without test edits. The ONE existing
  click-by-label test targets a Task-7 (`ff-theme-editor`) label and is out of
  scope. No test file needs editing for Task 8; if a scoped run reveals a
  label-text assertion, fix the `.ftl` VALUE to be byte-identical -- never weaken
  or edit the test.
- **Glyph buttons** (`<`, `>`, `\u{00d7}`, `\u{2715}`, mnemonic `Aa`/`W`/`.*`,
  clear `x`) are UI icons, not prose; the plan leaves them unlocalised and the
  implementer records that decision in the commit. If the reviewer rules a glyph
  IS prose, localise it with the ASCII unicode-escape placeable.
- **Product name / proper nouns** ("FileForge Workbench", "Amazon Q Developer",
  "AWS", "IBM ISPF", "File-AID", "Rust", "GCC", OS/tool names) stay English --
  either kept as a byte-identical value (when the whole string is the proper
  noun) or passed as a non-translated `t_args` argument (when prose surrounds it).
- **Pre-existing menu/POM descriptions** (Task 5, `menus.ftl` +
  `menu_workspace/i18n_keys.rs`) are NOT re-localised.
- **Framework conformance:** this is a presentation-layer literal swap only; it
  adds NO dispatch path, no navigation stack, no focus mechanism, no persistence
  format, and touches no public framework type (Req 10.1). It builds on the
  existing `ff_i18n` seam already wired into `ff-desktop`.

---

## Workflow note

This plan is executed by the existing implement-and-review loop, which stops when
`.agents/tasks/review.json` has `verdict = APPROVED`. The per-area commit
boundaries above are the implementer's unit of work; the reviewer should verify,
across the WHOLE diff at once, that (a) `ftl_dupe_check` reports no duplicate
ids, (b) every `.ftl` value is byte-identical to the replaced literal (ASCII
with escaped placeables for any non-ASCII / significant whitespace), (c) no
`egui::Id` / `id_salt` changed, and (d) `cargo check`/`clippy`/`fmt -p ff-desktop`
are clean. The whole-bundle collision check is why the review is kept whole-diff
rather than split per area.
