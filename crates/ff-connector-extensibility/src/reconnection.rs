//! Retry policy and reconnection management.
//!
//! Defines `RetryPolicy` — configurable reconnection behaviour — and
//! `ReconnectionManager` — a stateful manager that tracks attempts and
//! computes backoff durations with exponential growth.

use std::time::Duration;

use tokio_util::sync::CancellationToken;

/// Configures automatic reconnection behaviour for a connector.
///
/// Uses exponential backoff: each successive retry waits longer than the
/// previous one until `max_backoff` is reached, after which the interval
/// stays constant.
///
/// Addresses: Requirement 4 AC 4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 = no retry).
    pub max_retries: u32,
    /// Initial backoff duration between retries.
    pub initial_backoff: Duration,
    /// Maximum backoff duration (caps exponential growth).
    pub max_backoff: Duration,
    /// Whether to add random jitter to backoff intervals.
    pub use_jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            use_jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Compute the backoff duration for the given attempt number (0-indexed).
    ///
    /// Formula: min(initial_backoff × 2^attempt, max_backoff)
    /// Jitter is NOT applied here — callers add jitter separately if `use_jitter` is true.
    ///
    /// Addresses: Requirement 4 AC 5
    pub fn compute_backoff(&self, attempt: u32) -> Duration {
        let multiplier = 2u64.saturating_pow(attempt);
        let backoff = self.initial_backoff.saturating_mul(multiplier as u32);
        std::cmp::min(backoff, self.max_backoff)
    }

    /// Whether retries are allowed (max_retries > 0).
    pub fn allows_retry(&self) -> bool {
        self.max_retries > 0
    }
}

/// Manages automatic reconnection attempts for a connector
/// using exponential backoff with optional jitter.
///
/// Tracks the current attempt number and provides `next_backoff()` to
/// get the next wait duration. Call `reset()` when a connection succeeds.
///
/// Addresses: Requirement 4 AC 5
pub struct ReconnectionManager {
    /// The retry policy governing this manager.
    policy: RetryPolicy,
    /// Current retry attempt number (0-indexed).
    attempts: u32,
    /// Cancellation token for aborting reconnection.
    cancel: CancellationToken,
}

impl ReconnectionManager {
    /// Creates a new reconnection manager with the given policy.
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            policy,
            attempts: 0,
            cancel: CancellationToken::new(),
        }
    }

    /// Returns the next backoff duration, or `None` if max retries have been exhausted.
    ///
    /// Increments the attempt counter on each call.
    pub fn next_backoff(&mut self) -> Option<Duration> {
        if self.attempts >= self.policy.max_retries {
            return None;
        }
        let backoff = self.policy.compute_backoff(self.attempts);
        self.attempts += 1;
        Some(backoff)
    }

    /// Resets the attempt counter. Call this when a connection succeeds.
    pub fn reset(&mut self) {
        self.attempts = 0;
    }

    /// Returns the current attempt number.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// Returns a reference to the cancellation token.
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel
    }

    /// Cancel any ongoing reconnection attempts.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Returns a reference to the retry policy.
    pub fn policy(&self) -> &RetryPolicy {
        &self.policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4 AC 4
    #[test]
    fn default_retry_policy_has_expected_values() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_backoff, Duration::from_secs(1));
        assert_eq!(policy.max_backoff, Duration::from_secs(30));
        assert!(policy.use_jitter);
    }

    // Validates: Requirement 4 AC 5
    #[test]
    fn compute_backoff_exponential_growth() {
        let policy = RetryPolicy {
            max_retries: 10,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            use_jitter: false,
        };

        assert_eq!(policy.compute_backoff(0), Duration::from_secs(1));
        assert_eq!(policy.compute_backoff(1), Duration::from_secs(2));
        assert_eq!(policy.compute_backoff(2), Duration::from_secs(4));
        assert_eq!(policy.compute_backoff(3), Duration::from_secs(8));
        assert_eq!(policy.compute_backoff(4), Duration::from_secs(16));
        assert_eq!(policy.compute_backoff(5), Duration::from_secs(32));
    }

    // Validates: Requirement 4 AC 5
    #[test]
    fn compute_backoff_caps_at_max() {
        let policy = RetryPolicy {
            max_retries: 10,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(10),
            use_jitter: false,
        };

        // 2^4 = 16 > 10, so capped at 10
        assert_eq!(policy.compute_backoff(4), Duration::from_secs(10));
        assert_eq!(policy.compute_backoff(5), Duration::from_secs(10));
        assert_eq!(policy.compute_backoff(100), Duration::from_secs(10));
    }

    // Validates: Requirement 4 AC 5
    #[test]
    fn compute_backoff_monotonically_non_decreasing() {
        let policy = RetryPolicy {
            max_retries: 20,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
            use_jitter: false,
        };

        let mut prev = Duration::ZERO;
        for attempt in 0..20 {
            let current = policy.compute_backoff(attempt);
            assert!(
                current >= prev,
                "backoff decreased: attempt {attempt}, prev={prev:?}, current={current:?}"
            );
            prev = current;
        }
    }

    #[test]
    fn allows_retry_returns_true_when_max_retries_positive() {
        let policy = RetryPolicy {
            max_retries: 3,
            ..Default::default()
        };
        assert!(policy.allows_retry());
    }

    #[test]
    fn allows_retry_returns_false_when_max_retries_zero() {
        let policy = RetryPolicy {
            max_retries: 0,
            ..Default::default()
        };
        assert!(!policy.allows_retry());
    }

    // Validates: Requirement 4 AC 5
    #[test]
    fn reconnection_manager_exhausts_retries() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            use_jitter: false,
        };
        let mut mgr = ReconnectionManager::new(policy);

        assert_eq!(mgr.next_backoff(), Some(Duration::from_secs(1)));
        assert_eq!(mgr.next_backoff(), Some(Duration::from_secs(2)));
        assert_eq!(mgr.next_backoff(), Some(Duration::from_secs(4)));
        assert_eq!(mgr.next_backoff(), None); // exhausted
    }

    // Validates: Requirement 4 AC 5
    #[test]
    fn reconnection_manager_reset_allows_retries_again() {
        let policy = RetryPolicy {
            max_retries: 2,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            use_jitter: false,
        };
        let mut mgr = ReconnectionManager::new(policy);

        mgr.next_backoff();
        mgr.next_backoff();
        assert_eq!(mgr.next_backoff(), None);

        mgr.reset();
        assert_eq!(mgr.attempts(), 0);
        assert_eq!(mgr.next_backoff(), Some(Duration::from_secs(1)));
    }
}
