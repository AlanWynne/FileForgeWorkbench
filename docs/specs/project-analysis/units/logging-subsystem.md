# Analysis Record: logging-subsystem

- **Wave**: 0 (foundation -- every crate depends on `ff-logging`)
- **Backing crate**: `ff-logging`
- **Spec folder**: `docs/specs/logging-subsystem/`
- **Analysed**: Wave 0, task W0.5 (CR-NR-057 re-baseline)
- **Verdict**: FUNCTIONALLY COMPLETE, with a TRACKING GAP. Req 1-11 done + TCR-PASS.
  Req 12 (Logging Inventory Tool, CR-NR-055) is IMPLEMENTED and verified
  deterministic, but tasks 23-24 are `[ ]` and TCR Phase DD rows are NOT COVERED.
  NOT a split candidate. One 400-cap refactor (init.rs 640).
- **CR-NR-057 impact**: NONE (logging specs untouched). Re-verified from code.

---

## 1. Scope summary

`ff-logging` is the foundational structured file-logging crate. 12 requirements:
Req 1 init ordering; Req 2 record format; Req 3 level config; Req 4 file
location; Req 5 rotation + retention; Req 6 flush/shutdown; Req 7 GUI-independent
process (no stdout/stderr); Req 8 thread-safety + bounded buffer + drop counter;
Req 9 platform-core integration; Req 10 plugin logging handle; Req 11 runtime
reconfigure (two-phase init, bug B033); Req 12 Logging Inventory + Gap Report
tool (CR-NR-055, read-only `tools/`).

245 req lines, 12 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 245 lines; exactly 12 reqs (not >12) | No |
| 3+ distinct responsibilities | one responsibility: file logging (Req 12 tool is a satellite maintenance script already outside the crate) | No |
| 2+ crates | one crate `ff-logging`; Req 12 tool is a Python script under `tools/` | No |
| low-cohesion clusters | Req 1-11 tightly coupled; Req 12 is a separate out-of-process tool but already separated | Weak |
| file-size pressure | `init.rs` 640 non-test over cap | Yes (1) |

At most 1-2 weak criteria. **NOT a split candidate.** Fix is the init.rs refactor.

## 3. Consistency / conflict

- Public types owned here (LogLevel, LogConfig, LogRecord, PluginLogHandle, the
  `log_trace!`/`log_debug!`/`log_info!`/`log_warn!`/`log_error!` macros,
  `log()`/`log_lazy()`, dropped-record counter) -- sole owner `ff-logging`.
  Added to consistency-matrix.
- Req 3/4/5 config keys (`logging.level`, `logging.directory`,
  `logging.max_file_size_mb`, `logging.max_retained_files`) consistent with the
  configuration-system reserved `logging` namespace (W0.3). No conflict.
- Req 11 two-phase init (init defaults -> load config -> `reconfigure`) matches
  configuration-system startup ordering + CR-CH-015 / bug B033. Correct dependency
  direction (ff-logging initialises before ff-config; FFW-ARCH-001). No conflict.
- Req 10 plugin handle aligns with plugin-architecture PluginContext. No conflict.

## 4. Completeness -- TRACKING GAP

- Tasks: 125 `[x]`, 12 `[ ]`. ALL 12 open are Task 23 (7 subtasks) + Task 24
  (3 subtasks) -- the Logging Inventory Tool (Req 12).
- BUT the tool IS present and functional:
  - `tools/python/logging_inventory.py` exists (read-only, mirrors log to
    `tools/logs/`).
  - `docs/quality/logging-inventory.md` report exists and is current.
  - Verified deterministic: ran the tool twice; reports byte-identical except the
    `Generated:` timestamp line (Property 12 holds).
  - Report output: 128 log call sites, 56 crates with zero log calls, 792
    silent-error review candidates.
- Req 1-11 all `[x]`; TCR rows for Req 11.1-11.10 PASS.
- TCR gap: the "Phase DD" section rows for Req 12.1-12.5 are still NOT COVERED
  (red) even though the tool satisfies them.
- Completeness verdict: **implementation complete; task + TCR tracking stale.**
  Logged PA-TRACK-002.

