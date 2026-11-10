# Config View re-skinned into a demarcated tree (Requirement 22)

The Configuration Context is re-drawn from an undifferentiated list into a tree: a ruled header area (Command line / Filter / `Unsaved edits : yes|no`), bracketed `[Group]` headers, box-drawing child connectors, and a `:`/value column aligned vertically across the currently-visible rows. The mechanism is a new pure `layout.rs` (connector prefixes + a monospace char-count `Value_Column` computed per frame) plus a rewrite of the key-row body in `render.rs` from a single `ui.horizontal` into a two-line block (line 1 = aligned key -> `:` -> value; line 2 = description + right-to-left provenance/staged/Reset). No state fields were added; the column is recomputed each frame. The change tracks the LOCKED design delta D1-D9 closely.

Watch for: nothing blocking. The grouping, collapse model, persistence, SAVE/END/RETURN semantics, dirty indicator, key/description selectability, and the value-widget/Reset/LOCKED/provenance behaviour are all preserved (confirmed by reading the code). Two items need the owner's eyes: the pixel-level `:` alignment is a monospace-font assumption that only a visual/full-gate check can confirm (likely correct), and the new collapse test drives the collapsed-state model directly rather than through the mouse/keyboard trigger AC 22.8 names (non-blocking, the trigger paths are unchanged).

**Verdict**: APPROVED

## High-level view

The re-skin is genuinely layout-only. Every behavioural seam the guardrails protect is reached through the same code it was before: grouping and collapse come from the untouched `visible_rows`; the group header is still a `selectable_label` that toggles `state.collapsed[ns]`; the value widget is still `render_widget` with its `Option<egui::Id>` recorded into `widget_ids` for Enter-to-focus; Reset still calls `stage_reset`; SAVE is still a `ShellRequest::Command("SAVE")`. The key name and description both remain selectable controls (a frameless `selectable_label` and a `Button::selectable` respectively) rather than being moved to hover or downgraded to inert text.

The dirty indicator is exact: the `Unsaved edits` row reads `state.is_dirty()`, which is `!working.is_empty()` -- literally AC 22.2. It is a plain `ui.label`, so it adds no Tab stop, and the Filter `TextEdit` is still allocated first in the header, so `InteriorFocus::single(filter_field_id())` and the first-Tab contract are preserved. The retained first-Tab test still binds that.

Alignment is a pure, per-frame computation over the visible key labels only, so collapse and filter narrow the column correctly. The connector choice is an O(1) look-ahead that reads "last child" as "next row is not a Key", which is correct given `visible_rows` emits a group's keys contiguously and only when expanded. The eight `layout.rs` unit tests cover the empty, single, multi-group, and filtered-subset cases and pass.

The one thing this review cannot settle from source is whether the `:` lands on the same pixel column on screen: that depends on the monospace font actually being monospace for the connector glyphs and key text, which is the documented justified-manual check.

<details>
<summary>Issues (3)</summary>

1. **Collapse test drives state directly (non-blocking)** — the new test inserts into `state.config_panel.collapsed` rather than clicking the header or sending a keyboard gesture, so it proves the collapsed-state model and bracketed-header rendering but not AC 22.8's named mouse/keyboard triggers. The trigger code is unchanged, so no regression; consider adding a click/keyboard assertion for full AC 22.8 coverage.
2. **Pixel alignment is a font assumption (needs owner check)** — char-count-to-pixel mapping is only exact if the rendered font is truly monospace for both connector glyphs and key text. Confirm visually or via the full gate; the design already lists this as the justified-manual item.
3. **Test count / task wording (nit)** — the brief said "7 unit tests"; `layout.rs` ships 8. No action needed.

</details>

<details>
<summary>Details</summary>

### Strict-preservation guardrails: all intact

I read each protected behaviour against the new code rather than trusting the comments.

Grouping and the 8 groups (AC 22.7): `visible_rows` in `tree.rs` is unchanged by this diff; it still groups by `namespace_of` into a sorted `BTreeMap` and emits one `Namespace` row per group. `render.rs` consumes that output unchanged. The only header-string change is `format!("{} {} ({})", ...)` -> `format!("{} [{}] ({})", ...)` -- a bracket literal around the same `ns_display_name(ns)`. Grouping mechanism untouched.

Expand/collapse trigger + collapsed-state model (AC 22.8): the group header is still `ui.selectable_label(...)`, and its `clicked()` arm still writes `state.collapsed.insert(ns, !now)` -- the same state the keyboard driver writes. `reduce_config_key` / `config_keyboard_effects` are not in the diff. `visible_rows` still only emits a group's key rows when `expanded`, so collapse still hides children. No regression.

Persistence and SAVE/END/RETURN (AC 22.11): no change. The in-panel Save button still enqueues `ShellRequest::Command("SAVE".to_string())`; `commit`/`stage_set`/`stage_reset`/`take_working`/`clear_working` are outside the diff. The value widget still edits the working buffer via `render_widget` (which reads `working_value` and stages, not persists).

Dirty reflection (AC 22.2): `Unsaved edits : {yes|no}` is built from `state.is_dirty()`, defined as `!self.working.is_empty()`. That is exactly "working-copy buffer non-empty", matching the AC verbatim. The staged-edit `*` and the `unsaved changes` chip also read `is_dirty()`/`working.contains_key` as before.

Key name selectable (AC 22.9 / Req 14.2): the key is now a frameless `ui.selectable_label(is_cursor, RichText::new(segment).monospace())` -- still a selectable control, not a downgrade to inert text or hover. The segment now carries `connector + key + padding + " : "`, so the whole left strip is one hit target.

