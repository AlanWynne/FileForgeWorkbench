# Bootstrap Scripts -- Design

## Overview

The bootstrap scripts are thin shell wrappers around the official `rustup`
installer. They add three things the raw installer does not provide:

1. Explicit, non-default install paths (under `C:\tools\rust` on Windows,
   `~/.tools/rust` on Unix) so the toolchain does not land in the user's
   home directory roaming profile or interfere with a system-level Rust.
2. Idempotency: each script checks whether the toolchain is already present
   before downloading anything.
3. A structured "next steps" message that points the user directly at
   `cargo build` and `cargo test`.

No new crates are introduced. The scripts are plain shell files; they have
no Rust build dependency.

## Repository layout

```
bootstrap/
  bootstrap-windows.ps1   -- Windows PowerShell 5.1+
  bootstrap-linux.sh      -- Linux bash
  bootstrap-macos.sh      -- macOS bash/zsh
  README.md               -- usage, prerequisites, paths, next steps
  logs/                   -- timestamped log files (gitignored)
```

The `bootstrap/` folder is at the repository root so it is present
immediately after `git clone`.

## Windows script design

- Language: PowerShell 5.1 (available on every Windows 10/11 machine).
- Downloads `rustup-init.exe` from `https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe`.
- Sets `$env:CARGO_HOME` and `$env:RUSTUP_HOME` before invoking rustup-init.
- Passes `--no-modify-path` to rustup-init; PATH update is done explicitly
  via `HKCU:\Environment` registry key (no admin required).
- Idempotency check: if `$CargoHome\bin\rustc.exe` exists and `-ForceReinstall`
  is not set, skip download and install.
- Log file: `bootstrap\logs\bootstrap-windows-<timestamp>.log`.
- Mirrors the Rust installation pattern already proven in
  `C:\tools\scripts\Desktop-Preparation\setup-desktop.ps1` (lines ~310-340).

## Unix script design (Linux and macOS share the same pattern)

- Language: bash (POSIX-compatible subset).
- Downloads the rustup installer via `curl -sSf https://sh.rustup.rs` or
  falls back to `wget -qO-` if curl is absent.
- Sets `CARGO_HOME` and `RUSTUP_HOME` environment variables before piping
  to `sh`.
- Passes `--no-modify-path -y --default-toolchain stable` to the installer.
- Idempotency check: if `$CARGO_HOME/bin/rustc` exists, skip install.
- PATH update: appends `export PATH="$CARGO_HOME/bin:$PATH"` to `~/.profile`
  and `~/.bashrc` (Linux) or `~/.zshrc` and `~/.bash_profile` (macOS) if
  the line is not already present.
- Log file: `bootstrap/logs/bootstrap-<platform>-<timestamp>.log`.

## macOS-specific additions

- Checks for Xcode Command Line Tools via `xcode-select -p`; prints a
  warning if absent but does not abort (pure-Rust builds do not need them).
- Targets `~/.zshrc` as the primary shell profile (default since macOS Catalina).

## What the scripts do NOT do

- Do not install Git, VS Code, or any other tool.
- Do not modify system PATH or require elevation.
- Do not duplicate `setup-desktop.ps1`; that script is for a full personal
  workstation. These scripts are the minimal "build this one project" path.

## Relationship to existing tooling

- `tools/powershell/ffwb_make.ps1` -- the build/test/run script that users
  run after bootstrapping. The bootstrap README points to it.
- `C:\tools\scripts\Desktop-Preparation\setup-desktop.ps1` -- the full
  workstation setup script. The Windows bootstrap script reuses the same
  Rust installation pattern from that script.

## No design changes to existing crates

The bootstrap scripts are external to the Cargo workspace. No `Cargo.toml`,
no new crates, no changes to existing source files.
