# Bootstrap Scripts -- Requirements

## Introduction

This sub-project defines the acceptance criteria for a set of platform-specific
bootstrap scripts that allow a new contributor to download the FileForge Workbench
source repository and build it from scratch without requiring administrator rights
or any pre-installed toolchain beyond a shell.

The scripts live in `bootstrap/` at the repository root so they are present
immediately after `git clone`.

## Glossary

| Term | Meaning |
|------|---------|
| Bootstrap script | A platform-specific script that installs the Rust toolchain and verifies the build |
| CARGO_HOME | Directory where Cargo stores its registry, binaries, and caches |
| RUSTUP_HOME | Directory where rustup stores toolchain installations |
| Idempotent | Safe to run multiple times; skips steps already completed |
| No-admin | Does not require elevated privileges (UAC on Windows, sudo on Unix) |

## Requirements

### Requirement 1 -- Windows bootstrap script

1.1 WHEN a user runs `bootstrap\bootstrap-windows.ps1` on Windows 10 or later
    THE script SHALL install the Rust stable toolchain into `C:\tools\rust\cargo`
    (CARGO_HOME) and `C:\tools\rust\rustup` (RUSTUP_HOME) without requiring
    administrator rights.

1.2 WHEN the Rust toolchain is already present at the configured paths
    THE script SHALL skip the download and installation steps and report
    "Rust already installed".

1.3 WHEN the installation completes successfully
    THE script SHALL verify the installation by running `rustc --version` and
    `cargo --version` and displaying the output.

1.4 WHEN the installation completes successfully
    THE script SHALL add `C:\tools\rust\cargo\bin` to the current user's PATH
    via the Windows registry (HKCU:\Environment) without modifying the system PATH.

1.5 WHEN the script is invoked
    THE script SHALL accept an optional `-Root` parameter (default `C:\tools`)
    that redirects all installation paths under the specified root.

1.6 WHEN the script completes
    THE script SHALL print a "Next steps" summary telling the user to open a new
    terminal and run `cargo build` from the repository root.

1.7 WHEN the script is run on Windows PowerShell 5.1 or later
    THE script SHALL execute without requiring PowerShell 7 or any additional modules.

### Requirement 2 -- Linux bootstrap script

2.1 WHEN a user runs `bash bootstrap/bootstrap-linux.sh` on a Linux system
    THE script SHALL install the Rust stable toolchain via the official
    `rustup` installer with CARGO_HOME and RUSTUP_HOME set to
    `~/.tools/rust/cargo` and `~/.tools/rust/rustup` respectively.

2.2 WHEN the Rust toolchain is already present at the configured paths
    THE script SHALL skip the download and installation steps and report
    "Rust already installed".

2.3 WHEN the installation completes successfully
    THE script SHALL verify the installation by running `rustc --version` and
    `cargo --version` and displaying the output.

2.4 WHEN the installation completes successfully
    THE script SHALL append the cargo bin directory to PATH in `~/.profile`
    and `~/.bashrc` if not already present.

2.5 WHEN the script completes
    THE script SHALL print a "Next steps" summary telling the user to source
    their profile and run `cargo build` from the repository root.

2.6 WHEN `curl` is not available
    THE script SHALL attempt to use `wget` as a fallback to download the
    rustup installer.

### Requirement 3 -- macOS bootstrap script

3.1 WHEN a user runs `bash bootstrap/bootstrap-macos.sh` on macOS 12 or later
    THE script SHALL install the Rust stable toolchain via the official
    `rustup` installer with CARGO_HOME and RUSTUP_HOME set to
    `~/.tools/rust/cargo` and `~/.tools/rust/rustup` respectively.

3.2 WHEN the Rust toolchain is already present at the configured paths
    THE script SHALL skip the download and installation steps and report
    "Rust already installed".

3.3 WHEN the installation completes successfully
    THE script SHALL verify the installation by running `rustc --version` and
    `cargo --version` and displaying the output.

3.4 WHEN the installation completes successfully
    THE script SHALL append the cargo bin directory to PATH in `~/.zshrc`
    (default shell on macOS) and `~/.bash_profile` if not already present.

3.5 WHEN the script completes
    THE script SHALL print a "Next steps" summary telling the user to source
    their profile and run `cargo build` from the repository root.

3.6 WHEN Xcode Command Line Tools are not installed
    THE script SHALL print a warning and the command to install them
    (`xcode-select --install`) but SHALL NOT abort; Rust itself does not
    require them for a pure-Rust build.

### Requirement 4 -- bootstrap README

4.1 WHEN a user opens `bootstrap/README.md`
    THE file SHALL describe the purpose of each script, the prerequisites,
    the install paths used on each platform, and the exact command to run.

4.2 WHEN a user has run a bootstrap script successfully
    THE README SHALL describe the next steps: `cargo build`, `cargo test`,
    and how to launch `ffwb`.

### Requirement 5 -- cross-cutting constraints

5.1 WHEN any bootstrap script runs
    THE script SHALL NOT require administrator or root privileges at any step.

5.2 WHEN any bootstrap script runs
    THE script SHALL be idempotent: running it a second time SHALL produce
    no errors and SHALL leave the environment in the same state as after
    the first run.

5.3 WHEN any bootstrap script runs
    THE script SHALL write a timestamped log file to `bootstrap/logs/`
    (created if absent) so the user can diagnose failures.

5.4 WHEN any bootstrap script runs
    THE script SHALL use `--no-modify-path` with rustup-init so that PATH
    modification is handled explicitly by the script, not by rustup.

5.5 WHEN any bootstrap script runs
    THE script SHALL install the `stable` toolchain targeting the host
    platform's default target triple.

5.6 WHEN any bootstrap script runs
    THE script SHALL NOT install any toolchain other than `stable` unless
    the user explicitly passes a `-Toolchain` / `--toolchain` parameter.
