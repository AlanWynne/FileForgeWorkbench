# Implementation Plan -- CR-CH-116 write-through config model + ISPF END/RETURN/EXIT (X = EXIT)

Owner-confirmed, gated framework change. Build ON existing seams (single dispatch
front door, per-tab Navigation_Stack, three-layer alias resolver, ScreenProvider).
NO new dispatcher, NO new navigation stack. All work in the worktree
`c:/workspace/apps/FileForgeWorkbench/.worktrees/config-model-impl`. TDD-first:
each test carries `// Validates: <Req id>`. Agents compile-verify SCOPED
(`cargo check/clippy/fmt -p ff-desktop`, quick scoped tests); the owner runs the
full `cargo gate`. ASCII-only source/docs. No source file over 400 non-test lines
(split when a touched file would exceed it).

This plan is authored as four FEAT artifacts under
`c:/workspace/apps/FileForgeWorkbench/.agents/tasks/config-model-impl/` (task.json,
context.json, features/FEAT-001..004.json) and the workflow tail is restructured
into one coder step per FEAT plus a fix/review convergence loop. This file is the
equivalent single-list fallback if the restructure did not take effect.

---

- [ ] 1. Config_Rebuild_Helper + COMPLETE-mode menu serialiser + config_version stamp (FEAT-001).
      Add `crates/ff-desktop/src/shell/config_bootstrap.rs` with a crate-private
      `BootstrapMode { FirstRunMissingOnly, RebuildOverwrite }` and a free function
      `write_builtin_defaults(menus_dir, scrm_dir, alias_base, locale, mode) -> ...`
      that is the ONLY code serialising the Recovery_Source into COMPLETE config
      files: `pom.toml`, `settings.toml` (via `menus_dir`), `scrm.toml` (via
      `scrm_dir`) each a Complete_Config_File, plus `aliases/<locale>/FFCMD.toml`
      containing the single `X -> EXIT` row (via `alias_base`). Pass the three
      resolved paths in (do NOT call `self.menus_dir()` inside, so the helper is
      pure + unit-testable against TempDirs and both hooks share it). In
      `FirstRunMissingOnly` write only a file that is ABSENT (per-file); in
      `RebuildOverwrite` always write. Add `serialise_complete(menu)` to
      `menu_workspace/serialiser.rs` emitting EVERY flag explicitly
      (`show_calendar`, `group_separator`, `group_headers`, each option `enabled`,
      `show_in_menu_bar`, `group`) and a top-level `config_version` stamp; add an
      optional defaulted `config_version` to the loader's crate-private
      `RawMenuFile` (unused, backward compatible). The FFCMD seed is written with
      `user_aliases::serialise_user_aliases(&[X->EXIT row])` to
      `user_aliases::user_alias_override_file(base, locale, "FFCMD")`.
      Register `mod config_bootstrap;` in `shell/mod.rs`.
      Files: crates/ff-desktop/src/shell/config_bootstrap.rs (new),
      crates/ff-desktop/src/menu_workspace/serialiser.rs,
      crates/ff-desktop/src/menu_workspace/loader.rs,
      crates/ff-desktop/src/shell/mod.rs
      Verify: `cargo test -p ff-desktop config_bootstrap` and
      `cargo test -p ff-desktop serialiser` pass; `cargo clippy -p ff-desktop -- -D warnings`
      clean. New tests: FirstRunMissingOnly writes only absent files; RebuildOverwrite
      replaces; both write through the passed paths; written menu files are COMPLETE
      with `show_calendar` explicit (POM true, Settings false); `aliases/en/FFCMD.toml`
      contains `X -> EXIT`; `serialise_complete` round-trips through `parse_menu_str`;
      the `config_version` stamp loads and is ignored.

