# Analysis Record: jes-emulator (W5.1)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-jes` (FFW-JES -- Job Entry Subsystem: job submission, queue/
  scheduling, initiator pool, monitoring, SYSOUT/logs, retention, SDSF-style panels;
  registers as a `FileForgePlugin`)
- **Spec files**: requirements.md (657 lines, 18 requirements), tasks.md
  (282 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass
- **Naming**: folder `jes-emulator` (was `FFW-JES`, renamed Phase BR); crate `ff-jes`.

---

## 1. Split candidacy -- SPLIT CANDIDATE (PA-SPLIT-013)

STRONG split candidate. Signals (4+ of the criteria met):

- 18 requirements (>12 threshold); 657 requirement lines (>~350).
- TWO clear responsibilities, cleanly separated in BOTH the reqs and the files:
  - **JES job engine** (Reqs 1-15): plugin lifecycle, submission, queue/scheduling,
    initiator pool, monitoring, completion/failure/cancel, logs/SYSOUT, retained
    output/purge, hold/release, catalog integration, job+dataset APIs, command
    integration, provider abstraction, async/concurrency. Files: config, error,
    ffjcl, initiator, log, model, provider, queue, retention, scheduler (+ lib).
  - **SDSF panel framework** (Reqs 16-18): panel framework core/extended, P2
    overtype/help/log+system panels/browse+print/SET P2. Files: sdsf_action,
    sdsf_commands, sdsf_filter, sdsf_filter_expr, sdsf_help, sdsf_log_panels,
    sdsf_overtype, sdsf_panel, sdsf_set_p2, sdsf_system_panels (10 files).
- TWO files over the 400 non-test cap: ffjcl.rs (469), sdsf_system_panels.rs (409);
  five more in the 350-390 band (model 390, queue 390, sdsf_commands 369,
  sdsf_filter 375, sdsf_help 358) -- the crate is straining the cap broadly.

Recorded PA-SPLIT-013 (MEDIUM): split `ff-jes` into `ff-jes` (job engine, Reqs 1-15)
+ `ff-jes-sdsf` (SDSF panel framework, Reqs 16-18), with the SDSF layer depending on
the engine's public Job/JobProvider API. The file boundary already matches the seam
(sdsf_* vs the rest), so the split is low-risk. Also resolves the two cap violations
by separating the two large clusters. REFACTOR (no behaviour change) once the seam is
confirmed.

---

## 2. Cross-unit consistency

### PA-CONFLICT-015 (NEW) -- raw std::fs job-queue persistence bypasses ff-vfs

`ff-jes` persists its job queue (JSON job DB) with RAW `std::fs`, not through ff-vfs:
- queue.rs:49 `std::fs::read_to_string(db_path)` (load on open),
  queue.rs:328 `std::fs::create_dir_all(parent)`, queue.rs:334 `std::fs::write(path, data)`.
- The crate has NO ff-vfs dependency (sole dep thiserror + serde).

This is the FIRST raw-fs VIOLATION found in the analysis -- a direct contrast to the
EXEMPLARY VFS discipline documented last wave (external-mod PA-CONFLICT-014 record +
file-ops W4.15, both 0 real fs, all I/O through ff-vfs per FFW-ARCH-001). The rest of
the project routes file I/O through the VFS abstraction; the JES queue store bypasses
it. Recorded PA-CONFLICT-015 (MEDIUM): route the queue-persistence store through ff-vfs
(add the dep; replace read_to_string/create_dir_all/write with VFS equivalents) so the
JES job DB participates in the same virtual-filesystem layer (connectors, testing via
in-memory VFS, path abstraction). Code.

### PLUGIN -- NOT an orphan (explicit NON-finding)

`ff-jes` is referenced by no crate's Cargo.toml (0 static deps) BUT this is EXPECTED
and correct: it registers as a `FileForgePlugin` (Req 1; lib.rs exposes the JES
subsystem; ff-desktop/ff-plugin reference "jes" 19 times via named/dynamic wiring).
Unlike the four Wave-4 orphans (which were NOT plugins and had no registration path),
JES is a properly-wired dynamic plugin. Recording this explicitly so the 0-static-dep
signal is NOT mistaken for the orphan pattern. Provider abstraction (`JobProvider`
trait: submit/hold/release/cancel/list; `ProviderRegistry`, `DesktopJesProvider`) is
a clean extensibility seam (Req 14) -- consistent with the connector/toolchain plugin
model.

### Catalog + command integration

Req 11 (dataset catalog integration) + Req 13 (command integration) route through the
catalog + command-framework families (Waves 2/0-3) -- consumer side, consistent. Job
and dataset APIs (Req 12) are the plugin's public surface.

### Public types and ownership

- Job/JobId/JobFilter, queue, scheduler, initiator pool, retention, SYSOUT logs, the
  SDSF panel framework, JobProvider/ProviderRegistry -- sole-owned by `ff-jes`. No
  duplication with other units.

### Cross-reference integrity

Cross-refs (ff-plugin, ff-command, dataset-catalog, connectors) resolve as
sub-projects. Missing: ff-vfs (should be a dep -- PA-CONFLICT-015).

---

## 3. Completeness

Tracking: all 282 sub-tasks `[x]`. Implementation present across all 18 reqs incl. the
SDSF P1/P2 panel framework. No PA-INCOMPLETE. Complete.

### TCR -- EXEMPLARY (74 rows)

TCR.md has 74 rows for ff-jes -- the BEST coverage enumeration seen in the analysis so
far (contrast the four Wave-4 orphans at 0). Positive exemplar; no PA-TCR gap.

---

## 4. Logging audit

Scan of `crates/ff-jes/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: ABSENT -- NOT a dead-dep case. But see below.
- `println!` / `eprintln!`: 0
- `std::fs`: 3 (raw, PA-CONFLICT-015)

A JES/job-execution engine is a HIGH-VALUE logging site: job submit / queue enqueue /
scheduler dispatch / initiator assignment / job complete-fail-cancel / hold-release /
retention purge are exactly the lifecycle transitions an operator needs to trace, and
async concurrency (Req 15) makes silent failures hard to diagnose. Zero logging on a
job engine is a real CR-NR-058 gap. Recorded PA-LOG-041 (MEDIUM): add ff-logging + dev-
logging on the job-lifecycle transitions + scheduler/initiator decisions + queue-
persistence outcomes under the `dev-logging` gate. Pairs with the persistence-safety
theme (external-mod PA-LOG-039, file-ops PA-LOG-040).

---

## 5. Task revision proposals

- **PA-SPLIT-013 (MEDIUM)**: split `ff-jes` -> `ff-jes` (engine, Reqs 1-15) +
  `ff-jes-sdsf` (SDSF framework, Reqs 16-18); the sdsf_* file cluster already matches
  the seam; resolves the ffjcl.rs (469) + sdsf_system_panels.rs (409) cap violations.
  REFACTOR.
- **PA-CONFLICT-015 (MEDIUM)**: route the job-queue persistence store through ff-vfs
  (queue.rs:49/328/334) instead of raw std::fs; add the ff-vfs dep. FIRST raw-fs
  violation found -- contrasts the exemplary external-mod/file-ops discipline. Code.
- **PA-LOG-041 (MEDIUM)**: add ff-logging + job-lifecycle / scheduler / initiator /
  queue-persistence dev-logging (job engine = high-value trace site). Code.
- **PA-STD-056 (ASCII)**: 36 non-ASCII bytes, ALL in comments (0 non-comment) -- lower
  priority, but sweep the comment em-dashes/arrows for consistency. REFACTOR, no gate.

No PA-TCR gap (74 rows -- exemplary). No PA-INCOMPLETE (282/282). Not an orphan (plugin).

---

## Summary

jes-emulator (`ff-jes`) is a large (18 reqs, 657 lines, 282/282), well-tested
(74 TCR rows -- the best coverage in the analysis) JES plugin: job submission, queue/
scheduling, initiator pool, monitoring, SYSOUT, retention, plus a full SDSF-style panel
framework. It is a STRONG split candidate (PA-SPLIT-013, MEDIUM): two clean
responsibilities -- job engine (Reqs 1-15) vs SDSF panel framework (Reqs 16-18) -- already
separated at the file level (sdsf_* cluster), with two files over the 400 cap
(ffjcl.rs 469, sdsf_system_panels.rs 409) and five more straining it. The notable
consistency finding is PA-CONFLICT-015 (NEW, MEDIUM): the job-queue persistence store
uses RAW std::fs (queue.rs read/create_dir_all/write) and has no ff-vfs dep -- the FIRST
raw-fs VIOLATION found, in direct contrast to the exemplary VFS discipline of
external-mod + file-ops; route it through ff-vfs. Logging is a real CR-NR-058 gap
(PA-LOG-041, MEDIUM): a job-execution engine with async concurrency logs NOTHING across
the whole job lifecycle. IMPORTANT NON-finding: ff-jes is a properly-wired
`FileForgePlugin` (referenced by name 19x in shell/plugin), NOT an orphan -- its
0-static-dep signal must not be confused with the Wave-4 orphan pattern. Minor:
comment-only ASCII (PA-STD-056).
