//! Large-file status indicators for the status bar.

use std::time::{Duration, Instant};

/// Large-file status data for status bar display.
///
/// Tracks file size, line count, loading progress, and layout progress.
pub struct StatusIndicator {
    /// Whether the current file exceeds the large-file threshold.
    pub is_large_file: bool,
    /// File size in bytes (for display).
    pub file_size_bytes: u64,
    /// Total line count (None while counting).
    pub total_lines: Option<u64>,
    /// Loading progress percentage (None if not loading).
    pub loading_progress: Option<u8>,
    /// Layout computation progress (fraction of lines with layouts).
    pub layout_progress: Option<f64>,
    /// Whether layout computation is paused (user is editing).
    pub layout_paused: bool,
    /// Timestamp when last progress indicator completed (for fade timer).
    pub completion_time: Option<Instant>,
}

impl StatusIndicator {
    /// Create a new StatusIndicator (inactive state).
    pub fn new() -> Self {
        Self {
            is_large_file: false,
            file_size_bytes: 0,
            total_lines: None,
            loading_progress: None,
            layout_progress: None,
            layout_paused: false,
            completion_time: None,
        }
    }

    /// Format file size as human-readable string (e.g., "245 MB").
    ///
    /// Returns `None` if not a large file.
    pub fn formatted_file_size(&self) -> Option<String> {
        if !self.is_large_file {
            return None;
        }
        let bytes = self.file_size_bytes;
        if bytes >= 1024 * 1024 * 1024 {
            Some(format!(
                "{:.1} GB",
                bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            ))
        } else if bytes >= 1024 * 1024 {
            Some(format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0)))
        } else if bytes >= 1024 {
            Some(format!("{:.0} KB", bytes as f64 / 1024.0))
        } else {
            Some(format!("{bytes} B"))
        }
    }

    /// Format line count for display (or "counting…" placeholder).
    pub fn formatted_line_count(&self) -> String {
        match self.total_lines {
            Some(n) => format!("{n} lines"),
            None => "counting…".to_string(),
        }
    }

    /// Whether the status indicator should be visible.
    pub fn is_visible(&self) -> bool {
        self.is_large_file
    }

    /// Whether a completion indicator should still be shown (within 5s fade).
    pub fn is_showing_completion(&self) -> bool {
        if let Some(t) = self.completion_time {
            t.elapsed() < Duration::from_secs(5)
        } else {
            false
        }
    }

    /// Mark loading as complete.
    pub fn mark_complete(&mut self) {
        self.loading_progress = None;
        self.layout_progress = Some(1.0);
        self.completion_time = Some(Instant::now());
    }
}

impl Default for StatusIndicator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_visible_for_small_file() {
        // Validates: Requirement 6 AC 5
        let status = StatusIndicator::new();
        assert!(!status.is_visible());
        assert!(status.formatted_file_size().is_none());
    }

    #[test]
    fn visible_for_large_file() {
        // Validates: Requirement 6 AC 1
        let mut status = StatusIndicator::new();
        status.is_large_file = true;
        status.file_size_bytes = 245 * 1024 * 1024;
        assert!(status.is_visible());
        let size_str = status.formatted_file_size().unwrap();
        assert!(size_str.contains("MB"));
    }

    #[test]
    fn line_count_placeholder_while_counting() {
        // Validates: Requirement 6 AC 2
        let status = StatusIndicator::new();
        assert_eq!(status.formatted_line_count(), "counting…");
    }

    #[test]
    fn line_count_shows_when_known() {
        // Validates: Requirement 6 AC 2
        let mut status = StatusIndicator::new();
        status.total_lines = Some(1_234_567);
        assert!(status.formatted_line_count().contains("1234567"));
    }

    #[test]
    fn completion_shows_within_5_seconds() {
        // Validates: Requirement 6 AC 6
        let mut status = StatusIndicator::new();
        status.mark_complete();
        assert!(status.is_showing_completion());
    }

    #[test]
    fn gb_formatting() {
        let mut status = StatusIndicator::new();
        status.is_large_file = true;
        status.file_size_bytes = 2 * 1024 * 1024 * 1024;
        let s = status.formatted_file_size().unwrap();
        assert!(s.contains("GB"));
    }
}
