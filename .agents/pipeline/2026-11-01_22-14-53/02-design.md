# Design -- Config View Tree Rendering (Requirement 22, Visual Redesign)

> Imported from the prior run's design
> (`.agents/pipeline/2026-11-01_21-01-54/02-design.md`) and CORRECTED to resolve
> the prior design review's 3 MEDIUM and 4 NIT findings per user-APPROVED
> corrections. The overriding rule is: this is a VISUAL-ONLY re-skin -- the
> behaviour of individual nodes (selectable key name, visible selectable
> description, value widget, badge, Reset, focus, save) does NOT change; only the
> tree representation and column alignment change. Responses to every review
> finding are recorded at the end of this document.

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
and their working-copy edit semantics, the SELECTABLE key name, the SELECTABLE
always-visible key description, the `WorkspaceContext` focus contract, and the
ISPF explicit-commit save model are all preserved unchanged. Nothing in the
`ff-config` crate changes; nothing in the `tree.rs` reducer, `widgets.rs`,
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
  alignment in `primary_option_menu/calendar_header.rs`
  (`centered_calendar_header`, which pads a label to a constant monospace width
  so columns line up). No new dependency, no `egui::Grid`.
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
  collapsed) -> Vec<ConfigRow>` (namespaces sorted via `BTreeMap`, keys sorted
  within each EXPANDED group; depth 0 for `Namespace`, depth 1 for `Key`), the
  keyboard reducer `reduce_config_key`, and `reconcile_cursor`. **Unchanged by
  this design.**
- `config_panel/render.rs` -- `render(ui, state, config, requests)`: draws a
  Filter + Save horizontal bar (Filter label, the `TextEdit` carrying
  `filter_field_id()`, an `X` clear button, then a right-to-left Save block with
  a coloured "unsaved changes" dot), an optional persist-error line, a
  `ui.separator()`, an optional Source-File row + separator, builds
  `tree_entries` and `rows = visible_rows(...)`, clears `widget_ids`, reconciles
  the cursor + runs the keyboard driver, then a `ScrollArea` iterating `rows`:
  `Namespace` rows as a `selectable_label` with a `v`/`>` arrow prefix and
  `(count)` suffix; `Key` rows via `ui.indent(...)` into `render_entry`. No
  connectors, no column alignment today. The file is 407 lines TOTAL with the
  `#[cfg(test)]` module starting at line 298, so NON-TEST code is ~297 lines.
- `render_entry` (in `render.rs`) draws, inside one `ui.horizontal`:
  - a `ui.vertical` stack of TWO `egui::Button::selectable(false, ...)` widgets
    -- the key name (`RichText::monospace`) and the description
    (`RichText::small().weak()`), annotated `// Validates: Requirement 14.2 --
    selectable key name and description`; BOTH are selectable, always-visible
    widgets today;
  - a right-to-left block: provenance/LOCKED badge, a staged-edit `*`, and the
    `Reset` button (`add_enabled_ui(reset_enabled, ...)`);
  - then the value widget via `render_widget(...) -> Option<egui::Id>`, whose id
    is recorded into `state.widget_ids` (so Enter -> `FocusKeyWidget`, Req 21.6);
  - then an optional inline validation error line.
- `config_panel/widgets.rs` -- `render_widget(...) -> Option<egui::Id>` draws the
  type-appropriate value widget (checkbox / slider / combo / text) editing the
  working copy via `commit_value`, and returns the widget's `egui::Id`.
  Path-type string fields request `f32::INFINITY` width; bounded numbers draw
  sliders. **Unchanged.**
- `config_panel/keyboard.rs`, `commit.rs` -- keyboard driver and working-copy
  staging. **Unchanged.**
- The shell owns the `Command ===>` field as a `TopBottomPanel`
  (`shell/render_command_line.rs`, widget id `command_field_input`) OUTSIDE the
  central panel; the Config Context renders BELOW it. So "Command ===>" in the
  user's mockup is the shell's existing command line, not a panel-owned widget
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
downstream per-key widgets (value widget via `render_widget`, provenance badge,
staged `*`, Reset, the selectable key name, the selectable description) keep
their current call sites and behaviour. This is what makes the change provably
"visual only": the model and the edit controls are reused verbatim; only their
layout/decoration changes.

