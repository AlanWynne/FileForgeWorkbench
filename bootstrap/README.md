# Bootstrap -- Getting Started

These scripts install the Rust stable toolchain and verify that
FileForge Workbench builds correctly on your machine.
No administrator rights are required.

## Prerequisites

| Platform | Requirement |
|----------|-------------|
| Windows  | Windows 10 or later; PowerShell 5.1 (built-in) |
| Linux    | bash; curl or wget; standard build tools (gcc/make) |
| macOS    | bash or zsh; curl (built-in); Xcode CLT recommended |

No Rust installation is needed before running the script -- that is what
the script does.

## Install paths

| Platform | CARGO_HOME | RUSTUP_HOME |
|----------|-----------|------------|
| Windows  | `C:\tools\rust\cargo` | `C:\tools\rust\rustup` |
| Linux    | `~/.tools/rust/cargo` | `~/.tools/rust/rustup` |
| macOS    | `~/.tools/rust/cargo` | `~/.tools/rust/rustup` |

The Windows path can be changed with the `-Root` parameter (see below).
The toolchain is installed entirely under these directories; nothing is
written to system locations.

## Running the script

### Windows

Open PowerShell (no elevation needed) and run:

```powershell
powershell -ExecutionPolicy Bypass -File bootstrap\bootstrap-windows.ps1
```

Optional parameters:

```powershell
# Install under D:\tools instead of C:\tools
powershell -ExecutionPolicy Bypass -File bootstrap\bootstrap-windows.ps1 -Root D:\tools

# Install a specific toolchain channel
powershell -ExecutionPolicy Bypass -File bootstrap\bootstrap-windows.ps1 -Toolchain nightly

# Force re-download even if Rust is already present
powershell -ExecutionPolicy Bypass -File bootstrap\bootstrap-windows.ps1 -ForceReinstall
```

### Linux

```bash
bash bootstrap/bootstrap-linux.sh
```

Optional:

```bash
# Install a specific toolchain channel
bash bootstrap/bootstrap-linux.sh --toolchain nightly
```

### macOS

```bash
bash bootstrap/bootstrap-macos.sh
```

Optional:

```bash
bash bootstrap/bootstrap-macos.sh --toolchain nightly
```

## After the script completes

1. Open a new terminal (or source your shell profile) so the updated PATH
   takes effect.

2. From the repository root, build the project:

   ```bash
   cargo build
   ```

3. Run the test suite:

   ```bash
   cargo test
   ```

4. Launch the application:

   ```
   # Windows
   .\target\debug\ffwb.exe

   # Linux / macOS
   ./target/debug/ffwb
   ```

## Full build workflow

For the complete build-lint-test-run cycle used during development, see
`tools/powershell/ffwb_make.ps1` (requires PowerShell 7 on Linux/macOS).

## Idempotency

All scripts are safe to run more than once.  If the toolchain is already
installed at the expected path the script prints "Rust already installed"
and exits without downloading anything.

## Log files

Each run writes a timestamped log to `bootstrap/logs/`.  These files are
excluded from version control (see `.gitignore`).  If a run fails, check
the log for the exact error message.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `rustc: command not found` after script | PATH not refreshed | Open a new terminal or source your profile |
| Download fails on Windows | Corporate proxy / TLS inspection | Set `$env:HTTPS_PROXY` before running the script |
| `linker 'cc' not found` on Linux | Build tools missing | `sudo apt install build-essential` (Debian/Ubuntu) |
| `xcode-select --install` prompt on macOS | CLT not installed | Run `xcode-select --install` then re-run the script |
