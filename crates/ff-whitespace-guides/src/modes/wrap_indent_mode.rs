//! Wrap indent mode enum.

use serde::{Deserialize, Serialize};

/// Controls indentation of continuation sub-lines.
///
/// Addresses: Requirement 7 AC 7.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrapIndentMode {
    /// Fixed offset defined by Wrap_Start_Indent (default).
    #[default]
    Fixed,
    /// Same indentation as the first sub-line.
    Same,
    /// One additional tab stop beyond the first sub-line.
    Indent,
    /// Two additional tab stops beyond the first sub-line.
    DeepIndent,
}
