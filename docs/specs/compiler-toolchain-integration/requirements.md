# Requirements Document — Compiler Toolchain Integration

## Introduction

This spec defines the requirements for integrating compiler toolchains into FileForgeWorkbench.
The workbench must support the full **GNU Compiler Collection (GCC)** and the **Rust toolchain**,
including the ability to detect whether each toolchain is installed, install it from within the
application, invoke the compiler on open files or projects, and surface diagnostics (errors and
warnings) back into the editor.

The integration is delivered as a plugin-architecture extension, keeping the core workbench
decoupled from any specific toolchain. Each toolchain is managed by a dedicated plugin crate
(`ff-gcc-toolchain` and `ff-rust-toolchain`) that registers itself with the plugin registry on
startup.

### Design Principles

> **Gap Note (from gap-analysis):** This spec covers GCC and Rust toolchains specifically. A generic `ToolchainPlugin` trait enabling future toolchains (LLVM, GnuCOBOL, OpenJDK) is identified as a gap and will be addressed in a future requirement when those toolchains are scheduled for implementation.

1. **Non-blocking installation.** Toolchain installation runs in the background via `ff-bgio`
   and never freezes the UI.
2. **Detection before installation.** The workbench always checks for an existing installation
   before offering to install.
3. **User consent required.** No toolchain is downloaded or installed without explicit user
   confirmation.
4. **Diagnostics are first-class.** Compiler output is parsed and surfaced as editor annotations,
   not just raw terminal text.
5. **Plugin-isolated.** Toolchain logic lives entirely in plugin crates; the core workbench has
   no compile-time dependency on any toolchain.

### Glossary

| Term | Definition |
|------|-----------|
| **GCC** | GNU Compiler Collection — the full suite: `gcc`, `g++`, `gfortran`, `gccgo`, `gdc`, `gcj` (where available), `as`, `ld`, `ar`, `make`. |
| **Rust Toolchain** | The Rust compiler (`rustc`), package manager (`cargo`), and toolchain manager (`rustup`). |
| **Toolchain_State** | The detected state of a toolchain: `NotDetected`, `Detected(version)`, `Installing`, `InstallFailed(reason)`, `Ready`. |
| **Diagnostic** | A compiler-emitted error or warning with file path, line, column, severity, and message text. |
| **Toolchain_Panel** | The UI panel (docked or floating) that shows toolchain status, install controls, and build output. |
| **Install_Source** | The origin from which a toolchain is fetched: package manager (apt/brew/winget), official installer script (rustup), or direct binary download. |
| **Build_Profile** | A named set of compiler flags and targets (e.g., `debug`, `release`, `check-only`). |

---

## Requirements

### Requirement 1: GCC Toolchain Detection and Installation

**User Story:** As a developer, I want the workbench to detect whether GCC is installed and, if
not, offer to install the full GCC toolchain from within the application, so that I can compile
C, C++, and other GCC-supported languages without leaving the editor.

#### Acceptance Criteria

1. WHEN the GCC plugin is activated, THE workbench SHALL probe the system PATH for `gcc`,
     `g++`, `gfortran`, `as`, `ld`, and `ar` executables and record the detected version of
     each component in the Toolchain_State.

2. WHEN all required GCC components (`gcc`, `g++`, `as`, `ld`, `ar`) are detected at the same
     version, THE Toolchain_State SHALL transition to `Ready` and the Toolchain_Panel SHALL
     display the detected GCC version string (e.g., `GCC 13.2.0 — Ready`).

3. WHEN one or more required GCC components are not found on PATH, THE Toolchain_State SHALL
     be `NotDetected` and the Toolchain_Panel SHALL display a clear message: `GCC not found —
     [Install GCC]` with an actionable install button.

4. WHEN the user activates the `[Install GCC]` action, THE workbench SHALL display a
     confirmation dialog listing: the components to be installed, the Install_Source appropriate
     for the current platform (winget on Windows, apt/dnf on Linux, Homebrew on macOS), and the
     estimated disk space required.

5. WHEN the user confirms the GCC installation, THE workbench SHALL launch the installation
     process via the background I/O service (`ff-bgio`), transition Toolchain_State to
     `Installing`, and display a live progress indicator in the Toolchain_Panel. The UI SHALL
     remain fully interactive during installation.

6. WHEN the GCC installation completes successfully, THE workbench SHALL re-probe the PATH,
     transition Toolchain_State to `Ready`, and display a success notification: `GCC installed
     successfully — version <X.Y.Z>`.

