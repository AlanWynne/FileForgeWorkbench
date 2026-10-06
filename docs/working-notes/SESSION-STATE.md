# Session State -- CR-CH-053 Command Environments (resume point)

Last updated: 2026-10-03 (owner OOO; Kiro continued autonomously).

This is the authoritative resume point for the Command Environments work. Read
this first, then `docs/specs/command-environments/` and `docs/status/change-log.md`
(entry CR-CH-053 "DISPATCH-MODEL CORRECTION" + "CONFIRMABLE-COMMAND PRINCIPLE").

## Where we are (one line)

CR-CH-053 phase 1 E0-E7 is owner-confirmed DONE (full gate CLEAN, 9508/9508).
E8 (editor X = EXCLUDE; environment-before-FFCMD) + E9 13.1 (FFEDIT SAVE verb,
dirty-aware) + the CR-CH-054 Ctrl+S point-fix are CODE-COMPLETE pending the
owner's full gate (scoped 956/0). E9 13.2 (dirty-aware END/CANCEL/RETURN) is HELD
for owner review (new UX + behaviour). GATED but NOT implemented: E8b, E9 13.2,
E10, confirmable CC.1-CC.7, CR-CH-053 tasks 10/11, and the NEW CR-CH-054
(keyboard->command, full EARS spec not yet written).

## The gate command (owner-run, outside Kiro)

```
.\tools\ffwb-gate.ps1
```
CLEAN = empty `tools\logs\ffwb-gate.review.log`. (NOT `verify.ps1`, NOT
`ai-review.log` -- those names are retired; steering updated.) Kiro runs only
scoped `-p ff-desktop` checks and hands off.

## OWNER: the one thing to do on return

Run `.\tools\ffwb-gate.ps1`. If CLEAN, E8 is DONE -- tell Kiro and it will flip
the CR-CH-053 E8 status to owner-confirmed and continue the slice roadmap below.
If it reports issues, paste `tools\logs\ffwb-gate.review.log`.

## Owner decisions locked THIS session (do not re-litigate)

1. Editor `X` = FFEDIT EXCLUDE, not exit. (E8, DONE.)
2. The command handler does NOT special-case verbs; it offers the string to
   resolvers in order: `=` universal -> history/menu-fastpath -> ACTIVE ENVIRONMENT
   (first crack) -> FFCMD (receives whatever the env rejects, incl. X/RETURN/EXIT)
   -> engine. No "exit family" concept. `X`/`RETURN` are FFCMD aliases for EXIT.
3. `=` is ONE universal rule: leading `=` drops the nav ladder and runs the rest
   from the POM base; the environment never sees a `=`-prefixed string. So `=X`
   in the editor = return/exit (NOT EXCLUDE). (E8 does the NARROW form; E8b does
   the full one-place unification.)
4. Ownership principle: a verb is owned by the environment where it is MEANINGFUL.
   SAVE/CANCEL/UNDO/REDO/FIND/EXCLUDE/... -> FFEDIT; theme/menu/key/kind/config
   edits each own their SAVE; workbench verbs -> FFCMD. There is NO global SAVE and
   FFCMD has NOTHING of its own to save. (E9 + E10.)
5. Chaining (`A; B; C`) = handler-owned quote-aware split + per-segment loop
   through the one front door; the environment only ever gets ONE already-split
   segment. Active environment is re-evaluated PER SEGMENT (a mid-chain `=0`
   switches context; later segments resolve in the switched-to environment).
   Continue-on-error. (E10.)
6. Confirmable commands (popup) get a universal `-Y`/`-N`; behaviour is driven by
   a source-derived `interactive` flag (typed=interactive; macro/batch/automation
   =non-interactive), NOT by chaining. Non-interactive + no switch = assume cancel
   + record + continue (fail-safe). Chaining allowed in macros (a `;`-line ==
   successive lines). (command-framework Req 16; implement with E10.)

## What is DONE (code, scoped-green; owner-confirmed where noted)

- E0-E7 (framework + FFCMD naming + FFEDIT verb migration + ladder retirement +
  dispatch.rs/dispatch_ffedit.rs split): owner-confirmed full gate CLEAN 9508/9508.
- E8 (environment-before-FFCMD; editor bare X=EXCLUDE; narrow `=`; RESET/RESET BARE
  ownership boundary): CODE-COMPLETE, scoped 953/0, clippy+fmt clean. PENDING the
  owner's full gate.

### E8 implementation summary (files touched)
- `crates/ff-desktop/src/shell/commands.rs`:
  - `run_command_prelude`: `try_exit_family` is now GATED --
    `if (!editor_env_active || is_equals_prefixed) { try_exit_family }`. It could
    NOT be removed outright because the prelude's later `resolve_pom_option_key`
    resolves bare `X` to the POM Return option; gating it lets an editor BARE verb
    fall through to the ladder's FFEDIT gate while `=X` and non-editor `X` keep
    today's exit precedence.
  - `run_command_ladder`: the FFEDIT-claim gate MOVED to the START (before
    `try_commands_a`), gated by active env = FFEDIT and skipped when `cmd`
    starts with `=` (narrow `=` rule).
- `crates/ff-desktop/src/shell/dispatch.rs`: `ffedit_claim` RESET arm DECLINES
  `RESET BARE [..]` (first arg `BARE`, case-insensitive) so it falls through to
  FFCMD's `RESET BARE` ladder arm. (RESET = FFEDIT exclusion reset; RESET BARE =
  FFCMD profile reset -- the only FFCMD verb whose first token collides.)
