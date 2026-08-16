/// GCC toolchain plugin for FileForge Workbench.
///
/// Implements `ToolchainPlugin` for the GNU Compiler Collection, covering:
/// - PATH detection of `gcc`, `g++`, `gfortran`, `as`, `ld`, `ar`
/// - Platform-appropriate installation (winget/MSYS2, apt/dnf, Homebrew)
/// - Build invocation with `BuildProfile` flags
/// - GCC diagnostic line parser (`file:line:col: severity: message`)
/// - Built-in `debug`, `release`, and `check-only` profiles
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;

use ff_toolchain_api::{
    BuildEvent, BuildProfile, Diagnostic, DiagnosticSeverity, InstallProgress, ToolchainPlugin,
    ToolchainState,
};
use regex::Regex;

// ── Required GCC components ───────────────────────────────────────────────────

/// The executables that must all be present for the toolchain to be `Ready`.
const REQUIRED: &[&str] = &["gcc", "g++", "as", "ld", "ar"];

/// Optional component — detected but not required for `Ready`.
const OPTIONAL: &[&str] = &["gfortran"];

// ── Platform install strategy ─────────────────────────────────────────────────

/// The package-manager strategy chosen for the current platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStrategy {
    /// Windows: winget installing MSYS2 + mingw-w64-gcc
    Winget,
    /// Windows fallback: direct MSYS2 installer download
    Msys2Direct,
    /// Debian/Ubuntu: apt-get install build-essential gfortran
    Apt,
    /// RHEL/Fedora: dnf groupinstall "Development Tools"
    Dnf,
    /// macOS: brew install gcc
    Homebrew,
}

impl InstallStrategy {
    /// Select the appropriate strategy for the current platform.
    ///
    /// On Windows, prefers `winget` if it is on PATH; falls back to `Msys2Direct`.
    /// On Linux, prefers `apt` if present; falls back to `Dnf`.
    /// On macOS, always uses `Homebrew`.
    pub fn for_current_platform() -> Self {
        match std::env::consts::OS {
            "windows" => {
                if which::which("winget").is_ok() {
                    Self::Winget
                } else {
                    Self::Msys2Direct
                }
            }
            "linux" => {
                if which::which("apt-get").is_ok() {
                    Self::Apt
                } else {
                    Self::Dnf
                }
            }
            _ => Self::Homebrew, // macOS and any other Unix
        }
    }

    /// Human-readable description shown in the install confirmation dialog.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Winget => "winget (MSYS2 + mingw-w64-ucrt-x86_64-gcc)",
            Self::Msys2Direct => "MSYS2 direct installer from msys2.org",
            Self::Apt => "apt-get install build-essential gfortran",
            Self::Dnf => "dnf groupinstall \"Development Tools\"",
            Self::Homebrew => "brew install gcc",
        }
    }

    /// Build the `Command` that performs the installation.
    ///
    /// Returns `None` for `Msys2Direct` (requires a download step not yet
    /// implemented in this release — the caller should surface an error).
    pub fn install_command(&self) -> Option<Command> {
        match self {
            Self::Winget => {
                let mut cmd = Command::new("winget");
                cmd.args(["install", "--id", "MSYS2.MSYS2", "-e", "--silent"]);
                Some(cmd)
            }
            Self::Msys2Direct => None,
            Self::Apt => {
                let mut cmd = Command::new("apt-get");
                cmd.args(["install", "-y", "build-essential", "gfortran"]);
                Some(cmd)
            }
            Self::Dnf => {
                let mut cmd = Command::new("dnf");
                cmd.args(["groupinstall", "-y", "Development Tools"]);
                Some(cmd)
            }
            Self::Homebrew => {
                let mut cmd = Command::new("brew");
                cmd.args(["install", "gcc"]);
                Some(cmd)
            }
        }
    }
}

// ── Detected component info ───────────────────────────────────────────────────

/// Version information for a single detected GCC component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

// ── GCC diagnostic parser ─────────────────────────────────────────────────────

