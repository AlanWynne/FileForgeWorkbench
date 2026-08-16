//! Whitespace visibility mode enum.

use serde::{Deserialize, Serialize};

/// Controls when whitespace characters are rendered with visible glyphs.
///
/// Addresses: Requirement 1 AC 1.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhitespaceVisibility {
    /// No whitespace glyphs rendered (default).
    #[default]
    Invisible,
    /// All spaces and tabs rendered.
    VisibleAlways,
    /// Only spaces/tabs after the first non-whitespace character per line.
    VisibleAfterIndent,
    /// Only leading spaces/tabs before the first non-whitespace character.
    VisibleOnlyInIndent,
}

impl WhitespaceVisibility {
    /// Cycle to the next mode in order:
    /// Invisible → VisibleAlways → VisibleAfterIndent → VisibleOnlyInIndent → Invisible
    pub fn next(self) -> Self {
        match self {
            Self::Invisible => Self::VisibleAlways,
            Self::VisibleAlways => Self::VisibleAfterIndent,
            Self::VisibleAfterIndent => Self::VisibleOnlyInIndent,
            Self::VisibleOnlyInIndent => Self::Invisible,
        }
    }

    /// Return all variants in cycling order.
    pub fn variants() -> &'static [Self] {
        &[
            Self::Invisible,
            Self::VisibleAlways,
            Self::VisibleAfterIndent,
            Self::VisibleOnlyInIndent,
        ]
    }
}
