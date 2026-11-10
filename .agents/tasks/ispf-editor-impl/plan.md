# Implementation Plan -- ISPF Table Editor: Block commands (CR-CH-076), Single-chrome (CR-CH-077), Arrow navigation (CR-CH-078)

Worktree (all paths relative to it; use `git -C` with this root for any git):
`c:/workspace/apps/FileForgeWorkbench/.worktrees/ispf-editor-impl`

TDD per `.kiro/steering/testing.md`: the FAILING test (with its `// Validates:`
annotation) is written and seen to fail BEFORE the implementation that makes it
pass. Agents compile-verify SCOPED only: `cargo check -p ff-desktop`,
`cargo clippy -p ff-desktop -- -D warnings`, `cargo fmt -p ff-desktop -- --check`,
plus the QUICK scoped unit tests named below (`cargo test -p ff-desktop <name>`).
The OWNER runs the full `cargo gate` / whole suite (gate-policy.md). No source
`.rs` over 400 non-test lines; split a test module before ~200 lines. ASCII-only.

## Design decisions (made here, grounded in the code I read)

1. **Blocks reuse the engine with NO public-API change.** The adapter in
   `shell/ispf_table_editor/line_commands.rs` will, when blocks are enabled,
   build a local `ff_line_commands::PendingCommandStore`, `add` each parsed
   `*Block` `ParsedLineCommand` (with `PendingReason::AwaitingPair`) keyed by the
   row's BACKING index as the `line`, then pair via
   `BlockPairValidator::validate_pair(&store, BlockCommandKind)` and
   `check_overlaps`. These are the exact public items in
   `crates/ff-line-commands/src/{block_pair.rs,command.rs,pending.rs}` I read;
   nothing in `ff-line-commands` changes. `BlockPairValidator::validate_pair`
   already returns `AwaitingPair`/`TooManyMarkers` for lone/extra markers and
   normalises the inclusive range order-independently.

2. **Block opt-in is a single `blocks: bool` on `LineCommandSet`.** The brief
   permits a flag or RR/DD/CC/MM sub-flags; one `blocks` flag is sufficient
   because the enabled block set is fixed to exactly RR/DD/CC/MM (Req 9.1) and no
   host needs a subset. Alias sets `blocks: true` (Req 7.11); MM stays
   unreachable because `line_command_set().move_` is `false` and the adapter
   requires `move_` for a MoveBlock (so MM is disabled on Alias per Req 7.12
   while RR/DD/CC work).

3. **Block translation reuses the existing stable-original-index rebuild.** The
   adapter already rebuilds via per-row `before`/`after`/`removed` vectors over
   stable original indices. A resolved `BlockPair{start,end}` maps to:
   `DeleteBlock` -> `removed[start..=end]`; `RepeatBlock` -> clone each row in
   `start..=end` into its own `after[i]`; `CopyBlock`+A/B -> push clones of
   `start..=end` to the target's `after`/`before`; `MoveBlock`+A/B -> same as
   CopyBlock plus `removed[start..=end]`. This composes with single commands in
   one batch with no new machinery.

4. **Arrow walk: a pure target computation + a render-time consume-and-focus.**
   The pure `(focused row, col, direction) -> clamped (row, col)` function lives
   in a new `shell/ispf_table_editor/arrow_nav.rs` (keeps `render.rs`/`state.rs`
   under 400 lines and makes the clamp table-testable without egui). The render
   surface builds a field-matrix `Vec<Vec<egui::Id>>` as it lays out cells, then
   AFTER layout: read `ui.ctx().memory(|m| m.focused())`, if it is a matrix cell
   (never the command line), CONSUME the pressed arrow with
   `ui.input_mut(|i| i.consume_key(Modifiers::NONE, key))` and `request_focus`
   the target id. This is the render_menu_bar/render_chrome consume-before-egui
   pattern (Req 11.3). Tab/Shift+Tab and all stable ids are untouched (Req 11.4).

