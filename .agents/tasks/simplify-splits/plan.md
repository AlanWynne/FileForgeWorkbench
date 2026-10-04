# Implementation Plan -- Core-framework simplification: dead-code deletion + four 400-line splits

Behaviour-preserving REFACTOR + one deliberate dead-code deletion. NO observable
behaviour change, NO requirements gate. All paths relative to the worktree root
`c:\workspace\VSC\FileForgeWorkbench`.

## Ground rules for the coder

- SCOPED checks ONLY. Never run `--workspace` or `tools\ffwb-gate.ps1`. Use:
  - `cargo fmt`
  - `cargo check -p ff-desktop`
  - `cargo clippy -p ff-desktop`
  - `cargo test -p ff-desktop`
- Terminal is flaky. Run EACH command as ONE command via the wrapper and read a
  log, never inline, never `;`-chained:
  `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<ONE cmd>" *> tools\logs\<name>.txt`
  then read `tools\logs\<name>.txt`. Do NOT redirect to
  `tools\logs\phase2-linecounts.log` -- the line checker writes that file itself
  (a redirect to it causes PermissionError); let the checker own it and read it
  after.
- ASCII only in `.rs` files. Section separators use `// === Name ===...`.
- 400-line rule: no non-test `.rs` file over 400 NON-TEST lines. Checker:
  `python tools\python\check_line_limits.py` (measures lines up to the first
  top-level `#[cfg(test)]`). Run it after the splits; it writes
  `tools\logs\phase2-linecounts.log` -- read that.
- `mod.rs` re-exports only, no logic. A split primary file becomes a thin
  coordinator that declares sibling modules + the `use`/`pub use` the remaining
  code needs.
- Move tests WITH the functions they exercise so each moved fn stays covered.
- Preserve every existing `#[allow(dead_code)]` attribute on items that are
  moved (do NOT drop them).

## BEFORE state -- check_line_limits.py offender list (captured this session)

```
OVER-LIMIT non-test files (> 400 non-test lines):
    632  main.rs
    590  config_panel\render.rs
    404  tab_state.rs
    404  menu_workspace\loader.rs
```

`shell\dispatch.rs` is NOT over the limit; Change 1 is pure cleanup that lowers
it further. After all five changes the checker MUST report
`OK: no non-test .rs file exceeds 400 non-test lines.` (none of these four files
remain over the limit, and no new sibling is over the limit).

## Test-count expectation

Pre-change `ff-desktop` test count was 956. The ONLY test deleted is
`single_verb_arg_split_populates_arg_param` (Change 1). After the work expect
955 passing, 0 failed. The splits MOVE tests between files but delete none, so
they do not change the count.

## Discrepancies between the assessment and the real code (so the coder is not surprised)

1. Change 1 lines: `function_target_with_arg` is at approx lines 340-352 (not
   325-345) in `shell\dispatch.rs`; its test
   `single_verb_arg_split_populates_arg_param` and the KEEP test
   `verb_arg_split_preserves_argument_case` are both in the same `#[cfg(test)]`
   module. GREP already confirms the ONLY non-test caller count is zero: the fn
   is referenced only at its definition and inside that one test.
2. Change 2: `register_builtin_schema` is called OUTSIDE main.rs as
   `crate::register_builtin_schema` from `shell\tests_menu_workspace.rs`. So the
   move to `startup_schema.rs` MUST keep `crate::register_builtin_schema`
   resolving -- add a crate-root re-export (see Change 2 steps). Same applies to
   any main.rs test that moves into a sibling and still calls a fn now living in
   the OTHER sibling.
