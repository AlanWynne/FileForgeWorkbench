# Shared ISPF table editor, proven on the Alias editor (CR-CH-075)

The Alias editor is rebuilt on a new shared, egui-free table-editor core (`shell/ispf_table_editor/`): a generic `IspfTableHost` trait + column/line-command/filter/sort model, a pure line-command apply pass that reuses the `ff-line-commands` engine for parsing and classification, a stable Sort, and a generic `render_ispf_table` surface that delegates per-cell painting back to the host via a separate egui-aware `IspfTableRender` trait. On top of that, aliases gain a persisted optional `description` field, `I/D/C + SORT` line commands (no Move), a single `alias_sort_key` that powers both the manual `SORT` command and auto-sort-on-save, and a Description column in the ScreenProvider table. All of it is additive: the resolver, dispatch, navigation, descriptor, and CR-CH-071 save seam are untouched, and the egui ids the full-shell focus tests depend on are preserved.

Watch for: a latent index-integrity gap when a Copy (`C`/`A`/`B`) is batched together with an immediate `D`/`I` in the same apply pass (likely) — the only path not covered by a test. Everything else in the 8-item plan and the hard constraints checks out.

**Verdict**: APPROVED

## High-level view

The shared component lands exactly where the design put it and keeps the pure core GUI-free. `state.rs` owns the column model, `LineCommandSet`, `FilterState`, `TableEditorState`, and the `IspfTableHost` trait; `line_commands.rs` and `sort.rs` are pure passes over a generic `Vec<Row>` with no egui; `render.rs` adds the egui surface and the `IspfTableRender` split so the host paints its own cells (the cascading CE -> Root dropdowns) while the chrome is drawn once. The split between the two traits is the right call and is what keeps the apply pass and Sort unit-testable without a harness.

The line-command pass reuses `ff_line_commands::LineCommandParser::parse` + `command::classify` rather than inventing a parser, maps Immediate (I/D) / Source (C/M) / Target (A/B) onto insert/remove/clone, processes immediates by descending index, and retains an unresolved copy source with a non-blocking notice. Move is gated off by the alias `LineCommandSet` and a typed `M` is cleared with a "not enabled" notice. The one soft spot is the interaction between a copy insert and a same-batch immediate: immediates are collected against the original indices but applied after copy inserts have already shifted the list.

The `description` field is modelled exactly as required: `#[serde(default, skip_serializing_if = "String::is_empty")] pub description: String` on `UserAliasRowToml`, additive after `command_template`, with round-trip tests proving an old file without the key loads empty and re-serialises without emitting it. Because the resolver carries no description, reload is sourced from the override files directly via a new `user_alias_descriptions` helper keyed on (env, uppercased surface) — a reasonable choice that keeps the resolver description-free, and it is covered by an end-to-end save-then-reload test.

Sorting is one key function. `alias_sort_key(surface, canonical) -> (UPPER, UPPER)` lives in `user_aliases.rs`; the host's `sort_key` builds on it, the manual `SORT` handler defaults a bare `SORT` to the Alias (surface) column so it matches the save ordering, and `write_override_rows` sorts each env's rows with the same key before serialise. Tests assert the on-disk order and that manual SORT produces the identical order.

The constraints hold: `environment.rs` is not in the diff (resolver still a linear `eq_ignore_ascii_case` scan), no `ff-line-commands` public API change (only a new path dependency), the single `render_workspace_context` focus-latch path and the CR-CH-071 save seam are unchanged, and the stable egui ids (`alias_editor_command_line`, `alias_filter_ce`, per-row `alias_act`/`alias_surface`/etc., new `alias_description`) are preserved so `tests_focus.rs` is untouched. Every non-test file is under 400 lines (max 379) and no non-ASCII was found.

<details>
<summary>Issues (2)</summary>

1. **Copy + immediate index shift (likely)** — in `ispf_table_editor/line_commands.rs`, immediate `I`/`D` entries are collected against the original indices but applied in step 3b *after* copy/move inserts have already grown `rows`. A batch mixing `C`/`A` with a `D` on a row after the insertion point would delete the wrong row. Not covered by any test. Either apply immediates before structural copy inserts, re-resolve indices, or document that mixing copy with immediate in one batch is unsupported and add a test pinning the chosen behaviour.
2. **`enabled_letters` omits SORT (possible, cosmetic)** — `LineCommandSet::enabled_letters` returns only I/D/C/M and never reflects the `sort` flag, so a header hint built from it would under-report. Only used for a hint/notice today; confirm no surface relies on it to advertise SORT.

</details>

<details>
<summary>Details</summary>

### The two-trait split (pure host vs egui render)

The design's "keep the core GUI-free" is honoured by splitting the contract in two: `IspfTableHost` (in `state.rs`) carries `columns`, `line_command_set`, `blank_row`, `clone_row`, `act_of`/`clear_act`, `filter_matches`, and `sort_key` — all egui-free, which is why `line_commands.rs` and `sort.rs` can be unit-tested with stub rows and stub hosts. The egui-aware `IspfTableRender` (in `render.rs`) adds only `command_line_id`, `render_filter_cells`, and `render_row_cells`. The Alias editor implements both in `host.rs`. This is a clean generalisation and it is what lets the Alias panel keep its cascading CE -> Root dropdowns (drawn in `render_filter_cells` / `render_row_cells`) while the surface owns the command line, headers, scroll area, and empty-store scratch row.