- Tests: `tests_command.rs` added `editor_bare_x_excludes_does_not_exit`,
  `non_editor_bare_x_still_exits`, `editor_equals_x_exits_not_exclude`,
  `editor_ffedit_verb_still_resolves_after_gate_move`; `environment.rs` replaced
  the provisional `prelude_owned_verbs_are_not_shadowable_by_ffedit` with
  `ffedit_shadows_ffcmd_x_but_not_the_other_exit_verbs` +
  `equals_prefixed_command_bypasses_the_active_environment`; kept
  `ffedit_claims_owned_verb_only_when_active_env_is_ffedit`.
- NOTE: `make_shell()` opens on an EDITOR context (active_environment=FfEdit) --
  this is why editor-X behaviour surfaces immediately in shell tests.

## What is GATED (specs written) but NOT implemented -- the slice roadmap

Tracked in `docs/specs/command-environments/tasks.md` and
`docs/specs/command-framework/tasks.md`. Order:

1. **E8b** -- unify ALL `=` handling (`=X`/`=0`/`=0.K`/`=FILES`) into ONE universal
   `=` step at the top of the shared path; make the existing POM/chained/`=FILES`
   fastpaths consumers of the already-stripped remainder. Behaviour-preserving.
   (command-environments tasks.md Task 9b.)
2. **E9** -- migrate editor-BUFFER verbs SAVE/CANCEL/UNDO/REDO into FFEDIT
   (behaviour-preserving; END/RETURN stay navigation). (Task 13 / Req 10.)
3. **E10** -- per-editing-Context SAVE (Theme/Menus/Keys/Kinds/Config each own SAVE
   for their working copy, command parity with the Save button) + general
   `;`-chaining (quote-aware split, sequential re-dispatch, per-segment env
   re-eval, per-segment `=` scope, continue-on-error). (Tasks 14, 15 / Req 11.)
4. **Confirmable commands** (command-framework Req 16, Phase confirmable-commands
   CC.1-CC.7): universal `-Y`/`-N` + `interactive` flag + decision helper; RESET
   BARE is the reference consumer. Implement WITH E10 (chaining makes the
   non-interactive single-line path reachable).
5. **Task 10** -- macro ADDRESS reconciliation (TSO->FFCMD, ISREDIT->FFEDIT; RC).
6. **Task 11** -- plugin authoring boundary (built-in envs code-only; new env =
   plugin capability).

## Kiro's autonomous session outcome (owner OOO) -- STOPPED CLEANLY

Completed while OOO: the confirmable-command gate (command-framework Req 16, docs
only) and this save point. Then evaluated the two code slices and STOPPED without
implementing either, because BOTH hit owner-level design decisions:

- **E8b (`=` unification) -- DEFERRED.** The `=` handling is entangled across
  `try_chained_fastpath` (`=0.K`/`=0;E.T`, pops to POM), `resolve_pom_option_key`
  (`=1`/`=S`, reads POM menu from any context, does NOT pop), the `=FILES`
  ladder arm, and the E8 prelude `=X` gate. Unifying them is a real semantic
  change to `=1`/`=FILES`, AND it overlaps **CR-CH-052 (PENDING GATE)** which
  itself redefines the `=` ladder-drop semantics. Deciding the unified `=`
  semantics is an architecture call for the owner (and should be coordinated with
  CR-CH-052's approval). E8's narrow `=` rule already covers the editor
  correction, so nothing is broken by waiting.

- **E9 (SAVE/CANCEL/UNDO/REDO -> FFEDIT) -- NEEDS OWNER DESIGN CLARIFICATION.**
  Investigation showed these are NOT shell-ladder arms and NOT directly-typed
  registered verbs; editor SAVE/UNDO/REDO/CANCEL reach their effect via the
  **Command_Engine terminal** (`ff-command-semantics`), the final fallback (`SAVE`
  works in batch via that pipeline; `file.save` is a reserved registered id but
  the TYPED verb is `SAVE`). So E9 is NOT the clean "manager-delegation migration"
  the E1-E5 verbs were -- it means FFEDIT claiming verbs that today flow to the
  Command_Engine, which is exactly the **FFEDIT / Command_Engine boundary** the
  EXPLAINER (section 6.2 point 2) flagged as fuzzy and DEFERRED (open question Q5).
  Resolving that boundary (does FFEDIT front these verbs, or keep delegating to the
  engine, and how is behaviour preserved?) is an owner design decision. Gated
  wording ("behaviour-preserving migration") assumed a ladder arm that does not
  exist; it likely needs a design revision before implementation.

DECISION: stop here with everything GREEN and documented rather than implement
against an unconfirmed design or make `=`/engine-boundary decisions reserved for
the owner. No code changed in this autonomous stretch beyond E8 (already done
earlier) -- only docs (the Req 16 gate + this save point).

## CR-CH-054 -- Keyboard shortcuts resolve to commands (NEW CR, gated, owner-approved design)

Owner approved a new change request: every keyboard shortcut resolves to a
COMMAND dispatched through the single front door -> active environment. Full
decisions are in docs/status/change-log.md (CR-CH-054). Summary:
- Reserved CUA keys (Ctrl+Z=UNDO, Ctrl+Y=REDO, Ctrl+C=COPY, Ctrl+X=CUT,
  Ctrl+V=PASTE, Ctrl+A=SELECT ALL, Ctrl+S=SAVE) work everywhere, not overridable.
  Non-reserved Ctrl/Alt keys configurable via the existing keys editor/TOML.
- COPY/CUT source precedence: cursor selection -> else pending `C`/`CC` line block
  (to CLIPBOARD) -> else no-op. CUT also deletes (dirty+undo). Markers cleared.
- PASTE destination precedence: cursor-in-editor -> paste at cursor; else pending
  `A`/`B` marker -> paste as WHOLE LINES after/before (always line-granular);
  cursor wins the tie-break; markers cleared after paste; clipboard retained.
- FFLINE->FFEDIT coupling: expose pending `C`/`CC` (source) and `A`/`B` (dest)
  line-command markers so COPY/CUT/PASTE can read them.
- Clipboard backend = `ff-clipboard` (wires in a currently-orphan crate).
- Layered impl: slice 1 = keyboard->command dispatch wiring + SAVE + UNDO-extract;
  slice 2 = CUT/PASTE/SELECT-ALL/COPY-unify + REDO (new feature) + ff-clipboard +
  the FFLINE accessors.
- STATUS: PENDING GATE. The change-log CR is the durable decision record; the
  formal EARS requirements/design/tasks/TCR are NOT yet written (owner was still
  refining the design -- PASTE added late; write them once it settles, likely
  command-framework Req 17 + command-environments verb additions + line-commands
  accessor + clipboard-operations backend).

### Ctrl+S point-fix -- DONE (CR-CH-054, owner-approved ahead of full gate)
`shell/update.rs` Ctrl+S now `handle_command("SAVE")` (was a direct
`save_active_tab`), so the key routes through FFEDIT's dirty-aware SAVE and the
key/verb divergence introduced by E9 13.1 is closed. Test
`ctrl_s_on_clean_editor_is_noop_via_ffedit_save` (tests_focus.rs, harness: Ctrl+S
on a clean untitled buffer = no-op, no error). Scoped: 956 passed / 0 failed,
clippy + fmt clean.

## E9 progress (2026-10-03, owner approved the design mid-OOO)

Owner refined E9 into the dirty-aware editor-verb table (command-environments
Req 10, full detail there). Status:

- **13.1 SAVE verb -- DONE** (scoped 955/0, clippy/fmt clean). FFEDIT now owns
  SAVE: clean buffer = no-op; dirty = write via `save_active_tab` + stay + clear
  flag; write-fail (incl. untitled/no-path) = stay + error. Not confirmable.
  Files: `shell/environment.rs` (alias `("SAVE","SAVE")`), `shell/dispatch.rs`
  ("SAVE" arm), `shell/dispatch_ffedit.rs` (`ffedit_save`). Tests in
  tests_command.rs (`editor_save_on_clean_buffer_is_noop`,
  `editor_save_on_dirty_untitled_errors_and_stays`).
- **13.2 dirty-aware END/CANCEL/RETURN -- HELD for owner review.** This is NEW
  behaviour (dirty END saves then leaves; dirty CANCEL/RETURN open a NEW confirm
  dialog; FFEDIT shadows the navigation verbs in the editor) and its discard-
  confirm `-Y`/`-N` path depends on the Req 16 infra (CC.1-CC.7) not yet built.
  Not built unsupervised. Fully gated (Req 10.2/10.3/10.4/10.7); the clean-leave
  delegates to `nav_end`/`nav_return`, dirty-END to `save_active_tab`, and the
  dirty CANCEL/RETURN need a new `Option<LeaveEditorConfirm>` modal (mirror the
  `reset_bare_confirm` pattern). Implement on return, ideally alongside the
  confirmable-command slice.
- **UNDO/REDO -- DEFERRED (Req 10.6).** UNDO is keyboard-only inline (not a
  command); REDO does not exist (new feature). Owner decision pending.

## OWNER: decisions needed to unblock the remaining slices

1. **E8b `=` unification**: approve the unified `=` semantics (coordinated with
   CR-CH-052). Specifically: should leading `=` ALWAYS pop to POM then run the
   remainder (making `=1`/`=FILES` pop first), or keep today's per-form behaviour?
   This is the CR-CH-052 question; E8b is its FFWB-dispatch implementation.
2. **E9 FFEDIT/Command_Engine boundary**: should FFEDIT CLAIM SAVE/CANCEL/UNDO/REDO
   (and front the engine for them), or should these stay Command_Engine verbs with
   FFEDIT only owning the verbs it already claims? If FFEDIT claims them, confirm
   the behaviour-preservation path (delegate to the same engine/handler). This also
   decides EXPLAINER Q5.
3. **E10 + confirmable (Req 16)**: ready to implement once the chaining semantics
   (handler-owned split, per-segment env re-eval, continue-on-error) are
   confirmed; the gate is written. These are the largest slices.

## Environment notes

- `ff-desktop` is a BINARY crate: `cargo test -p ff-desktop <filter>` (no --lib).
- Scoped checks only (Kiro): `cargo check/clippy/test -p ff-desktop --test-threads=1`,
  `cargo fmt`. The full `ffwb-gate.ps1` / any `--workspace` run is the OWNER's.
- Terminal is FLAKY this session: `execute_pwsh` often returns exit -1 and
  mangles echo; background processes sometimes don't flush logs and can hold the
  build lock. Mitigations: redirect to `tools\logs\*.txt` and read via file tools;
  if a background test run writes nothing for minutes, stop ALL stopped terminals
  (they hold the build lock) and re-run; check for orphan cargo/rustc with
  Get-Process (none expected). `cargo test --test-threads=1` is used because the
  B048 env-var race flakes reset_bare/profile tests under multithreaded runs.
- Known gate noise is GONE: the ff-mdx `[profile.release]` warnings were fixed
  (moved to root Cargo.toml). A clean gate now has an empty review log.

---

## SIMPLIFICATION STREAM (2026-10-03/04, owner OOO) -- re-grounded on deletion-over-addition

The feature-design stream (CR-CH-053 E8b/E9-13.2/E10, CR-CH-054, confirmable
commands) is PARKED (all gated, none built unsupervised -- see the sections
above). At the owner's direction we stepped back to CORE-FRAMEWORK
SIMPLIFICATION. A read-only re-scoping assessment was run and acted on.

### Read-only assessment -- DONE
Report: `.agents/tasks/simplification-reassessment/findings.md` (keep it; it is
the reference for the remaining simplification targets). Key findings:
- The B080 "Step 7" nav/CustomWorkspace ladder arms are NOT dead. `resolve_target`
  runs ONLY in the front door `dispatch_command_string`; four callers invoke
  `handle_command` directly and bypass it: menu-bar BUTTON clicks
  (`shell/render_chrome.rs`), POM option-key recursion (`shell/commands.rs`),
  chained fastpath segments (`shell/commands_fastpath.rs`), nav-stack
  reconstruction (`shell/nav_stack.rs`). The in-code "unreachable fallback"
  comments describe the TYPED path only. Deleting those arms would break menu
  clicks + POM nav. Making them truly dead first requires rerouting those four
  callers through the front door -- a behaviour-neutral refactor that is an OWNER
  sequencing decision (NOT done).
- The ONLY confident immediate deletion was the dead `#[allow(dead_code)]` helper
  `function_target_with_arg` + its sole test.
- Staged APIs `run_command_definition` + `NOT_IMPLEMENTED_MSG` (target_dispatch.rs)
  are dead today but intentionally staged for the Command Configurator binding
  surface -- OWNER decision whether to delete (left in place).
- Orphan crates: no confident deletions. All trace to the parked editor-decouple
  CR (incl. ff-clipboard), the standalone ffmdx app, the governance test harness,
  or un-wired-but-specced subsystems. Keep-pending-wiring.
- Ctrl+C copy + Ctrl+Z undo still bypass the dispatcher (raw ui.input in
  editor_panel/paint.rs + input.rs) -- command-parity gaps that are GATED work
  (the parked editor-decouple CR), not deletions.

### Deletion + file-splits -- CODE-COMPLETE, committed, pending owner full gate
Commit `3b75928` on `decomp-and-scrm-wave` (local, `[origin/...: ahead 1]`, NOT
pushed): "refactor: split four 400-line files and delete dead
function_target_with_arg". 14 files, +1976/-1511. Behaviour-preserving (REFACTOR
+ one dead-fn delete; NO gate needed).
- Deleted dead `function_target_with_arg` + its only test
  `single_verb_arg_split_populates_arg_param` in `shell/dispatch.rs` (kept the
  live `verb_arg_split_preserves_argument_case`).
