---
inclusion: always
---

# Testing -- TDD, Commands, and Coverage

TDD is non-negotiable. Every piece of implementation code must be preceded by a
failing test.

## TDD Cycle -- for every acceptance criterion

```
1. READ    the acceptance criterion from requirements.md
2. WRITE   the test that exercises that criterion
3. RUN     cargo test -- confirm it FAILS (red)
            -> if it passes before implementation, the test is wrong; fix it
4. WRITE   the minimum implementation to pass
5. RUN     cargo test -- confirm it PASSES (green)
6. REFACTOR if needed, keeping all tests green
7. REPEAT
```

No implementation line may be written before steps 2 and 3 are complete.

## GUI Behaviour Testing -- egui_kittest (MANDATORY)

The TDD cycle applies to GUI behaviour too. `egui_kittest` renders a real panel
(or the whole shell) headlessly, injects input, and inspects focus/state under
`cargo test` / `cargo nextest`. Prefer it over deferring to manual verification.

### The rule

WHENEVER an acceptance criterion is about RENDERED WIDGET BEHAVIOUR, it MUST have
an `egui_kittest` harness test (written first, red before green). This includes:

- keyboard focus and Tab / Shift+Tab order (which widget is focused after input);
- a widget being present / absent, enabled / disabled, or a Tab stop / not a Tab
  stop;
- keyboard interaction and activation (Enter / Space / arrows / Escape) and the
  action it triggers;
- text entry landing in the intended field;
- state that changes in response to input (toggles, selection, month navigation).

`MANUAL` (TCR square) is NOT the default for GUI criteria. It is a JUSTIFIED
EXCEPTION, allowed ONLY when the behaviour genuinely cannot be driven headlessly:

- pixel-exact appearance / colour / layout geometry (use snapshot tests only if
  a stable image baseline is warranted; otherwise assert the model, not pixels);
- OS-native dialogs (`rfd` file pickers), real OS windows, and true
  multi-viewport / detached-window behaviour;
- screen-reader / assistive-technology output that requires a real AT client;
- anything requiring a GPU/display the CI headless runner does not provide.

When a criterion is marked `MANUAL` in `docs/quality/TCR.md`, the row MUST state
WHY it cannot be harness-tested. "Manual UI verification" with no reason is not
acceptable for a new criterion.

### The whole shell is harness-able

`egui_kittest`'s `build_eframe` drives a real `eframe::App` headlessly, so the
entire `WorkbenchShell` can be tested end-to-end, not just isolated panels. The
`eframe` feature is enabled on the `egui_kittest` dev-dependency for this reason.

```rust
use egui_kittest::Harness;
let mut harness = Harness::builder()
    .with_size(egui::Vec2::new(1200.0, 900.0))
    .build_eframe(|_cc| make_shell());   // make_shell() -> WorkbenchShell
for _ in 0..4 { harness.run(); }         // settle one-shot startup
harness.press_key(egui::Key::Tab);       // or press_key_modifiers(Modifiers::SHIFT, Key::Tab)
harness.run();
let focused = harness.ctx.memory(|m| m.focused());   // assert focus/state
```

Isolated-panel harness (`build_ui` / `build_ui_state`) remains fine for pure
render functions (e.g. `render_menu_workspace`, `render_calendar`). Widgets that
must be reachable by a stable id (for focus assertions or the shell
Boundary_Policy) should be given a stable `egui::Id` at their render site.

### Determinism

- Run a fixed, bounded number of `harness.run()` frames; never loop until a
  timeout. Cap Tab-walk loops (e.g. <= 40 presses) and assert the expected
  outcome is reached within the budget.
- Assert on widget ids / focus / model state, not on frame counts or pixels.
- Under `nextest` each test is process-isolated, so `make_shell()` env-var
  config isolation (B048) holds; do not share a `Harness` across tests.

### Converting existing MANUAL rows

When you touch code behind an existing `MANUAL` TCR row whose criterion is
actually harness-able, add the `egui_kittest` test and flip the row to `PASS`.
Do not leave harness-able behaviour as manual once you are already editing it.

## Test Organisation

- Unit tests: `#[cfg(test)] mod tests { ... }` at the bottom of the source file.
- Integration tests: `tests/` at crate root, one file per feature area.
- Property tests: `proptest`, minimum 100 iterations, with a comment:
  `// Feature: <sub-project>, Property N: <property statement>`

## Test Naming

Describe scenario and outcome as a sentence:
```rust
// Good
fn scroll_past_last_line_clamps_top_line_to_last_page() { ... }
// Bad
fn test_scroll() { ... }
```

## Test Quality

- Deterministic -- no `HashMap` iteration order, system time, or external files
  outside `tests/fixtures/`.
- Each test asserts one primary behaviour.
- `pretty_assertions::assert_eq!` for diff-friendly output.
- `tempfile::TempDir` for any test that writes to disk.
- Never `#[ignore]` a failing test -- fix or delete it.

## Requirement Coverage Annotation

Every test links to its criterion:
```rust
#[test]
fn scroll_down_clamps_at_last_line() {
    // Validates: Requirement 2.4 -- Down Arrow advances top_line by 1, clamped
    ...
}
```

