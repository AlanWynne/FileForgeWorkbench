//! Core newtypes for the viewport-scrolling crate.
//!
//! These types provide compile-time safety for commonly confused values
//! (line numbers, pixel offsets, fractions) while keeping the public API
//! expressive and self-documenting.

/// A 1-based display line number (accounts for wrapping/folding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayLine(pub u64);

/// A scrollbar position as a fraction in `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ScrollFraction(pub f64);

impl ScrollFraction {
    /// Create a clamped fraction in `[0.0, 1.0]`.
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Returns the inner f64 value.
    pub fn value(self) -> f64 {
        self.0
    }
}

/// A pixel offset for sub-line smooth scrolling.
/// Range: `[0, line_height)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PixelOffset(pub u32);

/// A column offset (1-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColumnOffset(pub u64);

/// Mouse wheel tick count (positive = scroll down/right, negative = scroll up/left).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WheelTicks(pub i32);

/// Scroll mode: line-level jumps or pixel-level smooth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollMode {
    /// Traditional whole-line scrolling (integer top_line values).
    #[default]
    Line,
    /// Pixel-level sub-line scrolling with animation targets.
    Smooth,
}
