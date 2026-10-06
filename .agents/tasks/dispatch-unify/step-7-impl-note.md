# B080 Step 7 -- retire the superseded CustomWorkspace/nav ladder arms: impl note

Step 7 (the FINAL step) of the unified command dispatch migration (B080, Phase 3
task 7). Owner-approved as a behaviour-PRESERVING refactor (dead-arm deletion of
ladder arms that `resolve_target` already shadows); the requirements gate is
CLOSED (no new criteria). Work was DIRECTLY in the main workspace, uncommitted
(owner standing direction). First iteration (no `step-7-review.json` present).

## Outcome: DONE-BY-PRIOR-REMOVAL -- nothing to retire

Every Step-2 CustomWorkspace/nav ladder arm that Step 7 was to delete had ALREADY
been removed by CR-CH-052 (the "B080 Step 2 follow-up" deletion) and CR-CH-053 E7
(the `commands_ladder_b2.rs` retirement). ZERO arms were deletable this step; ZERO
deletions were invented. This is the explicitly-sanctioned valid Step-7 outcome in
the brief ("If NO arms turn out to be safely deletable ... that is a VALID outcome:
report 'nothing to retire -- the Step-2 arms were already removed by CR-CH-052/053'
with evidence, and mark Step 7 done-by-prior-removal. Do NOT invent deletions.").

## Per-verb unreachability + prior-removal evidence (read-only confirmation)

The front-door order is `dispatch_command_string` (dispatch.rs) -> `=` step ->
`run_command_prelude` -> `resolve_target` (via `ShellTargetResolver`) ->
`run_command_ladder`. `resolve_target` consults `builtin_workspace_target`
(stage 2) BEFORE the ladder; when it classifies a verb, the ladder arm never runs.

LIVE classifier -- `command_config/mod.rs::builtin_workspace_target_for` classifies
ALL twelve in-scope verbs to `CommandTarget::CustomWorkspace`:

| Verb(s) | workspace_kind | arg handling |
|---------|----------------|--------------|
| FILES, =FILES | `file_explorer` | FILES claims arg-empty; =FILES single-token always |
| FILE CATALOGS, CATALOGS | `files` | two-word verb / lone CATALOGS arg-empty |
| CONFIG [<ns>] | `config` | non-empty ns lowercased into params["namespace"] |
| COMMANDS | `command_configurator` | arg-empty |
| LOG | `event_log` | arg-empty |
| PLUGINS | `plugin_manager` | arg-empty |
| MACROS | `macro_library` | arg-empty |
| GSEARCH, SEARCH | `search` | arg-empty |
| KEYS [<kind>] | `keys` | non-empty kind (case-preserved) into params["kind"] |
| KINDS | `kinds` | arg-empty |
| MENUS | `menus` | arg-empty |
| THEME (bare) | `theme_editor` | bare only; THEME <name> returns None (ladder apply) |

LIVE dispatch -- `target_dispatch.rs::dispatch_command_target` CustomWorkspace arm
routes each `workspace_kind` to the EXACT shell method the former ladder arm called
(`nav_to_kind(FileExplorer/Files/CommandConfigurator/PluginManager/MacroLibrary/
EventLog)`, `open_or_focus_search_panel`, `open_config_view`, `open_keys_editor`,
`open_kinds_editor`, `open_menus_editor`, `open_theme_editor`). The EventLog arm
preserves the ladder's `nav_to_kind(EventLog)` THEN `mark_all_read()` ordering.

ARMS ALREADY DELETED (confirmed by reading the files + grep):
- `commands_ladder_a.rs` -- KEYS and KINDS arms: GONE. The file carries the note
  "KEYS / KINDS arms DELETED (CR-CH-052, B080 Step 2 follow-up)".
- `commands_ladder_b.rs` (`try_commands_b1`) -- CONFIG, FILES/=FILES, GSEARCH/
  SEARCH, COMMANDS, MENUS, LOG, FILE CATALOGS/CATALOGS, PLUGINS, MACROS arms: GONE.
  The file carries a "DELETED-ARM NOTE (CR-CH-052, B080 Step 2 follow-up)" listing
  exactly these verbs and stating the LOG mark-all-read side effect now lives on
  the CustomWorkspace EventLog dispatch. The arms that REMAIN here (COMMAND /
  COMMAND <pos>, SNAPSHOT, CAPTURE, RESET BARE, RETRIEVE) are out-of-scope and were
  correctly KEPT reachable.
- bare-THEME -- `commands_ladder_b2.rs` no longer exists (retired by CR-CH-053 E7);
  the surviving THEME arm folded inline into `run_command_ladder` (commands.rs). Its
  bare branch is DELETED (CR-CH-052): `if arg.is_empty() { return; }` falls through,
  and only `THEME <name>` (theme-apply, incl. EXPORT/IMPORT) remains -- which is
  correct, because `builtin_workspace_target_for` returns None for a non-empty THEME
  arg so the apply path still reaches this arm.

GREP CONFIRMATIONS (no residual dead arm for any of the twelve verbs):
- `verb_arg(cmd, "(KEYS|KINDS|CONFIG|FILES|GSEARCH|SEARCH|COMMANDS|MENUS|LOG|
  PLUGINS|MACROS|CATALOGS)")` across `commands*.rs`: NO matches.
- `upper == "(KINDS|MENUS|COMMANDS|LOG|PLUGINS|MACROS|FILES|CATALOGS|GSEARCH|
  SEARCH|CONFIG|KEYS)"` across `shell/**/*.rs`: matches ONLY in `tests_common.rs`
  (a test helper listing known command tokens -- not a dispatch arm).
- `try_commands_b2` / `mod commands_ladder_b2`: matches are historical COMMENTS
  only (commands.rs, commands_ladder_b.rs, dispatch.rs, dispatch_ffedit.rs); no live
  call, no module declaration. `file_search commands_ladder_b2`: no file.
- The open-method references (`open_config_view`, `open_or_focus_search_panel`,
  etc.) found in `commands_menu.rs` are the method DEFINITIONS, not ladder arms.

EQUIVALENCE BACKSTOP present: `tests_command.rs::
typed_and_key_paths_reach_same_handler_for_builtin_verb` (Step 2) proves the
CustomWorkspace path handles these verbs identically on the typed and key seams.

## Arms KEPT (out of Step 7 scope -- correctly left reachable)

- EXIT family (EXIT/QUIT/LOGOFF) in `try_exit_family`; X/RETURN/END/DOCK in
  `try_commands_a`.
- Function family EDIT/BROWSE/VIEW/CLOSE (Step 3 deferred -- param-carrying
  Function dispatch not yet built).
- THEME <name> apply (EXPORT/IMPORT/named) in `run_command_ladder`.
- COMMAND / COMMAND <pos>, SNAPSHOT, CAPTURE, RESET BARE, RETRIEVE in
  `try_commands_b1`.
- All of `commands_ladder_c.rs` (DETACH/SPLIT/UNSPLIT/FOCUS/SWAP/AUTONUM/NUM/
  SUBMIT/TIME/STATUS/CREATE/REPLACE/BROWSE/VIEW/COMPARE/WORKSPACE) -- Steps 5-6
  deferred to CR-CH-053.
- The FFEDIT claim (CR-CH-053 E1-E5/E9) in `dispatch.rs`/`dispatch_ffedit.rs`.
- `builtin_workspace_target_for` and the `dispatch_command_target` CustomWorkspace
  arm (the LIVE path that replaced the deleted arms) -- NOT changed.

## Line-count reduction / file simplification

NONE this step -- no code was edited (nothing to delete; the deletions landed in
CR-CH-052/053). No ladder segment became empty or thin as a result of Step 7
(they were already trimmed by the prior CRs). No restructuring of live code.

## Scoped verification (Kiro-run; NEVER --workspace / cargo gate / ffwb-gate.ps1)

All via the clean non-interactive pwsh7 wrapper, output redirected to logs and read
back (the terminal shows `Exit Code: -1` from the known PSReadLine mangling, so the
redirected logs are the authoritative result).

| Command | Log | Result |
|---------|-----|--------|
| `cargo fmt -p ff-desktop` | (no log; ran clean, no diffs) | clean |
| `cargo check -p ff-desktop` | `tools/logs/b080-step7-check.txt` | Finished dev profile, 0 errors, 0 warnings |
| `cargo clippy -p ff-desktop --tests` | `tools/logs/b080-step7-clippy.txt` | Finished; 1 warning -- `clippy::doc_lazy_continuation` on `tests_command.rs:1473`, a doc-comment indentation nit UNRELATED to Step 7 (pre-existing; no code edited this step). NO dead-code warnings (confirming the removed arms are truly gone). |
| `cargo test -p ff-desktop -- --test-threads=1` | `tools/logs/b080-step7-test.txt` | 980 passed; 1 failed; 0 ignored. |

The ONE failing test is
`shell::tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme`
(tests_focus.rs:1251) -- a Theme Editor interactive render-to-action test (B081 /
theme-egui-rework / CR-CH-056), asserting that clicking the Save button writes a
USER `.toml`. It is:
- NOT a command-dispatch ladder test and NOT in B080 Step 7 scope.
- NOT attributable to this step: ZERO code was changed here (git status shows only
  pre-existing modifications from the CR-CH-052 and theme-egui-rework efforts; no
  Step-7 edits). The test's own `dispatch_command_string("THEME")` ->
  "bare THEME opens the Theme Editor Context" assertion PASSES, which in fact
  re-confirms the B080 Step-7 live path (bare THEME -> CustomWorkspace("theme_editor")
  -> open_theme_editor) is correct; the panic is downstream at the Save-click file
  write, owned by the theme effort.
- Consistent with the theme effort's own verification record
  (`.agents/tasks/theme-egui-rework/phase5-verification.md`), which ran ONLY the
  scoped `ff-theme`/`ff-theme-editor` crate tests in-agent and deferred the heavy
  ff-desktop shell suite + full gate to the owner's manual step -- so this full-shell
  theme-save test was never driven green in-agent by that effort either; its status
  is pending the owner's full gate.

Every command-dispatch behaviour-preservation test -- the entire tests_command /
tests_nav / tests_split_detach / tests_session suites and the equivalence test
`typed_and_key_paths_reach_same_handler_for_builtin_verb` -- PASSED, proving the
(already-removed) arms were dead and the live CustomWorkspace path is equivalent.

Also present (prior CR-CH-052 work, not this step): `tests_nav_ladder.rs` (untracked
new file) exercising the migrated nav verbs through the front door.

## Status

Step 7 outcome: DONE-BY-PRIOR-REMOVAL (CR-CH-052 + CR-CH-053 E7). Zero code change,
zero behaviour change this step. This is the LAST concrete B080 task, so the B080
migration/cleanup is now fully complete; the remaining verb HOMES (Function family,
manager families, split/detach/workspace, standalone, scrm) are CR-CH-053's, per the
Step 3-6 deferrals recorded in the RESUME doc. Not committed; no worktree.

The single ff-desktop test failure is an in-flight theme-egui-rework (B081/CR-CH-056)
issue outside B080 scope and not caused by this step; it will be resolved by that
effort / surfaced in the owner's full gate.

## Hand-off

Scoped checks are clean for the B080 Step-7 scope (fmt clean; `cargo check -p
ff-desktop` 0 warnings; `cargo clippy -p ff-desktop --tests` only a pre-existing
unrelated doc nit; the command-dispatch test suites all pass). The full gate is the
OWNER's manual step -- please run outside Kiro:
`cargo gate --build`  (fallback: `pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1`).
The owner's gate will also surface the unrelated theme-save test for the theme effort.
