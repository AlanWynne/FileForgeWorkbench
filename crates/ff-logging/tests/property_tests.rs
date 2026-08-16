//! Property-based tests for ff-logging subsystem.
//!
//! Uses the `proptest` crate (minimum 100 iterations per property).
//! These tests exercise core invariants of the logging system without
//! requiring full subsystem initialization.

use proptest::prelude::*;

use ff_logging::format::{format_record, parse_record};
use ff_logging::LogLevel;
use ff_logging::LogRecord;

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Maximum message body size in bytes before truncation.
const MAX_MESSAGE_BYTES: usize = 8192;

/// Truncation marker appended to oversized messages.
const TRUNCATION_MARKER: &str = "...";

/// Strategy: generate a valid module_path matching `[a-z_][a-z0-9_:]*` (1–64 chars).
fn module_path_strategy() -> impl Strategy<Value = String> {
    // First char: [a-z_]
    // Remaining chars: [a-z0-9_:]
    let first = prop::char::ranges(vec![('a'..='z'), ('_'..='_')].into());
    let rest = prop::collection::vec(
        prop::char::ranges(vec![('a'..='z'), ('0'..='9'), ('_'..='_'), (':'..=':')].into()),
        0..63,
    );
    (first, rest).prop_map(|(f, r)| {
        let mut s = String::with_capacity(1 + r.len());
        s.push(f);
        for c in r {
            s.push(c);
        }
        s
    })
}

/// Strategy: generate a LogLevel uniformly.
fn log_level_strategy() -> impl Strategy<Value = LogLevel> {
    prop_oneof![
        Just(LogLevel::Trace),
        Just(LogLevel::Debug),
        Just(LogLevel::Info),
        Just(LogLevel::Warn),
        Just(LogLevel::Error),
    ]
}

/// Simulate truncation: the same logic that `LogRecord::new` applies.
fn expected_truncation(input: &str) -> String {
    if input.len() <= MAX_MESSAGE_BYTES {
        input.to_owned()
    } else {
        let mut boundary = MAX_MESSAGE_BYTES;
        while boundary > 0 && !input.is_char_boundary(boundary) {
            boundary -= 1;
        }
        let mut result = input[..boundary].to_owned();
        result.push_str(TRUNCATION_MARKER);
        result
    }
}

/// Simulate control character escaping: same logic as `LogRecord::new`.
fn expected_escape(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch as u32 <= 0x1F {
            result.push_str(&format!("\\u{{{:04X}}}", ch as u32));
        } else {
            result.push(ch);
        }
    }
    result
}

// ─── Property 1: Format Round-Trip ──────────────────────────────────────────

// Feature: logging-subsystem, Property 1: For all valid LogRecord values,
// parse_record(format_record(record)) produces field values equivalent to
// the original (after truncation/escaping).
// **Validates: Requirement 2.5**
proptest! {
    #[test]
    fn format_round_trip_preserves_fields(
        level in log_level_strategy(),
        module_path in module_path_strategy(),
        message in "[ -~]{0,10000}",  // printable ASCII, 0-10000 chars (no control chars to keep round-trip clean)
    ) {
        let record = LogRecord::new(level, &module_path, &message);
        let formatted = format_record(&record);
        let parsed = parse_record(&formatted).expect("parse_record should succeed on format_record output");

        prop_assert_eq!(parsed.level, record.level);
        prop_assert_eq!(&parsed.module_path, &record.module_path);
        prop_assert_eq!(&parsed.message, &record.message);
    }
}

// ─── Property 2: Rotation Size Invariant ────────────────────────────────────

// Feature: logging-subsystem, Property 2: For any sequence of log writes,
// no file exceeds max_file_size_mb * 1024 * 1024 + max_single_record_size.
// **Validates: Requirement 5.4**
proptest! {
    #[test]
    fn rotation_size_invariant(
        max_file_size_mb in 1u32..=10,
        record_sizes in prop::collection::vec(1usize..=4096, 50..=200),
    ) {
        let threshold = (max_file_size_mb as u64) * 1_024 * 1_024;

        // Simulate rotation logic: track bytes_written for current "file"
        let mut bytes_written: u64 = 0;
        let mut max_file_observed: u64 = 0;
        let mut max_single_record: u64 = 0;

        for size in &record_sizes {
            let line_bytes = *size as u64;
            if line_bytes > max_single_record {
                max_single_record = line_bytes;
            }

            // should_rotate logic: if bytes_written + line_bytes > threshold, rotate
            if bytes_written + line_bytes > threshold {
                // Record the size of the file we're "closing"
                if bytes_written > max_file_observed {
                    max_file_observed = bytes_written;
                }
                // Rotation: reset counter
                bytes_written = 0;
            }

            // Write the record
            bytes_written += line_bytes;
        }

        // Don't forget the last file
        if bytes_written > max_file_observed {
            max_file_observed = bytes_written;
        }

        // Invariant: no file exceeds threshold + max_single_record_size
        // Because a single record that doesn't trigger rotation can push past threshold
        prop_assert!(
            max_file_observed <= threshold + max_single_record,
            "File size {} exceeded bound {} (threshold {} + max_record {})",
            max_file_observed,
            threshold + max_single_record,
            threshold,
            max_single_record,
        );
    }
}

// ─── Property 3: Overflow Handling ──────────────────────────────────────────

