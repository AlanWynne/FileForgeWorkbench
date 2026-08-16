//! Integration tests for thread safety and performance of the logging subsystem.
//!
//! Validates Requirement 8 (Thread Safety and Performance):
//! - AC 8.1: Safe from any thread without external locks, no data races
//! - AC 8.2: Log call doesn't block GUI thread for more than 1ms
//! - AC 8.3: Internal buffer/async channel decouples production from file I/O
//!
//! These tests spawn multiple threads, measure latency, and assert Send+Sync
//! bounds on public types. Because `init()` uses a global `OnceLock`, a single
//! controlled initialization is shared across all tests in this file.

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use ff_logging::{
    create_plugin_handle, dropped_count, init, log, LogConfig, LogLevel, LogRecord, LoggingStatus,
    PluginLogHandle,
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

/// Performs the one-time init() call shared by all tests in this file.
fn ensure_init() -> &'static InitState {
    INIT_STATE.get_or_init(|| {
        let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
        let log_dir = tmp.path().join("logs");

        let config = LogConfig {
            level: LogLevel::Trace,
            directory: log_dir.clone(),
            max_file_size_mb: 50,
            max_retained_files: 10,
        };

        let status = init(config);

        InitState {
            status,
            log_dir,
            _temp_dir: tmp,
        }
    })
}

// ─── 18.1: Multi-threaded Stress Test ───────────────────────────────────────

#[test]
fn concurrent_logging_from_multiple_threads_produces_no_panics() {
    // Validates: Requirement 8.1
    // Spawn 12 threads each logging 1000 records concurrently.
    // The test passes if no thread panics and all joins succeed.
    let state = ensure_init();
    assert_eq!(state.status, LoggingStatus::Active);

    let num_threads = 12;
    let records_per_thread = 1000;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            std::thread::spawn(move || {
                for i in 0..records_per_thread {
                    log(
                        LogLevel::Info,
                        "stress_test",
                        &format!("Thread {thread_id} msg {i}"),
                    );
                }
            })
        })
        .collect();

    for h in handles {
        h.join()
            .expect("thread should not panic during concurrent logging");
    }

    // Allow the writer thread time to process buffered records
    std::thread::sleep(Duration::from_millis(1500));

    // Verify that records were written by checking the log file is non-empty
    let log_files: Vec<PathBuf> = std::fs::read_dir(&state.log_dir)
        .expect("failed to read log directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "log").unwrap_or(false))
        .collect();

    assert!(
        !log_files.is_empty(),
        "At least one log file should exist after stress test"
    );

    // Count total lines written across all log files
    let total_lines: usize = log_files
        .iter()
        .map(|f| {
            std::fs::read_to_string(f)
                .expect("failed to read log file")
                .lines()
                .count()
        })
        .sum();

    // Total submitted = num_threads * records_per_thread + startup record(s)
    // Some may be dropped if channel fills, so verify:
    // written + dropped >= num_threads * records_per_thread
    let total_submitted = (num_threads * records_per_thread) as u64;
    let total_dropped = dropped_count();
    let total_accounted = total_lines as u64 + total_dropped;

    assert!(
        total_accounted >= total_submitted,
        "written ({total_lines}) + dropped ({total_dropped}) = {total_accounted} \
         should be >= submitted ({total_submitted})"
    );
}

#[test]
fn concurrent_logging_with_mixed_levels_and_plugin_handles() {
    // Validates: Requirement 8.1, Requirement 10.6
    // Tests mixed usage: direct log() calls and plugin handle calls from many threads.
    let state = ensure_init();
    assert_eq!(state.status, LoggingStatus::Active);

    let num_threads = 10;
    let records_per_thread = 500;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            std::thread::spawn(move || {
                let plugin_handle = create_plugin_handle(&format!("plugin-{thread_id}"));
                let levels = [
                    LogLevel::Trace,
                    LogLevel::Debug,
                    LogLevel::Info,
                    LogLevel::Warn,
                    LogLevel::Error,
                ];

                for i in 0..records_per_thread {
                    let level = levels[i % levels.len()];
                    if i % 2 == 0 {
                        log(level, "mixed_test", &format!("Direct t{thread_id} #{i}"));
                    } else {
                        plugin_handle.info("mixed", &format!("Plugin t{thread_id} #{i}"));
                    }
                }
            })
        })
        .collect();

    for h in handles {
        h.join()
            .expect("thread should not panic during mixed concurrent logging");
    }
}

// ─── 18.2: Data Race Detection ──────────────────────────────────────────────
//
// NOTE: Full Miri testing requires nightly Rust and cannot perform file I/O.
// This test exercises the channel-based concurrency (no file I/O) to be
// compatible with `cargo +nightly miri test` if available.
//
// To run under Miri:
//   cargo +nightly miri test --test thread_safety_tests -- miri_compatible
//
// To run under ThreadSanitizer (Linux):
//   RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test --test thread_safety_tests --target x86_64-unknown-linux-gnu