5. **CR-CH-077 is a REMOVE + a snapshot FILL, both host-side.** The shared
   surface already draws exactly one `Command ===>` line and one header
   (`render.rs`). Whatever second Title/heading or command line the live Alias
   panel shows on top is removed from the host render arm (to be pinpointed
   during task 10.2 by reading the shell Title_Line arm + the surface's own
   `title` label). The snapshot side is fixed by making
   `alias_editor_screen_model` emit the `Command ===>` line and the column header
   row so the captured model agrees with the live surface.

6. **Keep everything in the SHARED component** (adapter + render + state +
   arrow_nav), so Menu/Kinds/future hosts inherit blocks and arrow nav; the Alias
   host only flips `blocks: true` and keeps its cell rendering (reusability
   principle).

---

## Part A -- CR-CH-076 Block line commands (tasks group 9)

- [ ] 1. Add the `blocks` opt-in to `LineCommandSet` (red first).
      Add a failing unit test in `state.rs` `mod tests` asserting
      `LineCommandSet::alias().blocks == true` and that a `const fn` default set
      has `blocks == false`. Then add `pub blocks: bool` to `LineCommandSet`, set
      it `true` in `LineCommandSet::alias()`, `false` elsewhere, and extend
      `enabled_letters` only if a header hint is wanted (optional). Keep
      `move_: false` in `alias()` unchanged so MM is disabled (Req 7.12).
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/state.rs`
      Verify: `cargo test -p ff-desktop alias_line_command_set_reports_enabled_letters blocks` -- new + existing state tests pass; `cargo check -p ff-desktop`.
      Validates: Requirement 9.1, 9.5.

- [ ] 2. Write FAILING pure unit tests for block apply over the stub `Vec<Row>`.
      In `line_commands.rs` `mod tests`, add a second stub host (or extend
      `StubHost`) whose `line_command_set()` has `blocks: true` and `move_: true`
      so CC and MM can both be exercised. Add tests, each annotated:
      `// Validates: ispf-table-editor Requirement 9.1/9.2/9.3`:
      DD/DD deletes the inclusive range; RR/RR duplicates every row in the range;
      CC/CC + A copies the block after the target; CC/CC + B before; MM/MM + A
      moves the block (removes source range, inserts at target); reversed-order
      markers (`DD` on row 3 then row 1) form the same inclusive 1..=3 range.
      They fail today because the `Block` arm clears and ignores the markers.
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs`
      Verify: `cargo test -p ff-desktop --no-run -p ff-desktop` compiles the tests; running `cargo test -p ff-desktop block_` shows the new tests FAIL.
      Validates: Requirement 9.1, 9.2, 9.3.

- [ ] 3. Write FAILING unit tests for unterminated / malformed / overlap blocks.
      Add tests: a lone `DD` yields a non-blocking notice containing "block not
      closed" (or similar), RETAINS the marker (`rows[i].act == "DD"`), makes NO
      structural change, and a well-formed `I`/`D` in the SAME batch still
      applies; three `DD` markers yield a malformed/"too many" notice and no
      change; two overlapping blocks (e.g. `DD` 0..=2 and `RR` 1..=3) are rejected
      with a notice and neither applied. Also add the distinction test: a single
      `R` repeats one row while a matched `RR`/`RR` repeats the range (Req 9.2).
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs`
      Verify: `cargo test -p ff-desktop block_` -- the new tests FAIL (reason: blocks still cleared).
      Validates: Requirement 9.2, 9.4.

