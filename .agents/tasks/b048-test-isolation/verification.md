# B048 Test-Isolation -- Verification Note

Status: CODE COMPLETE, NOT VERIFIED (shell blocker -- see below).

## BLOCKER (read first)

Throughout this entire implementation session the Kiro integrated shell was
non-functional: every command (including a trivial `Write-Output`, a
`Set-Content`, a `cmd /c echo > file`, and the pwsh7 child-process wrapper)
returned ONLY the echoed command line plus `Exit Code: -1`, with:

- NO stdout captured (empty `Output:` block every time), AND
- NO file side effects (every probe file written to `.gate\probe*.txt` was
  absent when read back with the file-reading tool).

Both symptoms together mean this was NOT the benign phantom `-1` (which coexists
with correct output and a `rc=0` sentinel) and NOT a short "dead episode" that
clears on its own -- the commands genuinely did not execute, and the condition
persisted across the whole session despite repeated probes with waits in
between. I therefore could NOT run cargo fmt / clippy / build / test / nextest,
and could NOT produce the `KIRO_COMMAND_FINISHED rc=` evidence lines this note is
supposed to contain.

The reviewer (or a later run once the shell recovers) MUST run the commands in
the "Commands to run" section below and read the `.gate\*.log` files back to
confirm green before this task is considered done.

## Files changed

All changes are ASCII-only. Production runtime behaviour is unchanged: every new
code path is additive and guarded by an explicit injected `Some(path)`; when the
option is `None` (the production default) behaviour is byte-for-byte as before.

1. `crates/ff-config/src/init/mod.rs`
   - Added `pub user_config_path: Option<PathBuf>` field to `ConfigInitOptions`,
     defaulted to `None` in both `impl Default` and `ConfigInitOptions::new()`.
   - Added builder `with_user_config_path(PathBuf)`.
   - In `init()`: resolve the user-config path ONCE as
     `options.user_config_path.clone().or_else(paths::user_config_path)` and use
     it for the READ side (User layer load). After building the handle, call
     `handle.set_user_config_path(options.user_config_path.clone())` so the WRITE
     side resolves to the SAME file. When `None`, falls back to
     `paths::user_config_path()` (which still honours `FFWB_USER_CONFIG_PATH`),
     and the stored path stays `None`.
   - Extended `config_init_options_builder_sets_fields` and
     `config_init_options_default_has_hot_reload_enabled` to assert the new field
     defaults to `None` and round-trips via `with_user_config_path`.
   - Added test `init_with_injected_user_config_path_reads_and_writes_that_file`
     proving BOTH read (init loads the injected file) and write
     (`set_user_value` / `remove_user_value` persist to the injected file) use
     the injected path. Annotated
     `// Validates: test-isolation hazard B048 -- explicit user-config path seam`.

2. `crates/ff-config/src/config_handle/mod.rs`
   - Added `pub(crate) user_config_path: Option<std::path::PathBuf>` field to
     `ConfigSystem`, initialised to `None` in BOTH `ConfigHandle::new` and
     `ConfigHandle::with_profile_manager`.
   - Added `pub(crate) fn set_user_config_path(&self, Option<PathBuf>)` (write
     lock, stores it).
   - Added `pub(crate) fn user_config_path_resolved(&self) -> Option<PathBuf>`
     (read lock; returns the stored path or falls back to
     `crate::paths::user_config_path()`).

3. `crates/ff-config/src/config_handle/runtime.rs`
   - `set_user_value` and `remove_user_value`: replaced
     `crate::paths::user_config_path()` with `self.user_config_path_resolved()`
     (same `NotFound` error on `None`). THIS is the write-side redirection the
     prior attempt missed; it is what makes
     `close_workspace_removes_settings_from_config` isolated.
   - `clear_audit_log` and `record_audit_entry`: audit dir now comes from
     `self.resolved_audit_dir()`, which uses the INJECTED user-config path's
     parent when a path was injected, else falls back to
     `crate::paths::user_config_dir()`.
   - Added two private helpers: `resolved_audit_dir()` and
     `user_config_path_injected()` (the latter returns ONLY the explicitly stored
     path, so a `None` injection falls back to `user_config_dir()` rather than a
     derived parent).

