//! Error policy and retry logic for workflow step failures.
//!
//! Determines how step failures are handled: fail-fast (abort immediately),
//! continue-on-error (skip failed step), or retry (re-execute with delay).

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// The error handling strategy for step failures.
///
/// Addresses: Requirement 5, criterion 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorStrategy {
    /// Abort the workflow immediately on step failure.
    FailFast,
    /// Skip the failed step and continue to the next.
    ContinueOnError,
    /// Retry the step up to max_retries times, then fall back to FailFast.
    Retry,
}

/// Configures how step failures are handled within a workflow.
///
/// Addresses: Requirement 5, criteria 1/2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPolicy {
    /// The primary failure strategy.
    pub strategy: ErrorStrategy,
    /// Maximum retry count (only relevant with `Retry` strategy).
    pub max_retries: u32,
    /// Delay between retries.
    pub retry_delay: Duration,
    /// Whether to allow user interaction for error decisions.
    pub allow_user_interaction: bool,
}

impl Default for ErrorPolicy {
    fn default() -> Self {
        Self {
            strategy: ErrorStrategy::FailFast,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            allow_user_interaction: false,
        }
    }
}

impl ErrorPolicy {
    /// Creates a fail-fast policy (abort on first failure).
    pub fn fail_fast() -> Self {
        Self {
            strategy: ErrorStrategy::FailFast,
            ..Default::default()
        }
    }

    /// Creates a continue-on-error policy (skip failed steps).
    pub fn continue_on_error() -> Self {
        Self {
            strategy: ErrorStrategy::ContinueOnError,
            ..Default::default()
        }
    }

    /// Creates a retry policy with the given max retries and delay.
    pub fn retry(max_retries: u32, delay: Duration) -> Self {
        Self {
            strategy: ErrorStrategy::Retry,
            max_retries,
            retry_delay: delay,
            ..Default::default()
        }
    }

    /// Creates a retry policy with user interaction enabled.
    pub fn retry_with_interaction(max_retries: u32, delay: Duration) -> Self {
        Self {
            strategy: ErrorStrategy::Retry,
            max_retries,
            retry_delay: delay,
            allow_user_interaction: true,
        }
    }
}

/// Resolves the effective error policy for a step.
///
/// Per-step override takes precedence over the workflow-level default.
/// Addresses: Requirement 5, criterion 2
pub fn effective_policy(
    workflow_policy: &ErrorPolicy,
    step_override: Option<&ErrorPolicy>,
) -> ErrorPolicy {
    step_override
        .cloned()
        .unwrap_or_else(|| workflow_policy.clone())
}

/// Actions a user can take in response to a workflow error.
///
/// Addresses: Requirement 5, criterion 7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserErrorAction {
    /// Retry the failed step.
    Retry,
    /// Skip the failed step and continue.
    Skip,
    /// Abort the workflow.
    Abort,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 5.1 — ErrorPolicy enum strategies

    #[test]
    fn default_policy_is_fail_fast() {
        let policy = ErrorPolicy::default();
        assert_eq!(policy.strategy, ErrorStrategy::FailFast);
    }

    #[test]
    fn fail_fast_constructor() {
        let policy = ErrorPolicy::fail_fast();
        assert_eq!(policy.strategy, ErrorStrategy::FailFast);
        assert!(!policy.allow_user_interaction);
    }

    #[test]
    fn continue_on_error_constructor() {
        let policy = ErrorPolicy::continue_on_error();
        assert_eq!(policy.strategy, ErrorStrategy::ContinueOnError);
    }

    #[test]
    fn retry_constructor_sets_max_retries_and_delay() {
        let policy = ErrorPolicy::retry(5, Duration::from_millis(500));
        assert_eq!(policy.strategy, ErrorStrategy::Retry);
        assert_eq!(policy.max_retries, 5);
        assert_eq!(policy.retry_delay, Duration::from_millis(500));
    }

    // Validates: Requirement 5.2 — per-step override takes precedence

    #[test]
    fn effective_policy_returns_override_when_present() {
        let workflow_policy = ErrorPolicy::fail_fast();
        let step_override = ErrorPolicy::continue_on_error();
        let effective = effective_policy(&workflow_policy, Some(&step_override));
        assert_eq!(effective.strategy, ErrorStrategy::ContinueOnError);
    }

    #[test]
    fn effective_policy_returns_workflow_default_when_no_override() {
        let workflow_policy = ErrorPolicy::retry(3, Duration::from_secs(1));
        let effective = effective_policy(&workflow_policy, None);
        assert_eq!(effective.strategy, ErrorStrategy::Retry);
        assert_eq!(effective.max_retries, 3);
    }

    // Validates: Requirement 5.3 — retry with max_attempts

    #[test]
    fn retry_policy_has_default_three_retries() {
        let policy = ErrorPolicy::default();
        assert_eq!(policy.max_retries, 3);
    }

    #[test]
    fn retry_policy_has_default_one_second_delay() {
        let policy = ErrorPolicy::default();
        assert_eq!(policy.retry_delay, Duration::from_secs(1));
    }

    #[test]
    fn retry_with_interaction_enables_user_interaction() {
        let policy = ErrorPolicy::retry_with_interaction(2, Duration::from_millis(200));
        assert!(policy.allow_user_interaction);
        assert_eq!(policy.max_retries, 2);
    }

    #[test]
    fn error_policy_serialization_round_trip() {
        let policy = ErrorPolicy::retry(5, Duration::from_millis(750));
        let json = serde_json::to_string(&policy).expect("serialize");
        let restored: ErrorPolicy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.strategy, ErrorStrategy::Retry);
        assert_eq!(restored.max_retries, 5);
        assert_eq!(restored.retry_delay, Duration::from_millis(750));
    }
}
