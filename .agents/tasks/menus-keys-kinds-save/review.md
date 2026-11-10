# Review: menus-keys-kinds-save (B081, project-master Requirement 11)

Branch: `menus-keys-kinds-save`
Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\menus-keys-kinds-save`
Commit reviewed: `ed75bf9` (single commit, `main..HEAD`)

Verdict: APPROVED

## Scope of change

Diffstat (11 files, +787/-37):

- `editable_context.rs` (+arms) -- the shared CR-CH-071 seam
- `commands_menus_save.rs`, `commands_keys_save.rs`, `commands_kinds_save.rs` (new) -- per-editor seam ops
- `menus_editor.rs`, `keys_editor.rs`, `kinds_editor.rs` -- save fns now return `Result`, `A::Save` delegates to persist
- `mod.rs` -- module declarations (3 seam + 3 test modules)
- `tests_menus_save_convention.rs`, `tests_keys_save_convention.rs`, `tests_kinds_save_convention.rs` (new) -- 6 tests each

## Required outcomes (per Context: Menus / Keys / Kinds)

1. SINGLE SEAM -- PASS. `editable_context_kind()` now returns `Some(EditableKind::{MenusEditor,KeysEditor,KindsEditor})` for the matching `KindTag`. SAVE/END/RETURN/REFRESH/CANCEL all route through the pre-existing single seam in `commands_ladder_a.rs` (`editable_context_kind` -> `editable_persist` / `editable_discard` / `editable_reload`). That ladder file is UNCHANGED by this diff; the new KindTag arms simply make the already-wired verbs reach each editor. No bespoke `if upper == ...` intercept, no second dispatcher.

2. PERSIST DELEGATES -- PASS. `menus_editor_persist` -> `save_working_menu`; `keys_editor_persist` -> `save_keys_for_kind`; `kinds_editor_persist` -> `save_selected_kind`. Each returns `Result<(), String>`; no write logic is duplicated in the seam ops (they only add a nil-selection guard before delegating).

3. DISCARD CLEAN -- PASS. Each `*_discard` re-opens the persisted copy (`load_menu_for_editor` / `load_key_map_for_kind` / `kind_registry.effective`) into the working copy, or clears it when nothing is selected, and clears the inline error. No partially-applied state; re-entry reloads cleanly.

4. RELOAD -- PASS. Each `*_reload` re-opens the working copy from the persisted store (delegates to `*_discard`; no live preview to revert for these three, correctly noted).

5. SHARED SAVE PATH -- PASS. The in-panel `A::Save` button and the typed SAVE command reach the same persist function. Menus/Keys route the button through `*_persist`; Kinds routes the button through `save_selected_kind` directly, which `kinds_editor_persist` also delegates to -- same single persist function either way (Req 11.8).

6. TESTS -- PASS. 6 tests per Context, each annotated `// Validates: project-master Requirement 11.x`, driving the real `dispatch_command_string` seam and asserting the backing-file effect:
   - 11.3 SAVE persists + stays open
   - 11.4 END persists + exits
   - 11.4 END-on-persist-error stays open (forced via invalid menu command / dir-as-file for keys & kinds)
   - 11.5 RETURN discards + exits, backing file never written
   - 11.2 bare edit does not autosave (file absent)
   - 11.8 in-panel Save button shares the SAVE command path

## MUST-CONFIRM checks

- AliasEditor / ThemeEditor BYTE-IDENTICAL -- PASS. The `editable_context.rs` diff is purely additive; Alias and Theme match arms and their ops are untouched.
- Config/Settings and Command Configurator NOT touched -- PASS. ConfigPanel is explicitly left as a documented TODO (autosaves, needs a working-copy buffer first); not in the diffstat. Command Configurator not in the diffstat.
- All six required behaviours tested for EACH of the three Contexts -- PASS (see above).
- No source file over 400 non-test lines -- PASS. `editable_context.rs` 145, `kinds_editor.rs` 221, the three seam files ~60 each; the editor files read well under 400. Consistent with the commit's recorded counts.
- ASCII-only in docs/specs touched -- N/A / PASS. No docs or specs are in the diff; only `.rs` source, which is ASCII (comment separators use `===`).
- Recorded verification logs -- PASS (read from the commit message, not re-run per instructions): `cargo check -p ff-desktop` clean; `cargo clippy -p ff-desktop --all-targets -- -D warnings` clean (after a documented, justified `#[allow(clippy::enum_variant_names)]`); `cargo fmt -p ff-desktop -- --check` clean; `cargo test -p ff-desktop save_convention` 24 passed / 0 failed (18 new + 6 pre-existing theme).

## Observations (non-blocking)

- `A::SaveAs(new_name)` in `menus_editor.rs` still calls `save_working_menu(&new_name)` directly rather than the persist seam. This is correct: SaveAs is a save-under-a-new-name operation, distinct from the persist-current-selection contract, and is out of scope for Req 11.8's "same persist path" requirement.
- `save_keys_for_kind` / `save_selected_kind` set `error = Some("Saved ...")` as a success status line; `*_discard` clears it. Pre-existing status-channel behaviour, not changed by this work.
- The project-master Req 11 per-Context conformance table is not present in this worktree's committed base (it lives in main's uncommitted edits, as the Theme commit also noted); no conformance row updated here. Owner to reconcile when that doc lands. Not a blocker for this code change.

## Conclusion

The change wires the three editors into the single ISPF explicit-save seam exactly as the merged Theme Editor fix did, with no new dispatch mechanism, no duplicated write logic, behaviour-preserving refactors of the existing save functions, and complete per-Context test coverage of all six required behaviours. No blockers.