- `main.rs` 632 -> `main.rs` + `startup_schema.rs` (register_builtin_schema) +
  `startup_env.rs` (reduce-motion FFI + logging config).
- `config_panel/render.rs` 590 -> `render.rs` + `keyboard.rs` + `commit.rs` +
  `widgets.rs` (needed a 4th file: render.rs was still 426 after the first split).
- `tab_state.rs` 404 -> `tab_state.rs` (model) + `tab_state_ctors.rs` (base_tab!
  macro + ctors; theme_editor/menus_editor #[allow(dead_code)] CR-CH-022 attrs
  preserved).
- `menu_workspace/loader.rs` 404 -> `loader.rs` (load entry pts + serde shims,
  re-exports preserved) + `validate.rs` (validators + limits).
VERIFIED (scoped): `check_line_limits.py` = "OK: no non-test .rs file exceeds 400
non-test lines"; `cargo fmt` clean; `cargo check -p ff-desktop` compiles;
`cargo clippy -p ff-desktop` no warnings; `cargo test -p ff-desktop
--test-threads=1` = 955 passed / 0 failed (was 956; -1 is the deliberate deletion).

### Workflow mishap + cleanup (so the git state makes sense on return)
The coder workflow (wf_411004c725d20406) committed the refactor to the BASE
branch `decomp-and-scrm-wave` instead of the intended `simplify/...` worktree
branch, so the semantic reviewer saw an empty branch and wrote a (now-stale)
CHANGES_REQUESTED verdict. The WORK IS CORRECT and already on the branch it needed
to end up on (`decomp-and-scrm-wave`). Cleanup done by the orchestrator:
- Aborted the workflow.
- Removed the redundant `.worktrees/simplify-splits` worktree.
- Deleted the empty `simplify/dead-code-and-file-splits` branch (was e72cebd,
  never advanced -- the refactor was never on it).
- Deleted the stale `review.json`/`review.md` under
  `.agents/tasks/simplify-splits/` (kept `plan.md`).
Current worktrees: main checkout (`decomp-and-scrm-wave` @ 3b75928) + the
unrelated `organize-project-folder-structure` worktree.