4. `crates/ff-config/src/config_handle/export_ext.rs`
   - `import_settings` `ImportTarget::UserLayer` arm: replaced
     `crate::paths::user_config_path()` with `self.user_config_path_resolved()`.
     `ProjectLayer` arm unchanged.

5. `crates/ff-desktop/src/shell/tests_common.rs`
   - `make_shell()`: removed BOTH `std::env::set_var(...)` calls. Builds a unique
     temp config path and a unique temp history path (new helper
     `unique_temp_name(prefix)` keeps the pid+nanos uniqueness scheme), injects
     the config path via `ConfigInitOptions::new().with_hot_reload(false)
     .with_user_config_path(cfg_path)`, and constructs the shell via
     `WorkbenchShell::new_with_history_store(.., Some(HistoryStore::new(hist_path)))`.
   - `make_shell_with_history_path(history_path)`: removed BOTH `set_var` calls.
     Injects a unique temp user-config path via `with_user_config_path` and the
     given history path via
     `new_with_history_store(.., Some(HistoryStore::new(history_path.to_path_buf())))`.
   - Added `use ff_keys::HistoryStore;` inside each helper.

6. `crates/ff-desktop/src/theme_defaults.rs`
   - `temp_config(root)` now also passes
     `.with_user_config_path(root.join("iso-config.toml"))` so the user layer is
     isolated via the explicit seam.
   - `resolve_startup_palette_loads_materialised_mode_file`: removed the
     `std::env::set_var("FFWB_USER_CONFIG_PATH", ..)` and the matching
     `remove_var` (isolation now comes from `temp_config`'s injected path).

## Env-var SETTERS remaining in crates/ff-desktop

Goal was zero `set_var("FFWB_USER_CONFIG_PATH"/"FFWB_HISTORY_PATH")` in
ff-desktop tests. After the edits, grep for `set_var` / `remove_var` /
`FFWB_USER_CONFIG_PATH` / `FFWB_HISTORY_PATH` under `crates/ff-desktop` should
show only READERS and doc comments (the production
`resolve_history_path()` in `shell/mod_helpers.rs` reads `FFWB_HISTORY_PATH`;
doc comments in `construct.rs` / `state.rs`), and NO remaining `set_var` of
either variable. (This grep could not be re-run post-edit due to the shell
blocker; the edits above deleted every setter call that the pre-edit grep found,
which were ALL in `tests_common.rs` and `theme_defaults.rs`.)

The env-var READERS in `ff_config::paths::user_config_path()` and
`resolve_history_path()` are intentionally LEFT IN PLACE (other callers / the
`paths.rs` `user_config_path_is_profile_aware` test rely on them).

## Commands to run (reviewer -- READ EACH LOG BACK)

Run each as:
`C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<cmd> > .gate\<name>.log 2>&1; $rc=$LASTEXITCODE; Write-Output \"KIRO_COMMAND_FINISHED rc=$rc\""`

1. `cargo test -p ff-config`            -> .gate\b048-ffconfig.log
2. `cargo test -p ff-desktop --bin ffwb tests_session tests_menu_workspace`
   (MULTITHREADED default -- do NOT pass --test-threads=1), run THREE times:
   -> .gate\b048-ffdesktop-1.log / -2.log / -3.log  (all three must be green)
3. `cargo nextest run -p ff-desktop -p ff-config`  -> .gate\b048-nextest.log
4. `cargo fmt --all -- --check`         -> .gate\b048-fmt.log
5. `cargo clippy -p ff-config -p ff-desktop --tests`  -> .gate\b048-clippy.log  (expect zero warnings)
6. `cargo build -p ff-desktop`          -> .gate\b048-build.log

Canonical tests that must pass:
- shell::tests_session::startup_loads_persisted_command_history
- shell::tests_session::exit_saves_command_history_and_reloads
- shell::tests_menu_workspace::close_workspace_removes_settings_from_config
- shell::tests_session::startup_missing_or_corrupt_history_is_empty_no_panic (stays green)
- ff-config: init_with_injected_user_config_path_reads_and_writes_that_file (new)

## Evidence lines (to be filled when the suites run)

(none captured -- shell blocker; see BLOCKER section)
