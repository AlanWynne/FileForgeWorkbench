# B080 Step 7 -- dispatch-unification ladder retirement: semantic review

Step 7 is the final step of the B080 unified-command-dispatch migration. It was
scoped as a behaviour-PRESERVING refactor: delete the Step-2 CustomWorkspace/nav
ladder arms that `resolve_target` (via `builtin_workspace_target_for`) already
shadows. The coder reports a DONE-BY-PRIOR-REMOVAL outcome -- every targeted arm
had already been deleted by CR-CH-052 (the "Step 2 follow-up") and CR-CH-053 E7
(the `commands_ladder_b2.rs` retirement), so zero arms were deletable this step
and zero deletions were invented. The brief explicitly sanctions that outcome.

I read the live front door (`dispatch.rs`), the classifier
(`command_config/mod.rs`), the CustomWorkspace dispatch arm (`target_dispatch.rs`),
both surviving ladder segments (`commands_ladder_a.rs`, `commands_ladder_b.rs`),
and the folded ladder terminal (`commands.rs`); confirmed `commands_ladder_b2.rs`
is gone; and read the coder's recorded scoped logs (check/clippy/test). The code
on disk matches every claim in the impl note.

Watch for: the impl note records "980 passed; 1 failed" (a theme-save test), but
the recorded `tools/logs/b080-step7-test.txt` on disk shows "981 passed; 0 failed"
-- a FULLY clean run (confirmed). The note was more conservative than reality; the
discrepancy is in the safe direction and does not weaken the verdict. One
pre-existing clippy doc-nit remains on `tests_command.rs:1473` (unrelated to this
step). Pre-existing non-ASCII box-drawing/em-dash characters live in comments of
files this step did not edit; out of Step-7 scope.

**Verdict**: APPROVED

## High-level view

The deletions are genuinely done and genuinely dead. The front-door order is
`dispatch_command_string` -> `=` step -> `run_command_prelude` -> `resolve_target`
(which consults `builtin_workspace_target_for`) -> `run_command_ladder`. Because
`builtin_workspace_target_for` classifies all twelve in-scope verbs to a
`CustomWorkspace` target BEFORE the ladder is reached, the former ladder arms for
those verbs can never run again, so removing them changes no observable behaviour.
The arms are gone from `commands_ladder_a.rs` (KEYS, KINDS), `commands_ladder_b.rs`
(CONFIG, FILES/=FILES, GSEARCH/SEARCH, COMMANDS, MENUS, LOG, FILE CATALOGS/CATALOGS,
PLUGINS, MACROS), and the bare-THEME branch from the folded terminal in
`commands.rs`; `commands_ladder_b2.rs` no longer exists.

The live replacement path is intact and untouched. `builtin_workspace_target_for`
and the `dispatch_command_target` CustomWorkspace arm were not modified this step;
they route each verb to the exact shell method the old arm called, and the EventLog
arm preserves the ladder's nav-then-`mark_all_read` ordering.

No live arm was removed. The THEME-apply arm (bare falls through, `THEME <name>` /
EXPORT / IMPORT still handled), the EXIT family, the deferred Function family
(EDIT/CLOSE), the FFEDIT editor path, and the out-of-scope verbs (COMMAND /
COMMAND <pos>, SNAPSHOT, CAPTURE, RESET BARE, RETRIEVE) all remain reachable on
the ladder.

The behaviour-preservation backstop is green. The recorded scoped `cargo test -p
ff-desktop` log shows 981 passed / 0 failed, including the Step-2 equivalence test
`typed_and_key_paths_reach_same_handler_for_builtin_verb`, the classifier tests,
and the `=`-escape / FFEDIT-shadow environment tests. Only scoped ff-desktop checks
ran in-agent.

<details>
<summary>Issues (0 blocking, 3 informational)</summary>

1. **Impl-note test count understated (informational)** -- the note says
   "980 passed; 1 failed"; the recorded log shows 981 passed / 0 failed. Note is
   conservative; no action required, but the note could be reconciled to the log.
2. **Pre-existing clippy doc-nit (informational, out of scope)** --
   `clippy::doc_lazy_continuation` at `tests_command.rs:1473`; not introduced by
   this step (zero code edited). Owner's full gate / theme effort can address.
3. **Pre-existing non-ASCII in comments (informational, out of scope)** --
   box-drawing (`-` glyphs) and em/en dashes appear in comments of
   `commands_ladder_a.rs` / `commands.rs`, which this step did not edit. The
   documentation rule bans non-ASCII in `.rs`; flag for a future touch, not a
   Step-7 blocker.

