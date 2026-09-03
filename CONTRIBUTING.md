# Contributing to FileForge Workbench

Thank you for your interest in contributing. This document covers everything
you need to get started.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building](#building)
- [Running](#running)
- [Testing](#testing)
- [Code Style](#code-style)
- [TDD Workflow](#tdd-workflow)
- [Submitting a Pull Request](#submitting-a-pull-request)
- [Reporting Bugs](#reporting-bugs)
- [Requesting Features](#requesting-features)
- [Licence](#licence)
- [Project Structure](#project-structure)

---

## Prerequisites

| Tool | Minimum version | Install |
|------|----------------|---------|
| Rust | 1.78 | https://rustup.rs |
| Cargo | ships with Rust | — |

No other system dependencies are required for a basic build. The egui/eframe
UI toolkit is fetched automatically by Cargo.

---

## Building

```bash
# Check for compile errors without producing a binary
cargo check

# Debug build
cargo build

# Release build
cargo build --release
```

The binary is written to `target/debug/ffwb.exe` (Windows) or
`target/debug/ffwb` (Linux/macOS).

---

## Running

```bash
# Launch empty
.\target\debug\ffwb.exe

# Open one or more files as tabs
.\target\debug\ffwb.exe path\to\file.txt
.\target\debug\ffwb.exe file1.txt file2.rs
```

---

## Testing

```bash
# Run all tests
cargo test

# Run tests for a single crate
cargo test --package ff-keys

# Run a specific test by name
cargo test scroll_past_last_line

# Run with stdout visible
cargo test -- --nocapture
```

All tests must pass before a PR is merged. The CI pipeline runs
`cargo test` and `cargo clippy -- -D warnings` on every push.

---

## Code Style

- Format with `rustfmt` before every commit: `cargo fmt`
- All clippy lints are hard errors: `cargo clippy -- -D warnings`
- No `unwrap()` or `expect()` in library code (tests may use `expect("reason")`)
- Full rules are in [`.amazonq/rules/rust-coding-standards.md`](.amazonq/rules/rust-coding-standards.md)

---

## TDD Workflow

This project follows strict Test-Driven Development. The mandatory sequence is:

```
1. cargo test                    # confirm baseline is green
2. Write the failing test
3. cargo test                    # confirm the NEW test fails (red)
4. Write minimum implementation
5. cargo test                    # confirm test passes (green)
6. cargo clippy -- -D warnings   # no new lint violations
7. cargo fmt                     # format before committing
```

Never skip step 3. A test that passes before implementation is not testing
anything. Full rules are in [`.amazonq/rules/tdd-and-testing.md`](.amazonq/rules/tdd-and-testing.md).

---

## Submitting a Pull Request

1. Fork the repository and create a branch from `main`:
   ```bash
   git checkout -b feature/my-feature
   ```
2. Make your changes following the TDD workflow above.
3. Ensure `cargo test` and `cargo clippy -- -D warnings` both pass cleanly.
4. Run `cargo fmt` to format all changed files.
5. Push your branch and open a Pull Request against `main`.
6. Fill in the PR template — link the relevant issue or requirement criterion.

PRs that break tests, introduce clippy warnings, or skip the TDD cycle will
not be merged.

---

## Reporting Bugs

Open a GitHub Issue using the **Bug Report** template. Please include:

- Steps to reproduce
- Expected behaviour
- Actual behaviour
- Operating system and Rust version (`rustc --version`)

Bugs are tracked in [`docs/status/bugs.md`](docs/status/bugs.md).

---

## Requesting Features

Open a GitHub Issue using the **Feature Request** template. New capabilities
go through the requirements gate documented in
[`.amazonq/rules/new-requirements-gate.md`](.amazonq/rules/new-requirements-gate.md)
before any code is written.

Feature requests are tracked in [`docs/status/change-log.md`](docs/status/change-log.md).

---

## Licence

By contributing to FileForge Workbench you agree that your contributions will
be licensed under the **Apache License, Version 2.0**. See [LICENSE](LICENSE)
for the full licence text.

---

## Project Structure

```
FileForgeWorkbench/
├── crates/              # 60+ library crates + ff-desktop binary
│   ├── ff-desktop/      # egui/eframe application shell (the runnable binary)
│   ├── ff-core/         # Platform lifecycle
│   ├── ff-keys/         # Key map, function key bindings, RETRIEVE
│   ├── ff-config/       # TOML configuration system
│   └── ...              # See README.md for the full crate wave table
├── docs/
│   ├── specs/           # Per-feature requirements, design, and task files
│   ├── bugs.md          # Bug register
│   ├── change-log.md    # New requirements and change requests
│   ├── TCR.md           # Test Coverage Report
│   └── manual-test-plan.md
├── .amazonq/rules/      # AI assistant steering rules (TDD, coding standards)
├── Cargo.toml           # Workspace manifest
└── README.md
```
