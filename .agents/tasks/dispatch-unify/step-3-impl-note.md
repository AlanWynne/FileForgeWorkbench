# B080 Step 3 -- Function-target family: implementation note

Step 3 of the unified command dispatch migration (B080, Phase 3 task 7).
Owner-approved as a conformance fix; Req 8.4 behaviour-preservation is mandatory.
Finished DIRECTLY by Kiro (in-line) after the Step 3 workflow wedged on the
recurring infra/token dropout -- the wedged coder had already written the
per-verb analysis into the ladder-arm comments before it stalled; Kiro verified
and completed.

## Outcome: ALL FOUR Function-family verbs DEFERRED; zero migrated

EXIT family (EXIT/QUIT/=X/X/LOGOFF): NOT part of Step 3 -- it lives in the
prelude (`try_exit_family`, Step 2) at its original precedence (the A1 pin). Left
untouched. Pinned by `exit_family_from_non_menu_context_dispatches_file_exit_not_pom_return`.

EDIT / BROWSE / VIEW / CLOSE: DEFERRED, left on the ladder exactly as today, with
explanatory comments added (comment-only; zero behaviour change). Root-cause
reason (identical for all four):

- EDIT (commands_ladder_a.rs), BROWSE / VIEW (commands_ladder_c.rs) open a
  file/dataset via `dispatch.execute_command("file.open", { path })`, carrying
  the path in CommandParams to the registered FileOpenHandler.
- The Function-target dispatch arm in `target_dispatch.rs::dispatch_command_target`
  routes `CommandTarget::Function { command_id, .. }` by calling
  `self.handle_command(command_id)` -- which carries NO params. Classifying EDIT
  to `Function { "file.open" }` would DROP the path param and never reach the
  handler. That is a Req 8.4 observable-behaviour regression, so EDIT/BROWSE/VIEW
  stay on the ladder.
- CLOSE is a shell operation (`tabs.close_tab`), not a registered Command_ID, so
  it does not fit the Function model (no command_id to resolve to). Stays on the
  ladder.

This is the CORRECT Step-3 result per the brief ("if a verb does not fit cleanly,
LEAVE IT ON THE LADDER and report; a partial step is correct"). Migrating any of
them would require the Function dispatch arm to carry params -- i.e. route a
resolved `Function { command_id, params }` through `execute_command(command_id,
params)` instead of `handle_command(command_id)`. That is a FRAMEWORK ENHANCEMENT
(param-carrying Function dispatch), not in Step 3 scope; tracked below.

## Follow-up (not Step 3)

To migrate EDIT/BROWSE/VIEW later, `dispatch_command_target`'s Function arm must
dispatch `execute_command(command_id, params)` so the Req 9.2 `arg`/`path` param
survives. The `function_target_with_arg` helper (dispatch.rs) already folds `arg`
into params; it just has no param-carrying consumer yet. Consider as a Step 4+ or
post-migration enhancement. Until then these verbs correctly stay on the ladder.

## Verification (scoped, Kiro-run)

- `cargo clippy -p ff-desktop --tests`: clean (only the unrelated ff-mdx profile
  notices).
- `cargo test -p ff-desktop -- --test-threads=1`: 943 passed, 0 failed.
- No ladder arm removed; EXIT prelude placement intact; builtin_workspace_target_for
  unchanged (still only the Step-2 CustomWorkspace/nav family; no over-claim).
- Comment-only edits to commands_ladder_a.rs (EDIT) and commands_ladder_c.rs
  (BROWSE/VIEW) documenting the deferral. ASCII; files under 400 non-test lines.

## Status

Step 3 COMPLETE (outcome: deferral of the whole Function family, by design).
Steps 4-7 remain TODO. Not committed; no worktree.
