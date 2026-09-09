//! Integration tests for runtime reconfiguration (Requirement 11).
//!
//! These tests exercise the public `ff_logging::reconfigure` API against a
//! real, initialized subsystem writing to temporary directories.
//!
//! IMPORTANT: `ff-logging` uses a process-global singleton (`OnceLock`), so the
//! subsystem can be initialized only once per process. All assertions that need
//! a live subsystem therefore run inside a SINGLE `#[test]` function and are
//! serialized by construction. Reconfigure-before-init and post-shutdown no-op
//! behavior is validated at the unit level in `src/init.rs` where the global
//! state can be controlled in isolation.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ff_logging::{init, log, LogConfig, LogLevel, LoggingStatus};

/// Returns the newest `.log` file in `dir`, or `None` if the directory does not
/// exist or contains no log files.
fn newest_log_file(dir: &Path) -> Option<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("file_forge_workbench_") && n.ends_with(".log"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort();
    entries.pop()
}

/// Polls for `dir` to contain at least one log file whose contents include
/// `needle`, up to `timeout`. Returns the matching file contents, or `None`.
fn wait_for_log_containing(dir: &Path, needle: &str, timeout: Duration) -> Option<String> {
    let start = Instant::now();
    loop {
        if let Some(path) = newest_log_file(dir) {
            if let Ok(content) = fs::read_to_string(&path) {
                if content.contains(needle) {
                    return Some(content);
                }
            }
        }
        if start.elapsed() > timeout {
            return None;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Concatenation of every `.log` file in `dir`.
fn all_logs_concatenated(dir: &Path) -> String {
    let mut out = String::new();
    if let Ok(rd) = fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        paths.sort();
        for p in paths {
            if let Ok(c) = fs::read_to_string(&p) {
                out.push_str(&c);
            }
        }
    }
    out
}

#[test]
fn reconfigure_switches_directory_and_routes_subsequent_records() {
    let base = tempfile::TempDir::new().expect("tempdir");
    let dir1 = base.path().join("dir1");
    let dir2 = base.path().join("dir2");

    // Phase 1: initialize with dir1 at INFO.
    let status = init(LogConfig {
        level: LogLevel::Info,
        directory: dir1.clone(),
        max_file_size_mb: 10,
        max_retained_files: 5,
    });
    assert_eq!(status, LoggingStatus::Active, "init should be Active");

    // A record before reconfigure lands in dir1 (WARN flushes immediately).
    log(LogLevel::Warn, "test::before", "record-before-reconfigure");
    assert!(
        wait_for_log_containing(&dir1, "record-before-reconfigure", Duration::from_secs(3))
            .is_some(),
        "pre-reconfigure record should be in dir1"
    );

    // Phase 2: reconfigure to dir2 at DEBUG.
    let status = ff_logging::reconfigure(LogConfig {
        level: LogLevel::Debug,
        directory: dir2.clone(),
        max_file_size_mb: 10,
        max_retained_files: 5,
    });
    assert_eq!(
        status,
        LoggingStatus::Active,
        "reconfigure should be Active"
    );

    // Validates: Requirement 11.8 -- INFO "reconfigured" record in the new dir.
    assert!(
        wait_for_log_containing(&dir2, "Logging reconfigured", Duration::from_secs(3)).is_some(),
        "reconfigure INFO record should appear in dir2"
    );

    // Validates: Requirement 11.2 -- subsequent records land in dir2.
    log(LogLevel::Warn, "test::after", "record-after-reconfigure");
    let dir2_content =
        wait_for_log_containing(&dir2, "record-after-reconfigure", Duration::from_secs(3))
            .expect("post-reconfigure record should be in dir2");
    assert!(dir2_content.contains("record-after-reconfigure"));

    // Validates: Requirement 11.4 -- DEBUG now passes the filter (was INFO).
    log(LogLevel::Debug, "test::after", "debug-now-visible");
    assert!(
        wait_for_log_containing(&dir2, "debug-now-visible", Duration::from_secs(3)).is_some(),
        "DEBUG record should be written after level lowered to DEBUG"
    );

    // Validates: Requirement 11.2 -- the post-reconfigure records did NOT go to dir1.
    let dir1_content = all_logs_concatenated(&dir1);
    assert!(
        !dir1_content.contains("record-after-reconfigure"),
        "post-reconfigure record must not appear in the old directory"
    );

    // Validates: Requirement 11.5 -- a failed switch retains the current dir and
    // loses nothing. Point at a path inside a regular file (uncreatable dir).
    let blocker = base.path().join("blocker_file");
    fs::write(&blocker, "x").expect("write blocker");
    let bad_dir = blocker.join("nope");
    let status = ff_logging::reconfigure(LogConfig {
        level: LogLevel::Debug,
        directory: bad_dir,
        max_file_size_mb: 10,
        max_retained_files: 5,
    });
    assert_eq!(
        status,
        LoggingStatus::Active,
        "reconfigure dispatch still succeeds even if the switch later fails"
    );

    // The subsystem must remain usable and keep writing to dir2.
    log(
        LogLevel::Warn,
        "test::after_bad",
        "record-after-failed-switch",
    );
    assert!(
        wait_for_log_containing(&dir2, "record-after-failed-switch", Duration::from_secs(3))
            .is_some(),
        "records continue in the retained directory after a failed switch"
    );

    ff_logging::shutdown();

    // Validates: Requirement 11.6 -- reconfigure after shutdown is a safe no-op.
    let status = ff_logging::reconfigure(LogConfig::default());
    assert_eq!(
        status,
        LoggingStatus::Fallback,
        "reconfigure after shutdown must be a no-op returning Fallback"
    );
}
