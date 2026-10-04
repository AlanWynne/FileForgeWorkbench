# RESUME: ff-desktop Architectural Simplification

**Written:** 2026-10-01
**Author:** Kiro
**Working style this effort:** edits made DIRECTLY in the main workspace --
NO worktree, NO branch, NO commit (owner decision: worktrees come later, after
the app is optimised and the wiring template is standardised). All changes sit
UNCOMMITTED in the working tree for the owner to review and commit.

---

## WHAT THIS IS

A ruthless-simplification pass over the CURRENTLY-WIRED ff-desktop stack (the
`ffwb` binary and its Cargo.toml dependency-closure crates only -- orphan crates
out of scope), driven by a read-only architectural review. Goal: simplest,
cleanest, most maintainable FOUNDATION before more features are integrated, plus
a canonical wiring template for future features.

Source review report: `.agents/tasks/ffdesktop-simplification-review/review.md`
(scores: Architectural 7, Maintainability 5, Simplicity 5, Extensibility 7;
findings F1-F9, simplifications S1-S5).

---

## DELIVERED SO FAR

### Wiring standard (DONE)
Promoted to `.kiro/steering/wiring-standard.md` -- the canonical, always-included
"how to wire a new feature" rule (Feature/Command/Menu/Toolbar/Event/State/View
Registration + Testing + an 8-step checklist). Grounded in the existing framework
(WorkspaceContext + render_workspace_context, resolve_target/CommandTarget,
menus/*.toml, WorkspaceDescriptor, Kinds registry). It carries ONE caveat section
("Command Registration is temporarily weaker") that is to be REMOVED once Phase 3
task 7 (verb-table dispatch) lands -- that removal is tracked as task 9.

### Phase 1 -- Safe simplifications (DONE, owner-verified)
Workflow wf_e26563bfd4c8c342, reviewer-APPROVED, owner ran full verify.ps1 =
CLEAN.
- Task 1.1 (F7): deleted `crates/ff-desktop/src/panel_layout.rs` (whole unused
  module) + its mod line in main.rs; deleted the superseded TabManager openers
  `open_theme_editor_tab`, `open_menus_editor_tab`, `open_menu_workspace_here`
  from `tab_manager.rs`. `transform_active_pom_tab` was KEPT (has live test
  callers -- correct). Fixed stale comments in `tab_state.rs` that named the
  deleted openers.
- Task 1.4 (F3): split the 10,999-line `shell/tests.rs` into `tests_common.rs`
  (23 shared pub(crate) helpers) + 8 sibling feature modules
  (`tests_command/focus/menu_workspace/misc/nav/scrm/session/split_detach.rs`).
  454 `#[test]` before = 454 after, 0 lost/added/renamed/ignored (reviewer
  independently verified). `shell/mod.rs` rewired. Box-drawing comment
  separators converted to ASCII.
- Also cleared the pre-existing `GUTTER_CHAR_WIDTH` unused-import warning in
  `editor_panel/mod.rs` (moved the re-export behind `#[cfg(test)]`, since only
  tests use it via `super::`) and fixed a non-ASCII arrow in a comment there.

### DROPPED by owner decision -- original Phase 1 tasks 1.2 and 1.3
These were in the original plan as "delete speculative surface" but are NOT dead
code; they are framework surface mandated by LIVE APPROVED requirements, so they
are KEPT as-is:
- 1.2 (F6, notification mpsc channel: notification_rx/tx, notification_sender,
  per-frame drain) is mandated by `docs/specs/notification-system/requirements.md`
  Requirement 3 (Notification API: a Clone+Send NotificationSender over a bounded
  channel for background Tokio producers; Req 3.5 background-op completion
  notifications). The channel looks idle only because those producers are not
  wired YET.
- 1.3 (F5, WorkspaceContext ShellRequest / ShellServices.request_* surface) is
  mandated by `docs/specs/workspace-framework/requirements.md` Requirement 2
  (ShellServices access mediation; return-an-action / enqueue-a-request pattern).
- LESSON: the architectural review judged F5/F6 as "speculative" from
  `#[allow(dead_code)]` alone WITHOUT reading the specs. F1-F4 and F7 still stand.
  Do NOT re-propose deleting the notification channel or ShellServices surface
  without an EXPRESS owner-confirmed framework change (per framework-conformance).

---

## PHASE 2 -- Shell structure -- DONE (owner-verified, full gate clean)

**FINAL STATUS: DONE.** Owner ran the full gate (verify.ps1): 9495 tests run,
9495 passed, 0 failed. The only "issues" in ai-review.log are 10x the pre-existing
unrelated ff-mdx-app/ff-mdx-installer "profiles for the non root package" warnings
(not errors, not ours). Test-split invariant independently reconfirmed by
verify_test_split.txt: 454 before = 454 after, 0 missing/added. Full workspace
build + clippy clean. The semantic review of the change set returned APPROVED
(two NON-BLOCKING nits: a pre-existing non-ASCII comment sweep across shell .rs,
and a few pub(crate) fields that could be pub(super)).

Both Phase 2 tasks complete:
- 2.1 god-struct grouping: shell/state_groups.rs (FocusState/DetachSplitState/
  DirOverrides) + the WorkbenchShell struct itself moved to shell/state.rs with
  minimal pub(crate)/pub(super) field widening.
- 2.2 file splits: ALL in-scope shell files now <=400 non-test lines. update.rs
  1049 -> 352 via update_floating.rs (render_floating_tabs), update_dialogs.rs
  (render_overlays), update_dialogs_catalog.rs (render_catalog_dialogs +
  ui_params_to_ds_params), update_keys.rs. Plus the earlier mod.rs/render.rs/
  commands.rs splits. Only 4 PRE-EXISTING out-of-scope files remain >400
  (main.rs 632, config_panel/render.rs 590, tab_state.rs 404,
  menu_workspace/loader.rs 404) -- tracked separately, NOT Phase 2 scope.
- Kiro post-gate fixes applied along the way (all verified clean): the missing
  mod update_input/update_startup declarations, handle_split_region_tab ->
  pub(super), a dead TabKind import removed, GUTTER_CHAR_WIDTH and
  config_value_to_toml_value re-exports gated behind #[cfg(test)], and ASCII
  conversion of the moved comments in the new update_* files.

NOTE: Phase 2 took three workflow attempts; each died on an infrastructure/
network/token dropout (NOT code rejections), leaving partial state Kiro then
inspected and repaired by hand. The final update.rs split was done as a
single-agent run. Lesson for Phase 3: prefer tightly-scoped single-agent runs or
direct edits over long multi-step loops while the connection is unstable.

---

## PHASE 2 (historical) -- original tracking, superseded by the DONE block above

Workflow wf_2ca9220e6805b6c4 (label phase2-shell-structure). Two coordinated
behaviour-preserving refactors (NO gate):
- Task 2.1 (F4/S3): group the ~90-field WorkbenchShell god-struct into cohesive
  sub-structs (CommandLineState / FocusState / DetachSplitState / panels / test-
  only DirOverrides behind #[cfg(test)] only if grep-confirmed test-only).
  Grouping plan written to
  `.agents/tasks/ffdesktop-simplification-review/phase2-grouping-note.md`.
- Task 2.2 (F3/S2): split render.rs (~2720), commands.rs (~2638),
  tab_manager.rs (~2403), and mod.rs/update.rs if still over, to the 400
  non-test-line rule. handle_command STAYS a ladder here (moved into submodules
  only); its verb-table conversion is Phase 3 task 7, NOT Phase 2.
- Framework surfaces (notification channel, ShellServices/ShellRequest) explicitly
  protected in the brief -- may be moved into a sub-struct, never deleted.

**Phase 2 outcome (workflow wf_2ca9220e6805b6c4):** PARTIAL. The workflow FAILED
on an infrastructure error ("No valid token found" -- an auth/token dropout on
the `review` step), NOT a code rejection. State left in the working tree:

- **Task 2.1 (god-struct grouping): COMPLETE and clean.** New file
  `shell/state_groups.rs` (98 lines) with 3 sub-structs: `FocusState`
  (self.focus.*, 8 fields), `DetachSplitState` (self.detach_split.*, 6 fields),
  `DirOverrides` (self.dir_overrides.*, 5 fields, NOT cfg-gated -- resolver
  methods read them in production). 19 flat fields -> 3; 161 call-site rewrites.
  Notification channel and ShellServices/ShellRequest surfaces preserved intact
  (as mandated). Grouping rationale in phase2-grouping-note.md.
- **Task 2.2 (file splits): PARTIAL.** commands.rs was split into
  commands + 9 submodules (commands_fastpath / ladder_a / ladder_b / ladder_b2 /
  ladder_c / menu / scrm / session / theme); render.rs into render + 8 submodules
  (render_body / chrome / command_line / nav / nav_expand / nav_ops / split /
  split_region / status). handle_command remains a LADDER (correctly NOT converted
  to a table -- that is Phase 3 task 7). BUT the 400-line objective is NOT met:
  9 non-test files still exceed 400 lines (see tools/logs/phase2-linecounts.log):
    1680 shell/mod.rs   | 1442 shell/update.rs | 793 shell/render_chrome.rs
    632 main.rs         | 590 config_panel/render.rs | 437 shell/render_body.rs
    417 shell/nav_stack.rs | 404 tab_state.rs | 404 menu_workspace/loader.rs
  Of these, mod.rs / update.rs / render_chrome.rs / render_body.rs / nav_stack.rs
  are shell files in Phase 2 scope and STILL NEED SPLITTING. main.rs,
  config_panel/render.rs, tab_state.rs, menu_workspace/loader.rs were likely
  already over 400 before Phase 2 (out of the original 2.2 scope -- confirm and
  decide whether to fold them in).

- **Kiro repaired one real breakage the dead review step would have caught:**
  `tests_split_detach.rs` had 2 imports `use super::render::split_region_strip_rects;`
  for a fn that the split moved to `render_split.rs`. Fixed both to
  `super::render_split::`. The non-test `cargo check` had passed (fn is test-only),
  so only `clippy --tests` surfaced it.

- **Verification Kiro ran (scoped):** `cargo check -p ff-desktop` CLEAN;
  `cargo clippy -p ff-desktop --tests` CLEAN (both only the unrelated ff-mdx
  profile notices); build compiles. Test suite: not fully re-counted this session
  (terminal capture was flaky), but all tests COMPILE (clippy --tests clean) and
  the 2.1 run established 928 pass / 5 known-B048-flake; the only change since is
  the 2-line import path fix, which clippy --tests validated.

**Phase 2 is therefore NOT DONE.** It needs: (a) finish splitting the remaining
in-scope shell files under 400 lines, (b) a proper code review (the one that
died), and (c) the owner's full verify.ps1 gate.

---

## REMAINING WORK (task list ids)

- [ ] 2.1 Group WorkbenchShell into sub-structs (in flight, Phase 2 workflow).
- [ ] 2.2 Split render.rs/commands.rs/tab_manager.rs to 400 lines (in flight).
- [ ] 3.1 (task 7) Convert handle_command ladder to a verb dispatch table AND
      implement `builtin_workspace_target` so `resolve_target` becomes the single
      front door for typed + menu + key seams (F1/F2/S1). This is the ONE
      genuinely BEHAVIOURAL change in the whole plan -- it is NOT a pure refactor
      and MUST go through the requirements gate (workflow.md): write/adjust
      criteria for the unified-dispatch behaviour before any code. Large but
      incremental (move verb families one at a time). Unblocks the clean Command
      Registration step in the wiring standard.
- [ ] 3.2 (task 8) Move kind-specific tab state off TabState behind TabKind
      (F4 tail). Optional follow-on to 2.1.
- [ ] 3.3 (task 9) Remove the "Command Registration is temporarily weaker" caveat
      from `.kiro/steering/wiring-standard.md` once task 7 lands.

---

## HOW TO RESUME (next session)

Phases 1 and 2 are DONE (owner-verified full gate clean). Everything is
UNCOMMITTED in the working tree per owner direction (no worktree/branch/commit
yet). Remaining work:

1. **Phase 3 task 7 -- verb-table dispatch (NEXT, needs the REQUIREMENTS GATE).**
   This is the ONE behavioural change in the plan, so it is NOT a refactor and
   MUST run the gate before any code. Convert the handle_command ladder (now
   spread across shell/commands_ladder_*.rs after Phase 2) to a verb dispatch
   table and implement builtin_workspace_target so resolve_target is the single
   front door for typed + menu + key seams (F1/F2/S1). Gate steps: identify the
   sub-project (likely command-framework / command-semantics / command-palette
   under docs/specs), draft EARS criteria for "typed, menu, and key seams all
   resolve through resolve_target to one handler", update
   design/tasks/project-master/TCR, and get OWNER APPROVAL before code.
2. **Phase 3 task 8 -- move kind-specific tab state off TabState behind TabKind**
   (F4 tail; optional follow-on to 2.1).
3. **Phase 3 task 9 -- remove the Command Registration caveat** from
   .kiro/steering/wiring-standard.md once task 7 lands.

### Separate, non-blocking follow-ups (not part of the phases)
- The semantic review's two nits: (a) a pre-existing non-ASCII comment sweep
  across shell .rs files (77 lines carried verbatim; HEAD had 121+), and (b) a
  few pub(crate) fields that could tighten to pub(super). Low priority.
- The 4 pre-existing >400 files (main.rs 632, config_panel/render.rs 590,
  tab_state.rs 404, menu_workspace/loader.rs 404) -- decide whether to split.
- ff-mdx-app / ff-mdx-installer declare [profile.*] in their own Cargo.toml,
  which Cargo ignores for non-root packages (the 10 benign warnings in every
  build/gate log). Move/remove those profile sections to silence them. Orphan-
  crate housekeeping, out of ff-desktop scope.
- A bare `cargo nextest run --workspace` can hit a proptest
  `FileFailurePersistence::SourceParallel ... failed to find lib.rs or main.rs`
  abort for some crates (seen in ff-logging) and a log-file lock if run
  concurrently with verify.ps1. Use verify.ps1 (which sets up correctly and
  passed 9495/9495) as the canonical gate, not a bare nextest invocation.

## TOOLING NOTE
- `tools/python/check_line_limits.py` added this effort: reports ff-desktop
  non-test .rs files over 400 non-test lines. Run it to verify the 400-line rule.

## STANDING RULES FOR THIS EFFORT

- Scoped verification only (`cargo fmt`, `cargo check/clippy/test -p ff-desktop`);
  NEVER `--workspace` or verify.ps1 -- that is the owner's manual gate.
- Known pre-existing flake: B048 shared-env-var race (FFWB_HISTORY_PATH /
  FFWB_USER_CONFIG_PATH) fails only under multithreaded `cargo test`, passes under
  nextest. NOT caused by this effort; do not try to "fix" it.
- Behaviour-preserving refactors need no gate; anything that changes observable
  behaviour or retires a requirement DOES (task 7, and any revisit of 1.2/1.3).
- Keep the notification channel and ShellServices/ShellRequest surfaces
  (spec-mandated). ASCII-only .rs; 400-line rule on non-test source.

---

## B080 dispatch-unify migration -- progress + design-review notes

Steps 0-2 DONE and owner-reviewed (the dispatch design was eyeballed and approved).

- Step 0/1: shell/dispatch.rs front door `dispatch_command_string` (pure
  indirection + the single Req 9.7 verb/arg split + `function_target_with_arg`
  helper). Both outermost submit sites rerouted.
- Step 2: `handle_command` refactored into shared `run_command_prelude` +
  `run_command_ladder`; the front door runs prelude -> resolve_target -> ladder.
  `builtin_workspace_target_for` (command_config/mod.rs) is the verb-table
  classifier for the CustomWorkspace/nav family (FILES/=FILES/FILE CATALOGS/
  CONFIG/COMMANDS/LOG/PLUGINS/MACROS/GSEARCH/SEARCH/KEYS/KINDS/MENUS/THEME-bare);
  the dispatch_command_target CustomWorkspace arm navigates via the exact ladder
  method (nav_to_kind etc.). A1 ordering implemented: EXIT family before POM
  fastpath (pinned by `exit_family_from_non_menu_context_dispatches_file_exit_not_pom_return`).
  940 tests pass; typed_and_key_paths_reach_same_handler_for_builtin_verb green.
  Each run WEDGED on the usual infra/token dropout at the review step; Kiro
  aborted and verified in-line (abort-and-verify is the established recovery).

TWO TRACKED DESIGN NOTES (not bugs, owner-acknowledged):
1. Migrated ladder arms are left as "superseded, unreachable fallback" and
   retired together in Step 7 (per-step rollback safety). Carrying transitional
   dead-but-present arms through Steps 3-6 is intentional.
2. `resolve_pom_option_key` is config-driven (resolves against the loaded menu's
   option list, NOT hardcoded POM logic) -- behaviour is already non-special-
   cased and correct. But the NAME is legacy "POM" framing. Rename to e.g.
   `resolve_menu_option_key` when CR-CH-052 lands (its "no POM-specific
   behaviour" intent), NOT in B080 (B080 is behaviour-neutral).

REMAINING B080 STEPS: 3 Function family (EXIT/EDIT/BROWSE/VIEW/CLOSE); 4 manager
families (nav/exclude/find/profile/scroll); 5 split/detach/swap+workspace; 6
standalone+scrm; 7 retire the now-empty ladder segments + delete superseded arms.
Then CR-CH-052 (X/=X/`=`-prefix semantics) lands on top of the unified front door.
