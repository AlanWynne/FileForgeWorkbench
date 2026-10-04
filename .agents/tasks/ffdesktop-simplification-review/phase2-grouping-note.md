# Phase 2 Task 2.1 -- WorkbenchShell god-struct grouping plan

Pure refactor, no behaviour change. Source of truth: crates/ff-desktop/src/shell/mod.rs
(struct `WorkbenchShell`, lines ~250-720; constructor `new_with_history_store`).

## Constraints honoured
- Notification channel (`notification_rx` / `notification_tx` / `notification_queue`
  + `notification_sender()`): KEPT intact as top-level fields (mandated by
  notification-system Req 3). Not deleted, not inlined.
- WorkspaceContext ShellRequest / ShellServices surface: lives in
  shell/workspace_context.rs, untouched (mandated by workspace-framework Req 2).
- `*_dir_override` fields are READ by non-test code (`themes_dir()`, `menus_dir()`,
  `keymaps_dir()`, `workspace_kinds_dir()`, `scrm_dir()` are `pub(super)`, not
  cfg(test)). Therefore the DirOverrides sub-struct is NOT cfg-gated; production
  simply holds `None`s.

## Groups introduced (new sub-structs in shell/state_groups.rs)

Field-name collision note: `first_interior_id` / `last_interior_id` also exist on
panel structs in `crate::*_panel` (their OWN fields). Those modules never touch
`WorkbenchShell`, and all WorkbenchShell references live under
`crates/ff-desktop/src/shell/*.rs`, so renaming within shell/*.rs is unambiguous.

### 1. DirOverrides (`self.dir_overrides.*`) -- 5 fields
Test-only directory overrides (production = None), grouped but NOT cfg-gated
because production methods read them.
- themes_dir_override
- menus_dir_override
- keymaps_dir_override
- workspace_kinds_dir_override
- scrm_dir_override

### 2. DetachSplitState (`self.detach_split.*`) -- 6 fields
Detached-window + split-region bookkeeping.
- floating_tabs
- detach_pending
- split_tab_drag
- split_leaf_rects
- region_cmd_ctx
- focused_region_menu_first

### 3. FocusState (`self.focus.*`) -- 8 fields
Interior/menu focus anchors and one-shot focus latches.
- command_field_focus_requested
- first_interior_id
- last_interior_id
- menu_first_id
- menu_last_id
- last_active_tab
- focus_first_interior_requested
- focus_last_interior_requested

## Left as top-level fields (deliberately NOT grouped)
Grouping these would hurt clarity or fight the heavy swap logic in
`with_workspace_context` (commands.rs) / per-region command contexts, or they are
the single large panel/engine states the task says may stay top-level:
- command/dispatch core: command_text, pending_command_line_outcome, open_error,
  scroll_amount, scroll_field_text, dispatch, cmd_registry, cmd_engine,
  command_line_history, command_line_in_progress, history_store,
  cursor_context_snapshot, focused_menu_option (swapped field-by-field by
  WorkspaceCommandContext; grouping adds churn with no clarity gain)
- panel/Context states (config_panel, theme_editor_panel, ...): large, each a
  clear single-responsibility field already
- notification channel + queue (MANDATED intact)
- misc engines/managers (find_manager, nav_manager, exclude_manager,
  shell_engine, session, etc.)

## Mechanics
- New file shell/state_groups.rs: three plain `#[derive(Default)]`-able data
  structs (DetachSplitState and FocusState derive Default where helpful;
  DirOverrides derives Default). Must stay < 400 non-test lines (it is tiny).
- mod.rs: replace the 19 flat fields with 3 sub-struct fields; update constructor.
- Update references under crates/ff-desktop/src/shell/*.rs:
  `self.<field>` -> `self.<group>.<field>` for the 19 moved fields only.
- Update test references in shell/tests_*.rs likewise
  (`shell.<field>` -> `shell.<group>.<field>`, `state().<field>` etc.).
- Panel modules (crate::*_panel) are NOT touched (their same-named fields are
  unrelated).

## Verify
cargo fmt; cargo check -p ff-desktop; cargo clippy -p ff-desktop --tests;
cargo test -p ff-desktop. Known-acceptable: B048 FFWB_HISTORY_PATH /
FFWB_USER_CONFIG_PATH multithread env races (pass under nextest).
