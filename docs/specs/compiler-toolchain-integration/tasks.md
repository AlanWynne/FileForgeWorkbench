# Tasks -- Compiler Toolchain Integration

## Task 1. Create `ff-toolchain-api` shared abstractions crate

- [x] 1.1 Scaffold `crates/ff-toolchain-api/` with `Cargo.toml` and `src/lib.rs`
  - Satisfies: Req 1.1, 2.3, 3.1, 4.3 (shared types used by both plugins)
- [x] 1.2 Define `ToolchainState` enum: `NotDetected`, `Detected`, `Installing`, `InstallFailed`, `Ready`
  - Satisfies: Req 1.2, 1.3, 3.2, 3.3
- [x] 1.3 Define `Diagnostic` struct: `file`, `line`, `column`, `severity`, `message`
  - Satisfies: Req 2.3, 4.3
- [x] 1.4 Define `DiagnosticSeverity` enum: `Error`, `Warning`, `Note`
  - Satisfies: Req 2.3, 4.3
- [x] 1.5 Define `BuildProfile` struct: `name`, `flags: Vec<String>`
  - Satisfies: Req 2.6
- [x] 1.6 Define `BuildEvent` enum: `OutputLine(String)`, `Diagnostic(Diagnostic)`, `Finished(i32)`
  - Satisfies: Req 2.2, 4.2
- [x] 1.7 Define `InstallProgress` enum: `Started`, `Progress { message: String }`, `Completed`, `Failed { reason: String }`
  - Satisfies: Req 1.5, 3.5
- [x] 1.8 Define `ToolchainPlugin` trait with `name()`, `state()`, `detect()`, `install()`, `build()`
  - Satisfies: Req 1.1, 3.1
- [x] 1.9 Write unit tests for `ToolchainState` transitions and `Diagnostic` construction
  - Satisfies: Req 1.2, 1.3

## Task 2. Create `ff-gcc-toolchain` plugin crate

- [x] 2.1 Scaffold `crates/ff-gcc-toolchain/` with `Cargo.toml` (deps: `ff-toolchain-api`, `regex`, `which`)
  - Satisfies: Req 1 (all)
- [x] 2.2 Implement `GccToolchainPlugin` struct with `ToolchainState` field
  - Satisfies: Req 1.1
- [x] 2.3 Implement `detect()` -- probe PATH for `gcc`, `g++`, `gfortran`, `as`, `ld`, `ar` via `which`
  - Satisfies: Req 1.1, 1.2, 1.3, 1.9
- [x] 2.4 Implement platform detection (`std::env::consts::OS`) and install strategy selection
  - Satisfies: Req 1.8
- [x] 2.5 Implement `install()` for Windows (winget -> MSYS2 fallback)
  - Satisfies: Req 1.4, 1.5, 1.6, 1.7, 1.8
- [x] 2.6 Implement `install()` for Linux (apt -> dnf fallback)
  - Satisfies: Req 1.4, 1.5, 1.6, 1.7, 1.8
- [x] 2.7 Implement `install()` for macOS (Homebrew)
  - Satisfies: Req 1.4, 1.5, 1.6, 1.7, 1.8
- [x] 2.8 Implement `build()` -- invoke `gcc`/`g++` with `BuildProfile` flags, stream output
  - Satisfies: Req 2.1, 2.2
- [x] 2.9 Implement GCC diagnostic parser (regex: `file:line:col: severity: message`)
  - Satisfies: Req 2.3
- [x] 2.10 Implement built-in `BuildProfile` constants: `debug`, `release`, `check-only`
  - Satisfies: Req 2.6
- [x] 2.11 Implement `plugin_init()` entry point registering `GccToolchainPlugin`
  - Satisfies: Req 1 (plugin registration)
- [x] 2.12 Write unit tests: detection with mock PATH, diagnostic parser, build profile flags
  - Satisfies: Req 1.1, 1.2, 1.3, 2.3, 2.6

