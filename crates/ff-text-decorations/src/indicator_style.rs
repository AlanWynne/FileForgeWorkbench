//! Indicator visual style enumeration.
//!
//! Defines the 23 indicator drawing styles adapted from Scintilla's
//! indicator rendering system to egui primitives.

/// Visual style for an indicator decoration.
///
/// Addresses: Requirement 1 AC 1–24
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IndicatorStyle {
    /// Solid horizontal underline beneath text.
    Plain,
    /// Wavy (zigzag) underline beneath text.
    Squiggle,
    /// Series of small "T" shapes beneath text.
    TT,
    /// Repeated diagonal line segments beneath text.
    Diagonal,
    /// Horizontal line through vertical centre (strikethrough).
    Strike,
    /// No visual output; occupies indicator storage for programmatic queries.
    Hidden,
    /// Rectangular outline around full text height.
    Box,
    /// Rounded-corner rectangle with semi-transparent fill and border.
    RoundBox,
    /// Square-corner rectangle with semi-transparent fill and border.
    StraightBox,
    /// Dashed horizontal underline beneath text.
    Dash,
    /// Dotted horizontal underline (individual square dots) beneath text.
    Dots,
    /// Low-amplitude squiggle (half height of standard Squiggle).
    SquiggleLow,
    /// Dotted rectangular outline around full line height.
    DotBox,
    /// Pre-computed anti-aliased squiggle pattern using RGBA pixel image.
    SquigglePixmap,
    /// Thick underline (2px at standard DPI) for IME composition ranges.
    CompositionThick,
    /// Thin underline (1px at standard DPI) for confirmed IME composition.
    CompositionThin,
    /// Square-corner rectangle extending full line height (top to bottom).
    FullBox,
    /// Overrides text foreground colour without additional graphical elements.
    TextFore,
    /// Small downward-pointing triangle at left edge of first decorated character.
    Point,
    /// Small downward-pointing triangle at horizontal centre of first decorated character.
    PointCharacter,
    /// Top-to-bottom gradient fill from indicator colour to transparent.
    Gradient,
    /// Gradient fill: transparent → indicator colour at centre → transparent.
    GradientCentre,
    /// Small downward-pointing triangle at top-left of first decorated character.
    PointTop,
}