7. WHEN the GCC installation fails for any reason (network error, permission denied, package
     manager error), THE workbench SHALL transition Toolchain_State to `InstallFailed(reason)`,
     display the failure reason in the Toolchain_Panel, and offer a `[Retry]` and `[View Log]`
     action.

8. THE GCC plugin SHALL support the full GCC compiler collection components on each platform:
     - **Windows**: via winget (`mingw-w64` providing `gcc`, `g++`, `gfortran`, `as`, `ld`, `ar`,
       `make`) or MSYS2 if winget is unavailable.
     - **Linux**: via the system package manager (`build-essential` on Debian/Ubuntu,
       `gcc-toolset` on RHEL/Fedora).
     - **macOS**: via Homebrew (`gcc` formula providing the full collection).

9. WHEN the GCC toolchain is in `Ready` state, THE Toolchain_Panel SHALL list all detected
     GCC components with their individual version strings.

---

### Requirement 2: GCC Build and Diagnostics Integration

**User Story:** As a developer, I want to compile the file I am editing (or a project) using GCC
from within the workbench and see errors and warnings annotated directly in the editor, so that
I can fix issues without switching to a terminal.

#### Acceptance Criteria

1. WHEN the GCC toolchain is `Ready` and the active editor tab contains a C or C++ file,
     THE workbench SHALL enable a `Compile` action (menu item and keyboard shortcut) that
     invokes `gcc` or `g++` on the active file with the active Build_Profile flags.

2. WHEN a compile action is triggered, THE workbench SHALL run the compiler as a background
     process via `ff-bgio`, stream its stdout/stderr to the Toolchain_Panel build output area,
     and keep the editor fully interactive.

3. WHEN the compiler emits output in GCC diagnostic format (`file:line:col: severity: message`),
     THE workbench SHALL parse each line into a Diagnostic record and annotate the corresponding
     line in the editor with a coloured underline and inline message matching the severity
     (error = red, warning = yellow, note = blue).

4. WHEN the compiler exits with code 0, THE workbench SHALL display `Build succeeded` in the
     Toolchain_Panel status line and clear all previous Diagnostic annotations from the editor.

5. WHEN the compiler exits with a non-zero code, THE workbench SHALL display `Build failed —
     N error(s), M warning(s)` in the Toolchain_Panel status line and retain all Diagnostic
     annotations.

6. THE workbench SHALL provide at least the following built-in Build_Profiles for GCC:
     - `debug`: `-g -O0 -Wall -Wextra`
     - `release`: `-O2 -DNDEBUG`
     - `check-only`: `-fsyntax-only -Wall -Wextra`

7. WHEN the user clicks on a Diagnostic entry in the Toolchain_Panel output list, THE workbench
     SHALL navigate the editor to the file, line, and column referenced by that Diagnostic.

---

### Requirement 3: Rust Toolchain Detection and Installation

**User Story:** As a Rust developer, I want the workbench to detect whether the Rust toolchain
is installed and, if not, offer to install it via `rustup` from within the application, so that
I can build and check Rust projects without leaving the editor.

#### Acceptance Criteria

1. WHEN the Rust plugin is activated, THE workbench SHALL probe the system PATH for `rustc`,
     `cargo`, and `rustup` executables and record the detected version of each in the
     Toolchain_State.

2. WHEN `rustc` and `cargo` are detected, THE Toolchain_State SHALL transition to `Ready` and
     the Toolchain_Panel SHALL display the detected Rust version string (e.g.,
     `Rust 1.78.0 (stable) — Ready`) and the active toolchain channel (stable/beta/nightly).

3. WHEN `rustc` or `cargo` are not found on PATH, THE Toolchain_State SHALL be `NotDetected`
     and the Toolchain_Panel SHALL display: `Rust not found — [Install via rustup]` with an
     actionable install button.

4. WHEN the user activates the `[Install via rustup]` action, THE workbench SHALL display a
     confirmation dialog stating: the installation method (`rustup-init` official installer),
     the default toolchain channel to be installed (stable), the target directory
     (`~/.cargo/bin` on Unix, `%USERPROFILE%\.cargo\bin` on Windows), and the estimated disk
     space required.

5. WHEN the user confirms the Rust installation, THE workbench SHALL download and execute the
     official `rustup-init` installer via the background I/O service (`ff-bgio`), transition
     Toolchain_State to `Installing`, and display a live progress indicator. The UI SHALL remain
     fully interactive during installation.

6. WHEN the Rust installation completes successfully, THE workbench SHALL re-probe the PATH
     (including the newly added `~/.cargo/bin`), transition Toolchain_State to `Ready`, and
     display: `Rust installed successfully — rustc <version>`.

