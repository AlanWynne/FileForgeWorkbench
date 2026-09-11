# Project Analysis -- Executive Summary (CR-NR-056)

Whole-project systematic analysis of all 81 sub-projects across 7 dependency waves
(W0-W6). Every unit has an analysis record under `units/`; every finding is in
`incomplete-work-register.md`; cross-unit relationships are in `consistency-matrix.md`;
per-unit status is in `checklist.md`; remediation is proposed as Phases PA-W0..PA-W5 in
`docs/specs/project-master/tasks.md`. This analysis was NON-DESTRUCTIVE: no source
outside `docs/` was changed. All findings are PROPOSALS pending owner approval.

## Scope + coverage

- 81 sub-projects analysed; 81 DONE, 0 PENDING.
- 263 findings recorded, by category: PA-CONFLICT 22, PA-INCOMPLETE 16, PA-STD 73,
  PA-LOG 52, PA-TCR 36, PA-SPLIT 14, PA-WATCH 25, PA-DOC 11, PA-DEP 5, PA-TRACK 9.
- The vast majority of sub-projects TRACK as complete; the analysis measures the CODE +
  cross-unit REALITY against the specs, which is where the material findings lie.

## The headline: SPEC/TRACKING is healthy; INTEGRATION is where the gaps are

The single most important conclusion: requirements are well-mapped (ears-integration
finds 0 orphaned requirements) and most crates are individually complete + tested. The
gaps are almost entirely at the INTEGRATION / cross-unit level -- crates built but not
wired, capabilities implemented more than once, and write paths bypassing the safe layer.
The project's own pre-analysis validation (project-master readiness + the Task-18.x
verification reports) validated SPEC coverage, not implementation, and so presents a
"ready" picture the code contradicts (PA-TRACK-007/008).

## Theme 1 -- ORPHAN CRATES (dominant structural finding)

SIX complete, tested crates are used by NO crate, in three sub-flavors:
- Complete-but-unwired: `ff-file-tree` (PA-CONFLICT-011), `ff-idle-processing`
  (PA-CONFLICT-012), `ff-large-file-performance` (PA-CONFLICT-013), `ff-external-mod`
  (PA-CONFLICT-014).
- Foundation-crate-ignored-by-its-consumer: `ff-language-service` (PA-CONFLICT-018) --
  `ff-syntax-highlighting` reimplements detection inline; it ALSO reimplements idle-styling
  inline, ignoring `ff-idle-processing` too (the SAME consumer ignores TWO foundation
  crates).
- Framework-orphaned-while-point-solutions-win: `ff-viewers` (PA-CONFLICT-019) -- an
  orphaned viewer framework duplicating the wired `ff-asa` + `ff-hex`.
Plus `ff-idcams` (PA-CONFLICT-016): a complete engine wired to NOTHING, with its spec-
designed dual integration (command-framework registration + JES EXEC PGM=IDCAMS) unbuilt.
NOTE: `ff-connector-extensibility` was initially flagged as a 7th orphan (W5.13) but
DOWNGRADED (W5.15) -- it is the INTENTIONAL base for the DEFERRED remote connectors, not
abandoned. Recommendation: rewire consumers/shell onto these crates (preferred -- all are
tested + well-decomposed) or delete + reconcile the spec, per crate.

## Theme 2 -- REIMPLEMENT-INSTEAD-OF-REUSE duplication

Capabilities implemented multiple times instead of shared:
- ASA carriage control in THREE crates: `ff-asa` (wired owner) + `ff-viewers/asa_report.rs`
  + `ff-forge/asa.rs` (PA-CONFLICT-019/022).
- EBCDIC in TWO: `ff-encoding` (owner) + `ff-forge/ebcdic.rs` (PA-CONFLICT-022).
- hex in TWO: `ff-hex` + `ff-viewers/hex.rs` (PA-CONFLICT-019).
- PREVIEW command DOUBLE-OWNER: `ff-viewers` + `ff-asa` (PA-CONFLICT-019).
- language detection: `ff-language-service` vs inline in `ff-syntax-highlighting`
  (PA-CONFLICT-018).
