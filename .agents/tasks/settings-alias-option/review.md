# Review -- CR-CH-066 Settings-menu Alias Editor (+ Config mnemonic A -> C)

Branch: `settings-alias-option` (forked from `main` at `3e75200`)
Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\settings-alias-option`
Commit under review: `99b86ba feat(menu): add Alias Editor to Settings menu, remap Config A->C (CR-CH-066)`
Iteration: FIRST (no prior `review.json`).

## Scope of change

`git diff main...settings-alias-option --stat`: 11 files, 405 insertions / 45 deletions.
Exactly ONE production source file is touched:
`crates/ff-desktop/src/menu_workspace/defaults.rs` -- and within it only the
`DEFAULT_SETTINGS_TOML` constant (content), its doc comment, and the in-file
`#[cfg(test)]` module. The other `.rs` changes are test files
(`tests_alias_editor.rs`, `tests_nav.rs`, `tests_menu_workspace.rs`,
`tests_focus.rs`); the remaining files are docs (requirements, design, tasks,
change-log, ROADMAP, TCR). This matches a CONTENT-only change.

## Evidence read (not re-run)

- `tools/logs/settings-alias-check2.log` -- `cargo check -p ff-desktop` CLEAN.
- `tools/logs/settings-alias-clippy2.log` -- `cargo clippy -p ff-desktop
  --all-targets -- -D warnings` CLEAN (test targets included, so the new test
  code compiles clean too).
- `tools/logs/settings-alias-fmt.log` -- empty (= `cargo fmt -- --check` CLEAN).
- `tools/logs/settings-alias-test-defaults3.log` and `...defaults2.log` --
  `menu_workspace::defaults::tests` RAN: 19 passed, 0 failed, including
  `default_settings_toml_has_alias_editor_and_config_mnemonic_c`,
  `recovery_settings_menu_has_barebones_options`, and
  `default_settings_toml_has_recovery_baseline_options`.

The two activation tests (`settings_alias_option_dispatches_aliases_verb`,
`settings_config_option_dispatches_config`) were not captured in a run log (the
recorded run filtered on `menu_workspace::defaults`); they compile clean under
`--all-targets -D warnings`, and the authoritative full-suite pass is the
owner's manual `cargo gate` run per the testing.md compile-verify execution
split. This is the sanctioned agent/owner split for this workspace, not a gap.

## Judgement against the required criteria

### 1. Exact order and mnemonics (C Config, A Alias Editor, T Theme, M Menus, K Keys, W Kinds, R Reset bare) -- PASS
`DEFAULT_SETTINGS_TOML` (defaults.rs) lists, in order: `C`=Config, `A`=Aliases,
`T`=Theme, `M`=Menus, `K`=Keys, `W`=Kinds (group `Core`), then `R`=Reset bare
(group `Recovery`). Asserted exactly by
`default_settings_toml_has_alias_editor_and_config_mnemonic_c` (pair-wise
key/command ordered vector) and by the updated
`recovery_settings_menu_has_barebones_options` /
`default_settings_toml_has_recovery_baseline_options` (keys, commands, and the
`Core*6 + Recovery` group vector). Tests observed PASS in the run log.

### 2. A reuses the EXISTING single front door for the Alias Editor -- PASS
The new row carries `command = "Aliases"`. Menu option selection dispatches the
option's `command` string (not its key) through the single front door. I
confirmed `command_config/mod.rs::builtin_workspace_target_for` classifies
`ALIASES` (case-insensitive) as `custom("aliases")`, and
`shell/target_dispatch.rs` arm `"aliases" => self.open_alias_editor()` ->
`navigate_to` (shell/alias_editor.rs). This is the CR-NR-113 path verbatim; no
new verb, dispatcher, or Context is added. The `Aliases` Title-case spelling
mirrors the sibling options `Config`/`Theme`/`Menus`/`Keys`/`Kinds`, which all
resolve through the same case-insensitive classifier -- consistent encoding.
`settings_alias_option_dispatches_aliases_verb` asserts selecting `A` opens
`KindTag::AliasEditor` with title `[ALIASES]` via `handle_command`.

### 3. C opens Config; mnemonic remap correct everywhere it is handled -- PASS
I searched the production tree for any hardcoded match on the Settings mnemonic
key character (`== "A"` / `== "C"` / `eq_ignore_ascii_case("A"|"C")`). The only
hits are in test code (`tests_nav.rs`). Production dispatch keys off the option
`command` string, and `CONFIG` -> `custom("config")` is classified by verb, not
by mnemonic. Therefore the A->C remap is fully expressed by the single TOML
constant; there is no second hardcoded mnemonic site to update.
`settings_config_option_dispatches_config` asserts selecting `C` opens
`KindTag::ConfigPanel`. The pre-existing `settings_option_a_opens_flat_panel`
dispatches `CONFIG` directly (not via a key) and is unaffected.

### 4. framework-conformance: content-only, code-only built-in, single dispatch -- PASS
Only `DEFAULT_SETTINGS_TOML` (a compiled constant) changes; `ensure_menus_dir`
still never writes `settings.toml`, and
`ensure_menus_dir_does_not_materialise_built_in_menus` (PASS in the run log)
guards CR-CH-021. No new dispatch path is introduced -- the existing
`resolve_target` -> `builtin_workspace_target_for` -> `dispatch_command_target`
chain is reused. Not a framework change.

### 5. Acceptance criteria covered by annotated tests -- PASS
New menu-workspace Requirement 22 (22.1-22.5) is added to `requirements.md`.
Tests carry `// Validates: ... Requirement 22.x` annotations:
- 22.1/22.5: `default_settings_toml_has_alias_editor_and_config_mnemonic_c`,
  `recovery_settings_menu_has_barebones_options`,
  `default_settings_toml_has_recovery_baseline_options` (exact order + mnemonics;
  observed PASS).
- 22.2: `settings_alias_option_dispatches_aliases_verb` (A opens Alias Editor).
- 22.3: `settings_config_option_dispatches_config` (C opens Config).
- 22.4: `ensure_menus_dir_does_not_materialise_built_in_menus` (not written to
  disk; observed PASS).
The option-list test asserts the exact ordered key/command vectors; activation
tests cover both A and C.

### 6. rust-standards -- PASS
`defaults.rs` `#[cfg(test)]` begins at line 195, so production code is ~194
non-test lines (<400); it is the only production file touched. No
`unwrap()`/`expect()` added in non-test code (the two `.expect()` in
`recovery_*_menu()` are pre-existing programmer-error guards, unchanged by this
CR). `default_settings_toml_ascii_only` asserts the constant is ASCII; a
scan for non-ASCII in the file found none.

## Findings

None. No blocking correctness, security, data, or conformance issue found. The
change is a precise content edit that reuses the existing single-dispatch path,
keeps the built-in code-only, and is covered by annotated tests whose core
assertions were observed passing; remaining full-suite verification is correctly
deferred to the owner's gate per the workspace execution split.

## Verdict

APPROVED.
