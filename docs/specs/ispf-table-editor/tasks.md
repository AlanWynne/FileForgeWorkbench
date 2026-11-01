# Tasks -- Shared ISPF Table Editor + Line-Command Component (CR-CH-075)

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

## Acceptance Criteria Coverage (CR-CH-075)

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

## Notes

- This component MUST land before the Alias / Menu / Kinds redesign task groups
  (those consume it).
- The Config editor ISPF redesign is OUT OF SCOPE (a follow-up).
- No `ff-line-commands` public-API change; the engine is reused as-is.
