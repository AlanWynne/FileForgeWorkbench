//! Abstract text measurement interface.
//!
//! The caching layer delegates actual platform-specific font measurement
//! to implementors of this trait. GUI shells (egui, test mocks) provide
//! concrete implementations.

use crate::types::{StyleSlot, XPosition};

/// Abstract text measurement interface.
///
/// The caching layer delegates actual platform-specific font measurement
/// to implementors of this trait. This ensures platform independence (NFR-4).
pub trait Surface: Send + Sync {
    /// Measure the x-positions of characters in `text` using the given style.
    ///
    /// `positions[i]` = the x-coordinate of the right edge of character `i`.
    /// The `positions` slice must have length equal to the number of characters in `text`.
    fn measure_text(&self, style: StyleSlot, text: &str, positions: &mut [XPosition]);

    /// Get the average character width for a style (for estimation).
    fn average_char_width(&self, style: StyleSlot) -> f64;

    /// Get the line height for the current font configuration.
    fn line_height(&self) -> f64;
}

/// A mock surface for testing that uses fixed-width character metrics.
pub struct MockSurface {
    /// Fixed character width in pixels.
    pub char_width: f64,
    /// Fixed line height in pixels.
    pub line_height: f64,
}

impl MockSurface {
    /// Create a mock surface with the given character width and line height.
    pub fn new(char_width: f64, line_height: f64) -> Self {
        Self {
            char_width,
            line_height,
        }
    }
}

impl Default for MockSurface {
    fn default() -> Self {
        Self::new(8.0, 16.0)
    }
}

impl Surface for MockSurface {
    fn measure_text(&self, _style: StyleSlot, text: &str, positions: &mut [XPosition]) {
        let mut x = 0.0;
        for (i, _ch) in text.chars().enumerate() {
            x += self.char_width;
            if i < positions.len() {
                positions[i] = XPosition(x);
            }
        }
    }

    fn average_char_width(&self, _style: StyleSlot) -> f64 {
        self.char_width
    }

    fn line_height(&self) -> f64 {
        self.line_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_surface_measures_fixed_width() {
        // Validates: NFR-4 (Platform Independence)
        let surface = MockSurface::default();
        let text = "ABC";
        let mut positions = vec![XPosition(0.0); 3];
        surface.measure_text(StyleSlot(0), text, &mut positions);
        assert!((positions[0].0 - 8.0).abs() < f64::EPSILON);
        assert!((positions[1].0 - 16.0).abs() < f64::EPSILON);
        assert!((positions[2].0 - 24.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mock_surface_average_char_width() {
        let surface = MockSurface::new(10.0, 20.0);
        assert_eq!(surface.average_char_width(StyleSlot(0)), 10.0);
    }

    #[test]
    fn surface_is_object_safe() {
        // Validates: NFR-4
        let _: Box<dyn Surface> = Box::new(MockSurface::default());
    }
}
