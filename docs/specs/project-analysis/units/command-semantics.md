# Analysis Record: command-semantics (W3.1)

- **Wave**: 3 (Shell, commands, menus, session)
- **Backing crate**: `ff-command-semantics`
- **Spec files**: requirements.md (350 lines, 11 requirements -- numbered 1-11 but
  listed 1-6, 8, 7, 9-11: Req 8 precedes Req 7 in file order), tasks.md
  (181 sub-tasks, **173 done / 8 OPEN**), design.md present
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy

NOT a split candidate.

- Requirement volume: 11 reqs, 350 lines. At the borderline (11 reqs, ~350 lines).
  (1 weak signal at most)
- Responsibilities: ONE cohesive domain -- the primary/line command execution
  pipeline (parse -> normalize -> resolve scope -> validate -> execute -> report)
  + HELP + TSO commands + command chaining. All facets of the command engine.
- Crates: single crate; GUI-independent (Req: no rendering dep).
- Cohesion: high.

Below a confident 2-of-4. No split.

### Source file size violation (PA-STD-032)

`tso.rs` = 447 non-test lines, over the 400 cap (Req 9/10 TSO + FTSO commands:
OUTPUT/CANCEL/SEND/PROFILE/PRINTDS etc.). Split by command group. REFACTOR, no
gate. Recorded PA-STD-032. (Only file over cap; the rest are well-sized.)

---

## 2. Completeness -- PA-INCOMPLETE-007 (CR-NR-057 command chaining UNBUILT)

FIRST Wave-3 unit with incomplete tracking: **173/181 tasks done, 8 OPEN**. The 8
open tasks are all Task 28 (`chain.rs`), which implements Requirement 11 (Command
Chain Parsing and Sequential Execution) -- a CR-NR-057 addition:

- 28.1 `split_chain(line) -> Vec<ChainSegment>` (`.`/`;` split, quote/hex-aware) -- Req 11.1/11.2/11.6/11.7/11.8
- 28.2 failing unit tests for split_chain
- 28.3 `commands.max_chain_length` config (default 16) + over-long rejection
- 28.4 `CommandEngine::execute_chain` (left-to-right, fail-stop, forward separator
  to nav stack) -- Req 11.3/11.4/11.5
- 28.5 failing tests (order, fail-stop, `.`==`;` for non-navigating, per-segment
  transaction, `END ; EDIT`)
- 28.6 menu-workspace fastpath Chained_Path resolver calls the SAME `split_chain`
  helper (shared-helper wiring point)
- 28.7 TCR: set the CR-NR-057 Req 11 rows to correct status

Req 11 (8 acceptance criteria) is entirely UNBUILT -- gated (spec written) but not
implemented, tasks correctly left `[ ]` (NOT a false-positive -- honest tracking,
unlike PA-INCOMPLETE-002/006). This is the command-semantics HALF of the CR-NR-057
command-chaining bundle; the command-framework HALF is PA-INCOMPLETE-003
(Context Navigation Stack, Req 10, W0.7 = Phase DH). They are INTERDEPENDENT:
Req 11.5 says the Chain_Separator determines only Context-Navigation-Stack behaviour
(command-framework Req 10), so command chaining needs the nav stack. Recorded
PA-INCOMPLETE-007 (HIGH): implement Task 28 (`chain.rs` + execute_chain +
max_chain_length + tests + TCR), coordinated with PA-INCOMPLETE-003 (nav stack) and
the menu-workspace fastpath (28.6, W3.5). This pairs into the CR-NR-057 command-
chaining phase (existing Phase DH bundle).

Everything else (Req 1-10: pipeline, scope resolution, primary/line parsers, error
handling, config, HELP, TSO P1/P2) is implemented and tracked complete.

---

## 3. Cross-unit consistency

### PA-WATCH-009 (HILITE) -- command-semantics is NOT in the chain

W1.6/W1.10 flagged a possible "command-semantics collection step / compatibility
matrix / HILITE" involvement. Verified: this crate has ZERO references to HILITE or
CompatibilityMatrix (grep 0). The compatibility matrix is owned by line-commands
(W1.6), and HILITE is edit-operations(parse) -> syntax-highlighting(execute) ->
find (W1.10, PA-WATCH-009). command-semantics is NOT part of the HILITE delegation
chain -- it only provides generic primary/line command PARSING + scope resolution
that those subsystems consume. This NARROWS PA-WATCH-009 (command-semantics
excluded); the HILITE seam is confirmed to Wave 4/5 shell wiring, not here.

### Command-chaining shared-helper seam (Task 28.6) -- consistency point