Key description visible + selectable (AC 22.9 / Req 14.2): line 2 still renders the description via `egui::Button::selectable(false, RichText::new(entry.description).small().weak())` -- the same widget type as before, still visible and still selectable. It was NOT moved to a tooltip and NOT made inert. The only change is its vertical position (now line 2, previously stacked under the key in the old `ui.vertical`), which preserves its existing relationship to the key.

Value widget / Enter-to-focus / Reset / LOCKED / staged / provenance (AC 22.9): `render_widget` is called inside `add_enabled_ui(!locked, ...)` exactly as before, and its returned `Option<egui::Id>` is still inserted into `state.widget_ids` so Req 21.6 Enter-to-focus-value still resolves the id. The LOCKED/provenance badge, staged `*`, and `Reset` button (calling `stage_reset`, with the same `reset_enabled = !locked && (!is_at_default || has_staged_edit)` gate) are moved verbatim onto line 2's right-to-left sub-layout. The inline validation error line is retained. The only behavioural-looking change is that the value widget moved from a stacked line onto line 1 after the `:`; the widget itself and its width behaviour are unchanged (no width constraint added).

### Focus contract: preserved, test genuinely binds it

`WorkspaceContext::render` still returns `InteriorFocus::single(filter_field_id())` (unchanged in `mod.rs`). In the header, the Filter `TextEdit` carrying `filter_field_id()` is still allocated first; the new `Unsaved edits` row is a plain `ui.label`, which allocates no focusable widget and so cannot become a phantom Tab stop ahead of the Filter field. The Save button sits inside a right-to-left sub-layout allocated after the Filter field, as before.

The retained `full_shell_config_first_tab_focuses_filter_field` test is unmodified and still asserts (a) `first_interior_id == filter_field_id()`, (b) entry focus on the command field, and (c) the first `Tab` moves focus to exactly `filter_field_id()`. It was not weakened. The new `..._bracketed_group_headers_and_collapse_hides_children` test adds nothing focusable ahead of the filter, so AC 22.10 holds.

### Alignment correctness

`value_column_width` takes `max` of `chars().count()` over the labels passed, `+1`, and `unwrap_or(0)` for the empty set -- no panic on an empty visible set. In `render.rs` the input is built from `rows` (the freshly computed visible rows) filtering to `ConfigNodeId::Key`, so filtered-out and collapsed keys are excluded and the column is recomputed every frame. This matches AC 22.5/22.6: width over the currently-visible set, not all keys.

`aligned_left_segment` right-pads with `column.saturating_sub(label_len)` (no underflow panic) then appends `" : "`. For the widest label, `column = label_len + 1`, giving exactly one pad space before `" : "` -- the "+1" is deliberate and tested (`aligned_left_segment_widest_label_single_pad`). The `:` char-position equality across differing key lengths is tested (`aligned_left_segment_pads_to_column_and_aligns_colon`). The connector prefixes are both 4 cells, so they do not perturb the max; that invariant is also tested.

The residual uncertainty is pixel alignment: equal char counts map to equal pixels only if the font renders both the box-drawing connector glyphs and the ASCII key text at the same advance width. That is the documented justified-manual check (pixel-exact rendering), so it is correctly out of scope for the automated tests but should be eyeballed by the owner.

### Connector logic

`connector_for_row(rows, i)` returns `Last` when `rows.get(i+1)` is `None` or not a `Key`, else `Tee`. Because `visible_rows` emits each group's keys contiguously right after its `Namespace` row and only when expanded, the first non-Key row after a run of keys is always the next group's `Namespace` row (or end of list) -- so `Last` is picked for exactly the final visible child of each group, across collapse and filter states. Both prefixes are equal width, so the connector chosen while computing the column is irrelevant to the width (the code comments this and uses the real connector anyway). Correct.

### Box-drawing confinement and panics

The box-drawing code points (U+251C, U+2500, U+2514) appear only in the two `const` prefixes in `layout.rs` and their asserting unit tests. The `\u{21ba}` Reset glyph and `\u{25cf}` chip are pre-existing. No `unwrap`/`expect` was added on a non-test path: `render.rs` uses `saturating_sub`, `unwrap_or(0)`, `copied().unwrap_or(false)`, and pattern matches with `Err(_)` fallbacks; the only `.unwrap()` calls are inside `#[cfg(test)]`.

### File size

`render.rs` grew but `#[cfg(test)]` begins at line 353, so ~352 non-test lines -- under the 400-line rule. `layout.rs` is 208 lines total (incl. tests). Both fine.

### Verification run

`cargo test -p ff-desktop --bins config_panel::layout` compiled clean and ran the 8 layout tests green (log read back from `_ignore/logs/review-layout-test2.log`). Per the workspace execution split, the full suite / gate is the owner's run; this review did not run it.

</details>

<details>
<summary>File map</summary>

- `crates/ff-desktop/src/config_panel/layout.rs` (NEW) — pure connector/column helpers + 8 unit tests; no egui.
- `crates/ff-desktop/src/config_panel/mod.rs` — `mod layout;` only; state and focus contract unchanged.
- `crates/ff-desktop/src/config_panel/render.rs` — header `Unsaved edits` label + single separator; bracketed `[Group]` headers; two-line key row via `render_key_row`; per-frame `Value_Column`; `connector_for_row` look-ahead. ~352 non-test lines.
- `crates/ff-desktop/src/shell/tests_focus.rs` — first-Tab test retained unchanged; new bracketed-header + collapse-hides-children test added.
- `docs/specs/configuration-system/tasks.md` — task 37 marked complete.

Full diff: `git diff 23cbee6..a3059de` (captured in `_ignore/logs/review-diff-2026-11-01.log`).

</details>