### D2 -- Demarcated header area (FR1, AC 22.1, 22.2); Save + dirty row placement (resolves MEDIUM-3)

The panel renders, in order, a HEADER BLOCK and then the TREE BODY, separated by
a single `ui.separator()` horizontal rule. The header block sits inside the panel,
below the shell's `Command ===>` line (which the shell owns, Assumption 3).

The header block rows, in EXACTLY this allocation order:

1. **Filter + Save row (unchanged allocation order).** The existing
   `ui.horizontal` is kept with its existing allocation order so the focus
   contract is not disturbed:
   - FIRST, the Filter label (`ff-desktop-config-panel-filter`), then the Filter
     `egui::TextEdit` carrying `filter_field_id()`, then the `X` clear button;
   - THEN, in the right-to-left sub-layout, the **Save** button (still enqueuing
     `ShellRequest::Command("SAVE")`, the existing seam, enabled only when
     `is_dirty()`). The existing coloured "unsaved changes" dot MAY be retained
     as an accent, but it is NOT the authoritative indicator (that is the dirty
     row below).
   Because the Filter `TextEdit` is allocated BEFORE the Save button (exactly as
   the current code does inside this `ui.horizontal`), egui's native Tab order
   reaches the Filter field first; the Save button never becomes a phantom-ahead
   Tab stop (this is the pinned-down placement that preserves AC 22.10 -- see D7).
2. **`Unsaved edits : <yes|no>` row (new, non-interactive).** Its OWN row, a
   single non-interactive label built from `state.is_dirty()` (`yes` when dirty,
   `no` otherwise), rendered BELOW the Filter+Save row. Being a label, it
   introduces NO Tab stop. This is the authoritative indicator AC 22.2 ties to
   `is_dirty()`.
3. **Optional persist-error line** (unchanged, Req 11.4) -- a non-interactive
   coloured label, within the header block, below the dirty row.
4. **Optional Source-File row** (`ff-desktop-config-panel-source-file`) -- header
   metadata, non-interactive, kept in the header block.

Then exactly ONE `ui.separator()` rule divides the header from the tree body, so
the two areas are "clearly demarcated" (AC 22.1). The tree body is the
`ScrollArea` of rows.

Decision (MEDIUM-3 pinned down): the Save button stays WHERE IT IS TODAY --
after the Filter `TextEdit` in allocation order, inside the first header row's
right-to-left block. The `Unsaved edits : <yes|no>` indicator is a SEPARATE
non-interactive label row below it. Rows 2-4 are all non-interactive labels, so
the only interior Tab stop ahead of the tree is the Filter field. This is the
single chosen layout (not "same row or adjacent"); it keeps the Filter field the
first interior control and makes the full-shell first-Tab test pass with no
phantom stop.

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
(FR6, AC 22.8). An expand/collapse arrow glyph (`v`/`>`, as today) MAY be shown
before the bracket. The cursor highlight (`paint_cursor_highlight`) is retained.

### D4 -- Box-drawing child connectors (FR3, AC 22.4)

Under an expanded group, each key row is prefixed with a box-drawing connector
drawn in a monospace context. Because the tree is two-level and all keys of a
group render contiguously after the group header (guaranteed by `visible_rows`
ordering), the connector for a key is determined purely by whether it is the LAST
key of its group:

```
[Editor]
 ├── editor.indent_style             : space
 ├── editor.insert_final_newline     : true
 ├── editor.line_endings             : lf
 ├── editor.tab_size                 : 4
 └── editor.trim_trailing_whitespace : false
```

Connector glyphs (ASCII box-drawing range U+2500-U+257F, permitted inside code
blocks per documentation-charset.md):
- non-last child: U+251C U+2500 U+2500 (tee + two horizontals), rendered as the
  three-char prefix `├──` followed by a space.
