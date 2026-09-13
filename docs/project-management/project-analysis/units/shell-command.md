# Analysis Record: shell-command (W3.8)

- **Wave**: 3 (Shell, commands, menus, session)
- **Backing crate**: `ff-shell`
- **Spec files**: requirements.md (405 lines, 19 requirements), tasks.md
  (184 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 3 pass. SECURITY-CRITICAL (external process execution).

---

## 1. Split candidacy

Borderline; not a forced spec split, but two files exceed the cap.

- Requirement volume: 19 reqs, 405 lines. Above thresholds. (signal 1)
- Responsibilities: SHELL/TSO command with several facets -- external command
  execution, output capture, stdin piping, interactive terminal, and a terminal
  EMULATOR. The emulator (emulator.rs) is arguably a separable concern from the
  command-execution engine. (partial signal 2)
- Crates: single crate `ff-shell` (27 files -- large).
- Cohesion: high for exec/capture/pipe; the emulator is a distinct sub-concern.

Borderline. Recorded PA-SPLIT-010 (LOW, owner-gated): possible split of the
terminal EMULATOR (emulator.rs 535) from the command-execution engine; low priority.
The immediate issue is file size (below).

### Source file size violations (PA-STD-038)

Two files over the 400 cap: `engine.rs` 523 (the shell-mode gate + execution
orchestration), `emulator.rs` 535 (terminal emulator). Split engine.rs by concern
(mode gate / dispatch / outcome handling); the emulator split folds into
PA-SPLIT-010. REFACTOR, no gate. Recorded PA-STD-038.

---

## 2. Cross-unit consistency

### PA-WATCH-018 (shell.mode security gate) -- RESOLVED CLEAN, EXEMPLARY

The W3.4 watch (does `shell.mode` actually gate external execution, incl.
configurator-launched commands?) resolves strongly affirmative. The security model
is a first-class, ENFORCED control -- one of the best-designed in the analysis:

- Req 2 (Shell Mode Security Control) defines `shell.mode` with three modes:
  `disabled` (refuse ALL SHELL invocations), `prompt` (confirmation dialog before
  every exec -- the DEFAULT when unset, Req 2.5), `enabled` (no prompt).
- ENFORCED at the engine ENTRY: `engine.rs:62-67` -- "Enforces the shell.mode
  security control and the macro dual-gate"; matches on `mode`, returns
  `ShellError::ShellDisabled` for `Disabled`, prompts for `Prompt`. The gate is
  checked BEFORE any spawn (13 spawn sites, all downstream of the mode check).
- MACRO DUAL-GATE (Req 2.6/2.7): a SHELL invoked from a Lua macro requires BOTH the
  macro security mode AND shell.mode -- defense-in-depth. `error.rs:107`
  "macro: shell access denied". The two settings are independent (Req 2.6).
- SAFE DEFAULT: `prompt` when unset -- fails safe, not open.

command-configurator (W3.4) external execution DELEGATES to this engine, so its
external commands inherit the shell.mode gate (PA-WATCH-018 CONFIRMED: the gate
covers configurator-launched external commands because they route through
ff-shell's engine). No separate/weaker gate exists (matches command-configurator's
"no separate external-execution security switch"). CLEAN, single-owner security
control.

### Public types and ownership

- Shell_Engine, ShellMode, ExecutionMode/ExternalOutcome/TaskHandle,
  CommandExecutor (spawn), capture, stdin-pipe, terminal emulator -- sole-owned by
  `ff-shell`. No duplication.
- SHELL command (+ TSO alias) registered via command-framework. Output capture
  integrates with ff-workflow (progress) + ff-document-model (output-to-document) +
  ff-layout (Output_Panel). Deps: ff-command/ff-config/ff-layout/ff-workflow/
  ff-document-model/ff-logging -- all appropriate for a real exec engine. Clean.
- shell.mode config key -- sole owner; the security gate. Consumed by
  command-configurator (delegated exec). No collision.

### TSO alias vs command-semantics TSO commands -- WATCH

shell-command provides `TSO` as an alias for `SHELL` (ISPF compatibility). BUT
command-semantics (W3.1) also has Req 9/10 "TSO Commands and FTSO Operand Parsing"
(OUTPUT/CANCEL/SEND/PROFILE/PRINTDS, tso.rs 447). Two TSO-related surfaces: ff-shell
`TSO` = run an OS command (SHELL alias); ff-command-semantics TSO = emulated
mainframe TSO commands. Different meanings of "TSO" -- likely intentional (OS shell
vs emulated TSO) but worth confirming the `TSO` verb dispatch does not collide
(which TSO handler wins when the user types `TSO ...`). Recorded PA-WATCH-022 (LOW):
confirm the `TSO` command routing between ff-shell (SHELL alias) and
ff-command-semantics (emulated TSO P1/P2) is disambiguated.

### Cross-reference integrity

Cross-refs resolve. No dangling refs.

---

## 3. Completeness

Tracking: all 184 sub-tasks `[x]`. Implementation present across all 19 reqs
(SHELL/TSO command, shell.mode security, output capture, stdin pipe, interactive
terminal, emulator, macro dual-gate, workflow progress, output-to-document). Tests
in-file (capture/commands have rejection tests). No PA-INCOMPLETE. Complete.

### TCR (adequate)

25 TCR rows for 19 reqs -- adequate coverage (not raised as a gap; better than the
Wave-2/3 thin-TCR units). No PA-TCR raised.

---

## 4. Logging audit

Scan of `crates/ff-shell/src` (recursive):

- `ff_logging` / `log_*!`: 10 -- REAL logging (NOT a dead dep). For a process-
  spawning, security-gated subsystem this is important and correct.
- `println!` / `eprintln!`: 0
- `std::fs`: 1 (minor)
- process spawn: 13 (all downstream of the shell.mode gate)

This is the RIGHT logging posture for a security-critical exec engine -- and a
positive contrast to the dead-ff-logging-dep pattern across most W2-3 units.
CR-NR-058 opportunity: ensure the dev-logging captures external command
start/params (redacted)/exit-code + the shell.mode gate decision (allowed/denied/
prompted) at DEBUG/WARN. Recorded PA-LOG-025 (LOW): audit that the 10 existing logs
cover the security-relevant events (gate denial should be at least INFO/WARN for
audit) and add dev-logging on spawn/exit under the `dev-logging` gate. Lower
priority -- it already logs, unlike its siblings.

---

## 5. Task revision proposals

- **PA-STD-038 (REFACTOR)**: split `engine.rs` (523) by concern (mode gate /
  dispatch / outcome) and `emulator.rs` (535, folds into PA-SPLIT-010). No gate.
- **PA-SPLIT-010 (LOW, owner-gated)**: possible split of the terminal emulator from
  the exec engine. Low priority; defer.
- **PA-LOG-025 (LOW)**: audit that the shell.mode gate DENIAL is logged (INFO/WARN
  for audit) + dev-logging on spawn/exit. It already logs (10 calls) -- refinement,
  not a dead-dep fix.
- **PA-WATCH-022 (LOW)**: disambiguate the `TSO` verb between ff-shell (SHELL alias)
  and ff-command-semantics (emulated TSO P1/P2). Confirm routing.
- **PA-STD-039 (ASCII, runtime string)**: 65 non-ASCII bytes (1 non-comment) --
  em-dash in a runtime `#[error]` string (error.rs:107 "shell access denied -- ...").
  Replace with `--`. REFACTOR, no gate.

No requirement CHANGE proposed. The spec is complete and the security model is
exemplary.

---

## Summary

shell-command (`ff-shell`) is the SECURITY-CRITICAL external-execution engine
(SHELL/TSO: run OS commands, capture output, pipe stdin, interactive terminal +
emulator) -- and it is a POSITIVE exemplar on the two axes that matter most here:
(1) the `shell.mode` security gate (PA-WATCH-018 RESOLVED CLEAN) is a first-class,
ENFORCED control checked at the engine entry before any of the 13 spawn sites, with
a safe `prompt` default, three modes, and a macro DUAL-GATE (defense-in-depth); and
(2) it ACTUALLY LOGS (10 calls -- not the dead-dep pattern). command-configurator's
delegated external execution correctly inherits this gate. Complete tracking (184/184),
adequate TCR (25). Findings are mechanical: two over-cap files (engine.rs 523,
emulator.rs 535 -> PA-STD-038 + a LOW emulator split PA-SPLIT-010), a `TSO`-verb
disambiguation watch vs command-semantics' emulated TSO (PA-WATCH-022), a
security-audit-logging refinement (PA-LOG-025), and one runtime-string ASCII
(PA-STD-039). No new conflict, no dead-dep.
