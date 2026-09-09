# Analysis Record: logging-subsystem

- **Wave**: 0 (foundation -- every crate depends on `ff-logging`)
- **Backing crate**: `ff-logging`
- **Spec folder**: `docs/specs/logging-subsystem/`
- **Analysed**: Wave 0, task W0.5
- **Verdict**: FUNCTIONALLY COMPLETE, but with a TRACKING GAP.
  Req 1-11 done and TCR-PASS. Req 12 (Logging Inventory Tool, CR-NR-055) is
  IMPLEMENTED and works (tool + report present, deterministic), but its tasks
  (23-24) are still `[ ]` and its TCR rows (Phase DD) are still NOT COVERED.
  NOT a split candidate.

---

## 1. Scope summary

`ff-logging` is the foundational structured file-logging crate. 12 requirements:

- Req 1 init ordering; Req 2 record format; Req 3 level config; Req 4 file
  location; Req 5 rotation + retention; Req 6 flush/shutdown; Req 7
  GUI-independent process (no console, no stdout/stderr); Req 8 thread-safety +
  bounded buffer + drop counter; Req 9 platform-core integration; Req 10 plugin
  logging handle; Req 11 runtime reconfigure (two-phase init, bug B033);
  Req 12 Logging Inventory + Gap Report tool (CR-NR-055, read-only `tools/`).

245 req lines, 12 requirements, single backing crate. Cohesive.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 245 lines; exactly 12 reqs (not >12) | No |
| 3+ distinct responsibilities | one responsibility: file logging (Req 12 tool is a satellite maintenance script, not a separable product concern) | No |
| 2+ crates | one crate `ff-logging`; Req 12 tool is a Python script under `tools/`, not a crate | Borderline |
| low-cohesion concern clusters | Req 1-11 are one tightly-coupled subsystem; Req 12 is an out-of-process read-only tool | Weak |

Only the Req 12 tool is arguably separable, and it already lives outside the
crate (`tools/python/logging_inventory.py`). At most 1 weak criterion.
**NOT a split candidate.** Optional tidy: Req 12 could move to a `tooling`
or `project-analysis` spec, but it is small and self-contained -- leave as-is.

## 3. Consistency / conflict

- Public types owned here (LogLevel, LogConfig, LogRecord, PluginLogHandle,
  the `log_trace!`/`log_debug!`/`log_info!`/`log_warn!`/`log_error!` macros,
  `log()`/`log_lazy()`, dropped-record counter) -- sole owner `ff-logging`.
  Added to consistency-matrix.md. No duplicate ownership.
- Req 3/4/5 config keys (`logging.level`, `logging.directory`,
  `logging.max_file_size_mb`, `logging.max_retained_files`) are consistent with
  configuration-system reserved `logging` namespace and its `keys.rs` consts.
- Req 11 two-phase init (init with defaults -> load config -> `reconfigure`)
  matches configuration-system startup ordering and CR-CH-015 / bug B033.
  No conflict: ff-logging initialises before ff-config exists (FFW-ARCH-001),
  then is reconfigured. Correct dependency direction.
- Req 10 plugin handle aligns with plugin-architecture PluginContext. No
  conflict.

## 4. Completeness

- Tasks: 125 `[x]`, 12 `[ ]`. ALL 12 open boxes are Task 23 (7 subtasks) and
  Task 24 (3 subtasks) -- the Logging Inventory Tool (Req 12).
- BUT the tool IS present and functional:
  - `tools/python/logging_inventory.py` exists (v1.0.0, read-only, mirrors log
    to `tools/logs/logging-inventory.txt` per tooling.md).
  - `docs/quality/logging-inventory.md` report exists and is current.
  - Verified deterministic: ran the tool twice; reports byte-identical except
    the `Generated:` timestamp line (Property 12 holds).
  - Report output: 69 crates scanned, 1157 rust files, 128 log call sites,
    56 crates with zero log calls, 792 silent-error review candidates.
- Req 1-11: all task groups `[x]`; TCR rows for Req 11.1-11.10 all PASS.
- TCR gap: the "Phase DD" section rows for Req 12.1-12.5 are still `NOT COVERED`
  (red) even though the tool satisfies them.
- Completeness verdict: **implementation complete; tracking stale.** The
  remaining work is bookkeeping (check tasks 23-24, flip Phase DD TCR rows to
  PASS), NOT code. Logged as PA-TRACK-001 for owner action.

## 5. Logging audit

- Self-logging: `ff-logging` IS the logging subsystem, so it uses direct sink
  writes rather than the macros. Failure paths (dir create, file open, rotation,
  reconfigure) emit WARN records per Req 1.3/4.4/5.5/5.10/11.5 -- correct.
- `println!`/`eprintln!`: 2 hits at init.rs:14-15, both inside the module
  `//!` doc-comment that explicitly documents the crate NEVER writes to
  stdout/stderr (Req 7.4/7.5 invariant text). NOT runtime code -- no violation.
  Confirms Req 7 GUI-independence is upheld in code.
- PROJECT-WIDE logging data source: the generated
  `docs/quality/logging-inventory.md` is the authoritative inventory for the
  project-analysis logging audit (Req 5). Key signal: 56/69 crates have ZERO
  log call sites and 792 silent-error candidates exist. Reference this report
  for each later unit's logging audit rather than re-deriving. Recorded as
  PA-LOG-002.
- Logging verdict for THIS crate: **adequate**.

## 6. Findings logged

- **PA-TRACK-001** (tracking-fix): Req 12 (Logging Inventory Tool) is
  implemented, present, and verified deterministic, but tasks 23-24 remain `[ ]`
  and TCR "Phase DD" rows Req 12.1-12.5 (+ any 12.6-12.9) remain NOT COVERED.
  Owner action: check tasks 23.1-24.3 and set the Phase DD TCR rows to PASS with
  the tool/report as evidence. Bookkeeping only; no code change, no gate.
- **PA-LOG-002** (project-wide logging data): `docs/quality/logging-inventory.md`
  reports 56/69 crates with zero log call sites and 792 silent-error candidates.
  This is the master input for the project-analysis logging audit (Req 5); use
  it per-unit instead of re-scanning. Not a defect in ff-logging itself.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-logging contributes 88
  non-ASCII matches (em-dashes in doc-comments, box-drawing separators in test
  section headers). Box-drawing is disallowed in `.rs` per documentation.md
  (allowed only in Markdown). Rolled into project-wide PA-LOG-001; not fixed
  here.