## Task 3. Create `ff-rust-toolchain` plugin crate

- [x] 3.1 Scaffold `crates/ff-rust-toolchain/` with `Cargo.toml` (deps: `ff-toolchain-api`, `serde_json`, `which`)
  - Satisfies: Req 3 (all)
- [x] 3.2 Implement `RustToolchainPlugin` struct with `ToolchainState` field
  - Satisfies: Req 3.1
- [x] 3.3 Implement `detect()` -- probe PATH for `rustc`, `cargo`, `rustup`; read active channel
  - Satisfies: Req 3.1, 3.2, 3.3, 3.9
- [x] 3.4 Implement `install()` -- download and execute `rustup-init` (platform-appropriate method)
  - Satisfies: Req 3.4, 3.5, 3.6, 3.7
- [x] 3.5 Implement PATH extension after successful install (`~/.cargo/bin` / `%USERPROFILE%\.cargo\bin`)
  - Satisfies: Req 3.6
- [x] 3.6 Implement `update()` -- run `rustup update` in background
  - Satisfies: Req 3.8
- [x] 3.7 Implement `build()` -- invoke `cargo <subcommand> --message-format=json`, stream output
  - Satisfies: Req 4.1, 4.2, 4.7
- [x] 3.8 Implement Cargo.toml discovery (walk up directory tree from active file)
  - Satisfies: Req 4.1
- [x] 3.9 Implement JSON diagnostic parser (`serde_json`, extract `compiler-message` objects)
  - Satisfies: Req 4.3
- [x] 3.10 Implement `plugin_init()` entry point registering `RustToolchainPlugin`
  - Satisfies: Req 3 (plugin registration)
- [x] 3.11 Write unit tests: detection with mock PATH, JSON diagnostic parser, Cargo.toml discovery
  - Satisfies: Req 3.1, 3.2, 3.3, 4.1, 4.3

## Task 4. Toolchain_Panel UI in `ff-desktop`

- [x] 4.1 Create `crates/ff-desktop/src/toolchain_panel.rs` -- egui panel rendering toolchain status rows
  - Satisfies: Req 1.2, 1.3, 1.9, 3.2, 3.3, 3.9
- [x] 4.2 Render `[Install GCC]` / `[Install via rustup]` buttons when `NotDetected`
  - Satisfies: Req 1.3, 3.3
- [x] 4.3 Render installation progress indicator when `Installing`
  - Satisfies: Req 1.5, 3.5
- [x] 4.4 Render build output scrollable text area
  - Satisfies: Req 2.2, 4.2
- [x] 4.5 Render clickable diagnostics list; on click navigate editor to file/line/col
  - Satisfies: Req 2.7, 4.6
- [x] 4.6 Wire `Compilers` menu option `3` from Primary Option Menu to open Toolchain_Panel
  - Satisfies: Req 4.6 (option 3 = Compilers)
- [x] 4.7 Write unit tests for panel state rendering logic (status rows, button visibility)
  - Satisfies: Req 1.2, 1.3, 3.2, 3.3

## Task 5. Validate generic ToolchainPlugin trait contract (Req 5)

- [x] 5.1 Audit `ff-toolchain-api` trait definition -- confirm no GCC/Rust-specific
  assumptions exist in `ToolchainPlugin`; add doc comment citing Req 5.2
  - Satisfies: Req 5.1, 5.2
- [x] 5.2 Write a `MockToolchain` test double in `ff-toolchain-api` tests that implements
  `ToolchainPlugin` with no dependency on `ff-gcc-toolchain` or `ff-rust-toolchain`;
  confirm it compiles and registers correctly
  - Satisfies: Req 5.3, 5.4
- [x] 5.3 Confirm `ff-toolchain-api` Cargo.toml has no dev-dependency on either plugin crate;
  add a CI comment noting this constraint
  - Satisfies: Req 5.4
