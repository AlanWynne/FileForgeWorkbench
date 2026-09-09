# Analysis Record: command-framework

- **Wave**: 0 (foundation -- central command dispatch, consumed by nearly all crates)
- **Backing crate**: `ff-command`
- **Spec folder**: `docs/specs/command-framework/`
- **Analysed**: Wave 0, task W0.7
- **Verdict**: INCOMPLETE. Req 1-8 done and TCR-PASS; Req 9 (Command Arguments,
  CR-NR-054) is NOT implemented -- 6 open tasks (Phase DF), all TCR rows red.
  NOT a split candidate.

---

## 1. Scope summary

`ff-command` is the single dispatch mechanism for all user operations. 9
requirements:

- Req 1 registry; Req 2 dispatch (single `execute_command` entry point);
  Req 3 metadata (enabled/visibility predicates); Req 4 undo/redo integration;
  Req 5 keyboard shortcuts (chords, sequences, reserved list, conflict
  detection); Req 6 Lua scripting bridge; Req 7 command history (persistent,
  bounded, `commands.history_depth`); Req 8 unified Command_Target (5 variants,
  CR-NR-051); Req 9 command arguments (`DOWN 8`, key-forwarding, CR-NR-054).

247 req lines, 9 requirements, single backing crate. Cohesive.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 247 lines; 9 reqs | No |
| 3+ distinct responsibilities | registry/dispatch/metadata/undo/shortcuts/scripting/history are facets of ONE command subsystem, not separable products | No (borderline) |
| 2+ crates | one crate `ff-command`; Req 8.5/8.6/9.8 UI wiring lands in `ff-desktop` but that is consumption, not a second owning crate | No |
| low-cohesion concern clusters | all requirements orbit the command lifecycle; high cohesion | No |
| file-size pressure | largest non-test file is `shortcut/chord.rs` (354) -- under the 400 cap | No |

0-1 criteria met. **NOT a split candidate.** The shortcut subsystem
(`src/shortcut/`) is already cleanly modularised within the crate.

## 3. Consistency / conflict

- Public types owned here (CommandId, CommandParams, CommandResult,
  CommandRegistry, CommandMetadata, ExecutionContext, UndoRecord bridge,
  ShortcutBinding, Chord, CommandTarget, WorkspaceKind refs) -- sole owner
  `ff-command`. Added to consistency-matrix.md.
- **Command_Target (Req 8)** is a high-fan-out contract: referenced by
  menu-workspace (Req 10/11 Menu_Option.command), command-configurator (Req 1
  user definitions), startup-and-session (Req 21 persisted Workspace
  descriptors), shell-command (Req 19 External_Target), lua-macro-engine
  (Req 5 Macro_Target). WATCH item -- when those units are analysed, verify each
  resolves `command` values through Target_Resolution consistently. No conflict
  found now; ff-command owns the type, others consume it (correct direction).
- **Command Arguments (Req 9)** reserved param key `arg` and the
  verb/argument split at the dispatch boundary must be honoured identically by
  every input source (command line, menu option, key binding, macro). Because
  Req 9 is unimplemented, this contract is DEFINED but not yet enforced -- watch
  for divergent ad-hoc parsing in ff-desktop when Req 9 is built.
- Req 5.3 reserved-shortcut list overlaps layout-and-docking (Ctrl+Shift+D/T),
  view-zoom (Ctrl+/-/0), edit (undo/redo/clipboard). Consistent; ff-command is
  the declared owner of the reserved set (cross-cutting Req 10). No conflict.

## 4. Completeness

- Tasks: 143 `[x]`, 6 `[ ]`. ALL 6 open are Phase DF (Req 9, CR-NR-054):
  DF.1 parse_invocation, DF.2 arg-param wiring, DF.3 backward-compat,
  DF.4 fixed argument on definition/binding, DF.5 (ff-desktop) key-forwarding,
  DF.6 unit tests.
- Verified NOT implemented: no `parse_invocation` / `CommandInvocation` / `arg`
  reserved key in `ff-command/src`. TCR Phase DF rows Req 9.1-9.8, 9.10 all
  NOT COVERED (red).
- Req 8 (Command_Target) IS complete: `command_target.rs` present; TCR rows
  8.1-8.9 all PASS with `command_target_tests.rs` evidence; ff-desktop wiring
  (8.5/8.6) PASS.
- Req 1-7 complete: registry, dispatch, metadata, undo bridge, shortcut
  registry (chord/sequence/conflict/reserved modules), scripting bridge, history
  -- all task groups `[x]`, TCR PASS (per earlier phases).
- Completeness verdict: **INCOMPLETE** -- Req 9 is real outstanding code work,
  not a tracking artefact. Logged PA-INCOMPLETE-001.

## 5. Logging audit

- 7 log-macro call sites, all `log_warn!`, at the correct requirement paths:
  dispatch WARNs on disabled-command (Req 2.5) and on handler error (Req 2.6);
  history WARNs on depth clamp (Req 7.3), history-file parse failure and read
  failure (Req 7.6). Levels match the spec.
- No `println!`/`eprintln!` in the crate. GUI-independence upheld.
- Note: Req 9.5 in Req 9 mentions no logging; Req 2.6 WARN is the main audit
  path and is present. Command history (Req 7.1) records every successful
  invocation -- this is the user-action audit trail, distinct from ff-logging.
- Cross-check vs PA-LOG-002 inventory: ff-command is NOT among the 56 zero-log
  crates (it has call sites). Adequate for its failure paths.
- Logging verdict: **adequate**.

## 6. Findings logged

- **PA-INCOMPLETE-001** (incomplete work -- real code gap): Req 9 Command
  Arguments (CR-NR-054) is unimplemented. 6 Phase DF tasks open; TCR Phase DF
  rows red; no `parse_invocation`/`arg`-param code exists. This is the
  user-designed `DOWN 8` / `MENU SETTINGS EDITOR` / `RETRIEVE LIST`
  key-forwarding feature. Owner action: schedule Phase DF implementation
  (TDD per testing.md); it already has a completed requirements gate (Req 9
  authored), so it is ready to build. Belongs to the task-revision step.
- **PA-WATCH-001** (consistency watch): Command_Target (Req 8) is a high-fan-out
  contract consumed by menu-workspace, command-configurator,
  startup-and-session, shell-command, lua-macro-engine. Verify Target_Resolution
  consistency when each of those units is analysed (Waves 1-5). No conflict now.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-command contributes 72
  non-ASCII matches, including em-dashes inside two WARN log-message string
  literals (history.rs:120, 127) plus doc-comment/separator prose. The
  in-string em-dashes will render in the log file; recommend ASCII `--` when the
  project-wide cleanup runs. Rolled into PA-LOG-001; not fixed here.
