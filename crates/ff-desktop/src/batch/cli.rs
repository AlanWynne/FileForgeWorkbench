use crate::batch::return_code::AbortPolicy;
use crate::batch::return_code::StepReturnCode;

/// Parsed `--batch*` CLI arguments.
#[derive(Debug)]
pub struct BatchCliArgs {
    /// Path to the batch input file, or `"-"` for stdin.
    pub input: String,
    /// Output sink: None = stdout, Some(path) = file.
    pub output: Option<String>,
    /// Append to output file instead of overwriting.
    pub output_append: bool,
    /// Echo each command before its output.
    pub echo: bool,
    /// Abort policy derived from `--batch-abort-on-error <threshold>`.
    pub abort_policy: AbortPolicy,
    /// Dry-run: parse and validate but do not execute state-modifying commands.
    pub dry_run: bool,
    /// Named configuration profile.
    pub profile: Option<String>,
    /// Start with empty catalog registry.
    pub no_catalog: bool,
    /// Redirect structured log to this file.
    pub log_file: Option<String>,
}

/// Print usage/help text to stdout.
/// Validates: Requirement 1.5
pub fn print_help() {
    println!(
        "Usage: ffwb [OPTIONS] [FILE...]

Options:
  --batch <file>                  Execute commands from <file> in headless batch mode.
                                  Use '-' to read from stdin.
  --batch-output <file>           Write batch command output to <file> (overwrite).
  --batch-output-append <file>    Append batch command output to <file>.
  --batch-echo                    Echo each command before its output (===> <cmd>).
  --batch-abort-on-error <n>      Abort when any command RC >= n (4, 8, 12, or 16).
  --batch-dry-run                 Validate commands without executing state changes.
  --batch-profile <name>          Load named configuration profile for the batch run.
  --batch-no-catalog              Start with empty catalog registry.
  --batch-log <file>              Write structured batch log to <file>.
  --help                          Show this help message and exit.

Examples:
  ffwb --batch cmds.txt
  ffwb --batch - < cmds.txt
  ffwb --batch cmds.txt --batch-echo --batch-output results.txt
  ffwb file1.txt file2.rs"
    );
}

