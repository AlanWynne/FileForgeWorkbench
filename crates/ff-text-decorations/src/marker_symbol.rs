//! Marker symbol enumeration.
//!
//! Defines the 31 geometric shapes available for line markers,
//! plus support for custom pixmap images.

/// Opaque identifier for a registered custom pixmap marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixmapId(pub u32);

/// Geometric shape for a line marker rendered in the gutter margin.
///
/// Addresses: Requirement 9 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MarkerSymbol {
    /// Filled circle.
    Circle,
    /// Rounded rectangle.
    RoundRect,
    /// Right-pointing arrow.
    Arrow,
    /// Small filled rectangle.
    SmallRect,
    /// Short right-pointing arrow.
    ShortArrow,
    /// No visible symbol (used for background-only markers).
    Empty,
    /// Downward-pointing arrow.
    ArrowDown,
    /// Minus sign (horizontal line).
    Minus,
    /// Plus sign (cross).
    Plus,
    /// Vertical line.
    VLine,
    /// L-shaped corner (bottom-left).
    LCorner,
    /// T-shaped corner (tee junction).
    TCorner,
    /// Box with plus sign inside (collapsed fold).
    BoxPlus,
    /// Box with plus and vertical connector line.
    BoxPlusConnected,
    /// Box with minus sign inside (expanded fold).
    BoxMinus,
    /// Box with minus and vertical connector line.
    BoxMinusConnected,
    /// Curved L-shaped corner.
    LCornerCurve,
    /// Curved T-shaped corner.
    TCornerCurve,
    /// Circle with plus sign inside.
    CirclePlus,
    /// Circle with plus and vertical connector.
    CirclePlusConnected,
    /// Circle with minus sign inside.
    CircleMinus,
    /// Circle with minus and vertical connector.
    CircleMinusConnected,
    /// Full background colour fill (no shape, just fills margin background).
    Background,
    /// Three dots (ellipsis).
    DotDotDot,
    /// Multiple arrows.
    Arrows,
    /// Full-height filled rectangle.
    FullRect,
    /// Left-aligned narrow rectangle.
    LeftRect,
    /// Underline beneath the marker row.
    Underline,
    /// Bookmark flag/page-corner shape.
    Bookmark,
    /// Vertical bookmark tab.
    VerticalBookmark,
    /// Vertical bar (narrow full-height line).
    Bar,
    /// Custom RGBA pixmap image.
    Pixmap(PixmapId),
}
