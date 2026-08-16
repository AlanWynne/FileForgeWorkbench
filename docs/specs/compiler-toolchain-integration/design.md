# Design Document — Compiler Toolchain Integration

## 1. Overview

Compiler toolchain integration is delivered as two plugin crates that register with the existing
`ff-plugin` plugin architecture. The core workbench binary (`ff-desktop`) has no compile-time
dependency on either toolchain crate — they are loaded at runtime through the plugin registry.

```
ff-desktop
  └── ff-plugin (registry)
        ├── ff-gcc-toolchain   (plugin crate — GCC detection, install, build, diagnostics)
        └── ff-rust-toolchain  (plugin crate — Rust/rustup detection, install, build, diagnostics)
```

Both plugin crates share a common `ToolchainPlugin` trait (defined in a new `ff-toolchain-api`
crate) so that the Toolchain_Panel UI can be generic over any toolchain.

---

## 2. New Crates

| Crate | Wave | Purpose |
|-------|------|---------|
| `ff-toolchain-api` | Wave 2 (Platform) | Shared trait `ToolchainPlugin`, `Toolchain_State` enum, `Diagnostic` struct, `BuildProfile` struct |
| `ff-gcc-toolchain` | Wave 18 (Compilers) | GCC detection, platform-specific install, `gcc`/`g++` invocation, GCC diagnostic parser |
| `ff-rust-toolchain` | Wave 18 (Compilers) | rustup/rustc/cargo detection, rustup-init install, `cargo` invocation, JSON diagnostic parser |

---

## 3. `ff-toolchain-api` — Shared Abstractions

```rust
pub enum ToolchainState {
    NotDetected,
    Detected { version: String },
    Installing,
    InstallFailed { reason: String },
    Ready { version: String },
}

pub struct Diagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub severity: DiagnosticSeverity,  // Error | Warning | Note
    pub message: String,
}

pub struct BuildProfile {
    pub name: String,
    pub flags: Vec<String>,
}

pub trait ToolchainPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn state(&self) -> ToolchainState;
    fn detect(&mut self);
    fn install(&mut self, sender: mpsc::Sender<InstallProgress>);
    fn build(&self, profile: &BuildProfile, sender: mpsc::Sender<BuildEvent>);
}
```

---

## 4. GCC Plugin Design (`ff-gcc-toolchain`)

### 4.1 Detection

`detect()` runs `gcc --version`, `g++ --version`, `as --version`, `ld --version`, `ar --version`
via `std::process::Command`. All probes run synchronously on a background thread (via `ff-bgio`
thread pool) to avoid blocking the UI thread.

### 4.2 Installation — Platform Strategy

| Platform | Install_Source | Command |
|----------|---------------|---------|
| Windows | winget | `winget install --id MSYS2.MSYS2 -e` then `pacman -S mingw-w64-ucrt-x86_64-gcc` |
| Windows (fallback) | MSYS2 direct | Download MSYS2 installer from msys2.org |
| Linux (Debian/Ubuntu) | apt | `sudo apt-get install -y build-essential gfortran` |
| Linux (RHEL/Fedora) | dnf | `sudo dnf groupinstall -y "Development Tools"` |
| macOS | Homebrew | `brew install gcc` |

The plugin detects the platform at runtime using `std::env::consts::OS` and selects the
appropriate strategy. If the preferred package manager is not found, it falls back to the next
option in the table.

### 4.3 Build Invocation

```
gcc/g++ <source_file> <profile_flags> -o <output> 2>&1
```

Output is streamed line-by-line to the `BuildEvent::OutputLine(String)` channel. Each line is
also passed to the GCC diagnostic parser.

### 4.4 GCC Diagnostic Parser

GCC emits diagnostics in the format:
```
<file>:<line>:<col>: <severity>: <message>
```

