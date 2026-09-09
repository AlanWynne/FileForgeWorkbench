//! Requirement 11.6: calling `reconfigure` before the subsystem has been
//! initialized must be a safe no-op that returns `Fallback` and never panics.
//!
//! This lives in its own integration-test binary so that no other test in the
//! same process initializes the global singleton first (which would invalidate
//! the "before init" precondition).

use ff_logging::{reconfigure, LogConfig, LoggingStatus};

#[test]
fn reconfigure_before_init_is_safe_noop() {
    // Validates: Requirement 11.6
    let status = reconfigure(LogConfig::default());
    assert_eq!(
        status,
        LoggingStatus::Fallback,
        "reconfigure before init must return Fallback without panicking"
    );
}