3. Change 4: the `tab_state.rs` constructor list is exactly: `untitled`,
   `for_file`, `pom`, `files_panel`, `config_panel`, `file_explorer_panel`,
   `search_results_panel`, `plugin_manager`, `event_log`, `scrm_viewer`,
   `macro_library`, `command_configurator`, `theme_editor`, `menus_editor`,
   `menu_workspace_tab`, and `encoding_label`. There is NO `keys_editor`,
   `kinds_editor`, or `help_context` CONSTRUCTOR (those `TabKind` variants exist
   but have no ctor fn). Only `theme_editor` and `menus_editor` carry
   `#[allow(dead_code)]` (CR-CH-022 retained-constructor) -- preserve it.
   `encoding_label` is a data method; the task lists it as staying in
   `tab_state.rs` -- keep it there with the data model.
5. Change 5: `menu_workspace\loader.rs` is exactly 404 non-test lines (checker is
   authoritative), so it IS over. `option_limits_from_config` and `validate_menu`
   are called from SIX shell files via the path
   `crate::menu_workspace::loader::<fn>` (commands_fastpath, commands_menu,
   commands_session, menus_editor, nav_reconstruct). Those call sites MUST keep
   resolving -- `loader.rs` re-exports the moved validators so the
   `loader::<fn>` path is unchanged (see Change 5 steps). `OptionLimits` is also
   re-exported from `menu_workspace\mod.rs` as `pub use loader::{LoadedMenu,
   OptionLimits}` and used widely as `crate::menu_workspace::OptionLimits`; keep
   BOTH the `menu_workspace::OptionLimits` and `menu_workspace::loader::OptionLimits`
   paths valid.

---

- [ ] 1. Delete the dead `function_target_with_arg` helper and its sole test.
      In `crates\ff-desktop\src\shell\dispatch.rs`:
      (a) Confirm no caller: run
      `grep` for `function_target_with_arg` across `crates\**\*.rs` and verify
      the ONLY hits are the definition and the one test
      `single_verb_arg_split_populates_arg_param` (already verified this session
      -- re-confirm). If any OTHER live caller exists, STOP and report instead of
      deleting.
      (b) Delete the fn `function_target_with_arg` (private `fn`, carrying
      `#[allow(dead_code)] // no live caller yet; Step 3 Function verbs were
      deferred.`) together with its doc comment block, approx lines 338-353.
      (c) Delete ONLY the test `single_verb_arg_split_populates_arg_param` from
      the `#[cfg(test)] mod tests` block. KEEP `verb_arg_split_preserves_argument_case`
      (it tests the LIVE `split_verb_arg`).
      (d) Fix imports: the test module `use`s `super::*` and
      `crate::shell::helpers::split_verb_arg`. After deleting the fn, the file no
      longer uses `CommandTarget`, `TargetParams`, `TargetValue` in the deleted
      test; check whether the top-level `use ff_command::{CommandTarget,
      TargetParams, TargetValue};` is still needed elsewhere in the file (the
      router body uses `CommandTarget`/`TargetParams`/`TargetValue`? -- `ffedit_claim`
      and `dispatch_command_string` do NOT; they are used only by the deleted fn).
      Remove any now-unused import to keep clippy clean; let `cargo check`/`clippy`
      tell you exactly which. Do NOT remove `use super::WorkbenchShell` or the
      `ff_command` import if still referenced.
      Files: `crates\ff-desktop\src\shell\dispatch.rs`
      Verify: `cargo check -p ff-desktop` compiles; `cargo clippy -p ff-desktop`
      is clean (no unused-import warning); `cargo test -p ff-desktop` passes with
      the count reduced by exactly 1 (956 -> 955) and
      `verb_arg_split_preserves_argument_case` still present and passing.

