//! Tab draw mode enum.

use serde::{Deserialize, Serialize};

/// The rendering style for visible tab characters.
///
/// Addresses: Requirement 2 AC 2.2, 2.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabDrawMode {
    /// Rightward arrow spanning the full tab width (default).
    #[default]
    LongArrow,
    /// Horizontal line through the vertical centre of the tab span.
    Strikeout,
}
