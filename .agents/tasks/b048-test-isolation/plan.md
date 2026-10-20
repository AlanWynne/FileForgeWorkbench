# Implementation Plan -- B048 Test Isolation (constructor-seam injection)

Goal: eliminate the multithreaded `cargo test` flake in three ff-desktop shell
tests by removing the process-global env-var dependency from the shell test
helpers. Inject the user-config path and the command-line-history path directly
through constructor seams instead. This is a TEST-HARNESS fix plus a small
ADDITIVE library seam in `ff-config`; production runtime behaviour must not
change.

Failing tests (plain multithreaded `cargo test`, pass under nextest / `--test-threads=1`):
- `shell::tests_menu_workspace::close_workspace_removes_settings_from_config`
- `shell::tests_session::exit_saves_command_history_and_reloads`
- `shell::tests_session::startup_loads_persisted_command_history`

## Design decisions (grounded in the real code)

Decision 1 -- where the injected user-config path lives. The write side of
config (`set_user_value` / `remove_user_value` in
`crates/ff-config/src/config_handle/runtime.rs`) resolves its target by calling
`crate::paths::user_config_path()` directly; it does NOT reuse the User layer's
`source_path`. The read side (`init()` in `crates/ff-config/src/init/mod.rs`)
also calls `paths::user_config_path()`. So the injected path must be reachable
from BOTH `init()` and the `ConfigHandle` write methods. The cleanest seam is to
STORE the resolved user-config path on the handle's internal `ConfigSystem`
(field `user_config_path: Option<PathBuf>`) and have the write paths prefer it,
falling back to `paths::user_config_path()` when `None`. Rationale: the two
`ConfigHandle` constructors (`new`, `with_profile_manager`) are called from ~20
sites across `ff-config` (tests, `provider.rs`, `integration_tests.rs`) and
`init/mod.rs`; adding a stored field defaulted to `None` keeps every existing
`ConfigHandle::new(manager)` call compiling unchanged, versus threading a new
required parameter through all of them.

Decision 2 -- do NOT remove the env-var seam. `paths::user_config_path()` keeps
honouring `FFWB_USER_CONFIG_PATH`, and `resolve_history_path()` keeps honouring
`FFWB_HISTORY_PATH`, because `paths.rs` has its own test
(`user_config_path_is_profile_aware`) and other callers may rely on it. We only
ADD a higher-precedence explicit option and stop the ff-desktop tests from
SETTING the env vars.

Decision 3 -- history side already has its seam. `WorkbenchShell::new_with_history_store`
in `crates/ff-desktop/src/shell/construct.rs` already takes
`Option<ff_keys::HistoryStore>`; `HistoryStore::new(PathBuf)` builds one. The
test helpers must call this seam instead of setting `FFWB_HISTORY_PATH`. No new
history-side library code is required.

Decision 4 -- precedence. In both `init()` and the write methods: when the
explicit injected path is `Some`, use it; when `None`, behaviour is EXACTLY as
today (call `paths::user_config_path()`, which still consults the env var).
Production `init` in ff-desktop passes `None`, so production is unchanged.

Decision 5 -- which write paths to redirect. The acceptance test
`close_workspace_removes_settings_from_config` exercises
`set_user_value` + `remove_user_value` through the `ConfigHandle`. Those two
MUST use the stored path. For correctness and consistency, also redirect the
other `paths::user_config_path()` / `user_config_dir()` write-side callers that
a handle built with an injected path should honour: `import_settings`
(`config_handle/export_ext.rs`) and the audit-file writers
(`clear_audit_log`, `record_audit_entry` in `runtime.rs`, which derive a sibling
`audit.log` from the user-config DIR). These are additive redirections guarded
by `Some(path)`; when `None` they fall back to today's behaviour. (The audit
redirection derives the dir from the injected file's parent.)

## Plan

- [ ] 1. Add the `user_config_path` option to `ConfigInitOptions`.
      In `crates/ff-config/src/init/mod.rs`: add field `pub user_config_path: Option<PathBuf>`
      to `ConfigInitOptions`; default it to `None` in BOTH `impl Default` and
      `ConfigInitOptions::new()`; add builder `pub fn with_user_config_path(mut self, path: PathBuf) -> Self`
      mirroring `with_project_root`. Update the two existing builder-field unit
      tests only if they assert on all fields (they do not need to, but
      `config_init_options_default_has_hot_reload_enabled` and
      `config_init_options_builder_sets_fields` should be extended to assert the
      new field defaults to `None` and round-trips when set).
      Files: `crates/ff-config/src/init/mod.rs`
      Verify: `cargo test -p ff-config init::tests` (or `cargo nextest run -p ff-config`) -- builds and existing init tests pass.

