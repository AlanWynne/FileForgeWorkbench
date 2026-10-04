# Design Delta -- Unified Command Dispatch (B080)

Proposed addition to `docs/specs/command-framework/design.md`. CONFORMANCE fix
against EXISTING criteria (command-framework Req 2.1 single entry point, 2.7 no
UI mutates state outside the framework, 8.3 ordered Target_Resolution chain, 8.4
backward compatibility, 9.2/9.7 one verb/arg split at one boundary). NO new
acceptance criteria. Plain ASCII only (`--`, `->`, straight quotes).

Framework-conformance note: this BUILDS ON the existing framework
(`resolve_target` / `CommandTarget`, `open_menu_by_name`, `navigate_to`, the
`WorkspaceContext` latch, `WorkspaceDescriptor` persistence). It does NOT add a
second dispatcher or a parallel navigation stack. The single documented caveat
(wiring-standard.md "Command Registration is temporarily weaker") is exactly
what this delta closes: `builtin_workspace_target` stops being a stub and the
if-ladder becomes a verb table.

---

## 1. Target architecture -- one front door

### 1.1 The problem (confirmed by divergences.md)

- The TYPED seam (`run_command_line` -> `handle_command`, `commands.rs:27/81`)
  jumps straight to a ~60-branch if-ladder and NEVER calls `resolve_target`.
- The KEY/MENU seams DO call `resolve_target`
  (`target_dispatch.rs:66`), but `ShellTargetResolver::builtin_workspace_target`
  is a `None` stub (`command_config/mod.rs:167`), so every built-in verb
  FallThrough's to the SAME `handle_command` ladder. The two paths converge by
  accident, not by contract.
- `ff_command::execute_command` (`dispatch.rs:80`) takes an ALREADY-parsed
  `(id, params)` and never inserts the Req 9.2 `arg` param; the ladder improvises
  per-arm argument slicing (D3). So Req 9.7 (one split at one boundary) is
  unmet.

### 1.2 The target shape

Introduce ONE shell-side front door that all three seams call with a raw string:

```
dispatch_command_string(&mut self, raw: &str)
    1. stage 1  -- try_current_menu_option(raw)         (Req 8.3 stage 1)
    2. stage 1b -- resolve_pom_option_key / chained      (shell-local fastpaths)
    3. split    -- (verb, arg) = split_once(raw)         (Req 9.7, ONE place)
    4. resolve  -- ff_command::resolve_target(raw, &ShellTargetResolver)
                     stage 2 user_command_target
                     stage 2 builtin_workspace_target   <- NO LONGER a stub
                     stage 2 is_registered_command -> Function
                     stage 3 menu_name_target -> Menu
                     stage 4 macro_name_target (deferred)
    5. dispatch -- dispatch_command_target(target)  OR  Err -> engine fallback
```

- The TYPED path's `run_command_line` and the KEY path's `dispatch_key_command`
  both call `dispatch_command_string` INSIDE the single
  `begin_command_line()` / `finish_command_line()` wrap (D4) -- so the
  Command_Line_Outcome pass stays the one decision point (Req 13.1), applied
  uniformly.
- The KEY path still performs its field-merge FIRST (D2, Req 9.8), then hands the
  merged string to `dispatch_command_string`. The merge is the ONLY synthesis.
- The MENU-option path keeps dispatching an inline `[options.target]` directly
  (D10, Req 10.6); only a STRING option command enters `dispatch_command_string`.

### 1.3 `builtin_workspace_target` becomes the classifier

Replace the `None` stub (`command_config/mod.rs:167`) with a lookup into a new
verb dispatch TABLE keyed on the uppercased verb token. For a built-in verb it
returns the appropriate `CommandTarget`:

- A verb backed by a registered Command_ID -> `CommandTarget::Function { command_id, params }`
  with the Req 9.2 `arg` folded into `params` as `arg` (string).
- A verb that opens a built-in Context -> `CommandTarget::CustomWorkspace { workspace_kind, params }`
  (e.g. FILES, CONFIG, LOG, PLUGINS, MACROS, COMMANDS, KEYS, KINDS, MENUS).
