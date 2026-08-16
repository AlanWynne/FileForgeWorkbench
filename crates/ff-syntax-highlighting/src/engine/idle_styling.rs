//! Idle-time background styling configuration and result types.

/// Configuration for idle-time background styling.
/// Addresses: Requirement 9, criteria 9.3–9.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdleStylingConfig {
    /// Maximum lines to style per idle slice.
    pub lines_per_slice: usize,
    /// Maximum time budget per idle slice in milliseconds.
    pub time_budget_ms: u32,
}

impl Default for IdleStylingConfig {
    fn default() -> Self {
        Self {
            lines_per_slice: 256,
            time_budget_ms: 10,
        }
    }
}

/// Result of an idle styling increment.
/// Addresses: Requirement 9
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleStylingResult {
    /// More work remains; call again on next idle slice.
    MoreWork,
    /// All styling is complete; deregister from idle scheduler.
    Complete,
}
