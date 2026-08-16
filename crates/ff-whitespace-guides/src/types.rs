//! Data types for query results — glyph positions, guide columns, edge info, etc.

use crate::modes::WrapIndentMode;
use crate::modes::WrapVisualLocation;
use serde::{Deserialize, Serialize};

/// RGBA colour representation.
///
/// Simple 8-bit-per-channel colour type used throughout the crate.
/// No GUI dependency — just data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ColourRGBA {
    /// Red channel (0–255).
    pub r: u8,
    /// Green channel (0–255).
    pub g: u8,
    /// Blue channel (0–255).
    pub b: u8,
    /// Alpha channel (0=transparent, 255=opaque).
    pub a: u8,
}

/// The type of whitespace glyph to render at a position.
///
/// Addresses: Requirement 2 AC 2.1, 2.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceGlyph {
    /// Centred dot for a space character.
    SpaceDot,
    /// Arrow spanning the full tab width.
    TabArrow {
        /// Width of the tab in character columns.
        width_chars: u32,
    },
    /// Horizontal strikeout through the tab span.
    TabStrikeout {
        /// Width of the tab in character columns.
        width_chars: u32,
    },
}

/// A whitespace glyph at a specific column position within a line.
///
/// Addresses: Requirement 9 AC 9.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphPosition {
    /// 0-based column within the line.
    pub column: u32,
    /// The glyph to render.
    pub glyph: WhitespaceGlyph,
}

/// A column + colour pair for multi-edge configurations.
///
/// Addresses: Requirement 5 AC 5.5
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeProperties {
    /// The column position (0-based character column).
    pub column: u32,
    /// The colour for this edge line.
    pub colour: ColourRGBA,
}

/// Edge column information for the viewport renderer.
///
/// Addresses: Requirement 5 AC 5.3–5.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeInfo {
    /// Single vertical line at the specified column.
    Line {
        /// Column position.
        column: u32,
        /// Line colour.
        colour: ColourRGBA,
    },
    /// Background shading beyond the specified column.
    Background {
        /// Column at which shading starts.
        column: u32,
        /// Shading colour.
        colour: ColourRGBA,
    },
    /// Multiple vertical lines at different columns.
    MultiLine {
        /// Ordered list of edge properties.
        edges: Vec<EdgeProperties>,
    },
}

/// The set of indent guide columns for a line.
///
/// Addresses: Requirement 3 AC 3.3–3.5, Requirement 4 AC 4.1–4.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentGuideInfo {
    /// Columns at which inactive guides should be drawn.
    pub guide_columns: Vec<u32>,
    /// The column of the active (highlighted) guide, if any.
    pub active_column: Option<u32>,
}

/// Information about wrap markers for a document line's sub-lines.
///
/// Addresses: Requirement 6 AC 6.1–6.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapMarkerInfo {
    /// Sub-line indices that need an end marker (continuing to next sub-line).
    pub end_markers: Vec<u32>,
    /// Sub-line indices that need a start marker (continuation from previous).
    pub start_markers: Vec<u32>,
    /// Whether a margin marker should appear for this document line.
    pub margin_marker: bool,
    /// Location positioning for markers.
    pub location: WrapVisualLocation,
}

/// Continuation sub-line indentation info.
///
/// Addresses: Requirement 7 AC 7.1–7.6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrapIndentInfo {
    /// Mode in use.
    pub mode: WrapIndentMode,
    /// Effective indentation in character widths for continuation sub-lines.
    pub indent_chars: u32,
    /// Whether the indent was clamped at 3/4 viewport width.
    pub clamped: bool,
}
