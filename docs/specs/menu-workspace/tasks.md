# Tasks -- Menu Workspace Pattern

## Phase CU -- Specification (complete)

- [x] CU.1 Create `docs/specs/menu-workspace/requirements.md` (Reqs 1-5)
- [x] CU.2 Create `docs/specs/menu-workspace/design.md`
- [x] CU.3 Create `docs/specs/menu-workspace/tasks.md`
- [x] CU.4 Update `docs/specs/startup-and-session/requirements.md` Req 14.3 note
- [x] CU.5 Update `docs/quality/TCR.md` with CR-NR-045 NOT COVERED rows
- [x] CU.6 Add `menu-workspace` to `.amazonq/rules/specs.md` sub-project list

---

## Phase CU-impl -- Implementation (pending explicit instruction)

### Task 1: Scaffold MenuWorkspaceState and TabKind variant

- [x] 1.1 Add data-free `TabKind::MenuWorkspace` to `tab_state.rs` with the
  `MenuWorkspaceState` held in a separate `menu_workspace: Option<..>` field on
  `TabState` (TabKind is `Copy`; it does not carry the state inline)
  - Satisfies: Req 1 (data model), Req 2.1 (rendering contract)
  - CORRECTION (Phase DB): the earlier wording
    `TabKind::MenuWorkspace(MenuWorkspaceState)` was inaccurate -- the
    implemented variant is data-free. Design section 11 corrected to match.
- [ ] 1.2 Persist a Menu_Workspace across sessions
  - Satisfies: Req 4.6 (session persistence)
  - CORRECTION (Phase DB, CR-CH-012): the previously-checked plan to add
    `PersistedTabKind::MenuWorkspace { file_path }` was NEVER implemented (the
    enum has no such variant) and is superseded. A Menu_Workspace now persists
    as a `MenuWorkspace { name }` Workspace_Descriptor (startup-and-session
    Requirement 21.4), delivered in Phase DB (DB.11). Reopened as `[ ]`.
- [ ] 1.3 Write unit tests for Menu_Workspace persistence round-trip
  - Validates: Requirement 4.6, startup-and-session Requirement 21.4
  - CORRECTION (Phase DB): a `menu_workspace_persisted_tab_kind_round_trips`
    test for the never-added variant is not applicable; the descriptor
    round-trip test is written in Phase DB (DB.11). Reopened as `[ ]`.

### Task 2: TOML Loader

- [x] 2.1 Create `crates/ff-desktop/src/menu_workspace/mod.rs` (re-exports only)
  - Satisfies: Req 1 (file format)
- [x] 2.2 Create `crates/ff-desktop/src/menu_workspace/loader.rs` --
  `load_menu_file(path) -> Result<MenuFile, String>`
  - Satisfies: Req 1.1, 1.2, 1.3, 1.4
- [x] 2.3 Write unit tests: `load_valid_menu_file`, `load_missing_file_returns_error`,
  `load_invalid_toml_returns_error`, `load_missing_required_field_returns_error`,
  `load_unknown_key_is_ignored`, `load_key_normalised_to_uppercase`
  - Validates: Requirement 1.1-1.7

### Task 3: Hot-Reload

- [x] 3.1 Create `crates/ff-desktop/src/menu_workspace/hot_reload.rs` --
  `poll_reload()` method on `MenuWorkspaceState`
  - Satisfies: Req 4.3, 4.4, 4.5
- [x] 3.2 Wire `poll_reload()` into the egui `update()` loop for active
  Menu_Workspace tabs
  - Satisfies: Req 4.3
- [x] 3.3 Write unit tests: `poll_reload_detects_file_change`,
  `poll_reload_on_parse_error_retains_previous_menu`
  - Validates: Requirement 4.3, 4.5

### Task 4: Rendering

- [x] 4.1 Create `crates/ff-desktop/src/menu_workspace/render.rs` --
  `render_menu_workspace(state, ui, shell_state)`
  - Satisfies: Req 2.1, 2.2, 2.3, 2.4, 2.5, 2.6
