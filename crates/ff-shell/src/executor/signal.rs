//! Signal delivery for process termination.
//!
//! Provides cross-platform process termination with escalation:
//! SIGTERM → wait 5s → SIGKILL (POSIX) or TerminateProcess (Windows).

use std::time::Duration;

use crate::error::ShellError;

/// The grace period between SIGTERM and SIGKILL during escalation.
pub const ESCALATION_WAIT: Duration = Duration::from_secs(5);

/// Terminates a process by its OS PID with escalation.
///
/// On POSIX: sends SIGTERM, waits up to 5 seconds for exit, then sends SIGKILL.
/// On Windows: calls TerminateProcess (immediate).
///
/// # Arguments
///
/// * `pid` - The OS process ID to terminate.
///
/// # Errors
///
/// Returns `ShellError::IoError` if signal delivery fails.
pub async fn terminate_process(pid: u32) -> Result<(), ShellError> {
    #[cfg(unix)]
    {
        terminate_unix(pid).await
    }
    #[cfg(windows)]
    {
        terminate_windows(pid)
    }
}

/// POSIX termination with SIGTERM → SIGKILL escalation.
#[cfg(unix)]
async fn terminate_unix(pid: u32) -> Result<(), ShellError> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    let nix_pid = Pid::from_raw(pid as i32);

    // Send SIGTERM
    signal::kill(nix_pid, Signal::SIGTERM).map_err(|e| {
        ShellError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    // Wait for the process to exit
    tokio::time::sleep(ESCALATION_WAIT).await;

    // Check if still running, send SIGKILL
    if signal::kill(nix_pid, None).is_ok() {
        signal::kill(nix_pid, Signal::SIGKILL).map_err(|e| {
            ShellError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
    }

    Ok(())
}

/// Windows termination via TerminateProcess.
#[cfg(windows)]
fn terminate_windows(pid: u32) -> Result<(), ShellError> {
    use std::process::Command;

    // Use taskkill as a simple cross-version approach
    let output = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output()
        .map_err(ShellError::IoError)?;

    if !output.status.success() {
        return Err(ShellError::IoError(std::io::Error::other(format!(
            "taskkill failed for PID {}",
            pid
        ))));
    }

    Ok(())
}