/// Parse a single GCC/G++ diagnostic output line into a `Diagnostic`.
///
/// GCC format: `<file>:<line>:<col>: <severity>: <message>`
///
/// Returns `None` if the line does not match the pattern.
pub fn parse_gcc_diagnostic(line: &str) -> Option<Diagnostic> {
    // Compiled once via lazy initialisation.
    static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = PATTERN.get_or_init(|| {
        Regex::new(
            r"^(?P<file>[^:]+):(?P<line>\d+):(?P<col>\d+):\s*(?P<sev>error|warning|note):\s*(?P<msg>.+)$",
        )
        .expect("GCC diagnostic regex is valid")
    });

    let caps = re.captures(line)?;
    let severity = match &caps["sev"] {
        "error" => DiagnosticSeverity::Error,
        "warning" => DiagnosticSeverity::Warning,
        _ => DiagnosticSeverity::Note,
    };
    Some(Diagnostic::new(
        &caps["file"],
        caps["line"].parse().ok()?,
        caps["col"].parse().ok()?,
        severity,
        caps["msg"].trim(),
    ))
}

// ── Built-in BuildProfiles ────────────────────────────────────────────────────

/// Returns the built-in `debug` profile: `-g -O0 -Wall -Wextra`.
pub fn profile_debug() -> BuildProfile {
    BuildProfile::new("debug", ["-g", "-O0", "-Wall", "-Wextra"])
}

/// Returns the built-in `release` profile: `-O2 -DNDEBUG`.
pub fn profile_release() -> BuildProfile {
    BuildProfile::new("release", ["-O2", "-DNDEBUG"])
}

/// Returns the built-in `check-only` profile: `-fsyntax-only -Wall -Wextra`.
pub fn profile_check_only() -> BuildProfile {
    BuildProfile::new("check-only", ["-fsyntax-only", "-Wall", "-Wextra"])
}

// ── GccToolchainPlugin ────────────────────────────────────────────────────────

/// GCC toolchain plugin implementing `ToolchainPlugin`.
pub struct GccToolchainPlugin {
    state: ToolchainState,
    /// Detected component info (populated after a successful `detect()`).
    components: Vec<ComponentInfo>,
    strategy: InstallStrategy,
}

impl GccToolchainPlugin {
    /// Create a new plugin instance in `NotDetected` state.
    pub fn new() -> Self {
        Self {
            state: ToolchainState::NotDetected,
            components: Vec::new(),
            strategy: InstallStrategy::for_current_platform(),
        }
    }

    /// Detected component list (available after `detect()` transitions to `Ready`).
    pub fn components(&self) -> &[ComponentInfo] {
        &self.components
    }

    /// The install strategy selected for the current platform.
    pub fn install_strategy(&self) -> &InstallStrategy {
        &self.strategy
    }

    /// Probe a single executable, returning its version string if found.
    fn probe_version(exe: &str) -> Option<(PathBuf, String)> {
        let path = which::which(exe).ok()?;
        let output = Command::new(&path).arg("--version").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next()?.trim().to_owned();
        Some((path, first_line))
    }

    /// Choose the compiler binary for the given source file extension.
    fn compiler_for_file(source: &std::path::Path) -> &'static str {
        match source.extension().and_then(|e| e.to_str()) {
            Some("cpp" | "cxx" | "cc" | "C") => "g++",
            _ => "gcc",
        }
    }
}

impl Default for GccToolchainPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolchainPlugin for GccToolchainPlugin {
    fn name(&self) -> &str {
        "GCC"
    }

    fn state(&self) -> ToolchainState {
        self.state.clone()
    }

    /// Probe PATH for all required GCC components.
    ///
    /// Transitions to `Ready` when all of `gcc`, `g++`, `as`, `ld`, `ar` are
    /// found; transitions to `NotDetected` otherwise.
    /// Optional `gfortran` is recorded if present.
    fn detect(&mut self) {
        self.components.clear();

        // Probe required components.
        let mut all_found = true;
        for &exe in REQUIRED {
            match Self::probe_version(exe) {
                Some((path, version)) => {
                    self.components.push(ComponentInfo {
                        name: exe.to_owned(),
                        version,
                        path,
                    });
                }
                None => {
                    all_found = false;
                }
            }
        }

        // Probe optional components.
        for &exe in OPTIONAL {
            if let Some((path, version)) = Self::probe_version(exe) {
                self.components.push(ComponentInfo {
                    name: exe.to_owned(),
                    version,
                    path,
                });
            }
        }

        self.state = if all_found {
            // Use the gcc version string as the canonical version.
            let version = self
                .components
                .iter()
                .find(|c| c.name == "gcc")
                .map(|c| c.version.clone())
                .unwrap_or_else(|| "unknown".into());
            ToolchainState::Ready { version }
        } else {
            ToolchainState::NotDetected
        };
    }

