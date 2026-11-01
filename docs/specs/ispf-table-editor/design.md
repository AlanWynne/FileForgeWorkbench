# Design -- Shared ISPF Table Editor + Line-Command Component (CR-CH-075, extended by CR-CH-076 / 077 / 078)

> CR-CH-076 (block line commands RR/DD/CC/MM), CR-CH-077 (single-chrome layout:
> live panel and snapshot agree), and CR-CH-078 (full arrow-key focus
> navigation) EXTEND this shared component. Their design is in the dedicated
> sections near the end of this document ("CR-CH-076 -- Block line commands ...",
> "CR-CH-078 -- The component owns the arrow walk ...", "CR-CH-077 --
> Single-chrome ...", and the reusability principle). All three build ON the
> seams below with no framework or `ff-line-commands`-API change.

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

## CR-CH-076 -- Block line commands (RR / DD / CC / MM): the engine investigation

### Investigation verdict (REQUIRED by the brief): BUILD ON, no engine API change

The brief required settling, during the gate, whether `crates/ff-line-commands`
ALREADY parses the doubled block form and the paired-range semantics, or whether
CR-CH-076 needs a (framework-adjacent) engine API extension. The crate was read
end to end (`parser.rs`, `command.rs`, `block_pair.rs`, `pending.rs`,
`compatibility.rs`). VERDICT: the engine ALREADY models blocks fully; CR-CH-076
builds ON it with NO public-API change and NO new parser. Evidence:

- **Parser recognises the doubled form.** `LineCommandParser::parse` matches the
  doubled tokens FIRST, case-insensitively: `"DD" -> DeleteBlock`,
  `"RR" -> RepeatBlock`, `"CC" -> CopyBlock`, `"MM" -> MoveBlock` (plus XX / TT /
  UU / >> / << / (( / )) / WW / ]] which the table editors do NOT enable). Tests
  `parse_dd_delete_block`, `parse_rr_repeat_block`, `parse_cc_copy_block`,
  `parse_mm_move_block`, and `parse_mixed_case_works` (`"Dd" -> DeleteBlock`)
  already prove this.
- **Classification distinguishes blocks.** `command::classify` returns
  `LineCommandCategory::Block` for the `*Block` kinds; `is_block_marker` and
  `to_block_kind` map a block kind to its `BlockCommandKind`. The single letters
  stay `Immediate` (`R` / `D`) or `Source` (`C` / `M`), so `R`/`Rn` vs `RR`
  (Requirement 9.2) is already a type-level distinction, not a heuristic.
- **Pairing forms an inclusive, order-normalised range.**
  `BlockPairValidator::validate_pair` reads the two pending markers of a block
  kind and returns a `BlockPair { kind, start_line, end_line }` with
  `normalize(a, b)` so the smaller index is `start_line` regardless of entry
  order (Requirement 9.1). `check_overlaps` rejects overlapping ranges.
- **Unterminated block is already a typed signal.** With 0 or 1 marker,
  `validate_pair` returns `LineCommandError::AwaitingPair { kind }`; with > 2 it
  returns `TooManyMarkers`. These are EXACTLY the "block not closed" / malformed
  signal Requirement 9.4 needs -- the adapter maps them to a non-blocking notice
  and retains the marker, applying no structural change. `PendingCommandStore`
  with `PendingReason::AwaitingPair` already holds a lone marker across a cycle.

Therefore CR-CH-076 is NOT a framework change and needs no owner confirmation of
an engine-API change. (The brief asked that an API extension, if needed, be
flagged for owner confirmation; the investigation shows none is needed, so this
is recorded as a NON-issue rather than an open decision.)

### Where the change actually lands (the adapter, not the engine)

The gap is entirely in the shared component's apply adapter
(`crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs`). Its
`apply_line_commands` TODAY classifies every `LineCommandCategory::Block` as
"not enabled" and clears it with a notice (the `LineCommandCategory::Block` arm,
and `command_enabled` has no block flags). CR-CH-076 EXTENDS this adapter to:

1. Add a `blocks: bool` (or an explicit RR/DD/CC/MM sub-flag set) to
   `LineCommandSet` (`state.rs`), advertised per-host (Requirement 9.5). This is
   a new field on the shared component's own type, NOT an engine change.
2. In the parse/classify pass, when `blocks` is enabled, COLLECT the block
   markers per `BlockCommandKind` (keyed by backing index) instead of clearing
   them; feed them to `BlockPairValidator::validate_pair` to form each
   `BlockPair`.
