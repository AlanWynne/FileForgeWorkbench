//! Progress reporting types for background I/O operations.
//!
//! Defines [`ProgressState`] — the data payload emitted after each chunk via
//! a `tokio::sync::watch` channel — and [`IoPhase`] — the current phase of an
//! I/O operation.

use std::fmt;
use std::time::Duration;

/// The current phase of an I/O operation.
///
/// Used as a human-readable status indicator in [`ProgressState`] and
/// displayed in the task manager UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IoPhase {
    /// Queued, waiting for a concurrency slot.
    Queued,
    /// Reading chunks from VFS stream.
    Reading,
    /// Writing chunks to VFS.
    Writing,
    /// Flushing and syncing to durable storage.
    Finalizing,
    /// Operation was cancelled by the user.
    Cancelled,
    /// Operation failed with an error.
    Failed,
    /// Operation completed successfully.
    Complete,
}

impl fmt::Display for IoPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Reading => write!(f, "reading"),
            Self::Writing => write!(f, "writing"),
            Self::Finalizing => write!(f, "finalizing"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Failed => write!(f, "failed"),
            Self::Complete => write!(f, "complete"),
        }
    }
}

/// Represents the current state of an I/O operation.
///
/// Emitted after each chunk via a watch channel (latest-value semantics).
/// Contains bytes transferred, total size (if known), percentage, elapsed time,
/// estimated time remaining, and the current phase.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressState {
    /// Bytes transferred so far.
    pub bytes_transferred: u64,
    /// Total bytes (None if unknown, e.g., streaming-only providers).
    pub total_bytes: Option<u64>,
    /// Percentage complete (0–100, None if total unknown).
    pub percentage: Option<u8>,
    /// Elapsed time since task start.
    pub elapsed: Duration,
    /// Estimated time remaining (None if < 2 seconds of data).
    pub estimated_remaining: Option<Duration>,
    /// Human-readable phase description.
    pub phase: IoPhase,
}

impl ProgressState {
    /// Create a new ProgressState with initial values (zero progress, queued phase).
    pub fn new_queued() -> Self {
        Self {
            bytes_transferred: 0,
            total_bytes: None,
            percentage: None,
            elapsed: Duration::ZERO,
            estimated_remaining: None,
            phase: IoPhase::Queued,
        }
    }

    /// Calculate the percentage from bytes_transferred and total_bytes.
    ///
    /// Returns None if total_bytes is None or zero.
    /// Returns a value clamped to [0, 100].
    pub fn calculate_percentage(bytes_transferred: u64, total_bytes: Option<u64>) -> Option<u8> {
        total_bytes.map(|total| {
            if total == 0 {
                100
            } else {
                let clamped = bytes_transferred.min(total);
                // Use u128 to avoid overflow with large values
                let pct = ((clamped as u128) * 100) / (total as u128);
                pct.min(100) as u8
            }
        })
    }
}

/// Exponential Moving Average rate calculator for estimated time remaining.
///
/// Uses a 5-second window with EMA smoothing. Returns `None` if fewer than
/// 2 seconds of rate data are available.
#[derive(Debug, Clone)]
pub struct RateCalculator {
    /// Smoothed transfer rate in bytes per second.
    smoothed_rate: Option<f64>,
    /// Alpha factor for EMA (higher = more weight on recent data).
    alpha: f64,
    /// Total elapsed time with rate data.
    elapsed_with_data: Duration,
    /// Last recorded bytes_transferred for delta calculation.
    last_bytes: u64,
    /// Last recorded elapsed time.
    last_elapsed: Duration,
}

impl RateCalculator {
    /// Create a new rate calculator with a 5-second EMA window.
    ///
    /// The alpha is computed as `2 / (window_samples + 1)` where
    /// window_samples approximates 5 seconds of data at typical update rates.
    pub fn new() -> Self {
        // With chunk updates roughly every ~10ms for large files, 5 seconds ≈ 500 samples.
        // Alpha = 2/(N+1) where N is a reasonable sample count for 5s window
        // We use a more practical alpha that responds well to rate changes
        Self {
            smoothed_rate: None,
            alpha: 0.1, // Responsive but smoothed
            elapsed_with_data: Duration::ZERO,
            last_bytes: 0,
            last_elapsed: Duration::ZERO,
        }
    }