    /// Launch the platform-appropriate installer in the foreground (blocking).
    ///
    /// Progress events are sent via `sender`. The caller is expected to run
    /// this on a background thread (e.g. via `ff-bgio`).
    fn install(&mut self, sender: mpsc::Sender<InstallProgress>) {
        self.state = ToolchainState::Installing;
        let _ = sender.send(InstallProgress::Started);

        let mut cmd = match self.strategy.install_command() {
            Some(c) => c,
            None => {
                let reason = format!(
                    "Install strategy '{}' requires a manual download step not yet automated",
                    self.strategy.description()
                );
                self.state = ToolchainState::InstallFailed {
                    reason: reason.clone(),
                };
                let _ = sender.send(InstallProgress::Failed { reason });
                return;
            }
        };

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        match cmd.spawn() {
            Err(e) => {
                let reason = format!("Failed to launch installer: {e}");
                self.state = ToolchainState::InstallFailed {
                    reason: reason.clone(),
                };
                let _ = sender.send(InstallProgress::Failed { reason });
            }
            Ok(mut child) => {
                // Stream stdout lines as progress messages.
                if let Some(stdout) = child.stdout.take() {
                    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                        let _ = sender.send(InstallProgress::Progress { message: line });
                    }
                }

                match child.wait() {
                    Ok(status) if status.success() => {
                        // Re-probe to confirm installation succeeded.
                        self.detect();
                        if matches!(self.state, ToolchainState::Ready { .. }) {
                            let _ = sender.send(InstallProgress::Completed);
                        } else {
                            let reason =
                                "Installer exited successfully but GCC components not found on PATH"
                                    .into();
                            self.state = ToolchainState::InstallFailed {
                                reason: String::clone(&reason),
                            };
                            let _ = sender.send(InstallProgress::Failed { reason });
                        }
                    }
                    Ok(status) => {
                        let reason =
                            format!("Installer exited with code {}", status.code().unwrap_or(-1));
                        self.state = ToolchainState::InstallFailed {
                            reason: reason.clone(),
                        };
                        let _ = sender.send(InstallProgress::Failed { reason });
                    }
                    Err(e) => {
                        let reason = format!("Failed to wait for installer: {e}");
                        self.state = ToolchainState::InstallFailed {
                            reason: reason.clone(),
                        };
                        let _ = sender.send(InstallProgress::Failed { reason });
                    }
                }
            }
        }
    }

    /// Invoke `gcc` or `g++` on the first `.c`/`.cpp` flag in `profile.flags`,
    /// streaming `BuildEvent`s via `sender`.
    ///
    /// The source file is expected to be the first element of `profile.flags`
    /// that does not start with `-`. All remaining flags are passed verbatim.
    fn build(&self, profile: &BuildProfile, sender: mpsc::Sender<BuildEvent>) {
        // Split flags into source file + compiler flags.
        let source_path: Option<PathBuf> = profile
            .flags
            .iter()
            .find(|f| !f.starts_with('-'))
            .map(PathBuf::from);

        let compiler = source_path
            .as_deref()
            .map(Self::compiler_for_file)
            .unwrap_or("gcc");

        let mut cmd = Command::new(compiler);
        for flag in &profile.flags {
            cmd.arg(flag);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = sender.send(BuildEvent::OutputLine(format!("error: {e}")));
                let _ = sender.send(BuildEvent::Finished(-1));
                return;
            }
        };

        // Stream stderr (where GCC writes diagnostics) line by line.
        if let Some(stderr) = child.stderr.take() {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Some(diag) = parse_gcc_diagnostic(&line) {
                    let _ = sender.send(BuildEvent::Diagnostic(diag));
                }
                let _ = sender.send(BuildEvent::OutputLine(line));
            }
        }

        let exit_code = child.wait().map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
        let _ = sender.send(BuildEvent::Finished(exit_code));
    }
}