- [x] 4.2 Wire `render_menu_workspace` into `shell/render.rs` for
  `TabKind::MenuWorkspace`
  - Satisfies: Req 2.1
- [x] 4.3 Write unit tests: `render_empty_options_shows_placeholder`,
  `render_disabled_option_not_selectable`
  - Validates: Requirement 2.3, 2.6

### Task 5: Command Dispatch

- [x] 5.1 Create `crates/ff-desktop/src/menu_workspace/commands.rs` --
  `execute_option(option, shell)`
  - Satisfies: Req 3.1, 3.2, 3.3, 3.4, 3.5, 3.7
- [x] 5.2 Extend `shell/commands.rs` command field handler to look up typed key
  in active Menu_Workspace option list before falling through to standard pipeline
  - Satisfies: Req 3.1, 3.6
- [x] 5.3 Write unit tests: `option_key_lookup_case_insensitive`,
  `unknown_key_shows_not_found_message`, `disabled_option_shows_not_available_message`
  - Validates: Requirement 3.1, 3.6, 3.7

### Task 6: Chained Path Resolver

- [x] 6.1 Create `resolve_chained_path(path, menus) -> Result<String, String>`
  pure function in `menu_workspace/commands.rs`
  - Satisfies: Req 5.1, 5.2, 5.3
- [x] 6.2 Extend fastpath handler in `shell/commands.rs` to call
  `resolve_chained_path` for paths containing `.`
  - Satisfies: Req 5.1, 5.4
- [x] 6.3 Write unit tests: `chained_path_two_levels`, `chained_path_max_depth`,
  `chained_path_unknown_segment_stops_at_last_resolved`,
  `single_segment_fastpath_unchanged`
  - Validates: Requirement 5.1-5.4

### Task 7: Default Menu Files

- [x] 7.1 Create `crates/ff-desktop/src/menu_workspace/defaults.rs` with
  `DEFAULT_POM_TOML` and `DEFAULT_SETTINGS_TOML` const strings
  - Satisfies: Req 4.1, 4.2
- [x] 7.2 Create `ensure_default_menu_files(user_data_dir)` in
  `shell/update.rs`; wire into startup block
  - Satisfies: Req 4.1, 4.2, 4.6
- [x] 7.3 Write unit tests: `ensure_default_menu_files_creates_menus_dir`,
  `ensure_default_menu_files_does_not_overwrite_existing`
  - Validates: Requirement 4.1, 4.2, 4.6

### Task 8: TCR and Documentation Update

- [x] 8.1 Update `docs/quality/TCR.md` -- set all CR-NR-045 rows to correct status
- [x] 8.2 Update `docs/specs/project-master/tasks.md` -- mark Phase CU-impl tasks complete

- [x] CV.1 Review current POM options (0-8) against all implemented functionality
- [x] CV.2 Define revised 12-option list (0-8 unchanged, add 9/S/B for Jobs/Search/Batch)
- [x] CV.3 Create `docs/specs/menu-workspace/cv-requirements.md` (Reqs 6-8)
- [x] CV.4 Update `docs/specs/startup-and-session/requirements.md` Req 14.3 with revised list
- [x] CV.5 Update `docs/quality/TCR.md` with CR-NR-048 NOT COVERED rows

---

## Phase CV-impl -- POM Redesign Implementation (pending explicit instruction)

### Task 9: DEFAULT_POM_TOML constant

- [x] 9.1 Add `DEFAULT_POM_TOML` const string to `menu_workspace/defaults.rs` with all 12
  options, groups, and TOML structure matching cv-requirements.md Req 7.1
  - Satisfies: Req 7.1, 7.4, 7.5
- [x] 9.2 Update `ensure_default_menu_files()` to write `pom.toml` when absent
  - Satisfies: Req 7.2, 7.3
- [x] 9.3 Write unit tests: `default_pom_toml_is_valid_toml`,
  `default_pom_toml_has_12_options`, `default_pom_toml_ascii_only`
  - Validates: Requirement 7.1, 7.4, 7.5

### Task 10: POM command routing for options 9, S, B

