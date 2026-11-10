# Save-Convention Audit (B081) -- Findings

READ-ONLY diagnosis. No source, spec, or requirement files were changed. This
report audits every core wired-in Workspace Context against the ISPF-style
explicit-save convention in `.kiro/steering/save-convention.md` and pins down
the reported SAVE bug.

---

## 1. Summary answer (read this first)

There is a SINGLE shared SAVE / END / RETURN / CANCEL / REFRESH seam in the
FFCMD command base (`crates/ff-desktop/src/shell/commands_ladder_a.rs`), and it
routes to each editable Context through `editable_context_kind()` in
`crates/ff-desktop/src/shell/editable_context.rs`. That seam is the mechanism the
convention needs -- but it is wired for EXACTLY ONE Context:

- `editable_context_kind()` returns `Some(EditableKind::AliasEditor)` only for
  the Alias Editor (the Command Configurator's ISPF alias table). For EVERY
  other editable Context it returns `None` (an explicit `_ => None` with a
  documented TODO).

Consequences, which match the owner's report precisely:

- (a) "On SOME panels the SAVE command does not work; saving only happens on
  END/exit." -- Actually it is worse than "only on END." For the five sibling
  editors (Settings/Config, Theme, Menus, Keys, Kinds) the TYPED `SAVE` command
  is a NO-OP (returns `None` -> no persist), AND `END`/`RETURN` do NOT persist or
  discard either -- they are plain navigation. These panels persist ONLY through
  their in-panel BUTTONS (each panel's `A::Save` action), never through the
  ISPF command verbs. The one partial exception is the Config/Settings panel,
  which has NO Save button at all and instead AUTOSAVES on every edit.

- (b) "SAVE may not be working at all in the Theme workspace." -- CONFIRMED. In
  the Theme Editor:
  - The typed `SAVE` command does NOT persist (ThemeEditor is not wired into the
    seam; `SAVE` is a no-op).
  - `END` (F3) does NOT persist the theme -- it is plain `nav_end()` navigation
    (the END save-then-leave branch only runs when `editable_context_kind()` is
    `Some`, which it is not for ThemeEditor). So the convention's "END implies
    save" does NOT happen for the theme.
  - `RETURN` does NOT discard the working copy via the seam (no-op discard) --
    and the live-preview mutation of `self.palette` is left applied until a theme
    reload/relaunch, so RETURN does not cleanly revert the preview.
  - The ONLY way to persist a theme to its backing file is the in-panel "Save" /
    "Save As" BUTTON (`ThemeEditorAction::Save` -> `apply_theme_editor_action`).
    If the user edits colours and then issues SAVE or presses F3/END expecting a
    commit, NOTHING is written. That is the "save not working at all" symptom.

So the root cause is uniform and singular: the explicit-commit convention is
implemented by a shared seam that was only ever wired for the Alias Editor
(CR-CH-071 v1), and the five sibling config editors were left as a documented
"fast-follow" that was never done. The Theme Editor additionally has a
live-preview that mutates the running palette during edit (a blurred boundary
the convention explicitly calls out).

---

## 2. Master list of wired-in Workspace Contexts

Source of truth: the central-panel `match self.tabs.active_tab().kind` in
`crates/ff-desktop/src/shell/render_body.rs` (`render_active_tab_body`), which
enumerates every `TabKind` arm. The dispatch framework
(`render_workspace_context` / `apply_interior_focus`) is in
`crates/ff-desktop/src/shell/render.rs`.

| # | TabKind arm | Context | Editable? |
|---|-------------|---------|-----------|
| 1 | `FilesPanel` | Catalog Explorer (PARKED core-001) | n/a (parked notice) |
| 2 | `FileEditor` / `Untitled` | Editor (PARKED core-001) | n/a (parked notice) |
| 3 | `FileExplorerPanel` | File Explorer (PARKED core-001) | n/a (parked notice) |
| 4 | `HelpContext` | Help | NON-EDITABLE (display/search) |
| 5 | `ConfigPanel` | Settings / Config | EDITABLE |
| 6 | `PluginManager` | Plugin/Extension Manager | NON-EDITABLE (actions, no doc) |
| 7 | `EventLog` | Event Log | NON-EDITABLE (display + clear) |
| 8 | `ScrmViewer` | Screen Collection viewer | NON-EDITABLE (display/replay) |
| 9 | `SearchResults` | Search Results | NON-EDITABLE (query/results) |
| 10 | `MacroLibrary` | Macro Library | NON-EDITABLE (list + run/actions) |
| 11 | `MenuWorkspace(_)` | Home Context (POM) / named menus | NON-EDITABLE (menu display) |
| 12 | `ThemeEditor` | Theme Editor | EDITABLE |
| 13 | `MenusEditor` | Menus Editor | EDITABLE |
| 14 | `KeysEditor` | Keys Editor | EDITABLE |
| 15 | `KindsEditor` | Kinds Editor | EDITABLE |
| 16 | `AliasEditor` | Alias Editor (Command Configurator ISPF) | EDITABLE |
| 17 | `CommandConfigurator` | Command Configurator (definitions form) | EDITABLE |

Editable Contexts in scope: ConfigPanel, ThemeEditor, MenusEditor, KeysEditor,
KindsEditor, AliasEditor, CommandConfigurator. The three PARKED arms render an
"unavailable in core" notice and report `InteriorFocus::none()`; they own no
editable state in the core build (documented no-interior exception). Help, Event
Log, Scrm viewer, Search Results, Macro Library, and the Menu/POM Contexts are
read-only/action Contexts with no persisted working copy, so the save convention
does not apply (NO-SAVE-NEEDED).

---

## 3. The SAVE / END / RETURN / F3 dispatch analysis

### The shared seam (this is the key architecture fact)

`crates/ff-desktop/src/shell/commands_ladder_a.rs`, `try_commands_a`:

- `SAVE` (line ~224): `if self.editable_context_kind().is_some() { let _ =
  self.editable_persist(); }` else it is a no-op. SAVE persists and STAYS.
- `REFRESH` (~243): `editable_discard()` + `editable_reload()` + STAY.
- `CANCEL` (~255): `editable_discard()` then `nav_end()`.
- `END` (~258): when `editable_context_kind().is_some()`, `editable_persist()`;
  on Ok -> `nav_end()`, on Err -> STAY with the inline error. Otherwise plain
  `nav_end()` navigation (no save).
- `RETURN` (~300): `editable_discard()` then `nav_return()` (collapse to the
  tab's visual root). On a non-wired Context the discard is a no-op.

F3 is mapped to `END` in the default key map
(`crates/ff-desktop/src/shell/tests_command.rs` asserts F3 -> `END`;
`config_panel/render.rs` f3 test confirms the binding), so F3 == the END path
above.

### The seam's wiring table -- ONLY the Alias Editor

`crates/ff-desktop/src/shell/editable_context.rs`, `editable_context_kind()`:

```
match self.tabs.active_tab().kind.tag() {
    KindTag::AliasEditor => Some(EditableKind::AliasEditor),
    // TODO(CR-CH-071 fast-follow ... wire ConfigPanel / MenusEditor /
    //   ThemeEditor / KeysEditor / KindsEditor) -- until wired they fall
    //   through as non-editable (plain nav), Req 20a.14.
    _ => None,
}
```

`editable_persist` / `editable_discard` / `editable_reload` likewise only
match `EditableKind::AliasEditor`; every other Context returns `Ok(())` /
no-op. So the typed SAVE/END/RETURN verbs persist/discard ONLY for the Alias
Editor. The five sibling editors fall through: SAVE is a no-op, END is plain
navigation (no save), RETURN is plain navigation (no discard).

### How each editable Context ACTUALLY persists today

| Context | Working copy on entry? | Persist trigger that WORKS | Typed SAVE verb | END persists? | RETURN discards? |
|---------|------------------------|----------------------------|-----------------|---------------|------------------|
| ConfigPanel | NO (edits live store) | autosave on every widget change | no-op | n/a (already saved) | NO (edits already persisted) |
| ThemeEditor | YES (`load_working`) | in-panel Save/Save As button only | no-op | NO | NO (preview left applied) |
| MenusEditor | YES (`load_working`) | in-panel Save/Save As button only | no-op | NO | NO |
| KeysEditor | YES (`load_kind`) | in-panel Save button only | no-op | NO | NO |
| KindsEditor | YES (`load_kind`) | in-panel Save button only | no-op | NO | NO |
| AliasEditor | YES (`load_rows`) | seam SAVE / END / command line | WORKS | YES | YES |
| CommandConfigurator | partial (per-row edit form) | form Save button + Delete (immediate) | no-op | NO | NO |

---

## 4. Per-Context compliance verdicts (with evidence)

### 4.1 Alias Editor (`AliasEditor`) -- COMPLIANT

- Working copy on entry: `open_alias_editor` -> `load_effective_alias_rows` ->
  `alias_editor_panel.load_rows(rows)`
  (`crates/ff-desktop/src/shell/alias_editor.rs`).
- SAVE persists + stays: seam `editable_persist` -> `alias_editor_persist` ->
  `save_user_aliases` (validate -> `write_override_rows` -> refresh live store),
  returns `Result`. SAVE verb stays in the Context.
- END persists then exits: the END branch calls `editable_persist()` then
  `nav_end()` on Ok, stays on Err (commands_ladder_a.rs).
- RETURN discards then exits: `editable_discard` -> `alias_editor_discard`
  (clears rows + error, no write) then `nav_return()`.
- Typed command line SAVE/END/RETURN route through the front door
  (`apply_alias_command_line` in `alias_editor_line_commands.rs` ->
  `dispatch_command_string`), so typed and F3 behave identically.
- No autosave: per-field edits mutate working rows only; nothing writes until a
  commit verb.

This is the reference implementation the convention describes, and the only
Context that fully satisfies all four rules.

### 4.2 Theme Editor (`ThemeEditor`) -- VIOLATES (the reported bug)

Evidence: `crates/ff-desktop/src/theme_editor_panel.rs`,
`crates/ff-desktop/src/shell/commands_theme.rs`,
`crates/ff-desktop/src/shell/render_theme.rs`,
`crates/ff-theme-editor/src/lib.rs`.

- Rule 1 (working copy): PARTIAL/OK. `open_theme_editor` -> `load_working`
  takes a working palette. BUT `A::EditToken` in `apply_theme_editor_action`
  does `self.palette = preview;` -- it mutates the RUNNING palette as a live
  preview on every colour edit. The convention allows a preview, but here the
  preview is the actual active palette, and nothing reverts it on RETURN (see
  rule 4). Blurred boundary -- the exact B081 problem.
- Rule 2 (SAVE commits + stays): VIOLATES. The typed `SAVE` command is a no-op
  (ThemeEditor not in `editable_context_kind`). The ONLY working persist is the
  in-panel "Save"/"Save As" button, which produces `ThemeEditorAction::Save` ->
  `apply_theme_editor_action` `A::Save` -> `write_theme_file`. There is no
  `THEME SAVE` typed command (`commands.rs` THEME arm only handles bare THEME,
  `THEME <name>` apply, `THEME EXPORT`, `THEME IMPORT`).
- Rule 3 (END commits then exits): VIOLATES. END on ThemeEditor is plain
  `nav_end()` -- no persist. Editing colours then pressing F3/END writes NOTHING
  to the theme file.
- Rule 4 (RETURN discards then exits): VIOLATES. RETURN calls `editable_discard`
  (no-op for ThemeEditor) then `nav_return()`. The live-preview mutation of
  `self.palette` is NOT reverted to the persisted theme, so the discard is not
  clean.

Net: in the Theme workspace, SAVE/END do not persist the theme file at all; only
the in-panel button does. This is both report symptoms (a) and (b).

### 4.3 Config / Settings panel (`ConfigPanel`) -- VIOLATES (autosave)

Evidence: `crates/ff-desktop/src/config_panel/widgets.rs` +
`crates/ff-desktop/src/config_panel/commit.rs`.

- Rule 1 (working copy): VIOLATES. There is NO working copy. Every widget change
  calls `commit_value(...)` -> `config.set_user_value(key, value)` immediately
  (checkbox `resp.changed()`, slider `resp.changed()`, text field
  `resp.lost_focus()`, combo selection change). The Reset button calls
  `config.remove_user_value(key)` immediately (`config_panel/render.rs`
  `render_entry`). This is textbook autosave-on-edit, explicitly forbidden.
- Rule 2 (SAVE): VIOLATES/moot. Typed SAVE is a no-op; there is no Save button.
- Rule 3 (END): END is plain navigation; by the time END runs, edits are already
  persisted.
- Rule 4 (RETURN): VIOLATES. RETURN cannot discard -- edits are already written
  to the user config layer; there is nothing to revert.

Config is the most divergent Context: it is the "modern autosave" model the owner
explicitly rejected in favour of ISPF explicit commit.

### 4.4 Menus Editor (`MenusEditor`) -- PARTIAL

Evidence: `crates/ff-desktop/src/shell/menus_editor.rs`.

- Rule 1: OK. `open_menus_editor` -> `load_working` takes a working `MenuFile`;
  per-option and structural edits mutate the working copy only.
- Rule 2: VIOLATES (command). Persist works ONLY via the in-panel Save/Save As
  button (`A::Save` -> `save_working_menu` -> validate -> `write_menu_file`).
  Typed `SAVE` command is a no-op.
- Rule 3: VIOLATES. END is plain `nav_end()` -- no save-then-exit.
- Rule 4: VIOLATES. RETURN does not discard via the seam (no-op); it navigates.
  (Because the working copy is separate and not persisted, the net data effect of
  discard-on-RETURN is achieved by abandoning the working copy, but it is not the
  convention's explicit discard path and END does not save.)

### 4.5 Keys Editor (`KeysEditor`) -- PARTIAL

Evidence: `crates/ff-desktop/src/shell/keys_editor.rs`.

- Rule 1: OK. `open_keys_editor` -> `load_kind` builds working rows from the
  effective map.
- Rule 2: VIOLATES (command). Persist works ONLY via the in-panel Save button
  (`A::Save` -> `save_keys_for_kind` -> write `keymaps/<kind>.toml` + reload
  resolver). Typed `SAVE` is a no-op.
- Rule 3: VIOLATES. END is plain navigation (no save).
- Rule 4: VIOLATES. RETURN does not discard via the seam.
- Note: `A::Reset` reloads the compiled default into the working copy (in-memory
  only until a subsequent button Save), which is consistent with a working-copy
  model.

### 4.6 Kinds Editor (`KindsEditor`) -- PARTIAL

Evidence: `crates/ff-desktop/src/shell/kinds_editor.rs`.

- Rule 1: OK. `open_kinds_editor` / `A::SelectKind` / `A::NewKind` ->
  `load_kind` builds a working `KindConfig`.
- Rule 2: VIOLATES (command). Persist works ONLY via the in-panel Save button
  (`A::Save` -> `save_selected_kind` -> `persist_kind_config` write+reload).
  Typed `SAVE` is a no-op.
- Rule 3: VIOLATES. END is plain navigation (no save).
- Rule 4: VIOLATES. RETURN does not discard via the seam.

### 4.7 Command Configurator (`CommandConfigurator`) -- VIOLATES/PARTIAL

Evidence: `crates/ff-desktop/src/shell/configurator.rs`.

This is the command-DEFINITION table, distinct from the Alias Editor.

- Rule 1: PARTIAL. Editing uses a per-entry edit `form` (a working copy of ONE
  definition), but there is no working copy of the whole store, and Delete acts
  on the live store.
- Rule 2: VIOLATES (command). Add/edit persist via the form Save button
  (`ConfiguratorAction::Save` -> `commit_configurator_form` ->
  `command_store.save()`). Typed `SAVE` command is a no-op.
- Delete is an IMMEDIATE write: `ConfiguratorAction::Delete(id)` ->
  `command_store.remove` + `command_store.save()` with no commit gate -- an
  autosave-shaped mutation.
- Rule 3/4: VIOLATES. END/RETURN are plain navigation; no store-level
  commit/discard semantics.

---

## 5. Prioritized non-compliance list (input to a later fix plan)

Priority 1 -- functional "save does nothing" bugs (user loses work or is
confused):

1. Theme Editor (`ThemeEditor`) -- the reported bug. Typed SAVE no-op; END does
   NOT persist; live preview mutates the active palette and is not reverted on
   RETURN. Wire `KindTag::ThemeEditor` into `editable_context_kind` with a
   persist that calls the existing `A::Save` write path, a discard that reverts
   the preview palette to the persisted theme, and a reload that re-opens the
   working copy. Address the live-preview-as-persistence blur per B081.

2. Menus Editor, Keys Editor, Kinds Editor -- each has a correct working copy and
   a correct button-Save, but the ISPF command verbs (SAVE/END/RETURN, F3) do
   NOT reach them. Wire each `KindTag` into `editable_context_kind` +
   `editable_persist`/`discard`/`reload` using its existing
   `save_working_menu` / `save_keys_for_kind` / `save_selected_kind` and a
   reload that re-opens the working copy. These are the "SAVE only happens on
   END/exit" family -- except today SAVE and END BOTH fail to persist; only the
   button works.

Priority 2 -- autosave model that contradicts the convention:

3. Config / Settings panel (`ConfigPanel`) -- the biggest conceptual gap: pure
   autosave-on-edit with no working copy and no Save affordance. Needs a working
   copy taken on entry, edits buffered, and SAVE/END to commit, RETURN to
   discard. This is the largest change (the commit path writes straight to
   `ConfigHandle`; a working-copy layer must be introduced).

4. Command Configurator (`CommandConfigurator`) -- Delete writes immediately and
   there is no store-level commit/discard. Decide whether it adopts the
   working-copy model for the whole store or is explicitly documented as a
   per-row immediate-commit Context (an allowed exception would need owner
   sign-off, since the convention says "every editable Context").

Already compliant (reference): Alias Editor (`AliasEditor`).

No action (NO-SAVE-NEEDED): Help, Plugin Manager, Event Log, Scrm viewer, Search
Results, Macro Library, Home/POM + named menus, and the three parked Contexts
(Catalog Explorer, Editor, File Explorer).

---

## 6. Recommendations (do not implement here)

- The fix is structurally clean because the seam already exists. The core work
  is extending `editable_context_kind()` + `editable_persist/discard/reload`
  (`crates/ff-desktop/src/shell/editable_context.rs`) with one arm per editable
  Context, delegating persist to each panel's existing save function and adding a
  discard+reload that re-opens the working copy. This is exactly the documented
  CR-CH-071 "fast-follow" (command-environments tasks 34.1-34.5) that was never
  completed.
- Theme Editor needs the extra step of reverting the live-preview palette on
  discard/RETURN so the preview stops doubling as persistence (B081 intent).
- Config/Settings needs the most design: introduce a working-copy buffer in
  `ConfigPanelState` and change `commit_value` to write the buffer, with
  SAVE/END flushing the buffer to `ConfigHandle` and RETURN dropping it. This
  deserves its own requirement entry.
- Command Configurator: confirm with the owner whether it should become a
  working-copy editor or be documented as an allowed immediate-commit exception.
- Because this spans many Contexts and changes observable save behaviour, the
  owner's suggestion to capture the convention as a cross-cutting project
  requirement is well founded; the steering file `save-convention.md` already
  states the rule, and a matching requirement with per-Context acceptance
  criteria would anchor the fix and its tests.

### Files cited
- `.kiro/steering/save-convention.md`
- `crates/ff-desktop/src/shell/render_body.rs` (central-panel match)
- `crates/ff-desktop/src/shell/render.rs` (framework dispatch/focus)
- `crates/ff-desktop/src/shell/workspace_context.rs` (trait + InteriorFocus)
- `crates/ff-desktop/src/shell/commands_ladder_a.rs` (SAVE/END/RETURN/CANCEL/REFRESH seam)
- `crates/ff-desktop/src/shell/editable_context.rs` (editable_context_kind + persist/discard/reload)
- `crates/ff-desktop/src/theme_editor_panel.rs`, `shell/commands_theme.rs`, `shell/render_theme.rs`, `crates/ff-theme-editor/src/lib.rs`
- `crates/ff-desktop/src/config_panel/render.rs`, `config_panel/widgets.rs`, `config_panel/commit.rs`
- `crates/ff-desktop/src/shell/menus_editor.rs`, `shell/keys_editor.rs`, `shell/kinds_editor.rs`
- `crates/ff-desktop/src/shell/alias_editor.rs`, `shell/alias_editor_line_commands.rs`
- `crates/ff-desktop/src/shell/configurator.rs`
- `crates/ff-desktop/src/command_config/mod.rs` (FFCMD roots incl. SAVE/CANCEL/REFRESH/END/RETURN)

Content was rephrased for compliance where steering text was summarised.