### GATE CONFIRMED CLEAN (2026-10-04) -- simplification slice DONE
Owner ran `.\tools\ffwb-gate.ps1`: verdict CLEAN, 9517 run / 9517 passed / 0
failed, 0 review lines. Count 9518 -> 9517 = the expected -1 from the deliberate
dead-test deletion. This confirms commit 3b75928 AND everything below it on
`decomp-and-scrm-wave` is now OWNER-CONFIRMED DONE:
- E8 (editor X=EXCLUDE, environment-before-FFCMD, RESET BARE boundary)
- E9 13.1 (FFEDIT dirty-aware SAVE)
- CR-CH-054 Ctrl+S point-fix
- Simplification slice: dead-code deletion + four 400-line file splits.
Branch is `[origin/decomp-and-scrm-wave: ahead 1]` (local only, not pushed).

### OWNER: simplification items awaiting your decision
1. (DONE -- gate CLEAN above.) The remaining items below are the NEXT
   simplification targets from findings.md, each needing an owner decision.
2. B080 Step 7 proper: approve rerouting the four direct-`handle_command` callers
   (render_chrome.rs menu-bar clicks, commands.rs POM recursion,
   commands_fastpath.rs segments, nav_stack.rs reconstruction) through the front
   door so the nav/CustomWorkspace ladder arms + bare-THEME branch become
   genuinely dead and deletable. Behaviour-neutral but a real sequencing change.
3. Staged-API deletion: delete `run_command_definition` + `NOT_IMPLEMENTED_MSG`
   now, or keep for the imminent Command Configurator binding surface?
4. Orphan-crate review (findings.md Section 3d) only if crate-count reduction is a
   goal; otherwise keep-pending-wiring.

---

## B080 STEP 7 (menu-bar reroute + dead-arm deletion) -- IN PROGRESS (owner approved)

Owner approved item 1 (B080 Step 7). This is the first substantive simplification
after the file-splits. A read-only plan was produced and an implement-and-review
workflow launched.

### Read-only plan -- DONE
Report: `.agents/tasks/b080-step7-reroute/plan.md` (authoritative; the coder
follows it). What it establishes:
- Scope is NARROW: reroute ONLY the menu-bar click sites in
  `shell/render_chrome.rs` (~L179 non-menu option button, ~L189 peeked submenu
  child, optionally ~L172 THEME-apply child) from `handle_command` to the front
  door `dispatch_command_string`. Conditionally the Command Palette if it can emit
  an in-scope workspace verb.
- KEEP-DIRECT (do NOT reroute in Step 7): POM recursion (commands.rs ~L158),
  chained segments (commands_fastpath.rs ~L127), nav-stack START (nav_stack.rs
  ~L232), ShellRequest (render.rs ~L147), AUTONUM/NUM string-rewrites
  (commands_ladder_c.rs), activate_menu_option FallThrough + try_menu_name_dispatch
  (commands_menu.rs), and the Function terminal (target_dispatch.rs ~L175 -- would
  infinite-recurse). These carry the `=`/prelude semantics CR-CH-052 owns.
- Behaviour-preserving: the front door's CustomWorkspace dispatch arm calls the
  IDENTICAL shell method as each ladder arm for every in-scope verb
  (nav_to_kind/open_config_view/open_keys_editor/.../open_theme_editor), so the
  open is unchanged.
- CRITICAL correction to the earlier assessment: the superseded arms are also
  reachable from the TEST SUITE (first-Tab tests call handle_command directly), so
  Step 7 = reroute + TEST-REPOINT + delete (not reroute + delete). Test-repoint is
  in-scope.
- Deletable AFTER reroute+repoint: 11 CustomWorkspace/nav arms (CONFIG, FILES,
  GSEARCH/SEARCH, COMMANDS, MENUS, LOG, CATALOGS, PLUGINS, MACROS in
  commands_ladder_b.rs; KEYS, KINDS in commands_ladder_a.rs) + the bare-THEME
  branch in commands.rs (ONLY the arg.is_empty() branch; THEME <name> apply STAYS).

### Owner decisions RESOLVED (baked into the workflow brief, do not re-litigate)
1. User-command shadowing (plan risk 2 / 2.4): ACCEPTED -- a user command named
   like an in-scope verb shadows the built-in on a bar click, same as the typed
   line already does. Intended unification.
2. CR-CH-052 sequencing (plan risk 3): Step 7 is MENU-BAR SITES ONLY; the other
   direct callers wait for CR-CH-052 (PENDING GATE).

### Implementation -- IN PROGRESS
Workflow `wf_93783b8924cbb7af` (runLabel b080-step7-reroute-and-delete). Worktree
`.worktrees/b080-step7` on branch `simplify/b080-step7-reroute`, base
`decomp-and-scrm-wave`. Commits ON THE BRANCH (the earlier splits run mistakenly
committed to base; this brief pins commit-on-branch + final rebase/ff onto
decomp-and-scrm-wave). Steps: reroute menu-bar sites -> verify green (arms still
present) -> add reroute-proving egui_kittest test -> repoint direct-calling tests
to the front door -> delete the 11 arms + bare-THEME branch -> re-verify scoped ->
note in SESSION-STATE. Expected after: ff-desktop ~956 tests (prev ~955, +1 new
reroute-proving test), 0 failed; no behaviour change. Behaviour-preserving refactor
(reroute to existing seam + delete superseded arms) -- NO requirements gate.

### OWNER on return
After the workflow completes and the semantic reviewer APPROVES, this will be
CODE-COMPLETE on branch simplify/b080-step7-reroute (rebased + fast-forwarded onto
decomp-and-scrm-wave by the finalize step). Run `.\tools\ffwb-gate.ps1` to confirm
CLEAN at full-workspace scope.

---

## CRITICAL FINDING -- the decomposition baseline is UNCOMMITTED (owner decision needed)