3. Translate a resolved `BlockPair` into the SAME "stable-original-index"
   rebuild the adapter already uses for single commands:
   - `DeleteBlock`: mark `removed[start..=end]`.
   - `RepeatBlock`: clone each row in `start..=end` once into its own `after[i]`.
   - `CopyBlock` + `A`/`B`: snapshot clones of `start..=end` into the target's
     `before`/`after` list (Requirement 9.3), exactly like a single `C`.
   - `MoveBlock` + `A`/`B`: same as `CopyBlock` plus `removed[start..=end]`
     (Requirement 9.3).
4. On `AwaitingPair` / `TooManyMarkers`, push a "block not closed" notice onto
   `ApplyResult.notices` and RETAIN the marker (do not clear its Act cell), so
   the user supplies the partner next cycle (Requirement 9.4). No structural
   change is made for that block; the rest of the batch proceeds.

The adapter's existing rebuild-by-walk model (per-row `before` / `after` /
`removed` over STABLE original indices) already makes a mixed batch immune to
index shifting, so a block op batched with single ops composes without extra
machinery. The adapter stays PURE and egui-free, so block application is
unit-testable over the stub `Vec<Row>` exactly as the single commands are.

### Error handling (blocks, concrete)

- **Unterminated block (one XX of a pair, or > 2 markers):** recoverable.
  Non-blocking "block not closed" notice; the marker(s) are RETAINED; no
  structural change for that block; other commands in the batch still apply
  (Requirement 9.4).
- **Overlapping block ranges:** recoverable. `check_overlaps` rejects; a
  non-blocking notice is surfaced and neither overlapping block is applied.
- **`CC` / `MM` block with no A/B target:** recoverable. The block is RETAINED
  with a "copy/move source pending a target (A/B)" notice (same shape as a single
  unresolved source, Requirement 9.3 / 4.4).
- **A doubled marker when the host has NOT enabled blocks:** cleared and ignored
  with a non-blocking notice, unchanged from today (Requirement 9.5 / 4.5).

## CR-CH-078 -- The component owns the arrow walk without fighting egui

egui provides NO vertical / grid arrow-key focus traversal for a field matrix: by
default only Tab / Shift+Tab walk focus, in widget-creation order. So full
arrow navigation (Requirement 11) MUST be owned by the shared component, and it
must be implemented the SAME way the menu-bar keynav fix was, or it will fight
egui's own focus handling (the documented B-fix in
`crates/ff-desktop/src/shell/render_chrome.rs` /
`render_menu_bar.rs`: the arrows are CONSUMED with
`ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Arrow*))` BEFORE egui's
MenuBar renders, because egui's own navigation is unreliable and a late consume
is too late).

DESIGN: the component maintains, per frame, a FIELD MATRIX descriptor -- the
ordered list of focusable cells as (row backing-index, column index) -> stable
`egui::Id`. It already renders every cell with a stable salted id keyed by the
backing index (the `surface_id(i)` pattern, Requirement 7.2), so the matrix is
just those ids arranged by (row, column). On each frame, AFTER laying out the
cells and knowing which id currently holds focus
(`ui.ctx().memory(|m| m.focused())`), the component:

1. Determines the focused cell's (row, column) from the matrix (if focus is on a
   matrix cell -- not the `Command ===>` line, Requirement 11.5).
2. CONSUMES the relevant arrow key with `input_mut(..).consume_key(..)` so egui
   does not also act on it:
   - Down -> target (row+1, column), clamped to the last row; Up -> (row-1,
     column), clamped to the first row (Requirement 11.1).
   - Right -> next focusable column in the same row; Left -> previous; clamped at
     the row's first / last column (Requirement 11.2).
3. Calls `request_focus` (via `ui.ctx().memory_mut(|m| m.request_focus(id))` or
   the target `Response`) on the target cell's stable id.

Because the keys are consumed BEFORE egui's traversal and the move is expressed
as a direct `request_focus` on a stable id, the walk does not fight egui (the
menu-bar lesson). Tab / Shift+Tab are UNTOUCHED: they still flow through the
single `WorkspaceContext` + `InteriorFocus` latch in visual order (Requirement
7.1, 7.2), and the per-row salted ids the full-shell first-Tab tests assert are
UNCHANGED (Requirement 11.4) -- the arrow walk only adds a focus-move on the
arrow keys, it does not re-order or re-id any control. The `Command ===>` field
keeps its CR-NR-096 Up/Down history gesture because the arrow walk only fires
when a MATRIX cell holds focus, never the command line (Requirement 11.5); the
key is owned by whichever control has focus.

