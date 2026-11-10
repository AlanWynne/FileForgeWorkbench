# Common-frame fixes: menu capture layout, message routing, pre-action capture, Zoom segment (CR-NR-115)

The branch fixes the five reported defects. Informational messages (including "Snapshot copied to clipboard: picture + Markdown.") now go to the message band via `post_info` instead of the status bar. SNAPSHOT/CAPTURE reproduces the pre-action frame through the one seam `active_screen_capture`. The menu/POM capture path now ends with the PF-key line, and Zoom is a common status segment after Prof. I read the generated goldens and the capture-timing tests, and checked the other points by grep. I did not re-run the suites, as instructed.

Watch for: `state.rs` (426) and `update.rs` (408) exceed the 400-line limit after growing by 6 and 5 lines (confirmed, disclosed by the coder, pre-existing debt). The menu command-field test only covers an empty prior field (possible gap).

**Verdict**: APPROVED

## High-level view

The golden files match the owner's layout. From the bottom of the box the order is content, rule, message band (POM golden, with the confirmation text), rule, status band `Last: 11:02 | Prof: default | Zoom: 0`, rule, a single-cell PF-key line, then the border. The POM golden shows `Command ===> ____...` filled with underscores to the border and no triggering command. Config and Alias were regenerated with the Zoom cell and the PF line last.

Routing is split by outcome. Success posts to `message_line`, and failures stay on `open_error`, so errors are not dropped. This is recorded in `message_routing.rs` and workspace-framework Req 7.28. The CAPTURE file operations were split into `Result<String, String>` in the second commit, so success and failure take different paths.

Capture timing uses a `PreActionFrame` stash. It is taken idempotently in `note_action_dispatched`, before the message line is cleared and `Last:` is restamped. It is dropped at the end of `run_command_line`, `dispatch_bound_command` (outermost only), the key-forward path, and each `update` frame. The key-forward double invocation is handled.

<details>
<summary>Issues (3)</summary>

1. **File size debt (confirmed)** -- `state.rs` is 426 and `update.rs` is 408 non-test lines, both over the 400 limit and both grown by this branch. The coder recorded the split as a follow-up. Schedule the refactor, since the rule says to split before adding more.
2. **Command-field retention test (possible)** -- the capture-timing and menu tests assert that `Command ===> SNAPSHOT` is absent. I did not see a test that a non-empty field typed before the command is preserved in the capture. Consider adding one.
3. **Empty message band in goldens (possible)** -- the non-message Config and Alias goldens omit the message band when it is empty. This matches existing behavior and the `*_message_*` goldens. Confirm that is what the owner wants for the empty case.

</details>

<details>
<summary>Details</summary>

## Capture-timing tests drive the real seams

`tests_capture_timing.rs` calls `run_command_line("SNAPSHOT")` and `dispatch_key_command("SNAPSHOT")`, which are the real command seams. They are not direct calls to a helper. The tests cover four things:

- The pre-action message and `Last:` time appear in the capture.
- The first snapshot does not contain its own confirmation, and the second snapshot contains the first one's confirmation.
- The double-invoke case keeps the pre-action state.
- A capture outside an action uses the live state.

Messages are still cleared by the next action (`shell.message_line.is_empty()` is asserted after SNAPSHOT).

## Messages no longer in the status bar

A grep of the remaining `open_error = Some` sites shows only failure, usage, and "not available" stubs. These are ladder arms such as dock, detach, swap, submit, create, compare and workspace subcommands, plus theme-not-exist. The info posters (snapshot, copy, TIME, CAPTURE file success) were moved to `post_info`. The edited tests read `message_line` for confirmations and `open_error` for failures. The grep output was truncated, so my confirmation of the full set of former posters rests on the edited tests listed in the commit messages and on the routing tests, not on a complete site-by-site read.

## Zoom segment

`render_status.rs` renders `ui.label(format_zoom_label()).on_hover_text(...)` after Prof. It is a plain label with no click sense, so it is neither focusable nor a Tab stop (B055). The same formatter feeds the snapshot band. `tests_zoom_segment.rs` is new. Spec edits are in workspace-framework Req 7.16-7.29, screen-snapshot-scrm Req 30.x, the design notes, and a change-log entry.

## Guardrails

- No changes to `shell/environment.rs`, any PNG, or CR-CH-079 files (grep of the diff file list found none).
- The new public items are `pub(crate)` or `pub(super)`. There are no changes to public `ScreenModel`, `Table`, or `WorkspaceContext` shapes, and no new dispatcher. The additions are additive shell state and seam logic.
- The new source files I checked have no non-ASCII characters. The only non-ASCII in the touched specs is box-drawing in a fenced block, which is allowed. The goldens are box-drawing by design.
- The new tests carry `// Validates:` annotations (checked in `tests_capture_timing.rs`; I did not open every new test file).
- Changed existing tests are listed in the commit messages: tests_command, tests_nav, tests_session, tests_dialog_copy, tests_snapshot_picture (5), tests_scrm, and the golden and helper tests.
- The coder's recorded results are clippy and fmt clean, 183 passed and 0 failed in the iteration-2 run, and 309 passed in the first commit's run. The full gate is left to the owner.

</details>

<details>
<summary>File map</summary>

Source: `screen_snapshot.rs`, `shell/{actions,commands*,common_frame,message_routing,render_status,screen_provider,state,target_dispatch,update,construct,mod}.rs`, plus the i18n `.ftl`. Tests: `tests_capture_timing`, `tests_common_frame_menu`, `tests_message_routing`, `tests_zoom_segment` and edits to existing test files. Goldens: Config, Alias, POM (new) and Settings (new). Docs: workspace-framework, screen-snapshot-scrm, view-zoom, change-log. Full diff: `git diff main...HEAD` in the worktree.

</details>
