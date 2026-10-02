# Project Tools

This folder contains reusable scripts that support FileForge Workbench
development, documentation, validation, and maintenance. These tools are not
part of the product runtime.

## Organization

| Folder | Purpose |
|--------|---------|
| `python/` | Reusable Python utilities |
| `powershell/` | Reusable PowerShell utilities |
| `rust/` | Small Rust-based maintenance tools, if needed |
| `fixtures/` | Small input files required by a tool or validation |

Current reusable utilities:

- [`powershell/ffwb_make.ps1`](powershell/ffwb_make.ps1) - common build and
  test commands.
- [`powershell/push-to-github.ps1`](powershell/push-to-github.ps1) - explicitly
  confirmed commit, tag, and push workflow.

Each reusable tool should include:

- A descriptive filename.
- A short usage comment or help message.
- Explicit input and output paths.
- Safe default behavior that does not delete or overwrite data unexpectedly.
- A note in this README or a nearby README when the tool needs special setup.

## Available tools

### `ffwb-gate.ps1` / `ffwb-gate.sh` -- Unified verification gate

THE single verification gate. It replaces the former `allcargo.bat` driver and
`powershell/verify.ps1` (both now deleted), which duplicated each other's work --
`allcargo.bat` ran the whole suite and THEN called `verify.ps1`, recompiling and
re-running every test a second time. `ffwb-gate` runs each phase EXACTLY ONCE:
`cargo fmt --check`, `cargo clippy --workspace`, and the test suite (cargo-nextest
when installed, otherwise `cargo test`).

It writes ONE combined log of every message EXCEPT the noise: per-test success
lines (`<name> ... ok` from cargo test and the `        PASS [   ...]` lines from
cargo nextest) and blank lines are filtered out, so only section headers,
group/summary successes (e.g. `test result: ok. N passed`, the nextest `Summary`
line, `Compiling`/`Finished`), warnings, and errors remain.

It also carries the full feature set of the old `verify.ps1`: the `-AppOnly`,
`-Crate`, and `-Fast` scopes, a live single-file progress snapshot, an appended
run-history CSV, a median-based test-phase ETA, and a per-step watchdog that
terminates a hung step.

The repo root is derived from the script's own location, so these are fully
portable: pull the repo on any machine and run them unchanged.

- `ffwb-gate.ps1` -- Windows, Linux, and macOS via PowerShell (pwsh):

  ```
  pwsh -ExecutionPolicy Bypass -File tools/ffwb-gate.ps1
  pwsh -ExecutionPolicy Bypass -File tools/ffwb-gate.ps1 -Fast
  pwsh -ExecutionPolicy Bypass -File tools/ffwb-gate.ps1 -AppOnly
  pwsh -ExecutionPolicy Bypass -File tools/ffwb-gate.ps1 -Crate ff-keys
  ```

- `ffwb-gate.sh` -- Linux and macOS via bash:

  ```
  ./tools/ffwb-gate.sh
  ./tools/ffwb-gate.sh --fast
  ./tools/ffwb-gate.sh --app-only
  ./tools/ffwb-gate.sh --crate ff-keys
  ```

Outputs (under `tools/logs/`, git-ignored):

- `gate.combined.log` -- all messages, per-test pass/ok and blank lines filtered out.
- `ai-review.log` -- only errors/warnings/failures (empty means a clean gate).
- `cargo.*.log` -- the raw, unfiltered per-step output.
- `verify.history.csv` -- one appended row per run (survives the `*.log` cleanup).
- `verify.timing.log` / `verify.diag.log` -- per-run timing and phase/watchdog diagnostics.
- `verify.progress.txt` (PowerShell) -- live single-file progress snapshot during a run.

`-Fast` / `--fast` sets `PROPTEST_CASES=32` for a quick developer signal and is
NOT the full completion gate; nor are `-AppOnly`/`--app-only` or
`-Crate`/`--crate`. Only the default full `--workspace` run is the completion gate.

### `python/logging_inventory.py` -- Logging inventory and gap report

Read-only static scan of every `*.rs` under `crates/`. Regenerates a Markdown
report of every `ff-logging` call site (grouped by crate and level) plus a gap
report (crates with no logging, and silent-error review candidates such as
`let _ =`, `.ok()`, `unwrap()`, `expect()` in non-test code). Implements
logging-subsystem Requirement 12 (CR-NR-055).

Run:

```
C:\tools\python\python.exe tools\python\logging_inventory.py
```

- Report (tracked): `docs/quality/logging-inventory.md` (overwritten each run)
- Run log (ephemeral): `tools/logs/logging-inventory.txt`
- The tool mutates no source and writes nothing outside those two paths. It is
  deterministic -- re-runs differ only in the generation timestamp -- so it is
  safe to run periodically to track logging coverage over time.

## Temporary scripts

Do not place one-off experiments or failed patches here. Use the session
workspace or another explicitly temporary location for those files. A script
may be promoted into this folder when it is useful for a second task, has
clear ownership and usage, and is safe to rerun.

## Runtime prerequisites

Use the repository's documented commands first. When Python is required, the
standard local interpreter is `C:\tools\python`. Shared scripts in
`C:\tools\scripts` may be used, but project-specific reusable tools belong in
this folder so they are versioned with the project.
