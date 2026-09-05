//! Headless FFTest CLI runner.
//!
//! Handles `--run-tests`, `--run-script <path>`, and `--update-baselines`
//! CLI flags. When any of these flags are present, the application runs
//! scripts in-process without launching an eframe window and exits with
//! an appropriate exit code.
//!
//! Exit codes:
//!   0 -- all tests passed
//!   1 -- one or more test failures
//!   2 -- script parse error
//!   3 -- runner initialisation failure
//!
//! Validates: Requirement 6.1, 6.2, 6.3 (automated-dialog-testing)

use ff_fftest::automation::InMemoryAutomationRegistry;
use ff_fftest::capture::BaselineStore;
use ff_fftest::parser::{parse, ParseError};
use ff_fftest::report::{build_html_report, build_json_report, serialise_json};
use ff_fftest::runner::Runner;

// === Exit codes =============================================================

/// Exit code: all tests passed.
pub const EXIT_PASS: i32 = 0;
/// Exit code: one or more test failures.
pub const EXIT_FAILURE: i32 = 1;
/// Exit code: script parse error.
pub const EXIT_PARSE_ERROR: i32 = 2;
/// Exit code: runner initialisation failure.
pub const EXIT_INIT_FAILURE: i32 = 3;

// === CliMode ================================================================

/// The headless execution mode requested via CLI flags.
///
/// Validates: Requirement 6.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliMode {
    /// `--run-script <path>` -- run a single .fftest file.
    RunScript(String),
    /// `--run-tests` -- run all .fftest files under `tests/dialog/`.
    RunTests,
    /// `--update-baselines` -- clear all baselines so they are re-created on next run.
    UpdateBaselines,
}

/// Parse CLI arguments and return the headless mode if any flag is present.
///
/// Returns `None` if no headless flag is found (normal GUI launch).
///
/// Validates: Requirement 6.2
pub fn detect_cli_mode(args: &[String]) -> Option<CliMode> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--run-script" => {
                if let Some(path) = iter.next() {
                    return Some(CliMode::RunScript(path.clone()));
                }
            }
            "--run-tests" => return Some(CliMode::RunTests),
            "--update-baselines" => return Some(CliMode::UpdateBaselines),
            _ => {}
        }
    }
    None
}

// === run_headless ============================================================

/// Execute the requested headless mode and return the process exit code.
///
/// Validates: Requirement 6.1, 6.3
pub fn run_headless(mode: &CliMode, workspace_root: &std::path::Path) -> i32 {
    match mode {
        CliMode::UpdateBaselines => {
            let store = BaselineStore::new(workspace_root.join("tests").join("baselines"));
            match store.clear_all() {
                Ok(()) => {
                    eprintln!("[fftest] baselines cleared -- they will be re-created on next run");
                    EXIT_PASS
                }
                Err(e) => {
                    eprintln!("[fftest] failed to clear baselines: {e}");
                    EXIT_INIT_FAILURE
                }
            }
        }
        CliMode::RunScript(path) => run_single_script(std::path::Path::new(path), workspace_root),
        CliMode::RunTests => run_all_scripts(workspace_root),
    }
}

// === run_single_script ======================================================

fn run_single_script(script_path: &std::path::Path, workspace_root: &std::path::Path) -> i32 {
    let source = match std::fs::read_to_string(script_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[fftest] cannot read '{}': {e}", script_path.display());
            return EXIT_INIT_FAILURE;
        }
    };

    let script = match parse(&source) {
        Ok(s) => s,
        Err(ParseError::UnknownCommand { line, keyword }) => {
            eprintln!(
                "[fftest] parse error in '{}' line {line}: unknown command '{keyword}'",
                script_path.display()
            );
            return EXIT_PARSE_ERROR;
        }
        Err(ParseError::MissingArgument { line, command, .. }) => {
            eprintln!(
                "[fftest] parse error in '{}' line {line}: '{command}' missing argument",
                script_path.display()
            );
            return EXIT_PARSE_ERROR;
        }
        Err(ParseError::UnterminatedString { line }) => {
            eprintln!(
                "[fftest] parse error in '{}' line {line}: unterminated string",
                script_path.display()
            );
            return EXIT_PARSE_ERROR;
        }
    };

    // Headless: empty registry (no live window)
    let registry = InMemoryAutomationRegistry::new();
    let runner = Runner::new(&registry, vec![]);
    let report = runner.run(&script);

    let suite_name = script_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    let timestamp = chrono_timestamp();
    write_reports(workspace_root, &suite_name, &timestamp, &report);

    print_summary(&suite_name, &report);

    if report.all_passed() {
        EXIT_PASS
    } else {
        EXIT_FAILURE
    }
}

// === run_all_scripts ========================================================

fn run_all_scripts(workspace_root: &std::path::Path) -> i32 {
    let dialog_dir = workspace_root.join("tests").join("dialog");
    let entries = match std::fs::read_dir(&dialog_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[fftest] cannot read '{}': {e}", dialog_dir.display());
            return EXIT_INIT_FAILURE;
        }
    };

    let mut any_failure = false;
    let mut script_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "fftest").unwrap_or(false) {
            script_count += 1;
            let code = run_single_script(&path, workspace_root);
            if code != EXIT_PASS {
                any_failure = true;
            }
        }
    }

    if script_count == 0 {
        eprintln!(
            "[fftest] no .fftest scripts found under '{}'",
            dialog_dir.display()
        );
    }

    if any_failure {
        EXIT_FAILURE
    } else {
        EXIT_PASS
    }
}

