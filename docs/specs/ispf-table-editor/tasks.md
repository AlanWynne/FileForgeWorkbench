# Tasks -- Shared ISPF Table Editor + Line-Command Component (CR-CH-075, extended by CR-CH-076 / 077 / 078)

These tasks implement the shared component ONLY. The per-editor redesigns
(Alias / Menu / Kinds) have their own task groups in their own specs and DEPEND
on this component landing first. TDD per `.kiro/steering/testing.md`: write the
failing test first, then the minimum code. Agents compile-verify SCOPED
(`cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D warnings`,
`cargo fmt -p ff-desktop -- --check`) and run the quick scoped unit tests; the
OWNER runs the full `cargo gate` (`.kiro/steering/gate-policy.md`). No file over
400 non-test lines (`.kiro/steering/rust-standards.md`).

- [ ] 1. Define the host contract and column model
  - [ ] 1.1 Add the `IspfTableHost` trait and `ColumnSpec` / `LineCommandSet` /
        `FilterState` types in `shell/ispf_table_editor/state.rs` (no render yet).
        (Req 1.3, 3.3, 5.1)
  - [ ] 1.2 Add `TableEditorState` (command_line, filter) and document that the
        Act letter lives on the host Row_Model as a transient `act: String`.
        (Req 4.1)
  - [ ] 1.3 Unit tests: a `ColumnSpec` round-trips its filterable/sortable flags
        and comparison mode; a `LineCommandSet` reports enabled commands.
        (Validates Req 1.3, 5.1)

- [ ] 2. The line-command apply pass (pure, reuses ff-line-commands)
  - [ ] 2.1 Write failing unit tests for the generic apply adapter over a stub
        `Vec<Row>`: `I` inserts a blank after; `D` removes; descending-index
        processing keeps indices intact across a mixed I/D batch; an unrecognised
        letter is cleared and ignored. (Validates Req 4.2, 4.3, 4.5)
  - [ ] 2.2 Implement the immediate (I/D) apply pass in
        `shell/ispf_table_editor/line_commands.rs`, sourcing classification from
        `ff-line-commands` (no second parser). (Req 4.2, 4.3, 4.7)
  - [ ] 2.3 Write failing unit tests for C/M pairing: `C` + `A`/`B` copies to the
        target; `M` + `A`/`B` moves; an unresolved `C` source is RETAINED with a
        notice; a conflicting marker set is rejected and the cells cleared.
        (Validates Req 4.2, 4.4, error-handling)
  - [ ] 2.4 Implement C/M via the engine's source/target pairing + pending
        retention (line-commands Req 33/35). (Req 4.4, 4.7)

- [ ] 3. Sort
  - [ ] 3.1 Failing unit tests: stable sort by a declared key column (ASC/DESC);
        `SORT` on a host with no sortable column is a no-op with a notice; sort
        reorders the backing list and the filtered view reflects it.
        (Validates Req 5.1-5.4)
  - [ ] 3.2 Implement `SORT <column> [ASC|DESC]` over the backing row vector
        (stable). (Req 5.1-5.4)

- [ ] 4. Filter
  - [ ] 4.1 Failing unit tests: empty filter shows all; a single filter
        restricts; multiple filters AND; filtering never mutates the backing
        list; a line command on a filtered row maps to the correct backing index.
        (Validates Req 3.1-3.4)
  - [ ] 4.2 Implement the display-only filter + backing-index mapping. (Req 3.1-3.4)

- [ ] 5. The render surface
  - [ ] 5.1 Implement `render_ispf_table<H: IspfTableHost>(ui, host, &mut
        TableEditorState) -> TableAction` in `shell/ispf_table_editor/render.rs`:
        `Command ===>` line (stable id), header row, filter row, scrolling data
        rows with Act cells + host-rendered column cells, in visual Tab order.
        (Req 1.1, 2.1, 7.2)
  - [ ] 5.2 Emit the command-line Action on Enter (apply pending first, then
        interpret the verb); return focus to `Command ===>` after a structural
        change. (Req 2.1, 2.2, 2.4)
  - [ ] 5.3 Render the always-present insertable scratch row when the backing
        list is empty; promote it on edit / `I`. (Req 4.6)
  - [ ] 5.4 Honour `Command_Line_Position = Bottom` (workspace-kinds Req 8) when
        the host Context's Kind requests it. (Req 2.5)