## 5. Logging audit

- Self-logging: `ff-logging` IS the logging subsystem, so it uses direct sink
  writes rather than the macros. Failure paths (dir create, file open, rotation,
  reconfigure) emit WARN records per Req 1.3/4.4/5.5/5.10/11.5 -- correct.
- `println!`/`eprintln!`: 2 hits at init.rs:14-15, both inside the module `//!`
  doc-comment that explicitly documents the crate NEVER writes to stdout/stderr
  (Req 7.4/7.5 invariant text). NOT runtime code -- no violation.
- PROJECT-WIDE data source (PA-LOG-002): `docs/quality/logging-inventory.md` is
  the authoritative inventory for the whole-project logging audit (Req 5).
  Key signal unchanged: 56/69 crates have ZERO log call sites; 792 silent-error
  candidates. Reference per-unit rather than re-scanning.
- Logging verdict for THIS crate: **adequate.**

## 6. Findings logged

- **PA-TRACK-002** (TRACKING-FIX): Req 12 Logging Inventory Tool is implemented
  and verified deterministic, but tasks 23-24 remain `[ ]` and TCR Phase DD rows
  Req 12.1-12.5 remain NOT COVERED. Owner action: check tasks 23.1-24.3; set the
  Phase DD TCR rows to PASS with the tool/report as evidence. Bookkeeping only;
  no code, no gate.
- **PA-LOG-002** (LOGGING DATA, project-wide): `docs/quality/logging-inventory.md`
  reports 56/69 crates with zero log call sites and 792 silent-error candidates.
  Master input for the project-analysis logging audit (Req 5); use it per-unit
  instead of re-scanning. Not a defect in ff-logging.
- **PA-STD-003** (REFACTOR -- 400-line cap): `init.rs` is 640 non-test lines,
  over the cap. Split by concern (e.g. `init_bootstrap.rs` / `init_writer.rs`;
  keep init.rs the public init/log entry points). REFACTOR, no gate.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-logging contributes 88
  matches (em-dashes in doc-comments, box-drawing separators in test headers).
  Box-drawing is disallowed in `.rs` per documentation.md. Rolled into
  project-wide PA-LOG-001; not fixed here.

---

## 7. ADDENDUM -- CR-NR-058 (added after W0.5 analysis)

CR-NR-058 (Phase DI) added **Requirement 13: Build-Profile Compile-Time Level
Gating** to this spec AFTER section 1-6 above were written. Updated facts:

- Requirement count is now **13** (not 12). Req 13 defines a `dev-logging` cargo
  feature: TRACE/DEBUG removed entirely from a release binary (no branch, no
  format, no atomic read); INFO/WARN/ERROR always retained and runtime-controlled
  by `logging.level` (Req 3). A public `BUILD_PROFILE_LEVEL` const is exported so
  downstream crates can gate their own expensive diagnostics on the same profile.
- **Req 13 is UNIMPLEMENTED**: `dev-logging` feature absent from
  `crates/ff-logging/Cargo.toml` (grep 0); `BUILD_PROFILE_LEVEL` const / cfg-split
  of `log_trace!`/`log_debug!` absent from src (grep 0). Phase DI Task 25
  (25.1-25.x) open; TCR Phase DI rows present but NOT COVERED. Requirements gate
  IS complete (Req 13 authored + design + tasks + TCR rows) -- ready to build.
- Split candidacy unchanged: 13 reqs now crosses the >12 line, but Req 13 is a
  small compile-time gate tightly bound to the macros -- still NOT a split
  candidate.
- **This is the KEY enabler for the project-analysis logging audit.** Req 13's
  dev-logging gate is the correct vehicle for the optional/gap logging findings
  elsewhere (PA-LOG-003 document-model, PA-LOG-005 workflow checkpoint, and any
  instrumentation of the 56 zero-log crates from PA-LOG-002): TRACE/DEBUG added
  under `dev-logging` aids testing/debugging with zero release cost. Recorded as
  PA-CR058 in the register; those findings should be re-scoped to "add via the
  CR-NR-058 dev-logging convention".