- [x] 10.1 Extend `shell/commands.rs` POM option handler to route key `9` to JES panel
  - Satisfies: Req 6.2
- [x] 10.2 Extend handler to route key `S` (case-insensitive) to Global Search panel
  - Satisfies: Req 6.3
- [x] 10.3 Extend handler to route key `B` (case-insensitive) to Batch status message
  - Satisfies: Req 6.4
- [x] 10.4 Write unit tests: `pom_key_9_routes_to_jes`, `pom_key_s_routes_to_search`,
  `pom_key_b_shows_batch_message`, `pom_keys_0_to_8_unchanged`
  - Validates: Requirement 6.2, 6.3, 6.4, 6.5

### Task 11: BUILT_IN_OPTIONS update

- [x] 11.1 Add options 9, S, B to `BUILT_IN_OPTIONS` in `primary_option_menu.rs`
  - Satisfies: Req 6.1
- [x] 11.2 Update `built_in_options_contains_all_required_entries` test to assert 12 entries
  - Satisfies: Req 6.1
- [x] 11.3 Update `pom_navigate_action_returned_for_each_option` test for new keys
  - Satisfies: Req 6.1

### Task 12: TCR and Documentation Update (Phase CV-impl)

- [x] 12.1 Update `docs/quality/TCR.md` -- set all CR-NR-048 rows to correct status
- [x] 12.2 Update `docs/specs/project-master/tasks.md` -- mark Phase CV-impl tasks complete

---

### Task 8: TCR and Documentation Update

- [ ] 8.1 Update `docs/quality/TCR.md` -- set all CR-NR-045 rows to correct
  status after implementation
  - Satisfies: project standards
- [ ] 8.2 Update `docs/specs/project-master/tasks.md` -- mark Phase CU-impl
  tasks complete
  - Satisfies: project standards

---

## Phase CW -- Settings as a Menu Workspace Spec (complete)

- [x] CW.1 Create `docs/specs/menu-workspace/cw-requirements.md` (Reqs 9-12)
- [x] CW.2 Define 10-option Settings_Menu (E/T/C/V/L/K/S/P/X/A) with groups
- [x] CW.3 Update `docs/specs/configuration-system/requirements.md` Req 15 with Phase CW note
- [x] CW.4 Update `docs/quality/TCR.md` with CR-NR-047 NOT COVERED rows

---

## Phase CW-impl -- Settings Menu Implementation (pending explicit instruction)

### Task 13: DEFAULT_SETTINGS_TOML constant

- [x] 13.1 Add `DEFAULT_SETTINGS_TOML` const string to `menu_workspace/defaults.rs`
  with all 10 options, groups, and TOML structure matching cw-requirements.md Req 11.1
  - Satisfies: Req 11.1, 11.4, 11.5
- [x] 13.2 Update `ensure_default_menu_files()` to write `settings.toml` when absent
  - Satisfies: Req 11.2, 11.3
- [x] 13.3 Write unit tests: `default_settings_toml_is_valid_toml`,
  `default_settings_toml_has_10_options`, `default_settings_toml_ascii_only`
  - Validates: Requirement 11.1, 11.4, 11.5

### Task 14: Settings Namespace View routing

- [x] 14.1 Add `namespace_filter: Option<String>` field to `SettingsPanelState`
  - Satisfies: Req 10.1, 10.2
- [x] 14.2 Extend `shell/commands.rs` Settings routing to accept namespace argument
  (e.g. `SETTINGS editor` pre-populates filter with `editor.`)
  - Satisfies: Req 10.1, 10.2
- [x] 14.3 Update tab title to `[SETTINGS:<namespace>]` when filter is set
  - Satisfies: Req 10.5
- [x] 14.4 Update F3/END handler: return to Settings_Menu when in namespace view,
  return to POM when in Settings_Menu
  - Satisfies: Req 10.4, Req 12.2 (criterion 15.10)
