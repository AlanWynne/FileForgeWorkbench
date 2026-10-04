# Phase 2 ff-desktop shell simplification -- behaviour-preserving refactor

The Phase 2 refactor groups the ~90-field `WorkbenchShell` god-struct into three
plain sub-structs (`FocusState`, `DetachSplitState`, `DirOverrides` in
`shell/state_groups.rs`), moves the struct definition out of `shell/mod.rs` into
`shell/state.rs`, and splits the oversized shell source files by moving cohesive
groups into new sibling submodules declared with `mod ...;` in `shell/mod.rs`.
The work is a pure, gate-exempt, behaviour-preserving refactor in the
uncommitted working tree. I verified correctness by reading the extracted code
against the committed (HEAD) originals, confirming frame order, dispatch order,
framework surfaces, visibility, file sizes, and the test set.

Watch for: the only material finding is a documentation-rule (`documentation.md`:
Rust source MUST be plain ASCII) issue -- 77 non-ASCII comment lines remain
across shell `.rs` files, several of them in NEW files created by this split.
These are PRE-EXISTING debt carried verbatim (HEAD already had 121+ such lines in
the source files that were split), not newly authored, and they are comment-only
so they do not affect behaviour. Treated as a non-blocking nit for this
behaviour-preserving refactor; flagged for a follow-up ASCII sweep. One minor
over-widening nit (`config_value_to_toml_value` and the grouped `focus` /
`detach_split` fields widened to `pub(crate)` where `pub(super)` suffices).

**Verdict**: APPROVED

## High-level view

The `update()` frame loop is behaviourally identical. The large inline blocks
(one-shot startup; Ctrl+Scroll zoom / DPI; the Tab-order Boundary_Policy; the
floating-tab viewport loop; the dialog/overlay cluster; the catalog-dialog match
arm) were each lifted verbatim into a dedicated method
(`run_startup`, `handle_zoom_and_dpi`, `handle_tab_order_focus`,
`render_floating_tabs`, `render_overlays`, `render_catalog_dialogs`) and are
called from `update()` in the SAME order and SAME positions they ran inline. The
early `return`s inside the AllocateDataset arm still end the frame correctly
because `render_catalog_dialogs` is the last call of `render_overlays`, which is
the last statement of `update()`.

The two mandated framework surfaces survive untouched. The notification mpsc
channel (`notification_rx` / `notification_tx` on the struct, built in
`construct.rs`, exposed by `notification_sender()` in `actions.rs`) and its
per-frame drain in `update()` are intact; the `WorkspaceContext` /
`ShellServices` / `ShellRequest` surface and the single `render_workspace_context`
/ `apply_interior_focus` focus-latch path are intact.

The `handle_command` ladder keeps its exact dispatch order; the sub-ladders
(`try_commands_a` / `try_commands_b1` / `try_commands_b2` / `try_commands_c`,
plus the fastpath and menu-name stages) are called in the original sequence. The
ladder is deliberately NOT converted to a table (Phase 3) -- out of scope.

The field-group rewrite is mechanical: `self.<field>` became
`self.focus.<field>` / `self.detach_split.<field>` / `self.dir_overrides.<field>`
at every call site, with the sub-structs carrying the original field types and
semantics. Field visibility is unchanged except for the minimum widening a
sibling module needs (`files_panel`, `config_handle` read by the new
`update_*` modules).

Every in-scope shell `.rs` file is <=400 non-test lines; only the 4 known
out-of-scope files remain over (main.rs 632, config_panel/render.rs 590,
tab_state.rs 404, menu_workspace/loader.rs 404). The shell test set is conserved
exactly: 488 `#[test]`/`#[tokio::test]` functions in HEAD and 488 in the working
tree (delta 0); the 454-test `tests.rs` split cleanly into the `tests_*.rs`
siblings with no test dropped, renamed, or `#[ignore]`d.

<details>
<summary>Issues (2)</summary>

1. **Non-ASCII comment bytes in Rust source (nit, non-blocking)** -- 77 lines
   across shell `.rs` files (including new split files `commands_ladder_a/b/b2/c`,
   `render_body_arms`, `render_command_line`, `render_status`, `render_tab_bar`,
   `render_theme`) contain em dashes, en dashes, box-drawing separators, a
   rightwards-arrow glyph, and the `multiply-X` / `black-circle` glyphs, violating
   `documentation.md` (Rust = plain ASCII).
   Pre-existing debt moved verbatim (HEAD source had 121+ such lines), comment-only,
   no behaviour impact. Recommend a separate ASCII sweep; do not block this refactor.