- [ ] 2. First-run write hook in construct.rs + test isolation + degraded/parse-fallback (FEAT-002).
      In `shell/construct.rs::new_with_history_store`, AFTER `self` is fully built
      and BEFORE returning, call a new `pub(super) fn first_run_write_builtin_config(&mut self)`
      (new method, e.g. in `config_bootstrap.rs` as an `impl WorkbenchShell`) that:
      resolves `self.menus_dir()`, `self.scrm_dir()`, `self.user_alias_override_base()`
      and the active `ui.locale` (default `en`); SKIPS entirely in Degraded_Mode
      (User_Data_Dir not writable) falling back to the Recovery_Source; otherwise
      calls `write_builtin_defaults(.., FirstRunMissingOnly)`. Per-file detection is
      the helper's FirstRunMissingOnly mode. A file present but UNPARSEABLE must NOT
      be treated as absent and must NOT be overwritten (the helper only writes when
      `!path.exists()`, so a corrupt-but-present file is left alone; the existing
      `ensure_pom_menu_loaded` / `open_menu_by_name` parse-failure notice path is
      retained unchanged). Test isolation: because `make_shell`/`harness_shell` set
      `dir_overrides.menus` AFTER construction today, move the menus/scrm isolation
      to BEFORE the write -- set `dir_overrides.menus` and `dir_overrides.scrm` to
      the process-global isolated temp dirs (mirror `isolated_menus_dir()`) INSIDE
      `new_with_history_store` guarded by a test-only seam, OR restructure
      `make_shell`/`harness_shell` to inject them through the constructor before the
      first-run write runs. The alias base is already isolated at construction via
      `FFWB_USER_ALIASES_DIR` (`isolate_user_alias_base_once`).
      Files: crates/ff-desktop/src/shell/construct.rs,
      crates/ff-desktop/src/shell/config_bootstrap.rs,
      crates/ff-desktop/src/shell/tests_common.rs
      Verify: `cargo test -p ff-desktop first_run` plus the full `ff-desktop` shell
      suite under the harness compile and the new tests pass; `cargo clippy` clean.
      New tests: empty overridden user dir -> construct writes pom/settings/scrm +
      aliases/en/FFCMD.toml then loads them; a pre-existing user pom.toml is NOT
      overwritten; a corrupt present file -> recovery fallback + notice, NOT
      overwritten, NOT treated as absent; Degraded_Mode skips the write and falls
      back to Recovery_Source.

- [ ] 3. Reset bare: archive aliases + rebuild menus and FFCMD seed via the shared helper (FEAT-003).
      In `shell/reset_bare.rs` add `"aliases"` to `ARCHIVED_ITEMS`. In
      `execute_reset_bare`, AFTER `archive_config(udd)` succeeds for a profile, call
      `write_builtin_defaults(menus_dir, scrm_dir, alias_base, locale, RebuildOverwrite)`
      resolving the SAME seams used elsewhere for that profile's dir (for the active
      profile use `self.menus_dir()/.scrm_dir()/.user_alias_override_base()`; for a
      non-active targeted profile, build the paths under that profile's udd through
      the same path rules). The in-memory reset (`reset_in_memory_to_baseline`) is
      unchanged except that the reopened POM and the reloaded alias store now read
      the just-rebuilt files; call `self.refresh_alias_store_in_place()` after the
      rebuild so `X -> EXIT` is live. Keep the non-destructive archive-move.
      Files: crates/ff-desktop/src/shell/reset_bare.rs
      Verify: `cargo test -p ff-desktop reset_bare` passes; `cargo clippy` clean.
      New/updated tests: `ARCHIVED_ITEMS` now includes `aliases`; a user alias
      `SS -> SNAPSHOT` present at reset is archived/gone AND `X -> EXIT` is restored
      in `aliases/<locale>/FFCMD.toml`; menus pom/settings/scrm are rebuilt on disk;
      the two existing archive tests that enumerate `ARCHIVED_ITEMS` are updated.

