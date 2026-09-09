//! External program execution primitives (Detached and Captured modes).
//!
//! Provides the explicit-program execution model of shell-command
//! Requirement 19: run a program by name + argument list (not a shell string)
//! either fire-and-forget (Detached) or with captured output (Captured).
//!
//! The Detached spawn seam lives here; Captured runs reuse
//! [`crate::executor::spawn::CommandExecutor`] and the Output_Panel via the
//! [`crate::engine::ShellEngine`].

use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::ShellError;
use crate::process::{ExitStatus, ProcessId};

/// Selects how an external program is run.
///
/// Validates: shell-command Requirement 19.1, command-configurator Requirement 3.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Fire-and-forget: no capture, no panel, return immediately.
    Detached,
    /// Async run whose stdout/stderr and exit code go to the Output_Panel.
    Captured,
}

/// Opaque handle to a Detached (Started_Task) process.
///
/// The workbench does NOT track, monitor, restart, or persist the process
/// (shell-command Requirement 19.4); the operating system owns its lifecycle
/// after spawn. The handle is a named seam for possible future Started_Task
/// monitoring and may be dropped without affecting correctness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskHandle {
    /// The OS process id of the spawned child, if the platform reports one.
    pub pid: Option<u32>,
}

/// Result of an external execution via `ShellEngine::execute_external`.
///
/// Validates: shell-command Requirement 19.2, 19.3
#[derive(Debug)]
pub enum ExternalOutcome {
    /// A Detached spawn succeeded; the handle is opaque and may be dropped.
    Detached(TaskHandle),
    /// A Captured run completed; output was appended to the Output_Panel.
    Captured {
        /// The process identity for this run.
        process_id: ProcessId,
        /// The exit status of the captured process.
        exit_status: ExitStatus,
    },
}

/// Spawns an external program fire-and-forget (Detached mode).
///
/// Uses the synchronous `std::process::Command` so the child is not tied to a
/// captured pipe or the async runtime. Standard streams are redirected to null
/// and the child handle is dropped immediately after spawn -- the OS reaps it.
///
/// # Arguments
///
/// * `program` - The program name or path to execute.
/// * `args` - Argument list passed verbatim (no shell parsing).
/// * `working_dir` - The already-resolved working directory for the child.
/// * `env` - Environment variables injected into the child.
///
/// # Errors
///
/// Returns [`ShellError::SpawnFailed`] if the program cannot be launched
/// (not found, permission denied). Validates: shell-command Requirement 19.8.
pub fn spawn_detached(
    program: &str,
    args: &[String],
    working_dir: &Path,
    env: &HashMap<String, String>,
) -> Result<TaskHandle, ShellError> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.current_dir(working_dir);
    cmd.envs(env);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    let child = cmd.spawn().map_err(|e| ShellError::SpawnFailed {
        reason: format!("{}: {}", program, e),
    })?;

    // Capture the pid before dropping the handle; the OS owns lifecycle now.
    let handle = TaskHandle {
        pid: Some(child.id()),
    };
    // `child` drops here: for a Detached process we do not wait or track it.
    drop(child);
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Returns a program that exits immediately on the host platform.
    fn noop_program() -> (&'static str, Vec<String>) {
        if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "exit".to_string()])
        } else {
            ("true", Vec::new())
        }
    }

    // Validates: shell-command Requirement 19.1
    #[test]
    fn execution_mode_variants_are_distinct() {
        assert_ne!(ExecutionMode::Detached, ExecutionMode::Captured);
    }

    // Validates: shell-command Requirement 19.3, 19.4, command-configurator Req 3.2
    #[test]
    fn spawn_detached_returns_handle_without_waiting() {
        let temp = TempDir::new().unwrap();
        let (program, args) = noop_program();
        let env = HashMap::new();

        let handle = spawn_detached(program, &args, temp.path(), &env)
            .expect("detached spawn of a trivial program should succeed");

        // A pid is reported and the call returned without blocking on exit.
        assert!(handle.pid.is_some());
    }

    // Validates: shell-command Requirement 19.8
    #[test]
    fn spawn_detached_missing_program_reports_error() {
        let temp = TempDir::new().unwrap();
        let env = HashMap::new();
        let missing = "ffwb_nonexistent_program_db10";

        let result = spawn_detached(missing, &[], temp.path(), &env);

        match result {
            Err(ShellError::SpawnFailed { reason }) => {
                assert!(reason.contains(missing), "error should name the program");
            }
            other => panic!("expected SpawnFailed, got {:?}", other),
        }
    }
}
