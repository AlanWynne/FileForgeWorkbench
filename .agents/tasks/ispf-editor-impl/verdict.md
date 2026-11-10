# Review verdict -- ISPF editor FEAT-001/002/003 (branch ispf-editor-impl)

Verdict: APPROVED

Scope reviewed: `git diff 3db217a...HEAD` on `.worktrees/ispf-editor-impl`
(4 commits: FEAT-001 blocks, FEAT-002 arrow nav, FEAT-003 single-chrome, and the
docs-only CR-CH-077 spec reconcile). Task artifacts read: plan.md, RESUME-HANDOFF.md,
features/FEAT-001.json, FEAT-002.json, FEAT-003.json. Recorded scoped verification
read from the FEAT findings (agents compile-verify scoped per gate-policy; the full
`cargo gate` is the owner's pre-merge run and is correctly noted as pending). I did
not re-run suites.

## Framework / seam conformance -- CONFIRMED

- NO new dispatch path. `apply_alias_command_line` (shell/alias_editor_line_commands.rs)
  still falls through to the single FFCMD front door `dispatch_command_string` for
  SAVE/END/RETURN/REFRESH; FILTER/RESET/SORT stay panel-local as before. The only
  change (shell/commands_line.rs `run_command_line`) ROUTES the shell's app-level
  command field into that existing handler when `KindTag::AliasEditor` is active --
  it adds no parallel dispatcher.
- NO second navigation stack; no navigate_to/descriptor/save-seam change. The diff
  touches no nav_stack or descriptor code. Save remains the ISPF explicit-commit
  path reached via the FFCMD front door (SAVE/END/RETURN unchanged).
- NO ff-line-commands public-API change: `git diff --stat ... -- crates/ff-line-commands/`
  is EMPTY. block_apply.rs consumes only existing public items (PendingCommandStore,
  BlockPairValidator::validate_pair/check_overlaps, to_block_kind, BlockPair).
- Resolver `crates/ff-desktop/src/shell/environment.rs` is UNTOUCHED (empty diff
  for that path; file confirmed present).
- Single focus-latch + stable ids preserved. The surface still reports interior
  anchors via `render_filter_cells` -> `InteriorAnchors`; only the removed
  `command_line_id` trait method and the removed surface command field changed.
  Stable Act/field ids are unchanged (rows.rs keeps `row_control_id` ids; combo
  ids are now captured FRESH from the response per the B056 lesson, not guessed).
  `full_shell_alias_editor_first_tab_focuses_first_interior` recorded passing in
  all three FEATs -- the B056-B059 phantom-stop class does not reopen.

## FEAT-001 (CR-CH-076) block commands -- CONFIRMED

- Block set is EXACTLY RR/DD/CC/MM: `block_enabled` matches only
  DeleteBlock/RepeatBlock/CopyBlock/MoveBlock; every other `*Block` is rejected.
- Single R/Rn preserved alongside RR: `single_r_vs_block_rr_distinct` test.
- MM disabled on Alias: alias() keeps `move_: false`; resolve_blocks drops a
  MoveBlock with a "not enabled" notice and retains its markers when `!set.move_`
  (`block_mm_disabled_when_move_off`).
- Unterminated / too-many -> non-blocking "block not closed" notice, markers
  RETAINED (not added to plan.clear), batch continues; well-formed commands in the
  same batch still apply (`block_lone_dd_retained_notice_batch_continues`,
  `block_too_many_dd_markers_no_change`).
- Overlap -> non-blocking "overlapping block ranges rejected" notice, nothing
  applied (`block_overlapping_dd_and_rr_rejected`). Implementation drops all
  resolved pairs on overlap -- a conservative superset of "drop the overlapping
  pair", equivalent for the tested case; not a blocker.
- CC/MM require an A/B target: resolve_blocks consumes the first unconsumed target
  (shared `consumed_targets` with single C/M); with none, a "pending a target (A/B)"
  notice is pushed and no structural change made.
- Resolved pairs fold into the existing stable-index before/after/removed rebuild;
  clones snapshotted while original indices are valid.

## FEAT-002 (CR-CH-078) arrow navigation -- CONFIRMED

- Pure `arrow_target` clamps at every edge with no wrap; Up/Down move rows,
  Left/Right move cols; degenerate/out-of-range returns current (9 table tests).
- `apply_arrow_walk` consumes the pressed arrow via `ui.input_mut(consume_key(NONE,key))`
  BEFORE egui traverses, then `request_focus` the clamped target -- consume-before-egui
  pattern, not fighting egui focus.
- Self-guards: the walk only acts when a MATRIX cell holds focus (find_cell), so
  the command field keeps its CR-NR-096 Up/Down history gesture.
- Tab/Shift+Tab and all stable ids untouched; `full_shell_alias_editor_arrow_down_and_right_walk_the_matrix`
  plus the two regression tests recorded passing.

## FEAT-003 (CR-CH-077) single-chrome (option A) -- CONFIRMED

- Shared surface draws NEITHER a centred title NOR a `Command ===>` line:
  render_ispf_table has the title/command-field block removed; `title` dropped from
  TableRenderParams; `command_line_id` removed from IspfTableRender; the unreachable
  TableAction::CommandLine / AliasEditorAction::CommandLine removed.
- Shell owns the single Title + single Command ===> line; the Alias Context routes
  the shell command field into `apply_alias_command_line`.
- Live panel invariant: `full_shell_alias_editor_is_single_chrome` asserts exactly
  ONE "Command ===>" node, ZERO "FileForge Workbench Alias Editor" heading, ONE
  "Alias Editor" title.
- Snapshot agrees: `full_shell_alias_screen_model_carries_command_line_and_header`
  asserts the captured model carries the command line and the header
  [Act, CE, Command, Alias, Parameters, Description] + the seeded row (ONE Title ->
  Command ===> -> header -> rows), matching the live surface.

## Spec reconcile -- CONFIRMED

- ispf-table-editor Req 10.1/10.2 rewritten IN PLACE to the option-A shell-level
  wording (shell Title_Line + shell command field are the single chrome; Context
  must not draw duplicates; shell routes into the command-environment handler;
  snapshot agrees), each carrying a dated reconciliation NOTE.
- command-configurator Req 7.12 rewritten IN PLACE to the same option-A wording
  with the reconciliation note.
- docs/status/change-log.md carries a WORDING RECONCILIATION entry under CR-CH-077
  describing the option-A divergence and the in-place criteria edit.

## Hygiene -- CONFIRMED

- File sizes (non-test): line_commands.rs ~319 before its test module; render.rs
  ~258 (no test module); block_apply.rs 194 (no test module); arrow_nav.rs 173
  total (inline tests, ~70 non-test); rows.rs/host.rs/render.rs panel files small.
  All <= 400 non-test lines.
- ASCII-only: grep for `[^\x00-\x7F]` across the new/changed ispf_table_editor and
  alias_editor_panel source returns no matches.
- TDD annotations present: `// Validates:` on every new unit/full-shell/snapshot
  test (arrow_target, block_*, block_opt_in_flag_is_per_host, the full-shell focus
  and snapshot tests).

## Notes (non-blocking)

- The full `cargo gate` on this branch has not been run; gate-policy makes that the
  owner's mandatory pre-merge checkpoint. Recorded agent verification is scoped-only,
  as required. This is expected state, not a blocker on the review.