2. **Minor visibility over-widening (nit, non-blocking)** -- `focus` and
   `detach_split` grouped fields, and `shell::helpers::config_value_to_toml_value`,
   are `pub(crate)` though read only within `shell/` (`pub(super)` would suffice).
   No external reader found. Tighten opportunistically; not required.

</details>

<details>
<summary>Details</summary>

### 1. No observable behaviour change in the `update()` frame loop

I diffed the new `shell/update.rs` `update()` against `HEAD:shell/update.rs`.
The frame order is identical:

```
run_startup()                      <- was the inline `if !self.started { ... }` block
should_close check + early Close return
notification channel drain          (unchanged, in place)
active-tab focus re-arm (CR-CH-023 Req 16.1a)
pending_open / Ctrl+S / Ctrl+Shift+P / Ctrl+Shift+F
pending_new_pom / pending_new_file / pending_return_to_pom / pending_menu_option
detach_pending -> push FloatingTab
drop stale FloatingTabs
file-backed theme hot-reload + OS dark/light follow
apply_theme(ctx)
handle_zoom_and_dpi(ctx)            <- was the inline zoom/DPI block
handle_tab_order_focus(ctx)         <- was the inline Boundary_Policy block
render_menu_bar (if !split)
automation.begin_frame
render_tab_bar
render_title_line (if !split)
render_command_field (if !split)
render_key_label_bar
render_status_bar
refresh_cursor_context_snapshot
resolve_function_key_command -> dispatch_key_command
render_central_panel(ctx)
render_floating_tabs(ctx)           <- was the inline floating-tab for-loop
render_overlays(ctx)               <- was the inline dialog/overlay cluster
```

`render_floating_tabs` (`update_floating.rs`), `render_overlays`
(`update_dialogs.rs`), `run_startup` (`update_startup.rs`), and
`handle_zoom_and_dpi` + `handle_tab_order_focus` (`update_input.rs`) are verbatim
moves of the corresponding original blocks. The ONLY line-level differences are
the mechanical field-group rewrites applied project-wide
(`self.floating_tabs` -> `self.detach_split.floating_tabs`;
`self.first_interior_id` -> `self.focus.first_interior_id`;
`self.command_field_focus_requested` -> `self.focus.command_field_focus_requested`;
`self.detach_pending` -> `self.detach_split.detach_pending`), plus one
import-localisation in `run_startup` (`super::KeyBarScope::parse` ->
`KeyBarScope::parse` via `use`). No control flow, ordering, or condition changed.

Early-return correctness: `render_overlays` ends with `self.render_catalog_dialogs(ctx)`
as its last statement (`update_dialogs.rs`), and `render_overlays` is the last
statement of `update()`. The four early `return;` statements inside the
`AllocateDataset` arm of `render_catalog_dialogs` (`update_dialogs_catalog.rs`,
the allocation-error and DSN-parse-error paths) therefore end the frame exactly
as the original inline early returns did -- there is no code after the catalog
match arm in either the method or `update()`.

Focus / Tab order: `handle_tab_order_focus` preserves the full Boundary_Policy
(the `Boundary` enum, the `ctx.input_mut` Tab-consume-on-detection logic, the
File Explorer tree-transfer, the explorer Escape exit) byte-for-byte apart from
the `self.focus.*` field prefix. The `InteriorFocus` latch is still applied on the
single `render_workspace_context` / `apply_interior_focus` path in `render.rs`
(unchanged). Startup/session behaviour: `run_startup` is a verbatim move of the
`!self.started` block (CLI-file precedence, session restore, descriptor restore,
split-layout restore, default-catalog seeding, POM presence, menus-dir ensure).

### 2. Framework surfaces preserved

Notification mpsc channel (notification-system Req 1.1, 3.1):
- `state.rs:356` `pub(super) notification_rx: Receiver<Notification>`,
  `state.rs:361` `pub(super) notification_tx: SyncSender<Notification>`.
- `construct.rs:248` `let (notification_tx, notification_rx) = sync_channel(64);`
  with both stored on the struct.