// === helpers ================================================================

fn write_reports(
    workspace_root: &std::path::Path,
    suite_name: &str,
    timestamp: &str,
    report: &ff_fftest::runner::RunReport,
) {
    let reports_dir = workspace_root.join("reports");
    if std::fs::create_dir_all(&reports_dir).is_err() {
        return;
    }

    let json_rep = build_json_report(suite_name, timestamp, report);
    if let Ok(json) = serialise_json(&json_rep) {
        let _ = std::fs::write(reports_dir.join(format!("{suite_name}.json")), json);
    }

    let html = build_html_report(suite_name, timestamp, report);
    let _ = std::fs::write(reports_dir.join(format!("{suite_name}.html")), html);
}

fn print_summary(suite_name: &str, report: &ff_fftest::runner::RunReport) {
    let status = if report.all_passed() { "PASS" } else { "FAIL" };
    eprintln!(
        "[fftest] {suite_name}: {status} -- {}/{} assertions passed ({} ms)",
        report.passed,
        report.total_assertions,
        report.duration.as_millis(),
    );
    for step in &report.steps {
        if !step.passed {
            eprintln!("  FAIL L{}: {}", step.line, step.description);
            if let Some(diag) = &step.diagnostic {
                eprintln!("       {diag}");
            }
        }
    }
}

fn chrono_timestamp() -> String {
    // Use a simple fixed format without chrono dependency in this module.
    // In production this would use chrono::Utc::now().
    "".to_string()
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 6.2 -- --run-script flag detected
    #[test]
    fn detect_run_script_flag() {
        let args = vec![
            "--run-script".to_string(),
            "tests/dialog/smoke.fftest".to_string(),
        ];
        assert_eq!(
            detect_cli_mode(&args),
            Some(CliMode::RunScript("tests/dialog/smoke.fftest".to_string()))
        );
    }

    // Validates: Requirement 6.2 -- --run-tests flag detected
    #[test]
    fn detect_run_tests_flag() {
        let args = vec!["--run-tests".to_string()];
        assert_eq!(detect_cli_mode(&args), Some(CliMode::RunTests));
    }

    // Validates: Requirement 6.2 -- --update-baselines flag detected
    #[test]
    fn detect_update_baselines_flag() {
        let args = vec!["--update-baselines".to_string()];
        assert_eq!(detect_cli_mode(&args), Some(CliMode::UpdateBaselines));
    }

    // Validates: Requirement 6.2 -- no headless flag returns None
    #[test]
    fn no_headless_flag_returns_none() {
        let args = vec!["file.txt".to_string(), "--no-session-restore".to_string()];
        assert!(detect_cli_mode(&args).is_none());
    }

    // Validates: Requirement 6.3 -- exit code 2 on parse error
    #[test]
    fn run_script_returns_parse_error_code_for_bad_script() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let script_path = dir.path().join("bad.fftest");
        std::fs::write(&script_path, "FLORP SOMETHING\n").expect("write");
        let code = run_single_script(&script_path, dir.path());
        assert_eq!(code, EXIT_PARSE_ERROR);
    }

    // Validates: Requirement 6.3 -- exit code 3 on missing file
    #[test]
    fn run_script_returns_init_failure_for_missing_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let code = run_single_script(
            std::path::Path::new("/nonexistent/path/script.fftest"),
            dir.path(),
        );
        assert_eq!(code, EXIT_INIT_FAILURE);
    }

    // Validates: Requirement 6.3 -- exit code 0 for passing script
    #[test]
    fn run_script_returns_pass_for_empty_script() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let script_path = dir.path().join("empty.fftest");
        std::fs::write(&script_path, "# just a comment\n").expect("write");
        let code = run_single_script(&script_path, dir.path());
        assert_eq!(code, EXIT_PASS);
    }

    // Validates: Requirement 6.3 -- exit code 1 for failing assertion
    #[test]
    fn run_script_returns_failure_for_failing_assertion() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let script_path = dir.path().join("fail.fftest");
        std::fs::write(&script_path, "ASSERT STATUSBAR CONTAINS \"Ready\"\n").expect("write");
        // Empty registry -- statusbar.message not registered -- assertion fails
        let code = run_single_script(&script_path, dir.path());
        assert_eq!(code, EXIT_FAILURE);
    }

    // Validates: Requirement 7.6 -- reports written to reports/ directory
    #[test]
    fn run_script_writes_json_and_html_reports() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let script_path = dir.path().join("report_test.fftest");
        std::fs::write(&script_path, "# comment only\n").expect("write");
        run_single_script(&script_path, dir.path());
        assert!(dir.path().join("reports").join("report_test.json").exists());
        assert!(dir.path().join("reports").join("report_test.html").exists());
    }

    // Validates: Requirement 8.4 -- --update-baselines clears baseline files
    #[test]
    fn update_baselines_clears_existing_baselines() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let baselines_dir = dir.path().join("tests").join("baselines");
        std::fs::create_dir_all(&baselines_dir).expect("mkdir");
        std::fs::write(baselines_dir.join("old.png"), b"fake").expect("write");
        let code = run_headless(&CliMode::UpdateBaselines, dir.path());
        assert_eq!(code, EXIT_PASS);
        assert!(!baselines_dir.join("old.png").exists());
    }
}
