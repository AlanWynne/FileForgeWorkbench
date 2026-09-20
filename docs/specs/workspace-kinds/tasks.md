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

## Slice B.3 -- per-Kind profile attributes (PENDING GATE)
- [ ] (tasks added at the B.3 gate)

## Slice B.4 -- Kind config dialog + Settings entry + command + RESET BARE defaults (PENDING GATE)
- [ ] (tasks added at the B.4 gate)
