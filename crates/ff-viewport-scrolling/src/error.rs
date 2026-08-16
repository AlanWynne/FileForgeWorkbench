//! Error types for the viewport-scrolling crate.
//!
//! All errors follow the `[viewport] operation: description` format per project
//! error message standards.

/// Errors originating from the viewport-scrolling crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ViewportError {
    /// Attempted to set visible_count to zero.
    #[error("[viewport] set_visible_count: visible_count must be at least 1")]
    ZeroVisibleCount,

    /// Attempted to set line_height to zero.
    #[error("[viewport] set_line_height: line_height must be at least 1")]
    ZeroLineHeight,

    /// Scroll target line is invalid (zero).
    #[error("[viewport] scroll_to_line: target line 0 is invalid (must be >= 1)")]
    InvalidScrollTarget,

    /// Display line mapper returned inconsistent data.
    #[error("[viewport] {operation}: display mapper inconsistency — {detail}")]
    MapperInconsistency {
        /// The operation that encountered the inconsistency.
        operation: String,
        /// Details about the inconsistency.
        detail: String,
    },

    /// Snapshot restoration encountered out-of-bounds values (clamping applied).
    #[error("[viewport] restore: snapshot field '{field}' value {value} exceeds document bounds (max: {max})")]
    SnapshotOutOfBounds {
        /// The field that was out of bounds.
        field: String,
        /// The value that was out of bounds.
        value: u64,
        /// The maximum allowed value.
        max: u64,
    },

    /// Configuration value is invalid.
    #[error(
        "[viewport] config: key '{key}' has invalid value '{value}' — using default {default}"
    )]
    InvalidConfig {
        /// The configuration key.
        key: String,
        /// The invalid value.
        value: String,
        /// The default value being used instead.
        default: String,
    },
}