- last child: U+2514 U+2500 U+2500 (L-connector + two horizontals), rendered as
  the three-char prefix `└──` followed by a space.

The two prefixes are the SAME character width (four monospace cells including the
trailing space), so they do not perturb the column alignment computed in D5.

FR3 also lists "the vertical continuation bar for intermediate levels". That form
does NOT apply here: the tree is strictly two-level (`visible_rows` emits only
depth 0 and depth 1 -- verified in `tree.rs`), so there are no intermediate
levels that would need a continuation bar. Only the branch-tee and L-connector
are drawn, which is exactly what AC 22.4 enumerates. This is an explicit,
justified decision, not an omission (resolves NIT-6). The design deliberately
does NOT draw the user's full outer `┌──┐` box frame -- requirements Assumption 1
confirms the mockup frame is illustrative; a demarcated header/body (D2) plus
connectors is the best-practice substance.

"Last child of group" is computed while iterating `rows`: a `Key` row is the
last child of its group when the NEXT row is not a `Key` (i.e. it is a
`Namespace` row or the end of the list). This is an O(1) look-ahead during the
existing single pass over `rows`; no change to `visible_rows`.

### D5 -- Column alignment (FR4, AC 22.5, 22.6, 22.12)

The `:` separator and the value must line up in one vertical column across all
currently visible key rows. The alignment is computed by a NEW pure function over
the visible-row data so it is unit-testable independently of egui (NFR4, AC
22.12):

```rust
// config_panel/layout.rs  (new, pure, GUI-free)

/// The connector kind for a key row in the two-level tree.
pub enum Connector { Tee, Last } // ├── vs └──

/// The three-cell connector prefix glyphs plus a trailing space (both forms
/// are the same monospace width, so they do not perturb alignment).
pub fn connector_prefix(c: Connector) -> &'static str { /* "\u{251c}\u{2500}\u{2500} " | "\u{2514}\u{2500}\u{2500} " */ }

/// The label text drawn to the LEFT of the ':' column for a key row:
/// the box-drawing connector prefix + the full key path. Pure; no egui.
pub fn key_label(connector: Connector, key: &str) -> String { ... }

/// The number of monospace CHARACTER cells the ':' column must sit at:
/// the max char-count of key_label over the CURRENTLY VISIBLE key rows, plus
/// one padding space. Group-header rows do not participate. Returns 0 for the
/// empty set.
pub fn value_column_width(visible_key_labels: &[String]) -> usize { ... }

/// Pad a key_label on the RIGHT with spaces to `column` cells, so appending
/// " : " + value lines the ':' up across rows. Pure string op.
pub fn pad_to_column(label: &str, column: usize) -> String { ... }
```

The render loop then, for each visible `Key` row, builds
`pad_to_column(key_label(connector, key), column)` and renders it as a single
`RichText::monospace` label (see D6 for how the selectable key-name and
selectable description are preserved alongside this aligned label); the `:` and
the value widget are placed immediately after, at the shared column, so they
align. `column` is computed ONCE per frame from the visible key labels gathered
in a cheap first pass over `rows`, so expand/collapse or filter changes
automatically recompute it for the new visible set (AC 22.6) -- the function is
called fresh each frame against the freshly built `rows`.

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

### D6 -- Preserving the selectable key name + visible selectable description alongside the aligned column (FR7, AC 22.9; resolves MEDIUM-1 and MEDIUM-2)

This is the corrected core of the re-skin. The overriding rule (user-approved)
is: the key name stays a SELECTABLE control and the key description stays a
VISIBLE, SELECTABLE widget exactly as today -- neither is moved to hover/tooltip,
and neither is downgraded to inert text. The alignment is achieved with
PADDING/LAYOUT AROUND the existing selectable widgets, not by replacing them.

Concrete layout per key row, in one `ui.horizontal(|ui| { ... })`:

1. **Connector prefix label (inert).** A non-interactive `RichText::monospace`
   label holding ONLY the box-drawing connector prefix (`├── ` or `└── `, D4).
   This is decoration, correctly inert.
2. **Selectable key-name control (UNCHANGED selectable widget).** The existing
   `egui::Button::selectable(false, RichText::new(key).monospace())` is RETAINED
   exactly as today -- it stays a selectable control (Req 14.2 preserved,
   MEDIUM-2 resolved). It is NOT downgraded to a plain `RichText` label.
3. **Alignment padding (inert spacer).** To make the `:` column line up without
   downgrading the selectable key button, the row pads the gap AFTER the
   selectable key control up to the shared column with an inert monospace
   spacer label: the key button renders at its natural width, then an inert
   `RichText::monospace` label of
   `pad_to_column("", column - char_width(connector + key))` spaces fills to the
   column. Because the key text is drawn monospace in BOTH the button and the
   spacer, the char-count padding yields equal pixel widths, so the `:` lands in
   the same column on every row (the `calendar_header.rs` property). The pure
   `layout.rs` functions compute the char-count target column and the pad length;
   the selectable widget is untouched. (The selectable button's own frame insets
   are constant per row, so they shift the column uniformly and do not break
   alignment; the implementer confirms this with the first-Tab/visual check and,
   if the button frame proves non-uniform in practice, switches the key control
   to a frameless `selectable_label` sized to the padded monospace text -- still
   selectable, still Req 14.2 -- which is the fallback the alignment property
   needs. Either way the key control REMAINS selectable.)
4. **The `:` separator** rendered as an inert monospace `":"` at the shared
   column.
5. **The value widget** via `render_widget(ui, state, config, entry,
   &effective)` placed right after the `:` column, returning its `egui::Id`
   which is still recorded into `state.widget_ids` (so Enter -> FocusKeyWidget,
   Req 21.6, still works -- AC 22.9). UNCHANGED.
6. **The right-aligned block** (provenance/LOCKED badge, staged `*`, Reset
   button) via `ui.with_layout(right_to_left, ...)`, UNCHANGED.

The **selectable, always-visible key description** (MEDIUM-1 resolved) is
RETAINED as a visible, selectable widget -- it is NOT moved to hover text. It is
rendered as the existing `egui::Button::selectable(false,
RichText::new(entry.description).small().weak())` on a SECOND line beneath the
aligned key row (a two-line key row), OR as a trailing weak selectable label on
the same row after the value widget. Decision: render it on the SECOND line
beneath the key row, because that keeps the first (aligned) line a clean single
key-to-value row for the column math while preserving the description exactly as
a visible, selectable widget. Crucially, the column-alignment computation in D5
is derived ONLY from the first line's `key_label` (connector + key); the
description's second line does not participate in and does not perturb the `:`
column.

Net effect: every existing per-key control -- selectable key name, visible
selectable description, value widget, provenance/LOCKED badge, staged `*`, Reset
-- is preserved with UNCHANGED behaviour (FR7, AC 22.9). The only change is the
connector prefix, the inert alignment padding, and the `:`/value column
placement. No behaviour of any individual node changes; this is a visual-only
re-skin (MEDIUM-1 and MEDIUM-2 fully resolved).

Note on widget widths: path-type string fields currently request
`f32::INFINITY` width and bounded ints/floats draw sliders. A full-width text
field or a slider does not have a fixed character width, so the `:` column
aligns the START of the value widget (the `:` and the widget's left edge line
up); the widget then extends rightward as today. This satisfies AC 22.5 ("the
`:` column and the value text are vertically aligned" -- the column is the
alignment point) without constraining widget widths, which keeps FR7 intact.

### D7 -- Preserve the focus contract (FR8, AC 22.10)

