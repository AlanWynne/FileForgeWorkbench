# Write-through config model and ISPF END/RETURN/EXIT (CR-CH-116)
# Write-through config model and ISPF END/RETURN/EXIT (CR-CH-116)

Four FEAT commits implement the owner's write-through config model and the ISPF navigation verbs. The built-in menus (POM, Settings, SCRM) and the FFCMD `X -> EXIT` alias seed become write-through config: a single shared helper (`write_builtin_defaults`) writes COMPLETE config files on first run and run-from-disk thereafter, and Reset bare archives the old files and rebuilds through the same helper. Separately, `X` is retired as a navigation verb and becomes the FFCMD seed alias of `EXIT`, terminating the app on every non-editor Context; the old `nav_x` collapse/close/exit-when-last verb is deleted, END (PF3) still pops one level, and RETURN is reclassified (behaviour unchanged) to unwind to the tab's primary panel.

Watch for: the FFEDIT `X -> EXCLUDE` protection is NOT the claim-gate the commit messages imply (that gate is inert in this core-strip build) -- it works via the per-environment front-door alias rewrite instead. The outcome is correct and tested, but the stated mechanism is imprecise (confirmed). The displayed command column for the POM/SCRM X option renders `X`, not `EXIT`, which is the intended alias-surface display and not a bug (confirmed).

**Verdict**: APPROVED

## High-level view

The write engine is a single pure free function, `write_builtin_defaults(menus_dir, alias_base, locale, mode)`, with a two-variant `BootstrapMode`. `FirstRunMissingOnly` writes each file only when absent; `RebuildOverwrite` always writes. First-run and Reset bare both call this one function, so their output cannot diverge -- exactly the owner's "one helper" requirement. All paths arrive as arguments, so the three dir seams (`dir_overrides.menus`, the alias base via `FFWB_USER_ALIASES_DIR`, and the SCRM-under-menus path) are honoured and the function is `TempDir`-unit-testable.

The first-run hook runs during shell construction, resolves the seams, skips in Degraded_Mode, is non-blocking on error, and leaves a present-but-corrupt file untouched (write-if-absent is path existence, not parse success). Construction then refreshes the live alias store so `X -> EXIT` goes live on the first launch, not only after a restart. A thread-local test seam isolates the construction-time write to a temp dir without racing parallel tests.

`X = EXIT` everywhere is achieved by writing an editable `X -> EXIT` user-override row into the FFCMD alias file and letting the front door's `rewrite_verb_to_root` rewrite `X -> EXIT` before the prelude dispatches `file.exit`. On an editor Context the active environment is FFEDIT, so the same front-door rewrite resolves `X` against the FFEDIT table to `EXCLUDE` instead -- that is what keeps the editor's `X = EXCLUDE` binding intact. The `nav_x` function is deleted with no orphan caller; `close_workspace_or_exit` and `nav_collapse_to_visual_root` are retained because END and RETURN still use them.

Reset bare adds `aliases` to the archived set and, after archiving each targeted profile, rebuilds the menus and the seed via the shared helper in `RebuildOverwrite` mode, refreshing the active profile's live store so a prior user alias (SS -> SNAPSHOT) is gone and `X -> EXIT` is restored without a restart. The Menus editor lists `[POM, Settings, SCRM]` (de-duplicating an on-disk `scrm.toml`) and loads SCRM from `recovery_scrm_menu()` rather than the POM catch-all.

The `config_version` stamp is additive and inert: a `pub(crate)` constant emitted by the complete-mode serialiser and a crate-private `#[serde(default)] Option<u32>` on the loader that is ignored on load. No UPGRADE/merge is implemented, matching the owner's "worry about upgrades later" direction. The change is confined to `crates/ff-desktop` internals and `tools/`; no public cross-crate type changes.

<details>
<summary>Issues (3)</summary>

1. **FFEDIT protection mechanism mis-described** — the commit messages and some ladder comments say bare `X` on an editor is protected by the FFEDIT active-wins claim gate, but `EnvironmentRegistry::is_ffedit_active` returns `false` unconditionally in this core-strip build, so that gate never fires. The actual protection is the front-door `rewrite_verb_to_root` resolving `X` under the FFEDIT table to `EXCLUDE`. Behaviour and tests are correct; consider a comment/commit-message correction so a future reader does not rely on the inert gate.
2. **POM/SCRM command column renders `X`** — the rendered menu and markdown golden show `| X | X |` for the exit option whose command is `EXIT`. This is the intended alias-surface display (`display_surface_for_canonical(EXIT) -> X`), not a stale golden. No action needed; noted so it is not mistaken for a regression.
3. **Reset-bare rebuild errors are reported but the archive already happened** — if `write_builtin_defaults` fails during Reset bare, the old files are already archived, so a rebuild failure leaves the active profile running on the compiled Recovery_Source until next launch. This is the intended graceful-degradation posture and the error is surfaced; flagged only for awareness.

</details>

<details>
<summary>Details</summary>