/// Parse `--batch*` flags from an argv slice.
///
/// Returns `Some(BatchCliArgs)` when `--batch` is present, `None` otherwise.
/// Returns `Err(message)` when the arguments are invalid (e.g. `--batch`
/// combined with positional file paths, or an unrecognised `--batch-*` flag).
pub fn parse_batch_args(args: &[String]) -> Result<Option<BatchCliArgs>, String> {
    let mut batch_input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut output_append: bool = false;
    let mut echo = false;
    let mut abort_threshold: Option<StepReturnCode> = None;
    let mut dry_run = false;
    let mut profile: Option<String> = None;
    let mut no_catalog = false;
    let mut log_file: Option<String> = None;
    let mut positional: Vec<&str> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--batch" => {
                i += 1;
                let val = args.get(i).ok_or("--batch requires a file path or '-'")?;
                batch_input = Some(val.clone());
            }
            "--batch-output" => {
                i += 1;
                let val = args.get(i).ok_or("--batch-output requires a file path")?;
                output = Some(val.clone());
            }
            "--batch-output-append" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("--batch-output-append requires a file path")?;
                output = Some(val.clone());
                output_append = true;
            }
            "--batch-echo" => echo = true,
            "--batch-dry-run" => dry_run = true,
            "--batch-no-catalog" => no_catalog = true,
            "--batch-abort-on-error" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("--batch-abort-on-error requires a threshold (4, 8, 12, or 16)")?;
                let n: i32 = val.parse().map_err(|_| {
                    format!("--batch-abort-on-error: '{}' is not a valid threshold", val)
                })?;
                if ![4, 8, 12, 16].contains(&n) {
                    return Err(format!(
                        "--batch-abort-on-error: threshold must be 4, 8, 12, or 16; got {}",
                        n
                    ));
                }
                abort_threshold = Some(StepReturnCode::from(n));
            }
            "--batch-profile" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("--batch-profile requires a profile name")?;
                profile = Some(val.clone());
            }
            "--batch-log" => {
                i += 1;
                let val = args.get(i).ok_or("--batch-log requires a file path")?;
                log_file = Some(val.clone());
            }
            a if a.starts_with("--batch") => {
                return Err(format!("unrecognised batch flag: {}", a));
            }
            a if !a.starts_with('-') => {
                positional.push(a);
            }
            _ => {} // other flags (e.g. --no-session-restore) are ignored here
        }
        i += 1;
    }

    let input = match batch_input {
        None => return Ok(None),
        Some(v) => v,
    };

    // Req 1.3: --batch is incompatible with positional file path arguments.
    if !positional.is_empty() {
        return Err(
            "--batch is incompatible with positional file path arguments; \
             pass either --batch <file> or file paths, not both"
                .to_string(),
        );
    }

    let abort_policy = match abort_threshold {
        Some(t) => AbortPolicy::AbortOnError(t),
        None => AbortPolicy::BestEffort,
    };

    Ok(Some(BatchCliArgs {
        input,
        output,
        output_append,
        echo,
        abort_policy,
        dry_run,
        profile,
        no_catalog,
        log_file,
    }))
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::return_code::StepReturnCode;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // Validates: Requirement 1.1
    #[test]
    fn parse_batch_file_path_returns_some() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt"])).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().input, "cmds.txt");
    }

    // Validates: Requirement 1.2
    #[test]
    fn parse_batch_stdin_dash_returns_some() {
        let result = parse_batch_args(&args(&["--batch", "-"])).unwrap();
        assert_eq!(result.unwrap().input, "-");
    }

    // Validates: Requirement 1.4
    #[test]
    fn no_batch_flag_returns_none() {
        let result = parse_batch_args(&args(&["file.txt"])).unwrap();
        assert!(result.is_none());
    }

    // Validates: Requirement 1.3
    #[test]
    fn batch_with_positional_args_returns_error() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt", "file.txt"]));
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("incompatible"),
            "error should mention incompatible: {}",
            msg
        );
    }

    // Validates: Requirement 4.2
    #[test]
    fn batch_output_flag_sets_output_path() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt", "--batch-output", "out.txt"]))
            .unwrap()
            .unwrap();
        assert_eq!(result.output, Some("out.txt".to_string()));
        assert!(!result.output_append);
    }

    // Validates: Requirement 4.3
    #[test]
    fn batch_output_append_flag_sets_append_mode() {
        let result = parse_batch_args(&args(&[
            "--batch",
            "cmds.txt",
            "--batch-output-append",
            "out.txt",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(result.output, Some("out.txt".to_string()));
        assert!(result.output_append);
    }

    // Validates: Requirement 4.4
    #[test]
    fn batch_echo_flag_sets_echo() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt", "--batch-echo"]))
            .unwrap()
            .unwrap();
        assert!(result.echo);
    }

    // Validates: Requirement 6.2
    #[test]
    fn batch_abort_on_error_sets_policy() {
        let result = parse_batch_args(&args(&[
            "--batch",
            "cmds.txt",
            "--batch-abort-on-error",
            "8",
        ]))
        .unwrap()
        .unwrap();
        assert!(result.abort_policy.should_abort(StepReturnCode::Error));
        assert!(!result.abort_policy.should_abort(StepReturnCode::Warning));
    }

    // Validates: Requirement 6.2
    #[test]
    fn batch_abort_on_error_invalid_threshold_returns_error() {
        let result = parse_batch_args(&args(&[
            "--batch",
            "cmds.txt",
            "--batch-abort-on-error",
            "7",
        ]));
        assert!(result.is_err());
    }

    // Validates: Requirement 8.1
    #[test]
    fn batch_dry_run_flag_sets_dry_run() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt", "--batch-dry-run"]))
            .unwrap()
            .unwrap();
        assert!(result.dry_run);
    }

    // Validates: Requirement 7.3
    #[test]
    fn batch_profile_flag_sets_profile() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt", "--batch-profile", "ci"]))
            .unwrap()
            .unwrap();
        assert_eq!(result.profile, Some("ci".to_string()));
    }

    // Validates: Requirement 7.6
    #[test]
    fn batch_no_catalog_flag_sets_no_catalog() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt", "--batch-no-catalog"]))
            .unwrap()
            .unwrap();
        assert!(result.no_catalog);
    }

    // Validates: Requirement 10.2
    #[test]
    fn batch_log_flag_sets_log_file() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt", "--batch-log", "run.log"]))
            .unwrap()
            .unwrap();
        assert_eq!(result.log_file, Some("run.log".to_string()));
    }

    // Validates: Requirement 1.1 -- unrecognised --batch-* flag is an error
    #[test]
    fn unrecognised_batch_flag_returns_error() {
        let result = parse_batch_args(&args(&["--batch", "cmds.txt", "--batch-unknown"]));
        assert!(result.is_err());
    }

    // Validates: Requirement 1.5
    #[test]
    fn print_help_does_not_panic() {
        // Confirms help text is printable without panicking.
        // Actual output goes to stdout; we just verify no panic.
        // In a real test harness this would capture stdout.
        super::print_help();
    }
}
