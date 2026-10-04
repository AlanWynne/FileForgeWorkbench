# B080 Step 7 -- Reroute-Then-Delete Plan (READ-ONLY investigation)

Branch: decomp-and-scrm-wave @ 3b75928 (gate CLEAN, 9517 tests). This is a
READY-TO-APPROVE plan, not an approved one. No source was changed. ASCII only.

---

## SUMMARY ANSWER

B080 Step 7 is "make the SUPERSEDED CustomWorkspace / navigation ladder arms (and
the bare-THEME branch) genuinely dead, then delete them." They are NOT dead today
because `resolve_target` runs ONLY inside the front door `dispatch_command_string`
(dispatch.rs), while several callers invoke `handle_command` DIRECTLY and so skip
`resolve_target` and land on the ladder arm.

Two findings govern the plan:

1. Of the eight direct `handle_command` call sites, only ONE class should be
   rerouted to the front door: the menu-BAR button clicks in `render_chrome.rs`
   (3 sites). Every other direct caller is either (a) a LEGITIMATE TERMINAL that
   must stay direct (the Function-target executor in `target_dispatch.rs`, the
   AUTONUM/NUM string-rewrite redirects), or (b) a caller that ALREADY ran
   `resolve_target` on this input before falling through (`activate_menu_option`,
   `try_menu_name_dispatch` trailing token), or (c) a caller whose rerouting
   would be behaviour-NEUTRAL but is NOT SUFFICIENT to make the arms dead and
   risks the `=`/recursion model (POM recursion, chained segments, nav-stack
   reconstruction, `ShellRequest::Command`).

2. Even after rerouting every runtime caller, the ladder arms remain reachable
   from the TEST SUITE: the full-shell first-Tab tests and many unit tests call
   `shell.handle_command("MENUS" | "THEME" | "KEYS" | "KINDS" | ...)` DIRECTLY
   (tests_common.rs `assert_first_tab_lands_on_reported_interior`, tests_focus.rs,
   tests_session.rs). So deleting the arms is NOT purely a reroute follow-up: the
   tests that currently prove those arms must FIRST be repointed at the front door
   (or the behaviour re-proved through it), or the deletion breaks the build. This
   is the single most important correction to the prior assessment's "reroute then
   delete" framing, and it is an owner-visible scope item.

Behaviour-preservation is strong where it matters: the front door's
`CustomWorkspace` dispatch arm (`dispatch_command_target`, target_dispatch.rs
approx L158-250) calls the EXACT SAME shell methods as the ladder arms
(`nav_to_kind`, `open_config_view`, `open_keys_editor`, `open_kinds_editor`,
`open_menus_editor`, `open_theme_editor`, `open_or_focus_search_panel`, and for
LOG the same nav-then-mark-all-read order). So for the in-scope verbs a menu-bar
click routed through the front door produces the identical observable open.

One verb-level BLOCKER to flag, not silently reroute: `MENUS` (and the bare `M`
rows). See Section 2.4.

---

## EVIDENCE -- the reachability + precedence model

### The one resolver seam
`resolve_target` (ff-command/src/command_target.rs approx L267-305) is invoked in
exactly two shell places, both of which build the SAME `ShellTargetResolver`:
- `dispatch_command_string` (dispatch.rs approx L47-70) -- the front door.
- `resolve_and_dispatch_command` (target_dispatch.rs approx L70-85) -- used by the
  key/menu bound path and `activate_menu_option`.

`handle_command` (commands.rs approx L85-92) runs `run_command_prelude` then
`run_command_ladder` and DELIBERATELY omits `resolve_target`, because the
Function-target arm executes a resolved Command_ID by calling
`handle_command(command_id)` (target_dispatch.rs approx L174-176); if
`handle_command` re-resolved, a registered id would re-resolve to a Function and
recurse forever (documented in commands.rs approx L72-85 and dispatch.rs module
doc approx L20-33). CONFIRM: this is correct and is the reason the Function
terminal must stay direct.

### resolve_target precedence (first match wins)
command_target.rs approx L267-305:
1. user_command_target (a user Command_Definition id)
2. builtin_workspace_target  <- the in-scope CustomWorkspace/navigation family
3. is_registered_command -> Function
4. menu_name_target -> Menu
5. macro_name_target -> Macro (deferred)
6. Err -> fall through to ladder

