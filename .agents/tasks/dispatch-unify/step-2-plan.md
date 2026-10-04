# Step 2 Implementation Plan -- builtin_workspace_target + typed path through resolve_target (B080)

CONFORMANCE fix against EXISTING command-framework criteria (Req 2.1 one front
door, 2.7 no UI mutates state outside the framework, 8.3 ordered chain, 8.4 same
observable result, 9.2/9.7 one verb/arg split). The requirements gate is CLOSED:
no new requirements, no gate. Plain ASCII only.

Work happens DIRECTLY in the main workspace at
`c:\workspace\VSC\FileForgeWorkbench`. No worktree, branch, or commit. Edit in
place; leave uncommitted.

## Scope statement (READ FIRST)

STEP 2 is the ONLY scope. It does TWO things, for ONE verb family only:

1. Make `dispatch_command_string` the LIVE ordered front door: run stage 1
   (current-menu Option_Key) + the POM/chained fastpaths FIRST, THEN
   `resolve_target`, THEN fall through to the `handle_command` ladder. To avoid
   the prelude double-running, the stage-1/fastpath/history prelude is MOVED out
   of `handle_command` into the front door (see Item 1 -- this is the one
   non-trivial refactor and is behaviour-preserving).
2. Make `ShellTargetResolver::builtin_workspace_target` classify ONLY the
   CustomWorkspace / navigation family to `CommandTarget::CustomWorkspace`, and
   complete the `dispatch_command_target` CustomWorkspace arm so each kind calls
   the SAME shell method the ladder arm calls today (identical observable result,
   Req 8.4). Return `None` for every other verb (no over-claim).

VERBS IN SCOPE (design-delta section 2.3) -- and NOTHING else:
`FILES` / `=FILES`, `FILE CATALOGS` / `CATALOGS`, `CONFIG [<ns>]`, `COMMANDS`,
`LOG`, `PLUGINS`, `MACROS`, `GSEARCH` / `SEARCH`, `KEYS [<kind>]`, `KINDS`,
`MENUS`, bare `THEME` (Theme editor only).

OUT OF SCOPE for Step 2 (steps 3-7 remain TODO, DO NOT start them):
- Step 3: Function family (EXIT/QUIT/=X/X/LOGOFF, EDIT, BROWSE/VIEW, CLOSE).
- Step 4: manager families (nav, exclude/show, find, profile, scroll).
- Step 5: split/detach/swap and workspace families.
- Step 6: standalone verbs and scrm.
- Step 7: retire the empty ladder segments; fold `handle_command` into the
  terminal engine caller.
- DELETING the migrated verbs' ladder arms (Item 4 leaves them in place as an
  unreachable fallback for THIS step; deletion is a follow-up once green).
- POM and SETTINGS: already stage-3 `menu_name_target` -- DO NOT touch them.

## Confirmed current state (verified by reading the tree this pass)

- `shell/dispatch.rs` (67 non-test lines): `dispatch_command_string(&mut self,
  raw: &str)` computes `let (_verb,_arg) = split_verb_arg(raw);` then calls
  `self.handle_command(raw)` (pure indirection). It also has a dead-code-allowed
  `function_target_with_arg(command_id, arg) -> CommandTarget` helper (NOT used
  by Step 2 -- that is Step 3; leave it as-is, still `#[allow(dead_code)]`).
- `shell/commands.rs`: `run_command_line` (line ~30) wraps
  `begin_command_line()` / `dispatch_command_string(&original)` /
  `finish_command_line(&original)`. `handle_command` (line ~69) runs, IN ORDER:
  `command_line_history.record(cmd)` -> `try_current_menu_option(cmd)` ->
  `try_commands_a` -> `resolve_pom_option_key(&upper)` (recurses into
  `handle_command`) -> `try_commands_b1` -> `try_commands_b2` ->
  `try_chained_fastpath(&upper)` -> `NAME` arm -> `try_commands_c` ->
  `try_menu_name_dispatch(cmd.trim())` -> `cmd_engine.execute_command_line`.