- [ ] 4. Implement the block path in the apply adapter (make tasks 2-3 green).
      In `apply_line_commands`, replace the `LineCommandCategory::Block` arm: when
      `set.blocks` is true, COLLECT `(backing_index, LineCommandKind)` block
      markers instead of clearing them; otherwise keep today's clear+notice
      (Req 9.5). After the collect pass, for each distinct block kind present,
      build a `PendingCommandStore`, `add` its markers (line = backing index,
      `PendingReason::AwaitingPair`), call `BlockPairValidator::validate_pair`;
      on `Ok(pair)` record it, on `Err(AwaitingPair|TooManyMarkers)` push a
      non-blocking notice and leave those Act cells uncleared (retained). Run
      `check_overlaps` across resolved pairs; on overlap push a notice and drop
      the overlapping pairs. For CC/MM pairs, resolve the A/B target from the
      already-collected `targets` list (reuse the existing target-consume logic);
      a CC/MM with no target is retained with the existing "pending a target"
      notice. Translate each applied pair into the `before`/`after`/`removed`
      vectors per design decision 3, snapshotting clones while original indices
      are valid; clear the consumed block Act cells. Gate MoveBlock on
      `set.move_` so Alias (move_ = false) disables MM. Keep the function and
      file under 400 non-test lines -- if it would exceed, extract a
      `apply_blocks(...)` helper (same file or a sibling `block_apply.rs`).
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs` (optional new `crates/ff-desktop/src/shell/ispf_table_editor/block_apply.rs` + `mod.rs` re-export)
      Verify: `cargo test -p ff-desktop block_` all pass; `cargo test -p ff-desktop` for the module's existing single-command tests still pass; `cargo clippy -p ff-desktop -- -D warnings`; `cargo fmt -p ff-desktop -- --check`.
      Validates: Requirement 9.1-9.6.

- [ ] 5. Write a FAILING full-shell egui_kittest block end-to-end test, then confirm green.
      In `shell/tests_focus.rs`, add a test modelled on
      `full_shell_alias_editor_enter_in_act_cell_applies_line_command` (same
      harness: `harness_shell()`, `dispatch_command_string("ALIASES")`, seed rows,
      `request_focus(egui::Id::new(("alias_act", i)))`, push `Event::Text`,
      `request_focus(alias_editor_command_line)`, `key_press(Enter)`). Seed three
      rows; type `CC` on row 0 and `CC` on row 1 and `A` on row 2; after Enter
      assert the two-row block was copied after row 2. The test fails until
      `blocks: true` (task 1) and the adapter (task 4) are in; it is already
      satisfied once those land -- run it to confirm green.
      Files: `crates/ff-desktop/src/shell/tests_focus.rs`
      Verify: `cargo test -p ff-desktop full_shell_alias_editor` -- the new block test passes; the existing Act-cell and first-Tab tests still pass.
      Validates: Requirement 9.1, 9.3.

## Part B -- CR-CH-078 Full arrow-key focus navigation (tasks group 11)

- [ ] 6. Write FAILING pure unit tests for the clamped matrix-walk computation.
      Create `shell/ispf_table_editor/arrow_nav.rs` with a `#[cfg(test)] mod
      tests` containing table-driven tests for a `fn arrow_target(rows: usize,
      cols: usize, cur: (usize, usize), dir: ArrowDir) -> (usize, usize)` (and the
      `ArrowDir` enum): Down from (0,c) -> (1,c); Up from (0,c) -> (0,c) clamped;
      Down from (last,c) -> (last,c) clamped; Right from (r,0) -> (r,1); Left from
      (r,0) -> (r,0) clamped; Right from (r,last) -> (r,last) clamped; no wrap on
      any edge. Annotate `// Validates: ispf-table-editor Requirement 11.1/11.2`.
      Add `pub(crate) mod arrow_nav;` to `shell/ispf_table_editor/mod.rs`.
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/arrow_nav.rs`, `crates/ff-desktop/src/shell/ispf_table_editor/mod.rs`
      Verify: `cargo test -p ff-desktop arrow_target` -- FAILS to compile/pass (function not yet implemented).
      Validates: Requirement 11.1, 11.2.

- [ ] 7. Implement `arrow_target` + `ArrowDir` (make task 6 green).
      Pure, egui-free: saturating add/sub with clamp to `rows-1` / `cols-1`,
      no wrap. Keep the file small.
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/arrow_nav.rs`
      Verify: `cargo test -p ff-desktop arrow_target` all pass; `cargo clippy -p ff-desktop -- -D warnings`.
      Validates: Requirement 11.1, 11.2.

