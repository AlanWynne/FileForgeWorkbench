/// Shared abstractions for FileForge compiler toolchain plugins.
///
/// This crate defines the `ToolchainPlugin` trait and all supporting types
/// (`ToolchainState`, `Diagnostic`, `DiagnosticSeverity`, `BuildProfile`,
/// `BuildEvent`, `InstallProgress`) that are shared between `ff-gcc-toolchain`
/// and `ff-rust-toolchain`.
use std::path::PathBuf;
use std::sync::mpsc;

// ── ToolchainState ────────────────────────────────────────────────────────────

/// The detected lifecycle state of a compiler toolchain.
///
/// Transitions: `NotDetected` → `Installing` → `Ready` | `InstallFailed`.
/// `Detected` is an intermediate state used when components are found but
/// not yet fully validated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToolchainState {
    /// No toolchain components were found on PATH.
    NotDetected,
    /// Components found; version string captured but not yet fully validated.
    Detected { version: String },
    /// Installation is in progress (background task running).
    Installing,
    /// Installation failed; `reason` contains the human-readable cause.
    InstallFailed { reason: String },
    /// All required components are present and validated.
    Ready { version: String },
}

// ── DiagnosticSeverity ────────────────────────────────────────────────────────

/// Severity level of a compiler diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
}

// ── Diagnostic ────────────────────────────────────────────────────────────────

/// A single compiler-emitted diagnostic (error, warning, or note).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Source file the diagnostic refers to.
    pub file: PathBuf,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

impl Diagnostic {
    /// Construct a new `Diagnostic`.
    pub fn new(
        file: impl Into<PathBuf>,
        line: u32,
        column: u32,
        severity: DiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            severity,
            message: message.into(),
        }
    }
}

// ── BuildProfile ──────────────────────────────────────────────────────────────

/// A named set of compiler flags used for a build invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildProfile {
    pub name: String,
    pub flags: Vec<String>,
}

impl BuildProfile {
    /// Create a new `BuildProfile`.
    pub fn new(
        name: impl Into<String>,
        flags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            flags: flags.into_iter().map(Into::into).collect(),
        }
    }
}

// ── BuildEvent ────────────────────────────────────────────────────────────────

/// Events streamed from a running build process to the UI.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum BuildEvent {
    /// A raw output line from the compiler's stdout/stderr.
    OutputLine(String),
    /// A parsed diagnostic extracted from compiler output.
    Diagnostic(Diagnostic),
    /// The build process exited with the given exit code.
    Finished(i32),
}

// ── InstallProgress ───────────────────────────────────────────────────────────

/// Progress events streamed from a running toolchain installation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InstallProgress {
    /// Installation has started.
    Started,
    /// An intermediate progress message.
    Progress { message: String },
    /// Installation completed successfully.
    Completed,
    /// Installation failed with the given reason.
    Failed { reason: String },
}

// ── ToolchainPlugin ───────────────────────────────────────────────────────────

/// Trait implemented by every compiler toolchain plugin.
///
/// # Errors
/// `detect()` and `build()` are infallible at the trait level; errors are
/// communicated through `ToolchainState` and `BuildEvent::Finished` respectively.
pub trait ToolchainPlugin: Send + Sync {
    /// Human-readable name of this toolchain (e.g. `"GCC"`, `"Rust"`).
    fn name(&self) -> &str;

    /// Current lifecycle state of this toolchain.
    fn state(&self) -> ToolchainState;

    /// Probe the system for this toolchain and update internal state.
    ///
    /// This method is expected to be called on a background thread.
    fn detect(&mut self);

    /// Begin installing the toolchain, reporting progress via `sender`.
    ///
    /// Transitions state to `Installing` immediately; the background task
    /// transitions to `Ready` or `InstallFailed` when done.
    fn install(&mut self, sender: mpsc::Sender<InstallProgress>);

    /// Invoke the compiler with the given `profile`, streaming events via `sender`.
    fn build(&self, profile: &BuildProfile, sender: mpsc::Sender<BuildEvent>);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ToolchainState tests ──────────────────────────────────────────────────

