# Design Review -- Config View Tree Rendering (Requirement 22)

Reviewed: `02-design.md` against `01-requirements.md`, with all code claims
verified against the live source under `crates/ff-desktop/src/config_panel/` and
`crates/ff-desktop/src/shell/`.

Verdict: CHANGES_REQUESTED (0 HIGH, 3 MEDIUM, 4 NIT).

---

## Findings

### 1. MEDIUM -- D6 relocates the key description to hover text, which contradicts the "unchanged behaviour" requirement and drops a selectable surface

D6 ("How the value widget coexists with the aligned `:` column") decides: "move
the description to hover text so it does not disturb the single aligned column".
But FR7 and AC 22.9 require each key node to "continue to expose its existing
value widget, provenance/LOCKED badge, staged-edit indicator, and Reset-to-
Default control with UNCHANGED behaviour", and the existing `render_entry`
(verified in `config_panel/render.rs`) draws the description as a SELECTABLE
widget:

```rust
ui.vertical(|ui| {
    ui.add(egui::Button::selectable(false, egui::RichText::new(key.as_str()).monospace()));
    ui.add(egui::Button::selectable(false,
        egui::RichText::new(entry.description.as_str()).small().weak()));
});
```

The code comment ties this to `// Validates: Requirement 14.2 -- selectable key
name and description`. Moving the description into `on_hover_text` makes it (a)
no longer always-visible and (b) no longer a selectable widget, which is an
observable behaviour change that FR7/AC 22.9 and Requirement 14.2 forbid.

Concrete fix: do not move the description to hover text. Keep the description as
a visible, selectable element on the key row. Two acceptable layouts:
- Render the description as a secondary weak label on the SAME row, placed AFTER
  the value widget (the design's own fallback phrasing "kept as a secondary
  small weak label on the same row AFTER the value"), OR
- Keep the description on a second line under the aligned key row (a two-line
  key row), provided the `:`/value column alignment is still computed only from
  the key-label line.
Pick one explicitly and state that the description stays a selectable widget
(Req 14.2 preserved). If the design genuinely intends to change description
display, that is a behaviour change requiring the requirements gate, not a
design-only decision.

### 2. MEDIUM -- The description relocation also silently alters the row's left-hand layout that the alignment math depends on, with no stated handling of the key being `monospace`-styled

The pure `key_label`/`value_column_width`/`pad_to_column` design (D5) measures
width by `str::chars().count()` over "connector prefix + full key path", assuming
a single monospace label. But the existing key text is rendered as a
`Button::selectable(... .monospace())` and the design folds "render_entry's
widget/badge/Reset logic ... into the aligned row". The design does not state
whether the padded key label is still a `monospace` `RichText` label (so the
char-count->pixel-width assumption holds) OR a selectable button (whose padding/
frame breaks the equal-pixel-width property the alignment relies on). D5 says
"render it as a single `RichText::monospace` label", which DROPS the current
selectable-key affordance (same class of issue as Finding 1 for the key name).

Concrete fix: state explicitly that the key path is drawn as a non-interactive
`RichText::monospace` label (no button frame/padding), so the char-count width
measure is valid, AND confirm whether losing the selectable key-name button is
intended. If Requirement 14.2's "selectable key name" must be preserved, the
alignment must be computed over the label but the clickable/selectable surface
retained; specify how (e.g. a transparent `selectable_label` sized to the padded
monospace text) so the alignment property is not broken.

### 3. MEDIUM -- Save button + "Unsaved edits : <yes|no>" row placement is left ambiguous ("on the same row or immediately adjacent")

D2 says the new `Unsaved edits : <yes|no>` row's Save button "sits on the same
row or immediately adjacent". AC 22.1 requires a specific header layout (Command
line, Filter row, `Unsaved edits : <yes|no>` row) and AC 22.2 ties the indicator
to `is_dirty()`. "on the same row or immediately adjacent" is two valid
interpretations; it also interacts with the focus-ordering caveat in D7 (only
the Filter field is a Tab stop, and the Save button IS interactive). If the Save
button is allocated before the Filter field, it becomes a phantom-ahead Tab stop
and breaks AC 22.10.

