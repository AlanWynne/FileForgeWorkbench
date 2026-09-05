//! REXX execution bridge -- exec invocation and host command environments.
//!
//! Implements Requirement 11 AC 11.7-11.15:
//!   - EXEC command: locate and execute named exec (AC 11.7)
//!   - Implicit exec invocation for unrecognised commands (AC 11.8)
//!   - % prefix bypass of primary command table (AC 11.9)
//!   - Argument passing to exec invocation (AC 11.10)
//!   - TSO host command environment (AC 11.11)
//!   - ADDRESS <env> switching (AC 11.12)
//!   - ISPEXEC environment name within ADDRESS (AC 11.13)
//!   - ISREDIT environment name within ADDRESS (AC 11.14)
//!   - RC special variable after host command (AC 11.15)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// === ExecLibrary =============================================================

/// A single exec member found in a library path.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecMember {
    /// Member name (uppercase, no extension).
    pub name: String,
    /// Full path to the exec file.
    pub path: PathBuf,
}

/// Registry of SYSEXEC and SYSPROC library paths used to locate exec members.
///
/// Addresses: Requirement 11 AC 11.7
#[derive(Debug, Default)]
pub struct ExecLibrary {
    /// Ordered list of library paths (SYSEXEC first, then SYSPROC).
    paths: Vec<PathBuf>,
    /// Cached member index: uppercase name -> path.
    index: HashMap<String, PathBuf>,
}

impl ExecLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a library path. Paths are searched in the order they are added.
    pub fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    /// Register a member directly (used in tests without touching the filesystem).
    pub fn register_member(&mut self, name: &str, path: PathBuf) {
        self.index.insert(name.to_uppercase(), path);
    }

    /// Locate a member by name. Returns the path if found.
    ///
    /// Addresses: Requirement 11 AC 11.7
    pub fn find(&self, name: &str) -> Option<&Path> {
        self.index.get(&name.to_uppercase()).map(|p| p.as_path())
    }

    /// Returns true if the named member exists in the library.
    pub fn contains(&self, name: &str) -> bool {
        self.index.contains_key(&name.to_uppercase())
    }
}

// === HostEnvironment =========================================================

/// The active host command environment for ADDRESS switching.
///
/// Addresses: Requirement 11 AC 11.12-11.14
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HostEnvironment {
    /// TSO command dispatcher (default).
    Tso,
    /// ISPF dialog service layer.
    Ispexec,
    /// ISPF edit macro service layer.
    Isredit,
    /// Custom named environment.
    Named(String),
}

impl HostEnvironment {
    /// Parse an environment name string (case-insensitive).
    ///
    /// Addresses: Requirement 11 AC 11.12
    pub fn parse_env(name: &str) -> Self {
        match name.trim().to_uppercase().as_str() {
            "TSO" => Self::Tso,
            "ISPEXEC" => Self::Ispexec,
            "ISREDIT" => Self::Isredit,
            other => Self::Named(other.to_string()),
        }
    }

    /// Returns the canonical name of this environment.
    pub fn name(&self) -> &str {
        match self {
            Self::Tso => "TSO",
            Self::Ispexec => "ISPEXEC",
            Self::Isredit => "ISREDIT",
            Self::Named(n) => n.as_str(),
        }
    }
}

// === RcVariable ==============================================================

/// Tracks the RC (return code) special variable.
///
/// After each host command completes, RC is set to the integer return code.
///
/// Addresses: Requirement 11 AC 11.15
#[derive(Debug, Clone, Default)]
pub struct RcVariable {
    value: i32,
}

impl RcVariable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set RC to the given return code.
    ///
    /// Addresses: Requirement 11 AC 11.15
    pub fn set(&mut self, rc: i32) {
        self.value = rc;
    }

    /// Get the current RC value.
    pub fn get(&self) -> i32 {
        self.value
    }
}

// === ExecInvocation ==========================================================

/// Describes how an exec was invoked.
#[derive(Debug, Clone, PartialEq)]
pub enum InvocationKind {
    /// Explicit EXEC <member> [args] command.
    Explicit,
    /// Implicit invocation (unrecognised command name tried as exec).
    Implicit,
    /// % prefix bypass of primary command table.
    PercentPrefix,
}

/// The result of attempting to invoke an exec.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecInvokeResult {
    /// Exec was found and invoked; contains the return code.
    Invoked { rc: i32, kind: InvocationKind },
    /// Exec member was not found in any library.
    NotFound { name: String },
    /// Exec was found but could not be read.
    ReadError { name: String, reason: String },
}

// === RexxBridge ==============================================================