### builtin_workspace_target_for (the classifier)
command_config/mod.rs approx L152-240 classifies, with the SAME verb/arg split and
case rules as the ladder arms:
- `=FILES`, `FILES` -> CustomWorkspace("file_explorer")
- `FILE CATALOGS`, `CATALOGS` -> "files"
- `CONFIG [ns]` -> "config" (ns lowercased into params, matching open_config_view)
- `KEYS [kind]` -> "keys" (kind case-preserved)
- `KINDS` -> "kinds"
- `MENUS` -> "menus"
- `COMMANDS` -> "command_configurator"
- `LOG` -> "event_log"
- `PLUGINS` -> "plugin_manager"
- `MACROS` -> "macro_library"
- `GSEARCH`/`SEARCH` -> "search"
- bare `THEME` -> "theme_editor"; `THEME <name>` -> None (ladder keeps theme-apply)
- any other verb, or any of the "empty-arg only" verbs WITH an arg -> None

### The CustomWorkspace dispatch arm == the ladder arm
target_dispatch.rs approx L203-250 maps each `workspace_kind` string to the same
method the ladder arm calls:
- "file_explorer"/"files"/"command_configurator"/"plugin_manager"/"macro_library"
  -> `nav_to_kind(..)` (identical to commands_ladder_b.rs arms)
- "event_log" -> `nav_to_kind(EventLog)` THEN `notification_queue.mark_all_read()`
  (same order as the LOG ladder arm, commands_ladder_b.rs approx L150-159)
- "search" -> `open_or_focus_search_panel()`
- "config" -> `open_config_view(Some(ns)|None)`
- "keys" -> `open_keys_editor(kind)`
- "kinds" -> `open_kinds_editor()`
- "menus" -> `open_menus_editor()`
- "theme_editor" -> `open_theme_editor()`
Each sets `open_error = None` exactly as the ladder arm does. This is the
behaviour-preservation backbone for the in-scope verbs.

---

## DELIVERABLE 1 -- Every direct `handle_command` caller: verdict table

Call sites found by grep of `handle_command(` in crates/ff-desktop/src (excluding
the definition, doc comments, and the batch-runner scaffold comment which is not a
call):