- `shell/target_dispatch.rs`: `dispatch_bound_command` (line ~92) does
  `resolve_and_dispatch_command` and on `FallThrough` calls
  `dispatch_command_string(command)`. `dispatch_command_target` CustomWorkspace
  arm (line ~175) currently sets `open_error = "... not yet runnable ..."` -- a
  STUB that Step 2 completes.
- `command_config/mod.rs` (202 non-test lines): `ShellTargetResolver`.
  `builtin_workspace_target` returns `None` (the stub to replace, ~line 167).
- Helpers (`shell/helpers.rs`): `verb_arg(cmd, verb) -> Option<&str>`
  (case-insensitive verb, trimmed case-preserved arg) and
  `split_verb_arg(cmd) -> (&str, &str)` (same split rule, extracts the verb
  token). Use these -- do NOT hand-count byte offsets.
- Stage-1/fastpath signatures: `try_current_menu_option(&mut self, cmd: &str)
  -> bool`; `resolve_pom_option_key(&mut self, upper: &str) -> Option<String>`;
  `try_chained_fastpath(&mut self, upper: &str) -> Option<bool>`.
- `nav_to_kind(&mut self, kind: ff_session::session_state::WorkspaceKind)` is the
  parameterless in-place navigator used by FILES/CATALOGS/COMMANDS/LOG/PLUGINS/
  MACROS. `WorkspaceKind` variants present: `FileExplorer`, `Files`, `Config`,
  `Search`, `PluginManager`, `EventLog`, `MacroLibrary`, `CommandConfigurator`,
  `Editor`, `ScrmViewer`.
- 936 tests pass (454 shell tests). Build clean.

CRITICAL per-verb behaviour differences confirmed by reading the arms (these
drive the plan):
- `CONFIG` (ladder_b1): `verb_arg(cmd,"CONFIG")`; empty -> `open_config_view(None)`;
  else `open_config_view(Some(arg.to_lowercase()))`. NOTE the arg is LOWERCASED.
- `GSEARCH`/`SEARCH` (ladder_b1): `open_or_focus_search_panel()` (NOT nav_to_kind).
- `LOG` (ladder_b1): `nav_to_kind(EventLog)` THEN
  `notification_queue.lock().mark_all_read()`. The mark-all-read is part of the
  observable effect and MUST be preserved.
- `KEYS` (ladder_a): `verb_arg(cmd,"KEYS")`; `open_keys_editor(kind)` where
  `kind = if arg.is_empty() { None } else { Some(arg) }` (arg case PRESERVED).
- `KINDS` (ladder_a): exact `upper == "KINDS"` -> `open_kinds_editor()`.
- `MENUS` (ladder_b1): exact `upper == "MENUS"` -> `open_menus_editor()`.
- bare `THEME` ONLY (ladder_b2): `verb_arg(cmd,"THEME")` with `arg.is_empty()`
  -> `open_theme_editor()`. `THEME <name>` is a DIFFERENT action (applies a
  theme via `set_active_theme`) and is OUT OF SCOPE -- `builtin_workspace_target`
  MUST return `None` when THEME has a non-empty arg so it falls through to the
  ladder's theme-apply arm.
- `FILES`/`=FILES` (ladder_b1): `nav_to_kind(FileExplorer)`.
- `FILE CATALOGS`/`CATALOGS` (ladder_b1): `nav_to_kind(Files)` -- note the
  two-word verb `FILE CATALOGS`.
- `COMMANDS` (ladder_b1): `nav_to_kind(CommandConfigurator)`. Disjoint from
  `COMMAND`/`COMMAND <pos>` (ladder_b1) which is OUT OF SCOPE and must stay on
  the ladder.
- `PLUGINS` (ladder_b1): `nav_to_kind(PluginManager)`.
- `MACROS` (ladder_b1): `nav_to_kind(MacroLibrary)`.

## Design decisions (recorded, with rationale)