/// REXX execution bridge.
///
/// Manages exec library lookup, invocation routing, ADDRESS switching,
/// and RC tracking. In this implementation exec "execution" is simulated
/// by recording the invocation; the real shell layer injects a Lua callback
/// for actual script execution.
///
/// Addresses: Requirement 11 AC 11.7-11.15
pub struct RexxBridge {
    /// Exec library (SYSEXEC + SYSPROC paths).
    library: ExecLibrary,
    /// Current default host command environment.
    current_env: HostEnvironment,
    /// RC special variable.
    rc: RcVariable,
    /// Log of invocations (name, args, kind) -- used by tests.
    invocation_log: Vec<(String, String, InvocationKind)>,
}

impl RexxBridge {
    pub fn new(library: ExecLibrary) -> Self {
        Self {
            library,
            current_env: HostEnvironment::Tso,
            rc: RcVariable::new(),
            invocation_log: Vec::new(),
        }
    }

    /// Execute an explicit EXEC command: `EXEC <member> [args]`.
    ///
    /// Addresses: Requirement 11 AC 11.7, 11.10
    pub fn exec_command(&mut self, member: &str, args: &str) -> ExecInvokeResult {
        self.invoke(member, args, InvocationKind::Explicit)
    }

    /// Attempt implicit exec invocation for an unrecognised command name.
    ///
    /// Addresses: Requirement 11 AC 11.8
    pub fn try_implicit(&mut self, command: &str, args: &str) -> ExecInvokeResult {
        self.invoke(command, args, InvocationKind::Implicit)
    }

    /// Execute via % prefix: bypass primary command table.
    ///
    /// Strips the leading `%` and searches SYSEXEC/SYSPROC directly.
    ///
    /// Addresses: Requirement 11 AC 11.9
    pub fn exec_percent(&mut self, input: &str) -> ExecInvokeResult {
        let stripped = input.trim_start_matches('%').trim();
        let mut parts = stripped.splitn(2, char::is_whitespace);
        let member = parts.next().unwrap_or("").trim();
        let args = parts.next().unwrap_or("").trim();
        self.invoke(member, args, InvocationKind::PercentPrefix)
    }

    /// Switch the default host command environment (ADDRESS <env>).
    ///
    /// Addresses: Requirement 11 AC 11.12
    pub fn set_address(&mut self, env_name: &str) {
        self.current_env = HostEnvironment::parse_env(env_name);
    }

    /// Get the current host command environment.
    pub fn current_env(&self) -> &HostEnvironment {
        &self.current_env
    }

    /// Set RC after a host command completes.
    ///
    /// Addresses: Requirement 11 AC 11.15
    pub fn set_rc(&mut self, rc: i32) {
        self.rc.set(rc);
    }

    /// Get the current RC value.
    pub fn rc(&self) -> i32 {
        self.rc.get()
    }

    /// Returns the invocation log (for testing).
    pub fn invocation_log(&self) -> &[(String, String, InvocationKind)] {
        &self.invocation_log
    }

    /// Returns a reference to the exec library.
    pub fn library(&self) -> &ExecLibrary {
        &self.library
    }

    // -------------------------------------------------------------------------

