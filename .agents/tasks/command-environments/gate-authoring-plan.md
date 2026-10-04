# Implementation Plan -- CR-CH-053 Command Environments Requirements Gate (DOCS-ONLY)

Mode: planning from an approved design proposal. This is a DOCS-ONLY gate-authoring
task. NO `.rs` source may change; NO `cargo` command runs. The deliverables are the
six documents listed in the brief, authored to the project's spec conventions.

## Design decisions already fixed (do NOT re-open)

All owner decisions are baked in by the authoritative inputs and must be honoured
verbatim; the plan below only sequences the writing:

- NAMING: FF* convention. FFCMD (= existing `resolve_target` / `ShellTargetResolver`,
  the always-present base, NO rewrite), FFEDIT (editor command-line verbs migrated
  off the ladder), FFLINE (prefix-area line commands, SEPARATE sibling of FFEDIT,
  NAMED/MODELED only in phase 1 -- no code change, intake stays prefix-area ->
  Command_Engine), and FFBROWSE/FFAMS/FFJES/FFJOB/FFSQL/FFCICS/FFVFS/FFADMIN/
  FFDEBUG/FFMON/FFLIB (vision-only, future, each registers with its consuming
  subsystem).
- ALIAS MAP: mainframe names address FF* envs: TSO->FFCMD, ISREDIT->FFEDIT (future
  IDCAMS->FFAMS, SDSF->FFJES, etc.). Reconciles lua-macro-engine Req 11.11-11.14.
- SCOPE (CRITICAL): BUILD only the environment FRAMEWORK + FFCMD + FFEDIT. FFLINE
  named/modeled only. All other FF* envs vision-only. NO criterion may require
  building FFBROWSE/FFSQL/etc.
- Q1: DEFER the interactive command-line address prefix. Phase 1 addressing is
  MACROS ONLY (lua-macro-engine ADDRESS). Keep the `parse_address_prefix` router
  seam documented as a future additive extension; write NO acceptance criterion
  requiring an interactive prefix; include an explicit NON-requirement that it is
  deferred.
- Q2: FFLINE separate sibling of FFEDIT, active together when an editor context is
  focused; FFLINE intake stays prefix-area -> Command_Engine, NOT the front door.
