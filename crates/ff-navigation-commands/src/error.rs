//! Error types for the ff-navigation-commands crate.
//!
//! All errors follow the `[navigation] operation: description` format
//! per the Error Message Standards.

/// Errors originating from the navigation-commands crate.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum NavigationError {
    /// The line number is out of the valid range [1, line_count].
    #[error("[navigation] LOCATE: Line number out of range")]
    LineOutOfRange,

    /// The specified label does not exist in the current document.
    #[error("[navigation] LOCATE: Label not found: {label}")]
    LabelNotFound {
        /// The label that was not found.
        label: String,
    },

    /// The resolved SORT scope contains zero or one lines.
    #[error("[navigation] SORT: Nothing to sort")]
    NothingToSort,

    /// The bounds values are invalid (left < 1 or right <= left).
    #[error("[navigation] BOUNDS: Invalid bounds: left must be >= 1 and right must be > left")]
    InvalidBounds,

    /// A command argument could not be parsed.
    #[error("[navigation] {command}: {description}")]
    InvalidArgument {
        /// The command that received the invalid argument.
        command: String,
        /// Description of what went wrong.
        description: String,
    },

    /// A delegation command failed to dispatch.
    #[error("[navigation] delegation: {description}")]
    DelegationFailed {
        /// Description of the delegation failure.
        description: String,
    },
}