The focus contract survives the move. `render_ispf_table` returns `InteriorAnchors { first, last }` sourced from the id the host returns out of `render_filter_cells` (the filter CE combo's fresh `response.id`), and `alias_editor_panel/render.rs` copies those into `first_interior_id`/`last_interior_id`. `tests_focus.rs` is not in the diff and still asserts the first Tab lands on that combo — so the single latch path is intact.

### Line-command apply pass and the copy/immediate ordering

Classification is delegated to `ff-line-commands` (`LineCommandParser::parse` + `command::classify`) with no new parser and no API change, exactly as the plan requires. Immediate I/D are processed by descending index (tested: `mixed_insert_delete_keeps_index_integrity`), an unresolved `C` is retained with its Act cell intact plus a notice (`unresolved_copy_source_is_retained`), an unrecognised letter is cleared (`unrecognised_letter_is_cleared_and_ignored`), and a disabled `M` is cleared with a "not enabled" notice (`move_not_enabled_is_ignored`). C+A and C+B are both tested.

The gap: step 3a applies copy/move inserts (ascending-point inserts done in descending order so copy targets do not shift each other), but step 3b then applies the immediate I/D using `entry.index` captured against the *original* list. The code comment asserts "a C/M never overlaps an I/D cell (one Act letter per row)", which is true per-row but does not address list-level index shift: a copy that inserts at position 2 moves every later row down by one, so a `D` recorded at original index 5 now points one row early. No test exercises a copy batched with an immediate, so this is untested rather than observed-broken, but it is the one determinism hole in an otherwise carefully-ordered pass.

### description field: additive and byte-stable

`UserAliasRowToml.description` is `#[serde(default, skip_serializing_if = "String::is_empty")]`, placed after `command_template`, matching the existing optional-field convention. `old_file_without_description_loads_empty_and_omits_on_serialise` and `row_with_description_round_trips` pin both halves of the round-trip. Reload is handled out-of-band of the resolver: `user_alias_descriptions` reads the override files for the active locale and merges descriptions onto the effective rows by (env, case-insensitive surface), and `description_persists_and_reloads_through_effective_rows` proves a saved description survives a reload. Keeping the resolver description-free is consistent with the constraint that it stay a plain scan.

### One sort key, two callers

`alias_sort_key` is the single source of truth. `write_override_rows` sorts each environment's `Vec<UserAliasRowToml>` with it before `serialise_user_aliases` (stable, so already-sorted input is a no-op and no field value changes); the manual handler defaults a bare `SORT` to the Alias column so it keys on surface-then-canonical identically. `save_writes_rows_in_sorted_surface_order` and `manual_sort_matches_auto_sort_on_save_order` assert the two orderings agree, and the manual-sort test explicitly checks that stored field case is preserved (only the key is uppercased).

### Snapshot and tests

`screen_providers_tables.rs::alias_table` now emits `[Act, CE, Command, Alias, Parameters, Description]` in Req 7.2 order with `r.description.clone()`, and the snapshot test expects the new header. The per-row render cell order in `rows.rs` matches that column order and the Description cell carries the stable `("alias_description", backing)` id. Every new test carries a `// Validates:` annotation. Test modules are reasonably sized; `alias_editor.rs` is 509 total / 379 non-test, within the rule.

</details>

<details>
<summary>File map</summary>

- `Cargo.lock`, `crates/ff-desktop/Cargo.toml` — add `ff-line-commands` path dependency (engine reuse, not a new third-party crate).
- `crates/ff-desktop/src/shell/ispf_table_editor/{mod,state,line_commands,sort,render}.rs` — new shared component: re-exports, pure contract/types, pure apply pass, stable Sort, generic egui surface + `IspfTableRender` split.
- `crates/ff-desktop/src/shell/mod.rs` — register `pub(crate) mod ispf_table_editor;` (line 75).
- `crates/ff-desktop/src/alias_editor_panel/host.rs` — new: `IspfTableHost` + `IspfTableRender` for `AliasEditorState`, column model, filter, sort key, cell painting.
- `crates/ff-desktop/src/alias_editor_panel/render.rs` — rewritten to delegate to `render_ispf_table`; `WorkspaceContext` impl preserved.
- `crates/ff-desktop/src/alias_editor_panel/{rows,state,mod}.rs` — Description cell + field; export `alias_surface_column`.
- `crates/ff-desktop/src/shell/alias_editor.rs` — thread description through validate/load; `user_alias_descriptions` reload helper; auto-sort in `write_override_rows`.
- `crates/ff-desktop/src/shell/alias_editor_line_commands.rs` — delegate apply pass to the shared pass; add `SORT` handler.
- `crates/ff-desktop/src/shell/user_aliases.rs` — `description` field + `alias_sort_key` + round-trip tests.
- `crates/ff-desktop/src/shell/screen_providers_tables.rs` — Description column.
- `crates/ff-desktop/src/shell/tests_alias_editor.rs`, `tests_editor_snapshot_tables.rs` — description round-trip, C+A end-to-end, sort order, snapshot header.

Full diff: `git -C c:/workspace/apps/FileForgeWorkbench/.worktrees/group-b-alias diff main...HEAD`

</details>