- [x] 14.5 Persist `namespace_filter` in session and restore on launch
  - Satisfies: Req 10.6
  - RESOLVED BY Phase DB (CR-CH-012, DB.11): the namespace filter persists as a
    `CustomWorkspace { workspace_kind = settings, params = { namespace } }`
    descriptor (startup-and-session Requirement 21.3), not a bespoke
    `PersistedTabKind` variant. Delivered by the descriptor-based persistence
    work: save threads `settings_panel.namespace_filter` into the descriptor;
    `restore_workspace_descriptors` re-opens Settings with the filter applied.
    Tests: `restore_settings_descriptor_applies_namespace_filter`,
    `settings_namespace_descriptor_round_trips`.
- [x] 14.6 Write unit tests: `settings_namespace_filter_applied_on_open`,
  `settings_namespace_tab_title_includes_namespace`,
  `settings_end_from_namespace_view_returns_to_menu`
  - Validates: Requirement 10.1, 10.4, 10.5

### Task 15: TCR and Documentation Update (Phase CW-impl)

- [x] 15.1 Update `docs/quality/TCR.md` -- set all CR-NR-047 rows to correct status
- [x] 15.2 Update `docs/specs/project-master/tasks.md` -- mark Phase CW-impl tasks complete

---

## Phase DA -- Configurable Menu Option Limits Spec (complete)

- [x] DA.1 Add Requirement 9 (Configurable Menu Option Limits) to `docs/specs/menu-workspace/requirements.md`
- [x] DA.2 Add design section 12A to `docs/specs/menu-workspace/design.md`
- [x] DA.3 Add Phase DA-impl tasks to this file
- [x] DA.4 Add Phase DA to `docs/specs/project-master/tasks.md`
- [x] DA.5 Add CR-NR-050 NOT COVERED rows to `docs/quality/TCR.md`

---

## Phase DA-impl -- Configurable Menu Option Limits Implementation (pending explicit instruction)

### Task 16: Configuration keys for option limits

- [x] 16.1 Register `menu.soft_option_limit` (u32, default 64) and
  `menu.hard_option_limit` (u32, default 256) in the `ff-config` schema
  - Satisfies: Req 9.1, 9.6
  - Note: keys added to `ff-config/keys.rs` `menu` module; registered in
    `register_builtin_schema` (ff-desktop `main.rs`) as Integer with min 0.
- [x] 16.2 Write unit tests: `menu_limit_keys_have_correct_defaults`
  (in `main.rs`), `menu_keys_are_valid_dot_separated_paths` (in `keys.rs`)
  - Validates: Requirement 9.1, 9.6

### Task 17: Loader limit evaluation

- [x] 17.1 Add `OptionLimits` and `LoadedMenu` types and
  `load_menu_file_with_limits(path, limits)` in `menu_workspace/loader.rs`;
  `load_menu_file(path)` stays pure, and `option_limits_from_config` reads the
  keys for the migration-phase construction path
  - Satisfies: Req 9.2, 9.3, 9.4, 9.5, 9.8
- [x] 17.2 Write unit tests: `count_at_or_below_soft_limit_no_advisory`,
  `count_above_soft_below_hard_sets_advisory`,
  `count_above_hard_returns_load_error`,
  `hard_below_soft_clamps_effective_soft_to_hard`,
  `disabled_options_count_toward_limits`,
  `option_limits_from_config_falls_back_to_defaults`
  - Validates: Requirement 9.2, 9.3, 9.4, 9.5, 9.7, 9.8

### Task 18: State and render advisory

- [x] 18.1 Add `advisory: Option<String>` and `limits: OptionLimits` to
  `MenuWorkspaceState`; set advisory in the shared `apply_load_result` path
  - Satisfies: Req 9.3
- [x] 18.2 Render the advisory line above the option list in
  `menu_workspace/render.rs`
  - Satisfies: Req 9.3
- [x] 18.3 Write unit tests: `advisory_line_rendered_when_soft_exceeded`,
  `no_advisory_line_when_within_soft_limit`
  - Validates: Requirement 9.3

### Task 19: Hot-reload re-evaluation