While attempting B080 Step 7, a workflow coder discovered (and I independently
confirmed) that the ENTIRE shell decomposition / dispatch-unify / Command
Environments working tree is UNCOMMITTED. This corrects my earlier mistaken
belief (held through most of this session) that commit 3b75928 contained the
split files.

### The evidence (verified directly)
- `git show HEAD:crates/ff-desktop/src/shell/mod.rs` at 3b75928 lists ONLY the OLD
  modules (commands, configurator, nav_stack, render, render_chrome, target_dispatch,
  update, tests, ...). It has NO `mod dispatch;`, NO `mod commands_ladder_a/b/c;`,
  NO `mod commands_fastpath;`.
- `git status --short` in the main checkout = 120 changes: ~50 modified tracked
  files PLUS ~60 UNTRACKED new source files, including the whole decomposition:
  shell/commands_ladder_a.rs, commands_ladder_b.rs, commands_ladder_c.rs,
  commands_fastpath.rs, commands_menu.rs, commands_scrm.rs, commands_session.rs,
  commands_theme.rs, dispatch_ffedit.rs, environment.rs, actions.rs, construct.rs,
  handlers.rs, state.rs, state_groups.rs, nav_reconstruct.rs, titles.rs, types.rs,
  all the render_*.rs splits, all the update_*.rs splits, and all tests_*.rs
  (tests_command/common/focus/menu_workspace/misc/nav/scrm/session/split_detach).
  Plus deletions: shell/tests.rs, panel_layout.rs, allcargo.bat, several
  docs/source-documents *.txt (replaced by *.md).
- The git log DOES show commit 3b75928 "refactor: split four 400-line files and
  delete dead function_target_with_arg" and e72cebd/291ccd5 (ffwb-gate) and
  dd6cfac (DECOMP Wave 7 merge). But the committed CONTENT at 3b75928 does not
  match its message -- the split/decomposition files are still uncommitted on
  disk. (Why the commit message and content diverge is itself an owner question.)

### What this means
- The owner's gate runs (9518 then 9517 CLEAN) and all my "scoped 955/956 green"
  were against the UNCOMMITTED WORKING TREE. The code is real, builds, and passes
  -- it is simply not committed. Nothing is lost; it is all on disk in the main
  checkout.
- My earlier narrative that "the splits workflow committed 3b75928" and "the
  file-splits are owner-confirmed done on the branch" was WRONG in the git sense:
  the file-splits (startup_schema.rs/startup_env.rs/config_panel splits/
  tab_state_ctors.rs/menu_workspace/validate.rs) are ALSO part of the uncommitted
  set, not in a commit. The CONTENT is correct and gate-clean; it is just not
  committed.
- Any git-worktree-based workflow (like B080 Step 7) CANNOT work, because a fresh
  worktree of 3b75928 lacks all the uncommitted files and will not compile the
  reroute (dispatch_command_string etc. do not exist in the committed tree).

### B080 Step 7 -- HELD (not abandoned)
The plan at .agents/tasks/b080-step7-reroute/plan.md is sound and the owner
decisions are resolved; the ONLY blocker is the uncommitted baseline. Both Step 7
workflow attempts were aborted and their worktrees/branches cleaned up
(.worktrees/b080-step7 removed; branch simplify/b080-step7-reroute deleted, was at
the clean base). The coder's BLOCKER.md and git evidence remain at
.agents/tasks/b080-step7-reroute/ for the owner.

### OWNER: the decision needed (blocking all further worktree-based work)
Commit the decomposition baseline so the working tree and HEAD agree, THEN
re-run B080 Step 7. Specifically, the owner needs to decide and do ONE of:
1. (recommended) Review the 120 uncommitted changes in the main checkout and
   COMMIT them to decomp-and-scrm-wave as the real decomposition baseline (one
   commit, or a few logical commits). After that, HEAD will contain the split
   shell + dispatch_command_string + the real CustomWorkspace dispatch arm, and
   B080 Step 7 (and any future worktree workflow) can run as written.
2. Explain/point to where the committed decomposition actually lives if it is on
   another branch I did not find (I checked: it is NOT in 3b75928's tree, and the
   other branches -- main @ 45eb7d6, feature/db*, agents/organize... -- are not
   it; the work is only in the working tree).
IMPORTANT: committing 120 files is a large, owner-level action; I did NOT do it
unsupervised while the owner is OOO. Until the baseline is committed, I cannot
run worktree-based workflows against this repo, and B080 Step 7 stays HELD.

### Git state right now
- Branch decomp-and-scrm-wave @ 3b75928, with 120 uncommitted working-tree
  changes (the decomposition). Clean apart from those.
