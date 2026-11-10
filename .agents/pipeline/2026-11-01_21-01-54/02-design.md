# Design -- Config View Tree Rendering (Requirement 22, Visual Redesign)

## Overview

This design re-skins the Configuration Context (`TabKind::ConfigPanel`, the
`CONFIG` key browser/editor in `crates/ff-desktop/src/config_panel/`) so it
renders as a clearly demarcated, best-practice two-level tree: a ruled header
area (Filter row + an `Unsaved edits : <yes|no>` row) separated from a tree
body of bracketed namespace group headers whose child key rows are drawn with
box-drawing tree connectors, with the `:` / value column vertically aligned
across all currently visible rows.

It is a VISUAL/representation change only. The pure `tree::visible_rows` model
(the single source of visible-row order), the per-namespace `collapsed`
expand/collapse state and its mouse/keyboard triggers, the per-key value widgets
and their working-copy edit semantics, the `WorkspaceContext` focus contract,
and the ISPF explicit-commit save model are all preserved unchanged. Nothing in
the `ff-config` crate changes; nothing in the `tree.rs` reducer, `widgets.rs`,
`commit.rs`, `keyboard.rs`, or the command/persistence seams changes. The change
is confined to how `config_panel/render.rs` draws each `ConfigRow`, plus one new
pure helper module for the column-alignment computation.

This adds a new **Requirement 22** to
`docs/specs/configuration-system/requirements.md` and a matching **Design Delta**
section to `docs/specs/configuration-system/design.md` (following that file's
established convention of appending UI-side deltas after the crate design). It
does not replace Requirements 15, 20, or 21.

## Technology stack (locked once approved)

- Language: Rust (edition as per the existing `ff-desktop` crate).
- GUI: `egui` / `eframe` 0.33 (workspace-pinned) -- unchanged from the rest of
  the Context.
