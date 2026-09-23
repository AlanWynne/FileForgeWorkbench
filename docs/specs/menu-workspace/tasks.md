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

- [x] 23. Code-only menus, Recovery Baseline, configurable separator, RESET BARE
  - NOTE: 23.1-23.8, 23.10-23.12 done in CR-CH-021; 23.9 (Settings RESET BARE
    affordance) closed by CR-NR-075 via the Settings baseline `R -> RESET BARE`
    row (command-parity dispatch).
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
  - [x] 23.9 Settings affordance dispatches `RESET BARE` through the command path (parity, does not bypass the dialog)
    - Covers: configuration-system Requirement 19.7
    - DONE (CR-NR-075): the Settings Recovery_Baseline now carries an `R -> RESET
      BARE` row (group "Recovery"). Selecting it dispatches the RESET BARE command
      through the menu-option = command path, opening the confirmation dialog
      without bypassing it. Tested by `settings_reset_bare_affordance_dispatches_command`
      and `default_settings_toml_has_recovery_baseline_options`.
  - [x] 23.10 Home catalog regression criterion: assert `ensure_default_home_catalog` still seeds "Home" -> user home when no Native catalog exists
    - Covers: startup-and-session Requirement 14 (regression guard for CR-NR-004)
  - [x] 23.11 Tests: no-materialise (menus/ empty after startup); absent user file -> Recovery_Baseline no notice; corrupt user file -> Recovery_Baseline + notice; Settings fallback; group_separator space/line/none rendering; RESET BARE confirm archives+resets and Cancel no-ops; MENUS notice; Home catalog seeded. verify.ps1 CLEAN
    - Validates: Requirement 4.1/4.2/12.1-12.7/2.4; configuration-system 19.1-19.8; startup 11.8/11.9
  - [x] 23.12 Update `docs/quality/TCR.md` rows for the new criteria; mark tasks done; commit+push
    - Covers: project standards

---

## Phase (menus-editor) -- Menus editor Context (CR-NR-075, Requirement 13; closes 23.9)

- [x] 24. Menus editor Context (create / edit / reorder / save menu TOMLs)
  - [x] 24.1 MenuFile -> TOML serialiser: new `menu_workspace/serialiser.rs` `serialise(&MenuFile) -> String` (title; show_calendar/group_separator/group_headers only when non-default; one `[[options]]` block per option with key/command/description/enabled(when false)/group(when Some)/inline target when present)
    - Covers: Requirement 13.8, 13.10
  - [x] 24.2 Round-trip test: `parse_menu_str(serialise(&m)) == m` for representative menus (POM baseline, Settings baseline, a multi-group menu, options with enabled=false and groups); confirm/handle CommandTarget serialisation
    - Covers: Requirement 13.10
  - [x] 24.3 Shared validation: factor loader per-option checks into `validate_menu(&MenuFile, limits) -> Result<(), String>`; the load path and the editor Save path both call it (editor cannot produce an unloadable file)
    - Covers: Requirement 13.7, 13.13
  - [x] 24.4 Panel: new `menus_editor_panel/{mod,state,render}.rs` with `MenusEditorState` (available/selected/working/name_buffer/error) and `MenusEditorAction`; pure `render(ui, state) -> Action` with the two-slot B052 rule (button click wins over field lost_focus)
    - Covers: Requirement 13.4, 13.5, 13.6, 13.12
  - [x] 24.5 Tab wiring: `TabKind::MenusEditor` + `TabState::menus_editor` (`[MENUS]`) + `tab_manager::open_menus_editor_tab`; add MenusEditor to the END/RETURN return-to-POM branch; render dispatch arm in `shell/render.rs`
    - Covers: Requirement 13.1
  - [x] 24.6 Shell impl `shell/menus_editor.rs`: `open_menus_editor` (available = POM/Settings + user files; built-in -> Recovery_Baseline when no file; transform-in-place on POM else dedicated tab), `apply_menus_editor_action` (edit/add/delete/move mutate working; Save/SaveAs validate + write), `write_menu_file` (menu_slug; POM/Settings -> pom.toml/settings.toml), `menus_dir_override` field for test isolation
    - Covers: Requirement 13.1, 13.2, 13.3, 13.8, 13.9
  - [x] 24.7 Replace the `upper == "MENUS"` notice arm with `open_menus_editor()`; both POM `M` and Settings `M` rows now open the editor
    - Covers: Requirement 13.1
  - [x] 24.8 Save writes a user override the renderer prefers over the Recovery_Baseline; deleting the file restores the baseline (built-in stays code-only); saved file hot-reloads into an open POM/Settings within the existing window
    - Covers: Requirement 13.8, 13.11
  - [x] 24.9 Settings RESET BARE affordance: a button/option that dispatches the `RESET BARE` command through the command path (opens the dialog; does not bypass it) -- CLOSES task 23.9
    - Covers: configuration-system Requirement 19.7
  - [x] 24.10 Tests: serialiser round-trip; validation rejects bad key/empty command/over-limit and Save is blocked; add/delete/move reorder working; Select loads built-in baseline when no file; Save writes menus/<name>.toml and it reloads; MENUS opens the editor; RESET BARE affordance dispatches the command. verify.ps1 CLEAN
    - Validates: Requirement 13.1-13.13; configuration-system 19.7
  - [x] 24.11 Update `docs/quality/TCR.md` rows; mark tasks 24.x + 23.9 done; CR-NR-075 -> DONE; verify.ps1; rebuild; commit+push
    - Covers: project standards