#[test]
fn miri_compatible_concurrent_access_to_dropped_count() {
    // Validates: Requirement 8.1, 8.5
    // Exercises concurrent reads of dropped_count() while logging is active.
    // This is Miri-compatible because it only tests atomics and channel sends
    // (no file I/O in the assertion path).
    ensure_init();

    let num_threads = 4;
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            std::thread::spawn(move || {
                let mut prev_count = 0u64;
                for i in 0..200 {
                    log(LogLevel::Debug, "miri_test", &format!("t{thread_id} #{i}"));

                    // Interleave dropped_count reads with log calls
                    let current = dropped_count();
                    assert!(
                        current >= prev_count,
                        "dropped_count must be monotonically non-decreasing"
                    );
                    prev_count = current;
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

// ─── 18.3: Latency Test (Non-Blocking Check) ───────────────────────────────

#[test]
fn log_call_returns_within_1ms() {
    // Validates: Requirement 8.2
    // Measures the wall-clock time of individual log() calls.
    // Each call should return within 1ms because it only performs:
    // 1. An atomic level check
    // 2. Record formatting on the caller's thread
    // 3. A non-blocking try_send() to the channel
    ensure_init();

    // Warm up: a few calls to stabilize any lazy initialization
    for _ in 0..10 {
        log(LogLevel::Info, "warmup", "warmup message");
    }

    // Measure 100 individual log calls
    let mut max_elapsed = Duration::ZERO;
    for i in 0..100 {
        let start = Instant::now();
        log(
            LogLevel::Info,
            "latency_test",
            &format!("Latency measurement iteration {i}"),
        );
        let elapsed = start.elapsed();

        if elapsed > max_elapsed {
            max_elapsed = elapsed;
        }
    }

    assert!(
        max_elapsed < Duration::from_millis(1),
        "log() call should return within 1ms, but worst-case was {:?}",
        max_elapsed
    );
}

#[test]
fn log_lazy_call_returns_within_1ms() {
    // Validates: Requirement 8.2
    // Same as above but for log_lazy() which uses a closure for formatting.
    ensure_init();

    // Warm up
    for _ in 0..10 {
        ff_logging::log_lazy(LogLevel::Info, "warmup", || "warmup lazy".to_string());
    }

    let mut max_elapsed = Duration::ZERO;
    for i in 0..100 {
        let start = Instant::now();
        ff_logging::log_lazy(LogLevel::Info, "latency_test", || {
            format!("Lazy latency measurement iteration {i}")
        });
        let elapsed = start.elapsed();

        if elapsed > max_elapsed {
            max_elapsed = elapsed;
        }
    }

    assert!(
        max_elapsed < Duration::from_millis(1),
        "log_lazy() call should return within 1ms, but worst-case was {:?}",
        max_elapsed
    );
}

// ─── 18.4: Send + Sync Assertions ──────────────────────────────────────────

#[test]
fn public_types_implement_send_and_sync() {
    // Validates: Requirement 8.1
    // Compile-time assertions that all public types are Send + Sync.
    // If any type fails these bounds, this test will not compile.
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    // LogLevel
    assert_send::<LogLevel>();
    assert_sync::<LogLevel>();

    // LogRecord
    assert_send::<LogRecord>();
    assert_sync::<LogRecord>();

    // LogConfig
    assert_send::<LogConfig>();
    assert_sync::<LogConfig>();

    // LoggingStatus
    assert_send::<LoggingStatus>();
    assert_sync::<LoggingStatus>();

    // PluginLogHandle trait object
    assert_send::<Box<dyn PluginLogHandle>>();
    assert_sync::<Box<dyn PluginLogHandle>>();
}

#[test]
fn log_config_can_be_sent_across_threads() {
    // Validates: Requirement 8.1
    // Runtime verification that LogConfig can be moved to another thread.
    let config = LogConfig {
        level: LogLevel::Info,
        directory: std::path::PathBuf::from("/tmp/test"),
        max_file_size_mb: 10,
        max_retained_files: 5,
    };

    let handle = std::thread::spawn(move || {
        // Access the config on a different thread
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.max_file_size_mb, 10);
    });

    handle.join().expect("thread should not panic");
}

#[test]
fn plugin_handle_can_be_shared_across_threads_via_arc() {
    // Validates: Requirement 8.1, 10.6
    // Verifies that the plugin handle works correctly when shared via Arc.
    use std::sync::Arc;

    ensure_init();

    let handle: Arc<dyn PluginLogHandle> = Arc::from(create_plugin_handle("arc-test"));

    let threads: Vec<_> = (0..8)
        .map(|id| {
            let h = Arc::clone(&handle);
            std::thread::spawn(move || {
                for i in 0..100 {
                    h.info("shared", &format!("thread {id} msg {i}"));
                }
            })
        })
        .collect();

    for t in threads {
        t.join().expect("thread should not panic");
    }
}