- [ ] 2. Split `crates\ff-desktop\src\main.rs` (632 non-test) into a coordinator
      + `startup_schema.rs` + `startup_env.rs`.
      (a) Create `crates\ff-desktop\src\startup_schema.rs`. Move
      `register_builtin_schema` (fn + its large `SchemaEntry` array) into it,
      changing its visibility to `pub(crate)` (it is currently a bare private
      `fn` but is called as `crate::register_builtin_schema` from
      `shell\tests_menu_workspace.rs`). Add the `use` lines the fn body needs
      (the body already does `use ff_config::error::ValueType; use
      ff_config::schema::{Constraints, SchemaEntry}; use ff_config::value::ConfigValue;`
      inside the fn, so no file-level imports beyond `ff_config`/`ff_logging`
      paths used fully-qualified -- verify with `cargo check`). Move the
      schema-focused tests into this file's `#[cfg(test)] mod tests`:
      `reduce_motion_config_key_is_registered_in_schema`,
      `menu_limit_keys_have_correct_defaults`, and
      `project_config_logging_directory_is_resolved_for_reconfigure` (all call
      `register_builtin_schema`). Give the test module the `use` it needs
      (`use super::*;`, `use ff_config::init::{init, ConfigInitOptions};`,
      `tempfile::TempDir`).
      (b) Create `crates\ff-desktop\src\startup_env.rs`. Move
      `apply_os_reduce_motion`, `os_prefers_reduce_motion` (INCLUDING the
      `#[cfg(target_os = "windows")]` FFI `extern "system"` block and the
      `#[cfg(not(target_os = "windows"))]` arm), and `apply_logging_config`.
      Make the ones called from main() `pub(crate)` (all three are called from
      main()). Move the `apply_logging_config_is_safe_when_logging_not_active`
      test here. That test ALSO calls `register_builtin_schema`; reference it as
      `crate::register_builtin_schema` (resolved via the crate-root re-export in
      step (d)).
      (c) In `main.rs` KEEP: the `#![cfg_attr(not(test), windows_subsystem =
      "windows")]` attribute, the crate doc comment, ALL existing `mod ...;`
      declarations, `main()`, `extract_profile_arg`, `resolve_cli_paths`, and the
      remaining main.rs tests (`workbench_app_boots_and_shuts_down_cleanly`, the
      four `resolve_cli_paths_*`, the four `extract_profile_arg_*`). Add
      `mod startup_schema;` and `mod startup_env;` to the module list.
      (d) Make the moved fns reachable from their original call sites. main()
      currently calls `register_builtin_schema`, `apply_os_reduce_motion`,
      `apply_logging_config` unqualified. Add to main.rs (crate root):
      `use startup_schema::register_builtin_schema;` and
      `use startup_env::{apply_os_reduce_motion, apply_logging_config};`
      so main() is unchanged, AND add
      `pub(crate) use startup_schema::register_builtin_schema;` is NOT needed if
      the plain `use` at crate root already makes `crate::register_builtin_schema`
      resolve -- a crate-root `use` import IS reachable as `crate::<name>`.
      Confirm the external caller `crate::register_builtin_schema` in
      `shell\tests_menu_workspace.rs` still resolves (it will, via the crate-root
      `use`). `os_prefers_reduce_motion` is only called by `apply_os_reduce_motion`
      (same file), so it needs no re-export; keep it `fn` (private to
      startup_env) -- but it is referenced in a doc test? No: no external caller;
      keep private.
      (e) Trim the main.rs top-level `use`s: after the move, `main.rs` no longer
      needs the imports used only by the moved fns. Let `cargo check` /
      `cargo clippy` report unused imports and remove exactly those; keep
      everything main()/the CLI helpers/the kept tests still use
      (`anyhow::Context`, `eframe::egui`, `ff_config::init::*`, `ff_core::*`,
      `ff_logging::*`, `ff_session::UserDataDir`, `shell::WorkbenchShell`,
      `tokio::runtime::Runtime`).
      Files: create `crates\ff-desktop\src\startup_schema.rs`,
      `crates\ff-desktop\src\startup_env.rs`; modify
      `crates\ff-desktop\src\main.rs`.
      Verify: `cargo fmt`; `cargo check -p ff-desktop`; `cargo clippy -p
      ff-desktop` clean; `cargo test -p ff-desktop` passes (the moved schema/env
      tests run from their new files). Run `python tools\python\check_line_limits.py`
      and read `tools\logs\phase2-linecounts.log` -- `main.rs` must be gone from
      the offender list and neither new sibling may appear.