- Alignment mechanism: **monospace character-count padding** (see "Column
  alignment" below), the SAME idiom already proven in this codebase for column
  alignment (`primary_option_menu/calendar_header.rs`, the POM calendar rows,
  the Macro Library header). No new dependency, no `egui::Grid`.
- Tests: `cargo test` unit tests for the pure layout helper;
  `egui_kittest` 0.33 (`eframe` feature) full-shell test for the focus contract.

The alignment mechanism is a deliberate choice between two viable options; it is
justified in "Column alignment -- choice of mechanism" below and is locked on
approval.

## What exists today (verified against the code)

- `config_panel/mod.rs` -- `ConfigPanelState` (fields: `filter`,
  `namespace_filter`, `collapsed: HashMap<String,bool>`, `pending`, `errors`,
  `working: HashMap<String,WorkingEdit>`, `persist_error`, `cursor`,
  `widget_ids`), `is_dirty()`, `stage_*`, `working_value`, and the
  `impl WorkspaceContext for ConfigPanelState` whose `render` delegates to
  `render::render` and returns `InteriorFocus::single(filter_field_id())`.
  `filter_field_id()` = `egui::Id::new("config_panel_filter")`.
- `config_panel/tree.rs` -- the pure, GUI-free model: `ConfigNodeId`
  (`Namespace(String)` | `Key(String)`), `ConfigRow { id, depth, expandable,
  expanded }`, `TreeEntry { key, description }`, `visible_rows(entries, filter,
  collapsed) -> Vec<ConfigRow>` (namespaces sorted, keys sorted within each
  EXPANDED group), the keyboard reducer `reduce_config_key`, and
  `reconcile_cursor`. **Unchanged by this design.**
- `config_panel/render.rs` -- `render(ui, state, config, requests)`: draws a
  Filter + Save horizontal bar, an optional persist-error line, a
  `ui.separator()`, an optional Source-File row + separator, builds
  `tree_entries` and `rows = visible_rows(...)`, clears `widget_ids`, reconciles
  the cursor + runs the keyboard driver, then a `ScrollArea` iterating `rows`:
  `Namespace` rows as a `selectable_label` with a `v`/`>` arrow prefix and
  `(count)` suffix; `Key` rows via `ui.indent(...)` into `render_entry` (a
  horizontal row of key/description, right-aligned provenance badge + staged `*`
  + Reset button, then the value widget below). No connectors, no column
  alignment today.
- `config_panel/widgets.rs` -- `render_widget(...) -> Option<egui::Id>` draws the
  type-appropriate value widget (checkbox / slider / combo / text) editing the
  working copy via `commit_value`, and returns the widget's `egui::Id`.
- `config_panel/keyboard.rs`, `commit.rs` -- keyboard driver and working-copy
  staging. **Unchanged.**
- The shell owns the `Command ===>` field as a top/bottom `TopBottomPanel`
  (`shell/render_command_line.rs`, id `command_field_input`) OUTSIDE the central
  panel; the Config Context renders BELOW it. So "Command ===>" in the user's
  mockup is the shell's existing command line, not a panel-owned widget
  (confirms requirements Assumption 3).
- `shell/render_body.rs` dispatches the Config arm through
  `render_workspace_context` (the single focus-latch path).
- `shell/tests_focus.rs::full_shell_config_first_tab_focuses_filter_field` -- the
  first-Tab conformance test to keep green and extend.

## Design decisions

### D1 -- Scope of the visual change: re-skin the row loop only

The redesign touches ONLY the body of `render::render` that draws each
`ConfigRow`, plus the Filter/dirty header block at the top. The upstream steps
(`visible_rows`, cursor reconcile, keyboard driver, `widget_ids` rebuild) and the
downstream per-key widget (`render_entry` -> `render_widget`, provenance badge,
staged `*`, Reset) keep their current call sites and behaviour. This is what
makes the change provably "visual only": the model and the edit controls are
reused verbatim; only their layout/decoration changes.

### D2 -- Demarcated header area (FR1, AC 22.1, 22.2)

The panel renders, in order, a HEADER BLOCK and then the TREE BODY, separated by
a single `ui.separator()` horizontal rule:

Header block (inside the panel, below the shell's `Command ===>` line):
- The Filter row: the existing `ff-desktop-config-panel-filter` label + the
  existing `TextEdit` carrying `filter_field_id()` + the existing `X` clear
  button. Unchanged widgets.
- A new `Unsaved edits : <yes|no>` row: a label built from `state.is_dirty()`
  (`yes` when dirty, `no` otherwise). This REPLACES the current right-aligned
  "unsaved changes" dot as the authoritative dirty indicator required by the
  mockup; the in-panel **Save** button is retained (it still enqueues
  `ShellRequest::Command("SAVE")`, the existing seam) and sits on the same row or
  immediately adjacent, so the explicit-commit affordance is unchanged (FR9).
- The optional persist-error line (unchanged, Req 11.4) stays within the header
  block, below the dirty row.
- The optional Source-File row (`ff-desktop-config-panel-source-file`) stays in
  the header block (it is header metadata, consistent with the mockup's header
  area).

Then exactly ONE `ui.separator()` rule divides the header from the tree body, so
the two areas are "clearly demarcated" (AC 22.1). The tree body is the
`ScrollArea` of rows.

Decision: the dirty indicator is text (`yes`/`no`) per the mockup rather than the
current coloured dot, because AC 22.2 requires the literal `Unsaved edits :
<yes|no>` row and ties it to `is_dirty()`. The coloured styling MAY be kept as an
accent on the value, but the text form is authoritative.

### D3 -- Two-level tree, bracketed group headers (FR2, FR5, AC 22.3)

The tree stays exactly two deep (group -> key), matching `visible_rows` (which
only ever emits `depth` 0 for `Namespace` and `depth` 1 for `Key`). Each
`Namespace` row renders as a bracketed header `[<DisplayName>]` using the
existing `ns_display_name` capitalisation, in the existing sorted order, with an
expand/collapse affordance. The key `(count)` suffix MAY be retained after the
bracket (e.g. `[Editor] (5)`); it is additive and does not affect alignment
because it sits on the group header row, not a key row.

The group header remains a clickable `selectable_label` that toggles
`collapsed[ns]` and sets the cursor -- the EXISTING mouse trigger, unchanged
(FR6, AC 22.8). The cursor highlight (`paint_cursor_highlight`) is retained.

### D4 -- Box-drawing child connectors (FR3, AC 22.4)

Under an expanded group, each key row is prefixed with a box-drawing connector
drawn in a monospace context. Because the tree is two-level and all keys of a
group render contiguously after the group header (guaranteed by `visible_rows`
ordering), the connector for a key is determined purely by whether it is the LAST
key of its group:

```
[Editor]
 ├─ editor.indent_style            : space
 ├─ editor.insert_final_newline    : true
 ├─ editor.line_endings            : lf
 ├─ editor.tab_size                : 4
 └─ editor.trim_trailing_whitespace: false
```

Connector glyphs (ASCII box-drawing range U+2500-U+257F, permitted inside code
blocks per documentation-charset.md):
- non-last child: U+251C U+2500 U+2500 (tee + two horizontals): `├──` then a space.
- last child: U+2514 U+2500 U+2500 (L-connector + two horizontals): `└──` then a space.

Since the model is two-level there is no intervening vertical-continuation bar
required BETWEEN a group's own keys at a deeper level; the leading-space +
tee/L idiom is the standard two-level tree-control form and satisfies AC 22.4
(branch-tee for every child except the last; L-connector for the last). The
design deliberately does NOT introduce the user's full outer `┌──┐` box frame --
requirements Assumption 1 confirms the mockup frame is illustrative; a demarcated
header/body (D2) plus connectors is the best-practice substance.

"Last child of group" is computed while iterating `rows`: a `Key` row is the last
child of its group when the NEXT row is not a `Key` (i.e. it is a `Namespace`
row or the end of the list). This is an O(1) look-ahead during the existing
single pass over `rows`; no change to `visible_rows`.

### D5 -- Column alignment (FR4, AC 22.5, 22.6, 22.12)

The `:` separator and the value must line up in one vertical column across all
currently visible key rows. The alignment is computed by a NEW pure function over
the visible-row data so it is unit-testable independently of egui (NFR4, AC
22.12):

```rust
// config_panel/layout.rs  (new, pure, GUI-free)

/// The label text drawn to the LEFT of the ':' column for a key row:
/// the box-drawing connector prefix + the full key path. Pure; no egui.
pub fn key_label(connector: Connector, key: &str) -> String { ... }

/// The connector kind for a key row (two-level tree).
pub enum Connector { Tee, Last } // ├── vs └──

/// The number of monospace CHARACTER cells the ':' column must sit at:
/// the max display width (char count) of the key_label over the CURRENTLY
/// VISIBLE key rows, plus one padding space. Group-header rows do not
/// participate (their ':' /value is not aligned). Returns 0 for an empty set.
pub fn value_column_width(visible_key_labels: &[String]) -> usize { ... }

/// Pad a key_label on the RIGHT with spaces to `column` cells, so appending
/// " : " + value lines the ':' up across rows. Pure string op.
pub fn pad_to_column(label: &str, column: usize) -> String { ... }
```

The render loop then, for each visible `Key` row, builds
`pad_to_column(key_label(connector, key), column)` and renders it as a single
`RichText::monospace` label; the `:` and the value widget are placed
immediately after, at the shared column, so they align. `column` is computed
ONCE per frame from the visible key labels gathered in a first cheap pass over
`rows` (or inline, see D6), so expand/collapse or filter changes automatically
recompute it for the new visible set (AC 22.6) -- the function is called fresh
each frame against the freshly built `rows`.

Width is measured in CHARACTER COUNT over a monospace font, which is why
monospace padding aligns pixels: equal char counts give equal pixel widths (the
exact property `calendar_header.rs` relies on). ASCII key paths and the box-
drawing connectors are all single-cell glyphs in the monospace face, so
`str::chars().count()` is the correct width measure.

Alignment scope (requirements Assumption 4): the column is the max over the
WHOLE currently-visible key set (one column for the panel), NOT per-group. This
matches the user's "aligned vertically" intent (every visible value in one
column) and is what the mockup shows across groups. The function takes exactly
the visible labels, so a collapsed group's long keys do not widen the column.

### D6 -- How the value widget coexists with the aligned `:` column (FR7, AC 22.9)

Each key row is laid out as a single `ui.horizontal(|ui| { ... })`:
1. A monospace `RichText` label = `pad_to_column(key_label(...), column)` + `":"`
   (the connector + padded key + the `:` at the shared column). This REPLACES the
   current vertical key/description stack as the row's left-hand content; the
   description is retained but moved to the label's `on_hover_text` (tooltip) or
   kept as a secondary small weak label on the same row AFTER the value when
   space allows. Decision: move the description to hover text so it does not
   disturb the single aligned column; this keeps the row one visual line (matching
   the mockup) while preserving the information (FR7 keeps the description
   available, just relocated).
2. The existing value widget via `render_widget(ui, state, config, entry,
   &effective)` placed right after the `:` column, returning its `egui::Id` which
   is still recorded into `state.widget_ids` (so Enter -> FocusKeyWidget, Req
   21.6, still works -- AC 22.9).
3. The existing right-aligned block (provenance/LOCKED badge, staged `*`, Reset
   button) via `ui.with_layout(right_to_left, ...)`, UNCHANGED.

So "value" in AC 22.5 is the key's value widget, placed at the computed column;
the provenance badge and Reset stay right-aligned (requirements Assumption 5).
The per-key editing behaviour, staged-edit `*`, LOCKED disabling, and Reset are
byte-for-byte the existing `render_entry` logic, only re-ordered into the aligned
single-row layout.

Note on widget widths: path-type string fields currently request
`f32::INFINITY` width and bounded ints/floats draw sliders. A full-width text
field or a slider does not have a fixed character width, so the `:` column aligns
the START of the value widget (the `:` and the widget's left edge line up); the
widget then extends rightward as today. This satisfies AC 22.5 ("the `:` column
and the value text are vertically aligned" -- the column is the alignment point)
without constraining widget widths, which keeps FR7 intact.

### D7 -- Preserve the focus contract (FR8, AC 22.10)

`impl WorkspaceContext::render` still returns
`InteriorFocus::single(filter_field_id())`; the Filter `TextEdit` still carries
`filter_field_id()`. The header re-layout keeps the Filter field as the first
interactive interior control, so the first Tab from the shell command field still
lands exactly on it (no phantom stop). The arm stays dispatched via
`render_workspace_context`; no per-arm focus ring is introduced. The existing
`full_shell_config_first_tab_focuses_filter_field` test is kept and remains the
proof (D-Test below).

Ordering caveat the implementation must honour: the Filter `TextEdit` must be the
FIRST widget allocated in the panel (it already is -- the header block is drawn
before the tree body), so egui's Tab order reaches it first. The new
`Unsaved edits` row and Source-File row are non-interactive labels (no Tab
stops), so they do not introduce a phantom stop ahead of the Filter field.

### D8 -- Preserve save convention (FR9, AC 22.11)

No change to SAVE/END/RETURN. The in-panel Save button keeps enqueuing
`ShellRequest::Command("SAVE")`. The new `Unsaved edits : <yes|no>` row is a pure
read of `state.is_dirty()` (AC 22.2). No autosave is introduced; edits still
stage into `working` via the unchanged `commit_value` path.

### D9 -- File-size split (NFR2)

`render.rs` is near the 400-line non-test budget. The new pure alignment helpers
land in a new `config_panel/layout.rs` (pure functions + their unit tests),
mirroring the existing `tree.rs` GUI-free split. `render.rs` keeps the egui
drawing but delegates width/label/padding computation to `layout.rs`. If the
re-skinned row-drawing pushes `render.rs` over budget, the per-row draw is
extracted into a `render_key_row(...)` helper (and, if still over, a
`config_panel/render_rows.rs` holding the group-header and key-row draw helpers).
The implementer must run the 400-line check and split before landing. `mod.rs`
gains `mod layout; pub use layout::{...}` re-exports as needed.

## File-by-file change plan

- `crates/ff-desktop/src/config_panel/layout.rs` -- NEW. Pure, GUI-free:
  `Connector`, `key_label`, `value_column_width`, `pad_to_column`, plus a
  `#[cfg(test)]` module unit-testing single-group, multi-group, filtered-subset,
  and empty-set column computation (AC 22.12).
- `crates/ff-desktop/src/config_panel/mod.rs` -- add `mod layout;` and any
  `pub use` the render needs. No state fields added (the column is computed
  per-frame, not stored).
- `crates/ff-desktop/src/config_panel/render.rs` -- re-skin only:
  - header block: Filter row (unchanged widgets) + new `Unsaved edits : yes|no`
    row + retained Save button + persist-error + Source-File, then ONE separator;
  - tree body: group rows as `[<DisplayName>]` headers (unchanged toggle), key
    rows drawn via the new aligned single-row layout using `layout.rs` helpers
    and the last-child look-ahead; `render_entry`'s widget/badge/Reset logic
    folded into the aligned row (same behaviour), `widget_ids` still recorded;
  - compute `column` per frame from visible key labels.
- `crates/ff-desktop/src/shell/tests_focus.rs` -- keep
  `full_shell_config_first_tab_focuses_filter_field`; optionally extend with an
  assertion that focus reaches the Filter field after the header re-layout
  (still the first interior stop).
- NO change to `tree.rs`, `widgets.rs`, `keyboard.rs`, `commit.rs`, the shell
  dispatch, the `ff-config` crate, `menus/settings.toml`, or persistence.

## Error handling and edge cases

The redesign is pure rendering; it introduces no new fallible I/O. The edge
cases to handle are layout/data shape, each with concrete behaviour:

- Empty visible set (filter matches nothing): `value_column_width([]) == 0`; the
  tree body renders no key rows (group rows may still render per `visible_rows`).
  No panic, no divide-by-anything. Unit-tested (AC 22.12 empty case).
- All groups collapsed: only `[<DisplayName>]` headers render; no key rows, so
  the column computation sees an empty key-label set and the body is just
  headers. Recomputes correctly on the next expand (AC 22.6).
- A very long value (e.g. a path like
  `C:\Users\...\ffworkbench\catalogs\mainframe`): the `:` column is positioned by
  the widest KEY label, not the value; a long value simply extends rightward
  inside the `ScrollArea` (horizontal scroll if needed), exactly as paths render
  today. The column stays aligned.
- A very long key relative to its siblings widens the shared column for the whole
  visible set (whole-visible-set alignment, D5); this is intended and matches the
  mockup. It is not an error.
- Non-ASCII in a key: schema keys are ASCII dotted paths by construction; the
  char-count width measure assumes single-cell glyphs. If a non-ASCII key ever
  appeared, the measure would be char-count (not grapheme/width aware); this is an
  accepted limitation documented here, not a failure path. The box-drawing
  connectors themselves are single-cell in egui's monospace face.
- LOCKED key: value widget stays disabled (unchanged `add_enabled_ui(!locked)`),
  provenance badge shows `LOCKED`; the aligned layout does not change this.
- Keyboard cursor on a row whose layout changed: the cursor model is unchanged
  (`tree.rs`); the highlight still paints around the row rect via
  `paint_cursor_highlight`. Expand/collapse via keyboard rebuilds `rows` next
  frame and recomputes the column (AC 22.6).

Validation of external input: the only user input in scope is the Filter text
(unchanged) and per-key value edits (unchanged, validated by the existing
`commit_value` into `errors`). The redesign adds no new input to validate.

## Invariant ownership

- Visible-row order + group membership: owned by `tree::visible_rows` (unchanged).
  The render MUST NOT reorder or re-group; it consumes `rows` as given. This keeps
  FR5 / AC 22.7 (grouping frozen) enforced at the model layer.
- Last-child connector correctness: owned by the render look-ahead over `rows`,
  valid BECAUSE `visible_rows` guarantees a group's keys are contiguous and
  immediately follow the group header. The design depends on that invariant and
  states it explicitly; if `visible_rows` ever interleaved groups the look-ahead
  would break -- but that is out of scope and frozen.
- Column position: owned by the pure `layout::value_column_width` over the
  per-frame visible key labels; the render only consumes the returned column. This
  isolates the alignment rule for unit testing (NFR4).
- Focus first-interior anchor: owned by `impl WorkspaceContext::render` returning
  `InteriorFocus::single(filter_field_id())` and dispatched via
  `render_workspace_context` (the single latch path). The render MUST allocate the
  Filter field first so egui Tab order agrees (D7).
- Dirty state: owned by `ConfigPanelState::is_dirty()` (unchanged); the header row
  only reads it.

## Testability

- Unit (pure, no egui): `layout.rs` -- `value_column_width` over single group,
  multiple groups, a filtered subset, and the empty set (AC 22.12);
  `key_label`/`pad_to_column` producing the expected connector + padded strings;
  last-child connector selection given a `rows` slice. These mirror the existing
  `tree.rs` pure tests.
- Integration (full-shell `egui_kittest`): keep
  `full_shell_config_first_tab_focuses_filter_field` green -- open `CONFIG`, assert
  `first_interior_id == filter_field_id()`, assert first Tab from the command
  field focuses the Filter field with no phantom stop (FR8 / AC 22.10). This is the
  mandatory GUI behaviour test (workspace-conformance.md).
- What is NOT unit-testable and why: the pixel-exact rendering of box-drawing
  connectors and the on-screen column position are visual; they are covered
  indirectly (the pure width function is unit-tested; the connectors are a
  deterministic string from `key_label`). Pixel-exact appearance is the justified
  manual/visual exception per testing.md.

Agent verification is COMPILE-ONLY per the testing.md execution split: scoped
`cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D warnings`,
`cargo fmt -p ff-desktop -- --check`, and the single new/changed scoped test
where quick (`cargo test -p ff-desktop <name>`). The owner runs the authoritative
full gate. Logs are written under `_ignore/logs/` and read back (verify by
effect) using the pwsh7 non-interactive wrapper.

## Out of scope (restating requirements)

- Changing namespace derivation / group membership (`namespace_of`,
  `visible_rows`) -- frozen.
- Changing config persistence, layering, validation, or SAVE/END/RETURN flush.
- Changing the keyboard navigation transition table (Requirement 21) -- only its
  on-screen representation is re-skinned.
- New config keys, groups, or editing widgets.
- Any change to `menus/settings.toml` or `CONFIG` resolution (Requirement 20).
- Multi-level nesting below the two-level group -> key model.
- Drawing the user's full outer `┌──┐` box frame (illustrative only, Assumption 1).

## Alignment with requirements assumptions

- A1 (mockup illustrative): honoured -- no outer box frame; best-practice
  demarcated header + connectors + alignment (D2, D4).
- A2 (monospace for alignment): chosen and justified -- monospace char-count
  padding, not `egui::Grid` (D5, "choice of mechanism" below).
- A3 (Command ===> ownership): confirmed from code -- the shell owns the command
  line as a `TopBottomPanel`; the panel header sits below it, the demarcation rule
  is drawn inside the panel between the Filter/dirty header and the tree body (D2).
- A4 (alignment scope = visible set): adopted -- whole-visible-set single column
  (D5).
- A5 (Reset/provenance to the right): retained -- right-aligned block unchanged;
  value widget starts at the aligned column (D6).

## Column alignment -- choice of mechanism (the one open option, decided)

Two viable mechanisms were considered for aligning the `:` column:

1. `egui::Grid` with a key column and a value column. REJECTED: the keys_editor
   panel already documents that a `TextEdit`'s `desired_width` is unreliable
   inside an `egui::Grid` nested in a horizontal `ScrollArea` (cells collapse);
   the config value widgets include full-width text fields and sliders, which
   would fight a grid's column sizing. A grid also makes the box-drawing connector
   prefix awkward (it wants to live in the same cell as the key).

2. **Monospace character-count padding (CHOSEN).** Build the connector + key as
   one monospace string padded to the max visible key-label width, then place the
   `:` and value widget after it. This is the idiom already proven in this
   codebase (`calendar_header.rs`, the POM calendar, Macro Library headers): equal
   monospace char counts give equal pixel widths, so the `:` lands in the same
   column on every row. It keeps the connector and key in one label, needs no new
   dependency, keeps the value widgets at their natural widths, and the width rule
   is a pure, unit-testable function (NFR4). This is locked.
