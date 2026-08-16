//! Wrap boundary types and resolution.
//!
//! Defines whether wrapping occurs at the viewport edge (dynamic) or at a
//! fixed column number (static).

/// A validated wrap column number.
///
/// Invariant: value is in range \[1, 10000\].
///
/// Addresses: Requirement 4 AC 5, AC 7
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct WrapColumn(u16);

impl WrapColumn {
    /// Maximum permitted wrap column value.
    pub const MAX: u16 = 10_000;

    /// Create a validated wrap column. Returns `None` if value is 0 or exceeds `MAX`.
    pub fn new(value: u16) -> Option<Self> {
        if (1..=Self::MAX).contains(&value) {
            Some(Self(value))
        } else {
            Option::None
        }
    }

    /// Get the raw column value.
    pub fn value(self) -> u16 {
        self.0
    }
}

/// The wrap boundary — determines at what column position wrapping occurs.
///
/// Addresses: Requirement 4 (Wrap Boundary)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum WrapBoundary {
    /// Dynamic wrapping at the current text area width.
    /// Wrap positions adjust as the window is resized.
    #[default]
    Viewport,

    /// Static wrapping at a fixed column number regardless of viewport width.
    Column(WrapColumn),
}

impl WrapBoundary {
    /// Create a `WrapBoundary` from a raw integer column value.
    ///
    /// - `0` → `Viewport`
    /// - `1..=10000` → `Column(n)`
    /// - Negative or `>10000` → `Viewport` (caller should emit a warning)
    ///
    /// Returns `(boundary, is_valid)` where `is_valid` is false if the value
    /// was out of range and the default was applied.
    pub fn from_column_value(value: i64) -> (Self, bool) {
        if value == 0 {
            (Self::Viewport, true)
        } else if value >= 1 && value <= i64::from(WrapColumn::MAX) {
            // Safe cast: we validated the range
            let col = WrapColumn(value as u16);
            (Self::Column(col), true)
        } else {
            (Self::Viewport, false)
        }
    }

    /// Resolve the effective wrap column in characters given a viewport width.
    ///
    /// - `Viewport` → returns `viewport_width_cols`
    /// - `Column(n)` → returns `n`
    pub fn effective_column(self, viewport_width_cols: u16) -> u16 {
        match self {
            Self::Viewport => viewport_width_cols,
            Self::Column(col) => col.value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_column_new_valid_range() {
        // Validates: Requirement 4.5
        assert!(WrapColumn::new(1).is_some());
        assert!(WrapColumn::new(80).is_some());
        assert!(WrapColumn::new(10_000).is_some());
    }

    #[test]
    fn wrap_column_new_zero_returns_none() {
        // Validates: Requirement 4.7
        assert!(WrapColumn::new(0).is_none());
    }

    #[test]
    fn wrap_column_new_exceeds_max_returns_none() {
        // Validates: Requirement 4.7
        assert!(WrapColumn::new(10_001).is_none());
        assert!(WrapColumn::new(u16::MAX).is_none());
    }

    #[test]
    fn wrap_column_value_roundtrip() {
        let col = WrapColumn::new(80).unwrap();
        assert_eq!(col.value(), 80);
    }

    #[test]
    fn wrap_boundary_default_is_viewport() {
        assert_eq!(WrapBoundary::default(), WrapBoundary::Viewport);
    }

    #[test]
    fn wrap_boundary_from_column_value_zero_is_viewport() {
        // Validates: Requirement 4.5
        let (boundary, valid) = WrapBoundary::from_column_value(0);
        assert_eq!(boundary, WrapBoundary::Viewport);
        assert!(valid);
    }

    #[test]
    fn wrap_boundary_from_column_value_positive_valid() {
        let (boundary, valid) = WrapBoundary::from_column_value(80);
        assert_eq!(boundary, WrapBoundary::Column(WrapColumn::new(80).unwrap()));
        assert!(valid);
    }

    #[test]
    fn wrap_boundary_from_column_value_negative_falls_back() {
        // Validates: Requirement 4.7
        let (boundary, valid) = WrapBoundary::from_column_value(-5);
        assert_eq!(boundary, WrapBoundary::Viewport);
        assert!(!valid);
    }

    #[test]
    fn wrap_boundary_from_column_value_exceeds_max_falls_back() {
        // Validates: Requirement 4.7
        let (boundary, valid) = WrapBoundary::from_column_value(10_001);
        assert_eq!(boundary, WrapBoundary::Viewport);
        assert!(!valid);
    }

    #[test]
    fn effective_column_viewport_returns_viewport_width() {
        // Validates: Requirement 4.2
        let boundary = WrapBoundary::Viewport;
        assert_eq!(boundary.effective_column(120), 120);
    }

    #[test]
    fn effective_column_fixed_returns_column_value() {
        // Validates: Requirement 4.3
        let boundary = WrapBoundary::Column(WrapColumn::new(80).unwrap());
        assert_eq!(boundary.effective_column(120), 80);
    }
}
