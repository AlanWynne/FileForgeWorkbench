# Phase 2 Task 2.2 -- Verification Note (IN PROGRESS -- paused for owner decision)

Pure behaviour-preserving file-size split of oversized shell-structure `.rs`
files under the 400-non-test-line rule. No runtime behaviour, rendering, focus
order, command dispatch, or public cross-crate API changed.

## Status: PAUSED pending owner decision on `shell/mod.rs` struct (see below)

## Work completed so far (all compiles clean)

### shell/mod.rs split (1680 -> 480 non-test lines)
New sibling submodules created, each an `impl WorkbenchShell` block or type/fn
group moved VERBATIM (names, signatures, visibility preserved):

- `shell/handlers.rs` -- the built-in command handler structs
  (FileOpenHandler, FileExitHandler, MenuOpenHandler, ShellContextProvider,
  ConfigOpenHandler). Made `pub(super)` so `construct.rs` can build them.
- `shell/mod_helpers.rs` -- the bottom free functions line_end_from_name,
  title_line_text, truncate_title, load_context_maps_from_config,
  resolve_history_path, ensure_keymaps_dir, load_context_maps_from_keymaps_dir.
  Re-exported from mod.rs via `pub(crate) use mod_helpers::*;` so every
  `super::title_line_text` / `super::truncate_title` / `super::line_end_from_name`
  reference in siblings/tests still resolves.
- `shell/workspace_io.rs` -- open_workspace, open_workspace_force,
  save_workspace_to, close_workspace.
- `shell/titles.rs` -- kind_title, title_line_display, tab_header_label,
  key_list_context_for_tab, apply_kind_profile_to_active,
  command_line_position_for.
- `shell/construct.rs` -- new() and new_with_history_store() (constructor), with
  identical field-initialisation order.
- `shell/actions.rs` -- notification_sender, macro_dirs,
  apply_macro_library_action, apply_search_outcome, format_session_start,
  active_profile_label, format_logoff_message, shell_open_file,
  shell_new_untitled.
- `shell/types.rs` -- KeyBarScope enum + impl, WorkspaceCommandContext,
  FloatingTab. Re-exported via `pub(crate) use types::*;` so `super::KeyBarScope`
  etc. still resolve.

mod.rs now holds: the WorkbenchShell struct definition (lines 54-400, ~347
lines), its imports, an empty-but-for-comments `impl WorkbenchShell` block, all
`mod ...;` declarations, and the two re-export lines.

## The blocker -- mod.rs cannot reach <= 400 while the struct stays in mod.rs

The `WorkbenchShell` struct definition is 347 non-test lines by itself (fields +
their doc comments). mod.rs ALSO must carry the ~80 lines of mandatory `mod ...;`
submodule declarations plus ~30 lines of imports. So mod.rs has a hard floor of
~347 + 80 + 30 = ~457 lines WHILE the struct remains in mod.rs -- it cannot drop
below 400 without moving the struct out.

The task prompt explicitly said: "KEEP in mod.rs: the WorkbenchShell struct
definition" and "preserving ... pub(crate)/pub(super)/pub visibility". Moving the
struct to `shell/state.rs` (the layout `rust-standards.md` actually prescribes)
would require changing the struct's ~90 PRIVATE fields to `pub(crate)`, because
sibling modules (render.rs, commands.rs, update.rs, titles.rs, actions.rs, ...)
access those fields directly and private fields are only visible to the defining
module and its descendants -- not to siblings of a new `state` module.

That field-visibility change (private -> pub(crate), crate-internal only, no
external API change, behaviour-preserving) is the ONLY way to get mod.rs under
400, and it contradicts two explicit prompt instructions. This is a structural
decision escalated to the owner rather than made unilaterally.

## Still TODO (unblocked, not yet done -- will complete after the decision)
- shell/update.rs (1442) -- decompose the eframe update() frame loop into
  per-phase private helper methods in new submodules. (No visibility change
  needed; sibling impl blocks.)
- shell/render_chrome.rs (793) -- split theme / menu-bar / tab-bar render groups.
- shell/render_body.rs (437) -- extract one cohesive helper group.
- shell/nav_stack.rs (417) -- extract descriptor-reconstruction helpers.

## Scoped commands run so far (clean)
- `cargo check -p ff-desktop` -- clean (only unrelated ff-mdx profile notices).
- `cargo fmt` + `cargo fmt -- --check` -- clean (no diffs).
- `python tools/python/check_line_limits.py` -- mod.rs down to 480; the 4
  out-of-scope pre-existing files (main.rs 632, config_panel/render.rs 590,
  tab_state.rs 404, menu_workspace/loader.rs 404) remain, as expected.

(clippy and the test suite will be run after the remaining splits; the #[test]
count baseline is captured below.)

## #[test] count: unchanged so far
No test was moved, renamed, deleted, weakened, or ignored. All splits moved
non-test code only.