### One shared write engine, two modes

`write_builtin_defaults` is a pure free function in the new `config_bootstrap.rs` (399 lines incl. tests, under the 400 non-test limit). It writes `pom.toml`, `settings.toml`, and `scrm.toml` under the passed `menus_dir` via `serialise_complete`, plus the single `X -> EXIT` row to `<alias_base>/<locale>/FFCMD.toml` via the existing `user_aliases::serialise_user_aliases`. The `FirstRunMissingOnly` / `RebuildOverwrite` split is enforced in the per-file `write_menu` / `write_alias_seed` helpers by a single `mode == FirstRunMissingOnly && path.exists()` early-return, so "write if absent" vs "always write" is the only behavioural difference. Because both the first-run hook and Reset bare call this one function, the owner's "same reset-bare and first-run output" requirement is structurally guaranteed, not merely tested.

The SCRM-under-menus detail is handled correctly and tested: `scrm.toml` lands under `menus_dir` (the SCRM open path and Menus editor read `menus_dir()/scrm.toml`), not the unrelated `scrm_dir()` screen-collections directory. The FEAT-002 commit notes this was corrected mid-task (a mistaken separate `scrm_dir` param was dropped).

### First-run hook, Degraded_Mode, and store refresh

`first_run_write_builtin_config` is invoked once from `construct.rs` after the shell is built. It skips entirely when no menus override is set, no test seam is set, and `UserDataDir::resolve` fails (Degraded_Mode), logging a WARN and falling back to the compiled Recovery_Source. A present-but-corrupt file is left alone because the write-if-absent guard keys on `path.exists()`, not parse success -- so a user's broken file is never clobbered, and recovery happens at load time. Any write error is logged and non-blocking.

The construction sequence writes first, then calls `refresh_alias_store_in_place()`, because the alias store was seeded before the first-run write and would otherwise miss the freshly-written `X -> EXIT` seed until a restart. This is the kind of ordering bug that is easy to miss; the commit calls it out and the `x_to_exit_seed_row_is_visible_in_the_alias_editor` test covers the first-launch path.

Test isolation is preserved via a thread-local `TEST_FIRST_RUN_MENUS_DIR` seam set before construction (the per-shell `dir_overrides.menus` is set after construction, too late for the construction-time write). `make_shell` / `make_shell_with_history_path` set it to the process-global isolated menus dir, and `make_shell_first_run` sets a per-test temp dir. Thread-local (not a process global) is the right choice so parallel `cargo test` runs cannot clobber each other between set and read.

### X = EXIT and the FFEDIT boundary

The `X -> EXIT` binding is a real editable user-override row (same `UserAliasRowToml` type the Alias Editor reads and writes), written to the FFCMD environment file. At runtime the front door's `rewrite_verb_to_root(raw)` rewrites the verb against the ACTIVE environment's alias table before the prelude runs. On a non-editor Context the active environment is FFCMD, so `X -> EXIT` and `file.exit` terminates the app; `=X` reinitialises to the POM then runs `X -> EXIT`.

On an editor Context, `active_name(Editor) -> FFEDIT`, so `rewrite_verb_to_root("X")` resolves `X` under the FFEDIT english table to `EXCLUDE`, never reaching the FFCMD `X -> EXIT` seed. This is the genuine reason the editor's `X = EXCLUDE` survives. The commit messages and the `commands.rs` / `commands_ladder_a.rs` comments instead attribute the protection to the FFEDIT active-wins claim gate (`is_ffedit_active`), but that predicate is hardwired to `false` in this core-strip build and the gate never fires. The end state is correct and the editor tests (`editor_bare_x_excludes_does_not_exit`, `ffedit_bare_x_stays_exclude_but_equals_x_exits`) pass through the rewrite path, so this is a documentation/comment imprecision rather than a behavioural defect -- but a future reader wiring the editor back up should not assume the gate is doing the work.

`nav_x` is deleted outright (replaced by a comment block in `nav_stack.rs`) and the FEAT-004 commit records a reference search confirming no remaining caller. `close_workspace_or_exit` and `nav_collapse_to_visual_root` are retained and now used by END / RETURN. `nav_return` keeps its CR-CH-052 behaviour (collapse to `nav_stack[0]`) and is only re-documented as the ISPF "back out to primary panel" semantics -- the diff confirms the body is unchanged, so the "behaviour unchanged, reclassified" claim holds.

### Reset bare: archive aliases, rebuild through the shared helper

`ARCHIVED_ITEMS` gains `aliases`, and `archive_config_moves_aliases_dir` proves a live `aliases/` is moved into the archive non-destructively. After each successful archive, `execute_reset_bare` rebuilds via `write_builtin_defaults(.., RebuildOverwrite)`. The active profile uses the shell's seams (`menus_dir()` / `user_alias_override_base()`) so a test override and the live reader agree; a non-active targeted profile builds `<udd>/menus` and `<udd>/aliases` using the loaders' layout. The active profile's in-memory state is reset and the alias store refreshed, so `reset_bare_active_profile_rebuilds_menus_and_x_exit_seed` can assert SS -> SNAPSHOT is gone and X -> EXIT is restored on disk and live, with pom/settings/scrm all rebuilt.

