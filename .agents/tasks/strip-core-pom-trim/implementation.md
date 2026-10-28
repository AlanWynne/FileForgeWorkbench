# Implementation Note -- Strip core: trim the default POM (CR-CH-073)

**Effort:** strip-core-pom-trim (SUBTRACTIVE half; the extension re-introduction is
the separate effort wf_21b74c6aa15ba293).
**Decision followed:** owner-APPROVED Option B (`removeCommands: false`) -- remove
ONLY the default MENU options; LEAVE the `FILES`/`CATALOGS` commands and their
Contexts (File Explorer / Catalog Explorer) reachable when typed. The optional
`3 HELP` -> `1` renumber was DECLINED (Help stays on key `3`).
**Working style:** all edits made DIRECTLY in the main workspace; NO worktree, NO
branch, NO commit. Changes left UNCOMMITTED for owner review.

## What changed (the trimmed core)

`DEFAULT_POM_TOML` went from 5 options (`0 Settings`, `1 Catalogs`, `2 Files`,
`3 Help`, `X`) to the reconsidered trimmed core of 3 options, single `Core` group:

| key | command  | description                                                    | bar |
|-----|----------|----------------------------------------------------------------|-----|
| `0` | Settings | FFWB Settings and Client Parameters                            | yes |
| `3` | Help     | Help -- workbench help and topics                              | yes |
| `X` | X        | Collapse to the visual root / close workspace (exit when last) | no  |

Survivor keys `0`/`3`/`X` retained unchanged. Catalogs (key 1) / Files (key 2)
removed from the default menu only; their commands + Contexts remain (Option B).

## Files changed

Source (ASCII-only, under the 400-line non-test rule):

1. `crates/ff-desktop/src/menu_workspace/defaults.rs`
   - `DEFAULT_POM_TOML`: deleted the key `1` Catalogs and key `2` Files
     `[[options]]` blocks; updated the constant's doc-comment prose (CR-CH-073
     trim, Option B retention note) and the `default_menubar_menu` doc-comment
     ("leaving Settings / Help").
   - Test `recovery_pom_menu_has_barebones_options`: keys `["0","3","X"]`,
     commands `["Settings","Help","X"]`. `// Validates: Requirement 12.2`.
   - Test `default_pom_toml_has_recovery_baseline_options`: `len == 3`, commands
     `["Settings","Help","X"]`. `// Validates: Requirement 12.2, 2.1h`.
   - Test `default_menubar_bar_visible_options_are_settings_catalogs_files_help`
     RENAMED to `default_menubar_bar_visible_options_are_settings_help`; asserts
     `["Settings","Help"]`. `// Validates: Requirement 17.2a`.

2. `crates/ff-desktop/src/shell/tests_menu_workspace.rs`
   - `menu_bar_has_file_catalogs_menu` RETARGETED to
     `menu_bar_omits_catalogs_and_files` (asserts Catalogs AND Files absent).
     `// Validates: Requirement 17.2a`.
   - `menus_editor_loads_recovery_baseline_when_no_file`: expected keys updated
     to `["0","3","X"]` (menus editor loads the same trimmed baseline). Existing
     `// Validates: Requirement 13.3`.

3. `crates/ff-desktop/src/shell/tests_nav.rs`
   - `pom_key_resolves_to_configured_command`: retargeted from key 2 -> FILES to
     the retained key 0 -> SETTINGS (-> MenuWorkspace).
   - `pom_option_key_type_and_click_same_result`: retargeted from option 1
     (Catalogs) to the retained option 0 (Settings).
   - `fastpath_notation_navigates_to_option`: retargeted from `2.1` to `0.C`
     (Settings -> Config).
   - `return_from_drilled_in_collapses_to_visual_root`: drill-in retargeted from
     key 1 to key 0 (Settings).

4. `crates/ff-desktop/src/shell/tests_nav_ladder.rs`
   - `equals_1_works_from_a_non_pom_tab` RENAMED to
     `equals_0_works_from_a_non_pom_tab`: `=0` from a non-POM (Keys) tab resolves
     POM option 0 (Settings -> MenuWorkspace).