Recommendation: consolidate each capability onto a single owner; the others depend on it
(or document intentional divergence + share the low-level tables).

## Theme 3 -- DATA-SAFETY raw-fs WRITE bypasses (highest risk)

Two WRITE paths use raw `std::fs`, bypassing the mandated ff-vfs / ff-file-ops safe-write
(atomic rename-on-write + backup + read-only detection):
- JES job-queue persistence (`ff-jes/queue.rs`, PA-CONFLICT-015).
- global-search cross-file REPLACE (`ff-global-search/replace.rs`, PA-CONFLICT-020) -- a
  BULK mutation of user files with no atomic/backup guarantee (a crash mid-replace can
  corrupt files).
These are the highest-risk findings (data loss). NOTE: legitimate raw fs is OUT of scope
-- `ff-connector-local-fs` IS the fs layer; `ff-fftest` writes test artifacts; toolchain
locates OS compilers; lua scans the macro dir. Recommendation: route the two write paths
through ff-vfs / ff-file-ops.

UPDATE (PA-W5.2 IMPLEMENTED): the ATOMICITY half of both findings is now FIXED in code --
both write paths write via a sync temp-file + fsync + `std::fs::rename` helper
(`atomic_persist_write` in ff-jes/queue.rs; `atomic_write` in ff-global-search/replace.rs),
so an interrupted/partial write can no longer corrupt the job queue or a user file. TDD
(4 new tests), verify.ps1 FULL gate clean. Kept SYNCHRONOUS deliberately: the ff-vfs/
ff-file-ops safe-write API is uniformly async + URI-based with no sync wrapper, and
converting ff-jes would force adding tokio + an async ripple through all callers -- out of
proportion to the fix. STILL DEFERRED (owner-gated follow-up): full VFS-routing (the
FFW-ARCH-001 discipline half), backup-copy on replace, and PA-DEP-005 (global-search still
does not consume ff-file-ops). The corruption RISK -- the highest-priority concern -- is
resolved.

## Theme 4 -- TRACKING INTEGRITY

- `ff-database-tool` FALSE-POSITIVE-COMPLETE (PA-INCOMPLETE-014): 157/157 tasks `[x]` but
  the implementation is a thiserror+serde SKELETON with ~14/17 requirements (panels,
  pooling, async, ER, transfer, admin, integrations) UNBUILT. Re-open the tasks.
- The master narrative is stale vs the analysis (PA-TRACK-007) + the Task-18.x verification
  reports verify SPEC not implementation (PA-TRACK-008). Reconcile both to point at
  project-analysis as the source of truth for outstanding work.
- Honestly-tracked partial work (positive, contrast the above): batch-execution 10.2,
  lua-macro Macro Library panel + FFCMD chaining.

## Theme 5 -- CR-NR-057 command-chaining bundle (Phase DH)

Five sub-projects share ONE unbuilt command-chaining capability: command-framework Req 10
(PA-INCOMPLETE-003), command-semantics Req 11 split_chain/execute_chain
(PA-INCOMPLETE-007), menu-workspace Req 5 (PA-INCOMPLETE-008), function-keys Req 17
(PA-INCOMPLETE-009), and lua-macro FFCMD sequences (PA-INCOMPLETE-016) -- all should reuse
ONE shared chain executor (PA-WATCH-017). Unify as Phase DH; do not build separate
executors.

## Theme 6 -- CR-NR-058 dev/debug logging (user priority)

Logging is uneven. HIGH-VALUE gaps: job engines log NOTHING (JES PA-LOG-041, idcams
PA-LOG-042); lua-macro security-gate decisions are audit-relevant with a DEAD ff-logging
dep (PA-LOG-046); data-safety paths (external-mod watcher PA-LOG-039, file-ops persistence
PA-LOG-040, global-search bulk replace PA-LOG-050). A pervasive DEAD-ff-logging cluster
(declared, 0 uses) spans many crates. POSITIVE exemplars that DO log: batch-execution,
shell-command, connector-extensibility, connector-local-fs. Recommendation: resolve dead
deps + add gated dev-logging on execution/security/data-safety paths first.

