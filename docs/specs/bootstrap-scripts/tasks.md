# Bootstrap Scripts -- Tasks

## Task List

- [x] 1. Create `bootstrap/` directory at repository root
  - Satisfies: Req 1.1, 2.1, 3.1, 4.1

- [x] 2. Create `bootstrap/logs/.gitkeep` and add `bootstrap/logs/` to `.gitignore`
  - Satisfies: Req 5.3

- [x] 3. Write `bootstrap/bootstrap-windows.ps1`
  - [x] 3.1 Accept `-Root` parameter (default `C:\tools`) and derive CARGO_HOME / RUSTUP_HOME
  - [x] 3.2 Idempotency check: skip if `rustc.exe` already present
  - [x] 3.3 Download `rustup-init.exe` via `Invoke-WebRequest` with WebClient fallback
  - [x] 3.4 Invoke rustup-init with `--no-modify-path --default-toolchain stable`
  - [x] 3.5 Update user PATH via `HKCU:\Environment` registry key
  - [x] 3.6 Verify with `rustc --version` and `cargo --version`
  - [x] 3.7 Write timestamped log to `bootstrap\logs\`
  - [x] 3.8 Print "Next steps" summary
  - Satisfies: Req 1.1-1.7, 5.1-5.6

- [x] 4. Write `bootstrap/bootstrap-linux.sh`
  - [x] 4.1 Set CARGO_HOME and RUSTUP_HOME to `~/.tools/rust/...`
  - [x] 4.2 Idempotency check: skip if `rustc` already present
  - [x] 4.3 Download rustup installer via curl with wget fallback
  - [x] 4.4 Invoke installer with `--no-modify-path -y --default-toolchain stable`
  - [x] 4.5 Append PATH export to `~/.profile` and `~/.bashrc` if not present
  - [x] 4.6 Verify with `rustc --version` and `cargo --version`
  - [x] 4.7 Write timestamped log to `bootstrap/logs/`
  - [x] 4.8 Print "Next steps" summary
  - Satisfies: Req 2.1-2.6, 5.1-5.6

- [x] 5. Write `bootstrap/bootstrap-macos.sh`
  - [x] 5.1 Set CARGO_HOME and RUSTUP_HOME to `~/.tools/rust/...`
  - [x] 5.2 Idempotency check: skip if `rustc` already present
  - [x] 5.3 Download rustup installer via curl
  - [x] 5.4 Invoke installer with `--no-modify-path -y --default-toolchain stable`
  - [x] 5.5 Append PATH export to `~/.zshrc` and `~/.bash_profile` if not present
  - [x] 5.6 Check for Xcode Command Line Tools; print warning if absent
  - [x] 5.7 Verify with `rustc --version` and `cargo --version`
  - [x] 5.8 Write timestamped log to `bootstrap/logs/`
  - [x] 5.9 Print "Next steps" summary
  - Satisfies: Req 3.1-3.6, 5.1-5.6

- [x] 6. Write `bootstrap/README.md`
  - [x] 6.1 Document prerequisites for each platform
  - [x] 6.2 Document install paths (Windows: `C:\tools\rust`, Unix: `~/.tools/rust`)
  - [x] 6.3 Document exact command to run on each platform
  - [x] 6.4 Document next steps: `cargo build`, `cargo test`, launch `ffwb`
  - [x] 6.5 Reference `tools/powershell/ffwb_make.ps1` for the full build workflow
  - Satisfies: Req 4.1, 4.2

- [x] 7. Update root `README.md` "Building" section to reference `bootstrap/`
  - Satisfies: Req 4.1