| # | File:line (approx) | What it dispatches | Verdict | One-sentence justification |
|---|--------------------|--------------------|---------|----------------------------|
| 1 | render_chrome.rs ~172 | menu-bar dropdown DYNAMIC child (`THEME <name>`) | KEEP-DIRECT | The dynamic children are `THEME <name>` apply commands, which `builtin_workspace_target_for` returns None for (ladder owns theme-apply), so routing them through the front door changes nothing and only the bare-THEME button (site 3) matters -- but see note: if ALL three sites are rerouted uniformly this one is still behaviour-neutral, so it MAY be rerouted with sites 2/3; classified KEEP-DIRECT only because it never hits an in-scope arm. |
| 2 | render_chrome.rs ~179 | menu-bar NON-menu option command (direct verb) | REROUTE-TO-FRONT-DOOR | A top-level bar button whose command is an in-scope verb (e.g. a `FILES`/`CONFIG`/`LOG` button) must resolve through `resolve_target` so it hits the CustomWorkspace target, not the ladder arm; the open is identical (same shell method). |
| 3 | render_chrome.rs ~189 | menu-bar PEEKED submenu child command | REROUTE-TO-FRONT-DOOR | A peeked child button for an in-scope verb (e.g. Settings -> KEYS/KINDS/MENUS, POM -> FILES/LOG) is the primary live path keeping the arms reachable; front-door routing reaches the identical open via the CustomWorkspace arm. |
| 4 | commands.rs ~158 | POM option-key recursion (`self.handle_command(&pom_command)`) | KEEP-DIRECT (do not reroute in Step 7) | This runs INSIDE `run_command_prelude`, which the front door ALSO runs; rerouting it to `dispatch_command_string` would re-run the whole prelude (history record, EXIT family, POM resolve, chained) on the resolved command and risk a double-prelude / re-resolve loop, and it interacts with CR-CH-052's pending `=` semantics -- out of scope for a behaviour-neutral Step 7. |
| 5 | commands_fastpath.rs ~127 | chained fastpath segments (`self.handle_command(segment)`) | KEEP-DIRECT (do not reroute in Step 7) | Each segment is an Option_Key dispatched against the just-opened sub-menu; it depends on the prelude's stage-1 current-menu Option_Key lookup running first, and rerouting changes the `=`-origin/segment semantics that CR-CH-052 is about -- defer. |
| 6 | nav_stack.rs ~232 | `apply_start_command` resolved START arg | KEEP-DIRECT (do not reroute in Step 7) | `start_new_workspace` already created the new tab; the comment says it dispatches so the nav arms "do the work" on that tab. Rerouting is behaviour-neutral for the in-scope verbs (same methods) but is NOT required to make the arms dead and adds `=`/prelude interaction; defer unless the owner wants full unification. |
| 7 | commands_menu.rs ~185 | `activate_menu_option` FallThrough | KEEP-DIRECT (already correct) | This path ALREADY called `resolve_and_dispatch_command` (which runs `resolve_target`) and only falls through on `FallThrough`; an in-scope verb was ALREADY classified and dispatched, so it never reaches here for those verbs -- no change needed. |
| 8 | commands_menu.rs ~225 | `try_menu_name_dispatch` trailing-token re-dispatch | KEEP-DIRECT | The trailing token is an Option_Key to activate on the now-open menu via the stage-1 current-menu lookup in the prelude; it is not an in-scope workspace verb, and rerouting would bypass the current-menu context it needs. |
| 9 | commands_ladder_c.rs ~133 | AUTONUM ON/OFF -> `NUMBER...` redirect | KEEP-DIRECT | Pure string rewrite that must re-run the FULL pipeline for the rewritten verb; NUMBER is not an in-scope workspace verb, so it never touches a superseded arm. |
| 10 | commands_ladder_c.rs ~142 | NUM -> `NUMBER [rest]` redirect | KEEP-DIRECT | Same as #9: a string-rewrite redirect, legitimately direct. |
| 11 | render.rs ~147 | `ShellRequest::Command(cmd)` from a WorkspaceContext | KEEP-DIRECT (do not reroute in Step 7) | `apply_shell_requests` is documented (render.rs approx L138-162) as routing each request "through the existing shell pipelines so the observable result is identical to the pre-framework per-arm handling"; it is an internal already-chosen effect, not a user front-door entry, and the Target variant already exists for resolved targets. Rerouting is possible later but is not needed for Step 7. |
| 12 | target_dispatch.rs ~175 | Function-target execution terminal | KEEP-DIRECT (MUST NOT TOUCH) | This is the Function executor; rerouting it causes the documented infinite recursion (resolved id -> Function -> resolve -> Function ...). Do NOT change. |
| 13 | command_palette/mod.rs (dispatch via handle_command) | Command Palette selection | KEEP-DIRECT (verify in impl) | The palette dispatches a chosen command; it is a direct caller like a menu click. If the palette can select an in-scope workspace verb, it has the SAME phantom-arm reachability as the menu bar and SHOULD be rerouted too -- FLAG for the implementer to confirm the palette's dispatch call and reroute it alongside the menu bar if it can emit in-scope verbs. |

Net: the only runtime sites that MUST be rerouted for correctness are the menu-bar
click sites 2 and 3 (and site 1 may ride along harmlessly). Site 13 (Command
Palette) must be CONFIRMED by the implementer and rerouted iff it can emit an
in-scope verb. All others stay direct.

---

## DELIVERABLE 2 -- Exact edits + behaviour-preservation for each REROUTE

### Front-door entry function (CONFIRMED by reading dispatch.rs)
Name/signature: `fn dispatch_command_string(&mut self, raw: &str)` --
`pub(super)` on `impl WorkbenchShell` in `crates/ff-desktop/src/shell/dispatch.rs`.
It runs: `run_command_prelude` -> `resolve_target` (via `ShellTargetResolver`) ->
`run_command_ladder`. This is THE front door; replace `self.handle_command(x)` with
`self.dispatch_command_string(x)` at the reroute sites.

Note: `dispatch_bound_command` (target_dispatch.rs approx L92-98) is an ALTERNATIVE
front-door-equivalent that runs `resolve_and_dispatch_command` first and falls
through to `dispatch_command_string`. For menu-bar clicks, `dispatch_command_string`
is the correct and simplest choice (it reaches `resolve_target` directly). The
implementer MAY instead use `dispatch_bound_command` for parity with the key/menu
bound seam; both reach `resolve_target`. Recommend `dispatch_command_string` for
the minimal, obviously-equivalent change.