Concrete fix: pick one layout. Recommended: keep the existing Filter+Save bar as
the first header row (Save button stays where it is today, AFTER the Filter
`TextEdit` in allocation order, so the Filter field remains the first Tab stop),
and render `Unsaved edits : <yes|no>` as its own non-interactive label row BELOW
the Filter+Save bar. State that the Filter `TextEdit` is allocated before the
Save button so egui Tab order still reaches Filter first (the current code
allocates Filter first inside the `ui.horizontal`, then the right-to-left Save
block -- preserve that order).

### 4. NIT -- D9 overstates how close `render.rs` is to the 400-line budget

D9 says "`render.rs` is near the 400-line non-test budget." Verified: the file is
407 lines TOTAL, with the `#[cfg(test)]` module starting at line 298, so NON-TEST
code is ~297 lines -- roughly 100 lines of headroom, not "near the budget". The
split-out of `layout.rs` is still sound (it isolates the pure function for NFR4),
but the stated premise is inaccurate.

Concrete fix: restate as "`render.rs` has ~297 non-test lines; adding the aligned
row-draw plus header rows may approach the 400-line budget, so pure
width/label/padding helpers move to `layout.rs` and the per-row draw is extracted
if the file approaches budget." Keep the conditional split guidance.

### 5. NIT -- D5's grid-rejection rationale cites "the keys_editor panel" but the documented evidence is in `keys_editor_panel/render.rs`

D5 ("Column alignment -- choice of mechanism") rejects `egui::Grid` because "the
keys_editor panel already documents that a `TextEdit`'s `desired_width` is
unreliable inside an `egui::Grid` nested in a horizontal `ScrollArea` (cells
collapse)". Verified: that comment exists, but in
`crates/ff-desktop/src/shell/keys_editor_panel/render.rs` (not
`shell/keys_editor.rs`). The claim is substantively CORRECT; only the file
reference is loose.

Concrete fix: cite `config_panel`'s sibling `keys_editor_panel/render.rs` (lines
~83-88) as the source of the documented grid/ScrollArea collapse behaviour.

### 6. NIT -- AC 22.4 vs FR3 connector-set mismatch is resolved by the design but not called out

FR3 lists three connector forms (branch-tee, L-connector, AND "the vertical
continuation bar for intermediate levels"). D4 deliberately draws only tee/L and
omits the vertical continuation bar, correctly arguing the model is strictly
two-level (`visible_rows` only emits depth 0/1, verified in `tree.rs`). AC 22.4
itself lists only tee + L, so the design satisfies the testable AC; but it
silently diverges from FR3's prose.

Concrete fix: add one sentence in D4 noting that FR3's "vertical continuation bar
for intermediate levels" does not apply because the tree is two-level (no
intermediate levels exist), so only tee/L are drawn -- matching AC 22.4. This
makes the divergence an explicit, justified decision rather than an omission.

### 7. NIT -- "Macro Library header" alignment-idiom reference is unverified

D5 and the tech-stack section cite three precedents for the monospace char-count
padding idiom: `calendar_header.rs`, "the POM calendar rows", and "the Macro
Library header". The first two are verified (`centered_calendar_header` in
`primary_option_menu/calendar_header.rs` uses exactly this
constant-width-padding idiom). The "Macro Library header" precedent was not
located during review.

Concrete fix: either cite the exact file/function for the Macro Library
precedent or drop it; the `calendar_header.rs` precedent alone is sufficient to
justify the idiom.

---

## Verified Assumptions

- Config View lives in `crates/ff-desktop/src/config_panel/` with the stated
  modules (`mod.rs`, `render.rs`, `tree.rs`, `keyboard.rs`, `widgets.rs`,
  `commit.rs`) -- CONFIRMED (`commit.rs` referenced by `mod.rs` and `widgets.rs`).