If the rebuild write fails, the archive has already occurred, so the profile falls back to the compiled Recovery_Source for that session. The error is pushed into the reported `errors` list, consistent with the best-effort posture; this is intended graceful degradation.

### Menus editor: SCRM is a first-class built-in

`refresh_menus_editor_list` now seeds `[POM, Settings, SCRM]` and de-duplicates an on-disk `scrm.toml`, `menu_file_stem("SCRM") -> "scrm"`, and `load_menu_for_editor` maps SCRM to `recovery_scrm_menu()` rather than the POM catch-all. The former behaviour silently loaded the POM when SCRM was selected; `menus_editor_selector_lists_scrm` and `menus_editor_selecting_scrm_loads_scrm_menu_not_pom` lock the fix.

### config_version is additive and inert

`serialise_complete` emits every flag explicitly (title, show_calendar, group_separator, group_headers, per-option enabled/show_in_menu_bar) plus `config_version = 1`, and round-trips through `parse_menu_str`. The complete serialiser is a sibling of the existing clean `serialise`, which is unchanged (and the tests assert the clean path still omits defaults). The loader gains a crate-private `#[serde(default)] #[allow(dead_code)] config_version: Option<u32>` that is parsed and ignored. No UPGRADE/merge logic exists, matching the owner's deferral. The stamp exists only so a stamped file parses cleanly and to leave room for the deferred command.

### Test isolation inversions

The former CR-CH-021 "never materialise" invariant is correctly reversed. `ensure_menus_dir_does_not_materialise_built_in_menus` is renamed to `ensure_menus_dir_creates_dir_but_writes_no_menu_file_itself`: it still asserts `ensure_menus_dir` alone writes no menu file (the first-run hook is the writer), which is the right narrow invariant to keep. The write-through behaviour is proven separately in `tests_first_run.rs` (`make_shell_materialises_builtin_menus_on_first_run`). Alias-editor tests that repoint the override base now call `refresh_alias_store_in_place()` so they reflect the empty repointed base rather than the shared base's seed -- a necessary consequence of the first-run write seeding the shared base.

### Validation annotations and scope

Each FEAT's tests carry `// Validates:` with specific Req ids: FEAT-001 configuration-system Req 23.4/23.9; FEAT-002 startup-and-session Req 23.1 + configuration-system Req 23.2/23.7; FEAT-003 configuration-system Req 19.16/19.17/19.18 + command-configurator Req 8.4; FEAT-004 navigation-commands Req 21.x + menu-workspace Req 14.x + command-configurator Req 8.5/8.6. The diff touches only `crates/ff-desktop/**` and `tools/powershell/update-goldens.ps1`; no other crate and no public cross-crate type is modified, so there is no framework or public-API change. New source files are ASCII-clean; the box-drawing characters are confined to the golden fixtures, which legitimately contain them.

Per instruction, the coder's recorded per-FEAT verification (scoped check/clippy/fmt/tests) was taken as given and not re-run.

</details>

<details>
<summary>File map</summary>

- `menu_workspace/serialiser.rs` -- new `serialise_complete` (every flag explicit + `config_version`) and `CONFIG_VERSION` constant.
- `menu_workspace/loader.rs` -- crate-private additive `config_version: Option<u32>`, ignored on load.
- `menu_workspace/defaults.rs` -- POM/SCRM X option command `EXIT`; obsolete "never materialised" test inverted.
- `shell/config_bootstrap.rs` -- new: `BootstrapMode`, the shared `write_builtin_defaults` engine, the first-run hook, the thread-local test seam.
- `shell/construct.rs` -- call the first-run hook then refresh the alias store.
- `shell/reset_bare.rs` -- archive `aliases`, rebuild via the shared helper in `RebuildOverwrite`, profile-active helpers.
- `shell/nav_stack.rs` -- delete `nav_x`; re-document `nav_return` as ISPF RETURN; retain `close_workspace_or_exit` / `nav_collapse_to_visual_root`.
- `shell/commands_ladder_a.rs` -- remove the `X` ladder arm; update exit-family / RETURN docs.
- `shell/menus_editor.rs` -- SCRM in the built-in list; SCRM loads `recovery_scrm_menu()`; `menu_file_stem("SCRM")`.
- `shell/mod.rs` -- register `config_bootstrap` and `tests_first_run`.
- `shell/tests_*.rs` -- X=EXIT rewrites, first-run tests, SCRM editor tests, alias-seed visibility, nav-ladder rewrite.
- `shell/golden/*` -- regenerated POM/SCRM goldens (X-row text only).
- `tools/powershell/update-goldens.ps1` -- new golden-regen helper.

Full diff: `git -C .worktrees/config-model-impl diff main...HEAD`.

</details>
