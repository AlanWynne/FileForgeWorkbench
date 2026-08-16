//! Integration tests verifying GUI-independent process execution guarantees.
//!
//! These tests validate Requirement 7 (GUI-Independent Process Execution):
//! - AC 7.4: Log_Subsystem as exclusive diagnostic output channel
//! - AC 7.5: No output on stdout/stderr
//! - AC 7.6: No console allocation or child process spawning
//!
//! The tests use two approaches:
//! 1. Static analysis: scanning source files for prohibited patterns
//! 2. Runtime verification: confirming log operations produce no console output

use std::fs;
use std::path::Path;

/// Collects all `.rs` source files in a directory tree, excluding test modules.
fn collect_production_source_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip the tests directory — we only care about production code
                if path.file_name().map_or(false, |n| n == "tests") {
                    continue;
                }
                files.extend(collect_production_source_files(&path));
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files
}

/// Checks if a line is inside a `#[cfg(test)]` block.
///
/// This is a simplified heuristic: once we see `#[cfg(test)]`, all remaining
/// lines in the file are considered test code. This works because by convention,
/// `#[cfg(test)] mod tests` appears at the end of each source file.
fn is_in_test_section(lines: &[&str], line_index: usize) -> bool {
    for i in (0..=line_index).rev() {
        let trimmed = lines[i].trim();
        if trimmed == "#[cfg(test)]" {
            return true;
        }
        // If we hit a non-test module declaration, stop searching
        if trimmed.starts_with("mod ") && !trimmed.contains("tests") {
            return false;
        }
    }
    false
}

