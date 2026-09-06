# Build and Test Commands

Use these exact commands for all build, test, and quality operations.

## Testing

```bash
cargo test                          # run all tests
cargo test test_name_here           # run a specific test
cargo test -- --nocapture           # run with output visible
cargo test --test phase3_edit       # run a specific integration test file
```

### Scoped Testing (preferred during active development)

Run only the crates relevant to current work -- much faster than full workspace:

```bash
# Phase CO work (ff-desktop, ff-theme)
cargo test -p ff-desktop -p ff-theme

# Single crate
cargo test -p ff-desktop

# Single crate, specific test name
cargo test -p ff-desktop scroll_down_clamps

# Single crate, show stdout
cargo test -p ff-desktop -- --nocapture
```

### Background Full-Workspace Test (non-blocking)

Fire the full workspace run in the background and check the log when ready.
Do NOT pipe through `tail` -- it suppresses all output until the process exits.

**Windows (cmd.exe):**
```bat
start /B cargo test --workspace > tools\logs\test-run.txt 2>&1
REM ... do other work ...
type tools\logs\test-run.txt
```

**Check if still running:**
```bat
tasklist | findstr cargo
```

**Read last 20 lines of log:**
```bat
powershell "Get-Content tools\logs\test-run.txt -Tail 20"
```

### Scoped Test Map (Phase CO)

| Work area | Scoped command |
|-----------|---------------|
| Accessibility (ff-theme contrast) | `cargo test -p ff-theme -p ff-desktop` |
| Plugin Manager UI | `cargo test -p ff-desktop -p ff-plugin` |
| Notification System | `cargo test -p ff-desktop` |
| Compiler Toolchain (MockToolchain) | `cargo test -p ff-toolchain-api` |
| Full baseline check | background task -- see above |

## Building

```bash
cargo check                         # check for compile errors (no binary)
cargo build                         # debug build
cargo build --release               # release build
```

## Running

```bash
.\target\debug\file_forge_workbench.exe path\to\file.txt
.\target\release\file_forge_workbench.exe path\to\file.txt
```

## Code Quality

```bash
cargo clippy -- -D warnings         # lint — all warnings as errors
cargo fmt                           # format code
cargo fmt -- --check                # check formatting without changing files
rg "\.unwrap\(\)|\.expect\(" crates/ --glob "!**/tests/**"   # check for unwrap in library code
```

## Mandatory TDD Workflow Sequence

```
1. cargo test -p <crate>         # confirm scoped baseline is green (fast)
2. [write the failing test]
3. cargo test -p <crate>         # confirm NEW test fails (red)
4. [write minimum implementation]
5. cargo test -p <crate>         # confirm test passes (green)
6. cargo clippy -- -D warnings   # no new lint violations
7. cargo fmt                     # format before committing
8. [background] cargo test --workspace > tools\logs\test-run.txt 2>&1
```

Never skip step 3. A test that passes before implementation is not testing anything.

Step 8 (full workspace) runs in the background while you continue. Check the log
before committing or declaring a phase complete.
