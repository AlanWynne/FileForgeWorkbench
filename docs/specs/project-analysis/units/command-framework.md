# Analysis Record: command-framework

- **Wave**: 0 (foundation -- central command dispatch, consumed by nearly all crates)
- **Backing crate**: `ff-command`
- **Spec folder**: `docs/specs/command-framework/`
- **Analysed**: Wave 0, task W0.7 (CR-NR-057 re-baseline)
- **Verdict**: INCOMPLETE on TWO fronts. Req 1-8 done and TCR-PASS; Req 9 (Command
  Arguments, CR-NR-054) unimplemented; Req 10 (Context Navigation Stack,
  CR-NR-057, NEW) unimplemented. NOT a split candidate.
- **CR-NR-057 impact**: DIRECT -- added Requirement 10 (Context Navigation Stack,
  13 criteria) + design + Task 25 + TCR Phase DH rows. This is the primary reason
  for the re-baseline.

---

## 1. Scope summary

`ff-command` is the single dispatch mechanism for all user operations. 10
requirements (was 9 before CR-NR-057):

- Req 1 registry; Req 2 dispatch (single `execute_command`); Req 3 metadata;
  Req 4 undo/redo integration; Req 5 keyboard shortcuts; Req 6 Lua scripting
  bridge; Req 7 command history; Req 8 unified Command_Target (5 variants,
  CR-NR-051); Req 9 command arguments (`DOWN 8`, key-forwarding, CR-NR-054);
  **Req 10 Context Navigation Stack (`.` STOP vs `;` PUSH, `navigation.
  stack_max_depth` default 32, CR-NR-057 -- NEW)**.

276 req lines, 10 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 276 lines; 10 reqs | No |
| 3+ distinct responsibilities | registry/dispatch/metadata/undo/shortcuts/scripting/history/target/args/nav-stack are facets of ONE command subsystem | No |
| 2+ crates | one crate `ff-command`; Req 5.5/8.5/9.8/10 UI wiring lands in `ff-desktop` (consumption, not a second owner) | No |
| low-cohesion clusters | all requirements orbit the command lifecycle; high cohesion | No |
| file-size pressure | largest non-test `shortcut/chord.rs` 354 -- under the 400 cap | No |

0 criteria. **NOT a split candidate.** Well-modularised (shortcut/ subdir).

## 3. Consistency / conflict

- Public types owned here (CommandId, CommandParams, CommandResult,
  CommandRegistry, CommandMetadata, ExecutionContext, ShortcutBinding, Chord,
  CommandTarget) -- sole owner `ff-command`. Added to consistency-matrix.
- **Req 10 (Context Navigation Stack) is a high-fan-out contract** paired with
  command-semantics Req 11 (Command_Chain parsing, CR-NR-057 -- `.`/`;` split,
  `max_chain_length`, W3.1): command-semantics PARSES the chain; command-framework
  Req 10 manages the per-Workbench return stack the chain drives. Req 10.10 reuses
  Command_Target's `produces_visible_workspace` (Req 8.9) as the "opens/changes a
  Context" predicate -- VERIFIED present in command_target.rs. Consumers of the
  stack: startup-and-session (Contexts, session-state boundary), menu-workspace
  (MENU chaining), function-keys-and-history (END/F3/RETURN). WATCH: verify these
  consume Req 10 consistently when analysed (W3). No conflict now.
- **Command_Target (Req 8)** remains the fan-out contract (menu-workspace,
  command-configurator, startup-and-session, shell-command, lua-macro-engine) --
  carry PA-WATCH-001. Still done + TCR-PASS.
- Req 5.3 reserved-shortcut list overlaps layout-and-docking/view-zoom/edit/
  clipboard; ff-command is the declared owner (cross-cutting Req 10-of-project).
  No conflict.
- `commands.*` and NEW `navigation.stack_max_depth` config keys are consistent
  with the configuration-system reserved namespaces (W0.3). `navigation.
  stack_max_depth` is DEFINED (Req 10.11) but NOT yet registered in code (grep 0)
  -- part of the unimplemented Req 10.

## 4. Completeness -- INCOMPLETE (two unimplemented requirements)

- Tasks: 143 `[x]`, 14 `[ ]`.
  - 6 open = Phase DF (Req 9 Command Arguments, CR-NR-054): DF.1-DF.6.
  - 8 open = Task 25 (Req 10 Context Navigation Stack, CR-NR-057): 25.1-25.7.
- **Req 9 UNIMPLEMENTED** (unchanged from prior pass): no `parse_invocation` /
  `CommandInvocation` / reserved `arg` param in code (grep 0). TCR Phase DF rows
  red. The `DOWN 8` / `MENU SETTINGS EDITOR` / `RETRIEVE LIST` key-forwarding
  feature.
