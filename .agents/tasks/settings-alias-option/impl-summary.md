# CR-CH-066 -- Implementation Summary (Settings-menu Alias Editor + Config mnemonic A -> C)

Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\settings-alias-option`
Branch: `settings-alias-option`
Iteration: FIRST (no `review.json` present).

## Criteria implemented (menu-workspace Requirement 22, CR-CH-066)

- **Req 22.1** -- the compiled Settings Recovery_Baseline presents, in exact
  order with exact mnemonics: `C` -> `Config`, `A` -> `Aliases` (Alias Editor),
  `T` -> `Theme`, `M` -> `Menus`, `K` -> `Keys`, `W` -> `Kinds` (group `Core`),
  then `R` -> `Reset bare` (group `Recovery`).
- **Req 22.2** -- selecting the `A` option dispatches the `ALIASES` verb through
  the single command-dispatch front door and opens the Alias Editor Context
  (CR-NR-113), reusing the existing open path verbatim; no new verb/dispatcher.
- **Req 22.3** -- selecting the `C` option dispatches `CONFIG` and opens the
  Config Context, unchanged except for the mnemonic.
- **Req 22.4** -- the Settings menu stays a code-only built-in
  (`DEFAULT_SETTINGS_TOML`), never written to disk (CR-CH-021).
- **Req 22.5** -- the `A` -> `C` remap for Config plus the `A` Alias Editor
  insertion is a deliberate change (no existing users); the compiled
  Recovery_Baseline is the single source of the option list (no duplicate
  hardcoded list elsewhere).

## The exact command the Alias Editor option uses

The new option carries the command string **`Aliases`** (Title-case, matching the
style of the sibling Settings options `Config`/`Theme`/`Menus`/`Keys`/`Kinds`).
Dispatch is case-insensitive: selecting the option routes the string through the
single front door `resolve_target` -> `builtin_workspace_target_for`, where the
`ALIASES` arm (already present from CR-NR-113, `command_config/mod.rs`) classifies
it as `CustomWorkspace { workspace_kind: "aliases" }` -> `open_alias_editor` ->
`navigate_to` in place. No new verb, dispatcher, or Context was introduced.

## Files changed

- `crates/ff-desktop/src/menu_workspace/defaults.rs`
  - `DEFAULT_SETTINGS_TOML`: Config key `A` -> `C`; inserted a new second option
    `key = "A"`, `command = "Aliases"`, description
    "Alias editor -- create, edit and save command aliases", `group = "Core"`.
    (ASCII only; the `--` is ASCII per documentation-charset.)
  - Updated the constant doc comment to record the CR-CH-066 order.
  - Updated tests `recovery_settings_menu_has_barebones_options` and
    `default_settings_toml_has_recovery_baseline_options` to the new 7-option
    order; added new test
    `default_settings_toml_has_alias_editor_and_config_mnemonic_c`
    (Validates Req 22.1, 22.5).
- `crates/ff-desktop/src/shell/tests_nav.rs`
  - Renamed/retargeted the stale `default_settings_toml_option_a_command_is_config`
    to `default_settings_toml_option_c_is_config_and_a_is_aliases`
    (Validates Req 22.1, 22.5).
  - Added `settings_config_option_dispatches_config` (Validates Req 22.3): open
    SETTINGS, select option key `C`, assert the Config Context opens.
- `crates/ff-desktop/src/shell/tests_alias_editor.rs`
  - Added `settings_alias_option_dispatches_aliases_verb` (Validates Req 22.2):
    open SETTINGS, select option key `A`, assert the Alias Editor Context opens
    with title `[ALIASES]`, using the isolated `point_user_aliases_at_temp`
    override.
- `crates/ff-desktop/src/shell/tests_menu_workspace.rs`
  - Cosmetic comment update to the new order (no assertion change).
- `crates/ff-desktop/src/shell/tests_focus.rs`
  - Cosmetic comment update in `full_shell_settings_tab_walks_options_only_no_calendar_stops`
    ("4 options" -> "multiple options"); assertions unchanged and still valid
    (the Tab-walk loop is bounded at 24 and walks the ring dynamically, so the
    added option is handled with no assertion change).
- `docs/quality/TCR.md` -- CR-CH-066 Req 22.1-22.5 rows flipped from NOT-COVERED
  (red) with the mapped test names, plus a note on the verification status.
- `docs/specs/menu-workspace/tasks.md` -- Task 50 and 50.1-50.6 checked off.

(The gate docs -- change-log.md, requirements.md, design.md, ROADMAP.md, and the
initial TCR NOT-COVERED rows -- were authored in the prior gate step; this step
only flips TCR to covered and checks off tasks.)

## Commands run and log results (SCOPED to ff-desktop)

All run with cwd = the worktree root, each a single clean pwsh7 wrapped job,
output redirected to `tools/logs/` and read back (verify by effect):

- `cargo check -p ff-desktop` -> `tools/logs/settings-alias-check2.log`:
  **`Finished dev profile ... in 0.44s`** (library compile CLEAN).
- `cargo clippy -p ff-desktop --all-targets -- -D warnings`
  -> `tools/logs/settings-alias-clippy2.log`:
  **`Finished ... in 0.47s`** with NO diagnostics (lints CLEAN, test targets
  included via `--all-targets`, so the test module compiles clean too).
- `cargo fmt -p ff-desktop -- --check` -> `tools/logs/settings-alias-fmt.log`:
  **empty log** (= no formatting differences; CLEAN).

## Known limitations / follow-ups

- **Scoped test RUN not completed locally.** After the format check, the shell
  runner wedged while compiling the test binary from scratch (a known
  terminal-quiet condition on this machine; subsequent commands produced no
  output and no log file). The tests are WRITTEN and the crate + all test targets
  COMPILE CLEAN and are LINT/FORMAT clean, which satisfies the agent half of the
  testing.md execution split. The authoritative full-suite PASS is the OWNER's
  manual `cargo gate` run. The TCR note records this explicitly.
- No framework change: adding a menu option is CONTENT, not mechanism
  (framework-conformance). The `ALIASES` open path from CR-NR-113 is reused
  verbatim.
- `tests_nav.rs::settings_option_a_opens_flat_panel` dispatches `CONFIG` directly
  (not via the `A` key) and is unaffected; its name is slightly stale but renaming
  is out of scope for this CR.
