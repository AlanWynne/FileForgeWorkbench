# Review -- Strip core: trim the default POM (CR-CH-073, Option B)

**Verdict: APPROVED**

Working-tree change (no branch/commit, per the owner's standing decision) that
removes the Catalogs (key 1) and Files (key 2) options from the compiled
Recovery_Baseline POM, leaving the reconsidered trimmed core `0 Settings /
3 Help / X`. Reviewed against the owner-approved gate decision (Option B,
`removeCommands: false`), the EARS criteria, and the coder's recorded
verification. No blocking findings.

## What I verified

### Code -- the core change is correct and matches the approved set
- `crates/ff-desktop/src/menu_workspace/defaults.rs`: `DEFAULT_POM_TOML` deletes
  exactly the `key = "1" / command = "Catalogs"` and `key = "2" / command =
  "Files"` `[[options]]` blocks; survivors `0 Settings`, `3 Help`, `X` retained
  in a single `Core` group, keys unchanged. This matches the gate.md trimmed set
  (section (a)) and implementation.md exactly. Doc-comments updated to describe
  the trim and the Option B retention note.

### Tests assert ABSENCE and were updated, not weakened/deleted
- `recovery_pom_menu_has_barebones_options` -> `keys == ["0","3","X"]`,
  `commands == ["Settings","Help","X"]` (`// Validates: Requirement 12.2`).
- `default_pom_toml_has_recovery_baseline_options` -> `len == 3`,
  `commands == ["Settings","Help","X"]` (`// Validates: Requirement 12.2, 2.1h`).
- `default_menubar_bar_visible_options_are_settings_catalogs_files_help` renamed
  to `..._settings_help`, asserts `["Settings","Help"]` (`// Validates: 17.2a`).
- `tests_menu_workspace::menu_bar_has_file_catalogs_menu` retargeted to
  `menu_bar_omits_catalogs_and_files` -- now asserts BOTH Catalogs AND Files are
  absent (a positive absence assertion, not a deletion).
- `menus_editor_loads_recovery_baseline_when_no_file` -> `["0","3","X"]`.
- Retargeted nav/command tests (`pom_key_resolves_to_configured_command`,
  `pom_option_key_type_and_click_same_result`, `fastpath_notation_navigates_to_option`
  -> `0.C`, `return_from_drilled_in_collapses_to_visual_root`, `equals_1_... ->
  equals_0_works_from_a_non_pom_tab`, `equals_8_... -> equals_0_command_routes_to_settings`,
  `equals_5_... -> equals_3_command_routes_to_help`, `swap_n` precondition now
  `START`) all point at RETAINED options/commands. I confirmed every referenced
  `KindTag` variant exists in `tab_state.rs` (`MenuWorkspace`, `HelpContext`,
  `FilesPanel`, `FileExplorerPanel`).

### Option B honoured -- commands/Contexts retained
- `command_config/mod.rs`, `workspace_kind`, `TabKind`/`KindTag`, and descriptor
  reconstruction are NOT in the diff (git status confirms only defaults.rs + 5
  test files changed in source).
- The three regression guards exist and are unchanged:
  `tests_nav::equals_files_jump_path_still_opens_file_explorer`,
  `tests_nav::bare_files_still_opens_file_explorer_via_front_door`,
  `command_config::tests::bare_files_still_opens_file_explorer_after_equals_files_removal`.
  Typed `FILES`/`CATALOGS` and the `=FILES` jump remain reachable, per the gate.

### Docs internally consistent with the code
- menu-workspace Req 12.2 (trimmed set 0/3/X), Req 2.1h (SETTINGS/HELP/X resolve
  by name), Req 17.2a (bar-visible = Settings/Help), Req 12 User Story -- all
  restated consistently and cross-referencing CR-CH-073 and the extension effort
  wf_21b74c6aa15ba293.
- startup-and-session Req 14.3 barebones list reconciled to the same set; 14.3a
  reframed (CATALOGS reachable by typing, not a barebones option); Req 7 menu-bar
  mirrors Settings/Help.
- design.md CR-CH-073 section records the A/B decision (Option B approved),
  reasoning, scope, and testability. tasks.md Phase strip-core-pom-trim uses only
  `[ ]`/`[x]` with titled tasks and Req cross-refs. TCR.md flips the five
  CR-CH-073 rows to covered with backing test names. change-log.md CR-CH-073
  entry present.

### Standards
- 400-line non-test rule: `defaults.rs` non-test code ends at line 192 (`mod
  tests` at 193); the change removed TOML lines, so it is further under the limit.
- ASCII: changed `.rs` files and the new design.md content are clean. The only
  non-ASCII matches are PRE-EXISTING lines in `startup-and-session/requirements.md`
  (arrows/en-dashes/warning emoji at lines 27/66/69/97/248/251/284/368/452), none
  of which fall in this change's diff hunks -- out of scope.

### Verification evidence (read, not re-run)
implementation.md records the scoped runs the coder performed and their results:
`cargo fmt -p ff-desktop` clean; `cargo clippy -p ff-desktop --tests -- -D
warnings` returncode 0 no warnings; and per-module scoped test runs all green
(`defaults::tests` 32/32, `tests_nav` 45/45, `tests_command` 80/80,
`tests_nav_ladder` 9/9, `tests_misc` 60/60, `tests_menu_workspace` 98/98, plus
`pom_`/`equals_`/`menu_bar_` filter runs). A documented RED-first TDD trail is
present. Per the step instructions I did not re-run the suites or the full gate;
the owner runs the authoritative full `cargo gate`.

## Blocking findings

None.

## Notes (non-blocking, not required for approval)
- The pre-existing non-ASCII in `startup-and-session/requirements.md` predates
  this change and is out of scope here; worth a future ASCII cleanup pass but it
  does not gate this change.