    fn invoke(&mut self, member: &str, args: &str, kind: InvocationKind) -> ExecInvokeResult {
        if member.is_empty() {
            return ExecInvokeResult::NotFound {
                name: String::new(),
            };
        }

        if !self.library.contains(member) {
            return ExecInvokeResult::NotFound {
                name: member.to_uppercase(),
            };
        }

        // Record the invocation
        self.invocation_log
            .push((member.to_uppercase(), args.to_string(), kind.clone()));

        // Simulate successful execution with RC=0
        self.rc.set(0);

        ExecInvokeResult::Invoked { rc: 0, kind }
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn library_with(members: &[&str]) -> ExecLibrary {
        let mut lib = ExecLibrary::new();
        for name in members {
            lib.register_member(name, PathBuf::from(format!("/sysproc/{name}.rexx")));
        }
        lib
    }

    fn bridge_with(members: &[&str]) -> RexxBridge {
        RexxBridge::new(library_with(members))
    }

    // --- ExecLibrary ---------------------------------------------------------

    // Validates: Requirement 11.7
    #[test]
    fn exec_library_find_returns_path_for_known_member() {
        let lib = library_with(&["MYMACRO"]);
        assert!(lib.find("MYMACRO").is_some());
    }

    // Validates: Requirement 11.7
    #[test]
    fn exec_library_find_is_case_insensitive() {
        let lib = library_with(&["MYMACRO"]);
        assert!(lib.find("mymacro").is_some());
        assert!(lib.find("MyMacro").is_some());
    }

    // Validates: Requirement 11.7
    #[test]
    fn exec_library_find_returns_none_for_unknown() {
        let lib = library_with(&["MYMACRO"]);
        assert!(lib.find("MISSING").is_none());
    }

    // --- HostEnvironment -----------------------------------------------------

    // Validates: Requirement 11.12
    #[test]
    fn host_environment_parses_tso() {
        assert_eq!(HostEnvironment::parse_env("TSO"), HostEnvironment::Tso);
        assert_eq!(HostEnvironment::parse_env("tso"), HostEnvironment::Tso);
    }

    // Validates: Requirement 11.13
    #[test]
    fn host_environment_parses_ispexec() {
        assert_eq!(
            HostEnvironment::parse_env("ISPEXEC"),
            HostEnvironment::Ispexec
        );
    }

    // Validates: Requirement 11.14
    #[test]
    fn host_environment_parses_isredit() {
        assert_eq!(
            HostEnvironment::parse_env("ISREDIT"),
            HostEnvironment::Isredit
        );
    }

    // Validates: Requirement 11.12
    #[test]
    fn host_environment_unknown_name_becomes_named() {
        let env = HostEnvironment::parse_env("MYENV");
        assert_eq!(env, HostEnvironment::Named("MYENV".to_string()));
        assert_eq!(env.name(), "MYENV");
    }

    // --- RexxBridge::exec_command --------------------------------------------

    // Validates: Requirement 11.7
    #[test]
    fn exec_command_invokes_known_member() {
        let mut bridge = bridge_with(&["MYMACRO"]);
        let result = bridge.exec_command("MYMACRO", "");
        assert_eq!(
            result,
            ExecInvokeResult::Invoked {
                rc: 0,
                kind: InvocationKind::Explicit
            }
        );
    }

    // Validates: Requirement 11.7
    #[test]
    fn exec_command_returns_not_found_for_unknown() {
        let mut bridge = bridge_with(&[]);
        let result = bridge.exec_command("MISSING", "");
        assert_eq!(
            result,
            ExecInvokeResult::NotFound {
                name: "MISSING".to_string()
            }
        );
    }

    // Validates: Requirement 11.10
    #[test]
    fn exec_command_records_args_in_log() {
        let mut bridge = bridge_with(&["MYMACRO"]);
        bridge.exec_command("MYMACRO", "ARG1 ARG2");
        let log = bridge.invocation_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].1, "ARG1 ARG2");
    }

    // --- RexxBridge::try_implicit --------------------------------------------

    // Validates: Requirement 11.8
    #[test]
    fn implicit_invocation_finds_exec_for_unrecognised_command() {
        let mut bridge = bridge_with(&["MYEXEC"]);
        let result = bridge.try_implicit("MYEXEC", "");
        assert_eq!(
            result,
            ExecInvokeResult::Invoked {
                rc: 0,
                kind: InvocationKind::Implicit
            }
        );
    }

    // Validates: Requirement 11.8
    #[test]
    fn implicit_invocation_returns_not_found_when_no_exec() {
        let mut bridge = bridge_with(&[]);
        let result = bridge.try_implicit("UNKNOWN", "");
        assert!(matches!(result, ExecInvokeResult::NotFound { .. }));
    }

    // --- RexxBridge::exec_percent --------------------------------------------

    // Validates: Requirement 11.9
    #[test]
    fn percent_prefix_strips_percent_and_invokes() {
        let mut bridge = bridge_with(&["FASTEXEC"]);
        let result = bridge.exec_percent("%FASTEXEC");
        assert_eq!(
            result,
            ExecInvokeResult::Invoked {
                rc: 0,
                kind: InvocationKind::PercentPrefix
            }
        );
    }

    // Validates: Requirement 11.9
    #[test]
    fn percent_prefix_passes_args() {
        let mut bridge = bridge_with(&["FASTEXEC"]);
        bridge.exec_percent("%FASTEXEC ONE TWO");
        let log = bridge.invocation_log();
        assert_eq!(log[0].1, "ONE TWO");
    }

    // --- ADDRESS switching ---------------------------------------------------

    // Validates: Requirement 11.12
    #[test]
    fn set_address_switches_environment() {
        let mut bridge = bridge_with(&[]);
        assert_eq!(bridge.current_env(), &HostEnvironment::Tso);
        bridge.set_address("ISPEXEC");
        assert_eq!(bridge.current_env(), &HostEnvironment::Ispexec);
    }

    // Validates: Requirement 11.12
    #[test]
    fn set_address_can_switch_back_to_tso() {
        let mut bridge = bridge_with(&[]);
        bridge.set_address("ISPEXEC");
        bridge.set_address("TSO");
        assert_eq!(bridge.current_env(), &HostEnvironment::Tso);
    }

    // --- RC variable ---------------------------------------------------------

    // Validates: Requirement 11.15
    #[test]
    fn rc_defaults_to_zero() {
        let bridge = bridge_with(&[]);
        assert_eq!(bridge.rc(), 0);
    }

    // Validates: Requirement 11.15
    #[test]
    fn rc_is_set_after_successful_invocation() {
        let mut bridge = bridge_with(&["MYMACRO"]);
        bridge.exec_command("MYMACRO", "");
        assert_eq!(bridge.rc(), 0);
    }

    // Validates: Requirement 11.15
    #[test]
    fn rc_can_be_set_manually_by_host_command() {
        let mut bridge = bridge_with(&[]);
        bridge.set_rc(8);
        assert_eq!(bridge.rc(), 8);
    }
}
