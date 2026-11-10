# Implementation Plan -- Alias Workspace Kind gate (CR-NR-116)

Worktree: `c:/workspace/apps/FileForgeWorkbench/.worktrees/alias-kind-impl`
(all paths below are relative to that worktree root). Git: always
`git -C c:/workspace/apps/FileForgeWorkbench/.worktrees/alias-kind-impl`.

This gate makes the Alias Workspace Kind draw its content grid INSIDE the shared
common frame (CR-NR-115) via the shared framed renderer (CR-NR-117), with the six
cell conventions, a trailing scratch row, filter-as-status, status cells, and a
single `### Aliases` clipboard table. It builds ON the framework and changes NO
dispatch / navigation / descriptor / focus / save path. The only public-ish shape
change is two ADDITIVE fields on the ff-desktop-internal `ColumnSpec` (Req 7.27,
12.8) -- not a public framework type.

Spec sources already settled: command-configurator Req 7.14-7.28 + design 9.11
(normative fixture 9.11.1) + tasks Task 9; ispf-table-editor Req 12 + tasks group
12; screen-snapshot-scrm Req 30.32-30.40 + tasks Phase 2I. Owner decisions in
`.agents/tasks/alias-kind-gate/owner-final-decisions.md`; CE-case fix in
`.agents/tasks/alias-kind-gate/ce-case-investigation.md`.

Ordering is by dependency: the shared `ff-screen-model` framed-table bands
(step 1) and the shared `ispf_table_editor` cell helpers + scratch row (step 2)
MUST be green before the Alias provider tasks (steps 4-8) consume them. Each step
leaves the crate compiling. VERIFICATION IS COMPILE-ONLY for agents (gate-policy
/ testing execution split): scoped `cargo check -p`, `cargo clippy -p ... -- -D
warnings`, `cargo fmt -p ... -- --check`, plus quick scoped named tests; the owner
runs the full `cargo gate`. Redirect each command to a uniquely named log under
`_ignore/logs/` and read it back (verify by effect). Every touched source file
stays under 400 non-test lines; tests live in sibling `_tests` files.

---

- [ ] 1. Shared framed-table bands in `ff-screen-model` (scrm Req 30.32-30.40, Phase 2I).
      Replace the nested-box table drawing in the framed path with frame-ruled
      bands: for each `model.tables` entry draw a top rule (tee-down per column
      divider), a header band, a header/data rule (cross), one band per data row,
      and the Content_Area closing rule (tee-up), reusing the existing
      `Row::Band` / `Row::Rule` / `Row::GridTop` / `Row::GridBottom` + junction
      machinery in `framed.rs` -- NOT `text::render_table_grid`. No blank spacer
      row, no caption row inside the box (caption stays for Markdown/HTML). Column
      width = widest of header and all cells; a cell never truncates (widen).
      Cells are discrete: a cell containing ` | ` is NOT re-split; tab/CR/LF
      render as a space and are never read as the RIGHT_ALIGN marker. Coalesce
      adjacent rules (the grid's top rule MERGES with the frame-owned rule under
      the `Command ===>` line; the grid's bottom rule MERGES with the rule above
      the status band -- never two consecutive rules). ASCII fallback uses
      `+ - |`. Confirm the LEGACY `render_plain` path (model WITHOUT
      `header_lines`) still emits `render_table_grid` byte-for-byte. Confirm the
      `markdown_sections` path (markdown.rs + html.rs) already emits the fenced
      box first then the sections and nothing else; add a guard test (2I.4).
      Files: `crates/ff-screen-model/src/render/framed.rs`,
      `crates/ff-screen-model/src/render/framed_tests.rs`,
      `crates/ff-screen-model/src/render/framed_rule_tests.rs` (new/extended tests),
      and if `framed.rs` would exceed 400 non-test lines, split the band-layout
      helpers into `crates/ff-screen-model/src/render/framed_bands.rs`.
      Verify: `cargo test -p ff-screen-model render::framed` and the new
      `framed_tests` / `framed_rule_tests` names pass; `cargo clippy -p
      ff-screen-model -- -D warnings` clean; `cargo fmt -p ff-screen-model --
      --check` clean.

