//! Process handle and lifecycle management.
//!
//! Tracks running processes, manages state transitions, and provides
//! status querying for the shell subsystem.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Opaque identifier for a running shell process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(u64);

impl ProcessId {
    /// Creates a new unique process ID.
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the raw numeric value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for ProcessId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Process({})", self.0)
    }
}

/// Lifecycle state of a shell process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is currently running.
    Running,
    /// Process completed normally with an exit code.
    Exited(i32),
    /// Process was terminated by a signal (POSIX) or force-killed (Windows).
    Signalled(i32),
    /// Process was cancelled by user action.
    Cancelled,
    /// Process was terminated due to timeout.
    TimedOut,
}

impl ProcessState {
    /// Returns true if the process is still running.
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns true if the process has finished (any terminal state).
    pub fn is_finished(&self) -> bool {
        !self.is_running()
    }
}

/// Structured exit information for a completed process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitStatus {
    /// Exit code (None if killed by signal).
    pub code: Option<i32>,
    /// Signal number (POSIX) or None (Windows / normal exit).
    pub signal: Option<i32>,
    /// Whether the process was force-terminated.
    pub force_killed: bool,
}

impl ExitStatus {
    /// Creates an exit status from a normal exit with code.
    pub fn from_code(code: i32) -> Self {
        Self {
            code: Some(code),
            signal: None,
            force_killed: false,
        }
    }

    /// Creates an exit status from a signal termination.
    pub fn from_signal(signal: i32) -> Self {
        Self {
            code: None,
            signal: Some(signal),
            force_killed: false,
        }
    }

    /// Creates an exit status for a force-killed process.
    pub fn force_killed() -> Self {
        Self {
            code: None,
            signal: None,
            force_killed: true,
        }
    }

    /// Creates an exit status for a cancelled process.
    pub fn cancelled() -> Self {
        Self {
            code: None,
            signal: None,
            force_killed: false,
        }
    }

    /// Returns true if the process exited successfully (code 0).
    pub fn is_success(&self) -> bool {
        self.code == Some(0)
    }

    /// Returns true if at least one of code, signal, or force_killed is set.
    pub fn is_determined(&self) -> bool {
        self.code.is_some() || self.signal.is_some() || self.force_killed
    }
}

/// Represents a running or completed child process.
///
/// Provides lifecycle management, status querying, and metadata access.
#[derive(Debug)]
pub struct ProcessHandle {
    /// Unique identifier for this process instance.
    id: ProcessId,
    /// The command string as entered by the user.
    command_text: String,
    /// The resolved shell executable used.
    shell_executable: PathBuf,
    /// Current process state.
    state: ProcessState,
    /// Timestamp when the process was spawned.
    started_at: std::time::Instant,
    /// Exit status (populated after process terminates).
    exit_status: Option<ExitStatus>,
}

impl ProcessHandle {
    /// Creates a new process handle in the Running state.
    pub fn new(id: ProcessId, command_text: String, shell_executable: PathBuf) -> Self {
        Self {
            id,
            command_text,
            shell_executable,
            state: ProcessState::Running,
            started_at: std::time::Instant::now(),
            exit_status: None,
        }
    }

    /// Returns the process ID.
    pub fn id(&self) -> ProcessId {
        self.id
    }

    /// Returns the command text.
    pub fn command_text(&self) -> &str {
        &self.command_text
    }

    /// Returns the shell executable path.
    pub fn shell_executable(&self) -> &PathBuf {
        &self.shell_executable
    }

    /// Returns the current process state.
    pub fn state(&self) -> ProcessState {
        self.state
    }

    /// Returns the exit status, if the process has finished.
    pub fn exit_status(&self) -> Option<&ExitStatus> {
        self.exit_status.as_ref()
    }

    /// Returns elapsed time since the process was spawned.
    pub fn elapsed(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }

    /// Marks the process as exited with the given code.
    pub fn set_exited(&mut self, code: i32) {
        self.state = ProcessState::Exited(code);
        self.exit_status = Some(ExitStatus::from_code(code));
    }

    /// Marks the process as terminated by signal.
    pub fn set_signalled(&mut self, signal: i32) {
        self.state = ProcessState::Signalled(signal);
        self.exit_status = Some(ExitStatus::from_signal(signal));
    }

    /// Marks the process as cancelled by user action.
    pub fn set_cancelled(&mut self) {
        self.state = ProcessState::Cancelled;
        self.exit_status = Some(ExitStatus::cancelled());
    }

    /// Marks the process as timed out.
    pub fn set_timed_out(&mut self) {
        self.state = ProcessState::TimedOut;
        self.exit_status = Some(ExitStatus::force_killed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 13
    #[test]
    fn new_process_handle_is_running() {
        let handle = ProcessHandle::new(
            ProcessId::new(),
            "echo hello".to_string(),
            PathBuf::from("/bin/bash"),
        );
        assert!(handle.state().is_running());
        assert!(!handle.state().is_finished());
        assert!(handle.exit_status().is_none());
    }

    // Validates: Requirement 17.1
    #[test]
    fn set_exited_transitions_to_exited_state() {
        let mut handle = ProcessHandle::new(
            ProcessId::new(),
            "echo hello".to_string(),
            PathBuf::from("/bin/bash"),
        );
        handle.set_exited(0);
        assert_eq!(handle.state(), ProcessState::Exited(0));
        assert!(handle.exit_status().unwrap().is_success());
    }

    // Validates: Requirement 17.3
    #[test]
    fn non_zero_exit_is_not_success() {
        let mut handle = ProcessHandle::new(
            ProcessId::new(),
            "false".to_string(),
            PathBuf::from("/bin/bash"),
        );
        handle.set_exited(1);
        assert!(!handle.exit_status().unwrap().is_success());
    }

    // Validates: Requirement 17.4
    #[test]
    fn set_signalled_records_signal() {
        let mut handle = ProcessHandle::new(
            ProcessId::new(),
            "sleep 100".to_string(),
            PathBuf::from("/bin/bash"),
        );
        handle.set_signalled(9);
        assert_eq!(handle.state(), ProcessState::Signalled(9));
        assert_eq!(handle.exit_status().unwrap().signal, Some(9));
    }

    // Validates: Requirement 13.4
    #[test]
    fn set_cancelled_records_cancellation() {
        let mut handle = ProcessHandle::new(
            ProcessId::new(),
            "sleep 100".to_string(),
            PathBuf::from("/bin/bash"),
        );
        handle.set_cancelled();
        assert_eq!(handle.state(), ProcessState::Cancelled);
    }

    // Validates: Requirement 17.5
    #[test]
    fn exit_status_is_always_determined_for_finished_processes() {
        let mut handle = ProcessHandle::new(
            ProcessId::new(),
            "test".to_string(),
            PathBuf::from("/bin/sh"),
        );

        handle.set_exited(0);
        assert!(handle.exit_status().unwrap().is_determined());

        let mut handle2 = ProcessHandle::new(
            ProcessId::new(),
            "test".to_string(),
            PathBuf::from("/bin/sh"),
        );
        handle2.set_timed_out();
        assert!(handle2.exit_status().unwrap().is_determined());
    }

    // Validates: Requirement 13
    #[test]
    fn process_id_is_unique() {
        let id1 = ProcessId::new();
        let id2 = ProcessId::new();
        assert_ne!(id1, id2);
    }
}