- **Req 10 UNIMPLEMENTED** (NEW): no `ContextNavigationStack` / `navigation_stack`
  / `stack_max_depth` in code (grep 0); TCR Phase DH Req 10.1-10.13 all NOT
  COVERED (red). Requirements gate IS complete (Req 10 authored; design + Task 25
  + TCR rows present) -- ready to build.
- Req 1-8 complete: registry, dispatch, metadata, undo bridge, shortcuts,
  scripting, history, Command_Target -- task groups `[x]`, TCR PASS (Req 8.1-8.9
  PASS incl. produces_visible_workspace).
- Completeness verdict: **INCOMPLETE** -- two real outstanding code blocks (Req 9,
  Req 10), each with a completed gate. Logged PA-INCOMPLETE-001 (Req 9) and
  PA-INCOMPLETE-003 (Req 10).

## 5. Logging audit

- 7 log-macro call sites, all `log_warn!`, at correct paths: dispatch WARNs on
  disabled command (Req 2.5) and handler error (Req 2.6); history WARNs on depth
  clamp (Req 7.3), history-file parse/read failure (Req 7.6). Levels match spec.
- Req 10.11 mandates a WARN on Context_Navigation_Stack overflow (drop oldest) --
  NOT yet present because Req 10 is unimplemented (folds into PA-INCOMPLETE-003).
- No `println!`/`eprintln!`. GUI-independence upheld.
- Not among the 56 zero-log crates (PA-LOG-002). Adequate for implemented paths.
- Logging verdict: **adequate for Req 1-8; Req 10.11 WARN pending its impl.**

## 6. Findings logged

- **PA-INCOMPLETE-001** (INCOMPLETE, Req 9 -- code gap, gate complete): Command
  Arguments (CR-NR-054) unimplemented. 6 Phase DF tasks open; TCR Phase DF red; no
  `parse_invocation`/`arg`-param code. Owner: schedule Phase DF (TDD). Ready to
  build.
- **PA-INCOMPLETE-003** (INCOMPLETE, Req 10 -- NEW code gap, gate complete):
  Context Navigation Stack (CR-NR-057) unimplemented. Task 25 (25.1-25.7) open; no
  `ContextNavigationStack`/`stack_max_depth` code; TCR Phase DH Req 10.x red.
  Includes the Req 10.11 overflow WARN. Paired with command-semantics Req 11
  (Command_Chain, W3.1). Owner: schedule Phase DH impl (TDD). Ready to build.
- **PA-WATCH-001** (CONSISTENCY WATCH, carried): Command_Target (Req 8) consumed by
  menu-workspace, command-configurator, startup-and-session, shell-command,
  lua-macro-engine. NEW: Req 10 Context Navigation Stack consumed by
  startup-and-session, menu-workspace, function-keys-and-history + paired with
  command-semantics Req 11. Verify consistency when each is analysed (Waves 3-5).
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-command contributes non-ASCII
  incl. em-dashes in two WARN log-message literals (history.rs). Rolled into
  project-wide PA-LOG-001.

---

## 7. ADDENDUM -- CR-NR-058 (added after W0.7 analysis)

CR-NR-058 (Phase DI) added **Requirement 11: Uniform Command Execution
Instrumentation** to this spec AFTER section 1-6 above were written. Updated facts:

- Requirement count is now **11** (not 10). Req 11 instruments the single
  `execute_command` dispatch boundary so EVERY command invocation logs its start
  (id + redacted/bounded params) and completion (success/failure + result summary +
  duration) at a dev-only level (DEBUG); failures additionally log at WARN/ERROR.
  Sensitive params are redacted. The instrumentation is UNIFORM across all sources
  (keyboard, menu, command line, macro, plugin) and leverages the `dev-logging`
  compile-time gate from logging-subsystem Req 13 (CR-NR-058).
- **Req 11 is UNIMPLEMENTED**: no instrumentation wrapper in dispatch.rs, no
  redact_and_bound function, no Phase DI start/completion log records in code.
  Phase DI Task 26 (26.1-26.x) open; TCR Phase DI rows present but NOT COVERED.
  Requirements gate IS complete -- ready to build.
- This is a THIRD incomplete-work item on command-framework (alongside Req 9
  PA-INCOMPLETE-001 and Req 10 PA-INCOMPLETE-003). Logged as PA-INCOMPLETE-004.
- Split candidacy unchanged: 11 reqs, 276+ lines still not a split candidate (the
  instrumentation is one wrapper at the dispatch boundary).
- **Project-analysis logging audit significance**: once PA-INCOMPLETE-004 (Req 11)
  is implemented, EVERY command invocation across the entire project will log
  its start + params + outcome at DEBUG level in dev builds, providing the
  "logging to assist testing and debugging" the user requested. This is the
  single highest-leverage logging improvement for bug reporting, and it composes
  with the PA-LOG-002 inventory (many zero-log crates invoke commands but don't
  log themselves -- the dispatch boundary will cover them automatically).
  Recorded as PA-CR058 in the register.