- [ ] 2. Shared cell helpers + additive `ColumnSpec` metadata in `ispf_table_editor`
      (ispf-table-editor Req 12.1-12.4). Add two ADDITIVE fields to `ColumnSpec`
      in `shell/ispf_table_editor/state.rs`: `cell: CellKind` (new enum `{Text,
      Combo}`, default `Text`) and `snapshot_width: usize` (default = header
      width), plus a const builder `with_snapshot(self, kind, width)`; keep
      `ColumnSpec` `Copy`. Create new pure egui-free module
      `shell/ispf_table_editor/snapshot.rs` with: `combo_cell(value, width) ->
      [value v]` (empty value -> `[ v]`) padded to width; `text_cell(value,
      width)` -> value padded to width, or a run of `_` filling width when empty;
      `sanitise_cell` replacing tab/CR/LF with a space; a cell longer than width
      is NOT truncated. `snapshot_table(columns, rows, scratch_cells) ->
      ff_screen_model::Table` (headers from column headers, one table row per host
      row applying each column's cell kind + width, then the scratch row) and
      `raw_table(columns, rows) -> Table` (RAW unpadded values, no scratch, for
      `markdown_sections`). The helpers take RAW `Vec<String>` per row from a host
      `snapshot_cells(&Row)`; they have NO hint-text parameter. Register the new
      module in `shell/ispf_table_editor/mod.rs`.
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/state.rs`,
      `crates/ff-desktop/src/shell/ispf_table_editor/snapshot.rs` (new),
      `crates/ff-desktop/src/shell/ispf_table_editor/mod.rs`.
      Verify: `cargo test -p ff-desktop ispf_table_editor::snapshot` and the new
      `column_spec` snapshot-metadata test pass; clippy/fmt scoped clean.

- [ ] 3. Shared trailing scratch row in `render_ispf_table` (ispf-table-editor
      Req 12.5-12.6). Change the data-row loop in
      `shell/ispf_table_editor/render.rs` so that AFTER the last displayed data
      row it ALWAYS renders one empty insertable scratch row built from
      `blank_scratch()` (today the scratch row renders only when `rows.is_empty()`).
      Typing in the scratch row promotes it into `rows` (appended after the last
      backing row) exactly as the empty-list scratch does, then a fresh scratch
      appears next frame. The scratch row stays present while a filter is active,
      is the LAST focus stop of the field matrix, and does NOT change
      `first_interior_id` (the filter row stays the first stop). Preserve the
      empty-list behaviour identically. This is a shared behaviour change every
      table editor inherits (Menus/Kinds), so it lives in the shared surface, not
      Alias.
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/render.rs` (and its
      sibling tests file if present).
      Verify: `cargo test -p ff-desktop` scoped egui_kittest scratch-row test
      (non-empty list shows trailing scratch; typing promotes it; a new scratch
      appears; untouched scratch not persisted; first-Tab target unchanged) passes;
      clippy/fmt scoped clean.

- [ ] 4. CE uppercase display fix -- three render sites (ce-case-investigation;
      Req 7.19 keeps original stored case but the canonical CE identifier IS
      uppercase, and decision a/e renders `FFCMD`/`FFEDIT`). Change the DISPLAY
      from `AliasEnvironment::label()` (lowercase) to `env_name()` (uppercase) at:
      (a) the editor CE combo selected-text + options in
      `alias_editor_panel/rows.rs::render_ce_combo`; (b) the filter CE combo shown
      text in `alias_editor_panel/host.rs::render_filter_cells` (keep the match
      `eq_ignore_ascii_case`, which is already case-insensitive); (c) the snapshot
      CE cell source used by the row-to-cells mapping (step 5) and `alias_table`
      (step 6). Do NOT change storage, file names, dropdown constraints, or
      `from_env_name`. NOTE: the design fixture 9.11.1 and Task 9 test wording
      show lowercase `[ffcmd v]`; the owner's FINAL decision (owner-final-decisions
      + ce-case-investigation) overrides to UPPERCASE. The implementer MUST render
      `FFCMD`/`FFEDIT` and regenerate the goldens/fixtures to uppercase
      accordingly (see step 7). This is the one place the plan supersedes the
      fixture's literal casing.
      Files: `crates/ff-desktop/src/alias_editor_panel/rows.rs`,
      `crates/ff-desktop/src/alias_editor_panel/host.rs`.
      Verify: `cargo check -p ff-desktop` clean; the `render_ce_combo` /
      filter-cell change compiles; a scoped unit test asserting the CE cell text
      is `FFCMD` passes (added in step 5/6).