5. `crates/ff-desktop/src/shell/tests_command.rs`
   - `equals_8_command_routes_to_plugin_manager` RENAMED to
     `equals_0_command_routes_to_settings` (`=0` -> MenuWorkspace).
   - `equals_5_command_routes_to_macro_library` RENAMED to
     `equals_3_command_routes_to_help` (`=3` -> HelpContext).

6. `crates/ff-desktop/src/shell/tests_misc.rs`
   - `swap_n_out_of_range_errors_and_keeps_active`: the ">=2 tabs" precondition
     uses `START` instead of the removed POM key `2`.

Docs:

7. `docs/quality/TCR.md`: the five CR-CH-073 NOT-COVERED rows (Req 12.2, 2.1h,
   17.2a, startup 14.3, Option B regression guard) flipped to covered/PASS with
   the backing test names.

NOT touched (Option B -- deliberately left in place): `DEFAULT_POM_TOML`'s
Settings menu; `command_config/mod.rs::builtin_workspace_target_for`; the File
Explorer / Catalog Explorer Contexts; the `tests_common.rs::is_shell_command`
static membership list (legacy catch-all containing `1`/`2`/`=1`/`=2` plus many
non-default keys -- no test asserts those absent, removing them would be out of
scope and risk breaking `is_shell_command("=0")`-style membership tests).

## TDD trail

RED confirmed first (tests updated against the UNCHANGED 5-option TOML):
`defaults::tests` scoped run showed exactly 3 failures --
`recovery_pom_menu_has_barebones_options` (left `["0","1","2","3","X"]`),
`default_pom_toml_has_recovery_baseline_options` (left len 5),
`default_menubar_bar_visible_options_are_settings_help` (left
`["Settings","Catalogs","Files","Help"]`). Then `DEFAULT_POM_TOML` was trimmed
and all went GREEN. A second RED surfaced during module verification
(`menus_editor_loads_recovery_baseline_when_no_file`, left `["0","1","2","3","X"]`)
and was fixed to `["0","3","X"]`.

## Scoped verification (compile-oriented per gate-policy.md; owner runs the full gate)

Run via `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<cmd>"`.
Results read from the proxy result files / logs (verify by effect, not Exit Code):

- `cargo fmt -p ff-desktop` -- completed, returncode 0.
- `cargo clippy -p ff-desktop --tests -- -D warnings` -- returncode 0, no
  warnings (this also compiles all test code = `cargo check` coverage).
- `cargo test -p ff-desktop --bin ffwb defaults::tests` -- 32 passed, 0 failed.
- `cargo test -p ff-desktop --bin ffwb tests_nav::` -- 45 passed, 0 failed.
- `cargo test -p ff-desktop --bin ffwb tests_command::` -- 80 passed, 0 failed.
- `cargo test -p ff-desktop --bin ffwb tests_nav_ladder::` -- 9 passed, 0 failed.
- `cargo test -p ff-desktop --bin ffwb tests_misc::` -- 60 passed, 0 failed.
- `cargo test -p ff-desktop --bin ffwb tests_menu_workspace::` -- 98 passed,
  0 failed.
- `pom_` filter run (53 tests) and `equals_`/`menu_bar_` filter runs all PASS,
  including the Option B guards and the `tests_focus` option-1/2 transform tests
  (which drive transforms via the FILES command, not the default POM options).

(`ff-desktop` is a binary crate; tests run under `--bin ffwb`. The reported
`Exit Code: -1` on some raw-path runs is the known capture artifact -- results
were confirmed from the proxy result files / redirected logs.)

## Known limitations / notes

- The AUTHORITATIVE full `cargo gate` / whole-workspace suite is the OWNER's
  manual run (gate-policy.md). This note's verification is scoped + compile-clean
  per policy; the owner's full gate is still required before any merge to `main`.
- No commit made (owner's standing working-style decision for this repo).
- Ephemeral `tools/logs/*.log` scratch files created during verification were
  removed.