- [ ] 2. Store the resolved user-config path on `ConfigSystem` and expose a
      precedence helper on `ConfigHandle`.
      In `crates/ff-config/src/config_handle/mod.rs`: add field
      `pub(crate) user_config_path: Option<std::path::PathBuf>` to `ConfigSystem`;
      initialise it to `None` in BOTH `ConfigHandle::new` and
      `ConfigHandle::with_profile_manager`. Add a crate-visible setter
      `pub(crate) fn set_user_config_path(&self, path: Option<PathBuf>)` that
      takes the write lock and stores it, and a private resolver used by the
      write methods: `fn resolved_user_config_path(system: &ConfigSystem) -> Option<PathBuf>`
      returning `system.user_config_path.clone().or_else(crate::paths::user_config_path)`.
      (Keep it as a free/assoc helper so both `runtime.rs` and `export_ext.rs`
      can call it; or expose `pub(crate) fn user_config_path_resolved(&self) -> Option<PathBuf>`
      on `ConfigHandle` taking its own read lock -- prefer the latter, simpler
      single method on `ConfigHandle`.)
      Files: `crates/ff-config/src/config_handle/mod.rs`
      Verify: `cargo build -p ff-config` compiles; `cargo test -p ff-config config_handle::tests` passes (existing handle tests unchanged).

- [ ] 3. Redirect the config WRITE paths to the resolved path.
      In `crates/ff-config/src/config_handle/runtime.rs`: in `set_user_value` and
      `remove_user_value`, replace `crate::paths::user_config_path()` with
      `self.user_config_path_resolved()` (the method from item 2), keeping the
      same `ok_or_else(... NotFound ...)` error. In `clear_audit_log` and
      `record_audit_entry`, derive the audit dir from the resolved user-config
      path's parent when set (e.g. `self.user_config_path_resolved().and_then(|p| p.parent().map(Path::to_path_buf))`),
      else fall back to `crate::paths::user_config_dir()`. Do not change the
      locked-key guard, the reload-after-write, or any signatures.
      Files: `crates/ff-config/src/config_handle/runtime.rs`
      Verify: `cargo test -p ff-config config_handle::tests` passes (the existing
      `set_user_value_persists_to_file` / `remove_user_value_restores_default`
      tests call the `toml_io` helpers directly and are unaffected; this confirms
      no regression).

- [ ] 4. Redirect `import_settings(UserLayer)` to the resolved path.
      In `crates/ff-config/src/config_handle/export_ext.rs`: in the
      `ImportTarget::UserLayer` arm, replace `crate::paths::user_config_path()`
      with `self.user_config_path_resolved()` (same error on `None`). Leave the
      `ProjectLayer` arm unchanged.
      Files: `crates/ff-config/src/config_handle/export_ext.rs`
      Verify: `cargo build -p ff-config` compiles; `cargo nextest run -p ff-config` all green.

- [ ] 5. Wire the injected option through `init()`.
      In `crates/ff-config/src/init/mod.rs` `init()`: compute the user-config
      path ONCE as
      `let user_cfg = options.user_config_path.clone().or_else(paths::user_config_path);`
      Use `user_cfg` where the User layer is read (the `if let Some(user_path) = paths::user_config_path()`
      block becomes `if let Some(user_path) = user_cfg.clone()`), so the READ side
      honours the injected path. After building `handle`, call
      `handle.set_user_config_path(options.user_config_path.clone());` so the
      WRITE side (items 3-4) resolves to the SAME injected file. When
      `options.user_config_path` is `None`, `user_cfg` falls back to
      `paths::user_config_path()` and the stored path stays `None` -> behaviour
      byte-for-byte as today. Add a unit test
      `init_with_injected_user_config_path_reads_and_writes_that_file`:
      build a tempdir file, `init(ConfigInitOptions::new().with_hot_reload(false).with_user_config_path(p))`,
      `set_user_value("editor.tab_size", Integer(2))`, assert the file `p` now
      contains the key and `remove_user_value` clears it -- proving BOTH sides use `p`.
      Files: `crates/ff-config/src/init/mod.rs`
      Verify: `cargo nextest run -p ff-config` -- the new test and all existing init tests pass.

