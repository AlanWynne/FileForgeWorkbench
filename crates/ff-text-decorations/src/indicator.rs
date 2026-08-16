//! Indicator configuration — style, colours, alpha, stroke width, flags.
//!
//! Each of the 44 indicator slots has a complete configuration controlling
//! how the decoration renders in both normal and hover states.

use crate::indicator_style::IndicatorStyle;
use crate::ColourRGBA;

/// Flags controlling indicator behaviour.
///
/// Addresses: Requirement 2 AC 8
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndicatorFlags {
    /// When true, colour is derived from the indicator value (lower 24 bits = RGB).
    pub value_fore: bool,
}

/// Style + colour state for normal or hover appearance.
///
/// Addresses: Requirement 2 AC 6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleAppearance {
    /// The visual style used for drawing.
    pub style: IndicatorStyle,
    /// The primary drawing colour.
    pub fore: ColourRGBA,
}

/// Complete configuration for a single indicator slot.
///
/// Addresses: Requirement 2 AC 1–9
#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorConfig {
    /// Normal-state appearance.
    pub normal: StyleAppearance,
    /// Hover-state appearance (if different from normal, indicator is "dynamic").
    pub hover: StyleAppearance,
    /// Whether indicator renders below text glyphs.
    pub under: bool,
    /// Interior fill opacity for box-style indicators (0–255, default 30).
    pub fill_alpha: u8,
    /// Border/outline opacity for box-style indicators (0–255, default 50).
    pub outline_alpha: u8,
    /// Line thickness in logical pixels (default 1.0).
    pub stroke_width: f32,
    /// Behaviour flags (ValueFore, etc.).
    pub flags: IndicatorFlags,
}

impl IndicatorConfig {
    /// Returns true when the hover state differs from normal state.
    ///
    /// Addresses: Requirement 2 AC 7
    pub fn is_dynamic(&self) -> bool {
        self.normal != self.hover
    }
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        let appearance = StyleAppearance {
            style: IndicatorStyle::Plain,
            fore: ColourRGBA::new(0, 0, 0),
        };
        Self {
            normal: appearance,
            hover: appearance,
            under: false,
            fill_alpha: 30,
            outline_alpha: 50,
            stroke_width: 1.0,
            flags: IndicatorFlags::default(),
        }
    }
}