- D-A. Disentangle approach: MOVE the prelude (history record + stage 1 + POM
  fastpath + chained fastpath) OUT of `handle_command` INTO
  `dispatch_command_string`; `handle_command` becomes the ladder-only terminal.
  Rationale: the step prompt (A1, owner-approved stage-1-first for all seams)
  requires the front door run stage 1 + fastpaths before `resolve_target`, and
  the fallthrough `handle_command` must NOT re-run them (else they double-run).
  Moving the prelude up (rather than guarding it with a flag) is the cleanest
  behaviour-preserving option: today the prelude ALREADY runs exactly once per
  outermost submit, and inner recursions (`resolve_pom_option_key` ->
  `handle_command`, chained segments) MUST continue to re-enter the FRONT DOOR so
  each segment re-resolves consistently (D8). This is scoped and the 454 shell
  tests are the backstop. See Item 1 for the exact mechanics and the
  double-run/recursion analysis. If the implementer finds the recursion rewiring
  larger than this plan anticipates, STOP and report (do not guess) -- see the
  Item 1 "STOP condition".
- D-B. `workspace_kind` string vocabulary: `CommandTarget::CustomWorkspace`
  carries a free-form `workspace_kind: String`. Step 2 defines an EXPLICIT,
  stable string per in-scope verb (table in Item 2). The strings are chosen to
  match `ff_session::WorkspaceKind` serde snake_case where a 1:1 enum variant
  exists (`file_explorer`, `files`, `config`, `search`, `plugin_manager`,
  `event_log`, `macro_library`, `command_configurator`), and use distinct
  editor tokens where there is NO `WorkspaceKind` variant (`keys`, `kinds`,
  `menus`, `theme_editor`). The `dispatch_command_target` CustomWorkspace arm
  (Item 3) matches on these strings and calls the exact ladder method, so the
  observable result is identical regardless of the string spelling. Rationale:
  the transient editors (KEYS/KINDS/MENUS/THEME) are not `WorkspaceKind`
  variants, so a single enum cannot express the family; an explicit documented
  string table is the lowest-risk mapping and keeps the classifier pure.
- D-C. Leave migrated ladder arms IN PLACE (Item 4) as an unreachable fallback
  for Step 2; a follow-up deletes them once tests are green. Rationale:
  per-step rollback safety (design-delta section 4).

---

# Implementation items (ordered by dependency)

