//! Command-line argument parsing and validation.
//!
//! Handles positional file path/VFS URI arguments and named flags
//! (--new-window, --no-session-restore, --profile, --project, --log-level).
//!
//! Addresses: Requirement 6 (Command-Line Argument Handling)

use std::path::{Path, PathBuf};

use crate::SessionError;

/// Parsed command-line arguments for the workbench.
///
/// Positional arguments are file paths or VFS URIs to open.
/// Named flags control startup behaviour.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CliArgs {
    /// Positional file paths or VFS URIs to open.
    pub source_args: Vec<String>,

    /// `--new-window`: force a new workbench instance.
    pub new_window: bool,

    /// `--no-session-restore`: suppress session restore for this invocation.
    pub no_session_restore: bool,

    /// `--profile <name>`: activate a specific configuration profile.
    pub profile: Option<String>,

    /// `--project <path>`: set the project root directory.
    pub project: Option<PathBuf>,

    /// `--log-level <level>`: override configured log level.
    pub log_level: Option<String>,
}

/// The VFS URI scheme prefix used to detect VFS URIs.
const VFS_SCHEME: &str = "vfs://";

impl CliArgs {
    /// Parse command-line arguments from the given iterator of strings.
    ///
    /// This is the internal parse function that operates on an arbitrary
    /// iterator, making it testable without modifying process args.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::CliArgInvalid` when a flag requires a value
    /// that was not provided (e.g., `--profile` without a name).
    pub fn parse_from<I, S>(args: I) -> Result<Self, SessionError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut result = Self::default();
        let mut iter = args.into_iter().peekable();

        while let Some(arg) = iter.next() {
            let arg_str = arg.as_ref();

            match arg_str {
                "--new-window" => {
                    result.new_window = true;
                }
                "--no-session-restore" => {
                    result.no_session_restore = true;
                }
                "--profile" => {
                    let value = iter.next().ok_or_else(|| SessionError::CliArgInvalid {
                        argument: "--profile".to_string(),
                        reason: "requires a profile name argument".to_string(),
                    })?;
                    result.profile = Some(value.as_ref().to_string());
                }
                "--project" => {
                    let value = iter.next().ok_or_else(|| SessionError::CliArgInvalid {
                        argument: "--project".to_string(),
                        reason: "requires a project path argument".to_string(),
                    })?;
                    result.project = Some(PathBuf::from(value.as_ref()));
                }
                "--log-level" => {
                    let value = iter.next().ok_or_else(|| SessionError::CliArgInvalid {
                        argument: "--log-level".to_string(),
                        reason: "requires a log level argument".to_string(),
                    })?;
                    result.log_level = Some(value.as_ref().to_string());
                }
                other => {
                    if other.starts_with("--") {
                        return Err(SessionError::CliArgInvalid {
                            argument: other.to_string(),
                            reason: "unrecognised flag".to_string(),
                        });
                    }
                    // Positional argument — file path or VFS URI
                    result.source_args.push(other.to_string());
                }
            }
        }

        Ok(result)
    }

    /// Whether any source arguments (file paths or VFS URIs) were provided.
    pub fn has_source_args(&self) -> bool {
        !self.source_args.is_empty()
    }

    /// Resolve relative source arguments against the given working directory.
    ///
    /// VFS URIs (starting with `vfs://`) are left unchanged — only filesystem
    /// paths are resolved.
    ///
    /// Addresses: Requirement 6 AC 6.2, 6.3
    pub fn resolve_source_args(&mut self, working_dir: &Path) {
        for arg in &mut self.source_args {
            if is_vfs_uri(arg) {
                // VFS URIs pass through unchanged
                continue;
            }

            let path = Path::new(arg.as_str());
            if !path.is_absolute() {
                let resolved = working_dir.join(path);
                *arg = resolved.to_string_lossy().to_string();
            }
        }
    }

    /// Return source args with relative paths resolved against `working_dir`.
    ///
    /// Does not mutate self — returns a new Vec of resolved URIs/paths.
    pub fn resolved_source_args(&self, working_dir: &Path) -> Vec<String> {
        self.source_args
            .iter()
            .map(|arg| {
                if is_vfs_uri(arg) {
                    arg.clone()
                } else {
                    let path = Path::new(arg.as_str());
                    if path.is_absolute() {
                        arg.clone()
                    } else {
                        working_dir.join(path).to_string_lossy().to_string()
                    }
                }
            })
            .collect()
    }
}