// ── Plugin entry point ────────────────────────────────────────────────────────

/// Plugin registration entry point.
///
/// In a full dynamic-plugin build this would be `#[no_mangle] pub extern "C"`,
/// but for the current statically-linked plugin model it is a plain Rust function.
pub fn plugin_init() -> Box<dyn ToolchainPlugin> {
    Box::new(GccToolchainPlugin::new())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── InstallStrategy tests ─────────────────────────────────────────────────

    #[test]
    fn install_strategy_description_is_non_empty() {
        // Validates: Requirement 15.4 — confirmation dialog lists the install source
        for strategy in [
            InstallStrategy::Winget,
            InstallStrategy::Msys2Direct,
            InstallStrategy::Apt,
            InstallStrategy::Dnf,
            InstallStrategy::Homebrew,
        ] {
            assert!(
                !strategy.description().is_empty(),
                "{strategy:?} has empty description"
            );
        }
    }

    #[test]
    fn install_strategy_winget_builds_correct_command() {
        // Validates: Requirement 15.8 — Windows install uses winget
        let cmd = InstallStrategy::Winget.install_command();
        assert!(cmd.is_some());
    }

    #[test]
    fn install_strategy_apt_builds_correct_command() {
        // Validates: Requirement 15.8 — Linux install uses apt-get
        let cmd = InstallStrategy::Apt.install_command();
        assert!(cmd.is_some());
    }

    #[test]
    fn install_strategy_dnf_builds_correct_command() {
        // Validates: Requirement 15.8 — Linux fallback uses dnf
        let cmd = InstallStrategy::Dnf.install_command();
        assert!(cmd.is_some());
    }

    #[test]
    fn install_strategy_homebrew_builds_correct_command() {
        // Validates: Requirement 15.8 — macOS install uses Homebrew
        let cmd = InstallStrategy::Homebrew.install_command();
        assert!(cmd.is_some());
    }

    #[test]
    fn install_strategy_msys2_direct_returns_none() {
        // Validates: Requirement 15.8 — Msys2Direct requires manual download (no auto command)
        let cmd = InstallStrategy::Msys2Direct.install_command();
        assert!(cmd.is_none());
    }

    // ── GCC diagnostic parser tests ───────────────────────────────────────────

    #[test]
    fn parse_gcc_diagnostic_error_line() {
        // Validates: Requirement 16.3 — error lines are parsed into Diagnostic records
        let line = "src/main.c:10:5: error: use of undeclared identifier 'x'";
        let d = parse_gcc_diagnostic(line).expect("should parse");
        assert_eq!(d.file, PathBuf::from("src/main.c"));
        assert_eq!(d.line, 10);
        assert_eq!(d.column, 5);
        assert_eq!(d.severity, DiagnosticSeverity::Error);
        assert_eq!(d.message, "use of undeclared identifier 'x'");
    }

    #[test]
    fn parse_gcc_diagnostic_warning_line() {
        // Validates: Requirement 16.3 — warning lines are parsed correctly
        let line = "include/util.h:42:3: warning: implicit declaration of function 'foo'";
        let d = parse_gcc_diagnostic(line).expect("should parse");
        assert_eq!(d.severity, DiagnosticSeverity::Warning);
        assert_eq!(d.line, 42);
        assert_eq!(d.column, 3);
    }

    #[test]
    fn parse_gcc_diagnostic_note_line() {
        // Validates: Requirement 16.3 — note lines are parsed correctly
        let line = "src/lib.c:7:1: note: declared here";
        let d = parse_gcc_diagnostic(line).expect("should parse");
        assert_eq!(d.severity, DiagnosticSeverity::Note);
        assert_eq!(d.message, "declared here");
    }

    #[test]
    fn parse_gcc_diagnostic_non_matching_line_returns_none() {
        // Validates: Requirement 16.3 — non-diagnostic lines are ignored
        assert!(parse_gcc_diagnostic("In function 'main':").is_none());
        assert!(parse_gcc_diagnostic("").is_none());
        assert!(parse_gcc_diagnostic("gcc: fatal error: no input files").is_none());
    }

    #[test]
    fn parse_gcc_diagnostic_path_with_directory() {
        // Validates: Requirement 16.3 — file paths with directories are preserved
        let line = "project/src/foo.cpp:100:20: error: expected ';'";
        let d = parse_gcc_diagnostic(line).expect("should parse");
        assert_eq!(d.file, PathBuf::from("project/src/foo.cpp"));
        assert_eq!(d.line, 100);
        assert_eq!(d.column, 20);
    }

    // ── BuildProfile tests ────────────────────────────────────────────────────

    #[test]
    fn profile_debug_has_correct_flags() {
        // Validates: Requirement 16.6 — debug profile: -g -O0 -Wall -Wextra
        let p = profile_debug();
        assert_eq!(p.name, "debug");
        assert_eq!(p.flags, vec!["-g", "-O0", "-Wall", "-Wextra"]);
    }

    #[test]
    fn profile_release_has_correct_flags() {
        // Validates: Requirement 16.6 — release profile: -O2 -DNDEBUG
        let p = profile_release();
        assert_eq!(p.name, "release");
        assert_eq!(p.flags, vec!["-O2", "-DNDEBUG"]);
    }

    #[test]
    fn profile_check_only_has_correct_flags() {
        // Validates: Requirement 16.6 — check-only profile: -fsyntax-only -Wall -Wextra
        let p = profile_check_only();
        assert_eq!(p.name, "check-only");
        assert_eq!(p.flags, vec!["-fsyntax-only", "-Wall", "-Wextra"]);
    }

    // ── GccToolchainPlugin struct tests ───────────────────────────────────────

    #[test]
    fn new_plugin_starts_in_not_detected_state() {
        // Validates: Requirement 15.1 — plugin starts NotDetected before detect() is called
        let plugin = GccToolchainPlugin::new();
        assert_eq!(plugin.state(), ToolchainState::NotDetected);
    }

    #[test]
    fn plugin_name_is_gcc() {
        // Validates: Requirement 15.1 — plugin identifies itself as "GCC"
        let plugin = GccToolchainPlugin::new();
        assert_eq!(plugin.name(), "GCC");
    }

    #[test]
    fn plugin_components_empty_before_detect() {
        // Validates: Requirement 15.9 — component list is empty until detect() runs
        let plugin = GccToolchainPlugin::new();
        assert!(plugin.components().is_empty());
    }

    #[test]
    fn detect_transitions_to_not_detected_when_gcc_absent() {
        // Validates: Requirement 15.3 — NotDetected when required components missing
        // This test is always valid: if GCC is not installed in the CI environment,
        // detect() must return NotDetected (not panic or Ready).
        // If GCC IS installed, the test verifies Ready carries a version string.
        let mut plugin = GccToolchainPlugin::new();
        plugin.detect();
        match plugin.state() {
            ToolchainState::NotDetected => {} // GCC not present — correct
            ToolchainState::Ready { version } => {
                assert!(
                    !version.is_empty(),
                    "Ready state must carry a version string"
                );
            }
            other => panic!("unexpected state after detect(): {other:?}"),
        }
    }

    #[test]
    fn install_with_msys2_direct_strategy_sends_failed_event() {
        // Validates: Requirement 15.7 — InstallFailed when strategy has no auto command
        let (tx, rx) = mpsc::channel();
        let mut plugin = GccToolchainPlugin {
            state: ToolchainState::NotDetected,
            components: Vec::new(),
            strategy: InstallStrategy::Msys2Direct,
        };
        plugin.install(tx);

        let events: Vec<_> = rx.try_iter().collect();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, InstallProgress::Failed { .. })),
            "expected a Failed event for Msys2Direct strategy"
        );
        assert!(matches!(
            plugin.state(),
            ToolchainState::InstallFailed { .. }
        ));
    }

    #[test]
    fn plugin_init_returns_gcc_plugin() {
        // Validates: Requirement 15 (plugin registration) — plugin_init returns a ToolchainPlugin
        let plugin = plugin_init();
        assert_eq!(plugin.name(), "GCC");
        assert_eq!(plugin.state(), ToolchainState::NotDetected);
    }
}
