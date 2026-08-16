# Tasks — Compiler Toolchain Integration

## Task 1. Create `ff-toolchain-api` shared abstractions crate

- [x] 1.1 Scaffold `crates/ff-toolchain-api/` with `Cargo.toml` and `src/lib.rs`
  - Satisfies: Req 15.1, 16.3, 17.1, 18.3 (shared types used by both plugins)
- [x] 1.2 Define `ToolchainState` enum: `NotDetected`, `Detected`, `Installing`, `InstallFailed`, `Ready`
  - Satisfies: Req 15.2, 15.3, 17.2, 17.3
- [x] 1.3 Define `Diagnostic` struct: `file`, `line`, `column`, `severity`, `message`
  - Satisfies: Req 16.3, 18.3
- [x] 1.4 Define `DiagnosticSeverity` enum: `Error`, `Warning`, `Note`
  - Satisfies: Req 16.3, 18.3
- [x] 1.5 Define `BuildProfile` struct: `name`, `flags: Vec<String>`
  - Satisfies: Req 16.6
- [x] 1.6 Define `BuildEvent` enum: `OutputLine(String)`, `Diagnostic(Diagnostic)`, `Finished(i32)`
  - Satisfies: Req 16.2, 18.2
- [x] 1.7 Define `InstallProgress` enum: `Started`, `Progress { message: String }`, `Completed`, `Failed { reason: String }`
  - Satisfies: Req 15.5, 17.5
- [x] 1.8 Define `ToolchainPlugin` trait with `name()`, `state()`, `detect()`, `install()`, `build()`
  - Satisfies: Req 15.1, 17.1
- [x] 1.9 Write unit tests for `ToolchainState` transitions and `Diagnostic` construction
  - Satisfies: Req 15.2, 15.3

## Task 2. Create `ff-gcc-toolchain` plugin crate

- [x] 2.1 Scaffold `crates/ff-gcc-toolchain/` with `Cargo.toml` (deps: `ff-toolchain-api`, `regex`, `which`)
  - Satisfies: Req 15 (all)
- [x] 2.2 Implement `GccToolchainPlugin` struct with `ToolchainState` field
  - Satisfies: Req 15.1
- [x] 2.3 Implement `detect()` — probe PATH for `gcc`, `g++`, `gfortran`, `as`, `ld`, `ar` via `which`
  - Satisfies: Req 15.1, 15.2, 15.3, 15.9
- [x] 2.4 Implement platform detection (`std::env::consts::OS`) and install strategy selection
  - Satisfies: Req 15.8
- [x] 2.5 Implement `install()` for Windows (winget → MSYS2 fallback)
  - Satisfies: Req 15.4, 15.5, 15.6, 15.7, 15.8
- [x] 2.6 Implement `install()` for Linux (apt → dnf fallback)
  - Satisfies: Req 15.4, 15.5, 15.6, 15.7, 15.8
- [x] 2.7 Implement `install()` for macOS (Homebrew)
  - Satisfies: Req 15.4, 15.5, 15.6, 15.7, 15.8
- [x] 2.8 Implement `build()` — invoke `gcc`/`g++` with `BuildProfile` flags, stream output
  - Satisfies: Req 16.1, 16.2
- [x] 2.9 Implement GCC diagnostic parser (regex: `file:line:col: severity: message`)
  - Satisfies: Req 16.3
- [x] 2.10 Implement built-in `BuildProfile` constants: `debug`, `release`, `check-only`
  - Satisfies: Req 16.6
- [x] 2.11 Implement `plugin_init()` entry point registering `GccToolchainPlugin`
  - Satisfies: Req 15 (plugin registration)
- [x] 2.12 Write unit tests: detection with mock PATH, diagnostic parser, build profile flags
  - Satisfies: Req 15.1, 15.2, 15.3, 16.3, 16.6

## Task 3. Create `ff-rust-toolchain` plugin crate

- [x] 3.1 Scaffold `crates/ff-rust-toolchain/` with `Cargo.toml` (deps: `ff-toolchain-api`, `serde_json`, `which`)
  - Satisfies: Req 17 (all)
- [x] 3.2 Implement `RustToolchainPlugin` struct with `ToolchainState` field
  - Satisfies: Req 17.1
- [x] 3.3 Implement `detect()` — probe PATH for `rustc`, `cargo`, `rustup`; read active channel
  - Satisfies: Req 17.1, 17.2, 17.3, 17.9
- [x] 3.4 Implement `install()` — download and execute `rustup-init` (platform-appropriate method)
  - Satisfies: Req 17.4, 17.5, 17.6, 17.7
- [x] 3.5 Implement PATH extension after successful install (`~/.cargo/bin` / `%USERPROFILE%\.cargo\bin`)
  - Satisfies: Req 17.6
- [x] 3.6 Implement `update()` — run `rustup update` in background
  - Satisfies: Req 17.8
- [x] 3.7 Implement `build()` — invoke `cargo <subcommand> --message-format=json`, stream output
  - Satisfies: Req 18.1, 18.2, 18.7
- [x] 3.8 Implement Cargo.toml discovery (walk up directory tree from active file)
  - Satisfies: Req 18.1
- [x] 3.9 Implement JSON diagnostic parser (`serde_json`, extract `compiler-message` objects)
  - Satisfies: Req 18.3
- [x] 3.10 Implement `plugin_init()` entry point registering `RustToolchainPlugin`
  - Satisfies: Req 17 (plugin registration)
- [x] 3.11 Write unit tests: detection with mock PATH, JSON diagnostic parser, Cargo.toml discovery
  - Satisfies: Req 17.1, 17.2, 17.3, 18.1, 18.3

## Task 4. Toolchain_Panel UI in `ff-desktop`

- [x] 4.1 Create `crates/ff-desktop/src/toolchain_panel.rs` — egui panel rendering toolchain status rows
  - Satisfies: Req 15.2, 15.3, 15.9, 17.2, 17.3, 17.9
- [x] 4.2 Render `[Install GCC]` / `[Install via rustup]` buttons when `NotDetected`
  - Satisfies: Req 15.3, 17.3
- [x] 4.3 Render installation progress indicator when `Installing`
  - Satisfies: Req 15.5, 17.5
- [x] 4.4 Render build output scrollable text area
  - Satisfies: Req 16.2, 18.2
- [x] 4.5 Render clickable diagnostics list; on click navigate editor to file/line/col
  - Satisfies: Req 16.7, 18.6
- [x] 4.6 Wire `Compilers` menu option `3` from Primary Option Menu to open Toolchain_Panel
  - Satisfies: Req 14.6 (option 3 = Compilers)
- [x] 4.7 Write unit tests for panel state rendering logic (status rows, button visibility)
  - Satisfies: Req 15.2, 15.3, 17.2, 17.3
