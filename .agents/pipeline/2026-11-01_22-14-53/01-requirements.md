# Requirements -- Config View Tree Rendering (Visual Redesign)

> Imported verbatim from the prior, user-APPROVED run
> (`.agents/pipeline/2026-11-01_21-01-54/01-requirements.md`). Requirements are
> unchanged; the design-review findings against the prior design are resolved by
> user-approved design corrections carried into this run's `02-design.md`, not by
> changing these requirements. See "Carried-forward design corrections" at the
> end for the record of why no requirement text changed.

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
Default control (Requirement 15.3-15.6, 18.6), AND its existing SELECTABLE key
name and SELECTABLE, always-visible key description (Requirement 14.2). The tree
re-skin changes layout and connectors, not the per-key editing controls, the
selectable key name, the visible selectable description, or their behaviour.

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
exceed ~40 lines. The column-alignment algorithm and connector rendering SHALL
be factored into a dedicated pure helper module (`config_panel/layout.rs`) so
the files stay within budget and so the alignment logic is unit-testable,
consistent with the `_render` / `_state` split convention. (Note: this split is
justified on testability and separation-of-concerns, NOT on `render.rs` being
near the 400-line limit -- it has ~297 non-test lines today.)

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
with unchanged behaviour, AND its existing selectable key name and visible
selectable key description (Requirement 14.2) SHALL remain selectable and
visible (NOT moved to hover/tooltip, NOT downgraded to inert text). (FR7)

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
  two deep as today. Because the tree is strictly two-level (depth 0 group,
  depth 1 key -- no intermediate levels), FR3's "vertical continuation bar for
  intermediate levels" does not apply; only branch-tee and L-connectors are
  drawn, which matches AC 22.4.

## Assumptions (confirmed by the prior design review unless noted)

1. **ASCII mockup is illustrative, not prescriptive.** Per the user, the exact
   box frame around the whole panel and the exact glyphs are a mockup; the design
   may adopt a best-practice tree form (egui monospace rows with box-drawing
   connectors) rather than literally drawing a full outer box. The MUST-haves
   distilled are: demarcated header vs body, bracketed group headers, box-drawing
   child connectors, and vertical value alignment.
2. **Monospace rendering for alignment.** Vertical column alignment of the `:`
   and values is achieved by rendering key labels + connectors in a monospace
   font and computing a shared column offset (char-count padding) from the widest
   visible key, NOT by an `egui::Grid` (which collapses cells inside a horizontal
   `ScrollArea` -- documented in `keys_editor_panel/render.rs`). The padding is
   applied around the EXISTING selectable key-name/description widgets, not by
   downgrading them to inert text.
3. **"Command ===>" line ownership.** The `Command ===>` field shown in the
   header is the shell's existing command field for the Context (a
   `TopBottomPanel`, id `command_field_input`), not a new widget owned by the
   panel; the panel's header sits below it, with the rule between header and
   tree body drawn by the panel.
4. **Alignment scope is the visible set.** Column width is computed per-frame
   over currently visible rows only (so a collapsed group's long keys do not
   widen the column), giving one shared column for the whole visible tree.
5. **Reset/provenance controls remain to the right.** The existing right-aligned
   provenance badge and Reset button are retained; "value" in the alignment
   requirement refers to the key's effective/edited value text/widget, with the
   `:` column aligned so the value widget starts at the computed column.

## Carried-forward design corrections (resolve the prior review's 3 MEDIUM findings)

These are user-APPROVED corrections to the prior DESIGN (not to the requirements).
They are recorded here so the design-review loop confirms they are honoured.
Requirement text did not change because FR7/AC 22.9 already forbade the behaviour
changes the prior design had proposed; the corrections bring the design back into
line with the requirements:

- **Correction 1 (MEDIUM-1):** The per-key description MUST NOT move to
  hover/tooltip text. It stays visible and selectable exactly as today
  (Requirement 14.2). This is now stated explicitly in FR7 / AC 22.9.
- **Correction 2 (MEDIUM-2):** The selectable key-name button MUST NOT be
  downgraded to a plain inert monospace label. Char-count column alignment is
  achieved with padding/layout AROUND the existing selectable widgets. Stated in
  FR7 / AC 22.9 and Assumption 2.
- **Correction 3 (MEDIUM-3):** The Save button and `Unsaved edits : <yes|no>` row
  placement MUST NOT introduce a phantom Tab stop; the first interior control
  stays the Filter field. AC 22.10 remains the binding test; the exact placement
  is pinned down in the design.

NIT corrections carried for the design: justify the `layout.rs` split on
testability (not a false 400-line premise -- see NFR2); the grid-rejection
citation is `keys_editor_panel/render.rs`; the AC 22.4-vs-FR3 connector
divergence is stated explicitly (see Out of Scope); and the unverified
"Macro Library header" precedent is dropped in favour of the verified
`calendar_header.rs` precedent.