- [ ] 6. Focus contract
  - [ ] 6.1 Provide the `InteriorFocus { first, last }` anchors from the surface
        (first = filter row's first control's fresh id; last = last data-row
        control or the filter row) for the host to report through the single
        latch path. (Req 7.1, 7.2)

- [ ] 7. Save-seam routing
  - [ ] 7.1 Route `SAVE`/`END`/`CANCEL`/`RETURN`/`REFRESH` to the host's existing
        CR-CH-071 seam; apply pending line commands before a commit; never
        persist on a bare edit/line-command/sort. (Req 6.1-6.4)
  - [ ] 7.2 Unit test (pure, over a stub host): a bare edit does not call persist;
        `SAVE` applies the pending batch then calls persist once.
        (Validates Req 6.2, 6.3)

- [ ] 8. Snapshot-column alignment hook
  - [ ] 8.1 Expose the host's `ColumnSpec` order so its `ScreenProvider` can
        populate a `ff-screen-model::Table` with the same headers/order the
        surface renders (Group A capture). (Req 7.3)
  - [ ] 8.2 Document in each consuming editor's spec that its provider reads the
        shared `ColumnSpec` order. (Req 7.3)

- [ ] 9. Block line commands (RR / DD / CC / MM) -- CR-CH-076 (Req 9)
  - [ ] 9.1 Add a block opt-in to `LineCommandSet` in
        `shell/ispf_table_editor/state.rs` (e.g. a `blocks` flag / RR/DD/CC/MM
        sub-flags), advertised per-host; no engine change. (Req 9.1, 9.5)
  - [ ] 9.2 Write FAILING pure unit tests for the block apply adapter over a stub
        `Vec<Row>`: a matched `DD`/`DD` pair deletes the inclusive range; `RR`/`RR`
        duplicates every row in the range; `CC`/`CC` + `A`/`B` copies the block to
        the target; `MM`/`MM` + `A`/`B` moves the block; markers typed in reverse
        order still form the same inclusive range (order-normalised).
        (Validates Req 9.1, 9.2, 9.3)
  - [ ] 9.3 Write FAILING unit tests for the unterminated / malformed block: a
        lone `DD` yields a non-blocking "block not closed" notice, RETAINS the
        marker, and makes NO structural change while other batch commands still
        apply; three `DD` markers yield a malformed notice; overlapping blocks are
        rejected with a notice. (Validates Req 9.4)
  - [ ] 9.4 Write a FAILING unit test that the single `R` / `Rn` path is
        UNCHANGED alongside blocks (a single `R` repeats one row; `RR`/`RR`
        repeats the range), proving the distinction of Req 9.2.
        (Validates Req 9.2)
  - [ ] 9.5 Implement the block path in
        `shell/ispf_table_editor/line_commands.rs`: when `blocks` is enabled,
        collect block markers, pair them via
        `ff_line_commands::BlockPairValidator::validate_pair`, translate each
        resolved `BlockPair` into the existing stable-original-index rebuild
        (delete range / repeat range / copy-block+A/B / move-block+A/B), and map
        `AwaitingPair` / `TooManyMarkers` / overlap to a retained marker + notice.
        No `ff-line-commands` API change. (Req 9.1-9.6)
  - [ ] 9.6 Write a FAILING full-shell egui_kittest test (in a consuming editor,
        e.g. Alias) that a matched `CC`/`CC` + `A` block copy applied on Enter
        restructures the rows as expected. (Validates Req 9.1, 9.3)

- [x] 10. Single-chrome layout -- CR-CH-077 (Req 10)
  - [x] 10.1 Write a FAILING full-shell egui_kittest / snapshot test asserting the
        live panel shows exactly ONE Title line and ONE `Command ===>` line (no
        duplicate host-drawn heading / command line). (Validates Req 10.1, 10.2)
    - Added `full_shell_alias_editor_is_single_chrome` (tests_focus.rs): asserts
      exactly ONE "Command ===>" node, ZERO "FileForge Workbench Alias Editor"
      heading, and ONE "Alias Editor" Title via the accessibility tree.
  - [x] 10.2 Remove the duplicate title/command-line draw from the host Context
        render arm (Alias first; same fix pattern for Menu / Kinds) so only the
        shared component's chrome renders. (Req 10.1, 10.2)
    - Owner decision: the SURVIVING title + command line are the shell's
      app-level Title_Line + command field. The shared `render_ispf_table`
      surface now draws NEITHER a centred title NOR a `Command ===>` line; the
      shell's command field is routed into the Alias command-environment handler
      (`run_command_line` -> `apply_alias_command_line`) when the Alias Context
      is active. Same pattern applies to future Menu / Kinds hosts automatically.
  - [x] 10.3 Write a FAILING unit/snapshot test that the `ScreenProvider` output
        INCLUDES the `Command ===>` line and the column header row in the same
        order/columns as the live surface.
        (Validates Req 10.3)
    - Added `full_shell_alias_screen_model_carries_command_line_and_header`
      (tests_screen_directive.rs).
  - [x] 10.4 Extend the host `ScreenProvider` to emit the single-chrome layout
        (Title -> `Command ===>` -> header -> rows), reading the shared
        `ColumnSpec` order (Req 7.3), so live and snapshot AGREE. (Req 10.3, 10.4)
    - Already satisfied by the existing provider: `alias_editor_screen_model`
      sets `command_line` from the shell command text and pushes `alias_table()`
      (headers in ALIAS_COLUMNS order). Test 10.3 is the regression guard.

