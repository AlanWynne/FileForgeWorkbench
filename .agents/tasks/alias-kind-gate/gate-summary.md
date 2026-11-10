# Alias Workspace Kind gate summary (CR-NR-116)

Branch: alias-kind-gate (worktree .worktrees/alias-kind-gate), off a6aacae.
Docs only. No .rs changed, no cargo run. Not merged, not pushed.
Builds on CR-NR-115 (Common Workspace Frame). First per-Workspace-Kind gate.

## Criteria ids

- command-configurator Requirement 7.14-7.28 (new):
  - 7.14 ownership boundary (Content_Area + one status segment only)
  - 7.15 single representation (no `fields`, no Property/Value table)
  - 7.16 no stray rows (no blank row, no `Aliases` caption)
  - 7.17 grid in the frame's own rules (no nested box)
  - 7.18 columns and plain header
  - 7.19 live-value fidelity (displayed set, backing order, hints excluded)
  - 7.20 cell conventions and widths (Act 4, CE 10, Command 20, Alias 16, Parameters 28, Description 32)
  - 7.21 trailing scratch row
  - 7.22 filter row omitted from the snapshot
  - 7.23 cell safety
  - 7.24 clipboard flavors via `markdown_sections`
  - 7.25 status cells
  - 7.26 live/snapshot status parity
  - 7.27 no framework change
  - 7.28 verification
  - Revision notes added to 7.9 and 7.12.
- ispf-table-editor Requirement 12 (12.1-12.8); Req 4.6 marked extended by 12.5.
- screen-snapshot-scrm Requirement 30.27-30.35; design Section 13 "Third refinement".
- workspace-framework: pointer under Requirement 7.25 only.
- Design: command-configurator design Section 9.11 (fixture 9.11.1, findings 9.11.2, design 9.11.3, status 9.11.4, clipboard 9.11.5, framework verdict 9.11.8, decisions 9.11.9, split 9.11.10, tests 9.11.11).

## Tasks added

- command-configurator tasks.md Task 9 (9.1-9.10), including:
  - 9.6 GOLDEN full-text snapshot test: two-row working copy (row 1 `SS` with EMPTY Parameters and Description, row 2 `CS` with a FILLED Parameters `&1` and Description), trailing scratch row, status cells; plus filtered and ASCII goldens; the three existing Alias goldens regenerated.
  - 9.7 live-panel status-cells test on `workspace_status_cells()` (count, filtered range, `Filter:` only when active, no ` | ` in a cell) and a live/snapshot parity test.
  - 9.8 egui_kittest trailing scratch row; first-Tab test unchanged.
- ispf-table-editor tasks.md group 12 (12.1-12.8) and a coverage row.
- screen-snapshot-scrm tasks.md Phase 2I (2I.1-2I.4).
- All tasks are TDD-first (failing test before implementation). Verification for agents is COMPILE-ONLY: scoped `cargo check`, `cargo clippy -D warnings`, `cargo fmt --check` for ff-desktop / ff-screen-model, plus quick named tests. The owner runs the full `cargo gate`.

## Investigation verdicts

- D2 (duplicate data): `alias_editor_screen_model` builds `fields` and then pushes the table. The clipboard Property/Value table is `render_markdown` building a table from `fields`. Dropping `fields` removes the dot-leader line and the Property/Value table together.
- D5 (Parameters and Description "empty"): nothing is lost. `(&1)`, `note`, `name` and `_` are egui `hint_text` placeholders in `alias_editor_panel/rows.rs`. `alias_table()` maps the right fields and the working-copy values really are empty. The shipped `SS` row has no template. Placeholders are specified as never being snapshot content.
- Cell conventions: `[v v]`, `[ v]`, `____` exist only in renderer tests. The provider emits raw strings, which is why the owner capture shows bare `ffcmd` and blank cells.
- Nested box and stray rows: `render/framed.rs` gives each table a blank row, a caption row and a full inner box of plain text; the frame's band and rule machinery is not used for content.
- Scratch row: the live panel shows it only when the list is empty (ispf-table-editor 4.6), so it is not "always present".
- Status: `workspace_status_cells()` is the single seam read by the live bar and the snapshot. Sources: row count `alias_editor_panel.rows.len()`, visible count `displayed_indices(..).len()`, filter `filter_ce` / `filter_root`. No dirty tracking exists, so no `Modified` cell.

## Shared vs Alias-specific split