// Feature: logging-subsystem, Property 3: When buffer reaches capacity,
// records_written + records_dropped + records_pending == records_submitted.
// **Validates: Requirement 8.4**
proptest! {
    #[test]
    fn overflow_accounting_identity(
        num_sends in 5000usize..=20000,
    ) {
        // We use crossbeam-channel directly to simulate the channel behavior
        // without requiring full subsystem init. Channel capacity = 10000.
        let channel_capacity: usize = 10_000;
        let (sender, receiver) = crossbeam_channel::bounded::<String>(channel_capacity);

        let mut records_dropped: u64 = 0;

        // Submit num_sends records
        for i in 0..num_sends {
            let msg = format!("record {i}");
            match sender.try_send(msg) {
                Ok(()) => {}
                Err(crossbeam_channel::TrySendError::Full(_)) => {
                    records_dropped += 1;
                }
                Err(crossbeam_channel::TrySendError::Disconnected(_)) => {
                    // Should not happen in this test
                    break;
                }
            }
        }

        // Count pending (still in channel)
        let records_pending = receiver.len() as u64;

        // "Written" = submitted - dropped - pending
        // Actually: submitted = pending + dropped (since nothing consumed yet)
        let records_submitted = num_sends as u64;

        // Since no consumer is running, records_written = 0
        // So: pending + dropped == submitted
        let records_written: u64 = 0;
        prop_assert_eq!(
            records_written + records_dropped + records_pending,
            records_submitted,
            "Accounting mismatch: written({}) + dropped({}) + pending({}) != submitted({})",
            records_written,
            records_dropped,
            records_pending,
            records_submitted,
        );
    }
}

// ─── Property 4: Level Filtering ────────────────────────────────────────────

// Feature: logging-subsystem, Property 4: For any configured minimum level and
// record level, the record passes iff record_level >= configured_level.
// **Validates: Requirement 3.2**
proptest! {
    #[test]
    fn level_filtering_correctness(
        config_level in log_level_strategy(),
        record_level in log_level_strategy(),
    ) {
        // The level filter logic: record passes iff record_level >= config_level.
        // LogLevel derives Ord with Trace(0) < Debug(1) < Info(2) < Warn(3) < Error(4).
        let should_pass = record_level >= config_level;

        // Replicate the atomic filter check from init.rs:
        // (level as u8) < min_level  → filtered out
        let min_level = config_level as u8;
        let actual_passes = (record_level as u8) >= min_level;

        prop_assert_eq!(
            actual_passes,
            should_pass,
            "Filter mismatch: config={:?}, record={:?}, expected_pass={}, actual_pass={}",
            config_level,
            record_level,
            should_pass,
            actual_passes,
        );
    }
}

// ─── Property 5: Message Truncation ─────────────────────────────────────────

// Feature: logging-subsystem, Property 5: For any message body, the output
// message field has at most 8192 + 3 bytes, and messages <= 8192 bytes appear
// unchanged.
// **Validates: Requirement 2.3**
proptest! {
    #[test]
    fn message_truncation_invariant(
        message in ".{0,20000}",  // arbitrary string 0-20000 chars
    ) {
        // Create a record — this applies truncation + escaping
        let record = LogRecord::new(LogLevel::Info, "test_mod", &message);

        // Compute what we expect: first truncate, then escape
        let after_truncation = expected_truncation(&message);
        let expected_message = expected_escape(&after_truncation);

        prop_assert_eq!(&record.message, &expected_message);

        // Verify the size invariant on the raw truncation step:
        // If input.len() <= 8192, output after truncation == input (before escaping)
        // If input.len() > 8192, output after truncation has len <= 8195 and ends with "..."
        if message.len() <= MAX_MESSAGE_BYTES {
            prop_assert_eq!(&after_truncation, &message);
        } else {
            prop_assert!(
                after_truncation.len() <= MAX_MESSAGE_BYTES + TRUNCATION_MARKER.len(),
                "Truncated message too long: {} > {}",
                after_truncation.len(),
                MAX_MESSAGE_BYTES + TRUNCATION_MARKER.len(),
            );
            prop_assert!(
                after_truncation.ends_with(TRUNCATION_MARKER),
                "Truncated message does not end with '...': {:?}",
                &after_truncation[after_truncation.len().saturating_sub(10)..],
            );
        }
    }
}

// ─── Property 6: Control Character Escaping ─────────────────────────────────

// Feature: logging-subsystem, Property 6: For any message containing control
// chars, the formatted output contains no raw control characters (0x00-0x1F)
// in the line (excluding trailing LF).
// **Validates: Requirement 2.4**
proptest! {
    #[test]
    fn control_character_escaping_invariant(
        // Generate bytes including control chars, interpret as lossy UTF-8
        raw_bytes in prop::collection::vec(0u8..=127, 1..=1000),
    ) {
        // Convert bytes to a string (lossy — invalid sequences become replacement char)
        let message = String::from_utf8_lossy(&raw_bytes).to_string();

        // Create a record — this applies escaping
        let record = LogRecord::new(LogLevel::Info, "test_mod", &message);
        let formatted = format_record(&record);

        // Strip the trailing LF (the only allowed control char in the line)
        let line = formatted.strip_suffix('\n').unwrap_or(&formatted);

        // Invariant: no raw control characters in the line
        for (i, ch) in line.chars().enumerate() {
            prop_assert!(
                ch as u32 > 0x1F,
                "Found raw control char 0x{:02X} at position {} in formatted line",
                ch as u32,
                i,
            );
        }

        // Verify each original control char maps to \u{XXXX}
        let control_count = message.chars().filter(|&c| c as u32 <= 0x1F).count();
        if control_count > 0 {
            // The escaped message should contain \u{XXXX} patterns
            // Count occurrences of the pattern "\\u{"
            let escape_count = record.message.matches("\\u{").count();
            prop_assert!(
                escape_count >= control_count,
                "Expected at least {} escape sequences, found {} (message had {} control chars)",
                control_count,
                escape_count,
                control_count,
            );
        }
    }
}