`impl WorkspaceContext::render` still returns
`InteriorFocus::single(filter_field_id())`; the Filter `TextEdit` still carries
`filter_field_id()`. Per D2, the Filter `TextEdit` is allocated FIRST in the
header block (before the Save button, exactly as the current code orders the
first `ui.horizontal`), and every other header row (dirty indicator, persist
error, source file) is a non-interactive label. So the first Tab from the shell
command field still lands exactly on the Filter field (no phantom stop). The arm
stays dispatched via `render_workspace_context`; no per-arm focus ring is
introduced. The existing `full_shell_config_first_tab_focuses_filter_field` test
is kept and remains the proof (see Testability).

The selectable key-name and selectable-description buttons in the tree body ARE
focusable egui controls (they are today, too), but they come AFTER the Filter
field in allocation order, so they do not precede it in Tab order and do not
affect the first-Tab landing. This is unchanged from today's behaviour.

### D8 -- Preserve save convention (FR9, AC 22.11)

No change to SAVE/END/RETURN. The in-panel Save button keeps enqueuing
`ShellRequest::Command("SAVE")`. The new `Unsaved edits : <yes|no>` row is a pure
read of `state.is_dirty()` (AC 22.2). No autosave is introduced; edits still
stage into `working` via the unchanged `commit_value` path.

### D9 -- File-size split (NFR2; resolves NIT-4)

`render.rs` today has ~297 NON-TEST lines (407 total including the `#[cfg(test)]`
module starting at line 298) -- roughly 100 lines of headroom, so it is NOT near
the 400-line budget. The `layout.rs` split is therefore justified on TESTABILITY
and SEPARATION-OF-CONCERNS (isolating the pure alignment function for NFR4 /
AC 22.12), NOT on a false "near the limit" premise. Mirroring the existing
`tree.rs` GUI-free split, the new pure alignment helpers land in a new
`config_panel/layout.rs` (pure functions + their unit tests). `render.rs` keeps
the egui drawing but delegates width/label/padding computation to `layout.rs`.

That said, the re-skin adds the aligned row-draw plus the new header rows, which
may push `render.rs` toward the budget. If, after the re-skin, `render.rs`
approaches 400 non-test lines, the per-row draw is extracted into a
`render_key_row(...)` helper (and, if still over, a `config_panel/render_rows.rs`
holding the group-header and key-row draw helpers). The implementer MUST run the
400-line check and split before landing. `mod.rs` gains `mod layout; pub use
layout::{...}` re-exports as needed.

## File-by-file change plan

- `crates/ff-desktop/src/config_panel/layout.rs` -- NEW. Pure, GUI-free:
  `Connector`, `connector_prefix`, `key_label`, `value_column_width`,
  `pad_to_column`, plus a `#[cfg(test)]` module unit-testing single-group,
  multi-group, filtered-subset, and empty-set column computation (AC 22.12), and
  the connector-prefix / padding string shapes.
- `crates/ff-desktop/src/config_panel/mod.rs` -- add `mod layout;` and any
  `pub use` the render needs. No state fields added (the column is computed
  per-frame, not stored).
- `crates/ff-desktop/src/config_panel/render.rs` -- re-skin only:
  - header block (D2): Filter+Save row (UNCHANGED allocation order: Filter label,
    Filter `TextEdit` with `filter_field_id()`, `X`, then right-to-left Save) +
    new non-interactive `Unsaved edits : yes|no` label row + persist-error +
    Source-File, then ONE separator;
  - tree body: group rows as `[<DisplayName>]` headers (unchanged toggle), key
    rows drawn via the new aligned layout (D6) using `layout.rs` helpers and the
    last-child look-ahead; the selectable key name, selectable description, value
    widget, badge, staged `*`, and Reset all RETAINED with unchanged behaviour;
    `widget_ids` still recorded;
  - compute `column` per frame from visible key labels.
- `crates/ff-desktop/src/shell/tests_focus.rs` -- keep
  `full_shell_config_first_tab_focuses_filter_field` green (it already asserts the
  first Tab from the command field lands on `filter_field_id()` with no phantom
  stop -- the binding AC 22.10 proof after the header re-layout).
- NO change to `tree.rs`, `widgets.rs`, `keyboard.rs`, `commit.rs`, the shell
  dispatch, the `ff-config` crate, `menus/settings.toml`, or persistence.

