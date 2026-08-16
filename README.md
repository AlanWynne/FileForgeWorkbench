# FileForge Workbench

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](#building)

FileForge Workbench is a cross-platform enterprise file editor and mainframe workstation inspired by IBM ISPF and File-AID.

## Status

**Initial implementation complete.** All 60 library crates and the `ff-desktop` runnable binary have been built. The binary (`ffwb`) is functional for file viewing and navigation.

### What works today

- Launch with one or more files: `ffwb file.txt other.rs`
- Multi-tab editor — open files via `EDIT <path>` in the command field or `File > Open…`
- Per-tab viewport and cursor state — switching tabs restores scroll position and cursor
- Keyboard navigation — Arrow keys, Page Up/Down
- Live status bar — line/column, encoding, line count, modified indicator
- Session persistence — open tabs are saved on exit and restored on next launch
- Three built-in themes — dark, light, high-contrast (View menu)
- ISPF-style `Command ===>` field — supports `EDIT <path>`, `EXIT`, `QUIT`, `=X`

### Known gaps (Phase S)

- `File > Open…` menu item does not yet show a native file-open dialog
- Keyboard text input is not yet wired (editor is read-only)
- `File > Save` and `File > Save As…` are not yet implemented

## Architecture

60 library crates organised in dependency waves, assembled into the `ff-desktop` binary:

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

## Building

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

- Text and binary file editing
- Source code support (syntax highlighting etc.)
- Plugin architecture
- Dataset Catalogues
- Dataset Allocation
- IDCAMS emulation
- JES emulation
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