- [ ] 3. Split `crates\ff-desktop\src\config_panel\render.rs` (590 non-test) into
      `render.rs` (kept) + `keyboard.rs` + `commit.rs` siblings under
      `config_panel\`.
      (a) Create `crates\ff-desktop\src\config_panel\keyboard.rs`. Move
      `key_widget_id`, `tree_has_keyboard`, and `config_keyboard_effects` (all
      currently private `fn`). Keep them `fn` but visible to the parent module's
      other children: make them `pub(super)` (so `render.rs` can call
      `tree_has_keyboard` and `config_keyboard_effects`). `key_widget_id` is used
      only inside `keyboard.rs` (by `tree_has_keyboard` and
      `config_keyboard_effects`) -- keep it private `fn` within keyboard.rs. Add
      the `use`s this file needs: `use eframe::egui;` and
      `use super::{filter_field_id, reduce_config_key, ConfigNodeId,
      ConfigPanelState, ConfigRow, ConfigTreeEffect, ConfigTreeKey};`.
      Note: `config_keyboard_effects` reads/writes `state.cursor`,
      `state.collapsed`, and `state.widget_ids`. `widget_ids` is a PRIVATE field
      of `ConfigPanelState` (defined in `config_panel\mod.rs`); child modules of
      `config_panel` can access it, so NO visibility change to the struct field
      is required.
      (b) Create `crates\ff-desktop\src\config_panel\commit.rs`. Move
      `commit_value` and `validate_against_constraints` (both private `fn`). Make
      `commit_value` `pub(super)` (called by `render_widget` which stays in
      render.rs). `validate_against_constraints` is called by `commit_value` and
      by several render.rs tests -- make it `pub(super)` so the moved tests (next
      line) and any that stay can reach it. Move the constraint-validation unit
      tests that call `validate_against_constraints` into commit.rs's test module:
      `valid_value_passes_constraint_check`, `invalid_value_shows_error`,
      `string_pattern_validation_fails_for_non_matching_value`,
      `string_pattern_validation_passes_for_matching_value`. Add the file `use`s:
      `use ff_config::value::ConfigValue; use ff_config::ConfigHandle;` and
      `use super::ConfigPanelState;` (plus `ff_config::schema::Constraints` as
      used in the signature).
      (c) In `render.rs` KEEP: `render` (the `pub fn` view entry), `render_entry`,
      `render_widget`, `paint_cursor_highlight`, `entry_matches`, `namespace_of`,
      `ns_display_name`, `layer_label`, and the tests that do NOT call the moved
      fns (`namespace_grouping_correct`, `filter_hides_non_matching_keys`,
      `provenance_badge_shows_correct_layer`, `widget_type_selected_for_bool`,
      `widget_type_selected_for_enum_string`, `widget_type_selected_for_bounded_int`,
      `reset_button_hidden_when_at_default`, `f3_returns_to_pom_via_end_command`).
      Add to render.rs: `use super::keyboard::{config_keyboard_effects,
      tree_has_keyboard};` and `use super::commit::commit_value;`. The call sites
      in `render` (`tree_has_keyboard(ui, &rows)` and
      `config_keyboard_effects(ui, state, &rows)`) and in `render_widget`
      (`commit_value(...)`) are unchanged.
      (d) Update `crates\ff-desktop\src\config_panel\mod.rs`: add `mod keyboard;`
      and `mod commit;` alongside the existing `mod render;`. mod.rs keeps
      `pub use render::render;` (unchanged) and declares the new modules with NO
      logic. The `WorkspaceContext` impl in mod.rs still calls `render(...)` --
      unchanged.
      Files: create `crates\ff-desktop\src\config_panel\keyboard.rs`,
      `crates\ff-desktop\src\config_panel\commit.rs`; modify
      `crates\ff-desktop\src\config_panel\render.rs`,
      `crates\ff-desktop\src\config_panel\mod.rs`.
      Verify: `cargo fmt`; `cargo check -p ff-desktop`; `cargo clippy -p
      ff-desktop` clean; `cargo test -p ff-desktop` passes (moved
      constraint-validation tests run from commit.rs). `check_line_limits.py` must
      no longer list `config_panel\render.rs`, and neither `keyboard.rs` nor
      `commit.rs` may appear.

- [ ] 4. Split `crates\ff-desktop\src\tab_state.rs` (404 non-test) into
      `tab_state.rs` (data model, kept) + `tab_state_ctors.rs` (macro +
      constructors).
      Macro-ordering approach (STATED PRECISELY): the `base_tab!`
      `macro_rules!` and ALL the `impl TabState` constructor fns move TOGETHER
      into the SAME new file `tab_state_ctors.rs`, with the `macro_rules!`
      definition placed ABOVE the `impl TabState` block in that file. Because
      `macro_rules!` is textually-scoped, defining it earlier in the same file
      than its uses satisfies the ordering with no `#[macro_use]`/`#[macro_export]`
      and no cross-module export. The macro stays private to `tab_state_ctors.rs`.
      (a) Create `crates\ff-desktop\src\tab_state_ctors.rs`. Move: the
      `macro_rules! base_tab { ... }` definition, then an `impl TabState { ... }`
      block containing `untitled`, `for_file`, `pom`, `files_panel`,
      `config_panel`, `file_explorer_panel`, `search_results_panel`,
      `plugin_manager`, `event_log`, `scrm_viewer`, `macro_library`,
      `command_configurator`, `theme_editor`, `menus_editor`,
      `menu_workspace_tab`. KEEP `theme_editor` and `menus_editor` AND their
      `#[allow(dead_code)]` + the CR-CH-022 retained-constructor doc comment --
      do NOT delete them. Add the file `use`s the moved code needs:
      `use ff_document_model::{DocumentHandle, LineEndMode};`
      `use ff_edit_operations::EditProfile;`
      `use ff_viewport_scrolling::{CursorModel, ViewportModel};`
      `use std::collections::HashMap;`
      `use crate::menu_workspace::MenuWorkspaceState;`
      `use super::{TabId, TabKind, TabState};`
      (`UndoEntry` is referenced by the struct literal only via the
      `undo_stack: Vec::new()` field, so no `UndoEntry` import is needed; verify
      with `cargo check`).
      (b) In `tab_state.rs` KEEP: the module doc comment, the file-level `use`s
      the data model still needs, `enum TabKind`, `enum UndoEntry`, `struct TabId`,
      `struct TabState`, and the `impl TabState { encoding_label }` method (the
      data method the task assigns to the data file). Remove the `macro_rules!
      base_tab` and the moved constructor `impl` block. Add `mod tab_state_ctors;`
      is NOT valid here because `tab_state` is a single-file module, not a
      directory module. Instead declare the sibling at the CRATE ROOT (main.rs)
      -- see step (d).
      Keep the `#[allow(dead_code)]` on `TabState::id`'s field as-is.
      (c) `encoding_label` stays with the data model in `tab_state.rs`. If
      `tab_state.rs` after removing the constructors is now small, that is fine;
      the goal is both files under 400 non-test.
      (d) In `crates\ff-desktop\src\main.rs` add `mod tab_state_ctors;` to the
      module list (adjacent to `mod tab_state;`). Both are flat sibling modules
      at the crate root; `tab_state_ctors.rs` uses `super::` = crate root to
      reach `crate::tab_state::{TabState, TabId, TabKind}` -- adjust the import in
      (a) to `use crate::tab_state::{TabId, TabKind, TabState};` (NOT `super::`,
      since the file is a crate-root module, not a child of `tab_state`). This is
      the authoritative import form; `cargo check` confirms.
      (e) Keep any existing `#[cfg(test)]` tests in `tab_state.rs` with the code
      they exercise. (The current `tab_state.rs` has no test module below the
      data model in the shown content; if a test module exists it stays unless it
      tests a moved constructor, in which case move it to `tab_state_ctors.rs`.)
      Files: create `crates\ff-desktop\src\tab_state_ctors.rs`; modify
      `crates\ff-desktop\src\tab_state.rs`, `crates\ff-desktop\src\main.rs`.
      Verify: `cargo fmt`; `cargo check -p ff-desktop`; `cargo clippy -p
      ff-desktop` clean (the two `#[allow(dead_code)]` ctors must NOT produce
      warnings -- confirm the attribute moved with them); `cargo test -p
      ff-desktop` passes. `check_line_limits.py` must no longer list
      `tab_state.rs` and must not list `tab_state_ctors.rs`.

