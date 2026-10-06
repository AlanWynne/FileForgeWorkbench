# CR-CH-052 Requirements-Gate DRAFT (read-only investigation)

STATUS: gate DRAFT for owner review ONLY. No source changed, no spec files
written. ASCII only. All line anchors below were re-grepped against the live
tree in `crates/ff-desktop/src/shell/`.

---

## 1. Summary answer first

CR-CH-052 asks for ONE uniform, Context-agnostic navigation/exit model for every
Workspace, with NO POM-specific exit behaviour, plus the deferred deletion half
of B080 Step 7. The investigation confirms:

- The `=` prefix is NOT a single uniform rule today. It is handled in three
  different, non-uniform places: (a) the EXIT-family literal `"=X"` match
  (`commands_ladder_a.rs` `try_exit_family`), (b) the chained-fastpath origin pop
  to POM (`commands_fastpath.rs` `try_chained_fastpath`), and (c) the
  `resolve_pom_option_key` strip-`=` which reads the POM menu but does NOT pop the
  ladder. There is NO single front-door `=` ladder-drop step. The ladder is NEVER
  actually "dropped to empty" for a bare `=<key>`; only a chained `=...` path with
  a separator pops to POM, and even then by inserting/returning to a POM tab, not
  by clearing `nav_stack`.

- `X` is NOT uniform across Workspaces today. On a MenuWorkspace (incl. the POM),
  bare `X` is intercepted FIRST by `try_current_menu_option` (POM option key `X`
  -> command `RETURN` -> `nav_return`), so POM `X` returns-to-top / closes. On a
  NON-menu Context, `X` is NOT a menu option, so it falls through to
  `try_exit_family` and runs `file.exit` (APPLICATION exit). This is exactly the
  POM-vs-everything-else split CR-CH-052 removes.

- `=X` today = APPLICATION exit unconditionally (literal match in
  `try_exit_family`, governed by startup-and-session Req 12/40). CR-CH-052 revises
  this to close-the-Workspace (app-exit only when last).

- END is already uniform and already matches the CR's description
  (`nav_stack.rs` `nav_end`: pop one, close-at-empty, exit-when-last). It needs no
  behaviour change; only its spec wording stays as-is.

- The three nav callers that bypass the front door (`resolve_pom_option_key`
  recursion in `commands.rs`; chained segments in `commands_fastpath.rs`; START
  reconstruction via `apply_start_command` -> `handle_command` in `nav_stack.rs`)
  all call `handle_command` directly, which does NOT run `resolve_target`. That is
  why the 11 "superseded" ladder arms are still LIVE and not deletable. Rerouting
  them through `dispatch_command_string` is a behaviour change (it introduces
  `resolve_target`, hence user-command shadowing and the new `=` handling), which
  is why it correctly belongs to CR-CH-052.

This is a BEHAVIOUR CHANGE, not a neutral refactor. It builds ON the existing
single front door (framework mechanism 1) and the per-tab Navigation_Stack
(mechanism 3). No new dispatch path or second nav stack is required.

---

## 2. Current-behaviour map (file:line anchors, re-grepped)

### 2.1 Dispatch skeleton

- `run_command_line` (`commands.rs` ~L28) -> `dispatch_command_string`.
- Front door `dispatch_command_string` (`dispatch.rs` ~L46): runs
  `run_command_prelude`, then `ff_command::resolve_target`, then
  `run_command_ladder`.
- `handle_command` (`commands.rs` ~L83): runs `run_command_prelude` then
  `run_command_ladder` -- NO `resolve_target`. This is the Function-execution
  terminal and the direct-caller entry.
- `run_command_prelude` (`commands.rs` ~L107) order:
  1. `command_line_history.record`
  2. `try_current_menu_option` (`commands_menu.rs` ~L109) -- current-menu
     Option_Key, MenuWorkspace only.
  3. EXIT family gate: `if (!editor_env_active || is_equals_prefixed) &&
     self.try_exit_family(upper)` (`commands.rs` ~L142).
  4. `resolve_pom_option_key` recursion (`commands.rs` ~L163-168): if a POM option
     key resolves, `self.handle_command(&pom_command)` (DIRECT re-entry).
  5. `try_chained_fastpath` (`commands.rs` ~L173).

### 2.2 `=` handling TODAY (three non-uniform sites)