## Error handling and edge cases

The redesign is pure rendering; it introduces no new fallible I/O. The edge
cases to handle are layout/data shape, each with concrete behaviour:

- Empty visible set (filter matches nothing): `value_column_width([]) == 0`; the
  tree body renders no key rows (group rows may still render per `visible_rows`).
  No panic. Unit-tested (AC 22.12 empty case).
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
  immediately follow the group header (confirmed in `tree.rs`). The design depends
  on that invariant and states it explicitly.
- Column position: owned by the pure `layout::value_column_width` over the
  per-frame visible key labels; the render only consumes the returned column. This
  isolates the alignment rule for unit testing (NFR4).
- Focus first-interior anchor: owned by `impl WorkspaceContext::render` returning
  `InteriorFocus::single(filter_field_id())` and dispatched via
  `render_workspace_context` (the single latch path). The render MUST allocate the
  Filter field first so egui Tab order agrees (D7).
- Dirty state: owned by `ConfigPanelState::is_dirty()` (unchanged); the header row
  only reads it.
- Selectable key name / description preservation: owned by the render keeping the
  existing `Button::selectable` widgets (D6); the alignment math reads their
  monospace char widths but does not replace the widgets.

## Testability

- Unit (pure, no egui) in `layout.rs`: `value_column_width` over single group,
  multiple groups, a filtered subset, and the empty set (AC 22.12);
  `connector_prefix`/`key_label`/`pad_to_column` producing the expected connector
  + padded strings (including that `├── ` and `└── ` are equal width); last-child
  connector selection given a `rows` slice. These mirror the existing `tree.rs`
  and `calendar_header.rs` pure tests.
- Integration (full-shell `egui_kittest`): keep
  `full_shell_config_first_tab_focuses_filter_field` green -- open `CONFIG`, assert
  `first_interior_id == filter_field_id()`, assert the first Tab from the command
  field focuses the Filter field with no phantom stop (FR8 / AC 22.10). This is
  the mandatory GUI behaviour test (workspace-conformance.md). It directly proves
  the D2/D7 placement (Save button not a phantom-ahead stop) after the header
  re-layout.
- What is NOT unit-testable and why: the pixel-exact rendering of box-drawing
  connectors and the on-screen column position are visual; the pure width function
  is unit-tested and the connectors are a deterministic string from
  `connector_prefix`/`key_label`, so the logic is covered; pixel-exact appearance
  is the justified manual/visual exception per testing.md.

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
- Multi-level nesting below the two-level group -> key model (FR3's vertical
  continuation bar does not apply -- D4).
- Drawing the user's full outer box frame (illustrative only, Assumption 1).

## Alignment with requirements assumptions

- A1 (mockup illustrative): honoured -- no outer box frame; best-practice
  demarcated header + connectors + alignment (D2, D4).
- A2 (monospace for alignment): chosen and justified -- monospace char-count
  padding AROUND the existing selectable widgets, not `egui::Grid`, and not by
  downgrading the widgets (D5, D6, "choice of mechanism" below).
- A3 (Command ===> ownership): confirmed from code -- the shell owns the command
  line as a `TopBottomPanel`; the panel header sits below it, the demarcation rule
  is drawn inside the panel between the Filter/dirty header and the tree body (D2).
- A4 (alignment scope = visible set): adopted -- whole-visible-set single column
  (D5).
- A5 (Reset/provenance to the right): retained -- right-aligned block unchanged;
  value widget starts at the aligned column (D6).

## Column alignment -- choice of mechanism (the one open option, decided)

Two viable mechanisms were considered for aligning the `:` column:

1. `egui::Grid` with a key column and a value column. REJECTED: the Keys Editor
   panel already documents (in `crates/ff-desktop/src/keys_editor_panel/render.rs`,
   lines ~83-85) that a `TextEdit`'s `desired_width` is only honoured reliably
   inside a plain horizontal layout -- inside an `egui::Grid` nested in a
   horizontal `ScrollArea` the cells collapse to a few characters. The config
   value widgets include full-width text fields and sliders, which would fight a
   grid's column sizing. A grid also makes the box-drawing connector prefix
   awkward (it wants to live in the same cell as the key).