## Theme 7 -- STANDARDS hygiene (mechanical, low-risk)

- 400-line cap: worst is `ff-idcams/parser/mod.rs` at 1372 non-test (41 fns in a mod.rs);
  plus the ff-desktop shell (PA-STD-042), and ~10 other files 400-560. Split by concern.
- ASCII: 73 PA-STD findings -- runtime em-dashes / multiplication (U+00D7) / minus (U+2212)
  signs; mojibake comment separators in .rs (recurring: ff-jes/ffjcl, database-tool,
  toolchain, connector-local-fs); design.md mojibake STATUS lines. Replace with ASCII.
- TCR: 36 gaps -- total absence on several orphan crates; thin (1-2 rows) on many; EXEMPLARS
  (do NOT touch): JES 74, fftest 62, toolchain 38.
- Crate-name drift on ~10 crates (spec name != dir); a naming-reconciliation set (PA-DOC).

## POSITIVE exemplars (what "good" looks like here)

ff-theme (clean single-owner), compiler-toolchain-integration (complete + wired + 38 TCR),
batch-execution (uses ff-logging), compare-and-merge (single owner, clean-seam,
VFS-by-interface), connector-local-fs (correct primary VFS provider), automated-dialog-
testing (egui-decoupled AutomationId model, 62 TCR), bootstrap-scripts (zero findings),
JES (plugin, 74 TCR). The CLEAN-SEAM pattern (crate returns a data model; the shell
renders it; no egui dep) recurs correctly across many crates and is the project's sound
GUI-independence approach. The two security-execution surfaces (shell.mode + lua
SecurityMode) both enforce a gated safe default.

## Recommended remediation order (from Phases PA-W0..PA-W5)

1. DATA-SAFETY first: route JES queue + global-search replace through ff-vfs/ff-file-ops
   (PA-W5.2) -- highest risk.
2. TRACKING INTEGRITY: re-open database-tool (PA-W5.5); reconcile master + verification
   narrative (PA-TRACK-007/008).
3. ORPHAN + DUPLICATION decisions (owner-gated, MEDIUM-HIGH): rewire-or-delete the 6
   orphans (PA-W4.1 + PA-W5.4); consolidate ASA/EBCDIC/hex/PREVIEW/language-detection
   (PA-W5.1).
4. CR-NR-057 Phase DH (unify command chaining) + CR-NR-058 logging on execution/
   security/data-safety paths (PA-W5.9 + the per-wave PA-LOG tasks).
5. MECHANICAL hygiene (LOW, batchable): cap splits (PA-W5.7/8), ASCII cluster (PA-W5.10),
   TCR enumeration (PA-W5.11), naming/docs reconciliation (PA-W5.12), dead-dep pruning.

## Process notes

- Two evidence-driven SELF-CORRECTIONS during the analysis sharpen an important
  distinction: ABANDONED vs AWAITING-DEFERRED-WORK. (a) ff-file-ops "consumed by
  global-search" was declaration-only -- the consumer uses raw std::fs (PA-DEP-005). (b)
  connector-extensibility looked orphaned but is the intentional base for the deferred
  remote connectors (PA-CONFLICT-021 downgraded). Always check ACTUAL code usage, not
  Cargo.toml declarations, before asserting "consumed" / "orphan".
- The connector family is coherent: 1 built primary provider (local-fs) + 1 intentional
  base (connector-extensibility) + 4 honestly-DEFERRED remote connectors (network-fs,
  ftp-sftp, cloud, mainframe), with a clean LOCAL-emulation (dataset-catalog/JES/IDCAMS)
  vs REMOTE-connectivity boundary.