7. WHEN the Rust installation fails for any reason, THE workbench SHALL transition
     Toolchain_State to `InstallFailed(reason)`, display the failure reason, and offer
     `[Retry]` and `[View Log]` actions.

8. WHEN `rustup` is detected, THE Toolchain_Panel SHALL display a `[Update Toolchain]` button
     that runs `rustup update` in the background and reports the result.

9. WHEN `rustup` is detected, THE Toolchain_Panel SHALL display the list of installed
     toolchain channels (stable, beta, nightly) with their versions and allow the user to
     switch the active channel.

---

### Requirement 4: Rust Build and Diagnostics Integration

**User Story:** As a Rust developer, I want to run `cargo build`, `cargo check`, and `cargo test`
on the current project from within the workbench and see compiler errors annotated in the editor,
so that I can fix issues without switching to a terminal.

#### Acceptance Criteria

1. WHEN the Rust toolchain is `Ready` and the active editor tab is inside a Cargo workspace
     (a `Cargo.toml` is found by walking up the directory tree), THE workbench SHALL enable
     `Cargo Build`, `Cargo Check`, and `Cargo Test` actions in the Compilers menu and
     Toolchain_Panel.

2. WHEN a Cargo action is triggered, THE workbench SHALL run the corresponding `cargo`
     subcommand as a background process via `ff-bgio`, stream its output to the Toolchain_Panel
     build output area, and keep the editor fully interactive.

3. WHEN `cargo` emits JSON diagnostic output (`--message-format=json`), THE workbench SHALL
     parse each diagnostic message into a Diagnostic record and annotate the corresponding line
     in the editor with a coloured underline and inline message.

4. WHEN `cargo` exits with code 0, THE workbench SHALL display `Cargo succeeded` in the
     Toolchain_Panel status line and clear all previous Diagnostic annotations.

5. WHEN `cargo` exits with a non-zero code, THE workbench SHALL display `Cargo failed —
     N error(s), M warning(s)` and retain all Diagnostic annotations.

6. WHEN the user clicks on a Diagnostic entry in the Toolchain_Panel output list, THE workbench
     SHALL navigate the editor to the file, line, and column referenced by that Diagnostic.

7. THE workbench SHALL pass `--message-format=json` to all `cargo` invocations to enable
     structured diagnostic parsing (Requirement 18.3).

---

### Requirement 5: Generic Toolchain Plugin Extension Point

**User Story:** As a platform architect, I want the toolchain integration to define a
generic `ToolchainPlugin` trait that any future toolchain (LLVM/Clang, GnuCOBOL, OpenJDK,
Python, Go) can implement, so that new toolchains can be added as plugin crates without
modifying the core workbench or the existing GCC/Rust plugin crates.

**Gap reference:** FR-0971 identified in `docs/reviews/requirements-review/gap-analysis.md`
section 6.4 (Generic toolchain plugin trait -- PARTIAL, High priority).

#### Acceptance Criteria

1. THE `ff-toolchain-api` crate SHALL define a `ToolchainPlugin` trait as the sole
   extension point for toolchain integrations; the trait SHALL declare at minimum:
   `name() -> &str`, `state() -> ToolchainState`, `detect()`, `install()`, and `build()`.

2. THE `ToolchainPlugin` trait SHALL be documented as the general extension point for
   all future toolchains (LLVM/Clang, GnuCOBOL, OpenJDK, Python, Go); the trait
   contract SHALL NOT contain any GCC-specific or Rust-specific assumptions.

3. WHEN a new toolchain plugin crate implements `ToolchainPlugin` and registers itself
   with the plugin registry, THE Toolchain_Panel SHALL display its status row and
   install/build controls without any changes to the core workbench or existing plugin crates.

4. THE `ff-toolchain-api` crate SHALL be the only compile-time dependency that a new
   toolchain plugin crate requires from the workbench platform; it SHALL NOT need to
   depend on `ff-gcc-toolchain` or `ff-rust-toolchain`.

---

## Non-Functional Requirements

### Performance

- Toolchain detection (PATH probe) SHALL complete within 2 seconds on a modern desktop system.
- Compiler invocation and diagnostic parsing SHALL not block the UI thread — all operations run via `ff-bgio`.

### Reliability

- WHEN a toolchain installation fails, THE workbench SHALL remain fully functional for all non-compilation features.
- THE toolchain state SHALL be re-probed on each workbench startup to reflect any system changes made outside the workbench.

### Extensibility

- The plugin isolation principle SHALL be maintained: no toolchain-specific code SHALL appear in the core workbench crates.