| Site | File:anchor | What it does with a leading `=` |
|------|-------------|---------------------------------|
| EXIT family | `commands_ladder_a.rs` `try_exit_family` ~L29-31 | Literal `upper == "=X"` -> `file.exit`. No ladder drop; `=X` is just a second spelling of app-exit. |
| Chained fastpath | `commands_fastpath.rs` `try_chained_fastpath` ~L78-130 | Only triggers when the string has a `.`/`;` separator. `is_origin = upper.starts_with('=')`; if origin AND not home, `insert_pom_tab` + `ensure_pom_menu_loaded`, then `strip_prefix('=')` and dispatch each segment via `handle_command`. So `=0.K` pops to POM; bare `=1` (no separator) does NOT come here. |
| POM option-key | `commands_fastpath.rs` `resolve_pom_option_key` ~L150-156 | `key = upper.strip_prefix('=')`; reads the POM menu (even from another workspace) and returns the option's command. Does NOT pop/clear the ladder -- it just resolves the command, which is then re-dispatched via `handle_command`. |
| Front-door prelude | `commands.rs` ~L135, `dispatch.rs` ~L46 | `is_equals_prefixed = cmd.trim_start().starts_with('=')` only GATES the editor-env skip (CR-CH-053 E8 narrow `=` rule). There is NO uniform ladder-drop step. |
| Ladder active-env gate | `commands.rs` `run_command_ladder` ~L210 | `if !cmd.trim_start().starts_with('=')` skips the FFEDIT claim for `=`-prefixed input. Again only an escape-hatch gate, not a ladder drop. |

