//! Cooperative cancellation support for workflow execution.
//!
//! The `CancellationToken` provides a cooperative signal that workflows
//! check between steps. It propagates to all async operations within a
//! workflow via token cloning and child token creation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;

/// A cooperative cancellation signal propagated to all async operations
/// within a workflow.
///
/// Steps should periodically check `is_cancelled()` and return early when
/// cancellation is requested. Async steps can use `cancelled().await` to
/// receive a notification when cancellation occurs.
///
/// Addresses: Requirement 3, criteria 1/4
#[derive(Debug, Clone)]
pub struct CancellationToken {
    inner: Arc<CancellationInner>,
}

#[derive(Debug)]
struct CancellationInner {
    cancelled: AtomicBool,
    notify: Notify,
    /// Default cancellation timeout for steps.
    timeout: Duration,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    /// Creates a new token that is not cancelled, with default 5-second timeout.
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(5))
    }

    /// Creates a new token with a custom cancellation timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            inner: Arc::new(CancellationInner {
                cancelled: AtomicBool::new(false),
                notify: Notify::new(),
                timeout,
            }),
        }
    }

    /// Creates a child token that shares the same cancellation state.
    ///
    /// When the parent is cancelled, the child is also considered cancelled
    /// (they share the same inner state via `Arc`).
    pub fn child(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Requests cancellation. All waiters are notified immediately.
    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Ordering::Release);
        self.inner.notify.notify_waiters();
    }

    /// Checks if cancellation has been requested. Non-blocking.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Acquire)
    }

    /// Returns a future that completes when cancellation is requested.
    ///
    /// If already cancelled, resolves immediately.
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.inner.notify.notified().await;
    }

    /// Returns the configured cancellation timeout for this token.
    pub fn timeout(&self) -> Duration {
        self.inner.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.1 — cooperative cancellation via CancellationToken

    #[test]
    fn new_token_is_not_cancelled() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn cancel_sets_cancelled_flag() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn child_token_shares_cancellation_state() {
        let parent = CancellationToken::new();
        let child = parent.child();
        assert!(!child.is_cancelled());
        parent.cancel();
        assert!(child.is_cancelled());
    }

    #[test]
    fn default_timeout_is_five_seconds() {
        let token = CancellationToken::new();
        assert_eq!(token.timeout(), Duration::from_secs(5));
    }

    #[test]
    fn custom_timeout_is_preserved() {
        let token = CancellationToken::with_timeout(Duration::from_secs(10));
        assert_eq!(token.timeout(), Duration::from_secs(10));
    }

    #[tokio::test]
    async fn cancelled_future_resolves_immediately_when_already_cancelled() {
        let token = CancellationToken::new();
        token.cancel();
        // Should return immediately without hanging
        token.cancelled().await;
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn cancelled_future_resolves_on_cancel_signal() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            token_clone.cancelled().await;
            true
        });

        // Give the task a moment to register the waiter
        tokio::time::sleep(Duration::from_millis(10)).await;
        token.cancel();

        let result = tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("should not timeout")
            .expect("task should complete");
        assert!(result);
    }

    #[test]
    fn multiple_cancels_are_idempotent() {
        let token = CancellationToken::new();
        token.cancel();
        token.cancel();
        assert!(token.is_cancelled());
    }
}
