//! High-DPI pixel alignment utility.
//!
//! Provides coordinate snapping to device-pixel boundaries for
//! crisp decoration rendering at any scale factor.

/// High-DPI pixel alignment utility.
///
/// Addresses: Requirement 10 AC 1–8
pub struct PixelAligner {
    /// Display scale factor (e.g., 1.0, 1.5, 2.0).
    scale_factor: f32,
    /// Pixel divisions (1.0 / scale_factor) for sub-pixel snapping.
    pixel_division: f32,
}

impl PixelAligner {
    /// Create a new aligner for the given scale factor.
    pub fn new(scale_factor: f32) -> Self {
        let clamped = scale_factor.max(0.5);
        Self {
            scale_factor: clamped,
            pixel_division: 1.0 / clamped,
        }
    }

    /// Snap a coordinate to the nearest device-pixel boundary.
    ///
    /// Addresses: Requirement 10 AC 1
    pub fn align(&self, coord: f32) -> f32 {
        (coord * self.scale_factor).round() * self.pixel_division
    }

    /// Snap a rectangle outward to device-pixel boundaries.
    /// Returns (x, y, width, height) with all edges aligned.
    ///
    /// Addresses: Requirement 10 AC 4
    pub fn align_rect_outward(&self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
        let x0 = (x * self.scale_factor).floor() * self.pixel_division;
        let y0 = (y * self.scale_factor).floor() * self.pixel_division;
        let x1 = ((x + w) * self.scale_factor).ceil() * self.pixel_division;
        let y1 = ((y + h) * self.scale_factor).ceil() * self.pixel_division;
        (x0, y0, x1 - x0, y1 - y0)
    }

    /// Scale stroke width for the current DPI.
    ///
    /// Addresses: Requirement 10 AC 3
    pub fn scale_stroke(&self, logical_width: f32) -> f32 {
        let scaled = logical_width * self.scale_factor;
        // Round to nearest device pixel for crisp lines
        (scaled.round()).max(1.0) * self.pixel_division
    }

    /// Update the scale factor (e.g., when moving to a different monitor).
    pub fn set_scale_factor(&mut self, factor: f32) {
        let clamped = factor.max(0.5);
        self.scale_factor = clamped;
        self.pixel_division = 1.0 / clamped;
    }

    /// Get the current scale factor.
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_at_1x_rounds_to_integer() {
        // Validates: Requirement 10 AC 1
        let pa = PixelAligner::new(1.0);
        assert_eq!(pa.align(10.3), 10.0);
        assert_eq!(pa.align(10.7), 11.0);
        assert_eq!(pa.align(10.5), 11.0); // round half up
    }

    #[test]
    fn align_at_2x_snaps_to_half_pixel() {
        // Validates: Requirement 10 AC 1
        let pa = PixelAligner::new(2.0);
        // 10.3 * 2 = 20.6 → round to 21 → 21 / 2 = 10.5
        assert_eq!(pa.align(10.3), 10.5);
        // 10.1 * 2 = 20.2 → round to 20 → 20 / 2 = 10.0
        assert_eq!(pa.align(10.1), 10.0);
    }

    #[test]
    fn align_at_1_5x_snaps_to_third_pixel() {
        let pa = PixelAligner::new(1.5);
        // 10.0 * 1.5 = 15.0 → round to 15 → 15 / 1.5 = 10.0
        assert_eq!(pa.align(10.0), 10.0);
    }

    #[test]
    fn align_rect_outward_expands_to_pixel_boundaries() {
        // Validates: Requirement 10 AC 4
        let pa = PixelAligner::new(1.0);
        let (x, y, w, h) = pa.align_rect_outward(10.3, 20.7, 5.1, 3.2);
        // At 1x: floor(10.3)=10, floor(20.7)=20, ceil(15.4)=16, ceil(23.9)=24
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(w, 6.0); // 16 - 10
        assert_eq!(h, 4.0); // 24 - 20
    }

    #[test]
    fn scale_stroke_at_1x_returns_rounded() {
        // Validates: Requirement 10 AC 3
        let pa = PixelAligner::new(1.0);
        assert_eq!(pa.scale_stroke(1.0), 1.0);
        assert_eq!(pa.scale_stroke(1.5), 2.0);
    }

    #[test]
    fn scale_stroke_at_2x_scales_and_rounds() {
        let pa = PixelAligner::new(2.0);
        // 1.0 * 2.0 = 2.0 → round = 2.0 → 2.0 / 2.0 = 1.0
        assert_eq!(pa.scale_stroke(1.0), 1.0);
        // 0.5 * 2.0 = 1.0 → round = 1.0 → 1.0 / 2.0 = 0.5
        assert_eq!(pa.scale_stroke(0.5), 0.5);
    }

    #[test]
    fn set_scale_factor_updates_alignment() {
        let mut pa = PixelAligner::new(1.0);
        pa.set_scale_factor(2.0);
        assert_eq!(pa.scale_factor(), 2.0);
        assert_eq!(pa.align(10.3), 10.5);
    }

    #[test]
    fn scale_stroke_minimum_is_one_device_pixel() {
        let pa = PixelAligner::new(2.0);
        // Very thin stroke: 0.1 * 2.0 = 0.2 → round = 0 → max(1.0) → 1.0 / 2.0 = 0.5
        assert_eq!(pa.scale_stroke(0.1), 0.5);
    }
}