- Q3: active environment follows the FOCUSED context (active tab kind; for
  split/detached, the focused region/window's context).
- Q4: active-wins shadowing (verb in both active env and FFCMD -> active env claims
  it; FFCMD reachable via explicit address), mirroring command-framework Req 8.10.
- Q5: FFEDIT DELEGATES to existing managers (nav_manager / find_manager /
  exclude_manager / edit_profile / scroll_amount) in phase 1; does NOT front the
  Command_Engine.
- Q7: environment dispatch outcome carries a return code mapped to macro RC
  (lua-macro-engine Req 11.15).

Framework reconciliation that MUST be explicit in requirements.md and design.md:
FFCMD IS `resolve_target` (no rewrite / second dispatcher); the router inserts ONE
active-environment step into the single B080 front door (`dispatch_command_string`)
BEFORE `resolve_target`; the prelude (stage-1 current-menu Option_Key, EXIT family,
POM/chained fastpaths) stays FIRST and unshadowable; NO second navigation stack;
`CommandTarget` unchanged; does NOT implement CR-CH-052; does NOT design the
database tool or any future environment.

## Authoritative inputs (already read during planning; re-read before writing)

- `.agents/tasks/command-environments/design-proposal.md` -- full design, draft
  EARS E.1-E.12, migration E0-E7, open-question resolutions.
- `docs/specs/command-environments/environments-vision.md` -- ALREADY WRITTEN; do
  NOT overwrite. requirements.md must REFERENCE it for the catalogue.
- `docs/status/change-log.md` CR-CH-053 entry -- owner decisions + buildable scope.
- `.kiro/steering/specs.md` (spec file + task format), `workflow.md` (gate order),
  `framework-conformance.md`, `documentation.md` (plain ASCII only).
- Grounding/citation: `docs/specs/command-framework/requirements.md` (Req 2, 5, 6,
  8, 8.10, 9, 10) + `design.md` (B080 Unified Command Dispatch section);
  `docs/specs/command-semantics/requirements.md` (Command_Engine, line-command
  scope); `docs/specs/lua-macro-engine/requirements.md` Req 11.11-11.15.
- Code grounding (read-only): `crates/ff-desktop/src/shell/dispatch.rs`
  (`dispatch_command_string`), `shell/commands.rs` (`run_command_prelude` /
  `run_command_ladder`), `shell/commands_ladder_b2.rs` (`try_commands_b2` editor
  verbs), `command_config/mod.rs` (`ShellTargetResolver` /
  `builtin_workspace_target_for`), `crates/ff-command/src/command_target.rs`
  (`CommandTarget`, `resolve_target`), `tab_state.rs` (`TabKind`).

---

- [ ] 1. Write `docs/specs/command-environments/requirements.md` -- the full EARS requirements document.
      Structure per other requirements.md files (command-framework is the model):
      `# Requirements Document`, `## Introduction`, `## Glossary`, `## Requirements`
      with `### Requirement N: Title` -> `#### Acceptance Criteria` numbered `N.M`.
      Introduction MUST: state this is the authoritative home for the Command
      Environment model; cross-reference `environments-vision.md` for the FF*
      catalogue (do not restate the whole table); state the phase-1 buildable scope
      explicitly (framework + FFCMD + FFEDIT; FFLINE named-only; all other FF*
      vision-only); state the framework-conformance stance (builds ON B080 front
      door; FFCMD == resolve_target; no second dispatcher; no second nav stack;
      CommandTarget unchanged; does NOT implement CR-CH-052; does NOT design any
      future environment). Glossary MUST define: Command_Environment,
      Environment_Registry, Active_Environment, Base_Environment (FFCMD), FFEDIT,
      FFLINE, Address, Alias_Map (with a source tag citing the proposal / vision /
      command-framework where each term originates).
      Requirements MUST be derived from proposal E.1-E.12 but RENUMBERED and REFINED
      into coherent EARS `WHEN ... THE ... SHALL ...` criteria, phase-1-scoped. Cover
      at minimum: (a) Command_Environment definition + FFCMD-as-base (E.1);
      (b) Environment_Registry + active-env derivation from focused Context (E.2 +
      Q3 -- include split/detached: follows the focused region/window context);
      (c) front-door router insertion order: prelude FIRST and unshadowable -> active
      env -> FFCMD (resolve_target) -> ladder fallback, no second dispatcher (E.3);
      (d) backward-compatibility: identical observable result for every command that
      resolves today (E.4, mirror command-framework Req 8.4); (e) active-wins
      shadowing (E.5 + Q4, mirror Req 8.10); (f) FFEDIT owns the enumerated editor
      verbs (LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/SORT/EXCLUDE/X/SHOW/INCLUDE/RESET/
      FIND/RFIND/CHANGE/RCHANGE/CAPS/NULLS/STATS/LOCK/PROFILE/HILITE/SCROLL) and
      executes them via the existing managers (Q5 delegation) with IDENTICAL
      observable results incl. the B062 case-preservation rule for Argument_Strings
      (E.6); (g) active env = FFEDIT when active Context is editor (TabKind
      FileEditor or Untitled), else FFCMD (E.7); (h) macro ADDRESS routes to the
      named FF* environment through the Scripting_Bridge, reconciled via the alias
      map TSO=FFCMD / ISREDIT=FFEDIT (lua-macro-engine Req 11.11-11.14) (E.9);
      (i) addressed-env == active-env is redundant/no-op (E.10); (j) dispatch outcome
      carries a return code mapped to macro RC (Q7, lua-macro-engine Req 11.15);
      (k) FFLINE is NAMED as a sibling environment of FFEDIT, active together with
      FFEDIT when an editor is focused, intake UNCHANGED (prefix area ->
      Command_Engine, NOT the command-line front door), with NO line-command code
      change in phase 1; (l) future-context template: a new Context MAY register its
      own environment by mapping its kind in the registry with NO front-door change
      (E.12); (m) an explicit NON-REQUIREMENT clause: the interactive command-line
      address prefix is DEFERRED (phase 1 addressing is macros-only); the
      `parse_address_prefix` seam is reserved for a later additive change and NO
      criterion requires it now. Every criterion must build ON the existing framework
      and be phase-1-scoped (no criterion requiring FFBROWSE/FFSQL/etc.). Record the
      final requirement numbers and the full criterion count for the TCR step.
      Files: docs/specs/command-environments/requirements.md
      Verify: open the file and confirm it parses as the standard requirements.md
      shape (Introduction + Glossary + numbered Requirements with `#### Acceptance
      Criteria` and `N.M`-numbered EARS lines); confirm NO criterion requires any
      vision-only environment and the deferred-prefix non-requirement is present;
      run the ASCII check in step 8 over this file.

- [ ] 2. Write `docs/specs/command-environments/design.md` -- the architecture document.
      Structure per other design.md files: `# Design Document`, `## Overview`, then
      sections. MUST cover: (a) the `CommandEnvironment` trait reusing the
      `TargetResolver` shape (name() + dispatch()/claim contract returning an outcome
      with claimed-vs-fell-through + a return code for RC); (b) `EnvironmentRegistry`
      (FFCMD implicit base = resolve_target; FFEDIT phase-1; future envs keyed by
      name); (c) active-env derivation from `TabKind` / focused Context (table: editor
      kinds -> FFEDIT, all others -> FFCMD; split/detached follows focused
      region/window per Q3); (d) the front-door router insertion into
      `dispatch_command_string`: prelude (unchanged, FIRST, unshadowable) -> [deferred
      address prefix seam, documented not implemented] -> active env -> resolve_target
      (FFCMD) -> `run_command_ladder` fallback; show the exact before/after ordering;
      (e) FFCMD == resolve_target reconciliation (no rewrite, no second classifier);
      (f) FFEDIT as a resolver+executor delegating to managers (nav/find/exclude/
      edit_profile/scroll), recommend option (b) methods-on-shell-behind-marker first
      per the proposal, with the sub-struct form as the eventual clean form;
      (g) FFLINE named sibling with intake unchanged (prefix area -> Command_Engine);
      (h) macro ADDRESS/alias/RC mapping through the Scripting_Bridge, reconciling
      lua-macro-engine Req 11.11-11.15; (i) the E0-E7 migration shape at design level
      (E0 scaffold as pure indirection; E1 nav; E2 exclude/show; E3 find incl. B062
      case + parse_two_args; E4 profile; E5 scroll; E6 FFLINE naming no-code; E7
      retire emptied ladder arms); (j) explicit NON-GOALS: no second nav stack, no
      parallel dispatcher, no CommandTarget change, does NOT implement CR-CH-052,
      does NOT design the database tool or any future environment, interactive prefix
      deferred. Cite the real code symbols (dispatch_command_string, resolve_target,
      ShellTargetResolver, try_commands_b2, TabKind) and the backing spec reqs.
      Files: docs/specs/command-environments/design.md
      Verify: open the file; confirm every requirement area from step 1 has a
      corresponding design element, the router ordering matches the requirements, and
      the non-goals section is present and matches the fixed decisions; run the ASCII
      check in step 8 over this file.

- [ ] 3. Write `docs/specs/command-environments/tasks.md` -- ordered implementation tasks.
      Follow specs.md Task File Format EXACTLY: only `[ ]` / `[x]` markers (NEVER
      pre-check anything -- all `[ ]`), every task line has descriptive title text
      after the number, status notes only on indented sub-bullets. Number sequentially
      from 1. Tasks must be independently completable, leave the build green, and each
      cross-reference the requirement criterion(a) it satisfies (by the final numbers
      assigned in step 1). Cover, in dependency order: (1) E0 scaffold -- the
      CommandEnvironment trait + EnvironmentRegistry + active-env derivation from
      TabKind + front-door insertion as PURE INDIRECTION (empty FFEDIT dispatch, no
      behaviour change), prove no shell test regresses; (2) E1 navigation family
      (LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/SORT) into FFEDIT delegating to
      nav_manager incl. up_by_amount/down_by_amount vs scroll_amount, delete arms from
      try_commands_b2; (3) E2 exclude/show family (EXCLUDE/X [ALL], SHOW/INCLUDE
      [ALL], RESET variants) via exclude_manager with same snapshot closure; (4) E3
      find family (FIND/RFIND/CHANGE/RCHANGE) via find_manager, carrying parse_two_args
      quoting + B062 case preservation verbatim; (5) E4 profile family (CAPS/NULLS/
      STATS/LOCK/PROFILE/HILITE) via edit_profile; (6) E5 scroll (SCROLL amt) via
      scroll_amount; (7) E6 FFLINE naming/model ONLY -- no code, document it as the
      sibling environment with unchanged prefix-area intake; (8) E7 retire the
      migrated/emptied try_commands_b2 arms and drop the call from run_command_ladder;
      (9) macro ADDRESS alias/RC wiring through the Scripting_Bridge reconciling
      lua-macro-engine Req 11.11-11.15. Each migration task must state the per-step
      discipline: scoped `cargo test -p ff-desktop` green + behaviour-preservation
      (identical observable result) + a full-shell/unit test per criterion written
      first (red before green), per testing.md. Record the final task count for the
      summary.
      Files: docs/specs/command-environments/tasks.md
      Verify: open the file; confirm every task uses `[ ]` only (none pre-checked),
      every line has a title, every task cites at least one requirement criterion from
      step 1, and the ordering is E0 -> E1..E5 -> E6 (FFLINE no-code) -> E7 -> macro
      wiring; run the ASCII check in step 8 over this file.

- [ ] 4. Add a Phase section for CR-CH-053 to `docs/project-management/project-master/tasks.md`.
      Append a new `### Phase <label> -- Command Environments (CR-CH-053)` section in
      the file's existing style (one line per logical deliverable, `[ ]` only, never
      pre-checked). One line per the logical deliverables from step 3's tasks.md (E0
      scaffold/framework, FFEDIT migration E1-E5, FFLINE naming E6, ladder retire E7,
      macro ADDRESS alias/RC wiring), each referencing the command-environments
      tasks.md. If the file has a Summary / counts block, update the counts to include
      the new deliverables; if there is no numeric summary block, leave counts
      untouched and note that in the summary.
      Files: docs/project-management/project-master/tasks.md
      Verify: open the file; confirm the new Phase section exists in-sequence and uses
      `[ ]` only; confirm any Summary count edited matches the number of lines added;
      run the ASCII check in step 8 over this file.