### Edit R1 -- render_chrome.rs non-menu option button (site 2, ~L179)
Replace:
```
if ui.button(option.command.clone()).clicked() {
    self.handle_command(&option.command);
    ui.close();
}
```
with `self.dispatch_command_string(&option.command);`.

### Edit R2 -- render_chrome.rs peeked submenu child button (site 3, ~L189)
Replace:
```
if ui.button(child.command.clone()).clicked() {
    self.handle_command(&child.command);
    ui.close();
}
```
with `self.dispatch_command_string(&child.command);`.

### Edit R3 (optional, behaviour-neutral) -- render_chrome.rs dynamic child (site 1, ~L172)
`THEME <name>` resolves to None in `builtin_workspace_target_for`, so the front door
falls through to the ladder's `THEME <name>` apply branch -- identical to today.
Rerouting is OPTIONAL; doing it keeps all three bar paths uniform. If kept direct,
document why (theme-apply is ladder-owned).

### Edit R4 (conditional) -- Command Palette (site 13)
IF the palette can select an in-scope workspace verb, replace its
`handle_command(selected)` with `dispatch_command_string(selected)` for the same
reason as R1/R2. The implementer MUST read `command_palette/mod.rs` dispatch call
and confirm before editing.

### Behaviour-preservation argument (per in-scope verb)
For each verb the front door reaches `builtin_workspace_target_for` (resolve_target
stage 2) and dispatches the `CustomWorkspace`/`Menu` target whose arm calls the
IDENTICAL shell method as the ladder arm:

| Verb (bar click) | Front-door target | Method invoked | Ladder arm method | Same? |
|------------------|-------------------|----------------|-------------------|-------|
| FILES / =FILES | CustomWorkspace("file_explorer") | nav_to_kind(FileExplorer) | nav_to_kind(FileExplorer) | YES |
| FILE CATALOGS / CATALOGS | CustomWorkspace("files") | nav_to_kind(Files) | nav_to_kind(Files) | YES |
| CONFIG [ns] | CustomWorkspace("config"{ns}) | open_config_view(ns) | open_config_view(ns) | YES (ns lowercased both sides) |
| KEYS [kind] | CustomWorkspace("keys"{kind}) | open_keys_editor(kind) | open_keys_editor(kind) | YES (kind case-preserved) |
| KINDS | CustomWorkspace("kinds") | open_kinds_editor() | open_kinds_editor() | YES |
| COMMANDS | CustomWorkspace("command_configurator") | nav_to_kind(CommandConfigurator) | nav_to_kind(CommandConfigurator) | YES |
| LOG | CustomWorkspace("event_log") | nav_to_kind(EventLog)+mark_all_read | nav_to_kind(EventLog)+mark_all_read | YES (same order) |
| PLUGINS | CustomWorkspace("plugin_manager") | nav_to_kind(PluginManager) | nav_to_kind(PluginManager) | YES |
| MACROS | CustomWorkspace("macro_library") | nav_to_kind(MacroLibrary) | nav_to_kind(MacroLibrary) | YES |
| GSEARCH / SEARCH | CustomWorkspace("search") | open_or_focus_search_panel() | open_or_focus_search_panel() | YES |
| THEME (bare) | CustomWorkspace("theme_editor") | open_theme_editor()+open_error=None | open_theme_editor()+open_error=None | YES |
| MENUS | CustomWorkspace("menus") | open_menus_editor() | open_menus_editor() | YES method; see 2.4 BLOCKER on precedence |

#### 2.4 FLAGGED verb -- MENUS vs a user `menus.toml` (precedence difference, NOT silent)
Through the front door, `resolve_target` runs `builtin_workspace_target` (stage 2)
BEFORE `menu_name_target` (stage 4), so `MENUS` classifies to CustomWorkspace("menus")
-> `open_menus_editor()`. That matches the ladder arm. BUT note two precedence
subtleties the implementer and owner must confirm do not change observable effect:
- The front-door PRELUDE runs `try_current_menu_option` FIRST. A menu-bar click
  passes the option's `command` string (e.g. "KEYS"); if the ACTIVE menu happens to
  define an Option_Key equal to that token, the prelude's stage-1 current-menu
  lookup could claim it before resolve_target. For the typed path this is already
  the behaviour (the bar click currently calls handle_command, which ALSO runs the
  prelude), so rerouting does NOT introduce a new stage-1 interaction -- the prelude
  already runs today. CONFIRM: handle_command and dispatch_command_string run the
  SAME `run_command_prelude`, so stage-1 behaviour is unchanged by the reroute.
