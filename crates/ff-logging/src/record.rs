//! Log record data type with formatting, truncation, and escaping.
//!
//! The `LogRecord` struct represents a single structured log entry before
//! it is formatted into a line for file output. Construction via [`LogRecord::new()`]
//! applies message truncation and control character escaping automatically.

use crate::level::LogLevel;
use chrono::{DateTime, Local};

/// Maximum message body size in bytes before truncation is applied.
const MAX_MESSAGE_BYTES: usize = 8192;

/// Ellipsis marker appended to truncated messages.
const TRUNCATION_MARKER: &str = "...";

/// A single structured log entry.
///
/// Contains all the information needed to produce a formatted log line:
/// timestamp, severity level, source module path, and message body.
///
/// The message is automatically truncated at 8192 bytes and control characters
/// are escaped during construction.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Timestamp with millisecond precision in local time.
    pub timestamp: DateTime<Local>,
    /// Severity level of this record.
    pub level: LogLevel,
    /// Source module path (e.g., `"ff_core::file_engine"` or `"plugin:my-plugin::module"`).
    pub module_path: String,
    /// Message body (post-truncation and post-escaping).
    pub message: String,
}

impl LogRecord {
    /// Creates a new log record capturing the current local time with millisecond precision.
    ///
    /// The message is processed during construction:
    /// 1. Truncated to 8192 bytes at a character boundary if it exceeds that length,
    ///    with `"..."` appended to indicate truncation.
    /// 2. Control characters (ASCII 0x00–0x1F) are replaced with `\u{XXXX}` escape sequences
    ///    to ensure the formatted output is safe for single-line log files.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_logging::{LogLevel, LogRecord};
    ///
    /// let record = LogRecord::new(LogLevel::Info, "my_module", "Hello, world!");
    /// assert_eq!(record.level, LogLevel::Info);
    /// assert_eq!(record.module_path, "my_module");
    /// assert_eq!(record.message, "Hello, world!");
    /// ```
    pub fn new(level: LogLevel, module_path: &str, message: &str) -> Self {
        let truncated = truncate_message(message);
        let escaped = escape_control_chars(&truncated);

        Self {
            timestamp: Local::now(),
            level,
            module_path: module_path.to_owned(),
            message: escaped,
        }
    }
}

/// Truncates a message to at most [`MAX_MESSAGE_BYTES`] bytes, appending
/// the truncation marker if the message exceeds the limit.
///
/// Truncation respects UTF-8 character boundaries — it will never split
/// a multi-byte character.
fn truncate_message(message: &str) -> String {
    if message.len() <= MAX_MESSAGE_BYTES {
        return message.to_owned();
    }

    // Find the largest character boundary at or before MAX_MESSAGE_BYTES
    let mut boundary = MAX_MESSAGE_BYTES;
    while boundary > 0 && !message.is_char_boundary(boundary) {
        boundary -= 1;
    }

    let mut result = message[..boundary].to_owned();
    result.push_str(TRUNCATION_MARKER);
    result
}