- [ ] 8. Build the field-matrix in the render surface and extend the row-render contract.
      The surface needs the per-cell stable ids to drive the walk. Extend
      `IspfTableRender` with a method returning a row's focusable cell ids in
      visual order, e.g. `fn row_cell_ids(&self, backing: usize) -> Vec<egui::Id>`
      (Act, CE, Root, Alias, Parameters, Description for Alias -- the
      `row_control_id(prefix, backing)` ids and the CE/Root combo ids). Implement
      it on `AliasEditorState` in `alias_editor_panel/host.rs` using the same
      prefixes `rows.rs` uses (`alias_act`, `alias_ce` via combo id salt, etc.).
      Note: combo ids come from `from_id_salt((salt, backing))`; use
      `egui::Id::new(("alias_ce", backing))`-equivalent that matches what egui
      derives, OR restrict the first matrix column set to the stable
      `TextEdit`-`.id(...)` cells (Act/Alias/Parameters/Description) plus capture
      combo ids from their responses during `render_row_cells`. Prefer capturing
      the real same-frame ids: have `render_row_cells` (or a new sibling) return
      the row's ids so the matrix uses fresh ids that round-trip through focus
      (the B056 lesson: a guessed combo id will not match). Assemble
      `Vec<Vec<egui::Id>>` in `render_ispf_table` as rows are drawn.
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/render.rs`, `crates/ff-desktop/src/alias_editor_panel/host.rs`, `crates/ff-desktop/src/alias_editor_panel/rows.rs`
      Verify: `cargo check -p ff-desktop`; `cargo clippy -p ff-desktop -- -D warnings`.
      Validates: Requirement 11.2, 11.3.

- [ ] 9. Implement the arrow walk in the render path (consume-before-egui).
      In `render_ispf_table`, after the data rows are laid out and the matrix is
      built, read the focused id. If it matches a matrix cell (and is NOT
      `command_line_id()`), for each of Up/Down/Left/Right that
      `ui.input(|i| i.key_pressed(k))`, CONSUME it with
      `ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, k))`, compute the
      target with `arrow_nav::arrow_target`, and
      `ui.ctx().memory_mut(|m| m.request_focus(target_id))`. Do nothing when the
      command line holds focus (preserves the CR-NR-096 history gesture, Req 11.5).
      Keep `render.rs` under 400 non-test lines -- if it would exceed, move the
      walk into `arrow_nav.rs` as `fn apply_arrow_walk(ui, matrix, focused) ->
      Option<egui::Id>` and call it from the surface.
      Files: `crates/ff-desktop/src/shell/ispf_table_editor/render.rs` (+ `arrow_nav.rs` if extracted)
      Verify: `cargo check -p ff-desktop`; `cargo clippy -p ff-desktop -- -D warnings`; `cargo fmt -p ff-desktop -- --check`.
      Validates: Requirement 11.1, 11.2, 11.3, 11.5.

- [ ] 10. Write a FAILING full-shell egui_kittest arrow test, then confirm green.
      In `shell/tests_focus.rs`, add a test (harness as in task 5): seed two+ rows,
      `request_focus(("alias_act", 0))`, run, `key_press(ArrowDown)`, run, assert
      focus is now `("alias_act", 1)`; then `key_press(ArrowDown)` again on the
      last row and assert focus unchanged (clamp); `request_focus(("alias_act",
      0))`, `key_press(ArrowRight)`, assert focus moved to the next column cell in
      row 0 (the CE combo's captured id or the Alias field). Annotate
      `// Validates: ispf-table-editor Requirement 11.1, 11.2, 11.3`.
      Files: `crates/ff-desktop/src/shell/tests_focus.rs`
      Verify: `cargo test -p ff-desktop full_shell_alias_editor` -- the new arrow test passes.
      Validates: Requirement 11.1, 11.2, 11.3.

- [ ] 11. Regression-confirm Tab/Shift+Tab and first-Tab are unchanged.
      Do not modify the stable ids. Run the existing focus tests.
      Files: (none -- regression guard)
      Verify: `cargo test -p ff-desktop full_shell_alias_editor_first_tab_focuses_first_interior full_shell_alias_editor_enter_in_act_cell_applies_line_command` -- both pass.
      Validates: Requirement 11.4.