## Test Coverage Report (TCR)

`docs/quality/TCR.md` is the authoritative record of test status.

| Status | Symbol | Meaning |
|--------|--------|---------|
| PASS | ✅ | Automated test exists and passes |
| FAIL | ❌ | Automated test exists but fails |
| MANUAL | 🔲 | Requires manual/UI verification -- allowed only as a JUSTIFIED exception (see "GUI Behaviour Testing"); the row MUST state why it cannot be harness-tested |
| NOT COVERED | 🔴 | No test exists yet |

TCR is append-only for criteria rows -- never remove a row, only update status
in place. For GUI criteria, prefer `egui_kittest` (PASS) over MANUAL; MANUAL is
reserved for the exception list in the "GUI Behaviour Testing" section.

## Test Checklist Verification

Before claiming a task complete:
1. Read `.kiro/test-checklist.json`.
2. Check `status` of every `TestEntry` linked to the task's requirements.
3. All must be **PASS** or **MANUAL_PASS**.
4. Report any **FAIL**, **NEEDS_RETEST**, or **NOT_COVERED** before claiming completion.

---

## Build and Test Commands

Use these exact commands for all build, test, and quality operations.

### Testing
```bash
cargo test                          # run all tests
cargo test test_name_here           # run a specific test
cargo test -- --nocapture           # run with output visible
cargo test --test phase3_edit       # run a specific integration test file
```

### Scoped testing (preferred during active development)
```bash
cargo test -p ff-desktop -p ff-theme        # multiple crates
cargo test -p ff-desktop                    # single crate
cargo test -p ff-desktop scroll_down_clamps # single crate, specific test
cargo test -p ff-desktop -- --nocapture     # single crate, show stdout
```

### Full-workspace runs are the owner's, not Kiro's (non-blocking by design)
Kiro does NOT run full-workspace builds/tests. The full gate (`cargo gate
--build`) and any `--workspace` run are the OWNER's manual step, run outside Kiro,
so Kiro is never blocked on a multi-minute run and the connection never drops
mid-wait. The
command below is documented for the OWNER's reference (or a future explicit "run
this in the background and read the log" instruction from the owner) -- it is not
part of Kiro's normal loop:
```bat
REM OWNER-run reference only; Kiro uses scoped -p checks instead.
start /B cargo nextest run --workspace > tools\logs\test-run.txt 2>&1
type tools\logs\test-run.txt
```

### Scoped test map
| Work area | Scoped command |
|-----------|---------------|
| Accessibility (ff-theme contrast) | `cargo test -p ff-theme -p ff-desktop` |
| Plugin Manager UI | `cargo test -p ff-desktop -p ff-plugin` |
| Notification System | `cargo test -p ff-desktop` |
| Compiler Toolchain (MockToolchain) | `cargo test -p ff-toolchain-api` |
| Full baseline check | `cargo gate --build` -- OWNER-run manual gate, not a Kiro command |

### Full-workspace verification -- cargo gate --build (CANONICAL)
The canonical gate is **`cargo gate --build`** (the `cargo-gate` tool, a cargo
subcommand). The repo also ships the alias **`cargo full-gate`** (= `cargo gate
--build`, defined in `.cargo/config.toml`) so the canonical form is one command
that always includes `--build`. It runs `cargo fmt --check`, `cargo clippy
--workspace`, and the test suite (`cargo nextest run`) EXACTLY ONCE each, with one
live progress view.

