# Analysis Record: idcams-emulator (W5.2)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-idcams` (IDCAMS emulator -- command parsing + orchestration
  for IBM Access Method Services: DEFINE/DELETE/ALTER/LISTCAT/PRINT/REPRO/VERIFY/
  EXPORT/IMPORT/BLDINDEX, modal IF/THEN/ELSE, SYSIN, return codes)
- **Spec files**: requirements.md (556 lines, 26 requirements), tasks.md
  (245 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy + SEVERE cap violations (PA-STD-057)

26 requirements (the MOST of any unit; >12 threshold) and 556 req lines. But the reqs
are cohesive (one IDCAMS emulator) and the crate is already well-structured by concern
(parser/ + executor/ + pretty_printer/ + services + messages + sysin). So this is NOT
primarily a CRATE split -- it is a SEVERE FILE-SIZE problem:

- `parser/mod.rs` = **1372 non-test lines** (41 functions) -- the WORST file-size
  violation found in the entire analysis. A `mod.rs` should be re-exports only (per
  rust-standards), not 1372 lines of parse logic.
- `executor/handlers.rs` = 833, `services.rs` = 765, `parser/ast.rs` = 538,
  `pretty_printer/mod.rs` = 366. FIVE files over the 400 cap.

Recorded PA-STD-057 (HIGH -- severe cap): split `parser/mod.rs` by parse concern
(per-command DEFINE/DELETE/ALTER/... parse modules, or statement-group modules), move
logic OUT of the mod.rs into named submodules (mod.rs -> re-exports), and split
handlers.rs (per-command handler modules), services.rs (AllocatorService / CatalogService
/ VsamService into separate files), and ast.rs. This is comparable in severity to the
ff-desktop shell split (PA-STD-042 / PA-W3.6). REFACTOR (no behaviour change).

No separate PA-SPLIT crate-split is proposed (the crate is cohesive; the problem is
file granularity, not crate responsibility) -- contrast JES (W5.1) where a genuine
second responsibility (SDSF) warranted a crate split.

---

## 2. Cross-unit consistency -- PA-CONFLICT-016 (orphan + MISSING dual integration)

`ff-idcams` provides a clean, complete public engine: `execute_idcams(input, services)`,
`CommandExecutor`, `IdcamsParser`, `IdcamsServices` (Allocator/Catalog/Vsam), pretty
printer. But it is referenced by NO crate:

- 0 `ff_idcams` refs anywhere outside the crate. 0 refs in ff-command, ff-plugin,
  ff-desktop shell. 0 refs in **ff-jes**.
- No `FileForgePlugin` impl (0) -- unlike JES (W5.1) which registers as a plugin and is
  wired by name 19x, idcams has NO wiring scaffolding at all.

Yet the spec explicitly DESIGNS two integration paths (Req 20):
- "THE ff-idcams crate SHALL register with the FileForgeWorkbench command framework"
  (command-framework registration), AND
- "WHEN invoked via JCL (EXEC PGM=IDCAMS), THE Command_Executor SHALL ..." + the design
  shows `JCL EXEC PGM=IDCAMS` -- i.e. IDCAMS should be invokable as a JES BATCH JOB STEP.

Neither path is wired. This is an ORPHAN (like the four Wave-4 orphans) but with a
sharper twist: the spec NAMES the exact integration points (Req 20) that were never
built, and the natural coupling -- JES running IDCAMS as a batch step (EXEC PGM=IDCAMS)
-- is entirely absent (ff-jes has 0 idcams refs). Recorded PA-CONFLICT-016 (owner-gated,
MEDIUM-HIGH): (a) register ff-idcams with the command framework (Req 20.1), and (b)
wire the JES EXEC PGM=IDCAMS batch-step invocation (Req 20.4) so IDCAMS runs inside JES
jobs. Pairs with the Wave-4 orphan cluster (PA-W4.1) and the JES engine (W5.1). Code +
owner decision.

### VFS discipline -- CLEAN (0 raw fs, unlike JES)

Unlike JES (PA-CONFLICT-015 raw-fs violation), ff-idcams has 0 std::fs/tokio::fs. It
operates on `IdcamsServices` (injected Catalog/Vsam/Allocator services) rather than
touching the filesystem directly -- clean dependency injection, no VFS bypass. Positive
(the services it calls should themselves be VFS-mediated, verified in their own units).

### Public types and ownership

- IDCAMS parser (lexer/token/ast), CommandExecutor, condition codes (LASTCC/MAXCC),
  IdcamsServices trait surface, pretty printer -- sole-owned by `ff-idcams`. No
  duplication. Return-code management (Req 15: LASTCC/MAXCC) is IDCAMS-specific.

### Cross-reference integrity

Cross-refs to command-framework + the catalog/VSAM/allocator services resolve as
sub-projects, but the command-framework REGISTRATION and JES invocation are unbuilt
(PA-CONFLICT-016).

---

## 3. Completeness

Tracking: all 245 sub-tasks `[x]`. The engine is complete + parses/executes all 14
IDCAMS commands, modal IF/THEN/ELSE, SYSIN, return codes, pretty printer. FUNCTIONALLY
complete AS A CRATE -- but not wired (PA-CONFLICT-016), so IDCAMS is not reachable by
the user (no command registration, no JES step). No PA-INCOMPLETE for the crate;
the gap is the missing integration (architectural).

### TCR gap (PA-TCR-028) -- THIN (1 row vs 26 reqs)

TCR.md has 1 row for ff-idcams against 26 reqs / ~200 criteria -- the widest gap
relative to spec size seen (contrast JES's exemplary 74). The crate's in-file tests
exist (245 tasks). Recorded PA-TCR-028 (notable given the crate's size).

---

## 4. Logging audit

Scan of `crates/ff-idcams/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: ABSENT -- NOT a dead-dep case.
- `println!` / `eprintln!`: 0
- `std::fs`: 0 (clean, no VFS bypass)

An IDCAMS command orchestrator with atomic-execution guarantees (Req 22), return-code
registers (Req 15), and modal control flow (Req 16) is a meaningful trace site: command
parse, per-command execute, LASTCC/MAXCC transitions, atomic rollback, IF/THEN/ELSE
branch decisions. Zero logging. Recorded PA-LOG-042 (MEDIUM): add ff-logging + dev-
logging on command execute / condition-code transitions / atomic-rollback / modal
branches under the `dev-logging` gate when wired (PA-CONFLICT-016). Pairs with JES
PA-LOG-041 (both emulators log nothing).

---

## 5. Task revision proposals

- **PA-STD-057 (HIGH -- severe cap)**: split `parser/mod.rs` (1372!), handlers.rs (833),
  services.rs (765), ast.rs (538), pretty_printer/mod.rs (366) by concern; move logic
  out of mod.rs files into named submodules. Comparable to the shell split (PA-W3.6).
  REFACTOR.
- **PA-CONFLICT-016 (owner-gated, MEDIUM-HIGH)**: orphan + missing dual integration --
  (a) register with the command framework (Req 20.1); (b) wire JES EXEC PGM=IDCAMS
  batch-step invocation (Req 20.4). Code + owner decision.
- **PA-LOG-042 (MEDIUM)**: add ff-logging + command-execute / condition-code / atomic-
  rollback / modal-branch dev-logging (when wired).
- **PA-TCR-028**: enumerate per-requirement TCR rows (1 for 26 reqs -- widest gap by
  spec size). No code.

No PA-STD runtime-string ASCII item (0 non-comment non-ASCII -- clean). No raw-fs (clean,
unlike JES).

---

## Summary

idcams-emulator (`ff-idcams`) is a complete (245/245), cohesive IDCAMS command
parser + orchestrator covering all 14 commands, modal IF/THEN/ELSE, SYSIN, and
return-code management (26 reqs -- the most of any unit). Two headline findings.
(1) PA-STD-057 (HIGH): the WORST file-size violation in the analysis -- `parser/mod.rs`
at 1372 non-test lines (41 functions in a mod.rs that should be re-exports only), plus
handlers.rs (833), services.rs (765), ast.rs (538), pretty_printer/mod.rs (366) -- five
files over cap; split by concern (comparable to the shell split). (2) PA-CONFLICT-016
(MEDIUM-HIGH): ORPHAN + missing dual integration -- 0 refs anywhere, no FileForgePlugin,
and NEITHER of the two integration paths the spec explicitly designs (Req 20: command-
framework registration + JES EXEC PGM=IDCAMS batch-step invocation) is wired; notably
ff-jes has 0 idcams refs, so the natural JES-runs-IDCAMS coupling is entirely absent.
Positives: CLEAN VFS discipline (0 raw fs, unlike JES PA-CONFLICT-015 -- it uses injected
IdcamsServices). Gaps: PA-LOG-042 (MEDIUM, logs nothing across execute/condition-code/
atomic/modal) and PA-TCR-028 (1 row vs 26 reqs -- widest coverage gap by spec size,
contrast JES's 74). The engine itself is sound; the work is a big cap-split + the missing
command/JES wiring.
