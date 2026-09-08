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

- [x] 1.1 Add `TabKind::MenuWorkspace(MenuWorkspaceState)` to `tab_state.rs`
  - Satisfies: Req 1 (data model), Req 2.1 (rendering contract)
- [x] 1.2 Add `PersistedTabKind::MenuWorkspace { file_path: String }` to
  `session_manager.rs`
  - Satisfies: Req 4.6 (session persistence)
- [x] 1.3 Write unit tests: `menu_workspace_tab_kind_exists`,
  `menu_workspace_persisted_tab_kind_round_trips`
  - Validates: Requirement 1.7, Requirement 4 (session persistence)

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

- [ ] 13.1 Add `DEFAULT_SETTINGS_TOML` const string to `menu_workspace/defaults.rs`
  with all 10 options, groups, and TOML structure matching cw-requirements.md Req 11.1
  - Satisfies: Req 11.1, 11.4, 11.5
- [ ] 13.2 Update `ensure_default_menu_files()` to write `settings.toml` when absent
  - Satisfies: Req 11.2, 11.3
- [ ] 13.3 Write unit tests: `default_settings_toml_is_valid_toml`,
  `default_settings_toml_has_10_options`, `default_settings_toml_ascii_only`
  - Validates: Requirement 11.1, 11.4, 11.5

### Task 14: Settings Namespace View routing

- [ ] 14.1 Add `namespace_filter: Option<String>` field to `SettingsPanelState`
  - Satisfies: Req 10.1, 10.2
- [ ] 14.2 Extend `shell/commands.rs` Settings routing to accept namespace argument
  (e.g. `SETTINGS editor` pre-populates filter with `editor.`)
  - Satisfies: Req 10.1, 10.2
- [ ] 14.3 Update tab title to `[SETTINGS:<namespace>]` when filter is set
  - Satisfies: Req 10.5
- [ ] 14.4 Update F3/END handler: return to Settings_Menu when in namespace view,
  return to POM when in Settings_Menu
  - Satisfies: Req 10.4, Req 12.2 (criterion 15.10)
- [ ] 14.5 Persist `namespace_filter` in session and restore on launch
  - Satisfies: Req 10.6
- [ ] 14.6 Write unit tests: `settings_namespace_filter_applied_on_open`,
  `settings_namespace_tab_title_includes_namespace`,
  `settings_end_from_namespace_view_returns_to_menu`
  - Validates: Requirement 10.1, 10.4, 10.5

### Task 15: TCR and Documentation Update (Phase CW-impl)

- [ ] 15.1 Update `docs/quality/TCR.md` -- set all CR-NR-047 rows to correct status
- [ ] 15.2 Update `docs/specs/project-master/tasks.md` -- mark Phase CW-impl tasks complete

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