Shared (ispf_table_editor / ff-screen-model):
- framed renderer draws tables as frame-ruled bands with junctions and widths (30.27-30.34)
- `ispf_table_editor/snapshot.rs`: cell helpers and `snapshot_table` / `raw_table` builders
- `ColumnSpec` cell kind and width metadata (`with_snapshot`)
- trailing scratch row in `render_ispf_table` (12.5)
- rule that the filter row is omitted (12.6)
- `markdown_sections` mechanism (30.35)

Alias-specific:
- the six column declarations in `ALIAS_COLUMNS`
- `alias_snapshot_cells` (row to raw values)
- `alias_status_cells` and its `ff_i18n` keys, plus the `KindTag::AliasEditor` arm
- `alias_markdown_section()` (raw section)

Menu, Kinds, Keys and Theme inherit all the shared pieces and each declares only its own columns in its own per-Kind gate.

## Target layout (design 9.11.1, pfshow off, no message)

```
┌───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                         Alias Editor                                                          │
├───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Command ===> ________________________________________________________________________________________________________________ │
├──────┬────────────┬──────────────────────┬──────────────────┬──────────────────────────────┬──────────────────────────────────┤
│ Act  │ CE         │ Command              │ Alias            │ Parameters                   │ Description                      │
├──────┼────────────┼──────────────────────┼──────────────────┼──────────────────────────────┼──────────────────────────────────┤
│ ____ │ [ffcmd v]  │ [SNAPSHOT v]         │ SS               │ ____________________________ │ ________________________________ │
│ ____ │ [ffcmd v]  │ [CAPTURE START v]    │ CS               │ &1                           │ Session Recording start          │
│ ____ │ [ffcmd v]  │ [ v]                 │ ________________ │ ____________________________ │ ________________________________ │
├──────┴──────┬─────┴─────────┬────────────┼──────────────────┴──────────────────────────────┴──────────────────────────────────┤
│ Last: 12:06 │ Prof: default │ Aliases: 2 │ Rows 1-2 visible                                                                   │
└─────────────┴───────────────┴────────────┴────────────────────────────────────────────────────────────────────────────────────┘
```

## Decision points (owner accepts or flips; one place each)

- (a) Combo marker: RECOMMEND per cell with a plain header. It is the existing 29.8 / 30.13 convention and keeps an empty combo `[ v]` distinct from an empty text cell `____`. Both CE and Command are combos live, so both get the marker.
- (b) Filter row: RECOMMEND leave it out of the snapshot. It is an input and the Table model has no row type for it. An active filter is shown by a `Filter: <ce>/<root>` status cell.
- (c) Grid header rule directly under the Command line: RECOMMEND yes, matching the owner sketch. 30.30 clarifies that the frame adds no rule there and the grid's own top rule is content.
- (d) Status wording: RECOMMEND `Aliases: <n>`, `Rows 1-<m> visible`, and `Filter: ...` only while a filter is active. "Visible" means the filtered set because the snapshot cannot read scroll state.
- (e) Value fidelity: RECOMMEND mirror stored values exactly (`ffcmd`, not `FFCMD`). There is no `(&1)` value to mirror; if the user types `&1` it appears.
- (f) Scratch row (also in the author report): RECOMMEND the live panel shows it for non-empty lists too (12.5) so snapshot, live and the owner's expectation agree.
- (g) Clipboard (also in the author report): RECOMMEND box first, then one raw `### Aliases` GFM table via `markdown_sections`, and no Property/Value table.

## Framework flag

No framework change. Dispatch, navigation, descriptor, focus contract, save path and the public `ScreenModel`, `Table` and `WorkspaceContext` shapes are untouched. FIVE additive items need owner confirmation:

1. Every framed model with tables (Theme, Menu, Keys, Kinds, Alias) changes appearance; goldens regenerate in the implementation change.
2. The 30.18 clarification (decision c).
3. Live behaviour change in the shared table editor: the trailing scratch row (decision f).
4. Two new fields on `ColumnSpec` (internal to ff-desktop).
5. Structured clipboard flavors use `markdown_sections`; the Req 29.7a HTML scratch `<tr>` no longer holds for the Alias model.

## Files changed (docs only)

- docs/specs/command-configurator/requirements.md, design.md, tasks.md
- docs/specs/ispf-table-editor/requirements.md, design.md, tasks.md
- docs/specs/screen-snapshot-scrm/requirements.md, design.md, tasks.md
- docs/specs/workspace-framework/requirements.md
- docs/status/change-log.md (CR-NR-116)

## Verification of this gate

- Non-ASCII check run on the touched docs; violations outside fenced fixture blocks fixed (see commit message).
- No cargo run: nothing in crates/ changed.

GATE_READY_FOR_APPROVAL
