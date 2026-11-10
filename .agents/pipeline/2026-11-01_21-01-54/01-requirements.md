# Requirements -- Config View Tree Rendering (Visual Redesign)

## Summary

Redesign the VISUAL representation of the Configuration Context (the `CONFIG`
key browser/editor, `TabKind::ConfigPanel`) so it renders as a clearly
demarcated, best-practice TREE VIEW: a ruled header area (Command line, Filter,
Unsaved-edits indicator) separated from a tree body of bracketed namespace group
headers whose child key nodes are drawn with box-drawing tree connectors, with
the `:` / value column vertically aligned across all visible rows.

This is a VISUAL/representation change ONLY. The grouping mechanism, the
expand/collapse behaviour, the config persistence, the save semantics, the
focus contract, and the keyboard tree navigation (Requirement 21) are all
preserved -- only the way the existing `visible_rows` model is drawn changes.

This adds a new **Requirement 22** to
`docs/specs/configuration-system/requirements.md`. It does not replace
Requirements 15, 20, or 21; it refines how the Config View from those
requirements is rendered.

## Context (verified against the code)

- The Config View lives in `crates/ff-desktop/src/config_panel/`
  (`mod.rs`, `render.rs`, `tree.rs`, `keyboard.rs`, `widgets.rs`, `commit.rs`).
- Grouping is derived by `tree::namespace_of(key)` (first dot-segment) and
  `tree::visible_rows(entries, filter, collapsed)`, which returns an ordered
  `Vec<ConfigRow>` of `ConfigNodeId::Namespace(ns)` group rows (sorted) followed
  by their `ConfigNodeId::Key(key)` child rows (sorted) for each EXPANDED group.
  This produces the confirmed 8 groups (Accessibility, Catalogs, Editor, Logging,
  Menu, Theme, Ui, Vfs) from the schema registry. **This mechanism is unchanged.**
- Expand/collapse state is the per-namespace `ConfigPanelState.collapsed:
  HashMap<String, bool>`, toggled by mouse (click on the group header in
  `render.rs`) and by keyboard (`tree::reduce_config_key` Right/Left/Enter via
  `keyboard.rs`). **This mechanism is unchanged.**
- The current rendering (`render::render`) draws a Filter+Save bar, a
  `ui.separator()`, an optional Source-File row, then a `ScrollArea` iterating
  the `visible_rows`: namespace rows as `selectable_label` with a `v`/`>` arrow
  prefix, key rows via `ui.indent(...)` into `render_entry` (a horizontal row of
  key/description + provenance badge + Reset button + value widget). There are NO
  box-drawing connectors and NO column alignment today.
- The panel is a migrated `WorkspaceContext`: `impl WorkspaceContext for
  ConfigPanelState` returns `InteriorFocus::single(filter_field_id())` and is
  dispatched through `render_workspace_context`. `filter_field_id()` =
  `egui::Id::new("config_panel_filter")` is the reported first interior control.
- Save semantics are the ISPF explicit-commit model: edits stage into
  `ConfigPanelState.working` (the working-copy buffer); `is_dirty()` reflects
  unsaved edits; SAVE/END flush, RETURN discards. No autosave.
- Existing focus test: `full_shell_config_first_tab_focuses_filter_field`.

## Functional Requirements

### FR1 -- Demarcated header area
The Config View SHALL render a header area, visually separated from the tree
body by a horizontal rule, containing (top to bottom): the `Command ===>` line
(the shell command field, where it already appears for the Context), a `Filter`
row (label + the existing filter text field + clear control), and an
`Unsaved edits : <yes|no>` row reflecting working-copy dirty state. The header
area and the tree body SHALL be clearly demarcated from each other.

### FR2 -- Bracketed group headers
Each top-level namespace group SHALL render as a bracketed group header of the
form `[<DisplayName>]` (e.g. `[Accessibility]`, `[Catalogs]`, `[Editor]`), using
the existing `ns_display_name` capitalisation and the existing sorted group
order from `visible_rows`. The group header SHALL carry an expand/collapse
affordance and remain clickable to toggle, exactly as today.

### FR3 -- Box-drawing child connectors
Under an EXPANDED group header, each child key node SHALL render with
box-drawing tree connectors (ASCII box-drawing range U+2500-U+257F): a branch
tee for a non-last child, an L-connector for the last child, and the vertical
continuation bar for intermediate levels, in the standard tree-control idiom.
Connectors SHALL be drawn in a monospace context so they align visually.