- [ ] 5. Alias column cell metadata + row-to-cells mapping (Req 7.18, 7.19, 7.20,
      7.23; Task 9.1-9.2). In `alias_editor_panel/host.rs` extend `ALIAS_COLUMNS`
      with the shared `with_snapshot` builder so each column declares cell kind +
      snapshot width, in order: Act (Text, 4), CE (Combo, 10), Command (Combo, 20),
      Alias (Text, 16), Parameters (Text, 28), Description (Text, 32); headers stay
      plain text. Add `alias_snapshot_cells(&AliasRowEdit) -> Vec<String>`
      returning the six RAW values in order: `act`, CE from `environment.env_name()`
      (UPPERCASE per step 4), `canonical`, `surface`, `command_template`,
      `description`. Placeholder hint text (`_`, `name`, `(&1)`, `note`) is NEVER a
      value; an empty field stays empty (the shared `text_cell`/`combo_cell`
      renders it as `____` / `[ v]`); a typed `&1` appears; control characters
      become a space; a Description containing ` | ` stays one value.
      Files: `crates/ff-desktop/src/alias_editor_panel/host.rs` (+ sibling tests).
      Verify: `cargo test -p ff-desktop alias_snapshot_cells` and the
      `ALIAS_COLUMNS` metadata test pass; clippy/fmt scoped clean.

- [ ] 6. Rebuild `alias_table()` via the shared builder (Req 7.19, 7.21, 7.22;
      Task 9.3). In `shell/screen_providers_tables.rs::alias_table()` build the
      displayed set with `displayed_indices(&p.rows, p.filter_ce, &p.filter_root)`
      in BACKING order, map each row through `alias_snapshot_cells`, append the
      scratch row cells from `alias_snapshot_cells(&AliasRowEdit::default())`, and
      call the shared `snapshot_table(ALIAS_COLUMNS, displayed_cells, scratch_cells)`.
      Keep `caption = Some("Aliases")` for the structured flavors. The filter row
      is NEVER a table row. An empty working copy yields only the scratch row. Add
      a sibling `alias_markdown_section()` building a `MarkdownSection` with
      heading `Some("Aliases")` and the `raw_table(ALIAS_COLUMNS, displayed_cells)`
      (RAW values, no `[v]`/underscore/padding, no scratch row).
      Files: `crates/ff-desktop/src/shell/screen_providers_tables.rs` (+ tests).
      Verify: `cargo test -p ff-desktop alias_table` passes (rows = displayed set
      in backing order; scratch row last = default mapped through conventions;
      no filter row; empty copy = scratch only); clippy/fmt scoped clean.

- [ ] 7. Single representation, message routing, clipboard section (Req 7.14,
      7.15, 7.16, 7.24; Task 9.4-9.5). In
      `shell/screen_providers.rs::alias_editor_screen_model()` DELETE the `fields`
      loop (removes the `SS . . . : SNAPSHOT` dot-leader line and the
      `| Property | Value |` clipboard table) and the self-set `command_line`
      (the common frame owns it). Push exactly one `alias_table()` into
      `model.tables` and one `alias_markdown_section()` into
      `model.markdown_sections`. Keep the `p.error -> model.messages` mapping (the
      error shows in the Message_Line band, not as a content row). The model must
      carry NO `header_lines`/`command_line` of its own (added later by
      `apply_common_frame`).
      Files: `crates/ff-desktop/src/shell/screen_providers.rs` (+ tests).
      Verify: `cargo test -p ff-desktop alias_editor_screen_model` passes (`fields`
      empty; exactly one table; one markdown_section; error in messages); a
      clipboard test asserts the Markdown render contains the fenced box FIRST,
      then `### Aliases` once, and NO `| Property | Value |` table (and HTML
      mirrors it); clippy/fmt scoped clean.