GOVERNING SPEC: command-framework Req 10.2 ("WHEN a navigation chain begins with
`=`, THE Navigation_Origin SHALL be the POM"); menu-workspace Req 5.7 / 14.7
(leading `=` resets to POM with empty stack before applying segments); menu-
workspace Req 8-9 (`=0.E` STOP / `=0;E` PUSH).

KEY GAP vs the CR: today `=` means "origin is the POM (Home)". The CR generalises
it to "origin is the TOP of THIS tab's ladder" (drop `nav_stack` to empty, run
from the clean base of the current tab). For the POM tab these coincide; for a
NON-POM tab they differ (today `=` jumps to POM; the CR says `=` drops THIS tab's
ladder and runs from the top of THIS tab).

### 2.3 `X` handling TODAY (POM vs non-POM -- the split to remove)

- POM / any MenuWorkspace with an `X` option: `try_current_menu_option`
  (`commands_menu.rs` ~L109) runs FIRST in the prelude. The Recovery_Baseline POM
  (menu-workspace Req 12.2) binds `X` -> command `RETURN`. So POM `X` ->
  `activate_menu_option` -> `RETURN` -> `nav_return` (`nav_stack.rs` ~L196):
  non-home navigates to POM clearing stack; home closes the workspace / exits
  when last.
- Non-menu Context (editor, config, files, etc.): `X` is not a menu option, so it
  reaches the EXIT-family gate (`commands.rs` ~L142). For a non-editor Context
  `editor_env_active` is false, so `try_exit_family(upper)` matches `X` ->
  `file.exit` = APPLICATION exit.
- Editor Context (FFEDIT active) with bare `X`: the EXIT family is SKIPPED
  (`!editor_env_active` is false, `is_equals_prefixed` is false), so bare `X`
  falls through to `ffedit_claim` -> X aliases to EXCLUDE (`dispatch.rs` ~L203,
  CR-CH-053 E8). This FFEDIT ownership of bare `X` is SEPARATE from CR-CH-052 and
  must be reconciled (see Open Questions Q4).

GOVERNING SPEC: menu-workspace Req 1g (POM `X` option = `RETURN`), Req 12.2
(Recovery_Baseline POM has `X` -> `RETURN`); command-framework Req 8.4 /
startup-and-session Req 12 (`X`/`=X` exit precedence); command-environments Req
5.1 (editor bare `X` = EXCLUDE).

### 2.4 `=X` handling TODAY

- `commands_ladder_a.rs` `try_exit_family` ~L31 literal `upper == "=X"` ->
  `file.exit`. Reached via the prelude EXIT-family gate for ANY Context (the `=`
  prefix also forces the gate open on an editor Context, `is_equals_prefixed`).
- RESULT: `=X` = APPLICATION exit everywhere, today.

GOVERNING SPEC: startup-and-session Req 12 ("WHEN the user types `EXIT`, `=X`, or
presses Ctrl+X ... THE shell SHALL initiate the application exit sequence"); Req
40; TSO Req 3 (`LOGOFF` identical to `EXIT`/`=X`).

### 2.5 END handling TODAY (already uniform -- no change needed)

- `commands_ladder_a.rs` END arm ~L212: if split, `unsplit()`; else `nav_end()`.
- `nav_stack.rs` `nav_end` ~L186: `nav_stack.pop()` -> reconstruct parent; empty
  stack -> `close_workspace_or_exit()`.
- `close_workspace_or_exit` (`nav_stack.rs` ~L238): `tabs.len() <= 1` ->
  `file.exit`; else `close_current_and_navigate_back`.

GOVERNING SPEC: menu-workspace Req 14.4 (pop one), 14.5 (empty-stack END closes
Workspace, exits when last). This ALREADY matches CR-CH-052 behaviour 4.

### 2.6 RETURN handling TODAY

- `commands_ladder_a.rs` RETURN arm ~L252 -> `nav_return` (`nav_stack.rs` ~L196):
  - non-home: `nav_stack.clear()` + `set_active_tab_home()` (jump to POM in one
    step, stack cleared).
  - home: `close_workspace_or_exit()`.

GOVERNING SPEC: menu-workspace Req 14.10 (CR-CH-038): RETURN targets the POM
(not the tab's arbitrary root), POM-active RETURN closes the one workspace.
NOTE: this is itself POM-centric ("collapse to POM"), which conflicts with the
CR's "collapse to the TOP of THIS tab's ladder". See Open Questions Q5.

### 2.7 The three nav callers that bypass the front door

| Caller | File:anchor | Calls | In-scope verbs that flow through |
|--------|-------------|-------|----------------------------------|
| POM option-key recursion | `commands.rs` ~L163 `if pom_command.to_uppercase() != *upper { self.handle_command(&pom_command); }` | `handle_command` (no `resolve_target`) | The POM option commands: `SETTINGS`, `CATALOGS`, `FILES`, `HELP`, `RETURN` (Recovery_Baseline, Req 12.2), plus any user POM option command. FILES/CATALOGS are in-scope CustomWorkspace verbs that today land on their ladder arm. |
| Chained-segment dispatch | `commands_fastpath.rs` `try_chained_fastpath` ~L128 `for segment in segments { self.handle_command(segment); }` | `handle_command` | Any option-key or verb reached by a `=0.K` / `3.1` chain, incl. CONFIG/KEYS/KINDS/etc. |
| START reconstruction | `nav_stack.rs` `apply_start_command` ~L231 `self.handle_command(&resolved)` | `handle_command` | The resolved START target verb (e.g. `FILES`, `SETTINGS`, or a POM option command). |

CONFIRMED: all three call `handle_command`, which runs prelude + ladder but NOT
`resolve_target` (`commands.rs` ~L83 doc comment states this explicitly; the
front door comment at `dispatch.rs` ~L30 confirms `resolve_target` lives only in
the front door). Hence a `FILES`/`CONFIG`/`KEYS`/... segment lands on its
"superseded" ladder arm (`commands_ladder_a.rs` ~L213 KEYS, ~L225 KINDS;
`commands_ladder_b.rs` ~L24 CONFIG/FILES/GSEARCH/COMMANDS/MENUS/LOG/CATALOGS/
PLUGINS/MACROS; `commands.rs` ~L247 bare-THEME). These arms are therefore LIVE,
not dead.

BEHAVIOUR DELTA IF REROUTED to `dispatch_command_string`: `resolve_target` runs,
so (a) a user command named like an in-scope verb now shadows the built-in (same
unification already accepted for menu-bar clicks, SESSION-STATE "owner decisions
RESOLVED"), and (b) the new uniform `=` handling applies. For a plain option key
with NO `=` and NO user-command collision, the open is IDENTICAL -- the front
door's CustomWorkspace dispatch arm calls the same shell method
(`nav_to_kind`/`open_config_view`/`open_keys_editor`/.../`open_theme_editor`) as
each ladder arm. The existing proving test
`b080_menu_bar_front_door_matches_typed_path_for_in_scope_verbs`
(`tests_focus.rs` ~L2255) establishes this equivalence pattern for the menu-bar
reroute; the same assertion shape covers the POM / chained / START callers.

---

## 3. Draft EARS acceptance criteria (OLD -> NEW per revised criterion)

Numbering continues each sub-project's existing sequence; "NEW" marks additions.

### 3.1 command-framework -- Requirement 10 (generalise `=` origin)

OLD 10.2: "WHEN a navigation chain begins with `=`, THE Navigation_Origin SHALL
be the POM (Home Context)."

NEW 10.2 (revised): "WHEN a command begins with `=`, THE shell SHALL first drop
the active tab's Navigation_Stack to EMPTY (returning the tab to the top of its
ladder), THEN execute the remainder of the command as if issued from the top of
THAT tab's ladder. The `=` prefix is a GENERAL modifier applicable to ANY
command, not only a navigation chain, and the origin is the TOP OF THE CURRENT
TAB'S LADDER, not specifically the POM."

OLD 10.3: "WHEN a navigation command does not begin with `=`, THE
Navigation_Origin SHALL be the Workspace or Context active when the command was
issued." -- UNCHANGED (restated: without `=`, the ladder is not dropped).

NEW 10.14 (addition): "THE `=` ladder-drop SHALL be applied EXACTLY ONCE, at the
single command front door (`dispatch_command_string`), BEFORE target resolution,
so that every command seam (typed line, key, menu, chained segment, POM option
key, START reconstruction) observes the identical `=` semantics."

### 3.2 menu-workspace -- Requirement 1g, 14.x (make Context-agnostic)

OLD 1g: "THE terminate action SHALL be an ordinary data-driven `menus/pom.toml`
option (default Option_Key `X`, Option_Command `RETURN`) ... Selecting it
dispatches the `RETURN` command, which returns to the Home Context from any
Workspace and, when issued from the POM as the only Workspace, terminates the
application ... consistent with `=X`."

NEW 1g (revised): "THE POM `X` option SHALL dispatch the uniform `X` command (not
`RETURN`): `X` collapses the current tab to the TOP of its Navigation_Stack in one
action, and WHEN already at the top CLOSES the Workspace. THE POM SHALL NOT be
special-cased; the same `X` semantics apply to every Workspace. The application
terminates only when the closed Workspace is the LAST one." (The default POM
`menus/pom.toml` option command changes from `RETURN` to `X`.)

NEW 14.12 (addition -- the bare `X` command): "WHEN the `X` command is issued AND
the active tab's Navigation_Stack is NON-EMPTY, THE shell SHALL collapse the tab
to the TOP of its ladder (clear the Navigation_Stack and reconstruct the root
Context) in a single action. WHEN the Navigation_Stack is EMPTY (already at the
top), THE shell SHALL CLOSE the Workspace; when it is the last open Workspace, THE
shell SHALL terminate the application."

NEW 14.13 (addition -- the `=X` command): "WHEN the `=X` command is issued, THE
shell SHALL drop the active tab's Navigation_Stack to empty and then apply `X`
from the top, which (being at the top) CLOSES the Workspace; the application
terminates only when it is the last open Workspace. `=X` SHALL NOT initiate an
unconditional application exit."

NEW 14.14 (addition -- no POM special-casing): "THE X / `=X` / END / RETURN model
SHALL be identical for every Workspace Context, including the Home Context (POM).
No Context SHALL carry a bespoke exit, return, or close rule."

OLD 14.5: "WHEN END ... AND the Navigation_Stack is EMPTY, THE shell SHALL close
the Workspace ... terminate the application instead (preserving CR-CH-016)." --
UNCHANGED (already uniform; END = pop one, close-at-empty, exit-when-last).

OLD 14.10 (RETURN -> POM): see Open Question Q5 -- the CR text does not restate
RETURN; the owner must decide whether RETURN stays "collapse to POM" (CR-CH-038)
or becomes "collapse to the top of THIS tab's ladder" to match `X`. Draft
assumes RETURN is left as CR-CH-038 defines it UNLESS the owner folds it in; if
folded, revise 14.10 to "collapse to the top of THIS tab's Navigation_Stack"
(dropping the POM target) so RETURN and bare `X` converge.

### 3.3 startup-and-session -- Requirement 12, 40 (`=X` no longer app-exit)

OLD 12: "WHEN the user types `EXIT`, `=X`, or presses Ctrl+X in any `Command
===>` field, THE shell SHALL initiate the application exit sequence."

NEW 12 (revised): "WHEN the user types `EXIT` or `QUIT` (or `LOGOFF`), THE shell
SHALL initiate the application exit sequence directly. `=X` SHALL instead CLOSE
the current Workspace (menu-workspace Req 14.13); the application terminates only
when the closed Workspace is the last open Workspace."

NEW 40a (addition): "Closing a Workspace (by `X`-at-top, `=X`, END-at-empty, or
RETURN-at-POM) SHALL terminate the application ONLY WHEN it is the LAST open
Workspace; otherwise the Workspace closes and focus moves to a remaining
Workspace."

NOTE on Ctrl+X: per command-framework Req 5 / CR-CH-054, a key only invokes the
command it is bound to. Criteria MUST be stated in terms of the COMMAND. If a key
is bound to `=X`, it now closes the workspace; if bound to `EXIT`, it still exits.
Startup-and-session Req 12's "presses Ctrl+X" clause should be reworded to "the
command the key binds" rather than hardwiring Ctrl+X to app-exit.

### 3.4 navigation-commands

NO change. navigation-commands owns editor viewport verbs (SORT/UP/DOWN/word
navigation); the workspace X/END/RETURN model lives in menu-workspace Req 14 and
command-framework Req 10. Confirmed by grep: navigation-commands has no X / END /
RETURN workspace-close criteria.

---

## 4. Draft design delta

### 4.1 One `=` ladder-drop step at the front door (mechanism 1)

Add a single `=` prelude step at the TOP of `dispatch_command_string`
(`dispatch.rs`), BEFORE `run_command_prelude` / `resolve_target`:

```
if let Some(rest) = raw.trim_start().strip_prefix('=') {
    self.tabs.active_tab_mut().nav_stack.clear();   // drop THIS tab's ladder
    self.reconstruct_root_context_in_place();        // top of the ladder
    return self.dispatch_command_string(rest.trim()); // run from the clean base
}
```

- This replaces the three ad-hoc `=` sites with ONE. The chained-fastpath origin
  pop (`insert_pom_tab`) and the `resolve_pom_option_key` strip-`=` become
  consumers of an already-stripped remainder (no `=` reaches them). This is
  exactly the E8b unification the SESSION-STATE notes deferred and tie to this CR.
- "Top of the ladder" = the tab's ROOT Context after clearing `nav_stack`. For a
  POM tab the root is the POM; for a workspace rooted at a Context via `START
  <arg>` the root is that Context. This is per-tab (mechanism 3), NOT a jump to
  the global POM -- which is the behaviour change from today's "`=` means POM".
- `=X` then naturally = drop ladder + run `X` at top = close workspace. No literal
  `"=X"` match remains in `try_exit_family`.

### 4.2 X / =X / END / RETURN mapped to nav_stack operations (mechanism 3)

| Command | Operation | nav_stack method |
|---------|-----------|------------------|
| `X` (non-empty stack) | collapse to top | NEW `nav_collapse_to_top()`: `nav_stack.clear()` + reconstruct root in place |
| `X` (empty stack / at top) | close workspace | `close_workspace_or_exit()` |
| `=X` | drop ladder then X-at-top | front-door `=` step clears stack, then `X` sees empty stack -> `close_workspace_or_exit()` |
| END | pop one | `nav_end()` (UNCHANGED) |
| END (empty) | close workspace | `nav_end()` -> `close_workspace_or_exit()` (UNCHANGED) |
| RETURN | per Q5 (CR-CH-038 today) | `nav_return()` (unchanged unless folded) |

- `X` becomes a FIRST-CLASS registered command / ladder arm handled uniformly for
  ALL Contexts, replacing the EXIT-family literal `"X"` match AND the POM
  `X`->`RETURN` option indirection. Remove `"X"` from `try_exit_family` (leave
  EXIT/QUIT/LOGOFF). The POM `menus/pom.toml` default option `X` command changes
  from `RETURN` to `X`.
- `close_workspace_or_exit` already encodes "exit only when last" -- reuse it
  unchanged for X-at-top, =X, END-at-empty.

### 4.3 Reroute the three nav callers through the front door

Change `self.handle_command(x)` to `self.dispatch_command_string(x)` at:
- `commands.rs` ~L164 (POM option-key recursion),
- `commands_fastpath.rs` ~L128 (chained segment loop),
- `nav_stack.rs` ~L231 (`apply_start_command`).

After this, every in-scope verb reaches `resolve_target` ->
`builtin_workspace_target` -> `dispatch_command_target` (the CustomWorkspace arm
calling the identical shell open method), so the 11 superseded arms +
bare-THEME branch become genuinely DEAD and are deleted. This builds ON the single
front door (mechanism 1) -- NO new dispatch path.

NOTE (recursion safety): the front door runs `resolve_target`; the Function
terminal still re-enters via `handle_command` (not the front door), so no infinite
recursion is introduced (the `dispatch.rs` ~L30 rationale is preserved).

### 4.4 App exit only on last-close (mechanism preserved)

No new mechanism: `close_workspace_or_exit` (`nav_stack.rs` ~L238) already
executes `file.exit` only when `tabs.len() <= 1`. X-at-top / =X / END-at-empty /
RETURN-at-POM all route to it. The literal app-exit for `=X` in `try_exit_family`
is removed so `=X` goes through the same close path.

### 4.5 Framework conformance

- Mechanism 1 (single front door): the `=` drop + all reroutes land on
  `dispatch_command_string`. No parallel dispatcher, no `if upper == "..."`
  intercept added outside the seam.
- Mechanism 3 (per-tab Navigation_Stack): X/=X/END operate on the ACTIVE tab's
  `nav_stack`; no second stack.
- No change to CommandTarget, WorkspaceContext, InteriorFocus, or
  WorkspaceDescriptor shapes. This is NOT a framework change requiring separate
  owner confirmation beyond approving CR-CH-052 itself.

---

## 5. Draft task list (independently completable; `[ ]` only)

- [ ] 1. Add the single front-door `=` ladder-drop step in `dispatch.rs`
      (clear `nav_stack`, reconstruct root in place, re-dispatch remainder).
      Validates: command-framework Req 10.2 (revised), 10.14 (new).
- [ ] 2. Add `nav_collapse_to_top()` to `nav_stack.rs` and a uniform `X` command
      handler that collapses-to-top when the stack is non-empty and
      `close_workspace_or_exit()` when empty. Remove `"X"` and `"=X"` from
      `try_exit_family` (leave EXIT/QUIT/LOGOFF). Validates: menu-workspace Req
      14.12, 14.13, 1g (revised).
- [ ] 3. Change the default `menus/pom.toml` / Recovery_Baseline POM option `X`
      command from `RETURN` to `X` (code-only compiled default; never written to
      disk). Validates: menu-workspace Req 1g (revised), 12.2.
- [ ] 4. Revise startup-and-session Req 12/40 wording and behaviour: `=X` =
      close-workspace; app exits only on last close; key clauses stated in terms
      of the bound command. Validates: startup-and-session Req 12, 40a.
- [ ] 5. Reroute the POM option-key recursion (`commands.rs`) through
      `dispatch_command_string`. Validates: command-framework Req 2.1, 10.14.
- [ ] 6. Reroute the chained-segment loop (`commands_fastpath.rs`) through the
      front door; make the origin pop a consumer of the already-stripped
      remainder (no `=` reaches it). Validates: menu-workspace Req 5.7, 14.7;
      command-framework Req 10.14.
- [ ] 7. Reroute START reconstruction (`nav_stack.rs apply_start_command`) through
      the front door. Validates: menu-workspace Req 14.8/14.9.
- [ ] 8. Delete the now-dead ladder arms: KEYS, KINDS (`commands_ladder_a.rs`);
      CONFIG, FILES/=FILES, GSEARCH/SEARCH, COMMANDS, MENUS, LOG,
      CATALOGS/FILE CATALOGS, PLUGINS, MACROS (`commands_ladder_b.rs`); the
      bare-THEME branch (`commands.rs`, keep `THEME <name>`). Validates: command-
      framework Req 8.4 (behaviour-equivalent via resolve_target).
- [ ] 9. Repoint any tests that call `handle_command` directly for an in-scope
      verb to the front door (per the B080 Step 7 test-repoint note). Validates:
      the test suite exercises the live path.
- [ ] 10. Full-shell egui_kittest tests proving UNIFORM behaviour across the POM
      AND a non-POM workspace: `X`-collapses-to-top, `X`-at-top-closes,
      `=X`-closes-not-app-exit, END-pops-one, app-exit-only-on-last-close, and
      that POM is NOT special-cased (bare `X` on POM behaves like bare `X` on a
      Config/Files workspace). Validates: menu-workspace Req 14.12-14.14.
- [ ] 11. Reconcile the editor (FFEDIT) bare-`X`=EXCLUDE ownership with the new
      uniform `X` (per Q4 decision) and add a test. Validates: command-
      environments Req 5.1 vs menu-workspace Req 14.14.

---

## 6. TCR NOT COVERED rows (one per new/revised criterion)

Each row uses the NOT COVERED status glyph (red circle) in the real TCR.md;
shown here as `NC` placeholder so this draft stays ASCII.

```
| `ff-desktop` | NC | -- | command-framework Req 10.2 (revised): `=` drops THIS tab's ladder and runs from its top |
| `ff-desktop` | NC | -- | command-framework Req 10.14: `=` ladder-drop applied once at the front door |
| `ff-desktop` | NC | -- | menu-workspace Req 14.12: bare `X` collapses to top; at top closes the Workspace |
| `ff-desktop` | NC | -- | menu-workspace Req 14.13: `=X` closes the Workspace (not app exit) |
| `ff-desktop` | NC | -- | menu-workspace Req 14.14: X/=X/END/RETURN identical for every Context incl. POM |
| `ff-desktop` | NC | -- | menu-workspace Req 1g (revised): POM `X` option dispatches uniform `X`, not `RETURN` |
| `ff-desktop` | NC | -- | startup-and-session Req 12 (revised): `=X` closes workspace; EXIT/QUIT/LOGOFF app-exit |
| `ff-desktop` | NC | -- | startup-and-session Req 40a: app terminates only when last Workspace closes |
```

---

## 7. RISKS

- BEHAVIOUR CHANGE, not neutral. This WILL break existing tests that assert
  today's semantics. Confirmed existing tests that encode the OLD behaviour and
  will need updating:
  - `non_editor_bare_x_still_exits` (`tests_command.rs` ~L1541) -- asserts bare
    `X` on a non-editor Context runs app-exit. Under the CR, bare `X` on a
    non-POM workspace with an empty stack CLOSES the workspace (exit only when
    last). This test must be rewritten.
  - `editor_equals_x_exits_not_exclude` (`tests_command.rs` ~L1566) -- asserts
    `=X` exits; under the CR `=X` closes the workspace.
  - Any POM test asserting `X` -> `RETURN` -> Home, and any test asserting
    `=X`/`X` -> `file.exit` unconditionally.
- The owner's gate reported ~9518-9552 workspace tests and SESSION-STATE cites
  ~454-956 shell tests. A meaningful subset of the shell tests assert X/=X/POM
  exit behaviour and will need review. Expect the shell test count to shift (some
  rewritten, +~6 new uniform-behaviour tests). This is the reason the CR is
  explicitly sequenced AFTER the behaviour-neutral B080 Step 7 (which had to stay
  testable against the existing tests).
- startup-and-session app-exit tests (Req 12/40) depend on `=X` = app-exit; they
  must be revised to the close-workspace semantics.
- FFEDIT bare-`X`=EXCLUDE (CR-CH-053 E8) is a SILENT dependency on the current
  split: today the editor claims bare `X` because the EXIT family is gated off for
  editor Contexts. The new uniform `X` must decide whether an editor Context's
  bare `X` is EXCLUDE (editor-owned) or the uniform collapse/close. This is the
  biggest cross-sub-project interaction (see Q4).
- menu-workspace Req 14.10 RETURN (CR-CH-038 "collapse to POM") silently depends
  on POM being the universal return target; if the CR converges RETURN with `X`
  (collapse to THIS tab's top), CR-CH-038 is partially revised (see Q5).
- The POM default option `X` -> `RETURN` is referenced in menu-workspace Req 1g,
  12.2, 17.2a and in the compiled `DEFAULT_POM_TOML` / `recovery_pom_menu`.
  Changing it to `X` touches the Recovery_Baseline and its tests, and the
  menu-bar `show_in_menu_bar = false` terminal-option wording.
- Detached Workspaces: `nav_return` doc says RETURN behaviour is identical docked
  vs detached (menu-and-statusbar Req 18.3/18.11). The new `X`/`=X` close path
  must preserve that; closing a detached workspace that is the last open one still
  exits. Needs an explicit test.

---

## 8. OPEN QUESTIONS for the owner

- Q1. Does `=X` close EVEN THE LAST workspace (thereby exiting the app), or is it
  "close workspace, and only exit if it was the last"? The CR text says "`=X` ...
  CLOSES the Workspace" and "closing terminates the app only when last". Draft
  assumes `=X` on the last workspace closes it and therefore exits. Confirm.

- Q2. For a workspace rooted DIRECTLY at a Context via `START <arg>` (empty
  nav_stack by construction, `nav_stack.rs` ~L224), bare `X` is already "at the
  top" -> closes immediately. Is that the intended "x from the top closes the
  workspace", i.e. a START-rooted workspace closes on the first `X`? Confirm (it
  means a directly-rooted editor/files workspace has no "collapse" step, just
  close).

- Q3. "Drop the ladder, run from the top" for `=` on a NON-POM tab: top = that
  tab's ROOT Context (per-tab), NOT the global POM. This DIVERGES from today's
  `=` = "origin is POM" (command-framework Req 10.2, menu-workspace Req 5.7). Is
  the per-tab-root interpretation correct, or should `=` still mean "go to the
  POM"? (The CR wording "top of THIS tab's ladder" implies per-tab-root; the
  legacy spec says POM. This is the core semantic the owner must pin.)

- Q4. Editor interaction: today bare `X` in an editor Context = FFEDIT EXCLUDE
  (CR-CH-053 E8), NOT close. Under "all workspaces behave the same", should bare
  `X` in an editor now COLLAPSE/CLOSE the editor workspace (dropping EXCLUDE), or
  does FFEDIT retain bare-`X`=EXCLUDE as a documented environment ownership
  exception (with `=X` as the uniform close escape hatch)? This is the single
  biggest conflict and needs an explicit ruling.

- Q5. RETURN: the CR specifies `=`, `X`, `=X`, END but NOT RETURN. Today RETURN =
  "collapse to POM" (CR-CH-038). Should RETURN be left as CR-CH-038 (collapse to
  POM), or folded into the uniform model as "collapse to the top of THIS tab's
  ladder" so RETURN and bare `X`-with-non-empty-stack converge? If RETURN stays
  POM-targeted, that is itself a POM special-case the CR says to remove -- so this
  likely needs folding.

- Q6. Bare `X` on the POM/Home: under the CR it now CLOSES the POM workspace when
  the POM is at the top of its ladder (and exits if last), rather than being a
  "return to Home" no-op. Confirm the POM is no longer special (today POM `X` ->
  `RETURN` -> stay-at-Home-or-exit).

- Q7. Detached Workspaces: confirm `X`-at-top / `=X` close the detached workspace
  (re-dock vs close is governed by CR-CH-037, where the window Close button runs
  RETURN). Should the close path differ for a detached workspace, or is it
  identical to a docked one?

- Q8. Should `X` be a REGISTERED Command_ID (so it resolves via `resolve_target`
  uniformly and is bindable/aliasable like other commands), given the Known Caveat
  in the wiring standard that command registration still means a ladder verb until
  the verb table lands? Draft assumes `X` is handled as a ladder/prelude verb for
  now, consistent with EXIT/END/RETURN.

---

## Appendix: file:line anchor index (re-grepped this run)

- `crates/ff-desktop/src/shell/commands.rs`: `run_command_line` ~L28;
  `handle_command` ~L83; `run_command_prelude` ~L107 (try_current_menu_option,
  EXIT-family gate ~L142, resolve_pom_option_key recursion ~L163,
  try_chained_fastpath ~L173); `run_command_ladder` ~L188 (active-env `=` gate
  ~L210, bare-THEME branch ~L247).
- `crates/ff-desktop/src/shell/dispatch.rs`: `dispatch_command_string` ~L46;
  `ffedit_claim` ~L96 (X->EXCLUDE alias path).
- `crates/ff-desktop/src/shell/commands_ladder_a.rs`: `try_exit_family` ~L29
  (`=X`/`X` literal match ~L31); `try_commands_a` ~L48; KEYS arm ~L213; KINDS arm
  ~L225; END arm ~L212..L249; RETURN arm ~L252.
- `crates/ff-desktop/src/shell/commands_ladder_b.rs`: `try_commands_b1` superseded
  arms note ~L24.
- `crates/ff-desktop/src/shell/commands_fastpath.rs`: `try_chained_fastpath`
  ~L78 (origin pop ~L120, segment dispatch ~L128); `resolve_pom_option_key` ~L148.
- `crates/ff-desktop/src/shell/commands_menu.rs`: `try_current_menu_option` ~L109;
  `activate_menu_option` ~L160.
- `crates/ff-desktop/src/shell/nav_stack.rs`: `navigate_to` ~L124; `nav_to_kind`
  ~L150; `nav_end` ~L186; `nav_return` ~L196; `start_new_workspace` ~L210;
  `apply_start_command` ~L227 (handle_command ~L231); `close_workspace_or_exit`
  ~L238.
- `crates/ff-command/src/command_target.rs`: `resolve_target` ~L267.
- `crates/ff-desktop/src/command_config/mod.rs`: `builtin_workspace_target_for`
  ~L152.
- Tests: `tests_focus.rs` `b080_menu_bar_front_door_matches_typed_path...` ~L2255;
  `tests_command.rs` `non_editor_bare_x_still_exits` ~L1541,
  `editor_equals_x_exits_not_exclude` ~L1566.
- Specs: command-framework Req 10 ~L267-293; menu-workspace Req 1g ~L190, Req 5.7
  ~L349, Req 12.2 ~L586, Req 14 ~L693-790 (14.4/14.5 ~L728-734, 14.10 ~L766-781);
  startup-and-session Req 12 ~L366, Req 40 ~L458, TSO Req 3 ~L542.

Content was rephrased for compliance where it summarises spec/owner text.
