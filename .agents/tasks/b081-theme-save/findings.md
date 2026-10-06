# B081 -- Theme Editor "cannot SAVE a custom theme" -- Root-Cause Findings

## Root-cause summary (read first)

The theme save pipeline is actually correct and well tested at the **shell-apply layer**
(`apply_theme_editor_action`) and the **persistence layer** (`ff-theme` serialiser +
discovery): every `ThemeEditorAction` variant that reaches the shell does the right thing,
files are written to the user themes dir, and they round-trip (including `mode`, the B063
fix). The failure the owner sees lives in the **interactive render-to-action step that no
test exercises end to end**, combined with a UX contract gap. When the Theme Editor opens,
`selected` is the *active* theme, which in a default install is a **built-in**
(`Default Dark`/`Legacy`). The only always-enabled button is **Save**, and for a built-in
selection `apply_theme_editor_action(Save)` deliberately refuses to write and instead sets
an inline error telling the user to type a name and use Save As / Copy
(`commands_theme.rs` lines 116-124). The two buttons that *can* create a user theme --
**Copy** and **Save As** -- are `add_enabled(name_ok, ...)` and are therefore **disabled
until the user has already typed a name into the "New name" field** (`ff-theme-editor/src/lib.rs`,
the "Copy / Save As name field" block). So a user who edits colours and presses the obvious
"Save" button gets only an error message and no saved theme; the path that does work
(type a name, press the now-enabled Save As) is non-obvious and undiscoverable. The single
most likely mechanism the owner is hitting is **(ii)+(i) combined**: Save is produced but
intentionally no-ops into an error for a built-in selection, while the create-a-user-theme
actions are gated behind a non-empty name the UI never prompts for -- which is exactly the
"prompting for a new unique name" behaviour Requirement 20.4/20.5 specify but the render
does not implement. This is a **pure bug fix against existing criteria (Req 20.4, 20.5,
20.6)** -- no requirements gate needed.

---

## A. End-to-end path for Save and Save As

### Action production (pure render) -- `crates/ff-theme-editor/src/lib.rs`

The "Copy / Save As name field" block renders the name field then three buttons:

```rust
let name = state.name_buffer.trim().to_string();
let name_ok = !name.is_empty();
if ui.add_enabled(name_ok, egui::Button::new("Copy")).clicked() {
    action = ThemeEditorAction::Copy(name.clone());
}
if ui.add_enabled(name_ok, egui::Button::new("Save As")).clicked() {
    action = ThemeEditorAction::SaveAs(name.clone());
}
if ui.button("Save").clicked() {
    action = ThemeEditorAction::Save;
}
```

- **Save** is always enabled and produces `ThemeEditorAction::Save` (no name needed).
- **Save As** and **Copy** are `add_enabled(name_ok, ...)`, i.e. **disabled whenever
  `name_buffer` is empty**. They produce `SaveAs(name)` / `Copy(name)`.
- B052 clobber guard: a button action is kept in `action`; a token field's
  commit-on-lost-focus goes into a separate `token_action`; render returns `action` when
  set, else `token_action`. So a button press is not lost to a same-frame hex-field
  `lost_focus`. (This earlier bug is fixed and not the cause here.)

### Stashing (shell adapter) -- `crates/ff-desktop/src/theme_editor_panel.rs`

The `WorkspaceContext::render` impl calls the pure `render` and stashes the result:

```rust
self.pending_action = render(ui, self);
```

### Draining + applying -- `crates/ff-desktop/src/shell/render_body.rs` (TabKind::ThemeEditor arm)

```rust
let mut panel = std::mem::take(&mut self.theme_editor_panel);
self.render_workspace_context(ctx, ui, &mut panel);
let action = std::mem::take(&mut panel.pending_action);
self.theme_editor_panel = panel;
self.apply_theme_editor_action(action);
```

The owned-panel swap + apply is correct; the action is not dropped.

### Shell side effects -- `crates/ff-desktop/src/shell/commands_theme.rs::apply_theme_editor_action`