Req 11 + Task 28.6 establish that `split_chain` must be a SHARED pure helper used
by BOTH command-semantics (command-line chains) AND menu-workspace (fastpath
Chained_Path resolver). This is a designed single-owner-many-callers seam (avoids
a duplicate chain parser). Recorded as a WATCH to verify at menu-workspace (W3.5):
the fastpath must call ff-command-semantics::split_chain, not roll its own.
PA-WATCH-017.

### Public types and ownership

- Command_Engine, Command_Token/parser, Execution_Plan, Scope resolution,
  Session_State, Status_Message, TSO command impls, (pending) Command_Chain/
  ChainSegment/ChainSeparator -- sole-owned by ff-command-semantics. No duplication.
- Routes ALL commands through command-framework `CommandRegistry`/`Command_Dispatch`
  (Req design principle 2) -- correct direction; consistent with W0.7.
- Scope resolution with VISIBLE/EXCLUDED/ALL + TAGGED modifiers -- consumed by
  exclude-show-filter (W1.11) and find-and-replace (W1.8). command-semantics OWNS
  the parsing/scope; the feature crates own the execution. Consistent
  single-owner-many-readers.
- Provides parsing/scope for LOCATE/SORT/COLS/BOUNDS (navigation-commands, W1.7),
  FIND/CHANGE (find-and-replace), line commands (line-commands). Consistent.

### Cross-reference integrity

All 7 declared cross-refs resolve. No dangling refs.

---

## 4. Logging audit

Scan of `crates/ff-command-semantics/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0

This is the COMMAND EXECUTION PIPELINE -- the single most important CR-NR-058
dev-logging target after command-framework Req 11 (PA-INCOMPLETE-004). Every
command flows through here (parse -> scope -> execute -> status). Currently 0
logging. The uniform per-command instrumentation (start/params/result at DEBUG,
failures at WARN) belongs at the command-framework `execute_command` boundary
(PA-W0.2), which this engine dispatches through -- so much is covered automatically
once PA-W0.2 lands. Recorded PA-LOG-018 (MEDIUM): after PA-W0.2, add
command-semantics-specific dev-logging (scope resolution decisions, chain
execution step-by-step for Req 11, error-path WARN) under the `dev-logging` gate.
Tightly coupled to the CR-NR-058 command-instrumentation work.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-007 (HIGH)**: implement Task 28 (Req 11 command chaining --
  `chain.rs` split_chain + execute_chain + max_chain_length + tests + TCR).
  Coordinate with PA-INCOMPLETE-003 (Context Navigation Stack, command-framework
  Req 10 = Phase DH) since Req 11.5 depends on the nav stack, and with the
  menu-workspace fastpath (28.6, W3.5). Fold into the CR-NR-057 Phase DH bundle.
- **PA-STD-032 (REFACTOR)**: split `tso.rs` (447 non-test) by TSO command group.
  No gate.
- **PA-LOG-018 (MEDIUM)**: command-pipeline dev-logging after PA-W0.2 (the
  execute_command instrumentation covers the bulk automatically); add scope/chain/
  error-path trace under the dev-logging gate.
- **PA-WATCH-017**: verify menu-workspace fastpath calls the shared
  `split_chain` helper (Task 28.6), not a duplicate parser. Confirm at W3.5.

No requirement CHANGE proposed; the spec is internally consistent. Req 11 is a
correctly-gated NEW requirement pending implementation (honest `[ ]` tracking).
Minor: file lists Req 8 before Req 7 (ordering oddity, not a defect).

---

## Summary

command-semantics (`ff-command-semantics`) is the GUI-independent primary/line
command execution pipeline -- cohesive, well-tracked for Req 1-10, and with GOOD
TCR (39 rows). The headline finding is PA-INCOMPLETE-007 (HIGH): Requirement 11
(CR-NR-057 command chaining, Task 28, 8 open tasks) is entirely UNBUILT -- honestly
tracked as `[ ]` (not a false-positive). It is the command-semantics half of the
CR-NR-057 chaining bundle and interdepends with command-framework's Context
Navigation Stack (PA-INCOMPLETE-003) and the menu-workspace fastpath (PA-WATCH-017,
W3.5). PA-WATCH-009 (HILITE) is NARROWED -- command-semantics is confirmed NOT in
the HILITE chain (0 refs). Minor items: tso.rs over the 400 cap (PA-STD-032), and
the command-pipeline dev-logging (PA-LOG-018, largely covered by the CR-NR-058
execute_command instrumentation PA-W0.2). No spec split.