## Part C -- CR-CH-077 Single-chrome layout (tasks group 10)

- [ ] 12. Write a FAILING snapshot/unit test that the Alias screen model carries the chrome.
      In `shell/tests_screen_direct*.rs` (or a new `#[cfg(test)]` test near the
      alias provider), add a test that opens ALIASES, seeds a row, builds
      `alias_editor_screen_model()`, and asserts the model INCLUDES the
      `Command ===>` line (e.g. `model.command_line` reflects the surface command
      line) AND a table whose header row equals
      `[Act, CE, Command, Alias, Parameters, Description]` -- i.e. the header is
      present in the captured model. If the current `command_line`/header are
      already present, assert the stronger property the requirement adds: that the
      rendered snapshot text/markdown shows Title -> `Command ===>` -> header ->
      rows in that order (assert on the model fields the renderer consumes).
      Annotate `// Validates: ispf-table-editor Requirement 10.3`.
      Files: `crates/ff-desktop/src/shell/tests_screen_direct.rs` (or the nearest existing screen-model test module)
      Verify: `cargo test -p ff-desktop alias_screen_model` -- FAILS where the chrome is missing.
      Validates: Requirement 10.3.

- [ ] 13. Fill the snapshot chrome in the Alias provider (make task 12 green).
      In `alias_editor_screen_model`, ensure `model.command_line` is set to the
      surface's `Command ===>` text (the panel's command line, not only
      `self.command_text`) and that `alias_table()` (already headered) is pushed
      so the header row appears; if the renderer needs the header as an explicit
      element, add it. Keep the column order identical to the live
      `ALIAS_COLUMNS` order (Req 7.3). Do not change dispatch/nav/save.
      Files: `crates/ff-desktop/src/shell/screen_providers.rs`, `crates/ff-desktop/src/shell/screen_providers_tables.rs`
      Verify: `cargo test -p ff-desktop alias_screen_model` passes; `cargo clippy -p ff-desktop -- -D warnings`.
      Validates: Requirement 10.3, 10.4.

- [ ] 14. Write a FAILING full-shell test that the live panel shows exactly ONE Title and ONE Command line.
      In `shell/tests_focus.rs` (full-shell harness), open ALIASES, run frames,
      and assert the single-chrome invariant. Because egui text is not trivially
      queryable, assert via the accessibility tree / `harness.get_all_by_label`
      or a count of nodes whose text is the Alias title / starts with
      "Command ===>": exactly one each. Model the query on the existing
      `status_bar_segments_are_not_tab_focus_stops` / theme `get_by_label` usage.
      Annotate `// Validates: ispf-table-editor Requirement 10.1, 10.2`.
      Files: `crates/ff-desktop/src/shell/tests_focus.rs`
      Verify: `cargo test -p ff-desktop full_shell_alias_editor_single_chrome` -- FAILS if a duplicate Title/command line is drawn.
      Validates: Requirement 10.1, 10.2.

- [ ] 15. Remove the duplicate host-drawn chrome (make task 14 green).
      Identify the duplicate: the shared surface draws both a centred `title`
      ("FileForge Workbench Alias Editor") and the `Command ===>` line, while the
      shell also draws the Kind descriptive Title_Line, and/or the host arm draws
      an extra heading. Read the shell central-panel Alias arm and the Title_Line
      render to find which element double-draws, then REMOVE the duplicate (prefer
      removing the surface's redundant centred `title` label if the shell already
      renders the Kind Title_Line, OR remove the host-arm heading -- whichever is
      the genuine duplicate), leaving a single Title line + single `Command ===>`
      line + header + rows. Do NOT add a new chrome mechanism; do not touch
      dispatch/nav/save/descriptor (Req 10.4). Apply the same pattern note for
      Menu/Kinds hosts as a comment for the follow-up (they are out of scope here).
      Files: `crates/ff-desktop/src/shell/render.rs` (Alias central-panel arm) and/or `crates/ff-desktop/src/shell/ispf_table_editor/render.rs` and/or `crates/ff-desktop/src/alias_editor_panel/render.rs`
      Verify: `cargo test -p ff-desktop full_shell_alias_editor_single_chrome full_shell_alias_editor_first_tab_focuses_first_interior` -- single-chrome test passes and the first-Tab test still passes (entry control + anchors unchanged); `cargo clippy -p ff-desktop -- -D warnings`; `cargo fmt -p ff-desktop -- --check`.
      Validates: Requirement 10.1, 10.2, 10.4.