    #[test]
    fn toolchain_state_not_detected_is_distinct_from_ready() {
        // Validates: Requirement 15.2, 15.3 — NotDetected and Ready are separate states
        let not_detected = ToolchainState::NotDetected;
        let ready = ToolchainState::Ready {
            version: "13.2.0".into(),
        };
        assert_ne!(not_detected, ready);
    }

    #[test]
    fn toolchain_state_install_failed_carries_reason() {
        // Validates: Requirement 15.7, 17.7 — InstallFailed holds the failure reason
        let state = ToolchainState::InstallFailed {
            reason: "network timeout".into(),
        };
        match state {
            ToolchainState::InstallFailed { reason } => assert_eq!(reason, "network timeout"),
            other => panic!("unexpected state: {other:?}"),
        }
    }

    #[test]
    fn toolchain_state_detected_carries_version() {
        // Validates: Requirement 15.2, 17.2 — Detected state holds version string
        let state = ToolchainState::Detected {
            version: "1.78.0".into(),
        };
        match state {
            ToolchainState::Detected { version } => assert_eq!(version, "1.78.0"),
            other => panic!("unexpected state: {other:?}"),
        }
    }

    #[test]
    fn toolchain_state_ready_carries_version() {
        // Validates: Requirement 15.2, 17.2 — Ready state holds version string
        let state = ToolchainState::Ready {
            version: "GCC 13.2.0".into(),
        };
        match state {
            ToolchainState::Ready { version } => assert_eq!(version, "GCC 13.2.0"),
            other => panic!("unexpected state: {other:?}"),
        }
    }

    // ── Diagnostic tests ──────────────────────────────────────────────────────

    #[test]
    fn diagnostic_new_stores_all_fields() {
        // Validates: Requirement 16.3, 18.3 — Diagnostic captures file/line/col/severity/message
        let d = Diagnostic::new(
            "src/main.c",
            42,
            7,
            DiagnosticSeverity::Error,
            "use of undeclared identifier",
        );
        assert_eq!(d.file, PathBuf::from("src/main.c"));
        assert_eq!(d.line, 42);
        assert_eq!(d.column, 7);
        assert_eq!(d.severity, DiagnosticSeverity::Error);
        assert_eq!(d.message, "use of undeclared identifier");
    }

    #[test]
    fn diagnostic_severity_variants_are_distinct() {
        // Validates: Requirement 16.3, 18.3 — three severity levels are distinguishable
        assert_ne!(DiagnosticSeverity::Error, DiagnosticSeverity::Warning);
        assert_ne!(DiagnosticSeverity::Warning, DiagnosticSeverity::Note);
        assert_ne!(DiagnosticSeverity::Error, DiagnosticSeverity::Note);
    }

    // ── BuildProfile tests ────────────────────────────────────────────────────

    #[test]
    fn build_profile_debug_flags() {
        // Validates: Requirement 16.6 — debug profile has -g -O0 -Wall -Wextra
        let profile = BuildProfile::new("debug", ["-g", "-O0", "-Wall", "-Wextra"]);
        assert_eq!(profile.name, "debug");
        assert_eq!(profile.flags, vec!["-g", "-O0", "-Wall", "-Wextra"]);
    }

    #[test]
    fn build_profile_release_flags() {
        // Validates: Requirement 16.6 — release profile has -O2 -DNDEBUG
        let profile = BuildProfile::new("release", ["-O2", "-DNDEBUG"]);
        assert_eq!(profile.name, "release");
        assert_eq!(profile.flags, vec!["-O2", "-DNDEBUG"]);
    }

    #[test]
    fn build_profile_check_only_flags() {
        // Validates: Requirement 16.6 — check-only profile has -fsyntax-only -Wall -Wextra
        let profile = BuildProfile::new("check-only", ["-fsyntax-only", "-Wall", "-Wextra"]);
        assert_eq!(profile.name, "check-only");
        assert_eq!(profile.flags, vec!["-fsyntax-only", "-Wall", "-Wextra"]);
    }
}