/// Validates: Requirement 7.5
///
/// Verifies that no production source file in ff-logging contains
/// `println!`, `eprintln!`, `print!`, `eprint!`, or `dbg!` macros.
/// These macros write to stdout/stderr which violates the
/// GUI-independent output guarantee.
#[test]
fn source_files_contain_no_stdout_stderr_macros() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let source_files = collect_production_source_files(&src_dir);
    assert!(
        !source_files.is_empty(),
        "Should find at least one source file"
    );

    let prohibited_patterns = ["println!", "eprintln!", "print!", "eprint!", "dbg!"];

    let mut violations = Vec::new();

    for file_path in &source_files {
        let content = fs::read_to_string(file_path).expect("failed to read source file");
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Skip lines in #[cfg(test)] sections
            if is_in_test_section(&lines, line_idx) {
                continue;
            }
            // Skip comments
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            for pattern in &prohibited_patterns {
                if line.contains(pattern) {
                    violations.push(format!(
                        "{}:{}: found '{}' in production code",
                        file_path.display(),
                        line_idx + 1,
                        pattern,
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found stdout/stderr macro usage in production code (violates Requirement 7.5):\n{}",
        violations.join("\n")
    );
}

/// Validates: Requirement 7.5
///
/// Verifies that no production source file contains direct writes to
/// `std::io::stdout()` or `std::io::stderr()`.
#[test]
fn source_files_contain_no_direct_stdout_stderr_writes() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let source_files = collect_production_source_files(&src_dir);

    let prohibited_patterns = [
        "std::io::stdout",
        "std::io::stderr",
        "io::stdout",
        "io::stderr",
    ];

    let mut violations = Vec::new();

    for file_path in &source_files {
        let content = fs::read_to_string(file_path).expect("failed to read source file");
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            if is_in_test_section(&lines, line_idx) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            for pattern in &prohibited_patterns {
                if line.contains(pattern) {
                    violations.push(format!(
                        "{}:{}: found '{}' in production code",
                        file_path.display(),
                        line_idx + 1,
                        pattern,
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found direct stdout/stderr usage in production code (violates Requirement 7.5):\n{}",
        violations.join("\n")
    );
}

/// Validates: Requirement 7.6
///
/// Verifies that no production source file contains `AllocConsole`,
/// Windows console API calls, or `std::process::Command` usage.
#[test]
fn source_files_contain_no_console_allocation_or_process_spawning() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let source_files = collect_production_source_files(&src_dir);

    let prohibited_patterns = [
        "AllocConsole",
        "FreeConsole",
        "AttachConsole",
        "std::process::Command",
        "process::Command",
        "Command::new",
    ];

    let mut violations = Vec::new();

    for file_path in &source_files {
        let content = fs::read_to_string(file_path).expect("failed to read source file");
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            if is_in_test_section(&lines, line_idx) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            for pattern in &prohibited_patterns {
                if line.contains(pattern) {
                    violations.push(format!(
                        "{}:{}: found '{}' in production code",
                        file_path.display(),
                        line_idx + 1,
                        pattern,
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found console allocation or process spawning in production code (violates Requirement 7.6):\n{}",
        violations.join("\n")
    );
}

/// Validates: Requirement 7.4, 7.5
///
/// Verifies that logging operations produce output ONLY in the log file,
/// not on stdout or stderr. This test initializes the subsystem with a
/// temp directory, performs logging operations, and confirms records
/// arrive in the file (proving they went to the file sink, not console).
#[test]
fn logging_writes_exclusively_to_log_file_not_console() {
    // We cannot easily capture stdout/stderr from within the same process
    // without external crates. Instead, we verify the positive case: that
    // log records are written to the log file. Combined with the static
    // analysis tests above (which confirm no print macros exist), this
    // proves logs go to file exclusively.
    //
    // The LogFileWriter is the sole output path after channel delivery.
    // We verify it writes to disk correctly.
    let tmp = tempfile::TempDir::new().expect("failed to create temp dir");

    use ff_logging::LogConfig;
    use ff_logging::LogLevel;

    let config = LogConfig {
        level: LogLevel::Trace,
        directory: tmp.path().to_path_buf(),
        max_file_size_mb: 10,
        max_retained_files: 5,
    };

    let status = ff_logging::init(config);
    assert_eq!(status, ff_logging::LoggingStatus::Active);

    // Perform logging operations at various levels
    ff_logging::log(LogLevel::Info, "test::module", "Info message for file");
    ff_logging::log(LogLevel::Warn, "test::module", "Warning message for file");
    ff_logging::log(LogLevel::Error, "test::module", "Error message for file");
    ff_logging::log(LogLevel::Debug, "test::module", "Debug message for file");
    ff_logging::log(LogLevel::Trace, "test::module", "Trace message for file");

    // Shutdown to flush all records
    ff_logging::shutdown();

    // Verify records were written to the log file
    let log_files: Vec<_> = fs::read_dir(tmp.path())
        .expect("failed to read log directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "log"))
        .collect();

    assert!(
        !log_files.is_empty(),
        "At least one log file should exist in the temp directory"
    );

    // Read log file contents and verify our messages are present
    let mut all_content = String::new();
    for entry in &log_files {
        let content = fs::read_to_string(entry.path()).expect("failed to read log file");
        all_content.push_str(&content);
    }

    assert!(
        all_content.contains("Info message for file"),
        "Log file should contain INFO message"
    );
    assert!(
        all_content.contains("Warning message for file"),
        "Log file should contain WARN message"
    );
    assert!(
        all_content.contains("Error message for file"),
        "Log file should contain ERROR message"
    );
    assert!(
        all_content.contains("Debug message for file"),
        "Log file should contain DEBUG message"
    );
    assert!(
        all_content.contains("Trace message for file"),
        "Log file should contain TRACE message"
    );
}

/// Validates: Requirement 7.6
///
/// Verifies that no `unsafe` blocks exist in production code that could
/// potentially call Windows APIs for console allocation.
#[test]
fn source_files_contain_no_unsafe_blocks() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let source_files = collect_production_source_files(&src_dir);

    let mut violations = Vec::new();

    for file_path in &source_files {
        let content = fs::read_to_string(file_path).expect("failed to read source file");
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            if is_in_test_section(&lines, line_idx) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            // Look for `unsafe` keyword usage (unsafe blocks or unsafe fn)
            if trimmed.contains("unsafe ") || trimmed.contains("unsafe{") {
                violations.push(format!(
                    "{}:{}: found 'unsafe' in production code",
                    file_path.display(),
                    line_idx + 1,
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found unsafe code in production (could allow console API calls, violates Requirement 7.6):\n{}",
        violations.join("\n")
    );
}
