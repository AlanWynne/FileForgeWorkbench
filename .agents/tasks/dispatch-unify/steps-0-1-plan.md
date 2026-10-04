# Implementation Plan -- Unified Command Dispatch, STEPS 0 and 1 ONLY (B080, Phase 3 task 7)

Scope: CONFORMANCE fix against EXISTING command-framework criteria
(Req 2.1 / 2.7 / 8.3 / 8.4 / 9.2 / 9.7). Requirements gate is CLOSED; this is
NOT a new requirement. Work DIRECTLY in the main workspace at
`c:\workspace\VSC\FileForgeWorkbench` -- NO git worktree, branch, or commit.

Steps 0 and 1 are PURE INDIRECTION plus a COMPUTED-BUT-NOT-WIRED verb/arg split:
ZERO observable behaviour change. The 454 shell tests in
`crates/ff-desktop/src/shell/tests_*.rs` are the Req 8.4 backstop and must stay
green.

STEPS 2-7 ARE OUT OF SCOPE and remain TODO (separate runs): Step 2
`builtin_workspace_target` classification + ladder-arm removal, Step 3 Function
family, Step 4 manager families, Step 5 split/detach/swap + workspace, Step 6
standalone + scrm, Step 7 retire the empty ladder segments. This plan does NOT
remove any ladder arm, does NOT implement `builtin_workspace_target`, and does
NOT touch the notification channel or `ShellServices` surfaces.

## Verified facts (confirmed against source on this pass)

- TYPED seam: `run_command_line` (`crates/ff-desktop/src/shell/commands.rs:27`)
  wraps `begin_command_line()` ... `self.handle_command(&original)` ...
  `finish_command_line(&original)`. The inner call to change is
  `self.handle_command(&original)`; the wrap stays in `run_command_line`.
- KEY seam: `dispatch_key_command` (`target_dispatch.rs:116`) field-merges then
  calls `dispatch_bound_command` (`target_dispatch.rs:89`), whose `FallThrough`
  arm calls `self.handle_command(command)`. That `FallThrough` call is the one
  to reroute. `dispatch_key_command`'s merge + wrap stay unchanged.
- Verb/arg rule: `verb_arg(cmd, verb)` (`helpers.rs:242`) -- `split_once` on
  first whitespace, `head.eq_ignore_ascii_case(verb)`, case-PRESERVED trimmed
  remainder (B062). It takes a KNOWN verb; Step 1 needs a split that EXTRACTS
  the verb token, so Step 1 adds a sibling split helper in `helpers.rs` built on
  the SAME `split_once(char::is_whitespace)` + `trim` logic (not a divergent
  reimplementation).
- Function target shape: `ff_command::CommandTarget::Function { command_id:
  String, params: TargetParams }` where `TargetParams = BTreeMap<String,
  TargetValue>` and `TargetValue::String(String)` carries a string
  (`crates/ff-command/src/command_target.rs`). The Req 9.2 `arg` is inserted as
  `params.insert("arg".to_string(), TargetValue::String(arg.to_string()))`.
- `mod` block in `shell/mod.rs` is alphabetical; `dispatch` sits between
  `construct` (line 49) and `external_adapter` (line 50).
- Test harness: `make_shell()` (`tests_common.rs:99`), set `shell.command_text`,
  call `run_command_line` / `dispatch_key_command`, assert shell state -- the
  established pattern in `tests_command.rs` (e.g. lines 627-697).

---

- [ ] 1. Create the single front-door file `shell/dispatch.rs` (Step 0 body = pure indirection).
      Create `crates/ff-desktop/src/shell/dispatch.rs` containing exactly ONE
      `impl WorkbenchShell` block with a single method
      `pub(super) fn dispatch_command_string(&mut self, raw: &str)`. For Step 0
      its body is PURE INDIRECTION: `self.handle_command(raw);` and nothing else.
      Add a `//!` module doc and a `/// Validates: command-framework Requirement
      2.1` doc comment on the method explaining it is the single shell-side front
      door that Steps 1-7 will grow; no `resolve_target` call, no split in Step 0.
      ASCII only; file must stay <= 400 non-test lines (it is ~15 lines here).
      Files: `crates/ff-desktop/src/shell/dispatch.rs`
      Verify: `cargo check -p ff-desktop` compiles (method is unused until step 2
      wires callers -- if an unused-method warning appears under
      `cargo clippy -p ff-desktop --tests`, it is removed by step 2 of THIS plan
      which adds the callers in the same change set; keep steps 1 and 2 in one
      edit batch before running clippy).

