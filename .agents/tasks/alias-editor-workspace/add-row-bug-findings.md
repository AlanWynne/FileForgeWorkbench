# Alias Editor "no rows, no Add" bug -- findings report (CR-NR-113)

Read-only investigation against the main working tree at
`c:\workspace\apps\FileForgeWorkbench`, HEAD `aeff0d5`. The Alias Editor feature
landed as a single commit `422a19a` ("feat(ff-desktop): Alias Editor Context +
user alias store (CR-NR-113)"). No code was changed.

---

## 1. Summary answer (root cause)

The committed Alias Editor code at HEAD is, on inspection, **functionally wired
end to end**: an "Add alias" button IS rendered, it DOES emit
`AliasEditorAction::AddRow`, the shell handler DOES append a blank row, and the
panel DOES render from that working copy. There is no missing dispatch, no
missing handler arm, and the working copy is initialised when the editor opens.
A full-shell `egui_kittest` test even proves the "Add alias" button is laid out
and keyboard-focusable in the real shell when the store is empty.

Because of that, the source does **not** contain an obvious, unconditional
"header-only" defect of the render-gap or wiring-gap kind. The investigation
instead lands on two concrete, defensible conclusions, in priority order:

- **Primary (most-supported): a test-coverage blind spot masks a
  render-visibility / layout risk that is not provable from source alone.** No
  test ever drives the UI Add control and asserts a NEW ROW VISIBLY RENDERS.
  Every "add/delete" assertion calls the shell handler directly or assigns
  `rows` programmatically. The one render-level test asserts only that the Add
  button has a focusable `egui::Id` -- a widget can own an id and be
  Tab-focusable while being laid out BELOW the visible viewport (clipped). So an
  "Add button present but off-screen / not usable" symptom would pass every
  existing test. The suspect layout construct is the footer-after-scroll-area
  pattern in `alias_editor_panel/render.rs` (the `ScrollArea` is sized to
  `available_height - 72`, footer rendered after it).

- **Secondary (environment, strongly implied by the original transcript): the
  owner's RUNNING binary did not contain the fix when they looked.** The
  transcript shows `cargo build` failing with `error: failed to remove file ...
  ffwb.exe -- Access is denied (os error 5)` because a running instance held the
  exe locked (original request, message 8), and an earlier complaint that
  "option a is still config" (message 7) -- the pre-CR-CH-066 layout. A running
  process started before the rebuild would still show old behaviour regardless
  of the on-disk source.

Both must be stated because the source is clean enough that a hard,
source-provable "this line is the bug" render gap does not exist; the failure
that reaches the user is a combination of an untested render surface and a
likely stale/locked runtime.

---

## 2. Evidence -- how Add is meant to work, and whether each layer is present

### 2.1 The Add affordance IS rendered and DOES emit AddRow

`crates/ff-desktop/src/alias_editor_panel/render.rs`:

- Header row (the only thing the owner sees) is drawn unconditionally at
  `render.rs:64-72` ("Alias / Root / Env / Template" strong labels + separator).
- The editable rows are drawn in a `ScrollArea::both()` at `render.rs:76-116`.
- The footer with the **"Add alias"** button and **"Save"** button is drawn
  AFTER the scroll area at `render.rs:118-144`. The Add button is created via
  `ui.push_id(add_button_id(), ...)` with the stable id
  `egui::Id::new("alias_editor_add_row")` (`render.rs:14-16, 124-130`).
- On click it sets `action = AliasEditorAction::AddRow` (`render.rs:134-135`).
- When the store is empty (`row_count == 0`) the Add button is reported as the
  first interior focus stop (`render.rs:131-133`).

So the Add control is a **button in the footer**, not a key binding or a
command-line verb, and it is unconditionally constructed each frame. This is a
render that, by code, DOES draw an Add control.

### 2.2 AddRow IS handled (appends to the working copy the panel renders)

`crates/ff-desktop/src/shell/alias_editor.rs:80-96`
(`apply_alias_editor_action`):

```
A::AddRow => { self.alias_editor_panel.rows.push(AliasRowEdit::default()); }
A::DeleteRow(index) => { if index < ... { self.alias_editor_panel.rows.remove(index); } }
A::Save => self.save_user_aliases(),
```

The render arm in `crates/ff-desktop/src/shell/render_body.rs:201-211` does the
owned-panel swap correctly: `mem::take` the panel -> `render_workspace_context`
-> drain `pending_action` -> put the panel BACK (`self.alias_editor_panel =
panel`) -> THEN `apply_alias_editor_action(action)`. Because the panel is
restored before the action is applied, the `rows.push(...)` lands on the live
panel and the appended row renders on the next frame. The wiring is correct.

Note: `AliasEditorAction` has four variants only -- `None / AddRow / DeleteRow /
Save` (`alias_editor_panel/state.rs:28-42`). The brief mentions a
`Select/AddRow/DeleteRow/Save/SaveAs` family; this editor has **no SaveAs and no
Select** variant (it is a flat grid, not a per-item selector like the Menus
Editor). That is a design simplification, not the bug.

### 2.3 Why there are no rows when the store is empty -- EXPECTED, and the grid is still editable

An absent/empty `User_Alias_Store`
(`<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`, created on first save)
is expected to yield zero rows. `load_user_alias_rows`
(`shell/alias_editor.rs:57-76`) returns `Vec::new()` for an absent/unreadable or
malformed file. The render does NOT gate the footer on a non-empty store -- the
Add button is drawn whether `rows` is empty or not (`render.rs:118-144`,
unconditional). So, by code, an empty store should still show an Add button.
This is the crux the brief flags, and the code handles it correctly on paper.

### 2.4 The working copy IS initialised on open

`open_alias_editor` (`shell/alias_editor.rs:33-53`) calls
`self.alias_editor_panel.load_rows(rows)` BEFORE navigating, and `load_rows`
(`state.rs:72-76`) assigns `self.rows = rows` (an empty `Vec`, not `None`) and
clears error/pending-delete. `rows` is a plain `Vec<AliasRowEdit>`, never an
`Option`, so there is no "None vs empty Vec" state-init gap. The render iterates
`state.rows.iter_mut()` (`render.rs:84`) which is simply empty on first open.
**No state-init gap.**

### 2.5 The dispatch chain Settings->A / ALIASES reaches the editor

- Settings menu option `A` has `command = "Aliases"`
  (`menu_workspace/defaults.rs:89-93`; order C/A/T/M/K/W per CR-CH-066).
- `builtin_workspace_target_for` maps `ALIASES` -> `custom("aliases")`
  (`command_config/mod.rs:184, 268`).
- `target_dispatch.rs:267-270` dispatches `"aliases"` -> `self.open_alias_editor()`.
- `open_alias_editor` navigates to a `CommandConfigurator` descriptor with
  `editor = "aliases"` (`shell/alias_editor.rs:38-52`).
- `nav_reconstruct.rs:212-214` maps `editor == "aliases"` ->
  `set_active_tab_context(TabKind::AliasEditor, "[ALIASES]")`.
- `render_body.rs:201` dispatches `TabKind::AliasEditor` to the render arm.

The whole chain is present and consistent. This matches the passing test
`settings_alias_option_dispatches_aliases_verb`
(`shell/tests_alias_editor.rs:52-72`).

---

## 3. The precise test-coverage gap (how an unusable Add passed review)

Tests live in `crates/ff-desktop/src/shell/tests_alias_editor.rs` (shell
behaviour), `crates/ff-desktop/src/shell/alias_editor.rs` `#[cfg(test)]`
(validation only), and one full-shell focus test in
`crates/ff-desktop/src/shell/tests_focus.rs`.

### What IS covered

- Open/routing: `aliases_verb_opens_the_alias_editor_context`,
  `settings_alias_option_dispatches_aliases_verb`,
  `alias_editor_end_returns_to_opener_via_nav_stack`
  (`tests_alias_editor.rs:35-108`).
- AddRow/DeleteRow effect on the model:
  `add_and_delete_mutate_the_working_copy` (`tests_alias_editor.rs:112-123`) --
  but this calls `shell.apply_alias_editor_action(AliasEditorAction::AddRow)`
  **directly**; it never renders the panel, never clicks the Add button, and
  never asserts a row renders.
- Save/validate/resolver-refresh: `save_writes_file_...`,
  `refresh_keeps_base_layer_...`, `user_rows_route_...`,
  `load_skips_user_row_that_shadows_builtin_root`,
  `save_validation_failure_leaves_file_unchanged`,
  `parameterised_alias_authored_in_editor_expands_end_to_end`
  (`tests_alias_editor.rs:127-299`) -- every one assigns
  `shell.alias_editor_panel.rows = vec![...]` programmatically, bypassing the
  render entirely.
- Validation unit tests (`shell/alias_editor.rs` test mod) -- pure function, no
  render.
- One full-shell render test:
  `full_shell_alias_editor_first_tab_focuses_first_interior`
  (`tests_focus.rs:1163-1196`). It renders the real shell at 1200x900, asserts
  `focus.first_interior_id.is_some()`, and that the first Tab lands on it.

### What is NOT covered (the gap that let it through)

No test ever:

- clicks / activates the rendered "Add alias" button, nor
- asserts that activating Add makes a NEW EDITABLE ROW appear in the RENDERED
  panel (a visible surface/root/env/template row), nor
- asserts the footer (Add/Save) is actually WITHIN the rendered viewport rather
  than merely laid out with an id.

The sole render-level test (`...first_tab_focuses_first_interior`) proves only
that the Add button has a focusable `egui::Id` and that Tab reaches it. In egui,
a widget is assigned its id and becomes Tab-focusable during layout **even when
it is scrolled or clipped out of the visible region**. So "Add button exists and
is focusable" was proven; "Add button is visible and clicking it shows a new
row" was never proven. That is exactly how an Add affordance that the user
cannot see or use would still be reviewed APPROVED and gate-clean: the
acceptance evidence for Req 5.3/5.10 is the handler-level and focus-id tests, not
a render-visibility/interaction test.

---

## 4. Comparison with the Menus Editor (the model CR-NR-113 was built on)

`crates/ff-desktop/src/menus_editor_panel/render.rs`:

- Same footer-after-scroll-area pattern: options in a
  `ScrollArea::both().auto_shrink([false,false]).max_height(available - 64)`
  (`menus_editor_panel/render.rs` options block), footer (`Add option`, `Save`,
  `Save As`) created AFTER it with the explanatory comment that the reserve keeps
  the footer on screen.
- KEY DIVERGENCE: the Menus Editor has an **early return** before the scroll
  area/footer when no menu is selected
  (`let Some(menu) = state.working.as_mut() else { ui.label(select-prompt);
  return action; }`). So its "empty" state shows a selector + prompt, not an
  empty grid + footer. It is never exercised in the "empty working copy, footer
  must show" configuration the Alias Editor is in on first open.
- The Alias Editor mirrors the footer-reserve pattern but has NO early return
  and ALWAYS renders the empty grid + footer on first open. It therefore relies
  on the `available_height - footer_reserve` sizing being correct in the
  **empty** case -- a case the Menus Editor layout is never asked to handle and
  which, per section 3, the Alias Editor has no render test for.

So the Alias Editor did mirror the Menus Editor's Add/Save footer mechanism
(button in a footer emitting a structural action), but it diverges in the one
dimension that matters for the reported symptom: it renders the footer in the
empty-store state, and that state has no visibility test.

The Menus Editor's Add path, by contrast, is further guarded by its own
behaviour tests (e.g. `menus_editor_command_and_description_fields_accept_typed_input`,
`menus_editor_every_stable_control_is_tab_reachable` in
`menus_editor_panel/render.rs` tests) that drive the rendered panel through
`egui_kittest`. The Alias Editor has no equivalent panel-interaction test.

---

## 5. Classification of the defect

- Render gap (no Add control drawn): **NO** -- the Add button is unconditionally
  created (`render.rs:124-135`).
- Wiring gap (control drawn but AddRow not emitted/handled): **NO** -- click sets
  `AddRow` (`render.rs:134`), handler appends (`shell/alias_editor.rs:88-90`),
  owned-panel swap applies it to the live panel (`render_body.rs:201-211`).
- State-init gap (working copy not initialised): **NO** -- `rows` is a `Vec`
  initialised via `load_rows` on open (`shell/alias_editor.rs:35`,
  `state.rs:72-76`).
- **Test-coverage gap (render-visibility / interaction untested): YES** -- the
  decisive, source-provable defect. Combined with the footer-after-scroll-area
  layout risk in the empty-store case (`render.rs:76-144`) and the likely
  stale/locked running binary from the transcript, this is what produces a
  "header only, cannot add" experience that nonetheless passed review.

---

## 6. Recommended fix approach (implement nothing here)

1. **Make the empty-store render robust and the Add affordance unmissable.**
   Rather than relying on `available_height - footer_reserve` sizing with the
   footer drawn after a space-filling `ScrollArea` (`render.rs:76-144`), render
   the Add/Save footer in a dedicated bottom region that cannot be pushed out of
   view -- e.g. draw the footer FIRST into a reserved strip (or use
   `ui.allocate_ui_at_rect` / a `TopBottomPanel`-style reservation) and the rows
   scroll area into the remaining space, or in the empty case render an explicit
   "No aliases yet -- click Add alias to create one" hint next to the button.
   This mirrors the Menus Editor's intent (footer always on screen) but for the
   empty-grid state the Menus Editor never enters.

2. **Add the missing full-shell render/interaction test (the test that should
   have caught it).** In `tests_focus.rs` / a new `tests_alias_editor` full-shell
   block, using `harness_shell()` (`shell/tests_common.rs:318-333`,
   `build_eframe`, 1200x900):
   - open the editor via `dispatch_command_string("ALIASES")` and run frames;
   - assert the store is empty (0 rows);
   - locate the Add button by its stable id `egui::Id::new("alias_editor_add_row")`
     and assert it is WITHIN the visible viewport (e.g. its widget rect is
     contained in the central panel's clip rect, not merely that it has an id);
   - simulate a click on the Add button (not a direct `apply_alias_editor_action`
     call), run a frame;
   - assert `shell.alias_editor_panel.rows.len() == 1` AND that the new row's
     surface field (`egui::Id::new(("alias_surface", 0))`) is now rendered and
     editable (focus it and inject text, assert it persists into
     `rows[0].surface`), mirroring the Menus Editor's
     `menus_editor_command_and_description_fields_accept_typed_input`.
   This asserts the Add control is both VISIBLE and FUNCTIONAL through the render
   path, closing the exact gap in section 3.

3. **Operational check for the owner (secondary cause).** Fully exit the running
   `ffwb.exe` so the lock is released, rebuild from HEAD `aeff0d5` (the failed
   build in transcript message 8 left the on-disk exe potentially out of step
   with the running process), and relaunch. If the Add button then appears, the
   source was never the defect for that session; the test in step 2 still must be
   added to guarantee it stays usable.

---

## 7. Key file:line index

- `crates/ff-desktop/src/alias_editor_panel/render.rs:64-72` -- header row (the
  only thing the owner sees).
- `crates/ff-desktop/src/alias_editor_panel/render.rs:76-116` -- rows scroll area
  (`available_height - 72`, `auto_shrink([false,false])`).
- `crates/ff-desktop/src/alias_editor_panel/render.rs:118-144` -- footer with the
  "Add alias" button (stable id `alias_editor_add_row`) + "Save".
- `crates/ff-desktop/src/alias_editor_panel/state.rs:28-42` -- `AliasEditorAction`
  (`None/AddRow/DeleteRow/Save`; no SaveAs/Select).
- `crates/ff-desktop/src/alias_editor_panel/state.rs:72-76` -- `load_rows`
  (working copy init, `Vec` not `Option`).
- `crates/ff-desktop/src/shell/alias_editor.rs:33-53` -- `open_alias_editor`
  (loads rows, navigates).
- `crates/ff-desktop/src/shell/alias_editor.rs:80-96` -- `apply_alias_editor_action`
  (AddRow appends).
- `crates/ff-desktop/src/shell/render_body.rs:201-211` -- AliasEditor render arm
  (owned-panel swap, correct order).
- `crates/ff-desktop/src/shell/nav_reconstruct.rs:212-214` -- `editor=="aliases"`
  -> `TabKind::AliasEditor`.
- `crates/ff-desktop/src/command_config/mod.rs:184,268` -- `ALIASES` ->
  `custom("aliases")`.
- `crates/ff-desktop/src/shell/target_dispatch.rs:267-270` -- `"aliases"` ->
  `open_alias_editor`.
- `crates/ff-desktop/src/menu_workspace/defaults.rs:89-93` -- Settings `A` option
  (`command = "Aliases"`).
- `crates/ff-desktop/src/shell/tests_alias_editor.rs:112-123` --
  `add_and_delete_mutate_the_working_copy` (calls handler directly, no render).
- `crates/ff-desktop/src/shell/tests_focus.rs:1163-1196` --
  `full_shell_alias_editor_first_tab_focuses_first_interior` (focus-id only, no
  click, no visibility assertion).
- `crates/ff-desktop/src/menus_editor_panel/render.rs` -- Menus Editor model:
  footer-after-scroll pattern + early return on no selection; panel-interaction
  tests present.
- `crates/ff-desktop/src/shell/tests_common.rs:318-333` -- `harness_shell`
  (`build_eframe`, 1200x900) for the recommended full-shell test.

### Environment / timing evidence
- Feature commit `422a19a` authored 2026-10-25 13:31 +0200; HEAD `aeff0d5`
  2026-10-25 18:43 +0200; on-disk `target/debug/ffwb.exe` LastWriteTime
  2026-10-25 19:36 (local). The on-disk exe is newer than the feature commit, so
  the committed source is the built source -- but a process started before the
  rebuild (consistent with the locked-exe build failure in transcript message 8)
  would still show stale behaviour. Hence the stale-binary cause is a *running
  process* question, not an on-disk source question.

### Tooling note
Several wrapped PowerShell commands returned `Exit Code: -1` (the documented
intermittent capture artifact on this machine); results were confirmed by
reading the log files the commands wrote (`tools/logs/exe-time.log`,
`tools/logs/commit-time.log`, `tools/logs/head-time.log`) and via dedicated
read/grep tools. No code was modified.