- [x] 19.1 `poll_reload()` calls the shared `apply_load_result` (which uses
  `load_menu_file_with_limits`) and sets `menu`/`load_error`/`advisory` on each
  reload; `load_with_limits` seeds the limits at construction
  - Satisfies: Req 9.9
- [x] 19.2 Write unit tests: `reload_over_hard_limit_transitions_to_error`,
  `reload_back_under_limit_recovers`, `reload_into_advisory_range_sets_advisory`
  - Validates: Requirement 9.9

### Task 20: TCR and Documentation Update (Phase DA-impl)

- [x] 20.1 Update `docs/quality/TCR.md` -- set all CR-NR-050 rows to PASS
- [x] 20.2 Update `docs/specs/project-master/tasks.md` -- mark Phase DA-impl tasks complete

---

## Phase DF -- MENU Argument Chaining (CR-NR-054, Requirement 5.5-5.6, 11.7-11.10)

- [ ] DF.1 Extend `open_menu_by_name` to accept an optional trailing option key; after loading the menu, activate the option whose key equals it (case-insensitive, key match only) via the existing option-dispatch path (Target_Resolution + inline `[options.target]` apply)
  - Covers: Requirement 11.7, 11.10
- [ ] DF.2 Unknown chained key: open the menu and report `Option '<key>' not found.` (reuse the existing not-found message)
  - Covers: Requirement 11.9
- [ ] DF.3 Share one navigation-and-activation helper between the Chained_Path resolver (`=k1.k2`, Req 5) and the chained-MENU path so the notations cannot diverge
  - Covers: Requirement 5.5, 11.8
- [ ] DF.4 Option `command` that is itself a chained `MENU`/Chained_Path forwards its argument on selection (via the general command-argument mechanism)
  - Covers: Requirement 5.6, 11.10
- [ ] DF.5 Unit tests: `MENU <name> <key>` activates the option; unknown key -> not-found + menu open; `=0.E` equivalence; deeper chain forwards remainder
  - Validates: Requirement 5.5, 5.6, 11.7, 11.8, 11.9, 11.10

---

## Phase DG -- Chained Path Separator Semantics (CR-NR-057, Requirement 5.7-5.12)

- [ ] 21. Separator-aware chained path resolution
  - [ ] 21.1 Extend `resolve_chained_path` to return an ordered `Vec<PathStep>` (each with the separator that preceded it), reusing command-semantics `split_chain` so the fastpath and command-line notations cannot diverge
    - Covers: Requirement 5.11, 5.12
  - [ ] 21.2 Set the Navigation_Origin to the POM for a leading `=`, and to the current Workspace for a non-`=` navigation command
    - Covers: Requirement 5.7, 5.10
  - [ ] 21.3 Drive each PathStep through the option-dispatch path, pushing the intermediate Context for a PUSH (`;`) step and collapsing it for a STOP (`.`) step (command-framework Requirement 10.5, 10.6)
    - Covers: Requirement 5.8, 5.9, 5.11
  - [ ] 21.4 Write failing unit tests: `=0.E` -> END returns to POM; `=0;E` -> END returns to Settings then POM; `EDITOR` (no `=`) -> END returns to current Workspace; `SETTINGS ; EDITOR` two-hop stack; mixed `=0;E.T`
    - Validates: Requirement 5.7, 5.8, 5.9, 5.10, 5.11
  - [ ] 21.5 Update `docs/quality/TCR.md`: set the CR-NR-057 menu-workspace Req 5.7-5.12 rows to their correct status
    - Covers: Requirement 5.7-5.12

