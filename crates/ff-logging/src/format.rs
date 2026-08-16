//! Log record formatting and parsing utilities.
//!
//! Provides functions to format a `LogRecord` into a single-line string
//! and to parse a formatted line back into its component fields.
//!
//! ## Format Specification
//!
//! Each log record is formatted as:
//! ```text
//! YYYY-MM-DDTHH:MM:SS.mmm±HH:MM LEVEL [module::path] message\n
//! ```
//!
//! Example:
//! ```text
//! 2025-01-20T14:30:22.456+10:00 INFO  [ff_core::startup] Application started
//! ```

use crate::level::LogLevel;
use crate::record::LogRecord;
use chrono::{DateTime, Local, TimeZone};

/// Formats a log record into a single-line string with trailing LF.
///
/// The output format is:
/// `YYYY-MM-DDTHH:MM:SS.mmm±HH:MM LEVEL [module::path] message\n`
///
/// Level names are padded to 5 characters for alignment (e.g., `INFO `, `WARN `, `ERROR`).
/// The line ending is always LF (`\n`, `0x0A`) regardless of platform.
///
/// # Examples
///
/// ```
/// use ff_logging::{LogLevel, LogRecord};
/// use ff_logging::format::format_record;
///
/// let record = LogRecord::new(LogLevel::Info, "ff_core::startup", "Application started");
/// let line = format_record(&record);
/// assert!(line.contains("INFO "));
/// assert!(line.contains("[ff_core::startup]"));
/// assert!(line.contains("Application started"));
/// assert!(line.ends_with('\n'));
/// ```
pub fn format_record(record: &LogRecord) -> String {
    let timestamp = record.timestamp.format("%Y-%m-%dT%H:%M:%S%.3f%:z");
    let level = pad_level(record.level);

    format!(
        "{} {} [{}] {}\n",
        timestamp, level, record.module_path, record.message
    )
}

/// Pads the level name to exactly 5 characters with trailing spaces.
///
/// - `TRACE` → `"TRACE"` (already 5)
/// - `DEBUG` → `"DEBUG"` (already 5)
/// - `INFO`  → `"INFO "` (padded to 5)
/// - `WARN`  → `"WARN "` (padded to 5)
/// - `ERROR` → `"ERROR"` (already 5)
fn pad_level(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "TRACE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO ",
        LogLevel::Warn => "WARN ",
        LogLevel::Error => "ERROR",
    }
}