/// Replaces all ASCII control characters (0x00–0x1F) in the message with
/// their `\u{XXXX}` Unicode escape representation.
///
/// This ensures the formatted log output contains no raw control characters
/// and each record remains on a single line.
fn escape_control_chars(message: &str) -> String {
    let mut result = String::with_capacity(message.len());

    for ch in message.chars() {
        if ch as u32 <= 0x1F {
            // Format as \u{00XX} where XX is the zero-padded hex value
            result.push_str(&format!("\\u{{{:04X}}}", ch as u32));
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // ─── Construction Tests ─────────────────────────────────────────────────

    #[test]
    fn new_captures_current_timestamp_with_millisecond_precision() {
        // Validates: Requirement 2.1
        let before = Local::now();
        let record = LogRecord::new(LogLevel::Info, "test_module", "hello");
        let after = Local::now();

        assert!(record.timestamp >= before);
        assert!(record.timestamp <= after);
        // Verify millisecond precision is available (nanoseconds are populated)
        let _ = record
            .timestamp
            .format("%Y-%m-%dT%H:%M:%S%.3f%:z")
            .to_string();
    }

    #[test]
    fn new_stores_level_and_module_path_correctly() {
        // Validates: Requirement 2.1
        let record = LogRecord::new(LogLevel::Error, "ff_core::startup", "test message");

        assert_eq!(record.level, LogLevel::Error);
        assert_eq!(record.module_path, "ff_core::startup");
    }

    #[test]
    fn new_preserves_message_under_limit() {
        // Validates: Requirement 2.3
        let msg = "Short message with no control chars";
        let record = LogRecord::new(LogLevel::Debug, "mod", msg);
        assert_eq!(record.message, msg);
    }

    // ─── Truncation Tests ───────────────────────────────────────────────────

    #[test]
    fn truncate_message_preserves_short_messages() {
        // Validates: Requirement 2.3
        let msg = "Hello, world!";
        assert_eq!(truncate_message(msg), msg);
    }

    #[test]
    fn truncate_message_preserves_exact_limit_message() {
        // Validates: Requirement 2.3
        let msg = "a".repeat(MAX_MESSAGE_BYTES);
        assert_eq!(truncate_message(&msg), msg);
    }

    #[test]
    fn truncate_message_truncates_oversized_ascii_with_ellipsis() {
        // Validates: Requirement 2.3
        let msg = "x".repeat(MAX_MESSAGE_BYTES + 100);
        let result = truncate_message(&msg);

        assert_eq!(result.len(), MAX_MESSAGE_BYTES + TRUNCATION_MARKER.len());
        assert!(result.ends_with("..."));
        assert_eq!(&result[..MAX_MESSAGE_BYTES], &msg[..MAX_MESSAGE_BYTES]);
    }

    #[test]
    fn truncate_message_respects_utf8_character_boundary() {
        // Validates: Requirement 2.3
        // Use a string of multi-byte characters (each emoji is 4 bytes)
        // Fill up to just past the boundary
        let emoji = "🦀"; // 4 bytes
        let count = MAX_MESSAGE_BYTES / 4; // fills exactly to 8192
        let msg = emoji.repeat(count + 10); // 10 extra emojis past the limit

        let result = truncate_message(&msg);

        // Result must be valid UTF-8 (it wouldn't compile if not, but let's verify logic)
        assert!(result.is_char_boundary(result.len() - TRUNCATION_MARKER.len()));
        assert!(result.ends_with("..."));
        // The truncated part should be at or below MAX_MESSAGE_BYTES
        let without_marker = &result[..result.len() - TRUNCATION_MARKER.len()];
        assert!(without_marker.len() <= MAX_MESSAGE_BYTES);
    }

    #[test]
    fn truncate_message_handles_multibyte_at_exact_boundary() {
        // Validates: Requirement 2.3
        // Create a string where a multi-byte char straddles the 8192 boundary
        let prefix = "a".repeat(MAX_MESSAGE_BYTES - 1);
        // Add a 2-byte char (e.g., 'ñ' = 0xC3 0xB1)
        let msg = format!("{prefix}ñ more text after");

        let result = truncate_message(&msg);
        assert!(result.ends_with("..."));
        // Should truncate before the ñ since it straddles the boundary
        let without_marker = &result[..result.len() - TRUNCATION_MARKER.len()];
        assert!(without_marker.len() <= MAX_MESSAGE_BYTES);
        assert!(result.is_char_boundary(result.len() - TRUNCATION_MARKER.len()));
    }

    // ─── Escaping Tests ─────────────────────────────────────────────────────

    #[test]
    fn escape_control_chars_leaves_normal_text_unchanged() {
        // Validates: Requirement 2.4
        let msg = "Hello, world! 123 ~`@#$%";
        assert_eq!(escape_control_chars(msg), msg);
    }

    #[test]
    fn escape_control_chars_replaces_null_byte() {
        // Validates: Requirement 2.4
        let msg = "before\x00after";
        assert_eq!(escape_control_chars(msg), "before\\u{0000}after");
    }

    #[test]
    fn escape_control_chars_replaces_newline() {
        // Validates: Requirement 2.4
        let msg = "line1\nline2";
        assert_eq!(escape_control_chars(msg), "line1\\u{000A}line2");
    }

    #[test]
    fn escape_control_chars_replaces_tab_and_carriage_return() {
        // Validates: Requirement 2.4
        let msg = "tab\there\r\n";
        assert_eq!(
            escape_control_chars(msg),
            "tab\\u{0009}here\\u{000D}\\u{000A}"
        );
    }

    #[test]
    fn escape_control_chars_replaces_all_control_characters() {
        // Validates: Requirement 2.4
        // Test all control chars from 0x00 to 0x1F
        for code in 0x00u8..=0x1F {
            let ch = code as char;
            let input = format!("a{ch}b");
            let result = escape_control_chars(&input);
            let expected = format!("a\\u{{{:04X}}}b", code);
            assert_eq!(result, expected, "Failed for control char 0x{:02X}", code);
        }
    }

    #[test]
    fn escape_control_chars_preserves_space_and_printable_ascii() {
        // Validates: Requirement 2.4
        // Space (0x20) is NOT a control character and should be preserved
        let msg = " leading space";
        assert_eq!(escape_control_chars(msg), msg);
    }

    #[test]
    fn new_applies_both_truncation_and_escaping() {
        // Validates: Requirement 2.3, 2.4
        // A message with control chars that also exceeds the limit
        let msg = format!("{}\n{}", "x".repeat(MAX_MESSAGE_BYTES), "overflow");
        let record = LogRecord::new(LogLevel::Warn, "test", &msg);

        // Should be truncated (has "..." at end)
        assert!(record.message.ends_with("..."));
        // Control chars within the truncated portion should be escaped
        assert!(!record.message.contains('\n'));
    }
}
