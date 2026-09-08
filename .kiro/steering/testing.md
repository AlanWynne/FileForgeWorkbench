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
| MANUAL | 🔲 | Requires manual/UI verification |
| NOT COVERED | 🔴 | No test exists yet |

TCR is append-only for criteria rows -- never remove a row, only update status
in place.

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
Fire the full run in the background and read the log later. Do NOT pipe through
`tail` -- it suppresses output until the process exits.
```bat
start /B cargo test --workspace > tools\logs\test-run.txt 2>&1
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
| Full baseline check | background task (above) |

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
8. [background] cargo test --workspace > tools\logs\test-run.txt 2>&1
```
Never skip step 3. Step 8 runs in the background; check the log before committing
or declaring a phase complete.