- [ ] 11. Full arrow-key focus navigation -- CR-CH-078 (Req 11)
  - [ ] 11.1 Write FAILING pure unit tests for the matrix-walk target computation:
        Down/Up move to the same column in the adjacent row, clamped at
        first/last row; Right/Left move to the adjacent column, clamped at
        first/last column; no wrap. (Validates Req 11.1, 11.2)
  - [ ] 11.2 Implement the field-matrix descriptor (ordered (row, column) ->
        stable `egui::Id`) and the clamped target computation in
        `shell/ispf_table_editor/` (pure, egui-free where possible). (Req 11.1, 11.2)
  - [ ] 11.3 Implement the arrow walk in the render path: detect the focused
        matrix cell, CONSUME the arrow key with `input_mut(..).consume_key(..)`
        BEFORE egui traverses, and `request_focus` the target id -- the
        menu-bar-keynav pattern so it does not fight egui. Do NOT fire on the
        `Command ===>` field (preserve the CR-NR-096 history gesture). (Req 11.3, 11.5)
  - [ ] 11.4 Write a FAILING full-shell egui_kittest test (in a consuming editor):
        with focus on a data-row Act cell, a Down press moves focus to the Act
        cell of the next row; a Right press moves to the next column; both clamp at
        the edges. (Validates Req 11.1, 11.2, 11.3)
  - [ ] 11.5 Confirm (regression) the existing first-Tab egui_kittest test still
        passes (Tab/Shift+Tab + stable ids unchanged) with arrow nav added.
        (Validates Req 11.4)

## Acceptance Criteria Coverage (CR-CH-075 / 076 / 077 / 078)

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|--------------------|
| 1 Reusable surface | 1.1-1.4 | 1.1, 1.2, 5.1 |
| 2 Command line | 2.1-2.5 | 5.1, 5.2, 5.4, 7.1 |
| 3 Filter | 3.1-3.4 | 4.1, 4.2 |
| 4 Act + I/D/C/M | 4.1-4.7 | 2.1-2.4, 5.3 |
| 5 Sort | 5.1-5.4 | 3.1, 3.2 |
| 6 CR-CH-071 save | 6.1-6.4 | 7.1, 7.2 |
| 7 Focus + snapshot | 7.1-7.3 | 6.1, 8.1, 8.2 |
| 8 Scope | 8.1-8.4 | (scoping; no code -- enforced by the per-editor task groups) |
| 9 Block commands (CR-CH-076) | 9.1-9.6 | 9.1-9.6 |
| 10 Single-chrome (CR-CH-077) | 10.1-10.4 | 10.1-10.4 |
| 11 Arrow navigation (CR-CH-078) | 11.1-11.6 | 11.1-11.5 |

## Notes

- This component MUST land before the Alias / Menu / Kinds redesign task groups
  (those consume it).
- The Config editor ISPF redesign is OUT OF SCOPE (a follow-up).
- No `ff-line-commands` public-API change; the engine is reused as-is. CR-CH-076
  (task group 9) reuses the engine's EXISTING block model
  (`LineCommandParser` already parses DD/RR/CC/MM; `BlockPairValidator` already
  forms the inclusive `BlockPair`); the change lands only in the shared
  component's apply adapter + `LineCommandSet` opt-in.
- CR-CH-078 (task group 11) adds arrow navigation ADDITIVELY on the existing
  focus latch; Tab/Shift+Tab and the stable first-Tab ids are unchanged (task
  11.5 is the regression guard).
- CR-CH-077 (task group 10) removes a duplicated host chrome draw and fills the
  snapshot's missing chrome; no new chrome mechanism.
