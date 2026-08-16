//! Error types for the large-file-performance subsystem.

/// Errors originating from the ff-large-file-performance crate.
///
/// All `Display` output follows `[large-file-perf] operation: description`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LargeFilePerfError {
    /// Line is not yet available (progressive loading in progress).
    #[error("[large-file-perf] layout: line {line_number} not yet loaded (frontier: {frontier})")]
    LineNotAvailable { line_number: u64, frontier: u64 },

    /// Display line number is out of valid range.
    #[error("[large-file-perf] layout: display line {display_line} out of range (total: {total_display_lines})")]
    DisplayLineOutOfRange {
        display_line: u64,
        total_display_lines: u64,
    },

    /// Frame budget exceeded during measurement — layout deferred.
    #[error("[large-file-perf] measurement: frame budget exceeded after {measured_lines} lines (budget: {budget_ms}ms)")]
    FrameBudgetExceeded { measured_lines: u64, budget_ms: u32 },

    /// Memory budget exceeded — eviction required.
    #[error("[large-file-perf] cache: memory budget exceeded ({used_mb}MB / {budget_mb}MB)")]
    MemoryBudgetExceeded { used_mb: u64, budget_mb: u64 },

    /// Surface measurement failed (platform error).
    #[error(
        "[large-file-perf] measurement: surface measurement failed for style {style}: {reason}"
    )]
    MeasurementFailed { style: u16, reason: String },

    /// Configuration error.
    #[error("[large-file-perf] config: {description}")]
    ConfigError { description: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_follow_format() {
        let err = LargeFilePerfError::LineNotAvailable {
            line_number: 100,
            frontier: 50,
        };
        assert!(err.to_string().starts_with("[large-file-perf]"));
        assert!(err.to_string().contains("100"));
    }

    #[test]
    fn frame_budget_exceeded_message() {
        let err = LargeFilePerfError::FrameBudgetExceeded {
            measured_lines: 5,
            budget_ms: 12,
        };
        assert!(err.to_string().contains("12ms"));
    }
}