### FR4 -- Vertically aligned value column
Across all currently visible rows, the `:` separator and the value text SHALL be
vertically aligned into a single column. Alignment SHALL be computed from the
maximum rendered key-label width (connector prefix + key text) among the
CURRENTLY VISIBLE rows, so the `:` column sits just past the widest visible key.
When the set of visible rows changes (expand/collapse or filter change), the
alignment SHALL recompute for the new visible set.

### FR5 -- Preserve grouping mechanism
The derivation of groups and their membership SHALL remain exactly as today
(`namespace_of` + `visible_rows`). The 8 confirmed groups (Accessibility,
Catalogs, Editor, Logging, Menu, Theme, Ui, Vfs) and their contents SHALL be
unchanged. No change to grouping logic, config persistence, or save semantics.

### FR6 -- Preserve expand/collapse behaviour
Expand/collapse SHALL continue to toggle the visibility of a group's child key
nodes using the existing `collapsed` state and the existing mouse and keyboard
triggers (click on the header; keyboard Right/Left/Enter via
`reduce_config_key`). The new rendering SHALL re-skin this, not replace it.

### FR7 -- Preserve per-key edit controls
Each visible key node SHALL continue to expose its existing type-appropriate
value widget, provenance/LOCKED badge, staged-edit indicator, and Reset-to-
Default control (Requirement 15.3-15.6, 18.6). The tree re-skin changes layout
and connectors, not the per-key editing controls or their behaviour.

### FR8 -- Preserve the focus contract
The Config View SHALL remain on the single focus-latch path: `impl
WorkspaceContext::render` SHALL continue to return
`InteriorFocus::single(filter_field_id())`, dispatched via
`render_workspace_context`. The first Tab from the command field SHALL land
EXACTLY on the Filter field (the reported first interior control) with no
phantom stop. No per-arm focus ring or parallel focus mechanism SHALL be
introduced.

### FR9 -- Preserve save convention
The ISPF explicit-commit save semantics SHALL be unchanged: SAVE commits and
stays, END commits and exits, RETURN discards and exits; the `Unsaved edits`
indicator (FR1) SHALL reflect `ConfigPanelState.is_dirty()`. No autosave SHALL
be introduced.

## Non-Functional Requirements

### NFR1 -- Framework rails
The change SHALL build ON the existing framework (no new dispatch path, no new
navigation stack, no bespoke focus mechanism). It SHALL keep using the pure
`tree::visible_rows` model as the single source of visible-row order.

### NFR2 -- File-size and function limits (Rust standards)
No source `.rs` file SHALL exceed 400 non-test lines, and no function SHALL
exceed ~40 lines. `config_panel/render.rs` is already near the limit; the
column-alignment algorithm and connector rendering SHALL be factored so the
files stay within budget (e.g. a dedicated tree-rendering / layout helper split
by concern, consistent with the `_render` / `_state` split convention).

### NFR3 -- ASCII-first docs; box-drawing in code blocks only
Spec/design/task text SHALL be ASCII-first; box-drawing characters (U+2500-
U+257F) are permitted only inside code blocks / ASCII-art illustrating the tree
connectors (documentation-charset.md).

### NFR4 -- Testability
The column-alignment computation (max key-width over visible rows -> column
position) SHALL be expressed as a pure function over the visible-row data so it
is unit-testable independently of egui, mirroring the existing GUI-free
`tree.rs` model. GUI behaviour SHALL be covered by a full-shell `egui_kittest`
test.

## Acceptance Criteria (EARS, testable)

Numbered against the new Requirement 22; `<n>` is the sub-criterion.

22.1. THE Config View SHALL render a header area, separated from the tree body by
a horizontal rule, that shows the `Command ===>` line, a `Filter` row, and an
`Unsaved edits : <yes|no>` row; the two areas SHALL be clearly demarcated.
(FR1)

22.2. WHEN the working-copy buffer is non-empty, THE `Unsaved edits` row SHALL
show `yes`; WHEN it is empty, it SHALL show `no` -- i.e. the indicator SHALL
equal `ConfigPanelState.is_dirty()`. (FR1, FR9)

22.3. THE Config View SHALL render each top-level namespace group as a bracketed
header `[<DisplayName>]` using the existing display-name capitalisation, in the
existing sorted group order from `visible_rows`. (FR2, FR5)

22.4. WHEN a group is EXPANDED, THE Config View SHALL render each of its child
key nodes with box-drawing tree connectors (U+2500-U+257F): a branch-tee
connector for every child except the last, and an L-connector for the LAST
child of the group. (FR3)