- `actions.rs:19` `notification_sender()` returns `NotificationSender::new(self.notification_tx.clone())`.
- Per-frame drain still runs in `update()` (`update.rs:74-78`): the
  `while let Ok(n) = self.notification_rx.try_recv() { queue.push(n); }` block is
  in place, unchanged.

`WorkspaceContext` surface (workspace-framework Req 2): the `ShellServices` /
`ShellRequest` / `InteriorFocus` / `WorkspaceContext` types are intact and still
dispatched through `render.rs` (`render_workspace_context`, `apply_shell_requests`,
`apply_interior_focus`). Every migrated Context (`config_panel`, `help_context`,
`event_log_panel`, `macro_library_panel`, `plugin_manager_panel`,
`keys_editor_panel`, `menus_editor_panel`, `scrm_viewer_panel`,
`command_config`, `theme_editor_panel`, `search_results_panel`,
`kinds_editor_panel`) still implements the trait against `ShellServices`. No
surface was deleted or inlined away.

### 3. Visibility widening is minimal

Comparing `HEAD:shell/mod.rs` field visibility against `shell/state.rs`: the
fields that are `pub(crate)` in the new `state.rs` were ALREADY `pub(crate)` in
HEAD (active_workspace, pending_workspace_open, show_unsaved_workspace_dialog,
command_store, kind_registry, the editor-panel states, shell_engine,
pending_external, pending_menu_option, palette_state, recent_palette_commands,
search_results_panel, scroll_amount, scroll_field_text, notification_queue,
session_start, automation, and the former `focus_first/last_interior_requested`).
These are not new widening.

Originally-private fields now `pub(super)` (e.g. `files_panel`, `config_handle`)
are genuinely read by the new sibling modules -- `files_panel` by
`update_startup.rs`, `update_input.rs`, `update_dialogs_catalog.rs`;
`config_handle` by the theme hot-reload path. These widenings are warranted.

Over-widening nit (non-blocking): the two new grouped fields `focus` and
`detach_split` are declared `pub(crate)`, and `shell/state_groups.rs` declares
`FocusState` / `DetachSplitState` / `DirOverrides` `pub(crate)`, but a search for
`.focus.` / `.detach_split.` / `.dir_overrides.` OUTSIDE `shell/` returns no
matches -- every reader is a `shell` sibling, so `pub(super)` would suffice.
Likewise `shell::helpers::config_value_to_toml_value` was widened from
`pub(super)` (HEAD) to `pub(crate)`, yet all four readers (helpers.rs,
mod_helpers.rs, the mod.rs cfg(test) re-export, tests_menu_workspace.rs) are
`super::`-reachable. Confirmed, non-blocking.

### 4. 400-line rule and ASCII

Line limits: `tools/python/check_line_limits.py` reports only the 4 known
out-of-scope files over 400 non-test lines (main.rs 632, config_panel/render.rs
590, tab_state.rs 404, menu_workspace/loader.rs 404). Every new split file and
every touched in-scope shell file is <=400 non-test lines. Confirmed OUT OF SCOPE
per the brief; not blocking.

ASCII (confirmed finding, non-blocking nit): a scan of
`crates/ff-desktop/src/shell/**/*.rs` found 77 lines containing bytes > 0x7F,
including em dashes (U+2014), en dashes (U+2013), box-drawing (U+2500 family),
a rightwards arrow (U+2192), and the U+2715 / U+25CF button glyphs. Examples:
`render.rs:274` a close-button glyph literal; `render_tab_bar.rs:71` a
modified-marker glyph in `format!(...)`; numerous box-drawing separators and
em-dash `// Validates:` comments in `commands_ladder_a/b/b2/c.rs`,
`render_body_arms.rs`, `render_command_line.rs`, `render_status.rs`,
`render_theme.rs`, `update.rs`, `mod.rs:1`.

This is PRE-EXISTING debt carried verbatim, not introduced by the refactor: the
committed HEAD versions of the files that were split already contained these
bytes (mod.rs 31, render.rs 18, commands.rs 35, update.rs 15, render_chrome.rs
16, helpers.rs 6 -- 121+ lines total), and the split distributed the same comment
blocks into the new sibling files. All occurrences are in comments or button
glyph literals that are visual-only; none affects behaviour, and the build is
confirmed clean. `documentation.md` says to replace non-ASCII "whenever a file is
touched for another reason", so a strict reading flags the new files -- but for a
behaviour-preserving verbatim-move refactor this is a documentation nit, not a
correctness defect. Recommend a dedicated ASCII sweep as a follow-up (the two
button glyph literals in `render.rs` / `render_tab_bar.rs` are worth converting
to ASCII or named constants too).