2. **Monospace character-count padding (CHOSEN).** Build the connector + key
   width as a monospace char count and pad (with an inert monospace spacer, D6)
   to the max visible key-label width, then place the `:` and value widget after
   it. This is the idiom already proven in this codebase
   (`primary_option_menu/calendar_header.rs`'s `centered_calendar_header`): equal
   monospace char counts give equal pixel widths, so the `:` lands in the same
   column on every row. It needs no new dependency, keeps the value widgets at
   their natural widths, keeps the selectable key/description widgets, and the
   width rule is a pure, unit-testable function (NFR4). This is locked. (The
   earlier "Macro Library header" precedent cited in the prior design is DROPPED
   as unverified -- the `calendar_header.rs` precedent alone justifies the idiom;
   resolves NIT-7.)

---

## Responses to design-review findings (prior run 03-design-review.md)

This run imports the prior design and applies the user-APPROVED corrections.
Each finding from the prior review is addressed below.

- **MEDIUM-1 (description moved to hover text contradicts FR7/AC 22.9/Req 14.2)
  -- ADDRESSED.** D6 now KEEPS the description as a visible, selectable widget
  (the existing `Button::selectable`), rendered on a second line beneath the
  aligned key row; it is NOT moved to hover/tooltip. The alignment column is
  computed only from the key-label line, so the description does not perturb it.
  Requirement 14.2 is preserved.

- **MEDIUM-2 (key name downgraded to an inert monospace label breaks the
  selectable affordance) -- ADDRESSED.** D6 now RETAINS the existing selectable
  key-name control (`Button::selectable(... .monospace())`). Column alignment is
  achieved with an INERT monospace padding spacer AROUND the selectable widget
  (plus a stated frameless-`selectable_label` fallback if the button frame proves
  non-uniform), never by converting the key name to inert text. Requirement 14.2
  is preserved.

- **MEDIUM-3 (Save button + `Unsaved edits` row placement ambiguous; could
  introduce a phantom Tab stop) -- ADDRESSED.** D2 pins ONE layout: the Save
  button stays where it is today (after the Filter `TextEdit` in allocation
  order, in the first header row's right-to-left block), and `Unsaved edits :
  <yes|no>` is a SEPARATE non-interactive label row below it. The Filter field is
  allocated first, so it remains the first interior Tab stop; the Save button is
  never a phantom-ahead stop. D7 and the retained
  `full_shell_config_first_tab_focuses_filter_field` test are the binding proof
  of AC 22.10.

- **NIT-4 (D9 overstates how close `render.rs` is to the 400-line budget) --
  ADDRESSED.** D9 now states `render.rs` has ~297 non-test lines (407 total incl.
  tests) and justifies the `layout.rs` split on testability /
  separation-of-concerns, with conditional per-row extraction only if the re-skin
  approaches the budget. The false "near the budget" premise is removed.

- **NIT-5 (grid-rejection cites the wrong file) -- ADDRESSED.** The "choice of
  mechanism" section now cites
  `crates/ff-desktop/src/keys_editor_panel/render.rs` (lines ~83-85), the actual
  location of the documented grid/`ScrollArea` cell-collapse behaviour (verified).

- **NIT-6 (AC 22.4 vs FR3 connector-set mismatch not called out) -- ADDRESSED.**
  D4 now explicitly states that FR3's "vertical continuation bar for intermediate
  levels" does not apply because the tree is strictly two-level (no intermediate
  levels), so only the branch-tee and L-connector are drawn -- matching AC 22.4.

- **NIT-7 (unverified "Macro Library header" precedent) -- ADDRESSED.** The
  "Macro Library header" precedent is DROPPED. The idiom is justified solely by
  the verified `primary_option_menu/calendar_header.rs` precedent.