---

## Phase (nav-stack) -- Per-tab Navigation_Stack (CR-CH-022, Requirement 14; reconciles Req 5)

- [ ] 25. Per-tab Navigation_Stack (uniform END pop; transform-in-place; START the only tab-creator)
  - NOTE: 25.1-25.4, 25.6, 25.8-25.9 done. 25.5 (full multi-segment `=a.b`/`=a;b`
    separator parser) and 25.7 (session persistence of nav_stack) deferred -- see
    their sub-notes. The core stack, END/RETURN, START forms, and in-place
    navigation are implemented and tested; single-key `START =0` works via the
    existing single-segment fastpath resolver. Implementation used shell-level
    `navigate_to`/`start_new_workspace` (not TabManager methods as 25.2 drafted).
  - [x] 25.1 Add `nav_stack: Vec<ff_session::WorkspaceDescriptor>` to `TabState` (empty in every constructor/base_tab!); add `current_descriptor(&TabState)` deriving the current Context's descriptor (reuse descriptor_for_tab; add kind-only descriptors for ThemeEditor/MenusEditor)
    - Covers: Requirement 14.1, 14.6
  - [x] 25.2 Add `TabManager::navigate_here(descriptor, push)` (push current descriptor when push, then reconstruct in place) + `insert_rooted_tab(descriptor)` (new tab rooted at a Context, empty stack)
    - Covers: Requirement 14.2, 14.3
  - [x] 25.3 Route EVERY navigation arm through navigate_here on the current tab: SETTINGS/A/SETTINGS ns, FILES/=FILES, CATALOGS, PLUGINS, MACROS, THEMES, MENUS, COMMANDS, LOG, MENU/MENU <name>, POM fastpath; retire the transform-if-POM-else-open-new-tab split; wrap per-Context shell-global re-derivation (settings namespace, theme working copy, menus editor) in helpers that call navigate_here
    - Covers: Requirement 14.2, 14.6, 14.12
  - [x] 25.4 Rewrite END to pop-one-level (empty stack -> close tab; last tab -> file.exit); rewrite RETURN to collapse-to-root; delete pending_return_to_pom (mod.rs + update.rs), the namespace_filter END branch, and menus_editor_panel.opened_from_settings; files_panel ReturnToPom routes through the END-pop path
    - Covers: Requirement 14.4, 14.5, 14.10, 14.11
  - [ ] 25.5 Chained-path separator semantics: `=` resets to POM origin with empty stack (Req 14.7); `;` pushes each intermediate; `.` collapses (no push); implement/extend the chained resolver so `=0.E` END->POM and `=0;E` END->Settings->POM (reconciles Req 5.7-5.11)
    - Covers: Requirement 14.3, 14.7; menu-workspace 5.7-5.11
    - DEFERRED: multi-segment chained paths (`=a.b`, `=a;b`) were never
      implemented (the fastpath resolver only handles a single `=<key>`), so this
      is a follow-up. The stack machinery it needs (push vs collapse, `=` origin)
      is in place via `navigate_to(descriptor, push)`; wiring the multi-segment
      parser to it is the remaining work. Single-segment navigation and START =0
      work today.
  - [x] 25.6 START argument parsing: `START` (new POM tab, empty stack); `START =<path>` (new POM tab then drill, POM on stack); `START <arg>`/`START <name>` (new tab rooted DIRECTLY at the resolved Context, empty stack; unresolved -> POM tab + status)
    - Covers: Requirement 14.8, 14.9
  - [ ] 25.7 Optional session persistence: extend SessionTabState with an optional nav_stack (default empty) so a drilled-in Workspace restores its back-path; older sessions load empty
    - Covers: Requirement 14.6 (persistence note)
  - [x] 25.8 Tests: per-tab stacks independent; navigate pushes + transforms in place (no new tab); END pops one level and reconstructs; empty-stack END closes tab / exits when last; A->B->C->END->B->END->A->END->exit; `=0.E` vs `=0;E` END behaviour; START / START =0 / START Settings rooting; B053 (POM->Settings->Menus->END->Settings) via the stack; no navigation arm opens a new tab. verify.ps1 CLEAN
    - Validates: Requirement 14.1-14.12
  - [x] 25.9 Update TCR rows; mark tasks 25.x done; supersede B053 interim fix note; CR-CH-022 -> DONE; verify.ps1; rebuild; commit+push
    - Covers: project standards
