//! Async process spawning using `tokio::process::Command`.
//!
//! Spawns child processes with configured environment, working directory,
//! and shell arguments. Supports both piped output capture and interactive PTY modes.

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::error::ShellError;
use crate::executor::output::OutputCapture;
use crate::process::ExitStatus;

/// Executes shell commands asynchronously.
///
/// Spawns child processes using `tokio::process::Command` with full
/// environment and working directory configuration. Captures output
/// via piped stdout/stderr.
#[derive(Debug)]
pub struct CommandExecutor;

impl CommandExecutor {
    /// Spawns a command for execution with piped output.
    ///
    /// Returns the captured output and exit status after the process completes.
    ///
    /// # Arguments
    ///
    /// * `shell_path` - Path to the shell executable.
    /// * `shell_args` - Arguments to pass to the shell (e.g., `["-c"]`).
    /// * `command_text` - The user's command string.
    /// * `working_dir` - Working directory for the child process.
    /// * `env` - Environment variables for the child process.
    ///
    /// # Errors
    ///
    /// Returns `ShellError::SpawnFailed` if the process cannot be started.
    pub async fn execute(
        shell_path: &Path,
        shell_args: &[String],
        command_text: &str,
        working_dir: &Path,
        env: &HashMap<String, String>,
    ) -> Result<(OutputCapture, ExitStatus), ShellError> {
        let mut cmd = Command::new(shell_path);
        cmd.args(shell_args);
        cmd.arg(command_text);
        cmd.current_dir(working_dir);
        cmd.envs(env);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        let child = cmd.spawn().map_err(|e| ShellError::SpawnFailed {
            reason: e.to_string(),
        })?;

        let output = child
            .wait_with_output()
            .await
            .map_err(ShellError::IoError)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let stdout_lines: Vec<String> = if stdout.is_empty() {
            Vec::new()
        } else {
            stdout.lines().map(String::from).collect()
        };

        let stderr_lines: Vec<String> = if stderr.is_empty() {
            Vec::new()
        } else {
            stderr.lines().map(String::from).collect()
        };

        let capture = OutputCapture {
            stdout_lines,
            stderr_lines,
            is_streaming: false,
            bytes_received: output.stdout.len() + output.stderr.len(),
        };

        let exit_status = match output.status.code() {
            Some(code) => ExitStatus::from_code(code),
            None => {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    match output.status.signal() {
                        Some(sig) => ExitStatus::from_signal(sig),
                        None => ExitStatus::force_killed(),
                    }
                }
                #[cfg(windows)]
                {
                    ExitStatus::force_killed()
                }
            }
        };

        Ok((capture, exit_status))
    }

    /// Spawns a command with stdin piped from the given content.
    ///
    /// Writes `stdin_content` to the child's stdin, closes it (signalling EOF),
    /// then captures output.
    pub async fn execute_with_stdin(
        shell_path: &Path,
        shell_args: &[String],
        command_text: &str,
        stdin_content: &str,
        working_dir: &Path,
        env: &HashMap<String, String>,
    ) -> Result<(OutputCapture, ExitStatus), ShellError> {
        use tokio::io::AsyncWriteExt;

        let mut cmd = Command::new(shell_path);
        cmd.args(shell_args);
        cmd.arg(command_text);
        cmd.current_dir(working_dir);
        cmd.envs(env);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| ShellError::SpawnFailed {
            reason: e.to_string(),
        })?;

        // Write stdin content and close handle
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(stdin_content.as_bytes())
                .await
                .map_err(ShellError::IoError)?;
            // Drop closes the handle, signalling EOF
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(ShellError::IoError)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let stdout_lines: Vec<String> = if stdout.is_empty() {
            Vec::new()
        } else {
            stdout.lines().map(String::from).collect()
        };

        let stderr_lines: Vec<String> = if stderr.is_empty() {
            Vec::new()
        } else {
            stderr.lines().map(String::from).collect()
        };

        let capture = OutputCapture {
            stdout_lines,
            stderr_lines,
            is_streaming: false,
            bytes_received: output.stdout.len() + output.stderr.len(),
        };

        let exit_status = match output.status.code() {
            Some(code) => ExitStatus::from_code(code),
            None => {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    match output.status.signal() {
                        Some(sig) => ExitStatus::from_signal(sig),
                        None => ExitStatus::force_killed(),
                    }
                }
                #[cfg(windows)]
                {
                    ExitStatus::force_killed()
                }
            }
        };

        Ok((capture, exit_status))
    }
}
