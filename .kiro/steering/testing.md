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

### Background full-workspace test (non-blocking)
Prefer `verify.ps1` (below) for the full gate. If you need a raw background run,
fire it and read the log later. Do NOT pipe through `tail` -- it suppresses
output until the process exits. Prefer nextest for the aggregated summary.
```bat
start /B cargo nextest run --workspace > tools\logs\test-run.txt 2>&1
REM fallback if nextest is not installed:
REM start /B cargo test --workspace > tools\logs\test-run.txt 2>&1
type tools\logs\test-run.txt
tasklist | findstr cargo                      REM check if still running
powershell "Get-Content tools\logs\test-run.txt -Tail 20"
```

### Scoped test map
| Work area | Scoped command |
|-----------|---------------|
| Accessibility (ff-theme contrast) | `cargo test -p ff-theme -p ff-desktop` |
| Plugin Manager UI | `cargo test -p ff-desktop -p ff-plugin` |
| Notification System | `cargo test -p ff-desktop` |
| Compiler Toolchain (MockToolchain) | `cargo test -p ff-toolchain-api` |
| Full baseline check | `verify.ps1` (see below) |

### Full-workspace verification -- verify.ps1 (cargo-nextest)
The canonical gate is `tools\powershell\verify.ps1`. It runs three steps --
`cargo fmt --check`, `cargo clippy --workspace`, and the test suite -- capturing
each step's output to `tools\logs\` and accumulating any errors/warnings into
`tools\logs\ai-review.log` (an empty file means the gate is clean).

Tests run via `cargo-nextest` when installed: it executes every test binary
across all cores in parallel and prints one aggregated summary, which is much
faster than serial `cargo test` on this ~9000-test / 69-crate workspace. If
nextest is absent, verify.ps1 falls back to `cargo test --workspace`
automatically -- no behaviour change, just slower.

```powershell
# Full gate (proptests use their configured >=100 iterations):
powershell -ExecutionPolicy Bypass -File tools\powershell\verify.ps1

# Fast developer inner-loop signal (PROPTEST_CASES=32 -- NOT the full gate):
powershell -ExecutionPolicy Bypass -File tools\powershell\verify.ps1 -Fast
```

Rules:
- The `-Fast` switch is for quick iteration only. Declaring a task complete or a
  phase done REQUIRES a clean full run (no `-Fast`), so proptests keep their
  mandated >=100-iteration coverage.
- After any run, read `tools\logs\ai-review.log` before claiming success. Empty
  == clean; any lines == fix and rerun.
- Install nextest once with: `cargo install --locked cargo-nextest`.

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
6. cargo clippy -- -D warnings   # no new lint violations
7. cargo fmt                     # format before committing
8. verify.ps1                    # full gate (nextest); check ai-review.log
```
Never skip step 3. Step 8 is the full verification gate (`verify.ps1`, or
`-Fast` for a quick inner-loop signal); check `ai-review.log` before committing
or declaring a phase complete.