- [ ] 6. Rewire `make_shell()` to inject both paths (no `set_var`).
      In `crates/ff-desktop/src/shell/tests_common.rs` `make_shell()`: keep the
      existing per-call unique temp-name scheme (`process::id()` + nanos) for BOTH
      the config file (`ffwb_test_cfg_...toml`) and the history file
      (`ffwb_test_hist_...toml`). DELETE both `std::env::set_var(...)` calls.
      Build the config handle via
      `init(ConfigInitOptions::new().with_hot_reload(false).with_user_config_path(cfg_path))`.
      Construct the shell via `WorkbenchShell::new_with_history_store(app, runtime, palette, vec![], config_handle, Some(ff_keys::HistoryStore::new(hist_path)))`
      instead of `WorkbenchShell::new(...)`. Add `use ff_keys::HistoryStore;` (and
      `use std::path::PathBuf;` if needed) to the helper's `use` block. Keep the
      shell otherwise identical.
      Files: `crates/ff-desktop/src/shell/tests_common.rs`
      Verify: `cargo build -p ff-desktop --tests` compiles.

- [ ] 7. Rewire `make_shell_with_history_path()` to inject both paths (no `set_var`).
      In the same file: `make_shell_with_history_path(history_path)` builds a
      unique temp config path (same scheme), passes it via `with_user_config_path`,
      and constructs via `new_with_history_store(..., Some(HistoryStore::new(history_path.to_path_buf())))`.
      Remove BOTH `set_var` calls. This is the helper the two session tests use,
      so after this item `startup_loads_persisted_command_history` and
      `exit_saves_command_history_and_reloads` no longer touch any env var.
      Files: `crates/ff-desktop/src/shell/tests_common.rs`
      Verify: `cargo build -p ff-desktop --tests` compiles.

- [ ] 8. Audit and convert the remaining ff-desktop `set_var` user of the env seam.
      Convert the one other setter found at
      `crates/ff-desktop/src/theme_defaults.rs` (test
      `*after materialisation*`, ~line 428) which does
      `std::env::set_var("FFWB_USER_CONFIG_PATH", &cfg_path)` and later
      `remove_var`. Its `temp_config(dir.path())` helper calls `ff_config::init`;
      change `temp_config` (and this test) to pass
      `ConfigInitOptions::new().with_hot_reload(false).with_user_config_path(<file under dir>)`
      and DELETE the `set_var` + `remove_var`. Read `temp_config`'s definition in
      that file first and thread the path through its signature. Then re-grep to
      confirm goal: `grep FFWB_USER_CONFIG_PATH`, `FFWB_HISTORY_PATH`, `set_var`
      under `crates/ff-desktop` returns NO remaining `set_var` of either variable
      (reads in doc comments and the production `resolve_history_path` /
      `paths.rs` reader are fine and must stay).
      Files: `crates/ff-desktop/src/theme_defaults.rs`
      Verify: `cargo nextest run -p ff-desktop theme_defaults` passes; the grep
      shows zero `set_var("FFWB_...")` occurrences in `crates/ff-desktop`.

- [ ] 9. Full gate: prove the flake is gone under multithreaded `cargo test`.
      Run the whole workspace test suite under PLAIN multithreaded `cargo test`
      (the mode that reproduced the flake), not only nextest, specifically
      targeting the ff-desktop binary where the three tests live.
      Files: (none -- verification only)
      Verify: `cargo test -p ff-desktop --bin ffwb; $rc=$LASTEXITCODE; Write-Output "KIRO_COMMAND_FINISHED rc=$rc"`
      -- expect `rc=0` with the three previously-failing tests passing and `0 failed`.
      Then run the canonical gate `cargo gate --build` and confirm it reaches a
      clean state (fmt ok, clippy clean, build, test all pass). If any ASCII-only
      or clippy issue surfaces in the touched files, fix and rerun.

## Notes / assumptions

- No new requirements; reuse the existing `// Validates:` annotations on the
  three tests. The new unit test in item 5 validates the SAME B048 isolation
  concern already documented on `paths::user_config_path` and
  `new_with_history_store`; annotate it accordingly
  (`// Validates: test-isolation hazard B048 -- explicit user-config path seam`).
- No assertions are weakened or deleted; nothing is marked ignored/skip.
- `WorkbenchShell::new_with_history_store` is `pub(crate)`; the test helpers are
  in the same crate (`crates/ff-desktop/src/shell/tests_common.rs`), so the seam
  is reachable without a visibility change.
- ASCII only in all edited files.
