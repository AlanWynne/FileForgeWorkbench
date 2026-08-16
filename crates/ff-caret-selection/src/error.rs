//! Error types for the ff-caret-selection crate.
//!
//! All errors follow the `[caret] operation: description` format per
//! cross-cutting Requirement 8 (Error Message Standards).

/// Errors originating from the ff-caret-selection crate.
///
/// Formatted per Error Message Standards: `[caret] operation: description`
/// Messages are kept under 200 characters.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CaretSelectionError {
    /// Invalid caret width value (outside [1, 20] before clamping).
    #[error("[caret] set_width: value {value} is outside range [1, 20], clamped to {clamped}")]
    CaretWidthClamped {
        /// The original input value.
        value: u8,
        /// The clamped result.
        clamped: u8,
    },

    /// Invalid frame width (exceeds line_height / 3).
    #[error(
        "[caret] set_frame_width: value {value} exceeds max ({max}) for line height {line_height}"
    )]
    FrameWidthClamped {
        /// The original input value.
        value: u32,
        /// The maximum allowed.
        max: u32,
        /// The current line height.
        line_height: u32,
    },

    /// Configuration key has invalid value.
    #[error("[caret] config: key '{key}' has invalid value '{value}' — using default {default}")]
    InvalidConfig {
        /// The config key.
        key: String,
        /// The invalid value provided.
        value: String,
        /// The default being used instead.
        default: String,
    },

    /// Font metrics have zero or negative values.
    #[error(
        "[caret] render: invalid font metrics — char_width={char_width}, line_height={line_height}"
    )]
    InvalidFontMetrics {
        /// The char width provided.
        char_width: f32,
        /// The line height provided.
        line_height: f32,
    },
}
