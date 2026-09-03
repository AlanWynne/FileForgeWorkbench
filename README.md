# FileForge Workbench

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](#building)

FileForge Workbench is a cross-platform enterprise file editor and mainframe workstation inspired by IBM ISPF and File-AID.

## Technology Stack

| Concern | Choice | Notes |
|---------|--------|-------|
| Language | [Rust](https://www.rust-lang.org/) (stable) | Entire codebase — 64 crates |
| GUI framework | [egui](https://github.com/emilk/egui) + [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) | Immediate-mode UI; the GUI shell is a replaceable layer over a GUI-independent core |
| Async runtime | [Tokio](https://tokio.rs/) (multi-threaded) | All background I/O — file ops, search, toolchain invocation |
| Configuration | [TOML](https://toml.io/) | Themes, key maps, user preferences, session state, layout definitions |
| Scripting | [Lua](https://www.lua.org/) via `ff-lua` | User macro engine with editor API and event hooks |
| Error handling | [thiserror](https://github.com/dtolnay/thiserror) (library crates) + [anyhow](https://github.com/dtolnay/anyhow) (binary) | No `unwrap()` in library code |
| Testing | Built-in `cargo test` + [proptest](https://github.com/proptest-rs/proptest) | Property-based tests alongside unit and integration tests |
| Build system | Cargo workspace | Single `cargo build` compiles all 64 crates |

The full architectural rationale — GUI-independence principle, command-driven execution model, plugin contract, and async model — is documented in [`docs/specs/workbench-requirements-merge/architecture-brief.md`](docs/specs/workbench-requirements-merge/architecture-brief.md).

## Status

**Phase AX complete.** All 64 crates and the `ff-desktop` binary (`ffwb`) are built and passing 404 tests with 0 failures.

### Project documentation

- [Documentation index](docs/README.md)
- [Current work dashboard](docs/status/current-work.md)
- [Requirements source register](docs/requirements/source-register.md)
- [Canonical feature specifications](docs/specs/)
- [Reusable project tools](tools/)

### What works today

- ISPF-style Primary Option Menu (POM) — tabbed, with live calendar, month navigation, and interactive option buttons
- Multi-tab editor — open files via `EDIT <path>`, `File > Open…`, or CLI arguments
- Keyboard text input — typed characters insert, Backspace deletes, Enter splits lines
- File save — `File > Save` and `Ctrl+S` write to disk
- Per-tab viewport and cursor state — switching tabs restores scroll position and cursor
- Keyboard navigation — Arrow keys, Page Up/Down, mouse click to position cursor
- Ctrl+Z undo — restores document and cursor to previous state
- Live status bar — line/column, encoding, line count, modified indicator
- Session persistence — open tabs, zoom levels, key bar visibility, and catalog registry saved on exit and restored on next launch
- Three built-in themes — dark, light, high-contrast; user-configurable colour tokens via TOML
- ISPF-style `Command ===>` field — `EDIT`, `EXIT`, `QUIT`, `=X`, `=0`–`=8`, `=FILES`, `FILES`, `KEYS`, `PFSHOW`, `END`, `RETURN`, `FIND`, `CHANGE`, `LOCATE`, `SORT`, `EXCLUDE`, `SHOW`, `RESET`
- Virtual Catalog Manager — create, edit, delete Mainframe / POSIX / Native / Cloud catalogs; catalog registry persisted across restarts
- Default Home catalog — on first launch, a Native catalog pointing to the user's home directory is created automatically and cannot be deleted
- Dataset Allocation dialog — ISPF-style fields, HLQ pre-population, duplicate detection, uppercase enforcement
- File Explorer panel — `=2` / `=FILES` / `FILES` commands; tree view grouped by catalog type
- Settings panel — all config keys browsable, editable, and resettable; filter input; provenance badges
- Key Configuration dialog — 24-key grid per scope, modifier bindings (Shift/Ctrl/Alt+Fn), TOML persistence
- 24-key label bar — two rows, clickable slots, PFSHOW ON/OFF, session persistence
- Compiler Toolchain panel — GCC and Rust detection, install, build, diagnostic parsing and display
- Detachable tab windows — tabs can be moved to separate OS windows and redocked
- Tab-order focus cycle — Tab/Shift+Tab through command field, POM options, calendar, menu bar, tab headers
- Help > About dialog

### Known gaps

- File Explorer tree view (expand/collapse, double-click to open) — UI rendering deferred
- `File > Open…` native dialog on some platforms may need testing
- Per-context key maps TOML config parsing — deferred
- Contextual help (`HELP` command) — not yet implemented
- `File > Save As…` — not yet implemented

## Architecture

64 crates organised in dependency waves, assembled into the `ff-desktop` binary:

| Wave | Crates |
|------|--------|
| Foundation | `ff-logging` |
| Platform | `ff-core`, `ff-config`, `ff-command`, `ff-plugin`, `ff-workflow`, `ff-layout` |
| VFS | `ff-vfs`, `ff-connector-local`, `ff-connector-ext` |
| Core Editor | `ff-document-model`, `ff-edit`, `ff-undo`, `ff-viewport-scrolling`, `ff-linemap` |
| Command Engine | `ff-cmd-semantics`, `ff-find`, `ff-linecmd`, `ff-filter`, `ff-nav` |
| UI | `ff-menu`, `ff-theme`, `ff-decorations`, `ff-whitespace`, `ff-caret` |
| Language | `ff-lang`, `ff-syntax`, `ff-indent` |
| File I/O & Session | `ff-fileops`, `ff-bgio`, `ff-encoding`, `ff-extmod`, `ff-session`, `ff-tabs` |
| Desktop Integration | `ff-clipboard`, `ff-keys`, `ff-shell`, `ff-help`, `ff-zoom`, `ff-wrap` |
| Extensions | `ff-lua`, `ff-completion` |
| Display Modes | `ff-hex`, `ff-seqnum`, `ff-tabmask` |
| FileForge Domain | `ff-forge`, `ff-struct`, `ff-select`, `ff-asa`, `ff-viewers` |
| Dataset Catalog | `ff-dscatalog`, `ff-dsalloc`, `ff-idcams` |
| JES | `ff-jes` |
| File Explorer | `ff-tree`, `ff-compare` |
| Performance | `ff-idle`, `ff-largefile` |
| Database Tool | `ff-dbtool` |
| Compiler Toolchain | `ff-toolchain-api`, `ff-gcc-toolchain`, `ff-rust-toolchain` |

## Building

**Prerequisites:** Rust stable toolchain (`rustup` recommended — https://rustup.rs). No other runtime dependencies are required.

```bash
cargo build                        # debug build
cargo build --release              # release build
cargo test                         # run all tests
cargo clippy -- -D warnings        # lint
```

## Running

```bash
.\target\debug\ffwb.exe                        # launch empty
.\target\debug\ffwb.exe path\to\file.txt       # open a file
.\target\debug\ffwb.exe file1.txt file2.rs     # open multiple files as tabs
```

## Features

- Text file editing with keyboard input, undo, and save
- ISPF-style Primary Option Menu with live calendar
- Virtual Catalog Manager — Mainframe, POSIX, Native, and Cloud catalog types
- Default Home catalog auto-created on first launch
- Dataset Allocation dialog with ISPF-style fields
- File Explorer panel with catalog tree view
- Settings panel — all config keys browsable and editable
- Key Configuration dialog — 24-key grid with modifier bindings
- Compiler Toolchain panel — GCC and Rust detection, build, diagnostics
- Detachable tab windows
- Source code support (syntax highlighting etc.)
- Plugin architecture
- Dataset Catalogues
- Dataset Allocation
- IDCAMS emulation
- JES emulation
- User-configurable themes via TOML
- Session persistence across restarts
- Windows, Linux and macOS support

## Planned Plugin Capabilities

- Integration with the GNU Compiler Collection (GCC) toolchain
- Support for C, C++, and other GCC-supported languages
- Build, compile, and diagnostics management from within FileForge Workbench
- Job Scheduling maintenance
- CICS Emulation
- CICS Precompilation

## Future Plugin Ecosystem

Planned plugins will provide integration with industry-standard development toolchains, including:

- GNU Compiler Collection (GCC)
- LLVM/Clang
- GnuCOBOL
- OpenJDK
- Python
- Rust
- Go

These plugins will enable source editing, build automation, diagnostics, debugging, and project management from within FileForge Workbench.

## Deferred Connectors

The following connectors are designed but not yet implemented (initial release scope):

- `connector-network-fs` — network filesystem access
- `connector-ftp-sftp` — FTP/SFTP remote access
- `connector-mainframe` — z/OS mainframe connectivity
- `connector-cloud` — cloud storage integration

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for
build instructions, the mandatory TDD workflow, and the pull request process.

## Licence

Copyright 2024 FileForge Contributors.

Licensed under the **Apache License, Version 2.0**. You may not use this
project except in compliance with the Licence. A copy of the Licence is
included in the [LICENSE](LICENSE) file and is also available at:

> http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed
under the Licence is distributed on an **"AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND**, either express or implied.