- [ ] 2. Declare the module and route the TWO outermost submit callers through it.
      In `crates/ff-desktop/src/shell/mod.rs` add `mod dispatch;` in the
      alphabetical block between `mod construct;` (line 49) and
      `mod external_adapter;` (line 50). Then reroute ONLY the two outermost
      submit sites:
      (a) In `commands.rs` `run_command_line` (line ~27): change
      `self.handle_command(&original);` to
      `self.dispatch_command_string(&original);` -- the `begin_command_line()` /
      `finish_command_line(&original)` wrap STAYS in `run_command_line`.
      (b) In `target_dispatch.rs` `dispatch_bound_command` (line ~89): change the
      `ResolveOutcome::FallThrough => self.handle_command(command),` arm to
      `ResolveOutcome::FallThrough => self.dispatch_command_string(command),`.
      Do NOT change `dispatch_key_command`'s field-merge or its wrap. Do NOT
      reroute any inner/recursive `handle_command` call -- the `resolve_pom_option_key`
      recursion in `commands.rs`, the chained-fastpath segment re-dispatch in
      `commands_fastpath.rs`, and the AUTONUM->NUMBER redirect in
      `commands_ladder_c.rs` all stay DIRECT `self.handle_command(...)` calls
      (inner re-dispatches, decisions D4/D8).
      Files: `crates/ff-desktop/src/shell/mod.rs`,
      `crates/ff-desktop/src/shell/commands.rs`,
      `crates/ff-desktop/src/shell/target_dispatch.rs`
      Verify: `cargo check -p ff-desktop` compiles; then
      `cargo test -p ff-desktop -- --test-threads=1` -- all 454 shell tests pass
      (pure indirection, no behaviour change).

- [ ] 3. Add the Step 0 regression test proving the typed path routes through the front door.
      In `crates/ff-desktop/src/shell/tests_command.rs`, add
      `typed_submit_routes_through_dispatch_command_string`. Model it on the
      existing `run_command_line` tests (e.g. `tests_command.rs:690-697`): build
      `make_shell()`, run a representative typed command whose effect is
      observable via shell state through `run_command_line`, and assert the same
      resulting state as before the change. Use a command already exercised in
      this file so the assertion is grounded, e.g. `shell.handle_command("START")`
      to create a second tab, set `shell.command_text = "SWAP 1".to_string()`,
      call `shell.run_command_line("SWAP 1")`, and assert `shell.command_text ==
      ""` (success-clear) plus the active-tab change the SWAP produced -- i.e. the
      front door delegates identically to the old direct `handle_command`.
      Annotate `// Validates: command-framework Requirement 2.1`.
      Files: `crates/ff-desktop/src/shell/tests_command.rs`
      Verify: `cargo test -p ff-desktop typed_submit_routes_through_dispatch_command_string -- --test-threads=1`
      passes; full `cargo test -p ff-desktop -- --test-threads=1` stays green.

- [ ] 4. STEP 1: compute the single verb/arg split in the front door WITHOUT changing behaviour.
      In `crates/ff-desktop/src/shell/helpers.rs`, add
      `pub(super) fn split_verb_arg(cmd: &str) -> (&str, &str)` built on the SAME
      logic as `verb_arg` (helpers.rs:242): `let c = cmd.trim(); let (head, rest)
      = c.split_once(char::is_whitespace).unwrap_or((c, "")); (head, rest.trim())`.
      It returns the first token as the verb (compared case-insensitively by
      callers) and the case-PRESERVED trimmed remainder as the Argument_String
      (B062, D5). In `dispatch_command_string` (dispatch.rs), BEFORE delegating,
      compute `let (_verb, _arg) = crate::shell::helpers::split_verb_arg(raw);`
      (prefix with `_` so it is unused for now) and keep calling
      `self.handle_command(raw);` with the ORIGINAL `raw` -- the ladder's per-arm
      parsing still runs, so behaviour is unchanged. Add a code comment stating
      the split is COMPUTED here and will DRIVE dispatch in Step 2.
      Files: `crates/ff-desktop/src/shell/helpers.rs`,
      `crates/ff-desktop/src/shell/dispatch.rs`
      Verify: `cargo check -p ff-desktop` compiles;
      `cargo test -p ff-desktop -- --test-threads=1` all green (no behaviour
      change -- `raw` is still what `handle_command` receives).

