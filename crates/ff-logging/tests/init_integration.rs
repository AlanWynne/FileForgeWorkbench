//! Integration tests for the `ff-logging` initialization sequence.
//!
//! These tests verify the full `init()` flow including file creation, startup
//! record content, status reporting, and fallback behavior.
//!
//! Because `init()` uses a global `OnceLock<LogSubsystem>`, only ONE call to
//! `init()` can succeed per process. Integration test files in Rust's `tests/`
//! directory each compile to a separate binary, but test functions within a file
//! still share the same process and run in parallel by default.
//!
//! Strategy: We use `OnceLock` to perform a single controlled `init()` call
//! shared by all tests. Fallback tests trigger a second `init()` which returns
//! Fallback due to the OnceLock being already set.

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use ff_logging::{
    current_level, dropped_count, init, is_fallback, LogConfig, LogLevel, LoggingStatus,
};

/// State captured from the single successful `init()` call.
struct InitState {
    status: LoggingStatus,
    log_dir: PathBuf,
    /// Keep the TempDir alive so the directory isn't deleted.
    _temp_dir: tempfile::TempDir,
}

/// Global state from the one successful init() call.
static INIT_STATE: OnceLock<InitState> = OnceLock::new();

/// Performs the one-time init() call shared by all happy-path tests.
fn ensure_init() -> &'static InitState {
    INIT_STATE.get_or_init(|| {
        let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
        let log_dir = tmp.path().join("logs");

        let config = LogConfig {
            level: LogLevel::Debug,
            directory: log_dir.clone(),
            max_file_size_mb: 10,
            max_retained_files: 5,
        };

        let status = init(config);

        InitState {
            status,
            log_dir,
            _temp_dir: tmp,
        }
    })
}

// ─── Happy Path Tests ───────────────────────────────────────────────────────

#[test]
fn init_with_valid_directory_returns_active() {
    // Validates: Requirement 1.1
    let state = ensure_init();
    assert_eq!(
        state.status,
        LoggingStatus::Active,
        "init() with a valid writable directory should return Active"
    );
}

#[test]
fn is_fallback_returns_false_after_successful_init() {
    // Validates: Requirement 1.4
    let state = ensure_init();
    // Only valid if our init was the first one to run (Active status)
    if state.status == LoggingStatus::Active {
        assert!(
            !is_fallback(),
            "is_fallback() should return false after successful init"
        );
    }
}

#[test]
fn current_level_returns_configured_level_after_init() {
    // Validates: Requirement 3.2
    ensure_init();
    assert_eq!(
        current_level(),
        LogLevel::Debug,
        "current_level() should return the level configured at init (Debug)"
    );
}

#[test]
fn dropped_count_returns_zero_initially() {
    // Validates: Requirement 8.5
    ensure_init();
    assert_eq!(
        dropped_count(),
        0,
        "dropped_count() should be 0 immediately after init with no overflow"
    );
}

#[test]
fn log_file_exists_in_configured_directory_after_init() {
    // Validates: Requirement 1.5, 4.2
    let state = ensure_init();

    assert!(
        state.log_dir.exists(),
        "Log directory should be created by init()"
    );

    let log_files: Vec<PathBuf> = fs::read_dir(&state.log_dir)
        .expect("failed to read log directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "log").unwrap_or(false))
        .collect();

    assert!(
        !log_files.is_empty(),
        "At least one .log file should exist in the log directory after init"
    );
}

#[test]
fn startup_record_contains_app_name_and_version() {
    // Validates: Requirement 1.2
    let state = ensure_init();

    // Wait for the writer thread to flush the startup record.
    // The startup record is INFO level — relies on the 1-second periodic flush.
    std::thread::sleep(std::time::Duration::from_millis(1500));

    let log_files: Vec<PathBuf> = fs::read_dir(&state.log_dir)
        .expect("failed to read log directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "log").unwrap_or(false))
        .collect();

    let mut found_startup_record = false;
    for log_file in &log_files {
        let content = fs::read_to_string(log_file).expect("failed to read log file");
        if content.contains("FileForgeWorkbench") {
            found_startup_record = true;
            // Verify it also contains a version string (format: "vX.Y.Z")
            assert!(
                content.contains("v0."),
                "Startup record should contain a version string like 'v0.x.y', got:\n{content}"
            );
            break;
        }
    }

    assert!(
        found_startup_record,
        "The startup record containing 'FileForgeWorkbench' should be present in a log file. \
         Files found: {:?}",
        log_files
    );
}

// ─── Fallback Path Tests (double-init) ──────────────────────────────────────

#[test]
fn init_called_again_returns_fallback_due_to_oncelock() {
    // Validates: Requirement 1.3
    ensure_init(); // ensure first init happened

    let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
    let config = LogConfig {
        level: LogLevel::Info,
        directory: tmp.path().join("second_init_logs"),
        max_file_size_mb: 10,
        max_retained_files: 5,
    };

    // Second call to init() — OnceLock is already set, should return Fallback
    let status = init(config);
    assert_eq!(
        status,
        LoggingStatus::Fallback,
        "Second init() call should return Fallback (OnceLock already initialized)"
    );
}

#[test]
fn is_fallback_returns_true_after_double_init_triggers_noop_sink() {
    // Validates: Requirement 1.4
    ensure_init(); // ensure first init happened

    // Trigger a second init which will activate NoOpSink
    let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
    let config = LogConfig {
        level: LogLevel::Info,
        directory: tmp.path().join("fallback_test"),
        max_file_size_mb: 10,
        max_retained_files: 5,
    };
    let status = init(config);
    assert_eq!(status, LoggingStatus::Fallback);

    // After a fallback-triggering init, is_fallback() should be true
    assert!(
        is_fallback(),
        "is_fallback() should return true after a fallback init path was triggered"
    );
}
