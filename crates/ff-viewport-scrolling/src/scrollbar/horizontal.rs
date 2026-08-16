//! Horizontal scrollbar model.
//!
//! Pure-function mapping between `horizontal_offset` and scrollbar fraction.

use crate::types::ScrollFraction;

/// Pure-function horizontal scrollbar model.
pub struct HorizontalScrollbar;

impl HorizontalScrollbar {
    /// Compute horizontal scrollbar position fraction.
    pub fn position_fraction(horizontal_offset: u64, max_horizontal_extent: u64) -> ScrollFraction {
        if max_horizontal_extent == 0 {
            return ScrollFraction::new(0.0);
        }
        let fraction = horizontal_offset as f64 / max_horizontal_extent as f64;
        ScrollFraction::new(fraction)
    }

    /// Convert a scrollbar fraction to a horizontal_offset value.
    pub fn fraction_to_offset(fraction: ScrollFraction, max_horizontal_extent: u64) -> u64 {
        let offset = (fraction.value() * max_horizontal_extent as f64).round() as u64;
        offset.min(max_horizontal_extent)
    }

    /// Whether the horizontal scrollbar should be disabled.
    pub fn is_disabled(max_horizontal_extent: u64) -> bool {
        max_horizontal_extent == 0
    }
}
