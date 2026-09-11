# Analysis Record: lua-macro-engine (W5.8)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-lua` (Lua 5.4 macro/scripting engine via `mlua`: editor API,
  event hooks, per-buffer state, security modes, auto-reload, macro dir scanning,
  debugging, ISPF edit-macro API + REXX execution)
- **Spec files**: requirements.md (309 lines, 12 requirements), tasks.md
  (227 sub-tasks: 220 `[x]`, 7 `[ ]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy + cap violations (PA-STD-062)

12 reqs / 309 lines -- not a crate-split candidate on requirement count (cohesive
scripting engine). But FOUR files at/over the cap:
- `engine.rs` = 506 non-test lines (well over cap -- the runtime core).
- `tso_builtins.rs` = 431, `ispf.rs` = 405 (over cap); `execio.rs` = 382 (near).

Recorded PA-STD-062 (MEDIUM -- cap): split engine.rs by concern (runtime init /
execution + security gate / API-registration / state), and split the builtin surfaces
(tso_builtins, ispf) by command group. REFACTOR, no behaviour change.

---

## 2. Cross-unit consistency

### Security modes -- ENFORCED (POSITIVE exemplar, like shell.mode)

Req 7 (Macro Security Modes) is a FIRST-CLASS ENFORCED gate, not a stub: `security.rs`
defines `SecurityMode` (Disabled / Prompt / TrustedOnly / Enabled) + a
`SecurityPermission` decision, with a SAFE default (Prompt for new installations), and
engine.rs:84 GATES execution on the mode (`if security_mode != Enabled { ... }`). This
mirrors the shell-command shell.mode security gate (PA-WATCH-018, the W3.8 exemplar) --
untrusted-code execution is gated before it runs. POSITIVE consistency finding: the two
code-execution surfaces (external shell + Lua macros) BOTH enforce a mode gate with a
safe default. Recorded as a CLEAN/exemplar row.

### WIRED (NOT an orphan)

`ff-lua` is referenced 437x outside the crate (MACRO/EXEC/RUN commands, plugin API,
editor API surface). Heavily integrated. NOT an orphan.

### CR-NR-057 command-chaining -- another leg (cross-wave link)

Open tasks 26.x route "each macro/FFCMD line through the shared command-semantics chain
executor" + compose fail-stop (command-semantics Req 11.4) -- i.e. lua-macro Req 5.8-5.11
is ANOTHER LEG of the CR-NR-057 COMMAND-CHAINING BUNDLE identified in Wave 3
(command-framework Req 10 PA-INCOMPLETE-003, command-semantics Req 11 split_chain/
execute_chain PA-INCOMPLETE-007, menu-workspace Req 5 PA-INCOMPLETE-008, function-keys
Req 17 PA-INCOMPLETE-009, shared split_chain PA-WATCH-017). The Wave-3 record proposed
unifying these as "Phase DH". lua-macro's FFCMD-sequence work belongs in that same phase
(reuse the shared chain executor -- tasks 26.x explicitly say so; no piping introduced,
`editor.command(str)` returns only success). Recorded under PA-INCOMPLETE-016 with an
explicit cross-reference to the Phase DH bundle.

### raw-fs macro scanning (PA-WATCH-026)

engine.rs:163 reads a macro script (`std::fs::read_to_string`) and scanner.rs:94 scans
the macro directory (`std::fs::read_dir`) with raw std::fs (the other ~14 scanner fs
hits are `#[cfg(test)]` fixtures). Macro scripts live in a macro DIRECTORY (user files),
not the workbench VFS -- so raw fs is arguably acceptable (like toolchain locating OS
compilers, W5.5), LOWER-stakes than the JES job-DB raw-fs (PA-CONFLICT-015). Recorded
PA-WATCH-026 (LOW): decide whether macro-dir scanning should be VFS-mediated for
consistency, or is legitimately outside the VFS (likely the latter). Watch, not a hard
violation.

### Public types and ownership

- Lua runtime (mlua), editor-API bindings, event-hook registry, per-buffer state,
  SecurityMode/permission, script scanner, ISPF/REXX/TSO builtins -- sole-owned by
  `ff-lua`. No duplication (it is the only scripting engine).

---

## 3. Completeness -- PA-INCOMPLETE-016 (two honestly-tracked open groups)

220 of 227 `[x]`; 7 open in TWO coherent groups (honest tracking):
1. Task 25 "Macro Library panel (Phase CR)" -- the Req 12 Macro Library MANAGEMENT UI
   (panel) is unbuilt. The engine + management logic exist; the panel does not.
2. Tasks 26.1-26.5 "Macro and FFCMD command sequences" -- the CR-NR-057 chaining leg
   (route FFCMD lines through the shared chain executor, fail-stop, tests, TCR for
   Req 5.8-5.11).

Recorded PA-INCOMPLETE-016 (MEDIUM): (a) build the Macro Library panel (Req 12); (b)
complete the FFCMD-sequence chaining (tasks 26.x) AS PART OF the CR-NR-057 Phase DH
bundle (do not build a separate chain executor -- reuse the shared one). Honestly
tracked `[ ]` (contrast database-tool false-positive-complete). Code + tests.

### TCR (36 rows) -- reasonable

TCR.md has 36 rows for ff-lua across 12 reqs -- reasonable coverage (the open 26.x tasks
include a TCR update for Req 5.8-5.11). Minor top-up only; no PA-TCR item raised.

---

## 4. Logging audit

Scan of `crates/ff-lua/src` (recursive):

- `ff_logging` / `log_*!`: 0; `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0.
- `std::fs`: ~2 real (script read + dir scan) + test fixtures (PA-WATCH-026).

A SECURITY-SENSITIVE scripting engine is a high-value logging site: macro execution
start/end + which macro, SECURITY-MODE gate decisions (allowed/prompted/denied -- an
audit-relevant event), error + rollback (Req 6), auto-reload triggers (Req 8), directory
scan results (Req 9). ff-logging is a DEAD dep. Recorded PA-LOG-046 (MEDIUM-HIGH -- the
security-gate decisions make this audit-relevant, above the usual LOW): resolve the dead
dep + add dev-logging on macro execute / security-gate decisions / error-rollback /
auto-reload / scan under the `dev-logging` gate. Strong CR-NR-058 candidate (pairs with
the shell-command logging which DOES exist -- lua-macro should match it).

---

## 5. Task revision proposals

- **PA-STD-062 (MEDIUM -- cap)**: split engine.rs (506), tso_builtins.rs (431),
  ispf.rs (405) [execio.rs 382 near] by concern. REFACTOR.
- **PA-INCOMPLETE-016 (MEDIUM)**: (a) Macro Library panel (Req 12, task 25);
  (b) FFCMD-sequence chaining (tasks 26.x) as part of CR-NR-057 Phase DH (reuse the
  shared chain executor). Code + tests.
- **PA-LOG-046 (MEDIUM-HIGH)**: resolve dead ff-logging + add macro-execute / SECURITY-
  GATE-decision / error-rollback / auto-reload / scan dev-logging (audit-relevant;
  should match the shell-command logging).
- **PA-DOC-008 (naming)**: reconcile spec `ff-macro` to crate dir `ff-lua`
  (naming-reconciliation set). Docs only.
- **PA-WATCH-026 (LOW)**: decide VFS-mediation for macro-dir scanning (likely fine as-is,
  outside the VFS). Watch.

No PA-INCOMPLETE for the engine (complete + enforced). No orphan. No false-complete.

---

## Summary

lua-macro-engine (`ff-lua`) is a heavily-integrated (437 external refs), well-tested
(36 TCR) Lua 5.4 scripting engine (mlua) with a rich editor API, event hooks, per-buffer
state, auto-reload, and ISPF/REXX/TSO builtins. POSITIVE exemplar: Req 7 security modes
are a FIRST-CLASS ENFORCED gate (SecurityMode Disabled/Prompt/TrustedOnly/Enabled, safe
Prompt default, execution gated at engine.rs:84) -- mirroring the shell.mode gate
(PA-WATCH-018), so BOTH code-execution surfaces gate untrusted code before running.
Findings: PA-STD-062 (four cap files -- engine.rs 506, tso_builtins 431, ispf 405,
execio 382 near; split by concern); PA-INCOMPLETE-016 (MEDIUM, two HONEST open groups --
the Req 12 Macro Library panel [task 25], and the FFCMD-sequence chaining [tasks 26.x]
which is ANOTHER LEG of the CR-NR-057 command-chaining bundle -> fold into Phase DH,
reuse the shared chain executor); PA-LOG-046 (MEDIUM-HIGH -- DEAD ff-logging on a
security-sensitive engine; security-gate decisions are audit-relevant, and lua-macro
should match the shell-command logging that already exists); PA-DOC-008 (name drift
ff-macro vs ff-lua); PA-WATCH-026 (LOW -- raw-fs macro-dir scanning, likely fine outside
the VFS). Genuinely complete engine + honest tracking; the gaps are the panel, the
chaining leg, logging, and caps.
