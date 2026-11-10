# GROUP A capture-fidelity: real values and editor snapshots as tables

The change makes the editor Context snapshots faithful. The Theme provider now reads each token's real hex from the working palette instead of writing blank strings (A1). Five tabular editors (Theme, Alias, Menu, Keys, Kinds) and Config now populate `model.tables` with the editor's real columns and current values (A2/A3), so Markdown/HTML/text/YAML snapshots render proper aligned tables. All table building reads each panel's own logical state, never egui, and reuses `ff_screen_model::Table` with no renderer change. The two behavioral fixes the task enumerates under A3 -- the Config settings-tree faithful capture and the SNAPSHOT-collapses-focused-node side-effect bug -- were landed earlier in the same stream and live on `main` (the branch base); this branch adds the tabular Config capture on top and leaves both intact.

Watch for: the Config side-effect fix and its regression test are on `main`, not in this branch's diff (confirmed present). The em-dash mojibake (`ΓÇö`) visible in `render_command_line.rs` comments is pre-existing on `main`, not introduced here, so out of scope. (both confirmed)

**Verdict**: APPROVED

## High-level view

A1 is a direct bug fix: the Theme provider replaced `Field::new(label, String::new())` with `token.get(palette).to_hex()`, reading the Theme editor's working palette (falling back to the active running palette when no working copy is loaded). The same hex feeds the new Theme table. No colour-name column was added, per the owner's drop.

A2 adds one new provider-only module, `screen_providers_tables.rs`, holding a `Table` builder per tabular editor. Each builder reads the editor's own panel state (`theme_editor_panel.working`, `alias_editor_panel.rows`, `menus_editor_panel.working`, `keys_editor_panel.rows`, `kinds_editor_panel.kind_names` + `kind_registry.effective`) and emits the editor's real columns with current values. The Menu `Command` column carries the raw option command, which can be a root verb or an alias -- matching the owner's clarification that a menu command may be either. The split keeps `screen_providers.rs` at 323 lines, under the 400-line rule.

A3 captures the real visible settings tree as a `[Setting, Value, State]` table, reusing `config_panel::visible_rows` with the live filter and collapsed map, so it respects expand/collapse and filter exactly as the panel renders them. `config_key_display_value` was widened to `pub(super)` so the table builder shares the exact display logic (staged edits marked `unsaved`) rather than duplicating it.

The Config side-effect fix is not in this diff: `render_command_line.rs` on `main` consumes the Enter on submit (`consume_key`) so a background central-panel keyboard driver cannot also read it, and `tests_config_snapshot_fidelity.rs::command_submit_does_not_collapse_focused_config_tree_node` asserts the focused node's collapse state is unchanged after a SNAPSHOT command. Both are present and intact.

The renderers already iterate `model.tables` in all four output formats (text, html, markdown, yaml), so populating tables in the providers is the whole job -- no renderer change, and the diff touches no file under `ff-screen-model`.

<details>
<summary>Issues (2)</summary>

1. **A3 behavioral fix lives on main, not this branch** (confirmed) -- The Config side-effect fix and its regression test are on the branch base, not in `main...HEAD`. Confirmed present and correct; noted so the reviewer knows where to find them. No action required.
2. **Pre-existing em-dash mojibake in render_command_line.rs** (confirmed) -- `ΓÇö` appears in comments on `main`; out of scope for this change. Optional cleanup for a future ASCII sweep. No action required.

</details>

<details>
<summary>Details</summary>

### A1 -- Theme hex values from the working palette

The former bug wrote `Field::new(token.label().to_string(), String::new())` -- every token value blank. The fix resolves the palette as `theme_editor_panel.working.as_ref().unwrap_or(&self.palette)` and writes `token.get(palette).to_hex()`. Reading the working palette first is the correct source: it reflects unsaved edits the user is looking at, consistent with the ISPF working-copy convention. The active running palette is the fallback when no working copy is loaded. `EditableToken::label()` is the setting label, as the task specifies. No colour-name column is present, matching the owner's drop.

The test `theme_screen_model_has_real_hex_values_not_blank` asserts every non-empty-label field's value `starts_with('#')`, which pins the regression (blank values would fail).

### A2 -- tables via ff_screen_model::Table, logical-model-first

Each builder constructs a `Table { caption, headers, rows }` -- the exact struct shape in `ff-screen-model/src/model.rs`, unchanged. The columns match the task:

- Theme `[Setting, Value]` with real hex.
- Alias `[Act, CE, Command, Alias, Parameters]` from `alias_editor_panel.rows`. The task wording listed "the alias editor's real columns"; the builder uses the current alias model, and the doc comment notes Description was removed under CR-CH-072 so its absence is deliberate, not an oversight.
- Menu `[Option, Command, Description, Group, On, Bar]` from `menus_editor_panel.working.options`, with the On/Bar booleans rendered `yes`/`no`. The `Command` cell is the raw option command (root verb or alias), matching the owner's clarification.
- Keys `[Key, Base, Shift, Ctrl, Alt]`, one row per edited function key with the per-modifier-layer assignments.
- Kinds `[Kind, Modelled on, Title, Menu bar, Key list]` from `kind_registry.effective(name)`.

Every builder reads panel state directly; none reads egui. This is logical-model-first per Req 2.2. Each is a small, single-purpose function well under the 40-line guideline.

The tests assert real values, not just shape: `keys_screen_model_populates_table_with_real_assignments` checks F1 Base is `HELP` and the row count is exactly 12; `kinds_screen_model_populates_table_with_real_kind_rows` checks the built-in `editor` kind row carries title `[EDITOR]`; `alias_screen_model_populates_table_with_real_saved_row` seeds a real `GG -> GSEARCH` alias and asserts the captured row carries both the surface and the canonical command plus the CE label.

### A3 -- Config settings-tree capture and the shared display logic

`config_table` rebuilds the visible tree the same way the panel does: it maps the schema entries to `TreeEntry`, then calls `visible_rows(&tree_entries, &p.filter, &p.collapsed)`. Because it reuses the panel's own `visible_rows` with the live filter and collapsed map, filter and expand/collapse are honoured by construction rather than reimplemented. Namespace rows carry `[Name]` + an `expanded`/`collapsed` State; key rows carry the current value via `config_key_display_value`, suffixed `(unsaved)` when a working edit is staged. Widening that helper to `pub(super)` avoids duplicating the staged-vs-persisted-vs-default resolution.

`config_screen_model_populates_settings_table` asserts the `[Setting, Value, State]` headers and that at least one bracketed namespace row with an `expanded`/`collapsed` State is present. The side-effect regression (`command_submit_does_not_collapse_focused_config_tree_node`, on `main`) drives the real repro: command-line focus, type `SNAPSHOT`, press Enter, and assert no namespace collapsed.

### Constraints

Single dispatch is untouched -- the tests reach every provider through `dispatch_command_string("THEME"/"KEYS"/...)`, the existing front door; no new dispatch mechanism. No egui grab anywhere. `screen_providers.rs` is 323 lines, `screen_providers_config.rs` 148, the new `screen_providers_tables.rs` 238 -- all under 400. The new test file is 301 lines; test files are exempt from the 400 rule and the ~200 split is a soft guideline, so this is acceptable though it sits above the suggested split point. Both new files are ASCII-clean. The diff adds no Cargo.toml entries, so no new deps. All new tests carry `// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a` annotations.

### Verification

Per instruction I did not re-run builds or tests. The commit messages record scoped `cargo check`/`clippy`/`fmt` clean and the relevant scoped test suites passing (`tests_editor_snapshot_tables` 7 passed, `tests_config_snapshot_fidelity` 3 passed). I independently confirmed from the tree: the `Table` struct shape matches usage, all referenced config_panel symbols exist, all four renderers already iterate `model.tables`, the diff touches no renderer file, the regression test and the `consume_key` fix are present on `main`, and the new files are ASCII-only.

</details>

<details>
<summary>File map</summary>

- `shell/mod.rs` -- registers the new `screen_providers_tables` module and the `tests_editor_snapshot_tables` test module.
- `shell/screen_providers.rs` -- A1 Theme hex values; pushes the Theme/Menu/Keys/Kinds/Alias tables into each provider's model.
- `shell/screen_providers_config.rs` -- pushes the Config settings table; `config_key_display_value` widened to `pub(super)`.
- `shell/screen_providers_tables.rs` (new, 238 lines) -- the six `Table` builders, logical-model-first.
- `shell/tests_editor_snapshot_tables.rs` (new, 301 lines) -- A1/A2/A3 provider tests.

Full diff: `git -C .worktrees/group-a-capture-fidelity diff main...HEAD`.

</details>
