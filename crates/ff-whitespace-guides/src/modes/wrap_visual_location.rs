//! Wrap visual location enum.

use serde::{Deserialize, Serialize};

/// Controls positioning of wrap markers relative to text or display edge.
///
/// Addresses: Requirement 6 AC 6.6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrapVisualLocation {
    /// Markers placed at display edges (default).
    #[default]
    Default,
    /// End marker placed adjacent to last character.
    EndByText,
    /// Start marker placed adjacent to first character of continuation.
    StartByText,
}