- Worktrees: main checkout + the unrelated agents/organize-project-folder-structure.
  All simplify/* worktrees and branches cleaned up.
- Nothing committed by me this session beyond what was already in the log; no
  functional change left behind by the aborted Step 7 attempts.

---

## B080 STEP 7 -- RESCOPED to OPTION A (menu-bar reroute only); deletion folded into CR-CH-052

After four failed workflow attempts (two owner-restarts to clear a wedged
terminal/build-lock; one uncommitted-baseline block now resolved by commit
e444cfc being committed+pushed), Step 7 was done DIRECTLY in the main checkout
(owner runs the scoped cargo). During implementation a CRITICAL correction to
the plan surfaced and the owner rescoped the work.

### What SHIPPED (behaviour-neutral, Option A) -- code-complete pending owner gate
- `crates/ff-desktop/src/shell/render_chrome.rs`: the THREE menu-BAR click sites
  (dynamic THEME-list child, non-menu option button, peeked submenu child) now
  dispatch through the single front door `dispatch_command_string` instead of
  `handle_command`. This gives menu-bar command parity: a clicked verb resolves
  via `resolve_target` (same observable open as the typed line) rather than the
  ladder arm. Each site carries a `B080 Step 7` comment.
- `crates/ff-desktop/src/shell/tests_focus.rs`: ONE additive test
  `b080_menu_bar_front_door_matches_typed_path_for_in_scope_verbs` -- for each
  in-scope verb (FILES/CATALOGS/CONFIG/KEYS/KINDS/COMMANDS/LOG/PLUGINS/MACROS/
  MENUS/THEME) it asserts `dispatch_command_string(verb)` lands on the SAME
  active-tab Kind as `handle_command(verb)` and sets no open_error, proving the
  reroute is behaviour-preserving. (SEARCH omitted deliberately -- it focuses an
  existing panel rather than a tab Kind.)
- Diff is exactly those 2 files. CHECKPOINT 1 (reroute only, arms present) was
  owner-confirmed GREEN (fmt/check/clippy/test -p ff-desktop clean, ~955/0).
  Owner must re-run the scoped checks after the added test (expect +1 test), then
  commit (stage ONLY these 2 files by name -- the working tree has unrelated
  pre-existing noise: D EditorConfigProperties/ReloadEvent/Self/bool/{/git-status.txt,
  M change-log.md [now also my CR-CH-052 fold-in note], untracked .worktrees/ +
  localization task dir) and run the full ffwb-gate.ps1.

### What was NOT done, and WHY (the critical correction)
The plan said the reroute makes the 11 CustomWorkspace/nav ladder arms dead and
deletable. THAT IS WRONG. The arms are the LIVE production handler for POM-option
navigation, chained fastpath, and START reconstruction: `resolve_pom_option_key`
(commands.rs) resolves a POM key (1/2/6/8/...) to an in-scope verb and calls
`handle_command(&pom_command)`, which does NOT run `resolve_target` and so lands
on the ladder arm -- the commands.rs ~L150-153 comment says so explicitly. The
plan's Deliverable 3 self-contradicted (kept POM recursion direct, yet claimed
the arms it feeds are dead). Deleting them would break POM options + chained nav
+ START.

### Owner decision (2026-10-04): OPTION A
Ship ONLY the behaviour-neutral menu-bar reroute now; KEEP the arms; FOLD the
deletion into CR-CH-052. The deletion requires first routing the POM-option /
chained / START nav callers through the front door, which is a behaviour change
(front door runs resolve_target -> user-command shadowing + `=`/ladder-drop) that
belongs with CR-CH-052's `=`/X/nav-semantics redefinition. CR-CH-052's change-log
entry now carries a "FOLDED IN" note owning (a) rerouting those nav callers and
(b) deleting the then-dead arms. Driver: owner's principle that POM is just a
MenuWorkspace with no specialised actions -- the menu-bar click and the POM/menu
option key must reach a menu option's command via the SAME front door.

### decomp-and-scrm-wave state (2026-10-04)
In sync with origin at the commit level (origin/decomp-and-scrm-wave = e444cfc,
pushed). Working tree holds the 2 Step-7 files (to commit) PLUS unrelated
pre-existing noise listed above (NOT to be swept into the Step-7 commit; owner to
triage separately -- the junk-named D files EditorConfigProperties/ReloadEvent/
Self/bool/{ look like stray tracked artifacts worth a cleanup commit of their own).

---

## RESUME POINT (2026-10-04 end of day) -- CR-CH-052 implemented, commit HELD on ff-theme

### One-line status
CR-CH-052 (uniform navigation/exit model) is fully IMPLEMENTED and was scoped-green
(cargo test -p ff-desktop = 977 passed / 0 failed; fmt/check/clippy clean) but is
UNCOMMITTED in the working tree, with its commit HELD because a concurrent theme
session (CR for a major Themes upgrade, "B081") has left crates/ff-theme
non-compiling mid-migration. ff-theme is a dependency of ff-desktop, so CR-CH-052
cannot be re-verified or committed until ff-theme compiles again.

### What is DONE and PUSHED earlier today
- B080 Step 7 part (menu-bar reroute) -- commit 255c6b8, pushed.
- Junk-file cleanup -- commit 81fba6d, pushed.
- Localization Phase 1 (ff-i18n crate, ui.locale key, alias loader; English only,
  no translation data) -- integrated, commits 0552ff6..3493220, pushed. (Phase 2
  = string extraction Tasks 5-9, NOT started.)
- Stray worktrees cleaned up: localization worktree removed by the loc chat;
  organize-project-folder-structure worktree + branch removed by me. Only the main
  checkout remains (decomp-and-scrm-wave). feature/db* branches + main left as-is.
- CR-CH-052 requirements GATE -- commit 58cac12, pushed. APPROVED by owner.
  11 docs files: command-framework Req 10.2(rev)/10.14(new); menu-workspace Req
  1g(rev)/10-RETURN(rev)/13-16(new, referenced 14.13-14.16); startup-and-session
  Req 12/40/46 + TSO Req 3; design deltas; tasks; project-master; TCR; change-log.

### CR-CH-052 implementation -- IN WORKING TREE, NOT COMMITTED (the thing to finish)
All 13 plan items done (plan at .agents/tasks/crch052-impl/plan.md). The coder's
scoped run was 977/0 green BEFORE the ff-theme breakage appeared. Changes (all in
crates/ff-desktop, uncommitted):
- dispatch.rs: single front-door `=` step `reinitialise_active_tab_to_pom` applied
  ONCE before prelude/resolve_target/active-env; the 3 ad-hoc `=` sites now consume
  the already-stripped remainder.
- nav_stack.rs: new `nav_collapse_to_visual_root`, `nav_x`,
  `reinitialise_active_tab_to_pom`; `nav_return` repointed to collapse-to-
  Tab_Visual_Root (supersedes CR-CH-038 POM target).
- try_exit_family: dropped X and =X (now EXIT/QUIT/LOGOFF only); uniform `X` arm
  (nav_x) added to try_commands_a.
- menu_workspace/defaults.rs: DEFAULT_POM_TOML X option command RETURN -> X
  (code-only).
- Rerouted the 3 nav callers (POM option-key recursion commands.rs; chained-segment
  loop commands_fastpath.rs; START reconstruction nav_stack.rs apply_start_command)
  from handle_command to dispatch_command_string.
- Deleted the 11 superseded ladder arms (KEYS/KINDS in commands_ladder_a.rs;
  CONFIG/FILES/=FILES/GSEARCH/SEARCH/COMMANDS/MENUS/LOG/CATALOGS/FILE CATALOGS/
  PLUGINS/MACROS in commands_ladder_b.rs) + the bare-THEME branch in commands.rs
  (KEPT `THEME <name>` apply); removed the now-dead helpers import.
- Tests: rewrote old-semantics tests (non_editor_bare_x_still_exits,
  editor_equals_x_exits_not_exclude, EXIT-family/POM-X) to the new model; repointed
  ~77 in-scope-verb test calls handle_command -> dispatch_command_string; added new
  full-shell egui_kittest module crates/ff-desktop/src/shell/tests_nav_ladder.rs
  (9 tests: POM + START SETTINGS uniform behaviour, FFEDIT bare-X=EXCLUDE carve-out,
  =1-from-non-POM-tab, detached close).
- Docs flipped by the coder: TCR.md 13 CR-CH-052 rows -> PASS; menu-workspace +
  command-framework tasks.md items ticked. (change-log CR-CH-052 status note may
  still need setting to DONE-pending-gate at commit time.)

### THE BLOCKER (not ours) -- ff-theme non-compiling from the concurrent theme chat
git status shows the theme session mid-migration in crates/ff-theme:
  M crates/ff-theme/{Cargo.toml,defaults.rs,lib.rs,loader.rs,palette.rs,serialiser.rs}
  ?? crates/ff-theme/src/chrome_style.rs
  M docs/specs/theme-and-appearance/{design.md,requirements.md,tasks.md}
ChromeColours/.chrome removed from palette.rs but still referenced in
loader/defaults/serialiser/contrast (~24 compile errors). Because ff-theme is a
dep of ff-desktop, a FRESH cargo test -p ff-desktop currently fails at the ff-theme
compile step (it was clean during CR-CH-052's 977/0 run; broke afterward).
DO NOT touch any crates/ff-theme or docs/specs/theme-and-appearance file -- that is
the theme session's in-flight work; fixing it would clobber them.

### RESUME TOMORROW -- exact steps
1. Confirm the theme session has finished (or committed) its ff-theme migration so
   the tree compiles: `cargo check -p ff-theme` clean, then `cargo check -p ff-desktop`.
2. Re-verify CR-CH-052 is still green on the now-compiling tree:
   `cargo test -p ff-desktop -- --test-threads=1` (expect ~977 pass / 0 fail; the
   theme migration may have changed the ff-theme-dependent test count slightly --
   if any FAILURE is CR-CH-052-related, fix; if it is theme-API churn in ff-desktop
   call sites, that belongs to the theme chat, coordinate).
3. Commit CR-CH-052 on decomp-and-scrm-wave, staging ONLY the ff-desktop source +
   test files and the CR-CH-052 doc files BY EXACT NAME (never git add -A; do NOT
   stage any crates/ff-theme or docs/specs/theme-and-appearance file -- those are
   the theme session's). Commit message must contain NO semicolons (guard blocks
   them). Do NOT push -- owner runs the full ffwb-gate.ps1 and pushes.
4. Owner runs `.\tools\ffwb-gate.ps1` (full gate) from the MAIN checkout (not a
   worktree) to confirm CLEAN, then pushes.

### After CR-CH-052 lands
- Localization Phase 2 (Tasks 5-9, string extraction) is next. It also touches
  ff-desktop UI -- sequence AFTER CR-CH-052 commits to avoid overlap.
- Working-tree leftovers to triage sometime: M docs/status/change-log.md (small
  pre-existing edit), untracked .worktrees/ (consider .gitignore), the theme
  session's files (its own to commit).

### Environment reminders
- Single checkout = main repo, branch decomp-and-scrm-wave, in sync with origin at
  58cac12 (plus the uncommitted CR-CH-052 + theme working-tree changes).
- Flaky terminal: run one command via C:\tools\powershell7\pwsh.exe -NoProfile
  -NonInteractive -Command "<cmd>" *> tools\logs\X.txt then READ the log; never
  ;-chain; never pipe to Select-String/Format-Table/Out-File; empty log for
  minutes = wedged build lock, stop. Scoped -p checks only; owner runs the full gate.
- TWO chats share this working copy right now (this one = CR-CH-052; the other =
  theme upgrade in ff-theme). Keep code edits disjoint; stage by exact name.