- [ ] 5. STEP 1: add the (verb, arg) -> Function-target helper (NOT wired into the live path).
      In `crates/ff-desktop/src/shell/dispatch.rs`, add a private helper
      `fn function_target_with_arg(command_id: &str, arg: &str) ->
      ff_command::CommandTarget` that builds
      `CommandTarget::Function { command_id: command_id.to_string(), params }`
      where `params` is a `ff_command::TargetParams` (BTreeMap) into which, when
      `!arg.is_empty()`, it inserts `"arg".to_string() ->
      ff_command::TargetValue::String(arg.to_string())` (Req 9.2 `arg` folded
      into params as a string). This helper is NOT called from the live dispatch
      path in Step 1 -- it is exercised ONLY by the unit tests in step 6. Keep it
      `#[cfg_attr(not(test), allow(dead_code))]` or `#[allow(dead_code)]` (with a
      one-line comment: "wired into the live path in Step 2") so clippy is clean.
      Add the needed `use` for `CommandTarget`, `TargetParams`, `TargetValue`
      (all re-exported from `ff_command`). File stays ASCII, <= 400 non-test lines.
      Files: `crates/ff-desktop/src/shell/dispatch.rs`
      Verify: `cargo clippy -p ff-desktop --tests` clean (no dead-code warning);
      `cargo check -p ff-desktop` compiles.

- [ ] 6. STEP 1: add the two split/helper unit tests.
      In `crates/ff-desktop/src/shell/tests_command.rs`, add:
      (a) `single_verb_arg_split_populates_arg_param` -- call
      `split_verb_arg("DOWN 8")`, assert verb token `"DOWN"` matches `"DOWN"`
      case-insensitively and arg `"8"`; then call `function_target_with_arg` with
      the split verb mapped to a representative command_id and arg `"8"`, and
      assert the returned `CommandTarget::Function` carries
      `params["arg"] == TargetValue::String("8".to_string())`.
      (b) `verb_arg_split_preserves_argument_case` -- call
      `split_verb_arg("LOCATE Foo")`, assert verb token matches `"LOCATE"`
      case-insensitively and arg `"Foo"` (case PRESERVED, B062). Both annotate
      `// Validates: command-framework Requirement 9.2, 9.7`. Access the private
      `function_target_with_arg` via `super::dispatch::...` only if visibility
      allows; otherwise place these two tests in a `#[cfg(test)] mod tests` INSIDE
      `dispatch.rs` (sibling to the impl) so they can reach the private helper --
      prefer the in-file test module for the helper test and keep the pure
      `split_verb_arg` assertions in `tests_command.rs`. Keep any in-file test
      module under ~200 lines (it is tiny here).
      Files: `crates/ff-desktop/src/shell/tests_command.rs` and/or the
      `#[cfg(test)]` module in `crates/ff-desktop/src/shell/dispatch.rs`
      Verify: `cargo test -p ff-desktop single_verb_arg_split_populates_arg_param verb_arg_split_preserves_argument_case -- --test-threads=1`
      pass.

- [ ] 7. Full scoped verification and format.
      Run the SCOPED gate in order and read each result before proceeding. NEVER
      `--workspace` and NEVER `verify.ps1` (owner's manual step). Serial test
      threads avoid the known B048 `FFWB_HISTORY_PATH` / `FFWB_USER_CONFIG_PATH`
      env-var race (acceptable, documented noise under multithreaded runs only).
      Files: none (verification only)
      Verify, in this order:
      - `cargo fmt`
      - `cargo check -p ff-desktop` -- compiles clean.
      - `cargo clippy -p ff-desktop --tests` -- no warnings (dead-code allow on
        `function_target_with_arg` is intentional for Step 1).
      - `cargo test -p ff-desktop -- --test-threads=1` -- all 454 shell tests
        plus the 3 new tests pass; zero regressions (Req 8.4 backstop).
      Then HAND OFF: report the exact scoped commands run, state scoped checks are
      clean, and prompt the owner to run the full gate manually
      (`powershell -ExecutionPolicy Bypass -File tools\powershell\verify.ps1`).

---

## Out of scope -- remain TODO (do NOT plan or implement here)

- Step 2: implement `ShellTargetResolver::builtin_workspace_target`
  (`command_config/mod.rs:167`) classification for the CustomWorkspace/navigation
  family and WIRE `dispatch_command_string` to call `resolve_target` + dispatch
  the split-derived target; remove the covered ladder arms. This is where
  `function_target_with_arg` and the computed `(verb, arg)` split become live.
- Steps 3-7: Function family; manager families (nav, exclude/show, find,
  profile, scroll); split/detach/swap + workspace; standalone + scrm; retire the
  empty `try_commands_a/b1/b2/c` segments.

Each later step keeps BOTH the new table entry and the ladder arm reachable until
the entry is test-proven, then deletes the arm -- per the migration section of
`design-delta.md`.