- [ ] 4. ISPF END/RETURN/EXIT + X = EXIT everywhere + nav_x retirement + POM/SCRM X option + menus-editor SCRM fix + goldens (FEAT-004).
      Remove the `upper == "X"` arm in `commands_ladder_a.rs::try_commands_a` (the
      one calling `nav_x`). `X` now resolves on non-editor Contexts via the FFCMD
      seed `X -> EXIT`: `dispatch_command_string` runs `rewrite_verb_to_root("X")`
      against the active (FFCMD) table -> `"EXIT"`, which the prelude's
      `try_exit_family` dispatches to `file.exit`. Active-wins keeps FFEDIT bare `X`
      = EXCLUDE (the ladder active-env claim + the prelude EXIT-family skip when an
      editor env is active, both unchanged). `=X` reinitialises to POM then runs `X`
      -> EXIT. Retire `shell/nav_stack.rs::nav_x` and delete it after a reference
      search confirms no other caller; RETAIN `close_workspace_or_exit` (END/RETURN
      use it) and `nav_collapse_to_visual_root` (RETURN uses it). RETURN
      (`nav_return`) is UNCHANGED in behaviour (already collapses to Tab_Visual_Root);
      update its doc comment to the ISPF RETURN classification and remove the stale
      "converges with bare X" note. END (`nav_end`, PF3) UNCHANGED. EXIT
      (`try_exit_family`) UNCHANGED. Change `DEFAULT_POM_TOML` and
      `DEFAULT_SCRM_MENU_TOML` X-option `command` from `"X"` to `"EXIT"` (keep key
      `X`, keep `show_in_menu_bar = false`, update the description to drop the
      self-reference). Fix `menus_editor.rs`: `refresh_menus_editor_list` built-in
      set becomes `[POM, Settings, SCRM]` unioned with on-disk `*.toml`; the
      `load_menu_for_editor` fallback match maps `"SCRM" -> recovery_scrm_menu()`
      (not the `_ => recovery_pom_menu()` catch-all). Update inverted tests in
      `defaults.rs` (`ensure_menus_dir_does_not_materialise_built_in_menus` is now
      obsolete / asserts the opposite behaviour belongs to the first-run path;
      `default_pom_toml_terminate_is_x_x` and the `recovery_*` command-list asserts
      now expect `EXIT`). Regenerate goldens that legitimately change (POM/SCRM X
      option now dispatches EXIT; display may show `X EXIT`) via `FFWB_UPDATE_GOLDEN`,
      then review by hand.
      Files: crates/ff-desktop/src/shell/commands_ladder_a.rs,
      crates/ff-desktop/src/shell/nav_stack.rs,
      crates/ff-desktop/src/menu_workspace/defaults.rs,
      crates/ff-desktop/src/shell/menus_editor.rs,
      crates/ff-desktop/src/shell/golden/*.txt (regen as needed),
      and any shell test module asserting the old X/nav_x behaviour.
      Verify: `cargo test -p ff-desktop` X / END / RETURN / EXIT / menus_editor /
      defaults / golden tests pass; `cargo clippy -p ff-desktop -- -D warnings` clean;
      a reference search shows zero remaining `nav_x` callers.
      New/updated tests: bare `X` on a non-editor Context exits the app via the seed;
      `X` on an editor still EXCLUDEs; `=X` exits; POM/SCRM `X` option dispatches
      EXIT; PF3/END still pops one level; RETURN still collapses to the primary
      panel; EXIT unconditional; menus-editor selector lists SCRM and selecting SCRM
      loads the SCRM menu (not POM); `nav_x` removed with no caller.

## Digest

Dependency order: FEAT-001 (helper + serialiser) first; FEAT-002 and FEAT-003 both
depend on 001; FEAT-004 depends on 001 (POM/SCRM default TOML + seed resolution).
Run 001 -> 002 -> 003 -> 004.

### crates/ff-desktop/src/shell/config_bootstrap.rs (NEW)
- Add `pub(super) enum BootstrapMode { FirstRunMissingOnly, RebuildOverwrite }`.
- Add `pub(super) fn write_builtin_defaults(menus_dir: &Path, scrm_dir: &Path,
  alias_base: &Path, locale: &str, mode: BootstrapMode) -> Result<(), String>`:
  serialises `recovery_pom_menu()`/`recovery_settings_menu()` to
  `menus_dir/{pom,settings}.toml`, `recovery_scrm_menu()` to `scrm_dir/scrm.toml`
  (all via `serialise_complete`), and the `X -> EXIT` row via
  `user_aliases::serialise_user_aliases` to
  `user_aliases::user_alias_override_file(alias_base, locale, FFCMD_NAME)`;
  per-file `!path.exists()` guard in FirstRunMissingOnly.
- Add `impl WorkbenchShell { pub(super) fn first_run_write_builtin_config(&mut self) }`
  resolving `self.menus_dir()/.scrm_dir()/.user_alias_override_base()` + `active_locale`,
  Degraded_Mode skip, then `write_builtin_defaults(.., FirstRunMissingOnly)`.
- Tests: the helper modes, completeness, seam routing, seed content.
- Covered by: `cargo test -p ff-desktop config_bootstrap`.

### crates/ff-desktop/src/menu_workspace/serialiser.rs
- Current `pub fn serialise(menu) -> String` OMITS default-valued fields (lines
  ~22-48). ADD `pub fn serialise_complete(menu: &MenuFile) -> String` emitting
  every flag + `config_version`. Do NOT change `serialise` (the Menus-editor save
  keeps clean output). Existing round-trip tests stay green; add a
  `serialise_complete` round-trip + explicit-flag test.
- Covered by: `cargo test -p ff-desktop serialiser`.

### crates/ff-desktop/src/menu_workspace/loader.rs
- `struct RawMenuFile` (lines ~47-63): add
  `#[serde(default)] config_version: Option<u32>` (unused; keeps unknown-key
  tolerance for older files). `parse_menu_str`/`load_menu_file` unchanged otherwise.
- Covered by: existing loader tests + the serialise_complete round-trip.

### crates/ff-desktop/src/shell/construct.rs
- `new_with_history_store` (ends ~line 300, the `Self { .. }` literal): capture the
  built shell into `let mut shell = Self { .. };`, call
  `shell.first_run_write_builtin_config();`, then `shell`. (Or invoke after `new`
  delegates.) Degraded_Mode + seam resolution live in the method.
- Covered by: new first-run tests in the shell suite.

### crates/ff-desktop/src/shell/tests_common.rs
- `make_shell()` (sets `shell.dir_overrides.menus` AFTER construction, ~line 240)
  and `harness_shell()`: ensure menus/scrm isolation is in effect BEFORE the
  first-run write runs during construction. Mirror `isolated_menus_dir()` for scrm;
  the alias base is already isolated via `isolate_user_alias_base_once()`.
- Covered by: the whole shell suite still compiling + first-run tests isolated.

### crates/ff-desktop/src/shell/reset_bare.rs
- `const ARCHIVED_ITEMS` (lines 15-26): add `"aliases"`.
- `execute_reset_bare` (~line 268): after `archive_config(udd)` Ok, call
  `write_builtin_defaults(.., RebuildOverwrite)` for that profile's seams;
  `reset_in_memory_to_baseline` then reopens POM reading the rebuilt files; add a
  `refresh_alias_store_in_place()` call for the active profile.
- Tests: `archived_items_includes_workspace_kinds` pattern -> add aliases assertion;
  new reset-rebuild tests (SS archived/gone, X->EXIT restored, menus rebuilt).
- Covered by: `cargo test -p ff-desktop reset_bare`.

### crates/ff-desktop/src/shell/commands_ladder_a.rs
- Remove the `if upper == "X" { self.nav_x(); .. return true; }` arm (~lines 315-326).
  `try_exit_family` and the END/RETURN arms stay. Update the module doc note that
  previously said `X` is the close verb.
- Covered by: X/END/RETURN/EXIT dispatch tests.

### crates/ff-desktop/src/shell/nav_stack.rs
- Delete `pub(super) fn nav_x(&mut self)` (~lines 205-215) after confirming no other
  caller. KEEP `nav_end`, `nav_return`, `nav_collapse_to_visual_root`,
  `close_workspace_or_exit`, `reinitialise_active_tab_to_pom`. Update `nav_return`'s
  doc to the ISPF RETURN classification; drop the "converges with bare X" note.
- Covered by: END/RETURN tests + a no-caller reference search.

### crates/ff-desktop/src/menu_workspace/defaults.rs
- `DEFAULT_POM_TOML` X option: `command = "EXIT"` (was `"X"`), description drops the
  self-reference. `DEFAULT_SCRM_MENU_TOML` X option: `command = "EXIT"` (was `"X"`).
- Update tests: `default_pom_toml_terminate_is_x_x`,
  `recovery_pom_menu_has_barebones_options`, `bare_pom_contains_scrm_and_stays_minimal`,
  `default_pom_toml_has_recovery_baseline_options`,
  `recovery_scrm_menu_has_owner_confirmed_keys_and_verbs` (expect `EXIT`), and retire/
  invert `ensure_menus_dir_does_not_materialise_built_in_menus` (first-run now writes;
  this file's `ensure_menus_dir` still only creates the dir, so keep that part and
  drop the "scrm/pom/settings never materialised" assertion that the model reverses).
- Covered by: `cargo test -p ff-desktop defaults`.

### crates/ff-desktop/src/shell/menus_editor.rs
- `refresh_menus_editor_list` (~line 184): seed list `["POM","Settings","SCRM"]`
  (dedup SCRM against on-disk scan like pom/settings). `load_menu_for_editor`
  (~line 166): change the fallback `match` to add `"SCRM" => recovery_scrm_menu()`.
- `menu_file_stem` already slugs; SCRM -> `scrm` needs an explicit arm or relies on
  the lowercase slug (`scrm`); confirm it maps to `scrm.toml`.
- Tests: selector lists SCRM; selecting SCRM loads the SCRM menu not POM.
- Covered by: `cargo test -p ff-desktop menus_editor`.

### crates/ff-desktop/src/shell/golden/*.txt and golden harness
- Regenerate POM/SCRM goldens where the X option now reads/dispatches `EXIT`
  (`FFWB_UPDATE_GOLDEN=1 cargo test -p ff-desktop <golden test>`), then review the
  diff by hand (only the X row text should change).
- Covered by: `cargo test -p ff-desktop` golden modules.

### Verified during exploration
- Build/test commands: SCOPED `cargo check/clippy/fmt -p ff-desktop` and
  `cargo test -p ff-desktop <focused>` per gate-policy; full `cargo gate` is the
  owner's manual run (testing.md execution split). The worktree is on branch
  `config-model-impl`, clean, with the CR-CH-116 specs already merged in.
- X resolution path confirmed: `dispatch_command_string` -> `rewrite_verb_to_root`
  (active env table) -> prelude `try_exit_family`. FFEDIT `X -> EXCLUDE` preserved
  by the active-wins ladder claim + the prelude EXIT-family skip when an editor env
  is active. `=X` strips `=`, reinits to POM (FFCMD), then `X -> EXIT`.
- Seams confirmed: `menus_dir()` (`dir_overrides.menus`), `scrm_dir()`
  (`dir_overrides.scrm`), `user_alias_override_base()` (`dir_overrides.user_aliases`
  / `FFWB_USER_ALIASES_DIR`). Alias seed path `user_alias_override_file(base,
  locale, "FFCMD")`, serialiser `serialise_user_aliases`. `builtin_ffcmd_roots`
  already includes `EXIT` (an FFCMD ladder verb root) so `X -> EXIT` targets a real
  root -- NO change needed there; confirm by reading the FFCMD root set.
- Test-isolation RISK (primary): `make_shell`/`harness_shell` set
  `dir_overrides.menus` AFTER construction, but the first-run write runs DURING
  construction. The write would otherwise hit the real User_Data_Dir. FEAT-002 must
  resolve this (isolate menus/scrm before the write) or the shell suite's "code-only
  baseline" determinism and the new first-run tests both break. This is called out
  as the FEAT-002 blocking design point.
- No public cross-crate type is reshaped; the only data-shape change is the
  crate-private `config_version` on `RawMenuFile` + the serialised files. If any
  other reshape appears necessary, STOP and surface it (per the brief).