</details>

<details>
<summary>Details</summary>

## Front-door order and shadowing: each deleted arm is unreachable

`dispatch.rs::dispatch_command_string` runs, in order: the single `=` step
(reinitialise to POM then recurse on the stripped remainder), `run_command_prelude`
(history record, stage-1 current-menu Option_Key, EXIT family, POM / chained
fastpaths), then `ff_command::resolve_target(raw, &ShellTargetResolver)`, and only
on `Err` does it fall through to `run_command_ladder`. This matches the order the
brief specifies.

`resolve_target` consults `builtin_workspace_target` (stage 2) before the ladder.
`ShellTargetResolver::builtin_workspace_target` delegates to the free function
`builtin_workspace_target_for`, which classifies the twelve in-scope verbs:

- `=FILES` (single token) and `FILES` (arg-empty) -> `file_explorer`
- `FILE CATALOGS` (two-word) and `CATALOGS` (arg-empty) -> `files`
- `CONFIG [<ns>]` -> `config` (non-empty ns lowercased into `params["namespace"]`)
- `COMMANDS`, `LOG`, `PLUGINS`, `MACROS`, `GSEARCH`/`SEARCH`, `KINDS`, `MENUS`
  (each arg-empty) -> their respective kinds
- `KEYS [<kind>]` -> `keys` (non-empty kind case-preserved into `params["kind"]`)
- bare `THEME` -> `theme_editor`; `THEME <name>` returns `None`

Because a typed in-scope verb resolves to a `CustomWorkspace` target and is
dispatched before `run_command_ladder` is ever called, the corresponding ladder
arm is logically unreachable. The clippy run reporting zero dead-code warnings is
consistent with the arms having been physically removed (not merely shadowed in
place).

The per-verb removal is confirmed in the source:
- `commands_ladder_a.rs` carries the note "KEYS / KINDS arms DELETED (CR-CH-052,
  B080 Step 2 follow-up)" with no KEYS/KINDS branch present.
- `commands_ladder_b.rs::try_commands_b1` carries a "DELETED-ARM NOTE (CR-CH-052)"
  enumerating CONFIG, FILES/=FILES, GSEARCH/SEARCH, COMMANDS, MENUS, LOG, FILE
  CATALOGS/CATALOGS, PLUGINS, MACROS, and states the LOG mark-all-read side effect
  now lives on the CustomWorkspace EventLog dispatch. No branch for any of those
  verbs is present.
- `commands.rs::run_command_ladder` folds the former `commands_ladder_b2.rs`
  THEME arm inline; its bare branch is deleted (`if arg.is_empty() { return; }`
  falls through) and only `THEME <name>` / EXPORT / IMPORT remain.
- `file_search` for `commands_ladder_b2` returns no file.

## Live replacement path untouched (point 4)

`builtin_workspace_target_for` and the `dispatch_command_target` CustomWorkspace
arm are the live path that replaced the deleted arms, and neither was modified this
step (the impl note states zero code change, and the git working tree carries only
prior CR-CH-052 / theme-egui-rework modifications). The dispatch arm routes:

```
file_explorer/files/command_configurator/plugin_manager/macro_library
                                 -> nav_to_kind(<WorkspaceKind>)
event_log  -> nav_to_kind(EventLog) THEN notification_queue.mark_all_read()
search     -> open_or_focus_search_panel()
config     -> open_config_view(Some(ns)|None)   (ns pre-lowercased)
keys       -> open_keys_editor(kind.as_deref())  (kind case-preserved)
kinds/menus/theme_editor -> open_kinds_editor()/open_menus_editor()/open_theme_editor()
other      -> deferred-status error (no silent swallow)
```

The EventLog nav-then-mark ordering and the CONFIG/KEYS argument handling match the
former ladder arms exactly, so the observable result is identical (Req 8.4).

## No live arm removed (point 2)

Confirmed reachable on the ladder / live paths:
- THEME-apply: `run_command_ladder` keeps `THEME <name>`, `THEME EXPORT`,
  `THEME IMPORT`; a bare THEME that reached here falls through (handled by the
  front-door CustomWorkspace path on the live route).
- EXIT family (EXIT/QUIT/LOGOFF) in `try_exit_family`; X/RETURN/END/DOCK/START/
  MENU/MENU.OPEN/PFSHOW/HELP in `try_commands_a`.
- Function family EDIT and CLOSE remain on the ladder (Step 3 deferred -- the
  Function dispatch arm does not yet carry params, documented in the code).