- [ ] 5. Add NOT COVERED (red) TCR rows to `docs/quality/TCR.md`.
      In the `### Binary -- ff-desktop` area add a clearly-labelled sub-section (an
      HTML comment marker + rows, matching the existing CR-NR-097 / CR-CH-051 marker
      style) titled for CR-CH-053 -- Command Environments. Add ONE row per NEW
      criterion from step 1's requirements.md, each using the red circle status, the
      `-- ` placeholder for test file, and a one-line description naming the
      criterion: `| `ff-desktop` | <red> | -- | command-environments Req X.Y:
      <one-line description> |`. The row count MUST equal the criterion count recorded
      in step 1.
      Files: docs/quality/TCR.md
      Verify: open the file; confirm one red row per requirements.md criterion under
      the CR-CH-053 sub-section, counts match step 1; run the ASCII check in step 8
      (note TCR status emoji are allowed per documentation.md).

- [ ] 6. Add `command-environments` to the sub-project list in `.kiro/steering/specs.md`.
      Insert `- command-environments` in the alphabetical sub-project bullet list
      (between `command-configurator`/`command-framework` region as alphabetical order
      dictates), optionally with a short parenthetical note (CR-CH-053: Command
      Environment model -- framework + FFCMD + FFEDIT built, FFLINE named-only, rest
      vision-only). Do not disturb surrounding entries.
      Files: .kiro/steering/specs.md
      Verify: grep the file for `command-environments` and confirm exactly one new
      list entry in correct alphabetical position; run the ASCII check in step 8.

