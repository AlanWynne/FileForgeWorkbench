//! Line marker configuration.
//!
//! Each marker number (0–31) has an associated configuration controlling
//! the visual symbol, colours, alpha, layer, and stroke width.

use crate::marker_symbol::MarkerSymbol;
use crate::ColourRGBA;

/// Rendering layer for markers.
///
/// Addresses: Requirement 9 AC 4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkerLayer {
    /// Render in the base layer (behind text).
    #[default]
    Base,
    /// Render in the overlay layer (above everything except selection).
    Overlay,
}

/// Complete configuration for a single marker number slot.
///
/// Addresses: Requirement 9 AC 2–6
#[derive(Debug, Clone, PartialEq)]
pub struct LineMarkerConfig {
    /// The geometric shape or pixmap to render.
    pub symbol: MarkerSymbol,
    /// Foreground colour (used for outlines and geometric shapes).
    pub fore: ColourRGBA,
    /// Background fill colour.
    pub back: ColourRGBA,
    /// Background colour when the line is selected.
    pub back_selected: ColourRGBA,
    /// Opacity (0–255).
    pub alpha: u8,
    /// Rendering layer (base or overlay).
    pub layer: MarkerLayer,
    /// Stroke width for geometric outlines.
    pub stroke_width: f32,
}

impl Default for LineMarkerConfig {
    fn default() -> Self {
        Self {
            symbol: MarkerSymbol::Circle,
            fore: ColourRGBA::new(0, 0, 0),
            back: ColourRGBA::new(255, 255, 255),
            back_selected: ColourRGBA::new(200, 200, 200),
            alpha: 255,
            layer: MarkerLayer::Base,
            stroke_width: 1.0,
        }
    }
}
