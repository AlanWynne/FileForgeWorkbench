# Tasks -- Configurable Workspace Kinds (CR-NR-090)

Sliced delivery. Each slice is gated when reached (B.2-B.4 tasks are added at
their gates). Slice B.1 below.

## Slice B.1 -- Kind config model + registry + built-in defaults + title-from-config

- [x] 1. Kind config data model
  - [x] 1.1 Define `BaseKind` (`Builtin(BuiltinKind)` | `External(String)`), `BuiltinKind` (stable-name enum mirroring the compiled kinds), `KindProfile`, and `KindConfig` in a new `crates/ff-desktop/src/workspace_kind/` module
    - Covers: Requirement 1.1, 1.2
  - [x] 1.2 Serde/TOML round-trip for `KindConfig` via `KindConfigToml` with `modelled_on` as a string tag (`"editor"` / `"ext:<name>"`); unit tests round-trip incl. an External base
    - Covers: Requirement 1.6; design P3
  - [x] 1.3 `BuiltinKind` <-> stable name + `TabKind` mapping (from_tab_kind incl. POM/menu is_home split); unit tests mapping total + stable + unique
    - Covers: Requirement 1.4, 1.5
- [x] 2. Kind registry + compiled built-in defaults
  - [x] 2.1 `KindConfig::builtin_default(kind)` + `BuiltinKind::default_title` table: correct distinct titles (Catalogs `[CATALOGS]`, File Explorer `[FILES]`), `modelled_on = Builtin(self)`
    - Covers: Requirement 2.1, 2.4
  - [x] 2.2 `KindRegistry::with_builtin_defaults()` + `effective(name)` (total, safe fallback) + `resolve_base(name)` (single hop, allow(dead_code) until B.2); unit tests P1 + P2
    - Covers: Requirement 2.1, 1.4; design P1, P2
  - [x] 2.3 `KindRegistry::load(user_dir)`: read `workspace-kinds/*.toml`, user overrides built-in by name; skip unparseable with notice; External base unresolved -> POM fallback + notice; absent dir silent
    - Covers: Requirement 1.2, 1.3, 2.2, 2.3
  - [x] 2.4 Hold `kind_registry` on `WorkbenchShell`, loaded at startup from `<User_Data_Dir>/workspace-kinds/`; load notices logged at WARN
    - Covers: Requirement 2.1
- [x] 3. Title derived from the Kind config
  - [x] 3.1 Add `WorkbenchShell::kind_title(tab) -> String` returning the active Kind's effective title for panel Kinds (Home banner / non-Home menu label / editor path delegated to title_line_text)
    - Covers: Requirement 3.1
  - [x] 3.2 Route `render_title_line` + the tab-bar `base_title` + the detached Title_Line through `kind_title`; `title_line_text` free fn returns the compiled Kind default for panel Kinds (shell-less shim); `workspace_name` precedence + editor path unchanged
    - Covers: Requirement 3.1, 3.2
  - [x] 3.3 Corrected built-in titles (FilesPanel/Catalogs -> `[CATALOGS]`, distinct from File Explorer `[FILES]`); updated title_line_files_panel_shows_files + tab_manager files_panel title tests + files_panel constructor + Files reconstruct
    - Covers: Requirement 2.4; design P4
  - [x] 3.4 Full-shell test kind_title_derives_from_registry_and_user_override_wins (Catalogs [CATALOGS] vs File Explorer [FILES]; user override applies live)
    - Covers: Requirement 3.1, 3.2, 3.3
- [x] 4. Slice B.1 close
  - [x] 4.1 verify.ps1 CLEAN (FULL nextest); ffwb.exe rebuilt; TCR rows; change-log CR-NR-090 B.1 DONE; project-master WK.1; test-plan 2.4c
    - Covers: Slice B.1

## Slice B.2 -- per-Kind menu bar + key list

