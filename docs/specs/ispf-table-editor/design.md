# Design -- Shared ISPF Table Editor + Line-Command Component (CR-CH-075)

## Overview

The shared ISPF Table Editor is a reusable egui panel helper in `ff-desktop`
that gives the Alias, Menu, and Kinds editor Contexts an identical ISPF-style
surface: a `Command ===>` line, a filter row, a header row, and a vertically
scrolling list of data rows each carrying an Act prefix cell, with batch line
commands (I / D / C / M) and a Sort capability. It is deliberately a THIN,
GENERIC coordinator: it owns the surface and the line-command structuring but
delegates every field-specific concern (which columns exist, each column's
widget, validation, and the backing fields) to the host editor. The host editors
remain pure `render -> Action` panels dispatched through the `WorkspaceContext`
framework; the shared helper changes NONE of the dispatch, navigation,
persistence, or focus-latch mechanisms -- it builds on them.

This redesign reuses two existing assets rather than inventing anything:
(1) the GUI-free `ff-line-commands` engine (`crates/ff-line-commands`) for
line-command parsing, classification, and block pairing; and (2) the Alias
Editor's existing batch apply flow (`alias_editor_line_commands.rs`), generalised
from its hard-coded I/R/D set to the engine-backed I/D/C/M + Sort vocabulary.
There is NO second line-command engine, NO second dispatcher, NO second save
path, and NO second navigation stack.

## Technology stack (locked once approved)

Rust in the existing `ff-desktop` crate; egui + the `WorkspaceContext` /
`InteriorFocus` framework (CR-NR-078); the existing `ff-line-commands` crate
(already a `ff-desktop` dependency transitively via the editor stack) for the
line-command engine; the existing CR-CH-071 editable-Context seam
(`shell/editable_context.rs`) for SAVE / END / RETURN / CANCEL / REFRESH. NO new
crate, NO new third-party dependency, NO new on-disk format. This matches the
stack the Alias Editor redesign (CR-CH-068) already locked.

## Where the component lives

The component is a shared panel helper module under
`crates/ff-desktop/src/` (e.g. `shell/ispf_table_editor/` with `mod.rs` re-export,
`state.rs`, `render.rs`, `line_commands.rs`, kept each under the 400-line rule of
`.kiro/steering/rust-standards.md`). It is NOT a new crate: it is GUI code that
depends on egui and must sit beside the three editor panels it serves. The three
editor panels (`alias_editor_panel`, `menus_editor_panel`, `kinds_editor_panel`)
each OWN their Row_Model and columns and CALL the shared helper to draw the
surface, rather than the helper owning the Contexts. This keeps the
`WorkspaceContext` one-Context-per-panel shape intact.

Rationale for a `ff-desktop` helper over a new crate: the surface is egui-bound
and tightly coupled to the three editor panels and the shell's `pending_action`
effect pattern, none of which can move out of `ff-desktop`. The PURE logic that
CAN be crate-level (line-command classification and pairing) already lives in the
GUI-free `ff-line-commands` crate and is reused as-is, so the GUI/pure split the
project values is preserved without a redundant new crate.

## The generic contract the host editors implement

The shared helper is generic over the host's working-copy row list. Two viable
shapes were considered:

- **(A) A Rust trait** (`IspfTableHost`) the host panel implements, with
  associated methods: `columns() -> &[ColumnSpec]`, `line_command_set() ->
  LineCommandSet`, `sort_key() -> Option<ColumnId>`, `render_cell(ui, row_idx,
  col, &mut Row)`, `blank_row() -> Row`, `clone_row(&Row) -> Row`, and
  `filter_matches(&Row, &FilterState) -> bool`.
- **(B) A plain struct of closures** the host constructs each frame.

CHOICE: **(A) a trait.** A trait keeps the column/cell/filter contract typed and
discoverable, matches the project's existing `WorkspaceContext` trait pattern
(CR-NR-078), and avoids per-frame closure allocation. The host editor's panel
type implements `IspfTableHost`; the shared `render_ispf_table(ui, host, &mut
TableEditorState) -> TableAction` draws the surface by iterating
`host.columns()`, calling `host.render_cell(...)` per cell, and reading
`host.filter_matches(...)` for the filter. The trait is object-safe only if
needed; a generic `fn render_ispf_table<H: IspfTableHost>(...)` is sufficient and
avoids `dyn` overhead.