- [ ] 26. Menu Workspace tab order and focusable calendar (Requirement 15, CR-CH-023)
  - [x] 26.1 Confirm enabled option rows are focusable buttons and disabled options are non-focusable Labels; rely on the shared shell Boundary_Policy (no menu-workspace focus ring)
    - Covers: Requirement 15.1, 15.2, 15.3, 15.4
  - [x] 26.2 Replace the calendar `<`/`>` painted hit-region with two real focusable `egui::Button`s in `primary_option_menu::render_calendar`, returning `CalendarNav` on click or Enter/Space
    - Covers: Requirement 15.7, 15.8
  - [x] 26.3 Position the calendar buttons so egui-native traversal reaches them after the last enabled option and before the Menu_Bar when `show_calendar` is true; no calendar stops when false
    - `render_menu_workspace` reports `last_interior_id` = calendar `>` when shown, else the last enabled option.
    - Covers: Requirement 15.5, 15.6
  - [x] 26.4 Keep day cells non-focusable (calendar stays minimal)
    - Covers: Requirement 15.10
  - [x] 26.5 egui_kittest harness test: on a menu-workspace, Tab visits each enabled option in order then the calendar `<`/`>`; disabled options skipped; no-calendar case ends on the last option
    - `menu_workspace_tab_visits_options_then_calendar`, `menu_workspace_disabled_option_is_skipped`, `menu_workspace_no_calendar_last_interior_is_last_option`. Full command-line/menu-bar/wrap boundary is manual (plan 1.3b); Shift+Tab reverse is manual (15.9).
    - Covers: Requirement 15.2, 15.3, 15.4, 15.5, 15.6; automated-dialog-testing 14.9
  - [x] 26.6 Update TCR rows for Requirement 15
    - Covers: Requirement 15 (all criteria)
---

## Phase (command-resolution) -- Keyword-less menu-name resolution + Settings launcher (CR-CH-025, Req 3.6, 11.11-11.12, 12.3)