- [x] 5. Per-Kind menu bar
  - [x] 5.1 Added `resolve_menu_bar_menu_for(&self, tab) -> MenuFile`: bar name = active Kind's effective `menu_bar` (registry) else `DEFAULT_MENU_BAR_NAME`; reuses the existing slug + loader + compiled fallback. `resolve_menu_bar_menu()` is now a wrapper over `_for(active_tab)`
    - Covers: Requirement 4.1, 4.2
  - [x] 5.2 Detached menu bar uses its own Kind's bar (render_detached_menu_bar -> resolve_menu_bar_menu -> active tab, which is the detached tab inside the CR-CH-036 swap)
    - Covers: Requirement 4.2
  - [x] 5.3 Headless test menu_bar_uses_kind_menu_bar_else_default (configured bar resolves via a temp menus dir; default when unset)
    - Covers: Requirement 4.1, 4.4, 4.5
- [x] 6. Per-Kind key list
  - [x] 6.1 Added `key_list_context_for_tab(&self, tab) -> Option<String>`: the Kind's effective `key_list` if set, else `context_name_for_tab(tab)`
    - Covers: Requirement 4.3
  - [x] 6.2 Routed the tab-activation `set_context(...)` site (render_chrome) through `key_list_context_for_tab` (the only production set_context-by-tab site; navigate/START keep today's behaviour)
    - Covers: Requirement 4.3
  - [x] 6.3 Headless test key_list_context_uses_kind_key_list_else_base (configured key_list selects that context; unset -> base kind context; unknown -> global fallback via resolver precedence)
    - Covers: Requirement 4.3, 4.4, 4.5
- [x] 7. Slice B.2 close
  - [x] 7.1 verify.ps1 CLEAN (FULL nextest); ffwb.exe rebuilt; TCR rows; change-log B.2 DONE; project-master WK.2; test-plan 2.7a
    - Covers: Slice B.2

## Slice B.3 -- per-Kind profile attributes applied on open

- [x] 8. Apply the Kind profile on open
  - [x] 8.1 Added `line_end_from_name(&str) -> LineEndMode` mapper ("unicode" else default); unit test line_end_from_name_maps_unicode_else_default
    - Covers: Requirement 5.2
  - [x] 8.2 Added `WorkbenchShell::apply_kind_profile_to_active()`: edit_profile from the Kind (Untitled + FileEditor); line_end_mode from the profile ONLY for Untitled (loaded files keep detected); non-editor kinds no-op; applied once at open
    - Covers: Requirement 5.1, 5.2, 5.4
  - [x] 8.3 Added shell wrappers `shell_open_file(path)` + `shell_new_untitled()` (TabManager call + apply_kind_profile_to_active); routed all shell open sites (update.rs pending_open + cli_files + pending_new_file; render.rs search-open + toolchain-open; commands.rs session-restore Editor) through them
    - Covers: Requirement 5.1, 5.2
- [x] 9. Tests + behaviour-preserving
  - [x] 9.1 new_editor_takes_kind_edit_profile_on_open (CAPS On applied at open; a later per-tab toggle survives opening another tab; default Kind opens CAPS Off)
    - Covers: Requirement 5.1
  - [x] 9.2 new_buffer_takes_kind_line_end_mode (a Unicode Kind opens a new buffer Unicode) + line_end_from_name unit test
    - Covers: Requirement 5.2
  - [x] 9.3 built-in default profile opens with EditProfile::default() (asserted in new_editor_takes_kind_edit_profile_on_open)
    - Covers: Requirement 5.4, 5.5
- [x] 10. Slice B.3 close
  - [x] 10.1 verify.ps1 CLEAN (FULL nextest); ffwb.exe rebuilt; TCR rows; change-log B.3 DONE; project-master WK.3; test-plan 4.8b
    - Covers: Slice B.3

## Slice B.4 -- Kinds Editor Context + command + Settings + RESET BARE

### B.4a -- the Kinds Editor Context
- [x] 11. New Kinds Editor kind + built-in registration
  - [x] 11.1 Add `TabKind::KindsEditor` + `BuiltinKind::Kinds` (stable name "kinds", default title "[KINDS]"); wire from_tab_kind / context_name_for_kind / title_line_text / kind_title / nav_stack / session_manager arms + the compiled built-in default KindConfig
    - Covers: Requirement 6.6
  - [x] 11.2 Add `workspace_kinds_dir()` resolver (test override else <User_Data_Dir>/workspace-kinds/); used in the B.1 startup load and B.4 save
    - Covers: Requirement 6.3
- [x] 12. Kinds Editor panel (pure render + WorkspaceContext)
  - [x] 12.1 `crate::kinds_editor_panel`: `KindsEditorState` (selected name, working KindConfig, kind-name list, new-kind sub-form, interior ids, pending_action) + `KindsEditorAction`
    - Covers: Requirement 6.1, 6.2
  - [x] 12.2 Pure `render(ui, &mut state) -> KindsEditorAction`: selector + editable fields (title/menu_bar/key_list/modelled_on/profile toggles+tab_size+line_end_mode) + "New Kind modelled on <base>" + Save; `impl WorkspaceContext` returning InteriorFocus
    - Covers: Requirement 6.1, 6.2, 6.5
- [x] 13. Shell wiring + command + Settings
  - [x] 13.1 `shell/kinds_editor.rs`: `open_kinds_editor()` (navigate-in-place via Navigation_Stack) + `apply_kinds_editor_action()` (Save -> write workspace-kinds/<name>.toml + reload registry live; NewKind -> seed from base)
    - Covers: Requirement 6.3, 6.4
  - [x] 13.2 `KINDS` command arm -> open_kinds_editor (command parity); central-panel dispatch arm for TabKind::KindsEditor (owned-panel swap + drain pending_action)
    - Covers: Requirement 6.4, 6.5
  - [x] 13.3 Settings Recovery_Baseline option row (`W` -> KINDS) dispatching the KINDS command (menu == typed); Settings option-key tests updated
    - Covers: Requirement 6.4
- [x] 14. B.4a tests
  - [x] 14.1 Headless: open_kinds_editor_activates_kinds_editor_context; kinds_editor_save_writes_file_and_reloads_registry; kinds_editor_new_kind_seeds_copy_modelled_on_base
    - Covers: Requirement 6.1, 6.2, 6.3
  - [x] 14.2 Full-shell first-Tab egui_kittest test full_shell_kinds_first_tab_focuses_first_interior (workspace-conformance): the Kinds Editor reports its first interior control
    - Covers: Requirement 6.5

### B.4b -- RESET BARE integration
- [x] 15. RESET BARE
  - [x] 15.1 reset_in_memory_to_baseline also resets kind_registry to with_builtin_defaults() (drop user overrides); the profile's workspace-kinds/ dir added to ARCHIVED_ITEMS
    - Covers: Requirement 7.1, 7.2, 7.3
  - [x] 15.2 Headless: reset_bare_restores_builtin_kind_registry (after execute_reset_bare with active profile, a prior user override title is gone); archived_items_includes_workspace_kinds; archive_config_moves_workspace_kinds_dir
    - Covers: Requirement 7.1, 7.3
- [x] 16. Slice B.4 close
  - [x] 16.1 verify.ps1 CLEAN (FULL nextest); rebuild ffwb.exe; TCR rows; change-log B.4 DONE; project-master WK.4; test-plan rows
    - Covers: Slice B.4

- [x] 17. CR-CH-041: menu bar / key list resolved for the instance, rendered in-region (Requirement 4.6)
  - [x] 17.1 The B.2 menu-bar resolution keys on the instance's Kind (`resolve_menu_bar_menu_for`); the resolved bar renders at the instance's placement via the shared `render_menu_bar_into_ui` (in `render_region_menu_bar` for a split region), not as an app-level bar. No `KindConfig` schema change.
    - Covers: Requirement 4.6 (in-region rendering framing); layout-and-docking Requirement 16.2
  - [x] 17.2 Guarded the boundary: `kind_config_has_no_core_tab_container_field` structurally proves `KindConfig` has NO tab-container field; a Kind's internal composition is private (may reuse a `TabGroupTree` internally).
    - Covers: Requirement 4.6; layout-and-docking Requirement 16.7
  - [x] 17.3 Close: TCR Req 4.6 row PASS; verify.ps1 CLEAN FULL nextest; change-log CR-CH-041 DONE (shared with layout-and-docking task 22.8).
    - Covers: Requirement 4.6