The host declares its columns as an ordered `&[ColumnSpec]` where each
`ColumnSpec` carries a stable column id, a header label, a filterable flag (and a
comparison mode: equality or case-insensitive-contains), and a sortable flag. The
helper uses this both for rendering the header/filter rows AND as the single
source of the column list the host's `ScreenProvider` exposes to snapshot capture
(Requirement 7.3) -- the provider reads the same `ColumnSpec` order so the
captured `ff-screen-model::Table` headers match the live surface exactly.

## The shared `TableEditorState`

The helper owns a small transient state struct the host embeds in its panel
state (none of it persisted):

```
struct TableEditorState {
    command_line: String,              // the Command ===> text
    filter: FilterState,               // per-filterable-column inputs
    // per-row Act letters live on the host's Row_Model as a transient
    // `act: String` field (mirroring the existing AliasRowEdit.act), OR in a
    // parallel Vec<String> the helper owns keyed by backing index.
}
```

DECISION: the Act letter lives on the host's Row_Model as a transient `act:
String` (as `AliasRowEdit.act` already does), because the host already owns the
row vector and this keeps the Act cell's value co-located with its row across the
owned-panel swap. The helper reads/clears it during the apply pass. This matches
the Alias Editor's shipped pattern (CR-CH-068 section 9.10.4) and needs no new
parallel store.

## Line-command apply pass (reusing ff-line-commands)

Line-command letters typed into Act cells are PENDING state on the rows; nothing
restructures mid-frame (that would fight egui immediate mode and the Tab/focus
contract). On Enter on the `Command ===>` line the host emits an apply Action and
the shell performs ONE deterministic pass in `apply_<editor>_action`:

1. Collect the `(backing_index, act_letter)` pairs for non-empty Act cells.
2. Hand the letters to `ff-line-commands` for parsing and classification
   (immediate vs source-marker vs target-marker; block pairing for CC/MM where a
   host enables blocks). The engine already implements D/I/R/C/M classification,
   the compatibility matrix, and the pending/retention rules (line-commands
   Requirement 33/34/35) -- the table editor calls into these rather than
   re-deriving them.
3. Apply the classified batch to the host's row vector:
   - Immediate (`I`, `D`): process by DESCENDING backing index so an insert/delete
     never shifts an unprocessed row's index (the proven Alias-editor rule,
     CR-CH-068 9.10.4). `I` inserts `host.blank_row()` after the row; `D` removes
     the row.
   - Paired (`C` + `A`/`B`, `M` + `A`/`B`): resolve the source range against the
     target marker using the engine's pairing result; `C` inserts
     `host.clone_row(src)` copies at the target; `M` removes the source rows and
     re-inserts them at the target. An unresolved source marker is RETAINED and a
     non-blocking notice is surfaced (engine Requirement 35), not discarded.
4. Clear every consumed Act cell; an unrecognised or not-enabled letter is cleared
   and ignored with a notice (Requirement 4.5).

The engine is GUI-free and already unit-tested; the table editor's new logic is
only the small adapter that maps the engine's classified result onto a generic
`Vec<Row>` (insert/remove/clone), which is itself pure and unit-testable without
egui.

Why generalise the Alias mini-flow rather than keep three implementations: the
Alias flow already proves the descending-index immediate-apply model works with
the `pending_action` discipline; the only gaps are C/M (which the engine already
classifies and pairs) and Sort. Folding that one flow into the shared helper and
sourcing classification from `ff-line-commands` removes the duplication and gives
all three editors C/M/Sort uniformly -- the investigation's "shared table-editor
warranted" conclusion (findings section 5c).

## Sort

Sort reorders the host's backing row vector by a host-declared key column
(`ColumnSpec.sortable`), invoked as `SORT <column> [ASC|DESC]` on the command
line (ascending default) and/or a header click. It is a STABLE sort
(`slice::sort_by_key` is stable) so equal rows keep their order. Sort is a
persisted reordering (like Move). A host that declares no sortable column gets a
non-blocking "nothing to sort" notice for `SORT`. The Alias Editor enables Sort
(the owner wants it; Move is nonsensical for an unordered alias set); the Menu and
Kinds editors rely on Move for ordering and need not enable Sort, but MAY.

## Filter

`FilterState` holds one input per filterable column. The displayed data-row set
is the subset where every SET filter input matches its column per the
`ColumnSpec` comparison mode (equality or case-insensitive-contains), ANDed.
Filtering is DISPLAY-only: the render iterates the filtered subset but captures
each row's ORIGINAL backing index for the pending-line-command snapshot, so a `D`
on a visible row deletes the correct backing row (Requirement 3.4). The filter
applies on Enter / `FILTER <args>` / `RESET` (ISPF command-driven); a host may
also filter live from the field bindings, but the command form is always present.

## Focus contract and Tab order

The host Context's `WorkspaceContext::render` reports `InteriorFocus { first,
last }` through the single latch path. `first` is the FILTER row's first
focusable control, captured as its fresh same-frame `egui::Id` (a combo's id is
captured from its `response.id` per `.kiro/steering/workspace-conformance.md`);
`last` is the last data-row control (or the filter row when there are no data
rows). The `Command ===>` line is the ENTRY control, so the first Tab from it
lands on the filter row's first field with no phantom stop. Controls are created
in visual order (`Command ===>` -> filter fields -> per row: Act -> columns),
each per-row control carrying a stable salted id keyed by the row's backing index
(the existing `surface_id(i)` pattern). This is exactly the Alias Editor's shipped
reconciliation (CR-CH-068 9.10.8), lifted into the shared helper so all three
editors inherit it; it is NOT a new focus mechanism.

## Integration with the CR-CH-071 save seam

The helper never persists. It routes `Command ===>` verbs to the host's existing
`editable_persist` / `editable_discard` / `editable_reload` seam
(`shell/editable_context.rs`): `SAVE` -> apply pending line commands then persist
working copy, stay; `END` -> persist then exit via the Navigation_Stack;
`CANCEL` / `RETURN` -> discard working copy, exit; `REFRESH` (where the host
offers it) -> discard, reload. The working copy / backing store boundary is the
host's; the helper only mutates the working-copy row vector.

## Error handling (concrete, per operation)

- **Unresolved C/M source marker** (user typed `C` but no `A`/`B` target in the
  cycle): recoverable. The engine returns the source as still-pending; the helper
  RETAINS the pending marker (does not apply, does not clear) and surfaces a
  non-blocking status (e.g. "Copy/Move source pending a target (A/B)"). Logged at
  debug in the shell. The row list is unchanged.
- **Conflicting markers** (e.g. two targets, or an `I`/`D` conflicting with a
  pending block per the engine's compatibility matrix): recoverable. The engine's
  compatibility validation (line-commands Requirement 34) rejects the invalid
  combination; the helper clears the offending Act cells and surfaces a
  non-blocking notice; no structural change is applied for the rejected set.
- **Unrecognised / not-enabled Act letter**: recoverable. Cleared and ignored
  with a notice (Requirement 4.5); the row list is unchanged.
- **`SORT` on a host with no sortable column**: recoverable. Non-blocking
  "nothing to sort" notice; order unchanged.
- **Empty backing list**: not an error. The helper renders a transient scratch
  blank row so an insert surface is always visible (Requirement 4.6); an edit or
  `I` promotes it into the list.
- **Save/validation failure**: owned by the host's existing CR-CH-071 save path
  and its validator (e.g. the Alias `validate_user_alias_rows`); the helper does
  not change where or how validation errors are reported -- a save that the
  host's validator rejects leaves the file unchanged and the working copy intact,
  exactly as today.

The helper itself performs NO file or network I/O, so its only failure modes are
the line-command/validation cases above; all are recoverable and non-fatal, and
none crash the Context.

## Input validation (per external input)

- **Act cell**: a 1-2 character string (single letter, or the doubled block form
  where enabled). Optional (blank = no command). Validated by the
  `ff-line-commands` parser; an unparseable value is cleared with a notice.
- **`Command ===>` text**: a free string. Optional (bare Enter = apply pending
  only). The leading verb is matched case-insensitively against the known set
  (SAVE / END / CANCEL / RETURN / REFRESH / FILTER / RESET / SORT) plus whatever
  the host forwards; an unrecognised verb is forwarded to the host, which may
  reject it with a notice.
- **Filter inputs**: free strings per filterable column. Optional. No limit
  beyond the field; an empty value means "no filter on this column".
- **`SORT` argument**: a column name (matched case-insensitively against
  `ColumnSpec` ids) and an optional `ASC` / `DESC`. An unknown column -> notice,
  no reorder; a missing direction defaults to ascending.

## Invariants and which layer owns them

- "Nothing persists on a bare edit / line command / sort": owned by the helper +
  the host's CR-CH-071 seam (the helper only mutates the working copy; only a
  commit verb reaches `editable_persist`).
- "Line command index integrity" (a batch never corrupts indices): owned by the
  helper's apply pass (descending-index immediate processing + engine-resolved
  pairing).
- "Filtered-out rows are preserved and saved": owned by the helper (display-only
  filter; backing-index line commands).
- "No phantom Tab stop / first Tab lands on the filter row": owned by the host's
  `InteriorFocus` report through the single latch path (workspace-conformance.md),
  which the helper's visual-order rendering supports.
- "Snapshot columns match the live surface": owned by the host's `ScreenProvider`
  reading the SAME `ColumnSpec` order the helper renders.
- "No schema / dispatch / navigation / save-path change": owned by each host's
  existing seam; the helper touches none of them.

## Testability

- PURE unit tests on the apply-pass adapter (no egui): `I` inserts a blank after;
  `D` removes; `C` with an `A`/`B` target copies to the target; `M` moves;
  multiple pending commands apply deterministically (descending index) without
  corruption; an unresolved `C` is retained; an unrecognised letter is cleared
  and ignored. These drive a generic `Vec<Row>` with a stub Row.
- PURE unit tests on Sort: a stable sort by the key column; a `SORT` on a host
  with no sortable column is a no-op; sort respects the filter display contract.
- PURE unit tests on the filter: empty filter shows all; a single filter
  restricts; multiple filters AND; filtering never mutates the backing list;
  backing-index mapping is correct under a filter.
- REUSED: the `ff-line-commands` engine's own unit tests already cover parsing,
  classification, pairing, and the compatibility matrix; the table editor does
  not re-test the engine, only its adapter.
- Per-host MANDATORY full-shell `egui_kittest` first-Tab test (one per consuming
  editor, owned by that editor's spec): the first Tab from `Command ===>` lands
  on the filter row's first control (no phantom stop). This is a
  specialisation of the workspace-conformance.md GUI-behaviour rule.

The GUI surface itself (scrolling, exact pixel layout) is covered by the per-host
`egui_kittest` tests; the behavioural logic (line commands, sort, filter,
index-mapping) is fully unit-testable without egui, which is why the apply/sort/
filter logic is kept pure and separate from the render.

## Framework-conformance statement

Per `.kiro/steering/framework-conformance.md`, this component builds ON the
framework and changes NOTHING load-bearing:
- Dispatch: unchanged. The `Command ===>` verbs route to each host's existing
  apply pass / CR-CH-071 seam; no new dispatcher, no `resolve_target` bypass.
- Navigation: unchanged. END / RETURN use each host's existing Navigation_Stack
  return; no second stack.
- Save: unchanged. The CR-CH-071 working-copy + SAVE/END/RETURN seam is reused;
  no second save path.
- Focus: unchanged. The one `WorkspaceContext` + `InteriorFocus` latch path is
  used; no per-arm focus ring.
- Persistence: unchanged. No new on-disk format; each host keeps its own file
  shape.
- Line-command engine: reused as-is. No change to the `ff-line-commands` public
  API or vocabulary.

Therefore this is NOT a framework change and needs no framework-change
confirmation. (The one schema QUESTION -- the Alias Editor's new Description
column -- is raised in the command-configurator spec, NOT here, because it is a
host-editor field concern, not a Table_Editor concern.)