- A user command definition whose id equals an in-scope verb (stage 1 of
  resolve_target, `user_command_target`) would, through the front door, SHADOW the
  built-in (resolve_target stage 1 beats stage 2). The ladder path has no such
  stage. This is the ONE place a reroute could change effect: if a user has defined
  a command named e.g. "FILES", a bar click would now run the user definition
  instead of the built-in nav. This is almost certainly the DESIRED unified
  behaviour (it is what the typed line already does), but it IS an observable
  change for that edge case -- document it for the owner, do not treat it as a
  no-op.

---

## DELIVERABLE 3 -- Ladder arms that become dead ONLY AFTER reroute (+ test repoint)

These are the arms the Step-2 comments already mark SUPERSEDED. After R1/R2 (+R4 if
applicable) land AND the direct-calling TESTS are repointed (Section 5), they become
unreachable and deletable. File:line ranges (approx; verify at delete time):

| Verb(s) | Arm location (file : approx line range) | Notes on residual reachability |
|---------|-----------------------------------------|--------------------------------|
| CONFIG [ns] | commands_ladder_b.rs ~L33-46 (`try_commands_b1`) | Dead after reroute + test repoint. |
| FILES / =FILES | commands_ladder_b.rs ~L48-54 | Dead after reroute + test repoint. |
| GSEARCH / SEARCH | commands_ladder_b.rs ~L56-61 | Dead after reroute + test repoint. |
| COMMANDS | commands_ladder_b.rs ~L94-101 | Dead after reroute + test repoint. |
| MENUS | commands_ladder_b.rs ~L110-116 | Dead after reroute + test repoint; confirm 2.4. |
| LOG | commands_ladder_b.rs ~L148-158 | Dead after reroute + test repoint. |
| FILE CATALOGS / CATALOGS | commands_ladder_b.rs ~L160-166 | Dead after reroute + test repoint. |
| PLUGINS | commands_ladder_b.rs ~L168-174 | Dead after reroute + test repoint. |
| MACROS | commands_ladder_b.rs ~L178-185 | Dead after reroute + test repoint. |
| KEYS [kind] | commands_ladder_a.rs ~L168-178 (`try_commands_a`) | Dead after reroute + test repoint. |
| KINDS | commands_ladder_a.rs ~L180-189 | Dead after reroute + test repoint. |
| THEME (bare only) | commands.rs `run_command_ladder` ~L204-218 | DELETE ONLY the `arg.is_empty()` bare branch; the `THEME <name>` apply branch below it is NOT superseded (classifier returns None for a non-empty arg) and MUST stay. |

Arms that STAY (not superseded, still reachable and NOT deletable): in
`try_commands_b1` -- COMMAND / COMMAND <pos>, SNAPSHOT, CAPTURE, RESET BARE,
RETRIEVE. In `try_commands_a` -- the EXIT family, EDIT, START, MENU/MENU.OPEN,
CLOSE, DOCK, HELP, PFSHOW, END, RETURN. In `try_commands_c` -- all of DETACH/SPLIT/
SWAP/AUTONUM/NUM/SUBMIT/TIME/STATUS/CREATE/REPLACE/BROWSE/VIEW/COMPARE/WORKSPACE.
`THEME <name>` apply stays.

CRITICAL residual-reachability caveat (the correction to the prior assessment):
because the first-Tab and command tests call `handle_command("MENUS"|"THEME"|
"KEYS"|"KINDS"|...)` DIRECTLY (tests_common.rs `assert_first_tab_lands_on_reported_
interior` ~L341; tests_focus.rs `full_shell_menus_editor_first_tab...` ~L1130,
`full_shell_theme_editor_first_tab...` ~L1172, `full_shell_keys_first_tab...`
~L1499, `full_shell_kinds_first_tab...` ~L1672; the LOG/MACROS/SEARCH/COMMANDS/
PLUGINS asserts ~L1275-1294), these arms are reachable from TESTS even after every
runtime caller is rerouted. The delete step is therefore blocked until those tests
are repointed to the front door (change the helper and the direct calls to
`dispatch_command_string`), OR a decision is taken that the arms stay until the
tests migrate. This is why Step 7 is "reroute + test-repoint + delete", not just
"reroute + delete".