- [ ] 1. Move the dispatch prelude from `handle_command` into
      `dispatch_command_string`, keeping `handle_command` as the ladder-only
      terminal. This makes the front door the single ordered chain and prevents
      the prelude double-running.

      Mechanics (behaviour-preserving):
      - In `shell/dispatch.rs`, rewrite `dispatch_command_string(&mut self, raw:
        &str)` to run, IN THIS ORDER:
        1. `self.command_line_history.record(raw);` (moved verbatim from the top
           of `handle_command`; `record` already excludes RETRIEVE and resets the
           retrieve pointer -- unchanged).
        2. `if self.try_current_menu_option(raw) { return; }` (stage 1, Req 8.3
           stage 1 / D6).
        3. Compute `let upper = raw.trim().to_uppercase();` and
           `if let Some(pom_command) = self.resolve_pom_option_key(&upper) { if
           pom_command.to_uppercase() != upper { self.dispatch_command_string(
           &pom_command); return; } }` -- NOTE: the recursion now re-enters
           `dispatch_command_string` (the front door), NOT `handle_command`, so a
           POM option command that is itself an in-scope verb resolves through
           `resolve_target` too (D7). The self-reference guard is preserved
           verbatim.
        4. `if let Some(handled) = self.try_chained_fastpath(&upper) { if handled
           { return; } }` -- chained segments: the chained fastpath internally
           re-dispatches each segment; see the STOP condition below for the
           re-dispatch target.
        5. The single verb/arg split: `let (verb, arg) = split_verb_arg(raw);`
           (keep the existing call; `verb`/`arg` are now USED, so drop the `_`
           prefixes).
        6. `resolve_target`: build the resolver exactly as
           `resolve_and_dispatch_command` does
           (`ShellTargetResolver::new(&self.command_store.definitions,
           &self.cmd_registry, self.menus_dir())`), then
           `match ff_command::resolve_target(raw, &resolver) { Ok(target) =>
           { self.dispatch_command_target(&target); return; } Err(_) => {} }`.
           Pass `raw` (NOT the pre-split verb) so the existing `menu_name_target`
           first-token logic and user-definition exact-id match are unchanged.
        7. Fall through: `self.handle_command(raw);` (the ladder terminal).
      - In `shell/commands.rs`, DELETE from `handle_command` the moved prelude:
        the `command_line_history.record(cmd)` line, the `try_current_menu_option`
        block, the `resolve_pom_option_key` block, and the `try_chained_fastpath`
        block. `handle_command` now starts at `let upper = cmd.trim().
        to_uppercase();` then `try_commands_a` ... through the engine terminal,
        unchanged. Keep the explanatory comments trimmed to match.

      RECURSION / DOUBLE-RUN analysis (why this is behaviour-preserving):
      - Today the prelude runs once at the top of `handle_command`, and the ONLY
        callers that reach the prelude are: (a) `run_command_line` (via
        `dispatch_command_string` -> `handle_command`), (b) the FallThrough from
        `dispatch_bound_command`, (c) `resolve_pom_option_key`'s recursion
        `self.handle_command(&pom_command)`, (d) chained-fastpath per-segment
        re-dispatch, and (e) any ladder arm that calls `self.handle_command(...)`
        internally (e.g. the AUTONUM->NUMBER redirect in ladder_c).
      - After the move, (a) and (b) call `dispatch_command_string`, which runs the
        prelude ONCE then falls through to `handle_command` (now prelude-free) --
        same single execution.
      - (c) is rewired to `dispatch_command_string(&pom_command)` so the resolved
        POM command gets the full chain (prelude + resolve_target) -- a superset
        that is behaviour-identical for the common case and is the intended D7
        convergence.
      - (d) chained segments: inspect `try_chained_fastpath` /
        `commands_fastpath.rs` -- each segment is currently re-dispatched through
        `handle_command`. CHANGE those per-segment re-dispatch calls to
        `dispatch_command_string` so every segment re-enters the one front door
        (D8 "route segments through one door"). This is REQUIRED for correctness
        of the move: a segment that is an in-scope verb must resolve via
        `resolve_target`.
      - (e) ladder-internal `self.handle_command(...)` redirects (e.g.
        AUTONUM->NUMBER) must remain `handle_command` calls (NOT front-door) so
        they do NOT re-run stage 1 / fastpaths / history for an internal redirect
        -- today they already bypass a second record because record de-dups, but
        after the move `handle_command` no longer records at all, so an internal
        redirect simply runs the ladder. Confirm by grep that any such internal
        redirect is a genuine alias redirect (not a user resubmission) and leave
        it calling `handle_command`.

      STOP condition (report, do not guess): if, while rewiring the chained
      fastpath (d) or auditing ladder-internal `handle_command` calls (e), the
      implementer finds a re-dispatch whose correct target (front door vs ladder)
      is ambiguous OR that would change history-recording or outcome-wrap
      behaviour, STOP and report the specific call site rather than guessing.
      This is the one place the prompt flagged as potentially too large for one
      step.

      Files: `crates/ff-desktop/src/shell/dispatch.rs`,
      `crates/ff-desktop/src/shell/commands.rs`,
      `crates/ff-desktop/src/shell/commands_fastpath.rs` (chained per-segment
      re-dispatch target only).
      Verify: `cargo test -p ff-desktop` -- all 454 shell tests still pass
      (especially `tests_nav.rs` POM option / `=0.K` chained,
      `tests_split_detach.rs` `=0.M` detached, `tests_session.rs`
      restore-on-error, `tests_command.rs` SWAP merge + verb_arg case). This item
      alone must leave the build green with NO behaviour change (resolve_target
      currently classifies nothing new because `builtin_workspace_target` is still
      the `None` stub until Item 2).

