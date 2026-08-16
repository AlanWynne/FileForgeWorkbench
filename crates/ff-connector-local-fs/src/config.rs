//! Configuration for the local filesystem provider.
//!
//! Reads from the `[vfs.local]` TOML namespace and provides validated defaults.

use std::time::Duration;

/// Configuration for the local filesystem provider.
///
/// Read from the `[vfs.local]` TOML namespace in the workbench configuration.
///
/// Addresses: Requirement 3, criteria 5–7 (debounce config)
#[derive(Debug, Clone)]
pub struct LocalFsConfig {
    /// Debounce window for file watching (milliseconds).
    /// Range: 50–5000. Default: 500.
    pub debounce_ms: u64,
    /// Default chunk size for streaming reads (bytes).
    /// Range: 4096–1048576. Default: 65536 (64 KB).
    pub chunk_size: usize,
    /// Whether to use memory-mapped I/O when available.
    /// Default: true.
    pub enable_mmap: bool,
}

impl LocalFsConfig {
    /// Minimum allowed debounce window in milliseconds.
    pub const MIN_DEBOUNCE_MS: u64 = 50;
    /// Maximum allowed debounce window in milliseconds.
    pub const MAX_DEBOUNCE_MS: u64 = 5000;
    /// Default debounce window in milliseconds.
    pub const DEFAULT_DEBOUNCE_MS: u64 = 500;

    /// Minimum chunk size in bytes.
    pub const MIN_CHUNK_SIZE: usize = 4096;
    /// Maximum chunk size in bytes.
    pub const MAX_CHUNK_SIZE: usize = 1_048_576;
    /// Default chunk size in bytes (64 KB).
    pub const DEFAULT_CHUNK_SIZE: usize = 65536;

    /// Returns the debounce duration, clamped to the valid range.
    /// Logs a WARN if clamping was necessary.
    ///
    /// Validates: Requirement 3 AC 6, AC 7
    pub fn debounce_duration(&self) -> Duration {
        let clamped = self
            .debounce_ms
            .clamp(Self::MIN_DEBOUNCE_MS, Self::MAX_DEBOUNCE_MS);
        if clamped != self.debounce_ms {
            ff_logging::log_warn!(
                "[connector-local-fs] config: debounce_ms {} outside valid range [{}-{}], clamped to {}",
                self.debounce_ms,
                Self::MIN_DEBOUNCE_MS,
                Self::MAX_DEBOUNCE_MS,
                clamped
            );
        }
        Duration::from_millis(clamped)
    }

    /// Returns the chunk size, clamped to the valid range.
    pub fn effective_chunk_size(&self) -> usize {
        let clamped = self
            .chunk_size
            .clamp(Self::MIN_CHUNK_SIZE, Self::MAX_CHUNK_SIZE);
        if clamped != self.chunk_size {
            ff_logging::log_warn!(
                "[connector-local-fs] config: chunk_size {} outside valid range [{}-{}], clamped to {}",
                self.chunk_size,
                Self::MIN_CHUNK_SIZE,
                Self::MAX_CHUNK_SIZE,
                clamped
            );
        }
        clamped
    }
}

impl Default for LocalFsConfig {
    fn default() -> Self {
        Self {
            debounce_ms: Self::DEFAULT_DEBOUNCE_MS,
            chunk_size: Self::DEFAULT_CHUNK_SIZE,
            enable_mmap: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = LocalFsConfig::default();
        assert_eq!(config.debounce_ms, 500);
        assert_eq!(config.chunk_size, 65536);
        assert!(config.enable_mmap);
    }

    #[test]
    fn debounce_duration_clamps_below_minimum() {
        let config = LocalFsConfig {
            debounce_ms: 10,
            ..Default::default()
        };
        assert_eq!(config.debounce_duration(), Duration::from_millis(50));
    }

    #[test]
    fn debounce_duration_clamps_above_maximum() {
        let config = LocalFsConfig {
            debounce_ms: 10_000,
            ..Default::default()
        };
        assert_eq!(config.debounce_duration(), Duration::from_millis(5000));
    }

    #[test]
    fn debounce_duration_within_range_returns_exact() {
        let config = LocalFsConfig {
            debounce_ms: 250,
            ..Default::default()
        };
        assert_eq!(config.debounce_duration(), Duration::from_millis(250));
    }

    #[test]
    fn effective_chunk_size_clamps_below_minimum() {
        let config = LocalFsConfig {
            chunk_size: 100,
            ..Default::default()
        };
        assert_eq!(config.effective_chunk_size(), 4096);
    }

    #[test]
    fn effective_chunk_size_clamps_above_maximum() {
        let config = LocalFsConfig {
            chunk_size: 10_000_000,
            ..Default::default()
        };
        assert_eq!(config.effective_chunk_size(), 1_048_576);
    }
}