- COMMAND / COMMAND <pos>, SNAPSHOT, CAPTURE, RESET BARE, RETRIEVE in
  `try_commands_b1`.
- FFEDIT claim (`ffedit_claim` in `dispatch.rs`, CR-CH-053) intact: nav / exclude /
  show / reset / find / change / profile / scroll / save, with the `=`-escape rule
  and the RESET BARE decline-to-FFCMD boundary preserved.

## Test evidence (point 3)

The coder's recorded `tools/logs/b080-step7-test.txt` shows:
`test result: ok. 981 passed; 0 failed; 0 ignored`. The relevant unification tests
pass: `typed_and_key_paths_reach_same_handler_for_builtin_verb`,
`builtin_workspace_target_classifies_nav_verb`,
`shell_resolver_classifies_builtin_workspace_verb`,
`equals_prefixed_command_bypasses_the_active_environment`, and
`ffedit_shadows_ffcmd_x_but_not_the_other_exit_verbs`. The recorded log is cleaner
than the impl note's own "980 passed / 1 failed" summary; either way, every
command-dispatch behaviour-preservation suite is green. I did not re-run the suite
(the brief forbids it); I read the recorded result as instructed. Test evidence is
present, so no reject-for-missing-evidence condition applies.

## ASCII and file size (point 5)

`command_config/mod.rs` non-test code ends at line 310 (test module begins there),
well under 400. The surviving ladder files and front door are each under the limit
(`commands_ladder_b.rs` ~170, `commands_ladder_a.rs` ~260, `dispatch.rs` ~340,
`target_dispatch.rs` ~290, `commands.rs` ~370 with its test module elsewhere). The
code added/changed by Step 7 is NONE (done-by-prior-removal). A workspace-wide
grep surfaces pre-existing non-ASCII (box-drawing and em/en dashes) in COMMENTS of
`commands_ladder_a.rs` and `commands.rs` -- files this step did not touch; the
`command_config/mod.rs`, `dispatch.rs`, and `target_dispatch.rs` live-path code is
ASCII in the sections reviewed. These comment glyphs are a pre-existing documentation
-rule drift, not introduced here, and are noted as informational.

## Scoped-only checks (point 6)

The recorded logs are `cargo check -p ff-desktop` (clean), `cargo clippy -p
ff-desktop --tests` (1 pre-existing doc nit, no dead-code warnings), `cargo fmt`
(clean), and `cargo test -p ff-desktop` (981/0). All are `-p ff-desktop` scoped;
no `--workspace`, no `cargo gate`, no `ffwb-gate.ps1`. The hand-off correctly
defers the full gate to the owner.

## Done-by-prior-removal validity (point 7)

The outcome is valid: the arms were removed by CR-CH-052 / CR-CH-053 E7, each
removal is documented in-code with the originating CR, and no deletions were
invented this step. The brief explicitly admits this as an APPROVED outcome. The
B080 migration/cleanup is complete; the remaining verb homes (Function family,
manager families, split/detach/workspace, standalone, scrm) are CR-CH-053's, per
the recorded Step 3-6 deferrals.

</details>

<details>
<summary>File map</summary>

- `crates/ff-desktop/src/shell/dispatch.rs` -- front door (`=` step, prelude,
  resolve_target, ladder) + FFEDIT claim; read-only confirmation of order.
- `crates/ff-desktop/src/command_config/mod.rs` -- `builtin_workspace_target_for`
  classifier (live, untouched this step); non-test code < 400 lines.
- `crates/ff-desktop/src/shell/target_dispatch.rs` -- CustomWorkspace dispatch arm
  (live, untouched this step).
- `crates/ff-desktop/src/shell/commands.rs` -- `run_command_prelude` /
  `run_command_ladder`; bare-THEME branch deleted, THEME-apply retained.
- `crates/ff-desktop/src/shell/commands_ladder_a.rs` -- KEYS/KINDS arms removed;
  EXIT family + X/RETURN/END/DOCK/START/MENU/PFSHOW/HELP retained.
- `crates/ff-desktop/src/shell/commands_ladder_b.rs` -- nav/CustomWorkspace arms
  removed; COMMAND/SNAPSHOT/CAPTURE/RESET BARE/RETRIEVE retained.
- `crates/ff-desktop/src/shell/commands_ladder_b2.rs` -- confirmed GONE.
- Evidence logs: `tools/logs/b080-step7-{check,clippy,test}.txt`.

</details>
