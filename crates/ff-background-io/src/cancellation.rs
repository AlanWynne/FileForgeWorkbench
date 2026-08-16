//! Cooperative cancellation support for background I/O tasks.
//!
//! Defines [`IoCancellationToken`] — a wrapper around `tokio_util::sync::CancellationToken`
//! providing background-io-specific semantics: cooperative cancellation checked before
//! each chunk read/write.

use tokio_util::sync::CancellationToken;

/// Wrapper around `tokio_util::sync::CancellationToken` providing
/// background-io-specific semantics.
///
/// Cancellation is cooperative — tasks check this token before each chunk
/// read/write and terminate gracefully when triggered. No `tokio::task::abort()`
/// is ever used, ensuring all cleanup code always executes.
///
/// # Examples
///
/// ```
/// use ff_background_io::IoCancellationToken;
///
/// let token = IoCancellationToken::new();
/// assert!(!token.is_cancelled());
///
/// token.cancel();
/// assert!(token.is_cancelled());
/// ```
#[derive(Debug, Clone)]
pub struct IoCancellationToken {
    inner: CancellationToken,
}

impl IoCancellationToken {
    /// Create a new cancellation token.
    pub fn new() -> Self {
        Self {
            inner: CancellationToken::new(),
        }
    }

    /// Trigger cancellation. Returns immediately without waiting for the task to finish.
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.inner.is_cancelled()
    }

    /// Await cancellation (for use in `tokio::select!` patterns within tasks).
    pub async fn cancelled(&self) {
        self.inner.cancelled().await;
    }

    /// Create a child token that is cancelled when this token is cancelled.
    ///
    /// Useful for creating per-task tokens that inherit cancellation from
    /// a parent (e.g., shutdown token → individual task tokens).
    pub fn child_token(&self) -> Self {
        Self {
            inner: self.inner.child_token(),
        }
    }
}

impl Default for IoCancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_token_is_not_cancelled() {
        // Validates: Requirement 3 AC 1
        let token = IoCancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn cancel_sets_cancelled_state() {
        // Validates: Requirement 3 AC 1, AC 8
        let token = IoCancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn child_token_inherits_parent_cancellation() {
        // Validates: Requirement 3 AC 1
        let parent = IoCancellationToken::new();
        let child = parent.child_token();

        assert!(!child.is_cancelled());
        parent.cancel();
        assert!(child.is_cancelled());
    }

    #[test]
    fn child_token_can_be_cancelled_independently() {
        // Validates: Requirement 3 AC 1
        let parent = IoCancellationToken::new();
        let child = parent.child_token();

        child.cancel();
        assert!(child.is_cancelled());
        assert!(!parent.is_cancelled());
    }

    #[test]
    fn clone_shares_cancellation_state() {
        // Validates: Requirement 3 AC 1
        let token = IoCancellationToken::new();
        let clone = token.clone();

        token.cancel();
        assert!(clone.is_cancelled());
    }

    #[tokio::test]
    async fn cancelled_future_resolves_when_cancelled() {
        // Validates: Requirement 3 AC 8
        let token = IoCancellationToken::new();
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            token_clone.cancelled().await;
            true
        });

        // Give the task time to start
        tokio::task::yield_now().await;

        token.cancel();
        let result = handle.await.unwrap();
        assert!(result);
    }
}
