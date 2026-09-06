# Tasks -- Batch Command Execution

## Phase CP

### Task 1. Requirements gate
- [x] 1.1 Write `docs/specs/batch-execution/requirements.md` (Req 1-10)
- [x] 1.2 Write `docs/specs/batch-execution/design.md`
- [x] 1.3 Write `docs/specs/batch-execution/tasks.md` (this file)
- [x] 1.4 Add Phase CP to `docs/specs/project-master/tasks.md`
- [x] 1.5 Add NOT COVERED rows to `docs/quality/TCR.md` for all new criteria
- [x] 1.6 Add CR-NR-041 to `docs/status/change-log.md`
- [x] 1.7 Add `batch-execution` to `.amazonq/rules/specs.md` sub-project list

### Task 2. BatchInputSource (Req 1, 2)
- [x] 2.1 Create `crates/ff-desktop/src/batch/mod.rs` -- module declaration
  - Validates: Requirement 1.1, 1.2
- [x] 2.2 Create `crates/ff-desktop/src/batch/input.rs` -- `BatchInputSource`
  struct; `from_file(path)` and `from_stdin()` constructors; `next_command()`
  iterator that skips blanks and comments, handles `-` continuation
  - Validates: Requirement 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7
- [x] 2.3 Unit tests for `BatchInputSource`: blank skip, `*` comment, `/*`
  comment, continuation, BOM strip, line-length truncation warning
  - Validates: Requirement 2.1-2.7

### Task 3. BatchOutputSink (Req 4)
- [x] 3.1 Create `crates/ff-desktop/src/batch/output.rs` -- `BatchOutputSink`
  enum (Stdout, File, Append); `write_line()`, `write_command_echo()` methods
  - Validates: Requirement 4.1, 4.2, 4.3, 4.4, 4.5, 4.6
- [x] 3.2 Unit tests for `BatchOutputSink`: stdout routing, file creation,
  append mode, echo prefix format, stderr separation
  - Validates: Requirement 4.1-4.7

### Task 4. Return code model (Req 5, 6)
- [x] 4.1 Create `crates/ff-desktop/src/batch/return_code.rs` --
  `StepReturnCode` enum (Success=0, Warning=4, Error=8, Severe=12,
  Catastrophic=16); `BatchReturnCode` (max accumulator); `AbortPolicy`
  - Validates: Requirement 5.1, 5.2, 5.3, 5.4, 5.5
- [x] 4.2 Unit tests for return code accumulation and abort policy
  - Validates: Requirement 5.1-5.5, 6.1-6.5

### Task 5. BatchSession (Req 7)
- [x] 5.1 Create `crates/ff-desktop/src/batch/session.rs` -- `BatchSession`
  struct; loads config layers and catalog registry from standard paths;
  provides headless command context; does NOT restore GUI session state
  - Validates: Requirement 7.1, 7.2, 7.4, 7.5
- [x] 5.2 Add `--batch-profile` support to `BatchSession::new()`
  - Validates: Requirement 7.3
- [x] 5.3 Add `--batch-no-catalog` flag support
  - Validates: Requirement 7.6
- [x] 5.4 Unit tests for `BatchSession` initialisation, profile loading,
  no-catalog mode, non-destructive exit
  - Validates: Requirement 7.1-7.6

### Task 6. BatchRunner orchestration (Req 1, 3, 5, 6, 10)
- [x] 6.1 Create `crates/ff-desktop/src/batch/runner.rs` -- `BatchRunner`
  struct; `run(input, output, session, opts)` method; command loop;
  Step_Return_Code collection; abort policy check; final summary line
  - Validates: Requirement 1.1, 3.1, 3.2, 3.3, 3.6, 5.1-5.5, 6.1-6.4
- [x] 6.2 Wire `RequiresInteractiveInput` error from command pipeline to
  Step_Return_Code 8 with diagnostic message
  - Validates: Requirement 3.5
- [x] 6.3 Add `CONTROL ERRORS CANCEL` / `CONTROL ERRORS NOCANCEL` inline
  command recognition
  - Validates: Requirement 6.5
- [x] 6.4 Unit tests for `BatchRunner`: full run, abort-on-error, interactive
  command rejection, return code accumulation
  - Validates: Requirement 3.1-3.6, 5.1-5.5, 6.1-6.5

### Task 7. CLI entry point wiring (Req 1)
- [x] 7.1 Extend `main.rs` argument parsing to detect `--batch <file>`,
  `--batch -`, `--batch-output`, `--batch-output-append`, `--batch-echo`,
  `--batch-abort-on-error`, `--batch-dry-run`, `--batch-profile`,
  `--batch-no-catalog`, `--batch-log`
  - Validates: Requirement 1.1, 1.2, 1.3, 1.4, 1.5
- [x] 7.2 Branch in `main.rs`: if `--batch` present, call
  `BatchRunner::run()` and `std::process::exit(batch_return_code)`;
  otherwise proceed to `eframe::run_native` as before
  - Validates: Requirement 1.1, 1.4
- [x] 7.3 Reject `--batch` combined with file path arguments with error
  message and exit code 12
  - Validates: Requirement 1.3
- [x] 7.4 Unit tests for CLI argument parsing and branch selection
  - Validates: Requirement 1.1-1.6

### Task 8. Dry-run mode (Req 8)
- [x] 8.1 Add `dry_run: bool` to `BatchRunner` options; when true, skip
  execution of state-modifying commands; execute read-only commands normally
  - Validates: Requirement 8.1, 8.4
- [x] 8.2 Write `[DRY-RUN] <command> -> OK|ERROR: reason` lines to
  Batch_Output_Sink for each command
  - Validates: Requirement 8.2
- [x] 8.3 Dry-run return code reflects validation results (0 or 8)
  - Validates: Requirement 8.3
- [x] 8.4 Unit tests for dry-run mode
  - Validates: Requirement 8.1-8.4

### Task 9. FFCMD compatibility (Req 9)
- [x] 9.1 Confirm `BatchInputSource` parser handles `.ffcmd` format
  identically (same comment syntax, same continuation); add test with a
  `.ffcmd` fixture file
  - Validates: Requirement 9.1, 9.2, 9.3
- [x] 9.2 Confirm `BatchRunner` does NOT invoke the Lua engine for `.ffcmd`
  files; add test asserting Lua engine is not called
  - Validates: Requirement 9.4

### Task 10. Logging (Req 10)
- [x] 10.1 Add structured log events to `BatchRunner`: batch start, each
  command + Step_Return_Code + duration, final Batch_Return_Code
  - Validates: Requirement 10.1, 10.3, 10.4, 10.5
- [ ] 10.2 Add `--batch-log <file>` support: redirect ff-logging output to
  specified file for the duration of the batch run
  - Validates: Requirement 10.2
- [x] 10.3 Unit tests for log event emission
  - Validates: Requirement 10.1-10.5

### Task 11. TCR and documentation update
- [x] 11.1 Update `docs/quality/TCR.md` -- mark implemented criteria as PASS
- [x] 11.2 Update `docs/specs/project-master/tasks.md` -- mark Phase CP tasks
- [x] 11.3 Run `cargo test --workspace` -- confirm 0 failures
- [x] 11.4 Run `cargo clippy -- -D warnings` -- confirm 0 warnings
