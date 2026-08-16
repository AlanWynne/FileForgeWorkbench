//! Retry policy for transient VFS errors.
//!
//! Defines [`RetryPolicy`] which implements exponential backoff retry logic
//! for transient errors (timeouts, network issues) during load operations.

use std::time::Duration;

use ff_vfs::VfsError;

/// Retry policy configuration for transient errors.
///
/// Supports exponential backoff with a configurable initial delay and multiplier.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_retries: u8,
    /// Initial backoff duration.
    pub initial_backoff: Duration,
    /// Backoff multiplier (exponential factor, typically 2.0).
    pub backoff_multiplier: f64,
}

impl RetryPolicy {
    /// Create a new retry policy with the given parameters.
    pub fn new(max_retries: u8, initial_backoff_ms: u64) -> Self {
        Self {
            max_retries,
            initial_backoff: Duration::from_millis(initial_backoff_ms),
            backoff_multiplier: 2.0,
        }
    }

    /// Calculate the backoff duration for a given attempt (0-indexed).
    pub fn backoff_for_attempt(&self, attempt: u8) -> Duration {
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        self.initial_backoff.mul_f64(multiplier)
    }

    /// Determine if a VFS error is transient (eligible for retry).
    ///
    /// Transient errors include timeouts and I/O errors that may be
    /// temporary (network issues, resource contention).
    pub fn is_transient(error: &VfsError) -> bool {
        matches!(error, VfsError::Timeout { .. } | VfsError::Io { .. })
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3, 500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_retry_policy_has_expected_values() {
        // Validates: Requirement 6 AC 7
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_backoff, Duration::from_millis(500));
        assert_eq!(policy.backoff_multiplier, 2.0);
    }

    #[test]
    fn backoff_follows_exponential_pattern() {
        // Validates: Requirement 6 AC 7
        let policy = RetryPolicy::new(5, 500);

        assert_eq!(policy.backoff_for_attempt(0), Duration::from_millis(500));
        assert_eq!(policy.backoff_for_attempt(1), Duration::from_millis(1000));
        assert_eq!(policy.backoff_for_attempt(2), Duration::from_millis(2000));
        assert_eq!(policy.backoff_for_attempt(3), Duration::from_millis(4000));
        assert_eq!(policy.backoff_for_attempt(4), Duration::from_millis(8000));
    }

    #[test]
    fn timeout_error_is_transient() {
        // Validates: Requirement 8 AC 6
        let error = VfsError::Timeout {
            uri: "vfs://test/file.txt".to_string(),
            operation: "read".to_string(),
            duration_ms: 30000,
        };
        assert!(RetryPolicy::is_transient(&error));
    }

    #[test]
    fn io_error_is_transient() {
        // Validates: Requirement 6 AC 7
        let error = VfsError::Io {
            uri: "vfs://test/file.txt".to_string(),
            operation: "read".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::ConnectionReset, "reset"),
        };
        assert!(RetryPolicy::is_transient(&error));
    }

    #[test]
    fn not_found_error_is_not_transient() {
        // Validates: Requirement 6 AC 7
        let error = VfsError::NotFound {
            uri: "vfs://test/missing.txt".to_string(),
            operation: "read".to_string(),
        };
        assert!(!RetryPolicy::is_transient(&error));
    }

    #[test]
    fn permission_denied_is_not_transient() {
        // Validates: Requirement 6 AC 7
        let error = VfsError::PermissionDenied {
            uri: "vfs://test/restricted.txt".to_string(),
            operation: "read".to_string(),
        };
        assert!(!RetryPolicy::is_transient(&error));
    }
}