    /// Update the rate calculator with new progress data.
    ///
    /// Returns the estimated time remaining, or None if fewer than 2 seconds
    /// of data are available.
    pub fn update(
        &mut self,
        bytes_transferred: u64,
        total_bytes: Option<u64>,
        elapsed: Duration,
    ) -> Option<Duration> {
        let time_delta = elapsed.saturating_sub(self.last_elapsed);
        let bytes_delta = bytes_transferred.saturating_sub(self.last_bytes);

        // Only update rate if meaningful time has passed
        if time_delta.as_millis() > 0 {
            let instant_rate = bytes_delta as f64 / time_delta.as_secs_f64();

            self.smoothed_rate = Some(match self.smoothed_rate {
                Some(prev) => prev * (1.0 - self.alpha) + instant_rate * self.alpha,
                None => instant_rate,
            });

            self.elapsed_with_data += time_delta;
        }

        self.last_bytes = bytes_transferred;
        self.last_elapsed = elapsed;

        // Require at least 2 seconds of data for a meaningful estimate
        if self.elapsed_with_data < Duration::from_secs(2) {
            return None;
        }

        // Calculate remaining time from smoothed rate and remaining bytes
        let rate = self.smoothed_rate?;
        if rate <= 0.0 {
            return None;
        }

        let remaining_bytes = total_bytes?.saturating_sub(bytes_transferred);
        if remaining_bytes == 0 {
            return Some(Duration::ZERO);
        }

        let remaining_secs = remaining_bytes as f64 / rate;
        Some(Duration::from_secs_f64(remaining_secs))
    }

    /// Reset the calculator (e.g., after a pause/resume).
    pub fn reset(&mut self) {
        self.smoothed_rate = None;
        self.elapsed_with_data = Duration::ZERO;
        self.last_bytes = 0;
        self.last_elapsed = Duration::ZERO;
    }
}

