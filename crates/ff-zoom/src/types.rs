//! Core newtypes for the zoom subsystem.
//!
//! [`ZoomOffset`] is the fundamental type representing a signed integer
//! point-size offset applied to the base editor font.

/// A signed integer zoom offset in typographical points.
///
/// Positive values enlarge text; negative values shrink it. Zero means
/// no zoom (default rendering). The offset is clamped to a configured
/// range on construction.
///
/// # Examples
///
/// ```
/// use ff_zoom::ZoomOffset;
///
/// let offset = ZoomOffset::new(5, -10, 60);
/// assert_eq!(offset.value(), 5);
///
/// // Clamped to max
/// let clamped = ZoomOffset::new(100, -10, 60);
/// assert_eq!(clamped.value(), 60);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZoomOffset(i32);

impl ZoomOffset {
    /// Create a zoom offset, clamped to [min, max] inclusive.
    pub fn new(value: i32, min: i32, max: i32) -> Self {
        Self(value.clamp(min, max))
    }

    /// Create the zero (no-zoom) offset.
    pub fn zero() -> Self {
        Self(0)
    }

    /// Get the raw i32 value.
    pub fn value(self) -> i32 {
        self.0
    }

    /// Whether this offset represents the default (no zoom) state.
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Compute the effective font size given a base size in points.
    ///
    /// The result is always at least 1 — the rendered font size is never
    /// less than 1 point regardless of the offset value.
    ///
    /// # Arguments
    ///
    /// * `base_size` — The base font size in points (from theme configuration).
    pub fn effective_font_size(self, base_size: u32) -> u32 {
        let effective = base_size as i32 + self.0;
        effective.max(1) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.1 — ZoomOffset stores signed integer
    #[test]
    fn zoom_offset_stores_value() {
        let offset = ZoomOffset::new(5, -10, 60);
        assert_eq!(offset.value(), 5);
    }

    // Validates: Requirement 1.5 — clamping at max
    #[test]
    fn zoom_offset_clamps_at_max() {
        let offset = ZoomOffset::new(100, -10, 60);
        assert_eq!(offset.value(), 60);
    }

    // Validates: Requirement 1.5 — clamping at min
    #[test]
    fn zoom_offset_clamps_at_min() {
        let offset = ZoomOffset::new(-20, -10, 60);
        assert_eq!(offset.value(), -10);
    }

    // Validates: Requirement 1.4 — zero offset
    #[test]
    fn zoom_offset_zero_is_zero() {
        let offset = ZoomOffset::zero();
        assert_eq!(offset.value(), 0);
        assert!(offset.is_zero());
    }

    // Validates: Requirement 1.4 — is_zero only when value is 0
    #[test]
    fn zoom_offset_non_zero_is_not_zero() {
        let offset = ZoomOffset::new(3, -10, 60);
        assert!(!offset.is_zero());
    }

    // Validates: Requirement 1.2 — effective font size computation
    #[test]
    fn effective_font_size_adds_offset_to_base() {
        let offset = ZoomOffset::new(3, -10, 60);
        assert_eq!(offset.effective_font_size(12), 15);
    }

    // Validates: Requirement 1.2 — effective font size minimum is 1
    #[test]
    fn effective_font_size_never_less_than_one() {
        let offset = ZoomOffset::new(-10, -10, 60);
        // base 5, offset -10 → max(1, -5) = 1
        assert_eq!(offset.effective_font_size(5), 1);
    }

    // Validates: Requirement 1.2 — extreme negative offset
    #[test]
    fn effective_font_size_clamped_for_extreme_negative() {
        let offset = ZoomOffset::new(-10, -10, 60);
        // base 1, offset -10 → max(1, -9) = 1
        assert_eq!(offset.effective_font_size(1), 1);
    }

    // Validates: Requirement 1.5 — value within range stays unchanged
    #[test]
    fn zoom_offset_within_range_unchanged() {
        let offset = ZoomOffset::new(0, -10, 60);
        assert_eq!(offset.value(), 0);
    }

    // Validates: Requirement 1.5 — negative value within range
    #[test]
    fn zoom_offset_negative_within_range() {
        let offset = ZoomOffset::new(-5, -10, 60);
        assert_eq!(offset.value(), -5);
    }
}
