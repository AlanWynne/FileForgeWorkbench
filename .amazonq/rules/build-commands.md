# Build and Test Commands

Use these exact commands for all build, test, and quality operations.

## Testing

```bash
cargo test                          # run all tests
cargo test test_name_here           # run a specific test
cargo test -- --nocapture           # run with output visible
cargo test --test phase3_edit       # run a specific integration test file
```

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
1. cargo test                    # confirm baseline is green
2. [write the failing test]
3. cargo test                    # confirm NEW test fails (red)
4. [write minimum implementation]
5. cargo test                    # confirm test passes (green)
6. cargo clippy -- -D warnings   # no new lint violations
7. cargo fmt                     # format before committing
```

Never skip step 3. A test that passes before implementation is not testing anything.
