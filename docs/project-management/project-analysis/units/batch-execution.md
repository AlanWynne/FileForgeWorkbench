# Analysis Record: batch-execution (W5.6)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: NONE dedicated -- a FEATURE of the `ff-desktop` binary (CLI entry
  point + batch runner), reusing ff-command-semantics (parse/dispatch), ff-shell
  (output capture), ff-workflow (sequencing)
- **Spec files**: requirements.md (385 lines, 10 requirements), tasks.md
  (39 sub-tasks: 38 `[x]`, 1 `[ ]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy

N/A as a crate split -- batch-execution is a headless-mode feature ADDED to the
`ff-desktop` binary, not its own crate (design.md: "Batch execution adds a headless mode
to the ff-desktop binary"). The implementation is small + well-sized: `batch/cli.rs`
(168 nt), `batch/runner.rs` (189 nt), `fftest_cli.rs` (240 nt) -- all under the 400 cap.
No split. (It IS part of the broader ff-desktop shell-split concern PA-STD-042 / PA-W3.6,
but the batch files themselves are fine.)

---

## 2. Completeness -- PA-INCOMPLETE-015 (one honestly-tracked open task)

38 of 39 sub-tasks `[x]`; ONE open: task 10.2 "Add `--batch-log <file>` support:
redirect ff-logging output to <file>" (Validates Req 10.2). State verified:

- The `--batch-log <file>` FLAG is parsed + stored (cli.rs:43 help text, cli.rs:119-121
  arg handling, test cli.rs:303 `batch_log_flag_sets_log_file`).
- The remaining work is wiring the parsed path to actually REDIRECT the ff-logging sink
  to that file (the log-sink redirection is not yet connected).

This is an HONEST `[ ]` (not a false-positive-complete like database-tool W5.4 -- here
tracking correctly shows the gap). Recorded PA-INCOMPLETE-015 (LOW-MEDIUM): finish task
10.2 -- connect `--batch-log <file>` to an ff-logging file sink so headless runs write a
structured log file. Small, well-scoped; ties CR-NR-058 (the logging subsystem's file
sink). Code + test.

---

## 3. Cross-unit consistency

### Logging -- POSITIVE exemplar (BatchRunner actually logs)

Unlike the Wave-5 emulators/tools that log nothing (JES PA-LOG-041, idcams PA-LOG-042,
toolchain PA-LOG-044), `batch/runner.rs` GENUINELY USES ff-logging:
`log_info!("[batch] run started")`, abort-policy transitions, per-command RC + duration
(Req 10.1/10.3), and RC >= 8 at `log_error!` (Req 10.4). This is exactly the CR-NR-058
behaviour the project wants -- a headless mode that records its lifecycle. Positive
counter-example; recorded as a CLEAN/exemplar row. (The only logging gap is the OPEN
10.2 file-sink, PA-INCOMPLETE-015.)

### Reuse (no duplication)

Batch execution REUSES the existing command pipeline: ff-command-semantics for parse/
dispatch, ff-shell for output capture/routing, ff-workflow for optional multi-step
sequencing. It does not reimplement command execution -- clean layering (the headless
runner is a thin orchestration over the same engine the GUI uses). Consistent.

### VFS / raw-fs

The batch runner reads a command file + (per 10.2) will write a log file. Command-file
reading should use the same path the GUI file-ops use; the log sink is an ff-logging
concern (file sink), not a VFS document -- so raw file handling for the log sink is
acceptable (logging infra, like the logging-subsystem's own sinks). No raw-fs
document-store bypass (contrast JES PA-CONFLICT-015). Neutral/clean.

### Public surface

- CLI arg parsing (`parse_batch_args`), `BatchRunner`, abort policy (CANCEL/NOCANCEL),
  RC propagation -- ff-desktop-owned batch feature. No cross-unit duplication.

---

## 4. Logging audit

- `ff_logging` in batch/runner.rs: MULTIPLE (run start, abort policy, per-command RC +
  duration, RC>=8 at ERROR) -- GOOD.
- cli.rs / fftest_cli.rs: 0 direct log calls (arg parsing + orchestration -- acceptable;
  the runner does the lifecycle logging).
- The ONE gap is the `--batch-log` file sink (PA-INCOMPLETE-015 / task 10.2) -- i.e. the
  logging output has nowhere to be redirected in headless mode yet.

No dead ff-logging dep (it is actually used). No new PA-LOG item -- the logging gap here
IS the open task.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-015 (LOW-MEDIUM)**: complete task 10.2 -- wire `--batch-log <file>` to
  an ff-logging file sink (the flag is parsed; the redirect is missing). Small; ties
  CR-NR-058. Code + test. Then all 39 tasks close.
- No PA-SPLIT (feature in ff-desktop, files under cap). No PA-STD (batch files are
  ASCII-clean -- the ff-desktop shell ASCII items are tracked under PA-STD-042/PA-W3).
  No PA-LOG (runner logs; the gap is the open task). No PA-TCR gap (16 rows).

---

## Summary

batch-execution is a headless / non-interactive execution mode ADDED to the `ff-desktop`
binary (not a separate crate), cleanly LAYERED over the existing command pipeline
(ff-command-semantics parse/dispatch + ff-shell output capture + ff-workflow sequencing)
-- no duplication of command execution. Implementation is small + well-sized (batch/cli.rs
168, batch/runner.rs 189, fftest_cli.rs 240 -- all under cap) with decent coverage (16 TCR
rows). It is a POSITIVE LOGGING exemplar: BatchRunner genuinely uses ff-logging (run
start, abort policy, per-command RC + duration, RC>=8 at ERROR per Req 10.1/10.3/10.4) --
the CR-NR-058 behaviour the emulators/tools lack. The single finding is PA-INCOMPLETE-015
(LOW-MEDIUM): one HONESTLY-tracked open task (10.2) -- the `--batch-log <file>` flag is
parsed but not yet wired to an ff-logging file sink, so headless runs cannot redirect
their log to a file. Finish 10.2 (small) and the unit closes. No split, no orphan, no
raw-fs bypass, no false-positive-complete.
