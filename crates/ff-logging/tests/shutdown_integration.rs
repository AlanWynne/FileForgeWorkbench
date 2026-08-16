//! Integration tests for the `ff-logging` shutdown sequence.
//!
//! These tests verify that `shutdown()` correctly flushes buffered records,
//! writes the final "Application shutdown complete" record, rejects new log
//! calls after the shutdown signal, and is safe to call multiple times.
//!
//! Because `init()` uses a global `OnceLock` and `shutdown()` sets a global
//! `AtomicBool`, only ONE init+shutdown cycle can occur per process. All
//! shutdown assertions are therefore consolidated into a single test function
//! that exercises the full sequence in order.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ff_logging::{init, log, shutdown, LogConfig, LogLevel, LoggingStatus};

/// Read the combined content of all `.log` files in the given directory.
fn read_all_log_content(log_dir: &PathBuf) -> String {
    let mut content = String::new();
    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "log").unwrap_or(false) {
                if let Ok(file_content) = fs::read_to_string(&path) {
                    content.push_str(&file_content);
                }
            }
        }
    }
    content
}

#[test]
fn shutdown_flushes_records_writes_final_message_and_rejects_new_calls() {
    // Validates: Requirement 6.3, Requirement 8.6
    //
    // This test exercises the complete shutdown sequence:
    // 1. init() with a temp directory
    // 2. Write several log records
    // 3. Call shutdown() and measure that it completes within 5 seconds
    // 4. Verify "Application shutdown complete" is the last meaningful record
    // 5. Verify log() calls after shutdown are silently discarded
    // 6. Verify calling shutdown() again doesn't panic (idempotent)

    let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
    let log_dir = tmp.path().join("logs");

    let config = LogConfig {
        level: LogLevel::Debug,
        directory: log_dir.clone(),
        max_file_size_mb: 10,
        max_retained_files: 5,
    };

    let status = init(config);
    assert_eq!(
        status,
        LoggingStatus::Active,
        "init() should return Active for a valid temp directory"
    );

    // ─── Step 1: Write several log records ──────────────────────────────────

    log(LogLevel::Info, "test::shutdown", "Record one");
    log(LogLevel::Debug, "test::shutdown", "Record two");
    log(LogLevel::Warn, "test::shutdown", "Record three");

    // Write a batch to exercise buffer behavior
    for i in 0..50 {
        log(
            LogLevel::Debug,
            "test::shutdown",
            &format!("Batch record {i}"),
        );
    }

    // ─── Step 2: Call shutdown() and verify it completes within 5 seconds ───

    let start = Instant::now();
    shutdown();
    let elapsed = start.elapsed();

    // Validates: Requirement 8.6 — flush within 5 seconds
    assert!(
        elapsed < Duration::from_secs(5),
        "shutdown() should complete within 5 seconds under normal conditions, took {:?}",
        elapsed
    );

    // Give a brief moment for file handles to fully close
    std::thread::sleep(Duration::from_millis(100));

    // ─── Step 3: Read log file and verify all records were flushed ───────────

    let content = read_all_log_content(&log_dir);

    // All three named records should be present (flush worked)
    assert!(
        content.contains("Record one"),
        "Log file should contain 'Record one'. Content:\n{content}"
    );
    assert!(
        content.contains("Record two"),
        "Log file should contain 'Record two'. Content:\n{content}"
    );
    assert!(
        content.contains("Record three"),
        "Log file should contain 'Record three'. Content:\n{content}"
    );

    // ─── Step 4: Verify "Application shutdown complete" is last record ──────
    // Validates: Requirement 6.3

    assert!(
        content.contains("Application shutdown complete"),
        "Log file should contain 'Application shutdown complete'. Content:\n{content}"
    );

    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    let last_line = lines
        .last()
        .expect("log file should have at least one line");
    assert!(
        last_line.contains("Application shutdown complete"),
        "The last line should contain the shutdown message. Last line:\n{last_line}"
    );

    // ─── Step 5: Verify log calls after shutdown are silently discarded ──────
    // Validates: Requirement 8.6 — no new log calls after shutdown signal

    log(
        LogLevel::Error,
        "test::post_shutdown",
        "SHOULD_NOT_APPEAR_IN_LOG",
    );
    log(
        LogLevel::Warn,
        "test::post_shutdown",
        "ALSO_SHOULD_NOT_APPEAR",
    );

    // Brief sleep to ensure any hypothetical writes would have time to flush
    std::thread::sleep(Duration::from_millis(200));

    // Re-read log file to check for post-shutdown content
    let content_after = read_all_log_content(&log_dir);

    assert!(
        !content_after.contains("SHOULD_NOT_APPEAR_IN_LOG"),
        "Log file must NOT contain records written after shutdown. Content:\n{content_after}"
    );
    assert!(
        !content_after.contains("ALSO_SHOULD_NOT_APPEAR"),
        "Log file must NOT contain records written after shutdown. Content:\n{content_after}"
    );

    // ─── Step 6: Verify shutdown() is idempotent (no panic on re-call) ──────
    // Validates: Requirement 6.3

    shutdown(); // Second call — should be a no-op, no panic
    shutdown(); // Third call — still no panic
}