- [ ] 7. Confirm change-log.md CR-CH-053 status reflects the gate authoring.
      The CR-CH-053 entry already exists (PENDING GATE). Per workflow.md step 11,
      ensure the change-log reflects the gate having been authored: update its Status
      line to IN PROGRESS (gate authored, awaiting owner approval) if the file's
      convention tracks that; do NOT mark DONE. If the existing entry already captures
      everything and no status field needs changing, make no edit and note it.
      Files: docs/status/change-log.md
      Verify: read the CR-CH-053 entry and confirm the status is not DONE and reflects
      the authored gate; run the ASCII check in step 8.

- [ ] 8. ASCII compliance sweep over every file written in steps 1-7.
      Per documentation.md, all docs files must be plain ASCII except allowed
      box-drawing (U+2500-U+257F) in Markdown and the TCR status emoji in TCR.md.
      Check each written/edited file for prohibited characters (em/en dash, curly
      quotes, ellipsis, math/logic symbols, arrows, BOM). Replace any offender with
      its ASCII substitute (`--`, `-`, `'`, `"`, `...`, `->`, `<=`, `>=`, `!=`, etc.).
      Files: all files touched in steps 1-7.
      Verify: run the documentation.md enforcement check over the touched docs paths
      (`rg "[^\x00-\x7F\u2500-\u257F]" <files>` for Markdown; the common-offenders
      pattern for em dash / curly quotes / math symbols) and confirm zero matches
      outside the allowed ranges.

- [ ] 9. Produce the gate summary (do NOT mark anything DONE).
      Write a short closing summary stating: the final requirement numbers and total
      criterion count created in requirements.md; the task count in tasks.md; the
      number of TCR rows added; confirmation that scope is framework + FFCMD + FFEDIT
      with FFLINE named-only and all other FF* environments vision-only; and that NO
      source changed, NO cargo ran, and the owner must review/approve before any code.
      Files: none (summary is the step's output message).
      Verify: the summary names concrete numbers (criteria, tasks, TCR rows) that
      match steps 1, 3, and 5, and explicitly states the scope and the no-source /
      awaiting-approval status.

## Notes / assumptions

- The step prompt's `{{report_path}}` placeholder was not substituted at launch, so
  this plan is written to the task artifact folder
  `.agents/tasks/command-environments/gate-authoring-plan.md` (outside the worktree
  spec tree it describes). The downstream coder step should treat this file as its
  ordered worklist.
- Final requirement/criterion numbers are intentionally not fixed here: step 1
  assigns them when refining E.1-E.12 into coherent EARS, and steps 3 and 5 consume
  whatever numbers step 1 produces. This keeps numbering authoritative in one place
  (requirements.md) per the gate.
- `environments-vision.md` is already authored and must NOT be overwritten; it is
  referenced from requirements.md, not restated.
