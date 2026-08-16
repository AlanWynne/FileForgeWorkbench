//! Configuration types for the background I/O subsystem.
//!
//! Defines [`IoConfig`] — the central configuration struct with validated,
//! clamped fields for chunk size, thresholds, concurrency limits, and retry
//! settings.

use crate::types::{ChunkSize, LargeFileThreshold};

/// Configuration for the background I/O subsystem.
///
/// All fields are validated and clamped to their valid ranges on construction.
/// Read from the workbench configuration system at startup (namespace `io.*`).
#[derive(Debug, Clone)]
pub struct IoConfig {
    /// Default chunk size for read/write operations.
    pub chunk_size: ChunkSize,
    /// Large file threshold — files above this trigger streaming-only mode.
    pub large_file_threshold: LargeFileThreshold,
    /// Maximum concurrent I/O tasks. Range: 1–16. Default: 4.
    pub max_concurrent_tasks: u8,
    /// Maximum retry attempts for transient errors. Default: 3.
    pub retry_count: u8,
    /// Initial retry backoff in milliseconds. Default: 500.
    pub retry_backoff_ms: u64,
    /// Shutdown timeout in seconds. Default: 30.
    pub shutdown_timeout_secs: u32,
}

impl IoConfig {
    /// Minimum concurrent tasks.
    pub const MIN_CONCURRENT_TASKS: u8 = 1;
    /// Maximum concurrent tasks.
    pub const MAX_CONCURRENT_TASKS: u8 = 16;
    /// Default concurrent tasks.
    pub const DEFAULT_CONCURRENT_TASKS: u8 = 4;
    /// Default retry count.
    pub const DEFAULT_RETRY_COUNT: u8 = 3;
    /// Default retry backoff in milliseconds.
    pub const DEFAULT_RETRY_BACKOFF_MS: u64 = 500;
    /// Default shutdown timeout in seconds.
    pub const DEFAULT_SHUTDOWN_TIMEOUT_SECS: u32 = 30;

    /// Create an IoConfig with all values clamped to valid ranges.
    pub fn new(
        chunk_size_kb: u32,
        large_file_threshold_mb: u32,
        max_concurrent_tasks: u8,
        retry_count: u8,
        retry_backoff_ms: u64,
        shutdown_timeout_secs: u32,
    ) -> Self {
        Self {
            chunk_size: ChunkSize::new(chunk_size_kb * 1024),
            large_file_threshold: LargeFileThreshold::new(
                u64::from(large_file_threshold_mb) * 1024 * 1024,
            ),
            max_concurrent_tasks: max_concurrent_tasks
                .clamp(Self::MIN_CONCURRENT_TASKS, Self::MAX_CONCURRENT_TASKS),
            retry_count,
            retry_backoff_ms,
            shutdown_timeout_secs,
        }
    }
}

impl Default for IoConfig {
    fn default() -> Self {
        Self {
            chunk_size: ChunkSize::default(),
            large_file_threshold: LargeFileThreshold::default(),
            max_concurrent_tasks: Self::DEFAULT_CONCURRENT_TASKS,
            retry_count: Self::DEFAULT_RETRY_COUNT,
            retry_backoff_ms: Self::DEFAULT_RETRY_BACKOFF_MS,
            shutdown_timeout_secs: Self::DEFAULT_SHUTDOWN_TIMEOUT_SECS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        // Validates: Requirement 1 AC 7; Requirement 5 AC 2; Requirement 7 AC 2
        let config = IoConfig::default();
        assert_eq!(config.chunk_size.as_bytes(), 64 * 1024);
        assert_eq!(config.large_file_threshold.as_bytes(), 100 * 1024 * 1024);
        assert_eq!(config.max_concurrent_tasks, 4);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_backoff_ms, 500);
        assert_eq!(config.shutdown_timeout_secs, 30);
    }

    #[test]
    fn max_concurrent_tasks_clamped_below_minimum() {
        // Validates: Requirement 7 AC 2
        let config = IoConfig::new(64, 100, 0, 3, 500, 30);
        assert_eq!(config.max_concurrent_tasks, 1);
    }

    #[test]
    fn max_concurrent_tasks_clamped_above_maximum() {
        // Validates: Requirement 7 AC 2
        let config = IoConfig::new(64, 100, 255, 3, 500, 30);
        assert_eq!(config.max_concurrent_tasks, 16);
    }

    #[test]
    fn max_concurrent_tasks_accepts_values_in_range() {
        // Validates: Requirement 7 AC 2
        let config = IoConfig::new(64, 100, 8, 3, 500, 30);
        assert_eq!(config.max_concurrent_tasks, 8);
    }

    #[test]
    fn chunk_size_clamped_via_config_constructor() {
        // Validates: Requirement 1 AC 7
        // 1 KB (below minimum of 4 KB)
        let config = IoConfig::new(1, 100, 4, 3, 500, 30);
        assert_eq!(config.chunk_size.as_bytes(), ChunkSize::MIN);

        // 2048 KB (above maximum of 1024 KB)
        let config = IoConfig::new(2048, 100, 4, 3, 500, 30);
        assert_eq!(config.chunk_size.as_bytes(), ChunkSize::MAX);
    }

    #[test]
    fn large_file_threshold_clamped_via_config_constructor() {
        // Validates: Requirement 5 AC 2
        // 1 MB (below minimum of 10 MB)
        let config = IoConfig::new(64, 1, 4, 3, 500, 30);
        assert_eq!(
            config.large_file_threshold.as_bytes(),
            LargeFileThreshold::MIN
        );

        // 8192 MB (above maximum of 4096 MB)
        let config = IoConfig::new(64, 8192, 4, 3, 500, 30);
        assert_eq!(
            config.large_file_threshold.as_bytes(),
            LargeFileThreshold::MAX
        );
    }
}