- [ ] 2. Replace the `builtin_workspace_target` `None` stub in
      `command_config/mod.rs` with a classifier for the in-scope family ONLY.

      Implement on `impl TargetResolver for ShellTargetResolver<'_>`:
      ```
      fn builtin_workspace_target(&self, input: &str) -> Option<CommandTarget> {
          builtin_workspace_target_for(input)
      }
      ```
      and add a free function `fn builtin_workspace_target_for(input: &str) ->
      Option<CommandTarget>` (free fn so it is unit-testable without a registry
      and keeps the impl thin). It:
      - trims `input`, then splits once on whitespace into `(verb, arg)` using the
        SAME rule as `split_verb_arg` (first token, trimmed case-preserved
        remainder). Match the verb case-insensitively; preserve `arg` case.
      - returns `Some(CommandTarget::CustomWorkspace { workspace_kind, params })`
        for the in-scope verbs per the table below, else `None`.
      - CustomWorkspace carries NO behaviour itself; the arg-folding is ONLY for
        verbs whose ladder method reads an arg (CONFIG, KEYS). All others ignore
        any trailing arg EXCEPT where the ladder matched EXACTLY (KINDS, MENUS,
        COMMANDS, PLUGINS, LOG, MACROS, FILES, SEARCH) -- for those, require the
        arg to be EMPTY to classify (so `COMMANDS foo` is NOT claimed here and
        falls through, matching the ladder's exact `upper == "COMMANDS"`), with
        the exceptions noted in the table.

      verb -> workspace_kind -> params table (EXACT):

      | Input verb (case-insens.) | Claim when | workspace_kind | params |
      |---------------------------|-----------|----------------|--------|
      | FILES                     | arg empty | `file_explorer` | none |
      | =FILES                    | always (single token) | `file_explorer` | none |
      | FILE CATALOGS             | two-token exact (verb==`FILE`, arg==`CATALOGS` case-insens.) | `files` | none |
      | CATALOGS                  | arg empty | `files` | none |
      | CONFIG                    | always (arg optional) | `config` | if arg non-empty: `namespace` = `arg.to_lowercase()` (String); else none |
      | COMMANDS                  | arg empty | `command_configurator` | none |
      | LOG                       | arg empty | `event_log` | none |
      | PLUGINS                   | arg empty | `plugin_manager` | none |
      | MACROS                    | arg empty | `macro_library` | none |
      | GSEARCH                   | arg empty | `search` | none |
      | SEARCH                    | arg empty | `search` | none |
      | KEYS                      | always (arg optional) | `keys` | if arg non-empty: `kind` = `arg` (case-preserved String); else none |
      | KINDS                     | arg empty | `kinds` | none |
      | MENUS                     | arg empty | `menus` | none |
      | THEME                     | arg EMPTY ONLY | `theme_editor` | none |

      Notes:
      - `=FILES` is a single token (no whitespace), so `verb == "=FILES"`;
        classify it directly.
      - `FILE CATALOGS` is the ONLY two-word in-scope verb: when `verb` matches
        `FILE` case-insensitively AND `arg` equals `CATALOGS` case-insensitively,
        classify to `files`; `CATALOGS` as a lone verb also classifies to `files`.
      - THEME: return `None` for any non-empty arg (so `THEME legacy` falls
        through to the ladder theme-apply arm -- OUT OF SCOPE). This is the one
        verb where a non-empty arg means DO NOT claim.
      - Everything NOT in the table returns `None` (no over-claim) so user menus /
        macros / registered ids / editor verbs still resolve at the correct
        stage.
      - Update the `ShellTargetResolver` doc comment: delete the now-false
        sentence "`builtin_workspace_target` deliberately returns `None`" and
        replace with a note that it classifies the CustomWorkspace / navigation
        family (Step 2, B080) and returns `None` for all other verbs.
      - `UserCommandStore::builtin_workspace_target` stays `None` (it is only the
        user-definition view; do NOT add built-ins there).

      Files: `crates/ff-desktop/src/command_config/mod.rs`.
      Verify: `cargo test -p ff-desktop command_config` -- the new unit test
      `builtin_workspace_target_classifies_nav_verb` (Item 5) passes; existing
      `command_config` tests (menu-name, user-command) still pass.

- [ ] 3. Complete the `dispatch_command_target` CustomWorkspace arm in
      `target_dispatch.rs` so each in-scope kind calls the EXACT shell method the
      ladder arm calls (identical observable result, Req 8.4). Do NOT reimplement
      navigation -- reuse the existing methods.

      Replace the current stub arm:
      ```
      CommandTarget::CustomWorkspace { workspace_kind, params } => {
          self.open_error = Some(format!("... not yet runnable ..."));
      }
      ```
      with a match on `workspace_kind.as_str()` dispatching to the ladder method,
      reading `params` where the ladder reads an arg:

      | workspace_kind | shell method called (EXACT) | arg source |
      |----------------|-----------------------------|-----------|
      | `file_explorer` | `self.nav_to_kind(WorkspaceKind::FileExplorer); self.open_error = None;` | none |
      | `files` | `self.nav_to_kind(WorkspaceKind::Files); self.open_error = None;` | none |
      | `command_configurator` | `self.nav_to_kind(WorkspaceKind::CommandConfigurator); self.open_error = None;` | none |
      | `plugin_manager` | `self.nav_to_kind(WorkspaceKind::PluginManager); self.open_error = None;` | none |
      | `macro_library` | `self.nav_to_kind(WorkspaceKind::MacroLibrary); self.open_error = None;` | none |
      | `event_log` | `self.nav_to_kind(WorkspaceKind::EventLog); self.notification_queue.lock().expect("queue").mark_all_read(); self.open_error = None;` | none |
      | `search` | `self.open_or_focus_search_panel(); self.open_error = None;` | none |
      | `config` | `let ns = params.get("namespace")...String; if ns empty/absent { self.open_config_view(None) } else { self.open_config_view(Some(ns)) } self.open_error = None;` | `params["namespace"]` (already lowercased in Item 2) |
      | `keys` | `let kind = params.get("kind")...String (Option); self.open_keys_editor(kind.as_deref()); self.open_error = None;` | `params["kind"]` (case-preserved) |
      | `kinds` | `self.open_kinds_editor(); self.open_error = None;` | none |
      | `menus` | `self.open_menus_editor(); self.open_error = None;` | none |
      | `theme_editor` | `self.open_theme_editor(); self.open_error = None;` | none |
      | any other string | KEEP the existing deferred-error message (preserves behaviour for CustomWorkspace strings Step 2 does not own) |

      Implementation notes:
      - `use ff_session::session_state::WorkspaceKind;` at the arm (or fully
        qualify) -- `nav_to_kind` takes the enum, not the string.
      - To read a `TargetValue::String` from `params`: match
        `params.get("namespace")` / `params.get("kind")` against
        `Some(ff_command::TargetValue::String(s))`.
      - The LOG `mark_all_read` ordering (nav THEN mark) MUST match the ladder.
      - `open_config_view` expects the namespace already lowercased; Item 2
        lowercases it when folding into params, so the arm passes it through
        unchanged (do NOT lowercase twice -- idempotent, but keep it in Item 2 to
        mirror the ladder exactly).
      - Keep the `_ =>` fallback arm with the existing not-yet-runnable error so
        an out-of-scope CustomWorkspace string (none exist today via this path,
        but future ones) is not silently swallowed.

      Files: `crates/ff-desktop/src/shell/target_dispatch.rs`.
      Verify: `cargo test -p ff-desktop` -- all shell tests pass; the new
      `typed_and_key_paths_reach_same_handler_for_builtin_verb` (Item 5) passes.

- [ ] 4. Leave the migrated verbs' ladder arms IN PLACE as a superseded,
      now-unreachable fallback. Add a one-line comment above each in-scope arm
      noting it is superseded by `builtin_workspace_target` (Step 2, B080) and
      kept as a fallback pending deletion in a Step 2 follow-up.

      Arms to annotate (do NOT delete, do NOT change their bodies):
      - `commands_ladder_a.rs`: `KEYS` arm, `KINDS` arm.
      - `commands_ladder_b.rs` (`try_commands_b1`): `CONFIG`, `FILES`/`=FILES`,
        `GSEARCH`/`SEARCH`, `COMMANDS`, `MENUS`, `LOG`, `FILE CATALOGS`/`CATALOGS`,
        `PLUGINS`, `MACROS` arms. Do NOT touch the `COMMAND` / `COMMAND <pos>`
        arms (out of scope, must stay reachable).
      - `commands_ladder_b2.rs`: the BARE-`THEME` branch inside the `verb_arg(cmd,
        "THEME")` arm. The `THEME <name>` apply branch MUST stay reachable (it is
        the fall-through target when `builtin_workspace_target` returns `None` for
        a non-empty THEME arg).

      Rationale: these arms are now unreachable because the front door resolves
      the in-scope verbs via `resolve_target` BEFORE falling through to
      `handle_command`; keeping them is per-step rollback safety (design-delta
      section 4). They are proven unreachable by the Item 5 equivalence test.

      Files: `crates/ff-desktop/src/shell/commands_ladder_a.rs`,
      `commands_ladder_b.rs`, `commands_ladder_b2.rs` (comments only).
      Verify: `cargo clippy -p ff-desktop` -- no dead-code warnings (the arms are
      still compiled; they are logically, not statically, unreachable).

- [ ] 5. Add the two Step-2 tests (design-delta section 5.2). Write them FIRST
      (red) per TDD, then confirm green after Items 1-3.

      Test A -- unit, in `command_config/mod.rs` `#[cfg(test)] mod tests`:
      `builtin_workspace_target_classifies_nav_verb`.
      ```
      // Validates: command-framework Requirement 8.3 (stage 2) -- built-in
      // workspace verbs are classified by resolve_target (stub removed).
      ```
      Assert, using the free fn `builtin_workspace_target_for` directly (no
      registry needed):
      - `builtin_workspace_target_for("FILES")` ==
        `Some(CommandTarget::CustomWorkspace { workspace_kind: "file_explorer",
        params: empty })`.
      - case-insensitive: `builtin_workspace_target_for("files")` same.
      - `builtin_workspace_target_for("CONFIG core")` ==
        CustomWorkspace `config` with `params["namespace"] ==
        TargetValue::String("core")` (lowercased).
      - `builtin_workspace_target_for("THEME")` == CustomWorkspace
        `theme_editor`, but `builtin_workspace_target_for("THEME legacy")` ==
        `None` (non-empty THEME arg not claimed).
      - `builtin_workspace_target_for("ZXQWV")` == `None` (no over-claim).
      Also add one `resolve_target`-level assertion through a full
      `ShellTargetResolver` (empty defs, empty registry, temp menus dir) that
      `resolve_target("FILES", &resolver)` is `Ok(CustomWorkspace{file_explorer})`
      -- proving the stub no longer wins.

      Test B -- full-shell equivalence, in the shell test module (add to
      `crates/ff-desktop/src/shell/tests_command.rs`, the home for command-path
      tests): `typed_and_key_paths_reach_same_handler_for_builtin_verb`.
      ```
      // Validates: command-framework Requirement 2.1 -- the typed and key seams
      // reach ONE handler for a built-in verb (same observable result).
      ```
      Use the existing shell test harness/builder in `tests_command.rs` (follow
      the SWAP-key tests around lines 628-694 for the construction pattern):
      - Build two fresh shells in the SAME starting state (NOT a MenuWorkspace, so
        stage 1 does not intercept -- e.g. the default/editor home used by the
        existing command tests).
      - Shell 1: `shell.run_command_line("CONFIG core")`.
      - Shell 2: set `shell2.command_text = "core"`, then
        `shell2.dispatch_key_command("CONFIG")` (empty-field vs field-merge:
        prefer the empty-field form -- set `command_text` empty and
        `dispatch_key_command("CONFIG core")` -- to avoid depending on the merge;
        pick whichever the harness supports and assert the SAME resulting state).
      - Assert identical resulting shell state: the active tab is now the Config
        Context (`TabKind::ConfigPanel`) AND the namespace filter equals `core`
        (`config_panel.namespace_filter == Some("core")`), and `open_error` is
        `None` for both. Compare the two shells' observable fields for equality.
      - Keep the Tab-walk/`run()` budget bounded; assert state, not frame counts.

      If the exact harness constructor differs from the SWAP tests, mirror
      whatever those tests use -- do not invent a new builder.

      Files: `crates/ff-desktop/src/command_config/mod.rs` (Test A),
      `crates/ff-desktop/src/shell/tests_command.rs` (Test B).
      Verify: `cargo test -p ff-desktop builtin_workspace_target_classifies_nav_verb`
      and `cargo test -p ff-desktop typed_and_key_paths_reach_same_handler_for_builtin_verb`
      both pass; full `cargo test -p ff-desktop` stays green (937+ tests).

- [ ] 6. Scoped verification + hand-off. Run ONLY the scoped checks (never
      `--workspace`, never `ffwb-gate.ps1` -- those are the owner's manual step):
      - `cargo fmt`
      - `cargo check -p ff-desktop`
      - `cargo clippy -p ff-desktop` (clean, no `-D warnings` regressions)
      - `cargo test -p ff-desktop`
      Fix any failure attributable to this change and rerun the scoped checks
      until clean. Then STOP and hand off: state exactly which scoped commands
      ran and prompt the owner to run the full gate
      (`pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1`) outside Kiro.
      Do NOT self-certify the full gate.

      Files: none (verification only).
      Verify: scoped checks clean; hand-off message printed.

---

## PRESERVE list (Item 6 must confirm none of these changed)

- Key-path field-merge (Req 9.8): `dispatch_key_command` still merges
  `<command> <field>` BEFORE calling `dispatch_bound_command` -> front door. The
  merge remains the ONLY string synthesis. (D2)
- Single `begin_command_line` / `finish_command_line` wrap at the OUTERMOST
  submit only (`run_command_line`, `dispatch_key_command`); inner re-dispatches
  (POM recursion, chained segments) do NOT re-wrap. (D4)
- B062 case rule: verb case-insensitive, Argument_String case-preserved
  (CONFIG lowercases its own namespace by its own rule; KEYS kind stays
  case-preserved). (D5)
- Inline menu `[options.target]` direct dispatch via `dispatch_command_target`
  (NOT re-resolved) -- `activate_menu_option` is untouched. (D10)
- POM / SETTINGS stay stage-3 `menu_name_target` (Menu targets); not added to
  `builtin_workspace_target`.
- Notification channel + `ShellServices` surfaces unchanged.
- History recorded ONCE at the boundary (now in the front door, moved verbatim
  from the top of `handle_command`). (D11)
- `COMMAND` / `COMMAND <pos>` arms (out of scope) stay reachable on the ladder;
  only `COMMANDS` is migrated.
- `THEME <name>` theme-apply arm stays reachable (fall-through when
  `builtin_workspace_target` returns `None` for a non-empty THEME arg).

## File-size note (rust-standards 400 non-test lines)

- `shell/dispatch.rs`: 67 non-test lines today. Item 1 adds the ordered chain
  (~25-35 lines). Well under 400; no split needed.
- `command_config/mod.rs`: 202 non-test lines today. Item 2 adds the classifier
  free fn + table (~45-60 lines). Approaches but stays under 400; if it crosses
  400 after Item 2, split the classifier into
  `command_config/builtin_target.rs` (a `pub(crate) fn
  builtin_workspace_target_for`) and call it from the impl -- a pure code-move
  refactor, no behaviour change. Flag at implementation time; prefer the split
  only if the measured non-test count exceeds 400.
- `target_dispatch.rs`: the CustomWorkspace arm grows by ~20 lines; the file is
  well under 400 (verify after Item 3).

## Steps 3-7 remain TODO

This plan implements STEP 2 ONLY. Steps 3 (Function family), 4 (manager
families), 5 (split/detach/workspace), 6 (standalone + scrm), and 7 (retire the
ladder segments, fold `handle_command`) are OUT OF SCOPE and not started here.
Deleting the Item-4 superseded arms is a Step-2 follow-up once these tests are
green.