The `--build` flag adds a BUILD phase that runs AFTER a clean gate, so you are
left with a FRESH, RUNNABLE `ffwb.exe` in `target/` (fmt/clippy only
compile-check and nextest builds TEST binaries, so WITHOUT `--build` a clean gate
can still leave a STALE app binary -- that was the B082 "clean gate but
old-looking app" trap). Details of the flag (from `cargo gate --help`):
- `--build` is OFF by default (the default run stays a pure, side-effect-free
  verification gate); always pass it (or use `cargo full-gate`) for a completion
  gate so the binary is refreshed.
- The build phase is SKIPPED automatically if the gate is not clean (no point
  building a failing tree).
- Bare `--build` builds the active scope; `--build <bin>` builds `-p <bin>`.
- `--release` composes with `--build` for a release binary (slower -- a release
  profile shares no artifacts with the test build).

It writes its artifacts to `.gate/` (overridable with `--log-dir`): a combined
`.gate/gate.report.log` (signal only -- group/summary successes, warnings, errors;
per-test `ok`/`PASS` and blank lines filtered out), a problems file
`.gate/gate.review.log` (an EMPTY file means the gate is CLEAN -- this is the
"done" signal), plus `.gate/gate.history.csv` (one row per step with duration /
status / test counts) and `.gate/gate.timing.log`. `.gate/` is git-ignored
(ephemeral).

Tests run via `cargo-nextest`: it executes every test binary across all cores in
parallel and prints one aggregated summary, much faster than serial `cargo test`
on this ~9000-test / 69-crate workspace. Install nextest once with:
`cargo install --locked cargo-nextest`.

> FALLBACK: `tools\ffwb-gate.ps1` (and its `tools\ffwb-gate.sh` sibling) is the
> DEPRECATED predecessor, retained only as a fallback if `cargo gate` is
> unavailable. It writes to `tools\logs\ffwb-gate.*` instead of `.gate/` and has
> NO build step. Prefer `cargo gate --build`; reach for the script only when
> `cargo gate` cannot be run.

### Gate scopes
`cargo gate` supports the same scoping the old script did. `cargo fmt --check`,
`cargo clippy --workspace`, and `cargo build` run in ALL scopes; only the test
step's package set changes. Pick the scope by workload:

```powershell
# COMPLETION GATE (default): full --workspace, all crates, builds the binary.
# The ONLY scope that qualifies as "done". ~4-5 min.
cargo gate --build

# ROUTINE gate: the ffwb app dependency-closure only (excludes workspace orphan
# crates NOT wired into ff-desktop). Faster. PARTIAL -- not the completion gate.
cargo gate --build --app-only --closure-of ff-desktop

# INNER LOOP: a single crate (cargo nextest run -p <name>). PARTIAL.
cargo gate --crate ff-keys

# Fast proptest signal (PROPTEST_CASES=32). Orthogonal -- composes with any scope.
cargo gate --build --fast
```

The `--app-only` closure is DERIVED at runtime from `--closure-of <crate>` -- all
workspace members MINUS that crate's dependency closure -- so it NEVER drifts: a
crate wired into ff-desktop automatically re-enters the set, and no shipping crate
is silently skipped. The chosen scope is recorded per run in
`.gate/gate.history.csv`.

Rules:
- **The full gate (`cargo gate --build`) is the OWNER's MANUAL step, NOT a Kiro
  command.** Kiro NEVER runs `cargo gate` or any `--workspace` build/test -- those
  are the multi-minute runs that block Kiro and drop the connection. Kiro runs
  ONLY the scoped `-p <crate>` checks for what it changed, then hands off (see
  "Full-gate hand-off" below). The owner runs the full gate outside Kiro and
  reports the result back.
- **Completion is two-staged.** Kiro certifies "code-complete pending full gate"
  when its scoped checks are clean; the task is "done" only after the owner runs
  the full `cargo gate --build` and confirms a clean run (empty
  `.gate/gate.review.log`). A clean scoped run is NOT sufficient to claim "done" --
  but it IS all Kiro runs.
- The full run still matters for the not-yet-integrated ("orphan") crates and for
  proptests keeping their mandated >=100 iterations; that is precisely why it is
  run manually by the owner rather than skipped.

### Full-gate hand-off (Kiro <-> owner protocol)
This replaces Kiro ever running the full gate:
1. Kiro finishes a task's scoped checks (`cargo check/test/clippy -p <crate>`,
   `cargo fmt`) and confirms they are clean.
2. Kiro STOPS and prints the hand-off: which scoped commands it ran, and the
   exact full-gate command for the owner to run outside Kiro:
   `cargo gate --build`
   (fallback if cargo gate is unavailable: `pwsh -ExecutionPolicy Bypass -File
   tools\ffwb-gate.ps1`, or `./tools/ffwb-gate.sh` on Linux/macOS).
3. The OWNER runs the full gate manually and either replies "clean" or pastes the
   contents of `.gate\gate.review.log` / the failing output.
4. Kiro acts on that feedback: if clean, the task is DONE; if failures, Kiro fixes
   them (scoped checks only) and hands off again at step 1.
Kiro must NOT proceed to declare a task/phase/CR complete until step 3 returns
clean. Kiro must NOT run `cargo gate` itself to "save a round-trip".

Direct nextest use (outside the script) is also available:
```bash
cargo nextest run --workspace       # all crates, parallel, one summary
cargo nextest run -p ff-desktop     # scoped
```

### Building
```bash
cargo check                         # compile check, no binary
cargo build                         # debug build
cargo build --release               # release build
```

### Running
```bash
.\target\debug\file_forge_workbench.exe path\to\file.txt
.\target\release\file_forge_workbench.exe path\to\file.txt
```

### Code quality
```bash
cargo clippy -- -D warnings         # lint, all warnings as errors
cargo fmt                           # format code
cargo fmt -- --check                # check formatting without changing files
rg "\.unwrap\(\)|\.expect\(" crates/ --glob "!**/tests/**"   # unwrap in lib code
```

### Command-level TDD sequence
```
1. cargo test -p <crate>         # confirm scoped baseline green (fast)
2. [write the failing test]
3. cargo test -p <crate>         # confirm NEW test fails (red)
4. [write minimum implementation]
5. cargo test -p <crate>         # confirm test passes (green)
6. cargo clippy -p <crate>       # no new lint violations, scoped
7. cargo fmt                     # format before committing
8. HAND OFF -> owner runs the full gate manually (see below)
```
Never skip step 3. Steps 1-7 are ALL Kiro runs: SCOPED to the crate(s) touched,
so every command returns in seconds, not minutes. Step 8 is NOT a Kiro command:
Kiro stops after the scoped checks are clean and PROMPTS the owner to run the
full gate outside Kiro. See "Full-gate hand-off" below.