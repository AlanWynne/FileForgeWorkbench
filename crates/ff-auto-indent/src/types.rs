//! Core types for the auto-indent subsystem.
//!
//! Contains the `IndentLevel` newtype that represents a logical indentation
//! depth, with safe arithmetic that clamps at zero.

/// A logical indentation level (number of indent units).
///
/// The level is always non-negative. Decrementing at zero returns zero.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndentLevel(u32);

impl IndentLevel {
    /// Create a new `IndentLevel` with the given value.
    pub fn new(level: u32) -> Self {
        Self(level)
    }

    /// Returns the raw numeric value of the indent level.
    pub fn value(self) -> u32 {
        self.0
    }

    /// Increment the indent level by one.
    pub fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Decrement the indent level by one, clamped at zero.
    pub fn decrement(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

impl std::fmt::Display for IndentLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_level_new_returns_given_value() {
        // Validates: Requirement 4.6 — IndentLevel never goes negative
        let level = IndentLevel::new(5);
        assert_eq!(level.value(), 5);
    }

    #[test]
    fn indent_level_default_is_zero() {
        // Validates: Requirement 4.6
        let level = IndentLevel::default();
        assert_eq!(level.value(), 0);
    }

    #[test]
    fn indent_level_increment_adds_one() {
        // Validates: Requirement 4.6
        let level = IndentLevel::new(3);
        assert_eq!(level.increment().value(), 4);
    }

    #[test]
    fn indent_level_decrement_subtracts_one() {
        // Validates: Requirement 4.6
        let level = IndentLevel::new(3);
        assert_eq!(level.decrement().value(), 2);
    }

    #[test]
    fn indent_level_decrement_clamps_at_zero() {
        // Validates: Requirement 4.6 — decrement at zero returns zero
        let level = IndentLevel::new(0);
        assert_eq!(level.decrement().value(), 0);
    }

    #[test]
    fn indent_level_increment_saturates_at_max() {
        // Validates: Requirement 4.6 — no overflow panic
        let level = IndentLevel::new(u32::MAX);
        assert_eq!(level.increment().value(), u32::MAX);
    }

    #[test]
    fn indent_level_display_shows_numeric_value() {
        let level = IndentLevel::new(7);
        assert_eq!(format!("{}", level), "7");
    }
}