- [ ] 8. Workspace status cells + i18n (Req 7.25, 7.26; Task 9.7). Create
      `shell/workspace_status_alias.rs` (<400 lines) with
      `alias_status_cells(&self) -> Vec<String>` returning, in order:
      `Aliases: <n>` where n = `alias_editor_panel.rows.len()` (scratch excluded);
      `Rows 1-<m> visible` where m = `displayed_indices(..).len()` (`Rows 0
      visible` when none); and ONLY while a filter is active, `Filter: <ce>/<root>`
      with `--` for an unset half (CE rendered UPPERCASE to match step 4, e.g.
      `Filter: FFCMD/--`). No cell contains the band separator ` | `. Use `ff_i18n`
      keys following the `ff-desktop-shell-status-*` pattern (English `Aliases: {
      $count }`, `Rows 1-{ $shown } visible`, `Rows 0 visible`, `Filter: { $ce }/{
      $root }`). Add a `KindTag::AliasEditor` arm to
      `render_status.rs::workspace_status_cells()` BEFORE the FileEditor/Untitled
      early return, returning `self.alias_status_cells()`. Register the new module
      where the shell's `mod` list lives. Live and snapshot read the ONE method, so
      parity is by construction; the common `Last | Prof | Zoom` precede the Alias
      cells (already composed by `common_frame.rs`).
      Files: `crates/ff-desktop/src/shell/workspace_status_alias.rs` (new),
      `crates/ff-desktop/src/shell/render_status.rs`, the shell `mod` declaration
      file (`shell/mod.rs` or equivalent), the `ff_i18n` catalogue (locate the
      existing `ff-desktop-shell-status-*` `.ftl`).
      Verify: `cargo test -p ff-desktop` status-cell tests pass (2-row copy =>
      `["Aliases: 2","Rows 1-2 visible"]`; 1-of-40 filtered => `Aliases: 40`,
      `Rows 1-1 visible`, `Filter: FFCMD/SNAPSHOT`; unset half => `FFCMD/--`;
      nothing matches => `Rows 0 visible`; no cell has ` | `; no `Filter:` cell
      when inactive) and the live/snapshot parity test; clippy/fmt scoped clean.