22.5. THE Config View SHALL align the `:` separator and the value text of ALL
currently visible key rows into a single vertical column, positioned just past
the widest visible key label (connector prefix + key). (FR4)

22.6. WHEN the set of visible rows changes (a group is expanded or collapsed, or
the filter changes), THE Config View SHALL recompute the value-column position
from the new set of currently visible rows. (FR4, FR6)

22.7. THE grouping mechanism (namespace derivation and group membership via
`namespace_of` + `visible_rows`) and the 8 confirmed groups (Accessibility,
Catalogs, Editor, Logging, Menu, Theme, Ui, Vfs) SHALL be unchanged by this
rendering redesign. (FR5)

22.8. Expand/collapse SHALL continue to toggle the visibility of a group's child
key nodes using the existing per-namespace `collapsed` state, via BOTH the
existing mouse trigger (click the group header) AND the existing keyboard
triggers (Right/Left/Enter through `reduce_config_key`). (FR6)

22.9. Each visible key node SHALL continue to expose its existing value widget,
provenance/LOCKED badge, staged-edit indicator, and Reset-to-Default control
with unchanged behaviour. (FR7)

22.10. THE Config View's `WorkspaceContext::render` SHALL return
`InteriorFocus::single(filter_field_id())` and be dispatched via
`render_workspace_context`; the first Tab from the command field SHALL focus the
Filter field with no phantom stop (covered by the full-shell first-Tab
egui_kittest test, extended/kept green). (FR8)

22.11. THE SAVE / END / RETURN semantics SHALL be unchanged (SAVE commits+stays,
END commits+exits, RETURN discards+exits); no autosave SHALL be introduced.
(FR9)

22.12. THE value-column position SHALL be produced by a pure, egui-independent
function over the visible-row key labels (max-width -> column), unit-tested for:
single group, multiple groups, filtered subset, and the empty set. (NFR4)

## Out of Scope

- Changing how namespaces are derived or which keys belong to which group
  (grouping mechanism is frozen).
- Changing config persistence, layering, validation, or the SAVE/END/RETURN
  flush logic.
- Changing the keyboard navigation transition table (Requirement 21) -- only its
  on-screen representation is re-skinned; cursor/selection mechanics are
  preserved.
- Adding new config keys, new groups, or new editing widgets.
- Any change to the Settings MENU (`menus/settings.toml`) or the `CONFIG`
  command resolution (Requirement 20).
- Multi-level nesting below the two-level (group -> key) model; the tree stays
  two deep as today (connectors may still draw a continuation bar, but no new
  hierarchy is introduced).

## Assumptions (to be confirmed by the design review)

1. **ASCII mockup is illustrative, not prescriptive.** Per the user, the exact
   box frame around the whole panel and the exact glyphs are a mockup; the design
   may adopt a best-practice tree form (e.g. egui monospace rows with
   box-drawing connectors) rather than literally drawing a full `┌──┐` outer box.
   The MUST-haves distilled are: demarcated header vs body, bracketed group
   headers, box-drawing child connectors, and vertical value alignment.
2. **Monospace rendering for alignment.** Vertical column alignment of the `:`
   and values is assumed to be achieved by rendering key labels + connectors in a
   monospace font and computing a shared column offset from the widest visible
   key, rather than by a multi-column grid widget. The design will choose and
   justify the concrete mechanism (monospace padding vs an egui `Grid`/columns
   layout); either satisfies 22.5 as long as the `:` column aligns.
3. **"Command ===>" line ownership.** The `Command ===>` field shown in the
   header is the shell's existing command field for the Context, not a new widget
   owned by the panel; the panel's header simply sits below/with it. The design
   will confirm exactly where the rule is drawn relative to the shell command
   line vs the panel's own content.
4. **Alignment scope is the visible set.** Column width is computed per-frame
   over currently visible rows only (so a collapsed group's long keys do not
   widen the column). This matches the user's "aligned vertically" intent for the
   rows on screen and is cheaper than a global max. The design will confirm
   whether a per-group or whole-tree max is preferred; the requirements default
   to whole-visible-set alignment (one column for the panel).
5. **Reset/provenance controls remain to the right.** The existing right-aligned
   provenance badge and Reset button are retained; "value" in the alignment
   requirement refers to the key's effective/edited value text/widget, with the
   `:` column aligned. The design will specify how the value widget coexists with
   the aligned `:` column (e.g. value text/widget starts at the computed column).