- [ ] 22. Unified config-driven menu renderer -- POM + Settings + all menus (Requirement 2.1a-2.1c, 1.8; CR-CH-018)
  - [x] 22.1 Add `show_calendar` (bool, default true) to the Menu_File format + loader (`RawMenuFile`, `MenuFile`); unknown-key/ASCII rules unchanged
    - Covers: Requirement 1.8
  - [x] 22.2 Fold the POM column+calendar layout into `menu_workspace/render.rs`: render a three-column option list (key | command | description) replacing the single-line row; retain group separators, disabled style, scroll, empty-state
    - Covers: Requirement 2.1a
  - [x] 22.3 Render the live calendar panel on the right when `show_calendar` is true, reusing the calendar/date helpers (keep them as pure fns); return `CalendarNav` so month `<`/`>` works; full-width option columns when false
    - Covers: Requirement 2.1b, 1.8
  - [x] 22.4 Render the POM (Home Context) via the shared `render_menu_workspace` against a `pom.toml`-backed MenuWorkspaceState; preserve POM option dispatch + focus/keyboard behaviour; POM options become data-driven from `menus/pom.toml`
    - Covers: Requirement 2.1c
  - [x] 22.4a Attach a `MenuWorkspaceState` (loaded from `menus/pom.toml`) to the POM tab while KEEPING `TabKind::PrimaryOptionMenu` (lazy `ensure_pom_menu_loaded`, default-content fallback); tab title stays `[POM]` and Title_Line stays POM-styled
    - Covers: Requirement 2.1d
  - [x] 22.4b Call the shared `render_menu_workspace` from the `TabKind::PrimaryOptionMenu` render arm against the POM's MenuWorkspaceState; a clicked option flows into `pending_menu_option`; removed the `primary_option_menu::render` call
    - Covers: Requirement 2.1c, 2.1d
  - [x] 22.4c Made selection config-driven (Req 2.1e): dispatch ONLY the selected option's `command`; DELETED the digit-keyed `handle_command` arms; `=<key>`/bare key resolve via `resolve_pom_option_key` to the option's command
    - Covers: Requirement 2.1e, 2.1i
  - [x] 22.4d Ported the keyboard focus ring + Enter/Space activation to read the loaded POM option list (`pom_option_count` param; activation dispatches the option's command via `pending_menu_option`); degrades when the POM menu is absent
    - Covers: Requirement 2.1f
  - [x] 22.4e Terminate as a data-driven `menus/pom.toml` option (key `X`, command `RETURN`); removed the bespoke `EXIT_LINE_TEXT`/`PomAction::Exit`/`FocusStop::PomExit`
    - Covers: Requirement 2.1g
  - [x] 22.4f Every default `pom.toml` command resolves by NAME (added `CATALOGS` arm); trimmed `DEFAULT_POM_TOML` to built+testable options (0 SETTINGS, 1 CATALOGS, 2 FILES, 5 MACROS, 8 PLUGINS, S SEARCH, X RETURN); updated defaults tests
    - Covers: Requirement 2.1h
  - [x] 22.4g Retired `primary_option_menu::render`, `BUILT_IN_OPTIONS`, `PomAction`, `PomRenderResult`, `EXIT_LINE_TEXT`; kept the calendar/date helpers and `PomColours`
    - Covers: Requirement 2.1c
  - [x] 22.5 Wrote/migrated tests: POM routing config-driven (key->command); focus ring cycles loaded options; terminate `X`->RETURN per CR-CH-016; `=5`/`=8` fastpath; trimmed default pom.toml; retired-API tests removed; verify.ps1 CLEAN (nextest)
    - Covers: Requirement 2.1c-2.1i
  - [x] 22.6 Updated `docs/quality/TCR.md` rows for Req 2.1c-2.1i to PASS (1.8 / 2.1a / 2.1b already PASS)
    - Covers: Requirement 2.1c-2.1i

---

## Phase (menu-recovery) -- Code-only menus + Recovery Baseline + configurable group separator + RESET BARE (CR-CH-021, Req 4 revised, Req 12, configuration-system Req 19, startup Req 11.8/11.9)

- [ ] 23. Code-only menus, Recovery Baseline, configurable separator, RESET BARE
  - NOTE: 23.1-23.8, 23.10-23.12 complete; 23.9 (Settings button affordance)
    outstanding -- the RESET BARE command + dialog are implemented and
    dispatchable, only the dedicated Settings button is deferred.
  - [x] 23.1 Stop materialising built-in menus: remove the `write_if_absent(pom.toml)` / `write_if_absent(settings.toml)` writes; keep only a dir-ensure (`ensure_menus_dir`) called from `shell/update.rs`. Remove the `ensure_default_menu_files` materialise call at the startup site
    - Covers: Requirement 4.1, 4.2 (revised); startup-and-session 11.9
  - [x] 23.2 Recovery_Baseline content: update `DEFAULT_POM_TOML` to the barebones POM (0 Settings, 1 Catalogs, 2 Files, L Log, M Menus, X Return, single group) and `DEFAULT_SETTINGS_TOML` to (T Themes, M Menus, A All); add `recovery_pom_menu()`/`recovery_settings_menu()` builders that parse these (one source of content)
    - Covers: Requirement 12.1, 12.2, 12.3, 12.7
  - [x] 23.3 Settings fallback: in `open_settings_menu`, when the loaded state has no menu, fall back to `parse_menu_str(DEFAULT_SETTINGS_TOML)`; push a non-blocking notice ONLY when an on-disk `settings.toml` existed but failed to parse (silent when merely absent). POM fallback already exists; confirm it uses the new content
    - Covers: Requirement 12.4, 12.5; startup-and-session 11.8
  - [x] 23.4 Reserve the `MENUS` command: add a `handle_command` arm that shows `Menus editor is not yet available.` (non-blocking) and returns unchanged; add `LOG` row already resolves via existing `LOG`
    - Covers: Requirement 12.6
  - [x] 23.5 Configurable group separator: add `group_separator: GroupSeparator` (Line|Space|None, default Space) and `group_headers: bool` (default false) to `RawMenuFile`/`MenuFile`; replace the unconditional `ui.separator()` in `render_menu_workspace` with the style switch (space/line/none), boundary only between two different non-empty groups; optional header label when enabled
    - Covers: Requirement 2.4, 4a, 4b
  - [x] 23.6 RESET BARE command: `handle_command` arm for `RESET BARE` opens a modal confirmation dialog (no action until confirmed); Cancel = no-op
    - Covers: configuration-system Requirement 19.1, 19.2, 19.3
  - [x] 23.7 Archive helper: `archive_config(user_data_dir) -> Result<PathBuf, Vec<String>>` moves (rename, fallback copy+remove) menus/, themes/, session.toml, config.toml, catalog registry to `config-archive/<UTC-timestamp>/`; missing items skipped; per-item failures collected; never prune prior archives
    - Covers: configuration-system Requirement 19.4, 19.5, 19.8
  - [x] 23.8 On confirm: run archive, then reset in-memory config/menus/theme/catalog to compiled baselines, re-create the default Home catalog, and reopen the Recovery_Baseline POM (no relaunch)
    - Covers: configuration-system Requirement 19.6
  - [ ] 23.9 Settings affordance dispatches `RESET BARE` through the command path (parity, does not bypass the dialog)
    - Covers: configuration-system Requirement 19.7
    - NOTE: the `RESET BARE` command + confirmation dialog are implemented and
      dispatchable (typed command path). The Settings BUTTON affordance is not
      yet wired; it will be added with the Menus/Settings editor work. The
      command remains reachable by name and via any user menu option that maps
      to it, so parity is available; a dedicated button is the outstanding item.
  - [x] 23.10 Home catalog regression criterion: assert `ensure_default_home_catalog` still seeds "Home" -> user home when no Native catalog exists
    - Covers: startup-and-session Requirement 14 (regression guard for CR-NR-004)
  - [x] 23.11 Tests: no-materialise (menus/ empty after startup); absent user file -> Recovery_Baseline no notice; corrupt user file -> Recovery_Baseline + notice; Settings fallback; group_separator space/line/none rendering; RESET BARE confirm archives+resets and Cancel no-ops; MENUS notice; Home catalog seeded. verify.ps1 CLEAN
    - Validates: Requirement 4.1/4.2/12.1-12.7/2.4; configuration-system 19.1-19.8; startup 11.8/11.9
  - [x] 23.12 Update `docs/quality/TCR.md` rows for the new criteria; mark tasks done; commit+push
    - Covers: project standards
