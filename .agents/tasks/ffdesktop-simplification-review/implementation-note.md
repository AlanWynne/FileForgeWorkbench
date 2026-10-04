# ff-desktop simplification -- implementation note (iteration 1)

Owner decision this run: Change B SKIPPED (Option 3). Only Change A + Change C
were implemented. The reviewer should verify A + C only.

## Change A (F7) -- delete dead code -- DONE (one STOP, as intended)
Files:
- Deleted `crates/ff-desktop/src/panel_layout.rs` entirely (whole module,
  `#[allow(dead_code)]`, no caller). Grep confirmed the only references were
  inside the file itself plus the `mod panel_layout;` line.
- Removed `mod panel_layout;` from `crates/ff-desktop/src/main.rs`.
- Deleted from `crates/ff-desktop/src/tab_manager.rs` (grep-confirmed NO caller
  anywhere, including tests):
  - `open_theme_editor_tab`
  - `open_menus_editor_tab`
  - `open_menu_workspace_here`
- STOP (kept, NOT deleted): `transform_active_pom_tab` has LIVE test callers in
  the shell tests (originally shell/tests.rs lines ~616/634/1549/1581/1768; now
  in the split test files). Per the task STOP rule, it was left in place. The
  owner confirmed this STOP decision is correct.
- Also fixed one pre-existing non-ASCII em dash in a tab_manager.rs comment
  while the file was open (documentation.md ASCII rule).
- Note: the `TabState::menus_editor` constructor is now only referenced by its
  own `#[allow(dead_code)]`; `TabState::theme_editor` is still used by a test.
  No new dead_code warnings result (both already carried `#[allow(dead_code)]`).

## Change B (F6) -- SKIPPED by owner decision (Option 3)
Left exactly as-is: the notification channel, `notification_rx`/`notification_tx`,
`notification_sender()`, the per-frame drain in shell/update.rs, and the tests
`notifications_drained_from_channel_each_frame` and
`notification_sender_is_clone_and_send`. Rationale (owner): removing the channel
invalidates documented notification-system Requirement 1.1, so it is not a pure
behaviour-preserving refactor and must go through the requirements gate
separately. Deferred to the gate; out of scope for this run.

## Change C (F3) -- split shell/tests.rs by feature area -- DONE
- `crates/ff-desktop/src/shell/tests.rs` (10,999 lines, one `mod tests`) was
  split into sibling test modules, all DIRECT CHILDREN of `shell` so every
  `super::X` reference inside a moved test still resolves to `shell` unchanged
  (the file used `super::` heavily -- title_line_text, FloatingTab,
  WorkspaceCommandContext, target_dispatch::*, line_end_from_name, etc.; a
  deeper `tests/` subdir would have broken all of them).
- New files under `crates/ff-desktop/src/shell/`:
  - `tests_common.rs`  -- 23 shared helpers (make_shell, make_dispatch,
    is_shell_command, harness_shell, etc.), made `pub(crate)`, plus the verbatim
    preamble `use`s (incl. the multi-line `ff_command` import).
  - `tests_command.rs` (83), `tests_focus.rs` (87),
    `tests_menu_workspace.rs` (96), `tests_misc.rs` (71), `tests_nav.rs` (44),
    `tests_scrm.rs` (16), `tests_session.rs` (25), `tests_split_detach.rs` (32).
  Each area file does `use super::tests_common::*;` + the preamble `use`s and
  carries `#![allow(unused_imports)]` so the propagated preamble never trips
  clippy `-D warnings`.
- `shell/mod.rs`: `#[cfg(test)] mod tests;` replaced by `#[cfg(test)]` module
  declarations for `tests_common` + the eight area modules.
- Pre-existing non-ASCII box-drawing separators (U+2500) in comments, carried
  over verbatim from the old file, were converted to ASCII `-` in the new files
  (documentation.md: .rs must be ASCII-only). Comments only; no code bytes
  changed.

### Test-count / name invariant (Change C)
- BEFORE (git HEAD tests.rs): 454 `#[test]` fns, 454 unique names.
- AFTER (sum across tests_*.rs): 454 `#[test]` fns, 454 unique names.
- Symmetric diff of the name sets: 0 missing, 0 added, 0 duplicates.
  RESULT: IDENTICAL test-name set and count. No test renamed, deleted,
  weakened, or #[ignore]d. (`rg -c '#[test]'` per-file: 83+87+96+71+44+16+25+32
  = 454; tests_common.rs = 0.)

## Scoped verification (commands Kiro ran; NOT --workspace, NOT verify.ps1)
- `cargo fmt`                       -> clean (no diffs on `cargo fmt -- --check`).
- `cargo check -p ff-desktop`       -> compiles. Only pre-existing unrelated
  warning: unused import `GUTTER_CHAR_WIDTH` in editor_panel/mod.rs (not mine).
- `cargo clippy -p ff-desktop --tests` -> clean (no code warnings; only the
  unrelated ff-mdx Cargo-profile notices). FEWER dead_code allowances now
  (three tab_manager methods + the whole panel_layout module removed).
- `cargo test -p ff-desktop --no-run` -> tests compile.
- `cargo test -p ff-desktop` (multithreaded) -> 930 passed, 3 FAILED:
    - shell::tests_session::startup_loads_persisted_command_history
    - shell::tests_session::exit_saves_command_history_and_reloads
    - shell::tests_menu_workspace::close_workspace_removes_settings_from_config
  These are the KNOWN B048 shared-env-var race (make_shell / make_shell_with_
  history_path call std::env::set_var("FFWB_HISTORY_PATH"/"FFWB_USER_CONFIG_PATH")
  which races across threads under plain `cargo test`). They are NOT introduced
  by the split -- the tests are intact; only thread interleaving changed.
  Confirmed:
    - `cargo test -p ff-desktop -- --test-threads=1 <the 3 tests>` -> 3 passed.
    - `cargo nextest run -p ff-desktop` (process-per-test, the project's
      canonical runner per testing.md) -> no failures observed; this isolation
      is exactly why nextest is canonical (B048).
  Net: under the canonical runner / in isolation, all 454 shell tests pass; the
  3 are a pre-existing multithreaded-`cargo test` isolation artifact, documented.

## STOP items reported
- `transform_active_pom_tab` NOT deleted (live test callers) -- confirmed correct
  by owner.

## Scope / working-tree note
Changes made by this run: main.rs, panel_layout.rs (deleted), shell/mod.rs,
tab_manager.rs, shell/tests.rs (deleted), the 9 new shell/tests_*.rs files, and
this note under .agents/tasks/. No commit was made (left in the working tree for
owner review). Other modified/untracked files in `git status` (tooling.md,
pwsh_command_guard*.py, wiring-standard.md, cargo.tecst.txt) are from PRIOR
workflow steps, not this run.