The parser uses a regex:
```
^(?P<file>[^:]+):(?P<line>\d+):(?P<col>\d+):\s*(?P<sev>error|warning|note):\s*(?P<msg>.+)$
```

Each matched line produces a `Diagnostic` record.

---

## 5. Rust Plugin Design (`ff-rust-toolchain`)

### 5.1 Detection

`detect()` runs `rustc --version`, `cargo --version`, and `rustup --version` via
`std::process::Command`. The active toolchain channel is read from `rustup show active-toolchain`.

### 5.2 Installation

The official `rustup-init` installer is used on all platforms:

| Platform | Method |
|----------|--------|
| Windows | Download `rustup-init.exe` from `https://win.rustup.rs/x86_64`, execute with `--default-toolchain stable -y` |
| Linux/macOS | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh -s -- --default-toolchain stable -y` |

The download and execution are performed via `ff-bgio` background tasks. Progress is reported
through the `InstallProgress` channel.

After installation, `~/.cargo/bin` (Unix) or `%USERPROFILE%\.cargo\bin` (Windows) is added to
the PATH for the current process via `std::env::set_var("PATH", ...)` so that subsequent
`detect()` calls find the newly installed tools without requiring a restart.

### 5.3 Cargo Invocation

```
cargo <subcommand> --message-format=json [--manifest-path <Cargo.toml>]
```

The manifest path is found by walking up the directory tree from the active file until a
`Cargo.toml` is found (or the filesystem root is reached).

### 5.4 JSON Diagnostic Parser

`cargo --message-format=json` emits one JSON object per line. The plugin deserialises each line
using `serde_json` and extracts `compiler-message` objects:

```json
{
  "reason": "compiler-message",
  "message": {
    "level": "error|warning|note",
    "message": "...",
    "spans": [{ "file_name": "...", "line_start": N, "column_start": N }]
  }
}
```

Each `compiler-message` with at least one span produces a `Diagnostic` record.

---

## 6. Toolchain_Panel UI

The Toolchain_Panel is an egui panel rendered by `ff-desktop` when the Compilers menu is
activated. It is generic over `dyn ToolchainPlugin` and renders:

1. A status row per toolchain: icon + name + `ToolchainState` label + action button
2. A build output area: scrollable text with ANSI colour stripping
3. A diagnostics list: clickable rows that navigate the editor to the referenced location

The panel is docked to the bottom of the editor area by default (same zone as a terminal panel).

---

## 7. Data Flow — Build Cycle

```
User triggers "Compile" / "Cargo Build"
        │
        ▼
ToolchainPlugin::build(profile, sender)
        │
        ├── spawns background process via ff-bgio
        │
        ├── streams BuildEvent::OutputLine → Toolchain_Panel (raw output)
        │
        ├── streams BuildEvent::Diagnostic(d) → editor_panel (annotations)
        │
        └── BuildEvent::Finished(exit_code) → status line update
```

---

## 8. Dependencies

New crate dependencies (to be added to `Cargo.toml` of each plugin crate):

| Crate | Used by | Purpose |
|-------|---------|---------|
| `serde_json` | `ff-rust-toolchain` | Parse `cargo --message-format=json` output |
| `regex` | `ff-gcc-toolchain` | GCC diagnostic line parser |
| `reqwest` (optional, async) | both | Download rustup-init / fallback installers |
| `which` | both | Locate executables on PATH cross-platform |

The `reqwest` dependency is feature-gated (`toolchain-download`) so that builds without network
access do not pull in TLS dependencies.

---

## 9. Plugin Registration

Both plugin crates implement the existing `ff-plugin` `Plugin` trait in addition to
`ToolchainPlugin`. They register themselves in their `plugin_init()` entry point:

```rust
#[no_mangle]
pub extern "C" fn plugin_init(registry: &mut dyn PluginRegistry) {
    registry.register(Box::new(GccToolchainPlugin::new()));
}
```

This keeps the plugin loading mechanism identical to all other plugins in the system.