---

## DELIVERABLE 4 -- Ordered, behaviour-preserving execution plan

Reroute first, prove green, repoint tests, then delete. All commands SCOPED to
`ff-desktop` (never `--workspace`, never the gate -- owner runs that).

1. REROUTE (edits R1, R2; R3 optional; R4 only if the palette can emit an in-scope
   verb). Pure call-site swap `handle_command` -> `dispatch_command_string` at the
   menu-bar click sites. No arm deleted yet.
2. VERIFY unchanged behaviour with the arms STILL present (so any divergence is
   attributable to the reroute, not the deletion):
   - `cargo fmt -- --check`
   - `cargo check -p ff-desktop`
   - `cargo clippy -p ff-desktop`
   - `cargo test -p ff-desktop` -- all existing menu-bar / POM / first-Tab tests
     MUST stay green. (At this point the arms are still reachable from tests, so
     green here proves the reroute did not change the runtime open.)
3. ADD a reroute-proving test (menu-bar click path): a full-shell `egui_kittest`
   `build_eframe` test that opens the POM, drives the menu-bar dropdown for an
   in-scope verb (e.g. the Settings -> KEYS peeked child or a FILES bar button),
   and asserts the resulting active-tab Kind / first_interior is the SAME as the
   typed path. Model the harness on tests_focus.rs
   `full_shell_tab_reaches_settings_as_first_menu_item` (~L1088) and
   `full_shell_keys_first_tab_focuses_kind_dropdown` (~L1496). Red before the
   reroute (if the arm and front door differed) -> green after. Run `cargo test -p
   ff-desktop <name>`.
4. REPOINT the direct-calling tests to the front door so the arms lose their last
   reachability: change `tests_common.rs::assert_first_tab_lands_on_reported_interior`
   (~L341) and the direct `handle_command("...")` calls in tests_focus.rs
   (MENUS/THEME/KEYS/KINDS/LOG/MACROS/SEARCH/COMMANDS/PLUGINS first-Tab tests) and
   tests_session.rs to call `dispatch_command_string(...)` (or add a thin test
   helper that does). Re-run `cargo test -p ff-desktop` -- still green (front door
   reaches the identical open).
5. DELETE the now-dead arms listed in Deliverable 3 (the 11 CustomWorkspace/nav
   arms in commands_ladder_a.rs / commands_ladder_b.rs, and the bare-THEME branch
   in commands.rs -- bare branch ONLY). Keep `THEME <name>` apply. Update the
   SUPERSEDED-ARM NOTE comments.
6. RE-VERIFY: `cargo fmt`, `cargo check -p ff-desktop`, `cargo clippy -p
   ff-desktop`, `cargo test -p ff-desktop`. The reroute-proving test (step 3) and
   the repointed first-Tab tests (step 4) now prove the behaviour through the front
   door with the arms gone.
7. Update TCR rows for any criterion whose coverage path changed (the first-Tab
   B059 rows, menu-bar B056 rows) to reflect front-door proof.
8. HAND OFF: list the scoped commands run; prompt the owner to run
   `pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1` for the full gate.

### Tests that MUST stay green (prove menu-bar clicks + POM nav unchanged)
- tests_focus.rs `full_shell_tab_reaches_settings_as_first_menu_item` (~L1088) --
  menu-bar forward-Tab lands on the first (Settings) button.
- tests_focus.rs `full_shell_first_tab_focuses_reported_first_interior` (~L1067) --
  POM first Tab lands on the first option.
- tests_focus.rs `pom_option_1_on_pom_tab_transforms_tab_in_place` (~L68) and
  `option_2_on_pom_tab_transforms_to_file_explorer` (~L300) -- POM option-key nav.
- tests_focus.rs first-Tab tests for each in-scope workspace: MENUS (~L1127),
  THEME (~L1164), CONFIG (~L1207), KEYS (~L1496), KINDS (~L1669), and the
  `assert_first_tab_lands_on_reported_interior` set LOG/MACROS/SEARCH/COMMANDS/
  PLUGINS (~L1274-1293).
