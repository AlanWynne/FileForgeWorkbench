//! Platform shell detection and default shell resolution.
//!
//! Resolves the appropriate shell executable for the current platform,
//! following the fallback chain defined in Requirement 3.

use std::path::{Path, PathBuf};

use crate::error::ShellError;

/// Resolves the default shell executable for the current platform.
///
/// Implements the platform-specific detection logic:
/// - Windows: `cmd.exe` (Requirement 3.1)
/// - POSIX: `$SHELL` → `bash` → `sh` (Requirements 3.2, 3.3)
///
/// A configured `shell.default_shell` overrides all platform detection.
/// A shell override (first argument) overrides for a single invocation.
#[derive(Debug, Clone)]
pub struct PlatformDetector;

impl PlatformDetector {
    /// Resolves the shell executable to use for command execution.
    ///
    /// Priority order:
    /// 1. `shell_override` (per-invocation override from user input)
    /// 2. `configured_default` (from `shell.default_shell` config key)
    /// 3. Platform auto-detection
    ///
    /// # Errors
    ///
    /// Returns `ShellError::ShellNotFound` if no valid shell can be resolved.
    pub fn resolve_shell(
        shell_override: Option<&str>,
        configured_default: Option<&str>,
    ) -> Result<PathBuf, ShellError> {
        // Priority 1: per-invocation override
        if let Some(override_shell) = shell_override {
            return Self::validate_shell_path(override_shell);
        }

        // Priority 2: configured default
        if let Some(configured) = configured_default {
            return Self::validate_shell_path(configured);
        }

        // Priority 3: platform auto-detection
        Self::detect_platform_shell()
    }

    /// Detects the default shell for the current platform.
    fn detect_platform_shell() -> Result<PathBuf, ShellError> {
        #[cfg(windows)]
        {
            Self::detect_windows_shell()
        }
        #[cfg(unix)]
        {
            Self::detect_unix_shell()
        }
    }

    /// Windows shell detection: uses `cmd.exe`.
    #[cfg(windows)]
    fn detect_windows_shell() -> Result<PathBuf, ShellError> {
        // cmd.exe is always available on Windows
        let cmd = PathBuf::from("cmd.exe");
        Ok(cmd)
    }

    /// POSIX shell detection: `$SHELL` → `bash` → `sh`.
    #[cfg(unix)]
    fn detect_unix_shell() -> Result<PathBuf, ShellError> {
        // Try $SHELL environment variable first
        if let Ok(shell_env) = std::env::var("SHELL") {
            let path = PathBuf::from(&shell_env);
            if path.exists() && Self::is_executable(&path) {
                return Ok(path);
            }
        }

        // Fallback to bash
        if let Some(bash) = Self::find_on_path("bash") {
            return Ok(bash);
        }

        // Final fallback to sh
        if let Some(sh) = Self::find_on_path("sh") {
            return Ok(sh);
        }

        Err(ShellError::ShellNotFound {
            path: "no shell found (tried $SHELL, bash, sh)".to_string(),
        })
    }

    /// Validates a shell path — checks if it exists either as an absolute path
    /// or can be found on PATH.
    fn validate_shell_path(shell: &str) -> Result<PathBuf, ShellError> {
        let path = PathBuf::from(shell);

        // If absolute path, check existence
        if path.is_absolute() {
            if path.exists() {
                return Ok(path);
            }
            return Err(ShellError::ShellNotFound {
                path: shell.to_string(),
            });
        }

        // Otherwise, try to find on PATH
        if let Some(found) = Self::find_on_path(shell) {
            return Ok(found);
        }

        Err(ShellError::ShellNotFound {
            path: shell.to_string(),
        })
    }

    /// Searches PATH for an executable with the given name.
    fn find_on_path(name: &str) -> Option<PathBuf> {
        let path_var = std::env::var("PATH").ok()?;

        #[cfg(windows)]
        let separator = ';';
        #[cfg(unix)]
        let separator = ':';

        for dir in path_var.split(separator) {
            let candidate = PathBuf::from(dir).join(name);

            // On Windows, also check with .exe extension
            #[cfg(windows)]
            {
                if candidate.exists() {
                    return Some(candidate);
                }
                let with_exe = candidate.with_extension("exe");
                if with_exe.exists() {
                    return Some(with_exe);
                }
            }

            #[cfg(unix)]
            {
                if candidate.exists() && Self::is_executable(&candidate) {
                    return Some(candidate);
                }
            }
        }

        None
    }

    /// Checks if a path is executable (POSIX only — uses file metadata).
    #[cfg(unix)]
    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    /// Returns the default command-line arguments for invoking a command
    /// through the given shell.
    ///
    /// For example, `cmd.exe` uses `/C`, while bash/sh use `-c`.
    pub fn shell_command_args(shell_path: &Path) -> Vec<String> {
        let shell_name = shell_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match shell_name.as_str() {
            "cmd" => vec!["/C".to_string()],
            "powershell" | "pwsh" => vec!["-Command".to_string()],
            _ => vec!["-c".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.4
    #[test]
    fn configured_default_overrides_platform_detection() {
        // Use a shell that likely exists on all platforms
        #[cfg(windows)]
        let shell = "cmd.exe";
        #[cfg(unix)]
        let shell = "sh";

        let result = PlatformDetector::resolve_shell(None, Some(shell));
        assert!(result.is_ok());
    }

    // Validates: Requirement 3.5
    #[test]
    fn shell_override_takes_highest_priority() {
        #[cfg(windows)]
        let override_shell = "cmd.exe";
        #[cfg(unix)]
        let override_shell = "sh";

        let result =
            PlatformDetector::resolve_shell(Some(override_shell), Some("nonexistent_shell"));
        assert!(result.is_ok());
    }

    // Validates: Requirement 3.6
    #[test]
    fn nonexistent_shell_returns_not_found() {
        let result =
            PlatformDetector::resolve_shell(Some("/nonexistent/path/to/shell_xyz_123"), None);
        assert!(matches!(result, Err(ShellError::ShellNotFound { .. })));
    }

    // Validates: Requirement 3.1 (Windows) / 3.2-3.3 (POSIX)
    #[test]
    fn platform_detection_finds_default_shell() {
        let result = PlatformDetector::resolve_shell(None, None);
        assert!(
            result.is_ok(),
            "Platform detection should find a default shell"
        );
        let path = result.unwrap();
        assert!(!path.as_os_str().is_empty());
    }

    // Validates: Requirement 3
    #[test]
    fn shell_command_args_for_cmd() {
        let path = PathBuf::from("cmd.exe");
        let args = PlatformDetector::shell_command_args(&path);
        assert_eq!(args, vec!["/C"]);
    }

    // Validates: Requirement 3
    #[test]
    fn shell_command_args_for_bash() {
        let path = PathBuf::from("/bin/bash");
        let args = PlatformDetector::shell_command_args(&path);
        assert_eq!(args, vec!["-c"]);
    }

    // Validates: Requirement 3
    #[test]
    fn shell_command_args_for_powershell() {
        let path = PathBuf::from("pwsh");
        let args = PlatformDetector::shell_command_args(&path);
        assert_eq!(args, vec!["-Command"]);
    }
}