/// Parses a formatted log line back into a `LogRecord`.
///
/// Extracts the timestamp, level, module path, and message from a line
/// that was produced by [`format_record()`]. Returns `None` if the line
/// cannot be parsed (invalid format, unknown level, etc.).
///
/// This function is primarily used for testing round-trip correctness.
///
/// # Examples
///
/// ```
/// use ff_logging::{LogLevel, LogRecord};
/// use ff_logging::format::{format_record, parse_record};
///
/// let record = LogRecord::new(LogLevel::Warn, "my_mod", "something happened");
/// let line = format_record(&record);
/// let parsed = parse_record(&line).expect("should parse");
/// assert_eq!(parsed.level, LogLevel::Warn);
/// assert_eq!(parsed.module_path, "my_mod");
/// assert_eq!(parsed.message, "something happened");
/// ```
pub fn parse_record(line: &str) -> Option<LogRecord> {
    // Strip trailing newline if present
    let line = line.strip_suffix('\n').unwrap_or(line);

    // Format: "YYYY-MM-DDTHH:MM:SS.mmm±HH:MM LEVEL [module::path] message"
    // Timestamp is fixed-width: 29 chars (e.g., "2025-01-20T14:30:22.456+10:00")
    // Then a space, then level (5 chars), then a space, then "[module]", then space, then message.

    // Find the first space after the timestamp
    // The timestamp format is: YYYY-MM-DDTHH:MM:SS.mmm±HH:MM (29 chars)
    if line.len() < 30 {
        return None;
    }

    let timestamp_str = &line[..29];
    let rest = &line[30..]; // skip the space after timestamp

    // Parse timestamp
    let timestamp = DateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S%.3f%:z")
        .ok()
        .map(|dt| Local.from_utc_datetime(&dt.naive_utc()))?;

    // Rest is: "LEVEL [module::path] message"
    // Level is 5 chars (padded)
    if rest.len() < 6 {
        return None;
    }

    let level_str = rest[..5].trim();
    let level = level_str.parse::<LogLevel>().ok()?;

    // After level and space: "[module::path] message"
    let after_level = &rest[6..]; // skip "LEVEL "

    // Find the module path between brackets
    if !after_level.starts_with('[') {
        return None;
    }

    let close_bracket = after_level.find(']')?;
    let module_path = &after_level[1..close_bracket];

    // Message is everything after "] "
    let message_start = close_bracket + 2; // skip "] "
    let message = if message_start <= after_level.len() {
        &after_level[message_start..]
    } else {
        ""
    };

    Some(LogRecord {
        timestamp,
        level,
        module_path: module_path.to_owned(),
        message: message.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // ─── Format Tests ───────────────────────────────────────────────────────

    #[test]
    fn format_record_produces_correct_structure() {
        // Validates: Requirement 2.1
        let record = LogRecord::new(LogLevel::Info, "ff_core::startup", "Application started");
        let line = format_record(&record);

        // Should contain timestamp, level, module in brackets, and message
        assert!(line.contains("[ff_core::startup]"));
        assert!(line.contains("Application started"));
        assert!(line.ends_with('\n'));
    }

    #[test]
    fn format_record_pads_info_level_to_5_chars() {
        // Validates: Requirement 2.1
        let record = LogRecord::new(LogLevel::Info, "mod", "msg");
        let line = format_record(&record);
        assert!(
            line.contains("INFO "),
            "Expected 'INFO ' with trailing space, got: {line}"
        );
    }

    #[test]
    fn format_record_pads_warn_level_to_5_chars() {
        // Validates: Requirement 2.1
        let record = LogRecord::new(LogLevel::Warn, "mod", "msg");
        let line = format_record(&record);
        assert!(
            line.contains("WARN "),
            "Expected 'WARN ' with trailing space, got: {line}"
        );
    }

    #[test]
    fn format_record_uses_5_char_error_level() {
        // Validates: Requirement 2.1
        let record = LogRecord::new(LogLevel::Error, "mod", "msg");
        let line = format_record(&record);
        assert!(line.contains("ERROR"), "Expected 'ERROR', got: {line}");
    }

    #[test]
    fn format_record_uses_5_char_trace_level() {
        // Validates: Requirement 2.1
        let record = LogRecord::new(LogLevel::Trace, "mod", "msg");
        let line = format_record(&record);
        assert!(line.contains("TRACE"), "Expected 'TRACE', got: {line}");
    }

    #[test]
    fn format_record_uses_5_char_debug_level() {
        // Validates: Requirement 2.1
        let record = LogRecord::new(LogLevel::Debug, "mod", "msg");
        let line = format_record(&record);
        assert!(line.contains("DEBUG"), "Expected 'DEBUG', got: {line}");
    }

    #[test]
    fn format_record_ends_with_lf_not_crlf() {
        // Validates: Requirement 2.2
        let record = LogRecord::new(LogLevel::Info, "mod", "msg");
        let line = format_record(&record);
        assert!(line.ends_with('\n'));
        assert!(!line.ends_with("\r\n"));
    }

    #[test]
    fn format_record_timestamp_matches_iso8601_with_millis() {
        // Validates: Requirement 2.1
        let record = LogRecord::new(LogLevel::Info, "mod", "msg");
        let line = format_record(&record);

        // Extract timestamp (first 29 chars)
        let ts_str = &line[..29];
        // Should parse back as a valid datetime
        let parsed = DateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S%.3f%:z");
        assert!(
            parsed.is_ok(),
            "Timestamp '{}' did not match expected format",
            ts_str
        );
    }

    #[test]
    fn format_record_all_levels_produce_valid_output() {
        // Validates: Requirement 2.1
        let levels = [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];

        for level in levels {
            let record = LogRecord::new(level, "test::module", "test message");
            let line = format_record(&record);

            assert!(line.ends_with('\n'), "Level {:?} missing LF", level);
            assert!(
                line.contains("[test::module]"),
                "Level {:?} missing module",
                level
            );
            assert!(
                line.contains("test message"),
                "Level {:?} missing message",
                level
            );

            // Should be parseable
            let parsed = parse_record(&line);
            assert!(parsed.is_some(), "Level {:?} failed to parse", level);
        }
    }

    // ─── Parse Tests ────────────────────────────────────────────────────────

    #[test]
    fn parse_record_returns_none_for_empty_string() {
        // Validates: Requirement 2.5
        assert!(parse_record("").is_none());
    }

    #[test]
    fn parse_record_returns_none_for_malformed_input() {
        // Validates: Requirement 2.5
        assert!(parse_record("not a log line at all").is_none());
        assert!(parse_record("2025-01-20T14:30:22.456+10:00").is_none());
        assert!(parse_record("2025-01-20T14:30:22.456+10:00 INVALID [mod] msg\n").is_none());
    }

    #[test]
    fn parse_record_extracts_all_fields_correctly() {
        // Validates: Requirement 2.5
        let record = LogRecord::new(LogLevel::Warn, "ff_core::engine", "Something happened");
        let line = format_record(&record);
        let parsed = parse_record(&line).expect("should parse successfully");

        assert_eq!(parsed.level, LogLevel::Warn);
        assert_eq!(parsed.module_path, "ff_core::engine");
        assert_eq!(parsed.message, "Something happened");
    }

    // ─── Round-Trip Tests ───────────────────────────────────────────────────

    #[test]
    fn round_trip_preserves_level() {
        // Validates: Requirement 2.5
        let levels = [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];

        for level in levels {
            let record = LogRecord::new(level, "module", "message");
            let line = format_record(&record);
            let parsed = parse_record(&line).expect("should parse");
            assert_eq!(parsed.level, level, "Round-trip failed for {:?}", level);
        }
    }

    #[test]
    fn round_trip_preserves_module_path() {
        // Validates: Requirement 2.5
        let paths = [
            "simple",
            "ff_core::startup",
            "plugin:my_plugin::renderer",
            "deeply::nested::module::path",
        ];

        for path in paths {
            let record = LogRecord::new(LogLevel::Info, path, "msg");
            let line = format_record(&record);
            let parsed = parse_record(&line).expect("should parse");
            assert_eq!(
                parsed.module_path, path,
                "Round-trip failed for path: {path}"
            );
        }
    }

    #[test]
    fn round_trip_preserves_message() {
        // Validates: Requirement 2.5
        let messages = [
            "Simple message",
            "Message with special chars: <>&\"'",
            "Message with unicode: 日本語テスト 🦀",
            "",
        ];

        for msg in messages {
            let record = LogRecord::new(LogLevel::Info, "mod", msg);
            let line = format_record(&record);
            let parsed = parse_record(&line).expect("should parse");
            assert_eq!(
                parsed.message, record.message,
                "Round-trip failed for: {msg}"
            );
        }
    }

    #[test]
    fn round_trip_preserves_timestamp() {
        // Validates: Requirement 2.5
        let record = LogRecord::new(LogLevel::Info, "mod", "msg");
        let line = format_record(&record);
        let parsed = parse_record(&line).expect("should parse");

        // Timestamps should be equal to millisecond precision
        let orig_ms = record
            .timestamp
            .format("%Y-%m-%dT%H:%M:%S%.3f%:z")
            .to_string();
        let parsed_ms = parsed
            .timestamp
            .format("%Y-%m-%dT%H:%M:%S%.3f%:z")
            .to_string();
        assert_eq!(orig_ms, parsed_ms);
    }

    #[test]
    fn round_trip_with_escaped_control_chars() {
        // Validates: Requirement 2.4, 2.5
        let record = LogRecord::new(LogLevel::Info, "mod", "line1\nline2\ttab");
        // The message should have been escaped by LogRecord::new()
        assert!(!record.message.contains('\n'));
        assert!(!record.message.contains('\t'));

        let line = format_record(&record);
        let parsed = parse_record(&line).expect("should parse");
        assert_eq!(parsed.message, record.message);
    }
}