- [ ] 9. Goldens: regenerate and add (Req 7.17-7.22, 7.25, 7.28; Task 9.6).
      Regenerate the three existing Alias goldens to the new frame-ruled layout
      (grid merged into the frame rules, UPPERCASE `[FFCMD v]`, six columns with
      conventions, trailing scratch row, Alias status cells appended after
      `Last | Prof | Zoom`, no dot-leader line, no nested box, no caption row):
      `crates/ff-desktop/src/shell/golden/alias_pfshow_on.txt`,
      `alias_message_pfshow_off.txt`, `alias_message_pfshow_off_ascii.txt`. Add a
      new `golden/alias_pfshow_off_rows.txt` built to the design 9.11.1 layout but
      UPPERCASE CE, with the working copy: row 1 = the `X -> EXIT` FFCMD seed row
      (`X` / FFCMD / `EXIT` / empty Parameters / Description "Exit the
      application"); row 2 = `SS` / FFCMD / `SNAPSHOT` / empty / empty; row 3 =
      `CS` / FFCMD / `CAPTURE START` / Parameters `&1` / Description "Session
      Recording start"; then the trailing scratch row; status cells `Aliases: 3`,
      `Rows 1-3 visible`. Add a filtered golden `alias_filtered.txt` (filter
      `FFCMD/SNAPSHOT` shows only the matching row + scratch row, status band ends
      `Filter: FFCMD/SNAPSHOT`) and an ASCII variant. Write a diff-friendly
      full-text golden test (`tests_alias_golden.rs`, sibling of
      `tests_common_frame_golden.rs`) building the shell with the three-row copy
      above, taking the SNAPSHOT text, and asserting equality with the golden; in
      the same group assert the seed row is the first data row and is editable, the
      `SS` empty cells render as underscore runs (never `(&1)`/`note`), and there
      is no blank/caption row around the grid. Use the three-row copy as a direct
      in-memory working copy (set `alias_editor_panel.rows`), independent of the
      live seed set.
      Files: `crates/ff-desktop/src/shell/golden/alias_pfshow_on.txt`,
      `.../alias_message_pfshow_off.txt`, `.../alias_message_pfshow_off_ascii.txt`,
      `.../alias_pfshow_off_rows.txt` (new), `.../alias_filtered.txt` (new) + its
      ASCII variant, `crates/ff-desktop/src/shell/tests_alias_golden.rs` (new) and
      its `mod` registration.
      Verify: `cargo test -p ff-desktop tests_alias_golden` passes; each
      regenerated golden diff is reviewed against design 9.11.1 (CE uppercased).

- [ ] 10. Update existing tests to the new conventions (Req 7.28; Task 9.9).
      Update assertions that encode the OLD shape or lowercase CE, each a
      convention update (never a weakened assertion): in
      `tests_editor_snapshot_tables.rs` the alias test now expects `FFCMD` (not
      `ffcmd`) and the six-column conventions; in `tests_screen_directive.rs`,
      `tests_common_frame_snapshot.rs`, `tests_common_frame_order.rs`,
      `tests_common_frame_helper.rs` update any alias-grid / dot-leader / CE-case
      expectations to the framed-band + uppercase layout. If any existing test
      appears genuinely wrong (not just stale), report it rather than editing it.
      Files: `crates/ff-desktop/src/shell/tests_editor_snapshot_tables.rs`,
      `.../tests_screen_directive.rs`, `.../tests_common_frame_snapshot.rs`,
      `.../tests_common_frame_order.rs`, `.../tests_common_frame_helper.rs`.
      Verify: `cargo test -p ff-desktop` scoped alias/common-frame test names pass;
      clippy/fmt scoped clean.

- [ ] 11. Compile-only verification, size check, handoff (Req 7.14, 7.27, 7.28;
      Task 9.8, 9.10). Confirm the existing mandatory first-Tab test still lands on
      the filter CE combo (unchanged; no phantom stop). Run the scoped trio for
      both `ff-desktop` and `ff-screen-model` (`cargo check -p`, `cargo clippy -p
      ... -- -D warnings`, `cargo fmt -p ... -- --check`), each to a uniquely named
      `_ignore/logs/` file, and read the logs back. Confirm no touched source file
      exceeds 400 non-test lines and that no common-frame element is drawn by the
      Alias provider or panel. Update `docs/quality/TCR.md` rows for Req 7.14-7.28.
      Hand off for the OWNER's full `cargo gate` before any merge to main.
      Files: `docs/quality/TCR.md` (if present).
      Verify: the scoped trio logs for both crates are clean; the first-Tab test
      passes; a line-count check (e.g. per rust-ffwb-layout) shows every touched
      `.rs` under 400 non-test lines.

---

## Digest

### `crates/ff-screen-model/src/render/framed.rs` (step 1)
- `fn collect_rows(model, g) -> Vec<Row>`: today does
  `for table in &model.tables { rows.push(Row::Text("")); caption; grid =
  render_table_grid(..); rows.extend(grid.lines()...) }`. CHANGE: emit each table
  as `Row::Rule`/`GridTop` + `Row::Band(headers)` + `Row::Rule`(cross) +
  `Row::Band(row)` per data row, no blank spacer, no caption inside the box; let
  the existing `merge_top_rule` / `push_bottom_bands` coalesce rules.
- Existing machinery to reuse: `enum Row { Text, Centered, Command, Band, Rule,
  GridTop, GridBottom }`, `fn compose_band`, `fn push_rule`, `fn lay_row`, `fn
  merge_top_rule`, `fn push_bottom_bands`, `fn grid_rule_cuts`, const
  `CONTENT_START=2`, `RIGHT_ALIGN='\t'`, `CELL_SEPARATOR=" | "`.
- Tests today in `framed.rs` (`framed_layout_orders_rows_and_keeps_everything_inside_the_box`,
  `framed_layout_omits_message_and_key_rows_when_absent`) PASS; the new band tests
  go in `render/framed_tests.rs` / `render/framed_rule_tests.rs`.
- Verified today: `cargo test -p ff-screen-model` is the scoped runner; the crate
  builds (not run here -- compile-only per execution split; agent runs it in TDD).
- Line count: `framed.rs` ~410 physical lines incl. tests; adding band layout may
  push the non-test portion toward 400 -- split helpers into `framed_bands.rs` if so.

### `crates/ff-screen-model/src/render/text.rs` (step 1, legacy guard)
- `pub(super) fn render_table_grid(out, table, glyphs)` and `fn render_plain`:
  UNCHANGED. The legacy (no `header_lines`) path must still call
  `render_table_grid` byte-for-byte. Tests `box_draw_table_grid_*` cover it.

### `crates/ff-screen-model/src/render/markdown.rs` + `html.rs` (step 1, 2I.4)
- `render_markdown`: already emits `push_fenced_body` then
  `render_markdown_sections` and returns early when `markdown_sections` is
  non-empty (tests `markdown_sections_render_box_then_tables`,
  `no_sections_no_tables_renders_box_only`). No change beyond a guard test; confirm
  `html.rs` mirrors it.

### `crates/ff-desktop/src/shell/ispf_table_editor/state.rs` (step 2)
- `pub struct ColumnSpec { id, header, filterable, compare_mode, sortable }` with
  const builders `display`, `filterable`, `with_sortable`. ADD `cell: CellKind`
  and `snapshot_width: usize`, a new `pub enum CellKind { Text, Combo }` (default
  `Text`), and `pub const fn with_snapshot(self, kind, width) -> Self`. Keep
  `Copy`. Existing test `column_spec_carries_filter_and_sort_flags` stays; add a
  snapshot-metadata test.

### `crates/ff-desktop/src/shell/ispf_table_editor/snapshot.rs` (new, step 2)
- `combo_cell(&str, usize) -> String`, `text_cell(&str, usize) -> String`,
  `sanitise_cell(&str) -> String`, `snapshot_table(&[ColumnSpec],
  &[Vec<String>], &[String]) -> ff_screen_model::Table`, `raw_table(&[ColumnSpec],
  &[Vec<String>]) -> Table`. Pure, egui-free. Register in `mod.rs`.

### `crates/ff-desktop/src/shell/ispf_table_editor/render.rs` (step 3)
- `pub fn render_ispf_table<H, FScratch>(...)`: the data-row `ScrollArea` closure
  currently does `if rows.is_empty() { scratch } else { for &backing in displayed
  { render_row_cells } }`. CHANGE: always render data rows for `displayed`, THEN
  always render one trailing scratch row (same promote-on-edit logic as the
  empty-list branch), pushing its cell ids LAST into `field_matrix`. Keep
  `anchors.first` = filter cell. `RowCells`, `TableAction`, `apply_arrow_walk`
  unchanged.

### `crates/ff-desktop/src/alias_editor_panel/rows.rs` (step 4)
- `render_ce_combo`: `selected_text(env.label())` + `selectable_value(env, choice,
  choice.label())` -> use `env.env_name()` / `choice.env_name()`.

### `crates/ff-desktop/src/alias_editor_panel/host.rs` (steps 4, 5)
- `render_filter_cells`: `let shown = match self.filter_ce { Some(env) =>
  env.label(), None => "--" }` -> `env.env_name()`. Filter MATCH in
  `filter_matches` stays `eq_ignore_ascii_case` (works for either case).
- `const ALIAS_COLUMNS: [ColumnSpec; 6]`: add `.with_snapshot(CellKind::Text, 4)`
  etc. per step 5 widths. ADD `pub(super) fn alias_snapshot_cells(&AliasRowEdit)
  -> Vec<String>` (six RAW values; CE = `env_name()`).

### `crates/ff-desktop/src/shell/screen_providers_tables.rs` (step 6)
- `pub(super) fn alias_table(&self) -> Table`: today maps ALL `p.rows` through
  `r.environment.label()` with raw strings and no scratch. REBUILD via
  `displayed_indices` + `alias_snapshot_cells` + scratch + shared
  `snapshot_table`. ADD `alias_markdown_section` using `raw_table`.

### `crates/ff-desktop/src/shell/screen_providers.rs` (step 7)
- `pub(super) fn alias_editor_screen_model(&self) -> ScreenModel`: DELETE the
  `for row in &p.rows { model.with_field(Field::new(...)) }` loop and the
  `model.command_line = self.command_text.clone()` line; push `alias_table()` and
  `alias_markdown_section()`; keep `p.error -> model.messages`.

### `crates/ff-desktop/src/shell/workspace_status_alias.rs` (new, step 8)
- `pub(super) fn alias_status_cells(&self) -> Vec<String>`. Reads
  `self.alias_editor_panel.rows`, `filter_ce`, `filter_root` and
  `displayed_indices`. i18n keys `ff-desktop-shell-status-aliases-*`.

### `crates/ff-desktop/src/shell/render_status.rs` (step 8)
- `pub(super) fn workspace_status_cells(&self)`: add
  `if matches!(tab.kind.tag(), KindTag::AliasEditor) { return
  self.alias_status_cells(); }` before the current FileEditor/Untitled early
  return. Common `Last | Prof | Zoom` are prepended by `common_frame.rs`
  (`common_frame_status_bar_text`) -- unchanged.

### Goldens + tests (steps 9, 10)
- `shell/golden/alias_message_pfshow_off.txt` TODAY shows the broken shape
  (dot-leader `SS . . . : SNAPSHOT`, nested `Aliases` box, lowercase `ffcmd`,
  blank cells, no scratch, `Zoom: 0` only). Regenerate to the framed-band +
  UPPERCASE layout. New `alias_pfshow_off_rows.txt`, `alias_filtered.txt` (+ ASCII)
  and `tests_alias_golden.rs`. Update alias/CE-case assertions in
  `tests_editor_snapshot_tables.rs` (`alias_screen_model_populates_table_with_real_saved_row`
  currently asserts `ffcmd` -> `FFCMD` and the cell conventions),
  `tests_screen_directive.rs`, `tests_common_frame_{snapshot,order,helper}.rs`.

### Verification commands (discovered; used by every step)
- Build/check: `cargo check -p ff-desktop` / `cargo check -p ff-screen-model`.
- Lint: `cargo clippy -p <crate> -- -D warnings`. Format: `cargo fmt -p <crate>
  -- --check`.
- Scoped tests: `cargo test -p <crate> <name>` (quick, by name). Agents do NOT run
  the whole suite / `cargo nextest` / `cargo gate`; the owner runs the full gate.
- Redirect each to a unique `_ignore/logs/<name>-<pid>.log` and read it back.

### STOP items (surface to owner, do not self-resolve)
- None required by this plan. Two notes the implementer MUST honour, not deviate
  from: (1) CE renders UPPERCASE `FFCMD`/`FFEDIT` per the owner's FINAL decision,
  which SUPERSEDES the lowercase `[ffcmd v]` literal in design fixture 9.11.1 /
  Task 9 wording -- regenerate goldens uppercase. (2) This gate does NOT change the
  seed set (the SS/CS/CT Option-B seed expansion is a separate follow-up gate);
  fixtures use the CURRENT seed (`X -> EXIT`) plus `SS`/`CS` as in-memory working
  rows. If either turns out to require a public `ff-screen-model` / `WorkspaceContext`
  reshape (it should not -- `ColumnSpec` fields are ff-desktop-internal), STOP and
  flag it as a framework change.