## Part D -- finalize

- [ ] 16. Update the spec task checkboxes and coverage.
      Mark the completed items in
      `docs/specs/ispf-table-editor/tasks.md` (groups 9, 10, 11) and
      `docs/specs/command-configurator/` where Req 7.11-7.13 are tracked, noting
      MM-disabled on Alias. ASCII-only; checkbox markers `[ ]`/`[x]` only.
      Files: `docs/specs/ispf-table-editor/tasks.md`, `docs/specs/command-configurator/tasks.md` (if present)
      Verify: re-read the files; `cargo fmt -p ff-desktop -- --check` (docs unaffected but confirm no stray source edits).

- [ ] 17. Final scoped verification.
      Run the scoped trio and the touched test groups; read logs; confirm clean.
      Files: (none)
      Verify: `cargo check -p ff-desktop`; `cargo clippy -p ff-desktop -- -D warnings`; `cargo fmt -p ff-desktop -- --check`; `cargo test -p ff-desktop block_ arrow_target full_shell_alias_editor alias_screen_model` -- all pass. The OWNER runs the full `cargo gate` before merge (gate-policy.md).

---

## Digest

### `crates/ff-desktop/src/shell/ispf_table_editor/state.rs`
- Change: add `pub blocks: bool` to `struct LineCommandSet` (after `sort`); set
  `blocks: true` in `const fn alias()`, `false` in any other constructor/literal.
- Test (same file `mod tests`): extend `alias_line_command_set_reports_enabled_letters`
  and add a `blocks` assertion. Covers Req 9.1/9.5.
- Verified today: file reads cleanly; `alias()` currently
  `{insert, delete, repeat, copy, move_: false, sort: true}`; `move_: false`
  keeps MM disabled on Alias.

### `crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs`
- Block to change: the `LineCommandCategory::Block` arm in `apply_line_commands`
  (currently pushes "ignored block ... (not enabled)" and clears). Replace with
  collect-when-`set.blocks`, pair via `ff_line_commands::BlockPairValidator::validate_pair`
  over a local `PendingCommandStore`, translate `BlockPair` into the existing
  `before`/`after`/`removed` rebuild; retain markers + notice on
  `AwaitingPair`/`TooManyMarkers`/overlap; gate `MoveBlock` on `set.move_`.
- Imports to add: `ff_line_commands::{BlockPairValidator, PendingCommandStore,
  ParsedLineCommand, PendingReason, command::{BlockCommandKind, to_block_kind}}`.
- Tests (same file `mod tests`): new block tests (DD/RR/CC A|B/MM, reversed
  order, lone/too-many/overlap, R-vs-RR distinction) over a blocks-enabled stub
  host. Cover Req 9.1-9.4.
- If the file would exceed 400 non-test lines, extract `block_apply.rs` beside it
  (re-export from `mod.rs`).
- Verified today: the rebuild-by-walk model and the single-command tests exist
  and pass; the `Block` arm is the only not-enabled path.

### `crates/ff-desktop/src/shell/ispf_table_editor/arrow_nav.rs` (NEW)
- Add: `ArrowDir` enum + pure `fn arrow_target(rows, cols, cur, dir) -> (usize,
  usize)` (saturating, clamped, no wrap); optionally `fn apply_arrow_walk(...)`.
- Test (same file `mod tests`): table-driven clamp cases. Cover Req 11.1/11.2.
- `mod.rs`: add `pub(crate) mod arrow_nav;`.

