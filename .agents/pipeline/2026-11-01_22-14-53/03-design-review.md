# Design Review -- Config View Tree Rendering (Requirement 22)

Reviewed: `02-design.md` against `01-requirements.md`, with every claim about
existing code checked against the actual source under
`crates/ff-desktop/src/config_panel/` and neighbouring modules.

This is a revision (the design imports a prior run and records responses to the
prior review's MEDIUM-1/2/3 and NIT-4/5/6/7). I re-checked those carried-forward
responses against the new design and the code, then reviewed what the new design
proposes. The prior findings are addressed in the text (see "Prior findings
re-check" below); the findings in this review are NEW observations against the
current design.

## Findings

### 1. MEDIUM -- D6 single-`horizontal` row mixes a left-column value widget and a right-to-left block without pinning allocation order or coexistence

D6 ("Concrete layout per key row, in one `ui.horizontal(|ui| { ... })`") lists,
in order: (1) connector prefix, (2) selectable key button, (3) inert spacer to
the column, (4) `:` label, (5) the value widget "placed right after the `:`
column", then (6) "The right-aligned block ... via `ui.with_layout(right_to_left,
...)`".

Verified against the current code (`render.rs::render_entry`): TODAY the value
widget is NOT inside the key `ui.horizontal`. The horizontal row contains only
the `ui.vertical` key/description stack and the right-to-left provenance/Reset
block; `render_widget(...)` is called AFTER the `ui.horizontal` closes, so the
value widget currently sits on its OWN line below the key row. The design's D6
moves the value widget INTO the same horizontal and positions it at the aligned
`:` column, with the right-to-left block ALSO in that horizontal after it.

In egui, a `right_to_left` sub-layout greedily consumes the remaining horizontal
width from the right. If item 6's right-to-left block is allocated after a
left-positioned value widget in the same `ui.horizontal`, the two will fight for
the row's width and the column-alignment property (D5) can break: a full-width
(`f32::INFINITY`) path TextEdit (verified in `widgets.rs`) placed at item 5 would
expand to fill, leaving nothing for the right-to-left block, or push it off-row.
The design asserts the order but does not specify HOW the left-aligned aligned
value column and the right-greedy block coexist on one row.

Concrete fix: pin the row construction explicitly. Either (a) keep the current
two-line structure -- line 1 is the aligned `connector + key + : + value`, and
the right-to-left provenance/Reset block stays where it is today (on the key's
horizontal row, which is now line 1's left content) -- and state the exact egui
allocation order and which sub-`ui` owns the right block; or (b) specify that the
value widget is NOT `f32::INFINITY` inside this layout (give path fields a bounded
`desired_width` here) so the right-to-left block has room, and state the explicit
left-to-right then right-to-left allocation sequence. Name the chosen structure
and the allocation order so the implementer cannot reintroduce a width conflict.

### 2. MEDIUM -- Column alignment across a `ui.indent` + selectable-button row is asserted but the inset interaction is left unresolved (hedge, not a decision)

D5/D6 rely on monospace char-count padding producing equal pixel widths so the
`:` lands in one column. But the current render wraps each key row in
`ui.indent(("config_key_indent", key), ...)` (verified in `render.rs`), and D6
item 2 keeps the key as an `egui::Button::selectable(...)` which has frame insets.
D6 acknowledges this: "the selectable button's own frame insets are constant per
row, so they shift the column uniformly ... if the button frame proves
non-uniform in practice, switches the key control to a frameless
`selectable_label`". That is a HEDGE ("if ... in practice ... switches"), not a
locked decision -- design is meant to pin the mechanism, not defer it to runtime
discovery.

Two concrete unknowns the design leaves open: (i) the `ui.indent` wrapper adds a
left inset that the pure `layout.rs` char-count math does not model, so the
leftmost column origin is set by egui, not by `pad_to_column`; (ii) a framed
`Button::selectable` and an inert `RichText::monospace` spacer do NOT have the
same per-character box as the button adds padding/frame around its text, so
"append spacer to reach column N" does not guarantee the `:` lands at the same
pixel x as a row whose key is shorter.

Concrete fix: lock ONE mechanism now. Recommended: drop the `ui.indent` wrapper
for key rows (the connector prefix already conveys depth) and render the entire
aligned left segment -- `connector + key + padding + " : "` -- as a SINGLE
`RichText::monospace` string via `pad_to_column`, with the key SELECTABILITY
provided by a frameless `ui.selectable_label(is_cursor, that_monospace_text)`
(still a selectable control, Req 14.2 / AC 22.9 preserved) rather than a framed
`Button::selectable` plus a separate spacer. That makes the char-count-to-pixel
mapping exact (the `calendar_header.rs` property the design cites) and removes the
inset/frame variability. State this as the decision, not a conditional fallback.

### 3. NIT -- D6 "full-width text field" note is imprecise about non-path strings

D6's closing note says "path-type string fields currently request `f32::INFINITY`
width and bounded ints/floats draw sliders." Verified in `widgets.rs`: non-path
String fields use `desired_width(400.0)` (a FIXED width), not infinity, and not a
slider. The note omits this third case. It does not change the design's
conclusion (the `:` column aligns the START of the value widget regardless), but
the enumeration is incomplete.

Concrete fix: add "non-path string fields use a fixed `400.0` width" to the note
so all three widget-width cases (infinity path, fixed 400 string, slider) are
covered; the alignment-at-column-start argument then applies to each explicitly.

### 4. NIT -- Line-count citation is off by one

D9 and "What exists today" state the `#[cfg(test)]` module starts at line 298 and
non-test code is ~297 lines. Verified: `render.rs` is 407 lines total and the
`#[cfg(test)]` attribute is at line 297 (so ~296 non-test lines). Off by one;
immaterial to the headroom argument.

Concrete fix: change "line 298 ... ~297 non-test" to "line 297 ... ~296 non-test"
(or just say "~296 non-test lines, ~100 lines of headroom").

## Verified Assumptions

- `config_panel/` modules (`mod.rs`, `tree.rs`, `render.rs`, `widgets.rs`,
  `keyboard.rs`, `commit.rs`) exist as described. (The `file_search` tool
  returned nothing for "config_panel", but `grep_search` and direct reads confirm
  the files are present; the file_search miss is a tool quirk, not an absence.)
- `ConfigPanelState` fields (`filter`, `namespace_filter`, `collapsed`, `pending`,
  `errors`, `working: HashMap<String,WorkingEdit>`, `persist_error`, `cursor`,
  `widget_ids`), `is_dirty()`, `stage_set/stage_reset`, `working_value`, and the
  `impl WorkspaceContext` returning `InteriorFocus::single(filter_field_id())` --
  all verified in `mod.rs`.
- `filter_field_id() == egui::Id::new("config_panel_filter")` -- verified.
- `tree.rs` is pure/GUI-free, emits only depth 0 (Namespace) and depth 1 (Key),
  namespaces via `BTreeMap` (sorted), keys sorted within expanded groups, and a
  group's keys are contiguous after its header -- verified. This is the invariant
  D4's last-child look-ahead and D3's two-level claim depend on; it holds.
- `render.rs` draws the Filter+Save bar (Filter label, `TextEdit` with
  `filter_field_id()`, `X` button, then a right-to-left Save block with a coloured
  "unsaved changes" dot), a persist-error line, `ui.separator()`, an optional
  source-file row + separator, then the `ScrollArea` over `visible_rows` --
  verified, matching D2's "what exists today".
- The Save button enqueues `ShellRequest::Command("SAVE".to_string())` -- verified;
  `ShellRequest::Command(String)` exists in `shell/workspace_context.rs`.
- `render_entry` renders the key name and description as two stacked
  `egui::Button::selectable(false, ...)` widgets (monospace key, small/weak
  description), a right-to-left provenance/`*`/Reset block, then the value widget
  -- verified. Both key and description ARE selectable controls today (Req 14.2).
- The shell owns `Command ===>` as a `TopBottomPanel`, widget id
  `command_field_input` (`render_command_line.rs`), outside the central panel --
  verified (Assumption 3 confirmed).
- The Config arm in `shell/render_body.rs` uses
  `mem::take(&mut self.config_panel)` -> `render_workspace_context(...)` ->
  put-back (the single focus-latch path) -- verified (lines ~91-93).
- `full_shell_config_first_tab_focuses_filter_field` exists in
  `shell/tests_focus.rs` -- verified.
- `primary_option_menu/calendar_header.rs::centered_calendar_header` pads a label
  to a constant monospace width for column alignment -- verified (the cited,
  real precedent for the char-count padding idiom).
- `keys_editor_panel/render.rs` documents (lines ~80-85) that a `TextEdit`'s
  `desired_width` is only honoured in a plain horizontal layout and collapses
  inside a Grid-in-horizontal-ScrollArea -- verified (the grid-rejection citation
  is correct; resolves prior NIT-5).
- `widgets.rs::render_widget` returns `Option<egui::Id>`; path strings use
  `f32::INFINITY`, bounded ints/floats use sliders, booleans a checkbox, enum
  strings a combo -- verified.

## Unverified / Wrong Assumptions

- WRONG (minor): "`#[cfg(test)]` module starting at line 298 ... ~297 non-test
  lines" -- actual attribute is at line 297 (~296 non-test). See Finding 4.
- IMPRECISE: D6's widget-width note omits the non-path String fixed `400.0` width
  (states only infinity + sliders). See Finding 3.
- UNRESOLVED-IN-DESIGN (not wrong, but deferred): the exact egui row construction
  that makes (a) the aligned `:`/value column and (b) the right-to-left
  provenance/Reset block coexist on one row, and the frame/indent inset
  interaction with char-count padding. The design asserts these work and offers a
  runtime "if it proves non-uniform" fallback rather than deciding. See Findings
  1 and 2.

## Prior findings re-check (carried-forward)

- MEDIUM-1 (description -> hover text) -- CLOSED. D6 keeps the description as a
  visible, selectable `Button::selectable` on a second line; not moved to tooltip.
  Consistent with FR7/AC 22.9 and the current code.
- MEDIUM-2 (key name -> inert label) -- CLOSED in intent. D6 retains a selectable
  key control. (Finding 2 refines HOW to keep it selectable while guaranteeing
  alignment -- a new concern about the mechanism, not a reopening of MEDIUM-2.)
- MEDIUM-3 (Save/dirty-row phantom Tab stop) -- CLOSED. D2/D7 pin the Filter
  `TextEdit` first in allocation order, dirty row as a non-interactive label; the
  retained first-Tab test is the proof. Matches the current allocation order.
- NIT-4 (400-line overstatement) -- CLOSED (modulo the off-by-one in Finding 4).
- NIT-5 (grid-rejection citation) -- CLOSED; `keys_editor_panel/render.rs` is the
  correct, verified location.
- NIT-6 (AC 22.4 vs FR3 continuation bar) -- CLOSED; D4/Out-of-scope state the
  two-level tree has no intermediate levels, so only tee + L-connector are drawn.
- NIT-7 (unverified Macro Library precedent) -- CLOSED; dropped in favour of the
  verified `calendar_header.rs` precedent.

## Verdict

2 MEDIUM + 2 NIT. Per the mechanical rule (any HIGH or MEDIUM => CHANGES_REQUESTED):
**CHANGES_REQUESTED.**

The two MEDIUMs are both about the one genuinely novel piece of this design --
making a left-aligned, char-count-padded `:`/value column coexist with the
existing selectable key button, the `ui.indent` wrapper, and the right-to-left
provenance/Reset block on each row. The design asserts it works and hedges with a
runtime fallback; it should instead lock the exact egui row structure and the
padding-to-pixel mechanism. Everything else (scope, model reuse, focus contract,
save convention, framework rails, the prior-review corrections) is sound and
accurately grounded in the verified code.
