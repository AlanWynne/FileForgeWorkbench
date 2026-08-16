//! Indent guide mode enum.

use serde::{Deserialize, Serialize};

/// Controls which lines display indent guides.
///
/// Addresses: Requirement 3 AC 3.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndentGuideMode {
    /// No indent guides drawn (default).
    #[default]
    None,
    /// Guides only on lines with actual indentation at that column.
    Real,
    /// Extend guides through blank lines by scanning forward.
    LookForward,
    /// Extend guides through blank lines by scanning both directions.
    LookBoth,
}

impl IndentGuideMode {
    /// Cycle to the next mode in order:
    /// None → Real → LookForward → LookBoth → None
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Real,
            Self::Real => Self::LookForward,
            Self::LookForward => Self::LookBoth,
            Self::LookBoth => Self::None,
        }
    }

    /// Return all variants in cycling order.
    pub fn variants() -> &'static [Self] {
        &[Self::None, Self::Real, Self::LookForward, Self::LookBoth]
    }
}