/// Detect whether a string is a VFS URI.
///
/// VFS URIs start with the `vfs://` scheme and should not be treated
/// as filesystem paths.
pub fn is_vfs_uri(s: &str) -> bool {
    s.starts_with(VFS_SCHEME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_args_produces_default_cli_args() {
        // Validates: Requirement 6 AC 6.1
        let args: Vec<&str> = vec![];
        let result = CliArgs::parse_from(args).unwrap();
        assert_eq!(result, CliArgs::default());
        assert!(!result.has_source_args());
    }

    #[test]
    fn positional_args_captured_as_source_args() {
        // Validates: Requirement 6 AC 6.1
        let args = vec!["file1.txt", "file2.rs", "src/main.rs"];
        let result = CliArgs::parse_from(args).unwrap();
        assert_eq!(
            result.source_args,
            vec!["file1.txt", "file2.rs", "src/main.rs"]
        );
        assert!(result.has_source_args());
    }

    #[test]
    fn vfs_uri_captured_as_source_arg() {
        // Validates: Requirement 6 AC 6.3
        let args = vec!["vfs://local/path/to/file"];
        let result = CliArgs::parse_from(args).unwrap();
        assert_eq!(result.source_args, vec!["vfs://local/path/to/file"]);
    }

    #[test]
    fn new_window_flag_parsed() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--new-window"];
        let result = CliArgs::parse_from(args).unwrap();
        assert!(result.new_window);
    }

    #[test]
    fn no_session_restore_flag_parsed() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--no-session-restore"];
        let result = CliArgs::parse_from(args).unwrap();
        assert!(result.no_session_restore);
    }

    #[test]
    fn profile_flag_with_value_parsed() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--profile", "my-profile"];
        let result = CliArgs::parse_from(args).unwrap();
        assert_eq!(result.profile, Some("my-profile".to_string()));
    }

    #[test]
    fn profile_flag_without_value_returns_error() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--profile"];
        let result = CliArgs::parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn project_flag_with_value_parsed() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--project", "/home/user/my-project"];
        let result = CliArgs::parse_from(args).unwrap();
        assert_eq!(result.project, Some(PathBuf::from("/home/user/my-project")));
    }

    #[test]
    fn project_flag_without_value_returns_error() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--project"];
        let result = CliArgs::parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn log_level_flag_with_value_parsed() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--log-level", "debug"];
        let result = CliArgs::parse_from(args).unwrap();
        assert_eq!(result.log_level, Some("debug".to_string()));
    }

    #[test]
    fn log_level_flag_without_value_returns_error() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--log-level"];
        let result = CliArgs::parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_flag_returns_error() {
        // Validates: Requirement 6 AC 6.6
        let args = vec!["--unknown-flag"];
        let result = CliArgs::parse_from(args);
        assert!(result.is_err());
        if let Err(SessionError::CliArgInvalid { argument, reason }) = result {
            assert_eq!(argument, "--unknown-flag");
            assert!(reason.contains("unrecognised"));
        }
    }

    #[test]
    fn mixed_flags_and_positional_args_parsed() {
        // Validates: Requirement 6 AC 6.1, 6.6
        let args = vec![
            "file1.txt",
            "--profile",
            "dev",
            "--new-window",
            "vfs://remote/file",
            "--log-level",
            "warn",
        ];
        let result = CliArgs::parse_from(args).unwrap();
        assert_eq!(result.source_args, vec!["file1.txt", "vfs://remote/file"]);
        assert!(result.new_window);
        assert_eq!(result.profile, Some("dev".to_string()));
        assert_eq!(result.log_level, Some("warn".to_string()));
    }

    #[test]
    fn resolve_source_args_resolves_relative_paths() {
        // Validates: Requirement 6 AC 6.2
        let mut args = CliArgs {
            source_args: vec!["relative/file.txt".to_string()],
            ..Default::default()
        };

        #[cfg(windows)]
        let working_dir = Path::new("C:\\Users\\user\\projects");
        #[cfg(not(windows))]
        let working_dir = Path::new("/home/user/projects");

        args.resolve_source_args(working_dir);

        let expected = working_dir.join("relative/file.txt");
        assert_eq!(args.source_args[0], expected.to_string_lossy().to_string());
    }

    #[test]
    fn resolve_source_args_leaves_absolute_paths_unchanged() {
        // Validates: Requirement 6 AC 6.2
        #[cfg(windows)]
        let abs_path = "C:\\absolute\\path\\file.txt";
        #[cfg(not(windows))]
        let abs_path = "/absolute/path/file.txt";

        let mut args = CliArgs {
            source_args: vec![abs_path.to_string()],
            ..Default::default()
        };

        #[cfg(windows)]
        let working_dir = Path::new("C:\\Users\\user\\projects");
        #[cfg(not(windows))]
        let working_dir = Path::new("/home/user/projects");

        args.resolve_source_args(working_dir);
        assert_eq!(args.source_args[0], abs_path);
    }

    #[test]
    fn resolve_source_args_leaves_vfs_uris_unchanged() {
        // Validates: Requirement 6 AC 6.3
        let mut args = CliArgs {
            source_args: vec!["vfs://local/path/to/file".to_string()],
            ..Default::default()
        };

        #[cfg(windows)]
        let working_dir = Path::new("C:\\Users\\user");
        #[cfg(not(windows))]
        let working_dir = Path::new("/home/user");

        args.resolve_source_args(working_dir);
        assert_eq!(args.source_args[0], "vfs://local/path/to/file");
    }

    #[test]
    fn resolve_source_args_handles_mixed_args() {
        // Validates: Requirement 6 AC 6.2, 6.3
        #[cfg(windows)]
        let abs_path = "C:\\absolute\\file.rs";
        #[cfg(not(windows))]
        let abs_path = "/absolute/file.rs";

        let mut args = CliArgs {
            source_args: vec![
                "relative.txt".to_string(),
                abs_path.to_string(),
                "vfs://remote/doc.md".to_string(),
            ],
            ..Default::default()
        };

        #[cfg(windows)]
        let working_dir = Path::new("C:\\work");
        #[cfg(not(windows))]
        let working_dir = Path::new("/work");

        args.resolve_source_args(working_dir);

        let expected_relative = working_dir.join("relative.txt");
        assert_eq!(
            args.source_args[0],
            expected_relative.to_string_lossy().to_string()
        );
        assert_eq!(args.source_args[1], abs_path);
        assert_eq!(args.source_args[2], "vfs://remote/doc.md");
    }

    #[test]
    fn is_vfs_uri_detects_vfs_scheme() {
        assert!(is_vfs_uri("vfs://local/path"));
        assert!(is_vfs_uri("vfs://"));
        assert!(!is_vfs_uri("file.txt"));
        assert!(!is_vfs_uri("/absolute/path"));
        assert!(!is_vfs_uri("VFS://uppercase")); // Case-sensitive
    }

    #[test]
    fn resolved_source_args_returns_new_vec_without_mutation() {
        // Validates: Requirement 6 AC 6.2
        let args = CliArgs {
            source_args: vec!["relative.txt".to_string(), "vfs://remote/file".to_string()],
            ..Default::default()
        };

        #[cfg(windows)]
        let working_dir = Path::new("C:\\base");
        #[cfg(not(windows))]
        let working_dir = Path::new("/base");

        let resolved = args.resolved_source_args(working_dir);

        let expected_relative = working_dir.join("relative.txt");
        assert_eq!(resolved[0], expected_relative.to_string_lossy().to_string());
        assert_eq!(resolved[1], "vfs://remote/file");
        // Original unchanged
        assert_eq!(args.source_args[0], "relative.txt");
    }
}