### 5. No test lost or weakened

Shell `#[test]`/`#[tokio::test]` count is conserved exactly: HEAD = 488, working
tree = 488 (delta 0). The former monolithic `tests.rs` (454 tests) split into
`tests_scrm` (16) + `tests_session` (25) + `tests_split_detach` (32) +
`tests_nav` (44) + `tests_misc` (71) + `tests_command` (83) + `tests_focus` (87)
+ `tests_menu_workspace` (96) = 454, with the other 34 unchanged in
`update.rs` (6), `reset_bare.rs` (17), `command_line_outcome.rs` (6),
`workspace_context.rs` (4), `menus_editor.rs` (1). No `#[ignore]` exists anywhere
in the shell tests. No test was dropped, renamed, or weakened by this change.

### Out-of-scope items confirmed as not findings

- `handle_command` remains a ladder (sub-ladders `try_commands_a/b1/b2/c` called
  in the original order); the table conversion is deferred to Phase 3. Not flagged.
- The 4 pre-existing oversized files (main.rs, config_panel/render.rs,
  tab_state.rs, menu_workspace/loader.rs) are known debt. Not flagged.
- `panel_layout.rs` deletion is a safe dead-code removal: it had zero references
  in the source and `main.rs` only dropped its `mod panel_layout;` declaration.
- `tab_state.rs` changes are comment-only (documenting dead-code constructors).
- `editor_panel/mod.rs` moved `GUTTER_CHAR_WIDTH` under `#[cfg(test)]` (test-only
  reader) and fixed one comment arrow; no behaviour change.

</details>

<details>
<summary>File map</summary>

New shell submodules (verbatim moves / field-group rewrites):
`state.rs` (WorkbenchShell struct), `state_groups.rs` (FocusState /
DetachSplitState / DirOverrides), `construct.rs`, `handlers.rs`, `mod_helpers.rs`,
`workspace_io.rs`, `titles.rs`, `actions.rs`, `types.rs`, `nav_reconstruct.rs`;
render split: `render_body.rs`, `render_body_arms.rs`, `render_command_line.rs`,
`render_nav.rs`, `render_nav_expand.rs`, `render_nav_ops.rs`, `render_split.rs`,
`render_split_region.rs`, `render_status.rs`, `render_tab_bar.rs`,
`render_theme.rs`; commands split: `commands_fastpath.rs`, `commands_ladder_a.rs`,
`commands_ladder_b.rs`, `commands_ladder_b2.rs`, `commands_ladder_c.rs`,
`commands_menu.rs`, `commands_scrm.rs`, `commands_session.rs`, `commands_theme.rs`;
update split: `update_startup.rs`, `update_input.rs`, `update_floating.rs`,
`update_dialogs.rs`, `update_dialogs_catalog.rs`, `update_keys.rs`; test split:
`tests_command.rs`, `tests_common.rs`, `tests_focus.rs`,
`tests_menu_workspace.rs`, `tests_misc.rs`, `tests_nav.rs`, `tests_scrm.rs`,
`tests_session.rs`, `tests_split_detach.rs`.

Modified: `shell/mod.rs` (thinned to coordinator + mod decls + re-exports),
`shell/commands.rs` (ladder chains to sub-ladders), `shell/render.rs`,
`shell/render_chrome.rs`, `shell/update.rs`, `shell/nav_stack.rs`,
`shell/helpers.rs`, `shell/keys_editor.rs`, `shell/kinds_editor.rs`
(field-group rewrites), `main.rs` (dropped `mod panel_layout;`),
`editor_panel/mod.rs`, `tab_manager.rs`, `tab_state.rs`.

Deleted: `shell/tests.rs` (10999 lines, split into tests_*.rs),
`panel_layout.rs` (dead code).

Full diff: `git -C c:\workspace\VSC\FileForgeWorkbench diff HEAD` plus the
untracked new files (`git status --short`).

</details>