impl Default for RateCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_phase_display_produces_lowercase_strings() {
        // Validates: Requirement 2 AC 7
        assert_eq!(IoPhase::Queued.to_string(), "queued");
        assert_eq!(IoPhase::Reading.to_string(), "reading");
        assert_eq!(IoPhase::Writing.to_string(), "writing");
        assert_eq!(IoPhase::Finalizing.to_string(), "finalizing");
        assert_eq!(IoPhase::Cancelled.to_string(), "cancelled");
        assert_eq!(IoPhase::Failed.to_string(), "failed");
        assert_eq!(IoPhase::Complete.to_string(), "complete");
    }

    #[test]
    fn progress_state_new_queued_has_zero_values() {
        // Validates: Requirement 2 AC 1
        let state = ProgressState::new_queued();
        assert_eq!(state.bytes_transferred, 0);
        assert_eq!(state.total_bytes, None);
        assert_eq!(state.percentage, None);
        assert_eq!(state.elapsed, Duration::ZERO);
        assert_eq!(state.estimated_remaining, None);
        assert_eq!(state.phase, IoPhase::Queued);
    }

    #[test]
    fn calculate_percentage_with_known_total() {
        // Validates: Requirement 2 AC 3
        assert_eq!(ProgressState::calculate_percentage(0, Some(100)), Some(0));
        assert_eq!(ProgressState::calculate_percentage(50, Some(100)), Some(50));
        assert_eq!(
            ProgressState::calculate_percentage(100, Some(100)),
            Some(100)
        );
        assert_eq!(ProgressState::calculate_percentage(75, Some(200)), Some(37));
    }

    #[test]
    fn calculate_percentage_with_unknown_total() {
        // Validates: Requirement 2 AC 3
        assert_eq!(ProgressState::calculate_percentage(1000, None), None);
    }

    #[test]
    fn calculate_percentage_with_zero_total_returns_100() {
        // Validates: Requirement 2 AC 3
        assert_eq!(ProgressState::calculate_percentage(0, Some(0)), Some(100));
    }

    #[test]
    fn calculate_percentage_never_exceeds_100() {
        // Validates: Requirement 2 AC 3
        // bytes_transferred > total_bytes (shouldn't happen but handle gracefully)
        assert_eq!(
            ProgressState::calculate_percentage(200, Some(100)),
            Some(100)
        );
    }

    #[test]
    fn rate_calculator_returns_none_before_two_seconds() {
        // Validates: Requirement 2 AC 4
        let mut calc = RateCalculator::new();

        // Less than 2 seconds of data
        let result = calc.update(1000, Some(10000), Duration::from_millis(500));
        assert_eq!(result, None);

        let result = calc.update(2000, Some(10000), Duration::from_millis(1000));
        assert_eq!(result, None);

        let result = calc.update(3000, Some(10000), Duration::from_millis(1500));
        assert_eq!(result, None);
    }

    #[test]
    fn rate_calculator_returns_estimate_after_two_seconds() {
        // Validates: Requirement 2 AC 4
        let mut calc = RateCalculator::new();

        // Feed data points over more than 2 seconds
        calc.update(1000, Some(10000), Duration::from_millis(500));
        calc.update(2000, Some(10000), Duration::from_millis(1000));
        calc.update(3000, Some(10000), Duration::from_millis(1500));
        calc.update(4000, Some(10000), Duration::from_millis(2000));
        let result = calc.update(5000, Some(10000), Duration::from_millis(2500));

        assert!(result.is_some(), "should return estimate after >2s of data");
        let remaining = result.unwrap();
        assert!(
            remaining.as_secs() > 0,
            "should have non-zero remaining time"
        );
    }

    #[test]
    fn rate_calculator_returns_none_without_total_bytes() {
        // Validates: Requirement 2 AC 4
        let mut calc = RateCalculator::new();

        calc.update(1000, None, Duration::from_millis(500));
        calc.update(2000, None, Duration::from_millis(1000));
        calc.update(3000, None, Duration::from_millis(2000));
        let result = calc.update(4000, None, Duration::from_millis(2500));

        assert_eq!(result, None, "cannot estimate without total_bytes");
    }

    #[test]
    fn rate_calculator_returns_zero_when_transfer_complete() {
        // Validates: Requirement 2 AC 4
        let mut calc = RateCalculator::new();

        calc.update(5000, Some(10000), Duration::from_millis(1000));
        calc.update(8000, Some(10000), Duration::from_millis(2000));
        let result = calc.update(10000, Some(10000), Duration::from_millis(2500));

        assert_eq!(result, Some(Duration::ZERO));
    }

    #[test]
    fn rate_calculator_reset_clears_state() {
        // Validates: Requirement 2 AC 4
        let mut calc = RateCalculator::new();

        calc.update(5000, Some(10000), Duration::from_millis(1000));
        calc.update(8000, Some(10000), Duration::from_millis(2500));

        calc.reset();

        // After reset, should require 2 seconds of new data again
        let result = calc.update(1000, Some(10000), Duration::from_millis(500));
        assert_eq!(result, None);
    }

    #[test]
    fn progress_state_phases_cover_all_status_strings() {
        // Validates: Requirement 2 AC 7
        let phases = [
            IoPhase::Queued,
            IoPhase::Reading,
            IoPhase::Writing,
            IoPhase::Finalizing,
            IoPhase::Cancelled,
            IoPhase::Failed,
            IoPhase::Complete,
        ];
        let expected = [
            "queued",
            "reading",
            "writing",
            "finalizing",
            "cancelled",
            "failed",
            "complete",
        ];
        for (phase, expected_str) in phases.iter().zip(expected.iter()) {
            assert_eq!(phase.to_string(), *expected_str);
        }
    }
}