- `A::Save` (lines ~107-131): uses `(selected, working)`.
  - If `is_builtin_theme(selected)`: reads `name_buffer`. If **empty**, sets an inline
    error ("'...' is a built-in theme and cannot be overwritten. Enter a new name and use
    Save As (or Copy)...") and **writes nothing**. If non-empty, recurses into
    `A::SaveAs(new_name)`.
  - Else (user theme selected): `write_theme_file(name, working)`; clears error on success.
- `A::SaveAs(new_name)` (lines ~132-144): clones working, sets `p.name = new_name`,
  `write_theme_file`, then on success clears `name_buffer`, `refresh_theme_editor_list`,
  and `load_working(new_name, p)` so the editor now targets the new user theme.
- `A::Copy(new_name)` (lines ~94-105): same shape as SaveAs (writes a new file, refreshes,
  re-targets).

`write_theme_file` (same file) creates the themes dir, slugs the name, serialises via
`ff_theme::serialiser::serialise`, and writes `<themes>/<slug>.toml`, mapping IO errors to
a `String` (surfaced in `state.error`). It does **not** silently swallow errors with
`let _ =`.

**Conclusion for A:** once an action reaches the shell it is handled correctly. The
weakness is upstream (which action the UI lets the user produce) and in the built-in-Save
contract.

---

## B. Why a created custom theme is not saved -- candidate causes

- **(i) The create action is never produced by render -- CONFIRMED as a contributing cause.**
  `Save As` and `Copy` are `add_enabled(name_ok, ...)` where `name_ok = !name_buffer.trim().is_empty()`.
  On opening, `name_buffer` is empty, so **both buttons are disabled**. The user cannot
  create/save a new theme until they discover the "New name" field and type into it. There
  is no prompt. Requirement 20.4 ("prompting for a new unique name") and 20.5 ("behave as
  Save_As (prompting for a new user-theme name)") are **not** implemented in the render.

- **(ii) The action is produced but `apply_theme_editor_action` no-ops -- CONFIRMED for the
  built-in Save path.** `A::Save` with a built-in `selected` and an empty `name_buffer`
  deliberately writes nothing and only sets `state.error` (`commands_theme.rs` 116-124).
  Because the editor opens with a built-in selected by default, the obvious "Save" button
  appears to do nothing (it shows a red inline message). This matches the owner's "does not
  allow me to SAVE" verbatim.

- **(iii) Wrong/missing write dir or swallowed error -- RULED OUT.** `themes_dir()` resolves
  the test override or `<User_Data_Dir>/themes/`; `write_theme_file` does
  `create_dir_all` first and returns the IO error as a `String` shown inline. No `let _ =`
  swallowing on the write.

- **(iv) Saved theme not discoverable -- RULED OUT.** After Copy/SaveAs the code calls
  `refresh_theme_editor_list()` (-> `ff_theme::list_all_themes`) and `load_working(new_name,...)`.
  `list_all_themes` includes user `.toml` files (de-duplicated, built-in wins). A saved user
  theme is listed and selectable; `theme_editor_set_active_...` / `set_active_theme`
  (`render_theme.rs` 120) applies and persists it via `theme.active_name`.

- **(v) Validation rejects the custom theme -- RULED OUT.** Serialiser/loader round-trip is
  intact, including `mode` (B063 fix, `serialiser.rs` tests `serialise_round_trip_preserves_legacy_mode`).
  No schema/`allowed_values` rejection sits on this path.

---

## C. Single most likely root cause

**Mechanism:** The Theme Editor opens with a **built-in** theme selected (the active
palette), and the UI's primary, always-enabled control is **Save**. For a built-in,
`apply_theme_editor_action(A::Save)` with an empty `name_buffer` refuses to write and sets
only an inline error (`crates/ff-desktop/src/shell/commands_theme.rs`, approx. **lines
116-124**). The actions that would actually create a persisted user theme -- **Copy** and
**Save As** -- are gated by `add_enabled(name_ok, ...)` in the render
(`crates/ff-theme-editor/src/lib.rs`, the "Copy / Save As name field" block) and are
**disabled until the user has already typed a name**, which the UI never prompts for. Net
effect: a user editing a built-in and pressing Save cannot save; the working path is
undiscoverable.

**Affected actions:** **Save** is affected (no-op-with-error for a built-in selection with
no typed name). **Save As** and **Copy** are "affected" only in that they are disabled
until a name is typed -- when invoked with a name they work correctly (proven by the
direct-call tests). So the user-visible failure is primarily the **Save** button plus the
missing name prompt required by Req 20.4/20.5.

**Suggested fix direction (implement nothing yet):** make the create-a-user-theme path
reachable from a built-in selection in one obvious step -- e.g. when `Save` is pressed on a
built-in with an empty name, focus/prompt the "New name" field (or auto-suggest a unique
derived name) and perform Save As, exactly as Req 20.5 specifies ("behave as Save_As
(prompting for a new user-theme name)"); and/or keep Copy/Save As visible with inline
guidance rather than silently disabled. The shell-apply and persistence layers need no
change.

---

## D. Acceptance-criteria coverage (requirements gate decision)

Criteria for "create + Save / Save As a custom theme persists and becomes selectable"
**already exist** in `docs/specs/theme-and-appearance/requirements.md`, Requirement 20
(Theme Editor Context):

- **Req 20.4** -- Copy_Theme creates a new theme initialised from the selected theme,
  **prompting for a new unique name**; the copy becomes the edit target (derive a custom
  theme from a built-in without altering the built-in).
- **Req 20.5** -- Save writes `themes/<slug>.toml`; Save_As writes a new named file; for a
  built-in, Save **SHALL behave as Save_As (prompting for a new user-theme name)** OR be
  disabled with a message directing the user to Copy / Save As. Save/Save_As only ever
  write USER files.
- **Req 20.6** -- Set_Active makes the saved theme active and persists it for future
  launches (so it "becomes usable").

Because the behaviour is already specified, **the fix is a pure bug fix against existing
criteria (20.4, 20.5, 20.6) and does NOT require a new requirements gate.** The current
implementation partially satisfies 20.5 (it does disable/guide), but does not satisfy the
"prompting for a new unique name" half of 20.4/20.5 that makes creation discoverable, which
is the gap to close.

---

## E. Existing tests on the theme save / Save As path (for TDD)

All in `crates/ff-desktop/src/shell/tests_menu_workspace.rs` (CR-NR-074 block), driving
`apply_theme_editor_action` **directly** (they bypass the pure `render`, so the button
enablement / name-prompt gap is NOT covered):

- `theme_editor_copy_creates_new_named_theme_file` -- Copy writes a file, re-targets, lists it.
- `theme_editor_save_writes_edited_colour_to_disk` -- Copy -> EditToken -> Save persists.
- `theme_editor_save_as_writes_new_file` -- SaveAs writes `<name>.toml`, re-targets.
- `theme_editor_save_as_after_edit_writes_file_b052` -- SaveAs after an EditToken (B052).
- `theme_editor_set_active_swaps_palette_and_persists` -- SetActive applies + persists.
- `theme_editor_save_on_builtin_does_not_write_builtin` -- Save on a built-in with empty
  name writes nothing and sets an error (this is the behaviour the owner is hitting; the
  test asserts the *refusal*, not a usable save).
- `theme_editor_reset_loads_builtin_baseline`, `theme_editor_reset_non_builtin_errors`,
  `theme_editor_edit_token_updates_working_and_previews`,
  `theme_editor_list_has_no_duplicates`, `theme_editor_does_not_materialise_builtins`.

Serialiser round-trip tests in `crates/ff-theme/src/serialiser.rs`:
`serialise_round_trip_preserves_colours`, `serialise_round_trip_preserves_legacy_mode`
(B063), `load_without_mode_field_uses_passed_default`, `serialise_round_trip_preserves_fonts`.

Discovery/list tests in `crates/ff-theme/src/discovery.rs`:
`list_all_themes_includes_user_themes`, `list_all_themes_dedups_builtin_named_user_file`,
`is_builtin_theme_identifies_builtins`, `export_theme_round_trips_name`.

Full-shell `egui_kittest` tests exist for the Theme Editor **focus/Tab order only**
(`crates/ff-desktop/src/shell/tests_focus.rs::full_shell_theme_editor_first_tab_focuses_theme_selector`
and the bare-THEME open tests). **There is NO full-shell kittest that types a name and
clicks Save/Save As**, which is exactly the untested seam where this bug lives. A TDD fix
should add such a test: open THEME, drive the render to type a name and press Save (on a
built-in), and assert a user `.toml` is written and the theme becomes selectable/active --
per `workspace-conformance.md` the rendered-widget behaviour needs an `egui_kittest` test.