- `ConfigPanelState` fields (`filter`, `namespace_filter`, `collapsed`,
  `pending`, `errors`, `working`, `persist_error`, `cursor`, `widget_ids`),
  `is_dirty()`, `stage_set`/`stage_reset`, `working_value` -- CONFIRMED verbatim
  in `config_panel/mod.rs`.
- `filter_field_id()` == `egui::Id::new("config_panel_filter")` -- CONFIRMED.
- `impl WorkspaceContext for ConfigPanelState::render` returns
  `InteriorFocus::single(filter_field_id())` and delegates to `render::render`
  -- CONFIRMED.
- `tree.rs` is the pure GUI-free model: `ConfigNodeId` (`Namespace`/`Key`),
  `ConfigRow { id, depth, expandable, expanded }`, `TreeEntry { key,
  description }`, `visible_rows` (BTreeMap-sorted namespaces, sorted keys within
  expanded groups, depth 0 for namespace / depth 1 for key), `reduce_config_key`,
  `reconcile_cursor` -- CONFIRMED.
- `visible_rows` guarantees a group's keys are contiguous and immediately follow
  the group header (the invariant D4's last-child look-ahead depends on) --
  CONFIRMED by the build loop in `tree.rs`.
- The two-level depth model (only depth 0 and depth 1 emitted) -- CONFIRMED;
  supports D4's omission of the vertical continuation bar.
- Current `render.rs` draws a Filter+Save bar, persist-error line,
  `ui.separator()`, optional Source-File row + separator, then a `ScrollArea`
  over `rows`: namespace rows as `selectable_label` with `v`/`>` arrow + `(count)`
  suffix; key rows via `ui.indent(...)` into `render_entry` -- CONFIRMED.
- `render_entry` draws key/description as a vertical stack of two
  `Button::selectable` widgets, right-aligned provenance/LOCKED badge + staged
  `*` + Reset button, then the value widget via `render_widget` and records the
  widget id into `state.widget_ids` -- CONFIRMED.
- `render_widget` returns `Option<egui::Id>`, drawing checkbox/slider/combo/text
  per value type and editing via `commit_value`; path-type string fields request
  `f32::INFINITY` width -- CONFIRMED in `widgets.rs`.
- The Save button enqueues `ShellRequest::Command("SAVE")` and the
  `ShellRequest::Command(String)` variant exists -- CONFIRMED
  (`render.rs` + `shell/workspace_context.rs`).
- Shell owns the `Command ===>` field as a `TopBottomPanel` (`command_field`)
  with the text widget id `command_field_input`, OUTSIDE the central panel
  (`shell/render_command_line.rs`) -- CONFIRMED (confirms requirements
  Assumption 3).
- The Config arm is dispatched via `render_workspace_context` (owned-panel swap,
  single focus-latch path) in `shell/render_body.rs` -- CONFIRMED.
- `full_shell_config_first_tab_focuses_filter_field` exists in
  `shell/tests_focus.rs` -- CONFIRMED.
- `ns_display_name` ASCII-uppercases the first char and lives in `render.rs` --
  CONFIRMED.
- `calendar_header.rs` uses the constant-width monospace-padding idiom
  (`centered_calendar_header`) D5 cites as precedent -- CONFIRMED.
- The documented grid/`ScrollArea` cell-collapse behaviour D5 relies on to
  reject `egui::Grid` -- CONFIRMED to exist (see Finding 5 for the file-path
  correction).

## Unverified / Wrong Assumptions

- WRONG (minor): "`render.rs` is near the 400-line non-test budget" -- it is
  ~297 non-test lines (407 total incl. tests). See Finding 4.
- IMPRECISE: D5's grid-rejection evidence is in `keys_editor_panel/render.rs`,
  not `shell/keys_editor.rs` as the phrasing implies. See Finding 5.
- UNVERIFIED: the "Macro Library header" alignment-idiom precedent cited in D5 /
  tech-stack was not located. See Finding 7.
- NOT A CODE CLAIM (design decision, flagged as a conflict): D6's move of the
  description to hover text is not an existing-code fact but a proposed change
  that conflicts with FR7/AC 22.9 and the existing selectable-description
  behaviour (Req 14.2). See Findings 1 and 2.