- [x] 27. Retire SETTINGS/A special cases; menu-name resolution; Settings baseline reorder
  - [x] 27.1 Remove the hardcoded `SETTINGS`, `SETTINGS <ns>`, and `A` intercepts from `handle_command`; opening the Settings menu becomes menu-name resolution of the token `SETTINGS` (command-framework Req 8.11 stage 3), reusing `open_menu_by_name`/`open_settings_menu`
    - Covers: Requirement 11.11; command-framework Requirement 8.3, 8.11
  - [x] 27.2 Wire the keyword-less menu-name form: a bare token matching a resolvable Menu_Name (user `menus/<name>.toml` or built-in `pom`/`settings`) opens that menu without the `MENU` keyword; a trailing token is forwarded as the Option_Key (equals `MENU <name> <key>`)
    - Covers: Requirement 11.11; command-framework Requirement 8.13
  - [x] 27.3 Enforce built-in precedence: a Menu_Name resolves only when no earlier chain stage (current-menu Option_Key, built-in command/Command_ID) claims the token (shadowing rule)
    - Covers: Requirement 11.12; command-framework Requirement 8.10
  - [x] 27.4 Move the on-menu Option_Key lookup to be the first chain stage; a non-matching token falls through to the remaining stages instead of erroring (Req 3.6 revision)
    - Covers: Requirement 3.6
  - [x] 27.5 Reorder `DEFAULT_SETTINGS_TOML` in `menu_workspace/defaults.rs` to Core[`A` CONFIG, `T` THEME, `M` MENUS] then Recovery[`R` RESET BARE]; `group_separator = "line"`; correct stale `THEMES`->`THEME`; replace `A -> A` with `A -> CONFIG`. Built-ins stay code-only; RESET BARE unchanged
    - Covers: Requirement 12.3
  - [x] 27.6 Update the `default_settings_toml_*` unit tests (option count/order, `A -> CONFIG`, `T -> THEME`, `R -> RESET BARE`, groups Core/Recovery, ASCII-only); add/adjust tests for keyword-less `SETTINGS`/`SETTINGS T` resolution and the removed `A` command
    - Validates: Requirement 12.3, 11.11
  - [x] 27.7 Update `docs/quality/TCR.md`: set the CR-CH-025 menu-workspace rows to their correct status
    - Covers: Requirement 3.6, 11.11, 11.12, 12.3

---

## Phase (calendar-visibility) -- Calendar Visibility and Fit; Settings default off (CR-CH-026, Requirement 16, B060)

- [x] 28. Settings default: no calendar
  - [x] 28.1 Add `show_calendar = false` to `DEFAULT_SETTINGS_TOML` in `menu_workspace/defaults.rs` (compiled Recovery_Baseline Settings). Keep the `MenuFile` field default `true` (POM + author-enabled menus unaffected)
    - Covers: Requirement 16.1
  - [x] 28.2 Update `default_settings_toml_*` unit tests to assert `show_calendar = false` parses and is preserved (`default_settings_toml_hides_calendar`); the serialiser already emits `show_calendar = false` only when != default (covered by existing serialiser tests)
    - Validates: Requirement 16.1
  - [x] 28.3 Full-shell egui_kittest test `full_shell_settings_tab_walks_options_only_no_calendar_stops`: from the Settings command field, Tab walks the option rows; reported first/last interior are option ids (distinct), not a calendar id
    - Validates: Requirement 16.1, 16.4