- A verb that opens a menu -> handled by the EXISTING stage-3 `menu_name_target`
  (POM, SETTINGS) -- no change needed there.

Because `resolve_target` now classifies built-ins, the TYPED path routed through
`dispatch_command_string` gets the SAME classification as the key/menu path:
ONE front door (Req 2.1), ONE ordered chain (Req 8.3), same observable result
(Req 8.4).

### 1.4 Where the single verb/arg split lives

Exactly ONE split, at step 3 of `dispatch_command_string`, using the existing
`verb_arg` rule (`helpers.rs:242`): the first whitespace-delimited token is the
case-insensitive verb; the trimmed remainder is the case-PRESERVED
Argument_String (B062, D5). The verb token drives the table lookup; the
Argument_String becomes `params.arg` for Function targets (Req 9.2) and the
CustomWorkspace params where relevant. Table handlers READ `arg` instead of
re-slicing `cmd` (removes D3's per-arm parsing).

### 1.5 Command_Line_Outcome applies uniformly

`begin_command_line` / `finish_command_line` (`commands.rs:40/50`) wrap the
OUTERMOST submit for every seam exactly once; inner re-dispatches (chained
segments, AUTONUM->NUMBER) call `dispatch_command_string` WITHOUT a new wrap, as
the ladder does today (D4). History is recorded once at the boundary (D11).

---

## 2. Verb inventory -- current ladder verbs -> table entries

Built by reading `commands_ladder_a.rs`, `_b.rs`, `_b2.rs`, `_c.rs`,
`commands_fastpath.rs`, `commands_menu.rs`. Grouped: manager-backed FAMILIES are
single delegating entries; standalone verbs are listed.

### 2.1 Pre-table stages (NOT table entries -- run before resolve_target)

- Current-menu Option_Key: `try_current_menu_option` (stage 1, D6).
- POM Option_Key fastpath: `resolve_pom_option_key` (`=1`, bare `1`/`S`, D7).
- Chained fastpath: `try_chained_fastpath` (`=0.K`, `3.1`, D8).
- NAME: sets workspace name (`commands.rs`); standalone, state-mutating.

### 2.2 Function-target entries (map to registered Command_ID + `arg`)

| Verb(s) | Target / handler today |
|---------|------------------------|
| EXIT / QUIT / =X / X / LOGOFF | `file.exit` (ladder_a) |
| EDIT <path> | `file.open` with `path` (ladder_a) |
| BROWSE / VIEW <dsn> | `file.open` (ladder_c) |
| CLOSE | `tabs.close_tab` (ladder_a) -- wrap as Function or shell op |

### 2.3 CustomWorkspace / navigation entries (nav_to_kind, in place)

| Verb(s) | Workspace_Kind (ladder_b unless noted) |
|---------|----------------------------------------|
| FILES / =FILES | FileExplorer |
| FILE CATALOGS / CATALOGS | Files |
| CONFIG [<ns>] | Config (namespace param) -- `open_config_view` |
| COMMANDS | CommandConfigurator |
| LOG | EventLog (marks all read) |
| PLUGINS | PluginManager |
| MACROS | MacroLibrary |
| GSEARCH / SEARCH | Search Results -- `open_or_focus_search_panel` |
| KEYS [<kind>] | Keys editor (ladder_a) |
| KINDS | Kinds editor (ladder_a) |
| MENUS | Menus editor (ladder_b) |
| THEME (bare) | Theme editor (ladder_b2) |

### 2.4 Menu-opening entries (stage-3 menu_name_target, already unified)

| Verb(s) | Effect |
|---------|--------|
| POM | `open_menu_by_name("pom")` (in place) -- already stage 3 (CR-CH-044) |
| SETTINGS | `open_menu_by_name("settings")` (in place) -- stage 3 (CR-CH-025) |
| MENU <name> / MENU.OPEN <name> | `open_menu_by_name(arg)` (ladder_a) |
| START [arg] | `start_new_workspace` -- the ONLY tab-creator (ladder_a) |

### 2.5 Manager-backed FAMILIES (single delegating entry each)

| Family entry | Verbs | Backing manager |
|--------------|-------|-----------------|
| nav | LOCATE, TOP, BOTTOM, UP, DOWN, LEFT, RIGHT, SORT | `nav_manager` (ladder_b2) |
| exclude/show | EXCLUDE [ALL], X [ALL], SHOW [ALL], INCLUDE [ALL], RESET [EXCLUDED|ALL] | `exclude_manager` (ladder_b2) |
| find | FIND, RFIND, CHANGE, RCHANGE | `find_manager` (ladder_b2) |
| profile | CAPS [ON/OFF], NULLS, STATS, LOCK, PROFILE [kw], HILITE | `edit_profile` (ladder_b2) |
| scroll | SCROLL <amt> | `scroll_amount` (ladder_b2) |
| workspace | WORKSPACE OPEN/SAVE/SAVE AS/CLOSE/ADD ROOT/REMOVE ROOT | shell workspace ops (ladder_c) |
| split/detach | DETACH, SPLIT [DETACH|RIGHT|DOWN|...], UNSPLIT, FOCUS [OTHER], DOCK, END, RETURN, SWAP [n|LIST] | `tabs` / `detach_split` (ladder_a/c) |
| scrm | SNAPSHOT [fmt], CAPTURE <sub> | `handle_snapshot` / `handle_capture` (ladder_b) |

A family entry owns its OWN sub-parse of the Argument_String (e.g. the exclude
entry decides ALL vs text; the SPLIT entry decides DETACH vs RIGHT/DOWN) -- this
is where the D9 cross-arm ordering collapses into one entry.

### 2.6 Standalone verbs

HELP [topic], PFSHOW [scope], COMMAND [TOP|BOTTOM], AUTONUM ON/OFF, NUM ->
NUMBER (alias redirect), SUBMIT (stub), TIME, STATUS [job], CREATE/REPLACE/
COMPARE <dsn> (stubs), RESET BARE [profiles], RETRIEVE [field].

### 2.7 Terminal stage

Unresolved by table + resolve_target -> `cmd_engine.execute_command_line(cmd)`
(the ff-command-semantics engine, `commands.rs`), unchanged. On `Err`,
`open_error` is set and the Command_Line_Outcome Restores the field.

---

## 3. Ordering-hazard removal

A verb-token table keyed on the parsed verb removes every intra-ladder
order dependency enumerated in divergences.md D9:

| Hazard today | Table resolution |
|--------------|------------------|
| `COMMAND` arm before `COMMANDS` arm (ladder_b) | distinct keys `COMMAND` and `COMMANDS`; order irrelevant |
| `SPLIT DETACH` before bare `SPLIT` (ladder_c) | one `SPLIT` entry sub-parses `DETACH`/`RIGHT`/`DOWN`/bare |
| `EXCLUDE ALL` before `EXCLUDE ` prefix (ladder_b2) | one `EXCLUDE`/`X` entry sub-parses the `ALL` suffix |
| `RESET BARE` (ladder_b1) before `RESET`/`RESET ALL` (ladder_b2), across files | one `RESET` entry sub-parses `BARE`/`EXCLUDED`/`ALL`/default |
| `RETRIEVE ` prefix to catch merged field (ladder_b) | `RETRIEVE` entry reads `self.command_text` as source of truth (unchanged) |
| stage ordering across `try_commands_a/b1/b2/c` | one lookup; stages 1/1b run before resolve_target explicitly |

The shadowing rule (Req 8.10) is preserved structurally: the table (built-in
stage 2) is consulted before `menu_name_target` (stage 3), so a built-in verb
always beats a same-named user menu -- as `resolve_target` already orders it
(`command_target.rs:267`).

---

## 4. Incremental migration plan (each step behaviour-preserving, scoped-green)

Move verb FAMILIES into the table one at a time while the ladder shrinks. After
each step: `cargo check -p ff-desktop`, `cargo test -p ff-desktop`,
`cargo clippy -p ff-desktop`, `cargo fmt`; the 454 shell tests must stay green.

- Step 0 -- scaffold: add the verb-table type and `dispatch_command_string`
  front door that, for now, performs stage 1/1b then calls `handle_command`
  (pure indirection; no behaviour change). Route `run_command_line` and
  `dispatch_key_command` through it. Prove no test regresses.
- Step 1 -- split once: implement the single `(verb, arg)` split in
  `dispatch_command_string` and have it populate `params.arg` for Function
  dispatch; keep the ladder as the fallback. (Req 9.2/9.7.)
- Step 2 -- `builtin_workspace_target`: implement classification for the
  CustomWorkspace / navigation family (FILES, CONFIG, LOG, PLUGINS, MACROS,
  COMMANDS, KEYS, KINDS, MENUS, THEME-editor, SEARCH). Remove those arms from the
  ladder submodules as each is covered. (Req 8.3 stage 2.)
- Step 3 -- Function family (EXIT/EDIT/BROWSE/VIEW/CLOSE) into the table.
- Step 4 -- manager families (nav, exclude/show, find, profile, scroll) as single
  delegating entries; delete the corresponding ladder arms.
- Step 5 -- split/detach/swap and workspace families.
- Step 6 -- standalone verbs and scrm.
- Step 7 -- retire the now-empty `try_commands_a/b1/b2/c` segments; `handle_command`
  becomes the thin terminal-stage caller (engine fallback) or is folded into the
  front door. Confirm `commands_ladder_*.rs` shrink below the 400-line rule
  naturally (they are already split).

Rollback is per-step: each step leaves BOTH the table entry and (until deleted)
the ladder arm reachable; delete the arm only once its entry is test-proven.

---

## 5. Risk / verification

### 5.1 Req 8.4 behaviour-preservation obligation

Every string that resolves today must resolve to an equivalent target with the
SAME observable result. The 454 shell tests in
`crates/ff-desktop/src/shell/tests_*.rs` are the backstop (see the user's
`verify_test_split.txt`: tests_command 83, tests_focus 87,
tests_menu_workspace 96, tests_misc 71, tests_nav 44, tests_scrm 16,
tests_session 25, tests_split_detach 32). They cover:
- SWAP-key merge + field clear (`tests_command.rs:628-694`) -- D2/D4.
- Restore-on-error / clear-on-success (`tests_session.rs:160-173`) -- D4.
- POM option key typed vs clicked identical (`tests_nav.rs:625-662`) -- D7.
- POM "Settings" click vs typed SETTINGS identical (`tests_nav.rs` ~565-620) -- D1/D6.
- `verb_arg` case rule (`tests_command.rs:322`) -- D5.
- chained `=0.K`/`=0.M` incl. detached (`tests_split_detach.rs:1036`) -- D8.
- `dispatch_bound_command` resolves user def / falls through for built-in
  (`tests_command.rs:901-924`) -- D1.

### 5.2 NEW tests needed to prove the single front door (Req 2.1/2.7/8.3)

These assert the UNIFICATION itself (not currently covered):

- `typed_and_key_paths_reach_same_handler_for_builtin_verb`: dispatch a verb
  (e.g. `CAPS ON`) via `run_command_line` and via `dispatch_key_command` (empty
  field) and assert identical resulting state. (Req 2.1 single entry point.)
- `typed_path_resolves_user_definition_like_key_path`: push a user
  Command_Definition id; assert `run_command_line(<id>)` dispatches its target,
  matching `dispatch_bound_command(<id>)`. (Closes D1; Req 8.4.)
- `builtin_workspace_target_classifies_nav_verb`: unit test on
  `ShellTargetResolver` that `resolve_target("FILES", ...)` returns the
  FileExplorer CustomWorkspace target (no longer `None`). (Req 8.3 stage 2.)
- `single_verb_arg_split_populates_arg_param`: assert a Function target built
  from `DOWN 8` carries `arg = "8"` in params, split once at the boundary.
  (Req 9.2/9.7.)
- `stage1_current_menu_option_precedes_resolve_target_on_key_path`: on a
  Menu_Workspace, a key-dispatched Option_Key activates the option even if a
  same-named registered Command_ID exists. (Req 8.3 stage order; closes D6.)
- `verb_table_has_no_order_dependency_for_command_vs_commands`: dispatch
  `COMMAND` and `COMMANDS` and assert each reaches its own handler regardless of
  registration order. (Req 8.10; closes D9.)

### 5.3 Residual risk

- The engine terminal stage (`cmd_engine.execute_command_line`) must remain the
  LAST resort for strings the table + resolve_target do not own (editor pipeline
  verbs). Confirm no verb is accidentally claimed earlier.
- `builtin_workspace_target` must return `None` for anything NOT a built-in so
  user menus / macros still resolve at stages 3/4 (do not over-claim).
- Detached-window dispatch (`update_keys.rs:83`, `with_workspace_context`) must
  route through the SAME front door so a detached command line behaves
  identically (Req 2.1).

---

## 6. Folded-in divergence decisions (from divergences.md)

| ID | Decision | How this design honours it |
|----|----------|----------------------------|
| D1 | CONVERGE | typed path routed through `dispatch_command_string` -> `resolve_target` |
| D2 | PRESERVE | key-path field-merge kept as the sole string synthesis (Req 9.8) |
| D3 | CONVERGE | one `(verb,arg)` split at the boundary; `params.arg`; handlers read `arg` |
| D4 | PRESERVE | single `begin/finish_command_line` wrap at outermost submit only |
| D5 | PRESERVE | `verb_arg` rule reused: case-insensitive verb, case-preserved arg (B062) |
| D6 | CONVERGE | stage-1 current-menu Option_Key runs before resolve_target for all seams |
| D7 | CONVERGE | POM fastpath / config Option_Key run at the same pre-resolve point |
| D8 | PRESERVE | chained segments re-dispatch through the one front door; `=` origin kept |
| D9 | CONVERGE | verb-token table removes all cross-arm ordering hazards |
| D10 | PRESERVE | inline menu `[options.target]` dispatched directly, not re-resolved |
| D11 | PRESERVE | history recorded once at the boundary (executed/merged line) |

Owner decisions still open: A1 (stage-1-first ordering for key/menu path), A2
(record merged line), A3 (field-merge is the only synthesis). All three are
"preserve current typed-path behaviour and make the other seams match it";
confirm before coding.

---

## 7. Proposed TCR rows (docs/quality/TCR.md, ff-desktop section)

Not editing TCR.md. These rows PROVE command-framework Req 2.1/2.7/8.3/8.4 are
tested after the fix (NOT COVERED today -> PASS once the tests in 5.2 land):

```
| `ff-desktop` | PASS | typed_and_key_paths_reach_same_handler_for_builtin_verb | Req 2.1: single execute entry point -- typed and key seams reach one handler |
| `ff-desktop` | PASS | typed_path_resolves_user_definition_like_key_path | Req 8.4: every string that resolves today resolves equivalently via one door |
| `ff-desktop` | PASS | builtin_workspace_target_classifies_nav_verb | Req 8.3 stage 2: built-in verbs classified by resolve_target (stub removed) |
| `ff-desktop` | PASS | single_verb_arg_split_populates_arg_param | Req 9.2/9.7: one verb/arg split at the dispatch boundary -> params.arg |
| `ff-desktop` | PASS | stage1_current_menu_option_precedes_resolve_target_on_key_path | Req 8.3 stage order: current-menu Option_Key precedes later stages on all seams |
| `ff-desktop` | PASS | no_ui_mutates_state_outside_front_door | Req 2.7: command-line/menu/key state changes route through dispatch_command_string |
| `ff-desktop` | PASS | verb_table_has_no_order_dependency_for_command_vs_commands | Req 8.10: verb table removes branch-order shadowing hazards |
```