TESTABILITY: the matrix-walk target computation (given a focused (row, column)
and an arrow direction, compute the clamped target (row, column)) is PURE and
unit-testable without egui -- the clamp-at-edges rules (Requirement 11.1, 11.2)
are table-driven tests. The end-to-end focus move (first Down from the Act cell
lands on the Act cell of the next row) is proven by a full-shell `egui_kittest`
test per the GUI-behaviour rule (workspace-conformance.md), modelled on the
existing first-Tab tests.

## CR-CH-077 -- Single-chrome: remove the host's duplicate draw, fill the snapshot

The concern (concerns.md 1-3) is that the Alias Context draws its OWN title line
and `Command ===>` line ON TOP OF the shared component's, so the live panel shows
TWO title lines and TWO command lines; and separately the markdown snapshot
OMITS the header and command line entirely, so the live panel and the snapshot
disagree.

DESIGN -- the component is the single owner of the chrome:

- The shared `render_ispf_table` already renders ONE `Command ===>` line (the
  entry control, Requirement 2.1) and ONE header row (Requirement 1.1). The fix
  is to ensure the HOST Context arm renders NEITHER a second title/heading NOR a
  second command line. Concretely, the Alias Context's render arm currently emits
  an extra body heading ("FileForge Workbench Alias Editor") and/or an extra
  command line in addition to calling the component; the fix REMOVES that
  duplicate draw from the host arm. The remaining single Title line is the
  Context / Kind descriptive title drawn by the shell's standard Title_Line
  (CR-NR-090 / CR-CH-045), which is distinct from the component's `Command ===>`
  line and from the `[XXX]` Tab_Header tag. No new chrome mechanism is added; a
  duplicate is removed (Requirement 10.1, 10.2).

- The snapshot side is reconciled by the `ScreenProvider` building the SAME
  single-chrome layout: the provider's `ff-screen-model` output MUST include the
  `Command ===>` line and the column header row (today it emits only the title +
  rows, concern 3). Because the provider already reads the host's `ColumnSpec`
  order (Requirement 7.3), the header row it emits matches the live header; the
  fix adds the `Command ===>` line and the header row to the provider's emitted
  screen so the captured text/markdown shows Title -> `Command ===>` -> header ->
  rows, AGREEING with the live panel (Requirement 10.3).

This is a render-and-snapshot reconciliation on existing seams only: it changes
NO dispatch, navigation, descriptor, effect pattern, or save path (Requirement
10.4). It touches the host Context render arm (remove the duplicate) and the
host's `ScreenProvider` (add the missing chrome) -- both host-side, using the
component's already-owned single chrome as the single source of truth.

## Reusability principle (threads through CR-CH-076 / 077 / 078)

All three changes live in the SHARED component (or, for blocks, in the shared
component's adapter plus the already-shared `ff-line-commands` engine), NOT bolted
onto the Alias panel. Block commands (Requirement 9), single-chrome ownership
(Requirement 10), and arrow navigation (Requirement 11) are therefore inherited
by EVERY host that consumes the component -- Alias, Menu, Kinds, and any future
ISPF workspace -- with no per-host re-implementation. A host opts into blocks via
its Line_Command_Set; single-chrome and arrow navigation are automatic once a
host renders through `render_ispf_table` and reports focus through the one latch
path. This is the owner's "the pattern must be copyable/reusable into other ISPF
workspaces" intent, realised by keeping the behaviour in the shared surface and
engine rather than in any one editor.

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
- Line-command engine: reused as-is, INCLUDING its block model. The CR-CH-076
  block commands build on `LineCommandParser` / `BlockPairValidator` / `BlockPair`
  with NO change to the `ff-line-commands` public API or vocabulary (see the
  investigation verdict above).
- Focus (CR-CH-078 arrow walk): additive on the existing `WorkspaceContext` +
  `InteriorFocus` latch; Tab / Shift+Tab and the stable first-Tab ids are
  unchanged. The arrow-key consume-before-egui pattern is the same one the
  menu-bar keynav already uses; it is not a new focus mechanism.
- Chrome (CR-CH-077 single-chrome): removes a duplicated host draw and fills the
  snapshot's missing chrome; no new chrome mechanism, no dispatch / navigation /
  save-path change.

Therefore CR-CH-075, CR-CH-076, CR-CH-077, and CR-CH-078 are all NOT framework
changes and need no framework-change confirmation. The ONLY owner-confirm point
remaining in this spec area is the additive optional `description` field on the
alias row (flagged in command-configurator Req 7, carried over from CR-CH-075),
which is a persisted-shape question, not a framework change. (The one schema QUESTION -- the Alias Editor's new Description
column -- is raised in the command-configurator spec, NOT here, because it is a
host-editor field concern, not a Table_Editor concern.)