- [x] 29. Calendar fit-aware layout in `render_menu_workspace`
  - [x] 29.1 Define `CALENDAR_MIN_WIDTH` (180px), `OPTION_LIST_MIN_WIDTH` (260px), `CALENDAR_GAP` (32px); compute available width at the option/calendar row
    - Covers: Requirement 16.2, 16.6
  - [x] 29.2 WHEN `show_calendar` true AND width fits: render the calendar in a reserved right-hand column WITHIN the visible clip width and constrain the option `ScrollArea` (`max_width` + `set_max_width`) to the remaining width, so the calendar (and its `<`/`>`) is fully on-screen and never overlaps the option list
    - Covers: Requirement 16.2, 16.5, 16.6
  - [x] 29.3 WHEN `show_calendar` true BUT width does NOT fit: omit the calendar for that frame (decision uses only the current frame's available width -> restored on a wider frame; no persisted state) and set `result.last_interior_id = last_enabled_option_id` (do NOT report the calendar ids)
    - Covers: Requirement 16.3, 16.4
  - [x] 29.4 Failing egui_kittest tests first (RED-confirmed): `menu_calendar_shown_when_wide_next_button_is_on_screen` (reports `>` id AND its rect is inside `ui.clip_rect()` -- B060 regression guard); `menu_calendar_omitted_when_too_narrow_no_calendar_tab_stops` (narrow -> reported last interior is on-screen, no off-screen calendar stop)
    - Validates: Requirement 16.2, 16.3, 16.4, 16.5
  - [x] 29.5 Update `docs/quality/TCR.md`: set the CR-CH-026 menu-workspace Req 16.1-16.6 rows to ✅; bugs.md B060 -> FIXED
    - Covers: Requirement 16.1-16.6

## Phase (menu-bar) -- Configurable named menu bars (CR-NR-080, Requirement 17)

> The menu bar becomes a Menu_File rendered horizontally: top-level buttons PEEK
> their referenced submenu as a dropdown (not navigate); leaves dispatch commands
> (parity). Menu model + command resolution unchanged. SLICED: A is the immediate
> work; B-D follow in later phases.

- [x] 30. Slice A: data-driven horizontal menu bar (replaces the hardcoded bar)
  - [x] 30.1 Add `DEFAULT_MENUBAR_TOML` to `menu_workspace/defaults.rs` (code-only, parsed once), reproducing the current bar's top-level entries in order INCLUDING a trailing `Help`; each top-level option's `command` names its submenu (Settings -> `SETTINGS`, etc.). Add `default_menubar_menu()` accessor; unit tests for valid-TOML + ASCII-only + expected top-level option list
    - Covers: Requirement 17.2
  - [x] 30.2 Add `render_menu_bar_from_menu(&mut self, ctx, menu: &MenuFile)` (menu_workspace/render.rs or a `menu_bar` submodule): `TopBottomPanel::top("menu_bar")` + `menu::bar`, one `menu_button(option.description)` per top-level option in order
    - Covers: Requirement 17.1
  - [x] 30.3 PEEK inside each top-level button: resolve the option command to a menu (reuse `TargetResolver::menu_name_target` + loader/compiled default); render the referenced menu's options as `ui.button(child.description)`; on click `self.handle_command(&child.command)` + `ui.close_menu()`. Non-menu commands render as a direct actionable button dispatching their own command
    - Covers: Requirement 17.3, 17.4, 17.5
  - [x] 30.4 Boundary_Policy: capture the FIRST and LAST top-level button `response.id` into `self.menu_first_id` / `self.menu_last_id` from the data-driven loop; remove/replace `MENU_BAR_TOP_LEVEL_LABELS` + its `debug_assert_eq!`; retarget label-asserting tests to the Default_Menu_Bar option list
    - Covers: Requirement 17.6
  - [x] 30.5 Replace the hardcoded `render_menu_bar` body with a call to `render_menu_bar_from_menu(default_menubar_menu())` (Slice A always uses the default; naming/assignment come in B/C). POM/Settings vertical workspace untouched
    - Covers: Requirement 17.1, 17.7
  - [x] 30.6 Failing full-shell egui_kittest tests first: the bar renders the Default_Menu_Bar's top-level buttons in order (incl. Help); opening a top-level button peeks its submenu's options; activating a leaf dispatches its command (assert the resulting state change, e.g. THEME leaf changes the active theme); first Tab from the last interior still reaches the bar and wraps (Boundary_Policy preserved)
    - Validates: Requirement 17.1, 17.3, 17.4, 17.6
  - [x] 30.7 Update `docs/quality/TCR.md` (CR-NR-080 Slice A rows -> PASS); verify.ps1 CLEAN (FULL, nextest); rebuild ffwb.exe
    - Covers: Requirement 17.1-17.7

- [x] 31. Slice B: named + editable menu-bar files
  - [x] 31.1 `resolve_menu_bar_menu()` resolves the default bar name (`DEFAULT_MENU_BAR_NAME` = `MB-POM`, slug `mb-pom`) to `menus/<slug>.toml` via the loader; a user file OVERRIDES the compiled default (`default_menubar_menu` = barebones POM), else falls back to it. `render_menu_bar` calls it. `MB-` naming convention (not enforced); the file uses the same slugging as the Menus Editor so it is authorable there + round-trips `show_in_menu_bar` via the serialiser. Tests: `resolve_menu_bar_falls_back_to_compiled_default`, `resolve_menu_bar_loads_user_file_when_present`.
    - Covers: Requirement 17.8

- [ ] 32. Slice C: per-workspace-kind menu-bar assignment (LATER)
  - [ ] 32.1 Config mapping workspace-kind (context name) -> menu-bar name, mirroring the keymaps per-kind pattern (CR-CH-027); active tab kind selects the bar; default bar when unassigned
    - Covers: Requirement 17.9

- [x] 33. Slice D: dynamic option sources -> Themes dropdown (delivers ex-CR-NR-077)
  - [x] 33.1 A menu-bar option with a dynamic source (`THEME LIST`) yields a dropdown generated from `list_all_themes` (`dynamic_menu_options`), each item dispatching `THEME <name>` via `handle_command` (shared `set_active_theme` apply+persist path). The bar render consults `dynamic_menu_options` before `peek_menu_options`. Mechanism delivered + tested; the barebones default does not yet wire a Themes dropdown (Option B -- usable when a menu includes a `THEME LIST` option).
    - Covers: Requirement 17.10, 17.11

## Phase (workspace-unify) -- unify the POM into a single Menu Workspace (CR-NR-082 Slice 1, Req 18)

Behaviour-preserving. Option (b): fully remove `TabKind::PrimaryOptionMenu`.
(Note: the deferred Slice C per-kind menu-bar, task 32, is subsumed by CR-NR-082
Slice 3 -- per-named-workspace menu-bar/keymap -- gated later.)

- [x] 34. Unify POM and Menu Workspace into one kind
  - [x] 34.1 Fold the barebones-seed path: generalise `ensure_pom_menu_loaded`
          so any MenuWorkspace tab whose menu is unloaded gets its menu loaded,
          with the `recovery_pom_menu()` fallback used when the menu name is
          `pom` and no file/parse. Failing tests first.
    - Validates: menu-workspace Requirement 18.2
    - Note: implemented via a stable `is_home` marker on `TabState` (set by
      `TabState::pom`); `ensure_pom_menu_loaded` keys off `is_home` and seeds
      `menus/pom.toml` with the `recovery_pom_menu()` barebones fallback.
  - [x] 34.2 Remove `TabKind::PrimaryOptionMenu`; make the Home Context a
          `TabKind::MenuWorkspace` tab with menu name `pom`. Update
          `TabState::pom`/`insert_pom_tab` to build a MenuWorkspace Home tab.
    - Validates: menu-workspace Requirement 18.1
  - [x] 34.3 Collapse the two render arms in `shell/render.rs` into one
          `MenuWorkspace` arm (shared renderer); title-line `is_pom` keys off the
          menu name `pom`, not the removed kind.
    - Validates: menu-workspace Requirement 18.3, 18.5
    - Note: `title_line_text`/`render_title_line` key off `is_home` (not the
      menu name), which is stable before the menu loads.
  - [x] 34.4 nav_stack POM fallback + END/RETURN unwind target the Home Menu
          Workspace (menu `pom`); keymap `context_name_for_kind` resolves Home to
          the `pom` context by menu name (no binding change).
    - Validates: menu-workspace Requirement 18.4, 18.6
    - Note: new `context_name_for_tab` resolves Home (`is_home`) to `pom` and
      delegates otherwise to `context_name_for_kind`; a `set_active_tab_home`
      helper drives the nav-stack POM/legacy/fallback reconstructions.
  - [x] 34.5 Persistence: Home Context persists as `Menu { name: "pom" }`; remove
          the POM `CustomWorkspace` arm from `descriptor_for_tab`. Retain
          `WorkspaceKind::PrimaryOptionMenu` / `PersistedTabKind::PrimaryOptionMenu`
          as legacy-read mappings only (`from_legacy` -> Home Menu descriptor);
          new saves never emit them.
    - Validates: menu-workspace Requirement 18.7, 18.8
    - Note: both `descriptor_for_tab` and `descriptor_for_current_context` map
      Home to `Menu{name:"pom"}` (stable regardless of loaded title);
      `WorkspaceKind::PrimaryOptionMenu`/`PersistedTabKind::PrimaryOptionMenu`
      kept read-only and route legacy sessions to `set_active_tab_home`.
  - [x] 34.6 Retarget existing POM tests that name `TabKind::PrimaryOptionMenu`
          to the Home Menu Workspace (kind MenuWorkspace + menu `pom`); add tests:
          no `PrimaryOptionMenu` kind exists; fresh launch seeds barebones POM;
          END/RETURN returns to Home; a legacy-POM session restores the Home
          Context. verify.ps1 CLEAN (FULL, nextest); rebuild ffwb.exe; update TCR
          Req 18; update project-master.
    - Covers: menu-workspace Requirement 18 (all criteria; behaviour preserved)

## Phase (menu-desc-layout) -- description-driven menu layout (CR-CH-032, Req 16.7-16.11)

Behaviour change to the Menu_Workspace horizontal layout: the calendar's
visibility and the option-column width follow the descriptions' natural one-line
width. Calendar hides BEFORE descriptions wrap. Replaces the fixed-minimum fit
rule of CR-CH-026 (Req 16.3/16.6). No API change; the calendar renderer and the
B065 row-render are unchanged.

- [x] 35. Description-driven menu layout (3-tier ladder)
  - [x] 35.1 Add the pure `natural_option_list_width` helper in
          `menu_workspace/render.rs` (widest key+command prefix + widest
          single-line description, no wrap, + scrollbar allowance; uncapped).
          Failing unit tests first.
    - Validates: menu-workspace Requirement 16.7
    - Note: split into a pure `natural_width_from_rows` (unit-testable, no `Ui`)
      + `natural_option_list_width` (measures galleys against the live fonts).
  - [x] 35.2 Replace the `display_calendar`/`option_list_max_w` decision in
          `render_menu_workspace` with the 3-tier ladder: Tier 1 (calendar +
          one-line, option column = natural width, trailing blank space to the
          right of the calendar); Tier 2 (calendar hidden, option column = full
          width, one line); Tier 3 (calendar hidden, full width, descriptions
          wrap). Apply the option-column width as a DEFINITE width.
    - Validates: menu-workspace Requirement 16.8, 16.9, 16.11
    - Note: option column now uses `set_width` + `set_max_width` (definite
      width); removed the unused `OPTION_LIST_MIN_WIDTH` const.
  - [x] 35.3 Keep the description `.wrap()` as the tier-3 fallback (no wrap-mode
          flag); confirm the focus contract (last interior = `>` when shown, else
          last option) still keys off the tier-derived `display_calendar`.
    - Validates: menu-workspace Requirement 16.10; Requirement 15.5, 15.6
  - [x] 35.4 Add egui_kittest harness tests for the three tiers (wide = calendar
          shown + option-column rect approx natural width + trailing space;
          medium = calendar hidden + full width + no wrap; narrow = calendar
          hidden + wrapped). Confirm existing B060 calendar tests + POM/menu
          focus tests still pass. verify.ps1 CLEAN (FULL, nextest); rebuild
          ffwb.exe; update TCR Req 16.7-16.11; update project-master; core
          acceptance test-plan 2.2b/2.2d.
    - Covers: menu-workspace Requirement 16.7-16.11 (behaviour-driven layout)

## Phase menu-dispatch-converge (CR-CH-043) -- one Option-Selection path

One implementation slice satisfies BOTH menu-workspace Requirement 19 and
command-framework Requirement 14 (the two halves of the convergence). Behaviour-
preserving; TDD (failing full-shell test first). Do NOT start until the gate is
approved.

- [x] 36. Converge all Option_Selection onto one `handle_command` path
  - [x] 36.1 Write the failing full-shell egui_kittest tests FIRST:
          `full_shell_pom_settings_click_equals_typed_settings` (clicking POM
          `Settings` and typing `SETTINGS` reach the SAME in-place Settings menu --
          same active tab id + descriptor), and
          `full_shell_pom_option_key_and_click_same_result` (typing a POM
          Option_Key and clicking its row give identical results). Confirm they
          FAIL against the current divergent code (red).
    - Validates: menu-workspace Requirement 19.1, 19.2; command-framework Requirement 14.1, 14.2
  - [x] 36.2 Collapse `try_current_menu_option` (non-POM, skips `is_home`) and
          `resolve_pom_option_key` (POM only) into ONE current-menu Option_Key
          resolver that looks the key up against the ACTIVE menu regardless of
          `is_home` and dispatches its `command` via `handle_command`. Preserve
          the `=` Navigation_Origin fastpath as command-string parsing feeding the
          same resolver (`=0.K` still resolves against the POM).
    - Validates: menu-workspace Requirement 19.2, 19.4; command-framework Requirement 14.1
  - [x] 36.3 Remove the click-only pre-branch in `shell/update.rs`: a clicked
          option resolves to `option.command` and calls
          `handle_command(option.command)`, identical to a typed key. Preserve the
          inline `[options.target]` capability (Req 10.6) by resolving it inside
          the pipeline, not via a click-only fork.
    - Validates: menu-workspace Requirement 19.3; command-framework Requirement 14.1
  - [x] 36.4 Fold the `open_named_menu` name-switch into the command handlers:
          `SETTINGS` owns in-place (`open_settings_menu`), `POM` owns Home Context,
          user-menu commands own new tab (`open_menu_by_name`). The
          `CommandTarget::Menu { name }` arm and `try_menu_name_dispatch` invoke
          the owning command rather than choosing placement. Remove
          `open_named_menu` as a placement router.
    - Validates: menu-workspace Requirement 19.5; command-framework Requirement 14.3, 14.4
  - [x] 36.5 Confirm the disabled-option message (Req 3.7 / 19.6) and the
          unresolved-command error (Req 3.6 / 10.5 / 19.7) are emitted on the one
          path for every affordance. Run the full existing menu-workspace / B075 /
          workspace-conformance focus suite -- all must stay green (behaviour-
          preserving, Req 19.8 / 14.6). verify.ps1 CLEAN (FULL, nextest); rebuild
          ffwb.exe; update TCR Req 19.1-19.8 + command-framework 14.1-14.6; update
          project-master; core acceptance test-plan.
    - Covers: menu-workspace Requirement 19; command-framework Requirement 14

## Phase title-chrome-align (CR-CH-042) -- single config-driven centered title + short POM tab + POM command

One slice satisfies menu-workspace Requirement 20 and menu-and-statusbar
Requirement 17.3/17.6/17.11. Behaviour-preserving; TDD (failing tests first). Do
NOT start until the gate is approved.

- [ ] 37. Single config-driven centered Menu Workspace title; short POM tab; POM command
  - [ ] 37.1 Write failing full-shell egui_kittest tests FIRST:
          `full_shell_pom_title_line_shows_menu_title_not_banner` (POM Title_Line
          text == loaded `menus/pom.toml` title, not `FileForge Workbench v...`),
          `pom_tab_header_is_short_label` (POM tab header == `POM`),
          `pom_command_opens_home_context` + bare `START` alias equivalence.
          Confirm they FAIL (red).
    - Validates: menu-workspace Requirement 20.2, 20.3, 20.4, 20.5; menu-and-statusbar Req 17.3
  - [ ] 37.2 Re-source the Title_Line for a Menu_Workspace (incl. the POM) from the
          LIVE loaded `menu.title` (raw) in `title_line_text` / `kind_title`
          (`shell/mod.rs`); remove the POM `is_home` app-banner early return.
    - Validates: menu-workspace Requirement 20.1, 20.2, 20.3, 20.8; menu-and-statusbar Req 17.11
  - [ ] 37.3 Center the Title_Line for ALL `TabKind::MenuWorkspace` tabs uniformly
          in `render_title_line_into_ui` (`shell/render.rs`), generalising the
          current POM `is_pom` centered branch; keep POM theme styling; non-menu
          Contexts keep their existing Title_Line rendering.
    - Validates: menu-workspace Requirement 20.2, 20.7; menu-and-statusbar Req 17.11
  - [ ] 37.4 Remove the duplicate centered `menu.title` body heading (+ its
          `add_space`) above the option list in `menu_workspace/render.rs`; confirm
          the Layout_Tier / calendar-fit (Req 16) and focus contract are untouched.
    - Validates: menu-workspace Requirement 20.1, 20.9; menu-and-statusbar Req 17.11
  - [ ] 37.5 POM tab header short label `POM` (`shell/render_chrome.rs` Home-tab
          path; `TabState::pom` / derivation per Req 20.6); other tabs unchanged.
    - Validates: menu-workspace Requirement 20.4, 20.6
  - [ ] 37.6 Make `POM` the first-class Home opener + register it; accept bare
          `START` as its alias (preserve `START =<path>` / `START <arg>` forms).
    - Validates: menu-workspace Requirement 20.5; Requirement 14.8
  - [ ] 37.7 Close: existing menu / B050 stale-title / workspace-conformance focus
          tests green (behaviour-preserving); verify.ps1 CLEAN FULL nextest; rebuild
          ffwb.exe; TCR menu-workspace Req 20 + menu-and-statusbar Req 17.3/17.6/17.11
          PASS; update project-master; core acceptance test-plan.
    - Covers: menu-workspace Requirement 20; menu-and-statusbar Requirement 17 (revised)
