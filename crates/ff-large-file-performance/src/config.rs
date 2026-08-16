//! Configuration for the large-file-performance subsystem.

use crate::types::{CacheLevel, LongLineThreshold, RenderChunkSize};
use std::time::Duration;

/// Configuration values for the large-file-performance subsystem.
///
/// Read from ff-config `[performance.*]` namespace with clamped ranges.
#[derive(Debug, Clone)]
pub struct PerfConfig {
    /// Long-line threshold in characters. Default: 10,000. Range: [1000, 100000].
    pub long_line_threshold: LongLineThreshold,

    /// Horizontal overscan margin for long-line chunked measurement.
    /// Default: 500 characters. Range: [100, 5000].
    pub long_line_overscan_chars: u32,

    /// Render chunk size for long-line subdivision. Default: 300.
    pub render_chunk_size: RenderChunkSize,

    /// PositionCache capacity (number of entries). Default: 1024. Range: [256, 16384].
    pub position_cache_size: usize,

    /// LineLayoutCache level override (None = auto-select based on file size).
    pub line_layout_cache_level: Option<CacheLevel>,

    /// Overscan buffer size in lines. Default: 5. Range: [0, 50].
    pub overscan_lines: u32,

    /// Frame budget in milliseconds. Default: 12. Range: [4, 32].
    pub frame_budget_ms: u32,

    /// Layout cache memory budget in MB. Default: 64. Range: [16, 512].
    pub layout_cache_memory_mb: u32,
}

impl Default for PerfConfig {
    fn default() -> Self {
        Self {
            long_line_threshold: LongLineThreshold::default(),
            long_line_overscan_chars: 500,
            render_chunk_size: RenderChunkSize::default(),
            position_cache_size: 1024,
            line_layout_cache_level: None,
            overscan_lines: 5,
            frame_budget_ms: 12,
            layout_cache_memory_mb: 64,
        }
    }
}

impl PerfConfig {
    /// Get the frame budget as a Duration.
    pub fn frame_budget(&self) -> Duration {
        Duration::from_millis(self.frame_budget_ms as u64)
    }

    /// Get the memory budget in bytes.
    pub fn memory_budget_bytes(&self) -> usize {
        self.layout_cache_memory_mb as usize * 1024 * 1024
    }

    /// Auto-select cache level based on document line count.
    pub fn auto_cache_level(&self, line_count: u64) -> CacheLevel {
        if let Some(level) = self.line_layout_cache_level {
            return level;
        }
        if line_count < 10_000 {
            CacheLevel::Document
        } else if line_count < 1_000_000 {
            CacheLevel::Page
        } else {
            CacheLevel::Viewport
        }
    }

    /// Clamp all values to their valid ranges.
    pub fn clamped(mut self) -> Self {
        self.long_line_overscan_chars = self.long_line_overscan_chars.clamp(100, 5000);
        self.position_cache_size = self.position_cache_size.clamp(256, 16384);
        self.overscan_lines = self.overscan_lines.clamp(0, 50);
        self.frame_budget_ms = self.frame_budget_ms.clamp(4, 32);
        self.layout_cache_memory_mb = self.layout_cache_memory_mb.clamp(16, 512);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = PerfConfig::default();
        assert_eq!(cfg.long_line_threshold.0, 10_000);
        assert_eq!(cfg.long_line_overscan_chars, 500);
        assert_eq!(cfg.render_chunk_size.0, 300);
        assert_eq!(cfg.position_cache_size, 1024);
        assert_eq!(cfg.overscan_lines, 5);
        assert_eq!(cfg.frame_budget_ms, 12);
        assert_eq!(cfg.layout_cache_memory_mb, 64);
    }

    #[test]
    fn clamping_enforces_ranges() {
        // Validates: Requirement 1 AC 5, AC 9, Requirement 2 AC 3, Requirement 4 AC 2, AC 6
        let cfg = PerfConfig {
            long_line_overscan_chars: 0,
            position_cache_size: 10,
            overscan_lines: 100,
            frame_budget_ms: 0,
            layout_cache_memory_mb: 1,
            ..Default::default()
        }
        .clamped();
        assert_eq!(cfg.long_line_overscan_chars, 100);
        assert_eq!(cfg.position_cache_size, 256);
        assert_eq!(cfg.overscan_lines, 50);
        assert_eq!(cfg.frame_budget_ms, 4);
        assert_eq!(cfg.layout_cache_memory_mb, 16);
    }

    #[test]
    fn auto_cache_level_small_file() {
        let cfg = PerfConfig::default();
        assert_eq!(cfg.auto_cache_level(5_000), CacheLevel::Document);
    }

    #[test]
    fn auto_cache_level_medium_file() {
        let cfg = PerfConfig::default();
        assert_eq!(cfg.auto_cache_level(500_000), CacheLevel::Page);
    }

    #[test]
    fn auto_cache_level_large_file() {
        let cfg = PerfConfig::default();
        assert_eq!(cfg.auto_cache_level(2_000_000), CacheLevel::Viewport);
    }

    #[test]
    fn manual_override_respected() {
        let cfg = PerfConfig {
            line_layout_cache_level: Some(CacheLevel::Document),
            ..Default::default()
        };
        assert_eq!(cfg.auto_cache_level(5_000_000), CacheLevel::Document);
    }

    #[test]
    fn frame_budget_as_duration() {
        let cfg = PerfConfig::default();
        assert_eq!(cfg.frame_budget(), std::time::Duration::from_millis(12));
    }

    #[test]
    fn memory_budget_bytes() {
        let cfg = PerfConfig::default();
        assert_eq!(cfg.memory_budget_bytes(), 64 * 1024 * 1024);
    }
}