- [ ] 5. Split `crates\ff-desktop\src\menu_workspace\loader.rs` (404 non-test)
      into `loader.rs` (load entry points + serde shims, kept) + `validate.rs`
      (limits + validation).
      (a) Create `crates\ff-desktop\src\menu_workspace\validate.rs`. Move:
      `struct OptionLimits` + `impl OptionLimits` (incl. `DEFAULT_SOFT`,
      `DEFAULT_HARD`, `new` with its `#[allow(dead_code)]`, `effective_soft`) +
      `impl Default for OptionLimits`; `option_limits_from_config` (keep its
      `#[allow(dead_code)]`); `read_u32_or_default` (keep its `#[allow(dead_code)]`);
      `apply_limits`; `validate_menu`; `validate_option`. Visibility: keep
      `OptionLimits` and its public methods `pub` (it is part of the
      `menu_workspace` public surface). Keep `option_limits_from_config` and
      `validate_menu` `pub` (called from other crates-internal modules). Make
      `apply_limits` and `validate_option` `pub(super)` so loader.rs can call
      them (`load_menu_file`, `parse_menu_str` call `validate_option`;
      `load_menu_file_with_limits` calls `apply_limits`). Keep
      `read_u32_or_default` private to validate.rs (only
      `option_limits_from_config` uses it). Add the file `use`s:
      `use super::{MenuFile, MenuOption};` and (for `apply_limits`/`LoadedMenu`)
      `use super::loader::LoadedMenu;` -- see note on LoadedMenu below.
      (b) In `loader.rs` KEEP: `load_menu_file`, `load_menu_file_with_limits`,
      `parse_menu_str`, `struct LoadedMenu`, the serde shim types (`RawMenuFile`,
      `RawMenuOption`, `default_true`). `load_menu_file_with_limits` takes an
      `OptionLimits` param and calls `apply_limits`; `load_menu_file`/`parse_menu_str`
      call `validate_option`. Add to loader.rs:
      `use super::validate::{apply_limits, validate_option};` and keep
      `OptionLimits` reachable in loader.rs's signatures via
      `use super::validate::OptionLimits;`.
      (c) PRESERVE the existing call-site paths. Other modules call
      `crate::menu_workspace::loader::option_limits_from_config` and
      `crate::menu_workspace::loader::validate_menu` (six files:
      commands_fastpath, commands_menu, commands_session, menus_editor,
      nav_reconstruct). To keep the `loader::` path valid WITHOUT touching those
      callers, add RE-EXPORTS in loader.rs:
      `pub use super::validate::{option_limits_from_config, validate_menu, OptionLimits};`
      (so `loader::option_limits_from_config`, `loader::validate_menu`, and
      `loader::OptionLimits` all still resolve). Do NOT also `use` the same names
      -- a single `pub use` both imports and re-exports them.
      (d) Update `crates\ff-desktop\src\menu_workspace\mod.rs`: add `pub mod
      validate;` next to the existing `pub mod loader;`. The existing
      `pub use loader::{LoadedMenu, OptionLimits};` keeps working because loader.rs
      now re-exports `OptionLimits` (step c) and still defines `LoadedMenu`. Leave
      that line unchanged (both names still resolve through `loader`). mod.rs
      stays re-exports/data-only.
      (e) `LoadedMenu` ownership: `apply_limits` (moved to validate.rs) RETURNS
      `LoadedMenu`, which stays defined in loader.rs. validate.rs therefore
      imports it: `use super::loader::LoadedMenu;`. This is a normal
      sibling-to-sibling import (no cycle at type level: loader `use`s fns from
      validate; validate `use`s the `LoadedMenu` type from loader -- Rust allows
      mutual module imports).
      (f) Move the limit/validation tests that live in loader.rs's test module
      into validate.rs's test module (they call `apply_limits`,
      `OptionLimits::new`, `option_limits_from_config`): `count_at_or_below_soft_limit_no_advisory`,
      `count_above_soft_below_hard_sets_advisory`, `count_above_hard_returns_load_error`,
      `hard_below_soft_clamps_effective_soft_to_hard`, `disabled_options_count_toward_limits`,
      `default_limits_are_64_and_256`, `option_limits_from_config_falls_back_to_defaults`,
      and the `menu_with` test helper they share. KEEP in loader.rs the parse/IO
      tests (`load_valid_menu_file`, `load_inline_options_target_parses`, ... ,
      `load_group_headers_true_parses`) and `load_with_limits_end_to_end_within_soft`
      (it exercises `load_menu_file_with_limits`, a loader fn -- keep in loader,
      it reaches `OptionLimits::default()` via the loader re-export). Give each
      test module the `use super::*;` plus the `MenuFile`/`MenuOption`/`GroupSeparator`
      references it needs (note existing tests use `super::super::GroupSeparator`;
      in validate.rs that path is still `super::super::GroupSeparator` because
      validate.rs is also a child of `menu_workspace`).
      Files: create `crates\ff-desktop\src\menu_workspace\validate.rs`; modify
      `crates\ff-desktop\src\menu_workspace\loader.rs`,
      `crates\ff-desktop\src\menu_workspace\mod.rs`.
      Verify: `cargo fmt`; `cargo check -p ff-desktop`; `cargo clippy -p
      ff-desktop` clean (the three `#[allow(dead_code)]` items -- `new`,
      `option_limits_from_config`, `read_u32_or_default` -- must keep their
      attributes so no dead-code warning appears); `cargo test -p ff-desktop`
      passes. `check_line_limits.py` must no longer list `menu_workspace\loader.rs`
      and must not list `menu_workspace\validate.rs`. If `loader.rs` is STILL over
      400 non-test after the move (it should not be -- approx 174 remain), THEN
      also move the serde shims (`RawMenuFile`, `RawMenuOption`, `default_true`)
      into a new `menu_workspace\types.rs`, add `mod types;` to mod.rs, and
      `use super::types::{RawMenuFile, RawMenuOption};` in loader.rs. Only do this
      if the checker still reports loader.rs over the limit.

- [ ] 6. Final whole-task verification (ASCII + 400-line + scoped gate).
      (a) ASCII check on every touched/created `.rs` file: run
      `rg "[^\x00-\x7F]" crates\ff-desktop\src --glob "*.rs"` via the wrapper and
      read the log -- there must be ZERO matches in the new/edited files (new
      sibling files, the four split primaries, dispatch.rs, main.rs). Fix any
      non-ASCII with the ASCII substitute.
      (b) `python tools\python\check_line_limits.py` -> read
      `tools\logs\phase2-linecounts.log`: it MUST read
      `OK: no non-test .rs file exceeds 400 non-test lines.`
      (c) `cargo fmt -- --check` passes.
      (d) `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop` (clean,
      no new warnings), `cargo test -p ff-desktop` (955 passed, 0 failed).
      Files: none (verification only).
      Verify: all commands above succeed; test count is exactly 955 passing, 0
      failed; line checker reports OK; ASCII check empty.

## Hand-off

After step 6 is clean, this is CODE-COMPLETE pending the owner's full gate. State
exactly which scoped commands were run and prompt the owner to run
`pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1` outside Kiro. Do NOT run
the full gate here.