- tests_common.rs `assert_first_tab_lands_on_reported_interior` helper (~L341) --
  the shared engine for the above (this is the one to repoint in step 4).
- tests_session.rs history tests that drive `handle_command("THEME ...")` etc.
  (~L54-340) -- unaffected by the reroute (they test history/case, not nav
  classification), but re-run to confirm.

---

## DELIVERABLE 5 -- Risks / assumptions / OWNER decisions

1. DISPATCH-PATH CHANGE (owner-confirm). Per framework-conformance.md, routing a
   user action through the single dispatch seam is building ON the framework
   (good), but CHANGING which seam the menu-bar click flows through is a
   dispatch-path change the owner should confirm. This plan is READY-TO-APPROVE,
   not approved.

2. USER-DEFINED-COMMAND SHADOWING (2.4). Through the front door, a user command
   definition named identically to an in-scope verb (e.g. "FILES") would shadow the
   built-in on a bar click (resolve_target stage 1 beats stage 2). The ladder path
   had no such stage. This is almost certainly the intended unification (the typed
   line already behaves this way), but it is an observable change for that edge
   case. OWNER DECISION: accept the unification (recommended) or exclude bar clicks
   from user-command resolution.

3. CR-CH-052 INTERACTION (owner-confirm, PENDING GATE). CR-CH-052 (change-log.md
   ~L399; SESSION-STATE.md ~L125-131, 210-213) will redefine the `=` ladder-drop
   semantics (whether `=1`/`=FILES` always pop to POM then run). The reroute itself
   touches only non-`=` menu-bar clicks, so it does NOT implement CR-CH-052. BUT the
   KEEP-DIRECT decisions for the POM recursion (site 4), chained segments (site 5),
   and nav-stack START (site 6) are deliberately conservative BECAUSE those paths
   carry the `=`/prelude semantics CR-CH-052 owns. Rerouting them should be
   sequenced AFTER CR-CH-052 lands, not folded into Step 7. OWNER DECISION: confirm
   Step 7 is scoped to the menu-bar sites only and the other direct callers wait
   for CR-CH-052.

4. COMMAND PALETTE (implementer-confirm). Site 13 reroute is CONDITIONAL on the
   palette being able to emit an in-scope workspace verb. The implementer must read
   `command_palette/mod.rs` and either reroute it alongside the menu bar or record
   why it stays direct.

5. TEST-REPOINT IS IN-SCOPE, NOT A FOLLOW-UP. The arms are reachable from the test
   suite via direct `handle_command` calls; the delete step is blocked until those
   tests are repointed (step 4). ASSUMPTION: repointing the first-Tab tests to the
   front door is acceptable (it is strictly stronger -- it proves the real
   user-visible path). If the owner wants the arms kept as an explicit
   test-only fallback, the delete is deferred.

6. BARE-THEME ONLY. When deleting the THEME arm, delete ONLY the `arg.is_empty()`
   branch; the `THEME <name>` apply branch is not superseded and must remain. A
   careless whole-block delete would remove theme-apply (behaviour regression).

7. LINE-RANGE DRIFT. All line numbers are approximate on 3b75928; the files have
   been split/renamed repeatedly (the prior assessment's `function_target_with_arg`
   / `commands_ladder_c.rs` redirect lines have moved). The implementer must
   re-grep each arm by its verb string before deleting, not trust these numbers.

---

## CONCLUSION
Step 7 is safe and behaviour-preserving IF scoped precisely: reroute ONLY the
menu-bar click sites (render_chrome.rs ~L179, ~L189; optionally ~L172; conditionally
the Command Palette) from `handle_command` to `dispatch_command_string`; prove green
with the arms still present; add a menu-bar-click reroute test; repoint the direct-
calling first-Tab tests to the front door; THEN delete the 11 CustomWorkspace/nav
ladder arms plus the bare-THEME branch. The front door reaches the identical shell
method for every in-scope verb, so the open is unchanged. The two genuine owner
decisions are the user-command shadowing edge case (2.4 / risk 2) and the
sequencing against CR-CH-052 (risk 3). Do NOT reroute the POM-recursion, chained-
segment, nav-stack, ShellRequest, string-rewrite, or Function-terminal callers in
Step 7.
