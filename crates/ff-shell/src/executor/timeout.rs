//! Timeout guard for command execution.
//!
//! Enforces configurable deadlines on running processes, triggering
//! termination when the timeout elapses.

use std::time::Duration;

/// Guards a running process against exceeding its configured timeout.
///
/// When the timeout elapses, the guard signals that the process should be
/// terminated. The timeout is disabled when `timeout_seconds` is 0.
#[derive(Debug, Clone)]
pub struct TimeoutGuard {
    /// The configured timeout duration.
    timeout: Duration,
    /// Whether the timeout is enabled (false when timeout_seconds == 0).
    enabled: bool,
}

impl TimeoutGuard {
    /// Creates a new timeout guard with the specified timeout in seconds.
    ///
    /// A value of 0 disables the timeout.
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_seconds),
            enabled: timeout_seconds > 0,
        }
    }

    /// Returns the configured timeout duration.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Returns whether the timeout is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns a future that resolves when the timeout expires.
    ///
    /// If the timeout is disabled, this future never resolves.
    pub async fn wait(&self) {
        if self.enabled {
            tokio::time::sleep(self.timeout).await;
        } else {
            // Never resolves — timeout disabled
            std::future::pending::<()>().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 18.1
    #[test]
    fn timeout_guard_with_positive_seconds_is_enabled() {
        let guard = TimeoutGuard::new(30);
        assert!(guard.is_enabled());
        assert_eq!(guard.timeout(), Duration::from_secs(30));
    }

    // Validates: Requirement 18.4
    #[test]
    fn timeout_guard_with_zero_is_disabled() {
        let guard = TimeoutGuard::new(0);
        assert!(!guard.is_enabled());
    }

    // Validates: Requirement 18.1
    #[tokio::test]
    async fn timeout_wait_resolves_after_duration() {
        let guard = TimeoutGuard::new(1); // 1 second timeout
        let start = std::time::Instant::now();
        guard.wait().await;
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(900));
        assert!(elapsed < Duration::from_secs(2));
    }
}
