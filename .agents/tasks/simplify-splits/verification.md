# Verification Note -- shell file-size split (behaviour-preserving)

Pure file-split refactor (no gate). All edits made directly in the main
workspace, left UNCOMMITTED. This note records exactly what was run so the
reviewer need not re-run the suites.

## Scope recap

Three over-limit shell files split back under the 400 non-test-line rule:
- `crates/ff-desktop/src/shell/construct.rs`
- `crates/ff-desktop/src/shell/commands.rs`
- `crates/ff-desktop/src/shell/state.rs`

## Items moved (verbatim / grouped)

### construct.rs
- Free fn `build_live_provider_registry` -> new `shell/construct_provider.rs`
  (moved VERBATIM; visibility widened `fn` -> `pub(super)` so construct.rs can
  call it via `use super::construct_provider::build_live_provider_registry;`).
- The four built-in command registrations (file.open / file.exit / menu.open /
  config.open) -> new free fn `register_builtin_commands` in
  `shell/construct_commands.rs` (moved VERBATIM into a helper taking
  `&CommandRegistry`, `&PendingOpen`, `&Arc<Mutex<bool>>`; construct.rs now calls
  `register_builtin_commands(&registry, &pending_open, &should_close);`). Removed
  the now-unused `CommandId` / `CommandMetadata` imports and the four handler
  imports from construct.rs (kept `ShellContextProvider`, `CommandRegistry`).

### commands.rs
- Method `dispatch_to_environment` -> new `shell/commands_environment.rs`
  (moved VERBATIM; same `pub(super)` and signature). Imports `CommandEnvironment`
  (for the `claim` trait method), `EnvDispatchOutcome`, `RegisteredEnv`,
  `WorkbenchShell`. Removed the now-unused `CommandEnvironment` / `RegisteredEnv`
  imports from commands.rs (kept `EnvDispatchOutcome`, still used in
  `run_command_ladder`).
- Methods `run_command_line` / `begin_command_line` / `finish_command_line` ->
  new `shell/commands_line.rs` (moved VERBATIM; same `pub(super)`; the inner
  `use crate::shell::command_line_outcome::CommandLineOutcome;` kept exactly).

### state.rs (field grouping, mirroring the existing sub-struct pattern)
- `NavUiState` added to `shell/state_groups.rs`: the six modern-explorer nav_* UI
  fields (`nav_selection`, `nav_rename`, `nav_delete`, `nav_new`, `nav_focused`,
  `nav_file_clipboard`). `nav_model` deliberately kept flat.
- `HelpState` added: `help_registry` -> `registry`, `help_context_panel` ->
  `context_panel`, `help_missing_tally` -> `missing_tally`.
- `PendingTabActions` added: `pending_new_pom` -> `new_pom`, `pending_new_file`
  -> `new_file`, `pending_return_to_pom` -> `return_to_pom`.
- state.rs now holds single grouped fields `nav_ui`, `help`, `pending_tab_actions`.

Call sites rewritten to the grouped access path (mechanical, confined to
`crates/ff-desktop/src/shell/`):
- `self.nav_ui.*` in construct.rs, render_nav.rs, render_nav_ops.rs,
  update_input.rs.
- `self.help.*` in construct.rs, help.rs, render_body.rs.
- `self.pending_tab_actions.*` in construct.rs, render_body_arms.rs,
  render_tab_bar.rs, update.rs.

Modules declared in `shell/mod.rs`: `construct_provider`, `construct_commands`,
`commands_environment`, `commands_line` (`state_groups` already declared).

No public item renamed; no signature changed; no behaviour changed.

## Line counts (total lines; files contain no `#[cfg(test)]` module, so the
## checker counts every line)

| File                                   | Before | After |
|----------------------------------------|-------:|------:|
| shell/construct.rs                     |   453  |  348  |
| shell/commands.rs                      |   446  |  330  |
| shell/state.rs                         |   432  |  399  |
| shell/construct_provider.rs (new)      |    --  |   57  |
| shell/construct_commands.rs (new)      |    --  |   83  |
| shell/commands_environment.rs (new)    |    --  |   67  |
| shell/commands_line.rs (new)           |    --  |   71  |
| shell/state_groups.rs (extended)       |    98  |  171  |

`tools/python/check_line_limits.py` final run: "OK: no non-test .rs file exceeds
400 non-test lines." No NEW file is over the limit.

## Commands run (clean pwsh7 non-interactive wrapper, one per invocation, output
## redirected to tools/logs/ and read back)

Wrapper: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<cmd>"`

1. `cargo fmt` -- clean (no output; empty log `tools/logs/split-fmt.txt`).
2. `cargo check -p ff-desktop` -- CLEAN, finished, no warnings
   (`tools/logs/split-check.txt`).
3. `cargo clippy -p ff-desktop --tests` -- only 2 PRE-EXISTING warnings, both in
   `tests_command.rs:83` and `:96` (`clippy::type_complexity` on an
   `Arc<Mutex<Option<(String, Option<String>)>>>` local). That file was NOT
   touched by this split (`git status --porcelain` on it is empty), so the
   warnings are not attributable to the split (`tools/logs/split-clippy2.txt`).
4. `cargo test -p ff-desktop` -- 1000 passed; 2 failed in this run
   (`close_workspace_removes_settings_from_config`,
   `set_theme_disables_follow_os_so_selection_is_not_clobbered`). An earlier run
   failed a DIFFERENT set of 5 (theme/config/command-history). The failing set
   VARIES between runs and all are theme/config-dir/`FFWB_HISTORY_PATH`
   shared-state tests -- the known PRE-EXISTING B048 shared-env-var flake under
   multithreaded `cargo test`. Confirmed by re-running the failing tests
   single-threaded: `cargo test -p ff-desktop -- --test-threads=1 <names>` =>
   ALL pass (33 passed / 0 failed for the session batch; 2 passed / 0 failed for
   the theme/config pair). None of these tests touch any moved method or grouped
   field. B048 was NOT touched (per the standing rule).
5. `tools/python/check_line_limits.py` -- "OK: no non-test .rs file exceeds 400
   non-test lines" (`tools/logs/split-line-limits.txt`).

## Conclusion

All three target files are under 400 non-test lines; no new file is over.
Scoped `cargo fmt` / `cargo check` / `cargo clippy --tests` are clean (no new
warnings). The full ff-desktop test suite passes except for the pre-existing
B048 multithreaded shared-env-var flake, which passes in isolation and is
unrelated to the split. Behaviour is preserved (verbatim moves + a mechanical
field-access-path rename). HAND OFF to the owner for the full
`cargo gate --build`.