### `crates/ff-desktop/src/shell/ispf_table_editor/render.rs`
- Block to change: `render_ispf_table` -- build a `Vec<Vec<egui::Id>>` field
  matrix from the per-row cell ids as rows are drawn; after layout, read focused
  id, and for a focused matrix cell consume the pressed arrow
  (`ui.input_mut(|i| i.consume_key(Modifiers::NONE, key))`) and `request_focus`
  the `arrow_target` id; skip when `command_line_id()` holds focus.
- Contract change: extend `trait IspfTableRender` with a way to obtain a row's
  focusable cell ids in visual order (prefer returning them from the row render
  so ids are fresh/same-frame per the B056 lesson).
- Keep under 400 non-test lines; move the walk to `arrow_nav.rs` if needed.
- Verified today: the surface already owns the single `Command ===>` line + one
  header and renders the scratch row; `command_line_id()` is
  `"alias_editor_command_line"`.

### `crates/ff-desktop/src/alias_editor_panel/host.rs` and `rows.rs`
- Change: implement the new `IspfTableRender` id-providing method on
  `AliasEditorState`, using the `row_control_id(prefix, backing)` ids in `rows.rs`
  (`alias_act`/`alias_surface`/`alias_template`/`alias_description`) plus
  CE/Root combo ids captured from their responses. No change to the Alias
  `line_command_set()` other than inheriting `blocks: true` from `alias()`.
- Verified today: `render_data_row` creates stable `TextEdit` ids via
  `row_control_id`; combos use `from_id_salt((salt, backing))`.

### `crates/ff-desktop/src/shell/screen_providers.rs` and `screen_providers_tables.rs`
- Block to change: `alias_editor_screen_model` -- ensure the captured model
  carries the `Command ===>` line and the `alias_table()` header so the snapshot
  shows Title -> Command -> header -> rows (Req 10.3). `alias_table()` headers are
  already `[Act, CE, Command, Alias, Parameters, Description]` -- match that order.
- Test: a screen-model test asserting the chrome is present. Cover Req 10.3.
- Verified today: `alias_table()` exists with the right headers; the model sets
  `command_line = self.command_text` (needs to reflect the panel command line).

### `crates/ff-desktop/src/shell/render.rs` (Alias central-panel arm) / `alias_editor_panel/render.rs`
- Block to change: remove the DUPLICATE Title/heading or command line so the live
  panel shows exactly one of each (Req 10.1/10.2). Pinpoint during task 15 by
  reading the shell Title_Line arm and the surface `title` label. No
  dispatch/nav/save/descriptor change (Req 10.4).
- Verified today: the surface draws a centred `title` label AND a `Command ===>`
  line; the shell separately renders a Kind descriptive Title_Line -- one of these
  Title draws is the duplicate to remove.

### `crates/ff-desktop/src/shell/tests_focus.rs`
- Add: full-shell egui_kittest tests for (a) a CC/CC+A block copy end to end,
  (b) in-cell arrow Down/Up/Left/Right with edge clamps, (c) single-chrome
  (exactly one Title + one `Command ===>`). Model on
  `full_shell_alias_editor_enter_in_act_cell_applies_line_command` (harness
  `harness_shell()`, `dispatch_command_string("ALIASES")`, `request_focus` on
  `("alias_act", i)` / `"alias_editor_command_line"`, `Event::Text`,
  `key_press`). Cover Req 9.3, 11.1-11.3, 10.1-10.2.
- Verified today: the harness helpers (`harness_shell`, `cmd_field_id`), the
  stable ids, and the seed/focus/enter pattern all exist and pass.

### Build/test commands (discovered, scoped)
- Build/compile: `cargo check -p ff-desktop`
- Lint: `cargo clippy -p ff-desktop -- -D warnings`
- Format: `cargo fmt -p ff-desktop -- --check`
- Tests (scoped): `cargo test -p ff-desktop <name>` (e.g. `block_`, `arrow_target`,
  `full_shell_alias_editor`, `alias_screen_model`).
- Full gate (`cargo gate`) is the OWNER's pre-merge run, not an agent step.
