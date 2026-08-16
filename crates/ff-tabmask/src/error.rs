//! Error types for the ff-tabmask crate.
//!
//! All errors follow the `[tabs-mask] operation: description` format per project standards.

use crate::artifacts::EditorMode;

/// Errors originating from the ff-tabmask crate.
///
/// Formatted per Error Message Standards: `[tabs-mask] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TabsMaskError {
    /// One or more column arguments are not valid positive integers.
    /// Addresses: Requirement 2, criterion 2.7
    #[error("[tabs-mask] parse: invalid tab stop — column positions must be positive integers: {invalid_values:?}")]
    InvalidTabStops {
        /// The invalid values that were provided.
        invalid_values: Vec<String>,
    },

    /// The TABS or MASK command was issued in a mode where it is not valid.
    #[error("[tabs-mask] execute: command '{command}' is not valid in {mode:?} mode")]
    InvalidMode {
        /// The command that was issued.
        command: String,
        /// The mode in which the command was issued.
        mode: EditorMode,
    },

    /// Mask editing attempted in Browse mode.
    /// Addresses: Requirement 6, criterion 6.11
    #[error("[tabs-mask] edit: mask line is not editable in Browse mode")]
    MaskNotEditable,

    /// No active mask when MASK OFF was issued.
    /// Addresses: Requirement 7, criterion 7.3
    #[error("[tabs-mask] clear: no active mask to clear")]
    NoMaskToClear,

    /// No active mask when MASK display was requested.
    /// Addresses: Requirement 6, criterion 6.2
    #[error(
        "[tabs-mask] display: no active mask — use MASK to set one or check the language profile"
    )]
    NoActiveMask,

    /// Configuration key has invalid format.
    /// Addresses: Requirement 4, criterion 4.6; Requirement 13, criterion 13.3
    #[error("[tabs-mask] config: invalid value in '{key}' — {reason}")]
    InvalidConfig {
        /// The configuration key with the invalid value.
        key: String,
        /// The reason the value is invalid.
        reason: String,
    },

    /// Line width exceeded during mask application.
    #[error(
        "[tabs-mask] apply: mask truncated at line width {line_width} (mask length: {mask_length})"
    )]
    MaskTruncated {
        /// The configured line width.
        line_width: usize,
        /// The length of the mask content.
        mask_length: usize,
    },

    /// Anchor line for display artifact is out of range.
    #[error("[tabs-mask] position: anchor line {anchor_line} out of range (document has {line_count} lines)")]
    AnchorOutOfRange {
        /// The anchor line that was specified.
        anchor_line: usize,
        /// The number of lines in the document.
        line_count: usize,
    },
}
