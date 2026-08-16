//! Edge column indicator mode enum.

use serde::{Deserialize, Serialize};

/// The rendering style for the edge column indicator.
///
/// Addresses: Requirement 5 AC 5.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeMode {
    /// No edge indicator (default).
    #[default]
    None,
    /// Thin vertical line at the configured column.
    Line,
    /// Shaded background beyond the configured column.
    Background,
    /// Multiple vertical lines, each with its own column and colour.
    MultiLine,
}
